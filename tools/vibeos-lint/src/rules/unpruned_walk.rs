//! Ratchet on recursive source walks that filter mutable output only after descent.
use crate::{Corpus, Rule, SourceSet};
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;
pub struct UnprunedWalk;
static INPUTS: &[SourceSet] = &[
    SourceSet::tree("unpruned_python", &["tools", "tests"], &["py"], &[])
        .with_pruning(&["target"])
        .with_inputs(),
    SourceSet::tracked("unpruned_baseline", &[], &["scan_unpruned_walk.txt"]),
];
impl Rule for UnprunedWalk {
    fn name(&self) -> &'static str {
        "scan_unpruned_walk"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let mut found = Vec::<String>::new();
        let mut files: Vec<_> = c.sources("unpruned_python").collect();
        files.sort_by_key(|f| {
            (
                if f.path.starts_with("tools") { 0 } else { 1 },
                f.path.clone(),
            )
        });
        for f in files {
            if f.path.extension().map_or(true, |ext| ext != "py") {
                continue;
            }
            let name = f.path.file_name().unwrap_or_default().to_string_lossy();
            if [
                "rust_sources.py",
                "scan_unpruned_walk.py",
                "test_scanner_controls.py",
            ]
            .contains(&name.as_ref())
                || f.path.to_string_lossy().contains("/target/")
            {
                continue;
            }
            for (i, line) in f.text.lines().enumerate() {
                if line.contains("unpruned-walk: intentional") {
                    continue;
                }
                let kind = if WALK.is_match(line) {
                    Some("unpruned rust walk")
                } else if BUILD.is_match(line) {
                    Some("unpruned build walk")
                } else {
                    None
                };
                if let Some(k) = kind {
                    found.push(format!(
                        "{}:{}: {k}: {}",
                        f.path.display(),
                        i + 1,
                        line.trim()
                    ));
                }
            }
        }
        let baseline = c
            .sources("unpruned_baseline")
            .find(|f| f.path == std::path::Path::new("tools/lint-baseline/scan_unpruned_walk.txt"))
            .map(|f| f.text.as_str())
            .unwrap_or("");
        let keys: HashSet<_> = baseline
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty() && !s.starts_with('#'))
            .filter_map(|s| key(s))
            .collect();
        Ok(found
            .into_iter()
            .filter(|row| key(row).map_or(true, |k| !keys.contains(&k)))
            .collect())
    }
}
static WALK: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r#"\.(?:rglob\(\s*["']\*\.rs["']|glob\(\s*["']\*\*/\*\.rs["'])"#)
        .unwrap()
});
static BUILD: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r#"\b\w*build\w*\s*\.\s*(?:rglob\(|glob\(\s*["']\*\*/)"#).unwrap()
});
fn key(row: &str) -> Option<String> {
    let (file, rest) = row.split_once(':')?;
    let (_, offense) = rest.split_once(": ")?;
    Some(format!("{file}:{offense}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Corpus, SourceFile};
    use std::path::PathBuf;
    #[test]
    fn original_scanner_control_bad_and_good() {
        let bad="from pathlib import Path\n\ndef collect(root: Path):\n    for p in root.rglob(\"*.rs\"):\n        if \"/target/\" in str(p):\n            continue\n        yield p\n\n\ndef collect_other(root: Path):\n    for p in root.glob(\"**/*.rs\"):\n        yield p\n\n\ndef find_lib(build: Path, want: str):\n    # The BUILD-dir half of the class: not a Rust walk at all, and the post-filter\n    # names the very tree the descent must not enter.\n    for p in build.rglob(want):\n        if \"-target/\" not in str(p):\n            return p\n    return None\n";
        let good="import os\nfrom pathlib import Path\n\nfrom source_walk import source_files\n\ndef collect(root: Path):\n    yield from source_files(root)\n\ndef collect_pyfiles(root: Path):\n    # Not a Rust walk and not rooted at a build dir, so it crosses no mutable tree\n    # and is not this lint's business. Both halves of that matter: the same spelling\n    # rooted at a BUILD dir is a finding (see find_lib in the bad fixture).\n    yield from root.rglob(\"*.py\")\n\ndef find_lib(build: Path, want: str):\n    # Pruned during the descent, so the churning tree is never entered.\n    for dirpath, dirnames, filenames in os.walk(build):\n        dirnames[:] = [d for d in dirnames if not d.endswith(\"-target\")]\n        if want in filenames:\n            return Path(dirpath) / want\n    return None\n\ndef deliberate_control(root: Path):\n    return root.rglob(\"*.rs\")  # unpruned-walk: intentional\n";
        let c = Corpus::new(
            PathBuf::from("."),
            vec![
                SourceFile::new("walker.py".into(), bad.into()),
                SourceFile::new(
                    "tools/lint-baseline/scan_unpruned_walk.txt".into(),
                    String::new(),
                ),
            ],
        );
        let x = UnprunedWalk.run(&c).unwrap();
        assert!(x
            .iter()
            .any(|s| s.contains("walker.py:4: unpruned rust walk")));
        assert!(x
            .iter()
            .any(|s| s.contains("walker.py:18: unpruned build walk")));
        let goodc = Corpus::new(
            PathBuf::from("."),
            vec![
                SourceFile::new("walker.py".into(), good.into()),
                SourceFile::new(
                    "tools/lint-baseline/scan_unpruned_walk.txt".into(),
                    String::new(),
                ),
            ],
        );
        assert!(UnprunedWalk.run(&goodc).unwrap().is_empty());
    }
    #[test]
    fn baseline_key_ignores_shifted_line_but_keeps_offending_text() {
        let c = Corpus::new(
            PathBuf::from("."),
            vec![
                SourceFile::new(
                    "tools/walker.py".into(),
                    "\nfor p in root.rglob(\"*.rs\"):\n".into(),
                ),
                SourceFile::new(
                    "tools/lint-baseline/scan_unpruned_walk.txt".into(),
                    "tools/walker.py:391: unpruned rust walk: for p in root.rglob(\"*.rs\"):\n"
                        .into(),
                ),
            ],
        );
        assert!(UnprunedWalk.run(&c).unwrap().is_empty());
    }
}
