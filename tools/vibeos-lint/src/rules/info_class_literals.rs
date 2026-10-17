//! File-information class arguments must use named constants.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::LazyLock;
pub struct InfoClassLiterals;
impl Rule for InfoClassLiterals {
    fn name(&self) -> &'static str {
        "scan_info_class_literals"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.rust_files() {
            if SKIP.is_match(&file.path.to_string_lossy()) {
                continue;
            }
            let masked = file.masked(&COMMENTS, None, false);
            for (line, what) in scan_masked(&file.text, &masked) {
                out.push(format!(
                    "{}:{line}: info-class-literal {what}",
                    file.path.display()
                ));
            }
        }
        out.sort();
        Ok(out)
    }
}
static COMMENTS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)//[^\n]*|/\*.*?\*/").unwrap());
static SKIP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/island_[a-z0-9_]+\.rs$)").unwrap()
});
static CALL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:[A-Za-z_]\w*::)*(NtQueryInformationFile|ZwQueryInformationFile|NtSetInformationFile|ZwSetInformationFile|IoQueryFileInformation|IoSetInformation)\s*\(").unwrap()
});
static WORD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\w$").unwrap());
static DECL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bfn\s+$").unwrap());
static LITERAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\(?\s*-?(?:0x[0-9a-fA-F_]+|\d[\d_]*)(?:[iu](?:8|16|32|64|size))?\s*\)?(?:\s+as\s+[A-Za-z_][\w:]*)?$").unwrap()
});
#[cfg(test)]
fn code(text: &str) -> String {
    let mut out = text.as_bytes().to_vec();
    for m in COMMENTS.find_iter(text) {
        for b in &mut out[m.range()] {
            if *b != b'\n' {
                *b = b' ';
            }
        }
    }
    String::from_utf8(out).unwrap()
}
fn args(code: &str, start: usize) -> Option<Vec<&str>> {
    let mut depth: isize = 0;
    let mut argument_start = start + 1;
    let mut out = Vec::new();
    for (offset, byte) in code.as_bytes()[start..].iter().enumerate() {
        let at = start + offset;
        match byte {
            b'(' | b'[' | b'{' => {
                depth += 1;
                if depth == 1 {
                    continue;
                }
            }
            b')' | b']' | b'}' => {
                depth -= 1;
                if depth == 0 {
                    out.push(code[argument_start..at].trim());
                    return Some(out);
                }
            }
            b',' if depth == 1 => {
                out.push(code[argument_start..at].trim());
                argument_start = at + 1;
            }
            _ => {}
        }
    }
    None
}
#[cfg(test)]
fn scan_text(text: &str) -> Vec<(usize, String)> {
    let masked = code(text);
    scan_masked(text, &masked)
}
fn scan_masked(text: &str, masked: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for m in CALL.captures_iter(&masked) {
        let whole = m.get(0).unwrap();
        if masked[..whole.start()]
            .chars()
            .next_back()
            .is_some_and(|c| {
                let mut bytes = [0; 4];
                c == ':' || WORD.is_match(c.encode_utf8(&mut bytes))
            })
        {
            continue;
        }
        let preceding: String = masked[..whole.start()]
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
        let Some(args) = args(&masked, whole.end() - 1) else {
            continue;
        };
        let idx = if m[1].starts_with("Io") { 1 } else { 4 };
        let Some(class) = args.get(idx) else {
            continue;
        };
        if LITERAL.is_match(class) {
            let line = text[..whole.start()]
                .bytes()
                .filter(|&b| b == b'\n')
                .count()
                + 1;
            out.push((line, format!("{} class `{class}`", &m[1])));
        }
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn information_class_control() {
        let bad = "pub unsafe fn extend(fo: PFILE_OBJECT, size: *mut LARGE_INTEGER) -> NTSTATUS {\n    // C: FileEndOfFileInformation (20). 2 is FileFullDirectoryInformation.\n    IoSetInformation(fo, 2u32, 8, size as PVOID)\n}\n";
        assert_eq!(
            scan_text(bad),
            vec![(3, "IoSetInformation class `2u32`".into())]
        );
        let good = "pub unsafe fn extend(fo: PFILE_OBJECT, size: *mut LARGE_INTEGER) -> NTSTATUS {\n    IoSetInformation(fo, _FILE_INFORMATION_CLASS_FileEndOfFileInformation as u32, 8, size as PVOID)\n}\nextern \"system\" {\n    fn IoSetInformation(fo: PFILE_OBJECT, class: u32, len: u32, buf: PVOID) -> NTSTATUS;\n}\n";
        assert!(scan_text(good).is_empty());
    }
    #[test]
    fn nested_arguments_namespaces_and_truncated_calls() {
        assert_eq!(scan_text("ffi::NtQueryInformationFile(handle(a, b), [1,2], { x }, data, (0x2u32) as InfoClass);").len(), 1);
        assert!(scan_text("fn IoSetInformation(fo, 2u32, 8, p);").is_empty());
        assert!(scan_text("IoSetInformation(fo, 2u32,").is_empty());
        assert!(scan_text("// IoSetInformation(fo, 2u32, 8, p);").is_empty());
    }
}
