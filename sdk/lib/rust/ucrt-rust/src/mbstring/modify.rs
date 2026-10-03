//! In-place / copying mutators: `_mbccpy`, `_mbclen`, `_mbsncpy`, `_mbsnbcpy`,
//! `_mbsncat`, `_mbsnbcat`, `_mbsnset`, `_mbsnbset`, `_mbsset` — plain, `_l`.
//! (The `_s`/`_s_l` secure forms of several of these are `super::secure`.)
//!
//! ## Details that are silent if wrong
//!
//! * **`_mbccpy` is defined IN TERMS OF `_mbccpy_s_l`** (`_mbccpy_s_l(dst, 2,
//!   nullptr, src, plocinfo)`), not a hand-rolled copy — so it inherits that
//!   function's error behaviour (EILSEQ on a dangling lead byte) even though
//!   it has no error return of its own to report it through. `_mbccpy_s_l`
//!   lives in `super::secure`; this module calls it, does not reimplement it.
//! * **`_mbclen` never asserts, "too low level"**: a dangling lead byte at
//!   end of string reads as length 1, not 2 — reproduced with a plain
//!   boundary check, no debug-only assert path exists to skip.
//! * **`_mbsncpy`/`_mbsnbcpy` NUL-pad the remainder of `cnt`** if the source
//!   is shorter, and do **not** add a terminator if the source is exactly
//!   `cnt` (bytes/chars) or longer — `strncpy`'s contract, transcribed
//!   exactly, including that a caller who wants a terminated string must ask
//!   for `cnt - 1` and add it themselves.
//! * **`_mbsncat`/`_mbsnbcat` back up over a trailing lead byte in `dst`
//!   before appending** (`_ismbslead_l`/`_mbsbtype_l`), so appending onto a
//!   string that itself ends mid-character overwrites that dangling lead
//!   byte rather than appending after it.
//! * **`_mbsnbcat`'s byte-counted copy loop decrements `cnt` TWICE per DBCS
//!   character** (once per byte) but `_mbsncat`'s char-counted loop does not
//!   decrement `cnt` at all inside the DBCS arm (it is decremented once by
//!   the `while(cnt--)` for the whole character) — this is the byte-vs-char
//!   distinction made concrete, not a bug in either.
//! * **`_mbsnset`/`_mbsnbset`'s two-byte fill value is REJECTED if its low
//!   byte is `'\0'`**: `errno = EINVAL` and BOTH bytes of the fill value are
//!   forced to `' '` (0x20) instead — "so instead we fill the string with
//!   spaces... to ensure that no information leaks through unexpectedly" per
//!   the C's own comment. Reproduced verbatim, not "improved" to reject
//!   cleanly.
//! * **`_mbsnbset`'s odd-byte-remaining case pads with a single `' '`, not a
//!   lead byte** — filling stops one byte short of `count` in that case,
//!   which is why the function's own doc says count is only a MAXIMUM.
//! * **`_mbsnset`'s "leadbyte" tracking variable survives across BOTH the
//!   double-byte and single-byte arms** via `_ismbbtruelead_l(leadbyte, *string,
//!   ...)`, and after the fill loop, if the byte just past the fill region is
//!   non-NUL and the tracked state says the previous byte was a genuine lead
//!   byte, that byte is overwritten with `' '` to avoid orphaning a trail byte
//!   that used to pair with a lead byte the fill now removed. `_mbsnbset` has
//!   NO equivalent post-loop fixup — the two functions diverge here, not just
//!   in byte-vs-char counting.
//! * **`_mbsset`'s double-byte loop can silently drop the low byte only near
//!   NUL**, same "don't orphan lead byte" rule as `_mbsnset`'s double-byte
//!   arm, but `_mbsset` has no count budget to run out of, so its only stop
//!   condition is a NUL from the source string itself.

use super::{Locale, EINVAL};
use crate::ismbbyte::_ismbblead_l;
use crate::ismbc::_mbsbtype_l;
use crate::locale_update::LocaleUpdate;
use core::ffi::c_int;

