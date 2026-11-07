//! Targets made by a cmake function call: `add_d3dx9_target(24)` in d3dx9_24/CMakeLists.txt
//! runs a `function(add_d3dx9_target VERSION)` from an included `.cmake`, so no statement
//! in the module's own CMakeLists names the target and the model refused it ("target made
//! in a loop or function"). Twenty d3dx9 versions, eleven d3dx10, two d3dx11 and thirteen
//! d3dcompiler versions are made this way.
//!
//! [`expand`] gives the model the CMakeLists cmake effectively runs: the call replaced by
//! the function body, with the parameters bound and the body's leading single-value
//! `set()` locals (`_target`, `_srcdir`) substituted. [`template`] turns the rewritten
//! expansion back into a function body: the locals and parameters are put back in place of
//! their values, and every caller of the function must yield the same template, so the
//! function stays one function in the tree. Anything that does not fit is refused.
use super::graph::{args, statements};
use super::{read, Error, Result};
use regex::Regex;
use std::path::{Path, PathBuf};

pub const BEGIN: &str = "# vibeos-fncall begin\n";
pub const END: &str = "# vibeos-fncall end\n";

#[derive(Clone, Debug)]
pub struct Expansion {
    /// The caller's CMakeLists with the call replaced by the marked, substituted body.
    pub text: String,
    pub caller: PathBuf,
    /// The caller's CMakeLists as it is on disk.
    pub caller_text: String,
    pub call_span: (usize, usize),
    pub function_file: PathBuf,
    pub function: String,
    /// The function file's byte span of the body (after `function(...)`, before
    /// `endfunction()`), and of the leading locals within it.
    pub body_span: (usize, usize),
    pub locals_end: usize,
    /// (variable, value) pairs, locals then parameters, as substituted into the body.
    pub bindings: Vec<(String, String)>,
}

struct Function {
    name: String,
    params: Vec<String>,
    body: (usize, usize),
}

fn functions(text: &str) -> Vec<Function> {
    let stmts = statements(text);
    let mut out = Vec::new();
    for (i, (_, end, cmd, raw)) in stmts.iter().enumerate() {
        if cmd != "function" {
            continue;
        }
        let a = args(raw);
        let Some((name, params)) = a.split_first() else { continue };
        let mut depth = 0usize;
        for (start, _, c, _) in &stmts[i + 1..] {
            match c.as_str() {
                "function" => depth += 1,
                "endfunction" if depth == 0 => {
                    out.push(Function {
                        name: name.to_ascii_lowercase(),
                        params: params.to_vec(),
                        body: (*end, *start),
                    });
                    break;
                }
                "endfunction" => depth -= 1,
                _ => {}
            }
        }
    }
    out
}

fn includes(cmake: &Path, text: &str) -> Vec<PathBuf> {
    let dir = cmake.parent().unwrap_or(Path::new("."));
    statements(text)
        .iter()
        .filter(|s| s.2 == "include")
        .filter_map(|s| args(&s.3).into_iter().next())
        .map(|a| a.trim_matches('"').replace("${CMAKE_CURRENT_SOURCE_DIR}", &dir.to_string_lossy()))
        .filter(|a| a.ends_with(".cmake"))
        .map(|a| {
            let p = PathBuf::from(&a);
            normal(&if p.is_absolute() { p } else { dir.join(p) })
        })
        .filter(|p| p.is_file())
        .collect()
}

/// `a/b/../c` as `a/c`: every caller of one function must name the same file.
fn normal(p: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::ParentDir if out.file_name().is_some() => {
                out.pop();
            }
            Component::CurDir => {}
            c => out.push(c),
        }
    }
    out
}

fn defines(text: &str, target: &str) -> bool {
    statements(text).iter().any(|(_, _, c, r)| {
        ["add_library", "add_executable"].contains(&c.as_str())
            && args(r).first().is_some_and(|x| x == target)
    })
}

fn subst(text: &str, var: &str, value: &str) -> String {
    text.replace(&format!("${{{var}}}"), value)
}

