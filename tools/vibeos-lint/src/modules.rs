//! Source-only module reachability, preserving tools/rust_module_tree.py's grammar.
use crate::SourceFile;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use std::sync::LazyLock;

static MODULE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?\bmod\s+(?:r#)?(\w+)\s*([;{])")
        .unwrap()
});
static ATTR_MODULE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"((?:#\[[^\]]*\]\s*)+)(?:pub(?:\([^)]*\))?\s+)?mod\s+(?:r#)?(\w+)\s*;").unwrap()
});
static ATTR_PATH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\bpath\s*=\s*"([^"]+)""#).unwrap());
static INCLUDE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"include!\s*\(\s*"([^"]+)"\s*\)"#).unwrap());
static OPAQUE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"include!\s*\(\s*(?:env|concat)!").unwrap());
static KEYWORD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bmod\b|include").unwrap());
static CARGO_SECTION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^\s*\[").unwrap());
static CARGO_PATH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?m)^\s*path\s*=\s*"([^"]+)""#).unwrap());

pub fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if out.file_name().is_some_and(|n| n != "..") {
                    out.pop();
                } else if !out.has_root() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Comments disappear; quoted strings remain verbatim, as in the legacy walk.
pub fn strip_comments(text: &str) -> String {
    let bytes = text.as_bytes();
    let (mut pos, mut start) = (0, 0);
    let mut out = String::new();
    while pos < bytes.len() {
        if bytes[pos] == b'"' {
            pos += 1;
            while pos < bytes.len() {
                if bytes[pos] == b'\\' {
                    pos = (pos + 2).min(bytes.len());
                } else if bytes[pos] == b'"' {
                    pos += 1;
                    break;
                } else {
                    pos += 1;
                }
            }
        } else if bytes[pos..].starts_with(b"//") {
            out.push_str(&text[start..pos]);
            pos = bytes[pos..]
                .iter()
                .position(|&b| b == b'\n')
                .map_or(bytes.len(), |n| pos + n);
            start = pos;
        } else if bytes[pos..].starts_with(b"/*") {
            out.push_str(&text[start..pos]);
            pos += 2;
            let mut depth = 1;
            while pos < bytes.len() && depth > 0 {
                if bytes[pos..].starts_with(b"/*") {
                    depth += 1;
                    pos += 2;
                } else if bytes[pos..].starts_with(b"*/") {
                    depth -= 1;
                    pos += 2;
                } else {
                    pos += 1;
                }
            }
            out.push(' ');
            start = pos;
        } else {
            pos += 1;
        }
    }
    out.push_str(&text[start..]);
    out
}

#[derive(Default)]
struct Facts {
    declarations: Vec<(String, Vec<String>)>,
    paths: Vec<(String, Vec<String>)>,
    includes: Vec<String>,
    opaque: bool,
}
fn facts(raw: &str) -> Facts {
    if !KEYWORD.is_match(raw) {
        return Facts::default();
    }
    let code = strip_comments(raw);
    let mut out = Facts {
        opaque: OPAQUE.is_match(&code),
        ..Facts::default()
    };
    let mut pos = 0;
    let mut depth = 0isize;
    let mut stack: Vec<(String, isize)> = Vec::new();
    for matched in MODULE.captures_iter(&code) {
        let whole = matched.get(0).unwrap();
        for ch in code[pos..whole.start()].bytes() {
            if ch == b'{' {
                depth += 1;
            } else if ch == b'}' {
                depth -= 1;
                while stack.last().is_some_and(|(_, at)| *at >= depth) {
                    stack.pop();
                }
            }
        }
        if &matched[2] == ";" {
            out.declarations.push((
                matched[1].into(),
                stack.iter().map(|(name, _)| name.clone()).collect(),
            ));
        } else {
            depth += 1;
            stack.push((matched[1].into(), depth));
        }
        pos = whole.end();
    }
    for matched in ATTR_MODULE.captures_iter(&code) {
        let paths: Vec<_> = ATTR_PATH
            .captures_iter(&matched[1])
            .map(|m| m[1].to_owned())
            .collect();
        if !paths.is_empty() {
            out.paths.push((matched[2].into(), paths));
        }
    }
    out.includes = INCLUDE
        .captures_iter(&code)
        .map(|m| m[1].to_owned())
        .collect();
    out
}

