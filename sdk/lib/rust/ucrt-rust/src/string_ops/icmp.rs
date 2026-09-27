//! Case-insensitive compare: `_memicmp`/`_stricmp`/`_strnicmp`/`_wcsicmp`/
//! `_wcsnicmp`, each with its `_l` locale-explicit twin and (narrow forms
//! only export it under this name on i386; here it is arch-uniform) its
//! `__ascii_*` ASCII-only, locale-blind fast path.
//!
//! Replaces in full: `memicmp.cpp`, `stricmp.cpp`, `strnicmp.cpp`,
//! `wcsicmp.cpp`, `wcsnicmp.cpp` -- INCLUDING the `__ascii_memicmp`/
//! `__ascii_strnicmp` bodies that on i386 come from hand asm
//! (`i386/_memicmp.s`, `i386/_strnicm.s`) instead of these `.cpp` files'
//! `#if !defined(_M_IX86)` fallback (see `plain.rs`'s module doc for why one
//! Rust body correctly serves all three targets here).
//!
//! ## Details that are silent if wrong
//!
//! * **The "C locale" fast path is `__acrt_locale_changed_data == 0`**, not
//!   "locale is literally `"C"`" -- it means "no thread has EVER called
//!   `setlocale`", after which every one of these functions permanently takes
//!   the slower `_l`-shaped path (even a later `setlocale(LC_ALL, "C")` does
//!   not undo it, since the C never clears that flag).
//! * **`_memicmp`/`_strnicmp` validate `count <= INT_MAX` for the LOCALE
//!   path, but the ascii fast path has no such check** -- `_memicmp`'s
//!   `count`-bound check is absent entirely (only pointer-null validation),
//!   while `_strnicmp`'s locale path checks `count <= INT_MAX` and the ascii
//!   path does not.
//! * **`_wcsicmp_l`/`_wcsnicmp_l` take the ascii fast path when
//!   `locale_name[LC_CTYPE] == NULL`** (the "C" locale by NAME, not by the
//!   `locale_changed` flag) -- narrow `_stricmp_l`/`_strnicmp_l` have NO such
//!   short-circuit; they always run the full `_tolower_fast_internal` path
//!   once a `_locale_t` reaches the `_l` form. This asymmetry (checked only
//!   for wide) is in the vendored C exactly as read.
//! * **`_towlower_internal`/`_towlower_fast_internal` used here are NOT the
//!   same function as `tocase.rs`'s exported `_towlower_l`** -- the internal
//!   helper has no UTF-8 narrow-limit special case (`towlower_fast` below is
//!   a fresh, narrower port; reusing `tocase::_towlower_l` here would be
//!   silently wrong on a UTF-8 codepage).
//! * **Every one of these returns `_NLSCMPERROR` (`INT_MAX`) on a validation
//!   failure**, not `-1`/`EINVAL` alone -- distinguishing "strings differ" from
//!   "bad call" only by this sentinel (itself a valid, if wildly unlikely,
//!   ordinary difference magnitude for the narrow, locale-blind forms).
//! * **The loop shapes are NOT literal libcall idioms** (`stricmp` is not a
//!   recognized LLVM builtin the way `strcmp`/`memcmp` are), so none of these
//!   need the `black_box` self-recursion defense `plain.rs` requires.
use crate::locale::{CrtLocaleData, CrtLocalePointers, CRT_LC_CTYPE};
use crate::locale_update::LocaleUpdate;
use core::ffi::{c_int, c_void};

const EINVAL: c_int = 22;
const NLSCMPERROR: c_int = i32::MAX;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    /// Microsoft's wide ctype table (`locale/ctype.cpp`), indexed at `c + 1`
    /// (slot 0 is WEOF). Same static `tocase.rs`/`ctype.rs` already import.
    static _wctype: [u16; 257];
}

#[inline]
unsafe fn validate(e: bool) -> bool {
    if !e {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo();
    }
    e
}

/// `__ascii_tolower` (ctype.h macro form, `_CRT_DEFINE_ASCII_CTYPE_MACROS`
/// off): ASCII-only, no locale, no table.
#[inline]
fn ascii_tolower(c: u8) -> u8 {
    if c.is_ascii_uppercase() {
        c - b'A' + b'a'
    } else {
        c
    }
}

