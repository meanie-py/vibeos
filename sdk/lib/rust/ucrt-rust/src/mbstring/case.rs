//! In-place case conversion (`_mbslwr`, `_mbsupr`) and the 1-byte<->2-byte
//! Shift-JIS code converters (`_mbbtombc`, `_mbctombb`) — plain, `_l`, `_s`.
//!
//! ## Details that are silent if wrong
//!
//! * **`_mbslwr`/`_mbsupr` have no `ismbcodepage==0` fast path of their own**
//!   — every other family that reaches a locale checks it and falls to a
//!   plain `str*` primitive; this one always walks the string byte-by-byte
//!   checking `_ismbblead_l`, which just never matches under codepage 0.
//!   `_mbslwr_s_l`/`_mbsupr_s_l` (the real bodies; the plain forms are
//!   `_s_l` calls with `sizeInBytes = (size_t)-1` for a non-null string) DO
//!   validate and truncate-detect via `strnlen`, unconditionally.
//! * **A DBCS character maps through `__acrt_LCMapStringA`, not a table** —
//!   `LCMAP_LOWERCASE`/`LCMAP_UPPERCASE` over the 2 raw bytes; a single-byte
//!   character maps through the `_mbbtolower_l`/`_mbbtoupper_l` macros
//!   (`crate::ismbc`), which is a table lookup. Two different mechanisms in
//!   the same loop, not a fallback of one to the other.
//! * **A `__acrt_LCMapStringA` failure sets `errno = EILSEQ`, resets the
//!   string to empty (`_RESET_STRING`, a no-op in this build — see the module
//!   doc), and returns `EILSEQ`** — NOT `-1`/`EINVAL`; this is the one path
//!   in this family that produces `EILSEQ` rather than `EINVAL`.
//! * **`_mbslwr_l`/`_mbsupr_l`/plain forms return `nullptr` on ANY `_s_l`
//!   failure** (bad args OR `EILSEQ`), discarding the specific error code —
//!   the caller of the plain form has no way to distinguish "bad argument"
//!   from "illegal sequence" except by re-checking `errno` themselves.
//! * **`_mbbtombc`/`_mbctombb` are Shift-JIS-only** (`mbcodepage != _KANJI_CP`
//!   returns the input unchanged) and are TABLE-DRIVEN, not algorithmic:
//!   `mbbtable[]` (a dense 0x20..0x7E ASCII->Kanji map plus one hard-coded
//!   exception for `0xDC`) and `mbctable[]` (a Katakana/Hiragana single-byte
//!   <-> Kanji correspondence table with a `synonym` field used only by
//!   `_mbctombb`'s reverse search to prefer the LOWEST matching ASCII byte
//!   when several Kanji codes map to it). Both tables are transcribed
//!   verbatim, not regenerated, so a single wrong entry is silent until
//!   graded against the reference codepoint-by-codepoint.
//! * **`_mbctombb`'s reverse search is NOT an exact-match-only lookup**: past
//!   the exact `result == 0` hit, it also matches when `c` falls within
//!   `synonym` of a table entry's `mbccode` *on the same high byte* — i.e. a
//!   Kanji code adjacent to (but not equal to) a listed one still maps back
//!   to that entry's ASCII byte, provided the same 0xFF00-masked high byte
//!   and a smaller code. Get the high-byte mask or the strict `<` on
//!   `synonym` wrong and this silently drops or over-matches neighbours.

use super::Locale;
use crate::locale_update::LocaleUpdate;
use core::ffi::c_int;

const _KANJI_CP: c_int = 932;
const LCMAP_LOWERCASE: u32 = 0x0100;
const LCMAP_UPPERCASE: u32 = 0x0200;

// ── mbslwr.cpp / mbsupr.cpp ────────────────────────────────────────────────

