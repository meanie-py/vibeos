//! Detect FORCEINLINE assertions lost in Rust translations.
use crate::{Corpus, Rule, SourceSet};
use std::collections::HashMap;

pub struct DroppedAsserts;
static INPUTS: &[SourceSet] = &[SourceSet::tree(
    "dropped_assert_headers",
    &["ntoskrnl/include/internal", "ntoskrnl/mm/ARM3"],
    &["h"],
    &[],
)];
impl Rule for DroppedAsserts {
    fn name(&self) -> &'static str {
        "scan_dropped_asserts"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut cs: HashMap<String, (String, usize, usize)> = HashMap::new();
        for f in corpus.sources("dropped_assert_headers") {
            let p = f.path.to_string_lossy();
            if !(p.starts_with("ntoskrnl/include/internal/") || p.starts_with("ntoskrnl/mm/ARM3/"))
                || !p.ends_with(".h")
            {
                continue;
            }
            let code = crate::lexical::strip_noise(&f.text);
            let re =
                crate::regexes::cached(r"\bFORCEINLINE\b[^;{()]*?\b([A-Za-z_]\w*)\s*\(").unwrap();
            let ar = crate::regexes::cached(
                r"\b(?:NT_ASSERT|ASSERT|PAGED_CODE|debug_assert|assert)\s*[!(]",
            )
            .unwrap();
            for c in re.captures_iter(&code) {
                let m = c.get(0).unwrap();
                let open = code[m.end() - 1..].find('{').map(|x| m.end() - 1 + x);
                let Some(open) = open else { continue };
                if code[m.end() - 1..open].contains(';') {
                    continue;
                }
                if let Some(end) = brace_end(&code, open) {
                    let n = ar.find_iter(&code[open..end]).count();
                    if n > 0 {
                        let line = masked_line(&code, m.start());
                        let e = cs
                            .entry(c[1].to_owned())
                            .or_insert((p.to_string(), line, n));
                        if n > e.2 {
                            *e = (p.to_string(), line, n);
                        }
                    }
                }
            }
        }
        let rf = crate::regexes::cached(r"\bfn\s+([A-Za-z_]\w*)\s*[(<]").unwrap();
        let ar =
            crate::regexes::cached(r"\b(?:NT_ASSERT|ASSERT|PAGED_CODE|debug_assert|assert)\s*[!(]")
                .unwrap();
        let mut rs: HashMap<String, Vec<(String, usize, usize)>> = HashMap::new();
        for f in corpus.rust_files() {
            if f.path.components().any(|p| p.as_os_str() == "target") {
                continue;
            }
            let code = f.rust_noise();
            for c in rf.captures_iter(code) {
                let m = c.get(0).unwrap();
                let op = code[m.end() - 1..].find('{').map(|x| m.end() - 1 + x);
                let Some(op) = op else { continue };
                if code[m.end() - 1..op].contains(';') {
                    continue;
                }
                if let Some(end) = brace_end(code, op) {
                    let n = ar.find_iter(&code[op..end]).count();
                    rs.entry(c[1].into()).or_default().push((
                        f.path.to_string_lossy().into_owned(),
                        masked_line(code, m.start()),
                        n,
                    ));
                }
            }
        }
        let mut rows = Vec::new();
        for (name, (p, l, cn)) in cs {
            let mut rustsites = rs.get(&name).cloned().unwrap_or_default();
            let rn = rustsites.iter().map(|x| x.2).max().unwrap_or(0);
            rustsites.sort();
            let where_ = rustsites
                .iter()
                .take(3)
                .map(|(p, l, _)| format!("{p}:{l}"))
                .collect::<Vec<_>>()
                .join(", ");
            if rn < cn && !rustsites.is_empty() {
                rows.push((
                    p.clone(),
                    l,
                    format!("{p}:{l}: {name}: C has {cn} assert(s), Rust has {rn} ({where_})"),
                ));
            }
        }
        rows.sort_by(|a, b| (&a.0, a.1, &a.2).cmp(&(&b.0, b.1, &b.2)));
        Ok(rows.into_iter().map(|x| x.2).collect())
    }
}
// strip_noise preserves newline characters but replaces each non-newline character
// with one ASCII space. A byte offset in that masked string differs from the same
// character's byte offset in the original whenever a removed comment/literal contains
// non-ASCII text, so compute its line in the exact text the regex searched.
fn masked_line(text: &str, byte: usize) -> usize {
    text.as_bytes()[..byte]
        .iter()
        .filter(|&&b| b == b'\n')
        .count()
        + 1
}
fn brace_end(s: &str, open: usize) -> Option<usize> {
    let mut d = 0;
    for (i, b) in s.as_bytes().iter().enumerate().skip(open) {
        if *b == b'{' {
            d += 1
        } else if *b == b'}' {
            d -= 1;
            if d == 0 {
                return Some(i + 1);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Corpus, SourceFile};
    use std::path::PathBuf;
    fn c(files: &[(&str, &str)]) -> Corpus {
        Corpus::new(
            PathBuf::from("."),
            files
                .iter()
                .map(|(p, t)| SourceFile::new(PathBuf::from(p), (*t).into()))
                .collect(),
        )
    }
    const C_GUARDED:&str="\nFORCEINLINE\nVOID\nMI_MAKE_HARDWARE_PTE_USER(IN PMMPTE NewPte, IN ULONG_PTR ProtectionMask)\n{\n ASSERT(ProtectionMask & MM_PROTECT_ACCESS);\n ASSERT((ProtectionMask & MM_PROTECT_SPECIAL) != MM_GUARDPAGE);\n NewPte->u.Hard.Valid = TRUE;\n}\n";
    #[test]
    fn fully_dropped_is_reported() {
        let x = c(&[
            ("ntoskrnl/include/internal/miarm.h", C_GUARDED),
            (
                "sdk/lib/rust/bridge.rs",
                "pub unsafe fn MI_MAKE_HARDWARE_PTE_USER(p: usize) { set_valid(p); }",
            ),
        ]);
        let r = DroppedAsserts.run(&x).unwrap();
        assert_eq!(r.len(), 1);
        assert!(r[0].contains("MI_MAKE_HARDWARE_PTE_USER: C has 2 assert(s), Rust has 0"));
    }
    #[test]
    fn partial_drop_is_reported() {
        let x = c(&[
            ("ntoskrnl/include/internal/miarm.h", C_GUARDED),
            (
                "sdk/lib/rust/bridge.rs",
                "pub unsafe fn MI_MAKE_HARDWARE_PTE_USER(p: usize) { ASSERT!(a); set_valid(p); }",
            ),
        ]);
        assert!(DroppedAsserts.run(&x).unwrap()[0].contains("C has 2 assert(s), Rust has 1"));
    }
    #[test]
    fn parity_is_silent() {
        let x = c(&[
            ("ntoskrnl/include/internal/miarm.h", C_GUARDED),
            (
                "sdk/lib/rust/bridge.rs",
                "pub unsafe fn MI_MAKE_HARDWARE_PTE_USER(p: usize) { ASSERT!(a); ASSERT!(b); }",
            ),
        ]);
        assert!(DroppedAsserts.run(&x).unwrap().is_empty());
    }
    #[test]
    fn quoted_rust_assert_is_not_counted() {
        let x=c(&[("ntoskrnl/include/internal/miarm.h",C_GUARDED),("sdk/lib/rust/bridge.rs","pub unsafe fn MI_MAKE_HARDWARE_PTE_USER(p: usize) { // ASSERT(a)\n /* ASSERT(b) */ work(); }")]);
        assert!(DroppedAsserts.run(&x).unwrap()[0].contains("Rust has 0"));
    }
    #[test]
    fn commented_c_assert_is_not_counted() {
        let x = c(&[
            (
                "ntoskrnl/mm/ARM3/a.h",
                "FORCEINLINE VOID MiFoo(VOID) { /* ASSERT(a); */ // ASSERT(b)\n work(); }",
            ),
            ("sdk/lib/rust/foo.rs", "pub fn MiFoo() { work(); }"),
        ]);
        assert!(DroppedAsserts.run(&x).unwrap().is_empty());
    }
    #[test]
    fn literals_do_not_count() {
        let x=c(&[("ntoskrnl/include/internal/a.h",C_GUARDED),("sdk/lib/rust/foo.rs","pub fn MI_MAKE_HARDWARE_PTE_USER() { ASSERT(a); DbgPrint(\"ASSERT(x)\"); ASSERT(b); }")]);
        assert!(DroppedAsserts.run(&x).unwrap().is_empty());
    }
    #[test]
    fn preserves_c_line_numbers() {
        let header = format!("/*\n a\n multi\n line\n banner\n */\n{}", C_GUARDED);
        let x = c(&[
            ("ntoskrnl/include/internal/a.h", &header),
            ("sdk/lib/rust/foo.rs", "fn MI_MAKE_HARDWARE_PTE_USER() {}"),
        ]);
        let r = DroppedAsserts.run(&x).unwrap();
        assert!(r[0].starts_with("ntoskrnl/include/internal/a.h:8:"));
    }
    #[test]
    fn prototype_is_not_a_definition() {
        let x = c(&[
            (
                "ntoskrnl/include/internal/a.h",
                "FORCEINLINE VOID MiFoo(VOID);",
            ),
            ("sdk/lib/rust/foo.rs", "pub fn MiFoo() { work(); }"),
        ]);
        assert!(DroppedAsserts.run(&x).unwrap().is_empty());
    }
    #[test]
    fn unpaired_c_fn_is_ignored() {
        let x = c(&[
            ("ntoskrnl/include/internal/a.h", C_GUARDED),
            ("sdk/lib/rust/foo.rs", "pub fn unrelated() {}"),
        ]);
        assert!(DroppedAsserts.run(&x).unwrap().is_empty());
    }
    #[test]
    fn best_cfg_variant_wins() {
        let x=c(&[("ntoskrnl/include/internal/a.h",C_GUARDED),("sdk/lib/rust/foo.rs","#[cfg(a)] fn MI_MAKE_HARDWARE_PTE_USER(){ work(); } #[cfg(b)] fn MI_MAKE_HARDWARE_PTE_USER(){ ASSERT(a); ASSERT(b); }")]);
        assert!(DroppedAsserts.run(&x).unwrap().is_empty());
    }
    #[test]
    fn cargo_target_copy_does_not_mask_source_drop() {
        let x = c(&[
            ("ntoskrnl/include/internal/miarm.h", C_GUARDED),
            (
                "sdk/lib/rust/target/stale.rs",
                "fn MI_MAKE_HARDWARE_PTE_USER(){ ASSERT(a); ASSERT(b); }",
            ),
            (
                "sdk/lib/rust/bridge.rs",
                "fn MI_MAKE_HARDWARE_PTE_USER(){ work(); }",
            ),
        ]);
        assert_eq!(DroppedAsserts.run(&x).unwrap().len(), 1);
    }
    #[test]
    fn unicode_in_removed_comments_preserves_original_source_lines() {
        let x = c(&[
            (
                "ntoskrnl/include/internal/a.h",
                "/* π banner */\nFORCEINLINE VOID MiFoo(VOID) { ASSERT(a); }\n",
            ),
            (
                "sdk/lib/rust/foo.rs",
                "// π banner\npub fn MiFoo() { work(); }\n",
            ),
        ]);
        let got = DroppedAsserts.run(&x).unwrap();
        assert_eq!(got, vec![
            "ntoskrnl/include/internal/a.h:2: MiFoo: C has 1 assert(s), Rust has 0 (sdk/lib/rust/foo.rs:2)"
        ]);
    }
}

#[cfg(test)]
mod original_control {
    use super::*;
    use crate::SourceFile;
    use std::path::PathBuf;
    fn c(files: &[(&str, &str)]) -> Corpus {
        Corpus::new(
            PathBuf::from("."),
            files
                .iter()
                .map(|(p, t)| SourceFile::new(PathBuf::from(p), (*t).into()))
                .collect(),
        )
    }
    const DROPPED_ASSERT_H:&str="\nFORCEINLINE\nVOID\nMI_MAKE_HARDWARE_PTE_USER(IN PMMPTE NewPte, IN ULONG_PTR ProtectionMask)\n{\n    /* Check that we are not setting valid a page that should not be */\n    ASSERT(ProtectionMask & MM_PROTECT_ACCESS);\n    ASSERT((ProtectionMask & MM_PROTECT_SPECIAL) != MM_GUARDPAGE);\n    NewPte->u.Hard.Valid = TRUE;\n}\n";
    const DROPPED_ASSERT_BAD:&str="\npub unsafe fn MI_MAKE_HARDWARE_PTE_USER(new_pte: *mut MMPTE, mask: usize) {\n    // ASSERT(ProtectionMask & MM_PROTECT_ACCESS);\n    (*new_pte).set_valid(true);\n}\n";
    const DROPPED_ASSERT_GOOD:&str="\n#[cfg(target_arch = \"aarch64\")]\npub unsafe fn MI_MAKE_HARDWARE_PTE_USER(new_pte: *mut MMPTE, mask: usize) {\n    ASSERT!(mask & MM_PROTECT_ACCESS != 0);\n    ASSERT!(mask & MM_PROTECT_SPECIAL != MM_GUARDPAGE);\n    (*new_pte).set_valid(true);\n}\n\n#[cfg(not(target_arch = \"aarch64\"))]\npub unsafe fn MI_MAKE_HARDWARE_PTE_USER(new_pte: *mut MMPTE, mask: usize) {\n    (*new_pte).set_valid(true);\n}\n";
    #[test]
    fn exact_bad_good_scanner_control() {
        let bad = c(&[
            ("ntoskrnl/mm/ARM3/miarm.h", DROPPED_ASSERT_H),
            ("sdk/lib/rust/bridge.rs", DROPPED_ASSERT_BAD),
        ]);
        let good = c(&[
            ("ntoskrnl/mm/ARM3/miarm.h", DROPPED_ASSERT_H),
            ("sdk/lib/rust/bridge.rs", DROPPED_ASSERT_GOOD),
        ]);
        assert_eq!(DroppedAsserts.run(&bad).unwrap().len(), 1);
        assert!(DroppedAsserts.run(&good).unwrap().is_empty());
    }
}