const MBC_LEAD: c_int = super::MBC_LEAD;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    fn strncat(dst: *mut u8, src: *const u8, n: usize) -> *mut u8;
    fn strncpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8;
    fn _strnset(s: *mut u8, val: c_int, count: usize) -> *mut u8;
    fn _strset(s: *mut u8, val: c_int) -> *mut u8;
}

/// `_ismbbtruelead_l(_lb,_ch,p)` (`corecrt_internal_mbstring.h:31`):
/// `!(_lb) && _ismbblead_l((_ch), p)`.
#[inline]
unsafe fn ismbbtruelead(lb: bool, ch: u8, l: Locale) -> bool {
    !lb && _ismbblead_l(ch as u32, l) != 0
}

// ── mbclen.cpp ────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _mbclen_l(c: *const u8, locale: Locale) -> usize {
    if _ismbblead_l(*c as u32, locale) != 0 && *c.add(1) != 0 { 2 } else { 1 }
}

#[no_mangle]
pub unsafe extern "C" fn _mbclen(c: *const u8) -> usize {
    _mbclen_l(c, core::ptr::null_mut())
}

// ── mbccpy.cpp ────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _mbccpy_l(dst: *mut u8, src: *const u8, locale: Locale) {
    super::secure::_mbccpy_s_l(dst, 2, core::ptr::null_mut(), src, locale);
}

#[no_mangle]
pub unsafe extern "C" fn _mbccpy(dst: *mut u8, src: *const u8) {
    super::secure::_mbccpy_s_l(dst, 2, core::ptr::null_mut(), src, core::ptr::null_mut());
}

// ── mbsncpy.cpp / mbsnbcpy.cpp ────────────────────────────────────────────
// `bytes`: true selects the mbsnbcpy (byte-count) contract, false mbsncpy
// (char-count). The only difference is whether `cnt` is decremented once or
// twice for a DBCS character.

#[inline]
unsafe fn ncpy_common(dst: *mut u8, src: *const u8, mut cnt: usize, locale: Locale, bytes: bool) -> *mut u8 {
    let start = dst;
    if dst.is_null() && cnt != 0 {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo(); // `_VALIDATE_RETURN`: the handler, after errno.
        return core::ptr::null_mut();
    }
    if src.is_null() && cnt != 0 {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo(); // `_VALIDATE_RETURN`: the handler, after errno.
        return core::ptr::null_mut();
    }
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return strncpy(dst.cast(), src.cast(), cnt).cast();
    }
    let l = lu.locale_t();
    let (mut d, mut s) = (dst, src);
    while cnt != 0 {
        cnt -= 1;
        if _ismbblead_l(*s as u32, l) != 0 {
            *d = *s;
            d = d.add(1);
            s = s.add(1);
            // `mbsnbcpy` (bytes==true) checks the byte budget BEFORE
            // touching the trail byte at all (`if (!cnt) { dst[-1]=0; break; }`);
            // `mbsncpy` (bytes==false) has no such mid-character check.
            if bytes {
                if cnt == 0 {
                    *d.sub(1) = 0;
                    break;
                }
                cnt -= 1;
            }
            let b = *s;
            *d = b;
            d = d.add(1);
            s = s.add(1);
            if b == 0 {
                *d.sub(2) = 0;
                break;
            }
        } else {
            let b = *s;
            *d = b;
            d = d.add(1);
            s = s.add(1);
            if b == 0 {
                break;
            }
        }
    }
    while cnt > 0 {
        cnt -= 1;
        *d = 0;
        d = d.add(1);
    }
    start
}

