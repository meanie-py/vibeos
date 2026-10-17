//! Detect container-of cursors initialized at the head of their terminating list walk.
//! The masking and matching intentionally preserve scan_list_walk_from_head.py semantics.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::LazyLock;

pub struct ListWalkFromHead;

impl Rule for ListWalkFromHead {
    fn name(&self) -> &'static str {
        "scan_list_walk_from_head"
    }

    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.rust_files() {
            for (_, function, cursor) in scan_text(&file.text) {
                out.push(format!(
                    "{}: {}: list walk `{}` starts at the head it stops at (runs 0 times)",
                    file.resolved_path().display(),
                    function,
                    cursor
                ));
            }
        }
        Ok(out)
    }
}

static LEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r###"(?s)/\*.*?\*/|//[^\n]*|b?r#*"(?:.|\n)*?"#*|b?"(?:\\.|[^"\\])*"|b?'(?:\\.|[^\\'\n])'"###).unwrap()
});
static ASSIGN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:\blet\s+(?:mut\s+)?)?\b(?P<var>[A-Za-z_]\w*)\s*(?::[^=;{}]+)?=(?P<rhs>[^;]*);")
        .unwrap()
});
static CONTAINER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"offset_of!|CONTAINING_RECORD!|LIST_ENTRY!|\blist_entry\b").unwrap()
});
static HEAD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"addr_of(?:_mut)?!\(\s*(?P<a>[\w.()*]+?)\s*\)|&raw\s+(?:mut|const)\s+(?P<b>[\w.]+)|&mut\s+(?P<c>[\w.]+)").unwrap()
});
static STEP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\.(?:next|prev|Flink|Blink)\b|\blist_(?:head|tail|next|prev)\s*\(").unwrap()
});
static FUNCTION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bfn\s+([A-Za-z_]\w*)").unwrap());
static WHILE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*while\b").unwrap());
static WORD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\w$").unwrap());

fn mask(text: &str) -> String {
    // Preserve byte positions and newlines, so matches can refer back to source lines.
    let mut bytes = text.as_bytes().to_vec();
    for m in LEX.find_iter(text) {
        for byte in &mut bytes[m.range()] {
            if *byte != b'\n' {
                *byte = b' ';
            }
        }
    }
    String::from_utf8(bytes).expect("masking entire lexical matches preserves UTF-8")
}

fn while_condition(masked: &str, start: usize) -> Option<(usize, &str)> {
    let m = WHILE.find(&masked[start..])?;
    let condition_start = start + m.end();
    let mut depth: isize = 0;
    for (offset, c) in masked[condition_start..].char_indices() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            '{' if depth == 0 => {
                return Some((
                    condition_start - 5,
                    &masked[condition_start..condition_start + offset],
                ));
            }
            _ => {}
        }
    }
    None
}

fn is_word(c: char) -> bool {
    let mut bytes = [0; 4];
    WORD.is_match(c.encode_utf8(&mut bytes))
}

fn names_head(condition: &str, head: &str) -> bool {
    // Python's (?<![\w.])H(?![\w(]) guards must be checked separately:
    // Rust's regex engine deliberately has no lookbehind or lookahead.
    condition.match_indices(head).any(|(at, _)| {
        let before = condition[..at].chars().next_back();
        let after = condition[at + head.len()..].chars().next();
        !before.is_some_and(|c| is_word(c) || c == '.')
            && !after.is_some_and(|c| is_word(c) || c == '(')
    })
}

