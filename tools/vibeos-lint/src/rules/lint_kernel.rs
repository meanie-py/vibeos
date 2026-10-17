use crate::{Corpus, Rule, SourceFile, SourceSet};
use regex::Regex;
use std::collections::BTreeMap;
use std::sync::LazyLock;

pub struct LintKernel;
const ROOTS: &[&str] = &[
    "sdk/lib/rust/ntoskrnl-rust/src",
    "sdk/lib/rust/lpc-rust/src",
    "sdk/lib/rust/fsrtl-rust/src",
    "sdk/lib/rust/dbgk-rust/src",
    "sdk/lib/rust/wmi-rust/src",
    "sdk/lib/rust/dbgprint-rust/src",
    "sdk/lib/rust/cc-rust/src",
    "sdk/lib/rust/iomgr-rust/src",
    "sdk/lib/rust/kdbg-rust/src",
    "sdk/lib/rust/config-rust/src",
    "sdk/lib/rust/ex-rust/src",
    "sdk/lib/rust/ob-rust/src",
    "sdk/lib/rust/ps-rust/src",
    "sdk/lib/rust/kd-rust/src",
    "sdk/lib/rust/kd64-rust/src",
    "sdk/lib/rust/vf-rust/src",
    "sdk/lib/rust/rtlk-rust/src",
    "sdk/lib/rust/po-rust/src",
    "sdk/lib/rust/inbv-rust/src",
    "sdk/lib/rust/vdm-rust/src",
    "sdk/lib/rust/fstub-rust/src",
    "sdk/lib/rust/se-rust/src",
    "sdk/lib/rust/pnpmgr-rust/src",
];
const SETS: &[SourceSet] = &[SourceSet::tree("kernel-src", ROOTS, &["rs"], &[]).with_inputs()];
static FN: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"fn\s+(\w+)\s*\((.*?)\)").unwrap());
static PARAM: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"(\w+)\s*:\s*(.*)").unwrap());

