import struct
import subprocess
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


RC_FLAGS = g.RcFlags(("@build/base/m", "base/m", "@std"), (), ("UNICODE", "_UNICODE"))


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
        rc_flags={a: RC_FLAGS for a in g.ARCHES},
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


def test_rewrite_replaces_the_link_and_the_resource_compile(tmp_path):
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
    # The engine compiles the .rc (Linker::rc): its cmake deps and their GLOB go too.
    assert "rc_deps" not in out
    # The cohab census reads these: a translated TU's record survives the rewrite.
    assert "# main.c  # replaced by Rust (sdk/lib/rust/m-rust/src/m/main.rs)" in out
    assert "_res" not in out
    assert "add_cargo_module(m m-cargo m.exe" in out
    assert "set(M_REUSED\n    ${CMAKE_BINARY_DIR}/sdk/lib/libwine/libwine.a)" in out
    assert "EXTRA_DEPENDS psdk ${M_REUSED})" in out


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
    assert "EXTRA_DEPENDS psdk)" in out
    assert "REUSED" not in out


def test_build_rs_names_every_input_and_keeps_an_auto_dll_base():
    rs = g.build_rs(cell(import_libs=[("dll/x/kernel32.spec", "kernel32.dll", "kernel32")]))
    assert '.rust_staticlib("m-rust")' in rs
    assert '.import_lib("dll/x/kernel32.spec", "kernel32.dll", "kernel32")' in rs
    assert '.rc("base/m/rsrc.rc", rc_includes, rc_flags, rc_defines)' in rs
    assert '"@build/base/m", "base/m", "@std"' in rs
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
    assert "EXTRA_DEPENDS m_shims psdk" in out
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
    out = g.rewrite_cmake(f, cell(c_shims=["base/m/rust_varargs_shim.c"]))
    assert "target_compile_definitions(m_shims PRIVATE _CRT_NON_CONFORMING_SWPRINTFS)" in out
    assert "(m PRIVATE _CRT" not in out and "SUFFIX" not in out
    # The .rc's own command already carries them (Cell.rc_flags is read from it).
    assert "_res" not in g.rewrite_cmake(f, cell())


def test_a_generated_input_stays_ordered_before_the_compile_that_reads_it(tmp_path):
    # eventcreate.rc #includes the evtmsgstr.mc output: dropping the dependency raced windres.
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE + "add_dependencies(m m_msg)\n")
    out = g.rewrite_cmake(f, cell(c_shims=["base/m/rust_varargs_shim.c"]))
    assert "add_dependencies(m_shims m_msg)" in out
    assert "rust_m_rust" not in out.split("add_dependencies(m_shims", 1)[1].split(")")[0]
    # The engine's rc compile is the cell's, so with no OBJECT library the cell waits.
    no_objlib = g.rewrite_cmake(f, cell())
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


def test_convert_models_every_target_before_writing_anything(tmp_path, monkeypatch):
    """One refusal in a --convert batch must leave no half-applied cells behind."""
    import pytest

    writes = []

    def model(t, repo):
        if t == "bad":
            raise g.GenError("bad: refused")
        return t

    monkeypatch.setattr(g, "model", model)
    monkeypatch.setattr(g, "save_baseline", lambda *a: writes.append("baseline"))
    monkeypatch.setattr(g, "apply", lambda *a: writes.append("apply") or [])
    with pytest.raises(g.GenError, match="refused"):
        g.convert(["good", "bad"], tmp_path / "b", tmp_path)
    assert writes == []


