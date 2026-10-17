//! Detect exported printf-family functions with no argument after their format parameter.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::OnceLock;

pub struct FormatlessPrintf;
impl Rule for FormatlessPrintf {
    fn name(&self) -> &'static str {
        "scan_formatless_printf"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.rust_files() {
            if patterns().skip.is_match(&file.path.to_string_lossy()) {
                continue;
            }
            for (line, name) in scan_text(&file.text) {
                out.push(format!(
                    "{}:{line}: formatless-printf {name}",
                    file.path.display()
                ));
            }
        }
        Ok(out)
    }
}
struct Patterns {
    comments: Regex,
    definition: Regex,
    printf: Regex,
    vform: Regex,
    format_param: Regex,
    skip: Regex,
}
fn patterns() -> &'static Patterns {
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(|| Patterns {
        comments: crate::regexes::cached(r"(?s)//[^\n]*|/\*.*?\*/").unwrap(),
        definition: crate::regexes::cached(r#"(?s)#\[(?:unsafe\()?no_mangle\)?\]\s*(?:#\[[^\]]*\]\s*)*pub\s+(?:unsafe\s+)?extern\s+"[^"]+"\s+fn\s+(\w+)\s*\(([^)]*)\)"#).unwrap(),
        printf: crate::regexes::cached(r"[Pp]rintf").unwrap(),
        vform: crate::regexes::cached(r"(?i)(^|_)v\w*printf|VPrintf").unwrap(),
        format_param: crate::regexes::cached(r"(?i)^(?:mut\s+)?(\w*(?:fmt|format)\w*)\s*:").unwrap(),
        skip: crate::regexes::cached(r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/island_[a-z0-9_]+\.rs$)").unwrap(),
    })
}
fn code_only(text: &str) -> String {
    patterns()
        .comments
        .replace_all(text, |m: &regex::Captures<'_>| {
            m[0].chars()
                .map(|c| if c == '\n' { '\n' } else { ' ' })
                .collect::<String>()
        })
        .into_owned()
}
fn is_printf_name(name: &str) -> bool {
    patterns().printf.find_iter(name).any(|m| {
        !name[m.end()..]
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase())
    })
}
fn scan_text(text: &str) -> Vec<(usize, String)> {
    let code = code_only(text);
    patterns()
        .definition
        .captures_iter(&code)
        .filter_map(|m| {
            let name = m.get(1)?.as_str();
            if !is_printf_name(name) || patterns().vform.is_match(name) {
                return None;
            }
            let params: Vec<_> = m
                .get(2)?
                .as_str()
                .split(',')
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .collect();
            let fmt_at = params
                .iter()
                .position(|p| patterns().format_param.is_match(p))?;
            if fmt_at + 1 != params.len() {
                return None;
            }
            let line = code[..m.get(0)?.start()]
                .bytes()
                .filter(|b| *b == b'\n')
                .count()
                + 1;
            Some((line, name.to_owned()))
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dedicated_positive_and_negative_fixtures() {
        let bad = "\n#[no_mangle]\npub unsafe extern \"C\" fn RtlStringCbPrintfW(\n dest: *mut u16,\n cb_dest: usize,\n fmt: *const u16,\n) -> NTSTATUS { 0 }\n#[no_mangle] pub unsafe extern \"C\" fn RtlStringCchPrintfA(d: *mut i8, n: usize, fmt: *const i8) {}\n";
        assert_eq!(
            scan_text(bad)
                .into_iter()
                .map(|(_, n)| n)
                .collect::<Vec<_>>(),
            vec!["RtlStringCbPrintfW", "RtlStringCchPrintfA"]
        );
        let attr = "\n\n#[no_mangle]\npub unsafe extern \"C\" fn sprintf(b: *mut i8, format: *const i8) {}\n";
        let attr_code = code_only(attr);
        let m = patterns().definition.captures(&attr_code).unwrap();
        assert_eq!(
            attr_code[..m.get(0).unwrap().start()]
                .bytes()
                .filter(|b| *b == b'\n')
                .count()
                + 1,
            3
        );
        let good = "#[no_mangle]\npub unsafe extern \"C\" fn KdIoPrintf(Format: PCSTR, a0: usize, a1: usize) {}\n#[no_mangle]\npub unsafe extern \"C\" fn _vsnwprintf(b: *mut u16, n: usize, fmt: *const u16, ap: va_list) {}\n#[no_mangle]\npub unsafe extern \"C\" fn StringCchVPrintfW(d: *mut u16, n: usize, fmt: *const u16, ap: *mut u8) {}\nextern \"C\" { fn RtlStringCbPrintfW(dest: *mut u16, cb: usize, fmt: *const u16, ...) -> NTSTATUS; }\n// #[no_mangle] pub unsafe extern \"C\" fn RtlStringCbPrintfA(d: *mut i8, n: usize, fmt: *const i8) {}\n";
        assert!(scan_text(good).is_empty());
    }
    #[test]
    fn scanner_control_and_name_false_positive() {
        let source = "#[no_mangle]\npub unsafe extern \"C\" fn RtlStringCbPrintfW(d: *mut u16, cb: usize, fmt: *const u16) -> i32 { 0 }\n#[no_mangle]\npub unsafe extern \"C\" fn SeiPrintFunctionName(p: LPCSTR, szOrdProcFmt: *mut i8) -> LPCSTR { p }\n#[no_mangle]\npub unsafe extern \"C\" fn DbgPrintfEx(Format: PCSTR) -> u32 { 0 }\n";
        assert_eq!(
            scan_text(source)
                .into_iter()
                .map(|(_, n)| n)
                .collect::<Vec<_>>(),
            vec!["RtlStringCbPrintfW", "DbgPrintfEx"]
        );
    }
}
