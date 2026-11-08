//! `vibeos-cli repro`: the determinism lane's gauge (docs/engineering/deterministic-builds-plan.md).
//!
//! Answers, with no external dependency: are two builds of one commit byte-identical — and if
//! not, WHICH PE sections differ and does the COFF TimeDateStamp diverge (the measured live
//! leak), so a leak is localized instead of just "the bytes differ"; and what
//! determinism-relevant metadata an artifact carries today.
//!
//! The parser reads only the COFF header and section table: PE32 vs PE32+ is irrelevant
//! because SizeOfOptionalHeader is in the COFF header. `verify` (two cold builds whose output
//! streams to the terminal) and the optional `diffoscope` stay in the facade, which uses
//! [`compare_files`] here.
use crate::{Error, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Default artifacts to diff across two build trees (relative to build-<arch>/): the kernel,
/// the two cargo-driven non-standard links, and a representative DLL.
pub const DEFAULT_ARTIFACTS: [&str; 3] = [
    "ntoskrnl/ntoskrnl.exe",
    "win32ss/win32k.sys",
    "dll/win32/kernel32/kernel32.dll",
];

#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub name: String,
    pub raw_off: u32,
    pub raw_size: u32,
    pub digest: String,
}
#[derive(Debug, Clone, PartialEq)]
pub struct PeImage {
    pub machine: u16,
    pub timestamp: u32,
    pub sections: Vec<Section>,
    pub size: usize,
    pub digest: String,
}

fn sha(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}
fn u16_at(d: &[u8], off: usize) -> u16 {
    let mut b = [0u8; 2];
    for (i, x) in b.iter_mut().enumerate() {
        *x = d.get(off + i).copied().unwrap_or(0);
    }
    u16::from_le_bytes(b)
}
fn u32_at(d: &[u8], off: usize) -> u32 {
    let mut b = [0u8; 4];
    for (i, x) in b.iter_mut().enumerate() {
        *x = d.get(off + i).copied().unwrap_or(0);
    }
    u32::from_le_bytes(b)
}

