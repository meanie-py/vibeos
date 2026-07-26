"""The refusals matter more than the edits: a wrong splice here corrupts a paid translation."""

import importlib.util
import sys
from pathlib import Path

_SPEC = importlib.util.spec_from_file_location(
    "island_adopt", Path(__file__).resolve().parents[1] / "rustconv" / "island_adopt.py"
)
IA = importlib.util.module_from_spec(_SPEC)
assert _SPEC.loader is not None
# Register before exec: @dataclass resolves `cls.__module__` through sys.modules.
sys.modules["island_adopt"] = IA
_SPEC.loader.exec_module(IA)


def diag(code, message, file, line, byte_start, byte_end):
    return {
        "level": "error",
        "code": {"code": code},
        "message": message,
        "spans": [
            {
                "is_primary": True,
                "file_name": file,
                "line_start": line,
                "byte_start": byte_start,
                "byte_end": byte_end,
            }
        ],
    }


def reader(text):
    return lambda _f: text.encode()


def at(text, needle, occurrence=0):
    idx = -1
    for _ in range(occurrence + 1):
        idx = text.index(needle, idx + 1)
    return idx, idx + len(needle)


# --- bitfield accessors ---------------------------------------------------------------------


def test_bitfield_read_becomes_a_call():
    src = "let b = cs.fNewWindowMode != 0;\n"
    s, e = at(src, "fNewWindowMode")
    edits, refused = IA.propose(
        [diag("E0615", "attempted to take value of method `fNewWindowMode`", "a.rs", 1, s, e)],
        reader(src),
    )
    assert refused == []
    assert len(edits) == 1 and edits[0].replacement == "()" and edits[0].start == edits[0].end == e


def test_bitfield_write_becomes_a_setter():
    src = "    (*pss).fShowAllObjects = TRUE;\n"
    s, e = at(src, "fShowAllObjects")
    edits, refused = IA.propose(
        [diag("E0615", "attempted to take value of method `fShowAllObjects`", "a.rs", 1, s, e)],
        reader(src),
    )
    assert refused == []
    assert edits[0].replacement == "set_fShowAllObjects(TRUE);"


def test_bitfield_write_with_a_multiline_rhs_is_REFUSED():
    """The bug this rule exists to not repeat: taking the rest of the line truncates the call."""
    src = "    ss.fNoConfirmRecycle = (SendDlgItemMessage(\n        hwndDlg,\n    ) == BST_UNCHECKED);\n"
    s, e = at(src, "fNoConfirmRecycle")
    edits, refused = IA.propose(
        [diag("E0615", "attempted to take value of method `fNoConfirmRecycle`", "a.rs", 1, s, e)],
        reader(src),
    )
    assert edits == []
    assert [r.reason for r in refused] == ["multiline_rhs"]


def test_bitfield_write_with_unbalanced_parens_is_refused():
    src = "    ss.fWebView = foo(bar;\n"
    s, e = at(src, "fWebView")
    edits, refused = IA.propose(
        [diag("E0615", "attempted to take value of method `fWebView`", "a.rs", 1, s, e)],
        reader(src),
    )
    assert edits == [] and refused[0].reason == "unbalanced_rhs"


# --- uninitialised locals -------------------------------------------------------------------


def test_uninit_local_gets_an_initialiser():
    src = "fn f() {\n    let mut rect: RECT;\n    GetClientRect(h, &mut rect);\n}\n"
    s, e = at(src, "rect", 1)  # the USE site, not the declaration
    edits, refused = IA.propose(
        [diag("E0381", "used binding `rect` isn't initialized", "a.rs", 3, s, e)], reader(src)
    )
    assert refused == []
    assert edits[0].replacement == "    let mut rect: RECT = core::mem::zeroed();"


def test_uninit_local_without_a_bare_declaration_is_refused():
    src = "fn f() {\n    let rect;\n}\n"
    edits, refused = IA.propose(
        [diag("E0381", "used binding `rect` isn't initialized", "a.rs", 2, 14, 18)], reader(src)
    )
    assert edits == [] and refused[0].reason == "no_bare_declaration"


