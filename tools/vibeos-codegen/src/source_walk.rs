//! Prune mutable/generated trees before traversal; unreadable source is an error.
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn excluded(name: &std::ffi::OsStr) -> bool {
    let name = name.to_string_lossy();
    matches!(
        name.as_ref(),
        ".git"
            | ".claude"
            | ".cache"
            | ".venv"
            | "node_modules"
            | "__pycache__"
            | ".responses"
            | "target"
            | "build"
            | ".build-tools"
    ) || name.starts_with("build-")
        || name.starts_with("build_")
        || name.starts_with("cmake-build-")
}
/// Files at each directory precede its children, retaining legacy CMake discovery
/// precedence. Callers that require lexical order sort the returned paths.
pub fn files(root: &Path) -> Result<Vec<PathBuf>, String> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
        let mut dirs = Vec::new();
        for entry in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
            let entry = entry.map_err(|e| format!("{}: {e}", dir.display()))?;
            let path = entry.path();
            let ty = entry
                .file_type()
                .map_err(|e| format!("{}: {e}", path.display()))?;
            if ty.is_dir() {
                if !excluded(&entry.file_name()) {
                    dirs.push(path)
                }
            } else if ty.is_file() {
                out.push(path)
            } else if ty.is_symlink() {
                match fs::metadata(&path) {
                    Ok(m) if m.is_file() => out.push(path),
                    Ok(_) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => return Err(format!("{}: {e}", path.display())),
                }
            }
        }
        for dir in dirs {
            walk(&dir, out)?
        }
        Ok(())
    }
    let mut out = Vec::new();
    walk(root, &mut out)?;
    Ok(out)
}
