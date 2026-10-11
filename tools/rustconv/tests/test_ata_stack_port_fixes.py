"""Regressions from porting upstream's new ATA stack (pciidex/atapi/legide), 2026-10-06.

Each test pins one tool defect that stood between a kernel driver and `rustconv port`; each
was a CLASS, so the test is written against the shape, not the driver.
"""

from pathlib import Path

from rustconv import crate_integrate, port
from rustconv.__main__ import _preamble_is_function_macros_only
from rustconv.c2rust_anchor import _INTRINSIC_PREFIX, _is_own_data, _unprefix_intrinsics
from rustconv.crate_repair import _RE_NO_FIELD
from rustconv.profile_derive import _rust_mod_ident, prelude_provided_names
from rustconv.scanner import scan_file


# ---- scanner ------------------------------------------------------------------------------


def _scan(tmp_path: Path, src: str):
    p = tmp_path / "t.c"
    p.write_text(src)
    return scan_file(p)


def test_data_seg_table_is_data_not_an_unextractable_function(tmp_path):
    src = (
        '#define X_PAGED_DATA DATA_SEG("PAGECONS")\n'
        "X_PAGED_DATA\nstatic const struct { int a; } Table[] = { { 1 } };\n"
        "int F(void)\n{\n    X_PAGED_DATA static const int t = 2;\n    return t;\n}\n"
    )
    scan = _scan(tmp_path, src)
    assert [f.name for f in scan.functions] == ["F"]
    assert not scan.unextracted


def test_code_seg_between_return_type_and_name_keeps_the_function_and_its_section(tmp_path):
    src = 'static\nBOOLEAN\nCODE_SEG("PAGE")\nNvHasUdmaCable(\n    USHORT a)\n{\n    return 1;\n}\n'
    fns = _scan(tmp_path, src).functions
    assert [(f.name, f.code_seg) for f in fns] == [("NvHasUdmaCable", "PAGE")]


# ---- c2rust anchor --------------------------------------------------------------------------


def test_intrinsic_renames_round_trip_to_the_plain_name():
    text = f"{_INTRINSIC_PREFIX}BitScanReverse(&mut i, m); {_INTRINSIC_PREFIX}InterlockedOr(p, 1);"
    assert _unprefix_intrinsics(text) == "BitScanReverse(&mut i, m); InterlockedOr(p, 1);"


def test_harvest_keeps_only_the_files_own_data():
    c = "static IO_COMPLETION_ROUTINE Done;\nstatic const int Table[] = {1};\nDone(int x)\n{ }\n"
    own_fns = {"Done"}
    assert _is_own_data("Table", c, own_fns)
    assert not _is_own_data("Done", c, own_fns)  # a function-typed forward declaration
    assert not _is_own_data("GUID_DEVINTERFACE_DISK", c, own_fns)  # header data via initguid


# ---- translate ------------------------------------------------------------------------------


def test_a_preamble_of_function_like_macros_has_nothing_to_translate():
    assert _preamble_is_function_macros_only(
        "#define A(x) \\\n    ((x)->y >= 1)\n\n#define B(x, n) ((x) + (n))"
    )
    assert not _preamble_is_function_macros_only("#define A(x) (x)\nstatic int t;")
    assert not _preamble_is_function_macros_only("#define TAG 'PedI'")


# ---- integrate ------------------------------------------------------------------------------


def test_multichar_constant_folds_big_endian_and_spares_lifetimes():
    src = "pub const T: u32 = 'PedI' as u32;\nfn x<'a, 'b>(y: &'a u8, z: &'b u8) {}\n"
    out, folded = crate_integrate.fold_multichar_literals(src)
    assert "0x50656449" in out and folded == ["PedI"]
    assert "fn x<'a, 'b>(y: &'a u8, z: &'b u8)" in out


def test_function_typed_forward_static_is_dropped_beside_its_definition():
    src = (
        "pub static Done: IO_COMPLETION_ROUTINE = None;\n"
        "pub static Kept: PVOID = None;\n"
        'unsafe extern "C" fn Done(a: PVOID) -> NTSTATUS {\n    0\n}\n'
    )
    out, dropped = crate_integrate.drop_fn_forward_statics(src)
    assert dropped == ["Done"] and "static Done" not in out and "static Kept" in out


def test_provided_type_redefinition_and_self_alias_are_dropped():
    src = (
        "#[repr(C)]\npub struct _UNICODE_STRING {\n    pub Length: u16,\n}\n"
        "pub type _UNICODE_STRING = _UNICODE_STRING;\n"
        "pub struct Mine {\n    pub a: u8,\n}\n"
    )
    out, dropped = crate_integrate.drop_provided_type_defs(src, {"_UNICODE_STRING"})
    assert dropped == ["_UNICODE_STRING", "_UNICODE_STRING"]
    assert "_UNICODE_STRING" not in out and "pub struct Mine" in out


