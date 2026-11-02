use serde_json::json;
use std::{fs, path::Path};
use vibeos_build::artifact_inputs::{self as inputs, DiscoverOptions, Discovery};

fn put(root: &Path, path: &str, text: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

#[test]
fn real_locked_resolution_preserves_keys_under_unrelated_workspace_changes() {
    let root = tempfile::Builder::new()
        .prefix("artifact-native-projection-")
        .tempdir()
        .unwrap();
    let repo = root.path();
    put(
        repo,
        "Cargo.toml",
        "[workspace]\nresolver=\"2\"\nmembers=[\"body\",\"dep\",\"unrelated\"]\n",
    );
    for name in ["body", "dep", "unrelated"] {
        let extra = if name == "body" {
            "[dependencies]\ndep={path=\"../dep\"}\n"
        } else {
            ""
        };
        put(
            repo,
            &format!("{name}/Cargo.toml"),
            &format!("[package]\nname=\"{name}\"\nversion=\"0.1.0\"\nedition=\"2021\"\n{extra}"),
        );
        put(
            repo,
            &format!("{name}/src/lib.rs"),
            "pub fn value() -> u8 { 7 }\n",
        );
    }
    put(repo, "Cargo.lock", "version=4\n[[package]]\nname=\"body\"\nversion=\"0.1.0\"\ndependencies=[\"dep\"]\n[[package]]\nname=\"dep\"\nversion=\"0.1.0\"\n[[package]]\nname=\"unrelated\"\nversion=\"0.1.0\"\n");
    let options = DiscoverOptions {
        workspace: Some(repo.to_owned()),
        ..Default::default()
    };
    let context = json!({"target":{},"effective_flags":[],"effective_config":{},"toolchain":{},"environment":{},"execution_policy":"fixture"});
    let baseline = inputs::discover(repo, "body", context.clone(), &options);
    assert_eq!(baseline["eligible"], true, "{}", baseline);
    assert!(baseline["descriptor"]["packages"]
        .get("unrelated")
        .is_none());
    assert!(inputs::unchanged(&baseline));
    put(repo, "unrelated/src/lib.rs", "pub fn unrelated_edit() {}\n");
    assert!(inputs::unchanged(&baseline));
    put(repo, "body/Cargo.toml", "# Formatting is not a compiler input.\n[package]\nname=\"body\"\nversion=\"0.1.0\"\nedition=\"2021\"\n[dependencies]\ndep={path=\"../dep\"}\n");
    assert!(!inputs::unchanged(&baseline));
    let formatted = inputs::discover(repo, "body", context.clone(), &options);
    assert_eq!(formatted["action_key"], baseline["action_key"]);
    assert_ne!(
        formatted["whole_manifest_key"],
        baseline["whole_manifest_key"]
    );
    put(repo, "dep/new-data.txt", "new dependency input\n");
    assert!(!inputs::unchanged(&formatted));
    let changed = inputs::discover(repo, "body", context, &options);
    assert_ne!(changed["action_key"], baseline["action_key"]);
    let missing = inputs::discover(repo, "body", json!({}), &options);
    assert_eq!(missing["eligible"], true);
    assert!(missing["action_key"].is_null());
    assert!(missing["source_projection_key"].is_string());
}

#[test]
fn rereading_a_changed_file_and_unsupported_native_inputs_fail_loudly() {
    let root = tempfile::tempdir().unwrap();
    put(root.path(), "body/src/lib.rs", "pub fn initial() {}\n");
    let mut state = Discovery::new(root.path(), root.path(), json!({})).unwrap();
    let source = root.path().join("body/src/lib.rs");
    state.read(&source).unwrap();
    fs::write(&source, "pub fn changed() {}\n").unwrap();
    assert!(state
        .read(&source)
        .unwrap_err()
        .message
        .contains("changed during discovery"));
    state.roots.insert("body".into(), root.path().join("body"));
    state
        .audit_source(
            "body",
            &source,
            b"#[link(name=\"absent\", kind=\"dylib\")] extern \"C\" {}\n",
        )
        .unwrap();
    assert!(state
        .audit_source(
            "body",
            &source,
            b"#[link(name=\"absent\", kind=\"static\")] extern \"C\" {}\n"
        )
        .unwrap_err()
        .message
        .contains("native library input unsupported"));
    assert!(state
        .audit_source(
            "body",
            &source,
            b"// include!(\"../../outside\")\npub fn safe() {}\n"
        )
        .is_ok());
    assert!(state
        .audit_source(
            "body",
            &source,
            b"pub const QUOTE:char='\"'; include!(\"../../outside\");"
        )
        .is_err());
}

#[test]
fn nested_comments_preserve_raw_strings_and_descriptor_lists_stay_atomic() {
    let source = "r##\"/* text */\"## /* nested /* inner */ outer */ '\"' // ü\nend";
    let stripped = inputs::without_comments(source).unwrap();
    assert!(stripped.starts_with("r##\"/* text */\"## "));
    assert!(stripped.ends_with("\nend"));
    assert_eq!(stripped.chars().count(), source.chars().count());
    assert!(inputs::without_comments("/* unfinished").is_err());
    assert!(inputs::without_comments("r###\"unfinished").is_err());
    assert_eq!(
        inputs::explain(
            &json!({"ordered":[1,2],"null":null}),
            &json!({"ordered":[2,1]})
        ),
        json!([
            {"field":"/null","before":null,"after":null,"absent_before":false,"absent_after":true},
            {"field":"/ordered","before":[1,2],"after":[2,1]}
        ])
    );
}

#[test]
fn streaming_api_frames_errors_and_keeps_subsequent_requests_usable() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let mut child = Command::new(env!("CARGO_BIN_EXE_vibeos-build"))
        .arg("--api-jsonl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    input.write_all(b"malformed json\n{\"op\":\"artifact_inputs_canonical\",\"kwargs\":{\"value\":{\"b\":2,\"a\":1}}}\n").unwrap();
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let lines: Vec<serde_json::Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].get("error").is_some());
    assert_eq!(lines[1]["ok"], "{\"a\":1,\"b\":2}");
}
