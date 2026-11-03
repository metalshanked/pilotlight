use anyhow::{Context, Result, bail, ensure};
use flate2::read::GzDecoder;
use reqwest::{Url, blocking::Client, redirect::Policy};
use serde::Deserialize;
use std::{fs, io::{Read, Write}, path::{Component, Path, PathBuf}, time::Duration};
use tempfile::TempDir;

const MAX_DOWNLOAD: u64 = 100 * 1024 * 1024;
const MAX_EXPANDED: u64 = 512 * 1024 * 1024;
const MAX_FILES: usize = 100_000;

#[derive(Clone, Copy, serde::Serialize)]
pub struct AcquisitionLimits { pub max_download_bytes: u64, pub max_expanded_bytes: u64 }
impl Default for AcquisitionLimits {
    fn default() -> Self { Self { max_download_bytes: MAX_DOWNLOAD, max_expanded_bytes: MAX_EXPANDED } }
}
impl AcquisitionLimits {
    pub fn validate(&self) -> Result<()> {
        ensure!((1..=1024*1024*1024).contains(&self.max_download_bytes), "max-download-bytes must be 1..1073741824");
        ensure!((1..=4*1024*1024*1024).contains(&self.max_expanded_bytes), "max-expanded-bytes must be 1..4294967296");
        Ok(())
    }
}

pub struct Source {
    pub root: PathBuf,
    pub label: String,
    pub revision: Option<String>,
    pub archive_sha256: Option<String>,
    pub omissions: Vec<crate::model::Diagnostic>,
    _temporary: Option<TempDir>,
}

pub fn http_client() -> Result<Client> {
    Ok(Client::builder().user_agent(concat!("pilotlight/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(120)).connect_timeout(Duration::from_secs(15))
        .redirect(Policy::none()).build()?)
}

pub fn github_coordinates(input: &str) -> Result<(String, String)> {
    let url = Url::parse(input).context("expected https://github.com/OWNER/REPO")?;
    ensure!(url.scheme() == "https" && url.host_str() == Some("github.com") && url.port().is_none() && url.username().is_empty() && url.password().is_none() && url.query().is_none() && url.fragment().is_none(), "expected a plain HTTPS github.com repository URL");
    let parts: Vec<_> = url.path().trim_matches('/').split('/').collect();
    ensure!(parts.len() == 2, "use a repository URL and --ref for a branch/tag/commit");
    let repo = parts[1].strip_suffix(".git").unwrap_or(parts[1]);
    ensure!(parts[0].bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-') && !parts[0].is_empty(), "invalid GitHub owner");
    ensure!(!repo.is_empty() && repo != "." && repo != ".." && repo.bytes().all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c)), "invalid GitHub repository");
    Ok((parts[0].into(), repo.into()))
}

impl Source {
    pub fn open(input: &str, reference: Option<&str>) -> Result<Self> {
        Self::open_with_limits(input, reference, AcquisitionLimits::default())
    }

    pub fn open_with_limits(input: &str, reference: Option<&str>, limits: AcquisitionLimits) -> Result<Self> {
        limits.validate()?;
        if input.starts_with("https://") || input.contains("://") {
            let (owner, repo) = github_coordinates(input)?;
            Self::github(&owner, &repo, reference, limits)
        } else {
            ensure!(reference.is_none(), "--ref is only valid for GitHub sources");
            let root = Path::new(input).canonicalize().context("source directory not found")?;
            ensure!(root.is_dir(), "source must be a directory");
            Ok(Self { label: root.display().to_string(), root, revision: None, archive_sha256: None, omissions: vec![], _temporary: None })
        }
    }

    fn github(owner: &str, repo: &str, reference: Option<&str>, limits: AcquisitionLimits) -> Result<Self> {
        let client = http_client()?;
        let token = std::env::var("GITHUB_TOKEN").ok().filter(|s| !s.is_empty());
        // An unauthenticated immutable public pin needs no REST metadata lookup.
        // This avoids consuming two API requests per repository in public benchmarks.
        if token.is_none() {
            if let Some(sha) = reference.filter(|s| s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit())) {
                let response = client.get(format!("https://codeload.github.com/{owner}/{repo}/tar.gz/{sha}")).send()?.error_for_status()?;
                return Self::archive(owner, repo, sha, response, limits);
            }
        }
        let request = |url: Url| {
            let mut req = client.get(url).header("Accept", "application/vnd.github+json");
            if let Some(token) = &token { req = req.bearer_auth(token); }
            req
        };
        // Resolve to an immutable commit before downloading; branch names can move mid-scan.
        let reference = if let Some(reference) = reference { reference.to_owned() } else {
            #[derive(Deserialize)] struct Repo { default_branch: String }
            request(Url::parse(&format!("https://api.github.com/repos/{owner}/{repo}"))?).send()?.error_for_status()?.json::<Repo>()?.default_branch
        };
        let mut commit_url = Url::parse(&format!("https://api.github.com/repos/{owner}/{repo}/commits/"))?;
        commit_url.path_segments_mut().unwrap().pop_if_empty().push(&reference);
        #[derive(Deserialize)] struct Commit { sha: String }
        let sha = request(commit_url).send()?.error_for_status()?.json::<Commit>()?.sha;
        ensure!(sha.len() == 40 && sha.bytes().all(|c| c.is_ascii_hexdigit()), "invalid commit SHA from GitHub");
        let response = request(Url::parse(&format!("https://api.github.com/repos/{owner}/{repo}/tarball/{sha}"))?).send()?;
        let response = if response.status().is_redirection() {
            let location = response.headers().get(reqwest::header::LOCATION).context("missing archive redirect")?.to_str()?;
            let url = Url::parse(location)?;
            ensure!(url.scheme() == "https" && url.host_str() == Some("codeload.github.com") && url.port().is_none() && url.username().is_empty() && url.password().is_none(), "unexpected archive destination");
            // No authorization header is forwarded across hosts. GitHub's signed URL carries access.
            client.get(url).send().map_err(reqwest::Error::without_url)?
                .error_for_status().map_err(reqwest::Error::without_url)?
        } else { response.error_for_status()? };
        Self::archive(owner, repo, &sha, response, limits)
    }

    fn archive(owner: &str, repo: &str, sha: &str, response: reqwest::blocking::Response, limits: AcquisitionLimits) -> Result<Self> {
        let temporary = tempfile::tempdir()?;
        let archive_path = temporary.path().join("source.tar.gz");
        let mut output = fs::File::create(&archive_path)?;
        let copied = std::io::copy(&mut response.take(limits.max_download_bytes + 1), &mut output).context("archive download failed")?;
        ensure!(copied <= limits.max_download_bytes, "archive exceeds compressed limit; configure --max-download-bytes for a larger repository");
        output.flush()?;
        let root = temporary.path().join("repo"); fs::create_dir(&root)?;
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new(); let mut archive_file = fs::File::open(&archive_path)?; let mut buffer = [0u8;65536];
        loop { let n = archive_file.read(&mut buffer)?; if n == 0 { break; } hasher.update(&buffer[..n]); }
        let archive_sha256 = format!("{:x}", hasher.finalize());
        let omissions = extract(fs::File::open(archive_path)?, &root, true, limits)?;
        Ok(Self { root, label: format!("https://github.com/{owner}/{repo}"), revision: Some(sha.into()), archive_sha256: Some(archive_sha256), omissions, _temporary: Some(temporary) })
    }
}

