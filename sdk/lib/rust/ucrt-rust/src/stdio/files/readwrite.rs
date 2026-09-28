//! `fread.cpp`, `fwrite.cpp`.
//!
//! Details that are silent if wrong:
//! * `fread_s`/`_fread_nolock_s` sentinel-fill the WHOLE destination buffer
//!   (release build: the fill byte is 0) on every early-return path EXCEPT
//!   the ordinary EOF/error/short-read ones -- only the "buffer too small"
//!   (`ERANGE`) and the pre-lock null-stream/overflow checks fill it. A
//!   normal short read leaves the buffer holding whatever was actually read.
//! * A buffered read/write only touches `stream->_cnt`/`_ptr` (not a lowio
//!   call) when the stream already has data buffered; once a request is
//!   large enough to bypass the buffer, the buffer is explicitly reset
//!   (`fread`) or flushed (`fwrite`) first so a later seek can't assume it
//!   holds anything.
//! * `fwrite`'s big-buffer path treats a NEGATIVE `_cnt` as the "flush
//!   between read and write" bug class and reports it as `_IOERROR`, exactly
//!   like `fread`'s; it separately refuses to write into a buffer that still
//!   has `_IOREAD` set (an update stream that was just read from) rather
//!   than silently mixing directions.
//! * `_write_internal` returns `int`, assigned to an `unsigned`: a `-1`
//!   result (failure) becomes `UINT_MAX` in the C, which is why the check is
//!   an equality against `UINT_MAX` rather than `< 0`. Ported as: check `-1`
//!   on the raw `c_int` result before ever widening it.
//! * A **successful** `_write_internal` may report MORE bytes than requested
//!   (Unicode translation can expand what's on disk); only a SHORT write
//!   (`bytes_actually_written < bytes_to_write`) is an error.

use core::ffi::{c_int, c_void};
use core::ptr::null_mut;

use crate::ptd_host::PtdHost;
use crate::stdio::{reset_buffer, Stream, INTERNAL_BUFSIZ};
use crate::stdio_layout::flags::*;

const EINVAL: c_int = 22;
const ERANGE: c_int = 34;
const EOF: c_int = -1;
/// `_CRT_UNBOUNDED_BUFFER_SIZE`.
const UNBOUNDED: usize = usize::MAX;
/// Release build (`_DEBUG` undefined): `_BUFFER_FILL_PATTERN` is 0.
const BUFFER_FILL_PATTERN: u8 = 0;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    fn _lock_file(stream: *mut c_void);
    fn _unlock_file(stream: *mut c_void);
    fn memset(dst: *mut c_void, val: c_int, n: usize) -> *mut c_void;
    fn memcpy_s(dst: *mut c_void, dst_size: usize, src: *const c_void, src_size: usize) -> c_int;

    // Owned by the `bufio` piece.
    fn __acrt_stdio_refill_and_read_narrow_nolock(stream: *mut c_void) -> c_int;
    fn __acrt_stdio_flush_and_write_narrow_nolock(c: c_int, stream: *mut c_void, ptd: *mut PtdHost) -> c_int;
    fn __acrt_stdio_begin_temporary_buffering_nolock(stream: *mut c_void) -> bool;
    fn __acrt_stdio_end_temporary_buffering_nolock(flag: bool, stream: *mut c_void, ptd: *mut PtdHost);
}

unsafe fn fill_if_bounded(buffer: *mut c_void, buffer_size: usize) {
    if buffer_size != UNBOUNDED {
        memset(buffer, BUFFER_FILL_PATTERN as c_int, buffer_size);
    }
}

unsafe fn invalid0(err: c_int) -> usize {
    *_errno() = err;
    _invalid_parameter_noinfo();
    0
}

// ── fread.cpp ────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn fread_s(buffer: *mut c_void, buffer_size: usize, element_size: usize, element_count: usize, stream: *mut c_void) -> usize {
    if element_size == 0 || element_count == 0 {
        return 0;
    }

    if stream.is_null() {
        fill_if_bounded(buffer, buffer_size);
        return invalid0(EINVAL);
    }

    _lock_file(stream);
    let return_value = _fread_nolock_s(buffer, buffer_size, element_size, element_count, stream);
    _unlock_file(stream);
    return_value
}

