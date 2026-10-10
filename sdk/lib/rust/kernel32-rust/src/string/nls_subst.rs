//! nls_subst.rs — Win32 NLS substrate for the kernel32 string family.
//!
//! winnls.h types (GEOID/LCTYPE/CPINFO/…), the LOCALE_*/LCMAP_*/NORM_*/CP_*/C3_*
//! constant families, and the Wine `unicode.h` helper inlines (strcmpW/tolowerW/…)
//! that the rustconv-translated locale.rs references but ntdll-sys (NT-level) does
//! not carry. Constant values mirror sdk/include/psdk/winnls.h + winnt.h exactly;
//! the IDN/punycode constants + the dropped `parse_locale_name` helper come from
//! dll/win32/kernel32/winnls/string/locale.c.
//!
//! NOTE: symbols already declared *inside* locale.rs's own `extern "C"` block
//! (wine_cp_get_table, strlenW, tolowerW, memcpy, debugstr_w/_a, MAKELCID,
//! MAKELANGID, SORTIDFROMLCID, SUBLANGID, _snwprintf, SetLastError, GetLastError,
//! TRACE, FIXME, …) are intentionally NOT re-declared here — locale.rs's local
//! item shadows any glob import, and re-declaring would only invite drift.

#![allow(non_camel_case_types, non_upper_case_globals, dead_code, unused_imports)]

use ntdll_sys::nt::*;
use core::ffi::c_void;

// ===========================================================================
// Scalar typedef aliases  (winnls.h:  typedef DWORD LCTYPE; etc.)
// ===========================================================================
pub type LCTYPE = DWORD;
pub type LGRPID = DWORD;
pub type GEOID = DWORD;
pub type GEOTYPE = DWORD;
pub type GEOCLASS = DWORD;
// winnls.h:  typedef enum _NORM_FORM { ... } NORM_FORM;  (C enum => int)
pub type NORM_FORM = i32;

// ===========================================================================
// ENUMRESLANGPROC ABI bridges. The enum-lang callbacks are `extern "system"`
// (correct WINAPI callback ABI on every arch: stdcall on i386, C elsewhere).
// ntdll-sys's bindgen ENUMRESLANGPROCA/W use the *resolved* literal ABI per arch
// (stdcall on i386, C on amd64/arm64); rustc refuses to coerce a `system` fn-item
// to those literal-ABI fn-pointer types even though the ABI is identical, so bridge
// with a transmute (sound: same calling convention + fn-pointer layout). Used at
// the EnumResourceLanguagesA/W call sites.
// ===========================================================================
#[inline]
pub unsafe fn as_enumreslang_w(
    f: unsafe extern "system" fn(HMODULE, LPCWSTR, LPCWSTR, WORD, LONG_PTR) -> BOOL,
) -> ENUMRESLANGPROCW {
    core::mem::transmute(f)
}
#[inline]
pub unsafe fn as_enumreslang_a(
    f: unsafe extern "system" fn(HMODULE, LPCSTR, LPCSTR, WORD, LONG_PTR) -> BOOL,
) -> ENUMRESLANGPROCA {
    core::mem::transmute(f)
}

