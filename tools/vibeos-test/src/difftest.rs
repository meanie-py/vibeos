//! `vibeos-cli difftest`: which host differential harnesses exist, how each must be built,
//! and whether a run graded anything.
//!
//! The difftest crates compile the GENUINE ReactOS C as a host translation unit (public symbols
//! renamed `ref_*`), link it beside the shipping Rust, and fuzz the two for parity. They are the
//! only oracle that grades a translation against the code it replaced. Two rules, both learned
//! the hard way:
//!
//! * ONE cargo invocation PER crate. cnt-difftest wants cnt-rust's `cabi` feature and
//!   msvcrt-difftest must not have it; built together, feature unification turns it on and 208
//!   `#[no_mangle]` shims collide at link (`duplicate symbol: mbtowc`).
//! * DISCOVER the crates, never list them: a hardcoded list is how a new harness goes ungraded.
//!   Membership is the workspace manifest; a crate's own `[package.metadata.difftest]` declares
//!   its target triple, extra feature sets, or that its plain `#[test]`s are host known-answer
//!   checks.
//!
//! Execution and reporting stay with the facade's `Run` (`tests/qemu/cli/difftest.py`).
use crate::Result;
use fancy_regex::Regex as FancyRegex;
use regex::Regex;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const SUFFIX: &str = "-difftest";

fn ws(repo: &Path) -> PathBuf {
    repo.join("sdk/lib/rust")
}
fn manifest(repo: &Path, name: &str) -> String {
    fs::read_to_string(ws(repo).join(name).join("Cargo.toml")).unwrap_or_default()
}
fn re(p: &str) -> Regex {
    Regex::new(p).expect("static regex")
}

