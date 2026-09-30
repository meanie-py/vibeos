//! Qualifying a partial locale name -- `__acrt_get_qualified_locale` (locale
//! names, EnumSystemLocalesEx) and `__acrt_get_qualified_locale_downlevel`
//! (LCIDs, EnumSystemLocalesW), plus the legacy name tables they share.
//! Replaces `sdk/lib/ucrt/locale/get_qualified_locale.cpp` and
//! `getqloc_downlevel.cpp` in full.
//!
//! `setlocale` hands in `{language, country, codepage}` strings parsed from
//! e.g. "English_United States.1252"; these find the best installed locale,
//! write its name into the thread's `_setloc_data._cacheLocaleName`, and
//! return the canonical English names and codepage. The matching runs inside
//! enumeration callbacks, which carry no context of their own: all their
//! state lives in the calling thread's `__acrt_ptd` (`_setloc_data`, and for
//! the downlevel form `_setloc_downlevel_data`), reached again through
//! `__acrt_getptd()` from inside each callback, exactly as the C does.
//!
//! Every access to that state goes through raw pointers: `wsetlocale` passes
//! the SAME struct as input and output (`__acrt_get_qualified_locale(&names,
//! pcachecp, &names)`), and the callbacks re-enter the ptd while the caller
//! holds a pointer into it, so no `&mut` may be formed across either.
//!
//! ## Details that are silent if wrong
//!
//! * **`TranslateName` is a binary search with `_wcsicmp`**, over tables that
//!   must stay sorted case-insensitively; the language table is tried only
//!   AFTER the untranslated language failed, the country table always first.
//! * **`bAbbrevLanguage`/`bAbbrevCountry` mean "exactly 3 characters"**, which
//!   switches the comparison to LOCALE_SABBREV*NAME ("ENU", "USA"); a 3-letter
//!   BCP-47 language therefore matches the Windows abbreviation, not the ISO one.
//! * **`iPrimaryLen` is 2 for an abbreviation**, else the length of the leading
//!   ASCII-letter run ("english-us" -> 7).
//! * **The iLocState bits**: FULL 0x4, PRIMARY 0x2, DEFAULT 0x1, LANGUAGE
//!   0x100, EXISTS 0x200. Language+country succeeds only with LANGUAGE AND
//!   EXISTS AND any of FULL/PRIMARY/DEFAULT; language-only needs FULL.
//! * **An empty language takes the USER default locale** and the process ACP
//!   (`GetACP`, not the locale's) unless a codepage was given.
//! * **Codepage strings**: "" / "ACP" -> the locale's ANSI codepage, "OCP" ->
//!   its OEM codepage, both mapping a value < 3 to CP_UTF8; "utf8"/"utf-8"
//!   (case-insensitive) -> 65001; anything else `_wtol`. "ACP"/"OCP" are
//!   CASE-SENSITIVE (`wcscmp`). 0, CP_UTF7 and `!IsValidCodePage((WORD)cp)` fail.
//! * **UTF-8 is reported as "utf8"**, others via `_itow_s` base 10; a country
//!   name containing '_' or '.' is replaced by its abbreviation.
//! * **The downlevel form differs**: an empty language with a country searches
//!   by country; a locale with ANSI codepage 0 gets `GetACP()` (not UTF-8);
//!   "utf8" is `_wtol`'d to 0 and fails; UTF-7 is NOT refused; the result must
//!   also pass `IsValidLocale(LCID_INSTALLED)`; `iLocState` and
//!   `_cacheLocaleName` are NOT cleared on entry (the cache is overwritten via
//!   `__acrt_LCIDToLocaleName` only on success).
//! * **`_setloc_downlevel_data` points at a STACK local** of the downlevel call
//!   and is left dangling on return (the C never clears it; nothing reads it
//!   outside the callbacks). Transcribed rather than heap-allocated.
//! * **`lpInStr` is dereferenced before its NULL test** (both forms), so the
//!   later `lpInStr ? ... : nullptr` is dead; kept as written.
//! * **Dead branches kept**: the `else if` after "language matches" in the
//!   downlevel callbacks repeats the same `_wcsicmp` and can never be taken.
//!
//! Symbols: `__acrt_rg_language`/`__acrt_rg_country` (+ `_count`),
//! `__rglangidNotDefault` (the C's `__declspec(selectany)` global), and the
//! two entry points.

#![allow(non_upper_case_globals)]

use crate::locale::{
    LocaleStringTable, LocaleStrings, QualifiedLocaleData, QualifiedLocaleDataDownlevel,
    LOCALE_NAME_MAX_LENGTH, MAX_CP_LEN, MAX_CTRY_LEN, MAX_LANG_LEN,
};
use crate::locale_update::__acrt_getptd;
use core::ffi::{c_int, c_long, c_uint, c_void};
use core::ptr::{addr_of, addr_of_mut, null, null_mut};

// ── constants ────────────────────────────────────────────────────────────────

/// get_qualified_locale.cpp:22-26 / getqloc_downlevel.cpp:18-22 (same values).
const QLOC_DEFAULT: c_int = 0x1;
const QLOC_PRIMARY: c_int = 0x2;
const QLOC_FULL: c_int = 0x4;
const QLOC_LANGUAGE: c_int = 0x100;
const QLOC_EXISTS: c_int = 0x200;

/// sdk/include/psdk/winnls.h
const LOCALE_RETURN_NUMBER: u32 = 0x2000_0000;
const LOCALE_ILANGUAGE: u32 = 1;
const LOCALE_SENGLANGUAGE: u32 = 0x1001;
const LOCALE_SENGLISHLANGUAGENAME: u32 = 0x1001;
const LOCALE_SABBREVLANGNAME: u32 = 3;
const LOCALE_SENGCOUNTRY: u32 = 0x1002;
const LOCALE_SENGLISHCOUNTRYNAME: u32 = 0x1002;
const LOCALE_SABBREVCTRYNAME: u32 = 7;
const LOCALE_IDEFAULTCODEPAGE: u32 = 11;
const LOCALE_IDEFAULTANSICODEPAGE: u32 = 0x1004;
const LOCALE_SISO639LANGNAME: u32 = 89;
const LCID_INSTALLED: u32 = 1;
const LOCALE_WINDOWS: u32 = 0x01;
const LOCALE_SUPPLEMENTAL: u32 = 0x02;
const CP_UTF7: c_int = 65000;
const CP_UTF8: c_int = 65001;

