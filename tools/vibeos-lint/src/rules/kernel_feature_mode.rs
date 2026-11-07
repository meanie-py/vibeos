//! `feature = "kernel"` read as "this copy runs in the kernel", in a crate whose `kernel`
//! feature ntdll gets too.
//!
//! In the rtl crates `kernel` only switches on the `#[no_mangle]` export surface
//! (`kernel = ["cabi"]`), and rtl-rust turns it on in its DEFAULT features, which is what
//! ntdll links. So a `#[cfg(not(feature = "kernel"))]` user-mode path is compiled out of the
//! shipping ntdll as well as the kernel: rtl-sync's SRW backoff and keyed-event wait ran only
//! in the host harness for weeks, and fault64 waited ~41 ms per contended acquire on a lock
//! that spun (2026-11-06). The kernel's own copy is `feature = "ntoskrnl"`, which only the
//! ntoskrnl/ntkrnlmp links ask for. The crates are read from rtl-rust's `kernel` feature, so a
//! crate added there is covered without editing this rule.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::LazyLock;
pub struct KernelFeatureMode;
impl Rule for KernelFeatureMode {
    fn name(&self) -> &'static str {
        "scan_kernel_feature_mode"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let manifest = corpus.root.join("sdk/lib/rust/rtl-rust/Cargo.toml");
        let text = std::fs::read_to_string(&manifest)
            .map_err(|e| format!("{}: {e}", manifest.display()))?;
        let crates = shared_kernel_crates(&text);
        let mut out = Vec::new();
        for file in corpus.rust_files() {
            let rel = file.path.strip_prefix(&corpus.root).unwrap_or(&file.path);
            let rel = rel.to_string_lossy().replace('\\', "/");
            let Some(rest) = rel.strip_prefix("sdk/lib/rust/") else {
                continue;
            };
            let Some((krate, _)) = rest.split_once("/src/") else {
                continue;
            };
            if !crates.iter().any(|c| c == krate) {
                continue;
            }
            for line in scan_text(&file.text) {
                out.push(format!(
                    "{}:{line}: `feature = \"kernel\"` names the export surface, which ntdll \
                     also links (rtl-rust default features), not kernel mode; for the kernel's \
                     own copy use `feature = \"ntoskrnl\"`",
                    file.path.display()
                ));
            }
        }
        Ok(out)
    }
}
/// The crates whose `kernel` feature rtl-rust's `kernel` turns on, when its defaults include
/// `kernel` (ntdll's link). Empty otherwise: then `kernel` would mean kernel mode again.
fn shared_kernel_crates(manifest: &str) -> Vec<String> {
    static FEATURE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"(?m)^\s*(default|kernel)\s*=\s*\[([^\]]*)\]"#).unwrap());
    static ITEM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#""([^"]+)""#).unwrap());
    let mut default_has_kernel = false;
    let mut crates = Vec::new();
    for f in FEATURE.captures_iter(manifest) {
        let items: Vec<String> = ITEM
            .captures_iter(&f[2])
            .map(|m| m[1].to_string())
            .collect();
        if &f[1] == "default" {
            default_has_kernel = items.iter().any(|i| i == "kernel");
        } else {
            crates = items
                .iter()
                .filter_map(|i| i.strip_suffix("/kernel").map(str::to_string))
                .collect();
        }
    }
    if default_has_kernel {
        crates
    } else {
        Vec::new()
    }
}
fn scan_text(text: &str) -> Vec<usize> {
    static CFG: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"cfg(?:_attr)?!?\s*\(.*feature\s*=\s*"kernel""#).unwrap());
    text.lines()
        .enumerate()
        .filter(|(_, l)| {
            let t = l.trim_start();
            !t.starts_with("//") && CFG.is_match(l) && !l.contains("nolint: kernel-feature-mode")
        })
        .map(|(i, _)| i + 1)
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dedicated_fixture() {
        let text = "#[cfg(not(feature = \"kernel\"))]\nfn a() {}\nif cfg!(feature = \"kernel\") {}\n#[cfg_attr(feature = \"kernel\", allow(dead_code))]\n";
        assert_eq!(scan_text(text), vec![1, 3, 4]);
    }
    #[test]
    fn scanner_controls() {
        assert!(
            scan_text("#[cfg(feature = \"ntoskrnl\")]\n// cfg(feature = \"kernel\")\n").is_empty()
        );
        let m = "default = [\"kernel\", \"heaplist\"]\nkernel = [\"rtl-sync/kernel\", \"rtl-heap/kernel\"]\n";
        assert_eq!(shared_kernel_crates(m), vec!["rtl-sync", "rtl-heap"]);
        let m = "default = [\"heaplist\"]\nkernel = [\"rtl-sync/kernel\"]\n";
        assert!(shared_kernel_crates(m).is_empty());
    }
}
