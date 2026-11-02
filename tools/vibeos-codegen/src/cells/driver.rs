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
    pub baseline_dir: PathBuf,
    pub baseline: proof::BaselinePlan,
    pub apply: ApplyPlan,
}
pub fn prepare_convert(root: &Path, targets: &[String], baseline: &Path) -> Result<ConvertPlan> {
    if targets.is_empty() {
        return Err(Error::new("--convert needs at least one target"));
    }
    let cells: Vec<Cell> = targets
        .iter()
        .map(|t| graph::model(root, t))
        .collect::<Result<_>>()?;
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
    Ok(ConvertPlan {
        targets: targets.to_vec(),
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
        host.ninja(root, arch, &plan.targets)?;
        host.log(&format!("built {} cell(s) on {arch}", plan.targets.len()));
    }
    let result = host.check(root, &plan.targets, &plan.baseline_dir)?;
    host.log(&result.report());
    if result.bad == 0 {
        host.refresh(root)?;
    }
    Ok(result)
}
/// Run the same configured-graph build entry point as Python; no shell interpolation.
pub fn ninja(root: &Path, arch: &str, targets: &[String]) -> Result<()> {
    let build = root.join(format!("build-{arch}"));
    local(root, &build)?;
    let script = local(root, &root.join("tools/canon-run.sh"))?;
    for args in [vec!["build.ninja".to_owned()], targets.to_vec()] {
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
            let mut tail: Vec<_> = lines
                .iter()
                .copied()
                .filter(|s| s.starts_with("FAILED:") || error.is_match(s))
                .take(20)
                .collect();
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
}
pub fn clib_recipes(root: &Path) -> Result<RecipePlan> {
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
                Err(e) => return Err(e),
            }
        }
    }
    Ok(plan)
}
/// Run the permitted standalone manifest reporters; generated code remains native.
pub fn refresh_reporters(root: &Path, python: &Path) -> Result<()> {
    for tool in ["cell_manifest.py", "media_manifest.py"] {
        let path = local(root, &root.join("tools").join(tool))?;
        let output = Command::new(python)
            .arg(&path)
            .arg("--write")
            .current_dir(root)
            .output()
            .map_err(|e| Error::new(format!("{tool}: {e}")))?;
        if !output.status.success() {
            return Err(Error::new(format!(
                "{tool} failed ({}): {}{}",
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )));
        }
    }
    Ok(())
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
    }
    Ok(plan)
}
