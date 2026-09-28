//! `misc/getcwd.cpp` -- `_getcwd`/`_wgetcwd`/`_getdcwd`/`_wgetdcwd`. The `_dbg`
//! variants (`#ifdef _DEBUG`) are never built in this tree (only
//! `UCRT_MISC_SOURCES`, the release set, is referenced from
//! `sdk/lib/ucrt/CMakeLists.txt` -- `UCRTD_MISC_SOURCES` is dead), so they are
//! not ported.
//!
//! Reimplements the C++ template machinery
//! (`corecrt_internal_win32_buffer.h`'s `__crt_win32_buffer` +
//! `call_win32_function`, `__crt_no_alloc_win32_buffer`,
//! `__crt_public_win32_buffer`) as two small call-twice helpers
//! ([`grow_into`], [`no_resize_into`]) rather than a generic buffer type --
//! the OBSERVABLE contract (what grows, what fails with `ERANGE` vs. maps
//! `GetLastError`, what gets `malloc`'d vs. written in place) is preserved;
//! the C++ ownership machinery (debug-info tags, `detach()`) is not needed
//! since neither policy this file uses is the debug-allocator one.
//!
//! ## Details that are silent if wrong
//!
//! * `is_valid_drive`: drive 0 is always "valid" (the default drive); drive
//!   numbers > 26 fail CRT_EACCES via `_VALIDATE_RETURN` (so via the
//!   invalid-parameter handler, not just a quiet return); any other number is
//!   checked with `GetDriveTypeW("X:\\")`, treating `DRIVE_UNKNOWN` (0) and
//!   `DRIVE_NO_ROOT_DIR` (1) as invalid, anything else (including
//!   `DRIVE_REMOVABLE` with no media inserted) as valid.
//! * A non-zero, invalid `drive_number` fails CRT_EACCES too (not ENOENT), and
//!   ALSO sets `_doserrno = ERROR_INVALID_DRIVE` first -- both call sites
//!   (`is_valid_drive`'s own internal failure and `common_getdcwd`'s check of
//!   its result) set `_doserrno`, once each, before validating.
//! * The "drive string" passed to `GetFullPathName` is `"X:."` (drive +
//!   colon + dot), or plain `"."` for the default drive (0) -- NOT `"X:\\"`;
//!   `GetFullPathNameW` resolves `"X:."` against that drive's own last-set
//!   working directory, which is exactly the point (this is how `_getdcwd`
//!   can report drive Y's cwd while running on drive X).
//! * `max_count` must be `>= 0` (`_VALIDATE_RETURN`, EINVAL) always; when a
//!   caller buffer is given it must ALSO be `> 0` (a second, separate
//!   `_VALIDATE_RETURN`) -- `max_count == 0` with a non-NULL buffer fails
//!   EINVAL even though `>= 0` already passed.
//! * When `user_buffer` is NULL, a NEW buffer is always allocated (even if
//!   `max_count` is 0) sized to fit the full path; the caller owns it and
//!   must `free()` it -- this is the one path that can return a pointer
//!   distinct from `user_buffer`.
//! * When `user_buffer` is given, it is NEVER grown: if the full path
//!   (including NUL) does not fit in `max_count`, the call fails with
//!   `ERANGE` and returns NULL -- `user_buffer` itself is left with only its
//!   first byte zeroed (`user_buffer[0] = '\0'`; matches the C exactly, which
//!   NUL-terminates it up front before attempting the copy).
//! * A `GetFullPathNameW`/`GetCurrentDirectoryW` hard failure (returns 0) maps
//!   `GetLastError()` through `__acrt_errno_map_os_error` (sets BOTH errno and
//!   `_doserrno`); the `ERANGE`-from-no-resize path does NOT touch
//!   `_doserrno` at all -- only `errno`.

use core::ffi::{c_int, c_void};
use core::ptr::null_mut;

const EINVAL: c_int = 22;
const CRT_EACCES: c_int = 13;
const ENOMEM: c_int = 12;
const ERANGE: c_int = 34;
const ERROR_INVALID_DRIVE: u32 = 15;
const DRIVE_UNKNOWN: u32 = 0;
const DRIVE_NO_ROOT_DIR: u32 = 1;
const MAX_PATH: usize = 260;

