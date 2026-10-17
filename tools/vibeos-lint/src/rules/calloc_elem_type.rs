use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::OnceLock;
fn regex(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap()
}
fn line(text: &str, at: usize) -> usize {
    text[..at].bytes().filter(|b| *b == b'\n').count() + 1
}
pub struct CallocElemType;
impl Rule for CallocElemType {
    fn name(&self) -> &'static str {
        "scan_calloc_elem_type"
    }
    fn needs_sdk_rust(&self) -> bool {
        true
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        Ok(corpus.sdk_rust_files().flat_map(|f| {
            scan_text(&f.text).into_iter().map(|(n, sized, used)| format!("{}:{n}: CALLOC_ELEM_TYPE sized as {sized}, used as *mut {used} (the C's sizeof(*p)?)", f.path.display())).collect::<Vec<_>>()
        }).collect())
    }
}
fn narrow(t: &str) -> bool {
    matches!(
        t,
        "u8" | "i8"
            | "c_char"
            | "c_uchar"
            | "c_schar"
            | "CHAR"
            | "UCHAR"
            | "BYTE"
            | "TCHAR"
            | "u16"
            | "i16"
            | "c_ushort"
            | "c_short"
            | "USHORT"
            | "SHORT"
            | "WORD"
            | "WCHAR"
            | "OLECHAR"
            | "wchar_t"
    )
}
fn scan_text(text: &str) -> Vec<(usize, String, String)> {
    static COMMENT: OnceLock<Regex> = OnceLock::new();
    static CALLOC: OnceLock<Regex> = OnceLock::new();
    let code = COMMENT
        .get_or_init(|| regex(r"//[^\n]*"))
        .replace_all(text, "");
    let re = CALLOC.get_or_init(|| regex(r"\b(?:heap_)?calloc\s*\(\s*(?:[^;]{1,160}?),\s*\(?\s*(?:(?:::)?core::mem::|mem::)?size_of::<\s*([\w:]+)\s*>\s*\(\)\s*\)?\s*(?:as\s+[\w:]+\s*)?\)\s*as\s*\*mut\s+([\w:]+)"));
    re.captures_iter(&code)
        .filter_map(|c| {
            let sized = c.get(1).unwrap().as_str().rsplit("::").next().unwrap();
            let used = c.get(2).unwrap().as_str().rsplit("::").next().unwrap();
            (narrow(sized) && !narrow(used) && used != "c_void").then(|| {
                (
                    line(&code, c.get(0).unwrap().start()),
                    sized.to_string(),
                    used.to_string(),
                )
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_the_wscript_echo_array_fires() {
        let text = "    argc = (ubound - lbound + 1) as u32;\n    strs = calloc((argc as usize) as _,\n        (core::mem::size_of::<OLECHAR>()) as _) as *mut BSTR;\n";
        assert_eq!(scan_text(text), vec![(2, "OLECHAR".into(), "BSTR".into())]);
    }
    #[test]
    fn test_a_byte_array_of_pointers_fires_in_any_spelling() {
        let text = "let p = heap_calloc(n, ::core::mem::size_of::<u8>()) as *mut LPWSTR;\n";
        assert_eq!(scan_text(text), vec![(1, "u8".into(), "LPWSTR".into())]);
    }
    #[test]
    fn test_matching_and_pointer_sized_elements_are_silent() {
        let text = "strs = calloc(argc as _, size_of::<BSTR>() as _) as *mut BSTR;\nbuf = calloc(n, core::mem::size_of::<WCHAR>()) as *mut WCHAR;\nt = calloc(32, core::mem::size_of::<*mut core::ffi::c_void>() as _) as *mut _PVFV;\nv = calloc(1, size_of::<u8>()) as *mut c_void;\n";
        assert_eq!(scan_text(text), vec![]);
    }
    #[test]
    fn test_a_flexible_array_tail_and_a_comment_are_not_candidates() {
        let text = "cur = malloc(offset_of!(T, data) + n * size_of::<c_char>()) as *mut T;\n// strs = calloc(argc, size_of::<OLECHAR>()) as *mut BSTR;\n";
        assert_eq!(scan_text(text), vec![]);
    }
}
