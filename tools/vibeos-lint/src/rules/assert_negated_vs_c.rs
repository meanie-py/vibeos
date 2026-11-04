//! A translated `ASSERT`/`NT_ASSERT` whose condition is the NEGATION of its C twin's.
//!
//! fastfat's `FatNonCachedIo` asserts the C's
//! `KeGetCurrentThread() != Vcb->VerifyThread || Status != STATUS_VERIFY_REQUIRED`; the
//! translation wrapped it in `!( ... )` at all three sites, so once the crate was armed it fired
//! on every legitimate I/O failure (an empty CD drive's STATUS_NO_MEDIA_IN_DEVICE, 2026-11-01).
//! Dead code while assertions were compiled out, which is how it survived. A Rust assert is
//! flagged when, normalized, it reads `!(<cond>)` and `<cond>` is an assert in the same-module C
//! function of the same name. `// nolint: assert-negated-vs-c (why)` on the line suppresses.
use crate::{twins, Corpus, Rule};
use rayon::prelude::*;
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

pub struct AssertNegatedVsC;
impl Rule for AssertNegatedVsC {
    fn name(&self) -> &'static str {
        "scan_assert_negated_vs_c"
    }
    fn needs_c_twins(&self) -> bool {
        true
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let index = c.c_twins();
        let files: Vec<_> = c
            .rust_files()
            .filter(|f| RUST_ASSERT.is_match(&f.text))
            .filter(|f| !SKIP.is_match(&f.path.to_string_lossy()))
            .collect();
        let mut out: Vec<String> = files
            .par_iter()
            .flat_map_iter(|f| {
                scan(&f.text, &f.path, index)
                    .into_iter()
                    .map(move |x| format!("{}:{x}", f.path.display()))
            })
            .collect();
        out.sort();
        Ok(out)
    }
}

static SKIP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/types_[a-z0-9]+\.rs$)").unwrap()
});
static RUST_ASSERT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(?:NT_ASSERT|ASSERT)!\s*\(").unwrap());
static C_ASSERT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(?:NT_ASSERT|ASSERT)\s*\(").unwrap());
static NOLINT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bnolint:[\w\s,-]*\bassert-negated-vs-c\b").unwrap());
static CAST: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bas\s+(?:\*\s*(?:mut|const)\s+)?[\w:]+(?:::<[^>]*>)?").unwrap());
static BARE_PARENS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\((\w+)\)").unwrap());
static C_CAST: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\(\s*(?:[A-Z_][A-Z0-9_]*|struct\s+\w+|\w+\s*\*+)\s*\)").unwrap());

/// The text between the `(` at `open` and its matching `)`.
fn call_arg(text: &str, open: usize) -> Option<&str> {
    let mut depth = 0i32;
    for (i, ch) in text[open..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&text[open + 1..open + i]);
                }
            }
            _ => {}
        }
    }
    None
}

/// Strip one layer of parentheses that encloses the whole expression.
fn unwrap_parens(mut s: &str) -> &str {
    loop {
        let t = s.trim();
        if t.starts_with('(') && call_arg(t, 0).map(|inner| inner.len() + 2) == Some(t.len()) {
            s = &t[1..t.len() - 1];
        } else {
            return t;
        }
    }
}

/// A comparable spelling shared by C and Rust: casts and whitespace removed, `(*p).f` and
/// `p->f` both `p.f`, `.is_null()` as `== NULL`, `TRUE`/`FALSE` as `1`/`0`.
fn normalize(expr: &str, rust: bool) -> String {
    let mut s = if rust {
        CAST.replace_all(expr, "").into_owned()
    } else {
        C_CAST.replace_all(expr, "").into_owned()
    };
    s = s
        .replace("core::ffi::c_void", "")
        .replace("->", ".")
        .replace("(*", "(")
        .replace("!.is_null()", "!= NULL");
    let s = s.replace(".is_null()", "==NULL").replace("NULL", "0").replace("TRUE", "1").replace("FALSE", "0");
    let s: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    // `(*Vcb).f` became `(Vcb).f`; drop parentheses around a bare identifier.
    let s = BARE_PARENS.replace_all(&s, "$1").into_owned();
    unwrap_parens(&s).to_owned()
}

