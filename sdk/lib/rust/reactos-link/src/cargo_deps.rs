//! Propagate a nested Cargo build's actual inputs into its caller's depfile.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub fn inputs(raw: &str, artifact: &Path, workspace: &Path) -> Result<BTreeSet<PathBuf>, String> {
    // A crate emitting both staticlib and rlib gets ONE Cargo depfile; its
    // target may name the rlib even though this caller consumes the staticlib.
    // They come from the same rustc invocation and have the same source closure.
    let outputs = [artifact.to_path_buf(), artifact.with_extension("rlib")];
    let body = outputs
        .iter()
        .find_map(|output| {
            let prefix = format!("{}:", output.to_string_lossy().replace(' ', "\\ "));
            raw.strip_prefix(&prefix)
        })
        .ok_or("unexpected Cargo depfile target")?;
    let mut paths = BTreeSet::new();
    let mut token = String::new();
    let mut chars = body.trim_end().chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some('\n') => (),
                Some('\r') if chars.next() == Some('\n') => (),
                Some(c @ (' ' | '\t' | '#' | '\\')) => token.push(c),
                _ => return Err("unsupported Cargo depfile escape".into()),
            },
            '$' if chars.next() == Some('$') => token.push('$'),
            '$' | '#' | '\n' | '\r' | '\0' => return Err("unsupported Cargo depfile syntax".into()),
            ' ' | '\t' => {
                if !token.is_empty() {
                    paths.insert(workspace.join(std::mem::take(&mut token)));
                }
            }
            _ => token.push(c),
        }
    }
    if !token.is_empty() {
        paths.insert(workspace.join(token));
    }
    if paths.is_empty() {
        return Err("empty Cargo dependency closure".into());
    }
    // Cargo's .d includes compiler/build-script inputs, but Cargo.toml is tracked
    // internally by Cargo itself. The outer build must also notice a dependency's
    // manifest edit before deciding whether to invoke the nested Cargo at all.
    let mut manifests = BTreeSet::new();
    let mut seen = BTreeSet::new();
    for input in &paths {
        if !input.exists() {
            return Err(format!("missing Cargo prerequisite: {}", input.display()));
        }
        for parent in input.ancestors().skip(1) {
            if !seen.insert(parent.to_path_buf()) {
                break;
            }
            let manifest = parent.join("Cargo.toml");
            if manifest.is_file() {
                manifests.insert(manifest);
            }
            if parent == workspace {
                break;
            }
        }
    }
    paths.extend(manifests);
    Ok(paths)
}

pub fn watch(depfile: &Path, artifact: &Path, workspace: &Path) {
    let raw = std::fs::read_to_string(depfile)
        .unwrap_or_else(|e| panic!("read nested Cargo depfile {}: {e}", depfile.display()));
    let inputs =
        inputs(&raw, artifact, workspace).unwrap_or_else(|e| panic!("{}: {e}", depfile.display()));
    for input in inputs {
        println!("cargo:rerun-if-changed={}", input.display());
    }
}
