//! `_ismbc*` and friends — multibyte CHARACTER classification and the small
//! conversions built on it. Replaces, in full, these files of
//! `sdk/lib/ucrt/mbstring`: ismbalnm, ismbalph, ismbdgt, ismbgrph, ismbknj,
//! ismblgl, ismblwr, ismbprn, ismbpunc, ismbsle, ismbspc, ismbupr, mbbtype,
//! mbsbtype, mbclevel, mbtolwr, mbtoupr, mbtohira, mbtokata and tojisjms.
//!
//! ## A port, not msvcrt's bodies
//!
//! msvcrt-crt-rust already has every one of these, and they are the reason this
//! family showed as DIFF in the Tier 2 table: they are Wine's, and Wine maps
//! `_MB_CP_SBCS` to codepage 20127 where UCRT keeps codepage 0 and answers
//! single-byte questions from the C locale. That is a contract difference, not a
//! bug on either side, so the bodies here follow UCRT's C line for line over the
//! same `LocaleUpdate` the `_ismbb*` family uses.
//!
//! ## Details that are silent if wrong
//!
//! * **Return values are NOT normalised.** The `c <= 0xFF` arm returns whatever
//!   the single-byte primitive returns: `_ismbbalpha_l` gives 1, but
//!   `_ismbcdigit` gives the raw `_DIGIT` mask (4) through `_isdigit_fast_internal`
//!   and `_ismbcspace` whatever `_isspace_l` gives. The multibyte arm gives 1.
//! * **`_ismbcprint` tests `_CONTROL` with expected = false** — "printable" is
//!   "not a control character", the one inverted member.
//! * **The multibyte arm refuses a PAIR of single-byte characters.** `c` is split
//!   into two bytes and classified as a string; a nonzero second type means the
//!   high byte was not a lead byte and `c` was two characters, not one.
//! * **`_ismbcl0/1/2`, `_ismbchira/kata/symbol` and the JIS conversions are
//!   Shift-JIS only** and answer 0 (or return `c` unchanged) on any other
//!   codepage, including every SBCS one.
//! * **`_mbsbtype` loops `length + 1` times** (`do { } while (length--)`) and
//!   raises the invalid-parameter handler for a NUL met BEFORE the requested
//!   position, but returns `_MBC_ILLEGAL` quietly for a NUL AT it.
//!
//! ## The one deliberate divergence: an out-of-bounds read is not transcribed
//!
//! Inside UCRT's mbstring C, `_ismbblead_l`/`_ismbbtrail_l` (and every `_ismbb*_l`)
//! are MACROS from `corecrt_internal_mbstring.h`, not the exported functions:
//! `(mbctype + 1)[c] & _M1`, with no truncation to a byte. We call the exported
//! bodies, which truncate. For every argument the macros are handed here the two
//! agree -- a byte, or `c >> 8` of a `c <= 0xFFFF` -- and they are only ever
//! tested for truth, so the macro returning the raw mask bit instead of 1 is
//! invisible too. They DIFFER for `_ismbclegal(c)` with `c > 0xFFFF` (and
//! `_ismbcl0/1/2` on codepage 932): Microsoft indexes up to 16 MB past the
//! 257-byte table. `_ismbclegal(-1)` faults in Microsoft's object -- which is why
//! the Tier 2 table carries a FAULT for it on the `ref_` side, before and after
//! this takeover -- where ours answers from the low byte of `c >> 8`. Undefined
//! behaviour is not a contract, so it is not reproduced.

use crate::ctype::_isspace_l;
use crate::ismbbyte::{
    _ismbbalnum_l, _ismbbalpha_l, _ismbbblank_l, _ismbbgraph_l, _ismbblead_l, _ismbbprint_l,
    _ismbbpunct_l, _ismbbtrail_l,
};
use crate::locale::{CrtLocalePointers, CrtMultibyteData};
use crate::locale_update::LocaleUpdate;
use core::ffi::c_int;

