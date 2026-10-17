//! Source link model: cfg conjunctions and Cargo feature unification.
//! Contracts follow tools/rust_link_model.py; filesystem inputs belong to Corpus.
use crate::modules::normalize;
use crate::{Corpus, SourceSet};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

pub const ARCHES: [&str; 3] = ["x86", "x86_64", "aarch64"];
pub const SOURCE_SETS: &[SourceSet] = &[
    SourceSet::tracked("link-manifests", &[], &["Cargo.toml"]),
    SourceSet::tree("link-inputs", &[""], &["rs", "s", "S", "asm"], &[]).with_inputs(),
];
fn re(s: &str) -> regex::Regex {
    crate::regexes::cached(s).expect("link model pattern")
}
pub fn split_args(s: &str) -> Vec<String> {
    let (mut depth, mut quoted, mut start) = (0i32, false, 0);
    let mut out = Vec::new();
    for (i, c) in s.char_indices() {
        match c {
            '"' => quoted = !quoted,
            '(' if !quoted => depth += 1,
            ')' if !quoted => depth -= 1,
            ',' if !quoted && depth == 0 => {
                out.push(s[start..i].trim().to_owned());
                start = i + 1;
            }
            _ => {}
        }
    }
    if !s[start..].trim().is_empty() {
        out.push(s[start..].trim().to_owned());
    }
    out
}
fn atom(s: &str) -> String {
    re(r"\s*=\s*").replace_all(s.trim(), "=").into_owned()
}
fn fixed(s: &str, arch: &str) -> Option<bool> {
    if let Some(m) = re(r#"^target_arch="(\w+)"$"#).captures(s) {
        return Some(&m[1] == arch);
    }
    match s {
        "target_pointer_width=\"64\"" => Some(arch != "x86"),
        "target_pointer_width=\"32\"" => Some(arch == "x86"),
        "target_os=\"windows\""
        | "target_endian=\"little\""
        | "target_family=\"windows\""
        | "windows" => Some(true),
        "target_os=\"none\""
        | "target_os=\"linux\""
        | "target_endian=\"big\""
        | "unix"
        | "test"
        | "doc"
        | "miri" => Some(false),
        _ => None,
    }
}
pub fn eval_pred(pred: &str, arch: &str, free: &BTreeMap<String, bool>) -> bool {
    if let Some(m) = re(r"(?s)^(not|any|all)\s*\((.*)\)$").captures(pred.trim()) {
        let args = split_args(&m[2]);
        return match &m[1] {
            "not" => args.first().is_none_or(|x| !eval_pred(x, arch, free)),
            "any" => args.iter().any(|x| eval_pred(x, arch, free)),
            _ => args.iter().all(|x| eval_pred(x, arch, free)),
        };
    }
    let a = atom(pred);
    fixed(&a, arch).unwrap_or_else(|| free.get(&a).copied().unwrap_or(false))
}
pub fn feature_atom(krate: &str, feat: &str) -> String {
    format!("feature=\"{krate}/{feat}\"")
}
pub fn free_atoms(pred: &str) -> BTreeSet<String> {
    re(r#"[A-Za-z_]\w*(?:\s*=\s*"[^"]*")?"#)
        .find_iter(pred)
        .map(|m| atom(m.as_str()))
        .filter(|s| !matches!(s.as_str(), "not" | "any" | "all") && fixed(s, "x86").is_none())
        .collect()
}
fn build_cfgs(name: &str) -> Vec<(&'static str, &'static str)> {
    match name {
        "hal-rust" => vec![
            (
                "hal_apic",
                r#"any(target_arch = "x86_64", all(target_arch = "x86", feature = "apic"))"#,
            ),
            (
                "hal_pic",
                r#"all(target_arch = "x86", not(feature = "apic"))"#,
            ),
        ],
        "cnt-rust" => vec![
            (
                "cnt_libm",
                r#"not(all(target_arch = "x86", not(target_feature = "sse2"), not(target_feature = "soft-float")))"#,
            ),
            ("streamout_user32", "any()"),
        ],
        "user32-wsprintf-rust" | "cnt-difftest" => vec![("streamout_user32", "all()")],
        _ => vec![],
    }
}
pub fn namespace_features(pred: &str, name: &str) -> String {
    let mut out = pred.to_owned();
    for (n, v) in build_cfgs(name) {
        // Python's negative lookahead excludes cfg key/value atoms.
        let pat = re(&format!(r"\b{}\b", regex::escape(n)));
        let mut buf = String::new();
        let mut last = 0;
        for m in pat.find_iter(&out) {
            buf.push_str(&out[last..m.start()]);
            if out[m.end()..].trim_start().starts_with('=') {
                buf.push_str(m.as_str());
            } else {
                buf.push_str(&format!("all({v})"));
            }
            last = m.end();
        }
        buf.push_str(&out[last..]);
        out = buf;
    }
    re(r#"\bfeature\s*=\s*""#)
        .replace_all(&out, format!("feature=\"{name}/"))
        .into_owned()
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Cond {
    pub preds: Vec<String>,
}
impl Cond {
    pub fn and(&self, preds: impl IntoIterator<Item = String>) -> Self {
        let mut c = self.clone();
        c.preds.extend(preds);
        c
    }
    pub fn namespaced(&self, name: &str) -> Self {
        Self {
            preds: self
                .preds
                .iter()
                .map(|p| namespace_features(p, name))
                .collect(),
        }
    }
    pub fn arches(&self) -> BTreeSet<String> {
        ARCHES
            .into_iter()
            .filter(|a| self.satisfiable_with(&Self::default(), a, &BTreeMap::new()))
            .map(str::to_owned)
            .collect()
    }
    pub fn satisfiable_with(
        &self,
        other: &Self,
        arch: &str,
        pinned: &BTreeMap<String, bool>,
    ) -> bool {
        let preds: Vec<_> = self.preds.iter().chain(&other.preds).collect();
        let atoms: Vec<_> = preds
            .iter()
            .flat_map(|p| free_atoms(p))
            .filter(|a| !pinned.contains_key(a))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        if atoms.len() > 12 {
            return true;
        }
        for bits in 0..(1usize << atoms.len()) {
            let mut free = pinned.clone();
            for (i, a) in atoms.iter().enumerate() {
                free.insert(a.clone(), bits & (1 << i) != 0);
            }
            if preds.iter().all(|p| eval_pred(p, arch, &free)) {
                return true;
            }
        }
        false
    }
}
#[derive(Clone, Debug, Default)]
pub struct Crate {
    pub name: String,
    pub dir: PathBuf,
    pub roots: Vec<PathBuf>,
    pub deps: Vec<String>,
    pub staticlib: bool,
    pub features: BTreeMap<String, Vec<String>>,
    pub edges: BTreeMap<String, (Vec<String>, bool)>,
    pub optional: BTreeSet<String>,
}
#[derive(Default)]
struct Manifest {
    name: Option<String>,
    lib: BTreeMap<String, String>,
    bins: Vec<BTreeMap<String, String>>,
    features: BTreeMap<String, Vec<String>>,
    edges: Vec<(String, Vec<String>, bool, bool)>,
}
fn strings(s: &str) -> Vec<String> {
    re(r#""([^"]*)""#)
        .captures_iter(s)
        .map(|m| m[1].to_owned())
        .collect()
}
fn dep_edge(path: String, val: &str) -> (String, Vec<String>, bool, bool) {
    let fs = re(r"\bfeatures\s*=\s*\[([^\]]*)\]")
        .captures(val)
        .map(|m| strings(&m[1]))
        .unwrap_or_default();
    (
        path,
        fs,
        !re(r"default-features\s*=\s*false").is_match(val),
        re(r"optional\s*=\s*true").is_match(val),
    )
}
fn manifest(text: &str) -> Manifest {
    let mut out = Manifest::default();
    let mut table = String::new();
    let mut lines = text.lines();
    let mut edge_index = None;
    let kv = re(r#"^\s*([\w."-]+)\s*=\s*(.*)$"#);
    let tab = re(r"^\s*\[(\[)?\s*([^\]]+?)\s*\]\]?\s*(?:#.*)?$");
    while let Some(line) = lines.next() {
        if let Some(m) = tab.captures(line) {
            table = m[2].trim().to_owned();
            edge_index = None;
            if m.get(1).is_some() && table == "bin" {
                out.bins.push(BTreeMap::new());
            }
            continue;
        }
        let Some(m) = kv.captures(line) else {
            continue;
        };
        let key = &m[1];
        let mut val = m[2].to_owned();
        if table == "features" {
            while val.trim_start().starts_with('[')
                && !val.split('#').next().unwrap_or("").contains(']')
            {
                val.push(' ');
                val.push_str(lines.next().unwrap_or("]"));
            }
            out.features.insert(
                key.to_owned(),
                strings(&re(r"#[^\n]*").replace_all(&val, "")),
            );
            continue;
        }
        let isdep = table == "dependencies"
            || (table.starts_with("target.") && table.ends_with(".dependencies"));
        if isdep && val.trim_start().starts_with('{') {
            while !val.split('#').next().unwrap_or("").contains('}') {
                val.push(' ');
                val.push_str(lines.next().unwrap_or("}"));
            }
        }
        if isdep {
            if let Some(pm) = re(r#"\bpath\s*=\s*"([^"]+)""#).captures(&val) {
                out.edges.push(dep_edge(pm[1].to_owned(), &val));
            }
        } else if table.starts_with("dependencies.") || table.contains(".dependencies.") {
            if key == "path" {
                if let Some(p) = strings(&val).first() {
                    out.edges.push((p.clone(), vec![], true, false));
                    edge_index = Some(out.edges.len() - 1);
                }
            } else if let Some(i) = edge_index {
                match key {
                    "features" => out.edges[i].1 = strings(&val),
                    "default-features" => out.edges[i].2 = !val.contains("false"),
                    "optional" => out.edges[i].3 = val.contains("true"),
                    _ => {}
                }
            }
        } else if table == "package" && key == "name" {
            out.name = strings(&val).first().cloned();
        } else if table == "lib" {
            if let Some(s) = strings(&val).first() {
                out.lib.insert(key.to_owned(), s.clone());
            }
            if key == "crate-type" {
                out.lib.insert("@crate-type".into(), val);
            }
        } else if table == "bin" {
            if let (Some(b), Some(s)) = (out.bins.last_mut(), strings(&val).first()) {
                b.insert(key.to_owned(), s.clone());
            }
        }
    }
    out
}
pub fn crates(c: &Corpus) -> BTreeMap<String, Crate> {
    let files: BTreeSet<_> = c.files.iter().map(|f| normalize(&f.path)).collect();
    let mut dirs = BTreeMap::new();
    let mut edges = BTreeMap::new();
    for f in c
        .sources("link-manifests")
        .filter(|f| f.path.file_name().is_some_and(|s| s == "Cargo.toml"))
    {
        let m = manifest(&f.text);
        let Some(name) = m.name else {
            continue;
        };
        let dir = f
            .path
            .parent()
            .unwrap_or(std::path::Path::new(""))
            .to_path_buf();
        let mut roots = Vec::new();
        if let Some(p) = m.lib.get("path") {
            roots.push(normalize(&dir.join(p)));
        } else if files.contains(&dir.join("src/lib.rs")) {
            roots.push(dir.join("src/lib.rs"));
        }
        for b in &m.bins {
            if let Some(p) = b.get("path") {
                roots.push(normalize(&dir.join(p)));
            }
        }
        if m.bins.is_empty() && files.contains(&dir.join("src/main.rs")) {
            roots.push(dir.join("src/main.rs"));
        }
        roots.retain(|p| files.contains(p));
        let staticlib = m
            .lib
            .get("@crate-type")
            .is_some_and(|s| s.contains("staticlib") || s.contains("cdylib"));
        edges.insert(dir.clone(), m.edges);
        dirs.insert(
            dir.clone(),
            Crate {
                name,
                dir,
                roots,
                features: m.features,
                staticlib,
                ..Default::default()
            },
        );
    }
    let names: BTreeMap<_, _> = dirs
        .iter()
        .map(|(d, c)| (d.clone(), c.name.clone()))
        .collect();
    for (d, es) in edges {
        let kr = dirs.get_mut(&d).unwrap();
        for (path, fs, df, opt) in es {
            if let Some(n) = names.get(&normalize(&d.join(path))) {
                kr.deps.push(n.clone());
                kr.edges.insert(n.clone(), (fs, df));
                if opt {
                    kr.optional.insert(n.clone());
                }
            }
        }
    }
    dirs.into_values().map(|c| (c.name.clone(), c)).collect()
}
pub type Features = BTreeMap<String, BTreeSet<String>>;
pub fn resolve_features(
    all: &BTreeMap<String, Crate>,
    roots: &BTreeMap<String, (BTreeSet<String>, bool)>,
    maybe: &BTreeSet<String>,
) -> (Features, Features) {
    struct Work<'a> {
        all: &'a BTreeMap<String, Crate>,
        maybe: &'a BTreeSet<String>,
        on: Features,
        unknown: Features,
        defaults: BTreeSet<String>,
        todo: Vec<(String, String, bool)>,
    }
    impl Work<'_> {
        fn enable(&mut self, c: &str, fs: &[String], df: bool, certain: bool) {
            let Some(kr) = self.all.get(c) else {
                return;
            };
            let first = !self.on.contains_key(c);
            self.on.entry(c.into()).or_default();
            self.unknown.entry(c.into()).or_default();
            if df && self.defaults.insert(c.into()) {
                self.todo.push((c.into(), "default".into(), certain));
            }
            for f in fs {
                self.todo.push((c.into(), f.clone(), certain));
            }
            if first {
                for (dep, (fs, df)) in &kr.edges {
                    if !kr.optional.contains(dep) {
                        self.enable(dep, fs, *df, certain);
                    }
                }
            }
            for f in kr.features.keys() {
                if self.maybe.contains(f) && !self.on[c].contains(f) {
                    self.todo.push((c.into(), f.clone(), false));
                }
            }
        }
    }
    let mut w = Work {
        all,
        maybe,
        on: BTreeMap::new(),
        unknown: BTreeMap::new(),
        defaults: BTreeSet::new(),
        todo: vec![],
    };
    for (c, (fs, df)) in roots {
        w.enable(c, &fs.iter().cloned().collect::<Vec<_>>(), *df, true);
    }
    let mut seen = BTreeSet::new();
    while let Some((c, f, certain)) = w.todo.pop() {
        if !seen.insert((c.clone(), f.clone(), certain)) {
            continue;
        }
        (if certain { &mut w.on } else { &mut w.unknown })
            .entry(c.clone())
            .or_default()
            .insert(f.clone());
        for sub in &all[&c].features.get(&f).cloned().unwrap_or_default() {
            if sub.starts_with("dep:")
                || (all[&c].optional.contains(sub) && !all[&c].features.contains_key(sub))
            {
                let dep = sub.strip_prefix("dep:").unwrap_or(sub);
                if let Some((fs, df)) = all[&c].edges.get(dep) {
                    w.enable(dep, fs, *df, certain);
                }
                continue;
            }
            if let Some((dep, df)) = sub.split_once('/') {
                let weak = dep.ends_with('?');
                let dep = dep.trim_end_matches('?');
                if all.contains_key(dep) && (!weak || w.on.contains_key(dep)) {
                    if let Some((fs, defaults)) =
                        all[&c].edges.get(dep).filter(|_| !w.on.contains_key(dep))
                    {
                        w.enable(dep, fs, *defaults, certain);
                    } else {
                        w.enable(dep, &[], false, certain);
                    }
                    w.todo.push((dep.into(), df.into(), certain));
                }
            } else if all[&c].features.contains_key(sub) {
                w.todo.push((c.clone(), sub.clone(), certain));
            }
        }
    }
    for (c, fs) in &mut w.unknown {
        fs.retain(|f| !w.on[c].contains(f));
    }
    (w.on, w.unknown)
}
#[derive(Clone, Debug)]
pub struct Def {
    pub name: String,
    pub kind: String,
    pub krate: String,
    pub path: PathBuf,
    pub line: usize,
    pub cond: Cond,
    pub abi: String,
}
impl Def {
    pub fn link_name(&self, arch: &str) -> String {
        if arch != "x86" {
            return self.name.clone();
        }
        if self.kind == "asm" {
            if let Some(m) = re(r"^([_@]\w+)@\d+$").captures(&self.name) {
                return format!("{}@", &m[1]);
            }
            return self.name.clone();
        }
        match self.abi.as_str() {
            "fastcall" => format!("@{}@", self.name),
            "stdcall" | "system" => format!("_{}@", self.name),
            _ => format!("_{}", self.name),
        }
    }
    pub fn base(&self) -> String {
        if self.kind == "asm" {
            if let Some(m) = re(r"^[_@](\w+?)(?:@\d+)?$").captures(&self.name) {
                return m[1].to_owned();
            }
        }
        self.name.clone()
    }
    pub fn loc(&self) -> String {
        format!("{}:{}", self.path.display(), self.line)
    }
}
// Keep quote delimiters and line breaks while masking the exact legacy lexical
// forms; unlike the general Rust lexer this intentionally does not nest comments.
fn mask(text: &str) -> String {
    let lex = re(
        r#"(?s)//[^\n]*|/\*.*?\*/|\bb?r(?P<h>\#*)"|b?"(?:\\.|[^"\\])*"|'(?:\\.[^']*|[^\\'\n])'"#,
    );
    let blank = |s: &str| {
        s.chars()
            .map(|c| if c == '\n' { '\n' } else { ' ' })
            .collect::<String>()
    };
    let mut out = String::new();
    let mut pos = 0;
    while let Some(m) = lex.captures(&text[pos..]) {
        let whole = m.get(0).unwrap();
        let a = pos + whole.start();
        let b = pos + whole.end();
        out.push_str(&text[pos..a]);
        if let Some(h) = m.name("h") {
            let close = format!("\"{}", h.as_str());
            let end = text[b..].find(&close).map(|i| b + i).unwrap_or(text.len());
            out.push_str(&text[a..b]);
            out.push_str(&blank(&text[b..end]));
            out.push_str(&close);
            pos = (end + close.len()).min(text.len());
        } else {
            let tok = &text[a..b];
            if tok.starts_with('/') {
                out.push_str(&blank(tok));
            } else {
                let q = tok.find(tok.chars().last().unwrap()).unwrap();
                out.push_str(&tok[..q + 1]);
                out.push_str(&blank(&tok[q + 1..tok.len() - 1]));
                out.push_str(&tok[tok.len() - 1..]);
            }
            pos = b;
        }
    }
    out.push_str(&text[pos..]);
    out
}
#[derive(Clone)]
struct Segment {
    raw: String,
    masked: String,
    line: usize,
}
fn segments(raw: &str) -> Vec<Segment> {
    let masked = mask(raw);
    let mut out = vec![];
    for (n, (r, m)) in raw.split('\n').zip(masked.split('\n')).enumerate() {
        if !m.contains('#')
            && (m.matches(';').count() == 0
                || (m.matches(';').count() == 1 && m.trim_end().ends_with(';')))
        {
            out.push(Segment {
                raw: r.into(),
                masked: m.into(),
                line: n + 1,
            });
            continue;
        }
        // Character indices preserve matching raw/blanked Unicode columns.
        let rr: Vec<_> = r.chars().collect();
        let mm: Vec<_> = m.chars().collect();
        let (mut depth, mut attr, mut last) = (0i32, false, 0usize);
        for (i, ch) in mm.iter().enumerate() {
            if *ch == '#'
                && depth == 0
                && (mm.get(i + 1) == Some(&'[')
                    || (mm.get(i + 1) == Some(&'!') && mm.get(i + 2) == Some(&'[')))
            {
                attr = true;
            }
            let mut cut = false;
            match ch {
                '(' | '[' => depth += 1,
                ')' | ']' => {
                    depth -= 1;
                    if depth == 0 && attr && *ch == ']' {
                        attr = false;
                        cut = mm[i + 1..].iter().any(|c| !c.is_whitespace());
                    }
                }
                ';' if depth == 0 => cut = mm[i + 1..].iter().any(|c| !c.is_whitespace()),
                _ => {}
            }
            if cut {
                out.push(Segment {
                    raw: rr[last..i + 1].iter().collect(),
                    masked: mm[last..i + 1].iter().collect(),
                    line: n + 1,
                });
                last = i + 1;
            }
        }
        out.push(Segment {
            raw: rr[last..].iter().collect(),
            masked: mm[last..].iter().collect(),
            line: n + 1,
        });
    }
    out
}
fn attr_span(ss: &[Segment], i: usize) -> (String, usize) {
    let (mut out, mut d, mut j) = (String::new(), 0i32, i);
    while j < ss.len() {
        out.push_str(&ss[j].raw);
        out.push(' ');
        d += ss[j].raw.matches('[').count() as i32 - ss[j].raw.matches(']').count() as i32;
        if d <= 0 {
            break;
        }
        j += 1;
    }
    (out, j)
}
fn close(ss: &[Segment], i: usize, paren: bool) -> usize {
    let (mut depth, mut started) = (0i32, false);
    for (j, s) in ss.iter().enumerate().skip(i) {
        for ch in s.masked.chars() {
            if ch == '{' || (paren && matches!(ch, '(' | '[')) {
                depth += 1;
                started = true;
            } else if ch == '}' || (paren && matches!(ch, ')' | ']')) {
                depth -= 1;
                if started && depth == 0 {
                    return j;
                }
            }
        }
        if !paren && !started && s.masked.trim_end().ends_with(';') {
            return j;
        }
    }
    ss.len().saturating_sub(1)
}
struct Scanner<'a> {
    kr: &'a Crate,
    files: &'a BTreeMap<PathBuf, &'a crate::SourceFile>,
    seen: BTreeSet<PathBuf>,
    defs: Vec<Def>,
}
impl Scanner<'_> {
    fn file(&mut self, path: PathBuf, cond: Cond, mod_rs: bool) {
        let path = normalize(&path);
        if self.seen.contains(&path) {
            return;
        }
        let Some(file) = self.files.get(&path) else {
            return;
        };
        self.seen.insert(path.clone());
        if !re(r"no_mangle|export_name|global_asm|\bmod\s+\w+\s*;|include!|path\s*=")
            .is_match(&file.text)
        {
            return;
        }
        let ss = segments(&file.text);
        let parent = path.parent().unwrap_or(std::path::Path::new(""));
        let child = if mod_rs {
            parent.to_path_buf()
        } else {
            parent.join(path.file_stem().unwrap())
        };
        self.block(&path, &ss, 0, ss.len(), cond, child, true);
    }
    fn block(
        &mut self,
        path: &PathBuf,
        ss: &[Segment],
        start: usize,
        end: usize,
        mut cond: Cond,
        child: PathBuf,
        top: bool,
    ) {
        let (mut pending, mut mangle, mut export, mut path_attr, mut alts) = (
            Vec::<String>::new(),
            false,
            None::<String>,
            None::<String>,
            Vec::<(String, String)>::new(),
        );
        let mut i = start;
        let module = re(r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(?:r#)?(\w+)\s*(;|\{)");
        let fun = re(
            r#"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:(?:const|unsafe|async|safe)\s+)*(?:extern\s*(?:"[^"]*")?\s*)?fn\s+(\w+)"#,
        );
        let stat = re(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?static\s+(?:mut\s+)?(\w+)");
        while i < end {
            let ml = &ss[i].masked;
            let s = ml.trim();
            if s.is_empty() {
                i += 1;
                continue;
            }
            if top && s.starts_with("#![") {
                let (txt, j) = attr_span(ss, i);
                if let Some(m) =
                    re(r"(?s)^\s*#!\s*\[\s*cfg\s*\((.*)\)\s*\]\s*(?://.*)?$").captures(&txt)
                {
                    cond.preds.push(m[1].into());
                }
                i = j + 1;
                continue;
            }
            if re(r"^\s*#\s*\[").is_match(ml) {
                let (txt, j) = attr_span(ss, i);
                if let Some(m) =
                    re(r"(?s)^\s*#\s*\[\s*cfg\s*\((.*)\)\s*\]\s*(?://.*)?$").captures(&txt)
                {
                    pending.push(m[1].into());
                } else if re(r"^\s*#\s*\[\s*(?:unsafe\s*\(\s*)?no_mangle\b").is_match(&txt) {
                    mangle = true;
                } else if let Some(m) =
                    re(r#"^\s*#\s*\[\s*(?:unsafe\s*\(\s*)?export_name\s*=\s*"([^"]+)""#)
                        .captures(&txt)
                {
                    export = Some(m[1].into());
                } else if let Some(m) =
                    re(r#"^\s*#\s*\[\s*path\s*=\s*"([^"]+)"\s*\]"#).captures(&txt)
                {
                    path_attr = Some(m[1].into());
                } else if let Some(m) =
                    re(r#"(?s)^\s*#\s*\[\s*cfg_attr\s*\((.*),\s*path\s*=\s*"([^"]+)"\s*\)\s*\]"#)
                        .captures(&txt)
                {
                    alts.push((m[1].into(), m[2].into()));
                }
                i = j + 1;
                continue;
            }
            let item = cond.and(pending.iter().cloned());
            let mut next = i + 1;
            let parent = path.parent().unwrap_or(std::path::Path::new(""));
            if let Some(m) = module.captures(ml) {
                let name = &m[1];
                if &m[2] == ";" {
                    for (p, alt) in &alts {
                        self.file(parent.join(alt), item.and([p.clone()]), true);
                    }
                    if alts.is_empty() {
                        if let Some(p) = &path_attr {
                            let base = if top || child == parent {
                                parent.to_path_buf()
                            } else {
                                child.clone()
                            };
                            self.file(base.join(p), item, true);
                        } else {
                            let a = child.join(format!("{name}.rs"));
                            let exists = self.files.contains_key(&normalize(&a));
                            self.file(
                                if exists {
                                    a
                                } else {
                                    child.join(name).join("mod.rs")
                                },
                                item,
                                !exists,
                            );
                        }
                    }
                } else {
                    let j = close(ss, i, false);
                    self.block(path, ss, i + 1, j, item, child.join(name), false);
                    next = j + 1;
                }
            } else if let Some(m) =
                re(r#"^\s*include\s*!\s*\(\s*"([^"]+)"\s*\)"#).captures(&ss[i].raw)
            {
                self.file(parent.join(&m[1]), item, false);
            } else if re(r"^\s*(?:(?:core|std)\s*::\s*arch\s*::\s*)?global_asm\s*!\s*[\(\{\[]")
                .is_match(ml)
            {
                let j = close(ss, i, true);
                let mut body = ss[i..=j]
                    .iter()
                    .map(|s| s.raw.clone())
                    .collect::<Vec<_>>()
                    .join("\n");
                let incs: Vec<_> = re(r#"include_str\s*!\s*\(\s*"([^"]+)"\s*\)"#)
                    .captures_iter(&body)
                    .map(|m| m[1].to_owned())
                    .collect();
                for inc in incs {
                    if let Some(f) = self.files.get(&normalize(&parent.join(inc))) {
                        body.push('\n');
                        body.push_str(&f.text);
                    }
                }
                for m in re(r#"\.glob(?:a)?l\s+"?([@\w$.?]+)"?"#).captures_iter(&body) {
                    self.defs.push(Def {
                        name: m[1].into(),
                        kind: "asm".into(),
                        krate: self.kr.name.clone(),
                        path: path.clone(),
                        line: ss[i].line,
                        cond: item.namespaced(&self.kr.name),
                        abi: String::new(),
                    });
                }
                next = j + 1;
            } else if mangle || export.is_some() {
                if let Some(m) = fun.captures(ml).or_else(|| stat.captures(ml)) {
                    let abi = if fun.is_match(ml) {
                        re(r#"\bextern\s*"(\w+)""#)
                            .captures(&ss[i].raw)
                            .map(|m| m[1].to_owned())
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };
                    self.defs.push(Def {
                        name: export.clone().unwrap_or_else(|| m[1].to_owned()),
                        kind: "rust".into(),
                        krate: self.kr.name.clone(),
                        path: path.clone(),
                        line: ss[i].line,
                        cond: item.namespaced(&self.kr.name),
                        abi,
                    });
                }
            }
            if next == i + 1 && ml.contains('{') && !module.is_match(ml) {
                next = close(ss, i, false) + 1;
            }
            pending.clear();
            mangle = false;
            export = None;
            path_attr = None;
            alts.clear();
            i = next;
        }
    }
}
pub fn crate_defs(c: &Corpus, kr: &Crate) -> Vec<Def> {
    cached_scan(c, kr).0.clone()
}
pub fn crate_files(c: &Corpus, kr: &Crate) -> BTreeSet<PathBuf> {
    cached_scan(c, kr).1.clone()
}
fn cached_scan(c: &Corpus, kr: &Crate) -> std::sync::Arc<(Vec<Def>, BTreeSet<PathBuf>)> {
    // Definitions carry cfg predicates, so all architectures and link units can
    // share this source scan. Only the name and roots affect Scanner's result.
    let key = (kr.name.clone(), kr.roots.clone());
    let mut scans = c.link_scans.lock().expect("link scan cache poisoned");
    scans
        .entry(key)
        .or_insert_with(|| std::sync::Arc::new(scan_crate(c, kr)))
        .clone()
}
fn scan_crate(c: &Corpus, kr: &Crate) -> (Vec<Def>, BTreeSet<PathBuf>) {
    let files = c.files.iter().map(|f| (normalize(&f.path), f)).collect();
    let mut scan = Scanner {
        kr,
        files: &files,
        seen: BTreeSet::new(),
        defs: vec![],
    };
    for root in &kr.roots {
        scan.file(root.clone(), Cond::default(), true);
    }
    (scan.defs, scan.seen)
}
pub fn unknown_build_cfgs(c: &Corpus, all: &BTreeMap<String, Crate>) -> BTreeMap<String, String> {
    let files: BTreeMap<_, _> = c.files.iter().map(|f| (&f.path, f)).collect();
    let mut out = BTreeMap::new();
    for kr in all.values() {
        let path = kr.dir.join("build.rs");
        if let Some(f) = files.get(&path) {
            for m in re(r"rustc-cfg=(\w+)").captures_iter(&f.text) {
                if !build_cfgs(&kr.name).iter().any(|(n, _)| *n == &m[1]) {
                    out.insert(m[1].into(), path.display().to_string());
                }
            }
        }
    }
    out
}
pub fn closure(roots: &[String], all: &BTreeMap<String, Crate>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut todo = roots.to_vec();
    while let Some(n) = todo.pop() {
        if !seen.insert(n.clone()) {
            continue;
        }
        if let Some(c) = all.get(&n) {
            todo.extend(c.deps.clone());
        } else {
            seen.remove(&n);
        }
    }
    seen.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceFile;
    #[test]
    fn cfg_conjunctions_pin_arch_and_namespace_crate_features() {
        let a = Cond {
            preds: vec![r#"all(target_arch="x86", feature="smp")"#.into()],
        }
        .namespaced("a");
        let b = Cond {
            preds: vec![r#"not(feature="smp")"#.into()],
        }
        .namespaced("a");
        assert!(!a.satisfiable_with(&b, "x86", &BTreeMap::new()));
        assert!(a.satisfiable_with(&b.namespaced("b"), "x86", &BTreeMap::new()));
        assert!(!a.satisfiable_with(&Cond::default(), "x86_64", &BTreeMap::new()));
        let pic = Cond {
            preds: vec![namespace_features("hal_pic", "hal-rust")],
        };
        let apic = Cond {
            preds: vec![namespace_features("hal_apic", "hal-rust")],
        };
        for arch in ARCHES {
            assert!(!pic.satisfiable_with(&apic, arch, &BTreeMap::new()));
        }
        assert!(!a.satisfiable_with(
            &Cond::default(),
            "x86",
            &BTreeMap::from([(feature_atom("a", "smp"), false)])
        ));
    }
    #[test]
    fn reachable_definitions_keep_cfg_paths_asm_and_decorations() {
        let items = [
            (
                "Cargo.toml",
                "[package]\nname=\"fixture\"\n[lib]\ncrate-type=[\"staticlib\"]\n",
            ),
            (
                "src/lib.rs",
                r##"#![cfg(windows)]
#[cfg(target_arch="x86")]
#[path="alternate.rs"] mod selected;
mod normal;
#[cfg(feature="smp")] mod inline {
#[no_mangle] pub extern "system" fn Inline() {}
}
include!("included.rs");
core::arch::global_asm!(include_str!("labels.s"));
fn decoy(){ let s = "#[no_mangle] fn Fake() {}"; }
"##,
            ),
            (
                "src/alternate.rs",
                "#[no_mangle] pub extern \"fastcall\" fn Selected() {}\n",
            ),
            (
                "src/normal.rs",
                "#[export_name=\"Renamed\"] pub static VALUE:u32=0;\n",
            ),
            (
                "src/included.rs",
                "#[no_mangle] pub extern \"C\" fn Included() {}\n",
            ),
            ("src/labels.s", ".globl _Asm@12\n"),
            ("src/orphan.rs", "#[no_mangle] fn Orphan() {}\n"),
        ];
        let c = Corpus::new(
            ".".into(),
            items
                .into_iter()
                .map(|(p, s)| SourceFile::new(p.into(), s.into()))
                .collect(),
        );
        let all = crates(&c);
        let k = &all["fixture"];
        assert!(k.staticlib);
        let defs = crate_defs(&c, k);
        let names: BTreeSet<_> = defs.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(
            names,
            BTreeSet::from(["Selected", "Renamed", "Inline", "Included", "_Asm@12"])
        );
        let selected = defs.iter().find(|d| d.name == "Selected").unwrap();
        assert_eq!(selected.link_name("x86"), "@Selected@");
        assert_eq!(selected.cond.arches(), BTreeSet::from(["x86".into()]));
        let asm = defs.iter().find(|d| d.kind == "asm").unwrap();
        assert_eq!(asm.base(), "Asm");
        assert_eq!(asm.link_name("x86"), "_Asm@");
        assert!(!crate_files(&c, k).contains(&PathBuf::from("src/orphan.rs")));
    }
    #[test]
    fn cargo_features_unify_optional_edges_and_unknown_requests() {
        let items = [
            (
                "a/Cargo.toml",
                r#"[package]
name="a"
[dependencies]
b={path="../b", optional=true, default-features=false}
[features]
default=["activate"]
activate=["dep:b", "b/fast"]
maybe=[]
"#,
            ),
            (
                "b/Cargo.toml",
                "[package]\nname=\"b\"\n[features]\nfast=[]\n",
            ),
            ("a/src/lib.rs", ""),
            ("b/src/lib.rs", ""),
        ];
        let c = Corpus::new(
            ".".into(),
            items
                .into_iter()
                .map(|(p, s)| SourceFile::new(p.into(), s.into()))
                .collect(),
        );
        let all = crates(&c);
        let (on, unknown) = resolve_features(
            &all,
            &BTreeMap::from([("a".into(), (BTreeSet::new(), true))]),
            &BTreeSet::from(["maybe".into()]),
        );
        assert!(on["b"].contains("fast"));
        assert!(!on["b"].contains("default"));
        assert!(unknown["a"].contains("maybe"));
        assert!(!on["a"].contains("maybe"));
    }
}
