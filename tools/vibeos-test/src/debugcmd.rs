//! `vibeos-cli debug ...` and `check`: which shell tool a debug verb runs, whose session owns
//! a debug VM, what `debug diff` compares, and what a `check` invocation asks for.
//!
//! Execution stays with the facade (`tests/qemu/cli/debugcmd.py`): it runs the shell toolkit
//! and the lints with the plan made here.
use crate::verdict::py_lines;
use crate::Result;
use regex::Regex;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::Path;
use std::sync::OnceLock;

/// The shell tool behind a debug verb (`bt`, `triage-bugcheck`, `screenshot` and `gdb` are
/// inspect.sh `--command` forms).
pub fn verb_tool(verb: &str) -> Option<&'static str> {
    Some(match verb {
        "start" => "start.sh",
        "inspect" | "bt" | "triage-bugcheck" | "screenshot" | "gdb" => "inspect.sh",
        "run-until" => "run-until.sh",
        "wait-for" => "wait-for.sh",
        "tail" => "tail.sh",
        "status" => "status.sh",
        "stop" => "stop.sh",
        "profile" => "sample-pc.sh",
        _ => return None,
    })
}

/// One serial line with its run-specific numbers erased, so two runs compare by what they
/// said. `pid=1234`, `pid: 1234` and `TID 7` all become `pid=N`/`TID=N`: the Python kept the
/// number for every spelling but `pid 1234` (it rebuilt the match from its first
/// whitespace-separated word, which for `pid=1234` is the whole match), so every line naming
/// a pid showed up as a difference.
pub fn normalize(line: &str) -> String {
    static HEX: OnceLock<Regex> = OnceLock::new();
    static PID: OnceLock<Regex> = OnceLock::new();
    let hex = HEX.get_or_init(|| Regex::new(r"0x[0-9a-fA-F]+").unwrap());
    let pid = PID.get_or_init(|| Regex::new(r"(?i)\b(pid|tid)[=: ]+\d+").unwrap());
    let line = hex.replace_all(line, "0xX");
    pid.replace_all(&line, "${1}=N").trim_end().to_owned()
}

/// Normalized lines unique to each side, sorted. Blank lines say nothing.
pub fn diff(a: &str, b: &str) -> (Vec<String>, Vec<String>) {
    let set = |t: &str| -> BTreeSet<String> {
        py_lines(t)
            .iter()
            .filter(|l| !l.trim().is_empty())
            .map(|l| normalize(l))
            .collect()
    };
    let (a, b) = (set(a), set(b));
    (
        a.difference(&b).cloned().collect(),
        b.difference(&a).cloned().collect(),
    )
}

const SHELLS: [&str; 6] = ["bash", "sh", "dash", "zsh", "fish", "ksh"];

/// The first non-shell ancestor of `start`, read from `proc` (qemu/lib/common.sh's
/// owner_pid_for, seeded ABOVE the CLI). start.sh walks up from its own parent and stops at
/// the first non-shell, which under `vibeos-cli debug start` is the CLI: a process that exits
/// as soon as start.sh returns. The VM was then recorded as owned by a dead pid, and the next
/// smoke from ANY lane reaped it as an orphan (two arm64 debug VMs SIGTERMed mid-gdb,
/// 2026-10-03). The session that outlives the CLI is whatever ran it, past the shells.
pub fn session_owner(proc: &Path, start: i64) -> i64 {
    let mut p = start;
    for _ in 0..12 {
        if p <= 1 {
            break;
        }
        let dir = proc.join(p.to_string());
        let (Ok(comm), Ok(stat)) = (
            std::fs::read_to_string(dir.join("comm")),
            std::fs::read_to_string(dir.join("stat")),
        ) else {
            break;
        };
        let comm = comm.trim();
        let shell = SHELLS
            .iter()
            .any(|s| comm == *s || comm.strip_prefix('-') == Some(s));
        if !shell {
            return p;
        }
        // Field 4 (ppid), after the parenthesized comm, which may itself hold ") ".
        let Some(ppid) = stat
            .rsplit_once(") ")
            .and_then(|(_, rest)| rest.split_whitespace().nth(1))
            .and_then(|s| s.parse().ok())
        else {
            break;
        };
        p = ppid;
    }
    start
}

/// What a `check` invocation asks for, or why it is refused (`rc` 2 with the lines to print).
/// `known` is the lint tool inventory; it is consulted only by the modes that name tools.
pub struct CheckArgs<'a> {
    pub update_baseline: Option<&'a str>,
    pub only: Option<&'a str>,
    pub scope: Option<&'a str>,
    pub strict: bool,
    pub arch: Option<&'a str>,
    pub known: &'a BTreeSet<String>,
}

fn refuse(lines: &[String]) -> Value {
    json!({"rc": 2, "lines": lines})
}

