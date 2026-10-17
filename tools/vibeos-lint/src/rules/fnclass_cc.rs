//! Detect cdecl Rust definitions of callbacks declared through stdcall function classes.
use crate::{twins, Corpus, Rule, SourceFile, SourceSet};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub struct FnclassCc;
const INPUTS: &[SourceSet] = &[
    SourceSet::tree("fnclass-rust", &["sdk/lib/rust"], &["rs"], &[]).with_inputs(),
    SourceSet::tracked("fnclass-catalog", &["c", "cpp", "h"], &[]),
    SourceSet::tree(
        "fnclass-island",
        &["sdk/lib/rust/kmdf-sys/src"],
        &[],
        &["types_i386.rs"],
    ),
];
const NOLINT: &str = "nolint: fnclass-cc";

fn typedef_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        crate::regexes::cached(
            r"typedef\s+[\w\s\*]+?\(\s*(?:NTAPI|STDCALL|WINAPI|CALLBACK|APIENTRY|__stdcall|_stdcall)\s+(\w+)\s*\)\s*\(|typedef\s+[\w\s\*]+?\s+(?:NTAPI|STDCALL|WINAPI|CALLBACK|APIENTRY|__stdcall|_stdcall)\s+(\w+)\s*\(",
        )
        .unwrap()
    })
}
fn kmdf_pfn_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        crate::regexes::cached(
            r#"pub type (?:PFN_|EVT_)(\w+) = ::core::option::Option<unsafe extern "system""#,
        )
        .unwrap()
    })
}
fn declaration_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| crate::regexes::cached(r"(?m)^[ \t]*(\w+)[ \t]+(\w+)[ \t]*;").unwrap())
}
fn rust_def_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        crate::regexes::cached(r#"(?m)^\s*pub\s+(?:unsafe\s+)?extern\s+"C"\s+fn\s+(\w+)\s*\("#)
            .unwrap()
    })
}
fn generated_rust_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| crate::regexes::cached(r"(?:bindgen|types)_\w+\.rs$").unwrap())
}

#[derive(Clone)]
struct Declaration {
    class: String,
    path: PathBuf,
}

