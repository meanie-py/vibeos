//! Read ordinary PE imports directly, and resolve against every shipped kernel variant.
use super::{BinaryContext, CheckOutput};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
type Names = BTreeSet<String>;
type Imports = BTreeMap<String, Vec<String>>;
fn u16_at(d: &[u8], o: usize) -> Result<u16, String> {
    Ok(u16::from_le_bytes(
        d.get(o..o.checked_add(2).ok_or("offset overflow")?)
            .ok_or("truncated PE")?
            .try_into()
            .unwrap(),
    ))
}
use super::freeldr::u32_at;
fn u64_at(d: &[u8], o: usize) -> Result<u64, String> {
    Ok(u64::from_le_bytes(
        d.get(o..o.checked_add(8).ok_or("offset overflow")?)
            .ok_or("truncated PE")?
            .try_into()
            .unwrap(),
    ))
}
struct Pe<'a> {
    data: &'a [u8],
    wide: bool,
    dirs: Vec<(u32, u32)>,
    sections: Vec<(u32, u32, u32, u32)>,
}
impl<'a> Pe<'a> {
    fn new(data: &'a [u8]) -> Result<Self, String> {
        if data.get(..2) != Some(b"MZ") {
            return Err("no MZ header".into());
        }
        let pe = u32_at(data, 0x3c)? as usize;
        if data.get(pe..pe + 4) != Some(b"PE\0\0") {
            return Err("no PE signature".into());
        }
        let nsect = u16_at(data, pe + 6)?;
        let optsize = u16_at(data, pe + 20)? as usize;
        let opt = pe + 24;
        let (wide, dd) = match u16_at(data, opt)? {
            0x10b => (false, opt + 96),
            0x20b => (true, opt + 112),
            v => return Err(format!("bad optional header magic {v:#x}")),
        };
        let ndd = u32_at(data, dd - 4)?.min(16);
        let mut dirs = Vec::new();
        for i in 0..ndd as usize {
            dirs.push((u32_at(data, dd + 8 * i)?, u32_at(data, dd + 8 * i + 4)?))
        }
        let mut sections = Vec::new();
        for i in 0..nsect as usize {
            let o = opt + optsize + 40 * i;
            sections.push((
                u32_at(data, o + 12)?,
                u32_at(data, o + 8)?.max(u32_at(data, o + 16)?),
                u32_at(data, o + 20)?,
                u32_at(data, o + 16)?,
            ));
        }
        Ok(Self {
            data,
            wide,
            dirs,
            sections,
        })
    }
    fn off(&self, rva: u32) -> Result<usize, String> {
        for &(va, size, ptr, raw) in &self.sections {
            if rva >= va && rva - va < size && rva - va < raw {
                return Ok(ptr as usize + (rva - va) as usize);
            }
        }
        Err(format!("rva {rva:#x} not in any section"))
    }
    fn cstr(&self, rva: u32) -> Result<String, String> {
        let o = self.off(rva)?;
        let rest = self.data.get(o..).ok_or("string offset outside PE")?;
        let n = rest
            .iter()
            .position(|b| *b == 0)
            .ok_or("unterminated PE string")?;
        Ok(rest[..n].iter().map(|b| char::from(*b)).collect())
    }
    fn exports(&self) -> Result<Names, String> {
        let Some(&(rva, _)) = self.dirs.first() else {
            return Ok(Names::new());
        };
        if rva == 0 {
            return Ok(Names::new());
        }
        let o = self.off(rva)?;
        let n = u32_at(self.data, o + 24)? as usize;
        let mut names = Names::new();
        if n != 0 {
            let table = self.off(u32_at(self.data, o + 32)?)?;
            if n > self.data.len() / 4 {
                return Err("export name count exceeds image".into());
            }
            for i in 0..n {
                names.insert(self.cstr(u32_at(self.data, table + 4 * i)?)?);
            }
        }
        Ok(names)
    }
    fn imports(&self) -> Result<Imports, String> {
        let Some(&(rva, _)) = self.dirs.get(1) else {
            return Ok(Imports::new());
        };
        if rva == 0 {
            return Ok(Imports::new());
        }
        let mut o = self.off(rva)?;
        let mut result = Imports::new();
        loop {
            let ilt = u32_at(self.data, o)?;
            let name = u32_at(self.data, o + 12)?;
            let iat = u32_at(self.data, o + 16)?;
            if name == 0 {
                break;
            }
            let names = result.entry(self.cstr(name)?).or_insert_with(Vec::new);
            let mut t = self.off(if ilt != 0 { ilt } else { iat })?;
            loop {
                let thunk = if self.wide {
                    u64_at(self.data, t)?
                } else {
                    u64::from(u32_at(self.data, t)?)
                };
                if thunk == 0 {
                    break;
                }
                let ordinal = 1u64 << if self.wide { 63 } else { 31 };
                if thunk & ordinal == 0 {
                    names.push(self.cstr((thunk & 0x7fffffff) as u32 + 2)?);
                }
                t += if self.wide { 8 } else { 4 };
            }
            o += 20;
        }
        Ok(result)
    }
}
fn corpus(build: &Path) -> Result<BTreeMap<String, PathBuf>, String> {
    let list = build.join("boot/livecd.Debug.lst");
    let text = fs::read_to_string(&list).map_err(|_| {
        format!(
            "scan_unresolved_imports: {} missing (build livecd first)",
            list.display()
        )
    })?;
    let canon = format!(
        "/tmp/vibeos/{}/",
        build.file_name().unwrap_or_default().to_string_lossy()
    );
    let mut files = BTreeMap::new();
    for line in text.lines() {
        let Some((dest, src)) = line.split_once('=') else {
            continue;
        };
        if ![".sys", ".dll", ".exe", ".drv", ".cpl", ".ocx", ".acm"]
            .iter()
            .any(|s| dest.to_lowercase().ends_with(s))
        {
            continue;
        }
        let p = if let Some(s) = src.strip_prefix(&canon) {
            build.join(s)
        } else {
            PathBuf::from(src)
        };
        if p.is_file() {
            files
                .entry(
                    Path::new(dest)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_lowercase(),
                )
                .or_insert(p);
        }
    }
    Ok(files)
}
fn resolve(
    imports: &BTreeMap<String, Option<Imports>>,
    exports: &BTreeMap<String, Option<Names>>,
) -> Vec<String> {
    let mut findings = Vec::new();
    for (name, table) in imports {
        let Some(table) = table else {
            if name != "freeldr.sys" {
                findings.push(format!("UNREADABLE {name}"))
            }
            continue;
        };
        for (dll, names) in table {
            let key = dll.to_lowercase();
            if key == "freeldr.sys" {
                continue;
            }
            let variants = match key.as_str() {
                "ntoskrnl.exe" => vec!["ntoskrnl.exe", "ntkrnlmp.exe"],
                "hal.dll" => vec!["hal.dll", "halmp.dll"],
                _ => vec![key.as_str()],
            };
            let present: Vec<_> = variants
                .into_iter()
                .filter(|v| exports.contains_key(*v))
                .collect();
            if present.is_empty() {
                findings.push(format!("MISSING_DLL {name} -> {dll}"));
                continue;
            }
            for v in present {
                let Some(exp) = &exports[v] else {
                    findings.push(format!("UNREADABLE {v} (imported by {name})"));
                    continue;
                };
                for sym in names.iter().collect::<BTreeSet<_>>() {
                    if !exp.contains(sym) {
                        findings.push(format!("UNRESOLVED {name} -> {v}!{sym}"))
                    }
                }
            }
        }
    }
    findings
}
pub fn run(ctx: &BinaryContext) -> Result<CheckOutput, String> {
    let build = fs::canonicalize(&ctx.build_dir).map_err(|e| e.to_string())?;
    let mut imports = BTreeMap::new();
    let mut exports = BTreeMap::new();
    for (name, path) in corpus(&build)? {
        let data = fs::read(path).map_err(|e| e.to_string())?;
        match Pe::new(&data) {
            Ok(pe) => {
                imports.insert(name.clone(), pe.imports().ok());
                exports.insert(name, pe.exports().ok());
            }
            Err(_) => {
                imports.insert(name.clone(), None);
                exports.insert(name, None);
            }
        }
    }
    let findings = resolve(&imports, &exports);
    let mut out = String::new();
    for f in &findings {
        out += f;
        out.push('\n');
    }
    out += &format!(
        "scan_unresolved_imports[{}]: {} findings\n",
        build.file_name().unwrap_or_default().to_string_lossy(),
        findings.len()
    );
    Ok(CheckOutput::new(out, i32::from(!findings.is_empty())))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn kernel_variants() {
        let imports = BTreeMap::from([(
            "pcix.sys".into(),
            Some(BTreeMap::from([(
                "NTOSKRNL.EXE".into(),
                vec!["RtlAssert@16".into(), "RtlAssert".into()],
            )])),
        )]);
        let exports = BTreeMap::from([
            (
                "ntoskrnl.exe".into(),
                Some(Names::from(["RtlAssert".into()])),
            ),
            (
                "ntkrnlmp.exe".into(),
                Some(Names::from(["RtlAssert".into()])),
            ),
        ]);
        assert_eq!(
            resolve(&imports, &exports),
            vec![
                "UNRESOLVED pcix.sys -> ntoskrnl.exe!RtlAssert@16",
                "UNRESOLVED pcix.sys -> ntkrnlmp.exe!RtlAssert@16"
            ]
        );
    }
    #[test]
    fn unreadable_and_loader() {
        let imports = BTreeMap::from([
            (
                "a.dll".into(),
                Some(BTreeMap::from([("gone.dll".into(), vec!["X".into()])])),
            ),
            ("b.dll".into(), None),
            ("freeldr.sys".into(), None),
            (
                "rosload.exe".into(),
                Some(BTreeMap::from([(
                    "freeldr.sys".into(),
                    vec!["MmAllocateMemory".into()],
                )])),
            ),
        ]);
        assert_eq!(
            resolve(&imports, &BTreeMap::new()),
            vec!["MISSING_DLL a.dll -> gone.dll", "UNREADABLE b.dll"]
        );
    }
    #[test]
    fn pe32_and_pe64_names_and_ordinals() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/data/binary-pe.json")).unwrap();
        for name in ["pe32", "pe64"] {
            let bytes: Vec<u8> = data[name]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap() as u8)
                .collect();
            let pe = Pe::new(&bytes).unwrap();
            assert_eq!(pe.exports().unwrap(), Names::from(["A".into(), "B".into()]));
            assert_eq!(
                pe.imports().unwrap(),
                BTreeMap::from([(
                    "ntoskrnl.exe".into(),
                    vec!["RtlAssert@16".into(), "IoCreateDevice".into()]
                )])
            );
            for n in 0..bytes.len() {
                let Ok(short) = Pe::new(&bytes[..n]) else {
                    continue;
                };
                let _ = short.exports();
                let _ = short.imports();
            }
        }
    }
    #[test]
    fn truncated_headers_are_errors() {
        for n in 0..512 {
            let data = vec![0; n];
            assert!(Pe::new(&data).is_err());
        }
    }
}
