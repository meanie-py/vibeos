//! Rust port of `gen_ctype_tables.py --check <ctype.rs>`.
use crate::{Corpus, Rule, SourceFile, SourceSet};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct GenCtypeTables;
const INPUTS: &[SourceSet] = &[SourceSet::tree(
    "gen-ctype-inputs",
    &[
        "dll/win32/msvcrt",
        "sdk/lib/rust/msvcrt-sys/src",
        "sdk/lib/rust/msvcrt-crt-rust/src/msvcrt",
    ],
    &["c", "rs"],
    &[],
)
.with_inputs()];
const BEGIN: &str = "// <gen_ctype_tables BEGIN>";
const END: &str = "// <gen_ctype_tables END>";
const ABBREV: &[(&str, &str)] = &[
    ("_C_", "_CONTROL"),
    ("_S_", "_SPACE"),
    ("_P_", "_PUNCT"),
    ("_D_", "_DIGIT"),
    ("_H_", "_HEX"),
    ("_U_", "_UPPER"),
    ("_L_", "_LOWER"),
];
const GATED: &[(&str, &str, &str)] = &[
    ("B110", "_BLANK", "_MSVCR_VER <= 110"),
    ("D120", "4", "_MSVCR_VER != 120"),
    ("S140", "0", "_MSVCR_VER < 140"),
    ("L140", "0", "_MSVCR_VER < 140"),
    ("C140", "0", "_MSVCR_VER < 140"),
];
const MASKS: &[&str] = &[
    "_UPPER", "_LOWER", "_DIGIT", "_SPACE", "_PUNCT", "_CONTROL", "_BLANK", "_HEX",
];

impl Rule for GenCtypeTables {
    fn needs_workspace_rust(&self) -> bool {
        false
    }
    fn name(&self) -> &'static str {
        "gen_ctype_tables"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let find = |path: &str| {
            corpus
                .sources("gen-ctype-inputs")
                .find(|f| f.path == Path::new(path))
        };
        let c = find("dll/win32/msvcrt/ctype.c")
            .ok_or("gen_ctype_tables: missing dll/win32/msvcrt/ctype.c")?;
        let island = find("sdk/lib/rust/msvcrt-sys/src/bindgen_i386.rs")
            .ok_or("gen_ctype_tables: missing msvcrt-sys bindgen_i386.rs")?;
        let target = find("sdk/lib/rust/msvcrt-crt-rust/src/msvcrt/ctype.rs")
            .ok_or("gen_ctype_tables: missing ctype.rs target")?;
        let masks = island_masks(&island.text)?;
        let narrow_src = initializer(&c.text, "WORD MSVCRT__ctype [257]")?;
        let wide_src = initializer(&c.text, "WORD MSVCRT__wctype[257]")?;
        let narrow = evaluate(&narrow_src, &masks)?;
        let wide = evaluate(&wide_src, &masks)?;
        if let Some((compiled_narrow, compiled_wide)) = object_tables(&corpus.root)? {
            prove("MSVCRT__ctype", &narrow_src, &narrow, &compiled_narrow)?;
            prove("MSVCRT__wctype", &wide_src, &wide, &compiled_wide)?;
        }
        let block = build_block(&narrow_src, &wide_src, &narrow, &wide, &masks);
        let have = target
            .text
            .split_once(BEGIN)
            .and_then(|(_, rest)| rest.split_once(END).map(|(have, _)| have))
            .ok_or_else(|| {
                format!(
                    "gen_ctype_tables: {} has no {BEGIN} .. {END} block",
                    target.path.display()
                )
            })?
            .trim_matches('\n');
        if have.trim() != block.trim() {
            return Ok(vec![format!(
                "MISMATCH: {}: ctype table block is stale vs dll/win32/msvcrt/ctype.c",
                target.path.display()
            )]);
        }
        Ok(Vec::new())
    }
}

