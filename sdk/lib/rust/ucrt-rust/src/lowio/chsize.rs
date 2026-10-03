//! `lowio/chsize.cpp`: `_chsize`, `_chsize_s`, `_chsize_nolock` and
//! `_chsize_nolock_internal`.
//!
//! Details that are silent if wrong:
//! * Growing writes zeros through `_write_nolock` in `_INTERNAL_BUFSIZ`
//!   chunks, with the handle forced to `_O_BINARY` for the duration (so a
//!   text-mode file does not get CRs) and the old mode restored afterwards --
//!   but NOT restored if a write fails (the C returns from inside the loop).
//! * A failed zero-write whose `_doserrno` is `ERROR_ACCESS_DENIED` reports
//!   EACCES (not `_write_nolock`'s EBADF); the return is then the host's
//!   errno, or 0 if none was recorded (`value_or(0)`).
//! * The `__crt_seek_guard` destructor seeks back to the ORIGINAL position on
//!   every return path, including the failure ones, after the result is known.
//! * Shrinking seeks to `size` then `SetEndOfFile`; on failure `_doserrno` is
//!   the OS error and errno EACCES.
//! * `_chsize_s` with fh `-2` returns EBADF having set only `_doserrno = 0`
//!   (the `_ERRCODE` form of `_CHECK_FH` sets no errno).
//! * `_chsize(fh, long)` maps any non-zero errno_t to -1.

use super::common::*;
use super::setmode::_setmode_nolock;
use super::write::_write_nolock;
use super::lseek::_lseeki64_nolock;
use super::{_get_osfhandle, in_range, osfile, with_fh_locked, EACCES, ENOMEM, FOPEN};
use crate::ptd_host::PtdHost;
use core::ffi::{c_int, c_long, c_void};
use core::ptr::null_mut;

/// `_INTERNAL_BUFSIZ` (corecrt_internal_lowio.h).
const INTERNAL_BUFSIZ: c_int = 4096;

extern "system" {
    fn SetEndOfFile(handle: *mut c_void) -> c_int;
}

unsafe fn chsize_s_internal(fh: c_int, size: i64, ptd: &mut PtdHost) -> c_int {
    // _UCRT_CHECK_FH_CLEAR_OSSERR_RETURN_ERRCODE: _doserrno only, no errno.
    if fh as isize == super::NO_CONSOLE_FILENO {
        ptd.doserrno().set(0);
        return EBADF;
    }
    if !in_range(fh) {
        ptd_invalid_clear_oserr(ptd, EBADF);
        return EBADF;
    }
    if osfile(fh) & FOPEN == 0 {
        ptd_invalid_clear_oserr(ptd, EBADF);
        return EBADF;
    }
    if size < 0 {
        ptd_invalid_clear_oserr(ptd, EINVAL);
        return EINVAL;
    }
    with_fh_locked(fh, || {
        if osfile(fh) & FOPEN != 0 {
            _chsize_nolock_internal(fh, size, ptd)
        } else {
            ptd.errno().set(EBADF);
            EBADF
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn _chsize_s(fh: c_int, size: i64) -> c_int {
    let mut ptd = PtdHost::new(null_mut());
    chsize_s_internal(fh, size, &mut ptd)
}

/// `__crt_seek_guard`: remembers the position and the end, and seeks back to
/// the position when dropped.
struct SeekGuard {
    place: i64,
    end: i64,
    extend: i64,
    fhh: c_int,
}

impl SeekGuard {
    unsafe fn new(fh: c_int, size: i64) -> Self {
        let place = _lseeki64_nolock(fh, 0, SEEK_CUR_W);
        let end = _lseeki64_nolock(fh, 0, SEEK_END_W);
        SeekGuard { place, end, extend: size.wrapping_sub(end), fhh: fh }
    }
}

impl Drop for SeekGuard {
    fn drop(&mut self) {
        unsafe {
            _lseeki64_nolock(self.fhh, self.place, SEEK_SET_W);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn _chsize_nolock_internal(fh: c_int, size: i64, ptd: *mut PtdHost) -> c_int {
    let ptd = &mut *ptd;
    let mut seek_guard = SeekGuard::new(fh, size);

    if seek_guard.place == -1 || seek_guard.end == -1 {
        return errno_value_or(ptd, EINVAL);
    }

    if seek_guard.extend > 0 {
        // Grow: write zeros.
        let zero_buffer = HeapPtr(calloc(INTERNAL_BUFSIZ as usize, 1));
        if zero_buffer.0.is_null() {
            ptd.errno().set(ENOMEM);
            return ENOMEM;
        }

        let old_mode = _setmode_nolock(fh, CRT_O_BINARY);

        loop {
            let bytes_to_write = if seek_guard.extend >= INTERNAL_BUFSIZ as i64 {
                INTERNAL_BUFSIZ
            } else {
                seek_guard.extend as c_int
            };

            let bytes_written = _write_nolock(fh, zero_buffer.0, bytes_to_write as u32, ptd);
            if bytes_written == -1 {
                // Undocumented: ERROR_ACCESS_DENIED is reported as EACCES here.
                if ptd.doserrno().check(ERROR_ACCESS_DENIED_W as _) {
                    ptd.errno().set(EACCES);
                }
                return errno_value_or(ptd, 0);
            }

            seek_guard.extend -= bytes_written as i64;
            if seek_guard.extend <= 0 {
                break;
            }
        }

        _setmode_nolock(fh, old_mode);
    } else if seek_guard.extend < 0 {
        // Shrink: seek to the new end and truncate.
        let new_end = _lseeki64_nolock(fh, size, SEEK_SET_W);
        if new_end == -1 {
            return errno_value_or(ptd, 0);
        }
        if SetEndOfFile(_get_osfhandle(fh) as *mut c_void) == 0 {
            ptd.doserrno().set(GetLastError() as _);
            ptd.errno().set(EACCES);
            return EACCES;
        }
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn _chsize_nolock(fh: c_int, size: i64) -> c_int {
    let mut ptd = PtdHost::new(null_mut());
    _chsize_nolock_internal(fh, size, &mut ptd)
}

#[no_mangle]
pub unsafe extern "C" fn _chsize(fh: c_int, size: c_long) -> c_int {
    if _chsize_s(fh, size as i64) == 0 {
        0
    } else {
        -1
    }
}
