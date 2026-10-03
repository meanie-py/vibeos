//! `_filbuf.cpp`, `_flsbuf.cpp`.

use core::ffi::{c_char, c_int, c_uint, c_void};

use super::{
    __acrt_stdio_allocate_buffer_nolock, __acrt_should_use_temporary_buffer, osfile_safe, validate_fail,
    EOF, FAPPEND, FEOFLAG, FTEXT, INTERNAL_BUFSIZ, SMALL_BUFSIZ, WEOF,
};
use crate::ptd_host::PtdHost;
use crate::stdio::Stream;
use crate::stdio_layout::flags::*;

const EINVAL: c_int = 22;
const ERRNO_EBADF: c_int = 9;
const ERANGE: c_int = 34;

extern "C" {
    fn _read(fh: c_int, buf: *mut c_void, count: c_uint) -> c_int;
}

// ── _filbuf.cpp ──────────────────────────────────────────────────────────────

/// `__acrt_stdio_refill_and_read_narrow_nolock` (the `common_refill_and_read_nolock<char>`
/// instantiation): refills the buffer (allocating one first if necessary) and
/// returns the next byte, or `EOF`.
#[no_mangle]
pub unsafe extern "C" fn __acrt_stdio_refill_and_read_narrow_nolock(public_stream: *mut c_void) -> c_int {
    let stream = Stream::from_file(public_stream);

    if !stream.valid() {
        validate_fail(EINVAL);
        return EOF;
    }
    if !stream.is_in_use() || stream.is_string_backed() {
        return EOF;
    }
    if stream.has_all_of(STREAM_WRITE) {
        stream.set_flags(STREAM_ERROR);
        return EOF;
    }
    stream.set_flags(STREAM_READ);

    if !stream.has_any_buffer() {
        __acrt_stdio_allocate_buffer_nolock(public_stream);
    }

    let d = stream.data();
    let fh = super::_fileno(public_stream);
    (*d).ptr = (*d).base;
    (*d).cnt = _read(fh, (*d).base.cast(), (*d).bufsiz as c_uint);

    if (*d).cnt == 0 || (*d).cnt == -1 {
        stream.set_flags(if (*d).cnt != 0 { STREAM_ERROR } else { STREAM_EOF });
        (*d).cnt = 0;
        return EOF;
    }

    if !stream.has_any_of(STREAM_WRITE | STREAM_UPDATE)
        && (osfile_safe(fh) & (FTEXT | FEOFLAG)) == (FTEXT | FEOFLAG)
    {
        stream.set_flags(STREAM_CTRLZ);
    }

    // Following an `fseek` on a read-only stream, `_bufsiz` may still be the
    // small size a scan-mode reposition uses; restore it so a later refill
    // fills the whole (larger) CRT buffer.
    if (*d).bufsiz == SMALL_BUFSIZ && stream.has_crt_buffer() && !stream.has_all_of(STREAM_BUFFER_SETVBUF) {
        (*d).bufsiz = INTERNAL_BUFSIZ;
    }

    (*d).cnt -= 1;
    let c = *(*d).ptr as u8;
    (*d).ptr = (*d).ptr.add(1);
    c as c_int
}

/// `__acrt_stdio_refill_and_read_wide_nolock`. Returns the next UTF-16 code
/// unit widened to `int` (0..=0xFFFF -- the eof sentinel is `WEOF`, 0xFFFF,
/// not -1: `__acrt_stdio_char_traits<wchar_t>::eof` is `WEOF`, a `wint_t`).
/// Handles a code unit split across the old and new buffers: if exactly one
/// byte remained before the refill, its low-order byte is combined with the
/// first byte of the newly-filled buffer.
#[no_mangle]
pub unsafe extern "C" fn __acrt_stdio_refill_and_read_wide_nolock(public_stream: *mut c_void) -> c_int {
    let stream = Stream::from_file(public_stream);

    if !stream.valid() {
        validate_fail(EINVAL);
        return WEOF as c_int;
    }
    if !stream.is_in_use() || stream.is_string_backed() {
        return WEOF as c_int;
    }
    if stream.has_all_of(STREAM_WRITE) {
        stream.set_flags(STREAM_ERROR);
        return WEOF as c_int;
    }
    stream.set_flags(STREAM_READ);

    if !stream.has_any_buffer() {
        __acrt_stdio_allocate_buffer_nolock(public_stream);
    }

    let d = stream.data();
    let split = (*d).cnt == 1;
    let leftover = if split { *(*d).ptr as u8 } else { 0 };

    let fh = super::_fileno(public_stream);
    (*d).ptr = (*d).base;
    (*d).cnt = _read(fh, (*d).base.cast(), (*d).bufsiz as c_uint);

    if (*d).cnt == 0 || (*d).cnt == 1 || (*d).cnt == -1 {
        stream.set_flags(if (*d).cnt != 0 { STREAM_ERROR } else { STREAM_EOF });
        (*d).cnt = 0;
        return WEOF as c_int;
    }

    if !stream.has_any_of(STREAM_WRITE | STREAM_UPDATE)
        && (osfile_safe(fh) & (FTEXT | FEOFLAG)) == (FTEXT | FEOFLAG)
    {
        stream.set_flags(STREAM_CTRLZ);
    }

    if (*d).bufsiz == SMALL_BUFSIZ && stream.has_crt_buffer() && !stream.has_all_of(STREAM_BUFFER_SETVBUF) {
        (*d).bufsiz = INTERNAL_BUFSIZ;
    }

    let result: u16 = if split {
        let high = *(*d).ptr as u8;
        (*d).cnt -= 1;
        (*d).ptr = (*d).ptr.add(1);
        ((high as u16) << 8) | leftover as u16
    } else {
        let value = ((*d).ptr as *const u16).read_unaligned() & 0xffff;
        (*d).cnt -= 2;
        (*d).ptr = (*d).ptr.add(2);
        value
    };
    result as c_int
}

