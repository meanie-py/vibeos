//! The character-source wrapper every numeric conversion reads through, and
//! `__crt_stdio_input::input_processor`'s main loop (corecrt_internal_stdio_input.h:1064-1613).
//!
//! See the module doc in `mod.rs` for why integer scanning is a full
//! independent port of `parse_integer` (not a call into `crate::strtox`), and
//! why floating-point scanning copies its accepted characters into a private
//! buffer and reparses them through `crate::strtod::parse_floating_point`
//! rather than duplicating the correctly-rounded conversion.

use core::ffi::{c_int, c_uint, c_void};

use crate::locale::CrtLocalePointers;
use crate::strtod::{parse_floating_point as strtod_parse_floating_point, Status as FpStatus, Traits, DOUBLE, FLOAT};

use super::parser::{to_integer_length, DirectiveKind, FormatParser, Mode};
use super::{invalid_parameter_old, Adapter, Char};

const EINVAL: c_int = 22;
const ENOMEM: c_int = 12;
const ERANGE: c_int = 34;
const EOF: c_int = -1;
const MB_LEN_MAX: usize = 5;
const UNBOUNDED: usize = usize::MAX;

const FL_SIGNED: u32 = 0x01;
const FL_NEGATIVE: u32 = 0x02;
const FL_OVERFLOW: u32 = 0x04;
const FL_READ_DIGIT: u32 = 0x08;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _malloc_base(size: usize) -> *mut c_void;
    fn _free_base(block: *mut c_void);
    fn isleadbyte(c: c_int) -> c_int;
    /// `wctomb_s` (stdlib.h).
    fn wctomb_s(size_converted: *mut c_int, mbch: *mut u8, size_in_bytes: usize, wch: u16) -> c_int;
    /// `_mbtowc_l` (stdlib.h).
    fn _mbtowc_l(dst: *mut u16, src: *const u8, src_size: usize, locale: *mut CrtLocalePointers) -> c_int;
}

unsafe fn mb_cur_max(locale: *mut CrtLocalePointers) -> usize {
    (*(*locale).locinfo).public.locale_mb_cur_max as usize
}

// ── the varargs cursor ────────────────────────────────────────────────────────

/// A Windows `va_list`: a plain byte cursor (see `output.rs`'s `Va` -- same
/// idea, not shared because it is `pub(crate)` there but private, and scanf
/// only ever pulls pointers and one `unsigned`).
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

// ── the character source ─────────────────────────────────────────────────────

/// `__crt_strtox::input_adapter_character_source<InputAdapter>`.
///
/// **Not a real backtracking cursor.** `restore_state` only ever COMPARES the
/// saved and current get-counts; on a mismatch it sets a one-way failure latch
/// and does not un-read anything beyond what each caller already explicitly
/// `unget()`s one character at a time. See the module doc in `mod.rs` for the
/// user-visible quirks this reproduces on purpose.
///
/// Also records every character it actually reads from (and does not push
/// back to) the underlying adapter, verbatim, into a small heap buffer --
/// purely so the floating-point path can hand the exact substring it scanned
/// to `crate::strtod::parse_floating_point` for conversion. Nothing else reads
/// the recording.
pub(crate) struct CharSource<'a, C: Char, A: Adapter<C>> {
    adapter: &'a mut A,
    max_get_count: u64,
    get_count: u64,
    succeeded: bool,
    record: *mut C,
    record_len: usize,
    record_cap: usize,
}

impl<'a, C: Char, A: Adapter<C>> CharSource<'a, C, A> {
    pub(crate) fn new(adapter: &'a mut A, width: u64) -> Self {
        Self {
            adapter,
            max_get_count: width,
            get_count: 0,
            succeeded: true,
            record: core::ptr::null_mut(),
            record_len: 0,
            record_cap: 0,
        }
    }

    /// `input_adapter_character_source::validate()`: checks only its own two
    /// pointers (always non-null here), never the underlying adapter -- that
    /// is validated once, up front, by `InputProcessor::process`.
    pub(crate) fn validate(&self) -> bool {
        true
    }