// corecrt_wctype.h:48-58
const _UPPER: u16 = 0x01;
const _LOWER: u16 = 0x02;
const _DIGIT: u16 = 0x04;
const _SPACE: u16 = 0x08;
const _PUNCT: u16 = 0x10;
const _CONTROL: u16 = 0x20;
const _BLANK: u16 = 0x40;
const _ALPHA: u16 = 0x0100 | _UPPER | _LOWER;

// mbctype.h:49-50
const _SBUP: u8 = 0x10;
const _SBLOW: u8 = 0x20;

// mbctype.h:54-59
const _MBC_SINGLE: c_int = 0;
const _MBC_LEAD: c_int = 1;
const _MBC_TRAIL: c_int = 2;
const _MBC_ILLEGAL: c_int = -1;
const _KANJI_CP: c_int = 932;

const CT_CTYPE1: u32 = 0x0001;
const LCMAP_LOWERCASE: u32 = 0x0100;
const LCMAP_UPPERCASE: u32 = 0x0200;
const EINVAL: c_int = 22;
const EILSEQ: c_int = 42;

extern "C" {
    fn __acrt_GetStringTypeA(
        locale: *mut CrtLocalePointers,
        info_type: u32,
        src: *const u8,
        cch: c_int,
        char_type: *mut u16,
        code_page: c_int,
        error: c_int,
    ) -> c_int;
    fn __acrt_LCMapStringA(
        locale: *mut CrtLocalePointers,
        locale_name: *const u16,
        map_flags: u32,
        src: *const u8,
        cch_src: c_int,
        dest: *mut u8,
        cch_dest: c_int,
        code_page: c_int,
        error: c_int,
    ) -> c_int;
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
}

#[inline]
unsafe fn mbcinfo(locale: *mut CrtLocalePointers) -> *mut CrtMultibyteData {
    (*locale).mbcinfo
}

/// `__dcrt_multibyte_check_type` (corecrt_internal_mbstring.h:84).
unsafe fn multibyte_check_type(
    c: u32,
    locale: *mut CrtLocalePointers,
    category_bits: u16,
    expected: bool,
) -> c_int {
    let mb = mbcinfo(locale);
    if (*mb).ismbcodepage == 0 {
        return 0;
    }
    let code_page = (*mb).mbcodepage;
    let bytes = [((c >> 8) & 0xff) as u8, (c & 0xff) as u8];
    let mut ctypes = [0u16; 2];
    if __acrt_GetStringTypeA(locale, CT_CTYPE1, bytes.as_ptr(), 2, ctypes.as_mut_ptr(), code_page, 1)
        == 0
    {
        return 0;
    }
    if ctypes[1] != 0 {
        return 0;
    }
    ((ctypes[0] & category_bits != 0) == expected) as c_int
}

/// `_ctype_fast_check_internal`: the raw masked table value, not 1.
#[inline]
unsafe fn ctype_fast_check(c: u8, mask: u16, locale: *mut CrtLocalePointers) -> c_int {
    let pctype = (*(*(*locale).locinfo).public_ref()).locale_pctype;
    (*pctype.add(c as usize) & mask) as c_int
}

#[inline]
unsafe fn mbb_is(c: u32, bits: u8, locale: *mut CrtLocalePointers) -> bool {
    (*mbcinfo(locale)).mbctype[c as usize + 1] & bits == bits
}

/// The shared shape of every two-armed `_ismbc*_l`: `$single` answers
/// `c <= 0xFF` against the resolved locale, the multibyte arm tests `$bits`.
macro_rules! ismbc {
    ($name:ident, $name_l:ident, |$c:ident, $loc:ident| $single:expr, $bits:expr, $expected:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name_l(c: u32, locale: *mut CrtLocalePointers) -> c_int {
            let mut lu = LocaleUpdate::new(locale);
            let $loc = lu.locale_t();
            if c <= 0x00FF {
                let $c = c;
                return $single;
            }
            multibyte_check_type(c, $loc, $bits, $expected)
        }
        #[no_mangle]
        pub unsafe extern "C" fn $name(c: u32) -> c_int {
            $name_l(c, core::ptr::null_mut())
        }
    };
}

