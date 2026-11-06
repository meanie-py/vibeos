//! Per-vCPU register/stack sampling and batched kernel attribution.
use crate::{callback::Host, verdict};
use crate::{Error, Result};
use regex::Regex;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
pub const CALLER_DEPTH: usize = 3;
pub const STACK_WORDS: usize = 48;
const KERNEL_HALF: u64 = 0xffff800000000000;
fn hex(s: &str) -> Result<u64> {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).map_err(|e| Error::new(e.to_string()))
}
pub fn env_period(raw: &str) -> Result<Option<f64>> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(None);
    }
    let p: f64 = raw.parse().map_err(|_| {
        Error::new(format!(
            "VIBEOS_QEMU_PC_SAMPLE={raw:?} is not a number of seconds"
        ))
    })?;
    if !(0.05..=60.0).contains(&p) {
        return Err(format!("VIBEOS_QEMU_PC_SAMPLE={raw:?}: want 0.05..60 seconds").into());
    }
    Ok(Some(p))
}
pub fn parse_sample(state: &str) -> Result<Vec<Value>> {
    let fps: BTreeMap<_, _> = verdict::cpu_fps(state).into_iter().collect();
    let sps: BTreeMap<_, _> = verdict::cpu_sps(state).into_iter().collect();
    let banner = Regex::new(r"(?m)^CPU#(\d+)\b")?;
    let marks: Vec<_> = banner.captures_iter(state).collect();
    let hlt = Regex::new(r"\bHLT=([01])\b")?;
    let mut halted = BTreeMap::new();
    for (i, c) in marks.iter().enumerate() {
        let cpu: u64 = c[1].parse().map_err(|_| Error::new("invalid CPU index"))?;
        let end = marks
            .get(i + 1)
            .map(|v| v.get(0).unwrap().start())
            .unwrap_or(state.len());
        let flag = hlt
            .captures(&state[c.get(0).unwrap().end()..end])
            .map(|c| &c[1] == "1");
        halted.insert(cpu, flag);
    }
    Ok(verdict::cpu_pcs(state).into_iter().map(|(cpu,pc)|json!({"cpu":cpu,"pc":pc.to_lowercase(),"halted":halted.get(&cpu).copied().flatten(),"fp":fps.get(&cpu),"sp":sps.get(&cpu)})).collect())
}
fn hmp(host: &mut dyn Host, command: &str) -> Result<String> {
    host.call("diagnostic_hmp", json!({"command":command}))?
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| Error::new("monitor did not return text"))
}
pub fn callers(host: &mut dyn Host, cpu: u64, fp: Option<&str>, depth: usize) -> Vec<String> {
    let Some(fp) = fp.filter(|s| !s.is_empty()) else {
        return Vec::new();
    };
    let unit = if fp.len() > 8 { "g" } else { "w" };
    let mut out = Vec::new();
    let result = (|| -> Result<()> {
        hmp(host, &format!("cpu {cpu}"))?;
        let mut fp = hex(fp)?;
        let words = Regex::new(r"0x([0-9a-fA-F]+)")?;
        for _ in 0..depth {
            let body = hmp(host, &format!("x /2{unit}x 0x{fp:x}"))?;
            let body = body.split_once(':').map(|(_, s)| s).unwrap_or(&body);
            let values: Vec<_> = words
                .captures_iter(body)
                .map(|c| hex(&c[1]))
                .collect::<Result<_>>()?;
            if values.len() < 2 {
                break;
            }
            let (next, ret) = (values[0], values[1]);
            out.push(format!("{ret:x}"));
            if next <= fp || next - fp > 0x100000 {
                break;
            }
            fp = next;
        }
        Ok(())
    })();
    let _ = result;
    let _ = hmp(host, "cpu 0");
    out
}
pub fn stack_words(host: &mut dyn Host, cpu: u64, sp: Option<&str>) -> Vec<String> {
    let Some(sp) = sp.filter(|s| s.len() > 8) else {
        return Vec::new();
    };
    let result = (|| -> Result<String> {
        hmp(host, &format!("cpu {cpu}"))?;
        hmp(host, &format!("x /{STACK_WORDS}gx 0x{sp}"))
    })();
    let _ = hmp(host, "cpu 0");
    let Ok(body) = result else { return Vec::new() };
    let words = Regex::new(r"0x([0-9a-fA-F]{16})").unwrap();
    body.lines()
        .flat_map(|line| {
            words
                .captures_iter(line.split_once(':').map(|(_, s)| s).unwrap_or(line))
                .map(|c| c[1].to_lowercase())
                .collect::<Vec<_>>()
        })
        .filter(|s| hex(s).is_ok_and(|v| v >= KERNEL_HALF))
        .collect()
}
pub fn sample_once(host: &mut dyn Host) -> Result<Vec<Value>> {
    let mut rows = parse_sample(&hmp(host, "info registers -a")?)?;
    for row in &mut rows {
        let cpu = row["cpu"].as_u64().unwrap();
        let c = callers(host, cpu, row["fp"].as_str(), CALLER_DEPTH);
        let s = stack_words(host, cpu, row["sp"].as_str());
        let m = row.as_object_mut().unwrap();
        m.remove("fp");
        m.remove("sp");
        m.insert("callers".into(), json!(c));
        m.insert("stack".into(), json!(s));
    }
    Ok(rows)
}
pub fn resolve_batch(
    root: &Path,
    arch: &str,
    pcs: &[String],
    mp: bool,
    reloc: Option<&str>,
) -> Result<BTreeMap<String, String>> {
    let path = root.join(format!(
        "qemu/symbols-{arch}{}.tsv",
        if mp { "-mp" } else { "" }
    ));
    if !path.is_file() {
        return Err(Error::classified(
            "FileNotFoundError",
            format!(
                "{} missing: run qemu/build-index.sh {arch} (an MP boot needs the -mp index)",
                path.display()
            ),
        ));
    }
    let mut addrs = Vec::new();
    let mut names = Vec::new();
    for line in String::from_utf8_lossy(&fs::read(&path)?).lines() {
        let cols: Vec<_> = line.split('\t').collect();
        if cols.len() < 2 {
            continue;
        }
        if let Ok(a) = hex(cols[0]) {
            addrs.push(a);
            names.push(cols[1].to_owned());
        }
    }
    let shift = if let Some(r) = reloc.filter(|s| !s.is_empty()) {
        if let Some(r) = r.strip_prefix('-') {
            -(hex(r)? as i128)
        } else {
            hex(r)? as i128
        }
    } else {
        0
    };
    let mut out = BTreeMap::new();
    for pc in pcs {
        let a = hex(pc)? as i128 - shift;
        if !(0..=u64::MAX as i128).contains(&a) {
            continue;
        }
        let a = a as u64;
        let i = addrs.partition_point(|v| *v <= a);
        if i == 0 {
            continue;
        }
        let i = i - 1;
        let off = a - addrs[i];
        if off <= 0x20000 {
            out.insert(pc.clone(), format!("{}+{off:#x}", names[i]));
        }
    }
    Ok(out)
}
fn func(s: &str) -> &str {
    s.split('+').next().unwrap_or(s)
}
pub fn where_pc(pc: &str, sym: &BTreeMap<String, String>) -> Result<String> {
    if hex(pc)? == 0 {
        return Ok("(zero PC)".into());
    }
    if let Some(s) = sym.get(pc) {
        return Ok(func(s).into());
    }
    let n = hex(pc)?;
    Ok(if n < 0x800000000000 && (pc.len() > 8 || n < 0x80000000) {
        "(below kernel address range)"
    } else {
        "(kernel, not in ntoskrnl index)"
    }
    .into())
}
pub fn likely_caller(row: &Value, sym: &BTreeMap<String, String>) -> Result<String> {
    let own = where_pc(row["pc"].as_str().ok_or("missing pc")?, sym)?;
    let mut seen = Vec::new();
    let words = row
        .get("stack")
        .and_then(Value::as_array)
        .filter(|v| !v.is_empty())
        .or_else(|| row.get("callers").and_then(Value::as_array));
    if let Some(words) = words {
        for w in words {
            let name = where_pc(w.as_str().ok_or("invalid caller")?, sym)?;
            if name.starts_with('(') || name == own || seen.contains(&name) {
                continue;
            }
            seen.push(name);
            if seen.len() == 2 {
                break;
            }
        }
    }
    Ok(if seen.is_empty() {
        "?".into()
    } else {
        seen.join(" <- ")
    })
}
fn count_ordered<T: Ord + Clone>(values: impl Iterator<Item = T>) -> Vec<(T, usize)> {
    let mut out: Vec<(T, usize)> = Vec::new();
    let mut index: BTreeMap<T, usize> = BTreeMap::new();
    for v in values {
        if let Some(&i) = index.get(&v) {
            out[i].1 += 1
        } else {
            index.insert(v.clone(), out.len());
            out.push((v, 1));
        }
    }
    out.sort_by(|a, b| b.1.cmp(&a.1));
    out
}
pub fn report(
    root: &Path,
    arch: &str,
    samples: &Path,
    serial: &str,
    top: usize,
) -> Result<Vec<String>> {
    let mut rows = Vec::new();
    for line in fs::read_to_string(samples)?
        .lines()
        .filter(|s| !s.trim().is_empty())
    {
        rows.push(serde_json::from_str::<Value>(line)?);
    }
    if rows.is_empty() {
        return Ok(vec![format!(
            "[pc-sample] no samples in {}",
            samples.display()
        )]);
    }
    let mut uniq = BTreeSet::new();
    for r in &rows {
        uniq.insert(r["pc"].as_str().ok_or("missing pc")?.to_owned());
        for (key, limit) in [("callers", 2), ("stack", usize::MAX)] {
            if let Some(v) = r[key].as_array() {
                for pc in v.iter().take(limit) {
                    uniq.insert(pc.as_str().ok_or("invalid stack pc")?.to_owned());
                }
            }
        }
    }
    let relocation = verdict::kernel_relocation(arch, serial);
    let sym = resolve_batch(
        root,
        arch,
        &uniq.into_iter().collect::<Vec<_>>(),
        serial.contains("ntkrnlmp.exe"),
        relocation.as_deref(),
    )?;
    let cpus: BTreeSet<_> = rows.iter().filter_map(|r| r["cpu"].as_u64()).collect();
    let mut out = Vec::new();
    for cpu in cpus {
        let mine: Vec<_> = rows.iter().filter(|r| r["cpu"] == cpu).collect();
        let halted = mine.iter().filter(|r| r["halted"] == true).count();
        let known = mine.iter().filter(|r| r["halted"].is_boolean()).count();
        let state = if known == 0 {
            "halt state unavailable".to_owned()
        } else if known == mine.len() {
            format!("{:.0}% halted", 100.0 * halted as f64 / known as f64)
        } else {
            format!(
                "{:.0}% halted among {known} known samples; {} unknown",
                100.0 * halted as f64 / known as f64,
                mine.len() - known
            )
        };
        out.push(format!(
            "[pc-sample] CPU#{cpu}: {} samples, {state}",
            mine.len()
        ));
    }
    let busy: Vec<_> = rows.iter().filter(|r| r["halted"] != true).collect();
    if busy.is_empty() {
        return Ok(out);
    }
    let names = busy
        .iter()
        .map(|r| where_pc(r["pc"].as_str().ok_or("missing pc")?, &sym))
        .collect::<Result<Vec<_>>>()?;
    let heading = if rows.iter().any(|r| !r["halted"].is_boolean()) {
        "sampled vCPU locations, excluding known halted vCPUs"
    } else {
        "where non-halted vCPUs were found"
    };
    out.push(format!("[pc-sample] {heading} ({} samples):", busy.len()));
    for (name, n) in count_ordered(names.into_iter()).into_iter().take(top) {
        out.push(format!(
            "  {:5.1}%  {name}",
            100.0 * n as f64 / busy.len() as f64
        ));
    }
    let mut sites = Vec::new();
    for r in &busy {
        let name = where_pc(r["pc"].as_str().unwrap(), &sym)?;
        if !name.starts_with('(') {
            sites.push((name, likely_caller(r, &sym)?));
        }
    }
    let sites = count_ordered(sites.into_iter());
    if !sites.is_empty() {
        out.push(
            "[pc-sample] kernel PC <- likely callers (first 2 other symbols found on the stack):"
                .into(),
        );
        for ((name, caller), n) in sites.into_iter().take(top) {
            out.push(format!(
                "  {:5.1}%  {name}  <-  {caller}",
                100.0 * n as f64 / busy.len() as f64
            ));
        }
    }
    Ok(out)
}
pub fn dispatch(root: &Path, request: &Value, host: &mut dyn Host) -> Result<Value> {
    let a = &request["kwargs"];
    let op = request["op"]
        .as_str()
        .ok_or("missing op")?
        .trim_start_matches("pcsample_");
    let sym = || {
        serde_json::from_value::<BTreeMap<String, String>>(a["sym"].clone()).map_err(Error::from)
    };
    Ok(match op {
        "env_period" => json!(env_period(a["raw"].as_str().unwrap_or(""))?),
        "parse_sample" => json!(parse_sample(a["state"].as_str().ok_or("missing state")?)?),
        "sample_once" => json!(locked_sample(host)?),
        "callers" => json!(callers(
            host,
            a["cpu"].as_u64().ok_or("missing cpu")?,
            a["fp"].as_str(),
            a["depth"].as_u64().unwrap_or(CALLER_DEPTH as u64) as usize
        )),
        "stack_words" => json!(stack_words(
            host,
            a["cpu"].as_u64().ok_or("missing cpu")?,
            a["sp"].as_str()
        )),
        "resolve_batch" => json!(resolve_batch(
            root,
            a["arch"].as_str().ok_or("missing arch")?,
            &serde_json::from_value::<Vec<String>>(a["pcs"].clone())?,
            a["mp"].as_bool().unwrap_or(false),
            a["reloc"].as_str()
        )?),
        "where" => json!(where_pc(a["pc"].as_str().ok_or("missing pc")?, &sym()?)?),
        "likely_caller" => json!(likely_caller(&a["row"], &sym()?)?),
        "run" => run_loop(
            host,
            Path::new(a["out"].as_str().ok_or("missing output")?),
            a["period"].as_f64().ok_or("missing period")?,
        )?,
        "report" => json!(report(
            root,
            a["arch"].as_str().ok_or("missing arch")?,
            Path::new(a["samples"].as_str().ok_or("missing samples")?),
            a["serial"].as_str().unwrap_or(""),
            a["top"].as_u64().unwrap_or(15) as usize
        )?),
        _ => return Err(format!("unknown PC sampler operation {op}").into()),
    })
}
pub fn locked_sample(host: &mut dyn Host) -> Result<Vec<Value>> {
    host.call("diagnostic_begin_sample", json!({}))?;
    let result = sample_once(host);
    let unlock = host.call("diagnostic_end_sample", json!({}));
    unlock?;
    result
}
pub fn run_loop(host: &mut dyn Host, out: &Path, period: f64) -> Result<Value> {
    use std::{
        io::Write,
        time::{Duration, Instant},
    };
    if !(0.05..=60.0).contains(&period) {
        return Err("PC sampling period must be 0.05..60 seconds".into());
    }
    let mut file = fs::OpenOptions::new().create(true).append(true).open(out)?;
    let start = Instant::now();
    let mut samples = 0;
    loop {
        let rows = match locked_sample(host) {
            Ok(v) => v,
            Err(e) if samples > 0 && e.kind.as_deref() == Some("QmpClosed") => {
                eprintln!("[pc-sample] monitor closed after {samples} snapshots ({e})");
                return Ok(json!({"samples":samples}));
            }
            Err(e) => return Err(e),
        };
        if rows.is_empty() {
            return Err("monitor returned no vCPU registers for the requested PC sample".into());
        }
        let t = (start.elapsed().as_secs_f64() * 100.0).round() / 100.0;
        for mut row in rows {
            row["t"] = json!(t);
            writeln!(file, "{row}")?;
        }
        file.flush()?;
        samples += 1;
        std::thread::sleep(Duration::from_secs_f64(period));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Script {
        commands: Vec<String>,
        empty: bool,
    }
    impl Host for Script {
        fn call(&mut self, op: &str, a: Value) -> Result<Value> {
            self.commands.push(if op == "diagnostic_hmp" {
                a["command"].as_str().unwrap().into()
            } else {
                op.into()
            });
            match op {
            "diagnostic_begin_sample"|"diagnostic_end_sample"=>Ok(Value::Null),
            "diagnostic_hmp"=>Ok(json!(match a["command"].as_str().unwrap(){"info registers -a" if !self.empty=>"CPU#0\nRIP=fffff80000401010 RBP=fffff80000900f00 RSP=fffff80000900f10 HLT=0\n",command if command.starts_with("x ")=>"unsupported memory access",_=>""})),
            _=>Err("unexpected callback".into()),
        }
        }
    }
    #[test]
    fn a_snapshot_holds_the_monitor_lock_and_restores_cpu_selection_after_unwind_failure() {
        let mut host = Script {
            commands: vec![],
            empty: false,
        };
        let rows = locked_sample(&mut host).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["pc"], "fffff80000401010");
        assert_eq!(host.commands.first().unwrap(), "diagnostic_begin_sample");
        assert_eq!(host.commands.last().unwrap(), "diagnostic_end_sample");
        assert_eq!(
            host.commands
                .iter()
                .filter(|s| s.as_str() == "cpu 0")
                .count(),
            4
        );
        assert_eq!(rows[0]["callers"], json!([]));
        assert_eq!(rows[0]["stack"], json!([]));
    }
    #[test]
    fn a_requested_sampler_refuses_an_unrecognizable_register_dump() {
        let temp = tempfile::tempdir().unwrap();
        let mut host = Script {
            commands: vec![],
            empty: true,
        };
        let result = run_loop(&mut host, &temp.path().join("samples.jsonl"), 0.05);
        assert!(result.unwrap_err().message.contains("no vCPU registers"));
        assert_eq!(host.commands.last().unwrap(), "diagnostic_end_sample");
    }
}