fn island_masks(src: &str) -> Result<BTreeMap<String, u32>, String> {
    let mut out = BTreeMap::new();
    for name in MASKS {
        let prefix = format!("pub const {name}: u32 = ");
        let value = src
            .lines()
            .find_map(|line| line.strip_prefix(&prefix).and_then(|s| s.strip_suffix(';')))
            .ok_or_else(|| {
                format!("gen_ctype_tables: {name} is not in msvcrt-sys bindgen_i386.rs")
            })?
            .parse::<u32>()
            .map_err(|e| format!("gen_ctype_tables: invalid {name} mask: {e}"))?;
        out.insert((*name).to_owned(), value);
    }
    Ok(out)
}

fn initializer(src: &str, decl: &str) -> Result<Vec<String>, String> {
    let at = src
        .find(decl)
        .ok_or_else(|| format!("gen_ctype_tables: missing initializer {decl}"))?;
    let start = at
        + src[at..]
            .find('{')
            .ok_or_else(|| format!("gen_ctype_tables: missing '{{' after {decl}"))?;
    let end = start
        + src[start..]
            .find("};")
            .ok_or_else(|| format!("gen_ctype_tables: missing '}};' after {decl}"))?;
    let body = strip_block_comments(&src[start + 1..end]);
    let entries = body
        .split(',')
        .map(str::trim)
        .filter(|x| !x.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if entries.len() != 257 {
        return Err(format!(
            "gen_ctype_tables: {decl} parsed to {} entries, expected 257",
            entries.len()
        ));
    }
    Ok(entries)
}

fn strip_block_comments(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        let tail = &rest[start + 2..];
        let Some(end) = tail.find("*/") else {
            break;
        };
        out.push_str(&" ".repeat(0));
        rest = &tail[end + 2..];
    }
    out.push_str(rest);
    out
}

fn evaluate(entries: &[String], masks: &BTreeMap<String, u32>) -> Result<Vec<u16>, String> {
    let mut env = masks.clone();
    for (short, long) in ABBREV {
        env.insert((*short).into(), masks[*long]);
    }
    for (name, value, _) in GATED {
        let value = masks
            .get(*value)
            .copied()
            .or_else(|| value.parse().ok())
            .ok_or_else(|| format!("gen_ctype_tables: invalid gated value {value}"))?;
        env.insert((*name).into(), value);
    }
    entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let v = eval_expr(e, &env)
                .ok_or_else(|| format!("gen_ctype_tables: entry {i} {e:?} did not evaluate"))?;
            u16::try_from(v)
                .map_err(|_| format!("gen_ctype_tables: entry {i} {e:?} = {v} does not fit a WORD"))
        })
        .collect()
}

fn eval_expr(s: &str, env: &BTreeMap<String, u32>) -> Option<u32> {
    let mut tokens = Vec::<String>::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
        } else if bytes[i] == b'|' || bytes[i] == b'(' || bytes[i] == b')' {
            tokens.push((bytes[i] as char).to_string());
            i += 1;
        } else {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            if start == i {
                return None;
            }
            tokens.push(s[start..i].to_owned());
        }
    }
    fn atom(tokens: &[String], at: &mut usize, env: &BTreeMap<String, u32>) -> Option<u32> {
        match tokens.get(*at)?.as_str() {
            "(" => {
                *at += 1;
                let value = expr(tokens, at, env)?;
                if tokens.get(*at)?.as_str() != ")" {
                    return None;
                }
                *at += 1;
                Some(value)
            }
            x => {
                *at += 1;
                if let Some(hex) = x.strip_prefix("0x").or_else(|| x.strip_prefix("0X")) {
                    u32::from_str_radix(hex, 16).ok()
                } else if x.chars().all(|c| c.is_ascii_digit()) {
                    x.parse().ok()
                } else {
                    env.get(x).copied()
                }
            }
        }
    }
    fn expr(tokens: &[String], at: &mut usize, env: &BTreeMap<String, u32>) -> Option<u32> {
        let mut value = atom(tokens, at, env)?;
        while tokens.get(*at).is_some_and(|x| x == "|") {
            *at += 1;
            value |= atom(tokens, at, env)?;
        }
        Some(value)
    }
    let mut at = 0;
    let out = expr(&tokens, &mut at, env)?;
    (at == tokens.len()).then_some(out)
}