/// sdk/include/xdk/ntbasedef.h
const LANG_AFRIKAANS: u16 = 0x36;
const LANG_BASQUE: u16 = 0x2d;
const LANG_CATALAN: u16 = 0x03;
const LANG_FRENCH: u16 = 0x0c;
const LANG_GERMAN: u16 = 0x07;
const LANG_ITALIAN: u16 = 0x10;
const LANG_SERBIAN: u16 = 0x1a;
const LANG_SWEDISH: u16 = 0x1d;
const SUBLANG_DEFAULT: u16 = 0x01;
const SUBLANG_FRENCH_BELGIAN: u16 = 0x02;
const SUBLANG_FRENCH_CANADIAN: u16 = 0x03;
const SUBLANG_FRENCH_SWISS: u16 = 0x04;
const SUBLANG_GERMAN_LUXEMBOURG: u16 = 0x04;
const SUBLANG_ITALIAN_SWISS: u16 = 0x02;
const SUBLANG_SERBIAN_CYRILLIC: u16 = 0x03;
const SUBLANG_SWEDISH_FINLAND: u16 = 0x02;
const SORT_DEFAULT: u32 = 0x0;

const fn makelangid(primary: u16, sub: u16) -> u16 {
    (sub << 10) | primary
}

/// `LPARAM` (`LONG_PTR`), spelled per width rather than as `isize`.
#[cfg(target_pointer_width = "32")]
type LParam = c_long;
#[cfg(target_pointer_width = "64")]
type LParam = i64;

/// `LOCALE_ENUMPROCEX` / `LOCALE_ENUMPROCW`.
type LocaleEnumProcEx = unsafe extern "system" fn(*mut u16, u32, LParam) -> c_int;
type LocaleEnumProcW = unsafe extern "system" fn(*mut u16) -> c_int;

extern "system" {
    fn GetACP() -> c_uint;
    fn IsValidCodePage(code_page: c_uint) -> c_int;
    fn IsValidLocale(locale: u32, flags: u32) -> c_int;
    fn GetLocaleInfoW(locale: u32, lc_type: u32, data: *mut u16, cch_data: c_int) -> c_int;
    fn EnumSystemLocalesW(enum_proc: LocaleEnumProcW, flags: u32) -> c_int;
    fn GetUserDefaultLCID() -> u32;

    // corecrt_internal.h:1210-1310 -- WINAPI thunks over the Vista+ NLS API.
    fn __acrt_EnumSystemLocalesEx(
        enum_proc: LocaleEnumProcEx,
        flags: u32,
        param: LParam,
        reserved: *mut c_void,
    ) -> c_int;
    fn __acrt_GetLocaleInfoEx(locale_name: *const u16, lc_type: u32, data: *mut u16, data_count: c_int) -> c_int;
    fn __acrt_GetUserDefaultLocaleName(locale_name: *mut u16, locale_name_count: c_int) -> c_int;
    fn __acrt_LCIDToLocaleName(locale: u32, name: *mut u16, name_count: c_int, flags: u32) -> c_int;
}

extern "C" {
    fn _wcsicmp(a: *const u16, b: *const u16) -> c_int;
    fn _wcsnicmp(a: *const u16, b: *const u16, count: usize) -> c_int;
    fn wcscmp(a: *const u16, b: *const u16) -> c_int;
    fn wcsncmp(a: *const u16, b: *const u16, count: usize) -> c_int;
    fn wcslen(s: *const u16) -> usize;
    fn wcschr(s: *const u16, c: u16) -> *mut u16;
    fn wcsncpy_s(dst: *mut u16, size_in_words: usize, src: *const u16, count: usize) -> c_int;
    fn _itow_s(value: c_int, buffer: *mut u16, size_in_words: usize, radix: c_int) -> c_int;
    fn _wtol(s: *const u16) -> c_long;
    fn _invoke_watson(e: *const u16, f: *const u16, file: *const u16, line: c_uint, r: usize) -> !;
}

/// `_ERRCHECK(e)` in a release build: Watson on any nonzero errno_t.
unsafe fn errcheck(e: c_int) {
    if e != 0 {
        _invoke_watson(null(), null(), null(), 0, 0);
    }
}

/// An ASCII literal as a NUL-terminated UTF-16 array.
macro_rules! wide {
    ($s:literal) => {{
        const S: &[u8] = $s.as_bytes();
        const W: [u16; S.len() + 1] = {
            let mut w = [0u16; S.len() + 1];
            let mut i = 0;
            while i < S.len() {
                w[i] = S[i] as u16;
                i += 1;
            }
            w
        };
        W
    }};
}

// ── the name tables ──────────────────────────────────────────────────────────

/// A `const` table of `__crt_locale_string_table` rows. Transparent, so the
/// symbol is laid out exactly as the C array; the wrapper only exists so a
/// table of raw pointers can be a (read-only) `static`.
#[repr(transparent)]
pub struct StringTable<const N: usize>(pub [LocaleStringTable; N]);
unsafe impl<const N: usize> Sync for StringTable<N> {}

macro_rules! row {
    ($name:literal, $abbrev:literal) => {
        LocaleStringTable {
            name: &wide!($name) as *const [u16; $name.len() + 1] as *const u16,
            abbrev: wide!($abbrev),
        }
    };
}

