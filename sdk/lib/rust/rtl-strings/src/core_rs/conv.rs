//! Wave RTL-4a — the pure integer ⇄ string kernels of
//! `sdk/lib/rtl/unicode.c` (`RtlCharToInteger`,
//! `RtlUnicodeStringToInteger`, `RtlIntegerToUnicode`). No NLS, no
//! allocation, no `unsafe` — only the faithful misfeatures:
//!
//!  - the leading-whitespace skip is `*str <= ' '` on a **signed**
//!    `CHAR` (so every high byte 0x80–0xFF, being negative, is eaten
//!    as "whitespace"); the WCHAR variant compares the **unsigned**
//!    unit (so an embedded NUL / control unit is skipped too, since
//!    the loop is `CharsRemaining`-bounded, not NUL-bounded);
//!  - base validation happens **before** the `value == NULL` check
//!    (so an invalid base out-ranks a NULL out-pointer) — encoded by
//!    [`IntParse::InvalidBase`] being returned without ever looking at
//!    the out-pointer;
//!  - `RunningTotal = RunningTotal * base + digit` is `ULONG` and
//!    wraps; `*value = bMinus ? (0 - RunningTotal) : RunningTotal` is
//!    the two's-complement negate;
//!  - `RtlIntegerToUnicode`'s size gate is `ndigits * sizeof(WCHAR)
//!    >= Length` and it then writes `ndigits` units **plus** a NUL —
//!    i.e. the trailing NUL is *not* counted by the gate (the genuine
//!    off-by-one-NUL overrun quirk; the caller buffer must be sized
//!    for it).

/// Result of the two string→integer parsers. `InvalidBase` is yielded
/// strictly before the out-pointer is consulted (faithful ordering).
pub enum IntParse {
    InvalidBase,
    Value(u32),
}

#[inline]
fn classify_digit(c: i32) -> i32 {
    if (0x30..=0x39).contains(&c) {
        c - 0x30
    } else if (0x41..=0x5A).contains(&c) {
        c - 0x41 + 10
    } else if (0x61..=0x7A).contains(&c) {
        c - 0x61 + 10
    } else {
        -1
    }
}

#[inline]
fn accumulate(total: u32, base: u32, digit: i32) -> u32 {
    total.wrapping_mul(base).wrapping_add(digit as u32)
}

/// `RtlCharToInteger(PCSZ str, ULONG base, PULONG value)`. `s` is the
/// `strlen`-bounded byte view (NUL excluded). The signed-`CHAR`
/// comparisons are modelled with `as i8 as i32`.
pub fn char_to_integer(s: &[u8], mut base: u32) -> IntParse {
    let n = s.len();
    let mut i = 0usize;

    // `while (*str != '\0' && *str <= ' ')` — signed char.
    while i < n && (s[i] as i8 as i32) <= b' ' as i32 {
        i += 1;
    }

    let mut minus = false;
    if i < n {
        if s[i] == b'+' {
            i += 1;
        } else if s[i] == b'-' {
            minus = true;
            i += 1;
        }
    }

    if base == 0 {
        base = 10;
        if i < n && s[i] == b'0' {
            match s.get(i + 1).copied().unwrap_or(0) {
                b'b' => {
                    i += 2;
                    base = 2;
                }
                b'o' => {
                    i += 2;
                    base = 8;
                }
                b'x' => {
                    i += 2;
                    base = 16;
                }
                _ => {}
            }
        }
    } else if base != 2 && base != 8 && base != 10 && base != 16 {
        return IntParse::InvalidBase;
    }

    let mut total: u32 = 0;
    while i < n {
        let digit = classify_digit(s[i] as i8 as i32);
        if digit < 0 || digit >= base as i32 {
            break;
        }
        total = accumulate(total, base, digit);
        i += 1;
    }

    IntParse::Value(if minus { 0u32.wrapping_sub(total) } else { total })
}

