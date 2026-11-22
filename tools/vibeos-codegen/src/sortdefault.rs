//! `sortdefault`: Windows-10 sort weights for kernel32's ntsort, from Microsoft's
//! "Windows 10 Sorting Weight Table.txt".
//!
//! That file has no open licence, so it is NEVER committed: `nls_cli` fetches it at
//! generation time into a cache outside the repo and checks it against [`SOURCE_SHA256`].
//! Only the binary this module emits, `media/nls/sortdefault.nls`, is committed. Wine
//! builds its own sortdefault.nls the same way (tools/make_unicode).
//!
//! # Output layout (version [`LAYOUT_VERSION`], all integers little-endian)
//!
//! ```text
//! header   [u32; 17]: MAGIC, LAYOUT_VERSION, nls_version,
//!                     n_guids, guids_off, n_names, names_off,
//!                     n_expansion, expansion_off, n_compat, compat_off,
//!                     n_multi, multi_off, n_jamo, jamo_off, keys_off, total_size
//! keys     [u32; 0x10000] at keys_off: AW | SM<<8 | DW<<16 | CW<<24 (sortkey.nls's byte
//!          order). From the DEFAULT table, plus two kinds the source lists elsewhere:
//!          an EXPANSION code point not in DEFAULT gets SM 2 (expansion) with its position
//!          in the expansion table in the high 16 bits (`SM_EXPANSION_KEY`; 730 entries do
//!          not fit AW), and a COMPATIBILITY code point gets its equivalent's key. Code
//!          points still unlisted (unassigned in the source, e.g. U+0378) are 0, weightless.
//! guids    n_guids x GUID_RECORD bytes: guid[16], flags u32 (FLAG_*),
//!          exc_off u32, exc_n u32, ling_off u32, ling_n u32,
//!          then for compression lengths 2..=8: (off u32, n u32)
//! names    n_names x 24 bytes: ASCII name NUL-padded to 22, guid index u16
//! expansion n x 8: cp u16, first u16, second u16, 0 u16, in SOURCE ORDER: keys index it
//!          by position (the source warns to keep the first 37 positions stable)
//! compat   n x 4: cp u16, equivalent u16
//! multi    n x 4: first SM u8, count u8, 0 u16
//! jamo     n x 8: cp u16, then the five weight bytes, 0 u8
//! exception entry: cp u32, key u32
//! compression entry of length k: k x u16 code points, padded to 4, then key u32
//! ```
//!
//! Offsets are from the start of the file. Every section's declared count in the source
//! is checked against what was parsed, and an unknown section keyword is an error, so a
//! format change upstream fails here instead of shipping a silently partial table.
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const SOURCE_URL: &str = "https://download.microsoft.com/download/C/F/7/CF713A5E-9FBC-4FD6-9246-275F65C0E498/Windows%2010%20Sorting%20Weight%20Table.txt";
pub const SOURCE_SHA256: &str = "81fcfa1e5ed3e3a94d329959ff7d97d522ddf9d653d2c4d6ddcccc5cd4df663f";
pub const MAGIC: u32 = u32::from_le_bytes(*b"SORT");
pub const LAYOUT_VERSION: u32 = 1;
pub const FLAG_REVERSE_DIACRITICS: u32 = 0x1;
pub const FLAG_DOUBLE_COMPRESSION: u32 = 0x2;
pub const FLAG_INVERSE_CASING: u32 = 0x4;
pub const FLAG_THREE_BYTE_WEIGHTS: u32 = 0x8;
/// Ceiling on the generated file, which is committed. 3,803,508 bytes on 2026-11-22
/// (source sha256 above): 256 KiB of keys, the rest per-locale exception/compression
/// tables. A source or layout change that grows it past this must raise the constant
/// here, so the growth is seen in review rather than arriving silently in a binary.
pub const MAX_OUTPUT_BYTES: usize = 4_500_000;
/// The script member of an expansion key; the expansion index is `key >> 16`.
pub const SM_EXPANSION: u32 = 2;
const MAX_COMPRESSION: usize = 8;
const GUID_RECORD: usize = 16 + 4 * 5 + 8 * (MAX_COMPRESSION - 1);
const NAME_RECORD: usize = 24;

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

