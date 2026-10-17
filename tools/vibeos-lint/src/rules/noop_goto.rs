//! C `goto` translated into a labeled block that only breaks out of itself.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::OnceLock;

pub struct NoopGoto;

impl Rule for NoopGoto {
    fn name(&self) -> &'static str {
        "scan_noop_goto"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.rust_files() {
            let path = file.path.to_string_lossy();
            if patterns().skip.is_match(&path) {
                continue;
            }
            let code = file.masked(&patterns().comments, None, false);
            for cap in patterns().noop.captures_iter(&code) {
                if cap[1] != cap[2] {
                    continue;
                }
                let m = cap.get(0).unwrap();
                out.push(format!(
                    "{path}:{}: noop-goto '{}'",
                    code[..m.start()].bytes().filter(|b| *b == b'\n').count() + 1,
                    cap.get(1).unwrap().as_str()
                ));
            }
        }
        Ok(out)
    }
}
struct Patterns {
    noop: Regex,
    comments: Regex,
    skip: Regex,
}
fn patterns() -> &'static Patterns {
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(|| Patterns {
        noop: Regex::new(r"'(\w+):\s*\{\s*break\s+'(\w+)\s*;\s*\}").unwrap(),
        comments: Regex::new(r"(?s)//[^\n]*|/\*.*?\*/").unwrap(),
        skip: Regex::new(
            r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/island_[a-z0-9_]+\.rs$)",
        )
        .unwrap(),
    })
}

#[cfg(test)]
fn scan(text: &str) -> Vec<(usize, String)> {
    let code = patterns()
        .comments
        .replace_all(text, |c: &regex::Captures<'_>| {
            c[0].chars()
                .map(|ch| if ch == '\n' { '\n' } else { ' ' })
                .collect::<String>()
        });
    patterns()
        .noop
        .captures_iter(&code)
        .filter(|c| c[1] == c[2])
        .map(|c| {
            let m = c.get(0).unwrap();
            (
                code[..m.start()].bytes().filter(|b| *b == b'\n').count() + 1,
                c[1].to_owned(),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceFile;
    use std::path::PathBuf;
    const BAD: &str = "\
pub unsafe fn f(ramdisk: bool) -> i32 {
    if ramdisk {
        'quit: {
            break 'quit;
        }
    }
    parse_controller()
}
";
    const GOOD: &str = "\
// The fix replaced `'quit: { break 'quit; }` with a block that spans the parse.
pub unsafe fn f(ramdisk: bool) -> i32 {
    'quit: {
        if ramdisk {
            break 'quit;
        }
        parse_controller();
    }
    0
}
";
    #[test]
    fn scanner_control_bad_and_good_fixtures() {
        assert_eq!(scan(BAD), vec![(3, "quit".into())]);
        assert!(scan(GOOD).is_empty());
        let corpus = Corpus {
            root: PathBuf::from("/fixture"),
            files: vec![SourceFile::new("bad.rs".into(), BAD.into())],
        };
        assert_eq!(
            NoopGoto.run(&corpus).unwrap(),
            vec!["bad.rs:3: noop-goto 'quit'"]
        );
    }
    #[test]
    fn comments_and_a_different_break_label_are_clean() {
        let s = "// 'fake: { break 'fake; }\n'a: { break 'b; }\n'live: { if x { break 'live; } }";
        assert!(scan(s).is_empty());
    }
    #[test]
    fn regex_backreference_semantics_require_the_same_label() {
        assert!(scan("'a: { break 'b; }").is_empty());
    }
}
