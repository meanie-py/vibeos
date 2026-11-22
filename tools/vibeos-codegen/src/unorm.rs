//! `unorm`: Unicode normalization tables for `sdk/lib/rust/unorm-rust` from the vendored
//! UCD (`vendor/unicode-ucd/<version>/`, registered in vendor.yaml).
//!
//! Emits, all sorted by code point for binary search:
//! * `CCC`: every code point with a nonzero canonical combining class.
//! * `CANON` / `COMPAT`: each code point's FULL decomposition (recursively expanded) for
//!   NFD and NFKD respectively, as (code point, offset, length) into `DECOMP_DATA`. Hangul
//!   syllables are algorithmic and omitted. A code point absent from `COMPAT` but present
//!   in `CANON` decomposes the same under both.
//! * `COMPOSE`: (first, second, composite) for every primary composite, i.e. every
//!   two-code-point canonical decomposition that is not Full_Composition_Exclusion. That
//!   property is derived here, not read: the explicit CompositionExclusions.txt list, plus
//!   singletons (one-code-point canonical decompositions), plus non-starter decompositions
//!   (the code point or the first of its decomposition has a nonzero combining class).
//!   UAX #15 defines Full_Composition_Exclusion as exactly that union.
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

pub const UCD_VERSION: &str = "15.0.0";

struct Entry {
    ccc: u8,
    /// (is_compat, mapping)
    decomp: Option<(bool, Vec<u32>)>,
}

fn hex(s: &str) -> Result<u32, String> {
    u32::from_str_radix(s.trim(), 16).map_err(|e| format!("bad code point {s:?}: {e}"))
}

fn parse_unicode_data(text: &str) -> Result<BTreeMap<u32, Entry>, String> {
    let mut out = BTreeMap::new();
    for (n, line) in text.lines().enumerate() {
        if line.is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split(';').collect();
        if f.len() < 15 {
            return Err(format!("UnicodeData.txt:{}: {} fields, want 15", n + 1, f.len()));
        }
        let cp = hex(f[0])?;
        let ccc: u8 = f[3]
            .parse()
            .map_err(|e| format!("UnicodeData.txt:{}: ccc {:?}: {e}", n + 1, f[3]))?;
        let decomp = if f[5].is_empty() {
            None
        } else {
            let (compat, rest) = match f[5].strip_prefix('<') {
                Some(r) => (true, r.split_once('>').ok_or("unterminated <tag>")?.1),
                None => (false, f[5]),
            };
            let cps: Vec<u32> = rest.split_whitespace().map(hex).collect::<Result<_, _>>()?;
            if cps.is_empty() {
                return Err(format!("UnicodeData.txt:{}: empty decomposition", n + 1));
            }
            Some((compat, cps))
        };
        out.insert(cp, Entry { ccc, decomp });
    }
    if out.len() < 30000 {
        return Err(format!("UnicodeData.txt has only {} entries: truncated input?", out.len()));
    }
    Ok(out)
}

fn parse_exclusions(text: &str) -> Result<BTreeSet<u32>, String> {
    let mut out = BTreeSet::new();
    for line in text.lines() {
        let data = line.split('#').next().unwrap_or("").trim();
        if data.is_empty() {
            continue;
        }
        if data.contains("..") {
            return Err(format!("CompositionExclusions.txt: unexpected range {data:?}"));
        }
        out.insert(hex(data)?);
    }
    if out.is_empty() {
        return Err("CompositionExclusions.txt lists no code points".into());
    }
    Ok(out)
}

fn expand(db: &BTreeMap<u32, Entry>, cp: u32, compat: bool, out: &mut Vec<u32>, depth: u32) -> Result<(), String> {
    if depth > 32 {
        return Err(format!("decomposition of U+{cp:04X} does not terminate"));
    }
    match db.get(&cp).and_then(|e| e.decomp.as_ref()) {
        Some((is_compat, m)) if compat || !is_compat => {
            for &c in m {
                expand(db, c, compat, out, depth + 1)?;
            }
        }
        _ => out.push(cp),
    }
    Ok(())
}

fn ccc(db: &BTreeMap<u32, Entry>, cp: u32) -> u8 {
    db.get(&cp).map_or(0, |e| e.ccc)
}

