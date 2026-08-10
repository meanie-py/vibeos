//! The remainder of `dll/win32/msvcrt/wcs.c`: the wide integer parsers, the four
//! locale-taking `_l` case/compare bodies whose plain twins wave 8 already flipped, and
//! the wide→multibyte converter family.
//!
//! Everything here is a 1:1 transcription of the C, quirks included. Three of those
//! quirks are load-bearing enough to name up front, because each looks like a bug an
//! "improving" translation would quietly fix:
//!
//!  * `_wtoi64_l` has **no overflow check at all** — it multiply-accumulates into a
//!    `ULONGLONG` and lets it wrap. `_wcstoi64_l` right next to it saturates and sets
//!    ERANGE. Same file, same shape, deliberately different contract.
//!  * `wctob` tests `wchar < 0xff`, not `<= 0xff`, so U+00FF alone falls through to the
//!    `EOF` arm in the raw-codepage case.
//!  * `wcsrtombs_s_l` can index `mbstr[-1]`: reaching its `conv == size` arm with
//!    `conv == 0` evaluates `mbstr[conv-1]` on a `size_t`. The pointer arithmetic below
//!    is spelled `wrapping_*` so it lowers to exactly the byte the C touches instead of
//!    trapping on the subtraction.
//!
//! The `_l` bodies here and their already-flipped plain twins in `lib.rs` are separate
//! transcriptions of the same C, not a call chain — wave 8 wrote `_wcsicmp` etc. against
//! an already-resolved `locinfo` to keep the environment lookup O(1) per call. That
//! shape is preserved: the bodies below resolve the locale once via
//! `get_current_locale_noalloc` and pass it down, exactly as the C does.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use crate::prelude::*;
use core::ptr::{null, null_mut};

// ── constants bindgen would not fold ─────────────────────────────────────────────────
// Each is a macro, so there is no symbol for the island to snapshot. Values are the
// header's, and the `long` ones are the WINDOWS widths: msvcrt's `long` is 32 bits on
// every target we build, and the difftest's refstub pins `<limits.h>` to the same three
// literals so the reference C clamps where we clamp.

/// `dll/win32/msvcrt/include/limits.h` — `I64_MAX`/`I64_MIN`/`UI64_MAX`.
const I64_MAX: i64 = 9223372036854775807;
const I64_MIN: i64 = -9223372036854775807 - 1;
const UI64_MAX: u64 = 18446744073709551615;

/// `<limits.h>` under a 32-bit `long`. `ULONG_MAX` is a **signed 64-bit** literal on
/// purpose: `_wcstoul_l` compares it against an `__int64`, and an unsigned 64-bit
/// spelling would make `ret < -(__int64)ULONG_MAX` wrongly clamp every negative value.
const LONG_MAX: i64 = 0x7FFF_FFFF;
const LONG_MIN: i64 = -0x7FFF_FFFF - 1;
const ULONG_MAX: i64 = 0xFFFF_FFFF;

/// `<limits.h>` — the island carries `INT_MAX` but not its negative twin.
const INT_MIN: i64 = -2147483647 - 1;

/// `sdk/include/psdk/winnls.h:242` — `#define WC_NO_BEST_FIT_CHARS 0x0400`
const WC_NO_BEST_FIT_CHARS: DWORD = 0x0400;

/// `sdk/include/psdk/winerror.h:316` — `#define ERROR_INSUFFICIENT_BUFFER 122`
const ERROR_INSUFFICIENT_BUFFER: DWORD = 122;

/// `(size_t)-1`, the failure return of the `wcsrtombs` family and the value its callers
/// compare against with a bare `== -1`.
const MINUS_ONE: size_t = usize::MAX;

/// `corecrt.h` — the signed twin of the island's `__msvcrt_ulong`. 32-bit on both
/// branches of that header (`int` under `__LP64__`, LLP64 `long` on Windows), so it is
/// 32-bit on i386, amd64 and arm64 alike. Only the unsigned one appears in a struct, so
/// bindgen never emitted this alias.
#[allow(clippy::upper_case_acronyms)]
type __msvcrt_long = i32;

extern "C" {
    /// `msvcrt.h:341/:342`. These stay C (locale.c): the first fills a caller-provided
    /// `_locale_tstruct` from thread data and takes a reference on it, the second drops
    /// it. Both tolerate the zeroed struct a non-NULL `locale` argument leaves behind,
    /// which is why the C calls `free_locale_noalloc` unconditionally and so do we.
    fn get_current_locale_noalloc(locale: _locale_t) -> _locale_t;
    fn free_locale_noalloc(locale: _locale_t);
}

extern "system" {
    /// kernel32, therefore **WINAPI** — stdcall on i386, cdecl elsewhere. `extern "C"`
    /// here would link on amd64/arm64 and fail only on i386.
    fn GetLastError() -> DWORD;
}

/// `_locale_tstruct tmp = {0};` — the on-stack locale every `_l` body below opens with.
#[inline]
fn zeroed_locale() -> localeinfo_struct {
    localeinfo_struct { locinfo: null_mut(), mbcinfo: null_mut() }
}

/// `locale ? locale->locinfo : get_locinfo()`.
#[inline]
unsafe fn locinfo_of(locale: _locale_t) -> pthreadlocinfo {
    if locale.is_null() {
        get_locinfo()
    } else {
        (*locale).locinfo
    }
}

