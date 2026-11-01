//! A payload probe calling through `lpVtbl->` on an interface mingw-w64 declares with
//! `DECLARE_INTERFACE_`.
//!
//! The probes under tools/payloads are plain C built against the llvm-mingw headers. There
//! `DECLARE_INTERFACE_(IFoo, IUnknown)` expands to a vtable holding only the methods its body
//! lists: C has no inheritance, so IUnknown's three slots are missing and every member sits
//! three slots early. `ei->lpVtbl->GetIconLocation` compiles and calls QueryInterface.
//! ntobjshex's probe did exactly that: GIL_FORSHELL became the IID pointer, the call faulted
//! at address 2 in the C and the Rust alike, and it was filed as an extension bug for weeks.
//! Such an interface has no COBJMACROS either, so `lpVtbl->` is the only way to reach the
//! trap. Call through a local vtable struct that spells out the IUnknown slots.
use crate::{Corpus, Rule};
use regex::Regex;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub struct MingwCVtable;
impl Rule for MingwCVtable {
    fn name(&self) -> &'static str {
        "scan_mingw_c_vtable"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        // Explicit paths (a fixture, or a file named on the command line) replace the walk.
        let mut probes = Vec::new();
        for path in corpus.explicit_paths() {
            let path = corpus.root.join(path);
            if path.is_dir() {
                probes.extend(c_files(&path)?);
            } else if path.extension().is_some_and(|e| e == "c") {
                probes.push(path);
            }
        }
        if corpus.explicit_paths().is_empty() {
            probes = c_files(&corpus.root.join("tools/payloads"))?;
        }
        probes.retain(|p| p.extension().is_some_and(|e| e == "c"));
        if probes.is_empty() {
            return Ok(vec![]);
        }
        let include = corpus
            .root
            .join(".build-tools/llvm-mingw/generic-w64-mingw32/include");
        if !include.is_dir() {
            return Err(format!(
                "scan_mingw_c_vtable: {} is missing; the payload probes build against it \
                 (fix: ./vibeos-cli build --arch i386 provisions the toolchain)",
                include.display()
            ));
        }
        let mut short = BTreeSet::new();
        for header in c_files(&include)? {
            // Some mingw headers carry Latin-1 bytes in comments.
            let bytes = std::fs::read(&header).map_err(|e| format!("{}: {e}", header.display()))?;
            short.extend(short_interfaces(&String::from_utf8_lossy(&bytes)));
        }
        let mut findings = Vec::new();
        for probe in probes {
            let text = std::fs::read_to_string(&probe)
                .map_err(|e| format!("{}: {e}", probe.display()))?;
            for (line, iface, method) in scan_probe(&text, &short) {
                findings.push(format!(
                    "{}:{line}: {iface}->lpVtbl->{method} on a mingw DECLARE_INTERFACE_ \
                     vtable, which in C lacks IUnknown's slots; call through a vtable struct \
                     that lists them",
                    probe.display()
                ));
            }
        }
        findings.sort();
        Ok(findings)
    }
}

struct Patterns {
    declare: Regex,
    call: Regex,
}
fn patterns() -> &'static Patterns {
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(|| Patterns {
        declare: crate::regexes::cached(r"DECLARE_INTERFACE_(?:IID_)?\s*\(\s*(\w+)\s*,").unwrap(),
        call: crate::regexes::cached(r"\b(\w+)\s*->\s*lpVtbl\s*->\s*(\w+)").unwrap(),
    })
}

/// .c and .h files under `dir`, recursively; a missing dir is empty.
fn c_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for entry in entries {
            let path = entry.map_err(|e| format!("{}: {e}", d.display()))?.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "c" || e == "h") {
                out.push(path);
            }
        }
    }
    out.sort();
    Ok(out)
}

/// Interfaces declared with DECLARE_INTERFACE_ whose body does not list QueryInterface.
fn short_interfaces(text: &str) -> Vec<String> {
    patterns()
        .declare
        .captures_iter(text)
        .filter_map(|m| {
            let body_start = text[m.get(0)?.end()..].find('{')? + m.get(0)?.end();
            let body_end = text[body_start..].find("};").map_or(text.len(), |e| body_start + e);
            let body = &text[body_start..body_end];
            (!body.contains("QueryInterface")).then(|| m[1].to_string())
        })
        .collect()
}

/// (line, interface, method) for each `var->lpVtbl->Method` whose `var` is declared as a
/// pointer to a short interface in the same file.
fn scan_probe(text: &str, short: &BTreeSet<String>) -> Vec<(usize, String, String)> {
    patterns()
        .call
        .captures_iter(text)
        .filter_map(|m| {
            let var = &m[1];
            let decl = Regex::new(&format!(r"\b(\w+)\s*\*\s*{}\b", regex::escape(var))).ok()?;
            let iface = decl
                .captures_iter(text)
                .map(|d| d[1].to_string())
                .find(|t| short.contains(t))?;
            let line = text[..m.get(0)?.start()].matches('\n').count() + 1;
            Some((line, iface, m[2].to_string()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = r#"
#define INTERFACE IExtractIconW
  DECLARE_INTERFACE_IID_ (IExtractIconW, IUnknown, "000214fa-0000-0000-c000-000000000046") {
    STDMETHOD(GetIconLocation) (THIS_ UINT uFlags) PURE;
  };
#define INTERFACE IFull
  DECLARE_INTERFACE_(IFull, IUnknown) {
    STDMETHOD(QueryInterface) (THIS_ REFIID, void **) PURE;
    STDMETHOD(Go) (THIS) PURE;
  };
"#;

    #[test]
    fn short_only_when_body_lacks_iunknown() {
        assert_eq!(short_interfaces(HEADER), vec!["IExtractIconW".to_string()]);
    }

    #[test]
    fn flags_a_call_through_a_short_vtable() {
        let short: BTreeSet<_> = short_interfaces(HEADER).into_iter().collect();
        let probe = "IExtractIconW *ei = 0;\nIFull *f = 0;\n\
                     ei->lpVtbl->GetIconLocation(ei, 2);\nf->lpVtbl->Go(f);\n";
        assert_eq!(
            scan_probe(probe, &short),
            vec![(3, "IExtractIconW".to_string(), "GetIconLocation".to_string())]
        );
    }

    #[test]
    fn a_local_vtable_struct_is_clean() {
        let short: BTreeSet<_> = short_interfaces(HEADER).into_iter().collect();
        let probe = "IExtractIconW *ei = 0;\n(*(Slots **)ei)->GetIconLocation(ei, 2);\n";
        assert!(scan_probe(probe, &short).is_empty());
    }
}
