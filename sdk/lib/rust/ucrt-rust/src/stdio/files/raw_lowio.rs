//! Lowio accessors and externs this piece needs that `stdio_layout.rs`
//! doesn't already expose (`_startpos`, unchecked `_textmode`,
//! `_utf8translations`, and a checked, writable `_osfile_safe`), plus the
//! PTD-propagating lowio entry points `fseek.cpp`/`ftell.cpp`/`fflush.cpp`/
//! `fclose.cpp`/`fwrite.cpp` call into (still C; lowio is a different batch).
//!
//! The offsets below are NOT independently re-measured: they are derived by
//! arithmetic from the ones `stdio_layout.rs` already measured and pinned
//! (`startpos` sits immediately before the 8-bit-aligned `osfile`; `textmode`
//! immediately follows `osfile`; the `unicode`/`utf8translations` bitfield
//! byte follows the 3-byte `_pipe_lookahead` that follows `textmode`) --
//! see `corecrt_internal_lowio.h`'s `__crt_lowio_handle_data` for the field
//! order this depends on. Same redeclare-locally convention as
//! `lowio_open.rs`'s own `handle` module (its doc comment explains why: that
//! module's offsets are private to it).

use core::ffi::{c_int, c_long, c_void};

use crate::ptd_host::PtdHost;

const IOINFO_L2E: c_int = 6;
const IOINFO_ARRAY_ELTS: c_int = 1 << IOINFO_L2E;
const WIDE: bool = cfg!(target_pointer_width = "64");

/// Same values as `stdio_layout::handle` / `lowio_open.rs::handle`.
const HANDLE_DATA_SIZE: usize = if WIDE { 72 } else { 56 };
const HANDLE_OSFILE_OFFSET: usize = if WIDE { 56 } else { 40 };
const HANDLE_TEXTMODE_OFFSET: usize = HANDLE_OSFILE_OFFSET + 1;
const HANDLE_STARTPOS_OFFSET: usize = HANDLE_OSFILE_OFFSET - 8;
/// The byte holding `unicode:1, utf8translations:1, dbcsBufferUsed:1`, right
/// after the 3-byte `_pipe_lookahead` that follows `textmode`.
const HANDLE_BITFIELD_BYTE: usize = HANDLE_TEXTMODE_OFFSET + 1 + 3;

/// `FEOFLAG` (corecrt_internal_lowio.h).
pub(super) const FEOFLAG: u8 = 0x02;
/// `FTEXT`.
pub(super) const FTEXT: u8 = 0x80;
/// `FCRLF`.
pub(super) const FCRLF: u8 = 0x04;

/// `__crt_lowio_text_mode`: ansi=0 (already `crate::stdio_layout::TEXT_MODE_ANSI`), utf8=1, utf16le=2.
pub(super) const TEXT_MODE_UTF8: u8 = 1;
pub(super) const TEXT_MODE_UTF16LE: u8 = 2;

extern "C" {
    /// `__crt_lowio_handle_data* __pioinfo[IOINFO_ARRAYS]` (lowio/ioinit.cpp).
    /// Redeclared here because `stdio_layout.rs`'s copy is private to it.
    static __pioinfo: [*mut u8; 128];
    /// The record `_pioinfo_safe` substitutes for handles -1 and -2.
    static __badioinfo: u8;

    fn _read_nolock(fh: c_int, buf: *mut c_void, max_char_count: u32) -> c_int;
    fn _write_internal(fh: c_int, buf: *const c_void, max_char_count: u32, ptd: *mut PtdHost) -> c_int;
    fn _lseeki64_nolock(fh: c_int, offset: i64, origin: c_int) -> i64;
    fn _lseeki64_nolock_internal(fh: c_int, offset: i64, origin: c_int, ptd: *mut PtdHost) -> i64;
    fn _lseeki64_internal(fh: c_int, offset: i64, origin: c_int, ptd: *mut PtdHost) -> i64;
    fn _lseek_internal(fh: c_int, offset: c_long, origin: c_int, ptd: *mut PtdHost) -> c_long;
    fn _close_internal(fh: c_int, ptd: *mut PtdHost) -> c_int;

    /// `char _lookuptrailbytes[256]` (lowio/utf8.cpp -- not this batch): the
    /// table `_utf8_no_of_trailbytes` is a thin inline over.
    static _lookuptrailbytes: [i8; 256];
}

