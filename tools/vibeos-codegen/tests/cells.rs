use vibeos_codegen::cells::{pe, render, Cell};
#[test]
fn pe_contract_matches_frozen_python_oracle() {
    let cases: serde_json::Value = serde_json::from_str(include_str!("cells/pe.json")).unwrap();
    let decode = |c: &serde_json::Value| {
        pe::Pe::parse(serde_json::from_value(c["data"].clone()).unwrap()).unwrap()
    };
    let baseline = decode(&cases[0]);
    for c in cases.as_array().unwrap() {
        let p = decode(c);
        assert_eq!(
            serde_json::to_value(p.contract().unwrap()).unwrap(),
            c["contract"],
            "{}",
            c["name"]
        );
        assert_eq!(
            serde_json::to_value(p.loaded().unwrap()).unwrap(),
            c["loaded"]
        );
        assert_eq!(p.mapped_size(), c["mapped"].as_u64().unwrap());
        assert_eq!(
            serde_json::to_value(pe::compare(&baseline, &p, false).unwrap()).unwrap(),
            c["diff"]
        );
        assert_eq!(
            serde_json::to_value(pe::compare(&baseline, &p, true).unwrap()).unwrap(),
            c["code_diff"]
        );
    }
}
#[test]
fn truncated_pe_never_becomes_a_verdict() {
    let cases: serde_json::Value = serde_json::from_str(include_str!("cells/pe.json")).unwrap();
    let data: Vec<u8> = serde_json::from_value(cases[0]["data"].clone()).unwrap();
    for n in [0, 1, 63, 88, 100, 320] {
        assert!(pe::Pe::parse(data[..n].to_vec()).is_err());
    }
    let mut p = pe::Pe::parse(data).unwrap();
    p.dirs.insert("resource".into(), (0xfffffff0, 64));
    assert!(p.contract().is_err());
}
#[test]
fn cell_renderers_match_frozen_python_oracle() {
    let cases: serde_json::Value = serde_json::from_str(include_str!("cells/render.json")).unwrap();
    for case in cases.as_array().unwrap() {
        let c: Cell = serde_json::from_value(case["cell"].clone()).unwrap();
        assert_eq!(
            render::build_rs(&c).unwrap(),
            case["build"].as_str().unwrap(),
            "{}",
            case["name"]
        );
        assert_eq!(render::main_rs(&c).unwrap(), case["main"].as_str().unwrap());
        assert_eq!(
            render::cargo_toml(&c).unwrap(),
            case["cargo"].as_str().unwrap()
        );
    }
}
#[test]
fn integrated_models_match_python_with_real_pe_reader() {
    let fixtures: serde_json::Value =
        serde_json::from_str(include_str!("cells/models.json")).unwrap();
    let pe: serde_json::Value = serde_json::from_str(include_str!("cells/pe.json")).unwrap();
    let bytes: Vec<u8> = serde_json::from_value(pe[0]["data"].clone()).unwrap();
    for f in fixtures.as_array().unwrap() {
        let root = std::env::temp_dir().join(format!(
            "cell-model-{}-{}",
            std::process::id(),
            f["name"].as_str().unwrap()
        ));
        std::fs::create_dir_all(&root).unwrap();
        for (path, text) in f["files"].as_object().unwrap() {
            let p = root.join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, text.as_str().unwrap()).unwrap();
        }
        for arch in ["i386", "amd64", "arm64"] {
            let suffix = if f["name"] == "model" { "exe" } else { "dll" };
            let file = root.join(format!("build-{arch}/app/foo.{suffix}"));
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, &bytes).unwrap();
        }
        let mut model = vibeos_codegen::cells::graph::model(&root, "foo").unwrap();
        model.cmake_dir = model.cmake_dir.strip_prefix(&root).unwrap().to_owned();
        assert_eq!(
            serde_json::to_value(model).unwrap(),
            f["expected"],
            "{}",
            f["name"]
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
#[test]
fn an_arch_gate_in_the_modules_own_cmakelists_names_its_arches() {
    use vibeos_codegen::cells::graph::arch_gate;
    let gated = "if(ARCH STREQUAL \"arm64\" OR ARCH STREQUAL \"amd64\")\nspec2def(foo.dll foo.spec)\nadd_library(foo MODULE foo.c)\nendif()\n";
    assert_eq!(arch_gate(gated, "foo").unwrap(), ["amd64", "arm64"]);
    // Nested gates intersect.
    let nested = "if(ARCH STREQUAL \"arm64\" OR ARCH STREQUAL \"amd64\")\nif(ARCH STREQUAL amd64)\nadd_library(foo MODULE foo.c)\nendif()\nendif()\n";
    assert_eq!(arch_gate(nested, "foo").unwrap(), ["amd64"]);
    // Anything but a pure arch test refuses: a build option would drop the module silently.
    for refused in [
        "add_library(foo MODULE foo.c)\n",
        "if(ENABLE_ROSTESTS)\nadd_library(foo MODULE foo.c)\nendif()\n",
        "if(ARCH STREQUAL \"amd64\" AND ENABLE_X)\nadd_library(foo MODULE foo.c)\nendif()\n",
        "if(ARCH STREQUAL \"amd64\")\nelse()\nadd_library(foo MODULE foo.c)\nendif()\n",
        "if(ARCH STREQUAL \"ppc\")\nadd_library(foo MODULE foo.c)\nendif()\n",
    ] {
        assert!(arch_gate(refused, "foo").is_err(), "{refused}");
    }
    // An earlier, closed gate does not cover a later target.
    let closed = "if(ARCH STREQUAL amd64)\nset(X 1)\nendif()\nadd_library(foo MODULE foo.c)\n";
    assert!(arch_gate(closed, "foo").is_err());
}
#[test]
fn a_body_depends_only_on_a_custom_commands_output() {
    use vibeos_codegen::cells::graph::body_depends;
    let text = "set(_check ${CMAKE_CURRENT_BINARY_DIR}/x.checked)\nadd_custom_command(\n    OUTPUT ${_check}\n    COMMAND tool --check\n    DEPENDS a.spec)\n";
    assert_eq!(
        body_depends(text, &["${_check}".into()]).unwrap(),
        ["${_check}"]
    );
    // Not a command's output, not a variable, or set twice: still unmodelled.
    let no_cmd = "set(_check ${CMAKE_CURRENT_BINARY_DIR}/x.checked)\n";
    assert!(body_depends(no_cmd, &["${_check}".into()]).is_none());
    assert!(body_depends(text, &["literal".into()]).is_none());
    let twice = format!("{text}set(_check other)\n");
    assert!(body_depends(&twice, &["${_check}".into()]).is_none());
}
#[test]
fn a_gated_cell_renders_only_its_own_arches() {
    let cases: serde_json::Value = serde_json::from_str(include_str!("cells/render.json")).unwrap();
    for case in cases.as_array().unwrap() {
        let mut v = case["cell"].clone();
        for (_, field) in v.as_object_mut().unwrap().iter_mut() {
            if let Some(map) = field.as_object_mut() {
                map.remove("i386");
            }
        }
        v["arches"] = serde_json::json!(["amd64", "arm64"]);
        let c: Cell = serde_json::from_value(v).unwrap();
        assert_eq!(c.arches(), ["amd64", "arm64"]);
        let build = render::build_rs(&c).unwrap();
        assert!(!build.contains("\"x86\" =>"), "{}: {build}", case["name"]);
        assert!(build.contains("\"x86_64\" => (\""), "{}", case["name"]);
        let main = render::main_rs(&c).unwrap();
        assert!(!main.contains("\"x86\""), "{}: {main}", case["name"]);
        let toml = render::cargo_toml(&c).unwrap();
        assert!(!toml.contains("\"x86\""), "{}: {toml}", case["name"]);
        // An ungated cell keeps its i386 arm, and does not serialize the new field.
        let full: Cell = serde_json::from_value(case["cell"].clone()).unwrap();
        assert!(serde_json::to_value(&full).unwrap().get("arches").is_none());
    }
}