ismbc!(_ismbcalnum, _ismbcalnum_l, |c, l| _ismbbalnum_l(c, l), _ALPHA | _DIGIT, true);
ismbc!(_ismbcalpha, _ismbcalpha_l, |c, l| _ismbbalpha_l(c, l), _ALPHA, true);
ismbc!(_ismbcdigit, _ismbcdigit_l, |c, l| ctype_fast_check(c as u8, _DIGIT, l), _DIGIT, true);
ismbc!(_ismbcgraph, _ismbcgraph_l, |c, l| _ismbbgraph_l(c, l), _PUNCT | _ALPHA | _DIGIT, true);
ismbc!(_ismbclower, _ismbclower_l, |c, l| mbb_is(c, _SBLOW, l) as c_int, _LOWER, true);
ismbc!(_ismbcupper, _ismbcupper_l, |c, l| mbb_is(c, _SBUP, l) as c_int, _UPPER, true);
ismbc!(_ismbcprint, _ismbcprint_l, |c, l| _ismbbprint_l(c, l), _CONTROL, false);
ismbc!(_ismbcpunct, _ismbcpunct_l, |c, l| _ismbbpunct_l(c, l), _PUNCT, true);
ismbc!(_ismbcblank, _ismbcblank_l, |c, l| _ismbbblank_l(c, l), _BLANK, true);
ismbc!(_ismbcspace, _ismbcspace_l, |c, l| _isspace_l(c as c_int, l), _SPACE, true);

// ── ismblgl.cpp ──────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _ismbclegal_l(c: u32, locale: *mut CrtLocalePointers) -> c_int {
    let mut lu = LocaleUpdate::new(locale);
    let l = lu.locale_t();
    (_ismbblead_l(c >> 8, l) != 0 && _ismbbtrail_l(c & 0x0ff, l) != 0) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn _ismbclegal(c: u32) -> c_int {
    _ismbclegal_l(c, core::ptr::null_mut())
}

// ── ismbknj.cpp — Shift-JIS ranges ───────────────────────────────────────────

macro_rules! kanji_range {
    ($name:ident, $name_l:ident, |$c:ident| $test:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name_l($c: u32, locale: *mut CrtLocalePointers) -> c_int {
            let lu = LocaleUpdate::new(locale);
            ((*lu.mbcinfo()).mbcodepage == _KANJI_CP && $test) as c_int
        }
        #[no_mangle]
        pub unsafe extern "C" fn $name(c: u32) -> c_int {
            $name_l(c, core::ptr::null_mut())
        }
    };
}

kanji_range!(_ismbchira, _ismbchira_l, |c| (0x829f..=0x82f1).contains(&c));
kanji_range!(_ismbckata, _ismbckata_l, |c| (0x8340..=0x8396).contains(&c) && c != 0x837f);
kanji_range!(_ismbcsymbol, _ismbcsymbol_l, |c| (0x8141..=0x81ac).contains(&c) && c != 0x817f);

// ── mbclevel.cpp — JIS levels 0/1/2 ──────────────────────────────────────────

macro_rules! kanji_level {
    ($name:ident, $name_l:ident, |$c:ident| $range:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name_l($c: u32, locale: *mut CrtLocalePointers) -> c_int {
            let mut lu = LocaleUpdate::new(locale);
            let cp = (*lu.mbcinfo()).mbcodepage;
            let l = lu.locale_t();
            (cp == _KANJI_CP
                && _ismbblead_l($c >> 8, l) != 0
                && _ismbbtrail_l($c & 0x0ff, l) != 0
                && $range) as c_int
        }
        #[no_mangle]
        pub unsafe extern "C" fn $name(c: u32) -> c_int {
            $name_l(c, core::ptr::null_mut())
        }
    };
}

kanji_level!(_ismbcl0, _ismbcl0_l, |c| c < 0x889f);
kanji_level!(_ismbcl1, _ismbcl1_l, |c| (0x889f..=0x9872).contains(&c));
kanji_level!(_ismbcl2, _ismbcl2_l, |c| (0x989f..=0xEAA4).contains(&c));

