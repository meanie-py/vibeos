//! Conversion proof and baseline plans; never writes or copies files.
use super::{constants, local, read, Cell, Error, Result, ARCHES};
use super::{graph, pe, render};
use regex::Regex;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
fn re(s: &str) -> Regex {
    Regex::new(s).expect("static proof regex")
}
fn set(name: &str) -> BTreeSet<String> {
    constants()[name]
        .as_array()
        .expect("proof constants")
        .iter()
        .map(|x| x.as_str().expect("string constant").to_owned())
        .collect()
}
fn target_set(name: &str, target: &str) -> BTreeSet<String> {
    constants()[name][target]
        .as_array()
        .map(|a| a.iter().map(|x| x.as_str().unwrap().to_owned()).collect())
        .unwrap_or_default()
}
fn indexed_baseline(baseline: &Path) -> Result<serde_json::Value> {
    let p = baseline.join("targets.json");
    if p.exists() {
        Ok(serde_json::from_str(&read(&p)?)?)
    } else {
        Ok(serde_json::json!({}))
    }
}
/// The arches a baseline holds `target` on: those its index names, or every arch for an
/// index written before gated cells (which always held all three).
pub fn indexed_arches(index: &serde_json::Value, target: &str) -> Vec<&'static str> {
    let named: Vec<_> = ARCHES
        .into_iter()
        .filter(|a| index[target][*a].as_str().is_some_and(|x| !x.is_empty()))
        .collect();
    if named.is_empty() {
        ARCHES.to_vec()
    } else {
        named
    }
}
fn rel_output(s: &str) -> Result<&Path> {
    let p = Path::new(s);
    if p.as_os_str().is_empty()
        || p.components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
    {
        return Err(Error::new(format!("invalid relative output path: {s}")));
    }
    Ok(p)
}
pub fn cell_crate(root: &Path, target: &str) -> Result<String> {
    if !target.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        return Ok(format!("{target}-cargo"));
    }
    let pat = re(&format!(
        r"add_cargo_module\({} (\S+)",
        regex::escape(target)
    ));
    let mut files = crate::source_walk::files(root).map_err(Error::new)?;
    files.sort();
    for f in files
        .into_iter()
        .filter(|p| p.file_name().is_some_and(|x| x == "CMakeLists.txt"))
    {
        let text = read(&local(root, &f)?)?;
        if let Some(c) = pat.captures(&text) {
            return Ok(c[1].to_owned());
        }
    }
    Err(Error::new(format!(
        "{target}: no add_cargo_module names it"
    )))
}
pub fn cell_is_bodyless(root: &Path, target: &str) -> Result<bool> {
    let p = root
        .join("sdk/lib/rust")
        .join(cell_crate(root, target)?)
        .join("build.rs");
    if !p.exists() {
        return Ok(false);
    }
    Ok(read(&local(root, &p)?)?.contains("there is no Rust body"))
}
/// Existing conversion baselines are immutable on a retry.
pub fn baseline_reusable(targets: &[String], baseline: &Path) -> Result<bool> {
    if !baseline.join("targets.json").exists() {
        return Ok(false);
    }
    let index = indexed_baseline(baseline)?;
    if !targets
        .iter()
        .flat_map(|t| indexed_arches(&index, t))
        .all(|a| baseline.join(a).is_dir())
    {
        return Ok(false);
    }
    let missing: Vec<_> = targets
        .iter()
        .filter(|t| index.get(t.as_str()).is_none())
        .collect();
    if !missing.is_empty() {
        return Err(Error::new(format!(
            "{} has no baseline for {:?}",
            baseline.display(),
            missing
        )));
    }
    Ok(true)
}
pub fn cell_uses_panic_common(root: &Path, target: &str, arch: &str) -> Result<bool> {
    let p = root
        .join("sdk/lib/rust")
        .join(cell_crate(root, target)?)
        .join("src/main.rs");
    let text = if p.exists() {
        read(&local(root, &p)?)?
    } else {
        String::new()
    };
    if !text.contains("panic_lean") {
        return Ok(true);
    }
    let pat = re(r"#\[cfg\((.*)\)\]\n\s*return panic_common");
    let arch = constants()["CFG_ARCH"][arch]
        .as_str()
        .ok_or_else(|| Error::new("unknown architecture"))?;
    Ok(pat
        .captures(&text)
        .is_some_and(|c| c[1].contains(&format!("\"{arch}\""))))
}
/// Return baseline candidates; immutable baseline trees may reside outside the checkout.
pub fn baseline_pe(baseline: &Path, arch: &str, target: &str) -> Result<Vec<PathBuf>> {
    let index = indexed_baseline(baseline)?;
    if let Some(out) = index[target][arch].as_str().filter(|x| !x.is_empty()) {
        return Ok(vec![baseline.join(arch).join(rel_output(out)?)]);
    }
    let dir = baseline.join(arch);
    if !dir.is_dir() {
        return Ok(vec![]);
    }
    fn walk(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_dir() {
                walk(&entry.path(), files)?;
            } else if kind.is_file() || (kind.is_symlink() && entry.path().is_file()) {
                files.push(entry.path());
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    walk(&dir, &mut files)?;
    let prefix = format!("{target}.");
    files.retain(|p| {
        p.file_name()
            .is_some_and(|x| x.to_string_lossy().starts_with(&prefix))
    });
    files.sort();
    Ok(files)
}
#[derive(Debug, Default)]
pub struct ReshellPlan {
    pub edits: BTreeMap<PathBuf, String>,
    pub notes: Vec<String>,
}
pub fn reshell(root: &Path, targets: &[String], baseline: &Path) -> Result<ReshellPlan> {
    let mut plan = ReshellPlan::default();
    let index = indexed_baseline(baseline)?;
    for t in targets {
        let mut panics = BTreeMap::new();
        let arches = indexed_arches(&index, t);
        for a in arches.iter().copied() {
            let olds = baseline_pe(baseline, a, t)?;
            if olds.len() != 1 {
                return Err(Error::new(format!(
                    "{t} {a}: {} baseline candidates",
                    olds.len()
                )));
            }
            panics.insert(a.to_owned(), pe::carries_panic_handler(&olds[0])?);
        }
        let crate_dir = root.join("sdk/lib/rust").join(cell_crate(root, t)?);
        let cargo = crate_dir.join("Cargo.toml");
        let toml = read(&local(root, &cargo)?)?;
        let start = toml
            .find("[dependencies]")
            .ok_or_else(|| Error::new(format!("{}: missing [dependencies]", cargo.display())))?;
        let end = toml.find("[build-dependencies]").ok_or_else(|| {
            Error::new(format!("{}: missing [build-dependencies]", cargo.display()))
        })?;
        if end < start {
            return Err(Error::new("build dependencies precede dependencies"));
        }
        let cell = Cell {
            target: t.clone(),
            panics: panics.clone(),
            arches: if arches.len() == ARCHES.len() {
                vec![]
            } else {
                arches.iter().map(|a| a.to_string()).collect()
            },
            ..Cell::default()
        };
        plan.edits.insert(
            cargo,
            format!(
                "{}{}{}",
                &toml[..start],
                render::panic_deps(&cell),
                &toml[end..]
            ),
        );
        let main = crate_dir.join("src/main.rs");
        local(root, &main)?;
        plan.edits.insert(main, render::main_rs(&cell)?);
        let kind = if panics.values().all(|v| *v) {
            "full".to_owned()
        } else if panics.values().all(|v| !*v) {
            "lean".to_owned()
        } else {
            format!("{:?}", panics)
        };
        plan.notes.push(format!("{t:14} {kind}"));
    }
    Ok(plan)
}
#[derive(Debug)]
pub struct CopyPlan {
    pub source: PathBuf,
    pub destination: PathBuf,
}
#[derive(Debug)]
pub struct BaselinePlan {
    pub copies: Vec<CopyPlan>,
    pub index_path: PathBuf,
    pub index_text: String,
}
pub fn save_baseline(root: &Path, targets: &[String], baseline: &Path) -> Result<BaselinePlan> {
    let mut index = indexed_baseline(baseline)?;
    let mut copies = Vec::new();
    for t in targets {
        for (a, link) in graph::link_edges_gated(root, t)? {
            let a = a.as_str();
            let out = link.output;
            let src = root.join(format!("build-{a}")).join(rel_output(&out)?);
            if !src.is_file() {
                return Err(Error::new(format!(
                    "{t}: no cmake-built {out} in build-{a}; build every arch first"
                )));
            }
            let src = local(root, &src)?;
            let cd = graph::cd_placement(root, a, &out)?;
            if !index.get(t).is_some_and(|v| v.is_object()) {
                index[t] = serde_json::json!({});
            }
            index[t][a] = out.clone().into();
            index[t][format!("cd:{a}")] = serde_json::to_value(cd)?;
            copies.push(CopyPlan {
                source: src,
                destination: baseline.join(a).join(out),
            });
        }
    }
    let fmt = serde_json::ser::PrettyFormatter::with_indent(b" ");
    let mut bytes = Vec::new();
    use serde::Serialize;
    index.serialize(&mut serde_json::Serializer::with_formatter(&mut bytes, fmt))?;
    Ok(BaselinePlan {
        copies,
        index_path: baseline.join("targets.json"),
        index_text: String::from_utf8(bytes).map_err(|e| Error::new(e.to_string()))?,
    })
}
#[derive(Debug, Clone)]
pub struct PolicyInput<'a> {
    pub target: &'a str,
    pub arch: &'a str,
    pub diffs: Vec<String>,
    pub panic_common: bool,
    pub data_size: Option<u32>,
    pub bodyless_sizes: Option<BTreeMap<String, u32>>,
    pub old_mapped: u64,
    pub new_mapped: u64,
    pub cd_old: Option<BTreeSet<String>>,
    pub cd_new: BTreeSet<String>,
    pub txtsetup: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyResult {
    pub diffs: Vec<String>,
    pub note: String,
}
pub fn apply_policy(input: PolicyInput<'_>) -> PolicyResult {
    let PolicyInput {
        target,
        arch,
        mut diffs,
        panic_common,
        data_size,
        bodyless_sizes,
        old_mapped,
        new_mapped,
        cd_old,
        cd_new,
        txtsetup,
    } = input;
    let common = set("PANIC_COMMON_IMPORTS");
    let probes_set = set("STACK_PROBE_IMPORTS");
    let rs_set = set("I386_RSBEGIN_SECTIONS");
    let dead_set = target_set("DEAD_IMPORTS", target);
    let rebound_set = target_set("REBOUND_EXPORTS", target);
    let extra: BTreeSet<_> = diffs
        .iter()
        .filter(|d| common.contains(d.strip_prefix("imports: + ").unwrap_or(d)))
        .cloned()
        .collect();
    let probes: BTreeSet<_> = diffs
        .iter()
        .filter(|d| {
            arch != "i386" && probes_set.contains(d.strip_prefix("imports: + ").unwrap_or(d))
        })
        .cloned()
        .collect();
    let mut rsbegin: BTreeSet<_> = diffs
        .iter()
        .filter(|d| arch == "i386" && rs_set.contains(*d))
        .cloned()
        .collect();
    if !rsbegin.is_empty()
        && data_size == constants()["I386_RSBEGIN_DATA"].as_u64().map(|n| n as u32)
    {
        rsbegin.extend(
            diffs
                .iter()
                .filter(|d| d.starts_with("sections: + .data "))
                .cloned(),
        );
    }
    let mut shell_eh = BTreeSet::new();
    if arch == "i386" {
        if let Some(sizes) = bodyless_sizes {
            for (name, bound) in [
                (
                    ".eh_fram",
                    constants()["I386_BODYLESS_EH_FRAME_MAX"].as_u64().unwrap() as u32,
                ),
                (
                    ".reloc",
                    constants()["I386_BODYLESS_RELOC_MAX"].as_u64().unwrap() as u32,
                ),
            ] {
                shell_eh.extend(
                    diffs
                        .iter()
                        .filter(|d| {
                            d.starts_with(&format!("sections: + {name} "))
                                && sizes.get(name).is_some_and(|n| *n <= bound)
                        })
                        .cloned(),
                );
            }
        }
    }
    rsbegin.extend(shell_eh.iter().cloned());
    let unwind: BTreeSet<_> = diffs
        .iter()
        .filter(|d| arch != "i386" && !panic_common && d.starts_with("sections: + .pdata "))
        .cloned()
        .collect();
    let dead: BTreeSet<_> = diffs
        .iter()
        .filter(|d| {
            dead_set.contains(d.strip_prefix("imports: - ").unwrap_or(d))
                || (d.starts_with("imports: - ")
                    && rebound_set.contains(d.rsplit('!').next().unwrap_or("")))
        })
        .cloned()
        .collect();
    let lean: BTreeSet<_> = diffs
        .iter()
        .filter(|d| !panic_common && d.starts_with("imports: - ") && !dead.contains(*d))
        .cloned()
        .collect();
    diffs.retain(|d| {
        !extra.contains(d)
            && !probes.contains(d)
            && !rsbegin.contains(d)
            && !unwind.contains(d)
            && !dead.contains(d)
            && !lean.contains(d)
    });
    let slack = constants()["SIZE_SLACK"].as_u64().expect("SIZE_SLACK");
    let grew = new_mapped.saturating_sub(old_mapped);
    if grew > slack {
        diffs.push(format!(
            "size: mapped image grew {} KB (slack {})",
            grew / 1024,
            slack / 1024
        ));
    }
    let rebound: BTreeSet<_> = diffs
        .iter()
        .filter(|d| {
            !rebound_set.is_empty()
                && (d.starts_with("imports: + ")
                    || d.starts_with("sections: + ")
                    || d.starts_with("size: mapped image grew"))
        })
        .cloned()
        .collect();
    diffs.retain(|d| !rebound.contains(d));
    let (mut loose, mut note_txtsetup) = (false, false);
    if let Some(old) = cd_old {
        let mut gone: BTreeSet<_> = old.difference(&cd_new).cloned().collect();
        let added: BTreeSet<_> = cd_new.difference(&old).cloned().collect();
        loose = added
            .iter()
            .any(|x| x.starts_with("bootcd:") || x.starts_with("bootcdregtest:"));
        if txtsetup {
            let n = gone.len();
            gone.retain(|x| !x.starts_with("cab:"));
            note_txtsetup = n != gone.len();
        }
        diffs.extend(gone.into_iter().map(|x| format!("cd: - {x}")));
        diffs.extend(
            added
                .into_iter()
                .filter(|x| !x.starts_with("bootcd:") && !x.starts_with("bootcdregtest:"))
                .map(|x| format!("cd: + {x}")),
        );
    }
    let mut note = String::new();
    if !extra.is_empty() {
        note.push_str(" (+ panic_common imports)");
    }
    if !rebound.is_empty() {
        note.push_str(&format!(
            " (rebound exports: linked the body; {} expected differences: {})",
            rebound.len(),
            rebound.into_iter().collect::<Vec<_>>().join("; ")
        ));
    }
    if loose {
        note.push_str(" (+ loose on the boot CD)");
    }
    if note_txtsetup {
        note.push_str(" (- cab: txtsetup.sif installs it)");
    }
    if rsbegin.difference(&shell_eh).next().is_some() {
        note.push_str(" (+ rsbegin .ctors/.dtors)");
    }
    if !shell_eh.is_empty() {
        note.push_str(&format!(
            " (+ the shell's i386 {})",
            shell_eh
                .iter()
                .map(|d| d.split_whitespace().nth(2).unwrap())
                .collect::<Vec<_>>()
                .join("/")
        ));
    }
    if !unwind.is_empty() {
        note.push_str(" (+ the lean handler's .pdata)");
    }
    if !probes.is_empty() {
        note.push_str(" (+ __chkstk: frame pointers)");
    }
    if !dead.is_empty() {
        note.push_str(" (- dead-code import, see DEAD_IMPORTS)");
    }
    if !lean.is_empty() {
        note.push_str(&format!(
            " (- {} imports only the panic path reached)",
            lean.len()
        ));
    }
    PolicyResult { diffs, note }
}
#[derive(Debug)]
pub struct CheckRecord {
    pub target: String,
    pub arch: String,
    pub status: String,
    pub diffs: Vec<String>,
    pub note: String,
}
#[derive(Debug)]
pub struct CheckResult {
    pub records: Vec<CheckRecord>,
    pub bad: usize,
}
impl CheckResult {
    pub fn report(&self) -> String {
        let mut lines = Vec::new();
        for r in &self.records {
            lines.push(format!(
                "{:14} {:6} {}{}",
                r.target,
                r.arch,
                r.status,
                if r.status == "EQUIVALENT" {
                    &r.note
                } else {
                    ""
                }
            ));
            lines.extend(r.diffs.iter().take(12).map(|d| format!("    {d}")));
        }
        format!("{}\n\n{} problem(s)\n", lines.join("\n"), self.bad)
    }
}
fn txtsetup_files(root: &Path) -> Result<BTreeSet<String>> {
    let path = local(root, &root.join("boot/bootdata/txtsetup.sif"))?;
    let txt = read(&path)?;
    Ok(re(r"(?m)^([A-Za-z0-9_.-]+)[ \t]*=")
        .captures_iter(&txt)
        .map(|c| c[1].to_lowercase())
        .collect())
}
pub fn check(root: &Path, targets: &[String], baseline: &Path) -> Result<CheckResult> {
    let index = indexed_baseline(baseline)?;
    let mut records = Vec::new();
    for t in targets {
        for a in indexed_arches(&index, t) {
            let olds = baseline_pe(baseline, a, t)?;
            let mut record = CheckRecord {
                target: t.clone(),
                arch: a.into(),
                status: String::new(),
                diffs: vec![],
                note: String::new(),
            };
            if olds.len() != 1 {
                record.status = format!("NO BASELINE ({} candidates)", olds.len());
                records.push(record);
                continue;
            }
            let rel = olds[0]
                .strip_prefix(baseline.join(a))
                .map_err(|e| Error::new(e.to_string()))?;
            let new = root.join(format!("build-{a}")).join(rel);
            if !new.is_file() {
                record.status = "NOT RELINKED (still the baseline PE: did the build fail?)".into();
                records.push(record);
                continue;
            }
            let new = local(root, &new)?;
            let old_data = std::fs::read(&olds[0])?;
            let new_data = std::fs::read(&new)?;
            if old_data == new_data {
                record.status = "NOT RELINKED (still the baseline PE: did the build fail?)".into();
                records.push(record);
                continue;
            }
            let compare = (|| -> Result<_> {
                let old = pe::Pe::parse(old_data)?;
                let new_pe = pe::Pe::parse(new_data)?;
                let diffs = pe::compare(&old, &new_pe, false)?;
                Ok((old, new_pe, diffs))
            })();
            let (old, new_pe, diffs) = match compare {
                Ok(x) => x,
                Err(e) => {
                    record.status = format!("UNREADABLE {e}");
                    records.push(record);
                    continue;
                }
            };
            let cd_value = &index[t][format!("cd:{a}")];
            let cd_old = if cd_value.is_null() {
                None
            } else {
                Some(serde_json::from_value(cd_value.clone())?)
            };
            let cd_new = if cd_old.is_some() {
                graph::cd_placement(root, a, &rel.to_string_lossy())?
                    .into_iter()
                    .collect()
            } else {
                BTreeSet::new()
            };
            let txtsetup = if cd_old.is_some() {
                txtsetup_files(root)?.contains(
                    &new.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_lowercase(),
                )
            } else {
                false
            };
            let policy = apply_policy(PolicyInput {
                target: t,
                arch: a,
                diffs,
                panic_common: cell_uses_panic_common(root, t, a)?,
                bodyless_sizes: if a == "i386" && cell_is_bodyless(root, t)? {
                    Some(new_pe.sections.iter().map(|s| (s.0.clone(), s.2)).collect())
                } else {
                    None
                },
                data_size: new_pe.sections.iter().find(|s| s.0 == ".data").map(|s| s.2),
                old_mapped: old.mapped_size(),
                new_mapped: new_pe.mapped_size(),
                cd_old,
                cd_new,
                txtsetup,
            });
            record.status = if policy.diffs.is_empty() {
                "EQUIVALENT"
            } else {
                "DIFFERENT"
            }
            .into();
            record.diffs = policy.diffs;
            record.note = policy.note;
            records.push(record);
        }
    }
    let bad = records.iter().filter(|r| r.status != "EQUIVALENT").count();
    Ok(CheckResult { records, bad })
}