def test_convert_runs_the_wave_in_order(tmp_path, monkeypatch):
    steps = []
    monkeypatch.setattr(g, "model", lambda t, repo: t)
    monkeypatch.setattr(g, "save_baseline", lambda ts, b, repo: steps.append(("baseline", b)))
    monkeypatch.setattr(g, "apply", lambda c, repo: steps.append(("apply", c)) or [])
    monkeypatch.setattr(g, "_ninja", lambda a, ts, repo: steps.append(("ninja", a)))
    monkeypatch.setattr(g, "check", lambda ts, b, repo: steps.append(("check", b)) or 0)
    assert g.convert(["m"], tmp_path / "b", tmp_path) == 0
    assert steps == [
        ("baseline", tmp_path / "b"),
        ("apply", "m"),
        ("ninja", "i386"),
        ("ninja", "amd64"),
        ("ninja", "arm64"),
        ("check", tmp_path / "b"),
    ]


def _native_ratchet(root):
    from tests.qemu.cli import rustlints

    repo = Path(__file__).resolve().parent.parent
    cp = subprocess.run(
        [str(rustlints.ensure(repo)), "--root", str(root), "--rules", "scan_cmake_linked_rust"],
        capture_output=True,
        text=True,
        check=True,
    )
    return [line.split("\t", 1)[1] for line in cp.stdout.splitlines()]


def _ratchet_fixture(root, extra=""):
    def put(relative, text):
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)

    put(
        "foo/CMakeLists.txt",
        "add_library(foo MODULE rust_cohab_stub.c)\n"
        'rust_staticlib_variant(foo_rust foo-rust "" TRUE)\n'
        "add_cd_file(TARGET foo DESTINATION /system32 FOR all)\n" + extra,
    )
    put("foo/rust_cohab_stub.c", "/* placeholder */\n")
    put("sdk/lib/rust/foo-cargo/build.rs", "fn main() {}\n")
    pe = bytearray(0x200)
    pe[:2] = b"MZ"
    struct.pack_into("<I", pe, 0x3C, 0x80)
    pe[0x80:0x84] = b"PE\0\0"
    struct.pack_into("<H", pe, 0x84, 0x8664)
    struct.pack_into("<H", pe, 0x94, 0xF0)
    struct.pack_into("<H", pe, 0x98, 0x20B)
    for arch in ("i386", "amd64", "arm64"):
        put(
            f"build-{arch}/build.ninja",
            "build foo/foo.dll: C_MODULE_LIBRARY_LINKER__foo_Debug "
            "foo/CMakeFiles/foo.dir/rust_cohab_stub.c.obj\n"
            "  LINK_FLAGS = --subsystem,windows\n"
            "  LINK_LIBRARIES = sdk/lib/rust/foo_rust/libfoo_rust.a\n",
        )
        put(
            f"build-{arch}/boot/livecd.Release.lst",
            f"/system32/foo.dll=/tmp/vibeos/build-{arch}/foo/foo.dll\n",
        )
        path = root / f"build-{arch}/foo/foo.dll"
        path.parent.mkdir(parents=True)
        path.write_bytes(pe)


def test_ratchet_labels_follow_the_generator(tmp_path):
    # Exercise the real three-arch model instead of substituting a survey result.
    for i, (extra, label) in enumerate(
        [
            ("", "convertible"),
            ("unknown_target_property(foo X)\n", "refused"),
        ]
    ):
        root = tmp_path / str(i)
        _ratchet_fixture(root, extra)
        [(target, verdict)] = g.survey(root)
        assert target == "foo"
        assert verdict.startswith("OK") == (label == "convertible")
        archive = root / "sdk/lib/rust/c-cargo/build.rs"
        archive.parent.mkdir(parents=True)
        archive.write_text('&["body", "boot/x/libbios_asm.a"]\n')
        assert g.engine_lib("boot/x/libbios_asm.a", root) is None
        assert _native_ratchet(root) == sorted(
            [
                f"foo: fully-Rust module still linked by cmake ({label})",
                "c-cargo: links cmake-built boot/x/libbios_asm.a (not engine-buildable)",
            ]
        )


def _cml(root, rel, text):
    path = root / rel / "CMakeLists.txt"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


