//! Flag byte-string UTF-16 literals cast directly to wide pointers.
use crate::{Corpus, Rule, SourceFile, SourceSet};
use regex::Regex;
use std::sync::LazyLock;
pub struct UnalignedWide;
static INPUTS: &[SourceSet] =
    &[SourceSet::tree("unaligned_wide_rust", &["sdk/lib/rust"], &["rs"], &[]).with_inputs()];
impl Rule for UnalignedWide {
    fn name(&self) -> &'static str {
        "scan_unaligned_wide"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let mut files: Vec<_> = c
            .sources("unaligned_wide_rust")
            .filter(|f| f.path.extension().is_some_and(|x| x == "rs"))
            .collect();
        files.sort_by(|a, b| a.path.cmp(&b.path));
        let mut out = Vec::new();
        for f in files {
            for cap in SITE.captures_iter(&f.text) {
                let m = cap.get(0).unwrap();
                let snippet: String = m.as_str().chars().take(90).collect();
                out.push(format!("{}:{}: unaligned UTF-16 byte literal cast to a wide pointer; use win32macros_rust::wide_bytes!(...): {snippet}",f.path.display(),f.line(m.start())));
            }
        }
        Ok(out)
    }
}
const WIDE:&str="LPCWSTR|LPWSTR|PCWSTR|PWSTR|PWCHAR|PWCH|PCWCH|LPCOLESTR|LPOLESTR|BSTR|u16|wchar_t|WCHAR|c_ushort|USHORT";
static SITE: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(&format!(r#"(?P<lit>b"(?:[^"\\]|\\.)*\\0(?:[^"\\]|\\.)*")\s*\.as_ptr\(\)(?P<cast>\s*as\s*(?:\*\s*(?:const|mut)\s*)?(?:{WIDE})\b)"#)).unwrap()
});
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    const BAD: &str = r#"pub unsafe fn f() {
    RegQueryValueExW(key, b"L\0i\0b\0\0\0".as_ptr() as LPCWSTR, 0 as _);
}
"#;
    const GOOD: &str = r#"pub unsafe fn f() {
    RegQueryValueExW(key, win32macros_rust::wide_bytes!(b"L\0i\0b\0\0\0") as LPCWSTR, 0 as _);
    OutputDebugStringA(b"text\0".as_ptr() as PCSTR);
}
"#;
    #[test]
    fn original_regex_shapes() {
        assert!(SITE.is_match(r#"f(b"L\0i\0b\0\0\0".as_ptr() as LPCWSTR);"#));
        assert!(SITE.is_match(r#"x = b"a\0\0\0".as_ptr() as *const u16;"#));
        assert!(SITE.is_match(r#"x = b"a\0\0\0".as_ptr() as *mut WCHAR;"#));
        assert!(!SITE.is_match(r#"f(b"text\0".as_ptr() as PCSTR);"#));
        assert!(!SITE.is_match(r#"f(win32macros_rust::wide_bytes!(b"L\0\0\0") as LPCWSTR);"#));
    }
    #[test]
    fn rewrite_preserves_literal_and_cast() {
        let c = SITE
            .captures(r#"f(b"L\0i\0b\0\0\0".as_ptr() as LPCWSTR);"#)
            .unwrap();
        let out = format!("win32macros_rust::wide_bytes!({}){}", &c["lit"], &c["cast"]);
        assert_eq!(
            out,
            r#"win32macros_rust::wide_bytes!(b"L\0i\0b\0\0\0") as LPCWSTR"#
        );
        assert!(!SITE.is_match(&out));
    }
    #[test]
    fn scanner_control_bad_and_good() {
        let c = Corpus::new(
            PathBuf::from("."),
            vec![SourceFile::new("bad.rs".into(), BAD.into())],
        );
        let got = UnalignedWide.run(&c).unwrap();
        assert_eq!(got.len(), 1);
        assert!(
            got[0].starts_with("bad.rs:2: unaligned UTF-16 byte literal cast to a wide pointer;")
        );
        let g = Corpus::new(
            PathBuf::from("."),
            vec![SourceFile::new("good.rs".into(), GOOD.into())],
        );
        assert!(UnalignedWide.run(&g).unwrap().is_empty());
    }
}
