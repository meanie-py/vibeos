//! Read-only port of `tools/vendored.py`'s default `--check` audit.
//!
//! YAML decoding and filesystem inventory are intentionally outside this module. The
//! loader supplies parsed YAML values plus one immutable snapshot. Snapshot paths are
//! repo-relative, `/` separated paths; `files` contains decoded file text (replacement
//! semantics matching Python `errors="replace"`), `directories` contains every
//! descended directory, and `markers` contains marker paths encountered before their
//! directories were pruned (`.git`, `.rustc_info.json`, and `CACHEDIR.TAG`). This is the
//! data needed to preserve the oracle's nested-checkout/build-output walk boundaries.
use crate::Corpus;
use regex::{Regex, RegexBuilder};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::LazyLock;

const SRC: &[&str] = &[".c", ".cpp", ".cxx", ".cc", ".h", ".hpp", ".hxx"];
const SUSPECT_SKIP: &[&str] = &[
    ".git",
    ".claude",
    "build-i386",
    "build-amd64",
    "build-arm64",
];
const COLLECTION_ROOTS: &[&str] = &[
    "",
    "modules/rosapps",
    "modules/rostests",
    "sdk/include/psdk",
    "sdk/include/vcruntime",
];
const STAMP_NAME: &str = "VENDORED";
const NOTICES_PATH: &str = "THIRD-PARTY-NOTICES.md";
const CRATES_PATH: &str = "tools/rust-crates.yaml";
const CARGO_LOCK: &str = "sdk/lib/rust/Cargo.lock";
const TRANSLATE: &[&str] = &["planned", "keep", "undecided"];

/// Files and directories read by a single `vendored.py --check` execution.
///
/// `files` must contain every source-suffix file under the repo (for the four full-tree
/// ratchets), all directories (including empty directories), all registry-selected
/// license/version/patch/header paths, `vendor.yaml`, Cargo.lock, rust-crates.yaml,
/// THIRD-PARTY-NOTICES.md, and every VENDORED stamp. `markers` records entries encountered
/// before descent was pruned, so a `.git` directory or file, `.rustc_info.json`, or
/// `CACHEDIR.TAG` prevents walking that subtree exactly as `_is_unwalked` does.
#[derive(Clone, Debug, Default)]
pub struct AuditSnapshot {
    pub files: BTreeMap<String, String>,
    pub directories: BTreeSet<String>,
    pub markers: BTreeSet<String>,
    /// Native os.walk file visitation order, not lexical path order.
    pub ordered_files: Vec<String>,
    /// Native os.walk directory visitation order, not lexical path order.
    pub ordered_directories: Vec<String>,
    /// True only when the loader has established that this is the canonical
    /// project checkout. Python reconciles `_GRANTS_ACCEPTED` only for REPO.
    pub is_project_repo: bool,
}
impl AuditSnapshot {
    pub fn file(&self, path: &str) -> Option<&str> {
        self.files.get(&norm(path)).map(String::as_str)
    }
    pub fn exists(&self, path: &str) -> bool {
        let p = norm(path);
        self.files.contains_key(&p) || self.directories.contains(&p) || self.markers.contains(&p)
    }
    pub fn is_file(&self, path: &str) -> bool {
        self.files.contains_key(&norm(path))
    }
    pub fn is_dir(&self, path: &str) -> bool {
        self.directories.contains(&norm(path))
    }
}
fn norm(s: &str) -> String {
    s.replace('\\', "/").trim_matches('/').to_string()
}
fn child(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.into()
    } else {
        format!("{parent}/{name}")
    }
}
fn within(path: &str, prefix: &str) -> bool {
    path == prefix
        || path
            .strip_prefix(prefix)
            .is_some_and(|r| r.starts_with('/'))
}
fn json_str<'a>(v: &'a Value, k: &str) -> Option<&'a str> {
    v.get(k).and_then(Value::as_str)
}
fn optional_str(v: &Value, k: &str) -> Option<String> {
    json_str(v, k).filter(|s| !s.is_empty()).map(str::to_string)
}
fn strings(v: &Value, k: &str) -> Vec<String> {
    v.get(k)
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}
fn field_err(path: &str, field: &str) -> String {
    format!("{path} is missing required field `{field}`")
}

