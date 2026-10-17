//! Flag calls to kernel user-pointer probe primitives that raise outside an SEH frame.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::OnceLock;

pub struct RaisingProbes;
impl Rule for RaisingProbes {
    fn name(&self) -> &'static str {
        "scan_raising_probes"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.rust_files() {
            out.extend(scan_file(&file.path.to_string_lossy(), &file.text));
        }
        Ok(out)
    }
}

const RAISING: &[&str] = &[
    "ProbeForRead",
    "ProbeForWrite",
    "ProbeForWriteHandle",
    "ProbeForWriteUlong",
    "ProbeForWritePointer",
    "ProbeForWriteBoolean",
    "ProbeForWriteLangId",
    "ProbeForWriteSize_t",
    "ProbeForWriteIoStatusBlock",
    "ProbeForReadLargeInteger",
    "ProbeForReadUchar",
    "ProbeForReadUshort",
    "MmProbeAndLockPages",
];
const EXEMPT_FILES: &[&str] = &[
    "ex-rust/src/exintrin.rs",
    "ntkrnl-sys/src/inline/common.rs",
    "ntkrnl-sys/src/inline/mm.rs",
    "ntkrnl-sys/src/macros.rs",
    "ntkrnl-sys/src/probe.rs",
];
struct Patterns {
    calls: Regex,
    decl: Regex,
    try_frame: Regex,
    nolint: Regex,
}
fn patterns() -> &'static Patterns {
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(|| {
        let names = RAISING.join("|");
        Patterns {
            calls: crate::regexes::cached(&format!(r"\b({names})\s*\(")).unwrap(),
            decl: crate::regexes::cached(&format!(r"\bfn\s+({names})\b")).unwrap(),
            try_frame: crate::regexes::cached(r"\btry_(?:except|finally)\s*\(").unwrap(),
            nolint: crate::regexes::cached(
                r"nolint:\s*(raising-probe|raising_probe|scan_raising_probes|all)\b",
            )
            .unwrap(),
        }
    })
}

fn scan_file(path: &str, text: &str) -> Vec<String> {
    if EXEMPT_FILES.iter().any(|suffix| path.ends_with(suffix)) {
        return Vec::new();
    }
    let stripped = strip_comments(text);
    let codes: Vec<&str> = stripped.iter().map(|(code, _)| code.as_str()).collect();
    let guarded = try_frame_lines(&codes);
    let mut findings = Vec::new();
    for (i, (code, raw)) in stripped.iter().enumerate() {
        if patterns().decl.is_match(code)
            || patterns().nolint.is_match(raw)
            || guarded.contains(&(i + 1))
        {
            continue;
        }
        let Some(capture) = patterns().calls.captures(code) else {
            continue;
        };
        let m = capture.get(0).unwrap();
        let Some(name_match) = capture.get(1) else {
            continue;
        };
        let name = name_match.as_str();
        let before = code[..m.start()].chars().next_back();
        let any_path = name == "MmProbeAndLockPages";
        let blocked =
            before.is_some_and(|c| c == '_' || c.is_alphanumeric() || (!any_path && c == ':'));
        if blocked {
            continue;
        }
        findings.push(format!(
            "{path}:{}: raising {name}() — no SEH here; use ntkrnl_sys::probe",
            i + 1
        ));
    }
    findings
}

/// The per-line comment pass, with string-literal text blanked (the quotes stay).
fn strip_comments(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut in_block = false;
    for raw in text.split('\n') {
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        let bytes = raw.as_bytes();
        let mut code = Vec::new();
        let mut i = 0;
        let mut in_string = false;
        while i < bytes.len() {
            if in_block {
                if let Some(end) = bytes[i..].windows(2).position(|w| w == b"*/") {
                    in_block = false;
                    i += end + 2;
                } else {
                    i = bytes.len();
                }
                continue;
            }
            let b = bytes[i];
            if in_string {
                // A string's text is blanked: tdihelpers' `DPRINT!("MmProbeAndLockPages()
                // failed.\n")` inside a try_except's handler read as an unguarded call.
                if b == b'\\' {
                    code.extend_from_slice(if i + 1 < bytes.len() { b"  " } else { b" " });
                    i += 2;
                    continue;
                }
                if b == b'"' {
                    in_string = false;
                    code.push(b);
                } else {
                    code.push(b' ');
                }
                i += 1;
                continue;
            }
            if bytes.get(i..i + 2) == Some(b"//") {
                break;
            }
            if bytes.get(i..i + 2) == Some(b"/*") {
                in_block = true;
                i += 2;
                continue;
            }
            if b == b'"' {
                in_string = true;
                code.push(b' ');
                i += 1;
                continue;
            }
            code.push(b);
            i += 1;
        }
        out.push((String::from_utf8_lossy(&code).into_owned(), raw.to_owned()));
    }
    out
}

