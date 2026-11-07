//! Trace CLI mechanisms: retention, transcript identity, quiescence and progress.
use crate::{callback::Host, verdict};
use crate::{record_replay, Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};
fn text<'a>(a: &'a Value, k: &str) -> Result<&'a str> {
    a[k].as_str()
        .ok_or_else(|| Error::new(format!("missing trace {k}")))
}
pub fn resolve_trace(root: &Path, raw: &str) -> Result<PathBuf> {
    let direct = Path::new(raw);
    if direct.is_dir() {
        return Ok(direct.into());
    }
    if raw == "latest" {
        let mut dirs = if root.is_dir() {
            fs::read_dir(root)?
                .map(|v| v.map(|e| e.path()))
                .collect::<std::io::Result<Vec<_>>>()?
        } else {
            vec![]
        };
        dirs.retain(|p| p.join("manifest.json").is_file());
        let mut times = dirs
            .into_iter()
            .map(|p| Ok((fs::metadata(&p)?.modified()?, p)))
            .collect::<Result<Vec<_>>>()?;
        times.sort_by(|a, b| a.0.cmp(&b.0));
        return times
            .pop()
            .map(|(_, p)| p)
            .ok_or_else(|| Error::new(format!("no traces under {}", root.display())));
    }
    let candidate = root.join(raw);
    if candidate.is_dir() {
        return Ok(candidate);
    }
    Err(format!("no such trace: {raw} (looked in {})", root.display()).into())
}
pub fn await_quiescence(trace: &Path, settle: f64, limit: f64) -> Result<u64> {
    if !settle.is_finite() || !limit.is_finite() || settle < 0.0 || limit <= 0.0 {
        return Err("invalid trace quiescence bounds".into());
    }
    let path = trace.join("replay.bin");
    let deadline = Instant::now() + Duration::from_secs_f64(limit);
    let mut last = None;
    let mut stable = Instant::now();
    loop {
        if Instant::now() >= deadline {
            return Err(format!("trace still growing after {limit:.0}s; refusing to finalise a potentially truncated recording at {} bytes",last.unwrap_or(0)).into());
        }
        let size = match fs::metadata(&path) {
            Ok(m) if m.is_file() => m.len(),
            Ok(_) => 0,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => 0,
            Err(e) => return Err(e.into()),
        };
        if last != Some(size) {
            last = Some(size);
            stable = Instant::now();
        } else if stable.elapsed().as_secs_f64() >= settle {
            return Ok(size);
        }
        thread::sleep(Duration::from_secs_f64(
            2.0_f64.min(limit).min(settle.max(0.01)),
        ));
    }
}
pub fn keep_serial(trace: &Path, name: &str, expect: &str) -> Result<Value> {
    let Ok(run) = record_replay::last_run(trace) else {
        return Ok(json!({"path":null}));
    };
    if run["mode"].as_str() != Some(expect) {
        return Ok(
            json!({"path":null,"warning":format!("[rr] refusing to keep serial: last run through this trace was {:?}, expected {expect:?}",run["mode"].as_str())}),
        );
    }
    let src = Path::new(run["serial_log"].as_str().unwrap_or(""));
    if !src.is_file() {
        return Ok(json!({"path":null}));
    }
    if Path::new(name).components().count() != 1 {
        return Err("serial transcript name must be a filename".into());
    }
    let dst = trace.join(name);
    fs::copy(src, &dst)?;
    Ok(json!({"path":dst}))
}
pub fn guest_lines(path: &Path) -> Result<Vec<String>> {
    let raw = vibeos_build::read(path)?;
    let lines = verdict::py_lines(&raw);
    let i = lines
        .iter()
        .position(|s| !s.trim().is_empty() && !s.starts_with('#'))
        .unwrap_or(lines.len());
    let re = regex::Regex::new(r"\[\+\s*\d+\.\d+\] ?")?;
    let joined = lines[i..].join("\n");
    let cleaned = re.replace_all(&joined, "").replace('\r', "\n");
    Ok(verdict::py_lines(&cleaned)
        .into_iter()
        .filter(|s| !s.trim().is_empty())
        .collect())
}
pub fn serial_divergence(a: &[String], b: &[String]) -> Option<(usize, String)> {
    let limit = a.len().min(b.len());
    for (i, (x, y)) in a.iter().zip(b).enumerate() {
        if i == 0 && (x.ends_with(y) || y.ends_with(x)) {
            continue;
        }
        if i + 1 == limit && (x.starts_with(y) || y.starts_with(x)) {
            continue;
        }
        if x != y {
            return Some((
                i + 1,
                format!(
                    "record: {}\nreplay: {}",
                    x.chars().take(160).collect::<String>(),
                    y.chars().take(160).collect::<String>()
                ),
            ));
        }
    }
    None
}
fn commas(n: u64) -> String {
    let chars = n.to_string();
    let mut out = String::new();
    for (i, c) in chars.chars().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            out.push(',')
        }
        out.push(c);
    }
    out
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Progress {
    pub total: u64,
    pub max_pos: u64,
    pub stalled_for: f64,
}
pub fn progress_report(p: &Progress) -> Option<String> {
    if p.total == 0 || p.max_pos == 0 {
        return None;
    }
    let pct = 100.0 * p.max_pos as f64 / p.total as f64;
    Some(if p.stalled_for >= 60.0 {
        format!("replay appears WEDGED: read {} of {} trace bytes ({pct:.1}%) and stopped consuming events for {:.0}s while its trace handle stayed open. Inspect the replay and host scheduling before waiting longer. See docs/engineering/qemu-record-replay.md.",commas(p.max_pos),commas(p.total),p.stalled_for)
    } else {
        format!(
            "replay consumed {} of {} trace bytes ({pct:.1}%)",
            commas(p.max_pos),
            commas(p.total)
        )
    })
}
pub fn watch_progress(trace: &Path, period: f64, host: &mut dyn Host) -> Result<Progress> {
    if !period.is_finite() || period <= 0.0 {
        return Err("trace progress period must be finite and positive".into());
    }
    let mut p = Progress {
        total: fs::metadata(trace.join("replay.bin"))?.len(),
        ..Progress::default()
    };
    let mut last = None;
    let mut next = Instant::now() + Duration::from_secs_f64(period);
    loop {
        if host.call("rr_cli_alive", json!({}))?.as_bool() != Some(true) {
            return Ok(p);
        }
        if Instant::now() >= next {
            if let Some(pos) = record_replay::trace_position(trace) {
                p.max_pos = p.max_pos.max(pos);
                p.stalled_for = if last == Some(pos) {
                    p.stalled_for + period
                } else {
                    0.0
                };
                last = Some(pos);
            }
            next = Instant::now() + Duration::from_secs_f64(period);
        }
        thread::sleep(
            next.saturating_duration_since(Instant::now())
                .min(Duration::from_millis(500)),
        );
    }
}
fn tree_size(path: &Path) -> Result<u64> {
    let mut total = 0u64;
    for item in fs::read_dir(path)? {
        let item = item?;
        let p = item.path();
        if item.file_type()?.is_dir() {
            total = total
                .checked_add(tree_size(&p)?)
                .ok_or("trace size overflow")?;
        } else if p.is_file() {
            total = total
                .checked_add(fs::metadata(p)?.len())
                .ok_or("trace size overflow")?;
        }
    }
    Ok(total)
}
pub fn inventory(root: &Path) -> Result<Vec<Value>> {
    if !root.is_dir() {
        return Ok(vec![]);
    }
    let mut paths = fs::read_dir(root)?
        .map(|e| e.map(|e| e.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    paths.sort();
    let mut out = Vec::new();
    for path in paths {
        if !path.join("manifest.json").is_file() {
            continue;
        }
        if fs::symlink_metadata(&path)?.file_type().is_symlink() {
            return Err(format!("refusing a symlinked trace directory: {}", path.display()).into());
        }
        let m = record_replay::load_manifest(&path)?;
        let bytes = fs::metadata(path.join("replay.bin")).map_or(0, |m| m.len());
        let modified = fs::metadata(&path)?
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| Error::new(e.to_string()))?
            .as_secs_f64();
        out.push(json!({"path":path,"name":path.file_name().unwrap().to_string_lossy(),"manifest":m,"bytes":bytes,"total":tree_size(&path)?,"modified":modified}));
    }
    Ok(out)
}
pub fn prune(root: &Path, keep: usize, names: &[String], apply: bool) -> Result<Value> {
    let mut all = inventory(root)?;
    all.sort_by(|a, b| {
        b["modified"]
            .as_f64()
            .unwrap()
            .partial_cmp(&a["modified"].as_f64().unwrap())
            .unwrap()
    });
    let count = all.len();
    let doomed: Vec<_> = all
        .into_iter()
        .enumerate()
        .filter(|(i, v)| {
            if names.is_empty() {
                *i >= keep
            } else {
                names.iter().any(|n| v["name"] == n.as_str())
            }
        })
        .map(|(_, v)| v)
        .collect();
    let mut total = 0u64;
    for row in &doomed {
        total = total
            .checked_add(row["total"].as_u64().unwrap())
            .ok_or("trace retention size overflow")?;
        if apply {
            let path = Path::new(row["path"].as_str().unwrap());
            if fs::symlink_metadata(path)?.file_type().is_symlink() {
                return Err("trace changed into a symlink before deletion".into());
            }
            fs::remove_dir_all(path)?;
        }
    }
    Ok(json!({"count":count,"keep":keep,"doomed":doomed,"bytes":total,"applied":apply}))
}

/// A useful guest-crash recording can succeed, but explicitly requested host
/// evidence must exist. Keep this verdict separate from trace finalisation.
pub fn validate_record_diagnostics(trace: &Path, ram: bool, pc: bool) -> Result<Value> {
    use std::io::{BufRead, BufReader};
    let check = || -> Result<Value> {
        let mut counts = serde_json::Map::new();
        if ram || pc {
            let run = record_replay::last_run(trace)?;
            if run["mode"] != "record" {
                return Err("diagnostics belong to a non-record run".into());
            }
            let serial = Path::new(
                run["serial_log"]
                    .as_str()
                    .ok_or("record has no serial path")?,
            );
            let work = serial.parent().ok_or("record serial has no directory")?;
            for (requested, kind, path) in [
                (ram, "rr", trace.join("samples.jsonl")),
                (pc, "pc", work.join("pcsamples.jsonl")),
            ] {
                if !requested {
                    continue;
                }
                let result = work.join(format!("vm_diagnostic_sidecar-{kind}.json"));
                if result.is_file() && fs::metadata(&result)?.len() > 0 {
                    let value: Value = serde_json::from_slice(&fs::read(&result)?)?;
                    if let Some(error) = value.get("error") {
                        return Err(format!("requested {kind} sampler failed: {error}").into());
                    }
                }
                let mut count = 0u64;
                for line in BufReader::new(fs::File::open(&path).map_err(|e| {
                    Error::new(format!(
                        "requested {kind} samples unavailable at {}: {e}",
                        path.display()
                    ))
                })?)
                .lines()
                {
                    let row: Value = serde_json::from_str(&line?)?;
                    let valid = if kind == "rr" {
                        row["icount"].as_u64().is_some()
                            && row["regs"].as_str().is_some_and(|s| !s.trim().is_empty())
                            && row["mem"].as_array().is_some_and(|regions| {
                                !regions.is_empty()
                                    && regions.iter().all(|region| {
                                        region["base"].as_u64().is_some()
                                            && region["hashes"].as_array().is_some_and(|hashes| {
                                                !hashes.is_empty()
                                                    && hashes.iter().all(|h| {
                                                        h.as_str().is_some_and(|s| {
                                                            s.len() == 16
                                                                && s.bytes()
                                                                    .all(|b| b.is_ascii_hexdigit())
                                                        })
                                                    })
                                            })
                                    })
                            })
                    } else {
                        row["cpu"].as_u64().is_some()
                            && row["t"].as_f64().is_some()
                            && row["pc"].as_str().is_some_and(|s| {
                                !s.is_empty()
                                    && s.trim_start_matches("0x")
                                        .bytes()
                                        .all(|b| b.is_ascii_hexdigit())
                            })
                    };
                    if !valid {
                        return Err(format!(
                            "malformed requested {kind} sample {} at {}",
                            count + 1,
                            path.display()
                        )
                        .into());
                    }
                    count += 1;
                }
                if count == 0 {
                    return Err(format!(
                        "requested {kind} sampler produced no samples at {}",
                        path.display()
                    )
                    .into());
                }
                counts.insert(kind.into(), json!(count));
            }
        }
        Ok(json!({"ok":true,"samples":counts}))
    };
    let verdict = match check() {
        Ok(value) => value,
        Err(error) => json!({"ok":false,"error":error.message}),
    };
    fs::write(
        trace.join("record-diagnostics.json"),
        format!("{}\n", serde_json::to_string_pretty(&verdict)?),
    )?;
    Ok(verdict)
}

pub fn dispatch(request: &Value, host: &mut dyn Host) -> Result<Value> {
    let a = &request["kwargs"];
    let op = text(request, "op")?.trim_start_matches("rr_cli_");
    let trace = Path::new(a["trace_dir"].as_str().unwrap_or(""));
    Ok(match op {
        "resolve_trace" => json!(resolve_trace(
            Path::new(text(a, "trace_root")?),
            text(a, "raw")?
        )?),
        "await_quiescence" => json!(await_quiescence(
            trace,
            a["settle"].as_f64().unwrap_or(6.0),
            a["limit"].as_f64().unwrap_or(420.0)
        )?),
        "validate_record_diagnostics" => validate_record_diagnostics(
            trace,
            a["ram"].as_bool().unwrap_or(false),
            a["pc"].as_bool().unwrap_or(false),
        )?,
        "divergence_status" => crate::replay_sample::comparison_status(&a["verdict"])?,
        "keep_serial" => keep_serial(trace, text(a, "name")?, text(a, "expect_mode")?)?,
        "guest_lines" => json!(guest_lines(Path::new(text(a, "path")?))?),
        "serial_divergence" => json!(serial_divergence(
            &serde_json::from_value::<Vec<String>>(a["record"].clone())?,
            &serde_json::from_value::<Vec<String>>(a["replay"].clone())?
        )),
        "progress_report" => json!(progress_report(&serde_json::from_value(
            a["state"].clone()
        )?)),
        "watch_progress" => json!(watch_progress(
            trace,
            a["period"].as_f64().unwrap_or(20.0),
            host
        )?),
        "inventory" => json!(inventory(Path::new(text(a, "trace_root")?))?),
        "prune" => prune(
            Path::new(text(a, "trace_root")?),
            a["keep"].as_u64().unwrap_or(0) as usize,
            &serde_json::from_value::<Vec<String>>(a.get("names").cloned().unwrap_or(json!([])))?,
            a["apply"].as_bool().unwrap_or(false),
        )?,
        _ => return Err(format!("unknown trace CLI operation {op}").into()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn growing_trace_cannot_be_finalised() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("replay.bin"), b"recording").unwrap();
        let error = await_quiescence(dir.path(), 1.0, 0.03).unwrap_err();
        assert!(error.message.contains("refusing to finalise"));
    }
    #[test]
    fn prune_refuses_symlinked_trace_without_deleting_target() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        fs::write(target.path().join("manifest.json"), b"{}").unwrap();
        symlink(target.path(), root.path().join("outside")).unwrap();
        assert!(prune(root.path(), 0, &[], true).is_err());
        assert!(target.path().join("manifest.json").is_file());
    }
}

#[cfg(test)]
mod diagnostic_record_tests {
    use super::*;
    #[test]
    fn requested_evidence_must_exist_and_worker_failure_wins() {
        let temp = tempfile::tempdir().unwrap();
        let trace = temp.path();
        let work = trace.join("boot");
        fs::create_dir(&work).unwrap();
        fs::write(
            trace.join("last-run.json"),
            json!({"mode":"record","serial_log":work.join("serial.log")}).to_string(),
        )
        .unwrap();
        assert_eq!(
            validate_record_diagnostics(trace, false, false).unwrap()["ok"],
            true
        );
        fs::write(trace.join("samples.jsonl"), "").unwrap();
        assert_eq!(
            validate_record_diagnostics(trace, true, false).unwrap()["ok"],
            false
        );
        fs::write(trace.join("samples.jsonl"),format!("{}\n",json!({"icount":12,"regs":"CPU#0 PC=0","mem":[{"base":0,"hashes":["878633aa32a3b150"]}]}))).unwrap();
        // A bugcheck in serial is useful trace content, independent of host evidence.
        fs::write(work.join("serial.log"), "*** BUGCHECK 0xA").unwrap();
        assert_eq!(
            validate_record_diagnostics(trace, true, false).unwrap()["ok"],
            true
        );
        fs::write(
            work.join("vm_diagnostic_sidecar-rr.json"),
            json!({"error":{"kind":"QmpTransport","message":"timeout"}}).to_string(),
        )
        .unwrap();
        let bad = validate_record_diagnostics(trace, true, false).unwrap();
        assert_eq!(bad["ok"], false);
        assert!(bad["error"].as_str().unwrap().contains("timeout"));
        fs::write(
            work.join("pcsamples.jsonl"),
            format!("{}\n", json!({"cpu":0,"t":0.0,"pc":"fffff80000400000"})),
        )
        .unwrap();
        assert_eq!(
            validate_record_diagnostics(trace, false, true).unwrap()["ok"],
            true
        );
    }
}
