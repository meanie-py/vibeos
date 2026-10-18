//! A cpp_app constructor hook that a class defines and no constructor of it calls.
//!
//! cpp_app lowers what a C++ constructor does before its body into a generated prologue:
//! `__init_vtables()` for a polymorphic class, `m_psp = T::__psp_init(this_)` for a
//! CPropertyPageImpl one. A constructor that never reached the generator (a whole-module
//! hand bank, or a bank taken before the prologue gained a line) compiles without it and
//! leaves that state zeroed. acppage shipped that way: its banked CLayerUIPropPage
//! constructor predated `__psp_init`, `m_psp.dwSize` stayed 0, CreatePropertySheetPageW
//! returned NULL, and AddPages added no page while reporting S_OK. Only a guest run of
//! the page could see it. The hook exists only to be called from the type's own
//! constructors, so a type whose hook is called nowhere in its crate is that bug.
//! Suppress a verified false positive with `// nolint: uncalled-ctor-hook (reason)` on the
//! hook's definition line.
use crate::{Corpus, Rule};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::OnceLock;

pub struct UncalledCtorHook;
impl Rule for UncalledCtorHook {
    fn name(&self) -> &'static str {
        "scan_uncalled_ctor_hook"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        // Per crate: hook definitions (type, hook) -> location, and the calls seen.
        let mut defs: BTreeMap<String, BTreeMap<(String, &'static str), String>> =
            BTreeMap::new();
        let mut calls: BTreeMap<String, BTreeSet<(String, &'static str)>> = BTreeMap::new();
        for file in corpus.rust_files() {
            let rel = file.path.strip_prefix(&corpus.root).unwrap_or(&file.path);
            let key = crate_key(rel);
            let (d, c) = scan_text(&file.text);
            for (ty, hook, line) in d {
                defs.entry(key.clone())
                    .or_default()
                    .insert((ty, hook), format!("{}:{line}", file.path.display()));
            }
            calls.entry(key).or_default().extend(c);
        }
        let mut findings = Vec::new();
        for (key, d) in &defs {
            let seen = calls.get(key);
            for ((ty, hook), at) in d {
                if seen.is_some_and(|s| s.contains(&(ty.clone(), *hook))) {
                    continue;
                }
                findings.push(format!(
                    "{at}: {ty}::{hook} is defined but no constructor of {ty} calls it; a \
                     constructor that bypassed cpp_app's prologue (a hand bank) leaves that \
                     state zeroed"
                ));
            }
        }
        findings.sort();
        Ok(findings)
    }
}

const HOOKS: &[&str] = &["__psp_init", "__init_vtables"];

/// `sdk/lib/rust/<crate>/...` -> that crate; anything else is one group.
fn crate_key(rel: &Path) -> String {
    let parts: Vec<_> = rel.components().map(|c| c.as_os_str().to_string_lossy()).collect();
    for i in 0..parts.len().saturating_sub(3) {
        if parts[i] == "sdk" && parts[i + 1] == "lib" && parts[i + 2] == "rust" {
            return parts[..=i + 3].join("/");
        }
    }
    String::new()
}

fn impl_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?m)^\s*impl(?:<[^>]*>)?\s+(\w+)").unwrap())
}

fn def_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\bfn\s+(__psp_init|__init_vtables)\s*\(").unwrap())
}

fn call_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // `T::__psp_init(`, `Self::__psp_init(`, `.__init_vtables(`.
    RE.get_or_init(|| Regex::new(r"(?:(\w+)::|\.)(__psp_init|__init_vtables)\s*\(").unwrap())
}

fn hook(name: &str) -> &'static str {
    HOOKS.iter().find(|h| **h == name).copied().unwrap_or("")
}

/// Whether the body after a hook's `fn name(` assigns anything. A non-polymorphic class
/// still gets an `__init_vtables` that only binds `this`; skipping that one changes nothing.
fn stores_anything(rest: &str) -> bool {
    let Some(open) = rest.find('{') else {
        return false;
    };
    let mut depth = 0usize;
    for (i, ch) in rest[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let body = &rest[open + 1..open + i];
                    return body.lines().any(|l| {
                        let l = l.trim();
                        !l.starts_with("let ") && !l.starts_with("//") && !l.is_empty()
                    });
                }
            }
            _ => {}
        }
    }
    false
}

