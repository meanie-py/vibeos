//! Lint stage verdicts: findings vs `tools/lint-baseline/<tool>.txt`.
//!
//! The gate is zero NEW findings against a checked-in baseline, never absolute zero.
//! This module owns what a finding IS (summary tallies are not), when two findings are
//! the SAME one (location-insensitive, multiset), when a scan did not complete, and the
//! stage status each case earns. The Python CLI only runs scanners and records stages.
use regex::Regex;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

pub const BASELINE_DIR: &str = "tools/lint-baseline";

fn re(pattern: &str) -> Regex {
    Regex::new(pattern).expect("static verdict regex")
}

// Count-bearing SUMMARY lines are not findings: they drift whenever a count changes (a
// crate add/remove moves 'TOTAL: ... across N files'; a bindgen regen moves 'Bindgen
// catalog: N functions'), which made every cross-lane merge spuriously LINT_FAIL with
// "1 new". Findings are the per-site lines; tallies are presentation. The
// verify_signatures tallies move by ONE whenever a lane adds a #[no_mangle] fn (bounced
// a batch as "3 new", 2026-07-31). They are enumerated rather than matched by a generic
// '<label>: <number>' rule because this applies to EVERY tool, and a rule broad enough
// to catch any tally is broad enough to silently swallow another scanner's finding.
static SUMMARY_LINE: LazyLock<Regex> = LazyLock::new(|| {
    re(concat!(
        r"(^\s*TOTAL:",
        r"|^\s*SUMMARY:",
        r"|^\s*[A-Z_]+: \d+$",
        r"|catalog: \d+",
        r"|^\s*-> .*: *\d+$",
        r"|^\s*matched to a C array\s*: *\d+$",
        r"|^\s*no C counterpart\s*: *\d+$",
        r"|^\s*Rust integer tables found\s*: *\d+$",
        r"|^\s*Found \d+ unprotected",
        r"|^\s*Total: \d+ violations",
        r"|^\s*C prototypes: \d+",
        r"|^\s*exported Rust fns: \d+",
        r"|^\s*exported fns checked\s*: *\d+$",
        r"|^\s*matched to a C proto\s*: *\d+$",
        r"|^\s*no C proto \(internal\)\s*: *\d+$",
        r"|^\s*signature OK\s*: *\d+$",
        r"|^\s*mismatches\s*: *\d+)",
    ))
});
// Finding identity WITHOUT line/column, so LINE DRIFT is not a new finding: a comment
// inserted above a baselined finding moved it :90 -> :93 and the gate went RED
// (2026-07-25), teaching people to re-bless baselines wholesale, which launders whatever
// else drifted. Keys are compared as a MULTISET, so a genuinely additional occurrence
// still counts.
static LOCATION: LazyLock<Regex> = LazyLock::new(|| re(r"^(?P<file>[^\s:]+):\d+(?::\d+)?:"));
// lint_kernel prints a file header, then `    L 309 Fn(): ...` (or `L1062`, the number
// right-aligned with no space), with the line inside the message.
static EMBEDDED_LINE: LazyLock<Regex> = LazyLock::new(|| re(r"^(?P<indent>\s*)L\s*\d+\s"));
// `[C_MACRO] path:38: msg`: the tag is semantic and stays in the key; only line:col goes.
static TAGGED_LOCATION: LazyLock<Regex> =
    LazyLock::new(|| re(r"^(?P<tags>\s*(?:\[[^\]]+\]\s*)+)(?P<file>[^\s:]+):\d+(?::\d+)?:"));
// verify_signatures: `[MEDIUM] NtSetLdtEntries  (sdk/lib/rust/.../stubs.rs:210)`.
static TRAILING_LOCATION: LazyLock<Regex> =
    LazyLock::new(|| re(r"^(?P<head>.*)\((?P<file>[^\s:()]+):\d+(?::\d+)?\)\s*$"));
// Locations INSIDE the message (scan_dropped_asserts lists Rust twins as
// `(…/section.rs:252, …)`): a 6-line fix there read as "2 new" (2026-09-26).
static INNER_LOCATION: LazyLock<Regex> = LazyLock::new(|| {
    re(
        r"(?P<f>[\w./+-]+\.(?:rs|c|h|cpp|hpp|py|md|txt|cmake|spec|toml|sh|S|s|asm|inc)):\d+(?::\d+)?\b",
    )
});
static CRASH: LazyLock<Regex> =
    LazyLock::new(|| re(r"(?m)^(?:Traceback \(most recent call last\):|FATAL:)"));

