//! Check hand-encoded wide arrays and byte strings against comments and runtime use.
use crate::{regexes::cached, Corpus, Rule, SourceSet};
use regex::Regex;
use std::sync::OnceLock;

pub struct WideLiterals;
const INPUTS: &[SourceSet] =
    &[
        SourceSet::tree("wide-literals-rust", &["sdk/lib/rust"], &["rs"], &[])
            .with_pruning(&["target", ".responses"])
            .with_inputs(),
    ];
impl Rule for WideLiterals {
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn name(&self) -> &'static str {
        "scan_wide_literals"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for f in corpus.sources("wide-literals-rust") {
            if f.path.to_string_lossy().contains("/target/") {
                continue;
            }
            out.extend(wide_facts(&f.text, &f.path.to_string_lossy()));
        }
        Ok(out)
    }
}
const ELEM: &str = r"(?:u16|WCHAR|TCHAR|OLECHAR|wchar_t|MSVCRT_wchar_t)";
const WIDE_PTR: &str = r"(?:u16|wchar_t|WCHAR|MSVCRT_wchar_t|[LP]{1,2}C?W(?:STR|CHAR))";
fn re(s: &str) -> Regex {
    cached(s).expect("constant regex compiles")
}
fn decl() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(||re(&format!(r"(?m)^[ \t]*(?:pub(?:\([^)]*\))?[ \t]+)?(?:static|const)[ \t]+(?:mut[ \t]+)?([A-Za-z_]\w*)[ \t]*:[ \t]*\[[ \t]*(?P<elem>{ELEM})[ \t]*;[ \t]*(\d+)[ \t]*\][ \t]*=[ \t]*\[")))
}
fn elem() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(||re(&format!(r"b?'(?P<char>\\.|[^'])'[ \t]*as[ \t]*{ELEM}|0[xX](?P<hex>[0-9a-fA-F]+)(?:[ui](?:8|16|32|64|128|size))?|(?:^|[\s,])(?P<decimal>\d+)(?:[ui](?:8|16|32|64|128|size))?\b")))
}
fn intent() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| re(r#"L"((?:[^"\\]|\\.)*)""#))
}
fn bytecast() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(||re(&format!(r#"b"((?:[^"\\]|\\.)*)"\s*(?:\.as_ptr\(\)|\.as_mut_ptr\(\))\s*as\s*(?:\*\s*(?:const|mut)\s*{WIDE_PTR}|{WIDE_PTR})\b"#)))
}
fn bytetransmute() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(||re(r#"transmute(?:::<[^>]*>)?\([ \t\r\n]*b"((?:[^"\\]|\\.)*)"[ \t\r\n]*(?:\.as_ptr\(\)|\.as_mut_ptr\(\))"#))
}
fn any_bytestr() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| re(r#"b"((?:[^"\\]|\\.)*)""#))
}
fn nul_comment() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| re(r"nul-terminated|null-terminated|null terminated"))
}
fn claims_nul(block: &[String]) -> bool {
    nul_comment().is_match(&block.join(" ").to_lowercase())
}
fn decode_elem(m: &regex::Captures<'_>) -> Option<u32> {
    if let Some(s) = m.name("char") {
        let raw = s.as_str();
        if raw.starts_with('\\') {
            return Some(match raw {
                r"\\" => 92,
                r"\'" => 39,
                r#"\""# => 34,
                r"\n" => 10,
                r"\r" => 13,
                r"\t" => 9,
                r"\0" => 0,
                _ => return None,
            });
        }
        return raw.chars().next().map(|c| c as u32);
    }
    if let Some(s) = m.name("hex") {
        return u32::from_str_radix(s.as_str(), 16).ok();
    }
    m.name("decimal")?.as_str().parse().ok()
}
fn array_body(text: &str, open: usize) -> Option<(&str, usize)> {
    let bytes = text.as_bytes();
    let mut d = 0usize;
    for i in open..bytes.len() {
        match bytes[i] {
            b'[' => d += 1,
            b']' => {
                d -= 1;
                if d == 0 {
                    return Some((&text[open + 1..i], i));
                }
            }
            _ => {}
        }
    }
    None
}
fn comment_block(text: &str, start: usize) -> Vec<String> {
    let before = &text[..start];
    let mut lines: Vec<_> = before.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    let mut out = Vec::new();
    for line in lines.into_iter().rev() {
        let x = line.trim();
        if x.starts_with("//") {
            out.push(x.to_owned());
        } else {
            break;
        }
    }
    out.reverse();
    out
}
fn intent_for(text: &str, start: usize) -> Vec<String> {
    let mut out = Vec::new();
    for line in comment_block(text, start) {
        for c in intent().captures_iter(&line) {
            let Some(s) = c.get(1).map(|m| m.as_str()) else {
                continue;
            };
            if s.contains("...") || s.is_empty() {
                continue;
            }
            let mut decoded = String::new();
            let mut it = s.chars().peekable();
            let mut bad = false;
            while let Some(ch) = it.next() {
                if ch == '\\' {
                    let Some(n) = it.next() else {
                        bad = true;
                        break;
                    };
                    match n {
                        '\\' => decoded.push('\\'),
                        '"' => decoded.push('"'),
                        'n' => decoded.push('\n'),
                        'r' => decoded.push('\r'),
                        't' => decoded.push('\t'),
                        '0' => decoded.push('\0'),
                        _ => {
                            bad = true;
                            break;
                        }
                    }
                } else {
                    decoded.push(ch);
                }
            }
            if !bad {
                out.push(decoded.trim_end_matches('\0').to_owned());
            }
        }
    }
    out.retain(|s| !s.is_empty());
    out
}
fn unescape_bytes(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c as u32 as u8);
            continue;
        }
        let n = it.next()?;
        match n {
            '0' => out.push(0),
            'n' => out.push(10),
            'r' => out.push(13),
            't' => out.push(9),
            '\\' => out.push(92),
            '"' => out.push(34),
            '\'' => out.push(39),
            'x' => {
                let a = it.next()?.to_digit(16)?;
                let b = it.next()?.to_digit(16)?;
                out.push((a * 16 + b) as u8)
            }
            _ => return None,
        }
    }
    Some(out)
}
fn blank_line_comments(text: &str) -> String {
    let mut b = text.as_bytes().to_vec();
    let mut i = 0;
    while i + 1 < b.len() {
        if b[i] == b'"' {
            i += 1;
            while i < b.len() {
                if b[i] == b'\\' {
                    i += 2;
                } else if b[i] == b'"' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
            continue;
        }
        if b[i] == b'/' && b[i + 1] == b'/' {
            while i < b.len() && b[i] != b'\n' {
                b[i] = b' ';
                i += 1;
            }
            continue;
        }
        i += 1;
    }
    String::from_utf8(b).expect("comment masking preserves UTF-8")
}
fn byte_findings(text: &str, rel: &str) -> Vec<String> {
    let mut out = Vec::new();
    let stripped = blank_line_comments(text);
    for re_ in [bytecast(), bytetransmute()] {
        for m in re_.captures_iter(&stripped) {
            let raw = unescape_bytes(m.get(1).unwrap().as_str());
            let Some(raw) = raw else { continue };
            let line = text[..m.get(0).unwrap().start()]
                .bytes()
                .filter(|b| *b == b'\n')
                .count()
                + 1;
            let shown = m
                .get(0)
                .unwrap()
                .as_str()
                .split(".as_")
                .next()
                .unwrap_or("");
            if raw.len() % 2 == 1 {
                out.push(format!("{rel}:{line}: {shown} is {} bytes — an ODD length cannot be whole u16 code units",raw.len()));
            } else if !raw.ends_with(&[0, 0]) {
                out.push(format!("{rel}:{line}: {shown} has no u16 NUL terminator — a wide NUL is TWO zero bytes and Rust does not add them"));
            } else if raw.len() > 4 && raw[1..raw.len() - 2].iter().step_by(2).all(|x| *x != 0) {
                out.push(format!("{rel}:{line}: {shown} is 8-bit text cast to a u16 pointer — every second byte must be 0 for ASCII UTF-16LE, none is"));
            } else if (0..raw.len().saturating_sub(2))
                .step_by(2)
                .any(|i| raw.get(i) == Some(&0) && raw.get(i + 1) == Some(&0))
            {
                out.push(format!("{rel}:{line}: {shown} has an INTERIOR u16 NUL — the string ends there; a double-interleaved literal decodes to its first character"));
            }
        }
    }
    out
}
fn source_text_findings(text: &str, rel: &str) -> Vec<String> {
    let mut out = Vec::new();
    for m in any_bytestr().captures_iter(&blank_line_comments(text)) {
        let raw = unescape_bytes(m.get(1).unwrap().as_str());
        let Some(raw) = raw else { continue };
        let mut end = raw.len();
        while end > 0 && raw[end - 1] == 0 {
            end -= 1;
        }
        let body = &raw[..end];
        if body.len() >= 3 && body.starts_with(b"L\"") && body.ends_with(b"\"") {
            let line = text[..m.get(0).unwrap().start()]
                .bytes()
                .filter(|b| *b == b'\n')
                .count()
                + 1;
            out.push(format!("{rel}:{line}: {} carries the C SOURCE TEXT `L\"...\"`, not the string — the `L` and both quotes are data at runtime",m.get(0).unwrap().as_str()));
        }
    }
    out
}
fn enclosing_callee(text: &str, pos: usize) -> Option<String> {
    let b = text.as_bytes();
    let mut d = 0i32;
    let mut i = pos;
    while i > 0 {
        i -= 1;
        match b[i] {
            b')' | b']' | b'}' => d += 1,
            b'(' | b'[' | b'{' => {
                if d == 0 {
                    if b[i] != b'(' {
                        return None;
                    }
                    let before = &text[..i];
                    return callee_head()
                        .captures(before)
                        .and_then(|c| c.get(1))
                        .map(|m| m.as_str().to_owned());
                }
                d -= 1;
            }
            b';' if d == 0 => return None,
            _ => {}
        }
    }
    None
}
fn callee_head() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| re(r"(\w+)\s*!?\s*$"))
}
fn bounded() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(||re(r"^(?:wcsncmp|_wcsnicmp|wcsnicmp|strncmp|_strnicmp|memcmp|RtlCompareMemory|RtlEqualMemory|memcpy|memmove|RtlCopyMemory|RtlMoveMemory|copy_nonoverlapping|copy_from_slice|RtlCopyUnicodeString|wcsncpy|wcsncat)$"))
}
fn unterminated(text: &str, rel: &str, decls: &[regex::Captures<'_>]) -> Vec<String> {
    let mut out = Vec::new();
    for m in decls {
        let name = m.get(1).unwrap().as_str();
        if decls
            .iter()
            .filter(|x| x.get(1).unwrap().as_str() == name)
            .count()
            > 1
        {
            continue;
        }
        let Some((body, _)) = array_body(text, m.get(0).unwrap().end() - 1) else {
            continue;
        };
        let vals: Vec<_> = elem()
            .captures_iter(body)
            .map(|c| decode_elem(&c))
            .collect();
        let count = m
            .get(3)
            .unwrap()
            .as_str()
            .parse::<usize>()
            .unwrap_or(usize::MAX);
        if vals.is_empty() || vals.iter().any(Option::is_none) || vals.len() != count {
            continue;
        }
        let vals: Vec<u32> = vals.into_iter().map(Option::unwrap).collect();
        if vals.last() == Some(&0)
            || vals.len() < 3
            || !vals.iter().all(|v| (0x20..0x7f).contains(v))
        {
            continue;
        }
        let size_re = re(&format!(
            r"size_of_val\(&(?:mut\s+)?{}\b|\b{}\.len\(\)",
            regex::escape(name),
            regex::escape(name)
        ));
        if size_re.is_match(text) {
            continue;
        }
        let use_re = re(&format!(
            r"\b{}\s*\.\s*as_(?:mut_)?ptr\s*\(",
            regex::escape(name)
        ));
        for u in use_re.find_iter(text) {
            let from = text[..u.start()]
                .char_indices()
                .rev()
                .nth(199)
                .map(|(i, _)| i)
                .unwrap_or(0);
            let before = &text[from..u.start()];
            if re(r"Buffer\s*[:=]\s*$").is_match(before) {
                continue;
            }
            if enclosing_callee(text, u.start()).is_some_and(|c| bounded().is_match(&c)) {
                continue;
            }
            let decoded: String = vals.iter().filter_map(|v| char::from_u32(*v)).collect();
            let line = text[..m.get(0).unwrap().start()]
                .bytes()
                .filter(|b| *b == b'\n')
                .count()
                + 1;
            out.push(format!("{rel}:{line}: {name} [{}; {}] decodes to \"{decoded}\" with NO NUL terminator and is passed by pointer to something that reads until one",m.name("elem").unwrap().as_str(),m.get(3).unwrap().as_str()));
            break;
        }
    }
    out
}
fn wide_facts(text: &str, rel: &str) -> Vec<String> {
    let decls: Vec<_> = decl().captures_iter(text).collect();
    let mut out = byte_findings(text, rel);
    out.extend(source_text_findings(text, rel));
    out.extend(unterminated(text, rel, &decls));
    for m in &decls {
        let name = m.get(1).unwrap().as_str();
        let elemname = m.name("elem").unwrap().as_str();
        let declared = m
            .get(3)
            .unwrap()
            .as_str()
            .parse::<usize>()
            .unwrap_or(usize::MAX);
        let Some((body, _)) = array_body(text, m.get(0).unwrap().end() - 1) else {
            continue;
        };
        let vals: Vec<_> = elem()
            .captures_iter(body)
            .map(|c| decode_elem(&c))
            .collect();
        if vals.iter().any(Option::is_none) || vals.len() != declared {
            continue;
        }
        let vals: Vec<u32> = vals.into_iter().map(Option::unwrap).collect();
        let intents = intent_for(text, m.get(0).unwrap().start());
        if intents.is_empty() {
            continue;
        }
        let decoded: String = vals.iter().filter_map(|v| char::from_u32(*v)).collect();
        let actual = decoded.trim_end_matches('\0');
        let nul = decoded.ends_with('\0');
        if intents.iter().any(|w| actual == w)
            && (nul || !claims_nul(&comment_block(text, m.get(0).unwrap().start())))
        {
            continue;
        }
        let best = intents
            .iter()
            .min_by_key(|w| w.chars().count().abs_diff(actual.chars().count()))
            .unwrap();
        let line = text[..m.get(0).unwrap().start()]
            .bytes()
            .filter(|b| *b == b'\n')
            .count()
            + 1;
        if best.starts_with(actual) && actual.chars().count() < best.chars().count() {
            let dropped = best.strip_prefix(actual).unwrap_or("");
            out.push(format!("{rel}:{line}: {name} [{elemname}; {declared}] is TRUNCATED — decodes to \"{actual}\" but the comment says \"{best}\" (dropped {}; need {} elements)",python_repr(dropped),best.chars().count()+1));
        } else if !nul {
            out.push(format!("{rel}:{line}: {name} [{elemname}; {declared}] decodes to \"{actual}\" with NO NUL terminator (comment: \"{best}\")"));
        } else {
            out.push(format!("{rel}:{line}: {name} [{elemname}; {declared}] decodes to \"{actual}\" but the comment says \"{best}\""));
        }
    }
    out
}
fn coverage_counts(text: &str) -> (usize, usize) {
    let mut seen = 0;
    let mut compared = 0;
    for m in decl().captures_iter(text) {
        let Some((body, _)) = array_body(text, m.get(0).unwrap().end() - 1) else {
            continue;
        };
        seen += 1;
        let vals: Vec<_> = elem()
            .captures_iter(body)
            .map(|c| decode_elem(&c))
            .collect();
        if vals.iter().any(Option::is_none)
            || vals.len()
                != m.get(3)
                    .unwrap()
                    .as_str()
                    .parse::<usize>()
                    .unwrap_or(usize::MAX)
        {
            continue;
        }
        let intents = intent_for(text, m.get(0).unwrap().start());
        if intents.is_empty() {
            continue;
        }
        let wants: Vec<_> = intents
            .iter()
            .map(|x| unescape_intent(x).trim_end_matches('\0').to_owned())
            .filter(|x| !x.is_empty())
            .collect();
        if !wants.is_empty() {
            compared += 1;
        }
    }
    (seen, compared)
}
fn unescape_intent(s: &str) -> String {
    let mut out = String::new();
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match it.next() {
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('0') => out.push('\0'),
            Some(_) => return String::new(),
            None => return String::new(),
        }
    }
    out
}
fn python_repr(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::from(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if c.is_control() => out.push_str(&format!("\\x{:02x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn intent_array_positive_negative() {
        let good="// L\"System\\\\CurrentControlSet\\\\Services\\0\" NUL-terminated\nstatic KEY: [WCHAR; 34] = [0x53, 0x79, 0x73, 0x74, 0x65, 0x6D, 0x5C, 0x43, 0x75, 0x72, 0x72, 0x65, 0x6E, 0x74, 0x43, 0x6F, 0x6E, 0x74, 0x72, 0x6F, 0x6C, 0x53, 0x65, 0x74, 0x5C, 0x53, 0x65, 0x72, 0x76, 0x69, 0x63, 0x65, 0x73, 0];";
        assert!(
            wide_facts(good, "a.rs").is_empty(),
            "findings: {:?}",
            wide_facts(good, "a.rs")
        );
        let bad = "// L\"Services\" NUL-terminated\nstatic KEY: [WCHAR; 3] = [83, 101, 114];";
        assert!(wide_facts(bad, "a.rs")
            .iter()
            .any(|x| x.contains("TRUNCATED")));
    }
    #[test]
    fn python_motivating_services_key_truncation_exact_fixture() {
        let src = r#"// L"System\\CurrentControlSet\\Services" — NUL-terminated.
static SERVICES_KEY: [u16; 33] = [
b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16, b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, b'S' as u16, b'e' as u16, b't' as u16, b'\\' as u16, b'S' as u16, b'e' as u16, b'r' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, 0,
];
"#;
        let out = wide_facts(src, "t.rs");
        assert_eq!(out.len(), 1);
        assert!(
            out[0].contains("TRUNCATED")
                && out[0].contains("dropped 's'")
                && out[0].contains("need 34 elements"),
            "finding: {:?}",
            out
        );
    }
    #[test]
    fn bytecast_shapes_and_false_positives() {
        assert!(
            byte_findings("let x = b\"abc\".as_ptr() as *const u16;", "x.rs")
                .iter()
                .any(|x| x.contains("8-bit text") || x.contains("ODD length"))
        );
        assert!(
            byte_findings("let x = b\"a\".as_ptr() as *const u16;", "x.rs")
                .iter()
                .any(|x| x.contains("ODD length"))
        );
        assert!(
            byte_findings("let x = b\"a\\0b\\0\".as_ptr() as *const u16;", "x.rs")
                .iter()
                .any(|x| x.contains("no u16 NUL"))
        );
        assert!(byte_findings("// b\"abc\".as_ptr() as *const u16\n", "x.rs").is_empty());
    }
    #[test]
    fn termination_use_and_bounded_callee() {
        let bad="static NAME: [u16; 3] = [65,66,67];\nfn f(){ RtlInitUnicodeString(&mut u, NAME.as_ptr()); }";
        assert!(wide_facts(bad, "x.rs")
            .iter()
            .any(|x| x.contains("passed by pointer")));
        let good = "static NAME: [u16; 3] = [65,66,67];\nfn f(){ wcsncmp(NAME.as_ptr(), q, 3); }";
        assert!(wide_facts(good, "x.rs").is_empty());
    }
    #[test]
    fn scanner_controls_from_python_fixtures() {
        assert!(wide_facts("// L\"Services\" — NUL-terminated.\nstatic K: [u16; 8] = [83,101,114,118,105,99,101,115];", "x.rs")[0].contains("NO NUL"));
        assert!(wide_facts(
            "// L\"Widget\"\nstatic K: [u16; 7] = [87,111,100,103,101,116,0];",
            "x.rs"
        )[0]
        .contains("Wodget"));
        assert!(wide_facts("// L\"\\\\??\\\\\" — 4 wide chars, no null needed (Length == MaximumLength)\nstatic P: [u16; 4] = [0x5C,0x3F,0x3F,0x5C];", "x.rs").is_empty(), "findings: {:?}", wide_facts("// L\"\\\\??\\\\\" — 4 wide chars, no null needed (Length == MaximumLength)\nstatic P: [u16; 4] = [0x5C,0x3F,0x3F,0x5C];", "x.rs"));
        assert!(wide_facts(
            "// UTF-16 spelling L\"...\", NUL-terminated\nconst ID: [u16; 3] = [65,66,0];",
            "x.rs"
        )
        .is_empty());
        assert!(wide_facts(
            "// L\"\\\\Registry\\\\Machine\\\\...\\\\Version\"\nconst K: [u16; 2] = [65,0];",
            "x.rs"
        )
        .is_empty());
        assert!(wide_facts(
            "// L\"RXACT\\0\" / L\"Log\\0\"\nstatic K: [u16; 6] = [82,88,65,67,84,0];",
            "x.rs"
        )
        .is_empty());
        assert!(wide_facts("// L\"AB\"\nconst K: [u16; 4] = [65,66,0,0];", "x.rs").is_empty());
        assert!(wide_facts("// L\"DependOnService\" / L\"DependOnGroup\"\nstatic V: [u16; 14] = [68,101,112,101,110,100,79,110,71,114,111,117,112,0];", "x.rs").is_empty());
        assert!(wide_facts(
            "// L\"Something\"\n\nstatic K: [u16; 3] = [65,66,0];",
            "x.rs"
        )
        .is_empty());
        assert!(
            byte_findings(
                "wcscat(cmd, b\" /c \\0\".as_ptr() as *const wchar_t);",
                "x.rs"
            )
            .len()
                == 1
        );
        assert!(byte_findings(
            "env = _wgetenv(b\"P\\0A\\0T\\0H\\0\\0\".as_ptr() as *const wchar_t);",
            "x.rs"
        )
        .iter()
        .any(|x| x.contains("ODD")));
        assert!(
            byte_findings(
                "g(b\"\\\\Device\\\\CdRom\\0\".as_ptr() as LPCWSTR);",
                "x.rs"
            )
            .len()
                == 1
        );
        assert!(
            byte_findings(
                "let p = core::mem::transmute(b\"All Users\\0\".as_ptr());",
                "x.rs"
            )
            .len()
                == 1
        );
        assert!(byte_findings(
            "// replacing incorrect b\"...\".as_ptr() as *mut u16 casts\n",
            "x.rs"
        )
        .is_empty());
        assert!(byte_findings("puts(b\"hello\\0\".as_ptr() as *const i8);", "x.rs").is_empty());
        assert!(
            source_text_findings(
                "swprintf(b.as_mut_ptr(), b\"L\\\"%u\\\"\\0\".as_ptr() as *const _, n);",
                "x.rs"
            )
            .len()
                == 1
        );
        assert!(
            source_text_findings("f(b\"L\\\"unclosed\\0\".as_ptr() as LPCWSTR);", "x.rs")
                .is_empty()
        );
        assert!(
            source_text_findings("// swprintf(szBuffer, b\"L\\\"%u\\\"\\0\", n);\n", "x.rs")
                .is_empty()
        );
    }
    #[test]
    fn original_suite_coverage_and_remaining_unterminated_controls() {
        // Python's L"..." intent requires a closing quote. This source comment
        // ends in an escaped quote and is not a complete wide string annotation.
        let escaped_closer = "// L\"\\\"\nstatic BACKSLASH_W: [u16; 2] = [b'\\\\' as u16, 0];\n";
        assert!(intent_for(
            escaped_closer,
            decl()
                .captures(escaped_closer)
                .unwrap()
                .get(0)
                .unwrap()
                .start()
        )
        .is_empty());
        assert!(wide_facts(escaped_closer, "t.rs").is_empty());
        let no_intent = "static K: [u16; 3] = [b'A' as u16, b'B' as u16, 0];\n";
        assert_eq!(coverage_counts(no_intent), (1, 0));
        let prose="// UTF-16 string literals — Rust doesn't have `L\"...\"`, so spell them out.\n// All NUL-terminated.\nconst ID: [u16; 3] = [b'A' as u16, b'B' as u16, 0];\n";
        assert_eq!(coverage_counts(prose), (1, 0));
        let wchar="// L\"System\"\nstatic K: [WCHAR; 7] = [b'S' as WCHAR, b'y' as WCHAR, b's' as WCHAR, b't' as WCHAR,\n b'e' as WCHAR, b'm' as WCHAR, 0];\n";
        assert_eq!(
            coverage_counts(wchar),
            (1, 1),
            "matches={:?}; intents={:?}",
            elem()
                .captures_iter(wchar)
                .map(|c| (
                    c.get(0).map(|m| m.as_str()),
                    c.name("char").map(|m| m.as_str()),
                    c.name("hex").map(|m| m.as_str()),
                    c.name("decimal").map(|m| m.as_str()),
                    decode_elem(&c)
                ))
                .collect::<Vec<_>>(),
            intent_for(
                wchar,
                decl().captures(wchar).unwrap().get(0).unwrap().start()
            )
        );
        let tchar = "// L\"ab\"\nstatic K: [TCHAR; 3] = [b'a' as TCHAR, b'b' as TCHAR, 0];\n";
        assert_eq!(coverage_counts(tchar), (1, 1));
        let suffixed = "// L\"ip\"\nstatic K: [u16; 3] = [0x0069u16, 0x0070, 0u16];\n";
        assert!(
            wide_facts(suffixed, "t.rs").is_empty(),
            "findings: {:?}",
            wide_facts(suffixed, "t.rs")
        );
        let unterminated="static NLS_X: [u16; 3] = [b'a' as u16, b'b' as u16, b'c' as u16];\nfn f() { RtlInitUnicodeString(&mut n, NLS_X.as_ptr()); }\n";
        let hits = wide_facts(unterminated, "t.rs");
        assert_eq!(hits.len(), 1);
        assert!(hits[0].contains("NO NUL terminator") && hits[0].contains("\"abc\""));
        let terminated="static NLS_X: [u16; 4] = [b'a' as u16, b'b' as u16, b'c' as u16, 0];\nfn f() { RtlInitUnicodeString(&mut n, NLS_X.as_ptr()); }\n";
        assert!(wide_facts(terminated, "t.rs").is_empty());
        let nested="static NPFS_FS_NAME: [u16; 4] = [b'N' as u16, b'P' as u16, b'F' as u16, b'S' as u16];\nfn f() { RtlCopyMemory(addr_of_mut!((*Info).FileSystemName) as PVOID,\n NPFS_FS_NAME.as_ptr(), n); }\n";
        assert!(wide_facts(nested, "t.rs").is_empty());
        let buffer="static NAME_BUF: [u16; 3] = [b'a' as u16, b'b' as u16, b'c' as u16];\npub static mut NAME: _UNICODE_STRING = _UNICODE_STRING {\n Length: 6, MaximumLength: 6, Buffer: NAME_BUF.as_ptr() as *mut u16,\n};\n";
        assert!(wide_facts(buffer, "t.rs").is_empty());
        let numeric="static ZIPMASK: [u16; 5] = [0x0001, 0x0003, 0x0007, 0x000f, 0x001f];\nfn f() { g(ZIPMASK.as_ptr()); }\n";
        assert!(wide_facts(numeric, "t.rs").is_empty());
        let size_veto="static TRAILER: [u16; 3] = [b'a' as u16, b'b' as u16, b'c' as u16];\nfn f() { if n == core::mem::size_of_val(&TRAILER) { g(TRAILER.as_ptr()); } }\n";
        assert!(wide_facts(size_veto, "t.rs").is_empty());
        let duplicate="fn a() { static fmtW: [u16; 3] = [b'a' as u16, b'b' as u16, b'c' as u16];\n let _ = fmtW; }\nfn b() { static fmtW: [u16; 4] = [b'a' as u16, b'b' as u16, b'c' as u16, 0];\n RtlInitUnicodeString(&mut n, fmtW.as_ptr()); }\n";
        assert!(wide_facts(duplicate, "t.rs").is_empty());
        assert!(
            byte_findings("f(b\"M\\0M\\0\\0\".as_ptr() as *const u16);\n", "t.rs")
                .iter()
                .any(|x| x.contains("ODD length"))
        );
        assert!(byte_findings(
            "f(b\"I\\0\\0\\0n\\0\\0\\0f\\0\\0\\0\\0\\0\".as_ptr() as LPCWSTR);\n",
            "t.rs"
        )
        .iter()
        .any(|x| x.contains("INTERIOR")));
        assert!(byte_findings(
            "f(b\"a\\0\\0\\0b\\0\\0\\0\\0\\0\".as_ptr() as LPCWSTR);\n",
            "t.rs"
        )
        .iter()
        .any(|x| x.contains("INTERIOR")));
        assert!(
            byte_findings("f(b\"I\\0n\\0f\\0\\0\\0\".as_ptr() as LPCWSTR);\n", "t.rs").is_empty()
        );
        let wchar_bad="// L\"System\"\nstatic K: [WCHAR; 6] = [b'S' as WCHAR, b'y' as WCHAR, b's' as WCHAR, b't' as WCHAR, b'e' as WCHAR, 0];\n";
        assert!(wide_facts(wchar_bad, "t.rs")
            .iter()
            .any(|x| x.contains("[WCHAR; 6]") && x.contains("TRUNCATED")));
        let suffixed_bad = "// L\"ip6\"\nstatic K: [u16; 3] = [0x0069u16, 0x0070, 0u16];\n";
        assert!(wide_facts(suffixed_bad, "t.rs")
            .iter()
            .any(|x| x.contains("\"ip\"")));
    }
}