#[derive(Clone, Debug)]
struct Tree {
    path: String,
    name: String,
    reason: String,
    upstream: String,
    license: String,
    license_file: Option<String>,
    license_ref: Option<String>,
    patch: Option<String>,
    license_extra: BTreeMap<String, String>,
    license_note: Option<String>,
    headers: Vec<String>,
    version: Option<Value>,
    translate: String,
}
#[derive(Clone, Debug)]
struct Ours {
    name: String,
    reason: String,
    paths: Vec<String>,
}
#[derive(Clone, Debug)]
struct Registry {
    containers: Vec<String>,
    trees: Vec<Tree>,
    ours: Vec<Ours>,
}
impl Registry {
    fn parse(raw: &Value) -> Result<Self, String> {
        if !raw.is_object() {
            return Err("vendor.yaml is not a mapping".into());
        }
        let mut trees = Vec::new();
        let ta = raw
            .get("trees")
            .and_then(Value::as_array)
            .ok_or("vendor.yaml `trees` is not a list")?;
        for (i, v) in ta.iter().enumerate() {
            if !v.is_object() {
                return Err(format!("trees[{i}] is not a mapping"));
            }
            for field in ["path", "name", "reason"] {
                if json_str(v, field).is_none_or(str::is_empty) {
                    return Err(field_err(&format!("trees[{i}]"), field));
                }
            }
            let path = json_str(v, "path").unwrap().to_string();
            let name = json_str(v, "name").unwrap().to_string();
            let reason = json_str(v, "reason").unwrap().to_string();
            let license_extra = v
                .get("license_extra")
                .and_then(Value::as_object)
                .map(|m| {
                    m.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect()
                })
                .unwrap_or_default();
            trees.push(Tree {
                path,
                name,
                reason,
                upstream: json_str(v, "upstream").unwrap_or("unknown").into(),
                license: json_str(v, "license").unwrap_or("unknown").into(),
                license_file: optional_str(v, "license_file"),
                license_ref: optional_str(v, "license_ref"),
                patch: optional_str(v, "patch"),
                license_extra,
                license_note: optional_str(v, "license_note"),
                headers: strings(v, "headers"),
                version: v.get("version").filter(|x| !x.is_null()).cloned(),
                translate: match v.get("translate") {
                    None => "undecided".into(),
                    Some(Value::String(s)) => s.clone(),
                    Some(Value::Bool(true)) => "True".into(),
                    Some(Value::Bool(false)) => "False".into(),
                    Some(Value::Null) => "None".into(),
                    Some(value) => value.to_string(),
                },
            });
        }
        if trees.is_empty() {
            return Err("vendor.yaml registers no trees".into());
        }
        let mut seen = BTreeMap::new();
        for (i, t) in trees.iter().enumerate() {
            if let Some(old) = seen.insert(&t.path, i) {
                return Err(format!(
                    "duplicate path `{}` (trees[{old}] and [{i}])",
                    t.path
                ));
            }
        }
        let mut ours = Vec::new();
        for (i, v) in raw
            .get("ours")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .enumerate()
        {
            if !v.is_object() {
                return Err(format!("ours[{i}] is not a mapping"));
            }
            for field in ["name", "reason"] {
                if json_str(v, field).is_none_or(str::is_empty) {
                    return Err(field_err(&format!("ours[{i}]"), field));
                }
            }
            let mut paths = optional_str(v, "path").into_iter().collect::<Vec<_>>();
            if paths.is_empty() {
                paths = strings(v, "paths");
            }
            if paths.is_empty() {
                return Err(format!("ours[{i}] needs `path` or `paths`"));
            }
            ours.push(Ours {
                name: json_str(v, "name").unwrap().into(),
                reason: json_str(v, "reason").unwrap().into(),
                paths,
            });
        }
        let mut os = BTreeMap::new();
        for (i, o) in ours.iter().enumerate() {
            // Match Python's OurTree.path property: only the first path represents
            // the entry for duplicate/overlap validation; classification still uses
            // every item in paths.
            let p = &o.paths[0];
            if let Some(old) = os.insert(p, i) {
                return Err(format!("duplicate path `{p}` (ours[{old}] and [{i}])"));
            }
            if trees
                .iter()
                .any(|t| t.prefixes().iter().any(|x| within(p, x)))
            {
                return Err(format!("ours[{i}] `{p}` is also covered by a `trees:` entry — it cannot be both vendored and a translation target"));
            }
        }
        Ok(Self {
            containers: strings(raw, "containers"),
            trees,
            ours,
        })
    }
    fn is_vendored(&self, p: &str) -> bool {
        let p = norm(p);
        self.trees
            .iter()
            .any(|t| t.prefixes().iter().any(|x| within(&p, x)))
    }
    fn is_ours(&self, p: &str) -> bool {
        let p = norm(p);
        self.ours
            .iter()
            .any(|o| o.paths.iter().any(|x| within(&p, x)))
    }
    fn classified(&self, p: &str) -> bool {
        self.is_vendored(p) || self.is_ours(p)
    }
    fn owner(&self, p: &str) -> Option<&Tree> {
        let p = norm(p);
        let mut best: Option<(&Tree, usize)> = None;
        for t in &self.trees {
            for prefix in
                std::iter::once(t.path.as_str()).chain(t.headers.iter().map(String::as_str))
            {
                if within(&p, prefix) && best.is_none_or(|(_, n)| prefix.len() > n) {
                    best = Some((t, prefix.len()));
                }
            }
        }
        best.map(|(t, _)| t)
    }
}
impl Tree {
    fn prefixes(&self) -> Vec<String> {
        std::iter::once(self.path.clone())
            .chain(self.headers.iter().cloned())
            .collect()
    }
    fn is_single_file(&self, s: &AuditSnapshot) -> bool {
        s.is_file(&self.path)
    }
}