/// Python's `str.isspace`, which also counts the ASCII separators U+001C..U+001F.
fn py_space(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}
fn py_strip(s: &str) -> &str {
    s.trim_matches(py_space)
}
fn py_rstrip(s: &str) -> &str {
    s.trim_end_matches(py_space)
}
/// Python's `str.splitlines()`: every Unicode line boundary, `\r\n` as one.
pub fn splitlines(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut chars = s.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if matches!(
            c,
            '\n' | '\r'
                | '\u{0b}'
                | '\u{0c}'
                | '\u{1c}'
                | '\u{1d}'
                | '\u{1e}'
                | '\u{85}'
                | '\u{2028}'
                | '\u{2029}'
        ) {
            out.push(&s[start..i]);
            let mut end = i + c.len_utf8();
            if c == '\r' && chars.peek().is_some_and(|(_, n)| *n == '\n') {
                chars.next();
                end += 1;
            }
            start = end;
        }
    }
    if start < s.len() {
        out.push(&s[start..]);
    }
    out
}

pub fn finding_lines(out: &str) -> BTreeSet<String> {
    splitlines(out)
        .into_iter()
        .filter(|l| !py_strip(l).is_empty() && !SUMMARY_LINE.is_match(l))
        .map(|l| py_rstrip(l).to_owned())
        .collect()
}

/// Crashes outrank partial or baselined output: the scan never completed.
pub fn scanner_error(returncode: i64, stdout: &str, stderr: &str, findings_from: &str) -> bool {
    if returncode < 0 || CRASH.is_match(stderr) {
        return true;
    }
    let text = if findings_from == "stdout" {
        stdout.to_owned()
    } else {
        format!("{stdout}\n{stderr}")
    };
    returncode != 0 && finding_lines(&text).is_empty()
}

fn finding_key_outer(line: &str) -> String {
    if let Some(m) = LOCATION.captures(line) {
        let end = m.get(0).unwrap().end();
        return format!("{}: {}", &m["file"], py_strip(&line[end..]));
    }
    if let Some(m) = TAGGED_LOCATION.captures(line) {
        let end = m.get(0).unwrap().end();
        return format!(
            "{} {}: {}",
            py_strip(&m["tags"]),
            &m["file"],
            py_strip(&line[end..])
        );
    }
    if let Some(m) = TRAILING_LOCATION.captures(line) {
        return format!("{} ({})", py_strip(&m["head"]), &m["file"]);
    }
    if let Some(m) = EMBEDDED_LINE.captures(line) {
        let end = m.get(0).unwrap().end();
        return format!("{}L {}", &m["indent"], py_strip(&line[end..]));
    }
    line.to_owned()
}
pub fn finding_key(line: &str) -> String {
    INNER_LOCATION
        .replace_all(&finding_key_outer(line), "$f")
        .into_owned()
}

/// Current lines whose key the baseline does not cover, keeping their real locations.
/// Surplus occurrences of a baselined key are new too: this is a multiset.
pub fn new_findings(cur: &BTreeSet<String>, base: &BTreeSet<String>) -> Vec<String> {
    let mut budget = BTreeMap::<String, usize>::new();
    for line in base {
        *budget.entry(finding_key(line)).or_default() += 1;
    }
    let mut out = Vec::new();
    for line in cur {
        match budget.get_mut(&finding_key(line)) {
            Some(n) if *n > 0 => *n -= 1,
            _ => out.push(line.clone()),
        }
    }
    out
}

/// Repo-relative output, so a baseline captured in one worktree matches another.
pub fn rel(text: &str, repo: &Path) -> String {
    let root = repo.to_string_lossy();
    text.replace(&format!("{root}/"), "")
        .replace(root.as_ref(), "")
}

pub fn baseline(repo: &Path, tool: &str) -> Result<BTreeSet<String>, String> {
    let path = repo.join(BASELINE_DIR).join(format!("{tool}.txt"));
    match fs::read(&path) {
        Ok(bytes) => {
            let text = String::from_utf8(bytes)
                .map_err(|e| format!("{}: not UTF-8: {e}", path.display()))?;
            Ok(splitlines(&text)
                .into_iter()
                .filter(|l| !py_strip(l).is_empty())
                .map(|l| py_rstrip(l).to_owned())
                .collect())
        }
        Err(_) => Ok(BTreeSet::new()),
    }
}