# --- macro call syntax ----------------------------------------------------------------------


def test_macro_called_as_a_function_gets_a_bang():
    src = "    let n = _countof(buf);\n"
    s, e = at(src, "_countof")
    edits, refused = IA.propose(
        [diag("E0423", "expected function, found macro `_countof`", "a.rs", 1, s, e)], reader(src)
    )
    assert refused == [] and edits[0].replacement == "!" and edits[0].start == e


def test_a_non_macro_unresolved_name_is_left_alone():
    """Only diagnostics that actually say `macro` qualify; everything else is someone else's."""
    src = "    let n = SomeFunc(buf);\n"
    s, e = at(src, "SomeFunc")
    edits, refused = IA.propose(
        [diag("E0425", "cannot find function `SomeFunc` in this scope", "a.rs", 1, s, e)],
        reader(src),
    )
    assert edits == [] and refused == []


def test_macro_name_not_followed_by_a_call_is_refused():
    src = "    let f = _countof;\n"
    s, e = at(src, "_countof")
    edits, refused = IA.propose(
        [diag("E0423", "expected value, found macro `_countof`", "a.rs", 1, s, e)], reader(src)
    )
    assert edits == [] and refused[0].reason == "not_a_call"


# --- variadic promotion ---------------------------------------------------------------------


def test_variadic_argument_is_promoted_explicitly():
    src = "    swprintf(buf, fmt, szDrive[0]);\n"
    s, e = at(src, "szDrive[0]")
    edits, refused = IA.propose(
        [diag("E0617", "can't pass `u16` to variadic function", "a.rs", 1, s, e)], reader(src)
    )
    assert refused == [] and edits[0].replacement == " as ::core::ffi::c_int"


# --- reference cast to integer ---------------------------------------------------------------


def test_ref_as_int_takes_the_address_first():
    src = "    SendMessageW(h, m, 0, &mut li as LPARAM);\n"
    s, e = at(src, "&mut li as LPARAM")
    edits, refused = IA.propose(
        [diag("E0606", "casting `&mut LVITEMW` as `isize` is invalid", "a.rs", 1, s, e)],
        reader(src),
    )
    assert refused == []
    assert edits[0].replacement == "core::ptr::addr_of_mut!(li) as LPARAM"


def test_shared_ref_uses_addr_of():
    src = "    f(&lc as LPARAM);\n"
    s, e = at(src, "&lc as LPARAM")
    edits, _ = IA.propose(
        [diag("E0606", "casting `&LVCOLUMNW` as `isize` is invalid", "a.rs", 1, s, e)], reader(src)
    )
    assert edits[0].replacement == "core::ptr::addr_of!(lc) as LPARAM"


# --- overlap safety ---------------------------------------------------------------------------


def test_overlapping_edits_keep_the_outermost():
    a = IA.Edit("a.rs", 1, 10, 40, "X", "orig", "r1", "auth")
    b = IA.Edit("a.rs", 1, 20, 30, "Y", "orig", "r2", "auth")
    kept, dropped = IA.select_non_overlapping([a, b])
    assert [e.rule for e in kept] == ["r1"] and dropped[0].reason == "overlapping_span"


def test_zero_width_insertions_at_the_same_point_both_survive():
    a = IA.Edit("a.rs", 1, 10, 10, "()", "", "r1", "auth")
    b = IA.Edit("a.rs", 2, 10, 10, "!", "", "r2", "auth")
    kept, dropped = IA.select_non_overlapping([a, b])
    assert len(kept) == 2 and dropped == []


def test_apply_skips_an_edit_whose_span_went_stale(tmp_path):
    (tmp_path / "a.rs").write_text("hello world\n")
    stale = IA.Edit("a.rs", 1, 0, 5, "HI", "goodbye", "r", "auth")
    assert IA.apply([stale], tmp_path) == 0
    assert (tmp_path / "a.rs").read_text() == "hello world\n"
