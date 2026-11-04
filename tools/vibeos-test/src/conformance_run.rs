//! Native conformance sweep ownership and ratchet publication.
//! Parsing policy is shared with `conformance`; Host owns console/ledger adapters.
use crate::{callback::Host, Error, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};
fn s<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}
fn flag(v: &Value, k: &str) -> bool {
    v.get(k).and_then(Value::as_bool).unwrap_or(false)
}
fn n(v: &Value, k: &str, d: i64) -> i64 {
    v.get(k).and_then(Value::as_i64).unwrap_or(d)
}
fn txt(p: &Path) -> Result<String> {
    Ok(String::from_utf8_lossy(&fs::read(p)?).into_owned())
}
fn emit(h: &mut dyn Host, t: impl AsRef<str>, err: bool) -> Result<()> {
    h.call(
        "print",
        json!({"text":t.as_ref(),"stderr":err,"flush":true}),
    )?;
    Ok(())
}
fn parse(root: &Path, op: &str, a: Value) -> Result<Value> {
    crate::conformance::dispatch(
        root,
        &json!({"op":format!("conformance_parse_{op}"),"kwargs":a}),
    )
}
fn env(a: &Value) -> Result<BTreeMap<String, String>> {
    Ok(a.get("env")
        .cloned()
        .map(serde_json::from_value)
        .transpose()?
        .unwrap_or_else(|| std::env::vars().collect()))
}
fn farm(a: &Value, e: &BTreeMap<String, String>) -> PathBuf {
    a.get("farm_dir")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(e.get("HOME").cloned().unwrap_or_default()).join("vibeos-flake-farm")
        })
}
fn ledger(a: &Value, e: &BTreeMap<String, String>) -> PathBuf {
    a.get("ledger")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .unwrap_or_else(|| farm(a, e).join("conformance.jsonl"))
}
fn baseline(root: &Path, a: &Value, args: &Value) -> PathBuf {
    let base = s(args, "arch").split('-').next().unwrap_or("");
    let smp = n(args, "smp", 0);
    a.get("baseline_dir")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("tools/conformance-baseline"))
        .join(base)
        .join(if smp > 1 {
            format!("kmtest-smp{smp}.txt")
        } else {
            "kmtest.txt".into()
        })
}
pub fn cache_bools(path: &Path) -> BTreeMap<String, bool> {
    txt(path)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            line.split_once(":BOOL=").map(|(k, v)| {
                (
                    k.trim().to_owned(),
                    matches!(
                        v.trim().to_uppercase().as_str(),
                        "ON" | "TRUE" | "1" | "YES"
                    ),
                )
            })
        })
        .collect()
}
pub fn media_payload_gap(build: &Path) -> Option<String> {
    let stamp = build.join("payload-selection.json");
    let fix = "drop --no-build (conformance builds with VIBEOS_PAYLOAD=kmtest) or build with it";
    let raw = match fs::read(&stamp) {
        Ok(x) => x,
        Err(_) => {
            return Some(format!(
                "{} is missing, so nothing says what this media holds; {fix}",
                stamp.display()
            ))
        }
    };
    let value: Value = match serde_json::from_slice(&raw) {
        Ok(x) => x,
        Err(e) => return Some(format!("{} is unreadable ({e}); {fix}", stamp.display())),
    };
    let names = value.get("payloads").cloned().unwrap_or(json!([]));
    if !names
        .as_array()
        .is_some_and(|a| a.iter().any(|v| v == "kmtest"))
    {
        return Some(format!(
            "this media was built with payloads {names}, not 'kmtest'; {fix}"
        ));
    }
    let c = cache_bools(&build.join("CMakeCache.txt"));
    if !c.is_empty()
        && !c.get("ENABLE_ROSTESTS_KMTESTS").copied().unwrap_or(false)
        && !c.get("ENABLE_ROSTESTS").copied().unwrap_or(false)
    {
        return Some(format!("{} was configured without the kmtest corpus (ENABLE_ROSTESTS_KMTESTS=OFF); drop --no-build (conformance passes --kmtests) or `vibeos-cli build --arch <arch> --kmtests` with VIBEOS_PAYLOAD=kmtest",build.file_name().unwrap_or_default().to_string_lossy()));
    }
    None
}
pub fn provisioning_gaps(root: &Path, arch: &str) -> Vec<String> {
    let mut gaps = Vec::new();
    let tc = root.join(".build-tools/llvm-mingw");
    let shared = root
        .parent()
        .and_then(|p| fs::read_dir(p).ok())
        .is_some_and(|mut r| {
            r.any(|e| {
                e.ok()
                    .is_some_and(|x| x.file_name().to_string_lossy().starts_with("llvm-mingw-"))
            })
        });
    if !tc.exists() && !shared {
        gaps.push(format!("missing {} (pinned clang) -> CMAKE_CONFIGURE_FAIL 'no llvm-mingw toolchain found'\n    fix: mkdir -p {} && ln -s <path-to>/llvm-mingw-*-ubuntu-*-x86_64 {}",tc.display(),tc.parent().unwrap().display(),tc.display()));
    }
    let p = root.join(format!("tools/payloads/kmtest/kmtest-sweep-{arch}.exe"));
    if !p.exists() {
        gaps.push(format!("missing {} -> mkramdisk fails with \"payload 'kmtest': src missing\"\n    fix: bash tools/payloads/kmtest/build.sh {arch}",p.display()));
    }
    if arch == "arm64" {
        for name in ["QEMU_EFI.fd", "efivars.fd"] {
            let p = root.join("qemu").join(name);
            if !p.exists() {
                gaps.push(format!("missing {} (gitignored firmware blob)\n    fix: cp <main-checkout>/qemu/{name} {}",p.display(),p.display()));
            }
        }
    }
    gaps
}
/// Intentional tightening: even a payload-bearing image must prove current source.
fn freshness(root: &Path, a: &Value, arch: &str, e: &BTreeMap<String, String>) -> Result<Value> {
    let cfg = a
        .get("cfgs")
        .and_then(|v| v.get(arch))
        .ok_or_else(|| Error::new(format!("missing provenance config for {arch}")))?;
    vibeos_build::provenance::dispatch(
        root,
        &json!({"op":"provenance_check_staleness","kwargs":{"repo":root,"arch":arch,"cfg":cfg,"env":e,"store_root":a.get("store_root")}}),
    )
}
fn serials(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for e in fs::read_dir(dir)? {
        let p = e?.path();
        let name = p.file_name().unwrap_or_default().to_string_lossy();
        if p.is_file() && name.contains("serial") && name.ends_with(".log") {
            out.push(p)
        }
    }
    out.sort();
    Ok(out)
}
fn preserve(
    root: &Path,
    farm: &Path,
    run: &Path,
    arch: &str,
    shape: &str,
    e: &BTreeMap<String, String>,
) -> Result<Option<PathBuf>> {
    let logs = serials(run)?;
    if logs.is_empty() {
        return Ok(None);
    }
    let mp = logs
        .iter()
        .any(|p| txt(p).is_ok_and(|t| t.contains("ntkrnlmp.exe")));
    let tag = regex::Regex::new(r"[^A-Za-z0-9_.-]+")
        .unwrap()
        .replace_all(shape, "_")
        .trim_matches('_')
        .to_owned();
    let dest = farm.join("fails/auto").join(format!(
        "{}-{tag}",
        run.file_name().unwrap_or_default().to_string_lossy()
    ));
    fs::create_dir_all(&dest)?;
    for entry in fs::read_dir(run)? {
        let p = entry?.path();
        let name = p.file_name().unwrap_or_default().to_string_lossy();
        if p.is_file()
            && (logs.contains(&p)
                || matches!(
                    name.as_ref(),
                    "summary.json" | "full.log" | "hang-cpustate.txt"
                )
                || name.ends_with("-results.txt")
                || name.ends_with(".png"))
        {
            fs::copy(&p, dest.join(name.as_ref()))?;
        }
    }
    let (head, dirty) = crate::farm::git_state(root, e)?;
    let mut manifest = json!({"run_id":run.file_name().unwrap_or_default().to_string_lossy(),"arch":arch,"classification":shape,"mp_kernel":mp,"head":head,"dirty":dirty,"kernel_sha12":null,"kernels_dir":null});
    let base = root.join(format!("build-{arch}"));
    let kernel = base.join(if mp {
        "ntoskrnl/ntkrnlmp/ntkrnlmp.exe"
    } else {
        "ntoskrnl/ntoskrnl.exe"
    });
    if kernel.is_file() {
        let hash = format!("{:x}", Sha256::digest(fs::read(&kernel)?));
        let hash = &hash[..12];
        let dst = farm.join("kernels").join(hash);
        if !dst.is_dir() {
            fs::create_dir_all(dst.parent().unwrap())?;
            let temp = tempfile::Builder::new()
                .prefix(".kernel-")
                .tempdir_in(dst.parent().unwrap())?;
            fs::copy(&kernel, temp.path().join(kernel.file_name().unwrap()))?;
            for p in [
                root.join(format!(
                    "qemu/symbols-{arch}{}.tsv",
                    if mp { "-mp" } else { "" }
                )),
                base.join(if mp {
                    "hal/halmp/halmp.dll"
                } else {
                    "hal/hal/hal.dll"
                }),
            ] {
                if p.is_file() {
                    fs::copy(&p, temp.path().join(p.file_name().unwrap()))?;
                }
            }
            match fs::rename(temp.path(), &dst) {
                Ok(()) => {}
                Err(_) if dst.is_dir() => {}
                Err(e) => return Err(e.into()),
            }
        }
        manifest["kernel_sha12"] = json!(hash);
        manifest["kernels_dir"] = json!(dst);
    }
    fs::write(
        dest.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(Some(dest))
}
fn command(
    h: &mut dyn Host,
    root: &Path,
    argv: Vec<String>,
    e: &BTreeMap<String, String>,
    timeout: Option<i64>,
) -> Result<Value> {
    h.call(
        "command",
        json!({"argv":argv,"cwd":root,"env":e,"timeout":timeout,"capture":false}),
    )
}
fn once(
    root: &Path,
    a: &Value,
    args: &Value,
    h: &mut dyn Host,
    rounds: &mut Vec<Value>,
) -> Result<i64> {
    let raw = s(args, "arch");
    let arch = raw.split('-').next().unwrap_or("");
    if !matches!(arch, "i386" | "amd64" | "arm64") {
        return Err(Error::new(format!("unsupported conformance arch {raw}")));
    }
    let pa = pytest_arch(raw);
    let smp = n(args, "smp", 0);
    if smp < 0 {
        return Err(Error::new("negative SMP count"));
    }
    let write = flag(args, "write_baseline");
    if smp == 1 && write {
        emit(h,"[conformance] --smp 1 --write-baseline refused: the UP baseline must be produced by the UP kernel.",true)?;
        return Ok(2);
    }
    let bp = baseline(root, a, args);
    let only = args
        .get("only")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if !only.is_empty() && (!write || !bp.is_file()) {
        emit(
            h,
            format!(
                "[conformance] --only needs --write-baseline and an existing baseline at {}",
                bp.display()
            ),
            true,
        )?;
        return Ok(2);
    }
    let run = PathBuf::from(
        h.call(
            "create_run_dir",
            json!({"command":"conformance","arch":arch}),
        )?
        .as_str()
        .ok_or("create_run_dir returned non-path")?,
    );
    let mut e = env(a)?;
    if flag(args, "no_build") {
        if let Some(gap) = media_payload_gap(&root.join(format!("build-{arch}"))) {
            emit(h, format!("[conformance] --no-build refused: {gap}"), true)?;
            return Ok(2);
        }
    } else {
        let gaps = provisioning_gaps(root, arch);
        if !gaps.is_empty() {
            emit(h,format!("[conformance] this worktree is not provisioned to build a payload:\n  - {}\n  (all are gitignored, so `git worktree add` cannot bring them.)",gaps.join("\n  - ")),true)?;
            return Ok(2);
        }
        let mut be = e.clone();
        be.insert("VIBEOS_PAYLOAD".into(), "kmtest".into());
        emit(
            h,
            format!("[conformance] building {arch} with VIBEOS_PAYLOAD=kmtest ..."),
            false,
        )?;
        let r = command(
            h,
            root,
            vec![
                root.join("vibeos-cli").to_string_lossy().into_owned(),
                "build".into(),
                "--arch".into(),
                arch.into(),
                "--kmtests".into(),
            ],
            &be,
            None,
        )?;
        let rc = n(&r, "returncode", 1);
        if rc != 0 {
            emit(h, "[conformance] BUILD FAILED — aborting.", true)?;
            return Ok(rc);
        }
    }
    let fresh = freshness(root, a, arch, &e)?;
    if !flag(&fresh, "fresh") || flag(&fresh, "media_missing") {
        emit(
            h,
            format!("[conformance] stale media refused: {fresh}"),
            true,
        )?;
        return Ok(2);
    }
    if smp > 0 {
        e.insert("VIBEOS_QEMU_SMP".into(), smp.to_string());
    }
    e.insert(
        "VIBEOS_ARTIFACT_DIR".into(),
        run.to_string_lossy().into_owned(),
    );
    e.insert("VIBEOS_SERIAL_TIMESTAMPS".into(), "1".into());
    if matches!(arch, "i386" | "amd64") {
        e.entry("VIBEOS_WRITABLE_ROOT".into()).or_insert("1".into());
    }
    let quiet = parse(
        root,
        "quiet_token",
        json!({"tripwire":e.get("VIBEOS_TRIPWIRE").cloned().unwrap_or_default()}),
    )?;
    e.insert(
        "VIBEOS_TRIPWIRE".into(),
        quiet.as_str().ok_or("invalid quiet token")?.into(),
    );
    let timeout = n(args, "timeout", 1800);
    if timeout <= 0 || timeout > i64::MAX - 300 {
        return Err(Error::new("invalid conformance timeout"));
    }
    let python = a.get("python").and_then(Value::as_str).unwrap_or("python3");
    let argv = vec![
        python.into(),
        "-m".into(),
        "pytest".into(),
        root.join("tests/qemu/test_payload.py")
            .to_string_lossy()
            .into_owned(),
        "--arch".into(),
        pa.into(),
        "--payload".into(),
        "kmtest".into(),
        "--skip-clean-build".into(),
        "--timeout".into(),
        timeout.to_string(),
        "-q".into(),
        "-p".into(),
        "no:cacheprovider".into(),
        "-k".into(),
        "launched".into(),
    ];
    emit(
        h,
        format!("[conformance] kmtest sweep on {pa} — draining up to {timeout}s ..."),
        false,
    )?;
    let start = Instant::now();
    let response = command(h, root, argv, &e, Some(timeout + 300))?;
    let elapsed = start.elapsed().as_secs_f64();
    let sweep_rc = if flag(&response, "timed_out") {
        emit(
            h,
            "[conformance] sweep subprocess exceeded timeout — parsing captured serial.",
            true,
        )?;
        Value::Null
    } else {
        response.get("returncode").cloned().unwrap_or(Value::Null)
    };
    let log = run.join(format!("{pa}_serial.log"));
    if !log.is_file() {
        emit(
            h,
            format!(
                "[conformance] no serial log at {} — did the guest boot?",
                log.display()
            ),
            true,
        )?;
        return Ok(2);
    }
    let serial = parse(root, "read_serial", json!({"path":log}))?
        .as_str()
        .ok_or("invalid serial response")?
        .to_owned();
    let parsed = parse(root, "kmtest", json!({"text":serial}))?;
    let results = parsed[0]
        .as_array()
        .ok_or("invalid parsed results")?
        .clone();
    let done = parsed[1].clone();
    let bug = parsed[2].clone();
    let badset = parse(root, "bad", json!({}))?;
    let bads = badset.as_array().ok_or("invalid BAD set")?;
    let ok = results.iter().filter(|r| r["status"] == "OK").count();
    let bad = results
        .iter()
        .filter(|r| bads.contains(&r["status"]))
        .count();
    let asserts = crate::verdict::kernel_asserts(&serial);
    let mut lines = Vec::new();
    for r in &results {
        lines.push(
            parse(root, "format", json!({"result":r}))?
                .as_str()
                .ok_or("invalid formatted row")?
                .to_owned(),
        )
    }
    let rp = run.join("kmtest-results.txt");
    fs::write(&rp, format!("{}\n", lines.join("\n")))?;
    let state = run.join("hang-cpustate.txt");
    let mut pcs = Vec::new();
    if state.is_file() {
        pcs = crate::verdict::symbolicate_cpu_state(
            root,
            arch,
            &txt(&state)?,
            serial.contains("ntkrnlmp.exe"),
            &serial,
        );
        if !pcs.is_empty() {
            writeln!(
                OpenOptions::new().append(true).open(&state)?,
                "\n=== symbolicated ===\n{}",
                pcs.join("\n")
            )?;
        }
        emit(
            h,
            format!(
                "[conformance] guest was WEDGED — vCPU state: {}",
                state.display()
            ),
            false,
        )?;
    }
    if smp > 0 {
        let k =
            regex::Regex::new(r"(?i)\[FLDR\] load \S*[\\/](ntkrnlmp\.exe|ntoskrnl\.exe)").unwrap();
        let hal = regex::Regex::new(r"(?i)\[FLDR\] load \S*[\\/](halmp\.dll|hal\.dll)").unwrap();
        let kernel = k.captures(&serial).map(|x| x[1].to_lowercase());
        let hal = hal.captures(&serial).map(|x| x[1].to_lowercase());
        if kernel.as_deref().is_some_and(|x| x != "ntkrnlmp.exe")
            || hal.as_deref().is_some_and(|x| x != "halmp.dll")
        {
            emit(h,format!("[conformance] --smp {smp} did NOT get the MP pair: {kernel:?} + {hal:?}. Serial: {}",log.display()),true)?;
            return Ok(2);
        }
        emit(
            h,
            format!("[conformance] MP pair confirmed on serial: {kernel:?} + {hal:?}"),
            false,
        )?;
    }
    let have = bp.is_file();
    let base = if have {
        parse(root, "load_baseline", json!({"path":bp}))?
    } else {
        json!({})
    };
    let regress = if have {
        parse(root, "compare", json!({"results":results,"base":base}))?
    } else {
        json!([])
    };
    let newpasses: Vec<Value> = results
        .iter()
        .filter(|r| {
            r["status"] == "OK"
                && (base.get(s(r, "name")).is_none()
                    || bads.contains(&base[s(r, "name")]["status"]))
        })
        .map(|r| r["name"].clone())
        .collect();
    if write {
        let previous = if have { txt(&bp)? } else { String::new() };
        let selected: Vec<Value> = results
            .iter()
            .filter(|r| only.is_empty() || only.contains(&r["name"]))
            .cloned()
            .collect();
        let note = parse(
            root,
            "lowering_note",
            json!({"results":selected,"base":base}),
        )?;
        if let Some(note) = note.as_str() {
            emit(h, note, false)?;
        }
        let text = if !only.is_empty() {
            let m = parse(
                root,
                "merge_rows",
                json!({"text":previous,"results":results,"only":only}),
            )?;
            if let Some(r) = m["refusal"].as_str() {
                emit(h, r, true)?;
                return Ok(2);
            }
            s(&m, "text").to_owned()
        } else {
            if have {
                let refusal = parse(
                    root,
                    "coverage_refusal",
                    json!({"results":results,"base":base,"force":flag(args,"force_baseline")}),
                )?;
                if let Some(r) = refusal.as_str() {
                    emit(h, r, true)?;
                    return Ok(2);
                }
            }
            let mut blines = Vec::new();
            for r in &results {
                let flaky = parse(root, "flaky_of", json!({"text":previous,"name":r["name"]}))?;
                blines.push(
                    parse(root, "format", json!({"result":r,"flaky":flaky}))?
                        .as_str()
                        .ok_or("invalid baseline row")?
                        .to_owned(),
                )
            }
            format!("# kmtest conformance baseline — parse_kmtest_serial.py format\n# arch={arch}{}  generated by `vibeos-cli conformance --arch {arch}{} --write-baseline`\n{}\n",if smp>1{format!(" smp={smp}")}else{String::new()},if smp>1{format!(" --smp {smp}")}else{String::new()},blines.join("\n"))
        };
        fs::create_dir_all(bp.parent().unwrap())?;
        fs::write(&bp, text)?;
        emit(
            h,
            format!("[conformance] baseline written: {}", bp.display()),
            false,
        )?;
    }
    let summary = json!({"suite":"kmtest","arch":arch,"pytest_arch":pa,"smp":smp,"mp_kernel":serial.contains("ntkrnlmp.exe"),"writable_root":serial.contains("ArcBootDeviceName=ramdisk(0)"),"launched":results.len(),"ok":ok,"bad":bad,"sweep_done":done,"bugcheck":bug,"kernel_asserts":asserts,"sweep_rc":sweep_rc,"elapsed_s":(elapsed*10.).round()/10.,"have_baseline":have,"regressions":regress,"new_passes":newpasses,"results_file":rp,"serial_log":log,"serial_log_live":log,"hang_cpustate":if state.is_file(){json!(state)}else{Value::Null},"hang_pcs":pcs});
    fs::write(
        run.join("summary.json"),
        serde_json::to_vec_pretty(&summary)?,
    )?;
    let shape=parse(root,"round_shape",json!({"bugcheck":bug,"results":results,"done_line":done,"regressions":regress,"serial":serial}))?.as_str().ok_or("invalid round shape")?.to_owned();
    let (sha, dirty) = crate::farm::git_state(root, &e)?;
    let ev = crate::verdict::wedge_evidence(&serial);
    let mut rec = summary.clone();
    for (k,v) in json!({"ts":crate::farm::now_tag(),"shape":shape,"sha":sha,"tripwire":env(a)?.get("VIBEOS_TRIPWIRE").cloned().unwrap_or_default(),"run_dir":run,"census_stuck":ev["census_stuck"],"lockdep_hits":ev["lockdep"],"cswitch_hits":ev["cswitch"],"census_lines":ev["lines"],"wait_profile":serial.contains("[CENSUS] wait profile:"),"rust_panics":serial.matches("*** RUST PANIC").count()}).as_object().unwrap(){rec[k]=v.clone();}
    let fd = farm(a, &e);
    if shape != "clean" {
        if let Some(kept) = preserve(root, &fd, &run, arch, &shape, &e)? {
            rec["preserved"] = json!(kept);
        }
    }
    crate::farm::append_record(&ledger(a, &e), &rec)?;
    rounds.push(rec.clone());
    let sigs = h.call("conformance_signatures", json!({"record":rec}))?;
    h.call("ledger_append",json!({"repo":root,"cmd":"conformance","arch":arch,"verdict":if shape=="clean"{"OK"}else{"FAIL"},"classification":shape,"duration_ms":(elapsed*1000.) as u64,"extra":{"suite":"kmtest","smp":smp,"mp_kernel":summary["mp_kernel"],"launched":results.len(),"ok":ok,"bad":bad,"bugcheck":bug,"have_baseline":have,"regressions":regress,"new_passes":newpasses,"sigs":sigs,"dirty":dirty,"run_id":run.file_name().unwrap_or_default().to_string_lossy(),"preserved":rec["preserved"]}}))?;
    emit(h,format!("\nconformance kmtest {arch} — launched={} ok={ok} bad={bad} sweep={} ({elapsed:.0}s)\n  results: {}\n  summary: {}",results.len(),if done.is_null(){"DID NOT FINISH"}else{"done"},rp.display(),run.join("summary.json").display()),false)?;
    if !have {
        emit(
            h,
            format!(
                "  no baseline at {} — pass --write-baseline to seed it.",
                bp.display()
            ),
            false,
        )?;
        return Ok(0);
    }
    if !newpasses.is_empty() {
        emit(
            h,
            format!("  NEW PASSES: {}", serde_json::to_string(&newpasses)?),
            false,
        )?;
    }
    if !asserts.is_empty() {
        emit(
            h,
            format!("  KERNEL ASSERTIONS FAILED: {}", asserts.join("\n    ")),
            false,
        )?;
        if flag(args, "gate") {
            return Ok(1);
        }
    }
    if !regress.as_array().is_none_or(Vec::is_empty) {
        emit(h, format!("  RATCHET REGRESSIONS: {regress}"), false)?;
        return Ok(i64::from(flag(args, "gate")));
    }
    emit(h, "  ratchet holds: no regressions vs baseline.", false)?;
    Ok(0)
}
fn seed(root: &Path, a: &Value, args: &Value, rounds: &[Value], h: &mut dyn Host) -> Result<i64> {
    let bp = baseline(root, a, args);
    if !bp.is_file() {
        emit(h,format!("[conformance] --seed-asserts needs an existing baseline at {}; seed the counts first with --write-baseline",bp.display()),true)?;
        return Ok(2);
    }
    let mut rows = Vec::new();
    for r in rounds {
        let run = Path::new(s(r, "run_dir"));
        if !run.is_dir() {
            continue;
        }
        let logs: Vec<_> = serials(run)?
            .into_iter()
            .filter(|p| {
                p.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .ends_with("_serial.log")
            })
            .collect();
        if let Some(log) = logs.first() {
            let serial = parse(root, "read_serial", json!({"path":log}))?;
            let p = parse(root, "kmtest", json!({"text":serial}))?;
            if p[0].as_array().is_some_and(|r| !r.is_empty()) {
                rows.push(p[0].clone());
            }
        }
    }
    if rows.len() < 3 {
        emit(h,format!("[conformance] --seed-asserts: only {} round(s) left an archived serial log with results; need 3. Nothing written.",rows.len()),true)?;
        return Ok(2);
    }
    let seeded = parse(
        root,
        "seed_asserts",
        json!({"text":txt(&bp)?,"rounds":rows}),
    )?;
    fs::write(&bp, seeded[0].as_str().ok_or("invalid seeded text")?)?;
    emit(h, seeded[1].as_str().unwrap_or(""), false)?;
    emit(
        h,
        format!("[conformance] identity floor written: {}", bp.display()),
        false,
    )?;
    Ok(0)
}
pub fn run(root: &Path, a: &Value, h: &mut dyn Host) -> Result<Value> {
    if a.get("suite")
        .and_then(Value::as_str)
        .is_some_and(|suite| suite != "kmtest")
    {
        return Err(Error::new("conformance only supports the kmtest suite"));
    }
    let mut args = a.get("args").cloned().unwrap_or_else(|| a.clone());
    let repeat = n(&args, "repeat", 0);
    let seed_asserts = flag(&args, "seed_asserts");
    let mut rounds: Vec<Value> = a
        .get("rounds")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if seed_asserts && repeat < 3 {
        emit(h,"[conformance] --seed-asserts needs --repeat 3 or more: fewer rounds cannot distinguish a bistable assertion from a stable one.",true)?;
        return Ok(json!({"code":2,"rounds":rounds}));
    }
    if repeat <= 1 {
        let code = once(root, a, &args, h, &mut rounds)?;
        return Ok(json!({"code":code,"rounds":rounds}));
    }
    let mut worst = 0;
    let mut no_launch = 0;
    for i in 0..repeat {
        if i > 0 {
            args["no_build"] = json!(true)
        }
        emit(
            h,
            format!(
                "\n=== conformance round {}/{} ({}) ===",
                i + 1,
                repeat,
                s(&args, "arch")
            ),
            false,
        )?;
        let built = !flag(&args, "no_build");
        let before = rounds.len();
        let rc = match once(root, a, &args, h, &mut rounds) {
            Ok(rc) => rc,
            Err(e) => {
                emit(
                    h,
                    format!("[conformance] round {} raised {e} — continuing", i + 1),
                    true,
                )?;
                worst = worst.max(1);
                if built && rounds.len() == before {
                    emit(h,"[conformance] ABORTING the soak: the build round failed; further rounds would boot stale media.",true)?;
                    break;
                }
                continue;
            }
        };
        worst = worst.max(rc);
        if built && rc != 0 && rounds.len() == before {
            emit(h,"[conformance] ABORTING the soak: the build round failed; further rounds would boot stale media.",true)?;
            break;
        }
        if rounds.last().is_some_and(|r| r["launched"] == 0) {
            no_launch += 1;
            if no_launch >= 3 {
                emit(h,"[conformance] ABORTING the soak: 3 consecutive rounds launched ZERO tests. Rebuild and provision this worktree before continuing.",true)?;
                worst = worst.max(1);
                break;
            }
        } else {
            no_launch = 0
        }
    }
    let mut counts = BTreeMap::new();
    for r in &rounds {
        *counts.entry(s(r, "shape").to_owned()).or_insert(0usize) += 1;
    }
    let mut counts: Vec<_> = counts.into_iter().collect();
    counts.sort_by(|a, b| b.1.cmp(&a.1));
    emit(
        h,
        format!(
            "\n=== soak roll-up: {} round(s) -> {} ===",
            rounds.len(),
            ledger(a, &env(a)?).display()
        ),
        false,
    )?;
    for (shape, count) in counts {
        emit(h, format!("  {count:>3}  {shape}"), false)?
    }
    if seed_asserts {
        worst = worst.max(seed(root, a, &args, &rounds, h)?)
    }
    Ok(json!({"code":worst,"rounds":rounds}))
}
pub fn dispatch(root: &Path, request: &Value, h: &mut dyn Host) -> Result<Value> {
    let a = request.get("kwargs").unwrap_or(request);
    match s(request, "op") {
        "conformance_run" => run(root, a, h),
        "conformance_seed_rounds" => Ok(json!(seed(
            root,
            a,
            a.get("args").unwrap_or(a),
            a.get("rounds")
                .and_then(Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or(&[]),
            h
        )?)),
        "conformance_append_round" => {
            crate::farm::append_record(
                &ledger(a, &env(a)?),
                a.get("record").ok_or("missing conformance record")?,
            )?;
            Ok(Value::Null)
        }
        "conformance_media_payload_gap" => {
            Ok(json!(media_payload_gap(Path::new(s(a, "build_dir")))))
        }
        "conformance_provisioning_gaps" => Ok(json!(provisioning_gaps(root, s(a, "arch")))),
        "conformance_cache_bools" => Ok(json!(cache_bools(Path::new(s(a, "path"))))),
        "conformance_check_freshness" => freshness(root, a, s(a, "arch"), &env(a)?),
        _ => Err(Error::new("unknown conformance orchestration operation")),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_and_non_payload_media_refused() {
        let d = tempfile::tempdir().unwrap();
        assert!(media_payload_gap(d.path()).unwrap().contains("missing"));
        fs::write(
            d.path().join("payload-selection.json"),
            r#"{"payloads":[]}"#,
        )
        .unwrap();
        assert!(media_payload_gap(d.path())
            .unwrap()
            .contains("not 'kmtest'"));
        fs::write(
            d.path().join("payload-selection.json"),
            r#"{"payloads":["kmtest"]}"#,
        )
        .unwrap();
        fs::write(
            d.path().join("CMakeCache.txt"),
            "ENABLE_ROSTESTS:BOOL=OFF\nENABLE_ROSTESTS_KMTESTS:BOOL=OFF\n",
        )
        .unwrap();
        assert!(media_payload_gap(d.path())
            .unwrap()
            .contains("without the kmtest corpus"));
    }
    #[test]
    fn missing_freshness_evidence_fails_loud() {
        let d = tempfile::tempdir().unwrap();
        let a = json!({"cfgs":{"arm64":{"boot_media_rel":"livecd.iso","staged_bins_rel":[],"needs_mkramdisk":false}}});
        let v = freshness(d.path(), &a, "arm64", &BTreeMap::new()).unwrap();
        assert!(!flag(&v, "fresh"));
    }
    #[test]
    fn old_stamp_without_scope_hashes_cannot_bank_freshness() {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("livecd.iso"), b"old media").unwrap();
        let build = d.path().join("build-arm64");
        fs::create_dir(&build).unwrap();
        fs::write(
            build.join(".vibeos-build-stamp.json"),
            serde_json::to_vec(&json!({
                "arch":"arm64", "ts":1, "git_fingerprint":"old", "rust_src_hash":"old",
                "boot_media":"livecd.iso", "boot_media_sha":"old", "cold":true
            }))
            .unwrap(),
        )
        .unwrap();
        let a = json!({"cfgs":{"arm64":{"boot_media_rel":"livecd.iso","staged_bins_rel":[],"needs_mkramdisk":false}}});
        let v = freshness(d.path(), &a, "arm64", &BTreeMap::new()).unwrap();
        assert!(!flag(&v, "fresh"));
        assert!(!flag(&v, "media_missing"));
        assert!(!v["reasons"].as_array().unwrap().is_empty());
    }
    #[test]
    fn controller_refusal_returns_rounds_dto_without_booting() {
        struct Printing;
        impl Host for Printing {
            fn call(&mut self, op: &str, _args: Value) -> Result<Value> {
                assert_eq!(op, "print", "a preflight refusal must not launch commands");
                Ok(Value::Null)
            }
        }
        let old = json!({"shape":"clean","launched":1});
        let result = run(
            Path::new("/unused"),
            &json!({"args":{"seed_asserts":true,"repeat":2},"rounds":[old]}),
            &mut Printing,
        )
        .unwrap();
        assert_eq!(result["code"], 2);
        assert_eq!(result["rounds"], json!([old]));
    }
}

// Extract the existing pytest arch match in once() into this helper, then set
// `let pa = pytest_arch(raw);`. Replaces Python _PYTEST_ARCH implementation tests.
fn pytest_arch(raw: &str) -> &str {
    match raw {
        "i386" => "i386-uefi",
        "amd64" => "amd64-uefi",
        _ => raw,
    }
}
#[cfg(test)]
mod arch_policy_controls {
    use super::*;
    #[test]
    fn pytest_arch_normalization_preserves_explicit_variants() {
        assert_eq!(pytest_arch("amd64"), "amd64-uefi");
        assert_eq!(pytest_arch("i386"), "i386-uefi");
        assert_eq!(pytest_arch("arm64"), "arm64");
        assert_eq!(pytest_arch("amd64-uefi"), "amd64-uefi");
        assert_eq!(pytest_arch("i386-bios"), "i386-bios");
    }
}

// Append to conformance_run.rs. Fixtures manufacture only external authority
// inputs (build rc / raw serial); source and media freshness are always native.
#[cfg(test)]
mod controller_fixture_tests {
    use super::*;
    use std::process::Command;
    struct Fixture {
        _tmp: tempfile::TempDir,
        repo: PathBuf,
        options: Value,
        baseline: PathBuf,
        host: Authority,
    }
    struct Authority {
        outputs: PathBuf,
        serials: Vec<String>,
        sweeps: usize,
        build_rc: i64,
        commands: Vec<Value>,
        ledgers: Vec<Value>,
        printed: Vec<String>,
    }
    impl Host for Authority {
        fn call(&mut self, op: &str, a: Value) -> Result<Value> {
            match op {
                "print" => {
                    self.printed.push(s(&a, "text").into());
                    Ok(Value::Null)
                }
                "create_run_dir" => {
                    let p = self.outputs.join(format!("round-{}", self.commands.len()));
                    fs::create_dir_all(&p)?;
                    Ok(json!(p))
                }
                "command" => {
                    self.commands.push(a.clone());
                    let argv = a["argv"].as_array().ok_or("missing argv")?;
                    if argv.iter().any(|x| x == "build") {
                        return Ok(json!({"returncode":self.build_rc,"timed_out":false}));
                    }
                    assert!(argv.iter().any(|x| x == "pytest"));
                    assert!(argv.iter().any(|x| x == "--skip-clean-build"));
                    let env = &a["env"];
                    let run = Path::new(s(env, "VIBEOS_ARTIFACT_DIR"));
                    assert!(!run.as_os_str().is_empty());
                    assert_eq!(env["VIBEOS_SERIAL_TIMESTAMPS"], "1");
                    assert!(s(env, "VIBEOS_TRIPWIRE").contains("QUIET"));
                    let index = self.sweeps.min(self.serials.len() - 1);
                    self.sweeps += 1;
                    // The actual pytest arch is authoritative for the log name.
                    let idx = argv.iter().position(|x| x == "--arch").unwrap();
                    let arch = argv[idx + 1].as_str().unwrap();
                    fs::write(run.join(format!("{arch}_serial.log")), &self.serials[index])?;
                    Ok(json!({"returncode":0,"timed_out":false}))
                }
                "conformance_signatures" => Ok(json!([])), // existing reporting parser boundary
                "ledger_append" => {
                    self.ledgers.push(a);
                    Ok(Value::Null)
                }
                _ => Err(Error::new(format!("unexpected fixture callback {op}"))),
            }
        }
    }
    fn git(root: &Path, args: &[&str]) {
        let r = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            r.status.success(),
            "git {:?}: {}",
            args,
            String::from_utf8_lossy(&r.stderr)
        );
    }
    impl Fixture {
        fn new(baseline: Option<&str>, serials: Vec<String>) -> Self {
            let tmp = tempfile::tempdir().unwrap();
            let repo = tmp.path().join("repo");
            let build = repo.join("build-arm64");
            fs::create_dir_all(&build).unwrap();
            fs::create_dir_all(repo.join(".build-tools/llvm-mingw")).unwrap();
            fs::create_dir_all(repo.join("tools/payloads/kmtest")).unwrap();
            fs::write(
                repo.join("tools/payloads/kmtest/kmtest-sweep-arm64.exe"),
                b"fixture",
            )
            .unwrap();
            fs::create_dir_all(repo.join("qemu")).unwrap();
            for file in ["QEMU_EFI.fd", "efivars.fd"] {
                fs::write(repo.join("qemu").join(file), b"opaque fixture").unwrap();
            }
            fs::write(repo.join("source.c"), "int fixture;\n").unwrap();
            fs::write(repo.join(".gitignore"), "build-*/\n.build-tools/\n").unwrap();
            fs::write(build.join("livecd.iso"), b"fixture media").unwrap();
            fs::write(
                build.join("payload-selection.json"),
                r#"{"payloads":["kmtest"]}"#,
            )
            .unwrap();
            let bp = repo.join("tools/conformance-baseline/arm64/kmtest.txt");
            if let Some(text) = baseline {
                fs::create_dir_all(bp.parent().unwrap()).unwrap();
                fs::write(&bp, text).unwrap();
            }
            git(&repo, &["init", "-q"]);
            git(&repo, &["config", "user.email", "test@example.invalid"]);
            git(&repo, &["config", "user.name", "native fixture"]);
            git(&repo, &["add", "-A"]);
            git(&repo, &["commit", "-qm", "fixture"]);
            let mut env: BTreeMap<String, String> = std::env::vars().collect();
            env.insert(
                "VIBEOS_ARTIFACT_STORE_ROOT".into(),
                tmp.path().join("store").to_string_lossy().into_owned(),
            );
            let cfg = json!({"boot_media_rel":"build-arm64/livecd.iso","staged_bins_rel":[],"needs_mkramdisk":false});
            let stamp=vibeos_build::provenance::dispatch(&repo,&json!({"op":"provenance_make_stamp","kwargs":{"repo":repo,"arch":"arm64","cfg":cfg,"cold":true,"ts":1,"env":env}})).unwrap();
            vibeos_build::provenance::dispatch(
                &repo,
                &json!({"op":"provenance_stamp_write","kwargs":{"build_dir":build,"stamp":stamp}}),
            )
            .unwrap();
            let options = json!({"args":{"arch":"arm64","no_build":true,"repeat":1,"timeout":1,"write_baseline":false,"smp":0,"gate":true},"env":env,"cfgs":{"arm64":cfg},"farm_dir":tmp.path().join("farm")});
            let outputs = tmp.path().join("outputs");
            fs::create_dir(&outputs).unwrap();
            Self {
                _tmp: tmp,
                repo,
                options,
                baseline: bp,
                host: Authority {
                    outputs,
                    serials,
                    sweeps: 0,
                    build_rc: 0,
                    commands: Vec::new(),
                    ledgers: Vec::new(),
                    printed: Vec::new(),
                },
            }
        }
        fn run(&mut self) -> Value {
            run(&self.repo, &self.options, &mut self.host).unwrap()
        }
    }
    fn serial(name: &str, pass: u64, fail: u64) -> String {
        format!("[KMTEST-SWEEP] launch {name}\n[KMTEST:END {name} pass={pass} fail={fail} skip=0]\n[KMTEST-SWEEP] exit {name} code=0\n[KMTEST-SWEEP] done total=1\n")
    }
    #[test]
    fn build_asks_for_corpus_and_failed_first_build_never_sweeps() {
        let mut f = Fixture::new(None, vec![serial("A", 10, 0)]);
        f.options["args"]["no_build"] = json!(false);
        f.options["args"]["repeat"] = json!(3);
        f.host.build_rc = 1;
        let r = f.run();
        assert_eq!(r["code"], 1);
        assert_eq!(r["rounds"], json!([]));
        assert_eq!(f.host.sweeps, 0);
        assert_eq!(f.host.commands.len(), 1);
        let args = f.host.commands[0]["argv"].as_array().unwrap();
        assert!(args.contains(&json!("--kmtests")));
        assert!(!args.contains(&json!("--up-kernel")));
        assert_eq!(f.host.commands[0]["env"]["VIBEOS_PAYLOAD"], "kmtest");
        assert!(f.host.ledgers.is_empty());
    }
    #[test]
    fn only_without_write_refuses_before_any_build_or_sweep() {
        let mut f = Fixture::new(None, vec![serial("A", 10, 0)]);
        f.options["args"]["only"] = json!(["A"]);
        f.options["args"]["no_build"] = json!(false);
        assert_eq!(f.run()["code"], 2);
        assert!(f.host.commands.is_empty());
        assert!(f.host.ledgers.is_empty());
    }
    #[test]
    fn three_consecutive_empty_rounds_abort_but_isolated_ones_do_not() {
        let mut empty = Fixture::new(None, vec!["guest progress\n".into()]);
        empty.options["args"]["repeat"] = json!(20);
        let r = empty.run();
        assert_eq!(r["code"], 1);
        assert_eq!(r["rounds"].as_array().unwrap().len(), 3);
        assert_eq!(empty.host.sweeps, 3);
        assert_eq!(empty.host.ledgers.len(), 3);
        let mut isolated = Fixture::new(
            None,
            vec![
                "guest progress\n".into(),
                serial("A", 10, 0),
                "guest progress\n".into(),
                serial("A", 10, 0),
            ],
        );
        isolated.options["args"]["repeat"] = json!(4);
        let r = isolated.run();
        assert_eq!(r["rounds"].as_array().unwrap().len(), 4);
        assert_eq!(isolated.host.sweeps, 4);
    }
    #[test]
    fn no_baseline_remains_report_only_but_durable_ledger_records_failure() {
        let mut f = Fixture::new(None, vec!["*** BUGCHECK: 0x0000001E\n".into()]);
        let r = f.run();
        assert_eq!(r["code"], 0);
        assert_eq!(f.host.ledgers.len(), 1);
        assert_eq!(f.host.ledgers[0]["verdict"], "FAIL");
        assert_eq!(f.host.ledgers[0]["extra"]["have_baseline"], false);
        assert_eq!(r["rounds"].as_array().unwrap().len(), 1);
    }
    #[test]
    fn wholesale_reseed_refuses_coverage_loss_unless_forced() {
        let floor = "A OK pass=10 fail=0 skip=0\nB OK pass=10 fail=0 skip=0\n";
        let mut f = Fixture::new(Some(floor), vec![serial("A", 10, 0)]);
        f.options["args"]["write_baseline"] = json!(true);
        assert_eq!(f.run()["code"], 2);
        assert_eq!(txt(&f.baseline).unwrap(), floor);
        let mut forced = Fixture::new(Some(floor), vec![serial("A", 10, 0)]);
        forced.options["args"]["write_baseline"] = json!(true);
        forced.options["args"]["force_baseline"] = json!(true);
        forced.run();
        let text = txt(&forced.baseline).unwrap();
        assert!(text.contains("A OK"));
        assert!(!text.contains("B OK"));
    }
    #[test]
    fn lowered_floor_is_loud_and_quarantined_identity_survives_reseed() {
        let floor = "A OK pass=10 fail=0 skip=0\n    assert? Example.c:12\n";
        let mut f = Fixture::new(Some(floor), vec![serial("A", 9, 1)]);
        f.options["args"]["write_baseline"] = json!(true);
        f.run();
        assert!(f.host.printed.join("\n").contains("pass 10 -> 9"));
        assert!(txt(&f.baseline).unwrap().contains("assert? Example.c:12"));
    }
    #[test]
    fn stale_source_media_never_reaches_pytest_or_ledger() {
        let mut f = Fixture::new(None, vec![serial("A", 10, 0)]);
        fs::write(f.repo.join("source.c"), "int changed;\n").unwrap();
        assert_eq!(f.run()["code"], 2);
        assert_eq!(f.host.sweeps, 0);
        assert!(f.host.ledgers.is_empty());
    }
    // Insert inside conformance_run::controller_fixture_tests (uses its real Fixture).
    #[test]
    fn ledger_records_the_kernel_observed_in_serial_instead_of_the_smp_flag() {
        // The shipped MP kernel is often booted with no explicit --smp at all.
        let mut mp = Fixture::new(
            None,
            vec![format!(
        "[FLDR] load /reactos/system32/ntkrnlmp.exe\n[FLDR] load /reactos/system32/halmp.dll\n{}",
        serial("A",10,0)
    )],
        );
        mp.options["args"]["smp"] = json!(0);
        let r = mp.run();
        assert_eq!(r["rounds"][0]["smp"], 0);
        assert_eq!(r["rounds"][0]["mp_kernel"], true);
        assert_eq!(mp.host.ledgers[0]["extra"]["mp_kernel"], true);

        // Preserve the existing missing-witness behavior: the loader check refuses
        // an OBSERVED wrong pair; an absent loader witness is not guessed from flags.
        let mut unwitnessed = Fixture::new(None, vec![serial("A", 10, 0)]);
        unwitnessed.options["args"]["smp"] = json!(1);
        let r = unwitnessed.run();
        assert_eq!(r["rounds"][0]["smp"], 1);
        assert_eq!(r["rounds"][0]["mp_kernel"], false);
        assert_eq!(unwitnessed.host.ledgers[0]["extra"]["mp_kernel"], false);
    }
}
