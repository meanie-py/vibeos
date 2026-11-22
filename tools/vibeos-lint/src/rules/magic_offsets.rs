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
/// c2rust's spelling of a folded `offsetof()`: the host's 64-bit value as a literal, cast
/// through `c_ulonglong` to a signed offset. Right on amd64 and arm64, wrong on i386 for any
/// field behind a pointer: wined3d's light walks subtracted 232 where i386 needs 228, and
/// ddraw's DDSURFACEDESC2 compare table read every field after lpSurface 4 bytes late.
static F: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:[1-9]\d*)\s+as\s+(?:::)?core::ffi::c_ulonglong\s+as\s+(?:isize|ptrdiff_t)\b")
        .unwrap()
});
/// A container-of by a negated literal, `.offset(-(88 as isize))`: the same fold written
/// through `isize` directly. Two digits or more, so a typed `p.offset(-(1 as isize))` passes.
static G: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\.\s*offset\s*\(\s*-\s*\(\s*(?:[1-9]\d+)\s+as\s+isize\s*\)").unwrap()
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
                } else if F.is_match(code) {
                    Some("C2RUST_FOLDED_OFFSETOF")
                } else if G.is_match(code) {
                    Some("NEG_LITERAL_CONTAINER_OF")
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
    fn c2rust_folded_offsetof() {
        assert!(run("let l = (p as *mut c_char)\n    .offset(-(232 as ::core::ffi::c_ulonglong as isize));\n")
            .iter()
            .any(|x| x.contains("[C2RUST_FOLDED_OFFSETOF]")));
        assert!(run("    offset: 112 as ::core::ffi::c_ulonglong as ptrdiff_t,\n")
            .iter()
            .any(|x| x.contains("[C2RUST_FOLDED_OFFSETOF]")));
        assert!(run("let s = (e as *mut u8).offset(-(88 as isize)) as *mut S;\n")
            .iter()
            .any(|x| x.contains("[NEG_LITERAL_CONTAINER_OF]")));
        assert!(run("let s = (e as *mut u8).offset(-(core::mem::offset_of!(S, entry) as isize)) as *mut S;\n").is_empty());
        assert!(run("let c = *p.offset(-(1 as isize));\nlet z = 0 as ::core::ffi::c_ulonglong as isize;\n").is_empty());
    }
    #[test]
    fn comments_nolint_and_paths() {
        assert!(run("/*\n p.byte_add(20);\n*/\n// p.byte_add(20)\nlet x=p.byte_add(20); // nolint: magic-offset\n").is_empty());
    }
}