/// `RtlUnicodeStringToInteger(const UNICODE_STRING*, ULONG, ULONG*)`.
/// `u` is the `Length / sizeof(WCHAR)` unit view; the loop is
/// `CharsRemaining`-bounded so embedded NUL/control units are eaten by
/// the unsigned `*lpwstr <= ' '` whitespace skip.
pub fn unicode_string_to_integer(u: &[u16], mut base: u32) -> IntParse {
    let n = u.len();
    let mut i = 0usize;

    while i < n && u[i] <= b' ' as u16 {
        i += 1;
    }

    let mut minus = false;
    if i < n {
        if u[i] == b'+' as u16 {
            i += 1;
        } else if u[i] == b'-' as u16 {
            minus = true;
            i += 1;
        }
    }

    if base == 0 {
        base = 10;
        if n - i >= 2 && u[i] == b'0' as u16 {
            match u[i + 1] {
                x if x == b'b' as u16 => {
                    i += 2;
                    base = 2;
                }
                x if x == b'o' as u16 => {
                    i += 2;
                    base = 8;
                }
                x if x == b'x' as u16 => {
                    i += 2;
                    base = 16;
                }
                _ => {}
            }
        }
    } else if base != 2 && base != 8 && base != 10 && base != 16 {
        return IntParse::InvalidBase;
    }

    let mut total: u32 = 0;
    while i < n {
        let digit = classify_digit(u[i] as i32);
        if digit < 0 || digit >= base as i32 {
            break;
        }
        total = accumulate(total, base, digit);
        i += 1;
    }

    IntParse::Value(if minus { 0u32.wrapping_sub(total) } else { total })
}

/// Result of [`integer_to_unicode`]. `Digits` is already in output
/// (most-significant-first) order; the caller writes `count` units
/// then a NUL.
pub enum IntToUni {
    InvalidBase,
    TooSmall,
    Digits([u16; 33], usize),
}

/// `RtlIntegerToUnicode(ULONG Value, ULONG Base, ULONG Length, LPWSTR)`
/// — produces the digit units. The `Length` gate is the genuine
/// `ndigits * sizeof(WCHAR) >= Length` (the NUL is **not** counted).
pub fn integer_to_unicode(value: u32, base: u32, length: u32) -> IntToUni {
    let radix = if base == 0 { 10 } else { base };
    if radix != 2 && radix != 8 && radix != 10 && radix != 16 {
        return IntToUni::InvalidBase;
    }

    // `while (v || tp == temp)` — do-while: at least one digit.
    let mut tmp = [0u16; 33];
    let mut v = value;
    let mut c = 0usize;
    loop {
        let d = v % radix;
        v /= radix;
        tmp[c] = if d < 10 {
            (d as u16) + b'0' as u16
        } else {
            (d as u16) + b'a' as u16 - 10
        };
        c += 1;
        if v == 0 {
            break;
        }
    }

    if (c as u32) * 2 >= length {
        return IntToUni::TooSmall;
    }

    let mut out = [0u16; 33];
    for i in 0..c {
        out[i] = tmp[c - 1 - i];
    }
    IntToUni::Digits(out, c)
}

/// `RtlIntegerToChar` digit production (the do-while reverse fill into
/// `buffer[33]`). **Uppercase** hex (`'A'+d-10`) — unlike
/// [`integer_to_unicode`]'s lowercase. Returns the digits in output
/// order + count; `None` ⇒ invalid base. The caller applies the
/// `len`/`length` overflow / NUL / `str==NULL` contract.
pub fn integer_to_char(value: u32, base: u32) -> Option<([u8; 32], usize)> {
    let base = if base == 0 { 10 } else { base };
    if base != 2 && base != 8 && base != 10 && base != 16 {
        return None;
    }
    let mut tmp = [0u8; 32];
    let mut v = value;
    let mut c = 0usize;
    loop {
        let d = (v % base) as u8;
        v /= base;
        tmp[c] = if d < 10 { b'0' + d } else { b'A' + d - 10 };
        c += 1;
        if v == 0 {
            break;
        }
    }
    let mut out = [0u8; 32];
    for i in 0..c {
        out[i] = tmp[c - 1 - i];
    }
    Some((out, c))
}

/// `RtlLargeIntegerToChar` — the 64-bit twin of [`integer_to_char`]
/// (`buffer[65]`, up to 64 digits, uppercase).
pub fn large_integer_to_char(value: u64, base: u32) -> Option<([u8; 64], usize)> {
    let base = if base == 0 { 10 } else { base };
    if base != 2 && base != 8 && base != 10 && base != 16 {
        return None;
    }
    let base64 = base as u64;
    let mut tmp = [0u8; 64];
    let mut v = value;
    let mut c = 0usize;
    loop {
        let d = (v % base64) as u8;
        v /= base64;
        tmp[c] = if d < 10 { b'0' + d } else { b'A' + d - 10 };
        c += 1;
        if v == 0 {
            break;
        }
    }
    let mut out = [0u8; 64];
    for i in 0..c {
        out[i] = tmp[c - 1 - i];
    }
    Some((out, c))
}
