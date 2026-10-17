//! Post-link flexible-array bounds-panic reader, ported from tools/scan_flex_array_panic.py.
use crate::binary::{BinaryContext, CheckOutput};
use rayon::prelude::*;
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

const ARCHES: &[&str] = &["i386", "amd64", "arm64"];
#[derive(Debug, Clone, PartialEq, Eq)]
struct Hit {
    symbol: String,
    context: Vec<String>,
}
#[derive(Default)]
struct Setup {
    length: Option<u64>,
    loc: Option<(String, i64)>,
    pushes: Vec<Option<u64>>,
    pending: Option<i64>,
    after_call: bool,
    page: Option<u64>,
}

#[derive(Clone, Default)]
struct Member {
    rdata: BTreeMap<i64, Vec<u8>>,
    syms: HashMap<String, (i64, u64)>,
}

#[derive(Clone, Default)]
struct ArchiveMembers {
    ordered: Vec<(String, Member)>,
}

impl ArchiveMembers {
    fn get(&self, name: &str) -> Option<&Member> {
        self.ordered.iter().find(|(n, _)| n == name).map(|(_, m)| m)
    }
    fn values(&self) -> impl Iterator<Item = &Member> {
        self.ordered.iter().map(|(_, m)| m)
    }
}

impl Member {
    fn section_of(&self, symbol: &str) -> Option<(&[u8], u64)> {
        if symbol == ".rdata" && self.rdata.len() == 1 {
            return self
                .rdata
                .values()
                .next()
                .map(|bytes| (bytes.as_slice(), 0));
        }
        let (section, value) = *self.syms.get(symbol)?;
        self.rdata
            .get(&section)
            .map(|bytes| (bytes.as_slice(), value))
    }
}

#[derive(Clone, Default)]
struct ImageRdata {
    base: u64,
    bytes: Vec<u8>,
}
fn rx(p: &str) -> &'static Regex {
    thread_local! {
        static PATTERNS: std::cell::RefCell<HashMap<String, &'static Regex>> =
            std::cell::RefCell::new(HashMap::new());
    }
    PATTERNS.with(|patterns| {
        *patterns
            .borrow_mut()
            .entry(p.to_owned())
            .or_insert_with(|| {
                Box::leak(Box::new(
                    crate::regexes::cached(p).expect("valid scanner regex"),
                ))
            })
    })
}

fn hex(s: &str) -> Option<u64> {
    u64::from_str_radix(s, 16).ok()
}
fn normalize_symbol(mut s: String, arch: &str, decorated: bool) -> String {
    if decorated && arch == "i386" && (s.starts_with('_') || s.starts_with('@')) {
        s.remove(0);
    }
    if let Some(i) = s.rfind('@') {
        if s[i + 1..].bytes().all(|b| b.is_ascii_digit()) && i + 1 < s.len() {
            s.truncate(i);
        }
    }
    if let Some(i) = s.rfind(".llvm.") {
        if s[i + 6..].bytes().all(|b| b.is_ascii_digit()) {
            s.truncate(i);
        }
    }
    if s.starts_with("_R") {
        let re = rx(r"Cs[0-9A-Za-z]+_");
        return re.replace_all(&s, "C").into_owned();
    }
    let h = rx(r"17h[0-9a-f]{16}E$");
    h.replace(&s, "E").into_owned()
}
fn parse_allowlist(path: &Path, arch: &str) -> BTreeSet<(String, String)> {
    let mut out = BTreeSet::new();
    let Ok(txt) = fs::read_to_string(path) else {
        return out;
    };
    for raw in txt.lines() {
        let l = raw.split('#').next().unwrap_or("").trim();
        if l.is_empty() {
            continue;
        }
        let mut line = l;
        if let Some(rest) = line.strip_prefix('@') {
            let Some((scope, r)) = rest.split_once(' ') else {
                continue;
            };
            if !scope.split(',').any(|a| a == arch) {
                continue;
            }
            line = r.trim();
        }
        let Some((c, s)) = line.split_once(':') else {
            continue;
        };
        out.insert((
            c.trim().to_owned(),
            normalize_symbol(s.trim().to_owned(), arch, false),
        ));
    }
    out
}
fn is_graded(g: &BTreeSet<(String, String)>, c: &str, s: &str) -> bool {
    g.contains(&(c.to_owned(), s.to_owned())) || g.contains(&("*".into(), s.to_owned()))
}
fn declared_corpus(root: &Path) -> Result<Vec<String>, String> {
    let dir = root.join("sdk/lib/rust");
    let Ok(entries) = fs::read_dir(&dir) else {
        return Ok(Vec::new());
    };
    let mut paths = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.join("Cargo.toml").is_file())
        .collect::<Vec<_>>();
    paths.sort();
    let name = rx(r#"(?m)^\s*name\s*=\s*"([^"]+)""#);
    let mut out = Vec::new();
    for p in paths {
        let text = fs::read_to_string(p.join("Cargo.toml")).map_err(|e| e.to_string())?;
        if !text.contains("staticlib") {
            continue;
        }
        if let Some(c) = name.captures(&text) {
            out.push(c[1].to_owned());
        }
    }
    Ok(out)
}
fn artifact_index(build: &Path) -> Result<BTreeMap<String, PathBuf>, String> {
    fn walk(dir: &Path, out: &mut BTreeMap<String, PathBuf>) -> Result<(), String> {
        let rd = match fs::read_dir(dir) {
            Ok(v) => v,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(format!("cannot walk {}: {e}", dir.display())),
        };
        let mut entries = rd.filter_map(Result::ok).collect::<Vec<_>>();
        entries.sort_by_key(|e| e.file_name());
        let mut dirs = Vec::new();
        for e in entries {
            let p = e.path();
            let ty = e.file_type().map_err(|e| e.to_string())?;
            if ty.is_dir() {
                if !e.file_name().to_string_lossy().ends_with("-target") {
                    dirs.push(p);
                }
            } else {
                let n = e.file_name().to_string_lossy().into_owned();
                if n.starts_with("lib") && n.ends_with(".a") {
                    out.entry(n).or_insert(p);
                }
            }
        }
        // os.walk yields a directory before its lexically sorted descendants.
        for child in dirs {
            walk(&child, out)?;
        }
        Ok(())
    }
    let mut out = BTreeMap::new();
    walk(build, &mut out)?;
    Ok(out)
}
fn declared_images(
    build: &Path,
    root: &Path,
    arch: &str,
) -> Result<Vec<(String, PathBuf)>, String> {
    let p = build.join("build.ninja");
    if !p.is_file() {
        return Ok(vec![]);
    }
    let t = fs::read_to_string(&p).map_err(|e| format!("cannot read {}: {e}", p.display()))?;
    let pattern = format!(
        r"sdk/lib/rust/target/([A-Za-z0-9_-]+)-{}-cargo-link/[A-Za-z0-9_.-]+/release/[A-Za-z0-9_.-]+\.(?:exe|dll|sys|efi)\b",
        regex::escape(arch)
    );
    let re = Regex::new(&pattern).unwrap();
    let shared=Regex::new(&format!(r"sdk/lib/rust/target/cell-link-v1-{}-[0-9]+/[A-Za-z0-9_.-]+/release/[A-Za-z0-9_.-]+\.(?:exe|dll|sys|efi)\b",regex::escape(arch))).unwrap();
    let crate_re = rx(r" -p ([A-Za-z0-9_-]+-cargo) ");
    let mut found: BTreeMap<String, String> = BTreeMap::new();
    for c in re.captures_iter(&t) {
        found.insert(c[0].to_string(), c[1].to_string());
    }
    let mut statements = Vec::new();
    let mut statement = String::new();
    for line in t.lines() {
        if line.starts_with("build ") && !statement.is_empty() {
            statements.push(std::mem::take(&mut statement));
        }
        statement.push_str(line);
        statement.push('\n');
    }
    if !statement.is_empty() {
        statements.push(statement);
    }
    for stmt in statements {
        if let Some(c) = crate_re.captures(&stmt) {
            for m in shared.find_iter(&stmt) {
                found.entry(m.as_str().into()).or_insert(c[1].into());
            }
        }
    }
    Ok(found
        .into_iter()
        .map(|(rel, c)| (c, root.join(rel)))
        .collect())
}

