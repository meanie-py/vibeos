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


def test_every_deref_is_reported_not_just_the_first() -> None:
    """The scanner used to `break` at the first hit per probe.

    Effect: fixing one site "revealed" the next, so the tool looked like it kept
    finding new bugs and its total was always an undercount. Same failure shape as
    the is_comment bug that hid scrollex.rs -- an honest number or none at all.
    """
    src = """
pub unsafe extern "system" fn Three(pmsg: *mut MSG) -> BOOL {
    let Status = SafeProbeForWrite(pmsg as *mut c_void, size_of::<MSG>(), 1);
    (*pmsg).message = 1;
    (*pmsg).wParam = 2;
    (*pmsg).lParam = 3;
}
"""
    assert len(leads(src)) == 3


def test_reassignment_to_a_kernel_buffer_ends_the_taint() -> None:
    """dibobj copies the user BITMAPINFO into a pool block and rebinds the name.

    Every later `(*bmi).…` is then a kernel load. Reported as a lead until the
    scanner learned that an unrelated reassignment ends the tracking.
    """
    src = """
pub unsafe extern "system" fn Copied(bmi: *mut BITMAPINFO) -> BOOL {
    let Status = SafeProbeForRead(bmi as *const c_void, cj, 1);
    crate::copy::capture_from_user(pbmiSafe as *mut c_void, bmi as *const c_void, cj);
    bmi = pbmiSafe;
    let h = (*bmi).bmiHeader.biHeight;
}
"""
    assert leads(src) == []


def test_the_reassigned_name_is_matched_on_a_WORD_boundary() -> None:
    """`bmi = pbmiSafe;` -- "bmi" is a substring of "pbmiSafe".

    A substring test read that as self-reference and kept the taint alive, so the
    rule above silently did nothing on the one site it was written for.
    """
    src = """
pub unsafe extern "system" fn Rebound(p: *mut RECT) -> BOOL {
    let Status = SafeProbeForRead(p as *const c_void, 16, 1);
    p = pSafe;
    let v = (*p).left;
}
"""
    assert leads(src) == []


def test_nulling_the_pointer_does_NOT_end_the_taint() -> None:
    """Error paths null the pointer; the success path keeps using the original.

    Treating a NULL assignment as a rebind hid the real dwHookData leads that sat
    two lines below exactly such an error path.
    """
    src = """
pub unsafe extern "system" fn Nulled(ClientInfo: PCLIENTINFO) -> BOOL {
    let Status = SafeProbeForWrite(ClientInfo as *mut c_void, 64, 1);
    if !NT_SUCCESS(Status) {
        ClientInfo = core::ptr::null_mut();
    }
    let v = (*ClientInfo).dwHookData;
}
"""
    assert len(leads(src)) == 1


def test_a_bulk_copy_from_the_probed_pointer_is_a_lead() -> None:
    """`RtlCopyMemory(&mut local, p, n)` has no `*p` in it at all.

    It is still a plain memcpy off a probed-only user pointer, i.e. the same fault.
    dibobj shipped exactly this, one line above a site this scanner had already
    (wrongly) written off as a false positive.
    """
    src = """
pub unsafe extern "system" fn Bulk(pdsm: *const DOSENDMESSAGE) -> BOOL {
    let Status = SafeProbeForRead(pdsm as *const c_void, 32, 1);
    RtlCopyMemory(&mut dsm as *mut _ as *mut c_void, pdsm as *const c_void, 32);
}
"""
    assert len(leads(src)) == 1


def test_a_bulk_copy_is_matched_against_that_calls_OWN_arguments() -> None:
    """Three back-to-back copies must not each be blamed for the others' pointers.

    A line-window match reported stretchblt's `prclDest` copy as a lead for
    `prclSrc` and `MaskOrigin` too -- 3x the true count on one of the busiest files.
    """
    src = """
pub unsafe extern "system" fn Three(prclDest: *const RECTL, prclSrc: *const RECTL) -> BOOL {
    let Status = SafeProbeForRead(prclDest as *const c_void, 16, 1);
    let Status = SafeProbeForRead(prclSrc as *const c_void, 16, 1);
    RtlCopyMemory(&mut rclDest as *mut c_void, prclDest as *const c_void, 16);
    RtlCopyMemory(&mut rclSrc as *mut c_void, prclSrc as *const c_void, 16);
}
"""
    got = leads(src)
    assert len(got) == 2, got


def test_an_is_null_guard_is_not_a_use() -> None:
    """`if !p.is_null() { &SafeLocal } else { null }` passes a KERNEL local.

    The probed name appears only in the test. Three of the first four
    interprocedural hits were this shape -- all false.
    """
    src = """
pub unsafe extern "system" fn Guarded(prc: *const RECTL) -> BOOL {
    let Status = SafeProbeForRead(prc as *const c_void, 16, 1);
    crate::copy::capture_from_user(&mut Safe as *mut c_void, prc as *const c_void, 16);
    Result = UserScrollDC(hDC, if !prc.is_null() { &Safe } else { core::ptr::null() });
}
"""
    assert leads(src) == []
