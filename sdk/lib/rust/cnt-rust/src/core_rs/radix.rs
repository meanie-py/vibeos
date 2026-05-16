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
