use vibeos_lint::{module_inputs::ModuleModel, Corpus, SourceFile};
fn corpus(items: &[(&str, &str)]) -> Corpus {
    Corpus::new(
        ".".into(),
        items
            .iter()
            .map(|(p, s)| SourceFile::new((*p).into(), (*s).into()))
            .collect(),
    )
}
#[test]
fn module_model_keeps_live_transitive_c_and_excludes_nested_targets() {
    let c = corpus(&[
        (
            "dll/example/CMakeLists.txt",
            r#"include(sources.cmake)
if(UNKNOWN)
 add_library(unresolved STATIC conditional.c)
endif()
add_library(example MODULE ${SOURCE})
target_link_libraries(example helper)
rust_crate_sources(EX_RUST "${WS}" foo-rust)
"#,
        ),
        (
            "dll/example/sources.cmake",
            r#"set(SOURCE live.c ${CMAKE_CURRENT_SOURCE_DIR}/absolute.c)
if(ARCH STREQUAL "arm64")
 list(APPEND SOURCE other_arch.c)
endif()
"#,
        ),
        ("dll/example/live.c", ""),
        ("dll/example/absolute.c", ""),
        ("dll/example/conditional.c", ""),
        ("dll/example/other_arch.c", ""),
        ("dll/example/obsolete.c", ""),
        ("dll/example/rust_shim.c", ""),
        (
            "dll/example/sub/CMakeLists.txt",
            "add_library(sub STATIC child.c)",
        ),
        ("dll/example/sub/child.c", ""),
        (
            "sdk/lib/helper/CMakeLists.txt",
            "add_library(helper STATIC helper.c)",
        ),
        ("sdk/lib/helper/helper.c", ""),
        (
            "sdk/lib/rust/foo-rust/Cargo.toml",
            "[package]\nname=\"foo-rust\"",
        ),
        ("sdk/lib/rust/foo-rust/src/lib.rs", "mod sub;"),
        ("sdk/lib/rust/foo-rust/src/sub.rs", ""),
    ]);
    let model = ModuleModel::from_corpus(&c);
    let inputs = &model.modules[std::path::Path::new("dll/example")];
    assert_eq!(
        inputs.c_files,
        vec![
            std::path::PathBuf::from("dll/example/obsolete.c"),
            std::path::PathBuf::from("dll/example/other_arch.c")
        ]
    );
    assert_eq!(inputs.glue.len(), 3);
    assert!(inputs.glue.iter().any(|p| p.ends_with("absolute.c")));
    assert_eq!(inputs.rust_files.len(), 2);
}
#[test]
fn crate_cfg_features_follow_alias_requests_and_skip_dependency_feature_atoms() {
    let c = corpus(&[
        (
            "sdk/lib/rust/consumer/Cargo.toml",
            r#"[package]
name="consumer"
[build-dependencies]
alias={package="provider",path="../provider",features=["requested"]}
[target.'cfg(windows)'.dependencies]
provider={path="../provider",features=["target-request"]}
"#,
        ),
        (
            "sdk/lib/rust/provider/Cargo.toml",
            r#"[package]
name="provider"
[features]
default=[
 "base", # retained before this comment
 "nested",
]
base=[]
nested=["leaf", "dep:optional", "optional/feature"]
leaf=[]
requested=[]
target-request=[]
"#,
        ),
        (
            "sdk/lib/rust/provider/build.rs",
            "println!(\"cargo:rustc-cfg=custom_flag\");",
        ),
        ("sdk/lib/rust/provider/src/lib.rs", ""),
    ]);
    let cfg = c.crate_cfg(std::path::Path::new("sdk/lib/rust/provider/src/lib.rs"));
    assert_eq!(
        cfg.on,
        std::collections::BTreeSet::from([
            "base".into(),
            "nested".into(),
            "leaf".into(),
            "requested".into(),
            "target-request".into()
        ])
    );
    assert!(cfg.emitted.contains("custom_flag"));
    assert!(!cfg.declared.contains("default"));
}
