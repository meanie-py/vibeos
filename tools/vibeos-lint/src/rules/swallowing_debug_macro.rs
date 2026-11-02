//! A debug-print macro whose channel is off, written to SWALLOW its tokens:
//! `macro_rules! DPRINT { ($($t:tt)*) => {{}}; }`.
//!
//! Nothing inside such a call is compiled, so an argument rots silently: 2026-11-01 found
//! 65 calls naming a local that no longer existed, or a `debugstr_w` that was never in
//! scope, across 8 crates of one shared macro alone. Every value computed only for a trace
//! also read as `unused_variables`. The off form forwards to
//! `win32macros_rust::DBG_TYPECHECK!`, whose arguments are type-checked in a never-called
//! closure: checked, never evaluated, never emitted.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::LazyLock;
pub struct SwallowingDebugMacro;
impl Rule for SwallowingDebugMacro {
    fn name(&self) -> &'static str {
        "scan_swallowing_debug_macro"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.rust_files() {
            for (line, name) in scan_text(&file.text) {
                out.push(format!(
                    "{}:{line}: `{name}!` swallows its arguments uncompiled; forward to \
                     `win32macros_rust::DBG_TYPECHECK!`",
                    file.path.display()
                ));
            }
        }
        Ok(out)
    }
}
/// The printf-style debug family, plus `UNREFERENCED_PARAMETER` (its whole job is to use
/// its argument). Asserts and `UNIMPLEMENTED` are deliberately not here.
const NAMES: &[&str] = &[
    "DPRINT",
    "DPRINT2",
    "TRACE",
    "WARN",
    "FIXME",
    "ERR",
    "trace",
    "warn",
    "err",
    "fixme",
    "WINE_TRACE",
    "WINE_ERR",
    "WINE_WARN",
    "WINE_FIXME",
    "ATLTRACE",
    "INFO",
    "KDDBGPRINT",
    "MESSAGE",
    "DBGPRINT",
    "RD_TRACE",
    "DebugTrace",
    "TracePrint",
    "DH_DbgPrint",
    "AFD_DbgPrint",
    "KdPrintEx",
    "ACPI_DEBUG_PRINT",
    "RIPMSG",
    "UNREFERENCED_PARAMETER",
];
static SHAPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"macro_rules!\s*(\w+)\s*\{\s*\(\$\(\$\w+:tt\)\*\)\s*=>\s*(?:\{\{\s*\}\}|\{\s*\(\)\s*\}|\{\s*\})\s*;?\s*\}",
    )
    .unwrap()
});
fn scan_text(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for m in SHAPE.captures_iter(text) {
        let name = &m[1];
        if !NAMES.contains(&name) {
            continue;
        }
        let at = m.get(0).unwrap().start();
        let line = text[..at].bytes().filter(|&b| b == b'\n').count() + 1;
        if text
            .lines()
            .nth(line - 1)
            .unwrap_or("")
            .contains("nolint: swallowing-debug-macro")
        {
            continue;
        }
        out.push((line, name.to_string()));
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dedicated_fixture() {
        let text = "#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }\n\
                    macro_rules! TRACE {\n    ($($t:tt)*) => {\n        ()\n    };\n}\n\
                    macro_rules! UNIMPLEMENTED { ($($arg:tt)*) => {{}}; }\n";
        assert_eq!(
            scan_text(text),
            vec![(1, "DPRINT".into()), (2, "TRACE".into())]
        );
    }
    #[test]
    fn scanner_controls() {
        assert!(scan_text(
            "macro_rules! DPRINT { ($($arg:tt)*) => { ::win32macros_rust::DBG_TYPECHECK!($($arg)*) }; }\n"
        )
        .is_empty());
        assert!(scan_text("macro_rules! ASSERT { ($($arg:tt)*) => {{}}; }\n").is_empty());
    }
}
