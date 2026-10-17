//! Report reachable cohab bugcheck placeholders.
use crate::{Corpus, Rule, SourceFile, SourceSet};
use std::collections::{HashMap, HashSet};
pub struct BugcheckStubs;
static INPUTS: &[SourceSet] = &[
    SourceSet::tree("bugcheck_stub_files", &[""], &[], &["rust_cohab_stub.c"])
        .with_pruning(&["target", ".git"]),
    SourceSet::tree("bugcheck_driver_headers", &[""], &["h"], &[])
        .with_pruning(&["target", ".git"]),
    SourceSet::tree("bugcheck_git_markers", &[""], &[], &[".git"])
        .with_pruning(&["target", ".git"]),
];
impl Rule for BugcheckStubs {
    fn name(&self) -> &'static str {
        "scan_bugcheck_stubs"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let bug =
            crate::regexes::cached(r"(?i)STUB_BUGCHECK\s*\(\s*\)|KeBugCheckEx\s*\([^;]*0xdead")
                .unwrap();
        let def =
            crate::regexes::cached(r"(?m)^[A-Za-z_][\w \t\*]*?\b([A-Za-z_]\w*)\s*\(").unwrap();
        let com = crate::regexes::cached(r"//[^\n]*|/\*[\s\S]*?\*/").unwrap();
        let mut out = Vec::new();
        for f in c
            .sources("bugcheck_stub_files")
            .filter(|f| f.path.file_name().is_some_and(|n| n == "rust_cohab_stub.c"))
        {
            if nested_checkout(&f.path, &c.root, c.sources("bugcheck_git_markers")) {
                continue;
            }
            let src = com.replace_all(&f.text, " ").into_owned();
            let mut names = Vec::<String>::new();
            for hit in bug.find_iter(&src) {
                let before = &src[..hit.start()];
                let ls = before.rfind('\n').map_or(0, |x| x + 1);
                let mut logical = &before[ls..];
                let mut line_start = ls;
                while logical.trim_end().ends_with('\\') {
                    let previous = before[..line_start].rfind('\n').map_or(0, |x| x + 1);
                    logical = &before[previous..line_start];
                    line_start = previous;
                }
                if logical.trim_start().starts_with("#define") {
                    continue;
                }
                if let Some(m) = def.find_iter(before).last() {
                    if let Some(caps) = def.captures(m.as_str()) {
                        let n = caps[1].to_owned();
                        if n != "STUB_BUGCHECK" && !names.contains(&n) {
                            names.push(n)
                        }
                    }
                }
            }
            let driver = f.path.parent().unwrap_or(std::path::Path::new(""));
            let rel = f.path.to_string_lossy();
            let localcrate = format!(
                "{}-rust",
                driver.file_name().unwrap_or_default().to_string_lossy()
            );
            let crate_prefix = format!("sdk/lib/rust/{localcrate}/");
            let mut rust = c.rust_files().collect::<Vec<_>>();
            let own = rust
                .iter()
                .any(|r| r.path.to_string_lossy().starts_with(&crate_prefix));
            if own {
                rust.retain(|r| r.path.to_string_lossy().starts_with(&crate_prefix));
            }
            let mut declarations = HashMap::<String, String>::new();
            let mut calls = HashMap::<String, usize>::new();
            let mut macros = HashSet::<String>::new();
            for h in c.sources("bugcheck_driver_headers").filter(|h| {
                h.path.parent() == Some(driver) && h.path.extension().is_some_and(|e| e == "h")
            }) {
                for n in &names {
                    let pat = format!(r"(?m)^\s*#define\s+{}\s*\(", regex::escape(n));
                    if crate::regexes::cached(&pat).unwrap().is_match(&h.text) {
                        macros.insert(n.clone());
                    }
                }
            }
            for r in rust {
                let code = com.replace_all(&r.text, " ").into_owned();
                for n in &names {
                    let dp = format!(r"\bfn\s+{}\s*\(", regex::escape(n));
                    let cp = format!(r"\b{}\s*\(", regex::escape(n));
                    if !declarations.contains_key(n)
                        && crate::regexes::cached(&dp).unwrap().is_match(&r.text)
                    {
                        declarations.insert(n.clone(), r.path.to_string_lossy().into_owned());
                    }
                    let re = crate::regexes::cached(&cp).unwrap();
                    for m in re.find_iter(&code) {
                        if !crate::regexes::cached(r"\bfn\s+$")
                            .unwrap()
                            .is_match(&code[..m.start()])
                        {
                            *calls.entry(n.clone()).or_default() += 1;
                        }
                    }
                }
            }
            for n in names {
                let Some(decl) = declarations.get(&n) else {
                    continue;
                };
                if macros.contains(&n) {
                    out.push(format!("{rel}: [MACRO_AS_EXTERN] {n} is a #define in this driver's headers, never a function — the extern decl in {decl} can never resolve, so this stub is a guaranteed bugcheck on first call; implement it inline in Rust and delete the stub"));
                } else if calls.get(&n).copied().unwrap_or(0) > 0 {
                    let count = calls[&n];
                    out.push(format!("{rel}: [BUGCHECK_STUB] {n} is untranslated (bugchecks on entry) and Rust CALLS it at {count} site(s) (declared in {decl}) — reachable, so it crashes whenever that path first runs; translate it or prove the path is dead"));
                } else {
                    out.push(format!("{rel}: [ORPHAN_DECL] {n} is declared in {decl} but never called from Rust, so the stub cannot fire. The decl is what MADE the stub exist; check whether the function is already translated under another name before translating anything, then delete the decl and the stub"));
                }
            }
        }
        out.sort();
        Ok(out)
    }
}
fn nested_checkout<'a>(
    path: &std::path::Path,
    _root: &std::path::Path,
    markers: impl Iterator<Item = &'a SourceFile>,
) -> bool {
    let markers: Vec<_> = markers.collect();
    let mut p = path.parent();
    while let Some(dir) = p {
        if dir == std::path::Path::new(".") || dir.as_os_str().is_empty() {
            break;
        }
        if markers.iter().any(|m| {
            m.path.file_name().is_some_and(|n| n == ".git") && m.path.parent() == Some(dir)
        }) {
            return true;
        }
        p = dir.parent()
    }
    false
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Corpus, SourceFile};
    use std::path::PathBuf;
    #[test]
    fn live_call_is_reported() {
        let c = Corpus::new(
            PathBuf::from("."),
            vec![
                SourceFile::new(
                    "drivers/foo/rust_cohab_stub.c".into(),
                    "void Foo(void) { STUB_BUGCHECK(); }".into(),
                ),
                SourceFile::new(
                    "sdk/lib/rust/foo-rust/src/lib.rs".into(),
                    "extern \"C\" { fn Foo(); } fn call(){ unsafe { Foo(); } }".into(),
                ),
            ],
        );
        assert_eq!(BugcheckStubs.run(&c).unwrap().len(), 1);
    }
}

