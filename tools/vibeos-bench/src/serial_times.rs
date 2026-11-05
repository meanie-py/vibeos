//! Join raw serial bytes to arrival times, and report latest-boot silences.
use regex::Regex;
use serde_json::{json, Value};
use std::{fs, io, path::Path};

pub const BOOT: &str = r"bootblock starting|FreeLoader v";
/// Complete LF-terminated lines, stamped with the chunk containing the first byte.
pub fn load(run: &Path) -> crate::Result<Vec<(f64, String)>> {
    let data = fs::read(run.join("serial.bin"))?;
    let path = run.join("serial.times");
    let text = fs::read_to_string(&path).map_err(|e| {
        if e.kind() == io::ErrorKind::NotFound {
            io::Error::new(
                e.kind(),
                format!(
                    "serial_times: {} missing (capture predates timing); restart the listener",
                    path.display()
                ),
            )
        } else {
            e
        }
    })?;
    let mut times = Vec::<(usize, i64)>::new();
    for row in text.lines() {
        let parts: Vec<_> = row.split_whitespace().collect();
        if parts.len() == 2 {
            let offset = parts[0].parse().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "invalid serial.times offset")
            })?;
            let stamp = parts[1].parse().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "invalid serial.times timestamp")
            })?;
            times.push((offset, stamp));
        }
    }
    if times.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "serial_times: serial.times is empty",
        )
        .into());
    }
    let mut out = Vec::new();
    let mut start = 0;
    for (end, _) in data.iter().enumerate().filter(|(_, b)| **b == b'\n') {
        let index = times
            .partition_point(|(offset, _)| *offset <= start)
            .saturating_sub(1);
        let line: String = data[start..end]
            .iter()
            .filter(|b| **b != b'\r')
            .map(|b| char::from(*b))
            .collect();
        out.push((times[index].1 as f64 / 1000.0, line));
        start = end + 1;
    }
    Ok(out)
}
fn prefix(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}
pub fn report(
    lines: &[(f64, String)],
    pattern: &str,
    gaps: f64,
    all_lines: bool,
) -> crate::Result<String> {
    let regex = Regex::new(pattern)?;
    let first = lines
        .iter()
        .rposition(|(_, s)| regex.is_match(s))
        .unwrap_or(0);
    let lines = &lines[first..];
    if lines.is_empty() {
        return Ok(String::new());
    }
    let t0 = lines[0].0;
    let mut out = String::new();
    if all_lines {
        for (t, s) in lines {
            out.push_str(&format!("{:8.2}  {}\n", t - t0, s));
        }
        return Ok(out);
    }
    out.push_str(&format!(
        "{} lines over {:.1} s from: {}\n",
        lines.len(),
        lines.last().unwrap().0 - t0,
        prefix(&lines[0].1, 100)
    ));
    for pair in lines.windows(2) {
        let (ta, sa) = &pair[0];
        let (tb, sb) = &pair[1];
        if tb - ta >= gaps {
            out.push_str(&format!(
                "\n{:7.1} s silent at +{:.1} s\n   before: {}\n   after:  {}\n",
                tb - ta,
                ta - t0,
                prefix(sa, 150),
                prefix(sb, 150)
            ));
        }
    }
    Ok(out)
}
/// Metadata bridge for the legacy Python API and CLI adapter.
pub fn dispatch(req: &Value) -> crate::Result<Value> {
    let run = req
        .get("run")
        .and_then(Value::as_str)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "serial_times requires run"))?;
    let lines = load(Path::new(run))?;
    if req.get("op").and_then(Value::as_str) == Some("bench_serial_times_load") {
        return Ok(json!(lines));
    }
    let pattern = req.get("from").and_then(Value::as_str).unwrap_or(BOOT);
    let gaps = req.get("gaps").and_then(Value::as_f64).unwrap_or(10.0);
    let all_lines = req.get("lines").and_then(Value::as_bool).unwrap_or(false);
    Ok(json!({"output": report(&lines, pattern, gaps, all_lines)?}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn report_selects_last_boot_and_formats_gap() {
        let l = vec![
            (1., "bootblock starting".into()),
            (2., "old".into()),
            (3., "FreeLoader v1".into()),
            (15., "next".into()),
        ];
        assert_eq!(report(&l, BOOT, 10., false).unwrap(), "2 lines over 12.0 s from: FreeLoader v1\n\n   12.0 s silent at +0.0 s\n   before: FreeLoader v1\n   after:  next\n");
    }
}
