//! `misc/strerror.cpp` (Standard `strerror`/`_wcserror`/`strerror_s`/
//! `_wcserror_s`) and `misc/_strerr.cpp` (legacy `_strerror`/`__wcserror`/
//! `_strerror_s`/`__wcserror_s`, which prefix a caller message). Both share
//! the per-thread buffer in `AcrtPtd::strerror_buffer`/`wcserror_buffer`
//! (`ptd.rs`) and `syserr::get_sys_err_msg`.
//!
//! Kept as two non-generic (narrow/wide) implementations per function rather
//! than the C++'s `template <typename Character>`: the underlying `_s`
//! helpers differ in more than just character width (`strncpy_s` vs.
//! `mbstowcs_s`, `strncat_s`/`strcat_s` vs. `wcsncat_s`/`wcscat_s`), so a
//! shared-trait abstraction here would buy nothing.
//!
//! ## Details that are silent if wrong
//!
//! * The buffer is `strerror_buffer_count` = `94 + 38 + 2` = 134 elements
//!   (`char` or `wchar_t`) -- room for a 94-character caller prefix, `": "`,
//!   and the longest system message (38 chars incl. NUL).
//! * `strerror`/`_wcserror`'s buffer persists across calls on the SAME
//!   thread (lazily allocated once, reused); a failed `calloc` (or no ptd)
//!   returns a fixed literal failure string, not NULL.
//! * `_strerror`/`__wcserror` (the legacy, message-prefixing pair) return
//!   NULL on the same failure, NOT the failure string -- a real difference
//!   from `strerror`/`_wcserror`, transcribed as-is.
//! * `_strerror`'s message is truncated to leave room for `": \n\0"` (4
//!   chars); `strerror_s`/`_wcserror_s` truncate the caller prefix down to
//!   `minimum_message_length` (5) reserved characters and, if it doesn't
//!   fit at all, DROP the prefix entirely rather than truncating it (the
//!   condition is `strlen(message) < buffer_count - 2 - 5`; failing it skips
//!   the whole `if`, prefix and colon both).
//! * `strerror_s`/`_wcserror_s` treat `STRUNCATE` (80) from the internal
//!   `_ERRCHECK_EINVAL_ERANGE` truncating copy as SUCCESS (return 0), any
//!   other nonzero as a real failure code returned to the caller; EINVAL and
//!   ERANGE from that copy are fatal (Watson) before ever reaching that
//!   check, so the only nonzero this can actually return is unreachable in
//!   practice -- transcribed anyway.
//! * `_strerror`/`__wcserror`'s appended `": "` and trailing `"\n"` use
//!   `_ERRCHECK`, i.e. ANY failure from those fixed-size `cat_s` calls is
//!   fatal (Watson), never a quiet no-op.

use core::ffi::{c_char, c_int, c_void};
use core::ptr::null_mut;

use crate::ptd::AcrtPtd;

const EINVAL: c_int = 22;
const ERANGE: c_int = 34;
const STRUNCATE: c_int = 80;

/// `strerror_buffer_count` (`corecrt_internal.h:153`) = 94 + 38 + 2.
const BUFFER_COUNT: usize = 134;
/// `minimum_message_length` (`_strerr.cpp` uses `_strerr`'s own constant of
/// the same name and value independently in `common_strerror_s`).
const MINIMUM_MESSAGE_LENGTH: usize = 5;

