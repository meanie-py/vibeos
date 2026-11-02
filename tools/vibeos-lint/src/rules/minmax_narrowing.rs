//! `min`/`max` operands cast NARROWER than their declared type before the compare.
//!
//! C compares `min(a, b)` at the wider operand's width; a translation that writes
//! `min(len as ULONG, K)` for a LONGLONG `len` truncates first, so a 4 GiB length
//! becomes 0 (CcZeroData spun forever on exactly that). The fix is always the same
//! shape: compare at the wide type, then narrow the RESULT.
//!
//! Widths come from declarations the scanner can see without a type checker: the
//! enclosing fn's params and `let`s (annotated, `.. as T`, or a resolvable initializer)
//! and struct fields across the corpus (`pub Name: T`, accepted only when every
//! declaration of that field name agrees on width). Anything unresolved is skipped,
//! so this under-reports by design; it never guesses a width.
use crate::{Corpus, Rule};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

pub struct MinmaxNarrowing;
impl Rule for MinmaxNarrowing {
    fn name(&self) -> &'static str {
        "scan_minmax_narrowing"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let files: Vec<_> = corpus.rust_files().collect();
        let masked: Vec<String> = files.iter().map(|f| mask(&f.text)).collect();
        let crates: Vec<PathBuf> = files.iter().map(|f| crate_of(&corpus.root, &f.path)).collect();
        let mut global = Fields::new();
        let mut by_crate: BTreeMap<&PathBuf, Fields> = BTreeMap::new();
        let mut by_file = Vec::new();
        for (text, krate) in masked.iter().zip(&crates) {
            let mut own = Fields::new();
            collect_fields(text, &mut own);
            collect_fields(text, &mut global);
            collect_fields(text, by_crate.entry(krate).or_default());
            by_file.push(own);
        }
        let mut out = Vec::new();
        for (i, (file, text)) in files.iter().zip(&masked).enumerate() {
            let scopes = [&by_file[i], &by_crate[&crates[i]], &global];
            for (line, what) in scan_text(&file.text, text, &scopes) {
                out.push(format!("{}:{line}: minmax-narrowing {what}", file.path.display()));
            }
        }
        out.sort();
        Ok(out)
    }
}

/// Bits per type name; `PTR` is pointer-sized, which is 64 on two of our three targets.
const PTR: u32 = 64;
fn width(ty: &str) -> Option<u32> {
    let ty = ty.rsplit("::").next().unwrap_or(ty);
    Some(match ty {
        "u8" | "i8" | "UCHAR" | "BYTE" | "CHAR" | "CCHAR" | "SCHAR" | "BOOLEAN" | "c_char"
        | "c_uchar" | "c_schar" => 8,
        "u16" | "i16" | "USHORT" | "WORD" | "WCHAR" | "SHORT" | "CSHORT" | "c_short"
        | "c_ushort" => 16,
        "u32" | "i32" | "c_int" | "c_uint" | "INT" | "UINT" | "LONG" | "ULONG" | "DWORD"
        | "BOOL" | "c_long" | "c_ulong" | "NTSTATUS" | "HRESULT" | "ULONG32" | "LONG32"
        | "DWORD32" | "INT32" | "UINT32" => 32,
        "u64" | "i64" | "LONGLONG" | "ULONGLONG" | "c_longlong" | "c_ulonglong" | "ULONG64"
        | "LONG64" | "DWORD64" | "DWORDLONG" | "INT64" | "UINT64" => 64,
        "usize" | "isize" | "SIZE_T" | "SSIZE_T" | "ULONG_PTR" | "LONG_PTR" | "DWORD_PTR"
        | "UINT_PTR" | "INT_PTR" => PTR,
        _ => return None,
    })
}

