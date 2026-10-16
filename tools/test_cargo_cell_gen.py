import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import cargo_cell_gen as g

CMAKE = """\
list(APPEND SOURCE
    # main.c  # replaced by Rust (sdk/lib/rust/m-rust/src/m/main.rs)
    rust_cohab_stub.c
    main.h)

file(GLOB m_rc_deps rc/*.*)
add_rc_deps(rsrc.rc ${m_rc_deps})
# Every translation unit is Rust now; the target keeps a stub TU.
rust_staticlib_variant(m_rust m-rust "" TRUE)
add_executable(m ${SOURCE} rsrc.rc)
add_dependencies(m rust_m_rust)
target_link_libraries(m ${m_rust_LIB})
add_pch(m main.h SOURCE)
set_module_type(m win32gui UNICODE)
target_link_libraries(m wine)
add_importlibs(m user32 msvcrt kernel32 ntdll)
add_cd_file(TARGET m DESTINATION reactos/system32 FOR all)
"""


def cell(**kw):
    base = dict(
        target="m",
        cmake_dir=Path("."),
        output_name="m.exe",
        module_kind="Win32Exe",
        entries={a: "wWinMainCRTStartup" for a in g.ARCHES},
        subsystem="windows:5.01",
        image_base={a: None for a in g.ARCHES},
        stack=None,
        body_crate="m-rust",
        body_features=[],
        body_defaults=True,
        import_libs=[],
        msvcrtex="ExeWide",
        static_libs={a: ["sdk/lib/libwine/libwine.a"] for a in g.ARCHES},
        spec=None,
        spec_stubs=False,
        c_shims=[],
        rc="base/m/rsrc.rc",
        rc_defines=["UNICODE", "_UNICODE"],
        destinations=["reactos/system32"],
        name_on_cd=None,
    )
    base.update(kw)
    return g.Cell(**base)


def test_facts_read_the_body_crate_and_the_cd_destination(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE)
    facts = g.cmake_facts(f, "m")
    assert facts["body"] == ("m-rust", [], True)
    assert facts["destinations"] == ["reactos/system32"]
    assert facts["unknown"] == []


def test_an_unknown_statement_on_the_target_is_reported(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE + "set_target_properties(m PROPERTIES FOO 1)\n")
    assert g.cmake_facts(f, "m")["unknown"] == ["set_target_properties"]


def test_a_second_target_built_from_the_source_list_is_refused(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE + "add_library(m_static ${SOURCE})\n")
    assert g.cmake_facts(f, "m")["unknown"] == ["add_library m_static shares ${SOURCE}"]


def test_rewrite_replaces_the_link_and_keeps_the_resource_deps(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE)
    out = g.rewrite_cmake(f, cell())
    for gone in (
        "add_executable(",
        "set_module_type(",
        "add_importlibs(",
        "add_pch(",
        "rust_staticlib_variant(",
        "list(APPEND SOURCE",
        "keeps a stub TU",
    ):
        assert gone not in out
    assert "add_rc_deps(rsrc.rc ${m_rc_deps})" in out
    # The cohab census reads these: a translated TU's record survives the rewrite.
    assert "# main.c  # replaced by Rust (sdk/lib/rust/m-rust/src/m/main.rs)" in out
    assert "add_library(m_res OBJECT rsrc.rc)" in out
    assert "target_compile_definitions(m_res PRIVATE UNICODE _UNICODE)" in out
    assert "add_cargo_module(m m-cargo m.exe" in out
    assert "set(M_REUSED\n    ${CMAKE_BINARY_DIR}/sdk/lib/libwine/libwine.a)" in out
    assert "EXTRA_DEPENDS m_res ${M_REUSED})" in out


def test_an_arch_only_library_is_an_arch_only_dependency(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE)
    libs = {a: ["sdk/lib/libwine/libwine.a"] for a in g.ARCHES}
    libs["i386"].append("sdk/lib/uuid/libuuid.a")
    out = g.rewrite_cmake(f, cell(static_libs=libs))
    chk = "list(APPEND M_REUSED ${CMAKE_BINARY_DIR}/sdk/lib/uuid/libuuid.a)"
    assert f'if(ARCH STREQUAL "i386")\n    {chk}' in out
    rs = g.build_rs(cell(static_libs=libs))
    assert '"x86" => &["sdk/lib/libwine/libwine.a", "sdk/lib/uuid/libuuid.a"],' in rs


