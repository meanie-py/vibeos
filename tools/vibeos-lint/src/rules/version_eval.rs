//! Pure NTDDI / __REACTOS__ evaluation used by scan_version_vs_c.
use crate::{regexes::cached, twins};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

pub type Env = BTreeMap<String, i64>;
#[derive(Default, Clone)]
pub struct CrateCfg {
    pub on: BTreeSet<String>,
    pub declared: BTreeSet<String>,
    pub emitted: BTreeSet<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeadBranch {
    pub start: usize,
    pub end: usize,
    pub directive: String,
}
const ALWAYS_DEFINED: &[&str] = &["__REACTOS__"];
const VERSION_SYMS: &[&str] = &["NTDDI_VERSION", "_WIN32_WINNT", "WINVER"];
const C_KEYWORDS: &[&str] = &[
    "if", "while", "for", "switch", "return", "sizeof", "do", "else", "case", "defined", "goto",
    "break", "continue", "typedef", "struct", "union", "enum",
];

fn re(p: &str) -> Regex {
    cached(p).expect("version scanner regex")
}
fn token_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        re(r"\s*(0x[0-9A-Fa-f]+[uUlL]*|\d+[uUlL]*|[A-Za-z_]\w*|&&|\|\||[<>=!]=|[<>!()])")
    })
}
fn comment_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| re(r"(?s)/\*.*?\*/|//[^\n]*"))
}
fn number_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| re(r"^(0x[0-9A-Fa-f]+|\d+)[uUlL]*$"))
}
fn pp_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| re(r"(?m)^[ \t]*#[ \t]*(if|ifdef|ifndef|elif|else|endif)\b([^\n]*)$"))
}
fn calls_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| re(r"\b([A-Za-z_]\w*)\s*\("))
}
fn ident_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| re(r"\b[A-Za-z_]\w*\b"))
}
fn cfg_tokens() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| re(r#"[A-Za-z_]\w*|"[^"]*"|[(),=]"#))
}

fn parse_num(s: &str) -> Option<i64> {
    let m = number_re().captures(s)?;
    let v = m.get(1)?.as_str();
    if let Some(h) = v.strip_prefix("0x") {
        i64::from_str_radix(h, 16).ok()
    } else {
        v.parse().ok()
    }
}
struct Parser<'a> {
    tokens: Vec<&'a str>,
    i: usize,
    env: &'a Env,
    constants: &'a BTreeMap<String, i64>,
    mentions: bool,
}
impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&str> {
        self.tokens.get(self.i).copied()
    }
    fn take(&mut self) -> Option<&'a str> {
        let x = self.tokens.get(self.i).copied()?;
        self.i += 1;
        Some(x)
    }
    fn primary(&mut self) -> Option<i64> {
        let t = self.take()?;
        if t == "!" {
            return Some((self.primary()? == 0) as i64);
        }
        if t == "(" {
            let x = self.orx()?;
            if self.take()? != ")" {
                return None;
            }
            return Some(x);
        }
        if t == "defined" {
            let paren = self.peek() == Some("(");
            if paren {
                self.take()?;
            }
            let name = self.take()?;
            if paren && self.take()? != ")" {
                return None;
            }
            if ALWAYS_DEFINED.contains(&name) || VERSION_SYMS.contains(&name) {
                self.mentions = true;
                return Some(1);
            }
            return None;
        }
        if let Some(n) = parse_num(t) {
            return Some(n);
        }
        if let Some(v) = self.env.get(t) {
            self.mentions = true;
            return Some(*v);
        }
        self.constants.get(t).copied()
    }
    fn cmp(&mut self) -> Option<i64> {
        let mut v = self.primary()?;
        while matches!(self.peek(), Some("<" | ">" | "<=" | ">=" | "==" | "!=")) {
            let op = self.take()?;
            let w = self.primary()?;
            v = match op {
                "<" => (v < w) as i64,
                ">" => (v > w) as i64,
                "<=" => (v <= w) as i64,
                ">=" => (v >= w) as i64,
                "==" => (v == w) as i64,
                "!=" => (v != w) as i64,
                _ => return None,
            };
        }
        Some(v)
    }
    fn andx(&mut self) -> Option<i64> {
        let mut v = self.cmp()?;
        while self.peek() == Some("&&") {
            self.take()?;
            let w = self.cmp()?;
            v = (v != 0 && w != 0) as i64;
        }
        Some(v)
    }
    fn orx(&mut self) -> Option<i64> {
        let mut v = self.andx()?;
        while self.peek() == Some("||") {
            self.take()?;
            let w = self.andx()?;
            v = (v != 0 || w != 0) as i64;
        }
        Some(v)
    }
}
pub fn evaluate(expr: &str, env: &Env, constants: &BTreeMap<String, i64>) -> Option<bool> {
    let expr = comment_re().replace_all(expr, "");
    let mut tokens = Vec::new();
    let mut pos = 0;
    while pos < expr.len() {
        if expr[pos..].trim().is_empty() {
            break;
        }
        let m = token_re().find_at(&expr, pos)?;
        if m.start() != pos {
            return None;
        }
        let raw = m.as_str().trim();
        tokens.push(raw);
        pos = m.end();
    }
    let mut p = Parser {
        tokens,
        i: 0,
        env,
        constants,
        mentions: false,
    };
    let v = p.orx()?;
    if p.i != p.tokens.len() || !p.mentions {
        return None;
    }
    Some(v != 0)
}

