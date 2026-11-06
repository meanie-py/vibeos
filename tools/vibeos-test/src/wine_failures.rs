//! Wine todo statement attribution, shared by the report and its compatibility API.
use crate::{Error, Result};
use regex::Regex;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub fn todo_lines(lines: &[String], conditional: bool) -> Result<BTreeSet<usize>> {
    let marker = Regex::new(if conditional {
        r"\btodo_wine_if\s*\("
    } else {
        r"\btodo_wine\b"
    })?;
    let mut inside = BTreeSet::new();
    let (mut depth, mut cond) = (0_i64, 0_i64);
    let mut state = "";
    for (i, line) in lines.iter().enumerate() {
        let n = i + 1;
        let balance = |s: &str| s.matches('{').count() as i64 - s.matches('}').count() as i64;
        if depth > 0 {
            inside.insert(n);
            depth += balance(line);
            continue;
        }
        if state == "stmt" {
            inside.insert(n);
            state = if line.contains(';') { "" } else { "stmt" };
            continue;
        }
        let found = if cond == 0 { marker.find(line) } else { None };
        let mut rest = if let Some(m) = found {
            state = "pending";
            cond = i64::from(m.as_str().ends_with('('));
            &line[m.end()..]
        } else if state == "pending" {
            line.as_str()
        } else {
            continue;
        };
        while cond != 0 && !rest.is_empty() {
            let ch = rest.chars().next().unwrap();
            rest = &rest[ch.len_utf8()..];
            cond += match ch {
                '(' => 1,
                ')' => -1,
                _ => 0,
            };
        }
        let body = rest.trim();
        if cond != 0 || body.is_empty() {
            continue;
        }
        inside.insert(n);
        if body.starts_with('{') {
            depth = balance(body);
            state = "";
        } else {
            state = if body.contains(';') { "" } else { "stmt" };
        }
    }
    Ok(inside)
}

#[derive(Default)]
struct Row {
    todo: u64,
    conditional: u64,
    real: u64,
    detail: BTreeMap<(usize, String), u64>,
}
pub fn classify(root: &Path, text: &str) -> Result<Value> {
    let re = Regex::new(
        r"\[APITEST:(\w+)_winetest\.exe [^\]]*\] ([a-z_0-9]+\.c):(\d+): Test failed: ([^\r\n]{0,100})",
    )?;
    let mut sources = BTreeMap::new();
    // Preserve first-seen order for equal REAL counts in the CLI report.
    let mut order = Vec::new();
    let mut rows: BTreeMap<String, Row> = BTreeMap::new();
    for m in re.captures_iter(text) {
        let key = (m[1].to_owned(), m[2].to_owned());
        if !sources.contains_key(&key) {
            let p = root
                .join("modules/rostests/winetests")
                .join(&key.0)
                .join(&key.1);
            let source = if p.is_file() {
                let text = vibeos_build::read(&p)?;
                let lines: Vec<_> = text.split('\n').map(str::to_owned).collect();
                Some((todo_lines(&lines, false)?, todo_lines(&lines, true)?))
            } else {
                None
            };
            sources.insert(key.clone(), source);
        }
        let Some((todo, conditional)) = &sources[&key] else {
            continue;
        };
        let n: usize = m[3]
            .parse()
            .map_err(|_| Error::new("failure line number is too large"))?;
        let name = format!("{}:{}", key.0, key.1);
        if !rows.contains_key(&name) {
            order.push(name.clone());
        }
        let row = rows.entry(name).or_default();
        if todo.contains(&n) {
            row.todo += 1;
        } else if conditional.contains(&n) {
            row.conditional += 1;
        } else {
            row.real += 1;
            *row.detail
                .entry((n, m[4].chars().take(90).collect()))
                .or_default() += 1;
        }
    }
    Ok(Value::Array(order.iter().map(|name| {
        let r = &rows[name];
        json!({"name":name,"todo":r.todo,"todo_if":r.conditional,"real":r.real,
            "detail":r.detail.iter().map(|((line,message),count)|json!({"line":line,"message":message,"count":count})).collect::<Vec<_>>()})
    }).collect()))
}
pub fn dispatch(root: &Path, request: &Value) -> Result<Value> {
    let a = &request["kwargs"];
    match request["op"].as_str().unwrap_or("") {
        "wine_failures_todo_lines" => {
            let lines: Vec<String> = serde_json::from_value(a["lines"].clone())?;
            Ok(json!(todo_lines(
                &lines,
                a["conditional"].as_bool().unwrap_or(false)
            )?))
        }
        "wine_failures_classify" => classify(root, a["text"].as_str().ok_or("missing text")?),
        _ => Err("unknown Wine failure operation".into()),
    }
}
