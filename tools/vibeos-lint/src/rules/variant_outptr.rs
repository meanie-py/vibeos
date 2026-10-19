//! `V_DISPATCH!(v) as *mut *mut c_void` where the C wrote `(void**)&V_DISPATCH(v)`.
//!
//! The VARIANT accessor macros (`V_DISPATCH!`, `V_UNKNOWN!`, `V_BSTR!`, ...) expand to the
//! slot's VALUE. An out-parameter needs the slot's ADDRESS, and a translation that dropped the
//! C's `&` passes the (usually uninitialised) value instead: the callee then stores its result
//! through garbage. oleacc's Client_EnumVARIANT_Next handed AccessibleObjectFromWindow
//! `V_DISPATCH!(rgVar + n)` that way, and the heap corruption surfaced much later in
//! ntdll's RtlpRemoveFreeBlock. The address form is `addr_of_mut!(V_DISPATCH!(v))`.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::OnceLock;

fn line(text: &str, at: usize) -> usize {
    text[..at].bytes().filter(|b| *b == b'\n').count() + 1
}

fn skip(path: &str) -> bool {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/island_[a-z0-9_]+\.rs$)")
            .unwrap()
    })
    .is_match(path)
}

pub struct VariantOutptr;

impl Rule for VariantOutptr {
    fn name(&self) -> &'static str {
        "scan_variant_outptr"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        Ok(corpus
            .rust_files()
            .filter(|f| !skip(&f.path.to_string_lossy()))
            .flat_map(|f| {
                scan_text(&f.text)
                    .into_iter()
                    .map(|(n, m)| format!("{}:{n}: variant-outptr {m}", f.path.display()))
                    .collect::<Vec<_>>()
            })
            .collect())
    }
}

fn scan_text(text: &str) -> Vec<(usize, String)> {
    static COMMENTS: OnceLock<Regex> = OnceLock::new();
    static CAST: OnceLock<Regex> = OnceLock::new();
    let code = COMMENTS
        .get_or_init(|| Regex::new(r"(?s)//[^\n]*|/\*.*?\*/").unwrap())
        .replace_all(text, |caps: &regex::Captures<'_>| {
            caps.get(0)
                .unwrap()
                .as_str()
                .chars()
                .map(|c| if c == '\n' { '\n' } else { ' ' })
                .collect::<String>()
        });
    // An accessor call (one level of nested parentheses in its argument) cast straight to a
    // pointer-to-pointer.
    let re = CAST.get_or_init(|| {
        Regex::new(
            r"\b(V_[A-Z0-9_]+)!\((?:[^()]|\((?:[^()]|\([^()]*\))*\))*\)\s*as\s*\*mut\s*\*mut\b",
        )
        .unwrap()
    });
    re.captures_iter(&code)
        .filter(|c| {
            // Already the address form: `addr_of_mut!(V_X!(..)) as *mut *mut ..` is fine.
            let before = code[..c.get(0).unwrap().start()].trim_end();
            !(before.ends_with("addr_of_mut!(") || before.ends_with("&raw mut"))
        })
        .map(|c| {
            (
                line(&code, c.get(0).unwrap().start()),
                c.get(1).unwrap().as_str().to_string(),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn _variant_outptr_bad() {
        let text = "unsafe fn f(v: *mut VARIANT) {\n    AccessibleObjectFromWindow(h, 0, &IID_IDispatch,\n        V_DISPATCH!(rgVar.add(n as usize)) as *mut *mut core::ffi::c_void);\n}\n";
        let got = scan_text(text);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].1, "V_DISPATCH");
    }
    #[test]
    fn _variant_outptr_good() {
        let text = "unsafe fn f(v: *mut VARIANT) {\n    AccessibleObjectFromWindow(h, 0, &IID_IDispatch,\n        core::ptr::addr_of_mut!(V_DISPATCH!(v)) as *mut *mut core::ffi::c_void);\n    let p = V_DISPATCH!(v) as *mut IDispatch; // a value read: one level, fine\n    // V_DISPATCH!(v) as *mut *mut c_void in a comment is not code\n}\n";
        assert!(scan_text(text).is_empty());
    }
}
