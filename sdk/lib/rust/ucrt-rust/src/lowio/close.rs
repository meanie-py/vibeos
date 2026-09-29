//! `lowio/close.cpp`: `_close`, `_close_nolock` and their PTD-propagating
//! `_internal` forms (`dup2.cpp` and stdio's `fclose` call `_close_nolock_internal`
//! / `_close_internal`).
//!
//! Details that are silent if wrong:
//! * When fh 1 and fh 2 share one OS handle (both open), closing either does
//!   NOT close the OS handle -- the other still needs it -- and succeeds.
//! * The record is freed (`_free_osfhnd`, `_osfile = 0`) even when
//!   `CloseHandle` fails; the failure is reported afterwards.
//! * A record whose OS handle is already `INVALID_HANDLE_VALUE` closes
//!   successfully without touching the OS.

use super::common::*;
use super::{_free_osfhnd, _get_osfhandle, osfile, pioinfo, with_fh_locked, FOPEN, INVALID_HANDLE_VALUE};
use crate::ptd_host::PtdHost;
use core::ffi::{c_int, c_void};
use core::ptr::null_mut;

extern "system" {
    fn CloseHandle(handle: *mut c_void) -> i32;
}

unsafe fn close_os_handle_nolock(fh: c_int) -> u32 {
    if _get_osfhandle(fh) == INVALID_HANDLE_VALUE {
        return 0;
    }
    // If both stdout and stderr are open and refer to the same handle, the
    // handle is not closed: it is still in use by the other.
    let is_other_std_handle_open = (fh == 1 && osfile(2) & FOPEN != 0) || (fh == 2 && osfile(1) & FOPEN != 0);
    if is_other_std_handle_open && _get_osfhandle(1) == _get_osfhandle(2) {
        return 0;
    }
    if CloseHandle(_get_osfhandle(fh) as *mut c_void) != 0 {
        return 0;
    }
    GetLastError()
}

#[no_mangle]
pub unsafe extern "C" fn _close_internal(fh: c_int, ptd: *mut PtdHost) -> c_int {
    let ptd = &mut *ptd;
    if !ptd_validate_fh(ptd, fh) {
        return -1;
    }
    with_fh_locked(fh, || {
        if osfile(fh) & FOPEN != 0 {
            _close_nolock_internal(fh, ptd)
        } else {
            ptd.errno().set(EBADF);
            -1
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn _close(fh: c_int) -> c_int {
    let mut ptd = PtdHost::new(null_mut());
    _close_internal(fh, &mut ptd)
}

#[no_mangle]
pub unsafe extern "C" fn _close_nolock_internal(fh: c_int, ptd: *mut PtdHost) -> c_int {
    let close_os_handle_error = close_os_handle_nolock(fh);

    _free_osfhnd(fh);
    (*pioinfo(fh)).osfile = 0;

    if close_os_handle_error != 0 {
        __acrt_errno_map_os_error_ptd(close_os_handle_error, ptd);
        return -1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _close_nolock(fh: c_int) -> c_int {
    let mut ptd = PtdHost::new(null_mut());
    _close_nolock_internal(fh, &mut ptd)
}