extern "system" {
    fn GetDriveTypeW(root: *const u16) -> u32;
    fn GetFullPathNameW(file_name: *const u16, buffer_len: u32, buffer: *mut u16, file_part: *mut *mut u16) -> u32;
    fn GetLastError() -> u32;
}

extern "C" {
    fn _invalid_parameter_noinfo();
    fn __acrt_errno_map_os_error(err: u32);
    fn malloc(size: usize) -> *mut c_void;
    fn free(p: *mut c_void);
}

// `_getdrive()`: called here by its real, exported C name (`drive.rs`'s
// `#[no_mangle]` definition), the same way real UCRT code would link it --
// not a crate-internal re-export.
extern "C" {
    fn _getdrive() -> c_int;
}

/// `errno = value; _invalid_parameter_noinfo(); return ret` --
/// `_VALIDATE_RETURN`, without the `_doserrno`-clearing variant.
unsafe fn validate(ok: bool, err: c_int) {
    if !ok {
        *crate::misc::errno::_errno() = err;
        _invalid_parameter_noinfo();
    }
}

/// `is_valid_drive`.
unsafe fn is_valid_drive(drive_number: u32) -> bool {
    if drive_number > 26 {
        *crate::misc::errno::__doserrno() = ERROR_INVALID_DRIVE;
        validate(false, CRT_EACCES);
        return false;
    }
    if drive_number == 0 {
        return true;
    }
    let letter = b'A' + (drive_number as u8 - 1);
    let drive_string: [u16; 4] = [letter as u16, b':' as u16, b'\\' as u16, 0];
    let drive_type = GetDriveTypeW(drive_string.as_ptr());
    !(drive_type == DRIVE_UNKNOWN || drive_type == DRIVE_NO_ROOT_DIR)
}

/// `call_win32_function` over a buffer that may grow via `malloc`, starting
/// from `(initial, initial_cap)` (which may be `(null, 0)`). On success,
/// returns `(ptr, len_without_nul, was_malloced)`; the caller frees `ptr` iff
/// `was_malloced`. Matches `__crt_win32_buffer_internal_dynamic_resizing` /
/// `..._public_dynamic_resizing`, both plain `malloc`/`free` in this tree.
pub(crate) unsafe fn grow_into(
    initial: *mut u16,
    initial_cap: usize,
    call: impl Fn(*mut u16, u32) -> u32,
) -> Result<(*mut u16, usize, bool), c_int> {
    let required = call(initial, initial_cap as u32) as usize;
    if required == 0 {
        __acrt_errno_map_os_error(GetLastError());
        return Err(*crate::misc::errno::_errno());
    }
    if required <= initial_cap {
        return Ok((initial, required, false));
    }
    let cap = required + 1;
    let buf = malloc(cap * 2) as *mut u16;
    if buf.is_null() {
        return Err(ENOMEM);
    }
    let required2 = call(buf, cap as u32) as usize;
    if required2 == 0 {
        __acrt_errno_map_os_error(GetLastError());
        let e = *crate::misc::errno::_errno();
        free(buf.cast());
        return Err(e);
    }
    Ok((buf, required2, true))
}

/// `call_win32_function` over a fixed, caller-owned buffer
/// (`__crt_win32_buffer_no_resizing`): insufficient space sets `errno =
/// ERANGE` (NOT `_doserrno`) and fails, without ever calling `GetLastError`.
unsafe fn no_resize_into(buf: *mut u16, cap: usize, call: impl Fn(*mut u16, u32) -> u32) -> Result<usize, c_int> {
    let required = call(buf, cap as u32) as usize;
    if required == 0 {
        __acrt_errno_map_os_error(GetLastError());
        return Err(*crate::misc::errno::_errno());
    }
    if required <= cap {
        return Ok(required);
    }
    *crate::misc::errno::_errno() = ERANGE;
    Err(ERANGE)
}