type Defs = Vec<(String, &'static str, usize)>;
type Calls = Vec<(String, &'static str)>;

fn scan_text(text: &str) -> (Defs, Calls) {
    let impls: Vec<(usize, String)> = impl_re()
        .captures_iter(text)
        .map(|c| (c.get(0).unwrap().start(), c[1].to_owned()))
        .collect();
    // The type of the innermost `impl` opened before `at` (impls are not nested in Rust).
    let enclosing = |at: usize| {
        impls
            .iter()
            .rev()
            .find(|(pos, _)| *pos < at)
            .map(|(_, ty)| ty.clone())
    };
    let mut defs = Vec::new();
    for caps in def_re().captures_iter(text) {
        let whole = caps.get(0).unwrap();
        let eol = text[whole.end()..]
            .find('\n')
            .map_or(text.len(), |n| whole.end() + n);
        if text[whole.start()..eol].contains("nolint: uncalled-ctor-hook") {
            continue;
        }
        if !stores_anything(&text[whole.end()..]) {
            continue;
        }
        if let Some(ty) = enclosing(whole.start()) {
            let line = text[..whole.start()].matches('\n').count() + 1;
            defs.push((ty, hook(&caps[1]), line));
        }
    }
    let mut calls = Vec::new();
    for caps in call_re().captures_iter(text) {
        let at = caps.get(0).unwrap().start();
        let ty = match caps.get(1).map(|m| m.as_str()) {
            Some("Self") | None => enclosing(at),
            Some(t) => Some(t.to_owned()),
        };
        if let Some(ty) = ty {
            calls.push((ty, hook(&caps[2])));
        }
    }
    (defs, calls)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAGE: &str = "impl CPage {\n    pub unsafe fn __psp_init(this_: *mut CPage) -> P { p() }\n    unsafe fn __init_vtables(&mut self) {\n        (*this).__base0.lpVtbl = &VT;\n    }\n}\n";

    #[test]
    fn a_hand_ctor_without_the_prologue_is_found() {
        let ctor = "impl CPage {\n    pub unsafe fn __ctor_0(&mut self) {\n        let this_: *mut Self = self;\n        (*this_).__init_vtables();\n    }\n}\n";
        let (d, c) = scan_text(&format!("{PAGE}{ctor}"));
        let called: BTreeSet<_> = c.into_iter().collect();
        let missing: Vec<_> = d.iter().filter(|(t, h, _)| !called.contains(&(t.clone(), *h))).collect();
        assert_eq!(missing, vec![&("CPage".to_owned(), "__psp_init", 2)]);
    }

    #[test]
    fn explicit_and_self_calls_both_count() {
        let ctor = "impl CPage {\n    fn __ctor_0(&mut self) {\n        (*this_).__init_vtables();\n        (*this_).m_psp = CPage::__psp_init(this_);\n    }\n}\nimpl Other {\n    fn f() { Self::__psp_init(p); }\n}\n";
        let (d, c) = scan_text(&format!("{PAGE}{ctor}"));
        let called: BTreeSet<_> = c.into_iter().collect();
        assert!(d.iter().all(|(t, h, _)| called.contains(&(t.clone(), *h))));
        assert!(called.contains(&("Other".to_owned(), "__psp_init")));
    }

    #[test]
    fn nolint_suppresses_and_crates_group_by_dir() {
        let (d, _) = scan_text(
            "impl C {\n    unsafe fn __init_vtables(&mut self) { x = 1; } // nolint: uncalled-ctor-hook (t)\n}\nimpl D {\n    unsafe fn __init_vtables(&mut self) {\n        let this: *mut Self = self;\n        let _ = this;\n    }\n}\n",
        );
        assert!(d.is_empty());
        assert_eq!(
            crate_key(Path::new("sdk/lib/rust/acppage-rust/src/a.rs")),
            "sdk/lib/rust/acppage-rust"
        );
        assert_eq!(crate_key(Path::new("bad.rs")), "");
    }
}
