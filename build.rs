fn main() {
    // DuckDB 1.4's Windows file-lock diagnostics use the Restart Manager API.
    if std::env::var_os("CARGO_FEATURE_DUCKDB_CATALOG").is_some()
        && std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
    {
        println!("cargo:rustc-link-lib=Rstrtmgr");
    }
}
