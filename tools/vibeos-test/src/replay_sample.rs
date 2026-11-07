//! Guest-state fingerprints with bounded RAM streaming and interoperable chunk storage.
use crate::protocol::QmpClient;
use crate::{record_replay, Error, Result};
use blake2::{
    digest::{Update, VariableOutput},
    Blake2bVar,
};
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use regex::Regex;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::{fs::OpenOptionsExt, io::AsRawFd},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
// A FIFO normally holds only a few KiB. Sleeping per empty read throttles a
// GiB capture to minutes; poll wakes as soon as QEMU supplies the next bytes.
fn wait_readable(input: &File) -> Result<()> {
    let mut fd = libc::pollfd {
        fd: input.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    let rc = unsafe { libc::poll(&mut fd, 1, 100) };
    if rc < 0 {
        let error = std::io::Error::last_os_error();
        if error.kind() != std::io::ErrorKind::Interrupted {
            return Err(error.into());
        }
    }
    if fd.revents & (libc::POLLERR | libc::POLLNVAL) != 0 {
        return Err("pmemsave FIFO poll failed".into());
    }
    Ok(())
}
pub const CHUNK: usize = 1 << 20;
pub const SAMPLES_FILE: &str = "samples.jsonl";
pub const VERDICT_FILE: &str = "diverge.json";
pub const RAM_STORE: &str = "ram";
pub(crate) fn qerr(e: Error) -> Error {
    if e.kind.is_none() {
        e.with_kind("QmpTransport")
    } else {
        e
    }
}
fn string<'a>(a: &'a Value, key: &str) -> Result<&'a str> {
    a[key]
        .as_str()
        .ok_or_else(|| Error::new(format!("missing {key}")))
}
pub fn ram_bytes(argv: &[String]) -> Result<u64> {
    let i = argv
        .iter()
        .position(|s| s == "-m")
        .ok_or("no -m in the QEMU argv; cannot size guest RAM")?;
    let spec = argv
        .get(i + 1)
        .ok_or("no -m value; cannot size guest RAM")?
        .split(',')
        .next()
        .unwrap()
        .trim_start_matches("size=");
    let re = Regex::new(r"(?i)^(\d+)([KMGT]?)$")?;
    let c = re
        .captures(spec)
        .ok_or_else(|| Error::new(format!("cannot parse -m {spec:?}")))?;
    let n: u64 = c[1]
        .parse()
        .map_err(|_| Error::new("guest RAM size overflow"))?;
    let power = match c[2].to_uppercase().as_str() {
        "K" => 10,
        "" | "M" => 20,
        "G" => 30,
        _ => 40,
    };
    n.checked_mul(1u64 << power)
        .ok_or_else(|| Error::new("guest RAM size overflow"))
}
pub fn ram_ranges(argv: &[String]) -> Result<Vec<(u64, u64)>> {
    let n = ram_bytes(argv)?;
    if argv.first().is_some_and(|a| a.contains("aarch64")) {
        return Ok(vec![(0x40000000, n)]);
    }
    if n > 0x80000000 {
        return Err(format!("x86 guest RAM {n:#x} spans the PCI hole; ranges not modelled").into());
    }
    if n < 0x100000 {
        return Err("x86 guest RAM does not cover the modelled legacy hole".into());
    }
    Ok(vec![(0, 0xa0000), (0x100000, n - 0x100000)])
}
pub fn keep(store: &Path, bytes: &[u8]) -> Result<String> {
    let mut hasher = Blake2bVar::new(8).map_err(|e| Error::new(e.to_string()))?;
    hasher.update(bytes);
    let mut digest = [0; 8];
    hasher
        .finalize_variable(&mut digest)
        .map_err(|e| Error::new(e.to_string()))?;
    let hash = digest
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let dst = store.join(format!("{hash}.z"));
    if !dst.is_file() {
        fs::create_dir_all(store)?;
        let mut tmp = tempfile::NamedTempFile::new_in(store)?;
        {
            let mut z = ZlibEncoder::new(&mut tmp, Compression::new(1));
            z.write_all(bytes)?;
            z.finish()?;
        }
        tmp.persist(&dst).map_err(|e| Error::new(e.to_string()))?;
    }
    Ok(hash)
}
fn load(store: &Path, hash: &str) -> Result<Option<Vec<u8>>> {
    // A digest is a filename component, never an arbitrary path supplied by a trace.
    if hash.len() != 16 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid RAM chunk digest".into());
    }
    let path = store.join(format!("{hash}.z"));
    if !path.is_file() {
        return Ok(None);
    }
    let mut out = Vec::new();
    ZlibDecoder::new(File::open(path)?)
        .take((CHUNK + 1) as u64)
        .read_to_end(&mut out)?;
    if out.len() > CHUNK {
        return Err("RAM chunk exceeds one MiB".into());
    }
    Ok(Some(out))
}
fn grouped_hex(bytes: &[u8]) -> String {
    bytes
        .chunks(8)
        .map(|b| b.iter().map(|c| format!("{c:02x}")).collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}
pub fn byte_diff(store: &Path, base: u64, rec: &str, rep: &str, runs: usize) -> Result<Value> {
    let (Some(a), Some(b)) = (load(store, rec)?, load(store, rep)?) else {
        return Ok(
            json!({"base":format!("{base:#x}"),"error":"chunk not in the RAM store (older trace?)"}),
        );
    };
    let mut out = Vec::new();
    let mut total = 0;
    let mut i = 0;
    let n = a.len().min(b.len());
    while i < n {
        if a[i] == b[i] {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < n && a[j] != b[j] {
            j += 1
        }
        total += j - i;
        if out.len() < runs {
            let lo = i & !0xf;
            let hi = n.min((j.max(i + 16) + 15) & !0xf);
            out.push(json!({"addr":format!("{:#x}",base+i as u64),"len":j-i,"record":grouped_hex(&a[lo..hi]),"replay":grouped_hex(&b[lo..hi]),"at":format!("{:#x}",base+lo as u64)}));
        }
        i = j;
    }
    Ok(json!({"base":format!("{base:#x}"),"bytes_differing":total,"runs":out}))
}
pub fn compare(rec: &Value, rep: &Value, store: Option<&Path>) -> Result<Value> {
    let mut diff = serde_json::Map::new();
    if rec["icount"] != rep["icount"] {
        diff.insert("icount".into(), json!([rec["icount"], rep["icount"]]));
    }
    if rec["regs"] != rep["regs"] {
        let a = string(rec, "regs")?;
        let b = string(rep, "regs")?;
        let d = similar::TextDiff::from_lines(a, b);
        let mut lines = vec!["--- record".to_owned(), "+++ replay".into()];
        for change in d.iter_all_changes() {
            if change.tag() == similar::ChangeTag::Equal {
                continue;
            }
            let prefix = if change.tag() == similar::ChangeTag::Delete {
                "-"
            } else {
                "+"
            };
            lines.push(format!(
                "{prefix}{}",
                change.value().trim_end_matches(['\r', '\n'])
            ));
        }
        diff.insert("regs".into(), json!(lines));
    }
    let a = rec["mem"].as_array().ok_or("missing recorded memory map")?;
    let b = rep["mem"].as_array().ok_or("missing replay memory map")?;
    if a.len() != b.len() {
        return Err("fingerprint memory range counts differ".into());
    }
    let mut bad = Vec::new();
    for (a, b) in a.iter().zip(b) {
        let base = a["base"].as_u64().ok_or("invalid memory base")?;
        if a["base"] != b["base"] {
            return Err("fingerprint memory bases differ".into());
        }
        let aa = a["hashes"].as_array().ok_or("missing recorded hashes")?;
        let bb = b["hashes"].as_array().ok_or("missing replay hashes")?;
        if aa.len() != bb.len() {
            return Err("fingerprint chunk counts differ".into());
        }
        for (i, (ha, hb)) in aa.iter().zip(bb).enumerate() {
            if ha != hb {
                bad.push((
                    base + i as u64 * CHUNK as u64,
                    ha.as_str().ok_or("invalid chunk digest")?,
                    hb.as_str().ok_or("invalid chunk digest")?,
                ));
            }
        }
    }
    if !bad.is_empty() {
        diff.insert(
            "mem_mib".into(),
            json!(bad
                .iter()
                .map(|(base, _, _)| format!("{base:#x}"))
                .collect::<Vec<_>>()),
        );
        if let Some(store) = store {
            diff.insert(
                "mem_bytes".into(),
                json!(bad
                    .iter()
                    .take(16)
                    .map(|(base, a, b)| byte_diff(store, *base, a, b, 24))
                    .collect::<Result<Vec<_>>>()?),
            );
        }
    }
    Ok(if diff.is_empty() {
        Value::Null
    } else {
        Value::Object(diff)
    })
}
fn regs(text: &str) -> BTreeMap<String, String> {
    Regex::new(r"\b([A-Z][A-Z0-9_]*)=([0-9a-fA-F]+)\b")
        .unwrap()
        .captures_iter(text)
        .map(|c| (c[1].into(), c[2].into()))
        .collect()
}
pub fn artifact(diff: &Value, io: bool, irq: bool, rec: &str, rep: &str) -> Option<&'static str> {
    if diff.get("icount").is_some() {
        return None;
    }
    if io && diff.get("regs").is_none() {
        return Some("record pause completed I/O");
    }
    if irq && diff.get("mem_mib").is_none() {
        let a = regs(rec);
        let b = regs(rep);
        let keys: BTreeSet<_> = a.keys().chain(b.keys()).collect();
        if keys
            .into_iter()
            .filter(|k| a.get(*k) != b.get(*k))
            .all(|k| matches!(k.as_str(), "PC" | "PSTATE"))
        {
            return Some("interrupt entry on one side of the pause only");
        }
    }
    None
}
pub struct Fingerprinter {
    pub qmp: QmpClient,
    ranges: Vec<(u64, u64)>,
    work: PathBuf,
    pub store: PathBuf,
    sequence: u64,
}
impl Fingerprinter {
    pub fn new(ranges: Vec<(u64, u64)>, work: &Path, store: &Path) -> Result<Self> {
        fs::create_dir_all(store)?;
        Ok(Self {
            qmp: QmpClient::new(),
            ranges,
            work: work.into(),
            store: store.into(),
            sequence: 0,
        })
    }
    pub fn connect(&mut self, path: &Path, timeout: Duration) -> Result<()> {
        let deadline = Instant::now() + timeout;
        loop {
            match self.qmp.connect(path, Duration::from_secs(30)) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    if Instant::now() >= deadline {
                        return Err(qerr(e));
                    }
                    thread::sleep(Duration::from_millis(200));
                }
            }
        }
    }
    pub fn icount(&mut self) -> Result<u64> {
        self.qmp.execute("query-replay", json!({})).map_err(qerr)?["return"]["icount"]
            .as_u64()
            .ok_or_else(|| Error::new("query-replay has no icount"))
    }
    fn hash_range(&mut self, base: u64, length: u64) -> Result<Vec<String>> {
        let fifo = self.work.join(format!(
            "rrmem-{}-{}.fifo",
            std::process::id(),
            self.sequence
        ));
        self.sequence += 1;
        use std::os::unix::ffi::OsStrExt;
        let name = std::ffi::CString::new(fifo.as_os_str().as_bytes())
            .map_err(|_| Error::new("invalid FIFO path"))?;
        if unsafe { libc::mkfifo(name.as_ptr(), 0o600) } != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        let opened = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(&fifo);
        let mut input = match opened {
            Ok(v) => v,
            Err(e) => {
                let _ = fs::remove_file(&fifo);
                return Err(e.into());
            }
        };
        let stop = Arc::new(AtomicBool::new(false));
        let abort = stop.clone();
        let store = self.store.clone();
        let reader = thread::spawn(move || -> Result<Vec<String>> {
            let deadline = Instant::now() + Duration::from_secs(120);
            let mut remaining = length;
            let mut out = Vec::new();
            while remaining > 0 {
                let size = (remaining.min(CHUNK as u64)) as usize;
                let mut chunk = vec![0; size];
                let mut filled = 0;
                while filled < size {
                    if abort.load(Ordering::Acquire) {
                        return Err("pmemsave aborted".into());
                    }
                    match input.read(&mut chunk[filled..]) {
                        Ok(0) => wait_readable(&input)?,
                        Ok(n) => filled += n,
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            wait_readable(&input)?
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                        Err(e) => return Err(e.into()),
                    }
                    if Instant::now() > deadline {
                        return Err("pmemsave FIFO timed out".into());
                    }
                }
                out.push(keep(&store, &chunk)?);
                remaining -= size as u64;
            }
            Ok(out)
        });
        let result = self
            .qmp
            .execute_with_timeout(
                "pmemsave",
                json!({"val":base,"size":length,"filename":fifo}),
                Duration::from_secs(120),
            )
            .map_err(qerr);
        if result.is_err() {
            stop.store(true, Ordering::Release)
        }
        let chunks = reader
            .join()
            .map_err(|_| Error::new("RAM fingerprint reader panicked"));
        let _ = fs::remove_file(&fifo);
        result?;
        chunks?
    }
    pub fn fingerprint(&mut self) -> Result<Value> {
        let regs = self.qmp.hmp("info registers -a").map_err(qerr)?;
        let mut mem = Vec::new();
        for (base, length) in self.ranges.clone() {
            mem.push(json!({"base":base,"hashes":self.hash_range(base,length)?}));
        }
        Ok(json!({"icount":self.icount()?,"regs":regs,"mem":mem}))
    }
    pub fn wait_stop_or_stall(&mut self, window: Duration, limit: usize) -> Result<bool> {
        let mut last = None;
        let mut still = 0;
        loop {
            if self.qmp.wait_event("STOP", window).map_err(qerr)?.is_some() {
                return Ok(true);
            }
            let now = self.icount()?;
            if self.qmp.saw_event("STOP").is_some() {
                return Ok(true);
            }
            still = if last == Some(now) { still + 1 } else { 0 };
            if still >= limit {
                return Ok(false);
            }
            last = Some(now);
        }
    }
}
fn write_json(path: &Path, value: &Value) -> Result<()> {
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(value)?))?;
    Ok(())
}
// Disk-heavy phases keep a request in flight at nearly every instant, so a fixed short
// backoff found no gap there (4 of 11 samples exhausted 20 x 37ms). Grow the wait
// instead: ~7s at most, still under one sample period.
const DRAIN_RETRIES: u32 = 20;
fn drain_backoff(retry: u32) -> Duration {
    Duration::from_millis((37 * u64::from(retry)).min(500))
}
/// Completed block requests across every device, from `query-blockstats`. Any counter
/// that moves while the VM is stopped means the stop itself completed a request.
fn block_completions(fp: &mut Fingerprinter) -> Result<u64> {
    let r = fp.qmp.execute("query-blockstats", json!({})).map_err(qerr)?;
    let rows = r
        .get("return")
        .unwrap_or(&r)
        .as_array()
        .ok_or("query-blockstats did not return a device list")?;
    let mut n = 0u64;
    for row in rows {
        if let Some(stats) = row["stats"].as_object() {
            for (k, v) in stats {
                if k.ends_with("_operations") {
                    n = n.wrapping_add(v.as_u64().unwrap_or(0));
                }
            }
        }
    }
    Ok(n)
}
pub fn record_loop(fp: &mut Fingerprinter, period: f64, path: &Path) -> Result<Value> {
    if !period.is_finite() || period <= 0.0 {
        return Err("RR sample period must be finite and positive".into());
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let mut n = 0;
    loop {
        let result = (|| -> Result<Value> {
            // QMP `stop` drains block I/O. A request it completes is logged at exactly the
            // paused icount, but a replay's break lands BEFORE that completion is applied,
            // so such a sample can never compare exactly (`artifact`). Prefer a pause that
            // drained nothing: completion counters unchanged across the stop. The counters
            // are read while the guest still runs, so a completion just before the stop
            // also retries; that only costs a retry, never a false "clean".
            let mut retries = 0;
            loop {
                let before = block_completions(fp)?;
                fp.qmp.execute("stop", json!({})).map_err(qerr)?;
                let drained = block_completions(fp)? != before;
                if drained && retries < DRAIN_RETRIES {
                    fp.qmp.execute("cont", json!({})).map_err(qerr)?;
                    retries += 1;
                    thread::sleep(drain_backoff(retries));
                    continue;
                }
                let mut s = fp.fingerprint()?;
                fp.qmp.execute("cont", json!({})).map_err(qerr)?;
                s["pause_retries"] = json!(retries);
                s["io_at_pause"] = json!(drained);
                return Ok(s);
            }
        })();
        let mut s = match result {
            Ok(s) => s,
            Err(e) if n > 0 && e.kind.as_deref() == Some("QmpClosed") => {
                eprintln!("[rr-sample] recording ended after {n} samples ({e})");
                return Ok(json!({"samples":n}));
            }
            Err(e) => return Err(e),
        };
        s["t"] = json!(
            (SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| Error::new(e.to_string()))?
                .as_secs_f64()
                * 10.0)
                .round()
                / 10.0
        );
        writeln!(file, "{s}")?;
        file.flush()?;
        n += 1;
        thread::sleep(Duration::from_secs_f64(period));
    }
}
pub fn replay_loop(fp: &mut Fingerprinter, trace: &Path) -> Result<Value> {
    let path = trace.join(VERDICT_FILE);
    let mut samples = Vec::new();
    for line in fs::read_to_string(trace.join(SAMPLES_FILE))?
        .lines()
        .filter(|s| !s.is_empty())
    {
        samples.push(serde_json::from_str::<Value>(line)?);
    }
    let want: BTreeSet<u64> = samples
        .iter()
        .map(|s| {
            s["icount"]
                .as_u64()
                .ok_or_else(|| Error::new("sample has invalid icount"))
        })
        .collect::<Result<_>>()?;
    let (mut bounds, mut io, mut irq) = (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
    record_replay::walk(trace, false, |e| {
        if want.contains(&e.icount) {
            match e.eid {
                0 => {
                    bounds.insert(e.icount);
                }
                3..=9 => {
                    io.insert(e.icount);
                }
                1 => {
                    irq.insert(e.icount);
                }
                _ => {}
            }
        }
        Ok(true)
    })?;
    let mut verdict = json!({"samples":samples.len(),"compared":0,"skipped":[],"inconclusive":[],"last_match":null});
    let mut completed = true;
    let result = (|| -> Result<()> {
        for s in &samples {
            let ic = s["icount"].as_u64().ok_or("invalid sample icount")?;
            let off = !bounds.contains(&ic);
            verdict["attempting"] = json!({"icount":ic,"mid_chunk":off});
            write_json(&path, &verdict)?;
            let now = fp.icount()?;
            if now > ic {
                verdict["skipped"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"icount":ic,"why":format!("replay already at {now}")}));
                continue;
            }
            fp.qmp.events.clear();
            fp.qmp
                .execute("replay-break", json!({"icount":ic}))
                .map_err(qerr)?;
            if !fp.wait_stop_or_stall(Duration::from_secs(30), 4)? {
                verdict["error"]=json!(format!("replay WEDGED before reaching the break at {ic}: icount stopped advancing. Compare from the last match; the divergence is between it and the wedge."));
                verdict["wedged_at"] = json!(fp.icount()?);
                verdict["wedge_regs"] = json!(fp.qmp.hmp("info registers -a").map_err(qerr)?);
                completed = false;
                break;
            }
            let at = fp.icount()?;
            if at != ic {
                let status = fp.qmp.execute("query-status", json!({})).map_err(qerr)?;
                verdict["error"]=json!(format!("the VM stopped at icount {at} (status {:?}), short of the break at {ic} — not by rr diverge, so that state is not comparable",status["return"]["status"]));
                verdict["stopped_at"] = json!(at);
                verdict["stop_regs"] = json!(fp.qmp.hmp("info registers -a").map_err(qerr)?);
                completed = false;
                break;
            }
            let r = fp.fingerprint()?;
            verdict["compared"] = json!(verdict["compared"].as_u64().unwrap() + 1);
            verdict["mid_chunk_ok"] =
                json!(verdict["mid_chunk_ok"].as_u64().unwrap_or(0) + u64::from(off));
            let d = compare(s, &r, Some(&fp.store))?;
            let why = if d.is_null() {
                None
            } else {
                artifact(
                    &d,
                    io.contains(&ic),
                    irq.contains(&ic),
                    string(s, "regs")?,
                    string(&r, "regs")?,
                )
            };
            if let Some(why) = why {
                verdict["inconclusive"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"icount":ic,"why":why,"mem_mib":d.get("mem_mib")}));
            } else if !d.is_null() {
                let mut first = d;
                first["icount"] = json!(ic);
                verdict["first_mismatch"] = first;
                completed = false;
                break;
            }
            if !verdict["inconclusive"]
                .as_array()
                .unwrap()
                .iter()
                .any(|x| x["icount"] == ic)
            {
                verdict["last_match"] = json!(ic);
            }
            write_json(&path, &verdict)?;
            fp.qmp.execute("cont", json!({})).map_err(qerr)?;
        }
        Ok(())
    })();
    if let Err(e) = result {
        verdict["error"] = json!(e.message);
        completed = false;
    }
    verdict["completed"] = json!(completed);
    verdict["all_matched"] = json!(comparison_status(&verdict)?["exit"] == 0);
    if completed {
        verdict.as_object_mut().unwrap().remove("attempting");
    }
    write_json(&path, &verdict)?;
    if verdict.get("first_mismatch").is_some() || verdict.get("error").is_some() {
        let _ = fp.qmp.execute("quit", json!({}));
    }
    eprintln!("[rr-diverge] verdict written to {}", path.display());
    Ok(verdict)
}