#[derive(Default)]
struct Guid {
    flags: u32,
    exceptions: Vec<(u32, u32)>,
    linguistic: Vec<(u32, u32)>,
    /// compressions[k - 2] holds the length-k sequences.
    compressions: [Vec<(Vec<u32>, u32)>; MAX_COMPRESSION - 1],
}

#[derive(Default)]
struct Model {
    nls_version: u32,
    keys: BTreeMap<u32, u32>,
    guids: Vec<([u8; 16], Guid)>,
    names: BTreeMap<String, usize>,
    expansion: Vec<(u32, u32, u32)>,
    compat: Vec<(u32, u32)>,
    multi: Vec<(u32, u32)>,
    jamo: Vec<(u32, [u32; 5])>,
}

fn num(tok: &str, line: usize) -> Result<u32, String> {
    let r = match tok.strip_prefix("0x").or_else(|| tok.strip_prefix("0X")) {
        Some(h) => u32::from_str_radix(h, 16),
        None => tok.parse(),
    };
    r.map_err(|e| format!("line {line}: bad number {tok:?}: {e}"))
}

fn key(sm: u32, aw: u32, dw: u32, cw: u32, line: usize) -> Result<u32, String> {
    if [sm, aw, dw, cw].iter().any(|&w| w > 0xff) {
        return Err(format!("line {line}: weight above 255"));
    }
    Ok(aw | sm << 8 | dw << 16 | cw << 24)
}

fn parse_guid(text: &str, line: usize) -> Result<[u8; 16], String> {
    // Microsoft text form, stored in GUID memory layout (Data1-3 little-endian).
    let p: Vec<&str> = text.split('-').collect();
    let ok = p.len() == 5 && [8, 4, 4, 4, 12].iter().zip(&p).all(|(n, s)| s.len() == *n);
    if !ok {
        return Err(format!("line {line}: bad GUID {text:?}"));
    }
    let h = |s: &str| u64::from_str_radix(s, 16).map_err(|e| format!("line {line}: GUID {text:?}: {e}"));
    let mut g = [0u8; 16];
    g[0..4].copy_from_slice(&(h(p[0])? as u32).to_le_bytes());
    g[4..6].copy_from_slice(&(h(p[1])? as u16).to_le_bytes());
    g[6..8].copy_from_slice(&(h(p[2])? as u16).to_le_bytes());
    g[8..10].copy_from_slice(&(h(p[3])? as u16).to_be_bytes());
    g[10..16].copy_from_slice(&h(p[4])?.to_be_bytes()[2..]);
    Ok(g)
}

#[derive(Clone, Copy, PartialEq)]
enum Section {
    Top,
    Default,
    Tables,
    GuidFlag(u32),
    Multi(u32),
    Expansion(u32),
    Compat(u32),
    Compression(u32),
    Exception(u32),
    Jamo(u32),
}

