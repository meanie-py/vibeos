use crate::{Corpus, Rule, SourceFile, SourceSet};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::LazyLock;

pub struct VerifyConvMacros;
const ROOTS: &[&str] = &["sdk/include/psdk", "sdk/include/reactos"];
const INPUTS: &[SourceSet] = &[SourceSet::tree("conv-headers", ROOTS, &["h"], &[])];
const VENDOR_YAML: &str = "vendor.yaml";
static DEFINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*#\s*define\s+([A-Za-z_][A-Za-z0-9_]*)\s+(.+?)\s*$").unwrap());
static NOISE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(FAR|NEAR|EXTERN_C|extern|DECLSPEC_\w+)\b|__declspec\([^)]*\)|/\*.*?\*/|//.*$")
        .unwrap()
});
static SPLIT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

// This is checker implementation data, just like Python's imported KW_TO_CONV.
// Embed the canonical table so the executable remains standalone. The bootstrap
// fingerprint includes this source dependency as well as the Rust sources.
const CONVENTIONS: &str = include_str!("../../../rustconv/rustconv/conventions.py");
fn known_table(source: &str) -> Result<Vec<(String, String)>, String> {
    static START: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^KW_TO_CONV\s*(?::[^=]+)?=\s*\{\s*(?:#.*)?$").unwrap());
    static ENTRY: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"^\s*(?:"([A-Za-z_][A-Za-z0-9_]*)"|'([A-Za-z_][A-Za-z0-9_]*)')\s*:\s*(?:"([A-Za-z_][A-Za-z0-9_]*)"|'([A-Za-z_][A-Za-z0-9_]*)')\s*,?\s*(?:#.*)?$"#).unwrap()
    });
    let mut lines = source.lines();
    if !lines.by_ref().any(|line| START.is_match(line)) {
        return Err("conventions.py: KW_TO_CONV must be a multiline literal dictionary".into());
    }
    let mut table = BTreeMap::new();
    for line in lines {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "}"
            || line
                .strip_prefix('}')
                .is_some_and(|tail| tail.trim_start().starts_with('#'))
        {
            return Ok(table.into_iter().collect());
        }
        let row = ENTRY
            .captures(line)
            .ok_or_else(|| format!("conventions.py: unsupported KW_TO_CONV entry: {line}"))?;
        let name = row.get(1).or_else(|| row.get(2)).unwrap().as_str();
        let conv = row.get(3).or_else(|| row.get(4)).unwrap().as_str();
        table.insert(name.to_owned(), conv.to_owned());
    }
    Err("conventions.py: unterminated KW_TO_CONV dictionary".into())
}