/// `__ascii_towlower` (ctype.h): `__ascii_tolower` widened, per the header's
/// `return __ascii_tolower(_C);` (a plain int, not a real UTF-16 fold).
#[inline]
fn ascii_towlower(c: u16) -> u16 {
    if (b'A' as u16..=b'Z' as u16).contains(&c) {
        c - b'A' as u16 + b'a' as u16
    } else {
        c
    }
}

/// `_UPPER` bit in `_wctype`/`pctype` (locale.rs's `CrtLocaleData` doesn't
/// re-export the ctype bitmask constants, so re-declared here -- same value
/// as `ctype.rs`/`tocase.rs`).
const CTYPE_UPPER: u16 = 0x01;

/// `_towlower_fast_internal` (corecrt_internal.h): `c` must be `< 256`. NOT
/// `tocase.rs`'s `_towlower_l` -- see module doc.
#[inline]
unsafe fn towlower_fast(c: u8, locinfo: *mut CrtLocaleData) -> u16 {
    if _wctype[c as usize + 1] & CTYPE_UPPER != 0 {
        (*locinfo).pclmap.add(c as usize).read() as u16
    } else {
        c as u16
    }
}

/// `LCMAP_LOWERCASE` / `CP_UTF8` / `WEOF` -- same values as `tocase.rs`.
const LCMAP_LOWERCASE: u32 = 0x0100;
const CP_UTF8: u32 = 65001;
const WEOF: u16 = 0xFFFF;

/// The `c >= 256` tail of `_towlower_l` (tocase.rs's `towx_l`), reimplemented
/// here rather than called cross-module: `tocase` is a SEPARATE Cargo feature
/// (`#[cfg(feature = "tocase")]` in lib.rs), so a direct call would make
/// `string-ops` fail to build standalone. Semantics match `towx_l` exactly
/// for `c >= 256` (the only range this helper is ever invoked with): the
/// `narrow_limit`/`c < narrow_limit` branch never applies here, but the
/// `narrow_limit == 256 && name.is_null()` short-circuit still does.
unsafe fn towlower_l_ge256(c: u16, locale: *mut CrtLocalePointers) -> u16 {
    if c == WEOF {
        return c;
    }
    let locinfo = (*locale).locinfo;
    let narrow_limit_is_256 = (*(*locinfo).public_ref()).locale_lc_codepage != CP_UTF8;
    let name = (*locinfo).locale_name[CRT_LC_CTYPE as usize];
    if narrow_limit_is_256 && name.is_null() {
        return c;
    }
    let mut widechar: u16 = 0;
    if crate::locale_abi::__acrt_LCMapStringW(name, LCMAP_LOWERCASE, &c, 1, &mut widechar, 1) == 0 {
        return c;
    }
    widechar
}

/// `_towlower_internal` (corecrt_internal.h). `locale` is already a resolved
/// `_locale_t` (the caller's `LocaleUpdate::locale_t()`), so this reads
/// `locale->locinfo` directly rather than re-resolving.
unsafe fn towlower_internal(c: u16, locale: *mut CrtLocalePointers) -> u16 {
    if c < 256 {
        towlower_fast(c as u8, (*locale).locinfo)
    } else {
        towlower_l_ge256(c, locale)
    }
}

// ---------------------------------------------------------------------------
// memicmp
// ---------------------------------------------------------------------------

/// `_memicmp_l` (memicmp.cpp).
#[no_mangle]
pub unsafe extern "C" fn _memicmp_l(
    lhs: *const c_void,
    rhs: *const c_void,
    count: usize,
    plocinfo: *mut CrtLocalePointers,
) -> c_int {
    if !validate(!lhs.is_null() || count == 0) {
        return NLSCMPERROR;
    }
    if !validate(!rhs.is_null() || count == 0) {
        return NLSCMPERROR;
    }
    if count == 0 {
        return 0;
    }
    let mut l = lhs as *const u8;
    let mut r = rhs as *const u8;
    let mut lu = LocaleUpdate::new(plocinfo);
    let loc = lu.locale_t();
    let locinfo = (*loc).locinfo;
    let mut remaining = count;
    let mut result: c_int;
    loop {
        result = (towlower_fast_narrow(*l, locinfo) as c_int) - (towlower_fast_narrow(*r, locinfo) as c_int);
        l = l.add(1);
        r = r.add(1);
        remaining -= 1;
        if result != 0 || remaining == 0 {
            break;
        }
    }
    result
}