fn member_name(line: &str) -> Option<String> {
    rx(r"^(?:\S*\((?P<mem>[^)]+)\)|(?P<obj>\S+)):\s+file format ")
        .captures(line)
        .and_then(|c| c.name("mem").or_else(|| c.name("obj")))
        .map(|m| m.as_str().to_owned())
}

fn parse_dump_bytes(line: &str) -> Option<(u64, Vec<u8>)> {
    let c = rx(r"^\s*([0-9a-f]+)\s((?:[0-9a-f]{2,8}\s){1,4})\s").captures(line)?;
    let address = hex(&c[1])?;
    let text = c[2].split_whitespace().collect::<String>();
    if text.len() % 2 != 0 {
        return None;
    }
    let bytes = (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).ok())
        .collect::<Option<Vec<_>>>()?;
    Some((address, bytes))
}

fn members(ctx: &BinaryContext, path: &Path) -> Result<ArchiveMembers, String> {
    let path_s = path.to_string_lossy().into_owned();
    let headers = ctx.command(&ctx.objdump(), &["-h".into(), path_s.clone()])?;
    let mut out: HashMap<String, Member> = HashMap::new();
    let mut order: HashMap<String, Vec<i64>> = HashMap::new();
    let mut names = BTreeSet::new();
    let mut member_order = Vec::new();
    let mut current = String::new();
    for line in headers.lines() {
        if let Some(name) = member_name(line) {
            current = name.clone();
            if !out.contains_key(&name) {
                member_order.push(name.clone());
            }
            out.entry(name.clone()).or_default();
            order.entry(name).or_default();
            continue;
        }
        if let Some(c) = rx(r"^\s*(\d+)\s+(\S+)\s+([0-9a-f]+)\s").captures(line) {
            let section = &c[2];
            if !current.is_empty()
                && (section == ".rdata" || section.starts_with(".rdata$"))
                && hex(&c[3]).is_some_and(|size| size != 0)
            {
                order
                    .entry(current.clone())
                    .or_default()
                    .push(c[1].parse::<i64>().unwrap_or(0) + 1);
                names.insert(section.to_owned());
            }
        }
    }

    let symbols = ctx.command(&ctx.objdump(), &["-t".into(), path_s.clone()])?;
    current.clear();
    for line in symbols.lines() {
        if let Some(name) = member_name(line) {
            current = name;
            continue;
        }
        if let Some(c) =
            rx(r"^\[\s*\d+\]\(sec\s+(-?\d+)\).*\s0x([0-9a-f]+)\s+(\S+)\s*$").captures(line)
        {
            if let (Ok(section), Some(value)) = (c[1].parse::<i64>(), hex(&c[2])) {
                if let Some(m) = out.get_mut(&current) {
                    m.syms.entry(c[3].to_owned()).or_insert((section, value));
                }
            }
        }
    }

    let mut args = Vec::new();
    for name in &names {
        args.push("-j".to_owned());
        args.push(name.clone());
    }
    args.push(path_s);
    let contents = if names.is_empty() {
        String::new()
    } else {
        let mut full = vec!["-s".to_owned()];
        full.extend(args);
        ctx.command(&ctx.objdump(), &full)?
    };
    let mut section_order: HashMap<String, usize> = HashMap::new();
    let mut raw: HashMap<String, BTreeMap<i64, Vec<u8>>> = HashMap::new();
    current.clear();
    let mut active: Option<(String, i64)> = None;
    for line in contents.lines() {
        if let Some(name) = member_name(line) {
            current = name;
            active = None;
            continue;
        }
        if line.starts_with("Contents of section ") {
            let nth = section_order.entry(current.clone()).or_default();
            let section_number = order.get(&current).and_then(|xs| xs.get(*nth)).copied();
            *nth += 1;
            active = section_number.map(|n| (current.clone(), n));
            if let Some((member, section)) = &active {
                raw.entry(member.clone())
                    .or_default()
                    .entry(*section)
                    .or_default();
            }
            continue;
        }
        if let (Some((member, section)), Some((_, bytes))) =
            (active.as_ref(), parse_dump_bytes(line))
        {
            raw.entry(member.clone())
                .or_default()
                .entry(*section)
                .or_default()
                .extend(bytes);
        }
    }
    for (name, rdata) in raw {
        if let Some(m) = out.get_mut(&name) {
            m.rdata = rdata;
        }
    }
    Ok(ArchiveMembers {
        ordered: member_order
            .into_iter()
            .filter_map(|name| out.remove(&name).map(|m| (name, m)))
            .collect(),
    })
}

