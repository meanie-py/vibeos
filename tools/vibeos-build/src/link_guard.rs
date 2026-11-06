//! ld.lld duplicate-symbol policy and process wrapper.
use crate::{Error, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
    sync::OnceLock,
};
fn re(p: &'static str) -> &'static Regex {
    static R: OnceLock<std::sync::Mutex<std::collections::HashMap<&'static str, &'static Regex>>> =
        OnceLock::new();
    let mut m = R.get_or_init(Default::default).lock().unwrap();
    *m.entry(p)
        .or_insert_with(|| Box::leak(Box::new(Regex::new(p).unwrap())))
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dup {
    pub symbol: String,
    pub kept: String,
    pub dropped: String,
    pub kept_kind: String,
    pub dropped_kind: String,
    pub verdict: String,
}
pub fn origin_kind(origin: &str) -> &'static str {
    if re(r"compiler_builtins-|compiler-builtins|\.[/\\]lib/builtins[/\\]|libclang_rt\.builtins|libunwind\.a\(").is_match(origin){"runtime"}else if re(r"(?i)\.delay\.o\)?$|\([^()/]+\.(dll|exe|sys|drv|ocx|cpl|acm|efi|tsp)\)$|^[^()/\s]+\.(dll|exe|sys|drv|ocx|cpl|acm|efi|tsp)$").is_match(origin.rsplit(" | ").next().unwrap_or("").trim()){ "import" }else{"local"}
}
pub fn parse(stderr: &str) -> Vec<(String, String, String)> {
    let lines: Vec<_> = stderr.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let m = re(r"warning: duplicate symbol: (.+)$").captures(lines[i]);
        i += 1;
        let Some(m) = m else { continue };
        let symbol = m[1].trim().to_owned();
        let mut defs: Vec<Vec<String>> = Vec::new();
        while i < lines.len() && lines[i].starts_with(">>>") {
            let body = lines[i][3..].trim();
            if let Some(s) = body.strip_prefix("defined at ") {
                defs.push(vec![s.to_owned()]);
            } else if let Some(last) = defs.last_mut() {
                last.push(body.to_owned());
            }
            i += 1;
        }
        if defs.len() >= 2 {
            out.push((symbol, defs[0].join(" | "), defs[1].join(" | ")));
        }
    }
    out
}
pub fn judge(symbol: &str, kept: &str, dropped: &str) -> Dup {
    let k = origin_kind(kept);
    let d = origin_kind(dropped);
    let bare = symbol
        .strip_prefix("__declspec(dllimport) ")
        .unwrap_or(symbol);
    let verdict=if k=="runtime"&&d=="runtime"{"ok:toolchain runtime twice".into()}else if re(r"^_?(__rustc::)?(rust_begin_unwind|rust_eh_personality|rust_oom|__rust_[a-z_]+|_fltused|vibeos_unwind_resume_fallback)$").is_match(bare)&&(k=="local"||d=="local"){"ok:Rust panic runtime".into()}else if re(r"^_?vibeos_va_arg_(i32|u32|i64|u64|f64|ptr)$").is_match(bare)&&kept.contains("rust_varargs_shim.c")&&dropped.contains("rust_varargs_shim.c"){"ok:rustconv varargs puller (one generated body)".into()}else if k=="import"||d=="import"{if k=="import"&&d!="import"{format!("error:an import was kept over a {d} definition")}else{"ok:import".into()}}else if k=="runtime"{"ok:toolchain runtime kept over a local copy".into()}else if d=="runtime"&&re(r"^_?(memcpy|memmove|memset|memcmp|bcmp|strlen)$").is_match(bare){"ok:the CRT's own mem* over compiler_builtins' copy".into()}else if d=="runtime"{"error:a local copy was kept over the toolchain runtime".into()}else{"error:two local definitions, kept by link order".into()};
    Dup {
        symbol: symbol.into(),
        kept: kept.into(),
        dropped: dropped.into(),
        kept_kind: k.into(),
        dropped_kind: d.into(),
        verdict,
    }
}
pub fn output_of(argv: &[String]) -> String {
    for (i, a) in argv.iter().enumerate() {
        if a == "-o" {
            if let Some(n) = argv.get(i + 1) {
                return n.clone();
            }
        } else if let Some(n) = a.strip_prefix("-o") {
            if !n.is_empty() {
                return n.into();
            }
        }
    }
    "?".into()
}
/// The flag reactos-link's `Linker::shell_less` passes (its `SHELL_LESS_FLAG`).
pub const SHELL_LESS_FLAG: &str = "--vibeos-shell-less";

