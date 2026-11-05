//! Compute the complete guest PE dependency closure.
use regex::Regex;
use sha1::{Digest, Sha1};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub type Result<T> = std::result::Result<T, String>;
type Index = BTreeMap<String, PathBuf>;

fn name(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase()
}
fn module(s: &str) -> String {
    s.rsplit(['/', '\\']).next().unwrap_or(s).to_lowercase()
}
fn is_module(s: &str) -> bool {
    ["dll", "drv", "ocx", "acm", "cpl"].iter().any(|x| {
        s.rsplit_once('.')
            .is_some_and(|(_, e)| e.eq_ignore_ascii_case(x))
    })
}

fn declared_modules_inner(
    root: &Path,
    inf: &str,
    kinds: &[&str],
) -> Result<BTreeMap<String, String>> {
    let path = root.join(inf);
    let data = fs::read(&path).map_err(|e| {
        format!(
            "wow64-closure: catalog INF not found or unreadable: {}: {e}",
            path.display()
        )
    })?;
    let text = String::from_utf8_lossy(&data)
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    let mut out = BTreeMap::new();
    for kind in kinds {
        let pattern = match *kind {
            "LibraryPath" | "HelperDllName" | "AutodialDLL" => format!(r#""{}",0x00000000,"([^"]+)""#, kind),
            "Drivers32" => r#"(?m)^HKLM,"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Drivers32","[^"]+",0x00000000,"([^"]+)""#.to_string(),
            "PackedCatalogItem" => r#""PackedCatalogItem",0x00000001,\\\n([0-9A-Fa-f, \\\n]+)"#.to_string(),
            _ => return Err(format!("wow64-closure: unknown INF declaration kind: {kind}")),
        };
        let re = Regex::new(&pattern).map_err(|e| e.to_string())?;
        for cap in re.captures_iter(&text) {
            let val = if *kind == "PackedCatalogItem" {
                let octets = Regex::new(r"\b[0-9A-Fa-f]{2}\b").unwrap();
                let bytes: Vec<u8> = octets
                    .find_iter(&cap[1])
                    .map(|m| u8::from_str_radix(m.as_str(), 16).unwrap())
                    .take_while(|x| *x != 0)
                    .collect();
                bytes.iter().map(|x| char::from(*x)).collect::<String>()
            } else {
                cap[1].to_string()
            };
            if !val.is_empty() {
                out.insert(module(&val), format!("{inf} {kind}"));
            }
        }
    }
    if out.is_empty() {
        return Err(format!("wow64-closure: {inf} declared no {} entry — the INF changed shape and the dynamic edge is no longer derived", kinds.join("/")));
    }
    Ok(out)
}

fn stripped_twins_inner(build: &Path) -> Result<Index> {
    let dir = build.join("boot/stripped");
    let manifest = dir.join("strip-manifest.json");
    if !manifest.is_file() {
        return Ok(Index::new());
    }
    let v: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest).map_err(|e| e.to_string())?)
            .map_err(|e| format!("{}: {e}", manifest.display()))?;
    let keys: Vec<&str> = match &v {
        serde_json::Value::Object(o) => o.keys().map(String::as_str).collect(),
        serde_json::Value::Array(a) => a.iter().filter_map(|x| x.as_str()).collect(),
        _ => return Err(format!("invalid strip manifest: {}", manifest.display())),
    };
    let marker = format!(
        "/{}/",
        build.file_name().unwrap_or_default().to_string_lossy()
    );
    let mut out = Index::new();
    for key in keys {
        if let Some((_, rel)) = key.split_once(&marker) {
            let digest = format!("{:x}", Sha1::digest(key.as_bytes()));
            let twin = dir.join(format!(
                "{}-{}",
                &digest[..12],
                key.rsplit('/').next().unwrap_or(key)
            ));
            if twin.is_file() {
                out.insert(rel.to_string(), twin);
            }
        }
    }
    Ok(out)
}

