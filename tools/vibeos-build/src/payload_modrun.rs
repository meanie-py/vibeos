//! Host side of modrun: suite compilation, serial transport and verdicts.
use crate::{Error, Result};
use regex::Regex;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
type Key = (String, String);
type Records = BTreeMap<Key, Vec<String>>;
type Exits = BTreeMap<Key, (String, u64, String)>;
#[derive(Clone, Debug)]
pub struct Case {
    pub id: String,
    pub suite: String,
    pub variants: Vec<(String, String)>,
    pub args: String,
    pub stdin: Vec<String>,
    pub expect: Vec<String>,
    pub timeout: u64,
    pub normalize: Vec<(String, String)>,
    pub xfail: String,
    pub diverge: Vec<(String, BTreeMap<String, String>)>,
}
fn value_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "None".into(),
        _ => v.to_string(),
    }
}
fn strs(v: Option<&Value>) -> Vec<String> {
    v.and_then(Value::as_array)
        .map(|a| a.iter().map(value_str).collect())
        .unwrap_or_default()
}
fn rules(v: Option<&Value>) -> Result<Vec<(String, String)>> {
    v.and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .map(|r| {
                    let x = r.as_array().ok_or("modrun: normalize must contain pairs")?;
                    if x.len() != 2 {
                        return Err(Error::new("modrun: normalize must contain pairs"));
                    }
                    Ok((value_str(&x[0]), value_str(&x[1])))
                })
                .collect()
        })
        .unwrap_or(Ok(vec![]))
}
fn on(v: Option<&Value>, arch: &str) -> bool {
    v.and_then(Value::as_array).map_or(true, |a| {
        a.is_empty()
            || a.iter()
                .any(|x| x.as_str().is_some_and(|x| arch.starts_with(x)))
    })
}
pub fn load_suites(dir: &Path) -> Result<Vec<(String, Value)>> {
    let mut paths = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|x| x.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "yaml"))
        .collect::<Vec<_>>();
    paths.sort();
    let only = std::env::var("MODRUN_SUITES").unwrap_or_default();
    // Commas or whitespace: `MODRUN_SUITES="d3d8 d3d9 ddraw"` read as ONE suite name before.
    let selected = only
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|x| !x.is_empty())
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for path in paths {
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let spec = crate::payload_spec::parse_file(&path)?;
        if spec.get("module").and_then(Value::as_str) != Some(name.as_str()) {
            return Err(Error::new(format!(
                "modrun: {}: `module:` must be {:?}",
                path.display(),
                name
            )));
        }
        if spec
            .get("variants")
            .and_then(Value::as_object)
            .map_or(true, |x| x.is_empty())
            || spec
                .get("cases")
                .and_then(Value::as_array)
                .map_or(true, |x| x.is_empty())
        {
            return Err(Error::new(format!(
                "modrun: {}: needs `variants` and `cases`",
                path.display()
            )));
        }
        if selected.is_empty() || selected.contains(name.as_str()) {
            seen.insert(name.clone());
            out.push((name, spec));
        }
    }
    let missing = selected
        .iter()
        .filter(|x| !seen.contains(**x))
        .copied()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(Error::new(format!(
            "modrun: MODRUN_SUITES names no suite: {}",
            missing.join(", ")
        )));
    }
    Ok(out)
}
pub fn cases(dir: &Path, arch: &str) -> Result<Vec<Case>> {
    let mut out = Vec::new();
    for (name, spec) in load_suites(dir)? {
        if !on(spec.get("arches"), arch) {
            continue;
        }
        let yaml = std::fs::read_to_string(dir.join(format!("{name}.yaml")))?;
        let docs =
            yaml_rust::YamlLoader::load_from_str(&yaml).map_err(|e| Error::new(e.to_string()))?;
        let doc = docs.first().ok_or("modrun: empty YAML")?;
        for (case_index, c) in spec["cases"].as_array().unwrap().iter().enumerate() {
            if !on(c.get("arches"), arch) {
                continue;
            }
            let ycase = &doc["cases"][case_index];
            let yvariants = if ycase["variants"].as_hash().is_some_and(|h| !h.is_empty()) {
                &ycase["variants"]
            } else {
                &doc["variants"]
            };
            let variants = yvariants
                .as_hash()
                .ok_or("modrun: missing variants")?
                .iter()
                .map(|(k, v)| {
                    Ok((
                        k.as_str().ok_or("modrun: invalid variant name")?.to_owned(),
                        v.as_str().ok_or("modrun: invalid variant path")?.to_owned(),
                    ))
                })
                .collect::<Result<Vec<_>>>()?;
            let mut normalize = rules(spec.get("normalize"))?;
            normalize.extend(rules(c.get("normalize"))?);
            let mut diverge = Vec::new();
            for d in c
                .get("diverge")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let map = d.as_object().ok_or("modrun: diverge must be a mapping")?;
                let why = map
                    .get("why")
                    .map(value_str)
                    .ok_or("modrun: diverge missing why")?;
                diverge.push((
                    why,
                    map.iter()
                        .filter(|(k, _)| k.as_str() != "why")
                        .map(|(k, v)| (k.clone(), value_str(v)))
                        .collect(),
                ))
            }
            out.push(Case {
                id: format!(
                    "{name}.{}",
                    c.get("id")
                        .map(value_str)
                        .ok_or("modrun: case missing id")?
                ),
                suite: name.clone(),
                variants,
                args: c.get("args").map(value_str).unwrap_or_default(),
                stdin: strs(c.get("stdin")),
                expect: strs(c.get("expect")),
                timeout: c
                    .get("timeout")
                    .or_else(|| spec.get("timeout"))
                    .and_then(Value::as_u64)
                    .unwrap_or(60),
                normalize,
                xfail: c.get("xfail").map(value_str).unwrap_or_default(),
                diverge,
            });
        }
    }
    Ok(out)
}
pub fn case_file(dir: &Path, arch: &str) -> Result<String> {
    let mut out = format!("# GENERATED by tools/payloads/modrun/modrun.py gen --arch {arch}\n");
    for c in cases(dir, arch)? {
        out.push_str(&format!("case {}\n", c.id));
        for (v, exe) in c.variants {
            out.push_str(&format!(
                "variant {v} {exe}{}\n",
                if c.args.is_empty() {
                    String::new()
                } else {
                    format!(" {}", c.args)
                }
            ));
        }
        for s in c.stdin {
            out.push_str(&format!("stdin {s}\n"))
        }
        out.push_str(&format!("timeout {}\nend\n", c.timeout));
    }
    Ok(out)
}
pub fn checksum(text: &str, seq: u32) -> u16 {
    let mut h = 0x811c9dc5u32 ^ seq;
    for b in text.as_bytes() {
        h = (h ^ u32::from(*b)).wrapping_mul(0x01000193)
    }
    (h ^ (h >> 16)) as u16
}
pub fn parse(serial: &str) -> Result<(Records, Exits)> {
    let line = Regex::new(r"\[MODRUN:([^:\]]+):([^:\]]+):(\d+)(\+?):([0-9a-f]{4})\] ?(.*)")?;
    let exit = Regex::new(r"\[MODRUN\] exit (\S+) (\S+) rc=(\S+) ms=(\d+) records=(\d+)(.*)")?;
    let mut records: BTreeMap<Key, BTreeMap<usize, (String, bool)>> = BTreeMap::new();
    let mut exits = Exits::new();
    let mut totals = BTreeMap::new();
    for raw in serial.split('\n') {
        if let Some(m) = exit.captures(raw.trim_end_matches('\r')) {
            let k = (m[1].to_owned(), m[2].to_owned());
            exits.insert(
                k.clone(),
                (
                    m[3].into(),
                    m[4].parse()
                        .map_err(|e: std::num::ParseIntError| Error::new(e.to_string()))?,
                    m[6].trim().into(),
                ),
            );
            totals.insert(
                k,
                m[5].parse::<usize>()
                    .map_err(|e| Error::new(e.to_string()))?,
            );
            continue;
        }
        if let Some(m) = line.captures(raw) {
            let seq = m[3]
                .parse::<usize>()
                .map_err(|e| Error::new(e.to_string()))?;
            let want = u16::from_str_radix(&m[5], 16).map_err(|e| Error::new(e.to_string()))?;
            let text = &m[6];
            let latin = text
                .chars()
                .map(|c| u8::try_from(c as u32))
                .collect::<std::result::Result<Vec<_>, _>>()
                .ok()
                .and_then(|v| String::from_utf8(v).ok());
            let candidates = latin
                .iter()
                .map(String::as_str)
                .chain(std::iter::once(text));
            let mut found = None;
            for candidate in candidates {
                for t in [
                    candidate,
                    candidate
                        .char_indices()
                        .last()
                        .map(|(i, _)| &candidate[..i])
                        .unwrap_or(candidate),
                ] {
                    if checksum(t, seq as u32) == want {
                        found = Some(t.to_owned());
                        break;
                    }
                }
                if found.is_some() {
                    break;
                }
            }
            if let Some(t) = found {
                records
                    .entry((m[1].into(), m[2].into()))
                    .or_default()
                    .entry(seq)
                    .or_insert((t, !m[4].is_empty()));
            }
        }
    }
    let mut out = Records::new();
    for (key, (_, _, flags)) in &mut exits {
        let total = totals[key];
        let got = records.get(key);
        let lost = (0..total)
            .filter(|s| got.and_then(|g| g.get(s)).is_none())
            .collect::<Vec<_>>();
        if !lost.is_empty() {
            *flags = format!("{} LOST={}@{}", flags, lost.len(), lost[0])
                .trim()
                .to_owned()
        }
        let mut lines = Vec::new();
        let mut cur = String::new();
        for i in 0..total {
            let (text, more) = got
                .and_then(|g| g.get(&i))
                .map(|(t, m)| (t.as_str(), *m))
                .unwrap_or(("", false));
            cur.push_str(text);
            if !more {
                lines.push(std::mem::take(&mut cur));
            }
        }
        out.insert(key.clone(), lines);
    }
    Ok((out, exits))
}
fn regex_err(e: fancy_regex::Error) -> Error {
    Error::new(format!("modrun: regex: {e}"))
}
fn python_replacement(s: &str) -> String {
    let mut chars = s.chars().peekable();
    let mut out = String::new();
    while let Some(c) = chars.next() {
        if c == '$' {
            out.push_str("$$");
            continue;
        }
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some(c @ '1'..='9') => {
                let mut n = c.to_string();
                while chars.peek().is_some_and(|c| c.is_ascii_digit()) {
                    n.push(chars.next().unwrap());
                }
                out.push_str(&format!("${{{n}}}"));
            }
            Some('g') if chars.peek() == Some(&'<') => {
                chars.next();
                let name: String = chars.by_ref().take_while(|c| *c != '>').collect();
                out.push_str(&format!("${{{name}}}"));
            }
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some(c) => out.push(c),
            None => out.push('\\'),
        }
    }
    out
}
fn normal(lines: &[String], rules: &[(String, String)]) -> Result<Vec<String>> {
    let mut lines = lines
        .iter()
        .map(|l| l.trim_end().to_owned())
        .collect::<Vec<_>>();
    for (pat, repl) in rules {
        let re = fancy_regex::Regex::new(pat).map_err(regex_err)?;
        let replacement = python_replacement(repl);
        for line in &mut lines {
            *line = re
                .try_replacen(line, 0, replacement.as_str())
                .map_err(regex_err)?
                .into_owned();
        }
    }
    while lines.last().is_some_and(|s| s.is_empty()) {
        lines.pop();
    }
    Ok(lines)
}
// Produce unified hunks with two context lines. LCS is bounded; for larger changes,
// retaining the shared prefix/suffix gives useful diagnostics without quadratic memory.
fn diff(a: &[String], b: &[String], an: &str, bn: &str) -> String {
    let mut prefix = 0;
    while prefix < a.len() && prefix < b.len() && a[prefix] == b[prefix] {
        prefix += 1
    }
    let mut suffix = 0;
    while suffix < a.len() - prefix
        && suffix < b.len() - prefix
        && a[a.len() - 1 - suffix] == b[b.len() - 1 - suffix]
    {
        suffix += 1
    }
    let (aa, bb) = (&a[prefix..a.len() - suffix], &b[prefix..b.len() - suffix]);
    let mut edits: Vec<(char, &str)> = a[..prefix].iter().map(|x| (' ', x.as_str())).collect();
    if !aa.is_empty() && !bb.is_empty() && aa.len().saturating_mul(bb.len()) <= 1_000_000 {
        let mut lengths = vec![vec![0u32; bb.len() + 1]; aa.len() + 1];
        for i in (0..aa.len()).rev() {
            for j in (0..bb.len()).rev() {
                lengths[i][j] = if aa[i] == bb[j] {
                    1 + lengths[i + 1][j + 1]
                } else {
                    lengths[i + 1][j].max(lengths[i][j + 1])
                }
            }
        }
        let (mut i, mut j) = (0, 0);
        while i < aa.len() || j < bb.len() {
            if i < aa.len() && j < bb.len() && aa[i] == bb[j] {
                edits.push((' ', &aa[i]));
                i += 1;
                j += 1
            } else if i < aa.len() && (j == bb.len() || lengths[i + 1][j] >= lengths[i][j + 1]) {
                edits.push(('-', &aa[i]));
                i += 1
            } else {
                edits.push(('+', &bb[j]));
                j += 1
            }
        }
    } else {
        edits.extend(aa.iter().map(|x| ('-', x.as_str())));
        edits.extend(bb.iter().map(|x| ('+', x.as_str())));
    }
    edits.extend(a[a.len() - suffix..].iter().map(|x| (' ', x.as_str())));
    let mut groups: Vec<(usize, usize)> = Vec::new();
    for (i, (kind, _)) in edits.iter().enumerate() {
        if *kind == ' ' {
            continue;
        }
        let start = i.saturating_sub(2);
        let end = (i + 3).min(edits.len());
        if let Some(last) = groups.last_mut() {
            if start <= last.1 {
                last.1 = last.1.max(end);
                continue;
            }
        }
        groups.push((start, end));
    }
    fn range(start: usize, len: usize) -> String {
        if len == 1 {
            format!("{}", start + 1)
        } else if len == 0 {
            format!("{start},0")
        } else {
            format!("{},{}", start + 1, len)
        }
    }
    let mut result = vec![format!("--- {an}"), format!("+++ {bn}")];
    let mut consumed_a = 0;
    let mut consumed_b = 0;
    let mut cursor = 0;
    for (start, end) in groups {
        for (k, _) in &edits[cursor..start] {
            if *k != '+' {
                consumed_a += 1
            }
            if *k != '-' {
                consumed_b += 1
            }
        }
        let ac = edits[start..end].iter().filter(|(k, _)| *k != '+').count();
        let bc = edits[start..end].iter().filter(|(k, _)| *k != '-').count();
        result.push(format!(
            "@@ -{} +{} @@",
            range(consumed_a, ac),
            range(consumed_b, bc)
        ));
        for (k, t) in &edits[start..end] {
            result.push(format!("{k}{t}"));
        }
        consumed_a += ac;
        consumed_b += bc;
        cursor = end;
        if result.len() >= 60 {
            break;
        }
    }
    result.truncate(60);
    result.join("\n    ")
}
pub fn grade_dir(dir: &Path, serial: &str, arch: &str) -> Result<(Vec<String>, Vec<String>)> {
    let (out, exits) = parse(serial)?;
    let announced = Regex::new(r"\[MODRUN\] case (\S+) variants=")?
        .captures_iter(serial)
        .map(|m| m[1].to_owned())
        .collect::<BTreeSet<_>>();
    let finished = serial.contains("[MODRUN] done cases=");
    let mut report = Vec::new();
    let mut failures = Vec::new();
    for c in cases(dir, arch)? {
        let mut why = Vec::new();
        let ref_name = &c.variants.first().ok_or("modrun: no variants")?.0;
        let mut reference: Option<(Vec<String>, u32)> = None;
        let mut hits = vec![0usize; c.diverge.len()];
        for (v, _) in &c.variants {
            let key = (c.id.clone(), v.clone());
            let Some((rc, ms, flags)) = exits.get(&key) else {
                why.push(if finished&&!announced.contains(&c.id){format!("{v}: not on the media (the suite changed after it was built; rebuild with VIBEOS_PAYLOAD=modrun ./vibeos-cli build --arch <arch>)")}else{format!("{v}: never finished (no exit line: guest died, or modrun stopped)")});
                continue;
            };
            if rc == "LAUNCH" {
                why.push(format!("{v}: could not be launched ({flags})"));
                continue;
            }
            if flags.contains("TIMEOUT") {
                why.push(format!(
                    "{v}: TIMEOUT after {ms}ms (thread dump in the serial)"
                ))
            }
            if flags.contains("TRUNCATED") {
                why.push(format!("{v}: output over 1 MiB, truncated"))
            }
            if let Some(lost) = flags.split("LOST=").nth(1) {
                why.push(format!("{v}: TRANSPORT: the serial lost both copies of a record ({}); output not graded",lost.split_whitespace().next().unwrap_or("")));
                continue;
            }
            let code = u32::from_str_radix(rc.trim_start_matches("0x"), 16)
                .map_err(|e| Error::new(format!("modrun: invalid exit {rc}: {e}")))?;
            if code >= 0xc0000000 && code != 0xffffffff && !flags.contains("TIMEOUT") {
                why.push(format!("{v}: crashed, exit 0x{code:08x}"))
            }
            let lines = normal(
                out.get(&key).map(Vec::as_slice).unwrap_or(&[]),
                &c.normalize,
            )?;
            for e in &c.expect {
                if !lines.iter().any(|l| l.contains(e)) {
                    why.push(format!("{v}: never printed '{e}'"))
                }
            }
            let mut kept = Vec::new();
            for line in lines {
                let mut matched = false;
                for (i, (_, pats)) in c.diverge.iter().enumerate() {
                    if let Some(pat) = pats.get(v) {
                        if fancy_regex::Regex::new(pat)
                            .map_err(regex_err)?
                            .is_match(&line)
                            .map_err(regex_err)?
                        {
                            hits[i] += 1;
                            matched = true
                        }
                    }
                }
                if !matched {
                    kept.push(line)
                }
            }
            if let Some((ref_lines, ref_rc)) = &reference {
                if code != *ref_rc {
                    why.push(format!(
                        "{v}: exit 0x{code:08x}, {ref_name} exited 0x{ref_rc:08x}"
                    ))
                }
                if kept != *ref_lines {
                    why.push(format!(
                        "{v}: output differs from {ref_name}:\n    {}",
                        diff(ref_lines, &kept, ref_name, v)
                    ))
                }
            } else {
                reference = Some((kept, code));
            }
        }
        for ((reason, _), hit) in c.diverge.iter().zip(hits) {
            if hit == 0 {
                why.push(format!(
                    "diverge '{reason}' matched no line: the difference is gone, drop it"
                ))
            }
        }
        let n = out
            .get(&(c.id.clone(), ref_name.clone()))
            .map_or(0, Vec::len);
        if !c.xfail.is_empty() && !why.is_empty() {
            report.push(format!("XFAIL {} ({})", c.id, c.xfail));
            report.extend(why.iter().map(|w| format!("  {w}")));
            continue;
        }
        if !c.xfail.is_empty() {
            failures.push(format!(
                "{}: XPASS -- drop its `xfail:` ({})",
                c.id, c.xfail
            ));
            report.push(format!("XPASS {}: remove the xfail", c.id));
            continue;
        }
        if !why.is_empty() {
            failures.push(format!(
                "{}: {}",
                c.id,
                why.iter()
                    .map(|w| w.lines().next().unwrap_or(""))
                    .collect::<Vec<_>>()
                    .join("; ")
            ));
            report.push(format!("FAIL {}", c.id));
            report.extend(why.iter().map(|w| format!("  {w}")))
        } else {
            let note = if c.diverge.is_empty() {
                String::new()
            } else {
                format!(", {} declared divergence(s)", c.diverge.len())
            };
            report.push(format!(
                "PASS {} ({} variant(s), {n} line(s) each{note})",
                c.id,
                c.variants.len()
            ))
        }
    }
    Ok((report, failures))
}
pub fn grade(root: &Path, _path: &str, output: &str, arch: &str) -> Result<Value> {
    let (report, failures) = grade_dir(&root.join("tools/payloads/modrun/suites"), output, arch)?;
    Ok(json!({"report":report,"failures":failures}))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn duplicate_and_lost_records() {
        let txt = "hello\r";
        let sum = checksum(txt, 0);
        let s = format!(
            "[MODRUN:m.a:c:0:{sum:04x}] {txt}\n[MODRUN] exit m.a c rc=0x0 ms=3 records=2\n"
        );
        let (out, ex) = parse(&s).unwrap();
        let k = ("m.a".into(), "c".into());
        assert_eq!(out[&k][0], txt);
        assert!(ex[&k].2.contains("LOST=1@1"));
    }
}
pub fn stage_entries(dir: &Path, arch: &str) -> Result<Vec<Value>> {
    let mut out: Vec<Value> = Vec::new();
    for (_, suite) in load_suites(dir)? {
        if !on(suite.get("arches"), arch) {
            continue;
        }
        for item in suite
            .get("stage")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let dest = item
                .get("dest")
                .and_then(Value::as_str)
                .ok_or("modrun: stage missing dest")?;
            let src = item
                .get("src")
                .and_then(Value::as_str)
                .ok_or("modrun: stage missing src")?;
            if let Some(prev) = out
                .iter()
                .find(|v| v.get("dest").and_then(Value::as_str) == Some(dest))
            {
                if prev.get("src").and_then(Value::as_str) != Some(src) {
                    return Err(Error::new(format!(
                        "modrun: {dest} staged from {} and {src}",
                        prev["src"].as_str().unwrap_or("")
                    )));
                }
            } else {
                out.push(item.clone())
            }
        }
    }
    Ok(out)
}
pub fn ninja_targets(dir: &Path, arch: &str) -> Result<Vec<String>> {
    let mut out = Vec::new();
    for (_, suite) in load_suites(dir)? {
        if on(suite.get("arches"), arch) {
            out.extend(strs(suite.get("ninja")))
        }
    }
    Ok(out)
}
#[cfg(test)]
mod grader_fixture {
    use super::*;
    #[test]
    fn grades_first_variant_and_transport() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("m.yaml"),
            "module: m\nvariants: {z: z.exe, a: a.exe}\ncases: [{id: x, expect: [ok]}]\n",
        )
        .unwrap();
        let c = cases(dir.path(), "amd64").unwrap();
        assert_eq!(c[0].variants[0].0, "z");
        let sum = checksum("ok", 0);
        let serial=format!("[MODRUN:m.x:z:0:{sum:04x}] ok\n[MODRUN] exit m.x z rc=0x0 ms=1 records=1\n[MODRUN:m.x:a:0:{sum:04x}] ok\n[MODRUN] exit m.x a rc=0x0 ms=1 records=1\n");
        let (r, f) = grade_dir(dir.path(), &serial, "amd64").unwrap();
        assert!(f.is_empty());
        assert!(r[0].starts_with("PASS m.x"));
        let (_, f) = grade_dir(
            dir.path(),
            &serial.replace(&format!("[MODRUN:m.x:a:0:{sum:04x}] ok\n"), ""),
            "amd64",
        )
        .unwrap();
        assert!(f[0].contains("TRANSPORT"));
    }
}

#[cfg(test)]
mod bounded_diff_tests {
    use super::*;
    #[test]
    fn unified_context_and_large_inputs() {
        let a = (0..20).map(|n| n.to_string()).collect::<Vec<_>>();
        let mut b = a.clone();
        b[10] = "changed".into();
        assert_eq!(diff(&a,&b,"c","rust"),"--- c\n    +++ rust\n    @@ -9,5 +9,5 @@\n     8\n     9\n    -10\n    +changed\n     11\n     12");
        let a = vec!["a".into(); 20000];
        let b = vec!["b".into(); 20000];
        assert!(diff(&a, &b, "c", "r").contains("-a"));
    }
    #[test]
    fn python_backrefs_and_lookbehind() {
        assert_eq!(
            normal(&["aa".into()], &[(r"(?<=(a))a".into(), r"\1".into())]).unwrap(),
            ["aa"]
        );
    }
}
