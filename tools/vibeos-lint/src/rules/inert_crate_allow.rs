//! A crate-wide `allow` written as an OUTER attribute in a crate root, where it attaches to
//! the `extern crate` / `use` right below and allows nothing.
//!
//! version-rust opened with `#[allow(non_snake_case, non_camel_case_types, ...)]` where every
//! other translated crate has `#![allow(..)]`: the outer form bound to `extern crate
//! winedebug_rust;`, and the crate's 248 C-named items warned as if no allow existed
//! (2026-11-01). Only lints that can never fire on an `extern crate` / `use` item are flagged,
//! so `#[allow(unused_imports)] use ..` stays legitimate.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::LazyLock;
pub struct InertCrateAllow;
impl Rule for InertCrateAllow {
    fn name(&self) -> &'static str {
        "scan_inert_crate_allow"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.rust_files() {
            let root = matches!(
                file.path.file_name().and_then(|n| n.to_str()),
                Some("lib.rs") | Some("main.rs")
            );
            if !root {
                continue;
            }
            for (line, lints) in scan_text(&file.text) {
                out.push(format!(
                    "{}:{line}: outer `#[allow({lints})]` binds to the next `extern crate`/`use` \
                     and allows nothing; write `#![allow(..)]`",
                    file.path.display()
                ));
            }
        }
        Ok(out)
    }
}
static ALLOW: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*#\[allow\(([^)]*)\)\]\s*$").unwrap());
static TARGET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:extern\s+crate\b|use\b)").unwrap()
});
/// Lints that only ever fire on items an `extern crate` / `use` is not.
const ITEM_LINTS: &[&str] = &[
    "non_snake_case",
    "non_camel_case_types",
    "non_upper_case_globals",
    "dead_code",
    "unused_assignments",
    "unused_variables",
    "unused_mut",
];
fn scan_text(text: &str) -> Vec<(usize, String)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if !ALLOW.is_match(lines[i]) {
            i += 1;
            continue;
        }
        // The run of outer attributes this one belongs to, then the item they decorate
        // (blank and `//` lines between do not detach an attribute).
        let start = i;
        let mut j = i;
        while j < lines.len() {
            let t = lines[j].trim();
            if t.is_empty() || t.starts_with("//") || (t.starts_with("#[") && !t.starts_with("#!["))
            {
                j += 1;
            } else {
                break;
            }
        }
        if j < lines.len() && TARGET.is_match(lines[j]) {
            for (k, line) in lines.iter().enumerate().take(j).skip(start) {
                if let Some(a) = ALLOW.captures(line) {
                    let lints = a[1].trim().to_string();
                    if lints.split(',').any(|l| ITEM_LINTS.contains(&l.trim()))
                        && !line.contains("nolint: inert-crate-allow")
                    {
                        out.push((k + 1, lints));
                    }
                }
            }
        }
        i = j.max(i + 1);
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dedicated_fixture() {
        let text = "#![no_std]\n#[allow(non_snake_case, non_camel_case_types)]\n#[allow(dead_code)]\n\n// comment\n#[macro_use]\nextern crate winedebug_rust;\n";
        assert_eq!(
            scan_text(text),
            vec![
                (2, "non_snake_case, non_camel_case_types".into()),
                (3, "dead_code".into())
            ]
        );
    }
    #[test]
    fn scanner_controls() {
        // The inner form, an import-lint allow on a `use`, and an allow on a real item.
        assert!(scan_text("#![allow(non_snake_case)]\nextern crate x;\n").is_empty());
        assert!(scan_text("#[allow(unused_imports)]\nuse a::b;\n").is_empty());
        assert!(scan_text("#[allow(dead_code)]\nfn f() {}\n").is_empty());
        assert!(scan_text("#[allow(dead_code)]\nmod m;\n").is_empty());
    }
}
