//! Pointer/length navigation: `_mbslen`, `_mbsnlen` (from `mbslen_s.cpp` —
//! the file name says `_s`, the export does not), `_mbsinc`, `_mbsninc`,
//! `_mbsdec`, `_mbsnextc`, `_mbsnbcnt`, `_mbsnccnt`. Byte classification
//! (`_ismbblead_l` etc.) and `_mbsbtype_l` are `crate::ismbbyte` /
//! `crate::ismbc` — called, not reimplemented.
//!
//! ## Details that are silent if wrong
//!
//! * **`_mbslen`/`_mbsnlen` fall to `strlen`/`strnlen` on `ismbcodepage==0`.**
//!   Every other function here has no such fast path in the C (only the
//!   comparison/search/case families do) — transcribed faithfully: `_mbsinc`,
//!   `_mbsdec`, `_mbsnextc`, `_mbsnbcnt`, `_mbsnccnt` and `_mbsninc` always
//!   consult `_ismbblead_l`, even under codepage 0 where it can never answer
//!   true.
//! * **A dangling lead byte (lead then NUL) is never walked past.** `_mbsinc`
//!   /`_mbsnextc`/`_mbslen`/`_mbsnbcnt`/`_mbsnccnt` all stop AT the lead byte
//!   in that case rather than reading a byte past the NUL.
//! * **`_mbsdec` has no `ismbcodepage` shortcut for the loop itself** — the
//!   `== 0` arm returns `current - 1` directly, but the multibyte arm's
//!   backward-parity scan is unconditional once codepage != 0, exactly the
//!   comment in the C about why the lead-byte-of-`current-1` optimisation was
//!   removed.
//! * **`_mbsnlen`'s `sizeInBytes` is a BYTE budget, not a character budget**,
//!   and a lead byte consumes 2 of it; hitting the budget mid-pair returns
//!   `sizeInBytes` (not `n`), same as running off the end of `s` without a NUL
//!   would if there were no budget.
//! * **`_mbsninc`/`_mbsnbcnt` truncate the `size_t` byte count through
//!   `(unsigned int)`** (`_mbsninc_l`'s C: `(unsigned int)_mbsnbcnt_l(...)`)
//!   before adding it to the pointer — reproduced with an explicit `as u32`
//!   so a >4GiB pointer offset silently wraps exactly as the C's does, not a
//!   Rust panic from an inferred wider add.

use super::Locale;
use crate::ismbbyte::_ismbblead_l;
use crate::locale_update::LocaleUpdate;
use core::ffi::{c_char, c_int};

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    fn strlen(s: *const c_char) -> usize;
    fn strnlen(s: *const c_char, maxlen: usize) -> usize;
}

// ── mbslen.cpp ────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _mbslen_l(mut s: *const u8, locale: Locale) -> usize {
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return strlen(s.cast());
    }
    let l = lu.locale_t();
    let mut n = 0usize;
    while *s != 0 {
        if _ismbblead_l(*s as u32, l) != 0 {
            s = s.add(1);
            if *s == 0 {
                break;
            }
        }
        n += 1;
        s = s.add(1);
    }
    n
}

#[no_mangle]
pub unsafe extern "C" fn _mbslen(s: *const u8) -> usize {
    _mbslen_l(s, core::ptr::null_mut())
}

// ── mbslen_s.cpp (exports are `_mbsnlen*`, not `_mbslen_s*`) ─────────────

#[no_mangle]
pub unsafe extern "C" fn _mbsnlen_l(mut s: *const u8, size_in_bytes: usize, locale: Locale) -> usize {
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return strnlen(s.cast(), size_in_bytes);
    }
    let l = lu.locale_t();
    let mut n = 0usize;
    let mut size = 0usize;
    while size < size_in_bytes && *s != 0 {
        if _ismbblead_l(*s as u32, l) != 0 {
            size += 1;
            if size >= size_in_bytes {
                break;
            }
            s = s.add(1);
            if *s == 0 {
                break;
            }
        }
        n += 1;
        s = s.add(1);
        size += 1;
    }
    if size >= size_in_bytes { size_in_bytes } else { n }
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnlen(s: *const u8, maxsize: usize) -> usize {
    _mbsnlen_l(s, maxsize, core::ptr::null_mut())
}

// ── mbsinc.cpp ────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _mbsinc_l(current: *const u8, locale: Locale) -> *mut u8 {
    let mut cur = current;
    let c = *cur;
    cur = cur.add(1);
    if _ismbblead_l(c as u32, locale) != 0 && *cur != 0 {
        cur = cur.add(1);
    }
    cur as *mut u8
}