def test_engine_lib_builds_any_cmake_static_library_or_rust_variant(tmp_path):
    g._library_index.cache_clear()
    _cml(tmp_path, "sdk/lib/nt", "add_library(nt entry_point.c)\nadd_dependencies(nt psdk)\n")
    _cml(
        tmp_path,
        "vendor/sr",
        "add_library(libsamplerate STATIC ${SOURCE})\n",
    )
    _cml(
        tmp_path,
        "sdk/lib/mm",
        'rust_staticlib_variant(mm_rust mm-rust "user kernel" FALSE)\n'
        'rust_staticlib_variant(env_rust env-rust "" TRUE ENV RUSTFLAGS=-x)\n'
        "add_library(shared_one SHARED a.c)\n",
    )
    try:
        assert g.engine_lib("sdk/lib/nt/libnt.a", tmp_path) == ("engine:c:nt", ["psdk"])
        # The archive's directory need not be its CMakeLists' (add_subdirectory remaps it).
        assert g.engine_lib("sdk/lib/3rdparty/sr/liblibsamplerate.a", tmp_path) == (
            "engine:c:libsamplerate",
            ["psdk"],
        )
        assert g.engine_lib("sdk/lib/mm/libmm_rust.a", tmp_path) == (
            "engine:crate:mm-rust:user,kernel:FALSE",
            [],
        )
        assert g.engine_lib("sdk/lib/mm/libenv_rust.a", tmp_path) is None
        assert g.engine_lib("sdk/lib/mm/libshared_one.a", tmp_path) is None
        assert g.engine_lib("sdk/lib/x/libunknown.a", tmp_path) is None
        # The hand-kept table still wins.
        assert g.engine_lib("sdk/lib/crt/libchkstk.a", tmp_path)[0] == "engine:chkstk"
    finally:
        g._library_index.cache_clear()


def test_remap_moves_a_cell_off_cmake_archives(tmp_path):
    g._library_index.cache_clear()
    _cml(tmp_path, "sdk/lib/nt", "add_library(nt entry_point.c)\nadd_dependencies(nt xdk)\n")
    _cml(
        tmp_path,
        "base/m",
        "set(M_REUSED\n    ${CMAKE_BINARY_DIR}/sdk/lib/nt/libnt.a)\n"
        "add_cargo_module(m m-cargo m.exe\n    KIND win32exe\n    BODY_CRATE m-rust\n"
        "    EXTRA_DEPENDS m_res ${M_REUSED} psdk)\n",
    )
    rs = tmp_path / "sdk/lib/rust/m-cargo/build.rs"
    rs.parent.mkdir(parents=True)
    rs.write_text('&["body", "sdk/lib/nt/libnt.a", "import:ntdll"]\n')
    try:
        assert g.remap(None, tmp_path) == ["m-cargo"]
    finally:
        g._library_index.cache_clear()
    assert rs.read_text() == '&["body", "engine:c:nt", "import:ntdll"]\n'
    cml = (tmp_path / "base/m/CMakeLists.txt").read_text()
    assert "M_REUSED" not in cml
    assert "EXTRA_DEPENDS m_res psdk xdk)" in cml


def test_rebound_exports_name_what_the_spec_actually_exports():
    """REBOUND_EXPORTS accepts a module whose exports cmake bound to import thunks. Every
    name listed must still be an export of that module's spec, or the acceptance is
    covering for something that no longer exists."""
    repo = Path(__file__).resolve().parent.parent
    specs = {"user32_vista": repo / "win32ss/user/user32_vista/user32_vista.spec"}
    for module, names in g.REBOUND_EXPORTS.items():
        text = specs[module].read_text()
        for name in names:
            assert f" {name}(" in text, f"{module}: {name} is no longer exported"


