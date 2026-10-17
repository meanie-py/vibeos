use super::duplicate_symbols::{self, Unit};
use crate::link_model::{self, Def, ARCHES};
use crate::{Corpus, Rule, SourceSet};
use regex::Regex;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
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
    let Some(f) = c.files.iter().find(|f| f.path == d.path) else {
        return false;
    };
    f.text
        .lines()
        .skip(d.line.saturating_sub(6))
        .take(6)
        .any(|l| l.contains("nolint: export-shadow"))
}
fn fmt(d: &Def) -> String {
    format!("{} ({}, {} {})", d.loc(), d.krate, d.kind, d.name)
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
        let mut rows: BTreeMap<(String, String, String), BTreeSet<String>> = BTreeMap::new();
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
                        if d.kind != "rust"
                            || structural.is_match(&d.base())
                            || !d.cond.satisfiable_with(&d.cond, arch, &u.pinned)
                            || nolint(c, &d)
                        {
                            continue;
                        }
                        let Some(what) = names.get(arch).and_then(|m| m.get(&d.base())) else {
                            continue;
                        };
                        let e = rows
                            .entry((fmt(&d), d.name.clone(), what.clone()))
                            .or_default();
                        e.insert(arch.clone());
                    }
                }
            }
        }
        let rank = |a: &str| ARCHES.iter().position(|x| x == &a).unwrap_or(99);
        Ok(rows
            .into_iter()
            .map(|((loc, name, what), arches)| {
                let mut ar = arches.into_iter().collect::<Vec<_>>();
                ar.sort_by_key(|x| rank(x));
                format!("{loc}: {name}, a kernel export ({what}) [{}]", ar.join(","))
            })
            .collect())
    }
}
