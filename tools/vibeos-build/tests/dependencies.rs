use serde_json::json;
use vibeos_build::{cargo, depfile};
#[test]
fn retargeting_preserves_bytes_and_refuses_bad_prefix() {
    let body = " a.rs space\\ name.bin \\\n b.rs\na.rs:\nb.rs:\n";
    assert_eq!(
        depfile::retarget(
            &format!("/cargo/my\\ app:{body}"),
            "/cargo/my app",
            "module/my app"
        )
        .unwrap(),
        format!("module/my\\ app:{body}")
    );
    for text in ["", "/wrong: a.rs\n", "/cargo/app:a.rs\n"] {
        assert!(depfile::retarget(text, "/cargo/app", "app").is_err());
    }
    for target in [
        "line\nbreak",
        "hash#name",
        "dollar$name",
        "colon:name",
        "back\\slash",
    ] {
        assert!(depfile::retarget("/cargo/app: a.rs\n", "/cargo/app", target).is_err());
    }
}
#[test]
fn resolved_closure_keeps_registry_bridges_and_cycles() {
    let metadata = json!({"packages":[{"id":"app","name":"app","source":null,"manifest_path":"/app/Cargo.toml"},{"id":"registry","name":"r","source":"registry","manifest_path":"/r/Cargo.toml"},{"id":"target","name":"target","source":null,"manifest_path":"/target/Cargo.toml"}],"resolve":{"nodes":[{"id":"app","dependencies":["registry"]},{"id":"registry","dependencies":["target"]},{"id":"target","dependencies":["app"]}]}});
    let scope = cargo::local_closure(&metadata, &["app".into()]).unwrap();
    assert_eq!(
        scope.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
        ["app", "target"]
    );
    assert!(cargo::local_closure(&metadata, &[]).is_err());
    assert!(cargo::local_closure(&metadata, &["r".into()]).is_err());
    let mut invalid = metadata.clone();
    invalid["resolve"]["nodes"] = json!([]);
    assert!(cargo::local_closure(&invalid, &["app".into()]).is_err());
}
