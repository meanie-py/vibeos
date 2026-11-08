import json
import struct
import subprocess
import sys
from pathlib import Path

import pytest

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


def test_a_registry_script_is_a_resource_object_the_cell_links(tmp_path):
    """sti: add_idl_reg_scripts attaches sti_wia_r.o (its REGINST resource) to the target."""
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE + "add_idl_reg_scripts(m registry m_wia.idl)\n")
    facts = g.cmake_facts(f, "m")
    assert facts["unknown"] == []
    assert facts["reg_objects"] == ["m_wia_r.o"]


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


def test_a_custom_command_without_rustflags_is_its_body():
    # 38 cohab DLLs (version, advapi32, user32, ...) pass no RUSTFLAGS= at all; the
    # matcher used to require one and refused every one of them.
    text = DRIVER.replace("        RUSTFLAGS=${D_RUST_RUSTFLAGS}\n", "")
    assert "RUSTFLAGS=" not in text
    body = g._custom_body(text)
    assert body is not None and body["crate"] == "d-rust"
    assert body["rustflags"] == {} and body["custom_target"] == "rust_d"
    assert "D_RUST_RUSTFLAGS" not in body["vars"]


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


def test_a_custom_body_another_target_still_reads_keeps_its_command(tmp_path):
    # ws2help: its archive also feeds the INTERFACE ws2help_static that other modules link.
    f = tmp_path / "CMakeLists.txt"
    text = (
        DRIVER
        + "add_library(d_static INTERFACE)\n"
        + "target_link_libraries(d_static INTERFACE ${D_RUST_LIB})\n"
    )
    f.write_text(text)
    c = cell(
        target="d",
        cmake_dir=tmp_path,
        output_name="d.sys",
        module_kind="KernelDll",
        kernel_kind="kernelmodedriver",
        custom_body=g._custom_body(text),
        body_crate="d-rust",
        destinations=["reactos/system32/drivers"],
        no_cab=True,
        static_libs={a: [] for a in g.ARCHES},
        msvcrtex=None,
        rc=None,
    )
    out = g.rewrite_cmake(f, c)
    assert "cargo build" in out and "add_custom_target(rust_d DEPENDS ${D_RUST_LIB})" in out
    assert "target_link_libraries(d_static INTERFACE ${D_RUST_LIB})" in out
    assert "add_cargo_module(d d-cargo" in out


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


class _Cell:
    """The two Cell fields convert() reads before apply()."""

    def __init__(self, target, cmake_dir):
        self.target, self.cmake_dir = target, cmake_dir


@pytest.mark.parametrize("proof_code", [0, 1])
def test_convert_runs_the_wave_in_order(tmp_path, monkeypatch, proof_code):
    steps = []
    monkeypatch.setattr(g, "model", lambda t, repo: _Cell(t, tmp_path))
    monkeypatch.setattr(g, "rewrite_cmake", lambda f, c: "")
    monkeypatch.setattr(g, "save_baseline", lambda ts, b, repo: steps.append(("baseline", b)))
    monkeypatch.setattr(g, "apply", lambda c, repo: steps.append(("apply", c.target)) or [])
    monkeypatch.setattr(g, "_ninja", lambda a, ts, repo: steps.append(("ninja", a)))
    monkeypatch.setattr(g, "check", lambda ts, b, repo: steps.append(("check", b)) or proof_code)
    monkeypatch.setattr(g, "refresh_manifests", lambda repo: steps.append(("manifests", repo)))
    assert g.convert(["m"], tmp_path / "b", tmp_path) == proof_code
    expected = [
        ("baseline", tmp_path / "b"),
        ("apply", "m"),
        ("ninja", "i386"),
        ("ninja", "amd64"),
        ("ninja", "arm64"),
        ("check", tmp_path / "b"),
    ]
    if proof_code == 0:
        expected.append(("manifests", tmp_path))
    assert steps == expected


def test_convert_fails_loud_when_a_new_cells_own_c_has_no_recipe(tmp_path, monkeypatch, capsys):
    """A cell's `<m>_shims` reaches compile_commands.json only after cmake reconfigures, so
    refresh_manifests' per-arch skip used to drop its recipe silently (normaliz, batch J)."""
    for t in ("m", "n"):
        cell = tmp_path / f"sdk/lib/rust/{t}-cargo"
        cell.mkdir(parents=True)
        cell.joinpath("build.rs").write_text(
            f'.c_from_compile_db("{t}_shims", &["x/rust_cohab_stub.c"])\n'
            '.c_from_compile_db("libcntpr", &["sdk/lib/crt/a.c"])\n'
        )
    for a in g.ARCHES:
        g.clib_recipe_path(a, "n_shims", tmp_path).parent.mkdir(parents=True, exist_ok=True)
        g.clib_recipe_path(a, "n_shims", tmp_path).write_text("# n_shims\n")
    g.clib_recipe_path("arm64", "m_shims", tmp_path).write_text("# m_shims\n")
    monkeypatch.setattr(g, "save_baseline", lambda *a: None)
    monkeypatch.setattr(g, "apply", lambda c, repo: [])
    # convert() dry-runs every cmake rewrite before applying any (ctail wave 4), reading
    # each cell's cmake_dir: model a real cell and stub the rewrite itself.
    monkeypatch.setattr(g, "model", lambda t, repo: _Cell(t, tmp_path))
    monkeypatch.setattr(g, "rewrite_cmake", lambda *a: None)
    monkeypatch.setattr(g, "_ninja", lambda *a: None)
    monkeypatch.setattr(g, "check", lambda *a: 0)
    monkeypatch.setattr(g, "refresh_manifests", lambda repo: None)
    assert g.convert(["m", "n"], tmp_path / "b", tmp_path) == 1
    # libcntpr is a shared library, judged by refresh_manifests' own rule, not here.
    assert g.missing_cell_recipes(["m", "n"], tmp_path) == [
        ("i386", "m_shims"),
        ("amd64", "m_shims"),
    ]
    assert "clibs/i386/m_shims.recipe" in capsys.readouterr().out


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


def test_the_rc_template_file_reads_back(tmp_path):
    path = g.rc_template_path("amd64", tmp_path)
    path.parent.mkdir(parents=True)
    path.write_text(TMPL.text("amd64"))
    assert g.read_rc_template("amd64", tmp_path) == TMPL


