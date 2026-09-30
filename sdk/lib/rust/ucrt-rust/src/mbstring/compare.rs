//! The compare/collate family: `_mbscmp`, `_mbsicmp`, `_mbscoll`,
//! `_mbsicoll`, `_mbsnbcmp`, `_mbsnbicmp`, `_mbsnbcoll`, `_mbsnbicoll`,
//! `_mbsncmp`, `_mbsnicmp`, `_mbsncoll`, `_mbsnicoll` — plain, `_l`.
//!
//! ## Details that are silent if wrong
//!
//! * **`cmp` is ordinal, byte/codepoint order; `coll` calls `__acrt_CompareStringA`
//!   with `SORT_STRINGSORT` (`coll` adds `NORM_IGNORECASE` for the `i` forms).**
//!   Both fall to a plain `str*`/`_str*_l` primitive on `ismbcodepage == 0` —
//!   `strcmp`/`_stricmp_l`/`_strcoll_l`/`_stricoll_l` for the unbounded forms,
//!   `strncmp`/`_strnicmp`/`_strncoll_l`/`_strnicoll_l` for the bounded ones.
//!   The bounded `*cmp` forms pass `n` straight through even though `n` is a
//!   CHARACTER count for `_mbsncmp`/`_mbsnicmp` and a BYTE count for
//!   `_mbsnbcmp`/`_mbsnbicmp` — under codepage 0 a byte IS a character, so
//!   the C does not need to distinguish there, and neither do we.
//! * **The `coll` bounded forms convert a CHARACTER count to a BYTE count
//!   via `_mbsnbcnt_l` before calling `__acrt_CompareStringA`** — `_mbsncoll`/
//!   `_mbsnicoll` do this (`bcnt1`/`bcnt2`), `_mbsnbcoll`/`_mbsnbicoll` do not
//!   (their `n` is already bytes).
//! * **`_NLSCMPERROR` (`INT_MAX`) is returned on validation failure AND on a
//!   `__acrt_CompareStringA` failure** — the two are not the same "kind" of
//!   error in the C (one sets errno via `_VALIDATE_RETURN`, the other sets
//!   `errno = EINVAL` by hand) but they are indistinguishable from the return
//!   value alone, and this port keeps that.
//! * **The 2-arg `*cmp`/`*icmp` character loops build a 16-bit codepoint by
//!   shifting the lead byte up 8 and OR-ing the trail byte**, and treat a
//!   dangling lead byte (trail is NUL) as character 0 — i.e. as if the string
//!   had ended there, WITHOUT consuming the dangling lead byte from the other
//!   operand's cursor. This makes "trailing lead byte" sort as equal to a
//!   genuinely-ended string, by construction.
//! * **`_mbsnicmp`/`_mbsnbicmp`'s uppercasing is a closed-form range remap**
//!   (`_MBUPPERLOW1/2_MT` + `_MBCASEDIFF1/2_MT`), not a `__acrt_LCMapStringA`
//!   round-trip like `_mbsicmp` uses. Different mechanism, and for a
//!   character outside both ranges it is simply not remapped — no fallback.
//! * **`_mbsnbcmp`/`_mbsnbicmp`'s bounded loop can end on a "naked" lead byte
//!   exactly AT the byte budget** (`goto test` when `n==0` after consuming a
//!   lead byte) — the C treats that lead byte as if it were character 0
//!   *without* looking at whether the corresponding position in the other
//!   string is itself a lead byte needing the same treatment for `c2`; only
//!   `c1`'s "naked lead" arm re-checks `_ismbblead_l(*s2)` for `c2`, `c2`'s
//!   own "naked lead" arm (reached only when `c1` was NOT one) does not
//!   re-examine `s1`. Asymmetric, and transcribed as asymmetric.

