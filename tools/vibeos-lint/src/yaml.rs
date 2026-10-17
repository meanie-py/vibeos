//! Decode registry YAML into the shared JSON value representation.
use serde_json::{Map, Number, Value};
use std::collections::BTreeMap;
use yaml_rust::{
    parser::{Event, EventReceiver, Parser},
    scanner::{TScalarStyle, TokenType},
    Yaml,
};

pub fn parse(text: &str) -> Result<Value, String> {
    let mut loader = Loader::default();
    Parser::new(text.chars())
        .load(&mut loader, true)
        .map_err(|e| format!("invalid YAML: {e}"))?;
    if let Some(error) = loader.error {
        return Err(error);
    }
    match loader.docs.as_slice() {
        [] => Ok(Value::Null),
        [doc] => convert(doc),
        _ => Err("expected one YAML document".into()),
    }
}

// yaml-rust parses YAML syntax/anchors; retain PyYAML's YAML 1.1 scalar schema.
// In particular plain `no` is false, while quoted "no" remains a string.
#[derive(Default)]
struct Loader {
    docs: Vec<Yaml>,
    stack: Vec<(Yaml, usize)>,
    keys: Vec<Option<Yaml>>,
    anchors: BTreeMap<usize, Yaml>,
    error: Option<String>,
}
impl Loader {
    fn insert(&mut self, value: Yaml, anchor: usize) {
        if anchor != 0 {
            self.anchors.insert(anchor, value.clone());
        }
        match self.stack.last_mut() {
            Some((Yaml::Array(values), _)) => values.push(value),
            Some((Yaml::Hash(map), _)) => {
                let pending = self.keys.last_mut().unwrap();
                if let Some(key) = pending.take() {
                    map.insert(key, value);
                } else {
                    *pending = Some(value);
                }
            }
            None => self.docs.push(value),
            _ => self.error = Some("invalid YAML collection".into()),
        }
    }
}
fn scalar(text: String, style: TScalarStyle, tag: Option<TokenType>) -> Result<Yaml, String> {
    let explicit = match tag {
        None => None,
        Some(TokenType::Tag(handle, suffix)) if handle == "!!" => Some(suffix),
        Some(other) => return Err(format!("unsupported safe YAML tag {other:?}")),
    };
    if explicit.as_deref() == Some("str") || explicit.is_none() && style != TScalarStyle::Plain {
        return Ok(Yaml::String(text));
    }
    let boolean = match text.as_str() {
        "yes" | "Yes" | "YES" | "true" | "True" | "TRUE" | "on" | "On" | "ON" => Some(true),
        "no" | "No" | "NO" | "false" | "False" | "FALSE" | "off" | "Off" | "OFF" => Some(false),
        _ => None,
    };
    if let Some(b) = boolean {
        return Ok(Yaml::Boolean(b));
    }
    if explicit.as_deref() == Some("bool") {
        return Err(format!("invalid YAML boolean {text:?}"));
    }
    if text.is_empty() || matches!(text.as_str(), "~" | "null" | "Null" | "NULL") {
        return Ok(Yaml::Null);
    }
    if text == "<<" && explicit.is_none() {
        return Ok(Yaml::String("\0merge".into()));
    }
    if let Some(ref kind) = explicit {
        if !["int", "float", "null", "timestamp"].contains(&kind.as_str()) {
            return Err(format!("unsupported safe YAML tag !!{kind}"));
        }
    }
    if explicit.is_none()
        && !text.starts_with(|c: char| c.is_ascii_digit() || matches!(c, '+' | '-' | '.'))
    {
        return Ok(Yaml::String(text));
    }
    let numeric = text.replace('_', "");
    let signed = numeric
        .strip_prefix('-')
        .or_else(|| numeric.strip_prefix('+'))
        .unwrap_or(&numeric);
    if signed.starts_with('0') && signed.len() > 1 && signed.chars().all(|c| matches!(c, '0'..='7'))
    {
        let n = i64::from_str_radix(signed, 8).map_err(|e| e.to_string())?;
        return Ok(Yaml::Integer(if numeric.starts_with('-') { -n } else { n }));
    }
    let resolved = Yaml::from_str(&numeric);
    if matches!(resolved, Yaml::String(_)) {
        Ok(Yaml::String(text))
    } else {
        Ok(resolved)
    }
}
impl EventReceiver for Loader {
    fn on_event(&mut self, event: Event) {
        if self.error.is_some() {
            return;
        }
        match event {
            Event::DocumentStart => self.anchors.clear(),
            Event::SequenceStart(anchor) => self.stack.push((Yaml::Array(Vec::new()), anchor)),
            Event::MappingStart(anchor) => {
                self.stack.push((Yaml::Hash(Default::default()), anchor));
                self.keys.push(None);
            }
            Event::SequenceEnd | Event::MappingEnd => {
                if matches!(event, Event::MappingEnd) {
                    self.keys.pop();
                }
                if let Some((value, anchor)) = self.stack.pop() {
                    self.insert(value, anchor);
                } else {
                    self.error = Some("unbalanced YAML collection".into());
                }
            }
            Event::Scalar(text, style, anchor, tag) => match scalar(text, style, tag) {
                Ok(value) => self.insert(value, anchor),
                Err(error) => self.error = Some(error),
            },
            Event::Alias(anchor) => match self.anchors.get(&anchor).cloned() {
                Some(value) => self.insert(value, 0),
                None => self.error = Some("unresolved or recursive YAML alias".into()),
            },
            _ => (),
        }
    }
}