def test_an_engine_built_library_is_linked_but_not_a_cmake_dependency(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE)
    libs = {a: ["engine:oldnames", "sdk/lib/libwine/libwine.a"] for a in g.ARCHES}
    libs["i386"].append("engine:chkstk")
    out = g.rewrite_cmake(f, cell(static_libs=libs))
    assert "engine:" not in out
    assert "set(M_REUSED\n    ${CMAKE_BINARY_DIR}/sdk/lib/libwine/libwine.a)" in out
    rs = g.build_rs(cell(static_libs=libs))
    assert '"x86" => &["engine:oldnames", "sdk/lib/libwine/libwine.a", "engine:chkstk"],' in rs
    assert g.ENGINE_LIBS["sdk/lib/crt/libchkstk.a"] == "engine:chkstk"


def test_an_engine_replayed_library_orders_the_cell_after_its_headers(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE)
    libs = {a: ["engine:wine", "engine:conutils_rust"] for a in g.ARCHES}
    out = g.rewrite_cmake(f, cell(static_libs=libs))
    assert "EXTRA_DEPENDS m_res psdk)" in out
    assert "REUSED" not in out


def test_build_rs_names_every_input_and_keeps_an_auto_dll_base():
    rs = g.build_rs(cell(import_libs=[("dll/x/kernel32.spec", "kernel32.dll", "kernel32")]))
    assert '.rust_staticlib("m-rust")' in rs
    assert '.import_lib("dll/x/kernel32.spec", "kernel32.dll", "kernel32")' in rs
    assert '.resource_from_rc("base/m/rsrc.rc", "m_res")' in rs
    assert ".msvcrtex_crt(MsvcrtexKind::ExeWide)" in rs
    # A DLL cmake gave no --image-base keeps lld's auto base: None, never a guess.
    bases = {"i386": None, "amd64": 0x7FE60000000, "arm64": 0x7FE60000000}
    dll = g.build_rs(cell(module_kind="Win32Dll", image_base=bases, msvcrtex="Dll"))
    assert '"x86" => ("i686-pc-windows-gnu", "wWinMainCRTStartup", None)' in dll
    assert "Some(0x7fe60000000u64)" in dll


def test_a_placeholder_stub_is_recognised_and_real_c_is_not(tmp_path):
    stub = tmp_path / "rust_cohab_stub.c"
    stub.write_text("/* m is Rust; a CMake target needs one TU. */\ntypedef int m_placeholder;\n")
    assert g._placeholder(stub)
    stub.write_text("int WINAPI helper(void) { return 0; }\n")
    assert not g._placeholder(stub)


def test_real_c_is_replayed_from_its_own_object_library(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE)
    c = cell(c_shims=["base/m/rust_varargs_shim.c"])
    out = g.rewrite_cmake(f, c)
    assert "add_library(m_shims OBJECT rust_varargs_shim.c)" in out
    # What the target linked carried compile flags (libmsvcrt: _DLL __USE_CRTIMP).
    assert (
        "target_link_libraries(m_shims PRIVATE wine libuser32 libmsvcrt libkernel32 libntdll)"
        in out
    )
    assert "target_compile_definitions(m_shims PRIVATE UNICODE _UNICODE)" in out
    assert "EXTRA_DEPENDS m_shims m_res" in out
    assert '.c_from_compile_db("m_shims", &["base/m/rust_varargs_shim.c"])' in g.build_rs(c)
    assert ".c_shims(" not in g.build_rs(c)


