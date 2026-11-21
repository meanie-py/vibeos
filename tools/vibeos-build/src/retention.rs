//! The fleet maintenance interlock and scheduled retention: who may delete, under what
//! policy, from which unit files, and whether the fleet is safe to hand to a scheduler.
//!
//! The interlock is a FILE PROTOCOL shared with every lane's CLI, whatever commit it is on:
//! every command holds `maintenance.lock` shared, cleanup holds it exclusive, a pinned
//! `maintenance-owner` names the one checkout allowed to delete, and `maintenance-disabled`
//! is the kill switch. The flock itself stays with the facade (`tests/qemu/cli/maintenance.py`),
//! because it must be held for the life of the command; everything decided about it is here.
use crate::provenance::py_repr;
use crate::{Error, Result};
use regex::Regex;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::Path;

/// The interlock protocol this checkout's facade implements. A lane is safe to schedule
/// deletions around only if its CLI takes the same locks the same way.
pub const INTERLOCK_PROTOCOL: u32 = 1;

/// sha256 of the `maintenance.py` every lane carried from ef568b53f0e until the protocol was
/// declared in the file. It implements protocol 1, so lanes still on it stay compatible.
const LEGACY_V1_SHA256: &str = "6eb9c16f5dd097c141c11f8619c534085704b53068cf34a58363796856103ad7";

fn value_error(message: impl Into<String>) -> Error {
    Error {
        kind: Some("value".into()),
        message: message.into(),
    }
}

/// Why this checkout may not run cleanup now, or None when it may: the kill switch, then a
/// missing or foreign owner pin.
pub fn owner_refusal(dir: &Path, repo: &Path) -> Option<String> {
    if dir.join("maintenance-disabled").exists() {
        return Some("host maintenance-disabled kill switch is present".into());
    }
    let Ok(owner) = std::fs::read_to_string(dir.join("maintenance-owner")) else {
        return Some("prune disabled: drain old clients, then pin maintenance-owner".into());
    };
    let owner = owner.trim();
    // `repo` arrives resolved (Python's non-strict resolve); Path equality ignores a
    // trailing or doubled separator, as pathlib's does.
    if owner.is_empty() || Path::new(owner) != repo {
        return Some(format!(
            "prune must run from pinned owner {}",
            py_repr(owner)
        ));
    }
    None
}

/// The interlock protocol a lane's `maintenance.py` implements: the one it declares, or 1
/// for the exact legacy file that predates the declaration.
pub fn protocol_of(text: &[u8]) -> Option<u32> {
    let declared = Regex::new(r"(?m)^INTERLOCK_PROTOCOL = (\d+)\s*$").unwrap();
    if let Some(c) = declared.captures(&String::from_utf8_lossy(text)) {
        return c[1].parse().ok();
    }
    let digest = format!("{:x}", Sha256::digest(text));
    (digest == LEGACY_V1_SHA256).then_some(INTERLOCK_PROTOCOL)
}

/// A lane whose CLI would not honor this checkout's interlock: no interlock file, a CLI that
/// does not take the activity lock, or a different protocol. Byte-identical code always
/// agrees. Every lane is checked, including idle ones: a lane can start a build the moment
/// an exclusive lock releases.
pub fn incompatible(reference: &[u8], lanes: &[Value]) -> Vec<String> {
    let want = protocol_of(reference);
    lanes
        .iter()
        .filter(|lane| {
            let code = lane["maintenance"].as_str().map(str::as_bytes);
            let takes_lock = lane["main"]
                .as_str()
                .is_some_and(|m| m.contains("with maintenance.activity(REPO):"));
            let agrees =
                code.is_some_and(|c| c == reference || (want.is_some() && protocol_of(c) == want));
            !(takes_lock && agrees)
        })
        .map(|lane| lane["path"].as_str().unwrap_or("").to_owned())
        .collect()
}

/// A scheduled-retention policy, validated. `owner` is this checkout, resolved.
pub fn read_policy(text: &str, owner: &str) -> Result<Value> {
    let value: Value = serde_json::from_str(text).map_err(|e| value_error(e.to_string()))?;
    let fields = [
        "version",
        "owner",
        "retain_gib",
        "min_free_gib",
        "idle_hours",
    ];
    let Some(map) = value.as_object() else {
        return Err(value_error("invalid retention policy fields"));
    };
    if map.len() != fields.len() || !fields.iter().all(|f| map.contains_key(*f)) {
        return Err(value_error("invalid retention policy fields"));
    }
    if map["version"].as_f64() != Some(1.0) || map["owner"].as_str() != Some(owner) {
        return Err(value_error("retention policy owner/version mismatch"));
    }
    for name in ["retain_gib", "min_free_gib", "idle_hours"] {
        let ok = map[name]
            .as_f64()
            .is_some_and(|n| n.is_finite() && n >= 0.0);
        if !ok {
            return Err(value_error(format!("invalid retention {name}")));
        }
    }
    if map["idle_hours"].as_f64().unwrap_or(0.0) < 1.0 {
        return Err(value_error(
            "scheduled retention requires at least one idle hour",
        ));
    }
    Ok(value)
}

