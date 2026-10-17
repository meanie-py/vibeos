//! Bare HRESULT values checked against the C module's same-named functions.
use crate::{Corpus, Rule, SourceFile};
use rayon::prelude::*;
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::LazyLock;

pub struct HresultLiterals;
impl Rule for HresultLiterals {
    fn name(&self) -> &'static str {
        "scan_hresult_literals"
    }
    fn needs_c_sources(&self) -> bool {
        true
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let header = corpus
            .files
            .iter()
            .find(|f| f.path == Path::new("sdk/include/psdk/winerror.h"))
            .ok_or("HRESULT scanner requires sdk/include/psdk/winerror.h in the source corpus")?;
        let names = hresult_names(&header.text);
        let mut crates: BTreeMap<String, Vec<&SourceFile>> = BTreeMap::new();
        for file in corpus.rust_files() {
            let Some(rest) = file.path.strip_prefix("sdk/lib/rust").ok() else {
                continue;
            };
            let mut components = rest.components();
            let Some(name) = components.next() else {
                continue;
            };
            if components.next().is_some_and(|p| p.as_os_str() == "src") {
                crates
                    .entry(name.as_os_str().to_string_lossy().into_owned())
                    .or_default()
                    .push(file);
            }
        }
        let c_files: Vec<_> = corpus
            .files
            .iter()
            .filter(|f| f.path.extension().is_some_and(|e| e == "c"))
            .collect();
        let out: Vec<Vec<String>> = crates
            .into_par_iter()
            .map(|(name, mut files)| {
                if !files.iter().any(|f| HR_LITERAL.is_match(&f.text)) {
                    return Vec::new();
                }
                let mut out = Vec::new();
                let stem = name.strip_suffix("-rust").unwrap_or(&name);
                let stem = stem.strip_suffix("-sys").unwrap_or(stem);
                let stem = match stem {
                    "ntkrnl" => "ntoskrnl",
                    "rtlk" => "rtl",
                    "win32k" => "win32ss",
                    other => other,
                };
                let mut matching = Vec::new();
                for file in &c_files {
                    let path = file.path.to_string_lossy();
                    if ["build-", "modules/rostests", "sdk/lib/rust", ".git"]
                        .iter()
                        .any(|p| path.starts_with(p))
                    {
                        continue;
                    }
                    if file
                        .path
                        .parent()
                        .into_iter()
                        .flat_map(|p| p.ancestors())
                        .any(|p| p.file_name().is_some_and(|n| n == stem))
                    {
                        matching.push(*file);
                    }
                }
                if matching.is_empty() {
                    return Vec::new();
                }
                matching.sort_by(|a, b| a.path.cmp(&b.path));
                let functions = c_functions(matching.iter().map(|f| f.text.as_str()));
                files.sort_by(|a, b| a.path.cmp(&b.path));
                for file in files {
                    for (line, function, literal, _) in scan_text(&file.text, &functions, &names) {
                        out.push(format!(
                            "{}:{line}: {function}: {literal} not returned by the C function",
                            file.path.display()
                        ));
                    }
                }
                out
            })
            .collect();
        Ok(out.into_iter().flatten().collect())
    }
}

static HR_LITERAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b0x(8[0-9a-fA-F]{7})(?:u32)?\s+as\s+(?:HRESULT|i32|LONG)\b").unwrap()
});
static RUST_FN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^\s*(?:stdmethod!\s*\{\s*)?(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?(?:extern\s+"[^"]+"\s+)?fn\s+(\w+)"#).unwrap()
});
static RUST_CONST: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(?:pub(?:\([^)]*\))?\s+)?const\s+\w+\s*:").unwrap());
static C_DEF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(\w+)\s*\(((?:[^;{}()]|\([^;{}()]*\))*)\)\s*\{").unwrap());
static C_COMMENT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/|//[^\n]*").unwrap());
static NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:[A-Z][A-Z0-9]*_E_\w+|E_[A-Z][A-Z0-9_]*|[A-Z]+_S_\w+)\b").unwrap()
});
static HR_DEFINE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^#define\s+(\w+)\s+_HRESULT_TYPEDEF_\((0x[0-9A-Fa-f]+)L?\)").unwrap()
});

type Names = BTreeMap<u32, BTreeSet<String>>;
type Functions = BTreeMap<String, Vec<String>>;
fn hresult_names(header: &str) -> Names {
    let mut names = Names::new();
    for line in header.lines() {
        if let Some(m) = HR_DEFINE.captures(line) {
            if let Ok(value) = u32::from_str_radix(&m[2][2..], 16) {
                names.entry(value).or_default().insert(m[1].into());
            }
        }
    }
    names
}

