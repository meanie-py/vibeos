//! Shared payload YAML loader and schema validation.
use crate::{Error, Result};
use serde_json::{Map, Number, Value};
use std::{fs, path::Path};
use yaml_rust::{Yaml, YamlLoader};

fn value(y: &Yaml) -> Result<Value> {
    Ok(match y {
        Yaml::Null | Yaml::BadValue => Value::Null,
        Yaml::Boolean(b) => Value::Bool(*b),
        Yaml::Integer(n) => Value::Number((*n).into()),
        Yaml::Real(s) => Value::Number(
            Number::from_f64(s.parse().map_err(|_| Error::new("invalid YAML number"))?)
                .ok_or_else(|| Error::new("non-finite YAML number"))?,
        ),
        Yaml::String(s) => Value::String(s.clone()),
        Yaml::Array(a) => Value::Array(a.iter().map(value).collect::<Result<_>>()?),
        Yaml::Hash(h) => {
            let mut m = Map::new();
            for (k, v) in h {
                let key = k
                    .as_str()
                    .ok_or_else(|| Error::new("YAML mapping key must be a string"))?;
                m.insert(key.to_owned(), value(v)?);
            }
            Value::Object(m)
        }
        Yaml::Alias(_) => return Err(Error::new("unresolved YAML alias")),
    })
}
pub fn available(root: &Path) -> Result<Vec<String>> {
    let dir = root.join("tools/payloads");
    let mut out = Vec::new();
    for e in fs::read_dir(&dir).map_err(|e| Error::new(format!("{}: {e}", dir.display())))? {
        let e = e.map_err(|e| Error::new(e.to_string()))?;
        if e.path().join("payload.yaml").is_file() {
            out.push(e.file_name().to_string_lossy().into_owned());
        }
    }
    out.sort();
    Ok(out)
}
pub fn validate(name: &str, spec: &Value) -> Result<()> {
    let err = |s: &str| Error::new(format!("payload {name:?}: {s}"));
    if !spec.is_object() {
        return Err(err("top level must be a mapping"));
    }
    if spec.get("name").and_then(Value::as_str) != Some(name) {
        return Err(err("name does not match directory name"));
    }
    let binaries = spec
        .get("binaries")
        .and_then(Value::as_array)
        .ok_or_else(|| err("`binaries` must be a list"))?;
    if binaries.is_empty()
        && !spec
            .get("binaries_from")
            .is_some_and(|v| !v.is_null() && v != "")
    {
        return Err(err(
            "`binaries` must be non-empty or accompanied by `binaries_from`",
        ));
    }
    for b in binaries {
        if !b.is_object() || b.get("src").is_none() || b.get("dest").is_none() {
            return Err(err("each binaries entry needs `src` and `dest`"));
        }
        let dest = b["dest"]
            .as_str()
            .ok_or_else(|| err("dest must be a string"))?;
        if dest.starts_with(['/', '\\']) || dest.contains("..") {
            return Err(err("dest must be a relative guest path"));
        }
        if !["i386", "amd64", "arm64"].contains(
            &b.get("build_arch")
                .and_then(Value::as_str)
                .unwrap_or("i386"),
        ) {
            return Err(err("invalid build_arch"));
        }
        if !["reactos", "image"]
            .contains(&b.get("root").and_then(Value::as_str).unwrap_or("reactos"))
        {
            return Err(err("invalid root"));
        }
    }
    if let Some(sets) = spec.get("guest_sets") {
        for g in sets
            .as_array()
            .ok_or_else(|| err("`guest_sets` must be a list"))?
        {
            let dest = g
                .get("dest")
                .and_then(Value::as_str)
                .ok_or_else(|| err("guest set needs dest"))?;
            if g.get("build_dir").and_then(Value::as_str).is_none()
                || g.get("roots")
                    .and_then(Value::as_array)
                    .is_none_or(|r| r.is_empty())
            {
                return Err(err("guest set needs build_dir and non-empty roots"));
            }
            if dest.starts_with(['/', '\\']) || dest.contains("..") {
                return Err(err("guest set dest must be relative"));
            }
        }
    }
    Ok(())
}
pub fn load(root: &Path, name: &str) -> Result<Value> {
    let p = root.join("tools/payloads").join(name).join("payload.yaml");
    if !p.is_file() {
        return Err(Error::new(format!(
            "unknown payload {name:?} (no {}). Available: {}",
            p.display(),
            available(root)?.join(", ")
        )));
    }
    let spec = parse_file(&p)?;
    validate(name, &spec)?;
    Ok(spec)
}
pub fn parse_file(p: &Path) -> Result<Value> {
    let s = fs::read_to_string(p).map_err(|e| Error::new(format!("{}: {e}", p.display())))?;
    let docs = YamlLoader::load_from_str(&s)
        .map_err(|e| Error::new(format!("{}: YAML parse error: {e}", p.display())))?;
    if docs.len() != 1 {
        return Err(Error::new(format!(
            "{}: expected one YAML document",
            p.display()
        )));
    }
    value(&docs[0])
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn validates_paths_and_schema() {
        assert!(validate(
            "x",
            &json!({"name":"x","binaries":[{"src":"a","dest":"../b"}]})
        )
        .is_err());
        assert!(validate(
            "x",
            &json!({"name":"x","binaries":[],"binaries_from":"a.py"})
        )
        .is_ok());
    }
}
