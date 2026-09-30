//! `stdio/{_getbuf,_freebuf,_sftbuf,_filbuf,_flsbuf,fgetc,fgetwc,fputc,fputwc,
//! getw,putw,ungetc,ungetwc,gets,fgets,puts,fputs,putws,fputws,tmpfile,tempnam,
//! rmtmp,gettemppath}.cpp`, ported.
//!
//! This is one of three pieces over the stdio stream core (`super`); see its
//! module doc for the shared design. This piece owns the buffer lifecycle
//! (allocate/free/temporary-buffer), the refill/flush engines every character
//! function bottoms out on, and everything built directly on a single
//! character: `fgetc`/`fgetwc`/`fputc`/`fputwc` and their `_nolock` twins,
//! `getw`/`putw`, `ungetc`/`ungetwc`, the line functions `gets`/`fgets`/
//! `puts`/`fputs` and their wide twins, and the temporary-file family
//! (`tmpfile`, `tmpnam`, `_tempnam`, `_rmtmp`, `__acrt_GetTempPath2A`).
//!
//! Cross-piece calls (declared `extern "C"` here from the header prototype,
//! transcribed rather than guessed -- see AGENTS.md and the batch brief):
//! * `_fwrite_nolock_internal`, `feof`, `ferror`, `_fclose_nolock`,
//!   `__acrt_stdio_flush_nolock`, `_commode` -- owned by the `files` piece.
//! * `__acrt_GetTempPath2W`, `_sopen_s`, `_wctomb_internal`, `mbtowc`,
//!   `wctomb_s`, `isleadbyte`, `_write_internal`, `_get_osfhandle`,
//!   `_lseeki64`, `SetFilePointerEx`, `GetFileSizeEx`, `GetCurrentProcessId`,
//!   `__acrt_WideCharToMultiByte`, `__acrt_AreFileApisANSI` -- still C
//!   elsewhere in the tree (lowio, mbstring, string, winapi thunks).
//!
//! Details that are silent if wrong:
//! * `_osfile_safe`/`_textmode_safe` distinguish text vs. binary/Unicode
//!   streams throughout: `_filbuf`'s Ctrl-Z detection, `_flsbuf`'s FAPPEND
//!   re-seek, and `fgetwc`/`fputwc`/`ungetwc`'s three encodings (ANSI
//!   multibyte, UTF-16LE/UTF-8 "binary" widen, or true binary) all key off it.
//!   `stdio_layout.rs` exports the checked `textmode_safe`/`tm_unicode_safe`
//!   accessors but not `_osfile_safe`'s byte, so [`osfile_safe`] below
//!   duplicates its `_pioinfo_safe` lookup locally (same offsets, same
//!   `__badioinfo` fallback for fh -1/-2) -- reported to the lead as a gap
//!   worth closing in the shared layout module.
//! * **Microsoft quirk, preserved**: in `_flsbuf`'s `write_buffer_nolock`,
//!   the FAPPEND re-seek failure path `return stdio_traits::eof;` returns the
//!   EOF/WEOF *sentinel value* (-1 or 0xFFFF) from a function that returns
//!   `bool` -- a nonzero int converts to `true`. So a failed re-seek sets
//!   `_IOERROR` but is reported as a *successful* character write by
//!   `common_flush_and_write_nolock`, which sees `true` and returns the
//!   character rather than EOF. See [`flush::write_buffer_nolock_narrow`].
//! * `fgetwc`'s Unicode-text-mode path reads two bytes one at a time through
//!   `_getc_nolock` (not `_fgetc_nolock`) so that a `_filbuf` refill mid
//!   character is transparent; `_filbuf`'s wide refill separately handles a
//!   UTF-16 code unit split across the OLD/NEW buffer boundary (the
//!   "leftover low-order byte" context).
//! * `ungetc`/`ungetwc` can push back only ONE character: if `_cnt` is
//!   already nonzero when `_ptr == _base`, there is no room and both fail.
//! * `tmpfile`'s six-character buffer-id space (`s`/`v` tmpnam, `t`/`w`
//!   tmpfile, `u`/`x` tmpnam_s -- narrow/wide) and `_tempnam`'s `_tempoff`
//!   counter (reset to 1 when the prefix grows, to avoid an infinite loop)
//!   are both faithfully reproduced, including that `_tempoff` and `_tmpoff`
//!   (tmpnam's per-buffer trailing digits, not a global) are DIFFERENT
//!   counters despite the similar names -- the C's own comment about this.

