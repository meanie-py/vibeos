//! Media mechanisms; payload policy remains owned by the payload compiler.
use crate::{artifact_fs, Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    os::unix::{
        ffi::OsStrExt,
        fs::{symlink, MetadataExt},
    },
    path::{Component, Path, PathBuf},
    process::Command,
    time::Duration,
};

fn absolute(p: &Path) -> Result<PathBuf> {
    let p = if p.is_absolute() {
        p.to_owned()
    } else {
        std::env::current_dir()?.join(p)
    };
    if p.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(Error::new(format!(
            "parent traversal is forbidden: {}",
            p.display()
        )));
    }
    Ok(p)
}
fn atomic(path: &Path, data: &[u8]) -> Result<()> {
    use std::io::Write;
    if path.is_symlink() {
        return Err(Error::new(format!(
            "output is a symlink: {}",
            path.display()
        )));
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    file.write_all(data)?;
    file.as_file().sync_all()?;
    file.persist(path).map_err(|e| Error::new(e.to_string()))?;
    fs::File::open(parent)?.sync_all()?;
    Ok(())
}
pub fn parse_lst(path: &Path, canon: Option<&[String; 2]>) -> Result<Vec<(String, String)>> {
    let mut entries = Vec::new();
    for raw in fs::read_to_string(path)?.lines() {
        let line = raw.trim();
        let Some((dest, source)) = line.split_once('=') else {
            continue;
        };
        let mut source = source.to_owned();
        if let Some([from, to]) = canon {
            if let Some(tail) = source.strip_prefix(from) {
                source = format!("{to}{tail}");
            }
        }
        entries.push((dest.trim_matches('/').into(), source));
    }
    Ok(entries)
}
fn destination(dest: &str) -> Result<()> {
    if dest.contains(['\\', '\0', '\r', '\n'])
        || dest
            .split('/')
            .any(|p| p == "." || p == ".." || p.is_empty())
    {
        return Err(Error::new(format!("unsafe media destination {dest:?}")));
    }
    Ok(())
}
#[derive(Clone)]
struct Source {
    path: PathBuf,
    stamp: [i128; 5],
    digest: Vec<u8>,
    len: u64,
}
impl Source {
    fn read(path: &Path) -> Result<Self> {
        let path = fs::canonicalize(path)?;
        let before = fs::metadata(&path)?;
        if !before.is_file() {
            return Err(Error::new(format!(
                "source is not a regular file: {}",
                path.display()
            )));
        }
        let digest = Sha256::digest(artifact_fs::verified_read(&path, "media input")?).to_vec();
        let stamp = artifact_fs::stamp(&before);
        if stamp != artifact_fs::stamp(&fs::metadata(&path)?) {
            return Err(Error::new("media input changed while reading"));
        }
        Ok(Self {
            path,
            stamp,
            digest,
            len: before.len(),
        })
    }
    fn unchanged(&self) -> Result<()> {
        let now = Self::read(&self.path)?;
        if self.stamp != now.stamp || self.digest != now.digest {
            return Err(Error::new(format!(
                "media input changed: {}",
                self.path.display()
            )));
        }
        Ok(())
    }
}
/// Validate every input and collision before replacing a previously complete tree.
pub fn ramdisk_tree(lists: &[PathBuf], out: &Path, canon: Option<&[String; 2]>) -> Result<u64> {
    if lists.is_empty() {
        return Err("at least one graft list is required".into());
    }
    let out = absolute(out)?;
    if out.parent().is_none() || out.is_symlink() || out == std::env::current_dir()? {
        return Err("unsafe ramdisk output directory".into());
    }
    let mut spelling = BTreeMap::<String, String>::new();
    let mut files = BTreeMap::<String, (String, Source, usize)>::new();
    let mut dirs = BTreeMap::<String, String>::new();
    let mut inventories = Vec::new();
    let mut sources = Vec::new();
    for (index, list) in lists.iter().enumerate() {
        let inventory = Source::read(list)?;
        if inventory.path.starts_with(&out) {
            return Err("ramdisk output contains an input list".into());
        }
        inventories.push(inventory);
        for (dest, src) in parse_lst(list, canon)? {
            let source_path = fs::canonicalize(&src).map_err(|e| {
                Error::new(format!(
                    "{}: {dest}: source missing: {src}: {e}",
                    list.display()
                ))
            })?;
            if source_path.starts_with(&out) {
                return Err("ramdisk output contains a source".into());
            }
            let is_dir = source_path.is_dir();
            if dest.is_empty() && is_dir {
                continue;
            }
            destination(&dest)?;
            let parts: Vec<_> = dest.split('/').collect();
            let mut normalized = Vec::new();
            for (i, part) in parts.iter().enumerate() {
                if i + 1 < parts.len() || is_dir {
                    let key = parts[..=i].join("/").to_lowercase();
                    normalized.push(
                        spelling
                            .entry(key)
                            .or_insert_with(|| part.to_string())
                            .clone(),
                    );
                } else {
                    normalized.push(part.to_string());
                }
            }
            let dest = normalized.join("/");
            for i in 1..normalized.len() {
                let dir = normalized[..i].join("/");
                dirs.entry(dir.to_lowercase()).or_insert(dir);
            }
            if is_dir {
                dirs.entry(dest.to_lowercase()).or_insert(dest);
                continue;
            }
            let source = Source::read(&source_path)?;
            let key = dest.to_lowercase();
            if let Some((_, previous, previous_index)) = files.get(&key) {
                if *previous_index == index && previous.digest != source.digest {
                    return Err(Error::new(format!(
                        "{}: {dest}: named twice with different bytes ({} / {src})",
                        list.display(),
                        previous.path.display()
                    )));
                }
            }
            sources.push(source.clone());
            files.insert(key, (dest, source, index));
        }
    }
    for key in files.keys() {
        if dirs.contains_key(key) {
            return Err(Error::new(format!("media file/directory collision: {key}")));
        }
    }
    let parent = out.parent().ok_or("ramdisk output has no parent")?;
    fs::create_dir_all(parent)?;
    // Resolve the parent before creating staging so aliases cannot select an input tree.
    let resolved =
        fs::canonicalize(parent)?.join(out.file_name().ok_or("ramdisk output has no name")?);
    if inventories
        .iter()
        .chain(&sources)
        .any(|s| s.path.starts_with(&resolved))
    {
        return Err("ramdisk output aliases an input".into());
    }
    let stage = tempfile::Builder::new()
        .prefix(".ramdisk-stage-")
        .tempdir_in(parent)?;
    for dir in dirs.values() {
        fs::create_dir_all(stage.path().join(dir))?;
    }
    let mut total = 0u64;
    for (dest, source, _) in files.values() {
        symlink(&source.path, stage.path().join(dest))?;
        total = total.checked_add(source.len).ok_or("media size overflow")?;
    }
    for source in inventories.iter().chain(&sources) {
        source.unchanged()?;
    }
    let backup = tempfile::Builder::new()
        .prefix(".ramdisk-previous-")
        .tempdir_in(parent)?;
    let previous = backup.path().join("tree");
    if out.exists() {
        if !out.is_dir() {
            return Err("ramdisk output is not a directory".into());
        }
        fs::rename(&out, &previous)?;
    }
    if let Err(e) = fs::rename(stage.path(), &out) {
        if previous.exists() {
            fs::rename(&previous, &out)?;
        }
        return Err(e.into());
    }
    fs::File::open(parent)?.sync_all()?;
    Ok(total)
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Pe {
    pub checksum_offset: usize,
    pub sections: Vec<String>,
}
fn u16_at(data: &[u8], n: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(
        data.get(n..n + 2)
            .ok_or("truncated PE header")?
            .try_into()
            .unwrap(),
    ))
}
fn u32_at(data: &[u8], n: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        data.get(n..n + 4)
            .ok_or("truncated PE header")?
            .try_into()
            .unwrap(),
    ))
}
pub fn parse_pe(data: &[u8]) -> Result<Option<Pe>> {
    if data.len() < 0x40 || &data[..2] != b"MZ" {
        return Ok(None);
    }
    let pe = u32_at(data, 0x3c)? as usize;
    if data.get(pe..pe + 4) != Some(b"PE\0\0") {
        return Ok(None);
    }
    let coff = pe + 4;
    let count = u16_at(data, coff + 2)? as usize;
    let symtab = u32_at(data, coff + 8)? as usize;
    let symbols = u32_at(data, coff + 12)? as usize;
    let opt_size = u16_at(data, coff + 16)? as usize;
    let opt = coff + 20;
    if opt_size < 68 || !matches!(u16_at(data, opt)?, 0x10b | 0x20b) {
        return Err("invalid PE optional header".into());
    }
    let end = opt
        .checked_add(opt_size)
        .and_then(|x| x.checked_add(count.checked_mul(40)?))
        .ok_or("PE section table overflow")?;
    if end > data.len() {
        return Err("truncated PE section table".into());
    }
    let strings = symtab
        .checked_add(
            symbols
                .checked_mul(18)
                .ok_or("COFF string table overflow")?,
        )
        .ok_or("COFF string table overflow")?;
    let mut sections = Vec::new();
    for index in 0..count {
        let offset = opt + opt_size + index * 40;
        let raw = &data[offset..offset + 8];
        let mut name = &raw[..raw.iter().position(|b| *b == 0).unwrap_or(8)];
        if name.starts_with(b"/") && symtab != 0 {
            let n = std::str::from_utf8(&name[1..])
                .ok()
                .and_then(|s| s.parse::<usize>().ok())
                .ok_or("invalid COFF long section name")?;
            let offset = strings.checked_add(n).ok_or("COFF name overflow")?;
            let bytes = data
                .get(offset..)
                .ok_or("COFF section name outside image")?;
            let end = bytes
                .iter()
                .position(|b| *b == 0)
                .ok_or("unterminated COFF section name")?;
            name = &bytes[..end];
        }
        sections.push(name.iter().map(|b| char::from(*b)).collect());
    }
    Ok(Some(Pe {
        checksum_offset: opt + 64,
        sections,
    }))
}
pub fn has_debug(names: &[String]) -> bool {
    names
        .iter()
        .any(|n| n == ".debug" || n.starts_with(".debug_"))
}
pub fn checksum(data: &[u8], offset: usize) -> Result<u32> {
    if offset.checked_add(4).is_none_or(|n| n > data.len()) {
        return Err("PE checksum field outside image".into());
    }
    // Unlike the old Python helper, padding contributes zero words, never file length.
    let mut sum = 0u64;
    for (index, bytes) in data.chunks(2).enumerate() {
        let pos = index * 2;
        let lo = if (offset..offset + 4).contains(&pos) {
            0
        } else {
            bytes[0]
        };
        let hi = if (offset..offset + 4).contains(&(pos + 1)) {
            0
        } else {
            *bytes.get(1).unwrap_or(&0)
        };
        sum += u16::from_le_bytes([lo, hi]) as u64;
        sum = (sum & 0xffff) + (sum >> 16);
    }
    sum = (sum & 0xffff) + (sum >> 16);
    Ok((sum as u32)
        .wrapping_add(u32::try_from(data.len()).map_err(|_| Error::new("PE image exceeds 4GiB"))?))
}
pub fn strip_one(src: &Path, dst: &Path, objcopy: &str) -> Result<()> {
    if absolute(dst)? == fs::canonicalize(src)? || artifact_fs::same_file(src, dst) {
        return Err("stripped output aliases the original build image".into());
    }
    let parent = dst
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let scratch = tempfile::Builder::new()
        .prefix(".strip-")
        .tempdir_in(parent)?;
    let tmp = scratch.path().join("image");
    let output = crate::artifact_executor::supervised_output(
        Command::new(objcopy)
            .arg("--strip-debug")
            .arg(src)
            .arg(&tmp),
        Some(Duration::from_secs(600)),
    )?;
    if output.code != 0 {
        return Err(Error::new(format!(
            "objcopy failed ({}): {}",
            output.code, output.stderr
        )));
    }
    let mut data =
        fs::read(&tmp).map_err(|e| Error::new(format!("objcopy produced no image: {e}")))?;
    let pe = parse_pe(&data)?.ok_or("stripped output is not PE")?;
    if has_debug(&pe.sections) {
        return Err("objcopy left PE debug sections in stripped output".into());
    }
    let checksum = checksum(&data, pe.checksum_offset)?;
    data[pe.checksum_offset..pe.checksum_offset + 4].copy_from_slice(&checksum.to_le_bytes());
    atomic(dst, &data)
}
pub const MANIFEST: &str = "strip-manifest.json";
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Entry {
    mtime_ns: i128,
    size: u64,
    #[serde(default)]
    source_sha256: String,
    #[serde(default)]
    twin_sha256: String,
    #[serde(default)]
    tool_sha256: String,
    #[serde(default)]
    policy_sha256: String,
}
pub fn twin_for(
    src: &Path,
    cache: &Path,
    objcopy: &str,
    manifest: &mut Value,
) -> Result<Option<PathBuf>> {
    use std::io::Read;
    let metadata = fs::metadata(src)?;
    if metadata.is_dir() {
        return Ok(None);
    }
    if !metadata.is_file() {
        return Err("media strip input is not a regular file".into());
    }
    let mut head = [0; 2];
    if fs::File::open(src)?.read(&mut head)? < 2 || &head != b"MZ" {
        return Ok(None);
    }
    let source = Source::read(src)?;
    let data = artifact_fs::verified_read(src, "PE strip input")?;
    let Some(pe) = parse_pe(&data)? else {
        return Ok(None);
    };
    if !has_debug(&pe.sections) {
        return Ok(None);
    }
    let name = format!("{:x}", Sha1::digest(src.as_os_str().as_bytes()));
    let twin = cache.join(format!(
        "{}-{}",
        &name[..12],
        src.file_name()
            .ok_or("strip input has no name")?
            .to_string_lossy()
    ));
    let tool = resolve_program(objcopy)?;
    let tool_digest = format!(
        "{:x}",
        Sha256::digest(artifact_fs::verified_read(&tool, "objcopy")?)
    );
    let source_digest = format!("{:x}", Sha256::digest(&data));
    let policy = format!("{:x}", Sha256::digest(include_bytes!("media_stage.rs")));
    let st = fs::metadata(src)?;
    let ns = st.mtime() as i128 * 1_000_000_000 + st.mtime_nsec() as i128;
    let key = src.to_str().ok_or("non-UTF8 strip source")?;
    let entry = manifest
        .get(key)
        .and_then(|v| serde_json::from_value::<Entry>(v.clone()).ok());
    let cached = if let Some(entry) = entry {
        entry.mtime_ns == ns
            && entry.size == st.len()
            && entry.source_sha256 == source_digest
            && entry.tool_sha256 == tool_digest
            && entry.policy_sha256 == policy
            && twin.is_file()
            && !twin.is_symlink()
            && format!(
                "{:x}",
                Sha256::digest(artifact_fs::verified_read(&twin, "stripped twin")?)
            ) == entry.twin_sha256
    } else {
        false
    };
    let pending = if cached {
        None
    } else {
        let stage = tempfile::Builder::new()
            .prefix(".strip-publish-")
            .tempdir_in(cache)?;
        strip_one(
            src,
            &stage.path().join("image"),
            tool.to_str().ok_or("non-UTF8 objcopy path")?,
        )?;
        Some(stage)
    };
    source.unchanged()?;
    let current_tool = format!(
        "{:x}",
        Sha256::digest(artifact_fs::verified_read(&tool, "objcopy")?)
    );
    if current_tool != tool_digest {
        return Err("objcopy changed during stripping".into());
    }
    if let Some(stage) = pending {
        atomic(&twin, &fs::read(stage.path().join("image"))?)?;
    }
    let twin_digest = format!(
        "{:x}",
        Sha256::digest(artifact_fs::verified_read(&twin, "stripped twin")?)
    );
    manifest
        .as_object_mut()
        .ok_or("strip manifest must be an object")?
        .insert(
            key.into(),
            serde_json::to_value(Entry {
                mtime_ns: ns,
                size: st.len(),
                source_sha256: source_digest,
                twin_sha256: twin_digest,
                tool_sha256: tool_digest,
                policy_sha256: policy,
            })?,
        );
    Ok(Some(twin))
}
fn resolve_program(program: &str) -> Result<PathBuf> {
    if program.contains('/') {
        return Ok(fs::canonicalize(program)?);
    }
    for dir in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        let path = dir.join(program);
        if path.is_file() {
            return Ok(fs::canonicalize(path)?);
        }
    }
    Err(Error::new(format!("missing required objcopy: {program}")))
}
fn read_manifest(cache: &Path) -> Result<Value> {
    match fs::read(cache.join(MANIFEST)) {
        Ok(bytes) => {
            let value: Value = serde_json::from_slice(&bytes)?;
            if !value.is_object() {
                return Err("strip manifest is not an object".into());
            }
            Ok(value)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(json!({})),
        Err(e) => Err(e.into()),
    }
}
/// Drop every `dest=src` line whose destination (case-insensitively, as the guest's FAT/ISO
/// sees it) a payload also names; returns how many were dropped.
fn payload_overrides<'a>(lines: &mut Vec<String>, payload: impl Iterator<Item = &'a str>) -> usize {
    let names: std::collections::HashSet<String> = payload.map(str::to_lowercase).collect();
    let before = lines.len();
    lines.retain(|line| {
        line.split_once('=')
            .is_none_or(|(dest, _)| !names.contains(&dest.to_lowercase()))
    });
    before - lines.len()
}

