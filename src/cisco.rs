//! Offline import of the upstream component_catalog schema, with an independent overlay.
use anyhow::{Result, ensure};
use std::path::Path;
use crate::catalog::Catalog;

#[cfg(feature = "duckdb-catalog")]
pub fn import(input: &Path, output: &Path, expected_sha256: &str, source_url: &str, overlay: &Catalog) -> Result<Catalog> {
    use std::{collections::BTreeMap, fs, io::Read};
    use sha2::{Digest, Sha256};
    use duckdb::{AccessMode, Config, Connection};
    use crate::catalog::{Seed, prefix_matches};
    ensure!(!output.exists(), "output already exists");
    ensure!(overlay.seed.is_none(), "use the independently maintained overlay, not an already seeded catalog");
    ensure!(expected_sha256.len() == 64 && expected_sha256.bytes().all(|b| b.is_ascii_hexdigit()), "expected a SHA-256 digest");
    let url = reqwest::Url::parse(source_url)?;
    ensure!(url.scheme() == "https" && url.username().is_empty() && url.password().is_none(), "source URL must be HTTPS without credentials");
    ensure!(fs::metadata(input)?.len() <= 512 * 1024 * 1024, "seed database exceeds 512 MiB");
    let parent = output.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let staged = tempfile::NamedTempFile::new_in(parent)?.into_temp_path();
    let staged_path = staged.to_path_buf();
    fs::copy(input, &staged_path)?;
    let mut file = fs::File::open(&staged_path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop { let n = file.read(&mut buffer)?; if n == 0 { break; } hash.update(&buffer[..n]); }
    let actual = format!("{:x}", hash.finalize());
    ensure!(actual == expected_sha256.to_ascii_lowercase(), "seed checksum mismatch");
    drop(file);
    let conn = Connection::open_with_flags(&staged_path, Config::default().access_mode(AccessMode::ReadOnly)?)?;
    conn.execute_batch("SET enable_external_access=false; SET autoinstall_known_extensions=false; SET autoload_known_extensions=false;")?;
    let table_count: i64 = conn.query_row("SELECT count(*) FROM information_schema.tables WHERE table_schema='main' AND table_name='component_catalog' AND table_type='BASE TABLE'", [], |r| r.get(0))?;
    ensure!(table_count == 1, "missing component_catalog base table");
    let columns = conn.prepare("SELECT column_name, data_type FROM information_schema.columns WHERE table_schema='main' AND table_name='component_catalog' ORDER BY ordinal_position")?.query_map([], |r| Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?)))?.collect::<std::result::Result<Vec<_>,_>>()?;
    let expected = ["id","label","concept","framework","sig_name","type","catalog_label"];
    ensure!(columns.len() == expected.len() && columns.iter().zip(expected).all(|((name,ty),want)| name == want && ty == "VARCHAR"), "unsupported component_catalog schema");
    let count: u64 = conn.query_row("SELECT count(*) FROM component_catalog", [], |r| r.get(0))?;
    ensure!(count > 0 && count <= 3_000_000, "seed must contain 1–3,000,000 rows");
    let mut rows_by_label = BTreeMap::new();
    for row in conn.prepare("SELECT coalesce(label, '<null>'), count(*) FROM component_catalog GROUP BY label")?.query_map([], |r| Ok((r.get::<_,String>(0)?,r.get::<_,u64>(1)?)))? {
        let (label,count) = row?; rows_by_label.insert(label,count);
    }
    let mut catalog = overlay.clone();
    let mut unmapped = 0;
    // Keep all raw rows in the database. Only class/function identities become exact,
    // Python-only dependency references; concepts never automatically become AI roles.
    for row in conn.prepare("SELECT id FROM component_catalog WHERE label IN ('class','function')")?.query_map([], |r| r.get::<_,Option<String>>(0))? {
        let Some(symbol) = row? else { unmapped += 1; continue; };
        if symbol.len() > 1024 || symbol.split('.').any(|p| p.is_empty() || !p.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')) { unmapped += 1; continue; }
        let candidate = catalog.entries.iter().enumerate().flat_map(|(i,e)| e.imports.get("python").into_iter().flatten().map(move |p|(i,p)))
            .filter(|(_,p)| prefix_matches(&symbol,p)).max_by_key(|(_,p)| p.len()).map(|(i,_)|i);
        if let Some(index) = candidate { catalog.entries[index].api_references.insert(symbol); }
        else { unmapped += 1; }
    }
    let indexed = catalog.entries.iter().map(|e| e.api_references.len() as u64).sum();
    catalog.seed = Some(Seed { source_sha256: actual, source_url: source_url.into(), rows: count, rows_by_label, indexed_api_references: indexed, unmapped_callable_rows: unmapped });
    catalog.validate()?;
    let document = serde_json::to_string(&catalog)?;
    // Apply the same size validation as runtime catalog loading.
    ensure!(document.len() <= 8 * 1024 * 1024, "normalized catalog exceeds 8 MiB");
    drop(conn);
    let conn = Connection::open(&staged_path)?;
    conn.execute_batch("SET enable_external_access=false; SET autoinstall_known_extensions=false; SET autoload_known_extensions=false; BEGIN TRANSACTION; CREATE TABLE pilotlight_catalog (id INTEGER PRIMARY KEY, document VARCHAR);")?;
    conn.execute("INSERT INTO pilotlight_catalog VALUES (1, ?)", [&document])?;
    conn.execute_batch("COMMIT; CHECKPOINT;")?;
    drop(conn);
    staged.persist_noclobber(output)?;
    Ok(catalog)
}

#[cfg(not(feature = "duckdb-catalog"))]
pub fn import(_input: &Path, _output: &Path, _expected_sha256: &str, _source_url: &str, _overlay: &Catalog) -> Result<Catalog> {
    ensure!(false, "rebuild with --features duckdb-catalog to import a Cisco-format seed");
    unreachable!()
}
