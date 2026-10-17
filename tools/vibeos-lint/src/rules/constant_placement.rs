//! Find cycles where sealed C constants occupy one another's translation slots.
use crate::{twins, Corpus, Rule};
use rayon::prelude::*;
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::Path;
use std::sync::OnceLock;

pub struct ConstantPlacement;
impl Rule for ConstantPlacement {
    fn name(&self) -> &'static str {
        "scan_constant_placement"
    }
    fn needs_c_twins(&self) -> bool {
        true
    }
    fn extra_files(&self) -> &'static [&'static str] {
        &["tools/rustconv/rustconv/ddk_enum_constants.py"]
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let ddk_file = corpus
            .files
            .iter()
            .find(|f| {
                f.path.as_path() == Path::new("tools/rustconv/rustconv/ddk_enum_constants.py")
            })
            .ok_or_else(|| "missing DDK enum constant snapshot".to_owned())?;
        let ddk = parse_ddk_names(&ddk_file.text);
        let out = corpus
            .rust_files()
            .par_bridge()
            .flat_map_iter(|file| {
                let mut out = Vec::new();
                if patterns().skip_path.is_match(&file.path.to_string_lossy()) {
                    return out;
                }
                let absolute = corpus.root.join(&file.path);
                let nolint_lines: Vec<_> = file
                    .text
                    .lines()
                    .enumerate()
                    .filter_map(|(i, line)| {
                        line.contains("nolint: constant-placement").then_some(i + 1)
                    })
                    .collect();
                for function in file.twin_functions(&corpus.root) {
                    let start_line = function.line.saturating_sub(6).max(1);
                    let first = nolint_lines.partition_point(|&line| line < start_line);
                    let has_nolint = nolint_lines
                        .get(first)
                        .is_some_and(|&line| line <= function.line);
                    if has_nolint {
                        continue;
                    }
                    let candidates = corpus.c_twins().get(&function.name);
                    let ordered = twins::prefer(&absolute, candidates);
                    let Some(twin) = ordered
                        .first()
                        .filter(|t| twins::affinity(&absolute, t) > 0)
                    else {
                        continue;
                    };
                    for finding in placement_findings(&twin.body, &function.body, &ddk) {
                        out.push(format!(
                            "{}:{}: constant-placement {} {}",
                            file.path.display(),
                            function.line,
                            function.name,
                            finding.ring.join("/")
                        ));
                    }
                }
                out
            })
            .collect();
        Ok(out)
    }
}

struct Patterns {
    skip_path: Regex,
    c_integer: Regex,
    seal_integer: Regex,
    rs_integer: Regex,
    c_all_caps: Regex,
    camel: Regex,
    declarator: Regex,
    block_prototype: Regex,
    named_cast: Regex,
    zero_init: Regex,
    ident: Regex,
    ident_tail: Regex,
    as_cast: Regex,
    let_type: Regex,
    rust_zero: Regex,
    rust_one: Regex,
    array_len: Regex,
    type_before: Regex,
    case_c: Regex,
    case_rs: Regex,
    return_tail: Regex,
    comparison_after: Regex,
    assign: Regex,
    guard_if: Regex,
    guard_prev: Regex,
    c_condition: Regex,
    c_preprocessor: Regex,
    pseh_except: Regex,
    ddk_name: Regex,
    cast_closer: Regex,
    declaration: Regex,
    case_colon: Regex,
    case_cleanup: Regex,
    if_directive: Regex,
    endif_directive: Regex,
}
fn patterns() -> &'static Patterns {
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(|| Patterns {
        skip_path: crate::regexes::cached(r"(/target/|/\.responses/|/bindgen_|/types_[a-z0-9]+\.rs$)").unwrap(),
        c_integer: crate::regexes::cached(r"^(0[xX][0-9A-Fa-f]+|0[oO]?[0-7]+|[0-9]+)[uUlL]*$").unwrap(),
        seal_integer: crate::regexes::cached(r"(?:0[xX][0-9A-Fa-f]+|0[oO][0-7]+|[0-9]+)(?:[uUlL]{0,3})").unwrap(),
        rs_integer: crate::regexes::cached(r"(0[xX][0-9A-Fa-f_]+|0[oO][0-7_]+|0[bB][01_]+|[0-9][0-9_]*)(?:_?(?:[iu](?:8|16|32|64|128|size)))?").unwrap(),
        c_all_caps: crate::regexes::cached(r"\b_*[A-Z][A-Z0-9_]+\b").unwrap(),
        camel: crate::regexes::cached(r"\b[A-Z][A-Za-z0-9_]*\b").unwrap(),
        declarator: crate::regexes::cached(r"\b[A-Za-z_]\w*\s+\**\s*([A-Za-z_]\w*)\s*([;,=)\[])").unwrap(),
        block_prototype: crate::regexes::cached(r"\bextern\s+[^;{}=]*?\([^;{}]*\)\s*;").unwrap(),
        named_cast: crate::regexes::cached(r"\b(?:const_cast|static_cast|reinterpret_cast|dynamic_cast)\s*<[^<>;{}]*>").unwrap(),
        zero_init: crate::regexes::cached(r"=\s*\{\s*(0)\s*\}").unwrap(),
        ident: crate::regexes::cached(r"[A-Za-z_][A-Za-z0-9_]*").unwrap(),
        ident_tail: crate::regexes::cached(r"([A-Za-z_][A-Za-z0-9_]*)\s*(?:\[[^\]]*\]\s*)*$").unwrap(),
        as_cast: crate::regexes::cached(r"\s+as\s+[A-Za-z_][A-Za-z0-9_:<>*\s]*$").unwrap(),
        let_type: crate::regexes::cached(r"\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:[^=]*$").unwrap(),
        rust_zero: crate::regexes::cached(r"(?:[!=]=|\blet\s+(?:mut\s+)?\w+\s*(?::[^=;]+)?=)\s*\(?\s*$").unwrap(),
        rust_one: crate::regexes::cached(r"(?:[+-]=\s*|\.(?:add|sub|offset|wrapping_add|wrapping_sub|checked_add|checked_sub|overflowing_add|saturating_add|saturating_sub)\(\s*)$").unwrap(),
        array_len: crate::regexes::cached(r";\s*$").unwrap(),
        type_before: crate::regexes::cached(r"(?:::<|<|\*mut\s+|\*const\s+|:\s*|\bas\s+)$").unwrap(),
        case_c: crate::regexes::cached(r"\bcase\s*$").unwrap(),
        case_rs: crate::regexes::cached(r"^\s*(?:\|\s*[A-Za-z0-9_:]+\s*)*=>").unwrap(),
        return_tail: crate::regexes::cached(r"\breturn\s*$").unwrap(),
        comparison_after: crate::regexes::cached(r"^\s*(==|!=|<=|>=|<|>)\s*([A-Za-z_][A-Za-z0-9_]*)").unwrap(),
        assign: crate::regexes::cached(r"([+\-*/%&|^]?)=").unwrap(),
        guard_if: crate::regexes::cached(r"(?s)\bif\b(.*)$").unwrap(),
        guard_prev: crate::regexes::cached(r"\bif\b([^;{}]*)\{?\s*$").unwrap(),
        c_condition: crate::regexes::cached(r"\b(SUCCEEDED|FAILED)\s*\(").unwrap(),
        c_preprocessor: crate::regexes::cached(r"^[ \t]*#").unwrap(),
        pseh_except: crate::regexes::cached(r"\b(?:RpcExcept|_SEH2_EXCEPT|__except)\b[ \t]*\(").unwrap(),
        ddk_name: crate::regexes::cached(r#"(?m)^\s*\"([A-Za-z_]\w*)\",?\s*$"#).unwrap(),
        cast_closer: crate::regexes::cached(r"^\s*\**\s*\)").unwrap(),
        declaration: crate::regexes::cached(r"^\s+\**\s*[A-Za-z_]\w*").unwrap(),
        case_colon: crate::regexes::cached(r"^\s*:").unwrap(),
        case_cleanup: crate::regexes::cached(r"\bcase\b[^:]*:").unwrap(),
        if_directive: crate::regexes::cached(r"(?m)^[ \t]*#[ \t]*(if|ifdef|ifndef)\b").unwrap(),
        endif_directive: crate::regexes::cached(r"(?m)^[ \t]*#[ \t]*endif\b").unwrap(),
    })
}
fn parse_ddk_names(text: &str) -> HashSet<String> {
    patterns()
        .ddk_name
        .captures_iter(text)
        .map(|c| c[1].to_owned())
        .collect()
}

