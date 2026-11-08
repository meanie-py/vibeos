//! `vibeos-cli stress`: "it only fails under host load" as named mechanisms, one knob at a
//! time, and the reading of the resulting matrix.
//!
//! A loaded host changes three things at once and never the same way twice, so stress boots
//! the SAME media under one-knob profiles and reports a failure rate per profile:
//!
//! | profile     | knob                                   | a failure here is                  |
//! |-------------|----------------------------------------|------------------------------------|
//! | baseline    | none (control)                         | a plain bug, or the media          |
//! | slow-cpu    | `-icount shift=N` (guest clock races)  | a guest-side TIMEOUT               |
//! | lockstep    | `-smp 2`, single-threaded TCG          | a cross-CPU race, stably interleaved |
//! | starved-smp | `-smp 2` MTTCG pinned to few host cores| the host-scheduler preemption race |
//! | tight-clock | harness boot budgets x0.5              | a healthy boot crossing the budget |
//!
//! This module owns the matrix and its reading; the boots (and host-core pinning) stay with
//! the facade's smoke path until the CLI entry is native.
use crate::{Error, Result};
use serde_json::{json, Value};

pub const DEFAULT_ICOUNT_SHIFT: i64 = 7;
pub const DEFAULT_TIMEOUT_SCALE: f64 = 0.5;
const ORDER: [&str; 5] = [
    "baseline",
    "slow-cpu",
    "lockstep",
    "starved-smp",
    "tight-clock",
];

/// Python's `str(float)`, for the timeout-scale knob's environment value.
fn py_float(x: f64) -> String {
    if x.fract() == 0.0 && x.abs() < 1e16 {
        format!("{x:.1}")
    } else {
        format!("{x}")
    }
}

/// The selected profiles, in matrix order of selection. `selection` is a comma list, empty
/// or "all" for the whole matrix; an unknown name is a usage error.
pub fn profiles(
    selection: &str,
    shift: i64,
    scale: f64,
    smp: i64,
    cores: i64,
) -> Result<Vec<Value>> {
    let make = |name: &str| -> Value {
        let (mechanism, env, smp, pin) = match name {
            "baseline" => ("control: the media on an unloaded host", json!({}), 0, 0),
            "slow-cpu" => (
                "guest-side timeout (slow emulation)",
                json!({"VIBEOS_QEMU_ICOUNT_SHIFT": shift.to_string()}),
                0,
                0,
            ),
            "lockstep" => (
                "cross-CPU race (deterministic interleaving)",
                json!({"VIBEOS_TCG_THREAD": "single"}),
                smp,
                0,
            ),
            "starved-smp" => (
                "cross-CPU race (host scheduler preemption)",
                json!({}),
                smp,
                cores,
            ),
            _ => (
                "harness wall clock (healthy boot, budget too small)",
                json!({"VIBEOS_BOOT_TIMEOUT_SCALE": py_float(scale)}),
                0,
                0,
            ),
        };
        json!({"name": name, "mechanism": mechanism, "env": env, "smp": smp, "pin_cores": pin})
    };
    let want: Vec<&str> = selection
        .split(',')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();
    let want: Vec<&str> = if want.is_empty() || want == ["all"] {
        ORDER.to_vec()
    } else {
        want
    };
    let unknown: Vec<&str> = want
        .iter()
        .copied()
        .filter(|p| !ORDER.contains(p))
        .collect();
    if !unknown.is_empty() {
        let quoted: Vec<String> = unknown.iter().map(|u| format!("'{u}'")).collect();
        return Err(Error {
            kind: Some("usage".into()),
            message: format!(
                "stress: unknown profile(s) [{}]; known: {}",
                quoted.join(", "),
                ORDER.join(", ")
            ),
        });
    }
    Ok(want.into_iter().map(make).collect())
}

fn s<'a>(r: &'a Value, k: &str) -> &'a str {
    r[k].as_str().unwrap_or("")
}

/// Python's `statistics.median`.
fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(f64::total_cmp);
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

/// One line per profile: boots, failures, failure families, median duration.
pub fn summarize(rows: &[Value]) -> Vec<String> {
    let mut order: Vec<String> = Vec::new();
    for r in rows {
        let p = s(r, "profile").to_owned();
        if !order.contains(&p) {
            order.push(p);
        }
    }
    let mut out = vec![format!(
        "{:12} {:>5} {:>4} {:>7}  failures",
        "profile", "boots", "fail", "median"
    )];
    for name in order {
        let rs: Vec<&Value> = rows.iter().filter(|r| s(r, "profile") == name).collect();
        let fails: Vec<&&Value> = rs.iter().filter(|r| r["ok"] != true).collect();
        // Failure families, first-seen order, then most frequent first (a stable sort).
        let mut fam: Vec<(String, usize)> = Vec::new();
        for r in &fails {
            let mut key = [s(r, "classification"), s(r, "outcome")]
                .into_iter()
                .find(|k| !k.is_empty())
                .unwrap_or("?")
                .to_owned();
            if !s(r, "stop_code").is_empty() {
                key += &format!(" stop={}", s(r, "stop_code"));
            }
            if !s(r, "top_symbol").is_empty() {
                key += &format!(" @ {}", s(r, "top_symbol"));
            }
            match fam.iter_mut().find(|(k, _)| *k == key) {
                Some((_, n)) => *n += 1,
                None => fam.push((key, 1)),
            }
        }
        fam.sort_by(|a, b| b.1.cmp(&a.1));
        let med = median(rs.iter().filter_map(|r| r["duration_s"].as_f64()).collect());
        let fams = fam
            .iter()
            .map(|(k, n)| format!("{k} x{n}"))
            .collect::<Vec<_>>()
            .join("; ");
        out.push(format!(
            "{name:12} {:5} {:4} {med:6.0}s  {}",
            rs.len(),
            fails.len(),
            if fams.is_empty() { "-" } else { &fams }
        ));
    }
    out
}