/// Whether `arg` is an input rustc put on the link line rather than the engine: the
/// bin crate's codegen units, rustc's `symbols.o`, the sysroot's `rsbegin.o`/`rsend.o`,
/// and every rlib (the bin's dependencies, `core`, `compiler_builtins`).
pub fn is_rustc_input(arg: &str) -> bool {
    !arg.starts_with('-')
        && re(r"(\.rcgu\.o|[/\\]rustc[^/\\]*[/\\]symbols\.o|[/\\]lib[/\\]rustlib[/\\][^/\\]+[/\\]lib[/\\](self-contained[/\\])?rs(begin|end)\.o|\.rlib)$")
            .is_match(arg)
}

/// A shell-less link's argv: `argv` without [`SHELL_LESS_FLAG`] and without rustc's
/// inputs. `None` when the flag is absent. Refuses when the flag is present but no
/// bin object is found, or a response file hides the inputs: either way rustc's link
/// line has changed shape, and linking it unfiltered would ship the shell silently.
pub fn shell_less(argv: &[String]) -> Result<Option<Vec<String>>> {
    if !argv.iter().any(|a| a == SHELL_LESS_FLAG) {
        return Ok(None);
    }
    if let Some(rsp) = argv.iter().find(|a| a.starts_with('@')) {
        return Err(Error::new(format!(
            "link-guard: {SHELL_LESS_FLAG} link reads a response file ({rsp}), which may hold rustc's inputs; expand it here before filtering"
        )));
    }
    let kept: Vec<String> = argv
        .iter()
        .filter(|a| *a != SHELL_LESS_FLAG && !is_rustc_input(a))
        .cloned()
        .collect();
    if !argv.iter().any(|a| a.ends_with(".rcgu.o")) {
        return Err(Error::new(format!(
            "link-guard: {SHELL_LESS_FLAG} link of {} has no rustc bin object (*.rcgu.o) to drop; rustc's link line changed",
            output_of(argv)
        )));
    }
    Ok(Some(kept))
}

