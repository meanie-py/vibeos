//! Optional legacy byte-string check; separate from the registered wide-array rule.
use super::{BinaryContext, CheckOutput};
use regex::Regex;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};
fn decode(body: &str) -> Option<Vec<u8>> {
    let chars: Vec<_> = body.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();
    while i < chars.len() {
        if chars[i] != '\\' {
            out.push(chars[i] as u32 as u8);
            i += 1;
            continue;
        }
        i += 1;
        let c = *chars.get(i)?;
        if c == 'x' {
            let hex: String = chars.get(i + 1..i + 3)?.iter().collect();
            out.push(u8::from_str_radix(&hex, 16).ok()?);
            i += 3;
        } else {
            match c {
                'n' => out.push(10),
                'r' => out.push(13),
                't' => out.push(9),
                '\\' => out.push(92),
                '0' => out.push(0),
                '\'' => out.push(39),
                '"' => out.push(34),
                '\n' => {}
                _ => return None,
            }
            i += 1;
        }
    }
    Some(out)
}
fn uncomment(line: &str, block: &mut bool) -> String {
    let c: Vec<_> = line.chars().collect();
    let mut i = 0;
    let mut string = false;
    let mut out = String::new();
    while i < c.len() {
        if *block {
            if c[i] == '*' && c.get(i + 1) == Some(&'/') {
                *block = false;
                out.push_str("  ");
                i += 2
            } else {
                out.push(' ');
                i += 1
            }
            continue;
        }
        if string {
            out.push(c[i]);
            if c[i] == '\\' && i + 1 < c.len() {
                out.push(c[i + 1]);
                i += 2;
                continue;
            }
            if c[i] == '"' {
                string = false
            }
            i += 1;
            continue;
        }
        if c[i] == '"' {
            string = true;
            out.push(c[i]);
            i += 1;
            continue;
        }
        if c[i] == '/' && c.get(i + 1) == Some(&'/') {
            break;
        }
        if c[i] == '/' && c.get(i + 1) == Some(&'*') {
            *block = true;
            out.push_str("  ");
            i += 2;
            continue;
        }
        out.push(c[i]);
        i += 1;
    }
    out
}
fn facts(text: &str) -> Vec<(usize, &'static str, String, String)> {
    static LITERAL: OnceLock<Regex> = OnceLock::new();
    static OCTAL: OnceLock<Regex> = OnceLock::new();
    let re=LITERAL.get_or_init(||Regex::new(r#"(?P<deref>\*?)b"(?P<body>(?:\\.|[^"\\])*)"(?P<cast>\s*(?:\.as_ptr\(\)\s*)?as\s+(?:PCWSTR|LPCWSTR|PWSTR|LPWSTR|LPWCH|PWCH|PWCHAR|LPCWCH|\*const\ u16|\*mut\ u16))?"#).unwrap());
    let oct = OCTAL.get_or_init(|| Regex::new(r"\\0[0-9][0-9]").unwrap());
    let mut block = false;
    let mut hits = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = uncomment(line, &mut block);
        for m in re.captures_iter(&line) {
            if &m["deref"] == "*" || m.name("cast").is_none() {
                continue;
            }
            let body = &m["body"];
            let Some(bytes) = decode(body) else { continue };
            let (kind, why) = if oct.is_match(body) {
                (
                    "octal-escape",
                    r"`\0`+digits is NUL+literal digits (Rust has no octal)".into(),
                )
            } else if bytes.len() % 2 != 0 {
                (
                    "odd-length",
                    format!(
                        "{} bytes — not a whole number of UTF-16 code units (ANSI cast to wide?)",
                        bytes.len()
                    ),
                )
            } else if !bytes.ends_with(&[0, 0]) {
                (
                    "no-terminator",
                    "wide string does not end in a u16 NUL (00 00)".into(),
                )
            } else {
                continue;
            };
            hits.push((i + 1, kind, format!("b\"{body}\""), why));
        }
    }
    hits
}
fn walk(p: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    if p.is_dir() {
        for e in fs::read_dir(p).map_err(|e| e.to_string())? {
            let e = e.map_err(|e| e.to_string())?;
            let n = e.file_name();
            if e.file_type().map_err(|e| e.to_string())?.is_dir() {
                if n != "target" && n != ".responses" {
                    walk(&e.path(), files)?
                }
            } else if e.path().extension().is_some_and(|s| s == "rs") {
                files.push(e.path())
            }
        }
    } else if p.exists() {
        files.push(p.to_path_buf())
    } else {
        return Err(format!(
            "ERROR: {} not found. Run from vibeos/ root.",
            p.display()
        ));
    }
    Ok(())
}
pub fn run(ctx: &BinaryContext) -> Result<CheckOutput, String> {
    let roots = if ctx.inputs.is_empty() {
        vec![ctx.root.join("sdk/lib/rust")]
    } else {
        ctx.inputs.clone()
    };
    let mut files = Vec::new();
    for p in roots {
        walk(&p, &mut files)?
    }
    files.sort();
    files.dedup();
    let mut found = BTreeMap::new();
    let mut total = 0;
    for f in files {
        let text = fs::read_to_string(&f).map_err(|e| format!("{}: {e}", f.display()))?;
        let hits = facts(&text);
        if !hits.is_empty() {
            total += hits.len();
            found.insert(f, hits);
        }
    }
    if total == 0 {
        return Ok(CheckOutput::new(
            "No mis-encoded UTF-16 byte-string literals found.\n".into(),
            0,
        ));
    }
    let mut out = format!(
        "Found {total} mis-encoded wide byte-string literal(s) in {} file(s):\n\n",
        found.len()
    );
    for (p, hits) in &found {
        out += &format!("  {}:\n", p.display());
        for (line, kind, lit, why) in hits {
            out += &format!("    L{line:<5} [{kind}] {lit}\n            -> {why}\n")
        }
        out.push('\n')
    }
    out += &format!("Total: {total} finding(s) in {} file(s)\n", found.len());
    Ok(CheckOutput::new(out, 1))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wide_cast_binds_to_its_literal() {
        assert!(facts(r#"let a=b"ANSI"; let p = address as PWSTR;"#).is_empty());
        assert_eq!(facts(r#"let x=b"abc" as PWSTR;"#)[0].1, "odd-length");
        assert!(facts(r#"let x=*b"Tag4" as PWSTR;"#).is_empty());
        assert!(facts(r#"let x=b"A\0\0\0".as_ptr() as PCWSTR;"#).is_empty());
    }
    #[test]
    fn escape_classes_and_comments() {
        assert_eq!(facts(r#"let x=b"\012" as PWSTR;"#)[0].1, "octal-escape");
        assert_eq!(facts(r#"let x=b"ab" as *const u16;"#)[0].1, "no-terminator");
        assert!(facts("/* b\"abc\" as PWSTR;\n */ // ignored").is_empty());
        assert_eq!(decode(r"\x41\0\0\0").unwrap(), vec![65, 0, 0, 0]);
        assert!(decode(r"\q").is_none());
    }
}