use core::ffi::{c_int, c_void};
use core::ptr::{addr_of, addr_of_mut};

use crate::ptd_host::PtdHost;
use crate::stdio::Stream;
use crate::stdio_layout::{textmode_safe, tm_unicode_safe};

mod buf;
mod lineio;
mod refill_flush;
mod tempnam;
mod tmp;
mod chario;

pub use buf::{__acrt_should_use_temporary_buffer, __acrt_stdio_begin_temporary_buffering_nolock};

// ── shared constants ─────────────────────────────────────────────────────────

pub(super) const EOF: c_int = -1;
pub(super) const WEOF: u16 = 0xFFFF;
const EINVAL: c_int = 22;
pub(super) const MB_LEN_MAX: usize = 5;

/// `_INTERNAL_BUFSIZ` / `_SMALL_BUFSIZ`, re-exported under the C names used in
/// the ported bodies.
pub(super) use crate::stdio::{INTERNAL_BUFSIZ, SMALL_BUFSIZ};

/// `__crt_lowio_text_mode` (corecrt_internal_lowio.h): ansi=0 (already in
/// `stdio_layout::TEXT_MODE_ANSI`), utf8=1, utf16le=2.
pub(super) const TEXT_MODE_UTF8: u8 = 1;
pub(super) const TEXT_MODE_UTF16LE: u8 = 2;

/// lowio `_osfile` bits this piece reads (corecrt_internal_lowio.h).
pub(super) const FEOFLAG: u8 = 0x02;
pub(super) const FAPPEND: u8 = 0x20;
pub(super) const FTEXT: u8 = 0x80;

extern "C" {
    pub(super) fn _errno() -> *mut c_int;
    pub(super) fn _invalid_parameter_noinfo();
    pub(super) fn _fileno(stream: *mut c_void) -> c_int;
    pub(super) fn calloc(count: usize, size: usize) -> *mut c_void;
    pub(super) fn malloc(size: usize) -> *mut c_void;
    pub(super) fn free(p: *mut c_void);

    // Owned by the `files` piece (fwrite.cpp, clearerr.cpp/fileno.cpp,
    // fclose.cpp, fflush.cpp, ncommode.cpp); declared here from the header so
    // this piece builds and grades independently, per the batch brief.
    pub(super) fn _fwrite_nolock_internal(
        buffer: *const c_void,
        element_size: usize,
        element_count: usize,
        stream: *mut c_void,
        ptd: *mut PtdHost,
    ) -> usize;
    pub(super) fn feof(stream: *mut c_void) -> c_int;
    pub(super) fn ferror(stream: *mut c_void) -> c_int;
    pub(super) fn _fclose_nolock(stream: *mut c_void) -> c_int;
    pub(super) fn __acrt_stdio_flush_nolock(stream: *mut c_void, ptd: *mut PtdHost) -> c_int;
    pub(super) static _commode: c_int;
}

// `__acrt_stdio_allocate_buffer_nolock`/`__acrt_stdio_free_buffer_nolock`
// (this piece's own `_getbuf.cpp`/`_freebuf.cpp`), used across submodules.
extern "C" {
    pub(super) fn __acrt_stdio_allocate_buffer_nolock(stream: *mut c_void);
    pub(super) fn __acrt_stdio_free_buffer_nolock(stream: *mut c_void);
    pub(super) fn __acrt_stdio_refill_and_read_narrow_nolock(stream: *mut c_void) -> c_int;
    pub(super) fn __acrt_stdio_refill_and_read_wide_nolock(stream: *mut c_void) -> c_int;
    pub(super) fn __acrt_stdio_flush_and_write_narrow_nolock(
        c: c_int,
        stream: *mut c_void,
        ptd: *mut PtdHost,
    ) -> c_int;
    pub(super) fn __acrt_stdio_flush_and_write_wide_nolock(
        c: c_int,
        stream: *mut c_void,
        ptd: *mut PtdHost,
    ) -> c_int;
}

/// `stdin`/`stdout`/`stderr`: `&_iob[0..3]`, the same identity
/// `__acrt_iob_func` hands out.
pub(super) unsafe fn stdin_ptr() -> *mut c_void {
    addr_of_mut!(crate::stdio::_iob[0]).cast()
}
pub(super) unsafe fn stdout_ptr() -> *mut c_void {
    addr_of_mut!(crate::stdio::_iob[1]).cast()
}
pub(super) unsafe fn stderr_ptr() -> *mut c_void {
    addr_of_mut!(crate::stdio::_iob[2]).cast()
}