static FOREIGN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Copyright\s*(?:\(c\)|\u{a9}|&copy;)?\s*(?:[0-9]{4}\s*(?:[-,/ ]\s*[0-9]{4})?[\s,]*)*(Microsoft|Intel|Apple|Oracle|IBM|Sun Microsystems|Nvidia|Adobe|Silicon Graphics|Hewlett-Packard|Digital Equipment)|Copyright[^\n]{0,60}?\bfor (CodeWeavers)|Copyright[^\n]{0,60}?\b(The Wine Project)").unwrap()
});
static ANY_COPYRIGHT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)copyright\s+(?:\(c\)|\u{a9}|[0-9]{4})").unwrap());
static SPDX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"SPDX-License-Identifier:[ \t]*([^\n]+)").unwrap());
static ROS_LICENSE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*[*#/]*\s*LICENSE:\s*([^\n(]+)").unwrap());
static LICENSE_DECL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)LICEN[CS]E:\s*([^\n*]{0,90})").unwrap());
static INLINE_GRANT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Redistribution and use|Permission is hereby granted|Permission to use, copy|is free software|you (can|may) redistribute|Licensed under the|SPDX-License-Identifier|(placed|released|dedicated).{0,20}public domain|GNU (General|Lesser|Library) Public License").unwrap()
});
static KNOWN_LICENCE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(GPL|LGPL|MIT|BSD|0BSD|Zlib|Apache|ISC|CC0|CC-BY|HPND|Unlicense|X11|libpng|IJG|FDLIBM|SGI-B|MPL|Boost|PSF|Public\s+Domain|SPDX)\b").unwrap()
});
static POINTER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bsee\s+(?:the\s+)?(?:file\s+)?([A-Za-z0-9._-]*(?:COPYING|LICEN[CS]E|LGPL|GPL)[A-Za-z0-9._-]*|[A-Za-z0-9._-]+\.txt)").unwrap()
});
static LICENCE_NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)copying|licen[cs]e|gpl|cddl|artistic|apache|mozilla|mpl|bsd\b|zlib|\bmit\b")
        .unwrap()
});
static LICENCE_BASE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^(copying|licen[cs]e)([0-9]|[._-][a-z0-9._-]+)?$").unwrap());
static FLOSS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)GPL|General Public|Lesser|MIT|BSD|Apache|Zlib|zlib|ISC|Public\s*Domain|CC0|CC-BY|MPL|Mozilla|Unlicense|WTFPL|X11|Boost|SGI|Info-ZIP|IJG|FDLIBM|STLport|see COPYING|Dual-licensed").unwrap()
});
static JOINED_FIELD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\S[ \t]{2,}((?:path|paths|name|upstream|license|license_file|license_ref|license_extra|modified|headers|version|reason|review|unwelcome|origin|tier|license_note|staleness|translate|translate_note):)").unwrap()
});
static REF_LEAD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(/\*+|\*+/|\*+|//+)\s?").unwrap());

fn py_regex(pattern: &str) -> Result<Regex, String> {
    RegexBuilder::new(pattern)
        .case_insensitive(false)
        .build()
        .map_err(|e| format!("unsupported registry regex /{pattern}/: {e}"))
}
fn version(t: &Tree, s: &AuditSnapshot) -> Result<String, String> {
    let Some(v) = &t.version else {
        return Ok("unknown".into());
    };
    if let Some(l) = v.get("literal") {
        return Ok(l
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| l.to_string()));
    }
    let Some(path) = v.get("file").and_then(Value::as_str) else {
        return Err(format!(
            "{}: version probe has neither `literal` nor `file`",
            t.name
        ));
    };
    let Some(text) = s.file(path) else {
        return Err(format!("{}: version probe file is missing: {path}", t.name));
    };
    let pats = v
        .get("patterns")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).collect::<Vec<_>>())
        .filter(|x| !x.is_empty())
        .or_else(|| v.get("pattern").and_then(Value::as_str).map(|p| vec![p]))
        .ok_or_else(|| format!("{}: version probe names no `pattern`/`patterns`", t.name))?;
    let mut got = Vec::new();
    for p in pats {
        let re = py_regex(p)?;
        let Some(m) = re.captures(text) else {
            return Err(format!(
                "{}: version probe found no match for /{p}/ in {path}",
                t.name
            ));
        };
        got.push(
            m.get(1)
                .ok_or_else(|| format!("{}: version pattern /{p}/ has no capture group 1", t.name))?
                .as_str(),
        );
    }
    Ok(got.join(v.get("join").and_then(Value::as_str).unwrap_or("")))
}
fn stamp(t: &Tree) -> String {
    format!("{} — VENDORED THIRD-PARTY CODE. NOT A TRANSLATION TARGET.\n\nThis tree is foreign code carried by the project. It is deliberately NOT part of the\nC -> Rust effort and does not count toward translation progress.\n\n  upstream: {}\n  licence:  {}\n\nRegistered in vendor.yaml; see that file for the reason and tools/vendored.py for the\nchecks. Do not translate, and do not \"clean up\" this tree — local edits make the next\nre-vendor a merge instead of a copy.\n\n@generated by tools/vendored.py --write-stamps — do not hand-edit.\n",t.name,t.upstream,t.license)
}
fn structural(text: &str) -> Vec<String> {
    text.lines().enumerate().filter_map(|(i,l)|JOINED_FIELD.captures(l).map(|m|format!("vendor.yaml:{}: `{}` is not at the start of a line — it was absorbed into the value above it, and has stopped being a field. Line: {}",i+1,m[1].trim(),l.trim().chars().take(90).collect::<String>()))).collect()
}
fn read_head<'a>(s: &'a AuditSnapshot, p: &str, n: usize) -> Option<&'a str> {
    s.file(p)
        .map(|x| &x[..x.char_indices().nth(n).map_or(x.len(), |(i, _)| i)])
}
fn suffix(p: &str, allowed: &[&str]) -> bool {
    let low = p.to_ascii_lowercase();
    allowed.iter().any(|s| low.ends_with(s))
}
/// Run all branches of the registered default `vendored.py --check` audit.
/// `registry` is `yaml.safe_load(vendor.yaml)` serialized to JSON. `crate_snapshot` is
/// the parsed `tools/rust-crates.yaml` value plus the Cargo.lock (see `crate_rows` and
/// `lock_pairs` below); it is intentionally supplied by the central snapshot loader.
pub fn audit(
    _corpus: &Corpus,
    registry: &Value,
    crate_snapshot: &Value,
    snapshot: &AuditSnapshot,
) -> Result<Vec<String>, String> {
    let reg = Registry::parse(registry)?;
    let mut problems = Vec::<String>::new();
    let registered: BTreeSet<_> = reg.trees.iter().map(|t| t.path.as_str()).collect();
    for t in &reg.trees {
        if !snapshot.exists(&t.path) {
            problems.push(format!(
                "{}: registered path does not exist: {}",
                t.name, t.path
            ));
        }
        if !TRANSLATE.contains(&t.translate.as_str()) {
            problems.push(format!(
                "{}: translate: '{}' is not one of planned, keep, undecided",
                t.name, t.translate
            ));
        }
        for (key, label) in [
            (t.license_file.as_deref(), "license_file"),
            (t.license_ref.as_deref(), "license_ref"),
        ] {
            if let Some(p) = key {
                if !snapshot.is_file(p) {
                    problems.push(format!("{}: {label} does not exist: {p}", t.name));
                }
            }
        }
        if t.license != "unknown"
            && t.license_file.is_none()
            && t.license_ref.is_none()
            && t.license_note.is_none()
        {
            problems.push(format!("{}: records licence `{}` but has no `license_file`, no `license_ref` and no `license_note` — nothing for --notices to quote. Point at the grant, or state in `license_note` why the tree does not carry one.",t.name,t.license));
        }
        if let Some(p) = &t.patch {
            if !snapshot.is_file(p) {
                problems.push(format!("{}: patch does not exist: {p}", t.name));
            } else if snapshot.file(p).unwrap().trim().is_empty() {
                problems.push(format!("{}: patch is empty: {p}", t.name));
            }
        }
        for h in &t.headers {
            if !snapshot.exists(h) {
                problems.push(format!("{}: header path does not exist: {h}", t.name));
            }
        }
        if let Err(e) = version(t, snapshot) {
            problems.push(e);
        }
        if snapshot.is_file(&t.path) {
            continue;
        }
        let sp = child(&t.path, STAMP_NAME);
        match snapshot.file(&sp) {
            None => problems.push(format!(
                "{}: missing {t_path}/{STAMP_NAME} — run --write-stamps",
                t.name,
                t_path = t.path
            )),
            Some(txt) if txt != stamp(t) => {
                problems.push(format!("{}: {sp} is STALE — run --write-stamps", t.name))
            }
            _ => {}
        }
    }
    for container in &reg.containers {
        if !snapshot.is_dir(container) {
            problems.push(format!("container does not exist: {container}"));
            continue;
        }
        let mut children = snapshot
            .directories
            .iter()
            .filter(|d| {
                Path::new(d.as_str())
                    .parent()
                    .is_some_and(|p| p == Path::new(container))
            })
            .cloned()
            .collect::<Vec<_>>();
        children.sort();
        for rel in children {
            if !registered.contains(rel.as_str()) {
                problems.push(format!("UNREGISTERED vendored tree: {rel}\n    Add an entry to vendor.yaml (or, if this is OUR code, move it out of {container}/)."));
            }
        }
    }
    for o in &reg.ours {
        for p in &o.paths {
            if !snapshot.is_dir(p) {
                problems.push(format!("{}: `ours:` path does not exist: {p}", o.name));
            }
        }
    }
    for (d, n) in suspects(&reg, snapshot) {
        problems.push(format!("UNCLASSIFIED foreign-copyright code: {d} ({n} files)\n    Decide, and record it in vendor.yaml: a `trees:` entry if it is\n    vendored and upstream-tracked, an `ours:` entry if it is a translation\n    target. Do not answer this by copyright grep — see tools/vendored.py."));
    }
    if let Some(yaml) = snapshot.file("vendor.yaml") {
        problems.extend(structural(yaml));
    } else {
        return Err("vendor.yaml text missing from audit snapshot".into());
    }
    for (path, recorded, un) in license_drift(&reg, snapshot) {
        let listed = un
            .iter()
            .map(|(l, n)| format!("{l} x{n}"))
            .collect::<Vec<_>>()
            .join(", ");
        problems.push(format!("UNDECLARED LICENCE in {path}: recorded `{recorded}`, but its own files\n    declare: {listed}\n    Record them in `license_extra:` with a reason, or move those files out\n    of the tree if they are ours. A notices file built from the headline\n    licence alone would be wrong about this tree."));
    }
    for (value, example, n) in declared_licences(&reg, snapshot) {
        problems.push(format!("NON-FLOSS LICENCE DECLARED by {n} file(s): `{value}`\n    e.g. {example}\n    A ReactOS `LICENSE:` header naming terms that are not a FLOSS licence.\n    Settle it: delete the code, correct the header if it is wrong, or add a\n    vendor.yaml entry recording what the terms actually are and why we ship it."));
    }
    let (no_grant, dangling) = grant_gaps(&reg, snapshot);
    let grants: Value = serde_json::from_str(include_str!("../../vendor-grants.json"))
        .map_err(|e| format!("invalid vendor-grants.json: {e}"))?;
    let accepted: BTreeSet<&str> = if snapshot.is_project_repo {
        grants
            .as_object()
            .ok_or("vendor-grants.json must be a mapping")?
            .keys()
            .map(String::as_str)
            .collect()
    } else {
        BTreeSet::new()
    };
    let found: BTreeSet<String> = no_grant
        .iter()
        .map(|(d, _, _)| format!("dir:{d}"))
        .chain(dangling.keys().map(|k| format!("ptr:{k}")))
        .collect();
    for k in found.difference(&accepted.iter().map(|s| s.to_string()).collect()) {
        problems.push(format!("NEW licence-grant finding `{k}` — code that claims a copyright and never says what you may do with it, or a licence text it points at that does not ship. Establish the terms (see tools/vendored.py --grants), then either fix it or add it to _GRANTS_ACCEPTED with the argument for tolerating it."));
    }
    for k in accepted {
        if !found.contains(k) {
            problems.push(format!("`{k}` is listed in _GRANTS_ACCEPTED but --grants no longer reports it. If it was resolved, delete the entry; a tolerated finding that has quietly gone away is one nobody will re-read."));
        }
    }
    crate_drift(crate_snapshot, snapshot, &mut problems);
    if let Some(current) = snapshot.file(NOTICES_PATH) {
        if !problems.iter().any(|p| p.contains("--notices to quote")) {
            let generated = notices(&reg, snapshot, crate_snapshot)?;
            if current != generated {
                problems.push(format!("{NOTICES_PATH} is STALE — vendor.yaml or a licence file changed since it was generated. Regenerate: tools/vendored.py --notices {NOTICES_PATH}"));
            }
        }
    }
    Ok(problems)
}