// ═════════════════════════════════════════════════════════════════════════════════════
// wcsncat
// ═════════════════════════════════════════════════════════════════════════════════════

/// `wcsncat (NTDLL.@)` — note it is NOT `wcsncat_s`: no terminator is appended when the
/// source fills all `n`, because the `*s1 = 0` is only reached by falling out of the loop.
#[no_mangle]
pub unsafe extern "C" fn wcsncat(s1: *mut wchar_t, s2: *const wchar_t, n: size_t) -> *mut wchar_t {
    let ret = s1;
    let mut s1 = s1;
    let mut s2 = s2;
    let mut n = n;
    while *s1 != 0 {
        s1 = s1.add(1);
    }
    // C: `while (n-- > 0) if (!(*s1++ = *s2++)) return ret;` — the post-decrement wraps
    // past zero in the C too, but the loop has already exited by then.
    while n > 0 {
        n -= 1;
        let c = *s2;
        *s1 = c;
        s1 = s1.add(1);
        s2 = s2.add(1);
        if c == 0 {
            return ret;
        }
    }
    *s1 = 0;
    ret
}

// ═════════════════════════════════════════════════════════════════════════════════════
// The wide integer engines
// ═════════════════════════════════════════════════════════════════════════════════════

/// `wctoint (INTERNAL)` — file-static in the C, so there is no symbol to override and
/// this stays private. The C's `wcstombs`-side twin keeps its own copy for the same
/// reason.
fn wctoint(c: WCHAR, base: c_int) -> c_int {
    let mut v: c_int = -1;
    if (b'0' as WCHAR) <= c && c <= (b'9' as WCHAR) {
        v = (c - b'0' as WCHAR) as c_int;
    } else if (b'A' as WCHAR) <= c && c <= (b'Z' as WCHAR) {
        v = (c - b'A' as WCHAR) as c_int + 10;
    } else if (b'a' as WCHAR) <= c && c <= (b'z' as WCHAR) {
        v = (c - b'a' as WCHAR) as c_int + 10;
    } else {
        // NOTE: MAP_FOLDDIGITS supports too many things.
        // Unicode points that contain digits 0-9; keep this sorted!
        static zeros: [WCHAR; 17] = [
            0x660, 0x6f0, 0x966, 0x9e6, 0xa66, 0xae6, 0xb66, 0xc66, 0xce6, 0xd66, 0xe50,
            0xed0, 0xf20, 0x1040, 0x17e0, 0x1810, 0xff10,
        ];
        // Sorted-ness is what makes `c >= zeros[i]` a valid early-out: once the table
        // passes `c` no later entry can match.
        let mut i = 0usize;
        while i < zeros.len() && c >= zeros[i] {
            if zeros[i] <= c && c <= zeros[i] + 9 {
                v = (c - zeros[i]) as c_int;
                break;
            }
            i += 1;
        }
    }
    if v < base {
        v
    } else {
        -1
    }
}

/// `_wcstoi64_l (MSVCRT.@)`
///
/// The saturating accumulate is written so neither guard can itself overflow: `ret` is
/// already known to be within `I64_MAX/base` when `ret*base` is evaluated, because `||`
/// short-circuits. That is the C's own reasoning, not an added safety net — a
/// non-short-circuiting spelling would be a different program.
#[no_mangle]
pub unsafe extern "C" fn _wcstoi64_l(
    nptr: *const wchar_t,
    endptr: *mut *mut wchar_t,
    base: c_int,
    locale: _locale_t,
) -> i64 {
    let mut negative = false;
    let mut empty = true;
    let mut ret: i64 = 0;
    let mut base = base;
    let mut nptr = nptr;

    // The C opens with a TRACE() on the wine `msvcrt` channel. Dropped: it is a debug
    // macro, and a no-op `fn` in its place would be an optimization barrier.

    if MSVCRT_CHECK_PMT!(!nptr.is_null()) == 0 {
        return 0;
    }
    if MSVCRT_CHECK_PMT!(base == 0 || base >= 2) == 0 {
        return 0;
    }
    if MSVCRT_CHECK_PMT!(base <= 36) == 0 {
        return 0;
    }

    if !endptr.is_null() {
        *endptr = nptr as *mut wchar_t;
    }

    while crate::_iswspace_l(*nptr as u32, locale as *mut c_void) != 0 {
        nptr = nptr.add(1);
    }

    if *nptr == b'-' as wchar_t {
        negative = true;
        nptr = nptr.add(1);
    } else if *nptr == b'+' as wchar_t {
        nptr = nptr.add(1);
    }

    if (base == 0 || base == 16)
        && wctoint(*nptr, 1) == 0
        && (*nptr.add(1) == b'x' as wchar_t || *nptr.add(1) == b'X' as wchar_t)
    {
        base = 16;
        nptr = nptr.add(2);
    }

    if base == 0 {
        base = if wctoint(*nptr, 1) == 0 { 8 } else { 10 };
    }

    while *nptr != 0 {
        let mut v = wctoint(*nptr, base);
        if v < 0 {
            break;
        }

        if negative {
            v = -v;
        }

        nptr = nptr.add(1);
        empty = false;

        let b = base as i64;
        let v = v as i64;
        if !negative && (ret > I64_MAX / b || ret * b > I64_MAX - v) {
            ret = I64_MAX;
            *_errno() = ERANGE as c_int;
        } else if negative && (ret < I64_MIN / b || ret * b < I64_MIN - v) {
            ret = I64_MIN;
            *_errno() = ERANGE as c_int;
        } else {
            ret = ret * b + v;
        }
    }

    if !endptr.is_null() && !empty {
        *endptr = nptr as *mut wchar_t;
    }

    ret
}