fn index_build_inner(build: &Path, no_strip: bool) -> Result<Index> {
    fn walk(dir: &Path, out: &mut Index) -> Result<()> {
        let mut subdirs = Vec::new();
        for ent in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
            let ent = ent.map_err(|e| e.to_string())?;
            let p = ent.path();
            let n = ent.file_name().to_string_lossy().into_owned();
            if n == "CMakeFiles" || n == "stripped" || n.starts_with("livecd-rw-") {
                continue;
            }
            let ty = ent.file_type().map_err(|e| e.to_string())?;
            if ty.is_dir() {
                subdirs.push(p);
            } else if p.is_file() && is_module(&n) {
                out.entry(n.to_lowercase()).or_insert(p);
            }
        }
        for subdir in subdirs {
            walk(&subdir, out)?;
        }
        Ok(())
    }
    let mut out = Index::new();
    walk(build, &mut out)?;
    if !no_strip {
        let twins = stripped_twins_inner(build)?;
        for p in out.values_mut() {
            if let Ok(rel) = p.strip_prefix(build) {
                if let Some(t) = twins.get(&rel.to_string_lossy().replace('\\', "/")) {
                    *p = t.clone();
                }
            }
        }
    }
    Ok(out)
}

fn string_referenced_inner(pe: &Path, index: &Index) -> Result<BTreeSet<String>> {
    let data = fs::read(pe).map_err(|e| format!("{}: {e}", pe.display()))?;
    let token = Regex::new(r"(?i)[A-Za-z0-9_.-]+\.(?:dll|ocx|drv|acm|cpl)").unwrap();
    let mut found = BTreeSet::new();
    let mut harvest = |s: &str| {
        for m in token.find_iter(s) {
            let t = m.as_str().to_ascii_lowercase();
            for i in 0..t.len() {
                if index.contains_key(&t[i..]) {
                    found.insert(t[i..].to_string());
                    break;
                }
            }
        }
    };
    let mut i = 0;
    while i < data.len() {
        let start = i;
        while i < data.len() && (0x20..=0x7e).contains(&data[i]) && i - start < 120 {
            i += 1;
        }
        if i - start >= 2 {
            harvest(std::str::from_utf8(&data[start..i]).unwrap());
        }
        if i == start {
            i += 1;
        }
    }
    let mut i = 0;
    while i + 1 < data.len() {
        let start = i;
        let mut s = String::new();
        while i + 1 < data.len()
            && (0x20..=0x7e).contains(&data[i])
            && data[i + 1] == 0
            && s.len() < 120
        {
            s.push(data[i] as char);
            i += 2;
        }
        if s.len() >= 2 {
            harvest(&s);
        }
        if i == start {
            i += 1;
        }
    }
    Ok(found)
}

