//! The ctype family: `convert/_ctype.cpp`, `convert/_wctype.cpp`, `convert/isctype.cpp`
//! and `convert/iswctype.cpp` -- 63 exports, ported one-for-one.
//!
//! WHY A PORT AND NOT msvcrt's BODIES. msvcrt-crt-rust has these as one macro over
//! `_isctype_l`, but it is Wine's contract, and the UCRT one differs exactly where it
//! matters: UCRT takes the multibyte path only when `mb_cur_max > 1` and then asks
//! `__acrt_GetStringTypeA` with the locale's CODEPAGE, where Wine additionally requires
//! `c > 0` and asks `GetStringTypeExA` with an LCID. Every export here is a one-line
//! wrapper, so the faithful port is smaller than an adapter would be, and the Tier 2
//! harness grades it directly against the objects it replaces.
//!
//! Things that are silent if wrong, each transcribed deliberately:
//!   * `isblank` answers `_BLANK` for a literal tab before any table lookup, and so
//!     does `iswblank` -- but `_isblank_l` does NOT special-case tab (the C doesn't).
//!   * `__iscsym` casts its argument to `unsigned char`; `__iscsymf` does not.
//!   * Every wide `_l` form IGNORES its locale: `_iswalpha_l(c, loc)` is `iswalpha(c)`.
//!   * `wint_t` is 16 bits in UCRT, so `iswctype(0x1_0041, ...)` asks about `0x0041`.
//!   * `pctype` is indexed with a SIGNED `c` in -1..=255 (slot -1 is real, for EOF).
use crate::locale::CrtLocalePointers;
use crate::locale_update::LocaleUpdate;
use core::ffi::c_int;

const _UPPER: c_int = 0x01;
const _LOWER: c_int = 0x02;
const _DIGIT: c_int = 0x04;
const _SPACE: c_int = 0x08;
const _PUNCT: c_int = 0x10;
const _CONTROL: c_int = 0x20;
const _BLANK: c_int = 0x40;
const _HEX: c_int = 0x80;
const _LEADBYTE: c_int = 0x8000;
const _ALPHA: c_int = 0x0100 | _UPPER | _LOWER;
const CT_CTYPE1: u32 = 0x0001;
const WEOF: u16 = 0xFFFF;

extern "C" {
    static __acrt_locale_changed_data: core::ffi::c_long;
    /// Microsoft's wide ctype table (`locale/ctype.cpp`), 256 entries from 0. Read,
    /// not replaced: it is data, and taking it over belongs with that file.
    static _pwctype: *const u16;
    fn __acrt_GetStringTypeA(
        locale: *mut CrtLocalePointers,
        info_type: u32,
        src: *const u8,
        cch: c_int,
        char_type: *mut u16,
        code_page: c_int,
        error: c_int,
    ) -> c_int;
    fn __acrt_GetStringTypeW(info_type: u32, src: *const u16, cch: c_int, char_type: *mut u16)
        -> c_int;
}

/// `__acrt_locale_get_ctype_array_value`: the table value inside -1..=255, else 0.
#[inline]
unsafe fn ctype_array_value(pctype: *const u16, c: c_int, mask: c_int) -> c_int {
    if (-1..=255).contains(&c) {
        (*pctype.offset(c as isize) as c_int) & mask
    } else {
        0
    }
}

#[inline]
unsafe fn pctype_of(lu: &LocaleUpdate) -> *const u16 {
    (*(*lu.locinfo()).public_ref()).locale_pctype
}

#[inline]
unsafe fn mb_cur_max_of(lu: &LocaleUpdate) -> c_int {
    (*(*lu.locinfo()).public_ref()).locale_mb_cur_max
}

/// `fast_check_given_locale` / `fast_check_current_locale` (_ctype.cpp). A NULL
/// locale resolves the thread's -- LocaleUpdate already takes the initial-locale
/// fast path while `__acrt_locale_changed()` is false, as the C does.
unsafe fn fast_check(c: c_int, mask: c_int, locale: *mut CrtLocalePointers) -> c_int {
    let lu = LocaleUpdate::new(locale);
    if (-1..=255).contains(&c) {
        return (*pctype_of(&lu).offset(c as isize) as c_int) & mask;
    }
    if mb_cur_max_of(&lu) > 1 {
        return _isctype_l(c, mask, locale);
    }
    0
}

