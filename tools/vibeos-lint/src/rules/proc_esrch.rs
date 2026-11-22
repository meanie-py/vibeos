//! A `/proc/<pid>` read guarded by `except FileNotFoundError` alone.
//!
//! A process reaped between `open()` and `read()` of its `/proc/<pid>/...` file fails the
//! READ with ESRCH, which Python raises as `ProcessLookupError`, not `FileNotFoundError`.
//! The handler that says "the process is gone" then never runs and the caller crashes on
//! a process that is, in fact, gone. Four copies of this race were patched by hand
//! (957a48e2784 test_native_vm, 4536161b6d7 ×3, b895e8fddbf orchestrate_native, 76c1abdf9ad
//! buildbench) before this rule. Catch `(FileNotFoundError, ProcessLookupError)`, or
//! `OSError`, which covers both.
use crate::{Corpus, Rule, SourceSet};
use regex::Regex;
use std::sync::LazyLock;

pub struct ProcEsrch;
static INPUTS: &[SourceSet] =
    &[
        SourceSet::tree("proc_esrch_python", &["tools", "tests"], &["py"], &[])
            .with_pruning(&["target"])
            .with_inputs(),
    ];

impl Rule for ProcEsrch {
    fn name(&self) -> &'static str {
        "scan_proc_esrch"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let mut files: Vec<_> = c.sources("proc_esrch_python").collect();
        files.sort_by(|a, b| a.path.cmp(&b.path));
        let mut out = Vec::new();
        for f in files {
            // test_scanner_controls.py holds this rule's own known-bad fixture as a string.
            if f.path.extension().map_or(true, |e| e != "py")
                || f.path.to_string_lossy().contains("/target/")
                || f.path
                    .file_name()
                    .is_some_and(|n| n == "test_scanner_controls.py")
            {
                continue;
            }
            for line in findings(&f.text) {
                out.push(format!(
                    "{}:{line}: /proc/<pid> read guarded by FileNotFoundError alone \
                     (a process reaped between open and read raises ProcessLookupError)",
                    f.path.display()
                ));
            }
        }
        Ok(out)
    }
}

/// A `/proc/` path naming another process: built from a variable, never `/proc/self`.
static DYNAMIC_PROC: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(
        r#"["']/proc/\{|["']/proc/["']\s*\+|["']/proc/%|Path\(\s*["']/proc["']\s*\)\s*/"#,
    )
    .unwrap()
});
static EXCEPT: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"^(\s*)except\b\s*(.*?)\s*(?:\bas\s+\w+\s*)?:").unwrap()
});
static TRY: LazyLock<Regex> = LazyLock::new(|| crate::regexes::cached(r"^(\s*)try\s*:").unwrap());

/// 1-based lines of each `except` that catches FileNotFoundError but nothing ESRCH falls under,
/// whose `try` body reads another process's `/proc` entry.
fn findings(text: &str) -> Vec<usize> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let Some(m) = EXCEPT.captures(line) else {
            continue;
        };
        let caught = &m[2];
        if !caught.contains("FileNotFoundError")
            || [
                "ProcessLookupError",
                "OSError",
                "Exception",
                "BaseException",
            ]
            .iter()
            .any(|c| caught.contains(c))
        {
            continue;
        }
        let indent = m[1].len();
        // The try this handler belongs to: the nearest `try:` above at the same indent.
        let Some(t) = (0..i)
            .rev()
            .find(|&j| TRY.captures(lines[j]).is_some_and(|c| c[1].len() == indent))
        else {
            continue;
        };
        if lines[t + 1..i].iter().any(|l| DYNAMIC_PROC.is_match(l)) {
            out.push(i + 1);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::findings;

    #[test]
    fn a_pid_read_needs_esrch_covered_and_self_reads_are_exempt() {
        let bad = "def alive(pid):\n    try:\n        s = Path(f\"/proc/{pid}/stat\").read_text()\n    except FileNotFoundError:\n        return False\n";
        assert_eq!(findings(bad), vec![4]);
        for ok in [
            "except (FileNotFoundError, ProcessLookupError):",
            "except OSError:",
            "except Exception as e:",
        ] {
            assert!(
                findings(&bad.replace("except FileNotFoundError:", ok)).is_empty(),
                "{ok}"
            );
        }
        let concat = "try:\n    open('/proc/' + str(pid) + '/stat').read()\nexcept FileNotFoundError:\n    pass\n";
        assert_eq!(findings(concat), vec![3]);
        let own =
            "try:\n    os.readlink('/proc/self/fd/' + fd)\nexcept FileNotFoundError:\n    pass\n";
        assert!(findings(own).is_empty());
        // A handler at a deeper indent belongs to a nested try that reads nothing from /proc.
        let nested = "try:\n    x = 1\n    try:\n        y()\n    except FileNotFoundError:\n        pass\n    z = Path(f'/proc/{p}/stat')\nexcept ValueError:\n    pass\n";
        assert!(findings(nested).is_empty());
    }
}