// ===========================================================================
// Wine codepage table  (wine/unicode.h: union cptable { cp_info info; sbcs_table
// sbcs; dbcs_table dbcs; }). All three arms share the leading `info` header, so a
// repr(C) union matches the C layout exactly. locale.rs accesses .info / .sbcs /
// .dbcs arms directly (the c2rust `__bindgen_anon_1.` indirection was rewritten).
// ===========================================================================
#[repr(C)]
#[derive(Clone, Copy)]
pub struct cp_info {
    pub codepage: UINT,
    pub char_size: UINT,
    pub def_char: WCHAR,
    pub def_unicode_char: WCHAR,
    pub name: *const i8,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct sbcs_table {
    pub info: cp_info,
    pub cp2uni: *const WCHAR,
    pub cp2uni_glyphs: *const WCHAR,
    pub uni2cp_low: *const u8,
    pub uni2cp_high: *const u16,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct dbcs_table {
    pub info: cp_info,
    pub cp2uni: *const WCHAR,
    pub cp2uni_leadbytes: *const u8,
    pub uni2cp_low: *const u16,
    pub uni2cp_high: *const u16,
    pub lead_bytes: [u8; 12],
}
#[repr(C)]
pub union cptable {
    pub info: cp_info,
    pub sbcs: sbcs_table,
    pub dbcs: dbcs_table,
}

// ===========================================================================
// Constants — winnt.h C1_/C2_/C3_/CT_CTYPE* character-type flags
// ===========================================================================
pub const CT_CTYPE1: DWORD = 1;
pub const CT_CTYPE2: DWORD = 2;
pub const CT_CTYPE3: DWORD = 4;

pub const C1_UPPER: WORD = 1;
pub const C1_LOWER: WORD = 2;
pub const C1_DIGIT: WORD = 4;
pub const C1_SPACE: WORD = 8;
pub const C1_PUNCT: WORD = 16;
pub const C1_CNTRL: WORD = 32;
pub const C1_BLANK: WORD = 64;
pub const C1_XDIGIT: WORD = 128;
pub const C1_ALPHA: WORD = 256;
pub const C1_DEFINED: WORD = 512;

pub const C2_LEFTTORIGHT: WORD = 1;
pub const C2_RIGHTTOLEFT: WORD = 2;
pub const C2_EUROPENUMBER: WORD = 3;
pub const C2_EUROPESEPARATOR: WORD = 4;
pub const C2_EUROPETERMINATOR: WORD = 5;
pub const C2_ARABICNUMBER: WORD = 6;
pub const C2_COMMONSEPARATOR: WORD = 7;
pub const C2_BLOCKSEPARATOR: WORD = 8;
pub const C2_SEGMENTSEPARATOR: WORD = 9;
pub const C2_WHITESPACE: WORD = 10;
pub const C2_OTHERNEUTRAL: WORD = 11;
pub const C2_NOTAPPLICABLE: WORD = 0;

pub const C3_NONSPACING: WORD = 1;
pub const C3_DIACRITIC: WORD = 2;
pub const C3_VOWELMARK: WORD = 4;
pub const C3_SYMBOL: WORD = 8;
pub const C3_KATAKANA: WORD = 16;
pub const C3_HIRAGANA: WORD = 32;
pub const C3_HALFWIDTH: WORD = 64;
pub const C3_FULLWIDTH: WORD = 128;
pub const C3_IDEOGRAPH: WORD = 256;
pub const C3_KASHIDA: WORD = 512;
pub const C3_LEXICAL: WORD = 1024;
pub const C3_ALPHA: WORD = 32768;
pub const C3_NOTAPPLICABLE: WORD = 0;
pub const C3_HIGHSURROGATE: WORD = 0x0800;
pub const C3_LOWSURROGATE: WORD = 0x1000;

// ===========================================================================
// Constants — winnls.h LOCALE_* (LCTYPE values, all DWORD)
// ===========================================================================
pub const LOCALE_NOUSEROVERRIDE: DWORD = 0x80000000;
pub const LOCALE_USE_CP_ACP: DWORD = 0x40000000;
pub const LOCALE_RETURN_NUMBER: DWORD = 0x20000000;
pub const LOCALE_RETURN_GENITIVE_NAMES: DWORD = 0x10000000;

pub const LOCALE_ILANGUAGE: DWORD = 0x00000001;
pub const LOCALE_SABBREVLANGNAME: DWORD = 3;
pub const LOCALE_ICOUNTRY: DWORD = 5;
pub const LOCALE_SCOUNTRY: DWORD = 6;
pub const LOCALE_IDEFAULTLANGUAGE: DWORD = 9;
pub const LOCALE_IDEFAULTCODEPAGE: DWORD = 11;
pub const LOCALE_IDEFAULTANSICODEPAGE: DWORD = 0x1004;
pub const LOCALE_IDEFAULTMACCODEPAGE: DWORD = 0x1011;
pub const LOCALE_SLIST: DWORD = 12;
pub const LOCALE_IMEASURE: DWORD = 13;
pub const LOCALE_SDECIMAL: DWORD = 14;
pub const LOCALE_STHOUSAND: DWORD = 15;
pub const LOCALE_SGROUPING: DWORD = 16;
pub const LOCALE_IDIGITS: DWORD = 17;
pub const LOCALE_ILZERO: DWORD = 18;
pub const LOCALE_INEGNUMBER: DWORD = 0x1010;
pub const LOCALE_SNATIVEDIGITS: DWORD = 19;
pub const LOCALE_SCURRENCY: DWORD = 20;
pub const LOCALE_SMONDECIMALSEP: DWORD = 22;
pub const LOCALE_SMONTHOUSANDSEP: DWORD = 23;
pub const LOCALE_SMONGROUPING: DWORD = 24;
pub const LOCALE_ICURRDIGITS: DWORD = 25;
pub const LOCALE_ICURRENCY: DWORD = 27;
pub const LOCALE_INEGCURR: DWORD = 28;
pub const LOCALE_SDATE: DWORD = 29;
pub const LOCALE_STIME: DWORD = 30;
pub const LOCALE_SSHORTDATE: DWORD = 31;
pub const LOCALE_SLONGDATE: DWORD = 32;
pub const LOCALE_STIMEFORMAT: DWORD = 0x1003;
pub const LOCALE_IDATE: DWORD = 33;
pub const LOCALE_ILDATE: DWORD = 34;
pub const LOCALE_ITIME: DWORD = 35;
pub const LOCALE_ITIMEMARKPOSN: DWORD = 0x1005;
pub const LOCALE_ITLZERO: DWORD = 37;
pub const LOCALE_S1159: DWORD = 40;
pub const LOCALE_S2359: DWORD = 41;
pub const LOCALE_ICALENDARTYPE: DWORD = 0x1009;
pub const LOCALE_IFIRSTDAYOFWEEK: DWORD = 0x100C;
pub const LOCALE_IFIRSTWEEKOFYEAR: DWORD = 0x100D;
pub const LOCALE_SPOSITIVESIGN: DWORD = 80;
pub const LOCALE_SNEGATIVESIGN: DWORD = 81;
pub const LOCALE_FONTSIGNATURE: DWORD = 88;
pub const LOCALE_SNAME: DWORD = 92;
pub const LOCALE_SENGLANGUAGE: DWORD = 0x1001;
pub const LOCALE_SYEARMONTH: DWORD = 0x1006;
pub const LOCALE_IDIGITSUBSTITUTION: DWORD = 0x1014;
pub const LOCALE_IPAPERSIZE: DWORD = 0x100A;
pub const LOCALE_SSHORTTIME: DWORD = 0x0079;

// --- additional LOCALE_* (winnls.h) referenced by locale.rs ---------------
pub const LOCALE_SINTLSYMBOL: DWORD = 21;
pub const LOCALE_IINTLCURRDIGITS: DWORD = 26;
pub const LOCALE_ICENTURY: DWORD = 36;
pub const LOCALE_IDAYLZERO: DWORD = 38;
pub const LOCALE_IMONLZERO: DWORD = 39;
pub const LOCALE_IPOSSIGNPOSN: DWORD = 82;
pub const LOCALE_INEGSIGNPOSN: DWORD = 83;
pub const LOCALE_IPOSSYMPRECEDES: DWORD = 84;
pub const LOCALE_IPOSSEPBYSPACE: DWORD = 85;
pub const LOCALE_INEGSYMPRECEDES: DWORD = 86;
pub const LOCALE_INEGSEPBYSPACE: DWORD = 87;
pub const LOCALE_IOPTIONALCALENDAR: DWORD = 0x100B;
pub const LOCALE_SSORTNAME: DWORD = 0x1013;
pub const LOCALE_SNATIVECURRNAME: DWORD = 0x1008;
pub const LOCALE_IDEFAULTEBCDICCODEPAGE: DWORD = 0x1012;
pub const LOCALE_IDEFAULTUNIXCODEPAGE: DWORD = 0x1030; // Wine extension

pub const LOCALE_SDAYNAME1: DWORD = 42;
pub const LOCALE_SDAYNAME2: DWORD = 43;
pub const LOCALE_SDAYNAME3: DWORD = 44;
pub const LOCALE_SDAYNAME4: DWORD = 45;
pub const LOCALE_SDAYNAME5: DWORD = 46;
pub const LOCALE_SDAYNAME6: DWORD = 47;
pub const LOCALE_SDAYNAME7: DWORD = 48;
pub const LOCALE_SABBREVDAYNAME1: DWORD = 49;
pub const LOCALE_SABBREVDAYNAME2: DWORD = 50;
pub const LOCALE_SABBREVDAYNAME3: DWORD = 51;
pub const LOCALE_SABBREVDAYNAME4: DWORD = 52;
pub const LOCALE_SABBREVDAYNAME5: DWORD = 53;
pub const LOCALE_SABBREVDAYNAME6: DWORD = 54;
pub const LOCALE_SABBREVDAYNAME7: DWORD = 55;
pub const LOCALE_SMONTHNAME1: DWORD = 56;
pub const LOCALE_SMONTHNAME2: DWORD = 57;
pub const LOCALE_SMONTHNAME3: DWORD = 58;
pub const LOCALE_SMONTHNAME4: DWORD = 59;
pub const LOCALE_SMONTHNAME5: DWORD = 60;
pub const LOCALE_SMONTHNAME6: DWORD = 61;
pub const LOCALE_SMONTHNAME7: DWORD = 62;
pub const LOCALE_SMONTHNAME8: DWORD = 63;
pub const LOCALE_SMONTHNAME9: DWORD = 64;
pub const LOCALE_SMONTHNAME10: DWORD = 65;
pub const LOCALE_SMONTHNAME11: DWORD = 66;
pub const LOCALE_SMONTHNAME12: DWORD = 67;
pub const LOCALE_SMONTHNAME13: DWORD = 0x100E;
pub const LOCALE_SABBREVMONTHNAME1: DWORD = 68;
pub const LOCALE_SABBREVMONTHNAME2: DWORD = 69;
pub const LOCALE_SABBREVMONTHNAME3: DWORD = 70;
pub const LOCALE_SABBREVMONTHNAME4: DWORD = 71;
pub const LOCALE_SABBREVMONTHNAME5: DWORD = 72;
pub const LOCALE_SABBREVMONTHNAME6: DWORD = 73;
pub const LOCALE_SABBREVMONTHNAME7: DWORD = 74;
pub const LOCALE_SABBREVMONTHNAME8: DWORD = 75;
pub const LOCALE_SABBREVMONTHNAME9: DWORD = 76;
pub const LOCALE_SABBREVMONTHNAME10: DWORD = 77;
pub const LOCALE_SABBREVMONTHNAME11: DWORD = 78;
pub const LOCALE_SABBREVMONTHNAME12: DWORD = 79;
pub const LOCALE_SABBREVMONTHNAME13: DWORD = 0x100F;

// ===========================================================================
// Constants — winnls.h NORM_* / LINGUISTIC_* / SORT_*
// ===========================================================================
pub const NORM_IGNORECASE: DWORD = 1;
pub const NORM_IGNOREKANATYPE: DWORD = 65536;
pub const NORM_IGNORENONSPACE: DWORD = 2;
pub const NORM_IGNORESYMBOLS: DWORD = 4;
pub const NORM_IGNOREWIDTH: DWORD = 131072;
pub const NORM_LINGUISTIC_CASING: DWORD = 0x08000000;
pub const LINGUISTIC_IGNORECASE: DWORD = 0x00000010;
pub const SORT_STRINGSORT: DWORD = 4096;

// ===========================================================================
// Constants — winnls.h LCMAP_*
// ===========================================================================
pub const LCMAP_LOWERCASE: DWORD = 0x00000100;
pub const LCMAP_UPPERCASE: DWORD = 0x00000200;
pub const LCMAP_SORTKEY: DWORD = 0x00000400;
pub const LCMAP_BYTEREV: DWORD = 0x00000800;
pub const LCMAP_HIRAGANA: DWORD = 0x00100000;
pub const LCMAP_KATAKANA: DWORD = 0x00200000;
pub const LCMAP_HALFWIDTH: DWORD = 0x00400000;
pub const LCMAP_FULLWIDTH: DWORD = 0x00800000;
pub const LCMAP_LINGUISTIC_CASING: DWORD = 0x01000000;

// ===========================================================================
// Constants — winnls.h CP_* / LCID_SUPPORTED / CP_INSTALLED / CP_SUPPORTED
// ===========================================================================
pub const CP_INSTALLED: DWORD = 1;
pub const CP_SUPPORTED: DWORD = 2;
pub const LCID_SUPPORTED: DWORD = 2;
pub const CP_ACP: UINT = 0;
pub const CP_OEMCP: UINT = 1;
pub const CP_MACCP: UINT = 2;
pub const CP_THREAD_ACP: UINT = 3;
pub const CP_SYMBOL: UINT = 42;
pub const CP_UTF7: UINT = 65000;
pub const CP_UTF8: UINT = 65001;
pub const CP_UNIXCP: UINT = CP_ACP; // winnls.h: #define CP_UNIXCP CP_ACP

// winnls.h LCMAP simplified/traditional chinese (used by LCMapString)
pub const LCMAP_SIMPLIFIED_CHINESE: DWORD = 0x02000000;
pub const LCMAP_TRADITIONAL_CHINESE: DWORD = 0x04000000;

// ===========================================================================
// Constants — winnls.h MAP_* (string-mapping flags)
// ===========================================================================
pub const MAP_FOLDCZONE: DWORD = 16;
pub const MAP_FOLDDIGITS: DWORD = 128;
pub const MAP_PRECOMPOSED: DWORD = 32;
pub const MAP_COMPOSITE: DWORD = 64;
pub const MAP_EXPAND_LIGATURES: DWORD = 0x2000;

// ===========================================================================
// Constants — winnls.h WC_* / MB_*
// ===========================================================================
pub const WC_DISCARDNS: DWORD = 0x0010;
pub const WC_SEPCHARS: DWORD = 0x0020;
pub const WC_DEFAULTCHAR: DWORD = 0x0040;
pub const WC_ERR_INVALID_CHARS: DWORD = 0x0080;
pub const WC_COMPOSITECHECK: DWORD = 0x0200;
pub const WC_NO_BEST_FIT_CHARS: DWORD = 0x0400;

pub const MB_PRECOMPOSED: DWORD = 1;
pub const MB_COMPOSITE: DWORD = 2;
pub const MB_ERR_INVALID_CHARS: DWORD = 8;
pub const MB_USEGLYPHCHARS: DWORD = 4;

// ===========================================================================
// Constants — winnls.h CSTR_*
// ===========================================================================
pub const CSTR_LESS_THAN: DWORD = 1;
pub const CSTR_EQUAL: DWORD = 2;
pub const CSTR_GREATER_THAN: DWORD = 3;

// ===========================================================================
// Constants — winnls.h LGRPID_*
// ===========================================================================
pub const LGRPID_INSTALLED: DWORD = 1;
pub const LGRPID_SUPPORTED: DWORD = 2;
pub const LGRPID_WESTERN_EUROPE: DWORD = 1;
pub const LGRPID_CENTRAL_EUROPE: DWORD = 2;
pub const LGRPID_BALTIC: DWORD = 3;
pub const LGRPID_GREEK: DWORD = 4;
pub const LGRPID_CYRILLIC: DWORD = 5;
pub const LGRPID_TURKISH: DWORD = 6;
pub const LGRPID_JAPANESE: DWORD = 7;
pub const LGRPID_KOREAN: DWORD = 8;
pub const LGRPID_TRADITIONAL_CHINESE: DWORD = 9;
pub const LGRPID_SIMPLIFIED_CHINESE: DWORD = 10;
pub const LGRPID_THAI: DWORD = 11;
pub const LGRPID_HEBREW: DWORD = 12;
pub const LGRPID_ARABIC: DWORD = 13;
pub const LGRPID_VIETNAMESE: DWORD = 14;
pub const LGRPID_INDIC: DWORD = 15;
pub const LGRPID_GEORGIAN: DWORD = 16;
pub const LGRPID_ARMENIAN: DWORD = 17;

// ===========================================================================
// Constants — winnls.h GEOCLASS_* / GEOID_NOT_AVAILABLE (enum SYSGEOCLASS)
// ===========================================================================
pub const GEOCLASS_NATION: GEOCLASS = 16;
pub const GEOCLASS_REGION: GEOCLASS = 14;
// winnls.h:  #define GEOID_NOT_AVAILABLE (-1)
pub const GEOID_NOT_AVAILABLE: INT = -1;

// winnls.h enum SYSGEOTYPE — GEOTYPE values (GEO_NATION = 1, then sequential).
pub const GEO_NATION: GEOTYPE = 1;
pub const GEO_LATITUDE: GEOTYPE = 2;
pub const GEO_LONGITUDE: GEOTYPE = 3;
pub const GEO_ISO2: GEOTYPE = 4;
pub const GEO_ISO3: GEOTYPE = 5;
pub const GEO_RFC1766: GEOTYPE = 6;
pub const GEO_LCID: GEOTYPE = 7;
pub const GEO_FRIENDLYNAME: GEOTYPE = 8;
pub const GEO_OFFICIALNAME: GEOTYPE = 9;
pub const GEO_TIMEZONES: GEOTYPE = 10;
pub const GEO_OFFICIALLANGUAGES: GEOTYPE = 11;
pub const GEO_ISO_UN_NUMBER: GEOTYPE = 12;
pub const GEO_PARENT: GEOTYPE = 13;

// ===========================================================================
// Constants — winnls.h IDN_*
// ===========================================================================
pub const IDN_ALLOW_UNASSIGNED: DWORD = 0x1;
pub const IDN_USE_STD3_ASCII_RULES: DWORD = 0x2;

// ===========================================================================
// Constants — winnls.h MUI_*
// ===========================================================================
pub const MUI_LANGUAGE_ID: DWORD = 0x0004;
pub const MUI_LANGUAGE_NAME: DWORD = 0x0008;
pub const MUI_MACHINE_LANGUAGE_SETTINGS: DWORD = 0x0400;

// ===========================================================================
// Constants — locale.c file-local flag masks
//   LOCALE_LOCALEINFOFLAGSMASK = NOUSEROVERRIDE|USE_CP_ACP|RETURN_NUMBER|RETURN_GENITIVE_NAMES
//   MB_FLAGSMASK = PRECOMPOSED|COMPOSITE|USEGLYPHCHARS|ERR_INVALID_CHARS
//   WC_FLAGSMASK = DISCARDNS|SEPCHARS|DEFAULTCHAR|ERR_INVALID_CHARS|COMPOSITECHECK|NO_BEST_FIT_CHARS
// ===========================================================================
pub const LOCALE_LOCALEINFOFLAGSMASK: DWORD =
    LOCALE_NOUSEROVERRIDE | LOCALE_USE_CP_ACP | LOCALE_RETURN_NUMBER | LOCALE_RETURN_GENITIVE_NAMES;
pub const MB_FLAGSMASK: DWORD =
    MB_PRECOMPOSED | MB_COMPOSITE | MB_USEGLYPHCHARS | MB_ERR_INVALID_CHARS;
pub const WC_FLAGSMASK: DWORD = WC_DISCARDNS
    | WC_SEPCHARS
    | WC_DEFAULTCHAR
    | WC_ERR_INVALID_CHARS
    | WC_COMPOSITECHECK
    | WC_NO_BEST_FIT_CHARS;

// ===========================================================================
// Constants — winuser.h RT_STRING  (#define RT_STRING MAKEINTRESOURCE(6))
//   MAKEINTRESOURCE(i) == ((LPWSTR)((ULONG_PTR)((WORD)(i))))  →  numeric 6
// ===========================================================================
pub const RT_STRING: DWORD = 6;

// ===========================================================================
// winnt.h LANG_* / SUBLANG_* / SORT_DEFAULT are NOT defined here — they are
// already exported (as u32) by ntdll_sys::nt (bindgen). Re-defining them would
// create an ambiguous-glob-reexport in locale.rs (which globs both modules).
// This crate's own helpers reference the nt versions with explicit `as WORD`.
// ===========================================================================

// ===========================================================================
// Constants — winnt.h computed LCID/LANGID defaults
//   MAKELANGID(p,s) = (s<<10)|p ;  MAKELCID(lgid,srt) = (srt<<16)|lgid ; SORT_DEFAULT=0
//   LANG_USER_DEFAULT   = MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT)     = 0x0400
//   LANG_SYSTEM_DEFAULT = MAKELANGID(LANG_NEUTRAL, SUBLANG_SYS_DEFAULT) = 0x0800
//   LOCALE_USER_DEFAULT   = MAKELCID(LANG_USER_DEFAULT,   0) = 0x0400
//   LOCALE_SYSTEM_DEFAULT = MAKELCID(LANG_SYSTEM_DEFAULT, 0) = 0x0800
//   LOCALE_NEUTRAL   = MAKELCID(MAKELANGID(LANG_NEUTRAL,SUBLANG_NEUTRAL),0) = 0x0000
//   LOCALE_INVARIANT = MAKELCID(MAKELANGID(LANG_INVARIANT,SUBLANG_NEUTRAL),0) = 0x007f
// ===========================================================================
pub const LOCALE_USER_DEFAULT: LCID = 0x0400;
pub const LOCALE_SYSTEM_DEFAULT: LCID = 0x0800;
pub const LOCALE_NEUTRAL: LCID = 0x0000;
pub const LOCALE_INVARIANT: LCID = 0x007f;

// ===========================================================================
// Constants — locale categories (LC_*). DEAD: the only C users are locale.c's
// setlocale() paths at 801-1168, all inside `#ifndef __REACTOS__`, and nothing in
// this crate references them. The six sdk/include/crt/locale.h defines are given
// that header's values (LC_ALL=0 COLLATE=1 CTYPE=2 MONETARY=3 NUMERIC=4 TIME=5 —
// they were glibc ordinals before). MESSAGES/PAPER/TELEPHONE/MEASUREMENT have no
// ReactOS definition (locale.c:1154-1162 #ifdef-guards them); glibc values kept.
// ===========================================================================
pub const LC_ALL: i32 = 0;
pub const LC_COLLATE: i32 = 1;
pub const LC_CTYPE: i32 = 2;
pub const LC_MONETARY: i32 = 3;
pub const LC_NUMERIC: i32 = 4;
pub const LC_TIME: i32 = 5;
pub const LC_MESSAGES: i32 = 5;
pub const LC_PAPER: i32 = 7;
pub const LC_TELEPHONE: i32 = 10;
pub const LC_MEASUREMENT: i32 = 11;

// ===========================================================================
// Constants — RFC-3492 punycode (locale.c IdnToAscii `enum { ... }`)
//   BASE=36 TMIN=1 TMAX=26 SKEW=38 DAMP=700 (INIT_BIAS/INIT_N are fn-local in locale.rs)
//   Typed INT (i32) because they are used in signed adapt() arithmetic.
// ===========================================================================
pub const BASE: INT = 36;
pub const TMIN: INT = 1;
pub const TMAX: INT = 26;
pub const SKEW: INT = 38;
pub const DAMP: INT = 700;

// ===========================================================================
// CoreFoundation (Apple) stubs — referenced only by the dead Unix/Mac
// get_mac_locale path. Declared so the crate type-checks; never executed on
// Windows. If link later complains, cfg these (and their call sites) out.
// ===========================================================================
pub type CFStringRef = *const c_void;
pub type CFLocaleRef = *const c_void;
pub type CFArrayRef = *const c_void;
pub type CFDictionaryRef = *const c_void;
pub type CFAllocatorRef = *mut c_void;
pub type CFStringEncoding = u32;
pub type CFIndex = isize;
pub type CFTypeRef = *const c_void;

pub const kCFStringEncodingUTF8: CFStringEncoding = 0x08000100;

extern "C" {
    pub static kCFLocaleCountryCode: CFStringRef;
    pub static kCFLocaleLanguageCode: CFStringRef;

    pub fn CFLocaleCopyCurrent() -> CFLocaleRef;
    pub fn CFLocaleGetValue(locale: CFLocaleRef, key: CFStringRef) -> CFTypeRef;
    pub fn CFLocaleCopyPreferredLanguages() -> CFArrayRef;
    pub fn CFLocaleCreateComponentsFromLocaleIdentifier(
        alloc: CFAllocatorRef,
        localeID: CFStringRef,
    ) -> CFDictionaryRef;
    pub fn CFDictionaryGetValue(dict: CFDictionaryRef, key: *const c_void) -> *const c_void;
    pub fn CFArrayGetCount(array: CFArrayRef) -> CFIndex;
    pub fn CFArrayGetValueAtIndex(array: CFArrayRef, idx: CFIndex) -> *const c_void;
    pub fn CFStringCreateCopy(alloc: CFAllocatorRef, str_: CFStringRef) -> CFStringRef;
    pub fn CFStringCreateWithFormat(
        alloc: CFAllocatorRef,
        formatOptions: *mut c_void,
        format: CFStringRef,
        ...
    ) -> CFStringRef;
    pub fn CFStringGetCString(
        str_: CFStringRef,
        buffer: *mut i8,
        bufferSize: CFIndex,
        encoding: CFStringEncoding,
    ) -> u8;
    pub fn CFRelease(cf: CFTypeRef);
}

/// CFSTR("...") — CoreFoundation constant-string macro. Dead on Windows; the
/// real macro builds a CFString from a C literal. Returns the raw pointer
/// unchanged so the (never-executed) call sites type-check.
#[inline]
pub unsafe fn CFSTR(s: *const i8) -> CFStringRef {
    s as CFStringRef
}

// ===========================================================================
// winnls.h NLS structs
// ===========================================================================
pub const MAX_DEFAULTCHAR: usize = 2;
pub const MAX_LEADBYTES: usize = 12;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CPINFO {
    pub MaxCharSize: UINT,
    pub DefaultChar: [BYTE; MAX_DEFAULTCHAR],
    pub LeadByte: [BYTE; MAX_LEADBYTES],
}
pub type LPCPINFO = *mut CPINFO;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CPINFOEXA {
    pub MaxCharSize: UINT,
    pub DefaultChar: [BYTE; MAX_DEFAULTCHAR],
    pub LeadByte: [BYTE; MAX_LEADBYTES],
    pub UnicodeDefaultChar: WCHAR,
    pub CodePage: UINT,
    pub CodePageName: [CHAR; 260], // MAX_PATH
}
pub type LPCPINFOEXA = *mut CPINFOEXA;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CPINFOEXW {
    pub MaxCharSize: UINT,
    pub DefaultChar: [BYTE; MAX_DEFAULTCHAR],
    pub LeadByte: [BYTE; MAX_LEADBYTES],
    pub UnicodeDefaultChar: WCHAR,
    pub CodePage: UINT,
    pub CodePageName: [WCHAR; 260], // MAX_PATH
}
pub type LPCPINFOEXW = *mut CPINFOEXW;

// winnls.h: _nlsversioninfo (base, pre-Win8 layout used by kernel32)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NLSVERSIONINFO {
    pub dwNLSVersionInfoSize: DWORD,
    pub dwNLSVersion: DWORD,
    pub dwDefinedVersion: DWORD,
}
pub type LPNLSVERSIONINFO = *mut NLSVERSIONINFO;

// winnls.h: _FILEMUIINFO
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILEMUIINFO {
    pub dwSize: DWORD,
    pub dwVersion: DWORD,
    pub dwFileType: DWORD,
    pub pChecksum: [BYTE; 16],
    pub pServiceChecksum: [BYTE; 16],
    pub dwLanguageNameOffset: DWORD,
    pub dwTypeIDMainSize: DWORD,
    pub dwTypeIDMainOffset: DWORD,
    pub dwTypeNameMainOffset: DWORD,
    pub dwTypeIDMUISize: DWORD,
    pub dwTypeIDMUIOffset: DWORD,
    pub dwTypeNameMUIOffset: DWORD,
    pub abBuffer: [BYTE; 8],
}
pub type PFILEMUIINFO = *mut FILEMUIINFO;

// ===========================================================================
// winnls.h enumeration callback typedefs.
//   On i386 NTAPI/CALLBACK == stdcall; `extern "system"` resolves per-arch.
// ===========================================================================
pub type LOCALE_ENUMPROCA = Option<unsafe extern "system" fn(LPSTR) -> BOOL>;
pub type LOCALE_ENUMPROCW = Option<unsafe extern "system" fn(LPWSTR) -> BOOL>;
pub type CODEPAGE_ENUMPROCA = Option<unsafe extern "system" fn(LPSTR) -> BOOL>;
pub type CODEPAGE_ENUMPROCW = Option<unsafe extern "system" fn(LPWSTR) -> BOOL>;
pub type LANGUAGEGROUP_ENUMPROCA =
    Option<unsafe extern "system" fn(LGRPID, LPSTR, LPSTR, DWORD, LONG_PTR) -> BOOL>;
pub type LANGUAGEGROUP_ENUMPROCW =
    Option<unsafe extern "system" fn(LGRPID, LPWSTR, LPWSTR, DWORD, LONG_PTR) -> BOOL>;
pub type LANGGROUPLOCALE_ENUMPROCA =
    Option<unsafe extern "system" fn(LGRPID, LCID, LPSTR, LONG_PTR) -> BOOL>;
pub type LANGGROUPLOCALE_ENUMPROCW =
    Option<unsafe extern "system" fn(LGRPID, LCID, LPWSTR, LONG_PTR) -> BOOL>;
pub type UILANGUAGE_ENUMPROCA = Option<unsafe extern "system" fn(LPSTR, LONG_PTR) -> BOOL>;
pub type UILANGUAGE_ENUMPROCW = Option<unsafe extern "system" fn(LPWSTR, LONG_PTR) -> BOOL>;
pub type GEO_ENUMPROC = Option<unsafe extern "system" fn(GEOID) -> BOOL>;

// ===========================================================================
// locale.c file-local callback-table structs (typedef anonymous struct).
// ===========================================================================
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ENUMLANGUAGEGROUP_CALLBACKS {
    pub procA: LANGUAGEGROUP_ENUMPROCA,
    pub procW: LANGUAGEGROUP_ENUMPROCW,
    pub dwFlags: DWORD,
    pub lParam: LONG_PTR,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ENUMLANGUAGEGROUPLOCALE_CALLBACKS {
    pub procA: LANGGROUPLOCALE_ENUMPROCA,
    pub procW: LANGGROUPLOCALE_ENUMPROCW,
    pub dwFlags: DWORD,
    pub lgrpid: LGRPID,
    pub lParam: LONG_PTR,
}

// locale.c:  typedef struct { union { UILANGUAGE_ENUMPROCA procA;
//            UILANGUAGE_ENUMPROCW procW; } u; DWORD flags; LONG_PTR param; }
//            ENUM_UILANG_CALLBACK;
#[repr(C)]
#[derive(Clone, Copy)]
pub union ENUM_UILANG_CALLBACK_u {
    pub procA: UILANGUAGE_ENUMPROCA,
    pub procW: UILANGUAGE_ENUMPROCW,
}
// rustconv emitted both names for the anonymous union; alias them.
pub type C2RustUnnamed_10 = ENUM_UILANG_CALLBACK_u;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ENUM_UILANG_CALLBACK {
    pub u: ENUM_UILANG_CALLBACK_u,
    pub flags: DWORD,
    pub param: LONG_PTR,
}

// ===========================================================================
// libc wide-char primitives (the Wine `unicode.h` macros forward to these).
// ===========================================================================
extern "C" {
    pub fn wcscmp(s1: *const WCHAR, s2: *const WCHAR) -> i32;
    pub fn wcscpy(dst: *mut WCHAR, src: *const WCHAR) -> *mut WCHAR;
    pub fn wcscat(dst: *mut WCHAR, src: *const WCHAR) -> *mut WCHAR;
    pub fn wcsrchr(s: *const WCHAR, c: WCHAR) -> *mut WCHAR;
    pub fn wcschr(s: *const WCHAR, c: WCHAR) -> *mut WCHAR;
    pub fn wcspbrk(s: *const WCHAR, accept: *const WCHAR) -> *mut WCHAR;
    pub fn wcsncpy(dst: *mut WCHAR, src: *const WCHAR, n: usize) -> *mut WCHAR;
    pub fn towupper(ch: WCHAR) -> WCHAR;
    pub fn towlower(ch: WCHAR) -> WCHAR;
    pub fn wcstol(nptr: *const WCHAR, endptr: *mut *mut WCHAR, base: i32) -> i32;
    pub fn wcstoul(nptr: *const WCHAR, endptr: *mut *mut WCHAR, base: i32) -> u32;

    // Plain libc (narrow) helpers used in the Unix-locale paths.
    pub fn bsearch(
        key: *const c_void,
        base: *const c_void,
        nmemb: usize,
        size: usize,
        compar: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> i32>,
    ) -> *mut c_void;
    pub fn getenv(name: *const i8) -> *mut i8;
    pub fn sprintf(buf: *mut i8, fmt: *const i8, ...) -> i32;
    pub fn strcat(dst: *mut i8, src: *const i8) -> *mut i8;
    pub fn strtol(nptr: *const i8, endptr: *mut *mut i8, base: i32) -> i32;

    // NOTE: WARN and sprintfW are no longer declared here — they are defined as
    // `#[no_mangle]` Rust fns at the bottom of this file (the def is both the
    // symbol provider and the call-site item).
}

// ===========================================================================
// Wine `unicode.h` "W" string helpers — macro → libc wide-char mappings.
// ===========================================================================
#[inline]
pub unsafe fn strcmpW(s1: *const WCHAR, s2: *const WCHAR) -> i32 {
    wcscmp(s1, s2)
}
#[inline]
pub unsafe fn strcpyW(dst: *mut WCHAR, src: *const WCHAR) -> *mut WCHAR {
    wcscpy(dst, src)
}
#[inline]
pub unsafe fn strcatW(dst: *mut WCHAR, src: *const WCHAR) -> *mut WCHAR {
    wcscat(dst, src)
}
#[inline]
pub unsafe fn strrchrW(s: *const WCHAR, c: WCHAR) -> *mut WCHAR {
    wcsrchr(s, c)
}
#[inline]
pub unsafe fn strchrW(s: *const WCHAR, c: WCHAR) -> *mut WCHAR {
    wcschr(s, c)
}
#[inline]
pub unsafe fn strpbrkW(s: *const WCHAR, accept: *const WCHAR) -> *mut WCHAR {
    wcspbrk(s, accept)
}
#[inline]
pub unsafe fn strtolW(s: *const WCHAR, endptr: *mut *mut WCHAR, base: i32) -> i32 {
    wcstol(s, endptr, base)
}
#[inline]
pub unsafe fn strtoulW(s: *const WCHAR, endptr: *mut *mut WCHAR, base: i32) -> u32 {
    wcstoul(s, endptr, base)
}
#[inline]
pub unsafe fn toupperW(ch: WCHAR) -> WCHAR {
    towupper(ch)
}

// Wine `lstrcpynW`-style bounded copy used by parse_locale_name
// (strncpyW + explicit NUL terminate; matches Wine's lstrcpynW semantics).
#[inline]
pub unsafe fn lstrcpynW(dst: *mut WCHAR, src: *const WCHAR, n: i32) -> *mut WCHAR {
    if n <= 0 {
        return dst;
    }
    let mut i: isize = 0;
    let last = (n - 1) as isize;
    while i < last {
        let c = *src.offset(i);
        *dst.offset(i) = c;
        if c == 0 {
            return dst;
        }
        i += 1;
    }
    *dst.offset(last) = 0;
    dst
}

// lstrlenW / lstrcpyW are real kernel32 exports (NOT in unicode.h) — declare them.
extern "system" {
    pub fn lstrlenW(s: LPCWSTR) -> INT;
    pub fn lstrcpyW(dst: LPWSTR, src: LPCWSTR) -> LPWSTR;
}

// get_char_typeW — Wine `unicode.h` static-inline over wine_wctype_table.
extern "C" {
    pub static wine_wctype_table: [u16; 0];
}
#[inline]
pub unsafe fn get_char_typeW(ch: WCHAR) -> u16 {
    let base = wine_wctype_table.as_ptr();
    let hi = *base.offset((ch >> 8) as isize) as isize;
    *base.offset(hi + (ch & 0xff) as isize)
}

// strcmpiW / strncmpiW — Wine case-insensitive compares (unicode.h forwards to
// _wcsicmp / _wcsnicmp). Used by parse_locale_name's find_locale_id_callback.
extern "C" {
    pub fn _wcsicmp(s1: *const WCHAR, s2: *const WCHAR) -> i32;
    pub fn _wcsnicmp(s1: *const WCHAR, s2: *const WCHAR, n: usize) -> i32;
}
#[inline]
pub unsafe fn strcmpiW(s1: *const WCHAR, s2: *const WCHAR) -> i32 {
    _wcsicmp(s1, s2)
}
#[inline]
pub unsafe fn strncmpiW(s1: *const WCHAR, s2: *const WCHAR, n: i32) -> i32 {
    _wcsnicmp(s1, s2, n as usize)
}

// ===========================================================================
// Wine libunicode externs (defined in sortkey.c / casemap.c / codepage.c that
// stay compiled) — just DECLARE them, signatures from wine/unicode.h.
//   wine_is_dbcs_leadbyte returns u8 here to match locale.rs's BOOL=u8 call site
//   (IsDBCSLeadByte returns the value directly).  // TODO verify (C decl is int)
// ===========================================================================
extern "C" {
    pub fn wine_compare_string(
        flags: i32,
        str1: *const WCHAR,
        len1: i32,
        str2: *const WCHAR,
        len2: i32,
    ) -> i32;
    pub fn wine_get_sortkey(
        flags: i32,
        src: *const WCHAR,
        srclen: i32,
        dst: *mut i8,
        dstlen: i32,
    ) -> i32;
    pub fn wine_fold_string(
        flags: i32,
        src: *const WCHAR,
        srclen: i32,
        dst: *mut WCHAR,
        dstlen: i32,
    ) -> i32;
    // NOTE: the wine *codepage-table* helpers (wine_cp_get_table, wine_cp_mbstowcs,
    // wine_cp_wcstombs, wine_cpsymbol_*, wine_utf8_*, wine_is_dbcs_leadbyte) are
    // NOT declared here. They live only in the host libunicode.a; ReactOS routes
    // all of ANSI/OEM/UTF8/DBCS conversion through nls.c instead, so locale.c
    // gates every call to them behind `#ifndef __REACTOS__`. The translation kept
    // those dead branches, leaving dangling references. The `#[no_mangle]` stub
    // bodies below provide the symbols so the (runtime-unreachable, nls.c-shadowed)
    // dead branches link without dragging in the host wine codepage tables.
}

// wine_dbgstr_w — Wine debug helper (wine/debug.h). Returns an empty C string;
// dead under no-logging builds.  debugstr_w (with one arg) is already declared
// in locale.rs's own extern block, so only wine_dbgstr_w is provided here.
#[inline]
pub unsafe fn wine_dbgstr_w(_s: *const WCHAR) -> *const u8 {
    b"\0".as_ptr()
}

// debugstr_an / debugstr_wn — Wine TRACE() debug helpers (wine/debug.h forwards
// to wine_dbgstr_an/_wn). Dead under no logging — return an empty string.
#[inline]
pub unsafe fn debugstr_an(_s: *const i8, _n: i32) -> *const i8 {
    b"\0".as_ptr() as *const i8
}
#[inline]
pub unsafe fn debugstr_wn(_s: *const WCHAR, _n: i32) -> *const u8 {
    b"\0".as_ptr()
}

// ===========================================================================
// C `min` macro — locale.rs calls min(a, b) on INT operands.
// ===========================================================================
#[inline]
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b {
        a
    } else {
        b
    }
}

// ===========================================================================
// dllmain.c:  HMODULE kernel32_handle;
// ===========================================================================
extern "C" {
    pub static mut kernel32_handle: HMODULE;
}

// ===========================================================================
// MAKELANGID / PRIMARYLANGID  (winnt.h macros — locale.rs declares MAKELCID,
// MAKELANGID, SUBLANGID, SORTIDFROMLCID itself; only PRIMARYLANGID + a local
// MAKELANGID are needed by parse_locale_name below).
// ===========================================================================
#[inline]
pub fn PRIMARYLANGID(lgid: LANGID) -> WORD {
    (lgid as WORD) & 0x3ff
}
// winnt.h: MAKELANGID(p,s) = (((WORD)(s))<<10)|(WORD)(p) ; args come in as u32
// (the nt-exported LANG_*/SUBLANG_* constants), result is a LANGID (WORD).
#[inline]
pub fn MAKELANGID(p: DWORD, s: DWORD) -> LANGID {
    (((s as u16) << 10) | (p as u16)) as LANGID
}
// winnt.h: LANGIDFROMLCID(lcid) = (WORD)(lcid)
#[inline]
pub fn LANGIDFROMLCID(lcid: LCID) -> LANGID {
    lcid as LANGID
}

// ---------------------------------------------------------------------------
// locale.c table-accessor macros (full2half / dakuten kana tables) + ARRAY_SIZE
//   GET_FULL(t,i)=t[i][0]  GET_HALF1(t,i)=t[i][1]  GET_HALF2(t,i)=t[i][2]
//   GET_VOICED(t,i)=t[i][0] GET_SINGLE1(t,i)=t[i][1] GET_SINGLE2(t,i)=t[i][2]
//   ARRAY_SIZE(a)=sizeof(a)/sizeof(a[0])
// `t` is a *const [WCHAR;3] row pointer in locale.rs; `a` is a sized array.
// Re-exported via `pub(crate) use` so locale.rs's `use nls_subst::*` picks them up.
// ---------------------------------------------------------------------------
macro_rules! GET_FULL {
    ($t:expr, $i:expr) => {
        (*$t.offset($i as isize))[0]
    };
}
macro_rules! GET_HALF1 {
    ($t:expr, $i:expr) => {
        (*$t.offset($i as isize))[1]
    };
}
macro_rules! GET_HALF2 {
    ($t:expr, $i:expr) => {
        (*$t.offset($i as isize))[2]
    };
}
macro_rules! GET_VOICED {
    ($t:expr, $i:expr) => {
        (*$t.offset($i as isize))[0]
    };
}
macro_rules! GET_SINGLE1 {
    ($t:expr, $i:expr) => {
        (*$t.offset($i as isize))[1]
    };
}
macro_rules! GET_SINGLE2 {
    ($t:expr, $i:expr) => {
        (*$t.offset($i as isize))[2]
    };
}
macro_rules! ARRAY_SIZE {
    ($a:expr) => {
        ($a.len())
    };
}
pub(crate) use ARRAY_SIZE;
pub(crate) use GET_FULL;
pub(crate) use GET_HALF1;
pub(crate) use GET_HALF2;
pub(crate) use GET_SINGLE1;
pub(crate) use GET_SINGLE2;
pub(crate) use GET_VOICED;
#[inline]
fn make_langid(p: WORD, s: WORD) -> LANGID {
    (((s as u32) << 10) | (p as u32)) as LANGID
}
#[inline]
fn make_lcid(lgid: LANGID, srtid: DWORD) -> LCID {
    ((srtid << 16) | (lgid as u32)) as LCID
}

// ===========================================================================
// parse_locale_name — HAND-TRANSLATED from
//   dll/win32/kernel32/winnls/string/locale.c (the `#if 0`/kernel32_vista
//   helper that the build drops, hence "missing"). C signature:
//     static void parse_locale_name(const WCHAR *str, struct locale_name *name)
//   Parses lang[_country][.charset][@modifier] (Unix) or
//   lang[-script][-country][_modifier] (Windows) into `locale_name`.
//   `locale_name` is defined in crate::string::locale (we fill it in place).
// ===========================================================================
use crate::string::locale::locale_name;

// Helpers the dropped path used; reuse the data table + GetLocaleInfoW from locale.rs.
use crate::string::locale::{charset_entry, charset_names, GetLocaleInfoW};

extern "C" {
    fn isalnum(c: i32) -> i32;
}

// find_charset — locale.c: strip punctuation, bsearch charset_names → codepage.
unsafe fn find_charset(name: *const WCHAR) -> UINT {
    let mut charset_name: [u8; 16] = [0; 16];
    let mut i: isize = 0;
    let mut j: usize = 0;
    while *name.offset(i) != 0 && j < charset_name.len() - 1 {
        let ch = *name.offset(i);
        if ch < 256 && isalnum(ch as i32) != 0 {
            charset_name[j] = ch as u8;
            j += 1;
        }
        i += 1;
    }
    charset_name[j] = 0;

    let entry = bsearch(
        charset_name.as_ptr() as *const c_void,
        charset_names.as_ptr() as *const c_void,
        charset_names.len(),
        core::mem::size_of::<charset_entry>(),
        Some(charset_cmp),
    ) as *const charset_entry;
    if !entry.is_null() {
        return (*entry).codepage;
    }
    0
}

// charset_cmp — bsearch comparator: case-insensitive strcmp(key, entry->name).
unsafe extern "C" fn charset_cmp(key: *const c_void, elem: *const c_void) -> i32 {
    let name = key as *const i8;
    let entry = elem as *const charset_entry;
    strcasecmp(name, (*entry).charset_name)
}

// locale.c does `#define strcasecmp _stricmp` under __REACTOS__ — there is no
// `strcasecmp` symbol in the CRT, so map it to the CRT's `_stricmp` (libcntpr).
extern "C" {
    fn _stricmp(s1: *const i8, s2: *const i8) -> i32;
}
#[inline]
unsafe fn strcasecmp(s1: *const i8, s2: *const i8) -> i32 {
    _stricmp(s1, s2)
}

// get_default_sublang — locale.c (also private in locale.rs; duplicated here
// for find_locale_id_callback. Pure logic, no exported symbol → no clash).
unsafe fn get_default_sublang(mut lang: LANGID) -> LANGID {
    let span = LANG_SPANISH as WORD;
    let zh = LANG_CHINESE as WORD;
    match PRIMARYLANGID(lang) {
        x if x == span => {
            return make_langid(span, SUBLANG_SPANISH_MODERN as WORD)
        }
        x if x == zh => match lang {
            l if l == make_langid(zh, SUBLANG_CHINESE_SINGAPORE as WORD) => {
                return make_langid(zh, SUBLANG_CHINESE_SIMPLIFIED as WORD)
            }
            l if l == make_langid(zh, SUBLANG_CHINESE_TRADITIONAL as WORD)
                || l == make_langid(zh, SUBLANG_CHINESE_MACAU as WORD) =>
            {
                return make_langid(zh, SUBLANG_CHINESE_HONGKONG as WORD)
            }
            l if l == make_langid(zh, SUBLANG_NEUTRAL as WORD) => {
                return make_langid(zh, SUBLANG_CHINESE_SIMPLIFIED as WORD)
            }
            _ => {}
        },
        _ => {}
    }
    if (lang >> 10) == SUBLANG_NEUTRAL as WORD {
        lang = make_langid(PRIMARYLANGID(lang), SUBLANG_DEFAULT as WORD);
    }
    lang
}

// find_locale_id_callback — locale.c EnumResourceLanguagesW callback. Matches
// `name->win_name/lang/script/country/codepage` against each candidate LCID.
unsafe extern "system" fn find_locale_id_callback(
    _hmodule: HMODULE,
    _type: LPCWSTR,
    _name: LPCWSTR,
    lang: WORD,
    lparam: LONG_PTR,
) -> BOOL {
    let data = lparam as *mut locale_name;
    let mut buffer: [WCHAR; 128] = [0; 128];
    let mut matches: i32 = 0;
    let lcid: LCID = make_lcid(lang as LANGID, SORT_DEFAULT);

    if PRIMARYLANGID(lang) == LANG_NEUTRAL as WORD {
        return 1; /* continue search */
    }

    let buflen = (buffer.len()) as INT;

    /* first check exact name */
    if (*data).win_name[0] != 0
        && GetLocaleInfoW(
            lcid,
            (LOCALE_SNAME | LOCALE_NOUSEROVERRIDE) as LCTYPE,
            buffer.as_mut_ptr(),
            buflen,
        ) != 0
    {
        if strcmpiW((*data).win_name.as_ptr(), buffer.as_ptr()) == 0 {
            matches = 4; /* everything matches */
            return finish_match(data, lcid, matches);
        }
    }

    if GetLocaleInfoW(
        lcid,
        (LOCALE_SISO639LANGNAME | LOCALE_NOUSEROVERRIDE) as LCTYPE,
        buffer.as_mut_ptr(),
        buflen,
    ) == 0
    {
        return 1;
    }
    if strcmpiW(buffer.as_ptr(), (*data).lang.as_ptr()) != 0 {
        return 1;
    }
    matches += 1; /* language name matched */

    if !(*data).script.is_null() {
        if GetLocaleInfoW(
            lcid,
            (LOCALE_SSCRIPTS | LOCALE_NOUSEROVERRIDE) as LCTYPE,
            buffer.as_mut_ptr(),
            buflen,
        ) != 0
        {
            let mut p: *const WCHAR = buffer.as_ptr();
            let len = lstrlenW((*data).script);
            loop {
                if *p == 0 {
                    return finish_match(data, lcid, matches);
                }
                if strncmpiW(p, (*data).script, len) == 0
                    && (*p.offset(len as isize) == 0 || *p.offset(len as isize) == ';' as WCHAR)
                {
                    break;
                }
                let semi = strchrW(p, ';' as WCHAR);
                if semi.is_null() {
                    return finish_match(data, lcid, matches);
                }
                p = semi.offset(1);
            }
            matches += 1; /* script matched */
        }
    }

    if !(*data).country.is_null() {
        if GetLocaleInfoW(
            lcid,
            (LOCALE_SISO3166CTRYNAME | LOCALE_NOUSEROVERRIDE) as LCTYPE,
            buffer.as_mut_ptr(),
            buflen,
        ) != 0
        {
            if strcmpiW(buffer.as_ptr(), (*data).country) != 0 {
                return finish_match(data, lcid, matches);
            }
            matches += 1; /* country name matched */
        }
    } else {
        /* match default language */
        let def_lang: LANGID = if !(*data).script.is_null() {
            lang as LANGID
        } else {
            make_langid(PRIMARYLANGID(lang), LANG_NEUTRAL as WORD)
        };
        if lang as LANGID == get_default_sublang(def_lang) {
            matches += 1;
        }
    }

    if (*data).codepage != 0 {
        let mut unix_cp: UINT = 0;
        if GetLocaleInfoW(
            lcid,
            (LOCALE_IDEFAULTUNIXCODEPAGE | LOCALE_RETURN_NUMBER) as LCTYPE,
            &mut unix_cp as *mut UINT as LPWSTR,
            (core::mem::size_of::<UINT>() / core::mem::size_of::<WCHAR>()) as INT,
        ) != 0
        {
            if unix_cp == (*data).codepage {
                matches += 1;
            }
        }
    }

    finish_match(data, lcid, matches)
}

#[inline]
unsafe fn finish_match(data: *mut locale_name, lcid: LCID, matches: i32) -> BOOL {
    if matches > (*data).matches {
        (*data).lcid = lcid;
        (*data).matches = matches;
    }
    (((*data).matches < 4) as i32) as BOOL /* stop on perfect match */
}

// winnls.h LCTYPE values referenced only by find_locale_id_callback.
const LOCALE_SISO639LANGNAME: DWORD = 89;
const LOCALE_SISO3166CTRYNAME: DWORD = 90;
const LOCALE_SSCRIPTS: DWORD = 108;
// LOCALE_IDEFAULTUNIXCODEPAGE already defined above (pub const, line ~157).

extern "system" {
    fn EnumResourceLanguagesW(
        hModule: HMODULE,
        lpType: LPCWSTR,
        lpName: LPCWSTR,
        lpEnumFunc: ENUMRESLANGPROCW,
        lParam: LONG_PTR,
    ) -> BOOL;
}

/// parse_locale_name — fills `*name` from `str`. Faithful translation of the
/// locale.c implementation; returns void (the rustconv call sites ignore it).
pub unsafe fn parse_locale_name(str: *const WCHAR, name: *mut locale_name) {
    static SEP_W: [WCHAR; 5] = [
        '-' as WCHAR,
        '_' as WCHAR,
        '.' as WCHAR,
        '@' as WCHAR,
        0,
    ];
    static WINSEP_W: [WCHAR; 3] = ['-' as WCHAR, '_' as WCHAR, 0];
    static POSIX_W: [WCHAR; 6] = [
        'P' as WCHAR,
        'O' as WCHAR,
        'S' as WCHAR,
        'I' as WCHAR,
        'X' as WCHAR,
        0,
    ];
    static C_W: [WCHAR; 2] = ['C' as WCHAR, 0];
    static LATIN_W: [WCHAR; 6] = [
        'l' as WCHAR,
        'a' as WCHAR,
        't' as WCHAR,
        'i' as WCHAR,
        'n' as WCHAR,
        0,
    ];
    static LATN_W: [WCHAR; 6] = [
        '-' as WCHAR,
        'L' as WCHAR,
        'a' as WCHAR,
        't' as WCHAR,
        'n' as WCHAR,
        0,
    ];

    (*name).country = core::ptr::null_mut();
    (*name).charset = core::ptr::null_mut();
    (*name).script = core::ptr::null_mut();
    (*name).modifier = core::ptr::null_mut();
    (*name).lcid = make_lcid(
        make_langid(LANG_ENGLISH as WORD, SUBLANG_DEFAULT as WORD),
        SORT_DEFAULT,
    );
    (*name).matches = 0;
    (*name).codepage = 0;
    (*name).win_name[0] = 0;
    lstrcpynW(
        (*name).lang.as_mut_ptr(),
        str,
        ((*name).lang.len()) as i32,
    );

    if (*name).lang[0] == 0 {
        (*name).lcid = LOCALE_INVARIANT;
        (*name).matches = 4;
        return;
    }

    let mut p: *mut WCHAR = strpbrkW((*name).lang.as_ptr(), SEP_W.as_ptr());
    if p.is_null() {
        if strcmpW((*name).lang.as_ptr(), POSIX_W.as_ptr()) == 0
            || strcmpW((*name).lang.as_ptr(), C_W.as_ptr()) == 0
        {
            (*name).matches = 4; /* perfect match for default English lcid */
            return;
        }
        strcpyW((*name).win_name.as_mut_ptr(), (*name).lang.as_ptr());
    } else if *p == '-' as WCHAR {
        /* Windows format */
        strcpyW((*name).win_name.as_mut_ptr(), (*name).lang.as_ptr());
        *p = 0;
        p = p.offset(1);
        (*name).country = p;
        let mut q = strpbrkW(p, WINSEP_W.as_ptr());
        if !q.is_null() && *q == '-' as WCHAR {
            *q = 0;
            q = q.offset(1);
            (*name).script = (*name).country;
            (*name).country = q;
            q = strpbrkW(q, WINSEP_W.as_ptr());
        }
        if !q.is_null() {
            *q = 0;
            q = q.offset(1);
            (*name).modifier = q;
        }
        /* second value can be script or country; resolve by length */
        if (*name).script.is_null() && lstrlenW((*name).country) == 4 {
            (*name).script = (*name).country;
            (*name).country = core::ptr::null_mut();
        }
    } else {
        /* Unix format */
        if *p == '_' as WCHAR {
            *p = 0;
            p = p.offset(1);
            (*name).country = p;
            p = strpbrkW(p, SEP_W[2..].as_ptr()); /* ".@" */
        }
        if !p.is_null() && *p == '.' as WCHAR {
            *p = 0;
            p = p.offset(1);
            (*name).charset = p;
            p = strchrW(p, '@' as WCHAR);
        }
        if !p.is_null() {
            *p = 0;
            p = p.offset(1);
            (*name).modifier = p;
        }

        if !(*name).charset.is_null() {
            (*name).codepage = find_charset((*name).charset);
        }

        /* rebuild a Windows name if possible */
        if !(*name).charset.is_null() {
            // can't specify charset in Windows format → done
            EnumResourceLanguagesW(
                kernel32_handle,
                RT_STRING as LPCWSTR,
                LOCALE_ILANGUAGE as LPCWSTR,
                as_enumreslang_w(find_locale_id_callback),
                name as LONG_PTR,
            );
            return;
        }
        if !(*name).modifier.is_null() && strcmpW((*name).modifier, LATIN_W.as_ptr()) != 0 {
            // only Latn script supported for now → done
            EnumResourceLanguagesW(
                kernel32_handle,
                RT_STRING as LPCWSTR,
                LOCALE_ILANGUAGE as LPCWSTR,
                as_enumreslang_w(find_locale_id_callback),
                name as LONG_PTR,
            );
            return;
        }
        strcpyW((*name).win_name.as_mut_ptr(), (*name).lang.as_ptr());
        if !(*name).modifier.is_null() {
            strcatW((*name).win_name.as_mut_ptr(), LATN_W.as_ptr());
        }
        if !(*name).country.is_null() {
            let end = (*name)
                .win_name
                .as_mut_ptr()
                .offset(lstrlenW((*name).win_name.as_ptr()) as isize);
            *end = '-' as WCHAR;
            strcpyW(end.offset(1), (*name).country);
        }
    }

    EnumResourceLanguagesW(
        kernel32_handle,
        RT_STRING as LPCWSTR,
        LOCALE_ILANGUAGE as LPCWSTR,
        as_enumreslang_w(find_locale_id_callback),
        name as LONG_PTR,
    );
}

// ===========================================================================
// Dropped-by-translation symbols: Win32/kernel32 exports + internal helpers
// that the rustconv pass referenced but did not carry. Real bodies live in
// other (C) kernel32 modules; declared here as externs so locale.rs links.
// Signatures mirror sdk/include/psdk/winbase.h exactly.
// ===========================================================================

// winnls/string/locale.c:54 — #define NLSRC_OFFSET 5000  (also kernel32.rc:16)
pub const NLSRC_OFFSET: DWORD = 5000;

extern "system" {
    // winbase.h:1768 — HRSRC WINAPI FindResourceExW(HINSTANCE,LPCWSTR,LPCWSTR,WORD);
    pub fn FindResourceExW(hModule: HMODULE, lpType: LPCWSTR, lpName: LPCWSTR, wLanguage: WORD) -> HRSRC;
    // winbase.h:2441 — HGLOBAL WINAPI LoadResource(HINSTANCE,HRSRC);
    pub fn LoadResource(hModule: HMODULE, hResInfo: HRSRC) -> HGLOBAL;
    // winbase.h:2455 — PVOID WINAPI LockResource(HGLOBAL);
    pub fn LockResource(hResData: HGLOBAL) -> LPVOID;
    // winbase.h:1785 — BOOL WINAPI FreeResource(HGLOBAL);
    pub fn FreeResource(hResData: HGLOBAL) -> BOOL;
    // winbase.h:3220 — BOOL WINAPI WriteProfileStringW(LPCWSTR,LPCWSTR,LPCWSTR);
    pub fn WriteProfileStringW(lpAppName: LPCWSTR, lpKeyName: LPCWSTR, lpString: LPCWSTR) -> BOOL;
    // NOTE: HeapReAlloc is NOT declared here — like HeapAlloc/HeapFree it is a
    // pure ntdll forwarder in kernel32.def (no link-time @N symbol). Provided as a
    // Rust wrapper over RtlReAllocateHeap below.
    // NOTE: LCIDToLocaleName is NOT declared extern here — it is defined as a
    // #[no_mangle] Rust fn lower in this file (vista-gated; keeps the C vista
    // object and its unresolvable RtlLcidToLocaleName ref out of the module link).
}

// winnls/string/nls.c:1955 — UINT GetLocalisedText(UINT,LPWSTR,UINT,LANGID).
// Internal kernel32 helper; NOT WINAPI in nls.c → plain cdecl symbol
// (`_GetLocalisedText`, not `_GetLocalisedText@16`). Must live in an
// `extern "C"` block so i386 doesn't stdcall-decorate it.
extern "C" {
    pub fn GetLocalisedText(uID: UINT, lpszDest: LPWSTR, cchDest: UINT, lang: LANGID) -> UINT;
}

extern "C" {
    // winnls/string/japanese.c:38 — void JapaneseEra_ClearCache(void);  (cdecl, internal)
    pub fn JapaneseEra_ClearCache();
}

// winbase.h: #define ULongToPtr(ul) ((VOID*)(ULONG_PTR)(ULONG)ul)  (basetsd.h)
#[inline]
pub fn ULongToPtr(ul: u32) -> *mut c_void {
    ul as usize as *mut c_void
}

// ntdef.h: FORCEINLINE USHORT RtlUshortByteSwap(USHORT Source);  (byte-swap)
#[inline]
pub fn RtlUshortByteSwap(x: u16) -> u16 {
    x.swap_bytes()
}

// ===========================================================================
// NLS_GetGeoFriendlyName — static helper from winnls/string/locale.c:5440,
// dropped by the translation. Faithful hand-port (ReactOS 4-arg variant).
// ===========================================================================
#[inline]
pub unsafe fn NLS_GetGeoFriendlyName(
    location: GEOID,
    szFriendlyName: LPWSTR,
    cchData: ::core::ffi::c_int,
    lang: LANGID,
) -> ::core::ffi::c_int {
    // FIXME: move *.nls resources out of kernel32 into locale.nls
    let mut loc = location;
    loc = loc.wrapping_add(NLSRC_OFFSET);
    loc &= 0xFFFF;

    if cchData == 0 {
        return GetLocalisedText(loc, core::ptr::null_mut(), 0, lang) as ::core::ffi::c_int;
    }

    if GetLocalisedText(loc, szFriendlyName, cchData as UINT, lang) != 0 {
        return lstrlenW(szFriendlyName) + 1;
    }

    0
}

// ===========================================================================
// Link-symbol providers for the macros / Wine helpers that locale.rs (and this
// substrate) declare `extern` but that have no real C symbol after the C TU was
// retired. Each `#[no_mangle] extern "C"` body emits the `_Name` (cdecl) symbol
// the i386 linker needs; the existing extern decls remain the call-site type
// surface (a decl + a #[no_mangle] def with the same symbol resolve to one).
// ===========================================================================

// ---- winnt.h LCID/LANGID macros (exact semantics from psdk/winnt.h) ----
// #define MAKELCID(lgid, srtid) ((DWORD)((((DWORD)((USHORT)(srtid)))<<16)|((DWORD)((USHORT)(lgid)))))
#[no_mangle]
pub extern "C" fn MAKELCID(langid: LANGID, sortid: DWORD) -> LCID {
    (((sortid as u16 as u32) << 16) | (langid as u16 as u32)) as LCID
}
// #define SORTIDFROMLCID(lcid) ((USHORT)((((DWORD)(lcid)) >> 16) & 0xf))
#[no_mangle]
pub extern "C" fn SORTIDFROMLCID(lcid: LCID) -> DWORD {
    ((((lcid as u32) >> 16) & 0xf) as u16) as DWORD
}
// #define SUBLANGID(lgid) ((USHORT)(lgid) >> 10)
#[no_mangle]
pub extern "C" fn SUBLANGID(langid: LANGID) -> DWORD {
    ((langid as u16) >> 10) as DWORD
}

// ---- Wine debug macros: defs relocated to winetrace-shim (§9.99 — variadic
// call sites + fixed-arity in-crate defs = instcombine ud2 fold) ----
#[no_mangle]
pub extern "C" fn debugstr_w(_s: *const WCHAR) -> *const u8 {
    core::ptr::null()
}
#[no_mangle]
pub extern "C" fn debugstr_a(_s: *const i8) -> *const i8 {
    core::ptr::null()
}

// ---- Wine unicode.h "W" helpers (macro → libc mappings) ----
extern "C" {
    fn wcslen(s: *const WCHAR) -> usize;
    // _snwprintf(buffer, count, format, ...) — msvcrt wide formatted print.
    fn _snwprintf(buffer: *mut WCHAR, count: usize, format: *const WCHAR, ...) -> i32;
}
// #define strlenW(s) wcslen((s))
#[no_mangle]
pub unsafe extern "C" fn strlenW(s: LPCWSTR) -> INT {
    wcslen(s) as INT
}
// #define tolowerW(n) towlower((n))
#[no_mangle]
pub unsafe extern "C" fn tolowerW(ch: u16) -> u16 {
    towlower(ch as WCHAR) as u16
}
// #define sprintfW _swprintf  ==  _snwprintf(s, MAXLONG, fmt, ...). All call
// sites pass exactly one 4-byte integer vararg, so this fixed-arg cdecl body is
// ABI-compatible with the variadic call sites (caller cleans the stack).
#[no_mangle]
pub unsafe extern "C" fn sprintfW(buf: *mut WCHAR, format: *const WCHAR, arg: u32) -> i32 {
    // MAXLONG (0x7FFFFFFF) — effectively unbounded; the call-site buffers are
    // sized for the short integer renderings these formats produce.
    _snwprintf(buf, 0x7FFF_FFFF, format, arg)
}

// ---- RtlCompareUnicodeStrings link shim ----
// ntdll.spec gates RtlCompareUnicodeStrings as -version=0x600+, so it is absent
// from the ntdll importlib this build links against (only the singular
// RtlCompareUnicodeString@12, which takes UNICODE_STRINGs, is exported).
// CompareStringOrdinal (locale.rs) needs the raw-buffer plural form, so provide
// the symbol here, faithfully mirroring sdk/lib/rtl/unicode.c:2233 (upcase via
// the exported RtlUpcaseUnicodeChar@4 instead of the internal RtlpUpcase...).
#[no_mangle]
pub unsafe extern "system" fn RtlCompareUnicodeStrings(
    String1: *const WCHAR,
    String1Length: SIZE_T,
    String2: *const WCHAR,
    String2Length: SIZE_T,
    CaseInSensitive: BOOLEAN,
) -> LONG {
    let min_len = if String1Length < String2Length { String1Length } else { String2Length };
    let mut index: SIZE_T = 0;
    if CaseInSensitive != 0 {
        while index < min_len {
            let c1 = RtlUpcaseUnicodeChar(*String1.add(index as usize));
            let c2 = RtlUpcaseUnicodeChar(*String2.add(index as usize));
            let result = c1 as LONG - c2 as LONG;
            if result != 0 {
                return result;
            }
            index += 1;
        }
    } else {
        while index < min_len {
            let result = *String1.add(index as usize) as LONG - *String2.add(index as usize) as LONG;
            if result != 0 {
                return result;
            }
            index += 1;
        }
    }
    String1Length as LONG - String2Length as LONG
}

// ---- kernel32 heap APIs routed to ntdll (forwarder targets) ----
// HeapAlloc/HeapFree/HeapReAlloc are pure forwarders to ntdll in kernel32.def,
// so they have no link-time symbol inside the kernel32 module. Provide them as
// thin Rust wrappers over the real ntdll Rtl* heap exports (the same approach
// the proc.c cohab uses with RtlAllocateHeap/RtlGetProcessHeap). These keep the
// kernel32 HeapAlloc/HeapFree/HeapReAlloc call signatures so call sites are
// unchanged; they are plain (non-#[no_mangle]) helpers — not exports.
#[inline]
pub unsafe fn HeapAlloc(heap: HANDLE, flags: DWORD, size: usize) -> LPVOID {
    RtlAllocateHeap(heap as PVOID, flags as ULONG, size as SIZE_T) as LPVOID
}
#[inline]
pub unsafe fn HeapFree(heap: HANDLE, flags: DWORD, ptr: LPVOID) -> i32 {
    RtlFreeHeap(heap, flags as ULONG, ptr as PVOID) as i32
}
#[inline]
pub unsafe fn HeapReAlloc(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID, dwBytes: SIZE_T) -> LPVOID {
    RtlReAllocateHeap(hHeap, dwFlags as ULONG, lpMem as PVOID, dwBytes) as LPVOID
}

// ===========================================================================
// ReactOS-dead wine codepage helpers — link stubs.
// Every call site for these lives in a `#ifndef __REACTOS__` branch of locale.c
// (the wine ANSI/OEM/UTF8/DBCS conversion path), shadowed at runtime by nls.c's
// own GetCPInfo/MultiByteToWideChar/IsDBCSLeadByte exports (which win first-match
// at link). The branches are unreachable, but the translation emitted them, so
// provide the symbols. Return values mirror the "no table / nothing converted"
// outcome that makes the dead branch fall through harmlessly if ever taken.
// ===========================================================================
#[no_mangle]
pub extern "C" fn wine_cp_get_table(_codepage: UINT) -> *mut cptable {
    core::ptr::null_mut()
}
#[no_mangle]
pub extern "C" fn wine_cp_mbstowcs(
    _table: *const cptable, _flags: i32, _src: *const i8, _srclen: i32,
    _dst: *mut WCHAR, _dstlen: i32,
) -> i32 { 0 }
#[no_mangle]
pub extern "C" fn wine_cp_wcstombs(
    _table: *const cptable, _flags: i32, _src: *const WCHAR, _srclen: i32,
    _dst: *mut i8, _dstlen: i32, _defchar: *const i8, _used: *mut i32,
) -> i32 { 0 }
#[no_mangle]
pub extern "C" fn wine_cpsymbol_mbstowcs(_src: *const i8, _srclen: i32, _dst: *mut WCHAR, _dstlen: i32) -> i32 { 0 }
#[no_mangle]
pub extern "C" fn wine_cpsymbol_wcstombs(_src: *const WCHAR, _srclen: i32, _dst: *mut i8, _dstlen: i32) -> i32 { 0 }
#[no_mangle]
pub extern "C" fn wine_utf8_mbstowcs(_flags: i32, _src: *const i8, _srclen: i32, _dst: *mut WCHAR, _dstlen: i32) -> i32 { 0 }
#[no_mangle]
pub extern "C" fn wine_utf8_wcstombs(_flags: i32, _src: *const WCHAR, _srclen: i32, _dst: *mut i8, _dstlen: i32) -> i32 { 0 }
#[no_mangle]
pub extern "C" fn wine_is_dbcs_leadbyte(_table: *const cptable, _ch: u8) -> u8 { 0 }
#[no_mangle]
pub extern "C" fn __wine_init_codepages(
    _ansi_cp: *const cptable, _oem_cp: *const cptable, _unix_cp: *const cptable,
) {}
// setlocale — only called from the wine LOCALE_Init path (ReactOS-dead). No CRT
// `setlocale` is available in the kernel32 link, so stub it to NULL.
#[no_mangle]
pub extern "C" fn setlocale(_category: i32, _locale: *const i8) -> *mut i8 {
    core::ptr::null_mut()
}

// nameprep_char_type / nameprep_mapping — IDN/StringPrep tables (generated in
// wine). Referenced only from the `#ifndef __REACTOS__` IDN nameprep path in
// locale.c. Provide minimal symbols so the dead branch links; locale.rs's own
// extern declares them as zero-length arrays, so any non-empty array works.
#[no_mangle]
pub static nameprep_char_type: [u16; 1] = [0];
#[no_mangle]
pub static nameprep_mapping: [WCHAR; 1] = [0];

// LCIDToLocaleName is kernel32-vista-rust's (locale.rs), which kernel32.dll also
// links. A failing stub here was a duplicate symbol that only lost by link order;
// tools/link_guard.py now refuses those.

// ===========================================================================
// NLS substrate additions for nls.rs (ReactOS nls.c) + lcformat.rs (Wine
// lcformat.c). Types/constants mirror sdk/include/psdk/winnls.h + winnt.h
// exactly; internal structs come from dll/win32/kernel32/include/kernel32.h and
// winnls/string/{nls,lcformat,japanese}.{c,h}.
// ===========================================================================

// ---- winnls.h calendar id / type aliases (winnls.h:582-583) ----
pub type CALTYPE = DWORD;
pub type CALID = DWORD;

// ---- winnls.h NUMBERFMT / CURRENCYFMT (winnls.h:670-721) ----
#[repr(C)]
pub struct _numberfmtW {
    pub NumDigits: UINT,
    pub LeadingZero: UINT,
    pub Grouping: UINT,
    pub lpDecimalSep: LPWSTR,
    pub lpThousandSep: LPWSTR,
    pub NegativeOrder: UINT,
}
pub type NUMBERFMTW = _numberfmtW;
pub type LPNUMBERFMTW = *mut _numberfmtW;
#[repr(C)]
pub struct _numberfmtA {
    pub NumDigits: UINT,
    pub LeadingZero: UINT,
    pub Grouping: UINT,
    pub lpDecimalSep: LPSTR,
    pub lpThousandSep: LPSTR,
    pub NegativeOrder: UINT,
}
pub type NUMBERFMTA = _numberfmtA;
pub type LPNUMBERFMTA = *mut _numberfmtA;
#[repr(C)]
pub struct _currencyfmtW {
    pub NumDigits: UINT,
    pub LeadingZero: UINT,
    pub Grouping: UINT,
    pub lpDecimalSep: LPWSTR,
    pub lpThousandSep: LPWSTR,
    pub NegativeOrder: UINT,
    pub PositiveOrder: UINT,
    pub lpCurrencySymbol: LPWSTR,
}
pub type CURRENCYFMTW = _currencyfmtW;
pub type LPCURRENCYFMTW = *mut _currencyfmtW;
#[repr(C)]
pub struct _currencyfmtA {
    pub NumDigits: UINT,
    pub LeadingZero: UINT,
    pub Grouping: UINT,
    pub lpDecimalSep: LPSTR,
    pub lpThousandSep: LPSTR,
    pub NegativeOrder: UINT,
    pub PositiveOrder: UINT,
    pub lpCurrencySymbol: LPSTR,
}
pub type CURRENCYFMTA = _currencyfmtA;
pub type LPCURRENCYFMTA = *mut _currencyfmtA;

// ---- winnls.h NLSVERSIONINFOEX (winnls.h:699-705) ----
#[repr(C)]
pub struct _nlsversioninfoex {
    pub dwNLSVersionInfoSize: DWORD,
    pub dwNLSVersion: DWORD,
    pub dwDefinedVersion: DWORD,
    pub dwEffectiveId: DWORD,
    pub guidCustomVersion: GUID,
}
pub type NLSVERSIONINFOEX = _nlsversioninfoex;
pub type LPNLSVERSIONINFOEX = *mut _nlsversioninfoex;

// ---- winnls.h NLS_FUNCTION (winnls.h:614-617): enum {COMPARE_STRING=1} ----
pub type NLS_FUNCTION = DWORD;
pub const COMPARE_STRING: NLS_FUNCTION = 0x0001;

// ---- winnls.h enumeration callback typedefs (winnls.h:588-611).
// CALLBACK == __stdcall on x86 == extern "system". ----
pub type DATEFMT_ENUMPROCW =
    Option<unsafe extern "system" fn(LPWSTR) -> BOOL>;
pub type DATEFMT_ENUMPROCA =
    Option<unsafe extern "system" fn(LPSTR) -> BOOL>;
pub type DATEFMT_ENUMPROCEXW =
    Option<unsafe extern "system" fn(LPWSTR, CALID) -> BOOL>;
pub type DATEFMT_ENUMPROCEXA =
    Option<unsafe extern "system" fn(LPSTR, CALID) -> BOOL>;
pub type DATEFMT_ENUMPROCEXEX =
    Option<unsafe extern "system" fn(LPWSTR, CALID, LPARAM) -> BOOL>;
pub type TIMEFMT_ENUMPROCW =
    Option<unsafe extern "system" fn(LPWSTR) -> BOOL>;
pub type TIMEFMT_ENUMPROCA =
    Option<unsafe extern "system" fn(LPSTR) -> BOOL>;
pub type TIMEFMT_ENUMPROCEX =
    Option<unsafe extern "system" fn(LPWSTR, LPARAM) -> BOOL>;
pub type CALINFO_ENUMPROCW =
    Option<unsafe extern "system" fn(LPWSTR) -> BOOL>;
pub type CALINFO_ENUMPROCA =
    Option<unsafe extern "system" fn(LPSTR) -> BOOL>;
pub type CALINFO_ENUMPROCEXW =
    Option<unsafe extern "system" fn(LPWSTR, CALID) -> BOOL>;
pub type CALINFO_ENUMPROCEXA =
    Option<unsafe extern "system" fn(LPSTR, CALID) -> BOOL>;
pub type CALINFO_ENUMPROCEXEX =
    Option<unsafe extern "system" fn(LPWSTR, CALID, LPWSTR, LPARAM) -> BOOL>;

// ---- internal CODEPAGE_ENTRY (kernel32.h:59-66) ----
#[repr(C)]
pub struct _CODEPAGE_ENTRY {
    pub Entry: LIST_ENTRY,
    pub CodePage: UINT,
    pub SectionHandle: HANDLE,
    pub SectionMapping: PBYTE,
    pub CodePageTable: CPTABLEINFO,
}
pub type CODEPAGE_ENTRY = _CODEPAGE_ENTRY;
pub type PCODEPAGE_ENTRY = *mut _CODEPAGE_ENTRY;

// ---- japanese.h JAPANESE_ERA (winnls/string/japanese.h:9-19). japanese.c is now
// Rust (japanese.rs); JapaneseEra_* are #[no_mangle] there. Copy/Clone so the
// s_JapaneseEraTable `[JAPANESE_ERA {..}; N]` static initializer type-checks. ----
#[repr(C)]
#[derive(Copy, Clone)]
pub struct JAPANESE_ERA {
    pub wYear: WORD,
    pub wMonth: WORD,
    pub wDay: WORD,
    pub szEraName: [WCHAR; 16],
    pub szEraAbbrev: [WCHAR; 5],
    pub szEnglishEraName: [WCHAR; 24],
    pub szEnglishEraAbbrev: [WCHAR; 5],
}
pub type PJAPANESE_ERA = *mut JAPANESE_ERA;
pub type PCJAPANESE_ERA = *const JAPANESE_ERA;

// ---- winnls.h constants (exact header values) ----
pub const LOCALE_SMONTHDAY: DWORD = 0x0078; // winnls.h:169
pub const ENUM_ALL_CALENDARS: DWORD = 0xFFFFFFFF; // winnls.h:208 = (-1)
pub const DATE_SHORTDATE: DWORD = 1; // winnls.h:209
pub const DATE_LONGDATE: DWORD = 2; // winnls.h:210
pub const DATE_USE_ALT_CALENDAR: DWORD = 4; // winnls.h:211
pub const DATE_YEARMONTH: DWORD = 8; // winnls.h:530
pub const DATE_LTRREADING: DWORD = 16; // winnls.h:531
pub const DATE_RTLREADING: DWORD = 32; // winnls.h:532
pub const TIME_NOMINUTESORSECONDS: DWORD = 1; // winnls.h:295
pub const TIME_NOSECONDS: DWORD = 2; // winnls.h:296
pub const TIME_NOTIMEMARKER: DWORD = 4; // winnls.h:297
pub const TIME_FORCE24HOURFORMAT: DWORD = 8; // winnls.h:298
pub const CAL_JAPAN: DWORD = 3; // winnls.h:487
// winnls.h:534-536 alias to LOCALE_* (already defined above in this crate).
pub const CAL_NOUSEROVERRIDE: DWORD = LOCALE_NOUSEROVERRIDE; // 0x80000000
pub const CAL_RETURN_NUMBER: DWORD = LOCALE_RETURN_NUMBER; // 0x20000000
pub const CAL_USE_CP_ACP: DWORD = LOCALE_USE_CP_ACP; // 0x40000000
pub const JAPANESE_MAX_TWODIGITYEAR: INT = 99; // japanese.h:7

// ---- winnt.h / win32 helper macros as const fns (glob-importable) ----
// #define MAKEWORD(a,b) ((WORD)(((BYTE)(a))|(((WORD)((BYTE)(b)))<<8)))
#[inline]
pub const fn MAKEWORD(a: UCHAR, b: UCHAR) -> WORD {
    (a as WORD) | ((b as WORD) << 8)
}
// #define NT_SUCCESS(Status) ((NTSTATUS)(Status) >= 0)
#[inline]
pub fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}
// #define MAKEINTRESOURCEW(i) ((LPWSTR)((ULONG_PTR)((WORD)(i))))
#[inline]
pub fn MAKEINTRESOURCEW(i: u16) -> LPWSTR {
    i as usize as LPWSTR
}
// base.h:26 — NLS_SIZEOF_ACE_AND_SIDS(n):
//   FIELD_OFFSET(ACCESS_ALLOWED_ACE, SidStart) +
//   FIELD_OFFSET(SID, SubAuthority) + (n)*RTL_FIELD_SIZE(SID, SubAuthority)
// SubAuthority is a [DWORD;1] → RTL_FIELD_SIZE == 4. const fn for array-size use.
#[inline]
pub const fn NLS_SIZEOF_ACE_AND_SIDS(n: usize) -> usize {
    core::mem::offset_of!(ACCESS_ALLOWED_ACE, SidStart)
        + core::mem::offset_of!(SID, SubAuthority)
        + n * 4
}

