//! Bounded, static Python module exports. No imports, builds, or Python code are executed.
use crate::{catalog::Catalog, model::digest};
use std::{collections::{BTreeMap, BTreeSet}, fs, io::Read, path::{Path, PathBuf}};
use tree_sitter::{Node, Parser, Tree};

#[derive(Clone)]
struct Target { module: String, symbol: String, wrapper: bool, late_binding: Option<String> }

#[derive(Default)]
pub struct ModuleIndex {
    files: BTreeSet<String>,
    exports: BTreeMap<String, BTreeMap<String, Target>>,
    hashes: BTreeMap<String, String>,
    trees: BTreeMap<String, Tree>,
    pub limited: bool,
}

pub enum Resolution {
    External,
    Resolved { canonical: String, steps: Vec<String>, wrapper: bool },
    Unresolved,
}

impl ModuleIndex {
    pub fn build(root: &Path, paths: &[PathBuf], max_file_bytes: u64) -> Self {
        let mut index = Self::default();
        let python: Vec<_> = paths.iter().filter(|p| p.extension().is_some_and(|e| e == "py")).collect();
        for path in &python {
            if let Ok(relative) = path.strip_prefix(root) { index.files.insert(relative.to_string_lossy().replace('\\', "/")); }
        }
        let mut parser = Parser::new();
        if parser.set_language(&tree_sitter_python::LANGUAGE.into()).is_err() { index.limited = true; return index; }
        let mut bytes_seen = 0;
        let mut facts_seen = 0;
        let mut cached_source_bytes = 0;
        for path in python {
            let Ok(relative) = path.strip_prefix(root) else { continue; };
            let relative = relative.to_string_lossy().replace('\\', "/");
            // A separate whole-project budget bounds graph construction on very large trees.
            if bytes_seen >= 128 * 1024 * 1024 || facts_seen >= 100_000 { index.limited = true; break; }
            let read = || -> std::io::Result<Vec<u8>> {
                if !fs::symlink_metadata(path)?.file_type().is_file() { return Ok(vec![]); }
                let mut bytes = vec![];
                fs::File::open(path)?.take(max_file_bytes + 1).read_to_end(&mut bytes)?;
                Ok(bytes)
            };
            let Ok(bytes) = read() else { continue; };
            // The file pass reports individual size failures; only project-wide limits add a graph diagnostic.
            if bytes.len() as u64 > max_file_bytes { continue; }
            if bytes_seen + bytes.len() > 128 * 1024 * 1024 { index.limited = true; continue; }
            bytes_seen += bytes.len();
            let Ok(source) = std::str::from_utf8(&bytes) else { continue; };
            let Some(tree) = parser.parse(source, None) else { continue; };
            index.hashes.insert(relative.clone(), digest(&bytes));
            let exports = if tree.root_node().has_error() { BTreeMap::new() } else { exports(tree.root_node(), source) };
            facts_seen += exports.len();
            if facts_seen > 100_000 { index.limited = true; break; }
            // Reuse parsed trees for common-sized projects; bound cache by source bytes.
            if cached_source_bytes + bytes.len() <= 16 * 1024 * 1024 {
                cached_source_bytes += bytes.len(); index.trees.insert(relative.clone(), tree);
            }
            index.exports.insert(relative, exports);
        }
        index
    }

    pub fn unchanged(&self, path: &str, text: &str) -> bool {
        self.hashes.get(path).is_some_and(|hash| *hash == digest(text.as_bytes()))
    }

    pub fn tree(&self, path: &str) -> Option<&Tree> { self.trees.get(path) }

    pub fn local_namespace(&self, path: &str, module: &str) -> bool {
        !matches!(self.locate(path, module), Ok(None)) || !matches!(self.locate(path, module.split('.').next().unwrap_or(module)), Ok(None))
    }