extern "C" {
    fn __acrt_getptd_noexit() -> *mut AcrtPtd;
    fn _invalid_parameter_noinfo();
    fn _invoke_watson(e: *const u16, f: *const u16, file: *const u16, line: u32, r: usize) -> !;
    fn calloc(count: usize, size: usize) -> *mut c_void;

    fn strlen(s: *const c_char) -> usize;
    fn wcslen(s: *const u16) -> usize;
    fn strncpy_s(dst: *mut c_char, dst_size: usize, src: *const c_char, count: usize) -> c_int;
    /// Matches `exec.rs`'s existing declaration exactly (`u8`, not `c_char`) --
    /// `#![deny(clashing_extern_declarations)]` requires the identical type.
    fn strcat_s(dst: *mut u8, destsz: usize, src: *const u8) -> c_int;
    fn strncat_s(dst: *mut u8, size: usize, src: *const u8, count: usize) -> c_int;
    fn strcpy_s(dst: *mut u8, size: usize, src: *const u8) -> c_int;
    fn wcscat_s(dst: *mut u16, size: usize, src: *const u16) -> c_int;
    fn wcsncat_s(dst: *mut u16, size: usize, src: *const u16, count: usize) -> c_int;
    fn wcscpy_s(dst: *mut u16, size: usize, src: *const u16) -> c_int;
    fn mbstowcs_s(
        converted: *mut usize,
        dst: *mut u16,
        size: usize,
        src: *const c_char,
        count: usize,
    ) -> c_int;
}

/// `_ERRCHECK(e)`: fatal (never returns) if `e != 0`.
unsafe fn errcheck(e: c_int) {
    if e != 0 {
        _invoke_watson(null_mut(), null_mut(), null_mut(), 0, 0);
    }
}

/// `_ERRCHECK_EINVAL_ERANGE(e)`: fatal if `e` is EINVAL/ERANGE, otherwise
/// returns `e` (including a nonzero like `STRUNCATE`).
unsafe fn errcheck_einval_erange(e: c_int) -> c_int {
    if e == 0 || (e != EINVAL && e != ERANGE) {
        e
    } else {
        _invoke_watson(null_mut(), null_mut(), null_mut(), 0, 0);
    }
}

// ── `strerror`/`_wcserror`/`strerror_s`/`_wcserror_s` (strerror.cpp) ────────

/// Fixed failure string, returned by `strerror`/`_wcserror` (never NULL) when
/// no ptd is available or the lazy `calloc` fails.
const FAILURE_MSG: &[u8] = b"Visual C++ CRT: Not enough memory to complete call to strerror.\0";
const FAILURE_MSG_LEN: usize = FAILURE_MSG.len();
static mut FAILURE_MSG_W: [u16; FAILURE_MSG_LEN] = {
    let mut buf = [0u16; FAILURE_MSG_LEN];
    let mut i = 0;
    while i < FAILURE_MSG.len() {
        buf[i] = FAILURE_MSG[i] as u16;
        i += 1;
    }
    buf
};

/// `common_strerror<char>` -- `strerror`.
#[no_mangle]
pub unsafe extern "C" fn strerror(error_number: c_int) -> *mut c_char {
    let ptd = __acrt_getptd_noexit();
    if ptd.is_null() {
        return FAILURE_MSG.as_ptr() as *mut c_char;
    }
    if (*ptd).strerror_buffer.is_null() {
        (*ptd).strerror_buffer = calloc(BUFFER_COUNT, 1) as *mut u8;
    }
    if (*ptd).strerror_buffer.is_null() {
        return FAILURE_MSG.as_ptr() as *mut c_char;
    }
    let buffer = (*ptd).strerror_buffer;
    let msg = crate::misc::syserr::get_sys_err_msg(error_number as usize);
    errcheck(strncpy_s(buffer as *mut c_char, BUFFER_COUNT, msg, BUFFER_COUNT - 1));
    buffer as *mut c_char
}

/// `common_strerror<wchar_t>` -- `_wcserror`.
#[no_mangle]
pub unsafe extern "C" fn _wcserror(error_number: c_int) -> *mut u16 {
    let ptd = __acrt_getptd_noexit();
    if ptd.is_null() {
        return core::ptr::addr_of_mut!(FAILURE_MSG_W).cast();
    }
    if (*ptd).wcserror_buffer.is_null() {
        (*ptd).wcserror_buffer = calloc(BUFFER_COUNT, 2) as *mut u16;
    }
    if (*ptd).wcserror_buffer.is_null() {
        return core::ptr::addr_of_mut!(FAILURE_MSG_W).cast();
    }
    let buffer = (*ptd).wcserror_buffer;
    let msg = crate::misc::syserr::get_sys_err_msg(error_number as usize);
    // `mbstowcs_s`, matching `copy_string_into_buffer`'s wide overload.
    errcheck(mbstowcs_s(null_mut(), buffer, BUFFER_COUNT, msg, BUFFER_COUNT - 1));
    buffer
}

