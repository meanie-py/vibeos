use crate::{Corpus, Rule, SourceFile, SourceSet};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

pub struct VerifyTranslationLiterals;
const PAIRS: &[(&str, &str)] = &[
    (
        "sdk/lib/rust/msvcrt-crt-rust/src/msvcrt",
        "dll/win32/msvcrt",
    ),
    ("sdk/lib/rust/aclui-rust/src/aclui", "dll/win32/aclui"),
    ("sdk/lib/rust/authz-rust/src/authz", "dll/win32/authz"),
    (
        "sdk/lib/rust/browser-rust/src/browser",
        "base/services/browser",
    ),
    ("sdk/lib/rust/desk-rust/src/desk", "dll/cpl/desk"),
    ("sdk/lib/rust/intl-rust/src/intl", "dll/cpl/intl"),
    ("sdk/lib/rust/mprapi-rust/src/mprapi", "dll/win32/mprapi"),
    ("sdk/lib/rust/msv1_0-rust/src/msv1_0", "dll/win32/msv1_0"),
    ("sdk/lib/rust/msv1_0-rust/src/ntlm", "dll/win32/msv1_0/ntlm"),
    ("sdk/lib/rust/mswsock-rust/src/mswsock", "dll/win32/mswsock"),
    ("sdk/lib/rust/netcfgx-rust/src/netcfgx", "dll/win32/netcfgx"),
    ("sdk/lib/rust/netid-rust/src/netid", "dll/win32/netid"),
    ("sdk/lib/rust/ntmarta-rust/src/ntmarta", "dll/win32/ntmarta"),
    (
        "sdk/lib/rust/rasadhlp-rust/src/rasadhlp",
        "dll/win32/rasadhlp",
    ),
    (
        "sdk/lib/rust/rasapi32-rust/src/rasapi32",
        "dll/win32/rasapi32",
    ),
    ("sdk/lib/rust/rasdlg-rust/src/rasdlg", "dll/win32/rasdlg"),
    ("sdk/lib/rust/rasman-rust/src/rasman", "dll/win32/rasman"),
    ("sdk/lib/rust/samlib-rust/src/samlib", "dll/win32/samlib"),
    ("sdk/lib/rust/snmpapi-rust/src/snmpapi", "dll/win32/snmpapi"),
    ("sdk/lib/rust/sysdm-rust/src/sysdm", "dll/cpl/sysdm"),
    (
        "sdk/lib/rust/syssetup-rust/src/syssetup",
        "dll/win32/syssetup",
    ),
    (
        "sdk/lib/rust/umpnpmgr-rust/src/umpnpmgr",
        "base/services/umpnpmgr",
    ),
    ("sdk/lib/rust/userenv-rust/src/userenv", "dll/win32/userenv"),
    (
        "sdk/lib/rust/wkssvc-rust/src/wkssvc",
        "base/services/wkssvc",
    ),
    ("sdk/lib/rust/wlanapi-rust/src/wlanapi", "dll/win32/wlanapi"),
    (
        "sdk/lib/rust/wlansvc-rust/src/wlansvc",
        "base/services/wlansvc",
    ),
    ("sdk/lib/rust/wldap32-rust/src/wldap32", "dll/win32/wldap32"),
    ("sdk/lib/rust/wshirda-rust/src/wshirda", "dll/win32/wshirda"),
    (
        "sdk/lib/rust/wshtcpip-rust/src/wshtcpip",
        "dll/win32/wshtcpip",
    ),
    ("sdk/lib/rust/wsock32-rust/src/wsock32", "dll/win32/wsock32"),
    (
        "sdk/lib/rust/notepad-rust/src/notepad",
        "base/applications/notepad",
    ),
    (
        "sdk/lib/rust/cacls-rust/src/cacls",
        "base/applications/cacls",
    ),
    (
        "sdk/lib/rust/charmap-rust/src/charmap",
        "base/applications/charmap",
    ),
    (
        "sdk/lib/rust/clipbrd-rust/src/clipbrd",
        "base/applications/clipbrd",
    ),
    (
        "sdk/lib/rust/control-rust/src/control",
        "base/applications/control",
    ),
    (
        "sdk/lib/rust/dxdiag-rust/src/dxdiag",
        "base/applications/dxdiag",
    ),
    (
        "sdk/lib/rust/extrac32-rust/src/extrac32",
        "base/applications/extrac32",
    ),
    (
        "sdk/lib/rust/findstr-rust/src/findstr",
        "base/applications/findstr",
    ),
    (
        "sdk/lib/rust/fontview-rust/src/fontview",
        "base/applications/fontview",
    ),
    (
        "sdk/lib/rust/logoff-rust/src/logoff",
        "base/applications/logoff",
    ),
    (
        "sdk/lib/rust/magnify-rust/src/magnify",
        "base/applications/magnify",
    ),
    ("sdk/lib/rust/mmc-rust/src/mmc", "base/applications/mmc"),
    (
        "sdk/lib/rust/mplay32-rust/src/mplay32",
        "base/applications/mplay32",
    ),
    (
        "sdk/lib/rust/msconfig-rust/src/msconfig",
        "base/applications/msconfig",
    ),
    (
        "sdk/lib/rust/mstsc-rust/src/mstsc",
        "base/applications/mstsc",
    ),
    ("sdk/lib/rust/osk-rust/src/osk", "base/applications/osk"),
    (
        "sdk/lib/rust/runas-rust/src/runas",
        "base/applications/runas",
    ),
    ("sdk/lib/rust/sc-rust/src/sc", "base/applications/sc"),
    (
        "sdk/lib/rust/sdbinst-rust/src/sdbinst",
        "base/applications/sdbinst",
    ),
    (
        "sdk/lib/rust/shutdown-rust/src/shutdown",
        "base/applications/shutdown",
    ),
    (
        "sdk/lib/rust/winver-rust/src/winver",
        "base/applications/winver",
    ),
];
const C_ROOTS: &[&str] = &[
    "base/applications/cacls",
    "base/applications/charmap",
    "base/applications/clipbrd",
    "base/applications/control",
    "base/applications/dxdiag",
    "base/applications/extrac32",
    "base/applications/findstr",
    "base/applications/fontview",
    "base/applications/logoff",
    "base/applications/magnify",
    "base/applications/mmc",
    "base/applications/mplay32",
    "base/applications/msconfig",
    "base/applications/mstsc",
    "base/applications/notepad",
    "base/applications/osk",
    "base/applications/runas",
    "base/applications/sc",
    "base/applications/sdbinst",
    "base/applications/shutdown",
    "base/applications/winver",
    "base/services/browser",
    "base/services/umpnpmgr",
    "base/services/wkssvc",
    "base/services/wlansvc",
    "dll/cpl/desk",
    "dll/cpl/intl",
    "dll/cpl/sysdm",
    "dll/win32/aclui",
    "dll/win32/authz",
    "dll/win32/mprapi",
    "dll/win32/msv1_0",
    "dll/win32/msv1_0/ntlm",
    "dll/win32/msvcrt",
    "dll/win32/mswsock",
    "dll/win32/netcfgx",
    "dll/win32/netid",
    "dll/win32/ntmarta",
    "dll/win32/rasadhlp",
    "dll/win32/rasapi32",
    "dll/win32/rasdlg",
    "dll/win32/rasman",
    "dll/win32/samlib",
    "dll/win32/snmpapi",
    "dll/win32/syssetup",
    "dll/win32/userenv",
    "dll/win32/wlanapi",
    "dll/win32/wldap32",
    "dll/win32/wshirda",
    "dll/win32/wshtcpip",
    "dll/win32/wsock32",
];
const INPUTS: &[SourceSet] = &[
    SourceSet::tree("translation-literals-rust", &["sdk/lib/rust"], &["rs"], &[]).with_inputs(),
    SourceSet::tree("translation-literals-c", C_ROOTS, &["c"], &[]).with_inputs(),
];
static C_STR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#""((?:[^"\\\n]|\\.)*)""#).unwrap());
static C_COMMENT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/|//[^\n]*").unwrap());
static INCLUDE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?m)^\s*#\s*include\s+"[^"]*""#).unwrap());
static EVAL_NUM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d+$").unwrap());
static DEFINED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^defined\s*\(?\s*(\w+)\s*\)?$").unwrap());
static NOT_DEFINED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^!\s*defined\s*\(?\s*(\w+)\s*\)?$").unwrap());
static CMP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\w+)\s*(==|!=|<=|>=|<|>)\s*(\d+)$").unwrap());
static PP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^#\s*(ifdef|ifndef|if|elif|else|endif)\b(.*)").unwrap());
static WIDE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?:(?:[^\\]|\\[^0])\\0)+$").unwrap());
static CFG_ANY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"#\[cfg\(any\(\)\)\]").unwrap());
static RUST_ATTR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"#!?\[").unwrap());
static ARCH_ARM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"#\[cfg\((not\()?target_arch\s*=\s*"[^"]+"\)?\)\]\s*\n(?:\s*#\[[^\]]*\]\s*\n)*\s*(?:pub\s+)?(?:unsafe\s+)?(?:extern\s+"[^"]*"\s+)?fn\s+(\w+)"#).unwrap()
});
static RUST_STRING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"b"((?:[^"\\\n]|\\.)*)""#).unwrap());
static ARRAY_OPEN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[").unwrap());
static ARRAY_ELEM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"b?'(\\.|[^'])'\s*as\s*(?:u16|TCHAR|WCHAR|wchar_t|USHORT|WORD)|0[xX]([0-9a-fA-F]+)(?:u\d+)?(?:\s*as\s*(?:u16|TCHAR|WCHAR|wchar_t|USHORT|WORD))?|(\d+)(?:u\d+)?(?:\s*as\s*(?:u16|TCHAR|WCHAR|wchar_t|USHORT|WORD))?"#).unwrap()
});
static ARRAY_ELEM_VALID: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\A\s*(?:b?'(?:\\.|[^'])'\s*as\s*(?:u16|TCHAR|WCHAR|wchar_t|USHORT|WORD)|0[xX][0-9a-fA-F]+(?:u\d+)?(?:\s*as\s*(?:u16|TCHAR|WCHAR|wchar_t|USHORT|WORD))?|\d+(?:u\d+)?(?:\s*as\s*(?:u16|TCHAR|WCHAR|wchar_t|USHORT|WORD))?)\s*\z"#).unwrap()
});

