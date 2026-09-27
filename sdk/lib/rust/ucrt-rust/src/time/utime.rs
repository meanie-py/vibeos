//! time/utime.cpp: `_utime32`, `_wutime32`, `_utime64`, `_wutime64`,
//! `_futime32`, `_futime64`.
//!
//! ## Details that are silent if wrong
//! * **`common_utime` opens the file `_O_RDWR | _O_BINARY`, `_SH_DENYNO`,
//!   mode `0`** -- write access is required even though only the mtime/atime
//!   change, and binary mode avoids stripping a trailing Ctrl-Z. The
//!   descriptor is ALWAYS closed before returning, and a failure from
//!   `common_futime` has its `errno` saved/restored around the `_close`
//!   call (so `_close`'s own errno, if it sets one, never leaks out).
//! * **`common_futime` validates `fh` TWICE**: `_CHECK_FH_RETURN` (EBADF,
//!   no handler) then `_VALIDATE_RETURN` (EBADF, WITH handler) for the same
//!   range check, then a THIRD check that the descriptor's `FOPEN` bit is
//!   set (also with the handler) -- ported as three separate checks, not
//!   collapsed into one.
//! * **`times == nullptr` fills BOTH `actime` and `modtime` with the CURRENT
//!   time** (a single `time()` call, reused for both fields, not two
//!   independent calls).
//! * **The write time is computed FIRST, then the access time**, each via
//!   its own `localtime_s` -> `TzSpecificLocalTimeToSystemTime` ->
//!   `SystemTimeToFileTime` chain -- `SetFileTime`'s single call takes both
//!   results together at the very end, so a failure converting the ACCESS
//!   time still means neither time is ever written (the write-time
//!   `FILETIME`, though fully computed, is discarded).
//! * **A `localtime_s` failure here becomes EINVAL**, overwriting whatever
//!   `localtime_s` itself set errno to.

use core::ffi::{c_char, c_int, c_void};

use super::localtime::LocalSrc;
use super::time_get::{_time32, _time64};

const EINVAL: c_int = 22;
const ERRNO_EBADF: c_int = 9;

const O_RDWR: c_int = 0x0002;
const O_BINARY: c_int = 0x8000;
const SH_DENYNO: c_int = 0x40;

const FOPEN: u8 = 0x01;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    fn _sopen_s(pfh: *mut c_int, path: *const c_char, oflag: c_int, shflag: c_int, pmode: c_int) -> c_int;
    fn _wsopen_s(pfh: *mut c_int, path: *const u16, oflag: c_int, shflag: c_int, pmode: c_int) -> c_int;
    /// `corecrt_io.h`: `int _close(int _FileHandle)`.
    fn _close(fh: c_int) -> c_int;
    /// `corecrt_io.h`: `intptr_t _get_osfhandle(int _FileHandle)`.
    fn _get_osfhandle(fh: c_int) -> isize;
    static _nhandle: c_int;
}

extern "system" {
    fn TzSpecificLocalTimeToSystemTime(tz: *const c_void, local: *const SystemTime, utc: *mut SystemTime) -> c_int;
    fn SystemTimeToFileTime(st: *const SystemTime, ft: *mut FileTime64) -> c_int;
    fn SetFileTime(handle: isize, creation: *const FileTime64, access: *const FileTime64, write: *const FileTime64) -> c_int;
}

/// `SYSTEMTIME`, laid out identically to `tzset.rs`'s `SysTime` (kept as a
/// distinct local type since this file has no other reason to depend on
/// `tzset.rs`).
#[repr(C)]
struct SystemTime {
    w_year: u16,
    w_month: u16,
    w_day_of_week: u16,
    w_day: u16,
    w_hour: u16,
    w_minute: u16,
    w_second: u16,
    w_milliseconds: u16,
}

/// `FILETIME`: two `DWORD`s.
#[repr(C)]
#[derive(Default)]
struct FileTime64 {
    low: u32,
    high: u32,
}

/// `__utimbuf32`/`__utimbuf64` (`sys/utime.h`).
#[repr(C)]
pub struct Utimbuf32 {
    pub actime: i32,
    pub modtime: i32,
}
#[repr(C)]
pub struct Utimbuf64 {
    pub actime: i64,
    pub modtime: i64,
}

trait UtimeSrc: LocalSrc {
    type Buf;
    fn times(buf: &Self::Buf) -> (Self, Self);
    fn make(actime: Self, modtime: Self) -> Self::Buf;
    fn now() -> Self;
}
impl UtimeSrc for i32 {
    type Buf = Utimbuf32;
    fn times(buf: &Utimbuf32) -> (i32, i32) {
        (buf.actime, buf.modtime)
    }
    fn make(actime: i32, modtime: i32) -> Utimbuf32 {
        Utimbuf32 { actime, modtime }
    }
    fn now() -> i32 {
        unsafe { _time32(core::ptr::null_mut()) }
    }
}
impl UtimeSrc for i64 {
    type Buf = Utimbuf64;
    fn times(buf: &Utimbuf64) -> (i64, i64) {
        (buf.actime, buf.modtime)
    }
    fn make(actime: i64, modtime: i64) -> Utimbuf64 {
        Utimbuf64 { actime, modtime }
    }
    fn now() -> i64 {
        unsafe { _time64(core::ptr::null_mut()) }
    }
}

