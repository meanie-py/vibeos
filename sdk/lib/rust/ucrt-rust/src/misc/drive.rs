//! `misc/drive.cpp` (`_getdrive`/`_chdrive`), `misc/drivemap.cpp`
//! (`_getdrives`), `misc/drivfree.cpp` (`_getdiskfree`).
//!
//! ## Details that are silent if wrong
//!
//! * `_getdrive`'s fallback path (cwd longer than `MAX_PATH`) sets `errno =
//!   ENOMEM` on ANY failure past that point (both the heap allocation and a
//!   second `GetCurrentDirectoryW` failing) -- never maps `GetLastError`,
//!   never touches `_doserrno`, and returns 0 either way, same as "cannot be
//!   determined."
//! * `get_drive_number_from_path`: a path is only recognised as
//!   drive-rooted if `path[1] == ':'` -- a UNC path (`\\server\share`) or a
//!   relative path yields drive number 0.
//! * `_chdrive`'s validation range is `1..=26` (unlike `_getdcwd`'s `0..=26`
//!   with 0 meaning "default") -- 0 is OUT OF RANGE here and fails CRT_EACCES.
//! * `_getdiskfree`'s Win32 argument order is NOT struct field order:
//!   `GetDiskFreeSpaceW`'s four `LPDWORD`s bind to `sectors_per_cluster`,
//!   `bytes_per_sector`, `avail_clusters`, `total_clusters` IN THAT ORDER --
//!   transcribed exactly (getting this permuted would silently swap which
//!   field holds which count).
//! * `_getdiskfree` on failure sets `errno` via `__acrt_errno_from_os_error`
//!   directly (NOT `__acrt_errno_map_os_error`) -- `_doserrno` is untouched --
//!   and returns the raw OS error code as the function's own return value
//!   (not -1, not a bool).
//! * `_getdiskfree` validates `result != nullptr` and `drive_number <= 26`
//!   with plain `_VALIDATE_RETURN` (EINVAL, invalid-parameter handler), both
//!   returning `ERROR_INVALID_PARAMETER` as the function's return value on
//!   failure (not the errno value).

use core::ffi::{c_int, c_uint, c_ulong, c_void};
use core::ptr::null_mut;

const EINVAL: c_int = 22;
const CRT_EACCES: c_int = 13;
const ENOMEM: c_int = 12;
const ERROR_INVALID_DRIVE: u32 = 15;
const ERROR_INVALID_PARAMETER: u32 = 87;
const MAX_PATH: usize = 260;

extern "system" {
    fn GetCurrentDirectoryW(len: u32, buf: *mut u16) -> u32;
    fn SetCurrentDirectoryW(path: *const u16) -> c_int;
    fn GetLastError() -> u32;
    fn GetLogicalDrives() -> u32;
    fn GetDiskFreeSpaceW(
        root: *const u16,
        sectors_per_cluster: *mut u32,
        bytes_per_sector: *mut u32,
        free_clusters: *mut u32,
        total_clusters: *mut u32,
    ) -> c_int;
}

extern "C" {
    fn _invalid_parameter_noinfo();
    fn __acrt_errno_map_os_error(err: u32);
    fn __acrt_errno_from_os_error(err: c_ulong) -> c_int;
    fn calloc(count: usize, size: usize) -> *mut c_void;
    fn free(p: *mut c_void);
}

unsafe fn validate(ok: bool, err: c_int) {
    if !ok {
        *crate::misc::errno::_errno() = err;
        _invalid_parameter_noinfo();
    }
}

/// `get_drive_number_from_path`.
unsafe fn drive_number_from_path(path: *const u16) -> c_int {
    if *path == 0 || *path.add(1) != b':' as u16 {
        return 0;
    }
    // `__ascii_towupper`: uppercase only the ASCII range, exactly as the
    // C -- a non-ASCII first character (impossible for a real drive letter,
    // but the C doesn't special-case it either) is left as-is by
    // `to_ascii_uppercase`-equivalent arithmetic below.
    let c = *path;
    let upper = if (b'a' as u16..=b'z' as u16).contains(&c) { c - 32 } else { c };
    (upper as c_int) - (b'A' as c_int) + 1
}

