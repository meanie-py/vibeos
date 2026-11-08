//! JSON facade for Ninja closure and linker guard pure operations.
use crate::{link_guard, ninja, Error, Result};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
fn arg<'a>(args: &'a [Value], kw: &'a Value, i: usize, name: &str) -> Result<&'a Value> {
    args.get(i)
        .or_else(|| kw.get(name))
        .ok_or_else(|| Error::new(format!("missing argument {name}")))
}
fn string<'a>(v: &'a Value, name: &str) -> Result<&'a str> {
    v.as_str()
        .ok_or_else(|| Error::new(format!("{name} must be a string")))
}
fn strings(v: &Value, name: &str) -> Result<Vec<String>> {
    v.as_array()
        .ok_or_else(|| Error::new(format!("{name} must be an array")))?
        .iter()
        .map(|x| string(x, name).map(str::to_owned))
        .collect()
}
fn path(root: &Path, s: &str) -> PathBuf {
    let p = Path::new(s);
    if p.is_absolute() {
        p.to_owned()
    } else {
        root.join(p)
    }
}
fn edge_value(edges: &[ninja::Edge]) -> Value {
    Value::Array(edges.iter().map(|e| json!([e.outputs, e.inputs])).collect())
}
fn edges(v: &Value) -> Result<Vec<ninja::Edge>> {
    v.as_array()
        .ok_or_else(|| Error::new("edges must be an array"))?
        .iter()
        .map(|v| {
            let a = v
                .as_array()
                .ok_or_else(|| Error::new("edge must be a pair"))?;
            if a.len() != 2 {
                return Err(Error::new("edge must be a pair"));
            }
            Ok(ninja::Edge {
                outputs: strings(&a[0], "outputs")?,
                inputs: strings(&a[1], "inputs")?,
            })
        })
        .collect()
}
pub fn dispatch(root: &Path, request: &Value) -> Result<Value> {
    let op = string(
        request.get("op").ok_or_else(|| Error::new("missing op"))?,
        "op",
    )?;
    let empty = vec![];
    let args = match request.get("args") {
        Some(v) => v
            .as_array()
            .ok_or_else(|| Error::new("args must be an array"))?,
        None => &empty,
    };
    let kw = request.get("kwargs").unwrap_or(&Value::Null);
    if !kw.is_null() && !kw.is_object() {
        return Err(Error::new("kwargs must be an object"));
    }
    match op {
        "ninja__unescape" | "ninja__split" | "ninja__logical_lines" | "ninja__colon" => {
            let key = if op == "ninja__logical_lines" {
                "text"
            } else if op == "ninja__unescape" {
                "tok"
            } else {
                "s"
            };
            let input = string(arg(args, kw, 0, key)?, key)?;
            Ok(match op {
                "ninja__unescape" => json!(ninja::unescape(input)),
                "ninja__split" => json!(ninja::split(input)),
                "ninja__logical_lines" => json!(ninja::logical_lines(input)),
                _ => json!(ninja::colon(input)
                    .map(|i| input[..i].chars().count() as i64)
                    .unwrap_or(-1)),
            })
        }
        "ninja_prune" => {
            let build = path(root, string(arg(args, kw, 0, "build")?, "build")?);
            crate::local(root, &build)?;
            let targets = strings(arg(args, kw, 1, "targets")?, "targets")?;
            let optional = strings(kw.get("optional").unwrap_or(&json!([])), "optional")?;
            let canon = kw.get("canon_root").and_then(Value::as_str);
            let mut plan = ninja::plan(&build, &targets, canon)?;
            plan.unknown.retain(|t| !optional.contains(t));
            if !plan.unknown.is_empty() {
                return Err(Error::new(format!(
                    "unknown targets: {}",
                    plan.unknown.join(" ")
                )));
            }
            ninja::delete(&build, &plan)?;
            Ok(json!(plan.victims))
        }
        // Why `build`'s build.ninja describes a previous configure, or null when it is current.
        "ninja_configure_stale" => {
            let build = path(root, string(arg(args, kw, 0, "build")?, "build")?);
            Ok(match ninja::require_configured(&build, root) {
                Ok(()) => Value::Null,
                Err(e) => json!(e.to_string()),
            })
        }
        "ninja_parse_edges" => Ok(edge_value(&ninja::parse_edges(&path(
            root,
            string(arg(args, kw, 0, "manifest")?, "manifest")?,
        ))?)),
        "ninja_logged_outputs" => Ok(json!(ninja::logged_outputs(&path(
            root,
            string(arg(args, kw, 0, "build")?, "build")?
        ))?)),
        "ninja_plan" => {
            let build = path(root, string(arg(args, kw, 0, "build")?, "build")?);
            let targets = strings(arg(args, kw, 1, "targets")?, "targets")?;
            let canon = args
                .get(2)
                .or_else(|| kw.get("canon_root"))
                .filter(|v| !v.is_null())
                .map(|v| string(v, "canon_root"))
                .transpose()?;
            let plan = ninja::plan(&build, &targets, canon)?;
            Ok(json!([plan.victims, plan.unknown]))
        }
        "ninja_closure_outputs" => {
            let e = edges(arg(args, kw, 0, "edges")?)?;
            let targets = strings(arg(args, kw, 1, "targets")?, "targets")?;
            let mappings: BTreeMap<String, String> = match args.get(2).or_else(|| kw.get("norm")) {
                None | Some(Value::Null) => BTreeMap::new(),
                Some(v) => serde_json::from_value(v.clone())
                    .map_err(|e| Error::new(format!("norm must be a mapping: {e}")))?,
            };
            let (keep, unknown) = ninja::closure_outputs(&e, &targets, |s| {
                mappings.get(s).cloned().unwrap_or_else(|| s.to_owned())
            });
            Ok(json!([keep, unknown]))
        }
        "link_origin_kind" => Ok(json!(link_guard::origin_kind(string(
            arg(args, kw, 0, "origin")?,
            "origin"
        )?))),
        "link_parse" => Ok(json!(link_guard::parse(string(
            arg(args, kw, 0, "stderr")?,
            "stderr"
        )?))),
        "link_judge" => Ok(serde_json::to_value(link_guard::judge(
            string(arg(args, kw, 0, "symbol")?, "symbol")?,
            string(arg(args, kw, 1, "kept")?, "kept")?,
            string(arg(args, kw, 2, "dropped")?, "dropped")?,
        ))?),
        "link_output_of" => Ok(json!(link_guard::output_of(&strings(
            arg(args, kw, 0, "argv")?,
            "argv"
        )?))),
        _ => Err(Error::new(format!("unknown build helper operation: {op}"))),
    }
}
