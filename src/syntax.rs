use anyhow::{Context, Result};
use crate::{catalog::Catalog, model::{Asset, Evidence, Relationship}};
use std::{collections::BTreeMap, path::Path};
use tree_sitter::{Language, Node, Parser};

#[derive(Clone)]
struct Binding { canonical: String, asset: Option<String>, kind: Option<String>, resolution_path: Vec<String>, wrapper: bool, namespace: Option<String> }
mod patterns;
mod model_records;

type Bindings = BTreeMap<String, Binding>;

#[derive(Default)]
pub struct SyntaxResult {
    pub assets: Vec<Asset>,
    pub relationships: Vec<Relationship>,
    pub parse_errors: bool,
    pub unresolved_modules: bool,
}

pub fn language(path: &Path) -> Option<&'static str> {
    Some(match path.extension()?.to_str()? {
        "py" | "pyi" => "python", "js" | "jsx" | "mjs" | "cjs" => "javascript",
        "ts" | "tsx" | "mts" | "cts" => "typescript", "java" => "java", "go" => "go",
        "rs" => "rust", "cs" => "csharp", "rb" => "ruby", "php" => "php", "kt" | "kts" => "kotlin", _ => return None,
    })
}

fn grammar(language: &str, path: &str) -> Language {
    match language {
        "python" => tree_sitter_python::LANGUAGE.into(),
        "kotlin" => tree_sitter_kotlin_ng::LANGUAGE.into(),
        "javascript" => tree_sitter_javascript::LANGUAGE.into(),
        "typescript" if path.ends_with(".tsx") => tree_sitter_typescript::LANGUAGE_TSX.into(),
        "typescript" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        "java" => tree_sitter_java::LANGUAGE.into(), "go" => tree_sitter_go::LANGUAGE.into(),
        "rust" => tree_sitter_rust::LANGUAGE.into(), "csharp" => tree_sitter_c_sharp::LANGUAGE.into(),
        "ruby" => tree_sitter_ruby::LANGUAGE.into(), "php" => tree_sitter_php::LANGUAGE_PHP.into(), _ => unreachable!(),
    }
}

pub fn scan(path: &str, source: &str, language: &str, catalog: &Catalog) -> Result<SyntaxResult> {
    scan_with_modules(path, source, language, catalog, None)
}

pub fn scan_with_modules(path: &str, source: &str, language: &str, catalog: &Catalog, modules: Option<&crate::modules::ModuleIndex>) -> Result<SyntaxResult> {
    let changed_source = language == "python" && modules.is_some_and(|index| !index.unchanged(path, source));
    let modules = modules.filter(|_| language == "python" && !changed_source);
    let mut parser = Parser::new(); parser.set_language(&grammar(language, path))?;
    let parsed;
    let tree = if let Some(tree) = modules.and_then(|index| index.tree(path)) { tree }
        else { parsed = parser.parse(source, None).context("parser cancelled")?; &parsed };
    let mut scanner = Scanner { path, source, language, catalog, modules, result: SyntaxResult { parse_errors: tree.root_node().has_error(), unresolved_modules: changed_source, ..Default::default() } };
    let mut bindings = Bindings::new();
    // Ruby applications commonly load gems outside the current file. These exact,
    // cataloged constants are qualified-symbol evidence, never proof of a require.
    if language == "ruby" {
        for root in catalog.entries.iter().flat_map(|e| e.imports.get("ruby").into_iter().flatten())
            .filter(|p| p.bytes().next().is_some_and(|b| b.is_ascii_uppercase()) && !p.contains(['.', ':'])) {
            bindings.insert(root.clone(), Binding { canonical: root.clone(), asset: None, kind: None, resolution_path: vec![], wrapper: false, namespace: None });
        }
    }
    scanner.walk(tree.root_node(), &mut bindings, 0);
    scanner.agent_loops(tree.root_node());
    Ok(scanner.result)
}

struct Scanner<'a> { path: &'a str, source: &'a str, language: &'a str, catalog: &'a Catalog, modules: Option<&'a crate::modules::ModuleIndex>, result: SyntaxResult }