type Fields = BTreeMap<String, BTreeSet<Option<u32>>>;
static FIELD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bpub\s+(?:r#)?([A-Za-z_]\w*)\s*:\s*([\w:]+)\s*[,}]").unwrap());
fn collect_fields(masked: &str, fields: &mut Fields) {
    for m in FIELD.captures_iter(masked) {
        fields.entry(m[1].into()).or_default().insert(width(&m[2]));
    }
}
/// Nearest scope that declares the field wins: this file, then its crate, then the corpus.
/// A scope that declares it at two widths is ambiguous and answers None.
fn field_width(scopes: &[&Fields], name: &str) -> Option<u32> {
    let set = scopes.iter().find_map(|f| f.get(name))?;
    match set.iter().collect::<Vec<_>>()[..] {
        [Some(w)] => Some(*w),
        _ => None,
    }
}
fn crate_of(root: &Path, file: &Path) -> PathBuf {
    for parent in file.ancestors().skip(1) {
        if root.join(parent).join("Cargo.toml").is_file() {
            return parent.into();
        }
    }
    file.parent().unwrap_or(Path::new("")).into()
}

/// Blank comments and string/char literals, keeping byte offsets and newlines.
fn mask(text: &str) -> String {
    let b = text.as_bytes();
    let mut out = b.to_vec();
    let mut i = 0;
    let blank = |out: &mut Vec<u8>, from: usize, to: usize| {
        for c in &mut out[from..to] {
            if *c != b'\n' {
                *c = b' ';
            }
        }
    };
    while i < b.len() {
        if b[i..].starts_with(b"//") {
            let end = b[i..].iter().position(|&c| c == b'\n').map_or(b.len(), |p| i + p);
            blank(&mut out, i, end);
            i = end;
        } else if b[i..].starts_with(b"/*") {
            let end = text[i + 2..].find("*/").map_or(b.len(), |p| i + 2 + p + 2);
            blank(&mut out, i, end);
            i = end;
        } else if b[i] == b'"' {
            let mut j = i + 1;
            while j < b.len() && b[j] != b'"' {
                j += if b[j] == b'\\' { 2 } else { 1 };
            }
            let end = (j + 1).min(b.len());
            blank(&mut out, i + 1, end.saturating_sub(1).max(i + 1));
            i = end;
        } else if b[i] == b'\'' && i + 2 < b.len() && (b[i + 2] == b'\'' || b[i + 1] == b'\\') {
            // char literal ('x' or '\n'); lifetimes ('a) fall through untouched
            let end = b[i + 1..].iter().position(|&c| c == b'\'').map_or(b.len(), |p| i + 1 + p + 1);
            blank(&mut out, i + 1, end.saturating_sub(1).max(i + 1));
            i = end;
        } else {
            i += 1;
        }
    }
    String::from_utf8(out).unwrap_or_default()
}

static CALL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:\b(?:min|max)(?:_c)?\s*!?\s*\(|\.(?:min|max)\s*\()").unwrap()
});
static NOLINT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"nolint:\s*minmax-narrowing\b").unwrap());

fn scan_text(text: &str, masked: &str, fields: &[&Fields]) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for m in CALL.find_iter(masked) {
        let open = m.end() - 1;
        let Some(close) = matching(masked, open) else { continue };
        let mut operands: Vec<&str> = split_args(&masked[open + 1..close]);
        if m.as_str().starts_with('.') {
            if let Some(recv) = receiver(masked, m.start()) {
                operands.push(recv);
            }
        }
        if operands.len() != 2 && !m.as_str().starts_with('.') {
            continue;
        }
        let line = masked[..m.start()].bytes().filter(|&c| c == b'\n').count() + 1;
        let last = line + masked[m.start()..close].bytes().filter(|&c| c == b'\n').count();
        // a nolint anywhere on the lines the call spans (it may sit beside the operand)
        if text.lines().skip(line - 1).take(last - line + 1).any(|l| NOLINT.is_match(l)) {
            continue;
        }
        for op in operands {
            let Some((inner, ty)) = split_cast(op) else { continue };
            let Some(to) = width(ty) else { continue };
            let Some(from) = expr_width(inner, masked, m.start(), fields, 0) else { continue };
            if from > to {
                out.push((line, format!("`{} as {ty}` narrows a {from}-bit value before the compare; compare wide, then narrow the result", squash(inner))));
            }
        }
    }
    out
}

