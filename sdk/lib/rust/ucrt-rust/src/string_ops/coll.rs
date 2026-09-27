//! Collation: `strcoll`/`stricoll`/`strncoll`/`strnicoll`/`wcscoll`/
//! `wcsicoll`/`wcsncoll`/`wcsnicoll` (+ `_l`) and `strxfrm`/`wcsxfrm` (+ `_l`)
//! -- replaces `strcoll.cpp`, `stricoll.cpp`, `strncoll.cpp`, `strnicoll.cpp`,
//! `wcscoll.cpp`, `wcsicoll.cpp`, `wcsncoll.cpp`, `wcsnicoll.cpp`,
//! `strxfrm.cpp`, `wcsxfrm.cpp` in full (18 exports).
//!
//! Every compare-family function returns `ret - 2` on a successful
//! `__acrt_CompareString{A,W}` call: Win32's `CompareString` returns 1/2/3 for
//! less/equal/greater, and UCRT re-bases that to -1/0/1 with one subtraction
//! rather than a branch.
//!
//! ## Details that are silent if wrong
//!
//! * **The "C locale" fallback callee is NOT uniform across this family --
//!   transcribed per function, not inferred:**
//!   - `strcoll`/`_strcoll_l`: `strcmp` (this crate's `plain::strcmp`).
//!   - `_stricoll`/`_stricoll_l`: `_stricmp` (the TOP-level, locale-aware
//!     `icmp::_stricmp` -- NOT `__ascii_stricmp`), from BOTH the plain
//!     fast-path and the `_l` worker's own "C" branch.
//!   - `strncoll`/`_strncoll_l`: `strncmp` (`plain::strncmp`).
//!   - `_strnicoll`/`_strnicoll_l` DIFFER FROM EACH OTHER: the plain
//!     fast-path calls `_strnicmp` (`icmp::_strnicmp`, the locale-aware top
//!     level), but the `_l` worker's "C" branch calls `_strnicmp_l(...,
//!     resolved_locale)` (`icmp::_strnicmp_l`) -- passing the ALREADY-
//!     resolved locale through rather than re-resolving via NULL.
//!   - `wcscoll`/`_wcscoll_l`: `wcscmp` (`plain::wcscmp`).
//!   - `_wcsicoll`/`_wcsicoll_l`: **`__ascii_wcsicmp` directly**, from BOTH
//!     branches (unlike `stricoll`, this one bypasses the locale-aware
//!     `_wcsicmp` entirely).
//!   - `wcsncoll`/`_wcsncoll_l`: `wcsncmp` (`plain::wcsncmp`).
//!   - `_wcsnicoll`/`_wcsnicoll_l`: **`__ascii_wcsnicmp` directly**, both
//!     branches (same shape as `_wcsicoll`).
//! * **`strcoll`'s plain wrapper has NO fast path of its own** -- unlike
//!   every other function in this family, `strcoll()` unconditionally calls
//!   `_strcoll_l(s1, s2, NULL)` rather than checking
//!   `__acrt_locale_changed_data` itself first. The two are observably
//!   identical (`LocaleUpdate::new(NULL)` is cheap when the locale was never
//!   changed), but the C's shape -- one fewer `if` -- is kept.
//! * **`_stricoll`'s and `_wcsicoll`'s C-locale-changed fast path re-validates
//!   its own arguments** (`_VALIDATE_RETURN` before the `_stricmp`/
//!   `__ascii_wcsicmp` call) even though the callee ALSO validates; `_strcoll`
//!   has no such fast path to duplicate the check in.
//! * **`_strncoll`/`_wcsncoll`'s fast path validates including `count <=
//!   INT_MAX`**, matching `_strnicmp`/`_wcsnicmp`'s own validation --
//!   redundant with the callee, kept anyway.
//! * **`count == 0` short-circuits to `0` before any validation** in every
//!   `_l` counted form (`_strncoll_l`, `_strnicoll_l`, `_wcsncoll_l`,
//!   `_wcsnicoll_l`) -- a NULL string with `count == 0` is NOT an error here
//!   (contrast `icmp.rs`'s `_strnicmp_l`, which validates pointers
//!   unconditionally before its own `count == 0` check. Transcribed exactly
//!   per function -- the two families disagree and both are kept as read).
//! * **On a `CompareString*` failure (`ret == 0`), `errno` is unconditionally
//!   set to `EINVAL`** (not `EILSEQ` -- contrast `strxfrm`/`wcsxfrm`, which
//!   use `EILSEQ` for the analogous `LCMapString*` failure), and the return
//!   is `_NLSCMPERROR` (`INT_MAX`), not `-1`/`0`/`1`.
//! * **`strxfrm`'s C-locale test is a COMPOUND condition**:
//!   `locale_name[LC_COLLATE] == NULL && lc_collate_cp == CP_ACP` -- both
//!   must hold. `wcsxfrm`'s C-locale test is `locale_name[LC_COLLATE] ==
//!   NULL` ALONE, with no codepage check at all. This asymmetry is read
//!   directly from the two `.cpp` files, not inferred from the compare
//!   family (whose C-locale tests are all name-only, no codepage check
//!   anywhere).
//! * **`strxfrm`'s C-locale path is `strncpy(_string1, _string2, _count)` +
//!   `return strlen(_string2)`** -- NOT `strcpy_s`; the destination is
//!   NUL-padded/truncated exactly like `strncpy` (including strncpy's classic
//!   no-NUL-if-source-too-long misfeature when `_count <= strlen(_string2)`),
//!   and the returned length is the SOURCE's length regardless of whether it
//!   fit. `wcsxfrm`'s C-locale path is the same shape with `wcsncpy`/`wcslen`.
//! * **`strxfrm`'s locale-aware path is a plain two-call LCMapString-sizes-
//!   then-fills shape** (`LCMAP_SORTKEY`), like `case.rs`'s case-mapping --
//!   but `wcsxfrm`'s locale-aware path is NOT: `LCMAP_SORTKEY` against a
//!   *wide* string still returns a BYTE-length sort key, so after the second
//!   (fill) call, `_wcsxfrm_l` walks the BYTES of its own destination buffer
//!   backward, widening each byte into its own `wchar_t` slot in place
//!   (`_string1[i] = (wchar_t)((unsigned char*)_string1)[i]`, high index
//!   first so the widening in-place expansion never reads a slot it has not
//!   widened yet). Transcribed with the identical pointer casts and
//!   iteration order; this is genuinely Microsoft's own algorithm, not an
//!   invented equivalent.
//! * **`strxfrm`/`wcsxfrm` return `INT_MAX` on any error** (OM or API), and
//!   on `ERANGE` (buffer too small) return `size - 1` -- the transformed
//!   length WITHOUT the NUL, exactly like `strlen`'s contract, matching
//!   `1 + strxfrm(NULL, string, 0)` being "the size of the array needed".
//! * **`_count <= INT_MAX` is validated for both compare (`_strncoll_l` etc.)
//!   and transform (`_strxfrm_l`/`_wcsxfrm_l`)** -- `size_t` on the wire, but
//!   the Win32 calls underneath take `int`.
//! * **`_strxfrm_l` constructs its `_LocaleUpdate` BEFORE validating its
//!   arguments; `_wcsxfrm_l` validates FIRST, then constructs it** -- read
//!   directly off the two `.cpp` files, not a typo in one of them. The
//!   difference is unobservable through the return value or errno (a failed
//!   validation returns before the locale resolution's result is ever used
//!   either way) but does change whether the per-thread "own_locale" bit
//!   (`LocaleUpdate`'s arm 3 side effect) gets touched on a call that fails
//!   validation. Kept exactly rather than normalized to one order.
use crate::locale::CRT_LC_COLLATE;
use crate::locale_abi::{LocaleT, __acrt_CompareStringA, __acrt_CompareStringW, __acrt_LCMapStringA, __acrt_LCMapStringW};
use crate::locale_update::{LocaleUpdate, __acrt_locale_changed_data};
use crate::string_ops::icmp::{__ascii_wcsicmp, __ascii_wcsnicmp, _stricmp, _strnicmp, _strnicmp_l};
use crate::string_ops::plain::{strcmp, strncmp, wcscmp, wcsncmp};
use core::ffi::{c_char, c_int};

