//! `_getbuf.cpp`, `_freebuf.cpp`, `_sftbuf.cpp`.

use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};

use super::{calloc, free, malloc, INTERNAL_BUFSIZ};
use crate::stdio::Stream;
use crate::stdio_layout::flags::*;

/// `__acrt_stdio_allocate_buffer_nolock`: gives `public_stream` a buffer if it
/// does not already have one -- a big CRT buffer if `calloc` succeeds,
/// otherwise the stream's own two-byte `_charbuf` (`_IOBUFFER_NONE`).
///
/// ucrtbase is `CRTDLL` (see `stdio/mod.rs`), so the C's `#ifndef CRTDLL
/// ++_cflush` never fires here and is omitted.
#[no_mangle]
pub unsafe extern "C" fn __acrt_stdio_allocate_buffer_nolock(public_stream: *mut c_void) {
    debug_assert!(!public_stream.is_null());
    let stream = Stream::from_file(public_stream);
    let d = stream.data();

    let base = calloc(INTERNAL_BUFSIZ as usize, 1) as *mut i8;
    if !base.is_null() {
        stream.set_flags(STREAM_BUFFER_CRT);
        (*d).base = base;
        (*d).bufsiz = INTERNAL_BUFSIZ;
    } else {
        stream.set_flags(STREAM_BUFFER_NONE);
        (*d).base = addr_of_mut!((*d).charbuf).cast();
        (*d).bufsiz = 2;
    }

    (*d).ptr = (*d).base;
    (*d).cnt = 0;
}

/// `__acrt_stdio_free_buffer_nolock`: releases a CRT-owned buffer. A
/// user-supplied (`setvbuf`) buffer is left alone -- the CRT does not know
/// how to free it.
#[no_mangle]
pub unsafe extern "C" fn __acrt_stdio_free_buffer_nolock(public_stream: *mut c_void) {
    debug_assert!(!public_stream.is_null());
    let stream = Stream::from_file(public_stream);

    if !stream.is_in_use() {
        return;
    }
    if !stream.has_crt_buffer() {
        return;
    }

    let d = stream.data();
    free((*d).base.cast());
    stream.unset_flags(STREAM_BUFFER_CRT | STREAM_BUFFER_SETVBUF);
    (*d).base = null_mut();
    (*d).ptr = null_mut();
    (*d).cnt = 0;
}

// ── _sftbuf.cpp: temporary buffering for stdout/stderr ──────────────────────

/// `__acrt_stdout_buffer` / `__acrt_stderr_buffer`, by their C names:
/// `internal/initialization.cpp` frees them at shutdown. Private statics here
/// built and ran, with the C's copies still linked -- the two were then simply
/// different variables.
#[no_mangle]
pub static mut __acrt_stdout_buffer: *mut c_void = null_mut();
#[no_mangle]
pub static mut __acrt_stderr_buffer: *mut c_void = null_mut();

/// `__acrt_should_use_temporary_buffer`: stderr always; stdout only when it is
/// a console (`_isatty`).
#[no_mangle]
pub unsafe extern "C" fn __acrt_should_use_temporary_buffer(stream: *mut c_void) -> bool {
    extern "C" {
        fn _isatty(fh: core::ffi::c_int) -> core::ffi::c_int;
    }

    if stream == super::stderr_ptr() {
        return true;
    }
    if stream == super::stdout_ptr() && _isatty(super::_fileno(stream)) != 0 {
        return true;
    }
    false
}

/// `__acrt_stdio_begin_temporary_buffering_nolock`: gives stdout/stderr a
/// shared static buffer for the duration of one stdio output call, so an
/// unbuffered console stream does not do a syscall per character. Returns
/// whether buffering was actually set up (the C returns `bool`).
#[no_mangle]
pub unsafe extern "C" fn __acrt_stdio_begin_temporary_buffering_nolock(public_stream: *mut c_void) -> bool {
    debug_assert!(!public_stream.is_null());
    let stream = Stream::from_file(public_stream);

    if !__acrt_should_use_temporary_buffer(stream.public_stream()) {
        return false;
    }

    let buffer: *mut *mut c_void = if stream.public_stream() == super::stdout_ptr() {
        addr_of_mut!(__acrt_stdout_buffer)
    } else if stream.public_stream() == super::stderr_ptr() {
        addr_of_mut!(__acrt_stderr_buffer)
    } else {
        return false;
    };

    // Not `CRTDLL`'s `_cflush` bump (see `__acrt_stdio_allocate_buffer_nolock`).

    if stream.has_any_buffer() {
        return false;
    }

    stream.set_flags(STREAM_WRITE | STREAM_BUFFER_USER | STREAM_BUFFER_STBUF);

    if (*buffer).is_null() {
        *buffer = malloc(INTERNAL_BUFSIZ as usize);
    }

    let d = stream.data();
    if (*buffer).is_null() {
        // Fall back to the stream's own two-byte buffer.
        (*d).base = addr_of_mut!((*d).charbuf).cast();
        (*d).ptr = addr_of_mut!((*d).charbuf).cast();
        (*d).cnt = 2;
        (*d).bufsiz = 2;
        return true;
    }

    (*d).base = (*buffer).cast();
    (*d).ptr = (*buffer).cast();
    (*d).cnt = INTERNAL_BUFSIZ;
    (*d).bufsiz = INTERNAL_BUFSIZ;
    true
}

/// `__acrt_stdio_end_temporary_buffering_nolock`: if `flag` (the value
/// `begin_temporary_buffering_nolock` returned) and the stream still has a
/// temporary buffer, flushes and tears it down.
#[no_mangle]
pub unsafe extern "C" fn __acrt_stdio_end_temporary_buffering_nolock(
    flag: bool,
    public_stream: *mut c_void,
    ptd: *mut crate::ptd_host::PtdHost,
) {
    let stream = Stream::from_file(public_stream);

    if !flag {
        return;
    }

    if stream.has_temporary_buffer() {
        super::__acrt_stdio_flush_nolock(stream.public_stream(), ptd);
        stream.unset_flags(STREAM_BUFFER_USER | STREAM_BUFFER_STBUF);
        let d = stream.data();
        (*d).bufsiz = 0;
        (*d).base = null_mut();
        (*d).ptr = null_mut();
    }
}
