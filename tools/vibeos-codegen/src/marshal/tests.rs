use super::render::*;
use super::*;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};
fn stage() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/marshal")
}
type Db = HashMap<String, Record>;
fn fixture() -> (
    Db,
    Db,
    HashMap<String, Proto>,
    Vec<String>,
    HashSet<String>,
    HashSet<String>,
) {
    let v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(stage().join("fixture.json")).unwrap()).unwrap();
    (
        serde_json::from_value(v["db32"].clone()).unwrap(),
        serde_json::from_value(v["db64"].clone()).unwrap(),
        serde_json::from_value(v["protos"].clone()).unwrap(),
        serde_json::from_value(v["manual"].clone()).unwrap(),
        serde_json::from_value(v["linkable"].clone()).unwrap(),
        serde_json::from_value(v["hand"].clone()).unwrap(),
    )
}
#[test]
fn whole_generation_matches_python() {
    let (d32, d64, protos, manual, links, hand) = fixture();
    let g = generate(
        &d32,
        &d64,
        &protos,
        &manual,
        &links,
        &hand,
        "exports-input.txt",
    );
    assert_eq!(
        g.c,
        fs::read_to_string(stage().join("oracle-gen.txt")).unwrap()
    );
    assert_eq!((g.thunks, g.struct_pairs, g.skipped.len()), (2, 2, 5));
}
#[test]
fn callback_generation_matches_python() {
    let (d32, d64, _, _, _, _) = fixture();
    let g = generate_callback(&d32, &d64).unwrap();
    assert_eq!(
        g.c,
        fs::read_to_string(stage().join("oracle-out.txt")).unwrap()
    );
    assert_eq!(
        g.header,
        fs::read_to_string(stage().join("oracle-out-header.txt")).unwrap()
    );
    assert_eq!((g.table_messages, g.struct_pairs), (6, 7));
}
#[test]
fn failures_and_helpers() {
    let (d32, d64, p, _, _, _) = fixture();
    assert_eq!(element_size("BYTE[4]", &d32, 4), Some(4));
    assert_eq!(param_slots32(&p["NtFlat"].params[1], &d32), 2);
    assert!(is_deref_pointer("void *", &d32));
    assert!(!is_deref_pointer("HWND", &d32));
    assert!(flat_arg_exprs(&p["NtBig"], &d32).is_none());
    let mut no = d32.clone();
    no.remove("struct _DRAWITEMSTRUCT");
    assert!(generate_callback(&no, &d64)
        .unwrap_err()
        .contains("DRAWITEMSTRUCT: not in layout dump"));
}
