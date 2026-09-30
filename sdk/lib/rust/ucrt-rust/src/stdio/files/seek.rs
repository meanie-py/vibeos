//! `fseek.cpp`, `ftell.cpp`, `fgetpos.cpp`, `fsetpos.cpp`, `rewind.cpp`.
//!
//! `ftell` is ported line-by-line per the batch brief -- it is the hardest
//! function in this piece: text-mode CRLF adjustment, the UTF-8-translated
//! re-read-the-raw-buffer path, and the "buffer size might really have been
//! `_SMALL_BUFSIZ`" reconstruction when not at EOF.
//!
//! Details that are silent if wrong:
//! * `fseek`'s binary-mode read-only fast track only ever fires for
//!   `SEEK_SET`/`SEEK_CUR` on a buffered, read-only, ANSI-text-mode stream
//!   with a POSITIVE `_cnt` -- a zero `_cnt` is deliberately NOT trusted (the
//!   buffer might not reflect the immediately-preceding file content), so it
//!   falls to the expensive path exactly like a negative one.
//! * The expensive `fseek` path always calls `__acrt_stdio_flush_nolock`
//!   and resets the buffer UNCONDITIONALLY, even for a plain read-only
//!   stream where the flush is a guaranteed no-op -- matching the C exactly
//!   rather than special-casing it away.
//! * An update-mode stream has BOTH `_IOREAD`/`_IOWRITE` cleared after a
//!   successful non-fast-track seek (direction is undetermined until the next
//!   operation); a read-only CRT-buffered stream instead just shrinks
//!   `_bufsiz` to `_SMALL_BUFSIZ` (unless the user called `setvbuf`) so the
//!   next refill after a seek is cheap.
//! * `ftell`'s UTF-8-translated path seeks the lowio pointer back to
//!   `_startpos`, re-reads via raw Win32 `ReadFile` (not `_read`, to avoid
//!   another round of translation), then seeks back to the ORIGINAL lowio
//!   position no matter what it finds -- the stream's lowio position is
//!   restored even on some failure paths below that.
//! * `ftell`'s read-mode text adjustment tries "seek to end and see if we're
//!   already there" FIRST; only when that's false does it reconstruct
//!   `bytes_read` from `_bufsiz` (or `_SMALL_BUFSIZ`, if the buffer was small,
//!   CRT-owned and never `setvbuf`'d) and account for a `\r` believed to have
//!   preceded the buffer's first byte (`FCRLF`).
//! * `common_ftell_nolock<long>` fails with `EINVAL`/`-1` if the 64-bit
//!   result overflows `LONG_MAX` -- it does NOT truncate.

use core::ffi::{c_int, c_long, c_void};
use core::ptr::null_mut;

use crate::ptd_host::PtdHost;
use crate::stdio::{reset_buffer, Stream, INTERNAL_BUFSIZ, SMALL_BUFSIZ};
use crate::stdio_layout::flags::*;
use super::raw_lowio::{FCRLF, FTEXT, TEXT_MODE_UTF16LE, TEXT_MODE_UTF8};

const EINVAL: c_int = 22;
const SEEK_SET: c_int = 0;
const SEEK_CUR: c_int = 1;
const SEEK_END: c_int = 2;
/// `LONG_MAX`: `long` is 32 bits on every Windows target.
const LONG_MAX: i64 = i32::MAX as i64;
const CR: u8 = 13;
const LF: u8 = 10;

// ── fseek.cpp ────────────────────────────────────────────────────────────────