#[cfg(test)]
mod original_control_fixtures {
    use super::*;
    use crate::{Corpus, SourceFile};
    use std::path::PathBuf;
    const STUBS_HEADER:&str="#define FooBugCheck(A,B,C) { KeBugCheckEx(0x23, __LINE__, A, B, C); }\n#define FooSizeOf(X,OUT)   { *(OUT) = sizeof(*(X)); }\n";
    const STUBS_BAD_C:&str="#define STUB_BUGCHECK() do { KeBugCheckEx(0x23, 0xDEAD0000, 0, 0, 0); } while(0)\n\n/* A real wrapper for the FooBugCheck macro — forwards its args. NOT a stub. */\nVOID FooBugCheck(ULONG_PTR A, ULONG_PTR B, ULONG_PTR C)\n{\n    KeBugCheckEx(0x23, 0x00010000 | (ULONG)(__LINE__), A, B, C);\n}\n\n/* Genuinely untranslated, and Rust declares it => reachable crash. */\nNTSTATUS FooDismount(PVOID a) { (void)a; STUB_BUGCHECK(); return 0; }\n\n/* A MACRO stubbed as a function: the extern decl can never resolve. */\nULONG FooSizeOf(PVOID a, PVOID b) { (void)a;(void)b; STUB_BUGCHECK(); return 0; }\n\n/* Bugcheck stub that NO Rust module declares — unreachable, so not a finding. */\nNTSTATUS FooNeverCalled(PVOID a) { (void)a; STUB_BUGCHECK(); return 0; }\n\n/* Declared by Rust but never CALLED: the decl is what made this stub exist, and the\n   real function is already translated as FooOrphanReal. ORPHAN_DECL, not a crash. */\nNTSTATUS foo_orphan(PVOID a) { (void)a; STUB_BUGCHECK(); return 0; }\n";
    const STUBS_BAD_RS:&str="extern \"system\" {\n    fn FooDismount(a: *mut u8) -> i32;\n    fn FooSizeOf(a: *mut u8, b: *mut u32);\n    fn FooBugCheck(a: usize, b: usize, c: usize);\n    fn foo_orphan(a: *mut u8) -> i32;\n}\n\n// FooDismount is genuinely reachable: something calls it. This is the case that\n// separates a live crash from a dead declaration.\npub unsafe fn caller(a: *mut u8) -> i32 {\n    // A commented-out call must NOT count as a call site — fastfat keeps the whole\n    // original C body commented beside each translation:  foo_orphan(a);\n    FooDismount(a)\n}\n";
    const STUBS_GOOD_C:&str="#define STUB_BUGCHECK() do { KeBugCheckEx(0x23, 0xDEAD0000, 0, 0, 0); } while(0)\n\n/* Real wrapper — still must not be flagged. */\nVOID FooBugCheck(ULONG_PTR A, ULONG_PTR B, ULONG_PTR C)\n{\n    KeBugCheckEx(0x23, 0x00010000 | (ULONG)(__LINE__), A, B, C);\n}\n\n/* Both former stubs are now real Rust; only unreachable ones remain here. */\nNTSTATUS FooNeverCalled(PVOID a) { (void)a; STUB_BUGCHECK(); return 0; }\n";
    const STUBS_GOOD_RS:&str="extern \"system\" {\n    fn FooBugCheck(a: usize, b: usize, c: usize);\n}\n#[no_mangle]\npub unsafe extern \"system\" fn FooDismount(_a: *mut u8) -> i32 { 0 }\n";
    fn fixture(good: bool) -> Corpus {
        let (body, rs) = if good {
            (STUBS_GOOD_C, STUBS_GOOD_RS)
        } else {
            (STUBS_BAD_C, STUBS_BAD_RS)
        };
        Corpus::new(
            PathBuf::from("."),
            vec![
                SourceFile::new("drivers/foo/rust_cohab_stub.c".into(), body.into()),
                SourceFile::new("drivers/foo/foo.h".into(), STUBS_HEADER.into()),
                SourceFile::new("sdk/lib/rust/foo-rust/src/lib.rs".into(), rs.into()),
            ],
        )
    }
    #[test]
    fn original_bad_control_reports_all_three_classes() {
        let out = BugcheckStubs.run(&fixture(false)).unwrap();
        assert_eq!(out.len(), 3);
        assert!(out
            .iter()
            .any(|x| x.contains("[BUGCHECK_STUB] FooDismount")));
        assert!(out
            .iter()
            .any(|x| x.contains("[MACRO_AS_EXTERN] FooSizeOf")));
        assert!(out.iter().any(|x| x.contains("[ORPHAN_DECL] foo_orphan")));
    }
    #[test]
    fn original_good_control_is_quiet() {
        assert!(BugcheckStubs.run(&fixture(true)).unwrap().is_empty());
    }
    #[test]
    fn nested_checkout_marker_is_skipped() {
        let mut c = fixture(false);
        c.files.push(SourceFile::new(
            ".claude/worktrees/sibling/.git".into(),
            "gitdir: x".into(),
        ));
        c.files.push(SourceFile::new(
            ".claude/worktrees/sibling/drivers/foo/rust_cohab_stub.c".into(),
            STUBS_BAD_C.into(),
        ));
        assert_eq!(BugcheckStubs.run(&c).unwrap().len(), 3);
    }
}
