//! Native serial verdicts and offline symbolication. Configuration crosses explicitly.
use crate::{Error, Result};
use regex::Regex;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, path::Path, process::Command};

pub fn re(pattern: &str) -> Regex {
    Regex::new(pattern).expect("static verdict regex")
}
pub fn py_lines(text: &str) -> Vec<String> {
    text.replace("\r\n", "\n")
        .split_terminator(|c| {
            matches!(
                c,
                '\n' | '\r'
                    | '\u{b}'
                    | '\u{c}'
                    | '\u{1c}'
                    | '\u{1d}'
                    | '\u{1e}'
                    | '\u{85}'
                    | '\u{2028}'
                    | '\u{2029}'
            )
        })
        .map(str::to_owned)
        .collect()
}
fn head(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}
pub fn args(r: &Value) -> &Value {
    r.get("kwargs").unwrap_or(r)
}
pub fn string<'a>(a: &'a Value, name: &str) -> Result<&'a str> {
    a.get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| Error::new(format!("missing string argument {name}")))
}
fn opt_string<'a>(a: &'a Value, name: &str) -> Option<&'a str> {
    a.get(name).and_then(Value::as_str)
}
fn bool_arg(a: &Value, name: &str) -> bool {
    a.get(name).and_then(Value::as_bool).unwrap_or(false)
}
fn hex(s: &str) -> Option<u64> {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).ok()
}
const BUGCHECK: &str =
    r"BugCheck\s+(0x[0-9A-Fa-f]+)|\*\*\*\s*(?:STOP|BUGCHECK):?\s*(0x[0-9A-Fa-f]+)";
const BUGCHECK_PARAMS: &str = r"\*\*\*\s*BUGCHECK:\s*(0x[0-9A-Fa-f]+)\s*\(\s*(0x[0-9A-Fa-f]+)\s*,\s*(0x[0-9A-Fa-f]+)\s*,\s*(0x[0-9A-Fa-f]+)\s*,\s*(0x[0-9A-Fa-f]+)\s*\)";
const MODULE_OFFSET: &str = r"(?m)^\*\*\*\s+P(\d)\s*=\s*(\S+?)\+0x([0-9a-fA-F]+)\s*$";
const KERNEL_BASE: &str = r"\[KRNLBASE\] ntoskrnl runtime base=(?:0x)?([0-9a-fA-F]+)";
const KERNEL_ASSERT: &str = r"\*\*\* ASSERT FAILED: .* at ntoskrnl-rust/";
const CENSUS: &str = r"\[CENSUS\] STUCK\b";
const LOCKDEP: &str = r"\[LOCKDEP\][^\n]*(?:SELF-DEADLOCK|ABBA)";
const CSWITCH: &str = r"\[CSWITCH\][^\n]*(?:HIGH_IRQL_AT_SWITCH|IRQL_UNBALANCED_SWITCH|SWAP_IRQL|SWITCH_FROM_DPC|CURTHREAD_INCOHERENT|STACK_OOB)";
const PE_BASE: u64 = 0x400000;
const MAX_OFFSET: u64 = 0x100000;