// ── _flsbuf.cpp ──────────────────────────────────────────────────────────────

extern "C" {
    fn _write_internal(fh: c_int, buf: *const c_void, count: c_uint, ptd: *mut PtdHost) -> c_int;
    fn _get_osfhandle(fh: c_int) -> isize;
    fn _lseeki64(fh: c_int, offset: i64, origin: c_int) -> i64;
}

extern "system" {
    fn SetFilePointerEx(handle: *mut c_void, distance: i64, new_pos: *mut i64, method: c_uint) -> c_int;
    fn GetFileSizeEx(handle: *mut c_void, size: *mut i64) -> c_int;
}

const INVALID_HANDLE_VALUE: isize = -1;
const FILE_CURRENT: c_uint = 1;
const SEEK_END: c_int = 2;

/// `stream_is_at_end_of_file_nolock`.
unsafe fn stream_is_at_end_of_file_nolock(stream: Stream) -> bool {
    if stream.has_any_of(STREAM_EOF) {
        return true;
    }

    let d = stream.data();
    if stream.has_big_buffer() && (*d).ptr == (*d).base {
        return false;
    }

    let os_handle = _get_osfhandle(stream.lowio_handle()) as *mut c_void;
    if os_handle as isize == INVALID_HANDLE_VALUE {
        return false;
    }

    let mut current_position: i64 = 0;
    if SetFilePointerEx(os_handle, 0, &mut current_position, FILE_CURRENT) == 0 {
        return false;
    }

    let mut eof_position: i64 = 0;
    if GetFileSizeEx(os_handle, &mut eof_position) == 0 {
        return false;
    }

    current_position == eof_position
}

/// `write_buffer_nolock<char>`.
///
/// **Microsoft quirk, preserved** (see module doc): on the FAPPEND re-seek
/// failure path, the C `return`s the `eof` sentinel (nonzero) from a function
/// that returns `bool`, so this reports SUCCESS (`true`) to
/// `common_flush_and_write_nolock` even though `_IOERROR` was just set and
/// the character is never written to the buffer.
unsafe fn write_buffer_nolock_narrow(c: u8, stream: Stream, ptd: *mut PtdHost) -> bool {
    let fh = super::_fileno(stream.public_stream());
    let d = stream.data();

    if stream.has_big_buffer() {
        let chars_to_write = (*d).ptr.offset_from((*d).base) as c_int;
        (*d).ptr = (*d).base.add(1);
        (*d).cnt = (*d).bufsiz - 1;

        let mut chars_written = 0;
        if chars_to_write > 0 {
            chars_written = _write_internal(fh, (*d).base.cast(), chars_to_write as c_uint, ptd);
        } else if osfile_safe(fh) & FAPPEND != 0 && _lseeki64(fh, 0, SEEK_END) == -1 {
            stream.set_flags(STREAM_ERROR);
            return true; // quirk: see doc comment above.
        }

        *(*d).base = c as c_char;
        chars_written == chars_to_write
    } else {
        _write_internal(fh, core::ptr::addr_of!(c).cast(), 1, ptd) == 1
    }
}

