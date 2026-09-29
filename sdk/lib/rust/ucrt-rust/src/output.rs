//! stdio/output.cpp and the processor behind it (`corecrt_internal_stdio_output.h`):
//! the fourteen `__stdio_common_v*printf*` entry points every UCRT printf is a
//! thin inline over.
//!
//! ## A port, not msvcrt's engine
//!
//! msvcrt-crt-rust has a complete printf engine, and it even has a `_MSVCR_VER
//! >= 140` arm. It is still the wrong body for ucrtbase, because it is Wine's
//! parser and UCRT's is a different machine: a table-driven state machine with
//! TWO transition tables, the standard one and a stricter one for the `_s` and
//! `_p` functions. They disagree on every malformed format -- `%q` prints `q`
//! under the first and is an invalid parameter under the second, and `%n` is a
//! conversion under the first and not even a type character under the second
//! -- and Wine's parser matches neither. So this is Microsoft's processor,
//! ported structure for structure; see ctype.rs and strtox.rs for the same
//! call on the same grounds.
//!
//! ## The three "bases"
//!
//! The C instantiates one class template over three base classes that hide each
//! other's members: `standard_base` (plain printf), `format_validation_base`
//! (the `_s` functions: stricter table, a check at the end of the format) and
//! `positional_parameter_base` (the `_p` functions: `%1$d`, two passes, and
//! the validation of the other two). Here that is [`Base`], a runtime value
//! consulted at each of the points the C overrides.
//!
//! ## What is still C
//!
//! Floating-point conversion (`__acrt_fp_format`, convert/cvt.cpp), the
//! multibyte conversions (`_wctomb_internal`, `_mbtowc_internal`), the stream
//! writes (`_fputc_nolock_internal`, `_fputwc_nolock_internal`) and the
//! temporary-buffering hooks are called as they are. Each takes the call's
//! `__crt_cached_ptd_host`, which is why [`PtdHost`] is layout-identical to it:
//! errno set inside those calls lands in the same cache this processor reads
//! back. They go when their own families do.

#![allow(clippy::too_many_arguments)]

use core::ffi::{c_int, c_void};

use crate::locale::{CrtLocaleData, CrtLocalePointers};
use crate::ptd_host::{Cached, PtdHost};
use crate::stdio_layout::{textmode_safe, tm_unicode_safe, StreamData, TEXT_MODE_ANSI};
use crate::strtox::{parse, Char};

const EINVAL: c_int = 22;
const ERANGE: c_int = 34;
const ENOMEM: c_int = 12;
const EILSEQ: c_int = 42;

const EOF: c_int = -1;
const WEOF: u16 = 0xFFFF;

/// `_CVTBUFSIZE` (corecrt_internal_fltintrn.h): 309 + 40.
const CVTBUFSIZE: usize = 349;
/// `_ARGMAX`: positional parameters are numbered 1..=100.
const ARGMAX: i32 = 100;
const MB_LEN_MAX: usize = 5;
const INT_MAX: i32 = i32::MAX;
const TRUNCATE: usize = usize::MAX;

/// `_LEADBYTE` and `_DIGIT` in the ctype table.
const LEADBYTE: u16 = 0x8000;
const DIGIT: u16 = 0x4;

// corecrt_stdio_config.h -- the option bits the processor itself consults.
const LEGACY_VSPRINTF_NULL_TERMINATION: u64 = 0x0001;
const STANDARD_SNPRINTF_BEHAVIOR: u64 = 0x0002;
const LEGACY_WIDE_SPECIFIERS: u64 = 0x0004;
const LEGACY_MSVCRT_COMPATIBILITY: u64 = 0x0008;

/// `__acrt_rounding_mode::standard`.
const ROUNDING_STANDARD: c_int = 1;

extern "C" {
    fn __acrt_fp_format(
        value: *const u64,
        result_buffer: *mut u8,
        result_buffer_count: usize,
        scratch_buffer: *mut u8,
        scratch_buffer_count: usize,
        format: c_int,
        precision: c_int,
        options: u64,
        rounding_mode: c_int,
        ptd: *mut PtdHost,
    ) -> c_int;
    fn _wctomb_internal(
        size_converted: *mut c_int,
        mbch: *mut u8,
        size_in_bytes: usize,
        wch: u16,
        ptd: *mut PtdHost,
    ) -> c_int;
    fn _mbtowc_internal(dst: *mut u16, src: *const u8, src_size: usize, ptd: *mut PtdHost) -> c_int;
    fn _fputc_nolock_internal(c: c_int, stream: *mut c_void, ptd: *mut PtdHost) -> c_int;
    fn _fputwc_nolock_internal(c: u16, stream: *mut c_void, ptd: *mut PtdHost) -> u16;
    fn __acrt_stdio_begin_temporary_buffering_nolock(stream: *mut c_void) -> bool;
    fn __acrt_stdio_end_temporary_buffering_nolock(flag: bool, stream: *mut c_void, ptd: *mut PtdHost);
    fn _get_printf_count_output() -> c_int;
    fn _lock_file(stream: *mut c_void);
    fn _unlock_file(stream: *mut c_void);
    fn _fileno(stream: *mut c_void) -> c_int;
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    fn _malloc_base(size: usize) -> *mut c_void;
    fn _free_base(block: *mut c_void);
}

// ── characters ────────────────────────────────────────────────────────────────

/// The processor's `Character`: `char` or `wchar_t`.
pub(crate) trait OutChar: Char + 'static {
    const WIDE: bool;
    fn of(c: u32) -> Self;
}

impl OutChar for u8 {
    const WIDE: bool = false;
    fn of(c: u32) -> u8 {
        c as u8
    }
}

impl OutChar for u16 {
    const WIDE: bool = true;
    fn of(c: u32) -> u16 {
        c as u16
    }
}

// ── states and the transition tables ─────────────────────────────────────────

const ST_NORMAL: u8 = 0;
const ST_PERCENT: u8 = 1;
const ST_FLAG: u8 = 2;
const ST_WIDTH: u8 = 3;
const ST_DOT: u8 = 4;
const ST_PRECISION: u8 = 5;
const ST_SIZE: u8 = 6;
const ST_TYPE: u8 = 7;
const ST_INVALID: u8 = 8;

/// The character class of `' '..='z'`, the lower half of each table's entries.
/// Generated from the header. The two tables' class columns differ in exactly
/// one place: `'n'` is a type character in the standard table and an ordinary
/// one in the validation table, so `%n` is an invalid specifier in every `_s`
/// and `_p` function.
#[rustfmt::skip]
const STANDARD_CLASS: [u8; 91] = [
    6, 0, 0, 6, 0, 1, 0, 0, 0, 0, 3, 6, 0, 6, 2, 0,
    4, 5, 5, 5, 5, 5, 5, 5, 5, 5, 0, 0, 0, 0, 0, 0,
    0, 8, 0, 8, 0, 8, 7, 8, 0, 7, 0, 0, 7, 0, 7, 0,
    0, 0, 0, 8, 7, 0, 0, 0, 8, 0, 8, 0, 0, 0, 0, 0,
    0, 8, 0, 8, 8, 8, 8, 8, 7, 8, 7, 0, 7, 0, 8, 8,
    8, 0, 0, 8, 7, 8, 0, 7, 8, 0, 7,
];

/// `next_state = table[class * state_count + previous]`, standard table
/// (state_count 8: it has no `invalid` state).
#[rustfmt::skip]
const STANDARD_NEXT: [u8; 91] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1,
    0, 4, 4, 4, 0, 0, 0, 0, 0, 3, 3, 0, 5, 0, 0, 0,
    0, 2, 2, 3, 5, 5, 0, 0, 0, 3, 3, 3, 5, 5, 0, 0,
    0, 2, 2, 0, 0, 0, 0, 0, 0, 6, 6, 6, 6, 6, 6, 0,
    0, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// The validation table's next states (state_count 9).
