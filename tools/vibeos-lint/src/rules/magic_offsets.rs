use crate::{Corpus, Rule, SourceSet};
use regex::Regex;
use std::sync::LazyLock;

pub struct MagicOffsets;
const SOURCES: &[SourceSet] =
    &[
        SourceSet::tree("magic-offset-rust", &["sdk/lib/rust"], &["rs"], &[])
            .with_pruning(&["target", ".responses"])
            .with_inputs(),
    ];
static SKIP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(bindgen_[a-z0-9_]+\.rs$|offset_check[a-z0-9_]*\.rs$|size_check[a-z0-9_]*\.rs$|/target/|/\.responses/)").unwrap()
});
static NOLINT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"//\s*nolint(?::\s*magic-offset\b|\s*$|:?\s*\()").unwrap());
static B: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"as\s+usize\s*[+\-]\s*(0x[0-9A-Fa-f_]+|\d[\d_]{1,})(?:[ui](?:8|16|32|64|128|size))?\b.*\bas\s+\*(?:const|mut)\b").unwrap()
});
static C: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\.byte_(?:add|offset|sub)\s*\(\s*(0x[0-9A-Fa-f_]+|\d[\d_]*)(?:[ui](?:8|16|32|64|128|size))?\b").unwrap()
});
static D: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:\*(?:const|mut)\s+u8\)?\s*\.\s*(?:add|offset|sub)|cast::<u8>\(\)\s*\.\s*(?:add|offset|sub))\s*\(\s*(0x[0-9A-Fa-f]+\b|\d{2,}\b|[A-Za-z_][A-Za-z0-9_]*(?:::)?[A-Z][A-Z0-9_]{2,}\b)").unwrap()
});
static E: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\.\s*(?:add|offset|sub)\s*\(\s*0x[0-9A-Fa-f_]+(?:[ui](?:8|16|32|64|128|size))?\b")
        .unwrap()
});
impl Rule for MagicOffsets {
    fn name(&self) -> &'static str {
        "scan_magic_offsets"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        SOURCES
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for f in c.sources("magic-offset-rust") {
            let p = f.path.to_string_lossy().replace('\\', "/");
            if SKIP.is_match(&p) {
                continue;
            }
            let mut block = false;
            for (i, line) in f.text.lines().enumerate() {
                let stripped = line.trim();
                if block {
                    if stripped.contains("*/") {
                        block = false
                    }
                    continue;
                }
                if stripped.starts_with("/*") && !stripped.contains("*/") {
                    block = true;
                    continue;
                }
                if stripped.starts_with("//") || NOLINT.is_match(line) {
                    continue;
                }
                let code = line.split_once("//").map_or(line, |x| x.0);
                let found = if let Some(m) = B.captures(code) {
                    let l = &m[1];
                    if !l.starts_with("0x") && l.replace('_', "").parse::<u64>().unwrap_or(0) < 8 {
                        None
                    } else {
                        Some("PTR_RECAST_ARITH")
                    }
                } else if C.is_match(code) {
                    Some("BYTE_ADD")
                } else if D.is_match(code) {
                    Some("U8_BYTE_ARITH")
                } else if E.is_match(code) {
                    Some("TYPED_PTR_HEX_ARITH")
                } else {
                    None
                };
                if let Some(shape) = found {
                    out.push(format!(
                        "[{shape}] {}:{}: {}",
                        f.path.display(),
                        i + 1,
                        stripped.chars().take(160).collect::<String>()
                    ));
                }
            }
        }
        out.sort();
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Corpus, SourceFile};
    use std::path::PathBuf;
    fn run(s: &str) -> Vec<String> {
        let c = Corpus::new(
            PathBuf::from("."),
            vec![SourceFile::new(
                PathBuf::from("sdk/lib/rust/x/src/lib.rs"),
                s.into(),
            )],
        );
        MagicOffsets.run(&c).unwrap()
    }
    #[test]
    fn original_controls_and_shapes() {
        assert!(
            run("pub unsafe fn f(p:*const u8){ *(p.byte_add(0x18usize) as *const u64); }\n")
                .iter()
                .any(|x| x.contains("BYTE_ADD"))
        );
        assert!(
            run("pub unsafe fn f(p:*const u8){ *(p.byte_add(0x18usize) as *const u64); }\n")
                .iter()
                .any(|x| x.contains("[BYTE_ADD]"))
        );
        assert!(
            run("pub unsafe fn f(p:*const u8){ *(p.byte_add(0x18usize) as *const u64); }\n").len()
                == 1
        );
        assert!(run("pub unsafe fn f(p:*const Thing){ *((p as *const u8).byte_add(core::mem::offset_of!(Thing, field)) as *const u64); }\n").is_empty());
    }
    #[test]
    fn comments_nolint_and_paths() {
        assert!(run("/*\n p.byte_add(20);\n*/\n// p.byte_add(20)\nlet x=p.byte_add(20); // nolint: magic-offset\n").is_empty());
    }
}
