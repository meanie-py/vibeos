//! `fgetc.cpp`, `fgetwc.cpp`, `fputc.cpp`, `fputwc.cpp`, `getw.cpp`,
//! `putw.cpp`, `ungetc.cpp`, `ungetwc.cpp`: everything built directly on a
//! single character.

use core::ffi::{c_char, c_int, c_void};
use core::ptr::{addr_of, addr_of_mut};

use super::{
    __acrt_stdio_allocate_buffer_nolock, __acrt_stdio_flush_and_write_narrow_nolock,
    __acrt_stdio_flush_and_write_wide_nolock, __acrt_stdio_refill_and_read_narrow_nolock,
    __acrt_stdio_refill_and_read_wide_nolock, feof, ferror, osfile_safe, stdin_ptr, stdout_ptr,
    validate_fail, validate_stream_ansi_narrow, validate_stream_ansi_narrow_ptd, EOF, FTEXT,
    MB_LEN_MAX, TEXT_MODE_UTF16LE, TEXT_MODE_UTF8, WEOF,
};
use crate::ptd_host::PtdHost;
use crate::stdio::{with_stream_locked, Stream};
use crate::stdio_layout::{textmode_safe, TEXT_MODE_ANSI};

const EINVAL: c_int = 22;
const EILSEQ: c_int = 42;

extern "C" {
    fn isleadbyte(c: c_int) -> c_int;
    fn mbtowc(dst: *mut u16, src: *const c_char, src_size_bytes: usize) -> c_int;
    fn _wctomb_internal(
        size_converted: *mut c_int,
        mbch: *mut u8,
        size_in_bytes: usize,
        wch: u16,
        ptd: *mut PtdHost,
    ) -> c_int;
}

// ── fgetc.cpp ────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _fgetc_nolock(public_stream: *mut c_void) -> c_int {
    let stream = Stream::from_file(public_stream);
    if !stream.valid() {
        validate_fail(EINVAL);
        return EOF;
    }

    let d = stream.data();
    (*d).cnt -= 1;
    if (*d).cnt < 0 {
        return __acrt_stdio_refill_and_read_narrow_nolock(public_stream);
    }

    let c = *(*d).ptr as u8;
    (*d).ptr = (*d).ptr.add(1);
    c as c_int
}

#[no_mangle]
pub unsafe extern "C" fn _getc_nolock(stream: *mut c_void) -> c_int {
    _fgetc_nolock(stream)
}

#[no_mangle]
pub unsafe extern "C" fn fgetc(public_stream: *mut c_void) -> c_int {
    let stream = Stream::from_file(public_stream);
    if !stream.valid() {
        validate_fail(EINVAL);
        return EOF;
    }
    with_stream_locked(public_stream, || {
        if !validate_stream_ansi_narrow(stream) {
            return EOF;
        }
        _fgetc_nolock(public_stream)
    })
}

#[no_mangle]
pub unsafe extern "C" fn getc(stream: *mut c_void) -> c_int {
    fgetc(stream)
}

#[no_mangle]
pub unsafe extern "C" fn _fgetchar() -> c_int {
    fgetc(stdin_ptr())
}

#[no_mangle]
pub unsafe extern "C" fn getchar() -> c_int {
    _fgetchar()
}

// ── fgetwc.cpp ───────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _fgetwc_nolock(public_stream: *mut c_void) -> u16 {
    let stream = Stream::from_file(public_stream);
    let fh = super::_fileno(public_stream);

    // Real file, Unicode text mode (UTF-16LE or UTF-8 -- lowio translates
    // UTF-8 to UTF-16 on read, so both are read here as raw UTF-16 bytes).
    if !stream.is_string_backed() && textmode_safe(fh) != TEXT_MODE_ANSI {
        let mut wc: u16 = 0;
        let bytes = addr_of_mut!(wc).cast::<u8>();
        for i in 0..core::mem::size_of::<u16>() {
            let c = _getc_nolock(public_stream);
            if c == EOF {
                return WEOF;
            }
            *bytes.add(i) = c as u8;
        }
        return wc;
    }

    // Real file, ANSI text (multibyte) mode.
    if !stream.is_string_backed() && (osfile_safe(fh) & FTEXT) != 0 {
        let mut size: usize = 1;
        let mut mbc = [0u8; 4];

        let ch = _getc_nolock(public_stream);
        if ch == EOF {
            return WEOF;
        }
        mbc[0] = ch as u8;

        if isleadbyte(mbc[0] as c_int) != 0 {
            let ch2 = _getc_nolock(public_stream);
            if ch2 == EOF {
                ungetc(mbc[0] as c_int, public_stream);
                return WEOF;
            }
            mbc[1] = ch2 as u8;
            size = 2;
        }

        let mut wch: u16 = 0;
        if mbtowc(&mut wch, mbc.as_ptr().cast(), size) == -1 {
            *super::_errno() = EILSEQ;
            return WEOF;
        }
        return wch;
    }

    // Binary (Unicode) mode.
    let d = stream.data();
    if (*d).cnt >= core::mem::size_of::<u16>() as c_int {
        (*d).cnt -= core::mem::size_of::<u16>() as c_int;
        let v = ((*d).ptr as *const u16).read_unaligned();
        (*d).ptr = (*d).ptr.add(2);
        v
    } else {
        __acrt_stdio_refill_and_read_wide_nolock(public_stream) as u16
    }
}