#[rustfmt::skip]
const VALIDATION_NEXT: [u8; 91] = [
    0, 8, 8, 8, 8, 8, 8, 0, 0, 1, 0, 8, 8, 8, 8, 8,
    1, 0, 0, 4, 4, 4, 8, 8, 8, 0, 0, 0, 3, 3, 8, 5,
    8, 8, 0, 0, 0, 2, 2, 3, 5, 5, 8, 0, 0, 0, 3, 3,
    3, 5, 5, 8, 0, 0, 0, 2, 2, 8, 8, 8, 8, 0, 0, 0,
    6, 6, 6, 6, 6, 6, 0, 0, 0, 7, 7, 7, 7, 7, 7, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

const VALIDATION_CLASS_N: u8 = 0;

// ── flags and length modifiers ────────────────────────────────────────────────

const PF_FL_SIGN: u32 = 0x01;
const PF_FL_SIGNSP: u32 = 0x02;
const PF_FL_LEFT: u32 = 0x04;
const PF_FL_LEADZERO: u32 = 0x08;
const PF_FL_SIGNED: u32 = 0x10;
const PF_FL_ALTERNATE: u32 = 0x20;
const PF_FL_NEGATIVE: u32 = 0x40;
const PF_FL_FORCEOCTAL: u32 = 0x80;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Len {
    None,
    Hh,
    H,
    L,
    Ll,
    J,
    Z,
    T,
    BigL,
    I,
    I32,
    I64,
    W,
    BigT,
}

/// `to_integer_size`. `L`, `w` and `T` have no integer size, so an integer
/// conversion carrying one is an invalid parameter.
fn to_integer_size(length: Len) -> usize {
    match length {
        Len::None => 4,
        Len::Hh => 1,
        Len::H => 2,
        Len::L => 4, // long is 32-bit on Windows
        Len::Ll => 8,
        Len::J => 8,
        Len::Z | Len::T | Len::I => core::mem::size_of::<usize>(),
        Len::I32 => 4,
        Len::I64 => 8,
        Len::BigL | Len::W | Len::BigT => 0,
    }
}

/// `is_wide_character_specifier`: does this `%c`/`%s`/`%C`/`%S`/`%Z` read a
/// wide argument?
fn is_wide_character_specifier<C: OutChar>(options: u64, format_type: u32, length: Len) -> bool {
    match length {
        Len::L | Len::W => return true,
        Len::H => return false,
        Len::BigT => return C::WIDE,
        _ => {}
    }
    let is_naturally_wide = C::WIDE && options & LEGACY_WIDE_SPECIFIERS != 0;
    let is_natural_width = format_type == b'c' as u32 || format_type == b's' as u32;
    is_naturally_wide == is_natural_width
}

// ── the varargs cursor ────────────────────────────────────────────────────────

/// A Windows `va_list`: a plain byte cursor on all three targets. Every
/// argument occupies a whole number of pointer-sized slots, and on i386 the
/// 8-byte types take two 4-byte slots with no alignment padding.
#[derive(Clone, Copy)]
struct Va(*mut u8);

impl Va {
    unsafe fn read<T: Copy>(&mut self) -> T {
        let value = (self.0 as *const T).read_unaligned();
        let slot = core::mem::size_of::<usize>();
        self.0 = self.0.add(core::mem::size_of::<T>().div_ceil(slot) * slot);
        value
    }
}

/// The argument type a conversion asks for, i.e. the `RequestedParameterType`
/// of `extract_argument_from_va_list`. The narrow ones are read as the `int`
/// the default promotions made them, then converted, as `read_va_arg` does.
#[derive(Clone, Copy)]
enum Req {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    Ptr,
    Double,
}

/// Reads one argument, returned as the `__int64` the C widens it into (the
/// double's bits, for `Double`).
unsafe fn read_arg(va: &mut Va, req: Req) -> u64 {
    match req {
        Req::I8 => va.read::<i32>() as i8 as i64 as u64,
        Req::U8 => va.read::<i32>() as u8 as u64,
        Req::I16 => va.read::<i32>() as i16 as i64 as u64,
        Req::U16 => va.read::<i32>() as u16 as u64,
        Req::I32 => va.read::<i32>() as i64 as u64,
        Req::U32 => va.read::<u32>() as u64,
        Req::I64 | Req::U64 | Req::Double => va.read::<u64>(),
        Req::Ptr => va.read::<usize>() as u64,
    }
}

// ── positional parameters ────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum ParamType {
    Unused,
    Int32,
    Int64,
    Pointer,
    Real64,
}

/// `get_parameter_type`: the promoted argument category.
fn param_type(req: Req) -> ParamType {
    match req {
        Req::Ptr => ParamType::Pointer,
        Req::I64 | Req::U64 => ParamType::Int64,
        Req::Double => ParamType::Real64,
        _ => ParamType::Int32,
    }
}

#[derive(Clone, Copy)]
struct Param {
    actual_type: ParamType,
    format_type: u32,
    va: Va,
    length: Len,
}

const UNUSED_PARAM: Param = Param {
    actual_type: ParamType::Unused,
    format_type: 0,
    va: Va(core::ptr::null_mut()),
    length: Len::None,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Base {
    Standard,
    Validation,
    Positional,
}

// positional passes; the other two bases use NOT_STARTED -> OUTPUT -> FINISHED
// of their own, which is the same walk with the scan pass skipped.
const NOT_STARTED: u8 = 0;
const POSITION_SCAN: u8 = 1;
const OUTPUT: u8 = 2;
const FINISHED: u8 = 3;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Unknown,
    Nonpositional,
    Positional,
}

// ── the formatting buffer ────────────────────────────────────────────────────

const MEMBER_BUFFER_SIZE: usize = 1024;

/// `formatting_buffer`: 1024 bytes inline, grown on the heap on demand. Each
/// half of whichever is in use is a separate region: the result, then the
/// scratch area `__acrt_fp_format` needs.
struct FormattingBuffer {
    member: [u64; MEMBER_BUFFER_SIZE / 8],
    dynamic: *mut u8,
    dynamic_size: usize,
}

impl FormattingBuffer {
    fn new() -> Self {
        Self { member: [0; MEMBER_BUFFER_SIZE / 8], dynamic: core::ptr::null_mut(), dynamic_size: 0 }
    }

    unsafe fn ensure<T>(&mut self, count: usize, host: &mut PtdHost) -> bool {
        let size = core::mem::size_of::<T>();
        let max_count = usize::MAX / size / 2;
        if max_count < count {
            // _UCRT_VALIDATE_RETURN_NOEXC: errno, no handler.
            host.errno().set(ENOMEM);
            return false;
        }
        let required = count * size * 2;
        if self.dynamic.is_null() && required <= MEMBER_BUFFER_SIZE {
            return true;
        }
        if required <= self.dynamic_size {
            return true;
        }
        let new_buffer = _malloc_base(required) as *mut u8;
        if new_buffer.is_null() {
            return false;
        }
        _free_base(self.dynamic as *mut c_void);
        self.dynamic = new_buffer;
        self.dynamic_size = required;
        true
    }

    fn data<T>(&mut self) -> *mut T {
        if self.dynamic.is_null() {
            self.member.as_mut_ptr() as *mut T
        } else {
            self.dynamic as *mut T
        }
    }

    fn count<T>(&self) -> usize {
        let total = if self.dynamic.is_null() { MEMBER_BUFFER_SIZE } else { self.dynamic_size };
        total / core::mem::size_of::<T>() / 2
    }

    fn scratch_data<T>(&mut self) -> *mut T {
        let count = self.count::<T>();
        unsafe { self.data::<T>().add(count) }
    }
}

impl Drop for FormattingBuffer {
    fn drop(&mut self) {
        unsafe { _free_base(self.dynamic as *mut c_void) };
    }
}

// ── locale helpers ────────────────────────────────────────────────────────────

unsafe fn locinfo(locale: *mut CrtLocalePointers) -> *mut CrtLocaleData {
    (*locale).locinfo
}

/// `__acrt_locale_get_ctype_array_value`: in range only for -1..=255. Callers
/// passing a plain `char` hand it a NEGATIVE value for every byte above 0x7F,
/// which is out of range and so reads 0 -- see `state_case_normal_tchar`.
unsafe fn ctype_value(locale: *mut CrtLocalePointers, c: c_int, mask: u16) -> u16 {
    if (-1..=255).contains(&c) {
        return *(*locinfo(locale)).public.locale_pctype.offset(c as isize) & mask;
    }
    0
}

unsafe fn decimal_point(locale: *mut CrtLocalePointers) -> u8 {
    **((*locinfo(locale)).lconv as *const *const u8)
}

unsafe fn mb_cur_max(locale: *mut CrtLocalePointers) -> usize {
    (*locinfo(locale)).public.locale_mb_cur_max as usize
}

