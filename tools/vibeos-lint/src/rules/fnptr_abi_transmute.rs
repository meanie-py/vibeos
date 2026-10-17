use crate::{Corpus, Rule};
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct FnptrAbiTransmute;
struct Patterns {
    skip: Regex,
    nolint: Regex,
    function: Regex,
    transmute: Regex,
    typed: Regex,
    ntapi: Regex,
}
impl Patterns {
    fn new() -> Self {
        let re = |s| Regex::new(s).expect("valid fnptr-abi-transmute regex");
        Self {
            skip: re(r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/island_[a-z0-9_]+\.rs$)"),
            nolint: re(r"//\s*nolint(?::\s*fnptr-abi-transmute\b|\s*$|:?\s*\()"),
            function: re(
                r#"(?m)^[ \t]*(?:#\[[^\]\n]*\][ \t]*)*(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?(?:unsafe\s+)?(extern\s+"[A-Za-z0-9_-]+"\s+)?fn\s+(\w+)\s*(?:<[^>{;]*>)?\s*\("#,
            ),
            transmute: re(
                r"transmute(?:::<[^>]*>)?\(\s*(\w+)\s+as\s+\*(?:const|mut)\s*(?:\(\)|c_void|core::ffi::c_void|std::ffi::c_void)\s*\)",
            ),
            typed: re(
                r#"transmute::<\s*(?:unsafe\s+)?extern\s+"C"\s+fn\s*\((?:[^()]|\([^()]*\))*\)(?:\s*->\s*[^,>]+)?\s*,\s*([^>]*?)>\s*\(\s*(\w+)\s*\)"#,
            ),
            ntapi: re(r"\b((?:Io|Ke|Ex|Ps|Ob|FsRtl|Cc|Mm|Po)[A-Z]\w*)\s*\($"),
        }
    }
}
fn crate_of(path: &Path, root: &Path) -> PathBuf {
    for parent in path.ancestors().skip(1) {
        if parent.join("Cargo.toml").is_file() || parent == root {
            return parent.to_path_buf();
        }
    }
    path.parent().unwrap_or(path).to_path_buf()
}
fn foreign(text: &str, open: usize) -> bool {
    let mut depth = 1;
    let bytes = text.as_bytes();
    let mut k = open + 1;
    while k < bytes.len() && depth != 0 {
        if bytes[k] == b'(' {
            depth += 1;
        } else if bytes[k] == b')' {
            depth -= 1;
        }
        k += 1;
    }
    while k < bytes.len() && !b"{;".contains(&bytes[k]) {
        k += 1;
    }
    bytes.get(k) == Some(&b';')
}
fn enclosing_call(text: &str, pos: usize, p: &Patterns) -> bool {
    let mut depth = 0;
    let lo = pos.saturating_sub(800);
    for k in (lo..pos).rev() {
        let c = text.as_bytes()[k];
        if c == b';' && depth == 0 {
            return false;
        }
        if c == b')' {
            depth += 1;
        } else if c == b'(' {
            if depth == 0 {
                let mut start = k.saturating_sub(80);
                while !text.is_char_boundary(start) {
                    start += 1;
                }
                if p.ntapi.is_match(&text[start..k + 1]) {
                    return true;
                }
            } else {
                depth -= 1;
            }
        }
    }
    false
}
fn index(texts: &[&str], p: &Patterns) -> HashMap<String, u8> {
    let mut abi = HashMap::new();
    for text in texts {
        for m in p.function.captures_iter(text) {
            let external = m.get(1).is_some() || foreign(text, m.get(0).unwrap().end() - 1);
            *abi.entry(m[2].to_owned()).or_default() |= if external { 2 } else { 1 };
        }
    }
    abi
}
fn scan(text: &str, abi: &HashMap<String, u8>, p: &Patterns) -> Vec<(usize, String)> {
    let lines: Vec<_> = text.lines().collect();
    let mut out = Vec::new();
    for m in p.transmute.captures_iter(text) {
        if abi.get(&m[1]) != Some(&1) {
            continue;
        }
        let line = text[..m.get(0).unwrap().start()]
            .bytes()
            .filter(|b| *b == b'\n')
            .count()
            + 1;
        if !p.nolint.is_match(lines[line - 1]) {
            out.push((line, m[1].to_owned()));
        }
    }
    for m in p.typed.captures_iter(text) {
        if !m[1].contains("extern \"system\"")
            && !enclosing_call(text, m.get(0).unwrap().start(), p)
        {
            continue;
        }
        let line = text[..m.get(0).unwrap().start()]
            .bytes()
            .filter(|b| *b == b'\n')
            .count()
            + 1;
        if !p.nolint.is_match(lines[line - 1]) {
            out.push((line, format!("{} (extern \"C\" re-typed NTAPI)", &m[2])));
        }
    }
    out
}
impl Rule for FnptrAbiTransmute {
    fn name(&self) -> &'static str {
        "scan_fnptr_abi_transmute"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let p = Patterns::new();
        let mut by_crate = HashMap::<PathBuf, Vec<_>>::new();
        let mut dirs = HashMap::new();
        let root = corpus.root.join("sdk/lib/rust");
        for f in corpus.rust_files() {
            if p.skip.is_match(&f.path.to_string_lossy()) {
                continue;
            }
            let parent = f.path.parent().unwrap_or(&f.path);
            let dir = dirs
                .entry(parent.to_path_buf())
                .or_insert_with(|| crate_of(&corpus.root.join(&f.path), &root))
                .clone();
            by_crate.entry(dir).or_default().push(f);
        }
        let mut out = Vec::new();
        for files in by_crate.values() {
            let texts: Vec<_> = files.iter().map(|f| f.text.as_str()).collect();
            let abi = index(&texts, &p);
            for f in files {
                for (line, name) in scan(&f.text, &abi, &p) {
                    out.push((f.path.clone(), line, name));
                }
            }
        }
        out.sort();
        Ok(out
            .into_iter()
            .map(|(path, line, name)| {
                format!("{}:{line}: fnptr-abi-transmute {name}", path.display())
            })
            .collect())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn findings(texts: &[&str]) -> Vec<(usize, String)> {
        let p = Patterns::new();
        scan(texts[texts.len() - 1], &index(texts, &p), &p)
    }
    #[test]
    fn rust_abi_cross_file_and_external_definitions() {
        assert_eq!(
            findings(&[
                "unsafe fn callback(p: T) {}",
                "Some(transmute(callback as *const ()))"
            ]),
            vec![(1, "callback".to_owned())]
        );
        assert!(findings(&[
            "unsafe extern \"system\" fn callback(p:T) {}",
            "Some(transmute(callback as *const ()))"
        ])
        .is_empty());
        assert!(findings(&[
            "fn callback(p:T);",
            "Some(transmute(callback as *const ()))"
        ])
        .is_empty());
        assert!(findings(&[
            "fn callback(p:T) {}\nextern \"C\" fn callback(p:T) {}",
            "Some(transmute(callback as *const ()))"
        ])
        .is_empty());
    }
    #[test]
    fn typed_kernel_and_explicit_system_targets() {
        assert_eq!(findings(&["IoSetCompletionRoutine(irp,Some(transmute::<unsafe extern \"C\" fn(P) -> NTSTATUS,_>(F)),0); "]),vec![(1,"F (extern \"C\" re-typed NTAPI)".to_owned())]);
        assert!(
            findings(&["ordinary(Some(transmute::<unsafe extern \"C\" fn(P),_>(F)));"]).is_empty()
        );
        assert_eq!(
            findings(&["transmute::<extern \"C\" fn(P),extern \"system\" fn(P)>(F)"]).len(),
            1
        );
    }
    #[test]
    fn suppression_and_statement_boundaries() {
        assert!(findings(&[
            "fn f() {}",
            "transmute(f as *const c_void) // nolint: fnptr-abi-transmute reason"
        ])
        .is_empty());
        assert!(
            findings(&["fn f() {}", "transmute(f as *const ()) // nolint (reason)"]).is_empty()
        );
        let p = Patterns::new();
        assert!(!enclosing_call("IoFoo(x); ordinary(transmute", 25, &p));
    }
}
