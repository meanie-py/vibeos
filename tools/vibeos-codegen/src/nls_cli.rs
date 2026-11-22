//! `tools/codegen sortdefault [--check] [--root DIR]`: media/nls/sortdefault.nls from
//! Microsoft's Windows 10 sorting-weight table (see vibeos_codegen::sortdefault).
//!
//! The source is fetched at generation time into a cache OUTSIDE the repo
//! ($VIBEOS_NLS_CACHE, default ~/.cache/vibeos-nls/), so it can never be staged, and is
//! refused unless its sha256 is the pinned one. Not part of `--all`, because it needs that
//! file. `--check` exits 0 when the committed output matches a fresh generation, 1 when it
//! is stale, and 3 when it cannot be verified (no cache, no network), never 0 for that.
use std::path::{Path, PathBuf};
use std::process::Command;
use vibeos_codegen::sortdefault::{sha256_hex, sortdefault, SOURCE_SHA256, SOURCE_URL};

pub const NAMES: &[&str] = &["sortdefault"];
const OUTPUT: &str = "media/nls/sortdefault.nls";
const CACHED_NAME: &str = "win10-sorting-weight-table.txt";

fn cache_dir() -> Result<PathBuf, String> {
    if let Some(d) = std::env::var_os("VIBEOS_NLS_CACHE") {
        return Ok(PathBuf::from(d));
    }
    let home = std::env::var_os("HOME").ok_or("HOME is not set; set VIBEOS_NLS_CACHE")?;
    Ok(PathBuf::from(home).join(".cache/vibeos-nls"))
}

/// The verified source text, fetching it if it is not cached. Err(msg) when it cannot
/// be had; a checksum mismatch is always an error, cached or fresh.
fn source() -> Result<String, String> {
    let dir = cache_dir()?;
    let path = dir.join(CACHED_NAME);
    if !path.is_file() {
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let tmp = dir.join(format!("{CACHED_NAME}.partial"));
        let status = Command::new("curl")
            .args(["-sSfL", "--max-time", "600", "-o"])
            .arg(&tmp)
            .arg(SOURCE_URL)
            .status()
            .map_err(|e| format!("cannot run curl: {e}"))?;
        if !status.success() {
            let _ = std::fs::remove_file(&tmp);
            return Err(format!("fetching {SOURCE_URL} failed ({status})"));
        }
        std::fs::rename(&tmp, &path).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let got = sha256_hex(&bytes);
    if got != SOURCE_SHA256 {
        // Never keep or use a file that is not the pinned source.
        let _ = std::fs::remove_file(&path);
        return Err(format!(
            "{} has sha256 {got}, pinned {SOURCE_SHA256}: Microsoft changed the file or the \
             download was corrupted. Removed from the cache; review the new file before \
             re-pinning (tools/vibeos-codegen/src/sortdefault.rs).",
            path.display()
        ));
    }
    String::from_utf8(bytes).map_err(|e| format!("{}: not UTF-8: {e}", path.display()))
}

pub fn run(args: Vec<String>) -> Result<u8, String> {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut check = false;
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "sortdefault" => {}
            "--check" => check = true,
            "--root" => root = it.next().ok_or("--root requires DIR")?.into(),
            _ => return Err(format!("unexpected sortdefault argument: {a}")),
        }
    }
    let out = root.join(OUTPUT);
    let text = match source() {
        Ok(t) => t,
        Err(e) if check => {
            eprintln!("UNVERIFIED: {OUTPUT} could not be checked: {e}");
            return Ok(3);
        }
        Err(e) => return Err(e),
    };
    let bytes = sortdefault(&text)?;
    if check {
        return Ok(match std::fs::read(&out) {
            Ok(old) if old == bytes => {
                eprintln!("OK: {OUTPUT} matches a fresh generation");
                0
            }
            Ok(_) => {
                eprintln!("STALE: {OUTPUT} differs from a fresh generation; run tools/codegen sortdefault");
                1
            }
            Err(e) => {
                eprintln!("MISSING: {}: {e}", out.display());
                1
            }
        });
    }
    write_if_changed(&out, &bytes)?;
    eprintln!("OK: {OUTPUT} ({} bytes)", bytes.len());
    Ok(0)
}

fn write_if_changed(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if std::fs::read(path).ok().as_deref() == Some(bytes) {
        return Ok(());
    }
    let tmp = path.with_extension("nls.tmp");
    std::fs::write(&tmp, bytes).map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("{}: {e}", path.display()))
}
