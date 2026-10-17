//! Report Rust type aliases that resolve to contradictory widths in one arch bucket.
use crate::{Corpus, Rule, SourceSet};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::OnceLock;

pub struct TypeAliasContradictions;
const INPUTS: &[SourceSet] = &[
    SourceSet::tree("type-alias-rust", &["sdk/lib/rust"], &["rs"], &[])
        .with_pruning(&["target", ".responses"])
        .with_inputs(),
];
impl Rule for TypeAliasContradictions {
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn name(&self) -> &'static str {
        "scan_type_alias_contradictions"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut declarations: BTreeMap<String, Vec<(String, String, String)>> = BTreeMap::new();
        let mut by_arch: BTreeMap<String, HashMap<String, String>> = BTreeMap::new();
        for f in corpus.sources("type-alias-rust") {
            if skip().is_match(&f.path.to_string_lossy()) {
                continue;
            }
            let rel = f.path.to_string_lossy().into_owned();
            let arch = arch_of(&rel);
            let (decls, aliases) = alias_facts(&f.text);
            let table = by_arch.entry(arch.clone()).or_default();
            for (name, rhs) in aliases {
                table.entry(name).or_insert(rhs);
            }
            for (name, rhs) in decls {
                declarations
                    .entry(name)
                    .or_default()
                    .push((rel.clone(), rhs, arch.clone()));
            }
        }
        Ok(contradictions(&declarations, &by_arch)
            .into_iter()
            .map(|(name, rel, rhs, width)| format!("{rel}: {name} = {rhs} ({width} bytes)"))
            .collect())
    }
}
fn skip() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| crate::regexes::cached(r"(/target/|/\.responses/)").unwrap())
}
fn type_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        crate::regexes::cached(
            r"^\s*(?:pub\s*(?:\([^)]*\)\s*)?)?type\s+([A-Za-z_]\w*)\s*=\s*([^;]+);",
        )
        .unwrap()
    })
}
fn impl_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        crate::regexes::cached(r"^\s*(?:pub\s+)?(?:default\s+)?(?:unsafe\s+)?(?:impl|trait)\b")
            .unwrap()
    })
}
fn nolint_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        crate::regexes::cached(r"//\s*nolint(?::\s*type-alias-contradiction\b|\s*$|:?\s*\()")
            .unwrap()
    })
}
fn arch_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        crate::regexes::cached(r"(?:^|[/_])(arm64|aarch64|amd64|x86_64|i386|x86)(?:[/_.]|$)")
            .unwrap()
    })
}
fn ws_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| crate::regexes::cached(r"\s+").unwrap())
}
fn arch_of(path: &str) -> String {
    arch_re()
        .captures(path)
        .and_then(|c| c.get(1))
        .map(|m| {
            match m.as_str() {
                "aarch64" => "arm64",
                "x86_64" => "amd64",
                "x86" => "i386",
                x => x,
            }
            .to_owned()
        })
        .unwrap_or_else(|| "common".into())
}
fn normalize(rhs: &str) -> String {
    let mut rhs = ws_re().replace_all(rhs.trim(), " ").into_owned();
    if rhs.contains("::") && !rhs.starts_with('*') {
        rhs = rhs.rsplit("::").next().unwrap_or(&rhs).into();
    }
    rhs
}
fn alias_facts(text: &str) -> (Vec<(String, String)>, HashMap<String, String>) {
    let mut decls = Vec::new();
    let mut aliases = HashMap::new();
    let mut blocks = Vec::<bool>::new();
    let mut pending = false;
    let lines: Vec<_> = text.lines().collect();
    for (idx, line) in lines.iter().enumerate() {
        let m = type_re().captures(line);
        let in_impl = blocks.iter().any(|v| *v);
        if impl_re().is_match(line) {
            pending = true;
        }
        for brace in line.char_indices().filter(|(_, c)| matches!(c, '{' | '}')) {
            match brace.1 {
                '{' => {
                    blocks.push(pending);
                    pending = false
                }
                '}' => {
                    blocks.pop();
                }
                _ => {}
            }
        }
        let Some(m) = m else { continue };
        if in_impl {
            continue;
        }
        let (Some(name), Some(rhs)) = (m.get(1), m.get(2)) else {
            continue;
        };
        let name = name.as_str().to_owned();
        let rhs = normalize(rhs.as_str());
        // Keep cfg-gated declarations as alias targets but do not compare their widths.
        let mut gated = false;
        for back in 1..=3 {
            if idx < back {
                break;
            }
            let prev = lines[idx - back].trim();
            if prev.starts_with("#[cfg(") {
                gated = true;
                break;
            }
            if !(prev.starts_with("//") || prev.starts_with("#[")) {
                break;
            }
        }
        aliases.entry(name.clone()).or_insert_with(|| rhs.clone());
        if gated {
            continue;
        }
        if nolint_re().is_match(line)
            || rhs.contains('<')
            || rhs.contains("dyn ")
            || rhs.contains("impl ")
            || rhs.contains("fn(")
        {
            continue;
        }
        decls.push((name, rhs));
    }
    (decls, aliases)
}
fn width_map() -> &'static HashMap<&'static str, u8> {
    static M: OnceLock<HashMap<&'static str, u8>> = OnceLock::new();
    M.get_or_init(|| {
        [
            ("u8", 1),
            ("i8", 1),
            ("bool", 1),
            ("UCHAR", 1),
            ("CHAR", 1),
            ("BYTE", 1),
            ("BOOLEAN", 1),
            ("CCHAR", 1),
            ("u16", 2),
            ("i16", 2),
            ("USHORT", 2),
            ("SHORT", 2),
            ("WORD", 2),
            ("WCHAR", 2),
            ("LANGID", 2),
            ("u32", 4),
            ("i32", 4),
            ("f32", 4),
            ("char", 4),
            ("ULONG", 4),
            ("LONG", 4),
            ("DWORD", 4),
            ("ULONG32", 4),
            ("LONG32", 4),
            ("UINT", 4),
            ("INT", 4),
            ("NTSTATUS", 4),
            ("HRESULT", 4),
            ("BOOL", 4),
            ("u64", 8),
            ("i64", 8),
            ("f64", 8),
            ("ULONGLONG", 8),
            ("LONGLONG", 8),
            ("ULONG64", 8),
            ("LONG64", 8),
            ("DWORDLONG", 8),
            ("LARGE_INTEGER", 8),
            ("ULARGE_INTEGER", 8),
            ("DWORD64", 8),
            ("u128", 16),
            ("i128", 16),
        ]
        .into_iter()
        .collect()
    })
}
fn ptr_name(s: &str) -> bool {
    matches!(
        s,
        "usize"
            | "isize"
            | "ULONG_PTR"
            | "LONG_PTR"
            | "DWORD_PTR"
            | "SIZE_T"
            | "SSIZE_T"
            | "UINT_PTR"
            | "INT_PTR"
            | "PVOID"
            | "HANDLE"
            | "KAFFINITY"
    )
}
fn width_of(rhs: &str, aliases: &HashMap<String, String>, arch: &str, depth: u8) -> Option<u8> {
    if depth > 8 {
        return None;
    }
    let rhs = normalize(rhs);
    if rhs.starts_with('*') || rhs.starts_with('&') {
        return None;
    }
    if let Some(w) = width_map().get(rhs.as_str()) {
        return Some(*w);
    }
    if ptr_name(&rhs) {
        return match arch {
            "i386" => Some(4),
            "amd64" | "arm64" => Some(8),
            _ => None,
        };
    }
    aliases
        .get(&rhs)
        .filter(|v| normalize(v) != rhs)
        .and_then(|v| width_of(v, aliases, arch, depth + 1))
}
fn crate_of(rel: &str) -> String {
    static R: OnceLock<Regex> = OnceLock::new();
    let re = R.get_or_init(|| crate::regexes::cached(r"(?:^|/)sdk/lib/rust/([^/]+)/").unwrap());
    re.captures(rel)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_owned())
        .unwrap_or_else(|| rel.to_owned())
}
fn contradictions(
    decls: &BTreeMap<String, Vec<(String, String, String)>>,
    aliases: &BTreeMap<String, HashMap<String, String>>,
) -> Vec<(String, String, String, u8)> {
    let common = aliases.get("common").cloned().unwrap_or_default();
    let mut merged = BTreeMap::new();
    for (arch, local) in aliases {
        let mut table = common.clone();
        table.extend(local.clone());
        merged.insert(arch.clone(), table);
    }
    let mut out = BTreeSet::new();
    for (name, sites) in decls {
        if sites.len() < 2 {
            continue;
        }
        let mut groups: Vec<Vec<_>> = Vec::new();
        if matches!(
            name.as_str(),
            "TCHAR" | "_TCHAR" | "TBYTE" | "BCHAR" | "_TUCHAR" | "_TSCHAR" | "_TXCHAR"
        ) {
            let mut m: BTreeMap<String, Vec<_>> = BTreeMap::new();
            for s in sites {
                m.entry(crate_of(&s.0)).or_default().push(s);
            }
            groups.extend(m.into_values());
        } else {
            groups.push(sites.iter().collect());
        }
        for group in groups {
            if group.len() < 2 {
                continue;
            }
            let mut widths: BTreeMap<String, BTreeSet<u8>> = BTreeMap::new();
            let mut resolved = Vec::new();
            for (rel, rhs, arch) in group {
                let table = merged.get(arch).unwrap_or(&common);
                if let Some(w) = width_of(
                    rhs,
                    table,
                    if arch == "common" { "common" } else { arch },
                    0,
                ) {
                    widths.entry(arch.clone()).or_default().insert(w);
                    resolved.push((rel.clone(), rhs.clone(), arch.clone(), w));
                }
            }
            let common_w: BTreeSet<_> = resolved
                .iter()
                .filter(|(_, _, a, _)| a == "common")
                .map(|(_, _, _, w)| *w)
                .collect();
            let mut bad: BTreeSet<String> = widths
                .iter()
                .filter(|(_, v)| v.len() > 1)
                .map(|(a, _)| a.clone())
                .collect();
            for (a, ws) in &widths {
                if a != "common" && !common_w.is_empty() && ws.union(&common_w).count() > 1 {
                    bad.insert(a.clone());
                    bad.insert("common".into());
                }
            }
            for (rel, rhs, arch, w) in resolved {
                if bad.contains(&arch) {
                    out.insert((name.clone(), rel, rhs, w));
                }
            }
        }
    }
    out.into_iter().collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    const FP:&str="#[cfg(target_pointer_width = \"64\")]\npub type UINTN = u64;\n#[cfg(target_pointer_width = \"32\")]\npub type UINTN = u32;\n\ntrait Width { type Ch; }\nimpl Width for Narrow { type Ch = i8; }\nimpl Width for Wide { type Ch = u16; }\n";
    #[test]
    fn bad_and_good_fixtures_preserve_cfg_and_associated_types() {
        let mut files = vec![
            crate::SourceFile::new(
                "sdk/lib/rust/fat-rust/src/dirsup.rs".into(),
                format!("pub type VBO = ULONG32;\n{FP}"),
            ),
            crate::SourceFile::new(
                "sdk/lib/rust/fat-rust/src/dirctrl.rs".into(),
                "type VBO = LARGE_INTEGER;\n".into(),
            ),
        ];
        let corpus = Corpus::new("/repo".into(), files.drain(..).collect());
        let hits = TypeAliasContradictions.run(&corpus).unwrap();
        assert_eq!(
            hits,
            vec![
                "sdk/lib/rust/fat-rust/src/dirctrl.rs: VBO = LARGE_INTEGER (8 bytes)",
                "sdk/lib/rust/fat-rust/src/dirsup.rs: VBO = ULONG32 (4 bytes)"
            ]
        );
        let good = Corpus::new(
            "/repo".into(),
            vec![
                crate::SourceFile::new(
                    "sdk/lib/rust/fat-rust/src/dirsup.rs".into(),
                    format!("pub type VBO = ULONG32;\n{FP}"),
                ),
                crate::SourceFile::new(
                    "sdk/lib/rust/fat-rust/src/dirctrl.rs".into(),
                    "use crate::types::VBO;\n".into(),
                ),
            ],
        );
        assert!(TypeAliasContradictions.run(&good).unwrap().is_empty());
    }
    #[test]
    fn widths_arch_buckets_and_alias_resolution() {
        assert_eq!(
            width_of("crate::types::ULONG", &HashMap::new(), "common", 0),
            Some(4)
        );
        assert_eq!(width_of("ULONG_PTR", &HashMap::new(), "i386", 0), Some(4));
        assert_eq!(width_of("ULONG_PTR", &HashMap::new(), "common", 0), None);
        let aliases = HashMap::from([("VBO".into(), "ULONG32".into())]);
        assert_eq!(width_of("VBO", &aliases, "common", 0), Some(4));
    }
}
