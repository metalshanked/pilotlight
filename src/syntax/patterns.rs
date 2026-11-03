//! Bounded structural patterns. No target code is evaluated or macro-expanded.
use super::*;

fn binding(canonical: &str, kind: Option<&str>) -> Binding {
    Binding { canonical: canonical.into(), kind: kind.map(str::to_owned), asset: None, resolution_path: vec![], wrapper: false, namespace: None }
}

impl Scanner<'_> {
    pub(super) fn configured_tool_loop(&mut self, node: Node<'_>, b: &Binding, bindings: &Bindings) {
        if !matches!(self.language,"javascript"|"typescript") { return; }
        let Some(entry)=self.catalog.import(self.language,&b.canonical) else {return;};
        let Some(rule)=entry.configured_loops.get(&b.canonical) else {return;};
        if !self.catalog.import(self.language,&b.canonical).is_some_and(|e|self.catalog.symbol_kind(e,self.language,&b.canonical)==Some("llm_call")) { return; }
        let Some(args)=node.child_by_field_name("arguments") else { return; };
        let Some(tools)=argument(args,self.source,&rule.tools_argument) else { return; };
        if matches!(tools.kind(),"null"|"undefined") || (tools.kind()=="object" && tools.named_child_count()==0) { return; }
        let multiple=argument(args,self.source,&rule.count_argument).is_some_and(|n|self.text(n).parse::<u32>().is_ok_and(|n|n>1))
            || argument(args,self.source,&rule.stop_argument).is_some_and(|condition| {
                self.call_binding(condition,bindings).is_some_and(|b|rule.stop_functions.contains(&b.canonical))
                && condition.child_by_field_name("arguments").and_then(|a|a.named_child(0)).is_some_and(|n|self.text(n).parse::<u32>().is_ok_and(|n|n>1))
            });
        if !multiple { return; }
        let mut a=Asset::new("agent",&b.canonical,Some(&entry.id),self.evidence(node,"ast_configured_tool_loop",&b.canonical));
        a.subtype=Some("configured_tool_loop".into()); a.confidence="inferred_from_configuration".into();
        self.result.assets.push(a);
    }
    pub(super) fn csharp_namespace(&self, namespace: &str, bindings: &mut Bindings) {
        bindings.insert(format!("@using:{namespace}"), binding(namespace, None));
        let names: &[&str] = match namespace {
            "Microsoft.Extensions.DependencyInjection" => &["ServiceCollection", "IServiceCollection"],
            "Microsoft.Extensions.Hosting" => &["Host", "HostApplicationBuilder"],
            "Microsoft.AspNetCore.Builder" => &["WebApplication"],
            _ => return,
        };
        bindings.insert(format!("@using:{namespace}"), binding(namespace, None));
        for name in names { bindings.insert((*name).into(), binding(&format!("{namespace}.{name}"), None)); }
    }

    pub(super) fn csharp_parameters(&self, node: Node<'_>, bindings: &mut Bindings) {
        let Some(parameters) = node.child_by_field_name("parameters") else { return; };
        let mut cursor = parameters.walk();
        for parameter in parameters.named_children(&mut cursor) {
            if let (Some(ty), Some(name)) = (parameter.child_by_field_name("type"), parameter.child_by_field_name("name")) {
                if self.resolve(self.text(ty), bindings).is_some_and(|b| b.canonical == "Microsoft.Extensions.DependencyInjection.IServiceCollection") {
                    bindings.insert(self.text(name).into(), binding("Microsoft.Extensions.DependencyInjection.IServiceCollection", Some("service_collection")));
                }
            }
        }
    }

    pub(super) fn csharp_binding(&self, node: Node<'_>, bindings: &Bindings, depth: usize) -> Option<Binding> {
        if depth > 12 { return None; }
        if node.kind() == "identifier" { return self.resolve(self.text(node), bindings); }
        if node.kind() == "member_access_expression" {
            let object = node.child_by_field_name("expression")?;
            let name = node.child_by_field_name("name")?;
            let base = self.csharp_binding(object, bindings, depth + 1)?;
            return (self.text(name) == "Services" && base.kind.as_deref() == Some("host_builder"))
                .then(|| binding("Microsoft.Extensions.DependencyInjection.IServiceCollection", Some("service_collection")));
        }
        let callable = self.callable(node)?;
        if let Some(b) = self.resolve(self.text(callable), bindings) {
            match b.canonical.as_str() {
                "Microsoft.Extensions.DependencyInjection.ServiceCollection" => return Some(binding(&b.canonical, Some("service_collection"))),
                "Microsoft.Extensions.Hosting.Host.CreateApplicationBuilder" | "Microsoft.Extensions.Hosting.HostApplicationBuilder" |
                "Microsoft.AspNetCore.Builder.WebApplication.CreateBuilder" => return Some(binding(&b.canonical, Some("host_builder"))),
                _ => {}
            }
        }
        let object = callable.child_by_field_name("expression")?;
        let name = callable.child_by_field_name("name")?;
        let candidates=self.catalog.entries.iter().flat_map(|e| e.di_registrations.iter().map(move |symbol|(e,symbol)))
             .filter(|(_,symbol)|symbol.rsplit('.').next()==Some(self.text(name))
                && symbol.rsplit_once('.').and_then(|(class,_)|class.rsplit_once('.')).is_some_and(|(ns,_)|bindings.contains_key(&format!("@using:{ns}")))).collect::<Vec<_>>();
        if candidates.len()!=1 {return None;}
        let (entry,symbol)=candidates[0];
        let base = self.csharp_binding(object, bindings, depth + 1)?;
        (base.kind.as_deref() == Some("service_collection"))
            .then(|| binding(symbol, self.catalog.symbol_kind(entry,"csharp",symbol)))
    }

    pub(super) fn rust_annotation(&mut self, node: Node<'_>, bindings: &Bindings) {
        if node.kind() != "attribute_item" { return; }
        let Some(attribute) = node.named_child(0) else { return; };
        let Some(name) = attribute.named_child(0) else { return; };
        let raw = self.text(name);
        let Some(b) = self.resolve(raw, bindings) else { return; };
        let Some(entry) = self.catalog.import("rust", &b.canonical) else { return; };
        let Some(kind) = self.catalog.symbol_kind(entry,"rust", &b.canonical) else { return; };
        let Some(rule)=entry.rust_attributes.get(&b.canonical) else {return;};
        // Outer attributes are sibling nodes in the Rust grammar. Skip adjacent
        // attributes/comments, but never search a different declaration or scope.
        let mut target = node.next_named_sibling();
        while target.is_some_and(|n| matches!(n.kind(), "attribute_item" | "line_comment" | "block_comment")) { target = target.unwrap().next_named_sibling(); }
        let Some(target) = target else { return; };
        if rule.target == "impl_method" {
            if target.kind() != "function_item" || !node.parent().is_some_and(|p| p.kind() == "declaration_list" && p.parent().is_some_and(|p|p.kind()=="impl_item")) { return; }
        } else if rule.target == "trait_impl" {
            let Some(trait_name) = target.child_by_field_name("trait") else { return; };
            if !self.resolve(self.text(trait_name), bindings).is_some_and(|b| Some(&b.canonical) == rule.trait_symbol.as_ref()) { return; }
        } else {
            // A router alone is a tool collection, not proof of a server. The
            // explicit server_handler flag is required for this combined form.
            let flag = attribute.child_by_field_name("arguments").is_some_and(|args| {
                let mut cursor=args.walk(); args.named_children(&mut cursor).any(|n|n.kind()=="identifier" && Some(self.text(n))==rule.flag.as_deref())
            });
            if target.kind() != "impl_item" || !flag { return; }
        }
        let mut asset=Asset::new(kind,&b.canonical,Some(&entry.id),self.evidence(node,"ast_macro_attribute",&b.canonical));
        asset.confidence="inferred_from_import".into();
        asset.subtype=Some(if kind=="tool" {"macro_tool_definition"} else {"macro_server_definition"}.into());
        self.result.assets.push(asset);
    }

    pub(super) fn agent_loops(&mut self, root: Node<'_>) {
        if !matches!(self.language,"python"|"javascript"|"typescript") { return; }
        let mut pending=vec![(root,0usize)];
        while let Some((node,depth))=pending.pop() {
            if depth>256 { self.result.parse_errors=true; continue; }
            if matches!(node.kind(),"while_statement"|"for_statement"|"for_in_statement"|"do_statement") { self.agent_loop(node); }
            let mut c=node.walk(); for child in node.named_children(&mut c) { pending.push((child,depth+1)); }
        }
    }

    fn agent_loop(&mut self, loop_node: Node<'_>) {
        if !self.result.assets.iter().any(|a|a.kind=="llm_call" && a.evidence[0].line>=loop_node.start_position().row+1 && a.evidence[0].line<=loop_node.end_position().row+1) { return; }
        let Some(nodes)=scope_nodes(loop_node) else { self.result.parse_errors=true; return; };
        for call in nodes.iter().copied().filter(|n|matches!(n.kind(),"call"|"call_expression")) {
            let Some(model)=self.result.assets.iter().find(|a|a.kind=="llm_call" && a.evidence[0].line==call.start_position().row+1 && a.evidence[0].column==call.start_position().column+1).cloned() else { continue; };
            let Some((response,_))=assigned(call,self.source) else { continue; };
            let Some(args)=call.child_by_field_name("arguments") else { continue; };
            let Some(history)=argument(args,self.source,"messages").or_else(||argument(args,self.source,"input")) else { continue; };
            if !matches!(history.kind(),"identifier"|"shorthand_property_identifier") || argument(args,self.source,"tools").is_none() { continue; }
            let history=self.text(history);
            for iteration in nodes.iter().copied().filter(|n|matches!(n.kind(),"for_statement"|"for_in_statement") && n.start_byte()>call.end_byte()) {
                let Some(item)=iteration.child_by_field_name("left") else { continue; };
                let Some(collection)=iteration.child_by_field_name("right") else { continue; };
                if item.kind()!="identifier" { continue; }
                let item=self.text(item);
                let collection=self.text(collection);
                if !(collection.starts_with(&format!("{response}.")) && ["tool_calls","output","content"].iter().any(|s|collection.ends_with(s))) { continue; }
                if reassigned(&nodes,self.source,response,call.end_byte(),iteration.start_byte()) { continue; }
                let Some(body)=iteration.child_by_field_name("body") else { continue; };
                let Some(body_nodes)=scope_nodes(body) else { continue; };
                for dispatch in body_nodes.iter().copied().filter(|n|matches!(n.kind(),"call"|"call_expression")) {
                    let Some((result,_))=assigned(dispatch,self.source) else { continue; };
                    let Some(function)=dispatch.child_by_field_name("function") else { continue; };
                    // Parsing arguments is not tool execution. Require a tool item
                    // or its name in the dispatch expression, not arguments alone.
                    if matches!(self.text(function),"json.loads"|"JSON.parse"|"str"|"print"|"console.log") { continue; }
                    let Some(dispatch_nodes)=scope_nodes(dispatch) else { continue; };
                    let calls_item=dispatch_nodes.iter().any(|n|n.kind()=="identifier" && self.text(*n)==item)
                        && (dispatch_nodes.iter().any(|n|matches!(n.kind(),"attribute"|"member_expression") && [format!("{item}.name"),format!("{item}.function.name")].contains(&self.text(*n).to_owned())) || dispatch.child_by_field_name("arguments").is_some_and(|a|a.named_child(0).is_some_and(|n|n.kind()=="identifier" && self.text(n)==item)));
                    if !calls_item { continue; }
                    for feedback in body_nodes.iter().copied().filter(|n|matches!(n.kind(),"call"|"call_expression") && n.start_byte()>dispatch.end_byte()) {
                        let Some(function)=feedback.child_by_field_name("function") else { continue; };
                        if ![format!("{history}.append"),format!("{history}.push")].contains(&self.text(function).to_owned()) { continue; }
                        let Some(args)=feedback.child_by_field_name("arguments") else { continue; };
                        let Some(payload)=args.named_child(0).filter(|n|matches!(n.kind(),"dictionary"|"object")) else { continue; };
                        if reassigned(&nodes,self.source,history,call.end_byte(),feedback.start_byte()) || reassigned(&body_nodes,self.source,item,body.start_byte(),feedback.start_byte()) || reassigned(&body_nodes,self.source,result,dispatch.end_byte(),feedback.start_byte()) { continue; }
                        if !valid_feedback(payload,self.source,item,result) { continue; }
                        let mut a=Asset::new("agent","tool_calling_loop",model.framework.as_deref(),self.evidence(loop_node,"ast_tool_loop","tool_calling_loop"));
                        a.confidence="inferred_from_control_flow".into(); a.subtype=Some("tool_calling_loop_candidate".into());
                        a.evidence.push(model.evidence[0].clone());
                        a.evidence.push(self.evidence(dispatch,"ast_tool_dispatch","tool_dispatch"));
                        a.evidence.push(self.evidence(feedback,"ast_tool_feedback","tool_result_feedback"));
                        self.result.relationships.push(Relationship{from:a.id.clone(),to:model.id.clone(),kind:"uses".into()});
                        self.result.assets.push(a); return;
                    }
                }
            }
        }
    }
}

