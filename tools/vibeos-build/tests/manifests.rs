use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use vibeos_build::{cell_manifest as c, manifest_api, media_manifest as m};
#[test]
fn frozen_cell_and_unicode_rendering() {
    let f: Value = serde_json::from_str(include_str!("fixtures/cellfixture.json")).unwrap();
    assert_eq!(
        c::parse_cell(
            f["outs"].as_str().unwrap(),
            f["command"].as_str().unwrap(),
            "i386"
        )
        .unwrap(),
        f["expected"]
    );
    assert!(c::parse_cell("different", f["command"].as_str().unwrap(), "i386").is_err());
    let f: Value = serde_json::from_str(include_str!("fixtures/mediafixture.json")).unwrap();
    assert_eq!(
        m::render(&f).unwrap(),
        include_str!("fixtures/mediafixture.expected")
    );
    assert!(m::committed(
        &json!({"cds":{"livecd":{"entries":[[null,"repo:modules/rostests/winetests/a"]]}}}),
        "amd64"
    )
    .is_err());
}
#[test]
fn committed_manifest_bytes_are_preserved() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for arch in ["i386", "amd64", "arm64"] {
        let path = c::manifest_path(arch, &root);
        let text = std::fs::read_to_string(path).unwrap();
        let cells: Vec<Value> = text
            .lines()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect();
        assert_eq!(c::render(&cells), text, "cells {arch}");
        let text = std::fs::read_to_string(m::manifest_path(arch, &root)).unwrap();
        assert_eq!(
            m::render(&serde_json::from_str(&text).unwrap()).unwrap(),
            text,
            "media {arch}"
        );
    }
}
#[test]
fn invalid_api_requests_fail() {
    assert!(manifest_api::dispatch(Path::new("."), &json!({"op":"cell_parse_cell"})).is_err());
    assert!(manifest_api::dispatch(Path::new("."), &json!({"op":"media_foo"})).is_err());
}
