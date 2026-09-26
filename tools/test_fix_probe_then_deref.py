"""Each case here is output the fixer actually emitted before it was guarded.

The tool rewrites kernel syscall entry points, so a wrong rewrite is not a
compile error -- three of these four produce code that builds and is silently
wrong on the error path. They are pinned as tests rather than as comments for
that reason.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from fix_probe_then_deref import as_usize, as_void, find_probe_block, rewrite_site


def _at(src: str, probed: str, needle: str):
    lines = src.splitlines()
    site = next(i for i, ln in enumerate(lines) if needle in ln)
    return rewrite_site(lines, site, probed), lines, site


NEGATIVE = """
pub unsafe extern "system" fn NtUserThing(p: PMSG) -> BOOL {
    let __SehStatus = SafeProbeForRead(p as *mut _, 4, 1);
    if !NT_SUCCESS(__SehStatus) {
        SetLastNtError(__SehStatus);
        return 0;
    }
    RtlCopyMemory(
        core::ptr::addr_of_mut!(m) as *mut _,
        p as *const _,
        4,
    );
    1
}
"""


def test_negative_form_is_rewritten_and_replays_the_error_path():
    out, _, _ = _at(NEGATIVE, "p", "RtlCopyMemory(")
    assert out is not None
    text = out[1]
    assert "crate::copy::capture_from_user(" in text
    assert "return 0;" in text
    # The probe's status variable is out of scope here AND holds STATUS_SUCCESS,
    # so replaying `SetLastNtError(__SehStatus)` would report success.
    assert "__SehStatus" not in text
    assert "STATUS_ACCESS_VIOLATION" in text


POSITIVE = """
pub unsafe extern "system" fn NtUserThing(p: PVOID, n: u32) -> BOOL {
    Status = SafeProbeForRead(p, n as _, 1);
    if NT_SUCCESS(Status) {
        RtlCopyMemory(dst as *mut _, p as *const _, n as usize);
    }
    1
}
"""


def test_positive_form_is_refused():
    """`if NT_SUCCESS(s) { <copy> }` has no failure block -- its body IS the copy.

    Matching on the bare name proposed replaying the copy as its own error path.
    """
    out, _, _ = _at(POSITIVE, "p", "RtlCopyMemory(")
    assert out is None


EARLIER_BLOCK = """
pub unsafe extern "system" fn NtUserThing(psid: PVOID, size: u32) -> BOOL {
    'leave: {
        Status = SafeProbeForRead(other as *mut _, 4, 1);
        if !NT_SUCCESS(Status) {
            break 'leave;
        }
        Status = SafeProbeForRead(psid as *mut _, size as SIZE_T, 1);
        if NT_SUCCESS(Status) {
            RtlCopyMemory(dstbuf, psid, size as usize);
        }
        if !NT_SUCCESS(Status) {
            break 'leave;
        }
    }
    1
}
"""


def test_failure_block_of_an_earlier_probe_is_refused():
    """The wrong error path is not a smaller bug than no error path.

    NtUserSetWindowStationUser got a `break 'leave` belonging to an earlier
    probe, which skipped the `if !NT_SUCCESS(Status)` three lines BELOW the copy.
    """
    out, _, _ = _at(EARLIER_BLOCK, "psid", "RtlCopyMemory(dstbuf")
    assert out is None


STALE_STATUS = """
pub unsafe extern "system" fn NtUserThing(pOut: *mut u32) -> BOOL {
    'exit: {
        let Status = SafeProbeForWrite(pOut as *mut _, 4, 1);
        if !NT_SUCCESS(Status) {
            break 'exit;
        }
        Obj = ObReferenceObject(h);
        Status = STATUS_SUCCESS;
        core::ptr::write_volatile(pOut, 7u32);
    }
    1
}
"""


def test_reassigned_status_or_acquired_resource_is_refused():
    """NtUserGetObjectInformation is both, and replaying its transfer would have
    leaked two referenced objects AND returned success after a faulted write."""
    out, _, _ = _at(STALE_STATUS, "pOut", "write_volatile(pOut")
    assert out is None


MULTILINE_COND = """
pub unsafe extern "system" fn NtUserThing(lpsi: PVOID) -> BOOL {
    'exit: {
        if !NT_SUCCESS(SafeProbeForRead(
            lpsi as PVOID,
            core::mem::size_of::<SCROLLINFO>() as SIZE_T,
            1,
        )) {
            break 'exit;
        }
        RtlCopyMemory(
            core::ptr::addr_of_mut!(psi) as *mut _,
            lpsi as *const _,
            core::mem::size_of::<SCROLLINFO>() as _,
        );
    }
    1
}
"""


def test_multiline_condition_is_refused():
    """The brace scan starts below the check and scoops the probe's own ARGUMENTS
    up as the error body -- which is what happened to NtUserSBGetParms."""
    out, _, _ = _at(MULTILINE_COND, "lpsi", "RtlCopyMemory(")
    assert out is None


def test_reference_keeps_its_intermediate_cast():
    """`&mut ca as *mut c_void` is E0606; the reference supplies the pointee for
    an intermediate `*mut _`, a raw pointer does not."""
    assert as_void("&mut ca as *mut _", True) == "&mut ca as *mut _ as *mut core::ffi::c_void"
    assert as_void("p as *mut _", True) == "p as *mut core::ffi::c_void"


def test_no_redundant_usize_cast():
    """clippy flags `size_of::<T>() as usize` and `1 as usize`."""
    assert as_usize("core::mem::size_of::<MSG>() as _") == "core::mem::size_of::<MSG>()"
    assert as_usize("1") == "1"
    assert as_usize("cbSize as _") == "cbSize as usize"


def test_find_probe_block_reports_the_status_variable():
    lines = NEGATIVE.splitlines()
    site = next(i for i, ln in enumerate(lines) if "RtlCopyMemory(" in ln)
    blk = find_probe_block(lines, site)
    assert blk is not None
    assert blk[2] == "__SehStatus"
