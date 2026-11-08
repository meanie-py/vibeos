//! `vibeos-cli winref`: a real Windows 11 VM as the conformance ORACLE, and the reading of
//! what it said.
//!
//! The suites encode what someone believed Server 2003 did; for the Windows 10/11 surface the
//! expected result comes from Windows itself. This module owns every decision winref makes:
//! the QEMU command line, the reading of the guest's COM1 protocol
//!
//! ```text
//! WINREF-SETUP: ...        install-time steps, ending `done build=...`
//! WINREF: ready build=...  a run booted
//! OUT: <line>              the payload's output, one line per line
//! WINREF: exit=<n>         the payload's exit code
//! ```
//!
//! the reference column graded against the guest floor (with the native conformance parsers),
//! and the disk arithmetic `logs` needs to read NTFS out of a qcow2 without booting it.
//!
//! The VM itself (the console socket, QMP, the power-off watch, qemu-img and 7-Zip) stays with
//! the facade (`tests/qemu/cli/winref.py`) until the CLI entry is native.
use crate::conformance::{self, TestResult};
use crate::verdict::py_lines;
use crate::{Error, Result};
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;
use std::path::Path;

pub const CANON: &str = "/tmp/vibeos/";
pub const WIN_BASELINE: &str = "win11-amd64";
pub const MAINT_DONE: &str = "WINREF-MAINT: done";

fn usage(message: String) -> Error {
    Error {
        kind: Some("value".into()),
        message,
    }
}

/// The QEMU command line. The VM has no network device (no updates, no drift between runs)
/// and no USB controller: every `WHPX: Unexpected VP exit code 4` came right after `Ignoring
/// request for interrupt vector 0`, an MSI fired before its vector was programmed, and the
/// tablet's controller was the only optional MSI source. QMP and COM1 dial back to listeners
/// on the WSL side (vm.rs's windows-host wiring). Paths arrive as Windows paths.
pub fn launch_argv(a: &Value) -> Result<Vec<String>> {
    let s = |k: &str| -> Result<String> {
        a[k].as_str()
            .map(str::to_owned)
            .ok_or_else(|| format!("winref launch: missing {k}").into())
    };
    let n = |k: &str| -> Result<i64> {
        a[k].as_i64()
            .ok_or_else(|| format!("winref launch: missing {k}").into())
    };
    let accel = if s("accel")? == "whpx" {
        format!("whpx,kernel-irqchip={}", s("irqchip")?)
    } else {
        "tcg,thread=multi".into()
    };
    let mut argv: Vec<String> = vec![
        s("qemu")?,
        "-name".into(),
        s("uid")?,
        "-machine".into(),
        "q35".into(),
        "-accel".into(),
        accel,
        "-cpu".into(),
        s("cpu")?,
        "-smp".into(),
        n("cpus")?.to_string(),
        "-m".into(),
        s("mem")?,
        "-drive".into(),
        format!("if=pflash,format=raw,readonly=on,file={}", s("code")?),
        "-drive".into(),
        format!("if=pflash,format=raw,file={}", s("vars")?),
        "-drive".into(),
        format!("file={},if=none,id=hd0,format=qcow2", s("disk")?),
        "-device".into(),
        "ide-hd,drive=hd0,bus=ide.0".into(),
        "-nic".into(),
        "none".into(),
        "-chardev".into(),
        format!(
            "socket,id=qmp0,host=127.0.0.1,port={},server=off",
            n("qmp_port")?
        ),
        "-mon".into(),
        "chardev=qmp0,mode=control".into(),
        "-chardev".into(),
        format!(
            "socket,id=ser0,host=127.0.0.1,port={},server=off",
            n("console_port")?
        ),
        "-serial".into(),
        "chardev:ser0".into(),
        "-monitor".into(),
        "none".into(),
        "-display".into(),
        if a["show"] == true { "sdl" } else { "none" }.into(),
    ];
    // A one-shot boot must END when the guest is done. Windows turned the runner's
    // shutdown into a restart, and the runner ran the payload again on the next boot: every
    // OUT: line twice (20261108-010042-BXkgvb), and the apitest sweep would rerun every
    // suite. Install (Setup reboots) and kmtest (its sweep resumes after a bugcheck reboot)
    // need their reboots and pass no_reboot=false.
    if a["no_reboot"] == true {
        argv.push("-no-reboot".into());
    }
    // q35's ICH9 AHCI has six ports, ide.0..ide.5; the disk holds ide.0.
    let cds = a["cds"].as_array().cloned().unwrap_or_default();
    if cds.len() > 5 {
        return Err(usage(format!(
            "winref launch: {} CDs, but q35 has five free AHCI ports",
            cds.len()
        )));
    }
    for (i, iso) in cds.iter().enumerate() {
        let i = i + 1;
        let iso = iso.as_str().ok_or("winref launch: a CD is not a path")?;
        argv.extend([
            "-drive".into(),
            format!("file={iso},if=none,id=cd{i},media=cdrom,readonly=on"),
            "-device".into(),
            format!("ide-cd,drive=cd{i},bus=ide.{i}"),
        ]);
    }
    Ok(argv)
}

