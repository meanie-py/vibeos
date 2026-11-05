//! `KeQuerySystemTime` expanded into a typed 64-bit read of `KUSER_SHARED_DATA.SystemTime`.
//!
//! SystemTime is a `KSYSTEM_TIME` at `KI_USER_SHARED_DATA + 0x14`, so it is only 4-aligned. The
//! 64-bit C macro reads it as `*(PULONG64)&SharedUserData->SystemTime`, which x64 and arm64
//! hardware tolerate; in Rust a typed `u64` deref of it is UB, and once a crate's debug-assertions
//! were armed rustc's alignment check bugchecked every boot that reached it (usbport, usbehci,
//! usbuhci and ksecdd, 2026-11-05). Translations also invented a `SharedSystemTime` GLOBAL: classpnp
//! and fastfat each defined one as a static zero, so their system time always read 1601.
//! Read it with `ntkrnl_sys::ke::ke_query_system_time()` (or `read_unaligned`).
//! Flagged: a `static SharedSystemTime` definition or declaration, and a deref of a
//! `SharedSystemTime`/`SharedSystemTimePtr()` expression as a 64-bit type that is not wrapped in
//! `read_unaligned`/`read_volatile`. `// nolint: shared-system-time (why)` on the line suppresses.
use crate::{Corpus, Rule};
use rayon::prelude::*;
use regex::Regex;
use std::sync::LazyLock;

pub struct SharedSystemTime;
impl Rule for SharedSystemTime {
    fn name(&self) -> &'static str {
        "scan_shared_system_time"
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let files: Vec<_> = c
            .rust_files()
            .filter(|f| f.text.contains("SharedSystemTime"))
            .filter(|f| !SKIP.is_match(&f.path.to_string_lossy()))
            .collect();
        let mut out: Vec<String> = files
            .par_iter()
            .flat_map_iter(|f| scan(&f.text).into_iter().map(move |x| format!("{}:{x}", f.path.display())))
            .collect();
        out.sort();
        Ok(out)
    }
}

static SKIP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/types_[a-z0-9]+\.rs$)").unwrap()
});
static STATIC_DEF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bstatic\s+(?:mut\s+)?SharedSystemTime\s*:").unwrap());
/// `*(<expr naming SharedSystemTime> as *mut u64)` -- the deref star opens the parenthesis.
static TYPED_DEREF: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\*\s*\(\s*\(?[^;()]*\bSharedSystemTime(?:Ptr\s*\(\s*\))?[^;]*?\bas\s+\*\s*(?:mut|const)\s+(?:[\w:]+::)?(?:u64|i64|ULONG64|ULONGLONG|LONGLONG|LARGE_INTEGER|DWORD64)\s*\)",
    )
    .unwrap()
});
/// `*(SharedSystemTimePtr())` -- the helper already returns `*mut ULONG64`.
static PTR_DEREF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\*\s*\(\s*SharedSystemTimePtr\s*\(\s*\)\s*\)").unwrap());
static NOLINT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bnolint:[\w\s,-]*\bshared-system-time\b").unwrap());

fn scan(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let code = line.split("//").next().unwrap_or("");
        if NOLINT.is_match(line) || !code.contains("SharedSystemTime") {
            continue;
        }
        if STATIC_DEF.is_match(code) {
            out.push(format!(
                "{}: shared-system-time: a `SharedSystemTime` static -- KeQuerySystemTime reads KUSER_SHARED_DATA, use ntkrnl_sys::ke::ke_query_system_time()",
                n + 1
            ));
            continue;
        }
        let typed = TYPED_DEREF.find(code).or_else(|| PTR_DEREF.find(code));
        if let Some(m) = typed {
            let before = code[..m.start()].trim_end();
            if before.ends_with("read_unaligned(") || before.ends_with("read_volatile(") {
                continue;
            }
            out.push(format!(
                "{}: shared-system-time: typed 64-bit deref of SystemTime (KUSER_SHARED_DATA+0x14 is 4-aligned) -- use ke_query_system_time() or read_unaligned",
                n + 1
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_reads_and_fake_globals_are_flagged() {
        let s = "fn f() {\n    *(core::ptr::addr_of_mut!(t) as *mut u64) = *(SharedSystemTime as *mut u64);\n    let x = *(SharedSystemTimePtr() as *mut ULONG64);\n    let y = *(SharedSystemTimePtr());\n}\n#[no_mangle] pub static mut SharedSystemTime: LARGE_INTEGER = unsafe { core::mem::zeroed() };\nextern \"C\" {\n    static SharedSystemTime: LARGE_INTEGER;\n}\n";
        let got = scan(s);
        assert_eq!(got.len(), 5, "{got:?}");
        assert!(got[0].starts_with("2: shared-system-time"), "{got:?}");
    }

    #[test]
    fn unaligned_reads_helpers_and_comments_pass() {
        let ok = "fn f() {\n    let x = core::ptr::read_unaligned(SharedSystemTimePtr() as *const ULONG64);\n    t = ntkrnl_sys::ke::ke_query_system_time();\n    // was *(SharedSystemTime as *mut u64)\n    pub const SharedSystemTime: usize = BASE + 0x14;\n    let y = *(SharedSystemTime as *mut u64); // nolint: shared-system-time (i386 export form)\n}\n";
        assert!(scan(ok).is_empty(), "{:?}", scan(ok));
    }
}
