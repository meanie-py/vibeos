use std::{path::Path, process::Command};
use vibeos_build::{helper_api, link_guard, ninja};
#[test]
fn frozen_link() {
    let f: serde_json::Value = serde_json::from_str(include_str!("fixtures/link.json")).unwrap();
    for x in f["judgments"].as_array().unwrap() {
        let got = link_guard::judge(
            x["symbol"].as_str().unwrap(),
            x["kept"].as_str().unwrap(),
            x["dropped"].as_str().unwrap(),
        );
        assert_eq!(serde_json::to_value(got).unwrap(), *x);
    }
    assert_eq!(
        serde_json::to_value(link_guard::parse(f["stderr"].as_str().unwrap())).unwrap(),
        f["parsed"]
    );
    assert_eq!(link_guard::output_of(&["-ox.dll".into()]), "x.dll");
    assert!(helper_api::dispatch(
        Path::new("."),
        &serde_json::json!({"op":"link_judge","args":[42]})
    )
    .is_err());
}
#[test]
fn closure_and_safety() {
    let root = std::env::temp_dir().join(format!("ninja-fixture-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("build.ninja"),"include rules.ninja\nbuild app.exe | app.map: LINK app.obj || gen.h\nbuild app.obj: CC app.c\nbuild gen.h: GEN\nbuild dead.o: CC dead.c\nbuild sp$ ace.obj: CC sp$ ace.c\nbuild livecd: phony app.exe\n").unwrap();
    std::fs::write(root.join("rules.ninja"), "rule CC\n  command = cc\n").unwrap();
    let names = [
        "app.exe",
        "app.map",
        "app.obj",
        "gen.h",
        "dead.o",
        "sp ace.obj",
    ];
    for n in names {
        std::fs::write(root.join(n), "x").unwrap();
    }
    let log = names
        .iter()
        .map(|x| format!("1\t2\t3\t{x}\thash\n"))
        .collect::<String>();
    std::fs::write(root.join(".ninja_log"), log).unwrap();
    let p = ninja::plan(&root, &["livecd".into()], None).unwrap();
    let f: serde_json::Value = serde_json::from_str(include_str!("fixtures/ninja.json")).unwrap();
    assert_eq!(serde_json::json!([p.victims, p.unknown]), f["plan"]);
    let edges = ninja::parse_edges(&root.join("build.ninja")).unwrap();
    let edges: Vec<_> = edges
        .into_iter()
        .map(|e| serde_json::json!([e.outputs, e.inputs]))
        .collect();
    assert_eq!(serde_json::json!(edges), f["edges"]);
    let unknown = ninja::plan(&root, &["no-target".into()], None).unwrap();
    assert!(ninja::delete(&root, &unknown).is_err());
    assert!(root.join("dead.o").exists());
    #[cfg(unix)]
    {
        std::fs::remove_file(root.join("dead.o")).unwrap();
        std::os::unix::fs::symlink("/etc/hostname", root.join("dead.o")).unwrap();
        assert!(ninja::plan(&root, &["livecd".into()], None).is_err());
        std::fs::remove_file(root.join("dead.o")).unwrap();
    }
    std::fs::remove_file(root.join("rules.ninja")).unwrap();
    assert!(ninja::plan(&root, &["livecd".into()], None).is_err());
    std::fs::write(root.join("rules.ninja"), "build malformed\n").unwrap();
    assert!(ninja::plan(&root, &["livecd".into()], None).is_err());
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
#[cfg(unix)]
fn guard_refuses_and_reports() {
    use std::os::unix::fs::PermissionsExt;
    let root = std::env::temp_dir().join(format!("guard-fixture-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let linker = root.join("fake-ld");
    std::fs::write(&linker,"#!/bin/sh\nprintf x > \"$2\"\nprintf 'warning: duplicate symbol: foo\\n>>> defined at a.o\\n>>> defined at b.o\\n' >&2\n").unwrap();
    let mut perm = std::fs::metadata(&linker).unwrap().permissions();
    perm.set_mode(0o755);
    std::fs::set_permissions(&linker, perm).unwrap();
    let output = root.join("test.exe");
    let log = root.join("log.jsonl");
    let run = |report: bool| {
        let mut c = Command::new(env!("CARGO_BIN_EXE_vibeos-build"));
        c.arg("link-guard")
            .env("VIBEOS_REAL_LLD", &linker)
            .env("VIBEOS_LINK_GUARD_LOG", &log)
            .env_remove("VIBEOS_LINK_GUARD")
            .args(["-o", output.to_str().unwrap()]);
        if report {
            c.env("VIBEOS_LINK_GUARD", "report");
        }
        c.output().unwrap()
    };
    let p = run(false);
    assert_eq!(p.status.code(), Some(1));
    assert!(!output.exists());
    assert!(String::from_utf8_lossy(&p.stderr).contains("duplicate symbol(s)"));
    assert_eq!(run(true).status.code(), Some(0));
    assert!(output.exists());
    assert_eq!(std::fs::read_to_string(log).unwrap().lines().count(), 2);
    std::fs::remove_dir_all(root).unwrap();
}
