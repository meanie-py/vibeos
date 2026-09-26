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
    """Local-only scan, as the single-function tests intend."""
    return S.scan_text(text, "t.rs")


def leads_interproc(text: str):
    """Scan WITH the callee index, the way main() does.

    The interprocedural pass needs `deref_params` over the same text; calling
    scan_text bare silently skips it, so a test written without this would pass
    against a scanner that had no interprocedural detection at all.
    """
    return S.scan_text(text, "t.rs", S.deref_params(text))


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


def test_a_param_consumed_by_a_BULK_COPY_in_the_callee_is_a_lead() -> None:
    """The NtGdiCreateBitmap shape, which shipped a live BUGCHECK 0x1E.

    The probe is in the syscall; the access is an RtlCopyMemory one frame down, inside
    a helper, on the parameter the syscall handed it. Neither the deref-shaped search
    (there is no `*p`) nor the interprocedural pass on its own (it only looked for
    derefs of the parameter) could see it. An apitest found it instead, at
    NtGdiCreateBitmap+0x1F9 -> memcpy -> raw_move with 0xC0000005.
    """
    src = """
pub unsafe extern "system" fn Helper(dst: PUCHAR, pvBits: *const c_void) -> LONG {
    RtlCopyMemory(dst as *mut c_void, pvBits as *const c_void, n);
}

pub unsafe extern "system" fn NtGdiThing(pUnsafeBits: LPBYTE) -> HBITMAP {
    let Status = SafeProbeForRead(pUnsafeBits as *const c_void, cjSize, 1);
    Helper(pjDst, pUnsafeBits as *const c_void);
}
"""
    assert leads(src) == [], "no local deref here — only the callee touches it"
    got = leads_interproc(src)
    assert got, "the bulk-copy-in-callee shape must be reported"
    assert any("Helper" in row[2] for row in got), got


def test_a_bulk_copy_callee_is_reported_once_not_twice() -> None:
    """RtlCopyMemory is in UNSAFE_COPIES *and* derefs its args, so both the direct
    and the interprocedural rule fired on the same site. 281 rows for 165 real
    sites -- and the inflated 281 got quoted as the size of the remaining work."""
    src = """
pub unsafe fn f(p: *mut u8) {
    SafeProbeForRead(p as _, 8, 1);
    RtlCopyMemory(dst, p, 8);
}
"""
    rows = leads_interproc(src)
    assert len(rows) == 1, rows
    # and the one kept is the line itself, which is what you need to fix it
    assert "RtlCopyMemory" in rows[0][2]
    assert "passed to" not in rows[0][2]


def test_a_project_callee_is_still_reported_interprocedurally() -> None:
    """The dedupe must not silence the case the rule exists for: a deref hidden
    behind a project function (the NtUserSetSysColors -> IntSetSysColors shape)."""
    src = """
pub unsafe fn IntSetSysColors(a: *const u32) {
    let _x = *a;
}
pub unsafe fn g(p: *const u32) {
    SafeProbeForRead(p as _, 8, 1);
    IntSetSysColors(p);
}
"""
    rows = leads_interproc(src)
    assert any("passed to IntSetSysColors()" in r[2] for r in rows), rows


def test_repointing_to_a_kernel_local_ends_the_taint() -> None:
    """`prcl = addr_of_mut!(rclLocal);` is the tree's idiom for the CORRECT fix:
    capture into a kernel local, then repoint the name at it. The `addr_of!` skip
    used to run BEFORE the taint-end check and `continue` past this very line, so
    the scanner kept reporting sites after they were fixed -- 10 of them."""
    src = """
pub unsafe fn UserClipCursor(a: *mut RECTL) -> BOOL { let _ = *a; 1 }
pub unsafe fn NtUserClipCursor(mut prcl: *mut RECTL) -> BOOL {
    let mut rclLocal: RECTL = core::mem::zeroed();
    if !prcl.is_null() {
        let st = SafeProbeForRead(prcl as *mut _, 16, 1);
        if !NT_SUCCESS(st) { return 0; }
        if !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(rclLocal) as *mut c_void,
            prcl as *const c_void,
            16,
        ) { return 0; }
        prcl = core::ptr::addr_of_mut!(rclLocal);
    }
    UserClipCursor(prcl)
}
"""
    assert leads_interproc(src) == []


def test_addr_of_on_a_field_is_still_not_a_load() -> None:
    """Reordering must not lose the rule it was moved past: taking the address of a
    probed struct's field computes, it does not load, so it cannot fault."""
    src = """
pub unsafe fn f(p: *mut FOO) {
    SafeProbeForRead(p as _, 8, 1);
    let q = core::ptr::addr_of!((*p).field);
}
"""
    assert leads_interproc(src) == []


def test_an_interlocked_on_a_probed_pointer_IS_a_lead() -> None:
    """The exception to the rule above, and the reason it is not a contradiction.

    `addr_of_mut!` is skipped because computing an address cannot fault. But here
    the address never escapes the line: `InterlockedIncrement` dereferences it, so
    the callee IS the access. Skipping the whole line for containing `addr_of_mut!`
    hid ime.rs's two live unprotected writes to user memory while the scanner
    reported the entire class clear.
    """
    src = """
pub unsafe fn f(p: *mut FOO) {
    SafeProbeForRead(p as _, 8, 1);
    InterlockedIncrement(core::ptr::addr_of_mut!((*p).nCount));
}
"""
    hits = leads_interproc(src)
    assert len(hits) == 1, hits
    assert "InterlockedIncrement" in hits[0][2]


def test_an_interlocked_on_an_UNPROBED_pointer_is_not_a_lead() -> None:
    """The rule is scoped to probed (user) pointers. Interlocked ops on kernel
    memory are the overwhelming majority in the tree, and flagging those would
    bury the two real ones in noise."""
    src = """
pub unsafe fn f(p: *mut FOO) {
    InterlockedIncrement(core::ptr::addr_of_mut!((*p).nCount));
}
"""
    assert leads_interproc(src) == []