/// `_tolower_fast_internal` narrowed to a plain byte map read -- used only by
/// the *icmp family (no `EOF`/locale-changed dance, `c` always a real byte).
#[inline]
unsafe fn towlower_fast_narrow(c: u8, locinfo: *mut CrtLocaleData) -> u8 {
    (*locinfo).pclmap.add(c as usize).read()
}

/// `__ascii_memicmp` (i386 `_memicmp.s` / amd64+arm64 `memicmp.cpp` fallback).
#[no_mangle]
pub unsafe extern "C" fn __ascii_memicmp(lhs: *const c_void, rhs: *const c_void, count: usize) -> c_int {
    if count == 0 {
        return 0;
    }
    let mut l = lhs as *const u8;
    let mut r = rhs as *const u8;
    let mut remaining = count;
    let mut result: c_int;
    loop {
        result = (ascii_tolower(*l) as c_int) - (ascii_tolower(*r) as c_int);
        l = l.add(1);
        r = r.add(1);
        remaining -= 1;
        if result != 0 || remaining == 0 {
            break;
        }
    }
    result
}

/// `_memicmp` (memicmp.cpp): ascii fast path unless a thread has ever set a
/// non-initial locale.
#[no_mangle]
pub unsafe extern "C" fn _memicmp(lhs: *const c_void, rhs: *const c_void, count: usize) -> c_int {
    if crate::locale_update::__acrt_locale_changed_data == 0 {
        if !validate(!lhs.is_null() || count == 0) {
            return NLSCMPERROR;
        }
        if !validate(!rhs.is_null() || count == 0) {
            return NLSCMPERROR;
        }
        return __ascii_memicmp(lhs, rhs, count);
    }
    _memicmp_l(lhs, rhs, count, core::ptr::null_mut())
}

// ---------------------------------------------------------------------------
// stricmp
// ---------------------------------------------------------------------------

/// `_stricmp_l` (stricmp.cpp).
#[no_mangle]
pub unsafe extern "C" fn _stricmp_l(
    lhs: *const u8,
    rhs: *const u8,
    plocinfo: *mut CrtLocalePointers,
) -> c_int {
    if !validate(!lhs.is_null()) {
        return NLSCMPERROR;
    }
    if !validate(!rhs.is_null()) {
        return NLSCMPERROR;
    }
    let mut l = lhs;
    let mut r = rhs;
    let mut lu = LocaleUpdate::new(plocinfo);
    let loc = lu.locale_t();
    let locinfo = (*loc).locinfo;
    let mut lv;
    let mut rv;
    loop {
        lv = towlower_fast_narrow(*l, locinfo);
        rv = towlower_fast_narrow(*r, locinfo);
        l = l.add(1);
        r = r.add(1);
        if lv != rv || lv == 0 {
            break;
        }
    }
    (lv as c_int) - (rv as c_int)
}

/// `__ascii_stricmp` (stricmp.cpp).
#[no_mangle]
pub unsafe extern "C" fn __ascii_stricmp(lhs: *const u8, rhs: *const u8) -> c_int {
    let mut l = lhs;
    let mut r = rhs;
    let mut lv;
    let mut rv;
    loop {
        lv = ascii_tolower(*l);
        rv = ascii_tolower(*r);
        l = l.add(1);
        r = r.add(1);
        if lv != rv || lv == 0 {
            break;
        }
    }
    (lv as c_int) - (rv as c_int)
}