    fn locate(&self, path: &str, module: &str) -> Result<Option<String>, ()> {
        let dots = module.bytes().take_while(|b| *b == b'.').count();
        let mut parts: Vec<_> = if dots > 0 { path.rsplit_once('/').map(|(p, _)| p.split('/').collect()).unwrap_or_default() } else { vec![] };
        for _ in 1..dots { if parts.pop().is_none() { return Err(()); } }
        let tail = &module[dots..];
        if !tail.is_empty() { parts.extend(tail.split('.')); }
        if parts.iter().any(|p| p.is_empty() || *p == "..") { return Err(()); }
        let stem = parts.join("/");
        let candidates = if stem.is_empty() { vec!["__init__.py".into()] } else { vec![format!("{stem}.py"), format!("{stem}/__init__.py")] };
        let matches: Vec<_> = candidates.into_iter().filter(|p| self.files.contains(p)).collect();
        match matches.as_slice() { [one] => Ok(Some(one.clone())), [] if dots == 0 => Ok(None), _ => Err(()) }
    }

    pub fn resolve(&self, path: &str, module: &str, symbol: &str, catalog: &Catalog) -> Resolution {
        match self.locate(path, module) {
            Ok(None) => Resolution::External,
            Err(()) => Resolution::Unresolved,
            Ok(Some(local)) => {
                let mut seen = BTreeSet::new();
                let mut steps = vec![];
                self.follow(&local, symbol, catalog, &mut seen, &mut steps, 0)
                    .map(|(canonical, wrapper)| Resolution::Resolved { canonical, steps, wrapper })
                    .unwrap_or(Resolution::Unresolved)
            }
        }
    }

    fn follow(&self, path: &str, symbol: &str, catalog: &Catalog, seen: &mut BTreeSet<(String, String)>, steps: &mut Vec<String>, depth: usize) -> Option<(String, bool)> {
        if depth >= 8 || !seen.insert((path.into(), symbol.into())) { return None; }
        let target = self.exports.get(path)?.get(symbol)?;
        steps.push(format!("{path}:{symbol}"));
        match self.locate(path, &target.module).ok()? {
            Some(local) => { let (canonical, wrapper) = self.follow(&local, &target.symbol, catalog, seen, steps, depth + 1)?; Some((canonical, wrapper || target.wrapper)) }
            None => {
                let canonical = if target.symbol.is_empty() { target.module.clone() } else { format!("{}.{}", target.module, target.symbol) };
                let entry = catalog.import("python", &canonical)?;
                // Only reviewed constructors/functions may establish cross-file calls.
                if catalog.symbol_kind(entry, "python", &canonical).is_none() && !entry.model_arguments.contains_key(&canonical) { return None; }
                steps.push(canonical.clone());
                Some((canonical, target.wrapper))
            }
        }
    }
}

fn text<'a>(node: Node<'_>, source: &'a str) -> &'a str { &source[node.byte_range()] }
fn expression(node: Node<'_>, source: &str, bindings: &BTreeMap<String, Target>) -> Option<Target> {
    if !matches!(node.kind(), "identifier" | "attribute") { return None; }
    let value = text(node, source);
    let (root, suffix) = value.split_once('.').unwrap_or((value, ""));
    let mut target = bindings.get(root)?.clone();
    if !suffix.is_empty() {
        target.symbol = if target.symbol.is_empty() { suffix.into() } else { format!("{}.{}", target.symbol, suffix) };
    }
    Some(target)
}

