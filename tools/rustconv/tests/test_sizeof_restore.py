"""sizeof_restore: put back a `sizeof` the model dropped around a type name."""

from __future__ import annotations

from rustconv import sizeof_restore


def test_a_macros_type_argument_is_not_a_dropped_sizeof(tmp_path):
    """winprint: `FIELD_OFFSET(DATATYPES_INFO_1W, pName)` beside a real
    `sizeof(DATATYPES_INFO_1W)`; the type in FIELD_OFFSET!'s first slot was rewritten into
    `size_of::<..>()`, which no macro arm accepts."""
    c = tmp_path / "main.c"
    c.write_text(
        "DWORD F(void)\n{\n    return n * sizeof(DATATYPES_INFO_1W)"
        " + FIELD_OFFSET(DATATYPES_INFO_1W, pName);\n}\n"
    )
    rust = (
        'pub unsafe extern "system" fn F() -> DWORD {\n'
        "    n.wrapping_mul(DATATYPES_INFO_1W as DWORD)"
        ".wrapping_add(FIELD_OFFSET!(DATATYPES_INFO_1W, pName) as DWORD)\n"
        "}\n"
    )
    out, fixed = sizeof_restore.restore(rust, c, {"DATATYPES_INFO_1W"})
    assert "wrapping_mul(core::mem::size_of::<DATATYPES_INFO_1W>() as DWORD)" in out
    assert "FIELD_OFFSET!(DATATYPES_INFO_1W, pName)" in out
    assert fixed == ["F: sizeof(DATATYPES_INFO_1W) x1"]


def test_a_type_in_a_later_macro_slot_is_not_a_dropped_sizeof(tmp_path):
    """mfplat: `CONTAINING_RECORD(format, WAVEFORMATEXTENSIBLE, Format)` beside a real
    `sizeof(WAVEFORMATEXTENSIBLE)` -- the type is the SECOND argument, and a first-slot-only
    exemption rewrote it into `size_of::<..>()` on every integrate."""
    c = tmp_path / "mediatype.c"
    c.write_text(
        "HRESULT F(WAVEFORMATEX *format)\n{\n    n = sizeof(WAVEFORMATEXTENSIBLE);\n"
        "    e = CONTAINING_RECORD(format, WAVEFORMATEXTENSIBLE, Format);\n}\n"
    )
    rust = (
        'pub unsafe extern "system" fn F(format: *mut WAVEFORMATEX) -> HRESULT {\n'
        "    n = WAVEFORMATEXTENSIBLE as u32;\n"
        "    e = CONTAINING_RECORD!(format, WAVEFORMATEXTENSIBLE, Format);\n"
        "}\n"
    )
    out, fixed = sizeof_restore.restore(rust, c, {"WAVEFORMATEXTENSIBLE"})
    assert "n = core::mem::size_of::<WAVEFORMATEXTENSIBLE>() as u32;" in out
    assert "CONTAINING_RECORD!(format, WAVEFORMATEXTENSIBLE, Format)" in out
    assert fixed == ["F: sizeof(WAVEFORMATEXTENSIBLE) x1"]
