//! `size_of_val(&p)` where `p` is a raw pointer: the size of the POINTER (8), where the C's
//! `sizeof(*p)` meant the pointee. wow64's AFD select marshaller allocated
//! `size_of_val(&P64) + n * size_of_val(&H64)` for a 24-byte header and 16-byte handles,
//! so `select()` overran the 64-bit heap (RtlAllocateHeap faulted on the corrupt free list).
//! The fix is `size_of::<Pointee>()`: `size_of_val(&*p)` would also build a reference from a
//! possibly null or misaligned pointer, which `sizeof` never evaluates.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::OnceLock;

pub struct SizeofPointer;
impl Rule for SizeofPointer {
    fn name(&self) -> &'static str {
        "scan_sizeof_pointer"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.rust_files() {
            for (line, name) in scan_text(&file.text) {
                out.push(format!(
                    "{}:{line}: size_of_val(&{name}) measures the pointer `{name}`, not what it \
                     points to (the C's sizeof(*{name})?)",
                    file.resolved_path().display()
                ));
            }
        }
        Ok(out)
    }
}

struct Patterns {
    lexeme: Regex,
    function: Regex,
    binding: Regex,
    size_of_val: Regex,
}
fn patterns() -> &'static Patterns {
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(|| {
        let re = |s| Regex::new(s).unwrap();
        Patterns {
            lexeme: re(r#"(?s)//[^\n]*|/\*.*?\*/|b?"(?:\\.|[^"\\])*"|'(?:\\.|[^\\'\n])'"#),
            function: re(r"\bfn\s+\w+"),
            binding: re(r"\b([A-Za-z_]\w*)\s*:\s*([^,;=)]+)"),
            size_of_val: re(r"size_of_val\s*\(\s*&\s*(?:mut\s+)?([A-Za-z_]\w*)\s*\)"),
        }
    })
}
/// Comments and string literals blanked to spaces, newlines kept.
fn blank(raw: &str) -> String {
    let mut bytes = raw.as_bytes().to_vec();
    for m in patterns().lexeme.find_iter(raw) {
        for b in &mut bytes[m.start()..m.end()] {
            if *b != b'\n' {
                *b = b' ';
            }
        }
    }
    String::from_utf8(bytes).unwrap()
}
/// Each function's text, from `fn` to the brace closing its body.
fn functions(code: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for m in patterns().function.find_iter(code) {
        let Some(open) = code[m.end()..].find(['{', ';']).map(|i| m.end() + i) else {
            continue;
        };
        if code.as_bytes()[open] == b';' {
            continue; // a declaration
        }
        let mut depth = 0usize;
        for (i, b) in code.bytes().enumerate().skip(open) {
            match b {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        out.push((m.start(), i + 1));
                        break;
                    }
                }
                _ => {}
            }
        }
    }
    out
}
fn scan_text(raw: &str) -> Vec<(usize, String)> {
    let code = blank(raw);
    let mut hits = Vec::new();
    for (start, end) in functions(&code) {
        let body = &code[start..end];
        // Every `name: Type` binding (let, parameter), in order: the one a use sees is the
        // NEAREST before it. A function can rebind a name per `case` block, a pointer in
        // one and a by-value struct in the next (wow64's A32).
        let bindings: Vec<(usize, &str, bool)> = patterns()
            .binding
            .captures_iter(body)
            .map(|c| {
                let ty = c.get(2).unwrap().as_str().trim_start();
                (c.get(0).unwrap().start(), c.get(1).unwrap().as_str(), ty.starts_with('*'))
            })
            .collect();
        for c in patterns().size_of_val.captures_iter(body) {
            let name = c.get(1).unwrap().as_str();
            let pos = c.get(0).unwrap().start();
            let is_ptr = bindings
                .iter()
                .filter(|(at, n, _)| *at < pos && *n == name)
                .last()
                .is_some_and(|(_, _, p)| *p);
            if is_ptr {
                let at = start + pos;
                let line = code[..at].matches('\n').count() + 1;
                let raw_line = raw.lines().nth(line - 1).unwrap_or("");
                if !raw_line.contains("nolint: sizeof-pointer") {
                    hits.push((line, name.to_string()));
                }
            }
        }
    }
    hits.sort();
    hits.dedup();
    hits
}

#[cfg(test)]
mod tests {
    use super::*;
    const SRC: &str = "unsafe fn sel(n: usize, H: *mut H64) -> usize {\n    let mut P64: *mut Hdr = core::ptr::null_mut();\n    let a = size_of_val(&P64) + n * size_of_val(&H);   // two findings\n    let b = size_of::<Hdr>() + size_of_val(&*H);         // right: the pointee\n    let s = \"size_of_val(&P64)\";                           // right: a string\n    let v: Hdr = zeroed();\n    size_of_val(&v)                                       // right: not a pointer\n}\nfn other() -> usize {\n    let P64 = 3u64;\n    size_of_val(&P64)                                     // right: a different P64\n}\nfn cases(k: u32) -> usize {\n    if k == 0 { let A32: *mut S = p(); size_of_val(&A32) } // finding: the pointer\n    else { let mut A32: S = zeroed(); size_of_val(&A32) }   // right: rebound by value\n}\n";
    #[test]
    fn only_raw_pointer_bindings_of_the_same_function() {
        assert_eq!(
            scan_text(SRC),
            vec![(3, "H".into()), (3, "P64".into()), (14, "A32".into())]
        );
    }
    #[test]
    fn rule_identity() {
        assert_eq!(SizeofPointer.name(), "scan_sizeof_pointer");
    }
}
