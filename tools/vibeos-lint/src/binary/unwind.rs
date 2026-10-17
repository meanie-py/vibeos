//! Check ARM64 unwind save codes against stores in the actual function prologue.
use crate::binary::{BinaryContext, CheckOutput};
use regex::Regex;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

const DEFAULT_TARGETS: &[&str] = &["ntoskrnl/ntoskrnl.exe", "dll/ntdll/ntdll.dll"];

fn rx(pattern: &str) -> &'static Regex {
    static MAP: OnceLock<std::sync::Mutex<BTreeMap<String, &'static Regex>>> = OnceLock::new();
    let mut map = MAP
        .get_or_init(|| std::sync::Mutex::new(BTreeMap::new()))
        .lock()
        .unwrap();
    *map.entry(pattern.to_owned())
        .or_insert_with(|| Box::leak(Box::new(Regex::new(pattern).unwrap())))
}

fn bytes(data: &[u8], offset: usize, count: usize) -> Result<&[u8], String> {
    data.get(offset..offset.checked_add(count).ok_or("offset overflow")?)
        .ok_or_else(|| format!("truncated PE data at offset 0x{offset:x}"))
}
fn u16le(data: &[u8], offset: usize) -> Result<u16, String> {
    Ok(u16::from_le_bytes(
        bytes(data, offset, 2)?.try_into().unwrap(),
    ))
}
fn u32le(data: &[u8], offset: usize) -> Result<u32, String> {
    Ok(u32::from_le_bytes(
        bytes(data, offset, 4)?.try_into().unwrap(),
    ))
}
fn u64le(data: &[u8], offset: usize) -> Result<u64, String> {
    Ok(u64::from_le_bytes(
        bytes(data, offset, 8)?.try_into().unwrap(),
    ))
}

#[derive(Debug)]
struct Pe {
    data: Vec<u8>,
    image_base: u64,
    pdata_rva: u32,
    pdata_size: u32,
    sections: Vec<Section>,
}
#[derive(Clone, Copy, Debug)]
struct Section {
    vaddr: u32,
    vsize: u32,
    rawptr: u32,
    rawsize: u32,
}
fn parse_pe(path: &Path) -> Result<Option<Pe>, String> {
    let data = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if data.get(..2) != Some(b"MZ") {
        return Ok(None);
    }
    let pe_off = u32le(&data, 0x3c)? as usize;
    if bytes(&data, pe_off, 4)? != b"PE\0\0" {
        return Ok(None);
    }
    let fh = pe_off.checked_add(4).ok_or("PE header offset overflow")?;
    let machine = u16le(&data, fh)?;
    let nsec = u16le(&data, fh + 2)? as usize;
    let size_opt = u16le(&data, fh + 16)? as usize;
    if machine != 0xaa64 {
        return Ok(None);
    }
    let opt = fh
        .checked_add(20)
        .ok_or("optional header offset overflow")?;
    if u16le(&data, opt)? != 0x20b {
        return Ok(None);
    }
    let image_base = u64le(&data, opt + 24)?;
    let dd = opt + 112;
    let pdata_rva = u32le(&data, dd + 3 * 8)?;
    let pdata_size = u32le(&data, dd + 3 * 8 + 4)?;
    let sh = opt
        .checked_add(size_opt)
        .ok_or("section table offset overflow")?;
    let mut sections = Vec::with_capacity(nsec);
    for i in 0..nsec {
        let off = sh
            .checked_add(i.checked_mul(40).ok_or("section offset overflow")?)
            .ok_or("section offset overflow")?;
        sections.push(Section {
            vsize: u32le(&data, off + 8)?,
            vaddr: u32le(&data, off + 12)?,
            rawsize: u32le(&data, off + 16)?,
            rawptr: u32le(&data, off + 20)?,
        });
    }
    Ok(Some(Pe {
        data,
        image_base,
        pdata_rva,
        pdata_size,
        sections,
    }))
}
fn rva2off(sections: &[Section], rva: u32) -> Option<usize> {
    sections.iter().find_map(|s| {
        let extent = s.vsize.max(s.rawsize);
        let end = s.vaddr as u64 + extent as u64;
        if s.vaddr <= rva && (rva as u64) < end {
            Some(s.rawptr as usize + (rva - s.vaddr) as usize)
        } else {
            None
        }
    })
}

