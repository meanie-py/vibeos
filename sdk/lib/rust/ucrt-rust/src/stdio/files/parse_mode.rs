//! `__acrt_stdio_parse_mode` (corecrt_internal_stdio.h): the fopen-family mode
//! string parser, header-inline in the C (a function template over
//! `Character`), ported once here and used by [`super::open`]'s `fopen`,
//! `freopen`, `_fdopen` and `_openfile`/`_wopenfile` families.
//!
//! Generic over char width via [`ModeCursor`], implemented for `*const c_char`
//! (read as an unsigned byte, matching `path_requires_backslash_narrow`'s
//! style in `stdio/mod.rs`) and `*const u16`. A mode string is pure ASCII by
//! construction: every switch/comparison in the C is against an ASCII literal,
//! so a wide character above `0x7F` can never match anything and always falls
//! to the same "invalid mode" arm a mismatched ASCII character would.
//!
//! Details that are silent if wrong:
//! * On a duplicate/conflicting flag (e.g. two `+`, or `S` after `R`), the C's
//!   `for` loop just stops (its `continue_loop` becomes false without
//!   advancing `it`), and it is the *post-loop* `_VALIDATE_RETURN(*it=='\0',
//!   ...)` that actually sets `EINVAL` and invokes the handler -- `it` is still
//!   sitting on the offending, non-NUL character. This port invokes the
//!   handler immediately at the point of failure instead: same errno, same
//!   handler call, same `_success == false` result, one fewer indirection.
//! * `_commode` seeds `_stdio_mode` **before** the first character is even
//!   read, and is silently discarded by `case 'r'` and kept as extra bits by
//!   `case 'w'`/`case 'a'` (only `_IOCOMMIT` survives from it, since 'w'/'a'
//!   set `_stdio_mode` to a plain assignment that overwrites `_IOREAD` but the
//!   initial value already had `_IOCOMMIT` or'able bits -- actually the C
//!   *assigns*, not or's, `result._stdio_mode = _IOREAD` etc., so `_commode`
//!   only matters if the first character is invalid, in which case it is
//!   irrelevant anyway). Ported as a plain assignment, matching the C exactly.
//! * `x` is only accepted if `_O_TRUNC` is already set, i.e. only after `w`
//!   (never after `r` or `a`) and regardless of where else it appears in the
//!   suffix.
//! * The `ccs=` encoding comparison for `"ccs"` itself is case-SENSITIVE
//!   (`tcsncmp`); the encoding name (`UTF-8`/`UTF-16LE`/`UNICODE`) is
//!   case-INSENSITIVE (`tcsnicmp`), checked in that order.

use core::ffi::{c_char, c_int, c_long};

use crate::stdio_layout::flags::{STREAM_READ, STREAM_UPDATE, STREAM_WRITE};

// fcntl.h. Same values as lowio_open.rs's private copies (measured from the
// same header); duplicated here per this crate's convention of each
// consuming file redeclaring the constants and externs it needs (see
// lowio_open.rs's own `handle` module doc for the precedent).
const O_RDONLY: c_int = 0x0000;
const O_WRONLY: c_int = 0x0001;
const O_RDWR: c_int = 0x0002;
const O_APPEND: c_int = 0x0008;
const O_RANDOM: c_int = 0x0010;
const O_SEQUENTIAL: c_int = 0x0020;
const O_TEMPORARY: c_int = 0x0040;
const O_NOINHERIT: c_int = 0x0080;
const O_CREAT: c_int = 0x0100;
const O_TRUNC: c_int = 0x0200;
const O_EXCL: c_int = 0x0400;
const O_SHORT_LIVED: c_int = 0x1000;
const O_TEXT: c_int = 0x4000;
const O_BINARY: c_int = 0x8000;
const O_WTEXT: c_int = 0x10000;
const O_U16TEXT: c_int = 0x20000;
const O_U8TEXT: c_int = 0x40000;

/// `_IOCOMMIT` (corecrt_internal_stdio.h's stream-flags enum).
const STREAM_COMMIT: c_long = 0x0800;

const EINVAL: c_int = 22;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
}

/// `__acrt_stdio_stream_mode`.
#[derive(Clone, Copy)]
pub(super) struct StreamMode {
    pub lowio_mode: c_int,
    pub stdio_mode: c_long,
    pub success: bool,
}

/// `__acrt_stdio_char_traits<Character>`, the slice this parser needs: a
/// NUL-terminated cursor that can report its current character as ASCII (or
/// `None` for anything that cannot possibly be a mode letter) and advance.
pub(super) trait ModeCursor: Copy {
    unsafe fn ascii(self) -> Option<u8>;
    unsafe fn advance(self) -> Self;
    fn is_null(self) -> bool;
}

