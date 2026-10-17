//! Bare i386 cdecl aliases that point at stdcall bodies with stack arguments.
use crate::{integers::decimal, Corpus, Rule};
use regex::Regex;
use std::sync::OnceLock;

pub struct CdeclAlias;
impl Rule for CdeclAlias {
    fn name(&self) -> &'static str {
        "scan_cdecl_alias"
    }
    fn needs_cdecl_sources(&self) -> bool {
        true
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.cdecl_files() {
            if !file.text.contains(".set") && !file.text.contains(".equ") {
                continue;
            }
            for (line, bare, n) in scan(&file.text)? {
                out.push(format!(
                    "{}:{line}: cdecl-alias {bare}@{n}",
                    file.path.display()
                ));
            }
        }
        Ok(out)
    }
}
struct Patterns {
    alias: Regex,
}
fn patterns() -> &'static Patterns {
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(|| Patterns {
        alias: Regex::new(r"\.(?:set|equ)\s+_(\w+)\s*,\s*_(\w+)@(\d+)\b").unwrap(),
    })
}
fn scan(text: &str) -> Result<Vec<(usize, String, String)>, String> {
    let mut out = Vec::new();
    for (line_no, line) in text.lines().enumerate() {
        for c in patterns().alias.captures_iter(line) {
            let n = decimal(&c[3])?;
            if c[1] == c[2] && n != "0" {
                out.push((line_no + 1, c[1].to_owned(), n));
            }
        }
    }
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceFile;
    use std::path::PathBuf;
    const BAD: &str = r#"
__asm__(".globl _KeInitializeSpinLock\n"
        ".set _KeInitializeSpinLock, _KeInitializeSpinLock@4\n");
"#;
    const GOOD: &str = r#"
__asm__(".globl _KeGetCurrentProcessorNumber\n"
        ".set _KeGetCurrentProcessorNumber, _KeGetCurrentProcessorNumber@0");
__asm__(".globl _KeInitializeSpinLock\n_KeInitializeSpinLock:\n"
        " pushl 4(%esp)\n call _KeInitializeSpinLock@4\n ret\n");
"#;
    #[test]
    fn unbounded_and_unicode_decimal_suffixes() {
        let huge = "184467440737095516160000000000000000000000";
        assert_eq!(
            scan(&format!(".set _X, _X@000{huge}")).unwrap(),
            vec![(1, "X".into(), huge.into())]
        );
        assert_eq!(
            scan(".equ _X, _X@００١٢٤").unwrap(),
            vec![(1, "X".into(), "124".into())]
        );
        assert!(scan(".set _X, _X@٠０0").unwrap().is_empty());
    }
    #[test]
    fn bad_alias_is_reported_but_zero_and_real_thunk_are_clean() {
        assert_eq!(
            scan(BAD).unwrap(),
            vec![(3, "KeInitializeSpinLock".into(), "4".into())]
        );
        assert!(scan(GOOD).unwrap().is_empty());
        let corpus = Corpus {
            root: PathBuf::from("/fixture"),
            files: vec![SourceFile::new("bridge.c".into(), BAD.into())],
        };
        assert_eq!(
            CdeclAlias.run(&corpus).unwrap(),
            vec!["bridge.c:3: cdecl-alias KeInitializeSpinLock@4"]
        );
        let clean = Corpus {
            root: PathBuf::from("/fixture"),
            files: vec![SourceFile::new("bridge.c".into(), GOOD.into())],
        };
        assert!(CdeclAlias.run(&clean).unwrap().is_empty());
    }
    #[test]
    fn distinct_alias_target_does_not_match() {
        assert!(scan(".set _alias, _body@4").unwrap().is_empty());
    }
}