/// `force_decimal_point`: `%#.0e` and friends. Inserts the locale's decimal
/// point after the leading digits (or after the hex digit of `%a`) in place.
unsafe fn force_decimal_point(mut buffer: *mut u8, locale: *mut CrtLocalePointers) {
    let pclmap = (*locinfo(locale)).pclmap;
    if *pclmap.add(*buffer as usize) != b'e' {
        loop {
            buffer = buffer.add(1);
            if ctype_value(locale, *buffer as c_int, DIGIT) == 0 {
                break;
            }
        }
    }
    if *pclmap.add(*buffer as usize) == b'x' {
        buffer = buffer.add(2);
    }
    let mut holdchar = *buffer;
    *buffer = decimal_point(locale);
    buffer = buffer.add(1);
    loop {
        let nextchar = *buffer;
        *buffer = holdchar;
        holdchar = nextchar;
        let was = *buffer;
        buffer = buffer.add(1);
        if was == 0 {
            break;
        }
    }
}

/// `crop_zeroes`: `%g`'s trailing-zero removal.
unsafe fn crop_zeroes(mut buffer: *mut u8, locale: *mut CrtLocalePointers) {
    let decimal_point = decimal_point(locale);
    while *buffer != 0 && *buffer != decimal_point {
        buffer = buffer.add(1);
    }
    let at = *buffer;
    buffer = buffer.add(1);
    if at != 0 {
        while *buffer != 0 && *buffer != b'e' && *buffer != b'E' {
            buffer = buffer.add(1);
        }
        let mut stop = buffer;
        buffer = buffer.sub(1);
        while *buffer == b'0' {
            buffer = buffer.sub(1);
        }
        if *buffer == decimal_point {
            buffer = buffer.sub(1);
        }
        loop {
            buffer = buffer.add(1);
            *buffer = *stop;
            stop = stop.add(1);
            if *buffer == 0 {
                break;
            }
        }
    }
}

// ── output adapters ───────────────────────────────────────────────────────────

pub(crate) trait Adapter<C: OutChar> {
    unsafe fn validate(&self, host: &mut PtdHost) -> bool;
    unsafe fn write_character(&self, c: C, count_written: &mut i32, host: &mut PtdHost);
    unsafe fn write_string(&self, string: *const C, length: i32, count_written: &mut i32, host: &mut PtdHost);
}

/// `write_multiple_characters`.
unsafe fn write_multiple<C: OutChar, A: Adapter<C>>(
    adapter: &A,
    c: C,
    count: i32,
    count_written: &mut i32,
    host: &mut PtdHost,
) {
    let mut i = 0;
    while i < count {
        adapter.write_character(c, count_written, host);
        if *count_written == -1 {
            break;
        }
        i += 1;
    }
}

/// `string_output_adapter_context`.
struct StringContext<C> {
    buffer: *mut C,
    buffer_count: usize,
    buffer_used: usize,
    continue_count: bool,
}

/// `string_output_adapter`: the sprintf family's sink. With `continue_count`
/// it keeps COUNTING past the end of the buffer (snprintf's "how long would it
/// have been"); without it, the first character that does not fit fails the
/// whole call.
struct StringAdapter<C>(*mut StringContext<C>);

impl<C: OutChar> Adapter<C> for StringAdapter<C> {
    unsafe fn validate(&self, host: &mut PtdHost) -> bool {
        if self.0.is_null() {
            host.invalid_parameter(EINVAL);
            return false;
        }
        true
    }

    unsafe fn write_character(&self, c: C, count_written: &mut i32, _host: &mut PtdHost) {
        let ctx = &mut *self.0;
        if ctx.buffer_used == ctx.buffer_count {
            if ctx.continue_count {
                *count_written += 1;
            } else {
                *count_written = -1;
            }
            return;
        }
        *count_written += 1;
        ctx.buffer_used += 1;
        *ctx.buffer = c;
        ctx.buffer = ctx.buffer.add(1);
    }

    unsafe fn write_string(&self, string: *const C, length: i32, count_written: &mut i32, _host: &mut PtdHost) {
        if length == 0 {
            return;
        }
        let ctx = &mut *self.0;
        if ctx.buffer_used == ctx.buffer_count {
            if ctx.continue_count {
                *count_written += length;
            } else {
                *count_written = -1;
            }
            return;
        }
        let space_available = ctx.buffer_count - ctx.buffer_used;
        let elements_to_copy = space_available.min(length as usize);
        core::ptr::copy_nonoverlapping(string, ctx.buffer, elements_to_copy);
        ctx.buffer = ctx.buffer.add(elements_to_copy);
        ctx.buffer_used += elements_to_copy;
        if ctx.continue_count {
            *count_written += length;
        } else if elements_to_copy != length as usize {
            *count_written = -1;
        } else {
            *count_written += elements_to_copy as i32;
        }
    }
}

/// `stream_output_adapter`: the fprintf family's sink, writing through the
/// stream's own buffer with the lock already held.
struct StreamAdapter(*mut c_void);

impl StreamAdapter {
    fn data(&self) -> *mut StreamData {
        self.0 as *mut StreamData
    }

    /// A string-backed stream with no buffer only counts (`_vsnprintf(NULL, 0)`
    /// reaches here that way from older CRT shims).
    unsafe fn counts_only(&self) -> bool {
        (*self.data()).is_string_backed() && (*self.data()).base.is_null()
    }
}

/// How each width writes one character to a stream.
pub(crate) trait StreamChar: OutChar {
    unsafe fn put(self, stream: *mut c_void, host: &mut PtdHost) -> bool;
    /// `validate_stream_is_ansi_if_required`: narrow output refuses a stream
    /// opened in a Unicode text mode.
    unsafe fn validate_stream(stream: *mut c_void) -> bool;
}

impl StreamChar for u8 {
    unsafe fn put(self, stream: *mut c_void, host: &mut PtdHost) -> bool {
        // The C passes its `char` to an `int` parameter, sign-extending it.
        _fputc_nolock_internal(self as i8 as c_int, stream, host) != EOF
    }
    unsafe fn validate_stream(stream: *mut c_void) -> bool {
        let data = stream as *mut StreamData;
        if (*data).is_string_backed() {
            return true;
        }
        let fh = _fileno(stream);
        if textmode_safe(fh) == TEXT_MODE_ANSI && !tm_unicode_safe(fh) {
            return true;
        }
        // _VALIDATE_STREAM_ANSI_RETURN is the OLD macro: errno directly, then
        // the handler -- not the host's cache.
        *_errno() = EINVAL;
        _invalid_parameter_noinfo();
        false
    }
}

impl StreamChar for u16 {
    unsafe fn put(self, stream: *mut c_void, host: &mut PtdHost) -> bool {
        // WEOF is 0xFFFF, so writing U+FFFF reads back as a failure. Kept: it
        // is what the C does, and U+FFFF is a noncharacter.
        _fputwc_nolock_internal(self, stream, host) != WEOF
    }
    unsafe fn validate_stream(_stream: *mut c_void) -> bool {
        true
    }
}

impl StreamAdapter {
    /// `write_character_without_count_update`.
    unsafe fn put<C: StreamChar>(&self, c: C, host: &mut PtdHost) -> bool {
        if self.counts_only() {
            return true;
        }
        c.put(self.0, host)
    }
}

impl<C: StreamChar> Adapter<C> for StreamAdapter {
    unsafe fn validate(&self, host: &mut PtdHost) -> bool {
        if self.0.is_null() {
            host.invalid_parameter(EINVAL);
            return false;
        }
        C::validate_stream(self.0)
    }

    /// `output_adapter_common::write_character`.
    unsafe fn write_character(&self, c: C, count_written: &mut i32, host: &mut PtdHost) {
        if self.put(c, host) {
            *count_written += 1;
        } else {
            *count_written = -1;
        }
    }

    unsafe fn write_string(&self, string: *const C, length: i32, count_written: &mut i32, host: &mut PtdHost) {
        if self.counts_only() {
            *count_written += length;
            return;
        }
        // `write_string_impl`, under its errno guard.
        let reset_errno: Cached<c_int> = *host.errno();
        let mut i = 0;
        while i < length {
            if self.put(*string.add(i as usize), host) {
                *count_written += 1;
            } else {
                // Non-standard extension: an EILSEQ is recoverable -- write a
                // '?' and carry on with the rest of the string.
                if !host.errno().check(EILSEQ) {
                    *count_written = -1;
                    break;
                }
                <Self as Adapter<C>>::write_character(self, C::of(b'?' as u32), count_written, host);
            }
            i += 1;
        }
        *host.errno() = reset_errno;
    }
}

// ── the processor ─────────────────────────────────────────────────────────────