fn path_candidates(blobs: &[&[u8]], length: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for blob in blobs {
        let mut start = None;
        for i in 0..=blob.len() {
            let printable = i < blob.len() && (b' '..=b'~').contains(&blob[i]);
            if printable && start.is_none() {
                start = Some(i);
            }
            if !printable {
                if let Some(begin) = start.take() {
                    let run = &blob[begin..i];
                    if run.len() >= 4
                        && run.len() == length
                        && run.windows(5).any(|w| w == b"/src/")
                        && run.ends_with(b".rs")
                    {
                        let name = String::from_utf8_lossy(run).into_owned();
                        if seen.insert(name.clone()) {
                            out.push(name);
                        }
                    }
                }
            }
        }
    }
    out
}

fn read_u32(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}
fn read_u64(bytes: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_le_bytes(bytes.get(at..at + 8)?.try_into().ok()?))
}
fn resolve_location(
    member: &Member,
    reference: &(String, i64),
    ptr64: bool,
    others: &[Member],
) -> String {
    let mut selected = member;
    let mut found = selected.section_of(&reference.0);
    if found.is_none() && reference.0 != ".rdata" {
        for other in others {
            if let Some(data) = other.section_of(&reference.0) {
                selected = other;
                found = Some(data);
                break;
            }
        }
    }
    let Some((rdata, base)) = found else {
        return format!(
            "<location relocated against {}, defined in no member read>",
            reference.0
        );
    };
    let Some(off) = (base as i128 + reference.1 as i128)
        .try_into()
        .ok()
        .map(|v: usize| v)
    else {
        return String::new();
    };
    let (fptr, length, line, col) = if ptr64 {
        let (Some(f), Some(n), Some(l), Some(c)) = (
            read_u64(rdata, off),
            read_u64(rdata, off + 8),
            read_u32(rdata, off + 16),
            read_u32(rdata, off + 20),
        ) else {
            return String::new();
        };
        (f, n, l, c)
    } else {
        let (Some(f), Some(n), Some(l), Some(c)) = (
            read_u32(rdata, off),
            read_u32(rdata, off + 4),
            read_u32(rdata, off + 8),
            read_u32(rdata, off + 12),
        ) else {
            return String::new();
        };
        (f as u64, n as u64, l, c)
    };
    if !(1..=512).contains(&length) || !(1..1_000_000).contains(&line) {
        return String::new();
    }
    let len = length as usize;
    let add = fptr as i128;
    if add >= 0 && add + len as i128 <= rdata.len() as i128 {
        let name = &rdata[add as usize..add as usize + len];
        if name.windows(5).any(|w| w == b"/src/")
            && name.ends_with(b".rs")
            && name.is_ascii()
            && !name.contains(&0)
        {
            return format!("{}:{line}:{col}", String::from_utf8_lossy(name));
        }
    }
    let local_blobs = selected
        .rdata
        .values()
        .map(Vec::as_slice)
        .collect::<Vec<_>>();
    let mut candidates = path_candidates(&local_blobs, len);
    if candidates.is_empty() {
        let other_blobs = others
            .iter()
            .flat_map(|m| m.rdata.values().map(Vec::as_slice))
            .collect::<Vec<_>>();
        candidates = path_candidates(&other_blobs, len);
    }
    if candidates.len() == 1 {
        format!("{}:{line}:{col}", candidates[0])
    } else if !candidates.is_empty() {
        candidates.sort();
        format!(
            "AMBIGUOUS among {} — at :{line}:{col}",
            candidates.join(", ")
        )
    } else {
        format!("<unresolved file>:{line}:{col}")
    }
}

fn image_rdata(ctx: &BinaryContext, path: &Path) -> Result<ImageRdata, String> {
    let text = ctx.command(
        &ctx.objdump(),
        &[
            "-s".into(),
            "-j".into(),
            ".rdata".into(),
            path.to_string_lossy().into_owned(),
        ],
    )?;
    let mut image = ImageRdata::default();
    for line in text.lines() {
        if let Some((address, bytes)) = parse_dump_bytes(line) {
            if image.bytes.is_empty() {
                image.base = address;
            }
            image.bytes.extend(bytes);
        }
    }
    Ok(image)
}