fn parse(text: &str) -> Result<Model, String> {
    let mut m = Model::default();
    let mut sec = Section::Top;
    let mut cur: Option<usize> = None; // current GUID index
    let mut ling = false; // current EXCEPTION table is the LINGUISTIC_CASING one
    let mut clen = 0usize; // current compression length
    let mut cdeclared: Option<(usize, u32)> = None; // (start len, declared n)
    let mut seen_guids_in = 0u32;
    let mut exc_declared: Option<u32> = None;
    let (mut seen_default, mut seen_tables) = (false, false);

    let close_count = |sec: Section, n: usize, what: &str| -> Result<(), String> {
        let declared = match sec {
            Section::Multi(d) | Section::Expansion(d) | Section::Compat(d) | Section::Jamo(d) => d,
            _ => return Ok(()),
        };
        if declared as usize != n {
            return Err(format!("{what}: declared {declared}, parsed {n}"));
        }
        Ok(())
    };

    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    for (i, raw) in text.lines().enumerate() {
        let line = i + 1;
        let body = raw.split(';').next().unwrap_or("").trim();
        if body.is_empty() {
            continue;
        }
        let toks: Vec<&str> = body.split_whitespace().collect();
        let kw = toks[0];
        let data = kw.starts_with("0x") || kw.starts_with("0X") || kw.chars().all(|c| c.is_ascii_digit());

        if !data && kw != "LOCALENAME" {
            // A keyword other than LOCALENAME ends the current counted table: check what it
            // collected. (LOCALENAME lines sit between a SORTGUID and its rows.)
            let n = match sec {
                Section::Multi(_) => m.multi.len(),
                Section::Expansion(_) => m.expansion.len(),
                Section::Compat(_) => m.compat.len(),
                Section::Jamo(_) => m.jamo.len(),
                _ => 0,
            };
            close_count(sec, n, &format!("section ending at line {line}"))?;
            if let Some((start, declared)) = cdeclared {
                let g = &m.guids[cur.ok_or("compression count without SORTGUID")?].1;
                let got = g.compressions[clen - 2].len() - start;
                if got != declared as usize {
                    return Err(format!("line {line}: compression of length {clen} declared {declared}, parsed {got}"));
                }
                cdeclared = None;
            }
            if let Some(d) = exc_declared.take() {
                let g = &m.guids[cur.ok_or("EXCEPTION count without SORTGUID")?].1;
                let got = if ling { g.linguistic.len() } else { g.exceptions.len() };
                if got != d as usize {
                    return Err(format!("line {line}: exception table declared {d}, parsed {got}"));
                }
            }
            // Leaving a COMPRESSION/EXCEPTION section: its header counts SORTGUIDs.
            let inner = matches!(kw, "SORTGUID" | "TWO" | "THREE" | "FOUR" | "FIVE" | "SIX" | "SEVEN" | "EIGHT");
            if !inner {
                if let Section::Compression(d) | Section::Exception(d) = sec {
                    if d != seen_guids_in {
                        return Err(format!("line {line}: section declared {d} SORTGUIDs, parsed {seen_guids_in}"));
                    }
                }
            }
        }

        match (kw, sec) {
            ("SORTKEY", Section::Top) => {}
            ("DEFAULT", Section::Top) => {
                sec = Section::Default;
                seen_default = true;
            }
            ("ENDSORTKEY", Section::Default) => sec = Section::Top,
            ("SORTTABLES", Section::Top) => {
                sec = Section::Tables;
                seen_tables = true;
            }
            ("ENDSORTTABLES", _) => sec = Section::Top,
            ("RELEASE", _) => sec = Section::Tables,
            ("NLSVERSION", _) => {
                m.nls_version = num(toks.get(1).ok_or(format!("line {line}: NLSVERSION value"))?, line)?;
                sec = Section::Tables;
            }
            ("REVERSEDIACRITICS", _) => sec = Section::GuidFlag(FLAG_REVERSE_DIACRITICS),
            ("DOUBLECOMPRESSION", _) => sec = Section::GuidFlag(FLAG_DOUBLE_COMPRESSION),
            ("INVERSECASING", _) => sec = Section::GuidFlag(FLAG_INVERSE_CASING),
            ("MULTIPLEWEIGHTS", _) => sec = Section::Multi(num(toks.get(1).ok_or(format!("line {line}: count"))?, line)?),
            ("EXPANSION", _) => sec = Section::Expansion(num(toks.get(1).ok_or(format!("line {line}: count"))?, line)?),
            ("COMPATIBILITY", _) => sec = Section::Compat(num(toks.get(1).ok_or(format!("line {line}: count"))?, line)?),
            ("COMPRESSION", _) => {
                sec = Section::Compression(num(toks.get(1).ok_or(format!("line {line}: count"))?, line)?);
                seen_guids_in = 0;
            }
            ("EXCEPTION", _) => {
                sec = Section::Exception(num(toks.get(1).ok_or(format!("line {line}: count"))?, line)?);
                seen_guids_in = 0;
            }
            ("JAMOSORT", _) => sec = Section::Jamo(num(toks.get(1).ok_or(format!("line {line}: count"))?, line)?),
            ("SORTGUID", Section::GuidFlag(_) | Section::Compression(_) | Section::Exception(_)) => {
                let g = parse_guid(toks.get(1).ok_or(format!("line {line}: SORTGUID value"))?, line)?;
                let idx = match m.guids.iter().position(|(x, _)| *x == g) {
                    Some(i) => i,
                    None => {
                        m.guids.push((g, Guid::default()));
                        m.guids.len() - 1
                    }
                };
                cur = Some(idx);
                seen_guids_in += 1;
                let mut rest = toks[2..].iter();
                ling = false;
                match sec {
                    Section::GuidFlag(f) => m.guids[idx].1.flags |= f,
                    Section::Exception(_) => {
                        exc_declared = Some(match rest.next() {
                            Some(t) if t.chars().all(|c| c.is_ascii_digit()) => num(t, line)?,
                            Some(_) | None => 0,
                        });
                        for t in toks[2..].iter() {
                            match *t {
                                "HAS_3_BYTE_WEIGHTS" => m.guids[idx].1.flags |= FLAG_THREE_BYTE_WEIGHTS,
                                "LINGUISTIC_CASING" => ling = true,
                                t if t.chars().all(|c| c.is_ascii_digit()) => {}
                                t => return Err(format!("line {line}: unknown SORTGUID flag {t:?}")),
                            }
                        }
                        if !ling && !m.guids[idx].1.exceptions.is_empty() {
                            return Err(format!("line {line}: second non-linguistic EXCEPTION table for one GUID"));
                        }
                    }
                    _ => {
                        for t in toks[2..].iter() {
                            match *t {
                                "HAS_3_BYTE_WEIGHTS" => m.guids[idx].1.flags |= FLAG_THREE_BYTE_WEIGHTS,
                                t if t.chars().all(|c| c.is_ascii_digit()) => {}
                                t => return Err(format!("line {line}: unknown SORTGUID flag {t:?}")),
                            }
                        }
                    }
                }
            }
            ("LOCALENAME", Section::GuidFlag(_) | Section::Compression(_) | Section::Exception(_)) => {
                let name = toks.get(1).ok_or(format!("line {line}: LOCALENAME value"))?.to_string();
                let idx = cur.ok_or(format!("line {line}: LOCALENAME before SORTGUID"))?;
                if name.len() > NAME_RECORD - 3 || !name.is_ascii() {
                    return Err(format!("line {line}: locale name {name:?} does not fit the name record"));
                }
                if let Some(&prev) = m.names.get(&name) {
                    if prev != idx {
                        return Err(format!("line {line}: {name} mapped to two sort GUIDs"));
                    }
                }
                m.names.insert(name, idx);
            }
            ("TWO" | "THREE" | "FOUR" | "FIVE" | "SIX" | "SEVEN" | "EIGHT", Section::Compression(_)) => {
                clen = match kw {
                    "TWO" => 2,
                    "THREE" => 3,
                    "FOUR" => 4,
                    "FIVE" => 5,
                    "SIX" => 6,
                    "SEVEN" => 7,
                    _ => 8,
                };
                let idx = cur.ok_or(format!("line {line}: {kw} before SORTGUID"))?;
                let start = m.guids[idx].1.compressions[clen - 2].len();
                cdeclared = Some((start, num(toks.get(1).ok_or(format!("line {line}: {kw} count"))?, line)?));
            }
            (_, Section::Default) if data => {
                let v: Vec<u32> = toks.iter().map(|t| num(t, line)).collect::<Result<_, _>>()?;
                if v.len() != 5 || v[0] > 0xffff {
                    return Err(format!("line {line}: DEFAULT row wants a BMP code point and 4 weights"));
                }
                if m.keys.insert(v[0], key(v[1], v[2], v[3], v[4], line)?).is_some() {
                    return Err(format!("line {line}: U+{:04X} listed twice", v[0]));
                }
            }
            (_, Section::Multi(_)) if data => {
                let v: Vec<u32> = toks.iter().map(|t| num(t, line)).collect::<Result<_, _>>()?;
                if v.len() != 2 {
                    return Err(format!("line {line}: MULTIPLEWEIGHTS row wants 2 numbers"));
                }
                m.multi.push((v[0], v[1]));
            }
            (_, Section::Expansion(_)) if data => {
                let v: Vec<u32> = toks.iter().map(|t| num(t, line)).collect::<Result<_, _>>()?;
                if v.len() != 3 {
                    return Err(format!("line {line}: EXPANSION row wants 3 code points"));
                }
                m.expansion.push((v[0], v[1], v[2]));
            }
            (_, Section::Compat(_)) if data => {
                let v: Vec<u32> = toks.iter().map(|t| num(t, line)).collect::<Result<_, _>>()?;
                if v.len() != 2 {
                    return Err(format!("line {line}: COMPATIBILITY row wants 2 code points"));
                }
                m.compat.push((v[0], v[1]));
            }
            (_, Section::Jamo(_)) if data => {
                let v: Vec<u32> = toks.iter().map(|t| num(t, line)).collect::<Result<_, _>>()?;
                if v.len() != 6 || v[1..].iter().any(|&b| b > 0xff) {
                    return Err(format!("line {line}: JAMOSORT row wants a code point and 5 bytes"));
                }
                m.jamo.push((v[0], [v[1], v[2], v[3], v[4], v[5]]));
            }
            (_, Section::Compression(_)) if data => {
                let v: Vec<u32> = toks.iter().map(|t| num(t, line)).collect::<Result<_, _>>()?;
                if clen == 0 || v.len() != clen + 4 {
                    return Err(format!("line {line}: compression row wants {clen} code points and 4 weights"));
                }
                let k = key(v[clen], v[clen + 1], v[clen + 2], v[clen + 3], line)?;
                let idx = cur.ok_or(format!("line {line}: compression row before SORTGUID"))?;
                m.guids[idx].1.compressions[clen - 2].push((v[..clen].to_vec(), k));
            }
            (_, Section::Exception(_)) if data => {
                let v: Vec<u32> = toks.iter().map(|t| num(t, line)).collect::<Result<_, _>>()?;
                if v.len() != 5 {
                    return Err(format!("line {line}: EXCEPTION row wants a code point and 4 weights"));
                }
                let k = key(v[1], v[2], v[3], v[4], line)?;
                let idx = cur.ok_or(format!("line {line}: exception row before SORTGUID"))?;
                let g = &mut m.guids[idx].1;
                if ling { g.linguistic.push((v[0], k)) } else { g.exceptions.push((v[0], k)) }
            }
            _ => return Err(format!("line {line}: unexpected {kw:?} here")),
        }
    }
    if !seen_default || !seen_tables || sec != Section::Top {
        return Err("input is truncated: SORTKEY DEFAULT and SORTTABLES must both be present and closed".into());
    }
    Ok(m)
}

