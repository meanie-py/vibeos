//! Find numeric Rust call arguments where the embedded C twin names a constant.
use crate::{Corpus, Rule, SourceFile, SourceSet};
use regex::Regex;
use std::{
    collections::{BTreeSet, HashMap},
    path::{Path, PathBuf},
    sync::LazyLock,
};

pub struct CallLiteralVsC;
const INPUTS: &[SourceSet] = &[SourceSet::tree(
    "call-literal-rust",
    &["sdk/lib/rust"],
    &["rs"],
    &["Cargo.toml"],
)
.with_inputs()];
const MARKER: &str = "ORIGINAL C SOURCE";
static RUST_CALL: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"([A-Za-z_]\w*)\s*\(").unwrap());
static C_CALL: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"([A-Za-z_]\w*)\s*\(").unwrap());
static ARGS: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"(?s)//[^\n]*|/\*.*?\*/").unwrap());
static INT: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"^\(?\s*(-?)\s*(0x[0-9a-fA-F_]+|\d[\d_]*)(?:[iu](?:8|16|32|64|size))?\s*\)?(?:\s+as\s+[\w:]+)*$").unwrap()
});
static C_CONST: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"^\(?\s*(?:\([A-Z_]+\)\s*)?([A-Za-z][A-Za-z0-9_]{3,})\s*\)?$").unwrap()
});
static ALL_CAPS: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"^[A-Z][A-Z0-9_]{3,}$").unwrap());
static ISLAND_ENUM: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"(?m)^pub const (\w+): (\w+) = (-?\d+);").unwrap());
static CONST: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"\bconst\s+([A-Za-z_]\w*)\s*:\s*[\w:]+\s*=\s*\(?\s*(-?)\s*(0x[0-9a-fA-F_]+|\d[\d_]*)(?:[iu](?:8|16|32|64|128|size))?\s*\)?\s*(?:as\s+[\w:]+\s*)?;").unwrap()
});
static KEYWORDS: &[&str] = &[
    "if", "while", "for", "match", "return", "switch", "sizeof", "fn", "loop",
];
static NOLINT: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"nolint:\s*call-literal-vs-c\b").unwrap());
static SKIP: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(
        r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/island_[a-z0-9_]+\.rs$)",
    )
    .unwrap()
});

