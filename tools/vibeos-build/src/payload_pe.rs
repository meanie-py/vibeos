//! PE payload inspection and tree export checks.
use crate::{Error, Result};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

fn u16at(b: &[u8], p: usize) -> Result<u16> {
    let x = b.get(p..p + 2).ok_or("truncated PE image")?;
    Ok(u16::from_le_bytes([x[0], x[1]]))
}
fn u32at(b: &[u8], p: usize) -> Result<u32> {
    let x = b.get(p..p + 4).ok_or("truncated PE image")?;
    Ok(u32::from_le_bytes([x[0], x[1], x[2], x[3]]))
}
fn u64at(b: &[u8], p: usize) -> Result<u64> {
    let x = b.get(p..p + 8).ok_or("truncated PE image")?;
    Ok(u64::from_le_bytes(x.try_into().unwrap()))
}
fn header(b: &[u8]) -> Result<usize> {
    if b.get(..2) != Some(b"MZ") {
        return Err("not an MZ image".into());
    }
    let p = u32at(b, 0x3c)? as usize;
    if b.get(p..p + 4) != Some(b"PE\0\0") {
        return Err("no PE signature".into());
    }
    Ok(p)
}
pub fn machine_arch(path: &Path) -> Result<&'static str> {
    let b = fs::read(path)?;
    let p = header(&b)?;
    match u16at(&b, p + 4)? {
        0x14c => Ok("i386"),
        0x8664 => Ok("x86_64"),
        0xaa64 => Ok("arm64"),
        n => Err(Error::new(format!("unknown machine {n:#x}"))),
    }
}
pub fn parse(path: &Path) -> Result<(u16, BTreeMap<String, Vec<String>>)> {
    let b = fs::read(path)?;
    let p = header(&b)?;
    let n = u16at(&b, p + 6)? as usize;
    let optsize = u16at(&b, p + 20)? as usize;
    let opt = p + 24;
    let magic = u16at(&b, opt)?;
    let wide = match magic {
        0x10b => false,
        0x20b => true,
        _ => {
            return Err(Error::new(format!(
                "unknown optional-header magic {magic:#x}"
            )))
        }
    };
    let subsystem = u16at(&b, opt + 68)?;
    let imp = u32at(&b, opt + if wide { 112 } else { 96 } + 8)?;
    let mut sections = Vec::new();
    for i in 0..n {
        let s = opt + optsize + 40 * i + 8;
        sections.push((
            u32at(&b, s + 4)?,
            u32at(&b, s)?.max(u32at(&b, s + 8)?),
            u32at(&b, s + 12)?,
        ));
    }
    let off = |r: u32| -> Result<usize> {
        for &(va, size, raw) in &sections {
            if r >= va && r - va < size {
                return Ok((raw as usize) + (r - va) as usize);
            }
        }
        Err(Error::new(format!("RVA {r:#x} is in no section")))
    };
    let cstr = |r: u32| -> Result<String> {
        let o = off(r)?;
        let rest = b.get(o..).ok_or("truncated PE string")?;
        let end = rest
            .iter()
            .position(|&c| c == 0)
            .ok_or("unterminated PE string")?;
        Ok(rest[..end].iter().map(|&c| char::from(c)).collect())
    };
    let mut out = BTreeMap::new();
    if imp != 0 {
        let mut d = off(imp)?;
        loop {
            let oft = u32at(&b, d)?;
            let name = u32at(&b, d + 12)?;
            let ft = u32at(&b, d + 16)?;
            if oft == 0 && name == 0 && ft == 0 {
                break;
            }
            let mut t = off(if oft != 0 { oft } else { ft })?;
            let mut names = Vec::new();
            loop {
                let v = if wide {
                    u64at(&b, t)?
                } else {
                    u32at(&b, t)? as u64
                };
                if v == 0 {
                    break;
                }
                let bit = if wide { 1u64 << 63 } else { 1u64 << 31 };
                names.push(if v & bit != 0 {
                    format!("#{}", v & 0xffff)
                } else {
                    cstr(
                        ((v & 0x7fffffff) as u32)
                            .checked_add(2)
                            .ok_or("invalid import RVA")?,
                    )?
                });
                t += if wide { 8 } else { 4 };
            }
            out.insert(cstr(name)?, names);
            d += 20;
        }
    }
    Ok((subsystem, out))
}
pub fn export_version(repo: &Path) -> Result<u32> {
    let text = crate::read(&repo.join("sdk/cmake/config.cmake"))?;
    let re = regex::Regex::new(r#"set\(DLL_EXPORT_VERSION\s+\"(0x[0-9A-Fa-f]+)\""#)?;
    let v = re
        .captures(&text)
        .ok_or("DLL_EXPORT_VERSION default not found in sdk/cmake/config.cmake")?[1]
        .to_owned();
    u32::from_str_radix(&v[2..], 16).map_err(|e| Error::new(e.to_string()))
}
fn walk(dir: &Path, index: &mut BTreeMap<String, Vec<PathBuf>>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            walk(&path, index)?
        } else if path.extension().and_then(|x| x.to_str()) == Some("spec") {
            if let Some(stem) = path.file_stem().and_then(|x| x.to_str()) {
                index.entry(stem.to_lowercase()).or_default().push(path)
            }
        }
    }
    Ok(())
}
pub fn spec_index(repo: &Path) -> Result<BTreeMap<String, Vec<PathBuf>>> {
    let mut out = BTreeMap::new();
    for top in ["dll", "win32ss", "base"] {
        walk(&repo.join(top), &mut out)?
    }
    Ok(out)
}
pub fn spec_exports(spec: &Path, arch: &str, version: u32) -> Result<BTreeMap<String, String>> {
    let mut out = BTreeMap::new();
    let wide = if arch == "x86_64" || arch == "arm64" {
        "win64"
    } else {
        "win32"
    };
    for line in crate::read(spec)?.lines() {
        let toks: Vec<_> = line
            .split('#')
            .next()
            .unwrap_or("")
            .split_whitespace()
            .collect();
        if toks.len() < 3 || !(toks[0] == "@" || toks[0].bytes().all(|c| c.is_ascii_digit())) {
            continue;
        }
        let mut state = if toks[1] == "stub" {
            "stub".to_owned()
        } else {
            "ok".to_owned()
        };
        let mut i = 2;
        while i < toks.len() && toks[i].starts_with('-') {
            let opt = toks[i];
            if let Some(a) = opt.strip_prefix("-arch=") {
                if !a.split(',').any(|x| {
                    let neg = x.starts_with('!');
                    (x.trim_start_matches('!') == arch || x.trim_start_matches('!') == wide) != neg
                }) {
                    state = format!("not exported on {arch}")
                }
            } else if opt == "-i386" && arch != "i386" {
                state = format!("not exported on {arch}")
            } else if let Some(ranges) = opt.strip_prefix("-version=") {
                let mut include = false;
                for r in ranges.split(',') {
                    let plus = r.ends_with('+');
                    let r = r.trim_end_matches('+');
                    let (lo, hi) = r.split_once('-').unwrap_or((r, r));
                    let parse = |s: &str| {
                        u32::from_str_radix(s.trim_start_matches("0x"), 16)
                            .map_err(|e| Error::new(e.to_string()))
                    };
                    let lo = parse(lo)?;
                    let hi = if plus { 0xfff } else { parse(hi)? };
                    include |= lo <= version && version <= hi;
                }
                if !include && !state.starts_with("not exported on") {
                    state = format!("exported only for -version={ranges}, tree builds {version:#x}")
                }
            } else if (opt == "-stub" || opt.starts_with("-stub=")) && state == "ok" {
                state = "stub".into()
            }
            i += 1;
        }
        if i < toks.len() {
            let name = toks[i].split(['(', '=']).next().unwrap_or("").to_owned();
            if out.get(&name).map(String::as_str) != Some("ok") {
                out.insert(name, state);
            }
        }
    }
    Ok(out)
}
pub fn check_imports(
    exe: &Path,
    imports: &BTreeMap<String, Vec<String>>,
    repo: &Path,
) -> Result<Vec<String>> {
    let arch = machine_arch(exe)?;
    let version = export_version(repo)?;
    let index = spec_index(repo)?;
    let mut bad = Vec::new();
    for (dll, names) in imports {
        let lower = dll.to_lowercase();
        let key = lower.strip_suffix(".dll").unwrap_or(&lower);
        let Some(specs) = index.get(key) else {
            continue;
        };
        if specs.len() != 1 {
            continue;
        }
        let exports = spec_exports(&specs[0], arch, version)?;
        for name in names {
            if name.starts_with('#') {
                continue;
            }
            let state = exports
                .get(name)
                .map(String::as_str)
                .unwrap_or("not in its .spec");
            if state != "ok" {
                bad.push(format!(
                    "{dll}!{name}: {state} ({})",
                    specs[0].strip_prefix(repo).unwrap_or(&specs[0]).display()
                ));
            }
        }
    }
    Ok(bad)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_short_images() {
        let f = tempfile::NamedTempFile::new().unwrap();
        assert!(parse(f.path()).is_err());
    }
}

