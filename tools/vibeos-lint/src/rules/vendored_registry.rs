//! Default vendor registry audit; write/report APIs remain in tools/vendored.py.
use crate::{
    vendor_registry::{self, AuditSnapshot},
    Corpus, Rule,
};
use serde_json::Value;
use std::{collections::BTreeSet, fs, path::Path};

pub struct VendoredRegistry;

fn read(c: &Corpus, path: &str) -> Result<String, String> {
    if let Some(file) = c.file(Path::new(path)) {
        return Ok(file.text.clone());
    }
    fs::read(c.root.join(path))
        .map(|bytes| {
            String::from_utf8_lossy(&bytes)
                .replace("\r\n", "\n")
                .replace('\r', "\n")
        })
        .map_err(|e| format!("{path}: {e}"))
}
fn named_inputs(value: &Value, paths: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if matches!(
                    key.as_str(),
                    "path" | "license_file" | "license_ref" | "patch" | "file"
                ) {
                    if let Some(path) = value.as_str() {
                        paths.insert(path.into());
                    }
                }
                named_inputs(value, paths);
            }
        }
        Value::Array(values) => {
            for value in values {
                named_inputs(value, paths);
            }
        }
        _ => (),
    }
}
fn snapshot(c: &Corpus, registry: &Value) -> Result<AuditSnapshot, String> {
    let mut required: BTreeSet<String> = [
        "vendor.yaml",
        "tools/rust-crates.yaml",
        "sdk/lib/rust/Cargo.lock",
        "THIRD-PARTY-NOTICES.md",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    named_inputs(registry, &mut required);
    fn walk(
        c: &Corpus,
        rel: &Path,
        required: &BTreeSet<String>,
        out: &mut AuditSnapshot,
    ) -> Result<(), String> {
        let name = rel.to_string_lossy().replace('\\', "/");
        if !rel.as_os_str().is_empty() {
            for marker in [".git", ".rustc_info.json", "CACHEDIR.TAG"] {
                if c.root.join(rel).join(marker).exists() {
                    out.markers.insert(format!("{name}/{marker}"));
                    return Ok(());
                }
            }
        }
        out.directories.insert(name.clone());
        out.ordered_directories.push(name);
        let entries = fs::read_dir(c.root.join(rel))
            .map_err(|e| format!("vendor inventory {}: {e}", rel.display()))?;
        let mut dirs = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = rel.join(entry.file_name());
            let name = path.to_string_lossy().replace('\\', "/");
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            // read_dir already supplies the file type on supported filesystems.
            // Only symlinks need a separate stat to preserve os.walk's treatment
            // of directory links (recorded, but never descended).
            if kind.is_dir() || (kind.is_symlink() && entry.path().is_dir()) {
                out.directories.insert(name.clone());
                if !kind.is_symlink()
                    && ![
                        ".git",
                        ".claude",
                        "build-i386",
                        "build-amd64",
                        "build-arm64",
                    ]
                    .contains(&name.as_str())
                {
                    dirs.push(path);
                }
            } else {
                let source = path.extension().is_some_and(|ext| {
                    ["c", "cpp", "cxx", "cc", "h", "hpp", "hxx"]
                        .iter()
                        .any(|want| ext.to_string_lossy().eq_ignore_ascii_case(want))
                });
                let text = if source || required.contains(&name) || entry.file_name() == "VENDORED"
                {
                    read(c, &name)?
                } else {
                    String::new()
                };
                out.ordered_files.push(name.clone());
                out.files.insert(name, text);
            }
        }
        for dir in dirs {
            walk(c, &dir, required, out)?;
        }
        Ok(())
    }
    let mut out = AuditSnapshot {
        // Standalone --root registries do not inherit this project's tolerated
        // licence debts. A project checkout carries the canonical writer API.
        is_project_repo: c.root.join("tools/vendored.py").is_file(),
        ..Default::default()
    };
    walk(c, Path::new(""), &required, &mut out)?;
    // Registry-selected files can live below an otherwise pruned directory.
    for path in required {
        if c.root.join(&path).is_file() && !out.files.contains_key(&path) {
            out.files.insert(path.clone(), read(c, &path)?);
        }
    }
    Ok(out)
}
impl Rule for VendoredRegistry {
    fn needs_workspace_rust(&self) -> bool {
        false
    }
    fn name(&self) -> &'static str {
        "vendored_registry"
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let registry = crate::yaml::parse(&read(c, "vendor.yaml")?)?;
        let started = std::time::Instant::now();
        let snapshot = snapshot(c, &registry)?;
        if std::env::var_os("VIBEOS_LINT_TIMINGS").is_some() {
            eprintln!(
                "TIMING\tvendor_inventory\t{:.3}\t{} files",
                started.elapsed().as_secs_f64(),
                snapshot.files.len()
            );
        }
        let crates = snapshot
            .file("tools/rust-crates.yaml")
            .map(crate::yaml::parse)
            .transpose()?
            .unwrap_or(Value::Null);
        let problems = vendor_registry::audit(c, &registry, &crates, &snapshot)?;
        if problems.is_empty() {
            return Ok(vec![]);
        }
        let mut lines = vec![format!(
            "[vendored] {} disagrees with the tree ({} problem(s)):\n",
            c.root.join("vendor.yaml").display(),
            problems.len()
        )];
        lines.extend(problems.into_iter().map(|p| format!("  - {p}")));
        lines.push(
            "\n  vendor.yaml is the source of truth for which C is NOT ours to translate.".into(),
        );
        Ok(lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_registry_fails_loudly() {
        let c = Corpus::new(
            Path::new("/nonexistent-vibeos-vendor-fixture").into(),
            vec![],
        );
        assert!(VendoredRegistry
            .run(&c)
            .unwrap_err()
            .contains("vendor.yaml"));
    }
    #[test]
    fn selected_paths_include_version_probes_and_licences() {
        let mut paths = BTreeSet::new();
        named_inputs(
            &serde_json::json!({"trees":[{"license_file":"COPYING", "version":{"file":"version.h"}}]}),
            &mut paths,
        );
        assert_eq!(
            paths,
            ["COPYING".into(), "version.h".into()].into_iter().collect()
        );
    }
    #[test]
    fn committed_notices_regenerate_byte_for_byte() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let c = Corpus::new(root.into(), vec![]);
        let registry = crate::yaml::parse(&read(&c, "vendor.yaml").unwrap()).unwrap();
        let mut paths = BTreeSet::new();
        named_inputs(&registry, &mut paths);
        let mut snapshot = AuditSnapshot::default();
        for path in paths {
            if root.join(&path).is_file() {
                snapshot
                    .files
                    .insert(path.clone(), read(&c, &path).unwrap());
            } else if root.join(&path).is_dir() {
                snapshot.directories.insert(path);
            }
        }
        let crates = crate::yaml::parse(&read(&c, "tools/rust-crates.yaml").unwrap()).unwrap();
        let actual = vendor_registry::render_notices(&registry, &snapshot, &crates).unwrap();
        let expected = read(&c, "THIRD-PARTY-NOTICES.md").unwrap();
        let a: Vec<_> = actual.lines().collect();
        let b: Vec<_> = expected.lines().collect();
        let first = a
            .iter()
            .zip(&b)
            .position(|(a, b)| a != b)
            .unwrap_or(a.len().min(b.len()));
        assert!(
            actual == expected,
            "notices first differ at line {}: generated {:?}; committed {:?}",
            first + 1,
            a.get(first..(first + 3).min(a.len())),
            b.get(first..(first + 3).min(b.len()))
        );
    }
}