/// `write_buffer_nolock<wchar_t>`. Same quirk as the narrow form.
unsafe fn write_buffer_nolock_wide(c: u16, stream: Stream, ptd: *mut PtdHost) -> bool {
    let fh = super::_fileno(stream.public_stream());
    let d = stream.data();

    if stream.has_big_buffer() {
        let chars_to_write = (*d).ptr.offset_from((*d).base) as c_int;
        (*d).ptr = (*d).base.add(2);
        (*d).cnt = (*d).bufsiz - 2;

        let mut chars_written = 0;
        if chars_to_write > 0 {
            chars_written = _write_internal(fh, (*d).base.cast(), chars_to_write as c_uint, ptd);
        } else if osfile_safe(fh) & FAPPEND != 0 && _lseeki64(fh, 0, SEEK_END) == -1 {
            stream.set_flags(STREAM_ERROR);
            return true; // quirk: see doc comment above.
        }

        ((*d).base as *mut u16).write_unaligned(c);
        chars_written == chars_to_write
    } else {
        _write_internal(fh, core::ptr::addr_of!(c).cast(), 2, ptd) == 2
    }
}

/// `common_flush_and_write_nolock<char>`.
unsafe fn common_flush_and_write_narrow(c: c_int, stream: Stream, ptd: *mut PtdHost) -> c_int {
    if !stream.has_any_of(STREAM_WRITE | STREAM_UPDATE) {
        (*ptd).errno().set(ERRNO_EBADF);
        stream.set_flags(STREAM_ERROR);
        return EOF;
    } else if stream.is_string_backed() {
        (*ptd).errno().set(ERANGE);
        stream.set_flags(STREAM_ERROR);
        return EOF;
    }

    if stream.has_any_of(STREAM_READ) {
        let switch_to_write = stream_is_at_end_of_file_nolock(stream);
        (*stream.data()).cnt = 0;
        if switch_to_write {
            (*stream.data()).ptr = (*stream.data()).base;
            stream.unset_flags(STREAM_READ);
        } else {
            stream.set_flags(STREAM_ERROR);
            return EOF;
        }
    }

    stream.set_flags(STREAM_WRITE);
    stream.unset_flags(STREAM_EOF);
    (*stream.data()).cnt = 0;

    if !stream.has_any_buffer() && !__acrt_should_use_temporary_buffer(stream.public_stream()) {
        __acrt_stdio_allocate_buffer_nolock(stream.public_stream());
    }

    let masked = (c as u32) & 0xFF;
    if !write_buffer_nolock_narrow(masked as u8, stream, ptd) {
        stream.set_flags(STREAM_ERROR);
        return EOF;
    }
    masked as c_int
}

/// `common_flush_and_write_nolock<wchar_t>`.
unsafe fn common_flush_and_write_wide(c: c_int, stream: Stream, ptd: *mut PtdHost) -> c_int {
    if !stream.has_any_of(STREAM_WRITE | STREAM_UPDATE) {
        (*ptd).errno().set(ERRNO_EBADF);
        stream.set_flags(STREAM_ERROR);
        return WEOF as c_int;
    } else if stream.is_string_backed() {
        (*ptd).errno().set(ERANGE);
        stream.set_flags(STREAM_ERROR);
        return WEOF as c_int;
    }

    if stream.has_any_of(STREAM_READ) {
        let switch_to_write = stream_is_at_end_of_file_nolock(stream);
        (*stream.data()).cnt = 0;
        if switch_to_write {
            (*stream.data()).ptr = (*stream.data()).base;
            stream.unset_flags(STREAM_READ);
        } else {
            stream.set_flags(STREAM_ERROR);
            return WEOF as c_int;
        }
    }

    stream.set_flags(STREAM_WRITE);
    stream.unset_flags(STREAM_EOF);
    (*stream.data()).cnt = 0;

    if !stream.has_any_buffer() && !__acrt_should_use_temporary_buffer(stream.public_stream()) {
        __acrt_stdio_allocate_buffer_nolock(stream.public_stream());
    }

    let masked = (c as u32) & 0xFFFF;
    if !write_buffer_nolock_wide(masked as u16, stream, ptd) {
        stream.set_flags(STREAM_ERROR);
        return WEOF as c_int;
    }
    masked as c_int
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_stdio_flush_and_write_narrow_nolock(
    c: c_int,
    public_stream: *mut c_void,
    ptd: *mut PtdHost,
) -> c_int {
    common_flush_and_write_narrow(c, Stream::from_file(public_stream), ptd)
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_stdio_flush_and_write_wide_nolock(
    c: c_int,
    public_stream: *mut c_void,
    ptd: *mut PtdHost,
) -> c_int {
    common_flush_and_write_wide(c, Stream::from_file(public_stream), ptd)
}