/// `_wcstoui64_l (MSVCRT.@)`
///
/// Not a sign-flipped copy of `_wcstoi64_l`: it accumulates a MAGNITUDE and negates at
/// the very end, so `"-1"` yields `UI64_MAX` with no ERANGE, while `"-99999999999999999999"`
/// saturates to `UI64_MAX` first and then negates that. Both are msvcrt's answers.
#[no_mangle]
pub unsafe extern "C" fn _wcstoui64_l(
    nptr: *const wchar_t,
    endptr: *mut *mut wchar_t,
    base: c_int,
    locale: _locale_t,
) -> u64 {
    let mut negative = false;
    let mut empty = true;
    let mut ret: u64 = 0;
    let mut base = base;
    let mut nptr = nptr;

    if MSVCRT_CHECK_PMT!(!nptr.is_null()) == 0 {
        return 0;
    }
    if MSVCRT_CHECK_PMT!(base == 0 || base >= 2) == 0 {
        return 0;
    }
    if MSVCRT_CHECK_PMT!(base <= 36) == 0 {
        return 0;
    }

    if !endptr.is_null() {
        *endptr = nptr as *mut wchar_t;
    }

    while crate::_iswspace_l(*nptr as u32, locale as *mut c_void) != 0 {
        nptr = nptr.add(1);
    }

    if *nptr == b'-' as wchar_t {
        negative = true;
        nptr = nptr.add(1);
    } else if *nptr == b'+' as wchar_t {
        nptr = nptr.add(1);
    }

    if (base == 0 || base == 16)
        && wctoint(*nptr, 1) == 0
        && (*nptr.add(1) == b'x' as wchar_t || *nptr.add(1) == b'X' as wchar_t)
    {
        base = 16;
        nptr = nptr.add(2);
    }

    if base == 0 {
        base = if wctoint(*nptr, 1) == 0 { 8 } else { 10 };
    }

    while *nptr != 0 {
        let v = wctoint(*nptr, base);
        if v < 0 {
            break;
        }

        nptr = nptr.add(1);
        empty = false;

        let b = base as u64;
        let v = v as u64;
        if ret > UI64_MAX / b || ret * b > UI64_MAX - v {
            ret = UI64_MAX;
            *_errno() = ERANGE as c_int;
        } else {
            ret = ret * b + v;
        }
    }

    if !endptr.is_null() && !empty {
        *endptr = nptr as *mut wchar_t;
    }

    // C: `return negative ? -ret : ret;` — unary minus on an unsigned, i.e. modular.
    if negative {
        ret.wrapping_neg()
    } else {
        ret
    }
}

/// `_wcstoi64 (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn _wcstoi64(
    nptr: *const wchar_t,
    endptr: *mut *mut wchar_t,
    base: c_int,
) -> i64 {
    _wcstoi64_l(nptr, endptr, base, null_mut())
}

/// `_wcstoui64 (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn _wcstoui64(
    nptr: *const wchar_t,
    endptr: *mut *mut wchar_t,
    base: c_int,
) -> u64 {
    _wcstoui64_l(nptr, endptr, base, null_mut())
}

// ── the clamps ───────────────────────────────────────────────────────────────────────
// All four run the SIGNED 64-bit engine and clamp its result, including `_wcstoul_l`.
// That is why `wcstoul(L"18446744073709551615", ...)` is not `ULONG_MAX`: the engine
// saturates at `I64_MAX` first, and only then does the ULONG clamp see it.

/// `_wcstol_l (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn _wcstol_l(
    s: *const wchar_t,
    end: *mut *mut wchar_t,
    base: c_int,
    locale: _locale_t,
) -> __msvcrt_long {
    let mut ret = _wcstoi64_l(s, end, base, locale);

    if ret > LONG_MAX {
        ret = LONG_MAX;
        *_errno() = ERANGE as c_int;
    } else if ret < LONG_MIN {
        ret = LONG_MIN;
        *_errno() = ERANGE as c_int;
    }
    ret as __msvcrt_long
}

/// `wcstol (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn wcstol(
    s: *const wchar_t,
    end: *mut *mut wchar_t,
    base: c_int,
) -> __msvcrt_long {
    _wcstol_l(s, end, base, null_mut())
}

/// `_wcstoul_l (MSVCRT.@)`
///
/// The underflow arm clamps to **1**, not to 0 and not to `ULONG_MAX`. It looks like a
/// typo and it is msvcrt's behaviour; leave it.
#[no_mangle]
pub unsafe extern "C" fn _wcstoul_l(
    s: *const wchar_t,
    end: *mut *mut wchar_t,
    base: c_int,
    locale: _locale_t,
) -> __msvcrt_ulong {
    let mut ret = _wcstoi64_l(s, end, base, locale);

    if ret > ULONG_MAX {
        ret = ULONG_MAX;
        *_errno() = ERANGE as c_int;
    } else if ret < -ULONG_MAX {
        ret = 1;
        *_errno() = ERANGE as c_int;
    }
    ret as __msvcrt_ulong
}

