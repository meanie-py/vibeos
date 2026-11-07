use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
};
use vibeos_codegen::cells::{
    self, driver, graph, mutate, proof, render, rewrite, Cell, Error, Result,
};
pub const NAMES: &[&str] = &["cargo-cell"];

fn output_path(root: &Path, path: &Path) -> Result<()> {
    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        root.join(path)
    };
    if !path.starts_with(root)
        || path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(Error::new(format!(
            "output escapes selected checkout: {}",
            path.display()
        )));
    }
    let mut parent = path.as_path();
    loop {
        if parent.is_symlink() {
            return Err(Error::new(format!(
                "output traverses symlink: {}",
                parent.display()
            )));
        }
        if parent.exists() {
            cells::local(root, parent)?;
        }
        if parent == root {
            break;
        }
        parent = parent.parent().ok_or("output has no checkout ancestor")?;
    }
    Ok(())
}
fn write_edits(root: &Path, edits: &BTreeMap<PathBuf, String>) -> Result<()> {
    for path in edits.keys() {
        output_path(root, path)?;
    }
    for (i, (path, text)) in edits.iter().enumerate() {
        fs::create_dir_all(path.parent().ok_or("output has no parent")?)?;
        super::write_atomic(path, text.as_bytes(), i).map_err(Error::new)?;
    }
    Ok(())
}
fn baseline(plan: &proof::BaselinePlan) -> Result<()> {
    // Read every source before the first mutation; the index is written last.
    let copies = plan
        .copies
        .iter()
        .map(|c| Ok((c.destination.clone(), fs::read(&c.source)?)))
        .collect::<Result<Vec<_>>>()?;
    for (i, (path, bytes)) in copies.iter().enumerate() {
        fs::create_dir_all(path.parent().ok_or("baseline has no parent")?)?;
        super::write_atomic(path, bytes, i).map_err(Error::new)?;
    }
    fs::create_dir_all(
        plan.index_path
            .parent()
            .ok_or("baseline index has no parent")?,
    )?;
    super::write_atomic(&plan.index_path, plan.index_text.as_bytes(), copies.len())
        .map_err(Error::new)
}
fn refresh(root: &Path) -> Result<()> {
    let recipes = driver::clib_recipes(root)?;
    let mut edits = driver::manifest_edits(root)?;
    edits.extend(recipes.edits);
    write_edits(root, &edits)?;
    for line in recipes.skipped {
        eprintln!("{line}");
    }
    if !recipes.failed.is_empty() {
        return Err(Error::new(format!(
            "{}\n{} recipe(s) not derived",
            recipes.failed.join("\n"),
            recipes.failed.len()
        )));
    }
    Ok(())
}
struct Host {
    root: PathBuf,
    api: bool,
}
impl driver::ConvertHost for Host {
    fn baseline(&mut self, p: &proof::BaselinePlan) -> Result<()> {
        baseline(p)
    }
    fn apply(&mut self, p: &driver::ApplyPlan) -> Result<()> {
        write_edits(&self.root, &p.edits)
    }
    fn ninja(&mut self, root: &Path, arch: &str, targets: &[String]) -> Result<()> {
        driver::ninja(root, arch, targets)
    }
    fn refresh(&mut self, root: &Path) -> Result<()> {
        refresh(root)
    }
    fn log(&mut self, line: &str) {
        if self.api {
            eprintln!("{line}")
        } else {
            println!("{line}")
        }
    }
}
fn default_baseline() -> Result<PathBuf> {
    let home =
        std::env::var_os("HOME").ok_or("HOME is required for the default baseline directory")?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| Error::new(e.to_string()))?
        .as_secs();
    Ok(PathBuf::from(home)
        .join("vibeos-cargo-baselines")
        .join(format!("convert-{stamp}")))
}
fn convert(root: &Path, targets: &[String], dir: Option<&Path>, api: bool) -> Result<u8> {
    let dir = dir
        .map(PathBuf::from)
        .map(Ok)
        .unwrap_or_else(default_baseline)?;
    let plan = driver::prepare_convert(root, targets, &dir)?;
    let result = driver::run_convert(
        root,
        &plan,
        &mut Host {
            root: root.to_owned(),
            api,
        },
    )?;
    Ok(u8::from(result.bad != 0))
}
fn mutation(root: &Path, op: &str, targets: Option<&[String]>) -> Result<Vec<String>> {
    let plan = match op {
        "remap" => mutate::remap(root, targets)?,
        "own_c" => mutate::own_c(root, targets)?,
        "own_rc" => mutate::own_rc(root, targets)?,
        _ => return Err("unknown mutation".into()),
    };
    write_edits(root, &plan.edits)?;
    Ok(plan.changed)
}
// Writer operations retain Python positional signatures; pure operations live in cells::api.
fn dispatch(root: &Path, r: &Value) -> Result<Value> {
    let op = r["op"].as_str().ok_or("API request requires op")?;
    let args = r["args"].as_array().cloned().unwrap_or_default();
    let kwargs = r["kwargs"].as_object();
    let get = |i: usize, name: &str| -> Option<Value> {
        args.get(i)
            .cloned()
            .or_else(|| kwargs.and_then(|k| k.get(name).cloned()))
    };
    let required =
        |i, name| get(i, name).ok_or_else(|| Error::new(format!("{op}: missing {name}")));
    let targets =
        || -> Result<Vec<String>> { Ok(serde_json::from_value(required(0, "targets")?)?) };
    let dir = || -> Result<PathBuf> { Ok(serde_json::from_value(required(1, "baseline")?)?) };
    let value = match op {
        "apply" => {
            let c: Cell = serde_json::from_value(required(0, "cell")?)?;
            let plan = driver::plan_apply(root, &[c])?;
            write_edits(root, &plan.edits)?;
            serde_json::to_value(plan.report)?
        }
        "survey" => serde_json::to_value(driver::survey(root)?)?,
        "save_baseline" => {
            baseline(&proof::save_baseline(root, &targets()?, &dir()?)?)?;
            Value::Null
        }
        "check" => {
            let c = proof::check(root, &targets()?, &dir()?)?;
            json!({"code":u8::from(c.bad!=0),"report":c.report()})
        }
        "reshell" => {
            let p = proof::reshell(root, &targets()?, &dir()?)?;
            write_edits(root, &p.edits)?;
            json!(p.notes)
        }
        "remap" | "own_rc" | "own_c" => {
            let ts: Option<Vec<String>> =
                serde_json::from_value(get(0, "crates").unwrap_or(Value::Null))?;
            json!(mutation(root, op, ts.as_deref())?)
        }
        "_drop_res_in" | "_drop_shims_in" => {
            let cmake: PathBuf = serde_json::from_value(required(0, "cmake")?)?;
            let target: String = serde_json::from_value(required(1, "target")?)?;
            let text = cells::read(&cells::local(root, &cmake)?)?;
            let text = if op == "_drop_res_in" {
                mutate::drop_res_in(&text, &cmake, &target)?
            } else {
                let generated: Vec<String> = serde_json::from_value(required(2, "generated")?)?;
                mutate::drop_shims_in(&text, &cmake, &target, &generated)?
            };
            write_edits(root, &BTreeMap::from([(cmake, text)]))?;
            Value::Null
        }
        "_drop_res_library" => {
            let target: String = serde_json::from_value(required(0, "target")?)?;
            let mut p = mutate::Plan::default();
            mutate::drop_res_library(&mut p, root, &target)?;
            write_edits(root, &p.edits)?;
            Value::Null
        }
        "_ninja" => {
            let arch: String = serde_json::from_value(required(0, "arch")?)?;
            let ts: Vec<String> = serde_json::from_value(required(1, "targets")?)?;
            driver::ninja(root, &arch, &ts)?;
            Value::Null
        }
        "refresh_manifests" => {
            refresh(root)?;
            Value::Null
        }
        "convert" => {
            let p: Option<PathBuf> =
                serde_json::from_value(get(1, "baseline").unwrap_or(Value::Null))?;
            json!(convert(root, &targets()?, p.as_deref(), true)?)
        }
        _ => return cells::api::dispatch(root, r),
    };
    Ok(value)
}
pub fn run(raw: Vec<String>) -> std::result::Result<u8, String> {
    run_inner(raw).map_err(|e| e.to_string())
}
fn run_inner(raw: Vec<String>) -> Result<u8> {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (mut api, mut mode, mut dir, mut baseline_dir) = (false, String::new(), None, None);
    let mut targets = Vec::new();
    let mut args = raw.into_iter();
    while let Some(a) = args.next() {
        match a.as_str() {
            "cargo-cell" => {}
            "--root" => root = args.next().ok_or("--root requires DIR")?.into(),
            "--api-json" => api = true,
            "--baseline" => {
                baseline_dir = Some(PathBuf::from(args.next().ok_or("--baseline requires DIR")?))
            }
            "--save-baseline" | "--check" | "--reshell" => {
                if !mode.is_empty() {
                    return Err("choose one Cargo cell operation".into());
                }
                mode = a;
                dir = Some(PathBuf::from(args.next().ok_or("operation requires DIR")?));
            }
            "--interface" | "--interface-apply" | "--apply" | "--survey" | "--convert" | "--own-rc" | "--own-c" | "--clib-recipes"
            | "--clib-recipes-check" | "--c-template" | "--rc-template" | "--remap" => {
                if !mode.is_empty() {
                    return Err("choose one Cargo cell operation".into());
                }
                mode = a;
            }
            s if !s.starts_with('-') => targets.push(a),
            _ => return Err(format!("unexpected Cargo cell argument {a}").into()),
        }
    }
    let root = root.canonicalize()?;
    // Existing reporting helpers and canonical build wrapper expect the checkout cwd.
    std::env::set_current_dir(&root)?;
    if api {
        if !mode.is_empty() || !targets.is_empty() || baseline_dir.is_some() {
            return Err("--api-json cannot be combined with operations".into());
        }
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input)?;
        let result = serde_json::from_str(&input)
            .map_err(Error::from)
            .and_then(|r| dispatch(&root, &r));
        println!(
            "{}",
            match result {
                Ok(v) => json!({"ok":v}),
                Err(e) => json!({"error":e}),
            }
        );
        return Ok(0);
    }
    if baseline_dir.is_some() && mode != "--convert" {
        return Err("--baseline requires --convert".into());
    }
    match mode.as_str() {
        "--convert" => return convert(&root, &targets, baseline_dir.as_deref(), false),
        "--save-baseline" => {
            if targets.is_empty() {
                return Err("--save-baseline requires targets".into());
            }
            baseline(&proof::save_baseline(
                &root,
                &targets,
                dir.as_deref().unwrap(),
            )?)?;
            println!("saved {} modules x 3 arches", targets.len());
        }
        "--check" => {
            if targets.is_empty() {
                return Err("--check requires targets".into());
            }
            let c = proof::check(&root, &targets, dir.as_deref().unwrap())?;
            print!("{}", c.report());
            return Ok(u8::from(c.bad != 0));
        }
        "--reshell" => {
            let p = proof::reshell(&root, &targets, dir.as_deref().unwrap())?;
            write_edits(&root, &p.edits)?;
            for n in p.notes {
                println!("{n}")
            }
        }
        "--survey" => {
            if !targets.is_empty() {
                return Err("--survey does not accept targets".into());
            }
            print!("{}", driver::survey_report(&driver::survey(&root)?))
        }
        "--own-c" | "--own-rc" | "--remap" => {
            for n in mutation(
                &root,
                &mode[2..].replace('-', "_"),
                (!targets.is_empty()).then_some(targets.as_slice()),
            )? {
                println!("{n}")
            }
        }
        "--clib-recipes-check" => {
            // Recipes are DERIVED from cmake's compile commands, and nothing re-derived them
            // when a CMake flag moved (pseh's include went from -isystem to -I, 2026-11-06;
            // 380 recipes drifted). A drifted recipe only bites a cmake-off build, which is
            // why it went unnoticed. `check` runs the same test as vibeos-lint's
            // clib_recipes_fresh.
            if !targets.is_empty() {
                return Err("--clib-recipes-check does not accept targets".into());
            }
            let drift = driver::recipe_drift(&root)?;
            for d in &drift {
                println!("{d}");
            }
            if !drift.is_empty() {
                return Err(format!(
                    "{} recipe(s) drifted; regenerate: tools/codegen cargo-cell --clib-recipes",
                    drift.len()
                )
                .into());
            }
            eprintln!("recipes match cmake's compiles");
        }
        "--clib-recipes" | "--rc-template" | "--c-template" => {
            if !targets.is_empty() {
                return Err("template/recipe operations do not accept targets".into());
            }
            let p = match mode.as_str() {
                "--clib-recipes" => driver::clib_recipes(&root)?,
                "--rc-template" => driver::rc_templates(&root)?,
                _ => driver::c_templates(&root)?,
            };
            write_edits(&root, &p.edits)?;
            for k in p.edits.keys() {
                println!("wrote {}", k.strip_prefix(&root).unwrap_or(k).display())
            }
            if mode == "--clib-recipes" && p.failed.is_empty() {
                for path in driver::orphan_recipes(&root, &p) {
                    std::fs::remove_file(&path)?;
                    println!("removed {}", path.strip_prefix(&root).unwrap_or(&path).display())
                }
            }
            for n in p.skipped {
                eprintln!("{n}")
            }
            if !p.failed.is_empty() {
                return Err(format!(
                    "{}\n{} recipe(s) not derived; the rest were written",
                    p.failed.join("\n"),
                    p.failed.len()
                )
                .into());
            }
        }
        "--interface" | "--interface-apply" => {
            if targets.is_empty() {
                return Err(format!("{mode} requires targets").into());
            }
            // Plan the whole wave before writing anything; plans of one file compose.
            let mut edits: BTreeMap<PathBuf, String> = BTreeMap::new();
            let mut removed = Vec::new();
            let mut bad = false;
            for t in &targets {
                match cells::interface::plan_on(&root, t, &edits) {
                    Ok(p) => {
                        if mode == "--interface" {
                            println!("== {} ({t})\n{}", p.cmake.strip_prefix(&root).unwrap_or(&p.cmake).display(), p.text);
                        }
                        edits.insert(p.cmake, p.text);
                        edits.extend(p.cell_edits);
                        removed.extend(p.removed);
                    }
                    Err(e) => {
                        println!("REFUSED {e}");
                        bad = true;
                    }
                }
            }
            if mode == "--interface-apply" {
                if bad {
                    return Err("refusals above: nothing was written".into());
                }
                write_edits(&root, &edits)?;
                for path in edits.keys() {
                    println!("wrote {}", path.strip_prefix(&root).unwrap_or(path).display());
                }
                for path in removed {
                    output_path(&root, &path)?;
                    fs::remove_file(&path)?;
                    println!("removed {}", path.strip_prefix(&root).unwrap_or(&path).display());
                }
            }
            return Ok(u8::from(bad));
        }
        "--apply" => {
            if targets.is_empty() {
                return Err("--apply requires targets".into());
            }
            let cs = targets
                .iter()
                .map(|t| graph::model(&root, t))
                .collect::<Result<Vec<_>>>()?;
            let p = driver::plan_apply(&root, &cs)?;
            write_edits(&root, &p.edits)?;
            for path in p.report {
                println!(
                    "wrote {}",
                    path.strip_prefix(&root).unwrap_or(&path).display()
                )
            }
        }
        "" => {
            if targets.is_empty() {
                return Err("choose targets or a Cargo cell operation".into());
            }
            let mut bad = false;
            for t in targets {
                match graph::model(&root, &t) {
                    Ok(c) => {
                        println!("== {}/build.rs\n{}", c.crate_name(), render::build_rs(&c)?);
                        println!(
                            "== {}/CMakeLists.txt\n{}",
                            c.cmake_dir
                                .strip_prefix(&root)
                                .unwrap_or(&c.cmake_dir)
                                .display(),
                            rewrite::rewrite_cmake(&c.cmake_dir.join("CMakeLists.txt"), &c)?
                        )
                    }
                    Err(e) => {
                        println!("REFUSED {e}");
                        bad = true
                    }
                }
            }
            return Ok(u8::from(bad));
        }
        _ => unreachable!(),
    }
    Ok(0)
}
