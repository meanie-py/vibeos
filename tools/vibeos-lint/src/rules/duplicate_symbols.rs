use crate::link_model::{self, Crate as LinkCrate, Def, ARCHES};
use crate::{Corpus, Rule, SourceFile, SourceSet};
use regex::Regex;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::LazyLock,
};

pub struct DuplicateSymbols;
static SOURCES: &[SourceSet] = &[
    SourceSet::tracked("link-manifests", &[], &["Cargo.toml"]),
    SourceSet::tree(
        "link-inputs",
        &["sdk/lib/rust"],
        &["rs", "s", "S", "asm"],
        &[],
    )
    .with_inputs(),
];
static STRUCTURAL: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(
        r"^(rust_eh_personality|rust_begin_unwind|rust_oom|__rust_[a-z_]+|_fltused)$",
    )
    .unwrap()
});
static LINK_CALL: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r#"\.rust_(?:staticlib|lib)\s*\(\s*"([^"]+)""#).unwrap()
});
static LIB: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(
        r#"\.rust_lib\s*\(\s*"([^"]+)"\s*,\s*(&\[[^\]]*\]|\w+)\s*,\s*(true|false)"#,
    )
    .unwrap()
});
static STATICLIB: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r#"\.rust_staticlib\s*\(\s*"([^"]+)""#).unwrap());
static FEATS: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r#"\.rust_features\s*\(\s*&\[([^\]]*)\]"#).unwrap());
static HELPER: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"\b(\w+)_link::").unwrap());
static STRINGS: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r#""([^\"]*)""#).unwrap());