def test_ratchet_trusts_the_source_tree_over_a_stale_graph(tmp_path):
    graph = tmp_path / "build-amd64/build.ninja"
    graph.parent.mkdir()
    graph.write_text(
        "".join(
            f"build {t}/{t}.dll: C_SHARED_LIBRARY_LINKER__{t}_Debug rust_cohab_stub.c.obj\n"
            for t in ("m", "n", "o")
        )
    )
    _cml(tmp_path, "dll/m", "add_cargo_module(m m-cargo m.dll)\n")
    _cml(tmp_path, "dll/n", "add_library(n MODULE rust_cohab_stub.c)\n")
    _cml(tmp_path, "dll/o", "add_library(o MODULE o.c font.c)\n")
    assert _native_ratchet(tmp_path) == ["n: fully-Rust module still linked by cmake (refused)"]


def test_the_engine_compiles_the_rc_and_cmake_keeps_no_res_library(tmp_path):
    """The .rc is Linker::rc's: no `<m>_res` OBJECT library, and a generated input the .rc
    embeds (OBJECT_DEPENDS on a binary-dir file) orders the cell instead."""
    f = tmp_path / "CMakeLists.txt"
    f.write_text(
        CMAKE
        + "set_source_files_properties(rsrc.rc PROPERTIES OBJECT_DEPENDS "
        + "${CMAKE_CURRENT_BINARY_DIR}/m.tlb)\n"
    )
    out = g.rewrite_cmake(f, cell(rc="base/m/rsrc.rc"))
    assert "_res" not in out
    assert "add_rc_deps" not in out and "OBJECT_DEPENDS" not in out
    assert "EXTRA_DEPENDS psdk ${CMAKE_CURRENT_BINARY_DIR}/m.tlb ${M_REUSED})" in out


RC_CMD = (
    "/tmp/vibeos/.build-tools/llvm-mingw/bin/x86_64-w64-mingw32-windres -O coff --codepage=65001 "
    "-I /tmp/vibeos/build-amd64/base/m -I /tmp/vibeos/base/m "
    "-I /tmp/vibeos/sdk/include -I /tmp/vibeos/build-amd64/sdk/include "
    "-I /tmp/vibeos/vendor/x -I /tmp/vibeos/sdk/include/crt -U_X86_ -UWIN32 "
    "-DRC_INVOKED -D__WIN32__=1 -D__FLAT__=1 -DLANGUAGE_EN_US "
    "-DDBG=1 -DUNICODE -D_UNICODE -D__REACTOS__ "
    "/tmp/vibeos/base/m/rsrc.rc base/m/CMakeFiles/m.dir/rsrc.rc.res"
)
TMPL = g.RcTemplate(
    windres="{REPO}/.build-tools/llvm-mingw/bin/x86_64-w64-mingw32-windres",
    std=["sdk/include", "@build/sdk/include"],
    flags=["-I", "{REPO}/sdk/include/crt", "-U_X86_", "-UWIN32"],
    defs=["-DRC_INVOKED", "-D__WIN32__=1", "-D__FLAT__=1", "-DLANGUAGE_EN_US"],
    defines=["-DDBG=1", "-D__REACTOS__"],
)


def test_an_rc_command_is_the_template_plus_the_module_part():
    f = g.rc_encode(RC_CMD, TMPL, "amd64")
    assert f.includes == ("@build/base/m", "base/m", "@std", "vendor/x")
    assert f.flags == ()
    assert f.defines == ("UNICODE", "_UNICODE")
    # Linker::rc's order: the defines join the global ones, sorted as cmake sorts them.
    assert g.rc_decode(f, TMPL, "amd64", "/tmp/vibeos/base/m/rsrc.rc") == RC_CMD.split()[:-1]


def test_an_rc_command_the_template_cannot_rebuild_is_refused():
    import pytest

    with pytest.raises(g.GenError, match="global define"):
        g.rc_encode(RC_CMD.replace("-DDBG=1 ", ""), TMPL, "amd64")
    with pytest.raises(g.GenError, match="not sorted"):
        g.rc_encode(RC_CMD.replace("-DUNICODE -D_UNICODE", "-D_UNICODE -DUNICODE"), TMPL, "amd64")


