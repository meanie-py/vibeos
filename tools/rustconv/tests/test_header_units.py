"""Header inline functions as port units (`port headers`, `translate --via`), 2026-10-06.

identify_funcs.h's 44 `AtaDev*` helpers live in sdk/include, are defined in no .c and no crate,
and so reached atapi's resolve as "defined nowhere in the tree". Each test pins one piece of
the path that now translates them through a TU that includes the header.
"""

from pathlib import Path

from rustconv import macro_authority, port
from rustconv.function_coverage import restore_requested_name
from rustconv.scanner import scan_file

_HEADER = (
    "FORCEINLINE\nBOOLEAN\nAtaDevIsTape(\n    PIDENTIFY_PACKET_DATA Data)\n{\n    return 1;\n}\n"
)


def test_case_only_rename_is_restored_to_the_c_name():
    body = "unsafe fn AtaDevIsReadLookAheadEnabled(x: u8) -> u8 {\n    0\n}\n"
    out = restore_requested_name(body, "AtaDevIsReadLookAHeadEnabled")
    assert "fn AtaDevIsReadLookAHeadEnabled(" in out and "LookAhead" not in out


def test_name_restore_takes_no_other_freedom():
    exact = "fn Foo() {}\nfn foo() {}\n"
    assert restore_requested_name(exact, "Foo") == exact  # already present
    two = "fn FOO() {}\nfn foo() {}\n"
    assert restore_requested_name(two, "Foo") == two  # ambiguous
    other = "fn Bar() {}\n"
    assert restore_requested_name(other, "Foo") == other  # not a case variant


def test_a_function_defined_in_a_c_header_has_no_symbol_of_its_own(tmp_path):
    h = tmp_path / "identify_funcs.h"
    h.write_text(_HEADER)
    c = tmp_path / "x.c"
    c.write_text(_HEADER)
    assert [f.is_static for f in scan_file(h).functions] == [True]
    assert [f.is_static for f in scan_file(c).functions] == [False]  # .c unchanged


def test_a_header_borrows_its_via_tus_compilation(tmp_path):
    h, tu = tmp_path / "a.h", tmp_path / "a.c"
    h.write_text("")
    tu.write_text("")
    assert macro_authority.flags_source(h) == h
    macro_authority.register_via(h, tu)
    try:
        assert macro_authority.flags_source(h) == tu.resolve()
    finally:
        macro_authority._VIA.pop(h.resolve(), None)


def test_header_units_get_their_own_module_name():
    assert port._module_name("drivers/x/enum.c") == "enum"
    assert port._module_name("sdk/include/reactos/drivers/ata/identify_funcs.h") == (
        "identify_funcs_h"
    )
    state = {"tus": ["a/b.c"], "header_tus": {"sdk/include/reactos/x.h": "a/b.c"}}
    assert port._units(state) == ["a/b.c", "sdk/include/reactos/x.h"]


def test_wire_modules_adds_a_translated_unit_lib_rs_does_not_list(tmp_path, monkeypatch):
    crate = tmp_path / "atapi-rust"
    (crate / "src" / "atapi").mkdir(parents=True)
    (crate / "src" / "lib.rs").write_text(
        "pub mod prelude;\n\npub mod atapi {\n    pub mod fdo;\n    pub mod r#enum;\n}\n"
    )
    for name in ("fdo", "enum", "identify_funcs_h"):
        (crate / "src" / "atapi" / f"{name}.rs").write_text("unsafe fn F() {}\n")
    monkeypatch.setattr(port, "_crate_dir", lambda state: crate)
    state = {
        "stem": "atapi",
        "tus": ["d/fdo.c", "d/enum.c"],
        "header_tus": {"sdk/include/reactos/drivers/ata/identify_funcs.h": "d/fdo.c"},
    }
    assert port.wire_modules(state) == ["identify_funcs_h"]
    lib = (crate / "src" / "lib.rs").read_text()
    assert "    pub mod identify_funcs_h;\n}" in lib
    assert port.wire_modules(state) == []  # idempotent


