//! `core::ptr::read` of an element an atl_rust container still owns.
//!
//! C++ copies an element out of a container (`m_Fonts.Add(fonts.GetNext(it))`, the copy
//! constructor). A translation that writes `ptr::read(fonts.GetNext(&mut it))` makes a bitwise
//! duplicate and leaves the container owning the original too, so when the container drops it
//! frees what the copy still points at, and the copy's own drop frees it again. fontext's font
//! cache did exactly that: every font name was freed memory the moment the Fonts folder opened.
//! Move the element out (`RemoveHead`), or `clone()` it. Suppress a verified false positive
//! (a `Copy` element) with `// nolint: container-ptr-read (reason)` on the line.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::LazyLock;

pub struct ContainerPtrRead;
impl Rule for ContainerPtrRead {
    fn name(&self) -> &'static str {
        "scan_container_ptr_read"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut findings = Vec::new();
        for file in corpus.rust_files() {
            for (line, text) in scan(&file.text) {
                findings.push(format!(
                    "{}:{line}: `{text}` duplicates an element its container still owns; the \
                     container's drop frees it under the copy (move it out with RemoveHead, or \
                     clone it)",
                    file.path.display()
                ));
            }
        }
        findings.sort();
        Ok(findings)
    }
}

static RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\bptr::read\(\s*(?:\*\s*)?[\w.()*\s]*?\.(?:GetNext|GetPrev|GetAt|GetHead|GetTail|GetValueAt|GetKeyAt)\(",
    )
    .unwrap()
});

fn scan(text: &str) -> Vec<(usize, String)> {
    text.lines()
        .enumerate()
        .filter(|(_, l)| !l.trim_start().starts_with("//") && !l.contains("nolint: container-ptr-read"))
        .filter_map(|(i, l)| RE.find(l).map(|m| (i + 1, m.as_str().trim().to_string())))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::scan;

    #[test]
    fn a_read_out_of_a_list_is_found_and_a_move_is_not() {
        let bad = "    (*this_).m_Fonts.Add(core::ptr::read(fonts.GetNext(&mut it)));\n";
        assert_eq!(scan(bad).len(), 1);
        let good = "    while let Some(info) = fonts.RemoveHead() { m.Add(info); }\n";
        assert!(scan(good).is_empty());
        assert!(scan("let v = core::ptr::read(p);\n").is_empty());
        let quiet = "x(ptr::read(a.GetAt(0))); // nolint: container-ptr-read (u32 is Copy)\n";
        assert!(scan(quiet).is_empty());
    }
}