    unsafe fn record_push(&mut self, c: C) {
        if self.record_len == self.record_cap {
            let new_cap = if self.record_cap == 0 { 64 } else { self.record_cap * 2 };
            let new_buf = _malloc_base(new_cap * core::mem::size_of::<C>()) as *mut C;
            if new_buf.is_null() {
                // Out of memory mid-scan: stop recording (the eventual reparse
                // will simply see a shorter, still-consistent prefix). This
                // cannot happen in practice for any input a test harness feeds
                // it, and dropping characters here never desyncs `get_count`.
                return;
            }
            if !self.record.is_null() {
                core::ptr::copy_nonoverlapping(self.record, new_buf, self.record_len);
                _free_base(self.record as *mut c_void);
            }
            self.record = new_buf;
            self.record_cap = new_cap;
        }
        *self.record.add(self.record_len) = c;
        self.record_len += 1;
    }

    unsafe fn record_pop(&mut self) {
        if self.record_len > 0 {
            self.record_len -= 1;
        }
    }

    pub(crate) fn recorded(&self) -> (*const C, usize) {
        (self.record, self.record_len)
    }

    /// `get()`.
    pub(crate) unsafe fn get(&mut self) -> C {
        self.get_count += 1;
        if self.max_get_count != 0 && self.get_count > self.max_get_count {
            return C::of(0);
        }
        let c = self.adapter.get();
        if c == C::EOF_CODE {
            return C::of(0);
        }
        let ch = C::of(c as u32);
        self.record_push(ch);
        ch
    }

    /// `unget(c)`.
    pub(crate) unsafe fn unget(&mut self, c: C) {
        self.get_count -= 1;
        if self.max_get_count != 0 && self.get_count > self.max_get_count {
            return;
        }
        // `c == '\0' || c == traits::eof`: a genuine embedded NUL in the input
        // is indistinguishable from the sentinel here -- Microsoft's quirk,
        // not ours; see the module doc.
        if c.code() == 0 || c.code() as i32 == C::EOF_CODE {
            return;
        }
        self.record_pop();
        self.adapter.unget(c.code() as i32);
    }

    pub(crate) fn save_state(&self) -> u64 {
        self.get_count
    }

    /// `restore_state`: comparison, not a rewind. See the struct doc.
    pub(crate) fn restore_state(&mut self, saved: u64) -> bool {
        if saved != self.get_count {
            self.succeeded = false;
            return false;
        }
        true
    }

    /// The `~input_adapter_character_source()` destructor's contribution to
    /// `*_succeeded`, folded into one call: zero characters ever read (the
    /// very first attempt failed) reports as unsuccessful even though nothing
    /// explicitly failed a `restore_state` check.
    pub(crate) fn resolve_success(&mut self) -> bool {
        if self.get_count == 0 {
            self.succeeded = false;
        }
        self.succeeded
    }
}

impl<'a, C: Char, A: Adapter<C>> Drop for CharSource<'a, C, A> {
    fn drop(&mut self) {
        unsafe {
            if !self.record.is_null() {
                _free_base(self.record as *mut c_void);
            }
        }
    }
}

// ── integer scanning: `__crt_strtox::parse_integer`, ported a second time ────