fn text_of(v: &Value, key: &str) -> String {
    v.get(key).and_then(Value::as_str).unwrap_or("").to_owned()
}
fn stage(status: &str, exit: Option<i64>, detail: &str, salient: Vec<String>) -> Value {
    let mut s = json!({"status":status,"detail":detail,"salient":salient});
    if let Some(e) = exit {
        s["exit"] = json!(e);
    }
    s
}
fn head(lines: Vec<&str>, n: usize) -> Vec<String> {
    lines.into_iter().take(n).map(String::from).collect()
}

/// One lint stage's verdict. `kind` is "run" (a completed scanner with returncode,
/// stdout, stderr), "nocheck" (nothing to lint) or "missing" (tool unavailable).
/// `new_fail` defaults to `strict`: strict then governs only COVERAGE (a missing tool,
/// a scanner exiting without findings), while new_fail governs NEW findings.
/// `by_exit_code` judges by the tool's own exit status, for ARCH-SPECIFIC binary lints
/// that no single arch-agnostic baseline file can describe.
pub fn classify(repo: &Path, item: &Value) -> Result<Value, String> {
    let strict = item["strict"].as_bool().unwrap_or(false);
    let fail_new = item["new_fail"].as_bool().unwrap_or(strict);
    match item["kind"].as_str().unwrap_or("run") {
        "nocheck" => return Ok(stage("skip", None, "no sources to check", vec![])),
        // A missing TOOL is lost coverage, not a pass: the gate fails, a local check warns.
        "missing" => {
            return Ok(stage(
                if strict { "fail" } else { "warn" },
                None,
                "tool unavailable — LINT NOT RUN (no coverage)",
                vec![],
            ))
        }
        "run" => {}
        other => return Err(format!("unknown lint stage kind {other:?}")),
    }
    let name = item["name"].as_str().ok_or("lint stage needs a name")?;
    let rc = item["returncode"]
        .as_i64()
        .ok_or("lint stage needs an integer returncode")?;
    let (stdout, stderr) = (text_of(item, "stdout"), text_of(item, "stderr"));
    let by_exit = item["by_exit_code"].as_bool().unwrap_or(false);
    let from = item["findings_from"].as_str().unwrap_or("stdout");
    if scanner_error(rc, &stdout, &stderr, if by_exit { "both" } else { from }) {
        let shown = if !stderr.is_empty() {
            stderr.clone()
        } else if !stdout.is_empty() {
            stdout.clone()
        } else {
            format!("exit {rc}")
        };
        let shown = rel(&shown, repo);
        let lines = splitlines(&shown);
        let tail = lines[lines.len().saturating_sub(4)..]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let mut s = stage(
            "fail",
            Some(rc),
            "SCANNER_ERROR: scanner did not complete; coverage unavailable",
            tail,
        );
        s["classification"] = json!("SCANNER_ERROR");
        return Ok(s);
    }
    if by_exit {
        if rc == 0 {
            return Ok(stage("ok", Some(0), "", vec![]));
        }
        let text = rel(&format!("{stdout}\n{stderr}"), repo);
        let lines = splitlines(&text);
        let mut salient: Vec<String> = lines
            .iter()
            .filter(|l| l.contains("*** BUG") || l.contains("FAIL"))
            .take(3)
            .map(|s| s.to_string())
            .collect();
        if salient.is_empty() {
            salient = head(lines, 3);
        }
        return Ok(stage(
            if fail_new { "fail" } else { "warn" },
            Some(rc),
            &format!("exit {rc}"),
            salient,
        ));
    }
    let text = if from == "stdout" {
        stdout
    } else {
        format!("{stdout}\n{stderr}")
    };
    let text = rel(&text, repo);
    let cur = finding_lines(&text);
    let new = new_findings(&cur, &baseline(repo, name)?);
    // NEW vs baseline is the gate, not the exit code: most scanners exit nonzero
    // whenever any (possibly baselined) finding exists.
    if !new.is_empty() {
        return Ok(stage(
            if fail_new { "fail" } else { "warn" },
            Some(rc),
            &format!("{} new", new.len()),
            new.into_iter().take(3).collect(),
        ));
    }
    if !cur.is_empty() {
        return Ok(stage("ok", Some(rc), "baselined", vec![]));
    }
    if rc != 0 {
        return Ok(stage(
            if strict { "fail" } else { "warn" },
            Some(rc),
            &format!("exit {rc}"),
            head(splitlines(&text), 3),
        ));
    }
    Ok(stage("ok", Some(0), "", vec![]))
}