pub fn strip_cli(repo: &Path, args: &[String]) -> Result<u8> {
    if args.iter().any(|s| s == "--help" || s == "-h") {
        println!("Usage: strip-cd-media (--file FILE | --lst FILE --out FILE) --cache DIR --objcopy TOOL");
        return Ok(0);
    }
    let expanded = option_values(args);
    let args = expanded.as_slice();
    let mut values = BTreeMap::new();
    let mut i = 0;
    while i < args.len() {
        let key = &args[i];
        if !["--lst", "--out", "--file", "--cache", "--objcopy"].contains(&key.as_str()) {
            return Err(Error::new(format!("unknown strip option {key}")));
        }
        let value = args.get(i + 1).ok_or("strip option requires a value")?;
        if values.insert(key.as_str(), value.as_str()).is_some() {
            return Err("duplicate strip option".into());
        }
        i += 2;
    }
    let required = |k| {
        values
            .get(k)
            .copied()
            .ok_or_else(|| Error::new(format!("{k} is required")))
    };
    let cache = Path::new(required("--cache")?);
    let objcopy = required("--objcopy")?;
    if values.contains_key("--file") == values.contains_key("--lst")
        || (values.contains_key("--lst") && !values.contains_key("--out"))
    {
        return Err("choose --file or --lst with --out".into());
    }
    fs::create_dir_all(cache)?;
    let lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(cache.join(".strip.lock"))?;
    if unsafe { libc::flock(std::os::fd::AsRawFd::as_raw_fd(&lock), libc::LOCK_EX) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let mut manifest = read_manifest(cache)?;
    if let Some(file) = values.get("--file") {
        let twin = twin_for(Path::new(file), cache, objcopy, &mut manifest)?;
        atomic(&cache.join(MANIFEST), &serde_json::to_vec(&manifest)?)?;
        println!("{}", twin.as_deref().unwrap_or(Path::new(file)).display());
        return Ok(0);
    }
    let list = Path::new(required("--lst")?);
    let out = Path::new(required("--out")?);
    let inventory = Source::read(list)?;
    let mut lines = Vec::new();
    let mut count = 0;
    for line in fs::read_to_string(list)?.lines() {
        if let Some((dest, src)) = line.split_once('=') {
            if let Some(twin) = twin_for(Path::new(src), cache, objcopy, &mut manifest)? {
                lines.push(format!("{dest}={}", twin.display()));
                count += 1;
                continue;
            }
        }
        lines.push(line.into());
    }
    // The shared native payload compiler owns freshness, foreign-build and collision checks.
    let env: serde_json::Value =
        serde_json::to_value(std::env::vars().collect::<BTreeMap<_, _>>())?;
    let payload = crate::payload_build::media_manifest(repo, out, &env)?;
    let payload = payload
        .as_object()
        .ok_or("payload manifest is not a mapping")?;
    // A payload REPLACES a stock file at its destination. Appending it beside the stock line
    // left two entries for one path: mkisofs kept the stock file under the name and renamed
    // the payload's to `<name>000.exe`, so shlwapiwinetestc's C-bound exe never ran and its
    // "before" run silently tested the Rust; the arm64 ramdisk refused the pair outright.
    let replaced = payload_overrides(&mut lines, payload.keys().map(String::as_str));
    if replaced > 0 {
        println!("strip_cd_media: {replaced} stock entries replaced by payload files");
    }
    for (destination, source) in payload {
        lines.push(format!(
            "{destination}={}",
            source.as_str().ok_or("payload source is not a path")?
        ));
    }
    inventory.unchanged()?;
    atomic(&cache.join(MANIFEST), &serde_json::to_vec(&manifest)?)?;
    atomic(out, (lines.join("\n") + "\n").as_bytes())?;
    println!("strip_cd_media: {count} PE entries staged");
    Ok(0)
}
fn option_values(args: &[String]) -> Vec<String> {
    args.iter()
        .flat_map(|s| {
            if s.starts_with("--") {
                if let Some((key, value)) = s.split_once('=') {
                    return vec![key.into(), value.into()];
                }
            }
            vec![s.clone()]
        })
        .collect()
}
pub fn ramdisk_cli(args: &[String]) -> Result<u8> {
    if args.iter().any(|s| s == "--help" || s == "-h") {
        println!("Usage: ramdisk-tree LIST... --out DIR [--canon FROM TO]");
        return Ok(0);
    }
    let expanded = option_values(args);
    let args = expanded.as_slice();
    let mut lists = Vec::new();
    let mut out = None;
    let mut canon = None;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" => {
                if out.is_some() {
                    return Err("duplicate --out".into());
                }
                out = Some(PathBuf::from(args.next().ok_or("--out requires path")?));
            }
            "--canon" => {
                if canon.is_some() {
                    return Err("duplicate --canon".into());
                }
                canon = Some([
                    args.next().ok_or("--canon requires FROM TO")?.clone(),
                    args.next().ok_or("--canon requires FROM TO")?.clone(),
                ]);
            }
            s if s.starts_with('-') => {
                return Err(Error::new(format!("unknown ramdisk option {s}")))
            }
            _ => lists.push(PathBuf::from(arg)),
        }
    }
    println!(
        "{}",
        ramdisk_tree(&lists, &out.ok_or("--out is required")?, canon.as_ref())?
    );
    Ok(0)
}
pub fn api(request: &Value) -> Result<Value> {
    let a = &request["kwargs"];
    let path = |k: &str| {
        a[k].as_str()
            .map(PathBuf::from)
            .ok_or_else(|| Error::new(format!("missing path {k}")))
    };
    let data = || -> Result<Vec<u8>> { Ok(serde_json::from_value(a["data"].clone())?) };
    match request["op"].as_str().ok_or("missing stage op")? {
        "stage_parse_lst" => Ok(serde_json::to_value(parse_lst(
            &path("path")?,
            serde_json::from_value::<Option<[String; 2]>>(a["canon"].clone())?.as_ref(),
        )?)?),
        "stage_ramdisk_tree" => Ok(json!(ramdisk_tree(
            &serde_json::from_value::<Vec<PathBuf>>(a["lists"].clone())?,
            &path("out")?,
            serde_json::from_value::<Option<[String; 2]>>(a["canon"].clone())?.as_ref()
        )?)),
        "stage_parse_pe" => Ok(match parse_pe(&data()?)? {
            Some(pe) => json!([pe.checksum_offset, pe.sections]),
            None => Value::Null,
        }),
        "stage_has_debug" => Ok(json!(has_debug(&serde_json::from_value::<Vec<String>>(
            a["names"].clone()
        )?))),
        "stage_checksum" => Ok(json!(checksum(
            &data()?,
            a["csum_off"].as_u64().ok_or("missing checksum offset")? as usize
        )?)),
        "stage_strip_one" => {
            strip_one(
                &path("src")?,
                &path("dst")?,
                a["objcopy"].as_str().ok_or("missing objcopy")?,
            )?;
            Ok(Value::Null)
        }
        "stage_twin_for" => {
            let mut manifest = a["manifest"].clone();
            let twin = twin_for(
                &path("src")?,
                &path("cache")?,
                a["objcopy"].as_str().ok_or("missing objcopy")?,
                &mut manifest,
            )?;
            Ok(json!({"twin":twin,"manifest":manifest}))
        }
        _ => Err("unknown media stage operation".into()),
    }
}

#[cfg(test)]
mod payload_override_tests {
    use super::payload_overrides;

    #[test]
    fn a_payload_replaces_the_stock_line_at_its_destination() {
        let mut lines = vec![
            "/tmp/empty".to_owned(),
            "reactos/bin/shlwapi_winetest.exe=/b/stripped/1-shlwapi_winetest.exe".to_owned(),
            "reactos/system32/shlwapi.dll=/b/stripped/2-shlwapi.dll".to_owned(),
        ];
        let n = payload_overrides(
            &mut lines,
            [
                "reactos/bin/SHLWAPI_WINETEST.exe",
                "reactos/bin/shlwapc.dll",
            ]
            .into_iter(),
        );
        assert_eq!(n, 1);
        assert_eq!(
            lines,
            vec![
                "/tmp/empty",
                "reactos/system32/shlwapi.dll=/b/stripped/2-shlwapi.dll"
            ]
        );
    }
}