/// Every file a livecd manifest puts in reactos/bin, as (name under bin, source): what
/// add_rostests_file staged. The guest's sweeps find their binaries there, and kmtest_.exe
/// loads its drivers from its own directory, so the Windows run gets exactly that set, laid
/// out the same way. Build paths in the manifest are canonical (tools/canon-run.sh).
pub fn staged_bin(manifest: &str, repo: &Path) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for line in py_lines(manifest) {
        let Some((dst, src)) = line.split_once('=') else {
            continue;
        };
        let Some(name) = dst.strip_prefix("reactos/bin/") else {
            continue;
        };
        let src = src.trim();
        let src = match src.strip_prefix(CANON) {
            Some(rest) => repo.join(rest).to_string_lossy().into_owned(),
            None => src.to_owned(),
        };
        match out.iter_mut().find(|(n, _)| n == name) {
            Some(slot) => slot.1 = src,
            None => out.push((name.to_owned(), src)),
        }
    }
    out
}

/// The tests the Windows column runs, in order: the guest sweep's own list first (so tests
/// meet the same predecessors as in the guest), then every other visible test. The rest are
/// mostly the guest's QUARANTINES, and they are the point: a test the guest cannot survive
/// says nothing on the guest, but what Windows does with it is exactly the expected result.
/// Hidden (`-Name`) tests stay out, as they do in the guest.
pub fn kmtest_order(listed: &[String], built: &BTreeSet<String>) -> Vec<String> {
    let seen: BTreeSet<&String> = listed.iter().collect();
    listed
        .iter()
        .cloned()
        .chain(
            built
                .iter()
                .filter(|t| !seen.contains(t) && !t.starts_with('-'))
                .cloned(),
        )
        .collect()
}

/// The harness could not run this row (a driver that will not load, a binary that will not
/// start): its result is about the rig, not about Windows.
pub fn not_run(r: &TestResult) -> bool {
    let st = r.status.as_deref();
    st == Some("FAILSTART") || (st == Some("NO-END") && r.errors > 0)
}

fn status(r: &TestResult) -> &str {
    r.status.as_deref().unwrap_or("None")
}

fn guest_num(g: &Value, k: &str) -> i64 {
    g.get(k).and_then(Value::as_i64).unwrap_or(0)
}