/// Non-NLS language names -> three-letter Windows abbreviations. SORTED for
/// `TranslateName`'s case-insensitive binary search.
#[no_mangle]
pub static __acrt_rg_language: StringTable<65> = StringTable([
    row!("american", "ENU"),
    row!("american english", "ENU"),
    row!("american-english", "ENU"),
    row!("australian", "ENA"),
    row!("belgian", "NLB"),
    row!("canadian", "ENC"),
    row!("chh", "ZHH"),
    row!("chi", "ZHI"),
    row!("chinese", "CHS"),
    row!("chinese-hongkong", "ZHH"),
    row!("chinese-simplified", "CHS"),
    row!("chinese-singapore", "ZHI"),
    row!("chinese-traditional", "CHT"),
    row!("dutch-belgian", "NLB"),
    row!("english-american", "ENU"),
    row!("english-aus", "ENA"),
    row!("english-belize", "ENL"),
    row!("english-can", "ENC"),
    row!("english-caribbean", "ENB"),
    row!("english-ire", "ENI"),
    row!("english-jamaica", "ENJ"),
    row!("english-nz", "ENZ"),
    row!("english-south africa", "ENS"),
    row!("english-trinidad y tobago", "ENT"),
    row!("english-uk", "ENG"),
    row!("english-us", "ENU"),
    row!("english-usa", "ENU"),
    row!("french-belgian", "FRB"),
    row!("french-canadian", "FRC"),
    row!("french-luxembourg", "FRL"),
    row!("french-swiss", "FRS"),
    row!("german-austrian", "DEA"),
    row!("german-lichtenstein", "DEC"),
    row!("german-luxembourg", "DEL"),
    row!("german-swiss", "DES"),
    row!("irish-english", "ENI"),
    row!("italian-swiss", "ITS"),
    row!("norwegian", "NOR"),
    row!("norwegian-bokmal", "NOR"),
    row!("norwegian-nynorsk", "NON"),
    row!("portuguese-brazilian", "PTB"),
    row!("spanish-argentina", "ESS"),
    row!("spanish-bolivia", "ESB"),
    row!("spanish-chile", "ESL"),
    row!("spanish-colombia", "ESO"),
    row!("spanish-costa rica", "ESC"),
    row!("spanish-dominican republic", "ESD"),
    row!("spanish-ecuador", "ESF"),
    row!("spanish-el salvador", "ESE"),
    row!("spanish-guatemala", "ESG"),
    row!("spanish-honduras", "ESH"),
    row!("spanish-mexican", "ESM"),
    row!("spanish-modern", "ESN"),
    row!("spanish-nicaragua", "ESI"),
    row!("spanish-panama", "ESA"),
    row!("spanish-paraguay", "ESZ"),
    row!("spanish-peru", "ESR"),
    row!("spanish-puerto rico", "ESU"),
    row!("spanish-uruguay", "ESY"),
    row!("spanish-venezuela", "ESV"),
    row!("swedish-finland", "SVF"),
    row!("swiss", "DES"),
    row!("uk", "ENG"),
    row!("us", "ENU"),
    row!("usa", "ENU"),
]);

/// Non-NLS country/region names -> three-letter abbreviations. SORTED.
#[no_mangle]
pub static __acrt_rg_country: StringTable<23> = StringTable([
    row!("america", "USA"),
    row!("britain", "GBR"),
    row!("china", "CHN"),
    row!("czech", "CZE"),
    row!("england", "GBR"),
    row!("great britain", "GBR"),
    row!("holland", "NLD"),
    row!("hong-kong", "HKG"),
    row!("new-zealand", "NZL"),
    row!("nz", "NZL"),
    row!("pr china", "CHN"),
    row!("pr-china", "CHN"),
    row!("puerto-rico", "PRI"),
    row!("slovak", "SVK"),
    row!("south africa", "ZAF"),
    row!("south korea", "KOR"),
    row!("south-africa", "ZAF"),
    row!("south-korea", "KOR"),
    row!("trinidad & tobago", "TTO"),
    row!("uk", "GBR"),
    row!("united-kingdom", "GBR"),
    row!("united-states", "USA"),
    row!("us", "USA"),
]);

#[no_mangle]
pub static __acrt_rg_language_count: usize = 65;
#[no_mangle]
pub static __acrt_rg_country_count: usize = 23;

/// LANGIDs of locales whose sublanguage is NOT the default for their language.
#[no_mangle]
pub static __rglangidNotDefault: [u16; 10] = [
    makelangid(LANG_FRENCH, SUBLANG_FRENCH_CANADIAN),
    makelangid(LANG_SERBIAN, SUBLANG_SERBIAN_CYRILLIC),
    makelangid(LANG_GERMAN, SUBLANG_GERMAN_LUXEMBOURG),
    makelangid(LANG_AFRIKAANS, SUBLANG_DEFAULT),
    makelangid(LANG_FRENCH, SUBLANG_FRENCH_BELGIAN),
    makelangid(LANG_BASQUE, SUBLANG_DEFAULT),
    makelangid(LANG_CATALAN, SUBLANG_DEFAULT),
    makelangid(LANG_FRENCH, SUBLANG_FRENCH_SWISS),
    makelangid(LANG_ITALIAN, SUBLANG_ITALIAN_SWISS),
    makelangid(LANG_SWEDISH, SUBLANG_SWEDISH_FINLAND),
];

const W_ACP: [u16; 4] = wide!("ACP");
const W_OCP: [u16; 4] = wide!("OCP");
const W_UTF8: [u16; 5] = wide!("utf8");
const W_UTF_8: [u16; 6] = wide!("utf-8");

// ── shared helpers ──────────────────────────────────────────────────────────

/// The calling thread's `_setloc_data`.
unsafe fn setloc_data() -> *mut QualifiedLocaleData {
    addr_of_mut!((*__acrt_getptd()).setloc_data)
}

/// `TranslateName`: binary-search `table[0..=high]` case-insensitively and, on
/// a hit, repoint `*ppch_name` at the row's abbreviation. The two C files
/// spell the loop differently (a `cmp != 0` loop guard vs. an early return)
/// but they are the same search with the same result.
unsafe fn translate_name(table: *const LocaleStringTable, high: c_int, ppch_name: *mut *const u16) -> bool {
    let mut low: c_int = 0;
    let mut high = high;
    while low <= high {
        let i = (low + high) / 2;
        let cmp = _wcsicmp(*ppch_name, (*table.add(i as usize)).name);
        if cmp == 0 {
            *ppch_name = (*table.add(i as usize)).abbrev.as_ptr();
            return true;
        } else if cmp < 0 {
            high = i - 1;
        } else {
            low = i + 1;
        }
    }
    false
}

unsafe fn translate_country(ppch: *mut *const u16) -> bool {
    translate_name(__acrt_rg_country.0.as_ptr(), (__acrt_rg_country_count - 1) as c_int, ppch)
}

unsafe fn translate_language(ppch: *mut *const u16) -> bool {
    translate_name(__acrt_rg_language.0.as_ptr(), (__acrt_rg_language_count - 1) as c_int, ppch)
}

/// `GetPrimaryLen` (get_qualified_locale.cpp): the leading ASCII-letter run.
/// This copy tolerates NULL; the downlevel one does not.
unsafe fn get_primary_len(pch_language: *const u16) -> c_int {
    if pch_language.is_null() {
        return 0;
    }
    get_primary_len_downlevel(pch_language)
}

