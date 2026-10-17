use crate::{Corpus, Rule};
use regex::Regex;
pub struct ContainingRecordNull;
struct Patterns {
    skip: Regex,
    call: Regex,
    null_last: Regex,
    comments: Regex,
}
impl Patterns {
    fn new() -> Self {
        let re = |s| Regex::new(s).expect("valid containing-record-null regex");
        Self {
            skip: re(r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/island_[a-z0-9_]+\.rs$)"),
            call: re(r"\bCONTAINING_RECORD\w*(?:::<[^>]*>)?\s*\("),
            null_last: re(r"(?:core::ptr::|ptr::)?null(?:_mut)?\(\)\s*(?:as\s+[\w:]+\s*)?,?\s*$"),
            comments: re(r"(?s)//[^\n]*|/\*.*?\*/"),
        }
    }
}
fn args_text(text: &str, open: usize) -> Option<&str> {
    let mut depth = 0isize;
    for (j, c) in text.bytes().enumerate().skip(open) {
        if c == b'(' {
            depth += 1;
        } else if c == b')' {
            depth -= 1;
            if depth == 0 {
                return Some(&text[open + 1..j]);
            }
        }
    }
    None
}
fn scan(raw: &str, p: &Patterns) -> Vec<(usize, String)> {
    let raw = &raw[..raw.find("ORIGINAL C SOURCE").unwrap_or(raw.len())];
    let code = p.comments.replace_all(raw, |m: &regex::Captures<'_>| {
        m[0].as_bytes()
            .iter()
            .map(|c| if *c == b'\n' { '\n' } else { ' ' })
            .collect::<String>()
    });
    let mut out = Vec::new();
    for m in p.call.find_iter(&code) {
        let Some(args) = args_text(&code, m.end() - 1) else {
            continue;
        };
        if !args.contains(',') || code[..m.start()].trim_end().ends_with("fn") {
            continue;
        }
        if p.null_last.is_match(args.trim()) {
            let line = code[..m.start()].bytes().filter(|b| *b == b'\n').count() + 1;
            out.push((
                line,
                args.split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .chars()
                    .take(100)
                    .collect(),
            ));
        }
    }
    out
}
impl Rule for ContainingRecordNull {
    fn name(&self) -> &'static str {
        "scan_containing_record_null"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let p = Patterns::new();
        let mut rows = Vec::new();
        for f in corpus.rust_files() {
            if p.skip.is_match(&f.path.to_string_lossy()) {
                continue;
            }
            for (line, args) in scan(&f.text, &p) {
                rows.push((f.path.clone(), line, args));
            }
        }
        rows.sort();
        Ok(rows
            .into_iter()
            .map(|(path, line, args)| {
                format!("{}:{line}: containing-record-null ({args})", path.display())
            })
            .collect())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn null_fields_multiline_and_casts() {
        let p = Patterns::new();
        assert_eq!(
            scan(
                "CONTAINING_RECORD(p, T, core::ptr::null_mut() as PVOID,)",
                &p
            ),
            vec![(1, "p, T, core::ptr::null_mut() as PVOID,".to_owned())]
        );
        assert_eq!(
            scan("CONTAINING_RECORD_X::<T>(\n p, T, ptr::null()\n)", &p),
            vec![(1, "p, T, ptr::null()".to_owned())]
        );
    }
    #[test]
    fn offsets_declarations_comments_and_source_cutoff() {
        let p = Patterns::new();
        for text in [
            "CONTAINING_RECORD(p,T,offset_of!(T,field))",
            "fn CONTAINING_RECORD(p:T,q:T,null_mut()) {}",
            "/* CONTAINING_RECORD(p,T,null()) */",
            "// CONTAINING_RECORD(p,T,null())",
            "ORIGINAL C SOURCE\nCONTAINING_RECORD(p,T,null())",
        ] {
            assert!(scan(text, &p).is_empty(), "{text}");
        }
    }
    #[test]
    fn first_arg_only_is_ignored() {
        let p = Patterns::new();
        assert!(scan("CONTAINING_RECORD(null_mut())", &p).is_empty());
        assert!(scan("CONTAINING_RECORD(p,T,null_mut()", &p).is_empty());
    }
}
