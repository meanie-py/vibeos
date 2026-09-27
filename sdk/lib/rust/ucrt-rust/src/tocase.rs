//! `tolower`/`toupper` and their wide twins — case mapping for one character.
//! Replaces `sdk/lib/ucrt/convert/{tolower_toupper,towlower,towupper}.cpp` in
//! full (10 exports).
//!
//! A port, not msvcrt's bodies, and the Tier 2 table says why: msvcrt's
//! `_tolower_l`/`_toupper_l` DIFF against UCRT at the very first case, `EOF`.
//! UCRT returns `EOF` for `EOF` before resolving any locale; Wine does not.
//!
//! ## Details that are silent if wrong
//!
//! * **Only `0..=255` takes the table.** The test is `(unsigned)c < 256`, so a
//!   negative `c` other than `EOF` is a codepage lookup, not a table index.
//! * **The lookup sets `errno = EILSEQ` whenever `c` is not a lead/trail pair**
//!   (including every SBCS locale) and then maps the low byte anyway. The C marks
//!   that as undefined behaviour it keeps; it is kept here too.
//! * **`tolower` never resolves a locale until one has been changed**: until then
//!   it is `__ascii_tolower`, which passes every other value — `EOF`, 0xC0, 300 —
//!   through untouched.
//! * **`_tolower`/`_toupper` are unconditional arithmetic**: `_tolower(c)` is
//!   `c - 'A' + 'a'` for ANY `c`, so `_tolower('a')` is 0x81. Callers are meant to
//!   have checked `isupper` first; the function does not.
//! * **`_towlower_l` consults the UTF-16 category BEFORE the narrow map**: a
//!   character below 256 is only mapped if `_wctype` calls it upper (for lower-
//!   casing), since the narrow map is in the locale's codepage and would
//!   otherwise remap bytes that are not letters in UTF-16. In a UTF-8 locale the
//!   table covers only ASCII.

use crate::locale::CrtLocalePointers;
use crate::locale_abi::__acrt_LCMapStringA;
use crate::locale_update::LocaleUpdate;
use core::ffi::{c_int, c_long};

const EOF: c_int = -1;
const WEOF: u16 = 0xFFFF;
const EILSEQ: c_int = 42;
/// `LC_CTYPE` (ucrt locale.h:25) as an index into `locale_name[]`. Named apart
/// from the macro because kernel32's NLS code has an unrelated `LC_CTYPE` = 0.
const LOCALE_NAME_CTYPE: usize = 2;
const CP_UTF8: u32 = 65001;
const LCMAP_LOWERCASE: u32 = 0x0100;
const LCMAP_UPPERCASE: u32 = 0x0200;
const _UPPER: u16 = 0x01;
const _LOWER: u16 = 0x02;
const _LEADBYTE: u16 = 0x8000;

extern "C" {
    static __acrt_locale_changed_data: c_long;
    /// `locale/ctype.cpp` — the wide ctype table, starting at WEOF (so index `c + 1`).
    static _wctype: [u16; 257];
    fn __acrt_LCMapStringW(
        locale_name: *const u16,
        map_flags: u32,
        src: *const u16,
        cch_src: c_int,
        dest: *mut u16,
        cch_dest: c_int,
    ) -> c_int;
    fn _errno() -> *mut c_int;
}

/// `common_tox_win_lookup` (tolower_toupper.cpp).
unsafe fn win_lookup(c: c_int, map_flag: u32, locale: *mut CrtLocalePointers) -> c_int {
    let locinfo = (*locale).locinfo;
    let public = &*(*locinfo).public_ref();
    let mut in_buffer = [0u8; 3];
    let in_count;
    let hi = ((c >> 8) & 0xff) as usize;
    if public.locale_mb_cur_max > 1 && *public.locale_pctype.add(hi) & _LEADBYTE != 0 {
        in_buffer[0] = hi as u8;
        in_buffer[1] = c as u8;
        in_count = 2;
    } else {
        *_errno() = EILSEQ;
        in_buffer[0] = c as u8;
        in_count = 1;
    }
    let mut out_buffer = [0u8; 3];
    let out_count = __acrt_LCMapStringA(
        locale,
        (*locinfo).locale_name[LOCALE_NAME_CTYPE],
        map_flag,
        in_buffer.as_ptr().cast(),
        in_count,
        out_buffer.as_mut_ptr().cast(),
        3,
        public.locale_lc_codepage as c_int,
        1,
    );
    match out_count {
        0 => c,
        1 => out_buffer[0] as c_int,
        _ => out_buffer[1] as c_int | ((out_buffer[0] as c_int) << 8),
    }
}

