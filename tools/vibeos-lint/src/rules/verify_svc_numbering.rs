//! Rust port of the registry's default `verify_svc_numbering.py` invocation.
use crate::{Corpus, Rule};
use regex::Regex;
use std::collections::BTreeMap;
use std::process::Command;
use std::sync::OnceLock;

pub struct VerifySvcNumbering;
fn table_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        crate::regexes::cached(r#"^\s*\{\s*"([^"]+)",\s*(?:\S+|0),\s*\d+\s*\},\s*/\* \[(\d+)\]"#)
            .unwrap()
    })
}
fn label_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        crate::regexes::cached(r"^[0-9a-f]+ <_?(?:Nt|Zw)((?:User|Gdi)\w+?)(?:@\d+)?>:").unwrap()
    })
}
fn eax_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| crate::regexes::cached(r"movl\s+\$0x(1[0-9a-f]{3}),\s*%eax").unwrap())
}

impl Rule for VerifySvcNumbering {
    fn needs_workspace_rust(&self) -> bool {
        false
    }
    fn name(&self) -> &'static str {
        "verify_svc_numbering"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let root = &corpus.root;
        let generated = root.join("build-arm64/dll/win32/wow64/wow64_win32k_thunks.gen.c");
        let base32 = root.join("build-i386");
        let dlls = [
            base32.join("win32ss/user/user32/user32.dll"),
            base32.join("win32ss/gdi/gdi32/gdi32.dll"),
        ];
        if !generated.exists() || dlls.iter().any(|p| !p.exists()) {
            return Ok(Vec::new()); // original prints its SKIP only to stderr
        }
        let objdump = root.join(".build-tools/llvm-mingw/bin/llvm-objdump");
        if !objdump.exists() {
            return Ok(Vec::new());
        }
        let table_text = std::fs::read_to_string(&generated).map_err(|e| {
            format!(
                "verify_svc_numbering: cannot read {}: {e}",
                generated.display()
            )
        })?;
        let mut table = BTreeMap::new();
        for line in table_text.lines() {
            if let Some(m) = table_re().captures(line) {
                let idx = m[2]
                    .parse::<u32>()
                    .map_err(|e| format!("bad syscall index: {e}"))?;
                table.insert(idx, m[1].to_owned());
            }
        }
        let mut stubs = BTreeMap::new();
        for dll in &dlls {
            let output = Command::new(&objdump)
                .arg("-d")
                .arg(dll)
                .output()
                .map_err(|e| format!("{}: {e}", objdump.display()))?;
            // The Python scanner consumes stdout even if objdump returned nonzero.
            let mut current = None;
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                if let Some(m) = label_re().captures(line) {
                    current = Some(format!("Nt{}", &m[1]));
                    continue;
                }
                if let Some(name) = current.as_ref() {
                    if let Some(m) = eax_re().captures(line) {
                        let raw = u32::from_str_radix(&m[1], 16)
                            .map_err(|e| format!("invalid guest EAX service number: {e}"))?;
                        stubs.insert(name.clone(), raw - 0x1000);
                        current = None;
                    }
                }
            }
        }
        if table.is_empty() || stubs.is_empty() {
            return Ok(Vec::new());
        }
        let top = *table.keys().next_back().unwrap();
        let mut bad = stubs
            .iter()
            .filter(|(name, idx)| {
                **idx <= top && table.get(*idx).map(String::as_str) != Some(name.as_str())
            })
            .map(|(name, idx)| (*idx, name.clone(), table.get(idx).cloned()))
            .collect::<Vec<_>>();
        bad.sort();
        if bad.is_empty() {
            return Ok(Vec::new());
        }
        let mut report = format!(
            "verify_svc_numbering: FAIL — {} of {} guest syscalls are misrouted by the WoW64 win32k thunk table.\nThe table is indexed by the HOST's numbering; it must use the GUEST's (i386) numbering. See parse_sysfuncs() in wow64gen.py.\n",
            bad.len(), stubs.len()
        );
        for (idx, guest, host) in bad.iter().take(12) {
            report.push_str(&format!(
                "   svc 0x{:04x} (idx {:4}): guest calls {:34} -> host table dispatches {}\n",
                idx + 0x1000,
                idx,
                guest,
                host.as_deref().unwrap_or("None")
            ));
        }
        if bad.len() > 12 {
            report.push_str(&format!("   ... and {} more\n", bad.len() - 12));
        }
        Err(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn table_parser_includes_manual_rows_and_reads_guest_ordinal() {
        let m = table_re()
            .captures("{ \"NtUserFoo\", wow64_thunk_NtUserFoo, 2 }, /* [551] */")
            .unwrap();
        assert_eq!(&m[1], "NtUserFoo");
        assert_eq!(&m[2], "551");
        let manual = table_re()
            .captures("{ \"NtUserBar\", 0, 15 }, /* [342] MANUAL */")
            .unwrap();
        assert_eq!(&manual[1], "NtUserBar");
        assert_eq!(&manual[2], "342");
    }
    #[test]
    fn disassembly_parsers_match_the_original_stub_forms() {
        let label = label_re().captures("1234 <_ZwUserShowWindow@8>:").unwrap();
        assert_eq!(&label[1], "UserShowWindow");
        assert_eq!(
            u32::from_str_radix(
                &eax_re().captures("  0000: movl $0x1227, %eax").unwrap()[1],
                16
            )
            .unwrap()
                - 0x1000,
            0x227
        );
    }
}
