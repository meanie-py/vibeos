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

/// Python's compact, sorted, ASCII JSON encoding used for artifact identities.
pub fn canonical(value: &Value) -> crate::Result<String> {
    Ok(match value {
        Value::Array(a) => format!(
            "[{}]",
            a.iter()
                .map(canonical)
                .collect::<crate::Result<Vec<_>>>()?
                .join(",")
        ),
        Value::Object(m) => format!(
            "{{{}}}",
            m.iter()
                .map(|(k, v)| Ok(format!(
                    "{}:{}",
                    dumps(&Value::String(k.clone()), true),
                    canonical(v)?
                )))
                .collect::<crate::Result<Vec<_>>>()?
                .join(",")
        ),
        Value::Number(n) if n.to_string().contains(['.', 'e', 'E']) => python_float(n)?,
        _ => dumps(value, true),
    })
}
fn python_float(n: &serde_json::Number) -> crate::Result<String> {
    let f = n.as_f64().ok_or("non-finite artifact JSON number")?;
    if !f.is_finite() {
        return Err("non-finite artifact JSON number".into());
    }
    // Normalize the shortest decimal digits using Python's fixed/scientific boundary.
    let source = serde_json::to_string(&f)?;
    let (sign, s) = source
        .strip_prefix('-')
        .map(|s| ("-", s))
        .unwrap_or(("", source.as_str()));
    let (mantissa, exponent) = match s.split_once('e') {
        Some((m, e)) => (
            m,
            e.parse::<i32>()
                .map_err(|_| crate::Error::new("invalid float exponent"))?,
        ),
        None => (s, 0),
    };
    let decimal = mantissa.find('.').unwrap_or(mantissa.len()) as i32;
    let mut digits = mantissa.replace('.', "");
    let mut point = decimal + exponent;
    while digits.starts_with('0') && digits.len() > 1 {
        digits.remove(0);
        point -= 1;
    }
    while digits.ends_with('0') && digits.len() > 1 {
        digits.pop();
    }
    if f == 0.0 {
        return Ok(format!("{sign}0.0"));
    }
    let exp = point - 1;
    if !(-4..16).contains(&exp) {
        let tail = if digits.len() > 1 {
            format!(".{}", &digits[1..])
        } else {
            String::new()
        };
        Ok(format!(
            "{sign}{}{tail}e{}{:02}",
            &digits[..1],
            if exp < 0 { "-" } else { "+" },
            exp.abs()
        ))
    } else if point <= 0 {
        Ok(format!(
            "{sign}0.{}{}",
            "0".repeat((-point) as usize),
            digits
        ))
    } else if point >= digits.len() as i32 {
        Ok(format!(
            "{sign}{}{}.0",
            digits,
            "0".repeat(point as usize - digits.len())
        ))
    } else {
        Ok(format!(
            "{sign}{}.{}",
            &digits[..point as usize],
            &digits[point as usize..]
        ))
    }
}
