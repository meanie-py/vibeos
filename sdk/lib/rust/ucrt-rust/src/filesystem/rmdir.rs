//! `_rmdir`/`_wrmdir`. Replaces `sdk/lib/ucrt/filesystem/{rmdir,wrmdir}.cpp`
//! in full.
//!
//! No validation beyond `RemoveDirectoryW` itself: a non-empty directory, the
//! current working directory, or a drive root all fail with whatever Win32
//! maps (non-empty -> `ENOTEMPTY`, per `__acrt_errno_map_os_error`'s table).

use core::ffi::c_int;
use core::ptr::null;

use crate::findfile::{file_codepage, mbs_to_wcs, os_errno};
use super::helpers::{wide_ptr, RemoveDirectoryW};

/// `_wrmdir`.
#[no_mangle]
pub unsafe extern "C" fn _wrmdir(path: *const u16) -> c_int {
    if RemoveDirectoryW(path) == 0 {
        os_errno();
        return -1;
    }
    0
}

/// `_rmdir`: convert through the file-API codepage, call `_wrmdir`.
#[no_mangle]
pub unsafe extern "C" fn _rmdir(path: *const u8) -> c_int {
    if path.is_null() {
        return _wrmdir(null());
    }

    match mbs_to_wcs(path, file_codepage()) {
        Ok(wide) => _wrmdir(wide_ptr(&wide)),
        Err(_) => -1,
    }
}
