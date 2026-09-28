//! `openfile.cpp`, `fopen.cpp`, `freopen.cpp`, `fdopen.cpp`: everything that
//! opens a stdio stream over a name, an already-`fopen`-parsed mode string,
//! and (for `_fdopen`) an existing lowio handle.
//!
//! `common_openfile`/`_openfile`/`_wopenfile` are `stdio_traits::open_file`'s
//! two instantiations, called directly here (not through an extern
//! declaration) since both the caller (`fopen`/`freopen`) and the callee live
//! in this same source file/piece; only calls that cross to bufio/input/lowio
//! go through a transcribed `extern "C"` declaration.
//!
//! Details that are silent if wrong:
//! * ucrtbase is `CRTDLL` (see `stdio/mod.rs`), so the `#ifndef CRTDLL
//!   ++_cflush` pre-termination hint in `openfile.cpp`/`fdopen.cpp` is dead
//!   code here and is not ported.
//! * `fclose`'s internals may run inside `freopen` (to close the old file)
//!   while this stream is STILL LOCKED by `freopen` itself -- `_fclose_nolock`
//!   (not `fclose`) is what's called, exactly as the C does, to avoid
//!   recursive-lock deadlock.
//! * After that `_fclose_nolock`, `freopen` immediately re-sets
//!   `_IOALLOCATED` on the very stream `fclose` may just have deallocated:
//!   the lock is still held throughout, so no other thread can have reused
//!   the slot in between.
//! * `fopen`/`_wfopen` use `_SH_DENYNO`; `fopen_s`/`_wfopen_s` use
//!   `_SH_SECURE`; `freopen` uses `_SH_DENYNO`; `freopen_s` uses `_SH_SECURE`.
//! * `common_fsopen`'s `*file_name != 0` check is `_VALIDATE_RETURN_NOEXC`
//!   (errno only); everything else in these four files is the handler-calling
//!   `_VALIDATE_RETURN`/`_VALIDATE_RETURN_ERRCODE`.

use core::ffi::{c_char, c_int, c_long, c_void};
use core::ptr::null_mut;

use super::parse_mode::{parse_mode, ModeCursor};
use crate::stdio::{allocate_stream, free_stream, Stream};
use crate::stdio_layout::flags::STREAM_ALLOCATED;

const EINVAL: c_int = 22;
const EBADF: c_int = 9;
const EMFILE: c_int = 24;

const SH_DENYNO: c_int = 0x40;
const SH_SECURE: c_int = 0x80;

/// `FOPEN` (corecrt_internal_lowio.h): file handle open.
const FOPEN: u8 = 0x01;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    static _nhandle: c_int;
}

unsafe fn invalid_ret(err: c_int) -> *mut c_void {
    *_errno() = err;
    _invalid_parameter_noinfo();
    null_mut()
}
unsafe fn invalid_ret_noexc(err: c_int) -> *mut c_void {
    *_errno() = err;
    null_mut()
}
unsafe fn invalid_errcode(err: c_int) -> c_int {
    *_errno() = err;
    _invalid_parameter_noinfo();
    err
}
unsafe fn invalid_errcode_noexc(err: c_int) -> c_int {
    *_errno() = err;
    err
}

/// `_sopen_s`/`_wsopen_s` (lowio_open.rs), dispatched by character width --
/// `__acrt_stdio_char_traits<Character>::tsopen_s`.
trait OpenChar: ModeCursor {
    unsafe fn tsopen_s(pfh: *mut c_int, path: Self, oflag: c_int, shflag: c_int, pmode: c_int) -> c_int;
}

extern "C" {
    fn _sopen_s(pfh: *mut c_int, path: *const c_char, oflag: c_int, shflag: c_int, pmode: c_int) -> c_int;
    fn _wsopen_s(pfh: *mut c_int, path: *const u16, oflag: c_int, shflag: c_int, pmode: c_int) -> c_int;
}

impl OpenChar for *const c_char {
    unsafe fn tsopen_s(pfh: *mut c_int, path: Self, oflag: c_int, shflag: c_int, pmode: c_int) -> c_int {
        _sopen_s(pfh, path, oflag, shflag, pmode)
    }
}
impl OpenChar for *const u16 {
    unsafe fn tsopen_s(pfh: *mut c_int, path: Self, oflag: c_int, shflag: c_int, pmode: c_int) -> c_int {
        _wsopen_s(pfh, path, oflag, shflag, pmode)
    }
}