fn parse_nat(s: &str, radix: u32) -> Option<String> {
    if s.is_empty() {
        return None;
    }
    let mut digits = vec![0u32]; // base 1,000,000,000, little endian
    for ch in s.chars() {
        let value = ch.to_digit(radix)?;
        let mut carry = value as u64;
        for d in &mut digits {
            let n = (*d as u64) * radix as u64 + carry;
            *d = (n % 1_000_000_000) as u32;
            carry = n / 1_000_000_000;
        }
        while carry > 0 {
            digits.push((carry % 1_000_000_000) as u32);
            carry /= 1_000_000_000;
        }
    }
    let mut out = digits.pop().unwrap_or(0).to_string();
    while let Some(d) = digits.pop() {
        out.push_str(&format!("{d:09}"));
    }
    Some(out)
}
fn c_int_value(spelling: &str) -> Option<String> {
    let cap = patterns().c_integer.captures(spelling)?;
    if cap.get(0)?.start() != 0 || cap.get(0)?.end() != spelling.len() {
        return None;
    }
    let s = cap.get(1)?.as_str();
    if let Some(v) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        return parse_nat(v, 16);
    }
    if let Some(v) = s.strip_prefix("0o").or_else(|| s.strip_prefix("0O")) {
        return parse_nat(v, 8);
    }
    if s.len() > 1 && s.starts_with('0') {
        return parse_nat(s, 8);
    }
    parse_nat(s, 10)
}
fn rust_int_value(spelling: &str) -> Option<String> {
    let s = spelling.replace('_', "");
    if let Some(v) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        parse_nat(v, 16)
    } else if let Some(v) = s.strip_prefix("0o").or_else(|| s.strip_prefix("0O")) {
        parse_nat(v, 8)
    } else if let Some(v) = s.strip_prefix("0b").or_else(|| s.strip_prefix("0B")) {
        parse_nat(v, 2)
    } else {
        parse_nat(&s, 10)
    }
}
fn boundary_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '.'
}

