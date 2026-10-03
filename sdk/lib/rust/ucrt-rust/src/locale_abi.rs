//! The locale takeover's frozen interface: every internal symbol the locale
//! pieces call across each other, declared ONCE.
//!
//! The locale C (`sdk/lib/ucrt/locale/*.cpp`, `mbstring/mbctype.cpp`) is being
//! replaced in pieces, and ~160 other ucrt objects keep calling its internals
//! by their C names while that happens. So every symbol below is plain C ABI
//! whichever side defines it: a Rust piece that DEFINES one does so with
//! `#[no_mangle] pub unsafe extern "C"` and the exact signature here; a piece
//! that CALLS one imports it from this module and never re-declares it. Until
//! its owner lands, the C answers; afterwards the Rust does, with no change at
//! any call site.
//!
//! Ownership (the Rust module that will define each, and the C file it replaces):
//!
//! | owner module          | replaces                                               |
//! |-----------------------|--------------------------------------------------------|
//! | `locale_data`         | ctype.cpp, nlsdata.cpp, localeconv.cpp, glstatus.cpp,  |
//! |                       | lconv_unsigned_char_initialization.cpp,                |
//! |                       | lcidtoname_downlevel.cpp                               |
//! | `locale_init`         | initctype.cpp, initmon.cpp, initnum.cpp, inittime.cpp  |
//! | `nls_wrappers`        | CompareStringA/W, LCMapStringA/W, GetLocaleInfoA,      |
//! |                       | GetStringTypeA (.cpp)                                  |
//! | `mbctype`             | mbstring/mbctype.cpp                                   |
//! | `qualified_locale`    | get_qualified_locale.cpp, getqloc_downlevel.cpp        |
//! | `setlocale`           | wsetlocale.cpp, setlocale.cpp,                         |
//! |                       | locale_refcounting.cpp, locale_update.cpp              |
//!
//! Types come from `crate::locale`, whose layouts are pinned per arch. A
//! signature here that disagrees with the C is a silent ABI break, so each one
//! is transcribed from `corecrt_internal.h` (or the defining .cpp where the
//! header has no prototype) and says which.

#![allow(dead_code)]

use crate::locale::{
    CrtLocaleData, CrtLocalePointers, CrtMultibyteData, LcTimeData, Lconv, LocaleStrings,
    LocaleStringTable,
};
use core::ffi::{c_char, c_int, c_long, c_uint, c_void};

/// `_locale_t`
pub type LocaleT = *mut CrtLocalePointers;