/// `common_strerror_s<char>` -- `strerror_s`.
#[no_mangle]
pub unsafe extern "C" fn strerror_s(
    buffer: *mut c_char,
    buffer_count: usize,
    error_number: c_int,
) -> c_int {
    if buffer.is_null() || buffer_count == 0 {
        return validate_einval();
    }
    let msg = crate::misc::syserr::get_sys_err_msg(error_number as usize);
    let result = errcheck_einval_erange(strncpy_s(buffer, buffer_count, msg, usize::MAX /* _TRUNCATE */));
    if result == STRUNCATE {
        0
    } else {
        result
    }
}

/// `common_strerror_s<wchar_t>` -- `_wcserror_s`.
#[no_mangle]
pub unsafe extern "C" fn _wcserror_s(buffer: *mut u16, buffer_count: usize, error_number: c_int) -> c_int {
    if buffer.is_null() || buffer_count == 0 {
        return validate_einval();
    }
    let msg = crate::misc::syserr::get_sys_err_msg(error_number as usize);
    // wide `copy_string_into_buffer` is `mbstowcs_s(nullptr, buffer, buffer_count, string, max_count)`
    // with `max_count = _TRUNCATE`; `mbstowcs_s` treats a huge count as "no
    // limit but the buffer size" the same way `_TRUNCATE` does, so `usize::MAX`
    // stands in for the C's `(size_t)-1` sentinel.
    let result = errcheck_einval_erange(mbstowcs_s(null_mut(), buffer, buffer_count, msg, usize::MAX));
    if result == STRUNCATE {
        0
    } else {
        result
    }
}

// ── `_strerror`/`__wcserror`/`_strerror_s`/`__wcserror_s` (_strerr.cpp) ─────

/// `_VALIDATE_RETURN_ERRCODE(cond, EINVAL)`: errno, then the handler.
unsafe fn validate_einval() -> c_int {
    *crate::misc::errno::_errno() = EINVAL;
    _invalid_parameter_noinfo();
    EINVAL
}

/// `common_strerror<char>` (legacy) -- `_strerror`.
#[no_mangle]
pub unsafe extern "C" fn _strerror(message: *const c_char) -> *mut c_char {
    let original_errno = *crate::misc::errno::_errno();
    let ptd = __acrt_getptd_noexit();
    if ptd.is_null() {
        return null_mut();
    }
    if (*ptd).strerror_buffer.is_null() {
        (*ptd).strerror_buffer = calloc(BUFFER_COUNT, 1) as *mut u8;
    }
    if (*ptd).strerror_buffer.is_null() {
        return null_mut();
    }
    let buffer = (*ptd).strerror_buffer;
    *buffer = 0;

    if !message.is_null() && *message != 0 {
        // Leave room for ": \n\0" (4 chars).
        errcheck(strncat_s(buffer, BUFFER_COUNT, message.cast(), BUFFER_COUNT - 4));
        errcheck(strcat_s(buffer, BUFFER_COUNT, b": \0".as_ptr()));
    }

    let system_message = crate::misc::syserr::get_sys_err_msg(original_errno as usize);
    // `append_message` (narrow): `strncat_s(buffer, count, message, count - strlen(buffer) - 2)`.
    let used = strlen(buffer as *const c_char);
    errcheck(strncat_s(buffer, BUFFER_COUNT, system_message.cast(), BUFFER_COUNT - used - 2));
    errcheck(strcat_s(buffer, BUFFER_COUNT, b"\n\0".as_ptr()));

    buffer as *mut c_char
}