fn byte_mask(text: &str) -> String {
    let mut b = text.as_bytes().to_vec();
    let mut i = 0;
    while i < b.len() {
        if i + 1 < b.len() && b[i] == b'/' && b[i + 1] == b'/' {
            let end = b[i + 2..]
                .iter()
                .position(|x| *x == b'\n')
                .map_or(b.len(), |n| i + 2 + n);
            for x in &mut b[i..end] {
                *x = b' ';
            }
            i = end;
        } else if i + 1 < b.len() && b[i] == b'/' && b[i + 1] == b'*' {
            let end = b[i + 2..]
                .windows(2)
                .position(|w| w == b"*/")
                .map_or(b.len(), |n| i + 2 + n + 2);
            for x in &mut b[i..end] {
                if *x != b'\n' {
                    *x = b' ';
                }
            }
            i = end;
        } else if b[i] == b'"' || b[i] == b'\'' {
            let q = b[i];
            b[i] = b' ';
            i += 1;
            while i < b.len() {
                if b[i] == b'\\' {
                    b[i] = b' ';
                    if i + 1 < b.len() {
                        if b[i + 1] != b'\n' {
                            b[i + 1] = b' ';
                        }
                        i += 2;
                    } else {
                        i += 1;
                    }
                } else if b[i] == q {
                    b[i] = b' ';
                    i += 1;
                    break;
                } else {
                    if b[i] != b'\n' {
                        b[i] = b' ';
                    }
                    i += 1;
                }
            }
        } else {
            i += 1;
        }
    }
    String::from_utf8(b).expect("byte masking preserves UTF-8 length")
}
fn line_start(text: &str, pos: usize) -> usize {
    text[..pos].rfind('\n').map_or(0, |i| i + 1)
}
fn directives(text: &str, masked: &str) -> Vec<bool> {
    let mut out = vec![false; text.len()];
    let mut pos = 0;
    while pos < masked.len() {
        let end = masked[pos..].find('\n').map_or(masked.len(), |n| pos + n);
        let mut next = if end < masked.len() {
            end + 1
        } else {
            masked.len()
        };
        if patterns().c_preprocessor.is_match(&masked[pos..end]) {
            let mut stop = end;
            loop {
                out[pos..stop].fill(true);
                let continued = masked[pos..stop].trim_end().ends_with('\\');
                if !continued || stop >= masked.len() {
                    next = if stop < masked.len() {
                        stop + 1
                    } else {
                        masked.len()
                    };
                    break;
                }
                pos = stop + 1;
                stop = masked[pos..].find('\n').map_or(masked.len(), |n| pos + n);
            }
        }
        pos = next;
    }
    out
}
fn signature_bounds(masked: &str) -> Vec<(usize, usize)> {
    let body = masked.find('{');
    let Some(open) = masked.find('(') else {
        return vec![];
    };
    if body.is_some_and(|b| b < open) {
        return vec![];
    }
    let mut depth = 0i32;
    let mut close = None;
    for (i, ch) in masked[open..].char_indices() {
        let at = open + i;
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
            if depth == 0 {
                close = Some(at);
                break;
            }
        }
    }
    let Some(cl) = close else { return vec![] };
    let tail = masked[cl + 1..].trim_start();
    if tail.starts_with('{') || tail.starts_with(':') {
        vec![(open, cl + 1)]
    } else {
        vec![]
    }
}
fn dead_spans(text: &str, masked: &str) -> Vec<(usize, usize)> {
    let mut ev: Vec<(usize, i32)> = patterns()
        .if_directive
        .find_iter(text)
        .map(|m| (m.start(), 1))
        .chain(
            patterns()
                .endif_directive
                .find_iter(text)
                .map(|m| (m.start(), -1)),
        )
        .collect();
    ev.sort();
    let mut spans = Vec::new();
    let (mut depth, mut opened) = (0usize, 0usize);
    for (pos, d) in ev {
        if d == 1 {
            if depth == 0 {
                opened = line_start(text, pos);
            }
            depth += 1;
        } else if depth > 0 {
            depth -= 1;
            if depth == 0 {
                let end = text[pos..].find('\n').map_or(text.len(), |n| pos + n);
                spans.push((opened, end));
            }
        }
    }
    if depth > 0 {
        spans.push((opened, text.len()));
    }
    for m in patterns().pseh_except.find_iter(masked) {
        let paren = m.end() - 1;
        let (mut d, mut close) = (0i32, None);
        for (i, ch) in masked[paren..].char_indices() {
            let at = paren + i;
            if ch == '(' {
                d += 1
            } else if ch == ')' {
                d -= 1;
                if d == 0 {
                    close = Some(at);
                    break;
                }
            }
        }
        let Some(cl) = close else { continue };
        let Some(brace_rel) = masked[cl..].find('{') else {
            continue;
        };
        let brace = cl + brace_rel;
        if !masked[cl + 1..brace].trim().is_empty() {
            continue;
        }
        let mut dep = 0i32;
        let mut end = None;
        for (i, ch) in masked[brace..].char_indices() {
            let at = brace + i;
            if ch == '{' {
                dep += 1
            } else if ch == '}' {
                dep -= 1;
                if dep == 0 {
                    end = Some(at + 1);
                    break;
                }
            }
        }
        if let Some(end) = end {
            spans.push((m.start(), end));
        }
    }
    spans.sort();
    spans
}
fn in_dead(spans: &[(usize, usize)], a: usize, b: usize) -> bool {
    spans.iter().any(|(x, y)| *x <= a && b <= *y)
}
fn collect_facts(raw: &str, ddk: &HashSet<String>) -> (String, Vec<(String, String, bool)>) {
    let masked = byte_mask(raw);
    let declared: HashSet<String> = patterns()
        .declarator
        .captures_iter(&masked)
        .filter(|c| matches!(&c[2], ";" | "," | "=" | ")" | "["))
        .map(|c| c[1].to_owned())
        .collect();
    let dirs = directives(raw, &masked);
    let zeros: HashSet<(usize, usize)> = patterns()
        .zero_init
        .captures_iter(&masked)
        .map(|c| {
            let m = c.get(1).unwrap();
            (m.start(), m.end())
        })
        .collect();
    let sig = signature_bounds(&masked);
    let mut type_spans: Vec<(usize, usize)> = patterns()
        .block_prototype
        .find_iter(&masked)
        .map(|m| (m.start(), m.end()))
        .chain(
            patterns()
                .named_cast
                .find_iter(&masked)
                .map(|m| (m.start(), m.end())),
        )
        .collect();
    type_spans.extend(sig.iter().copied());
    let mut spans: Vec<(usize, usize, String)> = Vec::new();
    for m in patterns().seal_integer.find_iter(&masked) {
        let start = m.start();
        let end = m.end();
        let before = masked[..start].chars().next_back();
        let after = masked[end..].chars().next();
        if before.is_some_and(boundary_char) || after.is_some_and(boundary_char) {
            continue;
        }
        if zeros.contains(&(start, end)) || sig.iter().any(|(a, b)| *a <= start && start < *b) {
            continue;
        }
        spans.push((start, end, "integer".into()));
    }
    for m in patterns().c_all_caps.find_iter(&masked) {
        let word = m.as_str();
        let start = m.start();
        let tail = &masked[m.end()..];
        if non_fact_word(word)
            || word.starts_with("__RUSTCONV_")
            || tail.trim_start().starts_with('(')
        {
            continue;
        }
        let closing = patterns().cast_closer.find(tail);
        let after_cast = closing.map(|x| tail[x.end()..].trim_start()).unwrap_or("");
        let cast = start > 0
            && masked.as_bytes()[start - 1] == b'('
            && closing.is_some()
            && !after_cast.is_empty()
            && !",);]}".contains(after_cast.chars().next().unwrap());
        let declaration = patterns().declaration.is_match(tail);
        if cast || declaration || type_spans.iter().any(|(a, b)| *a <= start && start < *b) {
            continue;
        }
        spans.push((start, m.end(), "symbol".into()));
    }
    for m in patterns().camel.find_iter(&masked) {
        let word = m.as_str();
        let start = m.start();
        if !ddk.contains(word)
            || declared.contains(word)
            || masked[m.end()..].trim_start().starts_with('(')
        {
            continue;
        }
        let before = masked[..start].trim_end();
        if before.ends_with('.')
            || before.ends_with("->")
            || patterns().declaration.is_match(&masked[m.end()..])
        {
            continue;
        }
        spans.push((start, m.end(), "enum".into()));
    }
    spans.retain(|(a, b, _)| !dirs[*a..*b].iter().any(|v| *v));
    spans.sort_by_key(|x| (x.0, x.1));
    let dead = dead_spans(raw, &masked);
    let mut c = raw.to_owned();
    let mut facts = Vec::new();
    for (index, (a, b, _kind)) in spans.iter().enumerate().rev() {
        let orig = raw[*a..*b].to_owned();
        let ph = format!("__RUSTCONV_FACT_{index:04}__");
        facts.push((orig, ph.clone(), in_dead(&dead, *a, *b)));
        c.replace_range(*a..*b, &ph);
    }
    facts.reverse();
    (c, facts)
}
fn non_fact_word(s: &str) -> bool {
    matches!(
        s,
        "BOOL"
            | "BYTE"
            | "CLSID"
            | "DWORD"
            | "ERR"
            | "FIXME"
            | "GUID"
            | "HRESULT"
            | "INT"
            | "LONG"
            | "LPARAM"
            | "NULL"
            | "POINTL"
            | "SIZE_T"
            | "TRACE"
            | "UINT"
            | "ULONG"
            | "WCHAR"
            | "WARN"
            | "WPARAM"
    )
}