use super::{Locale, EINVAL, NLSCMPERROR};
use crate::ismbbyte::_ismbblead_l;
use crate::locale_abi::__acrt_CompareStringA;
use crate::locale_update::LocaleUpdate;
use core::ffi::{c_char, c_int};

const SORT_STRINGSORT: u32 = 4096;
const NORM_IGNORECASE: u32 = 1;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    fn strcmp(a: *const u8, b: *const u8) -> c_int;
    fn strncmp(a: *const u8, b: *const u8, n: usize) -> c_int;
    fn _stricmp_l(a: *const u8, b: *const u8, locale: Locale) -> c_int;
    fn _strnicmp(a: *const u8, b: *const u8, n: usize) -> c_int;
    fn _strcoll_l(a: *const u8, b: *const u8, locale: Locale) -> c_int;
    fn _stricoll_l(a: *const u8, b: *const u8, locale: Locale) -> c_int;
    fn _strncoll_l(a: *const u8, b: *const u8, n: usize, locale: Locale) -> c_int;
    fn _strnicoll_l(a: *const u8, b: *const u8, n: usize, locale: Locale) -> c_int;
}

/// Reads the next codepoint from `*p` for the unbounded `cmp`/`icmp` loops,
/// advancing `*p`. A dangling lead byte (trail is NUL) reads as 0 without
/// advancing past the NUL.
#[inline]
unsafe fn next_code_cmp(p: &mut *const u8, l: Locale) -> u16 {
    let c = **p;
    *p = p.add(1);
    if _ismbblead_l(c as u32, l) != 0 {
        if **p == 0 {
            0
        } else {
            let c2 = **p;
            *p = p.add(1);
            ((c as u16) << 8) | c2 as u16
        }
    } else {
        c as u16
    }
}

