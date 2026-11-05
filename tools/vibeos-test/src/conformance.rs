//! Serial sweep parsers and assertion-identity ratchet. Never infers an unobserved pass.
use crate::verdict::{args, kernel_asserts, py_lines, re, string};
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

pub const BAD: &[&str] = &[
    "CRASH",
    "TIMEOUT",
    "FAILSTART",
    "NO-END",
    "INCOMPLETE",
    "ENUM-TIMEOUT",
];
pub fn is_bad(s: Option<&str>) -> bool {
    s.is_some_and(|s| BAD.contains(&s))
}
fn set_or_null<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeSet<String>, D::Error> {
    Ok(Option::<BTreeSet<String>>::deserialize(d)?.unwrap_or_default())
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub status: Option<String>,
    #[serde(default)]
    pub pass: u64,
    #[serde(default)]
    pub fail: u64,
    #[serde(default)]
    pub skip: u64,
    #[serde(default)]
    pub exit: Option<i64>,
    #[serde(default)]
    pub errors: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_seen: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub todo: Option<u64>,
    #[serde(default, deserialize_with = "set_or_null")]
    pub asserts: BTreeSet<String>,
}
impl TestResult {
    fn new(name: &str, km: bool) -> Self {
        Self {
            name: name.into(),
            status: if km { None } else { Some("NO-END".into()) },
            pass: 0,
            fail: 0,
            skip: 0,
            exit: None,
            errors: 0,
            end_seen: if km { Some(false) } else { None },
            todo: if km { None } else { Some(0) },
            asserts: BTreeSet::new(),
        }
    }
}
fn launch(line: &str) -> Option<String> {
    re(r"SWEEP\] launch ([A-Za-z0-9_]+)")
        .captures(line)
        .map(|c| c[1].to_owned())
        .filter(|s| !s.starts_with("FAILED"))
}
fn begin(line: &str) -> Option<String> {
    re(r"KMTEST:BEGIN ([A-Za-z0-9_]+)\]")
        .captures(line)
        .map(|c| c[1].to_owned())
}
const END: &str = r"KMTEST:END (\S+) pass=(\d+) fail=(\d+) skip=(\d+)\]";
pub fn splice_split_end_markers(lines: &[String]) -> Vec<String> {
    let mut out = lines.to_vec();
    let mut launched: Option<String> = None;
    let end = re(END);
    let timestamp = re(r"^\[\+\s*\d+(?:\.\d+)?\]\s?");
    let prefix = "KMTEST:END ";
    let mut i = 0;
    while i < out.len() {
        let line = out[i].clone();
        if let Some(name) = launch(&line).or_else(|| begin(&line)) {
            launched = Some(name);
        }
        let Some(at) = line.find(prefix) else {
            i += 1;
            continue;
        };
        if end.is_match(&line) {
            i += 1;
            continue;
        }
        let frag = &line[at..];
        let cuts: Vec<usize> = if launched.is_some() {
            let mut c: Vec<_> = frag
                .char_indices()
                .map(|(i, _)| i)
                .filter(|i| *i >= prefix.len())
                .collect();
            c.push(frag.len());
            c.reverse();
            c
        } else {
            vec![frag.find('[').filter(|i| *i > 0).unwrap_or(frag.len())]
        };
        for k in i + 1..(i + 4).min(out.len()) {
            let body = timestamp.replace(&out[k], "").into_owned();
            let mut hit = None;
            for cut in &cuts {
                let merged = format!("{}{}", &frag[..*cut], body);
                if let Some(c) = end
                    .captures(&merged)
                    .filter(|c| c.get(0).unwrap().start() == 0)
                {
                    let m = c.get(0).unwrap();
                    if m.end() > *cut && launched.as_deref().is_none_or(|n| n == &c[1]) {
                        hit = Some((m.as_str().to_owned(), body[m.end() - *cut..].to_owned()));
                        break;
                    }
                }
            }
            if let Some((marker, rest)) = hit {
                let mut replacement = vec![marker];
                if !rest.trim().is_empty() {
                    replacement.push(rest);
                }
                out.splice(k..k + 1, replacement);
                break;
            }
        }
        i += 1;
    }
    out
}
fn close_incomplete(results: &mut [TestResult], current: Option<usize>) {
    if let Some(i) = current {
        if results[i].status.is_none() {
            results[i].status = Some("INCOMPLETE".into());
        }
    }
}
pub fn parse_kmtest(text: &str) -> Result<(Vec<TestResult>, Option<String>, Option<String>)> {
    let mut results: Vec<TestResult> = Vec::new();
    let mut current: Option<usize> = None;
    let mut done = None;
    let mut bugcheck = None;
    let end = re(END);
    let failed = re(r"SWEEP\] launch FAILED ([A-Za-z0-9_]+)");
    let assert = re(r"([A-Za-z][A-Za-z0-9_]*\.c(?:pp)?:\d+): Test failed:");
    let exit = re(r"SWEEP\] exit ([A-Za-z0-9_]+) code=(\d+)");
    let timeout = re(r"SWEEP\] TIMEOUT ([A-Za-z0-9_]+)");
    let done_re = re(r"SWEEP\] done (.*)");
    let bug = re(r"\*\*\* BUGCHECK: (0x[0-9A-Fa-f]+)");
    for line in splice_split_end_markers(&py_lines(text)) {
        let new = launch(&line)
            .or_else(|| begin(&line).filter(|n| current.is_none_or(|i| results[i].name != *n)));
        if let Some(name) = new {
            close_incomplete(&mut results, current);
            current = Some(results.len());
            results.push(TestResult::new(&name, true));
            continue;
        }
        if let Some(c) = failed.captures(&line) {
            let mut row = TestResult::new(&c[1], true);
            row.status = Some("FAILSTART".into());
            results.push(row);
            current = None;
            continue;
        }
        if let (Some(c), Some(i)) = (end.captures(&line), current) {
            if c[1] == results[i].name {
                results[i].pass = c[2]
                    .parse()
                    .map_err(|_| Error::new("conformance count exceeds supported u64 range"))?;
                results[i].fail = c[3]
                    .parse()
                    .map_err(|_| Error::new("conformance count exceeds supported u64 range"))?;
                results[i].skip = c[4]
                    .parse()
                    .map_err(|_| Error::new("conformance count exceeds supported u64 range"))?;
                results[i].end_seen = Some(true);
                continue;
            }
        }
        let hits: Vec<_> = assert
            .captures_iter(&line)
            .filter(|c| {
                let at = c.get(1).unwrap().start();
                line[..at]
                    .chars()
                    .next_back()
                    .is_none_or(|p| !(p.is_ascii_alphanumeric() || p == '_'))
            })
            .map(|c| c[1].to_owned())
            .collect();
        if let Some(i) = current {
            if !hits.is_empty() {
                results[i].asserts.extend(hits);
                continue;
            }
            if line.contains("[KMTEST:ERROR]") {
                results[i].errors += 1;
                continue;
            }
        }
        if let (Some(c), Some(i)) = (exit.captures(&line), current) {
            if c[1] == results[i].name {
                let code = c[2]
                    .parse()
                    .map_err(|_| Error::new("conformance exit code exceeds supported i64 range"))?;
                results[i].exit = Some(code);
                results[i].status = Some(
                    if results[i].end_seen == Some(true) {
                        if code == 0 {
                            "OK"
                        } else {
                            "FAIL-EXIT"
                        }
                    } else {
                        "NO-END"
                    }
                    .into(),
                );
                current = None;
                continue;
            }
        }
        if let (Some(c), Some(i)) = (timeout.captures(&line), current) {
            if c[1] == results[i].name {
                results[i].status = Some("TIMEOUT".into());
                current = None;
                continue;
            }
        }
        if let Some(c) = done_re.captures(&line) {
            done = Some(c[1].to_owned());
            break;
        }
        if let Some(c) = bug.captures(&line) {
            if bugcheck.is_none() {
                bugcheck = Some(c[1].to_owned());
            }
            if let Some(i) = current {
                if results[i].status.is_none() {
                    results[i].status = Some("CRASH".into());
                    current = None;
                }
            }
        }
    }
    close_incomplete(&mut results, current);
    fn rank(s: Option<&str>) -> u8 {
        match s {
            Some("OK") => 6,
            Some("FAIL-EXIT") => 5,
            Some("NO-END") => 4,
            Some("TIMEOUT" | "CRASH") => 3,
            Some("FAILSTART") => 2,
            Some("INCOMPLETE") => 1,
            _ => 0,
        }
    }
    let mut unique: Vec<TestResult> = Vec::new();
    let mut by_name: BTreeMap<String, usize> = BTreeMap::new();
    for r in results {
        if let Some(i) = by_name.get(&r.name).copied() {
            if rank(r.status.as_deref()) > rank(unique[i].status.as_deref()) {
                unique[i] = r;
            }
        } else {
            by_name.insert(r.name.clone(), unique.len());
            unique.push(r);
        }
    }
    Ok((unique, done, bugcheck))
}
fn api_row<'a>(
    rows: &'a mut Vec<TestResult>,
    index: &mut BTreeMap<String, usize>,
    name: &str,
) -> &'a mut TestResult {
    let i = if let Some(i) = index.get(name) {
        *i
    } else {
        let i = rows.len();
        rows.push(TestResult::new(name, false));
        index.insert(name.into(), i);
        i
    };
    &mut rows[i]
}
pub fn parse_apitest(text: &str) -> Result<(Vec<TestResult>, Value)> {
    let mut rows = Vec::new();
    let mut index = BTreeMap::new();
    let mut done = None::<String>;
    let mut bugcheck = None::<String>;
    let mut suites = Map::new();
    let mut nosubtests = Map::new();
    let mut deleted = BTreeSet::new();
    let enumerate = re(r"\[APITEST-SWEEP\] enumerate (\S+)");
    let suite = re(r"\[APITEST-SWEEP\] suite (\S+) subtests=(\d+)");
    let no = re(r"\[APITEST-SWEEP\] NOSUBTESTS (\S+) \(exit (-?\d+)\)(.*)");
    let skip = re(r"\[APITEST-SWEEP\] SKIP (\S+) (\S+)");
    let launch_re = re(r"\[APITEST-SWEEP\] launch (\S+) (\S+)");
    let fail = re(r"\[APITEST-SWEEP\] launch FAILED (\S+)");
    let enum_timeout =
        re(r"\[APITEST-SWEEP\] (?:ENUM-TIMEOUT (\S+)|TIMEOUT (\S+) during enumeration)");
    let timeout = re(r"\[APITEST-SWEEP\] TIMEOUT (\S+) (\S+) \(\d+s\) - terminated");
    let exit = re(r"\[APITEST-SWEEP\] exit (\S+) (\S+) code=(-?\d+)");
    let child = re(r"\[APITEST:(\S+) (\S+)\] (.*)");
    let summary = re(
        r"^\S+: (\d+) tests? executed \((\d+) marked as todo, (\d+) failures?\), (\d+) skipped\.",
    );
    let assertion = re(r"^(\S+\.(?:c|cpp|h):\d+): Test failed:");
    let done_re = re(r"\[APITEST-SWEEP\] done (.*)");
    let bug = re(r"\*\*\* BUGCHECK: (0x[0-9A-Fa-f]+(?:\s*\([^)]*\))?)");
    for line in py_lines(text) {
        if let Some(c) = enumerate.captures(&line) {
            let name = format!("{} <enumeration>", &c[1]);
            api_row(&mut rows, &mut index, &name);
            continue;
        }
        if let Some(c) = suite.captures(&line) {
            suites.insert(
                c[1].to_owned(),
                json!(c[2]
                    .parse::<u64>()
                    .map_err(|_| Error::new("conformance count exceeds supported u64 range"))?),
            );
            let name = format!("{} <enumeration>", &c[1]);
            if let Some(i) = index.remove(&name) {
                deleted.insert(i);
            }
            continue;
        }
        if let Some(c) = no.captures(&line) {
            nosubtests.insert(
                c[1].to_owned(),
                json!([
                    c[2].parse::<i64>().map_err(|_| Error::new(
                        "conformance exit code exceeds supported i64 range"
                    ))?,
                    c[3].trim()
                ]),
            );
            let name = format!("{} <enumeration>", &c[1]);
            if let Some(i) = index.remove(&name) {
                deleted.insert(i);
            }
            continue;
        }
        if let Some(c) = skip.captures(&line) {
            let name = format!("{} {}", &c[1], &c[2]);
            api_row(&mut rows, &mut index, &name).status = Some("SKIP".into());
            continue;
        }
        if let Some(c) = launch_re
            .captures(&line)
            .filter(|c| !c[1].starts_with("FAILED"))
        {
            let name = format!("{} {}", &c[1], &c[2]);
            api_row(&mut rows, &mut index, &name);
            continue;
        }
        if let Some(c) = fail.captures(&line) {
            api_row(&mut rows, &mut index, &c[1]).status = Some("FAILSTART".into());
            continue;
        }
        if let Some(c) = enum_timeout.captures(&line) {
            let name = format!(
                "{} <enumeration>",
                c.get(1).or_else(|| c.get(2)).unwrap().as_str()
            );
            api_row(&mut rows, &mut index, &name).status = Some("ENUM-TIMEOUT".into());
            continue;
        }
        if let Some(c) = timeout.captures(&line) {
            let name = format!("{} {}", &c[1], &c[2]);
            api_row(&mut rows, &mut index, &name).status = Some("TIMEOUT".into());
            continue;
        }
        if let Some(c) = exit.captures(&line) {
            let name = format!("{} {}", &c[1], &c[2]);
            let r = api_row(&mut rows, &mut index, &name);
            let code = c[3]
                .parse()
                .map_err(|_| Error::new("conformance exit code exceeds supported i64 range"))?;
            r.exit = Some(code);
            if r.status.as_deref() != Some("NO-END") {
                r.status = Some(if code == 0 { "OK" } else { "FAIL-EXIT" }.into());
            }
            continue;
        }
        if let Some(c) = child.captures(&line) {
            let name = format!("{} {}", &c[1], &c[2]);
            let r = api_row(&mut rows, &mut index, &name);
            let body = &c[3];
            if let Some(s) = summary.captures(body) {
                let executed = s[1]
                    .parse::<u64>()
                    .map_err(|_| Error::new("conformance count exceeds supported u64 range"))?;
                r.todo =
                    Some(s[2].parse().map_err(|_| {
                        Error::new("conformance count exceeds supported u64 range")
                    })?);
                r.fail = s[3]
                    .parse()
                    .map_err(|_| Error::new("conformance count exceeds supported u64 range"))?;
                r.skip = s[4]
                    .parse()
                    .map_err(|_| Error::new("conformance count exceeds supported u64 range"))?;
                r.pass = executed.saturating_sub(r.fail);
                r.status = Some(
                    if r.exit.unwrap_or(0) == 0 {
                        "OK"
                    } else {
                        "FAIL-EXIT"
                    }
                    .into(),
                );
                continue;
            }
            if let Some(c) = assertion.captures(body) {
                r.asserts.insert(c[1].to_owned());
            }
            continue;
        }
        if let Some(c) = done_re.captures(&line) {
            done = Some(c[1].to_owned());
            continue;
        }
        if let Some(c) = bug.captures(&line) {
            bugcheck = Some(c[1].to_owned());
            for (i, r) in rows.iter_mut().enumerate().rev() {
                if !deleted.contains(&i)
                    && r.exit.is_none()
                    && !matches!(
                        r.status.as_deref(),
                        Some("SKIP" | "TIMEOUT" | "ENUM-TIMEOUT" | "FAILSTART")
                    )
                {
                    r.status = Some("CRASH".into());
                    break;
                }
            }
        }
    }
    Ok((
        rows.into_iter()
            .enumerate()
            .filter(|(i, _)| !deleted.contains(i))
            .map(|(_, r)| r)
            .collect(),
        json!({"done":done,"bugcheck":bugcheck,"suites":suites,"nosubtests":nosubtests}),
    ))
}
pub fn fmt(r: &TestResult, flaky: &BTreeSet<String>) -> String {
    let mut text = format!(
        "{} {} pass={} fail={} skip={}",
        r.name,
        r.status.as_deref().unwrap_or("None"),
        r.pass,
        r.fail,
        r.skip
    );
    if let Some(exit) = r.exit {
        text += &format!(" exit={exit}");
    }
    if r.errors != 0 {
        text += &format!(" errors={}", r.errors);
    }
    for a in &r.asserts {
        text += &format!(
            "\n    assert{} {a}",
            if flaky.contains(a) { "?" } else { "" }
        );
    }
    for a in flaky.difference(&r.asserts) {
        text += &format!("\n    assert? {a}");
    }
    text
}
fn row_parts(line: &str) -> Result<(String, String, Vec<(String, i64)>)> {
    let parts: Vec<_> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err("invalid conformance baseline row".into());
    }
    let status_index = parts
        .iter()
        .position(|p| p.contains('='))
        .map(|n| n.saturating_sub(1))
        .unwrap_or(parts.len() - 1);
    if status_index == 0 {
        return Err("baseline row has no test name".into());
    }
    let name = parts[..status_index].join(" ");
    let status = parts[status_index].to_owned();
    let mut counts = Vec::new();
    for kv in &parts[status_index + 1..] {
        let (key, value) = kv.split_once('=').ok_or("invalid baseline count")?;
        if value.contains('=') {
            return Err("invalid baseline count".into());
        }
        counts.push((
            key.to_owned(),
            value
                .parse()
                .map_err(|_| Error::new(format!("invalid baseline count {kv}")))?,
        ));
    }
    Ok((name, status, counts))
}
pub fn baseline_entries(text: &str) -> Result<Vec<(String, Value)>> {
    let mut entries: Vec<(String, Value)> = Vec::new();
    let mut current: Option<usize> = None;
    for raw in py_lines(text) {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let words: Vec<_> = line.splitn(2, char::is_whitespace).collect();
        if raw.chars().next().is_some_and(char::is_whitespace)
            && matches!(words.first().copied(), Some("assert" | "assert?"))
        {
            if let Some(i) = current {
                let ident = words
                    .get(1)
                    .ok_or("baseline assertion has no identity")?
                    .trim();
                let key = words[0];
                let m = entries[i].1.as_object_mut().unwrap();
                for field in if key == "assert?" {
                    &["asserts", "flaky"][..]
                } else {
                    &["asserts"][..]
                } {
                    let mut ids: BTreeSet<String> = m
                        .get(*field)
                        .cloned()
                        .map(serde_json::from_value)
                        .transpose()?
                        .unwrap_or_default();
                    ids.insert(ident.into());
                    m.insert((*field).into(), json!(ids));
                }
            }
            continue;
        }
        let (name, status, counts) = row_parts(line)?;
        let mut value = Map::new();
        value.insert("status".into(), json!(status));
        for (k, v) in counts {
            value.insert(k, json!(v));
        }
        if let Some(i) = entries.iter().position(|(n, _)| n == &name) {
            entries[i].1 = Value::Object(value);
            current = Some(i);
        } else {
            current = Some(entries.len());
            entries.push((name, Value::Object(value)));
        }
    }
    Ok(entries)
}
pub fn load_baseline_text(text: &str) -> Result<Value> {
    let mut map = Map::new();
    for (n, v) in baseline_entries(text)? {
        map.insert(n, v);
    }
    Ok(Value::Object(map))
}
fn number(v: &Value, k: &str) -> i64 {
    v.get(k).and_then(Value::as_i64).unwrap_or(0)
}
fn ids(v: &Value, k: &str) -> BTreeSet<String> {
    v.get(k)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}