fn rust_syntax(rust: &str, start: usize, end: usize, value: Option<&str>) -> bool {
    let before = tail_chars(&rust[..start], 80);
    let line = before.rsplit('\n').next().unwrap_or("");
    if value == Some("0") && patterns().rust_zero.is_match(line) {
        return true;
    }
    if value == Some("1") && patterns().rust_one.is_match(line) {
        return true;
    }
    if value.is_some()
        && patterns().array_len.is_match(line)
        && prefix_chars(&rust[end..], 8).trim_start().starts_with(']')
    {
        return true;
    }
    value.is_none()
        && (patterns().type_before.is_match(line)
            || prefix_chars(&rust[end..], 4).trim_start().starts_with('>'))
}
fn tail_chars(text: &str, count: usize) -> &str {
    let Some((start, _)) = text.char_indices().rev().nth(count.saturating_sub(1)) else {
        return text;
    };
    &text[start..]
}
fn prefix_chars(text: &str, count: usize) -> &str {
    text.char_indices()
        .nth(count)
        .map_or(text, |(at, _)| &text[..at])
}
fn statements(text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut start = 0;
    for (i, b) in text.bytes().enumerate() {
        if b == b';' || b == b'{' || b == b'}' {
            out.push((start, i));
            start = i + 1;
        }
    }
    out.push((start, text.len()));
    out
}
fn norm(s: &str) -> String {
    s.replace('_', "").to_lowercase()
}
fn operand(expr: &str) -> Option<String> {
    let e = patterns()
        .as_cast
        .replace(expr.trim_end().trim_end_matches('(').trim_end(), "");
    let c = patterns().ident_tail.captures(e.trim_end())?;
    let name = c.get(1)?.as_str();
    if is_keyword(name) || name.starts_with("__RUSTCONV_") {
        None
    } else {
        Some(norm(name))
    }
}
fn first_ident(text: &str) -> String {
    patterns()
        .ident
        .find_iter(text)
        .map(|m| m.as_str())
        .find(|s| !is_keyword(s) && !s.starts_with("__RUSTCONV_"))
        .map(norm)
        .unwrap_or_default()
}
fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "if" | "else"
            | "while"
            | "for"
            | "do"
            | "switch"
            | "case"
            | "default"
            | "break"
            | "continue"
            | "return"
            | "goto"
            | "sizeof"
            | "struct"
            | "union"
            | "enum"
            | "typedef"
            | "const"
            | "volatile"
            | "static"
            | "extern"
            | "register"
            | "auto"
            | "signed"
            | "unsigned"
            | "void"
            | "char"
            | "short"
            | "int"
            | "long"
            | "float"
            | "double"
            | "let"
            | "mut"
            | "fn"
            | "match"
            | "loop"
            | "as"
            | "in"
            | "unsafe"
            | "pub"
            | "use"
            | "mod"
            | "impl"
            | "self"
            | "Self"
            | "true"
            | "false"
            | "ref"
            | "move"
            | "where"
            | "type"
            | "crate"
            | "super"
            | "dyn"
    )
}
fn guard(text: &str, stmt_start: usize, head: &str) -> String {
    if let Some(m) = patterns().guard_if.captures(head) {
        return first_ident(m.get(1).unwrap().as_str());
    }
    let prev = tail_chars(&text[..stmt_start], 2000);
    patterns()
        .guard_prev
        .captures(prev)
        .map(|m| first_ident(m.get(1).unwrap().as_str()))
        .unwrap_or_default()
}
fn slot(text: &str, at: usize, end: usize) -> Option<Vec<String>> {
    let mut depth = 0i32;
    let mut commas = 0usize;
    let mut boundary = 0usize;
    for (i, ch) in text[..at].char_indices().rev() {
        if ch == ';' || ch == '{' || ch == '}' {
            boundary = i + ch.len_utf8();
            break;
        }
        if ch == ')' || ch == ']' {
            depth += 1;
        } else if ch == '[' {
            if depth == 0 {
                return None;
            }
            depth -= 1;
        } else if ch == '(' {
            if depth == 0 {
                let before = tail_chars(&text[..i], 80)
                    .trim_end()
                    .trim_end_matches('!')
                    .trim_end();
                if let Some(m) = patterns().ident_tail.captures(before) {
                    let name = m.get(1)?.as_str();
                    if !is_keyword(name) {
                        return Some(vec![
                            "arg".into(),
                            norm(name).trim_start_matches('_').to_owned(),
                            commas.to_string(),
                        ]);
                    }
                }
                commas = 0;
            } else {
                depth -= 1;
            }
        } else if ch == ',' && depth == 0 {
            commas += 1;
        }
    }
    let head = &text[boundary..at];
    let after = prefix_chars(&text[end..], 120);
    if patterns().case_c.is_match(head) {
        if let Some(m) = patterns().case_colon.find(after) {
            let rest = &after[m.end()..];
            let rest = patterns().case_cleanup.replace(rest, " ");
            return Some(vec!["case".into(), first_ident(&rest)]);
        }
    }
    if let Some(m) = patterns().case_rs.find(after) {
        return Some(vec!["case".into(), first_ident(&after[m.end()..])]);
    }
    if patterns().return_tail.is_match(head) {
        return Some(vec![
            "ret".into(),
            guard(
                text,
                boundary,
                &head[..head.rfind("return").unwrap_or(head.len())],
            ),
        ]);
    }
    let mut assignments: Vec<_> = patterns()
        .assign
        .captures_iter(head)
        .filter_map(|m| {
            let whole = m.get(0)?;
            let before = whole
                .start()
                .checked_sub(1)
                .and_then(|_| head[..whole.start()].chars().next_back());
            let next = head[whole.end()..].chars().next();
            if before.is_some_and(|c| "=!<>+-*/%&|^".contains(c))
                || next.is_some_and(|c| c == '>' || c == '=')
            {
                return None;
            }
            Some((whole.start(), m))
        })
        .collect();
    if let Some((_, m)) = assignments.pop() {
        let lhs = head[..m.get(0).unwrap().start()].trim_end();
        let name = patterns()
            .let_type
            .captures(lhs)
            .map(|m| norm(&m[1]))
            .or_else(|| operand(lhs));
        if let Some(name) = name {
            return Some(vec!["set".into(), name]);
        }
    }
    let before = head.trim_end();
    for op in ["==", "!=", "<=", ">=", "<", ">", "&", "|", "^"] {
        if before.ends_with(op) && !before.ends_with(&format!("{op}{op}")) {
            let kind = if "=!<>".contains(op.chars().next().unwrap()) {
                "cmp"
            } else {
                "bit"
            };
            if let Some(name) = operand(&before[..before.len() - op.len()]) {
                return Some(vec![kind.into(), name]);
            }
        }
    }
    if let Some(m) = patterns().comparison_after.captures(after) {
        let name = m.get(2)?.as_str();
        if !is_keyword(name) {
            return Some(vec!["cmp".into(), norm(name)]);
        }
    }
    None
}
#[derive(Clone, Debug)]
struct Misplacement {
    spelling: String,
    rival: String,
    statement: String,
}
#[derive(Clone, Debug)]
struct Finding {
    ring: Vec<String>,
    evidence: Vec<Misplacement>,
}
fn placement_findings(c_body: &str, rust_body: &str, ddk: &HashSet<String>) -> Vec<Finding> {
    let (c_text, facts) = seal_c(c_body, ddk);
    if facts.len() < 2 {
        return Vec::new();
    }
    let mut c_slots: BTreeMap<String, BTreeSet<Vec<String>>> = BTreeMap::new();
    for (original, placeholder, dead) in &facts {
        if *dead {
            continue;
        }
        if let Some(at) = c_text.find(placeholder) {
            let sl = slot(&c_text, at, at + placeholder.len());
            let set = c_slots.entry(original.clone()).or_default();
            if let Some(sl) = sl {
                set.insert(sl);
            }
        }
    }
    let mut owners: BTreeMap<Vec<String>, BTreeSet<String>> = BTreeMap::new();
    for (sp, slots) in &c_slots {
        for sl in slots {
            owners.entry(sl.clone()).or_default().insert(sp.clone());
        }
    }
    let rust = twins::blank_rust(rust_body);
    let spans = statements(&rust);
    let mut values: HashMap<String, Vec<String>> = HashMap::new();
    for sp in c_slots.keys() {
        if let Some(v) = c_int_value(sp) {
            values.entry(v).or_default().push(sp.clone());
        }
    }
    let mut occ: Vec<(String, usize, usize)> = Vec::new();
    for m in patterns().rs_integer.captures_iter(&rust) {
        let whole = m.get(0).unwrap();
        let start = whole.start();
        let end = whole.end();
        let before = rust[..start].chars().next_back();
        let after = rust[end..].chars().next();
        if before.is_some_and(boundary_char) || after.is_some_and(boundary_char) {
            continue;
        }
        let Some(lit) = m.get(1).map(|x| x.as_str()) else {
            continue;
        };
        let Some(v) = rust_int_value(lit) else {
            continue;
        };
        if let Some(names) = values.get(&v) {
            if names.len() == 1 && !rust_syntax(&rust, start, end, Some(&v)) {
                occ.push((names[0].clone(), start, end));
            }
        }
    }
    for sp in c_slots.keys() {
        if c_int_value(sp).is_some() {
            continue;
        }
        let re = crate::regexes::cached(&format!(r"\b{}\b", regex::escape(sp))).unwrap();
        for m in re.find_iter(&rust) {
            if !rust_syntax(&rust, m.start(), m.end(), None) {
                occ.push((sp.clone(), m.start(), m.end()));
            }
        }
    }
    let mut edges: BTreeMap<String, BTreeMap<String, Misplacement>> = BTreeMap::new();
    for (sp, start, end) in occ {
        let Some(sl) = slot(&rust, start, end) else {
            continue;
        };
        let own = owners.get(&sl);
        if own.is_some_and(|x| x.contains(&sp)) {
            continue;
        }
        let rivals: BTreeSet<_> = own
            .into_iter()
            .flatten()
            .filter(|r| !same_value(r, &sp))
            .cloned()
            .collect();
        if rivals.is_empty() {
            continue;
        }
        let stmt = statement_text(rust_body, &spans, start);
        for rival in rivals {
            edges
                .entry(sp.clone())
                .or_default()
                .entry(rival.clone())
                .or_insert_with(|| Misplacement {
                    spelling: sp.clone(),
                    rival,
                    statement: stmt.clone(),
                });
        }
    }
    rings(&edges)
}
fn same_value(a: &str, b: &str) -> bool {
    match (c_int_value(a), c_int_value(b)) {
        (Some(x), Some(y)) => x == y,
        _ => false,
    }
}
fn statement_text(body: &str, spans: &[(usize, usize)], at: usize) -> String {
    let Some((a, b)) = spans.iter().find(|(a, b)| *a <= at && at <= *b) else {
        return String::new();
    };
    body[*a..*b]
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(160)
        .collect()
}
fn rings(edges: &BTreeMap<String, BTreeMap<String, Misplacement>>) -> Vec<Finding> {
    fn visit(
        v: &str,
        edges: &BTreeMap<String, BTreeMap<String, Misplacement>>,
        index: &mut HashMap<String, usize>,
        low: &mut HashMap<String, usize>,
        stack: &mut Vec<String>,
        on: &mut HashSet<String>,
        comps: &mut Vec<Vec<String>>,
        counter: &mut usize,
    ) {
        index.insert(v.into(), *counter);
        low.insert(v.into(), *counter);
        *counter += 1;
        stack.push(v.into());
        on.insert(v.into());
        if let Some(ns) = edges.get(v) {
            for w in ns.keys() {
                if !index.contains_key(w) {
                    visit(w, edges, index, low, stack, on, comps, counter);
                    let val = low[v].min(low[w]);
                    low.insert(v.into(), val);
                } else if on.contains(w) {
                    let val = low[v].min(index[w]);
                    low.insert(v.into(), val);
                }
            }
        }
        if low[v] == index[v] {
            let mut comp = Vec::new();
            loop {
                let w = stack.pop().unwrap();
                on.remove(&w);
                comp.push(w.clone());
                if w == v {
                    break;
                }
            }
            if comp.len() > 1 {
                comps.push(comp);
            }
        }
    }
    let mut idx = HashMap::new();
    let mut low = HashMap::new();
    let mut st = Vec::new();
    let mut on = HashSet::new();
    let mut comps = Vec::new();
    let mut n = 0;
    for v in edges.keys() {
        if !idx.contains_key(v) {
            visit(
                v, edges, &mut idx, &mut low, &mut st, &mut on, &mut comps, &mut n,
            );
        }
    }
    let mut out = Vec::new();
    for mut comp in comps {
        comp.sort();
        let members: HashSet<_> = comp.iter().cloned().collect();
        let mut ev = Vec::new();
        for s in &comp {
            if let Some(ts) = edges.get(s) {
                for (t, m) in ts {
                    if members.contains(t) {
                        ev.push(m.clone());
                    }
                }
            }
        }
        out.push(Finding {
            ring: comp,
            evidence: ev,
        });
    }
    out
}

