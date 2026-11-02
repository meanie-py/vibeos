//! Port of tools/gen_cflags.py's compile-command recipe extraction.
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

const CANON_ROOT: &str = "/tmp/vibeos";

/// Return the generated flags as one token per line, or None when this arch has no
/// configured compile database or the database has no source entries for the target.
pub fn generate(root: &Path, arch: &str, target: &str) -> Result<Option<String>, String> {
    validate_component("arch", arch)?;
    validate_component("target", target)?;
    let path = root.join(format!("build-{arch}/compile_commands.json"));
    let canonical_root = root
        .canonicalize()
        .map_err(|e| format!("gen_cflags: cannot resolve repo {}: {e}", root.display()))?;
    if !path.exists() {
        return Ok(None);
    }
    let db_real = path
        .canonicalize()
        .map_err(|e| format!("gen_cflags: cannot resolve {}: {e}", path.display()))?;
    if !db_real.starts_with(&canonical_root) {
        return Err(format!(
            "gen_cflags: compile database resolves outside checkout: {}",
            path.display()
        ));
    }
    let text = std::fs::read_to_string(&db_real)
        .map_err(|e| format!("gen_cflags: cannot read {}: {e}", path.display()))?;
    generate_target_from_json(&text, arch, target, &canonical_root)
}

/// Variant with a caller supplied target; the target string is used in CMake's marker.
pub fn generate_target_from_json(
    db_json: &str,
    arch: &str,
    target: &str,
    repo: &Path,
) -> Result<Option<String>, String> {
    validate_component("arch", arch)?;
    validate_component("target", target)?;
    let db: Value = serde_json::from_str(db_json)
        .map_err(|e| format!("gen_cflags: malformed compile_commands.json: {e}"))?;
    let entries = db
        .as_array()
        .ok_or("gen_cflags: compile_commands.json must be an array")?;
    let repo = repo.canonicalize().unwrap_or_else(|_| repo.to_path_buf());
    generate_entries(entries, arch, &repo, &format!("CMakeFiles/{target}.dir"))
}

fn validate_component(label: &str, value: &str) -> Result<(), String> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || value.contains('/')
        || value.contains('\\')
    {
        return Err(format!("gen_cflags: invalid {label}: {value:?}"));
    }
    Ok(())
}
fn generate_entries(
    entries: &[Value],
    arch: &str,
    repo: &Path,
    marker: &str,
) -> Result<Option<String>, String> {
    let mut templates = BTreeMap::<String, Vec<String>>::new();
    for (i, e) in entries.iter().enumerate() {
        let obj = e
            .as_object()
            .ok_or_else(|| format!("gen_cflags: compile command {i} is not an object"))?;
        let file = obj
            .get("file")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("gen_cflags: compile command {i} has no string file"))?;
        let tokens = if let Some(cmd) = obj.get("command") {
            let command = cmd.as_str().ok_or_else(|| {
                format!("gen_cflags: compile command {i} command is not a string")
            })?;
            if !command.contains(marker) || !file.ends_with(".c") || file.contains("cmake_pch") {
                continue;
            }
            shlex::split(command).ok_or_else(|| {
                format!("gen_cflags: cannot parse command quoting in compile command {i}")
            })?
        } else if let Some(argv) = obj.get("arguments") {
            if !file.ends_with(".c") || file.contains("cmake_pch") {
                continue;
            }
            let args = argv.as_array().ok_or_else(|| {
                format!("gen_cflags: compile command {i} arguments is not an array")
            })?;
            let mut toks = Vec::with_capacity(args.len());
            for arg in args {
                toks.push(
                    arg.as_str()
                        .ok_or_else(|| {
                            format!("gen_cflags: compile command {i} has a non-string argument")
                        })?
                        .to_owned(),
                );
            }
            // Match the Python selector, whose marker is in `command`; argv databases have no
            // shell string, so require the corresponding target object path in a token.
            if !toks.iter().any(|t| t.contains(marker)) {
                continue;
            }
            toks
        } else {
            return Err(format!(
                "gen_cflags: compile command {i} has neither command nor arguments"
            ));
        };
        let flags = candidate_flags(&tokens, arch, repo)?;
        templates.insert(file.to_owned(), flags);
    }
    if templates.is_empty() {
        return Ok(None);
    }
    let unique = templates.values().cloned().collect::<BTreeSet<_>>();
    if unique.len() != 1 {
        return Err(format!(
            "[{} {}] {} distinct flag sets across sources — the single-flag-set invariant broke",
            marker
                .strip_prefix("CMakeFiles/")
                .unwrap_or(marker)
                .strip_suffix(".dir")
                .unwrap_or(marker),
            arch,
            unique.len()
        ));
    }
    let flags = unique.into_iter().next().unwrap();
    Ok(Some(format!("{}\n", flags.join("\n"))))
}