/// `parse_integer<uint64_t>`. The accumulator is ALWAYS 64 bits here -- the
/// header always instantiates it that way for scanf, regardless of the
/// conversion's declared length -- so a `%hhd` can genuinely ERANGE on a
/// 20-digit input before ever truncating to the requested width; `write_integer`
/// then just takes the low bytes, unconditionally, as the header does.
unsafe fn parse_integer<C: Char, A: Adapter<C>>(
    locale: *mut CrtLocalePointers,
    source: &mut CharSource<C, A>,
    mut base: i32,
    is_result_signed: bool,
) -> u64 {
    if !source.validate() {
        return 0;
    }
    if !(base == 0 || (2..=36).contains(&base)) {
        invalid_parameter_old(EINVAL);
        return 0;
    }

    let initial_state = source.save_state();
    let mut c = source.get();

    // `TrimWhitespace` defaults to `true` at this call site in the header (the
    // caller already skipped leading whitespace via `process_whitespace`, so
    // this ordinarily finds none left -- but it is still live code, ported).
    while c.is_space(locale) {
        c = source.get();
    }

    let mut flags: u32 = if is_result_signed { FL_SIGNED } else { 0 };
    if c.code() == b'-' as u32 {
        flags |= FL_NEGATIVE;
    }
    if c.code() == b'-' as u32 || c.code() == b'+' as u32 {
        c = source.get();
    }

    if base == 0 || base == 16 {
        if c.digit() != 0 {
            if base == 0 {
                base = 10;
            }
        } else {
            let next_c = source.get();
            if next_c.code() == b'x' as u32 || next_c.code() == b'X' as u32 {
                if base == 0 {
                    base = 16;
                }
                c = source.get();
            } else {
                if base == 0 {
                    base = 8;
                }
                source.unget(next_c);
            }
        }
    }

    let base_u = base as u64;
    let max_pre_multiply_value = u64::MAX / base_u;
    let mut number: u64 = 0;
    loop {
        let digit = c.digit();
        if digit as u64 >= base_u {
            break;
        }
        flags |= FL_READ_DIGIT;
        let after_multiply = number.wrapping_mul(base_u);
        let after_add = after_multiply.wrapping_add(digit as u64);
        if number > max_pre_multiply_value || after_add < after_multiply {
            flags |= FL_OVERFLOW;
        }
        number = after_add;
        c = source.get();
    }
    source.unget(c);

    if flags & FL_READ_DIGIT == 0 {
        source.restore_state(initial_state);
        return 0;
    }

    // `is_overflow_condition<uint64_t>`.
    let sign_bit: u64 = 1u64 << 63;
    let signed_max: u64 = sign_bit - 1;
    let overflow = flags & FL_OVERFLOW != 0
        || (flags & FL_SIGNED != 0
            && if flags & FL_NEGATIVE != 0 { number > sign_bit } else { number > signed_max });
    if overflow {
        // `ptd.get_errno().set(ERANGE)`: a plain errno write here has the same
        // net, user-visible effect (input.cpp's own validations are all
        // old-style already; see `invalid_parameter_old`).
        *_errno() = ERANGE;
        if flags & FL_SIGNED == 0 {
            return u64::MAX;
        }
        return if flags & FL_NEGATIVE != 0 { sign_bit } else { signed_max };
    }
    if flags & FL_NEGATIVE != 0 {
        number = number.wrapping_neg();
    }
    number
}

// ── floating-point scanning: boundary only, conversion via `crate::strtod` ───

fn is_digit_or_nondigit(c: u32) -> bool {
    matches!(c, 0x30..=0x39 | 0x61..=0x7A | 0x41..=0x5A | 0x5F)
}

unsafe fn radix_point_byte(locale: *mut CrtLocalePointers) -> u8 {
    **((*(*locale).locinfo).lconv as *const *const u8)
}

fn radix_point_matches<C: Char>(c: C, radix: u8) -> bool {
    if core::mem::size_of::<C>() == 1 {
        c.code() == radix as u32
    } else {
        c.code() as i32 == radix as i8 as i32
    }
}

unsafe fn match_word<C: Char, A: Adapter<C>>(word: &[u8], c: &mut C, source: &mut CharSource<C, A>) -> bool {
    for &w in word {
        let code = c.code();
        if code != w.to_ascii_uppercase() as u32 && code != w.to_ascii_lowercase() as u32 {
            return false;
        }
        *c = source.get();
    }
    true
}

/// The `restore_state` lambda every `parse_floating_point_*` helper closes
/// over: unget the lookahead character, then ask the source whether it can
/// actually rewind to `saved` -- which, for our source, it usually cannot once
/// more than one character has been consumed since. See `CharSource`'s doc.
unsafe fn restore<C: Char, A: Adapter<C>>(source: &mut CharSource<C, A>, c: &mut C, saved: u64) -> bool {
    source.unget(*c);
    *c = C::of(0);
    source.restore_state(saved)
}