fn guest_ids(g: &Value) -> BTreeSet<String> {
    g.get("asserts")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

/// One line per row where Windows and the guest floor disagree, sorted by kind. It says what
/// was OBSERVED, not why: "Windows clean, guest fails" is where to look for a guest bug
/// first, and "fails on Windows 11 too" is a test whose expectation Windows does not meet,
/// one that is timing-sensitive in a VM (KeMutex fails 517 there), or one our toolchain
/// builds differently from the test's author.
pub fn column_report(win: &[TestResult], guest: &Map<String, Value>) -> Vec<String> {
    let mut rows: Vec<(i32, String)> = Vec::new();
    for r in win {
        let w_ok = r.status.as_deref() == Some("OK") && r.fail == 0;
        if not_run(r) {
            // Says nothing about the test: Windows never got to run it.
            rows.push((
                -1,
                format!(
                    "{}: NOT RUN on Windows ({}, {} harness error(s))",
                    r.name,
                    status(r),
                    r.errors
                ),
            ));
            continue;
        }
        let Some(g) = guest.get(&r.name) else {
            if !guest.is_empty() {
                rows.push((
                    if w_ok { 0 } else { 2 },
                    format!(
                        "{}: not in the guest floor; Windows {} pass={} fail={}",
                        r.name,
                        status(r),
                        r.pass,
                        r.fail
                    ),
                ));
            }
            continue;
        };
        let g_status = g.get("status").and_then(Value::as_str).unwrap_or("None");
        let g_fail = guest_num(g, "fail");
        let g_ok = g_status == "OK" && g_fail == 0;
        if w_ok && !g_ok {
            rows.push((
                1,
                format!("{}: Windows clean, guest {g_status} fail={g_fail}", r.name),
            ));
        } else if !w_ok {
            let shared = r.asserts.intersection(&guest_ids(g)).count();
            rows.push((
                3,
                format!(
                    "{}: fails on Windows 11 too: Windows {} fail={}, guest fail={g_fail}, \
                     {shared} failing assertion(s) shared",
                    r.name,
                    status(r),
                    r.fail
                ),
            ));
        }
    }
    rows.sort();
    rows.into_iter().map(|(_, line)| line).collect()
}

/// Exit codes of a process the loader killed before main(): the suite never ran.
fn load_failure(code: i64) -> Option<&'static str> {
    match code & 0xFFFF_FFFF {
        0xC000_0135 => Some("STATUS_DLL_NOT_FOUND"),
        0xC000_0139 => Some("STATUS_ENTRYPOINT_NOT_FOUND"),
        0xC000_0142 => Some("STATUS_DLL_INIT_FAILED"),
        _ => None,
    }
}

fn marker(suite: &str) -> Result<&'static str> {
    match suite {
        "kmtest" => Ok("[KMTEST-SWEEP]"),
        "apitest" => Ok("[APITEST-SWEEP]"),
        _ => Err(usage(format!("winref: unknown suite {suite:?}"))),
    }
}

fn stage(name: &str, status: &str, detail: String, salient: &[String]) -> Value {
    json!({"name": name, "status": status, "detail": detail, "salient": salient})
}

fn tail(v: &[String], n: usize) -> &[String] {
    &v[v.len().saturating_sub(n)..]
}

/// What a finished suite run says. Everything the facade must write or record comes back:
/// the parsed table, the column report, the run's extra fields, its stages, its
/// classification, and the reference baseline's text when one was asked for and may be
/// banked. `guest` is the guest floor's text (None when there is no floor file).
pub struct Grade<'a> {
    pub suite: &'a str,
    pub out: &'a [String],
    pub guest: Option<&'a str>,
    pub guest_path: &'a str,
    pub run_dir: &'a Path,
    pub windows: &'a Value,
    pub corpus_built: &'a Value,
    pub regraded: Option<&'a str>,
    pub write_baseline: bool,
    pub only: bool,
}

