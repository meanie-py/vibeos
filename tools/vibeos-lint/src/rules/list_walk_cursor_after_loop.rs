//! Flag wine/list.h cursor values read after a walk can run off the end.
use crate::{Corpus, Rule, SourceSet};
use regex::{Captures, Regex};
use std::sync::LazyLock;

pub struct ListWalkCursorAfterLoop;
const INPUTS: &[SourceSet] = &[SourceSet::tree(
    "list-cursor-after-loop-rust",
    &["sdk/lib/rust"],
    &["rs"],
    &[],
)
.with_pruning(&["target", ".responses"])
.with_inputs()];

static LEX: LazyLock<Regex> =
    LazyLock::new(|| re(r##"(?s)/\*.*?\*/|//[^\n]*|b?r#*"(?:.|\n)*?"#*|b?"(?:\\.|[^"\\])*""##));
static WHILE: LazyLock<Regex> =
    LazyLock::new(|| re(r"\bwhile\s+(?P<e>[A-Za-z_]\w*)\s*!=\s*(?P<head>[^{}]+?)\s*\{"));
static CONTAINER: LazyLock<Regex> =
    LazyLock::new(|| re(r"offset_of!|CONTAINING_RECORD!|LIST_ENTRY!|\blist_entry\b"));
static FUNCTION: LazyLock<Regex> = LazyLock::new(|| re(r"\bfn\s+([A-Za-z_]\w*)"));
static BREAK: LazyLock<Regex> = LazyLock::new(|| re(r"\bbreak\b"));
static LET_END: LazyLock<Regex> = LazyLock::new(|| re(r"\blet\s+(?:mut\s+)?$"));
static CURSOR_ASSIGN: LazyLock<Regex> = LazyLock::new(|| re(r"([A-Za-z_]\w*)\s*="));
static WORD: LazyLock<Regex> = LazyLock::new(|| re(r"[A-Za-z_]\w*"));
static FLINK: LazyLock<Regex> = LazyLock::new(|| re(r"\bFlink\b|\bBlink\b"));
static NEXT: LazyLock<Regex> = LazyLock::new(|| re(r"\.next\b|\blist_next\s*\("));
const NOLINT: &str = "nolint: list-cursor-after-loop";

fn re(pattern: &str) -> Regex {
    crate::regexes::cached(pattern).expect("valid list-cursor regex")
}
fn mask(text: &str) -> String {
    LEX.replace_all(text, |caps: &Captures<'_>| {
        caps[0]
            .chars()
            .flat_map(|c| {
                if c == '\n' {
                    vec!['\n']
                } else {
                    vec![' '; c.len_utf8()]
                }
            })
            .collect::<String>()
    })
    .into_owned()
}
fn block_end(text: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (offset, ch) in text[open..].char_indices() {
        if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(open + offset);
            }
        }
    }
    None
}
fn has_ident(text: &str, ident: &str) -> bool {
    WORD.find_iter(text).any(|m| m.as_str() == ident)
}
fn assignment_start_ok(text: &str, at: usize) -> bool {
    text[..at]
        .chars()
        .next_back()
        .map_or(true, |c| !(c.is_alphanumeric() || c == '_' || c == '.'))
}
fn scan_text(text: &str) -> Vec<(usize, String, String)> {
    if !text.contains("while") || !CONTAINER.is_match(text) {
        return Vec::new();
    }
    let masked = mask(text);
    let mut out = Vec::new();
    for loop_match in WHILE.captures_iter(&masked) {
        let loop_span = loop_match.get(0).unwrap();
        let entry = loop_match.name("e").unwrap().as_str();
        let open = loop_span.end() - 1;
        let Some(end) = block_end(&masked, open) else {
            continue;
        };
        let body = &masked[open + 1..end];
        let Some(first_break) = BREAK.find(body) else {
            continue;
        };
        let mut selected = None;
        for assign in CURSOR_ASSIGN.captures_iter(body) {
            let whole = assign.get(0).unwrap();
            if !assignment_start_ok(body, whole.start()) || body[whole.end()..].starts_with('=') {
                continue;
            }
            let Some(rhs_end) = body[whole.end()..].find(';').map(|x| whole.end() + x) else {
                continue;
            };
            let rhs = &body[whole.end()..rhs_end];
            let cursor = assign[1].to_owned();
            if !has_ident(rhs, entry) {
                continue;
            }
            if !CONTAINER.is_match(rhs) || cursor == entry {
                break;
            }
            selected = Some((cursor, whole.start()));
            break;
        }
        let Some((cursor, assign_start)) = selected else {
            continue;
        };
        if LET_END.is_match(&body[..assign_start]) {
            continue;
        }
        let reassigned_after_break =
            CURSOR_ASSIGN
                .captures_iter(&body[first_break.end()..])
                .any(|m| {
                    let whole = m.get(0).unwrap();
                    let at = first_break.end() + whole.start();
                    assignment_start_ok(body, at)
                        && !body[first_break.end() + whole.end()..].starts_with('=')
                        && m[1] == cursor
                });
        if reassigned_after_break || FLINK.is_match(body) {
            continue;
        }
        let stepped = CURSOR_ASSIGN.captures_iter(body).any(|m| {
            let whole = m.get(0).unwrap();
            if !assignment_start_ok(body, whole.start()) || &m[1] != entry {
                return false;
            }
            body[whole.end()..]
                .find(';')
                .is_some_and(|semi| NEXT.is_match(&body[whole.end()..whole.end() + semi]))
        });
        if !stepped {
            continue;
        }
        let rest = &masked[end + 1..];
        let mut depth = 0isize;
        let mut stop = rest.len();
        for (i, b) in rest.bytes().enumerate() {
            if b == b'{' {
                depth += 1;
            } else if b == b'}' {
                depth -= 1;
                if depth < 0 {
                    stop = i;
                    break;
                }
            }
        }
        let tail = &rest[..stop];
        let cursor_re = re(&format!(r"{}\b", regex::escape(&cursor)));
        let Some(use_match) = cursor_re
            .find_iter(tail)
            .find(|m| assignment_start_ok(tail, m.start()))
        else {
            continue;
        };
        let before = &tail[..use_match.start()];
        let after = tail[use_match.end()..].trim_start();
        if after.starts_with('=') && !after.starts_with("==") && !LET_END.is_match(before) {
            continue;
        }
        let line = text[..loop_span.start()].matches('\n').count() + 1;
        let src_line = text.lines().nth(line - 1).unwrap_or("");
        if src_line.contains(NOLINT) {
            continue;
        }
        let function = FUNCTION
            .captures_iter(&masked[..loop_span.start()])
            .last()
            .map(|m| m[1].to_owned())
            .unwrap_or_else(|| "?".into());
        out.push((line, function, cursor));
    }
    out
}
impl Rule for ListWalkCursorAfterLoop {
    fn name(&self) -> &'static str {
        "scan_list_walk_cursor_after_loop"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.sources("list-cursor-after-loop-rust") {
            for (line, function, cursor) in scan_text(&file.text) {
                out.push(format!(
                    "{}: {function}: cursor `{cursor}` read after a list walk that can run off the end (line {line})",
                    file.path.display()
                ));
            }
        }
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    const BAD: &str = "unsafe fn rearm(t: *mut obj) {\n    let head = core::ptr::addr_of_mut!(pending);\n    let mut other: *mut obj = core::ptr::null_mut();\n    let mut e = (*head).next;\n    while e != head {\n        other = (e as usize - core::mem::offset_of!(obj, entry)) as *mut obj;\n        if (*t).timeout < (*other).timeout {\n            break;\n        }\n        e = (*e).next;\n    }\n    list_add_before(core::ptr::addr_of_mut!((*other).entry), core::ptr::addr_of_mut!((*t).entry));\n}\n";
    const GOOD: &str = "unsafe fn rearm(t: *mut obj) {\n    let head = core::ptr::addr_of_mut!(pending);\n    let mut other: *mut obj = core::ptr::null_mut();\n    let mut e = (*head).next;\n    while e != head {\n        other = (e as usize - core::mem::offset_of!(obj, entry)) as *mut obj;\n        if (*t).timeout < (*other).timeout {\n            break;\n        }\n        e = (*e).next;\n    }\n    list_add_before(e, core::ptr::addr_of_mut!((*t).entry));\n}\n";

    #[test]
    fn original_control_fixture_bad_and_good() {
        assert_eq!(scan_text(BAD), vec![(5, "rearm".into(), "other".into())]);
        assert!(scan_text(GOOD).is_empty());
    }
    #[test]
    fn guards_for_flink_reassignment_nolint_and_no_tail_use() {
        let flink = BAD.replace("e = (*e).next;", "e = (*e).Flink;");
        let reassigned = BAD.replace(
            "list_add_before(core::ptr::addr_of_mut!((*other).entry),",
            "other = core::ptr::null_mut(); list_add_before(core::ptr::addr_of_mut!((*other).entry),",
        );
        let no_use = BAD.replace(
            "list_add_before(core::ptr::addr_of_mut!((*other).entry),",
            "list_add_before(e,",
        );
        let suppressed = BAD.replace(
            "while e != head {",
            "while e != head { // nolint: list-cursor-after-loop (intentional)",
        );
        assert!(scan_text(&flink).is_empty());
        assert!(scan_text(&reassigned).is_empty());
        assert!(scan_text(&no_use).is_empty());
        assert!(scan_text(&suppressed).is_empty());
    }
}
