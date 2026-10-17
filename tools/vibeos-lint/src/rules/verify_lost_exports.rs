//! Detect spec exports whose only implementation is in a C/C++ file flipped out of CMake.
use crate::{Corpus, Rule, SourceFile, SourceSet};
use regex::Regex;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::LazyLock,
};

pub struct VerifyLostExports;
const INPUTS: &[SourceSet] = &[
    SourceSet::tree("lost-crates", &["sdk/lib/rust"], &["rs"], &["Cargo.toml"]).with_directories(),
    SourceSet::tree(
        "lost-dll",
        &["dll"],
        &["c", "cpp", "spec", "S", "s", "asm"],
        &["CMakeLists.txt"],
    )
    .with_directories()
    .with_pruning(&[]),
];
static ENTRY: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"(?m)^\s*(?:@|\d+)\s+(?:stdcall|cdecl|varargs|thiscall|fastcall)\s+(?:-\S+[ \t]+)*([A-Za-z_]\w*)[ \t]*\([^)]*\)(?:[ \t]+(\S+))?[ \t]*(?:#.*)?$").unwrap()
});
static COMMENTED: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"(?m)^\s*#\s*(?:\$\{\w+\}/)?([\w./-]+\.c(?:pp)?)\b").unwrap()
});
static LIVE: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"(?m)^[^#\n]*?(?:\$\{\w+\}/)?([\w./-]+\.c(?:pp)?)\b").unwrap()
});
static RUST_FN: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r#"#\[no_mangle\](?:[ \t]*(?://[^\n]*)?\n(?:[ \t]*(?://[^\n]*|#\[[^\n]*\])?\n)*)?[ \t]*(?:pub(?:\([^)]*\))?[ \t]+)?(?:unsafe[ \t]+)?(?:extern[ \t]+(?:\"\w+\"|\$\w+)[ \t]+)?fn[ \t]+(\w+)"#).unwrap()
});
static IDENT: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"\b([A-Za-z_]\w*)\b").unwrap());

