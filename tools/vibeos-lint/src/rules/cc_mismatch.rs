use crate::{Corpus, Rule, SourceFile};
use regex::{Captures, Regex};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::LazyLock,
};

pub struct CcMismatch;

pub(crate) const NORM_I386: &[(&str, &str)] = &[
    ("system", "stdcall"),
    ("C", "C"),
    ("cdecl", "C"),
    ("fastcall", "fastcall"),
    ("stdcall", "stdcall"),
    ("thiscall", "thiscall"),
];
fn norm(cc: &str) -> &str {
    NORM_I386
        .iter()
        .find(|(k, _)| *k == cc)
        .map_or(cc, |(_, v)| v)
}
static DECL_BLOCK: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r#"(?s)(?P<cfg>(?:#\[cfg[^\]]*\]\s*)*)(?:unsafe\s+)?extern\s+"(?P<cc>\w+)"\s*\{(?P<body>[^}]*)\}"#).unwrap()
});
static DECL_FN: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r#"(?:#\[link_name\s*=\s*"(?P<link>[^"]+)"\]\s*)?(?:(?:pub(?:\([^)]*\))?|unsafe)\s+)*\bfn\s+(?P<name>\w+)\s*\("#).unwrap()
});
static DEF: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r#"(?P<cfg>(?:#\[cfg[^\]]*\]\s*)*)(?:#\[no_mangle\]|#\[export_name\s*=\s*"(?P<export>[^"]+)"\])\s*(?:pub(?:\([^)]*\))?\s+)?unsafe\s+extern\s+"(?P<cc>\w+)"\s+fn\s+(?P<name>\w+)"#).unwrap()
});
static CFG_MOD: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"#\[cfg\(([^\]]*)\)\]\s*(?:(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*)?\{")
        .unwrap()
});
static NOLINT: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"nolint:\s*cc-mismatch").unwrap());
static ARCH_FILE: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"(arm64|amd64|aarch64|x86_64)").unwrap());
static SYS_USE: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(
        r"\buse\s+(?P<crate>\w+_sys)(?P<path>(?:::\w+)*)::(?P<tail>\{[^;]*\}|\w+)\s*;",
    )
    .unwrap()
});
static SYS_CALL: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"\b(?P<crate>\w+_sys)(?P<path>(?:::\w+)*)::(?P<name>\w+)\s*\(").unwrap()
});
static ORPHAN: &str = "ORPHAN-BY-DESIGN:";

