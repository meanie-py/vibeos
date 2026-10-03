//! `_access`/`_access_s`/`_waccess`/`_waccess_s`. Replaces
//! `sdk/lib/ucrt/filesystem/{access,waccess}.cpp` in full.
//!
//! ## Details that are silent if wrong
//!
//! * **Mode validation**: any bit outside `0b110` (i.e. anything other than
//!   0/2/4/6) is EINVAL, via `_VALIDATE_CLEAR_OSSERR_RETURN_ERRCODE` --
//!   `_doserrno` is cleared to 0 first, then `errno=EINVAL`, then the handler.
//! * **A directory always passes**, for every mode including write.
//! * **A read-only FILE fails only when write access was requested** (mode
//!   bit 2): `_doserrno=ERROR_ACCESS_DENIED`, `errno=EACCES`. A read-only file
//!   with mode 0 or 4 succeeds.
//! * **`_access_s` on a failed narrow->wide conversion returns -1, not the
//!   conversion's own errno value** -- its return type is `errno_t`, but this
//!   one path returns a bare `-1` while still leaving whatever `errno`
//!   `mbs_to_wcs` set (EILSEQ, ENOMEM, or a mapped OS error) in place.
//! * **`_access`/`_waccess`** are pure `== 0 ? 0 : -1` wrappers over the `_s`
//!   forms -- they do not touch errno themselves.

use core::ffi::c_int;
use core::ptr::null;

use crate::findfile::{file_codepage, mbs_to_wcs, os_errno};
use super::helpers::{
    invalid_parameter_clear_oss, wide_ptr, GetFileAttributesExW, Win32FileAttributeData, EACCES,
    EINVAL, ERROR_ACCESS_DENIED, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_READONLY,
    GET_FILE_EX_INFO_STANDARD, __doserrno, _errno,
};

/// `_waccess_s`.
#[no_mangle]
pub unsafe extern "C" fn _waccess_s(path: *const u16, access_mode: c_int) -> c_int {
    if path.is_null() {
        invalid_parameter_clear_oss(EINVAL);
        return EINVAL;
    }
    if access_mode & !6 != 0 {
        invalid_parameter_clear_oss(EINVAL);
        return EINVAL;
    }

    let mut attributes: Win32FileAttributeData = core::mem::zeroed();
    if GetFileAttributesExW(path, GET_FILE_EX_INFO_STANDARD, &mut attributes) == 0 {
        return os_errno();
    }

    // All directories have both read and write access:
    if attributes.file_attributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
        return 0;
    }

    // If we require write access, make sure the read only flag is not set:
    let file_is_read_only = attributes.file_attributes & FILE_ATTRIBUTE_READONLY != 0;
    let mode_requires_write = access_mode & 2 != 0;

    if file_is_read_only && mode_requires_write {
        *__doserrno() = ERROR_ACCESS_DENIED;
        *_errno() = EACCES;
        return EACCES;
    }

    0
}

/// `_waccess`: `_waccess_s`, errors squashed to -1.
#[no_mangle]
pub unsafe extern "C" fn _waccess(path: *const u16, access_mode: c_int) -> c_int {
    if _waccess_s(path, access_mode) == 0 {
        0
    } else {
        -1
    }
}

/// `_access_s`: convert through the file-API codepage, call `_waccess_s`.
#[no_mangle]
pub unsafe extern "C" fn _access_s(path: *const u8, access_mode: c_int) -> c_int {
    if path.is_null() {
        return _waccess_s(null(), access_mode);
    }

    match mbs_to_wcs(path, file_codepage()) {
        Ok(wide) => _waccess_s(wide_ptr(&wide), access_mode),
        // NOTE (quirk, transcribed): a conversion failure returns a bare -1,
        // not the errno_t the conversion produced, even though this function's
        // return type IS errno_t. `errno` itself still reflects the real cause.
        Err(_) => -1,
    }
}

/// `_access`: `_access_s`, errors squashed to -1.
#[no_mangle]
pub unsafe extern "C" fn _access(path: *const u8, access_mode: c_int) -> c_int {
    if _access_s(path, access_mode) == 0 {
        0
    } else {
        -1
    }
}