pub fn extract_archive(reader: impl Read, destination: &Path) -> Result<()> {
    extract(reader, destination, false, AcquisitionLimits::default()).map(|_| ())
}

/// Inventory mode omits links with explicit coverage diagnostics; never follows them.
pub fn extract_inventory_archive(reader: impl Read, destination: &Path) -> Result<Vec<crate::model::Diagnostic>> {
    extract(reader, destination, true, AcquisitionLimits::default())
}

fn extract(reader: impl Read, destination: &Path, omit_links: bool, limits: AcquisitionLimits) -> Result<Vec<crate::model::Diagnostic>> {
    let mut omissions = vec![];
    let decoder = GzDecoder::new(reader);
    let mut archive = tar::Archive::new(decoder.take(limits.max_expanded_bytes + 16 * 1024 * 1024));
    let mut total = 0_u64;
    let mut entries = 0;
    for entry in archive.entries()? {
        let mut entry = entry?;
        entries += 1; ensure!(entries <= MAX_FILES, "archive has too many entries");
        let kind = entry.header().entry_type();
        // GitHub archives begin with a global PAX metadata header (for the commit).
        // It is not a filesystem entry and must never be materialized.
        if kind.as_byte() == b'g' { continue; }
        // Never materialize symlinks, hardlinks, device files, or executable special files.
        let omitted_link = omit_links && (kind.is_symlink() || kind.is_hard_link());
        ensure!(kind.is_file() || kind.is_dir() || omitted_link, "archive contains unsupported entry type");
        let path = entry.path()?.into_owned();
        ensure!(path.components().all(|c| matches!(c, Component::Normal(_))), "unsafe archive path");
        let mut components = path.components(); components.next();
        let relative: PathBuf = components.collect();
        if relative.as_os_str().is_empty() { continue; }
        // Reject NTFS alternate data streams and Windows path aliases even on Unix.
        ensure!(relative.components().all(|c| {
            let s = c.as_os_str().to_string_lossy();
            let stem = s.split('.').next().unwrap_or("").to_ascii_uppercase();
            !s.contains([':', '\\']) && !s.ends_with(['.', ' ']) && !["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"].contains(&stem.as_str())
        }), "unsafe archive filename");
        if omitted_link {
            omissions.push(crate::model::Diagnostic { path: relative.to_string_lossy().replace('\\', "/"), reason: "archive link omitted without following its target".into() });
            continue;
        }
        let target = destination.join(relative);
        if kind.is_dir() { fs::create_dir_all(&target)?; continue; }
        total = total.checked_add(entry.size()).context("archive size overflow")?;
        ensure!(total <= limits.max_expanded_bytes, "archive exceeds expanded size limit; configure --max-expanded-bytes for a larger repository");
        if let Some(parent) = target.parent() { fs::create_dir_all(parent)?; }
        let mut file = fs::OpenOptions::new().create_new(true).write(true).open(&target).context("duplicate or invalid archive file")?;
        std::io::copy(&mut entry, &mut file)?;
    }
    if entries == 0 { bail!("empty repository archive"); }
    Ok(omissions)
}