fn cfg_excludes_i386(cfg: &str) -> bool {
    if cfg.contains("not(target_arch = \"x86\")") {
        return true;
    }
    crate::regexes::cached(r#"#\[cfg\(target_arch\s*=\s*"(\w+)"\)\]"#)
        .unwrap()
        .captures(cfg)
        .is_some_and(|m| &m[1] != "x86")
}
fn excluded_spans(src: &str) -> Vec<(usize, usize)> {
    CFG_MOD
        .captures_iter(src)
        .filter_map(|m| {
            if !cfg_excludes_i386(&format!("#[cfg({})]", &m[1])) {
                return None;
            }
            let open = m.get(0)?.end() - 1;
            let mut depth = 0isize;
            for (i, b) in src.as_bytes()[open..].iter().enumerate() {
                if *b == b'{' {
                    depth += 1;
                } else if *b == b'}' {
                    depth -= 1;
                    if depth == 0 {
                        return Some((m.get(0).unwrap().start(), open + i));
                    }
                }
            }
            None
        })
        .collect()
}
fn in_spans(pos: usize, spans: &[(usize, usize)]) -> bool {
    spans.iter().any(|(a, b)| *a <= pos && pos <= *b)
}
fn py_list(values: impl IntoIterator<Item = String>) -> String {
    format!(
        "[{}]",
        values
            .into_iter()
            .map(|s| format!("'{}'", s.replace('\\', "\\\\").replace('\'', "\\'")))
            .collect::<Vec<_>>()
            .join(", ")
    )
}
fn symbol_def(m: &Captures<'_>) -> String {
    m.name("export")
        .or_else(|| m.name("name"))
        .unwrap()
        .as_str()
        .to_owned()
}
fn symbol_decl(m: &Captures<'_>) -> String {
    m.name("link")
        .or_else(|| m.name("name"))
        .unwrap()
        .as_str()
        .to_owned()
}

pub(crate) fn convention_facts(
    src: &str,
) -> (
    Vec<(String, String, bool)>,
    Vec<(String, String, usize, String)>,
) {
    let spans = excluded_spans(src);
    let defs = DEF
        .captures_iter(src)
        .map(|m| {
            (
                symbol_def(&m),
                m["cc"].into(),
                cfg_excludes_i386(m.name("cfg").unwrap().as_str())
                    || in_spans(m.get(0).unwrap().start(), &spans),
            )
        })
        .collect();
    let lines = src.lines().collect::<Vec<_>>();
    let mut decls = Vec::new();
    for block in DECL_BLOCK.captures_iter(src) {
        let whole = block.get(0).unwrap();
        let cfg = &block["cfg"];
        if cfg_excludes_i386(cfg) || in_spans(whole.start(), &spans) {
            continue;
        }
        let body = &block["body"];
        let body_start = block.name("body").unwrap().start();
        let block_line = src[..whole.start()].bytes().filter(|&b| b == b'\n').count();
        for d in DECL_FN.captures_iter(body) {
            let ds = d.get(0).unwrap().start();
            let line_prefix = &body[body[..ds].rfind('\n').map_or(0, |x| x + 1)..ds];
            if line_prefix.contains("//") {
                continue;
            }
            let line = block_line + body[..ds].bytes().filter(|&b| b == b'\n').count() + 1;
            decls.push((
                symbol_decl(&d),
                block["cc"].to_owned(),
                line,
                lines.get(line - 1).copied().unwrap_or("").to_owned(),
            ));
        }
        let _ = body_start;
    }
    (defs, decls)
}

fn crate_key(path: &Path) -> Option<String> {
    let text = path.to_string_lossy().replace('\\', "/");
    text.split("/src/").next().map(str::to_owned)
}
fn included_i386(path: &Path, text: &str) -> bool {
    let s = path.to_string_lossy().replace('\\', "/");
    if ARCH_FILE.is_match(path.file_name().and_then(|s| s.to_str()).unwrap_or(""))
        || s.contains("/target/")
    {
        return false;
    }
    !text
        .as_bytes()
        .get(..2048)
        .unwrap_or(text.as_bytes())
        .windows(ORPHAN.len())
        .any(|w| w == ORPHAN.as_bytes())
}
fn decl_split_findings(
    name: &str,
    decls: &[(String, String, usize, String, String)],
) -> Vec<String> {
    let ccs = decls.iter().map(|d| norm(&d.1)).collect::<BTreeSet<_>>();
    if ccs.len() < 2 {
        return vec![];
    }
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for (_, cc, _, _, path) in decls {
        let key = (path.clone(), cc.clone());
        if !seen.insert(key) {
            continue;
        }
        out.push(format!("{path}: [DECL_CC_SPLIT] {name} declared extern \"{cc}\" here; the crate also declares it {} (i386 esp/UB risk)", py_list(ccs.iter().filter(|&&x| x != norm(cc)).map(|s|s.to_string()))));
    }
    out
}

fn sys_module_has_body(
    crate_root: &str,
    module_path: &str,
    name: &str,
    all: &BTreeMap<String, Vec<&SourceFile>>,
) -> bool {
    let base = format!("{crate_root}/src");
    let mut module = module_path.trim_matches(':').replace("::", "/");
    if module.is_empty() {
        return false;
    }
    let prefix = format!("{base}/{module}");
    all.get(crate_root).into_iter().flatten().any(|f| {
        let path = f.path.to_string_lossy().replace('\\', "/");
        if !(path == format!("{prefix}.rs") || path == format!("{prefix}/mod.rs")) {
            return false;
        }
        let pat = crate::regexes::cached(&format!(
            r"(?m)^\s*pub\s+(?:unsafe\s+)?fn\s+{}\s*\([^;{{]*\)[^;{{]*\{{",
            regex::escape(name)
        ))
        .unwrap();
        pat.is_match(&f.text)
    })
}

fn crate_findings(
    files: &[&SourceFile],
    crate_name: &str,
    all: &BTreeMap<String, Vec<&SourceFile>>,
) -> Vec<String> {
    let mut defs = BTreeMap::<String, Vec<(String, bool)>>::new();
    let mut decls = BTreeMap::<String, Vec<(String, String, usize, String, String)>>::new();
    for f in files {
        if !included_i386(&f.path, &f.text) {
            continue;
        }
        let (ds, xs) = convention_facts(&f.text);
        let path = f.path.to_string_lossy().replace('\\', "/");
        for (n, cc, excluded) in ds {
            defs.entry(n).or_default().push((cc, excluded));
        }
        for (n, cc, line, text) in xs {
            if !NOLINT.is_match(&text) {
                decls
                    .entry(n.clone())
                    .or_default()
                    .push((n, cc, line, text, path.clone()));
            }
        }
        let lines = f.text.lines().collect::<Vec<_>>();
        for m in SYS_USE.captures_iter(&f.text) {
            let sys_ident = &m["crate"];
            let sys_root = format!("sdk/lib/rust/{}", sys_ident.replace('_', "-"));
            let module_path = &m["path"];
            let tail = &m["tail"];
            let names = if tail.starts_with('{') {
                tail.trim_matches(|c| c == '{' || c == '}')
                    .split(',')
                    .filter_map(|s| s.split_whitespace().next())
                    .filter(|s| !s.is_empty() && *s != "self")
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            } else {
                vec![tail.to_owned()]
            };
            for name in names {
                if sys_module_has_body(&sys_root, module_path, &name, all) {
                    continue;
                }
                if let Some(cc) = sys_decl(all, &sys_root, &name) {
                    let line = f.text[..m.get(0).unwrap().start()]
                        .bytes()
                        .filter(|&b| b == b'\n')
                        .count()
                        + 1;
                    let text = lines.get(line - 1).copied().unwrap_or("").to_owned();
                    if !NOLINT.is_match(&text) {
                        decls.entry(name.clone()).or_default().push((
                            name,
                            cc,
                            line,
                            text,
                            path.clone(),
                        ));
                    }
                }
            }
        }
        for m in SYS_CALL.captures_iter(&f.text) {
            let sys_ident = &m["crate"];
            let sys_root = format!("sdk/lib/rust/{}", sys_ident.replace('_', "-"));
            let name = &m["name"];
            if sys_module_has_body(&sys_root, &m["path"], name, all) {
                continue;
            }
            if let Some(cc) = sys_decl(all, &sys_root, name) {
                let line = f.text[..m.get(0).unwrap().start()]
                    .bytes()
                    .filter(|&b| b == b'\n')
                    .count()
                    + 1;
                let text = lines.get(line - 1).copied().unwrap_or("").to_owned();
                if !NOLINT.is_match(&text) {
                    decls.entry(name.to_owned()).or_default().push((
                        name.to_owned(),
                        cc,
                        line,
                        text,
                        path.clone(),
                    ));
                }
            }
        }
    }
    let mut out = Vec::new();
    for (name, ds) in decls {
        let Some(ddefs) = defs.get(&name) else {
            out.extend(decl_split_findings(&name, &ds));
            continue;
        };
        let defcc = ddefs
            .iter()
            .filter(|(_, e)| !e)
            .map(|(cc, _)| norm(cc).to_owned())
            .collect::<BTreeSet<_>>();
        if defcc.is_empty() {
            continue;
        }
        for (_, cc, _line, text, path) in ds {
            if NOLINT.is_match(&text) || defcc.contains(norm(&cc)) {
                continue;
            }
            out.push(format!("{path}: [CC_MISMATCH] {name} declared extern \"{cc}\" but defined {} in-crate (i386 UB-fold risk)",py_list(defcc.iter().cloned())));
        }
    }
    let _ = crate_name;
    out
}
fn sys_decl(
    all: &BTreeMap<String, Vec<&SourceFile>>,
    sys_root: &str,
    name: &str,
) -> Option<String> {
    all.get(sys_root)?
        .iter()
        .filter(|f| included_i386(&f.path, &f.text))
        .find_map(|f| {
            convention_facts(&f.text)
                .1
                .into_iter()
                .find(|d| d.0 == name)
                .map(|d| d.1)
        })
}

impl Rule for CcMismatch {
    fn name(&self) -> &'static str {
        "scan_cc_mismatch"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut groups = BTreeMap::<String, Vec<&SourceFile>>::new();
        for f in corpus.rust_files() {
            if let Some(k) = crate_key(&f.path) {
                groups.entry(k).or_default().push(f);
            }
        }
        let mut out = Vec::new();
        for (k, files) in &groups {
            out.extend(crate_findings(files, k, &groups));
        }
        out.sort();
        out.dedup();
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    fn corpus(files: &[(&str, &str)]) -> Corpus {
        Corpus::new(
            PathBuf::from("/repo"),
            files
                .iter()
                .map(|(p, s)| SourceFile::new((*p).into(), (*s).into()))
                .collect(),
        )
    }
    fn run(files: &[(&str, &str)]) -> Vec<String> {
        CcMismatch.run(&corpus(files)).unwrap()
    }

    #[test]
    fn original_buslogic_mismatch_and_nolint() {
        let src="extern \"C\" {\n pub fn BT958HwStartIO(dev: *mut u8) -> u8;\n}\n#[no_mangle]\npub unsafe extern \"stdcall\" fn BT958HwStartIO(dev: *mut u8) -> u8 { 0 }\n";
        let got = run(&[("sdk/lib/rust/buslogic-rust/src/hw.rs", src)]);
        assert_eq!(got,vec!["sdk/lib/rust/buslogic-rust/src/hw.rs: [CC_MISMATCH] BT958HwStartIO declared extern \"C\" but defined ['stdcall'] in-crate (i386 UB-fold risk)"]);
        let waived = src.replace("-> u8;", "-> u8; // nolint: cc-mismatch reviewed");
        assert!(run(&[("sdk/lib/rust/buslogic-rust/src/hw.rs", &waived)]).is_empty());
    }

    #[test]
    fn effective_symbol_and_i386_cfg_scope() {
        let src="extern \"C\" { #[link_name = \"RealTarget\"] pub fn decl_a(x:u32)->u32; }\n#[export_name = \"RealTarget\"] unsafe extern \"system\" fn actual(x:u32)->u32 {0}\n#[no_mangle]\nunsafe extern \"C\" fn decl_a(x:u32)->u32 {0}\n#[cfg(target_arch = \"x86_64\")] mod only64 { #[no_mangle] unsafe extern \"C\" fn Hidden() {} }\n";
        let got = run(&[("sdk/lib/rust/demo-rust/src/a.rs", src)]);
        assert_eq!(got.len(), 1, "{got:?}");
        assert!(got[0].contains("RealTarget") && got[0].contains("stdcall"));
    }

    #[test]
    fn declaration_split_and_commented_out_decl() {
        let got = run(&[
            (
                "sdk/lib/rust/demo-rust/src/a.rs",
                "extern \"C\" { fn KeRaiseIrql(n:u8,o:*mut u8); }\n",
            ),
            (
                "sdk/lib/rust/demo-rust/src/b.rs",
                "extern \"system\" { // DUPLICATE: fn KeRaiseIrql(n:u8,o:*mut u8);\n}\n",
            ),
        ]);
        assert!(got.is_empty(), "{got:?}");
        let got = run(&[
            (
                "sdk/lib/rust/demo-rust/src/a.rs",
                "extern \"C\" { fn X(); }\n",
            ),
            (
                "sdk/lib/rust/demo-rust/src/b.rs",
                "extern \"system\" { fn X(); }\n",
            ),
        ]);
        assert_eq!(got.len(), 2, "{got:?}");
        assert!(got.iter().all(|s| s.contains("[DECL_CC_SPLIT] X")));
    }

    #[test]
    fn sys_import_counts_unless_module_has_rust_body() {
        let got = run(&[
            (
                "sdk/lib/rust/demo-sys/src/bindgen_i386.rs",
                "extern \"C\" { pub fn IoCopyCurrentIrpStackLocationToNext(x:*mut u8); }\n",
            ),
            (
                "sdk/lib/rust/fat-rust/src/pnp.rs",
                "use demo_sys::ddk::{KeX, IoCopyCurrentIrpStackLocationToNext};\n",
            ),
            (
                "sdk/lib/rust/fat-rust/src/flush.rs",
                "extern \"system\" { fn IoCopyCurrentIrpStackLocationToNext(x:*mut u8); }\n",
            ),
        ]);
        assert!(
            got.iter()
                .any(|s| s.contains("pnp.rs") && s.contains("DECL_CC_SPLIT")),
            "{got:?}"
        );
        assert!(
            got.iter()
                .any(|s| s.contains("flush.rs") && s.contains("DECL_CC_SPLIT")),
            "{got:?}"
        );
        let inline = run(&[
            (
                "sdk/lib/rust/demo-sys/src/bindgen_i386.rs",
                "extern \"C\" { pub fn KeInitializeSpinLock(x:*mut usize); }\n",
            ),
            (
                "sdk/lib/rust/demo-sys/src/inline/ke.rs",
                "pub unsafe fn KeInitializeSpinLock(x:*mut usize) { *x=0; }\n",
            ),
            (
                "sdk/lib/rust/audio-rust/src/prelude.rs",
                "pub unsafe fn x(l:*mut usize){ demo_sys::inline::ke::KeInitializeSpinLock(l); }\n",
            ),
            (
                "sdk/lib/rust/audio-rust/src/types_i386.rs",
                "extern \"system\" { pub fn KeInitializeSpinLock(x:*mut usize); }\n",
            ),
        ]);
        assert!(inline.is_empty(), "{inline:?}");
    }

    #[test]
    fn arch_files_and_orphan_files_are_not_compiled_evidence() {
        let got=run(&[
            ("sdk/lib/rust/demo-rust/src/a.rs","extern \"C\" { fn X(); }\n"),
            ("sdk/lib/rust/demo-rust/src/types_arm64.rs","#[no_mangle] unsafe extern \"system\" fn X() {}\n"),
            ("sdk/lib/rust/demo-rust/src/parked.rs","//! ORPHAN-BY-DESIGN: intentionally parked\n#[no_mangle] unsafe extern \"system\" fn X() {}\n"),
        ]);
        assert!(got.is_empty(), "{got:?}");
    }
}