#[derive(Debug)]
struct Violation {
    line: usize,
    function: String,
    param: String,
    code: String,
}
fn scan(file: &SourceFile) -> Vec<Violation> {
    let path = file.path.to_string_lossy();
    if ["old_translations", ".bak", "shim_exports.rs"]
        .iter()
        .any(|s| path.contains(s))
    {
        return vec![];
    }
    let lines: Vec<_> = file.text.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim() != "#[no_mangle]" {
            i += 1;
            continue;
        }
        let mut j = i + 1;
        let mut declaration = String::new();
        let mut found = false;
        while j < lines.len() && j < i + 10 {
            let t = lines[j].trim();
            if t.starts_with("pub unsafe extern") {
                found = true;
                while j < lines.len() {
                    declaration.push(' ');
                    declaration.push_str(lines[j].trim());
                    if lines[j].contains('{') {
                        break;
                    }
                    j += 1;
                }
                break;
            }
            if t.starts_with("#[") || t.is_empty() {
                j += 1;
                continue;
            } else {
                break;
            }
        }
        if found {
            if let Some(c) = FN.captures(&declaration) {
                let fname = c[1].to_string();
                let mut params = Vec::new();
                for p in c[2].split(',') {
                    if let Some(pm) = PARAM.captures(p.trim()) {
                        let (n, t) = (&pm[1], pm[2].trim());
                        if t.contains("*mut")
                            || (t.starts_with('P')
                                && t[1..].chars().next().is_some_and(char::is_uppercase))
                        {
                            params.push((n.to_string(), t.to_string()));
                        }
                    }
                }
                if !params.is_empty() && j < lines.len() {
                    let body_start = j;
                    let mut depth = 0i32;
                    let mut end = j;
                    for (k, l) in lines.iter().enumerate().skip(j) {
                        depth += l.matches('{').count() as i32 - l.matches('}').count() as i32;
                        if depth <= 0 {
                            end = k;
                            break;
                        }
                    }
                    for (name, _) in params {
                        let pat = crate::regexes::cached(&format!(
                            r"^\s*\*{}\s*=\s*",
                            regex::escape(&name)
                        ))
                        .unwrap();
                        for (k, line) in lines[body_start..=end].iter().enumerate() {
                            let stripped = line.trim();
                            if stripped.starts_with("//")
                                || !pat.is_match(line)
                                || line.contains("write_volatile")
                            {
                                continue;
                            }
                            let code = line.split("//").next().unwrap_or("");
                            if pat.is_match(code) {
                                out.push(Violation {
                                    line: body_start + k + 1,
                                    function: fname.clone(),
                                    param: name.clone(),
                                    code: stripped.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
        i += 1;
    }
    out
}
impl Rule for LintKernel {
    fn name(&self) -> &'static str {
        "lint_kernel"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        SETS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let fs: Vec<_> = c
            .sources("kernel-src")
            .filter(|f| f.path.extension().is_some_and(|e| e == "rs"))
            .collect();
        if fs.is_empty() {
            return Err(
                "no kernel-crate sources found in corpus (expected kernel_crate_roots.py roots)"
                    .into(),
            );
        }
        let mut all: Vec<(String, Violation)> = Vec::new();
        for f in fs {
            for v in scan(f) {
                all.push((f.path.to_string_lossy().into_owned(), v));
            }
        }
        if all.is_empty() {
            return Ok(vec!["No unprotected output pointer writes found.".into()]);
        }
        let mut by: BTreeMap<String, Vec<Violation>> = BTreeMap::new();
        for (p, v) in all {
            by.entry(p).or_default().push(v)
        }
        let count: usize = by.values().map(Vec::len).sum();
        let nfiles = by.len();
        let mut out = vec![
            format!("Found {count} unprotected output pointer writes in {nfiles} files:"),
            String::new(),
        ];
        for (path, vs) in by {
            let short = path.replace("sdk/lib/rust/ntoskrnl-rust/src/", "");
            out.push(format!("  {short}:"));
            for v in vs {
                out.push(format!(
                    "    L{:4} {}(): *{} = ...",
                    v.line, v.function, v.param
                ));
                out.push(format!("           {}", v.code));
            }
            out.push(String::new());
        }
        out.push(format!("Total: {count} violations in {nfiles} files"));
        out.push("Run with --fix to see suggested replacements".into());
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Corpus;
    use std::path::PathBuf;
    fn file(p: &str, s: &str) -> SourceFile {
        SourceFile::new(PathBuf::from(p), s.into())
    }
    const BAD:&str="#[no_mangle]\npub unsafe extern \"system\" fn GetThing(OutValue: *mut u32) {\n    // Unprotected output write — the captures(none) -> ud2 class.\n    *OutValue = 42;\n}\n";
    const GOOD:&str="#[no_mangle]\npub unsafe extern \"system\" fn GetThing(OutValue: *mut u32) {\n    core::ptr::write_volatile(OutValue, 42);\n}\n";
    #[test]
    fn exact_control_bad_and_good() {
        assert_eq!(scan(&file("bad.rs", BAD)).len(), 1);
        assert!(scan(&file("good.rs", GOOD)).is_empty());
    }
    #[test]
    fn exact_lint_stdout_lines_match_the_control() {
        let c = Corpus::new(PathBuf::from("."), vec![file("bad.rs", BAD)]);
        assert_eq!(
            LintKernel.run(&c).unwrap(),
            vec![
                "Found 1 unprotected output pointer writes in 1 files:",
                "",
                "  bad.rs:",
                "    L   4 GetThing(): *OutValue = ...",
                "           *OutValue = 42;",
                "",
                "Total: 1 violations in 1 files",
                "Run with --fix to see suggested replacements"
            ]
        );
    }
    #[test]
    fn only_output_names_and_volatile_writes() {
        let s="#[no_mangle]\npub unsafe extern \"system\" fn f(p: *mut u32, Result: *mut u32) {\n *p = 1;\n *Result = 2;\n}\n";
        let got = scan(&file("x.rs", s));
        assert_eq!(got.len(), 2);
        assert_eq!(
            got.iter().map(|v| v.param.as_str()).collect::<Vec<_>>(),
            ["p", "Result"]
        );
    }
    #[test]
    fn skip_paths_and_line_numbers() {
        let mut f = file("x.rs", BAD);
        assert_eq!(scan(&f)[0].line, 4);
        f.path = PathBuf::from("shim_exports.rs");
        assert!(scan(&f).is_empty());
    }
    #[test]
    fn rule_no_finding_message() {
        let c = Corpus::new(PathBuf::from("."), vec![file("fixture.rs", GOOD)]);
        assert_eq!(
            LintKernel.run(&c).unwrap(),
            ["No unprotected output pointer writes found."]
        );
    }
}