/// The body of a manifest's `[package.metadata.difftest]` table, if it has one.
fn metadata(text: &str) -> Option<String> {
    let mut lines = text.lines();
    lines.find(|l| l.trim() == "[package.metadata.difftest]")?;
    Some(
        lines
            .take_while(|l| !l.trim_start().starts_with('['))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

/// A shared crate whose own `#[test]`s are known-answer checks run here too
/// (`host-tests = true`): win32macros-rust's 22 are what caught RTL_CONSTANT_STRING pointing
/// Buffer at a stack copy, and nothing ran them before.
fn host_tests(repo: &Path, name: &str) -> bool {
    metadata(&manifest(repo, name))
        .is_some_and(|m| re(r"(?m)^\s*host-tests\s*=\s*true\b").is_match(&m))
}

/// Workspace members that are differential harnesses — parsed from `[workspace] members`, not
/// globbed off disk, so a crate never wired into the workspace does not look covered.
pub fn discover(repo: &Path) -> Vec<String> {
    let Ok(text) = fs::read_to_string(ws(repo).join("Cargo.toml")) else {
        return Vec::new();
    };
    let Some(m) = re(r"(?ms)^\s*members\s*=\s*\[(.*?)\]").captures(&text) else {
        return Vec::new();
    };
    re(r#""([^"]+)""#)
        .captures_iter(&m[1])
        .map(|c| c[1].rsplit('/').next().unwrap_or("").to_owned())
        .filter(|n| n.ends_with(SUFFIX) || host_tests(repo, n))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// The `--target` triple a harness declares, or None for the host. msvcrt-seh-difftest grades
/// i386-only SEH, which compiles to nothing on a 64-bit host — so the crate names its triple.
pub fn crate_target(repo: &Path, name: &str) -> Option<String> {
    let m = metadata(&manifest(repo, name))?;
    re(r#"(?m)^\s*target\s*=\s*"([^"]+)""#)
        .captures(&m)
        .map(|c| c[1].to_owned())
}

/// Extra `--features` builds a harness declares, each graded as its own invocation
/// (rtl-sync-difftest builds srw.rs twice: the kernel's copy spins, the user one blocks).
pub fn crate_feature_sets(repo: &Path, name: &str) -> Vec<String> {
    let Some(m) = metadata(&manifest(repo, name)) else {
        return Vec::new();
    };
    re(r"(?m)^\s*feature-sets\s*=\s*\[([^\]]*)\]")
        .captures(&m)
        .map(|c| {
            re(r#""([^"]+)""#)
                .captures_iter(&c[1])
                .map(|x| x[1].to_owned())
                .collect()
        })
        .unwrap_or_default()
}

/// The lines a human needs from a failed harness: the assertion and its two values, not the
/// linker command line.
pub fn salient(out: &str, limit: usize) -> Vec<String> {
    let mut keep = Vec::new();
    for s in out.lines().map(str::trim).filter(|s| !s.is_empty()) {
        if s.starts_with("error")
            || s.contains("panicked at")
            || s.starts_with("assertion")
            || s.starts_with("left:")
            || s.starts_with("right:")
            || s.contains("FAILED")
        {
            keep.push(s.chars().take(200).collect());
        }
        if keep.len() >= limit {
            break;
        }
    }
    keep
}

/// How many cases a harness actually graded, across each crate's dialect. Every crate also
/// prints `test result: ok. 0 passed` for its empty targets, so zero means the harness ran,
/// exited 0 and compared nothing — the vacuous pass, which the caller fails.
pub fn graded(out: &str) -> u64 {
    const PATS: [&str; 7] = [
        r"difftest result: ok\. (\d+) passed",
        r"(\d+) mbcs cases passed",
        r"(\d+) create_mbcinfo codepages passed",
        r"(\d+) printf shell rows passed",
        r"(\d+) wave16 cases passed",
        r"(\d+) scanf cases passed",
        // The libtest form is a literal substring of the difftest form; without the
        // lookbehind cnt's 81 cases would count as 162.
        r"(?<![a-z])test result: ok\. (\d+) passed",
    ];
    PATS.iter()
        .map(|p| {
            FancyRegex::new(p)
                .expect("static regex")
                .captures_iter(out)
                .flatten()
                .filter_map(|c| c.get(1)?.as_str().parse::<u64>().ok())
                .sum::<u64>()
        })
        .sum()
}

/// `(label, argv)` for every generated msvcrt-difftest artifact's `--check`, DISCOVERED: a
/// `refstub/msvcrt_<stem>_protos.h` is its own registration (the set grew 3 -> 7 in two months).
pub fn generated_checks(repo: &Path) -> Vec<(String, Vec<String>)> {
    let codegen = repo.join("tools/codegen").display().to_string();
    let mut checks = vec![(
        "consts".to_owned(),
        vec![codegen.clone(), "msvcrt-consts".into(), "--check".into()],
    )];
    let refstub = ws(repo).join("msvcrt-difftest/refstub");
    let mut stems: Vec<String> = fs::read_dir(&refstub)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let n = e.file_name().to_string_lossy().into_owned();
            Some(
                n.strip_prefix("msvcrt_")?
                    .strip_suffix("_protos.h")?
                    .to_owned(),
            )
        })
        .collect();
    stems.sort();
    for stem in stems {
        checks.push((
            format!("protos:{stem}"),
            vec![
                codegen.clone(),
                "ref-protos".into(),
                format!("{stem}.c"),
                "--check".into(),
            ],
        ));
    }
    // The 32-bit harness's layout guardian: generated from headers this repo owns, so it can
    // go stale with no edit — and a stale SEH layout regrades against offsets msvcrt no longer has.
    if ws(repo)
        .join("msvcrt-seh-difftest/refstub/seh_layout.h")
        .is_file()
    {
        checks.push((
            "layout:seh-i386".into(),
            vec![codegen, "seh-layout".into(), "--check".into()],
        ));
    }
    checks
}

/// The cargo invocations for a run, in crate order: one per crate, plus one per declared
/// feature set (not when grading named cases).
pub fn plan(repo: &Path, crates: &[String], cases: &[String], test_target: &str) -> Vec<Value> {
    let mut builds: Vec<(usize, String, String, Option<String>)> = crates
        .iter()
        .enumerate()
        .map(|(i, c)| (i, c.clone(), c.clone(), None))
        .collect();
    if cases.is_empty() {
        for (i, c) in crates.iter().enumerate() {
            for fs in crate_feature_sets(repo, c) {
                builds.push((i, c.clone(), format!("{c}[{fs}]"), Some(fs)));
            }
        }
    }
    builds.sort_by_key(|b| b.0); // stable: base build, then its feature sets
    builds
        .into_iter()
        .map(|(_, krate, name, features)| {
            let mut argv: Vec<String> = ["cargo", "test", "--release", "-p", &krate]
                .map(String::from)
                .to_vec();
            if let Some(f) = features {
                argv.extend(["--features".into(), f]);
            }
            if let Some(t) = crate_target(repo, &krate) {
                argv.extend(["--target".into(), t]);
            }
            if !cases.is_empty() {
                argv.extend([
                    "--test".into(),
                    test_target.into(),
                    "--".into(),
                    "--exact".into(),
                ]);
                argv.extend(cases.iter().cloned());
            }
            json!({"crate": krate, "name": name, "argv": argv})
        })
        .collect()
}

/// The verdict on one harness invocation: green only if it exited 0 AND graded something AND,
/// when named cases were asked for, graded exactly those.
pub fn verdict(name: &str, out: &str, rc: i64, cases: usize) -> Value {
    let n = graded(out);
    let ok = rc == 0 && n > 0 && (cases == 0 || n == cases as u64);
    let detail = if rc != 0 {
        format!("exit {rc}")
    } else if cases > 0 && n != cases as u64 {
        format!("graded {n} cases, requested exactly {cases}")
    } else if n == 0 {
        "graded NOTHING".into()
    } else {
        format!("{n} cases")
    };
    let salient = if ok {
        Vec::new()
    } else {
        let s = salient(out, 6);
        if s.is_empty() {
            vec![format!("{name}: {detail}")]
        } else {
            s
        }
    };
    json!({"ok": ok, "detail": detail, "salient": salient, "graded": n})
}

fn repo_arg(a: &Value) -> Result<PathBuf> {
    a["repo"]
        .as_str()
        .map(PathBuf::from)
        .ok_or_else(|| "missing repo".into())
}
fn strings(v: &Value) -> Vec<String> {
    serde_json::from_value(v.clone()).unwrap_or_default()
}

pub fn dispatch(request: &Value) -> Result<Value> {
    let a = &request["kwargs"];
    let name = || a["name"].as_str().unwrap_or("").to_owned();
    match request["op"].as_str().unwrap_or("") {
        "difftest_discover" => Ok(json!(discover(&repo_arg(a)?))),
        "difftest_crate_target" => Ok(json!(crate_target(&repo_arg(a)?, &name()))),
        "difftest_crate_feature_sets" => Ok(json!(crate_feature_sets(&repo_arg(a)?, &name()))),
        "difftest_salient" => Ok(json!(salient(
            a["out"].as_str().unwrap_or(""),
            a["limit"].as_u64().unwrap_or(6) as usize
        ))),
        "difftest_graded" => Ok(json!(graded(a["out"].as_str().unwrap_or("")))),
        "difftest_generated_checks" => Ok(json!(generated_checks(&repo_arg(a)?))),
        "difftest_plan" => Ok(json!(plan(
            &repo_arg(a)?,
            &strings(&a["crates"]),
            &strings(&a["cases"]),
            a["test_target"].as_str().unwrap_or("diff"),
        ))),
        "difftest_verdict" => Ok(verdict(
            &name(),
            a["out"].as_str().unwrap_or(""),
            a["rc"].as_i64().unwrap_or(-1),
            a["cases"].as_u64().unwrap_or(0) as usize,
        )),
        op => Err(format!("unknown difftest operation {op}").into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn libtest_counts_are_not_double_counted_inside_the_difftest_form() {
        assert_eq!(graded("difftest result: ok. 81 passed; 0 failed"), 81);
        assert_eq!(
            graded("test result: ok. 0 passed\ntest result: ok. 5 passed\n12 scanf cases passed"),
            17
        );
    }

    #[test]
    fn the_metadata_table_ends_at_the_next_table() {
        let text = "[package]\nname=\"x\"\n[package.metadata.difftest]\ntarget = \"i686\"\n\
                    feature-sets = [\"kernel\", \"other\"]\n[dependencies]\ntarget = \"no\"\n";
        let m = metadata(text).unwrap();
        assert!(m.contains("i686") && !m.contains("\"no\""));
    }

    #[test]
    fn a_vacuous_or_short_run_is_not_green() {
        assert_eq!(
            verdict("x", "test result: ok. 0 passed", 0, 0)["detail"],
            "graded NOTHING"
        );
        let short = verdict("x", "test result: ok. 1 passed", 0, 2);
        assert_eq!(short["ok"], false);
        assert_eq!(
            short["salient"],
            json!(["x: graded 1 cases, requested exactly 2"])
        );
        assert_eq!(verdict("x", "test result: ok. 2 passed", 0, 2)["ok"], true);
        assert_eq!(
            verdict("x", "error: linking failed", 101, 0)["salient"],
            json!(["error: linking failed"])
        );
    }
}
