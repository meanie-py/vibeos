//! Source-only cohab registry. Generated/build/cache trees are deliberately excluded;
//! unlike the historical Python scanner, traversal/read errors are reported.
use regex::Regex;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug)]
pub struct Scan {
    pub entries: BTreeMap<String, Value>,
    pub warnings: Vec<String>,
}

fn paths(root: &Path) -> Result<Vec<PathBuf>, String> {
    let files = crate::source_walk::files(root)?;
    let mut lists = Vec::new();
    let mut cmakes = Vec::new();
    for p in files {
        if p.file_name().is_some_and(|n| n == "CMakeLists.txt") {
            lists.push(p)
        } else if p.extension().is_some_and(|e| e == "cmake") {
            cmakes.push(p)
        }
    }
    lists.extend(cmakes);
    Ok(lists)
}
fn norm(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            _ => out.push(c.as_os_str()),
        }
    }
    out
}
fn resolve(root: &Path, cmake: &Path, raw: &str, srcdir: bool) -> Option<String> {
    if srcdir {
        // pathlib.Path removes redundant separators and '.' components, while
        // retaining '..' for source-root-prefixed references.
        let path: PathBuf = Path::new(raw)
            .components()
            .filter(|c| !matches!(c, Component::CurDir))
            .collect();
        return Some(path.to_string_lossy().into_owned());
    }
    let candidate = norm(&cmake.parent()?.join(raw));
    let resolved = if candidate.exists() {
        candidate.canonicalize().unwrap_or(candidate)
    } else {
        candidate
    };
    Some(
        resolved
            .strip_prefix(root)
            .ok()?
            .to_string_lossy()
            .into_owned(),
    )
}
fn py_repr(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::new();
    out.push(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if c.is_control() => {
                use std::fmt::Write;
                let _ = write!(out, "\\x{:02x}", c as u32);
            }
            _ => out.push(c),
        }
    }
    out.push(quote);
    out
}
fn full_rs(root: &Path, crate_name: &str, rs: &str) -> Result<PathBuf, String> {
    let dir = root.join("sdk/lib/rust").join(crate_name);
    let direct = dir.join(rs);
    if direct.is_file() || !dir.is_dir() {
        return Ok(direct);
    }
    let basename = Path::new(rs)
        .file_name()
        .ok_or("Rust reference has no filename")?;
    let mut hits: Vec<_> = crate::source_walk::files(&dir)?
        .into_iter()
        .filter(|p| p.file_name() == Some(basename))
        .collect();
    Ok(if hits.len() == 1 {
        hits.remove(0)
    } else {
        direct
    })
}
fn captures(text: &str, cr: &Regex, rr: &Regex) -> (Option<String>, Option<String>) {
    (
        cr.captures(text).map(|c| c[1].to_string()),
        rr.captures(text).map(|c| c[1].to_string()),
    )
}
fn read(path: &Path) -> Result<String, String> {
    fs::read(path)
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .map_err(|e| format!("{}: {e}", path.display()))
}