fn reassigned(nodes: &[Node<'_>], source: &str, name: &str, start: usize, end: usize) -> bool {
    nodes.iter().any(|n| {
        if n.start_byte()<start || n.start_byte()>=end || !matches!(n.kind(),"assignment"|"assignment_expression"|"variable_declarator"|"augmented_assignment") { return false; }
        n.child_by_field_name("left").or_else(||n.child_by_field_name("name")).is_some_and(|lhs|lhs.kind()=="identifier" && &source[lhs.byte_range()]==name)
    })
}

fn scope_nodes(node: Node<'_>) -> Option<Vec<Node<'_>>> {
    let mut out=vec![]; let mut pending=vec![(node,0usize)];
    while let Some((n,depth))=pending.pop() {
        if depth>128 || out.len()>=20_000 { return None; }
        if n!=node && matches!(n.kind(),"function_definition"|"function_declaration"|"arrow_function"|"lambda"|"class_definition"|"class_declaration") { continue; }
        out.push(n);
        if matches!(n.kind(),"comment"|"string"|"string_literal"|"template_string") { continue; }
        let mut c=n.walk(); for child in n.named_children(&mut c) { pending.push((child,depth+1)); }
    }
    Some(out)
}

fn assigned<'a>(call: Node<'_>, source: &'a str) -> Option<(&'a str,usize)> {
    let mut n=call;
    if n.parent().is_some_and(|p|matches!(p.kind(),"await"|"await_expression")) { n=n.parent()?; }
    let p=n.parent()?;
    if !matches!(p.kind(),"assignment"|"variable_declarator"|"assignment_expression") { return None; }
    let lhs=p.child_by_field_name("left").or_else(||p.child_by_field_name("name"))?;
    (lhs.kind()=="identifier").then(||(&source[lhs.byte_range()],p.start_byte()))
}

fn argument<'a>(args: Node<'a>, source: &str, key: &str) -> Option<Node<'a>> {
    let mut c=args.walk();
    for n in args.named_children(&mut c) {
        if n.kind()=="keyword_argument" && n.child_by_field_name("name").is_some_and(|k| &source[k.byte_range()]==key) { return n.child_by_field_name("value"); }
        if n.kind()=="object" {
            let mut c=n.walk(); for p in n.named_children(&mut c) {
                if p.kind()=="shorthand_property_identifier" && &source[p.byte_range()]==key { return Some(p); }
                if p.kind()=="pair" && p.child_by_field_name("key").is_some_and(|k|unquote(&source[k.byte_range()])==key) { return p.child_by_field_name("value"); }
            }
        }
    }
    None
}