/// `wcstoul (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn wcstoul(
    s: *const wchar_t,
    end: *mut *mut wchar_t,
    base: c_int,
) -> __msvcrt_ulong {
    _wcstoul_l(s, end, base, null_mut())
}

/// `_wtoi_l (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn _wtoi_l(str_: *const wchar_t, locale: _locale_t) -> c_int {
    let mut ret = _wcstoi64_l(str_, null_mut(), 10, locale);

    if ret > INT_MAX as i64 {
        ret = INT_MAX as i64;
        *_errno() = ERANGE as c_int;
    } else if ret < INT_MIN {
        ret = INT_MIN;
        *_errno() = ERANGE as c_int;
    }
    ret as c_int
}

/// `_wtoi (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn _wtoi(str_: *const wchar_t) -> c_int {
    _wtoi_l(str_, null_mut())
}

/// `_wtol_l (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn _wtol_l(str_: *const wchar_t, locale: _locale_t) -> __msvcrt_long {
    let mut ret = _wcstoi64_l(str_, null_mut(), 10, locale);

    if ret > LONG_MAX {
        ret = LONG_MAX;
        *_errno() = ERANGE as c_int;
    } else if ret < LONG_MIN {
        ret = LONG_MIN;
        *_errno() = ERANGE as c_int;
    }
    ret as __msvcrt_long
}

/// `_wtol (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn _wtol(str_: *const wchar_t) -> __msvcrt_long {
    _wtol_l(str_, null_mut())
}

/// `_wtoi64_l (MSVCRT.@)`
///
/// A completely separate parser from `_wcstoi64_l`, and a much older one: no base
/// support, no `endptr`, no digit-folding table, and — the part that matters — **no
/// overflow check**. `"99999999999999999999"` comes back wrapped, not saturated, and
/// errno is untouched. Every arithmetic step below is spelled `wrapping_*` so the Rust
/// reproduces that instead of trapping on it.
#[no_mangle]
pub unsafe extern "C" fn _wtoi64_l(str_: *const wchar_t, locale: _locale_t) -> i64 {
    let mut running_total: u64 = 0;
    let mut b_minus = false;
    let mut str_ = str_;

    while crate::_iswspace_l(*str_ as u32, locale as *mut c_void) != 0 {
        str_ = str_.add(1);
    }

    if *str_ == b'+' as wchar_t {
        str_ = str_.add(1);
    } else if *str_ == b'-' as wchar_t {
        b_minus = true;
        str_ = str_.add(1);
    }

    while *str_ >= b'0' as wchar_t && *str_ <= b'9' as wchar_t {
        running_total = running_total
            .wrapping_mul(10)
            .wrapping_add(*str_ as u64)
            .wrapping_sub(b'0' as u64);
        str_ = str_.add(1);
    }

    if b_minus {
        running_total.wrapping_neg() as i64
    } else {
        running_total as i64
    }
}

/// `_wtoi64 (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn _wtoi64(str_: *const wchar_t) -> i64 {
    _wtoi64_l(str_, null_mut())
}

// ═════════════════════════════════════════════════════════════════════════════════════
// The locale-taking case folders and comparers
// ═════════════════════════════════════════════════════════════════════════════════════
//
// Their plain twins (`_wcsupr`, `_wcsicmp`, …) are already Rust in `lib.rs` and do NOT
// route through these — wave 8 transcribed them separately against a resolved `locinfo`.
// The bodies here are the `_l` entry points other msvcrt TUs and applications call
// directly, so they must exist independently and must agree.
//
// Two shared quirks, both from the C:
//   * `free_locale_noalloc(&tmp)` runs even when the caller PASSED a locale and `tmp` is
//     still the zeroed struct. locale.c tolerates that; dropping the call would leak the
//     reference taken in the other branch.
//   * the folders MUTATE BEFORE THEY FAIL — they fold as they scan, so a buffer with no
//     terminator inside `n` comes back with its first `n` chars folded and then `str[0]`
//     truncated. The narrow `_strlwr_s_l` scans first and never touches the buffer.

/// `_wcsupr_s_l (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn _wcsupr_s_l(str_: *mut wchar_t, n: size_t, locale: _locale_t) -> c_int {
    let mut tmp = zeroed_locale();
    let mut ptr = str_;
    let mut n = n;
    let mut locale = locale;

    if str_.is_null() || n == 0 {
        if !str_.is_null() {
            *str_ = 0;
        }
        // errno only — the C sets it by hand here rather than through CHECK_PMT, so no
        // invalid-parameter handler fires.
        *_errno() = EINVAL as c_int;
        return EINVAL as c_int;
    }

    if locale.is_null() {
        locale = get_current_locale_noalloc(&mut tmp);
    }

    while n != 0 {
        n -= 1;
        if *ptr == 0 {
            free_locale_noalloc(&mut tmp);
            return 0;
        }
        *ptr = crate::_towupper_l(*ptr, locale as *mut c_void);
        ptr = ptr.add(1);
    }

    free_locale_noalloc(&mut tmp);

    /* MSDN claims that the function should return and set errno to
     * ERANGE, which doesn't seem to be true based on the tests. */
    *str_ = 0;
    *_errno() = EINVAL as c_int;
    EINVAL as c_int
}

