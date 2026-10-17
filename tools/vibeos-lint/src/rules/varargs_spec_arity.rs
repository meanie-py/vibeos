use crate::{Corpus, Rule, SourceSet};
use regex::Regex;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::LazyLock,
};

pub struct VarargsSpecArity;

static SOURCES: &[SourceSet] = &[SourceSet::tree(
    "varargs_specs",
    &[
        "dll",
        "ntoskrnl",
        "hal",
        "win32ss",
        "drivers",
        "base",
        "subsystems",
        "sdk/lib",
    ],
    &["spec"],
    &[],
)];
static SPEC: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"(?m)^\s*@\s+varargs\s+(?:-[\w=.,-]+\s+)*(\w+)\(([^)]*)\)").unwrap()
});
static DECL: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r#"(?s)(?:#\[link_name\s*=\s*"(\w+)"\]\s*)?(?:pub\s+)?fn\s+(\w+)\s*\(([^;{]*?)\)\s*(?:->[^;{]*)?;"#).unwrap()
});
static COMMENT: LazyLock<Regex> = LazyLock::new(|| crate::regexes::cached(r"//[^\n]*").unwrap());
static PARENS: LazyLock<Regex> = LazyLock::new(|| crate::regexes::cached(r"\([^()]*\)").unwrap());
static ANGLES: LazyLock<Regex> = LazyLock::new(|| crate::regexes::cached(r"<[^<>]*>").unwrap());

fn spec_arities<'a>(texts: impl Iterator<Item = &'a str>) -> BTreeMap<String, BTreeSet<usize>> {
    let mut out = BTreeMap::<String, BTreeSet<usize>>::new();
    for text in texts {
        for m in SPEC.captures_iter(text) {
            let count = m[2].split_whitespace().count();
            out.entry(m[1].to_owned()).or_default().insert(count);
        }
    }
    out
}

fn scan_text(
    text: &str,
    spec: &BTreeMap<String, BTreeSet<usize>>,
) -> Vec<(usize, String, usize, Vec<usize>)> {
    let code = COMMENT.replace_all(text, "");
    let code = code.as_ref();
    let mut out = Vec::new();
    for m in DECL.captures_iter(code) {
        let params = &m[3];
        if !params.contains("...") {
            continue;
        }
        let name = m.get(1).map_or(&m[2], |x| x.as_str());
        let Some(want) = spec.get(name) else {
            continue;
        };
        let no_parens = PARENS.replace_all(params, "");
        let flat = ANGLES.replace_all(&no_parens, "");
        let fixed = flat
            .split(',')
            .filter(|a| !a.trim().is_empty() && a.trim() != "...")
            .count();
        if !want.contains(&fixed) {
            let name_at = m.get(2).unwrap().start();
            let line = code[..name_at].bytes().filter(|&b| b == b'\n').count() + 1;
            out.push((line, name.to_owned(), fixed, want.iter().copied().collect()));
        }
    }
    out
}

impl Rule for VarargsSpecArity {
    fn name(&self) -> &'static str {
        "scan_varargs_spec_arity"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        SOURCES
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let specs = corpus
            .sources("varargs_specs")
            .filter(|f| f.path.extension().is_some_and(|e| e == "spec"))
            .map(|f| f.text.as_str());
        let oracle = spec_arities(specs);
        let mut out = Vec::new();
        for file in corpus.rust_files() {
            let rel = file.path.to_string_lossy().replace('\\', "/");
            for (line, name, fixed, want) in scan_text(&file.text, &oracle) {
                out.push(format!(
                    "{rel}:{line}: VARARGS_SPEC_ARITY {name} declared with {fixed} fixed parameter(s), the export takes {want:?}"
                ));
            }
        }
        out.sort();
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceFile;
    use std::path::PathBuf;

    fn oracle() -> BTreeMap<String, BTreeSet<usize>> {
        spec_arities(["@ varargs _snwprintf(ptr long wstr)\n@ varargs swprintf(ptr wstr) _swprintf # Non-conforming swprintf\n@ cdecl vswprintf(ptr wstr ptr) _vswprintf\n"].into_iter())
    }

    #[test]
    fn exported_spec_arities_follow_original_syntax() {
        let o = spec_arities(["@ varargs _snwprintf(ptr long wstr)\n@ varargs swprintf(ptr wstr) _swprintf\n@ cdecl not_varargs(ptr)\n"].into_iter());
        assert_eq!(o["swprintf"], BTreeSet::from([2]));
        assert_eq!(o["_snwprintf"], BTreeSet::from([3]));
        assert!(!o.contains_key("not_varargs"));
    }

    #[test]
    fn dedicated_positive_and_negative_fixtures() {
        let spec = oracle();
        let bad = "extern \"C\" {\n pub fn swprintf(s: *mut u16, n: usize, fmt: *const u16, ...) -> c_int; // x.rs\n}\n";
        assert_eq!(
            scan_text(bad, &spec),
            vec![(2, "swprintf".into(), 3, vec![2])]
        );
        let good = "#[link_name = \"_snwprintf\"]\npub fn swprintf(s: *mut u16, n: usize, fmt: *const u16, ...) -> c_int;\nfn swprintf(buffer: *mut WCHAR, format: *const WCHAR, ...) -> INT;\npub fn _snwprintf(b: *mut u16, n: usize, f: *const u16, ...) -> i32;\n";
        assert!(scan_text(good, &spec).is_empty());
        let link_alias = "#[link_name = \"swprintf\"]\nfn my_fmt(b: *mut u16, n: Option<fn(u32, u32)>, f: *const u16, ...) -> i32;\n";
        assert_eq!(
            scan_text(link_alias, &spec),
            vec![(2, "swprintf".into(), 3, vec![2])]
        );
        let clean = "pub fn vswprintf(b: *mut u16, n: usize, f: *const u16, a: va_list) -> i32;\npub fn my_printf(a: i32, b: i32, c: i32, ...) -> i32;\n// pub fn swprintf(s: *mut u16, n: usize, fmt: *const u16, ...) -> c_int;\n";
        assert!(scan_text(clean, &spec).is_empty());
    }

    #[test]
    fn rule_formats_repo_relative_findings() {
        let f = SourceFile::new(
            "sdk/lib/rust/x-rust/src/lib.rs".into(),
            "pub fn swprintf(s: *mut u16, n: usize, fmt: *const u16, ...) -> c_int;\n".into(),
        );
        let spec = SourceFile::new(
            "dll/msvcrt/msvcrt.spec".into(),
            "@ varargs swprintf(ptr wstr) _swprintf\n".into(),
        );
        let c = Corpus::new(PathBuf::from("/repo"), vec![f, spec]);
        assert_eq!(
            VarargsSpecArity.run(&c).unwrap(),
            vec![
                "sdk/lib/rust/x-rust/src/lib.rs:1: VARARGS_SPEC_ARITY swprintf declared with 3 fixed parameter(s), the export takes [2]"
            ]
        );
    }
}