def test_rc_object_depends_reads_both_spellings():
    # (the args, as _statements gives them)
    assert g._rc_object_depends("add_rc_deps", "a.rc x.ico") == ["x.ico"]
    raw = 'a.rc PROPERTIES OBJECT_DEPENDS "${B}/t.tlb"'
    assert g._rc_object_depends("set_source_files_properties", raw) == ["${B}/t.tlb"]
    raw = "a.c PROPERTIES COMPILE_FLAGS -O0"
    assert g._rc_object_depends("set_source_files_properties", raw) is None


def test_a_fresh_conversion_reads_its_rc_part_from_the_module_compile(tmp_path):
    """--convert: the .rc compile is the module target's own (its object path is the marker);
    an arch the module is not built on takes another arch's part."""
    cmds = {
        "amd64": [{"file": "/tmp/vibeos/base/m/rsrc.rc", "command": RC_CMD}],
        "i386": [],
        "arm64": [
            {"file": "/tmp/vibeos/base/m/rsrc.rc", "command": RC_CMD.replace("m.dir", "n.dir")}
        ],
    }
    for arch in g.ARCHES:
        template = g.rc_template_path(arch, tmp_path)
        template.parent.mkdir(parents=True, exist_ok=True)
        template.write_text(TMPL.text(arch))
        build = tmp_path / f"build-{arch}"
        build.mkdir()
        (build / "compile_commands.json").write_text(json.dumps(cmds[arch]))
    flags = g.rc_flags_for("base/m/rsrc.rc", "base/m/CMakeFiles/m.dir/rsrc.rc.res", tmp_path)
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


def test_a_source_define_cmake_appends_is_kept_after_the_sorted_run():
    # ole32's stubmanager_seh.c: set_source_files_properties(... COMPILE_DEFINITIONS
    # _USE_NATIVE_SEH) lands after cmake's sorted target defines, not sorted in.
    cmd = C_CMD.replace("-D__REACTOS__ ", "-D__REACTOS__ -DSEH ")
    f = g.c_encode(cmd, CTMPL, "amd64")
    assert f.defines == ("UNICODE", "+SEH")
    assert g.c_decode(f, CTMPL, "amd64") == cmd.split()[: cmd.split().index("-o")]


def test_a_c_command_out_of_rule_order_is_refused():
    import pytest

    # Disorder inside the sorted run itself is not cmake's rule (here the global define it
    # moved no longer sits in the sorted run, so the template no longer explains it).
    with pytest.raises(g.GenError, match=r"not sorted|lacks the global define"):
        g.c_encode(
            C_CMD.replace("-DDBG=1 -DUNICODE -D__REACTOS__", "-DUNICODE -DDBG=1 -D__REACTOS__"),
            CTMPL,
            "amd64",
        )


def test_own_c_moves_a_shims_library_into_the_cell(tmp_path):
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
    for arch in g.ARCHES:
        template = g.c_template_path(arch, tmp_path)
        template.parent.mkdir(parents=True, exist_ok=True)
        template.write_text(CTMPL.text(arch))
        build = tmp_path / f"build-{arch}"
        build.mkdir()
        cmd = C_CMD.replace("build-amd64", f"build-{arch}")
        generated = f"/tmp/vibeos/build-{arch}/base/m/m_s.c"
        commands = [
            {"file": "/tmp/vibeos/base/m/shim.c", "command": cmd},
            {
                "file": generated,
                "command": cmd.replace("/tmp/vibeos/base/m/shim.c", generated).replace(
                    "shim.c.obj", "m_s.c.obj"
                ),
            },
        ]
        (build / "compile_commands.json").write_text(json.dumps(commands))
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


def test_spec_entries_that_reexport_an_import_are_recognized(tmp_path):
    """imagehlp re-exports dbghelp: `-import` entries and an entry naming itself as its target
    are exports bound to import thunks on purpose, not the user32_vista accident."""
    spec = tmp_path / "imagehlp.spec"
    spec.write_text(
        "@ stdcall -import EnumerateLoadedModules(long ptr ptr)\n"
        "@ stdcall -import -arch=win32 MapDebugInformation(long str str long)\n"
        "@ stdcall -import ImageNtHeader(ptr) RtlImageNtHeader\n"
        "@ stdcall SymEnumSourceFiles(ptr int64 str ptr ptr) SymEnumSourceFiles\n"
        "@ stdcall BindImage(str str str)\n"
        "@ stdcall Alias(long) Target\n"
    )
    assert g._spec_imported_exports(spec) == {
        "EnumerateLoadedModules",
        "MapDebugInformation",
        "ImageNtHeader",
        "SymEnumSourceFiles",
    }
    assert g._spec_imported_exports(tmp_path / "missing.spec") == set()