/// `parse_floating_point_possible_infinity`, boolean-collapsed: `true` means
/// "a token was found" (plain "inf" or "infinity"), `false` means no token --
/// which, per the source's one-way latch, also covers a partially-matched,
/// then-abandoned "infXXX" that could not be pushed back.
unsafe fn scan_possible_infinity<C: Char, A: Adapter<C>>(
    c: &mut C,
    source: &mut CharSource<C, A>,
    mut stored_state: u64,
) -> bool {
    if !match_word(b"inf", c, source) {
        restore(source, c, stored_state);
        return false;
    }
    source.unget(*c);
    stored_state = source.save_state();
    *c = source.get();
    if !match_word(b"inity", c, source) {
        return restore(source, c, stored_state);
    }
    source.unget(*c);
    true
}

/// `parse_floating_point_possible_nan`, boolean-collapsed the same way.
unsafe fn scan_possible_nan<C: Char, A: Adapter<C>>(c: &mut C, source: &mut CharSource<C, A>, mut stored_state: u64) -> bool {
    if !match_word(b"nan", c, source) {
        restore(source, c, stored_state);
        return false;
    }
    source.unget(*c);
    stored_state = source.save_state();
    *c = source.get();
    if c.code() != b'(' as u32 {
        return restore(source, c, stored_state);
    }
    *c = source.get(); // past '('
    if match_word(b"snan)", c, source) {
        source.unget(*c);
        return true;
    }
    if match_word(b"ind)", c, source) {
        source.unget(*c);
        return true;
    }
    while c.code() != b')' as u32 && c.code() != 0 {
        if !is_digit_or_nondigit(c.code()) {
            return restore(source, c, stored_state);
        }
        *c = source.get();
    }
    if c.code() != b')' as u32 {
        return restore(source, c, stored_state);
    }
    true
}

/// `parse_floating_point_from_source`, boolean-collapsed: every accepted
/// character is already sitting in `source`'s recording by the time this
/// returns, so this only needs to trace the SAME control flow (for the exact
/// consumption boundary and the exact one-way-latch failure quirks) without
/// separately accumulating digits -- `crate::strtod::parse_floating_point`
/// redoes that, faithfully, on the recorded text.
unsafe fn scan_float_token<C: Char, A: Adapter<C>>(locale: *mut CrtLocalePointers, source: &mut CharSource<C, A>) -> bool {
    if !source.validate() {
        return false;
    }
    let mut stored_state = source.save_state();
    let mut c = source.get();

    while c.is_space(locale) {
        c = source.get();
    }

    if c.code() == b'-' as u32 || c.code() == b'+' as u32 {
        c = source.get();
    }

    if c.code() == b'I' as u32 || c.code() == b'i' as u32 {
        return scan_possible_infinity(&mut c, source, stored_state);
    }
    if c.code() == b'N' as u32 || c.code() == b'n' as u32 {
        return scan_possible_nan(&mut c, source, stored_state);
    }

    let mut is_hexadecimal = false;
    if c.code() == b'0' as u32 {
        let next_stored_state = source.save_state();
        let next_c = source.get();
        if next_c.code() == b'x' as u32 || next_c.code() == b'X' as u32 {
            is_hexadecimal = true;
            c = source.get();
            stored_state = next_stored_state;
        } else {
            source.unget(next_c);
        }
    }

    let max_digit_value: u32 = if is_hexadecimal { 0xF } else { 9 };
    let mut found_digits = false;

    while c.code() == b'0' as u32 {
        found_digits = true;
        c = source.get();
    }
    loop {
        let d = c.digit();
        if d > max_digit_value {
            break;
        }
        found_digits = true;
        c = source.get();
    }

    let radix_point = radix_point_byte(locale);
    if radix_point_matches(c, radix_point) {
        c = source.get();
        while c.code() == b'0' as u32 {
            found_digits = true;
            c = source.get();
        }
        loop {
            let d = c.digit();
            if d > max_digit_value {
                break;
            }
            found_digits = true;
            c = source.get();
        }
    }

    if !found_digits {
        if !restore(source, &mut c, stored_state) {
            return false;
        }
        // "0x" with nothing after it: the subject is "0", found.
        return is_hexadecimal;
    }

    source.unget(c);
    stored_state = source.save_state();
    c = source.get();

    let has_exponent = match c.code() as u8 {
        b'e' | b'E' => !is_hexadecimal,
        b'p' | b'P' => is_hexadecimal,
        _ => false,
    };
    if has_exponent {
        c = source.get();
        if c.code() == b'+' as u32 || c.code() == b'-' as u32 {
            c = source.get();
        }
        let mut has_exponent_digits = false;
        while c.code() == b'0' as u32 {
            has_exponent_digits = true;
            c = source.get();
        }
        loop {
            let d = c.digit();
            if d >= 10 {
                break;
            }
            has_exponent_digits = true;
            c = source.get();
        }
        if !has_exponent_digits {
            // A malformed exponent ("1e", "1e+") over a real stream/string
            // source: if more than the bare introducer was consumed (a sign
            // was present), the one-way latch means the WHOLE conversion fails
            // -- unlike `strtod("1e+")`, which can fully rewind and report
            // just "1". Deliberately NOT the same as `strtod`'s contract; see
            // the module doc.
            if restore(source, &mut c, stored_state) {
                c = source.get();
            } else {
                return false;
            }
        }
    }
    source.unget(c);
    true
}