/// `common_getdcwd<wchar_t>`. The narrow entry points (below,
/// `common_getdcwd_narrow`) always resolve through THIS wide path plus a
/// codepage conversion, matching
/// `__acrt_get_full_path_name_narrow_acp_or_utf8` -- narrow UCRT paths never
/// touch `GetFullPathNameA`/`GetCurrentDirectoryA` directly.
unsafe fn common_getdcwd_wide(
    mut drive_number: c_int,
    user_buffer: *mut u16,
    max_count: c_int,
) -> *mut u16 {
    if max_count < 0 {
        validate(false, EINVAL);
        return null_mut();
    }

    if drive_number != 0 {
        if !is_valid_drive(drive_number as u32) {
            *crate::misc::errno::__doserrno() = ERROR_INVALID_DRIVE;
            validate(false, CRT_EACCES);
            return null_mut();
        }
    } else {
        drive_number = _getdrive();
    }

    let mut drive_string = [0u16; 4];
    if drive_number != 0 {
        drive_string[0] = (b'A' as c_int - 1 + drive_number) as u16;
        drive_string[1] = b':' as u16;
        drive_string[2] = b'.' as u16;
        drive_string[3] = 0;
    } else {
        drive_string[0] = b'.' as u16;
        drive_string[1] = 0;
    }
    let call = |buf: *mut u16, cap: u32| GetFullPathNameW(drive_string.as_ptr(), cap, buf, null_mut());

    if user_buffer.is_null() {
        // `__crt_public_win32_buffer::allocate(max_count)` up front, then
        // `call_win32_function` on whatever that produced (capacity 0 if the
        // pre-allocation itself failed -- see the module doc).
        let cap0 = max_count as usize;
        let buf0 = if cap0 != 0 { malloc(cap0 * 2) as *mut u16 } else { null_mut() };
        let (cap0, buf0) = if buf0.is_null() { (0, null_mut()) } else { (cap0, buf0) };
        match grow_into(buf0, cap0, call) {
            Ok((ptr, _len, was_malloced)) => {
                if !was_malloced && !buf0.is_null() {
                    // Fit in the pre-allocated buffer: that IS the returned,
                    // caller-owned allocation (matches `detach()` returning
                    // `_string` as-is when `_is_dynamic` was already true).
                    ptr
                } else {
                    if !buf0.is_null() && ptr != buf0 {
                        free(buf0.cast());
                    }
                    ptr
                }
            }
            Err(_) => {
                if !buf0.is_null() {
                    free(buf0.cast());
                }
                null_mut()
            }
        }
    } else {
        validate(max_count > 0, EINVAL);
        if max_count <= 0 {
            return null_mut();
        }
        *user_buffer = 0;
        match no_resize_into(user_buffer, max_count as usize, call) {
            Ok(_) => user_buffer,
            Err(_) => null_mut(),
        }
    }
}

/// `_getcwd`.
#[no_mangle]
pub unsafe extern "C" fn _getcwd(user_buffer: *mut u8, max_length: c_int) -> *mut u8 {
    common_getdcwd_narrow(0, user_buffer, max_length)
}

/// `_wgetcwd`.
#[no_mangle]
pub unsafe extern "C" fn _wgetcwd(user_buffer: *mut u16, max_length: c_int) -> *mut u16 {
    common_getdcwd_wide(0, user_buffer, max_length)
}

/// `_getdcwd`.
#[no_mangle]
pub unsafe extern "C" fn _getdcwd(drive_number: c_int, user_buffer: *mut u8, max_length: c_int) -> *mut u8 {
    common_getdcwd_narrow(drive_number, user_buffer, max_length)
}

/// `_wgetdcwd`.
#[no_mangle]
pub unsafe extern "C" fn _wgetdcwd(drive_number: c_int, user_buffer: *mut u16, max_length: c_int) -> *mut u16 {
    common_getdcwd_wide(drive_number, user_buffer, max_length)
}

/// `__acrt_get_utf8_acp_compatibility_codepage` (`corecrt_internal_win32_buffer.h`,
/// `inline`, not itself exported -- re-derived here the same way
/// `findfile.rs`'s private `file_codepage()` does, independently, since that
/// helper is not `pub`/`pub(crate)` there and this file may only edit its own
/// directory).
pub(crate) unsafe fn file_codepage() -> u32 {
    let lu = crate::locale_update::LocaleUpdate::new(null_mut());
    if (*(*lu.locinfo()).public_ref()).locale_lc_codepage == CP_UTF8 {
        return CP_UTF8;
    }
    if __acrt_AreFileApisANSI() == 0 {
        CP_OEMCP
    } else {
        CP_ACP
    }
}