/// Finishing a comparison loop does not make uncertain checkpoints equal.
pub fn comparison_status(verdict: &Value) -> Result<Value> {
    if verdict.get("first_mismatch").is_some_and(|v| !v.is_null())
        || verdict.get("error").is_some_and(|v| !v.is_null())
    {
        return Ok(json!({"exit":1,"message":"record/replay comparison failed"}));
    }
    let samples = verdict["samples"]
        .as_u64()
        .ok_or("comparison omitted sample count")?;
    let compared = verdict["compared"]
        .as_u64()
        .ok_or("comparison omitted compared count")?;
    let uncertain = verdict["inconclusive"]
        .as_array()
        .ok_or("comparison omitted inconclusive checkpoints")?
        .len() as u64;
    let skipped = verdict["skipped"]
        .as_array()
        .ok_or("comparison omitted skipped checkpoints")?
        .len() as u64;
    if uncertain > compared || compared > samples || skipped > samples - compared {
        return Err("inconsistent comparison coverage counts".into());
    }
    let exact = compared - uncertain;
    let matched = samples > 0
        && exact == samples
        && skipped == 0
        && verdict["completed"].as_bool() != Some(false);
    Ok(
        json!({"exit":if matched {0}else{2},"exact_matches":exact,"message":if matched {format!("all {samples} fingerprints matched")}else{format!("INCONCLUSIVE: {exact}/{samples} fingerprints matched exactly; {uncertain} uncertain, {skipped} skipped, {} not compared. Full equivalence is unproven.",samples-compared-skipped)}}),
    )
}