const EINVAL: c_int = 22;
const EILSEQ: c_int = 42;
const ERANGE: c_int = 34;
const NLSCMPERROR: c_int = i32::MAX;
const INT_MAX: usize = i32::MAX as usize;
const SORT_STRINGSORT: u32 = 4096;
const NORM_IGNORECASE: u32 = 1;
const LCMAP_SORTKEY: u32 = 0x0400;
/// `CP_ACP` (winnls.h) -- the "default ANSI code page" sentinel. Only
/// `strxfrm`'s C-locale test uses it (see module doc); `wcsxfrm`'s does not.
const CP_ACP: c_int = 0;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    /// Matches `mbstring/modify.rs`'s existing declaration exactly (`u8`, not
    /// `c_char`) -- `#[deny(clashing_extern_declarations)]` is crate-wide.
    fn strncpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8;
    fn strlen(s: *const c_char) -> usize;
    fn wcslen(s: *const u16) -> usize;
}
// `wcsncpy` is THIS crate's own `plain::wcsncpy` (`#[no_mangle]`, same
// binary) -- called directly rather than re-declared as an extern import, so
// there is nothing to clash.
use crate::string_ops::plain::wcsncpy;

#[inline]
unsafe fn validate(e: bool) -> bool {
    if !e {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo();
    }
    e
}

