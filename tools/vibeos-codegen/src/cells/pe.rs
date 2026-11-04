//! Bounded PE contract reader used to prove Cargo link conversions.
use super::{Error, Result};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
const DIRS: [&str; 16] = [
    "export",
    "import",
    "resource",
    "exception",
    "certificate",
    "basereloc",
    "debug",
    "architecture",
    "globalptr",
    "tls",
    "loadconfig",
    "boundimport",
    "iat",
    "delayimport",
    "clr",
    "reserved",
];
#[derive(Clone, Debug, serde::Serialize)]
pub struct Pe {
    pub data: Vec<u8>,
    pub machine: u16,
    pub pe64: bool,
    pub image_base: u64,
    pub characteristics: u16,
    pub sect_align: u32,
    pub file_align: u32,
    pub os_major: u16,
    pub os_minor: u16,
    pub sub_major: u16,
    pub sub_minor: u16,
    pub subsystem: u16,
    pub dll_characteristics: u16,
    pub stack: [u64; 4],
    pub dirs: BTreeMap<String, (u32, u32)>,
    pub sections: Vec<(String, u32, u32, u32, u32, u32)>,
}
fn bytes(d: &[u8], at: usize, n: usize) -> Result<&[u8]> {
    d.get(at..at.checked_add(n).ok_or("PE offset overflow")?)
        .ok_or_else(|| Error::new("truncated PE data"))
}
fn u16at(d: &[u8], at: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(bytes(d, at, 2)?.try_into().unwrap()))
}
fn u32at(d: &[u8], at: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(bytes(d, at, 4)?.try_into().unwrap()))
}
fn u64at(d: &[u8], at: usize) -> Result<u64> {
    Ok(u64::from_le_bytes(bytes(d, at, 8)?.try_into().unwrap()))
}
fn digest(d: &[u8]) -> String {
    format!("{:x}", Sha256::digest(d))[..16].to_owned()
}
fn latin(d: &[u8]) -> String {
    d.iter().map(|b| char::from(*b)).collect()
}
impl Pe {
    pub fn parse(data: Vec<u8>) -> Result<Self> {
        if bytes(&data, 0, 2)? != b"MZ" {
            return Err("no MZ header".into());
        }
        let p = u32at(&data, 0x3c)? as usize;
        if bytes(&data, p, 4)? != b"PE\0\0" {
            return Err("no PE signature".into());
        }
        let machine = u16at(&data, p + 4)?;
        let ns = u16at(&data, p + 6)? as usize;
        let optsz = u16at(&data, p + 20)? as usize;
        let characteristics = u16at(&data, p + 22)?;
        let o = p + 24;
        bytes(&data, o, optsz)?;
        let magic = u16at(&data, o)?;
        if ![0x10b, 0x20b].contains(&magic) {
            return Err("unknown PE optional header magic".into());
        }
        let pe64 = magic == 0x20b;
        if optsz < if pe64 { 112 } else { 96 } {
            return Err("truncated PE optional header".into());
        }
        let image_base = if pe64 {
            u64at(&data, o + 24)?
        } else {
            u32at(&data, o + 28)? as u64
        };
        let b = o + 32;
        let mut stack = [0; 4];
        for (i, s) in stack.iter_mut().enumerate() {
            *s = if pe64 {
                u64at(&data, b + 40 + i * 8)?
            } else {
                u32at(&data, b + 40 + i * 4)? as u64
            };
        }
        let nd = b + 40 + if pe64 { 36 } else { 20 };
        let n = u32at(&data, nd)?.min(16) as usize;
        if (nd - o) + 4 + n * 8 > optsz {
            return Err("data directories exceed PE optional header".into());
        }
        let mut dirs = BTreeMap::new();
        for (i, name) in DIRS.iter().take(n).enumerate() {
            dirs.insert(
                (*name).into(),
                (u32at(&data, nd + 4 + 8 * i)?, u32at(&data, nd + 8 + 8 * i)?),
            );
        }
        let mut sections = Vec::new();
        for i in 0..ns {
            let s = o + optsz + 40 * i;
            let name = bytes(&data, s, 8)?;
            let end = name
                .iter()
                .rposition(|b| *b != 0)
                .map(|i| i + 1)
                .unwrap_or(0);
            sections.push((
                latin(&name[..end]),
                u32at(&data, s + 12)?,
                u32at(&data, s + 8)?,
                u32at(&data, s + 20)?,
                u32at(&data, s + 16)?,
                u32at(&data, s + 36)?,
            ));
        }
        Ok(Self {
            machine,
            pe64,
            image_base,
            characteristics,
            sect_align: u32at(&data, b)?,
            file_align: u32at(&data, b + 4)?,
            os_major: u16at(&data, b + 8)?,
            os_minor: u16at(&data, b + 10)?,
            sub_major: u16at(&data, b + 16)?,
            sub_minor: u16at(&data, b + 18)?,
            subsystem: u16at(&data, b + 36)?,
            dll_characteristics: u16at(&data, b + 38)?,
            stack,
            dirs,
            sections,
            data,
        })
    }
    pub fn dir(&self, name: &str) -> (u32, u32) {
        self.dirs.get(name).copied().unwrap_or((0, 0))
    }
    pub fn off(&self, rva: u32) -> Result<usize> {
        for (_, va, vs, rp, rs, _) in &self.sections {
            if rva >= *va && (rva as u64) < *va as u64 + (*vs).max(*rs) as u64 {
                return Ok(*rp as usize + (rva - *va) as usize);
            }
        }
        Err(format!("RVA {rva:#x} is in no section").into())
    }
    pub fn cstr(&self, rva: u32) -> Result<String> {
        let p = self.off(rva)?;
        let tail = bytes(
            &self.data,
            p,
            self.data.len().checked_sub(p).ok_or("RVA beyond image")?,
        )?;
        let n = tail
            .iter()
            .position(|b| *b == 0)
            .ok_or("unterminated PE string")?;
        Ok(latin(&tail[..n]))
    }
    fn export_entries(&self) -> Result<Vec<(u32, String, u32)>> {
        let (rva, _) = self.dir("export");
        if rva == 0 {
            return Ok(vec![]);
        }
        let o = self.off(rva)?;
        bytes(&self.data, o, 40)?;
        let base = u32at(&self.data, o + 16)?;
        let nf = u32at(&self.data, o + 20)? as usize;
        let nn = u32at(&self.data, o + 24)? as usize;
        let fun = self.off(u32at(&self.data, o + 28)?)?;
        let names = self.off(u32at(&self.data, o + 32)?)?;
        let ord = self.off(u32at(&self.data, o + 36)?)?;
        bytes(
            &self.data,
            fun,
            nf.checked_mul(4).ok_or("export count overflow")?,
        )?;
        bytes(
            &self.data,
            names,
            nn.checked_mul(4).ok_or("export names overflow")?,
        )?;
        bytes(
            &self.data,
            ord,
            nn.checked_mul(2).ok_or("export ordinals overflow")?,
        )?;
        let mut named = BTreeMap::new();
        for i in 0..nn {
            named.insert(
                u16at(&self.data, ord + 2 * i)? as usize,
                self.cstr(u32at(&self.data, names + 4 * i)?)?,
            );
        }
        let mut out = Vec::new();
        for i in 0..nf {
            let f = u32at(&self.data, fun + 4 * i)?;
            if f != 0 {
                out.push((
                    base.checked_add(i as u32).ok_or("ordinal overflow")?,
                    named.get(&i).cloned().unwrap_or_else(|| "(noname)".into()),
                    f,
                ));
            }
        }
        Ok(out)
    }
    pub fn exports(&self) -> Result<Vec<String>> {
        let (r, s) = self.dir("export");
        self.export_entries()?
            .into_iter()
            .map(|(ord, n, f)| {
                let forward = if f >= r && (f as u64) < r as u64 + s as u64 {
                    format!(" -> {}", self.cstr(f)?)
                } else {
                    String::new()
                };
                Ok(format!("@{ord} {n}{forward}"))
            })
            .collect()
    }
    pub fn module_name(&self) -> Result<Option<String>> {
        let (r, _) = self.dir("export");
        if r == 0 {
            Ok(None)
        } else {
            Ok(Some(self.cstr(u32at(&self.data, self.off(r)? + 12)?)?))
        }
    }
    fn thunks(&self, rva: u32) -> Result<Vec<String>> {
        let mut o = self.off(rva)?;
        let mut out = Vec::new();
        loop {
            let v = if self.pe64 {
                u64at(&self.data, o)?
            } else {
                u32at(&self.data, o)? as u64
            };
            if v == 0 {
                break;
            }
            let flag = 1u64 << if self.pe64 { 63 } else { 31 };
            out.push(if v & flag != 0 {
                format!("#{}", v & 0xffff)
            } else {
                self.cstr(
                    ((v & 0x7fff_ffff) as u32)
                        .checked_add(2)
                        .ok_or("import RVA overflow")?,
                )?
            });
            o += if self.pe64 { 8 } else { 4 };
        }
        Ok(out)
    }
    pub fn imports(&self) -> Result<Vec<String>> {
        let mut out = Vec::new();
        let (r, _) = self.dir("import");
        if r != 0 {
            let mut o = self.off(r)?;
            loop {
                bytes(&self.data, o, 20)?;
                let ilt = u32at(&self.data, o)?;
                let name = u32at(&self.data, o + 12)?;
                let iat = u32at(&self.data, o + 16)?;
                if name == 0 {
                    break;
                }
                let dll = self.cstr(name)?.to_lowercase();
                for n in self.thunks(if ilt != 0 { ilt } else { iat })? {
                    out.push(format!("{dll}!{n}"));
                }
                o += 20;
            }
        }
        let (r, _) = self.dir("delayimport");
        if r != 0 {
            let mut o = self.off(r)?;
            loop {
                bytes(&self.data, o, 32)?;
                let name = u32at(&self.data, o + 4)?;
                if name == 0 {
                    break;
                }
                let dll = self.cstr(name)?.to_lowercase();
                for n in self.thunks(u32at(&self.data, o + 16)?)? {
                    out.push(format!("delay {dll}!{n}"));
                }
                o += 32;
            }
        }
        out.sort();
        Ok(out)
    }
    pub fn resources(&self) -> Result<Vec<String>> {
        let (r, _) = self.dir("resource");
        if r == 0 {
            return Ok(vec![]);
        }
        let root = self.off(r)?;
        let mut out = Vec::new();
        self.resource_walk(root, root, &[], &mut BTreeSet::new(), &mut out)?;
        out.sort();
        Ok(out)
    }
    fn resource_walk(
        &self,
        root: usize,
        at: usize,
        path: &[String],
        seen: &mut BTreeSet<usize>,
        out: &mut Vec<String>,
    ) -> Result<()> {
        if path.len() > 64 || !seen.insert(at) {
            return Err("cyclic or overly deep resource tree".into());
        }
        let n = u16at(&self.data, at + 12)? as usize + u16at(&self.data, at + 14)? as usize;
        bytes(&self.data, at, 16 + n * 8)?;
        for i in 0..n {
            let name = u32at(&self.data, at + 16 + i * 8)?;
            let target = u32at(&self.data, at + 20 + i * 8)?;
            let name = if name & 0x80000000 == 0 {
                format!("#{name}")
            } else {
                let o = root + (name & 0x7fffffff) as usize;
                let n = u16at(&self.data, o)? as usize;
                let raw = bytes(&self.data, o + 2, n * 2)?;
                let utf = raw
                    .chunks_exact(2)
                    .map(|b| u16::from_le_bytes([b[0], b[1]]))
                    .collect::<Vec<_>>();
                String::from_utf16(&utf).map_err(|e| Error::new(e.to_string()))?
            };
            let mut next = path.to_vec();
            next.push(name);
            if target & 0x80000000 != 0 {
                self.resource_walk(
                    root,
                    root + (target & 0x7fffffff) as usize,
                    &next,
                    seen,
                    out,
                )?;
            } else {
                let o = root + target as usize;
                let r = u32at(&self.data, o)?;
                let n = u32at(&self.data, o + 4)? as usize;
                let blob = bytes(&self.data, self.off(r)?, n)?;
                out.push(format!("{} {n} {}", next.join("/"), digest(blob)));
            }
        }
        seen.remove(&at);
        Ok(())
    }
    pub fn mapped_size(&self) -> u64 {
        self.sections
            .iter()
            .filter(|s| s.5 & 0x02000000 == 0)
            .map(|s| s.2.max(s.4) as u64)
            .sum()
    }
    pub fn loaded(&self) -> Result<Vec<String>> {
        let mut out = Vec::new();
        for (n, va, vs, rp, rs, c) in &self.sections {
            if (c & 0x02000000 != 0 && (n.starts_with('/') || n.starts_with(".debug")))
                || n == ".buildid"
            {
                continue;
            }
            let b = if *rp == 0 {
                &[][..]
            } else {
                bytes(&self.data, *rp as usize, (*vs).min(*rs) as usize)?
            };
            out.push(format!("{n} {va:#x}+{vs:#x} {}", digest(b)));
        }
        Ok(out)
    }
    pub fn contract(&self) -> Result<Vec<(String, Vec<String>)>> {
        let header = vec![
            format!("machine {:#x}", self.machine),
            format!("image_base {:#x}", self.image_base),
            format!(
                "alignment section={:#x} file={:#x}",
                self.sect_align, self.file_align
            ),
            format!("os {}.{}", self.os_major, self.os_minor),
            format!("subsystem_version {}.{}", self.sub_major, self.sub_minor),
            format!("subsystem {}", self.subsystem),
            format!("characteristics {:#x}", self.characteristics),
            format!("dll_characteristics {:#x}", self.dll_characteristics),
            format!(
                "stack/heap {}",
                self.stack
                    .iter()
                    .map(|s| format!("{s:#x}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            format!(
                "directories {}",
                [
                    "export",
                    "import",
                    "resource",
                    "tls",
                    "loadconfig",
                    "delayimport"
                ]
                .into_iter()
                .filter(|n| self.dir(n).0 != 0)
                .collect::<Vec<_>>()
                .join(" ")
            ),
        ];
        let sections = self
            .sections
            .iter()
            .filter(|s| {
                !(s.5 & 0x02000000 != 0 && (s.0.starts_with('/') || s.0.starts_with(".debug")))
            })
            .map(|s| format!("{} {:#x}", s.0, s.5))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let mut exports = self.exports()?;
        if let Some(n) = self.module_name()? {
            exports.insert(0, format!("module {}", n.to_lowercase()));
        }
        Ok(vec![
            ("header".into(), header),
            ("exports".into(), exports),
            ("imports".into(), self.imports()?),
            ("rsrc".into(), self.resources()?),
            ("sections".into(), sections),
        ])
    }
}
pub fn load(path: &Path) -> Result<Pe> {
    Pe::parse(std::fs::read(path)?).map_err(|e| Error::new(format!("{}: {e}", path.display())))
}
pub fn compare(a: &Pe, b: &Pe, code: bool) -> Result<Vec<String>> {
    let (mut ca, mut cb) = (a.contract()?, b.contract()?);
    if code {
        ca.push(("loaded".into(), a.loaded()?));
        cb.push(("loaded".into(), b.loaded()?));
    }
    let mut out = Vec::new();
    for ((key, old), (_, new)) in ca.into_iter().zip(cb) {
        let sa = old.into_iter().collect::<BTreeSet<_>>();
        let sb = new.into_iter().collect::<BTreeSet<_>>();
        for x in sa.difference(&sb) {
            out.push(format!("{key}: - {x}"));
        }
        for x in sb.difference(&sa) {
            out.push(format!("{key}: + {x}"));
        }
    }
    Ok(out)
}
pub fn export_is_import_thunk(code: &[u8], machine: u16) -> bool {
    if machine == 0xaa64 {
        code.get(..4)
            .is_some_and(|b| u32::from_le_bytes(b.try_into().unwrap()) & 0x9f00001f == 0x90000010)
    } else {
        code.starts_with(&[0xff, 0x25])
    }
}
pub fn exports_bound_to_imports(path: &Path) -> Result<BTreeSet<String>> {
    let pe = load(path)?;
    let imported = pe
        .imports()?
        .into_iter()
        .filter_map(|s| s.split_once('!').map(|(_, n)| n.to_owned()))
        .collect::<BTreeSet<_>>();
    let (r, size) = pe.dir("export");
    let mut out = BTreeSet::new();
    for (_, n, f) in pe.export_entries()? {
        if !imported.contains(&n) || (f >= r && (f as u64) < r as u64 + size as u64) {
            continue;
        }
        let o = pe.off(f)?;
        if export_is_import_thunk(bytes(&pe.data, o, 4)?, pe.machine) {
            out.insert(n);
        }
    }
    Ok(out)
}
pub fn carries_panic_handler(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Err(format!(
            "{}: build the module with cmake first (it decides the handler)",
            path.display()
        )
        .into());
    }
    Ok(load(path)?
        .imports()?
        .iter()
        .any(|s| s.ends_with("!RtlWalkFrameChain")))
}
