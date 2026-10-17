//! Pinned-stable source ratchet.
use crate::{Corpus, Rule, SourceSet};
pub struct NightlySurface;
static INPUTS: &[SourceSet] = &[
    SourceSet::tree("nightly_rust", &["sdk/lib/rust"], &["rs"], &[])
        .with_pruning(&["target", ".git", ".build-tools", "build-*"])
        .with_inputs(),
    SourceSet::tree("nightly_cmake", &[""], &["cmake"], &["CMakeLists.txt"]).with_pruning(&[
        "target",
        ".git",
        ".build-tools",
        "build-*",
    ]),
];
impl Rule for NightlySurface {
    fn name(&self) -> &'static str {
        "scan_nightly_surface"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let feature = crate::regexes::cached(r"^\s*#!\[feature\(").unwrap();
        let boot = crate::regexes::cached("RUSTC_BOOTSTRAP").unwrap();
        let mut out = Vec::new();
        for f in c.sources("nightly_rust").filter(|f| {
            f.path.extension().is_some_and(|e| e == "rs") && f.path.starts_with("sdk/lib/rust")
        }) {
            for (i, line) in f.text.lines().enumerate() {
                if feature.is_match(line) {
                    out.push(format!(
                        "{}:{}: [NIGHTLY] {} — the tree is on pinned stable",
                        f.path.display(),
                        i + 1,
                        line.trim()
                    ));
                }
            }
        }
        // CMake snapshots are centrally added to Corpus by the parent; don't silently read disk.
        for f in c.sources("nightly_cmake").filter(|f| {
            f.path.file_name().is_some_and(|n| n == "CMakeLists.txt")
                || f.path.extension().is_some_and(|e| e == "cmake")
        }) {
            let p = f.path.to_string_lossy();
            for (i, line) in f.text.lines().enumerate() {
                if line.trim_start().starts_with('#') {
                    continue;
                }
                if boot.is_match(line) {
                    out.push(format!(
                        "{}:{}: [NIGHTLY] RUSTC_BOOTSTRAP — stable-rustc escape hatch",
                        p,
                        i + 1
                    ));
                }
            }
        }
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Corpus, SourceFile};
    use std::path::PathBuf;
    #[test]
    fn rust_feature_reports_line() {
        let c = Corpus::new(
            PathBuf::from("."),
            vec![SourceFile::new(
                "sdk/lib/rust/x.rs".into(),
                "// #![feature(foo)]\n#![feature(c_variadic)]\n".into(),
            )],
        );
        let x = NightlySurface.run(&c).unwrap();
        assert_eq!(x.len(), 1);
        assert!(x[0].contains(":2: [NIGHTLY]"));
    }
    #[test]
    fn cmake_comment_is_ignored() {
        let c = Corpus::new(
            PathBuf::from("."),
            vec![SourceFile::new(
                "x/CMakeLists.txt".into(),
                "# RUSTC_BOOTSTRAP=1\nset(RUSTC_BOOTSTRAP 1)\n".into(),
            )],
        );
        assert_eq!(NightlySurface.run(&c).unwrap().len(), 1);
    }
}