impl Scanner<'_> {
    fn text(&self, n: Node<'_>) -> &str { &self.source[n.byte_range()] }
    fn evidence(&self, node: Node<'_>, mechanism: &str, symbol: &str) -> Evidence {
        Evidence { path: self.path.into(), line: node.start_position().row + 1, column: node.start_position().column + 1, mechanism: mechanism.into(), symbol: symbol.into(), cell: None, resolution_path: vec![] }
    }
    fn bind_import(&mut self, node: Node<'_>, canonical: &str, local: &str, bindings: &mut Bindings) {
        let asset = self.catalog.import(self.language, canonical).map(|e| Asset::new("dependency", &e.name, Some(&e.id), self.evidence(node, "ast_import", canonical)));
        let id = asset.as_ref().map(|a| a.id.clone());
        if let Some(asset) = asset { self.result.assets.push(asset); }
        if !local.is_empty() { bindings.insert(local.into(), Binding { canonical: canonical.into(), asset: id, kind: None, resolution_path: vec![], wrapper: false, namespace: None }); }
    }
    fn imports(&mut self, node: Node<'_>, bindings: &mut Bindings) -> bool {
        let text = self.text(node).to_owned();
        match (self.language, node.kind()) {
            ("kotlin", "import") => {
                if let Some(name) = node.named_child(0).filter(|n| n.kind() == "qualified_identifier") {
                    let canonical = self.text(name).to_owned();
                    let alias = node.named_child(1).filter(|n| n.kind() == "identifier").map(|n| self.text(n)).unwrap_or_else(|| canonical.rsplit('.').next().unwrap()).to_owned();
                    let wildcard = text.trim_end().ends_with(".*");
                    self.bind_import(node, &canonical, if wildcard { "" } else { &alias }, bindings);
                }
                true
            }
            ("python", "import_from_statement") => {
                let Some(module) = node.child_by_field_name("module_name") else { return true; };
                let module = self.text(module).to_owned();
                let mut cursor = node.walk();
                for name in node.children_by_field_name("name", &mut cursor) {
                    let original = name.child_by_field_name("name").unwrap_or(name);
                    let local = name.child_by_field_name("alias").unwrap_or(original);
                    let canonical = format!("{module}.{}", self.text(original));
                    if let Some(modules) = self.modules {
                        match modules.resolve(self.path, &module, self.text(original), self.catalog) {
                            crate::modules::Resolution::Resolved { canonical, steps, wrapper } => {
                                let local = self.text(local).to_owned();
                                let before = self.result.assets.len();
                                self.bind_import(node, &canonical, &local, bindings);
                                if let Some(binding) = bindings.get_mut(&local) { binding.resolution_path = steps.clone(); binding.wrapper = wrapper; }
                                for asset in &mut self.result.assets[before..] { asset.evidence[0].resolution_path = steps.clone(); }
                                continue;
                            }
                            crate::modules::Resolution::Unresolved => {
                                self.result.unresolved_modules = true; bindings.remove(self.text(local)); continue;
                            }
                            crate::modules::Resolution::External => {}
                        }
                    }
                    if module.starts_with('.') { self.result.unresolved_modules = true; continue; }
                    self.bind_import(node, &canonical, &self.text(local).to_owned(), bindings);
                }
                true
            }
            ("python", "import_statement") => {
                let mut cursor = node.walk();
                for name in node.children_by_field_name("name", &mut cursor) {
                    let original = name.child_by_field_name("name").unwrap_or(name);
                    let canonical = self.text(original).to_owned();
                    if self.modules.is_some_and(|index| index.local_namespace(self.path, &canonical)) {
                        self.result.unresolved_modules = true;
                        let local = name.child_by_field_name("alias").map(|n| self.text(n)).unwrap_or_else(|| canonical.split('.').next().unwrap());
                        bindings.insert(local.into(), Binding { canonical: canonical.clone(), asset: None, kind: None, resolution_path: vec![], wrapper: false, namespace: Some(if name.child_by_field_name("alias").is_some() { canonical.clone() } else { local.into() }) }); continue;
                    }
                    if let Some(alias) = name.child_by_field_name("alias") {
                        self.bind_import(node, &canonical, &self.text(alias).to_owned(), bindings);
                    } else {
                        self.bind_import(node, &canonical, "", bindings);
                        let root = canonical.split('.').next().unwrap();
                        bindings.insert(root.into(), Binding { canonical: root.into(), asset: self.result.assets.last().filter(|a| a.evidence[0].line == node.start_position().row + 1).map(|a| a.id.clone()), kind: None, resolution_path: vec![], wrapper: false, namespace: None });
                    }
                }
                true
            }
            ("javascript" | "typescript", "import_statement") => {
                let Some(source) = node.child_by_field_name("source") else { return true; };
                let module = unquote(self.text(source)).to_owned();
                self.bind_import(node, &module, "", bindings);
                if text.trim_start().starts_with("import type ") { return true; }
                let mut cursor = node.walk();
                for clause in node.named_children(&mut cursor).filter(|n| n.kind() == "import_clause") {
                    let mut c = clause.walk();
                    for item in clause.named_children(&mut c) {
                        match item.kind() {
                            "identifier" => self.bind_import(node, &format!("{module}.default"), &self.text(item).to_owned(), bindings),
                            "namespace_import" => {
                                if let Some(id) = item.named_child(0) { self.bind_import(node, &module, &self.text(id).to_owned(), bindings); }
                            }
                            "named_imports" => {
                                let mut c = item.walk();
                                for spec in item.named_children(&mut c) {
                                    if self.text(spec).trim_start().starts_with("type ") { continue; }
                                    if let Some(name) = spec.child_by_field_name("name") {
                                        let alias = spec.child_by_field_name("alias").unwrap_or(name);
                                        self.bind_import(node, &format!("{module}.{}", self.text(name)), &self.text(alias).to_owned(), bindings);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                true
            }
            ("rust", "use_declaration") => {
                if let Some(argument) = node.child_by_field_name("argument") { self.rust_use(argument, "", bindings, 0); }
                true
            }
            ("php", "namespace_use_declaration") => {
                let prefix = node.named_child(0).filter(|n| n.kind() == "namespace_name").map(|n| self.text(n).to_owned()).unwrap_or_default();
                let group = node.child_by_field_name("body").unwrap_or(node);
                let mut cursor = group.walk();
                for clause in group.named_children(&mut cursor).filter(|n| n.kind() == "namespace_use_clause") {
                    if clause.child_by_field_name("type").is_some() || node.child_by_field_name("type").is_some() { continue; }
                    if let Some(name) = clause.named_child(0) {
                        let full = if prefix.is_empty() { self.text(name).trim_start_matches('\\').to_owned() } else { format!("{prefix}\\{}", self.text(name)) };
                        let local = clause.child_by_field_name("alias").map(|n| self.text(n)).unwrap_or_else(|| full.rsplit('\\').next().unwrap()).to_owned();
                        self.bind_import(clause, &full, &local, bindings);
                    }
                }
                true
            }
            ("csharp", "using_directive") => {
                let value = text.trim().trim_start_matches("global ").trim_start_matches("using ").trim_end_matches(';').trim();
                if let Some((alias, target)) = value.split_once('=') {
                    self.bind_import(node, target.trim(), alias.trim(), bindings);
                } else if !value.starts_with("static ") {
                    self.bind_import(node, value, "", bindings);
                    self.csharp_namespace(value, bindings);
                    let prefix = format!("{value}.");
                    let symbols: Vec<_> = self.catalog.entries.iter().flat_map(|e| &e.symbols)
                        .filter_map(|(symbol, _)| symbol.strip_prefix(&prefix).map(|tail| tail.split('.').next().unwrap()).map(|tail| (format!("{prefix}{tail}"), tail.to_owned()))).collect();
                    for (symbol, local) in symbols {
                        if value == "Microsoft.Extensions.DependencyInjection" { bindings.insert(local, Binding { canonical: symbol, asset: None, kind: None, resolution_path: vec![], wrapper: false, namespace: None }); }
                        else { self.bind_import(node, &symbol, &local, bindings); }
                    }
                }
                true
            }
            ("java", "import_declaration") => {
                let canonical = text.trim().trim_start_matches("import ").trim_start_matches("static ").trim_start_matches("using ").trim_start_matches("use ").trim_end_matches(';').trim();
                let local = canonical.rsplit(['.', ':']).next().unwrap_or("");
                // Complex Rust grouped uses are reported at the crate root only.
                let canonical = if self.language == "rust" { canonical.split("::{").next().unwrap_or(canonical) } else { canonical };
                self.bind_import(node, canonical, local, bindings); true
            }
            ("go", "import_spec") => {
                if let Some(path) = node.child_by_field_name("path") {
                    let module = unquote(self.text(path)).to_owned();
                    let local = node.child_by_field_name("name").map(|n| self.text(n).to_owned()).unwrap_or_else(|| self.catalog.import("go", &module).and_then(|e| e.module_names.get(&module)).cloned().unwrap_or_else(|| module.rsplit('/').next().unwrap().into()));
                    self.bind_import(node, &module, if local == "." || local == "_" { "" } else { &local }, bindings);
                }
                true
            }
            _ => false,
        }
    }

    fn rust_use(&mut self, node: Node<'_>, prefix: &str, bindings: &mut Bindings, depth: usize) {
        if depth > 32 { self.result.parse_errors = true; return; }
        match node.kind() {
            "scoped_use_list" => {
                let path = node.child_by_field_name("path").map(|n| self.text(n)).unwrap_or("");
                let full = if prefix.is_empty() { path.to_owned() } else { format!("{prefix}::{path}") };
                if let Some(list) = node.child_by_field_name("list") { self.rust_use(list, &full, bindings, depth + 1); }
            }
            "use_list" => { let mut cursor = node.walk(); for child in node.named_children(&mut cursor) { self.rust_use(child, prefix, bindings, depth + 1); } }
            "use_wildcard" => {
                let module = node.named_child(0).map(|n| self.text(n).to_owned()).unwrap_or_else(|| prefix.into());
                self.bind_import(node, &module, "", bindings);
                let exports = self.catalog.import("rust", &module).and_then(|e| e.wildcard_exports.get(&module)).cloned().unwrap_or_default();
                for name in exports { self.bind_import(node, &format!("{module}::{name}"), &name, bindings); }
            }
            _ => {
                let name = if node.kind() == "use_as_clause" { node.child_by_field_name("path").unwrap_or(node) } else { node };
                let raw = self.text(name);
                let full = if raw == "self" { prefix.to_owned() } else if prefix.is_empty() { raw.to_owned() } else { format!("{prefix}::{raw}") };
                let local = node.child_by_field_name("alias").map(|n| self.text(n)).unwrap_or_else(|| full.rsplit("::").next().unwrap()).to_owned();
                self.bind_import(node, &full, &local, bindings);
            }
        }
    }

    fn resolve(&self, raw: &str, bindings: &Bindings) -> Option<Binding> {
        let (name,b) = bindings.iter().filter(|(name, _)| raw == name.as_str() || raw.strip_prefix(name.as_str()).is_some_and(|s| s.starts_with('.') || s.starts_with("::")))
            .max_by_key(|(name, _)| name.len())?;
        if let Some(module) = &b.namespace {
            let suffix = raw.strip_prefix(name)?.strip_prefix('.')?;
            let (symbol,tail) = suffix.split_once('.').unwrap_or((suffix,""));
            if let crate::modules::Resolution::Resolved {canonical,steps,wrapper} = self.modules?.resolve(self.path,module,symbol,self.catalog) {
                let kind=self.catalog.import("python",&canonical).and_then(|e|self.catalog.symbol_kind(e,"python",&canonical)).map(str::to_owned);
                return Some(Binding {canonical:if tail.is_empty(){canonical}else{format!("{canonical}.{tail}")},asset:None,kind,resolution_path:steps,wrapper,namespace:None});
            }
            return None;
        }
        Some(Binding { canonical: format!("{}{}", b.canonical, &raw[name.len()..]), asset: b.asset.clone(), kind: b.kind.clone(), resolution_path: b.resolution_path.clone(), wrapper: b.wrapper, namespace: None })
    }

    fn callable<'n>(&self, node: Node<'n>) -> Option<Node<'n>> {
        if self.language == "python" && node.kind() == "decorator" {
            return node.named_child(0).filter(|n| matches!(n.kind(), "identifier" | "attribute"));
        }
        if self.language == "csharp" && node.kind() == "attribute" { return node.child_by_field_name("name"); }
        if self.language == "ruby" && node.kind() == "call" { return node.child_by_field_name("method"); }
        if self.language == "kotlin" && node.kind() == "call_expression" { return node.named_child(0); }
        match node.kind() {
            "call" | "call_expression" | "invocation_expression" => node.child_by_field_name("function"),
            "new_expression" => node.child_by_field_name("constructor"),
            "object_creation_expression" => node.child_by_field_name("type").or_else(|| node.named_child(0)),
            "implicit_object_creation_expression" if self.language == "csharp" => node.parent().filter(|p| p.kind() == "variable_declarator").and_then(|p| p.parent()).and_then(|p| p.child_by_field_name("type")),
            "function_call_expression" => node.child_by_field_name("function"),
            "method_invocation" | "scoped_call_expression" | "member_call_expression" => node.child_by_field_name("name"),
            _ => None,
        }
    }

    fn call_binding(&self, node: Node<'_>, bindings: &Bindings) -> Option<Binding> {
        let callable = self.callable(node)?;
        if self.language == "csharp" {
            if let Some(b) = self.csharp_binding(node, bindings, 0) { return Some(b); }
        }
        if self.language == "ruby" {
            let receiver = node.child_by_field_name("receiver")?;
            if receiver.kind() == "call" {
                // Resolve a direct factory/operation chain only; avoid unbounded recursion.
                if receiver.child_by_field_name("receiver").is_some_and(|n| n.kind()=="call") { return None; }
                let mut base = self.call_binding(receiver, bindings)?;
                if self.catalog.import("ruby", &base.canonical).and_then(|e|self.catalog.symbol_kind(e,"ruby",&base.canonical)) != Some("llm_client") { return None; }
                base.kind=Some("llm_client".into());
                base.canonical=format!("{}.{}",base.canonical,self.text(callable));
                return Some(base);
            }
            return self.resolve(&format!("{}.{}", self.text(receiver), self.text(callable)), bindings);
        }
        if matches!(node.kind(), "method_invocation" | "scoped_call_expression" | "member_call_expression") {
            let object = node.child_by_field_name("object").or_else(|| node.child_by_field_name("scope"));
            if let Some(object) = object {
                if self.callable(object).is_some() { return None; }
                let separator = if node.kind() == "scoped_call_expression" { "::" } else { "." };
                return self.resolve(&format!("{}{separator}{}", self.text(object), self.text(callable)), bindings);
            }
        }
        self.resolve(self.text(callable), bindings)
    }

    fn record_call(&mut self, node: Node<'_>, bindings: &Bindings) {
        if self.record_local_model(node, bindings) { return; }
        self.record_catalog_model(node, bindings);
        // Recognize Ruby require and CommonJS require as imports only when structurally a call.
        if let Some(function) = node.child_by_field_name("function").or_else(|| node.child_by_field_name("method")) {
            if self.text(function) == "require" {
                if let Some(args) = node.child_by_field_name("arguments") {
                    if let Some(arg) = args.named_child(0).filter(|a| matches!(a.kind(), "string" | "string_literal")) {
                        self.bind_import(node, &unquote(self.text(arg)).to_owned(), "", &mut Bindings::new());
                    }
                }
            }
        }
        let Some(binding) = self.call_binding(node, bindings) else { return; };
        self.configured_tool_loop(node, &binding, bindings);
        if self.language == "rust" && self.catalog.import("rust", &binding.canonical).is_some_and(|e|e.rust_attributes.contains_key(&binding.canonical)) { return; }
        let Some(entry) = self.catalog.import(self.language, &binding.canonical) else { return; };
        let explicit = self.catalog.symbol_kind(entry, self.language, &binding.canonical);
        if matches!(node.kind(), "decorator" | "attribute") && explicit != Some("tool") { return; }
        let kind = explicit.or_else(|| {
            if binding.kind.as_deref() == Some("llm_client") {
                // Match the complete operation following an explicitly cataloged constructor.
                entry.symbols.iter().filter(|(symbol, _)| self.catalog.symbol_kind(entry, self.language, symbol) == Some("llm_client"))
                    .filter_map(|(constructor, _)| binding.canonical.strip_prefix(constructor).and_then(|suffix| suffix.strip_prefix('.')))
                    .find_map(|operation| entry.operations.get(operation).map(String::as_str))
            }
            else { None }
        });
        let Some(kind) = kind else { return; };
        let mechanism = if matches!(node.kind(), "decorator" | "attribute") { "ast_decorator" } else { "ast_call" };
        let mut asset = Asset::new(kind, &binding.canonical, Some(&entry.id), self.evidence(node, mechanism, &binding.canonical));
        asset.subtype = entry.symbol_subtypes.get(&binding.canonical).cloned();
        if asset.subtype.is_none() && !entry.symbols.contains_key(&binding.canonical) && self.language == "python" && entry.api_references.contains(&binding.canonical) {
            asset.subtype = Some("upstream_api_reference".into());
        }
        // Local binding is lexical, not whole-program type resolution.
        asset.confidence = "inferred_from_import".into();
        // Ruby's reviewed constant roots do not establish a verified require/load graph,
        // including when a subsequent operation follows a local client variable.
        if self.language == "ruby" { asset.confidence = "qualified_symbol".into(); }
        if !binding.resolution_path.is_empty() { asset.evidence[0].resolution_path = binding.resolution_path.clone(); asset.confidence = if binding.wrapper { "inferred_from_wrapper" } else { "inferred_from_module" }.into(); }
        if let Some(parent) = &binding.asset {
            self.result.relationships.push(Relationship { from: asset.id.clone(), to: parent.clone(), kind: "uses".into() });
        }
        if let Some(args) = node.child_by_field_name("arguments").filter(|_| kind != "dependency" && (!binding.wrapper || explicit.is_none())) {
            self.record_models(args, &asset, 0);
        }
        self.result.assets.push(asset);
    }

    fn record_models(&mut self, node: Node<'_>, parent: &Asset, depth: usize) {
        if depth > 64 { self.result.parse_errors = true; return; }
        // Only literal model/model_name arguments inside a recognized AI call.
        if matches!(node.kind(), "keyword_argument" | "pair") {
            let key = node.child_by_field_name("name").or_else(|| node.child_by_field_name("key"));
            let value = node.child_by_field_name("value");
            if let (Some(key), Some(value)) = (key, value) {
                if ["model", "model_name"].contains(&unquote(self.text(key))) && value.kind() == "string" {
                    let model = unquote(self.text(value));
                    if !model.is_empty() && model.len() <= 160 && model.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_/.:".contains(&b)) {
                        let mut asset = Asset::new("model", model, parent.framework.as_deref(), self.evidence(value, "literal_model_argument", model));
                        asset.subtype = Some("reference".into());
                        self.result.relationships.push(Relationship { from: parent.id.clone(), to: asset.id.clone(), kind: "references_model".into() });
                        self.result.assets.push(asset);
                    }
                }
            }
            return;
        }
        // Avoid attributing model literals inside unrelated nested calls to the outer call.
        if self.callable(node).is_some() { return; }
        let mut cursor = node.walk(); for child in node.named_children(&mut cursor) { self.record_models(child, parent, depth + 1); }
    }

    fn walk(&mut self, node: Node<'_>, bindings: &mut Bindings, depth: usize) {
        if depth > 256 { self.result.parse_errors = true; return; }
        if self.imports(node, bindings) { return; }
        if self.model_record_declaration(node, bindings, depth) { return; }
        if self.language == "rust" { self.rust_annotation(node, bindings); }
        if matches!(node.kind(), "comment" | "string" | "string_literal" | "template_string") { return; }
        if matches!(node.kind(), "function_definition" | "function_declaration" | "function_item" | "method_declaration" | "arrow_function" | "lambda" | "method_definition" | "class_definition" | "class_declaration" | "method" | "singleton_method" | "class" | "do_block") || (self.language == "ruby" && node.kind() == "module") {
            if let Some(name) = node.child_by_field_name("name") { bindings.remove(self.text(name)); }
            if self.language == "csharp" {
                let mut cursor = node.walk();
                for attributes in node.named_children(&mut cursor).filter(|n| n.kind() == "attribute_list") { self.walk(attributes, bindings, depth + 1); }
            }
            let mut scoped = bindings.clone();
            if let Some(parameters) = node.child_by_field_name("parameters") {
                if self.language == "csharp" { let mut c=parameters.walk(); for p in parameters.named_children(&mut c) { if let Some(n)=p.child_by_field_name("name") { scoped.remove(self.text(n)); } } }
                else { remove_identifiers(parameters, self.source, &mut scoped, 0); }
            }
            if let Some(parameter) = node.child_by_field_name("parameter") { remove_identifiers(parameter, self.source, &mut scoped, 0); }
            if self.language == "csharp" { self.csharp_parameters(node, &mut scoped); }
            if self.language == "kotlin" {
                let mut cursor = node.walk();
                for params in node.named_children(&mut cursor).filter(|n| n.kind() == "function_value_parameters") { remove_identifiers(params, self.source, &mut scoped, 0); }
            }
            let body = node.child_by_field_name("body").or_else(|| {
                let mut cursor = node.walk(); node.named_children(&mut cursor).find(|n| matches!(n.kind(), "function_body" | "class_body"))
            });
            if let Some(body) = body {
                remove_local_bindings(body, self.source, &mut scoped, 0);
                self.walk(body, &mut scoped, depth + 1);
            }
            return;
        }
        if matches!(node.kind(), "assignment" | "assignment_expression" | "assignment_statement" | "short_var_declaration" | "let_declaration" | "variable_declarator" | "property_declaration") {
            let kotlin_property = self.language == "kotlin" && node.kind() == "property_declaration" && self.text(node).contains('=');
            let lhs = node.child_by_field_name("left").or_else(|| node.child_by_field_name("name")).or_else(|| node.child_by_field_name("pattern"))
                .or_else(|| kotlin_property.then(|| node.named_child(0)?.named_child(0)).flatten());
            let rhs = node.child_by_field_name("right").or_else(|| node.child_by_field_name("value"))
                .or_else(|| (self.language == "csharp" && node.kind() == "variable_declarator").then(|| node.named_child(1)).flatten());
            let rhs = rhs.or_else(|| kotlin_property.then(|| node.named_child(node.named_child_count().saturating_sub(1))).flatten());
            if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                let lhs = single_expression(lhs);
                let rhs = single_expression(rhs);
                let resolved = self.call_binding(rhs, bindings).and_then(|mut b| {
                    b.kind = self.catalog.import(self.language, &b.canonical).and_then(|e| self.catalog.symbol_kind(e, self.language, &b.canonical)).map(str::to_owned).or(b.kind);
                    b.kind.as_ref().filter(|kind|kind.as_str()!="local_model_record")?; Some(b)
                }).or_else(|| matches!(rhs.kind(), "identifier" | "attribute" | "member_expression").then(|| self.resolve(self.text(rhs), bindings)).flatten());
                self.walk(rhs, bindings, depth + 1);
                remove_identifiers(lhs, self.source, bindings, 0);
                // Conventional Go SDK factories return (client, error). Bind only the
                // first result of a recognized client factory and an explicit err/_ slot.
                let lhs = if self.language == "go" && lhs.kind() == "expression_list"
                    && lhs.named_child_count() == 2 && resolved.as_ref().is_some_and(|b| b.kind.as_deref() == Some("llm_client"))
                    && lhs.named_child(1).is_some_and(|n| matches!(self.text(n), "err" | "_")) {
                    lhs.named_child(0).unwrap()
                } else { lhs };
                if matches!(lhs.kind(), "identifier" | "variable_name") {
                    if let Some(mut b) = resolved {
                        b.asset = self.result.assets.iter().rev().find(|a| a.name == b.canonical && a.evidence[0].line == rhs.start_position().row + 1 && a.evidence[0].column == rhs.start_position().column + 1).map(|a| a.id.clone()).or(b.asset);
                        bindings.insert(self.text(lhs).into(), b);
                    }
                }
                return;
            }
        }
        if self.callable(node).is_some() || node.kind() == "call" { self.record_call(node, bindings); }
        let mut cursor = node.walk(); for child in node.named_children(&mut cursor) { self.walk(child, bindings, depth + 1); }
    }
}

fn remove_identifiers(node: Node<'_>, source: &str, bindings: &mut Bindings, depth: usize) {
    if depth > 64 { bindings.clear(); return; }
    if matches!(node.kind(), "identifier" | "variable_name" | "constant") { bindings.remove(&source[node.byte_range()]); }
    let mut cursor = node.walk(); for child in node.named_children(&mut cursor) { remove_identifiers(child, source, bindings, depth + 1); }
}

fn remove_local_bindings(node: Node<'_>, source: &str, bindings: &mut Bindings, depth: usize) {
    if depth > 128 { bindings.clear(); return; }
    if matches!(node.kind(), "function_definition" | "function_declaration" | "class_definition" | "class_declaration") {
        if let Some(name) = node.child_by_field_name("name") { bindings.remove(&source[name.byte_range()]); }
        return;
    }
    if matches!(node.kind(), "assignment" | "variable_declarator" | "property_declaration") {
        if let Some(name) = node.child_by_field_name("left").or_else(|| node.child_by_field_name("name")) { remove_identifiers(name, source, bindings, 0); }
    }
    let mut cursor = node.walk(); for child in node.named_children(&mut cursor) { remove_local_bindings(child, source, bindings, depth + 1); }
}

fn unquote(value: &str) -> &str { value.trim_matches(['\'', '"', '`']) }

fn single_expression(node: Node<'_>) -> Node<'_> {
    if node.kind() == "expression_list" && node.named_child_count() == 1 { node.named_child(0).unwrap() } else { node }
}