unsafe fn fast_track_binary_read_only_nolock(stream: Stream, mut offset: i64, whence: c_int) -> bool {
    if whence == SEEK_END {
        return false;
    }
    if !stream.has_any_buffer() {
        return false;
    }
    if stream.has_any_of(STREAM_WRITE | STREAM_UPDATE) {
        return false;
    }

    let d = stream.data();
    if (*d).cnt <= 0 {
        return false;
    }

    let fh = stream.lowio_handle();
    if super::raw_lowio::osfile_unchecked(fh) & FTEXT != 0 || super::raw_lowio::textmode(fh) != crate::stdio_layout::TEXT_MODE_ANSI {
        return false;
    }

    if whence == SEEK_SET {
        let lowio_position = super::raw_lowio::lseeki64_nolock(fh, 0, SEEK_CUR);
        if lowio_position < 0 {
            return false;
        }
        let stdio_position = lowio_position - (*d).cnt as i64;
        match offset.checked_sub(stdio_position) {
            Some(v) => offset = v,
            None => return false,
        }
    }

    let minimum_reverse_seek = -(((*d).ptr as isize) - ((*d).base as isize)) as i64;
    let maximum_forward_seek = (*d).cnt as i64;

    if !(minimum_reverse_seek <= offset && offset <= maximum_forward_seek) {
        return false;
    }

    (*d).ptr = (*d).ptr.offset(offset as isize);
    (*d).cnt -= offset as c_int;
    true
}

unsafe fn common_fseek_nolock(stream: Stream, mut offset: i64, mut whence: c_int, ptd: &mut PtdHost) -> c_int {
    if !stream.is_in_use() {
        ptd.errno().set(EINVAL);
        return -1;
    }

    stream.unset_flags(STREAM_EOF);

    if fast_track_binary_read_only_nolock(stream, offset, whence) {
        return 0;
    }

    if whence == SEEK_CUR {
        offset += _ftelli64_nolock_internal(stream.public_stream(), ptd);
        whence = SEEK_SET;
    }

    super::flush::__acrt_stdio_flush_nolock(stream.public_stream(), ptd);
    // __acrt_stdio_flush_nolock will not reset the buffer unless _IOWRITE is
    // set; an update stream mid-read must still abandon its buffer here.
    reset_buffer(stream);

    if stream.has_all_of(STREAM_UPDATE) {
        stream.unset_flags(STREAM_WRITE | STREAM_READ);
    } else if stream.has_all_of(STREAM_READ | STREAM_BUFFER_CRT) && !stream.has_any_of(STREAM_BUFFER_SETVBUF) {
        (*stream.data()).bufsiz = SMALL_BUFSIZ;
    }

    if super::raw_lowio::lseeki64_nolock_internal(stream.lowio_handle(), offset, whence, ptd) == -1 {
        return -1;
    }

    0
}

unsafe fn common_fseek(stream: Stream, offset: i64, whence: c_int, ptd: &mut PtdHost) -> c_int {
    if !stream.valid() {
        ptd.invalid_parameter(EINVAL);
        return -1;
    }
    if !(whence == SEEK_SET || whence == SEEK_CUR || whence == SEEK_END) {
        ptd.invalid_parameter(EINVAL);
        return -1;
    }

    stream.lock();
    let return_value = common_fseek_nolock(stream, offset, whence, ptd);
    stream.unlock();
    return_value
}

#[no_mangle]
pub unsafe extern "C" fn fseek(public_stream: *mut c_void, offset: c_long, whence: c_int) -> c_int {
    let mut ptd = PtdHost::new(null_mut());
    common_fseek(Stream::from_file(public_stream), offset as i64, whence, &mut ptd)
}

#[no_mangle]
pub unsafe extern "C" fn _fseek_nolock(public_stream: *mut c_void, offset: c_long, whence: c_int) -> c_int {
    let mut ptd = PtdHost::new(null_mut());
    common_fseek_nolock(Stream::from_file(public_stream), offset as i64, whence, &mut ptd)
}

#[no_mangle]
pub unsafe extern "C" fn _fseeki64(public_stream: *mut c_void, offset: i64, whence: c_int) -> c_int {
    let mut ptd = PtdHost::new(null_mut());
    common_fseek(Stream::from_file(public_stream), offset, whence, &mut ptd)
}

