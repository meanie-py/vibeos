use super::{BinaryContext, CheckOutput};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
const FLAG: &str = "target-feature=-sse,-sse2";
const TARGET: &str = "i686-pc-windows-gnu";
const LATENT: &str = "OK_LATENT";
const CHANGED: &str = "ABI_CHANGED";
const TRIPPED: &str = "TRIPPED_HARD_ERROR";
const UNAVAILABLE: &str = "UNAVAILABLE";
fn doomed(root: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for e in entries.flatten() {
            let p = e.path();
            if e.file_type().is_ok_and(|t| t.is_dir()) {
                let n = e.file_name();
                if n != "target" && n != ".git" && !n.to_string_lossy().starts_with("build-") {
                    walk(&p, out)
                }
            } else if e.file_name() == "CMakeLists.txt" {
                if let Ok(text) = fs::read_to_string(&p) {
                    if text.contains(FLAG)
                        && text.contains(TARGET)
                        && !text.contains("i686-kernel.json")
                    {
                        out.push(p)
                    }
                }
            }
        }
    }
    let mut out = Vec::new();
    walk(root, &mut out);
    let extra = root.join("sdk/cmake/gcc.cmake");
    if fs::read_to_string(&extra).is_ok_and(|s| s.contains(FLAG) && s.contains(TARGET)) {
        out.push(extra)
    }
    out.sort();
    out
}
fn classify(success: bool, stderr: &str) -> &'static str {
    if !success {
        if (stderr.contains("sse2") && stderr.contains("ABI")) || stderr.contains("x86-softfloat") {
            TRIPPED
        } else {
            UNAVAILABLE
        }
    } else if stderr.contains("being phased out") {
        LATENT
    } else {
        CHANGED
    }
}
fn probe(root: &Path) -> Result<(&'static str, String), String> {
    let installed = match Command::new("rustc")
        .args(["--print", "target-list"])
        .current_dir(root)
        .output()
    {
        Ok(p) => p,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok((UNAVAILABLE, "rustc not found".into()))
        }
        Err(e) => return Err(e.to_string()),
    };
    if !installed.status.success()
        || !String::from_utf8_lossy(&installed.stdout)
            .split_whitespace()
            .any(|s| s == TARGET)
    {
        return Ok((
            UNAVAILABLE,
            format!("target {TARGET} not available to rustc"),
        ));
    }
    static SEQ: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "vibeos-abi-{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&dir).map_err(|e| e.to_string())?;
    let result = (|| {
        let src = dir.join("t.rs");
        fs::write(&src, "#![crate_type = \"lib\"]\npub fn f() {}\n").map_err(|e| e.to_string())?;
        let out = Command::new("rustc")
            .args(["--edition", "2021", "--target", TARGET, "-C", FLAG])
            .arg(&src)
            .arg("-o")
            .arg(dir.join("t.rlib"))
            .current_dir(root)
            .output()
            .map_err(|e| e.to_string())?;
        let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
        Ok((classify(out.status.success(), &stderr), stderr))
    })();
    let _ = fs::remove_dir_all(dir);
    result
}
pub fn run(ctx: &BinaryContext) -> Result<CheckOutput, String> {
    let doomed = doomed(&ctx.root);
    let (verdict, _) = probe(&ctx.root)?;
    let mut stdout = format!(
        "i386 no-SSE ABI tripwire — {} doomed CMakeLists, rustc verdict: {verdict}\n",
        doomed.len()
    );
    match verdict{TRIPPED=>{stdout+=&format!("\nTRIPPED: the pinned rustc now makes `-C target-feature=-sse,-sse2` on the\nbuiltin `{TARGET}` target a HARD ERROR (rust-lang#116344). All\n{} i386 crates below will fail to build.\n\nFix: migrate them to a softfloat custom target json (rustc-abi: x86-softfloat)\nbuilt with `-Z build-std` (nightly), the model the i386 `*-cargo` cells already\nuse (sdk/lib/rust/ntoskrnl-cargo/i686-kernel.json + gcc.cmake add_cargo_module).\nRoadmap: \"54 i386 crates build on an SSE/ABI combination rustc is phasing out\".\n\n",doomed.len());for p in doomed{stdout+=&format!("  {}\n",p.strip_prefix(&ctx.root).map_err(|e|e.to_string())?.display())}},CHANGED=>stdout+="\nNOTE: rustc compiled the doomed flag+triple with NO phase-out warning. The\nbuiltin target's ABI may have changed upstream; re-evaluate whether the\n`-sse,-sse2` flag is still needed, or whether a softfloat ABI is now builtin.\n\n",UNAVAILABLE=>stdout+="\nNOTE: rustc or the i686 target is unavailable here; tripwire not evaluated.\n",_=>stdout+="\nOK: still a warning (latent). No action until the pin is bumped.\n"}
    Ok(CheckOutput::new(stdout, i32::from(verdict == TRIPPED)))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn abi_errors_are_distinguished_from_missing_std() {
        assert_eq!(classify(false, "sse2 ABI required"), TRIPPED);
        assert_eq!(classify(false, "can't find crate for std"), UNAVAILABLE);
        assert_eq!(classify(true, "being phased out"), LATENT);
        assert_eq!(classify(true, ""), CHANGED);
    }
    #[test]
    fn pinned_rustc_abi_tripwire() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let (verdict, stderr) = probe(root).unwrap();
        assert_ne!(
            verdict, TRIPPED,
            "i386 no-SSE ABI now a hard error; migrate builtin targets to x86-softfloat: {stderr}"
        );
        if verdict == UNAVAILABLE {
            eprintln!("tripwire unavailable: {stderr}")
        }
    }
    #[test]
    fn custom_json_is_excluded() {
        let root = std::env::temp_dir().join(format!("vibeos-doomed-{}", std::process::id()));
        fs::create_dir_all(root.join("safe")).unwrap();
        fs::create_dir_all(root.join("doomed")).unwrap();
        fs::write(
            root.join("safe/CMakeLists.txt"),
            format!("{FLAG} {TARGET} i686-kernel.json"),
        )
        .unwrap();
        fs::write(
            root.join("doomed/CMakeLists.txt"),
            format!("{FLAG} {TARGET}"),
        )
        .unwrap();
        assert_eq!(doomed(&root), vec![root.join("doomed/CMakeLists.txt")]);
        let _ = fs::remove_dir_all(root);
    }
}