/// `_getdrive`.
#[no_mangle]
pub unsafe extern "C" fn _getdrive() -> c_int {
    let mut cwd = [0u16; MAX_PATH + 1];
    let cwd_length = GetCurrentDirectoryW((MAX_PATH + 1) as u32, cwd.as_mut_ptr());
    if cwd_length as usize <= MAX_PATH {
        return drive_number_from_path(cwd.as_ptr());
    }

    let heap_cwd = calloc(cwd_length as usize + 1, 2) as *mut u16;
    if heap_cwd.is_null() {
        *crate::misc::errno::_errno() = ENOMEM;
        return 0;
    }
    let heap_cwd_length = GetCurrentDirectoryW(cwd_length + 1, heap_cwd);
    if heap_cwd_length == 0 {
        *crate::misc::errno::_errno() = ENOMEM;
        free(heap_cwd.cast());
        return 0;
    }
    let result = drive_number_from_path(heap_cwd);
    free(heap_cwd.cast());
    result
}

/// `_chdrive`.
#[no_mangle]
pub unsafe extern "C" fn _chdrive(drive_number: c_int) -> c_int {
    if !(1..=26).contains(&drive_number) {
        *crate::misc::errno::__doserrno() = ERROR_INVALID_DRIVE;
        validate(false, CRT_EACCES);
        return -1;
    }

    let letter = (b'A' as c_int + drive_number - 1) as u16;
    let drive_string: [u16; 3] = [letter, b':' as u16, 0];

    if SetCurrentDirectoryW(drive_string.as_ptr()) == 0 {
        __acrt_errno_map_os_error(GetLastError());
        return -1;
    }
    0
}

/// `_getdrives`.
#[no_mangle]
pub unsafe extern "C" fn _getdrives() -> c_ulong {
    GetLogicalDrives()
}

/// `_diskfree_t` (`ucrt/direct.h`): four plain `unsigned`s, no
/// padding, in this DECLARATION order (which is NOT the order
/// `GetDiskFreeSpaceW`'s arguments bind to them -- see the module doc).
#[repr(C)]
pub struct DiskFreeT {
    pub total_clusters: c_uint,
    pub avail_clusters: c_uint,
    pub sectors_per_cluster: c_uint,
    pub bytes_per_sector: c_uint,
}

/// `_getdiskfree`.
#[no_mangle]
pub unsafe extern "C" fn _getdiskfree(drive_number: c_uint, result: *mut DiskFreeT) -> c_uint {
    if result.is_null() {
        validate(false, EINVAL);
        return ERROR_INVALID_PARAMETER;
    }
    if drive_number > 26 {
        validate(false, EINVAL);
        return ERROR_INVALID_PARAMETER;
    }

    (*result) = DiskFreeT { total_clusters: 0, avail_clusters: 0, sectors_per_cluster: 0, bytes_per_sector: 0 };

    let mut drive_name_buffer: [u16; 4] = [b'_' as u16, b':' as u16, b'\\' as u16, 0];
    let drive_name = if drive_number == 0 {
        null_mut()
    } else {
        drive_name_buffer[0] = drive_number as u16 + (b'A' as u16 - 1);
        drive_name_buffer.as_mut_ptr()
    };

    let ok = GetDiskFreeSpaceW(
        drive_name,
        &mut (*result).sectors_per_cluster,
        &mut (*result).bytes_per_sector,
        &mut (*result).avail_clusters,
        &mut (*result).total_clusters,
    );
    if ok == 0 {
        let os_error = GetLastError();
        *crate::misc::errno::_errno() = __acrt_errno_from_os_error(os_error);
        return os_error;
    }
    0
}
