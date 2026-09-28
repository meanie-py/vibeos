//! `__crt_stdio_input::format_string_parser` and its scanset bitmap
//! (corecrt_internal_stdio_input.h:340-1049), ported directive for directive.
//!
//! The format string is always a real, NUL-terminated C string (unlike the
//! *input* being scanned), so this walks a raw `*const C` freely -- there is no
//! character-source problem here, unlike `engine.rs`.

use core::ffi::{c_int, c_void};

use super::{invalid_parameter_old, Char};

const EINVAL: c_int = 22;
const EILSEQ: c_int = 42;
const ENOMEM: c_int = 12;

extern "C" {
    fn calloc(count: usize, size: usize) -> *mut c_void;
    fn free(p: *mut c_void);
    fn isleadbyte(c: c_int) -> c_int;
    // Plain (current-locale) `isspace`/`iswspace` -- `__acrt_stdio_char_traits::istspace`,
    // NOT the explicit-locale `_isspace_l`/`iswspace` the integer/float paths use.
    fn isspace(c: c_int) -> c_int;
    fn iswspace(c: u16) -> c_int;
}

unsafe fn istspace<C: Char>(c: C) -> bool {
    if C::WIDE {
        iswspace(c.code() as u16) != 0
    } else {
        isspace(c.code() as c_int) != 0
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DirectiveKind {
    UnknownError,
    EndOfString,
    Whitespace,
    LiteralCharacter,
    ConversionSpecifier,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Length {
    None,
    Hh,
    H,
    L,
    Ll,
    J,
    Z,
    T,
    BigL,
    I32,
    I64,
    BigT,
}

/// `to_integer_length`. Only reached for a mode/length pair `is_length_valid`
/// allows; the other lengths are unreachable there, as in the C.
pub(crate) fn to_integer_length(length: Length) -> usize {
    match length {
        Length::None => core::mem::size_of::<c_int>(),
        Length::Hh => 1,
        Length::H => 2,
        Length::L => 4, // `long` is 32 bits on every Windows target.
        Length::Ll => 8,
        Length::J => 8,  // intmax_t
        Length::Z => core::mem::size_of::<usize>(),
        Length::T => core::mem::size_of::<usize>(), // ptrdiff_t: same width as size_t here.
        Length::I32 => 4,
        Length::I64 => 8,
        Length::BigL | Length::BigT => 0,
    }
}

/// `to_floating_point_length`.
pub(crate) fn to_floating_point_length(length: Length) -> usize {
    match length {
        Length::None => 4,       // float
        Length::L => 8,          // double
        Length::BigL => 8,       // long double == double here
        _ => 0,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    Character,
    String,
    SignedUnknown,
    SignedDecimal,
    UnsignedOctal,
    UnsignedDecimal,
    UnsignedHexadecimal,
    FloatingPoint,
    Scanset,
    ReportCharacterCount,
}

/// `is_length_valid`'s table, transcribed row-for-row in `Mode` declaration
/// order (columns: none,hh,h,l,ll,j,z,t,L,I32,I64,T). Two of the C's row
/// *comments* ("signed decimal" before "signed unknown") are swapped from
/// enum-declaration order, but every row in that block is identical anyway, so
/// it does not matter which label goes with which index.
#[rustfmt::skip]
const LENGTH_VALID: [[bool; 12]; 10] = [
    /* character              */ [true, false, true, true, true, false, false, false, true, false, false, true],
    /* string                 */ [true, false, true, true, true, false, false, false, true, false, false, true],
    /* signed_unknown         */ [true, true, true, true, true, true, true, true, false, true, true, false],
    /* signed_decimal         */ [true, true, true, true, true, true, true, true, false, true, true, false],
    /* unsigned_octal         */ [true, true, true, true, true, true, true, true, false, true, true, false],
    /* unsigned_decimal       */ [true, true, true, true, true, true, true, true, false, true, true, false],
    /* unsigned_hexadecimal   */ [true, true, true, true, true, true, true, true, false, true, true, false],
    /* floating_point         */ [true, false, false, true, false, false, false, false, true, false, false, false],
    /* scanset                */ [true, false, false, true, false, false, false, false, false, false, false, true],
    /* report_character_count */ [true, true, true, true, true, true, true, true, false, true, true, false],
];

fn length_index(length: Length) -> usize {
    match length {
        Length::None => 0,
        Length::Hh => 1,
        Length::H => 2,
        Length::L => 3,
        Length::Ll => 4,
        Length::J => 5,
        Length::Z => 6,
        Length::T => 7,
        Length::BigL => 8,
        Length::I32 => 9,
        Length::I64 => 10,
        Length::BigT => 11,
    }
}

fn mode_index(mode: Mode) -> usize {
    match mode {
        Mode::Character => 0,
        Mode::String => 1,
        Mode::SignedUnknown => 2,
        Mode::SignedDecimal => 3,
        Mode::UnsignedOctal => 4,
        Mode::UnsignedDecimal => 5,
        Mode::UnsignedHexadecimal => 6,
        Mode::FloatingPoint => 7,
        Mode::Scanset => 8,
        Mode::ReportCharacterCount => 9,
    }
}

fn is_length_valid(mode: Mode, length: Length) -> bool {
    LENGTH_VALID[mode_index(mode)][length_index(length)]
}

// ── the scanset bitmap ────────────────────────────────────────────────────────

const NARROW_SCANSET_BYTES: usize = 256 / 8;
/// `(WCHAR_MAX + 1) / CHAR_BIT`. `wchar_t` is unsigned here (`WCHAR_MAX == 0xFFFF`).
const WIDE_SCANSET_BYTES: usize = 65536 / 8;

/// `scanset_buffer<UnsignedCharacter>`, over `scanset_storage`: a plain member
/// array for `char` (never fails), a lazily `calloc`'d one for `wchar_t` (can
/// fail -- `is_usable()` reports that, `scan_scanset_range` turns it into
/// `ENOMEM`), freed on drop like the C++'s `__crt_unique_heap_ptr`. One
/// allocation is reused for every `%[...]` the SAME format string parser sees.
pub(crate) struct Scanset<C> {
    narrow: [u8; NARROW_SCANSET_BYTES],
    wide: *mut u8,
    _marker: core::marker::PhantomData<C>,
}

impl<C: Char> Scanset<C> {
    const fn new() -> Self {
        Self { narrow: [0; NARROW_SCANSET_BYTES], wide: core::ptr::null_mut(), _marker: core::marker::PhantomData }
    }

    fn size(&self) -> usize {
        if C::WIDE {
            WIDE_SCANSET_BYTES
        } else {
            NARROW_SCANSET_BYTES
        }
    }

    unsafe fn data(&mut self) -> *mut u8 {
        if !C::WIDE {
            return self.narrow.as_mut_ptr();
        }
        if self.wide.is_null() {
            self.wide = calloc(1, WIDE_SCANSET_BYTES).cast();
        }
        self.wide
    }

    pub(crate) unsafe fn is_usable(&mut self) -> bool {
        !self.data().is_null()
    }

    pub(crate) unsafe fn set(&mut self, c: u32) {
        let d = self.data();
        *d.add((c / 8) as usize) |= 1 << (c % 8);
    }

    pub(crate) unsafe fn test(&mut self, c: u32) -> bool {
        (*self.data().add((c / 8) as usize) & (1 << (c % 8))) != 0
    }

    pub(crate) unsafe fn reset(&mut self) {
        let n = self.size();
        let d = self.data();
        if !d.is_null() {
            core::ptr::write_bytes(d, 0, n);
        }
    }

    pub(crate) unsafe fn invert(&mut self) {
        let n = self.size();
        let d = self.data();
        for i in 0..n {
            *d.add(i) ^= 0xFF;
        }
    }
}

impl<C> Drop for Scanset<C> {
    fn drop(&mut self) {
        unsafe {
            if !self.wide.is_null() {
                free(self.wide.cast());
            }
        }
    }
}

// ── the parser itself ────────────────────────────────────────────────────────

pub(crate) struct FormatParser<C: Char> {
    #[allow(dead_code)]
    options: u64,
    format_it: *const C,
    error_code: c_int,

    kind: DirectiveKind,
    literal_lead: C,
    literal_trail: C,
    suppress_assignment: bool,
    width: u64,
    length: Length,
    is_wide: bool,
    mode: Mode,
    scanset: Scanset<C>,
}

impl<C: Char> FormatParser<C> {
    pub(crate) fn new(options: u64, format: *const C) -> Self {
        Self {
            options,
            format_it: format,
            error_code: 0,
            kind: DirectiveKind::UnknownError,
            literal_lead: C::of(0),
            literal_trail: C::of(0),
            suppress_assignment: false,
            width: 0,
            length: Length::None,
            is_wide: false,
            mode: Mode::Character,
            scanset: Scanset::new(),
        }
    }

    pub(crate) unsafe fn validate(&self) -> bool {
        if self.format_it.is_null() {
            invalid_parameter_old(EINVAL);
            return false;
        }
        true
    }

    pub(crate) fn error_code(&self) -> c_int {
        self.error_code
    }
    pub(crate) fn kind(&self) -> DirectiveKind {
        self.kind
    }
    pub(crate) fn literal_lead(&self) -> C {
        self.literal_lead
    }
    pub(crate) fn literal_trail(&self) -> C {
        self.literal_trail
    }
    pub(crate) fn suppress_assignment(&self) -> bool {
        self.suppress_assignment
    }
    pub(crate) fn width(&self) -> u64 {
        self.width
    }
    pub(crate) fn mode(&self) -> Mode {
        self.mode
    }
    pub(crate) fn length_kind(&self) -> Length {
        self.length
    }
    /// `format_string_parser::length()`.
    pub(crate) fn length(&self) -> usize {
        match self.mode {
            Mode::Character | Mode::String | Mode::Scanset => {
                if self.is_wide {
                    core::mem::size_of::<u16>()
                } else {
                    core::mem::size_of::<u8>()
                }
            }
            Mode::SignedUnknown
            | Mode::SignedDecimal
            | Mode::UnsignedOctal
            | Mode::UnsignedDecimal
            | Mode::UnsignedHexadecimal
            | Mode::ReportCharacterCount => to_integer_length(self.length),
            Mode::FloatingPoint => to_floating_point_length(self.length),
        }
    }
    pub(crate) unsafe fn scanset(&mut self) -> &mut Scanset<C> {
        &mut self.scanset
    }

    fn reset_token_state(&mut self) {
        self.kind = DirectiveKind::UnknownError;
        self.literal_lead = C::of(0);
        self.literal_trail = C::of(0);
        self.suppress_assignment = false;
        self.width = 0;
        self.length = Length::None;
        self.is_wide = false;
        self.mode = Mode::Character;
    }

    fn reset_token_state_for_error(&mut self, error_code: c_int) {
        self.reset_token_state();
        self.error_code = error_code;
    }

    unsafe fn peek(&self, offset: isize) -> C {
        *self.format_it.offset(offset)
    }

    pub(crate) unsafe fn advance(&mut self) -> bool {
        if self.error_code != 0 {
            return false;
        }
        self.reset_token_state();

        if self.peek(0).code() == 0 {
            self.kind = DirectiveKind::EndOfString;
            return false;
        }

        if istspace(self.peek(0)) {
            self.kind = DirectiveKind::Whitespace;
            while istspace(self.peek(0)) {
                self.format_it = self.format_it.add(1);
            }
            return true;
        }

        if self.peek(0).code() != b'%' as u32 || self.peek(1).code() == b'%' as u32 {
            self.kind = DirectiveKind::LiteralCharacter;
            self.literal_lead = self.peek(0);
            self.format_it = self.format_it.add(if self.peek(0).code() != b'%' as u32 { 1 } else { 2 });
            return self.scan_optional_literal_trail();
        }

        self.kind = DirectiveKind::ConversionSpecifier;
        self.format_it = self.format_it.add(1); // past '%'

        self.scan_optional_assignment_suppressor();

        if !self.scan_optional_field_width() {
            return false;
        }
        self.scan_optional_length_modifier();
        self.scan_optional_wide_modifier();

        if !self.scan_conversion_specifier() {
            return false;
        }

        if !is_length_valid(self.mode, self.length) {
            self.reset_token_state_for_error(EINVAL);
            return false;
        }
        true
    }

    /// Narrow only has DBCS trail bytes; wide is a no-op (`scan_optional_literal_character_trail_bytes_tchar(wchar_t)`).
    unsafe fn scan_optional_literal_trail(&mut self) -> bool {
        if C::WIDE {
            return true;
        }
        if isleadbyte(self.literal_lead.code() as c_int) == 0 {
            return true;
        }
        if self.peek(0).code() == 0 {
            self.reset_token_state_for_error(EILSEQ);
            return false;
        }
        self.literal_trail = self.peek(0);
        self.format_it = self.format_it.add(1);
        true
    }

    unsafe fn scan_optional_assignment_suppressor(&mut self) {
        if self.peek(0).code() == b'*' as u32 {
            self.suppress_assignment = true;
            self.format_it = self.format_it.add(1);
        }
    }

    /// `scan_optional_field_width`: `traits::tcstoull`, reused via
    /// `crate::strtox::parse` -- the format string is a real C string, so the
    /// pointer-walking form is safe here (unlike the input being scanned).
    unsafe fn scan_optional_field_width(&mut self) -> bool {
        if self.peek(0).digit() <= 9 {
            // Fall through: there IS a leading digit.
        } else {
            return true;
        }
        let mut width_end: *const C = core::ptr::null();
        let width = crate::strtox::parse::<C>(
            self.format_it,
            &mut width_end,
            10,
            core::ptr::null_mut(),
            64,
            false,
            true,
        );
        if width == 0 || width_end == self.format_it {
            self.reset_token_state_for_error(EINVAL);
            return false;
        }
        self.width = width;
        self.format_it = width_end;
        true
    }

    unsafe fn scan_optional_length_modifier(&mut self) {
        match self.peek(0).code() as u8 {
            b'h' => {
                if self.peek(1).code() == b'h' as u32 {
                    self.format_it = self.format_it.add(2);
                    self.length = Length::Hh;
                } else {
                    self.format_it = self.format_it.add(1);
                    self.length = Length::H;
                }
            }
            b'I' => {
                let a = self.peek(1).code();
                if a == b'3' as u32 && self.peek(2).code() == b'2' as u32 {
                    self.format_it = self.format_it.add(3);
                    self.length = Length::I32;
                } else if a == b'6' as u32 && self.peek(2).code() == b'4' as u32 {
                    self.format_it = self.format_it.add(3);
                    self.length = Length::I64;
                } else if matches!(a as u8, b'd' | b'i' | b'o' | b'u' | b'x' | b'X') {
                    self.format_it = self.format_it.add(1);
                    self.length = if core::mem::size_of::<usize>() == 4 { Length::I32 } else { Length::I64 };
                }
            }
            b'l' => {
                if self.peek(1).code() == b'l' as u32 {
                    self.format_it = self.format_it.add(2);
                    self.length = Length::Ll;
                } else {
                    self.format_it = self.format_it.add(1);
                    self.length = Length::L;
                }
            }
            b'L' => {
                self.format_it = self.format_it.add(1);
                self.length = Length::BigL;
            }
            b'j' => {
                self.format_it = self.format_it.add(1);
                self.length = Length::J;
            }
            b't' => {
                self.format_it = self.format_it.add(1);
                self.length = Length::T;
            }
            b'z' => {
                self.format_it = self.format_it.add(1);
                self.length = Length::Z;
            }
            b'T' => {
                // The Microsoft TCHAR-generic length modifier (distinct from
                // lowercase `t` = `ptrdiff_t`): defaults `%T` to wide, per
                // `should_default_to_wide(wchar_t)`.
                self.format_it = self.format_it.add(1);
                self.length = Length::BigT;
            }
            _ => {}
        }
    }

    unsafe fn scan_optional_wide_modifier(&mut self) {
        if self.peek(0).code() == b'w' as u32 {
            self.format_it = self.format_it.add(1);
            self.is_wide = true;
            return;
        }
        let c = self.peek(0).code();
        if !C::WIDE {
            if c == b'C' as u32 || c == b'S' as u32 {
                self.is_wide = true;
            }
        } else {
            if c == b'C' as u32 || c == b'S' as u32 {
                return;
            }
            if self.length == Length::T {
                self.is_wide = true;
                return;
            }
            if self.options & super::SCANF_LEGACY_WIDE_SPECIFIERS != 0 {
                self.is_wide = true;
            }
        }
    }

    fn set_wide_for_c_s_or_scanset(&mut self) {
        if self.length == Length::H {
            self.is_wide = false;
        }
        if matches!(self.length, Length::L | Length::Ll | Length::BigL) {
            self.is_wide = true;
        }
    }

    unsafe fn scan_conversion_specifier(&mut self) -> bool {
        match self.peek(0).code() as u8 {
            b'C' | b'c' => {
                if self.width == 0 {
                    self.width = 1;
                }
                self.set_wide_for_c_s_or_scanset();
                self.mode = Mode::Character;
                self.format_it = self.format_it.add(1);
                true
            }
            b'S' | b's' => {
                self.set_wide_for_c_s_or_scanset();
                self.mode = Mode::String;
                self.format_it = self.format_it.add(1);
                true
            }
            b'i' | b'I' => {
                self.mode = Mode::SignedUnknown;
                self.format_it = self.format_it.add(1);
                true
            }
            b'd' => {
                self.mode = Mode::SignedDecimal;
                self.format_it = self.format_it.add(1);
                true
            }
            b'u' => {
                self.mode = Mode::UnsignedDecimal;
                self.format_it = self.format_it.add(1);
                true
            }
            b'X' | b'x' => {
                self.mode = Mode::UnsignedHexadecimal;
                self.format_it = self.format_it.add(1);
                true
            }
            b'p' => {
                self.length = if core::mem::size_of::<usize>() == 4 { Length::I32 } else { Length::I64 };
                self.mode = Mode::UnsignedHexadecimal;
                self.format_it = self.format_it.add(1);
                true
            }
            b'o' => {
                self.mode = Mode::UnsignedOctal;
                self.format_it = self.format_it.add(1);
                true
            }
            b'A' | b'a' | b'E' | b'e' | b'F' | b'f' | b'G' | b'g' => {
                self.mode = Mode::FloatingPoint;
                self.format_it = self.format_it.add(1);
                true
            }
            b'[' => {
                self.set_wide_for_c_s_or_scanset();
                self.mode = Mode::Scanset;
                self.format_it = self.format_it.add(1);
                self.scan_scanset_range()
            }
            b'n' => {
                self.mode = Mode::ReportCharacterCount;
                self.format_it = self.format_it.add(1);
                true
            }
            _ => {
                self.reset_token_state_for_error(EINVAL);
                false
            }
        }
    }

    unsafe fn scan_scanset_range(&mut self) -> bool {
        if !self.scanset.is_usable() {
            self.reset_token_state_for_error(ENOMEM);
            return false;
        }
        self.scanset.reset();

        let is_reject_set = self.peek(0).code() == b'^' as u32;
        if is_reject_set {
            self.format_it = self.format_it.add(1);
        }
        if self.peek(0).code() == b']' as u32 {
            self.format_it = self.format_it.add(1);
            self.scanset.set(b']' as u32);
        }

        let first = self.format_it;
        let mut last_range_end: *const C = core::ptr::null();
        while self.peek(0).code() != b']' as u32 && self.peek(0).code() != 0 {
            let cur = self.format_it;
            if self.peek(0).code() != b'-' as u32
                || cur.sub(1) == last_range_end
                || cur == first
                || self.peek(1).code() == b']' as u32
            {
                self.scanset.set(self.peek(0).code());
            } else {
                let mut lower = self.peek(-1).code();
                let mut upper = self.peek(1).code();
                last_range_end = self.format_it.add(1);
                if lower > upper {
                    core::mem::swap(&mut lower, &mut upper);
                }
                upper += 1;
                let mut c = lower;
                while c != upper {
                    self.scanset.set(c);
                    c += 1;
                }
            }
            self.format_it = self.format_it.add(1);
        }

        if self.peek(0).code() == 0 {
            self.reset_token_state_for_error(EINVAL);
            return false;
        }
        if is_reject_set {
            self.scanset.invert();
        }
        self.format_it = self.format_it.add(1); // past ']'
        true
    }
}
