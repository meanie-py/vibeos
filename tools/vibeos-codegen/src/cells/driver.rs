//! Cargo-cell conversion orchestration. Planning is read-only; execution is injected.
use super::{graph, local, proof, read, recipes, render, rewrite, Cell, Error, Result, ARCHES};
use regex::Regex;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    process::Command,
};
#[derive(Debug)]
pub struct ApplyPlan {
    pub edits: BTreeMap<PathBuf, String>,
    pub created_crates: Vec<PathBuf>,
    pub report: Vec<PathBuf>,
}
/// Plan an entire wave in memory; shared workspace and CMake files compose sequentially.
pub fn plan_apply(root: &Path, cells: &[Cell]) -> Result<ApplyPlan> {
    let rust = root.join("sdk/lib/rust");
    local(root, &rust)?;
    let ws = rust.join("Cargo.toml");
    let mut workspace = read(&local(root, &ws)?)?;
    let mut edits: BTreeMap<PathBuf, String> = BTreeMap::new();
    let mut created_crates = Vec::new();
    let mut report = Vec::new();
    let mut names = BTreeSet::new();
    let member_re = Regex::new(r"members = \[\n").unwrap();
    for cell in cells {
        let name = cell.crate_name();
        if name.contains('/') || name.contains('\\') || name == "." || name == ".." {
            return Err(Error::new(format!("invalid cargo crate name {name}")));
        }
        let dest = rust.join(&name);
        if dest.exists() || dest.is_symlink() || !names.insert(name.clone()) {
            return Err(Error::new(format!(
                "{} already exists",
                dest.strip_prefix(root).unwrap_or(&dest).display()
            )));
        }
        if !cell.body_crate.is_empty() {
            let body = rust.join(&cell.body_crate);
            if !body.is_dir() {
                return Err(Error::new(format!(
                    "{}: body crate {} not in the workspace",
                    cell.target, cell.body_crate
                )));
            }
            local(root, &body)?;
        }
        let cmake = cell.cmake_dir.join("CMakeLists.txt");
        let cmake = if cmake.is_absolute() {
            cmake
        } else {
            root.join(cmake)
        };
        local(root, &cmake)?;
        let text = match edits.get(&cmake) {
            Some(x) => x.clone(),
            None => read(&cmake)?,
        };
        let new_cmake = rewrite::rewrite_text(&cmake, &text, cell)?;
        let rs = render::build_rs(cell)?;
        let toml = render::cargo_toml(cell)?;
        let main = render::main_rs(cell)?;
        let member = format!("    \"{name}\",\n");
        if !workspace.contains(&member) {
            if !member_re.is_match(&workspace) {
                return Err(Error::new(format!(
                    "{}: workspace members list not found",
                    ws.display()
                )));
            }
            workspace = member_re
                .replacen(&workspace, 1, format!("members = [\n{member}"))
                .into_owned();
        }
        edits.insert(dest.join("Cargo.toml"), toml);
        edits.insert(dest.join("build.rs"), rs);
        edits.insert(dest.join("src/main.rs"), main);
        edits.insert(cmake.clone(), new_cmake);
        created_crates.push(dest.clone());
        report.extend([dest, ws.clone(), cmake]);
    }
    if !cells.is_empty() {
        edits.insert(ws, workspace);
    }
    Ok(ApplyPlan {
        edits,
        created_crates,
        report,
    })
}
/// Survey amd64 placeholder-stub edges; each model refusal remains a row.
pub fn survey(root: &Path) -> Result<Vec<(String, String)>> {
    let path = local(root, &root.join("build-amd64/build.ninja"))?;
    let ninja = read(&path)?;
    let re = Regex::new(r"(?m)^build \S+: C_\w+?_LINKER__(\w+?)_(?:Debug|Release)\S* ([^|\n]*)")
        .unwrap();
    let mut rows = Vec::new();
    for c in re.captures_iter(&ninja) {
        if !c[2]
            .split_whitespace()
            .any(|x| x.rsplit('/').next() == Some("rust_cohab_stub.c.obj"))
        {
            continue;
        }
        let t = &c[1];
        let verdict = match graph::model(root, t) {
            Ok(cell) => format!("OK {}", cell.module_kind),
            Err(e) => format!("REFUSED {e}"),
        };
        rows.push((t.to_owned(), verdict));
    }
    rows.sort();
    Ok(rows)
}
pub fn survey_report(rows: &[(String, String)]) -> String {
    let mut text = String::new();
    for (t, v) in rows {
        text.push_str(&format!("{t:24} {v}\n"));
    }
    let ok = rows.iter().filter(|(_, v)| v.starts_with("OK")).count();
    text.push_str(&format!(
        "\n{ok} of {} placeholder-stub modules generate cleanly\n",
        rows.len()
    ));
    text
}
#[derive(Debug)]
pub struct ConvertPlan {
    pub targets: Vec<String>,
    /// Per arch, the wave's targets built there (a gated cell is absent on some).
    pub targets_by_arch: BTreeMap<String, Vec<String>>,
    pub baseline_dir: PathBuf,
    pub baseline: proof::BaselinePlan,
    pub apply: ApplyPlan,
}
pub fn prepare_convert(root: &Path, targets: &[String], baseline: &Path) -> Result<ConvertPlan> {
    if targets.is_empty() {
        return Err(Error::new("--convert needs at least one target"));
    }
    // Model and dry-run the rewrite of EVERY target before refusing, and name all the
    // refusals at once: stopping at the first made a 17-module wave cost one baseline
    // build round per refused module (dbghelp, then libtirpc, ...).
    let mut cells = Vec::new();
    let mut refused = Vec::new();
    for t in targets {
        match graph::model(root, t) {
            Ok(cell) => match plan_apply(root, std::slice::from_ref(&cell)) {
                Ok(_) => cells.push(cell),
                Err(e) => refused.push(e.to_string()),
            },
            Err(e) => refused.push(e.to_string()),
        }
    }
    if !refused.is_empty() {
        return Err(Error::new(format!(
            "{}\n{} of {} targets refused; nothing was written",
            refused.join("\n"),
            refused.len(),
            targets.len()
        )));
    }
    let baseline_plan = if proof::baseline_reusable(targets, baseline)? {
        let index_path = baseline.join("targets.json");
        proof::BaselinePlan {
            copies: vec![],
            index_text: read(&index_path)?,
            index_path,
        }
    } else {
        proof::save_baseline(root, targets, baseline)?
    };
    let apply = plan_apply(root, &cells)?;
    let targets_by_arch = ARCHES
        .into_iter()
        .map(|a| {
            let on: Vec<String> = cells
                .iter()
                .filter(|c| c.arches().contains(&a))
                .map(|c| c.target.clone())
                .collect();
            (a.to_owned(), on)
        })
        .collect();
    Ok(ConvertPlan {
        targets: targets.to_vec(),
        targets_by_arch,
        baseline_dir: baseline.to_owned(),
        baseline: baseline_plan,
        apply,
    })
}
/// The host owns writing and launching; callbacks allow atomic writer reuse and audit/testing.
pub trait ConvertHost {
    fn baseline(&mut self, plan: &proof::BaselinePlan) -> Result<()>;
    fn apply(&mut self, plan: &ApplyPlan) -> Result<()>;
    fn ninja(&mut self, root: &Path, arch: &str, targets: &[String]) -> Result<()>;
    fn check(
        &mut self,
        root: &Path,
        targets: &[String],
        baseline: &Path,
    ) -> Result<proof::CheckResult> {
        proof::check(root, targets, baseline)
    }
    fn refresh(&mut self, root: &Path) -> Result<()>;
    fn log(&mut self, line: &str);
}
pub fn run_convert<H: ConvertHost>(
    root: &Path,
    plan: &ConvertPlan,
    host: &mut H,
) -> Result<proof::CheckResult> {
    host.baseline(&plan.baseline)?;
    host.log(&format!("baseline: {}", plan.baseline_dir.display()));
    host.apply(&plan.apply)?;
    for path in &plan.apply.report {
        host.log(&format!(
            "wrote {}",
            path.strip_prefix(root).unwrap_or(path).display()
        ));
    }
    for arch in ARCHES {
        let on = plan
            .targets_by_arch
            .get(arch)
            .cloned()
            .unwrap_or_else(|| plan.targets.clone());
        if on.is_empty() {
            host.log(&format!("no cell of this wave is built on {arch}"));
            continue;
        }
        host.ninja(root, arch, &on)?;
        host.log(&format!("built {} cell(s) on {arch}", on.len()));
    }
    let result = host.check(root, &plan.targets, &plan.baseline_dir)?;
    host.log(&result.report());
    if result.bad == 0 {
        host.refresh(root)?;
    }
    let missing = missing_cell_recipes(root, &plan.targets)?;
    if !missing.is_empty() {
        return Err(Error::new(missing.iter().map(|(arch,lib)| format!(
            "no clibs/{arch}/{lib}.recipe for a cell this wave built on {arch}: cmake has not configured that library yet; run ./vibeos-cli build --arch {arch}, then tools/cargo_cell_gen.py --clib-recipes"
        )).collect::<Vec<_>>().join("\n")));
    }
    Ok(result)
}
/// Run the same configured-graph build entry point as Python; no shell interpolation.
pub fn ninja(root: &Path, arch: &str, targets: &[String]) -> Result<()> {
    let build = root.join(format!("build-{arch}"));
    local(root, &build)?;
    let script = local(root, &root.join("tools/canon-run.sh"))?;
    // `-k 0` on the cells: a wave's failures are independent, and stopping at the first
    // two (fusion, msxml3r) left the rest of an 85-module wave unbuilt and unknown.
    let mut cells = vec!["-k".to_owned(), "0".to_owned()];
    cells.extend(targets.iter().cloned());
    for args in [vec!["build.ninja".to_owned()], cells] {
        let output = Command::new("bash")
            .arg(&script)
            .arg("ninja")
            .args(&args)
            .current_dir(&build)
            .output()
            .map_err(|e| Error::new(format!("build-{arch}: ninja launch failed: {e}")))?;
        if !output.status.success() {
            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            let lines: Vec<_> = combined.lines().collect();
            let error = Regex::new(r"^\s*(error|ld\.lld: error)\b").unwrap();
            // Every FAILED edge, then the first errors: with -k 0 there can be many.
            let mut tail: Vec<_> = lines
                .iter()
                .copied()
                .filter(|s| s.starts_with("FAILED:"))
                .collect();
            tail.extend(lines.iter().copied().filter(|s| error.is_match(s)).take(20));
            tail.extend(lines.iter().copied().skip(lines.len().saturating_sub(8)));
            let mut seen = BTreeSet::new();
            tail.retain(|s| seen.insert(*s));
            return Err(Error::new(format!(
                "build-{arch}: `ninja {}` failed:\n{}",
                args.join(" "),
                tail.join("\n")
            )));
        }
    }
    Ok(())
}
/// Compute native C recipes; absent-on-arch is reported and other failures are fatal.
#[derive(Debug, Default)]
pub struct RecipePlan {
    pub edits: BTreeMap<PathBuf, String>,
    pub skipped: Vec<String>,
    /// Targets whose recipe could not be derived. The others are still written: one
    /// unmodelled library (ole32's per-source defines) used to block every recipe.
    pub failed: Vec<String>,
}
/// Committed recipes no cell build script names any more (a C library that became a Rust
/// crate, like scrnsave/dmilib/udmihelp/blrtl): `--clib-recipes` removes them and
/// `--clib-recipes-check` reports them, so the directory is exactly what the cells read.
pub fn orphan_recipes(root: &Path, plan: &RecipePlan) -> Vec<PathBuf> {
    let dir = root.join("sdk/lib/rust/reactos-link/clibs");
    let mut out = Vec::new();
    for a in ARCHES {
        if let Ok(rd) = std::fs::read_dir(dir.join(a)) {
            for e in rd.flatten() {
                let path = e.path();
                if path.extension().is_some_and(|x| x == "recipe") && !plan.edits.contains_key(&path) {
                    out.push(path);
                }
            }
        }
    }
    out.sort();
    out
}
/// Each committed recipe that is not exactly what cmake's compile derives today, one line
/// per file: differs, missing, or read by no cell. Empty means the recipes are current.
pub fn recipe_drift(root: &Path) -> Result<Vec<String>> {
    let p = clib_recipes(root)?;
    if !p.failed.is_empty() {
        return Err(Error::new(format!("{}\nrecipes not derivable", p.failed.join("\n"))));
    }
    let rel = |path: &Path| path.strip_prefix(root).unwrap_or(path).display().to_string();
    let mut drift = Vec::new();
    for (path, text) in &p.edits {
        match std::fs::read_to_string(path) {
            Ok(old) if &old == text => {}
            Ok(_) => drift.push(format!("{}: differs from cmake's compile", rel(path))),
            Err(_) => drift.push(format!("{}: missing", rel(path))),
        }
    }
    for path in orphan_recipes(root, &p) {
        drift.push(format!("{}: no cell links this library", rel(&path)));
    }
    drift.sort();
    Ok(drift)
}
pub fn clib_recipes(root: &Path) -> Result<RecipePlan> {
    // A stale configure is one fact, not one failure per library: say it once, up front.
    for a in ARCHES {
        let build = root.join(format!("build-{a}"));
        if build.join("build.ninja").is_file() {
            vibeos_build::ninja::require_configured(&local(root, &build)?, root)
                .map_err(|e| Error::new(e.to_string()))?;
        }
    }
    let mut plan = RecipePlan::default();
    let targets = recipes::engine_c_targets(root)?;
    for a in ARCHES {
        for t in &targets {
            match recipes::clib_recipe(a, t, root) {
                Ok(text) => {
                    plan.edits
                        .insert(recipes::clib_recipe_path(a, t, root), text);
                }
                Err(e) if e.message == format!("{t}: no sources on {a}") => {
                    plan.skipped.push(format!("{a} {t}: {e}"))
                }
                Err(e) => plan.failed.push(format!("{a} {t}: {e}")),
            }
        }
    }
    Ok(plan)
}
/// Native manifest plans. All architectures render before the caller writes anything.
pub fn manifest_edits(root: &Path) -> Result<BTreeMap<PathBuf, String>> {
    let arches: Vec<String> = ARCHES.iter().map(|a| a.to_string()).collect();
    let mut edits = vibeos_build::cell_manifest::write_plan(root, &arches)
        .map_err(|e| Error::new(e.to_string()))?;
    edits.extend(
        vibeos_build::media_manifest::write_plan(root, &arches)
            .map_err(|e| Error::new(e.to_string()))?,
    );
    Ok(edits)
}
/// Plan the RC templates for all configured architectures.
pub fn rc_templates(root: &Path) -> Result<RecipePlan> {
    let mut plan = RecipePlan::default();
    for a in ARCHES {
        plan.edits.insert(
            recipes::rc_template_path(a, root),
            recipes::derive_rc_template(a, root)?.text(a),
        );
    }
    Ok(plan)
}
/// Plan the C and assembler templates together, as the Python command does.
pub fn c_templates(root: &Path) -> Result<RecipePlan> {
    let mut plan = RecipePlan::default();
    for a in ARCHES {
        plan.edits.insert(
            recipes::c_template_path(a, root),
            recipes::derive_c_template(a, root)?.text(a),
        );
        plan.edits.insert(
            recipes::asm_template_path(a, root),
            recipes::derive_asm_template(a, root)?.text(a),
        );
        if let Some(x) = recipes::derive_cxx_template(a, root)? {
            plan.edits
                .insert(recipes::cxx_template_path(a, root), x.text_for(a, "C++"));
        }
    }
    Ok(plan)
}