pub fn known_flakes() -> Value {
    json!([{
        "name":"i386_mp_win32k_av_0x1e", "when":"bugcheck", "code":"0x0000001E",
        "any":["win32k.sys+0x65EE1"],
        "note":"i386 MP boot: win32k AV at winlogon desktop bringup (1/204=0.49%); retry"
    }])
}
pub fn match_known_flake(
    serial: &str,
    code: Option<&str>,
    when: &str,
    catalog: &Value,
) -> Option<Value> {
    for fl in catalog.as_array().into_iter().flatten() {
        if fl.get("when").map_or(Some("bugcheck"), Value::as_str) != Some(when) {
            continue;
        }
        if let Some(want) = fl
            .get("code")
            .and_then(Value::as_str)
            .filter(|v| !v.is_empty())
        {
            if !code.is_some_and(|c| c.eq_ignore_ascii_case(want)) {
                continue;
            }
        }
        if !fl
            .get("any")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .all(|v| v.as_str().is_some_and(|s| serial.contains(s)))
        {
            continue;
        }
        if fl
            .get("none")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .any(|v| v.as_str().is_some_and(|s| serial.contains(s)))
        {
            continue;
        }
        return Some(fl.clone());
    }
    None
}
pub fn flake_note(flake: Option<&Value>) -> Vec<String> {
    flake
        .and_then(|f| f.get("note"))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(|s| vec![s.to_owned()])
        .unwrap_or_default()
}
pub fn wedge_evidence(serial: &str) -> Value {
    let census = re(CENSUS);
    let lockdep = re(LOCKDEP);
    let cswitch = re(CSWITCH);
    let lines: Vec<_> = py_lines(serial)
        .into_iter()
        .filter(|l| census.is_match(l) || lockdep.is_match(l) || cswitch.is_match(l))
        .map(|l| l.trim().to_owned())
        .collect();
    json!({"census_stuck":lines.iter().filter(|l|census.is_match(l)).count(),
        "lockdep":lines.iter().filter(|l|lockdep.is_match(l)).count(),
        "cswitch":lines.iter().filter(|l|cswitch.is_match(l)).count(),"lines":lines.iter().take(3).collect::<Vec<_>>()})
}
pub fn kernel_asserts(serial: &str) -> Vec<String> {
    let pattern = re(KERNEL_ASSERT);
    py_lines(serial)
        .into_iter()
        .filter(|l| pattern.is_match(l))
        .map(|l| l.trim().to_owned())
        .collect()
}
fn panic_loc(serial: &str) -> Option<String> {
    if let Some(c) = re(r"(?s)RUST PANIC HIT.*?PANIC LOC:\s*(\S+)").captures(serial) {
        return Some(c[1].to_owned());
    }
    re(r"panicked at\s+'([^']*)',\s*(\S+)")
        .captures(serial)
        .map(|c| c[2].to_owned())
}
fn kernel_panic(raw: &str) -> Option<String> {
    re(r"PANIC:([^\s:]+):(\d+)")
        .captures(raw)
        .map(|c| format!("{}:{}", &c[1], &c[2]))
}
fn bug_code(serial: &str) -> Option<String> {
    re(BUGCHECK)
        .captures(serial)
        .and_then(|c| c.get(1).or_else(|| c.get(2)).map(|m| m.as_str().to_owned()))
}
fn tri(classification: &str, salient: Vec<String>, flake: Option<&Value>) -> Value {
    json!({"classification":classification,"salient":salient,"known_flake":flake.and_then(|f|f.get("name"))})
}
fn add_field(mut value: Value, key: &str, field: Value) -> Value {
    value.as_object_mut().unwrap().insert(key.to_owned(), field);
    value
}
pub fn post_boot_check(serial: &str, raw: &str, catalog: &Value) -> Value {
    if let Some(loc) = panic_loc(serial) {
        return add_field(
            tri(
                "RUST_PANIC",
                vec![format!("RUST PANIC at {loc} (after boot marker)")],
                None,
            ),
            "panic_loc",
            json!(loc),
        );
    }
    if let Some(loc) = kernel_panic(raw) {
        return add_field(
            tri(
                "RUST_PANIC",
                vec![format!(
                    "KERNEL RUST PANIC at {loc} (debugcon, after boot marker)"
                )],
                None,
            ),
            "panic_loc",
            json!(loc),
        );
    }
    if let Some(code) = bug_code(serial) {
        let mut salient = vec![format!("BugCheck {code} (after boot marker)")];
        if let Some(line) = py_lines(serial).iter().find(|l| l.contains("BugCheck")) {
            let l = line.trim();
            if !l.is_empty() {
                salient.push(head(l, 140));
            }
        }
        let flake = match_known_flake(serial, Some(&code), "bugcheck", catalog);
        salient.extend(flake_note(flake.as_ref()));
        return add_field(
            tri(
                if flake.is_some() {
                    "KNOWN_FLAKE"
                } else {
                    "BUGCHECK"
                },
                salient,
                flake.as_ref(),
            ),
            "bugcheck",
            json!(code),
        );
    }
    if let Some(m) = re(r"\[[A-Z][A-Z0-9_-]*\][^\n]*?\(FAIL\)").find(serial) {
        return tri(
            "SELFTEST_FAIL",
            vec![format!("boot self-test failed: {}", head(m.as_str(), 160))],
            None,
        );
    }
    Value::Null
}
pub fn classify(
    serial: &str,
    reached: bool,
    fail_marker: Option<&str>,
    debugcon: Option<&str>,
    raw: &str,
    catalog: &Value,
) -> Value {
    if let Some(loc) = panic_loc(serial) {
        return add_field(
            tri("RUST_PANIC", vec![format!("RUST PANIC at {loc}")], None),
            "panic_loc",
            json!(loc),
        );
    }
    if let Some(loc) = kernel_panic(raw) {
        return add_field(
            tri(
                "RUST_PANIC",
                vec![format!("KERNEL RUST PANIC at {loc} (debugcon)")],
                None,
            ),
            "panic_loc",
            json!(loc),
        );
    }
    if let Some(code) = bug_code(serial) {
        let mut salient = vec![format!("BugCheck {code}")];
        if let Some(line) = py_lines(serial).iter().find(|l| l.contains("BugCheck")) {
            let l = line.trim();
            if !l.is_empty() {
                salient.push(head(l, 140));
            }
        }
        let flake = match_known_flake(serial, Some(&code), "bugcheck", catalog);
        salient.extend(flake_note(flake.as_ref()));
        return add_field(
            tri(
                if flake.is_some() {
                    "KNOWN_FLAKE"
                } else {
                    "BUGCHECK"
                },
                salient,
                flake.as_ref(),
            ),
            "bugcheck",
            json!(code),
        );
    }
    if let Some(fail) = fail_marker.filter(|s| !s.is_empty()) {
        return tri(
            "FAIL_MARKER",
            vec![format!("hit fail marker: {fail}")],
            None,
        );
    }
    if !reached {
        let mut salient = Vec::new();
        if let Some(d) = debugcon.filter(|s| !s.is_empty()) {
            let lines: Vec<_> = py_lines(d)
                .into_iter()
                .filter(|l| !l.trim().is_empty())
                .collect();
            salient.extend(
                lines
                    .iter()
                    .skip(lines.len().saturating_sub(3))
                    .map(|l| format!("debugcon: {l}")),
            );
        }
        let lines: Vec<_> = py_lines(serial)
            .into_iter()
            .filter(|l| !l.trim().is_empty())
            .collect();
        salient.extend(
            lines
                .iter()
                .skip(lines.len().saturating_sub(2))
                .map(|l| format!("serial tail: {}", head(l, 120))),
        );
        let ev = wedge_evidence(serial);
        let c = ev["census_stuck"].as_u64().unwrap();
        let l = ev["lockdep"].as_u64().unwrap();
        let s = ev["cswitch"].as_u64().unwrap();
        let mut fields = Map::new();
        if c + l + s > 0 {
            fields.insert("wedge_confirmed".into(), json!(true));
            fields.insert("census_stuck".into(), json!(c));
            fields.insert("lockdep_hits".into(), json!(l));
            fields.insert("cswitch_hits".into(), json!(s));
            salient.insert(0,format!("kernel self-report: {c} [CENSUS] STUCK, {l} [LOCKDEP], {s} [CSWITCH] — wedge CONFIRMED, not contention"));
            salient.extend(
                ev["lines"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .take(2)
                    .filter_map(Value::as_str)
                    .map(str::to_owned),
            );
        }
        let flake = match_known_flake(serial, None, "hang", catalog);
        salient.extend(flake_note(flake.as_ref()));
        let mut out = tri(
            if flake.is_some() {
                "KNOWN_FLAKE"
            } else {
                "HANG_TIMEOUT"
            },
            salient,
            flake.as_ref(),
        );
        out.as_object_mut().unwrap().extend(fields);
        return out;
    }
    tri("UNKNOWN", Vec::new(), None)
}
fn pc_param(code: u64) -> Option<usize> {
    match code {
        0x1e => Some(2),
        0x50 | 0x8e | 0x7e => Some(3),
        0x0a => Some(4),
        _ => None,
    }
}
pub fn kernel_runtime_base(serial: &str) -> String {
    re(KERNEL_BASE)
        .captures_iter(serial)
        .last()
        .and_then(|c| hex(&c[1]))
        .map(|n| format!("{n:#x}"))
        .unwrap_or_default()
}
pub fn kernel_relocation(arch: &str, serial: &str) -> Option<String> {
    if let Some(c) = re(KERNEL_BASE).captures_iter(serial).last() {
        let base = hex(&c[1])?;
        return Some(if base >= PE_BASE {
            format!("{:#x}", base - PE_BASE)
        } else {
            format!("-0x{:x}", PE_BASE - base)
        });
    }
    if matches!(arch, "i386" | "i386-uefi") {
        Some("0x800c3000".into())
    } else {
        None
    }
}
pub fn faulting_module_offset(serial: &str) -> Option<(String, u64)> {
    let c = re(MODULE_OFFSET).captures_iter(serial).last()?;
    let index = c[1].parse::<usize>().ok()?;
    if let Some(bc) = re(BUGCHECK_PARAMS).captures_iter(serial).last() {
        let code = hex(&bc[1])?;
        if matches!(code, 0xf4 | 0xef) {
            return None;
        }
        if pc_param(code).is_some_and(|want| want != index) {
            return None;
        }
    }
    Some((c[2].to_owned(), hex(&c[3])?))
}
pub fn faulting_pc(serial: &str) -> Option<String> {
    if let Some(c) = re(BUGCHECK_PARAMS).captures_iter(serial).last() {
        if let Some(index) = hex(&c[1]).and_then(pc_param) {
            if hex(&c[index + 1]).is_some_and(|n| n != 0) {
                return Some(c[index + 1].to_owned());
            }
        }
    }
    re(r"\bPC[:=]\s*(0x[0-9A-Fa-f]+)|\b(?:RIP|EIP)[:=]\s*(0x[0-9A-Fa-f]+)")
        .captures_iter(serial)
        .last()
        .and_then(|c| c.get(1).or_else(|| c.get(2)).map(|m| m.as_str().to_owned()))
}
fn cpu_blocks(state: &str) -> Vec<(u64, &str)> {
    let pattern = re(r"(?m)^CPU#(\d+)\b");
    let banners: Vec<_> = pattern.captures_iter(state).collect();
    banners
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            Some((
                c[1].parse().ok()?,
                &state[c.get(0)?.end()
                    ..banners
                        .get(i + 1)
                        .map(|b| b.get(0).unwrap().start())
                        .unwrap_or(state.len())],
            ))
        })
        .collect()
}
fn cpu_registers(state: &str, pattern: &str) -> Vec<(u64, String)> {
    let r = re(pattern);
    cpu_blocks(state)
        .into_iter()
        .filter_map(|(n, b)| r.captures(b).map(|c| (n, c[1].to_owned())))
        .collect()
}
pub fn cpu_pcs(s: &str) -> Vec<(u64, String)> {
    cpu_registers(s, r"\b(?:PC|RIP|EIP)=([0-9A-Fa-f]{8,16})\b")
}
pub fn cpu_sps(s: &str) -> Vec<(u64, String)> {
    cpu_registers(s, r"\b(?:SP|RSP|ESP)=([0-9A-Fa-f]{8,16})\b")
}
pub fn cpu_fps(s: &str) -> Vec<(u64, String)> {
    cpu_registers(s, r"\b(?:X29|RBP|EBP)=([0-9A-Fa-f]{8,16})\b")
}
pub fn cpu_frames(state: &str) -> Vec<(u64, u64, String)> {
    let banner = re(r"(?m)^=== CPU#(\d+) frames ");
    let frame = re(r"(?m)^  #(\d+) fp=([0-9a-f]+) ret=([0-9a-f]+)$");
    let banners: Vec<_> = banner.captures_iter(state).collect();
    let mut out = Vec::new();
    for (i, c) in banners.iter().enumerate() {
        let start = c.get(0).unwrap().end();
        let mut end = banners
            .get(i + 1)
            .map(|b| b.get(0).unwrap().start())
            .unwrap_or(state.len());
        if let Some(next) = state[start..].find("\n=== ") {
            end = end.min(start + next);
        }
        for f in frame.captures_iter(&state[start..end]) {
            out.push((
                c[1].parse().unwrap(),
                f[1].parse().unwrap(),
                f[3].to_owned(),
            ));
        }
    }
    out
}
pub fn cpu_kernel_at_user_pc(state: &str) -> Vec<(u64, String)> {
    let pc = re(r"\b(?:PC|RIP|EIP)=([0-9A-Fa-f]{8,16})\b");
    let ps = re(r"\bPSTATE=([0-9A-Fa-f]{8,16})\b");
    cpu_blocks(state)
        .into_iter()
        .filter_map(|(n, b)| {
            let p = pc.captures(b)?;
            let s = ps.captures(b)?;
            if hex(&s[1])? & 0xf != 0 && hex(&p[1])? < 0x0000800000000000 {
                Some((n, p[1].to_owned()))
            } else {
                None
            }
        })
        .collect()
}
pub fn guest_interrupt_time(state: &str) -> Option<u64> {
    re(r"(?m)^InterruptTime=0x([0-9a-f]+)\b")
        .captures(state)
        .and_then(|c| hex(&c[1]))
}

