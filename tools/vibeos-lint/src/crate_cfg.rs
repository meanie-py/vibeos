//! Cargo feature facts used by the version/profile comparisons.
use crate::{modules::normalize, Corpus};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CrateCfg {
    pub on: BTreeSet<String>,
    pub declared: BTreeSet<String>,
    pub emitted: BTreeSet<String>,
}
fn re(s: &str) -> regex::Regex {
    crate::regexes::cached(s).expect("Cargo cfg pattern")
}
fn strings(s: &str) -> Vec<String> {
    re(r#""([^"\\]*(?:\\.[^"\\]*)*)"|'([^']*)'"#)
        .captures_iter(s)
        .map(|m| m.get(1).or_else(|| m.get(2)).unwrap().as_str().to_owned())
        .collect()
}
fn strip_comment(s: &str) -> &str {
    let (mut quote, mut escaped) = (None, false);
    for (i, c) in s.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' && quote == Some('"') {
            escaped = true;
            continue;
        }
        if Some(c) == quote {
            quote = None;
        } else if quote.is_none() {
            if matches!(c, '"' | '\'') {
                quote = Some(c);
            } else if c == '#' {
                return &s[..i];
            }
        }
    }
    s
}
#[derive(Default)]
struct Facts {
    name: String,
    features: BTreeMap<String, Vec<String>>,
    requests: Vec<(String, Vec<String>)>,
}
fn facts(text: &str) -> Facts {
    let mut out = Facts::default();
    let mut table = String::new();
    let mut lines = text.lines();
    let mut edge_name = String::new();
    let mut edge_package = None::<String>;
    let mut edge_features = vec![];
    let flush =
        |out: &mut Facts, name: &mut String, pkg: &mut Option<String>, fs: &mut Vec<String>| {
            if !name.is_empty() {
                out.requests.push((
                    pkg.take().unwrap_or_else(|| name.clone()),
                    std::mem::take(fs),
                ));
                name.clear();
            }
        };
    while let Some(raw) = lines.next() {
        let line = strip_comment(raw).trim();
        if let Some(m) = re(r"^\[([^\[\]]+)\]$").captures(line) {
            flush(
                &mut out,
                &mut edge_name,
                &mut edge_package,
                &mut edge_features,
            );
            table = m[1].trim().to_owned();
            if let Some(n) = table
                .strip_prefix("dependencies.")
                .or_else(|| table.strip_prefix("build-dependencies."))
                .or_else(|| {
                    if table.starts_with("target.") {
                        table.split_once(".dependencies.").map(|(_, n)| n)
                    } else {
                        None
                    }
                })
            {
                edge_name = n.trim_matches(['"', '\'']).to_owned();
            }
            continue;
        }
        let Some(m) = re(r#"^([\w."'-]+)\s*=\s*(.*)$"#).captures(line) else {
            continue;
        };
        let key = m[1].trim_matches(['"', '\'']);
        let mut val = m[2].to_owned();
        if val.starts_with('[') || val.starts_with('{') {
            let close = if val.starts_with('[') { ']' } else { '}' };
            while !val.contains(close) {
                let Some(next) = lines.next() else {
                    break;
                };
                val.push('\n');
                val.push_str(strip_comment(next));
            }
        }
        if table == "package" && key == "name" {
            out.name = strings(&val).first().cloned().unwrap_or_default();
        } else if table == "features" {
            out.features.insert(key.into(), strings(&val));
        } else if table == "dependencies"
            || table == "build-dependencies"
            || (table.starts_with("target.") && table.ends_with(".dependencies"))
        {
            let package = re(r#"\bpackage\s*=\s*(?:"([^"]+)"|'([^']+)')"#)
                .captures(&val)
                .map(|m| m.get(1).or_else(|| m.get(2)).unwrap().as_str().to_owned())
                .unwrap_or_else(|| key.into());
            let fs = re(r"(?s)\bfeatures\s*=\s*\[([^\]]*)\]")
                .captures(&val)
                .map(|m| strings(&m[1]))
                .unwrap_or_default();
            out.requests.push((package, fs));
        } else if !edge_name.is_empty() {
            if key == "package" {
                edge_package = strings(&val).first().cloned();
            } else if key == "features" {
                edge_features = strings(&val);
            }
        }
    }
    flush(
        &mut out,
        &mut edge_name,
        &mut edge_package,
        &mut edge_features,
    );
    out
}
pub fn configs(c: &Corpus) -> BTreeMap<PathBuf, CrateCfg> {
    let all: BTreeMap<_, _> = c
        .files
        .iter()
        .filter(|f| f.path.file_name().is_some_and(|n| n == "Cargo.toml"))
        .map(|f| {
            (
                f.path.parent().unwrap_or(Path::new("")).to_path_buf(),
                facts(&f.text),
            )
        })
        .collect();
    let mut requested = BTreeMap::<String, BTreeSet<String>>::new();
    for (dir, f) in &all {
        if dir.parent() == Some(Path::new("sdk/lib/rust")) {
            for (n, fs) in &f.requests {
                requested.entry(n.clone()).or_default().extend(fs.clone());
            }
        }
    }
    all.into_iter()
        .map(|(dir, f)| {
            let declared = f
                .features
                .keys()
                .filter(|s| s.as_str() != "default")
                .cloned()
                .collect();
            let mut todo = f.features.get("default").cloned().unwrap_or_default();
            todo.extend(requested.get(&f.name).into_iter().flatten().cloned());
            let mut on = BTreeSet::new();
            while let Some(feature) = todo.pop() {
                if feature.contains('/')
                    || feature.starts_with("dep:")
                    || !on.insert(feature.clone())
                {
                    continue;
                }
                todo.extend(f.features.get(&feature).cloned().unwrap_or_default());
            }
            let build = normalize(&dir.join("build.rs"));
            let emitted = c
                .files
                .iter()
                .find(|file| file.path == build)
                .map(|file| {
                    re(r"rustc-cfg=(\w+)")
                        .captures_iter(&file.text)
                        .map(|m| m[1].into())
                        .collect()
                })
                .unwrap_or_default();
            (
                dir,
                CrateCfg {
                    on,
                    declared,
                    emitted,
                },
            )
        })
        .collect()
}
