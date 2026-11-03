use anyhow::Result;
use crate::model::Report;
use serde_json::json;
use std::io::Write;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Format { Table, Json, Jsonl, Csv, Cyclonedx }

pub fn write_many(reports: &[Report], format: Format, mut out: impl Write) -> Result<()> {
    if reports.len() == 1 { return write(&reports[0], format, out); }
    match format {
        Format::Json => { serde_json::to_writer_pretty(&mut out, &json!({"schema_version":1,"reports":reports}))?; writeln!(out)?; }
        Format::Jsonl => { for report in reports { write(report, format, &mut out)?; } }
        Format::Csv => csv(reports, out)?,
        Format::Table => { for report in reports { write(report, format, &mut out)?; writeln!(out)?; } }
        Format::Cyclonedx => anyhow::bail!("CycloneDX supports one repository per invocation; use JSON or CSV for batch scans"),
    }
    Ok(())
}

pub fn write(report: &Report, format: Format, mut out: impl Write) -> Result<()> {
    match format {
        Format::Json => { serde_json::to_writer_pretty(&mut out, report)?; writeln!(out)?; }
        Format::Jsonl => { serde_json::to_writer(&mut out, report)?; writeln!(out)?; }
        Format::Csv => csv(std::slice::from_ref(report), out)?,
        Format::Cyclonedx => {
            let components: Vec<_> = report.assets.iter().map(|a| {
                let mut component = json!({
                "type": if a.kind == "model" { "machine-learning-model" } else { "library" },
                "bom-ref": a.id, "name": a.name,
                "properties": [
                    {"name":"pilotlight:kind", "value":a.kind},
                    {"name":"pilotlight:framework", "value":a.framework.as_deref().unwrap_or("")},
                    {"name":"pilotlight:confidence", "value":a.confidence},
                    {"name":"pilotlight:path", "value":a.evidence[0].path},
                    {"name":"pilotlight:line", "value":a.evidence[0].line.to_string()}
                ]
            });
                let properties = component["properties"].as_array_mut().expect("properties array");
                properties.push(json!({"name":"pilotlight:detection_rules","value":serde_json::to_string(&a.detection_rules).expect("serializable rules")}));
                properties.push(json!({"name":"pilotlight:column","value":a.evidence[0].column.to_string()}));
                if !a.evidence[0].resolution_path.is_empty() { properties.push(json!({"name":"pilotlight:resolution_path","value":serde_json::to_string(&a.evidence[0].resolution_path).expect("serializable provenance")})); }
                for (key, value) in &a.metadata { properties.push(json!({"name":format!("pilotlight:metadata:{key}"),"value":value})); }
                if let Some(root) = &a.application_root { properties.push(json!({"name":"pilotlight:application_root","value":root})); }
                if let Some(subtype) = &a.subtype { properties.push(json!({"name":"pilotlight:subtype","value":subtype})); }
                if let Some(cell) = a.evidence[0].cell { properties.push(json!({"name":"pilotlight:cell","value":cell.to_string()})); }
                if let Some(package) = &a.package {
                    properties.push(json!({"name":"pilotlight:package","value":serde_json::to_string(package).expect("serializable package")}));
                    if let Some(version) = &package.version { component["version"] = json!(version); }
                }
                component
            }).collect();
            let mut properties: Vec<_> = report.metadata.iter().map(|(k,v)| json!({"name":format!("pilotlight:metadata:{k}"),"value":v})).collect();
            properties.push(json!({"name":"pilotlight:detection_packs","value":serde_json::to_string(&report.detection_packs)?}));
            properties.push(json!({"name":"pilotlight:catalog:version","value":report.catalog_version}));
            properties.push(json!({"name":"pilotlight:catalog:sha256","value":report.catalog_sha256}));
            properties.push(json!({"name":"pilotlight:coverage:diagnostics","value":report.coverage.diagnostics.len().to_string()}));
            properties.push(json!({"name":"pilotlight:selection","value":serde_json::to_string(&report.selection)?}));
            properties.push(json!({"name":"pilotlight:coverage","value":serde_json::to_string(&report.coverage)?}));
            properties.push(json!({"name":"pilotlight:status","value":report.status}));
            properties.push(json!({"name":"pilotlight:config:sha256","value":report.config_sha256}));
            if let Some(revision) = &report.revision { properties.push(json!({"name":"pilotlight:revision","value":revision})); }
            let dependencies: Vec<_> = report.assets.iter().map(|a| {
                let targets: std::collections::BTreeSet<_> = report.relationships.iter().filter(|r| r.from == a.id).map(|r| &r.to).collect();
                json!({"ref":a.id,"dependsOn":targets})
            }).collect();
            let bom = json!({"$schema":"http://cyclonedx.org/schema/bom-1.6.schema.json", "bomFormat":"CycloneDX", "specVersion":"1.6", "version":1,
                "metadata":{"component":{"type":"application","name":report.source},"tools":{"components":[{"type":"application","name":"pilotlight","version":report.scanner_version}]},"properties":properties},
                "components":components,"dependencies":dependencies});
            serde_json::to_writer_pretty(&mut out, &bom)?; writeln!(out)?;
        }
        Format::Table => {
            writeln!(out, "Pilot Light {} — static AI inventory", report.scanner_version)?;
            writeln!(out, "Source: {}", clean(&report.source))?;
            writeln!(out, "Status: {}", report.status)?;
            writeln!(out, "Selected types: {}", report.selection.effective.iter().cloned().collect::<Vec<_>>().join(", "))?;
            if let Some(revision) = &report.revision { writeln!(out, "Commit: {revision}")?; }
            for (key, value) in &report.metadata { writeln!(out, "{}: {}", clean(key), clean(value))?; }
            writeln!(out, "\n{:<18} {:<32} LOCATION", "KIND", "NAME")?;
            for asset in &report.assets {
                let evidence = &asset.evidence[0];
                writeln!(out, "{:<18} {:<32} {}:{}:{}{}", clean(&asset.kind), clean(&asset.name), clean(&evidence.path), evidence.line, evidence.column, evidence.cell.map(|c| format!(" (cell {c})")).unwrap_or_default())?;
            }
            if report.assets.is_empty() { writeln!(out, "No catalog-matched AI assets found in supported files.")?; }
            writeln!(out, "\n{} findings | {} files scanned | {} files skipped | {} diagnostics", report.assets.len(), report.coverage.files_scanned, report.coverage.files_skipped, report.coverage.diagnostics.len())?;
            writeln!(out, "Catalog: {} | Static evidence does not prove runtime use.", report.catalog_version)?;
            for (kind, coverage) in &report.coverage.by_type {
                if coverage.status != "not_requested" { writeln!(out, "  {kind}: {} ({} findings)", coverage.status, coverage.findings)?; }
            }
            for diagnostic in report.coverage.diagnostics.iter().take(20) { writeln!(out, "  warning: {}: {}", clean(&diagnostic.path), clean(&diagnostic.reason))?; }
            if report.coverage.diagnostics.len() > 20 { writeln!(out, "  More diagnostics available in JSON output.")?; }
        }
    }
    Ok(())
}