def test_the_rc_template_file_reads_back(tmp_path, monkeypatch):
    monkeypatch.setattr(g, "rc_template_path", lambda a, repo=g.REPO: tmp_path / f"rc.{a}.flags")
    (tmp_path / "rc.amd64.flags").write_text(TMPL.text("amd64"))
    assert g.read_rc_template("amd64") == TMPL


def test_rc_object_depends_reads_both_spellings():
    # (the args, as _statements gives them)
    assert g._rc_object_depends("add_rc_deps", "a.rc x.ico") == ["x.ico"]
    raw = 'a.rc PROPERTIES OBJECT_DEPENDS "${B}/t.tlb"'
    assert g._rc_object_depends("set_source_files_properties", raw) == ["${B}/t.tlb"]
    raw = "a.c PROPERTIES COMPILE_FLAGS -O0"
    assert g._rc_object_depends("set_source_files_properties", raw) is None


def test_a_fresh_conversion_reads_its_rc_part_from_the_module_compile(monkeypatch):
    """--convert: the .rc compile is the module target's own (its object path is the marker);
    an arch the module is not built on takes another arch's part."""
    cmds = {
        "amd64": [{"file": "/tmp/vibeos/base/m/rsrc.rc", "command": RC_CMD}],
        "i386": [],
        "arm64": [
            {"file": "/tmp/vibeos/base/m/rsrc.rc", "command": RC_CMD.replace("m.dir", "n.dir")}
        ],
    }
    monkeypatch.setattr(g, "_rc_commands", lambda a, repo=g.REPO: cmds[a])
    monkeypatch.setattr(g, "read_rc_template", lambda a, repo=g.REPO: TMPL)
    flags = g.rc_flags_for("base/m/rsrc.rc", "base/m/CMakeFiles/m.dir/rsrc.rc.res")
    assert flags["amd64"] == flags["i386"] == flags["arm64"]
    assert flags["amd64"].defines == ("UNICODE", "_UNICODE")


C_CMD = (
    "/tmp/vibeos/.build-tools/llvm-mingw/bin/x86_64-w64-mingw32-clang --target=x86_64-w64-mingw32 "
    "-DDBG=1 -DUNICODE -D__REACTOS__ "
    "-I/tmp/vibeos/build-amd64/base/m -I/tmp/vibeos/base/m -I/tmp/vibeos/sdk/include "
    "-I/tmp/vibeos/build-amd64/sdk/include "
    "-std=gnu99 -ffile-prefix-map=/tmp/vibeos= -pipe -O1 -Wno-x "
    "-o base/m/CMakeFiles/m_shims.dir/shim.c.obj -c /tmp/vibeos/base/m/shim.c"
)
CTMPL = g.CTemplate(
    cc="{REPO}/.build-tools/llvm-mingw/bin/x86_64-w64-mingw32-clang",
    target="--target=x86_64-w64-mingw32",
    std=["sdk/include", "@build/sdk/include"],
    core=["-ffile-prefix-map={REPO}=", "-pipe", "-O1"],
    defines=["-DDBG=1", "-D__REACTOS__"],
)


def test_a_c_command_is_the_template_plus_the_library_part():
    f = g.c_encode(C_CMD, CTMPL, "amd64")
    assert f.includes == ("@build/base/m", "base/m", "@std")
    assert f.flags == ("-std=gnu99", "@core", "-Wno-x")
    assert f.defines == ("UNICODE",)
    assert g.c_decode(f, CTMPL, "amd64") == C_CMD.split()[: C_CMD.split().index("-o")]


def test_a_c_command_out_of_rule_order_is_refused():
    import pytest

    with pytest.raises(g.GenError, match="not sorted"):
        g.c_encode(
            C_CMD.replace("-DUNICODE -D__REACTOS__", "-D__REACTOS__ -DUNICODE"), CTMPL, "amd64"
        )


