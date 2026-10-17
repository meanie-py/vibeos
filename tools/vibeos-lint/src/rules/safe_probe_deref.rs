use crate::{rules::raw_user_access, Corpus, Rule};
use regex::Regex;
use std::sync::LazyLock;

pub struct SafeProbeDeref;
static PROBE_CALL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b\w*SafeProbeFor(?:Read|Write)\s*\(").unwrap());
static IDENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\w+$").unwrap());
static NOLINT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"nolint:\s*(safe-probe-deref|safe_probe_deref|scan_safe_probe_deref|all)\b")
        .unwrap()
});
fn norm(expr: &str) -> String {
    let mut s = expr.trim().to_string();
    loop {
        let old = s.clone();
        let mut depth = 0i32;
        let mut cast = None;
        for (i, c) in s.char_indices() {
            if "([{ ".contains(c) && c != ' ' {
                depth += 1
            } else if ")] }".contains(c) && c != ' ' {
                depth -= 1
            } else if depth == 0
                && s[i..].starts_with(" as ")
                && (i == 0 || s.as_bytes()[i - 1] != b'_')
            {
                cast = Some(i)
            }
        }
        if let Some(i) = cast {
            s = s[..i].trim().into()
        }
        let cast_call = crate::regexes::cached(r"\.\s*cast(?:_mut|_const)?\s*\(\s*\)$").unwrap();
        s = cast_call.replace(&s, "").trim().into();
        if s.starts_with('(') && s.ends_with(')') {
            let mut d = 0i32;
            let mut wraps = true;
            for (i, c) in s.char_indices() {
                d += (c == '(') as i32 - (c == ')') as i32;
                if d == 0 && i + 1 < s.len() {
                    wraps = false;
                    break;
                }
            }
            if wraps {
                s = s[1..s.len() - 1].trim().into()
            }
        }
        if s == old {
            break;
        }
    }
    s.chars().filter(|c| !c.is_whitespace()).collect()
}
fn first_arg(text: &str, open: usize) -> Option<&str> {
    let (mut d, mut start) = (0i32, open + 1);
    for (i, c) in text.char_indices().filter(|(i, _)| *i > open) {
        if c == '(' || c == '[' || c == '{' {
            d += 1
        } else if c == ')' || c == ']' || c == '}' {
            if d == 0 {
                return Some(&text[start..i]);
            }
            d -= 1
        } else if c == ',' && d == 0 {
            return Some(&text[start..i]);
        }
    }
    None
}
fn probe_targets(body: &str) -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    for m in PROBE_CALL.find_iter(body) {
        let Some(arg) = first_arg(body, m.end() - 1) else {
            continue;
        };
        let n = norm(arg);
        if IDENT.is_match(&n) {
            out.insert(n);
        }
    }
    out
}
impl Rule for SafeProbeDeref {
    fn name(&self) -> &'static str {
        "scan_safe_probe_deref"
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in c.rust_files().filter(|f| !raw_user_access::is_skipped(f)) {
            for f in raw_user_access::parse_file(file, &c.root) {
                let body = raw_user_access::body_code(&f);
                let names = probe_targets(&body);
                if names.is_empty() {
                    continue;
                }
                for hit in raw_user_access::walk_hits(&f, &names) {
                    if hit.kind == "hit"
                        && !NOLINT.is_match(raw_user_access::raw_line(&f, hit.line))
                    {
                        out.push(format!("{}: {}", f.path.display(), f.function.name));
                        break;
                    }
                }
            }
        }
        out.sort();
        out.dedup();
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn rows(src: &str) -> Vec<String> {
        let f = crate::SourceFile::new("sdk/lib/rust/x-rust/src/a.rs".into(), src.into());
        let c = Corpus::new("/repo".into(), vec![f]);
        SafeProbeDeref.run(&c).unwrap()
    }
    #[test]
    fn raw_access_is_found_and_safe_copy_is_not() {
        let src="pub unsafe fn f(P: *const RECTL) -> i32 {\n if SafeProbeForRead(P as *const c_void,16,4)<0{return -1;}\n let rc=*(P as *const RECTL);\n rc.left\n}";
        assert_eq!(rows(src), vec!["sdk/lib/rust/x-rust/src/a.rs: f"]);
        let safe="pub unsafe fn f(P: *const c_void) -> i32 { if SafeProbeForRead(P,16,4)<0{return -1;} safe_copy_raw(dst,P,16); 0 }";
        assert!(rows(safe).is_empty())
    }
    #[test]
    fn copy_mirror_kernel_arm_nolint_and_capture_field() {
        let base="pub unsafe fn f(P: *const u8, dst:*mut u8, Mode:i8)->i32 {\n if Mode==0 {\n core::ptr::copy_nonoverlapping(P,dst,16);\n } else {\n if KsSafeProbeForWrite(P,16,1)<0{return -1;}\n core::ptr::copy_nonoverlapping(P,dst,16);\n }\n 0\n}";
        assert_eq!(rows(base), vec!["sdk/lib/rust/x-rust/src/a.rs: f"]);
        let good = base.replace(
            "core::ptr::copy_nonoverlapping(P,dst,16);\n }\n 0",
            "safe_copy_raw(dst,P,16);\n }\n 0",
        );
        assert!(rows(&good).is_empty());
        let waived = base.replace(
            "core::ptr::copy_nonoverlapping(P,dst,16);",
            "core::ptr::copy_nonoverlapping(P,dst,16); // nolint: safe_probe_deref reviewed",
        );
        assert!(rows(&waived).is_empty());
        let field="pub unsafe fn f(cap:&UNICODE_STRING)->i32 { if SafeProbeForRead(cap.Buffer,8,1)<0{return -1;} let x=*cap.Buffer; x as i32 }";
        assert!(rows(field).is_empty())
    }
    #[test]
    fn all_original_safe_probe_shapes() {
        let copy = "pub unsafe fn f(P:*const u8,dst:*mut u8)->i32 { if SafeProbeForRead(P,16,4)<0{return -1;} core::ptr::copy_nonoverlapping(P,dst,16); 0 }";
        assert_eq!(rows(copy), vec!["sdk/lib/rust/x-rust/src/a.rs: f"]);
        let capture = "pub unsafe fn f(pbmi:*const BMI)->i32 {\n if SafeProbeForRead(pbmi as *const c_void,40,1)<0{return -1;}\n let pbmiSafe=alloc(40) as *const BMI;\n safe_copy_raw(pbmiSafe as *mut c_void,pbmi as *const c_void,40);\n let pbmi=pbmiSafe;\n let h=(*pbmi).biHeight;\n h\n}";
        assert!(rows(capture).is_empty());
        let mirror = "pub unsafe fn f(P:*const RECTL)->i32 { if KsSafeProbeForRead(P as *const c_void,16,4)<0{return -1;} let rc=*(P as *const RECTL); rc.left }";
        assert_eq!(rows(mirror), vec!["sdk/lib/rust/x-rust/src/a.rs: f"]);
        let field = "pub unsafe fn f(cap:&UNICODE_STRING)->i32 { if SafeProbeForRead(cap.Buffer as *const c_void,16,4)<0{return -1;} let w=*cap.Buffer; w as i32 }";
        assert!(rows(field).is_empty());
    }
    #[test]
    fn original_scanner_control_bad_and_good() {
        let bad = r#"
pub unsafe fn KsFastPropertyHandler(Property: *const KSPROPERTY, Mode: i8) -> BOOLEAN {
    let mut PropRequest: KSPROPERTY = core::mem::zeroed();
    let dst = &mut PropRequest as *mut KSPROPERTY as *mut u8;
    if Mode == 0i8 {
        core::ptr::copy_nonoverlapping(Property as *const u8, dst, 24);
    } else {
        if !NT_SUCCESS(KsSafeProbeForRead(Property as *const c_void, 24, 1)) { return FALSE; }
        core::ptr::copy_nonoverlapping(Property as *const u8, dst, 24);
    }
    TRUE
}
"#;
        let good = r#"
pub unsafe fn KsFastPropertyHandler(Property: *const KSPROPERTY, Mode: i8) -> BOOLEAN {
    let mut PropRequest: KSPROPERTY = core::mem::zeroed();
    let dst = &mut PropRequest as *mut KSPROPERTY;
    if Mode == 0i8 {
        core::ptr::copy_nonoverlapping(Property as *const u8, dst as *mut u8, 24);
    } else {
        if !NT_SUCCESS(KsSafeProbeForRead(Property as *const c_void, 24, 1)) { return FALSE; }
        if ntkrnl_sys::probe::safe_copy_raw(
            dst as *mut c_void, Property as *const c_void, 24,
        ).is_err() { return FALSE; }
    }
    TRUE
}
"#;
        assert_eq!(
            rows(bad),
            vec!["sdk/lib/rust/x-rust/src/a.rs: KsFastPropertyHandler"]
        );
        assert!(rows(good).is_empty());
    }
}
