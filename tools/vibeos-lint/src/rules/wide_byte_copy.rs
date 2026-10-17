use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::OnceLock;
fn regex(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap()
}
fn normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
fn line(text: &str, at: usize) -> usize {
    text[..at].bytes().filter(|b| *b == b'\n').count() + 1
}
fn skip(path: &str) -> bool {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        regex(r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/island_[a-z0-9_]+\.rs$)")
    })
    .is_match(path)
}
fn call_args(text: &str, start: usize) -> Option<Vec<&str>> {
    let mut depth = 1usize;
    let mut last = start;
    let mut parts = Vec::new();
    for (at, b) in text.bytes().enumerate().skip(start) {
        match b {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                depth -= 1;
                if depth == 0 {
                    parts.push(text[last..at].trim());
                    return Some(parts.into_iter().filter(|p| !p.is_empty()).collect());
                }
            }
            b',' if depth == 1 => {
                parts.push(text[last..at].trim());
                last = at + 1;
            }
            _ => {}
        }
    }
    None
}
pub struct WideByteCopy;
impl Rule for WideByteCopy {
    fn name(&self) -> &'static str {
        "scan_wide_byte_copy"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        Ok(corpus
            .rust_files()
            .filter(|f| !skip(&f.path.to_string_lossy()))
            .flat_map(|f| {
                scan_text(&f.text)
                    .into_iter()
                    .map(|(n, count)| format!("{}:{n}: wide-byte-copy {count}", f.path.display()))
                    .collect::<Vec<_>>()
            })
            .collect())
    }
}
struct Patterns {
    call: Regex,
    wide: Regex,
    byte: Regex,
    count: Regex,
    divided: Regex,
    nolint: Regex,
}
fn patterns() -> &'static Patterns {
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(|| Patterns {
        call: regex(r"\b(?:core::ptr::|std::ptr::|ptr::)?(?:copy_nonoverlapping|copy)\s*\("),
        wide: regex(r"\bas\s+(?:\*(?:mut|const)\s+(?:u16|WCHAR|wchar_t)\b|(?:L?PC?WSTR|PWCHAR)\b)"),
        byte: regex(
            r"\*(?:mut|const)\s+(?:(?:core|std)::ffi::)?(?:u8|i8|c_void|c_char|UCHAR|CHAR|BYTE)\b",
        ),
        count: regex(r"\b(?:\w*Length|\w*Bytes\w*|\w*ByteCount|cb\w*|\w*Size)\b"),
        divided: regex(r"/\s*(?:2\b|\(?\s*(?:core::|std::)?(?:mem::)?size_of)|>>\s*1\b"),
        nolint: regex(r"//\s*nolint(?::\s*wide-byte-copy\b|\s*$|:?\s*\()"),
    })
}
fn scan_text(text: &str) -> Vec<(usize, String)> {
    let p = patterns();
    let mut out = Vec::new();
    for m in p.call.find_iter(text) {
        let Some(args) = call_args(text, m.end()) else {
            continue;
        };
        if args.len() != 3 {
            continue;
        }
        let ptrs = format!("{}\n{}", args[0], args[1]);
        if !p.wide.is_match(&ptrs)
            || p.byte.is_match(&ptrs)
            || !p.count.is_match(args[2])
            || p.divided.is_match(args[2])
        {
            continue;
        }
        let n = line(text, m.start());
        if p.nolint.is_match(text.lines().nth(n - 1).unwrap_or("")) {
            continue;
        }
        out.push((n, normalize(args[2])));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn _widebyte_bad() {
        let text = "pub unsafe fn f(name: *const UNICODE_STRING, dst: *mut u8, BytesConverted: u32) {\n    core::ptr::copy_nonoverlapping(\n        (*name).Buffer,\n        dst as *mut u16,\n        BytesConverted as usize,\n    );\n}\n";
        assert!(!scan_text(text).is_empty());
    }
    #[test]
    fn _widebyte_good() {
        let text = "pub unsafe fn f(name: *const UNICODE_STRING, dst: *mut u8, BytesConverted: u32) {\n    core::ptr::copy_nonoverlapping((*name).Buffer as *const u8, dst, BytesConverted as usize);\n    core::ptr::copy_nonoverlapping(\n        (*name).Buffer,\n        dst as *mut u16,\n        (*name).Length as usize / core::mem::size_of::<u16>(),\n    );\n}\npub unsafe fn g(a: *const ANSI_STRING, out: *mut i8) {\n    core::ptr::copy_nonoverlapping((*a).Buffer, out, (*a).Length as usize);\n}\n";
        assert!(scan_text(text).is_empty());
    }
}