pub fn run(a: &Value) -> Result<Value> {
    let mode = string(a, "mode")?;
    let trace = Path::new(string(a, "trace")?);
    let argv: Vec<String> = serde_json::from_value(a["argv"].clone())?;
    let result = (|| -> Result<Value> {
        let mut fp = Fingerprinter::new(
            ram_ranges(&argv)?,
            Path::new(string(a, "work")?),
            &trace.join(RAM_STORE),
        )?;
        fp.connect(Path::new(string(a, "sock")?), Duration::from_secs(60))?;
        let result = if mode == "record" {
            let period = string(a, "value")?
                .parse::<f64>()
                .map_err(|_| Error::new("RR sample period is not a number"))?;
            record_loop(&mut fp, period, &trace.join(SAMPLES_FILE))
        } else if mode == "replay" {
            replay_loop(&mut fp, trace)
        } else {
            Err("unknown RR sampler mode".into())
        };
        fp.qmp.close();
        result
    })();
    if let Err(e) = &result {
        if mode == "replay" {
            write_json(
                &trace.join(VERDICT_FILE),
                &json!({"samples":0,"compared":0,"skipped":[],"inconclusive":[],"last_match":null,"error":format!("checker died: {e}")}),
            )?;
        }
    }
    result
}
pub fn dispatch(request: &Value) -> Result<Value> {
    let a = &request["kwargs"];
    let op = request["op"]
        .as_str()
        .ok_or("missing op")?
        .trim_start_matches("rrsample_");
    Ok(match op {
        "qmp_args" => json!([
            "-qmp",
            &format!("unix:{},server,nowait", string(a, "sock_path")?)
        ]),
        "ram_bytes" => json!(ram_bytes(&serde_json::from_value::<Vec<String>>(
            a["argv"].clone()
        )?)?),
        "ram_ranges" => json!(ram_ranges(&serde_json::from_value::<Vec<String>>(
            a["argv"].clone()
        )?)?),
        "compare" => compare(&a["rec"], &a["rep"], a["store"].as_str().map(Path::new))?,
        "artifact" => json!(artifact(
            &a["diff"],
            a["io"].as_bool().unwrap_or(false),
            a["irq"].as_bool().unwrap_or(false),
            a["rec_regs"].as_str().unwrap_or(""),
            a["rep_regs"].as_str().unwrap_or("")
        )),
        "byte_diff" => byte_diff(
            Path::new(string(a, "store")?),
            a["base"].as_u64().ok_or("invalid RAM base")?,
            string(a, "rec_hash")?,
            string(a, "rep_hash")?,
            a["runs"].as_u64().unwrap_or(24) as usize,
        )?,
        "keep" => json!(keep(
            Path::new(string(a, "store")?),
            &serde_json::from_value::<Vec<u8>>(a["bytes"].clone())?
        )?),
        "run" => run(a)?,
        _ => return Err(format!("unknown RR sampler operation {op}").into()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{BufRead, BufReader},
        os::unix::net::UnixListener,
    };
    fn monitor(sock: &Path, fail: bool) -> thread::JoinHandle<()> {
        monitor_mode(sock, fail, None, false)
    }
    fn monitor_mode(
        sock: &Path,
        fail: bool,
        hold_after_sample: Option<bool>,
        drain_once: bool,
    ) -> thread::JoinHandle<()> {
        let listener = UnixListener::bind(sock).unwrap();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            writeln!(
                stream,
                "{}",
                json!({"QMP":{"version":{},"capabilities":[]}})
            )
            .unwrap();
            let mut input = BufReader::new(stream.try_clone().unwrap());
            let mut stops = 0;
            loop {
                let mut line = String::new();
                if input.read_line(&mut line).unwrap() == 0 {
                    break;
                }
                let a: Value = serde_json::from_str(&line).unwrap();
                let command = a["execute"].as_str().unwrap();
                if command == "stop" {
                    stops += 1;
                    if stops > 1 + u32::from(drain_once) {
                        if let Some(hold) = hold_after_sample {
                            if hold {
                                thread::sleep(Duration::from_millis(500));
                            }
                            break;
                        }
                    }
                }
                let mut result = json!({"return":{}});
                if command == "human-monitor-command" {
                    result = json!({"return":"CPU#0\nPC=0000000040000000 X29=0000000040000100\n"});
                }
                if command == "query-blockstats" {
                    // With `drain_once`, the first stop completes one request.
                    let done = if drain_once { stops.min(1) } else { 0 };
                    result = json!({"return":[{"device":"vda","stats":{"rd_operations":done,"rd_total_time_ns":stops}}]});
                }
                if command == "query-replay" {
                    result = json!({"return":{"icount":42}});
                }
                if command == "pmemsave" {
                    if fail {
                        result = json!({"error":{"class":"GenericError","desc":"controlled unsupported pmemsave"}});
                    } else {
                        let args = &a["arguments"];
                        let mut output = OpenOptions::new()
                            .write(true)
                            .open(args["filename"].as_str().unwrap())
                            .unwrap();
                        let mut n = args["size"].as_u64().unwrap();
                        let block = vec![0x5a; 8192];
                        while n > 0 {
                            let count = n.min(block.len() as u64) as usize;
                            output.write_all(&block[..count]).unwrap();
                            n -= count as u64;
                        }
                    }
                }
                writeln!(stream, "{result}").unwrap();
                stream.flush().unwrap();
            }
        })
    }
    #[test]
    fn fingerprints_stream_partial_last_chunks_and_use_python_blake2b8_digests() {
        let temp = tempfile::tempdir().unwrap();
        let socket = temp.path().join("qmp.sock");
        let server = monitor(&socket, false);
        let mut fp = Fingerprinter::new(
            vec![(0x40000000, CHUNK as u64 + 17)],
            temp.path(),
            &temp.path().join("ram"),
        )
        .unwrap();
        fp.connect(&socket, Duration::from_secs(1)).unwrap();
        let result = fp.fingerprint().unwrap();
        assert_eq!(result["icount"], 42);
        let hashes = result["mem"][0]["hashes"].as_array().unwrap();
        assert_eq!(hashes.len(), 2);
        assert_eq!(
            load(&fp.store, hashes[0].as_str().unwrap())
                .unwrap()
                .unwrap(),
            vec![0x5a; CHUNK]
        );
        assert_eq!(
            load(&fp.store, hashes[1].as_str().unwrap())
                .unwrap()
                .unwrap(),
            vec![0x5a; 17]
        );
        assert_eq!(keep(&fp.store, b"hello world").unwrap(), "878633aa32a3b150");
        assert!(!fs::read_dir(temp.path()).unwrap().any(|e| e
            .unwrap()
            .path()
            .extension()
            .is_some_and(|s| s == "fifo")));
        fp.qmp.close();
        server.join().unwrap();
    }
    #[test]
    fn rejected_memory_capture_cancels_the_fifo_reader_and_fails_the_requested_sampler() {
        let temp = tempfile::tempdir().unwrap();
        let socket = temp.path().join("qmp.sock");
        let server = monitor(&socket, true);
        let mut fp = Fingerprinter::new(
            vec![(0x40000000, CHUNK as u64)],
            temp.path(),
            &temp.path().join("ram"),
        )
        .unwrap();
        fp.connect(&socket, Duration::from_secs(1)).unwrap();
        let start = Instant::now();
        let error = record_loop(&mut fp, 0.1, &temp.path().join(SAMPLES_FILE)).unwrap_err();
        assert_eq!(error.kind.as_deref(), Some("QmpError"));
        assert!(error.message.contains("unsupported pmemsave"));
        assert!(start.elapsed() < Duration::from_secs(5));
        assert!(!fs::read_dir(temp.path()).unwrap().any(|e| e
            .unwrap()
            .path()
            .extension()
            .is_some_and(|s| s == "fifo")));
        fp.qmp.close();
        server.join().unwrap();
    }
    #[test]
    fn a_pause_that_completed_io_is_retried_and_the_clean_sample_says_so() {
        let temp = tempfile::tempdir().unwrap();
        let socket = temp.path().join("qmp.sock");
        // Stops 1 (drained, retried) and 2 (clean, sampled); the mock leaves at stop 3.
        let server = monitor_mode(&socket, false, Some(false), true);
        let mut fp = Fingerprinter::new(vec![], temp.path(), &temp.path().join("ram")).unwrap();
        fp.qmp.connect(&socket, Duration::from_millis(100)).unwrap();
        let path = temp.path().join(SAMPLES_FILE);
        let _ = record_loop(&mut fp, 0.005, &path);
        let text = fs::read_to_string(path).unwrap();
        let first: Value = serde_json::from_str(text.lines().next().unwrap()).unwrap();
        assert_eq!(first["pause_retries"], 1);
        assert_eq!(first["io_at_pause"], false);
        fp.qmp.close();
        server.join().unwrap();
    }
    #[test]
    fn different_memory_layouts_are_refused_instead_of_comparing_misaligned_chunks() {
        let a = json!({"icount":10,"regs":"PC=1","mem":[{"base":0,"hashes":["0000000000000000"]}]});
        let mut b = a.clone();
        b["mem"][0]["base"] = json!(4096);
        assert!(compare(&a, &b, None).is_err());
        b = a.clone();
        b["mem"][0]["hashes"] = json!([]);
        assert!(compare(&a, &b, None).is_err());
    }
    #[test]
    fn a_monitor_timeout_after_one_sample_is_not_successful_shutdown() {
        for hold in [false, true] {
            let temp = tempfile::tempdir().unwrap();
            let socket = temp.path().join("qmp.sock");
            let server = monitor_mode(&socket, false, Some(hold), false);
            let mut fp = Fingerprinter::new(vec![], temp.path(), &temp.path().join("ram")).unwrap();
            fp.qmp.connect(&socket, Duration::from_millis(100)).unwrap();
            let path = temp.path().join(SAMPLES_FILE);
            let result = record_loop(&mut fp, 0.005, &path);
            assert_eq!(fs::read_to_string(path).unwrap().lines().count(), 1);
            if hold {
                assert_eq!(result.unwrap_err().kind.as_deref(), Some("QmpTransport"));
            } else {
                assert_eq!(result.unwrap()["samples"], 1);
            }
            fp.qmp.close();
            server.join().unwrap();
        }
    }
}