// ---------------------------------------------------------------------------
// strcoll / _strcoll_l
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn _strcoll_l(
    string1: *const c_char,
    string2: *const c_char,
    plocinfo: LocaleT,
) -> c_int {
    let mut lu = LocaleUpdate::new(plocinfo);
    if !validate(!string1.is_null()) || !validate(!string2.is_null()) {
        return NLSCMPERROR;
    }
    let loc = lu.locale_t();
    let locinfo = (*loc).locinfo;
    if (*locinfo).locale_name[CRT_LC_COLLATE as usize].is_null() {
        return strcmp(string1, string2);
    }
    let ret = __acrt_CompareStringA(
        loc,
        (*locinfo).locale_name[CRT_LC_COLLATE as usize],
        SORT_STRINGSORT,
        string1,
        -1,
        string2,
        -1,
        (*locinfo).lc_collate_cp as c_int,
    );
    if ret == 0 {
        *_errno() = EINVAL;
        return NLSCMPERROR;
    }
    ret - 2
}

/// `strcoll` has NO fast path of its own -- see module doc.
#[no_mangle]
pub unsafe extern "C" fn strcoll(string1: *const c_char, string2: *const c_char) -> c_int {
    _strcoll_l(string1, string2, core::ptr::null_mut())
}

// ---------------------------------------------------------------------------
// _stricoll / _stricoll_l
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn _stricoll_l(
    string1: *const c_char,
    string2: *const c_char,
    plocinfo: LocaleT,
) -> c_int {
    let mut lu = LocaleUpdate::new(plocinfo);
    if !validate(!string1.is_null()) || !validate(!string2.is_null()) {
        return NLSCMPERROR;
    }
    let loc = lu.locale_t();
    let locinfo = (*loc).locinfo;
    if (*locinfo).locale_name[CRT_LC_COLLATE as usize].is_null() {
        return _stricmp(string1 as *const u8, string2 as *const u8);
    }
    let ret = __acrt_CompareStringA(
        loc,
        (*locinfo).locale_name[CRT_LC_COLLATE as usize],
        SORT_STRINGSORT | NORM_IGNORECASE,
        string1,
        -1,
        string2,
        -1,
        (*locinfo).lc_collate_cp as c_int,
    );
    if ret == 0 {
        *_errno() = EINVAL;
        return NLSCMPERROR;
    }
    ret - 2
}

#[no_mangle]
pub unsafe extern "C" fn _stricoll(string1: *const c_char, string2: *const c_char) -> c_int {
    if __acrt_locale_changed_data == 0 {
        return _stricmp(string1 as *const u8, string2 as *const u8);
    }
    _stricoll_l(string1, string2, core::ptr::null_mut())
}

// ---------------------------------------------------------------------------
// _strncoll / _strncoll_l
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn _strncoll_l(
    string1: *const c_char,
    string2: *const c_char,
    count: usize,
    plocinfo: LocaleT,
) -> c_int {
    if count == 0 {
        return 0;
    }
    if !validate(!string1.is_null()) || !validate(!string2.is_null()) || !validate(count <= INT_MAX) {
        return NLSCMPERROR;
    }
    let mut lu = LocaleUpdate::new(plocinfo);
    let loc = lu.locale_t();
    let locinfo = (*loc).locinfo;
    if (*locinfo).locale_name[CRT_LC_COLLATE as usize].is_null() {
        return strncmp(string1, string2, count);
    }
    let ret = __acrt_CompareStringA(
        loc,
        (*locinfo).locale_name[CRT_LC_COLLATE as usize],
        SORT_STRINGSORT,
        string1,
        count as c_int,
        string2,
        count as c_int,
        (*locinfo).lc_collate_cp as c_int,
    );
    if ret == 0 {
        *_errno() = EINVAL;
        return NLSCMPERROR;
    }
    ret - 2
}

