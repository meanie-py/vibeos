"""c2rust folds FIELD_OFFSET to ONE target's number; the table must be per-arch again."""

from __future__ import annotations

from rustconv.offset_restore import c_offset_tables, restore

C = """
static DWORD dwDriverInfo3Offsets[] = {
    FIELD_OFFSET(DRIVER_INFO_3W, pName),
    FIELD_OFFSET(DRIVER_INFO_3W, pEnvironment),  // a comment
    FIELD_OFFSET(DATA, type),
    MAXDWORD
};
static const WCHAR wszNotATable[] = L"x";
"""

RUST = """static mut dwDriverInfo3Offsets: [DWORD; 4] = [
    8 as ::core::ffi::c_ulonglong as LONG as DWORD,
    16 as ::core::ffi::c_ulonglong as LONG as DWORD,
    24 as ::core::ffi::c_ulonglong as LONG as DWORD,
    MAXDWORD as DWORD,
];
"""


def test_folded_offsets_become_offset_of_and_the_terminator_stays():
    # localspl: the 64-bit offsets put every string pointer in the wrong slot on i386, and
    # spoolsv crashed in ToMultiSz(pDependentFiles == NULL) on GetPrinterDriver level 3.
    assert c_offset_tables(C)["dwDriverInfo3Offsets"][3] is None
    new, fixed, refused = restore(RUST, C)
    assert fixed == ["dwDriverInfo3Offsets"] and refused == []
    assert "core::mem::offset_of!(DRIVER_INFO_3W, pName) as DWORD" in new
    assert "core::mem::offset_of!(DATA, type_) as DWORD" in new  # bindgen's keyword spelling
    assert "MAXDWORD as DWORD," in new and "8 as" not in new
    assert restore(new, C)[0] == new  # idempotent


def test_a_count_mismatch_or_a_non_number_is_refused_not_guessed():
    short = RUST.replace("    24 as ::core::ffi::c_ulonglong as LONG as DWORD,\n", "")
    assert restore(short, C)[1:] == ([], ["dwDriverInfo3Offsets (3 Rust entries, 4 in the C)"])
    odd = RUST.replace("16 as ::core::ffi::c_ulonglong", "SOME_CONST")
    new, fixed, refused = restore(odd, C)
    assert new == odd and fixed == [] and "not a folded number" in refused[0]