#[no_mangle]
pub unsafe extern "C" fn _getwc_nolock(stream: *mut c_void) -> u16 {
    _fgetwc_nolock(stream)
}

#[no_mangle]
pub unsafe extern "C" fn fgetwc(stream: *mut c_void) -> u16 {
    if stream.is_null() {
        validate_fail(EINVAL);
        return WEOF;
    }
    with_stream_locked(stream, || _fgetwc_nolock(stream))
}

#[no_mangle]
pub unsafe extern "C" fn getwc(stream: *mut c_void) -> u16 {
    fgetwc(stream)
}

#[no_mangle]
pub unsafe extern "C" fn _fgetwchar() -> u16 {
    fgetwc(stdin_ptr())
}

#[no_mangle]
pub unsafe extern "C" fn getwchar() -> u16 {
    _fgetwchar()
}

// ── fputc.cpp ────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _fputc_nolock_internal(c: c_int, public_stream: *mut c_void, ptd: *mut PtdHost) -> c_int {
    let stream = Stream::from_file(public_stream);
    let d = stream.data();
    (*d).cnt -= 1;

    if (*d).cnt < 0 {
        return __acrt_stdio_flush_and_write_narrow_nolock(c, public_stream, ptd);
    }

    let char_value = c as u8;
    *(*d).ptr = char_value as c_char;
    (*d).ptr = (*d).ptr.add(1);
    char_value as c_int
}

#[no_mangle]
pub unsafe extern "C" fn _fputc_nolock(c: c_int, public_stream: *mut c_void) -> c_int {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    _fputc_nolock_internal(c, public_stream, &mut ptd)
}

#[no_mangle]
pub unsafe extern "C" fn _putc_nolock(c: c_int, stream: *mut c_void) -> c_int {
    _fputc_nolock(c, stream)
}

unsafe fn fputc_internal(c: c_int, stream: *mut c_void, ptd: *mut PtdHost) -> c_int {
    if stream.is_null() {
        (*ptd).invalid_parameter(EINVAL);
        return EOF;
    }
    with_stream_locked(stream, || {
        if !validate_stream_ansi_narrow_ptd(Stream::from_file(stream), ptd) {
            return EOF;
        }
        _fputc_nolock_internal(c, stream, ptd)
    })
}

#[no_mangle]
pub unsafe extern "C" fn fputc(c: c_int, stream: *mut c_void) -> c_int {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    fputc_internal(c, stream, &mut ptd)
}

#[no_mangle]
pub unsafe extern "C" fn putc(c: c_int, stream: *mut c_void) -> c_int {
    fputc(c, stream)
}

#[no_mangle]
pub unsafe extern "C" fn _fputchar(c: c_int) -> c_int {
    fputc(c, stdout_ptr())
}

#[no_mangle]
pub unsafe extern "C" fn putchar(c: c_int) -> c_int {
    _fputchar(c)
}

// ── fputwc.cpp ───────────────────────────────────────────────────────────────

unsafe fn fputwc_binary_nolock(c: u16, stream: Stream, ptd: *mut PtdHost) -> u16 {
    let d = stream.data();
    (*d).cnt -= core::mem::size_of::<u16>() as c_int;

    if (*d).cnt < 0 {
        return __acrt_stdio_flush_and_write_wide_nolock(c as c_int, stream.public_stream(), ptd) as u16;
    }

    ((*d).ptr as *mut u16).write_unaligned(c);
    (*d).ptr = (*d).ptr.add(2);
    c
}

