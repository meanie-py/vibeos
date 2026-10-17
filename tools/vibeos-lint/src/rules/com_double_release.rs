//! Detect explicit COM Release calls that duplicate a CppComPtrGuard release.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::LazyLock;

pub struct ComDoubleRelease;
impl Rule for ComDoubleRelease {
    fn name(&self) -> &'static str {
        "scan_com_double_release"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut files: Vec<_> = corpus.rust_files().collect();
        files.sort_by(|a, b| a.path.cmp(&b.path));
        let mut out = Vec::new();
        for f in files {
            if f.path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("bindgen_") || n == "build.rs")
            {
                continue;
            }
            for (line, kind, msg) in scan_source(&f.text) {
                out.push(format!("{}:{line}: {kind}", f.path.display()));
                let _ = msg;
            }
        }
        Ok(out)
    }
}
static FN: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(
        r#"^\s*(?:pub\s+)?(?:unsafe\s+)?(?:extern\s+"[A-Za-z-]+"\s+)?fn\s+(\w+)"#,
    )
    .unwrap()
});
static GUARD: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"CppComPtrGuard::new\(\s*core::ptr::addr_of_mut!\(\s*(\w+)\s*\)\s*\)")
        .unwrap()
});
static RELEASE: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"\(\*\(\*(\w+)\)\.lpVtbl\)\.Release\)\(\s*(\w+)\s*[,)]").unwrap()
});
fn assigned_near(lines: &[&str], idx: usize, name: &str) -> bool {
    (idx + 1..(idx + 3).min(lines.len())).any(|j| {
        let line = lines[j];
        line.match_indices(name).any(|(start, _)| {
            let end = start + name.len();
            let before = line[..start].chars().next_back();
            let after_name = line[end..].chars().next();
            let boundary_before = before.map_or(true, |c| !c.is_alphanumeric() && c != '_');
            let boundary_after = after_name.map_or(true, |c| !c.is_alphanumeric() && c != '_');
            if !boundary_before || !boundary_after || before.is_some_and(|c| "=!<>".contains(c)) {
                return false;
            }
            let mut tail = line[end..].trim_start();
            if !tail.starts_with('=') {
                return false;
            }
            tail = &tail[1..];
            !tail.starts_with('=')
        })
    })
}
fn scan_source(text: &str) -> Vec<(usize, String, String)> {
    let lines: Vec<_> = text.split('\n').collect();
    let mut fn_name = "<file scope>".to_string();
    let mut guards = std::collections::HashSet::<(String, String)>::new();
    let mut releases = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if let Some(m) = FN.captures(line) {
            fn_name = m[1].to_string();
        }
        if let Some(m) = GUARD.captures(line) {
            guards.insert((m[1].to_string(), fn_name.clone()));
        }
        if let Some(m) = RELEASE.captures(line) {
            let name = m[1].to_string();
            if name == m[2] && !assigned_near(&lines, i, &name) {
                releases.push((i + 1, name, fn_name.clone()));
            }
        }
    }
    releases.into_iter().filter(|(_,n,f)|guards.contains(&(n.clone(),f.clone()))).map(|(ln,name,f)| (ln,"com-double-release".into(),format!("`{name}` in `{f}` is CppComPtrGuard-owned; this explicit Release is a second one (null the slot, or delete it)"))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Corpus, SourceFile};
    use std::path::PathBuf;
    fn kinds(s: &str) -> Vec<String> {
        scan_source(s).into_iter().map(|x| x.1).collect()
    }
    #[test]
    fn guard_plus_unnulled_release_is_a_finding() {
        let src = "\npub unsafe fn CDefView_FillList_body(this: *mut CDefView) -> HRESULT {\n    let mut pEnumIDList: *mut IEnumIDList = core::ptr::null_mut();\n    let _pEnumIDList_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pEnumIDList));\n    if !pEnumIDList.is_null() {\n        ((*(*pEnumIDList).lpVtbl).Release)(pEnumIDList);\n    }\n    return S_OK;\n}\n";
        assert_eq!(kinds(src), vec!["com-double-release"]);
    }
    #[test]
    fn release_then_null_is_correct() {
        let src = "\npub unsafe fn CDefView_Exec_body(this: *mut CDefView) -> HRESULT {\n    let mut pStream: *mut IStream = core::ptr::null_mut();\n    let _pStream_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pStream));\n    if !pStream.is_null() {\n        ((*(*pStream).lpVtbl).Release)(pStream);\n        pStream = core::ptr::null_mut();\n    }\n    return S_OK;\n}\n";
        assert!(kinds(src).is_empty());
    }
    #[test]
    fn hand_over_hand_walk_is_correct() {
        let src = "\npub unsafe fn CMenuDeskBar__IsSubMenuParent_body(hwnd: HWND) -> BOOL {\n    let _popup_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(popup));\n    while !popup.is_null() {\n        ((*(*popup).lpVtbl).Release)(popup);\n        popup = next;\n    }\n    FALSE\n}\n";
        assert!(kinds(src).is_empty());
    }
    #[test]
    fn release_without_guard_is_correct() {
        let src = "\npub unsafe fn CDefView_SaveViewState_helper_body(this: *mut CDefView) -> HRESULT {\n    let mut pPB: *mut IPropertyBag = core::ptr::null_mut();\n    ((*(*pPB).lpVtbl).Release)(pPB);\n    return S_OK;\n}\n";
        assert!(kinds(src).is_empty());
    }
    #[test]
    fn guard_and_release_in_different_functions_is_correct() {
        let src = "\npub unsafe fn CDefView_LoadViewState_body(this: *mut CDefView) -> HRESULT {\n    let _pPB_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pPB));\n    return S_OK;\n}\npub unsafe fn CDefView_SaveViewState_helper_body(this: *mut CDefView) -> HRESULT {\n    ((*(*pPB).lpVtbl).Release)(pPB);\n    return S_OK;\n}\n";
        assert!(kinds(src).is_empty());
    }
    #[test]
    fn release_of_different_pointer_is_not_a_finding() {
        let src = "\npub unsafe fn f() -> HRESULT {\n    let _pa_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pa));\n    ((*(*pa).lpVtbl).Release)(pb);\n    return S_OK;\n}\n";
        assert!(kinds(src).is_empty());
    }
    #[test]
    fn exact_control_corpus() {
        let bad="pub unsafe fn FillList_body(this: *mut T) -> HRESULT {\nlet mut pEnum: *mut IEnumIDList = core::ptr::null_mut();\nlet _pEnum_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pEnum));\n((*(*pEnum).lpVtbl).Release)(pEnum);\nreturn S_OK;\n}";
        let c = Corpus::new(
            PathBuf::from("."),
            vec![SourceFile::new("src/bad.rs".into(), bad.into())],
        );
        assert_eq!(
            ComDoubleRelease.run(&c).unwrap(),
            vec!["src/bad.rs:4: com-double-release"]
        );
    }
}