unsafe fn common_utime_narrow<T: UtimeSrc>(file_name: *const c_char, times: *mut T::Buf) -> c_int {
    if file_name.is_null() {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo();
        return -1;
    }

    let mut fh: c_int = 0;
    if _sopen_s(&mut fh, file_name, O_RDWR | O_BINARY, SH_DENYNO, 0) != 0 {
        return -1;
    }

    let result = common_futime::<T>(fh, times);

    let stored_errno = if result == -1 { *_errno() } else { 0 };
    _close(fh);
    if result == -1 {
        *_errno() = stored_errno;
    }
    result
}

unsafe fn common_utime_wide<T: UtimeSrc>(file_name: *const u16, times: *mut T::Buf) -> c_int {
    if file_name.is_null() {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo();
        return -1;
    }

    let mut fh: c_int = 0;
    if _wsopen_s(&mut fh, file_name, O_RDWR | O_BINARY, SH_DENYNO, 0) != 0 {
        return -1;
    }

    let result = common_futime::<T>(fh, times);

    let stored_errno = if result == -1 { *_errno() } else { 0 };
    _close(fh);
    if result == -1 {
        *_errno() = stored_errno;
    }
    result
}

#[no_mangle]
pub unsafe extern "C" fn _utime32(file_name: *const c_char, times: *mut Utimbuf32) -> c_int {
    common_utime_narrow::<i32>(file_name, times)
}
#[no_mangle]
pub unsafe extern "C" fn _wutime32(file_name: *const u16, times: *mut Utimbuf32) -> c_int {
    common_utime_wide::<i32>(file_name, times)
}
#[no_mangle]
pub unsafe extern "C" fn _utime64(file_name: *const c_char, times: *mut Utimbuf64) -> c_int {
    common_utime_narrow::<i64>(file_name, times)
}
#[no_mangle]
pub unsafe extern "C" fn _wutime64(file_name: *const u16, times: *mut Utimbuf64) -> c_int {
    common_utime_wide::<i64>(file_name, times)
}

unsafe fn to_systemtime(tm: &super::Tm) -> SystemTime {
    SystemTime {
        w_year: (tm.tm_year + 1900) as u16,
        w_month: (tm.tm_mon + 1) as u16,
        w_day_of_week: 0,
        w_day: tm.tm_mday as u16,
        w_hour: tm.tm_hour as u16,
        w_minute: tm.tm_min as u16,
        w_second: tm.tm_sec as u16,
        w_milliseconds: 0,
    }
}

unsafe fn common_futime<T: UtimeSrc>(fh: c_int, mut times: *mut T::Buf) -> c_int {
    if fh < 0 || (fh as u32) >= (_nhandle as u32) {
        *_errno() = ERRNO_EBADF;
        return -1;
    }
    if crate::stdio_layout::lowio::osfile(fh) & FOPEN == 0 {
        *_errno() = ERRNO_EBADF;
        _invalid_parameter_noinfo();
        return -1;
    }

    let default_times;
    if times.is_null() {
        let now = T::now();
        default_times = T::make(now, now);
        times = &default_times as *const T::Buf as *mut T::Buf;
    }

    let (actime, modtime) = T::times(&*times);

    let mut tm_value = super::Tm::default();
    if T::localtime_s(&mut tm_value, &modtime) != 0 {
        *_errno() = EINVAL;
        return -1;
    }
    let local_write = to_systemtime(&tm_value);
    let mut sys_write: SystemTime = core::mem::zeroed();
    let mut last_write_time = FileTime64::default();
    if TzSpecificLocalTimeToSystemTime(core::ptr::null(), &local_write, &mut sys_write) == 0
        || SystemTimeToFileTime(&sys_write, &mut last_write_time) == 0
    {
        *_errno() = EINVAL;
        return -1;
    }

    if T::localtime_s(&mut tm_value, &actime) != 0 {
        *_errno() = EINVAL;
        return -1;
    }
    let local_access = to_systemtime(&tm_value);
    let mut sys_access: SystemTime = core::mem::zeroed();
    let mut last_access_time = FileTime64::default();
    if TzSpecificLocalTimeToSystemTime(core::ptr::null(), &local_access, &mut sys_access) == 0
        || SystemTimeToFileTime(&sys_access, &mut last_access_time) == 0
    {
        *_errno() = EINVAL;
        return -1;
    }

    if SetFileTime(_get_osfhandle(fh), core::ptr::null(), &last_access_time, &last_write_time) == 0 {
        *_errno() = EINVAL;
        return -1;
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn _futime32(fh: c_int, times: *mut Utimbuf32) -> c_int {
    common_futime::<i32>(fh, times)
}

#[no_mangle]
pub unsafe extern "C" fn _futime64(fh: c_int, times: *mut Utimbuf64) -> c_int {
    common_futime::<i64>(fh, times)
}
