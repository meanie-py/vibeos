//! `tools/codegen sortdefault [--check] [--root DIR]`: media/nls/sortdefault.nls from
//! Microsoft's Windows 10 sorting-weight table (see vibeos_codegen::sortdefault).
//!
//! The source is fetched at generation time into a cache OUTSIDE the repo
//! ($VIBEOS_NLS_CACHE, default ~/.cache/vibeos-nls/), so it can never be staged, and is
//! refused unless its sha256 is the pinned one. Not part of `--all`, because a full check needs
//! that file. The committed output is also pinned by sha256 (OUTPUT_SHA256), so `--check`
//! always gives a real verdict: with the source it regenerates and requires the result to
//! equal both the committed file and the pin; without it (no cache, no network) it checks
//! the committed file against the pin and says that the source was not re-verified. Exit
//! 0 verified, 1 mismatch.
use std::path::{Path, PathBuf};
use std::process::Command;
use vibeos_codegen::sortdefault::{sha256_hex, sortdefault, OUTPUT_SHA256, SOURCE_SHA256, SOURCE_URL};

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
    let committed = std::fs::read(&out).ok();
    let pinned = committed.as_deref().map(sha256_hex).is_some_and(|h| h == OUTPUT_SHA256);
    let text = match source() {
        Ok(t) => t,
        Err(e) if check => {
            // No source: verify what is committed against the reviewed pin.
            return Ok(if pinned {
                eprintln!("OK (offline): {OUTPUT} matches OUTPUT_SHA256; the source was not re-verified ({e})");
                0
            } else {
                eprintln!("MISMATCH: {OUTPUT} does not match OUTPUT_SHA256 {OUTPUT_SHA256}, and the source is unavailable ({e})");
                1
            });
        }
        Err(e) => return Err(e),
    };
    let bytes = sortdefault(&text)?;
    let fresh = sha256_hex(&bytes);
    if check {
        let same = committed.as_deref() == Some(&bytes[..]);
        let pin_ok = fresh == OUTPUT_SHA256;
        if same && pin_ok {
            eprintln!("OK: {OUTPUT} matches a fresh generation and OUTPUT_SHA256");
            return Ok(0);
        }
        if !same {
            eprintln!(
                "STALE: {OUTPUT} is {} a fresh generation (sha256 {fresh}); run tools/codegen sortdefault",
                if committed.is_some() { "not" } else { "missing, not" }
            );
        }
        if !pin_ok {
            eprintln!("STALE PIN: a fresh generation is sha256 {fresh}, but OUTPUT_SHA256 says {OUTPUT_SHA256}");
        }
        return Ok(1);
    }
    if fresh != OUTPUT_SHA256 {
        return Err(format!(
            "the generated file's sha256 is {fresh}, not the pinned OUTPUT_SHA256 {OUTPUT_SHA256}. \
             If the change is intended, update OUTPUT_SHA256 in tools/vibeos-codegen/src/sortdefault.rs \
             in the same commit as the regenerated file."
        ));
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
