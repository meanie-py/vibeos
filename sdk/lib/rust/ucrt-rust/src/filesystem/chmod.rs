//! `_chmod`/`_wchmod`. Replaces `sdk/lib/ucrt/filesystem/{chmod,wchmod}.cpp`
//! in full.
//!
//! ## Details that are silent if wrong
//!
//! * **The only mode bit that does anything is `_S_IWRITE` (0x0080)**: set,
//!   it clears `FILE_ATTRIBUTE_READONLY`; clear, it sets it. Every other bit
//!   passed in `mode` is silently ignored.
//! * **`_wchmod(NULL, _)` is `_VALIDATE_CLEAR_OSSERR_RETURN(path!=NULL, EINVAL,
//!   -1)`**: `_doserrno` cleared, `errno=EINVAL`, handler invoked, but the
//!   RETURN VALUE is a bare `-1`, not `EINVAL` (unlike `_waccess_s`'s own NULL
//!   check, which returns the error code itself).

use core::ffi::c_int;
use core::ptr::null;

use crate::findfile::{file_codepage, mbs_to_wcs, os_errno};
use super::helpers::{
    invalid_parameter_clear_oss, wide_ptr, GetFileAttributesExW, SetFileAttributesW,
    Win32FileAttributeData, EINVAL, FILE_ATTRIBUTE_READONLY, GET_FILE_EX_INFO_STANDARD,
};

/// `_S_IWRITE` (sys/stat.h).
const S_IWRITE: c_int = 0x0080;

/// `_wchmod`.
#[no_mangle]
pub unsafe extern "C" fn _wchmod(path: *const u16, mode: c_int) -> c_int {
    if path.is_null() {
        invalid_parameter_clear_oss(EINVAL);
        return -1;
    }

    let mut attributes: Win32FileAttributeData = core::mem::zeroed();
    if GetFileAttributesExW(path, GET_FILE_EX_INFO_STANDARD, &mut attributes) == 0 {
        os_errno();
        return -1;
    }

    if mode & S_IWRITE != 0 {
        attributes.file_attributes &= !FILE_ATTRIBUTE_READONLY;
    } else {
        attributes.file_attributes |= FILE_ATTRIBUTE_READONLY;
    }

    if SetFileAttributesW(path, attributes.file_attributes) == 0 {
        os_errno();
        return -1;
    }

    0
}

/// `_chmod`: convert through the file-API codepage, call `_wchmod`.
#[no_mangle]
pub unsafe extern "C" fn _chmod(path: *const u8, mode: c_int) -> c_int {
    if path.is_null() {
        return _wchmod(null(), mode);
    }

    match mbs_to_wcs(path, file_codepage()) {
        Ok(wide) => _wchmod(wide_ptr(&wide), mode),
        Err(_) => -1,
    }
}
