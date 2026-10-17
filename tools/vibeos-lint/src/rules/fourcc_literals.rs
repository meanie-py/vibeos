//! Detect integer literals that disagree with adjacent FOURCC comments.
use crate::{integers::unsigned_base0 as parse_int, Corpus, Rule};
use regex::Regex;
use std::sync::OnceLock;

pub struct FourccLiterals;
impl Rule for FourccLiterals {
    fn name(&self) -> &'static str {
        "scan_fourcc_literals"
    }
    fn needs_sdk_rust(&self) -> bool {
        true
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.sdk_rust_files() {
            let path = file.path.to_string_lossy();
            for (line_no, line) in file.text.lines().enumerate() {
                for cap in pattern().captures_iter(line) {
                    let written = parse_int(cap.get(1).unwrap().as_str())
                        .map_err(|e| format!("{path}:{}: {e}", line_no + 1))?;
                    let chars: Vec<_> = (2..=5)
                        .map(|i| decode_char(cap.get(i).unwrap().as_str()))
                        .collect::<Result<_, _>>()
                        .map_err(|e| format!("{path}:{}: {e}", line_no + 1))?;
                    let correct = chars
                        .iter()
                        .enumerate()
                        .fold(0u64, |v, (i, (_, c))| v + ((*c as u64) << (8 * i)));
                    if written != correct.to_string() {
                        let text: String = chars.iter().map(|(raw, _)| raw.as_str()).collect();
                        out.push(format!(
                            "{path}:{}: FOURCC_MISFOLDED '{}' written as {}, is {} (0x{:08x})",
                            line_no + 1,
                            text,
                            written,
                            correct,
                            correct
                        ));
                    }
                }
            }
        }
        Ok(out)
    }
}

fn pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b(0[xX][0-9a-fA-F_]+|\d[\d_]*)(?:[iu](?:8|16|32|64|size))?[^\n/]*?//\s*(?:MAKEFOURCC|mmioFOURCC)\(\s*'(\\?.)'\s*,\s*'(\\?.)'\s*,\s*'(\\?.)'\s*,\s*'(\\?.)'\s*\)").unwrap())
}

/// Return the original capture (used in the finding) and Python's `_ord` value.
fn decode_char(s: &str) -> Result<(String, u32), String> {
    // The regex capture is the literal contents, without surrounding quotes.
    let inner = s;
    let value = match inner {
        "\\'" => '\'' as u32,
        "\\\\" => '\\' as u32,
        "\\0" => 0,
        _ => {
            let mut chars = inner.chars();
            let first = chars
                .next()
                .ok_or_else(|| format!("invalid character capture {s:?}"))?;
            if chars.next().is_some() {
                return Err(format!(
                    "Python ord() rejects multi-character FOURCC capture {inner:?}"
                ));
            }
            first as u32
        }
    };
    Ok((inner.to_owned(), value))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(line: &str) -> Vec<(String, String, u64)> {
        pattern()
            .captures_iter(line)
            .map(|cap| {
                let written = parse_int(cap.get(1).unwrap().as_str()).unwrap();
                let chars: Vec<_> = (2..=5)
                    .map(|i| decode_char(cap.get(i).unwrap().as_str()).unwrap())
                    .collect();
                let correct = chars
                    .iter()
                    .enumerate()
                    .fold(0u64, |v, (i, (_, c))| v + ((*c as u64) << (8 * i)));
                let text = chars.iter().map(|(raw, _)| raw.as_str()).collect();
                (text, written, correct)
            })
            .collect()
    }

    #[test]
    fn unbounded_integer_and_base_zero_semantics() {
        let huge = "340282366920938463463374607431768211456";
        assert_eq!(parse_int(huge).unwrap(), huge);
        assert_eq!(
            parse_int("0x100000000000000000000000000000000").unwrap(),
            huge
        );
        assert!(parse_int("00123").is_err());
        assert_eq!(parse_int("００٠").unwrap(), "0");
        assert_eq!(parse_int("١٢٤").unwrap(), "124");
        let corpus = Corpus::new(
            "/fixture".into(),
            vec![crate::SourceFile::new(
                "x.rs".into(),
                format!("{huge} // MAKEFOURCC('A','B','C','D')"),
            )],
        );
        assert_eq!(
            FourccLiterals.run(&corpus).unwrap(),
            vec![format!(
                "x.rs:1: FOURCC_MISFOLDED 'ABCD' written as {huge}, is 1145258561 (0x44434241)"
            )]
        );
    }
    #[test]
    fn the_evr_misfolds_fire_with_the_right_value() {
        let text = "        | 808530521 // MAKEFOURCC('I','M','C','1')\n        | 842094158 // MAKEFOURCC('N','V','1','1')\n";
        let got: Vec<_> = text
            .lines()
            .flat_map(values)
            .filter(|(_, w, c)| *w != c.to_string())
            .collect();
        assert_eq!(
            got,
            vec![
                ("IMC1".into(), "808530521".into(), 826494281),
                ("NV11".into(), "842094158".into(), 825316942)
            ]
        );
    }

    #[test]
    fn correct_folds_in_any_spelling_are_silent() {
        let text = "| 842094158 // MAKEFOURCC('N','V','1','2')\nconst Y: u32 = 0x3231_564e; // MAKEFOURCC('N', 'V', '1', '2')\nx = 808530521u32 as DWORD; // mmioFOURCC('Y','2','1','0')\n";
        assert!(text
            .lines()
            .flat_map(values)
            .all(|(_, w, c)| w == c.to_string()));
    }

    #[test]
    fn literal_without_comment_is_not_candidate() {
        assert!(values("| 808530521 // IMC1").is_empty());
        assert!(values("| 808530521").is_empty());
    }

    #[test]
    fn ord_matches_python_for_ascii_escapes_and_unicode() {
        assert_eq!(decode_char(r"\'").unwrap().1, '\'' as u32);
        assert_eq!(decode_char(r"\\").unwrap().1, '\\' as u32);
        assert_eq!(decode_char(r"\0").unwrap().1, 0);
        assert_eq!(decode_char("é").unwrap().1, 'é' as u32);
        assert!(decode_char(r"\x").is_err());
        assert_eq!(values("0 // MAKEFOURCC('é','A','B','C')")[0].0, "éABC");
    }
}