/// The systemd user units for scheduled retention, run from `repo`. systemd expands percent
/// specifiers even inside double quotes, so the path is escaped for that too.
pub fn units(repo: &str) -> (String, String) {
    let escaped = repo
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('%', "%%");
    let service = format!(
        "[Unit]\nDescription=VibeOS idle generated-output retention\n\
         [Service]\nType=oneshot\n\
         ExecStart=\"{escaped}/vibeos-cli\" retention --run\n\
         Nice=19\nIOSchedulingClass=idle\n\
         SuccessExitStatus=2\nTimeoutStartSec=30min\n"
    );
    // SuccessExitStatus=2: a busy fleet or the kill switch is an expected, skipped
    // maintenance opportunity, not a failed unit.
    let timer = "[Unit]\nDescription=Hourly VibeOS retention opportunity\n\
                 [Timer]\nOnBootSec=10min\nOnUnitActiveSec=1h\nRandomizedDelaySec=5min\n\
                 [Install]\nWantedBy=timers.target\n"
        .to_owned();
    (service, timer)
}

pub fn dispatch(request: &Value) -> Result<Value> {
    let a = &request["kwargs"];
    let text = |k: &str| a[k].as_str().unwrap_or("");
    match request["op"].as_str().unwrap_or("") {
        "retention_owner_refusal" => Ok(json!(owner_refusal(
            Path::new(text("dir")),
            Path::new(text("repo"))
        ))),
        "retention_protocol_of" => Ok(json!(protocol_of(text("text").as_bytes()))),
        "retention_incompatible" => Ok(json!(incompatible(
            text("reference").as_bytes(),
            a["lanes"].as_array().map_or(&[][..], Vec::as_slice)
        ))),
        "retention_read_policy" => read_policy(text("text"), text("owner")),
        "retention_units" => Ok(json!(units(text("repo")))),
        op => Err(format!("unknown retention operation {op}").into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_declared_protocol_or_the_legacy_bytes_are_compatible_and_nothing_else_is() {
        let new = format!("x\nINTERLOCK_PROTOCOL = {INTERLOCK_PROTOCOL}\n");
        let lane = |p: &str, m: Option<&str>| {
            json!({"path": p, "maintenance": m,
                   "main": "with maintenance.activity(REPO):\n"})
        };
        let lanes = [
            lane("/same", Some(&new)),
            lane("/edited", Some("comment\nINTERLOCK_PROTOCOL = 1\n")),
            lane("/v2", Some("INTERLOCK_PROTOCOL = 2\n")),
            lane("/none", None),
            lane("/undeclared", Some("something else")),
            json!({"path": "/nolock", "maintenance": new, "main": "pass"}),
        ];
        assert_eq!(
            incompatible(new.as_bytes(), &lanes),
            ["/v2", "/none", "/undeclared", "/nolock"]
        );
        // An undeclared reference still accepts byte-identical lanes.
        assert_eq!(
            incompatible(b"fixture", &[lane("/a", Some("fixture"))]),
            Vec::<String>::new()
        );
    }

    #[test]
    fn the_policy_names_the_field_it_refuses() {
        let ok = r#"{"version": 1, "owner": "/r", "retain_gib": 10, "min_free_gib": 1.5, "idle_hours": 2}"#;
        assert!(read_policy(ok, "/r").is_ok());
        let bad = |t: &str| read_policy(t, "/r").unwrap_err().message;
        assert_eq!(
            bad(&ok.replace("2}", "0.5}")),
            "scheduled retention requires at least one idle hour"
        );
        assert_eq!(
            bad(&ok.replace("10,", "true,")),
            "invalid retention retain_gib"
        );
        assert_eq!(
            bad(&ok.replace("\"/r\"", "\"/x\"")),
            "retention policy owner/version mismatch"
        );
        assert_eq!(bad(r#"{"version": 1}"#), "invalid retention policy fields");
    }

    #[test]
    fn the_unit_escapes_quotes_and_systemd_percent_specifiers() {
        let (service, _) = units(r#"/w/a"b%c"#);
        assert!(service.contains(r#"ExecStart="/w/a\"b%%c/vibeos-cli" retention --run"#));
    }
}
