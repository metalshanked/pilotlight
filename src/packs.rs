use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{collections::{BTreeMap, BTreeSet}, fs, io::{Read, Write}, path::{Path, PathBuf}};
use crate::{catalog::{Catalog, Entry}, model::{Asset, digest}};

const LIMIT: u64 = 8 * 1024 * 1024;
pub const CAPABILITIES: &[&str] = &["catalog-v1", "receiver-operations-v1", "python-decorators-v1", "rust-attributes-v1", "csharp-di-v1", "tool-loops-v1", "configured-loops-v1", "model-records-v1", "model-arguments-v1"];
const BUILTINS: &[&str] = &[include_str!("../packs/agents.json"), include_str!("../packs/mcp.json")];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pack {
    pub schema_version: u32,
    pub id: String,
    pub version: String,
    pub minimum_engine_version: String,
    pub description: String,
    pub license: String,
    pub sources: Vec<String>,
    pub requires: Vec<String>,
    pub rules: Vec<Rule>,
    #[serde(default)] pub fixtures: Vec<Fixture>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule { pub id: String, pub entry: Entry }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fixture {
    pub id: String, pub language: String, pub source: String,
    /// Exhaustive counts for the listed artifact types; dependencies can be checked separately.
    pub expect: BTreeMap<String, usize>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct PackInfo { pub id: String, pub version: String, pub sha256: String }
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct RuleOrigin { pub pack: PackInfo, pub rule_id: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppliedRule {
    pub origin: RuleOrigin, pub framework: String,
    pub symbols: BTreeSet<String>, pub operations: BTreeSet<String>,
    pub imports: BTreeMap<String, Vec<String>>, pub packages: BTreeMap<String, Vec<String>>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")] pub model_identifiers: BTreeSet<String>,
}
#[derive(Debug, Serialize)]
pub struct FixtureResult { pub id: String, pub passed: bool, pub expected: BTreeMap<String, usize>, pub actual: BTreeMap<String, usize> }

fn version(s: &str) -> Result<(u32,u32,u32)> {
    let parts=s.split('.').map(str::parse::<u32>).collect::<std::result::Result<Vec<_>,_>>()?;
    ensure!(parts.len()==3,"versions must use MAJOR.MINOR.PATCH");
    Ok((parts[0],parts[1],parts[2]))
}
fn identifier(s: &str) -> bool { !s.is_empty() && s.len()<=100 && s.bytes().all(|b|b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b,b'-'|b'_'|b'.')) }
impl Pack {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        ensure!(bytes.len()<=LIMIT as usize,"pack exceeds 8 MiB");
        let p: Self=serde_json::from_slice(bytes).context("invalid detection pack JSON")?;
        p.validate()?; Ok(p)
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(self.schema_version==1,"unsupported pack schema");
        ensure!(identifier(&self.id),"invalid pack id"); version(&self.version)?;
        ensure!(version(&self.minimum_engine_version)?<=version(env!("CARGO_PKG_VERSION"))?,"pack requires a newer engine");
        ensure!(!self.description.is_empty() && !self.license.is_empty(),"pack description and license required");
        ensure!(!self.sources.is_empty() && self.sources.iter().all(|s|s.starts_with("https://") && s.len()<=2048),"pack requires HTTPS provenance");
        ensure!(self.requires.iter().all(|s|CAPABILITIES.contains(&s.as_str())),"unsupported pack engine capability");
        ensure!(!self.rules.is_empty() && self.rules.len()<=1000,"pack needs 1–1000 rules");
        let mut ids=BTreeSet::new();
        for rule in &self.rules { ensure!(identifier(&rule.id) && ids.insert(&rule.id),"invalid/duplicate rule id"); }
        let catalog=Catalog {schema_version:1,version:self.version.clone(),entries:self.rules.iter().map(|r|r.entry.clone()).collect(),seed:None,packs:vec![],pack_rules:vec![]};
        catalog.validate()?;
        let mut fixtures=BTreeSet::new();
        ensure!(self.fixtures.len()<=500,"too many fixtures");
        for f in &self.fixtures {
            ensure!(identifier(&f.id) && fixtures.insert(&f.id),"invalid/duplicate fixture id");
            ensure!(["python","javascript","typescript","java","go","rust","csharp","ruby","php","kotlin"].contains(&f.language.as_str()),"unsupported fixture language");
            ensure!(f.source.len()<=65536 && !f.expect.is_empty() && f.expect.keys().all(|k|crate::artifact::TYPES.contains(&k.as_str())),"invalid fixture source/expectations");
        }
        Ok(())
    }
}
pub fn read(path: &Path) -> Result<(Pack,PackInfo)> {
    ensure!(fs::metadata(path)?.len()<=LIMIT,"pack exceeds 8 MiB");
    let mut bytes=vec![]; fs::File::open(path)?.take(LIMIT+1).read_to_end(&mut bytes)?;
    let p=Pack::parse(&bytes)?; let info=PackInfo{id:p.id.clone(),version:p.version.clone(),sha256:digest(&bytes)};
    Ok((p,info))
}
pub fn builtins() -> Result<Vec<(Pack,PackInfo)>> {
    BUILTINS.iter().map(|s|{ let p=Pack::parse(s.as_bytes())?; let info=PackInfo{id:p.id.clone(),version:p.version.clone(),sha256:digest(s.as_bytes())}; Ok((p,info)) }).collect()
}