#[no_mangle]
pub unsafe extern "C" fn _mbsncpy_l(dst: *mut u8, src: *const u8, cnt: usize, locale: Locale) -> *mut u8 {
    ncpy_common(dst, src, cnt, locale, false)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsncpy(dst: *mut u8, src: *const u8, cnt: usize) -> *mut u8 {
    _mbsncpy_l(dst, src, cnt, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcpy_l(dst: *mut u8, src: *const u8, cnt: usize, locale: Locale) -> *mut u8 {
    ncpy_common(dst, src, cnt, locale, true)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcpy(dst: *mut u8, src: *const u8, cnt: usize) -> *mut u8 {
    _mbsnbcpy_l(dst, src, cnt, core::ptr::null_mut())
}

// ── mbsncat.cpp / mbsnbcat.cpp ─────────────────────────────────────────────

#[inline]
unsafe fn ncat_common(dst: *mut u8, src: *const u8, mut cnt: usize, locale: Locale, bytes: bool) -> *mut u8 {
    if cnt == 0 {
        return dst;
    }
    if dst.is_null() || src.is_null() {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo(); // `_VALIDATE_RETURN`: the handler, after errno.
        return core::ptr::null_mut();
    }
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return strncat(dst.cast(), src.cast(), cnt).cast();
    }
    let l = lu.locale_t();
    let start = dst;
    let mut d = dst;
    while *d != 0 {
        d = d.add(1);
    }
    // Back up over a trailing lead byte, matching each file's own idiom:
    // mbsncat uses `_ismbslead_l`, mbsnbcat uses `_mbsbtype_l` -- both ask
    // "does the byte at d-1 begin an odd run of lead bytes back to start",
    // and are equivalent for a legal string; kept distinct to mirror source.
    if d != start {
        if bytes {
            if _mbsbtype_l(start, (d as usize - start as usize) - 1, l) == MBC_LEAD {
                d = d.sub(1);
            }
        } else if crate::ismbc::_ismbslead_l(start, d, l) != 0 {
            // NOTE: the C passes `dst` itself here (pointing at the NUL), not
            // `dst - 1` -- transcribed exactly as `mbsncat.cpp` calls it,
            // even though `_ismbslead_l`'s own contract (see `ismbc.rs`)
            // makes this a no-op in the "dangling lead before NUL" case: it
            // returns 0 there, not -1, so `_mbsncat`'s back-up branch this
            // port reproduces is likewise a no-op for that case. `mbsnbcat`
            // (the `bytes` arm above) checks `dst - 1` via `_mbsbtype_l`
            // instead and DOES catch it -- the two files genuinely differ.
            d = d.sub(1);
        }
    }
    let mut s = src;
    loop {
        if cnt == 0 {
            break;
        }
        cnt -= 1;
        if _ismbblead_l(*s as u32, l) != 0 {
            *d = *s;
            d = d.add(1);
            s = s.add(1);
            if bytes && cnt == 0 {
                *d.sub(1) = 0;
                break;
            }
            if bytes {
                cnt -= 1;
            }
            let b = *s;
            *d = b;
            d = d.add(1);
            s = s.add(1);
            if b == 0 {
                *d.sub(2) = 0;
                break;
            }
        } else {
            let b = *s;
            *d = b;
            d = d.add(1);
            s = s.add(1);
            if b == 0 {
                break;
            }
        }
    }
    if d != start && _mbsbtype_l(start, (d as usize - start as usize) - 1, l) == MBC_LEAD {
        *d.sub(1) = 0;
    } else {
        *d = 0;
    }
    start
}

#[no_mangle]
pub unsafe extern "C" fn _mbsncat_l(dst: *mut u8, src: *const u8, cnt: usize, locale: Locale) -> *mut u8 {
    ncat_common(dst, src, cnt, locale, false)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsncat(dst: *mut u8, src: *const u8, cnt: usize) -> *mut u8 {
    _mbsncat_l(dst, src, cnt, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcat_l(dst: *mut u8, src: *const u8, cnt: usize, locale: Locale) -> *mut u8 {
    ncat_common(dst, src, cnt, locale, true)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcat(dst: *mut u8, src: *const u8, cnt: usize) -> *mut u8 {
    _mbsnbcat_l(dst, src, cnt, core::ptr::null_mut())
}

// ── mbsnset.cpp / mbsnbset.cpp ─────────────────────────────────────────────
// `bytes`: true = mbsnbset (count is bytes, no post-loop orphan fixup),
// false = mbsnset (count is chars, HAS a post-loop orphan fixup via
// `leadbyte`). The two files genuinely diverge here -- see module doc.

#[no_mangle]
pub unsafe extern "C" fn _mbsnbset_l(string: *mut u8, val: u32, mut count: usize, locale: Locale) -> *mut u8 {
    let start = string;
    let lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return _strnset(string.cast(), val as c_int, count).cast();
    }
    if string.is_null() && count != 0 {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo(); // `_VALIDATE_RETURN`: the handler, after errno.
        return core::ptr::null_mut();
    }
    let highval = (val >> 8) as u8;
    let mut lowval = (val & 0xff) as u8;
    let mut hv = highval;
    if highval != 0 {
        if lowval == 0 {
            *_errno() = EINVAL;
            lowval = b' ';
            hv = b' ';
        }
        let mut p = string;
        while count != 0 && *p != 0 {
            count -= 1;
            if count == 0 || *p.add(1) == 0 {
                *p = b' ';
                break;
            }
            count -= 1;
            *p = hv;
            p = p.add(1);
            *p = lowval;
            p = p.add(1);
        }
    } else {
        let mut p = string;
        while count != 0 && *p != 0 {
            count -= 1;
            *p = val as u8;
            p = p.add(1);
        }
    }
    start
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbset(string: *mut u8, val: u32, count: usize) -> *mut u8 {
    _mbsnbset_l(string, val, count, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnset_l(string: *mut u8, val: u32, mut count: usize, locale: Locale) -> *mut u8 {
    let start = string;
    if string.is_null() && count != 0 {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo(); // `_VALIDATE_RETURN`: the handler, after errno.
        return core::ptr::null_mut();
    }
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return _strnset(string.cast(), val as c_int, count).cast();
    }
    let l = lu.locale_t();
    let mut leadbyte = false;
    let highval = (val >> 8) as u8;
    let mut lowval = (val & 0xff) as u8;
    let mut hv = highval;
    let mut p = string;
    if highval != 0 {
        if lowval == 0 {
            *_errno() = EINVAL;
            lowval = b' ';
            hv = b' ';
        }
        while count != 0 && *p != 0 {
            count -= 1;
            leadbyte = ismbbtruelead(leadbyte, *p, l);
            *p = hv;
            p = p.add(1);
            if *p != 0 {
                leadbyte = ismbbtruelead(leadbyte, *p, l);
                *p = lowval;
                p = p.add(1);
            } else {
                *p.sub(1) = b' ';
            }
        }
    } else {
        while count != 0 && *p != 0 {
            count -= 1;
            leadbyte = ismbbtruelead(leadbyte, *p, l);
            *p = val as u8;
            p = p.add(1);
        }
    }
    if leadbyte && *p != 0 {
        *p = b' ';
    }
    start
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnset(string: *mut u8, val: u32, count: usize) -> *mut u8 {
    _mbsnset_l(string, val, count, core::ptr::null_mut())
}

// ── mbsset.cpp ─────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _mbsset_l(string: *mut u8, val: u32, locale: Locale) -> *mut u8 {
    if string.is_null() {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo(); // `_VALIDATE_RETURN`: the handler, after errno.
        return core::ptr::null_mut();
    }
    let start = string;
    let lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return _strset(string.cast(), val as c_int).cast();
    }
    let highval = (val >> 8) as u8;
    let mut lowval = (val & 0xff) as u8;
    let mut hv = highval;
    let mut p = string;
    if highval != 0 {
        if lowval == 0 {
            *_errno() = EINVAL;
            lowval = b' ';
            hv = b' ';
        }
        while *p != 0 {
            *p = hv;
            p = p.add(1);
            if *p != 0 {
                *p = lowval;
                p = p.add(1);
            } else {
                *p.sub(1) = b' ';
            }
        }
    } else {
        while *p != 0 {
            *p = val as u8;
            p = p.add(1);
        }
    }
    start
}

#[no_mangle]
pub unsafe extern "C" fn _mbsset(string: *mut u8, val: u32) -> *mut u8 {
    _mbsset_l(string, val, core::ptr::null_mut())
}
