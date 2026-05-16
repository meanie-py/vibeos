//! Integer parse core. Faithful to ReactOS `_atoi64` (`atoi.c` →
//! `_ttoi64`): skip C `isspace`, optional `+`/`-`, accumulate decimal
//! digits with **i64 wrapping** (NO overflow detection — the
//! misfeature), stop at first non-digit. NULL/empty → 0.
//!
//! `atoi` = `(int)_atoi64(..)` — i32 truncation of the wrapped i64,
//! done in the cabi shim's collapse.
//!
//! The honest signal (`overflowed`) is returned alongside the wrapped
//! value: the C `atoi`/`_atoi64` shims discard it (return the wrap);
//! `_rs` callers map it to `Result`.

/// C `isspace` (ASCII): space, \t \n \v \f \r.
#[inline]
fn is_space(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r')
}

/// Returns `(wrapped_i64, overflowed)`. `wrapped_i64` is byte-faithful
/// to `_atoi64` (i64 wrapping arithmetic, sign applied). `overflowed`
/// is the honest signal (true if the magnitude exceeded i64 during
/// accumulation) — discarded by the C shim, surfaced to `_rs`.
#[inline]
pub fn parse_i64_wrapping(s: &[u8]) -> (i64, bool) {
    let mut i = 0;
    while i < s.len() && is_space(s[i]) {
        i += 1;
    }
    let mut neg = false;
    if i < s.len() && (s[i] == b'-' || s[i] == b'+') {
        neg = s[i] == b'-';
        i += 1;
    }
    let mut acc: i64 = 0;
    let mut overflowed = false;
    while i < s.len() && s[i].is_ascii_digit() {
        let d = (s[i] - b'0') as i64;
        // _atoi64 does `acc = 10*acc + d` in plain __int64 (wraps).
        let (m, o1) = acc.overflowing_mul(10);
        let (a, o2) = m.overflowing_add(d);
        overflowed |= o1 | o2;
        acc = a;
        i += 1;
    }
    if neg {
        acc = acc.wrapping_neg();
    }
    (acc, overflowed)
}

/// Result of `parse_i64_clamped` (the `_strtoi64_l` family). `base_ok`
/// false ⇒ invalid `base` arg: the C fn returns 0 and does NOT write
/// `*endptr`. Otherwise the shim writes `*endptr = nptr + end` where
/// `end` is the offset past the last consumed digit if any digit was
/// seen, else 0 (the *original* nptr — leading ws/sign are NOT
/// reflected when no digit parsed, per strtoi64.c).
pub struct ClampedI64 {
    pub value: i64,
    pub end: usize,
    pub base_ok: bool,
}

#[inline]
fn lc(b: u8) -> u8 {
    b.to_ascii_lowercase()
}

/// `_strtoi64_l` core, faithful to ReactOS `strtoi64.c`: base 0 or
/// 2..=36 (else `base_ok=false`); skip C isspace; optional `-`/`+`;
/// `0x` prefix when base is 0 or 16; base 0 ⇒ `0`→octal else decimal;
/// digits via tolower with per-base cutoff; overflow **CLAMPS** to
/// `_I64_MAX`/`_I64_MIN` (does NOT wrap — unlike `parse_i64_wrapping`)
/// and keeps consuming. `_LIBCNT_` build sets no errno.
#[inline]
pub fn parse_i64_clamped(s: &[u8], mut base: i32) -> ClampedI64 {
    if !(base == 0 || (2..=36).contains(&base)) {
        return ClampedI64 { value: 0, end: 0, base_ok: false };
    }
    let mut i = 0;
    while i < s.len() && is_space(s[i]) {
        i += 1;
    }
    let mut negative = false;
    if i < s.len() && (s[i] == b'-' || s[i] == b'+') {
        negative = s[i] == b'-';
        i += 1;
    }
    if (base == 0 || base == 16)
        && i + 1 < s.len()
        && s[i] == b'0'
        && lc(s[i + 1]) == b'x'
    {
        base = 16;
        i += 2;
    }
    if base == 0 {
        base = if i < s.len() && s[i] == b'0' { 8 } else { 10 };
    }
    let b64 = base as i64;
    let mut ret: i64 = 0;
    let mut got_digit = false;
    while i < s.len() {
        let cur = lc(s[i]);
        let v: i64 = if cur.is_ascii_digit() {
            if (cur as i32) >= b'0' as i32 + base {
                break;
            }
            (cur - b'0') as i64
        } else {
            if cur < b'a' || (cur as i32) >= b'a' as i32 + base - 10 {
                break;
            }
            (cur - b'a') as i64 + 10
        };
        got_digit = true;
        let v = if negative { -v } else { v };
        i += 1;
        if !negative && (ret > i64::MAX / b64 || ret * b64 > i64::MAX - v) {
            ret = i64::MAX;
        } else if negative && (ret < i64::MIN / b64 || ret * b64 < i64::MIN - v) {
            ret = i64::MIN;
        } else {
            ret = ret * b64 + v;
        }
    }
    ClampedI64 {
        value: ret,
        end: if got_digit { i } else { 0 },
        base_ok: true,
    }
}

/// Widest int→str digit count: a `u64` in radix 2 is 64 digits. The C
/// (`itoa.c`/`itow.c`) uses `buffer[65]` (64 digits + NUL); the cabi
/// shim appends sign + NUL itself, so the core only needs the digits.
pub const RADIX_DIGITS_MAX: usize = 64;