#[no_mangle]
pub unsafe extern "C" fn _mbsinc(current: *const u8) -> *mut u8 {
    if current.is_null() {
        *_errno() = super::EINVAL;
        _invalid_parameter_noinfo();
        return core::ptr::null_mut();
    }
    _mbsinc_l(current, core::ptr::null_mut())
}

// ── mbsninc.cpp ───────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _mbsninc_l(string: *const u8, ccnt: usize, locale: Locale) -> *mut u8 {
    if string.is_null() {
        return core::ptr::null_mut();
    }
    // C: `(unsigned int)_mbsnbcnt_l(...)` — truncate to 32 bits before the add.
    string.add(_mbsnbcnt_l(string, ccnt, locale) as u32 as usize) as *mut u8
}

#[no_mangle]
pub unsafe extern "C" fn _mbsninc(string: *const u8, ccnt: usize) -> *mut u8 {
    _mbsninc_l(string, ccnt, core::ptr::null_mut())
}

// ── mbsdec.cpp ────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _mbsdec_l(string: *const u8, current: *const u8, locale: Locale) -> *mut u8 {
    if string.is_null() || current.is_null() {
        *_errno() = super::EINVAL;
        _invalid_parameter_noinfo();
        return core::ptr::null_mut();
    }
    if string >= current {
        return core::ptr::null_mut();
    }
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return current.sub(1) as *mut u8;
    }
    let l = lu.locale_t();
    // C: `temp = current - 1; while ((string <= --temp) && ismbblead(*temp));`
    // -- `--temp` happens before either test, every iteration including the
    // first, so `temp` starts one PAST its C initial value here.
    let mut temp = current.sub(1);
    loop {
        temp = temp.sub(1);
        if string > temp {
            break;
        }
        if _ismbblead_l(*temp as u32, l) == 0 {
            break;
        }
    }
    let diff = current.offset_from(temp) as usize;
    current.sub(1).sub(diff & 1) as *mut u8
}

#[no_mangle]
pub unsafe extern "C" fn _mbsdec(string: *const u8, current: *const u8) -> *mut u8 {
    _mbsdec_l(string, current, core::ptr::null_mut())
}

// ── mbsnextc.cpp ──────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _mbsnextc_l(mut s: *const u8, locale: Locale) -> u32 {
    if s.is_null() {
        *_errno() = super::EINVAL;
        _invalid_parameter_noinfo();
        return 0;
    }
    let mut next: u32 = 0;
    if _ismbblead_l(*s as u32, locale) != 0 && *s.add(1) != 0 {
        next = (*s as u32) << 8;
        s = s.add(1);
    }
    next + (*s as u32)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnextc(s: *const u8) -> u32 {
    _mbsnextc_l(s, core::ptr::null_mut())
}

// ── mbsnbcnt.cpp ──────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcnt_l(string: *const u8, mut ccnt: usize, locale: Locale) -> usize {
    if string.is_null() && ccnt != 0 {
        *_errno() = super::EINVAL;
        _invalid_parameter_noinfo();
        return 0;
    }
    let mut p = string;
    // `for (p = string; (ccnt-- && *p); p++)`: test-then-decrement, so a
    // ccnt of 0 never enters the loop and is never decremented past.
    loop {
        if ccnt == 0 {
            break;
        }
        ccnt -= 1;
        if *p == 0 {
            break;
        }
        if _ismbblead_l(*p as u32, locale) != 0 {
            p = p.add(1);
            if *p == 0 {
                p = p.sub(1);
                break;
            }
        }
        p = p.add(1);
    }
    p as usize - string as usize
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcnt(string: *const u8, ccnt: usize) -> usize {
    _mbsnbcnt_l(string, ccnt, core::ptr::null_mut())
}

// ── mbsnccnt.cpp ──────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _mbsnccnt_l(string: *const u8, mut bcnt: usize, locale: Locale) -> usize {
    if string.is_null() && bcnt != 0 {
        *_errno() = super::EINVAL;
        _invalid_parameter_noinfo();
        return 0;
    }
    let mut p = string;
    let mut n = 0usize;
    loop {
        if bcnt == 0 {
            break;
        }
        bcnt -= 1;
        if *p == 0 {
            break;
        }
        if _ismbblead_l(*p as u32, locale) != 0 {
            if bcnt == 0 {
                break;
            }
            bcnt -= 1;
            p = p.add(1);
            if *p == 0 {
                break;
            }
        }
        n += 1;
        p = p.add(1);
    }
    n
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnccnt(string: *const u8, bcnt: usize) -> usize {
    _mbsnccnt_l(string, bcnt, core::ptr::null_mut())
}