fn matching_paren(text: &str, open: usize) -> Option<usize> {
    let mut depth = 0i32;
    for (i, ch) in text[open..].char_indices() {
        let at = open + i;
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
            if depth == 0 {
                return Some(at);
            }
        }
    }
    None
}
fn canonicalize_hresult(mut text: String) -> String {
    loop {
        let masked = byte_mask(&text);
        let matches: Vec<_> = patterns()
            .c_condition
            .captures_iter(&masked)
            .filter_map(|m| Some((m.get(1)?.start(), m.get(1)?.end(), m[1].to_owned())))
            .collect();
        if matches.is_empty() {
            return text;
        }
        let mut changed = false;
        for (start, end, kind) in matches.into_iter().rev() {
            let Some(open_rel) = masked[end..].find('(') else {
                continue;
            };
            let open = end + open_rel;
            let Some(close) = matching_paren(&masked, open) else {
                continue;
            };
            let expr = text[open + 1..close].to_owned();
            let op = if kind == "SUCCEEDED" { ">=" } else { "<" };
            text.replace_range(start..close + 1, &format!("(({expr}) {op} 0)"));
            changed = true;
        }
        if !changed {
            return text;
        }
    }
}
fn seal_c(input: &str, ddk: &HashSet<String>) -> (String, Vec<(String, String, bool)>) {
    collect_facts(&canonicalize_hresult(input.to_owned()), ddk)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ddk() -> HashSet<String> {
        HashSet::new()
    }
    fn rings(c: &str, r: &str) -> Vec<Vec<String>> {
        placement_findings(c, r, &ddk())
            .into_iter()
            .map(|f| f.ring)
            .collect()
    }
    #[test]
    fn swapped_assignments_and_faithful_values() {
        let c = "void f(void) { a = 10; b = 20; }";
        assert_eq!(
            rings(c, "unsafe fn f() { a = 20; b = 10; }"),
            vec![vec!["10".to_string(), "20".to_string()]]
        );
        assert!(rings(c, "unsafe fn f() { a = 10; b = 20; }").is_empty());
    }
    #[test]
    fn integer_values_are_compared_across_spellings() {
        let c = "void f(void) { a = 0x10; b = 3; }";
        assert!(rings(c, "fn f() { a = 16u32; b = 3_u32; }").is_empty());
        assert!(!rings(c, "fn f() { a = 3; b = 0x10; }").is_empty());
    }
    #[test]
    fn error_codes_in_similar_calls_are_insufficient_without_context() {
        let c="double f(double x) { if (x < 0) return math_error(_DOMAIN, \"exp\", x, 0, ret); if (x > MAXV) return math_error(_OVERFLOW, \"exp\", x, 0, ret); return y; }";
        let r="fn f(x: f64) -> f64 { if x < 0.0 { return math_error(_OVERFLOW, \"exp\", x, 0.0, ret); } if x > MAXV { return math_error(_DOMAIN, \"exp\", x, 0.0, ret); } y }";
        assert!(rings(c, r).is_empty());
    }
    #[test]
    fn three_way_and_one_way() {
        assert_eq!(
            rings(
                "void f(void) { a = 1; b = 2; c = 3; }",
                "fn f() { a = 2; b = 3; c = 1; }"
            ),
            vec![vec!["1".to_string(), "2".to_string(), "3".to_string()]]
        );
        assert!(rings(
            "void f(void) { a = 10; b = 20; }",
            "fn f() { let t = 10; a = t; b = 20; }"
        )
        .is_empty());
    }
    #[test]
    fn symbols_and_rust_syntax_exclusions() {
        let c = "void f(void) { x = FLAG_A; y = FLAG_B; z = FLAG_A; }";
        assert!(rings(c, "fn f() { x = FLAG_A; y = FLAG_B; z = FLAG_A; }").is_empty());
        assert_eq!(
            rings(c, "fn f() { x = FLAG_B; y = FLAG_A; z = FLAG_A; }"),
            vec![vec!["FLAG_A".to_string(), "FLAG_B".to_string()]]
        );
        let c2 = "void f(void) { n = 0; while (*p) { if (x & F) n++; p++; } total = 1; }";
        let r2="fn f() { let mut n = 0; while *p != 0 { if (x & F) != 0 { n += 1; } p = p.add(1); } total = 1; }";
        assert!(rings(c2, r2).is_empty());
    }
    #[test]
    fn type_positions_do_not_count_as_values() {
        let c = "void f(void) { a = 3; b = 4; memset(&v, 0, sizeof(VIEW)); }";
        let r="fn f() { let s: [u8; 4] = [0; 3]; a = 3; b = 4; write_bytes(&mut v, 0, size_of::<VIEW>()); }";
        assert!(rings(c, r).is_empty());
    }
    #[test]
    fn dead_regions_and_aggregate_initializer_facts_are_optional() {
        let (sealed, facts) = seal_c(
            "void f(void){\n#if 0\n a=10;\n#endif\n b=20; c=30; int x={0};\n}",
            &ddk(),
        );
        assert!(facts.iter().any(|(s, _, dead)| s == "10" && *dead));
        assert!(sealed.contains("__RUSTCONV_FACT_"));
        assert!(!facts.iter().any(|(s, _, _)| s == "0"));
    }
    #[test]
    fn arbitrary_integer_widths_compare_by_value() {
        let c = "void f(void){a=0x100000000000000000000000000000000; b=2;}";
        let r = "fn f(){a=340282366920938463463374607431768211456; b=2;}";
        assert!(rings(c, r).is_empty());
    }

    #[test]
    fn multiline_guards_do_not_read_as_a_swap() {
        let c = "{\n    if ((VolumeEntry == NULL) ||\n        (VolumeEntry->pExtents == NULL) ||\n        (PartEntry == NULL))\n        return FALSE;\n    if (VolumeEntry->Start == PartEntry->Start)\n        return TRUE;\n    return FALSE;\n}";
        let r = "{\n    if VolumeEntry.is_null() || (*VolumeEntry).pExtents.is_null() || PartEntry.is_null() {\n        return FALSE;\n    }\n    if (*VolumeEntry).Start == (*PartEntry).Start {\n        return TRUE;\n    }\n    return FALSE;\n}";
        assert!(rings(c, r).is_empty());
    }

    #[test]
    fn generated_ddk_set_is_read_as_quoted_names() {
        let names = parse_ddk_names(
            "DDK_ENUM_CONSTANTS = frozenset({\n    \"KernelMode\",\n    \"NonPagedPool\",\n})\n",
        );
        assert!(names.contains("KernelMode"));
        assert!(names.contains("NonPagedPool"));
    }
}

