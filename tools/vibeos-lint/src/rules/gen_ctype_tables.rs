//! Shared native generation and compiler proof for the ctype table block.
use crate::{Corpus, Rule, SourceSet};
use std::path::Path;
use vibeos_codegen::ctype::{build_block, initializer, island_masks, object_tables, BEGIN, END};
pub struct GenCtypeTables;
const INPUTS: &[SourceSet] = &[SourceSet::tree(
    "gen-ctype-inputs",
    &[
        "dll/win32/msvcrt",
        "sdk/lib/rust/msvcrt-sys/src",
        "sdk/lib/rust/msvcrt-crt-rust/src/msvcrt",
    ],
    &["c", "rs"],
    &[],
)
.with_inputs()];
impl Rule for GenCtypeTables {
    fn needs_workspace_rust(&self) -> bool {
        false
    }
    fn name(&self) -> &'static str {
        "gen_ctype_tables"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let find = |path: &str| {
            corpus
                .sources("gen-ctype-inputs")
                .find(|f| f.path == Path::new(path))
        };
        let c = find("dll/win32/msvcrt/ctype.c")
            .ok_or("gen_ctype_tables: missing dll/win32/msvcrt/ctype.c")?;
        let island = find("sdk/lib/rust/msvcrt-sys/src/bindgen_i386.rs")
            .ok_or("gen_ctype_tables: missing msvcrt-sys bindgen_i386.rs")?;
        let target = find("sdk/lib/rust/msvcrt-crt-rust/src/msvcrt/ctype.rs")
            .ok_or("gen_ctype_tables: missing ctype.rs target")?;
        let masks = island_masks(&island.text)?;
        let narrow_src = initializer(&c.text, "WORD MSVCRT__ctype [257]")?;
        let wide_src = initializer(&c.text, "WORD MSVCRT__wctype[257]")?;
        let compiled = object_tables(&corpus.root)?;
        let block = build_block(&narrow_src, &wide_src, &masks, compiled.as_ref())?;
        let have = target
            .text
            .split_once(BEGIN)
            .and_then(|(_, rest)| rest.split_once(END).map(|(have, _)| have))
            .ok_or_else(|| {
                format!(
                    "gen_ctype_tables: {} has no {BEGIN} .. {END} block",
                    target.path.display()
                )
            })?
            .trim_matches('\n');
        if have.trim() != block.trim() {
            return Ok(vec![format!(
                "MISMATCH: {}: ctype table block is stale vs dll/win32/msvcrt/ctype.c",
                target.path.display()
            )]);
        }
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use vibeos_codegen::ctype::{evaluate, prove};

    fn masks() -> BTreeMap<String, u32> {
        [
            ("_UPPER", 1),
            ("_LOWER", 2),
            ("_DIGIT", 4),
            ("_SPACE", 8),
            ("_PUNCT", 16),
            ("_CONTROL", 32),
            ("_BLANK", 64),
            ("_HEX", 128),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect()
    }

    #[test]
    fn abbreviation_and_version_values_match_python_controls() {
        let got = evaluate(
            &[
                "_S_|_C_".into(),
                "_D_|_H_".into(),
                "_U_|_H_".into(),
                "0".into(),
                "0x0028 | B110".into(),
                "0x0010 | D120".into(),
            ],
            &masks(),
        )
        .unwrap();
        assert_eq!(got, [40, 132, 129, 0, 104, 20]);
    }

    #[test]
    fn initializer_requires_257_entries_and_strips_comments() {
        let short = format!(
            "WORD MSVCRT__ctype [257] = {{{}}};",
            vec!["0"; 256].join(",")
        );
        assert!(initializer(&short, "WORD MSVCRT__ctype [257]")
            .unwrap_err()
            .contains("expected 257"));
        let src = format!(
            "WORD MSVCRT__wctype[257] = {{{}}};",
            (0..257)
                .map(|i| format!("/* {i} */ 0"))
                .collect::<Vec<_>>()
                .join(",")
        );
        let entries = initializer(&src, "WORD MSVCRT__wctype[257]").unwrap();
        assert_eq!(entries.len(), 257);
        assert!(entries.iter().all(|e| e == "0"));
    }

    #[test]
    fn evaluator_rejects_out_of_range_values_and_proof_detects_one_mismatch() {
        assert!(evaluate(&["0x1FFFF".into()], &masks())
            .unwrap_err()
            .contains("does not fit a WORD"));
        let entries = vec!["0".to_owned(); 257];
        let derived = vec![0; 257];
        let mut compiled = derived.clone();
        compiled[65] = 0x81;
        assert!(prove("MSVCRT__ctype", &entries, &derived, &compiled)
            .unwrap_err()
            .contains("do NOT ship"));
        assert!(prove("MSVCRT__ctype", &entries, &derived, &derived).is_ok());
    }
}