pub fn unorm_tables(unicode_data: &str, exclusions: &str) -> Result<String, String> {
    let db = parse_unicode_data(unicode_data)?;
    let explicit = parse_exclusions(exclusions)?;

    let mut data: Vec<u32> = Vec::new();
    let mut canon: Vec<(u32, usize, usize)> = Vec::new();
    let mut compat: Vec<(u32, usize, usize)> = Vec::new();
    let mut compose: Vec<(u32, u32, u32)> = Vec::new();
    for (&cp, e) in &db {
        let Some((is_compat, m)) = &e.decomp else { continue };
        let mut full_c = Vec::new();
        expand(&db, cp, false, &mut full_c, 0)?;
        let mut full_k = Vec::new();
        expand(&db, cp, true, &mut full_k, 0)?;
        if full_c != [cp] {
            canon.push((cp, data.len(), full_c.len()));
            data.extend(&full_c);
        }
        if full_k != full_c {
            compat.push((cp, data.len(), full_k.len()));
            data.extend(&full_k);
        }
        if !is_compat && m.len() == 2 {
            let excluded = explicit.contains(&cp) || e.ccc != 0 || ccc(&db, m[0]) != 0;
            if !excluded {
                compose.push((m[0], m[1], cp));
            }
        }
    }
    compose.sort();
    let ccc_list: Vec<(u32, u8)> = db.iter().filter(|(_, e)| e.ccc != 0).map(|(&c, e)| (c, e.ccc)).collect();
    if data.len() > u16::MAX as usize {
        return Err(format!("decomposition data has {} units; offsets are u16", data.len()));
    }

    let mut o = String::new();
    writeln!(o, "// @generated by `tools/codegen unorm` from vendor/unicode-ucd/{UCD_VERSION}/ (UnicodeData.txt,").unwrap();
    writeln!(o, "// CompositionExclusions.txt). Do not edit: regenerate. Unicode data (C) Unicode, Inc.,").unwrap();
    writeln!(o, "// Unicode License v3: see vendor/unicode-ucd/LICENSE.txt.").unwrap();
    writeln!(o).unwrap();
    writeln!(o, "/// The UCD version these tables were generated from.").unwrap();
    writeln!(o, "pub const UCD_VERSION: &str = \"{UCD_VERSION}\";").unwrap();
    writeln!(o).unwrap();
    writeln!(o, "/// (code point, canonical combining class) for every nonzero class.").unwrap();
    writeln!(o, "pub static CCC: [(u32, u8); {}] = [", ccc_list.len()).unwrap();
    for (c, k) in &ccc_list {
        writeln!(o, "    (0x{c:04X}, {k}),").unwrap();
    }
    writeln!(o, "];").unwrap();
    for (name, list, doc) in [
        ("CANON", &canon, "Full canonical (NFD) decompositions: (code point, offset, length) into DECOMP_DATA."),
        ("COMPAT", &compat, "Full compatibility (NFKD) decompositions where they differ from NFD."),
    ] {
        writeln!(o).unwrap();
        writeln!(o, "/// {doc}").unwrap();
        writeln!(o, "pub static {name}: [(u32, u16, u8); {}] = [", list.len()).unwrap();
        for (c, off, len) in list.iter() {
            writeln!(o, "    (0x{c:04X}, {off}, {len}),").unwrap();
        }
        writeln!(o, "];").unwrap();
    }
    writeln!(o).unwrap();
    writeln!(o, "pub static DECOMP_DATA: [u32; {}] = [", data.len()).unwrap();
    for chunk in data.chunks(8) {
        let line: Vec<String> = chunk.iter().map(|c| format!("0x{c:04X}")).collect();
        writeln!(o, "    {},", line.join(", ")).unwrap();
    }
    writeln!(o, "];").unwrap();
    writeln!(o).unwrap();
    writeln!(o, "/// (first, second, primary composite), sorted by (first, second).").unwrap();
    writeln!(o, "pub static COMPOSE: [(u32, u32, u32); {}] = [", compose.len()).unwrap();
    for (a, b, c) in &compose {
        writeln!(o, "    (0x{a:04X}, 0x{b:04X}, 0x{c:04X}),").unwrap();
    }
    writeln!(o, "];").unwrap();
    Ok(o)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATA: &str = "\
0041;LATIN CAPITAL LETTER A;Lu;0;L;;;;;N;;;;0061;
0300;COMBINING GRAVE ACCENT;Mn;230;NSM;;;;;N;NON-SPACING GRAVE;;;;
0340;COMBINING GRAVE TONE MARK;Mn;230;NSM;0300;;;;N;NON-SPACING GRAVE TONE MARK;;;;
00C0;LATIN CAPITAL LETTER A WITH GRAVE;Lu;0;L;0041 0300;;;;N;LATIN CAPITAL LETTER A GRAVE;;;00E0;
FB01;LATIN SMALL LIGATURE FI;Ll;0;L;<compat> 0066 0069;;;;N;;;;;
";

    fn big(extra: &str) -> String {
        // The parser refuses a truncated UnicodeData; pad with plain letters.
        let mut s = String::from(extra);
        for cp in 0x10000u32..0x10000 + 30000 {
            s.push_str(&format!("{cp:04X};X;Lo;0;L;;;;;N;;;;;\n"));
        }
        s
    }

    #[test]
    fn composes_primary_composites_and_derives_exclusions() {
        let out = unorm_tables(&big(DATA), "0958 # excluded\n").unwrap();
        assert!(out.contains("(0x0041, 0x0300, 0x00C0)"), "{out}");
        // U+0340 is a singleton: decomposes, never composes.
        assert!(out.contains("(0x0340, "));
        assert!(!out.contains("0x0340),"));
        // The ligature decomposes only under compatibility.
        assert!(out.contains("pub static COMPAT: [(u32, u16, u8); 1]"));
        assert!(out.contains("(0x0300, 230)"));
    }

    #[test]
    fn refuses_truncated_input() {
        assert!(unorm_tables(DATA, "0958\n").unwrap_err().contains("truncated"));
    }
}