/// `_isctype_l` (isctype.cpp): the table inside -1..=255, else the lead/trail pair
/// through `__acrt_GetStringTypeA` in the locale's codepage -- with NO mb_cur_max
/// check here; the predicates above make that decision before calling in.
#[no_mangle]
pub unsafe extern "C" fn _isctype_l(c: c_int, mask: c_int, locale: *mut CrtLocalePointers) -> c_int {
    let mut lu = LocaleUpdate::new(locale);
    if (-1..=255).contains(&c) {
        return (*pctype_of(&lu).offset(c as isize) as c_int) & mask;
    }
    let mut buffer = [0u8; 3];
    let len;
    // `_isleadbyte_fast_internal(c >> 8 & 0xff, ...)`
    if ctype_array_value(pctype_of(&lu), (c >> 8) & 0xff, _LEADBYTE) != 0 {
        buffer[0] = ((c >> 8) & 0xff) as u8;
        buffer[1] = c as u8;
        len = 2;
    } else {
        buffer[0] = c as u8;
        len = 1;
    }
    let mut character_type = [0u16; 3];
    let code_page = (*(*lu.locinfo()).public_ref()).locale_lc_codepage as c_int;
    if __acrt_GetStringTypeA(
        lu.locale_t(),
        CT_CTYPE1,
        buffer.as_ptr(),
        len,
        character_type.as_mut_ptr(),
        code_page,
        1,
    ) == 0
    {
        return 0;
    }
    (character_type[0] as c_int) & mask
}

/// `_isctype` (isctype.cpp): the initial locale's table unless the locale changed.
#[no_mangle]
pub unsafe extern "C" fn _isctype(c: c_int, mask: c_int) -> c_int {
    if __acrt_locale_changed_data != 0 {
        return _isctype_l(c, mask, core::ptr::null_mut());
    }
    let lu = LocaleUpdate::new(core::ptr::null_mut());
    ctype_array_value(pctype_of(&lu), c, mask)
}

macro_rules! isc {
    ($plain:ident, $l:ident, $mask:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $l(c: c_int, locale: *mut CrtLocalePointers) -> c_int {
            fast_check(c, $mask, locale)
        }
        #[no_mangle]
        pub unsafe extern "C" fn $plain(c: c_int) -> c_int {
            fast_check(c, $mask, core::ptr::null_mut())
        }
    };
}

isc!(isalpha, _isalpha_l, _ALPHA);
isc!(isupper, _isupper_l, _UPPER);
isc!(islower, _islower_l, _LOWER);
isc!(isdigit, _isdigit_l, _DIGIT);
isc!(isxdigit, _isxdigit_l, _HEX);
isc!(isspace, _isspace_l, _SPACE);
isc!(ispunct, _ispunct_l, _PUNCT);
isc!(isalnum, _isalnum_l, _ALPHA | _DIGIT);
isc!(isprint, _isprint_l, _BLANK | _PUNCT | _ALPHA | _DIGIT);
isc!(isgraph, _isgraph_l, _PUNCT | _ALPHA | _DIGIT);
isc!(iscntrl, _iscntrl_l, _CONTROL);

/// `_isblank_l` does NOT special-case tab; only the plain form does.
#[no_mangle]
pub unsafe extern "C" fn _isblank_l(c: c_int, locale: *mut CrtLocalePointers) -> c_int {
    fast_check(c, _BLANK, locale)
}