#[derive(Clone)]
struct Call {
    start: usize,
    name: String,
    args: Vec<String>,
}
fn allowed_call(text: &str, at: usize, rust: bool) -> bool {
    let prev = text[..at].chars().next_back();
    match prev {
        None => true,
        Some(c) => {
            !(c.is_alphanumeric()
                || c == '_'
                || c == '.'
                || (rust && c == '!')
                || (rust && c == ':')
                || (!rust && c == '.'))
        }
    }
}
fn call_args(text: &str, open: usize) -> Option<Vec<String>> {
    let mut depth = 0i32;
    let mut start = open + 1;
    let mut out = Vec::new();
    for (i, c) in text[open..].char_indices().map(|(i, c)| (open + i, c)) {
        if matches!(c, '(' | '[' | '{') {
            depth += 1;
            if depth == 1 {
                continue;
            }
        } else if matches!(c, ')' | ']' | '}') {
            depth -= 1;
            if depth == 0 {
                let arg = text[start..i].trim();
                if !arg.is_empty() {
                    out.push(arg.into())
                }
                return Some(out);
            }
        } else if c == ',' && depth == 1 {
            out.push(text[start..i].trim().into());
            start = i + 1;
            continue;
        }
    }
    None
}
fn calls(text: &str, rust: bool) -> Vec<Call> {
    let re = if rust { &*RUST_CALL } else { &*C_CALL };
    let mut out = Vec::new();
    for m in re.captures_iter(text) {
        let whole = m.get(0).unwrap();
        let name = &m[1];
        if KEYWORDS.contains(&name) || !allowed_call(text, whole.start(), rust) {
            continue;
        }
        if let Some(args) = call_args(text, whole.end() - 1) {
            out.push(Call {
                start: whole.start(),
                name: name.into(),
                args,
            })
        }
    }
    out
}
fn blank_comments(text: &str) -> String {
    ARGS.replace_all(text, |m: &regex::Captures<'_>| {
        m[0].chars()
            .map(|c| if c == '\n' { '\n' } else { ' ' })
            .collect::<String>()
    })
    .into_owned()
}
fn int_value(text: &str) -> Option<i64> {
    let m = INT.captures(text)?;
    let raw = m.get(2)?.as_str().replace('_', "");
    let n = if raw.starts_with("0x") {
        i64::from_str_radix(&raw[2..], 16).ok()?
    } else {
        raw.parse().ok()?
    };
    Some(if &m[1] == "-" { -n } else { n })
}
fn parsed_number(raw: &str) -> Option<i64> {
    let x = raw.replace('_', "");
    if x.starts_with("0x") {
        i64::from_str_radix(&x[2..], 16).ok()
    } else {
        x.parse().ok()
    }
}
fn crate_root(path: &Path, manifests: &[PathBuf]) -> PathBuf {
    manifests
        .iter()
        .filter_map(|m| m.parent())
        .filter(|p| path.starts_with(p))
        .max_by_key(|p| p.components().count())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| path.parent().unwrap_or(Path::new("")).to_path_buf())
}
fn islands(files: &[&SourceFile]) -> HashMap<String, BTreeSet<i64>> {
    let mut out = HashMap::new();
    for f in files {
        let parts: Vec<_> = f
            .path
            .components()
            .map(|x| x.as_os_str().to_string_lossy().into_owned())
            .collect();
        let file = f.path.file_name().unwrap_or_default().to_string_lossy();
        let is_sys = parts.iter().any(|x| x.ends_with("-sys"));
        if !is_sys
            || !parts.windows(2).any(|x| x[0] == "src")
            || !file.starts_with("bindgen_")
            || !file.ends_with(".rs")
        {
            continue;
        }
        for m in ISLAND_ENUM.captures_iter(&f.text) {
            let name = &m[1];
            let ty = &m[2];
            if name.starts_with(&format!("{ty}_")) {
                let member = name[ty.len() + 1..].to_string();
                if let Ok(n) = m[3].parse() {
                    out.entry(member).or_insert_with(BTreeSet::new).insert(n);
                }
            }
        }
    }
    out
}
fn consts(files: &[&SourceFile], root: &Path) -> HashMap<String, BTreeSet<i64>> {
    let mut out = HashMap::new();
    for f in files {
        if f.path.extension().map_or(true, |x| x != "rs") || !f.path.starts_with(root) {
            continue;
        }
        for m in CONST.captures_iter(&f.text) {
            if let Some(mut n) = parsed_number(&m[3]) {
                if &m[2] == "-" {
                    n = -n
                }
                out.entry(m[1].to_string())
                    .or_insert_with(BTreeSet::new)
                    .insert(n);
            }
        }
    }
    out
}
impl Rule for CallLiteralVsC {
    fn name(&self) -> &'static str {
        "scan_call_literal_vs_c"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let selected: Vec<_> = c.sources("call-literal-rust").collect();
        let rs: Vec<_> = selected
            .iter()
            .copied()
            .filter(|f| f.path.extension().is_some_and(|x| x == "rs"))
            .collect();
        let manifests: Vec<_> = selected
            .iter()
            .filter(|f| f.path.file_name().is_some_and(|n| n == "Cargo.toml"))
            .map(|f| f.path.clone())
            .collect();
        let enums = islands(&rs);
        let mut crate_cache: HashMap<PathBuf, HashMap<String, BTreeSet<i64>>> = HashMap::new();
        let mut found = Vec::<(PathBuf, usize, String)>::new();
        for f in &rs {
            let path = format!("/{}", f.path.to_string_lossy().replace('\\', "/"));
            if SKIP.is_match(&path) {
                continue;
            }
            let Some(cut) = f.text.find(MARKER) else {
                continue;
            };
            let raw_c = &f.text[cut..];
            let mut c_text = String::new();
            for line in raw_c.lines() {
                let l = line.trim_start();
                if let Some(rest) = l.strip_prefix("//") {
                    let content = rest
                        .chars()
                        .next()
                        .filter(|ch| ch.is_whitespace())
                        .map_or(rest, |ch| &rest[ch.len_utf8()..]);
                    c_text.push_str(content);
                } else {
                    c_text.push_str(line)
                }
                c_text.push('\n')
            }
            let c_calls = calls(&c_text, false);
            let mut by_name: HashMap<(String, usize), Vec<Vec<String>>> = HashMap::new();
            for call in c_calls {
                by_name
                    .entry((call.name, call.args.len()))
                    .or_default()
                    .push(call.args)
            }
            let rust = &f.text[..cut];
            let code = blank_comments(rust);
            let newlines: Vec<_> = code.match_indices('\n').map(|(at, _)| at).collect();
            let lines: Vec<_> = rust.lines().collect();
            let root = crate_root(&f.path, &manifests);
            let local = crate_cache
                .entry(root.clone())
                .or_insert_with(|| consts(&rs, &root));
            for call in calls(&code, true) {
                let Some(cands) = by_name.get(&(call.name.clone(), call.args.len())) else {
                    continue;
                };
                for (i, arg) in call.args.iter().enumerate() {
                    let Some(lit) = int_value(arg) else { continue };
                    let mut names = BTreeSet::new();
                    for ca in cands {
                        let Some(m) = C_CONST.captures(&ca[i]) else {
                            names.clear();
                            break;
                        };
                        let name = m[1].to_string();
                        if [
                            "TRUE", "FALSE", "NULL", "VOID", "ULONG", "USHORT", "UCHAR", "BOOLEAN",
                        ]
                        .contains(&name.as_str())
                        {
                            names.clear();
                            break;
                        }
                        names.insert(name);
                    }
                    if names.len() != 1 {
                        continue;
                    }
                    let cname = names.iter().next().unwrap();
                    let island = enums.get(cname);
                    let known = if island.is_some_and(|values| values.len() == 1) {
                        island
                    } else if ALL_CAPS.is_match(cname) {
                        local.get(cname)
                    } else {
                        // A camel-case C identifier without one unambiguous bindgen
                        // enum value is a variable, not a named constant candidate.
                        continue;
                    };
                    if known.is_some_and(|x| x.contains(&lit)) {
                        continue;
                    }
                    let line = newlines.partition_point(|at| *at < call.start) + 1;
                    if lines.get(line - 1).is_some_and(|l| NOLINT.is_match(l)) {
                        continue;
                    }
                    let kind = if known.is_some() {
                        "MISMATCH"
                    } else {
                        "UNRESOLVED"
                    };
                    let want = known
                        .map(|x| format!("{cname}={:?}", x.iter().collect::<Vec<_>>()))
                        .unwrap_or_else(|| cname.clone());
                    found.push((
                        f.path.clone(),
                        line,
                        format!("{kind} {} arg{i}: {arg} where C passes {want}", call.name),
                    ));
                }
            }
        }
        found.sort();
        Ok(found
            .into_iter()
            .map(|(p, l, w)| format!("{}:{l}: call-literal-vs-c {w}", p.display()))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    #[test]
    fn unicode_comments_preserve_python_call_line_offsets() {
        let text = "// éééééééééééééééééééé\nconst MY_CONST: u32 = 1;\nfn f() {\n Frob(0);\n}\n// ORIGINAL C SOURCE\n// Frob(MY_CONST);\n";
        let c = Corpus::new(
            ".".into(),
            vec![SourceFile::new(
                "sdk/lib/rust/demo/src/lib.rs".into(),
                text.into(),
            )],
        );
        let got = CallLiteralVsC.run(&c).unwrap();
        assert_eq!(got,vec!["sdk/lib/rust/demo/src/lib.rs:4: call-literal-vs-c MISMATCH Frob arg0: 0 where C passes MY_CONST=[1]"]);
    }
    #[test]
    fn parser_honors_nested_args_and_call_boundaries() {
        assert_eq!(
            call_args("f(a, g(b,c), d)", 1).unwrap(),
            vec!["a", "g(b,c)", "d"]
        );
        assert!(calls("foo!(); obj.foo(); ::foo(); foo();", true)
            .iter()
            .any(|x| x.name == "foo"));
        assert_eq!(int_value("0x01u32"), Some(1));
    }
    #[test]
    fn enum_and_crate_constant_oracles() {
        let f = SourceFile::new(
            "sdk/lib/rust/x-sys/src/bindgen_a.rs".into(),
            "pub const HAL_REBOOT_ROUTINE_HalRebootRoutine: HAL_REBOOT_ROUTINE = 3;".into(),
        );
        let e = islands(&[&f]);
        assert_eq!(
            e["HalRebootRoutine"].iter().copied().collect::<Vec<_>>(),
            vec![3]
        );
    }
    #[test]
    fn dedicated_enum_mismatch_and_island_precedence() {
        let rust = "pub unsafe fn f() { HalReturnToFirmware(1); }\n// ORIGINAL C SOURCE\n// HalReturnToFirmware(HalRebootRoutine);\n";
        let corpus = Corpus::new(
            PathBuf::from("."),
            vec![
                SourceFile::new("sdk/lib/rust/demo/Cargo.toml".into(), "[package]".into()),
                SourceFile::new("sdk/lib/rust/demo/src/lib.rs".into(), rust.into()),
                SourceFile::new("sdk/lib/rust/demo/src/constants.rs".into(), "const MmFlushForWrite: u32 = 2;\n".into()),
                SourceFile::new("sdk/lib/rust/demo-sys/src/bindgen_mm.rs".into(), "pub const HAL_REBOOT_ROUTINE_HalRebootRoutine: HAL_REBOOT_ROUTINE = 3;\npub const MM_FLUSH_TYPE_MmFlushForWrite: MM_FLUSH_TYPE = 1;\n".into()),
            ],
        );
        let got = CallLiteralVsC.run(&corpus).unwrap();
        assert_eq!(got, vec!["sdk/lib/rust/demo/src/lib.rs:1: call-literal-vs-c MISMATCH HalReturnToFirmware arg0: 1 where C passes HalRebootRoutine=[3]"]);
        let good = "pub unsafe fn f() { MmFlushImageSection(s, 1); }\n// ORIGINAL C SOURCE\n// MmFlushImageSection(Section, MmFlushForWrite);\n";
        let corpus = Corpus::new(
            PathBuf::from("."),
            vec![
                SourceFile::new("sdk/lib/rust/demo/Cargo.toml".into(), "[package]".into()),
                SourceFile::new("sdk/lib/rust/demo/src/lib.rs".into(), good.into()),
                SourceFile::new(
                    "sdk/lib/rust/demo/src/constants.rs".into(),
                    "const MmFlushForWrite: u32 = 2;\n".into(),
                ),
                SourceFile::new(
                    "sdk/lib/rust/demo-sys/src/bindgen_mm.rs".into(),
                    "pub const MM_FLUSH_TYPE_MmFlushForWrite: MM_FLUSH_TYPE = 1;\n".into(),
                ),
            ],
        );
        assert!(CallLiteralVsC.run(&corpus).unwrap().is_empty());
    }
    #[test]
    fn dedicated_camel_case_variable_is_not_a_constant() {
        let text =
            "pub unsafe fn f() { Frob(0); }\n// ORIGINAL C SOURCE\n// Frob(SomeLocalVariable);\n";
        let corpus = Corpus::new(
            PathBuf::from("."),
            vec![
                SourceFile::new("sdk/lib/rust/demo/Cargo.toml".into(), "[package]".into()),
                SourceFile::new("sdk/lib/rust/demo/src/lib.rs".into(), text.into()),
            ],
        );
        assert!(CallLiteralVsC.run(&corpus).unwrap().is_empty());
    }
    #[test]
    fn scanner_controls_bad_and_good() {
        let bad="const CM_CHECK_REGISTRY_PURGE_VOLATILES: u32 = 0x00000002;\npub unsafe fn load() {\n    CmpCmdHiveOpen(file, ctx, &mut a, &mut h, 0x00000001u32);\n}\n// ============================================================================\n// ORIGINAL C SOURCE\n// ============================================================================\n//     Status = CmpCmdHiveOpen(SourceFile, &Ctx, &Allocate, &CmHive,\n//                             CM_CHECK_REGISTRY_PURGE_VOLATILES);\n";
        let good="const CM_CHECK_REGISTRY_PURGE_VOLATILES: u32 = 0x00000002;\npub unsafe fn load() {\n    CmpCmdHiveOpen(file, ctx, &mut a, &mut h, 2u32);\n    SystemParametersInfoW(SPI_GETSCREENSAVETIMEOUT, 0, &mut t, 0);\n}\n// ============================================================================\n// ORIGINAL C SOURCE\n// ============================================================================\n//     Status = CmpCmdHiveOpen(SourceFile, &Ctx, &Allocate, &CmHive,\n//                             CM_CHECK_REGISTRY_PURGE_VOLATILES);\n//     if (!SystemParametersInfoW(SPI_GETSCREENSAVETIMEOUT, 0, &Timeout, 0))\n//         SystemParametersInfoW(SPI_SETSCREENSAVETIMEOUT, 600, 0, SPIF_UPDATEINIFILE);\n";
        let manifest = SourceFile::new("bad/Cargo.toml".into(), "[package]".into());
        let f = SourceFile::new("bad.rs".into(), bad.into());
        let c = Corpus::new(PathBuf::from("."), vec![manifest, f]);
        let got = CallLiteralVsC.run(&c).unwrap();
        assert_eq!(got.len(), 1);
        assert!(got[0].contains("MISMATCH CmpCmdHiveOpen arg4"));
        let c = Corpus::new(
            PathBuf::from("."),
            vec![
                SourceFile::new("good/Cargo.toml".into(), "[package]".into()),
                SourceFile::new("good.rs".into(), good.into()),
            ],
        );
        assert!(CallLiteralVsC.run(&c).unwrap().is_empty());
    }
}