def test_arch_specific_cmake_is_refused(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    # calc: an i386-only RUSTFLAGS for the body the cell would silently drop.
    f.write_text(CMAKE.replace('"" TRUE)', '"" TRUE ${M_RUST_ENV})'))
    assert g.cmake_facts(f, "m")["unknown"] == ["rust_staticlib_variant extras ['${M_RUST_ENV}']"]
    # Several destinations are read as a list (comctl32's winsxs copies); whether each
    # arch really ships to all of them (not winprint's one per arch) is model()'s check
    # against the build's CD lists.
    f.write_text(CMAKE + "add_cd_file(TARGET m DESTINATION reactos/other FOR all)\n")
    facts = g.cmake_facts(f, "m")
    assert facts["unknown"] == []
    assert facts["destinations"] == ["reactos/system32", "reactos/other"]


def test_the_panic_handler_follows_what_cmake_linked_per_arch():
    # No arch carried the body's handler: panic-lean only, and no panic_common at all
    # (a linked panic_common loads core through its #[no_mangle] fallback alone).
    lean = cell(panics=dict.fromkeys(g.ARCHES, False))
    assert "panic_lean::panic_lean" in g.main_rs(lean)
    assert "panic-common" not in g.cargo_toml(lean)
    # Per arch: panic_common is an arch-scoped dependency, used only there.
    mixed = cell(panics={"i386": True, "amd64": True, "arm64": False})
    rs, toml = g.main_rs(mixed), g.cargo_toml(mixed)
    on = 'any(target_arch = "x86", target_arch = "x86_64")'
    assert f"#[cfg({on})]\n    return panic_common::panic_common(" in rs
    assert f"#[cfg(not({on}))]\n    return panic_lean::panic_lean(" in rs
    assert f"[target.'cfg({on})'.dependencies]\npanic-common" in toml
    assert "panic_lean" not in g.main_rs(cell())


def test_the_body_is_the_variant_the_module_links(tmp_path):
    # help.exe's target is cmd_help and its body variant cmd_help_rust; a module whose
    # variant is named otherwise is matched by the lib<name>.a in its link.
    f = tmp_path / "CMakeLists.txt"
    f.write_text(
        CMAKE.replace(
            "rust_staticlib_variant(m_rust m-rust", "rust_staticlib_variant(odd_rust m-rust"
        )
    )
    assert g.cmake_facts(f, "m")["body"] is None
    facts = g.cmake_facts(f, "m", ["base/m/libodd_rust.a"])
    assert facts["body"] == ("m-rust", [], True) and facts["body_var"] == "odd_rust"


def test_a_target_made_in_a_loop_is_refused(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE.replace("add_executable(m ", "add_executable(${_layout} "))
    assert "target made in a loop or function" in g.cmake_facts(f, "m")["unknown"]


def test_compile_settings_move_to_the_object_libraries(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(
        CMAKE
        + "target_compile_definitions(m PRIVATE _CRT_NON_CONFORMING_SWPRINTFS)\n"
        + 'set_target_properties(m PROPERTIES SUFFIX ".com")\n'
    )
    assert g.cmake_facts(f, "m")["unknown"] == []
    out = g.rewrite_cmake(f, cell())
    assert "target_compile_definitions(m_res PRIVATE _CRT_NON_CONFORMING_SWPRINTFS)" in out
    assert "(m PRIVATE _CRT" not in out and "SUFFIX" not in out


def test_a_generated_input_stays_ordered_before_the_compile_that_reads_it(tmp_path):
    # eventcreate.rc #includes the evtmsgstr.mc output: dropping the dependency raced windres.
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE + "add_dependencies(m m_msg)\n")
    out = g.rewrite_cmake(f, cell())
    assert "add_dependencies(m_res m_msg)" in out
    assert "rust_m_rust" not in out.split("add_dependencies(m_res", 1)[1].split(")")[0]
    no_objlib = g.rewrite_cmake(f, cell(rc=None))
    assert no_objlib.rstrip().endswith("add_dependencies(m m_msg)")


def test_libs_cmake_linked_after_the_body_stay_after_it():
    libs = {a: ["sdk/lib/crt/oldnames.a", "sdk/lib/uuid/libuuid.a"] for a in g.ARCHES}
    late = {a: ["sdk/lib/uuid/libuuid.a"] for a in g.ARCHES}
    rs = g.build_rs(cell(static_libs=libs, late_libs=late))
    assert '"x86" => &["sdk/lib/crt/oldnames.a"],' in rs
    assert '"x86" => &["sdk/lib/uuid/libuuid.a"],' in rs
    assert ".static_libs_after_body(late_libs)" in rs
    assert ".static_libs_after_body" not in g.build_rs(cell())


def test_delay_imports_link_through_the_engine_and_fix_up_after(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE + "add_delay_importlibs(m user32)\n")
    assert g.cmake_facts(f, "m")["unknown"] == []
    c = cell(delay_import_libs=[("win32ss/user/user32/user32.spec", "user32.dll", "user32")])
    rs = g.build_rs(c)
    assert '.delay_import_lib("win32ss/user/user32/user32.spec", "user32.dll", "user32")' in rs
    out = g.rewrite_cmake(f, c)
    assert "add_delay_importlibs" not in out
    assert "    DELAY_IMPORTS" in out
    assert "DELAY_IMPORTS" not in g.rewrite_cmake(f, cell())


def test_generated_c_is_compiled_from_the_build_dir(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE)
    stub = "base/services/m/m_s.c"
    c = cell(c_shims=[stub], generated=[stub])
    assert "add_library(m_shims OBJECT ${CMAKE_CURRENT_BINARY_DIR}/m_s.c)" in g.rewrite_cmake(f, c)
    assert f'.c_from_compile_db("m_shims", &["{stub}"])' in g.build_rs(c)


def test_several_cd_destinations_go_to_one_call(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE)
    c = cell(destinations=["reactos/system32", "reactos/winsxs/${WINARCH}_x"])
    assert "    DESTINATION reactos/system32 reactos/winsxs/${WINARCH}_x" in g.rewrite_cmake(f, c)


def test_a_body_rustflags_env_is_modelled_only_in_its_exact_shape():
    block = (
        "set(M_RUST_ENV)\n"
        'if(ARCH STREQUAL "i386")\n'
        '    set(M_RUST_ENV ENV "RUSTFLAGS=-C target-feature=+sse,+sse2")\n'
        "endif()\n"
    )
    text = block + 'rust_staticlib_variant(m_rust m-rust "" TRUE ${M_RUST_ENV})\n'
    flags, found = g._body_env(text, ["${M_RUST_ENV}"])
    assert flags == {"i386": "-C target-feature=+sse,+sse2"}
    assert found == block
    # Read anywhere else, or not this shape: refused.
    assert g._body_env(text + "message(${M_RUST_ENV})\n", ["${M_RUST_ENV}"]) is None
    assert g._body_env(text, ["ENV", "RUSTFLAGS=x"]) is None
    rs = g.build_rs(cell(body_rustflags={"i386": "-C target-feature=+sse,+sse2"}))
    assert '"x86" => Some("-C target-feature=+sse,+sse2"),' in rs
    assert "link = link.rust_build_rustflags(flags);" in rs


def test_per_arch_cd_destinations_become_one_variable(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE)
    dests = {"i386": ["d/w32x86"], "amd64": ["d/x64"], "arm64": ["d/arm64"]}
    out = g.rewrite_cmake(
        f, cell(arch_destinations=dests, destinations=["d/x64", "d/arm64", "d/w32x86"])
    )
    assert 'if(ARCH STREQUAL "i386")\n    set(M_DESTINATION d/w32x86)\nelseif' in out
    assert "    DESTINATION ${M_DESTINATION}\n" in out


def test_a_native_app_links_as_an_exe_with_its_subsystem_and_no_crt():
    entries = {a: "NtProcessStartup" for a in g.ARCHES}
    rs = g.build_rs(cell(subsystem="native:5.01", entries=entries, msvcrtex=None))
    assert 'subsystem: "native:5.01".into(),' in rs
    assert "msvcrtex_crt" not in rs


def test_a_source_list_of_placeholders_is_spelled_out_for_the_other_target(tmp_path):
    (tmp_path / "rust_cohab_stub.c").write_text("/* placeholder */\ntypedef int x_t;\n")
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE + "add_library(m_static ${SOURCE})\n")
    assert g.cmake_facts(f, "m")["unknown"] == []
    out = g.rewrite_cmake(f, cell(cmake_dir=tmp_path))
    assert "add_library(m_static rust_cohab_stub.c main.h)" in out