pub(crate) struct Processor<'h, C: OutChar, A: Adapter<C>> {
    adapter: A,
    options: u64,
    host: &'h mut PtdHost,
    format_it: *const C,
    va: Va,
    characters_written: i32,

    state: u8,
    flags: u32,
    field_width: i32,
    precision: i32,
    length: Len,
    suppress_output: bool,
    format_char: C,

    /// `_narrow_string` / `_wide_string`: one union in the C, one pointer here.
    string: *mut u8,
    string_length: i32,
    string_is_wide: bool,

    buffer: FormattingBuffer,

    base: Base,
    current_pass: u8,
    format_mode: Mode,
    format: *const C,
    parameters: [Param; ARGMAX as usize],
    maximum_index: i32,
    type_index: i32,
}

impl<'h, C: OutChar, A: Adapter<C>> Processor<'h, C, A> {
    pub(crate) fn new(adapter: A, options: u64, format: *const C, host: &'h mut PtdHost, arglist: *mut u8, base: Base) -> Self {
        Self {
            adapter,
            options,
            host,
            format_it: format,
            va: Va(arglist),
            characters_written: 0,
            state: ST_NORMAL,
            flags: 0,
            field_width: 0,
            precision: 0,
            length: Len::None,
            suppress_output: false,
            format_char: C::of(0),
            string: core::ptr::null_mut(),
            string_length: 0,
            string_is_wide: false,
            buffer: FormattingBuffer::new(),
            base,
            current_pass: NOT_STARTED,
            format_mode: Mode::Unknown,
            format,
            parameters: [UNUSED_PARAM; ARGMAX as usize],
            maximum_index: -1,
            type_index: -1,
        }
    }

    fn has_flag(&self, f: u32) -> bool {
        self.flags & f != 0
    }
    fn set_flag(&mut self, f: u32) {
        self.flags |= f;
    }
    fn unset_flag(&mut self, f: u32) {
        self.flags &= !f;
    }

    fn fc(&self) -> u32 {
        self.format_char.code()
    }

    /// `_UCRT_VALIDATE_RETURN(ptd, cond, EINVAL, false)`.
    unsafe fn validate(&mut self, cond: bool) -> bool {
        if !cond {
            self.host.invalid_parameter(EINVAL);
        }
        cond
    }

    unsafe fn peek_format(&self, offset: usize) -> u32 {
        (*self.format_it.add(offset)).code()
    }

    /// `_tcstol_internal(ptd, p, &end, 10)`: `long` is 32 bits here, and it does
    /// NOT skip leading whitespace (its `TrimWhitespace` is false), so
    /// `%1$* 2$d` fails where a public strtol would read the 2. ERANGE is
    /// reported into the host's errno cache, where the C's lands.
    unsafe fn tcstol(&mut self, p: *const C, end: &mut *const C) -> i32 {
        let locale = self.host.get_locale();
        let errno = _errno();
        let saved = *errno;
        *errno = 0;
        let value = parse::<C>(p, end, 10, locale, 32, true, false) as u32 as i32;
        if *errno == ERANGE {
            self.host.errno().set(ERANGE);
        }
        *errno = saved;
        value
    }

    // ── the per-base hooks ───────────────────────────────────────────────────

    fn advance_to_next_pass(&mut self) -> bool {
        if self.base != Base::Positional {
            self.current_pass = if self.current_pass == NOT_STARTED { OUTPUT } else { FINISHED };
            return self.current_pass != FINISHED;
        }
        self.current_pass += 1;
        if self.current_pass == FINISHED {
            return false;
        }
        if self.current_pass == OUTPUT && self.format_mode == Mode::Nonpositional {
            return false;
        }
        self.format_mode = Mode::Unknown;
        self.maximum_index = -1;
        self.type_index = -1;
        self.field_width = 0;
        self.precision = 0;
        self.format_it = self.format;
        true
    }

    fn find_next_state(&self, c: C, previous: u8) -> u8 {
        let code = c.code();
        let class = if !(0x20..=0x7A).contains(&code) {
            0
        } else {
            let i = (code - 0x20) as usize;
            if self.base != Base::Standard && code == b'n' as u32 {
                VALIDATION_CLASS_N
            } else {
                STANDARD_CLASS[i]
            }
        };
        if self.base == Base::Standard {
            STANDARD_NEXT[(class * 8 + previous) as usize]
        } else {
            VALIDATION_NEXT[(class * 9 + previous) as usize]
        }
    }

    unsafe fn validate_and_update_state_at_end_of_format_string(&mut self) -> bool {
        if self.base == Base::Standard {
            return true;
        }
        if !self.validate(self.state == ST_NORMAL || self.state == ST_TYPE) {
            return false;
        }
        if self.base != Base::Positional
            || self.format_mode != Mode::Positional
            || self.current_pass != POSITION_SCAN
        {
            return true;
        }
        // Every parameter up to the highest one named gets its place in the
        // va_list, read off in order by type. A gap is a missing position.
        let mut i = 0;
        while i <= self.maximum_index {
            self.parameters[i as usize].va = self.va;
            match self.parameters[i as usize].actual_type {
                ParamType::Int32 => {
                    self.va.read::<i32>();
                }
                ParamType::Int64 | ParamType::Real64 => {
                    self.va.read::<u64>();
                }
                ParamType::Pointer => {
                    self.va.read::<usize>();
                }
                ParamType::Unused => {
                    self.host.invalid_parameter(EINVAL);
                    return false;
                }
            }
            i += 1;
        }
        true
    }

    fn should_format(&self) -> bool {
        self.base != Base::Positional
            || self.current_pass != POSITION_SCAN
            || self.format_mode == Mode::Nonpositional
    }

    /// `extract_argument_from_va_list`. `None` is failure; in the positional
    /// scan the value is not read, only recorded, and `Some(0)` comes back.
    unsafe fn extract(&mut self, req: Req) -> Option<u64> {
        if self.base != Base::Positional || self.format_mode == Mode::Nonpositional {
            return Some(read_arg(&mut self.va, req));
        }
        if !self.validate(self.type_index >= 0 && self.type_index < ARGMAX) {
            return None;
        }
        let ti = self.type_index as usize;
        if self.current_pass == POSITION_SCAN {
            let (fc, length) = (self.fc(), self.length);
            if !self.validate_and_store_parameter_data(ti, param_type(req), fc, length) {
                return None;
            }
            return Some(0);
        }
        let mut peek = self.parameters[ti].va;
        Some(read_arg(&mut peek, req))
    }

    unsafe fn update_width_or_precision(&mut self, is_width: bool) -> bool {
        if self.base != Base::Positional || self.format_mode == Mode::Nonpositional {
            let value = self.va.read::<i32>();
            if is_width {
                self.field_width = value;
            } else {
                self.precision = value;
            }
            return true;
        }
        let mut end = core::ptr::null();
        let index = self.tcstol(self.format_it, &mut end).wrapping_sub(1);
        self.format_it = end.add(1);
        if self.current_pass == POSITION_SCAN {
            if !self.validate(index >= 0 && (*end).code() == b'$' as u32 && index < ARGMAX) {
                return false;
            }
            if index > self.maximum_index {
                self.maximum_index = index;
            }
            let (fc, length) = (self.fc(), self.length);
            return self.validate_and_store_parameter_data(index as usize, ParamType::Int32, fc, length);
        }
        let mut peek = self.parameters[index as usize].va;
        let value = peek.read::<i32>();
        if is_width {
            self.field_width = value;
        } else {
            self.precision = value;
        }
        true
    }

    unsafe fn validate_state_for_type_case_a(&mut self) -> bool {
        if self.base == Base::Positional
            && self.format_mode == Mode::Positional
            && self.current_pass == POSITION_SCAN
        {
            if !self.validate(self.type_index >= 0 && self.type_index < ARGMAX) {
                return false;
            }
            let (ti, fc, length) = (self.type_index as usize, self.fc(), self.length);
            return self.validate_and_store_parameter_data(ti, ParamType::Real64, fc, length);
        }
        true
    }

    fn should_skip_normal_state_processing(&self) -> bool {
        if self.base != Base::Positional {
            return false;
        }
        if self.current_pass == POSITION_SCAN && self.format_mode == Mode::Positional {
            return true;
        }
        self.current_pass == OUTPUT && self.format_mode == Mode::Unknown
    }

