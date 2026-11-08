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
    // A closed if/elseif chain before the target does not refuse it (pl011's RUST_TARGET).
    let chain = "if(ARCH STREQUAL i386)\nset(T a)\nelseif(ARCH STREQUAL arm64)\nset(T b)\nendif()\nif(ARCH STREQUAL arm64)\nadd_library(foo MODULE foo.c)\nendif()\n";
    assert_eq!(arch_gate(chain, "foo").unwrap(), ["arm64"]);
}
#[test]
fn a_parents_add_subdirectory_gate_names_a_modules_arches() {
    use vibeos_codegen::cells::graph::subdirectory_gate;
    // legide: the option keeps gating the whole subdirectory, the cell included.
    let ide = "add_subdirectory(atapi)\nif(ARCH STREQUAL \"i386\" AND NOT ENABLE_UNIATA)\n    add_subdirectory(legide)\nendif()\n";
    assert_eq!(subdirectory_gate(ide, "legide").unwrap(), ["i386"]);
    assert!(subdirectory_gate(ide, "atapi").is_err());
    let serial = "add_subdirectory(serial)\nif(ARCH STREQUAL \"arm64\")\n    add_subdirectory(pl011)\nendif()\n";
    assert_eq!(subdirectory_gate(serial, "pl011").unwrap(), ["arm64"]);
    assert!(subdirectory_gate("if(ENABLE_X)\nadd_subdirectory(m)\nendif()\n", "m").is_err());
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
fn interface_fixture(name: &str, cmake: &str, objs: &str, stub: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("cell-iface-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let w = |p: &str, t: &str| {
        let p = root.join(p);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, t).unwrap();
    };
    for arch in ["i386", "amd64", "arm64"] {
        w(
            &format!("build-{arch}/build.ninja"),
            &format!("build lib/foo/libfoo.a: C_STATIC_LIBRARY_LINKER__foo_Debug {objs} || x\n"),
        );
    }
    w("lib/foo/CMakeLists.txt", cmake);
    w("lib/foo/rust_cohab_stub.c", stub);
    w(
        "sdk/lib/rust/bar-cargo/build.rs",
        "        \"x86_64\" => &[\"body\", \"engine:c:foo\", \"engine:foo_rust\"],\n",
    );
    w(
        "sdk/lib/rust/reactos-link/clibs/amd64/foo.recipe",
        "# foo (amd64)\nc\tlib/foo/rust_cohab_stub.c\t@std\t-O2\n",
    );
    root
}
#[test]
fn a_placeholder_static_library_becomes_an_interface_library() {
    use vibeos_codegen::cells::interface::plan;
    let cmake = "# Every translation unit is Rust now (foo-rust); the C stays on\n\
# disk. The target keeps a stub TU and links the Rust archive.\n\
rust_staticlib_variant(foo_rust foo-rust \"\" TRUE)\n\
list(APPEND SOURCE\n    # foo.c  # replaced by Rust (x)\n    rust_cohab_stub.c\n    precomp.h)\n\
add_library(foo STATIC ${SOURCE})\n\
add_dependencies(foo rust_foo_rust)\n\
target_link_libraries(foo PRIVATE ${foo_rust_LIB} pseh) # a note)\n\
add_importlibs(foo ntoskrnl)\n\
add_pch(foo precomp.h SOURCE)\n\
target_include_directories(foo PUBLIC inc PRIVATE priv)\n";
    let objs = "lib/foo/CMakeFiles/foo.dir/rust_cohab_stub.c.obj";
    let stub = "/* foo is Rust */\ntypedef int foo_placeholder;\n";
    let root = interface_fixture("ok", cmake, objs, stub);
    let p = plan(&root, "foo").unwrap();
    let t = &p.text;
    assert!(t.contains("add_library(foo INTERFACE)\n"), "{t}");
    assert!(t.contains("# foo.c  # replaced by Rust (x)\n"), "{t}");
    assert!(!t.contains("list(APPEND SOURCE"), "{t}");
    assert!(t.contains(
        "target_link_libraries(foo INTERFACE $<LINK_ONLY:${foo_rust_LIB}> $<LINK_ONLY:pseh>) # a note)\n"
    ), "{t}");
    assert!(
        t.contains("target_link_libraries(foo INTERFACE libntoskrnl)\n"),
        "{t}"
    );
    assert!(
        t.contains("target_include_directories(foo INTERFACE inc)\n"),
        "{t}"
    );
    assert!(
        !t.contains("add_pch") && !t.contains("keeps a stub TU"),
        "{t}"
    );
    assert!(t.contains("INTERFACE library over the Rust archive"), "{t}");
    assert!(
        t.lines()
            .filter(|l| l.starts_with("#"))
            .all(|l| l.len() <= 92),
        "{t}"
    );
    // The stub and its stub-only recipe go; the cell drops the stub's token, keeps the Rust.
    assert!(p
        .removed
        .iter()
        .any(|r| r.ends_with("lib/foo/rust_cohab_stub.c")));
    assert!(p
        .removed
        .iter()
        .any(|r| r.ends_with("clibs/amd64/foo.recipe")));
    let cell = p.cell_edits.values().next().unwrap();
    assert!(
        !cell.contains("engine:c:foo") && cell.contains("\"engine:foo_rust\""),
        "{cell}"
    );
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn an_archive_with_live_code_stays_a_static_library() {
    use vibeos_codegen::cells::interface::plan;
    let cmake = "rust_staticlib_variant(foo_rust foo-rust \"\" TRUE)\n\
add_library(foo STATIC rust_cohab_stub.c shim.c)\ntarget_link_libraries(foo ${foo_rust_LIB})\n";
    let stub = "typedef int foo_placeholder;\n";
    let live =
        "lib/foo/CMakeFiles/foo.dir/rust_cohab_stub.c.obj lib/foo/CMakeFiles/foo.dir/shim.c.obj";
    let root = interface_fixture("live", cmake, live, stub);
    assert!(plan(&root, "foo")
        .unwrap_err()
        .message
        .contains("more than the placeholder stub"));
    std::fs::remove_dir_all(&root).unwrap();
    // A stub that defines something is not a placeholder either (cportlib's anchor).
    let objs = "lib/foo/CMakeFiles/foo.dir/rust_cohab_stub.c.obj";
    let root = interface_fixture("anchor", cmake, objs, "void __foo_anchor(void) { }\n");
    assert!(plan(&root, "foo")
        .unwrap_err()
        .message
        .contains("not a placeholder"));
    std::fs::remove_dir_all(&root).unwrap();
    // No Rust archive linked: nothing to forward (gdi32_vista_static was dead).
    let none = "add_library(foo STATIC rust_cohab_stub.c)\ntarget_link_libraries(foo win32ksys)\n";
    let root = interface_fixture("none", none, objs, "typedef int x;\n");
    assert!(plan(&root, "foo")
        .unwrap_err()
        .message
        .contains("links no Rust archive"));
    std::fs::remove_dir_all(&root).unwrap();
}
