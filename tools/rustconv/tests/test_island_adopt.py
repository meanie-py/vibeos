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


# --- COM method calls written in C++ syntax ---------------------------------------------------

IFACES = frozenset({"IShellFolder", "IExtractIconW"})


def com_diag(method, iface, file, line, s, e):
    return diag(
        "E0599",
        f"no method named `{method}` found for struct `shellcom::{iface}`",
        file,
        line,
        s,
        e,
    )


def test_com_interfaces_are_derived_from_the_vtbl_structs():
    src = "pub struct IShellFolderVtbl {\n    pub Release: usize,\n}\npub struct NotAnIface {\n}\n"
    assert IA.com_interfaces(src) == frozenset({"IShellFolder"})


def test_nullary_com_call_becomes_a_vtable_dispatch():
    src = "    (*psf).Release();\n"
    s, e = at(src, "Release")
    edits, refused = IA.propose(
        [com_diag("Release", "IShellFolder", "a.rs", 1, s, e)], reader(src), interfaces=IFACES
    )
    assert refused == []
    out = src[: edits[0].start] + edits[0].replacement + src[edits[0].end :]
    assert out == "    ((*(*psf).lpVtbl).Release)(psf);\n"


def test_com_call_with_arguments_keeps_them_after_the_this_pointer():
    src = "    let hr = (*ei).GetIconLocation(flags, buf, cch);\n"
    s, e = at(src, "GetIconLocation")
    edits, refused = IA.propose(
        [com_diag("GetIconLocation", "IExtractIconW", "a.rs", 1, s, e)],
        reader(src),
        interfaces=IFACES,
    )
    assert refused == []
    out = src[: edits[0].start] + edits[0].replacement + src[edits[0].end :]
    assert out == "    let hr = ((*(*ei).lpVtbl).GetIconLocation)(ei, flags, buf, cch);\n"


def test_com_call_on_a_compound_receiver_expression():
    src = "    (*psf.as_raw()).Release();\n"
    s, e = at(src, "Release")
    edits, _ = IA.propose(
        [com_diag("Release", "IShellFolder", "a.rs", 1, s, e)], reader(src), interfaces=IFACES
    )
    out = src[: edits[0].start] + edits[0].replacement + src[edits[0].end :]
    assert out == "    ((*(*psf.as_raw()).lpVtbl).Release)(psf.as_raw());\n"


def test_a_type_that_is_not_a_com_interface_is_left_alone():
    """A translated crate has ordinary Rust structs; rustc reports those identically."""
    src = "    (*thing).Release();\n"
    s, e = at(src, "Release")
    edits, refused = IA.propose(
        [com_diag("Release", "SomePlainStruct", "a.rs", 1, s, e)], reader(src), interfaces=IFACES
    )
    assert edits == [] and refused == []


def test_the_rule_is_inert_without_an_interface_set():
    src = "    (*psf).Release();\n"
    s, e = at(src, "Release")
    edits, refused = IA.propose([com_diag("Release", "IShellFolder", "a.rs", 1, s, e)], reader(src))
    assert edits == [] and refused == []


def test_non_deref_receiver_is_REFUSED():
    """`psf.Release()` gives no pointer to pass as `this` — guessing one is how you dispatch
    on the wrong object."""
    src = "    psf.Release();\n"
    s, e = at(src, "Release")
    edits, refused = IA.propose(
        [com_diag("Release", "IShellFolder", "a.rs", 1, s, e)], reader(src), interfaces=IFACES
    )
    assert edits == [] and refused[0].reason == "receiver_not_parenthesised"


def test_parenthesised_but_not_a_deref_is_REFUSED():
    src = "    (psf).Release();\n"
    s, e = at(src, "Release")
    edits, refused = IA.propose(
        [com_diag("Release", "IShellFolder", "a.rs", 1, s, e)], reader(src), interfaces=IFACES
    )
    assert edits == [] and refused[0].reason == "receiver_not_a_deref"


