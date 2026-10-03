//! `remove`/`_unlink`/`_wremove`/`_wunlink`. Replaces
//! `sdk/lib/ucrt/filesystem/{unlink,wunlink}.cpp` in full.
//!
//! `_unlink` is a plain alias for `remove`; `_wunlink` is a plain alias for
//! `_wremove`. No validation beyond `DeleteFileW` itself: a directory or a
//! read-only file both fail with whatever Win32 maps (a directory ->
//! `EACCES`, per `__acrt_errno_map_os_error`'s table).

use core::ffi::c_int;
use core::ptr::null;

use crate::findfile::{file_codepage, mbs_to_wcs, os_errno};
use super::helpers::{wide_ptr, DeleteFileW};

/// `_wremove`.
#[no_mangle]
pub unsafe extern "C" fn _wremove(path: *const u16) -> c_int {
    if DeleteFileW(path) == 0 {
        os_errno();
        return -1;
    }
    0
}

/// `_wunlink`: a plain alias for `_wremove`.
#[no_mangle]
pub unsafe extern "C" fn _wunlink(path: *const u16) -> c_int {
    _wremove(path)
}

/// `remove`: convert through the file-API codepage, call `_wremove`.
#[no_mangle]
pub unsafe extern "C" fn remove(path: *const u8) -> c_int {
    if path.is_null() {
        return _wremove(null());
    }

    match mbs_to_wcs(path, file_codepage()) {
        Ok(wide) => _wremove(wide_ptr(&wide)),
        Err(_) => -1,
    }
}

/// `_unlink`: a plain alias for `remove`.
#[no_mangle]
pub unsafe extern "C" fn _unlink(path: *const u8) -> c_int {
    remove(path)
}
