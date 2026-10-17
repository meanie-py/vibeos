//! Integer constants silently truncated by a named narrow cast.
use crate::{Corpus, Rule, SourceFile};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
pub struct ConstNarrowingCast;
impl Rule for ConstNarrowingCast {
    fn name(&self) -> &'static str {
        "scan_const_narrowing_cast"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut groups: BTreeMap<PathBuf, Vec<&SourceFile>> = BTreeMap::new();
        for file in corpus.rust_files() {
            groups
                .entry(crate_of(&corpus.root, &file.path))
                .or_default()
                .push(file);
        }
        let mut out = Vec::new();
        for files in groups.values() {
            let mut values = Values::new();
            let codes: Vec<_> = files
                .iter()
                .map(|file| file.masked(&COMMENTS, None, false))
                .collect();
            for text in &codes {
                collect_values(text, &mut values)?;
            }
            for (file, masked) in files.iter().zip(&codes) {
                for (line, what) in scan_text(&file.text, masked, &values) {
                    out.push(format!(
                        "{}:{line}: const-narrowing-cast {what}",
                        file.path.display()
                    ));
                }
            }
        }
        out.sort();
        Ok(out)
    }
}
fn crate_of(root: &Path, file: &Path) -> PathBuf {
    for parent in file.ancestors().skip(1) {
        if root.join(parent).join("Cargo.toml").is_file() {
            return parent.into();
        }
    }
    file.parent().unwrap_or(Path::new("")).into()
}
static COMMENTS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)//[^\n]*|/\*.*?\*/").unwrap());
static CONSTANT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bconst\s+([A-Za-z_]\w*)\s*:\s*[\w:]+\s*=\s*\(?\s*(-?)\s*(0x[0-9a-fA-F_]+|\d[\d_]*)(?:[iu](?:8|16|32|64|128|size))?\s*\)?\s*(?:as\s+[\w:]+\s*)?;").unwrap()
});
static CAST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([A-Z_][A-Z0-9_]{2,})\s+as\s+(u8|UCHAR|BYTE|BOOLEAN|i8|CHAR|CCHAR|u16|USHORT|WORD|WCHAR|i16|SHORT|CSHORT)\b").unwrap()
});
static WORD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\w$").unwrap());
static NOLINT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"nolint:\s*const-narrowing-cast\b").unwrap());
#[cfg(test)]
fn code(text: &str) -> String {
    let mut out = text.as_bytes().to_vec();
    for m in COMMENTS.find_iter(text) {
        for b in &mut out[m.range()] {
            if *b != b'\n' {
                *b = b' ';
            }
        }
    }
    String::from_utf8(out).unwrap()
}