/// Shared body of `_mbslwr_s_l`/`_mbsupr_s_l`. `map` is the `LCMAP_*` flag,
/// `bb_case` maps a single byte through `_mbbtolower_l`/`_mbbtoupper_l`.
#[inline]
unsafe fn case_s_l(
    string: *mut u8,
    size_in_bytes: usize,
    locale: Locale,
    map: u32,
    bb_case: unsafe fn(u32, Locale) -> u32,
) -> c_int {
    extern "C" {
        fn strnlen(s: *const core::ffi::c_char, n: usize) -> usize;
        fn _errno() -> *mut c_int;
        fn _invalid_parameter_noinfo();
    }
    if !((!string.is_null() && size_in_bytes > 0) || (string.is_null() && size_in_bytes == 0)) {
        *_errno() = super::EINVAL;
        _invalid_parameter_noinfo(); // `_VALIDATE_RETURN`: the handler, after errno.
        return super::EINVAL;
    }
    if string.is_null() {
        return 0;
    }
    let stringlen = strnlen(string.cast(), size_in_bytes);
    if stringlen >= size_in_bytes {
        *string = 0; // `_RESET_STRING`: a no-op fill in this build, see module doc.
        *_errno() = super::EINVAL;
        _invalid_parameter_noinfo(); // `_VALIDATE_RETURN`: the handler, after errno.
        return super::EINVAL;
    }
    let mut lu = LocaleUpdate::new(locale);
    let l = lu.locale_t();
    let mb = lu.mbcinfo();
    let (mut cp, mut dst) = (string, string);
    while *cp != 0 {
        if crate::ismbbyte::_ismbblead_l(*cp as u32, l) != 0 {
            let mut ret = [0u8; 4];
            let src = [*cp, *cp.add(1)];
            let retval = crate::locale_abi::__acrt_LCMapStringA(
                l,
                (*mb).mblocalename,
                map,
                src.as_ptr().cast(),
                2,
                ret.as_mut_ptr().cast(),
                2,
                (*mb).mbcodepage,
                1,
            );
            if retval == 0 {
                *_errno() = super::EILSEQ;
                *string = 0;
                return super::EILSEQ;
            }
            *dst = ret[0];
            dst = dst.add(1);
            cp = cp.add(2);
            if retval > 1 {
                *dst = ret[1];
                dst = dst.add(1);
            }
        } else {
            *dst = bb_case(*cp as u32, l) as u8;
            dst = dst.add(1);
            cp = cp.add(1);
        }
    }
    *dst = 0;
    0
}

#[no_mangle]
pub unsafe extern "C" fn _mbslwr_s_l(string: *mut u8, size_in_bytes: usize, locale: Locale) -> c_int {
    case_s_l(string, size_in_bytes, locale, LCMAP_LOWERCASE, super::mbbtolower)
}