#[no_mangle]
pub unsafe extern "C" fn _fread_nolock_s(buffer: *mut c_void, buffer_size: usize, element_size: usize, element_count: usize, public_stream: *mut c_void) -> usize {
    let stream = Stream::from_file(public_stream);

    if element_size == 0 || element_count == 0 {
        return 0;
    }

    if buffer.is_null() {
        return invalid0(EINVAL);
    }

    if !stream.valid() || element_count > (usize::MAX / element_size) {
        fill_if_bounded(buffer, buffer_size);
        if !stream.valid() {
            return invalid0(EINVAL);
        }
        if element_count > (usize::MAX / element_size) {
            return invalid0(EINVAL);
        }
    }

    let mut stream_buffer_size: usize = if stream.has_any_buffer() { (*stream.data()).bufsiz as usize } else { INTERNAL_BUFSIZ as usize };

    let total_bytes = element_size * element_count;
    let mut data = buffer as *mut u8;
    let mut remaining_bytes = total_bytes;
    let mut remaining_buffer = buffer_size;

    while remaining_bytes != 0 {
        let d = stream.data();
        if stream.has_any_buffer() && (*d).cnt != 0 {
            if (*d).cnt < 0 {
                stream.set_flags(STREAM_ERROR);
                return (total_bytes - remaining_bytes) / element_size;
            }

            let bytes_to_read = core::cmp::min(remaining_bytes, (*d).cnt as usize);

            if bytes_to_read > remaining_buffer {
                fill_if_bounded(buffer, buffer_size);
                return invalid0(ERANGE);
            }

            memcpy_s(data.cast(), remaining_buffer, (*d).ptr.cast(), bytes_to_read);

            remaining_bytes -= bytes_to_read;
            (*d).cnt -= bytes_to_read as c_int;
            (*d).ptr = (*d).ptr.add(bytes_to_read);
            data = data.add(bytes_to_read);
            remaining_buffer -= bytes_to_read;
        } else if remaining_bytes >= stream_buffer_size {
            let maximum_bytes_to_read = core::cmp::min(remaining_bytes, i32::MAX as usize) as u32;
            let bytes_to_read = if stream_buffer_size != 0 {
                maximum_bytes_to_read - maximum_bytes_to_read % (stream_buffer_size as u32)
            } else {
                maximum_bytes_to_read
            };

            if bytes_to_read as usize > remaining_buffer {
                fill_if_bounded(buffer, buffer_size);
                return invalid0(ERANGE);
            }

            reset_buffer(stream);

            let bytes_read = super::raw_lowio::read_nolock(stream.lowio_handle(), data.cast(), bytes_to_read);
            if bytes_read == 0 {
                stream.set_flags(STREAM_EOF);
                return (total_bytes - remaining_bytes) / element_size;
            } else if bytes_read < 0 {
                stream.set_flags(STREAM_ERROR);
                return (total_bytes - remaining_bytes) / element_size;
            }

            remaining_bytes -= bytes_read as usize;
            data = data.add(bytes_read as usize);
            remaining_buffer -= bytes_read as usize;
        } else {
            let c = __acrt_stdio_refill_and_read_narrow_nolock(stream.public_stream());
            if c == EOF {
                return (total_bytes - remaining_bytes) / element_size;
            }

            if remaining_buffer == 0 {
                fill_if_bounded(buffer, buffer_size);
                return invalid0(ERANGE);
            }

            *data = c as u8;
            data = data.add(1);
            remaining_bytes -= 1;
            remaining_buffer -= 1;

            stream_buffer_size = (*stream.data()).bufsiz as usize;
        }
    }

    element_count
}

#[no_mangle]
pub unsafe extern "C" fn fread(buffer: *mut c_void, element_size: usize, element_count: usize, stream: *mut c_void) -> usize {
    fread_s(buffer, UNBOUNDED, element_size, element_count, stream)
}

#[no_mangle]
pub unsafe extern "C" fn _fread_nolock(buffer: *mut c_void, element_size: usize, element_count: usize, stream: *mut c_void) -> usize {
    _fread_nolock_s(buffer, UNBOUNDED, element_size, element_count, stream)
}

// ── fwrite.cpp ───────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _fwrite_internal(buffer: *const c_void, size: usize, count: usize, stream: *mut c_void, ptd: *mut PtdHost) -> usize {
    if size == 0 || count == 0 {
        return 0;
    }

    let ptd_ref = &mut *ptd;
    if stream.is_null() {
        ptd_ref.invalid_parameter(EINVAL);
        return 0;
    }

    crate::stdio::with_stream_locked(stream, || {
        let flag = __acrt_stdio_begin_temporary_buffering_nolock(stream);
        let result = _fwrite_nolock_internal(buffer, size, count, stream, ptd);
        __acrt_stdio_end_temporary_buffering_nolock(flag, stream, ptd);
        result
    })
}

