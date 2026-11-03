//! Bounded Python dataclass constructor mapping; class names are not a detector.
use super::*;
impl Scanner<'_> {
    pub(super) fn model_record_declaration(&mut self, node: Node<'_>, bindings: &mut Bindings, depth: usize) -> bool {
        if self.language!="python" || node.kind()!="decorated_definition" { return false; }
        let Some(class)=node.child_by_field_name("definition").filter(|n|n.kind()=="class_definition") else { return false; };
        let name=class.child_by_field_name("name").map(|n|self.text(n).to_owned());
        let mut valid=class.child_by_field_name("superclasses").is_none();
        let mut found=false;
        let mut cursor=node.walk();
        for decorator in node.named_children(&mut cursor).filter(|n|n.kind()=="decorator") {
            let Some(expr)=decorator.named_child(0) else {valid=false;continue};
            let function=expr.child_by_field_name("function").unwrap_or(expr);
            if !self.resolve(self.text(function), bindings).is_some_and(|b|b.canonical=="dataclasses.dataclass") {valid=false;continue;}
            found=true;
            // Only literal options known not to alter constructor argument order.
            if let Some(args)=expr.child_by_field_name("arguments") {
                let mut c=args.walk();
                for option in args.named_children(&mut c) {
                    let key=option.child_by_field_name("name").map(|n|self.text(n));
                    let value=option.child_by_field_name("value").map(|n|self.text(n));
                    if !matches!(key,Some("frozen"|"slots"|"repr"|"eq"|"order"|"unsafe_hash"|"match_args"|"weakref_slot")) || !matches!(value,Some("True"|"False")) {valid=false;}
                }
            }
        }
        let mut fields=vec![];
        if let Some(body)=class.child_by_field_name("body") {
            let mut c=body.walk();
            for statement in body.named_children(&mut c) {
                if matches!(statement.kind(),"comment"|"pass_statement") {continue;}
                if statement.kind()=="function_definition" {
                    if statement.child_by_field_name("name").is_some_and(|n|matches!(self.text(n),"__init__"|"__new__")) {valid=false;}
                    continue;
                }
                if statement.kind()=="decorated_definition" {
                    if !statement.child_by_field_name("definition").is_some_and(|n|n.kind()=="function_definition" && n.child_by_field_name("name").is_some_and(|name|!matches!(self.text(name),"__init__"|"__new__"))) {valid=false;}
                    continue;
                }
                let assignment=statement.named_child(0).filter(|n|n.kind()=="assignment");
                if let Some(field)=assignment {
                    let field_name=field.child_by_field_name("left");
                    let ty=field.child_by_field_name("type");
                    if let (Some(n),Some(t))=(field_name,ty) {
                        // Exclude ClassVar, InitVar, KW_ONLY and field(...) customizations.
                        if n.kind()!="identifier" || self.text(t)!="str" || bindings.contains_key("str") {valid=false;}
                        if field.child_by_field_name("right").is_some_and(|v|v.kind()!="string") {valid=false;}
                        fields.push(self.text(n).to_owned());
                    } else {valid=false;}
                } else if statement.kind()!="expression_statement" || !statement.named_child(0).is_some_and(|n|n.kind()=="string") {
                    valid=false;
                }
            }
        }
        let mut decorators=node.walk();
        for child in node.named_children(&mut decorators).filter(|n|n.id()!=class.id()) {self.walk(child,bindings,depth+1);}
        self.walk(class,bindings,depth+1);
        if valid && found && fields.iter().any(|s|matches!(s.as_str(),"model"|"model_name")) {
            if let Some(name)=name {
                bindings.insert(name,Binding {canonical:fields.join(","),asset:None,kind:Some("local_model_record".into()),resolution_path:vec![],wrapper:false,namespace:None});
            }
        }
        true
    }
    pub(super) fn record_local_model(&mut self,node:Node<'_>,bindings:&Bindings)->bool {
        if self.language!="python" {return false;}
        let Some(function)=node.child_by_field_name("function").filter(|n|n.kind()=="identifier") else {return false;};
        let Some(binding)=bindings.get(self.text(function)).filter(|b|b.kind.as_deref()==Some("local_model_record")) else {return false;};
        let Some(args)=node.child_by_field_name("arguments") else {return true;};
        let fields:Vec<_>=binding.canonical.split(',').collect();
        let mut values=BTreeMap::new();let mut positional=0;let mut keyword_seen=false;
        let mut c=args.walk();
        for arg in args.named_children(&mut c) {
            if arg.kind()=="comment" {continue;}
            let (name,value)=if arg.kind()=="keyword_argument" {
                keyword_seen=true;
                let (Some(n),Some(v))=(arg.child_by_field_name("name"),arg.child_by_field_name("value")) else {return true;};
                (self.text(n).to_owned(),v)
            } else {
                if keyword_seen || positional>=fields.len() || matches!(arg.kind(),"list_splat"|"dictionary_splat") {return true;}
                let n=fields[positional];positional+=1;(n.to_owned(),arg)
            };
            if !fields.contains(&name.as_str()) || values.insert(name,value).is_some() {return true;}
        }
        for (name,value) in values {
            if !matches!(name.as_str(),"model"|"model_name") || value.kind()!="string" {continue;}
            let literal=self.text(value);
            if !literal.starts_with(['\'', '"']) || literal.contains(['\\','\n','\r']) {continue;}
            let id=unquote(literal);
            if let Some(entry)=self.catalog.entries.iter().find(|e|e.model_identifiers.contains(id)) {
                let mut asset=Asset::new("model",id,Some(&entry.id),self.evidence(value,"literal_model_record",id));
                asset.subtype=Some("reference".into());asset.confidence="inferred_from_local_record".into();
                self.result.assets.push(asset);
            }
        }
        true
    }
}