#[no_mangle]
pub unsafe extern "C" fn _fputwc_nolock_internal(c: u16, public_stream: *mut c_void, ptd: *mut PtdHost) -> u16 {
    let stream = Stream::from_file(public_stream);

    if stream.is_string_backed() {
        return fputwc_binary_nolock(c, stream, ptd);
    }

    let fh = super::_fileno(public_stream);
    let text_mode = textmode_safe(fh);
    if text_mode == TEXT_MODE_UTF16LE || text_mode == TEXT_MODE_UTF8 {
        return fputwc_binary_nolock(c, stream, ptd);
    }
    if osfile_safe(fh) & FTEXT == 0 {
        return fputwc_binary_nolock(c, stream, ptd);
    }

    // ANSI text mode: translate to multibyte and write byte-by-byte.
    let mut mbc = [0u8; MB_LEN_MAX];
    let mut size: c_int = 0;
    if _wctomb_internal(&mut size, mbc.as_mut_ptr(), MB_LEN_MAX, c, ptd) != 0 {
        return WEOF;
    }
    for i in 0..size as usize {
        if _fputc_nolock_internal(mbc[i] as c_int, public_stream, ptd) == EOF {
            return WEOF;
        }
    }
    c
}

#[no_mangle]
pub unsafe extern "C" fn _fputwc_nolock(c: u16, public_stream: *mut c_void) -> u16 {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    _fputwc_nolock_internal(c, public_stream, &mut ptd)
}

#[no_mangle]
pub unsafe extern "C" fn _putwc_nolock(c: u16, stream: *mut c_void) -> u16 {
    _fputwc_nolock(c, stream)
}

unsafe fn fputwc_internal(c: u16, stream: *mut c_void, ptd: *mut PtdHost) -> u16 {
    if stream.is_null() {
        (*ptd).invalid_parameter(EINVAL);
        return WEOF;
    }
    with_stream_locked(stream, || _fputwc_nolock_internal(c, stream, ptd))
}

#[no_mangle]
pub unsafe extern "C" fn fputwc(c: u16, stream: *mut c_void) -> u16 {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    fputwc_internal(c, stream, &mut ptd)
}

#[no_mangle]
pub unsafe extern "C" fn putwc(c: u16, stream: *mut c_void) -> u16 {
    fputwc(c, stream)
}

#[no_mangle]
pub unsafe extern "C" fn _fputwchar(c: u16) -> u16 {
    fputwc(c, stdout_ptr())
}

#[no_mangle]
pub unsafe extern "C" fn putwchar(c: u16) -> u16 {
    _fputwchar(c)
}

// ── getw.cpp / putw.cpp ──────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _getw(stream: *mut c_void) -> c_int {
    if stream.is_null() {
        validate_fail(EINVAL);
        return EOF;
    }
    with_stream_locked(stream, || {
        let mut value: c_int = 0;
        let bytes = addr_of_mut!(value).cast::<u8>();
        for i in 0..core::mem::size_of::<c_int>() {
            *bytes.add(i) = _getc_nolock(stream) as u8;
        }

        if feof(stream) != 0 {
            return EOF;
        }
        if ferror(stream) != 0 {
            return EOF;
        }
        value
    })
}

#[no_mangle]
pub unsafe extern "C" fn _putw(value: c_int, stream: *mut c_void) -> c_int {
    if stream.is_null() {
        validate_fail(EINVAL);
        return EOF;
    }
    with_stream_locked(stream, || {
        let bytes = addr_of!(value).cast::<u8>();
        for i in 0..core::mem::size_of::<c_int>() {
            _fputc_nolock(*bytes.add(i) as c_int, stream);
        }

        if ferror(stream) != 0 {
            return EOF;
        }
        value
    })
}

// ── ungetc.cpp ───────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn ungetc(c: c_int, stream: *mut c_void) -> c_int {
    if stream.is_null() {
        validate_fail(EINVAL);
        return EOF;
    }
    with_stream_locked(stream, || _ungetc_nolock(c, stream))
}

#[no_mangle]
pub unsafe extern "C" fn _ungetc_nolock(c: c_int, public_stream: *mut c_void) -> c_int {
    let stream = Stream::from_file(public_stream);
    if !validate_stream_ansi_narrow(stream) {
        return EOF;
    }

    if c == EOF {
        return EOF;
    }

    let is_read_only = stream.has_all_of(crate::stdio_layout::flags::STREAM_READ);
    let is_rw_write =
        stream.has_all_of(crate::stdio_layout::flags::STREAM_UPDATE | crate::stdio_layout::flags::STREAM_WRITE);
    if !is_read_only && !is_rw_write {
        return EOF;
    }

    let d = stream.data();
    if (*d).base.is_null() {
        __acrt_stdio_allocate_buffer_nolock(public_stream);
    }

    if (*d).ptr == (*d).base {
        if (*d).cnt != 0 {
            return EOF;
        }
        (*d).ptr = (*d).ptr.add(1);
    }

    if stream.is_string_backed() {
        (*d).ptr = (*d).ptr.sub(1);
        if *(*d).ptr != c as c_char {
            (*d).ptr = (*d).ptr.add(1);
            return EOF;
        }
    } else {
        (*d).ptr = (*d).ptr.sub(1);
        *(*d).ptr = c as c_char;
    }

    (*d).cnt += 1;
    stream.unset_flags(crate::stdio_layout::flags::STREAM_EOF);
    stream.set_flags(crate::stdio_layout::flags::STREAM_READ);
    (c as u8) as c_int
}