fn walkable_dirs(s: &AuditSnapshot, root: &str) -> Vec<String> {
    let ordered = if s.ordered_directories.is_empty() {
        s.directories.iter().cloned().collect()
    } else {
        s.ordered_directories.clone()
    };
    ordered
        .into_iter()
        .filter(|d| root.is_empty() || within(d, root))
        .filter(|d| !blocked(s, d))
        .collect()
}
fn unwalked(s: &AuditSnapshot, d: &str) -> bool {
    s.markers.contains(&child(d, ".git"))
        || s.markers.contains(&child(d, ".rustc_info.json"))
        || s.markers.contains(&child(d, "CACHEDIR.TAG"))
        || s.files.contains_key(&child(d, ".git"))
        || s.directories.contains(&child(d, ".git"))
}
fn blocked(s: &AuditSnapshot, d: &str) -> bool {
    let mut cur = String::new();
    for part in d.split('/').filter(|p| !p.is_empty()) {
        cur = child(&cur, part);
        if part == ".git"
            || s.markers.contains(&child(&cur, ".git"))
            || s.markers.contains(&child(&cur, ".rustc_info.json"))
            || s.markers.contains(&child(&cur, "CACHEDIR.TAG"))
        {
            return true;
        }
    }
    false
}
fn skip_suspect(d: &str) -> bool {
    SUSPECT_SKIP.contains(&d)
}
fn suspects(reg: &Registry, s: &AuditSnapshot) -> Vec<(String, usize)> {
    let mut counts = BTreeMap::<String, usize>::new();
    let mut dir_order = Vec::<String>::new();
    let order = if s.ordered_files.is_empty() {
        s.files.keys().cloned().collect()
    } else {
        s.ordered_files.clone()
    };
    for p in order {
        let Some(text) = s.file(&p) else { continue };
        let d = Path::new(&p)
            .parent()
            .and_then(Path::to_str)
            .unwrap_or("")
            .replace('\\', "/");
        let root_dir = d.split('/').next().unwrap_or("");
        if skip_suspect(root_dir) || !suffix(&p, SRC) || blocked(s, &d) || reg.classified(&p) {
            continue;
        }
        let head = read_head(s, &p, 4000).unwrap_or(text);
        if FOREIGN.is_match(head) {
            if !counts.contains_key(&d) {
                dir_order.push(d.clone());
            }
            *counts.entry(d).or_default() += 1;
        }
    }
    let mut out: Vec<_> = dir_order
        .into_iter()
        .filter_map(|d| counts.get(&d).copied().filter(|n| *n >= 5).map(|n| (d, n)))
        .collect();
    out.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    out
}

