//! Shared generator for the 514 msvcrt ctype table entries.
//! Keep this crate-independent so the linter rule and the old CLI can both call it.
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

pub const BEGIN: &str = "// <gen_ctype_tables BEGIN>";
pub const END: &str = "// <gen_ctype_tables END>";
pub type Masks = BTreeMap<String, u32>;
pub type Tables = (Vec<u16>, Vec<u16>);
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
const MASK_NAMES: &[&str] = &[
    "_UPPER", "_LOWER", "_DIGIT", "_SPACE", "_PUNCT", "_CONTROL", "_BLANK", "_HEX",
];
const CTYPE: &str = "dll/win32/msvcrt/ctype.c";
const ISLAND: &str = "sdk/lib/rust/msvcrt-sys/src/bindgen_i386.rs";
const OBJ: &str = "build-i386/dll/win32/msvcrt/CMakeFiles/msvcrt_static.dir/ctype.c.obj";

pub fn island_masks(src: &str) -> Result<Masks, String> {
    let mut out = Masks::new();
    for name in MASK_NAMES {
        let prefix = format!("pub const {name}: u32 = ");
        let value = src
            .lines()
            .find_map(|l| l.strip_prefix(&prefix).and_then(|s| s.strip_suffix(';')))
            .ok_or_else(|| {
                format!("gen_ctype_tables: {name} is not in msvcrt-sys bindgen_i386.rs")
            })?
            .parse::<u32>()
            .map_err(|e| format!("gen_ctype_tables: invalid {name} mask: {e}"))?;
        out.insert((*name).to_owned(), value);
    }
    Ok(out)
}

