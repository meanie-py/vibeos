//! `stdio/input.cpp` and the engine it instantiates
//! (`corecrt_internal_stdio_input.h`, 1613 lines): the four
//! `__stdio_common_v*scanf` entry points every scanf-family function is a thin
//! inline over.
//!
//! ## Structure
//!
//! Three pieces, one per source file:
//! * [`chars`] (this file) -- the two input adapters (`stream_input_adapter`,
//!   `string_input_adapter`) and the `Char` trait every other piece is generic
//!   over.
//! * [`parser`] -- `format_string_parser` and the scanset bitmap.
//! * [`engine`] -- the character-source wrapper the integer/float conversions
//!   read through, and `input_processor`'s main loop.
//!
//! `console_input_adapter` (the `_cscanf` family, `conio.h`) is NOT ported
//! here: `input.cpp` never instantiates it, and `_cscanf` is owned by whichever
//! piece takes `conio.cpp` -- not `stdio/`.
//!
//! ## Reuse, and the one place it stops
//!
//! `crate::strtox::Char` (digit value, `is_space`) is reused as-is for both
//! integer and float scanning -- it is already exactly `__crt_strtox::parse_digit`
//! / `is_space`, character-source-agnostic.
//!
//! Integer scanning is a full, independent port of `__crt_strtox::parse_integer`
//! against [`engine::CharSource`] (see that module) -- `crate::strtox::parse`
//! cannot be called: it walks a raw `*const C`, which is exactly wrong for a
//! *stream*, whose input is not a C string at all, and unsafe even for the
//! *string* adapter, whose `buffer_count_for_stream` prefix is not guaranteed
//! NUL-terminated at its own end (a scan that ran to the count limit without
//! finding a NUL must not read past it looking for more digits).
//!
//! Floating-point scanning has the same character-source problem, but the
//! payload behind it -- correctly-rounded decimal/hex-to-double conversion via
//! `big_integer.rs` -- is real, delicate, paid-for work that must not be
//! duplicated. `crate::strtod`'s `convert_decimal`, `convert_hexadecimal` and
//! `write_result` (the only pieces that do that conversion) are private to
//! that module, and `FloatingPointString`'s fields are private too, so there
//! is no seam to call into with pre-scanned digits. **Needed change, reported
//! rather than made:** make those three functions and that struct's fields
//! `pub(crate)`, so a character-source-driven scan can hand its digits/exponent
//! straight to `write_result` instead of going through a round-trip string.
//! Until then, [`engine`] scans the input with the *grammar* transcribed from
//! `parse_floating_point_from_source` (so the character-source quirks below are
//! still exactly reproduced), copies the accepted characters VERBATIM into a
//! private heap buffer it NUL-terminates itself, and hands that buffer to the
//! already-`pub(crate)` `crate::strtod::parse_floating_point`. Same text in,
//! same conversion code, same rounding -- one extra copy and reparse of a
//! short buffer, not a re-derivation of the math.
//!
//! ## The character-source quirks this ports, deliberately
//!
//! `__crt_strtox::input_adapter_character_source` (corecrt_internal_strtox.h)
//! is NOT a real backtracking cursor: `restore_state` only ever *compares* the
//! saved and current get-counts and, on mismatch, sets a hard failure latch
//! (`*_succeeded = false`) -- it never un-reads anything beyond the ONE character
//! each caller already explicitly `unget()`s. Consequences kept faithfully by
//! [`engine::CharSource`]:
//! * `%d` on `"-x"` consumes the `-` and it is GONE -- not put back -- even
//!   though the conversion fails. (`"x"` alone puts its one character back
//!   fine: no sign was consumed before the digit scan gave up.)
//! * `%f` on a broken `"inf"`/`"nan(...)"` prefix has the same one-way-latch
//!   behaviour; a *fully* failed match (zero characters ever read) is the only
//!   case that reports as an ordinary non-match rather than an input failure.
//! * width capping (`%NNc`/`%NNd`/...) is enforced by the wrapper, not the
//!   underlying adapter: once the cap is hit, further "reads" return a `'\0'`
//!   sentinel without ever touching the real stream/string, and the matching
//!   `unget` of that sentinel is a no-op too -- so a width-limited scan cannot
//!   accidentally push back a character it never really took.

use core::ffi::{c_int, c_void};

use crate::locale::CrtLocalePointers;
use crate::locale_update::LocaleUpdate;
use crate::stdio_layout::{textmode_safe, tm_unicode_safe, StreamData, TEXT_MODE_ANSI};

pub mod engine;
pub mod parser;

use engine::InputProcessor;

const EINVAL: c_int = 22;
const EOF: c_int = -1;