fn license_drift(
    reg: &Registry,
    s: &AuditSnapshot,
) -> Vec<(String, String, BTreeMap<String, usize>)> {
    let mut out = Vec::new();
    for t in &reg.trees {
        let mut found = BTreeMap::<String, usize>::new();
        let mut prose = BTreeMap::<String, usize>::new();
        for (p, text) in s
            .files
            .iter()
            .filter(|(p, _)| within(p, &t.path) && suffix(p, SRC))
        {
            let head = read_head(s, p, 3000).unwrap_or(text);
            let mut declared = false;
            for re in [&*SPDX, &*ROS_LICENSE] {
                if let Some(m) = re.captures(head) {
                    let lic = m
                        .get(1)
                        .unwrap()
                        .as_str()
                        .trim()
                        .trim_end_matches("*/")
                        .trim()
                        .trim_end_matches('.')
                        .trim();
                    if !lic.is_empty() {
                        *found.entry(lic.to_string()).or_default() += 1;
                        declared = true;
                    }
                }
            }
            if declared {
                continue;
            }
            if let Some((family, _)) = prose_license(head) {
                *prose.entry(format!("{family} (prose)")).or_default() += 1;
            }
        }
        let mut un = BTreeMap::new();
        for (l, n) in found {
            if !t.license.to_lowercase().contains(&l.to_lowercase())
                && !t.license_extra.contains_key(&l)
            {
                un.insert(l, n);
            }
        }
        for (l, n) in prose {
            let fam = l.strip_suffix(" (prose)").unwrap();
            if !prose_accounted(
                fam,
                &format!(
                    "{} {}",
                    t.license,
                    t.license_extra
                        .keys()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
            ) {
                un.insert(l, n);
            }
        }
        if !un.is_empty() {
            out.push((t.path.clone(), t.license.clone(), un));
        }
    }
    out
}
fn prose_license(s: &str) -> Option<(&'static str, Regex)> {
    let pats: [(&str, &str); 10] = [
        (
            "SGI-B-1.1",
            r"(?i)SGI\s+Free\s+Software\s+License\s+B,?\s*Version\s*1\.1",
        ),
        (
            "SGI-B-2.0",
            r"(?i)SGI\s+Free\s+Software\s+License\s+B,?\s*Version\s*2\.0",
        ),
        ("Info-ZIP", r"(?i)\bInfo-ZIP\s+licen[cs]e\b"),
        (
            "LGPL",
            r"(?i)GNU\s+(?:Lesser|Library)\s+General\s+Public\s+Licen[cs]e",
        ),
        ("GPL", r"(?i)GNU\s+General\s+Public\s+Licen[cs]e"),
        ("Apache", r"(?i)Apache\s+Licen[cs]e"),
        ("MPL", r"(?i)Mozilla\s+Public\s+Licen[cs]e"),
        (
            "CDDL",
            r"(?i)Common\s+Development\s+and\s+Distribution\s+Licen[cs]e",
        ),
        ("BSL", r"(?i)Boost\s+Software\s+Licen[cs]e"),
        ("EPL", r"(?i)Eclipse\s+Public\s+Licen[cs]e"),
    ];
    for (f, p) in pats {
        let r = Regex::new(p).unwrap();
        if r.is_match(s) {
            return Some((f, r));
        }
    }
    None
}
fn prose_accounted(f: &str, s: &str) -> bool {
    let p = match f {
        "SGI-B-1.1" => r"(?i)SGI-B-1\.1",
        "SGI-B-2.0" => r"(?i)SGI-B-2\.0",
        "Info-ZIP" => r"(?i)Info-ZIP",
        "LGPL" => r"(?i)LGPL|Lesser|Library General",
        "GPL" => r"(?i)(^|[^A-Za-z])GPL",
        "Apache" => r"(?i)Apache",
        "MPL" => r"(?i)MPL|Mozilla",
        "CDDL" => r"(?i)CDDL",
        "BSL" => r"(?i)BSL|Boost",
        "EPL" => r"(?i)EPL|Eclipse",
        _ => return false,
    };
    Regex::new(p).unwrap().is_match(s)
}

// License declarations and grant gaps use the same path-local, marker-pruned snapshot.
fn declared_licences(reg: &Registry, s: &AuditSnapshot) -> Vec<(String, String, usize)> {
    // Python uses each Tree.path and Our.path (the singular field only), then
    // `rel_dir.startswith(registered)`, which is a raw string-prefix test.
    let prefixes: Vec<String> = reg
        .trees
        .iter()
        .map(|t| t.path.clone())
        .chain(reg.ours.iter().filter_map(|o| o.paths.first().cloned()))
        .collect();
    let order = if s.ordered_files.is_empty() {
        s.files.keys().cloned().collect()
    } else {
        s.ordered_files.clone()
    };
    let mut counts = Vec::<(String, String, usize, String)>::new();
    let mut index = BTreeMap::<String, usize>::new();
    for p in order {
        let Some(text) = s.file(&p) else { continue };
        let dir = Path::new(&p).parent().and_then(Path::to_str).unwrap_or("");
        if !SRC.iter().any(|ext| p.ends_with(ext))
            || prefixes.iter().any(|x| dir.starts_with(x))
            || dir
                .split('/')
                .any(|part| part.starts_with('.') || part.starts_with("build-"))
            || blocked(s, dir)
        {
            continue;
        }
        let Some(m) = ROS_LICENSE.captures(read_head(s, &p, 2000).unwrap_or(text)) else {
            continue;
        };
        let v = m[1].trim().trim_end_matches("*/").trim();
        if FLOSS.is_match(v) {
            continue;
        }
        let key = v.to_lowercase();
        if let Some(i) = index.get(&key).copied() {
            counts[i].2 += 1;
        } else {
            index.insert(key.clone(), counts.len());
            counts.push((key, v.into(), 1, p));
        }
    }
    let mut out: Vec<_> = counts.into_iter().map(|(_, v, n, p)| (v, p, n)).collect();
    out.sort_by_key(|(_, _, n)| std::cmp::Reverse(*n));
    out
}
fn grant_gaps(
    reg: &Registry,
    s: &AuditSnapshot,
) -> (
    Vec<(String, usize, String)>,
    BTreeMap<String, (usize, String)>,
) {
    // Group every file by its parent in one pass. Rescanning the whole snapshot
    // once per walkable directory was quadratic in the tree and took this rule
    // past pytest's 300 s timeout.
    let mut grouped = BTreeMap::<&Path, BTreeSet<String>>::new();
    for p in s.files.keys() {
        let path = Path::new(p.as_str());
        if let Some(parent) = path.parent() {
            grouped.entry(parent).or_default().insert(
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_lowercase(),
            );
        }
    }
    let mut by_dir = BTreeMap::<String, BTreeSet<String>>::new();
    by_dir.insert(
        String::new(),
        grouped.get(Path::new("")).cloned().unwrap_or_default(),
    );
    for d in walkable_dirs(s, "") {
        let root_dir = d.split('/').next().unwrap_or("");
        if skip_suspect(root_dir) || blocked(s, &d) {
            continue;
        }
        let names = grouped
            .get(Path::new(d.as_str()))
            .cloned()
            .unwrap_or_default();
        by_dir.insert(d, names);
    }
    fn resolve(name: &str, from: &str, by: &BTreeMap<String, BTreeSet<String>>, depth: u8) -> bool {
        if depth > 2 {
            return false;
        }
        let mut n = name.to_lowercase();
        for (a, b) in [
            ("lgpl.txt", "copying.lib"),
            ("gpl.txt", "copying"),
            ("minizip_info.txt", "minizip64_info.txt"),
        ] {
            if n == a && resolve(b, from, by, depth + 1) {
                return true;
            }
        }
        let loose = !n.contains('.');
        let mut d = from.to_string();
        loop {
            if let Some(here) = by.get(&d) {
                if here.contains(&n)
                    || loose && here.iter().any(|f| f.starts_with(&(n.clone() + ".")))
                {
                    return true;
                }
            }
            if d.is_empty() {
                return false;
            }
            d = d.rsplit_once('/').map_or(String::new(), |(p, _)| p.into());
        }
    }
    fn component_license(from: &str, by: &BTreeMap<String, BTreeSet<String>>) -> bool {
        let mut d = from.to_string();
        while !d.is_empty() {
            if !COLLECTION_ROOTS.contains(&d.as_str())
                && by
                    .get(&d)
                    .is_some_and(|names| names.iter().any(|n| LICENCE_BASE.is_match(n)))
            {
                return true;
            }
            d = d.rsplit_once('/').map_or(String::new(), |(p, _)| p.into());
        }
        false
    }
    let mut per = BTreeMap::<String, (usize, bool, String)>::new();
    let mut per_order = Vec::<String>::new();
    let mut dangling = BTreeMap::<String, (usize, String)>::new();
    let order = if s.ordered_files.is_empty() {
        s.files.keys().cloned().collect()
    } else {
        s.ordered_files.clone()
    };
    for p in order {
        let Some(text) = s.file(&p) else { continue };
        if !suffix(&p, SRC) {
            continue;
        }
        let dir = Path::new(&p)
            .parent()
            .and_then(Path::to_str)
            .unwrap_or("")
            .replace('\\', "/");
        let root_dir = dir.split('/').next().unwrap_or("");
        if skip_suspect(root_dir) || blocked(s, &dir) {
            continue;
        }
        let head = read_head(s, &p, 4000).unwrap_or(text);
        for m in POINTER.captures_iter(head) {
            let mut name = m[1].trim_end_matches('.').to_string();
            if !name.contains('.') && !isupper(&name) {
                continue;
            }
            name = name.to_lowercase();
            if resolve(&name, &dir, &by_dir, 0) {
                continue;
            }
            if reg
                .owner(&p)
                .and_then(|t| t.license_file.as_deref())
                .is_some_and(|f| s.is_file(f))
            {
                continue;
            }
            let e = dangling.entry(name).or_insert((0, p.clone()));
            e.0 += 1;
        }
        if reg.classified(&p) {
            continue;
        }
        if !per.contains_key(&dir) {
            per_order.push(dir.clone());
        }
        let st = per.entry(dir.clone()).or_insert((0, false, p.clone()));
        if ANY_COPYRIGHT.is_match(head) {
            st.0 += 1;
        }
        if INLINE_GRANT.is_match(head) {
            st.1 = true;
            continue;
        }
        if LICENSE_DECL
            .captures(head)
            .is_some_and(|m| KNOWN_LICENCE.is_match(m.get(1).unwrap().as_str()))
        {
            st.1 = true;
            continue;
        }
        for m in POINTER.captures_iter(head) {
            let nm = m[1].trim_end_matches('.');
            if !LICENCE_NAME.is_match(nm) {
                continue;
            }
            if (nm.contains('.') || isupper(nm)) && resolve(nm, &dir, &by_dir, 0) {
                st.1 = true;
                break;
            }
        }
        if !st.1 && component_license(&dir, &by_dir) {
            st.1 = true;
        }
    }
    let mut no = per_order
        .into_iter()
        .filter_map(|d| {
            per.get(&d)
                .filter(|(n, covered, _)| *n >= 3 && !*covered)
                .map(|(n, _, e)| (d, *n, e.clone()))
        })
        .collect::<Vec<_>>();
    no.sort_by_key(|(_, n, _)| std::cmp::Reverse(*n));
    (no, dangling)
}
fn isupper(s: &str) -> bool {
    s.chars().any(char::is_uppercase) && !s.chars().any(char::is_lowercase)
}
fn cargo_pairs(text: &str) -> BTreeSet<(String, String)> {
    let mut out = BTreeSet::new();
    for b in text.split("[[package]]").skip(1) {
        let name = Regex::new(r#"(?m)^name = "(.+)""#).unwrap().captures(b);
        let ver = Regex::new(r#"(?m)^version = "(.+)""#).unwrap().captures(b);
        let src = Regex::new(r#"(?m)^source = "(.+)""#).unwrap().captures(b);
        if let (Some(n), Some(v), Some(_)) = (name, ver, src) {
            out.insert((n[1].into(), v[1].into()));
        }
    }
    out
}
fn crate_rows(v: &Value) -> Vec<(String, String, String, bool)> {
    v.get("crates")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|r| {
            Some((
                json_str(r, "name")?.into(),
                json_str(r, "version").unwrap_or("?").into(),
                json_str(r, "license")
                    .unwrap_or("?")
                    .trim_matches(['\'', '\"'])
                    .into(),
                r.get("ships")
                    .and_then(Value::as_bool)
                    .or_else(|| json_str(r, "ships").map(|x| x == "true"))
                    .unwrap_or(false),
            ))
        })
        .collect()
}
fn crate_drift(v: &Value, s: &AuditSnapshot, out: &mut Vec<String>) {
    let Some(lock) = s.file(CARGO_LOCK) else {
        return;
    };
    let want = cargo_pairs(lock);
    let rows = crate_rows(v);
    let have: BTreeSet<_> = rows
        .iter()
        .map(|(n, ver, _, _)| (n.clone(), ver.clone()))
        .collect();
    if !s.exists(CRATES_PATH) {
        out.push(format!("{CRATES_PATH} is MISSING — the Rust half of the registry is not recorded. Generate: tools/vendored.py --write-crates"));
    } else if want != have {
        let add = want
            .difference(&have)
            .map(|(n, v)| format!("{n} {v}"))
            .collect::<Vec<_>>()
            .join(", ");
        let gone = have
            .difference(&want)
            .map(|(n, v)| format!("{n} {v}"))
            .collect::<Vec<_>>()
            .join(", ");
        out.push(format!("{CRATES_PATH} is STALE against {CARGO_LOCK}.\n    in the lockfile but not recorded: {}\n    recorded but no longer in the lockfile: {}\n    Regenerate: tools/vendored.py --write-crates",if add.is_empty(){"none"}else{&add},if gone.is_empty(){"none"}else{&gone}));
    }
}
fn python_lines(text: &str) -> impl Iterator<Item = &str> {
    static BREAKS: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"\r\n|[\n\r\x0b\x0c\x1c-\x1e\x{85}\x{2028}\x{2029}]").unwrap()
    });
    BREAKS.split(text)
}
fn license_text(t: &Tree, s: &AuditSnapshot) -> Result<(String, String), String> {
    if let Some(p) = &t.license_file {
        let text = s
            .file(p)
            .ok_or_else(|| format!("{}: license_file is missing from snapshot: {p}", t.name))?;
        return Ok((
            p.clone(),
            python_lines(text)
                .map(str::trim_end)
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .into(),
        ));
    }
    if let Some(p) = &t.license_ref {
        let text = s
            .file(p)
            .ok_or_else(|| format!("{}: license_ref is missing from snapshot: {p}", t.name))?;
        let mut out = Vec::new();
        let mut in_block = false;
        for line in python_lines(text).take(200) {
            let stripped = line.trim();
            if in_block {
                out.push(REF_LEAD.replace(line, "").trim_end().to_string());
                if stripped.contains("*/") {
                    in_block = false;
                }
                continue;
            }
            if stripped.is_empty() {
                out.push(String::new());
                continue;
            }
            if stripped.starts_with("/*") {
                out.push(REF_LEAD.replace(line, "").trim_end().to_string());
                if !stripped[2..].contains("*/") {
                    in_block = true;
                }
                continue;
            }
            if stripped.starts_with("//") {
                out.push(REF_LEAD.replace(line, "").trim_end().to_string());
                continue;
            }
            break;
        }
        return Ok((
            format!("{p} (header comment)"),
            out.join("\n").trim().into(),
        ));
    }
    let note = t
        .license_note
        .as_deref()
        .ok_or_else(|| format!("{}: no license_file, license_ref or license_note", t.name))?;
    Ok(("NO GRANT IN TREE".into(), note.trim().into()))
}
fn anchor(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '-' | ' '))
        .map(|c| if c == ' ' { '-' } else { c })
        .collect()
}
fn notices(r: &Registry, s: &AuditSnapshot, crates: &Value) -> Result<String, String> {
    let mut out=vec!["# Third-party notices".into(),String::new(),"This product includes third-party software. **This file is GENERATED** from `vendor.yaml` by `tools/vendored.py --notices` — edit the registry, not this file. Each entry reproduces the licence grant as it appears in the tree.".into(),String::new(),format!("{} vendored C/C++ components:",r.trees.len()),String::new()];
    let mut trees = r.trees.iter().collect::<Vec<_>>();
    trees.sort_by_key(|t| t.name.to_lowercase());
    for t in &trees {
        let ver = version(t, s).unwrap_or_else(|_| "unknown".into());
        out.push(format!(
            "- [{}](#{}) — {ver} — {}",
            t.name,
            anchor(&t.name),
            t.license
        ));
    }
    out.push(String::new());
    for t in &trees {
        let ver = version(t, s).unwrap_or_else(|_| "unknown".into());
        let (source, text) = license_text(t, s)?;
        out.extend([
            String::new(),
            format!("## {}", t.name),
            String::new(),
            format!("- Upstream: {}", t.upstream),
            format!("- Version: {ver}"),
            format!("- In this tree: `{}`", t.prefixes().join("`, `")),
            format!("- Licence: {}", t.license),
        ]);
        for (l, w) in &t.license_extra {
            out.push(format!("- Also contains: {l} — {w}"));
        }
        if let Some(p) = &t.patch {
            out.push(format!("- Modified by us: yes, see `{p}`"));
        }
        out.push(if source != "NO GRANT IN TREE" {
            format!("- Grant reproduced from: `{source}`")
        } else {
            "- Grant reproduced from: **the tree carries no licence grant**".into()
        });
        out.extend([String::new(), "```".into(), text, "```".into()]);
    }
    out.push(String::new());
    let rows = crate_rows(crates);
    if !rows.is_empty() {
        let ships = rows.iter().filter(|r| r.3).count();
        out.extend(["## Rust crates".into(),String::new(),format!("This product also links {ships} third-party Rust crates, out of {} that enter the build. Generated from `cargo metadata` into `tools/rust-crates.yaml`; see that file for the ones used only to build or test, and for why the shipped/host split errs toward listing more.",rows.len()),String::new(),"| crate | version | licence |".into(),"| --- | --- | --- |".into()]);
        for (n, v, l, ship) in rows.into_iter().filter(|r| r.3) {
            out.push(format!("| {n} | {v} | {l} |"));
        }
        out.push(String::new());
    }
    Ok(out.join("\n"))
}

