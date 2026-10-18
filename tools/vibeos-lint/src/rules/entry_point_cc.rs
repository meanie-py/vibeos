//! A Windows-called entry point defined with a convention other than WINAPI.
//!
//! The loader, COM and the control panel call these by a convention Windows fixes, not the
//! module: DllMain is `BOOL WINAPI`, the COM server exports are `STDAPI`, CPlApplet is
//! `CALLBACK`, DriverEntry/DllInitialize/DllUnload are `NTAPI` -- all stdcall on i386. An
//! `extern "C" fn DllMain` compiles and links on every arch, but on i386 it defines
//! `_DllMain` where crt-startup calls `_DllMain@12`, so the link silently binds crt-startup's
//! `-alternatename` default and the module's own DllMain never runs. wshtcpip shipped that
//! way; sendmail and mydocs came back from translation with it (the model read
//! `STDAPI_(BOOL)` as no convention). scan_cc_mismatch cannot see it: nothing in the crate
//! declares DllMain. The authority is Windows, so the table is fixed. Per-arch
//! `*_amd64.rs`/`*_arm64.rs` files are skipped (only i386 has a second convention).
//! Suppress a verified false positive with `// nolint: entry-point-cc (reason)`.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::OnceLock;

pub struct EntryPointCc;
impl Rule for EntryPointCc {
    fn name(&self) -> &'static str {
        "scan_entry_point_cc"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut findings = Vec::new();
        for file in corpus.rust_files() {
            let stem = file
                .path
                .file_stem()
                .map(|s| s.to_string_lossy())
                .unwrap_or_default();
            if stem.ends_with("_amd64") || stem.ends_with("_arm64") {
                continue;
            }
            for (line, name, abi) in scan_text(&file.text) {
                findings.push(format!(
                    "{}:{line}: {name} defined extern \"{abi}\"; Windows calls it WINAPI \
                     (stdcall on i386): extern \"system\"",
                    file.path.display()
                ));
            }
        }
        findings.sort();
        Ok(findings)
    }
}

const ENTRY_POINTS: &[&str] = &[
    "DllMain",
    "DllEntryPoint",
    "DllGetClassObject",
    "DllCanUnloadNow",
    "DllRegisterServer",
    "DllUnregisterServer",
    "DllInstall",
    "DllGetVersion",
    "CPlApplet",
    "DriverEntry",
    "DllInitialize",
    "DllUnload",
];

fn definition() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // `#[no_mangle]` (or `#[unsafe(no_mangle)]`), further attributes, then the definition.
    // No `extern` is the Rust ABI; `extern` with no string is "C".
    RE.get_or_init(|| {
        Regex::new(
            r#"#\[(?:unsafe\()?no_mangle\)?\]\s*(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?(?P<ext>extern\s+(?:"(?P<abi>[^"]+)"\s+)?)?fn\s+(?P<name>\w+)\b"#,
        )
        .unwrap()
    })
}

fn scan_text(text: &str) -> Vec<(usize, String, String)> {
    let mut out = Vec::new();
    for caps in definition().captures_iter(text) {
        let name = &caps["name"];
        if !ENTRY_POINTS.contains(&name) {
            continue;
        }
        let abi = match (caps.name("ext"), caps.name("abi")) {
            (_, Some(abi)) => abi.as_str(),
            (Some(_), None) => "C",
            (None, None) => "Rust",
        };
        if abi == "system" || abi == "stdcall" {
            continue;
        }
        let whole = caps.get(0).unwrap();
        let eol = text[whole.end()..]
            .find('\n')
            .map_or(text.len(), |n| whole.end() + n);
        if text[whole.start()..eol].contains("nolint: entry-point-cc") {
            continue;
        }
        let line = text[..whole.start()].matches('\n').count() + 1;
        out.push((line, name.to_owned(), abi.to_owned()));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::scan_text;

    fn hits(text: &str) -> Vec<(usize, String, String)> {
        scan_text(text)
    }

    #[test]
    fn a_cdecl_dllmain_is_found() {
        let text = "#[no_mangle]\npub unsafe extern \"C\" fn DllMain(h: HINSTANCE) -> BOOL {";
        assert_eq!(hits(text), vec![(1, "DllMain".into(), "C".into())]);
    }

    #[test]
    fn system_and_stdcall_pass() {
        assert!(
            hits("#[no_mangle]\npub unsafe extern \"system\" fn DllGetClassObject() {}").is_empty()
        );
        assert!(hits("#[no_mangle]\nextern \"stdcall\" fn CPlApplet() {}").is_empty());
    }

    #[test]
    fn attributes_and_a_bare_extern_do_not_hide_it() {
        let text =
            "#[no_mangle]\n#[allow(non_snake_case)]\npub extern fn DllCanUnloadNow() -> i32 { 0 }";
        assert_eq!(hits(text), vec![(1, "DllCanUnloadNow".into(), "C".into())]);
    }

    #[test]
    fn a_rust_abi_definition_is_found() {
        let text = "#[no_mangle]\npub unsafe fn DllRegisterServer() -> i32 { 0 }";
        assert_eq!(
            hits(text),
            vec![(1, "DllRegisterServer".into(), "Rust".into())]
        );
    }

    #[test]
    fn other_names_are_ignored_and_nolint_suppresses() {
        assert!(hits("#[no_mangle]\npub extern \"C\" fn CreateShellLink() {}").is_empty());
        assert!(hits(
            "#[no_mangle]\npub extern \"C\" fn DllMain() {} // nolint: entry-point-cc (t)"
        )
        .is_empty());
    }
}
