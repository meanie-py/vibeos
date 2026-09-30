//! `gets.cpp`, `fgets.cpp`, `puts.cpp`, `fputs.cpp`, `putws.cpp`, `fputws.cpp`.

use core::ffi::{c_char, c_int, c_void};
use core::ptr::null_mut;

use super::{
    _fwrite_nolock_internal, stdin_ptr, stdout_ptr, validate_fail, validate_stream_ansi_narrow,
    validate_stream_ansi_narrow_ptd, EOF, WEOF,
};
use crate::ptd_host::PtdHost;
use crate::stdio::{with_stream_locked, Stream};

const EINVAL: c_int = 22;
const ERANGE: c_int = 34;

extern "C" {
    fn strlen(s: *const c_char) -> usize;
}

/// `_fputc_nolock_internal`/`_fputwc_nolock_internal` and the temporary
/// buffering pair are owned by `chario.rs`/`buf.rs` in this same piece; call
/// them directly (no re-declaration -- avoids a self-clash).
use super::chario::{_fputc_nolock_internal, _fputwc_nolock_internal};
use super::buf::{__acrt_stdio_begin_temporary_buffering_nolock, __acrt_stdio_end_temporary_buffering_nolock};

/// `__acrt_stdio_temporary_buffering_guard`: begin on construction, end (flush
/// + teardown) on drop -- the RAII the C's destructor gives for free.
struct TemporaryBufferingGuard {
    stream: *mut c_void,
    ptd: *mut PtdHost,
    flag: bool,
}

impl TemporaryBufferingGuard {
    unsafe fn new(stream: *mut c_void, ptd: *mut PtdHost) -> Self {
        let flag = __acrt_stdio_begin_temporary_buffering_nolock(stream);
        Self { stream, ptd, flag }
    }
}

impl Drop for TemporaryBufferingGuard {
    fn drop(&mut self) {
        unsafe {
            __acrt_stdio_end_temporary_buffering_nolock(self.flag, self.stream, self.ptd);
        }
    }
}

// ── gets.cpp ─────────────────────────────────────────────────────────────────

const CRT_UNBOUNDED_BUFFER_SIZE: usize = usize::MAX;

unsafe fn common_gets_narrow(result: *mut c_char, count: usize, return_early_if_eof_first: bool) -> *mut c_char {
    if result.is_null() {
        validate_fail(EINVAL);
        return null_mut();
    }
    if count == 0 {
        validate_fail(EINVAL);
        return null_mut();
    }

    with_stream_locked(stdin_ptr(), || {
        if !validate_stream_ansi_narrow(Stream::from_file(stdin_ptr())) {
            return null_mut();
        }

        let mut c = super::_getc_nolock(stdin_ptr());
        let mut return_value = result;
        if c == EOF {
            return_value = null_mut();
            if return_early_if_eof_first {
                return return_value;
            }
        }

        if count == CRT_UNBOUNDED_BUFFER_SIZE {
            let mut it = result;
            while c != b'\n' as c_int && c != EOF {
                *it = c as c_char;
                it = it.add(1);
                c = super::_getc_nolock(stdin_ptr());
            }
            *it = 0;
        } else {
            let mut available = count;
            let mut it = result;
            while c != b'\n' as c_int && c != EOF {
                if available > 0 {
                    available -= 1;
                    *it = c as c_char;
                    it = it.add(1);
                }
                c = super::_getc_nolock(stdin_ptr());
            }

            if available == 0 {
                *result = 0; // `_RESET_STRING` (`_FILL_STRING` is a debug-only no-op in release).
                validate_fail(ERANGE); // `_RETURN_BUFFER_TOO_SMALL_ERROR`.
                return null_mut();
            }
            *it = 0;
        }

        return_value
    })
}