#[no_mangle]
pub unsafe extern "C" fn _fseeki64_nolock(public_stream: *mut c_void, offset: i64, whence: c_int) -> c_int {
    let mut ptd = PtdHost::new(null_mut());
    common_fseek_nolock(Stream::from_file(public_stream), offset, whence, &mut ptd)
}

// ── ftell.cpp ────────────────────────────────────────────────────────────────

fn buffer_contains_wide_characters(tm: u8) -> bool {
    tm == TEXT_MODE_UTF8 || tm == TEXT_MODE_UTF16LE
}
fn buffer_character_size(tm: u8) -> i64 {
    if buffer_contains_wide_characters(tm) {
        2
    } else {
        1
    }
}

unsafe fn count_newlines_narrow(first: *const u8, last: *const u8) -> i64 {
    let mut n = 0i64;
    let mut it = first;
    while it != last {
        if *it == b'\n' {
            n += 1;
        }
        it = it.add(1);
    }
    n
}
unsafe fn count_newlines_wide(first: *const u8, last: *const u8) -> i64 {
    let typed_first = first as *const u16;
    let typed_last = last as *const u16;
    let mut n = 0i64;
    let mut it = typed_first;
    while it != typed_last {
        if *it == b'\n' as u16 {
            n += 1;
        }
        it = it.add(1);
    }
    n
}
unsafe fn count_newline_bytes(first: *const u8, last: *const u8, tm: u8) -> i64 {
    if buffer_contains_wide_characters(tm) {
        count_newlines_wide(first, last) * 2
    } else {
        count_newlines_narrow(first, last)
    }
}

unsafe fn common_ftell_translated_utf8_nolock(stream: Stream, lowio_position: i64, ptd: &mut PtdHost) -> i64 {
    let fh = stream.lowio_handle();
    let d = stream.data();

    if (*d).cnt == 0 {
        return lowio_position;
    }

    let current_buffer_position = (((*d).ptr as isize) - ((*d).base as isize)) as i64 / 2;

    let file_startpos = super::raw_lowio::startpos(fh);
    let base_buffer_position = super::raw_lowio::lseeki64_internal(fh, file_startpos, SEEK_SET, ptd);
    if base_buffer_position != file_startpos {
        return -1;
    }

    let mut raw_buffer = [0u8; INTERNAL_BUFSIZ as usize];
    let mut bytes_read: u32 = 0;
    let handle = crate::stdio_layout::lowio::osfhnd(fh);
    if !super::raw_lowio::read_file(handle, raw_buffer.as_mut_ptr(), INTERNAL_BUFSIZ as u32, &mut bytes_read) {
        return -1;
    }

    // Seek back to where we were, so the stdio stream is left unmodified.
    if super::raw_lowio::lseeki64_internal(fh, lowio_position, SEEK_SET, ptd) < 0 {
        return -1;
    }

    if current_buffer_position > bytes_read as i64 {
        return -1;
    }

    let raw_first = raw_buffer.as_ptr();
    let raw_last = raw_buffer.as_ptr().add(bytes_read as usize);

    let mut raw_it = raw_first;
    let mut i: i64 = 0;
    while i != current_buffer_position && raw_it < raw_last {
        if *raw_it == CR {
            if raw_it.add(1) < raw_last && *raw_it.add(1) == LF {
                raw_it = raw_it.add(1);
            }
        } else {
            raw_it = raw_it.add(super::raw_lowio::utf8_no_of_trailbytes(*raw_it) as usize);
        }
        i += 1;
        raw_it = raw_it.add(1);
    }

    base_buffer_position + (raw_it as isize - raw_first as isize) as i64
}

