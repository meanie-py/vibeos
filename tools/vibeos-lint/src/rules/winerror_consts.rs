//! Check Rust winerror-named constants against the authoritative winerror.h values.
use crate::{Corpus, Rule, SourceSet};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

const HEADER: &str = "sdk/include/psdk/winerror.h";
const NOLINT: &str = "nolint: winerror-const";
pub struct WinerrorConsts;
const INPUTS: &[SourceSet] = &[
    SourceSet::tree("winerror-rust", &["sdk/lib/rust"], &["rs"], &[])
        .with_pruning(&["target", ".responses"])
        .with_inputs(),
    SourceSet::tree(
        "winerror-header",
        &["sdk/include/psdk/winerror.h"],
        &["h"],
        &[],
    ),
];
impl Rule for WinerrorConsts {
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn name(&self) -> &'static str {
        "scan_winerror_consts"
    }
    fn extra_files(&self) -> &'static [&'static str] {
        &[HEADER]
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let header = corpus
            .files
            .iter()
            .find(|f| f.path == std::path::Path::new(HEADER))
            .ok_or_else(|| format!("missing required snapshot {HEADER}"))?;
        let codes = header_values(&header.text);
        let mut out = Vec::new();
        for file in corpus.sources("winerror-rust") {
            let path = file.path.to_string_lossy();
            for (line, name, value) in scan_text(&file.text, &codes) {
                let want = codes[&name]
                    .iter()
                    .map(|v| format!("0x{v:08x}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                out.push(format!(
                    "{path}:{line}: {name} = 0x{value:08x}, winerror.h says {want}"
                ));
            }
        }
        Ok(out)
    }
}
fn define_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| crate::regexes::cached(r"(?m)^\s*#define\s+(\w+)\s+(?:_HRESULT_TYPEDEF_|_NDIS_ERROR_TYPEDEF_)?\(?\s*(0x[0-9A-Fa-f]+|\d+)L?\s*\)?\s*(?:$|//|/\*)").unwrap())
}
fn const_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| crate::regexes::cached(r"\bconst\s+(\w+)\s*:\s*[\w:]+\s*=\s*\(?\s*(0x[0-9A-Fa-f_]+|\d[\d_]*)(?:_?[ui](?:8|16|32|64|size))?\b").unwrap())
}
fn parse_value(s: &str) -> Option<u32> {
    let raw = s.replace('_', "");
    let (digits, radix) =
        if let Some(hex) = raw.strip_prefix("0x").or_else(|| raw.strip_prefix("0X")) {
            (hex, 16u32)
        } else {
            (raw.as_str(), 10u32)
        };
    if digits.is_empty() {
        return None;
    }
    let mut value = 0u32;
    for ch in digits.chars() {
        value = value.wrapping_mul(radix).wrapping_add(ch.to_digit(radix)?);
    }
    Some(value)
}

fn header_values(text: &str) -> BTreeMap<String, BTreeSet<u32>> {
    let mut values = BTreeMap::new();
    for m in define_re().captures_iter(text) {
        let (Some(name), Some(raw)) = (m.get(1), m.get(2)) else {
            continue;
        };
        if let Some(value) = parse_value(raw.as_str()) {
            values
                .entry(name.as_str().to_owned())
                .or_insert_with(BTreeSet::new)
                .insert(value);
        }
    }
    values
}
fn scan_text(text: &str, codes: &BTreeMap<String, BTreeSet<u32>>) -> Vec<(usize, String, u32)> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let Some(m) = const_re().captures(line) else {
            continue;
        };
        let (Some(name), Some(raw)) = (m.get(1), m.get(2)) else {
            continue;
        };
        let name = name.as_str();
        if line.contains(NOLINT) || line.trim_start().starts_with("//") {
            continue;
        }
        let Some(known) = codes.get(name) else {
            continue;
        };
        let Some(value) = parse_value(raw.as_str()) else {
            continue;
        };
        if !known.contains(&value) {
            out.push((i + 1, name.to_owned(), value));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    const HEADER_FIXTURE: &str = "\n#define TYPE_E_AMBIGUOUSNAME   _HRESULT_TYPEDEF_(0x8002802C)\n#define ERROR_FILE_NOT_FOUND   2L\n#define SEC_E_NO_CONTEXT       _HRESULT_TYPEDEF_(0x80090361L)\n";
    const SRC: &str = "\npub const TYPE_E_AMBIGUOUSNAME: HRESULT = 0x8002_8CA6u32 as HRESULT;\npub const ERROR_FILE_NOT_FOUND: DWORD = 2;\npub const SEC_E_NO_CONTEXT: SECURITY_STATUS = 0x8009_0361u32 as i32;\nconst ERROR_FILE_NOT_FOUND: u32 = 3; // nolint: winerror-const (test)\n// const TYPE_E_AMBIGUOUSNAME: HRESULT = 1;\nconst MY_OWN_CODE: u32 = 7;\nconst SEC_E_NO_CONTEXT: i32 = -2146893215;\n";
    #[test]
    fn header_wrappers_and_only_wrong_values() {
        let codes = header_values(HEADER_FIXTURE);
        assert_eq!(codes["ERROR_FILE_NOT_FOUND"], [2].into_iter().collect());
        assert_eq!(
            scan_text(SRC, &codes)
                .into_iter()
                .map(|(line, name, _)| (line, name))
                .collect::<Vec<_>>(),
            vec![(2, "TYPE_E_AMBIGUOUSNAME".into())]
        );
    }
    #[test]
    fn exact_lint_output_and_header_snapshot() {
        let corpus = Corpus::new(
            "/fixture".into(),
            vec![
                crate::SourceFile::new(HEADER.into(), HEADER_FIXTURE.into()),
                crate::SourceFile::new(
                    "sdk/lib/rust/a.rs".into(),
                    "pub const TYPE_E_AMBIGUOUSNAME: HRESULT = 0x8002_8CA6u32;".into(),
                ),
            ],
        );
        assert_eq!(WinerrorConsts.run(&corpus).unwrap(), vec!["sdk/lib/rust/a.rs:1: TYPE_E_AMBIGUOUSNAME = 0x80028ca6, winerror.h says 0x8002802c".to_owned()]);
    }
}
