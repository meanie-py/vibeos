//! The small lowio files, one section each: `commit.cpp` (`_commit`),
//! `creat.cpp` (`_creat`, `_wcreat`), `eof.cpp` (`_eof`), `filelength.cpp`
//! (`_filelength`, `_filelengthi64`), `isatty.cpp` (`_isatty`), `tell.cpp`
//! (`_tell`), `telli64.cpp` (`_telli64`), `umask.cpp` (`_umaskval`, `_umask`,
//! `_umask_s`) and `locking.cpp` (`_locking`).
//!
//! Details that are silent if wrong:
//! * `_isatty` and `_commit` use the plain `_CHECK_FH_RETURN` /
//!   `_VALIDATE_RETURN` (errno only; `_doserrno` untouched), unlike the rest,
//!   which clear `_doserrno` first. `_isatty` returns `FDEV` itself (0x40),
//!   not 1, and does not check `FOPEN`.
//! * `_commit` on a `FlushFileBuffers` failure sets `_doserrno` to the OS
//!   error but errno to EBADF (not the mapped error).
//! * `_eof` and `_filelength*` restore the position afterwards; `_filelength`
//!   seeks back only if it moved, `_eof` only if it is not at the end.
//! * `_filelength` goes through the `long` `_lseek_nolock`, so a file longer
//!   than `LONG_MAX` is -1/EINVAL, not truncated.
//! * `_umask` silently drops bits other than `_S_IREAD | _S_IWRITE`;
//!   `_umask_s` stores the OLD mask into `*old_mode` BEFORE validating the new
//!   one, so a rejected mode still reports the old mask.
//! * `_locking` locks from the CURRENT position. `_LK_LOCK` / `_LK_RLCK` retry
//!   ten times a second apart and then report EDEADLOCK (overwriting the
//!   mapped OS error); every other mode, including unknown values, is one try
//!   of an exclusive fail-immediately `LockFileEx`, except `_LK_UNLCK`.
//! * `_creat` returns -1 (not the errno_t) on failure; errno comes from
//!   `_sopen_s`.

use super::common::*;
use super::lseek::{_lseek, _lseek_nolock, _lseeki64, _lseeki64_nolock};
use super::{
    __acrt_errno_map_os_error, _get_osfhandle, in_range, invalid, invalid_clear_oserr, osfile, set_doserrno, set_errno,
    with_fh_locked,
    FDEV, FOPEN,
};
use core::ffi::{c_char, c_int, c_long, c_void};

// ── commit.cpp ──────────────────────────────────────────────────────────────

extern "system" {
    fn FlushFileBuffers(handle: *mut c_void) -> c_int;
}

#[no_mangle]
pub unsafe extern "C" fn _commit(fh: c_int) -> c_int {
    // _CHECK_FH_RETURN: errno only.
    if fh as isize == super::NO_CONSOLE_FILENO {
        set_errno(EBADF);
        return -1;
    }
    if !in_range(fh) {
        invalid(EBADF);
        return -1;
    }
    if osfile(fh) & FOPEN == 0 {
        invalid(EBADF);
        return -1;
    }
    with_fh_locked(fh, || {
        if osfile(fh) & FOPEN != 0 {
            if FlushFileBuffers(_get_osfhandle(fh) as *mut c_void) != 0 {
                return 0; // Success
            }
            set_doserrno(GetLastError() as _);
        }
        set_errno(EBADF);
        -1
    })
}

// ── creat.cpp ───────────────────────────────────────────────────────────────

extern "C" {
    fn _sopen_s(pfh: *mut c_int, path: *const c_char, oflag: c_int, shflag: c_int, pmode: c_int) -> c_int;
    fn _wsopen_s(pfh: *mut c_int, path: *const u16, oflag: c_int, shflag: c_int, pmode: c_int) -> c_int;
}