/// Rewrite baselines from completed scans. Refuses the WHOLE update, before writing any
/// file, if any scan did not complete: blessing partial output would silently accept
/// whatever the crash hid. Returns {name: {count, added, removed}} so a caller can show
/// exactly what is being blessed.
pub fn capture(repo: &Path, items: &[Value]) -> Result<Value, String> {
    let mut planned = Vec::new();
    for item in items {
        let name = item["name"].as_str().ok_or("baseline item needs a name")?;
        if name.is_empty() || name.contains(['/', '\\']) || name.starts_with('.') {
            return Err(format!("invalid baseline name {name:?}"));
        }
        let rc = item["returncode"]
            .as_i64()
            .ok_or("baseline item needs an integer returncode")?;
        let (stdout, stderr) = (text_of(item, "stdout"), text_of(item, "stderr"));
        let from = item["findings_from"].as_str().unwrap_or("stdout");
        if scanner_error(rc, &stdout, &stderr, from) {
            return Err(format!(
                "SCANNER_ERROR: {name} did not complete; refusing baseline update"
            ));
        }
        let text = if from == "stdout" {
            stdout
        } else {
            format!("{stdout}\n{stderr}")
        };
        planned.push((name.to_owned(), finding_lines(&rel(&text, repo))));
    }
    let dir = repo.join(BASELINE_DIR);
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut out = serde_json::Map::new();
    for (name, cur) in planned {
        let prev = baseline(repo, &name)?;
        let mut body = cur.iter().cloned().collect::<Vec<_>>().join("\n");
        if !cur.is_empty() {
            body.push('\n');
        }
        let path = dir.join(format!("{name}.txt"));
        fs::write(&path, body).map_err(|e| format!("{}: {e}", path.display()))?;
        out.insert(
            name,
            json!({"count":cur.len(),"added":cur.difference(&prev).count(),"removed":prev.difference(&cur).count()}),
        );
    }
    Ok(Value::Object(out))
}