/// `_stricmp` (stricmp.cpp).
#[no_mangle]
pub unsafe extern "C" fn _stricmp(lhs: *const u8, rhs: *const u8) -> c_int {
    if crate::locale_update::__acrt_locale_changed_data == 0 {
        if !validate(!lhs.is_null()) {
            return NLSCMPERROR;
        }
        if !validate(!rhs.is_null()) {
            return NLSCMPERROR;
        }
        return __ascii_stricmp(lhs, rhs);
    }
    _stricmp_l(lhs, rhs, core::ptr::null_mut())
}

// ---------------------------------------------------------------------------
// strnicmp
// ---------------------------------------------------------------------------

const INT_MAX: usize = i32::MAX as usize;

/// `_strnicmp_l` (strnicmp.cpp).
#[no_mangle]
pub unsafe extern "C" fn _strnicmp_l(
    lhs: *const u8,
    rhs: *const u8,
    count: usize,
    plocinfo: *mut CrtLocalePointers,
) -> c_int {
    if !validate(!lhs.is_null()) {
        return NLSCMPERROR;
    }
    if !validate(!rhs.is_null()) {
        return NLSCMPERROR;
    }
    if !validate(count <= INT_MAX) {
        return NLSCMPERROR;
    }
    if count == 0 {
        return 0;
    }
    let mut l = lhs;
    let mut r = rhs;
    let mut lu = LocaleUpdate::new(plocinfo);
    let loc = lu.locale_t();
    let locinfo = (*loc).locinfo;
    let mut remaining = count;
    let mut lv;
    let mut rv;
    loop {
        lv = towlower_fast_narrow(*l, locinfo);
        rv = towlower_fast_narrow(*r, locinfo);
        l = l.add(1);
        r = r.add(1);
        remaining -= 1;
        if lv != rv || lv == 0 || remaining == 0 {
            break;
        }
    }
    (lv as c_int) - (rv as c_int)
}

/// `__ascii_strnicmp` (i386 `_strnicm.s` / amd64+arm64 `strnicmp.cpp` fallback).
#[no_mangle]
pub unsafe extern "C" fn __ascii_strnicmp(lhs: *const u8, rhs: *const u8, count: usize) -> c_int {
    if count == 0 {
        return 0;
    }
    let mut l = lhs;
    let mut r = rhs;
    let mut remaining = count;
    let mut lv;
    let mut rv;
    loop {
        lv = ascii_tolower(*l);
        rv = ascii_tolower(*r);
        l = l.add(1);
        r = r.add(1);
        remaining -= 1;
        if lv != rv || lv == 0 || remaining == 0 {
            break;
        }
    }
    (lv as c_int) - (rv as c_int)
}

/// `_strnicmp` (strnicmp.cpp).
#[no_mangle]
pub unsafe extern "C" fn _strnicmp(lhs: *const u8, rhs: *const u8, count: usize) -> c_int {
    if crate::locale_update::__acrt_locale_changed_data == 0 {
        if !validate(!lhs.is_null()) {
            return NLSCMPERROR;
        }
        if !validate(!rhs.is_null()) {
            return NLSCMPERROR;
        }
        if !validate(count <= INT_MAX) {
            return NLSCMPERROR;
        }
        return __ascii_strnicmp(lhs, rhs, count);
    }
    _strnicmp_l(lhs, rhs, count, core::ptr::null_mut())
}

// ---------------------------------------------------------------------------
// wcsicmp
// ---------------------------------------------------------------------------

/// `_wcsicmp_l` (wcsicmp.cpp). Takes the ascii fast path when the locale's
/// `LC_CTYPE` name is NULL ("C" locale by name), unlike the narrow forms.
#[no_mangle]
pub unsafe extern "C" fn _wcsicmp_l(
    lhs: *const u16,
    rhs: *const u16,
    plocinfo: *mut CrtLocalePointers,
) -> c_int {
    if !validate(!lhs.is_null()) {
        return NLSCMPERROR;
    }
    if !validate(!rhs.is_null()) {
        return NLSCMPERROR;
    }
    let mut lu = LocaleUpdate::new(plocinfo);
    let loc = lu.locale_t();
    if (*(*loc).locinfo).locale_name[CRT_LC_CTYPE as usize].is_null() {
        return __ascii_wcsicmp(lhs, rhs);
    }
    let mut l = lhs;
    let mut r = rhs;
    let mut lv;
    let mut rv;
    loop {
        lv = towlower_internal(*l, loc);
        rv = towlower_internal(*r, loc);
        l = l.add(1);
        r = r.add(1);
        if lv != rv || lv == 0 {
            break;
        }
    }
    (lv as c_int) - (rv as c_int)
}

