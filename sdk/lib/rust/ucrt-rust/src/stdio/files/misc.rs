//! `clearerr.cpp`, `feoferr.cpp`, `fileno.cpp`, `setmaxf.cpp`, `ncommode.cpp`,
//! `printf_count_output.cpp`: the small accessors and the two process-wide
//! knobs (`_commode`, the `%n` enable cookie).
//!
//! Details that are silent if wrong:
//! * `clearerr`/`clearerr_s` clear BOTH the stdio flags (`_IOERROR|_IOEOF`)
//!   and the lowio `FEOFLAG` bit through the CHECKED `_osfile_safe` (handles
//!   `fh == -1/-2`, unlike the unchecked `_osfile` this piece's other files
//!   use once a handle is already known valid).
//! * `feof`/`ferror` reject a null stream with `EINVAL`/return 0 -- they do
//!   NOT dereference through `__crt_stdio_stream` first (unlike `_fileno`,
//!   which validates via `Stream::valid()` after construction; both end up
//!   checking the same null-ness, just spelled differently in the C).
//! * `_setmaxstdio` shrinking the table walks it BACKWARDS from the current
//!   max down to the new one and stops (returning failure) at the FIRST
//!   still-in-use slot it finds scanning that direction -- entries below that
//!   point are never inspected or freed even if some of them are also
//!   in-use-free.
//! * `_set_printf_count_output`'s cookie is `__security_cookie | 1`
//!   (never plain 0/1), specifically so a corrupted or zeroed static can't be
//!   mistaken for "enabled" -- both the read and the write always OR in `1`.

use core::ffi::{c_int, c_void};

use crate::stdio::{Stream, IOB_ENTRIES, STDIO_INDEX_LOCK};

const EINVAL: c_int = 22;
/// `FEOFLAG` (corecrt_internal_lowio.h).
const FEOFLAG: u8 = 0x02;
/// `_NHANDLE_` = `IOINFO_ARRAYS(128) * IOINFO_ARRAY_ELTS(64)`.
const NHANDLE_: c_int = 128 * 64;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    fn __acrt_lock(lock: c_int);
    fn __acrt_unlock(lock: c_int);
    fn free(p: *mut c_void);
    fn _recalloc(block: *mut c_void, count: usize, size: usize) -> *mut c_void;
    static mut __security_cookie: usize;
}

unsafe fn invalid_neg1(err: c_int) -> c_int {
    *_errno() = err;
    _invalid_parameter_noinfo();
    -1
}

/// `_VALIDATE_RETURN_ERRCODE(expr, errorcode)` == `_VALIDATE_RETURN(expr,
/// errorcode, errorcode)`: the failure return IS the error code, not -1.
unsafe fn invalid_errcode(err: c_int) -> c_int {
    *_errno() = err;
    _invalid_parameter_noinfo();
    err
}

// ── clearerr.cpp ─────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn clearerr_s(public_stream: *mut c_void) -> c_int {
    let stream = Stream::from_file(public_stream);
    if !stream.valid() {
        return invalid_errcode(EINVAL);
    }

    stream.lock();
    stream.unset_flags(crate::stdio_layout::flags::STREAM_ERROR | crate::stdio_layout::flags::STREAM_EOF);
    super::raw_lowio::osfile_safe_clear(stream.lowio_handle(), FEOFLAG);
    stream.unlock();

    0
}

#[no_mangle]
pub unsafe extern "C" fn clearerr(stream: *mut c_void) {
    clearerr_s(stream);
}

// ── feoferr.cpp ──────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn feof(public_stream: *mut c_void) -> c_int {
    if public_stream.is_null() {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo();
        return 0;
    }
    Stream::from_file(public_stream).eof() as c_int
}

#[no_mangle]
pub unsafe extern "C" fn ferror(public_stream: *mut c_void) -> c_int {
    if public_stream.is_null() {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo();
        return 0;
    }
    Stream::from_file(public_stream).error() as c_int
}

// ── fileno.cpp ───────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _fileno(public_stream: *mut c_void) -> c_int {
    let stream = Stream::from_file(public_stream);
    if !stream.valid() {
        return invalid_neg1(EINVAL);
    }
    stream.lowio_handle()
}

// ── setmaxf.cpp ──────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _setmaxstdio(new_maximum: c_int) -> c_int {
    if !(new_maximum >= IOB_ENTRIES as c_int && new_maximum <= NHANDLE_) {
        return invalid_neg1(EINVAL);
    }

    __acrt_lock(STDIO_INDEX_LOCK);

    let result = (|| -> c_int {
        let nstream = crate::stdio::_nstream;
        if new_maximum == nstream {
            return new_maximum;
        }

        if new_maximum < nstream {
            let mut i = nstream;
            while i != new_maximum {
                i -= 1;
                let entry = *crate::stdio::__piob.add(i as usize);
                if entry.is_null() {
                    continue;
                }
                if Stream(entry).is_in_use() {
                    return -1;
                }
                free(entry.cast());
            }
        }

        let new_piob = _recalloc(
            crate::stdio::__piob.cast(),
            new_maximum as usize,
            core::mem::size_of::<*mut c_void>(),
        );
        if new_piob.is_null() {
            return -1;
        }

        crate::stdio::_nstream = new_maximum;
        crate::stdio::__piob = new_piob.cast();
        new_maximum
    })();

    __acrt_unlock(STDIO_INDEX_LOCK);

    result
}

#[no_mangle]
pub unsafe extern "C" fn _getmaxstdio() -> c_int {
    __acrt_lock(STDIO_INDEX_LOCK);
    let n = crate::stdio::_nstream;
    __acrt_unlock(STDIO_INDEX_LOCK);
    n
}

// ── ncommode.cpp ─────────────────────────────────────────────────────────────

/// Default file commit mode: nocommit.
#[no_mangle]
pub static mut _commode: c_int = 0;

#[no_mangle]
pub unsafe extern "C" fn __p__commode() -> *mut c_int {
    core::ptr::addr_of_mut!(_commode)
}

// ── printf_count_output.cpp ──────────────────────────────────────────────────

static mut ENABLE_PERCENT_N: usize = 0;

#[no_mangle]
pub unsafe extern "C" fn _set_printf_count_output(value: c_int) -> c_int {
    let old = (ENABLE_PERCENT_N == (__security_cookie | 1)) as c_int;
    ENABLE_PERCENT_N = if value != 0 { __security_cookie | 1 } else { 0 };
    old
}

#[no_mangle]
pub unsafe extern "C" fn _get_printf_count_output() -> c_int {
    (ENABLE_PERCENT_N == (__security_cookie | 1)) as c_int
}