// Python's integer oracle is unbounded. Keep arbitrary-width magnitudes instead
// of silently dropping large constants or overflowing a host integer.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Integer {
    negative: bool,
    limbs: Vec<u32>,
}
impl Integer {
    fn parse(raw: &str, negative: bool) -> Result<Self, String> {
        let raw = raw.replace('_', "");
        let (digits, radix) = if let Some(hex) = raw.strip_prefix("0x") {
            (hex, 16)
        } else {
            if raw.starts_with('0') && raw.chars().any(|c| c != '0') {
                return Err(format!("invalid base-0 integer constant {raw}"));
            }
            (raw.as_str(), 10)
        };
        if digits.is_empty() {
            return Err(format!("invalid integer constant {raw}"));
        }
        let mut limbs = vec![0];
        for digit in digits.chars() {
            let mut carry = digit
                .to_digit(radix)
                .ok_or_else(|| format!("invalid integer constant {raw}"))?
                as u64;
            for limb in &mut limbs {
                carry += *limb as u64 * radix as u64;
                *limb = carry as u32;
                carry >>= 32;
            }
            if carry != 0 {
                limbs.push(carry as u32);
            }
        }
        while limbs.len() > 1 && limbs.last() == Some(&0) {
            limbs.pop();
        }
        Ok(Self {
            negative: negative && limbs != [0],
            limbs,
        })
    }
    fn fits(&self, bits: u32, signed: bool) -> bool {
        let mask = (1u32 << bits) - 1;
        let low = self.limbs[0] & mask;
        if (!self.negative && low == mask) || (self.negative && low == 1) {
            return true;
        }
        if self.limbs.len() > 1 {
            return false;
        }
        if self.negative {
            signed && self.limbs[0] <= 1 << (bits - 1)
        } else {
            self.limbs[0] < 1 << bits
        }
    }
    fn hex(&self) -> String {
        let mut out = if self.negative {
            "-0x".to_owned()
        } else {
            "0x".to_owned()
        };
        out.push_str(&format!("{:x}", self.limbs.last().unwrap()));
        for limb in self.limbs.iter().rev().skip(1) {
            out.push_str(&format!("{limb:08x}"));
        }
        out
    }
}
type Values = BTreeMap<String, BTreeSet<Integer>>;
fn collect_values(masked: &str, values: &mut Values) -> Result<(), String> {
    for m in CONSTANT.captures_iter(masked) {
        values
            .entry(m[1].into())
            .or_default()
            .insert(Integer::parse(&m[3], !m[2].is_empty())?);
    }
    Ok(())
}
fn scan_text(text: &str, masked: &str, values: &Values) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for m in CAST.captures_iter(masked) {
        let whole = m.get(0).unwrap();
        if masked[..whole.start()]
            .chars()
            .next_back()
            .is_some_and(|c| {
                let mut bytes = [0; 4];
                c == '.' || c == ':' || WORD.is_match(c.encode_utf8(&mut bytes))
            })
        {
            continue;
        }
        let Some(vals) = values.get(&m[1]).filter(|v| v.len() == 1) else {
            continue;
        };
        let value = vals.first().unwrap();
        let (bits, signed) = match &m[2] {
            "u8" | "UCHAR" | "BYTE" | "BOOLEAN" => (8, false),
            "i8" | "CHAR" | "CCHAR" => (8, true),
            "u16" | "USHORT" | "WORD" | "WCHAR" => (16, false),
            _ => (16, true),
        };
        if value.fits(bits, signed) {
            continue;
        }
        let line = text[..whole.start()]
            .bytes()
            .filter(|&b| b == b'\n')
            .count()
            + 1;
        if NOLINT.is_match(text.lines().nth(line - 1).unwrap_or("")) {
            continue;
        }
        out.push((line, format!("{} ({}) as {}", &m[1], value.hex(), &m[2])));
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    fn scan(text: &str) -> Vec<(usize, String)> {
        let masked = code(text);
        let mut values = Values::new();
        collect_values(&masked, &mut values).unwrap();
        scan_text(text, &masked, &values)
    }
    #[test]
    fn persistent_flag_control() {
        let bad = "pub const SRB_CLASS_FLAGS_PERSISTANT: u32 = 0x20000000;\npub unsafe fn flush(srb: *mut SRB) {\n    (*srb).SrbFlags |= (SRB_CLASS_FLAGS_PERSISTANT as USHORT) as u32;\n}\n";
        assert_eq!(
            scan(bad),
            vec![(
                3,
                "SRB_CLASS_FLAGS_PERSISTANT (0x20000000) as USHORT".into()
            )]
        );
        let good = "pub const SRB_CLASS_FLAGS_PERSISTANT: u32 = 0x20000000;\npub const EOF: i32 = -1;\npub const BLOCK_UNUSED: u32 = 0xffffffff;\npub unsafe fn flush(srb: *mut SRB) {\n    (*srb).SrbFlags |= SRB_CLASS_FLAGS_PERSISTANT;\n    let weof = EOF as u16;\n    let fill = BLOCK_UNUSED as u8;\n}\n";
        assert!(scan(good).is_empty());
    }
    #[test]
    fn ambiguity_comments_nolint_and_qualified_names() {
        assert!(scan("const FLAG: u32 = 999; const FLAG: u32 = 1; FLAG as u8;").is_empty());
        assert!(scan("const FLAG: u32 = 999; // FLAG as u8\ncrate::FLAG as u8; obj.FLAG as u8;\nFLAG as u8; // nolint: const-narrowing-cast").is_empty());
    }
    #[test]
    fn unbounded_integer_and_signed_limits() {
        let large = Integer::parse("0x10000000000000000000000000000000100", false).unwrap();
        assert!(!large.fits(8, false));
        assert_eq!(large.hex(), "0x10000000000000000000000000000000100");
        assert!(Integer::parse("128", true).unwrap().fits(8, true));
        assert!(!Integer::parse("129", true).unwrap().fits(8, true));
        assert!(Integer::parse("257", true).unwrap().fits(8, false));
    }
}