#[no_mangle]
pub unsafe extern "C" fn _creat(path: *const c_char, pmode: c_int) -> c_int {
    // creat is just the same as open...
    let mut fh: c_int = -1;
    let e = _sopen_s(&mut fh, path, CRT_O_CREAT + CRT_O_TRUNC + CRT_O_RDWR, CRT_SH_DENYNO, pmode);
    if e == 0 {
        fh
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn _wcreat(path: *const u16, pmode: c_int) -> c_int {
    let mut fh: c_int = -1;
    let e = _wsopen_s(&mut fh, path, CRT_O_CREAT + CRT_O_TRUNC + CRT_O_RDWR, CRT_SH_DENYNO, pmode);
    if e == 0 {
        fh
    } else {
        -1
    }
}

// ── eof.cpp ─────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _eof(fh: c_int) -> c_int {
    if !validate_fh_clear_oserr(fh) {
        return -1;
    }
    with_fh_locked(fh, || {
        if osfile(fh) & FOPEN == 0 {
            set_errno(EBADF);
            set_doserrno(0);
            return -1;
        }
        let here = _lseeki64_nolock(fh, 0, SEEK_CUR_W);
        if here == -1 {
            return -1;
        }
        let end = _lseeki64_nolock(fh, 0, SEEK_END_W);
        if end == -1 {
            return -1;
        }
        if here == end {
            return 1;
        }
        // Not at the end: put the position back.
        _lseeki64_nolock(fh, here, SEEK_SET_W);
        0
    })
}

// ── filelength.cpp ──────────────────────────────────────────────────────────

/// `common_filelength<Integer>` over the matching `lseek_nolock`.
macro_rules! filelength_body {
    ($fh:expr, $ty:ty, $seek:ident) => {{
        if !validate_fh_clear_oserr($fh) {
            return -1;
        }
        with_fh_locked($fh, || {
            let mut end: $ty = -1;
            if osfile($fh) & FOPEN == 0 {
                set_errno(EBADF);
                set_doserrno(0);
                return end;
            }
            // Seek to the end to get the length.
            let here: $ty = $seek($fh, 0, SEEK_CUR_W);
            if here == -1 {
                return end;
            }
            end = $seek($fh, 0, SEEK_END_W);
            if here != end {
                $seek($fh, here, SEEK_SET_W);
            }
            end
        })
    }};
}

#[no_mangle]
pub unsafe extern "C" fn _filelength(fh: c_int) -> c_long {
    filelength_body!(fh, c_long, _lseek_nolock)
}

#[no_mangle]
pub unsafe extern "C" fn _filelengthi64(fh: c_int) -> i64 {
    filelength_body!(fh, i64, _lseeki64_nolock)
}

// ── isatty.cpp ──────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _isatty(fh: c_int) -> c_int {
    // _CHECK_FH_RETURN: errno only.
    if fh as isize == super::NO_CONSOLE_FILENO {
        set_errno(EBADF);
        return 0;
    }
    if !in_range(fh) {
        invalid(EBADF);
        return 0;
    }
    (osfile(fh) & FDEV) as c_int
}

// ── tell.cpp / telli64.cpp ──────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _tell(fh: c_int) -> c_long {
    _lseek(fh, 0, SEEK_CUR_W)
}

#[no_mangle]
pub unsafe extern "C" fn _telli64(fh: c_int) -> i64 {
    _lseeki64(fh, 0, SEEK_CUR_W)
}

// ── umask.cpp ───────────────────────────────────────────────────────────────

/// `sys/stat.h`.
const CRT_S_IREAD: c_int = 0x0100;
const CRT_S_IWRITE: c_int = 0x0080;

/// The mask `_open` applies to its permission argument (`lowio_open.rs` reads it by name).
#[no_mangle]
pub static mut _umaskval: c_int = 0;

#[no_mangle]
pub unsafe extern "C" fn _umask_s(mode: c_int, old_mode: *mut c_int) -> c_int {
    if old_mode.is_null() {
        invalid(EINVAL);
        return EINVAL;
    }
    *old_mode = _umaskval;
    if mode & !(CRT_S_IREAD | CRT_S_IWRITE) != 0 {
        invalid(EINVAL);
        return EINVAL;
    }
    _umaskval = mode;
    0
}