fn put32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn put16(out: &mut Vec<u8>, v: u32) -> Result<(), String> {
    let v: u16 = v.try_into().map_err(|_| format!("value {v:#x} does not fit 16 bits"))?;
    out.extend_from_slice(&v.to_le_bytes());
    Ok(())
}

fn emit(m: &Model) -> Result<Vec<u8>, String> {
    const HEADER: usize = 17 * 4;
    let mut body: Vec<u8> = Vec::new();
    let at = |body: &Vec<u8>| (HEADER + body.len()) as u32;

    // DEFAULT, then synthesized expansion and compatibility keys (see the layout notes).
    let mut keys = m.keys.clone();
    for (i, (cp, _, _)) in m.expansion.iter().enumerate() {
        keys.entry(*cp).or_insert(SM_EXPANSION << 8 | (i as u32) << 16);
    }
    for (cp, target) in &m.compat {
        let k = *keys.get(target).ok_or(format!("COMPATIBILITY U+{cp:04X} -> U+{target:04X}, which has no key"))?;
        if keys.insert(*cp, k).is_some() {
            return Err(format!("COMPATIBILITY U+{cp:04X} already has a key"));
        }
    }
    let keys_off = at(&body);
    for cp in 0..0x10000u32 {
        put32(&mut body, keys.get(&cp).copied().unwrap_or(0));
    }
    // Per-GUID entry tables first, so the GUID records can point at them.
    let mut slices = Vec::new();
    for (_, g) in &m.guids {
        let exc = |list: &Vec<(u32, u32)>, body: &mut Vec<u8>| {
            let mut sorted = list.clone();
            sorted.sort();
            let off = at(body);
            for (cp, k) in &sorted {
                put32(body, *cp);
                put32(body, *k);
            }
            (off, sorted.len() as u32)
        };
        let e = exc(&g.exceptions, &mut body);
        let l = exc(&g.linguistic, &mut body);
        let mut comps = Vec::new();
        for (i, list) in g.compressions.iter().enumerate() {
            let k = i + 2;
            let mut sorted = list.clone();
            sorted.sort();
            let off = at(&body);
            for (cps, key) in &sorted {
                for &c in cps {
                    put16(&mut body, c)?;
                }
                if k % 2 == 1 {
                    body.extend_from_slice(&[0, 0]);
                }
                put32(&mut body, *key);
            }
            comps.push((off, sorted.len() as u32));
        }
        slices.push((e, l, comps));
    }
    let guids_off = at(&body);
    for ((g, guid), (e, l, comps)) in m.guids.iter().zip(&slices) {
        body.extend_from_slice(g);
        put32(&mut body, guid.flags);
        for v in [e.0, e.1, l.0, l.1] {
            put32(&mut body, v);
        }
        for (off, n) in comps {
            put32(&mut body, *off);
            put32(&mut body, *n);
        }
    }
    let names_off = at(&body);
    for (name, &idx) in &m.names {
        let mut rec = [0u8; NAME_RECORD];
        rec[..name.len()].copy_from_slice(name.as_bytes());
        rec[NAME_RECORD - 2..].copy_from_slice(&(idx as u16).to_le_bytes());
        body.extend_from_slice(&rec);
    }
    let expansion_off = at(&body);
    let ex = &m.expansion; // source order: keys index it by position
    for (c, a, b) in ex {
        put16(&mut body, *c)?;
        put16(&mut body, *a)?;
        put16(&mut body, *b)?;
        put16(&mut body, 0)?;
    }
    let compat_off = at(&body);
    let mut co = m.compat.clone();
    co.sort();
    for (c, e) in &co {
        put16(&mut body, *c)?;
        put16(&mut body, *e)?;
    }
    let multi_off = at(&body);
    for (sm, n) in &m.multi {
        if *sm > 0xff || *n > 0xff {
            return Err("MULTIPLEWEIGHTS value above 255".into());
        }
        body.extend_from_slice(&[*sm as u8, *n as u8, 0, 0]);
    }
    let jamo_off = at(&body);
    let mut jm = m.jamo.clone();
    jm.sort();
    for (c, b) in &jm {
        put16(&mut body, *c)?;
        body.extend_from_slice(&[b[0] as u8, b[1] as u8, b[2] as u8, b[3] as u8, b[4] as u8, 0]);
    }
    let total = (HEADER + body.len()) as u32;

    let mut out = Vec::with_capacity(total as usize);
    for v in [
        MAGIC,
        LAYOUT_VERSION,
        m.nls_version,
        m.guids.len() as u32,
        guids_off,
        m.names.len() as u32,
        names_off,
        ex.len() as u32,
        expansion_off,
        co.len() as u32,
        compat_off,
        m.multi.len() as u32,
        multi_off,
        jm.len() as u32,
        jamo_off,
        keys_off,
        total,
    ] {
        put32(&mut out, v);
    }
    out.extend_from_slice(&body);
    debug_assert_eq!(out.len(), total as usize);
    let _ = GUID_RECORD;
    Ok(out)
}