fn repo_path(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
fn class_name(capture: &regex::Captures<'_>) -> Option<String> {
    capture
        .get(1)
        .or_else(|| capture.get(2))
        .map(|m| m.as_str().to_owned())
}
fn path_is_excluded(path: &Path) -> bool {
    path.starts_with("sdk/lib/rust") || path.starts_with("modules/rostests")
}
fn same_module(rust_path: &Path, c_path: &Path) -> bool {
    let fake = twins::Function {
        name: String::new(),
        path: c_path.to_owned(),
        line: 0,
        body: String::new(),
        body_start: 0,
        params: String::new(),
        ret: String::new(),
    };
    twins::same_module(rust_path, &fake)
}

fn classed_declarations(
    catalog: &[&SourceFile],
    rust: &[&SourceFile],
) -> HashMap<String, Vec<Declaration>> {
    let mut headers: Vec<_> = catalog
        .iter()
        .copied()
        .filter(|f| f.path.extension().is_some_and(|e| e == "h") && !path_is_excluded(&f.path))
        .collect();
    headers.sort_by(|a, b| a.path.cmp(&b.path));
    let mut classes = HashSet::new();
    for h in &headers {
        if !h.text.contains("typedef") {
            continue;
        }
        for capture in typedef_re().captures_iter(&h.text) {
            if let Some(name) = class_name(&capture) {
                classes.insert(name);
            }
        }
    }
    for &file in rust {
        if file.path == Path::new("sdk/lib/rust/kmdf-sys/src/types_i386.rs") {
            for capture in kmdf_pfn_re().captures_iter(&file.text) {
                classes.insert(format!("EVT_{}", &capture[1]));
            }
        }
    }

    let mut files: Vec<_> = catalog
        .iter()
        .copied()
        .filter(|f| {
            matches!(
                f.path.extension().and_then(|e| e.to_str()),
                Some("c" | "cpp" | "h")
            ) && !path_is_excluded(&f.path)
        })
        .collect();
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let mut out = HashMap::<String, Vec<Declaration>>::new();
    for file in files {
        for capture in declaration_re().captures_iter(&file.text) {
            let class = capture.get(1).unwrap().as_str();
            if !classes.contains(class) {
                continue;
            }
            out.entry(capture[2].to_owned())
                .or_default()
                .push(Declaration {
                    class: class.to_owned(),
                    path: file.path.clone(),
                });
        }
    }
    out
}

fn run_scan(c: &Corpus) -> Vec<(String, usize, String, String, String)> {
    let mut rust_files: Vec<_> = c
        .sources("fnclass-rust")
        .filter(|f| f.path.extension().is_some_and(|ext| ext == "rs"))
        .collect();
    let island = Path::new("sdk/lib/rust/kmdf-sys/src/types_i386.rs");
    if !rust_files.iter().any(|f| f.path == island) {
        if let Some(file) = c.file(island) {
            rust_files.push(file);
        }
    }
    let catalog: Vec<_> = c.sources("fnclass-catalog").collect();
    let decls = classed_declarations(&catalog, &rust_files);
    let mut findings = Vec::new();
    for file in rust_files {
        if generated_rust_re()
            .is_match(&file.path.file_name().unwrap_or_default().to_string_lossy())
            || !file.text.contains("extern \"C\"")
        {
            continue;
        }
        let lines: Vec<_> = file.text.split('\n').collect();
        for m in rust_def_re().captures_iter(&file.text) {
            let name = &m[1];
            let line = file.text[..m.get(1).unwrap().start()]
                .bytes()
                .filter(|byte| *byte == b'\n')
                .count()
                + 1;
            if decls.get(name).map_or(true, Vec::is_empty)
                || lines
                    .get(line - 1)
                    .is_some_and(|text| text.contains(NOLINT))
            {
                continue;
            }
            let declaration = decls[name]
                .iter()
                .find(|d| same_module(&file.path, &d.path));
            let Some(declaration) = declaration else {
                continue;
            };
            findings.push((
                repo_path(&file.path, &c.root),
                line,
                name.to_owned(),
                declaration.class.clone(),
                repo_path(&declaration.path, &c.root),
            ));
        }
    }
    findings.sort();
    findings
}

impl Rule for FnclassCc {
    fn name(&self) -> &'static str {
        "scan_fnclass_cc"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        Ok(run_scan(c)
            .into_iter()
            .map(|(path, _line, name, _class, _cpath)| format!("{path}: {name} [FNCLASS_CC]"))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn control_corpus(rust: &str) -> Corpus {
        Corpus::new(
            PathBuf::from("/repo"),
            vec![
                SourceFile::new(
                    "drivers/cdrom/cdrom.h".into(),
                    "typedef void (NTAPI EVT_WDF_WORKITEM)(void *);\nEVT_WDF_WORKITEM DeviceRestoreDefaultSpeed;\n"
                        .into(),
                ),
                SourceFile::new(
                    "sdk/lib/rust/cdrom-rust/src/common.rs".into(),
                    rust.into(),
                ),
            ],
        )
    }

    #[test]
    fn scanner_control_fnclass_cc_bad_and_good() {
        let bad = "#[no_mangle]\npub unsafe extern \"C\" fn DeviceRestoreDefaultSpeed(WorkItem: WDFWORKITEM) {}\n";
        let good = "#[no_mangle]\npub unsafe extern \"system\" fn DeviceRestoreDefaultSpeed(WorkItem: WDFWORKITEM) {}\n#[no_mangle]\npub unsafe extern \"C\" fn StringsAreMatched(a: *const i8, b: *const i8) -> u8 { 0 }\n";
        let bad_result = FnclassCc.run(&control_corpus(bad)).unwrap();
        assert_eq!(
            bad_result,
            vec!["sdk/lib/rust/cdrom-rust/src/common.rs: DeviceRestoreDefaultSpeed [FNCLASS_CC]"]
        );
        assert!(FnclassCc.run(&control_corpus(good)).unwrap().is_empty());
    }

    #[test]
    fn fnclass_requires_a_same_module_classed_declaration() {
        let rust = "pub unsafe extern \"C\" fn DeviceRestoreDefaultSpeed() {}\n";
        // A different module must not pair merely because the function name matches.
        let other = SourceFile::new(
            "drivers/usb/other.h".into(),
            "EVT_WDF_WORKITEM DeviceRestoreDefaultSpeed;\n".into(),
        );
        let corpus = Corpus::new(
            PathBuf::from("/repo"),
            vec![
                other,
                SourceFile::new(
                    "drivers/cdrom/cdrom.h".into(),
                    "typedef void (NTAPI EVT_WDF_WORKITEM)(void *);\n".into(),
                ),
                SourceFile::new("sdk/lib/rust/cdrom-rust/src/common.rs".into(), rust.into()),
            ],
        );
        assert!(FnclassCc.run(&corpus).unwrap().is_empty());
    }

    #[test]
    fn fnclass_honors_same_line_nolint_and_kmdf_system_callback_classes() {
        let corpus = Corpus::new(
            PathBuf::from("/repo"),
            vec![
                SourceFile::new(
                    "drivers/cdrom/cdrom.h".into(),
                    "typedef void (NTAPI EVT_WDF_WORKITEM)(void *);\nEVT_WDF_WORKITEM DeviceRestoreDefaultSpeed;\nEVT_WDF_IO_QUEUE_IO_READ WdfIoQueueIoRead;\n"
                        .into(),
                ),
                SourceFile::new(
                    "sdk/lib/rust/kmdf-sys/src/types_i386.rs".into(),
                    "pub type PFN_WDF_IO_QUEUE_IO_READ = ::core::option::Option<unsafe extern \"system\" fn()>;\n"
                        .into(),
                ),
                SourceFile::new(
                    "sdk/lib/rust/cdrom-rust/src/common.rs".into(),
                    "pub unsafe extern \"C\" fn DeviceRestoreDefaultSpeed() {} // nolint: fnclass-cc\npub unsafe extern \"C\" fn WdfIoQueueIoRead() {}\n"
                        .into(),
                ),
            ],
        );
        assert_eq!(
            FnclassCc.run(&corpus).unwrap(),
            vec!["sdk/lib/rust/cdrom-rust/src/common.rs: WdfIoQueueIoRead [FNCLASS_CC]"]
        );
    }

    #[test]
    fn fnclass_skips_generated_bindgen_and_types_files() {
        let mut files =
            control_corpus("pub unsafe extern \"C\" fn DeviceRestoreDefaultSpeed() {}\n").files;
        files.push(SourceFile::new(
            "sdk/lib/rust/cdrom-rust/src/bindgen_i386.rs".into(),
            "pub unsafe extern \"C\" fn DeviceRestoreDefaultSpeed() {}\n".into(),
        ));
        let corpus = Corpus::new(PathBuf::from("/repo"), files);
        assert_eq!(
            FnclassCc.run(&corpus).unwrap(),
            vec!["sdk/lib/rust/cdrom-rust/src/common.rs: DeviceRestoreDefaultSpeed [FNCLASS_CC]"]
        );
    }
}