#[no_mangle]
pub unsafe extern "C" fn _strncoll(string1: *const c_char, string2: *const c_char, count: usize) -> c_int {
    if __acrt_locale_changed_data == 0 {
        if !validate(!string1.is_null()) || !validate(!string2.is_null()) || !validate(count <= INT_MAX) {
            return NLSCMPERROR;
        }
        return strncmp(string1, string2, count);
    }
    _strncoll_l(string1, string2, count, core::ptr::null_mut())
}

// ---------------------------------------------------------------------------
// _strnicoll / _strnicoll_l
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn _strnicoll_l(
    string1: *const c_char,
    string2: *const c_char,
    count: usize,
    plocinfo: LocaleT,
) -> c_int {
    let mut lu = LocaleUpdate::new(plocinfo);
    if count == 0 {
        return 0;
    }
    if !validate(!string1.is_null()) || !validate(!string2.is_null()) || !validate(count <= INT_MAX) {
        return NLSCMPERROR;
    }
    let loc = lu.locale_t();
    let locinfo = (*loc).locinfo;
    if (*locinfo).locale_name[CRT_LC_COLLATE as usize].is_null() {
        // NOTE: the resolved locale is threaded through, not NULL.
        return _strnicmp_l(string1 as *const u8, string2 as *const u8, count, loc);
    }
    let ret = __acrt_CompareStringA(
        loc,
        (*locinfo).locale_name[CRT_LC_COLLATE as usize],
        SORT_STRINGSORT | NORM_IGNORECASE,
        string1,
        count as c_int,
        string2,
        count as c_int,
        (*locinfo).lc_collate_cp as c_int,
    );
    if ret == 0 {
        *_errno() = EINVAL;
        return NLSCMPERROR;
    }
    ret - 2
}

#[no_mangle]
pub unsafe extern "C" fn _strnicoll(string1: *const c_char, string2: *const c_char, count: usize) -> c_int {
    if __acrt_locale_changed_data == 0 {
        return _strnicmp(string1 as *const u8, string2 as *const u8, count);
    }
    _strnicoll_l(string1, string2, count, core::ptr::null_mut())
}

// ---------------------------------------------------------------------------
// wcscoll / _wcscoll_l
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn _wcscoll_l(string1: *const u16, string2: *const u16, plocinfo: LocaleT) -> c_int {
    let mut lu = LocaleUpdate::new(plocinfo);
    if !validate(!string1.is_null()) || !validate(!string2.is_null()) {
        return NLSCMPERROR;
    }
    let loc = lu.locale_t();
    let locinfo = (*loc).locinfo;
    if (*locinfo).locale_name[CRT_LC_COLLATE as usize].is_null() {
        return wcscmp(string1, string2);
    }
    let ret = __acrt_CompareStringW(
        (*locinfo).locale_name[CRT_LC_COLLATE as usize],
        SORT_STRINGSORT,
        string1,
        -1,
        string2,
        -1,
    );
    if ret == 0 {
        *_errno() = EINVAL;
        return NLSCMPERROR;
    }
    ret - 2
}

#[no_mangle]
pub unsafe extern "C" fn wcscoll(string1: *const u16, string2: *const u16) -> c_int {
    if __acrt_locale_changed_data == 0 {
        if !validate(!string1.is_null()) || !validate(!string2.is_null()) {
            return NLSCMPERROR;
        }
        return wcscmp(string1, string2);
    }
    _wcscoll_l(string1, string2, core::ptr::null_mut())
}

// ---------------------------------------------------------------------------
// _wcsicoll / _wcsicoll_l
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn _wcsicoll_l(string1: *const u16, string2: *const u16, plocinfo: LocaleT) -> c_int {
    let mut lu = LocaleUpdate::new(plocinfo);
    if !validate(!string1.is_null()) || !validate(!string2.is_null()) {
        return NLSCMPERROR;
    }
    let loc = lu.locale_t();
    let locinfo = (*loc).locinfo;
    if (*locinfo).locale_name[CRT_LC_COLLATE as usize].is_null() {
        return __ascii_wcsicmp(string1, string2);
    }
    let ret = __acrt_CompareStringW(
        (*locinfo).locale_name[CRT_LC_COLLATE as usize],
        SORT_STRINGSORT | NORM_IGNORECASE,
        string1,
        -1,
        string2,
        -1,
    );
    if ret == 0 {
        *_errno() = EINVAL;
        return NLSCMPERROR;
    }
    ret - 2
}

