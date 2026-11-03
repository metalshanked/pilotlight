#[cfg(feature = "duckdb-catalog")]
#[test]
fn preserves_raw_schema_and_rows_but_does_not_infer_roles_from_upstream_concepts() {
    use pilotlight::{catalog::Catalog, cisco, model::digest, syntax};
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source.duckdb");
    let output = temp.path().join("seeded.duckdb");
    let conn = duckdb::Connection::open(&source).unwrap();
    conn.execute_batch("CREATE TABLE component_catalog (id VARCHAR, label VARCHAR, concept VARCHAR, framework VARCHAR, sig_name VARCHAR, type VARCHAR, catalog_label VARCHAR);
        INSERT INTO component_catalog VALUES
        ('openai.OpenAI','class','model','openai',NULL,NULL,NULL),
        ('openai.types.BaseModel','class','model','openai',NULL,NULL,NULL),
        ('openai.types.BaseModel','class','model','openai',NULL,NULL,NULL),
        ('openai.types.BaseModel.model','parameter','model','openai','','Any',NULL),
        ('unknown.NewThing','function','agent','unknown',NULL,NULL,NULL);").unwrap();
    drop(conn);
    let original = std::fs::read(&source).unwrap();
    let sha = digest(&original);
    let overlay = Catalog::parse(include_bytes!("../catalog/overlay.json")).unwrap();
    assert!(cisco::import(&source,&output,&"0".repeat(64),"https://example.com/seed",&overlay).is_err());
    assert!(!output.exists());
    let merged = cisco::import(&source,&output,&sha,"https://example.com/seed",&overlay).unwrap();
    assert_eq!(merged.seed.as_ref().unwrap().rows,5);
    assert_eq!(merged.seed.as_ref().unwrap().indexed_api_references,2);
    assert_eq!(merged.seed.as_ref().unwrap().unmapped_callable_rows,1);
    assert_eq!(original,std::fs::read(&source).unwrap());
    let conn = duckdb::Connection::open(&output).unwrap();
    assert_eq!(conn.query_row("SELECT count(*) FROM component_catalog",[],|r|r.get::<_,u64>(0)).unwrap(),5);
    drop(conn);
    let loaded = Catalog::load(Some(&output)).unwrap();
    assert_eq!(loaded.fingerprint(),merged.fingerprint());
    let result = syntax::scan("app.py","from openai import OpenAI\nclient = OpenAI()\nfrom openai.types import BaseModel\nx = BaseModel(model='ordinary-data-field')\n", "python", &loaded).unwrap();
    assert_eq!(result.assets.iter().filter(|a|a.kind=="llm_client").count(),1);
    assert_eq!(result.assets.iter().filter(|a|a.subtype.as_deref()==Some("upstream_api_reference")).count(),1);
    assert!(!result.assets.iter().any(|a| a.kind=="model"));
    assert!(cisco::import(&source,&output,&sha,"https://example.com/seed",&overlay).is_err());
}