/// `_wcsupr_l (MSVCRT.@)` — `n = (size_t)-1` means the terminator is always found, so
/// the EINVAL tail is unreachable and the return value is discarded.
#[no_mangle]
pub unsafe extern "C" fn _wcsupr_l(str_: *mut wchar_t, locale: _locale_t) -> *mut wchar_t {
    _wcsupr_s_l(str_, MINUS_ONE, locale);
    str_
}

/// `_wcslwr_s_l (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn _wcslwr_s_l(str_: *mut wchar_t, n: size_t, locale: _locale_t) -> c_int {
    let mut tmp = zeroed_locale();
    let mut ptr = str_;
    let mut n = n;
    let mut locale = locale;

    if str_.is_null() || n == 0 {
        if !str_.is_null() {
            *str_ = 0;
        }
        *_errno() = EINVAL as c_int;
        return EINVAL as c_int;
    }

    if locale.is_null() {
        locale = get_current_locale_noalloc(&mut tmp);
    }

    while n != 0 {
        n -= 1;
        if *ptr == 0 {
            free_locale_noalloc(&mut tmp);
            return 0;
        }
        *ptr = crate::_towlower_l(*ptr, locale as *mut c_void);
        ptr = ptr.add(1);
    }

    free_locale_noalloc(&mut tmp);

    /* MSDN claims that the function should return and set errno to
     * ERANGE, which doesn't seem to be true based on the tests. */
    *str_ = 0;
    *_errno() = EINVAL as c_int;
    EINVAL as c_int
}

/// `_wcslwr_l (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn _wcslwr_l(str_: *mut wchar_t, locale: _locale_t) -> *mut wchar_t {
    _wcslwr_s_l(str_, MINUS_ONE, locale);
    str_
}

/// `_wcsicmp_l (MSVCRT.@)`
///
/// Returns the RAW `wchar_t` difference — no clamp to -1/0/1, unlike `wcscmp`. The `||`
/// in the front door short-circuits, so a NULL `str1` fires the handler exactly once and
/// never evaluates the `str2` check.
#[no_mangle]
pub unsafe extern "C" fn _wcsicmp_l(
    str1: *const wchar_t,
    str2: *const wchar_t,
    locale: _locale_t,
) -> c_int {
    let mut tmp = zeroed_locale();
    let mut c1: wchar_t;
    let mut c2: wchar_t;
    let mut str1 = str1;
    let mut str2 = str2;
    let mut locale = locale;

    if MSVCRT_CHECK_PMT!(!str1.is_null()) == 0 || MSVCRT_CHECK_PMT!(!str2.is_null()) == 0 {
        return _NLSCMPERROR;
    }

    if locale.is_null() {
        locale = get_current_locale_noalloc(&mut tmp);
    }

    loop {
        c1 = crate::_towlower_l(*str1, locale as *mut c_void);
        str1 = str1.add(1);
        c2 = crate::_towlower_l(*str2, locale as *mut c_void);
        str2 = str2.add(1);
        if c1 == 0 || c1 != c2 {
            break;
        }
    }

    free_locale_noalloc(&mut tmp);
    c1 as c_int - c2 as c_int
}

/// `_wcsnicmp_l (MSVCRT.@)`
///
/// `!n` returns 0 BEFORE the front door, so `_wcsnicmp_l(NULL, NULL, 0)` is 0 and sets no
/// errno. `_wcsicmp_l` has no such early-out.
#[no_mangle]
pub unsafe extern "C" fn _wcsnicmp_l(
    str1: *const wchar_t,
    str2: *const wchar_t,
    n: size_t,
    locale: _locale_t,
) -> c_int {
    let mut tmp = zeroed_locale();
    let mut c1: wchar_t;
    let mut c2: wchar_t;
    let mut str1 = str1;
    let mut str2 = str2;
    let mut n = n;
    let mut locale = locale;

    if n == 0 {
        return 0;
    }

    if MSVCRT_CHECK_PMT!(!str1.is_null()) == 0 || MSVCRT_CHECK_PMT!(!str2.is_null()) == 0 {
        return _NLSCMPERROR;
    }

    if locale.is_null() {
        locale = get_current_locale_noalloc(&mut tmp);
    }

    // C: `do { ... } while(--n && c1 && (c1 == c2));` — `n != 0` on entry and on every
    // re-entry, so the decrement never wraps.
    loop {
        c1 = crate::_towlower_l(*str1, locale as *mut c_void);
        str1 = str1.add(1);
        c2 = crate::_towlower_l(*str2, locale as *mut c_void);
        str2 = str2.add(1);
        n -= 1;
        if n == 0 || c1 == 0 || c1 != c2 {
            break;
        }
    }

    free_locale_noalloc(&mut tmp);
    c1 as c_int - c2 as c_int
}

// ═════════════════════════════════════════════════════════════════════════════════════
// wide → multibyte
// ═════════════════════════════════════════════════════════════════════════════════════
//
// Every one of these branches on `locinfo->lc_codepage`. Codepage 0 is msvcrt's "raw"
// mode — a straight `wchar_t`→`char` truncation with an EILSEQ above 0xFF — and anything
// else goes through kernel32's `WideCharToMultiByte`. The `pused_default` dance is the
// same in all of them: UTF-8 has no best-fit substitution, so passing a
// `lpUsedDefaultChar` for CP_UTF8 is illegal and the C passes NULL instead.