fn convert(value: &Yaml) -> Result<Value, String> {
    Ok(match value {
        Yaml::Null => Value::Null,
        Yaml::Boolean(v) => Value::Bool(*v),
        Yaml::Integer(v) => Value::Number((*v).into()),
        Yaml::Real(v) => Value::Number(
            Number::from_f64(v.parse::<f64>().map_err(|e| e.to_string())?)
                .ok_or("non-finite YAML number")?,
        ),
        Yaml::String(v) => Value::String(v.clone()),
        Yaml::Array(v) => Value::Array(v.iter().map(convert).collect::<Result<_, _>>()?),
        Yaml::Hash(v) => {
            let mut out = Map::new();
            if let Some(merged) = v.get(&Yaml::String("\0merge".into())) {
                let defaults = if let Yaml::Array(values) = merged {
                    values.clone()
                } else {
                    vec![merged.clone()]
                };
                for default in defaults {
                    let Value::Object(map) = convert(&default)? else {
                        return Err("YAML merge requires a mapping".into());
                    };
                    for (key, value) in map {
                        out.entry(key).or_insert(value);
                    }
                }
            }
            for (key, value) in v {
                if key.as_str() == Some("\0merge") {
                    continue;
                }
                let key = key
                    .as_str()
                    .ok_or("registry YAML mapping keys must be strings")?;
                out.insert(key.to_string(), convert(value)?);
            }
            Value::Object(out)
        }
        Yaml::Alias(_) | Yaml::BadValue => {
            return Err("unresolved YAML alias or invalid value".into())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn multiline_aliases_and_typed_scalars() {
        let value =
            parse("base: &base [true, 12, null]\nalias: *base\ntext: >\n  hello\n  world\n")
                .unwrap();
        assert_eq!(value["base"], value["alias"]);
        assert_eq!(value["base"], serde_json::json!([true, 12, null]));
        assert_eq!(value["text"], "hello world\n");
    }
    #[test]
    fn invalid_or_multiple_documents_fail() {
        assert!(parse("a: [").is_err());
        assert!(parse("a: b\n---\nc: d").is_err());
        assert!(parse("? [a, b]\n: c").is_err());
    }
    #[test]
    fn python_boolean_schema_preserves_quoted_values_and_crt_symbol_names() {
        let value = parse("values: [no, yes, on, OFF, 'no', nan, inf, Infinity, 077]\n").unwrap();
        assert_eq!(
            value["values"],
            serde_json::json!([false, true, true, false, "no", "nan", "inf", "Infinity", 63])
        );
        let value = parse(
            "base: &base {translate: keep, name: default}\nentry: {<<: *base, name: override}\n",
        )
        .unwrap();
        assert_eq!(
            value["entry"],
            serde_json::json!({"translate":"keep", "name":"override"})
        );
    }
}
