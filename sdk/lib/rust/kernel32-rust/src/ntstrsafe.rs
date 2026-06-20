//! ntstrsafe.rs — faithful Rust bodies for the ntstrsafe.h FORCEINLINE string
//! routines that the C translation referenced as `extern` but which have no
//! exported symbol (they are `FORCEINLINE` in sdk/include/ddk/ntstrsafe.h).
//!
//! The Rust callers (compname.rs, find.rs) declared them `extern "system" fn`, so
//! they reference the i386 stdcall-decorated `_Name@N`; we provide real
//! `#[no_mangle] extern "system"` definitions with the exact ntstrsafe semantics.

use ntdll_sys::nt::NTSTATUS;

const NTSTRSAFE_MAX_CCH: usize = 2147483647; // 0x7FFFFFFF
const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000000Du32 as NTSTATUS;
const STATUS_BUFFER_OVERFLOW: NTSTATUS = 0x80000005u32 as NTSTATUS;

#[inline]
fn nt_success(s: NTSTATUS) -> bool {
    s >= 0
}

/// ntstrsafe.h `RtlStringCbPrintfW` result mapping → NTSTATUS, given the
/// `_snwprintf`/`_vsnwprintf` return `written` and the dest **byte** capacity
/// `cb_dest`. ntstrsafe's `RtlStringCbPrintfW` is FORCEINLINE (no exported
/// symbol) and is just `_vsnwprintf(dest, cchDest-1, fmt, args)` + NUL-handling;
/// callers invoke `_snwprintf` directly (same crate idiom as client/except.rs's
/// StringCchPrintfW) and pass the return here. Mirrors StringCbVPrintfWorker:
/// guarantee a NUL, STATUS_BUFFER_OVERFLOW on truncation, STATUS_INVALID_PARAMETER
/// when there isn't even room for a NUL.
#[inline]
pub unsafe fn rtl_strcbprintf_w_status(written: i32, cb_dest: usize, dest: *mut u16) -> NTSTATUS {
    let cch_dest = cb_dest / core::mem::size_of::<u16>();
    if cch_dest == 0 {
        return STATUS_INVALID_PARAMETER;
    }
    let cch_max = cch_dest - 1;
    if written < 0 || (written as usize) > cch_max {
        // overflow/truncation: NUL-terminate at the end of the buffer
        *dest.add(cch_max) = 0;
        STATUS_BUFFER_OVERFLOW
    } else {
        // _snwprintf does not NUL-terminate when the output exactly fills count
        if (written as usize) == cch_max {
            *dest.add(cch_max) = 0;
        }
        STATUS_SUCCESS
    }
}

/// RtlStringLengthWorkerW (ntstrsafe.h) — count up to `cch_max` WCHARs.
#[inline]
unsafe fn length_worker_w(
    mut psz: *const u16,
    mut cch_max: usize,
    pcch_length: *mut usize,
) -> NTSTATUS {
    let mut status = STATUS_SUCCESS;
    let cch_max_prev = cch_max;

    while cch_max != 0 && *psz != 0 {
        psz = psz.add(1);
        cch_max -= 1;
    }

    if cch_max == 0 {
        status = STATUS_INVALID_PARAMETER;
    }

    if !pcch_length.is_null() {
        if nt_success(status) {
            *pcch_length = cch_max_prev - cch_max;
        } else {
            *pcch_length = 0;
        }
    }

    status
}

/// RtlStringCopyNWorkerW (ntstrsafe.h) — bounded copy with guaranteed NUL.
#[inline]
unsafe fn copy_n_worker_w(
    mut psz_dest: *mut u16,
    mut cch_dest: usize,
    mut psz_src: *const u16,
    mut cch_to_copy: usize,
) -> NTSTATUS {
    let mut status = STATUS_SUCCESS;

    if cch_dest == 0 {
        status = STATUS_INVALID_PARAMETER;
    } else {
        while cch_dest != 0 && cch_to_copy != 0 && *psz_src != 0 {
            *psz_dest = *psz_src;
            psz_dest = psz_dest.add(1);
            psz_src = psz_src.add(1);
            cch_dest -= 1;
            cch_to_copy -= 1;
        }

        if cch_dest == 0 {
            psz_dest = psz_dest.sub(1);
            status = STATUS_BUFFER_OVERFLOW;
        }

        *psz_dest = 0;
    }

    status
}

/// RtlStringCchLengthW (ntstrsafe.h).
#[no_mangle]
pub unsafe extern "system" fn RtlStringCchLengthW(
    psz: *const u16,
    cch_max: usize,
    pcch_length: *mut usize,
) -> NTSTATUS {
    let status = if psz.is_null() || cch_max > NTSTRSAFE_MAX_CCH {
        STATUS_INVALID_PARAMETER
    } else {
        length_worker_w(psz, cch_max, pcch_length)
    };

    if !nt_success(status) && !pcch_length.is_null() {
        *pcch_length = 0;
    }

    status
}

/// RtlStringCchCopyNW (ntstrsafe.h) — char-count bounded copy.
#[no_mangle]
pub unsafe extern "system" fn RtlStringCchCopyNW(
    psz_dest: *mut u16,
    cch_dest: usize,
    psz_src: *const u16,
    cch_to_copy: usize,
) -> NTSTATUS {
    if cch_dest > NTSTRSAFE_MAX_CCH || cch_to_copy > NTSTRSAFE_MAX_CCH {
        if cch_dest > 0 {
            *psz_dest = 0;
        }
        return STATUS_INVALID_PARAMETER;
    }

    copy_n_worker_w(psz_dest, cch_dest, psz_src, cch_to_copy)
}

/// RtlStringCbCopyNW (ntstrsafe.h) — byte-count bounded copy (converts to chars).
#[no_mangle]
pub unsafe extern "system" fn RtlStringCbCopyNW(
    psz_dest: *mut u16,
    cb_dest: usize,
    psz_src: *const u16,
    cb_to_copy: usize,
) -> NTSTATUS {
    let cch_dest = cb_dest / core::mem::size_of::<u16>();
    let cch_to_copy = cb_to_copy / core::mem::size_of::<u16>();

    if cch_dest > NTSTRSAFE_MAX_CCH || cch_to_copy > NTSTRSAFE_MAX_CCH {
        if cch_dest > 0 {
            *psz_dest = 0;
        }
        return STATUS_INVALID_PARAMETER;
    }

    copy_n_worker_w(psz_dest, cch_dest, psz_src, cch_to_copy)
}