/// The expansion of the function call in `cmake` that makes `target`, or `None` when the
/// CMakeLists makes it directly (or not at all: the model's own refusal then applies).
pub fn expand(cmake: &Path, target: &str) -> Result<Option<Expansion>> {
    let text = read(cmake)?;
    if defines(&text, target) {
        return Ok(None);
    }
    let calls = statements(&text);
    for file in includes(cmake, &text) {
        let ftext = read(&file)?;
        for f in functions(&ftext) {
            for (cs, ce, cmd, raw) in calls.iter().filter(|s| s.2 == f.name) {
                let values = args(raw);
                if values.len() != f.params.len() {
                    continue;
                }
                let mut body = ftext[f.body.0..f.body.1].to_owned();
                let mut bindings = Vec::new();
                for (p, v) in f.params.iter().zip(&values) {
                    body = subst(&body, p, v);
                    bindings.push((p.clone(), v.clone()));
                }
                // The leading run of single-value set()s (only blank and comment lines
                // between them) are the locals: substituted, and taken out of the body.
                let mut locals = Vec::new();
                let mut locals_end_sub = 0;
                let mut locals_end_orig = f.body.0;
                let orig_stmts = statements(&ftext[f.body.0..f.body.1]);
                for (k, (s, e, c, r)) in statements(&body).iter().enumerate() {
                    let gap = &body[locals_end_sub..*s];
                    let a = args(r);
                    if c != "set"
                        || a.len() != 2
                        || a[1].starts_with('"')
                        || gap.lines().any(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
                    {
                        break;
                    }
                    locals.push((a[0].clone(), a[1].clone()));
                    locals_end_sub = *e;
                    locals_end_orig = f.body.0 + orig_stmts[k].1;
                }
                let mut rest = body[locals_end_sub..].to_owned();
                for (var, value) in &locals {
                    rest = subst(&rest, var, value);
                }
                if !defines(&rest, target) {
                    continue;
                }
                let mut all = locals.clone();
                all.extend(bindings);
                let expanded = format!("{}{BEGIN}{rest}{END}{}", &text[..*cs], &text[*ce..]);
                let _ = cmd;
                return Ok(Some(Expansion {
                    text: expanded,
                    caller: cmake.to_path_buf(),
                    caller_text: text.clone(),
                    call_span: (*cs, *ce),
                    function_file: file.clone(),
                    function: f.name.clone(),
                    body_span: f.body,
                    locals_end: locals_end_orig,
                    bindings: all,
                }));
            }
        }
    }
    Ok(None)
}

/// The CMakeLists text the model reads for `target`: the expansion when a function call
/// makes it, the file otherwise.
pub fn cmake_text(cmake: &Path, target: &str) -> Result<String> {
    Ok(match expand(cmake, target)? {
        Some(e) => e.text,
        None => read(cmake)?,
    })
}

fn desubst(body: &str, bindings: &[(String, String)]) -> String {
    let mut order: Vec<_> = bindings.iter().collect();
    // Longest value first: `d3dx9_24` must become ${_target} before `24` is ${VERSION}.
    order.sort_by_key(|(_, v)| std::cmp::Reverse(v.len()));
    let mut out = body.to_owned();
    for (var, value) in order {
        if !value.is_empty() {
            out = replace_token(&out, value, &format!("${{{var}}}"));
        }
    }
    out
}

/// `value` replaced only where no letter or digit touches it: VERSION=32 must not turn
/// `ole32` into `ole${VERSION}` (the d3dx9_32 caller caught exactly that).
fn replace_token(text: &str, value: &str, with: &str) -> String {
    let word = |c: Option<char>| c.is_some_and(|c| c.is_ascii_alphanumeric());
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find(value) {
        let before = rest[..i].chars().next_back();
        let after = rest[i + value.len()..].chars().next();
        out.push_str(&rest[..i]);
        out.push_str(if word(before) || word(after) { value } else { with });
        rest = &rest[i + value.len()..];
    }
    out.push_str(rest);
    out
}

fn resubst(body: &str, bindings: &[(String, String)]) -> String {
    let mut out = body.to_owned();
    for (var, value) in bindings {
        out = subst(&out, var, value);
    }
    out
}

/// The function body template for a rewritten expansion: the converted body between the
/// markers with each binding's value put back as its variable. Refused when the rewrite
/// touched the caller outside the call, or when the template does not re-substitute to
/// the converted body exactly.
pub fn template(root: &Path, e: &Expansion, converted: &str) -> Result<String> {
    let err = |why: &str| Error::new(format!("{}: {why}", e.function_file.display()));
    let b = converted.find(BEGIN).ok_or_else(|| err("rewrite dropped the call's begin marker"))?;
    let rel = converted[b..].find(END).ok_or_else(|| err("rewrite dropped the end marker"))?;
    let end = b + rel;
    let (before, after) = (&e.caller_text[..e.call_span.0], &e.caller_text[e.call_span.1..]);
    // Blank lines do not count: the caller is never written back, and the rewrite
    // normalises them (it drops a CMakeLists' leading blank line).
    let squash = |t: &str| -> String {
        t.lines().filter(|l| !l.trim().is_empty()).collect::<Vec<_>>().join("\n")
    };
    let changed = |was: &str, now: &str| -> Result<()> {
        let (was, now) = (squash(was), squash(now));
        if was == now {
            return Ok(());
        }
        let first = was
            .lines()
            .zip(now.lines())
            .find(|(a, b)| a != b)
            .map(|(a, b)| format!("`{a}` became `{b}`"))
            .unwrap_or_else(|| format!("{} lines became {}", was.lines().count(), now.lines().count()));
        Err(err(&format!(
            "the rewrite of {}() changed its caller outside the call: {first}",
            e.function
        )))
    };
    changed(before, &converted[..b])?;
    let mut body = converted[b + BEGIN.len()..end].to_owned();
    let tail = &converted[end + END.len()..];
    if squash(after).is_empty() {
        // The rewrite appends the target's add_cargo_module block at the end of the file;
        // with nothing after the call, that block is the call's, so it joins the body.
        if !tail.trim().is_empty() {
            body.push_str(tail);
        }
    } else {
        changed(after, tail)?;
    }
    // A local naming a directory (`_srcdir` = ${CMAKE_CURRENT_SOURCE_DIR}/../d3dx9_36) comes
    // back from the rewrite as `${REACTOS_SOURCE_DIR}/<that directory>`; spell it the
    // local's way first, so every caller's sources read `${_srcdir}/x.c`.
    let dir = e.caller.parent().unwrap_or(Path::new("."));
    for (_, value) in e.bindings.iter() {
        if let Some(rel) = value.strip_prefix("${CMAKE_CURRENT_SOURCE_DIR}/") {
            let resolved = normal(&dir.join(rel));
            if let Ok(r) = resolved.strip_prefix(root) {
                let alias = format!("${{REACTOS_SOURCE_DIR}}/{}", r.display());
                body = replace_token(&body, &alias, value);
            }
        }
    }
    let body = body.as_str();
    let t = desubst(body, &e.bindings);
    if resubst(&t, &e.bindings) != body {
        return Err(err(&format!(
            "{}()'s converted body does not round-trip through its variables",
            e.function
        )));
    }
    Ok(tidy(&t))
}

/// A list `set()` nothing reads any more is dropped (the rewrite spells the sources out),
/// and every statement starts at the body's indent rather than column 0.
fn tidy(t: &str) -> String {
    let mut out = t.to_owned();
    for (s, e, cmd, raw) in statements(t).into_iter().rev() {
        let a = args(&raw);
        if cmd == "set" && a.len() > 2 && !out.contains(&format!("${{{}}}", a[0])) {
            out.replace_range(s..e, "");
        }
    }
    let out = out
        .lines()
        .map(|l| {
            if l.is_empty() || l.starts_with(char::is_whitespace) {
                l.to_owned()
            } else {
                format!("    {l}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let out = Regex::new(r"\n{3,}").unwrap().replace_all(&out, "\n\n").into_owned();
    format!("{}\n", out.trim_end())
}

/// The function file with its body (after the locals) replaced by `template`.
pub fn apply_template(e: &Expansion, template: &str) -> Result<String> {
    let ftext = read(&e.function_file)?;
    Ok(format!(
        "{}{template}{}",
        &ftext[..e.locals_end],
        &ftext[e.body_span.1..]
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Tmp(PathBuf);
    impl Tmp {
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for Tmp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    fn tree() -> (Tmp, PathBuf) {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static N: AtomicUsize = AtomicUsize::new(0);
        let d = Tmp(std::env::temp_dir().join(format!(
            "fncall-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        )));
        let shared = d.path().join("lib");
        std::fs::create_dir_all(&shared).unwrap();
        std::fs::write(
            shared.join("fam.cmake"),
            "function(add_fam_target VERSION)\n    set(_target fam_${VERSION})\n    set(_srcdir ${CMAKE_CURRENT_SOURCE_DIR}/../lib)\n\n    add_library(${_target} MODULE ${_srcdir}/a.c)\n    target_compile_definitions(${_target} PRIVATE V=${VERSION})\n    set_module_type(${_target} win32dll)\nendfunction()\n",
        )
        .unwrap();
        for v in ["24", "25"] {
            let m = d.path().join(format!("fam_{v}"));
            std::fs::create_dir_all(&m).unwrap();
            std::fs::write(
                m.join("CMakeLists.txt"),
                format!("\ninclude(${{CMAKE_CURRENT_SOURCE_DIR}}/../lib/fam.cmake)\n\nadd_fam_target({v})\n"),
            )
            .unwrap();
        }
        let p = d.path().join("fam_24/CMakeLists.txt");
        (d, p)
    }
    #[test]
    fn a_function_made_target_expands_with_its_bindings() {
        let (_d, cm) = tree();
        let e = expand(&cm, "fam_24").unwrap().unwrap();
        assert!(e.text.contains("add_library(fam_24 MODULE ${CMAKE_CURRENT_SOURCE_DIR}/../lib/a.c)"));
        assert!(e.text.contains("V=24"));
        assert!(!e.text.contains("set(_target"));
        assert!(expand(&cm, "fam_25").unwrap().is_none());
    }
    #[test]
    fn the_converted_body_becomes_one_template_for_every_caller() {
        let (d, cm) = tree();
        let conv = |cm: &Path, t: &str| {
            let e = expand(cm, t).unwrap().unwrap();
            let c = e.text.replace(&format!("add_library({t} MODULE"), &format!("add_cargo_module({t} {t}-cargo {t}.dll)\nadd_library({t}_shims OBJECT"));
            template(Path::new("/"), &e, &c).unwrap()
        };
        let t24 = conv(&cm, "fam_24");
        let t25 = conv(&d.path().join("fam_25/CMakeLists.txt"), "fam_25");
        assert_eq!(t24, t25);
        assert!(t24.contains("add_cargo_module(${_target} ${_target}-cargo ${_target}.dll)"));
        assert!(t24.contains("V=${VERSION}"));
        let e = expand(&cm, "fam_24").unwrap().unwrap();
        let f = apply_template(&e, &t24).unwrap();
        assert!(f.starts_with("function(add_fam_target VERSION)\n    set(_target fam_${VERSION})\n    set(_srcdir"));
        assert!(f.ends_with("endfunction()\n"));
    }
    #[test]
    fn a_value_inside_a_word_stays_literal() {
        assert_eq!(
            replace_token("libole32 V=32 d3dx9_32 32", "32", "${V}"),
            "libole32 V=${V} d3dx9_${V} ${V}"
        );
    }
    #[test]
    fn a_rewrite_outside_the_call_is_refused() {
        let (_d, cm) = tree();
        let e = expand(&cm, "fam_24").unwrap().unwrap();
        let c = format!("# changed\n{}", e.text);
        assert!(template(Path::new("/"), &e, &c).is_err());
    }
}