fn valid_feedback(payload: Node<'_>, source: &str, item: &str, result: &str) -> bool {
    let Some(nodes)=scope_nodes(payload) else { return false; };
    nodes.iter().copied().filter(|n|matches!(n.kind(),"dictionary"|"object")).any(|object| {
        let value=|key: &str| {
            let mut c=object.walk();
            object.named_children(&mut c).find(|n|n.kind()=="pair" && n.child_by_field_name("key").is_some_and(|k|unquote(&source[k.byte_range()])==key)).and_then(|n|n.child_by_field_name("value"))
        };
        for (key,tag,id_key,out_key,id_field) in [("role","tool","tool_call_id","content","id"),("type","function_call_output","call_id","output","call_id"),("type","tool_result","tool_use_id","content","id")] {
            if !value(key).is_some_and(|v| matches!(v.kind(),"string"|"string_literal") && unquote(&source[v.byte_range()])==tag) { continue; }
            if !value(id_key).is_some_and(|v|matches!(v.kind(),"attribute"|"member_expression") && source[v.byte_range()]==format!("{item}.{id_field}")) { continue; }
            if value(out_key).and_then(scope_nodes).is_some_and(|ns|ns.iter().any(|n|n.kind()=="identifier" && &source[n.byte_range()]==result)) { return true; }
        }
        false
    })
}