/// `common_tox_l`. `upper` picks `pcumap` over `pclmap`.
unsafe fn tox_l(c: c_int, map_flag: u32, upper: bool, locale: *mut CrtLocalePointers) -> c_int {
    if c == EOF {
        return EOF;
    }
    let mut lu = LocaleUpdate::new(locale);
    if (c as u32) < 256 {
        let li = lu.locinfo();
        let map = if upper { (*li).pcumap } else { (*li).pclmap };
        return *map.add(c as usize) as c_int;
    }
    win_lookup(c, map_flag, lu.locale_t())
}

#[no_mangle]
pub unsafe extern "C" fn _tolower_l(c: c_int, locale: *mut CrtLocalePointers) -> c_int {
    tox_l(c, LCMAP_LOWERCASE, false, locale)
}

#[no_mangle]
pub unsafe extern "C" fn _toupper_l(c: c_int, locale: *mut CrtLocalePointers) -> c_int {
    tox_l(c, LCMAP_UPPERCASE, true, locale)
}

#[no_mangle]
pub unsafe extern "C" fn tolower(c: c_int) -> c_int {
    if __acrt_locale_changed_data != 0 {
        _tolower_l(c, core::ptr::null_mut())
    } else if (b'A' as c_int..=b'Z' as c_int).contains(&c) {
        c - b'A' as c_int + b'a' as c_int
    } else {
        c
    }
}

#[no_mangle]
pub unsafe extern "C" fn toupper(c: c_int) -> c_int {
    if __acrt_locale_changed_data != 0 {
        _toupper_l(c, core::ptr::null_mut())
    } else if (b'a' as c_int..=b'z' as c_int).contains(&c) {
        c - b'a' as c_int + b'A' as c_int
    } else {
        c
    }
}

#[no_mangle]
pub extern "C" fn _tolower(c: c_int) -> c_int {
    c.wrapping_sub(b'A' as c_int).wrapping_add(b'a' as c_int)
}

#[no_mangle]
pub extern "C" fn _toupper(c: c_int) -> c_int {
    c.wrapping_sub(b'a' as c_int).wrapping_add(b'A' as c_int)
}

/// `_towlower_l` / `_towupper_l`. `from` is the UTF-16 category that must hold
/// before the narrow map is applied: `_UPPER` when lowercasing.
unsafe fn towx_l(c: u16, map_flag: u32, from: u16, locale: *mut CrtLocalePointers) -> u16 {
    if c == WEOF {
        return c;
    }
    let lu = LocaleUpdate::new(locale);
    let locinfo = lu.locinfo();
    let narrow_limit = if (*(*locinfo).public_ref()).locale_lc_codepage == CP_UTF8 { 128 } else { 256 };
    if c < narrow_limit {
        // `_tow{lower,upper}_fast_internal`
        let b = c as u8 as usize;
        if _wctype[b + 1] & from != 0 {
            let map = if from == _UPPER { (*locinfo).pclmap } else { (*locinfo).pcumap };
            return *map.add(b) as u16;
        }
        return c;
    }
    let name = (*locinfo).locale_name[LOCALE_NAME_CTYPE];
    if narrow_limit == 256 && name.is_null() {
        return c;
    }
    let mut widechar: u16 = 0;
    if __acrt_LCMapStringW(name, map_flag, &c, 1, &mut widechar, 1) == 0 {
        return c;
    }
    widechar
}

#[no_mangle]
pub unsafe extern "C" fn _towlower_l(c: u16, locale: *mut CrtLocalePointers) -> u16 {
    towx_l(c, LCMAP_LOWERCASE, _UPPER, locale)
}

#[no_mangle]
pub unsafe extern "C" fn towlower(c: u16) -> u16 {
    _towlower_l(c, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _towupper_l(c: u16, locale: *mut CrtLocalePointers) -> u16 {
    towx_l(c, LCMAP_UPPERCASE, _LOWER, locale)
}

#[no_mangle]
pub unsafe extern "C" fn towupper(c: u16) -> u16 {
    _towupper_l(c, core::ptr::null_mut())
}