def test_method_not_called_is_REFUSED():
    src = "    let f = (*psf).Release;\n"
    s, e = at(src, "Release")
    edits, refused = IA.propose(
        [com_diag("Release", "IShellFolder", "a.rs", 1, s, e)], reader(src), interfaces=IFACES
    )
    assert edits == [] and refused[0].reason == "method_not_called"


def test_multiline_argument_list_does_not_get_trailing_whitespace():
    """`git diff --check` rejects a trailing space, so the comma must not carry one when the
    arguments start on the next line."""
    src = "    let hr = (*psf).BindToObject(\n        pidl,\n        ctx,\n    );\n"
    s, e = at(src, "BindToObject")
    edits, refused = IA.propose(
        [com_diag("BindToObject", "IShellFolder", "a.rs", 1, s, e)],
        reader(src),
        interfaces=IFACES,
    )
    assert refused == []
    out = src[: edits[0].start] + edits[0].replacement + src[edits[0].end :]
    assert not any(ln.rstrip() != ln for ln in out.split("\n")), out
    assert out.startswith("    let hr = ((*(*psf).lpVtbl).BindToObject)(psf,\n        pidl,")


# --- the same call with the vtable deref already written out ----------------------------------
#
# rustc names the VTABLE struct in this shape, so the rule has to recover the interface pointer
# from the receiver text rather than from the diagnostic.


def test_call_through_a_written_out_vtbl_becomes_a_dispatch():
    src = "    (*(*psf).lpVtbl).GetAttributesOf(cidl, apidl, rgf);\n"
    s, e = at(src, "GetAttributesOf")
    edits, refused = IA.propose(
        [com_diag("GetAttributesOf", "IShellFolderVtbl", "a.rs", 1, s, e)],
        reader(src),
        interfaces=IFACES,
    )
    assert refused == []
    out = src[: edits[0].start] + edits[0].replacement + src[edits[0].end :]
    assert out == "    ((*(*psf).lpVtbl).GetAttributesOf)(psf, cidl, apidl, rgf);\n"


def test_nullary_call_through_a_written_out_vtbl():
    src = "    (*(*psf).lpVtbl).Release();\n"
    s, e = at(src, "Release")
    edits, refused = IA.propose(
        [com_diag("Release", "IShellFolderVtbl", "a.rs", 1, s, e)], reader(src), interfaces=IFACES
    )
    assert refused == []
    out = src[: edits[0].start] + edits[0].replacement + src[edits[0].end :]
    assert out == "    ((*(*psf).lpVtbl).Release)(psf);\n"


def test_a_vtbl_struct_whose_interface_is_unknown_is_left_alone():
    """`FooVtbl` only qualifies when `Foo` is a known interface — not on the suffix alone."""
    src = "    (*(*p).lpVtbl).Release();\n"
    s, e = at(src, "Release")
    edits, refused = IA.propose(
        [com_diag("Release", "IUnrelatedVtbl", "a.rs", 1, s, e)], reader(src), interfaces=IFACES
    )
    assert edits == [] and refused == []


def test_vtbl_receiver_that_is_not_lpVtbl_is_REFUSED():
    """A vtable reached by another name is not the `lpVtbl` idiom; the owner is unknown."""
    src = "    (*(*psf).vtable).Release();\n"
    s, e = at(src, "Release")
    edits, refused = IA.propose(
        [com_diag("Release", "IShellFolderVtbl", "a.rs", 1, s, e)], reader(src), interfaces=IFACES
    )
    assert edits == [] and refused[0].reason == "vtbl_receiver_not_lpVtbl"


def test_vtbl_owner_that_is_not_a_deref_is_REFUSED():
    """`(*slot.lpVtbl)` has no interface pointer to pass as `this`."""
    src = "    (*slot.lpVtbl).Release();\n"
    s, e = at(src, "Release")
    edits, refused = IA.propose(
        [com_diag("Release", "IShellFolderVtbl", "a.rs", 1, s, e)], reader(src), interfaces=IFACES
    )
    assert edits == [] and refused[0].reason == "vtbl_owner_not_parenthesised"