DRIVER = """\
set(D_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(D_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-d-target" d-rust)
set(D_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libd_rust.a")
if(ARCH STREQUAL "i386")
    set(D_RUST_TARGET "i686-pc-windows-gnu")
    set(D_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(D_RUST_TARGET "x86_64-pc-windows-gnu")
    set(D_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(D_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(D_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(D_RUST_SRCS "${D_RUST_WORKSPACE_DIR}" d-rust)
add_custom_command(
    OUTPUT ${D_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${D_RUST_TARGET_DIR}
        RUSTFLAGS=${D_RUST_RUSTFLAGS}
        cargo build --release --target ${D_RUST_TARGET}
            -p d-rust
            --manifest-path ${D_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${D_RUST_TARGET_DIR}/${D_RUST_TARGET}/release/libd_rust.a
        ${D_RUST_LIB}
    DEPENDS ${D_RUST_SRCS}
            ${D_RUST_WORKSPACE_DIR}/d-rust/Cargo.toml
    WORKING_DIRECTORY ${D_RUST_WORKSPACE_DIR}
    COMMENT "Building d-rust (${D_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_d DEPENDS ${D_RUST_LIB})
add_library(d MODULE rust_cohab_stub.c d.rc)
set_module_type(d kernelmodedriver)
add_importlibs(d ntoskrnl hal)
add_dependencies(d rust_d)
target_link_libraries(d ${D_RUST_LIB})
add_cd_file(TARGET d DESTINATION reactos/system32/drivers NO_CAB FOR all)
add_driver_inf(d d.inf)
"""