// WINAPI: stdcall on i386 (`_ReadFile@20`), so never in an `extern "C"` block.
extern "system" {
    fn ReadFile(handle: isize, buffer: *mut c_void, bytes_to_read: u32, bytes_read: *mut u32, overlapped: *mut c_void) -> i32;
}

pub(super) unsafe fn read_nolock(fh: c_int, buf: *mut c_void, max_char_count: u32) -> c_int {
    _read_nolock(fh, buf, max_char_count)
}
pub(super) unsafe fn write_internal(fh: c_int, buf: *const c_void, max_char_count: u32, ptd: &mut PtdHost) -> c_int {
    _write_internal(fh, buf, max_char_count, ptd)
}
pub(super) unsafe fn lseeki64_nolock(fh: c_int, offset: i64, origin: c_int) -> i64 {
    _lseeki64_nolock(fh, offset, origin)
}
pub(super) unsafe fn lseeki64_nolock_internal(fh: c_int, offset: i64, origin: c_int, ptd: &mut PtdHost) -> i64 {
    _lseeki64_nolock_internal(fh, offset, origin, ptd)
}
pub(super) unsafe fn lseeki64_internal(fh: c_int, offset: i64, origin: c_int, ptd: &mut PtdHost) -> i64 {
    _lseeki64_internal(fh, offset, origin, ptd)
}
pub(super) unsafe fn lseek_internal(fh: c_int, offset: c_long, origin: c_int, ptd: &mut PtdHost) -> c_long {
    _lseek_internal(fh, offset, origin, ptd)
}
pub(super) unsafe fn close_internal(fh: c_int, ptd: &mut PtdHost) -> c_int {
    _close_internal(fh, ptd)
}

/// `ReadFile` (Win32), read exactly as `common_ftell_translated_utf8_nolock`
/// calls it: `nullptr` overlapped, ignoring partial-read/`FALSE` distinction
/// beyond a single bool.
pub(super) unsafe fn read_file(handle: isize, buffer: *mut u8, count: u32, bytes_read: &mut u32) -> bool {
    ReadFile(handle, buffer.cast(), count, bytes_read, core::ptr::null_mut()) != 0
}

/// `_utf8_no_of_trailbytes(c)`.
pub(super) unsafe fn utf8_no_of_trailbytes(c: u8) -> i8 {
    _lookuptrailbytes[c as usize]
}

unsafe fn pioinfo_mut(fh: c_int) -> *mut u8 {
    let array = __pioinfo[(fh >> IOINFO_L2E) as usize];
    array.add((fh & (IOINFO_ARRAY_ELTS - 1)) as usize * HANDLE_DATA_SIZE)
}

/// `_pioinfo_safe(fh)`: the handle record, or `__badioinfo` for -1 and -2.
unsafe fn pioinfo_safe(fh: c_int) -> *mut u8 {
    if fh == -1 || fh == -2 {
        return core::ptr::addr_of!(__badioinfo).cast_mut();
    }
    pioinfo_mut(fh)
}

/// `_startpos(fh)`: the UNCHECKED accessor, for a descriptor the caller has
/// already range-checked (as `stdio_layout::lowio::osfile`/`osfhnd` are).
pub(super) unsafe fn startpos(fh: c_int) -> i64 {
    (pioinfo_mut(fh).add(HANDLE_STARTPOS_OFFSET) as *const i64).read()
}

/// `_textmode(fh)`: the UNCHECKED accessor.
pub(super) unsafe fn textmode(fh: c_int) -> u8 {
    *pioinfo_mut(fh).add(HANDLE_TEXTMODE_OFFSET)
}

/// `_utf8translations(fh)`: bit 1 of the bitfield byte (bit 0 is `unicode`,
/// read by `stdio_layout::tm_unicode_safe`).
pub(super) unsafe fn utf8translations(fh: c_int) -> bool {
    *pioinfo_mut(fh).add(HANDLE_BITFIELD_BYTE) & 0b10 != 0
}

/// `_osfile_safe(fh) &= ~bits`.
pub(super) unsafe fn osfile_safe_clear(fh: c_int, bits: u8) {
    let p = pioinfo_safe(fh).add(HANDLE_OSFILE_OFFSET);
    *p &= !bits;
}

/// `_osfile(fh)`, re-exported from `stdio_layout` for callers in this crate
/// that only have `raw_lowio` imported.
pub(super) use crate::stdio_layout::lowio::osfile as osfile_unchecked;
