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
pub struct ByteStepOne;
impl Rule for ByteStepOne {
    fn name(&self) -> &'static str {
        "scan_byte_step_one"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        Ok(corpus
            .rust_files()
            .filter(|f| !skip(&f.path.to_string_lossy()))
            .flat_map(|f| {
                scan_text(&f.text)
                    .into_iter()
                    .map(|(n, expr)| format!("{}:{n}: byte-step-one {expr}", f.path.display()))
                    .collect::<Vec<_>>()
            })
            .collect())
    }
}
fn scan_text(text: &str) -> Vec<(usize, String)> {
    static COMMENTS: OnceLock<Regex> = OnceLock::new();
    static STEP: OnceLock<Regex> = OnceLock::new();
    let code = COMMENTS
        .get_or_init(|| regex(r"(?s)//[^\n]*|/\*.*?\*/"))
        .replace_all(text, |caps: &regex::Captures<'_>| {
            caps.get(0)
                .unwrap()
                .as_str()
                .chars()
                .map(|c| if c == '\n' { '\n' } else { ' ' })
                .collect::<String>()
        });
    let re = STEP.get_or_init(|| regex(r"\(\s*((?:[^()]|\(\*?[\w.]+\))+?)\s+as\s+\*(?:mut|const)\s+(?:u8|i8|c_char|core::ffi::c_char|UCHAR|BYTE|CHAR)\s*\)\s*\.(?:add|offset|wrapping_add|wrapping_offset)\(\s*1(?:\s+as\s+\w+)?\s*\)"));
    re.captures_iter(&code)
        .filter(|c| {
            code[..c.get(0).unwrap().start()]
                .chars()
                .next_back()
                .is_none_or(|ch| !(ch == '!' || ch == '_' || ch.is_alphanumeric()))
        })
        .map(|c| {
            (
                line(&code, c.get(0).unwrap().start()),
                normalize(c.get(1).unwrap().as_str()),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn _byte_step_one_bad() {
        let text = "pub unsafe fn folders(h: *mut CFHEADER) -> *mut u8 {\n    // C: (PUCHAR)(PCABHeader + 1) -- past the whole header, not one byte.\n    (h as *mut u8).add(1)\n}\n";
        assert!(!scan_text(text).is_empty());
    }
    #[test]
    fn _byte_step_one_good() {
        let text = "pub unsafe fn folders(h: *mut CFHEADER) -> *mut u8 {\n    h.add(1) as *mut u8\n}\npub unsafe fn len_plus_nul(p: *const i8) -> usize {\n    strlen(p as *const i8).wrapping_add(1)\n}\n";
        assert!(scan_text(text).is_empty());
    }
    #[test]
    fn test_flags_header_steps() {
        let text = "\nBuffer = ((*CabinetContext).PCABHeader as *mut u8).add(1);\nlet p = (pHeader as *mut u8).add(1 as usize) as *mut DELETED_FILE_RECORD;\n";
        assert_eq!(
            scan_text(text),
            vec![
                (2, "(*CabinetContext).PCABHeader".into()),
                (3, "pHeader".into())
            ]
        );
    }
    #[test]
    fn test_quiet_on_calls_and_the_fix() {
        let text = "\nlet n = strlen(p as *const i8).wrapping_add(1);\nlet t = strstr(buf.as_ptr(), b\".\\0\".as_ptr() as *const i8).offset(1);\nBuffer = (*CabinetContext).PCABHeader.add(1) as *mut u8;\n// Buffer = ((*CabinetContext).PCABHeader as *mut u8).add(1);\n";
        assert_eq!(scan_text(text), vec![]);
    }
}
