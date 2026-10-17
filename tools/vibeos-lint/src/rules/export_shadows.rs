use super::duplicate_symbols;
use crate::link_model::{self, Def, ARCHES};
use crate::{Corpus, Rule, SourceSet};
use regex::Regex;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::LazyLock,
};

pub struct ExportShadows;
static SOURCES: &[SourceSet] = &[
    SourceSet::tracked("link-manifests", &[], &["Cargo.toml"]),
    SourceSet::tree(
        "link-inputs",
        &["sdk/lib/rust"],
        &["rs", "s", "S", "asm"],
        &[],
    )
    .with_inputs(),
    SourceSet::tree(
        "link-build-inputs",
        &["sdk/lib/rust"],
        &["rs"],
        &["build.rs"],
    )
    .with_inputs(),
    SourceSet::tracked("export-specs", &["spec"], &["ntoskrnl.spec", "hal.spec"]).with_inputs(),
];
static LINE: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"^\s*@\s+(\w+)\s+(.*)$").unwrap());
static RANGE: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"^(?:0x)?([0-9A-Fa-f]+)(?:(\+)|-(?:0x)?([0-9A-Fa-f]+))?$ ".trim())
        .unwrap()
});
static IDENT: LazyLock<Regex> = LazyLock::new(|| crate::regexes::cached(r"^\w+$").unwrap());
const EXPORT_VERSION: u32 = 0x502;
const EXPORTERS: &[&str] = &[
    "ntoskrnl-cargo",
    "ntkrnlmp-cargo",
    "kernel-rust",
    "hal-cargo",
    "halmp-cargo",
    "hal-rust",
];
const NOT_DRIVERS: &[&str] = &[
    "rosload-cargo",
    "uefildr-cargo",
    "ntdll-cargo",
    "ntdll-rust",
    "ntdll_vista-cargo",
];
const STRUCTURAL: &str =
    r"^(rust_eh_personality|rust_begin_unwind|rust_oom|__rust_[a-z_]+|_fltused)$";