#[no_mangle]
pub unsafe extern "C" fn _wcsicoll(string1: *const u16, string2: *const u16) -> c_int {
    if __acrt_locale_changed_data == 0 {
        if !validate(!string1.is_null()) || !validate(!string2.is_null()) {
            return NLSCMPERROR;
        }
        return __ascii_wcsicmp(string1, string2);
    }
    _wcsicoll_l(string1, string2, core::ptr::null_mut())
}

// ---------------------------------------------------------------------------
// _wcsncoll / _wcsncoll_l
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn _wcsncoll_l(
    string1: *const u16,
    string2: *const u16,
    count: usize,
    plocinfo: LocaleT,
) -> c_int {
    if count == 0 {
        return 0;
    }
    let mut lu = LocaleUpdate::new(plocinfo);
    if !validate(!string1.is_null()) || !validate(!string2.is_null()) || !validate(count <= INT_MAX) {
        return NLSCMPERROR;
    }
    let loc = lu.locale_t();
    let locinfo = (*loc).locinfo;
    if (*locinfo).locale_name[CRT_LC_COLLATE as usize].is_null() {
        return wcsncmp(string1, string2, count);
    }
    let ret = __acrt_CompareStringW(
        (*locinfo).locale_name[CRT_LC_COLLATE as usize],
        SORT_STRINGSORT,
        string1,
        count as c_int,
        string2,
        count as c_int,
    );
    if ret == 0 {
        *_errno() = EINVAL;
        return NLSCMPERROR;
    }
    ret - 2
}

#[no_mangle]
pub unsafe extern "C" fn _wcsncoll(string1: *const u16, string2: *const u16, count: usize) -> c_int {
    if __acrt_locale_changed_data == 0 {
        if !validate(!string1.is_null()) || !validate(!string2.is_null()) || !validate(count <= INT_MAX) {
            return NLSCMPERROR;
        }
        return wcsncmp(string1, string2, count);
    }
    _wcsncoll_l(string1, string2, count, core::ptr::null_mut())
}

// ---------------------------------------------------------------------------
// _wcsnicoll / _wcsnicoll_l
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn _wcsnicoll_l(
    string1: *const u16,
    string2: *const u16,
    count: usize,
    plocinfo: LocaleT,
) -> c_int {
    if count == 0 {
        return 0;
    }
    if !validate(!string1.is_null()) || !validate(!string2.is_null()) || !validate(count <= INT_MAX) {
        return NLSCMPERROR;
    }
    let mut lu = LocaleUpdate::new(plocinfo);
    let loc = lu.locale_t();
    let locinfo = (*loc).locinfo;
    if (*locinfo).locale_name[CRT_LC_COLLATE as usize].is_null() {
        return __ascii_wcsnicmp(string1, string2, count);
    }
    let ret = __acrt_CompareStringW(
        (*locinfo).locale_name[CRT_LC_COLLATE as usize],
        SORT_STRINGSORT | NORM_IGNORECASE,
        string1,
        count as c_int,
        string2,
        count as c_int,
    );
    if ret == 0 {
        *_errno() = EINVAL;
        return NLSCMPERROR;
    }
    ret - 2
}

#[no_mangle]
pub unsafe extern "C" fn _wcsnicoll(string1: *const u16, string2: *const u16, count: usize) -> c_int {
    if __acrt_locale_changed_data == 0 {
        if !validate(!string1.is_null()) || !validate(!string2.is_null()) || !validate(count <= INT_MAX) {
            return NLSCMPERROR;
        }
        return __ascii_wcsnicmp(string1, string2, count);
    }
    _wcsnicoll_l(string1, string2, count, core::ptr::null_mut())
}

