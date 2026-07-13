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
    /// The parse SATURATED at `_I64_MAX`/`_I64_MIN` rather than representing
    /// the input exactly.
    ///
    /// The kernel (`_LIBCNT_`) has no errno and ignores this. msvcrt does not:
    /// `_strtoi64_l` sets `errno = ERANGE` on exactly this condition, and its
    /// callers (`strtol`, `strtoul`, `_atoi64`) inherit it. The caller cannot
    /// re-derive the flag from `value` alone — a clamped result and a legitimate
    /// `"9223372036854775807"` are the same i64 — so the engine, which is the
    /// only party that knows, has to say so. Not reporting it is how msvcrt's
    /// `strtol` shipped without its ERANGE contract (§9.66).
    pub overflowed: bool,
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
        return ClampedI64 { value: 0, end: 0, base_ok: false, overflowed: false };
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
    let mut overflowed = false;
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
        // The `||` short-circuit is load-bearing, exactly as in the C: the second
        // disjunct's `ret * b64` is only evaluated once the first has proved
        // `ret <= i64::MAX / b64`, which is precisely the condition under which
        // that multiply cannot overflow. (In C the overflow would be UB; in Rust
        // it would panic in debug. Neither happens, for the same reason.)
        if !negative && (ret > i64::MAX / b64 || ret * b64 > i64::MAX - v) {
            ret = i64::MAX;
            overflowed = true;
        } else if negative && (ret < i64::MIN / b64 || ret * b64 < i64::MIN - v) {
            ret = i64::MIN;
            overflowed = true;
        } else {
            ret = ret * b64 + v;
        }
    }
    ClampedI64 {
        value: ret,
        end: if got_digit { i } else { 0 },
        base_ok: true,
        overflowed,
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

/// Code unit for the generic BSD parsers: `u8` (narrow) / `u16`
/// (wide). Classification is ASCII (`isspace`/`isdigit`/`isalpha`/
/// `isupper` resp. `iswctype` for the ASCII range are standard);
/// non-ASCII is never fuzzed (documented domain limit).
pub trait Cu: Copy {
    fn u(self) -> u32;
}
impl Cu for u8 {
    #[inline]
    fn u(self) -> u32 {
        self as u32
    }
}
impl Cu for u16 {
    #[inline]
    fn u(self) -> u32 {
        self as u32
    }
}

#[inline]
fn sp(c: u32) -> bool {
    matches!(c, 0x20 | 0x09 | 0x0a | 0x0b | 0x0c | 0x0d)
}

/// Shared BSD `strtol`/`strtoull` prologue: skip space (do-while ≥1),
/// sign, `0x` prefix (base 0/16), base-0 → 8/10. Returns
/// `(base, negative, index, first_char)`.
#[inline]
fn bsd_prologue<C: Cu>(s: &[C], mut base: i32) -> (i32, bool, usize, u32) {
    let at = |i: usize| -> u32 {
        if i < s.len() {
            s[i].u()
        } else {
            0
        }
    };
    let mut i = 0usize;
    let mut c;
    loop {
        c = at(i);
        i += 1;
        if !sp(c) {
            break;
        }
    }
    let mut neg = false;
    if c == b'-' as u32 {
        neg = true;
        c = at(i);
        i += 1;
    } else if c == b'+' as u32 {
        c = at(i);
        i += 1;
    }
    if (base == 0 || base == 16) && c == b'0' as u32 && (at(i) == b'x' as u32 || at(i) == b'X' as u32)
    {
        c = at(i + 1);
        i += 2;
        base = 16;
    }
    if base == 0 {
        base = if c == b'0' as u32 { 8 } else { 10 };
    }
    (base, neg, i, c)
}

/// One BSD digit step: classify `c` (ASCII digit / a-z / A-Z) → value,
/// or `None` to stop. Faithful to `strtol.c`'s `c -= '0'` /
/// `c -= isupper ? 'A'-10 : 'a'-10` then the `c >= base` break.
#[inline]
fn bsd_digit(c: u32, base: i32) -> Option<i64> {
    let v: i64 = if (b'0' as u32..=b'9' as u32).contains(&c) {
        (c - b'0' as u32) as i64
    } else if (b'A' as u32..=b'Z' as u32).contains(&c) {
        (c - b'A' as u32) as i64 + 10
    } else if (b'a' as u32..=b'z' as u32).contains(&c) {
        (c - b'a' as u32) as i64 + 10
    } else {
        return None;
    };
    if v >= base as i64 {
        None
    } else {
        Some(v)
    }
}

/// `strtol`/`wcstol` result. `value` is the 32-bit Windows `long`
/// (LLP64 — `long` is 32-bit on every Windows arch). `end` is the
/// `*endptr` offset: `any ? (s-1) : nptr`.
pub struct LongRes {
    pub value: i32,
    pub end: usize,
}

/// `strtol`/`wcstol` core, faithful to ReactOS `strtol.c`/`wcstol.c`
/// (BSD cutoff/cutlim with the **signed** LONG clamp). `unsigned long`
/// is 32-bit on the target, so the cnt-rust model is exact i32; only
/// the LP64 host *reference* needs the `LONG_MAX`→0x7FFFFFFF + `(int)`
/// trick. ASCII-domain only. Overflow → `LONG_MIN`/`LONG_MAX`;
/// negative non-overflow → `0u32.wrapping_sub(acc)` as i32.
#[inline]
pub fn parse_long_bsd<C: Cu>(s: &[C], base: i32) -> LongRes {
    let at = |i: usize| -> u32 {
        if i < s.len() {
            s[i].u()
        } else {
            0
        }
    };
    let (base, neg, mut i, mut c) = bsd_prologue(s, base);
    let bm = base as i64;
    // cutoff = neg ? (unsigned long)LONG_MAX+1 : LONG_MAX  (LP32)
    let cut0: i64 = if neg { 0x8000_0000 } else { 0x7FFF_FFFF };
    let cutlim = cut0 % bm;
    let cutoff = cut0 / bm;
    let mut acc: i64 = 0;
    let mut any: i32 = 0;
    loop {
        let d = match bsd_digit(c, base) {
            Some(d) => d,
            None => break,
        };
        if any < 0 || acc > cutoff || (acc == cutoff && d > cutlim) {
            any = -1;
        } else {
            any = 1;
            acc = acc * bm + d;
        }
        c = at(i);
        i += 1;
    }
    let value: i32 = if any < 0 {
        if neg {
            i32::MIN
        } else {
            i32::MAX
        }
    } else if neg {
        0u32.wrapping_sub(acc as u32) as i32
    } else {
        acc as i32
    };
    LongRes {
        value,
        end: if any != 0 { i - 1 } else { 0 },
    }
}

/// `strtoull` result (`unsigned long long` — 64-bit on host AND
/// target, so no LP32 caveat, host reference is faithful directly).
pub struct U64Res {
    pub value: u64,
    pub end: usize,
}

/// `strtoull` core, faithful to ReactOS `strtoull.c` (BSD cutoff with
/// `ULLONG_MAX` clamp; negative → `0 - acc` wrapping u64). ASCII only.
#[inline]
pub fn parse_u64_bsd<C: Cu>(s: &[C], base: i32) -> U64Res {
    let at = |i: usize| -> u32 {
        if i < s.len() {
            s[i].u()
        } else {
            0
        }
    };
    let (base, neg, mut i, mut c) = bsd_prologue(s, base);
    let bm = base as u64;
    let cutoff = u64::MAX / bm;
    let cutlim = (u64::MAX % bm) as i64;
    let mut acc: u64 = 0;
    let mut any: i32 = 0;
    loop {
        let d = match bsd_digit(c, base) {
            Some(d) => d,
            None => break,
        };
        if any < 0 || acc > cutoff || (acc == cutoff && d > cutlim) {
            any = -1;
        } else {
            any = 1;
            acc = acc * bm + d as u64;
        }
        c = at(i);
        i += 1;
    }
    let value = if any < 0 {
        u64::MAX
    } else if neg {
        0u64.wrapping_sub(acc)
    } else {
        acc
    };
    U64Res {
        value,
        end: if any != 0 { i - 1 } else { 0 },
    }
}

// =====================================================================
// Wave G2 — wide-atoi parse family (wtoi64.c / wtol.c / atoi.c _UNICODE)
//
// Distinct from the BSD `parse_long_bsd`/`parse_u64_bsd` (cutoff/cutlim,
// `any?s-1:nptr` endptr) and from the narrow `parse_i64_clamped`
// (`got_digit ? nptr : p` endptr). The wide MSVCRT-lineage parsers use
// the per-digit `MAX/base` clamp AND a different endptr contract:
// `*endptr = nptr` **always** (final position incl. ws/sign/`0x` even
// with zero digits), unlike narrow `_strtoi64`. Generic over `Cu` so
// the same logic serves any future narrow twin; today only u16 callers.
// ASCII-domain only (`iswctype`/`towlower` for the ASCII range are
// standard; non-ASCII wide chars never fuzzed — documented limit, same
// as `parse_ulong_wide`).
// =====================================================================

/// ASCII lowercase on a code-unit value (faithful to `towlower` over
/// the ASCII range; non-ASCII passes through, never fuzzed).
#[inline]
fn lc_u32(c: u32) -> u32 {
    if (b'A' as u32..=b'Z' as u32).contains(&c) {
        c + 32
    } else {
        c
    }
}

/// `_wtoi64_l`/`_wtoi64` core (wtoi64.c) — also `_wtoi` = `(int)` of
/// this. Faithful: skip `iswctype _SPACE` (ASCII set), optional `+`/`-`,
/// accumulate decimal digits with **i64 wrapping** (NO overflow
/// detection — the misfeature; host `LONGLONG`==target i64), negate via
/// two's-complement. NO NULL guard in the C (it derefs immediately) —
/// callers never fuzz NULL. `overflowed` is the honest signal the C
/// shim discards. Generic `Cu`; narrow twin = `parse_i64_wrapping`.
#[inline]
pub fn parse_i64_wrapping_cu<C: Cu>(s: &[C]) -> (i64, bool) {
    let mut i = 0usize;
    while i < s.len() && sp(s[i].u()) {
        i += 1;
    }
    let mut neg = false;
    if i < s.len() {
        let c = s[i].u();
        if c == b'-' as u32 || c == b'+' as u32 {
            neg = c == b'-' as u32;
            i += 1;
        }
    }
    let mut acc: i64 = 0;
    let mut overflowed = false;
    while i < s.len() {
        let c = s[i].u();
        if !(b'0' as u32..=b'9' as u32).contains(&c) {
            break;
        }
        let d = (c - b'0' as u32) as i64;
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

/// `_wtol` core (wine wtol.c) — decimal-only, **u32 wrapping** (Windows
/// `unsigned long` is 32-bit), `bMinus ? 0-RunningTotal : RunningTotal`.
/// NULL→0 is the empty-slice case (the shim's `wcstr_slice` maps NULL→
/// `&[]`, identical to wtol.c's explicit `str==NULL` guard). No base, no
/// `0x`, no overflow detection.
#[inline]
pub fn parse_u32_dec_w<C: Cu>(s: &[C]) -> u32 {
    let mut i = 0usize;
    while i < s.len() && sp(s[i].u()) {
        i += 1;
    }
    let mut neg = false;
    if i < s.len() {
        let c = s[i].u();
        if c == b'+' as u32 {
            i += 1;
        } else if c == b'-' as u32 {
            neg = true;
            i += 1;
        }
    }
    let mut acc: u32 = 0;
    while i < s.len() {
        let c = s[i].u();
        if !(b'0' as u32..=b'9' as u32).contains(&c) {
            break;
        }
        acc = acc.wrapping_mul(10).wrapping_add(c - b'0' as u32);
        i += 1;
    }
    if neg {
        0u32.wrapping_sub(acc)
    } else {
        acc
    }
}

/// Shared wide prologue (wtoi64.c `_wcstoi64_l`/`_wcstoui64_l`): base
/// validity (0 or 2..=36), skip `iswctype _SPACE`, optional sign, `0x`
/// prefix when base 0/16 (consumed unconditionally — endptr reflects it
/// even with no following hex digit, the wide misfeature), base-0 ⇒
/// `0`→octal else decimal. `at(j)` returns 0 past end == C reading the
/// NUL. Returns `(base, negative, index)` or `None` if base invalid.
#[inline]
fn wide_parse_prologue<C: Cu>(s: &[C], mut base: i32) -> Option<(i32, bool, usize)> {
    if !(base == 0 || (2..=36).contains(&base)) {
        return None;
    }
    let at = |j: usize| -> u32 {
        if j < s.len() {
            s[j].u()
        } else {
            0
        }
    };
    let mut i = 0usize;
    while i < s.len() && sp(s[i].u()) {
        i += 1;
    }
    let mut negative = false;
    if i < s.len() {
        let c = s[i].u();
        if c == b'-' as u32 || c == b'+' as u32 {
            negative = c == b'-' as u32;
            i += 1;
        }
    }
    if (base == 0 || base == 16)
        && at(i) == b'0' as u32
        && lc_u32(at(i + 1)) == b'x' as u32
    {
        base = 16;
        i += 2;
    }
    if base == 0 {
        base = if at(i) == b'0' as u32 { 8 } else { 10 };
    }
    Some((base, negative, i))
}

/// Result of [`parse_i64_clamped_w`]/[`parse_u64_clamped_w`].
/// `base_ok==false` ⇒ invalid `base`: the C returns 0 and does NOT
/// write `*endptr`. Otherwise `*endptr = nptr + end` where `end` is the
/// **final** parse position (after ws/sign/`0x`/digits) — written even
/// when zero digits were consumed (the wide endptr contract, distinct
/// from narrow `_strtoi64`'s `got_digit ? nptr : p`). NULL `nptr` is
/// handled by the shim (return 0, no endptr write) — not here.
pub struct WideClamped<T> {
    pub value: T,
    pub end: usize,
    pub base_ok: bool,
}

/// `_wcstoi64_l`/`_wcstoi64` core (wtoi64.c) — signed i64, per-digit
/// clamp to `_I64_MAX`/`_I64_MIN` (NOT wrap), keeps consuming after
/// clamp. Digit via `towlower` then per-base cutoff. Endptr = final
/// index always (see [`WideClamped`]).
#[inline]
pub fn parse_i64_clamped_w<C: Cu>(s: &[C], base: i32) -> WideClamped<i64> {
    let (base, negative, mut i) = match wide_parse_prologue(s, base) {
        Some(t) => t,
        None => {
            return WideClamped { value: 0, end: 0, base_ok: false };
        }
    };
    let b64 = base as i64;
    let mut ret: i64 = 0;
    while i < s.len() {
        let cur = lc_u32(s[i].u());
        let v: i64 = if (b'0' as u32..=b'9' as u32).contains(&cur) {
            if cur >= b'0' as u32 + base as u32 {
                break;
            }
            (cur - b'0' as u32) as i64
        } else {
            if cur < b'a' as u32 || cur >= b'a' as u32 + base as u32 - 10 {
                break;
            }
            (cur - b'a' as u32) as i64 + 10
        };
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
    WideClamped { value: ret, end: i, base_ok: true }
}

/// `_wcstoui64_l`/`_wcstoui64` core (wtoi64.c) — unsigned u64, clamp to
/// `_UI64_MAX`, then `negative ? -(__int64)ret : ret` reinterpreted
/// unsigned (== `0u64.wrapping_sub(ret)`). Keeps consuming after clamp.
/// Endptr = final index always. The `||` short-circuit guarantees
/// `ret*base` cannot overflow u64 (only reached when `ret<=UMAX/base`).
#[inline]
pub fn parse_u64_clamped_w<C: Cu>(s: &[C], base: i32) -> WideClamped<u64> {
    let (base, negative, mut i) = match wide_parse_prologue(s, base) {
        Some(t) => t,
        None => {
            return WideClamped { value: 0, end: 0, base_ok: false };
        }
    };
    let bm = base as u64;
    let umax = u64::MAX;
    let mut ret: u64 = 0;
    while i < s.len() {
        let cur = lc_u32(s[i].u());
        let v: u64 = if (b'0' as u32..=b'9' as u32).contains(&cur) {
            if cur >= b'0' as u32 + base as u32 {
                break;
            }
            (cur - b'0' as u32) as u64
        } else {
            if cur < b'a' as u32 || cur >= b'a' as u32 + base as u32 - 10 {
                break;
            }
            (cur - b'a' as u32) as u64 + 10
        };
        i += 1;
        if ret > umax / bm || ret * bm > umax - v {
            ret = umax;
        } else {
            ret = ret * bm + v;
        }
    }
    let value = if negative {
        0u64.wrapping_sub(ret)
    } else {
        ret
    };
    WideClamped { value, end: i, base_ok: true }
}
