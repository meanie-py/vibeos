//! Byte-preserving debugcon catalog decoding and bounded reports.
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TracePoint {
    pub char: String,
    pub function: String,
    pub file: String,
    pub line: u64,
    pub description: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebugconResult {
    pub raw_bytes: Vec<u8>,
    pub milestones_reached: Vec<TracePoint>,
    pub last_milestone: Option<TracePoint>,
    pub next_expected: Option<TracePoint>,
    pub hex_dumps: BTreeMap<String, String>,
    #[serde(default)]
    pub hex_dump_order: Vec<String>,
}
impl DebugconResult {
    fn new(raw: &[u8]) -> Self {
        Self {
            raw_bytes: raw.into(),
            milestones_reached: Vec::new(),
            last_milestone: None,
            next_expected: None,
            hex_dumps: BTreeMap::new(),
            hex_dump_order: Vec::new(),
        }
    }
}
fn ascii(b: u8) -> Option<char> {
    if (32..127).contains(&b) {
        Some(b as char)
    } else {
        None
    }
}
fn text<'a>(v: &'a Value, k: &str) -> Result<&'a str> {
    v.get(k)
        .and_then(Value::as_str)
        .ok_or_else(|| Error::new(format!("debugcon catalog requires string {k}")))
}
pub fn decode(raw: &[u8], catalog: &Value) -> Result<DebugconResult> {
    let mut milestones = Vec::new();
    for tp in catalog
        .get("trace_points")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if text(tp, "char")? == "!" {
            continue;
        }
        milestones.push(serde_json::from_value::<TracePoint>(tp.clone())?);
    }
    let mut delimiters = BTreeMap::new();
    for d in catalog
        .get("hex_dump_delimiters")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        delimiters.insert(
            text(d, "open")?.to_owned(),
            (
                text(d, "close")?.to_owned(),
                text(d, "description")?.to_owned(),
            ),
        );
    }
    let mut r = DebugconResult::new(raw);
    let mut next = 0;
    let mut i = 0;
    while i < raw.len() {
        let Some(ch) = ascii(raw[i]).map(|c| c.to_string()) else {
            i += 1;
            continue;
        };
        if let Some((close, description)) = delimiters.get(&ch) {
            let mut j = i + 1;
            let mut content = String::new();
            while j < raw.len() {
                let c = ascii(raw[j]);
                if c.map(|c| c.to_string()).as_deref() == Some(close.as_str()) {
                    break;
                }
                if let Some(c) = c {
                    content.push(c);
                }
                j += 1;
            }
            if !r.hex_dumps.contains_key(description) {
                r.hex_dump_order.push(description.clone());
            }
            r.hex_dumps.insert(description.clone(), content);
            while next < milestones.len() && milestones[next].char == ch {
                r.milestones_reached.push(milestones[next].clone());
                next += 1;
            }
            if j < raw.len() {
                let c = (raw[j] as char).to_string();
                while next < milestones.len() && milestones[next].char == c {
                    r.milestones_reached.push(milestones[next].clone());
                    next += 1;
                }
            }
            i = j + 1;
            continue;
        }
        if next < milestones.len() && milestones[next].char == ch {
            r.milestones_reached.push(milestones[next].clone());
            next += 1;
        }
        i += 1;
    }
    r.last_milestone = r.milestones_reached.last().cloned();
    r.next_expected = milestones.get(next).cloned();
    Ok(r)
}
pub fn format_report(r: &DebugconResult) -> String {
    let n = r.raw_bytes.len();
    let mut lines = vec![format!("=== Debugcon Triage ({n} bytes captured) ===")];
    if n == 0 {
        lines.push("NO OUTPUT — kernel did not reach first trace point".into());
        return lines.join("\n");
    }
    let raw: String = r
        .raw_bytes
        .iter()
        .map(|b| if *b < 128 { *b as char } else { '\u{fffd}' })
        .collect();
    let count = raw.chars().count();
    let raw = if count > 120 {
        format!(
            "{}...{}",
            raw.chars().take(60).collect::<String>(),
            raw.chars().skip(count - 60).collect::<String>()
        )
    } else {
        raw
    };
    lines.push(format!("Raw: {raw}"));
    let hex = r
        .raw_bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let hex = if hex.len() > 400 {
        format!("{}...{}", &hex[..200], &hex[hex.len() - 200..])
    } else {
        hex
    };
    lines.push(format!("Hex: {hex}"));
    if let Some(tp) = &r.last_milestone {
        lines.push(format!("LAST REACHED: '{}' = {}()", tp.char, tp.function));
        lines.push(format!("  {}:{} — {}", tp.file, tp.line, tp.description));
    }
    if let Some(tp) = &r.next_expected {
        lines.push(format!("CRASHED BEFORE: '{}' = {}()", tp.char, tp.function));
        lines.push(format!("  {}:{} — {}", tp.file, tp.line, tp.description));
    }
    lines.push(format!(
        "Milestones: {} reached",
        r.milestones_reached.len()
    ));
    if !r.hex_dumps.is_empty() {
        lines.push("Hex dumps:".into());
        let mut keys = r.hex_dump_order.clone();
        for k in r.hex_dumps.keys() {
            if !keys.contains(k) {
                keys.push(k.clone());
            }
        }
        for k in keys {
            if let Some(content) = r.hex_dumps.get(&k) {
                lines.push(format!("  {k}: {content}"));
            }
        }
    }
    lines.join("\n")
}
pub fn load_catalog(root: &Path, arch: &str) -> Result<Option<Value>> {
    let path = root
        .join("tools/rustconv/data")
        .join(format!("trace_catalog_{arch}.json"));
    if !path.try_exists()? {
        return Ok(None);
    }
    Ok(Some(serde_json::from_slice(&fs::read(path)?)?))
}
pub fn report(root: &Path, raw: &[u8], arch: &str) -> Result<Option<String>> {
    load_catalog(root, arch)?
        .map(|c| decode(raw, &c).map(|r| format_report(&r)))
        .transpose()
}
pub fn dispatch(root: &Path, request: &Value) -> Result<Value> {
    let a = request.get("kwargs").unwrap_or(request);
    let op = crate::verdict::string(request, "op")?;
    let raw = || -> Result<Vec<u8>> {
        if let Some(s) = a.get("raw").and_then(Value::as_str) {
            s.chars()
                .map(|c| {
                    u8::try_from(c as u32).map_err(|_| {
                        Error::new("debugcon raw string is not Latin-1; supply a byte list")
                    })
                })
                .collect()
        } else {
            Ok(serde_json::from_value(
                a.get("raw").cloned().ok_or("missing debugcon raw bytes")?,
            )?)
        }
    };
    Ok(match op {
        "verdict_debugcon_load_catalog" => {
            json!(load_catalog(root, crate::verdict::string(a, "arch")?)?)
        }
        "verdict_debugcon_decode" => json!(decode(
            &raw()?,
            a.get("catalog").ok_or("missing debugcon catalog")?
        )?),
        "verdict_debugcon_format" => json!(format_report(&serde_json::from_value(
            a.get("result").cloned().ok_or("missing debugcon result")?
        )?)),
        "verdict_debugcon_report" => {
            json!(report(root, &raw()?, crate::verdict::string(a, "arch")?)?)
        }
        _ => return Err(Error::new(format!("unknown debugcon operation {op}"))),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sequential_and_hex_delimiters() {
        let c = json!({"trace_points":[{"char":"A","function":"a","file":"a.rs","line":1,"description":"first"},{"char":"!","function":"err","file":"a.rs","line":2,"description":"err"},{"char":"[","function":"open","file":"a.rs","line":3,"description":"open"},{"char":"]","function":"close","file":"a.rs","line":4,"description":"close"},{"char":"B","function":"b","file":"a.rs","line":5,"description":"last"}],"hex_dump_delimiters":[{"open":"[","close":"]","description":"register"}]});
        let d = decode(b"noise\x01A[ab\xffcd]B", &c).unwrap();
        assert_eq!(d.milestones_reached.len(), 4);
        assert_eq!(d.hex_dumps["register"], "abcd");
        assert!(d.next_expected.is_none());
        assert!(format_report(&d).contains("LAST REACHED: 'B' = b()"));
        assert!(format_report(&decode(&[], &c).unwrap()).contains("NO OUTPUT"));
    }
}
