"""Oracle for the probe-then-raw-deref scanner.

The scanner reports LEADS, so its own failure modes are what matter: a false
positive on every correct probe would bury the real ones, and a false negative
on the known-crashing shape would make it decoration.
"""

from __future__ import annotations

import importlib.util
from pathlib import Path

_spec = importlib.util.spec_from_file_location(
    "scan_probe_then_deref", Path(__file__).resolve().parent / "scan_probe_then_deref.py"
)
assert _spec and _spec.loader
S = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(S)


def leads(text: str):
    return S.scan_text(text, "t.rs")


def test_the_shape_that_actually_bugchecked_is_reported() -> None:
    """NtUserScrollDC: probe the RECT, then load it inline."""
    src = """
pub unsafe extern "system" fn NtUserScrollDC(prc: *const RECT) -> BOOL {
    let Status = SafeProbeForRead(prc as *const c_void, size_of::<RECT>(), 1);
    rcScroll = *(prc as *const RECTL);
}
"""
    got = leads(src)
    assert len(got) == 1, got
    assert got[0][1] == "prc"


def test_a_capture_through_the_fault_tolerant_copy_is_clean() -> None:
    """The fix must not still be reported, or nobody will fix anything."""
    src = """
pub unsafe extern "system" fn Ok1(prc: *const RECT) -> BOOL {
    let Status = SafeProbeForRead(prc as *const c_void, size_of::<RECT>(), 1);
    KmProtectedCopyMemory(&mut rcScroll as *mut _, prc as *mut _, size_of::<RECT>());
}
"""
    assert leads(src) == []


def test_the_commented_out_C_is_not_a_lead() -> None:
    """These files keep the original C as a comment block — the exact text."""
    src = """
pub unsafe extern "system" fn Ok2(prc: *const RECT) -> BOOL {
    let Status = SafeProbeForRead(prc as *const c_void, size_of::<RECT>(), 1);
    // rcScroll = *prc;
}
"""
    assert leads(src) == []


def test_addr_of_is_not_a_dereference() -> None:
    """It computes an address without loading, and the probes use it themselves."""
    src = """
pub unsafe extern "system" fn Ok3(bmi: *mut BITMAPINFO) -> HBITMAP {
    let Status = SafeProbeForRead(
        core::ptr::addr_of!((*bmi).bmiHeader.biSize) as *const c_void, 4, 1);
}
"""
    assert leads(src) == []


def test_a_probe_of_one_pointer_does_not_excuse_a_DIFFERENT_one() -> None:
    src = """
pub unsafe extern "system" fn Two(a: *const RECT, b: *const RECT) -> BOOL {
    let Status = SafeProbeForRead(a as *const c_void, 16, 1);
    let x = *a;
    let y = *b;
}
"""
    got = leads(src)
    assert [g[1] for g in got] == ["a"]


def test_a_STORE_at_the_start_of_a_line_is_not_a_comment() -> None:
    """The false negative that hid a live BUGCHECK.

    `is_comment` treated any line starting with `*` as a block-comment
    continuation, which swallowed every statement of the form
    `*(p as *mut T) = v;` -- the write-back shape. The scanner reported
    scrollex.rs clean while NtUserScrollDC was still bugchecking on exactly
    that line, and I believed it.
    """
    src = """
pub unsafe extern "system" fn W(prc: *mut RECT) -> BOOL {
    let Status = SafeProbeForWrite(prc as *mut c_void, size_of::<RECT>(), 1);
    *(prc as *mut RECTL) = rcUpdate;
}
"""
    got = leads(src)
    assert len(got) == 1, got
    assert got[0][1] == "prc"


def test_a_real_block_comment_continuation_is_still_a_comment() -> None:
    """The converse the original guard was for -- keep it working."""
    assert S.is_comment(" * rcScroll = *prc;")
    assert S.is_comment(" */")
    assert S.is_comment("*")
    assert not S.is_comment("*(prc as *mut RECTL) = v;")
    assert not S.is_comment("*prc = v;")


def test_the_fix_helpers_count_as_safe_sinks() -> None:
    """Or every site already fixed reports itself forever."""
    src = """
pub unsafe extern "system" fn Fixed(prc: *mut RECT) -> BOOL {
    let Status = SafeProbeForWrite(prc as *mut c_void, size_of::<RECT>(), 1);
    crate::copy::publish_to_user(prc as *mut c_void, x, 16);
}
"""
    assert leads(src) == []
