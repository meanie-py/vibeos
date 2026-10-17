use crate::{Corpus, Rule};
use regex::Regex;
use std::{path::Path, sync::LazyLock};

pub struct RawdylibLinkname;

const NON_DLL_MODULES: &[(&str, &str)] = &[
    ("ntoskrnl", "ntoskrnl.exe"),
    ("win32k", "win32k.sys"),
    ("classpnp", "classpnp.sys"),
    ("scsiport", "scsiport.sys"),
    ("storport", "storport.sys"),
    ("videoprt", "videoprt.sys"),
    ("ndis", "ndis.sys"),
    ("usbport", "usbport.sys"),
    ("usbd", "usbd.sys"),
    ("ks", "ks.sys"),
    ("portcls", "portcls.sys"),
];
const KNOWN_EXTS: &[&str] = &[".dll", ".exe", ".sys", ".drv", ".ocx", ".cpl"];

static LINK: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"(?s)#\s*\[\s*link\s*\((?P<body>[^]]*?)\)\s*\]").unwrap()
});
static NAME: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r#"name\s*=\s*"(?P<name>[^"]*)""#).unwrap());
static KIND: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r#"kind\s*=\s*"raw-dylib""#).unwrap());
static MODS: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r#"modifiers\s*=\s*"(?P<mods>[^"]*)""#).unwrap());

fn has_ext(name: &str) -> bool {
    let lower = name.to_lowercase();
    KNOWN_EXTS.iter().any(|ext| lower.ends_with(ext))
}

fn scan_text(path: &Path, text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for attr in LINK.captures_iter(text) {
        let body = &attr["body"];
        if !KIND.is_match(body) {
            continue;
        }
        let Some(nm) = NAME.captures(body) else {
            continue;
        };
        let name = &nm["name"];
        let mods = MODS.captures(body);
        let verbatim = mods.is_some_and(|m| m["mods"].contains("+verbatim"));
        let resolved = if verbatim {
            name.to_owned()
        } else {
            format!("{name}.dll")
        };
        let lower = name.to_lowercase();
        let lineno = text[..attr.get(0).unwrap().start()]
            .bytes()
            .filter(|&b| b == b'\n')
            .count()
            + 1;
        let (kind, hint) = if has_ext(name) && !verbatim {
            (
                "DOUBLE_EXT",
                "add modifiers = \"+verbatim\" — rustc appends .dll unconditionally".to_string(),
            )
        } else if !verbatim {
            if let Some((_, real)) = NON_DLL_MODULES.iter().find(|(stem, _)| *stem == lower) {
                (
                    "WRONG_EXT",
                    format!(
                        "the module is {real} — write name = \"{real}\", modifiers = \"+verbatim\""
                    ),
                )
            } else {
                continue;
            }
        } else if !has_ext(name) {
            (
                "NO_EXT",
                "+verbatim suppresses the .dll rustc would append, so the name must already carry the real extension".to_string(),
            )
        } else {
            continue;
        };
        out.push(format!(
            "{}:{lineno}: {kind} raw-dylib name=\"{name}\" resolves to '{resolved}' — {hint}",
            path.display()
        ));
    }
    out
}

impl Rule for RawdylibLinkname {
    fn name(&self) -> &'static str {
        "scan_rawdylib_linkname"
    }

    fn needs_sdk_rust(&self) -> bool {
        true
    }

    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut findings = corpus
            .sdk_rust_files()
            .filter(|f| f.path.extension().is_some_and(|e| e == "rs"))
            .filter(|f| !f.path.to_string_lossy().contains("/target/"))
            .flat_map(|f| scan_text(&f.path, &f.text))
            .collect::<Vec<_>>();
        findings.sort();
        Ok(findings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceFile;
    use std::path::PathBuf;

    fn scan(src: &str) -> Vec<String> {
        let file = SourceFile::new("sdk/lib/rust/x-rust/src/lib.rs".into(), src.into());
        let corpus = Corpus::new(PathBuf::from("/repo"), vec![file]);
        RawdylibLinkname.run(&corpus).unwrap()
    }

    #[test]
    fn original_self_test_cases() {
        let cases = [
            (
                r##"#[link(name = "ntoskrnl", kind = "raw-dylib") ]"##,
                "WRONG_EXT",
            ),
            (
                r##"#[link(name = "ntoskrnl.exe", kind = "raw-dylib") ]"##,
                "DOUBLE_EXT",
            ),
            (
                r##"#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim") ]"##,
                "",
            ),
            (r##"#[link(name = "ntdll", kind = "raw-dylib") ]"##, ""),
            (r##"#[link(name = "hal", kind = "raw-dylib") ]"##, ""),
            (
                r##"#[link(name = "win32k", kind = "raw-dylib") ]"##,
                "WRONG_EXT",
            ),
            (
                r##"#[link(name = "foo", kind = "raw-dylib", modifiers = "+verbatim") ]"##,
                "NO_EXT",
            ),
            (r##"#[link(name = "ntoskrnl", kind = "static") ]"##, ""),
            (
                r##"#[link(
    name = "ntoskrnl.exe",
    kind = "raw-dylib"
)]"##,
                "DOUBLE_EXT",
            ),
        ];
        for (src, want) in cases {
            let got = scan(src);
            if want.is_empty() {
                assert!(got.is_empty(), "{src}: {got:?}");
            } else {
                assert!(got.iter().any(|s| s.contains(want)), "{src}: {got:?}");
            }
        }
    }

    #[test]
    fn path_relative_line_and_rules() {
        assert_eq!(
            scan("// top\n#[link(name = \"ntoskrnl.exe\", kind = \"raw-dylib\")]"),
            vec!["sdk/lib/rust/x-rust/src/lib.rs:2: DOUBLE_EXT raw-dylib name=\"ntoskrnl.exe\" resolves to 'ntoskrnl.exe.dll' — add modifiers = \"+verbatim\" — rustc appends .dll unconditionally"]
        );
    }
}
