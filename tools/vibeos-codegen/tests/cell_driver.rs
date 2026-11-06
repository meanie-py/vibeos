use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use vibeos_codegen::cells::{
    driver::{self, ApplyPlan, ConvertHost, ConvertPlan},
    proof, Cell, Result,
};
#[test]
fn frozen_apply_and_refusal() {
    let f: serde_json::Value = serde_json::from_str(include_str!("cells/apply.json")).unwrap();
    let root = std::env::temp_dir().join(format!("driver-fixture-{}", std::process::id()));
    std::fs::create_dir_all(root.join("base/m")).unwrap();
    std::fs::create_dir_all(root.join("sdk/lib/rust/m-rust")).unwrap();
    std::fs::write(
        root.join("base/m/CMakeLists.txt"),
        f["cmake"].as_str().unwrap(),
    )
    .unwrap();
    std::fs::write(
        root.join("sdk/lib/rust/Cargo.toml"),
        f["workspace"].as_str().unwrap(),
    )
    .unwrap();
    let c: Cell = serde_json::from_value(f["cell"].clone()).unwrap();
    let plan = driver::plan_apply(&root, &[c.clone()]).unwrap();
    for (p, text) in f["expected"].as_object().unwrap() {
        assert_eq!(plan.edits[&root.join(p)], text.as_str().unwrap(), "{p}");
    }
    // A late rewrite refusal must not write an earlier valid cell or its CMakeLists.
    std::fs::create_dir_all(root.join("base/n")).unwrap();
    std::fs::write(root.join("base/n/CMakeLists.txt"),
        "add_library(n MODULE n.c)\nif(MSVC)\nelse()\n target_compile_options(n PRIVATE /wd4267)\nendif()\n").unwrap();
    let mut refused = c.clone();
    refused.target = "n".into();
    refused.cmake_dir = root.join("base/n");
    refused.body_crate.clear();
    let error = driver::plan_apply(&root, &[c.clone(), refused]).unwrap_err();
    assert!(error.message.contains("elseif/else branch"));
    assert_eq!(
        std::fs::read_to_string(root.join("base/m/CMakeLists.txt")).unwrap(),
        f["cmake"].as_str().unwrap()
    );
    assert!(!root.join("sdk/lib/rust/m-cargo").exists());
    assert!(driver::plan_apply(&root, &[c.clone(), c.clone()]).is_err());
    assert!(!root.join("sdk/lib/rust/m-cargo").exists());
    std::fs::create_dir_all(root.join("sdk/lib/rust/m-cargo")).unwrap();
    assert!(driver::plan_apply(&root, &[c]).is_err());
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn a_dropped_calls_same_line_comment_goes_with_it() {
    // 76 cells kept ` # CORE-5743: No ARRAY_SIZE macro` as an indented orphan line after
    // `set_wine_module_FIXME(t) # CORE-5743: ...` was transferred away.
    let f: serde_json::Value = serde_json::from_str(include_str!("cells/apply.json")).unwrap();
    let root = std::env::temp_dir().join(format!("driver-comment-{}", std::process::id()));
    std::fs::create_dir_all(root.join("base/m")).unwrap();
    std::fs::create_dir_all(root.join("sdk/lib/rust/m-rust")).unwrap();
    let cmake = f["cmake"].as_str().unwrap().replace(
        "set_module_type(m win32gui)\n",
        "set_module_type(m win32gui)\nset_wine_module_FIXME(m) # CORE-5743: No ARRAY_SIZE macro\n",
    );
    std::fs::write(root.join("base/m/CMakeLists.txt"), cmake).unwrap();
    std::fs::write(
        root.join("sdk/lib/rust/Cargo.toml"),
        f["workspace"].as_str().unwrap(),
    )
    .unwrap();
    let c: Cell = serde_json::from_value(f["cell"].clone()).unwrap();
    let plan = driver::plan_apply(&root, &[c]).unwrap();
    let out = &plan.edits[&root.join("base/m/CMakeLists.txt")];
    assert!(!out.contains("CORE-5743"), "{out}");
    assert_eq!(
        out,
        f["expected"]["base/m/CMakeLists.txt"].as_str().unwrap()
    );
    std::fs::remove_dir_all(root).unwrap();
}
#[derive(Default)]
struct Host {
    events: Vec<String>,
    fail: Option<String>,
    bad: usize,
}
impl ConvertHost for Host {
    fn baseline(&mut self, _: &proof::BaselinePlan) -> Result<()> {
        self.events.push("baseline".into());
        Ok(())
    }
    fn apply(&mut self, _: &ApplyPlan) -> Result<()> {
        self.events.push("apply".into());
        Ok(())
    }
    fn ninja(&mut self, _: &Path, a: &str, _: &[String]) -> Result<()> {
        self.events.push(a.into());
        if self.fail.as_deref() == Some(a) {
            return Err("ninja failure".into());
        }
        Ok(())
    }
    fn check(&mut self, _: &Path, _: &[String], _: &Path) -> Result<proof::CheckResult> {
        self.events.push("check".into());
        Ok(proof::CheckResult {
            records: vec![],
            bad: self.bad,
        })
    }
    fn refresh(&mut self, _: &Path) -> Result<()> {
        self.events.push("refresh".into());
        Ok(())
    }
    fn log(&mut self, _: &str) {}
}
fn plan() -> ConvertPlan {
    ConvertPlan {
        targets: vec!["x".into()],
        targets_by_arch: Default::default(),
        baseline_dir: PathBuf::from("base"),
        baseline: proof::BaselinePlan {
            copies: vec![],
            index_path: "base/targets.json".into(),
            index_text: "{}".into(),
        },
        apply: ApplyPlan {
            edits: BTreeMap::new(),
            created_crates: vec![],
            report: vec![],
        },
    }
}
#[test]
fn sequence_and_failure() {
    let mut host = Host {
        bad: 1,
        ..Host::default()
    };
    assert_eq!(
        driver::run_convert(Path::new("."), &plan(), &mut host)
            .unwrap()
            .bad,
        1
    );
    assert_eq!(
        host.events,
        ["baseline", "apply", "i386", "amd64", "arm64", "check"]
    );
    let mut host = Host {
        fail: Some("amd64".into()),
        ..Host::default()
    };
    assert!(driver::run_convert(Path::new("."), &plan(), &mut host).is_err());
    assert_eq!(host.events, ["baseline", "apply", "i386", "amd64"]);
}

#[test]
fn conversion_fails_when_refresh_leaves_a_cells_own_c_without_recipes() {
    let root = std::env::temp_dir().join(format!("driver-missing-recipes-{}", std::process::id()));
    let path = root.join("sdk/lib/rust/x-cargo/build.rs");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        &path,
        ".c_from_compile_db(\"x_shims\", &[])\n.c_from_compile_db(\"libcntpr\", &[])",
    )
    .unwrap();
    let mut host = Host::default();
    let error = driver::run_convert(&root, &plan(), &mut host).unwrap_err();
    assert!(error.message.contains("clibs/i386/x_shims.recipe"));
    assert!(!error.message.contains("libcntpr.recipe"));
    assert_eq!(
        host.events,
        ["baseline", "apply", "i386", "amd64", "arm64", "check", "refresh"]
    );
    for arch in ["i386", "amd64", "arm64"] {
        let recipe = vibeos_codegen::cells::recipes::clib_recipe_path(arch, "x_shims", &root);
        std::fs::create_dir_all(recipe.parent().unwrap()).unwrap();
        std::fs::write(recipe, "# recipe\n").unwrap();
    }
    assert_eq!(
        driver::run_convert(&root, &plan(), &mut Host::default())
            .unwrap()
            .bad,
        0
    );
    std::fs::remove_dir_all(root).unwrap();
}
