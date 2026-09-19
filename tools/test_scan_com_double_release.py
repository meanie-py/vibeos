#!/usr/bin/env python3
"""Tests for scan_com_double_release — each case is a shape seen in the tree."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from scan_com_double_release import scan_source


def _kinds(src: str):
    return [k for _, k, _ in scan_source(src)]


def test_guard_plus_unnulled_release_is_a_finding():
    """CDefView::FillList, the shape that killed explorer on arm64."""
    src = """
pub unsafe fn CDefView_FillList_body(this: *mut CDefView) -> HRESULT {
    let mut pEnumIDList: *mut IEnumIDList = core::ptr::null_mut();
    let _pEnumIDList_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pEnumIDList));
    if !pEnumIDList.is_null() {
        ((*(*pEnumIDList).lpVtbl).Release)(pEnumIDList);
    }
    return S_OK;
}
"""
    assert _kinds(src) == ["com-double-release"]


def test_release_then_null_is_correct():
    """CDefView::Exec nulls the slot, so the guard sees NULL and does nothing."""
    src = """
pub unsafe fn CDefView_Exec_body(this: *mut CDefView) -> HRESULT {
    let mut pStream: *mut IStream = core::ptr::null_mut();
    let _pStream_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pStream));
    if !pStream.is_null() {
        ((*(*pStream).lpVtbl).Release)(pStream);
        pStream = core::ptr::null_mut();
    }
    return S_OK;
}
"""
    assert _kinds(src) == []


def test_hand_over_hand_walk_is_correct():
    """CMenuDeskBar::_IsSubMenuParent releases the current node and takes the next.

    The slot is REASSIGNED rather than nulled, so the guard goes on to own the new
    node. An earlier cut of this scanner tested for NULL specifically and reported
    this correct loop.
    """
    src = """
pub unsafe fn CMenuDeskBar__IsSubMenuParent_body(hwnd: HWND) -> BOOL {
    let _popup_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(popup));
    while !popup.is_null() {
        ((*(*popup).lpVtbl).Release)(popup);
        popup = next;
    }
    FALSE
}
"""
    assert _kinds(src) == []


def test_release_without_a_guard_is_correct():
    """SaveViewState's pPB/pStream are raw locals the C++ really does release."""
    src = """
pub unsafe fn CDefView_SaveViewState_helper_body(this: *mut CDefView) -> HRESULT {
    let mut pPB: *mut IPropertyBag = core::ptr::null_mut();
    ((*(*pPB).lpVtbl).Release)(pPB);
    return S_OK;
}
"""
    assert _kinds(src) == []


def test_guard_and_release_in_DIFFERENT_functions_is_correct():
    """The reason matching is by enclosing function, not by variable name.

    A file-wide grep on the name alone pairs LoadViewState's guarded `pPB` with
    SaveViewState's raw `pPB` and reports correct code.
    """
    src = """
pub unsafe fn CDefView_LoadViewState_body(this: *mut CDefView) -> HRESULT {
    let _pPB_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pPB));
    return S_OK;
}
pub unsafe fn CDefView_SaveViewState_helper_body(this: *mut CDefView) -> HRESULT {
    ((*(*pPB).lpVtbl).Release)(pPB);
    return S_OK;
}
"""
    assert _kinds(src) == []


def test_release_of_a_DIFFERENT_pointer_is_not_a_finding():
    """`a->Release(b)` is not `b`'s release; the backreference keeps them apart."""
    src = """
pub unsafe fn f() -> HRESULT {
    let _pa_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pa));
    ((*(*pa).lpVtbl).Release)(pb);
    return S_OK;
}
"""
    assert _kinds(src) == []
