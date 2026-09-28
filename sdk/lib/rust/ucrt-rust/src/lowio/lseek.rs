//! `lowio/lseek.cpp`: `_lseek`, `_lseeki64`, their `_nolock` forms, and the
//! four PTD-propagating `_internal` forms stdio calls.
//!
//! Details that are silent if wrong:
//! * The 32-bit `_lseek` saves the position first and seeks BACK to it when
//!   the new position does not fit a `long` (EINVAL, position unchanged). The
//!   64-bit form has no save.
//! * The 32-bit result is `new_pos.LowPart`, reinterpreted as `long`; it is
//!   only reached for positions `<= LONG_MAX`.
//! * Only a SUCCESSFUL seek clears `FEOFLAG`.
//! * `SetFilePointerEx` failing (a negative result) is mapped through
//!   `__acrt_errno_map_os_error_ptd` -- `ERROR_NEGATIVE_SEEK` becomes EINVAL.

use super::common::*;
use super::{_get_osfhandle, pioinfo, FEOFLAG, FOPEN};
use crate::ptd_host::PtdHost;
use core::ffi::{c_int, c_long, c_void};
use core::ptr::null_mut;

/// The `long` overload of `common_lseek_do_seek_nolock`.
unsafe fn do_seek_long(os_handle: *mut c_void, offset: c_long, origin: c_int, ptd: &mut PtdHost) -> c_long {
    let mut saved_pos: i64 = 0;
    if SetFilePointerEx(os_handle, 0, &mut saved_pos, SEEK_CUR_W as u32) == 0 {
        __acrt_errno_map_os_error_ptd(GetLastError(), ptd);
        return -1;
    }
    let seek_pos: i64 = offset as i64;
    let mut new_pos: i64 = 0;
    if SetFilePointerEx(os_handle, seek_pos, &mut new_pos, origin as u32) == 0 {
        __acrt_errno_map_os_error_ptd(GetLastError(), ptd);
        return -1;
    }
    // The file pointer must fit in a `long`; if it doesn't, put it back.
    if new_pos > c_long::MAX as i64 {
        SetFilePointerEx(os_handle, saved_pos, null_mut(), SEEK_SET_W as u32);
        ptd.errno().set(EINVAL);
        return -1;
    }
    new_pos as u32 as c_long // `static_cast<long>(new_pos.LowPart)`
}

/// The `__int64` overload.
unsafe fn do_seek_i64(os_handle: *mut c_void, offset: i64, origin: c_int, ptd: &mut PtdHost) -> i64 {
    let mut new_pos: i64 = 0;
    if SetFilePointerEx(os_handle, offset, &mut new_pos, origin as u32) == 0 {
        __acrt_errno_map_os_error_ptd(GetLastError(), ptd);
        return -1;
    }
    new_pos
}

/// `common_lseek_nolock<Integer>`, as a macro over the two overloads.
macro_rules! lseek_nolock_body {
    ($fh:expr, $offset:expr, $origin:expr, $ptd:expr, $seek:ident) => {{
        let os_handle = _get_osfhandle($fh) as *mut c_void;
        if os_handle as isize == -1 {
            $ptd.errno().set(EBADF);
            return -1;
        }
        let new_position = $seek(os_handle, $offset, $origin, $ptd);
        if new_position == -1 {
            return -1;
        }
        (*pioinfo($fh)).osfile &= !FEOFLAG; // Clear the Ctrl-Z flag
        new_position
    }};
}

unsafe fn common_lseek_nolock_long(fh: c_int, offset: c_long, origin: c_int, ptd: &mut PtdHost) -> c_long {
    lseek_nolock_body!(fh, offset, origin, ptd, do_seek_long)
}

unsafe fn common_lseek_nolock_i64(fh: c_int, offset: i64, origin: c_int, ptd: &mut PtdHost) -> i64 {
    lseek_nolock_body!(fh, offset, origin, ptd, do_seek_i64)
}

/// `common_lseek<Integer>`: validate, lock, recheck, seek.
macro_rules! lseek_body {
    ($fh:expr, $offset:expr, $origin:expr, $ptd:expr, $nolock:ident) => {{
        if !ptd_validate_fh($ptd, $fh) {
            return -1;
        }
        super::with_fh_locked($fh, || {
            if (*pioinfo($fh)).osfile & FOPEN == 0 {
                $ptd.errno().set(EBADF);
                $ptd.doserrno().set(0);
                return -1;
            }
            $nolock($fh, $offset, $origin, $ptd)
        })
    }};
}

unsafe fn common_lseek_long(fh: c_int, offset: c_long, origin: c_int, ptd: &mut PtdHost) -> c_long {
    lseek_body!(fh, offset, origin, ptd, common_lseek_nolock_long)
}

unsafe fn common_lseek_i64(fh: c_int, offset: i64, origin: c_int, ptd: &mut PtdHost) -> i64 {
    lseek_body!(fh, offset, origin, ptd, common_lseek_nolock_i64)
}

#[no_mangle]
pub unsafe extern "C" fn _lseek_internal(fh: c_int, offset: c_long, origin: c_int, ptd: *mut PtdHost) -> c_long {
    common_lseek_long(fh, offset, origin, &mut *ptd)
}

#[no_mangle]
pub unsafe extern "C" fn _lseeki64_internal(fh: c_int, offset: i64, origin: c_int, ptd: *mut PtdHost) -> i64 {
    common_lseek_i64(fh, offset, origin, &mut *ptd)
}

#[no_mangle]
pub unsafe extern "C" fn _lseek_nolock_internal(fh: c_int, offset: c_long, origin: c_int, ptd: *mut PtdHost) -> c_long {
    common_lseek_nolock_long(fh, offset, origin, &mut *ptd)
}

#[no_mangle]
pub unsafe extern "C" fn _lseeki64_nolock_internal(fh: c_int, offset: i64, origin: c_int, ptd: *mut PtdHost) -> i64 {
    common_lseek_nolock_i64(fh, offset, origin, &mut *ptd)
}

#[no_mangle]
pub unsafe extern "C" fn _lseek(fh: c_int, offset: c_long, origin: c_int) -> c_long {
    let mut ptd = PtdHost::new(null_mut());
    common_lseek_long(fh, offset, origin, &mut ptd)
}

#[no_mangle]
pub unsafe extern "C" fn _lseeki64(fh: c_int, offset: i64, origin: c_int) -> i64 {
    let mut ptd = PtdHost::new(null_mut());
    common_lseek_i64(fh, offset, origin, &mut ptd)
}

#[no_mangle]
pub unsafe extern "C" fn _lseek_nolock(fh: c_int, offset: c_long, origin: c_int) -> c_long {
    let mut ptd = PtdHost::new(null_mut());
    common_lseek_nolock_long(fh, offset, origin, &mut ptd)
}

#[no_mangle]
pub unsafe extern "C" fn _lseeki64_nolock(fh: c_int, offset: i64, origin: c_int) -> i64 {
    let mut ptd = PtdHost::new(null_mut());
    common_lseek_nolock_i64(fh, offset, origin, &mut ptd)
}