extern "C" {
    // ── owner: locale_data ──────────────────────────────────────────────────
    // (data symbols: define with `#[no_mangle] pub static` / `static mut`)

    /// ctype.cpp: the C locale's classification table, 384 entries; `_pctype`
    /// points 128 in (so it can be indexed by -128..=255).
    pub static __newctype: [u16; 384];
    /// ctype.cpp: the C locale's lower/upper maps, 384 entries each.
    pub static __newclmap: [u8; 384];
    pub static __newcumap: [u8; 384];
    /// ctype.cpp: the wide table (`_wctype`, exported) and the two exported
    /// pointers into the narrow and wide ones.
    pub static _wctype: [u16; 258];
    pub static mut _pctype: *const u16;
    pub static mut _pwctype: *const u16;
    /// nlsdata.cpp: exported, initially 1.
    pub static mut __mb_cur_max: c_int;
    /// nlsdata.cpp
    pub static mut __acrt_initial_locale_data: CrtLocaleData;
    pub static mut __acrt_initial_locale_pointers: CrtLocalePointers;
    /// nlsdata.cpp: `dual_state_global<__crt_locale_data*>`. State isolation is
    /// compiled out (`state_index_count == 1`), so this is ONE pointer.
    pub static mut __acrt_current_locale_data: *mut CrtLocaleData;
    /// nlsdata.cpp: L"C"
    pub static mut __acrt_wide_c_locale_string: [u16; 2];
    pub static __lc_time_c: LcTimeData;
    /// localeconv.cpp
    pub static mut __acrt_lconv_c: Lconv;
    pub static mut __acrt_lconv: *mut Lconv;
    pub static mut __acrt_lconv_static_decimal: [c_char; 2];
    pub static mut __acrt_lconv_static_null: [c_char; 1];
    pub static mut __acrt_lconv_static_W_decimal: [u16; 2];
    pub static mut __acrt_lconv_static_W_null: [u16; 1];
    /// glstatus.cpp
    pub static mut __globallocalestatus: c_int;
    /// lcidtoname_downlevel.cpp
    pub fn __acrt_DownlevelLocaleNameToLCID(locale_name: *const u16) -> u32;
    pub fn __acrt_DownlevelLCIDToLocaleName(lcid: u32, out: *mut u16, cch: c_int) -> c_int;

    // ── owner: locale_init ──────────────────────────────────────────────────
    // All return 0 on success (corecrt_internal.h).
    pub fn __acrt_locale_initialize_ctype(locale_data: *mut CrtLocaleData) -> c_int;
    pub fn __acrt_locale_initialize_monetary(locale_data: *mut CrtLocaleData) -> c_int;
    pub fn __acrt_locale_initialize_numeric(locale_data: *mut CrtLocaleData) -> c_int;
    pub fn __acrt_locale_initialize_time(locale_data: *mut CrtLocaleData) -> c_int;
    pub fn __acrt_locale_free_monetary(lconv: *mut Lconv);
    pub fn __acrt_locale_free_numeric(lconv: *mut Lconv);
    pub fn __acrt_locale_free_time(lc_time_data: *mut LcTimeData);

    // ── owner: nls_wrappers ─────────────────────────────────────────────────
    pub fn __acrt_CompareStringA(
        locale: LocaleT,
        locale_name: *const u16,
        cmp_flags: u32,
        string1: *const c_char,
        count1: c_int,
        string2: *const c_char,
        count2: c_int,
        code_page: c_int,
    ) -> c_int;
    pub fn __acrt_CompareStringW(
        locale_name: *const u16,
        cmp_flags: u32,
        string1: *const u16,
        count1: c_int,
        string2: *const u16,
        count2: c_int,
    ) -> c_int;
    /// `lc_type` is LC_INT_TYPE / LC_STR_TYPE / LC_WSTR_TYPE; `address` receives
    /// a `char`, a `char*` or a `wchar_t*` accordingly (the strings malloc'd).
    pub fn __acrt_GetLocaleInfoA(
        locale: LocaleT,
        lc_type: c_int,
        locale_name: *const u16,
        field_type: u32,
        address: *mut c_void,
    ) -> c_int;
    /// Returns BOOL.
    pub fn __acrt_GetStringTypeA(
        locale: LocaleT,
        info_type: u32,
        src: *const c_char,
        cch_src: c_int,
        char_type: *mut u16,
        code_page: c_int,
        error: c_int,
    ) -> c_int;
    pub fn __acrt_LCMapStringA(
        locale: LocaleT,
        locale_name: *const u16,
        map_flags: u32,
        src: *const c_char,
        cch_src: c_int,
        dest: *mut c_char,
        cch_dest: c_int,
        code_page: c_int,
        error: c_int,
    ) -> c_int;
    pub fn __acrt_LCMapStringW(
        locale_name: *const u16,
        map_flags: u32,
        src: *const u16,
        cch_src: c_int,
        dest: *mut u16,
        cch_dest: c_int,
    ) -> c_int;

    // ── owner: mbctype ──────────────────────────────────────────────────────
    pub static mut __acrt_initial_multibyte_data: CrtMultibyteData;
    /// `dual_state_global<__crt_multibyte_data*>`: one pointer (see above).
    pub static mut __acrt_current_multibyte_data: *mut CrtMultibyteData;
    /// Returns C++ `bool` (one byte, 0/1).
    pub fn __acrt_initialize_multibyte() -> bool;
    pub fn __acrt_update_thread_multibyte_data() -> *mut CrtMultibyteData;
    /// wsetlocale.cpp's prototype: `int _setmbcp_nolock(int, __crt_multibyte_data*)`.
    pub fn _setmbcp_nolock(code_page: c_int, data: *mut CrtMultibyteData) -> c_int;

    // ── owner: qualified_locale ─────────────────────────────────────────────
    /// Both return BOOL. `code_page` and `result` may be NULL.
    pub fn __acrt_get_qualified_locale(
        input: *const LocaleStrings,
        code_page: *mut c_uint,
        result: *mut LocaleStrings,
    ) -> c_int;
    pub fn __acrt_get_qualified_locale_downlevel(
        input: *const LocaleStrings,
        code_page: *mut c_uint,
        result: *mut LocaleStrings,
    ) -> c_int;
    pub static __acrt_rg_language: [LocaleStringTable; 0];
    pub static __acrt_rg_language_count: usize;
    pub static __acrt_rg_country: [LocaleStringTable; 0];
    pub static __acrt_rg_country_count: usize;

    // ── owner: setlocale ────────────────────────────────────────────────────
    pub static mut __acrt_locale_changed_data: c_long;
    pub fn __acrt_set_locale_changed();
    pub fn __acrt_uninitialize_locale();
    pub fn __acrt_add_locale_ref(data: *mut CrtLocaleData);
    pub fn __acrt_release_locale_ref(data: *mut CrtLocaleData);
    pub fn __acrt_free_locale(data: *mut CrtLocaleData);
    pub fn __acrt_locale_add_lc_time_reference(lc_time: *const LcTimeData) -> c_long;
    pub fn __acrt_locale_release_lc_time_reference(lc_time: *const LcTimeData) -> c_long;
    pub fn __acrt_locale_free_lc_time_if_unreferenced(lc_time: *const LcTimeData);
    pub fn __acrt_update_thread_locale_data() -> *mut CrtLocaleData;
    pub fn _updatetlocinfoEx_nolock(
        data: *mut *mut CrtLocaleData,
        new_data: *mut CrtLocaleData,
    ) -> *mut CrtLocaleData;
    /// Returns a `_malloc_crt` copy, or NULL.
    pub fn __acrt_copy_locale_name(locale_name: *const u16) -> *mut u16;
}