fn guard_value(expr: &str, off: &std::collections::BTreeSet<String>) -> Option<bool> {
    let names = regex::Regex::new(r"UCRTDIFF_REF_(HAS|CALLABLE)_(\w+)").ok()?;
    let s = names.replace_all(expr, |c: &regex::Captures| {
        if &c[1] == "HAS" && off.contains(&c[2]) {
            "0"
        } else {
            "1"
        }
    });
    let comments = regex::Regex::new(r"/\*.*?\*/").ok()?;
    let s = comments.replace_all(&s, " ");
    let mut toks = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if i + 1 < bytes.len() && (&bytes[i..i + 2] == b"&&" || &bytes[i..i + 2] == b"||") {
            toks.push(if bytes[i] == b'&' { "&&" } else { "||" });
            i += 2;
            continue;
        }
        let tok = match bytes[i] {
            b'0' => "0",
            b'1' => "1",
            b'!' => "!",
            b'(' => "(",
            b')' => ")",
            _ => return None,
        };
        toks.push(tok);
        i += 1
    }
    fn atom(t: &[&str], i: &mut usize) -> Option<bool> {
        let x = *t.get(*i)?;
        *i += 1;
        match x {
            "0" => Some(false),
            "1" => Some(true),
            "!" => Some(!atom(t, i)?),
            "(" => {
                let v = or(t, i)?;
                if t.get(*i) != Some(&")") {
                    return None;
                }
                *i += 1;
                Some(v)
            }
            _ => None,
        }
    }
    fn and(t: &[&str], i: &mut usize) -> Option<bool> {
        let mut v = atom(t, i)?;
        while t.get(*i) == Some(&"&&") {
            *i += 1;
            let rhs = atom(t, i)?;
            v = v && rhs
        }
        Some(v)
    }
    fn or(t: &[&str], i: &mut usize) -> Option<bool> {
        let mut v = and(t, i)?;
        while t.get(*i) == Some(&"||") {
            *i += 1;
            let rhs = and(t, i)?;
            v = v || rhs
        }
        Some(v)
    }
    let mut n = 0;
    let v = or(&toks, &mut n)?;
    if n == toks.len() {
        Some(v)
    } else {
        None
    }
}
pub fn compiled_in(src: &str, off: &std::collections::BTreeSet<String>) -> Result<String> {
    let src = src.replace("\\\n", " ");
    let def = regex::Regex::new(r"^\s*#\s*define\s+(\w+)\s+(.+)$")?;
    let words = regex::Regex::new(r"\b\w+\b")?;
    let directive = regex::Regex::new(r"^#\s*(if|ifdef|ifndef|elif|else|endif)\b\s*(.*)$")?;
    let mut macros = BTreeMap::<String, String>::new();
    for line in src.lines() {
        if let Some(m) = def.captures(line) {
            if m[2].contains("UCRTDIFF_REF_") && guard_value(&m[2], off).is_some() {
                macros.insert(m[1].into(), format!("({})", &m[2]));
            }
        }
    }
    let mut stack: Vec<(bool, bool, bool)> = Vec::new();
    let mut kept = Vec::new();
    for line in src.lines() {
        let d = line.trim();
        if let Some(m) = directive.captures(d) {
            let expr = words
                .replace_all(&m[2], |c: &regex::Captures| {
                    macros
                        .get(&c[0])
                        .cloned()
                        .unwrap_or_else(|| c[0].to_owned())
                })
                .into_owned();
            match &m[1] {
                "if" => {
                    let v = guard_value(&expr, off);
                    stack.push(v.map(|x| (x, true, x)).unwrap_or((true, false, true)));
                }
                "ifdef" | "ifndef" => stack.push((true, false, true)),
                "elif" => {
                    if let Some((live, guard, taken)) = stack.last_mut() {
                        let v = if *guard {
                            guard_value(&expr, off)
                        } else {
                            None
                        };
                        if let Some(v) = v {
                            *live = v && !*taken;
                            *taken |= v
                        } else {
                            *live = true;
                            *guard = false;
                            *taken = true
                        }
                    }
                }
                "else" => {
                    if let Some((live, guard, taken)) = stack.last_mut() {
                        if *guard {
                            *live = !*taken
                        }
                        *taken = true
                    }
                }
                "endif" => {
                    stack.pop();
                }
                _ => {}
            }
            continue;
        }
        if stack.iter().all(|x| x.0) {
            kept.push(line);
            if let Some(m) = def.captures(d) {
                if !m[1].starts_with("UCRTDIFF_REF_") && (m[2].trim() == "0" || m[2].trim() == "1")
                {
                    macros.insert(m[1].into(), m[2].trim().into());
                }
            }
        }
    }
    Ok(kept.join("\n"))
}
fn arg<'a>(request: &'a serde_json::Value, key: &str) -> Result<&'a serde_json::Value> {
    request
        .get("kwargs")
        .and_then(|v| v.get(key))
        .ok_or_else(|| Error::new(format!("missing argument {key}")))
}
fn string<'a>(v: &'a serde_json::Value) -> Result<&'a str> {
    v.as_str().ok_or_else(|| Error::new("expected string"))
}
fn strset(v: &serde_json::Value) -> Result<std::collections::BTreeSet<String>> {
    Ok(v.as_array()
        .ok_or("expected array")?
        .iter()
        .map(|x| string(x).map(str::to_owned))
        .collect::<Result<_>>()?)
}
pub fn dispatch(root: &Path, request: &serde_json::Value) -> Result<serde_json::Value> {
    use serde_json::json;
    let op = string(request.get("op").ok_or("missing op")?)?.trim_start_matches("payload_pe_");
    let p = |k: &str| -> Result<PathBuf> {
        let s = string(arg(request, k)?)?;
        let path = PathBuf::from(s);
        Ok(if path.is_absolute() {
            path
        } else {
            root.join(path)
        })
    };
    match op {
        "parse" => {
            let (s, i) = parse(&p("path")?)?;
            Ok(json!([s, i]))
        }
        "machine_arch" => Ok(json!(machine_arch(&p("path")?)?)),
        "export_version" => Ok(json!(export_version(&p("repo")?)?)),
        "spec_index" => Ok(json!(spec_index(&p("repo")?)?)),
        "spec_exports" => Ok(json!(spec_exports(
            &p("spec")?,
            string(arg(request, "arch")?)?,
            arg(request, "version")?.as_u64().ok_or("invalid version")? as u32
        )?)),
        "check_imports" => {
            let imports = serde_json::from_value(arg(request, "imports")?.clone())?;
            Ok(json!(check_imports(&p("exe")?, &imports, &p("repo")?)?))
        }
        "compiled_in" => Ok(json!(compiled_in(
            string(arg(request, "src")?)?,
            &strset(arg(request, "off_refs")?)?
        )?)),
        "guard_value" => Ok(json!(guard_value(
            string(arg(request, "expr")?)?,
            &strset(arg(request, "off_refs")?)?
        ))),
        "main" => main(root, arg(request, "argv")?),
        _ => Err(Error::new(format!("unknown payload PE operation: {op}"))),
    }
}
fn main(root: &Path, argv: &serde_json::Value) -> Result<serde_json::Value> {
    use serde_json::json;
    let args = argv
        .as_array()
        .ok_or("argv must be an array")?
        .iter()
        .map(|x| string(x).map(str::to_owned))
        .collect::<Result<Vec<_>>>()?;
    let mut exe = None;
    let mut tag = None;
    let mut gui = false;
    let mut no_spec = false;
    let mut require = Vec::new();
    let mut forbid = Vec::new();
    let (mut graded, mut decls, mut refs) = (None, None, None);
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if a == "--gui" {
            gui = true
        } else if a == "--no-spec-check" {
            no_spec = true
        } else if a.starts_with("--") {
            i += 1;
            let v = args
                .get(i)
                .ok_or_else(|| Error::new(format!("{a} requires a value")))?
                .clone();
            match a.as_str() {
                "--tag" => tag = Some(v),
                "--require-dll" => require.push(v),
                "--forbid-dll" => forbid.push(v),
                "--graded-from" => graded = Some(v),
                "--graded-decls" => decls = Some(v),
                "--refs-header" => refs = Some(v),
                _ => return Err(Error::new(format!("unknown option {a}"))),
            }
        } else if exe.replace(a.clone()).is_some() {
            return Err("too many arguments".into());
        }
        i += 1
    }
    let exe = exe.ok_or("missing exe")?;
    let tag = tag.ok_or("--tag required")?;
    let p = Path::new(&exe);
    let p = if p.is_absolute() {
        p.to_owned()
    } else {
        root.join(p)
    };
    let name = p.file_name().unwrap_or_default().to_string_lossy();
    let fail = |msg: String, pre: &str| json!({"code":1,"stdout":format!("{pre}[{tag}] FAIL: {name}: {msg}\n")});
    let (subsystem, imports) = match parse(&p) {
        Ok(x) => x,
        Err(e) => return Ok(fail(format!("cannot read its PE headers: {e}"), "")),
    };
    if gui && subsystem != 2 {
        return Ok(fail(format!("subsystem={subsystem} (want GUI, 2)"), ""));
    }
    for want in require {
        if !imports.keys().any(|x| x.eq_ignore_ascii_case(&want)) {
            return Ok(fail(
                format!(
                    "does not import {want} (imports: {})",
                    imports.keys().cloned().collect::<Vec<_>>().join(", ")
                ),
                "",
            ));
        }
    }
    if !no_spec {
        let bad = match check_imports(&p, &imports, root) {
            Ok(x) => x,
            Err(e) => {
                return Ok(fail(
                    format!("cannot check imports against the tree's .spec files: {e}"),
                    "",
                ))
            }
        };
        if !bad.is_empty() {
            return Ok(fail(
                format!("imports our DLLs do not export: {}", bad.join("; ")),
                &format!("[{tag}]   the loader binds these to 0xffbadd11; the first call traps\n"),
            ));
        }
    }
    for bad in forbid {
        let hits = imports
            .keys()
            .filter(|d| d.to_lowercase().contains(&bad.to_lowercase()))
            .cloned()
            .collect::<Vec<_>>();
        if !hits.is_empty() {
            return Ok(fail(
                format!("imports {} (forbidden: {bad})", hits.join(", ")),
                "",
            ));
        }
    }
    if let Some(dll) = graded {
        let Some(decls) = decls else {
            return Ok(fail("--graded-from needs --graded-decls".into(), ""));
        };
        let have = imports
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(&dll))
            .map(|(_, v)| v.as_slice())
            .unwrap_or(&[]);
        let mut off = std::collections::BTreeSet::new();
        if let Some(refs) = refs {
            let file = PathBuf::from(refs);
            if file.exists() {
                let re = regex::Regex::new(r"(?m)^#define UCRTDIFF_REF_HAS_(\w+) 0$")?;
                for m in re.captures_iter(&crate::read(&file)?) {
                    off.insert(m[1].to_owned());
                }
            }
        }
        let src = compiled_in(&crate::read(Path::new(&decls))?, &off)?;
        let re = regex::Regex::new(r"__cdecl\s+\*?ref_([A-Za-z_][A-Za-z0-9_]*)\s*\(")?;
        let wanted = re
            .captures_iter(&src)
            .map(|m| m[1].to_owned())
            .collect::<std::collections::BTreeSet<_>>();
        let missing = wanted
            .iter()
            .filter(|n| !off.contains(*n) && !have.contains(*n))
            .cloned()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            return Ok(fail(
                format!("graded but NOT imported from {dll}: {}", missing.join(" ")),
                &format!("[{tag}]   those rows would grade the compiler, not our DLL\n"),
            ));
        }
    }
    Ok(json!({"code":0,"stdout":""}))
}
#[cfg(test)]
mod fixture_tests {
    use super::*;
    use std::io::Write;
    #[test]
    fn imports_and_ordinal_both_widths() {
        for wide in [false, true] {
            let mut b = vec![0u8; 0x500];
            b[..2].copy_from_slice(b"MZ");
            b[0x3c..0x40].copy_from_slice(&0x40u32.to_le_bytes());
            b[0x40..0x44].copy_from_slice(b"PE\0\0");
            b[0x44..0x46].copy_from_slice(&(if wide { 0x8664u16 } else { 0x14c }).to_le_bytes());
            b[0x46..0x48].copy_from_slice(&1u16.to_le_bytes());
            let os = if wide { 240u16 } else { 224 };
            b[0x54..0x56].copy_from_slice(&os.to_le_bytes());
            let opt = 0x58;
            b[opt..opt + 2].copy_from_slice(&(if wide { 0x20bu16 } else { 0x10b }).to_le_bytes());
            b[opt + 68..opt + 70].copy_from_slice(&2u16.to_le_bytes());
            let dd = opt + if wide { 112 } else { 96 };
            b[dd + 8..dd + 12].copy_from_slice(&0x1000u32.to_le_bytes());
            let sh = opt + os as usize;
            b[sh + 8..sh + 12].copy_from_slice(&0x300u32.to_le_bytes());
            b[sh + 12..sh + 16].copy_from_slice(&0x1000u32.to_le_bytes());
            b[sh + 16..sh + 20].copy_from_slice(&0x300u32.to_le_bytes());
            b[sh + 20..sh + 24].copy_from_slice(&0x200u32.to_le_bytes());
            b[0x200..0x204].copy_from_slice(&0x1040u32.to_le_bytes());
            b[0x20c..0x210].copy_from_slice(&0x1080u32.to_le_bytes());
            b[0x210..0x214].copy_from_slice(&0x1040u32.to_le_bytes());
            if wide {
                b[0x240..0x248].copy_from_slice(&0x1090u64.to_le_bytes());
                b[0x248..0x250].copy_from_slice(&((1u64 << 63) | 42).to_le_bytes())
            } else {
                b[0x240..0x244].copy_from_slice(&0x1090u32.to_le_bytes());
                b[0x244..0x248].copy_from_slice(&((1u32 << 31) | 42).to_le_bytes())
            }
            b[0x280..0x289].copy_from_slice(b"test.dll\0");
            b[0x292..0x296].copy_from_slice(b"foo\0");
            let mut f = tempfile::NamedTempFile::new().unwrap();
            f.write_all(&b).unwrap();
            let (sub, imports) = parse(f.path()).unwrap();
            assert_eq!(sub, 2);
            assert_eq!(imports["test.dll"], ["foo", "#42"]);
        }
    }
    #[test]
    fn compound_guard_and_macro() {
        let off = ["a".to_owned(), "b".to_owned()].into_iter().collect();
        let s="#define ANY (UCRTDIFF_REF_HAS_a || UCRTDIFF_REF_HAS_b)\n#if ANY\nint ref_hidden;\n#else\nint ref_live;\n#endif\n";
        let kept = compiled_in(s, &off).unwrap();
        assert!(!kept.contains("ref_hidden"));
        assert!(kept.contains("ref_live"));
    }
}
