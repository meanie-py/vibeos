//! Keep the hand-transcribed C va_start shims faithful to their source bodies.
use crate::{Corpus, Rule, SourceFile};
use regex::Regex;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::LazyLock,
};
pub struct VarargsShimParity;
const WCS: &str = "dll/win32/msvcrt/wcs.c";
const SHIM: &str = "dll/win32/msvcrt/rust_varargs_shim.c";
const SCAN: &str = "dll/win32/msvcrt/scanf.c";
const SSHIM: &str = "dll/win32/msvcrt/rust_scanf_shim.c";
const FILE: &str = "dll/win32/msvcrt/file.c";
const FSHIM: &str = "dll/win32/msvcrt/rust_file_varargs_shim.c";
const STDIO: &str = "sdk/include/crt/stdio.h";
impl Rule for VarargsShimParity {
    fn name(&self) -> &'static str {
        "scan_varargs_shim_parity"
    }
    fn extra_files(&self) -> &'static [&'static str] {
        &[WCS, SHIM, SCAN, SSHIM, FILE, FSHIM, STDIO]
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let fixture = c.files.iter().any(|f| {
            [
                "wcs.c",
                "rust_varargs_shim.c",
                "scanf.c",
                "rust_scanf_shim.c",
                "file.c",
                "rust_file_varargs_shim.c",
            ]
            .contains(
                &f.path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .as_ref(),
            ) && !f.path.starts_with("dll/win32/msvcrt")
        });
        let wcs = find_file(c, WCS).map(|f| f.text.as_str());
        let mut out = Vec::new();
        for (r, s) in [(WCS, SHIM), (SCAN, SSHIM), (FILE, FSHIM)] {
            let (Some(rt), Some(st)) = (find_file(c, r), find_file(c, s)) else {
                if fixture {
                    continue;
                }
                let missing = if find_file(c, r).is_none() { r } else { s };
                out.push(format!("{missing}: missing"));
                continue;
            };
            let stdio = find_file(c, STDIO).map(|x| x.text.as_str());
            out.extend(check_pair(
                r,
                &rt.text,
                s,
                &st.text,
                stdio,
                wcs.unwrap_or(&rt.text),
            ));
        }
        Ok(out)
    }
}
fn find_file<'a>(c: &'a Corpus, path: &str) -> Option<&'a SourceFile> {
    c.files
        .iter()
        .find(|f| f.path == std::path::Path::new(path))
        .or_else(|| {
            c.files.iter().find(|f| {
                f.path
                    .file_name()
                    .is_some_and(|n| n == std::path::Path::new(path).file_name().unwrap())
            })
        })
}
static COMMENT_BLOCK: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"(?s)/\*.*?\*/").unwrap());
static COMMENT_LINE: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"//[^\n]*").unwrap());
static IDENT: LazyLock<Regex> = LazyLock::new(|| crate::regexes::cached(r"\w+").unwrap());
static CALL: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"(?:=|\breturn)\s*([A-Za-z_]\w*)\s*\(").unwrap());
static WINAPIV_FN: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"(?m)^int WINAPIV (\w+)\s*\(").unwrap());
static CDECL_FN: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"(?m)^int CDECL (\w+)\s*\(").unwrap());
static SHIM_DEF: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"(?m)^int WINAPIV (\w+)\s*\(").unwrap());
static VLIST: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"\bva_list\s+(\w+)\s*;").unwrap());
static VASTART: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"\bva_start\s*\(\s*\w+\s*,\s*(\w+)\s*\)").unwrap());
static IF: LazyLock<Regex> = LazyLock::new(|| crate::regexes::cached(r"^#\s*if").unwrap());
static GATE_ZERO: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"_MSVCR_VER\s*==\s*0").unwrap());
static GATE_DEF: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"^int (?:WINAPIV|CDECL) (\w+)\s*\(").unwrap());
fn strip_comments(s: &str) -> String {
    COMMENT_LINE
        .replace_all(&COMMENT_BLOCK.replace_all(s, " "), " ")
        .into_owned()
}
fn split_args(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let (mut depth, mut start) = (0i32, 0usize);
    for (i, ch) in s.char_indices() {
        if ch == '(' {
            depth += 1
        } else if ch == ')' {
            depth -= 1
        }
        if ch == ',' && depth == 0 {
            let a = s[start..i].trim();
            out.push(a.to_owned());
            start = i + 1;
        }
    }
    let last = s[start..].trim();
    if !last.is_empty() {
        out.push(last.to_owned())
    }
    out
}
fn close_match(s: &str, open: usize, left: char, right: char) -> Option<usize> {
    let mut d = 0;
    for (i, ch) in s.char_indices().skip_while(|(i, _)| *i < open) {
        if ch == left {
            d += 1
        } else if ch == right {
            d -= 1;
            if d == 0 {
                return Some(i);
            }
        }
    }
    None
}
fn body_of(s: &str, open: usize) -> Option<(String, usize)> {
    close_match(s, open, '{', '}').map(|end| (s[open + 1..end].to_owned(), end))
}
fn gated_in(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut stack = Vec::new();
    for line in text.lines() {
        let s = line.trim();
        if IF.is_match(s) {
            stack.push(s.contains("_MSVCR_VER") && !GATE_ZERO.is_match(s));
        } else if s.starts_with("#else") && !stack.is_empty() {
            let n = stack.len() - 1;
            stack[n] = !stack[n];
        } else if s.starts_with("#endif") && !stack.is_empty() {
            stack.pop();
        }
        if let Some(m) = GATE_DEF.captures(s) {
            if stack.iter().any(|x| *x) {
                out.insert(m[1].to_owned());
            }
        }
    }
    out
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Shell {
    params: Vec<String>,
    callee: String,
    args: Vec<String>,
}
fn shells(text: &str, keyword: &str) -> BTreeMap<String, Shell> {
    let text = strip_comments(text);
    let gated = gated_in(&text);
    let re = if keyword == "WINAPIV" {
        &*WINAPIV_FN
    } else {
        &*CDECL_FN
    };
    let mut found = BTreeMap::new();
    for m in re.captures_iter(&text) {
        let name = m[1].to_owned();
        if gated.contains(&name) {
            continue;
        }
        let full = m.get(0).unwrap();
        let open = full.end() - 1;
        let Some(pclose) = close_match(&text, open, '(', ')') else {
            continue;
        };
        let params = split_args(&text[open + 1..pclose]);
        let pnames: Vec<_> = params
            .iter()
            .filter(|a| !matches!(a.trim(), "..." | "void" | ""))
            .filter_map(|a| IDENT.find_iter(a).last().map(|m| m.as_str().to_owned()))
            .collect();
        let Some(brace) = text[pclose + 1..].find('{').map(|i| pclose + 1 + i) else {
            continue;
        };
        let Some((body, _)) = body_of(&text, brace) else {
            continue;
        };
        let mut pos = 0;
        let mut cm = None;
        loop {
            let Some(m) = CALL.find_at(&body, pos) else {
                break;
            };
            let cap = CALL.captures_at(&body, m.start()).unwrap();
            let callname = cap[1].to_owned();
            if !["va_arg", "va_start", "va_end", "va_copy"].contains(&callname.as_str()) {
                cm = Some((m, callname));
                break;
            }
            pos = cap.get(1).unwrap().end();
        }
        let Some((mat, callee)) = cm else { continue };
        let call_open = mat.end() - 1;
        let Some(call_close) = close_match(&body, call_open, '(', ')') else {
            continue;
        };
        let args = split_args(&body[call_open + 1..call_close]);
        found.insert(
            name,
            Shell {
                params: pnames,
                callee,
                args,
            },
        );
    }
    found
}
fn body_facts(text: &str, name: &str) -> (String, Option<String>) {
    let stripped = strip_comments(text);
    let header = crate::regexes::cached(&format!(r"(?m)^int WINAPIV {}\s*\(", regex::escape(name)))
        .expect("escaped function name is a valid regex");
    let Some(m) = header.find(&stripped) else {
        return ("valist".into(), None);
    };
    let Some(brace) = stripped[m.end()..].find('{').map(|i| m.end() + i) else {
        return ("valist".into(), None);
    };
    let Some((body, _)) = body_of(&stripped, brace) else {
        return ("valist".into(), None);
    };
    let v = VLIST
        .captures(&body)
        .map(|x| x[1].to_owned())
        .unwrap_or_else(|| "valist".into());
    let a = VASTART.captures(&body).map(|x| x[1].to_owned());
    (v, a)
}
fn canon(args: &[String], valist: &str) -> Vec<String> {
    let re = crate::regexes::cached(&format!(r"\b{}\b", regex::escape(valist)))
        .expect("escaped va_list name is a valid regex");
    args.iter()
        .map(|a| re.replace_all(a, "V").trim().to_owned())
        .collect()
}
fn py_optional(value: &Option<String>) -> String {
    value.as_deref().unwrap_or("None").to_owned()
}
fn check_pair(
    ref_name: &str,
    ref_text: &str,
    shim_name: &str,
    shim_text: &str,
    stdio: Option<&str>,
    global_wcs: &str,
) -> Vec<String> {
    let mut findings = Vec::new();
    let rf = shells(ref_text, "WINAPIV");
    let sub = shells(shim_text, "WINAPIV");
    let fwd = shells(ref_text, "CDECL");
    let mut rewrites = Vec::<(String, String, Vec<usize>, HashMap<usize, String>)>::new();
    for (wrapper, want) in [
        (
            "_vswprintf_p_l",
            vec!["buffer", "length", "format", "0", "locale", "args"],
        ),
        (
            "_vscwprintf_p_l",
            vec!["NULL", "INT_MAX", "format", "0", "locale", "args"],
        ),
    ] {
        if let Some(x) = fwd.get(wrapper) {
            if x.callee != "vswprintf_p_l_opt"
                || x.args.iter().map(String::as_str).collect::<Vec<_>>() != want
            {
                findings.push(format!("{wrapper}: {} no longer forwards to vswprintf_p_l_opt({}) (now {}({})) — the shim's substitution is no longer an identity",PathName::base(ref_name),want.join(", "),x.callee,x.args.join(", ")));
                continue;
            }
            let keep: Option<Vec<_>> = x
                .params
                .iter()
                .map(|p| want.iter().position(|v| v == p))
                .collect();
            let Some(keep) = keep else {
                findings.push(format!(
                    "{wrapper}: forwards something other than its own parameters"
                ));
                continue;
            };
            let fixed = (0..want.len())
                .filter(|i| !keep.contains(i))
                .map(|i| (i, want[i].to_string()))
                .collect();
            rewrites.push(("vswprintf_p_l_opt".into(), wrapper.into(), keep, fixed));
        }
    }
    let needs_vsnprintf = rf.values().any(|x| x.callee == "vsnprintf");
    if needs_vsnprintf && stdio.is_some_and(|s| !s.contains("#define vsnprintf _vsnprintf")) {
        findings.push("sdk/include/crt/stdio.h no longer defines vsnprintf -> _vsnprintf; the shim calls _vsnprintf directly where wcs.c writes vsnprintf".into());
    }
    for (name, sh) in &sub {
        let Some(r) = rf.get(name) else {
            findings.push(format!(
                "{name}: defined in the shim but not in {}",
                PathName::base(ref_name)
            ));
            continue;
        };
        if sh.params != r.params {
            findings.push(format!(
                "{name}: parameter names differ — shim({}) vs {}({})",
                sh.params.join(", "),
                PathName::base(ref_name),
                r.params.join(", ")
            ));
            continue;
        }
        let mut rc = r.callee.clone();
        let mut ra = r.args.clone();
        if rc == "vsnprintf" {
            rc = "_vsnprintf".into()
        }
        rewrites.sort_by_key(|x| std::cmp::Reverse(x.3.len()));
        for (src, dst, keep, fixed) in &rewrites {
            if rc == *src
                && ra.len() == keep.len() + fixed.len()
                && fixed.iter().all(|(i, v)| ra[*i] == *v)
            {
                rc = dst.clone();
                ra = keep.iter().map(|i| ra[*i].clone()).collect();
                break;
            }
        }
        let (v_sub, anchor_sub) = body_facts(shim_text, name);
        let (v_ref, anchor_ref) = body_facts(ref_text, name);
        let a_sub = canon(&sh.args, &v_sub);
        let a_ref = canon(&ra, &v_ref);
        if anchor_sub != anchor_ref {
            findings.push(format!("{name}: va_start anchored on {}, {} anchors on {} — every variadic argument shifts by one slot",py_optional(&anchor_sub),PathName::base(ref_name),py_optional(&anchor_ref)));
        }
        if rc != sh.callee {
            findings.push(format!(
                "{name}: delegates to {}, {} delegates to {rc}",
                sh.callee,
                PathName::base(ref_name)
            ));
        } else if a_sub != a_ref {
            findings.push(format!(
                "{name}: argument drift — shim {}({}) vs {} {}({})",
                sh.callee,
                a_sub.join(", "),
                PathName::base(ref_name),
                rc,
                a_ref.join(", ")
            ));
        }
    }
    for n in rf.keys() {
        if !sub.contains_key(n) {
            findings.push(format!(
                "{n}: in {} but MISSING from the shim — its export stays on the C engine",
                PathName::base(ref_name)
            ));
        }
    }
    let gated_ref = gated_in(&strip_comments(global_wcs));
    let gated_sub = gated_in(&strip_comments(shim_text));
    let defs: BTreeSet<_> = SHIM_DEF
        .captures_iter(shim_text)
        .map(|m| m[1].to_owned())
        .collect();
    for n in gated_ref.intersection(&defs) {
        if !gated_sub.contains(n) {
            findings.push(format!("{n}: _MSVCR_VER-gated in {} but UNGATED in the shim — adds a symbol this vintage does not have",PathName::base(ref_name)));
        }
    }
    for n in gated_sub.difference(&gated_ref) {
        findings.push(format!(
            "{n}: gated in the shim but not in {} — export lost",
            PathName::base(ref_name)
        ));
    }
    findings
        .into_iter()
        .map(|x| format!("{shim_name}: {x}"))
        .collect()
}
struct PathName;
impl PathName {
    fn base(p: &str) -> &str {
        p.rsplit('/').next().unwrap_or(p)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    const WCS_FIX: &str = r#"int WINAPIV _snprintf(char *str, size_t len, const char *format, ...)
{
    int retval;
    va_list valist;
    va_start(valist, format);
    retval = vsnprintf(str, len, format, valist);
    va_end(valist);
    return retval;
}

int WINAPIV _snprintf_s_l(char *str, size_t len, size_t count,
        const char *format, _locale_t locale, ...)
{
    int retval;
    va_list valist;
    va_start(valist, locale);
    retval = _vsnprintf_s_l(str, len, count, format, locale, valist);
    va_end(valist);
    return retval;
}
"#;
    const SHIM_GOOD: &str = r#"int WINAPIV _snprintf(char *str, size_t len, const char *format, ...)
{
    int retval;
    va_list valist;
    va_start(valist, format);
    retval = _vsnprintf(str, len, format, valist);
    va_end(valist);
    return retval;
}

int WINAPIV _snprintf_s_l(char *str, size_t len, size_t count,
        const char *format, _locale_t locale, ...)
{
    int retval;
    va_list valist;
    va_start(valist, locale);
    retval = _vsnprintf_s_l(str, len, count, format, locale, valist);
    va_end(valist);
    return retval;
}
"#;
    const SHIM_STDIO: &str = "#define vsnprintf _vsnprintf\n";
    #[test]
    fn control_bad_and_good_pairs() {
        let bad = SHIM_GOOD.replace(
            "_vsnprintf_s_l(str, len, count,",
            "_vsnprintf_s_l(str, count, len,",
        );
        let findings = check_pair(
            "wcs.c",
            WCS_FIX,
            "rust_varargs_shim.c",
            &bad,
            Some(SHIM_STDIO),
            WCS_FIX,
        );
        assert!(
            findings
                .iter()
                .any(|x| x.contains("rust_varargs_shim.c: _snprintf_s_l: argument drift")),
            "{findings:?}"
        );
        assert!(check_pair(
            "wcs.c",
            WCS_FIX,
            "rust_varargs_shim.c",
            SHIM_GOOD,
            Some(SHIM_STDIO),
            WCS_FIX
        )
        .is_empty());
    }
    #[test]
    fn corpus_fixture_control() {
        let bad = SHIM_GOOD.replace(
            "_vsnprintf_s_l(str, len, count,",
            "_vsnprintf_s_l(str, count, len,",
        );
        let c = Corpus::new(
            PathBuf::from("."),
            vec![
                SourceFile::new("wcs.c".into(), WCS_FIX.into()),
                SourceFile::new("rust_varargs_shim.c".into(), bad),
                SourceFile::new("stdio.h".into(), SHIM_STDIO.into()),
            ],
        );
        assert!(VarargsShimParity
            .run(&c)
            .unwrap()
            .iter()
            .any(|x| x.contains("rust_varargs_shim.c: _snprintf_s_l")));
    }
}
