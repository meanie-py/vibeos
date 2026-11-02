//! Identity for cached Clang UCRT declarations. Only used when caching is requested.
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

// These variables can change Clang's include/config search outside the installation
// being fingerprinted. Refuse caching rather than silently reuse untracked inputs.
const UNSUPPORTED_ENV: &[&str] = &[
    "CPATH",
    "C_INCLUDE_PATH",
    "CPLUS_INCLUDE_PATH",
    "OBJC_INCLUDE_PATH",
    "INCLUDE",
    "CCC_OVERRIDE_OPTIONS",
    "CLANG_CONFIG_FILE_USER_DIR",
    "CLANG_CONFIG_FILE_SYSTEM_DIR",
    "SDKROOT",
    "GCC_EXEC_PREFIX",
    "COMPILER_PATH",
    "CLANG_RESOURCE_DIR",
    "RC_DEBUG_OPTIONS",
    "CCACHE",
];
fn update_file(hash: &mut Sha256, path: &Path) -> Result<(), String> {
    use std::io::Read;
    let mut file = fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let expected = file.metadata().map_err(|e| e.to_string())?.len();
    hash.update(expected.to_le_bytes());
    let mut buffer = [0u8; 64 * 1024];
    let mut total = 0u64;
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        total += n as u64;
        hash.update(&buffer[..n]);
    }
    if total != expected {
        return Err(format!("{} changed while fingerprinting", path.display()));
    }
    Ok(())
}

fn headers_under(
    path: &Path,
    visited_dirs: &mut HashSet<PathBuf>,
    output: &mut Vec<(PathBuf, PathBuf, bool)>,
) -> Result<(), String> {
    let canonical = path
        .canonicalize()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let meta = fs::metadata(&canonical).map_err(|e| format!("{}: {e}", canonical.display()))?;
    if meta.is_dir() {
        // Record every directory alias before deduplication, including cycle links.
        output.push((path.to_owned(), canonical.clone(), false));
        if !visited_dirs.insert(canonical.clone()) {
            return Ok(());
        }
        let mut entries: Vec<_> = fs::read_dir(path)
            .map_err(|e| format!("{}: {e}", path.display()))?
            .map(|r| r.map(|e| e.path()).map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
        entries.sort();
        for entry in entries {
            headers_under(&entry, visited_dirs, output)?;
        }
    } else if meta.is_file()
        && [path, canonical.as_path()].iter().any(|p| {
            matches!(
                p.extension().and_then(|e| e.to_str()),
                Some("h" | "hpp" | "inc" | "def" | "cfg")
            )
        })
    {
        output.push((path.to_owned(), canonical, true));
    }
    Ok(())
}

/// Compute a content identity for the compiler and installation headers/config.
/// External search overrides are rejected because their files are not fingerprinted.
pub fn identity(
    root: &Path,
    arch: &str,
    compiler: &Path,
    headers: &[&str],
) -> Result<String, String> {
    for name in UNSUPPORTED_ENV {
        if std::env::var_os(name).is_some_and(|value| !value.is_empty()) {
            return Err(format!(
                "cannot safely cache UCRT header AST while {name} is set; unset it or omit --cache"
            ));
        }
    }
    let install = root.join(".build-tools/llvm-mingw");
    let mut files = Vec::new();
    headers_under(&install, &mut HashSet::new(), &mut files)?;
    files.sort();
    let compiler = compiler
        .canonicalize()
        .map_err(|e| format!("{}: {e}", compiler.display()))?;
    let mut hash = Sha256::new();
    hash.update(b"ucrt-tier2-ast-v2\0");
    for item in [arch.as_bytes(), compiler.to_string_lossy().as_bytes()] {
        hash.update((item.len() as u64).to_le_bytes());
        hash.update(item);
    }
    for header in headers {
        hash.update((header.len() as u64).to_le_bytes());
        hash.update(header.as_bytes());
    }
    update_file(&mut hash, &compiler)?;
    // llvm-mingw's target executable is a shell wrapper around this compiler.
    // Its own bytes alone cannot identify the actual Clang implementation.
    if compiler
        .file_name()
        .is_some_and(|name| name == "clang-target-wrapper.sh")
    {
        let clang = compiler
            .parent()
            .unwrap()
            .join("clang")
            .canonicalize()
            .map_err(|e| format!("cannot identify wrapped clang: {e}"))?;
        if clang != compiler {
            let name = clang.to_string_lossy();
            hash.update((name.len() as u64).to_le_bytes());
            hash.update(name.as_bytes());
            update_file(&mut hash, &clang)?;
        }
    }
    let mut hashed_files = HashSet::new();
    hashed_files.insert(compiler);
    for (path, canonical, is_file) in files {
        for name in [&path, &canonical] {
            let name = name.to_string_lossy();
            hash.update((name.len() as u64).to_le_bytes());
            hash.update(name.as_bytes());
        }
        if is_file && hashed_files.insert(canonical.clone()) {
            update_file(&mut hash, &canonical)?;
        }
    }
    Ok(format!("{:x}", hash.finalize()))
}

#[cfg(all(test, unix))]
mod tests {
    use super::identity;
    use std::{
        fs,
        os::unix::fs::symlink,
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };
    static SEQUENCE: AtomicUsize = AtomicUsize::new(0);
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let p = std::env::temp_dir().join(format!(
                "ucrt-identity-{}-{}",
                std::process::id(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(p.join(".build-tools/llvm-mingw/include")).unwrap();
            fs::create_dir_all(p.join("outside")).unwrap();
            fs::write(p.join("compiler"), "compiler one").unwrap();
            fs::write(p.join("outside/header.h"), "header one").unwrap();
            symlink(
                p.join("outside"),
                p.join(".build-tools/llvm-mingw/include/external"),
            )
            .unwrap();
            symlink(p.join(".build-tools/llvm-mingw"), p.join("outside/cycle")).unwrap();
            Self(p)
        }
        fn key(&self) -> String {
            identity(&self.0, "amd64", &self.0.join("compiler"), &["x.h"]).unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    #[test]
    fn fingerprint_tracks_linked_headers_and_compiler_without_hashing_other_binaries() {
        let f = Fixture::new();
        let first = f.key(); // symlink cycle must terminate
        fs::write(f.0.join(".build-tools/llvm-mingw/unused-binary"), "one").unwrap();
        assert_eq!(first, f.key());
        fs::write(f.0.join("outside/header.h"), "header two").unwrap();
        let second = f.key();
        assert_ne!(first, second);
        fs::write(f.0.join("compiler"), "compiler two").unwrap();
        assert_ne!(second, f.key());
    }
}
