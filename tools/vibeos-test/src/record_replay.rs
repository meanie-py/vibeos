//! Deterministic QEMU trace transformations, media receipts and v12 event decoding.
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::VecDeque,
    fs::{self, File},
    io::{BufReader, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
pub const DEFAULT_SHIFT: u64 = 3;
pub const REPLAY_VERSION: u32 = 0xe0200c;
pub const FATAL_MARKER: &str = "[fatal]";
pub type Drive = Vec<(String, String)>;
pub type Media = Vec<(String, Value)>;
fn get<'a>(d: &'a Drive, k: &str) -> Option<&'a str> {
    d.iter().find(|(key, _)| key == k).map(|(_, v)| v.as_str())
}
fn put(d: &mut Drive, k: &str, value: impl Into<String>) {
    let value = value.into();
    if let Some((_, v)) = d.iter_mut().find(|(key, _)| key == k) {
        *v = value;
    } else {
        d.push((k.into(), value));
    }
}
pub fn parse_drive(spec: &str) -> Drive {
    let mut d = Vec::new();
    for s in spec.split(',').filter(|s| !s.is_empty()) {
        let (k, v) = s.split_once('=').unwrap_or(("file", s));
        put(&mut d, k, v);
    }
    d
}
pub fn format_drive(d: &Drive) -> String {
    d.iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(",")
}
pub fn is_x86(argv: &[String]) -> bool {
    argv.first()
        .is_some_and(|v| v.contains("x86_64") || v.contains("i386"))
}
pub fn sha256(path: &Path) -> Result<String> {
    let mut f = File::open(path)?;
    let mut h = Sha256::new();
    let mut b = vec![0; 1 << 20];
    loop {
        let n = f.read(&mut b)?;
        if n == 0 {
            break;
        }
        h.update(&b[..n]);
    }
    Ok(format!("{:x}", h.finalize()))
}
fn checked(mut command: Command) -> Result<String> {
    let output = vibeos_build::process::capture(&mut command, 300)?;
    if output.code != 0 {
        return Err(Error::new(format!(
            "{command:?} failed: {}",
            output.stderr.trim()
        )));
    }
    Ok(output.stdout)
}
pub fn stage_image(src: &Path, dst: &Path, format: &str) -> Result<()> {
    if let Some(p) = dst.parent() {
        fs::create_dir_all(p)?;
    }
    match fs::remove_file(dst) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    if format == "qcow2" {
        fs::copy(src, dst)?;
    } else {
        let mut c = Command::new("qemu-img");
        c.args(["create", "-f", "qcow2", "-F", format, "-b"])
            .arg(src.canonicalize()?)
            .arg(dst);
        checked(c)?;
    }
    Ok(())
}
pub fn transform(
    argv: &[String],
    mode: &str,
    trace: &Path,
    shift: u64,
    stage: bool,
) -> Result<(Vec<String>, Media)> {
    if !matches!(mode, "record" | "replay") {
        return Err(format!("mode must be 'record' or 'replay', got {mode:?}").into());
    }
    let mut out = Vec::new();
    let mut media: Media = Vec::new();
    let mut atapi = std::collections::BTreeSet::new();
    let mut n = 0;
    let mut i = 0;
    while i < argv.len() {
        let a = argv[i].as_str();
        let val = || {
            argv.get(i + 1)
                .map(String::as_str)
                .ok_or_else(|| Error::new(format!("missing value for {a}")))
        };
        match a {
            "-accel"=>{
                let value=val()?;
                if value.starts_with("kvm")||value.starts_with("whpx") {return Err(format!("record/replay needs TCG (instruction counting); got -accel {value}. Hardware virtualization has no instruction counter to anchor the trace to").into())}
                out.extend(["-accel".into(),"tcg,thread=single".into()]);i+=2;
            }
            "-smp"=>{
                let value=val()?.split(',').next().unwrap_or("");
                if value!="1" {return Err(format!("record/replay is single-vCPU only; got -smp {value}. Use --smp 1").into())}
                out.extend(["-smp".into(),"1".into()]);i+=2;
            }
            "-netdev"|"-nic"|"-net"=>return Err(format!("record/replay cannot record network input without replay net filters (got {a} {}). Record a boot config with no NIC",argv.get(i+1).map(String::as_str).unwrap_or("")).into()),
            "-icount"=>return Err("command line already carries -icount; refusing to record on top of someone else's instruction-counting config".into()),
            "-drive"=>{
                let d=parse_drive(val()?);let iface=get(&d,"if").unwrap_or("none");
                let did=get(&d,"id").filter(|v|!v.is_empty()).map(str::to_owned).unwrap_or_else(||format!("rrdrive{n}"));n+=1;
                let src=Path::new(get(&d,"file").ok_or("drive has no file")?);let fmt=get(&d,"format").unwrap_or("raw");
                let stem=src.file_stem().filter(|s|!s.is_empty()).map(|s|s.to_string_lossy().into_owned()).unwrap_or_else(||did.clone());
                let mut name=format!("{stem}.qcow2");if media.iter().any(|(key,_)|key==&name) {name=format!("{stem}-{did}.qcow2");}
                let dst=trace.join("media").join(&name);if stage {stage_image(src,&dst,fmt)?;}
                let digest=if stage&&fmt!="qcow2" {Some(sha256(src)?)}else{None};
                media.push((name,json!({"source":src,"format":fmt,"copied":fmt=="qcow2","sha256":digest})));
                if iface=="pflash" {
                    let mut fw=d.clone();put(&mut fw,"file",dst.to_string_lossy());put(&mut fw,"format","qcow2");fw.retain(|(k,_)|k!="readonly");
                    out.extend(["-drive".into(),format_drive(&fw)]);
                } else {
                    out.extend(["-drive".into(),format!("file={},format=qcow2,if=none,id={did}-img",dst.display()),"-drive".into(),format!("driver=blkreplay,if={iface},image={did}-img,id={did}")]);
                }
                i+=2;
            }
            "-device"=>{
                let spec=val()?;let model=spec.split(',').next().unwrap_or("");
                if matches!(model,"ide-cd"|"scsi-cd") {
                    for token in spec.split(',') {if let Some(id)=token.strip_prefix("drive="){atapi.insert(id.to_owned());}}
                }
                out.push(a.into());i+=1;
            }
            _=>{out.push(a.into());i+=1;}
        }
    }
    if !atapi.is_empty() {
        return Err(format!("cannot record: {} sits behind an ATAPI CD device (`ide-cd`), and QEMU 9.0.2's blkreplay filter stalls that path — the guest never leaves firmware. Re-record with supported media",atapi.into_iter().collect::<Vec<_>>().join(", ")).into());
    }
    if n == 0 {
        return Err("no -drive on the command line — rrsnapshot needs a snapshot-capable image to pin device state into".into());
    }
    if !argv.iter().any(|a| a == "-rtc") {
        out.extend(["-rtc".into(), "clock=vm".into()]);
    }
    out.extend(
        [
            "-nic",
            "user,id=rrnet0",
            "-object",
            "filter-replay,id=rrnetreplay0,netdev=rrnet0",
            "-global",
            "virtio-net-pci.tx=timer",
            "-global",
            "virtio-mmio.ioeventfd=off",
            "-global",
            "virtio-pci.ioeventfd=off",
        ]
        .map(str::to_owned),
    );
    let snap = if is_x86(argv) && mode == "replay" {
        ""
    } else {
        ",rrsnapshot=init"
    };
    out.extend([
        "-icount".into(),
        format!(
            "shift={shift},rr={mode},rrfile={}{snap}",
            trace.join("replay.bin").display()
        ),
    ]);
    Ok((out, media))
}
fn read_u32(f: &mut impl Read) -> Result<Option<u32>> {
    let mut b = [0; 4];
    match f.read_exact(&mut b) {
        Ok(()) => Ok(Some(u32::from_be_bytes(b))),
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(None),
        Err(e) => Err(e.into()),
    }
}
fn skip(f: &mut impl Read, n: u64) -> Result<bool> {
    let mut remain = n;
    let mut b = [0; 8192];
    while remain > 0 {
        let want = remain.min(b.len() as u64) as usize;
        let got = f.read(&mut b[..want])?;
        if got == 0 {
            return Ok(false);
        }
        remain -= got as u64;
    }
    Ok(true)
}
fn async_payload(f: &mut (impl Read + Seek), id: u8, path: &Path) -> Result<bool> {
    match id - 3 {
        0 | 1 | 5 => skip(f, 8),
        3 => Ok(true),
        4 | 6 => {
            if !skip(f, if id == 7 { 1 } else { 5 })? {
                return Ok(false);
            }
            match read_u32(f)? {
                Some(n) => skip(f, n as u64),
                None => Ok(false),
            }
        }
        _ => {
            let Some(kind) = read_u32(f)? else {
                return Ok(false);
            };
            let width = match kind {
                0 => {
                    let Some(key) = read_u32(f)? else {
                        return Ok(false);
                    };
                    match key {
                        0 => 9,
                        1 => 5,
                        _ => {
                            return Err(format!(
                                "{}: input key kind {key} unknown at {}",
                                path.display(),
                                f.stream_position()?
                            )
                            .into())
                        }
                    }
                }
                1 => 5,
                2 | 3 => 12,
                4 => 32,
                _ => {
                    return Err(format!(
                        "{}: input event kind {kind} unknown at {}",
                        path.display(),
                        f.stream_position()?
                    )
                    .into())
                }
            };
            skip(f, width)
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayEvent {
    pub offset: u64,
    pub eid: u8,
    pub icount: u64,
}
pub fn event_name(id: u8) -> String {
    match id {
        0 => "EVENT_INSTRUCTION".into(),
        22 => "EVENT_CHAR_WRITE".into(),
        27 => "EVENT_RANDOM".into(),
        3..=9 => format!("EVENT_ASYNC (id {id})"),
        10..=21 => format!("EVENT_SHUTDOWN (id {id})"),
        30..=38 => format!("EVENT_CHECKPOINT (id {id})"),
        _ => format!("id {id}"),
    }
}
/// Stream the trace once, without allocating variable-length event payloads.
fn walk_from(
    trace: &Path,
    cursor: Option<(u64, u64)>,
    stop_at_shutdown: bool,
    mut emit: impl FnMut(ReplayEvent, u64) -> Result<bool>,
) -> Result<bool> {
    let path = trace.join("replay.bin");
    let mut f = BufReader::new(File::open(&path).map_err(|e| {
        Error::new(format!(
            "{} does not exist — nothing was recorded here: {e}",
            path.display()
        ))
    })?);
    let mut head = [0; 12];
    let got = f.read(&mut head)?;
    if got < 12 {
        return Err(format!("{} is truncated: {got} bytes, no header", path.display()).into());
    }
    let version = u32::from_be_bytes(head[..4].try_into().unwrap());
    if version != REPLAY_VERSION {
        return Err(format!("{} is replay version 0x{version:x}; this decoder only knows 0x{REPLAY_VERSION:x}. Refusing to guess at event sizes",path.display()).into());
    }
    let mut total = 0u64;
    if let Some((offset, count)) = cursor {
        if offset < 12 || offset > f.get_ref().metadata()?.len() {
            return Err("invalid trace cursor".into());
        }
        f.seek(SeekFrom::Start(offset))?;
        total = count;
    }
    loop {
        let offset = f.stream_position()?;
        let mut id = [0];
        if f.read(&mut id)? == 0 {
            return Ok(false);
        }
        let eid = id[0];
        if stop_at_shutdown && (10..22).contains(&eid) {
            return Ok(true);
        }
        let complete = match eid {
            0 => {
                let Some(n) = read_u32(&mut f)? else {
                    return Ok(false);
                };
                total = total
                    .checked_add(n as u64)
                    .ok_or("instruction count overflow")?;
                true
            }
            3..=9 => async_payload(&mut f, eid, &path)?,
            27 => {
                if !skip(&mut f, 4)? {
                    return Ok(false);
                }
                match read_u32(&mut f)? {
                    Some(n) => skip(&mut f, n as u64)?,
                    None => false,
                }
            }
            22 | 28 | 29 => skip(&mut f, 8)?,
            26 => skip(&mut f, 4)?,
            _ => true,
        };
        if !complete {
            return Ok(false);
        }
        let next = f.stream_position()?;
        if !emit(
            ReplayEvent {
                offset,
                eid,
                icount: total,
            },
            next,
        )? {
            return Ok(false);
        }
    }
}
pub(crate) fn walk(
    trace: &Path,
    stop_at_shutdown: bool,
    mut emit: impl FnMut(ReplayEvent) -> Result<bool>,
) -> Result<bool> {
    walk_from(trace, None, stop_at_shutdown, |e, _| emit(e))
}
pub fn events_page(trace: &Path, cursor: Option<(u64, u64)>, limit: usize) -> Result<Value> {
    if limit == 0 || limit > 4096 {
        return Err("event page limit must be 1..4096".into());
    }
    let mut rows = Vec::new();
    let mut next = None;
    walk_from(trace, cursor, false, |e, offset| {
        let count = e.icount;
        rows.push(e);
        if rows.len() >= limit {
            next = Some((offset, count));
            return Ok(false);
        }
        Ok(true)
    })?;
    Ok(json!({"events":rows,"next":next}))
}
pub fn boundaries(trace: &Path) -> Result<Vec<u64>> {
    let mut out = Vec::new();
    walk(trace, false, |e| {
        if e.eid == 0 {
            out.push(e.icount);
        }
        Ok(true)
    })?;
    Ok(out)
}
pub fn provably_complete(trace: &Path) -> bool {
    walk(trace, true, |_| Ok(true)).unwrap_or(false)
}
pub fn snap_break(trace: &Path, want: u64) -> Result<u64> {
    let b = boundaries(trace)?;
    if b.is_empty() {
        return Err(format!(
            "{} records no instruction chunks at all — there is nowhere to break",
            trace.display()
        )
        .into());
    }
    b.iter().find(|n|**n>=want).copied().ok_or_else(||Error::new(format!("icount {want} is past the end of this recording (last boundary {}); a break there would never fire",b.last().unwrap())))
}
pub fn around(
    trace: &Path,
    offset: u64,
    before: usize,
    after: usize,
) -> Result<(Vec<ReplayEvent>, Vec<ReplayEvent>)> {
    let mut prev = VecDeque::new();
    let mut next = Vec::new();
    walk(trace, false, |e| {
        if e.offset < offset {
            prev.push_back(e);
            if prev.len() > before {
                prev.pop_front();
            }
        } else {
            next.push(e);
            if next.len() >= after {
                return Ok(false);
            }
        }
        Ok(true)
    })?;
    Ok((prev.into_iter().collect(), next))
}
fn copy_tree(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir(dst)?;
    for item in fs::read_dir(src)? {
        let item = item?;
        let p = item.path();
        let target = dst.join(item.file_name());
        if p.is_dir() {
            copy_tree(&p, &target)?;
        } else {
            fs::copy(&p, &target)?;
        }
    }
    Ok(())
}
pub fn snapshot_pristine(trace: &Path) -> Result<()> {
    let dst = trace.join("pristine");
    if dst.exists() {
        fs::remove_dir_all(&dst)?;
    }
    copy_tree(&trace.join("media"), &dst)
}
pub fn load_manifest(trace: &Path) -> Result<Value> {
    let path = trace.join("manifest.json");
    if !path.is_file() {
        return Err(format!(
            "no manifest.json in {} — not a vibeos rr trace",
            trace.display()
        )
        .into());
    }
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
pub fn prepare_replay(trace: &Path) -> Result<()> {
    let src = trace.join("pristine");
    let dst = trace.join("media");
    if !src.is_dir() {
        return Err(format!("{} has no pristine/ — it was not produced by a completed recording, so there is nothing to replay from",trace.display()).into());
    }
    if dst.exists() {
        fs::remove_dir_all(&dst)?;
    }
    copy_tree(&src, &dst)?;
    if trace.join("manifest.json").is_file() {
        let m = load_manifest(trace)?;
        let argv: Vec<String> =
            serde_json::from_value(m.get("argv").cloned().unwrap_or(json!([])))?;
        if is_x86(&argv) {
            let mut imgs = fs::read_dir(&dst)?
                .map(|e| e.map(|e| e.path()))
                .collect::<std::io::Result<Vec<_>>>()?;
            imgs.sort();
            for img in imgs
                .into_iter()
                .filter(|p| p.extension().is_some_and(|s| s == "qcow2"))
            {
                let mut c = Command::new("qemu-img");
                c.args(["snapshot", "-a", "init"]).arg(&img);
                checked(c).map_err(|e|Error::new(format!("cannot revert {} to its record-start state (`init`): {e} -- the replay would start from post-recording disks",img.display())))?;
            }
        }
    }
    Ok(())
}
pub fn write_manifest(trace: &Path, a: &Value) -> Result<PathBuf> {
    fs::create_dir_all(trace)?;
    let mut m = json!({"arch":a["arch"],"created":a["created"],"qemu_version":a["qemu_version"],"icount_shift":DEFAULT_SHIFT,"argv":a["argv"],"media":a["media"]});
    if let Some(extra) = a["extra"].as_object() {
        for (k, v) in extra {
            m[k] = v.clone();
        }
    }
    let p = trace.join("manifest.json");
    let mut bytes = serde_json::to_vec_pretty(&m)?;
    bytes.push(b'\n');
    fs::write(&p, bytes)?;
    Ok(p)
}
pub fn note_run(trace: &Path, mode: &str, serial: &str) -> Result<PathBuf> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Error::new(e.to_string()))?
        .as_secs_f64();
    let p = trace.join("last-run.json");
    fs::write(
        &p,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&json!({"mode":mode,"serial_log":serial,"when":now}))?
        ),
    )?;
    Ok(p)
}
pub fn last_run(trace: &Path) -> Result<Value> {
    let p = trace.join("last-run.json");
    if !p.is_file() {
        return Err(format!("{} has no last-run.json — the launch hook did not run, so this boot was not actually recorded or replayed",trace.display()).into());
    }
    Ok(serde_json::from_slice(&fs::read(p)?)?)
}
pub fn host_shape(argv: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < argv.len() {
        let a = &argv[i];
        if ["-serial", "-qmp", "-monitor", "-chardev", "-display"].contains(&a.as_str()) {
            let v = argv.get(i + 1).map(String::as_str).unwrap_or("");
            let kind = v
                .split(':')
                .next()
                .unwrap_or("")
                .split(',')
                .next()
                .unwrap_or("");
            out.push(format!("{a}={kind}"));
            i += 2;
        } else {
            i += 1;
        }
    }
    out
}
pub fn assert_same_shape(trace: &Path, argv: &[String]) -> Result<()> {
    let manifest = load_manifest(trace)?;
    let recorded: Vec<String> =
        serde_json::from_value(manifest.get("argv").cloned().unwrap_or(json!([])))?;
    let drives = |args: &[String]| {
        args.iter()
            .filter(|s| s.starts_with("driver=blkreplay") || s.starts_with("if="))
            .cloned()
            .collect::<Vec<_>>()
    };
    let rec = drives(&recorded);
    let cur = drives(argv);
    if rec != cur {
        return Err(format!("replay device set does not match the recording:\n  recorded: {rec:?}\n  now:      {cur:?}\nSomething changed the boot command line since the recording. Re-record.").into());
    }
    let rec = host_shape(&recorded);
    let cur = host_shape(argv);
    if rec != cur {
        return Err(format!("replay HOST-SIDE shape does not match the recording:\n  recorded: {rec:?}\n  now:      {cur:?}\nThese are backends the guest cannot see; replay through the same harness that recorded rather than a hand-built command line.").into());
    }
    Ok(())
}
pub fn check_trace(trace: &Path, version: &str) -> Result<Vec<String>> {
    let m = load_manifest(trace)?;
    let mut warnings = Vec::new();
    if !version.is_empty()
        && m["qemu_version"]
            .as_str()
            .is_some_and(|v| !v.is_empty() && v != version)
    {
        warnings.push(format!("recorded under {:?}, replaying under {version:?} — device models differ between QEMU versions and the trace may diverge",m["qemu_version"].as_str().unwrap()));
    }
    if let Some(media) = m["media"].as_object() {
        for (name, info) in media {
            if info["copied"].as_bool() == Some(true) {
                continue;
            }
            let src = if let Some(v) = info.as_str() {
                v
            } else {
                info["source"].as_str().unwrap_or("")
            };
            let p = Path::new(src);
            if !p.exists() {
                warnings.push(format!("{FATAL_MARKER} {name} is an overlay over {src}, which is GONE — this trace cannot be replayed. Re-record."));
            } else if let Some(digest) = info["sha256"].as_str().filter(|s| !s.is_empty()) {
                if sha256(p)? != digest {
                    warnings.push(format!("{FATAL_MARKER} {name} is an overlay over {src}, which has CHANGED since the recording (rebuilt media) — the guest would read different bytes than were recorded. Re-record."));
                }
            }
        }
    }
    if m["finalised"].as_bool() != Some(true) && !provably_complete(trace) {
        warnings.push(format!("{FATAL_MARKER} carries no `finalised` marker, and its log does not decode end-to-end onto a shutdown event, so there is no evidence the recording was allowed to finish. Traces made before `rr record` waited for the trace to stop growing are TRUNCATED. Re-record."));
    }
    if fs::metadata(trace.join("replay.bin")).map_or(true, |m| !m.is_file() || m.len() == 0) {
        warnings.push("replay.bin is missing or empty — the recording did not complete".into());
    }
    Ok(warnings)
}
pub fn trace_position(trace: &Path) -> Option<u64> {
    let wanted = trace.join("replay.bin").canonicalize().ok()?;
    for pid in fs::read_dir("/proc").ok()?.flatten() {
        if !pid
            .file_name()
            .to_string_lossy()
            .chars()
            .all(|c| c.is_ascii_digit())
        {
            continue;
        }
        let Ok(fds) = fs::read_dir(pid.path().join("fd")) else {
            continue;
        };
        for fd in fds.flatten() {
            if fs::read_link(fd.path()).ok().as_ref() != Some(&wanted) {
                continue;
            }
            if let Ok(info) = fs::read_to_string(pid.path().join("fdinfo").join(fd.file_name())) {
                for line in info.lines() {
                    if let Some(pos) = line
                        .strip_prefix("pos:")
                        .and_then(|s| s.trim().parse().ok())
                    {
                        return Some(pos);
                    }
                }
            }
        }
    }
    None
}
pub fn env_shift(raw: &str) -> Result<u64> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(DEFAULT_SHIFT);
    }
    let n: i64 = raw.parse().map_err(|_| {
        Error::new(format!(
            "VIBEOS_QEMU_RR_SHIFT must be an integer, got {raw:?}"
        ))
    })?;
    if !(0..=10).contains(&n) {
        return Err(format!("VIBEOS_QEMU_RR_SHIFT must be 0..10, got {n}").into());
    }
    Ok(n as u64)
}
pub fn env_break(raw: &str) -> Result<Option<u64>> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(None);
    }
    if raw.starts_with('-') && raw.parse::<i128>().is_ok() {
        return Err(format!("VIBEOS_QEMU_RR_BREAK must not be negative, got {raw}").into());
    }
    raw.parse::<u64>().map(Some).map_err(|_| {
        Error::new(format!(
            "VIBEOS_QEMU_RR_BREAK must be an integer, got {raw:?}"
        ))
    })
}
pub fn env_request(raw: &str) -> Result<Option<(String, PathBuf)>> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(None);
    }
    let (mode, path) = raw.split_once(':').ok_or_else(|| {
        Error::new(format!(
            "VIBEOS_QEMU_RR must be '<record|replay>:<trace-dir>', got {raw:?}"
        ))
    })?;
    if !matches!(mode, "record" | "replay") {
        return Err(format!("VIBEOS_QEMU_RR mode must be record or replay, got {mode:?}").into());
    }
    Ok(Some((mode.into(), PathBuf::from(path))))
}
pub fn dispatch(request: &Value) -> Result<Value> {
    let a = &request["kwargs"];
    let op = request["op"]
        .as_str()
        .ok_or("missing op")?
        .trim_start_matches("record_replay_");
    let trace = Path::new(a["trace_dir"].as_str().unwrap_or(""));
    let argv = || serde_json::from_value::<Vec<String>>(a["argv"].clone()).map_err(Error::from);
    Ok(match op {
        "parse_drive" => json!(parse_drive(a["spec"].as_str().ok_or("missing drive spec")?)),
        "format_drive" => json!(format_drive(&serde_json::from_value(a["drive"].clone())?)),
        "is_x86" => json!(is_x86(&argv()?)),
        "sha256" => json!(sha256(Path::new(
            a["path"].as_str().ok_or("missing path")?
        ))?),
        "stage_image" => {
            stage_image(
                Path::new(a["src"].as_str().ok_or("missing src")?),
                Path::new(a["dst"].as_str().ok_or("missing dst")?),
                a["format"].as_str().unwrap_or("raw"),
            )?;
            Value::Null
        }
        "transform" => json!(transform(
            &argv()?,
            a["mode"].as_str().ok_or("missing mode")?,
            trace,
            a["shift"].as_u64().unwrap_or(DEFAULT_SHIFT),
            a["stage"].as_bool().unwrap_or(true)
        )?),
        "snapshot_pristine" => {
            snapshot_pristine(trace)?;
            Value::Null
        }
        "prepare_replay" => {
            prepare_replay(trace)?;
            Value::Null
        }
        "write_manifest" => json!(write_manifest(trace, a)?),
        "load_manifest" => load_manifest(trace)?,
        "note_run" => json!(note_run(
            trace,
            a["mode"].as_str().ok_or("missing mode")?,
            a["serial_log"].as_str().ok_or("missing serial log")?
        )?),
        "last_run" => last_run(trace)?,
        "host_shape" => json!(host_shape(&argv()?)),
        "assert_same_shape" => {
            assert_same_shape(trace, &argv()?)?;
            Value::Null
        }
        "check_trace" => json!(check_trace(trace, a["version"].as_str().unwrap_or(""))?),
        "trace_position" => json!(trace_position(trace)),
        "event_name" => json!(event_name(
            a["eid"]
                .as_u64()
                .filter(|n| *n <= 255)
                .ok_or("invalid event id")? as u8
        )),
        "boundaries" => json!(boundaries(trace)?),
        "complete" => json!(provably_complete(trace)),
        "events_page" => events_page(
            trace,
            serde_json::from_value(a.get("cursor").cloned().unwrap_or(Value::Null))?,
            a["limit"].as_u64().unwrap_or(512) as usize,
        )?,
        "around" => json!(around(
            trace,
            a["offset"].as_u64().ok_or("missing offset")?,
            a["before"].as_u64().unwrap_or(5) as usize,
            a["after"].as_u64().unwrap_or(5) as usize
        )?),
        "snap_break" => json!(snap_break(
            trace,
            a["want"].as_u64().ok_or("invalid break icount")?
        )?),
        "env_shift" => json!(env_shift(a["raw"].as_str().unwrap_or(""))?),
        "env_break" => json!(env_break(a["raw"].as_str().unwrap_or(""))?),
        "env_request" => json!(env_request(a["raw"].as_str().unwrap_or(""))?),
        _ => return Err(format!("unknown record/replay operation {op}").into()),
    })
}

#[cfg(test)]
mod controls {
    use super::*;
    #[test]
    fn event_pages_resume_without_replaying_or_losing_instruction_counts() {
        let dir = tempfile::tempdir().unwrap();
        let mut bytes = REPLAY_VERSION.to_be_bytes().to_vec();
        bytes.extend([0; 8]);
        for _ in 0..1200 {
            bytes.push(0);
            bytes.extend(3u32.to_be_bytes());
        }
        fs::write(dir.path().join("replay.bin"), bytes).unwrap();
        let mut cursor = None;
        let mut count = 0;
        let mut last = 0;
        loop {
            let page = events_page(dir.path(), cursor, 512).unwrap();
            let rows: Vec<ReplayEvent> = serde_json::from_value(page["events"].clone()).unwrap();
            for e in rows {
                assert_eq!(e.icount, last + 3);
                last = e.icount;
                count += 1;
            }
            cursor = serde_json::from_value(page["next"].clone()).unwrap();
            if cursor.is_none() {
                break;
            }
        }
        assert_eq!(count, 1200);
        assert_eq!(last, 3600);
    }
}