#[no_mangle]
pub unsafe extern "C" fn _mbslwr_s(string: *mut u8, size_in_bytes: usize) -> c_int {
    _mbslwr_s_l(string, size_in_bytes, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbslwr_l(string: *mut u8, locale: Locale) -> *mut u8 {
    let n = if string.is_null() { 0 } else { usize::MAX };
    if _mbslwr_s_l(string, n, locale) == 0 { string } else { core::ptr::null_mut() }
}

#[no_mangle]
pub unsafe extern "C" fn _mbslwr(string: *mut u8) -> *mut u8 {
    _mbslwr_l(string, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsupr_s_l(string: *mut u8, size_in_bytes: usize, locale: Locale) -> c_int {
    case_s_l(string, size_in_bytes, locale, LCMAP_UPPERCASE, super::mbbtoupper)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsupr_s(string: *mut u8, size_in_bytes: usize) -> c_int {
    _mbsupr_s_l(string, size_in_bytes, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsupr_l(string: *mut u8, locale: Locale) -> *mut u8 {
    let n = if string.is_null() { 0 } else { usize::MAX };
    if _mbsupr_s_l(string, n, locale) == 0 { string } else { core::ptr::null_mut() }
}

#[no_mangle]
pub unsafe extern "C" fn _mbsupr(string: *mut u8) -> *mut u8 {
    _mbsupr_l(string, core::ptr::null_mut())
}

// ── tombbmbc.cpp ────────────────────────────────────────────────────────────

const ASCLOW: u32 = 0x20;
const ASCHIGH: u32 = 0x7e;
const TOMBBMBC_SBLOW: u32 = 0xA1;
const TOMBBMBC_SBHIGH: u32 = 0xDF;
const MBLIMIT: u32 = 0x8396;

/// `mbbtable[]` (`tombbmbc.cpp:24-33`): ASCII 0x20..0x7E -> Kanji code.
#[rustfmt::skip]
static MBBTABLE: [u16; (ASCHIGH - ASCLOW + 1) as usize] = [
    0x8140, 0x8149, 0x8168, 0x8194, 0x8190, 0x8193, 0x8195, 0x8166,
    0x8169, 0x816a, 0x8196, 0x817b, 0x8143, 0x817c, 0x8144, 0x815e,
    0x824f, 0x8250, 0x8251, 0x8252, 0x8253, 0x8254, 0x8255, 0x8256,
    0x8257, 0x8258, 0x8146, 0x8147, 0x8183, 0x8181, 0x8184, 0x8148,
    0x8197, 0x8260, 0x8261, 0x8262, 0x8263, 0x8264, 0x8265, 0x8266,
    0x8267, 0x8268, 0x8269, 0x826a, 0x826b, 0x826c, 0x826d, 0x826e,
    0x826f, 0x8270, 0x8271, 0x8272, 0x8273, 0x8274, 0x8275, 0x8276,
    0x8277, 0x8278, 0x8279, 0x816d, 0x818f, 0x816e, 0x814f, 0x8151,
    0x8165, 0x8281, 0x8282, 0x8283, 0x8284, 0x8285, 0x8286, 0x8287,
    0x8288, 0x8289, 0x828a, 0x828b, 0x828c, 0x828d, 0x828e, 0x828f,
    0x8290, 0x8291, 0x8292, 0x8293, 0x8294, 0x8295, 0x8296, 0x8297,
    0x8298, 0x8299, 0x829a, 0x816f, 0x8162, 0x8170, 0x8150,
];

/// `mbctable[]` (`tombbmbc.cpp:36-...`): `{asc, synonym, mbccode}`, terminated
/// by an all-zero entry.
#[rustfmt::skip]
static MBCTABLE: &[(u8, u8, u16)] = &[
    (0xA7,1,0x8340),(0xB1,1,0x8341),(0xA8,1,0x8342),(0xB2,1,0x8343),
    (0xA9,1,0x8344),(0xB3,1,0x8345),(0xAA,1,0x8346),(0xB4,1,0x8347),
    (0xAB,1,0x8348),(0xB5,1,0x8349),
    (0xB6,2,0x834A),(0xB7,2,0x834C),(0xB8,2,0x834E),(0xB9,2,0x8350),(0xBA,2,0x8352),
    (0xBB,2,0x8354),(0xBC,2,0x8356),(0xBD,2,0x8358),(0xBE,2,0x835A),(0xBF,2,0x835C),
    (0xC0,2,0x835E),(0xC1,2,0x8360),(0xAF,1,0x8362),(0xC2,2,0x8363),(0xC3,2,0x8365),(0xC4,2,0x8367),
    (0xC5,1,0x8369),(0xC6,1,0x836A),(0xC7,1,0x836B),(0xC8,1,0x836C),(0xC9,1,0x836D),
    (0xCA,3,0x836E),(0xCB,3,0x8371),(0xCC,3,0x8374),(0xCD,3,0x8377),(0xCE,3,0x837A),
    (0xCF,1,0x837D),(0xD0,1,0x837E),(0xD1,1,0x8380),(0xD2,1,0x8381),(0xD3,1,0x8382),
    (0xAC,1,0x8383),(0xD4,1,0x8384),(0xAD,1,0x8385),(0xD5,1,0x8386),(0xAE,1,0x8387),(0xD6,1,0x8388),
    (0xD7,1,0x8389),(0xD8,1,0x838A),(0xD9,1,0x838B),(0xDA,1,0x838C),(0xDB,1,0x838D),
    (0xDC,2,0x838E),(0xB2,1,0x8390),(0xB4,1,0x8391),
    (0xA6,1,0x8392),(0xDD,1,0x8393),
    (0xB3,1,0x8394),(0xB6,1,0x8395),(0xB9,1,0x8396),
    // Hiragana table.
    (0xA7,1,0x829F),(0xB1,1,0x82A0),(0xA8,1,0x82A1),(0xB2,1,0x82A2),
    (0xA9,1,0x82A3),(0xB3,1,0x82A4),(0xAA,1,0x82A5),(0xB4,1,0x82A6),
    (0xAB,1,0x82A7),(0xB5,1,0x82A8),
    (0xB6,2,0x82A9),(0xB7,2,0x82AB),(0xB8,2,0x82AD),(0xB9,2,0x82AF),(0xBA,2,0x82B1),
    (0xBB,2,0x82B3),(0xBC,2,0x82B5),(0xBD,2,0x82B7),(0xBE,2,0x82B9),(0xBF,2,0x82BB),
    (0xC0,2,0x82BD),(0xC1,2,0x82BF),(0xAF,1,0x82C1),(0xC2,2,0x82C2),(0xC3,2,0x82C4),(0xC4,2,0x82C6),
    (0xC5,1,0x82C8),(0xC6,1,0x82C9),(0xC7,1,0x82CA),(0xC8,1,0x82CB),(0xC9,1,0x82CC),
    (0xCA,3,0x82CD),(0xCB,3,0x82D0),(0xCC,3,0x82D3),(0xCD,3,0x82D6),(0xCE,3,0x82D9),
    (0xCF,1,0x82DC),(0xD0,1,0x82DD),(0xD1,1,0x82DE),(0xD2,1,0x82DF),(0xD3,1,0x82E0),
    (0xAC,1,0x82E1),(0xD4,1,0x82E2),(0xAD,1,0x82E3),(0xD5,1,0x82E4),(0xAE,1,0x82E5),(0xD6,1,0x82E6),
    (0xD7,1,0x82E7),(0xD8,1,0x82E8),(0xD9,1,0x82E9),(0xDA,1,0x82EA),(0xDB,1,0x82EB),
    (0xDC,2,0x82EC),(0xB2,1,0x82EE),(0xB4,1,0x82EF),
    (0xA6,1,0x82F0),(0xDD,1,0x82F1),
    (0x20,1,0x8140),
    (0xA1,1,0x8142),(0xA2,1,0x8175),(0xA3,1,0x8176),(0xA4,1,0x8141),(0xA5,1,0x8145),
    (0xB0,1,0x815b),(0xDE,1,0x814a),(0xDF,1,0x814b),
    (0,0,0),
];

#[no_mangle]
pub unsafe extern "C" fn _mbbtombc_l(c: u32, locale: Locale) -> u32 {
    let lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).mbcodepage != _KANJI_CP {
        return c;
    }
    if (ASCLOW..=ASCHIGH).contains(&c) {
        return MBBTABLE[(c - ASCLOW) as usize] as u32;
    }
    if c == 0xdc {
        return 0x838f;
    }
    if (TOMBBMBC_SBLOW..=TOMBBMBC_SBHIGH).contains(&c) {
        for &(asc, _syn, code) in MBCTABLE {
            if asc == 0 && code == 0 {
                break;
            }
            if c == asc as u32 {
                return code as u32;
            }
        }
    }
    c
}

#[no_mangle]
pub unsafe extern "C" fn _mbbtombc(c: u32) -> u32 {
    _mbbtombc_l(c, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbctombb_l(c: u32, locale: Locale) -> u32 {
    let lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).mbcodepage != _KANJI_CP {
        return c;
    }
    for (i, &v) in MBBTABLE.iter().enumerate() {
        if c == v as u32 {
            return i as u32 + ASCLOW;
        }
    }
    if c <= MBLIMIT {
        for &(asc, syn, code) in MBCTABLE {
            if asc == 0 && syn == 0 && code == 0 {
                break;
            }
            let result = c as i32 - code as i32;
            if result == 0 {
                return asc as u32;
            } else if (c & 0xff00) == (code as u32 & 0xff00) && result > 0 && (result - syn as i32) < 0 {
                return asc as u32;
            }
        }
    }
    c
}

#[no_mangle]
pub unsafe extern "C" fn _mbctombb(c: u32) -> u32 {
    _mbctombb_l(c, core::ptr::null_mut())
}