def test_discovery_adopts_private_headers_and_routes_platform_ones(tmp_path, monkeypatch):
    repo = tmp_path
    module = repo / "drivers" / "atapi"
    module.mkdir(parents=True)
    (module / "fdo.c").write_text("int main(void) { return AtaDevIsTape(0); }\n")
    private = repo / "sdk" / "include" / "reactos" / "drivers" / "ata" / "identify_funcs.h"
    private.parent.mkdir(parents=True)
    private.write_text(_HEADER)
    platform = repo / "sdk" / "include" / "ddk" / "ntstrsafe.h"
    platform.parent.mkdir(parents=True)
    platform.write_text("FORCEINLINE\nint\nRtlStringCchPrintfExA(int a)\n{\n    return a;\n}\n")
    monkeypatch.setattr(port, "REPO", repo)
    monkeypatch.setattr(
        macro_authority, "included_headers", lambda root, src, arch="amd64": {private, platform}
    )
    state = {
        "module_dir": "drivers/atapi",
        "tus": ["drivers/atapi/fdo.c"],
        "stages": [{"stage": "resolve", "undefined": ["AtaDevIsTape", "RtlStringCchPrintfExA"]}],
    }
    units, skipped = port.discover_header_units(state)
    assert units == {
        str(Path("sdk/include/reactos/drivers/ata/identify_funcs.h")): "drivers/atapi/fdo.c"
    }
    assert "-sys crate" in skipped["RtlStringCchPrintfExA"]


def test_a_fact_the_island_now_defines_is_found_on_every_arch(tmp_path, monkeypatch):
    src = tmp_path / "src"
    src.mkdir()
    for arch in ("i386", "amd64", "arm64"):
        extra = "pub const ONLY_64: u32 = 2;\n" if arch != "i386" else ""
        (src / f"types_{arch}.rs").write_text(f"pub const ATA_MAX_DEVICE: u32 = 15;\n{extra}")
    monkeypatch.setattr(port, "_crate_dir", lambda state: tmp_path)
    # Defined by every island: the prelude fact is a duplicate. Defined by some: not (yet).
    assert port._island_constants({}) == {"ATA_MAX_DEVICE"}


def test_enum_aliases_read_the_substrate_island_the_types_glob(tmp_path):
    src, sys_src = tmp_path / "src", tmp_path / "sys"
    src.mkdir()
    sys_src.mkdir()
    for arch in ("i386", "amd64", "arm64"):
        (src / f"types_{arch}.rs").write_text("use ntkrnl_sys::ddk::*;\n")
        (sys_src / f"bindgen_{arch}.rs").write_text(
            "pub const _SYSTEM_POWER_STATE_PowerSystemWorking: _SYSTEM_POWER_STATE = 1;\n"
        )
    from rustconv import prelude_resolve as pr

    assert pr.island_enum_aliases({"PowerSystemWorking"}, src) == {}
    got = pr.island_enum_aliases({"PowerSystemWorking"}, src, sys_src)
    assert got["PowerSystemWorking"].text == (
        "const PowerSystemWorking: _SYSTEM_POWER_STATE = _SYSTEM_POWER_STATE_PowerSystemWorking;"
    )


_SYS_LIB = """
pub mod base;
pub mod interlocked;
#[cfg(feature = "inline_exports")]
pub mod inline;
#[cfg(feature = "never_default")]
pub mod hidden;
"""
_SYS_INLINE = """
pub mod common;
pub mod ke;
"""
_SYS_KE = """
/// macro on amd64/arm64, an export on i386
#[cfg(not(target_arch = "x86"))]
#[inline]
pub unsafe fn KeQuerySystemTime(t: *mut i64) {
    extern "C" { static SharedUserData: *mut u8; }
    let s = "{ not a brace }";
}
impl Foo { pub fn NotAHelper() {} }
"""


def _fake_sys(tmp_path):
    sys_src = tmp_path / "ntkrnl-sys" / "src"
    (sys_src / "inline").mkdir(parents=True)
    (sys_src.parent / "Cargo.toml").write_text(
        '[package]\nname = "ntkrnl-sys"\n\n[features]\ndefault = ["inline_exports"]\n'
        "inline_exports = []\nnever_default = []\n"
    )
    (sys_src / "lib.rs").write_text(_SYS_LIB)
    # `base` is ntoskrnl's Rust-flavoured core, never a header port: ignored entirely.
    (sys_src / "base.rs").write_text("pub unsafe fn OnlyInBase() {}\n")
    (sys_src / "interlocked.rs").write_text(
        "pub unsafe fn InsertTailList(h: *mut u8, e: *mut u8) {}\n"
        '#[cfg(not(target_arch = "x86"))]\n'
        'extern "system" {\n    #[link_name = "X"]\n'
        "    pub fn InterlockedPushEntrySList(a: u8);\n}\n"
        '#[cfg(target_arch = "x86")]\n'
        "pub unsafe fn InterlockedPushEntrySList(a: u8) {}\n"
    )
    (sys_src / "hidden.rs").write_text("pub fn Hidden() {}\n")
    (sys_src / "inline" / "mod.rs").write_text(_SYS_INLINE)
    (sys_src / "inline" / "common.rs").write_text(
        "pub unsafe fn InsertTailList(h: *mut u8, e: *mut u8) {}\n"
    )
    (sys_src / "inline" / "ke.rs").write_text(_SYS_KE)
    for suffix in ("i386", "amd64", "arm64"):
        glob = "pub fn KeQuerySystemTime(t: *mut i64);\n" if suffix == "i386" else ""
        (sys_src / f"bindgen_{suffix}.rs").write_text(f'extern "system" {{\n    {glob}}}\n')
    return sys_src