fn code_len(b0: u8) -> usize {
    if b0 <= 0xbf {
        1
    } else if b0 <= 0xdf {
        2
    } else if b0 == 0xe0 {
        4
    } else if b0 == 0xe2 {
        2
    } else if b0 == 0xe7 {
        3
    } else {
        1
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Pair {
    first: u32,
    offset: u32,
    is_fp: bool,
}
fn decode_pair(codes: &[u8], j: usize) -> Option<Pair> {
    let b0 = *codes.get(j)?;
    let b1 = codes.get(j + 1).copied().unwrap_or(0);
    let p = if (0x20..=0x3f).contains(&b0) {
        Pair {
            first: 19,
            offset: 0,
            is_fp: false,
        }
    } else if (0x40..=0x7f).contains(&b0) {
        Pair {
            first: 29,
            offset: ((b0 & 0x3f) as u32) * 8,
            is_fp: false,
        }
    } else if (0x80..=0xbf).contains(&b0) {
        Pair {
            first: 29,
            offset: 0,
            is_fp: false,
        }
    } else if (0xc8..=0xcb).contains(&b0) {
        Pair {
            first: 19 + ((((b0 & 3) as u32) << 2) | (b1 >> 6) as u32),
            offset: (b1 as u32 & 0x3f) * 8,
            is_fp: false,
        }
    } else if (0xcc..=0xcf).contains(&b0) {
        Pair {
            first: 19 + ((((b0 & 3) as u32) << 2) | (b1 >> 6) as u32),
            offset: 0,
            is_fp: false,
        }
    } else if (0xd8..=0xd9).contains(&b0) {
        Pair {
            first: 8 + ((((b0 & 1) as u32) << 2) | (b1 >> 6) as u32),
            offset: (b1 as u32 & 0x3f) * 8,
            is_fp: true,
        }
    } else if (0xda..=0xdb).contains(&b0) {
        Pair {
            first: 8 + ((((b0 & 1) as u32) << 2) | (b1 >> 6) as u32),
            offset: 0,
            is_fp: true,
        }
    } else {
        return None;
    };
    Some(p)
}
fn unwind_stream(pe: &Pe, unwind_rva: u32) -> Result<Option<(u32, Vec<u8>)>, String> {
    let Some(off) = rva2off(&pe.sections, unwind_rva) else {
        return Ok(None);
    };
    let hdr = u32le(&pe.data, off)?;
    let func_len = (hdr & 0x3ffff) * 4;
    let e_bit = (hdr >> 21) & 1;
    let mut epilog_count = ((hdr >> 22) & 0x1f) as usize;
    let mut code_words = ((hdr >> 27) & 0x1f) as usize;
    let mut p = off + 4;
    if epilog_count == 0 && code_words == 0 {
        let ext = u32le(&pe.data, p)?;
        epilog_count = (ext & 0xffff) as usize;
        code_words = ((ext >> 16) & 0xff) as usize;
        p += 4;
    }
    if e_bit == 0 {
        p = p
            .checked_add(
                4usize
                    .checked_mul(epilog_count)
                    .ok_or("epilog size overflow")?,
            )
            .ok_or("unwind offset overflow")?;
    }
    let length = 4usize
        .checked_mul(code_words)
        .ok_or("unwind code size overflow")?;
    Ok(Some((func_len, bytes(&pe.data, p, length)?.to_vec())))
}
fn walk(codes: &[u8]) -> Vec<usize> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < codes.len() {
        out.push(i);
        if matches!(codes[i], 0xe4 | 0xe5) {
            break;
        }
        i += code_len(codes[i]);
    }
    out
}
fn resolve_save_next(codes: &[u8], j: usize) -> Option<Pair> {
    let mut end = j;
    while end < codes.len() && codes[end] == 0xe6 {
        end += 1;
    }
    if end >= codes.len() {
        return None;
    }
    let ordinal = (end - j) as u32;
    let base = decode_pair(codes, end)?;
    Some(Pair {
        first: base.first + 2 * ordinal,
        offset: base.offset + 16 * ordinal,
        is_fp: base.is_fp,
    })
}

fn parse_int(text: &str) -> Option<i64> {
    let s = text.trim();
    if let Some(rest) = s.strip_prefix("-0x") {
        i64::from_str_radix(rest, 16).ok().map(|n| -n)
    } else if let Some(rest) = s.strip_prefix("0x") {
        i64::from_str_radix(rest, 16).ok()
    } else {
        s.parse().ok()
    }
}
fn prologue_saves(
    ctx: &BinaryContext,
    path: &Path,
    va: u64,
    func_len: u32,
) -> Result<BTreeMap<i64, (u32, Option<u32>, bool)>, String> {
    let span = if func_len == 0 {
        128
    } else {
        func_len.min(128)
    };
    let args = vec![
        "-d".into(),
        format!("--start-address={va}"),
        format!("--stop-address={}", va.saturating_add(span as u64)),
        "--no-show-raw-insn".into(),
        path.to_string_lossy().into_owned(),
    ];
    let output = ctx.command(&ctx.objdump(), &args)?;
    Ok(parse_prologue_saves(&output))
}
fn parse_prologue_saves(output: &str) -> BTreeMap<i64, (u32, Option<u32>, bool)> {
    let insn = rx(r"^\s*([0-9a-f]+):\s+(\S+)\s*(.*)$");
    let store = rx(r"([xd])(\d+),\s*(?:([xd])(\d+),\s*)?\[sp(?:,\s*#(-?0x[0-9a-f]+|-?\d+))?\](!?)");
    let mut saves = BTreeMap::new();
    for line in output.lines() {
        let Some(c) = insn.captures(line) else {
            continue;
        };
        let mnemonic = &c[2];
        let ops = c[3].split("//").next().unwrap_or("").trim();
        if mnemonic == "stp" || mnemonic == "str" {
            let Some(sm) = store.captures(ops) else {
                continue;
            };
            let is_fp = &sm[1] == "d";
            let Ok(r1) = sm[2].parse::<u32>() else {
                continue;
            };
            let r2 = sm.get(4).and_then(|m| m.as_str().parse::<u32>().ok());
            let mut offset = sm.get(5).and_then(|m| parse_int(m.as_str())).unwrap_or(0);
            if sm.get(6).is_some_and(|m| m.as_str() == "!") {
                offset = 0;
            }
            saves.insert(offset, (r1, r2, is_fp));
        } else if (mnemonic == "add" || mnemonic == "mov") && ops.starts_with("x29,") {
            break;
        } else if ["bl", "ret", "cbz", "cbnz"].contains(&mnemonic) && !saves.is_empty() {
            break;
        }
    }
    saves
}

#[derive(Default)]
struct Stats(BTreeMap<&'static str, usize>);
impl Stats {
    fn inc(&mut self, key: &'static str) {
        *self.0.entry(key).or_default() += 1;
    }
    fn get(&self, key: &'static str) -> usize {
        self.0.get(key).copied().unwrap_or(0)
    }
}
fn verify_pe(ctx: &BinaryContext, path: &Path) -> Result<Option<(Stats, Vec<String>)>, String> {
    let Some(pe) = parse_pe(path)? else {
        return Ok(None);
    };
    let Some(poff) = rva2off(&pe.sections, pe.pdata_rva) else {
        return Ok(None);
    };
    if pe.pdata_size == 0 {
        return Ok(None);
    }
    let mut stats = Stats::default();
    let mut mismatches = Vec::new();
    for k in 0..(pe.pdata_size as usize / 8) {
        let begin = u32le(&pe.data, poff + k * 8)?;
        let unwind = u32le(&pe.data, poff + k * 8 + 4)?;
        if unwind & 3 != 0 {
            stats.inc("packed");
            continue;
        }
        let Some((func_len, codes)) = unwind_stream(&pe, unwind)? else {
            continue;
        };
        if codes.is_empty() {
            continue;
        }
        stats.inc("funcs");
        let mut targets: Vec<(usize, Pair, &'static str)> = Vec::new();
        for j in walk(&codes) {
            let b0 = codes[j];
            if b0 == 0xe6 {
                stats.inc("save_next");
                if let Some(pair) = resolve_save_next(&codes, j) {
                    targets.push((j, pair, "save_next"));
                } else {
                    stats.inc("save_next_unresolved");
                }
            } else if b0 == 0xe7 {
                stats.inc("save_any_reg");
            } else if b0 == 0xdf {
                stats.inc("alloc_z");
            } else if let Some(pair) = decode_pair(&codes, j) {
                targets.push((j, pair, "pair"));
            }
        }
        if targets.is_empty() {
            continue;
        }
        let va = pe.image_base + begin as u64;
        let real = prologue_saves(ctx, path, va, func_len)?;
        if real.is_empty() {
            stats.inc("no_prologue");
            continue;
        }
        for (j, expected, kind) in targets {
            let Some(&(r1, r2, got_fp)) = real.get(&(expected.offset as i64)) else {
                stats.inc(if kind == "save_next" {
                    "save_next_no_slot"
                } else {
                    "pair_no_slot"
                });
                continue;
            };
            if r1 == expected.first && r2 == Some(expected.first + 1) && got_fp == expected.is_fp {
                stats.inc(if kind == "save_next" {
                    "save_next_ok"
                } else {
                    "pair_ok"
                });
            } else {
                stats.inc(if kind == "save_next" {
                    "save_next_MISMATCH"
                } else {
                    "pair_MISMATCH"
                });
                if mismatches.len() < 20 {
                    let expected_cls = if expected.is_fp { "d" } else { "x" };
                    let got_cls = if got_fp { "d" } else { "x" };
                    let got_regs = r2.map(|r| format!(",{got_cls}{r}")).unwrap_or_default();
                    mismatches.push(format!(
                        "    {va:#x} code[{j}] ({kind}): decoder says {expected_cls}{},{expected_cls}{}@sp+{} but prologue stores {got_cls}{r1}{got_regs}@sp+{}",
                        expected.first, expected.first + 1, expected.offset, expected.offset
                    ));
                }
            }
        }
    }
    Ok(Some((stats, mismatches)))
}
fn basename(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}
fn run_internal(ctx: &BinaryContext, check: bool) -> Result<CheckOutput, String> {
    let objdump = ctx.objdump();
    if crate::binary::executable(&objdump).is_none() {
        return Ok(CheckOutput {
            stdout: String::new(),
            stderr: "verify_arm64_unwind: no llvm-objdump found (.build-tools/llvm-mingw/bin)\n"
                .into(),
            returncode: 1,
        });
    }
    let paths = if ctx.build_dir.is_dir() {
        DEFAULT_TARGETS
            .iter()
            .map(|rel| ctx.build_dir.join(rel))
            .filter(|p| p.exists())
            .collect::<Vec<_>>()
    } else {
        vec![ctx.build_dir.clone()]
    };
    if ctx.build_dir.is_dir() && paths.is_empty() {
        return Ok(CheckOutput {
            stdout: String::new(),
            stderr: format!(
                "verify_arm64_unwind: no ARM64 PEs found under {}\n",
                ctx.build_dir.display()
            ),
            returncode: 1,
        });
    }
    let mut stdout = String::new();
    let mut total_mismatch = 0usize;
    let mut total_checked = 0usize;
    for path in paths {
        let Some((stats, mismatches)) = verify_pe(ctx, &path)? else {
            stdout.push_str(&format!(
                "{}: not an ARM64 PE with .pdata -- skipped\n",
                path.display()
            ));
            continue;
        };
        let checked = stats.get("save_next_ok")
            + stats.get("save_next_MISMATCH")
            + stats.get("pair_ok")
            + stats.get("pair_MISMATCH");
        let bad = stats.get("save_next_MISMATCH") + stats.get("pair_MISMATCH");
        total_mismatch += bad;
        total_checked += checked;
        stdout.push_str(&format!(
            "{}: {checked} save sites verified against disassembly -- {}\n",
            basename(&path),
            if bad == 0 {
                "OK".to_owned()
            } else {
                format!("{bad} MISMATCH")
            }
        ));
        stdout.push_str(&format!(
            "    functions with unwind codes : {}\n",
            stats.get("funcs")
        ));
        stdout.push_str(&format!(
            "    save_next codes             : {} ({} verified)\n",
            stats.get("save_next"),
            stats.get("save_next_ok")
        ));
        stdout.push_str(&format!(
            "    explicit pair saves         : {}\n",
            stats.get("pair_ok") + stats.get("pair_MISMATCH")
        ));
        for key in [
            "save_next_unresolved",
            "save_next_no_slot",
            "pair_no_slot",
            "no_prologue",
            "save_any_reg",
            "alloc_z",
        ] {
            let n = stats.get(key);
            if n != 0 {
                stdout.push_str(&format!("    {key:28}: {n}\n"));
            }
        }
        for line in mismatches {
            stdout.push_str(&format!("{line}\n"));
        }
    }
    stdout.push('\n');
    let rc = if total_mismatch != 0 {
        stdout.push_str(&format!("FAIL: {total_mismatch}/{total_checked} unwind-code save sites disagree with the real prologue\n"));
        if check {
            1
        } else {
            0
        }
    } else {
        stdout.push_str(&format!(
            "PASS: {total_checked} unwind-code save sites match the real prologue\n"
        ));
        0
    };
    Ok(CheckOutput {
        stdout,
        stderr: String::new(),
        returncode: rc,
    })
}
/// Native post-link gate behavior (equivalent to the legacy `--check` mode).
pub fn run(ctx: &BinaryContext) -> Result<CheckOutput, String> {
    run_internal(ctx, true)
}
/// Preserve the legacy report-only versus `--check` exit behavior for CLI wiring.
pub fn run_with_check(ctx: &BinaryContext, check: bool) -> Result<CheckOutput, String> {
    run_internal(ctx, check)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn put16(data: &mut [u8], at: usize, value: u16) {
        data[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
    fn put32(data: &mut [u8], at: usize, value: u32) {
        data[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    fn put64(data: &mut [u8], at: usize, value: u64) {
        data[at..at + 8].copy_from_slice(&value.to_le_bytes());
    }
    fn fixture_pe() -> Vec<u8> {
        let mut d = vec![0u8; 0x600];
        d[..2].copy_from_slice(b"MZ");
        put32(&mut d, 0x3c, 0x80);
        d[0x80..0x84].copy_from_slice(b"PE\0\0");
        let fh = 0x84;
        put16(&mut d, fh, 0xaa64);
        put16(&mut d, fh + 2, 1);
        put16(&mut d, fh + 16, 0xf0);
        let opt = fh + 20;
        put16(&mut d, opt, 0x20b);
        put64(&mut d, opt + 24, 0x180000000);
        put32(&mut d, opt + 112 + 3 * 8, 0x1000);
        put32(&mut d, opt + 112 + 3 * 8 + 4, 8);
        let sh = opt + 0xf0;
        put32(&mut d, sh + 8, 0x400);
        put32(&mut d, sh + 12, 0x1000);
        put32(&mut d, sh + 16, 0x400);
        put32(&mut d, sh + 20, 0x200);
        put32(&mut d, 0x200, 0x1200);
        put32(&mut d, 0x204, 0x1100);
        // Four-byte xdata header: four code bytes, 16-byte function, then save_fplr and end.
        put32(&mut d, 0x300, (1 << 27) | (4));
        d[0x304..0x308].copy_from_slice(&[0x40, 0x00, 0xe4, 0x00]);
        d
    }
    #[test]
    fn every_pair_encoding_decodes_its_register_class_and_offset() {
        assert_eq!(
            decode_pair(&[0x20], 0),
            Some(Pair {
                first: 19,
                offset: 0,
                is_fp: false
            })
        );
        assert_eq!(
            decode_pair(&[0x4a], 0),
            Some(Pair {
                first: 29,
                offset: 80,
                is_fp: false
            })
        );
        assert_eq!(
            decode_pair(&[0x8f], 0),
            Some(Pair {
                first: 29,
                offset: 0,
                is_fp: false
            })
        );
        assert_eq!(
            decode_pair(&[0xc9, 0x85], 0),
            Some(Pair {
                first: 25,
                offset: 40,
                is_fp: false
            })
        );
        assert_eq!(
            decode_pair(&[0xcd, 0x80], 0),
            Some(Pair {
                first: 25,
                offset: 0,
                is_fp: false
            })
        );
        assert_eq!(
            decode_pair(&[0xd9, 0x83], 0),
            Some(Pair {
                first: 14,
                offset: 24,
                is_fp: true
            })
        );
        assert_eq!(
            decode_pair(&[0xda, 0x80], 0),
            Some(Pair {
                first: 10,
                offset: 0,
                is_fp: true
            })
        );
    }
    #[test]
    fn save_next_ordinals_are_based_on_the_following_pair() {
        let codes = [0xe6, 0xe6, 0xc8, 0x02, 0xe4];
        assert_eq!(
            resolve_save_next(&codes, 0),
            Some(Pair {
                first: 23,
                offset: 48,
                is_fp: false
            })
        );
        assert_eq!(
            resolve_save_next(&codes, 1),
            Some(Pair {
                first: 21,
                offset: 32,
                is_fp: false
            })
        );
        assert_eq!(resolve_save_next(&[0xe6], 0), None);
    }
    #[test]
    fn walk_obeys_variable_code_lengths_and_end_marker() {
        assert_eq!(code_len(0x01), 1);
        assert_eq!(code_len(0xc8), 2);
        assert_eq!(code_len(0xdf), 2);
        assert_eq!(code_len(0xe0), 4);
        assert_eq!(code_len(0xe2), 2);
        assert_eq!(code_len(0xe7), 3);
        assert_eq!(walk(&[0xc8, 0x01, 0x40, 0xe4, 0x20]), vec![0, 2, 3]);
    }
    #[test]
    fn rva_mapping_uses_larger_virtual_or_raw_extent() {
        let sections = [Section {
            vaddr: 0x1000,
            vsize: 0x80,
            rawptr: 0x200,
            rawsize: 0x100,
        }];
        assert_eq!(rva2off(&sections, 0x10ff), Some(0x2ff));
        assert_eq!(rva2off(&sections, 0x1100), None);
    }
    #[test]
    fn prologue_store_offsets_parse_negative_hex_and_decimal() {
        assert_eq!(parse_int("-0x20"), Some(-32));
        assert_eq!(parse_int("0x28"), Some(40));
        assert_eq!(parse_int("48"), Some(48));
    }
    #[test]
    fn prologue_truth_reads_pair_and_preindexed_stores_then_stops_at_fp_setup() {
        let listing="  180001000: stp x19, x20, [sp, #0x20]\n  180001004: stp d8, d9, [sp, #-0x20]!\n  180001008: add x29, sp, #0x20\n  18000100c: stp x21, x22, [sp, #0x30]\n";
        let got = parse_prologue_saves(listing);
        assert_eq!(got.get(&32), Some(&(19, Some(20), false)));
        assert_eq!(got.get(&0), Some(&(8, Some(9), true)));
        assert!(!got.contains_key(&48));
    }
    #[test]
    fn parses_arm64_pe_pdata_and_unwind_stream() {
        let path = std::env::temp_dir().join(format!("arm64-unwind-{}.exe", std::process::id()));
        fs::write(&path, fixture_pe()).unwrap();
        let pe = parse_pe(&path).unwrap().unwrap();
        assert_eq!(pe.image_base, 0x180000000);
        assert_eq!(rva2off(&pe.sections, pe.pdata_rva), Some(0x200));
        assert_eq!(
            unwind_stream(&pe, 0x1100).unwrap(),
            Some((16, vec![0x40, 0, 0xe4, 0]))
        );
        let _ = fs::remove_file(path);
    }
    #[test]
    fn non_arm64_pe_is_skipped_without_parsing_unwind_metadata() {
        let path =
            std::env::temp_dir().join(format!("not-arm64-unwind-{}.exe", std::process::id()));
        let mut data = fixture_pe();
        put16(&mut data, 0x84, 0x8664);
        fs::write(&path, data).unwrap();
        assert!(parse_pe(&path).unwrap().is_none());
        let _ = fs::remove_file(path);
    }
}
