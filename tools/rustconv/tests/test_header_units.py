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
