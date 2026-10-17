//! Detect casts of ATL CAtlList slot pointers instead of dereferencing the slot.
use crate::{Corpus, Rule, SourceSet};
use regex::Regex;
use std::sync::OnceLock;

pub struct AtlSlotCast;
const INPUTS: &[SourceSet] =
    &[SourceSet::tree("atl-slot-cast-rust", &["sdk/lib/rust"], &["rs"], &[]).with_inputs()];

fn slot_cast() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        crate::regexes::cached(r"\.Get(?:Next|Prev)\(\s*&mut\s+[^()]*\)\s+as\s+\*(?:mut|const)\b")
            .unwrap()
    })
}
fn skip_path() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| crate::regexes::cached(r"(/target/|/\.responses/|/atlclasses-rust/)").unwrap())
}

impl Rule for AtlSlotCast {
    fn name(&self) -> &'static str {
        "scan_atl_slot_cast"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut findings = Vec::new();
        let mut files: Vec<_> = corpus
            .sources("atl-slot-cast-rust")
            .filter(|file| file.path.extension().is_some_and(|ext| ext == "rs"))
            .collect();
        files.sort_by(|a, b| a.path.cmp(&b.path));
        for file in files {
            let rel = file.path.to_string_lossy();
            if skip_path().is_match(&format!("/{rel}")) {
                continue;
            }
            for (line, text) in file.text.lines().enumerate() {
                if slot_cast().is_match(text) && !text.contains("nolint: atl-slot-cast") {
                    findings.push(format!(
                        "{}:{}: GetNext/GetPrev result cast -- deref the slot instead",
                        file.path.display(),
                        line + 1
                    ));
                }
            }
        }
        Ok(findings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn scanner_control_atl_slot_cast_bad_and_good() {
        let bad = r#"
unsafe fn close_all(this_: *mut CMainWnd) {
    let mut pos: POSITION = (*this_).m_ViewList.GetHeadPosition();
    while !pos.is_null() {
        let console: *mut CConsoleWnd = (*this_).m_ViewList.GetNext(&mut pos) as *mut CConsoleWnd;
        (*console).__base0.SendMessageW__0(WM_CLOSE, 0, 0);
    }
}
"#;
        let good = r#"
unsafe fn close_all(this_: *mut CMainWnd) {
    let mut pos: POSITION = (*this_).m_ViewList.GetHeadPosition();
    while !pos.is_null() {
        let console: *mut CConsoleWnd = *(*this_).m_ViewList.GetNext(&mut pos);
        (*console).__base0.SendMessageW__0(WM_CLOSE, 0, 0);
    }
}
"#;
        let corpus = |text: &str| {
            Corpus::new(
                PathBuf::from("/repo"),
                vec![crate::SourceFile::new("mmc.rs".into(), text.into())],
            )
        };
        let result = AtlSlotCast.run(&corpus(bad)).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            "mmc.rs:5: GetNext/GetPrev result cast -- deref the slot instead"
        );
        assert!(AtlSlotCast.run(&corpus(good)).unwrap().is_empty());
    }

    #[test]
    fn atl_slot_cast_honors_same_line_nolint_and_excluded_paths() {
        let waived = "(*x).GetPrev(&mut pos) as *const Item; // nolint: atl-slot-cast\n";
        let unwaived = "(*x).GetPrev(&mut pos) as *const Item;\n";
        let corpus = Corpus::new(
            PathBuf::from("/repo"),
            vec![
                crate::SourceFile::new("sdk/lib/rust/a.rs".into(), waived.into()),
                crate::SourceFile::new("sdk/lib/rust/atlclasses-rust/a.rs".into(), unwaived.into()),
            ],
        );
        assert!(AtlSlotCast.run(&corpus).unwrap().is_empty());
    }
}