def test_own_c_moves_a_shims_library_into_the_cell(tmp_path, monkeypatch):
    """The OBJECT library leaves cmake; what it linked for usage requirements, what it was
    ordered after and its generated sources order the cell."""
    cml = tmp_path / "base/m/CMakeLists.txt"
    cml.parent.mkdir(parents=True)
    cml.write_text(
        "# The C still compiled, in an OBJECT library whose compile the cargo link replays.\n"
        "add_library(m_shims OBJECT shim.c ${CMAKE_CURRENT_BINARY_DIR}/m_s.c)\n"
        "target_link_libraries(m_shims PRIVATE wine libkernel32)\n"
        "add_dependencies(m_shims m_idl)\n"
        "add_cargo_module(m m-cargo m.exe\n    KIND win32exe\n    EXTRA_DEPENDS m_shims psdk)\n"
    )
    rs = tmp_path / "sdk/lib/rust/m-cargo/build.rs"
    rs.parent.mkdir(parents=True)
    rs.write_text(
        'fn main() {\n    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("x");\n'
        '    Linker::new(t)\n        .c_from_compile_db("m_shims", &["base/m/shim.c", '
        '"base/m/m_s.c"])\n        .link(k);\n}\n'
    )
    srcs = {a: ["base/m/shim.c", "@build/base/m/m_s.c"] for a in g.ARCHES}
    monkeypatch.setattr(g, "_cmakelists", lambda repo: [cml])
    monkeypatch.setattr(
        g,
        "c_lib_for",
        lambda t, repo=g.REPO: (srcs, {a: g.c_encode(C_CMD, CTMPL, "amd64") for a in g.ARCHES}),
    )
    assert g.own_c(None, tmp_path) == ["m-cargo"]
    out = cml.read_text()
    assert "m_shims" not in out
    assert "EXTRA_DEPENDS psdk wine libkernel32 m_idl ${REACTOS_BINARY_DIR}/base/m/m_s.c)" in out
    text = rs.read_text()
    assert '.c_compiled("m_shims", c_sources, c_includes, c_flags, c_defines)' in text
    assert '"@build/base/m/m_s.c"' in text


ASM_CMD = (
    "/tmp/vibeos/.build-tools/llvm-mingw/bin/x86_64-w64-mingw32-clang -x assembler-with-cpp "
    "-o sdk/lib/x/CMakeFiles/x.dir/a.s.obj -I/tmp/vibeos/sdk/include/asm "
    "-I/tmp/vibeos/build-amd64/sdk/include/asm -U_X86_ -pipe -O1 "
    "-DDBG=1 -D_M_AMD64 -DCRTDLL -D__CRT__NO_INLINE -D__ASM__ -c /tmp/vibeos/sdk/lib/x/a.s"
)
ATMPL = g.CTemplate(
    cc="{REPO}/.build-tools/llvm-mingw/bin/x86_64-w64-mingw32-clang",
    target="",
    std=["sdk/include/asm", "@build/sdk/include/asm"],
    core=["-pipe", "-O1"],
    defines=["-DDBG=1"],
)


def test_an_asm_command_keeps_the_defines_a_source_adds_after_cmakes_sorted_run():
    """libcntpr's asm adds CRTDLL/__CRT__NO_INLINE after the sorted <DEFINES>; the rule ends
    -D__ASM__ -c <src>."""
    f = g.asm_encode(ASM_CMD, ATMPL, "amd64")
    assert f.includes == ("@std",)
    assert f.flags == ("-U_X86_", "@core", "@defines", "-DCRTDLL", "-D__CRT__NO_INLINE")
    assert f.defines == ("_M_AMD64",)
    obj = "sdk/lib/x/CMakeFiles/x.dir/a.s.obj"
    assert g.asm_decode(f, ATMPL, "amd64", obj) == ASM_CMD.split()[:-2]
