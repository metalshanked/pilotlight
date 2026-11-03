use std::{fs, process::Command};

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_pilotlight"))
}

#[test]
fn malformed_operator_config_does_not_echo_private_values() {
    let temp = tempfile::tempdir().unwrap();
    let marker = "PRIVATE_OPERATOR_VALUE_DO_NOT_EXPORT";
    // A TOML type error normally includes the complete source line in its display.
    fs::write(temp.path().join("config.toml"), format!("indicators = '{marker}'\n")).unwrap();
    let result = cli().current_dir(temp.path()).args(["scan", ".", "--config", "config.toml"]).output().unwrap();
    assert_eq!(result.status.code(), Some(1));
    let error = String::from_utf8_lossy(&result.stderr);
    assert!(error.contains("invalid Pilot Light config"));
    assert!(!error.contains(marker));
    assert!(result.stdout.is_empty());
}

#[test]
fn batch_csv_and_json_preserve_repository_metadata() {
    let temp = tempfile::tempdir().unwrap();
    for (repo, id) in [("first", 1234), ("second", 5678)] {
        let root = temp.path().join(repo);
        fs::create_dir(&root).unwrap();
        fs::write(root.join("app.py"), "import openai\n").unwrap();
        fs::write(root.join("info.yaml"), format!("appid: {id}\n")).unwrap();
    }
    fs::write(temp.path().join("repos.txt"), "# local fixtures\nfirst\n\nsecond\n").unwrap();
    let args = ["scan", "--repos-file", "repos.txt", "--indicator-path", "info.yaml", "--indicator-key", "appid", "--indicator-required", "--format"];
    let json = cli().current_dir(temp.path()).args(args).arg("json").output().unwrap();
    assert!(json.status.success(), "{}", String::from_utf8_lossy(&json.stderr));
    let value: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    let reports = value["reports"].as_array().unwrap();
    assert_eq!(reports.len(), 2);
    assert_eq!(reports[0]["metadata"]["application_id"], "1234");
    assert_eq!(reports[1]["metadata"]["application_id"], "5678");
    let csv = cli().current_dir(temp.path()).args(args).args(["csv", "--coverage-output", "coverage.json"]).output().unwrap();
    assert!(csv.status.success());
    let mut reader = csv::Reader::from_reader(csv.stdout.as_slice());
    let column = reader.headers().unwrap().iter().position(|h| h == "metadata.application_id").unwrap();
    let ids: Vec<_> = reader.records().map(|r| r.unwrap()[column].to_owned()).collect();
    assert_eq!(ids, ["1234", "5678"]);
}

#[test]
fn strict_exit_and_fatal_scan_preserve_existing_output() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("package.json"), "{broken").unwrap();
    let strict = cli().current_dir(temp.path()).args(["scan", ".", "--strict", "--format", "json"]).output().unwrap();
    assert_eq!(strict.status.code(), Some(2));
    let value: serde_json::Value = serde_json::from_slice(&strict.stdout).unwrap();
    assert_eq!(value["coverage"]["diagnostics"].as_array().unwrap().len(), 1);
    fs::write(temp.path().join("inventory.csv"), "previous report").unwrap();
    let fatal = cli().current_dir(temp.path()).args(["scan", ".", "--indicator-path", "missing.yaml", "--indicator-key", "appid", "--indicator-required", "--output", "inventory.csv"]).output().unwrap();
    assert_eq!(fatal.status.code(), Some(1));
    assert_eq!(fs::read_to_string(temp.path().join("inventory.csv")).unwrap(), "previous report");
}