unsafe fn common_gets_wide(result: *mut u16, count: usize, return_early_if_eof_first: bool) -> *mut u16 {
    if result.is_null() {
        validate_fail(EINVAL);
        return null_mut();
    }
    if count == 0 {
        validate_fail(EINVAL);
        return null_mut();
    }

    with_stream_locked(stdin_ptr(), || {
        // Wide `validate_stream_is_ansi_if_required` is a no-op (only ANSI
        // functions validate).
        let mut c = super::_getwc_nolock(stdin_ptr()) as c_int;
        let mut return_value = result;
        if c == WEOF as c_int {
            return_value = null_mut();
            if return_early_if_eof_first {
                return return_value;
            }
        }

        if count == CRT_UNBOUNDED_BUFFER_SIZE {
            let mut it = result;
            while c != b'\n' as c_int && c != WEOF as c_int {
                *it = c as u16;
                it = it.add(1);
                c = super::_getwc_nolock(stdin_ptr()) as c_int;
            }
            *it = 0;
        } else {
            let mut available = count;
            let mut it = result;
            while c != b'\n' as c_int && c != WEOF as c_int {
                if available > 0 {
                    available -= 1;
                    *it = c as u16;
                    it = it.add(1);
                }
                c = super::_getwc_nolock(stdin_ptr()) as c_int;
            }

            if available == 0 {
                *result = 0;
                validate_fail(ERANGE);
                return null_mut();
            }
            *it = 0;
        }

        return_value
    })
}

#[no_mangle]
pub unsafe extern "C" fn gets_s(result: *mut c_char, count: usize) -> *mut c_char {
    common_gets_narrow(result, count, false)
}

#[no_mangle]
pub unsafe extern "C" fn _getws_s(result: *mut u16, count: usize) -> *mut u16 {
    common_gets_wide(result, count, false)
}

#[no_mangle]
pub unsafe extern "C" fn gets(result: *mut c_char) -> *mut c_char {
    common_gets_narrow(result, CRT_UNBOUNDED_BUFFER_SIZE, true)
}

#[no_mangle]
pub unsafe extern "C" fn _getws(result: *mut u16) -> *mut u16 {
    common_gets_wide(result, CRT_UNBOUNDED_BUFFER_SIZE, true)
}

// ── fgets.cpp ────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn fgets(string: *mut c_char, count: c_int, public_stream: *mut c_void) -> *mut c_char {
    if !(string != null_mut() || count == 0) {
        validate_fail(EINVAL);
        return null_mut();
    }
    if count < 0 {
        validate_fail(EINVAL);
        return null_mut();
    }
    let stream = Stream::from_file(public_stream);
    if !stream.valid() {
        validate_fail(EINVAL);
        return null_mut();
    }
    if count == 0 {
        return null_mut();
    }

    with_stream_locked(public_stream, || {
        if !validate_stream_ansi_narrow(stream) {
            return null_mut();
        }

        let mut it = string;
        let mut i = 1;
        while i != count {
            let c = super::_getc_nolock(public_stream);
            if c == EOF {
                if it == string {
                    return null_mut();
                }
                break;
            }
            *it = c as c_char;
            it = it.add(1);
            if c == b'\n' as c_int {
                break;
            }
            i += 1;
        }
        *it = 0;
        string
    })
}

#[no_mangle]
pub unsafe extern "C" fn fgetws(string: *mut u16, count: c_int, public_stream: *mut c_void) -> *mut u16 {
    if !(string != null_mut() || count == 0) {
        validate_fail(EINVAL);
        return null_mut();
    }
    if count < 0 {
        validate_fail(EINVAL);
        return null_mut();
    }
    let stream = Stream::from_file(public_stream);
    if !stream.valid() {
        validate_fail(EINVAL);
        return null_mut();
    }
    if count == 0 {
        return null_mut();
    }

    with_stream_locked(public_stream, || {
        let mut it = string;
        let mut i = 1;
        while i != count {
            let c = super::_getwc_nolock(public_stream);
            if c == WEOF {
                if it == string {
                    return null_mut();
                }
                break;
            }
            *it = c;
            it = it.add(1);
            if c == b'\n' as u16 {
                break;
            }
            i += 1;
        }
        *it = 0;
        string
    })
}

