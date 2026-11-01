//! Native implementation of sdk/tools/gen_baseaddress.py.
use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

const PRIORITIES: &str = include_str!("priorities.txt");
const EXCLUDE: &str = include_str!("exclude.txt");
const EXTENSIONS: &str = include_str!("all_extensions.txt");

#[derive(Debug)]
pub struct Generated {
    /// Destination basename under sdk/cmake.
    pub target_file: &'static str,
    pub content: String,
    /// Ordered missing-priority diagnostics, excluding the initial "# ".
    pub missing: Vec<String>,
}
#[derive(Debug, Clone)]
struct PeInfo {
    size: u64,
    sections: Vec<Vec<u8>>,
}
fn read_at(file: &mut File, offset: u64, len: usize, path: &Path) -> Result<Vec<u8>, String> {
    let mut bytes = vec![0; len];
    file.seek(SeekFrom::Start(offset))
        .and_then(|_| file.read_exact(&mut bytes))
        .map_err(|e| {
            format!(
                "{}: invalid or truncated PE at 0x{offset:x}: {e}",
                path.display()
            )
        })?;
    Ok(bytes)
}
fn u16_at(bytes: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([bytes[off], bytes[off + 1]])
}
fn u32_at(bytes: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([bytes[off], bytes[off + 1], bytes[off + 2], bytes[off + 3]])
}
fn pe(path: &Path, sections: bool) -> Result<PeInfo, String> {
    let mut file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let dos = read_at(&mut file, 0, 64, path)?;
    if &dos[..2] != b"MZ" {
        return Err(format!("{}: no DOS header", path.display()));
    }
    let off = u32_at(&dos, 0x3c) as u64;
    let nt = read_at(&mut file, off, 24, path)?;
    if &nt[..4] != b"PE\0\0" {
        return Err(format!("{}: no PE header", path.display()));
    }
    let n = u16_at(&nt, 6) as usize;
    let optional_size = u16_at(&nt, 20) as usize;
    if optional_size < 60 {
        return Err(format!("{}: PE optional header too short", path.display()));
    }
    let opt = read_at(&mut file, off + 24, optional_size, path)?;
    if !matches!(u16_at(&opt, 0), 0x10b | 0x20b) {
        return Err(format!("{}: unknown executable format", path.display()));
    }
    let size = u32_at(&opt, 56);
    if size == 0 || size > i32::MAX as u32 {
        return Err(format!("{}: invalid PE SizeOfImage {size}", path.display()));
    }
    let mut names = Vec::new();
    if sections {
        let data = read_at(&mut file, off + 24 + optional_size as u64, n * 40, path)?;
        for section in data.chunks_exact(40) {
            let mut name = &section[..8];
            while name.first() == Some(&0) {
                name = &name[1..];
            }
            while name.last() == Some(&0) {
                name = &name[..name.len() - 1];
            }
            names.push(name.to_vec());
        }
    }
    Ok(PeInfo {
        size: size as u64,
        sections: names,
    })
}
#[derive(Clone, Debug)]
struct Module {
    name: String,
    filename: PathBuf,
    address: u64,
    size: u64,
    reserved: bool,
}
fn scan(dir: &Path, modules: &mut HashMap<String, Module>, is_64: bool) -> Result<(), String> {
    let mut files = Vec::new();
    let mut dirs = Vec::new();
    for e in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let e = e.map_err(|e| e.to_string())?;
        let ty = e.file_type().map_err(|e| e.to_string())?;
        if ty.is_dir() {
            dirs.push(e.path());
        } else if ty.is_file() || (ty.is_symlink() && e.path().is_file()) {
            files.push(e.path());
        }
    }
    for path in files {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            return Err(format!("{}: non-UTF-8 filename", path.display()));
        };
        if !EXTENSIONS.lines().any(|ext| name.ends_with(ext))
            || EXCLUDE.lines().any(|e| e == name)
            || name.starts_with("api-ms-win-")
        {
            continue;
        }
        let info = pe(&path, false)?; // Validate even duplicates rather than silently accepting corrupt input.
        let reserved = !is_64 && name == "user32.dll";
        modules.entry(name.to_owned()).or_insert(Module {
            name: name.to_owned(),
            filename: path.clone(),
            address: if reserved { 0x77a20000 } else { 0 },
            size: if is_64 { 0xfe0000 } else { info.size },
            reserved,
        });
    }
    for path in dirs {
        scan(&path, modules, is_64)?;
    }
    Ok(())
}
fn next_address(
    start: &mut u64,
    initial: u64,
    size: u64,
    reserved: &mut Option<(u64, u64)>,
) -> Result<u64, String> {
    loop {
        let current = if *start == 0 {
            *start = initial;
            initial
        } else {
            let v = start
                .checked_sub(size)
                .and_then(|v| v.checked_sub(0x2000 + 0xffff))
                .ok_or("baseaddress allocation underflow")?
                & !0xffff;
            *start = v;
            v
        };
        let end = current
            .checked_add(size + 0x2000)
            .ok_or("baseaddress allocation overflow")?;
        if let Some((res_start, res_size)) = *reserved {
            let res_end = res_start + res_size + 0x2000;
            if (res_start <= current && current <= res_end)
                || (res_start <= end && end <= res_end)
                || (current < res_start && end > res_end)
            {
                *start = res_start.min(current);
                *reserved = None;
                continue;
            }
        }
        if current == 0 {
            return Err("baseaddress allocation reached zero".into());
        }
        return Ok(current);
    }
}
/// Return output and ordered diagnostics; no files are written.
pub fn generate(target: &Path, is_64: bool) -> Result<Generated, String> {
    let mut found = HashMap::new();
    scan(target, &mut found, is_64)?;
    let ntdll = found
        .get("ntdll.dll")
        .ok_or_else(|| format!("{}: ntdll.dll not found", target.display()))?;
    let nt = pe(&ntdll.filename, true)?;
    let slashes = nt.sections.iter().flatten().filter(|c| **c == b'/').count();
    let target_file = if is_64 {
        "baseaddress64.cmake"
    } else if nt.sections.iter().any(|s| s == b".rossym") {
        "baseaddress.cmake"
    } else if slashes == 0 {
        "baseaddress_msvc.cmake"
    } else if slashes > 3 {
        "baseaddress_dwarf.cmake"
    } else {
        return Err(format!(
            "{}: unknown ntdll section layout",
            ntdll.filename.display()
        ));
    };
    let initial = if is_64 { 0x7ffb7000000 } else { 0x7c920000 };
    let mut reserved = if is_64 {
        None
    } else {
        let module = found
            .get("user32.dll")
            .ok_or_else(|| "reserved module user32.dll not found".to_owned())?;
        Some((module.address, module.size))
    };
    let mut start = 0;
    let mut ordered = Vec::new();
    let mut missing = Vec::new();
    for name in PRIORITIES.lines() {
        if let Some(mut m) = found.remove(name) {
            if m.address == 0 {
                m.address = next_address(&mut start, initial, m.size, &mut reserved)?;
            }
            ordered.push(m)
        } else {
            missing.push(format!("Did not find {name} !"));
        }
    }
    for (_, mut m) in found.into_iter().collect::<BTreeMap<_, _>>() {
        m.address = next_address(&mut start, initial, m.size, &mut reserved)?;
        ordered.push(m);
    }
    let mut content = format!(
        "# Generated from {}\n# Generated by sdk/tools/gen_baseaddress.py\n\n",
        target.display()
    );
    for m in ordered {
        let (stem, ext) = if m.name.starts_with('.') && !m.name[1..].contains('.') {
            (m.name.as_str(), "")
        } else {
            m.name.rsplit_once('.').unwrap()
        };
        let name = if matches!(ext, "acm" | "drv") && m.name != "winspool.drv" {
            m.name.as_str()
        } else {
            stem
        };
        let postfix = if name == "ntdll" {
            format!(" # should be above 0x{:08x}", m.address)
        } else if m.reserved {
            " # reserved".to_owned()
        } else {
            String::new()
        };
        content.push_str(&format!(
            "set(baseaddress_{name:<30} 0x{:08x}){postfix}\n",
            m.address
        ));
    }
    Ok(Generated {
        target_file,
        content,
        missing,
    })
}