def test_a_driver_custom_command_is_its_body():
    body = g._custom_body(DRIVER)
    assert body["crate"] == "d-rust" and body["archive"] == "libd_rust.a"
    assert body["rustflags"] == {"i386": "-C target-feature=-sse,-sse2"}
    assert body["custom_target"] == "rust_d"
    # Any other shape of the command is not modelled: here, a feature.
    assert g._custom_body(DRIVER.replace("-p d-rust", "-p d-rust --features x")) is None


def test_a_driver_rewrite_drops_its_body_command_and_keeps_its_inf(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    # The header comment sits on the body's first set(): it outlives the statements
    # (cohab_census counts its `.c` names), and the emptied if-chain must not take it.
    f.write_text("# d.sys: a.c b.c -> d-rust\n" + DRIVER)
    facts = g.cmake_facts(f, "d")
    assert facts["unknown"] == [] and facts["no_cab"] and facts["module_type"] == "kernelmodedriver"
    c = cell(
        target="d",
        cmake_dir=tmp_path,
        output_name="d.sys",
        module_kind="KernelDll",
        kernel_kind="kernelmodedriver",
        custom_body=g._custom_body(DRIVER),
        body_crate="d-rust",
        destinations=["reactos/system32/drivers"],
        no_cab=True,
        static_libs={a: [] for a in g.ARCHES},
        msvcrtex=None,
        rc=None,
    )
    out = g.rewrite_cmake(f, c)
    assert "cargo build" not in out and "D_RUST" not in out and "rust_d" not in out
    assert "add_driver_inf(d d.inf)" in out
    assert out.startswith("# d.sys: a.c b.c -> d-rust\n")
    assert "KIND kernelmodedriver" in out and "    NO_CAB" in out


def _driver_cell(tmp_path):
    return cell(
        target="d",
        cmake_dir=tmp_path,
        output_name="d.sys",
        module_kind="KernelDll",
        kernel_kind="kernelmodedriver",
        custom_body=g._custom_body(DRIVER),
        body_crate="d-rust",
        destinations=["reactos/system32/drivers"],
        no_cab=True,
        static_libs={a: [] for a in g.ARCHES},
        msvcrtex=None,
        rc=None,
    )


def test_a_driver_built_under_a_condition_is_a_cell_under_the_same_condition(tmp_path):
    # pciide is built only if(NOT ENABLE_UNIATA). The per-arch body flags inside it empty
    # first, which empties the outer if: both go, and the cell is wrapped in the condition.
    f = tmp_path / "CMakeLists.txt"
    lines = [ln for ln in DRIVER.splitlines(keepends=True) if "add_driver_inf" not in ln]
    body = "".join("    " + ln if ln.strip() else ln for ln in lines)
    f.write_text("if(NOT ENABLE_UNIATA)\n" + body + "endif()\nadd_driver_inf(x x.inf)\n")
    out = g.rewrite_cmake(f, _driver_cell(tmp_path))
    assert out.count("if(NOT ENABLE_UNIATA)") == 1 and "ARCH STREQUAL" not in out
    guarded = out[out.index("if(NOT ENABLE_UNIATA)") :]
    assert "    add_cargo_module(d d-cargo d.sys" in guarded
    assert guarded.rstrip().endswith("endif()")


def test_a_driver_in_an_else_branch_is_refused(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text("if(X)\nelse()\n" + DRIVER + "endif()\n")
    try:
        g.rewrite_cmake(f, _driver_cell(tmp_path))
    except g.GenError as exc:
        assert "else" in str(exc)
    else:
        raise AssertionError("an else-branch link was rewritten")


def test_a_kernel_cell_builds_its_body_on_the_user_triple_and_keeps_every_section():
    rs = g.build_rs(
        cell(
            module_kind="KernelDll",
            entries={a: "DriverEntry" for a in g.ARCHES},
            image_base={a: 0x10000 for a in g.ARCHES},
            msvcrtex=None,
            static_libs={a: [] for a in g.ARCHES},
            link_order={a: ["import:ntoskrnl", "body", "engine:c:memcmp"] for a in g.ARCHES},
        )
    )
    assert '"x86" => ("i686-pc-windows-gnu", "DriverEntry", 0x10000u64),' in rs
    assert ".rust_user_target()" in rs and ".keep_all_sections()" in rs
    assert '"x86" => &["import:ntoskrnl", "body", "engine:c:memcmp"],' in rs
    assert ".link_order(link_order)" in rs and ".static_libs(" not in rs
    assert "ModuleKind::KernelDll" in rs


def test_a_kernel_cell_without_the_bodys_handler_traps():
    main = g.main_rs(cell(module_kind="KernelDll", panics=dict.fromkeys(g.ARCHES, False)))
    assert "panic_lean::panic_fastfail()" in main
    deps = g._panic_deps(cell(module_kind="KernelDll", panics=dict.fromkeys(g.ARCHES, False)))
    assert "panic-lean" in deps and "panic-common" not in deps


def test_a_boot_cd_only_placement_and_a_lowered_subsystem(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(
        CMAKE
        + 'if(ARCH STREQUAL "i386")\n'
        + '    target_link_options(m PRIVATE "-Wl,--subsystem,windows:4.00")\n'
        + "endif()\n"
        + "add_cd_file(TARGET m DESTINATION root NO_CAB NAME_ON_CD setup.exe FOR bootcd)\n"
    )
    facts = g.cmake_facts(f, "m")
    assert facts["unknown"] == [] and facts["name_on_cd"] is None
    assert facts["extra_cd"] == ["DESTINATION root NO_CAB NAME_ON_CD setup.exe FOR bootcd"]
    subs = {"i386": "windows:4.00", "amd64": "windows:5.01", "arm64": "windows:5.01"}
    c = cell(cmake_dir=tmp_path, extra_cd=facts["extra_cd"], subsystems=subs)
    out = g.rewrite_cmake(f, c)
    assert "target_link_options" not in out and "if(ARCH" not in out
    assert "add_cargo_cd_file(m DESTINATION root NO_CAB NAME_ON_CD setup.exe FOR bootcd)" in out
    assert '"x86" => "windows:4.00",' in g.build_rs(c)


def test_a_stack_protector_object_becomes_a_configure_error(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(
        CMAKE
        + "if(STACK_PROTECTOR)\n"
        + "    target_sources(m PRIVATE $<TARGET_OBJECTS:gcc_ssp_nt>)\n"
        + "endif()\n"
    )
    assert g.cmake_facts(f, "m")["ssp"]
    out = g.rewrite_cmake(f, cell(cmake_dir=tmp_path))
    assert 'message(FATAL_ERROR "m is linked by cargo' in out and "if(STACK_PROTECTOR)" in out


def test_a_cell_for_a_digit_leading_target_takes_its_body_crates_name():
    c = cell(target="3dtext", body_crate="text3d-rust")
    assert c.crate == "text3d-cargo"
    assert 'name = "text3d"' in g.cargo_toml(c)


def test_a_shim_from_another_directory_keeps_its_path(tmp_path):
    # cscript compiles wscript's varargs shim: `cscript.dir/__/wscript/rust_varargs_shim.c.obj`.
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE)
    shim = "base/wscript/rust_varargs_shim.c"
    c = cell(c_shims=[shim], foreign=[shim])
    out = g.rewrite_cmake(f, c)
    assert f"add_library(m_shims OBJECT ${{REACTOS_SOURCE_DIR}}/{shim})" in out
    assert f'.c_from_compile_db("m_shims", &["{shim}"])' in g.build_rs(c)


def test_the_engine_built_delayimp_counts_as_the_delayimp_runtime():
    # Checked after the ENGINE_LIBS mapping, so the engine's name must count: cscript was
    # refused "delay imports without the delayimp runtime" while linking it.
    assert g._links_delayimp(["engine:uuid", "engine:delayimp_rust"])
    assert g._links_delayimp(["sdk/lib/delayimp/libdelayimp.a"])
    assert not g._links_delayimp(["engine:uuid", "engine:wine"])