fn resolve_va(image: &ImageRdata, va: u64, ptr64: bool) -> String {
    let Some(off) = va
        .checked_sub(image.base)
        .and_then(|n| usize::try_from(n).ok())
    else {
        return format!("<location at 0x{va:x}, outside .rdata>");
    };
    let (fptr, length, line, col) = if ptr64 {
        let (Some(f), Some(n), Some(l), Some(c)) = (
            read_u64(&image.bytes, off),
            read_u64(&image.bytes, off + 8),
            read_u32(&image.bytes, off + 16),
            read_u32(&image.bytes, off + 20),
        ) else {
            return format!("<location at 0x{va:x}, outside .rdata>");
        };
        (f, n, l, c)
    } else {
        let (Some(f), Some(n), Some(l), Some(c)) = (
            read_u32(&image.bytes, off),
            read_u32(&image.bytes, off + 4),
            read_u32(&image.bytes, off + 8),
            read_u32(&image.bytes, off + 12),
        ) else {
            return format!("<location at 0x{va:x}, outside .rdata>");
        };
        (f as u64, n as u64, l, c)
    };
    let foff = fptr
        .checked_sub(image.base)
        .and_then(|n| usize::try_from(n).ok());
    if (1..=512).contains(&length) {
        if let Some(foff) = foff {
            if let Some(name) = image.bytes.get(foff..foff + length as usize) {
                if name.is_ascii() {
                    return format!("{}:{line}:{col}", String::from_utf8_lossy(name));
                }
            }
        }
    }
    format!("<unresolved file>:{line}:{col}")
}
fn parse_num(s: &str) -> Option<u64> {
    let s = s.trim();
    if let Some(h) = s.strip_prefix("$0x").or_else(|| s.strip_prefix("#0x")) {
        hex(h)
    } else if let Some(h) = s.strip_prefix("$0X") {
        hex(h)
    } else {
        None
    }
}
fn feed_i386(st: &mut Setup, line: &str, linked: bool) -> &'static str {
    if rx(r"\bpushl\s").is_match(line) {
        let v = rx(r"pushl\s+\$0x([0-9a-f]+)\s*(?:#.*)?$")
            .captures(line.trim_end())
            .and_then(|c| hex(&c[1]));
        st.pushes.push(v);
        if st.pushes.len() > 3 {
            st.pushes.remove(0);
        }
        return "";
    }
    if line.contains("panic_bounds_check") {
        if st.pushes.len() == 3 {
            st.length = st.pushes[1];
            st.loc = st.pushes[0].map(|x| (if linked { "@va" } else { ".rdata" }.into(), x as i64));
        }
        return "call";
    }
    if rx(r"pushl|calll|IMAGE_REL|movl|leal").is_match(line) {
        ""
    } else if line.trim().is_empty() {
        ""
    } else {
        "reset"
    }
}
fn feed_64(st: &mut Setup, line: &str, arch: &str, linked: bool) -> &'static str {
    if let Some(c) = rx(r"^\s+[0-9a-f]+:\s+(IMAGE_REL_\S+)\s+(\S+)\s*$").captures(line) {
        if st.after_call {
            return if c[2].contains("panic_bounds_check") {
                "call"
            } else {
                "reset"
            };
        }
        if st.pending.is_some() && (arch == "amd64" || c[1].ends_with("PAGEOFFSET_12A")) {
            st.loc = Some((c[2].into(), st.pending.unwrap()));
            st.pending = None;
        }
        return "";
    }
    if st.after_call {
        return "reset";
    }
    let parsed = if arch == "amd64" {
        rx(r"^\s*[0-9a-f]+:\s+(?:[0-9a-f]{2} )+\s*(\S+)\s*([^#]*?)\s*(?:#.*)?$").captures(line)
    } else {
        rx(r"^\s*[0-9a-f]+:\s+[0-9a-f]{8}\s+(\S+)\s*([^/]*?)\s*(?://.*)?$").captures(line)
    };
    let Some(c) = parsed else {
        return if line.trim().is_empty() { "" } else { "reset" };
    };
    let mn = &c[1];
    let ops = c[2].split(',').map(str::trim).collect::<Vec<_>>();
    let carry = if arch == "amd64" {
        [
            "movl", "movq", "movb", "movw", "leaq", "leal", "xorl", "xorq", "callq",
        ]
        .contains(&mn)
    } else {
        ["mov", "movz", "movk", "movn", "orr", "adrp", "add", "bl"].contains(&mn)
    };
    if !carry {
        return "reset";
    }
    if mn == "callq" || mn == "bl" {
        if line.contains("panic_bounds_check") {
            return "call";
        }
        st.after_call = true;
        return "";
    }
    if arch == "amd64" {
        if ops.len() < 2 {
            return "";
        }
        let dst = *ops.last().unwrap();
        let src = ops[0];
        if ["%edx", "%rdx", "%dx", "%dl"].contains(&dst) {
            st.length = if mn.starts_with("mov") {
                parse_num(src)
            } else {
                None
            };
        } else if mn.starts_with("lea") && dst == "%r8" && linked && line.contains("# 0x") {
            if let Some(v) = line
                .rsplit("# 0x")
                .next()
                .and_then(|tail| tail.split_whitespace().next())
                .and_then(hex)
            {
                st.loc = Some(("@va".into(), v as i64));
            }
        } else if mn.starts_with("lea") && dst == "%r8" {
            if let Some(i) = src.find("(%rip)") {
                let d = &src[..i];
                st.pending = if d.is_empty() {
                    Some(0)
                } else {
                    d.strip_prefix("0x")
                        .and_then(hex)
                        .map(|x| x as i64)
                        .or_else(|| d.strip_prefix("-0x").and_then(hex).map(|x| -(x as i64)))
                };
            }
        }
    } else if let Some(dst) = ops.first() {
        if ["w1", "x1"].contains(dst) {
            let rhs = ops[1..].join(", ");
            st.length = if ["mov", "movz", "orr"].contains(&mn) {
                rx(r"(?:[wx]zr, )?#0x([0-9a-f]+)$")
                    .captures(&rhs)
                    .and_then(|c| hex(&c[1]))
            } else {
                None
            };
        } else if mn == "adrp" && *dst == "x2" && linked {
            st.page = ops
                .get(1)
                .and_then(|s| rx(r"0x([0-9a-f]+)").captures(s))
                .and_then(|c| hex(&c[1]));
        } else if mn == "adrp" && *dst == "x2" {
            st.page = ops
                .get(1)
                .and_then(|s| rx(r"0x([0-9a-f]+)").captures(s))
                .and_then(|c| hex(&c[1]));
        } else if mn == "add" && *dst == "x2" {
            st.pending = ops
                .last()
                .and_then(|s| s.strip_prefix("#0x"))
                .and_then(hex)
                .map(|v| v as i64);
            if linked {
                if let (Some(page), Some(offset)) = (st.page, st.pending) {
                    st.loc = Some(("@va".into(), (page as i64).wrapping_add(offset)));
                }
            }
        }
    }
    ""
}
fn scan_listing_inner(
    listing: &str,
    arch: &str,
    linked: bool,
    explain: bool,
    members: &ArchiveMembers,
    image: &ImageRdata,
) -> Vec<Hit> {
    let sym_re = rx(r"^[0-9a-f]+ <(.+)>:$");
    let mut sym = "?".to_string();
    let mut st = Setup::default();
    let mut current_member = Member::default();
    let others = members.values().cloned().collect::<Vec<_>>();
    let mut window = Vec::<String>::new();
    let mut hits = Vec::new();
    for line in listing.lines() {
        if let Some(name) = member_name(line) {
            sym = "?".into();
            st = Setup::default();
            current_member = members.get(&name).cloned().unwrap_or_default();
            window.clear();
            continue;
        }
        if let Some(c) = sym_re.captures(line) {
            sym = c[1].into();
            st = Setup::default();
            window.clear();
            continue;
        }
        if explain {
            window.push(line.trim_end().to_owned());
            if window.len() > 14 {
                window.remove(0);
            }
        }
        let verdict = if arch == "i386" {
            feed_i386(&mut st, line, linked)
        } else {
            feed_64(&mut st, line, arch, linked)
        };
        if verdict == "call" {
            if st.length == Some(1) {
                let mut context = Vec::new();
                if explain {
                    if let Some((kind, address)) = &st.loc {
                        let where_ = if kind == "@va" {
                            resolve_va(image, *address as u64, arch != "i386")
                        } else {
                            resolve_location(
                                &current_member,
                                &(kind.clone(), *address),
                                arch != "i386",
                                &others,
                            )
                        };
                        if !where_.is_empty() {
                            context.push(format!("    -> {where_}   (from the panic Location)"));
                        }
                    }
                    context.extend(window.iter().cloned());
                }
                hits.push(Hit {
                    symbol: sym.clone(),
                    context,
                });
            }
            st = Setup::default();
        } else if verdict == "reset" {
            st = Setup::default();
        }
    }
    hits
}
fn scan_listing(listing: &str, arch: &str) -> Vec<Hit> {
    scan_listing_inner(
        listing,
        arch,
        false,
        false,
        &ArchiveMembers::default(),
        &ImageRdata::default(),
    )
}
fn objdump(ctx: &BinaryContext, args: &[String]) -> Result<String, String> {
    let mut a = vec!["-dr".to_string()];
    a.extend_from_slice(args);
    ctx.command(&ctx.objdump(), &a)
}
fn scan_file(ctx: &BinaryContext, p: &Path, explain: bool) -> Result<Vec<Hit>, String> {
    let out = objdump(ctx, &[p.to_string_lossy().into_owned()])
        .map_err(|e| format!("objdump failed on {}: {e}", p.display()))?;
    let linked = p.extension().is_none_or(|e| !e.eq_ignore_ascii_case("a"));
    let archive = if explain && !linked {
        members(ctx, p)?
    } else {
        ArchiveMembers::default()
    };
    let image = if explain && linked {
        image_rdata(ctx, p)?
    } else {
        ImageRdata::default()
    };
    Ok(scan_listing_inner(
        &out, &ctx.arch, linked, explain, &archive, &image,
    ))
}
fn hard_fail(s: String) -> Result<CheckOutput, String> {
    Err(s)
}
fn run_internal(ctx: &BinaryContext, explain: Option<&str>) -> Result<CheckOutput, String> {
    if !ARCHES.contains(&ctx.arch.as_str()) {
        return hard_fail(format!(
            "invalid --arch {} (expected i386, amd64, or arm64)",
            ctx.arch
        ));
    }
    let tool = ctx.objdump();
    if crate::binary::executable(&tool).is_none() {
        return hard_fail(format!("MISSING TOOL: {tool}"));
    }
    if !ctx.build_dir.is_dir() {
        return hard_fail(format!(
            "MISSING BUILD: {}\nfix: ./vibeos-cli build --arch {}",
            ctx.build_dir.display(),
            ctx.arch
        ));
    }
    let corpus = declared_corpus(&ctx.root)?;
    let images = declared_images(&ctx.build_dir, &ctx.root, &ctx.arch)?;
    let index = artifact_index(&ctx.build_dir)?;
    let mut missing = Vec::new();
    let mut work = Vec::<(String, PathBuf)>::new();
    for crate_name in &corpus {
        let f = format!("lib{}.a", crate_name.replace('-', "_"));
        if let Some(p) = index.get(&f) {
            work.push((crate_name.clone(), p.clone()))
        } else {
            missing.push(crate_name.clone());
        }
    }
    for (c, p) in &images {
        if p.is_file() {
            work.push((c.clone(), p.clone()))
        } else {
            missing.push(format!(
                "{}({})",
                c,
                p.file_name().unwrap_or_default().to_string_lossy()
            ));
        }
    }
    let allow = parse_allowlist(&ctx.root.join("tools/flex_array_allowlist.txt"), &ctx.arch);
    // Match the legacy scanner's bounded parallel sweep; BinaryContext memoizes each
    // objdump extraction by its complete tool/input identity.
    let scanned = rayon::ThreadPoolBuilder::new()
        .num_threads(8)
        .build()
        .map_err(|e| e.to_string())?
        .install(|| {
            work.par_iter()
                .map(|(c, p)| {
                    scan_file(ctx, p, explain.is_some()).map(|hits| {
                        hits.into_iter()
                            .map(|h| {
                                (
                                    c.clone(),
                                    normalize_symbol(h.symbol, &ctx.arch, true),
                                    h.context,
                                )
                            })
                            .collect::<Vec<_>>()
                    })
                })
                .collect::<Result<Vec<_>, _>>()
        })?;
    let rows = scanned.into_iter().flatten().collect::<Vec<_>>();
    let seen = rows
        .iter()
        .map(|(c, s, _)| (c.clone(), s.clone()))
        .collect::<BTreeSet<_>>();
    let mut stdout = String::new();
    if let Some(filter) = explain {
        for (c, s, context) in &rows {
            if (c.contains(filter) || s.contains(filter)) && !context.is_empty() {
                stdout.push_str(&format!("\n=== {c}: {s}\n"));
                for line in context {
                    stdout.push_str(&format!("    {line}\n"));
                }
            }
        }
    }
    let ungraded = seen
        .iter()
        .filter(|(c, s)| !is_graded(&allow, c, s))
        .cloned()
        .collect::<Vec<_>>();
    for (c, s) in &ungraded {
        stdout.push_str(&format!(
            "{c}: {s}: bounds check on a 1-element array (bindgen flexible array?)\n"
        ));
    }
    for (c, s) in seen.iter().filter(|(c, s)| is_graded(&allow, c, s)) {
        stdout.push_str(&format!(
            "{c}: {s}: GRADED OK (allowlisted) — a correct check; see its entry\n"
        ));
    }
    let symbols = seen
        .iter()
        .map(|(_, s)| s.as_str())
        .collect::<BTreeSet<_>>();
    let stale = allow
        .iter()
        .filter(|(c, s)| {
            if c == "*" {
                !symbols.contains(s.as_str())
            } else {
                !seen.contains(&(c.clone(), s.clone()))
            }
        })
        .cloned()
        .collect::<Vec<_>>();
    let mut stderr = String::new();
    if !stale.is_empty() {
        stderr.push_str(&format!("STALE ALLOWLIST ENTRIES on {} (no longer found — remove them, scope them to the arches that still have the site, or find out why the reader went blind): {}\n",ctx.arch,stale.iter().map(|(c,s)|format!("{c}: {s}")).collect::<Vec<_>>().join(", ")));
    }
    let modules = seen.iter().map(|(c, _)| c).collect::<BTreeSet<_>>().len();
    stderr.push_str(&format!("\n{}: {} site(s) in {} module(s); corpus {} staticlibs + {} cargo-linked images declared, {} not built\n",ctx.arch,seen.len(),modules,corpus.len(),images.len(),missing.len()));
    if !missing.is_empty() {
        missing.sort();
        stderr.push_str(&format!("NOT BUILT (no coverage): {}\n", missing.join(" ")));
    }
    // This scanner is wired into the native post-link gate, whose invocation is the
    // equivalent of the legacy CLI's --lint mode.
    let rc = if !ungraded.is_empty() || !stale.is_empty() {
        1
    } else {
        0
    };
    Ok(CheckOutput {
        stdout,
        stderr,
        returncode: rc,
    })
}

