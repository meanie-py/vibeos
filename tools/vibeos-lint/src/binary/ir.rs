use crate::binary::{BinaryContext, CheckOutput};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

// ---- scan_captures ----
const STD_CRATES: &[&str] = &["core", "alloc", "compiler_builtins"];
const INTRINSIC_PREFIX: &str = "@llvm.";
const CC_TOKENS: &[&str] = &[
    "x86_fastcallcc",
    "x86_stdcallcc",
    "x86_thiscallcc",
    "x86_vectorcallcc",
    "fastcc",
    "coldcc",
];

fn cap_define_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^define\s.*?@([\w$.]+)\((.*)\)[^)]*\{?\s*$").unwrap())
}
fn cap_call_name_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?:call|invoke)\s+.*?@([\w$.]+)\(").unwrap())
}
fn poison_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\b(?:poison|undef)\b").unwrap())
}
fn cap_name_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"%([\w.$]+)\s*$").unwrap())
}
fn unused_param_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^_[A-Za-z]").unwrap())
}
fn panic_callee_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"panicking|assert_failed").unwrap())
}
fn benign_unreachable_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"NeverShortCircuit|zero_div_fn").unwrap())
}
fn split_params(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut cur = String::new();
    for ch in s.chars() {
        if "([{".contains(ch) {
            depth += 1;
        } else if ")]}".contains(ch) {
            depth -= 1;
        }
        if ch == ',' && depth == 0 {
            let t = cur.trim();
            if !t.is_empty() {
                out.push(t.to_string());
            }
            cur.clear();
        } else {
            cur.push(ch);
        }
    }
    let tail = cur.trim();
    if !tail.is_empty() {
        out.push(tail.to_string());
    }
    out
}
fn exported(name: &str) -> bool {
    !(name.starts_with("_ZN") || name.starts_with("_R") || name.starts_with("anon."))
}