/// `common_strerror<wchar_t>` (legacy) -- `__wcserror`.
#[no_mangle]
pub unsafe extern "C" fn __wcserror(message: *const u16) -> *mut u16 {
    let original_errno = *crate::misc::errno::_errno();
    let ptd = __acrt_getptd_noexit();
    if ptd.is_null() {
        return null_mut();
    }
    if (*ptd).wcserror_buffer.is_null() {
        (*ptd).wcserror_buffer = calloc(BUFFER_COUNT, 2) as *mut u16;
    }
    if (*ptd).wcserror_buffer.is_null() {
        return null_mut();
    }
    let buffer = (*ptd).wcserror_buffer;
    *buffer = 0;

    if !message.is_null() && *message != 0 {
        static COLON: [u16; 3] = [b':' as u16, b' ' as u16, 0];
        errcheck(wcsncat_s(buffer, BUFFER_COUNT, message, BUFFER_COUNT - 4));
        errcheck(wcscat_s(buffer, BUFFER_COUNT, COLON.as_ptr()));
    }

    let system_message = crate::misc::syserr::get_sys_err_msg(original_errno as usize);
    // `append_message` (wide): `mbstowcs_s(nullptr, buffer + len, count - len, message, count - len - 2)`.
    let used = wcslen(buffer);
    let mut written = 0usize;
    errcheck(mbstowcs_s(
        &mut written,
        buffer.add(used),
        BUFFER_COUNT - used,
        system_message,
        BUFFER_COUNT - used - 2,
    ));

    static NEWLINE: [u16; 2] = [b'\n' as u16, 0];
    errcheck(wcscat_s(buffer, BUFFER_COUNT, NEWLINE.as_ptr()));

    buffer
}

/// `common_strerror_s<char>` (legacy) -- `_strerror_s`.
#[no_mangle]
pub unsafe extern "C" fn _strerror_s(
    buffer: *mut c_char,
    buffer_count: usize,
    message: *const c_char,
) -> c_int {
    let original_errno = *crate::misc::errno::_errno();
    if buffer.is_null() {
        return validate_einval();
    }
    if buffer_count == 0 {
        return validate_einval();
    }
    *buffer = 0;

    if !message.is_null()
        && *message != 0
        && strlen(message) < buffer_count.wrapping_sub(2 + MINIMUM_MESSAGE_LENGTH)
    {
        errcheck(strcpy_s(buffer.cast(), buffer_count, message.cast()));
        errcheck(strcat_s(buffer.cast(), buffer_count, b": \0".as_ptr()));
    }

    let used = strlen(buffer);
    let system_message = crate::misc::syserr::get_sys_err_msg(original_errno as usize);
    let result = errcheck_einval_erange(strncpy_s(
        buffer.add(used),
        buffer_count - used,
        system_message,
        usize::MAX,
    ));
    if result == STRUNCATE {
        0
    } else {
        result
    }
}

/// `common_strerror_s<wchar_t>` (legacy) -- `__wcserror_s`.
#[no_mangle]
pub unsafe extern "C" fn __wcserror_s(
    buffer: *mut u16,
    buffer_count: usize,
    message: *const u16,
) -> c_int {
    let original_errno = *crate::misc::errno::_errno();
    if buffer.is_null() {
        return validate_einval();
    }
    if buffer_count == 0 {
        return validate_einval();
    }
    *buffer = 0;

    if !message.is_null()
        && *message != 0
        && wcslen(message) < buffer_count.wrapping_sub(2 + MINIMUM_MESSAGE_LENGTH)
    {
        static COLON: [u16; 3] = [b':' as u16, b' ' as u16, 0];
        errcheck(wcscpy_s(buffer, buffer_count, message));
        errcheck(wcscat_s(buffer, buffer_count, COLON.as_ptr()));
    }

    let used = wcslen(buffer);
    let system_message = crate::misc::syserr::get_sys_err_msg(original_errno as usize);
    let result = errcheck_einval_erange(mbstowcs_s(
        null_mut(),
        buffer.add(used),
        buffer_count - used,
        system_message,
        usize::MAX,
    ));
    if result == STRUNCATE {
        0
    } else {
        result
    }
}