/// Translate a compiler argv to the inert, portable template. Token order is preserved.
pub fn candidate_flags(tokens: &[String], arch: &str, repo: &Path) -> Result<Vec<String>, String> {
    if tokens.is_empty() {
        return Err("gen_cflags: empty compiler command".into());
    }
    let repo = repo.to_string_lossy();
    let mut out = Vec::new();
    let mut i = 1;
    while i < tokens.len() {
        let t = &tokens[i];
        if matches!(
            t.as_str(),
            "-Winvalid-pch" | "-Xclang" | "-include" | "-include-pch"
        ) {
            i += 1;
            continue;
        }
        if t.contains("cmake_pch") {
            i += 1;
            continue;
        }
        if t.starts_with("-I") && t.contains(&format!("/build-{arch}/")) {
            i += 1;
            continue;
        }
        if t == "-o" {
            if i + 1 >= tokens.len() {
                return Err("gen_cflags: -o without output path".into());
            }
            i += 2;
            continue;
        }
        if t.ends_with(".c") && Path::new(t).is_absolute() {
            i += 1;
            continue;
        }
        out.push(
            t.replace(repo.as_ref(), "{REPO}")
                .replace(CANON_ROOT, "{REPO}"),
        );
        i += 1;
    }
    Ok(out)
}

/// Match `json.dumps(gen_content(...))` for callers that retain gen_cflags.py's --stdout ABI.
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};
    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let p = std::env::temp_dir().join(format!(
                "cflags-test-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir_all(&p).unwrap();
            Self(p)
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    fn entry(command: &str, file: &str) -> Value {
        serde_json::json!({"command":command,"file":file})
    }
    #[test]
    fn strips_build_pch_source_output_and_rebases_roots() {
        let root = Path::new("/work/checkout");
        let toks=shlex::split("clang -DNAME=\"two words\" -I/work/checkout/sdk/include -I/tmp/vibeos/build-i386/gen -Xclang -include-pch -Xclang /tmp/vibeos/build-i386/cmake_pch.pch -Xclang -include -Xclang /work/checkout/build-i386/cmake_pch.h /work/checkout/dll/genincdata.c -o out.obj -O2").unwrap().into_iter().collect::<Vec<_>>();
        let got = candidate_flags(&toks, "i386", root).unwrap();
        assert_eq!(got, ["-DNAME=two words", "-I{REPO}/sdk/include", "-O2"]);
    }
    #[test]
    fn parity_selects_command_flags_and_ignores_pch_tu() {
        let db=serde_json::json!([entry("clang -DCMAKEFILES= -I/tmp/vibeos/sdk -I/tmp/vibeos/build-arm64/generated -O2 /tmp/vibeos/tools/genincdata.c -o CMakeFiles/genincdata.dir/foo.o","/tmp/vibeos/tools/genincdata.c"),entry("clang CMakeFiles/genincdata.dir/cmake_pch.h.c -O0","/tmp/vibeos/cmake_pch.h.c")]);
        let got = generate_target_from_json(
            &db.to_string(),
            "arm64",
            "genincdata",
            Path::new("/tmp/vibeos"),
        )
        .unwrap()
        .unwrap();
        assert_eq!(got, "-DCMAKEFILES=\n-I{REPO}/sdk\n-O2\n");
    }
    #[test]
    fn arguments_vector_preserves_quoted_single_arg() {
        let db = serde_json::json!([{"arguments":["clang","-DMSG=hello world","-o","CMakeFiles/genincdata.dir/foo.o","-O2","/tmp/vibeos/foo.c"],"file":"/tmp/vibeos/foo.c"}]);
        assert_eq!(
            generate_target_from_json(
                &db.to_string(),
                "i386",
                "genincdata",
                Path::new("/tmp/vibeos")
            )
            .unwrap()
            .unwrap(),
            "-DMSG=hello world\n-O2\n"
        );
    }
    #[test]
    fn invariant_malformed_inputs_and_missing_build() {
        let db = serde_json::json!([
            entry(
                "clang CMakeFiles/genincdata.dir/a -O1 /root/a.c",
                "/root/a.c"
            ),
            entry(
                "clang CMakeFiles/genincdata.dir/b -O2 /root/b.c",
                "/root/b.c"
            )
        ]);
        assert!(generate_target_from_json(
            &db.to_string(),
            "amd64",
            "genincdata",
            Path::new("/tmp/vibeos")
        )
        .unwrap_err()
        .contains("single-flag-set"));
        assert!(
            generate_target_from_json("{", "i386", "genincdata", Path::new("/tmp/vibeos"))
                .unwrap_err()
                .contains("malformed")
        );
        let t = Temp::new();
        assert_eq!(generate(&t.0, "i386", "genincdata").unwrap(), None);
        assert_eq!(
            serde_json::to_string(&generate(&t.0, "i386", "genincdata").unwrap()).unwrap(),
            "null"
        );
    }
    #[test]
    fn malformed_quoting_and_trailing_output_are_errors() {
        let db = serde_json::json!([entry(
            "clang CMakeFiles/genincdata.dir/foo 'unterminated",
            "/a.c"
        )]);
        assert!(generate_target_from_json(
            &db.to_string(),
            "i386",
            "genincdata",
            Path::new("/tmp/vibeos")
        )
        .unwrap_err()
        .contains("quoting"));
        let t = vec!["clang".into(), "-o".into()];
        assert!(candidate_flags(&t, "i386", Path::new("/root"))
            .unwrap_err()
            .contains("without output"));
    }
}