/// The arches a rendered build.rs links on: the arms of its target match. One naming no
/// arm at all is not narrowed (every arch is owed a recipe), so an unreadable build
/// script can only make the check stricter.
pub fn built_arches(build_rs: &str) -> Vec<&'static str> {
    let named: Vec<_> = ARCHES
        .into_iter()
        .filter(|a| {
            let triple = super::constants()["TRIPLE"][*a].as_str().unwrap_or("?");
            build_rs.contains(&format!("        \"{triple}\" => (\""))
        })
        .collect();
    if named.is_empty() {
        ARCHES.to_vec()
    } else {
        named
    }
}
/// A newly built cell must have recipes for each of its own C libraries on every arch.
pub fn missing_cell_recipes(root: &Path, targets: &[String]) -> Result<Vec<(String, String)>> {
    let pattern = Regex::new(r#"\.c_from_compile_db\(\s*"(\w+)""#)?;
    let shared = super::constants()["SHARED_C_LIBS"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut out = Vec::new();
    for target in targets {
        let path = root
            .join("sdk/lib/rust")
            .join(proof::cell_crate(root, target)?)
            .join("build.rs");
        if !path.exists() {
            continue;
        }
        let text = read(&local(root, &path)?)?;
        let libs: BTreeSet<_> = pattern
            .captures_iter(&text)
            .map(|c| c[1].to_owned())
            .filter(|l| !shared.iter().any(|s| s.as_str() == Some(l)))
            .collect();
        for arch in built_arches(&text) {
            for lib in &libs {
                if !recipes::clib_recipe_path(arch, lib, root).exists() {
                    out.push((arch.to_owned(), lib.clone()));
                }
            }
        }
    }
    Ok(out)
}