pub fn grade(g: &Grade) -> Result<Value> {
    let marker = marker(g.suite)?;
    let text = g.out.join("\n");
    let (results, done, nosubtests): (Vec<TestResult>, Value, Map<String, Value>) =
        if g.suite == "kmtest" {
            let (rows, done, _bugcheck) = conformance::parse_kmtest(&text)?;
            (rows, json!(done), Map::new())
        } else {
            let (rows, meta) = conformance::parse_apitest(&text)?;
            let nos = meta["nosubtests"].as_object().cloned().unwrap_or_default();
            (rows, meta["done"].clone(), nos)
        };
    let finished = !matches!(&done, Value::Null | Value::Bool(false))
        && done.as_str().map_or(true, |s| !s.is_empty());
    let none = BTreeSet::new();
    let table = results
        .iter()
        .map(|r| conformance::fmt(r, &none))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let floor = match g.guest {
        Some(t) => conformance::load_baseline_text(t)?
            .as_object()
            .cloned()
            .unwrap_or_default(),
        None => Map::new(),
    };
    let mut report = column_report(&results, &floor);
    let clean = results
        .iter()
        .filter(|r| r.status.as_deref() == Some("OK") && r.fail == 0)
        .count();
    // A test the VM died under twice (kmtest-sweep.ps1) says nothing about Windows.
    let lost: BTreeSet<&str> = g
        .out
        .iter()
        .filter(|ln| ln.starts_with("[KMTEST-SWEEP] VM-LOST "))
        .filter_map(|ln| ln.split_whitespace().nth(2))
        .collect();
    let mut unrun: Vec<String> = results
        .iter()
        .filter(|r| not_run(r) || lost.contains(r.name.as_str()))
        .map(|r| {
            if lost.contains(r.name.as_str()) {
                format!("{} (VM lost under it twice)", r.name)
            } else {
                format!("{} ({}, {} harness error(s))", r.name, status(r), r.errors)
            }
        })
        .collect();
    // A suite whose binary Windows will not even load enumerates no subtests, so it has no
    // rows at all; without this it would vanish from the column silently.
    let mut nosubtests: Vec<(&String, &Value)> = nosubtests.iter().collect();
    nosubtests.sort_by(|a, b| a.0.cmp(b.0));
    for (name, v) in nosubtests {
        if let Some(why) = v[0].as_i64().and_then(load_failure) {
            unrun.push(format!("{name} (whole suite: {why})"));
        }
    }
    let mut column: Vec<String> = unrun
        .iter()
        .filter(|u| u.contains("whole suite"))
        .map(|u| format!("NOT RUN on Windows: {u}"))
        .collect();
    column.append(&mut report);
    let nothing_ran = !results
        .iter()
        .any(|r| matches!(r.status.as_deref(), Some("OK" | "FAIL-EXIT")));
    let signed = g
        .out
        .iter()
        .filter(|ln| ln.starts_with(marker) && ln.contains(" signed "))
        .find_map(|ln| ln.split_once("] ").map(|(_, rest)| rest.to_owned()));
    let has_floor = !floor.is_empty();
    let table_path = g.run_dir.join(format!("{}.txt", g.suite));
    let column_path = g.run_dir.join("column.txt");
    let extra = json!({
        "signed": signed,
        "sweep": done,
        "rows": results.len(),
        "not_run": unrun,
        "clean": clean,
        "guest_floor": if has_floor { json!(g.guest_path) } else { Value::Null },
        "disagreements": column.len(),
        "table": table_path.to_string_lossy(),
        "column": column_path.to_string_lossy(),
    });
    let mut detail = format!("{} rows, {clean} clean on Windows", results.len());
    if !unrun.is_empty() {
        detail += &format!(", {} NOT RUN (harness)", unrun.len());
    }
    detail += &if has_floor {
        format!(", {} disagree with the guest floor", column.len())
    } else {
        ", no guest floor".into()
    };
    if !finished {
        detail += "; SWEEP DID NOT FINISH";
    }
    let st = if !finished || nothing_ran {
        "fail"
    } else if !unrun.is_empty() {
        "warn"
    } else {
        "ok"
    };
    let mut stages = vec![stage("column", st, detail, &column[..column.len().min(12)])];
    let upper = g.suite.to_uppercase();
    let mut classification = Value::Null;
    let mut baseline = Value::Null;
    if !finished {
        classification = json!(format!("WINREF_{upper}_INCOMPLETE"));
    } else if nothing_ran {
        // Every row is a harness failure (the first kmtest column: a driver that would not
        // load). That is a broken rig, not a reference: refuse to bank it.
        classification = json!(format!("WINREF_{upper}_NOT_RUN"));
    } else if g.write_baseline && g.only {
        // A subset is not the column: banking it would drop every row it did not run.
        stages.push(stage(
            "baseline",
            "fail",
            "--write-baseline refused with --only".into(),
            &[],
        ));
    } else if g.write_baseline {
        let py = |v: &Value| match v {
            Value::String(s) => s.clone(),
            Value::Null => "None".into(),
            other => other.to_string(),
        };
        let mut head = format!(
            "# {s} REFERENCE column: what Windows 11 itself does with each test.\n\
             # `vibeos-cli winref {s} --write-baseline`, {}, corpus built {}",
            py(g.windows),
            py(g.corpus_built),
            s = g.suite,
        );
        if let Some(prev) = g.regraded {
            let name = Path::new(prev)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            head += &format!(", from {name}");
        }
        head += ".\n\
            # Not a floor the guest must match row for row. A row failing here fails on\n\
            # Windows 11 itself: an expectation Windows does not meet (often 2003-era\n\
            # behaviour), a test that is timing-sensitive in a VM, or one our toolchain\n\
            # builds differently from its author's.\n";
        for u in &unrun {
            head += &format!("# NOT RUN on Windows: {u}\n");
        }
        baseline = json!(head + &table);
    }
    Ok(json!({
        "table": table,
        "column": column,
        "extra": extra,
        "stages": stages,
        "classification": classification,
        "baseline": baseline,
    }))
}