    unsafe fn validate_and_update_state_at_beginning_of_format_character(&mut self) -> bool {
        if self.base != Base::Positional {
            return true;
        }
        if self.state != ST_PERCENT || self.peek_format(0) == b'%' as u32 {
            return true;
        }
        if self.format_mode == Mode::Unknown {
            let first = self.peek_format(0);
            if first < b'0' as u32 || first > b'9' as u32 {
                self.format_mode = Mode::Nonpositional;
            } else {
                let mut end = core::ptr::null();
                if self.tcstol(self.format_it, &mut end) > 0 && (*end).code() == b'$' as u32 {
                    if self.current_pass == POSITION_SCAN {
                        self.parameters = [UNUSED_PARAM; ARGMAX as usize];
                    }
                    self.format_mode = Mode::Positional;
                } else {
                    self.format_mode = Mode::Nonpositional;
                }
            }
        }
        if self.format_mode != Mode::Positional {
            return true;
        }
        let mut end = core::ptr::null();
        self.type_index = self.tcstol(self.format_it, &mut end).wrapping_sub(1);
        self.format_it = end.add(1);
        if self.current_pass != POSITION_SCAN {
            return true;
        }
        if !self.validate(self.type_index >= 0 && (*end).code() == b'$' as u32 && self.type_index < ARGMAX) {
            return false;
        }
        if self.type_index > self.maximum_index {
            self.maximum_index = self.type_index;
        }
        true
    }

    fn should_skip_type_state_output(&self) -> bool {
        self.base == Base::Positional
            && self.format_mode == Mode::Positional
            && self.current_pass == POSITION_SCAN
    }

    /// `is_positional_parameter_reappearance_consistent`.
    fn reappearance_consistent(&self, parameter: &Param, actual_type: ParamType, format_type: u32, length: Len) -> bool {
        let is_pointer = |c: u32| c == b'p' as u32;
        let is_string = |c: u32| c == b's' as u32 || c == b'S' as u32;
        let is_character = |c: u32| c == b'c' as u32 || c == b'C' as u32;
        let is_integral = |c: u32| b"diouxX*".iter().any(|&x| x as u32 == c);

        let (old_pointer, new_pointer) = (is_pointer(parameter.format_type), is_pointer(format_type));
        if old_pointer || new_pointer {
            return old_pointer == new_pointer;
        }
        let (old_string, new_string) = (is_string(parameter.format_type), is_string(format_type));
        let (old_char, new_char) = (is_character(parameter.format_type), is_character(format_type));
        if old_string || new_string || old_char || new_char {
            if old_string != new_string || old_char != new_char {
                return false;
            }
            let old_wide = is_wide_character_specifier::<C>(self.options, parameter.format_type, parameter.length);
            let new_wide = is_wide_character_specifier::<C>(self.options, format_type, length);
            return old_wide == new_wide;
        }
        let (old_integral, new_integral) = (is_integral(parameter.format_type), is_integral(format_type));
        if old_integral || new_integral {
            if old_integral != new_integral {
                return false;
            }
            if (parameter.length == Len::I) != (length == Len::I) {
                return false;
            }
            return to_integer_size(parameter.length) == to_integer_size(length);
        }
        parameter.actual_type == actual_type
    }

    unsafe fn validate_and_store_parameter_data(
        &mut self,
        index: usize,
        actual_type: ParamType,
        format_type: u32,
        length: Len,
    ) -> bool {
        let parameter = self.parameters[index];
        if parameter.actual_type == ParamType::Unused {
            let p = &mut self.parameters[index];
            p.actual_type = actual_type;
            p.format_type = format_type;
            p.length = length;
            return true;
        }
        let consistent = self.reappearance_consistent(&parameter, actual_type, format_type, length);
        self.validate(consistent)
    }

    // ── process ──────────────────────────────────────────────────────────────

    pub(crate) unsafe fn process(&mut self) -> i32 {
        if !self.adapter.validate(self.host) {
            return -1;
        }
        if !self.validate(!self.format_it.is_null()) {
            return -1;
        }
        while self.advance_to_next_pass() {
            self.string_length = 0;
            self.state = ST_NORMAL;
            loop {
                self.format_char = *self.format_it;
                self.format_it = self.format_it.add(1);
                if self.fc() == 0 || self.characters_written < 0 {
                    break;
                }
                self.state = self.find_next_state(self.format_char, self.state);
                if !self.validate_and_update_state_at_beginning_of_format_character() {
                    return -1;
                }
                if self.state >= ST_INVALID {
                    self.host.invalid_parameter(EINVAL);
                    return -1;
                }
                let result = match self.state {
                    ST_NORMAL => self.state_case_normal(),
                    ST_PERCENT => self.state_case_percent(),
                    ST_FLAG => self.state_case_flag(),
                    ST_WIDTH => self.state_case_width(),
                    ST_DOT => {
                        self.precision = 0;
                        true
                    }
                    ST_PRECISION => self.state_case_precision(),
                    ST_SIZE => self.state_case_size(),
                    _ => self.state_case_type(),
                };
                if !result {
                    return -1;
                }
            }
            if !self.validate_and_update_state_at_end_of_format_string() {
                return -1;
            }
        }
        self.characters_written
    }

    unsafe fn state_case_normal(&mut self) -> bool {
        if self.should_skip_normal_state_processing() {
            return true;
        }
        let ok = self.state_case_normal_common();
        self.validate(ok)
    }

    unsafe fn state_case_normal_common(&mut self) -> bool {
        if !C::WIDE {
            self.string_is_wide = false;
            // The C hands its (signed) `char` straight to the ctype lookup, so
            // every byte above 0x7F is out of range and never a lead byte.
            let as_char = self.fc() as u8 as i8 as c_int;
            let locale = self.host.get_locale();
            if ctype_value(locale, as_char, LEADBYTE) != 0 {
                let c = self.format_char;
                self.adapter.write_character(c, &mut self.characters_written, self.host);
                self.format_char = *self.format_it;
                self.format_it = self.format_it.add(1);
                if !self.validate(self.fc() != 0) {
                    return false;
                }
            }
        } else {
            self.string_is_wide = true;
        }
        let c = self.format_char;
        self.adapter.write_character(c, &mut self.characters_written, self.host);
        true
    }

    fn state_case_percent(&mut self) -> bool {
        self.field_width = 0;
        self.suppress_output = false;
        self.flags = 0;
        self.precision = -1;
        self.length = Len::None;
        self.string_is_wide = false;
        true
    }

    fn state_case_flag(&mut self) -> bool {
        match self.fc() {
            0x2D => self.set_flag(PF_FL_LEFT),      // '-'
            0x2B => self.set_flag(PF_FL_SIGN),      // '+'
            0x20 => self.set_flag(PF_FL_SIGNSP),    // ' '
            0x23 => self.set_flag(PF_FL_ALTERNATE), // '#'
            0x30 => self.set_flag(PF_FL_LEADZERO),  // '0'
            _ => {}
        }
        true
    }

    /// `parse_int_from_format_string`, under its errno guard: an overflowing
    /// width fails the call WITHOUT the handler, and leaves errno as it was.
    unsafe fn parse_int_from_format_string(&mut self, is_width: bool) -> bool {
        let reset_errno: Cached<c_int> = *self.host.errno();
        let mut end = core::ptr::null();
        let value = self.tcstol(self.format_it.sub(1), &mut end);
        let ok = if self.host.errno().check(ERANGE) || end < self.format_it {
            false
        } else {
            self.format_it = end;
            true
        };
        if ok {
            if is_width {
                self.field_width = value;
            } else {
                self.precision = value;
            }
        }
        *self.host.errno() = reset_errno;
        ok
    }

    unsafe fn state_case_width(&mut self) -> bool {
        if self.fc() != b'*' as u32 {
            return self.parse_int_from_format_string(true);
        }
        if !self.update_width_or_precision(true) {
            return false;
        }
        if !self.should_format() {
            return true;
        }
        if self.field_width < 0 {
            self.set_flag(PF_FL_LEFT);
            self.field_width = self.field_width.wrapping_neg();
        }
        true
    }

    unsafe fn state_case_precision(&mut self) -> bool {
        if self.fc() != b'*' as u32 {
            return self.parse_int_from_format_string(false);
        }
        if !self.update_width_or_precision(false) {
            return false;
        }
        if !self.should_format() {
            return true;
        }
        if self.precision < 0 {
            self.precision = -1;
        }
        true
    }

