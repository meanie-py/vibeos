//! A C `extern` variable translated into a second, module-private DEFINITION. The C has one
//! definition (wow64.c's `Wow64pBTCpuSetContext`) and `extern` declarations elsewhere; the
//! Rust gave wow64_manual.rs and wow64_callback.rs their own `static mut` copies, which
//! nothing ever assigned. NtContinue saw None and returned STATUS_INVALID_PARAMETER, and
//! every i386 guest died the moment its loader finished. Rust links the copies silently:
//! without `#[no_mangle]` they are distinct symbols. Flagged: a static DEFINED without
//! `#[no_mangle]` in one file of a crate whose other file exports the same name.
use crate::{Corpus, Rule};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

pub struct ExternCopy;
impl Rule for ExternCopy {
    fn name(&self) -> &'static str {
        "scan_extern_copy"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let files: Vec<(String, String)> = corpus
            .rust_files()
            .map(|f| (f.resolved_path().display().to_string(), f.text.clone()))
            .collect();
        Ok(scan(&files))
    }
}

fn static_def() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?m)^[ \t]*(?:pub(?:\([^)]*\))?\s+)?static\s+(?:mut\s+)?([A-Za-z_]\w*)\s*:[^;=]*=")
            .unwrap()
    })
}
/// The crate a file belongs to: `.../sdk/lib/rust/<crate>/...`.
fn crate_of(path: &str) -> Option<&str> {
    let i = path.find("sdk/lib/rust/")? + "sdk/lib/rust/".len();
    path[i..].split('/').next()
}
fn exported(text: &str, at: usize) -> bool {
    // The attribute lines directly above the item.
    text[..at]
        .lines()
        .rev()
        .take_while(|l| l.trim_start().starts_with("#[") || l.trim().is_empty())
        .any(|l| l.contains("no_mangle"))
}
fn scan(files: &[(String, String)]) -> Vec<String> {
    // crate -> exported names; crate -> (file, line, name) of private definitions
    let mut exports: BTreeMap<&str, BTreeMap<String, String>> = BTreeMap::new();
    let mut private: Vec<(&str, &str, usize, String)> = Vec::new();
    for (path, text) in files {
        let Some(krate) = crate_of(path) else { continue };
        for c in static_def().captures_iter(text) {
            let m = c.get(0).unwrap();
            let name = c.get(1).unwrap().as_str().to_string();
            let line = text[..m.start()].matches('\n').count() + 1;
            if exported(text, m.start()) {
                exports.entry(krate).or_default().insert(name, path.clone());
            } else {
                private.push((krate, path, line, name));
            }
        }
    }
    let mut out = BTreeSet::new();
    for (krate, path, line, name) in private {
        if let Some(home) = exports.get(krate).and_then(|e| e.get(&name)) {
            if home != path {
                out.insert(format!(
                    "{path}:{line}: `static {name}` is a private copy of the #[no_mangle] one in \
                     {home}: a C extern turned into a second definition? Import it instead."
                ));
            }
        }
    }
    out.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_private_copy_of_an_exported_static_in_the_same_crate() {
        let files = vec![
            ("sdk/lib/rust/w-rust/src/a.rs".to_string(),
             "#[no_mangle]\npub static mut Hook: Option<fn()> = None;\n".to_string()),
            ("sdk/lib/rust/w-rust/src/b.rs".to_string(),
             "pub static mut Hook: Option<fn()> = None;\nextern \"C\" { static mut Other: u32; }\n".to_string()),
            ("sdk/lib/rust/x-rust/src/c.rs".to_string(),
             "static mut Hook: u32 = 0;\n".to_string()), // another crate: not a copy
        ];
        assert_eq!(scan(&files), vec![
            "sdk/lib/rust/w-rust/src/b.rs:1: `static Hook` is a private copy of the #[no_mangle] one in sdk/lib/rust/w-rust/src/a.rs: a C extern turned into a second definition? Import it instead.".to_string()
        ]);
        assert_eq!(ExternCopy.name(), "scan_extern_copy");
    }
}
