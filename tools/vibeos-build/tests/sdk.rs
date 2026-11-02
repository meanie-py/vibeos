use serde_json::{json, Value};
use std::{fs, path::PathBuf};
use vibeos_build::{sdk_api, sdk_recipe as s};
fn fixture(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("sdk-{name}-{}", std::process::id()));
    let files: Value = serde_json::from_str(include_str!("fixtures/sdk-files.json")).unwrap();
    for (rel, text) in files.as_object().unwrap() {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text.as_str().unwrap()).unwrap();
    }
    fs::create_dir_all(root.join("build-amd64/host-tools/bin")).unwrap();
    root
}
#[test]
fn python_capture_and_factoring_parity() {
    let o: Value = serde_json::from_str(include_str!("fixtures/sdk-oracle.json")).unwrap();
    for (i, l) in o["samples"].as_array().unwrap().iter().enumerate() {
        let l = l.as_str().unwrap();
        assert_eq!(s::tool(l).unwrap(), o["tool"][i]);
        assert_eq!(s::keep(l).unwrap(), o["keep"][i]);
    }
    let lines: Vec<String> = serde_json::from_value(o["factor_lines"].clone()).unwrap();
    assert_eq!(json!(s::factor(&lines).unwrap()), o["factor"]);
    let root = fixture("capture");
    let (sdk, host) = s::derive(&root, "amd64").unwrap();
    assert_eq!(json!(sdk), o["sdk"]);
    assert_eq!(json!(host), o["host"]);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn replay_and_missing_recipe_refusal() {
    let root = fixture("replay");
    let gen = root.join("out");
    let p = s::replay_plan(&root, "amd64", &gen).unwrap();
    s::run(&p).unwrap();
    assert!(gen.join("host-tools/bin/sub/dir/tool.o").is_file());
    assert!(gen.join("a/b/c.h").is_file());
    assert!(s::replay_plan(&root, "i386", &root.join("missing")).is_err());
    assert!(!root.join("missing").exists());
    assert!(sdk_api::dispatch(
        &root,
        &json!({"op":"sdk__tool","kwargs":{"line":"cd nowhere"}})
    )
    .is_err());
    fs::remove_dir_all(root).unwrap();
}