fn try_frame_lines(lines: &[&str]) -> std::collections::HashSet<usize> {
    let mut inside = std::collections::HashSet::new();
    for (start, code) in lines.iter().enumerate() {
        for m in patterns().try_frame.find_iter(code) {
            let mut depth = 0i32;
            let mut row = start;
            let mut col = m.end() - 1;
            while row < lines.len() {
                let line = lines[row].as_bytes();
                while col < line.len() {
                    match line[col] {
                        b'(' => depth += 1,
                        b')' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => (),
                    }
                    col += 1;
                }
                inside.insert(row + 1);
                if depth == 0 && col < line.len() {
                    break;
                }
                row += 1;
                col = 0;
            }
        }
    }
    inside
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceFile;
    use std::path::PathBuf;

    const BAD: &str = r#"
extern "system" {
    fn ProbeForWrite(Address: PVOID, Length: usize, Alignment: usize);
}
pub unsafe extern "system" fn NtAllocateLocallyUniqueId(LocallyUniqueId: *mut LUID) -> NTSTATUS {
    if ExGetPreviousMode() != 0 {
        ProbeForWrite(LocallyUniqueId as PVOID, core::mem::size_of::<LUID>(), 4);
    }
    STATUS_SUCCESS
}
"#;
    const GOOD: &str = r#"
extern "system" {
    fn ProbeForWrite(Address: PVOID, Length: usize, Alignment: usize);
}
pub unsafe extern "C" fn ProbeForWriteUlong(address: *mut ULONG) {
    ntkrnl_sys::inline::common::ProbeForWriteUlong(address)
}
pub unsafe extern "system" fn NtAllocateLocallyUniqueId(LocallyUniqueId: *mut LUID) -> NTSTATUS {
    if ExGetPreviousMode() != 0 {
        // ProbeForWrite(LocallyUniqueId, sizeof(LUID), sizeof(ULONG));
        let probed = ntkrnl_sys::probe::probe_user_buffer(LocallyUniqueId as PVOID, 8, 4, 1);
        if let Err(status) = probed {
            return status;
        }
    }
    STATUS_SUCCESS
}
"#;
    const MDL_BAD: &str = r#"
pub unsafe extern "C" fn HackSecureVirtualMemory(pmdl: PMDL) -> NTSTATUS {
    ntkrnl_sys::ddk::MmProbeAndLockPages(pmdl, 1i8, 0);
    STATUS_SUCCESS
}
"#;
    const MDL_GOOD: &str = r#"
extern "system" { fn MmProbeAndLockPages(Mdl: PMDL, AccessMode: i8, Operation: u32); }
pub unsafe extern "C" fn HackSecureVirtualMemory(pmdl: PMDL) -> NTSTATUS {
    let status = ntkrnl_sys::probe::MmTryProbeAndLockPages(pmdl, 1i8, 0);
    match ntkrnl_sys::seh::try_except(&mut (), |_| { MmProbeAndLockPages(pmdl, 0i8, 1); }, |_, _| 1) {
        Ok(()) => STATUS_SUCCESS, Err(code) => code,
    }
}
"#;
    fn corpus(path: &str, text: &str) -> Corpus {
        Corpus::new(
            PathBuf::from("/fixture"),
            vec![SourceFile::new(path.into(), text.into())],
        )
    }

    #[test]
    fn scanner_control_probe_for_write_bad_and_good() {
        assert_eq!(RaisingProbes.run(&corpus("sdk/lib/rust/ex-rust/src/uuid.rs", BAD)).unwrap(), vec!["sdk/lib/rust/ex-rust/src/uuid.rs:7: raising ProbeForWrite() — no SEH here; use ntkrnl_sys::probe"]);
        assert!(RaisingProbes
            .run(&corpus("sdk/lib/rust/ex-rust/src/uuid.rs", GOOD))
            .unwrap()
            .is_empty());
    }
    #[test]
    fn a_probe_named_in_a_string_is_not_a_call() {
        let s = "DPRINT!(\"MmProbeAndLockPages() failed.\\n\");\nlet m = \"a \\\" ProbeForWrite(x)\";\n";
        assert!(scan_file("sdk/lib/rust/a/src/x.rs", s).is_empty());
        assert_eq!(scan_file("sdk/lib/rust/a/src/x.rs", "g(\"x\"); MmProbeAndLockPages(m, 0, 1);\n").len(), 1);
    }
    #[test]
    fn scanner_control_mdl_any_path_and_seh_guard() {
        assert_eq!(
            RaisingProbes
                .run(&corpus("sdk/lib/rust/win32k-rust/src/mem.rs", MDL_BAD))
                .unwrap()
                .len(),
            1
        );
        assert!(RaisingProbes
            .run(&corpus("sdk/lib/rust/win32k-rust/src/mem.rs", MDL_GOOD))
            .unwrap()
            .is_empty());
    }
    #[test]
    fn declarations_shadow_defs_qualified_exports_nolint_and_exempt_paths() {
        let s = "extern \"C\" { fn ProbeForWrite(a: u8); }\nfn ProbeForReadUnicodeString() -> Result<(), ()> { Ok(()) }\nntkrnl_sys::inline::common::ProbeForWrite(a);\nProbeForWrite(a); // nolint: raising_probe reviewed\nProbeForWrite(a);\n";
        assert_eq!(scan_file("sdk/lib/rust/a/src/x.rs", s), vec!["sdk/lib/rust/a/src/x.rs:5: raising ProbeForWrite() — no SEH here; use ntkrnl_sys::probe"]);
        assert!(scan_file(
            "sdk/lib/rust/ntkrnl-sys/src/inline/mm.rs",
            "ProbeForWrite(a);\n"
        )
        .is_empty());
    }
    #[test]
    fn multiline_seh_closure_and_all_nolint_aliases() {
        let s = "seh::try_finally(\n  || ProbeForRead(a),\n  || cleanup(),\n);\nProbeForRead(a); // nolint: scan_raising_probes reviewed\n";
        assert!(scan_file("x.rs", s).is_empty());
        let s = "seh::try_except(\n  || ProbeForRead(a),\n  || 0,\n);\nProbeForWrite(a); // nolint: all\n";
        assert!(scan_file("x.rs", s).is_empty());
    }
}