pub struct ModuleTree {
    owners: HashMap<PathBuf, PathBuf>,
    live: HashMap<PathBuf, HashSet<PathBuf>>,
    opaque: HashSet<PathBuf>,
    empty: HashSet<PathBuf>,
}
impl ModuleTree {
    pub fn from_files(files: &[SourceFile]) -> Self {
        let by_path: HashMap<_, _> = files.iter().map(|f| (normalize(&f.path), f)).collect();
        let manifests: HashMap<_, _> = files
            .iter()
            .filter(|f| f.path.file_name().is_some_and(|n| n == "Cargo.toml"))
            .map(|f| (normalize(f.path.parent().unwrap_or(Path::new(""))), f))
            .collect();
        let mut owners = HashMap::new();
        for file in files {
            if let Some(owner) = file
                .path
                .ancestors()
                .skip(1)
                .map(normalize)
                .find(|p| manifests.contains_key(p))
            {
                owners.insert(normalize(&file.path), owner);
            }
        }
        let mut live = HashMap::new();
        let mut opaque = HashSet::new();
        for (directory, manifest) in &manifests {
            let mut queue = Vec::new();
            for name in ["src/lib.rs", "src/main.rs"] {
                let target = normalize(&directory.join(name));
                if by_path.contains_key(&target) {
                    queue.push(target);
                }
            }
            let sections: Vec<_> = CARGO_SECTION
                .find_iter(&manifest.text)
                .map(|m| m.end())
                .collect();
            for (i, &start) in sections.iter().enumerate() {
                let end = sections
                    .get(i + 1)
                    .copied()
                    .map_or(manifest.text.len(), |n| n - 1);
                let block = &manifest.text[start..end];
                let Some((head, body)) = block.split_once(']') else {
                    continue;
                };
                let head = head
                    .trim()
                    .trim_matches('[')
                    .split('.')
                    .next()
                    .unwrap_or("");
                if ["lib", "bin", "test", "bench", "example"].contains(&head) {
                    for target in CARGO_PATH.captures_iter(body) {
                        let target = normalize(&directory.join(&target[1]));
                        if by_path.contains_key(&target) {
                            queue.push(target);
                        }
                    }
                }
            }
            let mut reached = HashSet::new();
            let mut real = HashSet::new();
            while let Some(path) = queue.pop() {
                if reached.contains(&path) {
                    continue;
                }
                let Some(file) = by_path.get(&path) else {
                    continue;
                };
                reached.insert(path.clone());
                real.insert(normalize(file.resolved_path()));
                let facts = facts(&file.text);
                if facts.opaque {
                    opaque.insert(directory.clone());
                }
                let parent = path.parent().unwrap_or(Path::new(""));
                let stem = path.file_stem().unwrap_or_default();
                let base = if ["lib", "main", "mod"].iter().any(|n| stem == *n) {
                    parent.to_owned()
                } else {
                    parent.join(stem)
                };
                let enclosures: HashMap<_, _> = facts
                    .declarations
                    .iter()
                    .map(|(name, enclosing)| (name.as_str(), enclosing))
                    .collect();
                for (name, paths) in facts.paths {
                    let enclosing = enclosures.get(name.as_str());
                    let mut path_dir = parent.to_owned();
                    if let Some(enclosing) = enclosing.filter(|e| !e.is_empty()) {
                        path_dir = base.clone();
                        for name in *enclosing {
                            path_dir.push(name);
                        }
                    }
                    for target in paths {
                        queue.push(normalize(&path_dir.join(target)));
                    }
                }
                for target in facts.includes {
                    queue.push(normalize(&parent.join(target)));
                }
                for (name, enclosing) in facts.declarations {
                    let mut path_dir = base.clone();
                    for name in enclosing {
                        path_dir.push(name);
                    }
                    for target in [
                        path_dir.join(format!("{name}.rs")),
                        path_dir.join(&name).join("mod.rs"),
                    ] {
                        let target = normalize(&target);
                        if by_path.contains_key(&target) {
                            queue.push(target);
                            break;
                        }
                    }
                }
            }
            live.insert(directory.clone(), real);
        }
        Self {
            owners,
            live,
            opaque,
            empty: HashSet::new(),
        }
    }
    pub fn crate_root_of(&self, path: &Path) -> Option<&Path> {
        self.owners.get(&normalize(path)).map(PathBuf::as_path)
    }
    pub fn live_files(&self, directory: &Path) -> &HashSet<PathBuf> {
        self.live.get(&normalize(directory)).unwrap_or(&self.empty)
    }
    pub fn crate_is_opaque(&self, directory: &Path) -> bool {
        self.opaque.contains(&normalize(directory))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn file(path: &str, text: &str) -> SourceFile {
        SourceFile::new(path.into(), text.into())
    }
    #[test]
    fn comments_preserve_strings_and_nested_delimiters() {
        assert_eq!(
            strip_comments(
                "// path/*/multiboot.S\npub mod live; /* nested /* é */ done */\n\"http://x/*\""
            ),
            "\npub mod live;  \n\"http://x/*\""
        );
        assert_eq!(strip_comments("α /* unterminated"), "α  ");
        assert_eq!(strip_comments("\"unterminated\\"), "\"unterminated\\");
    }
    #[test]
    fn raw_names_inline_modules_cfg_paths_and_includes() {
        let files = vec![file("x/Cargo.toml", "[package]\nname=\"x\""),
            file("x/src/lib.rs", "// path/*/asm\npub mod r#async; pub mod outer { #[path=\"special.rs\"] pub mod custom; pub mod other; }\n#[cfg_attr(target_arch=\"x86\", path=\"a.rs\")]\n#[cfg_attr(target_arch=\"aarch64\", path=\"b.rs\")] mod generated;\ninclude!(\"extra.rs\");"),
            file("x/src/async.rs", ""), file("x/src/outer/special.rs", ""), file("x/src/outer/other/mod.rs", ""),
            file("x/src/a.rs", ""), file("x/src/b.rs", ""), file("x/src/extra.rs", ""), file("x/src/dead.rs", "")];
        let tree = ModuleTree::from_files(&files);
        let live = tree.live_files(Path::new("x"));
        assert_eq!(live.len(), 7);
        assert!(live.contains(Path::new("x/src/outer/special.rs")));
        assert!(!live.contains(Path::new("x/src/dead.rs")));
        assert_eq!(
            tree.crate_root_of(Path::new("x/src/dead.rs")),
            Some(Path::new("x"))
        );
    }
    #[test]
    fn cargo_target_paths_skip_dependencies_and_generated_includes_are_opaque() {
        let files = vec![
            file(
                "x/Cargo.toml",
                "[dependencies.y]\npath = \"dependency.rs\"\n[[bin]]\npath = \"entry.rs\"\n",
            ),
            file("x/entry.rs", "include!(env!(\"OUT_RS\"));"),
            file("x/dependency.rs", ""),
        ];
        let tree = ModuleTree::from_files(&files);
        assert_eq!(
            tree.live_files(Path::new("x")),
            &HashSet::from([PathBuf::from("x/entry.rs")])
        );
        assert!(tree.crate_is_opaque(Path::new("x")));
    }
}