/// `_VALIDATE_RETURN`'s failure arm (non-PTD forms): set errno, invoke the
/// handler. The caller still supplies its own `return`.
#[cold]
pub(super) unsafe fn validate_fail(err: c_int) {
    *_errno() = err;
    _invalid_parameter_noinfo();
}

/// `_VALIDATE_STREAM_ANSI_RETURN(stream, EINVAL, ret)`'s condition, without
/// the ptd: true if the stream is string-backed, or is a real file open in
/// ANSI text mode (not `ccs=...` Unicode). Sets errno/invokes the handler and
/// returns false on failure; the caller returns its own EOF/nullptr.
pub(super) unsafe fn validate_stream_ansi_narrow(stream: Stream) -> bool {
    if stream.is_string_backed() {
        return true;
    }
    let fh = _fileno(stream.public_stream());
    if textmode_safe(fh) == crate::stdio_layout::TEXT_MODE_ANSI && !tm_unicode_safe(fh) {
        return true;
    }
    validate_fail(EINVAL);
    false
}

/// The PTD-propagating twin, `_UCRT_VALIDATE_STREAM_ANSI_RETURN`.
pub(super) unsafe fn validate_stream_ansi_narrow_ptd(stream: Stream, ptd: *mut PtdHost) -> bool {
    if stream.is_string_backed() {
        return true;
    }
    let fh = _fileno(stream.public_stream());
    if textmode_safe(fh) == crate::stdio_layout::TEXT_MODE_ANSI && !tm_unicode_safe(fh) {
        return true;
    }
    (*ptd).invalid_parameter(EINVAL);
    false
}

// ── `_osfile_safe` ───────────────────────────────────────────────────────────
//
// `stdio_layout.rs` exports the checked text-mode/unicode accessors but not
// this byte, and its `__pioinfo`/`__badioinfo` statics and offset table are
// private to that module. Duplicated here (same measured offsets, same
// `__badioinfo` fallback for the two negative pseudo-handles) rather than
// widening that module's public surface without the lead's say -- see the
// module doc.

const IOINFO_L2E: c_int = 6;
const IOINFO_ARRAY_ELTS: c_int = 1 << IOINFO_L2E;
const HANDLE_DATA_SIZE: usize = if cfg!(target_pointer_width = "64") { 72 } else { 56 };
/// One byte before `_textmode` (asserted equal in `stdio_layout::lowio`).
const HANDLE_OSFILE_OFFSET: usize = if cfg!(target_pointer_width = "64") { 56 } else { 40 };

extern "C" {
    static __pioinfo: [*mut u8; 128];
    static __badioinfo: u8;
}

/// `_osfile_safe(fh)`: like `_textmode_safe`/`_tm_unicode_safe`, falls back to
/// `__badioinfo` for the two negative pseudo-handles (-1, -2) instead of
/// indexing `__pioinfo` with a negative shift.
pub(super) unsafe fn osfile_safe(fh: c_int) -> u8 {
    let record = if fh == -1 || fh == -2 {
        addr_of!(__badioinfo)
    } else {
        let array = __pioinfo[(fh >> IOINFO_L2E) as usize];
        array.add((fh & (IOINFO_ARRAY_ELTS - 1)) as usize * HANDLE_DATA_SIZE)
    };
    *record.add(HANDLE_OSFILE_OFFSET)
}

pub use chario::{
    _fgetc_nolock, _fgetwc_nolock, _fputc_nolock, _fputc_nolock_internal, _fputwc_nolock,
    _fputwc_nolock_internal, _getc_nolock, _getw, _getwc_nolock, _putc_nolock, _putw,
    _putwc_nolock, _ungetc_nolock, _ungetwc_nolock, fgetc, fgetwc, fputc, fputwc, getc, getchar,
    putc, putchar, putwc, ungetc, ungetwc, _fgetchar, _fgetwchar, _fputchar,
    _fputwchar, getwc, getwchar, putwchar,
};
pub use lineio::{_getws, _getws_s, _putws, fgets, fgetws, fputs, fputws, gets, gets_s, puts};
pub use tempnam::{_tempnam, _wtempnam};
pub use tmp::{
    __acrt_force_use_of_tmpfile, __acrt_stdio_free_tmpfile_name_buffers_nolock,
    __acrt_uninitialize_tmpfile, _rmtmp, _tempoff, _old_pfxlen, _wtmpnam, _wtmpnam_s, tmpfile,
    tmpfile_s, tmpnam, tmpnam_s, __acrt_GetTempPath2A,
};
