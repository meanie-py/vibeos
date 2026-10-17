//! Retired field-manual counters and duplicate numbered documentation headings.
use crate::{Corpus, Rule, SourceSet};
use regex::Regex;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::LazyLock;

pub struct DocSections;
const FIELD: &str = "docs/engineering/translation-debugging-field-manual.md";
static HEAD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^#{2,4}\s+(\d+\.\d+[a-z]?(?:\.\d+)*)").unwrap());
fn headings(text: &str) -> BTreeMap<String, Vec<usize>> {
    let mut out = BTreeMap::<String, Vec<usize>>::new();
    let mut fenced = false;
    for (i, line) in text.lines().enumerate() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        let Some(hit) = HEAD.captures(line) else {
            continue;
        };
        let number = hit.get(1).unwrap();
        if line[number.end()..]
            .chars()
            .next()
            .is_some_and(|ch| ch.is_alphanumeric() || ch == '_' || ch == '.')
        {
            continue;
        }
        out.entry(number.as_str().to_owned())
            .or_default()
            .push(i + 1);
    }
    out
}
fn findings(path: &Path, text: &str, retired: bool) -> Vec<String> {
    let numbered = headings(text);
    let mut out = Vec::new();
    let mut count = 0;
    if retired {
        for (number, lines) in &numbered {
            for line in lines {
                count += 1;
                out.push(format!(
                    "{}:{line}: NEW NUMBERED SECTION `{number}` — the counter is retired",
                    path.display()
                ));
            }
        }
        if !numbered.is_empty() {
            out.extend(
                [
                    "    Don't mint a section number. Put the entry in the topical section its",
                    "    symptom belongs to and title it by the symptom; a heading is its own id.",
                    "    Sequential numbering caused five same-number collisions in one week and",
                    "    made every lane's merge conflict at end-of-file. Existing `§9.N` tokens",
                    "    are frozen inline tags for old cross-references, not a series to extend.",
                ]
                .into_iter()
                .map(str::to_owned),
            );
        }
    }
    for (number, lines) in &numbered {
        if lines.len() > 1 {
            count += 1;
            out.push(format!(
                "{}: DUPLICATE section {number} at lines {}",
                path.display(),
                lines
                    .iter()
                    .map(usize::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            out.push(format!("    Two lanes picked {number} independently. A duplicate merges CLEANLY, so nothing else will tell you: every cross-reference to {number} is now ambiguous. Merge the two entries, or retitle the later one by its symptom."));
        }
    }
    if count > 0 {
        out.push(format!("Total: {count} section-numbering finding(s)."));
    }
    out
}
impl Rule for DocSections {
    fn name(&self) -> &'static str {
        "scan_doc_sections"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        const SETS: &[SourceSet] =
            &[SourceSet::tree("doc-sections", &[FIELD], &["md"], &[]).with_inputs()];
        SETS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let docs: Vec<_> = c
            .sources("doc-sections")
            .filter(|f| f.path.extension().is_some_and(|ext| ext == "md"))
            .collect();
        if docs.is_empty() {
            return Err(format!("{FIELD}: MISSING"));
        }
        // The CLI normally selects one document; aggregate the legacy summary
        // across explicit documents, rather than invent one summary per file.
        let mut out = Vec::new();
        let mut total = 0;
        for doc in docs {
            let mut rows = findings(
                &doc.path,
                &doc.text,
                doc.resolved_path() == Path::new(FIELD),
            );
            if rows.last().is_some_and(|s| s.starts_with("Total: ")) {
                let last = rows.pop().unwrap();
                total += last
                    .split_whitespace()
                    .nth(1)
                    .unwrap()
                    .parse::<usize>()
                    .map_err(|e| e.to_string())?;
            }
            out.extend(rows);
        }
        if total > 0 {
            out.push(format!("Total: {total} section-numbering finding(s)."));
        }
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn duplicate_control_and_retired_guidance() {
        let text="# Manual\n\n### 9.121 (2026-07-25) Something earlier\n\nBody.\n\n### 9.122 (2026-07-25) Lane A appended here\n\nBody.\n\n### 9.122 (2026-07-24) Lane B independently picked the same number\n\nBody.\n";
        let ordinary = findings(Path::new("bad.md"), text, false);
        assert_eq!(
            ordinary[0],
            "bad.md: DUPLICATE section 9.122 at lines 7, 11"
        );
        assert_eq!(
            ordinary.last().unwrap(),
            "Total: 1 section-numbering finding(s)."
        );
        let retired = findings(Path::new(FIELD), text, true);
        assert_eq!(retired.len(), 11);
        assert_eq!(
            retired.last().unwrap(),
            "Total: 4 section-numbering finding(s)."
        );
    }
    #[test]
    fn suffixes_sublevels_boundaries_and_fenced_noise() {
        let text="### 9.6c First\n#### 9.6c.1 Second\n#### 9.6c.2 Third\n```\n### 9.6c ignored\n```\n### 9.6cats invalid\n### 9.6. bad\n";
        assert!(findings(Path::new("good.md"), text, false).is_empty());
        assert_eq!(headings(text).len(), 3);
    }
}
