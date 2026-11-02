//! A `DEVICE_OBJECT` pointer cast to a device-EXTENSION pointer type.
//!
//! The C reads `DeviceObject->DeviceExtension`; six classpnp sites translated it as
//! `DeviceObject as *mut FUNCTIONAL_DEVICE_EXTENSION`, so `ClasspAllocateReleaseRequest`
//! initialized its release-queue spinlock and flags inside the DEVICE_OBJECT itself. It surfaced
//! only when the crate was armed and the extension type's alignment was checked (2026-11-01).
//! A binding counts as a device object when the function declares it (parameter or `let`) with a
//! `PDEVICE_OBJECT`, `*mut DEVICE_OBJECT` or `*mut _DEVICE_OBJECT` type.
//! `// nolint: devobj-as-extension (why)` on the line suppresses.
use crate::{twins, Corpus, Rule};
use rayon::prelude::*;
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

pub struct DevobjAsExtension;
impl Rule for DevobjAsExtension {
    fn name(&self) -> &'static str {
        "scan_devobj_as_extension"
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let files: Vec<_> = c
            .rust_files()
            .filter(|f| CAST.is_match(&f.text))
            .filter(|f| !SKIP.is_match(&f.path.to_string_lossy()))
            .collect();
        let mut out: Vec<String> = files
            .par_iter()
            .flat_map_iter(|f| {
                scan(&f.text, &f.path)
                    .into_iter()
                    .map(move |x| format!("{}:{x}", f.path.display()))
            })
            .collect();
        out.sort();
        Ok(out)
    }
}

static SKIP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/types_[a-z0-9]+\.rs$)").unwrap()
});
const DEVOBJ_TYPE: &str = r"(?:P_?DEVICE_OBJECT|\*\s*mut\s+(?:[\w:]+::)?_?DEVICE_OBJECT)\b";
static CAST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b([A-Za-z_]\w*)\s+as\s+(?:\*\s*(?:mut|const)\s+|P)_?(\w*DEVICE_EXTENSION)\b").unwrap()
});
static PARAM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(r"(?:^|[,(])\s*(?:mut\s+)?([A-Za-z_]\w*)\s*:\s*{DEVOBJ_TYPE}")).unwrap()
});
static LET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(r"\blet\s+(?:mut\s+)?([A-Za-z_]\w*)\s*:\s*{DEVOBJ_TYPE}")).unwrap()
});
static NOLINT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bnolint:[\w\s,-]*\bdevobj-as-extension\b").unwrap());

fn scan(text: &str, path: &std::path::Path) -> Vec<String> {
    let blank = twins::blank_rust(text);
    let newlines: Vec<_> = blank.match_indices('\n').map(|(at, _)| at).collect();
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    for fun in twins::rust_functions(&blank, path) {
        let mut devobjs: HashSet<&str> = PARAM.captures_iter(&fun.params).map(|c| c.get(1).unwrap().as_str()).collect();
        devobjs.extend(LET.captures_iter(&fun.body).map(|c| c.get(1).unwrap().as_str()));
        if devobjs.is_empty() {
            continue;
        }
        for c in CAST.captures_iter(&fun.body) {
            let name = c.get(1).unwrap().as_str();
            if !devobjs.contains(name) {
                continue;
            }
            // `(*DeviceObject).DeviceExtension as ...` is the correct form; the capture is then
            // `DeviceExtension`, never a device-object binding, so it does not reach here.
            let at = fun.body_start + c.get(0).unwrap().start();
            let line = newlines.partition_point(|&nl| nl < at) + 1;
            if lines.get(line - 1).is_some_and(|l| NOLINT.is_match(l)) {
                continue;
            }
            out.push(format!(
                "{line}: devobj-as-extension {}: device object `{name}` cast to {} -- the C reads `{name}->DeviceExtension`",
                fun.name,
                c.get(2).unwrap().as_str()
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_object_cast_to_extension_is_flagged() {
        let s = "pub unsafe extern \"C\" fn ClasspAllocateReleaseRequest(Fdo: PDEVICE_OBJECT) -> NTSTATUS {\n    let fdoExtension = Fdo as PFUNCTIONAL_DEVICE_EXTENSION;\n    0\n}\n";
        let got = scan(s, std::path::Path::new("x.rs"));
        assert_eq!(got.len(), 1, "{got:?}");
        assert!(got[0].starts_with("2: devobj-as-extension ClasspAllocateReleaseRequest"));
        let let_bound = "fn f() {\n    let d: *mut DEVICE_OBJECT = g();\n    let e = d as *mut COMMON_DEVICE_EXTENSION;\n}\n";
        assert_eq!(scan(let_bound, std::path::Path::new("x.rs")).len(), 1);
    }

    #[test]
    fn reading_device_extension_passes() {
        let ok = "fn f(Fdo: PDEVICE_OBJECT) {\n    let e = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;\n    let other = Context as PFUNCTIONAL_DEVICE_EXTENSION;\n}\n";
        assert!(scan(ok, std::path::Path::new("x.rs")).is_empty());
        let nolint = "fn f(Fdo: PDEVICE_OBJECT) {\n    let e = Fdo as PFUNCTIONAL_DEVICE_EXTENSION; // nolint: devobj-as-extension (verified)\n}\n";
        assert!(scan(nolint, std::path::Path::new("x.rs")).is_empty());
    }
}
