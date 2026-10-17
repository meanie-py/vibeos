use crate::{Corpus, Rule};
use regex::Regex;
pub struct DanglingTransmute;
struct Patterns {
    skip: Regex,
    nolint: Regex,
    transmute: Regex,
    let_bound: Regex,
    field: Regex,
}
impl Patterns {
    fn new() -> Self {
        let re = |s: &str| Regex::new(s).expect("valid dangling-transmute regex");
        let transmute = r"(?:::)?core::mem::transmute::<.*?>\(.*?\)\.as_(?:mut_)?ptr\(\)";
        Self {
            skip: re(r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$)"),
            nolint: re(r"//\s*nolint(?::\s*dangling-transmute\b|\s*$|:?\s*\()"),
            transmute: re(transmute),
            let_bound: re(&format!(r"\blet\b[^=]*=\s*{transmute}")),
            field: re(&format!(r"^\s*[A-Za-z_]\w*\s*:\s*{transmute}")),
        }
    }
}
fn scan(text: &str, p: &Patterns) -> Vec<(usize, &'static str)> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if p.nolint.is_match(line) {
            continue;
        }
        let Some(m) = p.transmute.find(line) else {
            continue;
        };
        if m.as_str().contains("&[") {
            continue;
        }
        if p.let_bound.is_match(line) {
            out.push((i + 1, "let-bound"));
        } else if p.field.is_match(line) {
            out.push((i + 1, "struct-field"));
        }
    }
    out
}
impl Rule for DanglingTransmute {
    fn name(&self) -> &'static str {
        "scan_dangling_transmute"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let p = Patterns::new();
        let mut files: Vec<_> = corpus.rust_files().collect();
        files.sort_by(|a, b| a.path.cmp(&b.path));
        let mut out = Vec::new();
        for f in files {
            if p.skip.is_match(&f.path.to_string_lossy()) {
                continue;
            }
            for (line, kind) in scan(&f.text, &p) {
                out.push(format!(
                    "{}:{line}: dangling-transmute [{kind}]",
                    f.path.display()
                ));
            }
        }
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escaping_shapes() {
        let p = Patterns::new();
        assert_eq!(scan("let p = core::mem::transmute::<[u8; 4], [u16; 2]>(bytes).as_ptr();\n Name: ::core::mem::transmute::<[u8; 4], [u16; 2]>(bytes).as_mut_ptr(),",&p),vec![(1,"let-bound"),(2,"struct-field")]);
    }
    #[test]
    fn immediate_call_references_and_suppression() {
        let p = Patterns::new();
        for text in ["foo(core::mem::transmute::<[u8; 4], [u16; 2]>(bytes).as_ptr());","let p = core::mem::transmute::<&[u8; 4], &[u16; 2]>(bytes).as_ptr();","let p = core::mem::transmute::<[u8; 4], [u16; 2]>(bytes).as_ptr(); // nolint: dangling-transmute reason"]{assert!(scan(text,&p).is_empty(),"{text}");}
    }
    #[test]
    fn line_boundaries_and_plain_transmute_not_matched() {
        let p = Patterns::new();
        assert!(scan(
            "let p = core::mem::transmute::<[u8;4], [u16;2]>\n(bytes).as_ptr();",
            &p
        )
        .is_empty());
        assert!(scan("let p = transmute::<[u8;4], [u16;2]>(bytes).as_ptr();", &p).is_empty());
    }
}