/// A formatted magnitude — `digits[..ndigits]` MSB-first ASCII
/// (lowercase `a..z` for values 10..=35), NO sign, NO NUL. `negative`
/// records whether a `'-'` precedes them in the full C string. The
/// caller decides `negative` (the `value < 0 && radix == 10` rule lives
/// in the typed shim, not here — for any other radix the C reinterprets
/// the value's bit pattern as unsigned of its own width).
pub struct Formatted {
    pub digits: [u8; RADIX_DIGITS_MAX],
    pub ndigits: usize,
    pub negative: bool,
}

/// Faithful to `itoa.c`/`itow.c`: a `do {} while (val != 0)` — at
/// least one digit, so `mag == 0` → `"0"` — building least-significant
/// first, lowercase `a..z` for 10..=35. `radix` must be 2..=36
/// (libcntpr callers always pass a valid radix; out-of-range is
/// documented C UB — div-by-zero / OOB digit map — and is never
/// fuzzed). Returns the digits MSB-first plus the caller's sign flag.
pub fn format_radix(mut mag: u64, radix: u32, negative: bool) -> Formatted {
    let r = radix as u64;
    let mut lsb = [0u8; RADIX_DIGITS_MAX];
    let mut k = 0;
    loop {
        let d = (mag % r) as u8;
        mag /= r;
        lsb[k] = if d < 10 { b'0' + d } else { b'a' + d - 10 };
        k += 1;
        if mag == 0 {
            break;
        }
    }
    let mut digits = [0u8; RADIX_DIGITS_MAX];
    let mut j = 0;
    while j < k {
        digits[j] = lsb[k - 1 - j];
        j += 1;
    }
    Formatted { digits, ndigits: k, negative }
}

/// Result of [`parse_ulong_wide`] (the `wcstoul` BSD parser). `value`
/// is the 32-bit `unsigned long` result (Windows `long` is 32-bit on
/// ALL arches — LLP64 — so this is correct everywhere, no LP32 caveat
/// like the itoa long-family). `end` is the `*endptr` offset from
/// `nptr`: `any ? (s-1) : nptr`, i.e. the first non-consumed char if
/// at least one digit char was seen (even on overflow), else 0.
pub struct WcstoulRes {
    pub value: u32,
    pub end: usize,
}

#[inline]
fn isw_space(c: u16) -> bool {
    matches!(c, 0x20 | 0x09 | 0x0a | 0x0b | 0x0c | 0x0d)
}

/// `wcstoul` core, faithful to ReactOS `wstring/wcstoul.c` (the BSD
/// cutoff/cutlim algorithm). ASCII classification only — `iswctype`
/// for the ASCII range is standard; non-ASCII wide chars are NOT
/// fuzzed (documented domain limit, like the itoa radix / long-width
/// restrictions). Misfeatures pinned: overflow saturates to
/// `ULONG_MAX` but **still advances** `*endptr` past every digit
/// (sticky `any=-1`); a leading `-` parses then the unsigned result is
/// negated at the end (`0u32.wrapping_sub(acc)` — so `-1`→`0xFFFFFFFF`,
/// and a negated overflow yields `0u32.wrapping_sub(0xFFFFFFFF)==1`,
/// NOT `ULONG_MAX`); base 0 ⇒ `0x`→16, else `0`→8 else 10. `base` is
/// assumed non-negative (a negative base's `(unsigned long)base` cast
/// is host/target-divergent and never fuzzed).
#[inline]
pub fn parse_ulong_wide(s: &[u16], base: i32) -> WcstoulRes {
    let at = |i: usize| -> u16 {
        if i < s.len() {
            s[i]
        } else {
            0
        }
    };
    let mut i = 0usize;
    // do { c = *s++; } while (iswctype(c, _SPACE)); — always reads ≥1.
    let mut cur;
    loop {
        cur = at(i);
        i += 1;
        if !isw_space(cur) {
            break;
        }
    }
    let mut neg = false;
    if cur == b'-' as u16 {
        neg = true;
        cur = at(i);
        i += 1;
    } else if cur == b'+' as u16 {
        cur = at(i);
        i += 1;
    }
    let mut base = base;
    if (base == 0 || base == 16)
        && cur == b'0' as u16
        && (at(i) == b'x' as u16 || at(i) == b'X' as u16)
    {
        cur = at(i + 1);
        i += 2;
        base = 16;
    }
    if base == 0 {
        base = if cur == b'0' as u16 { 8 } else { 10 };
    }
    let b = base as u32;
    let cutoff = u32::MAX / b;
    let cutlim = u32::MAX % b;
    let mut acc: u32 = 0;
    let mut any: i32 = 0;
    loop {
        let is_digit = (b'0' as u16..=b'9' as u16).contains(&cur);
        let is_upper = (b'A' as u16..=b'Z' as u16).contains(&cur);
        let is_lower = (b'a' as u16..=b'z' as u16).contains(&cur);
        let d: u32 = if is_digit {
            (cur - b'0' as u16) as u32
        } else if is_upper || is_lower {
            let sub = if is_upper {
                b'A' as u16 - 10
            } else {
                b'a' as u16 - 10
            };
            (cur - sub) as u32
        } else {
            break;
        };
        if d >= b {
            break;
        }
        if any < 0 || acc > cutoff || (acc == cutoff && d > cutlim) {
            any = -1;
        } else {
            any = 1;
            acc = acc.wrapping_mul(b).wrapping_add(d);
        }
        cur = at(i);
        i += 1;
    }
    let value = if any < 0 {
        u32::MAX
    } else if neg {
        0u32.wrapping_sub(acc)
    } else {
        acc
    };
    WcstoulRes {
        value,
        end: if any != 0 { i - 1 } else { 0 },
    }
}