/// Scans one floating-point token off `source`, then reparses the exact
/// recorded text through `crate::strtod::parse_floating_point` for the
/// correctly-rounded bit pattern. `None` means "no conversion" (`SLD_NODIGITS`
/// or a hard source failure) -- the caller must not write through the
/// receiving argument.
unsafe fn parse_floating_point_via_adapter<C: Char, A: Adapter<C>>(
    locale: *mut CrtLocalePointers,
    source: &mut CharSource<C, A>,
    t: &Traits,
) -> Option<u64> {
    let found = scan_float_token(locale, source);
    if !source.resolve_success() || !found {
        return None;
    }
    let (ptr, len) = source.recorded();
    let buf = _malloc_base((len + 1) * core::mem::size_of::<C>()) as *mut C;
    if buf.is_null() {
        invalid_parameter_old(ENOMEM);
        return None;
    }
    core::ptr::copy_nonoverlapping(ptr, buf, len);
    *buf.add(len) = C::of(0);

    let mut out: u64 = 0;
    let mut end: *const C = core::ptr::null();
    let status = strtod_parse_floating_point::<C>(locale, buf, &mut end, C::of(0), t, &mut out);
    _free_base(buf as *mut c_void);

    if status == FpStatus::NoDigits {
        return None;
    }
    Some(out)
}

// ── string/scanset writing: the four `write_character` overloads ────────────

unsafe fn reset_buffer<B: Char>(buffer: *mut B, buffer_count: usize) {
    if buffer_count == UNBOUNDED {
        return;
    }
    // `_RESET_STRING`: `*(_String) = 0;` plus `_FILL_STRING`, which is a no-op
    // outside a `_SECURECRT_FILL_BUFFER` (debug) build -- this crate builds
    // release semantics, so only the leading NUL write is live.
    if !buffer.is_null() {
        *buffer = B::of(0);
    }
}

unsafe fn fill_buffer<B: Char>(_buffer: *mut B, _buffer_count: usize, _buffer_remaining: usize) {
    // `_FILL_STRING` alone: a complete no-op outside a debug build.
}