unsafe fn common_ftell_read_mode_nolock(stream: Stream, lowio_position: i64, buffer_offset: i64, ptd: &mut PtdHost) -> i64 {
    let fh = stream.lowio_handle();
    let tm = super::raw_lowio::textmode(fh);
    let translation_factor: i64 = if tm == TEXT_MODE_UTF8 { 2 } else { 1 };

    let d = stream.data();
    if (*d).cnt == 0 {
        return lowio_position;
    }

    let mut bytes_read = (*d).cnt as i64 + (((*d).ptr as isize) - ((*d).base as isize)) as i64;

    if super::raw_lowio::osfile_unchecked(fh) & FTEXT == 0 {
        return lowio_position - (bytes_read / translation_factor) + (buffer_offset / translation_factor);
    }

    if super::raw_lowio::lseeki64_internal(fh, 0, SEEK_END, ptd) == lowio_position {
        bytes_read += count_newline_bytes((*d).base.cast(), (*d).base.offset(bytes_read as isize).cast(), tm);
        if stream.ctrl_z() {
            bytes_read += buffer_character_size(tm);
        }
    } else {
        if super::raw_lowio::lseeki64_internal(fh, lowio_position, SEEK_SET, ptd) == -1 {
            return -1;
        }

        if bytes_read <= SMALL_BUFSIZ as i64 && stream.has_crt_buffer() && !stream.has_setvbuf_buffer() {
            bytes_read = SMALL_BUFSIZ as i64;
        } else {
            bytes_read = (*d).bufsiz as i64;
        }

        if super::raw_lowio::osfile_unchecked(fh) & FCRLF != 0 {
            bytes_read += buffer_character_size(tm);
        }
    }

    lowio_position - (bytes_read / translation_factor) + (buffer_offset / translation_factor)
}

unsafe fn common_ftell_nolock_i64(stream: Stream, ptd: &mut PtdHost) -> i64 {
    if !stream.valid() {
        ptd.invalid_parameter(EINVAL);
        return -1;
    }

    let fh = stream.lowio_handle();
    let d = stream.data();
    if (*d).cnt < 0 {
        (*d).cnt = 0;
    }

    let lowio_position = super::raw_lowio::lseeki64_internal(fh, 0, SEEK_CUR, ptd);
    if lowio_position < 0 {
        return -1;
    }

    if !stream.has_big_buffer() {
        return lowio_position - (*d).cnt as i64;
    }

    let tm = super::raw_lowio::textmode(fh);
    let mut buffer_offset = (((*d).ptr as isize) - ((*d).base as isize)) as i64;

    if stream.has_any_of(STREAM_WRITE | STREAM_READ) {
        if tm == TEXT_MODE_UTF8 && super::raw_lowio::utf8translations(fh) {
            return common_ftell_translated_utf8_nolock(stream, lowio_position, ptd);
        }
        if super::raw_lowio::osfile_unchecked(fh) & FTEXT != 0 {
            buffer_offset += count_newline_bytes((*d).base.cast(), (*d).ptr.cast(), tm);
        }
    } else if !stream.has_all_of(STREAM_UPDATE) {
        ptd.errno().set(EINVAL);
        return -1;
    }

    if lowio_position == 0 {
        return buffer_offset;
    }

    if stream.has_all_of(STREAM_READ) {
        return common_ftell_read_mode_nolock(stream, lowio_position, buffer_offset, ptd);
    }

    if tm == TEXT_MODE_UTF8 {
        buffer_offset /= 2;
    }

    lowio_position + buffer_offset
}

unsafe fn common_ftell_nolock_long(stream: Stream, ptd: &mut PtdHost) -> c_long {
    let result = common_ftell_nolock_i64(stream, ptd);
    if result > LONG_MAX {
        ptd.errno().set(EINVAL);
        return -1;
    }
    result as c_long
}

unsafe fn common_ftell_i64(stream: Stream, ptd: &mut PtdHost) -> i64 {
    if !stream.valid() {
        ptd.invalid_parameter(EINVAL);
        return -1;
    }
    stream.lock();
    let r = common_ftell_nolock_i64(stream, ptd);
    stream.unlock();
    r
}

