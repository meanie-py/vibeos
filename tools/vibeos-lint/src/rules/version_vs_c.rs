use crate::{regexes::cached, twins, Corpus, Rule, SourceSet};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[path = "version_eval.rs"]
mod eval;

pub struct VersionVsC;
const INPUTS: &[SourceSet] = &[
    SourceSet::tree("version-rust", &["sdk/lib/rust"], &["rs"], &[])
        .with_pruning(&["target", ".responses"])
        .with_inputs(),
    SourceSet::tracked("version-c", &["c", "cpp"], &[]).with_inputs(),
    SourceSet::tracked("version-cmake", &["cmake"], &["CMakeLists.txt"]).with_inputs(),
    SourceSet::tree(
        "version-ninja",
        &["build-amd64", "build-i386", "build-arm64"],
        &["ninja"],
        &[],
    )
    .with_inputs(),
    SourceSet::tree("version-sdkddkver", &["sdk/include/psdk"], &["h"], &[]).with_inputs(),
    SourceSet::tree("version-root-cmake", &["."], &[], &["CMakeLists.txt"]).with_inputs(),
];
fn re(p: &str) -> Regex {
    cached(p).expect("version rule regex")
}
fn path_ext(p: &Path, exts: &[&str]) -> bool {
    p.extension()
        .and_then(|x| x.to_str())
        .is_some_and(|x| exts.contains(&x))
}
fn parse_int(s: &str) -> Option<i64> {
    let s = s
        .trim()
        .trim_end_matches(|c: char| matches!(c, 'u' | 'U' | 'l' | 'L'));
    if let Some(h) = s.strip_prefix("0x") {
        i64::from_str_radix(h, 16).ok()
    } else {
        s.parse().ok()
    }
}
fn parse_override(texts: &[(String, String)]) -> Option<(Option<i64>, Option<i64>, bool, String)> {
    let def = re(
        r"(?s)(?:add_definitions|add_compile_definitions|target_compile_definitions)\s*\(([^)]*)\)",
    );
    let rem = re(r"(?s)remove_definitions\s*\(([^)]*)\)");
    let mut w = None;
    let mut n = None;
    let mut removed = false;
    let mut src = String::new();
    for (where_, raw) in texts {
        let text = re(r"(?m)#[^\n]*").replace_all(raw, "");
        for m in def.captures_iter(&text) {
            let arg = &m[1];
            if let Some(v) = re(r"_WIN32_WINNT=(\w+)")
                .captures(arg)
                .and_then(|x| parse_int(&x[1]))
            {
                w = Some(v);
                src = where_.clone()
            }
            if let Some(v) = re(r"NTDDI_VERSION=(\w+)")
                .captures(arg)
                .and_then(|x| parse_int(&x[1]))
            {
                n = Some(v);
                src = where_.clone()
            }
        }
        for m in rem.captures_iter(&text) {
            if m[1].contains("_WIN32_WINNT") {
                removed = true;
                if src.is_empty() {
                    src = where_.clone()
                }
            }
        }
    }
    if w.is_none() && n.is_none() && !removed {
        return None;
    }
    Some((w, n, removed && w.is_none(), src))
}
fn files_in_dir<'a>(c: &'a Corpus, dir: &Path) -> Vec<&'a crate::SourceFile> {
    let mut v: Vec<_> = c
        .sources("version-cmake")
        .filter(|f| {
            f.path.parent() == Some(dir)
                && (f.path.file_name().is_some_and(|n| n == "CMakeLists.txt")
                    || path_ext(&f.path, &["cmake"]))
        })
        .collect();
    v.sort_by_key(|f| {
        (
            if f.path.file_name().is_some_and(|n| n == "CMakeLists.txt") {
                0
            } else {
                1
            },
            f.path.clone(),
        )
    });
    v
}
fn env(w: i64, n: Option<i64>, consts: &BTreeMap<String, i64>) -> eval::Env {
    let nv = n.unwrap_or_else(|| {
        if w == 0x502 {
            *consts.get("NTDDI_WS03SP1").unwrap_or(&0x05020100)
        } else {
            w << 16
        }
    });
    BTreeMap::from([
        ("NTDDI_VERSION".into(), nv),
        ("_WIN32_WINNT".into(), w),
        ("WINVER".into(), w),
    ])
}
fn module_env(
    c: &Corpus,
    path: &Path,
    consts: &BTreeMap<String, i64>,
) -> Option<(eval::Env, String)> {
    let versions = ninja_versions(c);
    if let Some(set) = versions.get(path) {
        if set.len() != 1 {
            return None;
        }
        let &(w, n) = set.iter().next()?;
        return Some((
            env(w, n, consts),
            format!("build.ninja ({})", path.display()),
        ));
    }
    let mut dir = path.parent()?;
    while dir != Path::new("") {
        let current = files_in_dir(c, dir);
        let hist_path = dir.join("CMakeLists.txt");
        let hist = c.version_history().ok()?;
        let hist_override = if hist.ever_versioned_cmake.contains(&hist_path) {
            hist.last_parent_text.get(&hist_path).map(|t| {
                (
                    format!(
                        "{}@{}",
                        hist_path.display(),
                        hist.last_parent_revision
                            .get(&hist_path)
                            .map(String::as_str)
                            .unwrap_or("")
                    ),
                    t.clone(),
                )
            })
        } else {
            None
        };
        let current_text: Vec<_> = current
            .iter()
            .map(|f| (f.path.display().to_string(), f.text.clone()))
            .collect();
        let ov = parse_override(&current_text)
            .or_else(|| hist_override.and_then(|x| parse_override(&[x])));
        if let Some((w, n, removed, where_)) = ov {
            if removed {
                return Some((
                    env(0x600, None, consts),
                    format!("{where_} (removed; sdkddkver.h default)"),
                ));
            }
            return Some((
                env(w.unwrap_or_else(|| (n.unwrap_or(0) >> 16)), n, consts),
                where_,
            ));
        }
        if dir == Path::new(".") {
            break;
        }
        dir = dir.parent().unwrap_or(Path::new(""));
    }
    let prefix = path.parent().unwrap_or(Path::new(""));
    let mut same = Vec::new();
    for (p, vs) in &versions {
        if p.parent() == Some(prefix) {
            same.extend(vs.iter().copied())
        }
    }
    if !same.is_empty() {
        let mut counts = BTreeMap::new();
        for v in &same {
            *counts.entry(*v).or_insert(0usize) += 1
        }
        let (best, count) = counts.into_iter().max_by_key(|(v, n)| (*n, *v))?;
        return Some((
            env(best.0, best.1, consts),
            format!(
                "build.ninja ({}*, {count}/{} objects)",
                prefix.display(),
                same.len()
            ),
        ));
    }
    let root = c
        .sources("version-root-cmake")
        .find(|f| f.path == Path::new("CMakeLists.txt"))?;
    let w = re(r"-D_WIN32_WINNT=(0x[0-9A-Fa-f]+)")
        .captures(&root.text)
        .and_then(|x| parse_int(&x[1]))?;
    Some((env(w, None, consts), "CMakeLists.txt (root default)".into()))
}
fn ninja_versions(c: &Corpus) -> BTreeMap<PathBuf, BTreeSet<(i64, Option<i64>)>> {
    let mut out = BTreeMap::new();
    let build =
        re(r"^build \S+\.(?:c|cpp)\.obj: C(?:XX)?_COMPILER\S* /tmp/vibeos/(\S+?\.(?:c|cpp))\b");
    let mut cur: Option<PathBuf> = None;
    for f in c.sources("version-ninja") {
        for line in f.text.lines() {
            if let Some(m) = build.captures(line) {
                cur = Some(PathBuf::from(&m[1]))
            } else if cur.is_some() && line.starts_with("  DEFINES =") {
                let path = cur.take().unwrap();
                let ws: Vec<_> = re(r"-D_WIN32_WINNT=(\w+)")
                    .captures_iter(line)
                    .map(|m| m[1].to_owned())
                    .collect();
                let ns: Vec<_> = re(r"-DNTDDI_VERSION=(\w+)")
                    .captures_iter(line)
                    .map(|m| m[1].to_owned())
                    .collect();
                let w = ws.last().and_then(|x| parse_int(x)).unwrap_or(0x600);
                let n = ns.last().and_then(|x| parse_int(x));
                out.entry(path)
                    .or_insert_with(std::collections::BTreeSet::new)
                    .insert((w, n));
            }
        }
    }
    out
}
fn consts(c: &Corpus) -> BTreeMap<String, i64> {
    let Some(f) = c
        .sources("version-sdkddkver")
        .find(|f| f.path.ends_with("sdk/include/psdk/sdkddkver.h"))
    else {
        return BTreeMap::new();
    };
    eval::version_constants(&f.text)
}
impl Rule for VersionVsC {
    fn name(&self) -> &'static str {
        "scan_version_vs_c"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn needs_version_history(&self) -> bool {
        true
    }
    fn needs_c_twins(&self) -> bool {
        true
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let history = c.version_history()?;
        let _ = history;
        let constants = consts(c);
        let mut candidates: BTreeMap<
            String,
            Vec<(twins::Function, BTreeMap<String, String>, String)>,
        > = BTreeMap::new();
        for f in c.sources("version-c") {
            if f.path.starts_with("sdk/lib/rust")
                || f.path.starts_with("modules/rostests")
                || !path_ext(&f.path, &["c", "cpp"])
                || !re(r"NTDDI_VERSION|_WIN32_WINNT|__REACTOS__").is_match(&f.text)
                || !re(r"(?m)^[ \t]*#[ \t]*(?:el)?if").is_match(&f.text)
            {
                continue;
            }
            let Some((e, src)) = module_env(c, &f.path, &constants) else {
                continue;
            };
            let blank = twins::blank_c(&f.text);
            let names: HashSet<_> = twins::defined_names(&blank).into_iter().collect();
            if names.is_empty() {
                continue;
            }
            for fun in twins::c_functions(&blank, &f.path, &names) {
                let body = &f.text[fun.body_start..fun.body_start + fun.body.len()];
                if !body.contains('#') {
                    continue;
                }
                let (calls, _) = eval::dead_only_calls(body, &e, &constants);
                if !calls.is_empty() {
                    candidates
                        .entry(fun.name.clone())
                        .or_default()
                        .push((fun, calls, src.clone()))
                }
            }
        }
        let mut out = Vec::new();
        for f in c.sources("version-rust") {
            if f.path
                .file_name()
                .is_some_and(|n| re(r"bindgen_\w+\.rs$").is_match(&n.to_string_lossy()))
            {
                continue;
            }
            if candidates.is_empty() {
                continue;
            }
            let blank = twins::blank_rust(&f.text);
            let funcs = twins::rust_functions(&blank, &(c.root.join(&f.path)));
            let lines: Vec<_> = f.text.split('\n').collect();
            let shared_cfg = c.crate_cfg(&f.path);
            let cfg = eval::CrateCfg {
                on: shared_cfg.on.clone(),
                declared: shared_cfg.declared.clone(),
                emitted: shared_cfg.emitted.clone(),
            };
            let bools: BTreeMap<_, _> =
                re(r"\b(?:const|let)\s+(\w+)\s*:\s*bool\s*=\s*(true|false)\s*;")
                    .captures_iter(&f.text)
                    .map(|m| (m[1].to_owned(), &m[2] == "true"))
                    .collect();
            for fun in funcs {
                let Some(cs) = candidates.get(&fun.name) else {
                    continue;
                };
                let all = c.c_twins().get(&fun.name);
                let same: Vec<_> = all
                    .iter()
                    .filter(|t| {
                        !t.path.starts_with("modules/rostests") && twins::same_module(&f.path, t)
                    })
                    .cloned()
                    .collect();
                if same.is_empty() {
                    continue;
                }
                let best = twins::prefer(&f.path, &same).first().copied();
                let Some(best) = best else { continue };
                let Some((t, calls, src)) = cs
                    .iter()
                    .find(|(t, _, _)| t.path == best.path && t.line == best.line)
                else {
                    continue;
                };
                let _ = t;
                if lines
                    .get(fun.line - 1)
                    .is_some_and(|l| l.contains("nolint: version-vs-c"))
                {
                    continue;
                }
                if eval::item_compiled_out(&lines, fun.line, &cfg) {
                    continue;
                }
                let live = eval::mask_gated(
                    &fun.body,
                    &f.text[fun.body_start..fun.body_start + fun.body.len()],
                    &cfg,
                    &bools,
                );
                let hit: Vec<_> = calls
                    .keys()
                    .filter(|name| {
                        re(&format!(r"\b{}\s*!?\s*\(", regex::escape(name))).is_match(&live)
                    })
                    .cloned()
                    .collect();
                if hit.is_empty() {
                    continue;
                }
                let directive = &calls[&hit[0]];
                out.push(format!("{}: {} [VERSION_VS_C]", f.path.display(), fun.name));
                let _ = (src, directive);
            }
        }
        out.sort();
        Ok(out)
    }
}
