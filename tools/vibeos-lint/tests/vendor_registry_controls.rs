//! Integration controls for public vendor-registry audit branches not yet represented by
//! `vendor_registry.rs`'s local tests. Include this module from the crate test target.
use std::path::PathBuf;
use vibeos_lint::{
    vendor_registry::{audit, AuditSnapshot},
    Corpus,
};

fn corpus() -> Corpus {
    Corpus::new(PathBuf::from("."), Vec::new())
}
fn yaml(s: &str) -> serde_json::Value {
    vibeos_lint::yaml::parse(s).expect("fixture YAML is valid")
}
fn file(s: &mut AuditSnapshot, p: &str, body: &str) {
    s.files.insert(p.into(), body.into());
    if !s.ordered_files.iter().any(|x| x == p) {
        s.ordered_files.push(p.into());
    }
}
fn dir(s: &mut AuditSnapshot, p: &str) {
    s.directories.insert(p.into());
    if !s.ordered_directories.iter().any(|x| x == p) {
        s.ordered_directories.push(p.into());
    }
}
fn registry(extra: &str) -> serde_json::Value {
    yaml(&format!(
        "containers: []\ntrees:\n  - path: vendor/a\n    name: A\n    reason: fixture\n    license: MIT\n    license_note: no grant shipped in fixture\n{extra}"
    ))
}
fn audit_fixture(registry: &serde_json::Value, snap: &AuditSnapshot, crates: &str) -> Vec<String> {
    audit(&corpus(), registry, &yaml(crates), snap).expect("audit should parse fixture")
}

#[test]
fn required_license_paths_version_probe_translate_verdict_and_stamp_are_checked() {
    let reg = yaml(
        "containers: []\ntrees:\n  - path: vendor/a\n    name: A\n    reason: fixture\n    license: MIT\n    license_file: vendor/MISSING-LICENSE\n    license_ref: vendor/MISSING-HEADER\n    patch: vendor/MISSING.patch\n    headers: [vendor/MISSING-headers]\n    version:\n      file: vendor/version.h\n      pattern: 'VERSION ([0-9]+)'\n    translate: no\n",
    );
    let mut snap = AuditSnapshot::default();
    file(&mut snap, "vendor.yaml", "trees:\n");
    dir(&mut snap, "vendor");
    dir(&mut snap, "vendor/a");
    file(&mut snap, "vendor/version.h", "no version here\n");
    let got = audit_fixture(&reg, &snap, "{crates: []}");
    for needle in [
        "license_file does not exist",
        "license_ref does not exist",
        "patch does not exist",
        "header path does not exist",
        "version probe found no match",
        "translate:",
        "missing vendor/a/VENDORED",
    ] {
        assert!(
            got.iter().any(|x| x.contains(needle)),
            "missing {needle}: {got:#?}"
        );
    }
}

#[test]
fn missing_ours_path_and_missing_crate_snapshot_are_findings() {
    let reg = yaml(
        "containers: []\ntrees:\n  - path: vendor/a\n    name: A\n    reason: fixture\n    license_note: no grant\nours:\n  - path: ours/gone\n    name: O\n    reason: fixture\n",
    );
    let mut snap = AuditSnapshot::default();
    file(&mut snap, "vendor.yaml", "trees:\n");
    file(&mut snap, "sdk/lib/rust/Cargo.lock", "[[package]]\nname = \"external\"\nversion = \"1.0\"\nsource = \"registry+https://example.invalid\"\n");
    let got = audit_fixture(&reg, &snap, "{crates: []}");
    assert!(got.iter().any(|x| x.contains("ours/gone")), "{got:#?}");
    assert!(
        got.iter()
            .any(|x| x.contains("tools/rust-crates.yaml is MISSING")),
        "{got:#?}"
    );
}

#[test]
fn nested_checkout_marker_prunes_foreign_files_but_an_unmarked_sibling_is_found() {
    let reg = registry("");
    let mut snap = AuditSnapshot::default();
    file(&mut snap, "vendor.yaml", "trees:\n");
    for d in ["vendor", "nested", "nested/checkout", "nested/ours"] {
        dir(&mut snap, d);
    }
    snap.markers.insert("nested/checkout/.git".into());
    for i in 0..5 {
        file(
            &mut snap,
            &format!("nested/checkout/f{i}.c"),
            "/* Copyright (c) Microsoft Corporation */\n",
        );
        file(
            &mut snap,
            &format!("nested/ours/f{i}.c"),
            "/* Copyright (c) Microsoft Corporation */\n",
        );
    }
    let got = audit_fixture(&reg, &snap, "{crates: []}");
    assert!(
        !got.iter().any(|x| x.contains("nested/checkout")),
        "nested worktree leaked: {got:#?}"
    );
    assert!(
        got.iter()
            .any(|x| x.contains("UNCLASSIFIED foreign-copyright code: nested/ours")),
        "sibling was hidden: {got:#?}"
    );
}

#[test]
fn license_alias_resolves_only_while_the_underlying_grant_ships() {
    let reg = registry("");
    let mut snap = AuditSnapshot::default();
    file(&mut snap, "vendor.yaml", "trees:\n");
    dir(&mut snap, "vendor");
    dir(&mut snap, "vendor/a");
    dir(&mut snap, "drivers");
    dir(&mut snap, "drivers/psapi");
    file(
        &mut snap,
        "COPYING.LIB",
        "GNU LESSER GENERAL PUBLIC LICENSE\n",
    );
    file(&mut snap, "drivers/psapi/a.c", "/* Copyright (c) 2003 Somebody\n * LICENSE: See LGPL.txt in the top level directory\n */\n");
    let good = audit_fixture(&reg, &snap, "{crates: []}");
    assert!(
        !good.iter().any(|x| x.contains("ptr:lgpl.txt")),
        "alias failed: {good:#?}"
    );

    snap.files.remove("COPYING.LIB");
    let bad = audit_fixture(&reg, &snap, "{crates: []}");
    assert!(
        bad.iter().any(|x| x.contains("ptr:lgpl.txt")),
        "alias invented a grant: {bad:#?}"
    );
}

#[test]
fn notices_must_match_generated_content_and_recorded_translate_no_is_invalid() {
    let reg = registry("");
    let mut snap = AuditSnapshot::default();
    file(&mut snap, "vendor.yaml", "trees:\n");
    file(&mut snap, "vendor/a", "one-file tree\n");
    file(
        &mut snap,
        "THIRD-PARTY-NOTICES.md",
        "hand-written stale contents\n",
    );
    let got = audit_fixture(&reg, &snap, "{crates: []}");
    assert!(
        got.iter()
            .any(|x| x.contains("THIRD-PARTY-NOTICES.md is STALE")),
        "{got:#?}"
    );
}
