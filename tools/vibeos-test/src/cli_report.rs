//! Stable failure identities and ledger context shared by native CLI coordinators.
use crate::{Error, Result};
use regex::Regex;
use serde_json::{json, Value};
use std::{collections::BTreeSet, path::Path, sync::LazyLock};
static HEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"0x[0-9a-fA-F]+|\b[0-9a-fA-F]{8,16}\b").unwrap());
static NUM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b\d+\b").unwrap());
static BUG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"BugCheck (0x[0-9A-Fa-f]+)").unwrap());
static GUEST: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\w+32\.exe\b").unwrap());
fn text<'a>(v: &'a Value, k: &str, d: &'a str) -> &'a str {
    v[k].as_str().filter(|s| !s.is_empty()).unwrap_or(d)
}
fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .into_iter()
        .flatten()
        .map(|s| {
            s.as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| s.to_string())
        })
        .collect()
}
pub fn norm(s: &str, width: usize) -> String {
    NUM.replace_all(&HEX.replace_all(s, "#"), "#")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(width)
        .collect()
}
pub fn pc_symbol(line: &str) -> Option<String> {
    let sym = line.split('\t').nth(1)?.trim();
    if sym.is_empty() || sym.starts_with(['<', '(']) {
        return None;
    }
    let (name, off) = sym.split_once('+').unwrap_or((sym, ""));
    let off = off
        .strip_prefix("0x")
        .map(|s| u64::from_str_radix(s, 16))
        .unwrap_or_else(|| off.parse());
    if off.is_ok_and(|n| n > 0x10000) {
        return None;
    }
    Some(name.to_owned())
}
pub fn boot_signature(boot: &Value) -> String {
    let cls = text(boot, "classification", text(boot, "outcome", "FAIL"));
    let sal = strings(&boot["salient"]);
    let joined = sal.join("\n");
    // Match the first diagnostic line, even when it has no usable symbol.
    let pc = sal
        .iter()
        .find(|s| s.contains('\t') && s.contains("->"))
        .and_then(|s| pc_symbol(s));
    if let Some(m) = BUG.captures(&joined) {
        let raw = &m[1][2..];
        let code = raw.trim_start_matches('0');
        return format!(
            "{cls} 0x{}{}{}",
            if code.is_empty() {
                "0".into()
            } else {
                code.to_uppercase()
            },
            if joined.contains("(after boot marker)") {
                " post-marker"
            } else {
                ""
            },
            pc.map(|p| format!(" @ {p}")).unwrap_or_default()
        );
    }
    if cls == "HANG_TIMEOUT" || boot["outcome"] == "timeout" {
        let kind = text(boot, "hang_kind", "?");
        if let Some(pc) = pc {
            return format!("{cls} {kind} @ {pc}");
        }
        let last = sal
            .iter()
            .find_map(|s| s.split_once("last serial:").map(|(_, s)| s))
            .unwrap_or("");
        return if last.is_empty() {
            format!("{cls} {kind}")
        } else {
            format!("{cls} {kind} | {}", norm(last, 40))
        };
    }
    if cls == "FAIL_MARKER" {
        return format!(
            "{cls} {}",
            norm(
                sal.iter()
                    .find_map(|s| s.split_once("fail marker:").map(|(_, s)| s))
                    .unwrap_or(""),
                50
            )
        );
    }
    if cls == "LADDER_FAIL" {
        let guests = sal
            .iter()
            .filter(|s| GUEST.is_match(s))
            .filter_map(|s| s.split_whitespace().next())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join(",");
        return format!("{cls} {}", if guests.is_empty() { "?" } else { &guests });
    }
    sal.first()
        .map(|s| format!("{cls} | {}", norm(s, 60)))
        .unwrap_or_else(|| cls.to_owned())
}
pub fn run_signature(summary: &Value) -> Option<String> {
    if matches!(summary["verdict"].as_str(), Some("OK" | "WARN")) {
        return None;
    }
    if let Some(b) = summary["boots"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|b| b["ok"] != true)
    {
        return Some(boot_signature(b));
    }
    let cls = text(summary, "classification", "FAIL");
    Some(
        summary["stages"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|s| s["status"] == "fail")
            .map(|s| {
                format!(
                    "{cls} {}: {}",
                    text(s, "name", ""),
                    norm(text(s, "detail", ""), 50)
                )
            })
            .unwrap_or_else(|| cls.to_owned()),
    )
}
pub fn ledger_context(repo: Option<&Path>, summary: &Value) -> Value {
    let mut extra = json!({"run_id":summary.get("run_id").cloned().unwrap_or(Value::Null)});
    if let Some(repo) =
        repo.filter(|_| matches!(summary["command"].as_str(), Some("smoke" | "gate" | "boot")))
    {
        extra["head"] = json!(vibeos_build::cli_artifacts::head(repo));
        extra["dirty"] = json!(vibeos_build::cli_artifacts::dirty(repo));
        if summary.get("runs").is_some() {
            extra["boots"] = summary["runs"].clone();
            extra["passes"] = summary["passes"].clone()
        }
        if let Some(sig) = run_signature(summary) {
            extra["sig"] = json!(sig)
        }
        if !summary["preserved"].is_null() {
            extra["preserved"] = summary["preserved"].clone()
        }
    }
    extra
}
pub fn dispatch(request: &Value) -> Result<Value> {
    let a = &request["kwargs"];
    Ok(
        match request["op"]
            .as_str()
            .unwrap_or("")
            .trim_start_matches("cli_report_")
        {
            "norm" => json!(norm(
                text(a, "text", ""),
                a["width"].as_u64().unwrap_or(70) as usize
            )),
            "pc_symbol" => json!(pc_symbol(text(a, "line", ""))),
            "boot_signature" => json!(boot_signature(&a["boot"])),
            "run_signature" => json!(run_signature(&a["summary"])),
            "ledger_context" => ledger_context(a["repo"].as_str().map(Path::new), &a["summary"]),
            op => return Err(Error::new(format!("unknown CLI report operation {op}"))),
        },
    )
}
