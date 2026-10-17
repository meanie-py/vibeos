use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::LazyLock;
pub struct BindgenPaddingIndex;
fn scan_text(text: &str) -> Vec<usize> {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"__bindgen_padding_\d+\s*\[").unwrap());
    text.lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let code = line.split_once("//").map_or(line, |x| x.0);
            RE.is_match(code).then_some(i + 1)
        })
        .collect()
}
impl Rule for BindgenPaddingIndex {
    fn name(&self) -> &'static str {
        "scan_bindgen_padding_index"
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let generated = Regex::new(r"(^|/)(hal_)?bindgen_[a-z0-9_]+\.rs$").unwrap();
        let mut out = vec![];
        for f in c.rust_files() {
            let p = f.path.to_string_lossy();
            if p.contains("/target/") || generated.is_match(&p) {
                continue;
            }
            for n in scan_text(&f.text) {
                out.push(format!("{}:{}: [PADDING_INDEX] indexes a bindgen padding array -- read the member through its typed struct (_HEAD / _THROBJHEAD)", f.path.display(), n));
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn index_read_fires() {
        assert_eq!(scan_text("unsafe fn wnd_pti(p: PWND) -> PTHREADINFO {\n    (*p).head.__bindgen_padding_0[2] as PTHREADINFO\n}\n"),vec![2]);
    }
    #[test]
    fn address_of_padding_is_stable() {
        assert!(scan_text("let p = core::ptr::addr_of_mut!((*psi).__bindgen_padding_0) as *mut X;\nlet o = core::mem::offset_of!(tagSERVERINFO, __bindgen_padding_0);\n").is_empty());
    }
    #[test]
    fn comment_mention_is_skipped() {
        assert!(scan_text("// was head.__bindgen_padding_0[2] before the fix\n").is_empty());
    }
    #[test]
    fn generated_islands_are_excluded() {
        let re = Regex::new(r"(^|/)(hal_)?bindgen_[a-z0-9_]+\.rs$").unwrap();
        assert!(re.is_match("sdk/lib/rust/win32k-sys/src/bindgen_amd64.rs"));
        assert!(re.is_match("sdk/lib/rust/ntkrnl-sys/src/hal_bindgen_i386.rs"));
        assert!(!re.is_match("sdk/lib/rust/user32-rust/src/windows/message.rs"));
    }
}
