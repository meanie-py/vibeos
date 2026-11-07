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
/// MENUBARINFO's two 1-bit BOOLs share one storage unit that only MOVES between the
/// 32- and 64-bit layouts (28 -> 40). The unit is an ordinary scalar to copy; before
/// bitfield units the whole struct was refused as SEMANTIC and its syscall
/// (NtUserGetMenuBarInfo) shipped with a pointer passed straight through.
#[test]
fn moved_bitfield_unit_is_marshalled_as_one_scalar() {
    let dump = |hm: usize, hw: usize, bits: usize, size: usize| {
        format!(
            "*** Dumping AST Record Layout\n         0 | struct tagRECT\n         0 |   LONG left\n         4 |   LONG top\n         8 |   LONG right\n        12 |   LONG bottom\n           | [sizeof=16, align=4]\n*** Dumping AST Record Layout\n         0 | struct tagMENUBARINFO\n         0 |   DWORD cbSize\n         4 |   struct tagRECT rcBar\n         {hm} |   HMENU hMenu\n         {hw} |   HWND hwndMenu\n    {bits}:0-0 |   BOOL fBarFocused\n    {bits}:1-1 |   BOOL fFocused\n           | [sizeof={size}, align=4]\n"
        )
    };
    let d32 = parse_layout_dump(&dump(20, 24, 28, 32));
    let d64 = parse_layout_dump(&dump(24, 32, 40, 48));
    let r32 = &d32["struct tagMENUBARINFO"];
    assert_eq!(r32.marshal_fields().len(), 5);
    assert_eq!(r32.marshal_fields()[4].name, "fBarFocused/fFocused");
    let v = diff_struct("struct tagMENUBARINFO", &d32, &d64);
    assert_eq!(v.cls, "WIDENING", "{}", v.detail);

    let (mut emitted, mut out) = (Emitted(Vec::new()), Vec::new());
    assert!(gen_struct_marshaller(r32, &d64["struct tagMENUBARINFO"], &d32, &d64, &mut emitted, &mut out));
    let c = out.join("\n");
    assert!(c.contains("WowAmCpy(d + 40, s + 28, 4); /* fBarFocused/fFocused */"), "{c}");
    assert!(c.contains("WowAmSx(d + 24, s + 20); /* hMenu */"), "{c}");

    // Bits that land differently are still refused.
    let mut shifted = d64.clone();
    shifted.get_mut("struct tagMENUBARINFO").unwrap().fields[5].bits = "2-2".into(); // fFocused, was 1-1
    assert_eq!(diff_struct("struct tagMENUBARINFO", &d32, &shifted).cls, "SEMANTIC");
}