pub fn truncated_identity(ident: &str, known: &BTreeSet<String>) -> bool {
    known.iter().any(|k| k != ident && k.ends_with(ident))
}
pub fn compare(
    results: &[TestResult],
    base: &Value,
    base_order: Option<&[Value]>,
) -> Result<Vec<String>> {
    let map = base.as_object().ok_or("baseline must be an object")?;
    let mut regressions = Vec::new();
    for r in results {
        let Some(b) = map.get(&r.name) else {
            continue;
        };
        let bs = b
            .get("status")
            .and_then(Value::as_str)
            .ok_or("baseline row requires status")?;
        let status = r.status.as_deref();
        if is_bad(status) && !is_bad(Some(bs)) {
            regressions.push(format!("{}: {bs} -> {}", r.name, status.unwrap_or("None")));
            continue;
        }
        if !is_bad(status) && !is_bad(Some(bs)) {
            if (r.pass as i128) < number(b, "pass") as i128 {
                regressions.push(format!(
                    "{}: pass {} -> {} (dropped)",
                    r.name,
                    number(b, "pass"),
                    r.pass
                ));
            }
            if b.get("asserts").is_none() {
                if (r.fail as i128) > number(b, "fail") as i128 {
                    regressions.push(format!(
                        "{}: fail {} -> {} (grew)",
                        r.name,
                        number(b, "fail"),
                        r.fail
                    ));
                }
            } else {
                let known = ids(b, "asserts");
                for a in r.asserts.difference(&known) {
                    if !truncated_identity(a, &known) {
                        regressions.push(format!("{}: NEW failing assertion {a}", r.name));
                    }
                }
            }
        }
    }
    let ran: BTreeSet<_> = results.iter().map(|r| r.name.as_str()).collect();
    let names: Vec<_> = base_order
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_else(|| map.keys().map(String::as_str).collect());
    for name in names {
        if !ran.contains(name) {
            if let Some(b) = map.get(name) {
                regressions.push(format!(
                    "{name}: in baseline ({}) but never launched",
                    b["status"].as_str().unwrap_or("None")
                ));
            }
        }
    }
    Ok(regressions)
}
pub fn with_quiet_token(text: &str) -> String {
    if text
        .replace(';', " ")
        .split_whitespace()
        .any(|t| t == "QUIET")
    {
        text.into()
    } else {
        format!("{text} QUIET").trim().into()
    }
}
pub fn strip_host_timestamps(raw: &str) -> String {
    let pattern = re(r"^\[\+ *\d+\.\d+\] ");
    let mut out = String::new();
    let mut at = 0;
    let mut bol = true;
    while at < raw.len() {
        if bol {
            if let Some(m) = pattern.find(&raw[at..]) {
                at += m.end();
                if at >= raw.len() {
                    break;
                }
            }
        }
        let c = raw[at..].chars().next().unwrap();
        out.push(c);
        bol = matches!(c, '\n' | '\r');
        at += c.len_utf8();
    }
    out.replace("\r\n", "\n").replace('\r', "\n")
}
pub fn baseline_coverage_refusal(
    results: &[TestResult],
    base: &Value,
    force: bool,
) -> Option<String> {
    let names: BTreeSet<_> = results.iter().map(|r| r.name.as_str()).collect();
    let dropped: Vec<_> = base
        .as_object()?
        .keys()
        .filter(|n| !names.contains(n.as_str()))
        .collect();
    if dropped.is_empty() || force {
        return None;
    }
    Some(format!("[conformance] --write-baseline REFUSED: this sweep launched {} tests but the existing baseline has {}, and writing it would DELETE {} test(s) from the ratchet floor:\n{}  A short sweep is usually a boot flake or an early BUGCHECK, not a real coverage change — re-run it.\n  If the removal is intended (a test was retired), pass --force-baseline.",results.len(),base.as_object()?.len(),dropped.len(),dropped.iter().map(|n|format!("    {n}\n")).collect::<String>()))
}
fn py_value(v: Option<&Value>) -> String {
    match v {
        None | Some(Value::Null) => "None".into(),
        Some(Value::String(s)) => s.clone(),
        Some(v) => v.to_string(),
    }
}
pub fn baseline_lowering_note(results: &[TestResult], base: &Value) -> Option<String> {
    let mut worse = Vec::new();
    for r in results {
        if let Some(b) = base
            .get(&r.name)
            .filter(|b| b.as_object().is_none_or(|o| !o.is_empty()))
        {
            if r.fail as i128 > number(b, "fail") as i128
                || (r.pass as i128) < number(b, "pass") as i128
            {
                worse.push(format!(
                    "    {}: pass {} -> {}, fail {} -> {}",
                    r.name,
                    py_value(b.get("pass")),
                    r.pass,
                    py_value(b.get("fail")),
                    r.fail
                ));
            }
        }
    }
    for r in results {
        let Some(b) = base
            .get(&r.name)
            .filter(|b| b.get("asserts").is_some_and(|v| !v.is_null()))
        else {
            continue;
        };
        let known = ids(b, "asserts");
        let new: Vec<_> = r.asserts.difference(&known).cloned().collect();
        if !new.is_empty() {
            worse.push(format!(
                "    {}: absorbing {} newly-failing assertion(s): {}",
                r.name,
                new.len(),
                new.join(", ")
            ));
        }
    }
    if worse.is_empty() {
        None
    } else {
        Some(format!("[conformance] NOTE: --write-baseline is LOWERING the floor for {} test(s) — accepting these as the new normal:\n{}",worse.len(),worse.join("\n")))
    }
}
pub fn flaky_of(text: &str, name: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut in_row = false;
    for raw in py_lines(text) {
        let s = raw.trim();
        if raw.chars().next().is_some_and(char::is_whitespace) && s.starts_with("assert") {
            if let Some((kind, ident)) = s.split_once(' ') {
                if in_row && kind == "assert?" {
                    out.insert(ident.trim().into());
                }
            }
            continue;
        }
        if s.is_empty() || s.starts_with('#') {
            continue;
        }
        in_row = row_parts(s).map(|r| r.0 == name).unwrap_or(false);
    }
    out
}
pub fn seed_asserts(text: &str, rounds: &[Vec<TestResult>], bad: &[String]) -> (String, String) {
    let mut seen: BTreeMap<String, Vec<BTreeSet<String>>> = BTreeMap::new();
    for round in rounds {
        for r in round {
            if r.status.as_ref().is_some_and(|s| bad.contains(s)) {
                continue;
            }
            seen.entry(r.name.clone())
                .or_default()
                .push(r.asserts.clone());
        }
    }
    let (mut stable, mut flaky, mut count) = (0, 0, 0);
    let mut out = Vec::new();
    let mut unseen = Vec::new();
    for line in py_lines(text) {
        let s = line.trim();
        if line.chars().next().is_some_and(char::is_whitespace) && s.starts_with("assert") {
            continue;
        }
        out.push(line.clone());
        if s.is_empty() || s.starts_with('#') {
            continue;
        }
        let name = row_parts(s)
            .map(|r| r.0)
            .unwrap_or_else(|_| s.split_whitespace().next().unwrap_or("").into());
        let Some(obs) = seen.get(&name) else {
            unseen.push(name);
            continue;
        };
        count += 1;
        let union: BTreeSet<_> = obs.iter().flat_map(|s| s.iter().cloned()).collect();
        for id in union {
            let unanimous = obs.iter().all(|s| s.contains(&id));
            out.push(format!(
                "    assert{} {id}",
                if unanimous { "" } else { "?" }
            ));
            if unanimous {
                stable += 1
            } else {
                flaky += 1
            }
        }
    }
    let skipped: Vec<_> = seen
        .keys()
        .filter(|n| !text.contains(n.as_str()))
        .cloned()
        .collect();
    let mut report=format!("[conformance] seeded {count} row(s) with {stable} stable + {flaky} bistable assertion identities from {} round(s)",rounds.len());
    if !unseen.is_empty() {
        report+=&format!("\n  {} baseline row(s) got NO identities — no round completed them, so their count gate stands: {}{}",unseen.len(),unseen.iter().take(8).cloned().collect::<Vec<_>>().join(", "),if unseen.len()>8{" ..."}else{""});
    }
    if !skipped.is_empty() {
        report+=&format!("\n  {} test(s) ran but are not on the floor; adding a row is --write-baseline's job, not this: {}{}",skipped.len(),skipped.iter().take(8).cloned().collect::<Vec<_>>().join(", "),if skipped.len()>8{" ..."}else{""});
    }
    (format!("{}\n", out.join("\n")), report)
}
pub fn merge_rows(
    text: &str,
    results: &[TestResult],
    only: &[String],
    formatted: Option<&Value>,
) -> Value {
    let by_name: BTreeMap<_, _> = results.iter().map(|r| (r.name.as_str(), r)).collect();
    let missing: BTreeSet<_> = only
        .iter()
        .filter(|n| !by_name.contains_key(n.as_str()))
        .cloned()
        .collect();
    if !missing.is_empty() {
        return json!({"text":null,"refusal":format!("[conformance] --write-baseline --only REFUSED: this sweep has no result for {} named test(s):\n{}  A named test that never launched cannot be re-seeded from this run.",missing.len(),missing.iter().map(|n|format!("    {n}\n")).collect::<String>()),"added":[]});
    }
    let mut wanted = Vec::new();
    for n in only {
        if !wanted.contains(n) {
            wanted.push(n.clone());
        }
    }
    let render = |name: &str| {
        formatted
            .and_then(|f| f.get(name))
            .and_then(Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| fmt(by_name[name], &flaky_of(text, name)))
    };
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    let mut dropping = false;
    for line in py_lines(text) {
        let s = line.trim();
        if line.chars().next().is_some_and(char::is_whitespace) && s.starts_with("assert") {
            if !dropping {
                out.push(line);
            }
            continue;
        }
        let name = if !s.is_empty() && !line.starts_with('#') {
            Some(
                row_parts(s)
                    .map(|r| r.0)
                    .unwrap_or_else(|_| s.split_whitespace().next().unwrap_or("").into()),
            )
        } else {
            None
        };
        if let Some(name) = name.filter(|n| by_name.contains_key(n.as_str()) && wanted.contains(n))
        {
            dropping = true;
            out.push(render(&name));
            seen.insert(name);
        } else {
            dropping = false;
            out.push(line);
        }
    }
    let added: Vec<_> = wanted.into_iter().filter(|n| !seen.contains(n)).collect();
    for name in &added {
        out.push(render(name));
    }
    json!({"text":format!("{}\n",out.join("\n")),"refusal":null,"added":added})
}
pub fn round_shape<T>(
    bugcheck: Option<&str>,
    results: &[T],
    done: Option<&str>,
    regressions: &[String],
    serial: &str,
) -> String {
    if let Some(b) = bugcheck.filter(|s| !s.is_empty()) {
        return format!("A:bugcheck {b}");
    }
    if !kernel_asserts(serial).is_empty() {
        return "A:kernel-assert".into();
    }
    if !py_lines(serial)
        .iter()
        .any(|l| !l.trim().is_empty() && !l.starts_with('#'))
    {
        return "L:no-guest-serial".into();
    }
    if results.is_empty() {
        return "B:no-launch".into();
    }
    if done.is_none_or(|s| s.is_empty()) {
        return "stalled/DID-NOT-FINISH".into();
    }
    if !regressions.is_empty() {
        "clean-but-ratchet-regression".into()
    } else {
        "clean".into()
    }
}
pub fn cmake_cache_bools(path: &Path) -> BTreeMap<String, bool> {
    let Ok(bytes) = fs::read(path) else {
        return BTreeMap::new();
    };
    py_lines(&String::from_utf8_lossy(&bytes))
        .iter()
        .filter_map(|line| {
            let (name, value) = line.split_once(":BOOL=")?;
            Some((
                name.trim().to_owned(),
                matches!(
                    value.trim().to_uppercase().as_str(),
                    "ON" | "TRUE" | "1" | "YES"
                ),
            ))
        })
        .collect()
}
pub fn media_payload_gap(build_dir: &Path, suite: &str) -> Option<String> {
    let stamp = build_dir.join("payload-selection.json");
    let fix = format!(
        "drop --no-build (conformance builds with VIBEOS_PAYLOAD={suite}) or build with it"
    );
    let text = match fs::read_to_string(&stamp) {
        Ok(t) => t,
        Err(_) => {
            return Some(format!(
                "{} is missing, so nothing says what this media holds; {fix}",
                stamp.display()
            ))
        }
    };
    let v: Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => return Some(format!("{} is unreadable ({e}); {fix}", stamp.display())),
    };
    let names = v.get("payloads").cloned().unwrap_or(json!([]));
    if !names
        .as_array()
        .is_some_and(|ns| ns.iter().any(|n| n.as_str() == Some(suite)))
    {
        let rendered = names
            .as_array()
            .map(|ns| {
                format!(
                    "[{}]",
                    ns.iter()
                        .map(|n| n
                            .as_str()
                            .map(|s| format!("'{s}'"))
                            .unwrap_or_else(|| n.to_string()))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
            .unwrap_or_else(|| names.to_string());
        return Some(format!(
            "this media was built with payloads {rendered}, not '{suite}'; {fix}"
        ));
    }
    let cache = cmake_cache_bools(&build_dir.join("CMakeCache.txt"));
    if !cache.is_empty()
        && !cache
            .get("ENABLE_ROSTESTS_KMTESTS")
            .copied()
            .unwrap_or(false)
        && !cache.get("ENABLE_ROSTESTS").copied().unwrap_or(false)
    {
        return Some(format!("{} was configured without the kmtest corpus (ENABLE_ROSTESTS_KMTESTS=OFF); drop --no-build (conformance passes --kmtests) or `vibeos-cli build --arch <arch> --kmtests` with VIBEOS_PAYLOAD={suite}",build_dir.file_name().unwrap_or_default().to_string_lossy()));
    }
    None
}
pub fn cli_baseline_write(
    path: &Path,
    kind: &str,
    results: &[TestResult],
    base: &Value,
    only: Option<&[String]>,
) -> Result<usize> {
    let mut rows = BTreeMap::new();
    if kind == "apitest" {
        for (name, b) in base.as_object().into_iter().flatten() {
            let mut row = b.clone();
            row["name"] = json!(name);
            let r: TestResult = serde_json::from_value(row)?;
            let flaky: BTreeSet<String> = b
                .get("flaky")
                .cloned()
                .map(serde_json::from_value)
                .transpose()?
                .unwrap_or_default();
            rows.insert(name.clone(), fmt(&r, &flaky));
        }
        for r in results {
            if only.is_none_or(|ns| ns.is_empty() || ns.contains(&r.name)) {
                rows.insert(r.name.clone(), fmt(r, &BTreeSet::new()));
            }
        }
        fs::write(
            path,
            format!(
                "# apitest conformance baseline — parse_apitest_serial.py format\n{}\n",
                rows.values().cloned().collect::<Vec<_>>().join("\n")
            ),
        )?;
        Ok(rows.len())
    } else if kind == "kmtest" {
        let formatted: Vec<_> = results.iter().map(|r| fmt(r, &BTreeSet::new())).collect();
        fs::write(
            path,
            format!(
                "# kmtest conformance baseline — parse_kmtest_serial.py format\n{}\n",
                formatted.join("\n")
            ),
        )?;
        Ok(results.len())
    } else {
        Err(Error::new(format!("unknown baseline kind {kind}")))
    }
}
fn values<T: serde::de::DeserializeOwned>(a: &Value, key: &str) -> Result<T> {
    Ok(serde_json::from_value(a.get(key).cloned().ok_or_else(
        || Error::new(format!("missing argument {key}")),
    )?)?)
}
pub fn dispatch(_root: &Path, request: &Value) -> Result<Value> {
    let a = args(request);
    let op = string(request, "op")?;
    Ok(match op {
        "conformance_parse_bad" => json!(BAD),
        "conformance_parse_refmt" => {
            let mut row = a.get("base").cloned().ok_or("missing baseline row")?;
            row["name"] = json!(string(a, "name")?);
            let flaky: BTreeSet<String> = row
                .get("flaky")
                .cloned()
                .map(serde_json::from_value)
                .transpose()?
                .unwrap_or_default();
            json!(fmt(&serde_json::from_value(row)?, &flaky))
        }
        "conformance_parse_write_cli_baseline" => json!(cli_baseline_write(
            Path::new(string(a, "path")?),
            string(a, "kind")?,
            &values::<Vec<TestResult>>(a, "results")?,
            a.get("base").unwrap_or(&json!({})),
            a.get("only")
                .filter(|v| !v.is_null())
                .cloned()
                .map(serde_json::from_value::<Vec<String>>)
                .transpose()?
                .as_deref()
        )?),
        "conformance_parse_cmake_cache_bools" => {
            json!(cmake_cache_bools(Path::new(string(a, "path")?)))
        }
        "conformance_parse_media_payload_gap" => json!(media_payload_gap(
            Path::new(string(a, "build_dir")?),
            a["suite"].as_str().unwrap_or("kmtest")
        )),
        "conformance_parse_kmtest" => json!(parse_kmtest(string(a, "text")?)?),
        "conformance_parse_apitest" => json!(parse_apitest(string(a, "text")?)?),
        "conformance_parse_splice_end" => json!(splice_split_end_markers(&values::<Vec<String>>(
            a, "lines"
        )?)),
        "conformance_parse_format" => json!(fmt(
            &values(a, "result")?,
            &a.get("flaky")
                .cloned()
                .map(serde_json::from_value)
                .transpose()?
                .unwrap_or_default()
        )),
        "conformance_parse_load_baseline" => {
            load_baseline_text(&fs::read_to_string(string(a, "path")?)?)?
        }
        "conformance_parse_baseline_entries" => json!(baseline_entries(
            if let Some(t) = a.get("text").and_then(Value::as_str) {
                t.to_owned()
            } else {
                fs::read_to_string(string(a, "path")?)?
            }
            .as_str()
        )?),
        "conformance_parse_compare" => json!(compare(
            &values::<Vec<TestResult>>(a, "results")?,
            a.get("base").ok_or("missing baseline")?,
            a.get("base_order")
                .and_then(Value::as_array)
                .map(Vec::as_slice)
        )?),
        "conformance_parse_truncated_identity" => json!(truncated_identity(
            string(a, "ident")?,
            &values(a, "known")?
        )),
        "conformance_parse_quiet_token" => json!(with_quiet_token(string(a, "tripwire")?)),
        "conformance_parse_read_serial" => json!(strip_host_timestamps(&String::from_utf8_lossy(
            &fs::read(string(a, "path")?)?
        ))),
        "conformance_parse_strip_host_timestamps" => {
            json!(strip_host_timestamps(string(a, "text")?))
        }
        "conformance_parse_coverage_refusal" => json!(baseline_coverage_refusal(
            &values::<Vec<TestResult>>(a, "results")?,
            a.get("base").ok_or("missing baseline")?,
            a.get("force").and_then(Value::as_bool).unwrap_or(false)
        )),
        "conformance_parse_lowering_note" => json!(baseline_lowering_note(
            &values::<Vec<TestResult>>(a, "results")?,
            a.get("base").ok_or("missing baseline")?
        )),
        "conformance_parse_flaky_of" => json!(flaky_of(string(a, "text")?, string(a, "name")?)),
        "conformance_parse_seed_asserts" => {
            let bad: Vec<String> = a
                .get("bad")
                .cloned()
                .map(serde_json::from_value)
                .transpose()?
                .unwrap_or_else(|| BAD.iter().map(|s| s.to_string()).collect());
            json!(seed_asserts(
                string(a, "text")?,
                &values::<Vec<Vec<TestResult>>>(a, "rounds")?,
                &bad
            ))
        }
        "conformance_parse_merge_rows" => merge_rows(
            string(a, "text")?,
            &values::<Vec<TestResult>>(a, "results")?,
            &values::<Vec<String>>(a, "only")?,
            a.get("formatted"),
        ),
        "conformance_parse_round_shape" => json!(round_shape(
            a.get("bugcheck").and_then(Value::as_str),
            &values::<Vec<Value>>(a, "results")?,
            a.get("done_line").and_then(Value::as_str),
            &values::<Vec<String>>(a, "regressions")?,
            string(a, "serial")?
        )),
        _ => {
            return Err(Error::new(format!(
                "unknown conformance parsing operation {op}"
            )))
        }
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interrupted_kmtest_end_and_missing_results() {
        let log="[KMTEST-SWEEP] launch ExHardError\n[KMTEST:END ExHardE[INTRUDER]\nrror pass=30 fail=0 skip=0]\n[KMTEST-SWEEP] exit ExHardError code=0\n[KMTEST-SWEEP] launch Other\n";
        let (r, done, bug) = parse_kmtest(log).unwrap();
        assert_eq!(r[0].status.as_deref(), Some("OK"));
        assert_eq!(r[0].pass, 30);
        assert_eq!(r[1].status.as_deref(), Some("INCOMPLETE"));
        assert!(done.is_none() && bug.is_none());
    }
    #[test]
    fn identities_cannot_be_offset_by_equal_counts() {
        let base = load_baseline_text("A OK pass=1 fail=1 skip=0\n    assert Test.c:1\n").unwrap();
        let mut r = TestResult::new("A", true);
        r.status = Some("OK".into());
        r.pass = 1;
        r.fail = 1;
        r.asserts.insert("Test.c:2".into());
        assert_eq!(
            compare(&[r], &base, None).unwrap(),
            vec!["A: NEW failing assertion Test.c:2"]
        );
    }
    #[test]
    fn apitest_summary_then_crash_and_resolved_timeout() {
        let(s,m)=parse_apitest("[APITEST-SWEEP] launch a.exe One\n[APITEST:a.exe One] One: 1 test executed (0 marked as todo, 0 failures), 0 skipped.\n*** BUGCHECK: 0x1E (0x1,0,0,0)\n").unwrap();
        assert_eq!(s[0].status.as_deref(), Some("CRASH"));
        assert_eq!(s[0].pass, 1);
        assert_eq!(m["bugcheck"], "0x1E (0x1,0,0,0)");
        let (r, _) =
            parse_apitest("[APITEST-SWEEP] ENUM-TIMEOUT a.exe (60s)\n*** BUGCHECK: 0x1E\n")
                .unwrap();
        assert_eq!(r[0].status.as_deref(), Some("ENUM-TIMEOUT"));
    }
    #[test]
    fn stamped_crlf_is_not_a_double_newline() {
        assert_eq!(
            strip_host_timestamps("[+  1.000] A\r[+  1.001] \n[+  2.000] B\r\n"),
            "A\nB\n"
        );
    }
    #[test]
    fn explicit_apitest_baseline_write_keeps_unselected_named_rows_and_bistable_ids() {
        let root = tempfile::tempdir().unwrap();
        let baseline="suite.exe One OK pass=2 fail=1 skip=0\n    assert? file.c:9\nsuite.exe Two OK pass=3 fail=0 skip=0\n";
        let base = load_baseline_text(baseline).unwrap();
        let mut r = TestResult::new("suite.exe Two", false);
        r.status = Some("OK".into());
        r.pass = 4;
        let path = root.path().join("floor.txt");
        cli_baseline_write(
            &path,
            "apitest",
            &[r],
            &base,
            Some(&["suite.exe Two".into()]),
        )
        .unwrap();
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("suite.exe One OK pass=2 fail=1 skip=0\n    assert? file.c:9"));
        assert!(text.contains("suite.exe Two OK pass=4 fail=0 skip=0"));
        assert_eq!(baseline_entries(&text).unwrap().len(), 2);
    }
}