/// `GetPrimaryLen` (getqloc_downlevel.cpp).
unsafe fn get_primary_len_downlevel(mut pch_language: *const u16) -> c_int {
    let mut len = 0;
    let mut ch = *pch_language;
    pch_language = pch_language.add(1);
    while (ch >= b'A' as u16 && ch <= b'Z' as u16) || (ch >= b'a' as u16 && ch <= b'z' as u16) {
        len += 1;
        ch = *pch_language;
        pch_language = pch_language.add(1);
    }
    len
}

/// `wcsncpy_s(d->_cacheLocaleName, 85, src, wcslen(src) + 1)` under `_ERRCHECK`.
unsafe fn set_cache_locale_name(d: *mut QualifiedLocaleData, src: *const u16) {
    errcheck(wcsncpy_s(
        addr_of_mut!((*d).cache_locale_name) as *mut u16,
        LOCALE_NAME_MAX_LENGTH,
        src,
        wcslen(src) + 1,
    ));
}

// ── __acrt_get_qualified_locale (get_qualified_locale.cpp) ──────────────────

/// `__acrt_get_qualified_locale`: TRUE if `lpInStr` names an installed locale
/// with a valid codepage. `lpOutCodePage`/`lpOutStr` may be NULL, and
/// `lpOutStr` may alias `lpInStr`.
#[no_mangle]
pub unsafe extern "C" fn __acrt_get_qualified_locale(
    lp_in_str: *const LocaleStrings,
    lp_out_code_page: *mut c_uint,
    lp_out_str: *mut LocaleStrings,
) -> c_int {
    let d = setloc_data();
    (*d).cache_locale_name[0] = 0; // the invariant locale name

    (*d).i_loc_state = 0;
    (*d).pch_language = addr_of!((*lp_in_str).language) as *const u16;
    (*d).pch_country = addr_of!((*lp_in_str).country) as *const u16;

    // Legacy country names -> three-letter abbreviations, always first.
    if *(*d).pch_country != 0 {
        translate_country(addr_of_mut!((*d).pch_country));
    }

    if *(*d).pch_language != 0 {
        if *(*d).pch_country != 0 {
            get_locale_name_from_lang_country(d);
        } else {
            get_locale_name_from_language(d);
        }

        // Still nothing: retry with the legacy language name translated.
        if (*d).i_loc_state == 0 && translate_language(addr_of_mut!((*d).pch_language)) {
            if *(*d).pch_country != 0 {
                get_locale_name_from_lang_country(d);
            } else {
                get_locale_name_from_language(d);
            }
        }
    } else {
        // Empty language: the user default locale.
        get_locale_name_from_default(d);
    }

    if (*d).i_loc_state == 0 {
        return 0;
    }

    // `lp_in_str == NULL` is dead (dereferenced above); kept as written.
    let i_code_page: c_int = if lp_in_str.is_null()
        || (*lp_in_str).language[0] != 0
        || (*lp_in_str).code_page[0] != 0
    {
        process_code_page(
            if lp_in_str.is_null() { null() } else { (*lp_in_str).code_page.as_ptr() },
            d,
        )
    } else {
        // No language and no codepage: the user's settings, i.e. the ACP.
        GetACP() as c_int
    };

    if i_code_page == 0 || i_code_page == CP_UTF7 || IsValidCodePage(i_code_page as u16 as c_uint) == 0 {
        return 0;
    }

    if !lp_out_code_page.is_null() {
        *lp_out_code_page = i_code_page as c_uint;
    }

    if !lp_out_str.is_null() {
        let out_name = addr_of_mut!((*lp_out_str).locale_name) as *mut u16;
        let out_lang = addr_of_mut!((*lp_out_str).language) as *mut u16;
        let out_ctry = addr_of_mut!((*lp_out_str).country) as *mut u16;
        let out_cp = addr_of_mut!((*lp_out_str).code_page) as *mut u16;
        let cache = addr_of_mut!((*d).cache_locale_name) as *const u16;

        *out_name = 0;
        errcheck(wcsncpy_s(out_name, LOCALE_NAME_MAX_LENGTH, cache, wcslen(cache) + 1));

        if __acrt_GetLocaleInfoEx(out_name, LOCALE_SENGLISHLANGUAGENAME, out_lang, MAX_LANG_LEN as c_int) == 0 {
            return 0;
        }
        if __acrt_GetLocaleInfoEx(out_name, LOCALE_SENGLISHCOUNTRYNAME, out_ctry, MAX_CTRY_LEN as c_int) == 0 {
            return 0;
        }

        // '_' and '.' are setlocale separators: use the abbreviation instead.
        if (!wcschr(out_ctry, b'_' as u16).is_null() || !wcschr(out_ctry, b'.' as u16).is_null())
            && __acrt_GetLocaleInfoEx(out_name, LOCALE_SABBREVCTRYNAME, out_ctry, MAX_CTRY_LEN as c_int) == 0
        {
            return 0;
        }

        if i_code_page == CP_UTF8 {
            errcheck(wcsncpy_s(out_cp, MAX_CP_LEN, W_UTF8.as_ptr(), 5));
        } else {
            // Return value ignored, as in the C.
            _itow_s(i_code_page, out_cp, MAX_CP_LEN, 10);
        }
    }

    1
}

/// `GetLocaleNameFromLangCountry`.
unsafe fn get_locale_name_from_lang_country(d: *mut QualifiedLocaleData) {
    (*d).b_abbrev_language = (wcslen((*d).pch_language) == 3) as c_int;
    (*d).b_abbrev_country = (wcslen((*d).pch_country) == 3) as c_int;

    (*d).i_primary_len = if (*d).b_abbrev_language != 0 { 2 } else { get_primary_len((*d).pch_language) };

    // OS locales including replacements, excluding alternate sorts.
    __acrt_EnumSystemLocalesEx(lang_country_enum_proc_ex, LOCALE_WINDOWS | LOCALE_SUPPLEMENTAL, 0, null_mut());

    let s = (*d).i_loc_state;
    if s & QLOC_LANGUAGE == 0 || s & QLOC_EXISTS == 0 || s & (QLOC_FULL | QLOC_PRIMARY | QLOC_DEFAULT) == 0 {
        (*d).i_loc_state = 0;
    }
}

