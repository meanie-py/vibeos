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
        destination="reactos/system32",
        name_on_cd=None,
    )
    base.update(kw)
    return g.Cell(**base)


def test_facts_read_the_body_crate_and_the_cd_destination(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text(CMAKE)
    facts = g.cmake_facts(f, "m")
    assert facts["body"] == ("m-rust", [], True)
    assert facts["destination"] == "reactos/system32"
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
    libs["i386"].append("sdk/lib/crt/libchkstk.a")
    out = g.rewrite_cmake(f, cell(static_libs=libs))
    chk = "list(APPEND M_REUSED ${CMAKE_BINARY_DIR}/sdk/lib/crt/libchkstk.a)"
    assert f'if(ARCH STREQUAL "i386")\n    {chk}' in out
    rs = g.build_rs(cell(static_libs=libs))
    assert '"x86" => &["sdk/lib/libwine/libwine.a", "sdk/lib/crt/libchkstk.a"],' in rs


def test_build_rs_names_every_input_and_keeps_an_auto_dll_base():
    rs = g.build_rs(cell(import_libs=[("dll/x/kernel32.spec", "kernel32.dll", "kernel32")]))
    assert '.rust_staticlib("m-rust")' in rs
    assert '.import_lib("dll/x/kernel32.spec", "kernel32.dll", "kernel32")' in rs
    assert '.resource_from_rc("base/m/rsrc.rc", "m_res")' in rs
    assert ".msvcrtex_crt(MsvcrtexKind::ExeWide)" in rs
    # A DLL cmake gave no --image-base keeps lld's auto base: None, never a guess.
    bases = {"i386": None, "amd64": 0x7FE60000000, "arm64": 0x7FE60000000}
    dll = g.build_rs(cell(module_kind="Win32Dll", image_base=bases, msvcrtex="Dll"))
    assert '"x86" => ("i686-kernel.json", "wWinMainCRTStartup", None)' in dll
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
    assert "target_compile_definitions(m_shims PRIVATE UNICODE _UNICODE)" in out
    assert "EXTRA_DEPENDS m_shims m_res" in out
    assert '.c_from_compile_db("m_shims", &["base/m/rust_varargs_shim.c"])' in g.build_rs(c)
    assert ".c_shims(" not in g.build_rs(c)


def test_arch_specific_cmake_is_refused(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    # calc: an i386-only RUSTFLAGS for the body the cell would silently drop.
    f.write_text(CMAKE.replace('"" TRUE)', '"" TRUE ${M_RUST_ENV})'))
    assert g.cmake_facts(f, "m")["unknown"] == ["rust_staticlib_variant extras ['${M_RUST_ENV}']"]
    # winprint: a different CD destination per arch.
    f.write_text(CMAKE + "add_cd_file(TARGET m DESTINATION reactos/other FOR all)\n")
    assert g.cmake_facts(f, "m")["unknown"] == [
        "add_cd_file to both reactos/system32 and reactos/other"
    ]


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