#[cfg(test)]
mod original_python_fixture_contract {
    use super::*;
    #[test]
    fn original_placement_audit_cases() {
        let ddk: HashSet<String> = [].into_iter().collect();
        let cases: Vec<(&str, &str, Vec<Vec<String>>)> = vec![
            (
                r##########"void f(void) { a = 10; b = 20; }"##########,
                r##########"unsafe fn f() { a = 20; b = 10; }"##########,
                vec![vec![
                    r##########"10"##########.to_owned(),
                    r##########"20"##########.to_owned(),
                ]],
            ),
            (
                r##########"void f(void) { a = 10; b = 20; }"##########,
                r##########"unsafe fn f() { a = 10; b = 20; }"##########,
                vec![],
            ),
            (
                r##########"void f(void) { a = 0x10; b = 3; }"##########,
                r##########"fn f() { a = 16u32; b = 3_u32; }"##########,
                vec![],
            ),
            (
                r##########"void f(void) { a = 0x10; b = 3; }"##########,
                r##########"fn f() { a = 3; b = 0x10; }"##########,
                vec![vec![
                    r##########"0x10"##########.to_owned(),
                    r##########"3"##########.to_owned(),
                ]],
            ),
            (
                r##########"double f(double x) {
        if (x < 0) return math_error(_DOMAIN, "exp", x, 0, ret);
        if (x > MAXV) return math_error(_OVERFLOW, "exp", x, 0, ret);
        return y; }"##########,
                r##########"fn f(x: f64) -> f64 {
        if x < 0.0 { return math_error(_OVERFLOW, "exp", x, 0.0, ret); }
        if x > MAXV { return math_error(_DOMAIN, "exp", x, 0.0, ret); }
        y }"##########,
                vec![],
            ),
            (
                r##########"void f(void) { a = 1; b = 2; c = 3; }"##########,
                r##########"fn f() { a = 2; b = 3; c = 1; }"##########,
                vec![vec![
                    r##########"1"##########.to_owned(),
                    r##########"2"##########.to_owned(),
                    r##########"3"##########.to_owned(),
                ]],
            ),
            (
                r##########"void f(void) { a = 10; b = 20; }"##########,
                r##########"fn f() { let t = 10; a = t; b = 20; }"##########,
                vec![],
            ),
            (
                r##########"void f(void) { x = FLAG_A; y = FLAG_B; z = FLAG_A; }"##########,
                r##########"fn f() { x = FLAG_A; y = FLAG_B; z = FLAG_A; }"##########,
                vec![],
            ),
            (
                r##########"void f(void) { x = FLAG_A; y = FLAG_B; z = FLAG_A; }"##########,
                r##########"fn f() { x = FLAG_B; y = FLAG_A; z = FLAG_A; }"##########,
                vec![vec![
                    r##########"FLAG_A"##########.to_owned(),
                    r##########"FLAG_B"##########.to_owned(),
                ]],
            ),
            (
                r##########"void f(void) { n = 0; while (*p) { if (x & F) n++; p++; } total = 1; }"##########,
                r##########"fn f() { let mut n = 0; while *p != 0 { if (x & F) != 0 { n += 1; } p = p.add(1); } total = 1; }"##########,
                vec![],
            ),
            (
                r##########"void f(void) { a = 3; b = 4; memset(&v, 0, sizeof(VIEW)); }"##########,
                r##########"fn f() { let s: [u8; 4] = [0; 3]; a = 3; b = 4; write_bytes(&mut v, 0, size_of::<VIEW>()); }"##########,
                vec![],
            ),
            (
                r##########"{
    ULONG i;

    if ((VolumeEntry == NULL) ||
        (VolumeEntry->pExtents == NULL) ||
        (PartEntry == NULL))
        return FALSE;

    for (i = 0; i < VolumeEntry->pExtents->NumberOfDiskExtents; i++)
    {
        if (VolumeEntry->pExtents->Extents[i].DiskNumber == PartEntry->DiskEntry->DiskNumber)
        {
            if ((VolumeEntry->pExtents->Extents[i].StartingOffset.QuadPart == PartEntry->StartSector.QuadPart * PartEntry->DiskEntry->BytesPerSector) &&
                (VolumeEntry->pExtents->Extents[i].ExtentLength.QuadPart == PartEntry->SectorCount.QuadPart * PartEntry->DiskEntry->BytesPerSector))
                return TRUE;
        }
    }

    return FALSE;
}"##########,
                r##########"{
    let mut i: ULONG = 0;
    if VolumeEntry.is_null() || (*VolumeEntry).pExtents.is_null() || PartEntry.is_null() {
        return FALSE;
    }
    let extents = core::ptr::addr_of_mut!((*(*VolumeEntry).pExtents).Extents) as *mut DISK_EXTENT;
    i = 0;
    while i < (*(*VolumeEntry).pExtents).NumberOfDiskExtents {
        if (*extents.offset(i as isize)).DiskNumber == (*(*PartEntry).DiskEntry).DiskNumber {
            if (*extents.offset(i as isize)).StartingOffset.QuadPart as ULONGLONG
                == (*PartEntry).StartSector.QuadPart.wrapping_mul((*(*PartEntry).DiskEntry).BytesPerSector as ULONGLONG)
                && (*extents.offset(i as isize)).ExtentLength.QuadPart as ULONGLONG
                    == (*PartEntry).SectorCount.QuadPart.wrapping_mul((*(*PartEntry).DiskEntry).BytesPerSector as ULONGLONG)
            {
                return TRUE;
            }
        }
        i = i.wrapping_add(1);
    }
    return FALSE;
}"##########,
                vec![],
            ),
        ];
        for (i, (c, r, expected)) in cases.into_iter().enumerate() {
            let mut actual: Vec<_> = placement_findings(c, r, &ddk)
                .into_iter()
                .map(|f| f.ring)
                .collect();
            actual.sort();
            assert_eq!(actual, expected, "Python fixture case {i}");
        }
    }
}