/// `LangCountryEnumProcEx`: FALSE (stop) once a full match is found.
unsafe extern "system" fn lang_country_enum_proc_ex(lp_locale_string: *mut u16, _flags: u32, _lparam: LParam) -> c_int {
    let d = setloc_data();
    let mut rgc_info = [0u16; MAX_LANG_LEN]; // == MAX_CTRY_LEN
    let info = rgc_info.as_mut_ptr();
    let n = rgc_info.len() as c_int;

    // Country first.
    let ctry_type = if (*d).b_abbrev_country != 0 { LOCALE_SABBREVCTRYNAME } else { LOCALE_SENGLISHCOUNTRYNAME };
    if __acrt_GetLocaleInfoEx(lp_locale_string, ctry_type, info, n) == 0 {
        (*d).i_loc_state = 0;
        return 1;
    }

    let lang_type = if (*d).b_abbrev_language != 0 { LOCALE_SABBREVLANGNAME } else { LOCALE_SENGLISHLANGUAGENAME };

    if _wcsicmp((*d).pch_country, info) == 0 {
        if __acrt_GetLocaleInfoEx(lp_locale_string, lang_type, info, n) == 0 {
            (*d).i_loc_state = 0;
            return 1;
        }

        if _wcsicmp((*d).pch_language, info) == 0 {
            // Language and country: the best match.
            (*d).i_loc_state |= QLOC_FULL | QLOC_LANGUAGE | QLOC_EXISTS;
            set_cache_locale_name(d, lp_locale_string);
        } else if (*d).i_loc_state & QLOC_PRIMARY == 0 {
            if (*d).i_primary_len != 0
                && _wcsnicmp((*d).pch_language, info, (*d).i_primary_len as usize) == 0
            {
                // Primary language and country.
                (*d).i_loc_state |= QLOC_PRIMARY;
                set_cache_locale_name(d, lp_locale_string);
            } else if (*d).i_loc_state & QLOC_DEFAULT == 0 && test_default_country(lp_locale_string) {
                // The country's default language.
                (*d).i_loc_state |= QLOC_DEFAULT;
                set_cache_locale_name(d, lp_locale_string);
            }
        }
    }

    // Is the language installed at all, and is its default seen?
    if (*d).i_loc_state & (QLOC_LANGUAGE | QLOC_EXISTS) != (QLOC_LANGUAGE | QLOC_EXISTS) {
        if __acrt_GetLocaleInfoEx(lp_locale_string, lang_type, info, n) == 0 {
            (*d).i_loc_state = 0;
            return 1;
        }

        if _wcsicmp((*d).pch_language, info) == 0 {
            (*d).i_loc_state |= QLOC_EXISTS;

            if (*d).b_abbrev_language != 0 {
                (*d).i_loc_state |= QLOC_LANGUAGE;
                if (*d).cache_locale_name[0] == 0 {
                    set_cache_locale_name(d, lp_locale_string);
                }
            } else if (*d).i_primary_len != 0 && wcslen((*d).pch_language) as c_int == (*d).i_primary_len {
                // Primary language only: must be the default sublanguage.
                if test_default_language(lp_locale_string, true, d) {
                    (*d).i_loc_state |= QLOC_LANGUAGE;
                    if (*d).cache_locale_name[0] == 0 {
                        set_cache_locale_name(d, lp_locale_string);
                    }
                }
            } else {
                // Language with sublanguage.
                (*d).i_loc_state |= QLOC_LANGUAGE;
                if (*d).cache_locale_name[0] == 0 {
                    set_cache_locale_name(d, lp_locale_string);
                }
            }
        }
    }

    ((*d).i_loc_state & QLOC_FULL == 0) as c_int
}

/// `GetLocaleNameFromLanguage`.
unsafe fn get_locale_name_from_language(d: *mut QualifiedLocaleData) {
    (*d).b_abbrev_language = (wcslen((*d).pch_language) == 3) as c_int;
    (*d).i_primary_len = if (*d).b_abbrev_language != 0 { 2 } else { get_primary_len((*d).pch_language) };

    __acrt_EnumSystemLocalesEx(language_enum_proc_ex, LOCALE_WINDOWS | LOCALE_SUPPLEMENTAL, 0, null_mut());

    if (*d).i_loc_state & QLOC_FULL == 0 {
        (*d).i_loc_state = 0;
    }
}

/// `LanguageEnumProcEx`: the FIRST enumerated locale whose language name
/// matches wins (no default-country test in this form).
unsafe extern "system" fn language_enum_proc_ex(lp_locale_string: *mut u16, _flags: u32, _lparam: LParam) -> c_int {
    let d = setloc_data();
    let mut rgc_info = [0u16; 120];

    let lang_type = if (*d).b_abbrev_language != 0 { LOCALE_SABBREVLANGNAME } else { LOCALE_SENGLISHLANGUAGENAME };
    if __acrt_GetLocaleInfoEx(lp_locale_string, lang_type, rgc_info.as_mut_ptr(), rgc_info.len() as c_int) == 0 {
        (*d).i_loc_state = 0;
        return 1;
    }

    if _wcsicmp((*d).pch_language, rgc_info.as_ptr()) == 0 {
        set_cache_locale_name(d, lp_locale_string);
        (*d).i_loc_state |= QLOC_FULL;
    }

    ((*d).i_loc_state & QLOC_FULL == 0) as c_int
}

/// `GetLocaleNameFromDefault`: the user default locale name.
unsafe fn get_locale_name_from_default(d: *mut QualifiedLocaleData) {
    let mut locale_name = [0u16; LOCALE_NAME_MAX_LENGTH];
    (*d).i_loc_state |= QLOC_FULL | QLOC_LANGUAGE;

    // The size includes the NUL, so only a non-empty name (> 1) is stored.
    if __acrt_GetUserDefaultLocaleName(locale_name.as_mut_ptr(), LOCALE_NAME_MAX_LENGTH as c_int) > 1 {
        set_cache_locale_name(d, locale_name.as_ptr());
    }
}

