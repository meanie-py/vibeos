use crate::rules::dropped_null_guard as nulls;
use crate::{
    twins::{self, Function},
    Corpus, Rule,
};
use rayon::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{LazyLock, Mutex};

pub struct StoreOnFailure;
const SUCCESS: &[&str] = &[
    "STATUS_SUCCESS",
    "S_OK",
    "TRUE",
    "ERROR_SUCCESS",
    "NO_ERROR",
    "NOERROR",
    "true",
];
static RETURNS: LazyLock<Regex> = LazyLock::new(|| crate::regexes::cached(r"\breturn\b").unwrap());
static RET_EXPR: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"\breturn\b\s*([^;]*);").unwrap());
static FAIL_NAME: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(
        r"(?:\w+::)*(?:STATUS_\w+|E_\w+|\w+_E_\w+|ERROR_\w+|CR_\w+|FALSE|false|NULL)",
    )
    .unwrap()
});
static STATUS_TEST: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"\b(?:NT_SUCCESS|SUCCEEDED|FAILED|NT_ERROR)!?\s*\(\s*([A-Za-z_]\w*(?:\s*\.\s*\w+)*)\s*\)|\b([A-Za-z_]\w*(?:\s*\.\s*\w+)*)\s*(?:==|!=|<|>=)\s*(?:[\w:]*_SUCCESS|S_OK|0|TRUE|FALSE)\b|\b([A-Za-z_]\w*(?:\s*\.\s*\w+)*)\s*\.\s*is_(?:ok|err)\s*\(").unwrap()
});
static LET_VAL: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"\blet\s+(mut\s+)?([A-Za-z_]\w*)\s*(?::[^=;]*)?=\s*([^;]+);").unwrap()
});
static NULL_STORE: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"=\s*(?:core::|std::)?(?:ptr::)?null(?:_mut)?\s*(?:::\s*<[^>]*>)?\s*\(\s*\)\s*;|=\s*NULL\s*;").unwrap()
});
static C_CALL: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"\b([A-Za-z_]\w*)\s*\(").unwrap());
static RET_CALL: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"\breturn\s*\(?\s*([A-Za-z_]\w*)\s*\(").unwrap());
static C_MACRO: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"(?:^|[;{}])\s*([A-Z][A-Z0-9_]{3,})\s*\(").unwrap());
static HARMLESS: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"(?:DPRINT\w*|TRACE\w*|WARN\w*|ERR|FIXME\w*|ASSERT\w*|NT_ASSERT\w*|UNREFERENCED\w*|DBG\w*|DEBUG\w*|PAGED_CODE|KdPrint\w*|INFO\w*|MSG\w*|RtlZero\w*|SetLastError)").unwrap()
});
static SUCCESS_EXPR: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"\b([A-Za-z_]\w*)\s*(?:==|!=|<|>=)\s*(?:[\w:]*_SUCCESS|S_OK|0|TRUE|FALSE)\b|\b([A-Za-z_]\w*)\s*\.\s*is_(?:ok|err)\s*\(").unwrap()
});
static IDENT_EXPR: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"^[A-Za-z_]\w*(?:\s*\.\s*\w+)*$").unwrap());
static FAIL_IDENT: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"^[A-Z][A-Z0-9_]*$|^(?:\w+::)+[A-Z][A-Z0-9_]*$").unwrap()
});
static INTO_SUFFIX: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"\.\s*(?:try_)?into\s*\(\s*\)(?:\s*\.\s*unwrap\s*\(\s*\))?$").unwrap()
});
static AS_TYPE_SUFFIX: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"\s+as\s+[\w:]+$").unwrap());
static LITERAL: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"^(-?)(0x[0-9a-fA-F]+|[0-9]+)(?:_?[iu](?:8|16|32|64|size))?$").unwrap()
});
static ELSE_START: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"(?s)^\s*else\b").unwrap());
static ERR_EXPR: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"(?s)^Err\s*\(.*\)$").unwrap());
static UNIT_OR_NEVER: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"->\s*(?:\(\s*\)|!)").unwrap());
static INC_POST: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"(?:\+\+|--)").unwrap());
static STORE_NOLINT: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"//\s*nolint(?::\s*store-on-failure\b|\s*$)").unwrap()
});
static CALL_SKIP: &[&str] = &[
    "if",
    "while",
    "for",
    "switch",
    "return",
    "sizeof",
    "defined",
    "ASSERT",
    "NT_ASSERT",
    "DPRINT",
    "DPRINT1",
    "TRACE",
    "WARN",
    "ERR",
    "FIXME",
];