    unsafe fn state_case_size(&mut self) -> bool {
        let fc = self.fc();
        if fc == b'F' as u32 {
            // Only a length modifier in legacy mode; otherwise it is %F.
            if self.options & LEGACY_MSVCRT_COMPATIBILITY == 0 {
                self.state = ST_TYPE;
                return self.state_case_type();
            }
            return true;
        }
        if fc == b'N' as u32 {
            if self.options & LEGACY_MSVCRT_COMPATIBILITY == 0 {
                self.state = ST_INVALID;
                self.host.invalid_parameter(EINVAL);
                return false;
            }
            return true;
        }
        if !self.validate(self.length == Len::None) {
            return false;
        }
        match fc {
            0x68 => {
                // 'h'
                if self.peek_format(0) == b'h' as u32 {
                    self.format_it = self.format_it.add(1);
                    self.length = Len::Hh;
                } else {
                    self.length = Len::H;
                }
            }
            0x49 => {
                // 'I': I32, I64, or bare I before an integer conversion.
                // The second look-ahead only once the first matched, as the C's
                // `&&` does: a format ending in "%I" has nothing after its NUL.
                let a = self.peek_format(0);
                if a == b'3' as u32 && self.peek_format(1) == b'2' as u32 {
                    self.format_it = self.format_it.add(2);
                    self.length = Len::I32;
                } else if a == b'6' as u32 && self.peek_format(1) == b'4' as u32 {
                    self.format_it = self.format_it.add(2);
                    self.length = Len::I64;
                } else if matches!(a, 0x64 | 0x69 | 0x6F | 0x75 | 0x78 | 0x58) {
                    // d i o u x X
                    self.length = Len::I;
                }
            }
            0x6C => {
                // 'l'
                if self.peek_format(0) == b'l' as u32 {
                    self.format_it = self.format_it.add(1);
                    self.length = Len::Ll;
                } else {
                    self.length = Len::L;
                }
            }
            0x4C => self.length = Len::BigL,
            0x6A => self.length = Len::J,
            0x74 => self.length = Len::T,
            0x7A => self.length = Len::Z,
            0x77 => self.length = Len::W,
            0x54 => self.length = Len::BigT,
            _ => {}
        }
        true
    }

    unsafe fn state_case_type(&mut self) -> bool {
        let result = match self.fc() as u8 {
            b'C' | b'c' => self.type_case_c(),
            b'Z' => self.type_case_z(),
            b'S' | b's' => self.type_case_s(),
            b'A' | b'E' | b'F' | b'G' | b'a' | b'e' | b'f' | b'g' => self.type_case_a(),
            b'd' | b'i' => {
                self.set_flag(PF_FL_SIGNED);
                self.type_case_integer(10, false)
            }
            b'u' => self.type_case_integer(10, false),
            b'o' => {
                if self.has_flag(PF_FL_ALTERNATE) {
                    self.set_flag(PF_FL_FORCEOCTAL);
                }
                self.type_case_integer(8, false)
            }
            b'X' => self.type_case_integer(16, true),
            b'x' => self.type_case_integer(16, false),
            b'p' => {
                self.precision = 2 * core::mem::size_of::<usize>() as i32;
                self.length = if core::mem::size_of::<usize>() == 4 { Len::I32 } else { Len::I64 };
                self.type_case_integer(16, true)
            }
            b'n' => self.type_case_n(),
            _ => false,
        };
        // Only characters the tables class as type characters reach here, and
        // they are all ASCII, so `as u8` above cannot alias a wide one.
        if !result {
            return false;
        }
        if self.should_skip_type_state_output() || self.suppress_output {
            return true;
        }

        let mut prefix = [C::of(0); 3];
        let mut prefix_length = 0usize;
        if self.has_flag(PF_FL_SIGNED) {
            if self.has_flag(PF_FL_NEGATIVE) {
                prefix[prefix_length] = C::of(b'-' as u32);
                prefix_length += 1;
            } else if self.has_flag(PF_FL_SIGN) {
                prefix[prefix_length] = C::of(b'+' as u32);
                prefix_length += 1;
            } else if self.has_flag(PF_FL_SIGNSP) {
                prefix[prefix_length] = C::of(b' ' as u32);
                prefix_length += 1;
            }
        }
        let fc = self.fc();
        let print_integer_0x = (fc == b'x' as u32 || fc == b'X' as u32) && self.has_flag(PF_FL_ALTERNATE);
        let print_floating_point_0x = fc == b'a' as u32 || fc == b'A' as u32;
        if print_integer_0x || print_floating_point_0x {
            prefix[prefix_length] = C::of(b'0' as u32);
            prefix_length += 1;
            let capital = fc == b'X' as u32 || fc == b'A' as u32;
            prefix[prefix_length] = C::of(if capital { b'X' } else { b'x' } as u32);
            prefix_length += 1;
        }

        let padding = self
            .field_width
            .wrapping_sub(self.string_length)
            .wrapping_sub(prefix_length as i32);

        if !self.has_flag(PF_FL_LEFT | PF_FL_LEADZERO) {
            write_multiple(&self.adapter, C::of(b' ' as u32), padding, &mut self.characters_written, self.host);
        }
        self.adapter
            .write_string(prefix.as_ptr(), prefix_length as i32, &mut self.characters_written, self.host);
        if self.has_flag(PF_FL_LEADZERO) && !self.has_flag(PF_FL_LEFT) {
            write_multiple(&self.adapter, C::of(b'0' as u32), padding, &mut self.characters_written, self.host);
        }
        self.write_stored_string();
        if self.characters_written >= 0 && self.has_flag(PF_FL_LEFT) {
            write_multiple(&self.adapter, C::of(b' ' as u32), padding, &mut self.characters_written, self.host);
        }
        true
    }

    unsafe fn type_case_c(&mut self) -> bool {
        if !C::WIDE {
            if is_wide_character_specifier::<C>(self.options, self.fc(), self.length) {
                let Some(v) = self.extract(Req::U16) else { return false };
                if !self.should_format() {
                    return true;
                }
                let wide_character = v as u16;
                let (data, count) = (self.buffer.data::<u8>(), self.buffer.count::<u8>());
                let status = _wctomb_internal(&mut self.string_length, data, count, wide_character, self.host);
                if status != 0 {
                    self.suppress_output = true;
                }
            } else {
                let Some(v) = self.extract(Req::U16) else { return false };
                if !self.should_format() {
                    return true;
                }
                *self.buffer.data::<u8>() = v as u8;
                self.string_length = 1;
            }
            self.string = self.buffer.data::<u8>();
            return true;
        }

        self.string_is_wide = true;
        let Some(v) = self.extract(Req::U16) else { return false };
        if !self.should_format() {
            return true;
        }
        let wide_character = v as u16;
        if !is_wide_character_specifier::<C>(self.options, self.fc(), self.length) {
            let local_buffer = [(wide_character & 0x00FF) as u8, 0];
            let locale = self.host.get_locale();
            let data = self.buffer.data::<u16>();
            let mbc_length = _mbtowc_internal(data, local_buffer.as_ptr(), mb_cur_max(locale), self.host);
            if mbc_length < 0 {
                self.suppress_output = true;
            }
        } else {
            *self.buffer.data::<u16>() = wide_character;
        }
        self.string = self.buffer.data::<u8>();
        self.string_length = 1;
        true
    }

    /// `%Z`: a counted `ANSI_STRING`/`UNICODE_STRING`.
    unsafe fn type_case_z(&mut self) -> bool {
        #[repr(C)]
        struct AnsiString {
            length: u16,
            maximum_length: u16,
            buffer: *mut u8,
        }
        let Some(v) = self.extract(Req::Ptr) else { return false };
        if !self.should_format() {
            return true;
        }
        let string = v as usize as *mut AnsiString;
        if string.is_null() || (*string).buffer.is_null() {
            self.string = NARROW_NULL.as_ptr() as *mut u8;
            self.string_length = 6;
            self.string_is_wide = false;
        } else if is_wide_character_specifier::<C>(self.options, self.fc(), self.length) {
            self.string = (*string).buffer;
            self.string_length = (*string).length as i32 / 2;
            self.string_is_wide = true;
        } else {
            self.string = (*string).buffer;
            self.string_length = (*string).length as i32;
            self.string_is_wide = false;
        }
        true
    }

    unsafe fn type_case_s(&mut self) -> bool {
        let Some(v) = self.extract(Req::Ptr) else { return false };
        self.string = v as usize as *mut u8;
        if !self.should_format() {
            return true;
        }
        let maximum_length = if self.precision == -1 { INT_MAX } else { self.precision };
        if is_wide_character_specifier::<C>(self.options, self.fc(), self.length) {
            if self.string.is_null() {
                self.string = WIDE_NULL.as_ptr() as *mut u8;
            }
            self.string_is_wide = true;
            self.string_length = wcsnlen(self.string as *const u16, maximum_length);
        } else {
            if self.string.is_null() {
                self.string = NARROW_NULL.as_ptr() as *mut u8;
            }
            self.string_length = if C::WIDE {
                // Counting CHARACTERS for a wide sink: a lead byte and its
                // trail are one.
                let locale = self.host.get_locale();
                let mut n = 0;
                let mut p = self.string as *const u8;
                while n < maximum_length && *p != 0 {
                    if ctype_value(locale, *p as c_int, LEADBYTE) != 0 {
                        p = p.add(1);
                    }
                    p = p.add(1);
                    n += 1;
                }
                n
            } else {
                strnlen(self.string, maximum_length)
            };
        }
        true
    }