// ── mbscmp.cpp ────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _mbscmp_l(s1: *const u8, s2: *const u8, locale: Locale) -> c_int {
    if s1.is_null() || s2.is_null() {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo(); // `_VALIDATE_RETURN`: the handler, after errno.
        return NLSCMPERROR;
    }
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return strcmp(s1.cast(), s2.cast());
    }
    let l = lu.locale_t();
    let (mut p1, mut p2) = (s1, s2);
    loop {
        let c1 = next_code_cmp(&mut p1, l);
        let c2 = next_code_cmp(&mut p2, l);
        if c1 != c2 {
            return if c1 > c2 { 1 } else { -1 };
        }
        if c1 == 0 {
            return 0;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn _mbscmp(s1: *const u8, s2: *const u8) -> c_int {
    _mbscmp_l(s1, s2, core::ptr::null_mut())
}

// ── mbsicmp.cpp ───────────────────────────────────────────────────────────

const LCMAP_UPPERCASE: u32 = 0x0200;

#[no_mangle]
pub unsafe extern "C" fn _mbsicmp_l(s1: *const u8, s2: *const u8, locale: Locale) -> c_int {
    if s1.is_null() || s2.is_null() {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo(); // `_VALIDATE_RETURN`: the handler, after errno.
        return NLSCMPERROR;
    }
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return _stricmp_l(s1.cast(), s2.cast(), lu.locale_t());
    }
    let l = lu.locale_t();
    let mb = lu.mbcinfo();
    let mblocalename = (*mb).mblocalename;
    let codepage = (*mb).mbcodepage;
    let (mut p1, mut p2) = (s1, s2);

    unsafe fn upper_next(p: &mut *const u8, l: Locale, mblocalename: *const u16, codepage: c_int) -> Result<u16, ()> {
        let c = **p;
        *p = p.add(1);
        if _ismbblead_l(c as u32, l) != 0 {
            if **p == 0 {
                return Ok(0);
            }
            let mut result = [0u8; 4];
            let src = [c, **p];
            *p = p.add(1);
            let retval = acrt_lcmap_string_a_upper(l, mblocalename, src.as_ptr().cast(), &mut result, codepage);
            match retval {
                1 => Ok(result[0] as u16),
                2 => Ok(((result[0] as u16) << 8) + result[1] as u16),
                _ => Err(()),
            }
        } else {
            Ok(super::mbbtolower(c as u32, l) as u16)
        }
    }

    // Thin wrapper matching the C's `__acrt_LCMapStringA(..., LCMAP_UPPERCASE, s-1, 2, result, 2, cp, TRUE)`.
    unsafe fn acrt_lcmap_string_a_upper(l: Locale, mblocalename: *const u16, src: *const u8, dst: &mut [u8; 4], codepage: c_int) -> c_int {
        crate::locale_abi::__acrt_LCMapStringA(l, mblocalename, LCMAP_UPPERCASE, src.cast(), 2, dst.as_mut_ptr().cast(), 2, codepage, 1)
    }

    loop {
        let c1 = match upper_next(&mut p1, l, mblocalename, codepage) {
            Ok(v) => v,
            Err(()) => {
                *_errno() = EINVAL;
                return NLSCMPERROR;
            }
        };
        let c2 = match upper_next(&mut p2, l, mblocalename, codepage) {
            Ok(v) => v,
            Err(()) => {
                *_errno() = EINVAL;
                return NLSCMPERROR;
            }
        };
        if c1 != c2 {
            return if c1 > c2 { 1 } else { -1 };
        }
        if c1 == 0 {
            return 0;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn _mbsicmp(s1: *const u8, s2: *const u8) -> c_int {
    _mbsicmp_l(s1, s2, core::ptr::null_mut())
}

// ── mbscoll.cpp / mbsicoll.cpp ────────────────────────────────────────────

unsafe fn coll_common(s1: *const u8, s2: *const u8, extra_flags: u32, locale: Locale) -> c_int {
    if s1.is_null() || s2.is_null() {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo(); // `_VALIDATE_RETURN`: the handler, after errno.
        return NLSCMPERROR;
    }
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return if extra_flags == 0 {
            _strcoll_l(s1.cast(), s2.cast(), lu.locale_t())
        } else {
            _stricoll_l(s1.cast(), s2.cast(), lu.locale_t())
        };
    }
    let l = lu.locale_t();
    let mb = lu.mbcinfo();
    let ret = __acrt_CompareStringA(
        l,
        (*mb).mblocalename,
        SORT_STRINGSORT | extra_flags,
        s1.cast(),
        -1,
        s2.cast::<c_char>(),
        -1,
        (*mb).mbcodepage,
    );
    if ret == 0 {
        *_errno() = EINVAL;
        return NLSCMPERROR;
    }
    ret - 2
}

#[no_mangle]
pub unsafe extern "C" fn _mbscoll_l(s1: *const u8, s2: *const u8, locale: Locale) -> c_int {
    coll_common(s1, s2, 0, locale)
}

#[no_mangle]
pub unsafe extern "C" fn _mbscoll(s1: *const u8, s2: *const u8) -> c_int {
    _mbscoll_l(s1, s2, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsicoll_l(s1: *const u8, s2: *const u8, locale: Locale) -> c_int {
    coll_common(s1, s2, NORM_IGNORECASE, locale)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsicoll(s1: *const u8, s2: *const u8) -> c_int {
    _mbsicoll_l(s1, s2, core::ptr::null_mut())
}

// ── mbsnbcmp.cpp / mbsnbicm.cpp (byte-bounded) ────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcmp_l(s1: *const u8, s2: *const u8, mut n: usize, locale: Locale) -> c_int {
    if n == 0 {
        return 0;
    }
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return strncmp(s1.cast(), s2.cast(), n);
    }
    if s1.is_null() || s2.is_null() {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo(); // `_VALIDATE_RETURN`: the handler, after errno.
        return NLSCMPERROR;
    }
    let l = lu.locale_t();
    let (mut p1, mut p2) = (s1, s2);
    while n > 0 {
        n -= 1;
        let mut c1 = *p1 as u16;
        p1 = p1.add(1);
        if _ismbblead_l(c1 as u32, l) != 0 {
            if n == 0 {
                // `goto test` with c1 forced to 0: always terminates (c1==0
                // forces a `return` on either arm of the final test below).
                let c2 = if _ismbblead_l(*p2 as u32, l) != 0 { 0 } else { *p2 as u16 };
                return if c2 != 0 { -1 } else { 0 };
            }
            c1 = if *p1 == 0 {
                0
            } else {
                let lo = *p1 as u16;
                p1 = p1.add(1);
                (c1 << 8) | lo
            };
        }
        let mut c2 = *p2 as u16;
        p2 = p2.add(1);
        if _ismbblead_l(c2 as u32, l) != 0 {
            if n == 0 {
                // `goto test` with c2 forced to 0: always terminates.
                return if c1 != 0 { 1 } else { 0 };
            }
            n -= 1;
            c2 = if *p2 == 0 {
                0
            } else {
                let lo = *p2 as u16;
                p2 = p2.add(1);
                (c2 << 8) | lo
            };
        }
        if c1 != c2 {
            return if c1 > c2 { 1 } else { -1 };
        }
        if c1 == 0 {
            return 0;
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcmp(s1: *const u8, s2: *const u8, n: usize) -> c_int {
    _mbsnbcmp_l(s1, s2, n, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbicmp_l(s1: *const u8, s2: *const u8, mut n: usize, locale: Locale) -> c_int {
    if n == 0 {
        return 0;
    }
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return _strnicmp(s1.cast(), s2.cast(), n);
    }
    if s1.is_null() || s2.is_null() {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo(); // `_VALIDATE_RETURN`: the handler, after errno.
        return NLSCMPERROR;
    }
    let l = lu.locale_t();
    let (mut p1, mut p2) = (s1, s2);
    while n > 0 {
        n -= 1;
        let mut c1 = *p1 as u16;
        p1 = p1.add(1);
        let mut naked1 = false;
        if _ismbblead_l(c1 as u32, l) != 0 {
            if n == 0 {
                naked1 = true;
            } else if *p1 == 0 {
                c1 = 0;
            } else {
                let lo = *p1 as u16;
                p1 = p1.add(1);
                c1 = (c1 << 8) | lo;
                c1 = mbcase_upperify(c1, l);
            }
        } else {
            c1 = super::mbbtolower(c1 as u32, l) as u16;
        }
        if naked1 {
            let c1v = 0u16;
            let c2v = if _ismbblead_l(*p2 as u32, l) != 0 { 0 } else { *p2 as u16 };
            return if c1v != c2v { if c1v > c2v { 1 } else { -1 } } else { 0 };
        }

        let mut c2 = *p2 as u16;
        p2 = p2.add(1);
        if _ismbblead_l(c2 as u32, l) != 0 {
            if n == 0 {
                c2 = 0;
                return if c1 != c2 { if c1 > c2 { 1 } else { -1 } } else { 0 };
            }
            n -= 1;
            if *p2 == 0 {
                c2 = 0;
            } else {
                let lo = *p2 as u16;
                p2 = p2.add(1);
                c2 = (c2 << 8) | lo;
                c2 = mbcase_upperify(c2, l);
            }
        } else {
            c2 = super::mbbtolower(c2 as u32, l) as u16;
        }

        if c1 != c2 {
            return if c1 > c2 { 1 } else { -1 };
        }
        if c1 == 0 {
            return 0;
        }
    }
    0
}

/// `_MBUPPERLOW1/2_MT` + `_MBCASEDIFF1/2_MT` remap used by `_mbsnicmp`'s and
/// `_mbsnbicmp`'s DBCS arm (`corecrt_internal_mbstring.h:38-45`).
#[inline]
unsafe fn mbcase_upperify(c: u16, l: Locale) -> u16 {
    let mb = (*l).mbcinfo;
    let ul = (*mb).mbulinfo;
    if c >= ul[0] && c <= ul[1] {
        c.wrapping_add(ul[2])
    } else if c >= ul[3] && c <= ul[4] {
        c.wrapping_add(ul[5])
    } else {
        c
    }
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbicmp(s1: *const u8, s2: *const u8, n: usize) -> c_int {
    _mbsnbicmp_l(s1, s2, n, core::ptr::null_mut())
}

// ── mbsncmp.cpp / mbsnicmp.cpp (char-bounded) ─────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _mbsncmp_l(s1: *const u8, s2: *const u8, mut n: usize, locale: Locale) -> c_int {
    let mut lu = LocaleUpdate::new(locale);
    if n == 0 {
        return 0;
    }
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return strncmp(s1.cast(), s2.cast(), n);
    }
    if s1.is_null() || s2.is_null() {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo(); // `_VALIDATE_RETURN`: the handler, after errno.
        return NLSCMPERROR;
    }
    let l = lu.locale_t();
    let (mut p1, mut p2) = (s1, s2);
    while n > 0 {
        n -= 1;
        let c1 = next_code_cmp(&mut p1, l);
        let c2 = next_code_cmp(&mut p2, l);
        if c1 != c2 {
            return if c1 > c2 { 1 } else { -1 };
        }
        if c1 == 0 {
            return 0;
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _mbsncmp(s1: *const u8, s2: *const u8, n: usize) -> c_int {
    _mbsncmp_l(s1, s2, n, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnicmp_l(s1: *const u8, s2: *const u8, mut n: usize, locale: Locale) -> c_int {
    if n == 0 {
        return 0;
    }
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return _strnicmp(s1.cast(), s2.cast(), n);
    }
    if s1.is_null() || s2.is_null() {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo(); // `_VALIDATE_RETURN`: the handler, after errno.
        return NLSCMPERROR;
    }
    let l = lu.locale_t();
    let (mut p1, mut p2) = (s1, s2);
    while n > 0 {
        n -= 1;
        let mut c1 = *p1 as u16;
        p1 = p1.add(1);
        if _ismbblead_l(c1 as u32, l) != 0 {
            if *p1 == 0 {
                c1 = 0;
            } else {
                let lo = *p1 as u16;
                p1 = p1.add(1);
                c1 = mbcase_upperify((c1 << 8) | lo, l);
            }
        } else {
            c1 = super::mbbtolower(c1 as u32, l) as u16;
        }
        let mut c2 = *p2 as u16;
        p2 = p2.add(1);
        if _ismbblead_l(c2 as u32, l) != 0 {
            if *p2 == 0 {
                c2 = 0;
            } else {
                let lo = *p2 as u16;
                p2 = p2.add(1);
                c2 = mbcase_upperify((c2 << 8) | lo, l);
            }
        } else {
            c2 = super::mbbtolower(c2 as u32, l) as u16;
        }
        if c1 != c2 {
            return if c1 > c2 { 1 } else { -1 };
        }
        if c1 == 0 {
            return 0;
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnicmp(s1: *const u8, s2: *const u8, n: usize) -> c_int {
    _mbsnicmp_l(s1, s2, n, core::ptr::null_mut())
}

// ── mbsnbcol.cpp / mbsnbico.cpp (byte-bounded coll) ───────────────────────

unsafe fn nbcoll_common(s1: *const u8, s2: *const u8, n: usize, extra_flags: u32, locale: Locale) -> c_int {
    if n == 0 {
        return 0;
    }
    if s1.is_null() || s2.is_null() || n > c_int::MAX as usize {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo(); // `_VALIDATE_RETURN`: the handler, after errno.
        return NLSCMPERROR;
    }
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return if extra_flags == 0 {
            _strncoll_l(s1.cast(), s2.cast(), n, lu.locale_t())
        } else {
            _strnicoll_l(s1.cast(), s2.cast(), n, lu.locale_t())
        };
    }
    let l = lu.locale_t();
    let mb = lu.mbcinfo();
    let ret = __acrt_CompareStringA(
        l,
        (*mb).mblocalename,
        SORT_STRINGSORT | extra_flags,
        s1.cast(),
        n as c_int,
        s2.cast::<c_char>(),
        n as c_int,
        (*mb).mbcodepage,
    );
    if ret == 0 {
        *_errno() = EINVAL;
        return NLSCMPERROR;
    }
    ret - 2
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcoll_l(s1: *const u8, s2: *const u8, n: usize, locale: Locale) -> c_int {
    nbcoll_common(s1, s2, n, 0, locale)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcoll(s1: *const u8, s2: *const u8, n: usize) -> c_int {
    _mbsnbcoll_l(s1, s2, n, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbicoll_l(s1: *const u8, s2: *const u8, n: usize, locale: Locale) -> c_int {
    nbcoll_common(s1, s2, n, NORM_IGNORECASE, locale)
}

/// NOTE: the C's `_mbsnbicoll` (unlike every other plain wrapper here) is
/// declared and defined directly `extern "C"`, not via the `(__cdecl f)(...)`
/// parenthesised-name idiom the rest of the file uses — a cosmetic C detail
/// with no ABI consequence, kept here only as a comment since Rust has no
/// equivalent syntax to preserve.
#[no_mangle]
pub unsafe extern "C" fn _mbsnbicoll(s1: *const u8, s2: *const u8, n: usize) -> c_int {
    _mbsnbicoll_l(s1, s2, n, core::ptr::null_mut())
}

// ── mbsncoll.cpp / mbsnicol.cpp (char-bounded coll) ───────────────────────

unsafe fn ncoll_common(s1: *const u8, s2: *const u8, n: usize, extra_flags: u32, locale: Locale) -> c_int {
    if n == 0 {
        return 0;
    }
    if s1.is_null() || s2.is_null() || n > c_int::MAX as usize {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo(); // `_VALIDATE_RETURN`: the handler, after errno.
        return NLSCMPERROR;
    }
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return if extra_flags == 0 {
            _strncoll_l(s1.cast(), s2.cast(), n, lu.locale_t())
        } else {
            _strnicoll_l(s1.cast(), s2.cast(), n, lu.locale_t())
        };
    }
    let l = lu.locale_t();
    let bcnt1 = _mbsnbcnt_via(s1, n, l);
    let bcnt2 = _mbsnbcnt_via(s2, n, l);
    let mb = lu.mbcinfo();
    let ret = __acrt_CompareStringA(
        l,
        (*mb).mblocalename,
        SORT_STRINGSORT | extra_flags,
        s1.cast(),
        bcnt1 as c_int,
        s2.cast::<c_char>(),
        bcnt2 as c_int,
        (*mb).mbcodepage,
    );
    if ret == 0 {
        *_errno() = EINVAL;
        return NLSCMPERROR;
    }
    ret - 2
}

#[inline]
unsafe fn _mbsnbcnt_via(s: *const u8, n: usize, l: Locale) -> usize {
    super::nav::_mbsnbcnt_l(s, n, l)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsncoll_l(s1: *const u8, s2: *const u8, n: usize, locale: Locale) -> c_int {
    ncoll_common(s1, s2, n, 0, locale)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsncoll(s1: *const u8, s2: *const u8, n: usize) -> c_int {
    _mbsncoll_l(s1, s2, n, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnicoll_l(s1: *const u8, s2: *const u8, n: usize, locale: Locale) -> c_int {
    ncoll_common(s1, s2, n, NORM_IGNORECASE, locale)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnicoll(s1: *const u8, s2: *const u8, n: usize) -> c_int {
    _mbsnicoll_l(s1, s2, n, core::ptr::null_mut())
}
