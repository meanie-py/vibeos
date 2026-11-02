//! Compiler-derived msvcrt reference constants and layout assertions.
//!
//! The embedded specifications preserve the Python generators' ordered probes
//! and output text. All values are obtained from the current worktree's headers.
use crate::probe::{harvest, Recipe};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

fn spec(kind: &str) -> Result<Value, String> {
    let source = match kind {
        "msvcrt-consts" => include_str!("msvcrt-consts.json"),
        "seh-layout" => include_str!("seh-layout.json"),
        _ => return Err(format!("unknown CRT probe kind: {kind}")),
    };
    serde_json::from_str(source).map_err(|e| format!("invalid {kind} specification: {e}"))
}

fn string(v: &Value) -> Result<String, String> {
    v.as_str()
        .map(str::to_owned)
        .ok_or_else(|| "expected string in CRT probe specification".into())
}
fn strings(v: &Value) -> Result<Vec<String>, String> {
    v.as_array()
        .ok_or("expected array in CRT probe specification")?
        .iter()
        .map(string)
        .collect()
}
fn pairs(v: &Value) -> Result<Vec<(String, String)>, String> {
    v.as_array()
        .ok_or("expected array in CRT probe specification")?
        .iter()
        .map(|entry| {
            let pair = entry
                .as_array()
                .ok_or("expected pair in CRT probe specification")?;
            if pair.len() != 2 {
                return Err("expected two elements in CRT probe specification pair".into());
            }
            Ok((string(&pair[0])?, string(&pair[1])?))
        })
        .collect()
}

pub fn recipe(kind: &str) -> Result<Recipe, String> {
    let s = spec(kind)?;
    Ok(Recipe {
        arch: string(&s["arch"])?,
        includes: strings(&s["includes"])?,
        include_dirs: strings(&s["include_dirs"])?,
        defines: strings(&s["defines"])?,
        expressions: pairs(&s["expressions"])?,
    })
}

fn const_literal(v: i64) -> String {
    if v < 0 {
        format!("((int)0x{:08X})", v as u32)
    } else if v > 0x7fff_ffff {
        format!("0x{v:08X}u")
    } else if v > 9 {
        format!("0x{v:08X}")
    } else {
        v.to_string()
    }
}

fn expand(
    template: &str,
    values: &BTreeMap<String, i64>,
    crt_constants: bool,
) -> Result<String, String> {
    let mut rest = template;
    let mut result = String::with_capacity(template.len());
    while let Some(start) = rest.find("{{") {
        result.push_str(&rest[..start]);
        rest = &rest[start + 2..];
        let end = rest
            .find("}}")
            .ok_or("unterminated CRT probe template placeholder")?;
        let key = &rest[..end];
        let v = values
            .get(key)
            .ok_or_else(|| format!("missing CRT probe value: {key}"))?;
        if crt_constants && key.starts_with("const:") {
            result.push_str(&const_literal(*v));
        } else {
            result.push_str(&v.to_string());
        }
        rest = &rest[end + 2..];
    }
    if rest.contains("}}") {
        return Err("unexpected closing CRT probe template placeholder".into());
    }
    result.push_str(rest);
    Ok(result)
}

pub fn render(kind: &str, values: &BTreeMap<String, i64>) -> Result<Vec<(String, String)>, String> {
    let s = spec(kind)?;
    // Refuse incomplete harvests even if a missing expression is not currently rendered.
    for (key, _) in pairs(&s["expressions"])? {
        if !values.contains_key(&key) {
            return Err(format!("missing CRT probe value: {key}"));
        }
    }
    pairs(&s["outputs"])?
        .into_iter()
        .map(|(path, template)| {
            let constants = kind == "msvcrt-consts" && path.ends_with("msvcrt_crt_consts.h");
            Ok((path, expand(&template, values, constants)?))
        })
        .collect()
}

pub fn generate(root: &Path, kind: &str) -> Result<Vec<(String, String)>, String> {
    render(kind, &harvest(root, &recipe(kind)?)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_bad_kind_and_missing_values() {
        assert!(recipe("bad").is_err());
        assert!(render("seh-layout", &BTreeMap::new())
            .unwrap_err()
            .contains("missing CRT probe value"));
        assert!(expand("{{missing}}", &BTreeMap::new(), false).is_err());
        assert!(expand("{{broken", &BTreeMap::new(), false).is_err());
    }
    #[test]
    fn python_generator_fixture_parity() {
        for (kind, fixture) in [
            ("msvcrt-consts", include_str!("msvcrt-consts-fixture.json")),
            ("seh-layout", include_str!("seh-layout-fixture.json")),
        ] {
            let data: Value = serde_json::from_str(fixture).unwrap();
            let values = serde_json::from_value(data["values"].clone()).unwrap();
            let expected: Vec<(String, String)> =
                serde_json::from_value(data["outputs"].clone()).unwrap();
            assert_eq!(render(kind, &values).unwrap(), expected);
        }
    }
    #[test]
    fn constant_signedness_and_formats() {
        assert_eq!(const_literal(-1073741819), "((int)0xC0000005)");
        assert_eq!(const_literal(4294967295), "0xFFFFFFFFu");
        assert_eq!(const_literal(10), "0x0000000A");
        assert_eq!(const_literal(9), "9");
    }
}