/// The four `write_character` overloads, folded into one function generic over
/// both the adapter's character type `C` and the destination buffer's `B`; for
/// any one instantiation exactly one of the four branches is live (`C::WIDE`/
/// `B::WIDE` are both compile-time constants once `C` and `B` are fixed), the
/// others are dead but still type-check via plain pointer-width reinterpret
/// casts.
#[allow(clippy::too_many_arguments)]
unsafe fn write_one<C: Char, A: Adapter<C>, B: Char>(
    proc: &mut InputProcessor<C, A>,
    buffer: *mut B,
    buffer_count: usize,
    buffer_pointer: &mut *mut B,
    buffer_remaining: &mut usize,
    c: C,
) -> bool {
    if !C::WIDE && !B::WIDE {
        (*buffer_pointer as *mut u8).write_unaligned(c.code() as u8);
        *buffer_pointer = buffer_pointer.add(1);
        *buffer_remaining -= 1;
        true
    } else if C::WIDE && B::WIDE {
        (*buffer_pointer as *mut u16).write_unaligned(c.code() as u16);
        *buffer_pointer = buffer_pointer.add(1);
        *buffer_remaining -= 1;
        true
    } else if !C::WIDE && B::WIDE {
        // Narrow input character into a WIDE buffer. `_mbtowc_l` is called
        // ONLY to consume a possible DBCS trail byte from the input adapter
        // and validate the sequence; its converted value is then DISCARDED --
        // Microsoft's own quirk, kept exactly: the buffer receives the narrow
        // byte zero-extended, never the multibyte-converted character.
        let mut local = [c.code() as u8, 0u8];
        if isleadbyte(c.code() as c_int) != 0 {
            local[1] = proc.adapter.get() as u8;
        }
        let mut wide_temp: u16 = b'?' as u16;
        _mbtowc_l(&mut wide_temp, local.as_ptr(), mb_cur_max(proc.locale), proc.locale);
        let _ = wide_temp; // discarded, per the C -- see the comment above.
        (*buffer_pointer as *mut u16).write_unaligned(c.code() as u16);
        *buffer_pointer = buffer_pointer.add(1);
        *buffer_remaining -= 1;
        true
    } else {
        // WIDE input character into a narrow buffer: `wctomb_s`.
        if buffer_count == UNBOUNDED {
            let mut narrow_count: c_int = 0;
            if wctomb_s(&mut narrow_count, (*buffer_pointer).cast(), MB_LEN_MAX, c.code() as u16) == 0 {
                *buffer_pointer = (*buffer_pointer as *mut u8).add(narrow_count as usize).cast();
                *buffer_remaining -= narrow_count as usize;
            }
        } else {
            let mut narrow_count: c_int = 0;
            if wctomb_s(&mut narrow_count, (*buffer_pointer).cast(), *buffer_remaining, c.code() as u16) == ERANGE {
                reset_buffer(buffer, buffer_count);
                return false;
            }
            if narrow_count > 0 {
                *buffer_pointer = (*buffer_pointer as *mut u8).add(narrow_count as usize).cast();
                *buffer_remaining -= narrow_count as usize;
            }
        }
        true
    }
}

// ── the input processor ──────────────────────────────────────────────────────

pub(crate) struct InputProcessor<C: Char, A: Adapter<C>> {
    options: u64,
    adapter: A,
    format_parser: FormatParser<C>,
    locale: *mut CrtLocalePointers,
    va: Va,
    receiving_arguments_assigned: usize,
}

impl<C: Char, A: Adapter<C>> InputProcessor<C, A> {
    pub(crate) fn new(
        adapter: A,
        options: u64,
        format: *const C,
        locale: *mut CrtLocalePointers,
        arglist: *mut u8,
    ) -> Self {
        Self {
            options,
            adapter,
            format_parser: FormatParser::new(options, format),
            locale,
            va: Va(arglist),
            receiving_arguments_assigned: 0,
        }
    }

    fn secure_buffers(&self) -> bool {
        self.options & super::SCANF_SECURECRT != 0
    }

    pub(crate) unsafe fn process(&mut self) -> c_int {
        if !self.adapter.validate() {
            return EOF;
        }
        if !self.format_parser.validate() {
            return EOF;
        }

        while self.format_parser.advance() {
            if !self.process_state() {
                break;
            }
        }

        let mut result = self.receiving_arguments_assigned as c_int;
        if self.receiving_arguments_assigned == 0 && self.format_parser.kind() != DirectiveKind::EndOfString {
            let c = self.adapter.get();
            if c == C::EOF_CODE {
                result = EOF;
            }
            self.adapter.unget(c);
        }

        if self.secure_buffers() {
            let err = self.format_parser.error_code();
            if err != 0 {
                invalid_parameter_old(err);
                return result;
            }
        }
        result
    }