    unsafe fn type_case_a(&mut self) -> bool {
        self.set_flag(PF_FL_SIGNED);
        if !self.validate_state_for_type_case_a() {
            return false;
        }
        if !self.should_format() {
            return true;
        }
        let fc = self.fc();
        if self.precision < 0 {
            self.precision = if fc == b'a' as u32 || fc == b'A' as u32 { 13 } else { 6 };
        } else if self.precision == 0 && (fc == b'g' as u32 || fc == b'G' as u32) {
            self.precision = 1;
        }
        // `_CVTBUFSIZE + _precision` is `int` arithmetic, converted to size_t
        // afterwards: a huge precision wraps NEGATIVE and then fails the
        // buffer's size check (ENOMEM, precision capped) instead of asking the
        // heap for gigabytes.
        let wanted = (CVTBUFSIZE as i32).wrapping_add(self.precision) as isize as usize;
        if !self.buffer.ensure::<u8>(wanted, self.host) {
            self.precision = (self.buffer.count::<u8>() - CVTBUFSIZE) as i32;
        }
        self.string = self.buffer.data::<u8>();

        let Some(bits) = self.extract(Req::Double) else { return false };
        let value = bits;
        let (data, count) = (self.buffer.data::<u8>(), self.buffer.count::<u8>());
        let (scratch, scratch_count) = (self.buffer.scratch_data::<u8>(), self.buffer.count::<u8>());
        __acrt_fp_format(
            &value,
            data,
            count,
            scratch,
            scratch_count,
            fc as c_int,
            self.precision,
            self.options,
            ROUNDING_STANDARD,
            self.host,
        );

        if self.has_flag(PF_FL_ALTERNATE) && self.precision == 0 {
            let locale = self.host.get_locale();
            force_decimal_point(self.string, locale);
        }
        if (fc == b'g' as u32 || fc == b'G' as u32) && !self.has_flag(PF_FL_ALTERNATE) {
            let locale = self.host.get_locale();
            crop_zeroes(self.string, locale);
        }
        if *self.string == b'-' {
            self.set_flag(PF_FL_NEGATIVE);
            self.string = self.string.add(1);
        }
        // inf/nan: no "0x", and padded with spaces, not zeroes.
        let first = *self.string;
        if matches!(first, b'i' | b'I' | b'n' | b'N') {
            self.unset_flag(PF_FL_LEADZERO);
            self.format_char = C::of(b's' as u32);
        }
        self.string_length = strnlen(self.string, INT_MAX);
        true
    }

    unsafe fn type_case_integer(&mut self, radix: u32, capital_hexits: bool) -> bool {
        let integer_size = to_integer_size(self.length);
        let signed = self.has_flag(PF_FL_SIGNED);
        let req = match (integer_size, signed) {
            (1, true) => Req::I8,
            (1, false) => Req::U8,
            (2, true) => Req::I16,
            (2, false) => Req::U16,
            (4, true) => Req::I32,
            (4, false) => Req::U32,
            (8, true) => Req::I64,
            (8, false) => Req::U64,
            _ => {
                self.host.invalid_parameter(EINVAL);
                return false;
            }
        };
        let Some(original_number) = self.extract(req) else { return false };
        if !self.should_format() {
            return true;
        }
        let original_number = original_number as i64;
        let number = if signed && original_number < 0 {
            self.set_flag(PF_FL_NEGATIVE);
            original_number.wrapping_neg() as u64
        } else {
            original_number as u64
        };

        if self.precision < 0 {
            self.precision = 1;
        } else {
            self.unset_flag(PF_FL_LEADZERO);
            // The C ignores this result.
            self.buffer.ensure::<C>(self.precision as usize, self.host);
        }
        if number == 0 {
            self.unset_flag(PF_FL_ALTERNATE);
        }
        self.string_is_wide = C::WIDE;

        // The 32-bit path formats the TRUNCATED value, as the C does.
        let number = if integer_size == 8 { number } else { number as u32 as u64 };
        let last_digit = self.buffer.data::<C>().add(self.buffer.count::<C>() - 1);
        let mut p = last_digit;
        let mut n = number;
        while self.precision > 0 || n != 0 {
            self.precision -= 1;
            let mut digit = (n % radix as u64) as u32 + b'0' as u32;
            n /= radix as u64;
            if digit > b'9' as u32 {
                digit = adjust_hexit(digit, capital_hexits);
            }
            *p = C::of(digit);
            p = p.sub(1);
        }
        self.string_length = last_digit.offset_from(p) as i32;
        p = p.add(1);
        self.string = p as *mut u8;

        if self.has_flag(PF_FL_FORCEOCTAL)
            && (self.string_length == 0 || (*(self.string as *const C)).code() != b'0' as u32)
        {
            let q = (self.string as *mut C).sub(1);
            *q = C::of(b'0' as u32);
            self.string = q as *mut u8;
            self.string_length += 1;
        }
        true
    }

    unsafe fn type_case_n(&mut self) -> bool {
        let Some(v) = self.extract(Req::Ptr) else { return false };
        if !self.should_format() {
            return true;
        }
        if _get_printf_count_output() == 0 {
            self.host.invalid_parameter(EINVAL);
            return false;
        }
        let p = v as usize as *mut u8;
        let n = self.characters_written;
        match to_integer_size(self.length) {
            1 => *(p as *mut i8) = n as i8,
            2 => (p as *mut i16).write_unaligned(n as i16),
            4 => (p as *mut i32).write_unaligned(n),
            8 => (p as *mut i64).write_unaligned(n as i64),
            _ => {
                self.host.invalid_parameter(EINVAL);
                return false;
            }
        }
        self.suppress_output = true;
        true
    }

    unsafe fn write_stored_string(&mut self) {
        if !C::WIDE {
            if !self.string_is_wide || self.string_length <= 0 {
                self.adapter
                    .write_string(self.string as *const C, self.string_length, &mut self.characters_written, self.host);
                return;
            }
            let mut p = self.string as *const u16;
            let mut i = 0;
            while i != self.string_length {
                let mut local_buffer = [0u8; MB_LEN_MAX + 1];
                let mut mbc_length: c_int = 0;
                let status = _wctomb_internal(&mut mbc_length, local_buffer.as_mut_ptr(), local_buffer.len(), *p, self.host);
                p = p.add(1);
                if status != 0 || mbc_length == 0 {
                    self.characters_written = -1;
                    return;
                }
                self.adapter.write_string(
                    local_buffer.as_ptr() as *const C,
                    mbc_length,
                    &mut self.characters_written,
                    self.host,
                );
                i += 1;
            }
            return;
        }
        if self.string_is_wide || self.string_length <= 0 {
            self.adapter
                .write_string(self.string as *const C, self.string_length, &mut self.characters_written, self.host);
            return;
        }
        let locale = self.host.get_locale();
        let mut p = self.string as *const u8;
        let mut i = 0;
        while i != self.string_length {
            let mut wide_character: u16 = 0;
            let mbc_length = _mbtowc_internal(&mut wide_character, p, mb_cur_max(locale), self.host);
            if mbc_length <= 0 {
                self.characters_written = -1;
                return;
            }
            self.adapter
                .write_character(C::of(wide_character as u32), &mut self.characters_written, self.host);
            p = p.add(mbc_length as usize);
            i += 1;
        }
    }
}

/// `adjust_hexit`: an out-of-range "digit" moved onto 'a'..'f' or 'A'..'F'.
fn adjust_hexit(value: u32, capitalize: bool) -> u32 {
    let base = if capitalize { b'A' } else { b'a' } as u32;
    let offset = base.wrapping_sub(b'9' as u32 + 1);
    offset.wrapping_add(value) & 0xFF
}

static NARROW_NULL: [u8; 7] = *b"(null)\0";
static WIDE_NULL: [u16; 7] = [b'(' as u16, b'n' as u16, b'u' as u16, b'l' as u16, b'l' as u16, b')' as u16, 0];

