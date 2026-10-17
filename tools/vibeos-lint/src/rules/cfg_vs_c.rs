//! Compare in-body Rust architecture cfgs with the partitions drawn by the C twin.
use crate::{twins, Corpus, Rule, SourceSet};
use regex::Regex;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::LazyLock,
};

pub struct CfgVsC;
const U: &[&str] = &[
    "x86",
    "x86/mp",
    "x86_64",
    "x86_64/mp",
    "aarch64",
    "aarch64/mp",
];
const ARCHES: &[&str] = &["x86", "x86_64", "aarch64"];
const INPUTS: &[SourceSet] =
    &[SourceSet::tree("cfg-vs-c-rust", &["sdk/lib/rust"], &["rs"], &["Cargo.toml"]).with_inputs()];
const NOLINT: &str = "nolint: cfg-vs-c";
static C_SMP: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"\bCONFIG_SMP\b").unwrap());
static C_ARM64: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(
        r"_M_ARM64|_ARM64_|__aarch64__|_WIN64|_M_ARM\b|_ARM_\b|__arm__|_MI_PAGING_LEVELS",
    )
    .unwrap()
});
static C_TOK: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"\s*(defined|\|\||&&|==|!=|>=|<=|>|<|!|\(|\)|[A-Za-z_]\w*|\d+)")
        .unwrap()
});
static PP: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"(?m)^[ \t]*#[ \t]*(if|ifdef|ifndef|elif|else|endif)\b(.*)$").unwrap()
});
static CFG: LazyLock<Regex> = LazyLock::new(|| crate::regexes::cached(r"#\[cfg\(").unwrap());
static FN_DEF: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"\bfn\s+([A-Za-z_]\w*)\s*(?:<[^>{;]*>)?\s*\([^;{]*\)[^;{]*\{").unwrap()
});
static EXEMPT: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"^\s*(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?(?:extern\b|use\b|type\b|mod\b|global_asm!|(?:core::arch::)?asm!|\{\s*(?:core::arch::)?asm!)").unwrap()
});
static CFG_LINE: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"^\s*#\[cfg\((.*)\)\]\s*(?://.*)?$").unwrap());
static RUST_ARCH_PRED: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r#"^target_arch\s*=\s*"(\w+)"$"#).unwrap());
static RUST_WIDTH_PRED: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r#"^target_pointer_width\s*=\s*"(\d+)"$"#).unwrap());
static RUST_FEATURE_PRED: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r#"^feature\s*=\s*"smp"$"#).unwrap());
static RUST_COMBO_PRED: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"(?s)^(not|any|all)\s*\((.*)\)$").unwrap());
static TARGET_WORD: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r#"target_arch|target_pointer_width|feature\s*=\s*"smp""#).unwrap()
});
static OS_WINDOWS: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r#"^target_os\s*=\s*"windows"$"#).unwrap());
static NOT_WINDOWS: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r#"^not\s*\(\s*target_os\s*=\s*"windows"\s*\)$"#).unwrap()
});
static RS_ARCH: &[(&str, &[&str])] = &[
    ("x86", &["x86"]),
    ("x86_64", &["x86_64"]),
    ("aarch64", &["aarch64"]),
    ("arm", &[]),
];

