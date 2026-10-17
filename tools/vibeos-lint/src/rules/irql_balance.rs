//! Detect early exits with manually raised IRQL/spin locks still held.
use crate::{Corpus, Rule, SourceFile};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use tree_sitter::{Node, Tree};

pub struct IrqlBalance;
impl Rule for IrqlBalance {
    fn name(&self) -> &'static str {
        "scan_irql_balance"
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let files: Vec<_> = c.rust_files().filter(|f| !skip(f)).collect();
        let (summaries, calls) = build_summaries(&files)?;
        let mut out = Vec::new();
        for (ix, f) in files.into_iter().enumerate() {
            if !mentions_builtin(&calls[ix])
                && !calls[ix].iter().any(|name| summaries.ret.contains(name))
            {
                continue;
            }
            for (ln, k, _) in scan_file(f, &summaries)? {
                out.push(format!("{}:{ln}: {k}", f.path.display()));
            }
        }
        out.sort();
        Ok(out)
    }
}
const OUT: &[&str] = &[
    "KeRaiseIrql",
    "KeAcquireSpinLock",
    "KeAcquireInStackQueuedSpinLock",
];
const RAISE: &[&str] = &[
    "KfRaiseIrql",
    "KeRaiseIrqlToDpcLevel",
    "KeRaiseIrqlToSynchLevel",
    "KeAcquireSpinLockRaiseToSynch",
];
const LOWER: &[&str] = &["KeLowerIrql", "KfLowerIrql"];
const RELEASE: &[&str] = &["KeReleaseSpinLock"];
const SEEDS: &[&str] = &[
    "KeWaitForSingleObject",
    "KeWaitForMultipleObjects",
    "KeDelayExecutionThread",
    "KeWaitForGate",
    "wait_for",
    "wait_for_armed",
];
const CPS: &[&str] = &["wait_for", "wait_for_armed"];
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
static AS_CAST: LazyLock<Regex> = LazyLock::new(|| crate::regexes::cached(r"\bas\b").unwrap());
static CONST_LEVEL: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"\b[A-Z][A-Z0-9_]*_LEVEL\b").unwrap());
static CTX: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(
        r"\bKeGetPcr\b|\bKeGetCurrentPrcb\b|\bKeGetPcrArm64\b|\bPcpu\s*<|\bget_pcr\b",
    )
    .unwrap()
});
static CALL_TOKEN: LazyLock<regex::bytes::Regex> =
    LazyLock::new(|| regex::bytes::Regex::new(r"(?-u)\b([A-Za-z_]\w*)\s*\(").unwrap());