unsafe fn strnlen(s: *const u8, max: i32) -> i32 {
    let mut n = 0;
    while n < max && *s.add(n as usize) != 0 {
        n += 1;
    }
    n
}

unsafe fn wcsnlen(s: *const u16, max: i32) -> i32 {
    let mut n = 0;
    while n < max && *s.add(n as usize) != 0 {
        n += 1;
    }
    n
}

// ── output.cpp ────────────────────────────────────────────────────────────────

/// `common_vfprintf`.
unsafe fn common_vfprintf<C: StreamChar>(
    base: Base,
    options: u64,
    stream: *mut c_void,
    format: *const C,
    locale: *mut CrtLocalePointers,
    arglist: *mut u8,
) -> c_int {
    let mut host = PtdHost::new(locale);
    if stream.is_null() {
        host.invalid_parameter(EINVAL);
        return -1;
    }
    if format.is_null() {
        host.invalid_parameter(EINVAL);
        return -1;
    }
    // `__acrt_lock_stream_and_call`, then the temporary-buffering guard, whose
    // destructor runs before the unlock.
    _lock_file(stream);
    let flag = __acrt_stdio_begin_temporary_buffering_nolock(stream);
    let result = Processor::<C, StreamAdapter>::new(StreamAdapter(stream), options, format, &mut host, arglist, base)
        .process();
    __acrt_stdio_end_temporary_buffering_nolock(flag, stream, &mut host);
    _unlock_file(stream);
    result
}

/// `common_vsprintf`.
unsafe fn common_vsprintf<C: OutChar>(
    base: Base,
    options: u64,
    buffer: *mut C,
    buffer_count: usize,
    format: *const C,
    host: &mut PtdHost,
    arglist: *mut u8,
) -> c_int {
    if format.is_null() {
        host.invalid_parameter(EINVAL);
        return -1;
    }
    if !(buffer_count == 0 || !buffer.is_null()) {
        host.invalid_parameter(EINVAL);
        return -1;
    }
    let mut context = StringContext {
        buffer,
        buffer_count,
        buffer_used: 0,
        continue_count: options & STANDARD_SNPRINTF_BEHAVIOR != 0 || buffer.is_null(),
    };
    let result = Processor::<C, StringAdapter<C>>::new(
        StringAdapter(&mut context),
        options,
        format,
        host,
        arglist,
        base,
    )
    .process();

    if buffer.is_null() {
        return result;
    }
    let nul = C::of(0);
    if options & LEGACY_VSPRINTF_NULL_TERMINATION != 0 {
        if buffer_count == 0 && result != 0 {
            return -1;
        } else if context.buffer_used != buffer_count {
            *buffer.add(context.buffer_used) = nul;
        } else if result >= 0 && result as usize > buffer_count {
            return -1;
        }
    } else if options & STANDARD_SNPRINTF_BEHAVIOR != 0 {
        if buffer_count == 0 {
            // no-op
        } else if result < 0 {
            *buffer = nul;
        } else if context.buffer_used == buffer_count {
            *buffer.add(buffer_count - 1) = nul;
        } else {
            *buffer.add(context.buffer_used) = nul;
        }
    } else if buffer_count == 0 {
        return -1;
    } else if context.buffer_used == buffer_count {
        *buffer.add(buffer_count - 1) = nul;
        return -2;
    } else {
        *buffer.add(context.buffer_used) = nul;
    }
    result
}

/// `common_vsprintf_s`.
unsafe fn common_vsprintf_s<C: OutChar>(
    options: u64,
    buffer: *mut C,
    buffer_count: usize,
    format: *const C,
    host: &mut PtdHost,
    arglist: *mut u8,
) -> c_int {
    if format.is_null() {
        host.invalid_parameter(EINVAL);
        return -1;
    }
    if buffer.is_null() || buffer_count == 0 {
        host.invalid_parameter(EINVAL);
        return -1;
    }
    let result = common_vsprintf(Base::Validation, options, buffer, buffer_count, format, host, arglist);
    if result < 0 {
        *buffer = C::of(0);
    }
    if result == -2 {
        // "Buffer too small"
        host.invalid_parameter(ERANGE);
        return -1;
    }
    result
}

/// `common_vsnprintf_s`.
unsafe fn common_vsnprintf_s<C: OutChar>(
    options: u64,
    buffer: *mut C,
    buffer_count: usize,
    max_count: usize,
    format: *const C,
    host: &mut PtdHost,
    arglist: *mut u8,
) -> c_int {
    if format.is_null() {
        host.invalid_parameter(EINVAL);
        return -1;
    }
    if max_count == 0 && buffer.is_null() && buffer_count == 0 {
        return 0;
    }
    if buffer.is_null() || buffer_count == 0 {
        host.invalid_parameter(EINVAL);
        return -1;
    }

    // The restore point: taken now, disabled, and re-enabled only on the two
    // truncation paths -- so a truncating `_TRUNCATE` call leaves errno as it
    // found it rather than ERANGE.
    let restore_point: Cached<c_int> = *host.errno();
    let result;
    if buffer_count > max_count {
        result = common_vsprintf(Base::Validation, options, buffer, max_count + 1, format, host, arglist);
        if result == -2 {
            if host.errno().check(ERANGE) {
                *host.errno() = restore_point;
            }
            return -1;
        }
    } else {
        result = common_vsprintf(Base::Validation, options, buffer, buffer_count, format, host, arglist);
        *buffer.add(buffer_count - 1) = C::of(0);
        if result == -2 && max_count == TRUNCATE {
            if host.errno().check(ERANGE) {
                *host.errno() = restore_point;
            }
            return -1;
        }
    }

    if result < 0 {
        *buffer = C::of(0);
        if result == -2 {
            host.invalid_parameter(ERANGE);
            return -1;
        }
        return -1;
    }
    result
}

macro_rules! vfprintf {
    ($name:ident, $c:ty, $base:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(
            options: u64,
            stream: *mut c_void,
            format: *const $c,
            locale: *mut CrtLocalePointers,
            arglist: *mut u8,
        ) -> c_int {
            common_vfprintf::<$c>($base, options, stream, format, locale, arglist)
        }
    };
}

vfprintf!(__stdio_common_vfprintf, u8, Base::Standard);
vfprintf!(__stdio_common_vfwprintf, u16, Base::Standard);
vfprintf!(__stdio_common_vfprintf_s, u8, Base::Validation);
vfprintf!(__stdio_common_vfwprintf_s, u16, Base::Validation);
vfprintf!(__stdio_common_vfprintf_p, u8, Base::Positional);
vfprintf!(__stdio_common_vfwprintf_p, u16, Base::Positional);

macro_rules! vsprintf {
    ($name:ident, $c:ty, $base:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(
            options: u64,
            buffer: *mut $c,
            buffer_count: usize,
            format: *const $c,
            locale: *mut CrtLocalePointers,
            arglist: *mut u8,
        ) -> c_int {
            let mut host = PtdHost::new(locale);
            common_vsprintf::<$c>($base, options, buffer, buffer_count, format, &mut host, arglist)
        }
    };
}

vsprintf!(__stdio_common_vsprintf, u8, Base::Standard);
vsprintf!(__stdio_common_vswprintf, u16, Base::Standard);
vsprintf!(__stdio_common_vsprintf_p, u8, Base::Positional);
vsprintf!(__stdio_common_vswprintf_p, u16, Base::Positional);

macro_rules! vsprintf_s {
    ($name:ident, $c:ty) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(
            options: u64,
            buffer: *mut $c,
            buffer_count: usize,
            format: *const $c,
            locale: *mut CrtLocalePointers,
            arglist: *mut u8,
        ) -> c_int {
            let mut host = PtdHost::new(locale);
            common_vsprintf_s::<$c>(options, buffer, buffer_count, format, &mut host, arglist)
        }
    };
}

vsprintf_s!(__stdio_common_vsprintf_s, u8);
vsprintf_s!(__stdio_common_vswprintf_s, u16);

macro_rules! vsnprintf_s {
    ($name:ident, $c:ty) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(
            options: u64,
            buffer: *mut $c,
            buffer_count: usize,
            max_count: usize,
            format: *const $c,
            locale: *mut CrtLocalePointers,
            arglist: *mut u8,
        ) -> c_int {
            let mut host = PtdHost::new(locale);
            common_vsnprintf_s::<$c>(options, buffer, buffer_count, max_count, format, &mut host, arglist)
        }
    };
}

vsnprintf_s!(__stdio_common_vsnprintf_s, u8);
vsnprintf_s!(__stdio_common_vsnwprintf_s, u16);