#[cfg(test)]
mod unknown_execution_state_tests {
    use super::*;
    #[test]
    fn missing_halt_state_is_not_a_zero_percent_halted_measurement() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("qemu")).unwrap();
        fs::write(
            root.path().join("qemu/symbols-arm64-mp.tsv"),
            "fffff80000401000\tKiIdleLoop\tidle.rs:1\n",
        )
        .unwrap();
        let rows = [
            json!({"cpu":0,"pc":"fffff80000401010","halted":null}),
            json!({"cpu":0,"pc":"000000007fd25204","halted":null}),
            json!({"cpu":1,"pc":"0000000000000000","halted":null}),
        ];
        let path = root.path().join("samples.jsonl");
        fs::write(
            &path,
            rows.iter().map(|r| format!("{r}\n")).collect::<String>(),
        )
        .unwrap();
        let output = report(root.path(), "arm64", &path, "ntkrnlmp.exe", 15)
            .unwrap()
            .join("\n");
        assert!(output.contains("CPU#0: 2 samples, halt state unavailable"));
        assert!(!output.contains("0% halted"));
        assert!(!output.contains("non-halted"));
        assert!(output.contains("(below kernel address range)"));
        assert!(output.contains("(zero PC)"));
        assert!(output.contains("KiIdleLoop"));
    }
}
