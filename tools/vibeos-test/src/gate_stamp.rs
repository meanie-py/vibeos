//! A gate stamp certifies the clean tree sampled at START, never a later checkout.
use crate::{Error, Result};
use serde_json::{json, Value};
use std::{
    fs,
    io::Write,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
fn git(repo: &Path, args: &[&str]) -> Result<String> {
    let out =
        vibeos_build::process::capture(Command::new("git").arg("-C").arg(repo).args(args), 60)?;
    if out.code != 0 {
        return Err(Error::new(out.stderr.trim()));
    }
    Ok(out.stdout.trim().to_owned())
}
pub fn tree_state(repo: &Path) -> Result<Value> {
    Ok(json!([
        git(repo, &["rev-parse", "HEAD^{tree}"])?,
        !git(repo, &["status", "--porcelain", "--untracked-files=no"])?.is_empty()
    ]))
}
fn warn(detail: &str, text: &str) -> Value {
    json!({"stage":{"name":"stamp","status":"warn","detail":detail},"log":text})
}
pub fn write(repo: &Path, a: &Value) -> Result<Value> {
    let arches = a["arches"].as_array().ok_or("missing gate stamp arches")?;
    if !vibeos_build::ARCHES
        .iter()
        .all(|arch| arches.iter().any(|v| v == arch))
    {
        return Ok(json!({"log":"gate green but partial arch set — no push stamp minted"}));
    }
    if !git(repo, &["status", "--porcelain", "--untracked-files=no"])?.is_empty() {
        return Ok(warn("dirty tree — no push stamp","gate green but the tree is DIRTY — no push stamp (commit first, then re-gate; the stamp must hash the exact content that was gated)"));
    }
    let tree = git(repo, &["rev-parse", "HEAD^{tree}"])?;
    if let Some(gated) = a["gated"].as_array() {
        if gated
            .first()
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        {
            return Ok(warn("start tree unavailable — no push stamp", "gate green but its starting tree identity is unavailable — nothing can certify that this checkout is what ran. Re-gate after git identity is readable."));
        }
        if gated.get(1) == Some(&json!(true)) {
            return Ok(warn("dirty at start — no push stamp","gate green but the tree was DIRTY when the gate STARTED — what ran is not what any commit hashes to. Commit first, then gate."));
        }
        if gated
            .first()
            .and_then(Value::as_str)
            .is_some_and(|t| !t.is_empty() && t != tree)
        {
            return Ok(warn("tree changed mid-gate — no push stamp","gate green but the checkout MOVED during the gate — the boots validated the old tree, so stamping the new one would certify content nothing built. Re-gate the tree you mean to push."));
        }
    }
    let head = git(repo, &["rev-parse", "HEAD"])?;
    let common = git(repo, &["rev-parse", "--git-common-dir"])?;
    let dir = repo.join(common).join("vibeos-gate-stamps");
    fs::create_dir_all(&dir)?;
    prune(&dir, 86400)?;
    let mut arches = arches.clone();
    arches.sort_by_key(|v| v.to_string());
    let stamp = json!({"tree":tree,"head":head,"time":SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),"arches":arches,"worktree":repo});
    let mut temp = tempfile::NamedTempFile::new_in(&dir)?;
    serde_json::to_writer_pretty(&mut temp, &stamp)?;
    temp.flush()?;
    temp.as_file().sync_all()?;
    temp.persist(dir.join(format!("{tree}.json")))
        .map_err(|e| Error::new(e.to_string()))?;
    Ok(
        json!({"stage":{"name":"stamp","status":"ok","detail":format!("push stamp minted for tree {}",tree.chars().take(12).collect::<String>())}}),
    )
}
pub fn prune(dir: &Path, age: u64) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for e in fs::read_dir(dir)? {
        let p = e?.path();
        if p.extension().is_some_and(|s| s == "json")
            && fs::metadata(&p)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| SystemTime::now().duration_since(t).ok())
                .is_some_and(|d| d.as_secs() > age)
        {
            let _ = fs::remove_file(p);
        }
    }
    Ok(())
}
pub fn dispatch(repo: &Path, r: &Value) -> Result<Value> {
    let a = &r["kwargs"];
    match r["op"].as_str().unwrap_or("") {
        "gate_stamp_state" => Ok(tree_state(repo).unwrap_or(json!(["", false]))),
        "gate_stamp_prune" => {
            prune(
                Path::new(a["dir"].as_str().ok_or("missing stamp dir")?),
                a["max_age_s"].as_u64().unwrap_or(86400),
            )?;
            Ok(Value::Null)
        }
        "gate_stamp_write" => Ok(write(repo, a).unwrap_or_else(|e| {
            warn(
                &format!("stamp write failed: {e}"),
                "gate stamp publication failed",
            )
        })),
        op => Err(Error::new(format!("unknown stamp operation {op}"))),
    }
}