/// `_wctomb_s_l (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn _wctomb_s_l(
    len: *mut c_int,
    mbchar: *mut c_char,
    size: size_t,
    wch: wchar_t,
    locale: _locale_t,
) -> c_int {
    let mut error: BOOL = 0;
    let locinfo: pthreadlocinfo;
    let mblen: c_int;

    if mbchar.is_null() && size > 0 {
        if !len.is_null() {
            *len = 0;
        }
        return 0;
    }

    if !len.is_null() {
        *len = -1;
    }

    if MSVCRT_CHECK_PMT!(size <= INT_MAX as size_t) == 0 {
        return EINVAL as c_int;
    }

    locinfo = locinfo_of(locale);

    if (*locinfo).lc_codepage == 0 {
        if wch as c_int > 0xff {
            if !mbchar.is_null() && size > 0 {
                memset(mbchar as *mut c_void, 0, size);
            }
            *_errno() = EILSEQ as c_int;
            return EILSEQ as c_int;
        }

        if MSVCRT_CHECK_PMT_ERR!(size >= 1, ERANGE as c_int) == 0 {
            return ERANGE as c_int;
        }

        *mbchar = wch as c_char;
        if !len.is_null() {
            *len = 1;
        }
        return 0;
    }

    let perror: *mut BOOL = if (*locinfo).lc_codepage != CP_UTF8 {
        &mut error
    } else {
        null_mut()
    };
    mblen = WideCharToMultiByte(
        (*locinfo).lc_codepage,
        0,
        &wch,
        1,
        mbchar,
        size as c_int,
        null(),
        perror,
    );
    if mblen == 0 || error != 0 {
        if mblen == 0 && GetLastError() == ERROR_INSUFFICIENT_BUFFER {
            if !mbchar.is_null() && size > 0 {
                memset(mbchar as *mut c_void, 0, size);
            }

            MSVCRT_INVALID_PMT!(
                b"insufficient buffer size\0".as_ptr() as *const c_char,
                ERANGE as c_int
            );
            return ERANGE as c_int;
        }

        *_errno() = EILSEQ as c_int;
        return EILSEQ as c_int;
    }

    if !len.is_null() {
        *len = mblen;
    }
    0
}

/// `wctomb_s (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn wctomb_s(
    len: *mut c_int,
    mbchar: *mut c_char,
    size: size_t,
    wch: wchar_t,
) -> c_int {
    _wctomb_s_l(len, mbchar, size, wch, null_mut())
}

/// `_wctomb_l (MSVCRT.@)`
///
/// This is the ReactOS `#ifdef __REACTOS__` arm, not Wine's: it sizes the destination
/// from `mb_cur_max` and maps a non-zero status to -1, where Wine passes `MB_LEN_MAX` and
/// ignores the status entirely.
#[no_mangle]
pub unsafe extern "C" fn _wctomb_l(dst: *mut c_char, ch: wchar_t, locale: _locale_t) -> c_int {
    let mut len: c_int = 0;
    let maxlen: c_int = if !locale.is_null() {
        (*(*locale).locinfo).mb_cur_max
    } else {
        (*get_locinfo()).mb_cur_max
    };
    if _wctomb_s_l(&mut len, dst, maxlen as size_t, ch, locale) != 0 {
        return -1;
    }
    len
}

/// `wctomb (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn wctomb(dst: *mut c_char, ch: wchar_t) -> c_int {
    _wctomb_l(dst, ch, null_mut())
}

/// `wctob (MSVCRT.@)`
///
/// `wchar < 0xff` — strictly less. U+00FF is representable in the raw codepage and the
/// truncation below would produce it, but this arm reports EOF for it anyway.
#[no_mangle]
pub unsafe extern "C" fn wctob(wchar: wint_t) -> c_int {
    let mut out: c_char = 0;
    let mut error: BOOL = 0;
    let codepage: UINT = (*get_locinfo()).lc_codepage;

    let perror: *mut BOOL = if codepage != CP_UTF8 {
        &mut error
    } else {
        null_mut()
    };

    if codepage == 0 {
        if (wchar as c_int) < 0xff {
            wchar as i8 as c_int
        } else {
            EOF
        }
    } else if WideCharToMultiByte(codepage, 0, &wchar, 1, &mut out, 1, null(), perror) != 0
        && error == 0
    {
        out as c_int
    } else {
        EOF
    }
}

/// `wcrtomb_s (MSVCRT.@)`
///
/// `*len` is written from an `int`, so the -1 that `wctomb_s` leaves on every failure
/// path becomes `(size_t)-1` in the caller, not 0xFFFFFFFF on a 64-bit target.
#[no_mangle]
pub unsafe extern "C" fn wcrtomb_s(
    len: *mut size_t,
    mbchar: *mut c_char,
    size: size_t,
    wch: wchar_t,
    s: *mut mbstate_t,
) -> c_int {
    let mut ilen: c_int = 0;
    let ret: c_int;

    if !s.is_null() {
        *s = 0;
    }
    ret = wctomb_s(&mut ilen, mbchar, size, wch);
    if !len.is_null() {
        *len = ilen as size_t;
    }
    ret
}