// corecrt_stdio_config.h -- distinct bit values from printf's option word
// (output.rs's `LEGACY_WIDE_SPECIFIERS`/`LEGACY_MSVCRT_COMPATIBILITY` are 0x4/0x8;
// these are scanf's own, unrelated numbering).
pub(crate) const SCANF_SECURECRT: u64 = 0x1;
pub(crate) const SCANF_LEGACY_WIDE_SPECIFIERS: u64 = 0x2;
pub(crate) const SCANF_LEGACY_MSVCRT_COMPATIBILITY: u64 = 0x4;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();

    // Owned by the `bufio` piece (`_getc_nolock`/`_ungetc_nolock` and their wide
    // forms). Prototypes transcribed from stdio.h / corecrt_wstdio.h.
    fn _getc_nolock(stream: *mut c_void) -> c_int;
    fn _ungetc_nolock(c: c_int, stream: *mut c_void) -> c_int;
    fn _getwc_nolock(stream: *mut c_void) -> u16;
    fn _ungetwc_nolock(c: u16, stream: *mut c_void) -> u16;
    fn _fileno(stream: *mut c_void) -> c_int;
}

#[cold]
unsafe fn invalid_parameter_old(err: c_int) {
    // `_VALIDATE_RETURN`: plain errno, then the (old-style) handler -- input.cpp
    // is one of the files still on `_ALLOW_OLD_VALIDATE_MACROS`, not the
    // ptd-cached `_UCRT_VALIDATE_*` family output.rs uses.
    *_errno() = err;
    _invalid_parameter_noinfo();
}

// ── the character type ───────────────────────────────────────────────────────

/// `__acrt_stdio_char_traits<Character>`'s bits this engine needs, layered over
/// `crate::strtox::Char` (digit value, `is_space`) so integer/float scanning get
/// those for free.
pub(crate) trait Char: crate::strtox::Char + 'static {
    const WIDE: bool;
    /// `traits::eof`: `EOF` (-1) for `char`, `WEOF` (0xFFFF) for `wchar_t`. Held
    /// as `i32` so both fit one adapter-return type.
    const EOF_CODE: i32;
    fn of(c: u32) -> Self;
}

impl Char for u8 {
    const WIDE: bool = false;
    const EOF_CODE: i32 = -1;
    fn of(c: u32) -> u8 {
        c as u8
    }
}

impl Char for u16 {
    const WIDE: bool = true;
    const EOF_CODE: i32 = 0xFFFF;
    fn of(c: u32) -> u16 {
        c as u16
    }
}

// ── input adapters ────────────────────────────────────────────────────────────

/// The adapter interface every `input_processor` reads through: `get`/`unget`
/// on the underlying source, plus `%n`'s running count.
pub(crate) trait Adapter<C: Char> {
    unsafe fn validate(&mut self) -> bool;
    /// Returns the character's code, or `C::EOF_CODE`.
    unsafe fn get(&mut self) -> i32;
    /// `c` may be `C::EOF_CODE`, in which case this is a no-op (matches every
    /// C++ adapter's `if (c == traits::eof) return;`).
    unsafe fn unget(&mut self, c: i32);
    fn characters_read(&self) -> u64;
}

/// `stream_input_adapter<Character>`.
pub(crate) struct StreamAdapter<C> {
    stream: *mut c_void,
    characters_read: u64,
    _marker: core::marker::PhantomData<C>,
}

impl<C> StreamAdapter<C> {
    pub(crate) fn new(stream: *mut c_void) -> Self {
        Self { stream, characters_read: 0, _marker: core::marker::PhantomData }
    }
}

impl Adapter<u8> for StreamAdapter<u8> {
    unsafe fn validate(&mut self) -> bool {
        if self.stream.is_null() {
            invalid_parameter_old(EINVAL);
            return false;
        }
        // `validate_stream_is_ansi_if_required` / `_VALIDATE_STREAM_ANSI_RETURN`:
        // the same check printf's narrow path makes (output.rs's `StreamChar`),
        // transcribed again here rather than imported -- `output` is a separate,
        // not-implied Cargo feature, and this piece must build without it.
        let data = self.stream as *mut StreamData;
        if (*data).is_string_backed() {
            return true;
        }
        let fh = _fileno(self.stream);
        if textmode_safe(fh) == TEXT_MODE_ANSI && !tm_unicode_safe(fh) {
            return true;
        }
        invalid_parameter_old(EINVAL);
        false
    }
    unsafe fn get(&mut self) -> i32 {
        let c = _getc_nolock(self.stream);
        if c != EOF {
            self.characters_read += 1;
        }
        c
    }
    unsafe fn unget(&mut self, c: i32) {
        if c == EOF {
            return;
        }
        self.characters_read -= 1;
        _ungetc_nolock(c, self.stream);
    }
    fn characters_read(&self) -> u64 {
        self.characters_read
    }
}

impl Adapter<u16> for StreamAdapter<u16> {
    unsafe fn validate(&mut self) -> bool {
        if self.stream.is_null() {
            invalid_parameter_old(EINVAL);
            return false;
        }
        // Narrow-only validation; always true for wide (the C's override is a
        // plain `return true`).
        true
    }
    unsafe fn get(&mut self) -> i32 {
        let c = _getwc_nolock(self.stream);
        if c as i32 != u16::EOF_CODE {
            self.characters_read += 1;
        }
        c as i32
    }
    unsafe fn unget(&mut self, c: i32) {
        if c == u16::EOF_CODE {
            return;
        }
        self.characters_read -= 1;
        _ungetwc_nolock(c as u16, self.stream);
    }
    fn characters_read(&self) -> u64 {
        self.characters_read
    }
}