fn u16at(d: &[u8], p: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        d.get(p..p.checked_add(2)?)?.try_into().ok()?,
    ))
}
fn u32at(d: &[u8], p: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        d.get(p..p.checked_add(4)?)?.try_into().ok()?,
    ))
}
fn cstr(d: &[u8], p: usize) -> Option<&str> {
    let s = d.get(p..)?;
    let n = s.iter().position(|b| *b == 0)?;
    std::str::from_utf8(&s[..n]).ok()
}
#[derive(Clone, Copy)]
struct Section {
    va: u32,
    span: u32,
    raw: u32,
    size: u32,
    chars: u32,
}
struct Pe<'a> {
    data: &'a [u8],
    secs: Vec<Section>,
    image: u32,
    dirs: usize,
}
impl<'a> Pe<'a> {
    fn parse(d: &'a [u8]) -> Option<Self> {
        if d.get(..2)? != b"MZ" {
            return None;
        }
        let p = u32at(d, 0x3c)? as usize;
        if d.get(p..p.checked_add(4)?)? != b"PE\0\0" {
            return None;
        }
        let n = u16at(d, p + 6)? as usize;
        let os = u16at(d, p + 20)? as usize;
        let opt = p + 24;
        let image = u32at(d, opt + 56)?;
        let dirs = opt
            + match u16at(d, opt)? {
                0x10b => 96,
                0x20b => 112,
                _ => return None,
            };
        let mut secs = Vec::new();
        for i in 0..n {
            let s = opt.checked_add(os)?.checked_add(i.checked_mul(40)?)?;
            let vs = u32at(d, s + 8)?;
            let va = u32at(d, s + 12)?;
            let size = u32at(d, s + 16)?;
            let raw = u32at(d, s + 20)?;
            let chars = u32at(d, s + 36)?;
            secs.push(Section {
                va,
                span: vs.max(size),
                raw,
                size,
                chars,
            });
        }
        Some(Self {
            data: d,
            secs,
            image,
            dirs,
        })
    }
    fn off(&self, r: u32) -> Option<usize> {
        for s in &self.secs {
            let delta = r.checked_sub(s.va);
            if let Some(d) = delta {
                if d < s.span && d < s.size {
                    return (s.raw as usize)
                        .checked_add(d as usize)
                        .filter(|x| *x < self.data.len());
                }
            }
        }
        None
    }
    fn directory(&self, i: usize) -> Option<(u32, u32)> {
        Some((
            u32at(self.data, self.dirs + i * 8)?,
            u32at(self.data, self.dirs + i * 8 + 4)?,
        ))
    }
    fn imports(&self) -> Vec<String> {
        let mut out = Vec::new();
        if let Some((r, _)) = self.directory(1) {
            if let Some(mut p) = self.off(r) {
                while let (Some(orig), Some(n), Some(first)) = (
                    u32at(self.data, p),
                    u32at(self.data, p + 12),
                    u32at(self.data, p + 16),
                ) {
                    if orig == 0 && n == 0 && first == 0 {
                        break;
                    }
                    if let Some(s) = self.off(n).and_then(|x| cstr(self.data, x)) {
                        out.push(s.to_lowercase());
                    }
                    p += 20;
                }
            }
        }
        out
    }
    fn forwards(&self) -> Vec<String> {
        let mut out = BTreeSet::new();
        if let Some((r, size)) = self.directory(0) {
            if let Some(p) = self.off(r) {
                if let (Some(n), Some(funcs)) = (u32at(self.data, p + 20), u32at(self.data, p + 28))
                {
                    if let Some(f) = self.off(funcs) {
                        for i in 0..n.min((self.data.len() / 4) as u32) {
                            if let Some(x) = u32at(self.data, f + i as usize * 4) {
                                if x >= r && x.checked_sub(r).is_some_and(|d| d < size) {
                                    if let Some(s) = self.off(x).and_then(|o| cstr(self.data, o)) {
                                        if let Some((dll, _)) = s.split_once('.') {
                                            out.insert(format!("{}.dll", dll.to_lowercase()));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        out.into_iter().collect()
    }
    fn delays(&self) -> Vec<String> {
        let re = Regex::new(r"(?i)^[A-Za-z0-9_.-]+\.(?:dll|ocx|drv|acm|cpl)$").unwrap();
        let mut out = Vec::new();
        for s in &self.secs {
            if s.chars & 0x20000000 != 0 {
                continue;
            }
            let start = s.raw as usize;
            let end = (start.saturating_add(s.size as usize)).min(self.data.len());
            if end.saturating_sub(start) < 32 {
                continue;
            }
            for p in (start..=end - 32).step_by(4) {
                if u32at(self.data, p) != Some(1)
                    || (5..8).any(|i| u32at(self.data, p + i * 4) != Some(0))
                {
                    continue;
                }
                let vals: Vec<u32> = (1..5).filter_map(|i| u32at(self.data, p + i * 4)).collect();
                if vals.len() != 4 || vals.iter().any(|v| *v == 0 || *v >= self.image) {
                    continue;
                }
                if let Some(n) = self.off(vals[0]).and_then(|x| cstr(self.data, x)) {
                    if re.is_match(n) {
                        out.push(n.to_lowercase());
                    }
                }
            }
        }
        out
    }
}

fn delay_imports_of_inner(pe: &Path) -> Result<Vec<String>> {
    let data = fs::read(pe).map_err(|e| format!("{}: {e}", pe.display()))?;
    Ok(Pe::parse(&data).map(|p| p.delays()).unwrap_or_default())
}

pub fn declared_modules(
    root: &Path,
    inf: &str,
    kinds: &[&str],
) -> crate::Result<BTreeMap<String, String>> {
    declared_modules_inner(root, inf, kinds).map_err(crate::Error::new)
}
pub fn stripped_twins(build: &Path) -> crate::Result<BTreeMap<String, PathBuf>> {
    stripped_twins_inner(build).map_err(crate::Error::new)
}
pub fn index_build(build: &Path, no_strip: bool) -> crate::Result<BTreeMap<String, PathBuf>> {
    index_build_inner(build, no_strip).map_err(crate::Error::new)
}
pub fn string_referenced(
    pe: &Path,
    index: &BTreeMap<String, PathBuf>,
) -> crate::Result<BTreeSet<String>> {
    string_referenced_inner(pe, index).map_err(crate::Error::new)
}
pub fn delay_imports_of(pe: &Path) -> crate::Result<Vec<String>> {
    delay_imports_of_inner(pe).map_err(crate::Error::new)
}

struct Walker<'a> {
    root: &'a Path,
    index: Index,
    prestaged: bool,
    literals: bool,
    seen: HashSet<String>,
    dynamic_seen: HashSet<String>,
    missing: BTreeMap<String, BTreeSet<String>>,
    out: Index,
    stack: Vec<PathBuf>,
    dynamic: BTreeMap<String, String>,
}
impl Walker<'_> {
    fn admit(&mut self, n: &str, why: String) {
        if !self.seen.insert(n.to_string()) {
            return;
        }
        if let Some(p) = self.index.get(n) {
            if self.prestaged
                || ![
                    "ntdll.dll",
                    "kernel32.dll",
                    "user32.dll",
                    "gdi32.dll",
                    "ntdll_vista.dll",
                ]
                .contains(&n)
            {
                self.out.insert(n.to_string(), p.clone());
            }
            self.stack.push(p.clone());
        } else {
            self.missing.entry(n.to_string()).or_default().insert(why);
        }
    }
    fn follow(&mut self, n: &str) -> Result<()> {
        if !self.dynamic_seen.insert(n.to_string()) {
            return Ok(());
        }
        let spec: Option<(&str, &[&str])> = match n {
            "ws2_32.dll" => Some((
                "media/inf/nettcpip.inf",
                &["LibraryPath", "PackedCatalogItem", "AutodialDLL"],
            )),
            "msafd.dll" => Some(("media/inf/nettcpip.inf", &["HelperDllName"])),
            "winmm.dll" => Some(("boot/bootdata/hivesft.inf", &["Drivers32"])),
            _ => None,
        };
        if let Some((inf, kinds)) = spec {
            for (target, decl) in declared_modules_inner(self.root, inf, kinds)? {
                self.dynamic
                    .entry(target.clone())
                    .or_insert_with(|| decl.clone());
                self.admit(&target, format!("{n} via {decl}"));
                self.follow(&target)?;
            }
        }
        Ok(())
    }
    fn run(
        &mut self,
        roots: &[String],
        build: &Path,
        emit_roots: bool,
        no_strip: bool,
    ) -> Result<()> {
        let twins = if no_strip {
            Index::new()
        } else {
            stripped_twins_inner(build)?
        };
        for r in roots {
            let p = Path::new(r);
            let p = if p.is_absolute() {
                p.to_owned()
            } else {
                build.join(p)
            };
            if !p.exists() {
                return Err(format!("wow64-closure: root not found: {}", p.display()));
            }
            if emit_roots {
                let twin = p
                    .canonicalize()
                    .ok()
                    .zip(build.canonicalize().ok())
                    .and_then(|(p, b)| {
                        p.strip_prefix(b)
                            .ok()
                            .map(|x| x.to_string_lossy().replace('\\', "/"))
                    })
                    .and_then(|r| twins.get(&r).cloned());
                self.out.insert(name(&p), twin.unwrap_or_else(|| p.clone()));
            }
            self.stack.push(p.clone());
            self.follow(&name(&p))?;
            for dep in string_referenced_inner(&p, &self.index)? {
                self.admit(&dep, format!("{} via string literal", name(&p)));
                self.follow(&dep)?;
            }
        }
        while let Some(p) = self.stack.pop() {
            let data = fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?;
            let pe = Pe::parse(&data)
                .ok_or_else(|| format!("wow64-closure: invalid PE: {}", p.display()))?;
            let mut deps = pe.imports();
            deps.extend(pe.delays());
            deps.extend(pe.forwards());
            if self.literals {
                deps.extend(string_referenced_inner(&p, &self.index)?);
            }
            for dep in deps {
                self.admit(&dep, name(&p));
                self.follow(&dep)?;
            }
        }
        if !self.missing.is_empty() {
            let mut msg = String::new();
            for (dep, why) in &self.missing {
                msg.push_str(&format!(
                    "wow64-closure: UNRESOLVED import {dep} (wanted by {})\n",
                    why.iter().cloned().collect::<Vec<_>>().join(", ")
                ));
            }
            msg.push_str("wow64-closure: refusing to emit a PARTIAL closure — a half-bound module crashes at first call, not at load. Build the missing module(s) first.");
            return Err(msg);
        }
        Ok(())
    }
}
fn compute(
    root: &Path,
    build: &Path,
    roots: &[String],
    include_prestaged: bool,
    emit_roots: bool,
    no_strip: bool,
    library_literals: bool,
) -> Result<(Index, BTreeMap<String, String>)> {
    if !build.is_dir() {
        return Err(format!(
            "wow64-closure: no such build dir: {}",
            build.display()
        ));
    }
    let mut w = Walker {
        root,
        index: index_build_inner(build, no_strip)?,
        prestaged: include_prestaged,
        literals: library_literals,
        seen: HashSet::new(),
        dynamic_seen: HashSet::new(),
        missing: BTreeMap::new(),
        out: Index::new(),
        stack: Vec::new(),
        dynamic: BTreeMap::new(),
    };
    w.run(roots, build, emit_roots, no_strip)?;
    Ok((w.out, w.dynamic))
}
pub fn closure(
    root: &Path,
    build: &Path,
    roots: &[String],
    include_prestaged: bool,
    emit_roots: bool,
    no_strip: bool,
    library_literals: bool,
) -> crate::Result<BTreeMap<String, PathBuf>> {
    compute(
        root,
        build,
        roots,
        include_prestaged,
        emit_roots,
        no_strip,
        library_literals,
    )
    .map(|x| x.0)
    .map_err(crate::Error::new)
}

/// JSON dispatch used by the compatibility facade and command-line frontend.
pub fn dispatch(root: &Path, req: &serde_json::Value) -> crate::Result<serde_json::Value> {
    let op = req.get("op").and_then(|x| x.as_str()).unwrap_or("closure");
    let path = |key: &str| -> crate::Result<PathBuf> {
        let s = req
            .get(key)
            .and_then(|x| x.as_str())
            .ok_or_else(|| crate::Error::new(format!("missing {key}")))?;
        let p = Path::new(s);
        Ok(if p.is_absolute() {
            p.to_owned()
        } else {
            root.join(p)
        })
    };
    let flag = |key: &str| req.get(key).and_then(|v| v.as_bool()).unwrap_or(false);
    let result: Result<serde_json::Value> = (|| match op {
        "declared_modules" => {
            let inf = req
                .get("inf")
                .and_then(|x| x.as_str())
                .ok_or("missing inf")?;
            let kinds = req
                .get("kinds")
                .and_then(|x| x.as_array())
                .ok_or("missing kinds")?
                .iter()
                .map(|x| x.as_str().ok_or_else(|| "invalid kind".to_owned()))
                .collect::<Result<Vec<_>>>()?;
            Ok(serde_json::json!(declared_modules_inner(
                root, inf, &kinds
            )?))
        }
        "find_objdump" => {
            let bundled = root.join(".build-tools/llvm-mingw/bin/llvm-objdump");
            if bundled.exists() {
                Ok(serde_json::json!(bundled))
            } else {
                let paths = std::env::var_os("PATH").unwrap_or_default();
                let mut found = None;
                for program in ["llvm-objdump", "objdump"] {
                    for dir in std::env::split_paths(&paths) {
                        let p = dir.join(program);
                        if p.is_file() {
                            found = Some(p);
                            break;
                        }
                    }
                    if found.is_some() {
                        break;
                    }
                }
                Ok(serde_json::json!(found.ok_or(
                    "wow64-closure: no llvm-objdump found (need .build-tools/llvm-mingw)"
                )?))
            }
        }
        "index_build" => Ok(serde_json::json!(index_build_inner(
            &path("build_dir").map_err(|e| e.to_string())?,
            flag("no_strip")
        )?)),
        "stripped_twins" => Ok(serde_json::json!(stripped_twins_inner(
            &path("build_dir").map_err(|e| e.to_string())?
        )?)),
        "delay_imports_of" => Ok(serde_json::json!(delay_imports_of_inner(
            &path("pe").map_err(|e| e.to_string())?
        )?)),
        "forward_targets_of" | "imports_of" => {
            let p = path("pe").map_err(|e| e.to_string())?;
            let data = fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?;
            let pe = Pe::parse(&data)
                .ok_or_else(|| format!("wow64-closure: invalid PE: {}", p.display()))?;
            Ok(serde_json::json!(if op == "imports_of" {
                pe.imports()
            } else {
                pe.forwards()
            }))
        }
        "string_referenced" => {
            let pe = path("pe").map_err(|e| e.to_string())?;
            let index: Index =
                serde_json::from_value(req.get("index").cloned().ok_or("missing index")?)
                    .map_err(|e| e.to_string())?;
            Ok(serde_json::json!(string_referenced_inner(&pe, &index)?))
        }
        "closure" => {
            let build = path("build_dir").map_err(|e| e.to_string())?;
            let roots = req
                .get("roots")
                .and_then(|x| x.as_array())
                .ok_or("missing roots")?
                .iter()
                .map(|x| {
                    x.as_str()
                        .map(str::to_owned)
                        .ok_or_else(|| "invalid root".to_string())
                })
                .collect::<Result<Vec<_>>>()?;
            let (paths, dynamic) = compute(
                root,
                &build,
                &roots,
                flag("include_prestaged"),
                flag("emit_roots"),
                flag("no_strip"),
                flag("library_literals"),
            )?;
            Ok(serde_json::json!({"paths":paths,"dynamic":dynamic}))
        }
        _ => Err(format!("unknown closure operation: {op}")),
    })();
    result.map_err(crate::Error::new)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inf_catalog_and_drivers_are_derived() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("media/inf")).unwrap();
        fs::write(tmp.path().join("media/inf/a.inf"), "\"LibraryPath\",0x00000000,\"%SystemRoot%\\system32\\mswsock.dll\"\n\"PackedCatalogItem\",0x00000001,\\\n6D,73,61,66,64,2E,64,6C,6C,00,\\\n00,00\n\"HelperDllName\",0x00000000,\"wshtcpip.dll\"\n").unwrap();
        let a = declared_modules_inner(
            tmp.path(),
            "media/inf/a.inf",
            &["LibraryPath", "PackedCatalogItem"],
        )
        .unwrap();
        assert!(a.contains_key("mswsock.dll"));
        assert!(a.contains_key("msafd.dll"));
        assert!(declared_modules_inner(tmp.path(), "missing", &["LibraryPath"]).is_err());
        fs::write(tmp.path().join("drivers.inf"), ";HKLM,\"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Drivers32\",\"wave\",0x00000000,\"bad.dll\"\nHKLM,\"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Drivers32\",\"wave\",0x00000000,\"midimap.dll\"\n").unwrap();
        let d = declared_modules_inner(tmp.path(), "drivers.inf", &["Drivers32"]).unwrap();
        assert_eq!(d.len(), 1);
        assert!(d.contains_key("midimap.dll"));
    }
    #[test]
    fn delay_descriptor_shape_and_literals() {
        let mut d = vec![0u8; 0x300];
        d[0..2].copy_from_slice(b"MZ");
        d[0x3c..0x40].copy_from_slice(&0x80u32.to_le_bytes());
        d[0x80..0x84].copy_from_slice(b"PE\0\0");
        d[0x86..0x88].copy_from_slice(&1u16.to_le_bytes());
        d[0x94..0x96].copy_from_slice(&0xe0u16.to_le_bytes());
        d[0x98..0x9a].copy_from_slice(&0x10bu16.to_le_bytes());
        d[0xd0..0xd4].copy_from_slice(&0x2000u32.to_le_bytes());
        let sh = 0x178;
        for (o, v) in [
            (8, 0x100u32),
            (12, 0x1000),
            (16, 0x100),
            (20, 0x200),
            (36, 0x40000040),
        ] {
            d[sh + o..sh + o + 4].copy_from_slice(&v.to_le_bytes());
        }
        for (i, v) in [1u32, 0x1040, 0x1080, 0x1090, 0x10a0, 0, 0, 0]
            .iter()
            .enumerate()
        {
            d[0x200 + i * 4..0x204 + i * 4].copy_from_slice(&v.to_le_bytes());
        }
        d[0x240..0x24c].copy_from_slice(b"WinInet.dll\0");
        assert_eq!(Pe::parse(&d).unwrap().delays(), vec!["wininet.dll"]);
        d[0x200] = 2;
        assert!(Pe::parse(&d).unwrap().delays().is_empty());
        let t = tempfile::tempdir().unwrap();
        let f = t.path().join("a.exe");
        fs::write(&f, b"Aaclui.dll\0w\0i\0n\0i\0n\0e\0t\0.\0d\0l\0l\0").unwrap();
        let index = Index::from([
            ("aclui.dll".into(), PathBuf::new()),
            ("wininet.dll".into(), PathBuf::new()),
        ]);
        assert_eq!(string_referenced_inner(&f, &index).unwrap().len(), 2);
    }
    #[test]
    fn index_ignores_staging_copies_and_directories() {
        let t = tempfile::tempdir().unwrap();
        fs::create_dir_all(t.path().join("dll/foo.dll")).unwrap();
        fs::write(t.path().join("dll/foo.dll/foo.dll"), b"x").unwrap();
        fs::create_dir_all(t.path().join("livecd-rw-x")).unwrap();
        fs::write(t.path().join("livecd-rw-x/bad.dll"), b"x").unwrap();
        let i = index_build_inner(t.path(), true).unwrap();
        assert!(i["foo.dll"].is_file());
        assert!(!i.contains_key("bad.dll"));
    }
    fn put(d: &mut [u8], p: usize, v: u32) {
        d[p..p + 4].copy_from_slice(&v.to_le_bytes());
    }
    // PE32 with one data section. Optional tables live at fixed RVAs within it.
    fn fixture(import: Option<&str>, delay: Option<&str>, forward: Option<&str>) -> Vec<u8> {
        let mut d = vec![0u8; 0x800];
        d[..2].copy_from_slice(b"MZ");
        put(&mut d, 0x3c, 0x80);
        d[0x80..0x84].copy_from_slice(b"PE\0\0");
        d[0x86..0x88].copy_from_slice(&1u16.to_le_bytes());
        d[0x94..0x96].copy_from_slice(&0xe0u16.to_le_bytes());
        d[0x98..0x9a].copy_from_slice(&0x10bu16.to_le_bytes());
        put(&mut d, 0xd0, 0x2000);
        let sh = 0x178;
        for (o, v) in [
            (8, 0x600),
            (12, 0x1000),
            (16, 0x600),
            (20, 0x200),
            (36, 0x40000040),
        ] {
            put(&mut d, sh + o, v);
        }
        if let Some(n) = import {
            put(&mut d, 0x100, 0x1000);
            put(&mut d, 0x104, 40);
            put(&mut d, 0x200, 0x10a0);
            put(&mut d, 0x20c, 0x1080);
            put(&mut d, 0x210, 0x10b0);
            d[0x280..0x280 + n.len()].copy_from_slice(n.as_bytes());
        }
        if let Some(n) = delay {
            for (i, v) in [1, 0x1140, 0x1180, 0x1190, 0x11a0, 0, 0, 0]
                .iter()
                .enumerate()
            {
                put(&mut d, 0x300 + i * 4, *v);
            }
            d[0x340..0x340 + n.len()].copy_from_slice(n.as_bytes());
        }
        if let Some(n) = forward {
            put(&mut d, 0xf8, 0x1200);
            put(&mut d, 0xfc, 0x100);
            put(&mut d, 0x414, 1);
            put(&mut d, 0x41c, 0x1240);
            put(&mut d, 0x440, 0x1260);
            d[0x460..0x460 + n.len()].copy_from_slice(n.as_bytes());
        }
        d
    }
    #[test]
    fn closure_traverses_static_delay_forwarders_and_fails_missing() {
        let t = tempfile::tempdir().unwrap();
        let b = t.path().join("build-i386");
        fs::create_dir(&b).unwrap();
        fs::write(b.join("app.exe"), fixture(Some("one.dll"), None, None)).unwrap();
        fs::write(
            b.join("one.dll"),
            fixture(None, Some("two.dll"), Some("three.func")),
        )
        .unwrap();
        fs::write(b.join("two.dll"), fixture(None, None, None)).unwrap();
        fs::write(b.join("three.dll"), fixture(None, None, None)).unwrap();
        let roots = vec!["app.exe".into()];
        let result = closure(t.path(), &b, &roots, false, true, true, false).unwrap();
        assert_eq!(
            result.keys().map(String::as_str).collect::<Vec<_>>(),
            vec!["app.exe", "one.dll", "three.dll", "two.dll"]
        );
        fs::remove_file(b.join("three.dll")).unwrap();
        let err = closure(t.path(), &b, &roots, false, false, true, false)
            .unwrap_err()
            .to_string();
        assert!(err.contains("UNRESOLVED import three.dll"));
        assert!(err.contains("one.dll"));
    }
    #[test]
    fn closure_follows_dynamic_hops_and_reports_missing_helper() {
        let t = tempfile::tempdir().unwrap();
        let b = t.path().join("build-i386");
        fs::create_dir(&b).unwrap();
        fs::create_dir_all(t.path().join("media/inf")).unwrap();
        fs::write(t.path().join("media/inf/nettcpip.inf"), "\"LibraryPath\",0x00000000,\"msafd.dll\"\n\"HelperDllName\",0x00000000,\"wshtcpip.dll\"\n").unwrap();
        fs::write(b.join("app.exe"), fixture(Some("ws2_32.dll"), None, None)).unwrap();
        for n in ["ws2_32.dll", "msafd.dll", "wshtcpip.dll"] {
            fs::write(b.join(n), fixture(None, None, None)).unwrap();
        }
        let roots = vec!["app.exe".into()];
        let result = closure(t.path(), &b, &roots, false, false, true, false).unwrap();
        assert!(result.contains_key("msafd.dll"));
        assert!(result.contains_key("wshtcpip.dll"));
        fs::remove_file(b.join("wshtcpip.dll")).unwrap();
        let err = closure(t.path(), &b, &roots, false, false, true, false)
            .unwrap_err()
            .to_string();
        assert!(err.contains("UNRESOLVED import wshtcpip.dll"));
        assert!(err.contains("HelperDllName"));
    }
    #[test]
    fn stripped_manifest_matches_each_original_path() {
        let t = tempfile::tempdir().unwrap();
        let b = t.path().join("build-i386");
        let dir = b.join("dll/a");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("a.dll"), b"fat").unwrap();
        let stripped = b.join("boot/stripped");
        fs::create_dir_all(&stripped).unwrap();
        let key = "/tmp/vibeos/build-i386/dll/a/a.dll";
        fs::write(
            stripped.join("strip-manifest.json"),
            serde_json::to_vec(&serde_json::json!({(key):{}})).unwrap(),
        )
        .unwrap();
        let digest = format!("{:x}", Sha1::digest(key.as_bytes()));
        let twin = stripped.join(format!("{}-a.dll", &digest[..12]));
        fs::write(&twin, b"stripped").unwrap();
        assert_eq!(index_build(&b, false).unwrap()["a.dll"], twin);
        assert_eq!(index_build(&b, true).unwrap()["a.dll"], dir.join("a.dll"));
    }
}
