//! Narrow byte string passed where the matching C call uses a wide string.
use crate::{Corpus, Rule};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

pub struct NarrowVsWideLiteral;
impl Rule for NarrowVsWideLiteral {
    fn name(&self) -> &'static str {
        "scan_narrow_vs_wide_literal"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.rust_files() {
            let path = file.path.to_string_lossy();
            if patterns().skip.is_match(&path) {
                continue;
            }
            let Some(cut) = file.text.find("ORIGINAL C SOURCE") else {
                continue;
            };
            let c = c_comments_to_code(&file.text[cut..]);
            let mut wide: HashMap<(String, usize, usize), HashSet<String>> = HashMap::new();
            for (_, name, args) in calls(&c, true) {
                for (i, arg) in args.iter().enumerate() {
                    if let Some(cap) = patterns().wide.captures(arg.trim()) {
                        wide.entry((name.clone(), args.len(), i))
                            .or_default()
                            .insert(cap[1].to_owned());
                    }
                }
            }
            if wide.is_empty() {
                continue;
            }
            let rust = mask_line_comments(&file.text[..cut]);
            for (pos, name, args) in calls(&rust, false) {
                for (i, arg) in args.iter().enumerate() {
                    let Some(cap) = patterns().narrow.captures(arg) else {
                        continue;
                    };
                    let start = cap.get(0).unwrap().start();
                    if patterns().macro_prefix.is_match(&arg[..start]) {
                        continue;
                    }
                    let value = strip_nul(&cap[1]);
                    if wide
                        .get(&(name.clone(), args.len(), i))
                        .is_some_and(|s| s.contains(value))
                    {
                        let end = call_end(&rust, pos);
                        let start_char = rust[..pos].chars().count();
                        let end_char = rust[..end].chars().count();
                        let original_start = byte_at_char(&file.text, start_char);
                        let original_end = byte_at_char(&file.text, end_char);
                        if patterns()
                            .nolint
                            .is_match(&file.text[original_start..original_end])
                        {
                            continue;
                        }
                        let line = rust[..pos].bytes().filter(|b| *b == b'\n').count() + 1;
                        out.push(format!("{path}:{line}: narrow-vs-wide-literal {name} arg{i}: b\"{value}\" where C passes L\"...\""));
                    }
                }
            }
        }
        Ok(out)
    }
}
struct Patterns {
    skip: Regex,
    narrow: Regex,
    wide: Regex,
    nolint: Regex,
    macro_prefix: Regex,
    comments: Regex,
    call_rust: Regex,
    call_c: Regex,
    before_rust: Regex,
    before_c: Regex,
}
fn patterns() -> &'static Patterns {
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(|| Patterns {
        skip: Regex::new(
            r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/island_[a-z0-9_]+\.rs$)",
        )
        .unwrap(),
        narrow: Regex::new(r#"\bb"((?:\\.|[^"\\])*)"#).unwrap(),
        wide: Regex::new(r#"^\(?\s*L"((?:\\.|[^"\\])*)"#).unwrap(),
        nolint: Regex::new(r"nolint:\s*narrow-vs-wide\b").unwrap(),
        macro_prefix: Regex::new(r"!\s*\(\s*$").unwrap(),
        comments: Regex::new(r"//[^\n]*").unwrap(),
        call_rust: Regex::new(r"([A-Za-z_]\w*)\s*\(").unwrap(),
        call_c: Regex::new(r"([A-Za-z_]\w*)\s*\(").unwrap(),
        before_rust: Regex::new(r"[\w.:!]$").unwrap(),
        before_c: Regex::new(r"[\w.]$").unwrap(),
    })
}
fn byte_at_char(text: &str, character: usize) -> usize {
    text.char_indices()
        .nth(character)
        .map_or(text.len(), |(byte, _)| byte)
}
fn strip_nul(s: &str) -> &str {
    s.strip_suffix(r"\0").unwrap_or(s)
}
fn allowed_call(code: &str, start: usize, c: bool) -> bool {
    let before = &code[..start];
    if c {
        !patterns().before_c.is_match(before)
    } else {
        !patterns().before_rust.is_match(before)
    }
}
fn calls(code: &str, c: bool) -> Vec<(usize, String, Vec<String>)> {
    let re = if c {
        &patterns().call_c
    } else {
        &patterns().call_rust
    };
    let keywords = [
        "if", "while", "for", "match", "return", "switch", "sizeof", "fn", "loop",
    ];
    let mut out = Vec::new();
    for cap in re.captures_iter(code) {
        let whole = cap.get(0).unwrap();
        let start = whole.start();
        let name = &cap[1];
        if !allowed_call(code, start, c) || keywords.contains(&name) {
            continue;
        }
        if !c && code[whole.end()..].trim_start().starts_with('!') {
            continue;
        }
        let open = whole.end() - 1;
        if let Some(args) = split_args(code, open) {
            out.push((start, name.to_owned(), args));
        }
    }
    out
}
fn split_args(text: &str, open: usize) -> Option<Vec<String>> {
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    let mut start = open + 1;
    let mut out = Vec::new();
    for i in open..bytes.len() {
        match bytes[i] {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                depth -= 1;
                if depth == 0 {
                    let last = text[start..i].trim();
                    if !last.is_empty() {
                        out.push(last.to_owned());
                    }
                    return Some(out);
                }
            }
            b',' if depth == 1 => {
                out.push(text[start..i].trim().to_owned());
                start = i + 1;
            }
            _ => {}
        }
    }
    None
}
fn call_end(code: &str, pos: usize) -> usize {
    let Some(open) = code[pos..].find('(').map(|i| pos + i) else {
        return code.len();
    };
    let mut depth = 0i32;
    for (i, b) in code.bytes().enumerate().skip(open) {
        if b == b'(' {
            depth += 1;
        } else if b == b')' {
            depth -= 1;
            if depth == 0 {
                return i + 1;
            }
        }
    }
    code.len()
}
fn mask_line_comments(text: &str) -> String {
    patterns()
        .comments
        .replace_all(text, |c: &regex::Captures<'_>| {
            c[0].chars()
                .map(|ch| if ch == '\n' { '\n' } else { ' ' })
                .collect::<String>()
        })
        .into_owned()
}
fn c_comments_to_code(text: &str) -> String {
    text.lines()
        .map(|line| {
            let line = line.trim_start();
            if let Some(s) = line.strip_prefix("//") {
                s.strip_prefix(' ').unwrap_or(s)
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
#[cfg(test)]
fn scan_text(text: &str) -> Vec<(usize, String)> {
    let Some(cut) = text.find("ORIGINAL C SOURCE") else {
        return vec![];
    };
    let c = c_comments_to_code(&text[cut..]);
    let mut wide: HashMap<(String, usize, usize), HashSet<String>> = HashMap::new();
    for (_, name, args) in calls(&c, true) {
        for (i, a) in args.iter().enumerate() {
            if let Some(m) = patterns().wide.captures(a.trim()) {
                wide.entry((name.clone(), args.len(), i))
                    .or_default()
                    .insert(m[1].to_owned());
            }
        }
    }
    let rust = mask_line_comments(&text[..cut]);
    let mut out = Vec::new();
    for (pos, name, args) in calls(&rust, false) {
        for (i, a) in args.iter().enumerate() {
            if let Some(m) = patterns().narrow.captures(a) {
                let st = m.get(0).unwrap().start();
                if patterns().macro_prefix.is_match(&a[..st]) {
                    continue;
                }
                let val = strip_nul(&m[1]);
                if wide
                    .get(&(name.clone(), args.len(), i))
                    .is_some_and(|s| s.contains(val))
                    && !patterns().nolint.is_match(&text[pos..call_end(&rust, pos)])
                {
                    let line = rust[..pos].bytes().filter(|b| *b == b'\n').count() + 1;
                    out.push((
                        line,
                        format!("{name} arg{i}: b\"{val}\" where C passes L\"...\""),
                    ));
                }
            }
        }
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    const BAD: &str = "\
pub unsafe fn pattern(FilePattern: UNICODE_STRING) -> bool {
    RtlCompareMemory(FilePattern.Buffer as PVOID, b\"*.*\\0\".as_ptr() as PVOID, 6) == 6
}
// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//     if (RtlCompareMemory(FilePattern.Buffer, L\"*.*\", 6) == 6)
";
    const GOOD: &str = "\
pub unsafe fn pattern(FilePattern: UNICODE_STRING) -> bool {
    RtlCompareMemory(FilePattern.Buffer as PVOID, STAR_DOT_STAR_W.as_ptr() as PVOID, 6) == 6
        && RtlInitUnicodeString(&mut s, wstr!(b\"*.*\\0\")) == 0
        && DbgPrint(b\"hi\\0\".as_ptr()) == 0
}
// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//     if (RtlCompareMemory(FilePattern.Buffer, L\"*.*\", 6) == 6)
//     RtlInitUnicodeString(&s, L\"*.*\");
//     DbgPrint(\"hi\");
";
    #[test]
    fn flags_same_function_arity_and_position() {
        assert_eq!(
            scan_text(BAD),
            vec![(
                2,
                "RtlCompareMemory arg1: b\"*.*\" where C passes L\"...\"".into()
            )]
        );
        let corpus = Corpus::new(
            std::path::PathBuf::from("/fixture"),
            vec![crate::SourceFile::new("bad.rs".into(), BAD.into())],
        );
        assert_eq!(NarrowVsWideLiteral.run(&corpus).unwrap(), vec!["bad.rs:2: narrow-vs-wide-literal RtlCompareMemory arg1: b\"*.*\" where C passes L\"...\""]);
    }
    #[test]
    fn macro_literal_different_call_and_nolint_are_clean() {
        assert!(scan_text(GOOD).is_empty());
        let corpus = Corpus::new(
            std::path::PathBuf::from("/fixture"),
            vec![crate::SourceFile::new("good.rs".into(), GOOD.into())],
        );
        assert!(NarrowVsWideLiteral.run(&corpus).unwrap().is_empty());
    }
    #[test]
    fn suppression_is_on_the_call_span() {
        let s = BAD.replace("6) == 6", "6 /* nolint: narrow-vs-wide */) == 6");
        assert!(scan_text(&s).is_empty());
    }
}