fn names(list: &str) -> BTreeSet<String> {
    list.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

fn joined(set: &BTreeSet<String>) -> String {
    set.iter().cloned().collect::<Vec<_>>().join(", ")
}

pub fn check_plan(a: &CheckArgs) -> Value {
    let arch = a.arch.filter(|s| !s.is_empty());
    if let Some(scope) = a.scope {
        let known_scope = ["source", "source-early", "source-deferred", "arch"].contains(&scope);
        if !known_scope || a.update_baseline.is_some() || a.only.is_some() {
            return refuse(&[
                "invalid --scope or combination with --only/--update-baseline".into(),
            ]);
        }
        if !a.strict || (scope == "arch" && arch.is_none()) {
            return refuse(&[
                "--scope requires --strict; --scope arch also requires --arch".into(),
            ]);
        }
        return json!({
            "mode": "scope",
            "scope": scope,
            "run": format!("check-{scope}"),
            "arch": arch.unwrap_or("all"),
            "extra": {"coverage": "partial", "scope": scope},
            "stage": {"name": "partial", "status": "warn",
                      "detail": format!("{scope} partition only; not a full gate")},
        });
    }
    if let Some(selection) = a.only {
        if a.update_baseline.is_some() {
            return refuse(&["--only cannot be combined with --update-baseline".into()]);
        }
        let only = names(selection);
        let unknown: BTreeSet<String> = only.difference(a.known).cloned().collect();
        if only.is_empty() || !unknown.is_empty() {
            return refuse(&[
                format!("empty or unknown scanner selection: {}", joined(&unknown)),
                format!("known: {}", joined(a.known)),
            ]);
        }
        // A separate command identity prevents a partial result being mistaken for a
        // complete check in artifact discovery or fleet health history.
        return json!({
            "mode": "partial",
            "run": "check-partial",
            "arch": arch.unwrap_or("all"),
            "only": only,
            "strict": a.strict,
            "extra": {"coverage": "partial", "selected_scanners": only,
                      "omitted_scanners": a.known.difference(&only).collect::<Vec<_>>()},
            "stage": {"name": "partial", "status": "warn",
                      "detail": "selected source scanners only; not a full gate"},
        });
    }
    if let Some(ub) = a.update_baseline {
        // "__ALL__" is the blanket re-bless (the footgun, see lints); a list scopes it.
        if ub == "__ALL__" {
            return json!({"mode": "update_baseline", "only": Value::Null});
        }
        let only = names(ub);
        let unknown: BTreeSet<String> = only.difference(a.known).cloned().collect();
        if !unknown.is_empty() {
            return refuse(&[
                format!("unknown lint tool(s): {}", joined(&unknown)),
                format!("known: {}", joined(a.known)),
            ]);
        }
        return json!({"mode": "update_baseline", "only": only});
    }
    // NEW findings fail with or without --strict; --strict adds the coverage rule (a missing
    // tool fails) and the per-arch guards.
    json!({"mode": "full", "run": "check", "arch": arch.unwrap_or("all"), "strict": a.strict})
}

/// Whether a `check` invocation needs the lint tool inventory to be planned.
pub fn check_needs_inventory(a: &Value) -> bool {
    a["scope"].is_null()
        && (a["only"].is_string()
            || a["update_baseline"]
                .as_str()
                .is_some_and(|ub| ub != "__ALL__"))
}

pub fn dispatch(request: &Value) -> Result<Value> {
    let a = &request["kwargs"];
    let text = |k: &str| a[k].as_str().unwrap_or("");
    match request["op"].as_str().unwrap_or("") {
        "debugcmd_verb_tool" => Ok(json!(verb_tool(text("verb")))),
        "debugcmd_normalize" => Ok(json!(normalize(text("line")))),
        "debugcmd_diff" => Ok(json!(diff(text("a"), text("b")))),
        "debugcmd_session_owner" => Ok(json!(session_owner(
            Path::new(a["proc"].as_str().unwrap_or("/proc")),
            a["start"].as_i64().ok_or("debugcmd: start pid")?
        ))),
        "debugcmd_check_needs_inventory" => Ok(json!(check_needs_inventory(a))),
        "debugcmd_check_plan" => {
            let known: BTreeSet<String> = a["known"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect();
            Ok(check_plan(&CheckArgs {
                update_baseline: a["update_baseline"].as_str(),
                only: a["only"].as_str(),
                scope: a["scope"].as_str(),
                strict: a["strict"] == true,
                arch: a["arch"].as_str(),
                known: &known,
            }))
        }
        op => Err(format!("unknown debugcmd operation {op}").into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pid_spelling_normalizes_not_just_the_spaced_one() {
        assert_eq!(
            normalize("smss pid=1234 at 0xFFFF8000  "),
            "smss pid=N at 0xX"
        );
        assert_eq!(normalize("PID: 77 TID 9"), "PID=N TID=N");
        assert_eq!(normalize("rapid=5"), "rapid=5");
    }

    #[test]
    fn the_owner_walk_skips_shells_and_reads_a_ppid_past_a_paren_in_comm() {
        let d = tempfile::tempdir().unwrap();
        let mk = |pid: i64, comm: &str, ppid: i64| {
            let p = d.path().join(pid.to_string());
            std::fs::create_dir_all(&p).unwrap();
            std::fs::write(p.join("comm"), format!("{comm}\n")).unwrap();
            std::fs::write(p.join("stat"), format!("{pid} ({comm}) S {ppid} 1 1")).unwrap();
        };
        mk(30, "bash", 20);
        mk(20, "-sh", 10);
        mk(10, "claude (x) ", 1);
        assert_eq!(session_owner(d.path(), 30), 10);
        assert_eq!(session_owner(d.path(), 99), 99);
    }

    #[test]
    fn check_refuses_mixed_modes_and_names_unknown_tools() {
        let known: BTreeSet<String> = ["a", "b"].map(String::from).into();
        let base = CheckArgs {
            update_baseline: None,
            only: None,
            scope: None,
            strict: false,
            arch: None,
            known: &known,
        };
        let p = check_plan(&CheckArgs {
            only: Some(" b,b ,"),
            ..base
        });
        assert_eq!(p["mode"], "partial");
        assert_eq!(p["extra"]["omitted_scanners"], json!(["a"]));
        let p = check_plan(&CheckArgs {
            update_baseline: Some("a,zz"),
            ..base
        });
        assert_eq!(p["lines"][0], "unknown lint tool(s): zz");
        let p = check_plan(&CheckArgs {
            scope: Some("arch"),
            strict: true,
            ..base
        });
        assert_eq!(p["rc"], 2);
        assert_eq!(check_plan(&base)["arch"], "all");
    }
}