    unsafe fn process_state(&mut self) -> bool {
        match self.format_parser.kind() {
            DirectiveKind::Whitespace => self.process_whitespace(),
            DirectiveKind::LiteralCharacter => self.process_literal_character(),
            DirectiveKind::ConversionSpecifier => {
                let result = self.process_conversion_specifier();
                // `%n` and a suppressed conversion are not "receiving arguments".
                if result && self.format_parser.mode() != Mode::ReportCharacterCount && !self.format_parser.suppress_assignment() {
                    self.receiving_arguments_assigned += 1;
                }
                result
            }
            _ => false,
        }
    }

    unsafe fn skip_whitespace(&mut self) -> i32 {
        loop {
            let c = self.adapter.get();
            if c == C::EOF_CODE {
                return c;
            }
            if !C::of(c as u32).is_space(self.locale) {
                return c;
            }
        }
    }

    unsafe fn process_whitespace(&mut self) -> bool {
        let c = self.skip_whitespace();
        self.adapter.unget(c);
        true
    }

    unsafe fn process_literal_character(&mut self) -> bool {
        let c = self.adapter.get();
        if c == C::EOF_CODE {
            return false;
        }
        if c as u32 != self.format_parser.literal_lead().code() {
            self.adapter.unget(c);
            return false;
        }
        if C::WIDE {
            return true; // No-op for Unicode.
        }
        if isleadbyte(c) == 0 {
            return true;
        }
        let trail = self.adapter.get();
        if trail as u32 != self.format_parser.literal_trail().code() {
            self.adapter.unget(trail);
            self.adapter.unget(c);
            return false;
        }
        true
    }

    unsafe fn process_conversion_specifier(&mut self) -> bool {
        match self.format_parser.mode() {
            Mode::Character => self.process_string_specifier(Mode::Character),
            Mode::String => self.process_string_specifier(Mode::String),
            Mode::Scanset => self.process_string_specifier(Mode::Scanset),
            Mode::SignedUnknown => self.process_integer_specifier(0, true),
            Mode::SignedDecimal => self.process_integer_specifier(10, true),
            Mode::UnsignedOctal => self.process_integer_specifier(8, false),
            Mode::UnsignedDecimal => self.process_integer_specifier(10, false),
            Mode::UnsignedHexadecimal => self.process_integer_specifier(16, false),
            Mode::FloatingPoint => self.process_floating_point_specifier(),
            Mode::ReportCharacterCount => self.process_character_count_specifier(),
        }
    }

    unsafe fn process_string_specifier(&mut self, mode: Mode) -> bool {
        if mode == Mode::String {
            self.process_whitespace();
        }
        match self.format_parser.length() {
            1 => self.process_string_specifier_tchar::<u8>(mode),
            2 => self.process_string_specifier_tchar::<u16>(mode),
            _ => false,
        }
    }

    unsafe fn is_character_allowed_in_string(&mut self, mode: Mode, c: i32) -> bool {
        if c == C::EOF_CODE {
            return false;
        }
        match mode {
            Mode::Character => true,
            Mode::String => {
                if (0x09..=0x0D).contains(&c) {
                    return false;
                }
                if c == 0x20 {
                    return false;
                }
                true
            }
            Mode::Scanset => self.format_parser.scanset().test(c as u32),
            _ => false,
        }
    }