/// `wcrtomb (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn wcrtomb(dst: *mut c_char, ch: wchar_t, s: *mut mbstate_t) -> size_t {
    if !s.is_null() {
        *s = 0;
    }
    wctomb(dst, ch) as size_t
}

/// `wcsrtombs_l (INTERNAL)` — file-static in the C, therefore private here and NOT a
/// cohab override. The C `wcstombs` (still C) keeps calling its own copy; that is
/// duplicated code, not divergent behaviour.
unsafe fn wcsrtombs_l(
    mbstr: *mut c_char,
    wcstr: *mut *const wchar_t,
    count: size_t,
    locale: _locale_t,
) -> size_t {
    let locinfo: pthreadlocinfo;
    let mut tmp: size_t = 0;
    let mut used_default: BOOL = 0;

    locinfo = locinfo_of(locale);

    if (*locinfo).lc_codepage == 0 {
        if mbstr.is_null() {
            return crate::wcslen(*wcstr);
        }

        // The loop exits either by running out of `count` (i == count) or by the `break`
        // on the NUL — which leaves `i` AT the terminator, so the terminator is copied
        // but not counted. Both facts are read back below.
        let mut i: size_t = 0;
        while i < count {
            if *(*wcstr).add(i) > 255 {
                *_errno() = EILSEQ as c_int;
                return MINUS_ONE;
            }

            *mbstr.add(i) = *(*wcstr).add(i) as c_char;
            if *(*wcstr).add(i) == 0 {
                break;
            }
            i += 1;
        }

        if i < count {
            *wcstr = null();
        } else {
            *wcstr = (*wcstr).add(i);
        }
        return i;
    }

    let pused_default: *mut BOOL = if (*locinfo).lc_codepage != CP_UTF8 {
        &mut used_default
    } else {
        null_mut()
    };

    if mbstr.is_null() {
        tmp = WideCharToMultiByte(
            (*locinfo).lc_codepage,
            WC_NO_BEST_FIT_CHARS,
            *wcstr,
            -1,
            null_mut(),
            0,
            null(),
            pused_default,
        ) as size_t;
        if tmp == 0 || used_default != 0 {
            *_errno() = EILSEQ as c_int;
            return MINUS_ONE;
        }
        return tmp - 1;
    }

    while **wcstr != 0 {
        let mut buf: [c_char; 3] = [0; 3];

        let size = WideCharToMultiByte(
            (*locinfo).lc_codepage,
            WC_NO_BEST_FIT_CHARS,
            *wcstr,
            1,
            buf.as_mut_ptr(),
            3,
            null(),
            pused_default,
        ) as size_t;
        if size == 0 || used_default != 0 {
            *_errno() = EILSEQ as c_int;
            return MINUS_ONE;
        }
        if tmp + size > count {
            return tmp;
        }

        let mut i: size_t = 0;
        while i < size {
            *mbstr.add(tmp) = *buf.as_ptr().add(i);
            tmp += 1;
            i += 1;
        }
        *wcstr = (*wcstr).add(1);
    }

    if tmp < count {
        *mbstr.add(tmp) = 0;
        *wcstr = null();
    }
    tmp
}

/// `wcsrtombs_s_l (INTERNAL)` — file-static in the C, private here.
///
/// The `conv == size` arm reads `mbstr[conv-1]`. With `size == 0` and a non-NULL `mbstr`
/// that is `mbstr[-1]`: a genuine out-of-bounds touch in msvcrt, reproduced with wrapping
/// pointer arithmetic so it lands on the same byte instead of trapping on the `size_t`
/// subtraction.
unsafe fn wcsrtombs_s_l(
    ret: *mut size_t,
    mbstr: *mut c_char,
    size: size_t,
    wcstr: *mut *const wchar_t,
    count: size_t,
    locale: _locale_t,
) -> c_int {
    let mut conv: size_t;
    let mut err: c_int;

    if mbstr.is_null() && size == 0 && !wcstr.is_null() {
        conv = wcsrtombs_l(null_mut(), wcstr, 0, locale);
        if !ret.is_null() {
            // `conv+1` on the failure path is (size_t)-1 + 1 == 0, which is what the
            // caller sees. Modular, not saturating.
            *ret = conv.wrapping_add(1);
        }
        if conv == MINUS_ONE {
            return *_errno();
        }
        return 0;
    }

    if MSVCRT_CHECK_PMT!(!mbstr.is_null()) == 0 {
        return EINVAL as c_int;
    }
    if size != 0 {
        *mbstr = 0;
    }
    if MSVCRT_CHECK_PMT!(!wcstr.is_null()) == 0 {
        return EINVAL as c_int;
    }
    if MSVCRT_CHECK_PMT!(!(*wcstr).is_null()) == 0 {
        return EINVAL as c_int;
    }

    if count == _TRUNCATE || size < count {
        conv = size;
    } else {
        conv = count;
    }

    err = 0;
    conv = wcsrtombs_l(mbstr, wcstr, conv, locale);
    if conv == MINUS_ONE {
        conv = 0;
        if size != 0 {
            *mbstr = 0;
        }
        err = *_errno();
    } else if conv < size {
        *mbstr.add(conv) = 0;
        conv += 1;
    } else if conv == size
        && (count == _TRUNCATE || *mbstr.wrapping_add(conv.wrapping_sub(1)) == 0)
    {
        *mbstr.wrapping_add(conv.wrapping_sub(1)) = 0;
        if count == _TRUNCATE {
            err = STRUNCATE as c_int;
        }
    } else {
        MSVCRT_INVALID_PMT!(
            b"mbstr[size] is too small\0".as_ptr() as *const c_char,
            ERANGE as c_int
        );
        conv = 0;
        if size != 0 {
            *mbstr = 0;
        }
        err = ERANGE as c_int;
    }

    if !ret.is_null() {
        *ret = conv;
    }
    err
}