fn array_elems_only(body: &str) -> bool {
    let body = body.trim();
    if body.is_empty() {
        return false;
    }
    let mut elems: Vec<_> = body.split(',').map(str::trim).collect();
    if elems.last() == Some(&"") {
        elems.pop();
    }
    !elems.is_empty()
        && elems
            .into_iter()
            .all(|elem| ARRAY_ELEM_VALID.is_match(elem))
}

type Counts = BTreeMap<String, usize>;
const CHANNELS: &[&str] = &[
    "TRACE",
    "WARN",
    "ERR",
    "FIXME",
    "MESSAGE",
    "DPRINT",
    "DPRINT1",
    "DPRINT2",
    "DbgPrint",
    "DbgPrintEx",
];
const C_ASM: &[&str] = &[
    "__ASM_GLOBAL_FUNC",
    "__ASM_STDCALL_FUNC",
    "__ASM_DEFINE_FUNC",
    "__volatile__",
    "__asm__",
    "__asm",
    "asm",
];
const RUST_ASM: &[&str] = &[
    "global_asm!",
    "asm!",
    "core::arch::global_asm!",
    "core::arch::asm!",
];
fn is_word(ch: char) -> bool {
    ch == '_' || ch.is_alphanumeric()
}
fn strip_calls(src: &str, names: &[&str]) -> String {
    let mut out = String::new();
    let mut at = 0;
    loop {
        let mut best: Option<(usize, usize)> = None;
        for name in names {
            let mut search = at;
            while let Some(rel) = src[search..].find(name) {
                let start = search + rel;
                let end = start + name.len();
                let prev_ok = src[..start].chars().next_back().is_none_or(|c| !is_word(c));
                let mut j = end;
                while j < src.len() {
                    let ch = src[j..].chars().next().unwrap();
                    if !ch.is_whitespace() {
                        break;
                    }
                    j += ch.len_utf8();
                }
                if prev_ok
                    && j < src.len()
                    && ['(', '[', '{'].contains(&src[j..].chars().next().unwrap())
                {
                    best =
                        Some(best.map_or((start, j), |b| if start < b.0 { (start, j) } else { b }));
                    break;
                }
                search = end;
            }
        }
        let Some((start, open)) = best else {
            out.push_str(&src[at..]);
            break;
        };
        out.push_str(&src[at..start]);
        let b = src.as_bytes();
        let mut i = open;
        let mut depth = 0i32;
        let mut quote = None;
        while i < b.len() {
            let ch = b[i] as char;
            if let Some(q) = quote {
                if ch == '\\' {
                    i += 2;
                    continue;
                }
                if ch == q {
                    quote = None;
                }
            } else if ch == '"' || ch == '\'' {
                quote = Some(ch);
            } else if ['(', '[', '{'].contains(&ch) {
                depth += 1;
            } else if ")]}".contains(ch) {
                depth -= 1;
                if depth == 0 {
                    i += 1;
                    break;
                }
            }
            i += 1;
        }
        at = i.min(src.len());
    }
    out
}
fn strip_c_comments(src: &str) -> String {
    C_COMMENT.replace_all(src, " ").into_owned()
}
fn eval_cond(expr: &str) -> Option<bool> {
    let e = expr.trim();
    if EVAL_NUM.is_match(e) {
        return Some(e.parse::<u64>().ok()? != 0);
    }
    if let Some(c) = DEFINED.captures(e) {
        return match c.get(1)?.as_str() {
            "__REACTOS__" | "__UCRTSUPPORT__" => Some(true),
            "_MSVCR_VER" => Some(true),
            "_UCRT" | "_CRTDLL" => Some(false),
            _ => None,
        };
    }
    if let Some(c) = NOT_DEFINED.captures(e) {
        return eval_cond(&format!("defined({})", c.get(1)?.as_str())).map(|x| !x);
    }
    if let Some(c) = CMP.captures(e) {
        let n = c.get(1)?.as_str();
        let left = match n {
            "__REACTOS__" | "__UCRTSUPPORT__" => 1,
            "_MSVCR_VER" => 0,
            _ => return None,
        };
        let right = c.get(3)?.as_str().parse::<i64>().ok()?;
        return Some(match c.get(2)?.as_str() {
            "==" => left == right,
            "!=" => left != right,
            "<" => (left as i64) < right,
            ">" => (left as i64) > right,
            "<=" => (left as i64) <= right,
            ">=" => (left as i64) >= right,
            _ => return None,
        });
    }
    None
}
fn strip_untaken_arms(src: &str) -> String {
    let mut out = Vec::new();
    let mut stack: Vec<(bool, bool)> = Vec::new();
    for line in src.split('\n') {
        let s = line.trim();
        if let Some(m) = PP.captures(s) {
            let kind = m.get(1).unwrap().as_str();
            let rest = m.get(2).unwrap().as_str();
            match kind {
                "ifdef" | "ifndef" | "if" => {
                    let cond = match kind {
                        "ifdef" => format!("defined({})", rest.trim()),
                        "ifndef" => format!("!defined({})", rest.trim()),
                        _ => rest.to_string(),
                    };
                    let v = eval_cond(&cond);
                    stack.push((v.is_some(), v.unwrap_or(true)));
                }
                "elif" if !stack.is_empty() => {
                    let (decided, keeping) = *stack.last().unwrap();
                    let v = eval_cond(rest);
                    *stack.last_mut().unwrap() = (
                        decided && v.is_some(),
                        if !decided || v.is_none() {
                            true
                        } else {
                            v.unwrap() && !keeping
                        },
                    );
                }
                "else" if !stack.is_empty() => {
                    let (d, k) = *stack.last().unwrap();
                    *stack.last_mut().unwrap() = (d, if d { !k } else { true });
                }
                "endif" if !stack.is_empty() => {
                    stack.pop();
                }
                _ => {}
            }
            out.push(String::new());
            continue;
        }
        out.push(if stack.iter().all(|(_, keep)| *keep) {
            line.to_string()
        } else {
            String::new()
        });
    }
    out.join("\n")
}
fn norm_escapes(s: &str) -> String {
    s.replace('\t', r"\t")
        .replace('\n', r"\n")
        .replace('\r', r"\r")
}
fn c_strings(src: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut prev = None;
    for c in C_STR.captures_iter(src) {
        let m = c.get(0).unwrap();
        let gap = prev.map(|p: usize| src[p..m.start()].trim());
        let merge = gap.is_some_and(|g| {
            g.is_empty()
                || ["L", "u", "U", "u8"].contains(&g)
                || Regex::new(r"^\)\s*(?:_T|TEXT|_TEXT)\s*\(\s*L?$")
                    .unwrap()
                    .is_match(g)
        });
        if merge {
            out.last_mut().unwrap().push_str(c.get(1).unwrap().as_str())
        } else {
            out.push(c.get(1).unwrap().as_str().to_string())
        }
        prev = Some(m.end());
    }
    out
}
fn is_nul_escape_at(s: &str, i: usize) -> bool {
    if s.get(i..i + 2) != Some(r"\0") {
        return false;
    }
    let bytes = s.as_bytes();
    let mut n = 0;
    let mut j = i;
    while j > 0 && bytes[j - 1] == b'\\' {
        n += 1;
        j -= 1
    }
    n % 2 == 0
}
fn strip_trailing_nul(s: &str) -> String {
    if s.len() >= 2 && is_nul_escape_at(s, s.len() - 2) {
        s[..s.len() - 2].to_string()
    } else {
        s.to_string()
    }
}
fn dewiden(lit: &str) -> String {
    let lit = lit.replace(r"\x00", r"\0");
    let mut body = lit.as_str();
    if lit.ends_with(r"\0\0")
        && is_nul_escape_at(&lit, lit.len() - 4)
        && WIDE.is_match(&lit[..lit.len() - 4])
    {
        body = &lit[..lit.len() - 4]
    } else if lit.len() >= 2 && is_nul_escape_at(&lit, lit.len() - 2) {
        body = &lit[..lit.len() - 2]
    }
    if body.is_empty() || !WIDE.is_match(body) {
        return body.to_string();
    }
    let mut out = String::new();
    let b = body.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\\' {
            out.push_str(&body[i..i + 2]);
            i += 2
        } else {
            out.push(b[i] as char);
            i += 1
        }
        if body.get(i..i + 2) != Some(r"\0") {
            return body.to_string();
        }
        i += 2;
    }
    out
}
fn add_count(c: &mut Counts, s: String, n: usize) {
    *c.entry(s).or_default() += n;
}
fn c_literals(text: &str) -> Counts {
    let mut src = strip_c_comments(text);
    src = strip_untaken_arms(&src);
    src = INCLUDE.replace_all(&src, " ").into_owned();
    let mut names = CHANNELS.to_vec();
    names.extend_from_slice(C_ASM);
    src = strip_calls(&src, &names);
    let mut out = Counts::new();
    for s in c_strings(&src) {
        let x = norm_escapes(&strip_trailing_nul(&s));
        add_count(&mut out, x, 1);
    }
    out
}