fn first<'a>(lines: &'a [String], prefix: &str) -> Option<&'a String> {
    lines.iter().find(|ln| ln.starts_with(prefix))
}

fn removeprefix<'a>(s: &'a str, p: &str) -> &'a str {
    s.strip_prefix(p).unwrap_or(s)
}

/// A payload run's reading: the payload's output (`OUT: ` stripped), the Windows build that
/// ran it, its exit code, and the stage. Graded OK only when QEMU exited (the guest powered
/// off) AND the payload reported an exit code.
pub fn payload_result(lines: &[String], outcome: &str) -> Result<Value> {
    let out: Vec<String> = lines
        .iter()
        .filter_map(|ln| ln.strip_prefix("OUT: ").map(str::to_owned))
        .collect();
    let ready = first(lines, "WINREF: ready");
    let exit_line = first(lines, "WINREF: exit=");
    let exit =
        match exit_line {
            Some(ln) => {
                let v = ln.split_once('=').map_or("", |(_, v)| v).trim();
                Some(v.parse::<i64>().map_err(|_| {
                    usage(format!("winref: unreadable payload exit code in {ln:?}"))
                })?)
            }
            None => None,
        };
    let windows = ready.map(|r| removeprefix(r, "WINREF: ready ").trim().to_owned());
    let (stage, classification, ok) = match exit_line {
        Some(ln) if outcome == "exited" => (
            stage(
                "run",
                "ok",
                format!(
                    "{}, {} output lines",
                    removeprefix(ln, "WINREF: "),
                    out.len()
                ),
                tail(&out, 8),
            ),
            Value::Null,
            true,
        ),
        _ => (
            stage(
                "run",
                "fail",
                if exit_line.is_none() {
                    format!(
                        "{outcome}; ready={}; no exit line",
                        if ready.is_some() { "yes" } else { "no" }
                    )
                } else {
                    outcome.to_owned()
                },
                tail(lines, 8),
            ),
            json!("WINREF_RUN_FAIL"),
            false,
        ),
    };
    Ok(json!({
        "output": out,
        "windows": windows,
        "payload_exit": exit,
        "stage": stage,
        "classification": classification,
        "ok": ok,
    }))
}