pub fn resolve_addr(
    repo: &Path,
    arch: &str,
    pc: &str,
    mp: bool,
    reloc: Option<&str>,
) -> Option<String> {
    let tool = repo.join("qemu/addr2sym.sh");
    if !tool.is_file() {
        return None;
    }
    let mut cmd = Command::new("bash");
    cmd.arg(&tool).arg(arch);
    if let Some(r) = reloc.filter(|s| !s.is_empty()) {
        cmd.args(["--reloc", r]);
    }
    if mp {
        cmd.arg("--mp");
    }
    cmd.arg(pc).current_dir(repo);
    let output = vibeos_build::process::capture(&mut cmd, 30).ok()?;
    let out = output.stdout.trim();
    if out.is_empty() {
        return None;
    }
    if let Some(offset) = re(r"\+(\d+)\b")
        .captures(out)
        .and_then(|c| c[1].parse::<u64>().ok())
        .filter(|n| *n > MAX_OFFSET)
    {
        return Some(format!(
            "{pc} -> (no symbol: nearest is {offset:#x} away — outside any function)"
        ));
    }
    Some(format!("{pc} -> {out}"))
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Module {
    pub base: u64,
    pub end: u64,
    pub name: String,
    pub path: String,
    pub unique: bool,
}
pub fn load_modules(repo: &Path, arch: &str) -> Vec<Module> {
    let Ok(text) = fs::read_to_string(repo.join(format!("qemu/modules-{arch}.tsv"))) else {
        return Vec::new();
    };
    py_lines(&text)
        .iter()
        .filter_map(|l| {
            let p: Vec<_> = l.split('\t').collect();
            if p.len() != 5 {
                return None;
            }
            Some(Module {
                base: hex(p[0])?,
                end: hex(p[1])?,
                name: p[2].into(),
                path: p[3].into(),
                unique: p[4] == "unique",
            })
        })
        .collect()
}
fn symbol_hit(text: &str, module: Option<&str>, addr: u64) -> Option<(String, u64, String)> {
    let mut best = None;
    for l in text.lines() {
        let p: Vec<_> = l.split('\t').collect();
        if p.len() != 4 {
            continue;
        }
        if module.is_some_and(|m| p[0] != m) {
            if best.is_some() {
                break;
            }
            continue;
        }
        let Some(a) = hex(p[1]) else {
            continue;
        };
        if a <= addr && best.as_ref().is_none_or(|(_, b, _)| a > *b) {
            best = Some((p[2].into(), a, p[3].into()));
        }
    }
    best
}
pub fn lookup_symbol(
    repo: &Path,
    arch: &str,
    module: &str,
    addr: u64,
) -> Option<(String, u64, String)> {
    let t = fs::read_to_string(repo.join(format!("qemu/symbols-{arch}-user.tsv"))).ok()?;
    symbol_hit(&t, Some(module), addr)
}
pub fn lookup_symbol_live(
    repo: &Path,
    arch: &str,
    module: &Module,
    addr: u64,
) -> Option<(String, u64, String)> {
    let mut c = Command::new("bash");
    c.arg(repo.join("qemu/build-user-index.sh"))
        .arg(arch)
        .arg("--one")
        .arg(repo.join(&module.path))
        .current_dir(repo);
    let out = vibeos_build::process::capture(&mut c, 30).ok()?;
    if out.code != 0 {
        return None;
    }
    symbol_hit(&out.stdout, None, addr)
}
fn all_identical(repo: &Path, modules: &[Module]) -> bool {
    let mut digests = BTreeSet::new();
    for m in modules {
        let Ok(b) = fs::read(repo.join(&m.path)) else {
            return false;
        };
        digests.insert(format!("{:x}", Sha256::digest(&b)));
    }
    digests.len() == 1
}
pub fn symbol_resolve(
    repo: &Path,
    arch: &str,
    mut addr: u64,
    want: Option<&str>,
    offset: bool,
) -> String {
    let modules = load_modules(repo, arch);
    if modules.is_empty() {
        return format!("{addr:#x} -> (no index; run qemu/build-user-index.sh {arch})");
    }
    let mut candidates: Vec<_> = modules
        .into_iter()
        .filter(|m| {
            want.map(|w| m.name.eq_ignore_ascii_case(w))
                .unwrap_or(m.base <= addr && addr < m.end)
        })
        .collect();
    if candidates.is_empty() {
        return if let Some(w) = want {
            format!("{addr:#x} -> (no module named {w} in the {arch} index)")
        } else {
            format!("{addr:#x} -> (no user-mode module covers this address)")
        };
    }
    let identical = candidates.len() > 1 && all_identical(repo, &candidates);
    if identical {
        candidates.truncate(1);
    }
    let unique: Vec<_> = candidates
        .iter()
        .filter(|m| want.is_some() || identical || m.unique)
        .collect();
    if unique.is_empty() {
        let names: BTreeSet<_> = candidates.iter().map(|m| m.name.clone()).collect();
        let names = names.into_iter().take(6).collect::<Vec<_>>().join(", ");
        return format!(
            "{addr:#x} -> (ambiguous: {} modules share this base — {names}{}; pass --module)",
            candidates.len(),
            if candidates.len() > 6 { ", ..." } else { "" }
        );
    }
    if unique.len() > 1 {
        let names: BTreeSet<_> = unique.iter().map(|m| m.name.clone()).collect();
        return format!(
            "{addr:#x} -> (overlapping modules: {}; pass --module)",
            names.into_iter().collect::<Vec<_>>().join(", ")
        );
    }
    let m = unique[0];
    if offset || want.is_some_and(|_| addr < m.base) {
        let Some(a) = addr.checked_add(m.base) else {
            return format!("{addr:#x} -> (address overflow)");
        };
        addr = a;
    }
    let hit = lookup_symbol(repo, arch, &m.name, addr).or_else(|| {
        if want.is_some() || identical {
            lookup_symbol_live(repo, arch, m, addr)
        } else {
            None
        }
    });
    let Some((name, sym_addr, src)) = hit else {
        return format!(
            "{addr:#x} -> {}+{:#x} (no symbol table entry)",
            m.name,
            addr.saturating_sub(m.base)
        );
    };
    let off = addr - sym_addr;
    if off > 0x20000 {
        return format!(
            "{addr:#x} -> {}+{:#x} (no symbol: nearest is {off:#x} away — outside any function)",
            m.name,
            addr.saturating_sub(m.base)
        );
    }
    format!(
        "{addr:#x} -> {}!{name}+{off:#x}{}",
        m.name,
        if src.is_empty() || src == "-" {
            String::new()
        } else {
            format!("  {src}")
        }
    )
}
pub fn resolve_in_module(repo: &Path, arch: &str, pc: &str, module: &str, rva: u64) -> String {
    let found = load_modules(repo, arch)
        .into_iter()
        .find(|m| m.name.eq_ignore_ascii_case(module));
    let Some(m) = found else {
        return format!("{pc} -> {module}+{rva:#x} (module not in the {arch} user index; run qemu/build-user-index.sh {arch}, then ./vibeos-cli sym)");
    };
    format!(
        "{pc} = {}",
        symbol_resolve(repo, arch, m.base.saturating_add(rva), Some(module), false)
    )
}
pub fn symbolicate_pc(repo: &Path, arch: &str, serial: &str) -> Option<String> {
    let mod_off = faulting_module_offset(serial);
    let pc =
        faulting_pc(serial).or_else(|| mod_off.as_ref().map(|(m, r)| format!("{m}+{r:#x}")))?;
    if let Some((module, rva)) = &mod_off {
        if !matches!(
            module.to_ascii_lowercase().as_str(),
            "ntoskrnl.exe" | "ntkrnlmp.exe"
        ) {
            return Some(resolve_in_module(repo, arch, &pc, module, *rva));
        }
    }
    let reloc = if mod_off.is_some() {
        None
    } else {
        kernel_relocation(arch, serial)
    };
    let pc = mod_off
        .as_ref()
        .map(|(_, r)| format!("{:#x}", PE_BASE + r))
        .unwrap_or(pc);
    let mp = serial.contains("ntkrnlmp.exe")
        || mod_off
            .as_ref()
            .is_some_and(|(m, _)| m.eq_ignore_ascii_case("ntkrnlmp.exe"));
    resolve_addr(repo, arch, &pc, mp, reloc.as_deref())
}
pub fn symbolicate_cpu_state(
    repo: &Path,
    arch: &str,
    state: &str,
    mp: bool,
    serial: &str,
) -> Vec<String> {
    let reloc = kernel_relocation(arch, serial);
    let mut out = Vec::new();
    for (cpu, pc) in cpu_pcs(state) {
        let p = format!("0x{pc}");
        out.push(format!(
            "CPU#{cpu} {}",
            resolve_addr(repo, arch, &p, mp, reloc.as_deref())
                .unwrap_or(format!("{p} -> (unresolved)"))
        ));
    }
    for (cpu, depth, ret) in cpu_frames(state) {
        let p = format!("0x{ret}");
        out.push(format!(
            "CPU#{cpu}   #{depth} {}",
            resolve_addr(repo, arch, &p, mp, reloc.as_deref())
                .unwrap_or(format!("{p} -> (unresolved)"))
        ));
    }
    out
}
pub fn dispatch(root: &Path, request: &Value) -> Result<Value> {
    let a = args(request);
    let op = string(request, "op")?;
    let serial = || string(a, "serial");
    let state = || string(a, "state");
    let default = known_flakes();
    let catalog = a.get("known_flakes").unwrap_or(&default);
    Ok(match op {
        "verdict_known_flakes" => default,
        "verdict_flake_note" => json!(flake_note(a.get("flake").filter(|v| !v.is_null()))),
        "verdict_match_known_flake" => json!(match_known_flake(
            serial()?,
            opt_string(a, "code"),
            string(a, "when")?,
            catalog
        )),
        "verdict_wedge_evidence" => wedge_evidence(serial()?),
        "verdict_kernel_asserts" => json!(kernel_asserts(serial()?)),
        "verdict_classify" => classify(
            serial()?,
            bool_arg(a, "reached"),
            opt_string(a, "fail_marker"),
            opt_string(a, "debugcon"),
            opt_string(a, "debugcon_raw").unwrap_or(""),
            catalog,
        ),
        "verdict_post_boot_check" => post_boot_check(
            serial()?,
            opt_string(a, "debugcon_raw").unwrap_or(""),
            catalog,
        ),
        "verdict_kernel_runtime_base" => json!(kernel_runtime_base(serial()?)),
        "verdict_kernel_relocation" => json!(kernel_relocation(string(a, "arch")?, serial()?)),
        "verdict_faulting_module_offset" => json!(faulting_module_offset(serial()?)),
        "verdict_faulting_pc" => json!(faulting_pc(serial()?)),
        "verdict_cpu_pcs" => json!(cpu_pcs(state()?)),
        "verdict_cpu_sps" => json!(cpu_sps(state()?)),
        "verdict_cpu_fps" => json!(cpu_fps(state()?)),
        "verdict_cpu_frames" => json!(cpu_frames(state()?)),
        "verdict_cpu_kernel_at_user_pc" => json!(cpu_kernel_at_user_pc(state()?)),
        "verdict_guest_interrupt_time" => json!(guest_interrupt_time(state()?)),
        "verdict_resolve_addr" => json!(resolve_addr(
            root,
            string(a, "arch")?,
            string(a, "pc")?,
            bool_arg(a, "mp"),
            opt_string(a, "reloc")
        )),
        "verdict_resolve_in_module" => json!(resolve_in_module(
            root,
            string(a, "arch")?,
            string(a, "pc")?,
            string(a, "module")?,
            a["rva"].as_u64().ok_or("missing RVA")?
        )),
        "verdict_symbolicate_pc" => json!(symbolicate_pc(root, string(a, "arch")?, serial()?)),
        "verdict_symbolicate_cpu_state" => json!(symbolicate_cpu_state(
            root,
            string(a, "arch")?,
            state()?,
            bool_arg(a, "mp"),
            opt_string(a, "serial").unwrap_or("")
        )),
        "verdict_symbol_load_modules" => json!(load_modules(root, string(a, "arch")?)),
        "verdict_symbol_resolve" => json!(symbol_resolve(
            root,
            string(a, "arch")?,
            a["addr"].as_u64().ok_or("missing address")?,
            opt_string(a, "want_module"),
            bool_arg(a, "offset")
        )),
        "verdict_symbol_lookup" => json!(lookup_symbol(
            root,
            string(a, "arch")?,
            string(a, "module")?,
            a["addr"].as_u64().ok_or("missing address")?
        )),
        "verdict_symbol_lookup_live" => json!(lookup_symbol_live(
            root,
            string(a, "arch")?,
            &serde_json::from_value(a["module"].clone())?,
            a["addr"].as_u64().ok_or("missing address")?
        )),
        "verdict_symbol_all_identical" => json!(all_identical(
            root,
            &serde_json::from_value::<Vec<Module>>(a["modules"].clone())?
        )),
        "verdict_symbol_index_stale" => {
            let index = root.join(format!("qemu/symbols-{}-user.tsv", string(a, "arch")?));
            let iso = root.join(format!("build-{}/livecd.iso", string(a, "arch")?));
            json!(if !index.is_file() {
                true
            } else if !iso.is_file() {
                false
            } else {
                fs::metadata(iso)?.modified()? > fs::metadata(index)?.modified()?
            })
        }
        _ => return Err(Error::new(format!("unknown verdict operation {op}"))),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fatal_precedence_and_post_marker_checks() {
        let c = known_flakes();
        let s = "BugCheck 0xF4\npanicked at 'oops', x.rs:7\n";
        assert_eq!(
            classify(s, false, Some("bad"), None, "", &c)["classification"],
            "RUST_PANIC"
        );
        assert_eq!(
            post_boot_check("marker\n", "PANIC:ke/wait.rs:99\n", &c)["panic_loc"],
            "ke/wait.rs:99"
        );
        assert!(post_boot_check("shell ready\n", "", &c).is_null());
    }
    #[test]
    fn no_broad_flake_matching() {
        let c = known_flakes();
        assert_eq!(
            classify(
                "*** BUGCHECK: 0x0000001E win32k.sys+0x65EE1",
                false,
                None,
                None,
                "",
                &c
            )["classification"],
            "KNOWN_FLAKE"
        );
        assert_eq!(
            classify(
                "*** BUGCHECK: 0x0000001E win32k.sys+0x65EE2",
                false,
                None,
                None,
                "",
                &c
            )["classification"],
            "BUGCHECK"
        );
    }
    #[test]
    fn fault_parameter_and_cpu_slicing() {
        let s="PC=0x1234\n*** BUGCHECK: 0x0000001E (0xc0000005, 0xffff800004001234, 0x0, 0x0)\n***   P2 = ntoskrnl.exe+0x1234\n";
        assert_eq!(faulting_pc(s).as_deref(), Some("0xffff800004001234"));
        assert_eq!(
            faulting_module_offset(s),
            Some(("ntoskrnl.exe".into(), 0x1234))
        );
        let state = "CPU#0\n PC=ffff800000001234\nCPU#1\n PC=0000000000001234 PSTATE=00000005\n";
        assert_eq!(cpu_pcs(state).len(), 2);
        assert_eq!(
            cpu_kernel_at_user_pc(state),
            vec![(1, "0000000000001234".into())]
        );
    }
    #[test]
    fn faulting_driver_uses_driver_index_and_module_relative_address() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("qemu")).unwrap();
        fs::write(
            root.path().join("qemu/modules-amd64.tsv"),
            "10000\t90000\tndis.sys\tdrivers/ndis.sys\tunique\n",
        )
        .unwrap();
        fs::write(
            root.path().join("qemu/symbols-amd64-user.tsv"),
            "ndis.sys\t10020\tDriverFault\tndis.rs:7\n",
        )
        .unwrap();
        let serial="*** BUGCHECK: 0x1E (0xc0000005, 0xfffff88000010020, 0x0, 0x0)\n***   P2 = ndis.sys+0x20\n";
        let resolved = symbolicate_pc(root.path(), "amd64", serial).unwrap();
        assert!(resolved.contains("ndis.sys!DriverFault+0x0"), "{resolved}");
        assert!(!resolved.contains("ntoskrnl"));
    }
    #[test]
    fn kernel_symbolication_passes_relocation_and_mp_to_existing_authority() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("qemu")).unwrap();
        fs::write(
            root.path().join("qemu/addr2sym.sh"),
            "printf '%s ' \"$@\"\n",
        )
        .unwrap();
        let serial="ntkrnlmp.exe\n[KRNLBASE] ntoskrnl runtime base=fffff80000400000\nPC: 0xfffff80000401234\n";
        let out = symbolicate_pc(root.path(), "amd64", serial).unwrap();
        assert!(
            out.contains("amd64 --reloc 0xfffff80000000000 --mp 0xfffff80000401234"),
            "{out}"
        );
    }
    #[test]
    fn explicit_null_flake_phase_does_not_broaden_the_catalog() {
        assert!(match_known_flake(
            "needle",
            None,
            "bugcheck",
            &json!([{"when":null,"any":["needle"]}])
        )
        .is_none());
    }
}