unsafe fn common_ftell_long(stream: Stream, ptd: &mut PtdHost) -> c_long {
    if !stream.valid() {
        ptd.invalid_parameter(EINVAL);
        return -1;
    }
    stream.lock();
    let r = common_ftell_nolock_long(stream, ptd);
    stream.unlock();
    r
}

#[no_mangle]
pub unsafe extern "C" fn ftell(public_stream: *mut c_void) -> c_long {
    let mut ptd = PtdHost::new(null_mut());
    common_ftell_long(Stream::from_file(public_stream), &mut ptd)
}

#[no_mangle]
pub unsafe extern "C" fn _ftell_nolock(public_stream: *mut c_void) -> c_long {
    let mut ptd = PtdHost::new(null_mut());
    common_ftell_nolock_long(Stream::from_file(public_stream), &mut ptd)
}

#[no_mangle]
pub unsafe extern "C" fn _ftelli64(public_stream: *mut c_void) -> i64 {
    let mut ptd = PtdHost::new(null_mut());
    common_ftell_i64(Stream::from_file(public_stream), &mut ptd)
}

#[no_mangle]
pub unsafe extern "C" fn _ftelli64_nolock(public_stream: *mut c_void) -> i64 {
    let mut ptd = PtdHost::new(null_mut());
    common_ftell_nolock_i64(Stream::from_file(public_stream), &mut ptd)
}

#[no_mangle]
pub unsafe extern "C" fn _ftelli64_nolock_internal(public_stream: *mut c_void, ptd: *mut PtdHost) -> i64 {
    common_ftell_nolock_i64(Stream::from_file(public_stream), &mut *ptd)
}

// ── fgetpos.cpp / fsetpos.cpp ────────────────────────────────────────────────
//
// Neither file includes corecrt_internal_ptd_propagation.h: both use the OLD
// (non-PTD) `_VALIDATE_RETURN`.

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
}

unsafe fn invalid_neg1(err: c_int) -> c_int {
    *_errno() = err;
    _invalid_parameter_noinfo();
    -1
}

#[no_mangle]
pub unsafe extern "C" fn fgetpos(stream: *mut c_void, position: *mut i64) -> c_int {
    if stream.is_null() {
        return invalid_neg1(EINVAL);
    }
    if position.is_null() {
        return invalid_neg1(EINVAL);
    }

    *position = _ftelli64(stream);
    if *position == -1 {
        return -1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn fsetpos(stream: *mut c_void, position: *const i64) -> c_int {
    if stream.is_null() {
        return invalid_neg1(EINVAL);
    }
    if position.is_null() {
        return invalid_neg1(EINVAL);
    }
    _fseeki64(stream, *position, SEEK_SET)
}

// ── rewind.cpp ───────────────────────────────────────────────────────────────

unsafe fn rewind_internal(public_stream: *mut c_void, ptd: &mut PtdHost) {
    let stream = Stream::from_file(public_stream);
    if !stream.valid() {
        ptd.invalid_parameter(EINVAL);
        return;
    }

    let fh = stream.lowio_handle();

    stream.lock();

    super::flush::__acrt_stdio_flush_nolock(stream.public_stream(), ptd);
    reset_buffer(stream);

    stream.unset_flags(STREAM_ERROR | STREAM_EOF);
    super::raw_lowio::osfile_safe_clear(fh, super::raw_lowio::FEOFLAG);

    if stream.has_all_of(STREAM_UPDATE) {
        stream.unset_flags(STREAM_READ | STREAM_WRITE);
    }

    if super::raw_lowio::lseek_internal(fh, 0, SEEK_SET, ptd) == -1 {
        stream.set_flags(STREAM_ERROR);
    }

    stream.unlock();
}

#[no_mangle]
pub unsafe extern "C" fn rewind(public_stream: *mut c_void) {
    let mut ptd = PtdHost::new(null_mut());
    rewind_internal(public_stream, &mut ptd);
}