pub fn initializer(src: &str, decl: &str) -> Result<Vec<String>, String> {
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
    let body = strip_block_comments(&src[start + 1..end])?;
    let entries = body
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
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
fn strip_block_comments(s: &str) -> Result<String, String> {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    loop {
        match rest.find("/*") {
            None => {
                out.push_str(rest);
                break;
            }
            Some(start) => {
                out.push_str(&rest[..start]);
                let tail = &rest[start + 2..];
                let end = tail
                    .find("*/")
                    .ok_or("gen_ctype_tables: unterminated block comment in initializer")?;
                out.push_str(&" ".repeat(end));
                rest = &tail[end + 2..];
            }
        }
    }
    Ok(out)
}

pub fn evaluate(entries: &[String], masks: &Masks) -> Result<Vec<u16>, String> {
    let mut env = masks.clone();
    for (short, long) in ABBREV {
        let v = masks
            .get(*long)
            .ok_or_else(|| format!("gen_ctype_tables: missing mask {long}"))?;
        env.insert((*short).into(), *v);
    }
    for (name, raw, _) in GATED {
        let v = masks
            .get(*raw)
            .copied()
            .or_else(|| raw.parse().ok())
            .ok_or_else(|| format!("gen_ctype_tables: invalid gated value {raw}"))?;
        env.insert((*name).into(), v);
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
fn eval_expr(s: &str, env: &Masks) -> Option<u32> {
    let mut tokens = Vec::<String>::new();
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i].is_ascii_whitespace() {
            i += 1;
        } else if matches!(b[i], b'|' | b'(' | b')') {
            tokens.push((b[i] as char).to_string());
            i += 1;
        } else {
            let start = i;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            if i == start {
                return None;
            }
            tokens.push(s[start..i].to_owned());
        }
    }
    fn atom(t: &[String], i: &mut usize, e: &Masks) -> Option<u32> {
        match t.get(*i)?.as_str() {
            "(" => {
                *i += 1;
                let v = expr(t, i, e)?;
                if t.get(*i)?.as_str() != ")" {
                    return None;
                }
                *i += 1;
                Some(v)
            }
            x => {
                *i += 1;
                if let Some(h) = x.strip_prefix("0x").or_else(|| x.strip_prefix("0X")) {
                    u32::from_str_radix(h, 16).ok()
                } else if x.chars().all(|c| c.is_ascii_digit()) {
                    x.parse().ok()
                } else {
                    e.get(x).copied()
                }
            }
        }
    }
    fn expr(t: &[String], i: &mut usize, e: &Masks) -> Option<u32> {
        let mut v = atom(t, i, e)?;
        while t.get(*i).is_some_and(|x| x == "|") {
            *i += 1;
            v |= atom(t, i, e)?;
        }
        Some(v)
    }
    let mut at = 0;
    let out = expr(&tokens, &mut at, env)?;
    (at == tokens.len()).then_some(out)
}

pub fn prove(
    name: &str,
    entries: &[String],
    derived: &[u16],
    compiled: &[u16],
) -> Result<(), String> {
    if entries.len() != 257 || derived.len() != 257 || compiled.len() != 257 {
        return Err(format!(
            "gen_ctype_tables: malformed proof inputs for {name}: expected 257 entries and values"
        ));
    }
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
    Err(format!("gen_ctype_tables: {} of 257 {name} entries disagree with compiled object -- do NOT ship this table\n{rows}",bad.len()))
}

pub fn build_block(
    narrow_src: &[String],
    wide_src: &[String],
    masks: &Masks,
    compiled: Option<&Tables>,
) -> Result<String, String> {
    let narrow = evaluate(narrow_src, masks)?;
    let wide = evaluate(wide_src, masks)?;
    if let Some((cn, cw)) = compiled {
        prove("MSVCRT__ctype", narrow_src, &narrow, cn)?;
        prove("MSVCRT__wctype", wide_src, &wide, cw)?;
    }
    let direct = narrow_src
        .iter()
        .chain(wide_src)
        .flat_map(|e| e.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')))
        .filter(|n| masks.contains_key(*n))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let mut a = vec![
        "// ctype.c's own abbreviations, so the two tables below stay diffable against".to_owned(),
        "// the C line by line. WORD-typed, because the island types the masks u32.".into(),
        "#[allow(non_upper_case_globals)]".into(),
    ];
    a.extend(
        ABBREV
            .iter()
            .map(|(x, y)| format!("const {x}: WORD = msvcrt_sys::{y} as WORD;")),
    );
    a.extend(
        direct
            .iter()
            .map(|n| format!("const {n}: WORD = msvcrt_sys::{n} as WORD;")),
    );
    a.push(String::new());
    a.push("// The `_MSVCR_VER`-gated arms, resolved for THIS DLL (`-D_MSVCR_VER=0`).".into());
    a.push("#[allow(non_upper_case_globals)]".into());
    a.extend(GATED.iter().map(|(n, v, w)| {
        format!(
            "const {n}: WORD = {};  // {w}",
            if v.starts_with('_') {
                format!("msvcrt_sys::{v} as WORD")
            } else {
                (*v).into()
            }
        )
    }));
    Ok(format!(
        "{}\n\n{}\n\n{}\n",
        a.join("\n"),
        render("MSVCRT__ctype", narrow_src),
        render("MSVCRT__wctype", wide_src)
    ))
}
fn render(name: &str, entries: &[String]) -> String {
    let mut lines = Vec::new();
    for row in (0..257).step_by(8) {
        let end = (row + 8).min(257);
        let tag = if row == 0 {
            "EOF + 0x00".into()
        } else {
            format!("0x{:02x}", row - 1)
        };
        lines.push(format!("    /* {tag} */ {},", entries[row..end].join(", ")));
    }
    let body = lines.join("\n").trim_end_matches(',').to_owned();
    format!("#[no_mangle]\npub static mut {name}: [WORD; 257] = [\n{body}\n];")
}

/// Reads the source, mask island and optional object exclusively under `root`.
/// `proof` is false only when this checkout has no object; that condition is warned explicitly.
pub fn generate(root: &Path) -> Result<Generated, String> {
    let c = read_inside(root, CTYPE)?;
    let island = read_inside(root, ISLAND)?;
    let masks = island_masks(&island)?;
    let n = initializer(&c, "WORD MSVCRT__ctype [257]")?;
    let w = initializer(&c, "WORD MSVCRT__wctype[257]")?;
    let object = object_tables(root)?;
    let proof = object.is_some();
    if let Some(ref tables) = object {
        let nv = evaluate(&n, &masks)?;
        let wv = evaluate(&w, &masks)?;
        prove("MSVCRT__ctype", &n, &nv, &tables.0)?;
        prove("MSVCRT__wctype", &w, &wv, &tables.1)?;
    } else {
        eprintln!("gen_ctype_tables: WARNING -- no {OBJ}, so the 514 values were NOT proven against the compiler's own output; only the source-vs-island derivation ran.\n  to get the proof: ./vibeos-cli build --arch i386");
    }
    Ok(Generated {
        block: build_block(&n, &w, &masks, object.as_ref())?,
        proof,
    })
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Generated {
    pub block: String,
    pub proof: bool,
}
fn read_inside(root: &Path, rel: &str) -> Result<String, String> {
    let path = root.join(rel);
    let base = root.canonicalize().map_err(|e| {
        format!(
            "gen_ctype_tables: cannot resolve checkout {}: {e}",
            root.display()
        )
    })?;
    let real = path
        .canonicalize()
        .map_err(|e| format!("gen_ctype_tables: cannot read {}: {e}", path.display()))?;
    if !real.starts_with(&base) {
        return Err(format!(
            "gen_ctype_tables: input resolves outside checkout: {}",
            path.display()
        ));
    }
    std::fs::read_to_string(real)
        .map_err(|e| format!("gen_ctype_tables: cannot read {}: {e}", path.display()))
}
pub fn object_tables(root: &Path) -> Result<Option<Tables>, String> {
    let obj = read_path_inside(root, OBJ)?;
    if obj.is_none() {
        return Ok(None);
    }
    let obj = obj.unwrap();
    let raw = std::fs::read(&obj)
        .map_err(|e| format!("gen_ctype_tables: cannot read {}: {e}", obj.display()))?;
    let nm = root.join(".build-tools/llvm-mingw/bin/llvm-nm");
    if !nm.is_file() {
        return Err("gen_ctype_tables: object exists but llvm-nm is missing".into());
    }
    let out = Command::new(&nm)
        .arg("--defined-only")
        .arg(&obj)
        .output()
        .map_err(|e| format!("gen_ctype_tables: cannot run {}: {e}", nm.display()))?;
    if !out.status.success() {
        return Err(format!(
            "gen_ctype_tables: llvm-nm failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let mut offsets = BTreeMap::<String, usize>::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let p = line.split_whitespace().collect::<Vec<_>>();
        if p.len() == 3 && p[1] == "D" {
            let off = usize::from_str_radix(p[0], 16)
                .map_err(|e| format!("gen_ctype_tables: bad symbol offset: {e}"))?;
            offsets.insert(p[2].trim_start_matches('_').into(), off);
        }
    }
    let n = *offsets
        .get("MSVCRT__ctype")
        .ok_or("gen_ctype_tables: MSVCRT__ctype is not a DATA symbol of the object")?;
    let w = *offsets
        .get("MSVCRT__wctype")
        .ok_or("gen_ctype_tables: MSVCRT__wctype is not a DATA symbol of the object")?;
    let data = crate::asm::read_coff_section(&raw, ".data")?;
    Ok(Some(tables_from_data(&data, n, w)?))
}
fn tables_from_data(data: &[u8], n: usize, w: usize) -> Result<Tables, String> {
    let read = |off: usize| -> Result<Vec<u16>, String> {
        let end = off
            .checked_add(514)
            .ok_or("gen_ctype_tables: object table offset overflow")?;
        let bytes = data
            .get(off..end)
            .ok_or("gen_ctype_tables: object table lies outside .data")?;
        Ok(bytes
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect())
    };
    Ok((read(n)?, read(w)?))
}
fn read_path_inside(root: &Path, rel: &str) -> Result<Option<PathBuf>, String> {
    let p = root.join(rel);
    if !p.exists() {
        return Ok(None);
    }
    let base = root.canonicalize().map_err(|e| e.to_string())?;
    let real = p
        .canonicalize()
        .map_err(|e| format!("gen_ctype_tables: cannot resolve {}: {e}", p.display()))?;
    if !real.starts_with(base) {
        return Err(format!(
            "gen_ctype_tables: input resolves outside checkout: {}",
            p.display()
        ));
    }
    Ok(Some(real))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn masks() -> Masks {
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
        .map(|(k, v)| (k.into(), v))
        .collect()
    }
    #[test]
    fn expressions_and_gated_values() {
        let got = evaluate(
            &[
                "_S_|_C_".into(),
                "_D_|_H_".into(),
                "0".into(),
                "0x0028 | B110".into(),
                "0x10 | D120".into(),
            ],
            &masks(),
        )
        .unwrap();
        assert_eq!(got, [40, 132, 0, 104, 20]);
    }
    #[test]
    fn malformed_initializer_and_comments() {
        let short = format!("WORD X = {{{}}};", vec!["0"; 256].join(","));
        assert!(initializer(&short, "WORD X")
            .unwrap_err()
            .contains("expected 257"));
        let s = format!(
            "WORD X={{{}}};",
            (0..257)
                .map(|i| format!("/*{i}*/ 0"))
                .collect::<Vec<_>>()
                .join(",")
        );
        assert!(initializer(&s, "WORD X").unwrap().iter().all(|x| x == "0"));
        assert!(initializer("WORD X={ /* no end 0,0", "WORD X").is_err());
    }
    #[test]
    fn bad_values_and_proof() {
        assert!(evaluate(&["0x1ffff".into()], &masks())
            .unwrap_err()
            .contains("does not fit a WORD"));
        let e = vec!["0".into(); 257];
        let z = vec![0; 257];
        let mut bad = z.clone();
        bad[65] = 129;
        assert!(prove("MSVCRT__ctype", &e, &z, &bad)
            .unwrap_err()
            .contains("do NOT ship"));
        assert!(prove("MSVCRT__ctype", &e, &z, &z).is_ok());
    }
    #[test]
    fn proof_rejects_malformed_bounds() {
        let e = vec!["0".into(); 257];
        assert!(prove("x", &e, &[0; 256], &[0; 257])
            .unwrap_err()
            .contains("malformed proof"));
        assert!(tables_from_data(&[0; 513], 0, 0)
            .unwrap_err()
            .contains("outside .data"));
        assert!(tables_from_data(&[0; 1028], usize::MAX, 0)
            .unwrap_err()
            .contains("offset overflow"));
    }
}
#[cfg(test)]
mod parity_test {
    use super::*;
    #[test]
    fn current_tree_block_byte_parity() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let root = base.as_path();
        let generated = generate(root).unwrap();
        let target =
            std::fs::read_to_string(root.join("sdk/lib/rust/msvcrt-crt-rust/src/msvcrt/ctype.rs"))
                .unwrap();
        let have = target
            .split_once(BEGIN)
            .unwrap()
            .1
            .split_once(END)
            .unwrap()
            .0
            .trim();
        assert_eq!(have, generated.block.trim());
    }
}
