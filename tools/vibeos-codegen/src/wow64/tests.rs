use super::*;
use sha2::{Digest, Sha256};
use std::{collections::HashSet, fs, path::PathBuf};
fn stage() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/wow64")
}
#[test]
fn real_tables_match_python() {
    let temp = crate::resources::TempDir::new().unwrap();
    let root = temp.path();
    let inputs: std::collections::BTreeMap<String, String> =
        serde_json::from_str(include_str!("../../tests/wow64/real-inputs.json")).unwrap();
    for (name, text) in inputs {
        let file = root.join(name);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, text).unwrap();
    }
    let oracle: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(stage().join("real-oracles.json")).unwrap())
            .unwrap();
    for (name, sp, sf, lists, prefix) in [
        (
            "ntdll",
            "dll/ntdll/def/ntdll.spec",
            "ntoskrnl/include/sysfuncs.h",
            vec!["tools/wow64gen/manual.txt"],
            "Wow64Thunk",
        ),
        (
            "win32k",
            "win32ss/win32u/win32u.spec",
            "win32ss/w32ksvc32.h",
            vec![
                "tools/wow64gen/manual-win32k.txt",
                "tools/wow64gen/automarshal-win32k.txt",
            ],
            "Wow64Win32kThunk",
        ),
    ] {
        for arch in ["arm64", "amd64", "i386"] {
            let manual =
                load_manual_lists(&lists.iter().map(|p| root.join(p)).collect::<Vec<_>>()).unwrap();
            let specs = read_spec(&root.join(sp), arch, &manual).unwrap();
            let sys = read_sysfuncs(&root.join(sf)).unwrap();
            let table = order_by_sysfuncs(&sys, &specs);
            let out = emit_c(&table, sp, arch, "output.gen.c", prefix).unwrap();
            let expect = &oracle[format!("{name}/{arch}")];
            assert_eq!(
                format!("{:x}", Sha256::digest(out.as_bytes())),
                expect["sha256"].as_str().unwrap(),
                "{name} {arch}"
            );
            assert_eq!(table.len(), expect["count"].as_u64().unwrap() as usize);
            assert_eq!(
                table.iter().filter(|s| s.needs_manual()).count(),
                expect["manual"].as_u64().unwrap() as usize
            );
            let computed = compute_thunk_table(&root, name, arch).unwrap();
            for s in table {
                assert_eq!(computed[&s.name], !s.needs_manual());
            }
        }
    }
}
#[test]
fn synthetic_output_and_json_match() {
    let p = stage();
    let specs = read_spec(
        &p.join("synthetic.spec"),
        "amd64",
        &HashSet::from(["NtFoo".into()]),
    )
    .unwrap();
    let sys = read_sysfuncs(&p.join("synthetic.services")).unwrap();
    let table = order_by_sysfuncs(&sys, &specs);
    assert_eq!(
        emit_c(&table, "fixtures/spec path", "amd64", "-", "TestPrefix").unwrap(),
        fs::read_to_string(p.join("synthetic-output.txt")).unwrap()
    );
    let expected: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(p.join("synthetic-table.json")).unwrap()).unwrap();
    assert_eq!(serde_json::to_value(&table).unwrap(), expected);
}
#[test]
fn fail_missing_inputs_and_invalid_render() {
    let p = stage().join("no-such-file");
    assert!(read_sysfuncs(&p).is_err());
    assert!(load_manual_lists(&[p]).is_err());
    let bad = Syscall {
        name: "NtBad".into(),
        args: vec!["garbage".into()],
        ordinal: 0,
        arch: None,
        manual_reason: String::new(),
        slots: 0,
    };
    assert!(emit_c(&[bad], "s", "amd64", "-", "P").is_err());
    assert!(compute_thunk_table(&stage(), "bad", "amd64").is_err());
}
#[test]
fn legacy_classifier_and_duplicate_service_behavior() {
    let mut sc = Syscall {
        name: "NtA".into(),
        args: vec!["long".into(), "banana".into()],
        ordinal: -1,
        arch: None,
        manual_reason: String::new(),
        slots: 17,
    };
    classify(&mut sc);
    assert_eq!(sc.slots, 17);
    assert!(sc.manual_reason.contains("banana"));
    assert_eq!(arg_kinds().len(), 9);
    let specs = parse_spec("@ stdcall NtA(long)\n", "amd64", &HashSet::new());
    let table = order_by_sysfuncs(&[("NtA".into(), 2), ("NtA".into(), 1)], &specs);
    assert_eq!(table[0], table[1]);
    assert_eq!(table[0].ordinal, 1);
    assert_eq!(table[0].slots, 2);
    assert!(table[0].needs_manual());
}