/// Parse the sorting-weight table text and emit sortdefault.nls.
pub fn sortdefault(text: &str) -> Result<Vec<u8>, String> {
    let out = emit(&parse(text)?)?;
    if out.len() > MAX_OUTPUT_BYTES {
        return Err(format!(
            "sortdefault.nls would be {} bytes, over MAX_OUTPUT_BYTES ({MAX_OUTPUT_BYTES}); if \
             the growth is intended, raise the constant in tools/vibeos-codegen/src/sortdefault.rs",
            out.len()
        ));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = "\u{feff}; header comment\r
SORTKEY\r
\tDEFAULT\r
0x0000\t0\t0\t0\t0\t;<control>\r
0x003b\t6\t26\t2\t2\t;semicolon\r
0x0041\t14\t2\t2\t18\t;A\r
0x0061\t14\t2\t2\t2\t;a\r
ENDSORTKEY\r
SORTTABLES\r
  RELEASE\r
\tNLSVERSION    0x00060305 ; version\r
  REVERSEDIACRITICS\r
\tSORTGUID\t00000003-57EE-1E5C-00B4-D0000BB1E11E\r
\tLOCALENAME\tfr\t;French\r
  MULTIPLEWEIGHTS  1\r
192\t49\r
\tEXPANSION\t1\r
0x00c6\t0x0041\t0x0045\t;AE\r
\tCOMPATIBILITY\t1\r
0x037e\t0x003b\r
\tCOMPRESSION\t1\r
\tSORTGUID\t00000007-57EE-1E5C-00B4-D0000BB1E11E\r
\tLOCALENAME\tsd\r
\tTWO\t1\r
0x0063\t0x0068\t14\t30\t2\t2\t;ch\r
EXCEPTION\t1\r
\tSORTGUID\t00000002-57EE-1E5C-00B4-D0000BB1E11E\t1\r
\tLOCALENAME\thaw\r
0x0304\t1\t0\t21\t0\r
    JAMOSORT    1\r
0x1100      0x00    0x00    0x00    0x00    0x03\r
ENDSORTTABLES\r
";

    fn u32_at(b: &[u8], off: usize) -> u32 {
        u32::from_le_bytes(b[off..off + 4].try_into().unwrap())
    }

    #[test]
    fn emits_the_documented_layout() {
        let b = sortdefault(FIXTURE).unwrap();
        assert_eq!(u32_at(&b, 0), MAGIC);
        assert_eq!(u32_at(&b, 4), LAYOUT_VERSION);
        assert_eq!(u32_at(&b, 8), 0x00060305);
        assert_eq!(u32_at(&b, 12), 3, "three distinct sort GUIDs");
        let keys = u32_at(&b, 15 * 4) as usize;
        assert_eq!(u32_at(&b, keys + 0x41 * 4), 2 | 14 << 8 | 2 << 16 | 18 << 24);
        assert_eq!(u32_at(&b, keys + 0x42 * 4), 0, "unlisted code points are weightless");
        assert_eq!(u32_at(&b, 16 * 4) as usize, b.len());
        // The French GUID carries the reverse-diacritics flag.
        let guids = u32_at(&b, 16) as usize;
        assert_eq!(&b[guids..guids + 4], &3u32.to_le_bytes());
        assert_eq!(u32_at(&b, guids + 16) & FLAG_REVERSE_DIACRITICS, FLAG_REVERSE_DIACRITICS);
        // Expansion: synthesized SM 2 key indexing the expansion table (entry 0).
        assert_eq!(u32_at(&b, keys + 0xc6 * 4), SM_EXPANSION << 8);
        // Compatibility: U+037E takes U+003B's key.
        assert_eq!(u32_at(&b, keys + 0x37e * 4), 26 | 6 << 8 | 2 << 16 | 2 << 24);
    }

    #[test]
    fn refuses_count_mismatch_unknown_keywords_and_truncation() {
        let bad_count = FIXTURE.replace("EXPANSION\t1", "EXPANSION\t2");
        assert!(sortdefault(&bad_count).unwrap_err().contains("declared 2, parsed 1"));
        let unknown = FIXTURE.replace("  RELEASE", "  SOMETHINGNEW");
        assert!(sortdefault(&unknown).unwrap_err().contains("SOMETHINGNEW"));
        let truncated = &FIXTURE[..FIXTURE.find("ENDSORTTABLES").unwrap()];
        assert!(sortdefault(truncated).unwrap_err().contains("truncated"));
    }

    #[test]
    fn output_is_deterministic() {
        assert_eq!(sortdefault(FIXTURE).unwrap(), sortdefault(FIXTURE).unwrap());
    }
}
