use serde_json::Value;
/// Python json.dumps defaults: ASCII escapes, a space after separators, optional sorted keys.
pub fn dumps(v: &Value, sort: bool) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => {
            let mut out = String::from("\"");
            for c in s.chars() {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    '\u{8}' => out.push_str("\\b"),
                    '\u{c}' => out.push_str("\\f"),
                    c if !((' '..='~').contains(&c)) => {
                        let mut buf = [0u16; 2];
                        for x in c.encode_utf16(&mut buf) {
                            out.push_str(&format!("\\u{x:04x}"));
                        }
                    }
                    c => out.push(c),
                }
            }
            out.push('"');
            out
        }
        Value::Array(a) => format!(
            "[{}]",
            a.iter()
                .map(|x| dumps(x, sort))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Value::Object(m) => {
            let mut pairs: Vec<_> = m.iter().collect();
            if sort {
                pairs.sort_by(|a, b| a.0.cmp(b.0));
            }
            format!(
                "{{{}}}",
                pairs
                    .into_iter()
                    .map(|(k, v)| format!(
                        "{}: {}",
                        dumps(&Value::String(k.clone()), sort),
                        dumps(v, sort)
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}