/// `ProcessCodePage`: the codepage string as a number; 0 if the locale query
/// failed.
unsafe fn process_code_page(lp_code_page_str: *const u16, d: *mut QualifiedLocaleData) -> c_int {
    // Zeroed where the C leaves it uninitialised (only read after a successful
    // query writes it).
    let mut i_code_page: c_int = 0;
    let cache = addr_of_mut!((*d).cache_locale_name) as *const u16;
    let cch = (core::mem::size_of::<c_int>() / core::mem::size_of::<u16>()) as c_int;

    if lp_code_page_str.is_null() || *lp_code_page_str == 0 || wcscmp(lp_code_page_str, W_ACP.as_ptr()) == 0 {
        if __acrt_GetLocaleInfoEx(
            cache,
            LOCALE_IDEFAULTANSICODEPAGE | LOCALE_RETURN_NUMBER,
            &mut i_code_page as *mut c_int as *mut u16,
            cch,
        ) == 0
        {
            return 0;
        }
        // 0/1/2 are the "Unicode-only" CP_ACP/OEMCP/MACCP markers.
        if i_code_page < 3 {
            return CP_UTF8;
        }
    } else if _wcsicmp(lp_code_page_str, W_UTF8.as_ptr()) == 0 || _wcsicmp(lp_code_page_str, W_UTF_8.as_ptr()) == 0 {
        return CP_UTF8;
    } else if wcscmp(lp_code_page_str, W_OCP.as_ptr()) == 0 {
        if __acrt_GetLocaleInfoEx(
            cache,
            LOCALE_IDEFAULTCODEPAGE | LOCALE_RETURN_NUMBER,
            &mut i_code_page as *mut c_int as *mut u16,
            cch,
        ) == 0
        {
            return 0;
        }
        if i_code_page < 3 {
            return CP_UTF8;
        }
    } else {
        i_code_page = _wtol(lp_code_page_str) as c_int;
    }

    i_code_page
}

/// `TestDefaultCountry`: is this a neutral-language locale name, i.e. does it
/// begin with (at most 9 characters of) its own ISO 639 language name?
unsafe fn test_default_country(locale_name: *const u16) -> bool {
    let mut s_iso639_lang_name = [0u16; 9];

    if __acrt_GetLocaleInfoEx(
        locale_name,
        LOCALE_SISO639LANGNAME,
        s_iso639_lang_name.as_mut_ptr(),
        s_iso639_lang_name.len() as c_int,
    ) == 0
    {
        return false;
    }

    wcsncmp(s_iso639_lang_name.as_ptr(), locale_name, s_iso639_lang_name.len()) == 0
}

/// `TestDefaultLanguage`.
unsafe fn test_default_language(locale_name: *const u16, b_test_primary: bool, d: *mut QualifiedLocaleData) -> bool {
    if !test_default_country(locale_name)
        && b_test_primary
        && get_primary_len((*d).pch_language) == wcslen((*d).pch_language) as c_int
    {
        return false;
    }
    true
}

// ── __acrt_get_qualified_locale_downlevel (getqloc_downlevel.cpp) ───────────

/// The calling thread's `_setloc_downlevel_data`.
unsafe fn setloc_downlevel_data() -> *mut QualifiedLocaleDataDownlevel {
    (*__acrt_getptd()).setloc_downlevel_data
}

/// `__acrt_get_qualified_locale_downlevel`: the LCID-based form.
#[no_mangle]
pub unsafe extern "C" fn __acrt_get_qualified_locale_downlevel(
    lp_in_str: *const LocaleStrings,
    lp_out_code_page: *mut c_uint,
    lp_out_str: *mut LocaleStrings,
) -> c_int {
    let d = setloc_data();

    // A STACK local published through the ptd for the callbacks, and never
    // unpublished: the pointer dangles after return, as in the C.
    let mut downlevel_setloc = QualifiedLocaleDataDownlevel { i_lcid_state: 0, lcid_language: 0, lcid_country: 0 };
    let dl: *mut QualifiedLocaleDataDownlevel = &mut downlevel_setloc;
    (*__acrt_getptd()).setloc_downlevel_data = dl;

    (*d).pch_language = addr_of!((*lp_in_str).language) as *const u16;

    (*d).pch_country = addr_of!((*lp_in_str).country) as *const u16;
    if !(*d).pch_country.is_null() && *(*d).pch_country != 0 {
        translate_country(addr_of_mut!((*d).pch_country));
    }

    (*dl).i_lcid_state = 0;

    if !(*d).pch_language.is_null() && *(*d).pch_language != 0 {
        if !(*d).pch_country.is_null() && *(*d).pch_country != 0 {
            get_lcid_from_lang_country(dl);
        } else {
            get_lcid_from_language(dl);
        }

        if (*dl).i_lcid_state == 0 && translate_language(addr_of_mut!((*d).pch_language)) {
            if !(*d).pch_country.is_null() && *(*d).pch_country != 0 {
                get_lcid_from_lang_country(dl);
            } else {
                get_lcid_from_language(dl);
            }
        }
    } else if !(*d).pch_country.is_null() && *(*d).pch_country != 0 {
        get_lcid_from_country(dl);
    } else {
        get_lcid_from_default(dl);
    }

    if (*dl).i_lcid_state == 0 {
        return 0;
    }

    let i_code_page = process_code_page_downlevel(
        if lp_in_str.is_null() { null() } else { (*lp_in_str).code_page.as_ptr() },
        dl,
    );

    if i_code_page == 0 || IsValidCodePage(i_code_page as u16 as c_uint) == 0 {
        return 0;
    }

    if IsValidLocale((*dl).lcid_language, LCID_INSTALLED) == 0 {
        return 0;
    }

    if !lp_out_code_page.is_null() {
        *lp_out_code_page = i_code_page as c_uint;
    }

    __acrt_LCIDToLocaleName(
        (*dl).lcid_language,
        addr_of_mut!((*d).cache_locale_name) as *mut u16,
        LOCALE_NAME_MAX_LENGTH as c_int,
        0,
    );

    if !lp_out_str.is_null() {
        __acrt_LCIDToLocaleName(
            (*dl).lcid_language,
            addr_of_mut!((*lp_out_str).locale_name) as *mut u16,
            LOCALE_NAME_MAX_LENGTH as c_int,
            0,
        );

        if GetLocaleInfoW(
            (*dl).lcid_language,
            LOCALE_SENGLANGUAGE,
            addr_of_mut!((*lp_out_str).language) as *mut u16,
            MAX_LANG_LEN as c_int,
        ) == 0
        {
            return 0;
        }

        if GetLocaleInfoW(
            (*dl).lcid_country,
            LOCALE_SENGCOUNTRY,
            addr_of_mut!((*lp_out_str).country) as *mut u16,
            MAX_CTRY_LEN as c_int,
        ) == 0
        {
            return 0;
        }

        _itow_s(i_code_page, addr_of_mut!((*lp_out_str).code_page) as *mut u16, MAX_CP_LEN, 10);
    }

    1
}