/// Render from the same immutable inputs as the audit's freshness assertion.
pub fn render_notices(
    registry: &Value,
    snapshot: &AuditSnapshot,
    crates: &Value,
) -> Result<String, String> {
    notices(&Registry::parse(registry)?, snapshot, crates)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn corpus() -> Corpus {
        Corpus::new(PathBuf::from("/fixture"), Vec::new())
    }
    fn file(s: &mut AuditSnapshot, p: &str, text: &str) {
        s.files.insert(p.into(), text.into());
        s.ordered_files.push(p.into());
    }
    fn dir(s: &mut AuditSnapshot, p: &str) {
        s.directories.insert(p.into());
        s.ordered_directories.push(p.into());
    }
    fn base_registry() -> Value {
        serde_json::json!({"trees":[{"path":"vendor/a","name":"A","reason":"fixture","license":"MIT","license_note":"none","translate":"keep"}],"ours":[],"containers":[]})
    }

    #[test]
    fn schema_requires_tree_fields_and_rejects_duplicate_or_overlapping_paths() {
        assert!(Registry::parse(&serde_json::json!({}))
            .unwrap_err()
            .contains("trees` is not a list"));
        assert!(
            Registry::parse(&serde_json::json!({"trees":[{"path":"x","name":"X"}]}))
                .unwrap_err()
                .contains("reason")
        );
        let dup = serde_json::json!({"trees":[{"path":"x","name":"X","reason":"r"},{"path":"x","name":"Y","reason":"r"}]});
        assert!(Registry::parse(&dup)
            .unwrap_err()
            .contains("duplicate path"));
        let overlap = serde_json::json!({"trees":[{"path":"vendor/x","name":"X","reason":"r"}],"ours":[{"path":"vendor/x/sub","name":"O","reason":"r"}]});
        assert!(Registry::parse(&overlap)
            .unwrap_err()
            .contains("also covered"));
    }

    #[test]
    fn exact_path_boundaries_and_owner_header_prefixes() {
        let r=Registry::parse(&serde_json::json!({"trees":[{"path":"vendor/x","headers":["sdk/include/x"],"name":"X","reason":"r"}]})).unwrap();
        assert!(r.is_vendored("vendor/x/a.c"));
        assert!(!r.is_vendored("vendor/xyz/a.c"));
        assert_eq!(r.owner("sdk/include/x/foo.h").unwrap().name, "X");
    }

    #[test]
    fn version_probe_literals_and_capture_groups_match_registry_contract() {
        let t=Registry::parse(&serde_json::json!({"trees":[{"path":"x","name":"X","reason":"r","version":{"file":"v.h","patterns":["#define A ([0-9]+)","#define B ([0-9]+)"],"join":"."}}]})).unwrap();
        let mut s = AuditSnapshot::default();
        file(&mut s, "v.h", "#define A 1\n#define B 2\n");
        assert_eq!(version(&t.trees[0], &s).unwrap(), "1.2");
        let err = py_regex("(?<=x)y").unwrap_err();
        assert!(err.contains("unsupported registry regex"));
    }

    #[test]
    fn license_drift_reports_machine_headers_but_spdx_suppresses_prose_fallback() {
        let r=Registry::parse(&serde_json::json!({"trees":[{"path":"vendor/a","name":"A","reason":"r","license":"MIT"}]})).unwrap();
        let mut s = AuditSnapshot::default();
        file(
            &mut s,
            "vendor/a/a.c",
            "/* SPDX-License-Identifier: GPL-2.0-only\n * GNU General Public License\n */\n",
        );
        file(&mut s, "vendor/a/b.h", "/* GNU General Public License */\n");
        let drift = license_drift(&r, &s);
        assert_eq!(drift.len(), 1);
        assert_eq!(drift[0].2.get("GPL-2.0-only"), Some(&1));
        assert_eq!(drift[0].2.get("GPL (prose)"), Some(&1));
    }

    #[test]
    fn declared_license_uses_python_prefix_boundary_quirk_and_floss_filter() {
        let r=Registry::parse(&serde_json::json!({"trees":[{"path":"vendor/a","name":"A","reason":"r"}],"ours":[{"path":"ours/one","paths":["ours/two"],"name":"O","reason":"r"}]})).unwrap();
        let mut s = AuditSnapshot::default();
        file(&mut s, "vendor/ab/x.c", "/* LICENSE: Proprietary */\n");
        file(&mut s, "ours/two/x.c", "/* LICENSE: Proprietary */\n");
        file(&mut s, "sdk/x.c", "/* LICENSE: MIT */\n");
        let got = declared_licences(&r, &s);
        // Raw `rel_dir.startswith("vendor/a")` intentionally excludes vendor/ab.
        assert_eq!(got, vec![("Proprietary".into(), "ours/two/x.c".into(), 1)]);
    }

    #[test]
    fn grant_resolves_only_at_or_above_referring_directory_and_respects_markers() {
        let r = Registry::parse(
            &serde_json::json!({"trees":[{"path":"vendor/a","name":"A","reason":"r"}],"ours":[]}),
        )
        .unwrap();
        let mut s = AuditSnapshot::default();
        dir(&mut s, "src");
        dir(&mut s, "elsewhere");
        file(
            &mut s,
            "src/a.c",
            "/* COPYRIGHT: x\n * LICENSE: BSD - see COPYING.BSD in the top level directory\n */\n",
        );
        file(&mut s, "elsewhere/COPYING.BSD", "grant\n");
        let (no, dangling) = grant_gaps(&r, &s);
        assert!(dangling.contains_key("copying.bsd"));
        assert_eq!(no.len(), 0); // the local pointer is a coverage claim regardless of resolution only if it resolves
    }

    #[test]
    fn audit_reports_missing_registered_path_stamp_and_unregistered_children() {
        let mut s = AuditSnapshot::default();
        dir(&mut s, "vendor");
        dir(&mut s, "vendor/new");
        file(
            &mut s,
            "vendor.yaml",
            "trees:\n  - path: vendor/a\n    name: A\n    reason: r\n",
        );
        let mut registry = base_registry();
        registry["containers"] = serde_json::json!(["vendor"]);
        let got = audit(&corpus(), &registry, &serde_json::json!({"crates":[]}), &s).unwrap();
        assert!(got
            .iter()
            .any(|x| x.contains("registered path does not exist")));
        assert!(got.iter().any(|x| x.contains("missing vendor/a/VENDORED")));
        assert!(got
            .iter()
            .any(|x| x.contains("UNREGISTERED vendored tree: vendor/new")));
    }

    #[test]
    fn marker_is_recorded_as_existence_and_blocks_descendant_scans() {
        let mut s = AuditSnapshot::default();
        dir(&mut s, "nested");
        dir(&mut s, "nested/check");
        s.markers.insert("nested/check/.git".into());
        assert!(unwalked(&s, "nested/check"));
        assert!(blocked(&s, "nested/check/inner"));
        assert!(s.exists("nested/check/.git"));
    }

    #[test]
    fn notices_render_license_grant_and_shipped_crates() {
        let r=Registry::parse(&serde_json::json!({"trees":[{"path":"vendor/a","name":"A","reason":"r","upstream":"u","license":"MIT","license_file":"vendor/a/LICENSE"}]})).unwrap();
        let mut s = AuditSnapshot::default();
        file(&mut s, "vendor/a/LICENSE", "MIT grant\n\x0c\nTerms\n");
        let crates = serde_json::json!({"crates":[{"name":"x","version":"1.0","license":"MIT","ships":true},{"name":"build-x","version":"2","license":"MIT","ships":false}]});
        let text = notices(&r, &s, &crates).unwrap();
        assert!(text.contains("Grant reproduced from: `vendor/a/LICENSE`"));
        assert!(text.contains("MIT grant\n\n\nTerms"));
        assert!(!text.contains('\x0c'));
        assert!(text.contains("| x | 1.0 | MIT |"));
        assert!(!text.contains("| build-x |"));
    }

    #[test]
    fn scanner_control_detects_unregistered_container_child_and_accepts_registered_good_fixture() {
        const YAML_BAD: &str = "containers: [libs]\ntrees:\n  - path: libs/known\n    name: Known\n    upstream: http://example.invalid/\n    license: MIT\n    license_note: fixture tree; no grant shipped\n    reason: a registered vendored tree\n";
        const YAML_GOOD: &str = "containers: [libs]\ntrees:\n  - path: libs/known\n    name: Known\n    upstream: http://example.invalid/\n    license: MIT\n    license_note: fixture tree; no grant shipped\n    reason: a registered vendored tree\n  - path: libs/surprise\n    name: Surprise\n    upstream: http://example.invalid/\n    license: MIT\n    license_note: fixture tree; no grant shipped\n    reason: the library that used to arrive unnoticed\n";
        let make_registry = |extra: bool| {
            let mut trees = vec![
                serde_json::json!({"path":"libs/known","name":"Known","upstream":"http://example.invalid/","license":"MIT","license_note":"fixture tree; no grant shipped","reason":"a registered vendored tree"}),
            ];
            if extra {
                trees.push(serde_json::json!({"path":"libs/surprise","name":"Surprise","upstream":"http://example.invalid/","license":"MIT","license_note":"fixture tree; no grant shipped","reason":"the library that used to arrive unnoticed"}));
            }
            serde_json::json!({"containers":["libs"],"trees":trees})
        };
        let mut bad = AuditSnapshot::default();
        for d in ["libs", "libs/known", "libs/surprise"] {
            dir(&mut bad, d);
        }
        file(&mut bad, "vendor.yaml", YAML_BAD);
        file(
            &mut bad,
            "libs/known/VENDORED",
            &stamp(&Registry::parse(&make_registry(false)).unwrap().trees[0]),
        );
        file(&mut bad, "libs/known/a.c", "int a;\n");
        file(&mut bad, "libs/surprise/b.c", "int b;\n");
        let got = audit(
            &corpus(),
            &make_registry(false),
            &serde_json::json!({"crates":[]}),
            &bad,
        )
        .unwrap();
        assert!(got
            .iter()
            .any(|p| p.contains("UNREGISTERED vendored tree: libs/surprise")));

        let mut good = AuditSnapshot::default();
        for d in ["libs", "libs/known", "libs/surprise"] {
            dir(&mut good, d);
        }
        file(&mut good, "vendor.yaml", YAML_GOOD);
        let gr = Registry::parse(&make_registry(true)).unwrap();
        file(&mut good, "libs/known/VENDORED", &stamp(&gr.trees[0]));
        file(&mut good, "libs/surprise/VENDORED", &stamp(&gr.trees[1]));
        file(&mut good, "libs/known/a.c", "int a;\n");
        file(&mut good, "libs/surprise/b.c", "int b;\n");
        let got = audit(
            &corpus(),
            &make_registry(true),
            &serde_json::json!({"crates":[]}),
            &good,
        )
        .unwrap();
        assert!(got.is_empty(), "{got:#?}");
    }
}