// ── ismbsle.cpp ──────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _ismbslead_l(
    string: *const u8,
    current: *const u8,
    locale: *mut CrtLocalePointers,
) -> c_int {
    if string.is_null() || current.is_null() {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo();
        return 0;
    }
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return 0;
    }
    let l = lu.locale_t();
    let mut s = string;
    while s <= current && *s != 0 {
        if _ismbblead_l(*s as u32, l) != 0 {
            let was = s;
            s = s.add(1);
            if was == current {
                return -1;
            }
            if *s == 0 {
                return 0;
            }
        }
        s = s.add(1);
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _ismbslead(string: *const u8, current: *const u8) -> c_int {
    _ismbslead_l(string, current, core::ptr::null_mut())
}

// ── mbbtype.cpp / mbsbtype.cpp ───────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _mbbtype_l(c: u8, ctype: c_int, locale: *mut CrtLocalePointers) -> c_int {
    let mut lu = LocaleUpdate::new(locale);
    let l = lu.locale_t();
    if ctype == _MBC_LEAD {
        return if _ismbbtrail_l(c as u32, l) != 0 { _MBC_TRAIL } else { _MBC_ILLEGAL };
    }
    // _MBC_TRAIL, _MBC_SINGLE, _MBC_ILLEGAL and anything else.
    if _ismbblead_l(c as u32, l) != 0 {
        _MBC_LEAD
    } else if _ismbbprint_l(c as u32, l) != 0 {
        _MBC_SINGLE
    } else {
        _MBC_ILLEGAL
    }
}

