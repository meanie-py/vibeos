//! Board lease commands. Lease storage is shared with the shell front door.
use crate::{callback::Host, Error, Result};
use serde_json::{json, Value};
use std::{collections::BTreeMap, path::Path};
fn err(e: vibeos_bench::Error) -> Error {
    Error::new(e.message)
}
pub fn command(
    root: &Path,
    args: &Value,
    env: &BTreeMap<String, String>,
    host: &mut dyn Host,
) -> Result<Value> {
    let owner = vibeos_bench::lease::owner_of(root, env).map_err(err)?;
    let action = args.get("action").and_then(Value::as_str).unwrap_or("show");
    let output = match action {
        "show" => (true, vibeos_bench::lease::current(env, None).map_err(err)?),
        "take" => vibeos_bench::lease::take(
            &owner,
            args.get("minutes").and_then(Value::as_f64).unwrap_or(30.0),
            args.get("why").and_then(Value::as_str).unwrap_or(""),
            env,
            std::process::id(),
        )
        .map_err(err)?,
        "drop" => vibeos_bench::lease::drop(
            &owner,
            args.get("force").and_then(Value::as_bool).unwrap_or(false),
            env,
        )
        .map_err(err)?,
        _ => return Err(Error::new(format!("unknown lease action {action}"))),
    };
    let description = vibeos_bench::lease::describe(&output.1, None).map_err(err)?;
    let text = if !output.0 {
        format!(
            "refused: the bench is {description}{}",
            if action == "drop" {
                "\n  --force releases another lane's lease; only do that for a dead lane"
            } else {
                ""
            }
        )
    } else if action == "drop" {
        "released".into()
    } else {
        description
    };
    host.call("print", json!({"text":text}))?;
    Ok(json!(if output.0 { 0 } else { 75 }))
}