pub fn run(ctx: &BinaryContext) -> Result<CheckOutput, String> {
    run_internal(ctx, None)
}

pub fn explain(ctx: &BinaryContext, substring: &str) -> Result<CheckOutput, String> {
    run_internal(ctx, Some(substring))
}

#[cfg(test)]
mod tests {
    use super::*;
    const FLEX:&str="\n00001000 <_Victim@8>:\n    1000: 68 70 05 00 00               \tpushl\t$0x570                  # imm = 0x570\n\t\t\t00001001:  IMAGE_REL_I386_DIR32\t.rdata\n    1005: 6a 01                        \tpushl\t$0x1\n    1007: 6a 01                        \tpushl\t$0x1\n    1009: e8 00 00 00 00               \tcalll\t0x100e\n\t\t\t0000100a:  IMAGE_REL_I386_REL32\t__ZN4core9panicking18panic_bounds_checkE\n";
    #[test]
    fn i386_middle_push_is_len() {
        assert_eq!(scan_listing(FLEX, "i386")[0].symbol, "_Victim@8");
    }
    #[test]
    fn unrelated_instruction_clears_setup() {
        let x = FLEX.replace(
            "1009: e8",
            "1008: 90                        \tnop\n    1009: e8",
        );
        assert!(scan_listing(&x, "i386").is_empty());
    }
    #[test]
    fn symbol_boundary_clears_setup() {
        let x="00001000 <Pusher>:\n 1000: 6a 01 pushl $0x1\n00002000 <Other>:\n 2000: e8 call\n relocation panic_bounds_check\n";
        assert!(scan_listing(x, "i386").is_empty());
    }
    #[test]
    fn register_arches_require_literal_length() {
        let x="0000000000001000 <Victim>:\n 1000: ba 01 00 00 00 \tmovl\t$0x1, %edx\n 100c: e8 00 00 00 00 \tcallq\t0x1011\n 100d: IMAGE_REL_AMD64_REL32 _ZN4core9panicking18panic_bounds_checkE\n";
        assert_eq!(scan_listing(x, "amd64").len(), 1);
        assert!(scan_listing(&x.replace("$0x1, %edx", "%eax, %edx"), "amd64").is_empty());
    }
    #[test]
    fn arm64_register_argument_uses_w1_and_branch_clears_setup() {
        let listing = "0000000000000000 <Victim>:\n 5f8: 52800021 \tmov\tw1, #0x1                // =1\n 5fc: 94000000 \tbl\t0x5fc <Victim+0x5fc>\n\t\t00000000000005fc:  IMAGE_REL_ARM64_BRANCH26\t_ZN4core9panicking18panic_bounds_checkE\n";
        assert_eq!(scan_listing(listing, "arm64").len(), 1);
        let blocked = listing.replace(
            "5fc: 94000000",
            "5fa: eb01001f \tcmp\tx0, x1\n 5fc: 94000000",
        );
        assert!(scan_listing(&blocked, "arm64").is_empty());
    }
    #[test]
    fn i386_requires_middle_literal_and_only_panic_call() {
        let wrong_len = FLEX.replace("pushl\t$0x1\n    1007", "pushl\t$0x2\n    1007");
        assert!(scan_listing(&wrong_len, "i386").is_empty());
        let other = FLEX.replace("panic_bounds_check", "other_helper");
        assert!(scan_listing(&other, "i386").is_empty());
    }
    #[test]
    fn v0_symbol_drops_crate_disambiguator_and_llvm_suffix() {
        assert_eq!(
            normalize_symbol(
                "__RNvNtCs6vQPevndiDL_7ex_rust7sysinfo26QSI_SystemNumaProcessorMap.llvm.42".into(),
                "i386",
                true
            ),
            "_RNvNtC7ex_rust7sysinfo26QSI_SystemNumaProcessorMap"
        );
    }
    #[test]
    fn cargo_image_corpus_comes_from_ninja_and_is_arch_scoped() {
        let d = std::env::temp_dir().join(format!("flex-ninja-{}", std::process::id()));
        fs::create_dir_all(&d).unwrap();
        fs::write(d.join("build.ninja"), "build sdk/lib/rust/target/smss-cargo-amd64-cargo-link/x64/release/smss.exe: X\nbuild sdk/lib/rust/target/smss-cargo-i386-cargo-link/x86/release/smss.exe: X\n").unwrap();
        let got = declared_images(&d, &d, "amd64").unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].0, "smss-cargo");
        assert!(got[0]
            .1
            .ends_with("smss-cargo-amd64-cargo-link/x64/release/smss.exe"));
        let _ = fs::remove_dir_all(d);
    }
    #[test]
    fn wildcard_allowlist_grades_the_function_in_each_image() {
        let graded = BTreeSet::from([("*".into(), "_ZN7network4pure11format_ipv6E".into())]);
        assert!(is_graded(
            &graded,
            "ntdll-cargo",
            "_ZN7network4pure11format_ipv6E"
        ));
        assert!(!is_graded(
            &graded,
            "ntdll-cargo",
            "_ZN7network4pure5otherE"
        ));
    }
    #[test]
    fn arch_normalization_strips_hash_and_decoration() {
        assert_eq!(
            normalize_symbol("__ZN17foo3bar17hb6081ba0f0486feeE@24".into(), "i386", true),
            "_ZN17foo3barE"
        );
        assert_eq!(
            normalize_symbol("_Private".into(), "amd64", true),
            "_Private"
        );
        assert_eq!(
            normalize_symbol("@ftGetFontUnicodeRanges@8".into(), "i386", true),
            "ftGetFontUnicodeRanges"
        );
        // Preserve the Python order: @N is stripped before .llvm.N, so this unusual
        // combined suffix is not silently normalized as a different spelling.
        assert_eq!(
            normalize_symbol("__ZN4foo17habcdef0123456789E@8.llvm.2".into(), "i386", true),
            "_ZN4foo17habcdef0123456789E@8"
        );
    }
    #[test]
    fn allowlist_scope_and_normalized_names() {
        let p = std::env::temp_dir().join(format!("flex-allow-{}", std::process::id()));
        fs::write(
            &p,
            "everywhere-rust: Everywhere # x\n@amd64,arm64 wide-rust: _ZN4wide1fE\n",
        )
        .unwrap();
        assert_eq!(parse_allowlist(&p, "i386").len(), 1);
        assert_eq!(parse_allowlist(&p, "arm64").len(), 2);
        let _ = fs::remove_file(p);
    }
    #[test]
    fn corpus_prunes_target_dirs_and_first_hit() {
        let d = std::env::temp_dir().join(format!("flex-corpus-{}", std::process::id()));
        fs::create_dir_all(d.join("x")).unwrap();
        fs::create_dir_all(d.join("x-target/deep")).unwrap();
        fs::write(d.join("x/libx.a"), []).unwrap();
        fs::write(d.join("x-target/deep/libx.a"), []).unwrap();
        let got = artifact_index(&d).unwrap();
        assert_eq!(got["libx.a"], d.join("x/libx.a"));
        let _ = fs::remove_dir_all(d);
    }
    #[test]
    fn explain_collects_window_only_when_requested_and_keeps_relocation_context() {
        let listing = FLEX.replace(
            "    1005: 6a 01",
            "    1004: 8d 3d 00 00 00 00            \tleal\t0x0, %edi\n\t\t\t00001005:  IMAGE_REL_I386_DIR32\t_ReparseBuffer_PathBuffer\n    1005: 6a 01",
        );
        let quiet = scan_listing(&listing, "i386");
        assert!(quiet[0].context.is_empty());
        let explained = scan_listing_inner(
            &listing,
            "i386",
            false,
            true,
            &ArchiveMembers::default(),
            &ImageRdata::default(),
        );
        assert!(explained[0]
            .context
            .iter()
            .any(|line| line.contains("_ReparseBuffer_PathBuffer")));
        let neighbor = format!(
            "00000900 <Neighbour>:\n\t\t\t00000901: IMAGE_REL_I386_DIR32 _NOT_THIS_ONE\n{}",
            FLEX.trim_start()
        );
        let got = scan_listing_inner(
            &neighbor,
            "i386",
            false,
            true,
            &ArchiveMembers::default(),
            &ImageRdata::default(),
        );
        assert!(!got[0]
            .context
            .iter()
            .any(|line| line.contains("_NOT_THIS_ONE")));
    }
    #[test]
    fn linked_location_resolves_filename_line_and_column() {
        let base = 0x14002f000u64;
        let name = b"win32k-rust/src/freetype.rs";
        let mut bytes = vec![0u8; 0x100];
        bytes[0x80..0x80 + name.len()].copy_from_slice(name);
        bytes[0x20..0x28].copy_from_slice(&(base + 0x80).to_le_bytes());
        bytes[0x28..0x30].copy_from_slice(&(name.len() as u64).to_le_bytes());
        bytes[0x30..0x34].copy_from_slice(&4694u32.to_le_bytes());
        bytes[0x34..0x38].copy_from_slice(&49u32.to_le_bytes());
        let image = ImageRdata { base, bytes };
        assert_eq!(
            resolve_va(&image, base + 0x20, true),
            "win32k-rust/src/freetype.rs:4694:49"
        );
        assert!(resolve_va(&image, base + 0x1000, true).contains("outside .rdata"));
    }
    #[test]
    fn archive_location_uses_its_member_section_and_relocation_addend() {
        let name = b"ks-rust/src/filter.rs";
        let mut rdata = vec![0u8; 0x80];
        rdata[0x40..0x40 + name.len()].copy_from_slice(name);
        rdata[0x10..0x14].copy_from_slice(&0x40u32.to_le_bytes());
        rdata[0x14..0x18].copy_from_slice(&(name.len() as u32).to_le_bytes());
        rdata[0x18..0x1c].copy_from_slice(&71u32.to_le_bytes());
        rdata[0x1c..0x20].copy_from_slice(&9u32.to_le_bytes());
        let member = Member {
            rdata: BTreeMap::from([(2, rdata)]),
            syms: HashMap::new(),
        };
        assert_eq!(
            resolve_location(&member, &(".rdata".into(), 0x10), false, &[]),
            "ks-rust/src/filter.rs:71:9"
        );
    }
    #[test]
    fn archive_location_declines_ambiguous_equal_length_paths() {
        let first = b"ks-rust/src/filter.rs";
        let second = b"ks-rust/src/swenum.rs";
        assert_eq!(first.len(), second.len());
        let mut loc = vec![0u8; 0x40];
        loc[4..8].copy_from_slice(&(first.len() as u32).to_le_bytes());
        loc[8..12].copy_from_slice(&71u32.to_le_bytes());
        loc[12..16].copy_from_slice(&9u32.to_le_bytes());
        let mut paths_a = vec![0u8; 0x40];
        paths_a[..first.len()].copy_from_slice(first);
        let mut paths_b = vec![0u8; 0x40];
        paths_b[..second.len()].copy_from_slice(second);
        let member = Member {
            rdata: BTreeMap::from([(2, loc), (3, paths_a), (4, paths_b)]),
            syms: HashMap::from([("location".into(), (2, 0))]),
        };
        let got = resolve_location(&member, &("location".into(), 0), false, &[]);
        assert!(got.contains("AMBIGUOUS among"), "{got}");
        assert!(got.contains("ks-rust/src/filter.rs"), "{got}");
        assert!(got.contains("ks-rust/src/swenum.rs"), "{got}");
    }
    #[test]
    fn archive_location_follows_thinlto_symbol_into_another_member() {
        let name = b"ks-rust/src/filter.rs";
        let mut rdata = vec![0u8; 0x80];
        rdata[0x40..0x40 + name.len()].copy_from_slice(name);
        rdata[0x10..0x14].copy_from_slice(&0x40u32.to_le_bytes());
        rdata[0x14..0x18].copy_from_slice(&(name.len() as u32).to_le_bytes());
        rdata[0x18..0x1c].copy_from_slice(&71u32.to_le_bytes());
        rdata[0x1c..0x20].copy_from_slice(&9u32.to_le_bytes());
        let owner = Member::default();
        let defining = Member {
            rdata: BTreeMap::from([(2, rdata)]),
            syms: HashMap::from([("anon.location".into(), (2, 0x10))]),
        };
        assert_eq!(
            resolve_location(&owner, &("anon.location".into(), 0), false, &[defining]),
            "ks-rust/src/filter.rs:71:9"
        );
    }
    #[test]
    fn absent_workspace_and_allowlist_are_empty_not_errors() {
        let d = std::env::temp_dir().join(format!("flex-empty-{}", std::process::id()));
        fs::create_dir_all(&d).unwrap();
        assert!(declared_corpus(&d).unwrap().is_empty());
        assert!(parse_allowlist(&d.join("missing.txt"), "arm64").is_empty());
        let _ = fs::remove_dir_all(d);
    }
}