#[derive(Clone)]
struct Frame {
    taken: Option<bool>,
    alive: Option<bool>,
    start: usize,
    directive: String,
    known: bool,
}
pub fn dead_ranges(
    body: &str,
    env: &Env,
    constants: &BTreeMap<String, i64>,
) -> (Vec<DeadBranch>, usize) {
    let mut dead = Vec::new();
    let mut judged = 0;
    let mut stack: Vec<Frame> = Vec::new();
    for m in pp_re().captures_iter(body) {
        let kind = &m[1];
        let rest = &m[2];
        if matches!(kind, "if" | "ifdef" | "ifndef") {
            let value = if kind == "ifdef" {
                rest.split_whitespace()
                    .next()
                    .and_then(|n| evaluate(&format!("defined({n})"), env, constants))
            } else if kind == "ifndef" {
                rest.split_whitespace()
                    .next()
                    .and_then(|n| evaluate(&format!("!defined({n})"), env, constants))
            } else {
                evaluate(rest, env, constants)
            };
            stack.push(Frame {
                taken: value,
                alive: value,
                start: m.get(0).unwrap().end(),
                directive: format!("#{kind}{rest}"),
                known: value.is_some(),
            });
            continue;
        }
        let Some(top) = stack.last_mut() else {
            continue;
        };
        if top.alive == Some(false) {
            dead.push(DeadBranch {
                start: top.start,
                end: m.get(0).unwrap().start(),
                directive: top.directive.trim().to_owned(),
            });
        }
        if kind == "endif" {
            if top.known {
                judged += 1
            }
            stack.pop();
            continue;
        }
        if kind == "elif" {
            let val = if top.taken.is_some() {
                evaluate(rest, env, constants)
            } else {
                None
            };
            if top.taken.is_none() || val.is_none() {
                top.taken = None;
                top.alive = None;
                top.start = m.get(0).unwrap().end();
                top.directive = format!("#elif{rest}");
            } else if top.taken == Some(true) {
                top.taken = Some(true);
                top.alive = Some(false);
                top.start = m.get(0).unwrap().end();
                top.directive = format!("#elif{rest}");
                top.known = true;
            } else {
                top.taken = val;
                top.alive = val;
                top.start = m.get(0).unwrap().end();
                top.directive = format!("#elif{rest}");
                top.known = true;
            }
        } else if top.taken.is_none() {
            top.alive = None;
            top.start = m.get(0).unwrap().end();
            top.directive = "#else".into()
        } else {
            let alive = top.taken == Some(false);
            top.taken = Some(true);
            top.alive = Some(alive);
            top.start = m.get(0).unwrap().end();
            top.directive = "#else".into();
        }
    }
    (dead, judged)
}

pub fn dead_only_calls(
    body: &str,
    env: &Env,
    constants: &BTreeMap<String, i64>,
) -> (BTreeMap<String, String>, usize) {
    let (ranges, judged) = dead_ranges(body, env, constants);
    if ranges.is_empty() {
        return (BTreeMap::new(), judged);
    }
    let blanked = twins::blank_c(body);
    let mut keep = blanked.into_bytes();
    for r in &ranges {
        for k in r.start..r.end.min(keep.len()) {
            if keep[k] != b'\n' {
                keep[k] = b' '
            }
        }
    }
    let live = String::from_utf8(keep).expect("ASCII blanking preserves UTF-8");
    let live = pp_re().replace_all(&live, "");
    let live_names: BTreeSet<_> = ident_re()
        .find_iter(&live)
        .map(|m| m.as_str().to_owned())
        .collect();
    let blanked = twins::blank_c(body);
    let mut out = BTreeMap::new();
    for r in ranges {
        for m in calls_re().captures_iter(&blanked[r.start..r.end]) {
            let name = &m[1];
            if !live_names.contains(name) && !C_KEYWORDS.contains(&name) {
                out.entry(name.to_owned()).or_insert(r.directive.clone());
            }
        }
    }
    (out, judged)
}