/// Read the matrix the way the module docs say to.
pub fn diagnosis(rows: &[Value]) -> String {
    let ran: std::collections::BTreeSet<&str> = rows.iter().map(|r| s(r, "profile")).collect();
    let fails: std::collections::BTreeSet<&str> = rows
        .iter()
        .filter(|r| r["ok"] != true)
        .map(|r| s(r, "profile"))
        .collect();
    if fails.is_empty() {
        return "no profile failed: whatever this flake is, these three mechanisms are not it \
                at this sample size — go look for a real bug, not for host weather"
            .into();
    }
    if fails.contains("baseline") {
        return "baseline fails: this is not a load flake, it is a bug (or stale media)".into();
    }
    let mut parts = Vec::new();
    if fails.contains("slow-cpu") {
        parts.push(
            "slow-cpu fails -> a guest-side timeout fires when the guest clock outruns the CPU"
                .to_owned(),
        );
    }
    let races: Vec<&str> = ["lockstep", "starved-smp"]
        .into_iter()
        .filter(|p| fails.contains(p))
        .collect();
    if !races.is_empty() {
        parts.push(format!(
            "{} fails -> a cross-CPU race; lockstep gives a stable interleaving to debug",
            races.join(", ")
        ));
    }
    if fails.contains("tight-clock") && fails.len() == 1 {
        parts.push(
            "ONLY tight-clock fails -> a healthy boot crossing the wall clock; the budget is the bug"
                .into(),
        );
    } else if fails.contains("tight-clock") {
        parts.push("tight-clock also fails (expected alongside a real slowdown)".into());
    }
    let missing: Vec<&str> = ["baseline", "lockstep", "slow-cpu", "starved-smp"]
        .into_iter()
        .filter(|p| !ran.contains(p))
        .collect();
    if !missing.is_empty() {
        parts.push(format!("(profiles not run: {})", missing.join(", ")));
    }
    parts.join("; ")
}

pub fn dispatch(request: &Value) -> Result<Value> {
    let a = &request["kwargs"];
    let rows = || a["rows"].as_array().cloned().unwrap_or_default();
    let num = |k: &str, default: i64| a[k].as_i64().filter(|&n| n != 0).unwrap_or(default);
    match request["op"].as_str().unwrap_or("") {
        "stress_profiles" => Ok(json!(profiles(
            a["selection"].as_str().unwrap_or(""),
            num("shift", DEFAULT_ICOUNT_SHIFT),
            a["timeout_scale"]
                .as_f64()
                .filter(|&x| x != 0.0)
                .unwrap_or(DEFAULT_TIMEOUT_SCALE),
            num("smp", 2),
            num("cores", 1),
        )?)),
        "stress_summarize" => Ok(json!(summarize(&rows()))),
        "stress_diagnosis" => Ok(json!(diagnosis(&rows()))),
        op => Err(format!("unknown stress operation {op}").into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(p: &str, ok: bool, d: f64) -> Value {
        json!({"profile": p, "ok": ok, "duration_s": d,
               "classification": if ok { "" } else { "HANG_TIMEOUT" }})
    }

    #[test]
    fn medians_round_half_to_even_like_the_python_format() {
        let rows = [row("baseline", true, 40.0), row("baseline", true, 41.0)];
        assert!(
            summarize(&rows)[1].contains("    40s"),
            "{:?}",
            summarize(&rows)
        );
        let rows = [row("baseline", true, 41.0), row("baseline", true, 42.0)];
        assert!(summarize(&rows)[1].contains("    42s"));
    }

    #[test]
    fn the_matrix_names_unknown_profiles_and_keeps_selection_order() {
        let p = profiles("tight-clock,baseline", 7, 0.5, 2, 1).unwrap();
        assert_eq!(p[0]["env"]["VIBEOS_BOOT_TIMEOUT_SCALE"], "0.5");
        assert_eq!(p[1]["name"], "baseline");
        let e = profiles("hot-host,x", 7, 0.5, 2, 1).unwrap_err();
        assert!(e.message.contains("['hot-host', 'x']"), "{}", e.message);
        assert_eq!(
            profiles("", 7, 1.0, 2, 1).unwrap()[4]["env"]["VIBEOS_BOOT_TIMEOUT_SCALE"],
            "1.0"
        );
    }
}