fn squash(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn matching(s: &str, open: usize) -> Option<usize> {
    let b = s.as_bytes();
    let mut depth = 0i32;
    for (i, &c) in b.iter().enumerate().skip(open) {
        match c {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

fn split_args(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let (mut depth, mut start) = (0i32, 0);
    for (i, c) in s.bytes().enumerate() {
        match c {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b',' if depth == 0 => {
                out.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if !s[start..].trim().is_empty() {
        out.push(&s[start..]);
    }
    out
}

/// The parenthesized receiver of `.min(`: only `( .. ).min(` is resolvable.
fn receiver(s: &str, dot: usize) -> Option<&str> {
    let b = s.as_bytes();
    if dot == 0 || b[dot - 1] != b')' {
        return None;
    }
    let mut depth = 0i32;
    for i in (0..dot).rev() {
        match b[i] {
            b')' | b']' => depth += 1,
            b'(' | b'[' => {
                depth -= 1;
                if depth == 0 {
                    // `foo(..)` is a call, not a parenthesized expression
                    let prev = s[..i].trim_end();
                    if prev.ends_with(|c: char| c.is_alphanumeric() || c == '_' || c == '!') {
                        return None;
                    }
                    return Some(&s[i + 1..dot - 1]);
                }
            }
            _ => {}
        }
    }
    None
}

static CAST_TAIL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s+as\s+([\w:]+)\s*$").unwrap());
/// `<inner> as T` at the top level of an operand (outer parens stripped).
fn split_cast(op: &str) -> Option<(&str, &str)> {
    let op = strip_parens(op.trim());
    let m = CAST_TAIL.captures(op)?;
    let inner = &op[..m.get(0).unwrap().start()];
    // the cast must bind the whole operand, not the last term of `a + b as T`
    if top_level_binop(inner) {
        return None;
    }
    Some((strip_parens(inner.trim()), m.get(1).unwrap().as_str()))
}

fn strip_parens(mut s: &str) -> &str {
    while s.starts_with('(') && matching(s, 0) == Some(s.len() - 1) {
        s = s[1..s.len() - 1].trim();
    }
    s
}

fn top_level_binop(s: &str) -> bool {
    !split_binop(s).is_empty()
}
/// Leaves of a top-level `+ - * / % & | ^ << >>` chain; empty if there is none.
fn split_binop(s: &str) -> Vec<&str> {
    let b = s.as_bytes();
    let (mut depth, mut start, mut out) = (0i32, 0, Vec::new());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b'-' if depth == 0 && b.get(i + 1) == Some(&b'>') => i += 1,
            b'+' | b'-' | b'*' | b'/' | b'%' | b'&' | b'|' | b'^' if depth == 0 => {
                // unary `-x`, `*p`, `&x` at an operand start are not binary
                if s[start..i].trim().is_empty() {
                    i += 1;
                    continue;
                }
                out.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    if out.is_empty() {
        return out;
    }
    out.push(&s[start..]);
    out
}

static IDENT_ONLY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z_]\w*$").unwrap());
static FIELD_TAIL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\.\s*(?:r#)?([A-Za-z_]\w*)\s*$").unwrap());
static LITERAL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^-?(?:0x[0-9a-fA-F_]+|\d[\d_]*)(?:[iu](?:8|16|32|64|size))?$").unwrap());

/// Declared width of an expression, or None when any part is unknown.
fn expr_width(e: &str, masked: &str, at: usize, fields: &[&Fields], depth: u8) -> Option<u32> {
    let e = strip_parens(e.trim());
    let leaves = split_binop(e);
    if !leaves.is_empty() {
        let mut best: Option<u32> = None;
        for leaf in leaves {
            let leaf = strip_parens(leaf.trim());
            if LITERAL.is_match(leaf) {
                continue;
            }
            let w = expr_width(leaf, masked, at, fields, depth)?;
            best = Some(best.map_or(w, |b| b.max(w)));
        }
        return best;
    }
    if let Some((_, ty)) = split_cast(e) {
        return width(ty);
    }
    if let Some(m) = FIELD_TAIL.captures(e) {
        return field_width(fields, &m[1]);
    }
    if IDENT_ONLY.is_match(e) {
        return local_width(e, masked, at, fields, depth);
    }
    None
}

/// The last declaration of `name` before `at`, inside the enclosing fn.
fn local_width(name: &str, masked: &str, at: usize, fields: &[&Fields], depth: u8) -> Option<u32> {
    if depth > 2 {
        return None;
    }
    static FN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bfn\s+\w+").unwrap());
    let start = FN.find_iter(&masked[..at]).last()?.start();
    let body = &masked[start..at];
    let n = regex::escape(name);
    let typed = Regex::new(&format!(r"(?:\blet\s+(?:mut\s+)?|[(,]\s*(?:mut\s+)?){n}\s*:\s*([\w:]+)")).ok()?;
    let init = Regex::new(&format!(r"\blet\s+(?:mut\s+)?{n}\s*=\s*([^;]+);")).ok()?;
    let a = typed.captures_iter(body).last().map(|c| (c.get(0).unwrap().start(), c[1].to_owned()));
    let b = init.captures_iter(body).last().map(|c| (c.get(0).unwrap().start(), c[1].to_owned()));
    match (a, b) {
        (Some((pa, ty)), Some((pb, _))) if pa >= pb => width(&ty),
        (Some((_, ty)), None) => width(&ty),
        (_, Some((pb, e))) => expr_width(&e, masked, start + pb, fields, depth + 1),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn scan(text: &str) -> Vec<(usize, String)> {
        let m = mask(text);
        let mut fields = Fields::new();
        collect_fields(&m, &mut fields);
        scan_text(text, &m, &[&fields])
    }
    #[test]
    fn today_s_bugs_flag() {
        // CcZeroData: LONGLONG length narrowed before min
        let cc = "pub struct LARGE_INTEGER { pub QuadPart: LONGLONG }\nfn zero(e: *const LARGE_INTEGER, s: *const LARGE_INTEGER) {\n    let mut length = (*e).QuadPart - (*s).QuadPart;\n    let c = core::cmp::min(length as ULONG, MAX_ZERO_LENGTH);\n}\n";
        assert_eq!(scan(cc).len(), 1, "{:?}", scan(cc));
        // freeldr hwide: ULONG field narrowed to u16
        let hw = "pub struct R { pub DataTransferLength: ULONG, }\nfn f(r: *mut R) { let b: USHORT = min(((*r).DataTransferLength) as u16 , 0xFFFE); }\n";
        assert_eq!(scan(hw).len(), 1);
        // ole32: ULONGLONG difference narrowed before min
        let ole = "pub struct U { pub QuadPart: ULONGLONG }\nfn f(a: U, b: U) { let n = min(4096, (a.QuadPart - b.QuadPart) as ULONG); }\n";
        assert_eq!(scan(ole).len(), 1);
        // method form with a parenthesized receiver
        let m = "fn f(x: u64) { let y = (x as u32).min(7); }\n";
        assert_eq!(scan(m).len(), 1);
    }
    #[test]
    fn the_fixed_forms_and_widening_pass() {
        let ok = "pub struct LARGE_INTEGER { pub QuadPart: LONGLONG }\nfn zero(e: *const LARGE_INTEGER) {\n    let length = (*e).QuadPart;\n    let c = core::cmp::min(length, MAX_ZERO_LENGTH as LONGLONG) as ULONG;\n    let d = min(length as i64, 4);\n    let w: u16 = 3; let e2 = min(w as u32, 9);\n}\n";
        assert!(scan(ok).is_empty(), "{:?}", scan(ok));
    }
    #[test]
    fn unknown_ambiguous_and_nolint_skip() {
        // a field whose width differs across structs is never guessed
        let amb = "pub struct A { pub Len: u64 }\npub struct B { pub Len: u16 }\nfn f(a: A) { let x = min(a.Len as u32, 5); }\n";
        assert!(scan(amb).is_empty());
        // ...but a declaration closer to the use settles it
        let near = Fields::from([("Len".to_owned(), BTreeSet::from([Some(32)]))]);
        let far = Fields::from([("Len".to_owned(), BTreeSet::from([Some(8), Some(32)]))]);
        let t = "fn f(a: A) { let x = min(a.Len as u16, 5); }\n";
        assert_eq!(scan_text(t, &mask(t), &[&near, &far]).len(), 1);
        assert!(scan_text(t, &mask(t), &[&far]).is_empty());
        let unk = "fn f() { let x = min(foo() as u32, 5); let y = min(q as u8, 1); }\n";
        assert!(scan(unk).is_empty());
        let nl = "fn f(x: u64) { let y = min(x as u32, 5); // nolint: minmax-narrowing\n}\n";
        assert!(scan(nl).is_empty());
        let span = "fn f(x: u64) { let y = min(\n    x as u32, // nolint: minmax-narrowing\n    5); }\n";
        assert!(scan(span).is_empty());
        let cm = "fn f(x: u64) { // min(x as u32, 5)\n}\n";
        assert!(scan(cm).is_empty());
    }
}
