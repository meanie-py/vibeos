//! `x < 0 as DWORD`: an unsigned value compared against zero, so the test is constant.
//!
//! C's `FAILED(hr)` is `((HRESULT)(hr)) < 0`, and Wine stores HRESULTs in a `DWORD` often
//! enough that a translation keeps the variable unsigned and drops the cast. shlwapi's
//! `UrlGetPartA` shipped `if ret < 0 as DWORD`, never true, so a failed `UrlGetPartW` fell
//! through and wrote a size (82 extra failures in Wine's url suite against the genuine C).
//! rustc's `unused_comparisons` does not see it: the `0 as DWORD` cast hides the literal.
//! The shape is decidable from the text alone: `< 0 as U` (and `0 as U > x`) is false for
//! every left side, so the branch it guards is dead. Only the always-FALSE forms are flagged:
//! the always-true `>= 0 as U` / `0 as U <= x` are lower bounds of range checks the C writes
//! the same way (autochk's `0 <= i && i <= 'Z' - 'A'`), harmless and not worth a baseline.
//! Fix: compare the signed type the C meant (`(ret as HRESULT) < 0`).
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::OnceLock;

const UNSIGNED: &str = "DWORD|UINT|ULONG|SIZE_T|ULONG_PTR|DWORD_PTR|UINT_PTR|WORD|USHORT|BYTE|\
                        UCHAR|ULONGLONG|DWORD64|ULONG64|WPARAM|u8|u16|u32|u64|u128|usize";

fn skip(path: &str) -> bool {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/island_[a-z0-9_]+\.rs$|/types_[a-z0-9_]+\.rs$)")
            .unwrap()
    })
    .is_match(path)
}

pub struct UnsignedVsZero;
impl Rule for UnsignedVsZero {
    fn name(&self) -> &'static str {
        "scan_unsigned_vs_zero"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        Ok(corpus
            .rust_files()
            .filter(|f| !skip(&f.path.to_string_lossy()))
            .flat_map(|f| {
                scan_text(&f.text)
                    .into_iter()
                    .map(|(n, expr)| format!("{}:{n}: unsigned-vs-zero {expr}", f.path.display()))
                    .collect::<Vec<_>>()
            })
            .collect())
    }
}

fn scan_text(text: &str) -> Vec<(usize, String)> {
    static COMMENTS: OnceLock<Regex> = OnceLock::new();
    static CMP: OnceLock<Regex> = OnceLock::new();
    let code = COMMENTS
        .get_or_init(|| Regex::new(r"(?s)//[^\n]*|/\*.*?\*/").unwrap())
        .replace_all(text, |caps: &regex::Captures<'_>| {
            caps[0]
                .chars()
                .map(|c| if c == '\n' { '\n' } else { ' ' })
                .collect::<String>()
        });
    // `<` before the zero, or the zero first with `>`. The guards (`[^<>=-]` before the
    // operator, `[^>=]` after `>`) keep `<<`, `>>`, `<=`, `>=`, `->` and `=>` out.
    let re = CMP.get_or_init(|| {
        Regex::new(&format!(
            r"(?:[^<>=\-]<\s*\(?\s*0\s+as\s+(?:{u})\b)|(?:\b0\s+as\s+(?:{u})\s*\)?\s*>[^>=])",
            u = UNSIGNED
        ))
        .unwrap()
    });
    re.find_iter(&code)
        .map(|m| {
            let at = m.start();
            let line_no = code[..at].bytes().filter(|b| *b == b'\n').count() + 1;
            let line = code.lines().nth(line_no - 1).unwrap_or("").trim().to_owned();
            (line_no, line)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fires_on_the_shapes() {
        let text = "fn f(ret: DWORD, n: u32) {\n    if ret < 0 as DWORD { return; }\n    if 0 as UINT > n {}\n    if (n < (0 as usize)) {}\n}\n";
        let hits: Vec<_> = scan_text(text).into_iter().map(|(n, _)| n).collect();
        assert_eq!(hits, vec![2, 3, 4]);
    }
    #[test]
    fn quiet_on_signed_shifts_and_comments() {
        let text = "fn f(ret: DWORD) {\n    if (ret as HRESULT) < 0 {}\n    if (ret as i32) < 0 as i32 {}\n    let m = 1u32 << 0 as u32;\n    let s = x >> 0 as u32;\n    if n > 0 as DWORD {}\n    if n == 0 as DWORD {}\n    // if ret < 0 as DWORD {\n    if n >= 0 as LONG {}\n    if n >= 0 as u32 {}\n    if (0 as ULONG) <= i && i <= 25 {}\n}\n";
        assert_eq!(scan_text(text), vec![]);
    }
}
