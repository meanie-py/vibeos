//! CMake source ownership for the remaining C-to-Rust comparison rules.
use crate::{modules::normalize, Corpus, SourceFile, SourceSet};
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use std::path::{Path, PathBuf};
fn re(s: &str) -> regex::Regex {
    crate::regexes::cached(s).expect("CMake source model pattern")
}
pub const SOURCE_SETS: &[SourceSet] = &[
    SourceSet::tree("module-meta", &[""], &["cmake"], &["CMakeLists.txt"])
        .with_pruning(&["target", ".git", "build-*"]),
    SourceSet::tree("module-c", &[""], &["c"], &[]).with_pruning(&["target", ".git", "build-*"]),
    SourceSet::tree("module-rust", &["sdk/lib/rust"], &["rs"], &["Cargo.toml"]).with_inputs(),
];
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ModuleInputs {
    pub c_files: Vec<PathBuf>,
    pub rust_files: Vec<PathBuf>,
    pub glue: Vec<PathBuf>,
}
pub struct ModuleModel {
    pub modules: BTreeMap<PathBuf, ModuleInputs>,
}
struct Snapshot<'a> {
    files: BTreeMap<PathBuf, &'a SourceFile>,
}
fn strip_comments(text: &str) -> String {
    text.lines()
        .map(|l| {
            let mut quoted = false;
            for (i, c) in l.char_indices() {
                if c == '"' {
                    quoted = !quoted;
                }
                if c == '#' && !quoted {
                    return l[..i].to_owned();
                }
            }
            l.to_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
}
fn commands(text: &str) -> Vec<(String, String)> {
    let pat = re(r"([A-Za-z_]\w*)\s*\(");
    let mut out = vec![];
    let mut pos = 0;
    while let Some(m) = pat.captures(&text[pos..]) {
        let whole = m.get(0).unwrap();
        let begin = pos + whole.end();
        let (mut depth, mut quoted, mut end) = (1i32, false, text.len());
        for (n, c) in text[begin..].char_indices() {
            match c {
                '"' => quoted = !quoted,
                '(' if !quoted => depth += 1,
                ')' if !quoted => {
                    depth -= 1;
                    if depth == 0 {
                        end = begin + n;
                        break;
                    }
                }
                _ => {}
            }
        }
        out.push((m[1].to_ascii_lowercase(), text[begin..end].to_owned()));
        pos = (end + 1).min(text.len());
    }
    out
}
fn expand(value: &str, vars: &BTreeMap<String, Vec<String>>) -> Vec<String> {
    let mut value = value.to_owned();
    for _ in 0..8 {
        let Some(m) = re(r"\$\{(\w+)\}").captures(&value) else {
            return if value.trim_matches('"').is_empty() {
                vec![]
            } else {
                vec![value.trim_matches('"').into()]
            };
        };
        let Some(repl) = vars.get(&m[1]) else {
            return vec![value.trim_matches('"').into()];
        };
        let whole = m.get(0).unwrap();
        let (head, tail) = (&value[..whole.start()], &value[whole.end()..]);
        if repl.len() == 1 {
            value = format!("{head}{}{tail}", repl[0]);
        } else {
            return repl
                .iter()
                .flat_map(|r| expand(&format!("{head}{r}{tail}"), vars))
                .collect();
        }
    }
    vec![value.trim_matches('"').into()]
}
fn unroll(text: &str) -> String {
    let pat =
        re(r"(?ms)^[ \t]*foreach\s*\(\s*(\w+)\s+([^)]*)\)(.*?)^[ \t]*endforeach\s*\(\s*\w*\s*\)");
    pat.replace_all(text, |m: &regex::Captures| {
        let (var, spec, body) = (&m[1], m[2].split_whitespace().collect::<Vec<_>>(), &m[3]);
        if !re(r"\badd_(?:library|executable)\s*\(").is_match(body)
            || re(r"\bforeach\s*\(").is_match(body)
            || spec.first().is_some_and(|s| matches!(*s, "IN" | "RANGE"))
        {
            return m[0].to_owned();
        }
        let mut items = vec![];
        for word in spec {
            if let Some(r) = re(r"^\$\{(\w+)\}$").captures(word) {
                let mut listed = vec![];
                let p = re(&format!(
                    r"(?m)^[ \t]*(?:set\s*\(\s*{}\b|list\s*\(\s*APPEND\s+{}\b)([^)]*)\)",
                    regex::escape(&r[1]),
                    regex::escape(&r[1])
                ));
                for lm in p.captures_iter(&text[..m.get(0).unwrap().start()]) {
                    if lm[0].trim_start().starts_with("set") {
                        listed.clear();
                    }
                    let ws = lm[1].split_whitespace().collect::<Vec<_>>();
                    if ws.iter().any(|w| w.contains('$') || w.contains('"')) {
                        return m[0].to_owned();
                    }
                    listed.extend(ws.into_iter().map(str::to_owned));
                }
                if listed.is_empty() {
                    return m[0].to_owned();
                }
                items.extend(listed);
            } else if word.contains('$') {
                return m[0].to_owned();
            } else {
                items.push(word.into());
            }
        }
        items
            .into_iter()
            .map(|v| body.replace(&format!("${{{var}}}"), &v))
            .collect::<String>()
    })
    .into_owned()
}
impl Snapshot<'_> {
    fn text(&self, path: &Path) -> Option<&str> {
        self.files.get(&normalize(path)).map(|f| f.text.as_str())
    }
    fn spliced(&self, path: &Path, seen: &HashSet<PathBuf>) -> String {
        let path = normalize(path);
        if seen.contains(&path) {
            return String::new();
        }
        let Some(raw) = self.text(&path) else {
            return String::new();
        };
        let mut seen = seen.clone();
        seen.insert(path.clone());
        re(r#"(?m)^[ \t]*include\s*\(\s*([^)${}\s"]+\.cmake)\s*\)"#)
            .replace_all(raw, |m: &regex::Captures| {
                let inc = normalize(&path.parent().unwrap_or(Path::new("")).join(&m[1]));
                if self.files.contains_key(&inc) {
                    self.spliced(&inc, &seen)
                } else {
                    m[0].into()
                }
            })
            .into_owned()
    }
    fn cargo_crates(&self, dir: &Path) -> Vec<PathBuf> {
        let mut texts = vec![];
        let mut queue = VecDeque::from([dir.join("CMakeLists.txt")]);
        let mut seen = BTreeSet::new();
        while let Some(p) = queue.pop_front() {
            let p = normalize(&p);
            if !seen.insert(p.clone()) {
                continue;
            }
            let Some(raw) = self.text(&p) else {
                continue;
            };
            let text = raw
                .lines()
                .map(|l| l.split('#').next().unwrap_or(""))
                .collect::<Vec<_>>()
                .join("\n");
            for m in re(r"include\s*\(\s*([^\s)]+\.cmake)\s*\)").captures_iter(&text) {
                queue.push_back(p.parent().unwrap().join(&m[1]));
            }
            texts.push(text);
        }
        let text = texts.join("\n");
        let mut names: BTreeSet<String> = re(r"rust_crate_sources\([^)]*?\s([\w.-]+)\s*\)")
            .captures_iter(&text)
            .map(|m| m[1].into())
            .collect();
        for m in re(r"(?s)add_cargo_(?:module|kernel_dll)\(\s*([\w.-]+)\s+([\w.-]+)([^)]*)\)")
            .captures_iter(&text)
        {
            names.insert(m[2].into());
            if let Some(b) = re(r"BODY_CRATES?((?:[ \t]+[\w.-]+)+)").captures(&m[3]) {
                names.extend(b[1].split_whitespace().map(str::to_owned));
            }
        }
        for n in names.clone() {
            if let Some(text) = self.text(&PathBuf::from("sdk/lib/rust").join(n).join("build.rs")) {
                names.extend(
                    re(r#"\.rust_lib\(\s*"([\w.-]+)""#)
                        .captures_iter(text)
                        .map(|m| m[1].to_owned()),
                );
            }
        }
        names
            .into_iter()
            .map(|n| PathBuf::from("sdk/lib/rust").join(n))
            .filter(|p| self.files.contains_key(&p.join("Cargo.toml")))
            .collect()
    }
    fn parsed_targets(&self, cml: &Path, arch: &str) -> BTreeMap<String, Vec<PathBuf>> {
        let Some(_) = self.text(cml) else {
            return BTreeMap::new();
        };
        let dir = cml.parent().unwrap_or(Path::new(""));
        let text = unroll(&strip_comments(&self.spliced(cml, &HashSet::new())));
        let mut vars = BTreeMap::from([
            (
                "CMAKE_CURRENT_SOURCE_DIR".into(),
                vec![format!("/{}", dir.display())],
            ),
            ("REACTOS_SOURCE_DIR".into(), vec!["/".into()]),
            ("CMAKE_SOURCE_DIR".into(), vec!["/".into()]),
            ("ARCH".into(), vec![arch.into()]),
        ]);
        let mut conds = Vec::<bool>::new();
        let mut entries = BTreeMap::<String, Vec<String>>::new();
        let mut targets = BTreeMap::<String, Vec<String>>::new();
        for (name, raw) in commands(&text) {
            let ar: Vec<_> = raw.split_whitespace().map(str::to_owned).collect();
            match name.as_str() {
                "if" => {
                    let live = re(r#"^ARCH\s+STREQUAL\s*"?(\w+)"?"#)
                        .captures(raw.trim())
                        .is_none_or(|m| &m[1] == arch);
                    conds.push(live);
                    continue;
                }
                "elseif" | "else" => {
                    if let Some(last) = conds.last_mut() {
                        *last = true;
                    }
                    continue;
                }
                "endif" => {
                    conds.pop();
                    continue;
                }
                _ => {}
            }
            if conds.contains(&false) {
                continue;
            }
            let args: Vec<_> = ar.iter().flat_map(|a| expand(a, &vars)).collect();
            match name.as_str() {
                "set" if !args.is_empty() => {
                    vars.insert(ar[0].clone(), args[1..].to_vec());
                }
                "list" if ar.len() >= 2 && ar[0].eq_ignore_ascii_case("APPEND") => {
                    let vals: Vec<_> = ar[2..].iter().flat_map(|a| expand(a, &vars)).collect();
                    vars.entry(ar[1].clone()).or_default().extend(vals.clone());
                    entries.entry(ar[1].clone()).or_default().extend(vals);
                }
                "add_library" | "add_executable" if !args.is_empty() => {
                    targets.insert(
                        args[0].clone(),
                        ar[1..]
                            .iter()
                            .filter(|a| {
                                !matches!(a.as_str(), "MODULE" | "SHARED" | "STATIC" | "WIN32")
                            })
                            .cloned()
                            .collect(),
                    );
                }
                "add_cargo_module" if ar.len() >= 3 => {
                    let source = if ar.iter().any(|a| a == "kernelexe") {
                        vec!["${SOURCE}".into()]
                    } else {
                        vec![]
                    };
                    targets.insert(args[0].clone(), source);
                }
                _ => {}
            }
        }
        targets
            .into_iter()
            .map(|(name, args)| {
                let mut raw = Vec::new();
                for a in args {
                    if let Some(m) = re(r"^\$\{(\w+)\}$").captures(&a) {
                        if let Some(es) = entries.get(&m[1]).or_else(|| vars.get(&m[1])) {
                            raw.extend(es.clone());
                        }
                    } else {
                        raw.push(a);
                    }
                }
                let paths = raw
                    .into_iter()
                    .filter(|a| {
                        a.to_ascii_lowercase().ends_with(".c")
                            && !a.to_ascii_lowercase().contains("cmake_current_binary_dir")
                            && !a.to_ascii_lowercase().starts_with("${cmake_binary_dir}")
                    })
                    .map(|a| {
                        let p = normalize(&dir.join(a));
                        p.strip_prefix("/").unwrap_or(&p).to_path_buf()
                    })
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect();
                (name, paths)
            })
            .collect()
    }
    fn compiled(
        &self,
        dir: &Path,
        index: &BTreeMap<String, PathBuf>,
        parsed: &mut BTreeMap<PathBuf, BTreeMap<String, Vec<PathBuf>>>,
    ) -> Vec<PathBuf> {
        let cml = dir.join("CMakeLists.txt");
        let Some(raw) = self.text(&cml) else {
            return vec![];
        };
        let mut roots: Vec<String> =
            re(r"(?m)^[ \t]*add_(?:library|executable)\([ \t]*([A-Za-z_][\w.-]*)")
                .captures_iter(raw)
                .map(|m| m[1].into())
                .collect();
        if roots.is_empty() {
            let ts = parsed
                .entry(cml.clone())
                .or_insert_with(|| self.parsed_targets(&cml, "i386"));
            {
                if ts.len() == 1 {
                    roots.extend(ts.keys().cloned());
                } else if let Some(n) = dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .filter(|n| ts.contains_key(*n))
                {
                    roots.push(n.into());
                }
            }
        }
        let mut todo: Vec<_> = roots.into_iter().map(|n| (cml.clone(), n)).collect();
        let mut seen = BTreeSet::new();
        let mut out = BTreeSet::new();
        while let Some((path, n)) = todo.pop() {
            if !seen.insert(n.clone()) {
                continue;
            }
            if let Some(paths) = parsed
                .entry(path.clone())
                .or_insert_with(|| self.parsed_targets(&path, "i386"))
                .get(&n)
            {
                out.extend(
                    paths
                        .iter()
                        .filter(|p| self.files.contains_key(*p))
                        .cloned(),
                );
            } else {
                continue;
            }
            let text = self.text(&path).unwrap_or("");
            for m in re(r"(?s)target_link_libraries\(\s*([\w.-]+)\s+([^)]*)\)").captures_iter(text)
            {
                if m[1] != n {
                    continue;
                }
                for line in m[2].lines() {
                    let line = re(r"\$\{[^}]*\}")
                        .replace_all(line.split('#').next().unwrap_or(""), " ")
                        .into_owned();
                    for word in re(r"[A-Za-z_][\w.-]*").find_iter(&line) {
                        if matches!(
                            word.as_str(),
                            "PUBLIC" | "PRIVATE" | "INTERFACE" | "debug" | "optimized" | "general"
                        ) {
                            continue;
                        }
                        if let Some(p) = index.get(word.as_str()) {
                            todo.push((p.clone(), word.as_str().into()));
                        }
                    }
                }
            }
        }
        out.into_iter().collect()
    }
    fn other_target(&self, file: &Path, dir: &Path) -> bool {
        for d in file.ancestors().skip(1) {
            if d == dir || !d.starts_with(dir) {
                break;
            }
            if self.text(&d.join("CMakeLists.txt")).is_some_and(|t| {
                re(r"(?m)^[ \t]*(?:set_module_type|add_library|add_executable|add_cargo_\w+)\(")
                    .is_match(t)
            }) {
                return true;
            }
        }
        false
    }
}
impl ModuleModel {
    pub fn from_corpus(c: &Corpus) -> Self {
        let snap = Snapshot {
            files: c.files.iter().map(|f| (normalize(&f.path), f)).collect(),
        };
        let mut index = BTreeMap::new();
        for f in c
            .ordered_sources("module-meta")
            .into_iter()
            .filter(|f| f.path.file_name().is_some_and(|n| n == "CMakeLists.txt"))
        {
            for m in re(r"(?m)^[ \t]*add_(?:library|executable)\([ \t]*([A-Za-z_][\w.-]*)")
                .captures_iter(&f.text)
            {
                index.entry(m[1].to_owned()).or_insert(f.path.clone());
            }
        }
        let mut parsed = BTreeMap::new();
        let roots = [
            "base",
            "dll",
            "drivers",
            "hal",
            "ntoskrnl",
            "subsystems",
            "win32ss",
            "boot",
        ];
        let mut modules = BTreeMap::new();
        for (p, f) in &snap.files {
            if !p.file_name().is_some_and(|n| n == "CMakeLists.txt")
                || !roots.iter().any(|root| p.starts_with(root))
                || !(f.text.contains("rust_crate_sources(") || f.text.contains("add_cargo_"))
            {
                continue;
            }
            let dir = p.parent().unwrap();
            let pure = f.text.contains("add_cargo_")
                && !re(r"(?m)^[ \t]*add_(?:library|executable)\(").is_match(&f.text);
            let live = if pure {
                vec![]
            } else {
                snap.compiled(dir, &index, &mut parsed)
            };
            let cs = snap
                .files
                .keys()
                .filter(|q| {
                    q.starts_with(dir)
                        && q.extension().is_some_and(|e| e == "c")
                        && !live.contains(q)
                        && !q
                            .file_name()
                            .is_some_and(|n| n.to_string_lossy().starts_with("rust_"))
                        && !snap.other_target(q, dir)
                })
                .cloned()
                .collect();
            let mut rust = vec![];
            for kr in snap.cargo_crates(dir) {
                rust.extend(
                    snap.files
                        .keys()
                        .filter(|q| {
                            q.starts_with(kr.join("src"))
                                && q.extension().is_some_and(|e| e == "rs")
                        })
                        .cloned(),
                );
            }
            let glue = live.into_iter().filter(|p| p.starts_with(dir)).collect();
            modules.insert(
                dir.into(),
                ModuleInputs {
                    c_files: cs,
                    rust_files: rust,
                    glue,
                },
            );
        }
        Self { modules }
    }
}
