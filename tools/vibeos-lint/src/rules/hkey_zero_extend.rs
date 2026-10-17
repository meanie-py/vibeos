//! A predefined registry key ZERO-extended to a 64-bit HKEY.
//!
//! ```text
//! winreg.h:    #define HKEY_LOCAL_MACHINE ((HKEY)(ULONG_PTR)((LONG)0x80000002))
//! translated:  pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002usize as HKEY;
//! ```
//!
//! The `(LONG)` makes the value negative before it widens, so on amd64 and arm64 the C
//! handle is 0xFFFFFFFF80000002; a literal cast through `usize`/`u32` is 0x0000000080000002.
//! advapi32 recognises a predefined key by masking the low 32 bits, so calls still work, but
//! the Rust constant never compares equal to a HKEY that C code made. 118 sites in 64 crates
//! when this landed (2026-10-17), inline call arguments as well as prelude constants.
//!
//! Fix: `0x8000_0002u32 as i32 as isize as HKEY`, the sign-extending chain winreg.h means.
//! A `port replay` that restores a bought body's old literal trips this again.
use crate::{Corpus, Rule};
use regex::Regex;

pub struct HkeyZeroExtend;

struct Patterns {
    /// 0x80000000..0x80000006 (HKEY_CLASSES_ROOT..HKEY_DYN_DATA), then an `as` chain. A `)`
    /// inside the chain closes a parenthesis opened before the literal.
    site: Regex,
    ends_hkey: Regex,
    /// A step that makes the value a signed 32-bit integer before it widens.
    signed32: Regex,
    nolint: Regex,
}

impl Patterns {
    fn new() -> Self {
        let re = |s| Regex::new(s).expect("valid hkey-zero-extend regex");
        Self {
            site: re(
                r"\b0x8000_?000[0-6](?:_?(?:u32|i32|u64|i64|usize|isize))?((?:\s*\)*\s*as\s+(?:::)?[\w:]+)+)",
            ),
            ends_hkey: re(r"\bas\s+(?:[\w:]*::)?HKEY\s*$"),
            signed32: re(r"\bas\s+(?:(?:::)?core::ffi::)?(?:i32|c_int|c_long|LONG|INT)\b"),
            nolint: re(r"//\s*nolint(?::\s*hkey-zero-extend\b|\s*$)"),
        }
    }
}

fn scan(text: &str, p: &Patterns) -> Vec<usize> {
    let mut out = Vec::new();
    for caps in p.site.captures_iter(text) {
        let chain = caps.get(1).unwrap().as_str();
        if !p.ends_hkey.is_match(chain) || p.signed32.is_match(chain) {
            continue;
        }
        let at = caps.get(0).unwrap().start();
        let start = text[..at].rfind('\n').map_or(0, |i| i + 1);
        let end = text[at..].find('\n').map_or(text.len(), |i| at + i);
        let line = &text[start..end];
        if p.nolint.is_match(line) || line.trim_start().starts_with("//") {
            continue;
        }
        let lineno = text[..at].matches('\n').count() + 1;
        if out.last() != Some(&lineno) {
            out.push(lineno);
        }
    }
    out
}

impl Rule for HkeyZeroExtend {
    fn name(&self) -> &'static str {
        "scan_hkey_zero_extend"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let p = Patterns::new();
        let mut files: Vec<_> = corpus
            .rust_files()
            .filter(|f| f.text.contains("0x8000") && f.text.contains("HKEY"))
            .collect();
        files.sort_by(|a, b| a.path.cmp(&b.path));
        let mut out = Vec::new();
        for f in files {
            for line in scan(&f.text, &p) {
                out.push(format!("{}:{line}: hkey-zero-extend", f.path.display()));
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hits(s: &str) -> usize {
        scan(s, &Patterns::new()).len()
    }

    #[test]
    fn zero_extending_chains_are_flagged() {
        // winlogon's preamble and iphlpapi's prelude, and an inline call argument (userenv).
        assert_eq!(hits("pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002usize as HKEY;"), 1);
        assert_eq!(hits("const HKEY_USERS: HKEY = 0x80000003u32 as usize as HKEY;"), 1);
        assert_eq!(hits("f(0x8000_0002u32 as isize as HKEY);"), 1);
        assert_eq!(hits("RegOpenKeyExW((0x80000001) as HKEY, k, 0, KEY_READ, &mut h);"), 1);
    }

    #[test]
    fn sign_extending_and_unrelated_casts_pass() {
        assert_eq!(hits("const K: HKEY = 0x8000_0002u32 as i32 as isize as HKEY;"), 0);
        assert_eq!(hits("const K: HKEY = 0x80000003u32 as ::core::ffi::c_int as isize as HKEY;"), 0);
        assert_eq!(hits("const S: NTSTATUS = 0x8000_0001u32 as NTSTATUS;"), 0);
        assert_eq!(hits("// was 0x8000_0002usize as HKEY"), 0);
        assert_eq!(hits("let k = 0x8000_0002usize as HKEY; // nolint: hkey-zero-extend"), 0);
    }
}
