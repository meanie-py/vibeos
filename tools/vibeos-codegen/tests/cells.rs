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