#[no_mangle]
pub unsafe extern "C" fn _umask(mode: c_int) -> c_int {
    // Silently ignore non-Windows modes.
    let masked_mode = mode & (CRT_S_IREAD | CRT_S_IWRITE);
    let mut old_mode: c_int = 0;
    _umask_s(masked_mode, &mut old_mode);
    old_mode
}

// ── locking.cpp ─────────────────────────────────────────────────────────────

/// `sys/locking.h`.
const LK_UNLCK: c_int = 0;
const LK_LOCK: c_int = 1;
const LK_RLCK: c_int = 3;
/// winbase.h.
const LOCKFILE_FAIL_IMMEDIATELY: u32 = 0x1;
const LOCKFILE_EXCLUSIVE_LOCK: u32 = 0x2;

/// `OVERLAPPED`: `Internal`, `InternalHigh` (ULONG_PTR), then the offset
/// union, then `hEvent`.
#[repr(C)]
struct Overlapped {
    internal: usize,
    internal_high: usize,
    offset: u32,
    offset_high: u32,
    h_event: *mut c_void,
}
const _: () = assert!(core::mem::size_of::<Overlapped>() == if cfg!(target_pointer_width = "64") { 32 } else { 20 });

extern "system" {
    fn LockFileEx(
        handle: *mut c_void,
        flags: u32,
        reserved: u32,
        bytes_low: u32,
        bytes_high: u32,
        overlapped: *mut Overlapped,
    ) -> c_int;
    fn UnlockFileEx(
        handle: *mut c_void,
        reserved: u32,
        bytes_low: u32,
        bytes_high: u32,
        overlapped: *mut Overlapped,
    ) -> c_int;
    fn Sleep(ms: u32);
}

unsafe fn locking_nolock(fh: c_int, locking_mode: c_int, number_of_bytes: c_long) -> c_int {
    let lock_offset = _lseeki64_nolock(fh, 0, SEEK_CUR_W);
    if lock_offset == -1 {
        return -1;
    }

    let mut overlapped = Overlapped {
        internal: 0,
        internal_high: 0,
        offset: lock_offset as u32,
        offset_high: ((lock_offset >> 32) & 0xffff_ffff) as u32,
        h_event: core::ptr::null_mut(),
    };

    // Set the retry count, based on the mode.
    let allow_retry = locking_mode == LK_LOCK || locking_mode == LK_RLCK;
    let retry_count = if allow_retry { 10 } else { 1 };

    // Ask the OS to lock the file either until the request succeeds or the
    // retry count is reached, whichever comes first.
    let mut succeeded = false;
    let mut i = 0;
    while i != retry_count {
        if locking_mode == LK_UNLCK {
            succeeded = UnlockFileEx(_get_osfhandle(fh) as *mut c_void, 0, number_of_bytes as u32, 0, &mut overlapped) == 1;
        } else {
            succeeded = LockFileEx(
                _get_osfhandle(fh) as *mut c_void,
                LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
                0,
                number_of_bytes as u32,
                0,
                &mut overlapped,
            ) == 1;
        }
        if succeeded {
            break;
        }
        // Sleep between retries, but not after the last.
        if i != retry_count - 1 {
            Sleep(1000);
        }
        i += 1;
    }

    // If an OS error occurred (the file is already locked), return failure;
    // for _LK_LOCK and _LK_RLCK the error is EDEADLOCK.
    if !succeeded {
        __acrt_errno_map_os_error(GetLastError());
        if locking_mode == LK_LOCK || locking_mode == LK_RLCK {
            set_errno(CRT_EDEADLOCK);
        }
        return -1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _locking(fh: c_int, locking_mode: c_int, number_of_bytes: c_long) -> c_int {
    if !validate_fh_clear_oserr(fh) {
        return -1;
    }
    if number_of_bytes < 0 {
        invalid_clear_oserr(EINVAL);
        return -1;
    }
    with_fh_locked(fh, || {
        if osfile(fh) & FOPEN == 0 {
            set_errno(EBADF);
            set_doserrno(0);
            return -1;
        }
        locking_nolock(fh, locking_mode, number_of_bytes)
    })
}