fn render(name: &str, entries: &[String]) -> String {
    let mut lines = Vec::new();
    for row in (0..257).step_by(8) {
        let end = (row + 8).min(257);
        let tag = if row == 0 {
            "EOF + 0x00".to_owned()
        } else {
            format!("0x{:02x}", row - 1)
        };
        lines.push(format!("    /* {tag} */ {},", entries[row..end].join(", ")));
    }
    let mut body = lines.join("\n");
    body = body.trim_end_matches(',').to_owned();
    format!("#[no_mangle]\npub static mut {name}: [WORD; 257] = [\n{body}\n];")
}

fn build_block(
    narrow_src: &[String],
    wide_src: &[String],
    narrow: &[u16],
    wide: &[u16],
    masks: &BTreeMap<String, u32>,
) -> String {
    let direct = narrow_src
        .iter()
        .chain(wide_src)
        .flat_map(|e| e.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')))
        .filter(|n| masks.contains_key(*n))
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let mut aliases = vec![
        "// ctype.c's own abbreviations, so the two tables below stay diffable against".to_owned(),
        "// the C line by line. WORD-typed, because the island types the masks u32.".to_owned(),
        "#[allow(non_upper_case_globals)]".to_owned(),
    ];
    aliases.extend(
        ABBREV
            .iter()
            .map(|(a, r)| format!("const {a}: WORD = msvcrt_sys::{r} as WORD;")),
    );
    aliases.extend(
        direct
            .iter()
            .map(|n| format!("const {n}: WORD = msvcrt_sys::{n} as WORD;")),
    );
    aliases.push(String::new());
    aliases
        .push("// The `_MSVCR_VER`-gated arms, resolved for THIS DLL (`-D_MSVCR_VER=0`).".into());
    aliases.push("#[allow(non_upper_case_globals)]".into());
    aliases.extend(GATED.iter().map(|(name, value, why)| {
        let v = if value.starts_with('_') {
            format!("msvcrt_sys::{value} as WORD")
        } else {
            (*value).to_owned()
        };
        format!("const {name}: WORD = {v};  // {why}")
    }));
    format!(
        "{}\n\n{}\n\n{}\n",
        aliases.join("\n"),
        render("MSVCRT__ctype", narrow_src),
        render("MSVCRT__wctype", wide_src)
    )
}

fn object_tables(root: &Path) -> Result<Option<(Vec<u16>, Vec<u16>)>, String> {
    let obj = root.join("build-i386/dll/win32/msvcrt/CMakeFiles/msvcrt_static.dir/ctype.c.obj");
    if !obj.exists() {
        return Ok(None);
    }
    let nm = root.join(".build-tools/llvm-mingw/bin/llvm-nm");
    let objcopy = root.join(".build-tools/llvm-mingw/bin/llvm-objcopy");
    let output = Command::new(&nm)
        .arg("--defined-only")
        .arg(&obj)
        .output()
        .map_err(|e| format!("gen_ctype_tables: cannot run {}: {e}", nm.display()))?;
    if !output.status.success() {
        return Err(format!(
            "gen_ctype_tables: llvm-nm failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let mut offsets = BTreeMap::<String, usize>::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let parts = line.split_whitespace().collect::<Vec<_>>();
        if parts.len() == 3 && parts[1] == "D" {
            let offset = usize::from_str_radix(parts[0], 16)
                .map_err(|e| format!("gen_ctype_tables: bad symbol offset: {e}"))?;
            offsets.insert(parts[2].trim_start_matches('_').into(), offset);
        }
    }
    let narrow = *offsets
        .get("MSVCRT__ctype")
        .ok_or("gen_ctype_tables: MSVCRT__ctype is not a DATA symbol of the object")?;
    let wide = *offsets
        .get("MSVCRT__wctype")
        .ok_or("gen_ctype_tables: MSVCRT__wctype is not a DATA symbol of the object")?;
    let dump = root.join("build-i386/.ctype-data.bin");
    let output = Command::new(&objcopy)
        .arg(format!("--dump-section=.data={}", dump.display()))
        .arg(&obj)
        .arg("/dev/null")
        .output()
        .map_err(|e| format!("gen_ctype_tables: cannot run {}: {e}", objcopy.display()))?;
    if !output.status.success() {
        return Err(format!(
            "gen_ctype_tables: llvm-objcopy failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let data =
        std::fs::read(&dump).map_err(|e| format!("gen_ctype_tables: cannot read dump: {e}"))?;
    std::fs::remove_file(&dump)
        .map_err(|e| format!("gen_ctype_tables: cannot remove dump: {e}"))?;
    let read = |offset: usize| -> Result<Vec<u16>, String> {
        let bytes = data
            .get(offset..offset + 257 * 2)
            .ok_or("gen_ctype_tables: object table lies outside .data")?;
        Ok(bytes
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect())
    };
    Ok(Some((read(narrow)?, read(wide)?)))
}

fn prove(name: &str, entries: &[String], derived: &[u16], compiled: &[u16]) -> Result<(), String> {
    let bad = (0..257)
        .filter(|&i| derived[i] != compiled[i])
        .collect::<Vec<_>>();
    if bad.is_empty() {
        return Ok(());
    }
    let rows = bad
        .iter()
        .take(10)
        .map(|&i| {
            format!(
                "{name}[{i}] (char 0x{:02x}): source {:?} = {:#06x} but object has {:#06x}",
                i as i32 - 1,
                entries[i],
                derived[i],
                compiled[i]
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    Err(format!(
        "gen_ctype_tables: {} of 257 {name} entries disagree with compiled object -- do NOT ship this table\n{rows}",
        bad.len()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn masks() -> BTreeMap<String, u32> {
        [
            ("_UPPER", 1),
            ("_LOWER", 2),
            ("_DIGIT", 4),
            ("_SPACE", 8),
            ("_PUNCT", 16),
            ("_CONTROL", 32),
            ("_BLANK", 64),
            ("_HEX", 128),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect()
    }

    #[test]
    fn abbreviation_and_version_values_match_python_controls() {
        let got = evaluate(
            &[
                "_S_|_C_".into(),
                "_D_|_H_".into(),
                "_U_|_H_".into(),
                "0".into(),
                "0x0028 | B110".into(),
                "0x0010 | D120".into(),
            ],
            &masks(),
        )
        .unwrap();
        assert_eq!(got, [40, 132, 129, 0, 104, 20]);
    }

    #[test]
    fn initializer_requires_257_entries_and_strips_comments() {
        let short = format!(
            "WORD MSVCRT__ctype [257] = {{{}}};",
            vec!["0"; 256].join(",")
        );
        assert!(initializer(&short, "WORD MSVCRT__ctype [257]")
            .unwrap_err()
            .contains("expected 257"));
        let src = format!(
            "WORD MSVCRT__wctype[257] = {{{}}};",
            (0..257)
                .map(|i| format!("/* {i} */ 0"))
                .collect::<Vec<_>>()
                .join(",")
        );
        let entries = initializer(&src, "WORD MSVCRT__wctype[257]").unwrap();
        assert_eq!(entries.len(), 257);
        assert!(entries.iter().all(|e| e == "0"));
    }

    #[test]
    fn evaluator_rejects_out_of_range_values_and_proof_detects_one_mismatch() {
        assert!(evaluate(&["0x1FFFF".into()], &masks())
            .unwrap_err()
            .contains("does not fit a WORD"));
        let entries = vec!["0".to_owned(); 257];
        let derived = vec![0; 257];
        let mut compiled = derived.clone();
        compiled[65] = 0x81;
        assert!(prove("MSVCRT__ctype", &entries, &derived, &compiled)
            .unwrap_err()
            .contains("do NOT ship"));
        assert!(prove("MSVCRT__ctype", &entries, &derived, &derived).is_ok());
    }
}