/// `_S_IREAD | _S_IWRITE`.
const S_IREAD_IWRITE: c_int = 0x0100 | 0x0080;

/// `common_openfile<Character>`: `_openfile`/`_wopenfile`'s shared body.
unsafe fn common_openfile<P: OpenChar>(file_name: P, mode: P, share_flag: c_int, stream: Stream) -> *mut c_void {
    let parsed = parse_mode(mode);
    if !parsed.success {
        return null_mut();
    }

    let mut fh: c_int = 0;
    if P::tsopen_s(&mut fh, file_name, parsed.lowio_mode, share_flag, S_IREAD_IWRITE) != 0 {
        return null_mut();
    }

    stream.set_flags(parsed.stdio_mode);
    let d = stream.data();
    (*d).cnt = 0;
    (*d).tmpfname = null_mut();
    (*d).base = null_mut();
    (*d).ptr = null_mut();
    (*d).file = fh as c_long;

    stream.public_stream()
}

#[no_mangle]
pub unsafe extern "C" fn _openfile(file_name: *const c_char, mode: *const c_char, share_flag: c_int, public_stream: *mut c_void) -> *mut c_void {
    common_openfile(file_name, mode, share_flag, Stream::from_file(public_stream))
}

#[no_mangle]
pub unsafe extern "C" fn _wopenfile(file_name: *const u16, mode: *const u16, share_flag: c_int, public_stream: *mut c_void) -> *mut c_void {
    common_openfile(file_name, mode, share_flag, Stream::from_file(public_stream))
}

// ── fopen.cpp ────────────────────────────────────────────────────────────────

unsafe fn common_fsopen<P: OpenChar>(file_name: P, mode: P, share_flag: c_int) -> *mut c_void {
    if file_name.is_null() {
        return invalid_ret(EINVAL);
    }
    if mode.is_null() {
        return invalid_ret(EINVAL);
    }
    if mode.ascii() == Some(0) {
        return invalid_ret(EINVAL);
    }
    if file_name.ascii() == Some(0) {
        return invalid_ret_noexc(EINVAL);
    }

    // Obtain a free stream. Note that the stream is returned locked.
    let stream = allocate_stream();
    if !stream.valid() {
        *_errno() = EMFILE;
        return null_mut();
    }

    let return_value = common_openfile(file_name, mode, share_flag, stream);
    if return_value.is_null() {
        free_stream(stream);
    }
    stream.unlock();

    return_value
}