def test_substrate_helpers_are_found_per_arch_and_rendered_as_imports(tmp_path):
    from rustconv import prelude_resolve as pr

    sys_src = _fake_sys(tmp_path)
    names = {"InsertTailList", "KeQuerySystemTime", "InterlockedPushEntrySList"}
    names |= {"Hidden", "NotAHelper", "OnlyInBase"}
    found, why = pr.substrate_helpers(names, sys_src, "ntkrnl_sys")
    # inline:: beats another module for the same name; i386's KeQuerySystemTime comes from the glob.
    assert found["InsertTailList"] == dict.fromkeys(pr.ARCHES, "inline::common")
    assert found["KeQuerySystemTime"] == {
        "x86": None,
        "x86_64": "inline::ke",
        "aarch64": "inline::ke",
    }
    # The extern-block declaration counts, under the block's own cfg.
    assert found["InterlockedPushEntrySList"] == dict.fromkeys(pr.ARCHES, "interlocked")
    # Behind a non-default feature, or a method of an impl: not a helper at all.
    assert "Hidden" not in found and "NotAHelper" not in found and not why
    assert "OnlyInBase" not in found
    block = pr.render_substrate(found, "ntkrnl_sys")
    assert (
        '#[cfg(not(target_arch = "x86"))]\npub use ntkrnl_sys::inline::ke::KeQuerySystemTime;'
        in block
    )
    assert "pub use ntkrnl_sys::inline::common::InsertTailList;" in block
    prelude = tmp_path / "prelude.rs"
    prelude.write_text("pub use crate::types::*;\n")
    pr.write_substrate(prelude, block)
    assert pr.existing_substrate(prelude) == set(found)
    assert not pr.write_substrate(prelude, block)  # idempotent


def test_a_helper_two_modules_define_for_one_arch_is_reported_not_chosen(tmp_path):
    from rustconv import prelude_resolve as pr

    sys_src = _fake_sys(tmp_path)
    (sys_src / "inline" / "ke.rs").write_text(_SYS_KE + "pub fn Twice() {}\n")
    (sys_src / "inline" / "common.rs").write_text("pub fn Twice() {}\n")
    found, why = pr.substrate_helpers({"Twice"}, sys_src, "ntkrnl_sys")
    assert not found and "inline::common, inline::ke" in why["Twice"]


def test_glob_ties_are_read_per_arch_from_the_islands(tmp_path):
    # atapi: the i386 island declares KeGetCurrentIrql (a real export there), the amd64 one
    # does not -- so the `crate::types::*` / `ntkrnl_sys::ddk::*` collision is i386-only, and
    # resolve (which compiles amd64) never saw it.
    from rustconv import prelude_resolve as pr

    src, sys_src = tmp_path / "c" / "src", tmp_path / "sys" / "src"
    (src / "m").mkdir(parents=True)
    sys_src.mkdir(parents=True)
    decl = 'extern "C" {\n    pub fn KeGetCurrentIrql() -> KIRQL;\n    pub fn Other(a: u32);\n}\n'
    (src / "types_i386.rs").write_text(decl + "impl X { pub fn set_Foo(&mut self) {} }\n")
    (src / "types_amd64.rs").write_text('extern "C" {\n    pub fn Other(a: u32);\n}\n')
    (src / "prelude.rs").write_text("pub use crate::types::*;\npub use ntkrnl_sys::ddk::*;\n")
    for suffix in ("i386", "amd64", "arm64"):
        (sys_src / f"bindgen_{suffix}.rs").write_text(decl)
    (src / "m" / "a.rs").write_text(
        "use ntkrnl_sys::ddk::{Other};\nfn f() { KeGetCurrentIrql(); Other(1); x.set_Foo(); }\n"
    )
    ties = pr.glob_ties(src, src / "m", sys_src)
    assert ties == {"KeGetCurrentIrql": ["x86"]}  # Other is imported explicitly in the file
    found = pr.with_glob_ties({"KeQuerySystemTime": {"x86": None, "x86_64": "inline::ke"}}, ties)
    out = pr.render_substrate(found, "ntkrnl_sys")
    assert '#[cfg(target_arch = "x86")]\npub use ntkrnl_sys::ddk::KeGetCurrentIrql;' in out
    # A name the prelude already spells out is settled there (E0252 otherwise).
    (src / "prelude.rs").write_text("pub use ntkrnl_sys::ddk::KeGetCurrentIrql;\n")
    assert pr.glob_ties(src, src / "m", sys_src) == {}
