use crate::{Corpus, Rule, SourceSet};
use regex::Regex;
use std::collections::BTreeMap;
use std::sync::LazyLock;

pub struct CohabCensus;
const CMAKE_SET: SourceSet = SourceSet::tree("cmake-files", &[""], &["cmake"], &["CMakeLists.txt"])
    .with_pruning(&[
        ".git",
        "build",
        "output",
        "target",
        "__pycache__",
        ".claude",
        "build-*",
    ]);
const BASE_SET: SourceSet = SourceSet::tree(
    "census-baseline",
    &["tools"],
    &["txt"],
    &["cohab-census.txt"],
)
.with_inputs();
const SETS: &[SourceSet] = &[CMAKE_SET, BASE_SET];
static SRC_IN_COMMENT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*#.*?[A-Za-z0-9_${}/.+-]*[A-Za-z0-9_+-]\.(?:c|cpp|cxx|cc|S|s|asm|inc)\b")
        .unwrap()
});
type Census = BTreeMap<(String, String), usize>;
fn normalized(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}
fn scan(c: &Corpus) -> Census {
    let mut out = Census::new();
    for f in c.sources("cmake-files") {
        if !(f.path.file_name().is_some_and(|n| n == "CMakeLists.txt")
            || f.path.extension().is_some_and(|e| e == "cmake"))
        {
            continue;
        }
        let path = f.path.to_string_lossy().replace('\\', "/");
        for line in f.text.lines() {
            if SRC_IN_COMMENT.is_match(line) {
                *out.entry((path.clone(), normalized(line))).or_default() += 1;
            }
        }
    }
    out
}
fn load(s: &str) -> Result<Census, String> {
    let mut out = Census::new();
    for (i, line) in s.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let mut p = line.splitn(3, '\t');
        let f = p.next().unwrap_or("");
        let n = p
            .next()
            .ok_or_else(|| format!("cohab-census.txt:{}: malformed entry", i + 1))?
            .parse::<usize>()
            .map_err(|_| format!("cohab-census.txt:{}: invalid count", i + 1))?;
        let t = p
            .next()
            .ok_or_else(|| format!("cohab-census.txt:{}: missing normalized text", i + 1))?;
        out.insert((f.to_string(), t.to_string()), n);
    }
    Ok(out)
}
fn report(before: &Census, now: &Census) -> Vec<String> {
    let mut out = Vec::new();
    let mut removed = 0;
    let mut added = 0;
    for key in before
        .keys()
        .chain(now.keys())
        .collect::<std::collections::BTreeSet<_>>()
    {
        let b = *before.get(key).unwrap_or(&0);
        let a = *now.get(key).unwrap_or(&0);
        if a == b {
            continue;
        }
        let tag = if a < b {
            removed += 1;
            "CENSUS-REMOVED"
        } else {
            added += 1;
            "CENSUS-ADDED"
        };
        out.push(format!("[{tag}] {}: {b} -> {a}: {}", key.0, key.1));
    }
    if out.is_empty() {
        return out;
    }
    out.push(String::new());
    if removed > 0 {
        out.push(
            "  {removed} REMOVED: a comment-out disappeared. That is the dangerous direction --"
                .replace("{removed}", &removed.to_string()),
        );
        out.push(
            "  it can mean a flipped .c went back into the build beside its Rust twin, which"
                .into(),
        );
        out.push(
            "  is the 2026-09-19 partial flip that froze the guest while smoke passed on all"
                .into(),
        );
        out.push("  three arches. Read each one before accepting it.".into());
    }
    if added > 0 {
        out.push(format!(
            "  {added} ADDED: a new comment-out. Routine -- it is what flipping a"
        ));
        out.push("  module to Rust looks like.".into());
    }
    out.push(String::new());
    out.push("  If every line above is a change you MEANT to make (including one that".into());
    out.push("  arrived by merging another lane's flip), re-record the snapshot:".into());
    out.push("      python3 tools/cohab_census.py --write".into());
    out.push("  and commit tools/cohab-census.txt with that change.".into());
    out
}
impl Rule for CohabCensus {
    fn name(&self) -> &'static str {
        "cohab_census"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        SETS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let Some(b) = c
            .sources("census-baseline")
            .find(|f| f.path.ends_with("tools/cohab-census.txt"))
        else {
            return Ok(vec![format!(
                "[CENSUS-MISSING] {} absent -- run --write to seed it",
                c.root.join("tools/cohab-census.txt").display()
            )]);
        };
        let was = load(&b.text)?;
        Ok(report(&was, &scan(c)))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Corpus, SourceFile};
    use std::path::PathBuf;
    fn f(p: &str, s: &str) -> SourceFile {
        SourceFile::new(PathBuf::from(p), s.into())
    }
    const FLIP: &str = "    #${REACTOS_SOURCE_DIR}/ntoskrnl/lpc/reply.c\n";
    #[test]
    fn comment_line_addition_and_removal_are_counted() {
        let before = Census::new();
        let mut now = Census::new();
        now.insert(("ntoskrnl/ntos.cmake".into(), normalized(FLIP)), 1);
        let o = report(&before, &now);
        assert!(o[0].starts_with("[CENSUS-ADDED] ntoskrnl/ntos.cmake: 0 -> 1:"));
        assert!(o.join("\n").contains("Routine"));
        let r = report(&now, &before);
        assert!(r[0].starts_with("[CENSUS-REMOVED]"));
        assert!(r.join("\n").contains("dangerous direction"));
    }
    #[test]
    fn duplicate_comments_retain_count_and_normalized_key() {
        let c=Corpus::new(PathBuf::from("."),vec![f("ntoskrnl/ntos.cmake","# ${REACTOS_SOURCE_DIR}/ntoskrnl/lpc/reply.c\n#  ${REACTOS_SOURCE_DIR}/ntoskrnl/lpc/reply.c\n"),f("tools/cohab-census.txt","")]);
        let got = scan(&c);
        assert_eq!(got.values().copied().sum::<usize>(), 2);
    }
    #[test]
    fn exact_shape_and_nonmatching_comments() {
        let c = Corpus::new(
            PathBuf::from("."),
            vec![
                f(
                    "x/CMakeLists.txt",
                    "# source.c\n# no filename\n# file.txt\n",
                ),
                f("tools/cohab-census.txt", ""),
            ],
        );
        let got = scan(&c);
        assert_eq!(got.len(), 1);
    }
}