/// The COFF header and section table of a PE image; an error for a malformed one.
pub fn parse_pe(data: &[u8]) -> Result<PeImage> {
    if data.len() < 0x40 || &data[..2] != b"MZ" {
        return Err("not a PE image (no MZ header)".into());
    }
    let pe = u32_at(data, 0x3C) as usize;
    if pe + 24 > data.len() || &data[pe..pe + 4] != b"PE\0\0" {
        return Err("not a PE image (no PE signature)".into());
    }
    let coff = pe + 4;
    let count = u16_at(data, coff + 2) as usize;
    let table = coff + 20 + u16_at(data, coff + 16) as usize;
    let mut sections = Vec::with_capacity(count);
    for i in 0..count {
        let base = table + i * 40;
        if base + 40 > data.len() {
            return Err("truncated section table".into());
        }
        let raw = &data[base..base + 8];
        let end = raw.iter().rposition(|&b| b != 0).map_or(0, |p| p + 1);
        // latin-1: every byte is the code point of the same value.
        let name: String = raw[..end].iter().map(|&b| b as char).collect();
        let raw_size = u32_at(data, base + 16);
        let raw_off = u32_at(data, base + 20);
        let start = (raw_off as usize).min(data.len());
        let stop = (raw_off as usize + raw_size as usize).min(data.len());
        let body = if raw_size == 0 {
            &[][..]
        } else {
            &data[start..stop]
        };
        sections.push(Section {
            name,
            raw_off,
            raw_size,
            digest: sha(body),
        });
    }
    Ok(PeImage {
        machine: u16_at(data, coff),
        timestamp: u32_at(data, coff + 4),
        sections,
        size: data.len(),
        digest: sha(data),
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompareReport {
    pub identical: bool,
    pub timestamp_a: u32,
    pub timestamp_b: u32,
    /// Sections present in both but differing; and the section-set difference.
    pub section_diffs: Vec<String>,
    pub only_in_a: Vec<String>,
    pub only_in_b: Vec<String>,
}

/// Byte identity, and where two PE images diverge.
pub fn compare(a: &[u8], b: &[u8]) -> Result<CompareReport> {
    let (a, b) = (parse_pe(a)?, parse_pe(b)?);
    let map = |p: &PeImage| -> BTreeMap<String, String> {
        p.sections
            .iter()
            .map(|s| (s.name.clone(), s.digest.clone()))
            .collect()
    };
    let (sa, sb) = (map(&a), map(&b));
    let ka: BTreeSet<_> = sa.keys().cloned().collect();
    let kb: BTreeSet<_> = sb.keys().cloned().collect();
    Ok(CompareReport {
        identical: a.digest == b.digest,
        timestamp_a: a.timestamp,
        timestamp_b: b.timestamp,
        section_diffs: ka
            .intersection(&kb)
            .filter(|n| sa[*n] != sb[*n])
            .cloned()
            .collect(),
        only_in_a: ka.difference(&kb).cloned().collect(),
        only_in_b: kb.difference(&ka).cloned().collect(),
    })
}

/// Days since 1970-01-01 to a proleptic Gregorian (year, month, day).
fn civil(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}
/// A COFF TimeDateStamp: 0 is deterministic; anything else is hex plus a wall-clock hint.
pub fn ts(v: u32) -> String {
    if v == 0 {
        return "0 (deterministic)".into();
    }
    let secs = i64::from(v);
    let (y, mo, d) = civil(secs.div_euclid(86_400));
    let s = secs.rem_euclid(86_400);
    format!(
        "0x{v:08x} ({y:04}-{mo:02}-{d:02} {:02}:{:02}:{:02}Z)",
        s / 3600,
        s / 60 % 60,
        s % 60
    )
}

/// The report lines for one compared artifact.
pub fn compare_lines(name: &str, r: &CompareReport) -> Vec<String> {
    let mark = if r.identical {
        "✓ identical"
    } else {
        "✗ DIFFERS"
    };
    let mut out = vec![format!("  {mark}  {name}")];
    if r.identical {
        return out;
    }
    let ts_differs = r.timestamp_a != r.timestamp_b;
    if ts_differs {
        out.push(format!(
            "      TimeDateStamp: {}  vs  {}",
            ts(r.timestamp_a),
            ts(r.timestamp_b)
        ));
    }
    if !r.section_diffs.is_empty() {
        out.push(format!(
            "      sections differ: {}",
            r.section_diffs.join(", ")
        ));
    }
    let py_list = |v: &[String]| {
        format!(
            "[{}]",
            v.iter()
                .map(|s| format!("'{s}'"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    if !r.only_in_a.is_empty() || !r.only_in_b.is_empty() {
        out.push(format!(
            "      section set differs: -{} +{}",
            py_list(&r.only_in_a),
            py_list(&r.only_in_b)
        ));
    }
    if !(ts_differs
        || !r.section_diffs.is_empty()
        || !r.only_in_a.is_empty()
        || !r.only_in_b.is_empty())
    {
        out.push("      headers/overlay differ (no section-body or timestamp delta)".into());
    }
    out
}

fn read(path: &Path, lines: &mut Vec<String>) -> Option<Vec<u8>> {
    match fs::read(path) {
        Ok(d) => Some(d),
        Err(e) => {
            lines.push(format!(
                "  ! cannot read {}: {}",
                path.display(),
                py_os_error(&e, path)
            ));
            None
        }
    }
}
/// Python's `str(OSError)` for a path, which callers of the old facade saw.
fn py_os_error(e: &std::io::Error, path: &Path) -> String {
    let code = e.raw_os_error().unwrap_or(0);
    let msg = e.to_string();
    let text = msg.split(" (os error").next().unwrap_or(&msg);
    format!("[Errno {code}] {text}: '{}'", path.display())
}

/// `repro compare A B` (and verify's per-artifact check): lines, and identical or not.
/// `ok` is false only when nothing could be compared.
pub fn compare_files(a: &Path, b: &Path, name: &str) -> Value {
    let mut lines = Vec::new();
    let (Some(da), Some(db)) = (read(a, &mut lines), read(b, &mut lines)) else {
        return json!({"ok": false, "identical": false, "lines": lines});
    };
    match compare(&da, &db) {
        Ok(r) => json!({"ok": true, "identical": r.identical, "lines": compare_lines(name, &r)}),
        Err(e) => {
            lines.push(format!("  ! parse error: {e}"));
            json!({"ok": false, "identical": false, "lines": lines})
        }
    }
}

/// `repro report F...`: the metadata determinism cares about, per artifact.
pub fn report(files: &[PathBuf]) -> Value {
    let (mut lines, mut rc) = (Vec::new(), 0);
    for p in files {
        let Some(data) = read(p, &mut lines) else {
            rc = 1;
            continue;
        };
        match parse_pe(&data) {
            Ok(pe) => {
                let leak = if pe.timestamp != 0 { " ← LEAK" } else { "" };
                lines.push(format!("  {}", p.display()));
                lines.push(format!("      sha256        {}", pe.digest));
                lines.push(format!("      TimeDateStamp {}{leak}", ts(pe.timestamp)));
                lines.push(format!("      sections      {}", pe.sections.len()));
            }
            Err(e) => {
                lines.push(format!("  ! {}: {e}", p.display()));
                rc = 1;
            }
        }
    }
    json!({"lines": lines, "rc": rc})
}

/// `repro diff-dirs A B`: the staged artifact set across two build trees. `differing` names
/// the pairs a facade may hand to diffoscope.
pub fn diff_dirs(dir_a: &Path, dir_b: &Path, artifacts: &[String]) -> Value {
    let mut lines = vec![format!(
        "vibeos-cli repro — diff {}  vs  {}\n",
        dir_a.display(),
        dir_b.display()
    )];
    let (mut any_diff, mut any_found) = (false, false);
    let mut differing = Vec::new();
    for rel in artifacts {
        let (a, b) = (dir_a.join(rel), dir_b.join(rel));
        if !a.is_file() || !b.is_file() {
            lines.push(format!("  - skip {rel} (missing in one tree)"));
            continue;
        }
        any_found = true;
        let r = compare_files(&a, &b, rel);
        if r["ok"] != true {
            // Unreadable or unparsable: the old facade printed "! rel: why".
            let why: Vec<String> = r["lines"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|l| l.as_str())
                .map(|l| {
                    l.trim_start()
                        .trim_start_matches("! parse error: ")
                        .to_owned()
                })
                .collect();
            lines.push(format!("  ! {rel}: {}", why.join("; ")));
            any_diff = true;
            continue;
        }
        lines.extend(
            r["lines"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|l| l.as_str().map(str::to_owned)),
        );
        if r["identical"] != true {
            any_diff = true;
            differing.push(json!([a, b]));
        }
    }
    if !any_found {
        lines.push("  (no artifacts found in both trees — build both first)".into());
        return json!({"lines": lines, "rc": 1, "differing": differing});
    }
    lines.push(String::new());
    lines.push(
        if any_diff {
            "  → not reproducible (see diffs above)"
        } else {
            "  → byte-identical ✓"
        }
        .into(),
    );
    json!({"lines": lines, "rc": i32::from(any_diff), "differing": differing})
}

fn path_arg(a: &Value, k: &str) -> Result<PathBuf> {
    a[k].as_str()
        .map(PathBuf::from)
        .ok_or_else(|| Error::new(format!("missing {k}")))
}
fn artifacts(a: &Value) -> Vec<String> {
    let given: Vec<String> = serde_json::from_value(a["artifacts"].clone()).unwrap_or_default();
    if given.is_empty() {
        DEFAULT_ARTIFACTS.iter().map(|s| (*s).to_owned()).collect()
    } else {
        given
    }
}

pub fn dispatch(request: &Value) -> Result<Value> {
    let a = &request["kwargs"];
    match request["op"].as_str().unwrap_or("") {
        "repro_compare" => Ok(compare_files(
            &path_arg(a, "a")?,
            &path_arg(a, "b")?,
            a["name"].as_str().unwrap_or(""),
        )),
        "repro_report" => {
            let files: Vec<PathBuf> = serde_json::from_value(a["files"].clone())?;
            Ok(report(&files))
        }
        "repro_diff_dirs" => Ok(diff_dirs(
            &path_arg(a, "dir_a")?,
            &path_arg(a, "dir_b")?,
            &artifacts(a),
        )),
        "repro_default_artifacts" => Ok(json!(DEFAULT_ARTIFACTS)),
        op => Err(format!("unknown repro operation {op}").into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal valid PE: COFF header, section table and bodies, no optional header.
    fn mk_pe(timestamp: u32, sections: &[(&str, &[u8])]) -> Vec<u8> {
        let pe_off = 0x40usize;
        let mut img = vec![0u8; pe_off];
        img[..2].copy_from_slice(b"MZ");
        img[0x3C..0x40].copy_from_slice(&(pe_off as u32).to_le_bytes());
        img.extend_from_slice(b"PE\0\0");
        img.extend_from_slice(&0x8664u16.to_le_bytes());
        img.extend_from_slice(&(sections.len() as u16).to_le_bytes());
        img.extend_from_slice(&timestamp.to_le_bytes());
        img.extend_from_slice(&[0u8; 12]); // symtab ptr, symbol count, opt size, characteristics
        let mut cursor = img.len() + 40 * sections.len();
        let mut bodies = Vec::new();
        for (name, body) in sections {
            let mut nm = [0u8; 8];
            nm[..name.len().min(8)].copy_from_slice(&name.as_bytes()[..name.len().min(8)]);
            img.extend_from_slice(&nm);
            for v in [body.len() as u32, 0x1000, body.len() as u32, cursor as u32] {
                img.extend_from_slice(&v.to_le_bytes());
            }
            img.extend_from_slice(&[0u8; 16]);
            bodies.extend_from_slice(body);
            cursor += body.len();
        }
        img.extend(bodies);
        img
    }

    #[test]
    fn the_parser_reads_the_fields_determinism_cares_about() {
        let img = mk_pe(0x1234_5678, &[(".text", &[0xC3; 16]), (".data", &[0; 8])]);
        let pe = parse_pe(&img).unwrap();
        assert_eq!((pe.machine, pe.timestamp), (0x8664, 0x1234_5678));
        let names: Vec<_> = pe.sections.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, [".text", ".data"]);
        assert_eq!(pe.sections[0].raw_size, 16);
        assert!(parse_pe(&b"not a pe at all".repeat(8)).is_err());
        let mut bad = img.clone();
        bad.truncate(0x40 + 24 + 20);
        assert!(parse_pe(&bad)
            .unwrap_err()
            .to_string()
            .contains("truncated"));
    }

    #[test]
    fn a_timestamp_only_difference_is_the_leak_signature() {
        let img = mk_pe(0xCAFE_BABE, &[(".text", &[0x90; 32])]);
        let same = compare(&img, &img).unwrap();
        assert!(same.identical && same.section_diffs.is_empty());
        assert_eq!(compare_lines("k.exe", &same), ["  ✓ identical  k.exe"]);
        let a = mk_pe(0x1000_0000, &[(".text", &[0x90; 32]), (".rdata", b"abc")]);
        let b = mk_pe(0x2000_0000, &[(".text", &[0x90; 32]), (".rdata", b"abc")]);
        let r = compare(&a, &b).unwrap();
        assert!(!r.identical && r.timestamp_a != r.timestamp_b && r.section_diffs.is_empty());
        assert_eq!(
            compare_lines("k.exe", &r)[1],
            "      TimeDateStamp: 0x10000000 (1978-07-04 21:24:16Z)  vs  0x20000000 (1987-01-05 18:48:32Z)"
        );
    }

    #[test]
    fn a_changed_section_is_localized_and_a_set_change_named() {
        let a = mk_pe(0x1000_0000, &[(".text", &[0x90; 32]), (".data", b"old")]);
        let b = mk_pe(0x1000_0000, &[(".text", &[0x90; 32]), (".data", b"new")]);
        let r = compare(&a, &b).unwrap();
        assert_eq!(r.section_diffs, [".data"]);
        let c = mk_pe(0x1000_0000, &[(".text", &[0x90; 32]), (".tls", b"x")]);
        let lines = compare_lines("m.dll", &compare(&a, &c).unwrap());
        assert_eq!(lines[1], "      section set differs: -['.data'] +['.tls']");
    }

    #[test]
    fn timestamps_render_like_the_python() {
        assert_eq!(ts(0), "0 (deterministic)");
        assert_eq!(ts(1), "0x00000001 (1970-01-01 00:00:01Z)");
        assert_eq!(ts(0xFFFF_FFFF), "0xffffffff (2106-02-07 06:28:15Z)");
        assert_eq!(ts(951_782_400), "0x38bb0c00 (2000-02-29 00:00:00Z)");
    }

    #[test]
    fn diff_dirs_skips_missing_and_reports_the_verdict() {
        let tmp = tempfile::tempdir().unwrap();
        let (a, b) = (tmp.path().join("a"), tmp.path().join("b"));
        for d in [&a, &b] {
            fs::create_dir_all(d.join("nt")).unwrap();
        }
        fs::write(a.join("nt/k.exe"), mk_pe(0, &[(".text", b"x")])).unwrap();
        fs::write(b.join("nt/k.exe"), mk_pe(0, &[(".text", b"x")])).unwrap();
        fs::write(a.join("nt/only.dll"), b"MZ").unwrap();
        let arts = vec!["nt/k.exe".to_owned(), "nt/only.dll".to_owned()];
        let r = diff_dirs(&a, &b, &arts);
        assert_eq!(r["rc"], 0);
        let lines: Vec<&str> = r["lines"]
            .as_array()
            .unwrap()
            .iter()
            .map(|l| l.as_str().unwrap())
            .collect();
        assert!(lines.contains(&"  - skip nt/only.dll (missing in one tree)"));
        assert_eq!(*lines.last().unwrap(), "  → byte-identical ✓");
        fs::write(b.join("nt/k.exe"), mk_pe(7, &[(".text", b"x")])).unwrap();
        let r = diff_dirs(&a, &b, &arts);
        assert_eq!(
            (r["rc"].as_i64(), r["differing"].as_array().unwrap().len()),
            (Some(1), 1)
        );
        let none = diff_dirs(&a, &tmp.path().join("missing"), &arts);
        assert_eq!(none["rc"], 1);
    }
}
