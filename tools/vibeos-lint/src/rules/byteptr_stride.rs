//! Byte-offset arithmetic through a pointer to a byte-pointer typedef.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::LazyLock;
pub struct ByteptrStride;
impl Rule for ByteptrStride {
    fn name(&self) -> &'static str {
        "scan_byteptr_stride"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.rust_files() {
            for (line, ty, method) in scan_text(&file.text) {
                out.push(format!(
                    "{}:{line}: `as *mut {ty}` then `.{method}(` steps by pointer size, not bytes",
                    file.path.display()
                ));
            }
        }
        Ok(out)
    }
}
static SHAPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)as\s+\*(?:mut|const)\s+((?:P|LP)(?:BYTE|UCHAR))\s*\)\s*\.\s*(offset|add|sub|wrapping_add|wrapping_sub|wrapping_offset)\s*\(").unwrap()
});
fn scan_text(text: &str) -> Vec<(usize, String, String)> {
    let mut out = Vec::new();
    for m in SHAPE.captures_iter(text) {
        let line = text[..m.get(0).unwrap().start()]
            .bytes()
            .filter(|&b| b == b'\n')
            .count()
            + 1;
        if text
            .lines()
            .nth(line - 1)
            .unwrap_or("")
            .contains("nolint: byteptr-stride")
        {
            continue;
        }
        out.push((line, m[1].into(), m[2].into()));
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dedicated_fixture() {
        let text = "\nlet a = (rec as *mut PBYTE)\n    .offset(off);                                      // finding (spans lines)\nlet b = (rec as *mut BYTE).offset(off);                // right: BYTE*\nlet c = (rec as PBYTE).offset(off);                    // right: PBYTE itself\nlet d = (tbl as *const PULONG).add(i);                 // an array of pointers: other type\nlet e = (argv as *mut PUCHAR).add(1); // nolint: byteptr-stride (argv-style table)\n";
        assert_eq!(scan_text(text), vec![(2, "PBYTE".into(), "offset".into())]);
    }
    #[test]
    fn scanner_controls() {
        assert_eq!(scan_text("unsafe fn copy(src_rec: *mut c_void, off: isize) -> *mut c_void {\n    (src_rec as *mut PBYTE)\n        .offset(off) as *mut c_void\n}\n").len(),1);
        assert!(scan_text("unsafe fn copy(src_rec: *mut c_void, off: isize) -> *mut c_void {\n    (src_rec as *mut BYTE).offset(off) as *mut c_void\n}\n").is_empty());
    }
}