impl Scanner<'_> {
    pub(super) fn record_catalog_model(&mut self,node:Node<'_>,bindings:&Bindings) {
        if self.language!="python" {return;}
        let Some(binding)=self.call_binding(node,bindings) else {return;};
        // The bounded module graph does not prove wrapper argument forwarding.
        if binding.wrapper {return;}
        let Some(entry)=self.catalog.import("python",&binding.canonical) else {return;};
        let Some(rule)=entry.model_arguments.get(&binding.canonical) else {return;};
        let Some(args)=node.child_by_field_name("arguments") else {return;};
        let mut selected=vec![];let mut position=0;let mut keywords=std::collections::BTreeSet::new();let mut keyword_seen=false;
        let mut c=args.walk();
        for arg in args.named_children(&mut c) {
            if arg.kind()=="comment" {continue;}
            if matches!(arg.kind(),"list_splat"|"dictionary_splat") {return;}
            if arg.kind()=="keyword_argument" {
                keyword_seen=true;
                let (Some(name),Some(value))=(arg.child_by_field_name("name"),arg.child_by_field_name("value")) else {return;};
                if !keywords.insert(self.text(name)) {return;}
                if rule.keywords.iter().any(|k|k==self.text(name)) {selected.push(value);}
            } else {
                if keyword_seen {return;}
                if rule.position==Some(position) {selected.push(arg);}
                position+=1;
            }
        }
        if selected.len()!=1 {return;}
        let value=selected[0];let raw=self.text(value);
        if value.kind()!="string" || !raw.starts_with(['\'', '"']) || raw.contains(['\\','\n','\r']) {return;}
        let id=unquote(raw);
        if rule.require_provider_prefix && !id.split_once(':').is_some_and(|(provider, model)| !provider.is_empty() && !model.is_empty() && provider.bytes().all(|b|b.is_ascii_alphanumeric() || b"-_".contains(&b))) {return;}
        // The qualified constructor supplies model context, including private/future IDs.
        if id.is_empty() || id.len()>160 || !id.bytes().all(|b|b.is_ascii_alphanumeric() || b"-_/.:".contains(&b)) {return;}
        let mut asset=Asset::new("model",id,Some(&entry.id),self.evidence(value,"catalog_model_argument",id));
        asset.evidence.push(self.evidence(node,"catalog_model_constructor",&binding.canonical));
        asset.subtype=Some("reference".into());asset.confidence="inferred_from_import".into();
        if !binding.resolution_path.is_empty() {
            asset.confidence="inferred_from_module".into();
            for evidence in &mut asset.evidence { evidence.resolution_path=binding.resolution_path.clone(); }
        }
        self.result.assets.push(asset);
    }
}