pub fn version_constants(header: &str) -> BTreeMap<String, i64> {
    let def = re(r"(?m)^\s*#\s*define\s+((?:NTDDI|_WIN32_WINNT)_\w+)\s+(0x[0-9A-Fa-f]+|\w+)");
    let mut raw = BTreeMap::new();
    for m in def.captures_iter(header) {
        raw.entry(m[1].to_owned())
            .or_insert_with(|| m[2].to_owned());
    }
    let mut out = BTreeMap::new();
    for name in raw.keys() {
        let mut v = raw[name].clone();
        for _ in 0..8 {
            if let Some(next) = raw.get(&v) {
                if v == *name {
                    break;
                }
                v = next.clone()
            } else {
                break;
            }
        }
        if let Some(n) = parse_num(&v) {
            out.insert(name.clone(), n);
        }
    }
    out
}

pub fn cfg_value(pred: &str, env: &CrateCfg) -> Option<bool> {
    let toks: Vec<_> = cfg_tokens().find_iter(pred).map(|m| m.as_str()).collect();
    let mut i = 0usize;
    fn item(t: &[&str], i: &mut usize, env: &CrateCfg) -> Result<Option<bool>, ()> {
        let atom = *t.get(*i).ok_or(())?;
        *i += 1;
        if matches!(atom, "any" | "all" | "not") && t.get(*i) == Some(&"(") {
            *i += 1;
            let mut vals = Vec::new();
            while t.get(*i) != Some(&")") {
                vals.push(item(t, i, env)?);
                if t.get(*i) == Some(&",") {
                    *i += 1;
                } else if t.get(*i) != Some(&")") {
                    return Err(());
                }
            }
            *i += 1;
            return Ok(match atom {
                "not" => {
                    if vals.len() != 1 {
                        return Err(());
                    } else {
                        vals[0].map(|v| !v)
                    }
                }
                "any" => {
                    if vals.contains(&Some(true)) {
                        Some(true)
                    } else if vals.contains(&None) {
                        None
                    } else {
                        Some(false)
                    }
                }
                _ => {
                    if vals.contains(&Some(false)) {
                        Some(false)
                    } else if vals.contains(&None) {
                        None
                    } else {
                        Some(true)
                    }
                }
            });
        }
        if t.get(*i) == Some(&"=") {
            *i += 1;
            let val = t.get(*i).ok_or(())?.trim_matches('"');
            *i += 1;
            if atom == "feature" {
                return Ok(Some(env.on.contains(val)));
            }
            return Ok(None);
        }
        if matches!(
            atom,
            "test"
                | "debug_assertions"
                | "doc"
                | "doctest"
                | "miri"
                | "unix"
                | "windows"
                | "panic"
                | "overflow_checks"
                | "rustfmt"
                | "clippy"
        ) || env.emitted.contains(atom)
        {
            Ok(None)
        } else {
            Ok(Some(false))
        }
    }
    let v = item(&toks, &mut i, env).ok()?;
    if i == toks.len() {
        v
    } else {
        None
    }
}