impl ModeCursor for *const c_char {
    unsafe fn ascii(self) -> Option<u8> {
        Some(*self as u8)
    }
    unsafe fn advance(self) -> Self {
        self.add(1)
    }
    fn is_null(self) -> bool {
        <*const c_char>::is_null(self)
    }
}

impl ModeCursor for *const u16 {
    unsafe fn ascii(self) -> Option<u8> {
        let v = *self;
        if v <= 0x7F {
            Some(v as u8)
        } else {
            None
        }
    }
    unsafe fn advance(self) -> Self {
        self.add(1)
    }
    fn is_null(self) -> bool {
        <*const u16>::is_null(self)
    }
}

unsafe fn skip_spaces<P: ModeCursor>(mut it: P) -> P {
    while it.ascii() == Some(b' ') {
        it = it.advance();
    }
    it
}

unsafe fn advance_n<P: ModeCursor>(mut it: P, n: usize) -> P {
    for _ in 0..n {
        it = it.advance();
    }
    it
}

/// `tcsncmp(it, lit, lit.len()) == 0`: exact, case-sensitive, byte-for-byte
/// (matches the C even if `it` is shorter than `lit` -- the comparison simply
/// fails at the position of `it`'s NUL, which is a readable byte).
unsafe fn matches_exact<P: ModeCursor>(mut it: P, lit: &[u8]) -> bool {
    for &want in lit {
        if it.ascii() != Some(want) {
            return false;
        }
        it = it.advance();
    }
    true
}

/// `tcsnicmp(it, lit, lit.len()) == 0`: ASCII case-insensitive (the C's
/// `_strnicmp`/`_wcsnicmp` fold only the ASCII range for these literals).
unsafe fn matches_ci<P: ModeCursor>(mut it: P, lit: &[u8]) -> bool {
    for &want in lit {
        match it.ascii() {
            Some(c) if c.to_ascii_lowercase() == want.to_ascii_lowercase() => {}
            _ => return false,
        }
        it = it.advance();
    }
    true
}

unsafe fn invalid(result: StreamMode) -> StreamMode {
    *_errno() = EINVAL;
    _invalid_parameter_noinfo();
    result
}

// ── the seven/eight optional-flag helpers ───────────────────────────────────

fn parse_plus(result: &mut StreamMode, seen_plus: &mut bool) -> bool {
    if *seen_plus {
        return false;
    }
    *seen_plus = true;
    if result.lowio_mode & O_RDWR != 0 {
        return false;
    }
    result.lowio_mode |= O_RDWR;
    result.lowio_mode &= !(O_RDONLY | O_WRONLY);
    result.stdio_mode |= STREAM_UPDATE;
    result.stdio_mode &= !(STREAM_READ | STREAM_WRITE);
    true
}

fn parse_b(result: &mut StreamMode) -> bool {
    if result.lowio_mode & (O_TEXT | O_BINARY) != 0 {
        return false;
    }
    result.lowio_mode |= O_BINARY;
    true
}

fn parse_t(result: &mut StreamMode) -> bool {
    if result.lowio_mode & (O_TEXT | O_BINARY) != 0 {
        return false;
    }
    result.lowio_mode |= O_TEXT;
    true
}

fn parse_c(result: &mut StreamMode, seen_commit_mode: &mut bool) -> bool {
    if *seen_commit_mode {
        return false;
    }
    *seen_commit_mode = true;
    result.stdio_mode |= STREAM_COMMIT;
    true
}

fn parse_n(result: &mut StreamMode, seen_commit_mode: &mut bool) -> bool {
    if *seen_commit_mode {
        return false;
    }
    *seen_commit_mode = true;
    result.stdio_mode &= !STREAM_COMMIT;
    true
}

fn parse_seq(result: &mut StreamMode, seen_scan_mode: &mut bool) -> bool {
    if *seen_scan_mode {
        return false;
    }
    *seen_scan_mode = true;
    result.lowio_mode |= O_SEQUENTIAL;
    true
}

fn parse_rand(result: &mut StreamMode, seen_scan_mode: &mut bool) -> bool {
    if *seen_scan_mode {
        return false;
    }
    *seen_scan_mode = true;
    result.lowio_mode |= O_RANDOM;
    true
}