// ---------------------------------------------------------------------------
// strxfrm / _strxfrm_l
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn _strxfrm_l(
    string1: *mut c_char,
    string2: *const c_char,
    count: usize,
    plocinfo: LocaleT,
) -> usize {
    // `_LocaleUpdate` is constructed BEFORE validation in the C (it is an
    // unconditional side effect -- see `wcsxfrm`'s twin below, same order),
    // so this mirrors that rather than short-circuiting on validation first.
    let mut lu = LocaleUpdate::new(plocinfo);
    if !validate(count <= INT_MAX) {
        return NLSCMPERROR as usize;
    }
    if !validate(!string1.is_null() || count == 0) {
        return NLSCMPERROR as usize;
    }
    if !validate(!string2.is_null()) {
        return NLSCMPERROR as usize;
    }

    if !string1.is_null() && count > 0 {
        *string1 = 0;
    }

    let loc = lu.locale_t();
    let locinfo = (*loc).locinfo;

    let code_page = (*locinfo).lc_collate_cp as c_int;
    if (*locinfo).locale_name[CRT_LC_COLLATE as usize].is_null() && code_page == CP_ACP {
        strncpy(string1 as *mut u8, string2 as *const u8, count);
        return strlen(string2);
    }
    let dstlen = __acrt_LCMapStringA(
        loc,
        (*locinfo).locale_name[CRT_LC_COLLATE as usize],
        LCMAP_SORTKEY,
        string2,
        -1,
        core::ptr::null_mut(),
        0,
        code_page,
        1,
    );
    if dstlen == 0 {
        *_errno() = EILSEQ;
        return NLSCMPERROR as usize;
    }
    let mut retval = dstlen as usize;
    if retval > count {
        if !string1.is_null() && count > 0 {
            *string1 = 0;
            *_errno() = ERANGE;
        }
        return retval - 1;
    }
    if __acrt_LCMapStringA(
        loc,
        (*locinfo).locale_name[CRT_LC_COLLATE as usize],
        LCMAP_SORTKEY,
        string2,
        -1,
        string1,
        count as c_int,
        code_page,
        1,
    ) == 0
    {
        *_errno() = EILSEQ;
        return NLSCMPERROR as usize;
    }
    retval -= 1;
    retval
}

#[no_mangle]
pub unsafe extern "C" fn strxfrm(string1: *mut c_char, string2: *const c_char, count: usize) -> usize {
    _strxfrm_l(string1, string2, count, core::ptr::null_mut())
}

// ---------------------------------------------------------------------------
// wcsxfrm / _wcsxfrm_l
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn _wcsxfrm_l(
    string1: *mut u16,
    string2: *const u16,
    count: usize,
    plocinfo: LocaleT,
) -> usize {
    if !validate(count <= INT_MAX) {
        return NLSCMPERROR as usize;
    }
    if !validate(!string1.is_null() || count == 0) {
        return NLSCMPERROR as usize;
    }
    if !validate(!string2.is_null()) {
        return NLSCMPERROR as usize;
    }

    let mut lu = LocaleUpdate::new(plocinfo);
    let loc = lu.locale_t();
    let locinfo = (*loc).locinfo;

    if (*locinfo).locale_name[CRT_LC_COLLATE as usize].is_null() {
        wcsncpy(string1, string2, count);
        return wcslen(string2);
    }

    let mut size = __acrt_LCMapStringW(
        (*locinfo).locale_name[CRT_LC_COLLATE as usize],
        LCMAP_SORTKEY,
        string2,
        -1,
        core::ptr::null_mut(),
        0,
    );
    if size == 0 {
        *_errno() = EILSEQ;
        return i32::MAX as usize;
    }
    if size as usize <= count {
        size = __acrt_LCMapStringW(
            (*locinfo).locale_name[CRT_LC_COLLATE as usize],
            LCMAP_SORTKEY,
            string2,
            -1,
            string1,
            count as c_int,
        );
        if size == 0 {
            *_errno() = EILSEQ;
            return i32::MAX as usize;
        }
        // Widen the sort key's bytes into wchar_t slots, in place, from the
        // HIGH index down -- `size` bytes were written starting at
        // `string1`'s address; each byte at index i becomes the wchar_t at
        // index i. Highest index first so a not-yet-widened byte is never
        // clobbered by an earlier widen (matches the C's cast/iteration
        // order exactly, including its own `size--` before the loop).
        size -= 1;
        let mut remaining = size as usize;
        let bytes = string1 as *mut u8;
        while remaining > 0 {
            remaining -= 1;
            let b = *bytes.add(remaining);
            *string1.add(remaining) = b as u16;
        }
    } else {
        if !string1.is_null() && count > 0 {
            *string1 = 0;
            *_errno() = ERANGE;
        }
        size -= 1;
    }
    size as usize
}

#[no_mangle]
pub unsafe extern "C" fn wcsxfrm(string1: *mut u16, string2: *const u16, count: usize) -> usize {
    _wcsxfrm_l(string1, string2, count, core::ptr::null_mut())
}
