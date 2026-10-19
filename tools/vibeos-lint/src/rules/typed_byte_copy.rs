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
pub struct TypedByteCopy;
impl Rule for TypedByteCopy {
    fn name(&self) -> &'static str {
        "scan_typed_byte_copy"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        Ok(corpus
            .rust_files()
            .filter(|f| !skip(&f.path.to_string_lossy()))
            .flat_map(|f| {
                scan_text(&f.text)
                    .into_iter()
                    .map(|(n, elem, count)| {
                        format!("{}:{n}: typed-byte-copy {elem} {count}", f.path.display())
                    })
                    .collect::<Vec<_>>()
            })
            .collect())
    }
}
struct Patterns {
    call: Regex,
    cast: Regex,
    bare: Regex,
    base: Regex,
    declared: Regex,
    alias: Regex,
    count: Regex,
    times_sizeof: Regex,
    divided: Regex,
    byte_named: Regex,
    nolint: Regex,
}
fn patterns() -> &'static Patterns {
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(|| Patterns {
        call: regex(r"\b(?:core::ptr::|std::ptr::|ptr::)?(copy_nonoverlapping|copy|write_bytes)\s*(?:::<\s*([^>]+)>\s*)?\("),
        cast: regex(r"\bas\s+\*(?:mut|const)\s+((?:[\w:]+::)?\w+|_)"),
        bare: regex(r"^\s*([A-Za-z_]\w*)\s*$"),
        base: regex(r"^\s*\(?\s*([A-Za-z_]\w*)\s*\)?\s+as\b"),
        declared: regex(r"\b(\w+)\s*:\s*(?:\*(?:mut|const)\s+((?:[\w:]+::)?\w+)|((?:[\w:]+::)?\w+))"),
        alias: regex(r"^L?P[A-Z0-9_]+$"),
        count: regex(r"\b(?:size|\w*Size|\w*_size|\w*Length|\w*Bytes\w*|\w*ByteCount|cb[A-Z_]\w*|bytes)\b"),
        times_sizeof: regex(r"\*\s*(?:\(\s*)?(?:core::|std::)?(?:mem::)?size_of\b|size_of\b[^/]*\*"),
        divided: regex(r"/\s*\(?\s*(?:core::|std::)?(?:mem::)?size_of"),
        // Stricter than `count`: names that only ever mean bytes. A pointer typed by its
        // declaration (no cast at the call) is flagged with one of these, not with a bare
        // `size` (`OldListSize` is an element count). classpnp's ClasspReadCapacity16
        // zeroed `allocation_buffer_length` (32) ELEMENTS of a 32-byte buffer through a
        // `*mut _READ_CAPACITY16_DATA`: 1024 bytes of pool, a BAD_POOL_HEADER from user mode.
        byte_named: regex(r"(?i:\b\w*(?:_length|_bytes|_byte_count)\b)|\b\w*(?:Length|Bytes|ByteCount)\b|\bcb[A-Z_]\w*\b|\b(?:bytes|length)\b"),
        nolint: regex(r"//\s*nolint(?::\s*typed-byte-copy\b|\s*$|:?\s*\()"),
    })
}
fn leaf(t: &str) -> &str {
    t.rsplit("::").next().unwrap()
}
fn byte_type(t: &str) -> bool {
    matches!(
        t,
        "u8" | "i8"
            | "c_void"
            | "c_char"
            | "c_uchar"
            | "c_schar"
            | "UCHAR"
            | "CHAR"
            | "BYTE"
            | "BOOLEAN"
            | "INT8"
            | "UINT8"
            | "CCHAR"
            | "__u8"
    )
}
fn wide_type(t: &str) -> bool {
    matches!(t, "u16" | "WCHAR" | "wchar_t" | "c_ushort")
}
fn declared(text: &str, before: usize, name: &str) -> Option<String> {
    let prefix = &text[..before];
    let start = prefix
        .char_indices()
        .rev()
        .nth(19999)
        .map_or(0, |(at, _)| at);
    let best = patterns()
        .declared
        .captures_iter(&prefix[start..])
        .filter(|c| c.get(1).unwrap().as_str() == name)
        .last()?;
    if let Some(t) = best.get(2) {
        return Some(leaf(t.as_str()).into());
    }
    let alias = leaf(best.get(3).unwrap().as_str());
    if matches!(
        alias,
        "PVOID"
            | "LPVOID"
            | "PCVOID"
            | "LPCVOID"
            | "PUCHAR"
            | "PBYTE"
            | "LPBYTE"
            | "PCHAR"
            | "PSTR"
            | "LPSTR"
            | "PCSTR"
            | "LPCSTR"
            | "PCCH"
            | "PCH"
            | "PBOOLEAN"
    ) {
        return Some("u8".into());
    }
    if matches!(
        alias,
        "PWSTR" | "LPWSTR" | "PCWSTR" | "LPCWSTR" | "PWCHAR" | "PWCH" | "PCWCH" | "PTSTR"
            | "LPTSTR" | "PCTSTR" | "LPCTSTR" | "PTCHAR" | "LPOLESTR" | "LPCOLESTR"
            | "POLESTR" | "PCOLESTR"
    ) {
        return Some("u16".into());
    }
    patterns()
        .alias
        .is_match(alias)
        .then(|| alias[if alias.starts_with("LP") { 2 } else { 1 }..].into())
}
fn element_type(
    text: &str,
    at: usize,
    ptrs: &[&str],
    turbofish: Option<&str>,
) -> (Option<String>, bool) {
    if let Some(t) = turbofish.filter(|t| !t.is_empty()) {
        return (Some(leaf(t.trim()).into()), true);
    }
    let mut inferred = None;
    let mut uncast = None;
    for ptr in ptrs {
        let casts: Vec<_> = patterns().cast.captures_iter(ptr).collect();
        if casts.is_empty() {
            if uncast.is_none() {
                if let Some(bare) = patterns().bare.captures(ptr) {
                    uncast = declared(text, at, bare.get(1).unwrap().as_str());
                }
            }
            continue;
        }
        let t = casts.last().unwrap().get(1).unwrap().as_str();
        if t != "_" {
            return (Some(leaf(t).into()), true);
        }
        if casts.len() == 1 && inferred.is_none() {
            if let Some(base) = patterns().base.captures(ptr) {
                inferred = declared(text, at, base.get(1).unwrap().as_str());
            }
        }
    }
    if inferred.is_some() {
        (inferred, true)
    } else {
        (uncast, false)
    }
}
fn scan_text(text: &str) -> Vec<(usize, String, String)> {
    let p = patterns();
    let mut out = Vec::new();
    for call in p.call.captures_iter(text) {
        let m = call.get(0).unwrap();
        let Some(args) = call_args(text, m.end()) else {
            continue;
        };
        if args.len() != 3 {
            continue;
        }
        let write = call.get(1).unwrap().as_str() == "write_bytes";
        let ptrs = if write { &args[..1] } else { &args[..2] };
        let count = args[2];
        let (elem, cast) = element_type(text, m.start(), ptrs, call.get(2).map(|c| c.as_str()));
        let Some(elem) = elem else {
            continue;
        };
        if byte_type(&elem) || wide_type(&elem) || (write && matches!(count.trim(), "1" | "1usize"))
        {
            continue;
        }
        if !(p.times_sizeof.is_match(count)
            || (cast && p.count.is_match(count))
            // Not a call: `BYTES_TO_PAGES(ReadLength)` is already in elements.
            || (p.byte_named.is_match(count) && !count.contains('(')))
            || p.divided.is_match(count)
        {
            continue;
        }
        let n = line(text, m.start());
        if p.nolint.is_match(text.lines().nth(n - 1).unwrap_or("")) {
            continue;
        }
        out.push((n, elem, normalize(count)));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn _typedbyte_bad() {
        let text = "pub unsafe fn f(Irp: PIRP, size: usize) {\n    let mut layoutEx: *mut _DRIVE_LAYOUT_INFORMATION_EX = core::ptr::null_mut();\n    core::ptr::copy_nonoverlapping(\n        layoutEx as *const _,\n        (*Irp).AssociatedIrp.SystemBuffer as *mut _,\n        size,\n    );\n}\npub unsafe fn g(a: *const ENTRY, b: *mut ENTRY, n: usize) {\n    core::ptr::copy_nonoverlapping(a, b, n * core::mem::size_of::<ENTRY>());\n}\n";
        assert!(!scan_text(text).is_empty());
    }
    #[test]
    fn _typedbyte_good() {
        let text = "pub unsafe fn f(Irp: PIRP, size: usize, h: *const HDR, out: *mut u8) {\n    let mut layoutEx: *mut _DRIVE_LAYOUT_INFORMATION_EX = core::ptr::null_mut();\n    core::ptr::copy_nonoverlapping(\n        layoutEx as *const u8,\n        (*Irp).AssociatedIrp.SystemBuffer as *mut u8,\n        size,\n    );\n    core::ptr::copy_nonoverlapping(h as *const HDR as *const u8, out, (*h).HeaderSize as usize);\n}\npub unsafe fn g(a: *const ENTRY, b: *mut ENTRY, OldListSize: u32) {\n    core::ptr::copy_nonoverlapping(a, b, OldListSize as usize);\n}\n";
        assert!(scan_text(text).is_empty());
    }
    #[test]
    fn turbofish_inference_and_count_controls() {
        assert_eq!(scan_text("copy::<Thing>(a, b, byteSize);")[0].1, "Thing");
        assert!(scan_text("copy::<Thing>(a, b, byteSize / size_of::<Thing>());").is_empty());
        assert!(scan_text("write_bytes(p as *mut Thing, 0, 1usize);").is_empty());
        assert!(scan_text("copy(a as *const u8, b as *mut _, byteSize);").is_empty());
        // A declaration-typed pointer with a byte-named count (classpnp's READ CAPACITY 16).
        let rc16 = "let data_buffer: *mut _READ_CAPACITY16_DATA = alloc();\n\
                    core::ptr::write_bytes(data_buffer, 0, allocation_buffer_length as usize);";
        assert_eq!(scan_text(rc16)[0].1, "_READ_CAPACITY16_DATA");
        let ok = "let data_buffer: *mut _READ_CAPACITY16_DATA = alloc();\n\
                  core::ptr::write_bytes(data_buffer as *mut u8, 0, allocation_buffer_length as usize);";
        assert!(scan_text(ok).is_empty());
        // Element counts that merely mention a length.
        assert!(scan_text("let p: *mut PFN_NUMBER = q;\nwrite_bytes(p, 0, BYTES_TO_PAGES(ReadLength as usize));").is_empty());
        assert!(scan_text("let v: PCTSTR = q;\ncopy_nonoverlapping(v, d, VarLength);").is_empty());
        assert!(scan_text("let t: *mut ENTRY = q;\ncopy_nonoverlapping(s, t, len + 1);").is_empty());
        assert_eq!(
            declared("fn f(p: LPTHING) {}", 17, "p"),
            Some("THING".into())
        );
    }
}