fn clean_yaml_scalar(s: &str) -> String {
    let s = s.split('#').next().unwrap_or("").trim();
    s.trim_matches(['"', '\'']).to_string()
}
fn vendor_prefixes(text: &str) -> Result<Vec<String>, String> {
    let mut in_trees = false;
    let mut in_headers = false;
    let mut out = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if line.starts_with("trees:") {
            in_trees = true;
            continue;
        }
        if in_trees
            && !line.starts_with(' ')
            && !line.starts_with('\t')
            && !trimmed.is_empty()
            && !trimmed.starts_with('#')
        {
            in_trees = false;
            in_headers = false;
        }
        if !in_trees {
            continue;
        }
        let indent = line.chars().take_while(|c| *c == ' ').count();
        if indent <= 2 {
            in_headers = false;
        }
        if indent == 2 && trimmed.starts_with("- path:") {
            out.push(clean_yaml_scalar(trimmed[7..].trim()));
        } else if indent == 4 && trimmed == "headers:" {
            in_headers = true;
        } else if in_headers && indent >= 6 && trimmed.starts_with("- ") {
            out.push(clean_yaml_scalar(trimmed[2..].trim()));
        } else if in_headers && indent < 6 {
            in_headers = false;
        }
    }
    if out.is_empty() {
        return Err("vendor.yaml has no trees/path prefixes".into());
    }
    Ok(out)
}
fn is_vendored(path: &str, prefixes: &[String]) -> bool {
    prefixes.iter().any(|p| {
        path == p
            || path
                .strip_prefix(p)
                .is_some_and(|rest| rest.starts_with('/'))
    })
}
fn harvest(files: &[&SourceFile], prefixes: &[String]) -> BTreeMap<String, (String, String)> {
    let mut defs = BTreeMap::<String, (Vec<String>, String, bool)>::new();
    for root in ROOTS {
        let mut headers: Vec<_> = files
            .iter()
            .copied()
            .filter(|f| f.path.starts_with(root) && f.path.extension().is_some_and(|e| e == "h"))
            .collect();
        headers.sort_by_key(|f| f.path.clone());
        for h in headers {
            let rel = h.path.to_string_lossy().replace('\\', "/");
            let vendored = is_vendored(&rel, prefixes);
            for (i, line) in h.text.lines().enumerate() {
                if let Some(c) = DEFINE.captures(line) {
                    let name = c.get(1).unwrap().as_str();
                    let rhs = NOISE.replace_all(c.get(2).unwrap().as_str(), " ");
                    let toks: Vec<_> = SPLIT
                        .split(rhs.trim())
                        .filter(|t| !t.is_empty())
                        .map(str::to_string)
                        .collect();
                    if !toks.is_empty() {
                        defs.entry(name.to_string()).or_insert((
                            toks,
                            format!("{rel}:{}", i + 1),
                            vendored,
                        ));
                    }
                }
            }
        }
    }
    fn resolve(
        name: &str,
        defs: &BTreeMap<String, (Vec<String>, String, bool)>,
        seen: &BTreeSet<String>,
    ) -> Option<&'static str> {
        match name {
            "__stdcall" | "_stdcall" => return Some("system"),
            "__cdecl" | "_cdecl" => return Some("cdecl"),
            "__fastcall" | "_fastcall" => return Some("fastcall"),
            _ => {}
        }
        if seen.contains(name) {
            return None;
        }
        let mut next = seen.clone();
        next.insert(name.to_string());
        let (tokens, _, _) = defs.get(name)?;
        for tok in tokens {
            if let Some(c) = resolve(tok, defs, &next) {
                return Some(c);
            }
        }
        None
    }
    let mut out = BTreeMap::new();
    for (name, (_, cite, foreign)) in &defs {
        if *foreign {
            continue;
        }
        if let Some(conv) = resolve(name, &defs, &BTreeSet::new()) {
            out.insert(name.clone(), (conv.to_string(), cite.clone()));
        }
    }
    out
}
fn findings(found: &BTreeMap<String, (String, String)>, known: &[(&str, &str)]) -> Vec<String> {
    let table: BTreeMap<_, _> = known.iter().copied().collect();
    let mut out = Vec::new();
    let mut missing = Vec::new();
    let mut wrong = Vec::new();
    for (name, (want, cite)) in found {
        match table.get(name.as_str()) {
            None => missing.push((name, want, cite)),
            Some(have) if *have != want => wrong.push((name, *have, want, cite)),
            _ => {}
        }
    }
    for (name, have, want, cite) in &wrong {
        out.push(format!(
            "WRONG   {name}: table says '{have}', headers say '{want}'  ({cite})"
        ));
    }
    for (name, conv, cite) in &missing {
        out.push(format!("MISSING {name} -> {conv}  ({cite})"));
    }
    if !missing.is_empty() || !wrong.is_empty() {
        out.push(String::new());
        out.push(format!(
            "SUMMARY: {} missing, {} wrong (of {} convention aliases in the headers)",
            missing.len(),
            wrong.len(),
            found.len()
        ));
        out.push("An UNKNOWN alias is not a no-op: _derive_convention falls back to cdecl,".into());
        out.push(
            "silently. Add these to rustconv/abi_enforce.py::_KW_TO_CONV with citations.".into(),
        );
    } else {
        out.push(format!(
            "SUMMARY: 0 missing, 0 wrong ({} convention aliases, all known)",
            found.len()
        ));
    }
    out
}
impl Rule for VerifyConvMacros {
    fn name(&self) -> &'static str {
        "verify_conv_macros"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn extra_files(&self) -> &'static [&'static str] {
        &[VENDOR_YAML]
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let vendor = c
            .files
            .iter()
            .find(|f| f.path == Path::new(VENDOR_YAML))
            .ok_or_else(|| {
                "missing vendor.yaml snapshot required by verify_conv_macros".to_string()
            })?;
        let prefixes = vendor_prefixes(&vendor.text)?;
        let headers: Vec<_> = c
            .files
            .iter()
            .filter(|f| {
                ROOTS.iter().any(|r| f.path.starts_with(r))
                    && f.path.extension().is_some_and(|e| e == "h")
            })
            .collect();
        let found = harvest(&headers, &prefixes);
        let known = known_table(CONVENTIONS)?;
        let refs: Vec<_> = known
            .iter()
            .map(|(name, conv)| (name.as_str(), conv.as_str()))
            .collect();
        Ok(findings(&found, &refs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    fn file(path: &str, text: &str) -> SourceFile {
        SourceFile::new(PathBuf::from(path), text.into())
    }
    const VENDOR: &str = "trees:\n  - path: vendor/foreign\n    headers:\n      - sdk/include/psdk/foreign\nours: []\n";
    #[test]
    fn canonical_convention_table_includes_master_mi_aliases() {
        let table: BTreeMap<_, _> = known_table(CONVENTIONS).unwrap().into_iter().collect();
        assert_eq!(table.get("MI_CALL").map(String::as_str), Some("system"));
        assert_eq!(table.get("MI_MAIN_CALL").map(String::as_str), Some("cdecl"));
    }
    #[test]
    fn literal_table_changes_drive_missing_and_wrong_alias_findings() {
        let found = BTreeMap::from([("NEW_CALL".into(), ("system".into(), "alias.h:1".into()))]);
        let parse = |source| known_table(source).unwrap();
        let table = parse("KW_TO_CONV: dict[str, str] = {\n 'NEW_CALL': 'system', # citation\n}\n");
        let refs: Vec<_> = table
            .iter()
            .map(|(n, c)| (n.as_str(), c.as_str()))
            .collect();
        assert_eq!(
            findings(&found, &refs),
            ["SUMMARY: 0 missing, 0 wrong (1 convention aliases, all known)"]
        );
        let table = parse("KW_TO_CONV = {\n 'NEW_CALL': 'cdecl',\n}\n");
        let refs: Vec<_> = table
            .iter()
            .map(|(n, c)| (n.as_str(), c.as_str()))
            .collect();
        assert_eq!(
            findings(&found, &refs)[0],
            "WRONG   NEW_CALL: table says 'cdecl', headers say 'system'  (alias.h:1)"
        );
        assert!(findings(&found, &[])[0].starts_with("MISSING NEW_CALL"));
        assert!(known_table("KW_TO_CONV = {\n **BASE,\n}\n").is_err());
    }
    #[test]
    fn resolves_alias_chains_and_first_definition_wins() {
        let files = vec![file(
            "sdk/include/psdk/a.h",
            "#define ROOT __stdcall\n#define MID FAR ROOT\n#define API MID\n#define API __cdecl\n",
        )];
        let refs: Vec<_> = files.iter().collect();
        let got = harvest(&refs, &[]);
        assert_eq!(got.get("API").unwrap().0, "system");
    }
    #[test]
    fn exact_vendor_registry_prefixes_exclude_registered_headers_only() {
        let p = vendor_prefixes(VENDOR).unwrap();
        assert!(is_vendored("sdk/include/psdk/foreign/alias.h", &p));
        assert!(!is_vendored("sdk/include/psdk/foreignish/alias.h", &p));
        assert!(is_vendored("vendor/foreign/a.h", &p));
    }
    #[test]
    fn positive_and_negative_alias_fixtures_report_exact_stdout() {
        let files = vec![file(
            "sdk/include/psdk/alias.h",
            "#define WINAPI __stdcall\n#define MYSTERY WINAPI\n#define GOOD __stdcall\n",
        )];
        let refs: Vec<_> = files.iter().collect();
        let found = harvest(&refs, &[]);
        let got = findings(&found, &[("GOOD", "system")]);
        assert!(got[0].starts_with("MISSING MYSTERY -> system  (sdk/include/psdk/alias.h:2)"));
        assert_eq!(
            findings(
                &found,
                &[
                    ("GOOD", "system"),
                    ("MYSTERY", "system"),
                    ("WINAPI", "system")
                ],
            ),
            vec!["SUMMARY: 0 missing, 0 wrong (3 convention aliases, all known)"]
        );
    }
    #[test]
    fn wrong_table_entry_is_reported_and_unresolvable_alias_is_ignored() {
        let files = vec![file(
            "sdk/include/psdk/alias.h",
            "#define API __stdcall\n#define CYCLE OTHER\n#define OTHER CYCLE\n",
        )];
        let refs: Vec<_> = files.iter().collect();
        let found = harvest(&refs, &[]);
        assert!(findings(&found, &[("API", "cdecl")])[0]
            .contains("WRONG   API: table says 'cdecl', headers say 'system'"));
        assert!(!found.contains_key("CYCLE"));
    }
}