/// An install's reading: the setup steps it logged, and whether it finished. `done` is the
/// `WINREF-SETUP: done ...` line; `setup` in the result is what state.json records.
pub fn install_result(lines: &[String], outcome: &str) -> Value {
    let setup: Vec<String> = lines
        .iter()
        .filter(|ln| ln.starts_with("WINREF-SETUP:"))
        .cloned()
        .collect();
    let done = first(&setup, "WINREF-SETUP: done");
    let fail = (outcome != "exited" || done.is_none()).then(|| {
        stage(
            "install",
            "fail",
            format!(
                "{outcome}; last setup line: {}",
                setup.last().map_or("(none)", String::as_str)
            ),
            tail(&setup, 8),
        )
    });
    json!({
        "setup_lines": setup,
        "done": done,
        "build": done.map(|d| removeprefix(d, "WINREF-SETUP: done ").trim().to_owned()),
        "stage": fail,
        "classification": fail.as_ref().map(|_| "WINREF_INSTALL_FAIL"),
    })
}

/// A maintenance run's reading. Success needs the script's own `WINREF-MAINT: done`: any
/// WINREF: line is not enough, because the runner prints its own, and a maintenance run whose
/// script never ran graded itself OK that way.
pub fn maint_result(lines: &[String], outcome: &str) -> Value {
    let maint: Vec<String> = lines
        .iter()
        .filter(|ln| {
            ln.starts_with("WINREF-MAINT:")
                || ln.starts_with("WINREF:")
                || ln.starts_with("OUT: WINREF-MAINT:")
        })
        .map(|ln| removeprefix(ln, "OUT: ").to_owned())
        .collect();
    let ok = outcome == "exited"
        && !maint.iter().any(|ln| ln.contains("FAILED"))
        && maint.iter().any(|ln| ln == MAINT_DONE);
    json!({
        "maint_lines": maint,
        "stage": stage(
            "maint",
            if ok { "ok" } else { "fail" },
            format!(
                "{outcome}; {}",
                maint.last().map_or("no maintenance output", String::as_str)
            ),
            tail(&maint, 10),
        ),
    })
}

fn py_int(v: &str, what: &str) -> Result<i64> {
    v.trim().parse().map_err(|_| {
        usage(format!(
            "winref: unreadable {what} {v:?} in the 7-Zip listing"
        ))
    })
}

/// (offset, size) in bytes of the largest NTFS partition in a `7z l -slt` listing; (0, 0)
/// when there is none.
pub fn largest_ntfs(listing: &str) -> Result<(i64, i64)> {
    let mut best = (0i64, 0i64);
    let mut entry: Vec<(String, String)> = Vec::new();
    let mut lines = py_lines(&listing.replace('\r', ""));
    lines.push(String::new());
    let get = |e: &[(String, String)], k: &str| {
        e.iter()
            .rev()
            .find(|(key, _)| key == k)
            .map(|(_, v)| v.clone())
    };
    for line in lines {
        if let Some((k, v)) = line.split_once(" = ") {
            entry.push((k.to_owned(), v.to_owned()));
            continue;
        }
        let path = get(&entry, "Path").unwrap_or_default();
        if path.ends_with(".ntfs") {
            if let Some(off) = get(&entry, "Offset") {
                let off = py_int(&off, "Offset")?;
                let size = get(&entry, "Size")
                    .ok_or_else(|| usage(format!("winref: {path} has an Offset but no Size")))?;
                let size = py_int(&size, "Size")?;
                if size > best.1 {
                    best = (off, size);
                }
            }
        }
        entry.clear();
    }
    Ok(best)
}

/// (skip, count) in MiB for `qemu-img dd bs=1M` to copy exactly one partition. qemu-img's dd
/// counts the SKIPPED blocks in `count` (unlike coreutils dd): a count of the partition's
/// length alone copied 317 MiB short, dropped NTFS's backup boot sector off the end, and
/// 7-Zip refused the result as "not an archive".
pub fn dd_window(offset: i64, size: i64) -> Result<(i64, i64)> {
    const MIB: i64 = 1 << 20;
    if offset.rem_euclid(MIB) != 0 {
        return Err(usage(format!("partition at {offset} is not MiB-aligned")));
    }
    let skip = offset.div_euclid(MIB);
    Ok((skip, skip - (-size).div_euclid(MIB)))
}