fn skip(f: &SourceFile) -> bool {
    let p = f.path.to_string_lossy();
    f.path
        .file_name()
        .is_some_and(|n| n.to_string_lossy().starts_with("bindgen_") || n == "build.rs")
        || p.contains("/.responses/")
        || f.path.components().any(|x| x.as_os_str() == "target")
}
fn walk<'a>(n: Node<'a>, v: &mut Vec<Node<'a>>) {
    v.extend(crate::syntax::preorder(n));
}
fn functions<'a>(t: &'a Tree) -> Vec<Node<'a>> {
    crate::syntax::preorder(t.root_node())
        .filter(|n| n.kind() == "function_item" && n.child_by_field_name("body").is_some())
        .collect()
}
fn txt<'a>(n: Node<'_>, s: &'a str) -> &'a str {
    &s[n.start_byte()..n.end_byte()]
}
fn call<'a>(n: Node<'a>, s: &str) -> Option<(String, Option<Node<'a>>)> {
    if n.kind() != "call_expression" {
        return None;
    }
    let f = n.child_by_field_name("function")?;
    let name = match f.kind() {
        "identifier" => txt(f, s),
        "scoped_identifier" => txt(f.child_by_field_name("name")?, s),
        _ => return None,
    };
    Some((name.to_owned(), n.child_by_field_name("arguments")))
}
fn real_args<'a>(a: Option<Node<'a>>) -> Vec<Node<'a>> {
    let Some(a) = a else { return vec![] };
    let mut c = a.walk();
    a.children(&mut c)
        .filter(|n| {
            !matches!(
                n.kind(),
                "(" | ")" | "," | "line_comment" | "block_comment" | "comment"
            )
        })
        .collect()
}
fn ids(s: &str) -> HashSet<String> {
    IDENT
        .find_iter(s)
        .map(|m| m.as_str())
        .filter(|x| !NOISE.contains(x))
        .map(str::to_owned)
        .collect()
}
fn all_ids(s: &str) -> HashSet<String> {
    IDENT.find_iter(s).map(|m| m.as_str().to_owned()).collect()
}
fn call_tokens(s: &str) -> HashSet<String> {
    CALL_TOKEN
        .captures_iter(s.as_bytes())
        .filter_map(|m| {
            m.get(1)
                .map(|x| std::str::from_utf8(x.as_bytes()).unwrap().to_owned())
        })
        .collect()
}
fn mentions_builtin(names: &HashSet<String>) -> bool {
    OUT.iter()
        .chain(RAISE)
        .chain(LOWER)
        .chain(RELEASE)
        .any(|n| names.contains(*n))
}
fn arg_ids_from(a: &[Node<'_>], s: &str, idx: usize) -> HashSet<String> {
    if idx >= a.len() {
        return HashSet::new();
    }
    let mut v = txt(a[idx], s);
    if let Some(i) = AS_CAST.find(v) {
        v = &v[..i.start()]
    }
    ids(v)
}
fn bound_var<'a>(n: Node<'a>, s: &'a str) -> Option<String> {
    let mut p = n.parent();
    while let Some(x) = p {
        if matches!(
            x.kind(),
            "let_declaration" | "assignment_expression" | "block" | "function_item"
        ) {
            return match x.kind() {
                "let_declaration" => {
                    let pat = x.child_by_field_name("pattern")?;
                    IDENT.find(txt(pat, s)).map(|m| m.as_str().to_owned())
                }
                "assignment_expression" => {
                    let l = x.child_by_field_name("left")?;
                    let v = txt(l, s).trim();
                    IDENT
                        .is_match(v)
                        .then(|| v.to_owned())
                        .filter(|v| IDENT.find_iter(v).count() == 1)
                }
                _ => None,
            };
        }
        p = x.parent()
    }
    None
}
fn nolint(line: &str) -> bool {
    let low = line.to_ascii_lowercase();
    let Some(at) = low.find("//") else {
        return false;
    };
    let tail = low[at + 2..].trim_start();
    let Some(rest) = tail.strip_prefix("nolint") else {
        return false;
    };
    if let Some(rest) = rest.strip_prefix(':') {
        let label = rest.trim_start();
        label.starts_with("irql-balance")
            && label["irql-balance".len()..]
                .chars()
                .next()
                .map_or(true, |c| !c.is_ascii_alphanumeric() && c != '_')
    } else if rest
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        false
    } else {
        true
    }
}
#[derive(Default, Clone)]
struct Summary {
    calls: HashSet<String>,
    ret: bool,
    lowers: HashSet<usize>,
}
#[derive(Default, Clone)]
struct Sums {
    sleep: HashSet<String>,
    ret: HashSet<String>,
    lower: HashMap<String, HashSet<usize>>,
}
fn summarize(file: &SourceFile, t: &Tree) -> HashMap<String, Summary> {
    let s = &file.text;
    let mut out = HashMap::new();
    for f in functions(t) {
        let name = f
            .child_by_field_name("name")
            .map(|x| txt(x, s).to_owned())
            .unwrap_or_default();
        let body = f.child_by_field_name("body").unwrap();
        let params = f.child_by_field_name("parameters");
        let mut nodes = Vec::new();
        walk(body, &mut nodes);
        let mut sm = Summary::default();
        // Python's call graph edges are textual call tokens within each function
        // body (including macro/textual calls), rather than only call_expression
        // nodes. Keep that distinction for the two bounded propagation rings.
        sm.calls = call_tokens(txt(body, s));
        let mut held = HashSet::new();
        let mut lowered = HashSet::new();
        let mut returns = HashSet::new();
        for n in nodes {
            if n.kind() == "return_expression" {
                returns.extend(all_ids(txt(n, s)));
            }
            if let Some((cn, a)) = call(n, s) {
                let real = real_args(a);
                if OUT.contains(&cn.as_str()) {
                    let acquire = if cn == "KeRaiseIrql" && !real.is_empty() {
                        CONST_LEVEL.is_match(txt(real[0], s))
                    } else {
                        true
                    };
                    if acquire && !real.is_empty() {
                        held.extend(arg_ids_from(&real, s, real.len() - 1));
                    }
                } else if RAISE.contains(&cn.as_str()) {
                    if let Some(v) = bound_var(n, s) {
                        held.insert(v);
                    }
                } else if LOWER.contains(&cn.as_str()) {
                    let x = arg_ids_from(&real, s, 0);
                    for i in &x {
                        held.remove(i);
                    }
                    lowered.extend(x);
                } else if RELEASE.contains(&cn.as_str()) {
                    let x = arg_ids_from(&real, s, 1);
                    for i in &x {
                        held.remove(i);
                    }
                    lowered.extend(x);
                }
            }
        }
        let mut body_cursor = body.walk();
        let ch = body.named_children(&mut body_cursor).collect::<Vec<_>>();
        if let Some(last) = ch.last() {
            if !matches!(last.kind(), "expression_statement" | "let_declaration") {
                returns.extend(all_ids(txt(*last, s)));
            }
        }
        sm.ret = !held.is_disjoint(&returns);
        if let Some(ps) = params {
            let mut params_cursor = ps.walk();
            let psn = ps.named_children(&mut params_cursor).collect::<Vec<_>>();
            for (i, p) in psn
                .iter()
                .filter(|p| matches!(p.kind(), "parameter" | "self_parameter"))
                .enumerate()
            {
                if let Some(pat) = p.child_by_field_name("pattern") {
                    if let Some(id) = IDENT.find(txt(pat, s)) {
                        if lowered.contains(id.as_str()) {
                            sm.lowers.insert(i);
                        }
                    }
                }
            }
        }
        out.entry(name)
            .and_modify(|old: &mut Summary| {
                old.calls.extend(sm.calls.iter().cloned());
                old.ret |= sm.ret;
                old.lowers.extend(sm.lowers.iter().copied());
            })
            .or_insert(sm);
    }
    out
}
fn build_summaries(files: &[&SourceFile]) -> Result<(Sums, Vec<HashSet<String>>), String> {
    let mut sums = Sums::default();
    let calls: Vec<_> = files.iter().map(|f| call_tokens(&f.text)).collect();
    // Wrapper facts are direct-only, so only syntax-parse files with a textual
    // raise/lower builtin token. Cache each parsed file for the sleep rings and
    // eventual findings pass.
    let mut parsed = HashMap::<usize, HashMap<String, Summary>>::new();
    for (ix, f) in files.iter().enumerate() {
        if !mentions_builtin(&calls[ix]) {
            continue;
        }
        let tree = f.rust_tree()?;
        let facts = summarize(f, &tree);
        for (name, sm) in &facts {
            if sm.ret {
                sums.ret.insert(name.clone());
            }
            if !sm.lowers.is_empty() {
                sums.lower
                    .entry(name.clone())
                    .or_default()
                    .extend(sm.lowers.iter().copied());
            }
        }
        parsed.insert(ix, facts);
    }
    for n in RAISE {
        sums.ret.remove(*n);
    }
    for n in LOWER.iter().chain(RELEASE) {
        sums.lower.remove(*n);
    }
    sums.sleep = SEEDS.iter().map(|x| x.to_string()).collect();
    let mut frontier = sums.sleep.clone();
    for _ in 0..2 {
        if frontier.is_empty() {
            break;
        }
        let previous = frontier.clone();
        frontier.clear();
        for (ix, f) in files.iter().enumerate() {
            if !calls[ix].iter().any(|x| previous.contains(x)) {
                continue;
            }
            if !parsed.contains_key(&ix) {
                let tree = f.rust_tree()?;
                parsed.insert(ix, summarize(f, &tree));
            }
            for (name, sm) in parsed.get(&ix).unwrap() {
                if !sums.sleep.contains(name)
                    && name != "KeSynchronizeExecution"
                    && sm.calls.iter().any(|x| previous.contains(x))
                {
                    frontier.insert(name.clone());
                }
            }
        }
        sums.sleep.extend(frontier.iter().cloned());
    }
    Ok((sums, calls))
}
fn scan_body(body: Node<'_>, s: &str, lines: &[&str], sums: &Sums) -> Vec<(usize, String, String)> {
    let mut findings = Vec::new();
    let mut held = HashMap::<String, bool>::new();
    let mut lowered = false;
    let mut lets: Option<Vec<(String, String, usize, usize)>> = None;
    let mut nodes = Vec::new();
    walk(body, &mut nodes);
    // AST preorder has the same ordering as the prior explicit sort.
    for n in nodes.iter().copied() {
        let line = n.start_position().row;
        let sup = lines.get(line).is_some_and(|x| nolint(x));
        if matches!(n.kind(), "return_expression" | "try_expression") {
            if !held.is_empty() && !sup {
                let ret = all_ids(txt(n, s));
                let leak: Vec<_> = held.keys().filter(|x| !ret.contains(*x)).cloned().collect();
                if !leak.is_empty() {
                    let mut leak = leak;
                    leak.sort();
                    let kind = if n.kind() == "return_expression" {
                        "IRQL_LEAK_RETURN"
                    } else {
                        "IRQL_LEAK_TRY"
                    };
                    let phrase = if n.kind() == "return_expression" {
                        "return"
                    } else {
                        "`?`"
                    };
                    findings.push((line+1,kind.into(),format!("early {phrase} while holding {} — raise/acquire not lowered on this path (use IrqlGuard/SpinLockGuard)",leak.join(", "))));
                }
            }
            continue;
        }
        if let Some((name, args)) = call(n, s) {
            let real = real_args(args);
            if OUT.contains(&name.as_str()) {
                let acquire = if name == "KeRaiseIrql" && !real.is_empty() {
                    CONST_LEVEL.is_match(txt(real[0], s))
                } else {
                    true
                };
                if acquire && !real.is_empty() {
                    let level = if real.is_empty() { "" } else { txt(real[0], s) };
                    let illegal = name != "KeRaiseIrql" || !level.contains("APC_LEVEL");
                    for v in arg_ids_from(&real, s, real.len() - 1) {
                        held.insert(v, illegal);
                    }
                }
            } else if RAISE.contains(&name.as_str()) || sums.ret.contains(&name) {
                if let Some(v) = bound_var(n, s) {
                    held.insert(v, true);
                }
            } else if LOWER.contains(&name.as_str()) {
                for v in arg_ids_from(&real, s, 0) {
                    held.remove(&v);
                }
                lowered = true;
            } else if RELEASE.contains(&name.as_str()) {
                for v in arg_ids_from(&real, s, 1) {
                    held.remove(&v);
                }
                lowered = true;
            } else if let Some(ix) = sums.lower.get(&name) {
                for i in ix {
                    for v in arg_ids_from(&real, s, *i) {
                        held.remove(&v);
                    }
                }
                lowered = true;
            }
            let mut blockers: Vec<_> = held
                .iter()
                .filter(|(_, illegal)| **illegal)
                .map(|(v, _)| v.clone())
                .collect();
            blockers.sort();
            if sums.sleep.contains(&name) && !blockers.is_empty() && !sup {
                findings.push((line+1,"IRQL_SLEEP_WHILE_HELD".into(),format!("call to may-sleep `{name}` while holding {} — a wait above APC_LEVEL (park with a live raise/spinlock)",blockers.join(", "))))
            }
            if CPS.contains(&name.as_str()) && args.is_some() && !sup {
                let real = real_args(args);
                if real
                    .last()
                    .is_some_and(|x| x.kind() == "closure_expression")
                {
                    let cl = *real.last().unwrap();
                    let closure_ids = all_ids(txt(cl, s));
                    if lets.is_none() {
                        let mut vals = Vec::new();
                        // `nodes` is the already collected body walk. Reuse it instead
                        // of walking the whole body again on the first continuation.
                        for &x in &nodes {
                            if x.kind() == "let_declaration" {
                                if let (Some(pat), Some(val)) = (
                                    x.child_by_field_name("pattern"),
                                    x.child_by_field_name("value"),
                                ) {
                                    if let Some(id) = IDENT.find(txt(pat, s)) {
                                        vals.push((
                                            id.as_str().to_owned(),
                                            txt(val, s).to_owned(),
                                            x.start_byte(),
                                            x.end_byte(),
                                        ));
                                    }
                                }
                            }
                        }
                        lets = Some(vals);
                    }
                    for (var, val, start, _) in lets.as_ref().unwrap() {
                        if closure_ids.contains(var)
                            && !(cl.start_byte() <= *start && *start < cl.end_byte())
                            && CTX.is_match(val)
                        {
                            findings.push((line+1,"CTX_CAPTURE_ACROSS_WAIT".into(),format!("`{name}` continuation captures `{var}` (minted from per-CPU state) — stale after the switch; re-derive it inside the continuation")));
                        }
                    }
                }
            }
        }
    }
    if lowered {
        return findings;
    }
    findings.retain(|x| x.1 == "CTX_CAPTURE_ACROSS_WAIT");
    findings
}
fn scan_file(f: &SourceFile, sums: &Sums) -> Result<Vec<(usize, String, String)>, String> {
    if skip(f) {
        return Ok(vec![]);
    }
    let tree = f.rust_tree()?;
    let lines: Vec<_> = f.text.split('\n').collect();
    let mut out = Vec::new();
    for fun in functions(&tree) {
        out.extend(scan_body(
            fun.child_by_field_name("body").unwrap(),
            &f.text,
            &lines,
            sums,
        ));
    }
    out.sort();
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    fn file(path: &str, s: &str) -> SourceFile {
        SourceFile::new(PathBuf::from(path), s.into())
    }
    #[test]
    fn preorder_is_the_legacy_node_sort_order() {
        let source = "fn outer(){ if cond { inner(); } fn nested(){ leaf(); } }";
        let tree = crate::syntax::parse(source).unwrap();
        let mut preorder = Vec::new();
        walk(tree.root_node(), &mut preorder);
        let mut sorted = preorder.clone();
        sorted.sort_by_key(|n| (n.start_byte(), usize::MAX - n.end_byte()));
        let key = |nodes: &[Node<'_>]| {
            nodes
                .iter()
                .map(|n| (n.start_byte(), n.end_byte(), n.kind()))
                .collect::<Vec<_>>()
        };
        assert_eq!(key(&preorder), key(&sorted));
    }
    #[test]
    fn controls_leak_and_balance() {
        let bad = file(
            "sdk/lib/rust/bad.rs",
            "pub unsafe fn leaky(cond: bool) -> u32 {\nlet mut old: u8 = 0;\nKeRaiseIrql(/* NewIrql */ DISPATCH_LEVEL, &mut old /* OldIrql */);\nif cond { return 1; }\nKeLowerIrql(old);\n0\n}",
        );
        let good = file(
            "sdk/lib/rust/good.rs",
            "pub unsafe fn balanced(cond: bool) -> u32 { let mut old=0; KeRaiseIrql(DISPATCH_LEVEL,&mut old); let rc=if cond {1}else{0}; KeLowerIrql(old); rc }",
        );
        let (sums, _) = build_summaries(&[&bad, &good]).unwrap();
        assert_eq!(scan_file(&bad, &sums).unwrap()[0].1, "IRQL_LEAK_RETURN");
        assert!(scan_file(&good, &sums).unwrap().is_empty());
    }
    #[test]
    fn cross_file_raise_wrapper_summary() {
        let w = file(
            "sdk/lib/rust/wrapper.rs",
            "unsafe fn raise_wrapper()->u8 {let saved=KeRaiseIrqlToDpcLevel(); return saved;}",
        );
        let c = file(
            "sdk/lib/rust/caller.rs",
            "unsafe fn caller(cond:bool){let saved=raise_wrapper();if cond{return;}KeLowerIrql(saved);}",
        );
        let (sums, _) = build_summaries(&[&w, &c]).unwrap();
        assert!(scan_file(&c, &sums)
            .unwrap()
            .iter()
            .any(|x| x.1 == "IRQL_LEAK_RETURN"));
    }
    #[test]
    fn preorder_walk_preserves_nested_exit_and_capture_findings() {
        let f = file(
            "sdk/lib/rust/many.rs",
            "fn f(){ let p=KeGetPcr(); wait_for(|| use_it(p)); }\nfn g(){let mut old=0;KeRaiseIrql(DISPATCH_LEVEL,&mut old);if c{return;}KeLowerIrql(old);}",
        );
        let (sums, _) = build_summaries(&[&f]).unwrap();
        let found = scan_file(&f, &sums).unwrap();
        assert!(found.iter().any(|x| x.1 == "CTX_CAPTURE_ACROSS_WAIT"));
        assert!(found.iter().any(|x| x.1 == "IRQL_LEAK_RETURN"));
    }
    #[test]
    fn passive_only_wait_is_not_propagated() {
        let f = file(
            "sdk/lib/rust/ke.rs",
            "fn KeSynchronizeExecution(){IopSync();}\nfn IopSync(){KeWaitForSingleObject();}\nfn Caller(){KeSynchronizeExecution();}",
        );
        let (sums, _) = build_summaries(&[&f]).unwrap();
        assert!(sums.sleep.contains("IopSync"));
        assert!(!sums.sleep.contains("KeSynchronizeExecution"));
        assert!(!sums.sleep.contains("Caller"));
    }
    #[test]
    fn sleep_edges_are_the_legacy_textual_body_tokens() {
        let f = file(
            "sdk/lib/rust/comments.rs",
            "fn Caller(){ /* KeWaitForSingleObject() */ }",
        );
        let (sums, _) = build_summaries(&[&f]).unwrap();
        assert!(sums.sleep.contains("Caller"));
    }
}
