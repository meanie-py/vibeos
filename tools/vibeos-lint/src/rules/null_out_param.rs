//! A pointer local initialised to null and never given a value, yet dereferenced: the C had a
//! STACK struct whose address a call fills (`OBJECT_HANDLE_INFORMATION HandleInfo;` passed as
//! `&HandleInfo`), and the translation made it a null `P...` pointer. The callee is told "no
//! out-param", fills nothing, and every later read sees the `is_null()` fallback. NtFilterToken
//! and NtDuplicateToken shipped this: every CreateRestrictedToken handle had access 0, which
//! is what stopped Chrome's sandbox (SBOX_ERROR_CANNOT_CREATE_RESTRICTED_TOKEN).
//!
//! The pointer is "given a value" by any `name = ...`, `&mut name`, `addr_of_mut!(name)`, or by
//! being an argument to a macro (`LIST_FOR_EACH_ENTRY!(name, ...)` assigns its cursor). Scope is
//! the rest of the enclosing function, approximated as text up to the next top-level `fn`.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::LazyLock;

pub struct NullOutParam;

#[cfg(test)]
use crate::lexical::strip_noise as blank_noise;
#[cfg(test)]
fn scan(text: &str) -> Vec<(usize, String)> {
    scan_code(&blank_noise(text))
}

fn scan_code(code: &str) -> Vec<(usize, String)> {
    static DECL: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"let\s+mut\s+(\w+)\s*:\s*(?:P[A-Z0-9_]+|\*mut\s+[\w:]+)\s*=\s*(?:core::ptr::)?null_mut\(\)\s*;")
            .unwrap()
    });
    static NEXT_FN: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\n(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?(?:extern\s+\S+\s+)?fn\s").unwrap());
    let mut out = vec![];
    for m in DECL.captures_iter(code) {
        let name = regex::escape(&m[1]);
        let end = m.get(0).unwrap().end();
        let rest = &code[end..];
        let body = &rest[..NEXT_FN.find(rest).map_or(rest.len(), |f| f.start())];
        let given = [
            format!(r"(?:^|[^\w.]){name}\s*=[^=]"),
            format!(r"&\s*(?:raw\s+)?mut\s+{name}\b"),
            format!(r"addr_of_mut!\(\s*{name}\s*\)"),
            format!(r"\w+!\s*[(\[{{][^;]*?\b{name}\b"),
        ]
        .iter()
        .any(|p| Regex::new(p).unwrap().is_match(body));
        let deref = Regex::new(&format!(r"\(\s*\*\s*{name}\s*\)")).unwrap().is_match(body);
        if !given && deref {
            out.push((code[..m.get(0).unwrap().start()].matches('\n').count() + 1, m[1].to_string()));
        }
    }
    out
}

impl Rule for NullOutParam {
    fn name(&self) -> &'static str {
        "scan_null_out_param"
    }
    fn needs_sdk_rust(&self) -> bool {
        true
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        Ok(c.sdk_rust_files()
            .flat_map(|f| {
                scan_code(&f.rust_noise()).into_iter().map(move |(n, name)| {
                    format!(
                        "{}:{}: NULL_OUT_PARAM `{name}` starts null, is never given a value, and is dereferenced: the C almost certainly had a stack struct whose ADDRESS a call fills. Declare the struct and pass `&mut` it.",
                        f.path.display(),
                        n
                    )
                })
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn the_shipped_shape_fires() {
        let src = "unsafe fn f() {\n    let mut handle_info: POBJECT_HANDLE_INFORMATION = core::ptr::null_mut();\n    ObReferenceObjectByHandle(h, 0, t, m, &mut o, handle_info);\n    let g = if !handle_info.is_null() { (*handle_info).GrantedAccess } else { 0 };\n}\n";
        assert_eq!(scan(src), vec![(2, "handle_info".to_string())]);
    }
    #[test]
    fn assigned_or_borrowed_or_macro_cursor_does_not_fire() {
        for src in [
            "fn f() {\n    let mut p: PFOO = core::ptr::null_mut();\n    p = alloc();\n    (*p).x = 1;\n}\n",
            "fn f() {\n    let mut p: PFOO = core::ptr::null_mut();\n    get(&mut p);\n    (*p).x = 1;\n}\n",
            "fn f() {\n    let mut p: *mut foo = core::ptr::null_mut();\n    LIST_FOR_EACH_ENTRY!(p, head, foo, entry, { (*p).x += 1; });\n}\n",
            "fn f() {\n    let mut p: PFOO = core::ptr::null_mut();\n    if !p.is_null() { g(p); }\n}\n",
        ] {
            assert!(scan(src).is_empty(), "{src}");
        }
    }
    #[test]
    fn scope_ends_at_the_next_fn() {
        let src = "fn f() {\n    let mut p: PFOO = core::ptr::null_mut();\n    use_it(p);\n}\nfn g(p: PFOO) {\n    (*p).x = 1;\n}\n";
        assert!(scan(src).is_empty());
    }
}