fn csv(reports: &[Report], out: impl Write) -> Result<()> {
    let mut csv = csv::WriterBuilder::new().from_writer(out);
    let mut header: Vec<String> = ["source", "revision", "asset_id", "kind", "name", "framework", "confidence", "path", "line", "mechanism", "catalog_version", "diagnostics", "subtype", "application_root", "cell", "package_ecosystem", "package_name", "package_version", "dependency_scope", "resolution", "column", "resolution_path", "detection_rules"].iter().map(|s| (*s).into()).collect();
    let metadata_keys: std::collections::BTreeSet<_> = reports.iter().flat_map(|r| r.metadata.keys().chain(r.assets.iter().flat_map(|a| a.metadata.keys())).chain(r.applications.iter().flat_map(|a| a.metadata.keys()))).collect();
    header.extend(metadata_keys.iter().map(|k| format!("metadata.{k}")));
    csv.write_record(header.iter().map(|s| spreadsheet_safe(s)))?;
    for report in reports {
    for asset in &report.assets {
        for evidence in &asset.evidence {
            let mut row = vec![report.source.clone(), report.revision.clone().unwrap_or_default(), asset.id.clone(), asset.kind.clone(), asset.name.clone(), asset.framework.clone().unwrap_or_default(), asset.confidence.clone(), evidence.path.clone(), evidence.line.to_string(), evidence.mechanism.clone(), report.catalog_version.clone(), report.coverage.diagnostics.len().to_string(), asset.subtype.clone().unwrap_or_default(), asset.application_root.clone().unwrap_or_default()];
            row.push(evidence.cell.map(|v| v.to_string()).unwrap_or_default());
            if let Some(package) = &asset.package { row.extend([package.ecosystem.clone(), package.name.clone(), package.version.clone().unwrap_or_default(), package.scope.clone(), package.resolution.clone()]); }
            else { row.extend(std::iter::repeat_n(String::new(), 5)); }
            row.push(evidence.column.to_string());
            row.push(serde_json::to_string(&evidence.resolution_path)?);
            row.push(serde_json::to_string(&asset.detection_rules)?);
            row.extend(metadata_keys.iter().map(|key| asset.metadata.get(*key).or_else(|| report.metadata.get(*key)).cloned().unwrap_or_default()));
            csv.write_record(row.iter().map(|s| spreadsheet_safe(s)))?;
        }
    }
    }
    csv.flush()?; Ok(())
}

fn clean(value: &str) -> String { value.chars().map(|c| if c.is_control() { ' ' } else { c }).collect() }
fn spreadsheet_safe(value: &str) -> String {
    if value.trim_start().starts_with(['=', '+', '-', '@']) || value.starts_with(['\t', '\r', '\n']) { format!("'{value}") } else { value.into() }
}

/// Companion audit survives zero findings and binds to the exact primary report bytes.
pub fn coverage(reports: &[Report], primary: &[u8], mut out: impl Write) -> Result<()> {
    let repositories: Vec<_> = reports.iter().map(|r| json!({
        "source": r.source, "revision": r.revision, "status": r.status, "scanner_version": r.scanner_version,
        "catalog_version": r.catalog_version, "catalog_sha256": r.catalog_sha256, "config_sha256":r.config_sha256,
        "detection_packs": r.detection_packs,
        "metadata": r.metadata, "applications": r.applications,
        "selection": r.selection, "coverage": r.coverage
    })).collect();
    serde_json::to_writer_pretty(&mut out, &json!({"schema_version":1,"primary_sha256":crate::model::digest(primary),"repositories":repositories}))?;
    writeln!(out)?;
    Ok(())
}

pub fn relationships(reports: &[Report], out: impl Write) -> Result<()> {
    let mut writer = csv::Writer::from_writer(out);
    writer.write_record(["source", "revision", "from", "to", "kind"])?;
    for report in reports {
        for edge in &report.relationships {
            writer.write_record([report.source.as_str(), report.revision.as_deref().unwrap_or(""), &edge.from, &edge.to, &edge.kind].map(spreadsheet_safe))?;
        }
    }
    writer.flush()?;
    Ok(())
}