#[no_mangle]
pub unsafe extern "C" fn _mbbtype(c: u8, ctype: c_int) -> c_int {
    _mbbtype_l(c, ctype, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsbtype_l(
    string: *const u8,
    mut length: usize,
    locale: *mut CrtLocalePointers,
) -> c_int {
    if string.is_null() {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo();
        return _MBC_ILLEGAL;
    }
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return _MBC_SINGLE;
    }
    let l = lu.locale_t();
    let mut s = string;
    let mut chartype = _MBC_ILLEGAL;
    loop {
        if length == 0 && *s == 0 {
            return _MBC_ILLEGAL;
        }
        if *s == 0 {
            *_errno() = EINVAL;
            _invalid_parameter_noinfo();
            return _MBC_ILLEGAL;
        }
        chartype = _mbbtype_l(*s, chartype, l);
        s = s.add(1);
        // `while (length--)`: test the old value, decrement regardless.
        let more = length != 0;
        length = length.wrapping_sub(1);
        if !more {
            break;
        }
    }
    chartype
}

#[no_mangle]
pub unsafe extern "C" fn _mbsbtype(string: *const u8, length: usize) -> c_int {
    _mbsbtype_l(string, length, core::ptr::null_mut())
}

// ── mbtolwr.cpp / mbtoupr.cpp ────────────────────────────────────────────────

/// The shared body of `_mbctolower_l` / `_mbctoupper_l`. `from_bit` is the SBCS
/// case bit being mapped FROM (`_SBUP` for tolower), as in `_mbbtolower_l`.
unsafe fn mbc_case(c: u32, locale: *mut CrtLocalePointers, map: u32, from_bit: u8) -> u32 {
    let mut lu = LocaleUpdate::new(locale);
    let l = lu.locale_t();
    let mb = mbcinfo(l);
    if c > 0x00FF {
        let val = [((c >> 8) & 0xFF) as u8, (c & 0xFF) as u8];
        let mut ret = [0u8; 4];
        if _ismbblead_l(val[0] as u32, l) == 0 {
            return c;
        }
        if __acrt_LCMapStringA(
            l,
            (*mb).mblocalename,
            map,
            val.as_ptr(),
            2,
            ret.as_mut_ptr(),
            2,
            (*mb).mbcodepage,
            1,
        ) == 0
        {
            return c;
        }
        return ret[1] as u32 + ((ret[0] as u32) << 8);
    }
    // `_mbbtolower_l` / `_mbbtoupper_l`
    if mb_is_bit(mb, c, from_bit) {
        (*mb).mbcasemap[c as usize] as u32
    } else {
        c
    }
}

#[inline]
unsafe fn mb_is_bit(mb: *mut CrtMultibyteData, c: u32, bits: u8) -> bool {
    (*mb).mbctype[c as usize + 1] & bits == bits
}

#[no_mangle]
pub unsafe extern "C" fn _mbctolower_l(c: u32, locale: *mut CrtLocalePointers) -> u32 {
    mbc_case(c, locale, LCMAP_LOWERCASE, _SBUP)
}

#[no_mangle]
pub unsafe extern "C" fn _mbctolower(c: u32) -> u32 {
    _mbctolower_l(c, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbctoupper_l(c: u32, locale: *mut CrtLocalePointers) -> u32 {
    mbc_case(c, locale, LCMAP_UPPERCASE, _SBLOW)
}

#[no_mangle]
pub unsafe extern "C" fn _mbctoupper(c: u32) -> u32 {
    _mbctoupper_l(c, core::ptr::null_mut())
}

// ── mbtohira.cpp / mbtokata.cpp ──────────────────────────────────────────────
// No LocaleUpdate of their own: the C passes the caller's `plocinfo` straight
// into the range test.

#[no_mangle]
pub unsafe extern "C" fn _mbctohira_l(mut c: u32, locale: *mut CrtLocalePointers) -> u32 {
    if _ismbckata_l(c, locale) != 0 && c <= 0x8393 {
        c -= if c < 0x837f { 0xa1 } else { 0xa2 };
    }
    c
}

#[no_mangle]
pub unsafe extern "C" fn _mbctohira(c: u32) -> u32 {
    _mbctohira_l(c, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbctokata_l(mut c: u32, locale: *mut CrtLocalePointers) -> u32 {
    if _ismbchira_l(c, locale) != 0 {
        c += 0xa1;
        if c >= 0x837f {
            c += 1;
        }
    }
    c
}

#[no_mangle]
pub unsafe extern "C" fn _mbctokata(c: u32) -> u32 {
    _mbctokata_l(c, core::ptr::null_mut())
}

// ── tojisjms.cpp — JIS <-> Shift-JIS ─────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _mbcjistojms_l(c: u32, locale: *mut CrtLocalePointers) -> u32 {
    let lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).mbcodepage != _KANJI_CP {
        return c;
    }
    let mut h = (c >> 8) & 0xff;
    let mut l = c & 0xff;
    if !(0x21..=0x7e).contains(&h) || !(0x21..=0x7e).contains(&l) {
        *_errno() = EILSEQ;
        return 0;
    }
    if h & 0x01 != 0 {
        l += if l <= 0x5f { 0x1f } else { 0x20 };
    } else {
        l += 0x7e;
    }
    h = ((h - 0x21) >> 1) + 0x81;
    if h > 0x9f {
        h += 0x40;
    }
    (h << 8) | l
}

#[no_mangle]
pub unsafe extern "C" fn _mbcjistojms(c: u32) -> u32 {
    _mbcjistojms_l(c, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbcjmstojis_l(mut c: u32, locale: *mut CrtLocalePointers) -> u32 {
    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).mbcodepage != _KANJI_CP {
        return c;
    }
    let loc = lu.locale_t();
    let mut h = (c >> 8) & 0xff;
    let l = c & 0xff;
    if _ismbblead_l(h, loc) == 0 || _ismbbtrail_l(l, loc) == 0 {
        *_errno() = EILSEQ;
        return 0;
    }
    // Unsigned arithmetic throughout, as in the C: every step below wraps
    // exactly as `unsigned int` does for the inputs the lead/trail test admits.
    h = h.wrapping_sub(if h >= 0xa0 { 0xc1 } else { 0x81 });
    if l >= 0x9f {
        c = (h << 9).wrapping_add(0x2200);
        c |= l.wrapping_sub(0x7e);
    } else {
        c = (h << 9).wrapping_add(0x2100);
        c |= l.wrapping_sub(if l <= 0x7e { 0x1f } else { 0x20 });
    }
    if c > 0x7E7E || c < 0x2121 || (c & 0xFF) > 0x7E || (c & 0xFF) < 0x21 {
        *_errno() = EILSEQ;
        return 0;
    }
    c
}

#[no_mangle]
pub unsafe extern "C" fn _mbcjmstojis(c: u32) -> u32 {
    _mbcjmstojis_l(c, core::ptr::null_mut())
}