/// `__ascii_wcsicmp` (wcsicmp.cpp).
#[no_mangle]
pub unsafe extern "C" fn __ascii_wcsicmp(lhs: *const u16, rhs: *const u16) -> c_int {
    let mut l = lhs;
    let mut r = rhs;
    let mut lv;
    let mut rv;
    loop {
        lv = ascii_towlower(*l);
        rv = ascii_towlower(*r);
        l = l.add(1);
        r = r.add(1);
        if lv != rv || lv == 0 {
            break;
        }
    }
    (lv as c_int) - (rv as c_int)
}

/// `_wcsicmp` (wcsicmp.cpp).
#[no_mangle]
pub unsafe extern "C" fn _wcsicmp(lhs: *const u16, rhs: *const u16) -> c_int {
    if crate::locale_update::__acrt_locale_changed_data == 0 {
        if !validate(!lhs.is_null()) {
            return NLSCMPERROR;
        }
        if !validate(!rhs.is_null()) {
            return NLSCMPERROR;
        }
        return __ascii_wcsicmp(lhs, rhs);
    }
    _wcsicmp_l(lhs, rhs, core::ptr::null_mut())
}

// ---------------------------------------------------------------------------
// wcsnicmp
// ---------------------------------------------------------------------------

/// `_wcsnicmp_l` (wcsnicmp.cpp).
#[no_mangle]
pub unsafe extern "C" fn _wcsnicmp_l(
    lhs: *const u16,
    rhs: *const u16,
    count: usize,
    plocinfo: *mut CrtLocalePointers,
) -> c_int {
    if !validate(!lhs.is_null()) {
        return NLSCMPERROR;
    }
    if !validate(!rhs.is_null()) {
        return NLSCMPERROR;
    }
    if count == 0 {
        return 0;
    }
    let mut lu = LocaleUpdate::new(plocinfo);
    let loc = lu.locale_t();
    if (*(*loc).locinfo).locale_name[CRT_LC_CTYPE as usize].is_null() {
        return __ascii_wcsnicmp(lhs, rhs, count);
    }
    let mut l = lhs;
    let mut r = rhs;
    let mut remaining = count;
    let mut lv;
    let mut rv;
    loop {
        lv = towlower_internal(*l, loc);
        rv = towlower_internal(*r, loc);
        l = l.add(1);
        r = r.add(1);
        remaining -= 1;
        if lv != rv || lv == 0 || remaining == 0 {
            break;
        }
    }
    (lv as c_int) - (rv as c_int)
}

/// `__ascii_wcsnicmp` (wcsnicmp.cpp).
#[no_mangle]
pub unsafe extern "C" fn __ascii_wcsnicmp(lhs: *const u16, rhs: *const u16, count: usize) -> c_int {
    if count == 0 {
        return 0;
    }
    let mut l = lhs;
    let mut r = rhs;
    let mut remaining = count;
    let mut lv;
    let mut rv;
    loop {
        lv = ascii_towlower(*l);
        rv = ascii_towlower(*r);
        l = l.add(1);
        r = r.add(1);
        remaining -= 1;
        if lv != rv || lv == 0 || remaining == 0 {
            break;
        }
    }
    (lv as c_int) - (rv as c_int)
}

/// `_wcsnicmp` (wcsnicmp.cpp).
#[no_mangle]
pub unsafe extern "C" fn _wcsnicmp(lhs: *const u16, rhs: *const u16, count: usize) -> c_int {
    if crate::locale_update::__acrt_locale_changed_data == 0 {
        if !validate(!lhs.is_null()) {
            return NLSCMPERROR;
        }
        if !validate(!rhs.is_null()) {
            return NLSCMPERROR;
        }
        return __ascii_wcsnicmp(lhs, rhs, count);
    }
    _wcsnicmp_l(lhs, rhs, count, core::ptr::null_mut())
}