def test_vtbl_owner_parenthesised_but_not_dereferenced_is_REFUSED():
    src = "    (*(psf).lpVtbl).Release();\n"
    s, e = at(src, "Release")
    edits, refused = IA.propose(
        [com_diag("Release", "IShellFolderVtbl", "a.rs", 1, s, e)], reader(src), interfaces=IFACES
    )
    assert edits == [] and refused[0].reason == "vtbl_owner_not_a_deref"


def test_written_out_vtbl_multiline_args_get_no_trailing_whitespace():
    src = "    (*(*psf).lpVtbl).GetUIObjectOf(\n        cidl,\n    );\n"
    s, e = at(src, "GetUIObjectOf")
    edits, refused = IA.propose(
        [com_diag("GetUIObjectOf", "IShellFolderVtbl", "a.rs", 1, s, e)],
        reader(src),
        interfaces=IFACES,
    )
    assert refused == []
    out = src[: edits[0].start] + edits[0].replacement + src[edits[0].end :]
    assert "psf,\n" in out and " \n" not in out


def test_written_out_vtbl_that_already_passes_this_does_NOT_get_a_second_one():
    """The bug this guard exists to not repeat: `(...Release)(p, p)` compiled, and only an
    arity error two layers down said anything was wrong."""
    src = "    (*(*spFolder).lpVtbl).Release(spFolder);\n"
    s, e = at(src, "Release")
    edits, refused = IA.propose(
        [com_diag("Release", "IShellFolderVtbl", "a.rs", 1, s, e)], reader(src), interfaces=IFACES
    )
    assert refused == []
    out = src[: edits[0].start] + edits[0].replacement + src[edits[0].end :]
    assert out == "    ((*(*spFolder).lpVtbl).Release)(spFolder);\n"


def test_written_out_vtbl_with_this_plus_more_args_keeps_the_rest():
    src = "    (*(*psf).lpVtbl).GetAttributesOf(psf, cidl, apidl);\n"
    s, e = at(src, "GetAttributesOf")
    edits, refused = IA.propose(
        [com_diag("GetAttributesOf", "IShellFolderVtbl", "a.rs", 1, s, e)],
        reader(src),
        interfaces=IFACES,
    )
    assert refused == []
    out = src[: edits[0].start] + edits[0].replacement + src[edits[0].end :]
    assert out == "    ((*(*psf).lpVtbl).GetAttributesOf)(psf, cidl, apidl);\n"


def test_a_first_argument_that_merely_starts_with_the_object_is_not_mistaken_for_this():
    """`psfChild` is not `psf`; the guard must compare the whole argument."""
    src = "    (*(*psf).lpVtbl).Release(psfChild);\n"
    s, e = at(src, "Release")
    edits, _ = IA.propose(
        [com_diag("Release", "IShellFolderVtbl", "a.rs", 1, s, e)], reader(src), interfaces=IFACES
    )
    out = src[: edits[0].start] + edits[0].replacement + src[edits[0].end :]
    assert out == "    ((*(*psf).lpVtbl).Release)(psf, psfChild);\n"


def test_first_arg_stops_at_a_top_level_comma_not_one_inside_a_nested_call():
    src = "    (*(*psf).lpVtbl).GetUIObjectOf(f(a, b), c);\n"
    s, e = at(src, "GetUIObjectOf")
    edits, _ = IA.propose(
        [com_diag("GetUIObjectOf", "IShellFolderVtbl", "a.rs", 1, s, e)],
        reader(src),
        interfaces=IFACES,
    )
    out = src[: edits[0].start] + edits[0].replacement + src[edits[0].end :]
    assert out == "    ((*(*psf).lpVtbl).GetUIObjectOf)(psf, f(a, b), c);\n"
