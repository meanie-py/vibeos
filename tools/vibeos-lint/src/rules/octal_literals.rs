//! Detect C octal literals that Rust silently reads as decimal.
use crate::{lexical, Corpus, Rule};
use regex::Regex;
use std::sync::OnceLock;

pub struct OctalLiterals;
impl Rule for OctalLiterals {
    fn name(&self) -> &'static str {
        "scan_octal_literals"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.rust_files() {
            for (line_no, lit, decimal, octal) in scan_text(&file.text) {
                let c_value = octal_decimal(&lit[..suffix_start(&lit)]);
                out.push(format!(
                    "{}:{line_no}: OCTAL_AS_DECIMAL `{lit}` — Rust reads {decimal}, C reads {octal} ({c_value}). If this came from C, write {octal}.",
                    file.path.display()
                ));
            }
        }
        Ok(out)
    }
}
fn candidate() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"0[0-7]+(?:[iu](?:8|16|32|64|128|size))?").unwrap())
}
fn word(c: char) -> bool {
    static WORD: OnceLock<Regex> = OnceLock::new();
    WORD.get_or_init(|| Regex::new(r"^\w$").unwrap())
        .is_match(&c.to_string())
}
fn decimal_value(digits: &str) -> String {
    let value = digits.trim_start_matches('0');
    if value.is_empty() {
        "0".into()
    } else {
        value.into()
    }
}
fn octal_decimal(digits: &str) -> String {
    let mut decimal = vec![0u8];
    for ch in digits.bytes() {
        let mut carry = (ch - b'0') as u16;
        for d in decimal.iter_mut().rev() {
            let v = (*d as u16) * 8 + carry;
            *d = (v % 10) as u8;
            carry = v / 10;
        }
        while carry > 0 {
            decimal.insert(0, (carry % 10) as u8);
            carry /= 10;
        }
    }
    decimal.into_iter().map(|d| (b'0' + d) as char).collect()
}
fn octal_spelling(literal: &str) -> String {
    let digits = &literal[..suffix_start(literal)];
    let digits = digits.trim_start_matches('0');
    format!("0o{}", if digits.is_empty() { "0" } else { digits })
}
fn suffix_start(lit: &str) -> usize {
    lit.find(['i', 'u']).unwrap_or(lit.len())
}
fn scan_text(text: &str) -> Vec<(usize, String, String, String)> {
    let code = lexical::strip_noise(text);
    let mut out = Vec::new();
    for (line_idx, line) in code.split('\n').enumerate() {
        for m in candidate().find_iter(line) {
            let (start, end) = (m.start(), m.end());
            let prev = line[..start].chars().next_back();
            let next = line[end..].chars().next();
            if prev.is_some_and(|c| word(c) || matches!(c, '.' | '+' | '-'))
                || next.is_some_and(|c| word(c) || c == '.')
            {
                continue;
            }
            if start > 0 && line.as_bytes()[start - 1] == b'.' {
                continue;
            }
            let lit = m.as_str();
            let digits_end = suffix_start(lit);
            let digits = &lit[..digits_end];
            let dec = decimal_value(digits);
            let oct = octal_decimal(digits);
            if dec != oct {
                let full = lit.to_owned();
                let octal = octal_spelling(lit);
                out.push((line_idx + 1, full, dec, octal));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    fn lits(src: &str) -> Vec<String> {
        scan_text(src).into_iter().map(|x| x.1).collect()
    }
    #[test]
    fn shipped_bug_and_suffix() {
        assert_eq!(
            lits("(*buf).st_mode = (_S_IFREG | 0444) as u16;\n(*buf).st_mode |= 0222;"),
            vec!["0444", "0222"]
        );
        assert_eq!(lits("let mode: u32 = 0644u32;"), vec!["0644u32"]);
        let x = scan_text("let x = 010;");
        assert_eq!((&*x[0].2, &*x[0].3), ("10", "0o10"));
    }
    #[test]
    fn equivalent_and_non_octal_numbers_are_silent() {
        assert!(lits("let a=0;let b=00;let c=07;let d=007;").is_empty());
        assert!(lits("let a=0x1FE;let b=0o444;let c=0b0101;let d=0.5;let t=pair.0;").is_empty());
        assert!(lits("const X:f64=6.931e-01; const Y:f64=1.4e+00; const Z:f32=9e-07;").is_empty());
    }
    #[test]
    fn comments_strings_raw_strings_and_lifetimes() {
        assert!(lits("// 0444\n/* 0444 /* 0222 */ 0666 */ let x=1;").is_empty());
        assert!(lits("let a=\"http://0444\"; let b=b\"0640\\0\"; let c=r#\"0444\"#;").is_empty());
        assert_eq!(
            lits("fn f(s: &'static str) -> &'a u8 { s }\nlet mode=0444;"),
            vec!["0444"]
        );
        assert_eq!(lits(r"let c = '\''; let mode = 0444;"), vec!["0444"]);
    }
    #[test]
    fn lint_line_preserves_exact_python_format() {
        let corpus = Corpus::new(
            "/fixture".into(),
            vec![crate::SourceFile::new(
                "sdk/lib/rust/a.rs".into(),
                "let x = 0444;".into(),
            )],
        );
        assert_eq!(OctalLiterals.run(&corpus).unwrap(), vec![
            "sdk/lib/rust/a.rs:1: OCTAL_AS_DECIMAL `0444` — Rust reads 444, C reads 0o444 (292). If this came from C, write 0o444.".to_owned()
        ]);
    }
    #[test]
    fn lines_survive_multiline_noise_and_rendered_values() {
        let text = "/*\n\n\n*/\nlet s=\"a\\\nb\";\nlet mode=0444;\n";
        let got = scan_text(text);
        assert_eq!((got[0].0, got[0].1.as_str()), (7, "0444"));
        assert_eq!(&*got[0].2, "444");
        assert_eq!(&*got[0].3, "0o444");
    }
}