fn scan(text: &str, path: &std::path::Path, index: &twins::CIndex) -> Vec<String> {
    let blank = twins::blank_rust(text);
    let newlines: Vec<_> = blank.match_indices('\n').map(|(at, _)| at).collect();
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    for fun in twins::rust_functions(&blank, path) {
        if !RUST_ASSERT.is_match(&fun.body) {
            continue;
        }
        let mut c_conds: HashSet<String> = HashSet::new();
        for twin in index.get(&fun.name).iter().filter(|t| twins::same_module(path, t)) {
            for m in C_ASSERT.find_iter(&twin.body) {
                if let Some(arg) = call_arg(&twin.body, m.end() - 1) {
                    c_conds.insert(normalize(arg, false));
                }
            }
        }
        if c_conds.is_empty() {
            continue;
        }
        for m in RUST_ASSERT.find_iter(&fun.body) {
            let Some(arg) = call_arg(&fun.body, m.end() - 1) else { continue };
            let n = normalize(arg, true);
            let Some(inner) = n.strip_prefix('!') else { continue };
            // Only a `!` over the whole parenthesized condition: `!a || b` negates `a` alone.
            if !inner.starts_with('(') {
                continue;
            }
            let inner = unwrap_parens(inner);
            if !c_conds.contains(inner) || c_conds.contains(&n) {
                continue;
            }
            let at = fun.body_start + m.start();
            let line = newlines.partition_point(|&nl| nl < at) + 1;
            if lines.get(line - 1).is_some_and(|l| NOLINT.is_match(l)) {
                continue;
            }
            out.push(format!(
                "{line}: assert-negated-vs-c {}: asserts !({inner}) where the C asserts ({inner})",
                fun.name
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceFile;
    use std::path::PathBuf;

    const C: &str = "NTSTATUS FatNonCachedIo(PIRP_CONTEXT IrpContext, PVCB Vcb)\n{\n    NT_ASSERT( KeGetCurrentThread() != Vcb->VerifyThread || Status != STATUS_VERIFY_REQUIRED );\n    return 0;\n}\n";

    fn run(c_path: &str, rust: &str) -> Vec<String> {
        let c = Corpus::new(
            PathBuf::from("."),
            vec![
                SourceFile::new(c_path.into(), C.into()),
                SourceFile::new("sdk/lib/rust/fastfat-rust/src/read.rs".into(), rust.into()),
            ],
        );
        AssertNegatedVsC.run(&c).unwrap()
    }

    #[test]
    fn negated_twin_assert_is_flagged() {
        let got = run(
            "drivers/filesystems/fastfat/read.c",
            "pub unsafe fn FatNonCachedIo(IrpContext: PIRP_CONTEXT, Vcb: PVCB) -> NTSTATUS {\n    NT_ASSERT!((!(KeGetCurrentThread() as *mut core::ffi::c_void != (*Vcb).VerifyThread as *mut core::ffi::c_void || Status != STATUS_VERIFY_REQUIRED)) as i32);\n    0\n}\n",
        );
        assert_eq!(got.len(), 1, "{got:?}");
        assert!(got[0].starts_with("sdk/lib/rust/fastfat-rust/src/read.rs:2: assert-negated-vs-c FatNonCachedIo"), "{got:?}");
    }

    #[test]
    fn faithful_assert_and_other_modules_pass() {
        let ok = "pub unsafe fn FatNonCachedIo(IrpContext: PIRP_CONTEXT, Vcb: PVCB) -> NTSTATUS {\n    NT_ASSERT!((KeGetCurrentThread() as *mut core::ffi::c_void != (*Vcb).VerifyThread as *mut core::ffi::c_void || Status != STATUS_VERIFY_REQUIRED) as i32);\n    0\n}\n";
        assert!(run("drivers/filesystems/fastfat/read.c", ok).is_empty());
        let negated = "pub unsafe fn FatNonCachedIo(IrpContext: PIRP_CONTEXT, Vcb: PVCB) -> NTSTATUS {\n    NT_ASSERT!((!(KeGetCurrentThread() != (*Vcb).VerifyThread || Status != STATUS_VERIFY_REQUIRED)) as i32);\n    0\n}\n";
        assert!(run("dll/win32/shell32/other.c", negated).is_empty());
    }
}
