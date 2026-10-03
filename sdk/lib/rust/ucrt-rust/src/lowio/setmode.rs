//! `lowio/setmode.cpp` (`_setmode`, `_setmode_nolock`, `_set_fmode`,
//! `_get_fmode`) and `lowio/txtmode.cpp` (`_fmode`, `__p__fmode`,
//! `__acrt_initialize_fmode`).
//!
//! Details that are silent if wrong:
//! * `_setmode(fh, _O_BINARY)` clears only `FTEXT`: the handle's `textmode`
//!   (ANSI/UTF-8/UTF-16) survives, so `_read`/`_write` on a binary handle that
//!   was UTF-8 still take their UTF-8 buffer paths. Only `_O_TEXT` resets it
//!   to ANSI.
//! * The returned previous mode is `_O_BINARY` when `FTEXT` was clear,
//!   otherwise `_O_TEXT` / `_O_U8TEXT` / `_O_WTEXT` by `textmode` -- a handle
//!   set with `_O_U16TEXT` reports `_O_WTEXT`.
//! * `_setmode` validates the MODE before the handle, and uses the plain
//!   `_VALIDATE_RETURN` (`_doserrno` untouched) and `_CHECK_FH_RETURN` (errno
//!   only). The in-lock recheck sets errno only.
//! * `_set_fmode` accepts only `_O_TEXT`, `_O_BINARY` and `_O_WTEXT` (not
//!   `_O_U8TEXT` / `_O_U16TEXT`).
//! * `_fmode` is a plain `int` here (`state_index_count == 1` in this build's
//!   `corecrt_internal_state_isolation.h`), exported by C name.
//! * `__acrt_initialize_fmode` runs from `.CRT$XIC` via `__acrt_fmode_initializer`
//!   (now `internal/init.rs`), which nothing calls by name: txtmode.cpp's
//!   `_CRT_LINKER_FORCE_INCLUDE` is what pulled it in -- hence the `force_include!`.

use super::common::*;
use super::{invalid, is_open, pioinfo, set_errno, with_fh_locked, FOPEN, FTEXT, TEXT_MODE_ANSI, TEXT_MODE_UTF16LE, TEXT_MODE_UTF8};
use core::ffi::c_int;
use core::ptr::addr_of_mut;
use core::sync::atomic::{AtomicI32, Ordering};

crate::force_include!(__acrt_fmode_initializer);

/// `_fmode`: the default translation mode for newly opened files.
#[no_mangle]
pub static mut _fmode: c_int = 0;

#[no_mangle]
pub unsafe extern "C" fn __p__fmode() -> *mut c_int {
    addr_of_mut!(_fmode)
}

/// A zero `_fmode` becomes `_O_TEXT`, the default.
#[no_mangle]
pub unsafe extern "C" fn __acrt_initialize_fmode() -> c_int {
    let fmode_state = addr_of_mut!(_fmode);
    if *fmode_state == 0 {
        *fmode_state = CRT_O_TEXT;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _setmode(fh: c_int, mode: c_int) -> c_int {
    if !(mode == CRT_O_TEXT
        || mode == CRT_O_BINARY
        || mode == CRT_O_WTEXT
        || mode == CRT_O_U8TEXT
        || mode == CRT_O_U16TEXT)
    {
        invalid(EINVAL);
        return -1;
    }
    // _CHECK_FH_RETURN: errno only.
    if fh as isize == super::NO_CONSOLE_FILENO {
        set_errno(EBADF);
        return -1;
    }
    if !super::in_range(fh) {
        invalid(EBADF);
        return -1;
    }
    if !is_open(fh) {
        invalid(EBADF);
        return -1;
    }
    with_fh_locked(fh, || {
        if (*pioinfo(fh)).osfile & FOPEN == 0 {
            set_errno(EBADF);
            return -1;
        }
        _setmode_nolock(fh, mode)
    })
}

#[no_mangle]
pub unsafe extern "C" fn _setmode_nolock(fh: c_int, mode: c_int) -> c_int {
    let pio = pioinfo(fh);
    let old_mode = (*pio).osfile & FTEXT;
    let old_textmode = (*pio).textmode;

    match mode {
        CRT_O_BINARY => (*pio).osfile &= !FTEXT,
        CRT_O_TEXT => {
            (*pio).osfile |= FTEXT;
            (*pio).textmode = TEXT_MODE_ANSI;
        }
        CRT_O_U8TEXT => {
            (*pio).osfile |= FTEXT;
            (*pio).textmode = TEXT_MODE_UTF8;
        }
        CRT_O_U16TEXT | CRT_O_WTEXT => {
            (*pio).osfile |= FTEXT;
            (*pio).textmode = TEXT_MODE_UTF16LE;
        }
        _ => {}
    }

    if old_mode == 0 {
        return CRT_O_BINARY;
    }
    if old_textmode == TEXT_MODE_ANSI {
        CRT_O_TEXT
    } else if old_textmode == TEXT_MODE_UTF8 {
        CRT_O_U8TEXT
    } else {
        CRT_O_WTEXT
    }
}

#[no_mangle]
pub unsafe extern "C" fn _set_fmode(mode: c_int) -> c_int {
    if !(mode == CRT_O_TEXT || mode == CRT_O_BINARY || mode == CRT_O_WTEXT) {
        invalid(EINVAL);
        return EINVAL;
    }
    // _InterlockedExchange.
    AtomicI32::from_ptr(addr_of_mut!(_fmode)).swap(mode, Ordering::SeqCst);
    0
}

#[no_mangle]
pub unsafe extern "C" fn _get_fmode(p_mode: *mut c_int) -> c_int {
    if p_mode.is_null() {
        invalid(EINVAL);
        return EINVAL;
    }
    // __crt_interlocked_read.
    *p_mode = AtomicI32::from_ptr(addr_of_mut!(_fmode)).load(Ordering::SeqCst);
    0
}