#[derive(Default)]
struct CapturesFindings {
    poison_raw: BTreeSet<(String, String, String, Vec<usize>)>,
    readnone_raw: BTreeSet<(String, String, usize, String)>,
    poison: BTreeSet<String>,
    poison_benign: BTreeSet<String>,
    readnone: BTreeSet<String>,
    unreachable: BTreeSet<String>,
    defines: HashMap<String, Vec<String>>,
    stub_fns: BTreeSet<String>,
}
impl CapturesFindings {
    fn finalize(&mut self) {
        for (krate, caller, callee, idxs) in &self.poison_raw {
            let params = self.defines.get(callee);
            let where_ = idxs
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(",");
            let line = format!("[POISON_ARG] {krate}: @{caller} -> @{callee} arg#{where_}");
            if params.is_some_and(|ps| {
                idxs.iter()
                    .all(|i| *i < ps.len() && unused_param_re().is_match(&ps[*i]))
            }) {
                self.poison_benign
                    .insert(format!("{line} (benign: param unused in source)"));
            } else if STD_CRATES.contains(&krate.as_str()) {
                self.poison_benign
                    .insert(format!("{line} (benign: std-internal idiom)"));
            } else if panic_callee_re().is_match(callee) {
                self.poison_benign
                    .insert(format!("{line} (benign: panic plumbing)"));
            } else {
                self.poison.insert(line);
            }
        }
        for (krate, fun, i, name) in &self.readnone_raw {
            if !self.stub_fns.contains(fun) {
                self.readnone.insert(format!(
                    "[READNONE_OUTPARAM] {krate}: @{fun} param#{i} %{name}"
                ));
            }
        }
    }
}
fn scan_capture_text(text: &str, findings: &mut CapturesFindings, crate_name: &str) {
    let mut current = "?".to_string();
    let mut after_define = false;
    for line in text.lines() {
        let stripped = line.trim();
        if let Some(m) = cap_define_re().captures(line) {
            current = m[1].to_string();
            after_define = true;
            let params = split_params(&m[2]);
            let names: Vec<String> = params
                .iter()
                .map(|p| {
                    cap_name_re()
                        .captures(p)
                        .map(|m| m[1].to_string())
                        .unwrap_or_default()
                })
                .collect();
            findings.defines.insert(current.clone(), names.clone());
            if exported(&current) {
                for (i, p) in params.iter().enumerate() {
                    if p.starts_with("ptr")
                        && p.contains("readnone")
                        && (p.contains("captures(none)") || p.contains("nocapture"))
                        && !unused_param_re().is_match(&names[i])
                    {
                        findings.readnone_raw.insert((
                            crate_name.into(),
                            current.clone(),
                            i,
                            names[i].clone(),
                        ));
                    }
                }
            }
            continue;
        }
        if after_define {
            if stripped.ends_with(':') || stripped.is_empty() {
                continue;
            }
            if stripped == "unreachable" || stripped.starts_with("unreachable") {
                if !STD_CRATES.contains(&crate_name) && !benign_unreachable_re().is_match(&current)
                {
                    findings
                        .unreachable
                        .insert(format!("[UNREACHABLE_BODY] {crate_name}: @{current}"));
                }
            } else if stripped.starts_with("ret ") {
                findings.stub_fns.insert(current.clone());
            }
            after_define = false;
        }
        if (line.contains("call") || line.contains("invoke")) && poison_re().is_match(line) {
            let Some(cm) = cap_call_name_re().captures(line) else {
                continue;
            };
            let callee = cm[1].to_string();
            if format!("@{callee}").starts_with(INTRINSIC_PREFIX) {
                continue;
            }
            // Only parse the call argument group, not a surrounding function type.
            let Some(at) = line.find(&format!("@{callee}")) else {
                continue;
            };
            let open = line[at + callee.len() + 1..]
                .find('(')
                .map(|i| at + callee.len() + 1 + i);
            let Some(open) = open else { continue };
            let end = match_paren(line, open);
            let args = split_params(&line[open + 1..end.saturating_sub(1)]);
            let idxs: Vec<usize> = args
                .iter()
                .enumerate()
                .filter_map(|(i, a)| {
                    if a.starts_with("ptr") && poison_re().is_match(a) {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect();
            if !idxs.is_empty() {
                findings
                    .poison_raw
                    .insert((crate_name.into(), current.clone(), callee, idxs));
            }
        }
    }
}
fn match_paren(s: &str, open: usize) -> usize {
    let mut depth = 0i32;
    for (i, ch) in s.char_indices().skip_while(|(i, _)| *i < open) {
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
            if depth == 0 {
                return i + 1;
            }
        }
    }
    s.len()
}
fn crate_of(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .split('-')
        .next()
        .unwrap_or("")
        .to_string()
}
fn expand_ll(inputs: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    fn walk(p: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
        let mut entries: Vec<PathBuf> = fs::read_dir(p)
            .map_err(|e| format!("{}: {e}", p.display()))?
            .map(|e| e.map(|e| e.path()).map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
        entries.sort();
        for child in entries {
            if child.is_dir() {
                walk(&child, out)?;
            } else if child.extension().is_some_and(|e| e == "ll") {
                out.push(child);
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    for p in inputs {
        if p.is_dir() {
            walk(p, &mut out)?;
        } else if p.extension().is_some_and(|e| e == "ll") {
            out.push(p.clone());
        } else {
            return Err(format!(
                "scan_captures: not a .ll file or directory: {}",
                p.display()
            ));
        }
    }
    Ok(out)
}
pub fn captures(ctx: &BinaryContext, inputs: &[PathBuf]) -> Result<CheckOutput, String> {
    let paths = expand_ll(inputs)?;
    if paths.is_empty() {
        return Ok(CheckOutput {
            stdout: String::new(),
            stderr: "scan_captures: no .ll files found — emit with RUSTFLAGS=--emit=llvm-ir\n"
                .into(),
            returncode: 2,
        });
    }
    let mut findings = CapturesFindings::default();
    let mut scanned = 0;
    for path in &paths {
        let crate_name = crate_of(path);
        if !ctx.crates.is_empty() && !ctx.crates.iter().any(|wanted| wanted == &crate_name) {
            continue;
        }
        let bytes = fs::read(path)
            .map_err(|e| format!("scan_captures: cannot read {}: {e}", path.display()))?;
        let text = String::from_utf8_lossy(&bytes);
        scan_capture_text(&text, &mut findings, &crate_name);
        scanned += 1;
    }
    findings.finalize();
    let mut stdout = String::new();
    for group in [
        &findings.poison,
        &findings.unreachable,
        &findings.readnone,
        &findings.poison_benign,
    ] {
        for line in group {
            stdout.push_str(line);
            stdout.push('\n');
        }
    }
    let stderr=format!("scan_captures: {scanned} CGU files; {} POISON_ARG ({} benign), {} UNREACHABLE_BODY, {} READNONE_OUTPARAM\n",findings.poison.len(),findings.poison_benign.len(),findings.unreachable.len(),findings.readnone.len());
    Ok(CheckOutput {
        stdout,
        stderr,
        returncode: if !findings.poison.is_empty() || !findings.unreachable.is_empty() {
            1
        } else {
            0
        },
    })
}

// ---- scan_ir_mismatch ----
fn attr_noise_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(||Regex::new(r"\b(noundef|nonnull|signext|zeroext|inreg|returned|nofree|noalias|nocapture|readonly|writeonly|writable|dead_on_unwind|dead_on_return|immarg|allocptr|swiftself|swifterror)\b|align \d+|byval\([^)]*\)|byref\([^)]*\)|sret\([^)]*\)|inalloca\([^)]*\)|preallocated\([^)]*\)|dereferenceable(?:_or_null)?\(\d+\)|captures\([^)]*\)|range\([^)]*\)|initializes\([^)]*\)").unwrap())
}
fn keyword_noise_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(||Regex::new(r"\b(tail|musttail|notail|call|invoke|callbr|define|declare|internal|private|external|available_externally|linkonce\w*|weak\w*|appending|common|extern_weak|hidden|protected|default|dso_local|dso_preemptable|unnamed_addr|local_unnamed_addr|dllimport|dllexport|thread_local|noundef|signext|zeroext|fast|nnan|ninf|nsz|arcp|contract|afn|reassoc)\b").unwrap())
}
fn cc_alt_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(&format!(
            r"\b(?:{})\b",
            CC_TOKENS
                .iter()
                .map(|s| regex::escape(s))
                .collect::<Vec<_>>()
                .join("|")
        ))
        .unwrap()
    })
}
fn cc_of(s: &str) -> String {
    static R: OnceLock<Vec<Regex>> = OnceLock::new();
    let regexes = R.get_or_init(|| {
        CC_TOKENS
            .iter()
            .map(|t| Regex::new(&format!(r"\b{}\b", regex::escape(t))).unwrap())
            .collect()
    });
    for (re, t) in regexes.iter().zip(CC_TOKENS) {
        if re.is_match(s) {
            return (*t).into();
        }
    }
    "ccc".into()
}
fn strip_attr_noise(s: &str) -> String {
    attr_noise_re().replace_all(s, "").to_string()
}
fn norm_ty(tok: &str) -> String {
    strip_attr_noise(tok)
        .split_whitespace()
        .next()
        .unwrap_or("?")
        .to_string()
}
fn strip_keyword_noise(s: &str) -> String {
    let cc = cc_alt_re().replace_all(s, "");
    keyword_noise_re().replace_all(&cc, "").to_string()
}
fn split_top(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut cur = String::new();
    for ch in s.chars() {
        if "(<[{".contains(ch) {
            depth += 1;
        } else if ")>]}".contains(ch) {
            depth -= 1;
        }
        if ch == ',' && depth == 0 {
            out.push(cur.trim().to_string());
            cur.clear();
        } else {
            cur.push(ch);
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}
fn match_paren_index(s: &str, open: usize) -> usize {
    let mut depth = 0i32;
    for (i, ch) in s.char_indices().skip_while(|(i, _)| *i < open) {
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
            if depth == 0 {
                return i + 1;
            }
        }
    }
    s.len()
}
fn rmatch_paren(s: &str, close: usize) -> usize {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut i = close.min(bytes.len().saturating_sub(1));
    loop {
        match bytes[i] {
            b')' => depth += 1,
            b'(' => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
            }
            _ => {}
        }
        if i == 0 {
            break;
        }
        i -= 1;
    }
    0
}
fn sym_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r#"@("[^"]+"|[-\w$.]+)\s*\("#).unwrap())
}
fn call_re_mismatch() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\b(call|invoke)\b").unwrap())
}
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct Shape {
    cc: String,
    ret: String,
    args: Vec<String>,
    variadic: bool,
}
#[derive(Clone, Debug)]
struct Authority {
    cc: String,
    ret: String,
    args: Vec<String>,
    variadic: bool,
}
fn parse_def(rest: &str) -> Option<(String, Authority)> {
    let sm = sym_re().captures(rest)?;
    let whole = sm.get(0)?;
    let sym = sm[1].trim_matches('"').to_string();
    let pre = rest[..whole.start()].to_string();
    let cc = cc_of(&pre);
    let head = strip_keyword_noise(&pre);
    let ret = strip_attr_noise(&head)
        .split_whitespace()
        .last()
        .unwrap_or("?")
        .to_string();
    let close = match_paren_index(rest, whole.end() - 1);
    let params = split_top(&rest[whole.end()..close.saturating_sub(1)]);
    let variadic = params.last().is_some_and(|p| p == "...");
    let args = params
        .iter()
        .filter(|p| p.as_str() != "...")
        .map(|p| norm_ty(p))
        .filter(|p| !p.is_empty())
        .collect();
    Some((
        sym,
        Authority {
            cc,
            ret,
            args,
            variadic,
        },
    ))
}
fn parse_call(line: &str) -> Option<(String, Shape)> {
    let cm = call_re_mismatch().find(line)?;
    let rest = &line[cm.end()..];
    let sm = sym_re().captures(rest)?;
    let whole = sm.get(0)?;
    let sym = sm[1].trim_matches('"').to_string();
    let pre = rest[..whole.start()].trim_end().to_string();
    let cc = cc_of(&pre);
    let (ret, args, variadic) = if pre.ends_with(')') {
        let o = rmatch_paren(&pre, pre.len() - 1);
        let params = split_top(&pre[o + 1..pre.len() - 1]);
        let variadic = params.last().is_some_and(|p| p == "...");
        let head = strip_keyword_noise(&pre[..o]);
        let ret = strip_attr_noise(&head)
            .split_whitespace()
            .last()
            .unwrap_or("?")
            .to_string();
        let args = params
            .iter()
            .filter(|p| p.as_str() != "...")
            .map(|p| norm_ty(p))
            .filter(|p| !p.is_empty())
            .collect();
        (ret, args, variadic)
    } else {
        let head = strip_keyword_noise(&pre);
        let ret = strip_attr_noise(&head)
            .split_whitespace()
            .last()
            .unwrap_or("?")
            .to_string();
        let end = match_paren_index(rest, whole.end() - 1);
        let raw = split_top(&rest[whole.end()..end.saturating_sub(1)]);
        let args = raw
            .iter()
            .filter(|p| !p.is_empty() && p.as_str() != "...")
            .map(|p| norm_ty(p))
            .collect();
        (ret, args, false)
    };
    Some((
        sym,
        Shape {
            cc,
            ret,
            args,
            variadic,
        },
    ))
}
fn py_list(items: &[String]) -> String {
    format!(
        "[{}]",
        items
            .iter()
            .map(|s| format!("'{}'", s.replace('\'', "\\'")))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

#[derive(Clone, Debug)]
struct Mismatch {
    class: String,
    sym: String,
    detail: String,
    line: usize,
}
fn scan_mismatch_text(text: &str, include_decl_sig: bool) -> Vec<Mismatch> {
    let mut defs: HashMap<String, Authority> = HashMap::new();
    let mut decls: HashMap<String, Authority> = HashMap::new();
    let mut calls: BTreeMap<String, BTreeSet<Shape>> = BTreeMap::new();
    let mut sample: HashMap<(String, Shape), usize> = HashMap::new();
    for (idx, line) in text.lines().enumerate() {
        let s = line.trim();
        if let Some(rest) = s.strip_prefix("define ") {
            if let Some((sym, a)) = parse_def(rest) {
                defs.insert(sym, a);
            }
            continue;
        }
        if let Some(rest) = s.strip_prefix("declare ") {
            if let Some((sym, a)) = parse_def(rest) {
                decls.insert(sym, a);
            }
            continue;
        }
        if line.contains('@') && call_re_mismatch().is_match(line) {
            if let Some((sym, shape)) = parse_call(line) {
                calls.entry(sym.clone()).or_default().insert(shape.clone());
                sample.entry((sym, shape)).or_insert(idx + 1);
            }
        }
    }
    let mut findings = Vec::new();
    for (sym, shapes) in calls {
        let is_def = defs.contains_key(&sym);
        let Some(auth) = defs.get(&sym).or_else(|| decls.get(&sym)) else {
            continue;
        };
        let kind = if is_def { "def" } else { "decl" };
        for call in shapes {
            let mut row: Option<(String, String)> = None;
            if call.cc != auth.cc {
                row = Some((
                    "CC".into(),
                    format!("authority {} ({kind}), call {}", auth.cc, call.cc),
                ));
            } else if !is_def && !include_decl_sig {
                continue;
            } else if (call.variadic || call.args.len() > auth.args.len())
                && !auth.variadic
                && is_def
            {
                row = Some((
                    "VAR".into(),
                    format!(
                        "variadic/extra-arg call ({}) against fixed-arity def ({})",
                        py_list(&call.args),
                        py_list(&auth.args)
                    ),
                ));
            } else if auth.variadic {
                if call.args.iter().take(auth.args.len()).collect::<Vec<_>>()
                    != auth.args.iter().collect::<Vec<_>>()
                {
                    row = Some((
                        "SIG".into(),
                        format!(
                            "fixed prefix {} ({kind}), call {}",
                            py_list(&auth.args),
                            py_list(&call.args)
                        ),
                    ));
                }
            } else if call.args != auth.args {
                row = Some((
                    "SIG".into(),
                    format!(
                        "args {} ({kind}), call {}",
                        py_list(&auth.args),
                        py_list(&call.args)
                    ),
                ));
            } else if (call.ret == "void" || auth.ret == "void") && call.ret != auth.ret {
                row = Some((
                    "SIG".into(),
                    format!("ret {} ({kind}), call ret {}", auth.ret, call.ret),
                ));
            }
            if let Some((class, detail)) = row {
                let line = sample
                    .get(&(sym.clone(), call.clone()))
                    .copied()
                    .unwrap_or(0);
                findings.push(Mismatch {
                    class,
                    sym: sym.clone(),
                    detail,
                    line,
                });
            }
        }
    }
    findings.sort_by(|a, b| {
        (&a.sym, &a.class, &a.detail, a.line).cmp(&(&b.sym, &b.class, &b.detail, b.line))
    });
    findings
}
pub fn mismatch(
    _ctx: &BinaryContext,
    inputs: &[PathBuf],
    include_all: bool,
) -> Result<CheckOutput, String> {
    if inputs.is_empty() {
        return Ok(CheckOutput {
            stdout: String::new(),
            stderr: "scan_ir_mismatch: expected one or more .ll files\n".into(),
            returncode: 2,
        });
    }
    let mut stdout = String::new();
    let mut total = 0usize;
    for path in inputs {
        if !path.is_file() {
            stdout.push_str(&format!("ERROR: {} not found\n", path.display()));
            return Ok(CheckOutput {
                stdout,
                stderr: String::new(),
                returncode: 2,
            });
        }
        let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let text = String::from_utf8_lossy(&bytes);
        let findings = scan_mismatch_text(&text, include_all);
        if findings.is_empty() {
            stdout.push_str(&format!(
                "== {}: clean ==\n",
                path.file_name().unwrap_or_default().to_string_lossy()
            ));
        } else {
            stdout.push_str(&format!(
                "== {}: {} same-module mismatches ==\n",
                path.file_name().unwrap_or_default().to_string_lossy(),
                findings.len()
            ));
            for f in &findings {
                stdout.push_str(&format!(
                    "  [{}] {}: {}  (e.g. line {})\n",
                    f.class, f.sym, f.detail, f.line
                ));
            }
            total += findings.len();
        }
    }
    if total > 0 {
        stdout.push_str(&format!(
            "\nTOTAL: {total} same-module mismatches — each is a §9.99 landmine\n"
        ));
    }
    Ok(CheckOutput::new(stdout, if total > 0 { 1 } else { 0 }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn captures_classifies_poison_unused_and_stub_readnone() {
        let ir="define void @callee(ptr readnone captures(none) %out, ptr %_ignored) {\nentry:\n ret void\n}\ndefine void @caller() {\n call void @callee(ptr poison, ptr poison)\n ret void\n}\ndefine void @Export(ptr readnone captures(none) %out) {\n call void @helper()\n ret void\n}\ndefine void @Stub(ptr readnone captures(none) %p) {\n ret void\n}\ndefine void @Bad() {\n unreachable\n}\n";
        let mut f = CapturesFindings::default();
        scan_capture_text(ir, &mut f, "demo");
        f.finalize();
        assert!(f.poison.iter().any(|x| x.contains("arg#0,1")));
        assert!(f.poison_benign.is_empty());
        assert_eq!(f.readnone.len(), 1);
        assert!(f.unreachable.iter().any(|x| x.contains("@Bad")));
    }
    #[test]
    fn captures_unused_parameter_benign_and_intrinsic_ignored() {
        let ir="define void @callee(ptr %_out) {\n ret void\n}\ndefine void @caller() {\n call void @callee(ptr poison)\n call void @llvm.memcpy.p0.p0.i64(ptr poison, ptr poison, i64 4, i1 false)\n ret void\n}\n";
        let mut f = CapturesFindings::default();
        scan_capture_text(ir, &mut f, "demo");
        f.finalize();
        assert!(f.poison.is_empty());
        assert_eq!(f.poison_benign.len(), 1);
    }
    #[test]
    fn mismatch_detects_cc_sig_and_variadic_with_sample_lines() {
        let ir="define dso_local i32 @f(ptr %p) {\n ret i32 0\n}\ndefine void @v(i32 %x) {\n ret void\n}\ndefine void @fixed(i32 %x) {\n ret void\n}\ndefine void @ret() {\n ret void\n}\ncall fastcc i32 @f(ptr %x)\ncall i32 @f(i32 4)\ncall void (i32, ...) @fixed(i32 1, i32 2)\ncall void (i32, ...) @v(i32 1, i32 2)\ncall i32 @ret()\n";
        let f = scan_mismatch_text(ir, false);
        assert!(f.iter().any(|x| x.class == "CC" && x.sym == "f"));
        assert!(f.iter().any(|x| x.class == "SIG" && x.sym == "f"));
        assert!(f.iter().any(|x| x.class == "VAR" && x.sym == "fixed"));
        assert!(f.iter().any(|x| x.class == "SIG" && x.sym == "ret"));
    }
    #[test]
    fn declaration_signature_is_default_ignored_but_cc_is_checked() {
        let ir = "declare i32 @decl(i32)\ncall i32 @decl(ptr null)\ncall fastcc i32 @decl(i32 1)\n";
        let f = scan_mismatch_text(ir, false);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].class, "CC");
        assert_eq!(scan_mismatch_text(ir, true).len(), 2);
    }
    #[test]
    fn captures_respects_crates_selector() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let dir = std::env::temp_dir().join(format!(
            "captures-filter-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("wanted-123.cgu.ll"),
            "define void @wanted() {\n ret void\n}\n",
        )
        .unwrap();
        fs::write(
            dir.join("other-456.cgu.ll"),
            "define void @other() {\n ret void\n}\n",
        )
        .unwrap();
        let mut ctx = BinaryContext::new(PathBuf::new(), PathBuf::new(), "amd64".into());
        ctx.crates = vec!["wanted".into()];
        let out = captures(&ctx, &[dir.clone()]).unwrap();
        assert_eq!(out.returncode, 0);
        assert!(out.stderr.starts_with("scan_captures: 1 CGU files;"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn mismatch_parses_quoted_symbols_and_nested_parameter_attributes() {
        let (_, def) =
            parse_def("dso_local void @\"quoted-name\"(ptr sret(%struct.Foo) align 8 %p) #0 {")
                .unwrap();
        assert_eq!(def.ret, "void");
        assert_eq!(def.args, vec!["ptr"]);
        let (name, call) =
            parse_call("call void @\"quoted-name\"(ptr sret(%struct.Foo) align 8 null)").unwrap();
        assert_eq!(name, "quoted-name");
        assert_eq!(call.ret, "void");
        assert_eq!(call.args, vec!["ptr"]);
    }
}