/// `GetLcidFromLangCountry`.
unsafe fn get_lcid_from_lang_country(dl: *mut QualifiedLocaleDataDownlevel) {
    let d = setloc_data();

    (*d).b_abbrev_language = (wcslen((*d).pch_language) == 3) as c_int;
    (*d).b_abbrev_country = (wcslen((*d).pch_country) == 3) as c_int;
    (*dl).lcid_language = 0;
    (*d).i_primary_len = if (*d).b_abbrev_language != 0 { 2 } else { get_primary_len_downlevel((*d).pch_language) };

    EnumSystemLocalesW(lang_country_enum_proc, LCID_INSTALLED);

    let s = (*dl).i_lcid_state;
    if s & QLOC_LANGUAGE == 0 || s & QLOC_EXISTS == 0 || s & (QLOC_FULL | QLOC_PRIMARY | QLOC_DEFAULT) == 0 {
        (*dl).i_lcid_state = 0;
    }
}

/// `LangCountryEnumProc`.
unsafe extern "system" fn lang_country_enum_proc(lp_lcid_string: *mut u16) -> c_int {
    let d = setloc_data();
    let dl = setloc_downlevel_data();
    let lcid = lcid_from_hex_string(lp_lcid_string);
    let mut rgc_info = [0u16; 120];
    let info = rgc_info.as_mut_ptr();
    let n = rgc_info.len() as c_int;

    let ctry_type = if (*d).b_abbrev_country != 0 { LOCALE_SABBREVCTRYNAME } else { LOCALE_SENGCOUNTRY };
    if GetLocaleInfoW(lcid, ctry_type, info, n) == 0 {
        (*dl).i_lcid_state = 0;
        return 1;
    }

    let lang_type = if (*d).b_abbrev_language != 0 { LOCALE_SABBREVLANGNAME } else { LOCALE_SENGLANGUAGE };

    if _wcsicmp((*d).pch_country, info) == 0 {
        if GetLocaleInfoW(lcid, lang_type, info, n) == 0 {
            (*dl).i_lcid_state = 0;
            return 1;
        }
        if _wcsicmp((*d).pch_language, info) == 0 {
            (*dl).i_lcid_state |= QLOC_FULL | QLOC_LANGUAGE | QLOC_EXISTS;
            (*dl).lcid_country = lcid;
            (*dl).lcid_language = lcid;
        } else if (*dl).i_lcid_state & QLOC_PRIMARY == 0 {
            if (*d).i_primary_len != 0
                && _wcsnicmp((*d).pch_language, info, (*d).i_primary_len as usize) == 0
            {
                (*dl).i_lcid_state |= QLOC_PRIMARY;
                (*dl).lcid_country = lcid;

                // Primary-only language string: this is also the language LCID.
                if wcslen((*d).pch_language) as c_int == (*d).i_primary_len {
                    (*dl).lcid_language = lcid;
                }
            } else if (*dl).i_lcid_state & QLOC_DEFAULT == 0 && test_default_country_lcid(lcid) {
                (*dl).i_lcid_state |= QLOC_DEFAULT;
                (*dl).lcid_country = lcid;
            }
        }
    }

    if (*dl).i_lcid_state & (QLOC_LANGUAGE | QLOC_EXISTS) != (QLOC_LANGUAGE | QLOC_EXISTS) {
        if GetLocaleInfoW(lcid, lang_type, info, n) == 0 {
            (*dl).i_lcid_state = 0;
            return 1;
        }

        if _wcsicmp((*d).pch_language, info) == 0 {
            (*dl).i_lcid_state |= QLOC_EXISTS;

            if (*d).b_abbrev_language != 0 {
                (*dl).i_lcid_state |= QLOC_LANGUAGE;
                if (*dl).lcid_language == 0 {
                    (*dl).lcid_language = lcid;
                }
            } else if (*d).i_primary_len != 0 && wcslen((*d).pch_language) as c_int == (*d).i_primary_len {
                if test_default_language_lcid(lcid, true, dl) {
                    (*dl).i_lcid_state |= QLOC_LANGUAGE;
                    if (*dl).lcid_language == 0 {
                        (*dl).lcid_language = lcid;
                    }
                }
            } else {
                (*dl).i_lcid_state |= QLOC_LANGUAGE;
                if (*dl).lcid_language == 0 {
                    (*dl).lcid_language = lcid;
                }
            }
        } else if (*d).b_abbrev_language == 0
            && (*d).i_primary_len != 0
            && _wcsicmp((*d).pch_language, info) == 0
        {
            // DEAD: the same _wcsicmp just returned nonzero. Kept (with its
            // call) exactly as the C has it.
            if test_default_language_lcid(lcid, false, dl) {
                (*dl).i_lcid_state |= QLOC_LANGUAGE;
                if (*dl).lcid_language == 0 {
                    (*dl).lcid_language = lcid;
                }
            }
        }
    }

    ((*dl).i_lcid_state & QLOC_FULL == 0) as c_int
}

/// `GetLcidFromLanguage`.
unsafe fn get_lcid_from_language(dl: *mut QualifiedLocaleDataDownlevel) {
    let d = setloc_data();

    (*d).b_abbrev_language = (wcslen((*d).pch_language) == 3) as c_int;
    (*d).i_primary_len = if (*d).b_abbrev_language != 0 { 2 } else { get_primary_len_downlevel((*d).pch_language) };

    EnumSystemLocalesW(language_enum_proc, LCID_INSTALLED);

    if (*dl).i_lcid_state & QLOC_FULL == 0 {
        (*dl).i_lcid_state = 0;
    }
}

/// `LanguageEnumProc`.
unsafe extern "system" fn language_enum_proc(lp_lcid_string: *mut u16) -> c_int {
    let d = setloc_data();
    let dl = setloc_downlevel_data();
    let lcid = lcid_from_hex_string(lp_lcid_string);
    let mut rgc_info = [0u16; 120];
    let info = rgc_info.as_mut_ptr();

    let lang_type = if (*d).b_abbrev_language != 0 { LOCALE_SABBREVLANGNAME } else { LOCALE_SENGLANGUAGE };
    if GetLocaleInfoW(lcid, lang_type, info, rgc_info.len() as c_int) == 0 {
        (*dl).i_lcid_state = 0;
        return 1;
    }

    if _wcsicmp((*d).pch_language, info) == 0 {
        // Language matched: take it if abbreviated, or if its country is the
        // default (or implied by the language string).
        if (*d).b_abbrev_language != 0 || test_default_language_lcid(lcid, true, dl) {
            (*dl).lcid_country = lcid;
            (*dl).lcid_language = lcid;
            (*dl).i_lcid_state |= QLOC_FULL;
        }
    } else if (*d).b_abbrev_language == 0
        && (*d).i_primary_len != 0
        && _wcsicmp((*d).pch_language, info) == 0
    {
        // DEAD, as in LangCountryEnumProc.
        if test_default_language_lcid(lcid, false, dl) {
            (*dl).lcid_country = lcid;
            (*dl).lcid_language = lcid;
            (*dl).i_lcid_state |= QLOC_FULL;
        }
    }

    ((*dl).i_lcid_state & QLOC_FULL == 0) as c_int
}

