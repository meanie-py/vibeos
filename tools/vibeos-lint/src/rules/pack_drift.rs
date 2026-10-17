//! A C pshpack region whose translated public Rust struct has lost packing.
use crate::{Corpus, Rule, SourceSet};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

pub struct PackDrift;
static FLIP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"#\s*\S+\.(?:c|cpp)\s+#\s*replaced by Rust \((sdk/lib/rust/[\w.-]+)").unwrap()
});
static REGION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)#include\s*[<"]pshpack(\d)\.h[>"](.*?)#include\s*[<"]poppack\.h[>"]"#)
        .unwrap()
});
static TYPEDEF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)typedef\s+struct\s+(\w+)?\s*\{.*?\}\s*(\w+)").unwrap());
static TAGGED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bstruct\s+(\w+)\s*\{").unwrap());
static COMMENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/|//[^\n]*").unwrap());
fn packed(text: &str) -> Vec<(String, BTreeSet<String>)> {
    let text = COMMENT.replace_all(text, "");
    let mut out = Vec::new();
    for region in REGION.captures_iter(&text) {
        for item in TYPEDEF.captures_iter(&region[2]) {
            out.push((
                region[1].to_owned(),
                [item.get(1), item.get(2)]
                    .into_iter()
                    .flatten()
                    .map(|m| m.as_str().to_owned())
                    .collect(),
            ));
        }
        for item in TAGGED.captures_iter(&region[2]) {
            if !region[2][..item.get(0).unwrap().start()].ends_with("typedef ") {
                out.push((region[1].to_owned(), BTreeSet::from([item[1].to_owned()])));
            }
        }
    }
    out
}
impl Rule for PackDrift {
    fn name(&self) -> &'static str {
        "scan_pack_drift"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        const SETS: &[SourceSet] = &[
            SourceSet::tree("pack-meta", &[""], &[], &["CMakeLists.txt"])
                .with_pruning(&["target", ".*", "build-*", "output-*"]),
            SourceSet::tree("pack-c", &[""], &["c", "h"], &[]),
        ];
        SETS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let mut modules = BTreeMap::<PathBuf, BTreeSet<PathBuf>>::new();
        for meta in c.sources("pack-meta").filter(|f| {
            f.path.file_name().is_some_and(|n| n == "CMakeLists.txt")
                && !f.path.starts_with("vendor")
        }) {
            for hit in FLIP.captures_iter(&meta.text) {
                let crate_dir = PathBuf::from(&hit[1]);
                if c.rust_files().any(|f| f.path.starts_with(&crate_dir)) {
                    modules
                        .entry(meta.path.parent().unwrap_or(Path::new("")).to_owned())
                        .or_default()
                        .insert(crate_dir);
                }
            }
        }
        let candidates: Vec<_> = c
            .sources("pack-c")
            .filter(|f| {
                f.path
                    .extension()
                    .is_some_and(|ext| ext == "c" || ext == "h")
                    && f.text.contains("pshpack")
            })
            .collect();
        let mut out = Vec::new();
        for (module, crates) in modules {
            let rust = c
                .rust_files()
                .filter(|f| {
                    crates.iter().any(|dir| f.path.starts_with(dir))
                        && !f.path.file_name().is_some_and(|n| {
                            n.to_string_lossy().starts_with("bindgen_")
                                || n.to_string_lossy().starts_with("island_")
                        })
                })
                .map(|f| f.text.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            for file in candidates.iter().filter(|f| f.path.starts_with(&module)) {
                for (pack, names) in packed(&file.text) {
                    for name in names {
                        let pattern = crate::regexes::cached(&format!(
                            r"#\[repr\(([^)\]]*)\)\]\s*(?:#\[[^\]]*\]\s*)*pub struct {}\b",
                            regex::escape(&name)
                        ))
                        .map_err(|e| e.to_string())?;
                        for hit in pattern.captures_iter(&rust) {
                            if !hit[1].contains("packed") {
                                out.push(format!("{}/{}: pack-drift {name} is pshpack{pack} in the C, #[repr({})] in the Rust",module.display(),file.path.file_name().unwrap().to_string_lossy(),&hit[1]));
                            }
                        }
                    }
                }
            }
        }
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceFile;
    #[test]
    fn original_partition_sector_bad_and_packed_controls() {
        for (repr, count) in [("C", 1), ("C, packed", 0)] {
            let c=Corpus::new("/repo".into(),vec![
                SourceFile::new("base/system/dp/CMakeLists.txt".into(),"list(APPEND SOURCE\n # partlist.c # replaced by Rust (sdk/lib/rust/dp-rust/src/partlist.rs)\n)".into()),
                SourceFile::new("base/system/dp/partlist.c".into(),"#include <pshpack1.h>\ntypedef struct _PARTITION_SECTOR { UCHAR BootCode[440]; ULONG Signature; } PARTITION_SECTOR, *PPARTITION_SECTOR;\n#include <poppack.h>\ntypedef struct _LOOSE { ULONG a; } LOOSE;".into()),
                SourceFile::new("sdk/lib/rust/dp-rust/src/partlist.rs".into(),format!("#[repr({repr})]\npub struct _PARTITION_SECTOR {{ pub BootCode: [u8;440] }}\n#[repr(C)] pub struct LOOSE {{pub a:u32}}"))]);
            let rows = PackDrift.run(&c).unwrap();
            assert_eq!(rows.len(), count);
            if count == 1 {
                assert_eq!(rows[0],"base/system/dp/partlist.c: pack-drift _PARTITION_SECTOR is pshpack1 in the C, #[repr(C)] in the Rust");
            }
        }
    }
    #[test]
    fn comments_and_tag_only_structs() {
        assert!(packed("// #include <pshpack1.h>\nstruct X {};\n#include <poppack.h>").is_empty());
        assert_eq!(
            packed("#include <pshpack2.h>\nstruct X {};\n#include <poppack.h>")[0].0,
            "2"
        );
    }
}