// ── puts.cpp / fputs.cpp / putws.cpp / fputws.cpp ────────────────────────────

unsafe fn puts_internal(string: *const c_char, ptd: *mut PtdHost) -> c_int {
    if string.is_null() {
        (*ptd).invalid_parameter(EINVAL);
        return EOF;
    }
    let stream = stdout_ptr();
    if !validate_stream_ansi_narrow_ptd(Stream::from_file(stream), ptd) {
        return EOF;
    }

    let length = strlen(string);
    with_stream_locked(stream, || {
        let _guard = TemporaryBufferingGuard::new(stream, ptd);
        let written = _fwrite_nolock_internal(string.cast(), 1, length, stream, ptd);
        if written != length || _fputc_nolock_internal(b'\n' as c_int, stream, ptd) == EOF {
            return EOF;
        }
        0
    })
}

#[no_mangle]
pub unsafe extern "C" fn puts(string: *const c_char) -> c_int {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    puts_internal(string, &mut ptd)
}

unsafe fn fputs_internal(string: *const c_char, stream: *mut c_void, ptd: *mut PtdHost) -> c_int {
    if string.is_null() {
        (*ptd).invalid_parameter(EINVAL);
        return EOF;
    }
    if stream.is_null() {
        (*ptd).invalid_parameter(EINVAL);
        return EOF;
    }
    if !validate_stream_ansi_narrow_ptd(Stream::from_file(stream), ptd) {
        return EOF;
    }

    let length = strlen(string);
    with_stream_locked(stream, || {
        let _guard = TemporaryBufferingGuard::new(stream, ptd);
        let written = _fwrite_nolock_internal(string.cast(), 1, length, stream, ptd);
        if written == length {
            0
        } else {
            EOF
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn fputs(string: *const c_char, stream: *mut c_void) -> c_int {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    fputs_internal(string, stream, &mut ptd)
}

unsafe fn putws_internal(string: *const u16, ptd: *mut PtdHost) -> c_int {
    if string.is_null() {
        (*ptd).invalid_parameter(EINVAL);
        return WEOF as c_int;
    }
    let stream = stdout_ptr();

    with_stream_locked(stream, || {
        let _guard = TemporaryBufferingGuard::new(stream, ptd);
        let mut it = string;
        while *it != 0 {
            if _fputwc_nolock_internal(*it, stream, ptd) == WEOF {
                return WEOF as c_int;
            }
            it = it.add(1);
        }
        if _fputwc_nolock_internal(b'\n' as u16, stream, ptd) == WEOF {
            return WEOF as c_int;
        }
        0
    })
}

#[no_mangle]
pub unsafe extern "C" fn _putws(string: *const u16) -> c_int {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    putws_internal(string, &mut ptd)
}

unsafe fn fputws_internal(string: *const u16, stream: *mut c_void, ptd: *mut PtdHost) -> c_int {
    if string.is_null() {
        (*ptd).invalid_parameter(EINVAL);
        return EOF;
    }
    if stream.is_null() {
        (*ptd).invalid_parameter(EINVAL);
        return EOF;
    }

    with_stream_locked(stream, || {
        let _guard = TemporaryBufferingGuard::new(stream, ptd);
        let mut it = string;
        while *it != 0 {
            if _fputwc_nolock_internal(*it, stream, ptd) == WEOF {
                return EOF; // note: EOF, not WEOF -- the C's documented divergence.
            }
            it = it.add(1);
        }
        0
    })
}

#[no_mangle]
pub unsafe extern "C" fn fputws(string: *const u16, stream: *mut c_void) -> c_int {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    fputws_internal(string, stream, &mut ptd)
}
