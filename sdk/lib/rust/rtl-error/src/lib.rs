//! Rust translation of `sdk/lib/rtl/error.c` — the NTSTATUS→DOS Win32 error
//! mapping (`RtlNtStatusToDosError[NoTeb]`), the last-error / last-status TEB
//! accessors, and the per-thread error mode.
//!
//! The data is generated deterministically from the SDK, not the LLM (which
//! silently drops entries from 400-element tables):
//!   * `winerr.rs`  — the ~696 `ERROR_*`/`STATUS_*`/`STG_E_*`/… constants the
//!     tables reference, resolved from winerror.h + ntstatus.h
//!     (tools/gen_winerror_consts.py).
//!   * `tables.rs`  — the 19 status→code tables + the dispatch array, extracted
//!     verbatim from error.c with verified element counts
//!     (tools/gen_error_tables.py).
//! Only the small dispatch functions below come from the translation.
//! Shared kernel+user via librtl_rust.a.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]
#![allow(unused_variables)] // RtlMapSecurityErrorToNtStatus(SecurityError) stub, faithful to C

pub type DWORD = u32;

mod tables;
mod winerr;
use tables::ERROR_TABLE; // the dispatch array (generated from error.c)
use winerr::*; // ERROR_MR_MID_NOT_FOUND etc. (the rest feed tables.rs)

use ntkrnl_sys::base::*; // ULONG, NTSTATUS, PULONG, STATUS_SUCCESS
use ntkrnl_sys::constants::STATUS_INVALID_PARAMETER_1;
use ntkrnl_sys::ddk::PTEB;

// NtCurrentTeb is an inline segment-register read, NOT a linkable export — the
// bindgen `ddk::NtCurrentTeb` extern has no symbol (undefined at link). Use the
// inline helper (as rtl-ppb/rtl-rxact do).
#[inline]
unsafe fn NtCurrentTeb() -> PTEB {
    ntkrnl_sys::inline::ke::NtCurrentTeb() as PTEB
}

// Fixed ntstatus.h / winnt.h values not surfaced by ntkrnl-sys.
const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC000_0002_u32 as i32;
// ndk/rtltypes.h:74-76: the SEM_* bits shifted left by 4 (TEB HardErrorMode form).
const RTL_SEM_FAILCRITICALERRORS: ULONG = 0x0000_0010;
const RTL_SEM_NOGPFAULTERRORBOX: ULONG = 0x0000_0020;
const RTL_SEM_NOALIGNMENTFAULTEXCEPT: ULONG = 0x0000_0040;

#[inline]
fn HIWORD(l: DWORD) -> DWORD {
    (l >> 16) & 0xffff
}
#[inline]
fn LOWORD(l: DWORD) -> DWORD {
    l & 0xffff
}

/// Dispatch table entry: maps the half-open NTSTATUS range `[start, end)` to a
/// dense array of Win32 codes. Read-only static data → safe to share.
#[repr(C)]
pub struct error_table {
    pub start: DWORD,
    pub end: DWORD,
    pub table: *const DWORD,
}
unsafe impl Sync for error_table {}

#[no_mangle]
pub unsafe extern "system" fn RtlNtStatusToDosErrorNoTeb(Status: NTSTATUS) -> ULONG {
    let mut Status = Status;
    let mut table: *const error_table = ERROR_TABLE.as_ptr();

    if Status == 0 || (Status & 0x20000000) != 0 {
        return Status as ULONG;
    }

    if (Status as u32 & 0xf0000000) == 0xd0000000 {
        Status &= !0x10000000;
    }

    'no_mapping: loop {
        while (*table).start != 0 {
            if (Status as ULONG) < (*table).start {
                break;
            }
            if (Status as ULONG) < (*table).end {
                let ret = *(*table).table.offset((Status as ULONG - (*table).start) as isize);
                if ret == 0 {
                    break 'no_mapping;
                }
                return ret;
            }
            table = table.offset(1);
        }

        if HIWORD(Status as DWORD) == 0xc001 {
            return LOWORD(Status as DWORD) as ULONG;
        }
        if HIWORD(Status as DWORD) == 0x8007 {
            return LOWORD(Status as DWORD) as ULONG;
        }

        break 'no_mapping;
    }

    ERROR_MR_MID_NOT_FOUND
}

#[no_mangle]
pub unsafe extern "system" fn RtlNtStatusToDosError(Status: NTSTATUS) -> ULONG {
    let Teb: PTEB = NtCurrentTeb();

    if !Teb.is_null() {
        (*Teb).LastStatusValue = Status as ULONG;
    }
    RtlNtStatusToDosErrorNoTeb(Status)
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetLastNtStatus() -> NTSTATUS {
    (*NtCurrentTeb()).LastStatusValue as NTSTATUS
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetLastWin32Error() -> ULONG {
    (*NtCurrentTeb()).LastErrorValue
}

#[no_mangle]
pub unsafe extern "system" fn RtlSetLastWin32Error(LastError: ULONG) {
    (*NtCurrentTeb()).LastErrorValue = LastError;
}

#[no_mangle]
pub unsafe extern "system" fn RtlSetLastWin32ErrorAndNtStatusFromNtStatus(Status: NTSTATUS) {
    (*NtCurrentTeb()).LastErrorValue = RtlNtStatusToDosError(Status);
}

#[no_mangle]
pub unsafe extern "system" fn RtlMapSecurityErrorToNtStatus(SecurityError: ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn RtlSetThreadErrorMode(NewMode: ULONG, OldMode: PULONG) -> NTSTATUS {
    let Teb: PTEB = NtCurrentTeb();

    if NewMode
        & !(RTL_SEM_FAILCRITICALERRORS | RTL_SEM_NOGPFAULTERRORBOX | RTL_SEM_NOALIGNMENTFAULTEXCEPT)
        != 0
    {
        return STATUS_INVALID_PARAMETER_1;
    }

    if !OldMode.is_null() {
        *OldMode = (*Teb).HardErrorMode;
    }

    (*Teb).HardErrorMode = NewMode;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetThreadErrorMode() -> ULONG {
    (*NtCurrentTeb()).HardErrorMode
}