fn canonical(values: Vec<String>) -> Vec<String> {
    U.iter()
        .filter(|x| values.iter().any(|v| v.as_str() == **x))
        .map(|x| (*x).to_owned())
        .collect()
}
fn lift(arches: &[&str]) -> Vec<String> {
    canonical(
        arches
            .iter()
            .flat_map(|a| [a.to_string(), format!("{a}/mp")])
            .collect(),
    )
}
fn split_args(s: &str) -> Vec<&str> {
    let (mut d, mut start) = (0i32, 0usize);
    let mut out = Vec::new();
    for (i, c) in s.char_indices() {
        if c == '(' {
            d += 1
        } else if c == ')' {
            d -= 1
        }
        if c == ',' && d == 0 {
            out.push(s[start..i].trim());
            start = i + 1;
        }
    }
    if !s[start..].trim().is_empty() {
        out.push(s[start..].trim());
    }
    out
}
fn universe() -> Vec<String> {
    U.iter().map(|x| (*x).to_owned()).collect()
}
fn complement(s: &[String], u: &[String]) -> Vec<String> {
    canonical(u.iter().filter(|x| !s.contains(x)).cloned().collect())
}
fn union(a: &[String], b: &[String]) -> Vec<String> {
    canonical(a.iter().chain(b).cloned().collect())
}
fn intersect(a: &[String], b: &[String]) -> Vec<String> {
    canonical(a.iter().filter(|x| b.contains(x)).cloned().collect())
}
fn rust_set(pred: &str) -> Option<Vec<String>> {
    let p = pred.trim();
    if let Some(m) = RUST_ARCH_PRED.captures(p) {
        return RS_ARCH.iter().find(|x| x.0 == &m[1]).map(|x| lift(x.1));
    }
    if RUST_FEATURE_PRED.is_match(p) {
        return Some(
            U.iter()
                .filter(|x| x.ends_with("/mp"))
                .map(|x| (*x).into())
                .collect(),
        );
    }
    if let Some(m) = RUST_WIDTH_PRED.captures(p) {
        return Some(lift(if &m[1] == "64" {
            &["x86_64", "aarch64"]
        } else {
            &["x86"]
        }));
    }
    if let Some(m) = RUST_COMBO_PRED.captures(p) {
        let op = &m[1];
        let parts: Option<Vec<_>> = split_args(&m[2]).into_iter().map(rust_set).collect();
        let parts = parts?;
        if op == "not" {
            return if parts.len() == 1 {
                Some(complement(&parts[0], &universe()))
            } else {
                None
            };
        }
        let mut acc = if op == "any" { Vec::new() } else { universe() };
        for x in parts {
            acc = if op == "any" {
                union(&acc, &x)
            } else {
                intersect(&acc, &x)
            };
        }
        return Some(acc);
    }
    None
}
fn c_arch(name: &str) -> Option<Vec<String>> {
    let x = match name {
        "_M_IX86" | "_X86_" | "__i386__" | "i386" | "_M_I386" => lift(&["x86"]),
        "_M_AMD64" | "_AMD64_" | "__x86_64__" | "_M_X64" | "__amd64__" => lift(&["x86_64"]),
        "_M_ARM64" | "_ARM64_" | "__aarch64__" => lift(&["aarch64"]),
        "_M_ARM" | "_ARM_" | "__arm__" => Vec::new(),
        "_WIN64" => lift(&["x86_64", "aarch64"]),
        "_WIN32" => universe(),
        "CONFIG_SMP" => U
            .iter()
            .filter(|x| x.ends_with("/mp"))
            .map(|x| (*x).into())
            .collect(),
        _ => return None,
    };
    Some(x)
}
fn c_set(expr: &str) -> Option<Vec<String>> {
    let toks: Vec<String> = C_TOK
        .find_iter(expr.trim())
        .map(|m| m.as_str().trim().into())
        .collect();
    if toks.join("")
        != expr
            .trim()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
    {
        return None;
    }
    let mut pos = 0usize;
    fn peek(t: &[String], p: usize) -> Option<&str> {
        t.get(p).map(String::as_str)
    }
    fn primary(t: &[String], p: &mut usize) -> Option<Vec<String>> {
        let tok = peek(t, *p)?;
        *p += 1;
        if tok == "!" {
            let v = primary(t, p)?;
            return Some(complement(&v, &universe()));
        }
        if tok == "(" {
            let v = orx(t, p)?;
            if peek(t, *p)? != ")" {
                return None;
            }
            *p += 1;
            return Some(v);
        }
        if tok == "defined" {
            let par = peek(t, *p) == Some("(");
            if par {
                *p += 1
            }
            let n = peek(t, *p)?;
            *p += 1;
            if par {
                if peek(t, *p)? != ")" {
                    return None;
                }
                *p += 1
            }
            return c_arch(n);
        }
        if tok == "_MI_PAGING_LEVELS"
            && matches!(peek(t, *p), Some("==" | "!=" | ">=" | "<=" | ">" | "<"))
        {
            let op = peek(t, *p)?;
            *p += 1;
            let n = peek(t, *p)?.parse::<i32>().ok()?;
            *p += 1;
            let vals = [("x86", 2), ("x86_64", 4), ("aarch64", 4)];
            let a: Vec<_> = vals
                .iter()
                .filter(|(_, v)| match op {
                    "==" => *v == n,
                    "!=" => *v != n,
                    ">=" => *v >= n,
                    "<=" => *v <= n,
                    ">" => *v > n,
                    "<" => *v < n,
                    _ => false,
                })
                .map(|(a, _)| *a)
                .collect();
            return Some(lift(&a));
        }
        c_arch(tok)
    }
    fn andx(t: &[String], p: &mut usize) -> Option<Vec<String>> {
        let mut v = primary(t, p)?;
        while peek(t, *p) == Some("&&") {
            *p += 1;
            v = intersect(&v, &primary(t, p)?)
        }
        Some(v)
    }
    fn orx(t: &[String], p: &mut usize) -> Option<Vec<String>> {
        let mut v = andx(t, p)?;
        while peek(t, *p) == Some("||") {
            *p += 1;
            v = union(&v, &andx(t, p)?)
        }
        Some(v)
    }
    let v = orx(&toks, &mut pos)?;
    if pos == toks.len() {
        Some(v)
    } else {
        None
    }
}
fn c_branch_sets(body: &str) -> Vec<(Vec<String>, String)> {
    let mut out = Vec::new();
    let mut stack: Vec<Vec<Option<Vec<String>>>> = Vec::new();
    for m in PP.captures_iter(body) {
        let kind = &m[1];
        let rest = m[2]
            .split("//")
            .next()
            .unwrap_or("")
            .split("/*")
            .next()
            .unwrap_or("")
            .trim();
        if ["if", "ifdef", "ifndef", "elif"].contains(&kind) {
            let s = match kind {
                "ifdef" => c_set(&format!("defined({rest})")),
                "ifndef" => c_set(&format!("!defined({rest})")),
                _ => c_set(rest),
            };
            if kind == "elif" {
                if let Some(x) = stack.last_mut() {
                    x.push(s.clone())
                } else {
                    continue;
                }
            } else {
                stack.push(vec![s.clone()])
            }
            if let Some(x) = s {
                out.push((x, format!("#{kind} {rest}")))
            }
        } else if kind == "else" {
            if let Some(prior) = stack.last_mut() {
                if prior.iter().all(Option::is_some) {
                    let mut excl = Vec::new();
                    for x in prior.iter().flatten() {
                        excl = union(&excl, x)
                    }
                    let s = complement(&excl, &universe());
                    out.push((s, "#else".into()))
                }
                prior.push(None)
            }
        } else if kind == "endif" && !stack.is_empty() {
            stack.pop();
        }
    }
    out
}
fn atoms(sets: &[Vec<String>], u: &[String]) -> Vec<Vec<String>> {
    let mut parts = vec![u.to_vec()];
    for s in sets {
        let mut next = Vec::new();
        for p in parts {
            let yes = intersect(&p, s);
            let no = complement(s, &p);
            if !yes.is_empty() {
                next.push(yes)
            }
            if !no.is_empty() {
                next.push(no)
            }
        }
        parts = next
    }
    parts
}
fn splits(s: &[String], parts: &[Vec<String>]) -> Option<Vec<String>> {
    parts
        .iter()
        .find(|p| !intersect(p, s).is_empty() && !complement(s, p).is_empty())
        .cloned()
}
fn show(points: &[String]) -> Vec<String> {
    ARCHES
        .iter()
        .filter_map(|a| {
            let up = points.iter().any(|p| p == a);
            let mp = points.iter().any(|p| p == &format!("{a}/mp"));
            if !up && !mp {
                None
            } else if up && mp {
                Some((*a).into())
            } else {
                Some(format!("{a} ({})", if mp { "SMP" } else { "UP" }))
            }
        })
        .collect()
}
fn rust_cfgs(body: &str) -> Vec<(usize, String, String)> {
    let mut out = Vec::new();
    for m in CFG.find_iter(body) {
        let mut depth = 1i32;
        let mut i = m.end();
        for (off, ch) in body[m.end()..].char_indices() {
            if ch == '(' {
                depth += 1
            } else if ch == ')' {
                depth -= 1;
                if depth == 0 {
                    i = m.end() + off;
                    break;
                }
            }
        }
        if depth != 0 {
            continue;
        }
        let pred = &body[m.end()..i];
        let close = body[i..].find(']').map(|x| i + x);
        if let Some(close) = close {
            out.push((
                m.start(),
                pred.into(),
                body[close + 1..].chars().take(200).collect(),
            ))
        }
    }
    out
}
fn stacked_dead(path: &Path, raw: &str) -> Vec<(usize, String)> {
    let lines: Vec<_> = raw.split('\n').collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let (mut j, mut run) = (i, Vec::new());
        while j < lines.len()
            && (CFG_LINE.is_match(lines[j]) || (!run.is_empty() && lines[j].trim().is_empty()))
        {
            if let Some(c) = CFG_LINE.captures(lines[j]) {
                run.push((j + 1, c[1].to_string()))
            }
            j += 1
        }
        if run.len() >= 2 && !lines[run[0].0 - 1].contains(NOLINT) {
            let sets: Option<Vec<_>> = run.iter().map(|(_, p)| os_or_arch(p)).collect();
            if let Some(ss) = sets {
                let mut all = universe();
                for x in &ss {
                    all = intersect(&all, x)
                }
                if all.is_empty() {
                    let preds = run
                        .iter()
                        .map(|(_, p)| p.split_whitespace().collect::<Vec<_>>().join(" "))
                        .collect::<Vec<_>>()
                        .join(" & ");
                    out.push((run[0].0, preds))
                }
            }
        }
        i = j.max(i + 1)
    }
    let _ = path;
    out
}
fn os_or_arch(pred: &str) -> Option<Vec<String>> {
    if OS_WINDOWS.is_match(pred.trim()) {
        Some(universe())
    } else if NOT_WINDOWS.is_match(pred.trim()) {
        Some(Vec::new())
    } else {
        rust_set(pred)
    }
}
fn crate_of(path: &Path, crate_dirs: &std::collections::BTreeSet<PathBuf>) -> String {
    path.ancestors()
        .find(|parent| crate_dirs.contains(*parent))
        .map(|x| x.to_string_lossy().into_owned())
        .unwrap_or_default()
}
fn line_starts(text: &str) -> Vec<usize> {
    let mut out = vec![0];
    out.extend(text.match_indices('\n').map(|(i, _)| i + 1));
    out
}
fn lineno(starts: &[usize], at: usize) -> usize {
    starts.partition_point(|&start| start <= at)
}
impl Rule for CfgVsC {
    fn name(&self) -> &'static str {
        "scan_cfg_vs_c"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn needs_c_twins(&self) -> bool {
        true
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let files: Vec<_> = c
            .sources("cfg-vs-c-rust")
            .filter(|f| f.path.extension().is_some_and(|e| e == "rs"))
            .collect();
        let manifests: Vec<_> = c
            .sources("cfg-vs-c-rust")
            .filter(|f| f.path.file_name().is_some_and(|n| n == "Cargo.toml"))
            .map(|f| f.path.clone())
            .collect();
        let crate_dirs: std::collections::BTreeSet<_> = manifests
            .iter()
            .filter_map(|p| p.parent().map(Path::to_path_buf))
            .collect();
        let mut defs = HashMap::<(String, String), usize>::new();
        let mut parsed = Vec::new();
        for f in &files {
            let raw = &f.text;
            let owner = crate_of(&f.path, &crate_dirs);
            for m in FN_DEF.captures_iter(raw) {
                *defs.entry((owner.clone(), m[1].into())).or_default() += 1
            }
            if raw.contains("#[cfg(") && raw.contains("target_") {
                parsed.push((*f, owner))
            }
        }
        let mut sites = Vec::new();
        let mut dead = Vec::new();
        for (f, owner) in parsed {
            let raw = &f.text;
            let starts = line_starts(raw);
            let raw_lines: Vec<_> = raw.lines().collect();
            let blank = twins::blank_rust(raw);
            for fnc in twins::rust_functions(&blank, &f.path) {
                let body = &raw[fnc.body_start..fnc.body_start + fnc.body.len()];
                for (off, pred, nxt) in rust_cfgs(body) {
                    if !TARGET_WORD.is_match(&pred) {
                        continue;
                    }
                    let Some(s) = rust_set(&pred) else { continue };
                    if s == universe() || s.is_empty() {
                        continue;
                    }
                    if EXEMPT.is_match(&nxt)
                        || nxt
                            .split(';')
                            .next()
                            .unwrap_or("")
                            .contains("__bindgen_anon_")
                    {
                        continue;
                    }
                    if defs
                        .get(&(owner.clone(), fnc.name.clone()))
                        .copied()
                        .unwrap_or(0)
                        > 1
                    {
                        continue;
                    }
                    let line = lineno(&starts, fnc.body_start + off);
                    if raw_lines.get(line - 1).unwrap_or(&"").contains(NOLINT) {
                        continue;
                    }
                    sites.push((f, fnc.name.clone(), line, pred, s, nxt));
                }
            }
            dead.extend(
                stacked_dead(&f.path, raw)
                    .into_iter()
                    .map(|(ln, p)| (f, ln, p)),
            );
        }
        let index = c.c_twins();
        let mut out = Vec::new();
        for (f, name, line, pred, s, nxt) in sites {
            let twins = twins::prefer(&f.path, index.get(&name));
            let Some(t) = twins.into_iter().find(|t| twins::same_module(&f.path, t)) else {
                continue;
            };
            let branches = c_branch_sets(&t.body);
            if branches.is_empty() {
                continue;
            }
            let u = if branches.iter().any(|(_, x)| C_ARM64.is_match(x)) {
                universe()
            } else {
                lift(&["x86", "x86_64"])
            };
            let u = if branches.iter().any(|(_, x)| C_SMP.is_match(x)) {
                u
            } else {
                u.into_iter().filter(|x| !x.ends_with("/mp")).collect()
            };
            let sets: Vec<_> = branches.iter().map(|(x, _)| intersect(x, &u)).collect();
            let ss = intersect(&s, &u);
            let Some(cut) = splits(&ss, &atoms(&sets, &u)) else {
                continue;
            };
            let ev = branches
                .iter()
                .find(|(b, _)| *b == cut || cut.iter().all(|x| b.contains(x)))
                .map(|x| x.1.as_str())
                .unwrap_or(&branches[0].1);
            out.push(format!("{}:{}: cfg-vs-c {name}", f.path.display(), line));
            let _ = (pred, nxt, ev, show(&s), show(&cut));
        }
        for (f, ln, _) in dead {
            out.push(format!("{}:{ln}: dead-cfg", f.path.display()))
        }
        out.sort();
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn indexed_line_lookup_matches_legacy_prefix_count() {
        let src = "a\nbb\nccc";
        let starts = line_starts(src);
        for at in 0..=src.len() {
            assert_eq!(
                lineno(&starts, at),
                src[..at].bytes().filter(|b| *b == b'\n').count() + 1
            );
        }
    }
    #[test]
    fn architecture_predicates_and_c_branches() {
        assert_eq!(
            rust_set("target_pointer_width = \"64\"").unwrap(),
            lift(&["x86_64", "aarch64"])
        );
        let b = c_branch_sets("#ifdef _WIN64\nA();\n#else\nB();\n#endif");
        assert_eq!(b.len(), 2);
        let p = atoms(
            &b.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
            &universe(),
        );
        assert!(splits(&lift(&["x86_64"]), &p).is_some());
    }
    #[test]
    fn dead_cfg_control() {
        let src =
            "#[cfg(not(target_arch = \"x86\"))]\n#[cfg(target_arch = \"x86\")]\nfn dead() {}\n";
        let got = stacked_dead(Path::new("dead.rs"), src);
        assert_eq!(got.len(), 1);
        assert!(stacked_dead(
            Path::new("good.rs"),
            "#[cfg(target_arch = \"x86\")]\nfn ok() {}\n"
        )
        .is_empty());
    }
    #[test]
    fn cfg_finding_uses_the_c_twin_partition() {
        let rust = "pub fn KeBugCheckWithTf() {\n    #[cfg(target_arch = \"x86\")]\n    { KxFreezeExecution(); }\n}\n";
        let c_src = "void KeBugCheckWithTf(void) {\n#ifdef CONFIG_SMP\n    KxFreezeExecution();\n#else\n    KeBugCheck();\n#endif\n}\n";
        let corpus = Corpus::new(
            PathBuf::from("."),
            vec![
                crate::SourceFile::new(
                    "sdk/lib/rust/ntoskrnl-rust/Cargo.toml".into(),
                    "[package]".into(),
                ),
                crate::SourceFile::new("sdk/lib/rust/ntoskrnl-rust/src/bug.rs".into(), rust.into()),
                crate::SourceFile::new("ntoskrnl/ke/bug.c".into(), c_src.into()),
            ],
        );
        let got = CfgVsC.run(&corpus).unwrap();
        assert_eq!(got.len(), 1, "{got:?}");
        assert!(got[0].ends_with(":2: cfg-vs-c KeBugCheckWithTf"), "{got:?}");
    }
}