fn chars(text: &str) -> Vec<char> {
    text.chars().collect()
}
fn chars_to_string(cs: &[char]) -> String {
    cs.iter().collect()
}
fn skip_cfg_any_items(text: &str) -> String {
    let cs = chars(text);
    let mut out = String::new();
    let mut i = 0;
    for m in CFG_ANY.find_iter(text) {
        let start = text[..m.start()].chars().count();
        let end = text[..m.end()].chars().count();
        out.push_str(&chars_to_string(&cs[i..start]));
        let j = end;
        let brace = cs[j..].iter().position(|c| *c == '{').map(|x| j + x);
        let semi = cs[j..].iter().position(|c| *c == ';').map(|x| j + x);
        let comma = cs[j..]
            .windows(2)
            .position(|w| w == [',', '\n'])
            .map(|x| j + x);
        let end_mark = [semi, comma].into_iter().flatten().min();
        if brace.is_none() || end_mark.is_some_and(|x| x < brace.unwrap()) {
            i = end_mark.map_or(j, |x| x + 1);
            continue;
        }
        let mut depth = 0i32;
        let mut k = brace.unwrap();
        while k < cs.len() {
            if cs[k] == '{' {
                depth += 1
            } else if cs[k] == '}' {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            k += 1;
        }
        i = k + 1;
    }
    out.push_str(&chars_to_string(&cs[i..]));
    out
}
fn strip_rust_comments(text: &str) -> String {
    let cs = chars(text);
    let mut out = String::new();
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        if c == '\'' {
            let sample = chars_to_string(&cs[i..(i + 4).min(cs.len())]);
            let ch = Regex::new(r"^'(?:\\.|[^'\\])'").unwrap().is_match(&sample);
            if !ch {
                out.push(c);
                i += 1;
                continue;
            }
        }
        if c == '"' || c == '\'' {
            let q = c;
            let start = i;
            i += 1;
            while i < cs.len() {
                if cs[i] == '\\' {
                    i += 2;
                    continue;
                }
                if cs[i] == q || cs[i] == '\n' {
                    break;
                }
                i += 1;
            }
            let end = (i + 1).min(cs.len());
            out.extend(cs[start..end].iter());
            i = end;
        } else if i + 1 < cs.len() && c == '/' && cs[i + 1] == '/' {
            while i < cs.len() && cs[i] != '\n' {
                i += 1;
            }
        } else if i + 1 < cs.len() && c == '/' && cs[i + 1] == '*' {
            i += 2;
            while i + 1 < cs.len() && !(cs[i] == '*' && cs[i + 1] == '/') {
                i += 1;
            }
            i = (i + 2).min(cs.len());
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}
fn strip_rust_attributes(text: &str) -> String {
    let cs = chars(text);
    let mut out = String::new();
    let mut i = 0;
    while i < cs.len() {
        let tail = chars_to_string(&cs[i..]);
        let Some(m) = RUST_ATTR.find(&tail) else {
            out.extend(cs[i..].iter());
            break;
        };
        let start = i + tail[..m.start()].chars().count();
        let end = i + tail[..m.end()].chars().count();
        out.extend(cs[i..start].iter());
        let mut depth = 0i32;
        let mut j = end - 1;
        while j < cs.len() {
            if cs[j] == '[' {
                depth += 1
            } else if cs[j] == ']' {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            j += 1;
        }
        i = (j + 1).min(cs.len());
    }
    out
}
fn collapse_arch_duals(text: &str) -> String {
    let mut spans: BTreeMap<String, Vec<(bool, usize)>> = BTreeMap::new();
    for c in ARCH_ARM.captures_iter(text) {
        let m = c.get(0).unwrap();
        let name = c.get(2).unwrap().as_str().to_string();
        let start = text[..m.start()].chars().count();
        spans
            .entry(name)
            .or_default()
            .push((c.get(1).is_some(), start));
    }
    let cs = chars(text);
    let mut kill = Vec::new();
    for arms in spans.values() {
        if arms.len() != 2 || arms[0].0 == arms[1].0 {
            continue;
        }
        let start = arms.iter().find(|x| x.0).unwrap().1;
        let mut brace = start;
        while brace < cs.len() && cs[brace] != '{' {
            brace += 1
        }
        if brace == cs.len() {
            continue;
        }
        let (mut depth, mut k) = (0i32, brace);
        while k < cs.len() {
            if cs[k] == '{' {
                depth += 1
            } else if cs[k] == '}' {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            k += 1;
        }
        kill.push((start, k + 1));
    }
    kill.sort();
    let mut out = String::new();
    let mut i = 0;
    for (s, e) in kill {
        if s < i {
            continue;
        }
        out.push_str(&chars_to_string(&cs[i..s]));
        i = e;
    }
    out.push_str(&chars_to_string(&cs[i..]));
    out
}
fn decode_array_char(s: &str) -> Option<char> {
    if !s.starts_with('\\') {
        return s.chars().next();
    }
    match s {
        "\\\\" => Some('\\'),
        "\\'" => Some('\''),
        "\\\"" => Some('"'),
        "\\n" => Some('\n'),
        "\\r" => Some('\r'),
        "\\t" => Some('\t'),
        _ => None,
    }
}
fn reescape(c: char) -> String {
    match c {
        '\\' => r"\\".into(),
        '"' => r#"\""#.into(),
        '\n' => r"\n".into(),
        '\r' => r"\r".into(),
        '\t' => r"\t".into(),
        _ => c.to_string(),
    }
}
fn u16_array_literals(text: &str) -> Vec<String> {
    let mut out = vec![];
    for m in ARRAY_OPEN.find_iter(text) {
        let start = m.start();
        let bytes = text.as_bytes();
        let (mut depth, mut i) = (0i32, start);
        while i < bytes.len() {
            if bytes[i] == b'[' {
                depth += 1
            } else if bytes[i] == b']' {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            i += 1;
        }
        if i >= bytes.len() {
            continue;
        }
        let body = &text[start + 1..i];
        if !array_elems_only(body) {
            continue;
        }
        let mut codes = vec![];
        let mut okay = true;
        for e in ARRAY_ELEM.captures_iter(body) {
            if let Some(ch) = e.get(1) {
                if let Some(c) = decode_array_char(ch.as_str()) {
                    codes.push(c as u32)
                } else {
                    okay = false;
                    break;
                }
            } else if let Some(n) = e.get(2) {
                if let Ok(x) = u32::from_str_radix(n.as_str(), 16) {
                    codes.push(x)
                } else {
                    okay = false;
                    break;
                }
            } else if let Some(n) = e.get(3) {
                if let Ok(x) = n.as_str().parse::<u32>() {
                    codes.push(x)
                } else {
                    okay = false;
                    break;
                }
            }
        }
        if !okay || codes.len() < 2 || codes.last() != Some(&0) {
            continue;
        }
        while codes.last() == Some(&0) {
            codes.pop();
        }
        if codes.is_empty()
            || codes
                .iter()
                .any(|&c| !(c == 9 || c == 10 || c == 13 || (0x20..=0x7e).contains(&c)))
        {
            continue;
        }
        if let Some(s) = codes
            .into_iter()
            .map(char::from_u32)
            .collect::<Option<String>>()
        {
            out.push(s.chars().map(reescape).collect())
        }
    }
    out
}
fn utf16le_text(lit: &str) -> Option<String> {
    let mut parts: Vec<_> = lit.split(r"\0").collect();
    while parts.len() > 1 && parts.last() == Some(&"") {
        parts.pop();
    }
    if parts.len() < 2 || parts.iter().any(|p| p.chars().count() != 1) {
        return None;
    }
    Some(parts.concat())
}
fn normalize(mut cl: Counts, rl: Counts) -> (Counts, Counts) {
    cl.retain(|k, _| !k.is_empty());
    let mut out = Counts::new();
    for (k, v) in rl {
        if k.is_empty() || Regex::new(r"^(?:\\0)+$").unwrap().is_match(&k) {
            continue;
        }
        let key = utf16le_text(&k).unwrap_or(k);
        add_count(&mut out, key, v);
    }
    (cl, out)
}
fn word_char_before(text: &str, at: usize) -> Option<char> {
    text[..at].chars().next_back()
}
fn rust_literals(text: &str) -> Counts {
    let mut s = strip_rust_comments(text);
    s = skip_cfg_any_items(&s);
    s = collapse_arch_duals(&s);
    let mut calls = CHANNELS.iter().map(|x| format!("{x}!")).collect::<Vec<_>>();
    calls.extend(RUST_ASM.iter().copied().map(str::to_string));
    let refs = calls.iter().map(String::as_str).collect::<Vec<_>>();
    s = strip_calls(&s, &refs);
    static EXTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"\bextern\s+"[^"]*""#).unwrap());
    s = EXTERN.replace_all(&s, " ").into_owned();
    s = strip_rust_attributes(&s);
    s = strip_calls(&s, &["cfg!", "cfg_attr!"]);
    let mut chars_s = s.clone();
    static CHAR_DOUBLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"b?'"'"#).unwrap());
    chars_s = CHAR_DOUBLE.replace_all(&chars_s, " ").into_owned();
    let arrays_text = s;
    let mut names = CHANNELS
        .iter()
        .filter(|x| x.starts_with("DbgPrint"))
        .copied()
        .collect::<Vec<_>>();
    let mut rust_dbg = names.iter().map(|x| format!("{x}!")).collect::<Vec<_>>();
    let refs = rust_dbg.iter().map(String::as_str).collect::<Vec<_>>();
    chars_s = strip_calls(&chars_s, &refs);
    let mut out = Counts::new();
    for cap in RUST_STRING.captures_iter(&chars_s) {
        let lit = dewiden(cap.get(1).unwrap().as_str());
        add_count(&mut out, lit, 1);
    }
    static PLAIN: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#""((?:[^"\\\n]|\\.)*)""#).unwrap());
    for cap in PLAIN.captures_iter(&chars_s) {
        let m = cap.get(0).unwrap();
        if word_char_before(&chars_s, m.start()).is_some_and(|c| c == 'b' || is_word(c)) {
            continue;
        }
        add_count(&mut out, dewiden(cap.get(1).unwrap().as_str()), 1);
    }
    for arr in u16_array_literals(&arrays_text) {
        add_count(&mut out, arr, 1);
    }
    let out = out
        .into_iter()
        .map(|(k, v)| (norm_escapes(&k), v))
        .collect();
    out
}
fn py_repr(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::new();
    out.push(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c)
            }
            c if c.is_control() => {
                let n = c as u32;
                if n <= 0xff {
                    out.push_str(&format!("\\x{n:02x}"))
                } else if n <= 0xffff {
                    out.push_str(&format!("\\u{n:04x}"))
                } else {
                    out.push_str(&format!("\\U{n:08x}"))
                }
            }
            _ => out.push(c),
        }
    }
    out.push(quote);
    out
}
fn check_pair(
    rs: &SourceFile,
    c: &SourceFile,
    companions: &[&SourceFile],
    shown: &str,
) -> Vec<String> {
    let (cl, rl) = normalize(c_literals(&c.text), rust_literals(&rs.text));
    let mut rl_seen = rl.clone();
    for comp in companions {
        let (_, more) = normalize(Counts::new(), rust_literals(&comp.text));
        for (k, v) in more {
            add_count(&mut rl_seen, k, v);
        }
    }
    let hoisted: BTreeSet<_> = u16_array_literals(&strip_rust_comments(&rs.text))
        .into_iter()
        .collect();
    let mut out = vec![];
    for (lit, n) in &cl {
        let seen = rl_seen.get(lit).copied().unwrap_or(0);
        if *n > seen && !(hoisted.contains(lit) && seen >= 1) {
            out.push(format!(
                "MISSING: {shown}: {} appears {n}x in {} and {seen}x here",
                py_repr(lit),
                c.path.file_name().unwrap_or_default().to_string_lossy()
            ));
        }
    }
    for (lit, n) in &rl {
        let have = cl.get(lit).copied().unwrap_or(0);
        if *n > have && !(hoisted.contains(lit) && have >= 1) {
            out.push(format!(
                "EXTRA: {shown}: {} appears {n}x here and {have}x in {}",
                py_repr(lit),
                c.path.file_name().unwrap_or_default().to_string_lossy()
            ));
        }
    }
    out
}
fn path_match(f: &SourceFile, p: &str) -> bool {
    f.path == Path::new(p)
}
impl Rule for VerifyTranslationLiterals {
    fn name(&self) -> &'static str {
        "verify_translation_literals"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let mut out = vec![];
        for (rdir, cdir) in PAIRS {
            let mut rust: Vec<_> = c
                .files
                .iter()
                .filter(|f| {
                    f.path.starts_with(rdir)
                        && f.path.parent().is_some_and(|p| p == Path::new(rdir))
                        && f.path.extension().is_some_and(|e| e == "rs")
                        && f.path.file_name().is_some_and(|n| n != "mod.rs")
                })
                .collect();
            rust.sort_by_key(|f| f.path.clone());
            for rs in rust {
                let stem = rs.path.file_stem().unwrap().to_string_lossy();
                let cpath = format!("{cdir}/{stem}.c");
                let Some(cf) = c.files.iter().find(|f| path_match(f, &cpath)) else {
                    continue;
                };
                let mut comps: Vec<_> = c
                    .files
                    .iter()
                    .filter(|f| {
                        f.path.parent() == rs.path.parent()
                            && f.path.extension().is_some_and(|e| e == "rs")
                            && f.path.file_stem().is_some_and(|n| {
                                n.to_string_lossy().starts_with(&format!("{stem}_"))
                            })
                    })
                    .filter(|f| {
                        let st = f.path.file_stem().unwrap().to_string_lossy();
                        !c.files
                            .iter()
                            .any(|x| path_match(x, &format!("{cdir}/{st}.c")))
                    })
                    .collect();
                comps.sort_by_key(|f| f.path.clone());
                out.extend(check_pair(rs, cf, &comps, &rs.path.to_string_lossy()));
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn pair(c: &str, rs: &str) -> Vec<String> {
        let c = SourceFile::new(PathBuf::from("mod.c"), c.into());
        let rs = SourceFile::new(PathBuf::from("mod.rs"), rs.into());
        check_pair(&rs, &c, &[], "mod.rs")
    }
    #[test]
    fn permuted_literal_is_caught_and_corrected_pair_is_clean() {
        let c = r#"static void f(char ch) { switch (ch) { case 'A': conv = "cdecl"; break; case 'C': conv = "pascal"; break; case 'G': conv = "stdcall"; break; } }"#;
        let rs = r#"unsafe fn f(ch: u8) { match ch { b'A' => conv = b"cdecl\0".as_ptr(), b'C' => conv = b"stdcall\0".as_ptr(), b'G' => conv = b"stdcall\0".as_ptr(), _ => {} } }"#;
        let f = pair(c, rs);
        assert!(
            f.iter()
                .any(|x| x.contains("MISSING") && x.contains("pascal")),
            "{f:?}"
        );
        assert!(
            f.iter()
                .any(|x| x.contains("EXTRA") && x.contains("stdcall")),
            "{f:?}"
        );
        let good = rs.replace(
            "b\"stdcall\\0\".as_ptr(), b'G'",
            "b\"pascal\\0\".as_ptr(), b'G'",
        );
        assert!(pair(c, &good).is_empty());
    }
    #[test]
    fn count_change_and_debug_format_controls() {
        let f = pair(
            r#"char *a = "x"; char *b = "x";"#,
            r#"let a = b"x\0"; let b2 = b"y\0";"#,
        );
        assert!(
            f.iter().any(|x| x.contains("MISSING") && x.contains("'x'")),
            "{f:?}"
        );
        assert!(pair(r#"ERR("Unknown type %c\n", dt);"#, "ERR!(dt);").is_empty());
        assert!(pair(
            r#"void f(void) { DPRINT1("failed %lu\\n", e); g(L"real"); }"#,
            r#"unsafe fn f() { g(w!("real")); }"#
        )
        .is_empty());
    }
    #[test]
    fn wide_nul_and_adjacent_c_string_spellings_match() {
        assert!(pair(r#"x = L"COMSPEC";"#, r#"x = b"C\0O\0M\0S\0P\0E\0C\0\0\0";"#).is_empty());
        assert!(pair(r#"x = "hi";"#, r#"x = b"hi\x00";"#).is_empty());
        assert!(pair(
            r#"void f(void) { g(L"abc " 
 L"def" 
 L"\n"); }"#,
            r#"unsafe fn f() { g(b"abc def\n\0".as_ptr()); }"#
        )
        .is_empty());
    }
    #[test]
    fn compiler_untaken_and_rust_cfg_any_arms_are_skipped() {
        let c = r#"#if _MSVCR_VER>=140
_snprintf(buf, 26, "%2d");
#else
_snprintf(buf, 26, "%02d");
#endif
"#;
        assert!(pair(c, r#"_snprintf(buf, 26, b"%02d\0");"#).is_empty());
        let rs = r#"match x { #[cfg(any())] // `#ifdef _UCRT`; note the semicolon in this comment
 b'G' => { f(b"&\0"); } _ => {} }"#;
        assert!(pair("int x;", rs).is_empty());
        assert!(pair("void f(void) {}",r#"#[cfg(target_arch = "x86")] extern "C" fn f() { let _ = cfg!(target_arch = "arm"); }"#).is_empty());
        assert!(pair(
            "void f(void) {}",
            "'fail: loop { break 'fail; }\n// let s = \"ghost\";"
        )
        .is_empty());
    }
    #[test]
    fn undecidable_c_condition_keeps_both_arms_and_dewiden_is_narrow_safe() {
        let kept = strip_untaken_arms("#if SOMETHING_UNKNOWN\n\"a\";\n#else\n\"b\";\n#endif\n");
        assert!(kept.contains("\"a\"") && kept.contains("\"b\""));
        assert_eq!(dewiden(r"hello\0"), "hello");
        assert_eq!(dewiden(r"h\0e\0l\0l\0o\0\0"), "hello");
        assert_eq!(dewiden(r"a\0b"), r"a\0b");
    }
    #[test]
    fn every_wide_array_spelling_is_seen() {
        let c = r#"void f(void) { g(L"State"); }"#;
        for rs in [
            "static NAME: [u16; 6] = [b'S' as u16, b't' as u16, b'a' as u16, b't' as u16, b'e' as u16, 0]; unsafe fn f() { g(NAME.as_ptr()); }",
            "const NAME: &[u16] = &[0x53, 0x74, 0x61, 0x74, 0x65, 0x0000]; unsafe fn f() { g(NAME.as_ptr()); }",
            "unsafe fn f() { let n: [u16; 6] = [0x53, 0x74, 0x61, 0x74, 0x65, 0]; g(n.as_ptr()); }",
            "unsafe fn f() { let n = &[0x53u16, 0x74, 0x61, 0x74, 0x65, 0]; g(n.as_ptr()); }",
            "unsafe fn f() { g(['S' as u16, 't' as u16, 'a' as u16, 't' as u16, 'e' as u16, 0].as_ptr()); }",
        ] { assert!(pair(c,rs).is_empty(),"{rs}"); }
        let rs =
            "static NAME: [u16; 8] = [0x53, 0x74, 0x61, 0x74, 0x65, 0, 0, 0]; unsafe fn f() {}";
        assert!(pair(r#"void f(void) { g(L"State"); }"#, rs).is_empty());
    }
    #[test]
    fn narrow_arrays_and_nontext_arrays_do_not_create_literals() {
        assert!(pair(
            r#"void f(void) { g(L"\\"); }"#,
            r#"static SEP: [u16; 2] = [b'\\' as u16, 0]; unsafe fn f() {}"#
        )
        .is_empty());
        assert!(pair(
            "void f(void) {}",
            "static T: [u16; 4] = [1000, 2000, 3000, 4000]; unsafe fn f() {}"
        )
        .is_empty());
        let f=pair(r#"void f(void) { g(L"Char"); }"#,"unsafe fn f() { let n: &[u16] = &[0x43,0x00,0x68,0x00,0x61,0x00,0x72,0x00,0x00,0x00]; g(n.as_ptr()); }");
        assert!(
            f.iter()
                .any(|x| x.contains("MISSING") && x.contains("Char")),
            "{f:?}"
        );
    }
    #[test]
    fn escape_controls_preserve_backslash_tab_and_embedded_nul_semantics() {
        let c = r#"void f(void) { g(L"HARDWARE\\CentralProcessor\\0"); }"#;
        let rs = r#"unsafe fn f() { g(w!("HARDWARE\\CentralProcessor\\0")); }"#;
        assert!(pair(c, rs).is_empty());
        assert!(pair(
            "void f(void) { g(L\"a\\tb\"); }",
            "unsafe fn f() { g(\"a\tb\"); }"
        )
        .is_empty());
        assert!(pair(
            r#"void f(void) { g(L"0.0.0.0\0"); }"#,
            r#"unsafe fn f() { g(w!("0.0.0.0\0")); }"#
        )
        .is_empty());
        assert!(pair(
            "void f(void) {\n#if 0\n g(L\"parked\");\n#endif\n g(L\"live\");\n}",
            r#"unsafe fn f() { g(w!("live")); }"#
        )
        .is_empty());
    }
    #[test]
    fn hoisted_arrays_are_presence_compared_not_counted() {
        let c = r#"void f(void) { g(L"Path"); h(L"Path"); i(L"Path"); }"#;
        let rs="static P: [u16; 5] = [b'P' as u16, b'a' as u16, b't' as u16, b'h' as u16, 0]; unsafe fn f() { g(P.as_ptr()); h(P.as_ptr()); i(P.as_ptr()); }";
        assert!(pair(c, rs).is_empty());
    }
}