fn exports(root: Node<'_>, source: &str) -> BTreeMap<String, Target> {
    let mut bindings = BTreeMap::new();
    let mut cursor = root.walk();
    for statement in root.named_children(&mut cursor) {
        let node = if statement.kind() == "expression_statement" { statement.named_child(0).unwrap_or(statement) } else { statement };
        match node.kind() {
            "import_from_statement" => {
                let Some(module) = node.child_by_field_name("module_name") else { continue; };
                let mut cursor = node.walk();
                for name in node.children_by_field_name("name", &mut cursor) {
                    let original = name.child_by_field_name("name").unwrap_or(name);
                    let alias = name.child_by_field_name("alias").unwrap_or(original);
                    invalidate_wrappers(&mut bindings, text(alias, source));
                    bindings.insert(text(alias, source).into(), Target { module: text(module, source).into(), symbol: text(original, source).into(), wrapper: false, late_binding: None });
                }
            }
            "import_statement" => {
                let mut cursor = node.walk();
                for name in node.children_by_field_name("name", &mut cursor) {
                    let original = name.child_by_field_name("name").unwrap_or(name);
                    let module = text(original, source);
                    let alias = name.child_by_field_name("alias");
                    let local = alias.map(|n| text(n, source)).unwrap_or_else(|| module.split('.').next().unwrap());
                    invalidate_wrappers(&mut bindings, local);
                    bindings.insert(local.into(), Target { module: if alias.is_some() { module.into() } else { local.into() }, symbol: String::new(), wrapper: false, late_binding: None });
                }
            }
            "assignment" => {
                if let Some(lhs) = node.child_by_field_name("left").filter(|n| n.kind() == "identifier") {
                    let value = node.child_by_field_name("right").and_then(|rhs| expression(rhs, source, &bindings));
                    invalidate_wrappers(&mut bindings, text(lhs, source));
                    bindings.remove(text(lhs, source));
                    if let Some(value) = value { bindings.insert(text(lhs, source).into(), value); }
                } else { invalidate(node, source, &mut bindings, 0); }
            }
            "function_definition" => {
                let Some(name) = node.child_by_field_name("name") else { continue; };
                invalidate_wrappers(&mut bindings, text(name, source));
                bindings.remove(text(name, source));
                if text(node, source).trim_start().starts_with("async ") { continue; }
                let mut local = bindings.clone();
                if let Some(parameters) = node.child_by_field_name("parameters") { remove_identifiers(parameters, source, &mut local, 0); }
                let Some(body) = node.child_by_field_name("body") else { continue; };
                let mut cursor = body.walk();
                let statements: Vec<_> = body.named_children(&mut cursor).filter(|n| n.kind() != "comment").collect();
                if statements.len() != 1 || statements[0].kind() != "return_statement" { continue; }
                let Some(call) = statements[0].named_child(0).filter(|n| n.kind() == "call") else { continue; };
                let Some(mut target) = call.child_by_field_name("function").and_then(|n| expression(n, source, &local)) else { continue; };
                target.wrapper = true;
                target.late_binding = call.child_by_field_name("function").map(|n| text(n, source).split('.').next().unwrap_or("").into());
                bindings.insert(text(name, source).into(), target);
            }
            "comment" | "string" => {}
            _ => invalidate(node, source, &mut bindings, 0),
        }
    }
    bindings
}

fn remove_identifiers(node: Node<'_>, source: &str, bindings: &mut BTreeMap<String, Target>, depth: usize) {
    if depth > 128 { bindings.clear(); return; }
    if node.kind() == "identifier" { invalidate_wrappers(bindings, text(node, source)); bindings.remove(text(node, source)); }
    let mut cursor = node.walk(); for child in node.named_children(&mut cursor) { remove_identifiers(child, source, bindings, depth + 1); }
}

fn invalidate(node: Node<'_>, source: &str, bindings: &mut BTreeMap<String, Target>, depth: usize) {
    if depth > 128 { bindings.clear(); return; }
    if matches!(node.kind(), "function_definition" | "class_definition") {
        if let Some(name) = node.child_by_field_name("name") { invalidate_wrappers(bindings, text(name, source)); bindings.remove(text(name, source)); }
        return;
    }
    if matches!(node.kind(), "assignment" | "augmented_assignment") {
        if let Some(lhs) = node.child_by_field_name("left") { remove_identifiers(lhs, source, bindings, 0); }
    }
    if matches!(node.kind(), "import_from_statement" | "import_statement" | "delete_statement") { remove_identifiers(node, source, bindings, 0); }
    let mut cursor = node.walk(); for child in node.named_children(&mut cursor) { invalidate(child, source, bindings, depth + 1); }
}

fn invalidate_wrappers(bindings: &mut BTreeMap<String, Target>, name: &str) {
    let mut changed = BTreeSet::from([name.to_owned()]);
    for _ in 0..8 {
        let affected: Vec<_> = bindings.iter().filter(|(_, target)| target.late_binding.as_ref().is_some_and(|name| changed.contains(name))).map(|(name, _)| name.clone()).collect();
        if affected.is_empty() { return; }
        for name in affected { bindings.remove(&name); changed.insert(name); }
    }
    bindings.retain(|_, target| !target.wrapper);
}
