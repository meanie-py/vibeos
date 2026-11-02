use super::{parse_layout_dump, Record};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

/// Replay only the include/define/target/diagnostic flags used by layout probes.
pub fn extract_flags(root: &Path, build: &Path, suffix: &str) -> Result<Vec<String>, String> {
    let path = crate::probe::local(root, &build.join("compile_commands.json"))?;
    let db: serde_json::Value =
        serde_json::from_str(&crate::read(&path)?).map_err(|e| e.to_string())?;
    let entries = db
        .as_array()
        .ok_or("compile_commands.json is not an array")?;
    let entry = entries
        .iter()
        .find(|e| e["file"].as_str().is_some_and(|s| s.ends_with(suffix)))
        .ok_or_else(|| format!("NO_TU:{suffix}"))?;
    let tokens = if let Some(args) = entry.get("arguments") {
        args.as_array()
            .ok_or("compile arguments must be an array")?
            .iter()
            .map(|v| {
                v.as_str()
                    .map(str::to_owned)
                    .ok_or("non-string compile argument")
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        shlex::split(
            entry["command"]
                .as_str()
                .ok_or("compile entry has no command or arguments")?,
        )
        .ok_or("invalid compile command quoting")?
    };
    let mut out = Vec::new();
    let mut i = 1;
    while i < tokens.len() {
        let t = &tokens[i];
        if [
            "-I",
            "-isystem",
            "-include",
            "-D",
            "-U",
            "-iquote",
            "-idirafter",
            "-target",
            "--target",
        ]
        .contains(&t.as_str())
        {
            out.push(t.clone());
            out.push(
                tokens
                    .get(i + 1)
                    .ok_or_else(|| format!("missing value after {t}"))?
                    .replace("/tmp/vibeos", &root.to_string_lossy()),
            );
            i += 2;
            continue;
        }
        if ["-I", "-D", "-isystem", "--target=", "-f", "-W", "-m"]
            .iter()
            .any(|p| t.starts_with(p))
            && ![
                "-fdiagnostics",
                "-fcolor",
                "-flto",
                "-ffile-prefix",
                "-fdebug",
            ]
            .iter()
            .any(|p| t.starts_with(p))
        {
            out.push(t.replace("/tmp/vibeos", &root.to_string_lossy()));
        }
        i += 1;
    }
    validate_generated_paths(root, &out)?;
    Ok(out)
}
fn validate_generated_paths(root: &Path, flags: &[String]) -> Result<(), String> {
    let mut i = 0;
    while i < flags.len() {
        let flag = &flags[i];
        let path =
            if ["-I", "-isystem", "-include", "-iquote", "-idirafter"].contains(&flag.as_str()) {
                i += 1;
                flags.get(i).map(String::as_str)
            } else {
                ["-isystem", "-I", "--sysroot="]
                    .iter()
                    .find_map(|p| flag.strip_prefix(p))
            };
        if let Some(path) = path {
            let path = PathBuf::from(path);
            if path.components().any(|c| {
                c.as_os_str()
                    .to_str()
                    .is_some_and(|s| s.starts_with("build-"))
            }) {
                let path = if path.is_absolute() {
                    path
                } else {
                    root.join(path)
                };
                crate::probe::local(root, &path)?;
            }
        }
        i += 1;
    }
    Ok(())
}
pub fn dump_layouts(
    root: &Path,
    flags: &[String],
    tu: &str,
) -> Result<HashMap<String, Record>, String> {
    validate_generated_paths(root, flags)?;
    let temp = crate::resources::TempDir::new()?;
    let input = temp.path().join("layout.c");
    fs::write(&input, tu).map_err(|e| e.to_string())?;
    let clang = root.join(".build-tools/llvm-mingw/bin/clang");
    let result = Command::new(&clang)
        .current_dir(root)
        .args(flags)
        .args(["-fsyntax-only", "-Xclang", "-fdump-record-layouts-complete"])
        .arg(&input)
        .output()
        .map_err(|e| format!("cannot run {}: {e}", clang.display()))?;
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        let errors = stderr
            .lines()
            .filter(|s| s.contains("error:"))
            .collect::<Vec<_>>()
            .join("\n");
        return Err(format!(
            "layout TU failed to compile:\n{}",
            if errors.is_empty() {
                stderr.chars().take(2000).collect::<String>()
            } else {
                errors.chars().take(2000).collect()
            }
        ));
    }
    let records = parse_layout_dump(&String::from_utf8_lossy(&result.stdout));
    if records.is_empty() {
        return Err("layout dump came back empty; refusing generation without layout data".into());
    }
    Ok(records)
}
pub fn dump_expected(
    root: &Path,
    flags: &[String],
    tu: &str,
    width: usize,
) -> Result<HashMap<String, Record>, String> {
    let source=format!("{tu}\n_Static_assert(sizeof(void*) == {width}, \"marshalgen: wrong pointer width for selected build\");\n");
    dump_layouts(root, flags, &source)
}
