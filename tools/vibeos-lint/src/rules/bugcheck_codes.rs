//! Literal kernel stop codes, including wrapped and suffixed integer spellings.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::LazyLock;
pub struct BugcheckCodes;
impl Rule for BugcheckCodes {
    fn name(&self) -> &'static str {
        "scan_bugcheck_codes"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.rust_files() {
            if SKIP.is_match(&file.path.to_string_lossy()) {
                continue;
            }
            for (line, literal) in scan_text(&file.text) {
                out.push(format!(
                    "{}:{line}: bugcheck-code {literal}",
                    file.path.display()
                ));
            }
        }
        Ok(out)
    }
}
static SKIP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(bindgen_[a-z0-9_]+\.rs$|/target/|/\.responses/|/tests?/)").unwrap()
});
static NOLINT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"//\s*nolint(?::\s*bugcheck-code\b|\s*$|:?\s*\()").unwrap());
static CALL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bKeBugCheck(?:Ex)?\s*\(").unwrap());
static NUMBER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(0x[0-9A-Fa-f_]+|\d[\d_]*)(?:[ui](?:8|16|32|64|128|size))?\s*[,)]").unwrap()
});
fn scan_text(text: &str) -> Vec<(usize, String)> {
    let lines: Vec<_> = text.lines().collect();
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("//") || line.trim().starts_with('*') {
            continue;
        }
        for m in CALL.find_iter(line) {
            let mut probe = &line[m.end()..];
            let mut probe_line = i;
            for ahead in 1..3 {
                if !probe.trim().is_empty() || i + ahead >= lines.len() {
                    break;
                }
                probe_line = i + ahead;
                probe = lines[probe_line];
            }
            let Some(hit) = NUMBER.captures(probe) else {
                continue;
            };
            if NOLINT.is_match(line) || NOLINT.is_match(lines[probe_line]) {
                continue;
            }
            out.push((i + 1, hit[1].into()));
        }
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scanner_control() {
        let bad = "#[no_mangle]\npub unsafe extern \"system\" fn Boom() {\n    // The literal that hid from this very scanner for six weeks (§9.120).\n    KeBugCheckEx(0x35u32, 0, 0, 0, 0);\n}\n";
        assert_eq!(scan_text(bad), vec![(4, "0x35".into())]);
        assert!(scan_text("#[no_mangle]\npub unsafe extern \"system\" fn Boom() {\n    KeBugCheckEx(ntkrnl_sys::bugcodes::INVALID_AFFINITY_SET, 0, 0, 0, 0);\n}\n").is_empty());
    }
    #[test]
    fn wrapping_suffixes_suppression_and_comment_lines() {
        assert_eq!(
            scan_text("KeBugCheckEx(\n\n0x1E_u32, 0);"),
            vec![(1, "0x1E_".into())]
        );
        assert!(scan_text("KeBugCheckEx(\n\n\n0x1E, 0);").is_empty());
        assert!(scan_text(
            "KeBugCheck(5usize); // nolint: bugcheck-code\n// KeBugCheck(5);\n* KeBugCheck(5);"
        )
        .is_empty());
        assert!(scan_text("KeBugCheck(\n5); // nolint (reviewed)").is_empty());
    }
}