/// QMP send-key chords that type `text` on a US layout: one chord per character.
pub fn keys_for(text: &str) -> Result<Vec<Vec<String>>> {
    let plain = |c: char| {
        Some(match c {
            ' ' => "spc",
            '-' => "minus",
            '\\' => "backslash",
            '.' => "dot",
            '/' => "slash",
            '=' => "equal",
            ';' => "semicolon",
            '\'' => "apostrophe",
            ',' => "comma",
            _ => return None,
        })
    };
    let shifted = |c: char| {
        Some(match c {
            ':' => "semicolon",
            '_' => "minus",
            '"' => "apostrophe",
            '+' => "equal",
            '?' => "slash",
            _ => return None,
        })
    };
    text.chars()
        .map(|c| {
            Ok(if c.is_ascii_lowercase() || c.is_ascii_digit() {
                vec![c.to_string()]
            } else if c.is_ascii_uppercase() {
                vec!["shift".into(), c.to_ascii_lowercase().to_string()]
            } else if let Some(k) = plain(c) {
                vec![k.into()]
            } else if let Some(k) = shifted(c) {
                vec!["shift".into(), k.into()]
            } else {
                return Err(usage(format!("cannot type {}", py_repr_char(c))));
            })
        })
        .collect()
}

fn py_repr_char(c: char) -> String {
    if c == '\'' {
        "\"'\"".into()
    } else {
        format!("'{c}'")
    }
}

fn strings(a: &Value, k: &str) -> Vec<String> {
    a[k].as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect()
}

fn text<'a>(a: &'a Value, k: &str) -> &'a str {
    a[k].as_str().unwrap_or("")
}

fn rows(a: &Value, k: &str) -> Result<Vec<TestResult>> {
    Ok(serde_json::from_value(a[k].clone()).map_err(|e| format!("winref: {k}: {e}"))?)
}

