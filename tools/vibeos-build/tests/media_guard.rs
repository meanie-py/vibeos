use serde_json::{json, Value};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
use vibeos_build::media_manifest as m;

fn invoke(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vibeos-build"))
        .arg("--root")
        .arg(root)
        .arg("media-manifest")
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn stage_counts_span_media_and_require_the_directory_to_vanish() {
    let entries: Vec<Value> = (0..5)
        .map(|i| json!([format!("reactos/syswow64/m{i}.dll"), "x"]))
        .collect();
    let have = json!({"cds":{
        "livecd":{"entries": entries},
        "bootcd":{"entries": entries},
        "bootcdregtest":{"entries":[[null,"x"],["readme.txt","x"],["a/b","x"]]}
    }});
    assert_eq!(
        m::vanished_dirs(&have, &json!({"cds":{}})).unwrap(),
        vec!["reactos/syswow64"]
    );
    let reduced = json!({"cds":{"livecd":{"entries":[["reactos/syswow64/one.dll", "x"]]}}});
    assert!(m::vanished_dirs(&have, &reduced).unwrap().is_empty());
    let small = json!({"cds":{"livecd":{"entries": (0..9).map(|i| json!([format!("reactos/syswow64/{i}"), "x"])).collect::<Vec<_>>()}}});
    assert!(m::vanished_dirs(&small, &json!({"cds":{}}))
        .unwrap()
        .is_empty());
}

#[test]
fn write_refusal_preserves_every_arch_and_override_is_explicit() {
    let root = std::env::temp_dir().join(format!("media-guard-test-{}", std::process::id()));
    fs::create_dir_all(root.join("boot/media")).unwrap();
    let mut originals = Vec::new();
    for arch in ["i386", "amd64", "arm64"] {
        let build = root.join(format!("build-{arch}"));
        fs::create_dir_all(build.join("boot")).unwrap();
        fs::write(build.join("build.ninja"), "# empty configured graph\n").unwrap();
        for cd in m::CDS {
            fs::write(
                build.join(format!("boot/{cd}.Debug.lst")),
                format!("reactos/system32/a.dll=/tmp/vibeos/build-{arch}/a.dll\n"),
            )
            .unwrap();
        }
        let entries: Vec<_> = if arch == "amd64" {
            (0..12)
                .map(|i| json!([format!("reactos/syswow64/m{i}.dll"), "build:old.dll"]))
                .collect()
        } else {
            vec![]
        };
        let bytes = serde_json::to_vec(
            &json!({"config":"Debug","cds":{"livecd":{"entries":entries,"command":"old"}}}),
        )
        .unwrap();
        let path = m::manifest_path(arch, &root);
        fs::write(&path, &bytes).unwrap();
        originals.push((path, bytes));
    }
    let result = invoke(&root, &["--write"]);
    assert_eq!(result.status.code(), Some(2));
    let error = String::from_utf8_lossy(&result.stderr);
    assert!(error.contains("reactos/syswow64"), "{error}");
    assert!(error.contains("--allow-drop"), "{error}");
    for (path, bytes) in &originals {
        assert_eq!(&fs::read(path).unwrap(), bytes);
    }
    // The library entry point used by Cargo-cell refresh has the same guard.
    assert!(m::write_plan(&root, &["amd64".into()]).is_err());
    let check = invoke(&root, &["--check"]);
    assert_eq!(check.status.code(), Some(1));
    assert!(!String::from_utf8_lossy(&check.stderr).contains("--allow-drop"));
    for (path, bytes) in &originals {
        assert_eq!(&fs::read(path).unwrap(), bytes);
    }
    let misuse = invoke(&root, &["--check", "--allow-drop"]);
    assert_eq!(misuse.status.code(), Some(2));
    let result = invoke(&root, &["--write", "--allow-drop"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(invoke(&root, &["--check"]).status.success());
    for (path, bytes) in &originals {
        assert_ne!(&fs::read(path).unwrap(), bytes);
    }
    fs::remove_dir_all(root).unwrap();
}
