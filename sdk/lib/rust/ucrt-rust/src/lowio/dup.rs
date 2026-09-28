//! `lowio/dup.cpp` and `lowio/dup2.cpp`: `_dup` and `_dup2`.
//!
//! Details that are silent if wrong:
//! * The duplicate is made INHERITABLE (`bInheritHandle = TRUE`) and its
//!   `_osfile` drops `FNOINHERIT` -- whatever the source was.
//! * `textmode` and the `unicode` bit are copied; `utf8translations`,
//!   `dbcsBufferUsed`, the pipe lookahead and `startpos` are NOT.
//! * `_dup` allocates the new handle (returned locked) BEFORE duplicating; if
//!   the duplication fails its `FOPEN` is cleared again before unlocking.
//! * `_dup2` duplicates the OS handle FIRST and only then closes an open
//!   target, so a failed duplication leaves the target open.
//! * `_dup2` grows the table to reach `target_fh` (any value below
//!   `_NHANDLE_`), and `source == target` returns 0 without locking anything.
//!   The two locks are always taken lowest fh first.

use super::common::*;
use super::{
    __acrt_lowio_ensure_fh_exists, __acrt_lowio_lock_fh, __acrt_lowio_set_os_handle, __acrt_lowio_unlock_fh,
    _alloc_osfhnd, _get_osfhandle, osfile, pioinfo, with_fh_locked, HandleData, EMFILE, FNOINHERIT, FOPEN, NHANDLE,
    _nhandle,
};
use super::close::_close_nolock_internal;
use crate::ptd_host::PtdHost;
use core::ffi::{c_int, c_void};
use core::ptr::null_mut;

const DUPLICATE_SAME_ACCESS: u32 = 0x2;

extern "system" {
    fn GetCurrentProcess() -> *mut c_void;
    fn DuplicateHandle(
        source_process: *mut c_void,
        source_handle: *mut c_void,
        target_process: *mut c_void,
        target_handle: *mut *mut c_void,
        desired_access: u32,
        inherit_handle: c_int,
        options: u32,
    ) -> c_int;
}

/// Copies the parts of the handle record a duplicate inherits.
unsafe fn copy_mode(target: c_int, source: c_int) {
    let src = pioinfo(source);
    let dst = pioinfo(target);
    (*dst).osfile = (*src).osfile & !FNOINHERIT;
    (*dst).textmode = (*src).textmode;
    let unicode = (*src).unicode();
    (*dst).set_bit(HandleData::BIT_UNICODE, unicode);
}

unsafe fn duplicate_osfhnd(fh: c_int, new_fh: c_int, ptd: &mut PtdHost) -> c_int {
    let mut new_osfhandle: *mut c_void = null_mut();
    let result = DuplicateHandle(
        GetCurrentProcess(),
        _get_osfhandle(fh) as *mut c_void,
        GetCurrentProcess(),
        &mut new_osfhandle,
        0,
        1,
        DUPLICATE_SAME_ACCESS,
    );
    if result == 0 {
        __acrt_errno_map_os_error_ptd(GetLastError(), ptd);
        return -1;
    }
    __acrt_lowio_set_os_handle(new_fh, new_osfhandle as isize);
    copy_mode(new_fh, fh);
    new_fh
}

unsafe fn dup_nolock_internal(fh: c_int, ptd: &mut PtdHost) -> c_int {
    if osfile(fh) & FOPEN == 0 {
        ptd.errno().set(EBADF);
        ptd.doserrno().set(0);
        return -1;
    }
    let new_fh = _alloc_osfhnd();
    if new_fh == -1 {
        ptd.errno().set(EMFILE);
        ptd.doserrno().set(0);
        return -1;
    }
    let return_value = duplicate_osfhnd(fh, new_fh, ptd);
    // The C's __finally:
    if return_value == -1 {
        (*pioinfo(new_fh)).osfile &= !FOPEN;
    }
    __acrt_lowio_unlock_fh(new_fh);
    return_value
}

unsafe fn dup_internal(fh: c_int, ptd: &mut PtdHost) -> c_int {
    if !ptd_validate_fh(ptd, fh) {
        return -1;
    }
    with_fh_locked(fh, || dup_nolock_internal(fh, ptd))
}

#[no_mangle]
pub unsafe extern "C" fn _dup(fh: c_int) -> c_int {
    let mut ptd = PtdHost::new(null_mut());
    dup_internal(fh, &mut ptd)
}

unsafe fn dup2_nolock_internal(source_fh: c_int, target_fh: c_int, ptd: &mut PtdHost) -> c_int {
    if osfile(source_fh) & FOPEN == 0 {
        ptd.errno().set(EBADF);
        ptd.doserrno().set(0);
        return -1;
    }
    let mut new_osfhandle: *mut c_void = null_mut();
    let result = DuplicateHandle(
        GetCurrentProcess(),
        _get_osfhandle(source_fh) as *mut c_void,
        GetCurrentProcess(),
        &mut new_osfhandle,
        0,
        1,
        DUPLICATE_SAME_ACCESS,
    );
    if result == 0 {
        __acrt_errno_map_os_error_ptd(GetLastError(), ptd);
        return -1;
    }
    // If the target is open, close it once we know the duplication succeeded.
    // Its result is ignored, as in the C.
    if osfile(target_fh) & FOPEN != 0 {
        _close_nolock_internal(target_fh, ptd);
    }
    __acrt_lowio_set_os_handle(target_fh, new_osfhandle as isize);
    copy_mode(target_fh, source_fh);
    0
}

unsafe fn dup2_internal(source_fh: c_int, target_fh: c_int, ptd: &mut PtdHost) -> c_int {
    if !ptd_validate_fh(ptd, source_fh) {
        return -1;
    }
    if ptd_no_console_fh(ptd, target_fh, EBADF) {
        return -1;
    }
    if (target_fh as u32) >= NHANDLE as u32 {
        ptd_invalid_clear_oserr(ptd, EBADF);
        return -1;
    }

    // Make sure there is a record for the target (this sets the global errno,
    // not the host's, on failure -- as in the C).
    if target_fh >= _nhandle && __acrt_lowio_ensure_fh_exists(target_fh) != 0 {
        return -1;
    }

    if source_fh == target_fh {
        return 0;
    }

    // Lock both, lowest first, to avoid deadlock.
    if source_fh < target_fh {
        __acrt_lowio_lock_fh(source_fh);
        __acrt_lowio_lock_fh(target_fh);
    } else if source_fh > target_fh {
        __acrt_lowio_lock_fh(target_fh);
        __acrt_lowio_lock_fh(source_fh);
    }

    let result = dup2_nolock_internal(source_fh, target_fh, ptd);

    __acrt_lowio_unlock_fh(source_fh);
    __acrt_lowio_unlock_fh(target_fh);
    result
}

#[no_mangle]
pub unsafe extern "C" fn _dup2(source_fh: c_int, target_fh: c_int) -> c_int {
    let mut ptd = PtdHost::new(null_mut());
    dup2_internal(source_fh, target_fh, &mut ptd)
}