    unsafe fn process_string_specifier_tchar<B: Char>(&mut self, mode: Mode) -> bool {
        let mut buffer: *mut B = core::ptr::null_mut();
        if !self.format_parser.suppress_assignment() {
            buffer = self.va.read::<*mut B>();
            if buffer.is_null() {
                invalid_parameter_old(EINVAL);
                return false;
            }
        }
        let buffer_count: usize =
            if !buffer.is_null() && self.secure_buffers() { self.va.read::<c_uint>() as usize } else { UNBOUNDED };

        if buffer_count == 0 {
            if self.options & super::SCANF_LEGACY_MSVCRT_COMPATIBILITY != 0 {
                // Legacy quirk: fails to unget the last character read, and
                // still writes a terminator despite the caller saying the
                // buffer has zero elements.
                self.adapter.get();
                if !buffer.is_null() {
                    *buffer = B::of(0);
                }
            }
            *_errno() = ENOMEM;
            return false;
        }

        let width = self.format_parser.width();
        let mut buffer_pointer = buffer;
        let mut buffer_remaining = buffer_count;
        if mode != Mode::Character && buffer_remaining != UNBOUNDED {
            buffer_remaining -= 1; // room for the terminator.
        }

        let mut width_consumed: u64 = 0;
        loop {
            if width != 0 && width_consumed == width {
                break;
            }
            let c_code = self.adapter.get();
            if !self.is_character_allowed_in_string(mode, c_code) {
                self.adapter.unget(c_code);
                break;
            }
            if self.format_parser.suppress_assignment() {
                width_consumed += 1;
                continue;
            }
            if buffer_remaining == 0 {
                reset_buffer(buffer, buffer_count);
                *_errno() = ENOMEM;
                return false;
            }
            if !write_one::<C, A, B>(self, buffer, buffer_count, &mut buffer_pointer, &mut buffer_remaining, C::of(c_code as u32)) {
                break;
            }
            width_consumed += 1;
        }

        if width_consumed == 0 {
            return false;
        }

        // `%c` matches EXACTLY the requested width unless legacy mode allows
        // "up to" that many.
        if mode == Mode::Character
            && width_consumed != width
            && self.options & super::SCANF_LEGACY_MSVCRT_COMPATIBILITY == 0
        {
            return false;
        }

        if self.format_parser.suppress_assignment() {
            return true;
        }
        if mode != Mode::Character {
            *buffer_pointer = B::of(0);
            fill_buffer(buffer, buffer_count, buffer_remaining);
        }
        true
    }

    unsafe fn process_integer_specifier(&mut self, base: i32, is_signed: bool) -> bool {
        self.process_whitespace();
        let width = self.format_parser.width();
        let number = {
            let mut source = CharSource::new(&mut self.adapter, width);
            let number = parse_integer(self.locale, &mut source, base, is_signed);
            if !source.resolve_success() {
                return false;
            }
            number
        };
        if self.format_parser.suppress_assignment() {
            return true;
        }
        self.write_integer(number)
    }

    unsafe fn process_floating_point_specifier(&mut self) -> bool {
        self.process_whitespace();
        match self.format_parser.length() {
            4 => self.process_floating_point_specifier_t(&FLOAT, false),
            8 => self.process_floating_point_specifier_t(&DOUBLE, true),
            _ => false,
        }
    }

    unsafe fn process_floating_point_specifier_t(&mut self, t: &Traits, is_double: bool) -> bool {
        let width = self.format_parser.width();
        let bits = {
            let mut source = CharSource::new(&mut self.adapter, width);
            match parse_floating_point_via_adapter(self.locale, &mut source, t) {
                Some(bits) => bits,
                None => return false,
            }
        };
        if self.format_parser.suppress_assignment() {
            return true;
        }
        self.write_floating_point(bits, is_double)
    }

    unsafe fn process_character_count_specifier(&mut self) -> bool {
        if self.format_parser.suppress_assignment() {
            return true;
        }
        self.write_integer(self.adapter.characters_read())
    }

    unsafe fn write_integer(&mut self, value: u64) -> bool {
        let result_pointer = self.va.read::<*mut c_void>();
        if result_pointer.is_null() {
            invalid_parameter_old(EINVAL);
            return false;
        }
        match to_integer_length(self.format_parser.length_kind()) {
            1 => (result_pointer as *mut u8).write_unaligned(value as u8),
            2 => (result_pointer as *mut u16).write_unaligned(value as u16),
            4 => (result_pointer as *mut u32).write_unaligned(value as u32),
            8 => (result_pointer as *mut u64).write_unaligned(value as u64),
            _ => {
                invalid_parameter_old(EINVAL);
                return false;
            }
        }
        true
    }

    unsafe fn write_floating_point(&mut self, bits: u64, is_double: bool) -> bool {
        let result_pointer = self.va.read::<*mut c_void>();
        if result_pointer.is_null() {
            invalid_parameter_old(EINVAL);
            return false;
        }
        if is_double {
            (result_pointer as *mut u64).write_unaligned(bits);
        } else {
            (result_pointer as *mut u32).write_unaligned(bits as u32);
        }
        true
    }
}