fn dead_regions(text: &str) -> Vec<(usize, usize)> {
    let re =
        crate::regexes::cached(r"(?m)^\s*#\s*(if\s+0\b|if\b|ifdef\b|ifndef\b|endif\b)").unwrap();
    let mut out = vec![];
    let mut stack: Vec<isize> = vec![];
    for m in re.captures_iter(text) {
        let x = m.get(1).unwrap();
        let k = x.as_str();
        if k.starts_with("if 0") {
            stack.push(x.start() as isize)
        } else if k.starts_with("if") && !k.starts_with("endif") {
            if !stack.is_empty() {
                stack.push(-1)
            }
        } else if k.starts_with("endif") && !stack.is_empty() {
            let begin = stack.pop().unwrap();
            if stack.is_empty() && begin >= 0 {
                out.push((begin as usize, m.get(0).unwrap().end()))
            }
        }
    }
    out
}
fn defined_live(text: &str, sym: &str) -> bool {
    let pat = format!(
        r"(?m)^(?:[A-Za-z_][^;()\n]*?[\s*])?{}\s*\(",
        regex::escape(sym)
    );
    let Ok(re) = crate::regexes::cached(&pat) else {
        return false;
    };
    let dead = dead_regions(text);
    for m in re.find_iter(text) {
        let mut i = m.end() - 1;
        let mut depth = 0i32;
        let b = text.as_bytes();
        while i < b.len() {
            if b[i] == b'(' {
                depth += 1
            } else if b[i] == b')' {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            i += 1;
        }
        let rest: String = text.get(i + 1..).unwrap_or("").chars().take(199).collect();
        let rest = rest.trim_start();
        if rest.starts_with('{') && !dead.iter().any(|(a, z)| *a <= m.start() && m.start() <= *z) {
            return true;
        }
    }
    false
}
fn relpath(c: &Corpus, p: &Path) -> String {
    p.strip_prefix(&c.root)
        .unwrap_or(p)
        .to_string_lossy()
        .replace('\\', "/")
}
fn children<'a>(c: &'a Corpus, group: &'a str, p: &Path) -> Vec<&'a SourceFile> {
    c.ordered_sources(group)
        .into_iter()
        .filter(|f| !f.is_directory() && f.path.starts_with(p))
        .collect()
}
fn basename(p: &Path) -> String {
    p.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}
fn audit(c: &Corpus, name: &str, ddir: &Path, crate_dir: &Path) -> Vec<(String, String)> {
    let files = children(c, "lost-dll", ddir);
    let spec = files.iter().find(|f| {
        f.path
            .file_name()
            .is_some_and(|n| n.to_string_lossy() == format!("{name}.spec"))
    });
    let Some(spec) = spec else { return vec![] };
    let exports: BTreeSet<String> = ENTRY
        .captures_iter(&spec.text)
        .filter_map(|m| {
            let f = m.get(2).map_or("", |x| x.as_str());
            if !f.is_empty() && !f.starts_with(['#', '-']) {
                None
            } else {
                Some(m[1].to_owned())
            }
        })
        .collect();
    let mut defined: BTreeSet<String> = BTreeSet::new();
    let mut rust_files = children(c, "lost-crates", &crate_dir.join("src"));
    rust_files.sort_by(|a, b| a.path.cmp(&b.path));
    for rf in rust_files {
        if rf.path.extension().is_some_and(|e| e == "rs") {
            defined.extend(RUST_FN.captures_iter(&rf.text).map(|x| x[1].to_owned()));
        }
    }
    for af in files.iter().filter(|f| {
        matches!(
            f.path.extension().and_then(|x| x.to_str()),
            Some("S" | "s" | "asm")
        )
    }) {
        defined.extend(IDENT.captures_iter(&af.text).map(|x| x[1].to_owned()));
    }
    let cm = files
        .iter()
        .find(|f| f.path.file_name().is_some_and(|n| n == "CMakeLists.txt"))
        .map_or("", |f| f.text.as_str());
    let comments: BTreeSet<_> = COMMENTED
        .captures_iter(cm)
        .map(|m| {
            Path::new(&m[1])
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    let live_names: BTreeSet<_> = cm
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .flat_map(|line| {
            LIVE.captures_iter(line).map(|m| {
                Path::new(&m[1])
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned()
            })
        })
        .collect();
    let flipped_names: BTreeSet<_> = comments.difference(&live_names).cloned().collect();
    let cfiles: Vec<_> = files
        .iter()
        .copied()
        .filter(|f| {
            matches!(
                f.path.extension().and_then(|x| x.to_str()),
                Some("c" | "cpp")
            )
        })
        .collect();
    let flipped: Vec<_> = cfiles
        .iter()
        .filter(|f| flipped_names.contains(&basename(&f.path)))
        .copied()
        .collect();
    let live: Vec<_> = cfiles
        .iter()
        .filter(|f| !flipped_names.contains(&basename(&f.path)))
        .copied()
        .collect();
    let mut out = vec![];
    for sym in exports.difference(&defined) {
        if live.iter().any(|f| defined_live(&f.text, sym)) {
            continue;
        }
        if let Some(f) = flipped.iter().find(|f| defined_live(&f.text, sym)) {
            out.push((
                sym.clone(),
                f.path
                    .strip_prefix(ddir)
                    .unwrap_or(&f.path)
                    .to_string_lossy()
                    .replace('\\', "/"),
            ));
        }
    }
    out
}
impl Rule for VerifyLostExports {
    fn name(&self) -> &'static str {
        "verify_lost_exports"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let mut crates: BTreeMap<String, PathBuf> = BTreeMap::new();
        for d in c
            .files
            .iter()
            .filter(|f| f.is_directory() && f.path.starts_with("sdk/lib/rust"))
        {
            let n = basename(&d.path);
            if let Some(name) = n.strip_suffix("-rust") {
                crates.insert(name.into(), d.path.clone());
            }
        }
        let dll_dirs: Vec<_> = c
            .ordered_sources("lost-dll")
            .into_iter()
            .filter(|f| f.is_directory())
            .collect();
        let mut out = vec![];
        for (name, crdir) in crates {
            let Some(dd) = dll_dirs.iter().find(|d| basename(&d.path) == name) else {
                continue;
            };
            let dir = &dd.path;
            if !children(c, "lost-dll", dir).iter().any(|f| {
                f.path
                    .file_name()
                    .is_some_and(|x| x.to_string_lossy() == format!("{name}.spec"))
            }) {
                continue;
            }
            for (sym, cf) in audit(c, &name, dir, &crdir) {
                out.push(format!("{name}: LOST export {sym} — implemented in {cf}, which is commented out; no Rust fn"));
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(
        spec: &str,
        cmake: &str,
        sources: &[(&str, &str)],
        rust: &[(&str, &str)],
    ) -> (Corpus, PathBuf, PathBuf) {
        let root = PathBuf::from("/fixture");
        let dll = PathBuf::from("dll/win32/m");
        let cr = PathBuf::from("sdk/lib/rust/m-rust");
        let mut fs = vec![
            SourceFile::new(dll.join("m.spec"), spec.into()),
            SourceFile::new(dll.join("CMakeLists.txt"), cmake.into()),
        ];
        fs.extend(
            sources
                .iter()
                .map(|(p, t)| SourceFile::new(dll.join(p), (*t).into())),
        );
        fs.extend(
            rust.iter()
                .map(|(p, t)| SourceFile::new(cr.join("src").join(p), (*t).into())),
        );
        (Corpus::new(root, fs), dll, cr)
    }
    #[test]
    fn genuine_drop_and_forwarder() {
        let cm = "list(APPEND SOURCE\n# impl.c\n other.c)";
        let (c, d, r) = fixture(
            "@ stdcall Gone()\n",
            cm,
            &[
                ("impl.c", "VOID\nGone(void)\n{\n}\n"),
                ("other.c", "int x;\n"),
            ],
            &[("lib.rs", "// nothing\n")],
        );
        assert_eq!(
            audit(&c, "m", &d, &r),
            vec![("Gone".into(), "impl.c".into())]
        );
        let (c, d, r) = fixture(
            "@ stdcall Gone() Real\n",
            cm,
            &[("impl.c", "VOID\nGone(void)\n{\n}\n")],
            &[],
        );
        assert!(audit(&c, "m", &d, &r).is_empty());
    }
    #[test]
    fn prototype_call_and_dead_code_are_not_definitions() {
        assert!(!defined_live("VOID\nGone(void);\n", "Gone"));
        assert!(!defined_live("void caller(void)\n{\n Gone();\n}\n", "Gone"));
        assert!(!defined_live(
            "#if 0\nVOID\nGone(void)\n{\n}\n#endif\n",
            "Gone"
        ));
    }
    #[test]
    fn no_mangle_shapes_and_comment_live_cancel() {
        assert!(RUST_FN.is_match("#[no_mangle] pub unsafe extern \"system\" fn Gone() {}"));
        assert!(RUST_FN
            .is_match("#[no_mangle]\n// note\n// note2\npub unsafe extern \"C\" fn Gone() {}"));
        let cm = "# shim.c explains why\nlist(APPEND SOURCE\n# impl.c\n shim.c)";
        let comments: BTreeSet<_> = COMMENTED
            .captures_iter(cm)
            .map(|m| basename(Path::new(&m[1])))
            .collect();
        let lives: BTreeSet<_> = LIVE
            .captures_iter(cm)
            .map(|m| basename(Path::new(&m[1])))
            .collect();
        assert!(comments.difference(&lives).all(|x| x != "shim.c"));
    }
    #[test]
    fn all_original_served_export_controls() {
        let cm = "list(APPEND SOURCE\n# impl.c\n other.c)";
        let cbody = "VOID\nGone(void)\n{\n}\n";
        let served=[
            ("no_mangle_separated",cm,&[("impl.c",cbody)][..],&[("lib.rs","#[no_mangle]\n// note\n// note2\npub unsafe extern \"C\" fn Gone() {}\n")][..]),
            ("live_c", "list(APPEND SOURCE\n# impl.c\n rust_varargs_shim.c)",&[("impl.c",cbody),("rust_varargs_shim.c","void CDECL Gone(int a, ...)\n{\n}\n")][..],&[][..]),
            ("comment_live", "# rust_varargs_shim.c is NOT a translation reverted to C.\nlist(APPEND SOURCE\n# impl.c\n rust_varargs_shim.c)",&[("impl.c",cbody),("rust_varargs_shim.c","void CDECL Gone(int a, ...)\n{\n}\n")][..],&[][..]),
            ("cpp", "list(APPEND SOURCE\n# impl.cpp\n live.cpp)",&[("impl.cpp",cbody),("live.cpp","HRESULT WINAPI Gone(void)\n{\n return 0;\n}\n")][..],&[][..]),
            ("prototype",cm,&[("impl.c","VOID\nGone(void);\n")][..],&[][..]),
            ("dead",cm,&[("impl.c","#if 0\nVOID\nGone(void)\n{\n}\n#endif\n")][..],&[][..]),
            ("call",cm,&[("impl.c",cbody),("other.c","void caller(void)\n{\n Gone();\n}\n")][..],&[][..]),
            ("same_line",cm,&[("impl.c",cbody)][..],&[("lib.rs","#[no_mangle] pub unsafe extern \"system\" fn Gone() {}\n")][..]),
            ("trailing_comment",cm,&[("impl.c",cbody)][..],&[("lib.rs","#[no_mangle] // SHELL32.235\npub unsafe extern \"system\" fn Gone() {}\n")][..]),
        ];
        for (name, cm, sources, rust) in served {
            let (c, d, r) = fixture("@ varargs Gone(long str)\n", cm, sources, rust);
            let got = audit(&c, "m", &d, &r);
            if name == "call" {
                assert_eq!(got.len(), 1, "call control");
            } else {
                assert!(got.is_empty(), "case {name}: {got:?}");
            }
        }
        let (c, d, r) = fixture(
            "@ stdcall Gone()\n",
            cm,
            &[("impl.c", cbody), ("other.c", "int x;\n")],
            &[("lib.rs", "#[no_mangle]\npub static Gone: u32 = 0;\n")],
        );
        assert_eq!(audit(&c, "m", &d, &r).len(), 1);
    }
}
