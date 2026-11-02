use std::collections::BTreeSet;
use vibeos_codegen::cells::proof::{self, CheckRecord, CheckResult, PolicyInput};
#[test]
fn policy_frozen_python() {
    let cases: serde_json::Value = serde_json::from_str(include_str!("cells/policy.json")).unwrap();
    for c in cases.as_array().unwrap() {
        let list = |key: &str| {
            c[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_str().unwrap().to_owned())
                .collect::<BTreeSet<_>>()
        };
        let target = c["target"].as_str().unwrap();
        let arch = c["arch"].as_str().unwrap();
        let p = proof::apply_policy(PolicyInput {
            target,
            arch,
            diffs: c["diffs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_str().unwrap().to_owned())
                .collect(),
            panic_common: c["panic_common"].as_bool().unwrap(),
            bodyless_sizes: None,
            data_size: c["data_size"].as_u64().map(|x| x as u32),
            old_mapped: c["old_mapped"].as_u64().unwrap(),
            new_mapped: c["new_mapped"].as_u64().unwrap(),
            cd_old: if c["cd_old"].is_null() {
                None
            } else {
                Some(list("cd_old"))
            },
            cd_new: list("cd_new"),
            txtsetup: c["txtsetup"].as_bool().unwrap(),
        });
        let bad = usize::from(!p.diffs.is_empty());
        let result = CheckResult {
            records: vec![CheckRecord {
                target: target.into(),
                arch: arch.into(),
                status: if bad == 0 { "EQUIVALENT" } else { "DIFFERENT" }.into(),
                diffs: p.diffs,
                note: p.note,
            }],
            bad,
        };
        assert_eq!(
            result.report(),
            c["expected"].as_str().unwrap(),
            "{}",
            c["name"]
        );
    }
}
#[test]
fn rejects_unchanged_and_missing() {
    let root = std::env::temp_dir().join(format!("proof-test-{}", std::process::id()));
    let base = root.join("baseline");
    for a in ["i386", "amd64", "arm64"] {
        std::fs::create_dir_all(base.join(a)).unwrap();
        std::fs::create_dir_all(root.join(format!("build-{a}"))).unwrap();
        std::fs::write(base.join(a).join("foo.dll"), b"same").unwrap();
    }
    std::fs::write(root.join("build-i386/foo.dll"), b"same").unwrap();
    let result = proof::check(&root, &["foo".into()], &base).unwrap();
    assert_eq!(result.bad, 3);
    assert!(result
        .records
        .iter()
        .all(|r| r.status.starts_with("NOT RELINKED")));
    std::fs::write(base.join("i386/foo.exe"), b"other").unwrap();
    let result = proof::check(&root, &["foo".into()], &base).unwrap();
    assert_eq!(result.bad, 3);
    assert!(result.records[0].status.starts_with("NO BASELINE (2"));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn bodyless_i386_section_allowance_is_bounded_and_specific() {
    for (arch, bodyless, eh, reloc, expected) in [
        ("i386", true, 0x88, 0x48, 0),
        ("amd64", true, 0x88, 0x48, 2),
        ("i386", false, 0x88, 0x48, 2),
        ("i386", true, 0x401, 0x81, 2),
    ] {
        let result = proof::apply_policy(PolicyInput {
            target: "m",
            arch,
            diffs: vec![
                "sections: + .eh_fram 0x40000040".into(),
                "sections: + .reloc 0x42000040".into(),
            ],
            panic_common: false,
            data_size: None,
            bodyless_sizes: bodyless.then(|| {
                [(".eh_fram".into(), eh), (".reloc".into(), reloc)]
                    .into_iter()
                    .collect()
            }),
            old_mapped: 100,
            new_mapped: 100,
            cd_old: None,
            cd_new: Default::default(),
            txtsetup: false,
        });
        assert_eq!(result.diffs.len(), expected);
        if expected == 0 {
            assert_eq!(result.note, " (+ the shell's i386 .eh_fram/.reloc)");
        }
    }
}