fn cfg_attr_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| re(r"#\[cfg\("))
}
fn cfg_macro_if_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| re(r"\bif\s+(!\s*)?cfg!\s*\("))
}
fn cfg_macro_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| re(r"(!\s*)?\bcfg!\s*\("))
}
fn bool_if_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| re(r"\bif\s+(!\s*)?(\w+)\s*\{"))
}
fn bool_const_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| re(r"\b(?:const|let)\s+(\w+)\s*:\s*bool\s*=\s*(true|false)\s*;"))
}
fn paren_end(text: &str, at: usize) -> Option<usize> {
    let b = text.as_bytes();
    let mut depth = 0;
    for k in at..b.len() {
        if b[k] == b'(' {
            depth += 1
        } else if b[k] == b')' {
            depth -= 1;
            if depth == 0 {
                return Some(k);
            }
        }
    }
    None
}
fn stmt_end(text: &str, at: usize) -> usize {
    let b = text.as_bytes();
    let mut k = at;
    let mut depth = 0i32;
    while k < b.len() {
        let c = b[k];
        if c == b'(' || c == b'[' {
            depth += 1
        } else if c == b')' || c == b']' {
            depth -= 1
        } else if c == b';' && depth == 0 {
            return k + 1;
        } else if c == b'{' && depth == 0 {
            let Some(end) = twins::match_brace(text, k) else {
                return b.len();
            };
            if re(r"^\s*else\b").is_match(&text[end..]) {
                let m = re(r"^\s*else\b").find(&text[end..]).unwrap();
                k = end + m.end();
                continue;
            }
            return end;
        }
        k += 1
    }
    b.len()
}
fn expr_end(text: &str, at: usize, op: &str) -> usize {
    let b = text.as_bytes();
    let mut depth = 0i32;
    let mut k = at;
    while k < b.len() {
        let c = b[k];
        if c == b'(' || c == b'[' {
            depth += 1
        } else if c == b')' || c == b']' {
            if depth == 0 {
                return k;
            }
            depth -= 1
        } else if depth == 0
            && (matches!(c, b'{' | b';' | b',') || (op == "&&" && b.get(k..k + 2) == Some(b"||")))
        {
            return k;
        }
        k += 1
    }
    k
}
pub fn mask_gated(
    body: &str,
    raw: &str,
    env: &CrateCfg,
    consts: &BTreeMap<String, bool>,
) -> String {
    let mut spans = Vec::<(usize, usize)>::new();
    for m in cfg_attr_re().find_iter(body) {
        let open = m.end() - 1;
        let Some(close) = paren_end(body, open) else {
            continue;
        };
        if cfg_value(&raw[open + 1..close], env) == Some(false) {
            let bracket = body[close..]
                .find(']')
                .map(|x| close + x + 1)
                .unwrap_or(body.len());
            spans.push((m.start(), stmt_end(body, bracket)));
        }
    }
    for m in cfg_macro_if_re().captures_iter(body) {
        let whole = m.get(0).unwrap();
        let open = whole.end() - 1;
        let Some(close) = paren_end(body, open) else {
            continue;
        };
        if cfg_value(&raw[open + 1..close], env) != Some(false)
            || !re(r"^\s*\{").is_match(&body[close + 1..])
        {
            continue;
        }
        let then_open = body[close..].find('{').unwrap() + close;
        let Some(then_end) = twins::match_brace(body, then_open) else {
            continue;
        };
        if m.get(1).is_none() {
            spans.push((whole.start(), then_end))
        } else {
            spans.push((then_end, stmt_end(body, then_end)))
        }
    }
    let mut known = BTreeMap::from([("true".to_owned(), true), ("false".to_owned(), false)]);
    known.extend(consts.clone());
    for m in bool_if_re().captures_iter(body) {
        let Some(value) = known.get(&m[2]).copied() else {
            continue;
        };
        let then_open = m.get(0).unwrap().end() - 1;
        let Some(then_end) = twins::match_brace(body, then_open) else {
            continue;
        };
        if value != m.get(1).is_some() {
            spans.push((then_end, stmt_end(body, then_end)))
        } else {
            spans.push((m.get(0).unwrap().start(), then_end))
        }
    }
    for m in cfg_macro_re().captures_iter(body) {
        let whole = m.get(0).unwrap();
        let open = whole.end() - 1;
        let Some(close) = paren_end(body, open) else {
            continue;
        };
        if cfg_value(&raw[open + 1..close], env) != Some(false) {
            continue;
        }
        let bang = m.get(1).is_some();
        let after = &body[close + 1..];
        let Some(op) = re(r"^\s*(&&|\|\|)").captures(after) else {
            continue;
        };
        let op = op[1].to_owned();
        if (op == "||") != bang {
            continue;
        }
        let start = close + 1 + op.len();
        let mut end = expr_end(body, start, &op);
        if op == "&&"
            && !bang
            && re(r"\bif\s*$").is_match(&body[..whole.start()])
            && body[end..].starts_with('{')
        {
            if let Some(e) = twins::match_brace(body, end) {
                end = e
            }
        }
        spans.push((whole.start(), if end > 0 { end } else { body.len() }))
    }
    if spans.is_empty() {
        return body.to_owned();
    }
    let mut out = body.as_bytes().to_vec();
    let len = out.len();
    for (s, e) in spans {
        for b in out.get_mut(s..e.min(len)).into_iter().flatten() {
            if *b != b'\n' {
                *b = b' '
            }
        }
    }
    String::from_utf8(out).expect("ASCII masking preserves UTF-8")
}
pub fn item_compiled_out(lines: &[&str], fn_line: usize, env: &CrateCfg) -> bool {
    if fn_line < 2 {
        return false;
    }
    let mut k = fn_line - 2;
    loop {
        if k >= lines.len() {
            break;
        }
        let t = lines[k].trim();
        if t.starts_with("#[cfg(") {
            if let Some(m) = re(r"^#\[cfg\((.*)\)\]\s*$").captures(t) {
                if cfg_value(&m[1], env) == Some(false) {
                    return true;
                }
            }
        } else if !(t.starts_with("#[") || t.starts_with("///") || t.starts_with("//")) {
            break;
        }
        if k == 0 {
            break;
        }
        k -= 1
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    fn env(w: i64, n: i64) -> Env {
        BTreeMap::from([
            ("NTDDI_VERSION".into(), n),
            ("_WIN32_WINNT".into(), w),
            ("WINVER".into(), w),
        ])
    }
    fn constants() -> BTreeMap<String, i64> {
        BTreeMap::from([
            ("NTDDI_WIN7".into(), 0x06010000),
            ("NTDDI_WIN8".into(), 0x06020000),
        ])
    }
    #[test]
    fn python_scanner_selftest_expression_cases() {
        let ws = env(0x502, 0x05020100);
        let win7 = env(0x601, 0x06010000);
        assert_eq!(
            evaluate("(NTDDI_VERSION >= NTDDI_WIN7)", &ws, &constants()),
            Some(false)
        );
        assert_eq!(
            evaluate("NTDDI_VERSION >= NTDDI_WIN7", &win7, &constants()),
            Some(true)
        );
        assert_eq!(
            evaluate("NTDDI_VERSION >= NTDDI_WIN7 && _M_AMD64", &ws, &constants()),
            None
        );
        assert_eq!(evaluate("1", &ws, &constants()), None);
        assert_eq!(
            evaluate("!defined(__REACTOS__)", &ws, &constants()),
            Some(false)
        );
        assert_eq!(
            evaluate("!defined(__REACTOS__) == 0", &ws, &constants()),
            Some(true)
        );
    }
    #[test]
    fn branch_classifier_pins_unknown_chains_and_live_else() {
        let c = "{\nPrep(x);\n#if (NTDDI_VERSION >= NTDDI_WIN7)\nFsRtlChangeBackingFileObject(x);\n#else\nOldPath(x);\n#endif\n#ifndef __REACTOS__\nNotOnReactos(x);\n#endif\n#if DBG\nDbgOnly(x);\n#endif\n#if (NTDDI_VERSION >= NTDDI_WIN7) && defined(_M_AMD64)\nUnknowable(x);\n#endif\n}";
        let ws = env(0x502, 0x05020100);
        let (calls, n) = dead_only_calls(c, &ws, &constants());
        assert_eq!(
            calls.keys().cloned().collect::<Vec<_>>(),
            ["FsRtlChangeBackingFileObject", "NotOnReactos"]
        );
        assert_eq!(n, 2);
    }
    #[test]
    fn scanner_control_gates_short_circuit_and_cfg_values_match_python() {
        let cfg = CrateCfg {
            on: BTreeSet::from(["reactos".into()]),
            declared: BTreeSet::from(["reactos".into(), "smp".into()]),
            emitted: BTreeSet::from(["cnt_libm".into()]),
        };
        assert_eq!(cfg_value("not(feature = \"reactos\")", &cfg), Some(false));
        assert_eq!(
            cfg_value("any(target_arch = \"x86\", ntddi_ge_ntddi_win7)", &cfg),
            None
        );
        assert_eq!(
            cfg_value("all(target_arch = \"x86\", ntddi_ge_ntddi_win7)", &cfg),
            Some(false)
        );
        assert_eq!(cfg_value("cnt_libm", &cfg), None);
        assert_eq!(cfg_value("test", &cfg), None);
        let body =
            "{ if cfg!(ntddi_ge_ntddi_win8) && a { FsRtlChangeBackingFileObject(x); } y(); }";
        let env = CrateCfg::default();
        let masked = mask_gated(body, body, &env, &BTreeMap::new());
        assert!(!re(r"FsRtlChangeBackingFileObject\s*\(").is_match(&masked));
        assert!(masked.contains("y();"));
    }
    #[test]
    fn compiled_out_item_reads_the_immediately_preceding_attribute() {
        let env = CrateCfg::default();
        let lines = [
            "#[cfg(any())]",
            "#[no_mangle]",
            "pub unsafe extern \"C\" fn F() {",
        ];
        assert!(item_compiled_out(&lines, 3, &env));
        assert!(!item_compiled_out(&lines[1..], 2, &env));
    }
}