def test_a_c_module_with_no_rust_body_is_a_cell_with_no_rust_archive(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(
        "add_executable(m main.c rsrc.rc)\n"
        "set_module_type(m win32gui UNICODE)\n"
        "add_importlibs(m user32 msvcrt kernel32 ntdll)\n"
        "add_cd_file(TARGET m DESTINATION reactos/system32 FOR all)\n"
    )
    c = cell(body_crate="", c_shims=["base/m/main.c"], static_libs={a: [] for a in g.ARCHES})
    rs = g.build_rs(c)
    assert "rust_staticlib" not in rs and "rust_lib(" not in rs
    assert "there is no Rust body" in rs
    assert '"m_shims"' in rs
    out = g.rewrite_cmake(f, c)
    assert "BODY_CRATE" not in out
    assert "is C, and cargo compiles and links it" in out
    assert "add_library(m_shims OBJECT main.c)" in out


def test_a_shell_less_cell_takes_nothing_from_rustcs_bin():
    """A forwarder or resource DLL (rsabase, shdoclc) has no code: the bin's panic
    handler and i386's rsbegin gave it a .text and an ntdll import cmake's had not."""
    c = cell(
        body_crate="",
        msvcrtex=None,
        shell_less=True,
        output_name="m.dll",
        module_kind="Win32Dll",
        entries={a: "__ImageBase" for a in g.ARCHES},
        static_libs={a: [] for a in g.ARCHES},
    )
    assert ".shell_less()" in g.build_rs(c)
    assert "panic_lean::panic_fastfail()" in g.main_rs(c)
    toml = g.cargo_toml(c)
    assert "panic-lean" in toml and "panic-common" not in toml
    assert ".shell_less()" not in g.build_rs(
        cell(body_crate="", static_libs={a: [] for a in g.ARCHES})
    )


# ---------------------------------------------------------------- the build-graph lookup


def _fake_build(root: Path, arch: str, ninja: str, installs: dict[str, str] | None = None):
    """A build-<arch> with the given build.ninja and per-directory cmake_install.cmake."""
    b = root / f"build-{arch}"
    b.mkdir(parents=True, exist_ok=True)
    (b / "build.ninja").write_text(ninja)
    (b / "CMakeCache.txt").write_text("CMAKE_HOME_DIRECTORY:INTERNAL=/tmp/vibeos\n")
    for outdir, src in (installs or {}).items():
        (b / outdir).mkdir(parents=True, exist_ok=True)
        (b / outdir / "cmake_install.cmake").write_text(
            f"# Install script for directory: /tmp/vibeos/{src}\n\nset(x 1)\n"
        )


def _edge(out: str, rule: str, target: str, extra: str = "x.obj") -> str:
    return (
        f"build {out}: {rule}__{target}_Debug {extra}\n"
        "  LINK_FLAGS = -Wl,--subsystem,windows:5.01\n"
        "  LINK_LIBRARIES = a.a\n\n"
    )


def test_a_cxx_link_is_the_same_edge_as_a_c_link(tmp_path):
    """Any .cpp in a module makes cmake link it with CXX_*_LINKER, which the C-only pattern
    missed: 40 modules were refused as 'a static lib?' for that alone."""
    for rule, want in (
        ("C_EXECUTABLE_LINKER", "C_EXECUTABLE"),
        ("CXX_EXECUTABLE_LINKER", "C_EXECUTABLE"),
        ("CXX_MODULE_LIBRARY_LINKER", "C_MODULE_LIBRARY"),
        ("CXX_SHARED_LIBRARY_LINKER", "C_SHARED_LIBRARY"),
    ):
        _fake_build(tmp_path, "amd64", _edge("base/m/m.exe", rule, "m"))
        link = g.link_edge("amd64", "m", tmp_path)
        assert link.kind == want and link.output == "base/m/m.exe"
        g._read.cache_clear()


def test_the_link_edge_name_is_anchored_by_the_config(tmp_path):
    _fake_build(
        tmp_path,
        "amd64",
        _edge("base/m/m_c.exe", "C_EXECUTABLE_LINKER", "m_c")
        + "build cmake_object_order_depends_target_m: phony\n\n",
    )
    with pytest.raises(g.GenError, match="no link edge"):
        g.link_edge("amd64", "m", tmp_path)
    g._read.cache_clear()


def test_a_missing_link_edge_says_what_is_there_instead(tmp_path):
    cell = (
        "build dll/win32/a/a.dll | ${cmake_ninja_workdir}dll/win32/a/a.dll: CUSTOM_COMMAND "
        "/tmp/vibeos/sdk/lib/rust/a-cargo/Cargo.toml /tmp/vibeos/x\n  COMMAND = c\n\n"
    )
    gated = "build cmake_object_order_depends_target_other: phony\n\n"
    for t, text, why in (("a", cell, "cell"), ("w", gated, "absent")):
        _fake_build(tmp_path, "i386", text)
        with pytest.raises(g.NoLinkEdge) as e:
            g.link_edge("i386", t, tmp_path)
        assert e.value.why == why
        g._read.cache_clear()
    # A target that exists but has no edge of any kind stays the original message.
    _fake_build(tmp_path, "i386", "build cmake_object_order_depends_target_u: phony\n\n")
    with pytest.raises(g.NoLinkEdge) as e:
        g.link_edge("i386", "u", tmp_path)
    assert e.value.why == "unknown"
    g._read.cache_clear()


def test_a_module_missing_on_some_arches_is_refused_by_arch(tmp_path):
    """wow64 exists on amd64/arm64 only: the refusal names i386, it is not 'a static lib'."""
    here = _edge("dll/win32/w/w.dll", "C_MODULE_LIBRARY_LINKER", "w")
    for a in ("amd64", "arm64"):
        _fake_build(tmp_path, a, here)
    _fake_build(tmp_path, "i386", "build cmake_object_order_depends_target_z: phony\n\n")
    want = r"w: i386: not defined on i386.*cmake links it on \['amd64', 'arm64'\]"
    with pytest.raises(g.GenError, match=want):
        g._link_edges("w", tmp_path)
    g._read.cache_clear()


def test_a_module_that_is_a_cell_on_every_arch_is_not_converted_again(tmp_path):
    cell = (
        "build dll/win32/a/a.dll | ${cmake_ninja_workdir}dll/win32/a/a.dll: CUSTOM_COMMAND "
        "/tmp/vibeos/sdk/lib/rust/a-cargo/Cargo.toml\n  COMMAND = c\n\n"
    )
    for a in g.ARCHES:
        _fake_build(tmp_path, a, cell)
    with pytest.raises(g.GenError, match="already a cargo cell on every arch"):
        g._link_edges("a", tmp_path)
    g._read.cache_clear()


def test_a_cell_on_some_arches_only_is_a_partial_conversion(tmp_path):
    cell = (
        "build d/a.dll | ${w}d/a.dll: CUSTOM_COMMAND "
        "/tmp/vibeos/sdk/lib/rust/a-cargo/Cargo.toml\n  C = c\n\n"
    )
    _fake_build(tmp_path, "i386", cell)
    for a in ("amd64", "arm64"):
        _fake_build(tmp_path, a, _edge("d/a.dll", "C_MODULE_LIBRARY_LINKER", "a"))
    with pytest.raises(g.GenError, match="a cargo cell on \\['i386'\\] but still cmake-linked"):
        g._link_edges("a", tmp_path)
    g._read.cache_clear()


def test_the_source_dir_is_where_the_build_says_not_where_it_wrote_the_output(tmp_path):
    """vendor/ac97 builds into drivers/wdm/audio/drivers/ac97 (add_subdirectory with a
    binary dir) and dll/3rdparty/libpng is vendor/libpng: the output dir has no CMakeLists."""
    _fake_build(tmp_path, "amd64", "", {"drivers/wdm/audio/drivers/ac97": "vendor/ac97"})
    got = g.source_dir("amd64", "drivers/wdm/audio/drivers/ac97/ac97.sys", "ac97", tmp_path)
    assert got == tmp_path / "vendor/ac97"
    # With no cmake_install.cmake to read, the output dir is the best guess.
    assert g.source_dir("amd64", "x/y/y.dll", "y", tmp_path) == tmp_path / "x/y"
    g._read.cache_clear()


def test_a_cpp_placeholder_is_dropped_and_real_cpp_is_not(tmp_path):
    stub = tmp_path / "rust_cohab_stub.cpp"
    stub.write_text("/* placeholder */\ntypedef int x_t;\n")
    assert g._placeholder(stub)
    anchored = tmp_path / "anchored.cpp"
    anchored.write_text('extern char a __asm__("f");\nstatic const void *const t[] = {&a};\n')
    assert not g._placeholder(anchored)


def test_cpp_properties_are_inert_once_no_cpp_is_compiled(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE + "set_target_cpp_properties(m WITH_EXCEPTIONS)\n")
    assert g.cmake_facts(f, "m")["unknown"] == []
    assert "set_target_cpp_properties" not in g.rewrite_cmake(f, cell(cmake_dir=tmp_path))


def test_the_dll_variant_is_the_body_when_the_module_also_has_a_static_one(tmp_path):
    """psapi: psapi_rust belongs to psapi_static (kernel32 links it), psapi_dll_rust to
    the DLL. The name rule picked the first and then the i386 link had no libpsapi_rust.a."""
    f = tmp_path / "CMakeLists.txt"
    f.write_text(
        'rust_staticlib_variant(m_rust m-rust "" TRUE)\n'
        'rust_staticlib_variant(m_dll_rust m-rust "dllmain" TRUE)\n'
        "add_library(m MODULE ${SOURCE})\n"
        "target_link_libraries(m ${m_dll_rust_LIB})\n"
    )
    facts = g.cmake_facts(f, "m", ["base/m/libm_dll_rust.a"])
    assert facts["body_var"] == "m_dll_rust" and facts["body"] == ("m-rust", ["dllmain"], True)
    # Linking neither, the `<target>_rust` name still decides.
    assert g.cmake_facts(f, "m", [])["body_var"] == "m_rust"


def test_only_an_export_that_is_an_import_thunk_counts_as_bound(tmp_path):
    """evr and mf export names their delay-import lib also carries (their own stubs, and
    forwarders to the DLL it imports), which is not a rebound export: only a thunk is."""
    from test_pe_equiv import make_pe

    def with_import(code):
        data = bytearray(make_pe(["Alpha", "Beta"], code=code))
        # Actual delay import, so the parser and thunk check both run.
        struct.pack_into("<II", data, 0x40 + 24 + 112 + 13 * 8, 0x1100, 64)
        struct.pack_into("<IIIIIIII", data, 0x500, 1, 0x1180, 0, 0, 0x11A0, 0, 0, 0)
        data[0x580:0x58B] = b"mfplat.dll\0"
        struct.pack_into("<QQ", data, 0x5A0, 0x11C0, 0)
        data[0x5C0:0x5C8] = b"\0\0Alpha\0"
        return data

    thunk = tmp_path / "thunk.dll"
    thunk.write_bytes(with_import(b"\xff\x25\0\0\0\0"))
    real = tmp_path / "real.dll"
    real.write_bytes(with_import(b"\x55\x48\x89\xe5\xc3"))
    assert g._exports_bound_to_imports(thunk) == {"Alpha"}  # Beta is not imported
    assert g._exports_bound_to_imports(real) == set()
    # ARM64's import thunk opens with `adrp x16, ...`.
    assert g._export_is_import_thunk((0x90000010).to_bytes(4, "little"), 0xAA64)
    assert not g._export_is_import_thunk((0xD65F03C0).to_bytes(4, "little"), 0xAA64)


# ---- statements that shape the link (set_subsystem, set_entrypoint, ...) ----


def _link_edge_with(arch, **kw):
    base = dict(
        arch=arch,
        output="dll/m.dll",
        kind="C_MODULE_LIBRARY",
        entry="",
        subsystem=None,
        image_base=None,
        stack=None,
        inputs=[],
        libs=[],
    )
    base.update(kw)
    return g.Link(**base)


def test_set_subsystem_is_checked_against_every_link_edge(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE + "set_subsystem(m console)\n")
    facts = g.cmake_facts(f, "m")
    assert facts["unknown"] == []
    assert facts["set_subsystem"] == "console:5.01"
    ok = {a: _link_edge_with(a, subsystem="console:5.01") for a in g.ARCHES}
    g._check_statement_effects("m", facts, ok)
    bad = dict(ok, arm64=_link_edge_with("arm64", subsystem="windows:5.01"))
    with pytest.raises(g.GenError, match=r"set_subsystem says console:5\.01 but the arm64"):
        g._check_statement_effects("m", facts, bad)
    # lld takes no numeric EFI id, so gcc.cmake maps it.
    assert g._subsystem_flag("10") == "efi_application:5.01"


def test_set_entrypoint_is_rebuilt_per_arch_and_checked(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE + "set_entrypoint(m DllMain 12)\n")
    facts = g.cmake_facts(f, "m")
    assert facts["unknown"] == [] and facts["set_entrypoint"] == ("DllMain", "12")
    assert g._entry_symbol("i386", "DllMain", "12") == "_DllMain@12"
    assert g._entry_symbol("amd64", "DllMain", "12") == "DllMain"
    assert g._entry_symbol("i386", "0", None) == "___ImageBase"
    assert g._entry_symbol("arm64", "0", None) == "__ImageBase"
    edges = {
        "i386": _link_edge_with("i386", entry="_DllMain@12"),
        "amd64": _link_edge_with("amd64", entry="DllMain"),
        "arm64": _link_edge_with("arm64", entry="DllMain"),
    }
    g._check_statement_effects("m", facts, edges)
    # A guard that is false in this build leaves the other entry on the edge.
    edges["amd64"] = _link_edge_with("amd64", entry="DllMainCRTStartup")
    with pytest.raises(g.GenError, match="set_entrypoint"):
        g._check_statement_effects("m", facts, edges)


def test_rewrite_drops_the_subsystem_and_entry_statements_with_their_guard(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(
        CMAKE
        + "if(DLL_EXPORT_VERSION LESS 0x600)\n    set_entrypoint(m DllMain 12)\nendif()\n"
        + "set_subsystem(m console)\n"
    )
    out = g.rewrite_cmake(f, cell())
    assert "set_entrypoint" not in out and "set_subsystem" not in out
    assert "DLL_EXPORT_VERSION" not in out


def test_link_strict_and_the_whole_archive_cohab_are_refused_with_the_reason(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE + "link_strict(m)\nlink_msvcrt_rust_cohab(m)\n")
    unknown = g.cmake_facts(f, "m")["unknown"]
    assert len(unknown) == 2
    assert "--allow-multiple-definition" in unknown[0]
    assert "whole-archive" in unknown[1]


def test_linker_language_is_not_a_cell_property(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE + "set_target_properties(m PROPERTIES LINKER_LANGUAGE C)\n")
    assert g.cmake_facts(f, "m")["unknown"] == []
    assert "LINKER_LANGUAGE" not in g.rewrite_cmake(f, cell())


REQUIRE = (
    'if(ARCH STREQUAL "i386")\n'
    '    target_link_options(m PRIVATE "LINKER:--require-defined=_A@4,--require-defined=_B@8")\n'
    "else()\n"
    '    target_link_options(m PRIVATE "LINKER:--require-defined=A,--require-defined=B")\n'
    "endif()\n"
)


def test_require_defined_comes_from_each_link_edge_into_link_args(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE + REQUIRE)
    facts = g.cmake_facts(f, "m")
    assert facts["unknown"] == [] and facts["require_defined"]
    edges = {a: _link_edge_with(a, require_defined=("A", "B")) for a in g.ARCHES}
    edges["i386"] = _link_edge_with("i386", require_defined=("_A@4", "_B@8"))
    g._check_statement_effects("m", facts, edges)
    edges["arm64"] = _link_edge_with("arm64")
    with pytest.raises(g.GenError, match="--require-defined is on no link edge"):
        g._check_statement_effects("m", facts, edges)
    out = g.rewrite_cmake(f, cell())
    assert "require-defined" not in out and "ARCH STREQUAL" not in out
    rs = g.build_rs(
        cell(
            require_defined={
                "i386": ["_A@4", "_B@8"],
                "amd64": ["A", "B"],
                "arm64": ["A", "B"],
            }
        )
    )
    assert '"x86" => &["-Wl,--require-defined=_A@4", "-Wl,--require-defined=_B@8"],' in rs
    assert '"x86_64" => &["-Wl,--require-defined=A", "-Wl,--require-defined=B"],' in rs
    assert ".link_args(require_defined)" in rs
    assert "require_defined" not in g.build_rs(cell())


def test_link_edge_reads_the_require_defined_operands(tmp_path):
    for a in g.ARCHES:
        d = tmp_path / f"build-{a}"
        d.mkdir()
        (d / "build.ninja").write_text(
            "build dll/m.dll: C_SHARED_LIBRARY_LINKER__m_Release a.obj\n"
            "  LINK_FLAGS = -Wl,--require-defined=_X@4,--require-defined=_Y@8"
            " -Wl,--subsystem,console:5.01\n"
        )
    assert g.link_edge("i386", "m", tmp_path).require_defined == ("_X@4", "_Y@8")


def test_kmt_native_seh_moves_to_the_object_library(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE + "kmt_enable_native_seh(m)\n")
    assert g.cmake_facts(f, "m")["unknown"] == []
    out = g.rewrite_cmake(f, cell(c_shims=["base/m/x.c"]))
    assert "kmt_enable_native_seh(m_shims)" in out and "kmt_enable_native_seh(m)" not in out


DBGHELP = """\
if(NOT CMAKE_CROSSCOMPILING)
    list(APPEND SOURCE host.c)
    add_library(m_host ${SOURCE})
else()
    list(APPEND SOURCE a.c)
    # main.c  # replaced by Rust (sdk/lib/rust/m-rust/src/m/main.rs)
    rust_staticlib_variant(m_rust m-rust "" TRUE)
    add_library(m MODULE ${SOURCE})
    set_module_type(m win32dll)
    add_importlibs(m kernel32)
endif()
"""


def test_a_library_in_the_other_branch_does_not_share_the_source_list(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(DBGHELP)
    # dbghelp's real shape: the module is in the else() of the host tool's if().
    assert g.cmake_facts(f, "m")["unknown"] == []
    # The rewrite only wraps a link in a plain if(): swap the branches to rewrite it.
    host, module = DBGHELP.split("else()\n")
    host = host.replace("if(NOT CMAKE_CROSSCOMPILING)\n", "")
    module = module.replace("endif()\n", "")
    f.write_text(f"if(CMAKE_CROSSCOMPILING)\n{module}else()\n{host}endif()\n")
    out = g.rewrite_cmake(f, cell(target="m", output_name="m.dll", module_kind="Win32Dll"))
    # The host tool keeps its own list and library; the module's list goes with its link.
    assert "list(APPEND SOURCE host.c)" in out and "add_library(m_host ${SOURCE})" in out
    assert "list(APPEND SOURCE a.c)" not in out


def test_a_library_in_the_targets_own_branch_still_shares_it(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(DBGHELP.replace("else()", "endif()\nif(TRUE)", 1))
    assert g.cmake_facts(f, "m")["unknown"]


def test_a_target_made_by_a_shared_function_names_the_function(tmp_path):
    (tmp_path / "t.cmake").write_text("function(add_t_target VERSION)\nendfunction()\n")
    f = tmp_path / "CMakeLists.txt"
    f.write_text("include(${CMAKE_CURRENT_SOURCE_DIR}/t.cmake)\nadd_t_target(24)\n")
    unknown = g.cmake_facts(f, "m_24")["unknown"]
    assert "target made in a loop or function" in unknown
    assert any("add_t_target(24)" in u for u in unknown)
    f.write_text("add_bootvid(m x.cmake)\n")
    assert any("add_bootvid(m x.cmake)" in u for u in g.cmake_facts(f, "m")["unknown"])


def test_a_bodyless_cell_is_recognised_by_its_build_script(tmp_path):
    crate = tmp_path / "sdk/lib/rust/m-cargo"
    crate.mkdir(parents=True)
    assert not g._cell_is_bodyless("m", tmp_path)
    (crate / "build.rs").write_text(g.build_rs(cell(body_crate="", c_shims=["base/m/main.c"])))
    assert g._cell_is_bodyless("m", tmp_path)
    (crate / "build.rs").write_text(g.build_rs(cell()))
    assert not g._cell_is_bodyless("m", tmp_path)


def test_a_rerun_keeps_the_first_runs_baseline(tmp_path, monkeypatch):
    """After a failed build, build-<arch> may hold cell-linked PEs: re-baselining from it
    would compare the cells with themselves."""
    saved = []
    monkeypatch.setattr(g, "model", lambda t, repo: _Cell(t, tmp_path))
    monkeypatch.setattr(g, "rewrite_cmake", lambda f, c: "")
    monkeypatch.setattr(g, "save_baseline", lambda ts, b, repo: saved.append(b))
    monkeypatch.setattr(g, "apply", lambda c, repo: [])
    monkeypatch.setattr(g, "_ninja", lambda a, ts, repo: None)
    monkeypatch.setattr(g, "check", lambda ts, b, repo: 0)
    monkeypatch.setattr(g, "refresh_manifests", lambda repo: None)
    b = tmp_path / "b"
    for a in g.ARCHES:
        (b / a).mkdir(parents=True)
    (b / "targets.json").write_text('{"m": {}}')
    assert g.convert(["m"], b, tmp_path) == 0 and saved == []
    with pytest.raises(g.GenError, match="no baseline for"):
        g.convert(["m", "n"], b, tmp_path)


def test_a_failed_build_names_the_failed_edge_not_the_lock_noise(tmp_path):
    out = "\n".join(
        ["[1/9] cargo"] * 3
        + ["FAILED: drivers/base/kdstub/kdstub.dll", "error[E0425]: cannot find value"]
        + ["    Blocking waiting for file lock on build directory"] * 20
    )
    script = tmp_path / "tools/canon-run.sh"
    script.parent.mkdir()
    # Exercise the native subprocess capture with a deterministic failing build tool.
    import shlex

    script.write_text("printf '%s\\n' " + shlex.quote(out) + "\nexit 1\n")
    (tmp_path / "build-i386").mkdir()
    with pytest.raises(g.GenError) as e:
        g._ninja("i386", ["kdstub"], tmp_path)
    assert "FAILED: drivers/base/kdstub/kdstub.dll" in str(e.value)
    assert "error[E0425]" in str(e.value)


def test_a_wave_build_keeps_going_and_names_every_failed_cell(tmp_path):
    # An 85-module wave stopped at its first two failures; the rest went unbuilt.
    script = tmp_path / "tools/canon-run.sh"
    script.parent.mkdir()
    log = tmp_path / "args"
    failed = "".join(f"FAILED: m{i}.dll\\n" for i in range(25))
    script.write_text(
        f'echo "$*" >> {log}\n[ "$2" = build.ninja ] && exit 0\nprintf \'{failed}\'\nexit 1\n'
    )
    (tmp_path / "build-i386").mkdir()
    with pytest.raises(g.GenError) as e:
        g._ninja("i386", ["a", "b"], tmp_path)
    assert log.read_text().splitlines() == ["ninja build.ninja", "ninja -k 0 a b"]
    assert all(f"FAILED: m{i}.dll" in str(e.value) for i in range(25))


def test_a_rewrite_refusal_writes_no_cell(tmp_path, monkeypatch):
    """One target's CMakeLists refusing must stop the wave before any cell is written."""
    applied = []

    class C:
        def __init__(self, t):
            self.target, self.cmake_dir = t, tmp_path

    def rewrite(f, c):
        if c.target == "bad":
            raise g.GenError("bad: its link is in an elseif/else branch")
        return ""

    monkeypatch.setattr(g, "model", lambda t, repo: C(t))
    monkeypatch.setattr(g, "rewrite_cmake", rewrite)
    monkeypatch.setattr(g, "save_baseline", lambda ts, b, repo: None)
    monkeypatch.setattr(g, "apply", lambda c, repo: applied.append(c.target) or [])
    with pytest.raises(g.GenError, match="elseif"):
        g.convert(["good", "bad"], tmp_path / "b", tmp_path)
    assert applied == []


def test_a_target_another_cmakelists_reads_by_genex_is_found(tmp_path):
    (tmp_path / "a").mkdir()
    (tmp_path / "b").mkdir()
    (tmp_path / "a/CMakeLists.txt").write_text("add_executable(filebrowser main.c)\n")
    (tmp_path / "b/CMakeLists.txt").write_text(
        "add_custom_command(TARGET rshell POST_BUILD\n"
        '  COMMAND x "$<TARGET_FILE_DIR:filebrowser>/y")\n'
        # A target's own POST_BUILD on itself is not a consumer (legide-rust's cohab copy).
        "add_custom_command(TARGET legide POST_BUILD\n"
        "  COMMAND native-pefixup --section:.rsrc,!D $<TARGET_FILE:legide>)\n"
    )
    g._GENEX_REFS.pop(tmp_path, None)
    assert g._target_file_refs(tmp_path) == {"filebrowser": ["b/CMakeLists.txt"]}


def test_a_conditional_compile_setting_keeps_its_condition(tmp_path):
    """libxslt's MSVC-only /wd4267 was moved out of its if(MSVC) onto the OBJECT library
    unconditionally and broke clang; it must keep the if()s it sat under."""
    f = tmp_path / "CMakeLists.txt"
    body = (
        "add_executable(m main.c rsrc.rc)\n"
        "set_module_type(m win32gui UNICODE)\n"
        "if(MSVC)\n"
        '    if(ARCH STREQUAL "amd64")\n'
        "        target_compile_options(m PRIVATE /wd4267)\n"
        "    endif()\n"
        "endif()\n"
        "add_cd_file(TARGET m DESTINATION reactos/system32 FOR all)\n"
    )
    f.write_text(body)
    c = cell(body_crate="", c_shims=["base/m/main.c"], static_libs={a: [] for a in g.ARCHES})
    out = g.rewrite_cmake(f, c)
    i = out.index("target_compile_options(m_shims PRIVATE /wd4267)")
    assert out.rfind("if(MSVC)", 0, i) != -1
    assert out.rfind('if(ARCH STREQUAL "amd64")', 0, i) > out.rfind("if(MSVC)", 0, i)
    f.write_text(body.replace('    if(ARCH STREQUAL "amd64")\n', "else()\n    if(1)\n"))
    with pytest.raises(g.GenError, match="elseif/else branch"):
        g.rewrite_cmake(f, c)


def test_a_comment_inside_a_moved_statement_is_not_carried_as_a_dead_source(tmp_path):
    """d3dx9's `# math.c wants M_PI` sits inside its compile definitions; the statement moves
    to the OBJECT library with it, so it must not also be listed as translated C on disk."""
    f = tmp_path / "CMakeLists.txt"
    f.write_text(
        "add_library(m MODULE main.c)\n"
        "set_module_type(m win32dll)\n"
        "target_compile_definitions(m PRIVATE\n"
        "    # math.c wants M_PI.\n"
        "    _USE_MATH_DEFINES)\n"
        "add_cd_file(TARGET m DESTINATION reactos/system32 FOR all)\n"
    )
    c = cell(body_crate="", c_shims=["base/m/main.c"], static_libs={a: [] for a in g.ARCHES})
    out = g.rewrite_cmake(f, c)
    assert out.count("# math.c wants M_PI.") == 1
    assert "kept on disk" not in out


def test_a_list_variable_of_import_libs_is_not_prefixed_as_one_lib(tmp_path):
    """d3dx10's forwarders import `${_forward_targets}`, a list; `lib${_forward_targets}`
    prefixed only its first element and left the rest naming the (now cargo) modules."""
    f = tmp_path / "CMakeLists.txt"
    f.write_text(
        "add_library(m MODULE main.c)\n"
        "set_module_type(m win32dll)\n"
        "add_importlibs(m ${_fwd} msvcrt kernel32)\n"
        "add_cd_file(TARGET m DESTINATION reactos/system32 FOR all)\n"
    )
    c = cell(body_crate="", c_shims=["base/m/main.c"], static_libs={a: [] for a in g.ARCHES})
    out = g.rewrite_cmake(f, c)
    assert "lib${_fwd}" not in out
    assert "libmsvcrt libkernel32" in out


def test_a_post_build_pefixup_on_the_target_becomes_the_cells_pefixup(tmp_path):
    """pcix and legide fix .rsrc's flags in a POST_BUILD on the target; left in place it
    names a target the cell removed and the whole tree's configure fails."""
    f = tmp_path / "CMakeLists.txt"
    body = (
        "add_library(m MODULE main.c)\n"
        "set_module_type(m kernelmodedriver)\n"
        "add_custom_command(\n"
        "    TARGET m POST_BUILD\n"
        "    COMMAND native-pefixup --section:.rsrc,!D $<TARGET_FILE:m>\n"
        "    VERBATIM)\n"
        "add_cd_file(TARGET m DESTINATION reactos/system32/drivers FOR all)\n"
    )
    f.write_text(body)
    c = cell(body_crate="", c_shims=["base/m/main.c"], static_libs={a: [] for a in g.ARCHES})
    out = g.rewrite_cmake(f, c)
    assert "TARGET m POST_BUILD" not in out
    assert "    PEFIXUP --section:.rsrc,!D" in out
    f.write_text(body.replace("native-pefixup --section:.rsrc,!D", "strip"))
    with pytest.raises(g.GenError, match="other than one POST_BUILD native-pefixup"):
        g.rewrite_cmake(f, c)


def test_a_linker_flag_is_no_usage_requirement_and_no_reused_archive(tmp_path):
    """cdrom links its Rust body between -Wl,--whole-archive and -Wl,--no-whole-archive; the
    flags reached the OBJECT library's target_link_libraries and the REUSED file list."""
    f = tmp_path / "CMakeLists.txt"
    f.write_text(
        "add_library(m MODULE main.c)\n"
        "set_module_type(m kernelmodedriver)\n"
        "target_link_libraries(m libcntpr\n"
        "    -Wl,--whole-archive ${M_RUST_LIB} -Wl,--no-whole-archive)\n"
        "add_cd_file(TARGET m DESTINATION reactos/system32/drivers FOR all)\n"
    )
    libs = ["-Wl,--whole-archive", "-Wl,--no-whole-archive"]
    c = cell(body_crate="", c_shims=["base/m/main.c"], static_libs={a: libs for a in g.ARCHES})
    out = g.rewrite_cmake(f, c)
    assert "-Wl," not in out


def test_a_conditional_reg_script_is_a_dependency_only_where_it_exists(tmp_path):
    """comdlg32 registers its classes only if(DLL_EXPORT_VERSION GREATER_EQUAL 0x600); with
    the if() false, comdlg32_r does not exist and a plain name is a missing file to ninja."""
    f = tmp_path / "CMakeLists.txt"
    body = (
        "add_library(m MODULE main.c rsrc.rc)\n"
        "set_module_type(m win32dll)\n"
        "if(DLL_EXPORT_VERSION GREATER_EQUAL 0x600)\n"
        "    add_idl_reg_scripts(m registry m_classes.idl)\n"
        "endif()\n"
        "add_cd_file(TARGET m DESTINATION reactos/system32 FOR all)\n"
    )
    f.write_text(body)
    c = cell(body_crate="", c_shims=["base/m/main.c"], static_libs={a: [] for a in g.ARCHES})
    out = g.rewrite_cmake(f, c)
    assert "$<TARGET_NAME_IF_EXISTS:m_r>" in out
    f.write_text(
        body.replace("if(DLL_EXPORT_VERSION GREATER_EQUAL 0x600)\n", "").replace("endif()\n", "")
    )
    out = g.rewrite_cmake(f, c)
    assert "TARGET_NAME_IF_EXISTS" not in out and " m_r" in out


def test_a_moved_statement_ending_in_a_comment_keeps_its_closing_paren(tmp_path):
    """wdf01000's include list ends in `dir # why` before `)`: re-emitted trimmed, the paren
    joined the comment and the whole tree's configure failed."""
    f = tmp_path / "CMakeLists.txt"
    f.write_text(
        "add_library(m MODULE main.c)\n"
        "set_module_type(m win32dll)\n"
        "target_include_directories(m PRIVATE\n"
        "    inc # why\n"
        "    )\n"
        "add_cd_file(TARGET m DESTINATION reactos/system32 FOR all)\n"
    )
    c = cell(body_crate="", c_shims=["base/m/main.c"], static_libs={a: [] for a in g.ARCHES})
    out = g.rewrite_cmake(f, c)
    assert "inc # why\n)" in out


def test_a_sibling_reading_the_targets_compile_properties_reads_the_shims(tmp_path):
    """shdocvw_sublib compiles with $<TARGET_PROPERTY:shdocvw,COMPILE_DEFINITIONS>; the cell's
    custom target has none, so it lost them and failed to compile."""
    f = tmp_path / "CMakeLists.txt"
    f.write_text(
        "add_library(m MODULE main.c $<TARGET_OBJECTS:m_sub>)\n"
        "set_module_type(m win32dll)\n"
        "add_library(m_sub OBJECT sub.cpp)\n"
        "target_compile_definitions(m_sub PRIVATE $<TARGET_PROPERTY:m,COMPILE_DEFINITIONS>)\n"
        "add_cd_file(TARGET m DESTINATION reactos/system32 FOR all)\n"
    )
    c = cell(body_crate="", c_shims=["base/m/main.c"], static_libs={a: [] for a in g.ARCHES})
    out = g.rewrite_cmake(f, c)
    assert "$<TARGET_PROPERTY:m_shims,COMPILE_DEFINITIONS>" in out


def test_the_modules_own_target_file_genex_becomes_the_cells_output(tmp_path):
    """sdk/include/asm runs native-geninc on $<TARGET_FILE:genincdata> in genincdata's own
    file; a cell is a custom target that genex cannot resolve."""
    f = tmp_path / "CMakeLists.txt"
    f.write_text(
        "add_custom_command(OUTPUT x.inc COMMAND gen $<TARGET_FILE:m> x.inc DEPENDS m)\n"
        "add_library(m MODULE main.c)\n"
        "set_module_type(m win32dll)\n"
        "add_cd_file(TARGET m DESTINATION reactos/system32 FOR all)\n"
    )
    c = cell(body_crate="", c_shims=["base/m/main.c"], static_libs={a: [] for a in g.ARCHES})
    out = g.rewrite_cmake(f, c)
    assert "$<TARGET_FILE:m>" not in out
    assert f"${{CMAKE_CURRENT_BINARY_DIR}}/{c.output_name}" in out


def test_an_object_in_a_subdirectory_maps_to_that_subdirectorys_source():
    d = Path("/r/dll/win32/crypt32")
    obj = "dll/win32/crypt32/CMakeFiles/crypt32.dir/reactos/unixlib.c.obj"
    assert g._obj_source(d, obj) == d / "reactos/unixlib.c"
    assert g._obj_source(d, "dll/win32/crypt32/CMakeFiles/crypt32.dir/str.c.obj") == d / "str.c"
    c = cell(cmake_dir=g.REPO / "dll/win32/crypt32", target="crypt32")
    assert g._shim_name("dll/win32/crypt32/reactos/unixlib.c", c) == "reactos/unixlib.c"


def test_a_generated_rc_input_behind_a_list_variable_orders_the_cell():
    text = (
        "list(APPEND m_rc_deps\n"
        "    ${CMAKE_CURRENT_SOURCE_DIR}/m.rgs\n"
        "    ${CMAKE_CURRENT_BINARY_DIR}/m_v1.tlb)\n"
    )
    deps = g._expand(["${m_rc_deps}"], g._list_vars(text))
    assert deps == ["${CMAKE_CURRENT_SOURCE_DIR}/m.rgs", "${CMAKE_CURRENT_BINARY_DIR}/m_v1.tlb"]


@pytest.mark.parametrize("kind", ["FILE", "FILE_DIR", "LINKER_FILE_NAME"])
def test_model_refuses_another_cmake_consumer_before_conversion(tmp_path, kind):
    _ratchet_fixture(tmp_path)
    assert g.model("foo", tmp_path).target == "foo"
    _cml(tmp_path, "consumer", f'add_custom_target(copy COMMAND x "$<TARGET_{kind}:foo>")\n')
    with pytest.raises(g.GenError, match="other CMakeLists read its file"):
        g.model("foo", tmp_path)
    assert (tmp_path / "sdk/lib/rust/foo-cargo/build.rs").read_text() == "fn main() {}\n"


def test_model_and_rewrite_preserve_nested_sources_with_colliding_basenames(tmp_path):
    _ratchet_fixture(tmp_path)
    source = tmp_path / "foo/reactos/unixlib.c"
    source.parent.mkdir()
    source.write_text("int nested(void) { return 7; }\n")
    (tmp_path / "foo/unixlib.c").write_text("int wrong(void) { return 8; }\n")
    for arch in g.ARCHES:
        ninja = tmp_path / f"build-{arch}/build.ninja"
        ninja.write_text(
            ninja.read_text().replace("rust_cohab_stub.c.obj", "reactos/unixlib.c.obj")
        )
    c = g.model("foo", tmp_path)
    assert c.c_shims == ["foo/reactos/unixlib.c"]
    out = g.rewrite_cmake(tmp_path / "foo/CMakeLists.txt", c)
    assert "add_library(foo_shims OBJECT reactos/unixlib.c)" in out


def test_resource_dependency_lists_survive_both_native_rewriters(tmp_path):
    deps = (
        "set(first ${CMAKE_CURRENT_BINARY_DIR}/m.tlb)\n"
        "set(all_deps ${first})\n"
        "list(APPEND all_deps ${CMAKE_CURRENT_BINARY_DIR}/extra.tlb)\n"
        "set_source_files_properties(rsrc.rc PROPERTIES OBJECT_DEPENDS ${all_deps})\n"
    )
    cmake = tmp_path / "CMakeLists.txt"
    cmake.write_text(CMAKE + deps)
    out = g.rewrite_cmake(cmake, cell())
    assert (
        "EXTRA_DEPENDS psdk ${CMAKE_CURRENT_BINARY_DIR}/m.tlb ${CMAKE_CURRENT_BINARY_DIR}/extra.tlb"
        in out
    )
    cmake.write_text(
        "add_library(m_res OBJECT rsrc.rc)\n"
        "add_cargo_module(m m-cargo m.exe\n    KIND win32exe\n    EXTRA_DEPENDS m_res psdk)\n"
        + deps
    )
    g._rpc("_drop_res_in", {"cmake": cmake, "target": "m_res"}, tmp_path)
    out = cmake.read_text()
    assert "OBJECT_DEPENDS" not in out
    assert (
        "EXTRA_DEPENDS psdk ${CMAKE_CURRENT_BINARY_DIR}/m.tlb ${CMAKE_CURRENT_BINARY_DIR}/extra.tlb"
        in out
    )
    cmake.write_text(CMAKE + deps + "set(first ${all_deps})\n")
    before = cmake.read_bytes()
    with pytest.raises(g.GenError, match="recursive CMake list variable"):
        g.rewrite_cmake(cmake, cell())
    assert cmake.read_bytes() == before