fn scan_text(text: &str) -> Vec<(usize, String, String)> {
    // Masking cannot create either required token. Avoid tokenizing generated
    // files and unrelated modules that cannot possibly produce a finding.
    if !text.contains("while")
        || ![
            "offset_of!",
            "CONTAINING_RECORD!",
            "LIST_ENTRY!",
            "list_entry",
        ]
        .iter()
        .any(|token| text.contains(token))
    {
        return Vec::new();
    }
    let masked = mask(text);
    let mut out = Vec::new();
    let mut search_at = 0;
    while let Some(m) = ASSIGN.captures_at(&masked, search_at) {
        let whole = m.get(0).unwrap();
        search_at = whole.end();
        let rhs = m.name("rhs").unwrap();
        let equals = rhs.start() - 1;
        if masked[..equals]
            .chars()
            .next_back()
            .is_some_and(|c| "=!<>".contains(c))
            || masked.as_bytes().get(equals + 1) == Some(&b'=')
        {
            // A rejected equals does not consume the candidate in Python: a later
            // assignment before this same semicolon can still match.
            search_at = whole.start() + masked[whole.start()..].chars().next().unwrap().len_utf8();
            continue;
        }
        if !CONTAINER.is_match(rhs.as_str()) || STEP.is_match(rhs.as_str()) {
            continue;
        }
        let heads: Vec<_> = HEAD
            .captures_iter(rhs.as_str())
            .filter_map(|h| {
                ["a", "b", "c"]
                    .into_iter()
                    .find_map(|name| h.name(name).map(|s| s.as_str()))
            })
            .collect();
        if heads.is_empty() {
            continue;
        }
        let Some((at, condition)) = while_condition(&masked, whole.end()) else {
            continue;
        };
        let cursor = m.name("var").unwrap().as_str();
        let cursor_pattern = Regex::new(&format!(r"\b{}\b", regex::escape(cursor))).unwrap();
        if !cursor_pattern.is_match(condition)
            || !heads.iter().any(|head| names_head(condition, head))
        {
            continue;
        }
        let line = text[..at].bytes().filter(|&b| b == b'\n').count() + 1;
        if text
            .lines()
            .nth(line - 1)
            .unwrap_or("")
            .contains("nolint: list-walk-from-head")
        {
            continue;
        }
        let function = FUNCTION
            .captures_iter(&masked[..whole.start()])
            .last()
            .map(|f| f[1].to_owned())
            .unwrap_or_else(|| "?".into());
        out.push((line, function, cursor.to_owned()));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const BAD: &str = r#"
unsafe extern "C" fn IrotIsRunning(data: *const MonikerComparisonData) -> HRESULT {
    rot_entry = (core::ptr::addr_of_mut!(RunningObjectTable) as *mut list as *mut u8)
        .sub(core::mem::offset_of!(rot_entry, entry)) as *const rot_entry;
    while core::ptr::addr_of!((*rot_entry).entry)
        != core::ptr::addr_of_mut!(RunningObjectTable) as *const list
    {
        rot_entry = ((*rot_entry).entry.next as *mut u8).sub(8) as *const rot_entry;
    }
}
"#;

    #[test]
    fn walk_from_head_is_flagged() {
        assert_eq!(
            scan_text(BAD),
            vec![(5, "IrotIsRunning".into(), "rot_entry".into())]
        );
    }

    #[test]
    fn walk_from_head_next_is_clean() {
        let good = BAD.replace(
            "(core::ptr::addr_of_mut!(RunningObjectTable) as *mut list as *mut u8)",
            "(RunningObjectTable.next as *mut u8)",
        );
        assert!(scan_text(&good).is_empty());
    }

    #[test]
    fn nolint_and_comments() {
        let nolint = BAD.replacen(
            "    while core",
            "    while core // nolint: list-walk-from-head (x)",
            1,
        );
        assert!(scan_text(&nolint).is_empty());
        let commented =
            "// e = CONTAINING_RECORD!(addr_of_mut!(H), T, f);\nwhile e != addr_of_mut!(H) {}";
        assert!(scan_text(commented).is_empty());
    }

    #[test]
    fn guards_and_statement_adjacency() {
        for head in ["OtherH", "obj.H", "H()"] {
            assert!(scan_text(&format!(
                "e = CONTAINING_RECORD!(addr_of_mut!(H), T, f); while e != {head} {{}} "
            ))
            .is_empty());
        }
        assert!(scan_text(
            "e = CONTAINING_RECORD!(addr_of_mut!(H), T, f); step(); while e != H {}"
        )
        .is_empty());
        assert!(
            scan_text("e == CONTAINING_RECORD!(addr_of_mut!(H), T, f); while e != H {}").is_empty()
        );
    }
}
