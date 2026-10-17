//! Calls passing a different local than the matching call in the retained C source.
use crate::{Corpus, Rule};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

pub struct CallArgSwap;
impl Rule for CallArgSwap {
    fn name(&self) -> &'static str {
        "scan_call_arg_swap"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.rust_files() {
            if patterns().skip.is_match(&file.path.to_string_lossy()) {
                continue;
            }
            let Some(cut) = file.text.find("ORIGINAL C SOURCE") else {
                continue;
            };
            let rust = blank_comments(&file.text[..cut]);
            let c = c_source(&file.text[cut..]);
            let c_fns = c_functions(&c);
            for cap in patterns().rust_fn.captures_iter(&rust) {
                let whole = cap.get(0).unwrap();
                let name = cap.get(1).unwrap().as_str();
                let Some(cbody) = c_fns.get(name) else {
                    continue;
                };
                let Some((rbody, _)) = brace_body(&rust, whole.end()) else {
                    continue;
                };
                if patterns().goto.is_match(cbody) {
                    continue;
                }
                let c_locals: HashSet<String> = patterns()
                    .c_local
                    .captures_iter(cbody)
                    .map(|m| m[1].to_owned())
                    .collect();
                let c_calls = by_callee(cbody, true);
                for (key, rust_calls) in by_callee(rbody, false) {
                    let Some(c_calls) = c_calls.get(&key) else {
                        continue;
                    };
                    if c_calls.len() != rust_calls.len() {
                        continue;
                    }
                    for (k, (ra, ca)) in rust_calls.iter().zip(c_calls).enumerate() {
                        for (i, (x, y)) in ra.iter().zip(ca).enumerate() {
                            let xs = rust_arg(x);
                            let ys = c_arg(y);
                            if !is_identifier(&xs)
                                || !is_identifier(&ys)
                                || xs.to_lowercase() == ys.to_lowercase()
                            {
                                continue;
                            }
                            if !c_locals.contains(&xs) || !c_locals.contains(&ys) {
                                continue;
                            }
                            if patterns()
                                .alias
                                .captures_iter(rbody)
                                .any(|a| a[1] == xs && a[2] == ys)
                            {
                                continue;
                            }
                            let line = rust[..whole.start()]
                                .bytes()
                                .filter(|b| *b == b'\n')
                                .count()
                                + 1;
                            out.push(format!("{}:{line}: call-arg-swap {name}: {} call#{k} arg{i}: {xs} where C passes {ys}", file.path.display(), key.0));
                        }
                    }
                }
            }
        }
        out.sort();
        Ok(out)
    }
}
struct Patterns {
    skip: Regex,
    comments: Regex,
    rust_fn: Regex,
    c_fn: Regex,
    c_local: Regex,
    goto: Regex,
    alias: Regex,
    as_cast: Regex,
    rust_prefix: Regex,
    ptr_method: Regex,
    c_cast: Regex,
    call_rust: Regex,
    call_c: Regex,
    block_comments: Regex,
    keyword: Regex,
}
fn patterns() -> &'static Patterns {
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(|| Patterns {
        skip: crate::regexes::cached(
            r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/island_[a-z0-9_]+\.rs$)",
        )
        .unwrap(),
        comments: crate::regexes::cached(r"(?s)//[^\n]*|/\*.*?\*/").unwrap(),
        rust_fn: crate::regexes::cached(r"\bfn\s+(\w+)\s*(?:<[^>]*>)?\(").unwrap(),
        c_fn: crate::regexes::cached(r"(?m)^(\w+)\s*\(").unwrap(),
        c_local: crate::regexes::cached(r"\b(\w+)\s*(?:\[[^\]]*\])?\s*;").unwrap(),
        goto: crate::regexes::cached(r"\bgoto\b").unwrap(),
        alias: crate::regexes::cached(
            r"\blet\s+(?:mut\s+)?([A-Za-z_]\w*)\b[^;=]*=\s*([A-Za-z_]\w*)\b",
        )
        .unwrap(),
        // The Python oracle's lookahead is implemented by strip_as_cast below;
        // look-around is unsupported by Rust's regex crate.
        as_cast: crate::regexes::cached(r"\bas\s+[\w:*\s]+").unwrap(),
        rust_prefix: crate::regexes::cached(
            r"^(&raw\s+(?:mut|const)\s+|&mut\s+|&|core::ptr::addr_of(?:_mut)?!\()",
        )
        .unwrap(),
        ptr_method: crate::regexes::cached(r"\.(?:as_mut_ptr|as_ptr|cast)\(\)").unwrap(),
        c_cast: crate::regexes::cached(r"\((?:[A-Z_][A-Z0-9_]*\s*\*?)\)").unwrap(),
        call_rust: crate::regexes::cached(r"([A-Za-z_]\w*)\s*\(").unwrap(),
        call_c: crate::regexes::cached(r"([A-Za-z_]\w*)\s*\(").unwrap(),
        block_comments: crate::regexes::cached(r"(?s)/\*.*?\*/").unwrap(),
        keyword: crate::regexes::cached(r"^(?:if|while|for|match|return|switch|sizeof|fn|loop)$")
            .unwrap(),
    })
}
fn is_identifier(text: &str) -> bool {
    static ID: OnceLock<Regex> = OnceLock::new();
    ID.get_or_init(|| crate::regexes::cached(r"^[A-Za-z_]\w*$").unwrap())
        .is_match(text)
}
fn blank_comments(text: &str) -> String {
    patterns()
        .comments
        .replace_all(text, |m: &regex::Captures<'_>| {
            m[0].chars()
                .map(|c| if c == '\n' { '\n' } else { ' ' })
                .collect::<String>()
        })
        .into_owned()
}
fn c_source(text: &str) -> String {
    let stripped = text
        .lines()
        .map(|line| {
            let t = line.trim_start();
            if let Some(s) = t.strip_prefix("//") {
                s.strip_prefix(' ').unwrap_or(s)
            } else {
                t
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    patterns()
        .block_comments
        .replace_all(&stripped, "")
        .into_owned()
}
fn brace_body(text: &str, start: usize) -> Option<(&str, usize)> {
    let open = text[start..].find('{')? + start;
    let mut depth = 0usize;
    for (i, b) in text.bytes().enumerate().skip(open) {
        if b == b'{' {
            depth += 1;
        } else if b == b'}' {
            depth -= 1;
            if depth == 0 {
                return Some((&text[open..i], open));
            }
        }
    }
    None
}
fn c_functions(text: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for cap in patterns().c_fn.captures_iter(text) {
        let m = cap.get(0).unwrap();
        let name = cap[1].to_owned();
        let Some(open) = text[m.end()..].find('{').map(|i| i + m.end()) else {
            continue;
        };
        if text[m.end()..open].contains(';') {
            continue;
        }
        if let Some((body, _)) = brace_body(text, m.end()) {
            out.entry(name).or_insert_with(|| body.to_owned());
        }
    }
    out
}
fn calls(text: &str, c: bool) -> Vec<(String, Vec<String>)> {
    let re = if c {
        &patterns().call_c
    } else {
        &patterns().call_rust
    };
    let mut out = Vec::new();
    for cap in re.captures_iter(text) {
        let name = cap[1].to_owned();
        if patterns().keyword.is_match(&name) {
            continue;
        }
        let m = cap.get(0).unwrap();
        if !c && text[m.end()..].trim_start().starts_with('!') {
            continue;
        }
        if let Some(args) = split_args(text, m.end() - 1) {
            out.push((name, args));
        }
    }
    out
}
fn split_args(text: &str, open: usize) -> Option<Vec<String>> {
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    let mut last = open + 1;
    let mut out = Vec::new();
    for i in open..bytes.len() {
        match bytes[i] {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                depth -= 1;
                if depth == 0 {
                    let tail = text[last..i].trim();
                    out.push(tail.to_owned());
                    if out.last().is_some_and(String::is_empty) {
                        out.pop();
                    }
                    return Some(out);
                }
            }
            b',' if depth == 1 => {
                out.push(text[last..i].trim().to_owned());
                last = i + 1;
            }
            _ => {}
        }
    }
    None
}
fn by_callee(body: &str, c: bool) -> HashMap<(String, usize), Vec<Vec<String>>> {
    let mut out = HashMap::new();
    for (name, args) in calls(body, c) {
        out.entry((name, args.len()))
            .or_insert_with(Vec::new)
            .push(args);
    }
    out
}
fn rust_arg(arg: &str) -> String {
    let a = strip_as_cast(arg.trim());
    let a = patterns().rust_prefix.replace(&a, "");
    let a = patterns().ptr_method.replace_all(&a, "");
    a.trim_matches(|c| c == '(' || c == ')' || c == ' ')
        .to_owned()
}
fn strip_as_cast(arg: &str) -> String {
    let mut out = String::with_capacity(arg.len());
    let mut end = 0;
    for m in patterns().as_cast.find_iter(arg) {
        let guarded =
            arg[m.end()..].trim_start().is_empty() || arg[m.end()..].trim_start().starts_with(')');
        if guarded {
            out.push_str(&arg[end..m.start()]);
            end = m.end();
        }
    }
    out.push_str(&arg[end..]);
    out
}
fn c_arg(arg: &str) -> String {
    let a = patterns().c_cast.replace_all(arg.trim(), "");
    a.trim_start_matches('&')
        .trim_matches(|c| c == '(' || c == ')' || c == ' ')
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceFile;
    use std::path::PathBuf;
    const BAD: &str = r##########"pub unsafe extern "system" fn InitializeProfiles() -> BOOL {
    let mut szProfilesPath: [WCHAR; 260] = [0; 260];
    let mut szBuffer: [WCHAR; 260] = [0; 260];
    LoadStringW(hInstance, IDS_PROFILEPATH, szProfilesPath.as_mut_ptr(), 260);
    ExpandEnvironmentStringsW(szBuffer.as_mut_ptr(), szProfilesPath.as_mut_ptr(), 260);
    1
}
// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
// BOOL
// InitializeProfiles(VOID)
// {
//     WCHAR szProfilesPath[MAX_PATH];
//     WCHAR szBuffer[MAX_PATH];
//     LoadStringW(hInstance, IDS_PROFILEPATH, szBuffer, ARRAYSIZE(szBuffer));
//     ExpandEnvironmentStringsW(szBuffer, szProfilesPath, ARRAYSIZE(szProfilesPath));
//     return TRUE;
// }
"##########;
    const GOOD: &str = r##########"pub unsafe extern "system" fn InitializeProfiles() -> BOOL {
    let mut szProfilesPath: [WCHAR; 260] = [0; 260];
    let mut szBuffer: [WCHAR; 260] = [0; 260];
    LoadStringW(hInstance, IDS_PROFILEPATH, szBuffer.as_mut_ptr(), 260);
    ExpandEnvironmentStringsW(szBuffer.as_mut_ptr(), szProfilesPath.as_mut_ptr(), 260);
    1
}
pub unsafe fn ReadKey(InputBuffer: PVOID) {
    let keyParameters: PVOID = InputBuffer;
    ReadDvd(keyParameters);
}
// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
// BOOL
// InitializeProfiles(VOID)
// {
//     WCHAR szProfilesPath[MAX_PATH];
//     WCHAR szBuffer[MAX_PATH];
//     LoadStringW(hInstance, IDS_PROFILEPATH, szBuffer, ARRAYSIZE(szBuffer));
//     ExpandEnvironmentStringsW(szBuffer, szProfilesPath, ARRAYSIZE(szProfilesPath));
//     return TRUE;
// }
// VOID
// ReadKey(PVOID InputBuffer)
// {
//     PVOID keyParameters;
//     PVOID InputBuffer;
//     ReadDvd(InputBuffer);
// }
"##########;
    #[test]
    fn scanner_control_bad_and_good() {
        // The Python fixture uses a triple-quoted `\\` opener, which strips its first newline.
        let bad = Corpus::new(
            PathBuf::from("/fixture"),
            vec![SourceFile::new("bad.rs".into(), BAD.trim_start().into())],
        );
        assert_eq!(CallArgSwap.run(&bad).unwrap(), vec!["bad.rs:1: call-arg-swap InitializeProfiles: LoadStringW call#0 arg2: szProfilesPath where C passes szBuffer"]);
        let good = Corpus::new(
            PathBuf::from("/fixture"),
            vec![SourceFile::new("good.rs".into(), GOOD.trim_start().into())],
        );
        assert!(CallArgSwap.run(&good).unwrap().is_empty());
    }
    #[test]
    fn alias_and_goto_c_functions_are_exempt() {
        let text = "fn f() { let alias: PVOID = Original; ReadDvd(alias); }\n// ORIGINAL C SOURCE\n// VOID f() { PVOID Original; PVOID other; ReadDvd(Original); }\n";
        let corpus = Corpus::new(
            PathBuf::from("/fixture"),
            vec![SourceFile::new("x.rs".into(), text.into())],
        );
        assert!(CallArgSwap.run(&corpus).unwrap().is_empty());
        let text = "fn f() { ReadDvd(other); }\n// ORIGINAL C SOURCE\n// VOID f() { PVOID Original; PVOID other; goto done; ReadDvd(Original); }\n";
        let corpus = Corpus::new(
            PathBuf::from("/fixture"),
            vec![SourceFile::new("x.rs".into(), text.into())],
        );
        assert!(CallArgSwap.run(&corpus).unwrap().is_empty());
    }
    #[test]
    fn cast_suffix_guard_only_removes_end_or_closing_paren_casts() {
        // Python's non-greedy cast substitution leaves the pre-cast space and close paren;
        // the following `.strip("() ")` in _rust_arg removes both at the edges.
        assert_eq!(strip_as_cast("value as usize)"), "value )");
        assert_eq!(rust_arg("value as usize)"), "value");
        assert_eq!(
            strip_as_cast("value as usize + other"),
            "value as usize + other"
        );
        assert_eq!(strip_as_cast("value as *mut T"), "value ");
    }
}