/// `_wcstombs_l (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn _wcstombs_l(
    mbstr: *mut c_char,
    wcstr: *const wchar_t,
    count: size_t,
    locale: _locale_t,
) -> size_t {
    let mut wcstr = wcstr;
    wcsrtombs_l(mbstr, &mut wcstr, count, locale)
}

/// `wcsrtombs (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn wcsrtombs(
    mbstr: *mut c_char,
    wcstr: *mut *const wchar_t,
    count: size_t,
    mbstate: *mut mbstate_t,
) -> size_t {
    if !mbstate.is_null() {
        *mbstate = 0;
    }

    wcsrtombs_l(mbstr, wcstr, count, null_mut())
}

/// `_wcstombs_s_l (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn _wcstombs_s_l(
    ret: *mut size_t,
    mbstr: *mut c_char,
    size: size_t,
    wcstr: *const wchar_t,
    count: size_t,
    locale: _locale_t,
) -> c_int {
    let mut wcstr = wcstr;
    wcsrtombs_s_l(ret, mbstr, size, &mut wcstr, count, locale)
}

/// `wcstombs_s (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn wcstombs_s(
    ret: *mut size_t,
    mbstr: *mut c_char,
    size: size_t,
    wcstr: *const wchar_t,
    count: size_t,
) -> c_int {
    let mut wcstr = wcstr;
    wcsrtombs_s_l(ret, mbstr, size, &mut wcstr, count, null_mut())
}

/// `wcsrtombs_s (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn wcsrtombs_s(
    ret: *mut size_t,
    mbstr: *mut c_char,
    size: size_t,
    wcstr: *mut *const wchar_t,
    count: size_t,
    mbstate: *mut mbstate_t,
) -> c_int {
    if !mbstate.is_null() {
        *mbstate = 0;
    }

    wcsrtombs_s_l(ret, mbstr, size, wcstr, count, null_mut())
}

// ── the last three of wcs.c ──────────────────────────────────────────────────
// Wave 16 left these behind because each delegates to something that was still C at the
// time. All three delegates are Rust now, so the wrappers are $0 hand-written rather
// than a paid re-translation of a file that already has output.

/// `_wcsdup (MSVCRT.@)`
///
/// Not `_strdup`'s shape despite reading like it: the C sizes the copy itself
/// (`(wcslen+1) * sizeof(wchar_t)`) and `memcpy`s, where `_strdup` calls `strcpy`. Kept
/// as written — a `wcscpy` here would be the same bytes today and a different function.
///
/// A NULL `str` is a quiet NULL return and a failed `malloc` returns NULL with whatever
/// errno malloc left; no `_errno` write and no invalid-parameter handler, exactly as the C.
#[no_mangle]
pub unsafe extern "C" fn _wcsdup(str_: *const wchar_t) -> *mut wchar_t {
    let mut ret: *mut wchar_t = null_mut();
    if !str_.is_null() {
        let size = (crate::wcslen(str_) + 1) * core::mem::size_of::<wchar_t>();
        ret = malloc(size) as *mut wchar_t;
        if !ret.is_null() {
            memcpy(ret as *mut c_void, str_ as *const c_void, size);
        }
    }
    ret
}

/// `dll/win32/msvcrt/include/wctype.h:71` — `typedef wchar_t wctrans_t;`. A typedef has no
/// symbol, so the island cannot snapshot it and it is spelled here against the header.
#[allow(non_camel_case_types)]
type wctrans_t = wchar_t;

/// `towctrans`
///
/// The C tests `category == 1` and nothing else: `wctrans("toupper")` returns 1 and
/// `wctrans("tolower")` returns 2, so every value that is not 1 — including a category
/// this msvcrt never issued — folds to LOWER. That is the C's contract, not a default
/// worth improving.
#[no_mangle]
pub unsafe extern "C" fn towctrans(c: wint_t, category: wctrans_t) -> wint_t {
    if category == 1 {
        return crate::_towupper_l(c, null_mut());
    }
    crate::_towlower_l(c, null_mut())
}

/// `wcstombs (MSVCRT.@)`
///
/// Takes `wcstr` by value and hands `wcsrtombs_l` a pointer to the local copy, so the
/// caller's pointer is never advanced — which is the whole difference between `wcstombs`
/// and `wcsrtombs`.
#[no_mangle]
pub unsafe extern "C" fn wcstombs(
    mbstr: *mut c_char,
    wcstr: *const wchar_t,
    count: size_t,
) -> size_t {
    let mut wcstr = wcstr;
    wcsrtombs_l(mbstr, &mut wcstr, count, null_mut())
}