#[no_mangle]
pub unsafe extern "C" fn isblank(c: c_int) -> c_int {
    if c == b'\t' as c_int {
        return _BLANK;
    }
    fast_check(c, _BLANK, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn __isascii(c: c_int) -> c_int {
    ((c as u32) < 0x80) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn __toascii(c: c_int) -> c_int {
    c & 0x7f
}

/// `__iscsymf(c)` is the header macro `isalpha(c) || c == '_'` -- no cast.
#[no_mangle]
pub unsafe extern "C" fn __iscsymf(c: c_int) -> c_int {
    (isalpha(c) != 0 || c == b'_' as c_int) as c_int
}

/// `__iscsym` casts to `unsigned char` first; `__iscsymf` above does not.
#[no_mangle]
pub unsafe extern "C" fn __iscsym(c: c_int) -> c_int {
    let c = c as u8 as c_int;
    (isalnum(c) != 0 || c == b'_' as c_int) as c_int
}

// ── wide (iswctype.cpp, _wctype.cpp) ─────────────────────────────────────────

/// `iswctype`: WEOF is nothing, 0..255 from the table, the rest from the OS.
#[no_mangle]
pub unsafe extern "C" fn iswctype(c: u16, mask: u16) -> c_int {
    if c == WEOF {
        return 0;
    }
    if c < 256 {
        return (*_pwctype.add(c as usize) & mask) as c_int;
    }
    let wide_character = c;
    let mut char_type: u16 = 0;
    if __acrt_GetStringTypeW(CT_CTYPE1, &wide_character, 1, &mut char_type) == 0 {
        return 0;
    }
    (char_type & mask) as c_int
}

/// The locale is IGNORED: `_iswctype_l(c, mask, loc)` is `iswctype(c, mask)`.
#[no_mangle]
pub unsafe extern "C" fn _iswctype_l(c: u16, mask: u16, _locale: *mut CrtLocalePointers) -> c_int {
    iswctype(c, mask)
}

macro_rules! isw {
    ($plain:ident, $l:ident, $mask:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $plain(c: u16) -> c_int {
            iswctype(c, $mask as u16)
        }
        #[no_mangle]
        pub unsafe extern "C" fn $l(c: u16, _locale: *mut CrtLocalePointers) -> c_int {
            iswctype(c, $mask as u16)
        }
    };
}

isw!(iswalpha, _iswalpha_l, _ALPHA);
isw!(iswupper, _iswupper_l, _UPPER);
isw!(iswlower, _iswlower_l, _LOWER);
isw!(iswdigit, _iswdigit_l, _DIGIT);
isw!(iswxdigit, _iswxdigit_l, _HEX);
isw!(iswspace, _iswspace_l, _SPACE);
isw!(iswpunct, _iswpunct_l, _PUNCT);
isw!(iswalnum, _iswalnum_l, _ALPHA | _DIGIT);
isw!(iswprint, _iswprint_l, _BLANK | _PUNCT | _ALPHA | _DIGIT);
isw!(iswgraph, _iswgraph_l, _PUNCT | _ALPHA | _DIGIT);
isw!(iswcntrl, _iswcntrl_l, _CONTROL);

/// `iswblank` special-cases tab in BOTH forms (the header macro does; the `_l`
/// wrapper calls it).
#[no_mangle]
pub unsafe extern "C" fn iswblank(c: u16) -> c_int {
    if c == b'\t' as u16 {
        return _BLANK;
    }
    iswctype(c, _BLANK as u16)
}

#[no_mangle]
pub unsafe extern "C" fn _iswblank_l(c: u16, _locale: *mut CrtLocalePointers) -> c_int {
    iswblank(c)
}

#[no_mangle]
pub unsafe extern "C" fn iswascii(c: u16) -> c_int {
    ((c as u32) < 0x80) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn __iswcsymf(c: u16) -> c_int {
    (iswalpha(c) != 0 || c == b'_' as u16) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn __iswcsym(c: u16) -> c_int {
    (iswalnum(c) != 0 || c == b'_' as u16) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn _iswcsymf_l(c: u16, _locale: *mut CrtLocalePointers) -> c_int {
    __iswcsymf(c)
}

#[no_mangle]
pub unsafe extern "C" fn _iswcsym_l(c: u16, _locale: *mut CrtLocalePointers) -> c_int {
    __iswcsym(c)
}

/// `_isleadbyte_l` (_wctype.cpp): table-only, with the locale resolved.
#[no_mangle]
pub unsafe extern "C" fn _isleadbyte_l(c: c_int, locale: *mut CrtLocalePointers) -> c_int {
    let lu = LocaleUpdate::new(locale);
    ctype_array_value(pctype_of(&lu), c, _LEADBYTE)
}

#[no_mangle]
pub unsafe extern "C" fn isleadbyte(c: c_int) -> c_int {
    _isleadbyte_l(c, core::ptr::null_mut())
}