pub fn scan(root: &Path) -> Result<Scan, String> {
    let root = root
        .canonicalize()
        .map_err(|e| format!("{}: {e}", root.display()))?;
    let files = paths(&root)?;
    let path_re = Regex::new(r"^#+\s*(?P<srcdir>\$\{REACTOS_SOURCE_DIR\}/)?(?P<path>[A-Za-z0-9_./-]+\.(?:cpp|c))\b\s*(?P<rest>.*)$").unwrap();
    let ref_re = Regex::new(
        r"(?P<srcdir>\$\{REACTOS_SOURCE_DIR\}/)?(?P<path>[A-Za-z0-9_./-]+\.(?:cpp|c))\b",
    )
    .unwrap();
    let cr = Regex::new(r"\b([A-Za-z0-9_]+(?:-[A-Za-z0-9_]+)*-rust)\b").unwrap();
    let rr = Regex::new(r"\b(src/[A-Za-z0-9_./-]+\.rs)\b").unwrap();
    let signal = Regex::new(r"(?i)\brust\b").unwrap();
    let mut entries: BTreeMap<String, Value> = BTreeMap::new();
    let mut warnings = vec![];
    let mut refs = HashSet::new();
    for file in &files {
        let contents = read(file)?;
        let relfile = file.strip_prefix(&root).unwrap().to_string_lossy();
        let (mut hc, mut hr, mut hl): (Option<String>, Option<String>, Option<String>) =
            (None, None, None);
        for (i, line) in contents.lines().enumerate() {
            let s = line.trim();
            if s.is_empty() || !s.starts_with('#') {
                hc = None;
                hr = None;
                hl = None;
                if !s.is_empty() {
                    for c in ref_re.captures_iter(s) {
                        if let Some(p) =
                            resolve(&root, file, &c["path"], c.name("srcdir").is_some())
                        {
                            refs.insert(p);
                        }
                    }
                }
                continue;
            }
            let m = path_re.captures(s).filter(|c| {
                let r = &c["rest"];
                r.is_empty()
                    || r.starts_with('(')
                    || r.starts_with('-')
                    || r.starts_with('—')
                    || r.starts_with('→')
            });
            if let Some(c) = m {
                let Some(key) = resolve(&root, file, &c["path"], c.name("srcdir").is_some()) else {
                    continue;
                };
                let (ic, ir) = captures(&c["rest"], &cr, &rr);
                let parent = Path::new(&key)
                    .parent()
                    .and_then(Path::file_name)
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let guess = format!("{parent}-rust");
                let (crate_name, rs, kind, detail) = if let Some(name) = ic {
                    (name, ir, "inline", format!("{relfile}:{}: {s}", i + 1))
                } else if let Some(name) = hc.clone() {
                    (
                        name,
                        hr.clone(),
                        "block-hint",
                        format!(
                            "{relfile}: {} (block header)",
                            py_repr(hl.as_deref().unwrap_or(""))
                        ),
                    )
                } else if !parent.is_empty() && root.join("sdk/lib/rust").join(&guess).is_dir() {
                    (
                        guess.clone(),
                        None,
                        "dir-name-guess",
                        format!("derived '{guess}' from parent dir '{parent}'"),
                    )
                } else {
                    warnings.push(format!(
                        "skip (no rust signal): {key} @ {relfile}:{}",
                        i + 1
                    ));
                    continue;
                };
                let e = entries.entry(key.clone()).or_insert_with(|| json!({"c_path":key,"exists_on_disk":root.join(&key).is_file(),"disabled_at":[],"rust_crate":null,"rust_files":[],"evidence":[],"confidence":"LOW","also_compiled_uncommented":false}));
                e["disabled_at"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!(format!("{relfile}:{}", i + 1)));
                e["evidence"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"kind":kind,"detail":detail}));
                if e["rust_crate"].is_null() {
                    let exists = root.join("sdk/lib/rust").join(&crate_name).is_dir();
                    e["rust_crate"] = json!(crate_name);
                    e["_crate_exists"] = json!(exists);
                    e["_evidence_kind"] = json!(kind);
                    let mut confirmed = false;
                    if let Some(rs) = rs {
                        let dir = root.join("sdk/lib/rust").join(&crate_name);
                        if dir.is_dir() {
                            let full = full_rs(&root, &crate_name, &rs)?;
                            confirmed = full.is_file();
                            e["rust_files"].as_array_mut().unwrap().push(json!(full
                                .strip_prefix(&root)
                                .map_err(|e| e.to_string())?
                                .to_string_lossy()));
                        } else {
                            e["rust_files"].as_array_mut().unwrap().push(json!(rs));
                        }
                    }
                    e["_rs_confirmed"] = json!(confirmed);
                }
            } else if signal.is_match(s) {
                let (c, r) = captures(s, &cr, &rr);
                if c.is_some() {
                    hc = c;
                    hr = r;
                    hl = Some(s.to_owned());
                }
            }
        }
    }
    for (key, e) in &mut entries {
        let live = refs.contains(key);
        e["also_compiled_uncommented"] = json!(live);
        let obj = e.as_object_mut().unwrap();
        let kind = obj.remove("_evidence_kind").unwrap_or(Value::Null);
        let exists = obj
            .remove("_crate_exists")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let confirmed = obj
            .remove("_rs_confirmed")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let confidence = if live {
            "MEDIUM"
        } else if kind == "inline"
            && exists
            && (confirmed || obj["rust_files"].as_array().unwrap().is_empty())
        {
            if confirmed {
                "HIGH"
            } else {
                "MEDIUM"
            }
        } else if kind == "block-hint" && exists {
            "HIGH"
        } else if kind == "dir-name-guess" && exists {
            "MEDIUM"
        } else {
            "LOW"
        };
        obj.insert("confidence".into(), json!(confidence));
        for field in ["disabled_at", "rust_files"] {
            obj.get_mut(field)
                .unwrap()
                .as_array_mut()
                .unwrap()
                .sort_by(|a, b| a.as_str().cmp(&b.as_str()));
        }
        obj.get_mut("evidence")
            .unwrap()
            .as_array_mut()
            .unwrap()
            .sort_by(|a, b| a["detail"].as_str().cmp(&b["detail"].as_str()));
    }
    Ok(Scan { entries, warnings })
}
pub fn report(scan: &Scan) -> String {
    let mut counts = [0; 3];
    for e in scan.entries.values() {
        match e["confidence"].as_str() {
            Some("HIGH") => counts[0] += 1,
            Some("MEDIUM") => counts[1] += 1,
            _ => counts[2] += 1,
        }
    }
    format!(
        "gen_cohab_registry: {} disabled-with-Rust-signal TUs (HIGH={} MEDIUM={} LOW={})",
        scan.entries.len(),
        counts[0],
        counts[1],
        counts[2]
    )
}
pub fn generate(root: &Path) -> Result<String, String> {
    render(&scan(root)?)
}
pub fn render(scan: &Scan) -> Result<String, String> {
    let s = serde_json::to_string_pretty(&scan.entries).map_err(|e| e.to_string())?;
    let mut out = String::new();
    for c in s.chars() {
        if (c as u32) < 0x7f {
            out.push(c);
        } else {
            use std::fmt::Write;
            for u in c.encode_utf16(&mut [0; 2]).iter() {
                let _ = write!(out, "\\u{u:04x}");
            }
        }
    }
    out.push('\n');
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn python_ascii_json_escapes_del_and_surrogate_pairs() {
        let scan = Scan {
            entries: BTreeMap::from([("entry".into(), json!("\u{7f}😀"))]),
            warnings: Vec::new(),
        };
        assert_eq!(
            render(&scan).unwrap(),
            concat!("{\n", r#"  "entry": "\u007f\ud83d\ude00""#, "\n}\n")
        );
    }
}