#[cfg(test)]
mod bulk_qemu_control {
    use super::*;
    #[test]
    #[ignore = "requires installed qemu-system-aarch64; explicitly run for bulk transfer changes"]
    fn real_qemu_gib_ram_capture_is_bounded_and_complete() {
        use std::process::{Child, Command, Stdio};
        struct Owned(Child);
        impl Drop for Owned {
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
        let temp = tempfile::tempdir().unwrap();
        let socket = temp.path().join("qmp.sock");
        let _child = Owned(
            Command::new("qemu-system-aarch64")
                .args([
                    "-M",
                    "virt",
                    "-cpu",
                    "cortex-a57",
                    "-m",
                    "1024",
                    "-S",
                    "-accel",
                    "tcg,thread=single",
                    "-display",
                    "none",
                    "-nic",
                    "none",
                    "-qmp",
                ])
                .arg(format!("unix:{},server=on,wait=off", socket.display()))
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::inherit())
                .spawn()
                .unwrap(),
        );
        let mut fp = Fingerprinter::new(vec![], temp.path(), &temp.path().join("ram")).unwrap();
        fp.connect(&socket, Duration::from_secs(5)).unwrap();
        let start = Instant::now();
        let hashes = fp.hash_range(0x40000000, 1 << 30).unwrap();
        assert_eq!(hashes.len(), 1024);
        let zero = vec![0; CHUNK];
        let reference = temp.path().join("first.raw");
        fp.qmp
            .execute(
                "pmemsave",
                json!({"val":0x40000000u64,"size":CHUNK,"filename":reference}),
            )
            .unwrap();
        assert!(
            load(&fp.store, &hashes[0]).unwrap().unwrap() == fs::read(reference).unwrap(),
            "FIFO and regular-file RAM captures differ"
        );
        assert!(
            load(&fp.store, &hashes[1023]).unwrap().unwrap() == zero,
            "last RAM chunk is not zero"
        );
        assert!(hashes[1..].iter().all(|h| h == &hashes[1023]));
        eprintln!(
            "real QEMU 1GiB FIFO capture: {:.3}s",
            start.elapsed().as_secs_f64()
        );
    }
}

#[cfg(test)]
mod comparison_coverage_controls {
    use super::*;
    #[test]
    fn uncertain_or_partial_comparisons_never_claim_equivalence() {
        let mut verdict =
            json!({"samples":2,"compared":2,"inconclusive":[],"skipped":[],"completed":true});
        assert_eq!(comparison_status(&verdict).unwrap()["exit"], 0);
        verdict["inconclusive"] = json!([{"icount":1},{"icount":2}]);
        let result = comparison_status(&verdict).unwrap();
        assert_eq!(result["exit"], 2);
        assert_eq!(result["exact_matches"], 0);
        verdict["inconclusive"] = json!([]);
        verdict["compared"] = json!(1);
        verdict["skipped"] = json!([{"icount":2}]);
        assert_eq!(comparison_status(&verdict).unwrap()["exit"], 2);
        verdict["samples"] = json!(0);
        verdict["compared"] = json!(0);
        verdict["skipped"] = json!([]);
        assert_eq!(comparison_status(&verdict).unwrap()["exit"], 2);
    }
}
