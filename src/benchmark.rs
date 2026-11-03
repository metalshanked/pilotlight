//! Reproducible development corpus runs. Findings are never treated as gold labels.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::json;
use std::{collections::{BTreeMap, BTreeSet}, fs, io::{Read, Write}, path::Path, time::{Instant, SystemTime, UNIX_EPOCH}};
use sha2::{Digest, Sha256};
use crate::{catalog::Catalog, model::digest, scan::{self, Options}, source::{Source, github_coordinates}};

#[derive(Deserialize)]
pub struct Target { pub repository: String, pub commit: String }
#[derive(Deserialize)]
struct Corpus { schema_version: u32, repositories: Vec<Target> }

pub fn targets(path: &Path) -> Result<Vec<Target>> {
    ensure!(fs::metadata(path)?.len() <= 2 * 1024 * 1024, "corpus exceeds 2 MiB");
    let corpus: Corpus = serde_json::from_slice(&fs::read(path)?)?;
    ensure!(corpus.schema_version == 1 && !corpus.repositories.is_empty() && corpus.repositories.len() <= 1000, "invalid corpus");
    let mut seen = BTreeSet::new();
    for target in &corpus.repositories {
        let (owner, repo) = github_coordinates(&target.repository)?;
        ensure!(target.commit.len() == 40 && target.commit.bytes().all(|b| b.is_ascii_hexdigit()), "benchmark needs full commit SHAs");
        ensure!(seen.insert((owner.to_lowercase(), repo.to_lowercase(), target.commit.to_lowercase())), "duplicate corpus target");
    }
    Ok(corpus.repositories)
}

pub fn run(corpus: &Path, destination: &Path, catalog: &Catalog, options: &Options, limits: crate::source::AcquisitionLimits) -> Result<i32> {
    limits.validate()?;
    let targets = targets(corpus)?;
    ensure!(!destination.exists(), "benchmark output directory already exists; use a fresh run directory");
    fs::create_dir_all(destination)?;
    let executable = std::env::current_exe()?;
    let metadata = json!({"schema_version":1,"evaluation_status":"unlabeled_development_run","accuracy_metrics":null,
        "started_unix_seconds":SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        "scanner_version":env!("CARGO_PKG_VERSION"),"binary_sha256":digest(&fs::read(executable)?),
        "cargo_lock_sha256":digest(include_bytes!("../Cargo.lock")),
        "corpus_sha256":digest(&fs::read(corpus)?),"catalog_sha256":catalog.fingerprint(),
        "catalog_entries":catalog.entries.len(),"jobs":options.jobs,"max_file_bytes":options.max_file_bytes,
        "os":std::env::consts::OS,"architecture":std::env::consts::ARCH,
        "authentication":if std::env::var("GITHUB_TOKEN").is_ok_and(|s| !s.is_empty()) { "environment_token" } else { "unauthenticated" },
        "memory_measurement":"not_measured","trials_per_target":1,
        "acquisition_limits":limits,"max_archive_entries":100000,"scope":"scanner scope recorded in each report"});
    fs::write(destination.join("run.json"), serde_json::to_vec_pretty(&metadata)?)?;
    fs::write(destination.join("catalog.json"), serde_json::to_vec_pretty(catalog)?)?;
    fs::copy(corpus, destination.join("corpus.lock.json"))?;
    let mut outcomes = fs::File::create(destination.join("outcomes.jsonl"))?;
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for (index, target) in targets.iter().enumerate() {
        eprintln!("Benchmark {}/{}: {}", index + 1, targets.len(), target.repository);
        let directory = destination.join(format!("{:03}",index + 1)); fs::create_dir(&directory)?;
        let start = Instant::now();
        let source = Source::open_with_limits(&target.repository, Some(&target.commit), limits);
        let acquisition_seconds = start.elapsed().as_secs_f64();
        let mut outcome = json!({"index":index+1,"repository":target.repository,"commit":target.commit,"acquisition_seconds":acquisition_seconds});
        match source {
            Err(error) => { outcome["status"]=json!("acquisition_failed"); outcome["error"]=json!(format!("{error:#}")); }
            Ok(source) => {
                outcome["archive_sha256"]=json!(source.archive_sha256);
                let start=Instant::now();
                let hashes=hash_sources(&source.root, &directory.join("source-files.jsonl"))?;
                outcome["source_hashing_seconds"]=json!(start.elapsed().as_secs_f64());
                outcome["input_files"]=json!(hashes.0); outcome["input_bytes"]=json!(hashes.1);
                let start=Instant::now();
                let result=scan::scan(&source,catalog,options);
                outcome["scan_seconds"]=json!(start.elapsed().as_secs_f64());
                match result {
                    Err(error) => {outcome["status"]=json!("scan_failed");outcome["error"]=json!(format!("{error:#}"));}
                    Ok(report) => {
                        outcome["status"]=json!(report.status);outcome["findings"]=json!(report.assets.len());
                        outcome["files_scanned"]=json!(report.coverage.files_scanned);outcome["diagnostics"]=json!(report.coverage.diagnostics.len());
                        let mut kinds:BTreeMap<&str,usize>=BTreeMap::new();for a in &report.assets {*kinds.entry(&a.kind).or_default()+=1;}
                        outcome["by_type"]=json!(kinds);
                        fs::write(directory.join("inventory.json"),serde_json::to_vec_pretty(&report)?)?;
                    }
                }
            }
        }
        let status=outcome["status"].as_str().unwrap().to_owned();*counts.entry(status.clone()).or_default()+=1;
        eprintln!("  {status}; findings={}",outcome["findings"]);
        serde_json::to_writer(&mut outcomes,&outcome)?;writeln!(outcomes)?;outcomes.flush()?;
    }
    let failures:usize=counts.iter().filter(|(s,_)|s.ends_with("failed")).map(|(_,n)|n).sum();
    fs::write(destination.join("summary.json"),serde_json::to_vec_pretty(&json!({"repositories":targets.len(),"outcomes":counts,"accuracy_metrics":null,"qualification":"development_only_unlabeled"}))?)?;
    Ok(if failures>0 {1} else {0})
}

fn hash_sources(root:&Path, output:&Path)->Result<(usize,u64)> {
    let mut paths=vec![];
    let mut walker=ignore::WalkBuilder::new(root);walker.hidden(false).ignore(false).git_ignore(false).git_global(false).git_exclude(false).follow_links(false);
    for entry in walker.build() {let entry=entry?;if entry.file_type().is_some_and(|t|t.is_file()){paths.push(entry.into_path());}}
    paths.sort();let mut writer=fs::File::create(output)?;let mut bytes=0;
    for path in &paths {
        let mut file=fs::File::open(path)?;let mut hasher=Sha256::new();let mut buffer=[0u8;65536];let mut size=0u64;
        loop {let n=file.read(&mut buffer)?;if n==0 {break;}hasher.update(&buffer[..n]);size+=n as u64;}
        bytes+=size;
        let relative=path.strip_prefix(root).context("source path escaped root")?.to_string_lossy().replace('\\',"/");
        serde_json::to_writer(&mut writer,&json!({"path":relative,"bytes":size,"sha256":format!("{:x}",hasher.finalize())}))?;writeln!(writer)?;
    }
    Ok((paths.len(),bytes))
}
