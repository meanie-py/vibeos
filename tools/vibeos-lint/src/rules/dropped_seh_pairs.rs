//! Pair a C SEH handler with a Rust raiser lacking any supported catch idiom.
use crate::{Corpus, Rule, SourceFile, SourceSet};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
pub struct DroppedSehPairs;
static INPUTS: &[SourceSet] = &[
    SourceSet::tree("seh_c_sources", &[""], &["c"], &[]).with_pruning(&["target"]),
    SourceSet::tree("seh_cmake", &[""], &["cmake"], &["CMakeLists.txt"]).with_pruning(&["target"]),
];
const RAISERS: &[&str] = &[
    "ProbeForRead",
    "ProbeForWrite",
    "ExRaiseStatus",
    "ExRaiseAccessViolation",
    "ExRaiseDatatypeMisalignment",
];
const CATCHERS: &[&str] = &[
    "probe::",
    "probe_user_buffer",
    "safe_copy_raw",
    "safe_copy_to_user",
    "safe_copy_from_user",
    "Result<(), NTSTATUS>",
    "catch_unwind",
    "_SEH2",
    "try_probe",
];
static COMMENTED_C: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"(?m)^\s*#.*?([A-Za-z0-9_./-]+\.c)\s*$").unwrap());
static LIVE_C: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"(?m)^\s*(\S*?([A-Za-z0-9_./-]+\.c))\s*$").unwrap());
impl Rule for DroppedSehPairs {
    fn name(&self) -> &'static str {
        "scan_dropped_seh_pairs"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        scan(c)
    }
}
fn scan(c: &Corpus) -> Result<Vec<String>, String> {
    let mut rust = HashMap::<String, Vec<&SourceFile>>::new();
    for r in c.rust_files() {
        rust.entry(
            r.path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
        )
        .or_default()
        .push(r)
    }
    let mut flipped = HashSet::new();
    let mut live = HashSet::new();
    for f in c.sources("seh_cmake").filter(|f| {
        f.path.file_name().is_some_and(|n| n == "CMakeLists.txt")
            || f.path.extension().is_some_and(|e| e == "cmake")
    }) {
        for m in COMMENTED_C.captures_iter(&f.text) {
            insert_key(&mut flipped, &m[1]);
        }
        for m in LIVE_C.captures_iter(&f.text).filter(|m| {
            !f.text[m.get(0).unwrap().start()..]
                .trim_start()
                .starts_with('#')
        }) {
            insert_key(&mut live, &m[2]);
        }
    }
    let mut out = Vec::new();
    for cf in c.sources("seh_c_sources").filter(|f| {
        !f.path.components().any(|p| p.as_os_str() == "target")
            && f.path.extension().is_some_and(|e| e == "c")
            && f.text.contains("_SEH2_EXCEPT")
    }) {
        let stem = cf.path.file_stem().unwrap_or_default().to_string_lossy();
        let key = cf
            .path
            .iter()
            .rev()
            .take(2)
            .map(|p| p.to_string_lossy())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("/");
        let state = if flipped.contains(&key) && !live.contains(&key) {
            "C-FLIPPED-OUT"
        } else {
            "C-STILL-BUILT"
        };
        for rf in rust.get(stem.as_ref()).into_iter().flatten() {
            let raises: Vec<_> = RAISERS
                .iter()
                .copied()
                .filter(|r| rf.text.contains(r))
                .collect();
            if raises.is_empty() || CATCHERS.iter().any(|x| rf.text.contains(x)) {
                continue;
            }
            let subsys = cf
                .path
                .parent()
                .and_then(|p| p.file_name())
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase();
            let krate = rf
                .path
                .strip_prefix("sdk/lib/rust")
                .ok()
                .and_then(|p| p.components().next())
                .map(|p| p.as_os_str().to_string_lossy().to_lowercase())
                .unwrap_or_default();
            let same = !subsys.is_empty()
                && (krate.contains(&subsys)
                    || rf
                        .path
                        .parent()
                        .and_then(|p| p.file_name())
                        .is_some_and(|p| p.to_string_lossy().to_lowercase() == subsys));
            if !same || state != "C-FLIPPED-OUT" {
                continue;
            }
            let mut raises = raises;
            raises.sort();
            out.push(format!(
                "{}: raises {} with no catch — C at {} has _SEH2_EXCEPT [{}]",
                rf.path.display(),
                raises.join(","),
                cf.path.display(),
                state
            ));
        }
    }
    out.sort();
    Ok(out)
}
fn insert_key(set: &mut HashSet<String>, path: &str) {
    let key = path
        .rsplit('/')
        .take(2)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("/");
    set.insert(key);
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    fn c(files: &[(&str, &str)]) -> Corpus {
        Corpus::new(
            PathBuf::from("."),
            files
                .iter()
                .map(|(p, t)| SourceFile::new(p.into(), (*t).into()))
                .collect(),
        )
    }
    #[test]
    fn actual_control_bad_and_good() {
        let bad=c(&[("ntoskrnl/lpc/reply.c","NTSTATUS NtReplyPort(){ _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { ProbeForRead(P,1,1); } }"),("sdk/lib/rust/lpc-rust/src/reply.rs","pub unsafe fn NtReplyPort(){ ProbeForRead(p,1,1); }"),("CMakeLists.txt","#${REACTOS_SOURCE_DIR}/ntoskrnl/lpc/reply.c")]);
        assert_eq!(scan(&bad).unwrap().len(), 1);
        let good=c(&[("ntoskrnl/lpc/reply.c","NTSTATUS NtReplyPort(){ _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { ProbeForRead(P,1,1); } }"),("sdk/lib/rust/lpc-rust/src/reply.rs","pub unsafe fn NtReplyPort(){ ntkrnl_sys::probe::probe_read_user(p); }"),("CMakeLists.txt","#${REACTOS_SOURCE_DIR}/ntoskrnl/lpc/reply.c")]);
        assert!(scan(&good).unwrap().is_empty());
    }
}

