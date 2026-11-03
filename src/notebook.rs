use anyhow::{Context, Result, ensure};
use crate::{catalog::Catalog, syntax::{self, SyntaxResult}};

/// Cell outputs, attachments, notebook prose and metadata strings are never scanned as code.
pub fn scan(path: &str, text: &str, catalog: &Catalog) -> Result<(SyntaxResult, String)> {
    let notebook: serde_json::Value = serde_json::from_str(text).context("invalid notebook JSON")?;
    ensure!(notebook["nbformat"] == 4, "only notebook format 4 is supported");
    let language = notebook["metadata"]["language_info"]["name"].as_str()
        .or_else(|| notebook["metadata"]["kernelspec"]["language"].as_str()).context("notebook language metadata is missing")?;
    let extension = match language { "python" => "py", "javascript" => "js", "typescript" => "ts", _ => anyhow::bail!("notebook language is unsupported") };
    let cells = notebook["cells"].as_array().context("notebook cells missing")?;
    ensure!(cells.len() <= 10000, "notebook has too many cells");
    let mut result = SyntaxResult::default();
    for (index, cell) in cells.iter().enumerate() {
        if cell["cell_type"] != "code" { continue; }
        let code = match &cell["source"] {
            serde_json::Value::String(code) => code.clone(),
            serde_json::Value::Array(parts) => parts.iter().map(|p| p.as_str().context("notebook source parts must be strings")).collect::<Result<String>>()?,
            _ => anyhow::bail!("notebook code source missing"),
        };
        // Each cell is an independent lexical scope: execution order/state is not inferred.
        let mut found = syntax::scan(&format!("{path}#cell-{}.{}", index + 1, extension), &code, language, catalog)?;
        for asset in &mut found.assets {
            for evidence in &mut asset.evidence { evidence.path = path.into(); evidence.cell = Some(index + 1); }
        }
        result.assets.append(&mut found.assets);
        result.relationships.append(&mut found.relationships);
        result.parse_errors |= found.parse_errors;
    }
    Ok((result, language.into()))
}