const CP_ACP: u32 = 0;
const CP_OEMCP: u32 = 1;
const CP_UTF8: u32 = 65001;

extern "system" {
    // Matches `findfile.rs`'s existing declaration exactly (that file also
    // puts this specific symbol under `extern "system"`, even though the
    // header marks it `__cdecl` -- kept consistent to avoid a
    // `clashing_extern_declarations` error against it).
    fn __acrt_AreFileApisANSI() -> c_int;
}

extern "C" {
    // Matches `findfile.rs`'s existing declaration exactly (`extern "C"`
    // there).
    fn __acrt_WideCharToMultiByte(
        cp: u32,
        flags: u32,
        src: *const u16,
        src_len: c_int,
        dst: *mut c_char,
        dst_len: c_int,
        default_char: *const c_char,
        used_default: *mut c_int,
    ) -> c_int;
}

use core::ffi::c_char;

/// `__acrt_wcs_to_mbs_cp`'s "needed size, then convert" shape, over a fresh
/// `malloc`'d buffer sized to fit exactly (no caller buffer involved -- the
/// no-resize/fixed-buffer case is handled separately below, directly against
/// `user_buffer`). Returns the converted, NUL-terminated, `malloc`-owned
/// string and its length INCLUDING the NUL.
pub(crate) unsafe fn wcs_to_mbs_alloc(wide: *const u16, cp: u32) -> Result<(*mut c_char, usize), c_int> {
    let need = __acrt_WideCharToMultiByte(cp, 0, wide, -1, null_mut(), 0, null_mut(), null_mut());
    if need == 0 {
        __acrt_errno_map_os_error(GetLastError());
        return Err(*crate::misc::errno::_errno());
    }
    let buf = malloc(need as usize) as *mut c_char;
    if buf.is_null() {
        return Err(ENOMEM);
    }
    if __acrt_WideCharToMultiByte(cp, 0, wide, -1, buf, need, null_mut(), null_mut()) == 0 {
        __acrt_errno_map_os_error(GetLastError());
        let e = *crate::misc::errno::_errno();
        free(buf.cast());
        return Err(e);
    }
    Ok((buf, need as usize))
}

/// Narrow entry points: get the WIDE result through
/// `common_getdcwd_wide`'s own auto-allocating (NULL-buffer) path -- so this
/// file has exactly one place that knows drive-string construction and
/// validation -- convert it to the target codepage, then place it per
/// `getcwd`'s caller-visible contract (`user_buffer` vs. auto-allocate).
unsafe fn common_getdcwd_narrow(drive_number: c_int, user_buffer: *mut u8, max_count: c_int) -> *mut u8 {
    if max_count < 0 {
        validate(false, EINVAL);
        return null_mut();
    }

    let wide = common_getdcwd_wide(drive_number, null_mut(), MAX_PATH as c_int);
    if wide.is_null() {
        return null_mut();
    }

    let cp = file_codepage();
    let (narrow, narrow_len) = match wcs_to_mbs_alloc(wide, cp) {
        Ok(v) => v,
        Err(e) => {
            free(wide.cast());
            *crate::misc::errno::_errno() = e;
            return null_mut();
        }
    };
    free(wide.cast());

    if user_buffer.is_null() {
        // Already exactly the fit-size `malloc`'d allocation -- hand it to
        // the caller as-is, matching `_getcwd`'s "malloc is used" contract.
        narrow.cast()
    } else {
        validate(max_count > 0, EINVAL);
        if max_count <= 0 {
            free(narrow.cast());
            return null_mut();
        }
        if narrow_len > max_count as usize {
            *crate::misc::errno::_errno() = ERANGE;
            free(narrow.cast());
            return null_mut();
        }
        core::ptr::copy_nonoverlapping(narrow.cast::<u8>(), user_buffer, narrow_len);
        free(narrow.cast());
        user_buffer
    }
}
