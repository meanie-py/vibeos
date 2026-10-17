//! Detect leaked Ob object references on early exits.
use crate::{Corpus, Rule, SourceFile};
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};
use tree_sitter::{Node, Tree};
pub struct ObRefcount;
impl Rule for ObRefcount {
    fn name(&self) -> &'static str {
        "scan_ob_refcount"
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for f in c.rust_files().filter(|f| !skip(f)) {
            for (ln, k, _) in scan_file(f)? {
                out.push(format!("{}:{ln}: {k}", f.path.display()));
            }
        }
        out.sort();
        Ok(out)
    }
}
const OUT: &[(&str, usize)] = &[
    ("ObReferenceObjectByHandle", 4),
    ("ObReferenceObjectByName", 7),
    ("ObReferenceFileObjectForWrite", 2),
];
const ACQ: &[&str] = &[
    "ObReferenceObject",
    "ObfReferenceObject",
    "ObReferenceObjectByPointer",
    "ObReferenceObjectSafe",
];
const DEREF: &[&str] = &[
    "ObDereferenceObject",
    "ObfDereferenceObject",
    "ObDereferenceObjectEx",
    "ObDereferenceObjectDeferDelete",
    "ObDereferenceObjectDeferDeleteWithTag",
];
const NOISE: &[&str] = &[
    "core",
    "ptr",
    "addr_of_mut",
    "addr_of",
    "mut",
    "raw",
    "self",
    "crate",
];
static IDENT: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"[A-Za-z_][A-Za-z0-9_]*").unwrap());
static AS: LazyLock<Regex> = LazyLock::new(|| crate::regexes::cached(r"\bas\b").unwrap());
fn skip(f: &SourceFile) -> bool {
    let p = f.path.to_string_lossy();
    f.path
        .file_name()
        .is_some_and(|n| n.to_string_lossy().starts_with("bindgen_") || n == "build.rs")
        || p.contains("/.responses/")
        || f.path.components().any(|x| x.as_os_str() == "target")
}
fn walk<'a>(n: Node<'a>, v: &mut Vec<Node<'a>>) {
    v.push(n);
    let mut c = n.walk();
    for x in n.children(&mut c) {
        walk(x, v)
    }
}
fn functions<'a>(t: &'a Tree) -> Vec<Node<'a>> {
    let mut v = Vec::new();
    walk(t.root_node(), &mut v);
    let mut out: Vec<_> = v
        .into_iter()
        .filter(|n| n.kind() == "function_item" && n.child_by_field_name("body").is_some())
        .collect();
    out.sort_by_key(|n| n.start_byte());
    out
}
fn text<'a>(n: Node<'_>, s: &'a str) -> &'a str {
    &s[n.start_byte()..n.end_byte()]
}
fn call<'a>(n: Node<'a>, s: &str) -> Option<(String, Option<Node<'a>>)> {
    if n.kind() != "call_expression" {
        return None;
    }
    let f = n.child_by_field_name("function")?;
    let name = match f.kind() {
        "identifier" => text(f, s),
        "scoped_identifier" => text(f.child_by_field_name("name")?, s),
        _ => return None,
    };
    Some((name.to_owned(), n.child_by_field_name("arguments")))
}
fn args<'a>(n: Option<Node<'a>>) -> Vec<Node<'a>> {
    let Some(a) = n else { return vec![] };
    let mut c = a.walk();
    a.children(&mut c)
        .filter(|x| {
            !matches!(
                x.kind(),
                "(" | ")" | "," | "line_comment" | "block_comment" | "comment"
            )
        })
        .collect()
}
fn identifiers(s: &str) -> HashSet<String> {
    IDENT
        .find_iter(s)
        .map(|m| m.as_str())
        .filter(|x| !NOISE.contains(x))
        .map(str::to_owned)
        .collect()
}
fn arg_ids(a: Option<Node<'_>>, s: &str, idx: usize) -> HashSet<String> {
    let a = args(a);
    if idx >= a.len() {
        return HashSet::new();
    }
    let mut v = text(a[idx], s);
    if let Some(m) = AS.find(v) {
        v = &v[..m.start()]
    }
    identifiers(v)
}
fn bound<'a>(n: Node<'a>, s: &str) -> Option<String> {
    let mut p = n.parent();
    while let Some(x) = p {
        if matches!(
            x.kind(),
            "let_declaration" | "assignment_expression" | "block" | "function_item"
        ) {
            return match x.kind() {
                "let_declaration" => {
                    let pat = x.child_by_field_name("pattern")?;
                    IDENT.find(text(pat, s)).map(|m| m.as_str().to_owned())
                }
                "assignment_expression" => {
                    let l = x.child_by_field_name("left")?;
                    let v = text(l, s).trim();
                    (IDENT.find_iter(v).count() == 1
                        && IDENT
                            .find(v)
                            .is_some_and(|m| m.start() == 0 && m.end() == v.len()))
                    .then(|| v.into())
                }
                _ => None,
            };
        }
        p = x.parent()
    }
    None
}
fn line_suppressed(s: &str) -> bool {
    let low = s.to_ascii_lowercase();
    let Some(at) = low.find("//") else {
        return false;
    };
    let Some(rest) = low[at + 2..].trim_start().strip_prefix("nolint") else {
        return false;
    };
    if let Some(label) = rest.strip_prefix(':') {
        let label = label.trim_start();
        label.starts_with("ob-refcount")
            && label["ob-refcount".len()..]
                .chars()
                .next()
                .map_or(true, |c| !c.is_ascii_alphanumeric() && c != '_')
    } else {
        !rest
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
    }
}
fn enclosing_ids(n: Node<'_>, s: &str, stop: Node<'_>) -> HashSet<String> {
    let mut out = HashSet::new();
    let mut p = n.parent();
    while let Some(x) = p {
        if x == stop {
            break;
        }
        match x.kind() {
            "if_expression" | "while_expression" => {
                if let Some(c) = x.child_by_field_name("condition") {
                    out.extend(identifiers(text(c, s)))
                }
            }
            "match_expression" => {
                if let Some(c) = x.child_by_field_name("value") {
                    out.extend(identifiers(text(c, s)))
                }
            }
            _ => (),
        }
        p = x.parent()
    }
    out
}
fn enclosing_guard<'a>(n: Node<'a>, body: Node<'a>) -> Option<Node<'a>> {
    let mut child = n;
    let mut p = n.parent();
    while let Some(x) = p {
        if matches!(x.kind(), "if_expression" | "while_expression") {
            if let Some(c) = x.child_by_field_name("condition") {
                if c.start_byte() <= child.start_byte() && child.end_byte() <= c.end_byte() {
                    return Some(x);
                }
            }
        }
        if x == body {
            break;
        }
        child = x;
        p = x.parent()
    }
    None
}
fn within(n: Node<'_>, c: Option<Node<'_>>) -> bool {
    c.is_some_and(|x| x.start_byte() <= n.start_byte() && n.end_byte() <= x.end_byte())
}
fn scan_body(body: Node<'_>, s: &str) -> Vec<(usize, String, String)> {
    let lines: Vec<_> = s.split('\n').collect();
    let mut all = Vec::new();
    walk(body, &mut all);
    all.sort_by_key(|x| (x.start_byte(), usize::MAX - x.end_byte()));
    let mut held = HashMap::<String, (Option<String>, Option<Node<'_>>)>::new();
    let mut alias = HashMap::<String, String>::new();
    let mut findings = Vec::new();
    let mut did_deref = false;
    let canon = |v: &str, alias: &HashMap<String, String>| {
        alias.get(v).cloned().unwrap_or_else(|| v.into())
    };
    for n in all {
        let row = n.start_position().row;
        let suppressed = lines.get(row).is_some_and(|x| line_suppressed(x));
        if matches!(n.kind(), "return_expression" | "try_expression") {
            if !held.is_empty() && !suppressed {
                let ret = identifiers(text(n, s));
                let guards = enclosing_ids(n, s, body);
                let mut leaked = Vec::new();
                for (v, (status, guard)) in &held {
                    if ret.iter().any(|x| canon(x, &alias) == *v) {
                        continue;
                    }
                    if status.as_ref().is_some_and(|x| guards.contains(x)) {
                        continue;
                    }
                    if within(n, *guard) {
                        continue;
                    }
                    leaked.push(v.clone())
                }
                if !leaked.is_empty() {
                    leaked.sort();
                    findings.push((row+1,if n.kind()=="return_expression"{"OBREF_LEAK_RETURN"}else{"OBREF_LEAK_TRY"}.into(),format!("early exit while holding Ob reference(s) {} — not dereferenced on this path (object + backing pool can never be freed)",leaked.join(", "))));
                }
            }
            continue;
        }
        if let Some((name, a)) = call(n, s) {
            if let Some((_, idx)) = OUT.iter().find(|(x, _)| *x == name) {
                let found = arg_ids(a, s, *idx);
                let status = bound(n, s);
                let guard = enclosing_guard(n, body);
                if found.len() == 1 {
                    held.insert(found.into_iter().next().unwrap(), (status, guard));
                }
            } else if ACQ.contains(&name.as_str()) {
                let found = arg_ids(a, s, 0);
                let status = if matches!(
                    name.as_str(),
                    "ObReferenceObjectByPointer" | "ObReferenceObjectSafe"
                ) {
                    bound(n, s)
                } else {
                    None
                };
                let guard = if status.is_some() {
                    enclosing_guard(n, body)
                } else {
                    None
                };
                if found.len() == 1 {
                    held.insert(found.into_iter().next().unwrap(), (status, guard));
                }
            } else if DEREF.contains(&name.as_str()) {
                for id in arg_ids(a, s, 0) {
                    let c = canon(&id, &alias);
                    held.remove(&id);
                    held.remove(&c);
                    alias.retain(|_, v| v != &id && v != &c);
                    alias.remove(&id);
                }
                did_deref = true;
            } else if !held.is_empty() {
                for i in 0..args(a).len() {
                    for id in arg_ids(a, s, i) {
                        let c = canon(&id, &alias);
                        if held.contains_key(&c) {
                            held.remove(&c);
                            alias.retain(|_, v| v != &c);
                        }
                    }
                }
            }
        }
        if matches!(n.kind(), "assignment_expression" | "let_declaration") && !held.is_empty() {
            let (l, r) = if n.kind() == "assignment_expression" {
                (
                    n.child_by_field_name("left"),
                    n.child_by_field_name("right"),
                )
            } else {
                (
                    n.child_by_field_name("pattern"),
                    n.child_by_field_name("value"),
                )
            };
            if let (Some(l), Some(r)) = (l, r) {
                let lt = text(l, s)
                    .trim()
                    .strip_prefix("mut ")
                    .unwrap_or(text(l, s).trim())
                    .trim();
                let mut rt = text(r, s).trim();
                if let Some(m) = AS.find(rt) {
                    rt = rt[..m.start()].trim()
                }
                let lm = (IDENT.find_iter(lt).count() == 1
                    && IDENT
                        .find(lt)
                        .is_some_and(|m| m.start() == 0 && m.end() == lt.len()))
                .then_some(lt);
                if IDENT.find_iter(rt).count() == 1
                    && IDENT
                        .find(rt)
                        .is_some_and(|m| m.start() == 0 && m.end() == rt.len())
                    && held.contains_key(&canon(rt, &alias))
                {
                    if let Some(lm) = lm {
                        alias.insert(lm.to_owned(), canon(rt, &alias));
                    } else {
                        let c = canon(rt, &alias);
                        held.remove(&c);
                        alias.retain(|_, v| v != &c);
                    }
                } else if let Some(lm) = lm {
                    let c = canon(lm, &alias);
                    if held.contains_key(&c) {
                        held.remove(&c);
                        alias.retain(|_, v| v != &c);
                    }
                }
            }
        }
    }
    if did_deref {
        findings
    } else {
        Vec::new()
    }
}
fn scan_file(f: &SourceFile) -> Result<Vec<(usize, String, String)>, String> {
    if skip(f) {
        return Ok(vec![]);
    }
    let t = f.rust_tree()?;
    let mut out = Vec::new();
    for fnn in functions(t) {
        out.extend(scan_body(fnn.child_by_field_name("body").unwrap(), &f.text));
    }
    out.sort();
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    fn f(s: &str) -> SourceFile {
        SourceFile::new(PathBuf::from("sdk/lib/rust/test.rs"), s.into())
    }
    #[test]
    fn control_leak_and_balanced() {
        let bad=f("pub unsafe fn leaky(cond:bool){ObReferenceObject(obj);if cond{return;}ObDereferenceObject(obj);}");
        let good = f("pub unsafe fn good(){ObReferenceObject(obj);ObDereferenceObject(obj);}");
        assert_eq!(scan_file(&bad).unwrap()[0].1, "OBREF_LEAK_RETURN");
        assert!(scan_file(&good).unwrap().is_empty());
    }
    #[test]
    fn fixed_argument_inline_comment_is_not_misread() {
        let src="pub unsafe fn good(h:HANDLE){let mut fo=ptr::null_mut();let status=ObReferenceObjectByHandle(h,0,IoFileObjectType as *mut _,1,/* UserMode */ &mut fo as *mut *mut FILE_OBJECT as *mut PVOID,null_mut());if !NT_SUCCESS(status){return;}ObDereferenceObject(fo as *mut _);}";
        assert!(scan_file(&f(src)).unwrap().is_empty());
    }
    #[test]
    fn control_output() {
        let c = Corpus::new(
            PathBuf::from("."),
            vec![SourceFile::new(
                "bad.rs".into(),
                "fn f(){ObReferenceObject(x);if c{return;}ObDereferenceObject(x);}".into(),
            )],
        );
        assert_eq!(
            ObRefcount.run(&c).unwrap(),
            vec!["bad.rs:1: OBREF_LEAK_RETURN"]
        );
    }
}