#[no_mangle]
pub unsafe extern "C" fn fwrite(buffer: *const c_void, size: usize, count: usize, stream: *mut c_void) -> usize {
    let mut ptd = PtdHost::new(null_mut());
    _fwrite_internal(buffer, size, count, stream, &mut ptd)
}

#[no_mangle]
pub unsafe extern "C" fn _fwrite_nolock_internal(buffer: *const c_void, element_size: usize, element_count: usize, public_stream: *mut c_void, ptd: *mut PtdHost) -> usize {
    if element_size == 0 || element_count == 0 {
        return 0;
    }

    let ptd = &mut *ptd;
    let stream = Stream::from_file(public_stream);

    if !stream.valid() {
        ptd.invalid_parameter(EINVAL);
        return 0;
    }
    if buffer.is_null() {
        ptd.invalid_parameter(EINVAL);
        return 0;
    }
    if element_count > (usize::MAX / element_size) {
        ptd.invalid_parameter(EINVAL);
        return 0;
    }

    let mut stream_buffer_size: usize = if stream.has_any_buffer() { (*stream.data()).bufsiz as usize } else { INTERNAL_BUFSIZ as usize };

    let total_bytes = element_size * element_count;
    let mut data = buffer as *const u8;
    let mut remaining_bytes = total_bytes;

    while remaining_bytes != 0 {
        let d = stream.data();
        if stream.has_big_buffer() && (*d).cnt != 0 {
            if (*d).cnt < 0 {
                stream.set_flags(STREAM_ERROR);
                return (total_bytes - remaining_bytes) / element_size;
            }

            if stream.has_any_of(STREAM_READ) {
                // "Flush between consecutive read and write."
                return (total_bytes - remaining_bytes) / element_size;
            }

            let bytes_to_write = core::cmp::min(remaining_bytes, (*d).cnt as usize);

            core::ptr::copy_nonoverlapping(data, (*d).ptr.cast(), bytes_to_write);

            remaining_bytes -= bytes_to_write;
            (*d).cnt -= bytes_to_write as c_int;
            (*d).ptr = (*d).ptr.add(bytes_to_write);
            data = data.add(bytes_to_write);
        } else if remaining_bytes >= stream_buffer_size {
            if stream.has_big_buffer() && super::flush::__acrt_stdio_flush_nolock(stream.public_stream(), ptd) != 0 {
                return (total_bytes - remaining_bytes) / element_size;
            }

            let max_bytes_to_write = if stream_buffer_size > 0 {
                remaining_bytes - remaining_bytes % stream_buffer_size
            } else {
                remaining_bytes
            };
            let bytes_to_write = core::cmp::min(max_bytes_to_write, (u32::MAX - 1) as usize) as u32;

            let raw = super::raw_lowio::write_internal(stream.lowio_handle(), data.cast(), bytes_to_write, ptd);
            if raw == -1 {
                // UINT_MAX, per the C's `unsigned const bytes_actually_written`.
                stream.set_flags(STREAM_ERROR);
                return (total_bytes - remaining_bytes) / element_size;
            }
            let bytes_actually_written = raw as u32;

            // A write can report MORE bytes than requested (Unicode
            // translation); only a genuinely short write is an error.
            let bytes_written = if bytes_actually_written > bytes_to_write { bytes_to_write } else { bytes_actually_written };

            remaining_bytes -= bytes_written as usize;
            data = data.add(bytes_written as usize);

            if bytes_actually_written < bytes_to_write {
                stream.set_flags(STREAM_ERROR);
                return (total_bytes - remaining_bytes) / element_size;
            }
        } else {
            if __acrt_stdio_flush_and_write_narrow_nolock(*data as c_int, stream.public_stream(), ptd) == EOF {
                return (total_bytes - remaining_bytes) / element_size;
            }

            data = data.add(1);
            remaining_bytes -= 1;

            let bufsiz = (*stream.data()).bufsiz;
            stream_buffer_size = if bufsiz > 0 { bufsiz as usize } else { 1 };
        }
    }

    element_count
}

#[no_mangle]
pub unsafe extern "C" fn _fwrite_nolock(buffer: *const c_void, element_size: usize, element_count: usize, public_stream: *mut c_void) -> usize {
    let mut ptd = PtdHost::new(null_mut());
    _fwrite_nolock_internal(buffer, element_size, element_count, public_stream, &mut ptd)
}
