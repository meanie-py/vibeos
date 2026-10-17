//! Unreachable, rootless and phantom modules, plus stale orphan exemption banners.
use crate::{modules::normalize, Corpus, Rule};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub struct OrphanModules;
const BANNER: &str = "ORPHAN-BY-DESIGN:";
fn banner(text: &str) -> bool {
    text.lines().take(40).any(|line| line.contains(BANNER))
}
fn has_root(directory: &Path, paths: &HashSet<PathBuf>) -> bool {
    ["src/lib.rs", "src/main.rs", "lib.rs", "main.rs"]
        .iter()
        .any(|name| paths.contains(&directory.join(name)))
}
fn skipped(path: &Path, paths: &HashSet<PathBuf>, hand: bool) -> bool {
    if path.file_name().is_some_and(|n| n == "build.rs") {
        return true;
    }
    for ancestor in path.ancestors().skip(1) {
        if ancestor.file_name().is_some_and(|name| {
            ["tests", "benches", "examples", "target", ".git"]
                .iter()
                .any(|skip| name == *skip)
        }) {
            return true;
        }
        if hand && ancestor.file_name().is_some_and(|name| name == "hand") {
            let parent = ancestor.parent().unwrap_or(Path::new(""));
            if paths.contains(&parent.join("Cargo.toml"))
                && paths.iter().any(|p| p.starts_with(parent.join("src")))
            {
                return true;
            }
        }
    }
    false
}
impl Rule for OrphanModules {
    fn name(&self) -> &'static str {
        "scan_orphan_modules"
    }
    fn needs_module_tree(&self) -> bool {
        true
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let tree = corpus.module_tree();
        let paths: HashSet<_> = corpus.files.iter().map(|f| normalize(&f.path)).collect();
        let mut out = Vec::new();
        for file in corpus.rust_files() {
            if skipped(&file.path, &paths, true) {
                continue;
            }
            let crate_dir = tree.crate_root_of(&file.path);
            if crate_dir.is_none_or(|dir| !has_root(dir, &paths)) {
                if !banner(&file.text) {
                    out.push(format!("{}:1: [ROOTLESS] no crate root (lib.rs/main.rs) above this file — nothing compiles it; scaffold the crate, or declare it with a `//! {BANNER} <reason>` banner", file.path.display()));
                }
                continue;
            }
            let crate_dir = crate_dir.unwrap();
            if tree.crate_is_opaque(crate_dir) {
                continue;
            }
            let live = tree.live_files(crate_dir);
            if live.is_empty() {
                continue;
            }
            if live.contains(&normalize(file.resolved_path())) {
                if banner(&file.text) {
                    out.push(format!("{}:1: [STALE-BANNER] a `mod` chain DOES reach this file, so its `{BANNER}` banner is false — and scan_extern_arity, scan_cc_mismatch and scan_silent_failures all skip a file on sight of that banner. Delete the banner now that the file is wired.",file.path.display()));
                }
            } else if !banner(&file.text) {
                out.push(format!("{}:1: [ORPHAN] no `mod` chain reaches this file — {} lines that nothing compiles; wire it up, or declare it with a `//! {BANNER} <reason>` banner",file.path.display(),file.text.lines().count()));
            }
        }
        for manifest in corpus.cargo_manifests() {
            if skipped(&manifest.path, &paths, false) {
                continue;
            }
            let directory = manifest.path.parent().unwrap_or(Path::new(""));
            if !has_root(directory, &paths) || tree.crate_is_opaque(directory) {
                continue;
            }
            for (decl, name, candidates) in tree.phantom_modules(directory, &corpus.files) {
                let where_ = candidates
                    .iter()
                    .map(|p| p.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join(" | ");
                out.push(format!("{}:1: [PHANTOM] `mod {name};` resolves to no file (looked for {where_}) — write it, or drop the declaration",decl.display()));
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceFile;
    fn corpus(files: &[(&str, &str)]) -> Corpus {
        Corpus::new(
            "/repo".into(),
            files
                .iter()
                .map(|(p, t)| SourceFile::new((*p).into(), (*t).into()))
                .collect(),
        )
    }
    #[test]
    fn orphan_parked_stale_and_membership_are_snapshot_local() {
        let mut files = vec![
            ("x/Cargo.toml", "[package]\nname=\"x\""),
            ("x/src/lib.rs", "mod live;"),
            ("x/src/live.rs", "//! ORPHAN-BY-DESIGN: stale assertion\n"),
            ("x/src/orphan.rs", "fn unused() {}\n"),
            (
                "x/src/parked.rs",
                "//! ORPHAN-BY-DESIGN: deliberately parked\n",
            ),
        ];
        let rows = OrphanModules.run(&corpus(&files)).unwrap();
        assert_eq!(rows.len(), 2);
        assert!(rows
            .iter()
            .any(|r| r.starts_with("x/src/orphan.rs:1: [ORPHAN]")));
        assert!(rows
            .iter()
            .any(|r| r.starts_with("x/src/live.rs:1: [STALE-BANNER]")));
        files[1].1 = "mod live; mod orphan;";
        assert_eq!(OrphanModules.run(&corpus(&files)).unwrap().len(), 1);
    }
    #[test]
    fn phantom_inline_include_and_opaque_roots() {
        let c = corpus(&[
            ("x/Cargo.toml", ""),
            (
                "x/src/lib.rs",
                "mod live; mod missing; include!(\"included.rs\");",
            ),
            ("x/src/live.rs", "fn live(){}"),
            ("x/src/included.rs", "mod second_missing;"),
        ]);
        let rows = OrphanModules.run(&c).unwrap();
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(|r| r.contains("[PHANTOM]")));
        let opaque = corpus(&[
            ("x/Cargo.toml", ""),
            ("x/src/lib.rs", "include!(env!(\"GENERATED\"));"),
            ("x/src/orphan.rs", "fn unused(){}"),
        ]);
        assert!(OrphanModules.run(&opaque).unwrap().is_empty());
    }
    #[test]
    fn rootless_hand_overlay_test_dirs_and_banner_window() {
        let c = corpus(&[
            ("workspace/Cargo.toml", "[workspace]"),
            ("workspace/dump/a.rs", "fn unused(){}"),
            ("x/Cargo.toml", ""),
            ("x/src/lib.rs", ""),
            ("x/hand/a.rs", "fn overlay(){}"),
            ("x/tests/a.rs", "fn test(){}"),
            ("x/src/parked.rs", "//! ORPHAN-BY-DESIGN: parked"),
        ]);
        let rows = OrphanModules.run(&c).unwrap();
        assert_eq!(rows.len(), 1);
        assert!(rows[0].contains("[ROOTLESS]"));
        assert!(!banner(&format!("{}//! {BANNER} late", "\n".repeat(40))));
        assert!(banner(&format!("{}//! {BANNER}", "\n".repeat(39))));
    }
}