fn version_included(flags: &[&str]) -> Result<bool, String> {
    for f in flags {
        let Some(spec) = f.strip_prefix("-version=") else {
            continue;
        };
        for r in spec.split(',') {
            let c = RANGE
                .captures(r)
                .ok_or_else(|| format!("error: unparsed spec version range {f:?}"))?;
            let lo = u32::from_str_radix(&c[1], 16).unwrap();
            let hi = if c.get(2).is_some() {
                0xfff
            } else {
                c.get(3)
                    .map_or(lo, |x| u32::from_str_radix(x.as_str(), 16).unwrap())
            };
            if lo <= EXPORT_VERSION && EXPORT_VERSION <= hi {
                return Ok(true);
            }
        }
        return Ok(false);
    }
    Ok(true)
}
fn spec_arches(flags: &[&str]) -> BTreeSet<&'static str> {
    for f in flags {
        if let Some(s) = f.strip_prefix("-arch=") {
            let vals = s.split(',').collect::<Vec<_>>();
            let map = |x: &str| match x {
                "i386" => Some("x86"),
                "x86_64" | "amd64" => Some("x86_64"),
                "arm64" => Some("aarch64"),
                _ => None,
            };
            let neg = vals
                .iter()
                .filter_map(|a| a.strip_prefix('!').and_then(map))
                .collect::<BTreeSet<_>>();
            let pos = vals.iter().filter_map(|a| map(a)).collect::<BTreeSet<_>>();
            if !neg.is_empty() {
                return ARCHES
                    .iter()
                    .copied()
                    .filter(|a| !neg.contains(a))
                    .collect();
            }
            return pos;
        }
    }
    ARCHES.iter().copied().collect()
}
fn kernel_names(c: &Corpus) -> Result<BTreeMap<String, BTreeMap<String, String>>, String> {
    let mut out: BTreeMap<String, BTreeMap<String, String>> = ARCHES
        .iter()
        .map(|a| (a.to_string(), BTreeMap::new()))
        .collect();
    for wanted in ["ntoskrnl.spec", "hal.spec"] {
        let f = c
            .sources("export-specs")
            .find(|f| f.path.file_name().is_some_and(|n| n == wanted))
            .ok_or_else(|| format!("error: {wanted} missing"))?;
        for raw in f.text.lines() {
            let line = raw.split('#').next().unwrap_or("");
            let Some(m) = LINE.captures(line) else {
                continue;
            };
            let directive = &m[1];
            let toks = m[2].split_whitespace().collect::<Vec<_>>();
            let flags = toks
                .iter()
                .copied()
                .filter(|t| t.starts_with('-'))
                .collect::<Vec<_>>();
            let rest = toks
                .iter()
                .copied()
                .filter(|t| !t.starts_with('-'))
                .collect::<Vec<_>>();
            if rest.is_empty()
                || flags
                    .iter()
                    .any(|x| *x == "-stub" || x.starts_with("-stub="))
                || directive == "stub"
                || !version_included(&flags)?
            {
                continue;
            }
            let export = rest[0].split(['(', ' ']).next().unwrap_or(rest[0]);
            let mut names = BTreeSet::from([export.to_string()]);
            let tail = if let Some((_, after)) = m[2].split_once(')') {
                after.split_whitespace().collect::<Vec<_>>()
            } else {
                rest.get(1..).unwrap_or(&[]).to_vec()
            };
            if let Some(x) = tail.into_iter().find(|x| !x.starts_with('-')) {
                names.insert(x.rsplit('.').next().unwrap_or(x).to_string());
            }
            for a in spec_arches(&flags) {
                for n in &names {
                    if IDENT.is_match(n) {
                        out.get_mut(a)
                            .unwrap()
                            .entry(n.clone())
                            .or_insert_with(|| format!("{}: {export}", wanted));
                    }
                }
            }
        }
    }
    Ok(out)
}
fn nolint(c: &Corpus, d: &Def) -> bool {
    let Some(file) = c.file(&d.path) else {
        return false;
    };
    (d.line.saturating_sub(6) + 1..=d.line)
        .any(|line| file.line_text(line).contains("nolint: export-shadow"))
}
impl Rule for ExportShadows {
    fn name(&self) -> &'static str {
        "scan_export_shadows"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        SOURCES
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let names = kernel_names(c)?;
        let units = duplicate_symbols::link_units(c);
        let all = link_model::crates(c);
        let mut exporter_crates = BTreeSet::new();
        for n in EXPORTERS.iter().chain(NOT_DRIVERS) {
            if let Some(u) = units.get(*n) {
                for x in u.values().flat_map(|x| &x.crates) {
                    exporter_crates.insert(x.clone());
                }
            }
        }
        let structural = crate::regexes::cached(STRUCTURAL).unwrap();
        // Python keys findings by Def, then keeps the first matching spec description while
        // accumulating arches. Keep the description as the value, not part of the row key.
        let mut rows: BTreeMap<(String, String, String), (String, BTreeSet<String>)> =
            BTreeMap::new();
        for (unit, per) in &units {
            if EXPORTERS.contains(&unit.as_str()) || NOT_DRIVERS.contains(&unit.as_str()) {
                continue;
            }
            for (arch, u) in per {
                if !u.crates.iter().any(|x| x == "ntkrnl-sys") {
                    continue;
                }
                for cn in &u.crates {
                    if exporter_crates.contains(cn) {
                        continue;
                    }
                    let Some(k) = all.get(cn) else { continue };
                    for d in link_model::crate_defs(c, k) {
                        if d.kind != "rust" {
                            continue;
                        }
                        // Most definitions are not kernel exports. Resolve the name
                        // before the cfg and source-suppression checks.
                        let Some(what) = names.get(arch).and_then(|m| m.get(&d.name)) else {
                            continue;
                        };
                        if structural.is_match(&d.name)
                            || !d.cond.satisfiable_with(&d.cond, arch, &u.pinned)
                            || nolint(c, &d)
                        {
                            continue;
                        }
                        let (_, arches) = rows
                            .entry((d.loc(), d.name.clone(), d.krate.clone()))
                            .or_insert_with(|| (what.clone(), BTreeSet::new()));
                        arches.insert(arch.to_string());
                    }
                }
            }
        }
        Ok(rows
            .into_iter()
            .map(|((loc, name, krate), (what, arches))| {
                let ar = ARCHES
                    .iter()
                    .filter(|a| arches.contains(**a))
                    .copied()
                    .collect::<Vec<_>>();
                format!(
                    "{loc}: {krate} defines {name}, a kernel export ({what}) [{}]",
                    ar.join(",")
                )
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arch_rows_keep_first_spec_description_and_combine_arches() {
        let mut rows: BTreeMap<(String, String, String), (String, BTreeSet<String>)> =
            BTreeMap::new();
        for (arch, what) in [
            ("x86", "hal.spec: KeLowerIrql"),
            ("x86_64", "ntoskrnl.spec: KeLowerIrql"),
        ] {
            let (_, arches) = rows
                .entry((
                    "file.rs:495".into(),
                    "KeLowerIrql".into(),
                    "classpnp-rust".into(),
                ))
                .or_insert_with(|| (what.to_string(), BTreeSet::new()));
            arches.insert(arch.to_string());
        }
        let ((_, _, _), (what, arches)) = rows.into_iter().next().unwrap();
        assert_eq!(what, "hal.spec: KeLowerIrql");
        assert_eq!(
            ARCHES
                .iter()
                .filter(|a| arches.contains(**a))
                .copied()
                .collect::<Vec<_>>(),
            ["x86", "x86_64"]
        );
    }
    #[test]
    fn python_version_probe_fixtures() {
        assert!(version_included(&["-arch=x86_64"]).unwrap());
        assert!(!version_included(&["-version=0x602+"]).unwrap());
        assert!(!version_included(&["-version=0xA00+"]).unwrap());
        assert!(version_included(&["-version=0x400-0x502"]).unwrap());
        assert!(!version_included(&["-version=0x400-0x501"]).unwrap());
        assert!(version_included(&["-version=0x600+,0x502"]).unwrap());
        assert!(version_included(&["-version=0x502+"]).unwrap());
    }

    #[test]
    fn spec_export_and_forward_target_are_both_names_but_stubs_are_not() {
        let c=Corpus::new(std::path::PathBuf::from("."),vec![
            crate::SourceFile::new("ntoskrnl/ntoskrnl.spec".into(), "@ stdcall -arch=x86_64,arm64 ExpInterlockedPopEntrySList(ptr ptr) RtlInterlockedPopEntrySList\n@ stub DeadStub\n@ stdcall -version=0x602+ PoFxLater()\n".into()),
            crate::SourceFile::new("hal/hal.spec".into(), "@ extern HalPort\n".into()),
        ]);
        let names = kernel_names(&c).unwrap();
        for arch in ["x86_64", "aarch64"] {
            assert_eq!(
                names[arch]["ExpInterlockedPopEntrySList"],
                "ntoskrnl.spec: ExpInterlockedPopEntrySList"
            );
            assert_eq!(
                names[arch]["RtlInterlockedPopEntrySList"],
                "ntoskrnl.spec: ExpInterlockedPopEntrySList"
            );
            assert!(!names[arch].contains_key("DeadStub"));
            assert!(!names[arch].contains_key("PoFxLater"));
        }
        assert!(!names["x86"].contains_key("ExpInterlockedPopEntrySList"));
    }

    #[test]
    fn scanner_control_bad_forward_target_is_reported_and_nolint_good_is_quiet() {
        let specs=[("ntoskrnl/ntoskrnl.spec", "@ stdcall -arch=x86_64,arm64 ExpInterlockedPopEntrySList(ptr ptr) RtlInterlockedPopEntrySList\n@ stdcall KeInitializeSpinLock(ptr)\n"), ("hal/hal.spec","@ extern KdComPortInUse\n")];
        let manifest="[package]\nname = \"drv-rust\"\nversion = \"0.1.0\"\n\n[lib]\ncrate-type = [\"staticlib\"]\npath = \"src/lib.rs\"\n\n[dependencies]\nntkrnl-sys = { path = \"../ntkrnl-sys\" }\n";
        let sys = "[package]\nname = \"ntkrnl-sys\"\nversion = \"0.1.0\"\n";
        let mk = |good: bool| {
            let mut fs = specs
                .iter()
                .map(|(p, t)| crate::SourceFile::new((*p).into(), (*t).into()))
                .collect::<Vec<_>>();
            fs.push(crate::SourceFile::new(
                "ntkrnl-sys/Cargo.toml".into(),
                sys.into(),
            ));
            fs.push(crate::SourceFile::new(
                "ntkrnl-sys/src/lib.rs".into(),
                String::new(),
            ));
            fs.push(crate::SourceFile::new(
                "drv-rust/Cargo.toml".into(),
                manifest.into(),
            ));
            let body = if good {
                "// nolint: export-shadow (wdm.h FORCEINLINE; expected local body)\n#[no_mangle]\npub unsafe extern \"C\" fn KeInitializeSpinLock(l: *mut usize) { *l = 0; }\n#[no_mangle]\npub unsafe extern \"system\" fn DrvPopEntry(_h: PVOID) -> PVOID { null_mut() }\n"
            } else {
                "#[no_mangle]\npub unsafe extern \"system\" fn RtlInterlockedPopEntrySList(_h: PVOID) -> PVOID { null_mut() }\n"
            };
            fs.push(crate::SourceFile::new(
                "drv-rust/src/lib.rs".into(),
                body.into(),
            ));
            Corpus::new(std::path::PathBuf::from("."), fs)
        };
        let bad = ExportShadows.run(&mk(false)).unwrap();
        assert!(
            bad.iter().any(
                |x| x.contains("drv-rust defines RtlInterlockedPopEntrySList")
                    && x.contains("[x86_64,aarch64]")
            ),
            "{bad:#?}"
        );
        assert!(ExportShadows.run(&mk(true)).unwrap().is_empty());
    }
}