unsafe fn common_fopen_s<P: OpenChar>(result: *mut *mut c_void, file_name: P, mode: P) -> c_int {
    if result.is_null() {
        return invalid_errcode(EINVAL);
    }

    *result = common_fsopen(file_name, mode, SH_SECURE);
    if (*result).is_null() {
        return *_errno();
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _fsopen(file: *const c_char, mode: *const c_char, share_flag: c_int) -> *mut c_void {
    common_fsopen(file, mode, share_flag)
}

#[no_mangle]
pub unsafe extern "C" fn fopen(file: *const c_char, mode: *const c_char) -> *mut c_void {
    common_fsopen(file, mode, SH_DENYNO)
}

#[no_mangle]
pub unsafe extern "C" fn fopen_s(result: *mut *mut c_void, file: *const c_char, mode: *const c_char) -> c_int {
    common_fopen_s(result, file, mode)
}

#[no_mangle]
pub unsafe extern "C" fn _wfsopen(file: *const u16, mode: *const u16, share_flag: c_int) -> *mut c_void {
    common_fsopen(file, mode, share_flag)
}

#[no_mangle]
pub unsafe extern "C" fn _wfopen(file: *const u16, mode: *const u16) -> *mut c_void {
    common_fsopen(file, mode, SH_DENYNO)
}

#[no_mangle]
pub unsafe extern "C" fn _wfopen_s(result: *mut *mut c_void, file: *const u16, mode: *const u16) -> c_int {
    common_fopen_s(result, file, mode)
}

// ── freopen.cpp ──────────────────────────────────────────────────────────────

unsafe fn common_freopen<P: OpenChar>(result: *mut *mut c_void, file_name: P, mode: P, stream: Stream, share_flag: c_int) -> c_int {
    if result.is_null() {
        return invalid_errcode(EINVAL);
    }
    *result = null_mut();

    // We do not currently support changing just the mode of an already-open
    // stream (a null file_name in C11); this is a hard failure.
    if file_name.is_null() {
        return invalid_errcode_noexc(EBADF);
    }
    if mode.is_null() {
        return invalid_errcode(EINVAL);
    }
    if !stream.valid() {
        return invalid_errcode(EINVAL);
    }
    if file_name.ascii() == Some(0) {
        return invalid_errcode_noexc(EINVAL);
    }

    let mut return_value: c_int = 0;

    stream.lock();

    // If the stream is in use, try to close it, ignoring possible errors.
    if stream.is_in_use() {
        super::close::_fclose_nolock(stream.public_stream());
    }

    let d = stream.data();
    (*d).ptr = null_mut();
    (*d).base = null_mut();
    (*d).cnt = 0;
    stream.unset_flags(-1);

    // fclose above may have deallocated the stream; we still hold the lock,
    // so just reset the allocated flag to retain ownership.
    stream.set_flags(STREAM_ALLOCATED);

    *result = common_openfile(file_name, mode, share_flag, stream);
    if (*result).is_null() {
        stream.unset_flags(STREAM_ALLOCATED);
        return_value = *_errno();
    }

    stream.unlock();

    return_value
}

#[no_mangle]
pub unsafe extern "C" fn freopen(file_name: *const c_char, mode: *const c_char, public_stream: *mut c_void) -> *mut c_void {
    let mut result_stream: *mut c_void = null_mut();
    common_freopen(&mut result_stream, file_name, mode, Stream::from_file(public_stream), SH_DENYNO);
    result_stream
}

#[no_mangle]
pub unsafe extern "C" fn freopen_s(result: *mut *mut c_void, file_name: *const c_char, mode: *const c_char, public_stream: *mut c_void) -> c_int {
    common_freopen(result, file_name, mode, Stream::from_file(public_stream), SH_SECURE)
}

#[no_mangle]
pub unsafe extern "C" fn _wfreopen(file_name: *const u16, mode: *const u16, public_stream: *mut c_void) -> *mut c_void {
    let mut result_stream: *mut c_void = null_mut();
    common_freopen(&mut result_stream, file_name, mode, Stream::from_file(public_stream), SH_DENYNO);
    result_stream
}

#[no_mangle]
pub unsafe extern "C" fn _wfreopen_s(result: *mut *mut c_void, file_name: *const u16, mode: *const u16, public_stream: *mut c_void) -> c_int {
    common_freopen(result, file_name, mode, Stream::from_file(public_stream), SH_SECURE)
}

// ── fdopen.cpp ───────────────────────────────────────────────────────────────

unsafe fn common_fdopen<P: OpenChar>(fh: c_int, mode: P) -> *mut c_void {
    if mode.is_null() {
        return invalid_ret(EINVAL);
    }

    // _CHECK_FH_RETURN: no handler call, errno only.
    if fh as isize == crate::stdio::NO_CONSOLE_FILENO {
        *_errno() = EBADF;
        return null_mut();
    }
    if !(fh >= 0 && (fh as u32) < (_nhandle as u32)) {
        return invalid_ret(EBADF);
    }
    if super::raw_lowio::osfile_unchecked(fh) & FOPEN == 0 {
        return invalid_ret(EBADF);
    }

    let parsed = parse_mode(mode);
    if !parsed.success {
        return null_mut();
    }

    let stream = allocate_stream();
    if !stream.valid() {
        *_errno() = EMFILE;
        return null_mut();
    }

    stream.set_flags(parsed.stdio_mode);
    (*stream.data()).file = fh as c_long;
    stream.unlock();

    stream.public_stream()
}

#[no_mangle]
pub unsafe extern "C" fn _fdopen(fh: c_int, mode: *const c_char) -> *mut c_void {
    common_fdopen(fh, mode)
}

#[no_mangle]
pub unsafe extern "C" fn _wfdopen(fh: c_int, mode: *const u16) -> *mut c_void {
    common_fdopen(fh, mode)
}
