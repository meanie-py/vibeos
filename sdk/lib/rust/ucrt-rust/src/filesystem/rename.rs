//! `rename`/`_wrename`. Replaces
//! `sdk/lib/ucrt/filesystem/{rename,wrename}.cpp` in full.
//!
//! ## Details that are silent if wrong
//!
//! * **Neither name is NULL-checked before conversion.** Unlike every other
//!   narrow wrapper in this directory, `rename` calls `mbs_to_wcs` on
//!   `old_name`/`new_name` directly; a NULL name converts to a NULL wide
//!   pointer (`mbs_to_wcs`'s `Ok(None)` arm, the `set_to_nullptr` case), which
//!   then reaches `_wrename` and fails there via `MoveFileExW`, mapped the
//!   ordinary way.
//! * **A conversion failure re-assigns `errno` from the conversion's own
//!   return value before returning -1** -- redundant with what the conversion
//!   already wrote to `errno` as a side effect, but transcribed as Microsoft
//!   wrote it.
//! * **`_wrename` passes `MOVEFILE_COPY_ALLOWED`**: a cross-volume "rename"
//!   silently becomes copy+delete instead of failing.

use core::ffi::c_int;

use crate::findfile::{file_codepage, mbs_to_wcs, os_errno};
use super::helpers::{wide_ptr, MoveFileExW, _errno};

/// `MOVEFILE_COPY_ALLOWED`.
const MOVEFILE_COPY_ALLOWED: u32 = 0x2;

/// `_wrename`.
#[no_mangle]
pub unsafe extern "C" fn _wrename(old_name: *const u16, new_name: *const u16) -> c_int {
    if MoveFileExW(old_name, new_name, MOVEFILE_COPY_ALLOWED) == 0 {
        os_errno();
        return -1;
    }
    0
}

/// `rename`: convert both names through the file-API codepage, call
/// `_wrename`.
#[no_mangle]
pub unsafe extern "C" fn rename(old_name: *const u8, new_name: *const u8) -> c_int {
    let code_page = file_codepage();

    let wide_old_name = match mbs_to_wcs(old_name, code_page) {
        Ok(w) => w,
        Err(cvt1) => {
            *_errno() = cvt1;
            return -1;
        }
    };

    let wide_new_name = match mbs_to_wcs(new_name, code_page) {
        Ok(w) => w,
        Err(cvt2) => {
            *_errno() = cvt2;
            return -1;
        }
    };

    _wrename(wide_ptr(&wide_old_name), wide_ptr(&wide_new_name))
}
