//! Declarative CLI grammar, parsed natively for the compatibility and standalone front doors.
use crate::{Error, Result};
use serde_json::{json, Map, Value};
use std::{collections::BTreeSet, sync::LazyLock};
static GRAMMAR: LazyLock<Value> =
    LazyLock::new(|| serde_json::from_str(include_str!("cli_arguments.json")).unwrap());
fn text<'a>(v: &'a Value, k: &str) -> &'a str {
    v[k].as_str().unwrap_or("")
}
fn failure(p: &Value, msg: impl AsRef<str>) -> Error {
    Error::classified(
        "UsageError",
        format!("{}: error: {}", text(p, "prog"), msg.as_ref()),
    )
}
fn convert(p: &Value, a: &Value, s: &str) -> Result<Value> {
    let value = match text(a, "type") {
        "int" | "positive" => {
            let n = s.parse::<i64>().map_err(|_| {
                failure(
                    p,
                    format!("{}: invalid integer value: '{s}'", text(a, "dest")),
                )
            })?;
            if text(a, "type") == "positive" && n <= 0 {
                return Err(failure(p, "value must be positive"));
            }
            json!(n)
        }
        // Finite only: JSON cannot carry inf/nan, so serde wrote them as null and
        // `--retain-gib inf` arrived as "no budget at all" — prune everything eligible.
        "float" => match s.parse::<f64>() {
            Ok(f) if f.is_finite() => json!(f),
            _ => {
                let why = format!("{}: invalid float value: '{s}'", text(a, "dest"));
                return Err(failure(p, why));
            }
        },
        "Path" => json!({"path":s}),
        _ => json!(s),
    };
    if let Some(choices) = a["choices"].as_array() {
        if !choices.contains(&value) {
            return Err(failure(
                p,
                format!("{}: invalid choice: '{s}'", text(a, "dest")),
            ));
        }
    }
    Ok(value)
}
fn option<'a>(p: &Value, actions: &'a [Value], s: &str) -> Result<Option<(&'a Value, String)>> {
    if let Some(a) = actions.iter().find(|a| {
        a["options"]
            .as_array()
            .is_some_and(|opts| opts.iter().any(|v| v == s))
    }) {
        return Ok(Some((a, s.into())));
    }
    if !s.starts_with("--") {
        return Ok(None);
    }
    let matches = actions
        .iter()
        .flat_map(|a| {
            a["options"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(move |o| {
                    o.as_str()
                        .filter(|o| o.starts_with(s))
                        .map(|o| (a, o.to_string()))
                })
        })
        .collect::<Vec<_>>();
    if matches.len() > 1 {
        return Err(failure(p, format!("ambiguous option: {s}")));
    }
    Ok(matches.into_iter().next())
}
fn parse_level(p: &Value, args: &[String], out: &mut Map<String, Value>) -> Result<Option<String>> {
    let actions = p["actions"].as_array().unwrap();
    let mut seen = BTreeSet::new();
    let mut pos = actions
        .iter()
        .filter(|a| {
            a["kind"] == "subparsers" || a["options"].as_array().is_some_and(|x| x.is_empty())
        })
        .peekable();
    for a in actions {
        if a["default"] != "==SUPPRESS==" {
            out.insert(
                text(a, "dest").into(),
                if a["options"].as_array().is_some_and(|v| v.is_empty())
                    && matches!(a["nargs"].as_str(), Some("*" | "..."))
                    && a["default"].is_null()
                {
                    json!([])
                } else {
                    a["default"].clone()
                },
            );
        }
    }
    if let Some(defaults) = p["defaults"].as_object() {
        out.extend(defaults.clone());
    }
    let mut i = 0;
    let mut options = true;
    while i < args.len() {
        let arg = &args[i];
        if pos.peek().is_some_and(|a| a["nargs"] == "...")
            && (i > 0 || !arg.starts_with('-') || arg == "--")
        {
            let a = pos.next().unwrap();
            seen.insert(text(a, "dest").into());
            out.insert(
                text(a, "dest").into(),
                json!(args[i..]
                    .iter()
                    .map(|s| convert(p, a, s))
                    .collect::<Result<Vec<_>>>()?),
            );
            break;
        }
        if options && matches!(arg.as_str(), "-h" | "--help") {
            return Ok(Some(text(p, "help").into()));
        }
        if options && arg == "--" {
            options = false;
            i += 1;
            continue;
        }
        let (flag, inline) = arg
            .split_once('=')
            .map_or((arg.as_str(), None), |(a, b)| (a, Some(b)));
        let selected = if options {
            option(p, actions, flag)?
        } else {
            None
        };
        if let Some((a, actual)) = selected {
            let dest = text(a, "dest");
            seen.insert(dest.to_string());
            i += 1;
            if a["nargs"] == 0 {
                if inline.is_some() {
                    return Err(failure(p, format!("{flag}: ignored explicit argument")));
                }
                let val = if a["kind"] == "BooleanOptionalAction" {
                    !actual.starts_with("--no-")
                } else {
                    true
                };
                out.insert(dest.into(), json!(val));
                continue;
            }
            let mut values = vec![];
            if let Some(v) = inline {
                values.push(convert(p, a, v)?)
            }
            let nargs = &a["nargs"];
            let max = if nargs == "*" || nargs == "+" || nargs == "..." {
                usize::MAX
            } else {
                1
            };
            while values.len() < max && i < args.len() {
                let s = &args[i];
                let is_option = s == "--"
                    || s == "-h"
                    || s == "--help"
                    || option(p, actions, s.split('=').next().unwrap_or(s))?.is_some()
                    || (s.starts_with('-') && s.parse::<f64>().is_err() && s != "-");
                if nargs != "..." && is_option {
                    break;
                }
                values.push(convert(p, a, s)?);
                i += 1;
            }
            if values.is_empty() && nargs != "?" && nargs != "*" && nargs != "..." {
                return Err(failure(p, format!("{flag}: expected an argument")));
            }
            let v = if nargs == "*" || nargs == "+" || nargs == "..." {
                json!(values)
            } else {
                values
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| a["const"].clone())
            };
            if a["kind"] == "_AppendAction" {
                let entry = out.entry(dest.to_owned()).or_insert_with(|| json!([]));
                if entry.is_null() {
                    *entry = json!([])
                }
                entry
                    .as_array_mut()
                    .ok_or("invalid append default")?
                    .push(v);
            } else {
                out.insert(dest.into(), v);
            }
            continue;
        }
        if options && arg.starts_with('-') && arg != "-" && arg.parse::<f64>().is_err() {
            return Err(failure(p, format!("unrecognized arguments: {arg}")));
        }
        let a = pos.next().ok_or_else(|| {
            failure(
                p,
                format!("unrecognized arguments: {}", args[i..].join(" ")),
            )
        })?;
        if a["kind"] == "subparsers" {
            let child = a["choices"].get(arg).ok_or_else(|| {
                failure(p, format!("{}: invalid choice: '{arg}'", text(a, "dest")))
            })?;
            out.insert(text(a, "dest").into(), json!(arg));
            let help = parse_level(child, &args[i + 1..], out)?;
            validate(p, actions, &seen, out)?;
            return Ok(help);
        }
        let dest = text(a, "dest");
        seen.insert(dest.into());
        let nargs = &a["nargs"];
        let many = matches!(nargs.as_str(), Some("+" | "*" | "..."));
        let mut values = vec![];
        if many {
            while i < args.len() {
                if nargs != "..."
                    && options
                    && args[i].starts_with('-')
                    && args[i].parse::<f64>().is_err()
                {
                    break;
                }
                values.push(convert(p, a, &args[i])?);
                i += 1;
            }
        } else {
            values.push(convert(p, a, arg)?);
            i += 1;
        }
        out.insert(
            dest.into(),
            if many {
                json!(values)
            } else {
                values.remove(0)
            },
        );
    }
    validate(p, actions, &seen, out)?;
    Ok(None)
}
fn validate(
    p: &Value,
    actions: &[Value],
    seen: &BTreeSet<String>,
    out: &Map<String, Value>,
) -> Result<()> {
    for a in actions {
        let dest = text(a, "dest");
        let positional = a["options"].as_array().is_some_and(|o| o.is_empty());
        if ((a["required"] == true
            && !(positional && matches!(a["nargs"].as_str(), Some("*" | "..."))))
            || (positional && a["nargs"] != "?" && a["nargs"] != "*" && a["nargs"] != "..."))
            && !seen.contains(dest)
            && !out
                .get(dest)
                .is_some_and(|v| a["kind"] == "subparsers" && !v.is_null())
        {
            return Err(failure(
                p,
                format!("the following arguments are required: {dest}"),
            ));
        }
        if positional && !seen.contains(dest) && matches!(a["nargs"].as_str(), Some("*" | "...")) { /* preserve null default, matching argparse */
        }
    }
    for group in p["exclusive"].as_array().into_iter().flatten() {
        let count = group["dests"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|d| seen.contains(d.as_str().unwrap()))
            .count();
        if count > 1 {
            return Err(failure(p, "arguments are mutually exclusive"));
        }
        if group["required"] == true && count == 0 {
            return Err(failure(
                p,
                "one of the mutually exclusive arguments is required",
            ));
        }
    }
    Ok(())
}
pub fn dispatch(r: &Value) -> Result<Value> {
    let a = &r["kwargs"];
    match r["op"].as_str().unwrap_or("") {
        "cli_arguments_schema" => Ok(GRAMMAR.clone()),
        "cli_arguments_parse" => {
            let argv: Vec<String> = serde_json::from_value(a["argv"].clone())?;
            let mut out = Map::new();
            let help = parse_level(&GRAMMAR, &argv, &mut out)?;
            Ok(json!({"arguments":out,"help":help}))
        }
        op => Err(format!("unknown argument operation {op}").into()),
    }
}
