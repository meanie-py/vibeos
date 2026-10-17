//! Find extern declarations/exports whose BOOLEAN width disagrees with the implementation.
use crate::{regexes::cached, twins, Corpus, Rule, SourceSet};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const WIDE: &[&str] = &[
    "i32", "u32", "LONG", "ULONG", "NTSTATUS", "BOOL", "INT", "UINT",
];
const C_WIDE: &[&str] = &[
    "BOOL", "INT", "int", "LONG", "ULONG", "DWORD", "UINT", "NTSTATUS", "HRESULT",
];
const SKIP: &[&str] = &["target", "old_translations", ".responses"];
pub struct BoolWidth;
const INPUTS: &[SourceSet] = &[
    SourceSet::tree("bool-width-rust", &["sdk/lib/rust"], &["rs"], &[])
        .with_pruning(&["target", "old_translations", ".responses"])
        .with_inputs(),
    SourceSet::tree(
        "bool-width-headers",
        &["drivers", "sdk/include", "boot"],
        &["h"],
        &[],
    )
    .with_pruning(&["target", ".responses"])
    .with_inputs(),
    SourceSet::tree(
        "bool-width-cohab",
        &["drivers", "sdk/include", "boot"],
        &[],
        &["rust_cohab*.c"],
    )
    .with_pruning(&["target", ".responses"])
    .with_inputs(),
];
impl Rule for BoolWidth {
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn name(&self) -> &'static str {
        "scan_bool_width"
    }
    fn needs_c_twins(&self) -> bool {
        true
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let rust: Vec<_> = corpus
            .sources("bool-width-rust")
            .filter(|f| !skip(&f.path))
            .collect();
        let mut narrow = BTreeMap::<String, (String, String)>::new();
        let mut wide_defs = HashSet::new();
        let mut exports = Vec::<(PathBuf, usize, String, String)>::new();
        let mut returns = BTreeMap::<(PathBuf, String), BTreeSet<String>>::new();
        for f in &rust {
            for m in def_re().captures_iter(&f.text) {
                let (Some(name), Some(ret)) = (m.get(1), m.get(2)) else {
                    continue;
                };
                if is_wide(ret.as_str()) {
                    wide_defs.insert(name.as_str().to_owned());
                }
                if is_narrow(ret.as_str()) {
                    narrow.insert(
                        name.as_str().into(),
                        (f.path.to_string_lossy().into_owned(), ret.as_str().into()),
                    );
                }
            }
            for m in export_re().captures_iter(&f.text) {
                let (Some(name), Some(ret)) = (m.get(1), m.get(2)) else {
                    continue;
                };
                let n = name.as_str().to_owned();
                let r = ret.as_str().to_owned();
                returns
                    .entry((f.path.clone(), n.clone()))
                    .or_default()
                    .insert(r.clone());
                if is_narrow(&r) {
                    let line = f.text[..name.start()]
                        .bytes()
                        .filter(|b| *b == b'\n')
                        .count()
                        + 1;
                    if !line_text(&f.text, line).contains("nolint: bool-width") {
                        exports.push((f.path.clone(), line, n, r));
                    }
                }
            }
        }
        // The C twin snapshot covers definitions. BOOLEAN prototypes in headers and cohab C are
        // an additional oracle supplied in the same corpus by the central source loader.
        for file in corpus
            .sources("bool-width-headers")
            .chain(corpus.sources("bool-width-cohab"))
        {
            let rel = file.path.to_string_lossy();
            for rx in [c_bool_proto_a(), c_bool_proto_b()] {
                for m in rx.captures_iter(&file.text) {
                    if let Some(n) = m.get(1) {
                        narrow
                            .entry(n.as_str().into())
                            .or_insert((rel.to_string(), "BOOLEAN".into()));
                    }
                }
            }
        }
        let mut out = Vec::new();
        for f in &rust {
            let mut in_extern = false;
            for (i, line) in f.text.lines().enumerate() {
                if extern_open().is_match(line) {
                    in_extern = true;
                } else if in_extern && close_brace().is_match(line) && !line.contains('{') {
                    in_extern = false;
                }
                if !in_extern || line.contains("nolint: bool-width") {
                    continue;
                }
                let Some(m) = decl_re().captures(line) else {
                    continue;
                };
                let (Some(name), Some(ret)) = (m.get(1), m.get(2)) else {
                    continue;
                };
                let name = name.as_str();
                let declared = ret.as_str();
                if !is_wide(declared) || wide_defs.contains(name) {
                    continue;
                }
                let Some((def_file, def_ret)) = narrow.get(name) else {
                    continue;
                };
                out.push(format!(
                    "{}:{}: {name} declared -> {declared}, defined -> {def_ret} ({def_file})",
                    f.path.display(),
                    i + 1
                ));
            }
        }
        let index = corpus.c_twins();
        for (path, line, name, ret) in exports {
            let twins: Vec<_> = twins::prefer(&path, index.get(&name))
                .into_iter()
                .filter(|t| twins::same_module(&path, t))
                .collect();
            let wide_sibling = returns
                .get(&(path.clone(), name.clone()))
                .and_then(|rs| rs.iter().find(|r| is_wide(r)))
                .cloned();
            if let Some(w) =
                wide_sibling.filter(|_| twins.first().is_none_or(|t| !is_c_wide(&t.ret)))
            {
                out.push(format!(
                    "{}:{line}: {name} defined -> {ret}, its cfg twin returns {w}",
                    path.display()
                ));
                continue;
            }
            if let Some(t) = twins.first().filter(|t| is_c_wide(&t.ret)) {
                out.push(format!(
                    "{}:{line}: {name} defined -> {ret}, C returns {} ({})",
                    path.display(),
                    t.ret,
                    t.path.display()
                ));
            }
        }
        Ok(out)
    }
}
fn skip(p: &Path) -> bool {
    p.components()
        .any(|c| SKIP.contains(&c.as_os_str().to_string_lossy().as_ref()))
}
fn is_wide(s: &str) -> bool {
    WIDE.contains(&s)
}
fn is_c_wide(s: &str) -> bool {
    C_WIDE.contains(&s)
}
fn is_narrow(s: &str) -> bool {
    matches!(s, "u8" | "BOOLEAN")
}
fn def_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(||cached(r#"(?s)pub (?:unsafe )?extern "(?:system|C|stdcall|win64|fastcall)" fn (\w+)\s*\((?:[^;{]*?)\)\s*->\s*(\w+)"#).unwrap())
}
fn export_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(||cached(r#"(?s)#\[no_mangle\][^\n]*\n(?:\s*#\[[^\n]*\n)*\s*pub (?:unsafe )?extern "(?:system|C|stdcall|win64|fastcall)" fn (\w+)\s*\((?:[^;{]*?)\)\s*->\s*(\w+)"#).unwrap())
}
fn extern_open() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| cached(r#"extern "(?:system|C|stdcall|win64|fastcall)"\s*\{"#).unwrap())
}
fn close_brace() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| cached(r"^\s*\}").unwrap())
}
fn decl_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| cached(r"^\s*(?:pub )?fn (\w+)\s*\(.*\)\s*->\s*(\w+)\s*;").unwrap())
}
fn c_bool_proto_a() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| cached(r"\bBOOLEAN\s+(?:NTAPI\s+)?(\w+)\s*\(").unwrap())
}
fn c_bool_proto_b() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| cached(r"\bBOOLEAN\s*\n(?:NTAPI\s*\n)?\s*(\w+)\s*\(").unwrap())
}
fn line_text(s: &str, n: usize) -> &str {
    s.lines().nth(n.saturating_sub(1)).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_scanner_control_decl_and_export_fixtures() {
        let files=vec![
   crate::SourceFile::new("sdk/lib/rust/userenv-rust/src/gpolicy.rs".into(),"#[no_mangle]\npub unsafe extern \"system\" fn RefreshPolicy(bMachine: BOOL) -> u8 { 1u8 }\n#[no_mangle]\npub unsafe extern \"system\" fn MyNarrowHelper() -> u8 { 1u8 }\n".into()),
   crate::SourceFile::new("sdk/lib/rust/caller-rust/src/lib.rs".into(),"extern \"system\" {\n    fn MyNarrowHelper() -> i32;\n}\n".into()),
   crate::SourceFile::new("sdk/lib/rust/win32k-rust/src/region.rs".into(),"#[cfg(target_arch = \"x86\")]\n#[no_mangle]\npub unsafe extern \"fastcall\" fn MyRegionTest(p: PREGION) -> BOOL { 1 }\n#[cfg(not(target_arch = \"x86\"))]\n#[no_mangle]\npub unsafe extern \"C\" fn MyRegionTest(p: PREGION) -> u8 { 1 }\n".into()),
   crate::SourceFile::new("drivers/userenv/gpolicy.c".into(),"BOOL NTAPI RefreshPolicy(BOOL bMachine) { return 1; }\n".into()),
   crate::SourceFile::new("drivers/win32k/region.c".into(),"BOOL MyRegionTest(void) { return 1; }\n".into()),
  ];
        let corpus = Corpus::new("/repo".into(), files);
        let got = BoolWidth.run(&corpus).unwrap();
        assert!(got
            .iter()
            .any(|s| s.contains("MyNarrowHelper declared -> i32, defined -> u8")));
        assert!(got
            .iter()
            .any(|s| s.contains("MyRegionTest defined -> u8, C returns BOOL")));
    }
    #[test]
    fn header_boolean_prototype_pattern() {
        assert!(c_bool_proto_a().is_match("BOOLEAN NTAPI IsThing("));
        assert!(c_bool_proto_b().is_match("BOOLEAN\nNTAPI\nIsThing("));
    }
}
