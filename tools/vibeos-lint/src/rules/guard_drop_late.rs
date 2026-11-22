//! Detect a lock or IRQL guard that outlives the C's release point.
//!
//! Guard conversions replace `MiReleasePfnLock(OldIrql);` / `KeLowerIrql(x);` with a
//! `// [guard-drop] <the C call>` marker and leave the release to the guard's scope end. That
//! is equivalent only when nothing runs between the marker and the scope end. When the C
//! released early and then did more work, the work now runs under the lock or at the raised
//! IRQL. rmap.rs `MmDeleteSectionAssociation` freed pool under the PFN lock that way (i386
//! SPECIALPOOL=* 0x0F), and MmAccessFault waited for memory at APC_LEVEL.
//!
//! Flags a marker followed, before the block closes or a same-depth `return`/`continue`/
//! `break`, by a function or method call that is not `drop(...)`. Arithmetic, tail
//! expressions and macros (`ASSERT!(...)`) are not work in this sense.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::OnceLock;

pub struct GuardDropLate;
impl Rule for GuardDropLate {
    fn name(&self) -> &'static str {
        "scan_guard_drop_late"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.rust_files() {
            if patterns().skip.is_match(&file.path.to_string_lossy()) {
                continue;
            }
            for (line, call) in scan_text(&file.text) {
                out.push(format!(
                    "{}:{line}: [guard-drop] then {call}( while the guard is still held",
                    file.path.display()
                ));
            }
        }
        Ok(out)
    }
}
struct Patterns {
    strings: Regex,
    call: Regex,
    exit: Regex,
    macro_stmt: Regex,
    skip: Regex,
}
fn patterns() -> &'static Patterns {
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(|| Patterns {
        strings: crate::regexes::cached(r#"b?"(?:\\.|[^"\\])*""#).unwrap(),
        // An identifier directly followed by `(`, not by `!(` (a macro).
        call: crate::regexes::cached(r"([A-Za-z_][A-Za-z0-9_]*)\s*\(").unwrap(),
        exit: crate::regexes::cached(r"^\s*(?:return\b|continue\b|break\b)").unwrap(),
        // A statement that is a macro invocation (ASSERT!, a debug print): its arguments
        // are checks or diagnostics, not work.
        macro_stmt: crate::regexes::cached(r"^\s*[A-Za-z_][\w:]*!\s*[(\[{]").unwrap(),
        skip: crate::regexes::cached(r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$)").unwrap(),
    })
}
const NOT_WORK: &[&str] = &[
    "if", "while", "for", "match", "loop", "return", "Some", "Ok", "Err", "drop", "as",
];
/// The code part of a line: string literals blanked, the comment removed.
fn code_of(line: &str) -> String {
    let s = patterns().strings.replace_all(line, "\"\"");
    match s.find("//") {
        Some(at) => s[..at].to_string(),
        None => s.into_owned(),
    }
}
fn scan_text(text: &str) -> Vec<(usize, String)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let Some(at) = line.find("//") else { continue };
        if !line[at..].contains("[guard-drop]") || !code_of(line).trim().is_empty() {
            continue;
        }
        let mut depth: i32 = 0;
        'scan: for (j, next) in lines.iter().enumerate().skip(i + 1) {
            let code = code_of(next);
            if depth == 0 && patterns().exit.is_match(&code) {
                break;
            }
            let work = if patterns().macro_stmt.is_match(&code) { "" } else { code.as_str() };
            for m in patterns().call.captures_iter(work) {
                let name = m.get(1).unwrap().as_str();
                if name == "drop" {
                    break 'scan;
                }
                if NOT_WORK.contains(&name) {
                    continue;
                }
                out.push((j + 1, name.to_string()));
                break 'scan;
            }
            for c in code.chars() {
                match c {
                    '{' => depth += 1,
                    '}' => depth -= 1,
                    _ => {}
                }
                if depth < 0 {
                    break 'scan;
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    // rmap.rs before the fix: the free runs under the PFN lock.
    const LATE: &str = "fn f() {\n    let g = Guard::acquire();\n    if x {\n        // [guard-drop] MiReleasePfnLock(OldIrql);\n        ExFreeToNPagedLookasideList(l, e);\n        return;\n    }\n}\n";
    // pagfault.rs before the fix: the wait is nested in an `if`, ahead of the `continue`.
    const NESTED: &str = "    // [guard-drop] KeLowerIrql(LockIrql);\n\n    if Status == STATUS_NO_MEMORY {\n        MmRebalanceMemoryConsumersAndWait();\n        continue 'Retry;\n    }\n    return Status;\n}\n";
    const DROPPED: &str = "    // [guard-drop] MiReleasePfnLock(OldIrql);\n    drop(pfn_guard);\n    ExFreeToNPagedLookasideList(l, e);\n";
    const BENIGN: &str = "    {\n        if locked {\n            // [guard-drop] MiReleasePfnLock(OldIrql);\n            i += 1;\n            continue;\n        }\n    }\n    // [guard-drop] KeLowerIrql(x);\n    ASSERT!(KeGetCurrentIrql() == 0);\n    (*Frame).Eax as NTSTATUS\n}\nfn later() { Other(); }\n";
    #[test]
    fn flags_work_after_the_marker() {
        assert_eq!(scan_text(LATE), vec![(5, "ExFreeToNPagedLookasideList".into())]);
        assert_eq!(scan_text(NESTED), vec![(4, "MmRebalanceMemoryConsumersAndWait".into())]);
    }
    #[test]
    fn drop_exit_macros_tail_and_scope_end_are_fine() {
        assert!(scan_text(DROPPED).is_empty());
        assert!(scan_text(BENIGN).is_empty());
    }
}