fn strip(e: &str) -> String {
    let mut s = e.split_whitespace().collect::<Vec<_>>().join(" ");
    while s.starts_with('(') && s.ends_with(')') {
        s = s[1..s.len() - 1].trim().to_owned()
    }
    s = INTO_SUFFIX.replace(&s, "").to_string();
    AS_TYPE_SUFFIX.replace(&s, "").trim().to_string()
}
fn last(e: &str) -> &str {
    e.rsplit("::").next().unwrap_or(e)
}
fn is_success(e: &str) -> bool {
    let s = strip(e);
    let l = last(&s);
    SUCCESS.contains(&l) || l.ends_with("_SUCCESS")
}
fn literal_value(e: &str) -> Option<i64> {
    let s = strip(e);
    let l = last(&s);
    if l == "TRUE" || l == "true" {
        return Some(1);
    }
    if l == "FALSE" || l == "false" {
        return Some(0);
    }
    if SUCCESS.contains(&l) || l.ends_with("_SUCCESS") {
        return Some(0);
    }
    let c = LITERAL.captures(&s)?;
    let mut n = if c[2].starts_with("0x") {
        i64::from_str_radix(&c[2][2..], 16).ok()?
    } else {
        c[2].parse().ok()?
    };
    if &c[1] == "-" {
        n = -n
    }
    Some(n)
}
fn stmt_end(body: &str, at: usize) -> usize {
    let (mut depth, mut i) = (0i32, at);
    let b = body.as_bytes();
    while i < b.len() {
        let c = b[i];
        if b"([{\"'".contains(&c) {
            depth += 1
        } else if b")]}'".contains(&c) {
            if depth == 0 {
                return i;
            }
            depth -= 1
        } else if c == b';' && depth == 0 {
            return i;
        }
        i += 1
    }
    body.len()
}
fn open_of(body: &str, close: usize) -> Option<usize> {
    let (mut depth, mut i) = (0i32, close as isize);
    let b = body.as_bytes();
    while i >= 0 {
        let c = b[i as usize];
        if c == b'}' {
            depth += 1
        } else if c == b'{' {
            depth -= 1;
            if depth == 0 {
                return Some(i as usize);
            }
        }
        i -= 1
    }
    None
}
fn match_brace(body: &str, open: usize) -> Option<usize> {
    twins::match_brace(body, open)
}
fn skip_unreachable(body: &str, close: usize) -> usize {
    let Some(op) = open_of(body, close) else {
        return close;
    };
    let before = if op > 0 {
        body[..op]
            .chars()
            .rev()
            .take(3)
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>()
            .trim_end()
            .to_string()
    } else {
        String::new()
    };
    if before.ends_with("=>") {
        let (mut depth, mut j) = (0i32, close + 1);
        let b = body.as_bytes();
        while j < b.len() {
            if b[j] == b'{' {
                depth += 1
            } else if b[j] == b'}' {
                if depth == 0 {
                    return j;
                }
                depth -= 1
            }
            j += 1
        }
        return close;
    }
    let mut j = close;
    loop {
        let Some(m) = ELSE_START.find(&body[j + 1..]) else {
            return j;
        };
        let k = body[j + 1 + m.end()..]
            .find('{')
            .map(|x| j + 1 + m.end() + x);
        let Some(k) = k else { return j };
        let Some(end) = match_brace(body, k) else {
            return close;
        };
        j = end;
    }
}
fn next_value(body: &str, pos: usize) -> Option<String> {
    let b = body.as_bytes();
    let (mut depth, mut i, mut seg) = (0i32, pos, pos);
    while i < b.len() {
        if b[i] == b'r'
            && body[i..].starts_with("return")
            && body[i + 6..]
                .chars()
                .next()
                .map_or(true, |c| !c.is_alphanumeric() && c != '_')
            && body[..i]
                .chars()
                .next_back()
                .map_or(true, |c| !c.is_alphanumeric() && c != '_')
        {
            let end = stmt_end(body, i + 6);
            let v = body[i + 6..end].trim();
            return (!v.is_empty()).then(|| v.to_string());
        }
        if b[i] == b'{' {
            depth += 1
        } else if b[i] == b'}' {
            if depth == 0 {
                let tail = body[seg..i].trim();
                if !tail.is_empty() {
                    return Some(tail.to_string());
                }
                i = skip_unreachable(body, i);
                seg = i + 1
            } else {
                depth -= 1;
                if depth == 0 {
                    seg = i + 1
                }
            }
        } else if b[i] == b';' && depth == 0 {
            seg = i + 1
        }
        i += 1
    }
    None
}
fn success_tested(var: &str, body: &str, before: usize) -> bool {
    let matches_var = |value: &str| {
        value.strip_suffix(var).is_some_and(|prefix| {
            !prefix
                .chars()
                .next_back()
                .is_some_and(|c| c.is_alphanumeric() || c == '_')
        })
    };
    STATUS_TEST
        .captures_iter(&body[..before])
        .any(|m| (1..=3).any(|i| m.get(i).is_some_and(|x| matches_var(x.as_str()))))
}
fn may_fail(v: &str, body: &str, store_at: usize) -> bool {
    if literal_value(v).is_some() {
        return true;
    }
    if is_success(v) {
        return false;
    }
    if ERR_EXPR.is_match(v) {
        return true;
    }
    if FAIL_IDENT.is_match(v) {
        return FAIL_NAME.is_match(v);
    }
    if !IDENT_EXPR.is_match(v) {
        return false;
    }
    if success_tested(v, body, store_at) {
        return false;
    }
    for m in LET_VAL.captures_iter(body) {
        if m.get(2).unwrap().as_str() == v {
            let mutable = m.get(1).is_some();
            let init = m.get(3).unwrap().as_str();
            return !(!mutable && is_success(init));
        }
    }
    true
}
fn returns_value(text: &str, f: &Function) -> bool {
    let Some(close) = text[..f.body_start].rfind(')') else {
        return false;
    };
    let sig = &text[close..f.body_start];
    sig.contains("->") && !UNIT_OR_NEVER.is_match(sig)
}
fn null_store(body: &str, start: usize, end: usize) -> bool {
    NULL_STORE.is_match(&body[start..end])
}
fn c_ident_ok(text: &str, at: usize, end: usize, name: &str) -> bool {
    if &text[at..end] != name {
        return false;
    }
    let prev = text[..at].chars().next_back();
    let next = text[end..].chars().next();
    !prev.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == '*' || c == '>')
        && !next.is_some_and(|c| c.is_alphanumeric() || c == '_')
}
fn c_stores(p: &str, body: &str) -> Vec<usize> {
    let v = regex::escape(p);
    let lhs = crate::regexes::cached(&format!(
        r"(?:\*\s*\(?\s*{v}\s*\)?|{v}\s*(?:->\s*\w+(?:\s*(?:\.|->)\s*\w+)*|\[[^\]]*\]))"
    ))
    .unwrap();
    let mut out = vec![];
    for m in lhs.find_iter(body) {
        let at = m.start();
        let end = m.end();
        let lhs_text = m.as_str();
        let after = &body[end..];
        let before = body[..at].chars().next_back();
        let direct = lhs_text.trim_start().starts_with('*');
        let left_ok = if direct {
            !before.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == ')' || c == ']')
        } else {
            !before.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == '>')
        };
        if !left_ok {
            continue;
        }
        let trimmed = after.trim_start();
        let ws = after.len() - trimmed.len();
        // regex crate has no negative lookahead; mirror Python's `(?!=)` guard.
        let assignment = ["<<=", ">>=", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^="]
            .iter()
            .any(|op| trimmed.starts_with(op))
            || (trimmed.starts_with('=') && !trimmed.starts_with("=="));
        if assignment {
            out.push(at);
            continue;
        }
        if INC_POST.find(trimmed).is_some_and(|x| x.start() == 0) {
            out.push(at);
            continue;
        }
        let prefix = &body[..at];
        let prefix_trim = prefix.trim_end();
        if prefix_trim.ends_with("++") || prefix_trim.ends_with("--") {
            let op_at = prefix_trim.len() - 2;
            let prior = prefix_trim[..op_at].chars().next_back();
            if !prior.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == ')' || c == ']') {
                out.push(op_at)
            }
        }
        let _ = ws;
    }
    let calls = C_CALL.captures_iter(body).collect::<Vec<_>>();
    for m in calls {
        let name = &m[1];
        if CALL_SKIP.contains(&name) {
            continue;
        }
        let whole = m.get(0).unwrap();
        let open = whole.end() - 1;
        let Some(close) = match_paren(body, open) else {
            continue;
        };
        let args = nulls::_split_top(&body[whole.end()..close]);
        if args.iter().any(|a| {
            let arg = a.trim();
            arg == p
                || arg.strip_prefix('&').is_some_and(|x| {
                    let x = x.trim();
                    x == p
                        || x.strip_prefix(p)
                            .is_some_and(|tail| tail.trim_start().starts_with("->"))
                })
        }) {
            out.push(whole.start())
        }
    }
    out.sort_unstable();
    out
}
fn match_paren(text: &str, open: usize) -> Option<usize> {
    let mut depth = 0i32;
    for (i, c) in text.char_indices().filter(|(i, _)| *i >= open) {
        if c == '(' {
            depth += 1
        } else if c == ')' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}
fn c_next_return(body: &str, at: usize) -> Option<String> {
    let m = RET_EXPR.captures_at(body, at)?;
    Some(m[1].to_string())
}
type SuccessKey = (std::path::PathBuf, usize, String, String, u8);
#[derive(Default)]
struct SuccessCache(Mutex<HashMap<SuccessKey, Option<(String, String)>>>);

fn c_success_only(
    p: &str,
    twin: &Function,
    index: &crate::twins::CIndex,
    depth: u8,
    cache: &SuccessCache,
) -> Option<(String, String)> {
    let key = (
        twin.path.clone(),
        twin.body_start,
        twin.name.clone(),
        p.to_owned(),
        depth,
    );
    let cached = cache
        .0
        .lock()
        .expect("store-on-failure cache poisoned")
        .get(&key)
        .cloned();
    if let Some(result) = cached {
        return result.clone();
    }
    let result = c_success_only_uncached(p, twin, index, depth, cache);
    cache
        .0
        .lock()
        .expect("store-on-failure cache poisoned")
        .insert(key, result.clone());
    result
}

fn c_success_only_uncached(
    p: &str,
    twin: &Function,
    index: &crate::twins::CIndex,
    depth: u8,
    cache: &SuccessCache,
) -> Option<(String, String)> {
    let body = &twin.body;
    let stores = c_stores(p, body);
    let rets: Vec<_> = RET_EXPR
        .captures_iter(body)
        .filter_map(|m| {
            let v = m.get(1)?.as_str();
            (!v.trim().is_empty()).then(|| {
                (
                    m.get(0).unwrap().start(),
                    v.to_string(),
                    m.get(0).unwrap().range(),
                )
            })
        })
        .collect();
    if stores.is_empty()
        || stores
            .iter()
            .all(|&at| rets.iter().any(|(_, _, r)| r.start <= at && at < r.end))
    {
        return if depth > 0 {
            through_callee(p, twin, index, depth, cache)
        } else {
            None
        };
    }
    if stores
        .iter()
        .any(|&at| rets.iter().any(|(_, _, r)| r.start <= at && at < r.end))
    {
        return None;
    }
    let fails: Vec<_> = rets
        .iter()
        .map(|(at, v, _range)| (*at, strip(v)))
        .filter(|(_, v)| !is_success(v))
        .collect();
    if fails.is_empty() || fails.last().unwrap().0 > stores[0] {
        return None;
    }
    let mut succ = vec![];
    for &at in &stores {
        let r = c_next_return(body, at)?;
        if !is_success(&r) {
            return None;
        }
        succ.push(strip(&r));
    }
    // Iterate captures once. Searching again from each match's start rescanned
    // the remainder of large bodies and kept the same first capture each time.
    for c in C_MACRO.captures_iter(body) {
        if c.get(0).unwrap().start() >= stores[0] && !HARMLESS.is_match(&c[1]) {
            return None;
        }
    }
    let lo = body[..stores[0]].rfind('\n').map_or(0, |i| i + 1);
    let hi = body[stores[0]..]
        .find('\n')
        .map_or(body.len(), |i| stores[0] + i);
    let site = body[lo..hi]
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(80)
        .collect::<String>();
    let ev = format!(
        "{}  … only after its last failure `return {}`",
        site,
        fails.last().unwrap().1
    );
    Some((ev, succ.first()?.clone()))
}
fn through_callee(
    p: &str,
    twin: &Function,
    index: &crate::twins::CIndex,
    depth: u8,
    cache: &SuccessCache,
) -> Option<(String, String)> {
    for m in RET_CALL.captures_iter(&twin.body) {
        let whole = m.get(0).unwrap();
        let open = whole.end() - 1;
        let close = match_paren(&twin.body, open)?;
        let args = nulls::_split_top(&twin.body[whole.end()..close]);
        if !args.iter().any(|a| a.trim() == p) {
            continue;
        }
        for sub in index.in_file(&twin.path, &m[1]) {
            let names = nulls::c_param_names(&sub.params);
            if names.len() == args.len() {
                if let Some((ev, succ)) = c_success_only(
                    &names[args.iter().position(|a| a.trim() == p).unwrap()],
                    &sub,
                    index,
                    depth - 1,
                    cache,
                ) {
                    return Some((format!("{}(): {}", &m[1], ev), succ));
                }
            }
        }
    }
    None
}
struct Site {
    line: usize,
    fn_name: String,
    param: String,
    params: Vec<(String, String)>,
    idx: usize,
    value: String,
}
fn candidates(file: &crate::SourceFile, root: &Path) -> Vec<Site> {
    let raw = &file.text;
    let prefix: String = raw.chars().take(2000).collect();
    if nulls::GEN_MARKERS.iter().any(|g| prefix.contains(g))
        || nulls::SKIP_PATH_RE.is_match(&file.path.to_string_lossy())
    {
        return vec![];
    }
    let funcs = file.twin_functions(root);
    let text = twins::blank_rust(raw);
    let mut out = vec![];
    for f in funcs {
        if !returns_value(&text, f) {
            continue;
        }
        let params = nulls::split_params(&f.params);
        let ptrs: std::collections::HashMap<_, _> = params
            .iter()
            .enumerate()
            .filter(|(_, (_, t))| nulls::is_pointer(t))
            .map(|(i, (n, _))| (n.as_str(), i))
            .collect();
        if ptrs.is_empty() {
            continue;
        }
        for (name, at, end) in nulls::all_stores(&f.body) {
            let Some(&idx) = ptrs.get(name.as_str()) else {
                continue;
            };
            if null_store(&f.body, at, end) {
                continue;
            }
            let Some(value) = next_value(&f.body, end) else {
                continue;
            };
            let head = strip(&value);
            if !may_fail(&head, &f.body, at) {
                continue;
            }
            let line = file.line(f.body_start + at);
            if STORE_NOLINT.is_match(file.line_text(line)) {
                continue;
            }
            out.push(Site {
                line,
                fn_name: f.name.clone(),
                param: name,
                params: params.clone(),
                idx,
                value: head.chars().take(60).collect(),
            });
            break;
        }
    }
    out
}
impl Rule for StoreOnFailure {
    fn name(&self) -> &'static str {
        "scan_store_on_failure"
    }
    fn needs_c_twins(&self) -> bool {
        true
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let index = c.c_twins();
        let success_cache = SuccessCache::default();
        let out = c
            .rust_files()
            .par_bridge()
            .flat_map_iter(|f| {
                let mut out = vec![];
                for s in candidates(f, &c.root) {
                    let abs = c.root.join(&f.path);
                    let twins: Vec<_> = twins::prefer(&abs, index.get(&s.fn_name))
                        .into_iter()
                        .filter(|t| twins::same_module(&abs, t))
                        .collect();
                    let Some(twin) = twins.first() else { continue };
                    let cname =
                        nulls::c_name_for(&s.params, s.idx, &nulls::c_param_names(&twin.params));
                    let Some(cname) = cname else { continue };
                    let Some((_, succ)) = c_success_only(&cname, twin, index, 1, &success_cache)
                    else {
                        continue;
                    };
                    if is_success(&s.value)
                        || (literal_value(&s.value).is_some()
                            && literal_value(&s.value) == literal_value(&succ))
                    {
                        continue;
                    }
                    out.push(format!(
                        "{}:{}: store-on-failure {}({})",
                        f.path.display(),
                        s.line,
                        s.fn_name,
                        s.param
                    ));
                }
                out
            })
            .collect();
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn success_values_and_next_result() {
        assert!(is_success("STATUS_SUCCESS"));
        assert!(is_success("nt::ERROR_SUCCESS"));
        assert_eq!(literal_value("1u8"), Some(1));
        assert_eq!(
            next_value("{ *out=1; status }", 10).as_deref(),
            Some("status")
        );
    }
    #[test]
    fn success_tests_and_null_store() {
        assert!(success_tested(
            "status",
            "if status == STATUS_SUCCESS { *out = 1; }",
            40
        ));
        assert!(null_store(
            "*out = core::ptr::null_mut(); return Err;",
            5,
            33
        ));
    }
    #[test]
    fn c_success_only_proves_success_and_failure_paths() {
        let f = Function {
            name: "F".into(),
            path: "x.c".into(),
            line: 1,
            body: "{ if (!ok) return STATUS_BAD; *out = n; return STATUS_SUCCESS; }".into(),
            body_start: 0,
            params: "int ok, int *out".into(),
            ret: "int".into(),
        };
        let c = crate::twins::CIndex::from_files(Vec::<&crate::SourceFile>::new());
        let cache = SuccessCache::default();
        let first = c_success_only("out", &f, &c, 0, &cache);
        assert!(first.is_some());
        assert_eq!(c_success_only("out", &f, &c, 0, &cache), first);
    }
    #[test]
    fn candidate_next_failure_guarded_success_and_nolint() {
        let root = Path::new("/");
        let src = "pub unsafe fn F(out: *mut u32) -> u32 { *out = 1; STATUS_BAD }";
        let file = crate::SourceFile::new("x.rs".into(), src.into());
        assert_eq!(candidates(&file, root).len(), 1);
        let good = "pub unsafe fn F(out: *mut u32, status:u32) -> u32 { if status == STATUS_SUCCESS { *out=1; } status }";
        assert!(candidates(&crate::SourceFile::new("x.rs".into(), good.into()), root).is_empty());
        let waived = "pub unsafe fn F(out: *mut u32) -> u32 { *out = 1; // nolint: store-on-failure\n STATUS_BAD }";
        assert!(candidates(&crate::SourceFile::new("x.rs".into(), waived.into()), root).is_empty());
    }
    #[test]
    fn rule_matches_full_control_fixture_and_suppresses_good_fixture() {
        let bad = r#"pub unsafe extern "system" fn RtlDecompressFragment(format:u16, uncompressed:*mut u8, uncompressed_size:u32, compressed:*const u8, compressed_size:u32, offset:u32, final_size:*mut u32, workspace:*mut u8) -> u32 { let result=decompress(uncompressed,compressed,offset); if !final_size.is_null() { *final_size=result.bytes_written as u32; } result.status }"#;
        let c = "unsigned RtlDecompressFragment(unsigned format, unsigned char *uncompressed, unsigned n, const unsigned char *compressed, unsigned cs, unsigned off, unsigned *final_size, unsigned char *workspace) { if (!compressed) return STATUS_BAD_COMPRESSION_BUFFER; *final_size = n; return STATUS_SUCCESS; }";
        let corpus = Corpus::new(
            Path::new("/").to_path_buf(),
            vec![
                crate::SourceFile::new("sdk/lib/rust/compress/src/lib.rs".into(), bad.into()),
                crate::SourceFile::new("sdk/lib/rtl/compress.c".into(), c.into()),
            ],
        );
        assert_eq!(
            StoreOnFailure.run(&corpus).unwrap(),
            vec![
                "sdk/lib/rust/compress/src/lib.rs:1: store-on-failure RtlDecompressFragment(final_size)"
            ]
        );
        let good = r#"pub unsafe extern "system" fn RtlDecompressFragment(format:u16, uncompressed:*mut u8, uncompressed_size:u32, compressed:*const u8, compressed_size:u32, offset:u32, final_size:*mut u32, workspace:*mut u8) -> u32 { let result=decompress(uncompressed,compressed,offset); if result.status == STATUS_SUCCESS && !final_size.is_null() { *final_size=result.bytes_written as u32; } result.status }"#;
        let corpus = Corpus::new(
            Path::new("/").to_path_buf(),
            vec![
                crate::SourceFile::new("sdk/lib/rust/compress/src/lib.rs".into(), good.into()),
                crate::SourceFile::new("sdk/lib/rtl/compress.c".into(), c.into()),
            ],
        );
        assert!(StoreOnFailure.run(&corpus).unwrap().is_empty());
    }
}
