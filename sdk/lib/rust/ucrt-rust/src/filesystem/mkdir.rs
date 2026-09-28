//! `_mkdir`/`_wmkdir`. Replaces `sdk/lib/ucrt/filesystem/{mkdir,wmkdir}.cpp`
//! in full.
//!
//! No validation at all beyond what `CreateDirectoryW` itself does: a NULL
//! wide path is passed straight through to Win32 (which rejects it with its
//! own error, mapped the same as any other `CreateDirectoryW` failure -- there
//! is no `_VALIDATE_*` here, unlike `_wchmod`/`_waccess_s`).

use core::ffi::c_int;
use core::ptr::{null, null_mut};

use crate::findfile::{file_codepage, mbs_to_wcs, os_errno};
use super::helpers::{wide_ptr, CreateDirectoryW};

/// `_wmkdir`.
#[no_mangle]
pub unsafe extern "C" fn _wmkdir(path: *const u16) -> c_int {
    if CreateDirectoryW(path, null_mut()) == 0 {
        os_errno();
        return -1;
    }
    0
}

/// `_mkdir`: convert through the file-API codepage, call `_wmkdir`.
#[no_mangle]
pub unsafe extern "C" fn _mkdir(path: *const u8) -> c_int {
    if path.is_null() {
        return _wmkdir(null());
    }

    match mbs_to_wcs(path, file_codepage()) {
        Ok(wide) => _wmkdir(wide_ptr(&wide)),
        Err(_) => -1,
    }
}