// ─── ntstrsafe RtlStringCch/CbCopyW (FORCEINLINE in ntstrsafe.h — no real export;
// lcformat.rs declares them extern "C", so provide #[no_mangle] cdecl bodies). ───
#[no_mangle]
pub unsafe extern "C" fn RtlStringCchCopyW(dest: LPWSTR, cch_dest: usize, src: LPCWSTR) -> NTSTATUS {
    if cch_dest == 0 {
        return 0xC000000Du32 as NTSTATUS; // STATUS_INVALID_PARAMETER
    }
    let mut i: usize = 0;
    while i + 1 < cch_dest && *src.add(i) != 0 {
        *dest.add(i) = *src.add(i);
        i += 1;
    }
    *dest.add(i) = 0;
    if *src.add(i) != 0 {
        0x80000005u32 as NTSTATUS // STATUS_BUFFER_OVERFLOW
    } else {
        0 // STATUS_SUCCESS
    }
}
#[no_mangle]
pub unsafe extern "C" fn RtlStringCbCopyW(dest: LPWSTR, cb_dest: usize, src: LPCWSTR) -> NTSTATUS {
    RtlStringCchCopyW(dest, cb_dest / core::mem::size_of::<WCHAR>(), src)
}

// ─── RtlLocaleNameToLcid: ntdll spec is -version=0x600+ → NOT exported by this build's
// ntdll.dll. kernel32_vista/LocaleNameToLCID.c (pulled because lcformat.rs calls
// LocaleNameToLCID) references it. Shim via parse_locale_name (already in this module) so
// the vista LocaleNameToLCID wrapper links without the version-gated ntdll import. ───
#[no_mangle]
pub unsafe extern "system" fn RtlLocaleNameToLcid(
    name: *const WCHAR,
    lcid_out: *mut LCID,
    _flags: ULONG,
) -> NTSTATUS {
    if name.is_null() || lcid_out.is_null() {
        return 0xC000000Du32 as NTSTATUS; // STATUS_INVALID_PARAMETER
    }
    let mut ln: locale_name = core::mem::zeroed();
    parse_locale_name(name, &mut ln);
    if ln.matches == 0 {
        return 0xC000000Du32 as NTSTATUS; // not found → STATUS_INVALID_PARAMETER
    }
    *lcid_out = ln.lcid as LCID;
    0 // STATUS_SUCCESS
}

// kernel32_vista/LocaleNameToLCID.c references RtlLocaleNameToLcid via __declspec(dllimport)
// — i.e. through the import thunk `__imp__RtlLocaleNameToLcid@12`, not the direct symbol.
// ntdll doesn't export it (version-gated), so provide the import slot as a data symbol
// holding the address of our in-crate shim above. (i386 only; the dllimport indirection +
// @12 stdcall decoration are i386 PE specifics.)
#[cfg(target_arch = "x86")]
#[used]
#[export_name = "_imp__RtlLocaleNameToLcid@12"]
pub static IMP_RtlLocaleNameToLcid: unsafe extern "system" fn(*const WCHAR, *mut LCID, ULONG) -> NTSTATUS =
    RtlLocaleNameToLcid;
// amd64/arm64 PE: import thunk is `__imp_RtlLocaleNameToLcid` (no `@N`, no extra leading
// underscore — these arches don't decorate symbols the way i386 does).
#[cfg(not(target_arch = "x86"))]
#[used]
#[export_name = "__imp_RtlLocaleNameToLcid"]
pub static IMP_RtlLocaleNameToLcid: unsafe extern "system" fn(*const WCHAR, *mut LCID, ULONG) -> NTSTATUS =
    RtlLocaleNameToLcid;