// ── ungetwc.cpp ──────────────────────────────────────────────────────────────

extern "C" {
    fn wctomb_s(size_converted: *mut c_int, mbch: *mut u8, size_in_bytes: usize, wch: u16) -> c_int;
}

unsafe fn ungetwc_text_mode_nolock(c: u16, stream: Stream) -> u16 {
    let fh = super::_fileno(stream.public_stream());
    let text_mode = textmode_safe(fh);

    let mut count: c_int = 0;
    let mut characters = [0u8; MB_LEN_MAX];

    if text_mode == TEXT_MODE_ANSI {
        if wctomb_s(&mut count, characters.as_mut_ptr(), MB_LEN_MAX, c) != 0 {
            return WEOF;
        }
    } else {
        let bytes = addr_of!(c).cast::<u8>();
        characters[0] = *bytes;
        characters[1] = *bytes.add(1);
        count = 2;
    }

    let d = stream.data();
    if (*d).ptr < (*d).base.add(count as usize) {
        if (*d).cnt != 0 {
            return WEOF;
        }
        if count > (*d).bufsiz {
            return WEOF;
        }
        (*d).ptr = (*d).base.add(count as usize);
    }

    for i in (0..count).rev() {
        (*d).ptr = (*d).ptr.sub(1);
        *(*d).ptr = characters[i as usize] as c_char;
    }

    (*d).cnt += count;
    stream.unset_flags(crate::stdio_layout::flags::STREAM_EOF);
    stream.set_flags(crate::stdio_layout::flags::STREAM_READ);
    c
}

unsafe fn ungetwc_binary_mode_nolock(c: u16, stream: Stream) -> u16 {
    let d = stream.data();
    let width = core::mem::size_of::<u16>();

    if (*d).ptr < (*d).base.add(width) {
        if (*d).cnt != 0 {
            return WEOF;
        }
        if width as c_int > (*d).bufsiz {
            return WEOF;
        }
        (*d).ptr = (*d).base.add(width);
    }

    if stream.is_string_backed() {
        (*d).ptr = (*d).ptr.sub(width);
        if ((*d).ptr as *const u16).read_unaligned() != c {
            (*d).ptr = (*d).ptr.add(width);
            return WEOF;
        }
    } else {
        (*d).ptr = (*d).ptr.sub(width);
        ((*d).ptr as *mut u16).write_unaligned(c);
    }

    (*d).cnt += width as c_int;
    stream.unset_flags(crate::stdio_layout::flags::STREAM_EOF);
    stream.set_flags(crate::stdio_layout::flags::STREAM_READ);
    c
}

#[no_mangle]
pub unsafe extern "C" fn ungetwc(c: u16, stream: *mut c_void) -> u16 {
    if stream.is_null() {
        validate_fail(EINVAL);
        return WEOF;
    }
    with_stream_locked(stream, || _ungetwc_nolock(c, stream))
}

#[no_mangle]
pub unsafe extern "C" fn _ungetwc_nolock(c: u16, public_stream: *mut c_void) -> u16 {
    let stream = Stream::from_file(public_stream);

    if c == WEOF {
        return WEOF;
    }

    let is_read = stream.has_all_of(crate::stdio_layout::flags::STREAM_READ);
    let is_update = stream.has_all_of(crate::stdio_layout::flags::STREAM_UPDATE);
    let is_write = stream.has_all_of(crate::stdio_layout::flags::STREAM_WRITE);
    if !is_read && !(is_update && !is_write) {
        return WEOF;
    }

    if (*stream.data()).base.is_null() {
        __acrt_stdio_allocate_buffer_nolock(public_stream);
    }

    let fh = super::_fileno(public_stream);
    if !stream.is_string_backed() && (osfile_safe(fh) & FTEXT) != 0 {
        return ungetwc_text_mode_nolock(c, stream);
    }
    ungetwc_binary_mode_nolock(c, stream)
}