/// `GetLcidFromCountry`.
unsafe fn get_lcid_from_country(dl: *mut QualifiedLocaleDataDownlevel) {
    let d = setloc_data();
    (*d).b_abbrev_country = (wcslen((*d).pch_country) == 3) as c_int;

    EnumSystemLocalesW(country_enum_proc, LCID_INSTALLED);

    if (*dl).i_lcid_state & QLOC_FULL == 0 {
        (*dl).i_lcid_state = 0;
    }
}

/// `CountryEnumProc`.
unsafe extern "system" fn country_enum_proc(lp_lcid_string: *mut u16) -> c_int {
    let d = setloc_data();
    let dl = setloc_downlevel_data();
    let lcid = lcid_from_hex_string(lp_lcid_string);
    let mut rgc_info = [0u16; 120];
    let info = rgc_info.as_mut_ptr();

    let ctry_type = if (*d).b_abbrev_country != 0 { LOCALE_SABBREVCTRYNAME } else { LOCALE_SENGCOUNTRY };
    if GetLocaleInfoW(lcid, ctry_type, info, rgc_info.len() as c_int) == 0 {
        (*dl).i_lcid_state = 0;
        return 1;
    }
    if _wcsicmp((*d).pch_country, info) == 0 && test_default_country_lcid(lcid) {
        (*dl).lcid_country = lcid;
        (*dl).lcid_language = lcid;
        (*dl).i_lcid_state |= QLOC_FULL;
    }
    ((*dl).i_lcid_state & QLOC_FULL == 0) as c_int
}

/// `GetLcidFromDefault`: the user default LCID for both.
unsafe fn get_lcid_from_default(dl: *mut QualifiedLocaleDataDownlevel) {
    (*dl).i_lcid_state |= QLOC_FULL | QLOC_LANGUAGE;
    let lcid = GetUserDefaultLCID();
    (*dl).lcid_country = lcid;
    (*dl).lcid_language = lcid;
}

/// `ProcessCodePage` (downlevel): like the other, except a zero ANSI codepage
/// is the process ACP, an OEM codepage is taken as-is, and there is no
/// "utf8" spelling.
unsafe fn process_code_page_downlevel(lp_code_page_str: *const u16, dl: *mut QualifiedLocaleDataDownlevel) -> c_int {
    let mut i_code_page: c_int = 0;
    let cch = (core::mem::size_of::<c_int>() / core::mem::size_of::<u16>()) as c_int;

    if lp_code_page_str.is_null() || *lp_code_page_str == 0 || wcscmp(lp_code_page_str, W_ACP.as_ptr()) == 0 {
        if GetLocaleInfoW(
            (*dl).lcid_country,
            LOCALE_IDEFAULTANSICODEPAGE | LOCALE_RETURN_NUMBER,
            &mut i_code_page as *mut c_int as *mut u16,
            cch,
        ) == 0
        {
            return 0;
        }
        // Locales with no ANSI codepage (e.g. Hindi).
        if i_code_page == 0 {
            return GetACP() as c_int;
        }
    } else if wcscmp(lp_code_page_str, W_OCP.as_ptr()) == 0 {
        if GetLocaleInfoW(
            (*dl).lcid_country,
            LOCALE_IDEFAULTCODEPAGE | LOCALE_RETURN_NUMBER,
            &mut i_code_page as *mut c_int as *mut u16,
            cch,
        ) == 0
        {
            return 0;
        }
    } else {
        i_code_page = _wtol(lp_code_page_str) as c_int;
    }

    i_code_page
}

/// `TestDefaultCountry` (downlevel): TRUE unless the LANGID is listed in
/// `__rglangidNotDefault`.
fn test_default_country_lcid(lcid: u32) -> bool {
    let langid = lcid as u16; // LANGIDFROMLCID
    let mut i = 0usize;
    while i < __rglangidNotDefault.len() {
        if langid == __rglangidNotDefault[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// `TestDefaultLanguage` (downlevel). `_dl` is unreferenced, as in the C.
unsafe fn test_default_language_lcid(lcid: u32, b_test_primary: bool, _dl: *mut QualifiedLocaleDataDownlevel) -> bool {
    // MAKELCID(MAKELANGID(PRIMARYLANGID(LANGIDFROMLCID(lcid)), SUBLANG_DEFAULT), SORT_DEFAULT)
    let lcid_default: u32 = ((SORT_DEFAULT) << 16) | makelangid((lcid as u16) & 0x3ff, SUBLANG_DEFAULT) as u32;
    let d = setloc_data();
    let mut dw_language: u32 = 0;

    if GetLocaleInfoW(
        lcid_default,
        LOCALE_ILANGUAGE | LOCALE_RETURN_NUMBER,
        &mut dw_language as *mut u32 as *mut u16,
        (core::mem::size_of::<u32>() / core::mem::size_of::<u16>()) as c_int,
    ) == 0
    {
        return false;
    }

    if lcid != dw_language
        && b_test_primary
        && get_primary_len_downlevel((*d).pch_language) == wcslen((*d).pch_language) as c_int
    {
        return false;
    }
    true
}

/// `LcidFromHexString`: no validation -- any character that is not a hex
/// letter is taken as `ch - '0'`, all in wrapping 16/32-bit arithmetic.
unsafe fn lcid_from_hex_string(mut lp_hex_string: *const u16) -> u32 {
    let mut lcid: u32 = 0;
    loop {
        let mut ch = *lp_hex_string;
        lp_hex_string = lp_hex_string.add(1);
        if ch == 0 {
            break;
        }
        if ch >= b'a' as u16 && ch <= b'f' as u16 {
            ch = ch.wrapping_add((b'9' as u16 + 1).wrapping_sub(b'a' as u16));
        } else if ch >= b'A' as u16 && ch <= b'F' as u16 {
            ch = ch.wrapping_add((b'9' as u16 + 1).wrapping_sub(b'A' as u16));
        }
        lcid = lcid.wrapping_mul(0x10).wrapping_add(ch as u32).wrapping_sub(b'0' as u32);
    }
    lcid
}
