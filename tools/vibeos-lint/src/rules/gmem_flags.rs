//! Global/Local allocation calls supplied foreign flag bits or HEAP_ constants.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::LazyLock;
pub struct GmemFlags;
impl Rule for GmemFlags {
    fn name(&self) -> &'static str {
        "scan_gmem_flags"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.rust_files() {
            for (line, flag) in scan_text(&file.text)? {
                let diagnostic = format!(
                    "{}:{line}: Global/LocalAlloc flags `{flag}` are not GMEM_/LMEM_ flags",
                    file.path.display()
                );
                // Python prints the entire flag expression, including line breaks.
                // The caller's finding oracle splits and rstrips that stdout.
                out.extend(
                    diagnostic
                        .lines()
                        .filter(|line| !line.trim().is_empty())
                        .map(|line| line.trim_end().to_owned()),
                );
            }
        }
        Ok(out)
    }
}
static CALL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(?:Global|Local)(Alloc|ReAlloc)\s*\(").unwrap());
static LITERAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(0x[0-9a-fA-F_]+|\d[\d_]*)(?:u16|u32|i32|usize)?(?:\s+as\s+\w+)?$").unwrap()
});
static DECL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bfn\s+$").unwrap());
static COMMENTS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/|//[^\n]*").unwrap());
fn args(text: &str, start: usize) -> Option<Vec<&str>> {
    let mut depth: isize = 0;
    let mut argument_start = start;
    let mut out = Vec::new();
    for (offset, byte) in text.as_bytes()[start..].iter().enumerate() {
        let at = start + offset;
        match byte {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                if depth == 0 {
                    out.push(text[argument_start..at].trim());
                    return Some(out);
                }
                depth -= 1;
            }
            b',' if depth == 0 => {
                out.push(text[argument_start..at].trim());
                argument_start = at + 1;
            }
            _ => {}
        }
    }
    None
}
fn low_bits(raw: &str) -> Result<u16, String> {
    let raw = raw.replace('_', "");
    let (digits, radix) = if let Some(hex) = raw.strip_prefix("0x") {
        (hex, 16)
    } else {
        if raw.starts_with('0') && raw.chars().any(|c| c != '0') {
            return Err(format!("invalid base-0 allocation flag literal {raw}"));
        }
        (raw.as_str(), 10)
    };
    if digits.is_empty() {
        return Err(format!("invalid allocation flag literal {raw}"));
    }
    let mut value = 0u16;
    for digit in digits.chars() {
        let digit = digit
            .to_digit(radix)
            .ok_or_else(|| format!("invalid allocation flag literal {raw}"))?;
        value = value.wrapping_mul(radix as u16).wrapping_add(digit as u16);
    }
    Ok(value)
}
fn scan_text(text: &str) -> Result<Vec<(usize, String)>, String> {
    let mut out = Vec::new();
    for m in CALL.captures_iter(text) {
        let whole = m.get(0).unwrap();
        let preceding: String = text[..whole.start()]
            .chars()
            .rev()
            .take(8)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        if DECL.is_match(&preceding) {
            continue;
        }
        let Some(args) = args(text, whole.end()) else {
            continue;
        };
        let idx = if &m[1] == "Alloc" { 0 } else { 2 };
        let Some(flag) = args.get(idx) else {
            continue;
        };
        let flag = COMMENTS.replace_all(flag, "");
        let flag = flag.trim();
        let line = text[..whole.start()]
            .bytes()
            .filter(|&b| b == b'\n')
            .count()
            + 1;
        if text
            .lines()
            .nth(line - 1)
            .unwrap_or("")
            .contains("nolint: gmem-flags")
        {
            continue;
        }
        if flag.contains("HEAP_") {
            out.push((line, flag.into()));
            continue;
        }
        if let Some(lit) = LITERAL.captures(flag) {
            if low_bits(&lit[1])? & !0x7f72u16 != 0 {
                out.push((line, flag.into()));
            }
        }
    }
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    const SOURCE: &str = r#"
h = GlobalAlloc(HEAP_ZERO_MEMORY, n as SIZE_T);           // finding
h = GlobalAlloc(0 /* GMEM_FIXED */, n);                   // right
h = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, f(a, b));  // right
h = LocalAlloc(0x8, n);                                   // finding: not an LMEM bit
h = GlobalReAlloc(h, n, 0x42);                            // right: MOVEABLE|ZEROINIT
h = GlobalReAlloc(h, n,
    HEAP_REALLOC_IN_PLACE_ONLY);                          // finding (spans lines)
h = LocalAlloc(HEAP_ZERO_MEMORY, n); // nolint: gmem-flags (test)
pub unsafe extern "system" fn GlobalAlloc(uFlags: UINT, dwBytes: SIZE_T) -> HGLOBAL {
"#;
    #[test]
    fn dedicated_fixture() {
        assert_eq!(
            scan_text(SOURCE)
                .unwrap()
                .iter()
                .map(|hit| hit.0)
                .collect::<Vec<_>>(),
            vec![2, 5, 7]
        );
    }
    #[test]
    fn scanner_controls() {
        assert_eq!(scan_text("unsafe fn load(n: u32) -> HGLOBAL {\n    GlobalAlloc(HEAP_ZERO_MEMORY, n as SIZE_T)\n}\n").unwrap().len(),1);
        assert!(scan_text(
            "unsafe fn load(n: u32) -> HGLOBAL {\n    GlobalAlloc(GMEM_FIXED, n as SIZE_T)\n}\n"
        )
        .unwrap()
        .is_empty());
    }
    #[test]
    fn only_low_sixteen_bits_count_and_calls_in_comments_remain_visible() {
        assert!(
            scan_text("GlobalAlloc(0x100000000000000000000000000000000, n)")
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            scan_text("// GlobalAlloc(0x8u32 as UINT, n)")
                .unwrap()
                .len(),
            1
        );
        assert!(scan_text("GlobalAlloc(010, n)").is_err());
    }
}