/// Run the real linker, relay its output, append duplicate JSON, and refuse unsafe links.
pub fn execute(root: &Path, argv: &[String]) -> Result<i32> {
    let filtered = shell_less(argv)?;
    let argv = filtered.as_deref().unwrap_or(argv);
    let lld = std::env::var_os("VIBEOS_REAL_LLD")
        .filter(|x| !x.is_empty())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join(".build-tools/llvm-mingw/bin/ld.lld"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if !std::fs::metadata(&lld)
            .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        {
            eprintln!(
                "link-guard: real ld.lld not found at {} (set VIBEOS_REAL_LLD)",
                lld.display()
            );
            return Ok(2);
        }
    }
    let proc = Command::new(&lld)
        .args(argv)
        .stdout(Stdio::inherit())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| Error::new(format!("{}: {e}", lld.display())))?;
    std::io::stderr().write_all(&proc.stderr)?;
    if !proc.status.success() {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            return Ok(proc
                .status
                .code()
                .unwrap_or(128 + proc.status.signal().unwrap_or(1)));
        }
        #[cfg(not(unix))]
        {
            return Ok(proc.status.code().unwrap_or(1));
        }
    }
    let out = output_of(argv);
    let dups: Vec<_> = parse(&String::from_utf8_lossy(&proc.stderr))
        .iter()
        .map(|(s, k, d)| judge(s, k, d))
        .collect();
    if !dups.is_empty() {
        if let Some(log) = std::env::var_os("VIBEOS_LINK_GUARD_LOG").filter(|x| !x.is_empty()) {
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log)?;
            for d in &dups {
                let mut obj = serde_json::Map::new();
                obj.insert("output".into(), out.clone().into());
                let serde_json::Value::Object(rest) = serde_json::to_value(d)? else {
                    unreachable!()
                };
                obj.extend(rest);
                serde_json::to_writer(&mut f, &obj)?;
                writeln!(f)?;
            }
        }
    }
    let bad: Vec<_> = dups
        .iter()
        .filter(|d| d.verdict.starts_with("error:"))
        .collect();
    if bad.is_empty() || std::env::var("VIBEOS_LINK_GUARD").ok().as_deref() == Some("report") {
        return Ok(0);
    }
    eprintln!("\nlink-guard: {out}: {} duplicate symbol(s) resolved by link order (--allow-multiple-definition keeps the first). Delete the copy that should not exist; reordering the link is not a fix. See tools/link_guard.py.",bad.len());
    for d in bad {
        eprintln!(
            "  {}: {}\n      kept    ({}): {}\n      dropped ({}): {}",
            d.symbol,
            d.verdict.trim_start_matches("error:"),
            d.kept_kind,
            d.kept,
            d.dropped_kind,
            d.dropped
        );
    }
    if out != "?" {
        let path = Path::new(&out);
        match std::fs::remove_file(path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(Error::new(format!(
                    "failed to remove refused output {}: {e}",
                    path.display()
                )))
            }
        }
    }
    Ok(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn v(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }
    #[test]
    fn a_link_without_the_flag_is_untouched() {
        assert_eq!(shell_less(&v(&["-o", "a.exe", "x.rcgu.o"])).unwrap(), None);
    }
    #[test]
    fn a_shell_less_link_drops_only_rustcs_inputs() {
        let sys = "/r/lib/rustlib/i686-pc-windows-gnu/lib";
        let argv = v(&[
            "-m", "i386pe",
            &format!("{sys}/rsbegin.o"),
            "/t/deps/lz32-0ab.lz32.1c2d-cgu.0.rcgu.o",
            "/tmp/rustcAbC12/symbols.o",
            "/t/deps/libpanic_lean-9f.rlib",
            &format!("{sys}/libcore-1.rlib"),
            "/t/build/lz32-cargo-1/out/lz32.res.o",
            "/b/dll/win32/lz32/lz32.def",
            "/t/build/lz32-cargo-1/out/libkernel32.a",
            "--vibeos-shell-less",
            "-e", "___ImageBase",
            &format!("{sys}/rsend.o"),
            "-o", "/t/deps/lz32-0ab.exe",
        ]);
        assert_eq!(
            shell_less(&argv).unwrap().unwrap(),
            v(&[
                "-m", "i386pe",
                "/t/build/lz32-cargo-1/out/lz32.res.o",
                "/b/dll/win32/lz32/lz32.def",
                "/t/build/lz32-cargo-1/out/libkernel32.a",
                "-e", "___ImageBase",
                "-o", "/t/deps/lz32-0ab.exe",
            ])
        );
    }
    #[test]
    fn a_shell_less_link_with_no_bin_object_is_refused() {
        let e = shell_less(&v(&["--vibeos-shell-less", "-o", "a.exe", "a.o"])).unwrap_err();
        assert!(e.to_string().contains("no rustc bin object"), "{e}");
    }
    #[test]
    fn a_shell_less_link_through_a_response_file_is_refused() {
        let e = shell_less(&v(&["--vibeos-shell-less", "@/tmp/x.rsp"])).unwrap_err();
        assert!(e.to_string().contains("response file"), "{e}");
    }
}
