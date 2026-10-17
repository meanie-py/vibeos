use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::LazyLock;

pub struct PtrHighwordMask;
const MASK: &str = r"&\s*0x[fF]{4}_?0{4}(?:u32|u64|usize)?\s*\)\s*[!=]=\s*0(?:u32|u64|usize)?";
const TYPES: &str = r"(?:usize|isize|ULONG_PTR|UINT_PTR|DWORD_PTR|LONG_PTR|INT_PTR|SIZE_T)";
#[cfg(test)]
use crate::lexical::strip_noise as blank_noise;
#[cfg(test)]
fn scan(text: &str) -> Vec<(usize, String)> {
    scan_code(&blank_noise(text))
}
fn scan_code(stripped: &str) -> Vec<(usize, String)> {
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(MASK).unwrap());
    static CASTS: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(&format!(r"\bas\s+{}\b", TYPES)).unwrap());
    static BARE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*([A-Za-z_]\w*)\s*$").unwrap());
    let mut out = vec![];
    for (ln, line) in stripped.split('\n').enumerate() {
        for m in RE.find_iter(line) {
            if line.as_bytes().get(m.end()).is_some_and(|b| {
                b.is_ascii_alphanumeric() || *b == b'_' || *b == b'x' || *b == b'X'
            }) {
                continue;
            }
            let amp = m.start();
            let mut d = 0i32;
            let mut open = None;
            for j in (0..amp).rev() {
                match line.as_bytes()[j] {
                    b')' => d += 1,
                    b'(' => {
                        if d == 0 {
                            open = Some(j);
                            break;
                        } else {
                            d -= 1
                        }
                    }
                    _ => (),
                }
            }
            let Some(o) = open else { continue };
            let operand = &line[o + 1..amp];
            let typed = CASTS.is_match(operand)
                || BARE.captures(operand).is_some_and(|c| {
                    let id = &c[1];
                    Regex::new(&format!(r"\b{}\s*:\s*{}\b", regex::escape(id), TYPES))
                        .unwrap()
                        .is_match(&stripped)
                });
            if typed {
                out.push((
                    ln + 1,
                    format!("{}{}", operand.trim(), &line[amp..m.end()])
                        .trim()
                        .to_string(),
                ));
            }
        }
    }
    out
}
impl Rule for PtrHighwordMask {
    fn name(&self) -> &'static str {
        "scan_ptr_highword_mask"
    }
    fn needs_sdk_rust(&self) -> bool {
        true
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        Ok(c.sdk_rust_files().flat_map(|f|scan_code(f.rust_noise()).into_iter().map(move |(n,e)|format!("{}:{}: PTR_HIGHWORD_MASK `{}` tests bits 16-31 only; on 64-bit a real pointer at ...0000_xxxx passes. Write `p & !0xFFFF == 0`.",f.path.display(),n,e))).collect())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn integer_atom_shape_fires() {
        assert_eq!(
            scan(
                "pub fn check(ptr_bits: usize) -> u16 {\n    if (ptr_bits & 0xFFFF_0000) == 0 {\n"
            )
            .iter()
            .map(|x| x.0)
            .collect::<Vec<_>>(),
            vec![2]
        );
    }
    #[test]
    fn bare_u32_identifier_does_not_fire() {
        assert!(scan("fn f(rop4: u32) -> bool {\n    (rop4 & 0xFFFF0000) == 0\n}\n").is_empty());
    }
    #[test]
    fn resource_name_shape_fires() {
        assert_eq!(
            scan("    if ((name as usize) & 0xFFFF0000) == 0 {\n").len(),
            1
        );
    }
    #[test]
    fn not_equal_ulong_ptr_fires() {
        assert_eq!(
            scan("let x = ((p as ULONG_PTR) & 0xffff0000u64) != 0;\n").len(),
            1
        );
    }
    #[test]
    fn untyped_rop_does_not_fire() {
        assert!(scan("    if (rop4 & 0xFFFF0000) == 0 { 1 } else { 0 }\n").is_empty());
    }
    #[test]
    fn version_dword_does_not_fire() {
        assert!(scan(
            "    if ((CmNtCSDVersion & 0xFFFF0000) == 0) && (CmNtCSDReleaseType == 1) {\n"
        )
        .is_empty());
    }
    #[test]
    fn callproc_equality_form_does_not_fire() {
        assert!(scan("    ((v as usize & 0xFFFF0000) == 0xFFFF0000) as BOOL\n").is_empty());
    }
    #[test]
    fn fixed_form_does_not_fire() {
        assert!(scan("    if (name as usize) & !0xFFFF == 0 {\n").is_empty());
    }
    #[test]
    fn comment_quoting_c_does_not_fire() {
        assert!(scan(
            "// C: if (!((ULONG_PTR)AtomName & 0xFFFF0000)) -- (x as usize & 0xFFFF0000) == 0\n"
        )
        .is_empty());
    }
    #[test]
    fn tokenizer_matches_lexical_controls() {
        let source = "'x' \"http://ignored\" r#\"ignored /* text */\"# /* outer /* nested */ end */ &'static str\nlet ptr_bits: usize = 0;\n(ptr_bits & 0xFFFF0000) == 0";
        let clean = blank_noise(source);
        assert_eq!(clean.matches('\n').count(), source.matches('\n').count());
        assert!(clean.contains("&'static str"));
        assert_eq!(scan(source).len(), 1);
        let unicode = "// λ\n(name as usize & 0xFFFF0000) == 0";
        assert_eq!(scan(unicode).len(), 1);
    }
}