/// Additive composition: identical data is deduplicated, conflicting semantics fail.
/// Order never chooses the winning classification. Replacements require a new explicit base catalog.
pub fn apply(catalog: &mut Catalog, pack: &Pack, info: PackInfo) -> Result<()> {
    pack.validate()?;
    ensure!(info.id==pack.id && info.version==pack.version && info.sha256.len()==64 && info.sha256.bytes().all(|b|b.is_ascii_hexdigit()), "pack identity mismatch");
    ensure!(!catalog.packs.iter().any(|p|p.id==info.id),"duplicate pack id: {}",info.id);
    let mut next=catalog.clone();
    for rule in &pack.rules {
        if let Some(existing)=next.entries.iter_mut().find(|e|e.id==rule.entry.id) { merge_entry(existing,&rule.entry)?; }
        else { next.entries.push(rule.entry.clone()); }
        next.pack_rules.push(AppliedRule {origin:RuleOrigin{pack:info.clone(),rule_id:rule.id.clone()},framework:rule.entry.id.clone(),symbols:rule.entry.symbols.keys().chain(rule.entry.model_arguments.keys()).cloned().collect(),operations:rule.entry.operations.keys().cloned().collect(),imports:rule.entry.imports.clone(),packages:rule.entry.packages.clone(),model_identifiers:rule.entry.model_identifiers.clone()});
    }
    next.packs.push(info); next.packs.sort();
    next.pack_rules.sort_by(|a,b|a.origin.cmp(&b.origin));
    next.entries.sort_by(|a,b|a.id.cmp(&b.id));
    ensure!(serde_json::to_vec(&next)?.len()<=8*1024*1024,"composed catalog exceeds 8 MiB");
    next.validate()?; *catalog=next; Ok(())
}
fn merge_entry(base: &mut Entry, extra: &Entry) -> Result<()> {
    // JSON object union keeps this extensible as the typed catalog gains fields.
    // Scalar conflicts fail; set-like arrays are canonicalized rather than overwritten.
    fn merge(a: &mut serde_json::Value,b: serde_json::Value,path: &str) -> Result<()> {
        match (a,b) {
            (serde_json::Value::Object(a),serde_json::Value::Object(b)) => { for (k,v) in b { if let Some(old)=a.get_mut(&k) { merge(old,v,&format!("{path}.{k}"))?; } else {a.insert(k,v);} } }
            (serde_json::Value::Array(a),serde_json::Value::Array(b)) => { a.extend(b); a.sort_by_key(|v|v.to_string()); a.dedup(); }
            (a,b) => ensure!(*a==b,"conflicting pack field: {path}"),
        } Ok(())
    }
    let mut value=serde_json::to_value(&*base)?; merge(&mut value,serde_json::to_value(extra)?,&base.id)?;
    *base=serde_json::from_value(value)?; Ok(())
}
pub fn load(base: Option<&Path>, paths: &[PathBuf], no_defaults: bool) -> Result<Catalog> {
    ensure!(paths.len()<=32,"at most 32 external packs per scan");
    let defaults=builtins()?;
    let mut catalog=Catalog::load_base(base)?;
    let inherited_refs: BTreeMap<_,_>=catalog.entries.iter().map(|e|(e.id.clone(),e.api_references.clone())).collect();
    if base.is_none() {
        // The complete catalog snapshot remains exportable; these framework definitions
        // are owned by the bundled packs at runtime, including when packs are disabled.
        let owned: BTreeSet<_>=defaults.iter().flat_map(|(p,_)|p.rules.iter().map(|r|r.entry.id.as_str())).collect();
        catalog.entries.retain(|e|!owned.contains(e.id.as_str()));
        if !no_defaults { for (pack,info) in defaults { apply(&mut catalog,&pack,info)?; } }
    }
    let mut external=paths.iter().map(|p|read(p)).collect::<Result<Vec<_>>>()?;
    external.sort_by(|a,b|a.1.cmp(&b.1));
    for (pack,info) in external { apply(&mut catalog,&pack,info)?; }
    if base.is_none() {
        for entry in &mut catalog.entries {
            if let Some(refs)=inherited_refs.get(&entry.id) {entry.api_references.extend(refs.iter().filter(|s|entry.imports.get("python").is_some_and(|ps|ps.iter().any(|p|crate::catalog::prefix_matches(s,p)))).cloned());}
        }
    }
    if let Some(seed)=&mut catalog.seed {seed.indexed_api_references=catalog.entries.iter().map(|e|e.api_references.len() as u64).sum();}
    ensure!(serde_json::to_vec(&catalog)?.len()<=8*1024*1024,"composed catalog exceeds 8 MiB");
    catalog.validate()?; Ok(catalog)
}
pub fn annotate(catalog: &Catalog, asset: &mut Asset) {
    for rule in &catalog.pack_rules {
        if asset.framework.as_deref()!=Some(&rule.framework) {continue;}
        let matched=(asset.kind=="model" && rule.model_identifiers.contains(&asset.name)) || asset.evidence.iter().any(|e|rule.symbols.contains(&e.symbol)
            || rule.operations.iter().any(|op|e.symbol.ends_with(&format!(".{op}")))
            || (asset.kind=="dependency" && rule.imports.values().flatten().any(|p|crate::catalog::prefix_matches(&e.symbol,p))))
            || asset.package.as_ref().is_some_and(|p|rule.packages.get(&p.ecosystem).is_some_and(|names|names.iter().any(|n|crate::catalog::normalize_package(&p.ecosystem,n)==crate::catalog::normalize_package(&p.ecosystem,&p.name))));
        if matched {asset.detection_rules.push(rule.origin.clone());}
    }
    asset.detection_rules.sort(); asset.detection_rules.dedup();
}
pub fn test(pack: &Pack,info: PackInfo) -> Result<Vec<FixtureResult>> {
    ensure!(!pack.fixtures.is_empty(),"pack contains no fixtures");
    let mut catalog=Catalog {schema_version:1,version:"0.1.0".into(),entries:vec![],seed:None,packs:vec![],pack_rules:vec![]};
    apply(&mut catalog,pack,info)?;
    pack.fixtures.iter().map(|f|{
        let r=crate::syntax::scan("fixture",&f.source,&f.language,&catalog)?;
        ensure!(!r.parse_errors,"fixture {} has syntax errors",f.id);
        let actual=f.expect.keys().map(|k|(k.clone(),r.assets.iter().filter(|a|a.kind==*k).count())).collect::<BTreeMap<_,_>>();
        Ok(FixtureResult{id:f.id.clone(),passed:actual==f.expect,expected:f.expect.clone(),actual})
    }).collect()
}
pub fn install(input: &Path,sha256: &str,output: &Path) -> Result<()> {
    let (_,info)=read(input)?;
    ensure!(sha256.len()==64 && info.sha256==sha256.to_ascii_lowercase(),"pack checksum mismatch");
    ensure!(!output.exists(),"pack output already exists; choose a new pinned path");
    let mut bytes=vec![]; fs::File::open(input)?.take(LIMIT+1).read_to_end(&mut bytes)?;
    ensure!(bytes.len()<=LIMIT as usize,"pack exceeds 8 MiB");
    ensure!(digest(&bytes)==info.sha256,"pack changed during install");
    let parent=output.parent().filter(|p|!p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let mut temp=tempfile::NamedTempFile::new_in(parent)?;
    temp.write_all(&bytes)?; temp.as_file().sync_all()?;
    temp.persist_noclobber(output).context("could not install pack")?; Ok(())
}