#[cfg(test)]
mod original_control {
    use super::*;
    use std::path::PathBuf;
    const SEHPAIR_CMAKE: &str =
        "\nlist(APPEND SOURCE\n    #${REACTOS_SOURCE_DIR}/ntoskrnl/lpc/reply.c\n)\n";
    const SEHPAIR_C:&str="\nNTSTATUS NtReplyPort(HANDLE PortHandle, PPORT_MESSAGE ReplyMessage)\n{\n    if (PreviousMode != KernelMode)\n    {\n        _SEH2_TRY\n        {\n            ProbeForRead(ReplyMessage, sizeof(*ReplyMessage), sizeof(ULONG));\n            Captured = *(volatile PORT_MESSAGE*)ReplyMessage;\n        }\n        _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)\n        {\n            _SEH2_YIELD(return _SEH2_GetExceptionCode());\n        }\n        _SEH2_END;\n    }\n    return STATUS_SUCCESS;\n}\n";
    const SEHPAIR_BAD_RS:&str="\n#[no_mangle]\npub unsafe extern \"system\" fn NtReplyPort(ReplyMessage: PPORT_MESSAGE) -> NTSTATUS {\n    if previous_mode != 0i8 {\n        ProbeForRead(ReplyMessage as PVOID, core::mem::size_of::<PORT_MESSAGE>(), 4);\n        let captured = *(ReplyMessage as *const PORT_MESSAGE);\n    }\n    STATUS_SUCCESS\n}\n";
    const SEHPAIR_GOOD_RS:&str="\n#[no_mangle]\npub unsafe extern \"system\" fn NtReplyPort(ReplyMessage: PPORT_MESSAGE) -> NTSTATUS {\n    if previous_mode != 0i8 {\n        let captured = match ntkrnl_sys::probe::probe_read_generic::<PORT_MESSAGE>(\n            ReplyMessage as *const PORT_MESSAGE, 4, previous_mode,\n        ) {\n            Ok(v) => v,\n            Err(st) => return st,\n        };\n    }\n    STATUS_SUCCESS\n}\n";
    fn fixture(rs: &str) -> Corpus {
        Corpus::new(
            PathBuf::from("."),
            vec![
                SourceFile::new("ntoskrnl/lpc/reply.c".into(), SEHPAIR_C.into()),
                SourceFile::new("sdk/lib/rust/lpc-rust/src/reply.rs".into(), rs.into()),
                SourceFile::new("ntos.cmake".into(), SEHPAIR_CMAKE.into()),
            ],
        )
    }
    #[test]
    fn exact_bad_good_scanner_control() {
        let bad = scan(&fixture(SEHPAIR_BAD_RS)).unwrap();
        assert_eq!(bad.len(), 1);
        assert!(bad[0].contains("C-FLIPPED-OUT"));
        assert!(scan(&fixture(SEHPAIR_GOOD_RS)).unwrap().is_empty());
    }
}