pub fn dispatch(request: &Value) -> Result<Value> {
    let a = &request["kwargs"];
    match request["op"].as_str().unwrap_or("") {
        "winref_launch_argv" => Ok(json!(launch_argv(a)?)),
        "winref_staged_bin" => Ok(json!(staged_bin(
            text(a, "manifest"),
            Path::new(text(a, "repo"))
        ))),
        "winref_kmtest_order" => Ok(json!(kmtest_order(
            &strings(a, "listed"),
            &strings(a, "built").into_iter().collect()
        ))),
        "winref_not_run" => Ok(json!(not_run(
            &serde_json::from_value(a["row"].clone()).map_err(|e| format!("winref: row: {e}"))?
        ))),
        "winref_column_report" => Ok(json!(column_report(
            &rows(a, "win")?,
            &a["guest"].as_object().cloned().unwrap_or_default()
        ))),
        "winref_grade" => grade(&Grade {
            suite: text(a, "suite"),
            out: &strings(a, "out"),
            guest: a["guest"].as_str(),
            guest_path: text(a, "guest_path"),
            run_dir: Path::new(text(a, "run_dir")),
            windows: &a["windows"],
            corpus_built: &a["corpus_built"],
            regraded: a["regraded"].as_str(),
            write_baseline: a["write_baseline"] == true,
            only: a["only"] == true,
        }),
        "winref_payload_result" => payload_result(&strings(a, "lines"), text(a, "outcome")),
        "winref_install_result" => Ok(install_result(&strings(a, "lines"), text(a, "outcome"))),
        "winref_maint_result" => Ok(maint_result(&strings(a, "lines"), text(a, "outcome"))),
        "winref_largest_ntfs" => Ok(json!(largest_ntfs(text(a, "listing"))?)),
        "winref_dd_window" => Ok(json!(dd_window(
            a["offset"].as_i64().ok_or("winref: offset")?,
            a["size"].as_i64().ok_or("winref: size")?
        )?)),
        "winref_keys_for" => Ok(json!(keys_for(text(a, "text"))?)),
        op => Err(format!("unknown winref operation {op}").into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_payload_without_an_exit_line_fails_even_when_windows_powered_off() {
        let got = payload_result(
            &lines(&[
                "WINREF: ready build=26300",
                "OUT: hello",
                "WINREF: poweroff",
            ]),
            "exited",
        )
        .unwrap();
        assert_eq!(got["classification"], "WINREF_RUN_FAIL");
        assert_eq!(got["stage"]["detail"], "exited; ready=yes; no exit line");
        assert_eq!(got["windows"], "build=26300");
        let ok = payload_result(&lines(&["OUT: hi", "WINREF: exit=3"]), "exited").unwrap();
        assert_eq!(ok["stage"]["detail"], "exit=3, 1 output lines");
        assert_eq!(ok["payload_exit"], 3);
    }

    #[test]
    fn maintenance_needs_the_scripts_own_done_not_the_runners() {
        let runner_only = maint_result(&lines(&["WINREF: ready", "WINREF: exit=0"]), "exited");
        assert_eq!(runner_only["stage"]["status"], "fail");
        let done = maint_result(&lines(&["OUT: WINREF-MAINT: done"]), "exited");
        assert_eq!(done["stage"]["status"], "ok");
        let failed = maint_result(
            &lines(&["OUT: WINREF-MAINT: step FAILED", "OUT: WINREF-MAINT: done"]),
            "exited",
        );
        assert_eq!(failed["stage"]["status"], "fail");
    }

    #[test]
    fn the_column_is_not_banked_from_a_subset_or_a_rig_that_ran_nothing() {
        let out = lines(&[
            "[KMTEST-SWEEP] launch A",
            "[KMTEST:BEGIN A]",
            "[KMTEST:END A pass=1 fail=0 skip=0]",
            "[KMTEST-SWEEP] exit A code=0",
            "[KMTEST-SWEEP] done total=1",
        ]);
        let base = |only| Grade {
            suite: "kmtest",
            out: &out,
            guest: None,
            guest_path: "",
            run_dir: Path::new("/r"),
            windows: &Value::Null,
            corpus_built: &Value::Null,
            regraded: None,
            write_baseline: true,
            only,
        };
        let subset = grade(&base(true)).unwrap();
        assert_eq!(subset["baseline"], Value::Null);
        assert_eq!(
            subset["stages"][1]["detail"],
            "--write-baseline refused with --only"
        );
        let whole = grade(&base(false)).unwrap();
        let text = whole["baseline"].as_str().unwrap();
        assert!(text
            .contains("`vibeos-cli winref kmtest --write-baseline`, None, corpus built None.\n"));
        assert!(
            text.ends_with("A OK pass=1 fail=0 skip=0 exit=0\n"),
            "{text}"
        );
    }

    #[test]
    fn a_launch_names_every_cd_on_its_own_ahci_port() {
        let a = json!({"qemu": "q.exe", "uid": "u", "accel": "whpx", "irqchip": "on",
            "cpu": "EPYC", "cpus": 4, "mem": "8G", "code": "C", "vars": "V", "disk": "D",
            "qmp_port": 1, "console_port": 2, "show": false, "cds": ["a.iso", "b.iso"]});
        let argv = launch_argv(&a).unwrap();
        assert!(argv.contains(&"whpx,kernel-irqchip=on".to_owned()));
        assert_eq!(argv.last().unwrap(), "ide-cd,drive=cd2,bus=ide.2");
        assert!(!argv.contains(&"-no-reboot".to_owned()));
        let mut once = a.clone();
        once["no_reboot"] = json!(true);
        assert!(launch_argv(&once)
            .unwrap()
            .contains(&"-no-reboot".to_owned()));
        let six = json!({"cds": ["1", "2", "3", "4", "5", "6"]});
        let mut a6 = a.clone();
        a6["cds"] = six["cds"].clone();
        assert!(launch_argv(&a6).is_err());
    }
}