/// JSON request from the CLI: {"op":"classify"|"capture"|"scanner_error"|"finding_lines",
/// "items":[...]}.
pub fn dispatch(repo: &Path, request: &Value) -> Result<Value, String> {
    let items = request["items"]
        .as_array()
        .ok_or("verdict request needs an items array")?;
    match request["op"].as_str().unwrap_or("") {
        "classify" => Ok(Value::Array(
            items
                .iter()
                .map(|i| classify(repo, i))
                .collect::<Result<_, _>>()?,
        )),
        "capture" => capture(repo, items),
        // Finding lines of each repo-relativized text, sorted: for parity tooling.
        "finding_lines" => Ok(Value::Array(
            items
                .iter()
                .map(|i| json!(finding_lines(&rel(i["text"].as_str().unwrap_or(""), repo))))
                .collect(),
        )),
        "scanner_error" => Ok(Value::Array(
            items
                .iter()
                .map(|i| {
                    json!(scanner_error(
                        i["returncode"].as_i64().unwrap_or(0),
                        i["stdout"].as_str().unwrap_or(""),
                        i["stderr"].as_str().unwrap_or(""),
                        i["findings_from"].as_str().unwrap_or("stdout"),
                    ))
                })
                .collect(),
        )),
        other => Err(format!("unknown verdict op {other:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn set(lines: &[&str]) -> BTreeSet<String> {
        lines.iter().map(|s| s.to_string()).collect()
    }
    #[test]
    fn count_summaries_are_not_findings() {
        let got = finding_lines(
            "TOTAL: 160 issues across 2348 files\n  [C_MACRO] sdk/lib/rust/cc-rust/src/cacheman.rs:38: real finding\nC_MACRO: 94\nBindgen catalog: 12 functions\n      - ARITY: rust 0 params vs C 5\n  exported Rust fns: 412\n  mismatches   : 3\n",
        );
        assert_eq!(
            got,
            set(&[
                "  [C_MACRO] sdk/lib/rust/cc-rust/src/cacheman.rs:38: real finding",
                "      - ARITY: rust 0 params vs C 5"
            ])
        );
    }
    #[test]
    fn line_drift_is_not_new_but_an_extra_occurrence_is() {
        let base = set(&["qemu/lib/common.sh:90:17: SC2034 SLOT_INDEX unused"]);
        assert!(new_findings(
            &set(&["qemu/lib/common.sh:93:17: SC2034 SLOT_INDEX unused"]),
            &base
        )
        .is_empty());
        let two = set(&[
            "qemu/lib/common.sh:93:17: SC2034 SLOT_INDEX unused",
            "qemu/lib/common.sh:120:3: SC2034 SLOT_INDEX unused",
        ]);
        assert_eq!(new_findings(&two, &base).len(), 1);
    }
    #[test]
    fn every_location_shape_drops_only_the_line() {
        assert_eq!(
            finding_key("  [C_MACRO] a/b.rs:177: msg"),
            finding_key("  [C_MACRO] a/b.rs:178: msg")
        );
        assert_ne!(
            finding_key("[C_MACRO] a/b.rs:177: msg"),
            finding_key("[FLEX_ARRAY] a/b.rs:177: msg")
        );
        assert_eq!(
            finding_key("[MEDIUM] NtSetLdtEntries  (x/stubs.rs:210)"),
            "[MEDIUM] NtSetLdtEntries (x/stubs.rs)"
        );
        assert_eq!(
            finding_key("    L 309 F(): x"),
            finding_key("    L1062 F(): x")
        );
        assert_eq!(
            finding_key("a.c:1: Rust has 0 (s/section.rs:252, t/shim.rs:1454)"),
            "a.c: Rust has 0 (s/section.rs, t/shim.rs)"
        );
    }
    #[test]
    fn crashes_outrank_findings_and_baselines() {
        const TRACE: &str = "Traceback (most recent call last):\nFileNotFoundError: x\n";
        assert!(scanner_error(1, "", TRACE, "stdout"));
        assert!(scanner_error(1, "file.rs:1: finding\n", TRACE, "stdout"));
        assert!(scanner_error(-9, "file.rs:1: finding\n", "", "stdout"));
        assert!(scanner_error(1, "", "permission denied", "stdout"));
        assert!(scanner_error(0, "", TRACE, "stdout"));
        assert!(!scanner_error(1, "a.rs:1: bug\n", "", "stdout"));
        assert!(!scanner_error(1, "", "a.rs:1: bug", "both"));
    }
    #[test]
    fn stages_follow_coverage_and_new_finding_policy() {
        let dir = tempdir();
        let repo = dir.as_path();
        fs::create_dir_all(repo.join(BASELINE_DIR)).unwrap();
        fs::write(repo.join(BASELINE_DIR).join("t.txt"), "a.rs:1: bug\n").unwrap();
        let run = |rc: i64, out: &str, strict: bool, new_fail: Option<bool>| {
            let mut i =
                json!({"name":"t","returncode":rc,"stdout":out,"stderr":"","strict":strict});
            if let Some(n) = new_fail {
                i["new_fail"] = json!(n);
            }
            classify(repo, &i).unwrap()
        };
        assert_eq!(run(1, "a.rs:9: bug\n", true, None)["detail"], "baselined");
        assert_eq!(run(1, "b.rs:1: bug\n", false, None)["status"], "warn");
        assert_eq!(run(1, "b.rs:1: bug\n", false, Some(true))["status"], "fail");
        assert_eq!(run(0, "", true, None)["status"], "ok");
        let crash = classify(
            repo,
            &json!({"name":"t","returncode":1,"stdout":"a.rs:1: bug","stderr":"FATAL: x"}),
        )
        .unwrap();
        assert_eq!(crash["classification"], "SCANNER_ERROR");
        assert_eq!(crash["status"], "fail");
        let missing = |strict| {
            classify(repo, &json!({"kind":"missing","strict":strict})).unwrap()["status"].clone()
        };
        assert_eq!(
            (missing(true), missing(false)),
            (json!("fail"), json!("warn"))
        );
        let by_exit = classify(repo, &json!({"name":"t","returncode":1,"stdout":"x\n*** BUG here","stderr":"","by_exit_code":true})).unwrap();
        assert_eq!(by_exit["salient"], json!(["*** BUG here"]));
        let mut absolute = json!({"name":"t","returncode":1,"stdout":format!("{}/a.rs:4: bug\n", repo.display()),"stderr":""});
        absolute["strict"] = json!(true);
        assert_eq!(classify(repo, &absolute).unwrap()["detail"], "baselined");
    }
    #[test]
    fn capture_refuses_everything_when_one_scan_crashed() {
        let dir = tempdir();
        let repo = dir.as_path();
        let items = [
            json!({"name":"good","returncode":1,"stdout":"a.rs:1: bug\n","stderr":""}),
            json!({"name":"bad","returncode":1,"stdout":"a.rs:1: bug\n","stderr":"Traceback (most recent call last):\n"}),
        ];
        assert!(capture(repo, &items)
            .unwrap_err()
            .contains("refusing baseline"));
        assert!(!repo.join(BASELINE_DIR).join("good.txt").exists());
        let out = capture(repo, &items[..1]).unwrap();
        assert_eq!(out["good"], json!({"count":1,"added":1,"removed":0}));
        assert_eq!(
            fs::read_to_string(repo.join(BASELINE_DIR).join("good.txt")).unwrap(),
            "a.rs:1: bug\n"
        );
    }
    #[test]
    fn splitlines_matches_python() {
        assert_eq!(
            splitlines("a\r\nb\rc\x0cd\u{2028}e\n"),
            vec!["a", "b", "c", "d", "e"]
        );
        assert_eq!(splitlines("a\n\nb"), vec!["a", "", "b"]);
        assert!(splitlines("").is_empty());
    }
    fn tempdir() -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!(
            "vibeos-verdict-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }
}

/// The baseline identity contract, ported from the Python suite with its incidents.
#[cfg(test)]
mod contract {
    use super::*;
    fn set(lines: &[&str]) -> BTreeSet<String> {
        lines.iter().map(|s| s.to_string()).collect()
    }
    fn keys(text: &str) -> BTreeSet<String> {
        finding_lines(text).iter().map(|l| finding_key(l)).collect()
    }
    #[test]
    fn a_new_file_or_message_is_new() {
        let base = set(&["a.sh:10:1: warning: X unused. [SC2034]"]);
        assert!(!new_findings(&set(&["b.sh:10:1: warning: X unused. [SC2034]"]), &base).is_empty());
        assert!(!new_findings(&set(&["a.sh:10:1: warning: Y unused. [SC2034]"]), &base).is_empty());
    }
    #[test]
    fn lint_kernel_embedded_lines_drift_but_extra_sites_count() {
        let base = set(&["    L 309 RtlpCheckDeviceName(): *NameInvalid = ..."]);
        assert!(new_findings(
            &set(&["    L 313 RtlpCheckDeviceName(): *NameInvalid = ..."]),
            &base
        )
        .is_empty());
        let cur = set(&[
            "    L 313 RtlpCheckDeviceName(): *NameInvalid = ...",
            "    L 480 RtlpCheckDeviceName(): *NameInvalid = ...",
        ]);
        assert_eq!(new_findings(&cur, &base).len(), 1);
        assert!(
            !new_findings(&set(&["    L 313 RtlpOther(): *NameInvalid = ..."]), &base).is_empty()
        );
    }
    #[test]
    fn tagged_locations_drift_and_tags_are_identity() {
        let base = set(&["  [C_MACRO] a/b.rs:177: IoGetCurrentIrpStackLocation: is a C macro"]);
        let cur = set(&["  [C_MACRO] a/b.rs:178: IoGetCurrentIrpStackLocation: is a C macro"]);
        assert!(new_findings(&cur, &base).is_empty());
        let other = set(&["  [FLEX_ARRAY] a/b.rs:177: IoGetCurrentIrpStackLocation: is a C macro"]);
        assert!(!new_findings(&other, &base).is_empty());
        let mut both = cur.clone();
        both.insert("  [C_MACRO] a/b.rs:900: IoGetCurrentIrpStackLocation: is a C macro".into());
        assert_eq!(new_findings(&both, &base).len(), 1);
    }
    #[test]
    fn trailing_parenthesized_locations_drift() {
        let base = set(&["[MEDIUM] NtSetLdtEntries  (sdk/lib/rust/x/stubs.rs:210)"]);
        assert!(new_findings(
            &set(&["[MEDIUM] NtSetLdtEntries  (sdk/lib/rust/x/stubs.rs:255)"]),
            &base
        )
        .is_empty());
        assert!(!new_findings(
            &set(&["[MEDIUM] NtOther  (sdk/lib/rust/x/stubs.rs:210)"]),
            &base
        )
        .is_empty());
        assert!(!new_findings(
            &set(&["[HIGH] NtSetLdtEntries  (sdk/lib/rust/x/stubs.rs:210)"]),
            &base
        )
        .is_empty());
    }
    #[test]
    fn verify_signatures_tallies_never_drift() {
        // Adding ONE #[no_mangle] fn bounced a batch as LINT_FAIL "3 new" (2026-07-31).
        let before = "  C prototypes: 5034 FunctionDecls, 4536 typedefs\n  exported Rust fns: 3359 (deduped twins)\n\n── coverage ──\n  exported fns checked : 3359\n  matched to a C proto : 2238\n  no C proto (internal): 1121\n  signature OK         : 2229\n  mismatches           : 9  (3 shown, 6 LOW-suppressed)\n";
        let after = before.replace("3359", "3360").replace("1121", "1122");
        assert!(new_findings(&keys(&after), &keys(before)).is_empty());
        assert_eq!(keys(before), set(&["── coverage ──"]));
    }
    #[test]
    fn the_tally_filter_keeps_real_signature_findings() {
        let finding = "[HIGH] KiThreadStartup  (sdk/lib/rust/x/thrdini.rs:98)\n    rust: fn KiThreadStartup() -> ()\n    C   : void KiThreadStartup(PKSYSTEM_ROUTINE, PKSTART_ROUTINE, PVOID, BOOLEAN, KTRAP_FRAME)\n      - ARITY: rust 0 params vs C 5\n      - param#2 u32 vs ULONG_PTR(unsigned long long): width 4 vs 8 [MEDIUM]\n";
        let kept = keys(finding);
        assert_eq!(kept.len(), 5, "{kept:?}");
        assert!(!new_findings(&kept, &BTreeSet::new()).is_empty());
    }
    #[test]
    fn locations_inside_the_message_drift() {
        let base = set(&[
            "x/miarm.h:1033: MI_X: C has 3 assert(s), Rust has 0 (r/section.rs:252, r/s.rs:14)",
        ]);
        assert!(new_findings(
            &set(&[
                "x/miarm.h:1033: MI_X: C has 3 assert(s), Rust has 0 (r/section.rs:258, r/s.rs:14)"
            ]),
            &base
        )
        .is_empty());
        assert!(!new_findings(
            &set(&[
                "x/miarm.h:1033: MI_X: C has 3 assert(s), Rust has 0 (r/other.rs:252, r/s.rs:14)"
            ]),
            &base
        )
        .is_empty());
    }
    #[test]
    fn prefixless_findings_compare_verbatim() {
        let base = set(&["MISMATCH: NtFoo stdcall vs cdecl"]);
        assert!(new_findings(&set(&["MISMATCH: NtFoo stdcall vs cdecl"]), &base).is_empty());
        assert!(!new_findings(&set(&["MISMATCH: NtBar stdcall vs cdecl"]), &base).is_empty());
    }
    /// The GUARD over every baseline we ship: perturbing any location's line number must
    /// never change a finding's key. A scanner whose output shape this cannot parse fails
    /// here, where its author is, instead of in a stranger's merge.
    #[test]
    fn every_shipped_baseline_line_is_immune_to_line_drift() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../lint-baseline");
        let loc =
            Regex::new(r"(?P<f>[\w./+-]+\.(?:rs|c|h|py|md|txt|cmake|spec|toml|sh)):(?P<n>\d+)")
                .unwrap();
        let mut files: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "txt"))
            .collect();
        files.sort();
        assert!(
            !files.is_empty(),
            "no lint baselines under {}",
            dir.display()
        );
        let mut blind = Vec::new();
        for path in files {
            for line in splitlines(&fs::read_to_string(&path).unwrap()) {
                for m in loc.captures_iter(line) {
                    let n = m.name("n").unwrap();
                    let bumped = (n.as_str().parse::<u64>().unwrap() + 1000).to_string();
                    let drifted = format!("{}{}{}", &line[..n.start()], bumped, &line[n.end()..]);
                    if finding_key(line) != finding_key(&drifted) {
                        blind.push(format!(
                            "{}: {}",
                            path.file_name().unwrap().to_string_lossy(),
                            line.chars().take(110).collect::<String>()
                        ));
                        break;
                    }
                }
            }
        }
        assert!(
            blind.is_empty(),
            "{} baseline finding(s) are not drift-immune:\n  {}",
            blind.len(),
            blind[..blind.len().min(10)].join("\n  ")
        );
    }
}