# ---- crate-repair / derive / port ----------------------------------------------------------


def test_no_field_message_matches_a_module_qualified_type():
    m = _RE_NO_FIELD.search("no field `Control` on type `types_arch::_IDE_REGISTERS`")
    assert m and m.group("ty").split("::")[-1] == "_IDE_REGISTERS"


def test_keyword_module_names_are_raw_identifiers():
    assert _rust_mod_ident("enum") == "r#enum" and _rust_mod_ident("fdo") == "fdo"


def test_prelude_names_count_explicit_reexports(tmp_path):
    prelude = tmp_path / "prelude.rs"
    prelude.write_text(
        "pub const CmResourceTypePort: u8 = 1;\n"
        "pub use ntkrnl_sys::driver_ddk::{\n    IRP_MN_START_DEVICE, IRP_MN_STOP_DEVICE,\n};\n"
        "pub use crate::types::_SCSI_REQUEST_BLOCK;\n"
    )
    assert {
        "CmResourceTypePort",
        "IRP_MN_START_DEVICE",
        "IRP_MN_STOP_DEVICE",
        "_SCSI_REQUEST_BLOCK",
    } <= prelude_provided_names(prelude)


def test_kernel_modules_translate_through_their_registered_target():
    args = port._target_args(
        {"stem": "pciidex", "crate": "pciidex-rust", "usermode": False, "module_dir": "x"}
    )
    assert args == ["--target", "pciidex"]
    user = port._target_args(
        {"stem": "setuplib", "crate": "setuplib-rust", "usermode": True, "module_dir": "base/x"}
    )
    assert user[:2] == ["--target", "setuplib:base/x"]


def test_multichar_fold_skips_the_gap_between_char_literals():
    # cmd's parser.rs: `', '` between two one-char literals is not a multi-char constant.
    src = "    let f = [' ', '%', '%', '%', 'c', ' ', 'i', 'n', '\\'', '\\u{2c}', 0];\n"
    assert crate_integrate.fold_multichar_literals(src) == (src, [])
    folded, names = crate_integrate.fold_multichar_literals("    let t = 'PedI'; let c = ' ';\n")
    assert names == ["PedI"] and "0x50656449 /* 'PedI' */" in folded and "' '" in folded
    comment = "    /*\n     * any label containing '/?' in it.\n     */\n"
    assert crate_integrate.fold_multichar_literals(comment) == (comment, [])


def test_a_bank_bought_before_kept_diagnostics_stays_in_its_namespace(tmp_path, monkeypatch):
    # cmd: 55 TUs banked stripped; extending or replaying under __diag_v1 is refused.
    monkeypatch.setattr(port, "REPO", tmp_path)
    responses = tmp_path / "tools" / "rustconv" / ".responses"
    old = responses / "cmd__copy__factseal_v1__callseal_v1__wideseal_v1"
    old.mkdir(parents=True)
    assert port._bank_predates_kept_diagnostics("cmd", "base/shell/cmd/copy.c")
    (responses / f"{old.name}__diag_v1").mkdir()
    assert not port._bank_predates_kept_diagnostics("cmd", "base/shell/cmd/copy.c")
    assert not port._bank_predates_kept_diagnostics("cmd", "base/shell/cmd/new.c")


def test_integrate_keeps_a_definition_the_prelude_reexports_from_this_module(tmp_path):
    # cmd: resolve wired `pub use crate::cmd::dir::_DirSwitchesFlags;`, and integrate then
    # dropped dir.rs's own struct as "prelude-provided" -- defined nowhere after that.
    src_dir = tmp_path / "src"
    (src_dir / "cmd").mkdir(parents=True)
    (src_dir / "prelude.rs").write_text("pub use crate::cmd::dir::_DirSwitchesFlags;\n")
    body = (
        "use crate::prelude::*;\n#[repr(C)]\npub struct _DirSwitchesFlags {\n    pub b: i32,\n}\n"
    )
    crate_integrate._PROVIDED_CACHE.clear()
    kept, _ = crate_integrate.integrate(body, tmp_path, module="dir")
    assert "pub struct _DirSwitchesFlags" in kept
    crate_integrate._PROVIDED_CACHE.clear()
    dropped, _ = crate_integrate.integrate(body, tmp_path, module="vol")
    assert "pub struct _DirSwitchesFlags" not in dropped
