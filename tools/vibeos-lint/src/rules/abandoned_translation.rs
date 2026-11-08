//! Detect a translation the model abandoned partway, recognised by its own narration.
//!
//! kernel32's NLS_EnumCalendarInfo shipped with "// I give up trying to write it from
//! scratch", "// Let me just use the anchor's code" and an empty `loop { match }` between
//! them: it compiled, and every EnumCalendarInfo call spun forever (Wine's locale test
//! hung there, 2026-11-08). The narration is the reliable signal, because the code it
//! leaves behind can be any shape. A comment that talks to itself about giving up,
//! restarting, or what it will "just" do instead is not a code comment, it is the model's
//! scratchpad, and the function around it needs rewriting from the C.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::OnceLock;

pub struct AbandonedTranslation;
impl Rule for AbandonedTranslation {
    fn name(&self) -> &'static str {
        "scan_abandoned_translation"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.rust_files() {
            if skip().is_match(&file.path.to_string_lossy()) {
                continue;
            }
            for (line, phrase) in scan_text(&file.text) {
                out.push(format!(
                    "{}:{line}: abandoned-translation narration {phrase:?}",
                    file.path.display()
                ));
            }
        }
        Ok(out)
    }
}

fn skip() -> &'static Regex {
    static P: OnceLock<Regex> = OnceLock::new();
    P.get_or_init(|| {
        crate::regexes::cached(r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/island_[a-z0-9_]+\.rs$|/vibeos-lint/)")
            .unwrap()
    })
}

fn narration() -> &'static Regex {
    static P: OnceLock<Regex> = OnceLock::new();
    P.get_or_init(|| {
        crate::regexes::cached(
            r"(?i)\b(I give up|let me (?:just|restart|rewrite|write it)|actually, (?:let me|I'll|I realize)|this is getting too (?:complex|complicated)|I'll (?:just )?(?:copy|use) the anchor)",
        )
        .unwrap()
    })
}

fn scan_text(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let Some(at) = line.find("//") else {
            continue;
        };
        if let Some(m) = narration().find(&line[at + 2..]) {
            out.push((i + 1, m.as_str().to_string()));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    const BAD: &str = "unsafe fn f() {\n    loop {\n        match s {\n            3 => {\n                // This is getting too complex. Let me just use the anchor's code.\n            }\n            _ => {}\n        }\n    }\n    // I give up trying to write it from scratch.\n}\n";
    const GOOD: &str = "unsafe fn f() {\n    // The C gives up after three retries; so do we.\n    // Let the caller decide: actually returning here matches Windows.\n    0\n}\n";
    #[test]
    fn scanner_control_bad_and_good_fixtures() {
        assert_eq!(
            scan_text(BAD),
            vec![
                (5, "This is getting too complex".into()),
                (10, "I give up".into())
            ]
        );
        assert!(scan_text(GOOD).is_empty());
    }
}