/// `string_input_adapter<Character>`.
pub(crate) struct StringAdapter<C> {
    first: *const C,
    last: *const C,
    it: *const C,
}

impl<C: Char> StringAdapter<C> {
    pub(crate) fn new(string: *const C, length: usize) -> Self {
        Self { first: string, last: unsafe { string.add(length) }, it: string }
    }
}

impl<C: Char> Adapter<C> for StringAdapter<C> {
    unsafe fn validate(&mut self) -> bool {
        // `_VALIDATE_RETURN(_it != nullptr, ...)`, `_VALIDATE_RETURN(_it <= _last, ...)`.
        if self.it.is_null() {
            invalid_parameter_old(EINVAL);
            return false;
        }
        if self.it > self.last {
            invalid_parameter_old(EINVAL);
            return false;
        }
        true
    }
    unsafe fn get(&mut self) -> i32 {
        if self.it == self.last {
            return C::EOF_CODE;
        }
        let c = (*self.it).code();
        self.it = self.it.add(1);
        c as i32
    }
    unsafe fn unget(&mut self, c: i32) {
        if c == C::EOF_CODE {
            return;
        }
        if self.it == self.first {
            return;
        }
        if self.it == self.last && c == C::EOF_CODE {
            return;
        }
        self.it = self.it.sub(1);
    }
    fn characters_read(&self) -> u64 {
        unsafe { self.it.offset_from(self.first) as u64 }
    }
}

// ── input.cpp ─────────────────────────────────────────────────────────────────

/// `common_vfscanf`.
unsafe fn common_vfscanf<C: Char>(
    options: u64,
    stream: *mut c_void,
    format: *const C,
    locale: *mut CrtLocalePointers,
    arglist: *mut u8,
) -> c_int
where
    StreamAdapter<C>: Adapter<C>,
{
    if stream.is_null() {
        invalid_parameter_old(EINVAL);
        return EOF;
    }
    if format.is_null() {
        invalid_parameter_old(EINVAL);
        return EOF;
    }
    crate::stdio::with_stream_locked(stream, || {
        let mut locale_update = LocaleUpdate::new(locale);
        let adapter = StreamAdapter::<C>::new(stream);
        let mut processor = InputProcessor::new(adapter, options, format, locale_update.locale_t(), arglist);
        processor.process()
    })
}

/// `common_vsscanf`.
unsafe fn common_vsscanf<C: Char>(
    options: u64,
    buffer: *const C,
    buffer_count: usize,
    format: *const C,
    locale: *mut CrtLocalePointers,
    arglist: *mut u8,
) -> c_int
where
    StringAdapter<C>: Adapter<C>,
{
    if buffer.is_null() {
        invalid_parameter_old(EINVAL);
        return EOF;
    }
    if format.is_null() {
        invalid_parameter_old(EINVAL);
        return EOF;
    }
    // `char_traits::tcsnlen(buffer, buffer_count)`: the lesser of the declared
    // count and the distance to the first NUL. If neither is found within
    // `buffer_count`, the effective length IS `buffer_count` and the region may
    // not be NUL-terminated at all -- the adapter must never read past it.
    let buffer_count_for_stream = tcsnlen(buffer, buffer_count);

    let mut locale_update = LocaleUpdate::new(locale);
    let adapter = StringAdapter::<C>::new(buffer, buffer_count_for_stream);
    let mut processor = InputProcessor::new(adapter, options, format, locale_update.locale_t(), arglist);
    processor.process()
}

/// `char_traits::tcsnlen`: `strnlen`/`wcsnlen`, transcribed rather than declared
/// `extern` -- trivial, and avoids yet another cross-piece dependency for one
/// bounded scan.
unsafe fn tcsnlen<C: Char>(s: *const C, max: usize) -> usize {
    let mut n = 0;
    while n < max && (*s.add(n)).code() != 0 {
        n += 1;
    }
    n
}

macro_rules! vfscanf {
    ($name:ident, $c:ty) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(
            options: u64,
            stream: *mut c_void,
            format: *const $c,
            locale: *mut CrtLocalePointers,
            arglist: *mut u8,
        ) -> c_int {
            common_vfscanf::<$c>(options, stream, format, locale, arglist)
        }
    };
}

vfscanf!(__stdio_common_vfscanf, u8);
vfscanf!(__stdio_common_vfwscanf, u16);

macro_rules! vsscanf {
    ($name:ident, $c:ty) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(
            options: u64,
            buffer: *const $c,
            buffer_count: usize,
            format: *const $c,
            locale: *mut CrtLocalePointers,
            arglist: *mut u8,
        ) -> c_int {
            common_vsscanf::<$c>(options, buffer, buffer_count, format, locale, arglist)
        }
    };
}

vsscanf!(__stdio_common_vsscanf, u8);
vsscanf!(__stdio_common_vswscanf, u16);