fn parse_short_lived(result: &mut StreamMode) -> bool {
    if result.lowio_mode & O_SHORT_LIVED != 0 {
        return false;
    }
    result.lowio_mode |= O_SHORT_LIVED;
    true
}

fn parse_temporary(result: &mut StreamMode) -> bool {
    if result.lowio_mode & O_TEMPORARY != 0 {
        return false;
    }
    result.lowio_mode |= O_TEMPORARY;
    true
}

fn parse_noinherit(result: &mut StreamMode) {
    result.lowio_mode |= O_NOINHERIT;
}

fn parse_excl(result: &mut StreamMode) -> bool {
    if result.lowio_mode & O_TRUNC == 0 {
        // 'x' only permitted with 'w'.
        return false;
    }
    result.lowio_mode |= O_EXCL;
    true
}

/// `__acrt_stdio_parse_mode<Character>(mode)`.
///
/// # Safety
/// `mode` must be a valid NUL-terminated string of its character width.
pub(super) unsafe fn parse_mode<P: ModeCursor>(mode: P) -> StreamMode {
    let mut result = StreamMode {
        lowio_mode: 0,
        stdio_mode: crate::stdio::files::misc::_commode as c_long,
        success: false,
    };

    let mut it = skip_spaces(mode);

    match it.ascii() {
        Some(b'r') => {
            result.lowio_mode = O_RDONLY;
            result.stdio_mode = STREAM_READ;
        }
        Some(b'w') => {
            result.lowio_mode = O_WRONLY | O_CREAT | O_TRUNC;
            result.stdio_mode = STREAM_WRITE;
        }
        Some(b'a') => {
            result.lowio_mode = O_WRONLY | O_CREAT | O_APPEND;
            result.stdio_mode = STREAM_WRITE;
        }
        _ => return invalid(result),
    }
    it = it.advance();

    let mut seen_commit_mode = false;
    let mut seen_plus = false;
    let mut seen_scan_mode = false;
    let mut seen_encoding_flag = false;

    loop {
        match it.ascii() {
            Some(0) => break,
            Some(b'+') => {
                if !parse_plus(&mut result, &mut seen_plus) {
                    return invalid(result);
                }
            }
            Some(b'b') => {
                if !parse_b(&mut result) {
                    return invalid(result);
                }
            }
            Some(b't') => {
                if !parse_t(&mut result) {
                    return invalid(result);
                }
            }
            Some(b'c') => {
                if !parse_c(&mut result, &mut seen_commit_mode) {
                    return invalid(result);
                }
            }
            Some(b'n') => {
                if !parse_n(&mut result, &mut seen_commit_mode) {
                    return invalid(result);
                }
            }
            Some(b'S') => {
                if !parse_seq(&mut result, &mut seen_scan_mode) {
                    return invalid(result);
                }
            }
            Some(b'R') => {
                if !parse_rand(&mut result, &mut seen_scan_mode) {
                    return invalid(result);
                }
            }
            Some(b'T') => {
                if !parse_short_lived(&mut result) {
                    return invalid(result);
                }
            }
            Some(b'D') => {
                if !parse_temporary(&mut result) {
                    return invalid(result);
                }
            }
            Some(b'N') => parse_noinherit(&mut result),
            Some(b'x') => {
                if !parse_excl(&mut result) {
                    return invalid(result);
                }
            }
            Some(b' ') => {}
            Some(b',') => {
                seen_encoding_flag = true;
                break;
            }
            _ => return invalid(result),
        }
        it = it.advance();
    }

    if seen_encoding_flag {
        it = it.advance(); // past the comma
    }
    it = skip_spaces(it);

    if !seen_encoding_flag {
        if it.ascii() != Some(0) {
            return invalid(result);
        }
        result.success = true;
        return result;
    }

    if !matches_exact(it, b"ccs") {
        return invalid(result);
    }
    it = advance_n(it, 3);
    it = skip_spaces(it);

    if it.ascii() != Some(b'=') {
        return invalid(result);
    }
    it = it.advance();
    it = skip_spaces(it);

    if matches_ci(it, b"UTF-8") {
        it = advance_n(it, 5);
        result.lowio_mode |= O_U8TEXT;
    } else if matches_ci(it, b"UTF-16LE") {
        it = advance_n(it, 8);
        result.lowio_mode |= O_U16TEXT;
    } else if matches_ci(it, b"UNICODE") {
        it = advance_n(it, 7);
        result.lowio_mode |= O_WTEXT;
    } else {
        return invalid(result);
    }

    it = skip_spaces(it);
    if it.ascii() != Some(0) {
        return invalid(result);
    }

    result.success = true;
    result
}