fn c_functions<'a>(sources: impl IntoIterator<Item = &'a str>) -> Functions {
    let mut out = Functions::new();
    for source in sources {
        let text = C_COMMENT.replace_all(source, " ");
        for m in C_DEF.captures_iter(&text) {
            let name = &m[1];
            if [
                "if", "for", "while", "switch", "return", "sizeof", "do", "else",
            ]
            .contains(&name)
            {
                continue;
            }
            let whole = m.get(0).unwrap();
            let mut end = text.len();
            let mut depth = 1;
            for (offset, byte) in text.as_bytes()[whole.end()..].iter().enumerate() {
                match byte {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    _ => {}
                }
                if depth == 0 {
                    end = whole.end() + offset + 1;
                    break;
                }
            }
            out.entry(name.into())
                .or_default()
                .push(text[whole.start()..end].into());
        }
    }
    out
}

fn scan_text(
    rust: &str,
    functions: &Functions,
    names: &Names,
) -> Vec<(usize, String, String, String)> {
    let mut function: Option<&str> = None;
    let mut out = Vec::new();
    for (index, line) in rust.lines().enumerate() {
        if let Some(m) = RUST_FN.captures(line) {
            if !line.trim_end().ends_with(';') {
                let name = m.get(1).unwrap().as_str();
                function = if name.chars().any(char::is_uppercase)
                    && !name.chars().any(char::is_lowercase)
                {
                    None
                } else {
                    Some(name)
                };
            }
        }
        let Some(function) = function else {
            continue;
        };
        if line.contains("nolint: hresult-literal") || RUST_CONST.is_match(line) {
            continue;
        }
        for hm in HR_LITERAL.captures_iter(line) {
            let value = u32::from_str_radix(&hm[1], 16).unwrap();
            let Some(bodies) = functions.get(function) else {
                continue;
            };
            let body = bodies.join("\n");
            let c_has: BTreeSet<_> = NAME.find_iter(&body).map(|m| m.as_str()).collect();
            if c_has.is_empty() {
                continue;
            }
            let empty = BTreeSet::new();
            let valnames = names.get(&value).unwrap_or(&empty);
            if valnames
                .iter()
                .any(|name| Regex::new(&format!(r"\b{name}\b")).unwrap().is_match(&body))
            {
                continue;
            }
            if Regex::new(&format!(r"(?i)0x0*{}", &hm[1]))
                .unwrap()
                .is_match(&body)
            {
                continue;
            }
            let label = if valnames.is_empty() {
                "unnamed".into()
            } else {
                valnames
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .join("/")
            };
            out.push((
                index + 1,
                function.into(),
                format!("0x{value:08X} ({label})"),
                c_has.into_iter().collect::<Vec<_>>().join(", "),
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    const C: &str = r#"
static HRESULT WINAPI Stream_Read(IStream *iface, void *pv,
                                  ULONG cb /* [in] */, ULONG *read)
{
    if (!pv) return STG_E_INVALIDPOINTER;
    if (!cb) return STG_E_READFAULT;
    return S_OK;
}

static NTSTATUS Driver_Query(PVOID buffer)
{
    return STATUS_BUFFER_OVERFLOW;
}
"#;
    const RUST: &str = r#"
unsafe extern "system" fn Stream_Read(iface: *mut IStream, pv: *mut c_void, cb: u32) -> HRESULT {
    if pv.is_null() {
        return 0x80030009u32 as HRESULT;
    }
    if cb == 0 {
        return 0x80030004u32 as HRESULT;
    }
    const E_FAIL: HRESULT = 0x80004005u32 as HRESULT;
    let _ = 0x80030004u32 as HRESULT; // nolint: hresult-literal (deliberate)
    0
}

extern "system" {
    fn Stream_Read(iface: *mut IStream) -> HRESULT;
}

unsafe fn Driver_Query(buffer: *mut c_void) -> i32 {
    0x80000005u32 as i32
}
"#;
    #[test]
    fn reports_only_the_wrong_literal() {
        let names = hresult_names("#define STG_E_INVALIDPOINTER _HRESULT_TYPEDEF_(0x80030009L)\n#define STG_E_TOOMANYOPENFILES _HRESULT_TYPEDEF_(0x80030004L)");
        let hits = scan_text(RUST, &c_functions([C]), &names);
        assert_eq!(
            hits.iter().map(|h| (h.0, h.1.as_str())).collect::<Vec<_>>(),
            vec![(7, "Stream_Read")]
        );
        assert!(hits[0].2.contains("STG_E_TOOMANYOPENFILES"));
        assert!(hits[0].3.contains("STG_E_READFAULT"));
    }
    #[test]
    fn c_numeric_alias_and_macro_shim_are_clean() {
        let functions = c_functions(["int Foo() { E_FAIL; return 0x000080030004; }"]);
        assert!(scan_text(
            "fn Foo() { 0x80030004u32 as LONG; }",
            &functions,
            &Names::new()
        )
        .is_empty());
        assert!(scan_text(
            "fn FOO() { 0x80030004u32 as LONG; }",
            &functions,
            &Names::new()
        )
        .is_empty());
    }
}