#[derive(Clone, Debug, Default)]
pub struct Unit {
    pub crates: Vec<String>,
    pub pinned: BTreeMap<String, bool>,
}
fn strip_comments(src: &str) -> String {
    crate::regexes::cached(r"//[^\n]*")
        .unwrap()
        .replace_all(src, "")
        .into_owned()
}
fn arch_guard(cond: &str) -> Option<BTreeSet<String>> {
    let cond = cond.trim().trim_matches('(').trim_matches(')');
    for (op, union) in [("||", true), ("&&", false)] {
        if cond.contains(op) {
            let mut parts = cond.split(op).map(arch_guard);
            let mut acc = parts.next()??;
            for p in parts {
                let p = p?;
                if union {
                    acc.extend(p)
                } else {
                    acc.retain(|x| p.contains(x));
                }
            }
            return Some(acc);
        }
    }
    static G: LazyLock<Regex> = LazyLock::new(|| {
        crate::regexes::cached(r#"^arch(?:\.as_str\(\))?\s*(==|!=)\s*"(\w+)"$"#).unwrap()
    });
    let c = G.captures(cond)?;
    let all: BTreeSet<String> = ARCHES.iter().map(|x| x.to_string()).collect();
    if &c[1] == "==" {
        Some([c[2].to_string()].into_iter().collect())
    } else {
        Some(all.into_iter().filter(|x| x != &c[2]).collect())
    }
}
fn arches_at(src: &str) -> Vec<BTreeSet<String>> {
    let all: BTreeSet<String> = ARCHES.iter().map(|x| x.to_string()).collect();
    let mut out = Vec::with_capacity(src.len());
    let mut stack = vec![all.clone()];
    let mut last_if: Option<BTreeSet<String>> = None;
    let mut header_start = 0;
    for (i, ch) in src.char_indices() {
        out.push(stack.last().unwrap().clone());
        match ch {
            '{' => {
                let header = &src[header_start..i];
                let cur = stack.last().unwrap().clone();
                static IF: LazyLock<Regex> = LazyLock::new(|| {
                    crate::regexes::cached(r"(?s)(?:^|\s)(else\s+)?if\s+(.*)$").unwrap()
                });
                static OTHER: LazyLock<Regex> =
                    LazyLock::new(|| crate::regexes::cached(r"\bmatch\b|=>|\blet\b").unwrap());
                static ELSE: LazyLock<Regex> =
                    LazyLock::new(|| crate::regexes::cached(r"(?:^|\s)else\s*$").unwrap());
                if let Some(m) = IF.captures(header).filter(|m| !OTHER.is_match(&m[2])) {
                    let g = arch_guard(&m[2]);
                    let mut sel = g.clone().unwrap_or_else(|| all.clone());
                    if m.get(1).is_some() {
                        if let Some(prev) = &last_if {
                            sel = sel.difference(prev).cloned().collect();
                        }
                    }
                    last_if = if g.is_some() { Some(sel.clone()) } else { None };
                    stack.push(cur.intersection(&sel).cloned().collect());
                } else if ELSE.is_match(header) && last_if.is_some() {
                    let prev = last_if.take().unwrap();
                    stack.push(cur.difference(&prev).cloned().collect());
                } else {
                    stack.push(cur);
                }
                header_start = i + 1;
            }
            '}' => {
                if stack.len() > 1 {
                    stack.pop();
                }
                header_start = i + 1;
            }
            ';' => {
                header_start = i + 1;
                last_if = None;
            }
            _ => {}
        }
    }
    out
}
fn strings(s: &str) -> BTreeSet<String> {
    STRINGS.captures_iter(s).map(|m| m[1].to_string()).collect()
}
fn unit(
    all: &BTreeMap<String, LinkCrate>,
    roots: &BTreeMap<String, (BTreeSet<String>, bool)>,
    maybe: &BTreeSet<String>,
) -> Unit {
    let (on, unknown) = link_model::resolve_features(all, roots, maybe);
    let mut pinned = BTreeMap::new();
    for (name, fs) in &on {
        if let Some(c) = all.get(name) {
            for f in c.features.keys() {
                if f != "default" && !unknown.get(name).is_some_and(|u| u.contains(f)) {
                    pinned.insert(link_model::feature_atom(name, f), fs.contains(f));
                }
            }
        }
    }
    Unit {
        crates: on.keys().cloned().collect(),
        pinned,
    }
}
/// One feature-unified path-dependency unit per SDK build script and staticlib crate.
pub fn link_units(c: &Corpus) -> BTreeMap<String, BTreeMap<String, Unit>> {
    let all = link_model::crates(c);
    let mut units = BTreeMap::new();
    let mut builds = c
        .sources("link-build-inputs")
        .filter(|f| f.path.file_name().is_some_and(|n| n == "build.rs"))
        .collect::<Vec<_>>();
    builds.sort_by(|a, b| a.path.cmp(&b.path));
    for br in builds {
        let mut src = strip_comments(&br.text);
        let mut helpers = HELPER
            .find_iter(&src)
            .map(|m| m.as_str().trim_end_matches("_link::").to_owned())
            .collect::<BTreeSet<_>>();
        helpers.remove("reactos");
        for helper in helpers {
            let name = format!("{}-link", helper.replace('_', "-"));
            if let Some(k) = all.get(&name) {
                let p = k.dir.join("src/lib.rs");
                if let Some(f) = c.files.iter().find(|f| f.path == p) {
                    src.push('\n');
                    src.push_str(&strip_comments(&f.text));
                }
            }
        }
        let at = arches_at(&src);
        let maybe: BTreeSet<String> = STRINGS
            .captures_iter(&src)
            .map(|m| m[1].to_string())
            .filter(|s| {
                s.chars()
                    .all(|x| x.is_ascii_alphanumeric() || x == '_' || x == '-')
            })
            .collect();
        let mut per = BTreeMap::new();
        for arch in ARCHES {
            let mut roots = BTreeMap::<String, (BTreeSet<String>, bool)>::new();
            for m in LIB.captures_iter(&src) {
                let pos = src[..m.get(0).unwrap().start()].chars().count();
                if at.get(pos).is_some_and(|a| a.contains(arch)) {
                    let fs = if m[2].starts_with('&') {
                        strings(&m[2])
                    } else {
                        BTreeSet::new()
                    };
                    let e = roots.entry(m[1].to_string()).or_default();
                    e.0.extend(fs);
                    e.1 = &m[3] == "true";
                }
            }
            let mut off = 0;
            for stmt in src.split(';') {
                if let Some(m) = STATICLIB.captures(stmt) {
                    let pos = src[..off + m.get(0).unwrap().start()].chars().count();
                    if at.get(pos).is_some_and(|a| a.contains(arch)) {
                        let fs = FEATS
                            .captures_iter(stmt)
                            .flat_map(|x| strings(&x[1]))
                            .collect();
                        roots.insert(m[1].to_string(), (fs, true));
                    }
                }
                off += stmt.len() + 1;
            }
            for m in LINK_CALL.captures_iter(&src) {
                let pos = src[..m.get(0).unwrap().start()].chars().count();
                if at.get(pos).is_some_and(|a| a.contains(arch)) {
                    roots
                        .entry(m[1].to_string())
                        .or_insert((BTreeSet::new(), true));
                }
            }
            roots.retain(|n, _| all.contains_key(n));
            if !roots.is_empty() {
                per.insert(arch.to_string(), unit(&all, &roots, &maybe));
            }
        }
        if !per.is_empty() {
            let name = br
                .path
                .parent()
                .and_then(Path::file_name)
                .unwrap()
                .to_string_lossy()
                .to_string();
            units.insert(name, per);
        }
    }
    for k in all.values() {
        if k.staticlib && !units.contains_key(&k.name) {
            let root = [(k.name.clone(), (BTreeSet::new(), true))]
                .into_iter()
                .collect();
            let u = unit(&all, &root, &BTreeSet::new());
            units.insert(
                k.name.clone(),
                ARCHES.iter().map(|a| (a.to_string(), u.clone())).collect(),
            );
        }
    }
    units
}
fn nolint(c: &Corpus, d: &Def) -> bool {
    let Some(f) = c.files.iter().find(|f| f.path == d.path) else {
        return false;
    };
    let lines = f.text.lines().collect::<Vec<_>>();
    let lo = d.line.saturating_sub(6);
    lines
        .get(lo..d.line)
        .unwrap_or(&[])
        .iter()
        .any(|l| l.contains("nolint: duplicate-symbol"))
}
fn fmt(d: &Def) -> String {
    format!("{} ({}, {} {})", d.loc(), d.krate, d.kind, d.name)
}
impl Rule for DuplicateSymbols {
    fn name(&self) -> &'static str {
        "scan_duplicate_symbols"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        SOURCES
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let all = link_model::crates(c);
        let units = link_units(c);
        let mut findings =
            BTreeMap::<(String, String, String), (BTreeSet<String>, BTreeSet<String>)>::new();
        for (uname, per) in &units {
            for (arch, u) in per {
                let mut defs = Vec::new();
                for name in &u.crates {
                    if let Some(k) = all.get(name) {
                        defs.extend(link_model::crate_defs(c, k));
                    }
                }
                let mut by = BTreeMap::<String, Vec<Def>>::new();
                for d in defs {
                    if STRUCTURAL.is_match(&d.base())
                        || !d.cond.satisfiable_with(&d.cond, arch, &u.pinned)
                    {
                        continue;
                    }
                    by.entry(d.link_name(arch)).or_default().push(d);
                }
                for group in by.values_mut() {
                    group.sort_by_key(|d| (d.loc(), d.name.clone()));
                    group.dedup_by(|a, b| a.loc() == b.loc() && a.name == b.name);
                    for i in 0..group.len() {
                        for j in i + 1..group.len() {
                            let (a, b) = (&group[i], &group[j]);
                            if a.loc() == b.loc()
                                || !a.cond.satisfiable_with(&b.cond, arch, &u.pinned)
                                || nolint(c, a)
                                || nolint(c, b)
                            {
                                continue;
                            }
                            let key = (fmt(a), fmt(b), a.base());
                            let e = findings.entry(key).or_default();
                            e.0.insert(arch.clone());
                            e.1.insert(uname.clone());
                        }
                    }
                }
            }
        }
        let unknown = link_model::unknown_build_cfgs(c, &all);
        let linked = units
            .values()
            .flat_map(|per| per.values())
            .flat_map(|u| u.crates.iter())
            .collect::<BTreeSet<_>>();
        let used = linked
            .iter()
            .filter_map(|n| all.get(*n))
            .flat_map(|k| link_model::crate_defs(c, k))
            .flat_map(|d| d.cond.preds)
            .flat_map(|p| link_model::free_atoms(&p))
            .collect::<BTreeSet<_>>();
        let hits = unknown
            .keys()
            .filter(|n| used.contains(*n))
            .collect::<Vec<_>>();
        if !hits.is_empty() {
            return Err(hits.into_iter().map(|n| format!("error: cfg `{n}` is set by {}; add what it means to BUILD_CFGS in tools/rust_link_model.py", unknown[n])).collect::<Vec<_>>().join("\n"));
        }
        let rank = |a: &str| ARCHES.iter().position(|x| x == &a).unwrap_or(99);
        Ok(findings
            .into_iter()
            .map(|((a, b, base), (arches, _units))| {
                let mut ar = arches.into_iter().collect::<Vec<_>>();
                ar.sort_by_key(|x| rank(x));
                format!("{a}: {base} [{}] also defined at {b}", ar.join(","))
            })
            .collect())
    }
}
