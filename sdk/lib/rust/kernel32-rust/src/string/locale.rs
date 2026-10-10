use winedebug_rust::{TRACE, WARN};
use ntdll_sys::nt::*;
use core::ffi::c_void;
// NLS substrate: winnls.h types/constants + Wine unicode helpers not in ntdll-sys.
use crate::string::nls_subst::*;

// File-local struct from locale.c (typedef struct tagWCHAR_PAIR { WCHAR from, to; }).
// The c2rust preamble emits `struct WCHAR_PAIR { from, to }`; the mapping tables use
// the C tag name, so alias it (one type, no dup).
pub type tagWCHAR_PAIR = WCHAR_PAIR;
pub type PWCHAR_PAIR = *mut WCHAR_PAIR;

// Disambiguate: nls_subst provides the real RtlCompareUnicodeStrings symbol
// (the ntdll importlib gates it as version 0x600+ and omits it from this build),
// so prefer the substrate shim over the bindgen extern decl glob-imported above.
use crate::string::nls_subst::RtlCompareUnicodeStrings;

extern "C" {
    // These local *variadic* decls shadow the non-variadic #[no_mangle] bodies that
    // nls_subst.rs glob-imports above; the call sites pass printf-style varargs, the
    // resolved symbol is still the nls_subst body (which ignores the extra args).
    fn _snwprintf( buffer: *mut WCHAR, count: usize, format: *const WCHAR, ... ) -> i32;
    fn sprintfW(buf: *mut WCHAR, format: *const WCHAR, ...) -> i32;
    fn FIXME(fmt: *const i8, ...);
    fn wcslen(s: *const WCHAR) -> usize;
    fn MAKELCID(langid: LANGID, sortid: DWORD) -> LCID;
    fn SORTIDFROMLCID(lcid: LCID) -> DWORD;
    fn SUBLANGID(langid: LANGID) -> DWORD;
    fn strlen(s: *const i8) -> usize;
    fn strlenW(s: LPCWSTR) -> INT;
    fn tolowerW(ch: u16) -> u16;
    fn debugstr_w(s: *const WCHAR) -> *const u8;
    fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8;
    // Internal cdecl C function (winnls/string/japanese.c — `void JapaneseEra_ClearCache(void)`).
    fn JapaneseEra_ClearCache();
}
extern "system" {
    // GetProcessHeap is a real kernel32 stdcall export (@0). HeapAlloc/HeapFree are
    // pure forwarders to ntdll in kernel32.def (no @N symbol in this module), so they
    // come from the nls_subst.rs Rust wrappers over RtlAllocateHeap/RtlFreeHeap
    // (glob-imported above) — do NOT declare them here.
    fn GetProcessHeap() -> HANDLE;
    fn EnumResourceLanguagesA( hModule: HMODULE, lpType: LPCSTR, lpName: LPCSTR, lpEnumFunc: ENUMRESLANGPROCA, lParam: LONG_PTR, ) -> BOOL;
    fn EnumResourceLanguagesW( hModule: HMODULE, lpType: LPCWSTR, lpName: LPCWSTR, lpEnumFunc: ENUMRESLANGPROCW, lParam: LONG_PTR, ) -> BOOL;
    fn SetLastError(dwErrCode: u32);
    // ---- LIVE codepage exports that locale.c only had under the dead `#ifndef
    //      __REACTOS__` Wine branch. The real implementations stay C and live in
    //      OTHER kernel32 .c files (nls.c). Bind callers to those live symbols
    //      (stdcall @N on i386, C on amd64/arm64).
    fn GetACP() -> UINT;
    fn MultiByteToWideChar(
        page: u32,
        flags: u32,
        src: *const i8,
        srclen: i32,
        dst: *mut u16,
        dstlen: i32,
    ) -> i32;
    fn WideCharToMultiByte(
        page: u32,
        flags: u32,
        src: *const u16,
        srclen: i32,
        dst: *mut i8,
        dstlen: i32,
        defchar: *const i8,
        used: LPBOOL,
    ) -> i32;
}

// ===========================================================================
// File-scope type definitions (the c2rust preamble omits these struct/type/const
// declarations; they were emitted only inside the now-stripped dead-code region in
// the previous integration). Ported verbatim from the prior locale.rs.
// ===========================================================================
pub type union_cptable = *mut cptable;

#[repr(C)]
pub struct charset_entry {
    pub charset_name: *const i8,
    pub codepage: u32,
}
// Immutable read-only static data; the *const i8 names are 'static literals.
unsafe impl Sync for charset_entry {}

pub static charset_names: [charset_entry; 52] = [
    charset_entry { charset_name: b"BIG5\0".as_ptr() as *const i8, codepage: 950 },
    charset_entry { charset_name: b"CP1250\0".as_ptr() as *const i8, codepage: 1250 },
    charset_entry { charset_name: b"CP1251\0".as_ptr() as *const i8, codepage: 1251 },
    charset_entry { charset_name: b"CP1252\0".as_ptr() as *const i8, codepage: 1252 },
    charset_entry { charset_name: b"CP1253\0".as_ptr() as *const i8, codepage: 1253 },
    charset_entry { charset_name: b"CP1254\0".as_ptr() as *const i8, codepage: 1254 },
    charset_entry { charset_name: b"CP1255\0".as_ptr() as *const i8, codepage: 1255 },
    charset_entry { charset_name: b"CP1256\0".as_ptr() as *const i8, codepage: 1256 },
    charset_entry { charset_name: b"CP1257\0".as_ptr() as *const i8, codepage: 1257 },
    charset_entry { charset_name: b"CP1258\0".as_ptr() as *const i8, codepage: 1258 },
    charset_entry { charset_name: b"CP932\0".as_ptr() as *const i8, codepage: 932 },
    charset_entry { charset_name: b"CP936\0".as_ptr() as *const i8, codepage: 936 },
    charset_entry { charset_name: b"CP949\0".as_ptr() as *const i8, codepage: 949 },
    charset_entry { charset_name: b"CP950\0".as_ptr() as *const i8, codepage: 950 },
    charset_entry { charset_name: b"EUCJP\0".as_ptr() as *const i8, codepage: 20932 },
    charset_entry { charset_name: b"GB2312\0".as_ptr() as *const i8, codepage: 936 },
    charset_entry { charset_name: b"IBM037\0".as_ptr() as *const i8, codepage: 37 },
    charset_entry { charset_name: b"IBM1026\0".as_ptr() as *const i8, codepage: 1026 },
    charset_entry { charset_name: b"IBM424\0".as_ptr() as *const i8, codepage: 424 },
    charset_entry { charset_name: b"IBM437\0".as_ptr() as *const i8, codepage: 437 },
    charset_entry { charset_name: b"IBM500\0".as_ptr() as *const i8, codepage: 500 },
    charset_entry { charset_name: b"IBM850\0".as_ptr() as *const i8, codepage: 850 },
    charset_entry { charset_name: b"IBM852\0".as_ptr() as *const i8, codepage: 852 },
    charset_entry { charset_name: b"IBM855\0".as_ptr() as *const i8, codepage: 855 },
    charset_entry { charset_name: b"IBM857\0".as_ptr() as *const i8, codepage: 857 },
    charset_entry { charset_name: b"IBM860\0".as_ptr() as *const i8, codepage: 860 },
    charset_entry { charset_name: b"IBM861\0".as_ptr() as *const i8, codepage: 861 },
    charset_entry { charset_name: b"IBM862\0".as_ptr() as *const i8, codepage: 862 },
    charset_entry { charset_name: b"IBM863\0".as_ptr() as *const i8, codepage: 863 },
    charset_entry { charset_name: b"IBM864\0".as_ptr() as *const i8, codepage: 864 },
    charset_entry { charset_name: b"IBM865\0".as_ptr() as *const i8, codepage: 865 },
    charset_entry { charset_name: b"IBM866\0".as_ptr() as *const i8, codepage: 866 },
    charset_entry { charset_name: b"IBM869\0".as_ptr() as *const i8, codepage: 869 },
    charset_entry { charset_name: b"IBM874\0".as_ptr() as *const i8, codepage: 874 },
    charset_entry { charset_name: b"IBM875\0".as_ptr() as *const i8, codepage: 875 },
    charset_entry { charset_name: b"ISO88591\0".as_ptr() as *const i8, codepage: 28591 },
    charset_entry { charset_name: b"ISO885910\0".as_ptr() as *const i8, codepage: 28600 },
    charset_entry { charset_name: b"ISO885913\0".as_ptr() as *const i8, codepage: 28603 },
    charset_entry { charset_name: b"ISO885914\0".as_ptr() as *const i8, codepage: 28604 },
    charset_entry { charset_name: b"ISO885915\0".as_ptr() as *const i8, codepage: 28605 },
    charset_entry { charset_name: b"ISO885916\0".as_ptr() as *const i8, codepage: 28606 },
    charset_entry { charset_name: b"ISO88592\0".as_ptr() as *const i8, codepage: 28592 },
    charset_entry { charset_name: b"ISO88593\0".as_ptr() as *const i8, codepage: 28593 },
    charset_entry { charset_name: b"ISO88594\0".as_ptr() as *const i8, codepage: 28594 },
    charset_entry { charset_name: b"ISO88595\0".as_ptr() as *const i8, codepage: 28595 },
    charset_entry { charset_name: b"ISO88596\0".as_ptr() as *const i8, codepage: 28596 },
    charset_entry { charset_name: b"ISO88597\0".as_ptr() as *const i8, codepage: 28597 },
    charset_entry { charset_name: b"ISO88598\0".as_ptr() as *const i8, codepage: 28598 },
    charset_entry { charset_name: b"ISO88599\0".as_ptr() as *const i8, codepage: 28599 },
    charset_entry { charset_name: b"KOI8R\0".as_ptr() as *const i8, codepage: 20866 },
    charset_entry { charset_name: b"KOI8U\0".as_ptr() as *const i8, codepage: 21866 },
    charset_entry { charset_name: b"UTF8\0".as_ptr() as *const i8, codepage: 65001 },
];

#[repr(C)]
pub struct locale_name {
    pub win_name: [u16; 128],
    pub lang: [u16; 128],
    pub country: *mut u16,
    pub charset: *mut u16,
    pub script: *mut u16,
    pub modifier: *mut u16,
    pub lcid: u32,
    pub matches: i32,
    pub codepage: u32,
}

#[repr(C)]
pub struct registry_value {
    pub lctype: u32,
    pub name: *const u16,
    pub cached_value: *mut u16,
}

pub const LOCATION_NATION: i32 = 0;
pub const LOCATION_REGION: i32 = 1;
pub const LOCATION_BOTH: i32 = 2;

#[repr(C)]
pub struct geoinfo_t {
    pub id: u32,
    pub iso2W: [u16; 3],
    pub iso3W: [u16; 4],
    pub parent: u32,
    pub uncode: i32,
    pub kind: i32,
}

#[repr(C)]
pub struct ENUMSYSTEMCODEPAGES_CALLBACKS {
    pub procA: CODEPAGE_ENUMPROCA,
    pub procW: CODEPAGE_ENUMPROCW,
    pub dwFlags: u32,
}

pub type FULL2HALF_ENTRY = [u16; 3];
pub type DAKUTEN_ENTRY = [u16; 3];

#[repr(C)]
pub struct WCHAR_PAIR {
    pub from: u16,
    pub to: u16,
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut gDebugChannel: ULONG = 207 as ULONG;

static mut szLocaleKeyName: [WCHAR; 62] = [
    '\\' as i32 as WCHAR,
    'R' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'M' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'h' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    't' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'N' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    's' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'L' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut szLangGroupsKeyName: [WCHAR; 71] = [
    '\\' as i32 as WCHAR,
    'R' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'M' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'h' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    't' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'N' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    's' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'L' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    ' ' as i32 as WCHAR,
    'G' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut lcid_LC_COLLATE: LCID = 0;

static mut lcid_LC_CTYPE: LCID = 0;

static mut lcid_LC_MONETARY: LCID = 0;

static mut lcid_LC_NUMERIC: LCID = 0;

static mut lcid_LC_TIME: LCID = 0;

static mut lcid_LC_PAPER: LCID = 0;

static mut lcid_LC_MEASUREMENT: LCID = 0;

static mut lcid_LC_TELEPHONE: LCID = 0;

static mut iCalendarTypeW: [WCHAR; 14] = [
    'i' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'T' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut iCountryW: [WCHAR; 9] = [
    'i' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut iCurrDigitsW: [WCHAR; 12] = [
    'i' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    't' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut iCurrencyW: [WCHAR; 10] = [
    'i' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut iDateW: [WCHAR; 6] = [
    'i' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut iDigitsW: [WCHAR; 8] = [
    'i' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    't' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut iFirstDayOfWeekW: [WCHAR; 16] = [
    'i' as i32 as WCHAR,
    'F' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    'O' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    'W' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'k' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut iFirstWeekOfYearW: [WCHAR; 17] = [
    'i' as i32 as WCHAR,
    'F' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'W' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'k' as i32 as WCHAR,
    'O' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    'Y' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut iLDateW: [WCHAR; 7] = [
    'i' as i32 as WCHAR,
    'L' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut iLZeroW: [WCHAR; 7] = [
    'i' as i32 as WCHAR,
    'L' as i32 as WCHAR,
    'Z' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut iMeasureW: [WCHAR; 9] = [
    'i' as i32 as WCHAR,
    'M' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut iNegCurrW: [WCHAR; 9] = [
    'i' as i32 as WCHAR,
    'N' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut iNegNumberW: [WCHAR; 11] = [
    'i' as i32 as WCHAR,
    'N' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'N' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'b' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut iPaperSizeW: [WCHAR; 11] = [
    'i' as i32 as WCHAR,
    'P' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'z' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut iTLZeroW: [WCHAR; 8] = [
    'i' as i32 as WCHAR,
    'T' as i32 as WCHAR,
    'L' as i32 as WCHAR,
    'Z' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut iTimePrefixW: [WCHAR; 12] = [
    'i' as i32 as WCHAR,
    'T' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'P' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'x' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut iTimeW: [WCHAR; 6] = [
    'i' as i32 as WCHAR,
    'T' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut s1159W: [WCHAR; 6] = [
    's' as i32 as WCHAR,
    '1' as i32 as WCHAR,
    '1' as i32 as WCHAR,
    '5' as i32 as WCHAR,
    '9' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut s2359W: [WCHAR; 6] = [
    's' as i32 as WCHAR,
    '2' as i32 as WCHAR,
    '3' as i32 as WCHAR,
    '5' as i32 as WCHAR,
    '9' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut sCountryW: [WCHAR; 9] = [
    's' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut sCurrencyW: [WCHAR; 10] = [
    's' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut sDateW: [WCHAR; 6] = [
    's' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut sDecimalW: [WCHAR; 9] = [
    's' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut sGroupingW: [WCHAR; 10] = [
    's' as i32 as WCHAR,
    'G' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut sLanguageW: [WCHAR; 10] = [
    's' as i32 as WCHAR,
    'L' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut sListW: [WCHAR; 6] = [
    's' as i32 as WCHAR,
    'L' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut sLongDateW: [WCHAR; 10] = [
    's' as i32 as WCHAR,
    'L' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut sMonDecimalSepW: [WCHAR; 15] = [
    's' as i32 as WCHAR,
    'M' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut sMonGroupingW: [WCHAR; 13] = [
    's' as i32 as WCHAR,
    'M' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'G' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut sMonThousandSepW: [WCHAR; 16] = [
    's' as i32 as WCHAR,
    'M' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'T' as i32 as WCHAR,
    'h' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut sNativeDigitsW: [WCHAR; 14] = [
    's' as i32 as WCHAR,
    'N' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    't' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut sNegativeSignW: [WCHAR; 14] = [
    's' as i32 as WCHAR,
    'N' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut sPositiveSignW: [WCHAR; 14] = [
    's' as i32 as WCHAR,
    'P' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut sShortDateW: [WCHAR; 11] = [
    's' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'h' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut sThousandW: [WCHAR; 10] = [
    's' as i32 as WCHAR,
    'T' as i32 as WCHAR,
    'h' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut sTimeFormatW: [WCHAR; 12] = [
    's' as i32 as WCHAR,
    'T' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'F' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    't' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut sTimeW: [WCHAR; 6] = [
    's' as i32 as WCHAR,
    'T' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut sYearMonthW: [WCHAR; 11] = [
    's' as i32 as WCHAR,
    'Y' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'M' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'h' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut NumShapeW: [WCHAR; 9] = [
    'N' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'h' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut registry_values: [registry_value; 39] = unsafe {
    [
        registry_value {
            lctype: LOCALE_ICALENDARTYPE as DWORD,
            name: &raw const iCalendarTypeW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_ICURRDIGITS as DWORD,
            name: &raw const iCurrDigitsW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_ICURRENCY as DWORD,
            name: &raw const iCurrencyW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_IDIGITS as DWORD,
            name: &raw const iDigitsW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_IFIRSTDAYOFWEEK as DWORD,
            name: &raw const iFirstDayOfWeekW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_IFIRSTWEEKOFYEAR as DWORD,
            name: &raw const iFirstWeekOfYearW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_ILZERO as DWORD,
            name: &raw const iLZeroW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_IMEASURE as DWORD,
            name: &raw const iMeasureW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_INEGCURR as DWORD,
            name: &raw const iNegCurrW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_INEGNUMBER as DWORD,
            name: &raw const iNegNumberW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_IPAPERSIZE as DWORD,
            name: &raw const iPaperSizeW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_ITIME as DWORD,
            name: &raw const iTimeW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_S1159 as DWORD,
            name: &raw const s1159W as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_S2359 as DWORD,
            name: &raw const s2359W as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_SCURRENCY as DWORD,
            name: &raw const sCurrencyW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_SDATE as DWORD,
            name: &raw const sDateW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_SDECIMAL as DWORD,
            name: &raw const sDecimalW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_SGROUPING as DWORD,
            name: &raw const sGroupingW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_SLIST as DWORD,
            name: &raw const sListW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_SLONGDATE as DWORD,
            name: &raw const sLongDateW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_SMONDECIMALSEP as DWORD,
            name: &raw const sMonDecimalSepW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_SMONGROUPING as DWORD,
            name: &raw const sMonGroupingW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_SMONTHOUSANDSEP as DWORD,
            name: &raw const sMonThousandSepW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_SNEGATIVESIGN as DWORD,
            name: &raw const sNegativeSignW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_SPOSITIVESIGN as DWORD,
            name: &raw const sPositiveSignW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_SSHORTDATE as DWORD,
            name: &raw const sShortDateW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_STHOUSAND as DWORD,
            name: &raw const sThousandW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_STIME as DWORD,
            name: &raw const sTimeW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_STIMEFORMAT as DWORD,
            name: &raw const sTimeFormatW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_SYEARMONTH as DWORD,
            name: &raw const sYearMonthW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_ICOUNTRY as DWORD,
            name: &raw const iCountryW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_IDATE as DWORD,
            name: &raw const iDateW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_ILDATE as DWORD,
            name: &raw const iLDateW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_ITLZERO as DWORD,
            name: &raw const iTLZeroW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_SCOUNTRY as DWORD,
            name: &raw const sCountryW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_SABBREVLANGNAME as DWORD,
            name: &raw const sLanguageW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_IDIGITSUBSTITUTION as DWORD,
            name: &raw const NumShapeW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_SNATIVEDIGITS as DWORD,
            name: &raw const sNativeDigitsW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        registry_value {
            lctype: LOCALE_ITIMEMARKPOSN as DWORD,
            name: &raw const iTimePrefixW as *const WCHAR,
            cached_value: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
    ]
};

static mut cache_section: RTL_CRITICAL_SECTION = _RTL_CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: -(1 as ::core::ffi::c_int) as LONG,
    RecursionCount: 0 as LONG,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0 as ULONG_PTR,
};

static mut full2half_table: [FULL2HALF_ENTRY; 94] = [
    [
        0x3000 as ::core::ffi::c_int as WCHAR,
        0x20 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x3001 as ::core::ffi::c_int as WCHAR,
        0xff64 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x3002 as ::core::ffi::c_int as WCHAR,
        0xff61 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x300c as ::core::ffi::c_int as WCHAR,
        0xff62 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x300d as ::core::ffi::c_int as WCHAR,
        0xff63 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x309b as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x309c as ::core::ffi::c_int as WCHAR,
        0xff9f as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30a1 as ::core::ffi::c_int as WCHAR,
        0xff67 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30a2 as ::core::ffi::c_int as WCHAR,
        0xff71 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30a3 as ::core::ffi::c_int as WCHAR,
        0xff68 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30a4 as ::core::ffi::c_int as WCHAR,
        0xff72 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30a5 as ::core::ffi::c_int as WCHAR,
        0xff69 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30a6 as ::core::ffi::c_int as WCHAR,
        0xff73 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30a7 as ::core::ffi::c_int as WCHAR,
        0xff6a as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30a8 as ::core::ffi::c_int as WCHAR,
        0xff74 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30a9 as ::core::ffi::c_int as WCHAR,
        0xff6b as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30aa as ::core::ffi::c_int as WCHAR,
        0xff75 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30ab as ::core::ffi::c_int as WCHAR,
        0xff76 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30ac as ::core::ffi::c_int as WCHAR,
        0xff76 as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30ad as ::core::ffi::c_int as WCHAR,
        0xff77 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30ae as ::core::ffi::c_int as WCHAR,
        0xff77 as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30af as ::core::ffi::c_int as WCHAR,
        0xff78 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30b0 as ::core::ffi::c_int as WCHAR,
        0xff78 as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30b1 as ::core::ffi::c_int as WCHAR,
        0xff79 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30b2 as ::core::ffi::c_int as WCHAR,
        0xff79 as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30b3 as ::core::ffi::c_int as WCHAR,
        0xff7a as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30b4 as ::core::ffi::c_int as WCHAR,
        0xff7a as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30b5 as ::core::ffi::c_int as WCHAR,
        0xff7b as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30b6 as ::core::ffi::c_int as WCHAR,
        0xff7b as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30b7 as ::core::ffi::c_int as WCHAR,
        0xff7c as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30b8 as ::core::ffi::c_int as WCHAR,
        0xff7c as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30b9 as ::core::ffi::c_int as WCHAR,
        0xff7d as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30ba as ::core::ffi::c_int as WCHAR,
        0xff7d as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30bb as ::core::ffi::c_int as WCHAR,
        0xff7e as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30bc as ::core::ffi::c_int as WCHAR,
        0xff7e as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30bd as ::core::ffi::c_int as WCHAR,
        0xff7f as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30be as ::core::ffi::c_int as WCHAR,
        0xff7f as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30bf as ::core::ffi::c_int as WCHAR,
        0xff80 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30c0 as ::core::ffi::c_int as WCHAR,
        0xff80 as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30c1 as ::core::ffi::c_int as WCHAR,
        0xff81 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30c2 as ::core::ffi::c_int as WCHAR,
        0xff81 as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30c3 as ::core::ffi::c_int as WCHAR,
        0xff6f as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30c4 as ::core::ffi::c_int as WCHAR,
        0xff82 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30c5 as ::core::ffi::c_int as WCHAR,
        0xff82 as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30c6 as ::core::ffi::c_int as WCHAR,
        0xff83 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30c7 as ::core::ffi::c_int as WCHAR,
        0xff83 as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30c8 as ::core::ffi::c_int as WCHAR,
        0xff84 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30c9 as ::core::ffi::c_int as WCHAR,
        0xff84 as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30ca as ::core::ffi::c_int as WCHAR,
        0xff85 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30cb as ::core::ffi::c_int as WCHAR,
        0xff86 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30cc as ::core::ffi::c_int as WCHAR,
        0xff87 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30cd as ::core::ffi::c_int as WCHAR,
        0xff88 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30ce as ::core::ffi::c_int as WCHAR,
        0xff89 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30cf as ::core::ffi::c_int as WCHAR,
        0xff8a as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30d0 as ::core::ffi::c_int as WCHAR,
        0xff8a as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30d1 as ::core::ffi::c_int as WCHAR,
        0xff8a as ::core::ffi::c_int as WCHAR,
        0x309c as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30d1 as ::core::ffi::c_int as WCHAR,
        0xff8a as ::core::ffi::c_int as WCHAR,
        0xff9f as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30d2 as ::core::ffi::c_int as WCHAR,
        0xff8b as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30d3 as ::core::ffi::c_int as WCHAR,
        0xff8b as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30d4 as ::core::ffi::c_int as WCHAR,
        0xff8b as ::core::ffi::c_int as WCHAR,
        0x309c as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30d4 as ::core::ffi::c_int as WCHAR,
        0xff8b as ::core::ffi::c_int as WCHAR,
        0xff9f as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30d5 as ::core::ffi::c_int as WCHAR,
        0xff8c as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30d6 as ::core::ffi::c_int as WCHAR,
        0xff8c as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30d7 as ::core::ffi::c_int as WCHAR,
        0xff8c as ::core::ffi::c_int as WCHAR,
        0xff9f as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30d8 as ::core::ffi::c_int as WCHAR,
        0xff8d as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30d9 as ::core::ffi::c_int as WCHAR,
        0xff8d as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30da as ::core::ffi::c_int as WCHAR,
        0xff8d as ::core::ffi::c_int as WCHAR,
        0xff9f as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30db as ::core::ffi::c_int as WCHAR,
        0xff8e as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30dc as ::core::ffi::c_int as WCHAR,
        0xff8e as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30dd as ::core::ffi::c_int as WCHAR,
        0xff8e as ::core::ffi::c_int as WCHAR,
        0xff9f as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30de as ::core::ffi::c_int as WCHAR,
        0xff8f as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30df as ::core::ffi::c_int as WCHAR,
        0xff90 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30e0 as ::core::ffi::c_int as WCHAR,
        0xff91 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30e1 as ::core::ffi::c_int as WCHAR,
        0xff92 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30e2 as ::core::ffi::c_int as WCHAR,
        0xff93 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30e3 as ::core::ffi::c_int as WCHAR,
        0xff6c as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30e4 as ::core::ffi::c_int as WCHAR,
        0xff94 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30e5 as ::core::ffi::c_int as WCHAR,
        0xff6d as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30e6 as ::core::ffi::c_int as WCHAR,
        0xff95 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30e7 as ::core::ffi::c_int as WCHAR,
        0xff6e as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30e8 as ::core::ffi::c_int as WCHAR,
        0xff96 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30e9 as ::core::ffi::c_int as WCHAR,
        0xff97 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30ea as ::core::ffi::c_int as WCHAR,
        0xff98 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30eb as ::core::ffi::c_int as WCHAR,
        0xff99 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30ec as ::core::ffi::c_int as WCHAR,
        0xff9a as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30ed as ::core::ffi::c_int as WCHAR,
        0xff9b as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30ef as ::core::ffi::c_int as WCHAR,
        0xff9c as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30f2 as ::core::ffi::c_int as WCHAR,
        0xff66 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30f3 as ::core::ffi::c_int as WCHAR,
        0xff9d as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30f4 as ::core::ffi::c_int as WCHAR,
        0xff73 as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30f7 as ::core::ffi::c_int as WCHAR,
        0xff9c as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30fa as ::core::ffi::c_int as WCHAR,
        0xff66 as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30fb as ::core::ffi::c_int as WCHAR,
        0xff65 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30fc as ::core::ffi::c_int as WCHAR,
        0xff70 as ::core::ffi::c_int as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ],
];

static mut dakuten_table: [DAKUTEN_ENTRY; 54] = [
    [
        0x304c as ::core::ffi::c_int as WCHAR,
        0x304b as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x304e as ::core::ffi::c_int as WCHAR,
        0x304d as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x3050 as ::core::ffi::c_int as WCHAR,
        0x304f as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x3052 as ::core::ffi::c_int as WCHAR,
        0x3051 as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x3054 as ::core::ffi::c_int as WCHAR,
        0x3053 as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x3056 as ::core::ffi::c_int as WCHAR,
        0x3055 as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x3058 as ::core::ffi::c_int as WCHAR,
        0x3057 as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x305a as ::core::ffi::c_int as WCHAR,
        0x3059 as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x305c as ::core::ffi::c_int as WCHAR,
        0x305b as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x305e as ::core::ffi::c_int as WCHAR,
        0x305d as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x3060 as ::core::ffi::c_int as WCHAR,
        0x305f as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x3062 as ::core::ffi::c_int as WCHAR,
        0x3061 as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x3065 as ::core::ffi::c_int as WCHAR,
        0x3064 as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x3067 as ::core::ffi::c_int as WCHAR,
        0x3066 as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x3069 as ::core::ffi::c_int as WCHAR,
        0x3068 as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x3070 as ::core::ffi::c_int as WCHAR,
        0x306f as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x3071 as ::core::ffi::c_int as WCHAR,
        0x306f as ::core::ffi::c_int as WCHAR,
        0x309c as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x3073 as ::core::ffi::c_int as WCHAR,
        0x3072 as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x3074 as ::core::ffi::c_int as WCHAR,
        0x3072 as ::core::ffi::c_int as WCHAR,
        0x309c as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x3076 as ::core::ffi::c_int as WCHAR,
        0x3075 as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x3077 as ::core::ffi::c_int as WCHAR,
        0x3075 as ::core::ffi::c_int as WCHAR,
        0x309c as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x3079 as ::core::ffi::c_int as WCHAR,
        0x3078 as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x307a as ::core::ffi::c_int as WCHAR,
        0x3078 as ::core::ffi::c_int as WCHAR,
        0x309c as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x307c as ::core::ffi::c_int as WCHAR,
        0x307b as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x307d as ::core::ffi::c_int as WCHAR,
        0x307b as ::core::ffi::c_int as WCHAR,
        0x309c as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x3094 as ::core::ffi::c_int as WCHAR,
        0x3046 as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30ac as ::core::ffi::c_int as WCHAR,
        0xff76 as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30ae as ::core::ffi::c_int as WCHAR,
        0xff77 as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30b0 as ::core::ffi::c_int as WCHAR,
        0xff78 as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30b2 as ::core::ffi::c_int as WCHAR,
        0xff79 as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30b4 as ::core::ffi::c_int as WCHAR,
        0xff7a as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30b6 as ::core::ffi::c_int as WCHAR,
        0xff7b as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30b8 as ::core::ffi::c_int as WCHAR,
        0xff7c as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30ba as ::core::ffi::c_int as WCHAR,
        0xff7d as ::core::ffi::c_int as WCHAR,
        0xff9e as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30bc as ::core::ffi::c_int as WCHAR,
        0xff7e as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30be as ::core::ffi::c_int as WCHAR,
        0xff7f as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30c0 as ::core::ffi::c_int as WCHAR,
        0xff80 as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30c2 as ::core::ffi::c_int as WCHAR,
        0xff81 as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30c5 as ::core::ffi::c_int as WCHAR,
        0xff82 as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30c7 as ::core::ffi::c_int as WCHAR,
        0xff83 as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30c9 as ::core::ffi::c_int as WCHAR,
        0xff84 as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30d0 as ::core::ffi::c_int as WCHAR,
        0xff8a as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30d1 as ::core::ffi::c_int as WCHAR,
        0xff8a as ::core::ffi::c_int as WCHAR,
        0x309c as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30d3 as ::core::ffi::c_int as WCHAR,
        0xff8b as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30d4 as ::core::ffi::c_int as WCHAR,
        0xff8b as ::core::ffi::c_int as WCHAR,
        0x309c as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30d6 as ::core::ffi::c_int as WCHAR,
        0xff8c as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30d7 as ::core::ffi::c_int as WCHAR,
        0xff8c as ::core::ffi::c_int as WCHAR,
        0x309c as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30d9 as ::core::ffi::c_int as WCHAR,
        0xff8d as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30da as ::core::ffi::c_int as WCHAR,
        0xff8d as ::core::ffi::c_int as WCHAR,
        0x309c as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30dc as ::core::ffi::c_int as WCHAR,
        0xff8e as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30dd as ::core::ffi::c_int as WCHAR,
        0xff8e as ::core::ffi::c_int as WCHAR,
        0x309c as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30f4 as ::core::ffi::c_int as WCHAR,
        0xff73 as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30f7 as ::core::ffi::c_int as WCHAR,
        0xff9c as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x30fa as ::core::ffi::c_int as WCHAR,
        0xff66 as ::core::ffi::c_int as WCHAR,
        0x309b as ::core::ffi::c_int as WCHAR,
    ],
];

static mut s_sim2tra: [WCHAR_PAIR; 2191] = [
    tagWCHAR_PAIR {
        from: 0x4e07 as WCHAR,
        to: 0x842c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e0e as WCHAR,
        to: 0x8207 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e13 as WCHAR,
        to: 0x5c08 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e1a as WCHAR,
        to: 0x696d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e1b as WCHAR,
        to: 0x53e2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e1c as WCHAR,
        to: 0x6771 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e1d as WCHAR,
        to: 0x7d72 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e22 as WCHAR,
        to: 0x4e1f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e24 as WCHAR,
        to: 0x5169 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e25 as WCHAR,
        to: 0x56b4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e27 as WCHAR,
        to: 0x55aa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e2a as WCHAR,
        to: 0x500b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e30 as WCHAR,
        to: 0x8c50 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e34 as WCHAR,
        to: 0x81e8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e3a as WCHAR,
        to: 0x70ba as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e3d as WCHAR,
        to: 0x9e97 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e3e as WCHAR,
        to: 0x8209 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e49 as WCHAR,
        to: 0x7fa9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e4c as WCHAR,
        to: 0x70cf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e50 as WCHAR,
        to: 0x6a02 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e54 as WCHAR,
        to: 0x55ac as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e60 as WCHAR,
        to: 0x7fd2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e61 as WCHAR,
        to: 0x9109 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e66 as WCHAR,
        to: 0x66f8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e70 as WCHAR,
        to: 0x8cb7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e71 as WCHAR,
        to: 0x4e82 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e89 as WCHAR,
        to: 0x722d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e8f as WCHAR,
        to: 0x8667 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e98 as WCHAR,
        to: 0x4e99 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e9a as WCHAR,
        to: 0x4e9e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4ea7 as WCHAR,
        to: 0x7522 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4ea9 as WCHAR,
        to: 0x755d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4eb2 as WCHAR,
        to: 0x89aa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4eb5 as WCHAR,
        to: 0x893b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4ebf as WCHAR,
        to: 0x5104 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4ec5 as WCHAR,
        to: 0x50c5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4ece as WCHAR,
        to: 0x5f9e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4ed1 as WCHAR,
        to: 0x4f96 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4ed3 as WCHAR,
        to: 0x5009 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4eea as WCHAR,
        to: 0x5100 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4eec as WCHAR,
        to: 0x5011 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4ef7 as WCHAR,
        to: 0x50f9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4f17 as WCHAR,
        to: 0x773e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4f18 as WCHAR,
        to: 0x512a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4f1a as WCHAR,
        to: 0x6703 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4f1b as WCHAR,
        to: 0x50b4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4f1e as WCHAR,
        to: 0x5098 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4f1f as WCHAR,
        to: 0x5049 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4f20 as WCHAR,
        to: 0x50b3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4f24 as WCHAR,
        to: 0x50b7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4f25 as WCHAR,
        to: 0x5000 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4f26 as WCHAR,
        to: 0x502b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4f27 as WCHAR,
        to: 0x5096 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4f2a as WCHAR,
        to: 0x507d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4f2b as WCHAR,
        to: 0x4f47 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4f53 as WCHAR,
        to: 0x9ad4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4f63 as WCHAR,
        to: 0x50ad as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4f65 as WCHAR,
        to: 0x50c9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4fa0 as WCHAR,
        to: 0x4fe0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4fa3 as WCHAR,
        to: 0x4fb6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4fa5 as WCHAR,
        to: 0x50e5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4fa6 as WCHAR,
        to: 0x5075 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4fa7 as WCHAR,
        to: 0x5074 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4fa8 as WCHAR,
        to: 0x50d1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4fa9 as WCHAR,
        to: 0x5108 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4faa as WCHAR,
        to: 0x5115 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4fac as WCHAR,
        to: 0x5102 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4fe3 as WCHAR,
        to: 0x4fc1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4fe6 as WCHAR,
        to: 0x5114 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4fe8 as WCHAR,
        to: 0x513c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4fe9 as WCHAR,
        to: 0x5006 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4fea as WCHAR,
        to: 0x5137 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4fed as WCHAR,
        to: 0x5109 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x503a as WCHAR,
        to: 0x50b5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x503e as WCHAR,
        to: 0x50be as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x506c as WCHAR,
        to: 0x50af as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x507b as WCHAR,
        to: 0x50c2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x507e as WCHAR,
        to: 0x50e8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x507f as WCHAR,
        to: 0x511f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x50a5 as WCHAR,
        to: 0x513b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x50a7 as WCHAR,
        to: 0x5110 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x50a8 as WCHAR,
        to: 0x5132 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x50a9 as WCHAR,
        to: 0x513a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x513f as WCHAR,
        to: 0x5152 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5151 as WCHAR,
        to: 0x514c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5156 as WCHAR,
        to: 0x5157 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x515a as WCHAR,
        to: 0x9ee8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5170 as WCHAR,
        to: 0x862d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5173 as WCHAR,
        to: 0x95dc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5174 as WCHAR,
        to: 0x8208 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5179 as WCHAR,
        to: 0x8332 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x517b as WCHAR,
        to: 0x990a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x517d as WCHAR,
        to: 0x7378 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5181 as WCHAR,
        to: 0x56c5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5185 as WCHAR,
        to: 0x5167 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5188 as WCHAR,
        to: 0x5ca1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x518c as WCHAR,
        to: 0x518a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5199 as WCHAR,
        to: 0x5beb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x519b as WCHAR,
        to: 0x8ecd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x519c as WCHAR,
        to: 0x8fb2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x51af as WCHAR,
        to: 0x99ae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x51b2 as WCHAR,
        to: 0x6c96 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x51b3 as WCHAR,
        to: 0x6c7a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x51b5 as WCHAR,
        to: 0x6cc1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x51bb as WCHAR,
        to: 0x51cd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x51c0 as WCHAR,
        to: 0x51c8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x51c6 as WCHAR,
        to: 0x6e96 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x51c9 as WCHAR,
        to: 0x6dbc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x51cf as WCHAR,
        to: 0x6e1b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x51d1 as WCHAR,
        to: 0x6e4a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x51db as WCHAR,
        to: 0x51dc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x51e0 as WCHAR,
        to: 0x5e7e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x51e4 as WCHAR,
        to: 0x9cf3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x51eb as WCHAR,
        to: 0x9ce7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x51ed as WCHAR,
        to: 0x6191 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x51ef as WCHAR,
        to: 0x51f1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x51f6 as WCHAR,
        to: 0x5147 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x51fb as WCHAR,
        to: 0x64ca as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x51ff as WCHAR,
        to: 0x947f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x520d as WCHAR,
        to: 0x82bb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5212 as WCHAR,
        to: 0x5283 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5218 as WCHAR,
        to: 0x5289 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5219 as WCHAR,
        to: 0x5247 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x521a as WCHAR,
        to: 0x525b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x521b as WCHAR,
        to: 0x5275 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5220 as WCHAR,
        to: 0x522a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x522b as WCHAR,
        to: 0x5225 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x522d as WCHAR,
        to: 0x5244 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5239 as WCHAR,
        to: 0x524e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x523d as WCHAR,
        to: 0x528a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x523f as WCHAR,
        to: 0x528c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5240 as WCHAR,
        to: 0x5274 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5242 as WCHAR,
        to: 0x5291 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5250 as WCHAR,
        to: 0x526e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5251 as WCHAR,
        to: 0x528d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5265 as WCHAR,
        to: 0x525d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5267 as WCHAR,
        to: 0x5287 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x529d as WCHAR,
        to: 0x52f8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x529e as WCHAR,
        to: 0x8fa6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x52a1 as WCHAR,
        to: 0x52d9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x52a2 as WCHAR,
        to: 0x52f1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x52a8 as WCHAR,
        to: 0x52d5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x52b1 as WCHAR,
        to: 0x52f5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x52b2 as WCHAR,
        to: 0x52c1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x52b3 as WCHAR,
        to: 0x52de as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x52bf as WCHAR,
        to: 0x52e2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x52cb as WCHAR,
        to: 0x52db as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5300 as WCHAR,
        to: 0x52fb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5326 as WCHAR,
        to: 0x532d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x532e as WCHAR,
        to: 0x5331 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x533a as WCHAR,
        to: 0x5340 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x533b as WCHAR,
        to: 0x91ab as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x534e as WCHAR,
        to: 0x83ef as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x534f as WCHAR,
        to: 0x5354 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5355 as WCHAR,
        to: 0x55ae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5356 as WCHAR,
        to: 0x8ce3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5362 as WCHAR,
        to: 0x76e7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5364 as WCHAR,
        to: 0x9e75 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5367 as WCHAR,
        to: 0x81e5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x536b as WCHAR,
        to: 0x885b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5374 as WCHAR,
        to: 0x537b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x537a as WCHAR,
        to: 0x5df9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5382 as WCHAR,
        to: 0x5ee0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5385 as WCHAR,
        to: 0x5ef3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5386 as WCHAR,
        to: 0x6b77 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5389 as WCHAR,
        to: 0x53b2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x538b as WCHAR,
        to: 0x58d3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x538c as WCHAR,
        to: 0x53ad as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x538d as WCHAR,
        to: 0x5399 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5395 as WCHAR,
        to: 0x5ec1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x53a2 as WCHAR,
        to: 0x5ec2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x53a3 as WCHAR,
        to: 0x53b4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x53a6 as WCHAR,
        to: 0x5ec8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x53a8 as WCHAR,
        to: 0x5eda as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x53a9 as WCHAR,
        to: 0x5ec4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x53ae as WCHAR,
        to: 0x5edd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x53bf as WCHAR,
        to: 0x7e23 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x53c2 as WCHAR,
        to: 0x53c3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x53cc as WCHAR,
        to: 0x96d9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x53d1 as WCHAR,
        to: 0x767c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x53d8 as WCHAR,
        to: 0x8b8a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x53d9 as WCHAR,
        to: 0x6558 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x53e0 as WCHAR,
        to: 0x758a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x53f0 as WCHAR,
        to: 0x81fa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x53f6 as WCHAR,
        to: 0x8449 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x53f7 as WCHAR,
        to: 0x865f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x53f9 as WCHAR,
        to: 0x5606 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x53fd as WCHAR,
        to: 0x5630 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5413 as WCHAR,
        to: 0x5687 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5415 as WCHAR,
        to: 0x5442 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5417 as WCHAR,
        to: 0x55ce as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5428 as WCHAR,
        to: 0x5678 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x542c as WCHAR,
        to: 0x807d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x542f as WCHAR,
        to: 0x555f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5434 as WCHAR,
        to: 0x5433 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5450 as WCHAR,
        to: 0x5436 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5452 as WCHAR,
        to: 0x5638 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5453 as WCHAR,
        to: 0x56c8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5455 as WCHAR,
        to: 0x5614 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5456 as WCHAR,
        to: 0x56a6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5457 as WCHAR,
        to: 0x5504 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5458 as WCHAR,
        to: 0x54e1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5459 as WCHAR,
        to: 0x54bc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x545b as WCHAR,
        to: 0x55c6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x545c as WCHAR,
        to: 0x55da as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x548f as WCHAR,
        to: 0x8a60 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5499 as WCHAR,
        to: 0x56a8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x549b as WCHAR,
        to: 0x5680 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x54cd as WCHAR,
        to: 0x97ff as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x54d1 as WCHAR,
        to: 0x555e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x54d2 as WCHAR,
        to: 0x5660 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x54d3 as WCHAR,
        to: 0x5635 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x54d4 as WCHAR,
        to: 0x55f6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x54d5 as WCHAR,
        to: 0x5666 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x54d7 as WCHAR,
        to: 0x5629 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x54d9 as WCHAR,
        to: 0x5672 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x54dc as WCHAR,
        to: 0x568c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x54dd as WCHAR,
        to: 0x5665 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x54df as WCHAR,
        to: 0x55b2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x551b as WCHAR,
        to: 0x561c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5520 as WCHAR,
        to: 0x562e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5522 as WCHAR,
        to: 0x55e9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5524 as WCHAR,
        to: 0x559a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5567 as WCHAR,
        to: 0x5616 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x556c as WCHAR,
        to: 0x55c7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x556d as WCHAR,
        to: 0x56c0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x556e as WCHAR,
        to: 0x5699 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5578 as WCHAR,
        to: 0x562f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x55b7 as WCHAR,
        to: 0x5674 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x55bd as WCHAR,
        to: 0x560d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x55be as WCHAR,
        to: 0x56b3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x55eb as WCHAR,
        to: 0x56c1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x55f3 as WCHAR,
        to: 0x566f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5618 as WCHAR,
        to: 0x5653 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5624 as WCHAR,
        to: 0x56b6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5631 as WCHAR,
        to: 0x56d1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x565c as WCHAR,
        to: 0x5695 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x56a3 as WCHAR,
        to: 0x56c2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x56e2 as WCHAR,
        to: 0x5718 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x56ed as WCHAR,
        to: 0x5712 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x56f1 as WCHAR,
        to: 0x56ea as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x56f4 as WCHAR,
        to: 0x570d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x56f5 as WCHAR,
        to: 0x5707 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x56fd as WCHAR,
        to: 0x570b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x56fe as WCHAR,
        to: 0x5716 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5706 as WCHAR,
        to: 0x5713 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5739 as WCHAR,
        to: 0x58d9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x573a as WCHAR,
        to: 0x5834 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x574f as WCHAR,
        to: 0x58de as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5757 as WCHAR,
        to: 0x584a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x575a as WCHAR,
        to: 0x5805 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x575b as WCHAR,
        to: 0x58c7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x575c as WCHAR,
        to: 0x58e2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x575d as WCHAR,
        to: 0x58e9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x575e as WCHAR,
        to: 0x5862 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x575f as WCHAR,
        to: 0x58b3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5760 as WCHAR,
        to: 0x589c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5784 as WCHAR,
        to: 0x58df as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5786 as WCHAR,
        to: 0x58da as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5792 as WCHAR,
        to: 0x58d8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x57a6 as WCHAR,
        to: 0x58be as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x57a7 as WCHAR,
        to: 0x5770 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x57a9 as WCHAR,
        to: 0x580a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x57ab as WCHAR,
        to: 0x588a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x57ad as WCHAR,
        to: 0x57e1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x57b2 as WCHAR,
        to: 0x584f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x57d8 as WCHAR,
        to: 0x5852 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x57d9 as WCHAR,
        to: 0x5864 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x57da as WCHAR,
        to: 0x581d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5811 as WCHAR,
        to: 0x5879 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5815 as WCHAR,
        to: 0x58ae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5892 as WCHAR,
        to: 0x5891 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5899 as WCHAR,
        to: 0x58bb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x58ee as WCHAR,
        to: 0x58ef as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x58f0 as WCHAR,
        to: 0x8072 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x58f3 as WCHAR,
        to: 0x6bbc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x58f6 as WCHAR,
        to: 0x58fa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5904 as WCHAR,
        to: 0x8655 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5907 as WCHAR,
        to: 0x5099 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x590d as WCHAR,
        to: 0x5fa9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x591f as WCHAR,
        to: 0x5920 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5934 as WCHAR,
        to: 0x982d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5939 as WCHAR,
        to: 0x593e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x593a as WCHAR,
        to: 0x596a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5941 as WCHAR,
        to: 0x5969 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5942 as WCHAR,
        to: 0x5950 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x594b as WCHAR,
        to: 0x596e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5956 as WCHAR,
        to: 0x734e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5965 as WCHAR,
        to: 0x5967 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5986 as WCHAR,
        to: 0x599d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5987 as WCHAR,
        to: 0x5a66 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5988 as WCHAR,
        to: 0x5abd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x59a9 as WCHAR,
        to: 0x5af5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x59aa as WCHAR,
        to: 0x5ad7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x59ab as WCHAR,
        to: 0x5aaf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x59d7 as WCHAR,
        to: 0x59cd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5a04 as WCHAR,
        to: 0x5a41 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5a05 as WCHAR,
        to: 0x5a6d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5a06 as WCHAR,
        to: 0x5b08 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5a07 as WCHAR,
        to: 0x5b0c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5a08 as WCHAR,
        to: 0x5b4c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5a31 as WCHAR,
        to: 0x5a1b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5a32 as WCHAR,
        to: 0x5aa7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5a34 as WCHAR,
        to: 0x5afb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5a74 as WCHAR,
        to: 0x5b30 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5a75 as WCHAR,
        to: 0x5b0b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5a76 as WCHAR,
        to: 0x5b38 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5aaa as WCHAR,
        to: 0x5abc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5ad2 as WCHAR,
        to: 0x5b21 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5ad4 as WCHAR,
        to: 0x5b2a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5af1 as WCHAR,
        to: 0x5b19 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5b37 as WCHAR,
        to: 0x5b24 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5b59 as WCHAR,
        to: 0x5b6b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5b66 as WCHAR,
        to: 0x5b78 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5b6a as WCHAR,
        to: 0x5b7f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5b81 as WCHAR,
        to: 0x5be7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5b9d as WCHAR,
        to: 0x5bf6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5b9e as WCHAR,
        to: 0x5be6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5ba0 as WCHAR,
        to: 0x5bf5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5ba1 as WCHAR,
        to: 0x5be9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5baa as WCHAR,
        to: 0x61b2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5bab as WCHAR,
        to: 0x5bae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5bbd as WCHAR,
        to: 0x5bec as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5bbe as WCHAR,
        to: 0x8cd3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5bdd as WCHAR,
        to: 0x5be2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5bf9 as WCHAR,
        to: 0x5c0d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5bfb as WCHAR,
        to: 0x5c0b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5bfc as WCHAR,
        to: 0x5c0e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5bff as WCHAR,
        to: 0x58fd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c06 as WCHAR,
        to: 0x5c07 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c14 as WCHAR,
        to: 0x723e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c18 as WCHAR,
        to: 0x5875 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c1d as WCHAR,
        to: 0x5617 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c27 as WCHAR,
        to: 0x582f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c34 as WCHAR,
        to: 0x5c37 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c3d as WCHAR,
        to: 0x76e1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c42 as WCHAR,
        to: 0x5c64 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c49 as WCHAR,
        to: 0x5c5c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c4a as WCHAR,
        to: 0x5c46 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c5e as WCHAR,
        to: 0x5c6c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c61 as WCHAR,
        to: 0x5c62 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c66 as WCHAR,
        to: 0x5c68 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c7f as WCHAR,
        to: 0x5dbc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c81 as WCHAR,
        to: 0x6b72 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c82 as WCHAR,
        to: 0x8c48 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c96 as WCHAR,
        to: 0x5d87 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c97 as WCHAR,
        to: 0x5d17 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c98 as WCHAR,
        to: 0x5cf4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c9a as WCHAR,
        to: 0x5d50 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c9b as WCHAR,
        to: 0x5cf6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5ca9 as WCHAR,
        to: 0x5dd6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5cad as WCHAR,
        to: 0x5dba as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5cbd as WCHAR,
        to: 0x5d20 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5cbf as WCHAR,
        to: 0x5dcb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5cc4 as WCHAR,
        to: 0x5da7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5ce1 as WCHAR,
        to: 0x5cfd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5ce4 as WCHAR,
        to: 0x5da0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5ce5 as WCHAR,
        to: 0x5d22 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5ce6 as WCHAR,
        to: 0x5dd2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5d02 as WCHAR,
        to: 0x5d97 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5d03 as WCHAR,
        to: 0x5d0d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5d2d as WCHAR,
        to: 0x5d84 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5d58 as WCHAR,
        to: 0x5db8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5d5b as WCHAR,
        to: 0x5d33 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5d5d as WCHAR,
        to: 0x5d81 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5dc5 as WCHAR,
        to: 0x5dd4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5de9 as WCHAR,
        to: 0x978f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5def as WCHAR,
        to: 0x5df0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e01 as WCHAR,
        to: 0x5e63 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e05 as WCHAR,
        to: 0x5e25 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e08 as WCHAR,
        to: 0x5e2b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e0f as WCHAR,
        to: 0x5e43 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e10 as WCHAR,
        to: 0x5e33 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e18 as WCHAR,
        to: 0x7c3e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e1c as WCHAR,
        to: 0x5e5f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e26 as WCHAR,
        to: 0x5e36 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e27 as WCHAR,
        to: 0x5e40 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e2e as WCHAR,
        to: 0x5e6b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e31 as WCHAR,
        to: 0x5e6c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e3b as WCHAR,
        to: 0x5e58 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e3c as WCHAR,
        to: 0x5e57 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e42 as WCHAR,
        to: 0x51aa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e7f as WCHAR,
        to: 0x5ee3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e84 as WCHAR,
        to: 0x838a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e86 as WCHAR,
        to: 0x6176 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e90 as WCHAR,
        to: 0x5eec as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e91 as WCHAR,
        to: 0x5ee1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e93 as WCHAR,
        to: 0x5eab as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e94 as WCHAR,
        to: 0x61c9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e99 as WCHAR,
        to: 0x5edf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e9e as WCHAR,
        to: 0x9f90 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e9f as WCHAR,
        to: 0x5ee2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5eea as WCHAR,
        to: 0x5ee9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f00 as WCHAR,
        to: 0x958b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f02 as WCHAR,
        to: 0x7570 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f03 as WCHAR,
        to: 0x68c4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f11 as WCHAR,
        to: 0x5f12 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f20 as WCHAR,
        to: 0x5f35 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f25 as WCHAR,
        to: 0x5f4c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f2a as WCHAR,
        to: 0x5f33 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f2f as WCHAR,
        to: 0x5f4e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f39 as WCHAR,
        to: 0x5f48 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f3a as WCHAR,
        to: 0x5f37 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f52 as WCHAR,
        to: 0x6b78 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f53 as WCHAR,
        to: 0x7576 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f55 as WCHAR,
        to: 0x9304 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f66 as WCHAR,
        to: 0x5f65 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f7b as WCHAR,
        to: 0x5fb9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f84 as WCHAR,
        to: 0x5f91 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f95 as WCHAR,
        to: 0x5fa0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5fc6 as WCHAR,
        to: 0x61b6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5fcf as WCHAR,
        to: 0x61fa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5fe7 as WCHAR,
        to: 0x6182 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5ffe as WCHAR,
        to: 0x613e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6000 as WCHAR,
        to: 0x61f7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6001 as WCHAR,
        to: 0x614b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6002 as WCHAR,
        to: 0x616b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6003 as WCHAR,
        to: 0x61ae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6004 as WCHAR,
        to: 0x616a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6005 as WCHAR,
        to: 0x60b5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6006 as WCHAR,
        to: 0x6134 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x601c as WCHAR,
        to: 0x6190 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x603b as WCHAR,
        to: 0x7e3d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x603c as WCHAR,
        to: 0x61df as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x603f as WCHAR,
        to: 0x61cc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x604b as WCHAR,
        to: 0x6200 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6073 as WCHAR,
        to: 0x61c7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6076 as WCHAR,
        to: 0x60e1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6078 as WCHAR,
        to: 0x615f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6079 as WCHAR,
        to: 0x61e8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x607a as WCHAR,
        to: 0x6137 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x607b as WCHAR,
        to: 0x60fb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x607c as WCHAR,
        to: 0x60f1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x607d as WCHAR,
        to: 0x60f2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x60a6 as WCHAR,
        to: 0x6085 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x60ab as WCHAR,
        to: 0x6128 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x60ac as WCHAR,
        to: 0x61f8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x60ad as WCHAR,
        to: 0x6173 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x60af as WCHAR,
        to: 0x61ab as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x60ca as WCHAR,
        to: 0x9a5a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x60e7 as WCHAR,
        to: 0x61fc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x60e8 as WCHAR,
        to: 0x6158 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x60e9 as WCHAR,
        to: 0x61f2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x60eb as WCHAR,
        to: 0x618a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x60ec as WCHAR,
        to: 0x611c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x60ed as WCHAR,
        to: 0x615a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x60ee as WCHAR,
        to: 0x619a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x60ef as WCHAR,
        to: 0x6163 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6120 as WCHAR,
        to: 0x614d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6124 as WCHAR,
        to: 0x61a4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6126 as WCHAR,
        to: 0x6192 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6151 as WCHAR,
        to: 0x61fe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x61d1 as WCHAR,
        to: 0x61e3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x61d2 as WCHAR,
        to: 0x61f6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6206 as WCHAR,
        to: 0x6207 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x620b as WCHAR,
        to: 0x6214 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x620f as WCHAR,
        to: 0x6232 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6217 as WCHAR,
        to: 0x6227 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6218 as WCHAR,
        to: 0x6230 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x622c as WCHAR,
        to: 0x6229 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6237 as WCHAR,
        to: 0x6236 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6251 as WCHAR,
        to: 0x64b2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x625e as WCHAR,
        to: 0x634d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6267 as WCHAR,
        to: 0x57f7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6269 as WCHAR,
        to: 0x64f4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x626a as WCHAR,
        to: 0x636b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x626b as WCHAR,
        to: 0x6383 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x626c as WCHAR,
        to: 0x63da as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6270 as WCHAR,
        to: 0x64fe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x629a as WCHAR,
        to: 0x64ab as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x629b as WCHAR,
        to: 0x62cb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x629f as WCHAR,
        to: 0x6476 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x62a0 as WCHAR,
        to: 0x6473 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x62a1 as WCHAR,
        to: 0x6384 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x62a2 as WCHAR,
        to: 0x6436 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x62a4 as WCHAR,
        to: 0x8b77 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x62a5 as WCHAR,
        to: 0x5831 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x62c5 as WCHAR,
        to: 0x64d4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x62df as WCHAR,
        to: 0x64ec as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x62e2 as WCHAR,
        to: 0x650f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x62e3 as WCHAR,
        to: 0x63c0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x62e5 as WCHAR,
        to: 0x64c1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x62e6 as WCHAR,
        to: 0x6514 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x62e7 as WCHAR,
        to: 0x64f0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x62e8 as WCHAR,
        to: 0x64a5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x62e9 as WCHAR,
        to: 0x64c7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6302 as WCHAR,
        to: 0x639b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x631a as WCHAR,
        to: 0x646f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x631b as WCHAR,
        to: 0x6523 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x631d as WCHAR,
        to: 0x64be as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x631e as WCHAR,
        to: 0x64bb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x631f as WCHAR,
        to: 0x633e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6320 as WCHAR,
        to: 0x6493 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6321 as WCHAR,
        to: 0x64cb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6322 as WCHAR,
        to: 0x649f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6323 as WCHAR,
        to: 0x6399 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6324 as WCHAR,
        to: 0x64e0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6325 as WCHAR,
        to: 0x63ee as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x635e as WCHAR,
        to: 0x6488 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x635f as WCHAR,
        to: 0x640d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6361 as WCHAR,
        to: 0x64bf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6362 as WCHAR,
        to: 0x63db as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6363 as WCHAR,
        to: 0x6417 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x636e as WCHAR,
        to: 0x64da as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x63b3 as WCHAR,
        to: 0x64c4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x63b4 as WCHAR,
        to: 0x6451 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x63b7 as WCHAR,
        to: 0x64f2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x63b8 as WCHAR,
        to: 0x64a3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x63ba as WCHAR,
        to: 0x647b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x63bc as WCHAR,
        to: 0x645c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x63fd as WCHAR,
        to: 0x652c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x63ff as WCHAR,
        to: 0x64b3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6400 as WCHAR,
        to: 0x6519 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6401 as WCHAR,
        to: 0x64f1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6402 as WCHAR,
        to: 0x645f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6405 as WCHAR,
        to: 0x652a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x643a as WCHAR,
        to: 0x651c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6444 as WCHAR,
        to: 0x651d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6445 as WCHAR,
        to: 0x6504 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6446 as WCHAR,
        to: 0x64fa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6447 as WCHAR,
        to: 0x6416 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6448 as WCHAR,
        to: 0x64ef as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x644a as WCHAR,
        to: 0x6524 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6484 as WCHAR,
        to: 0x6516 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6491 as WCHAR,
        to: 0x6490 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64b5 as WCHAR,
        to: 0x6506 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64b7 as WCHAR,
        to: 0x64f7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64b8 as WCHAR,
        to: 0x64fc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64ba as WCHAR,
        to: 0x651b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64c0 as WCHAR,
        to: 0x641f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64de as WCHAR,
        to: 0x64fb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6512 as WCHAR,
        to: 0x6522 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x654c as WCHAR,
        to: 0x6575 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x655b as WCHAR,
        to: 0x6582 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6570 as WCHAR,
        to: 0x6578 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x658b as WCHAR,
        to: 0x9f4b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6593 as WCHAR,
        to: 0x6595 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x65a9 as WCHAR,
        to: 0x65ac as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x65ad as WCHAR,
        to: 0x65b7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x65e0 as WCHAR,
        to: 0x7121 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x65e7 as WCHAR,
        to: 0x820a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x65f6 as WCHAR,
        to: 0x6642 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x65f7 as WCHAR,
        to: 0x66e0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6619 as WCHAR,
        to: 0x66c7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x663c as WCHAR,
        to: 0x665d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x663e as WCHAR,
        to: 0x986f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x664b as WCHAR,
        to: 0x6649 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6652 as WCHAR,
        to: 0x66ec as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6653 as WCHAR,
        to: 0x66c9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6654 as WCHAR,
        to: 0x66c4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6655 as WCHAR,
        to: 0x6688 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6656 as WCHAR,
        to: 0x6689 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6682 as WCHAR,
        to: 0x66ab as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x66a7 as WCHAR,
        to: 0x66d6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x672f as WCHAR,
        to: 0x8853 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6734 as WCHAR,
        to: 0x6a38 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x673a as WCHAR,
        to: 0x6a5f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6740 as WCHAR,
        to: 0x6bba as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6742 as WCHAR,
        to: 0x96dc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6743 as WCHAR,
        to: 0x6b0a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6746 as WCHAR,
        to: 0x687f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6761 as WCHAR,
        to: 0x689d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6765 as WCHAR,
        to: 0x4f86 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6768 as WCHAR,
        to: 0x694a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6769 as WCHAR,
        to: 0x69aa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6781 as WCHAR,
        to: 0x6975 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6784 as WCHAR,
        to: 0x69cb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x679e as WCHAR,
        to: 0x6a05 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x67a2 as WCHAR,
        to: 0x6a1e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x67a3 as WCHAR,
        to: 0x68d7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x67a5 as WCHAR,
        to: 0x6aea as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x67a8 as WCHAR,
        to: 0x68d6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x67aa as WCHAR,
        to: 0x69cd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x67ab as WCHAR,
        to: 0x6953 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x67ad as WCHAR,
        to: 0x689f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x67e0 as WCHAR,
        to: 0x6ab8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x67fd as WCHAR,
        to: 0x6a89 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6800 as WCHAR,
        to: 0x6894 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6805 as WCHAR,
        to: 0x67f5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6807 as WCHAR,
        to: 0x6a19 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6808 as WCHAR,
        to: 0x68e7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6809 as WCHAR,
        to: 0x6adb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x680a as WCHAR,
        to: 0x6af3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x680b as WCHAR,
        to: 0x68df as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x680c as WCHAR,
        to: 0x6ae8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x680e as WCHAR,
        to: 0x6adf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x680f as WCHAR,
        to: 0x6b04 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6811 as WCHAR,
        to: 0x6a39 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6816 as WCHAR,
        to: 0x68f2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6837 as WCHAR,
        to: 0x6a23 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x683e as WCHAR,
        to: 0x6b12 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6860 as WCHAR,
        to: 0x690f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6861 as WCHAR,
        to: 0x6a48 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6862 as WCHAR,
        to: 0x6968 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6863 as WCHAR,
        to: 0x6a94 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6864 as WCHAR,
        to: 0x69bf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6865 as WCHAR,
        to: 0x6a4b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6866 as WCHAR,
        to: 0x6a3a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6867 as WCHAR,
        to: 0x6a9c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6868 as WCHAR,
        to: 0x69f3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6869 as WCHAR,
        to: 0x6a01 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x68a6 as WCHAR,
        to: 0x5922 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x68c0 as WCHAR,
        to: 0x6aa2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x68c2 as WCHAR,
        to: 0x6b1e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6901 as WCHAR,
        to: 0x69e8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x691f as WCHAR,
        to: 0x6add as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6920 as WCHAR,
        to: 0x69e7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6924 as WCHAR,
        to: 0x6b0f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x692d as WCHAR,
        to: 0x6a62 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x697c as WCHAR,
        to: 0x6a13 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6984 as WCHAR,
        to: 0x6b16 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6987 as WCHAR,
        to: 0x6aec as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6988 as WCHAR,
        to: 0x6ada as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6989 as WCHAR,
        to: 0x6af8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x69db as WCHAR,
        to: 0x6abb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x69df as WCHAR,
        to: 0x6ab3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x69e0 as WCHAR,
        to: 0x6ae7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a2a as WCHAR,
        to: 0x6a6b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a2f as WCHAR,
        to: 0x6aa3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a31 as WCHAR,
        to: 0x6afb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a71 as WCHAR,
        to: 0x6ae5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a79 as WCHAR,
        to: 0x6ad3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a7c as WCHAR,
        to: 0x6ade as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6aa9 as WCHAR,
        to: 0x6a81 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b22 as WCHAR,
        to: 0x6b61 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b24 as WCHAR,
        to: 0x6b5f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b27 as WCHAR,
        to: 0x6b50 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b7c as WCHAR,
        to: 0x6bb2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b81 as WCHAR,
        to: 0x6b7f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b87 as WCHAR,
        to: 0x6ba4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b8b as WCHAR,
        to: 0x6b98 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b92 as WCHAR,
        to: 0x6b9e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b93 as WCHAR,
        to: 0x6bae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b9a as WCHAR,
        to: 0x6bab as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ba1 as WCHAR,
        to: 0x6baf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6bb4 as WCHAR,
        to: 0x6bc6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6bc1 as WCHAR,
        to: 0x6bc0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6bc2 as WCHAR,
        to: 0x8f42 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6bd5 as WCHAR,
        to: 0x7562 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6bd9 as WCHAR,
        to: 0x6583 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6be1 as WCHAR,
        to: 0x6c08 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6bf5 as WCHAR,
        to: 0x6bff as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6c07 as WCHAR,
        to: 0x6c0c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6c14 as WCHAR,
        to: 0x6c23 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6c22 as WCHAR,
        to: 0x6c2b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6c29 as WCHAR,
        to: 0x6c2c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6c32 as WCHAR,
        to: 0x6c33 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6c47 as WCHAR,
        to: 0x532f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6c49 as WCHAR,
        to: 0x6f22 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6c64 as WCHAR,
        to: 0x6e6f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6c79 as WCHAR,
        to: 0x6d36 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6c9f as WCHAR,
        to: 0x6e9d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ca1 as WCHAR,
        to: 0x6c92 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ca3 as WCHAR,
        to: 0x7043 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ca4 as WCHAR,
        to: 0x6f1a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ca5 as WCHAR,
        to: 0x701d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ca6 as WCHAR,
        to: 0x6dea as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ca7 as WCHAR,
        to: 0x6ec4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ca9 as WCHAR,
        to: 0x6e88 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6caa as WCHAR,
        to: 0x6eec as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6cde as WCHAR,
        to: 0x6fd8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6cea as WCHAR,
        to: 0x6dda as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6cf6 as WCHAR,
        to: 0x6fa9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6cf7 as WCHAR,
        to: 0x7027 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6cf8 as WCHAR,
        to: 0x7018 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6cfa as WCHAR,
        to: 0x6ffc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6cfb as WCHAR,
        to: 0x7009 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6cfc as WCHAR,
        to: 0x6f51 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6cfd as WCHAR,
        to: 0x6fa4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6cfe as WCHAR,
        to: 0x6d87 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6d01 as WCHAR,
        to: 0x6f54 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6d12 as WCHAR,
        to: 0x7051 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6d43 as WCHAR,
        to: 0x6d79 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6d45 as WCHAR,
        to: 0x6dfa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6d46 as WCHAR,
        to: 0x6f3f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6d47 as WCHAR,
        to: 0x6f86 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6d48 as WCHAR,
        to: 0x6e5e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6d4a as WCHAR,
        to: 0x6fc1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6d4b as WCHAR,
        to: 0x6e2c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6d4d as WCHAR,
        to: 0x6fae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6d4e as WCHAR,
        to: 0x6fdf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6d4f as WCHAR,
        to: 0x700f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6d51 as WCHAR,
        to: 0x6e3e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6d52 as WCHAR,
        to: 0x6ef8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6d53 as WCHAR,
        to: 0x6fc3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6d54 as WCHAR,
        to: 0x6f6f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6d9b as WCHAR,
        to: 0x6fe4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6d9d as WCHAR,
        to: 0x6f87 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6d9e as WCHAR,
        to: 0x6df6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6d9f as WCHAR,
        to: 0x6f23 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6da0 as WCHAR,
        to: 0x6f7f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6da1 as WCHAR,
        to: 0x6e26 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6da3 as WCHAR,
        to: 0x6e19 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6da4 as WCHAR,
        to: 0x6ecc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6da6 as WCHAR,
        to: 0x6f64 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6da7 as WCHAR,
        to: 0x6f97 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6da8 as WCHAR,
        to: 0x6f32 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6da9 as WCHAR,
        to: 0x6f80 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e0a as WCHAR,
        to: 0x6df5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e0c as WCHAR,
        to: 0x6de5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e0d as WCHAR,
        to: 0x6f2c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e0e as WCHAR,
        to: 0x7006 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e10 as WCHAR,
        to: 0x6f38 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e11 as WCHAR,
        to: 0x6fa0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e14 as WCHAR,
        to: 0x6f01 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e16 as WCHAR,
        to: 0x700b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e17 as WCHAR,
        to: 0x6ef2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e29 as WCHAR,
        to: 0x6eab as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e7e as WCHAR,
        to: 0x7063 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e7f as WCHAR,
        to: 0x6fd5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e83 as WCHAR,
        to: 0x6f70 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e85 as WCHAR,
        to: 0x6ffa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ed7 as WCHAR,
        to: 0x6f77 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6eda as WCHAR,
        to: 0x6efe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ede as WCHAR,
        to: 0x6eef as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ee0 as WCHAR,
        to: 0x7044 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ee1 as WCHAR,
        to: 0x6eff as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ee2 as WCHAR,
        to: 0x7005 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ee4 as WCHAR,
        to: 0x6ffe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ee5 as WCHAR,
        to: 0x6feb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ee6 as WCHAR,
        to: 0x7064 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ee8 as WCHAR,
        to: 0x6ff1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ee9 as WCHAR,
        to: 0x7058 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f46 as WCHAR,
        to: 0x7020 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f47 as WCHAR,
        to: 0x701f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f4b as WCHAR,
        to: 0x7032 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f4d as WCHAR,
        to: 0x6ff0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f5c as WCHAR,
        to: 0x6f5b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f9c as WCHAR,
        to: 0x703e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6fd1 as WCHAR,
        to: 0x7028 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6fd2 as WCHAR,
        to: 0x7015 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x704f as WCHAR,
        to: 0x705d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x706d as WCHAR,
        to: 0x6ec5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x706f as WCHAR,
        to: 0x71c8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7075 as WCHAR,
        to: 0x9748 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x707e as WCHAR,
        to: 0x707d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x707f as WCHAR,
        to: 0x71e6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7080 as WCHAR,
        to: 0x716c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7089 as WCHAR,
        to: 0x7210 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7096 as WCHAR,
        to: 0x71c9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x709c as WCHAR,
        to: 0x7152 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x709d as WCHAR,
        to: 0x7197 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x70b9 as WCHAR,
        to: 0x9ede as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x70bc as WCHAR,
        to: 0x7149 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x70bd as WCHAR,
        to: 0x71be as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x70c1 as WCHAR,
        to: 0x720d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x70c2 as WCHAR,
        to: 0x721b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x70c3 as WCHAR,
        to: 0x70f4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x70db as WCHAR,
        to: 0x71ed as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x70df as WCHAR,
        to: 0x7159 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x70e6 as WCHAR,
        to: 0x7169 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x70e7 as WCHAR,
        to: 0x71d2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x70e8 as WCHAR,
        to: 0x71c1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x70e9 as WCHAR,
        to: 0x71f4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x70eb as WCHAR,
        to: 0x71d9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x70ec as WCHAR,
        to: 0x71fc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x70ed as WCHAR,
        to: 0x71b1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7115 as WCHAR,
        to: 0x7165 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7116 as WCHAR,
        to: 0x71dc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7118 as WCHAR,
        to: 0x71fe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7231 as WCHAR,
        to: 0x611b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7237 as WCHAR,
        to: 0x723a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x724d as WCHAR,
        to: 0x7258 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7275 as WCHAR,
        to: 0x727d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x727a as WCHAR,
        to: 0x72a7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x728a as WCHAR,
        to: 0x72a2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x72b6 as WCHAR,
        to: 0x72c0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x72b7 as WCHAR,
        to: 0x7377 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x72b9 as WCHAR,
        to: 0x7336 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x72c8 as WCHAR,
        to: 0x72fd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x72de as WCHAR,
        to: 0x7370 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x72ec as WCHAR,
        to: 0x7368 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x72ed as WCHAR,
        to: 0x72f9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x72ee as WCHAR,
        to: 0x7345 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x72ef as WCHAR,
        to: 0x736a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x72f0 as WCHAR,
        to: 0x7319 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x72f1 as WCHAR,
        to: 0x7344 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x72f2 as WCHAR,
        to: 0x733b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x72f8 as WCHAR,
        to: 0x8c8d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7303 as WCHAR,
        to: 0x736b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x730e as WCHAR,
        to: 0x7375 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7315 as WCHAR,
        to: 0x737c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7321 as WCHAR,
        to: 0x7380 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x732a as WCHAR,
        to: 0x8c6c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x732b as WCHAR,
        to: 0x8c93 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x732e as WCHAR,
        to: 0x737b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x736d as WCHAR,
        to: 0x737a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7391 as WCHAR,
        to: 0x74a3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x739b as WCHAR,
        to: 0x746a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x73ae as WCHAR,
        to: 0x744b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x73af as WCHAR,
        to: 0x74b0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x73b0 as WCHAR,
        to: 0x73fe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x73ba as WCHAR,
        to: 0x74bd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x73cf as WCHAR,
        to: 0x73a8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x73d0 as WCHAR,
        to: 0x743a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x73d1 as WCHAR,
        to: 0x74cf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x73f2 as WCHAR,
        to: 0x743f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7405 as WCHAR,
        to: 0x746f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x740f as WCHAR,
        to: 0x7489 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7410 as WCHAR,
        to: 0x7463 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x743c as WCHAR,
        to: 0x74ca as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7476 as WCHAR,
        to: 0x7464 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7477 as WCHAR,
        to: 0x74a6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x748e as WCHAR,
        to: 0x74d4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x74d2 as WCHAR,
        to: 0x74da as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x74ee as WCHAR,
        to: 0x7515 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x74ef as WCHAR,
        to: 0x750c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7535 as WCHAR,
        to: 0x96fb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x753b as WCHAR,
        to: 0x756b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7545 as WCHAR,
        to: 0x66a2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7572 as WCHAR,
        to: 0x756c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7574 as WCHAR,
        to: 0x7587 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7596 as WCHAR,
        to: 0x7664 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7597 as WCHAR,
        to: 0x7642 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x759f as WCHAR,
        to: 0x7627 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x75a0 as WCHAR,
        to: 0x7658 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x75a1 as WCHAR,
        to: 0x760d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x75ae as WCHAR,
        to: 0x7621 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x75af as WCHAR,
        to: 0x760b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x75b1 as WCHAR,
        to: 0x76b0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x75c7 as WCHAR,
        to: 0x7665 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x75c8 as WCHAR,
        to: 0x7670 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x75c9 as WCHAR,
        to: 0x75d9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x75d2 as WCHAR,
        to: 0x7662 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x75e8 as WCHAR,
        to: 0x7646 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x75ea as WCHAR,
        to: 0x7613 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x75eb as WCHAR,
        to: 0x7647 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x75f4 as WCHAR,
        to: 0x7661 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7605 as WCHAR,
        to: 0x7649 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7617 as WCHAR,
        to: 0x761e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7618 as WCHAR,
        to: 0x763a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x762a as WCHAR,
        to: 0x765f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x762b as WCHAR,
        to: 0x7671 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x763e as WCHAR,
        to: 0x766e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x763f as WCHAR,
        to: 0x766d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x765e as WCHAR,
        to: 0x7669 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7663 as WCHAR,
        to: 0x766c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x766b as WCHAR,
        to: 0x7672 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7691 as WCHAR,
        to: 0x769a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x76b1 as WCHAR,
        to: 0x76ba as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x76b2 as WCHAR,
        to: 0x76b8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x76cf as WCHAR,
        to: 0x76de as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x76d0 as WCHAR,
        to: 0x9e7d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x76d1 as WCHAR,
        to: 0x76e3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x76d6 as WCHAR,
        to: 0x84cb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x76d7 as WCHAR,
        to: 0x76dc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x76d8 as WCHAR,
        to: 0x76e4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7726 as WCHAR,
        to: 0x7725 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x772f as WCHAR,
        to: 0x7787 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7740 as WCHAR,
        to: 0x8457 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7741 as WCHAR,
        to: 0x775c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7743 as WCHAR,
        to: 0x8127 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7750 as WCHAR,
        to: 0x775e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7751 as WCHAR,
        to: 0x77bc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x777e as WCHAR,
        to: 0x776a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7792 as WCHAR,
        to: 0x779e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x77a9 as WCHAR,
        to: 0x77da as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x77eb as WCHAR,
        to: 0x77ef as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x77f6 as WCHAR,
        to: 0x78ef as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x77fe as WCHAR,
        to: 0x792c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x77ff as WCHAR,
        to: 0x7926 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7800 as WCHAR,
        to: 0x78ad as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7801 as WCHAR,
        to: 0x78bc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7816 as WCHAR,
        to: 0x78da as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7817 as WCHAR,
        to: 0x7868 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x781a as WCHAR,
        to: 0x786f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x783a as WCHAR,
        to: 0x792a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x783b as WCHAR,
        to: 0x7931 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x783e as WCHAR,
        to: 0x792b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7840 as WCHAR,
        to: 0x790e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7855 as WCHAR,
        to: 0x78a9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7856 as WCHAR,
        to: 0x7864 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7857 as WCHAR,
        to: 0x78fd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x786e as WCHAR,
        to: 0x78ba as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7877 as WCHAR,
        to: 0x9e7c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x788d as WCHAR,
        to: 0x7919 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x789b as WCHAR,
        to: 0x78e7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x789c as WCHAR,
        to: 0x78e3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x78b1 as WCHAR,
        to: 0x583f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x793c as WCHAR,
        to: 0x79ae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7962 as WCHAR,
        to: 0x79b0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x796f as WCHAR,
        to: 0x798e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7977 as WCHAR,
        to: 0x79b1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7978 as WCHAR,
        to: 0x798d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7980 as WCHAR,
        to: 0x7a1f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7984 as WCHAR,
        to: 0x797f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7985 as WCHAR,
        to: 0x79aa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x79bb as WCHAR,
        to: 0x96e2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x79c3 as WCHAR,
        to: 0x79bf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x79c6 as WCHAR,
        to: 0x7a08 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x79cd as WCHAR,
        to: 0x7a2e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x79ef as WCHAR,
        to: 0x7a4d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x79f0 as WCHAR,
        to: 0x7a31 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x79fd as WCHAR,
        to: 0x7a62 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7a0e as WCHAR,
        to: 0x7a05 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7a23 as WCHAR,
        to: 0x7a4c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7a33 as WCHAR,
        to: 0x7a69 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7a51 as WCHAR,
        to: 0x7a61 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7a77 as WCHAR,
        to: 0x7aae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7a83 as WCHAR,
        to: 0x7aca as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7a8d as WCHAR,
        to: 0x7ac5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7a91 as WCHAR,
        to: 0x7aaf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7a9c as WCHAR,
        to: 0x7ac4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7a9d as WCHAR,
        to: 0x7aa9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7aa5 as WCHAR,
        to: 0x7aba as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7aa6 as WCHAR,
        to: 0x7ac7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7aad as WCHAR,
        to: 0x7ab6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ad6 as WCHAR,
        to: 0x8c4e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ade as WCHAR,
        to: 0x7af6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7b03 as WCHAR,
        to: 0x7be4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7b0b as WCHAR,
        to: 0x7b4d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7b14 as WCHAR,
        to: 0x7b46 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7b15 as WCHAR,
        to: 0x7b67 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7b3a as WCHAR,
        to: 0x7b8b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7b3c as WCHAR,
        to: 0x7c60 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7b3e as WCHAR,
        to: 0x7c69 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7b5a as WCHAR,
        to: 0x7bf3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7b5b as WCHAR,
        to: 0x7be9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7b5d as WCHAR,
        to: 0x7b8f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7b79 as WCHAR,
        to: 0x7c4c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7b7e as WCHAR,
        to: 0x7c3d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7b80 as WCHAR,
        to: 0x7c21 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ba6 as WCHAR,
        to: 0x7c00 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ba7 as WCHAR,
        to: 0x7bcb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ba8 as WCHAR,
        to: 0x7c5c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ba9 as WCHAR,
        to: 0x7c6e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7baa as WCHAR,
        to: 0x7c1e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7bab as WCHAR,
        to: 0x7c2b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7bd1 as WCHAR,
        to: 0x7c23 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7bd3 as WCHAR,
        to: 0x7c0d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7bee as WCHAR,
        to: 0x7c43 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7bf1 as WCHAR,
        to: 0x7c6c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c16 as WCHAR,
        to: 0x7c6a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c41 as WCHAR,
        to: 0x7c5f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c74 as WCHAR,
        to: 0x7cf4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c7b as WCHAR,
        to: 0x985e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c7c as WCHAR,
        to: 0x79c8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c9c as WCHAR,
        to: 0x7cf6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c9d as WCHAR,
        to: 0x7cf2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ca4 as WCHAR,
        to: 0x7cb5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7caa as WCHAR,
        to: 0x7cde as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7cae as WCHAR,
        to: 0x7ce7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7cc1 as WCHAR,
        to: 0x7cdd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d27 as WCHAR,
        to: 0x7dca as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d77 as WCHAR,
        to: 0x7e36 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ea0 as WCHAR,
        to: 0x7cfe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ea1 as WCHAR,
        to: 0x7d06 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ea2 as WCHAR,
        to: 0x7d05 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ea3 as WCHAR,
        to: 0x7d02 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ea4 as WCHAR,
        to: 0x7e96 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ea5 as WCHAR,
        to: 0x7d07 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ea6 as WCHAR,
        to: 0x7d04 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ea7 as WCHAR,
        to: 0x7d1a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ea8 as WCHAR,
        to: 0x7d08 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ea9 as WCHAR,
        to: 0x7e8a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7eaa as WCHAR,
        to: 0x7d00 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7eab as WCHAR,
        to: 0x7d09 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7eac as WCHAR,
        to: 0x7def as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ead as WCHAR,
        to: 0x7d1c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7eaf as WCHAR,
        to: 0x7d14 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7eb0 as WCHAR,
        to: 0x7d15 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7eb1 as WCHAR,
        to: 0x7d17 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7eb2 as WCHAR,
        to: 0x7db1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7eb3 as WCHAR,
        to: 0x7d0d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7eb5 as WCHAR,
        to: 0x7e31 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7eb6 as WCHAR,
        to: 0x7db8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7eb7 as WCHAR,
        to: 0x7d1b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7eb8 as WCHAR,
        to: 0x7d19 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7eb9 as WCHAR,
        to: 0x7d0b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7eba as WCHAR,
        to: 0x7d21 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ebd as WCHAR,
        to: 0x7d10 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ebe as WCHAR,
        to: 0x7d13 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ebf as WCHAR,
        to: 0x7dda as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ec0 as WCHAR,
        to: 0x7d3a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ec1 as WCHAR,
        to: 0x7d32 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ec2 as WCHAR,
        to: 0x7d31 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ec3 as WCHAR,
        to: 0x7df4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ec4 as WCHAR,
        to: 0x7d44 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ec5 as WCHAR,
        to: 0x7d33 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ec6 as WCHAR,
        to: 0x7d30 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ec7 as WCHAR,
        to: 0x7e54 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ec8 as WCHAR,
        to: 0x7d42 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ec9 as WCHAR,
        to: 0x7e10 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7eca as WCHAR,
        to: 0x7d46 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ecb as WCHAR,
        to: 0x7d3c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ecc as WCHAR,
        to: 0x7d40 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ecd as WCHAR,
        to: 0x7d39 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ece as WCHAR,
        to: 0x7e79 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ecf as WCHAR,
        to: 0x7d93 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ed0 as WCHAR,
        to: 0x7d3f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ed1 as WCHAR,
        to: 0x7d81 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ed2 as WCHAR,
        to: 0x7d68 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ed3 as WCHAR,
        to: 0x7d50 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ed5 as WCHAR,
        to: 0x7e5e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ed7 as WCHAR,
        to: 0x7d4e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ed8 as WCHAR,
        to: 0x7e6a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ed9 as WCHAR,
        to: 0x7d66 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7eda as WCHAR,
        to: 0x7d62 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7edb as WCHAR,
        to: 0x7d73 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7edc as WCHAR,
        to: 0x7d61 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7edd as WCHAR,
        to: 0x7d55 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ede as WCHAR,
        to: 0x7d5e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7edf as WCHAR,
        to: 0x7d71 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ee0 as WCHAR,
        to: 0x7d86 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ee1 as WCHAR,
        to: 0x7d83 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ee2 as WCHAR,
        to: 0x7d79 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ee3 as WCHAR,
        to: 0x7e61 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ee5 as WCHAR,
        to: 0x7d8f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ee6 as WCHAR,
        to: 0x7d73 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ee7 as WCHAR,
        to: 0x7e7c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ee8 as WCHAR,
        to: 0x7d88 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ee9 as WCHAR,
        to: 0x7e3e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7eea as WCHAR,
        to: 0x7dd2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7eeb as WCHAR,
        to: 0x7dbe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7eed as WCHAR,
        to: 0x7e8c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7eee as WCHAR,
        to: 0x7dba as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7eef as WCHAR,
        to: 0x7dcb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ef0 as WCHAR,
        to: 0x7dbd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ef2 as WCHAR,
        to: 0x7dc4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ef3 as WCHAR,
        to: 0x7e69 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ef4 as WCHAR,
        to: 0x7dad as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ef5 as WCHAR,
        to: 0x7dbf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ef6 as WCHAR,
        to: 0x7dac as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ef7 as WCHAR,
        to: 0x7e43 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ef8 as WCHAR,
        to: 0x7da2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7efa as WCHAR,
        to: 0x7db9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7efb as WCHAR,
        to: 0x7da3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7efc as WCHAR,
        to: 0x7d9c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7efd as WCHAR,
        to: 0x7dbb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7efe as WCHAR,
        to: 0x7db0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7eff as WCHAR,
        to: 0x7da0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f00 as WCHAR,
        to: 0x7db4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f01 as WCHAR,
        to: 0x7dc7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f02 as WCHAR,
        to: 0x7dd9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f03 as WCHAR,
        to: 0x7dd7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f04 as WCHAR,
        to: 0x7dd8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f05 as WCHAR,
        to: 0x7dec as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f06 as WCHAR,
        to: 0x7e9c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f07 as WCHAR,
        to: 0x7df9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f08 as WCHAR,
        to: 0x7df2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f09 as WCHAR,
        to: 0x7ddd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f0b as WCHAR,
        to: 0x7e62 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f0c as WCHAR,
        to: 0x7de6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f0d as WCHAR,
        to: 0x7d9e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f0e as WCHAR,
        to: 0x7dde as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f0f as WCHAR,
        to: 0x7df6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f11 as WCHAR,
        to: 0x7df1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f12 as WCHAR,
        to: 0x7e0b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f13 as WCHAR,
        to: 0x7de9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f14 as WCHAR,
        to: 0x7de0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f15 as WCHAR,
        to: 0x7e37 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f16 as WCHAR,
        to: 0x7de8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f17 as WCHAR,
        to: 0x7de1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f18 as WCHAR,
        to: 0x7de3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f19 as WCHAR,
        to: 0x7e09 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f1a as WCHAR,
        to: 0x7e1b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f1b as WCHAR,
        to: 0x7e1f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f1c as WCHAR,
        to: 0x7e1d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f1d as WCHAR,
        to: 0x7e2b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f1f as WCHAR,
        to: 0x7e1e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f20 as WCHAR,
        to: 0x7e8f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f21 as WCHAR,
        to: 0x7e2d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f22 as WCHAR,
        to: 0x7e0a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f23 as WCHAR,
        to: 0x7e11 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f24 as WCHAR,
        to: 0x7e7d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f25 as WCHAR,
        to: 0x7e39 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f26 as WCHAR,
        to: 0x7e35 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f27 as WCHAR,
        to: 0x7e32 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f28 as WCHAR,
        to: 0x7e93 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f29 as WCHAR,
        to: 0x7e2e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f2a as WCHAR,
        to: 0x7e46 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f2b as WCHAR,
        to: 0x7e45 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f2c as WCHAR,
        to: 0x7e88 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f2d as WCHAR,
        to: 0x7e5a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f2e as WCHAR,
        to: 0x7e55 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f2f as WCHAR,
        to: 0x7e52 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f30 as WCHAR,
        to: 0x97c1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f31 as WCHAR,
        to: 0x7e7e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f32 as WCHAR,
        to: 0x7e70 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f33 as WCHAR,
        to: 0x7e6f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f34 as WCHAR,
        to: 0x7e73 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f35 as WCHAR,
        to: 0x7e98 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f42 as WCHAR,
        to: 0x7f4c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f51 as WCHAR,
        to: 0x7db2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f57 as WCHAR,
        to: 0x7f85 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f5a as WCHAR,
        to: 0x7f70 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f62 as WCHAR,
        to: 0x7f77 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f74 as WCHAR,
        to: 0x7f86 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f81 as WCHAR,
        to: 0x7f88 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f9f as WCHAR,
        to: 0x7fa5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7fa1 as WCHAR,
        to: 0x7fa8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7fd8 as WCHAR,
        to: 0x7ff9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8027 as WCHAR,
        to: 0x802c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8038 as WCHAR,
        to: 0x8073 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x803b as WCHAR,
        to: 0x6065 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8042 as WCHAR,
        to: 0x8076 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x804b as WCHAR,
        to: 0x807e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x804c as WCHAR,
        to: 0x8077 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x804d as WCHAR,
        to: 0x8079 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8054 as WCHAR,
        to: 0x806f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8069 as WCHAR,
        to: 0x8075 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x806a as WCHAR,
        to: 0x8070 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8083 as WCHAR,
        to: 0x8085 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x80a0 as WCHAR,
        to: 0x8178 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x80a4 as WCHAR,
        to: 0x819a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x80ae as WCHAR,
        to: 0x9aaf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x80be as WCHAR,
        to: 0x814e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x80bf as WCHAR,
        to: 0x816b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x80c0 as WCHAR,
        to: 0x8139 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x80c1 as WCHAR,
        to: 0x8105 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x80c6 as WCHAR,
        to: 0x81bd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x80dc as WCHAR,
        to: 0x52dd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x80e7 as WCHAR,
        to: 0x6727 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x80ea as WCHAR,
        to: 0x81da as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x80eb as WCHAR,
        to: 0x811b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x80f6 as WCHAR,
        to: 0x81a0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8109 as WCHAR,
        to: 0x8108 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x810d as WCHAR,
        to: 0x81be as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x810f as WCHAR,
        to: 0x81df as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8110 as WCHAR,
        to: 0x81cd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8111 as WCHAR,
        to: 0x8166 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8113 as WCHAR,
        to: 0x81bf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8114 as WCHAR,
        to: 0x81e0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x811a as WCHAR,
        to: 0x8173 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8131 as WCHAR,
        to: 0x812b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8136 as WCHAR,
        to: 0x8161 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8138 as WCHAR,
        to: 0x81c9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x814a as WCHAR,
        to: 0x81d8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x817b as WCHAR,
        to: 0x81a9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x817c as WCHAR,
        to: 0x9766 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x817d as WCHAR,
        to: 0x8183 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x817e as WCHAR,
        to: 0x9a30 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8191 as WCHAR,
        to: 0x81cf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8206 as WCHAR,
        to: 0x8f3f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8223 as WCHAR,
        to: 0x8264 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8230 as WCHAR,
        to: 0x8266 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8231 as WCHAR,
        to: 0x8259 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x823b as WCHAR,
        to: 0x826b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8270 as WCHAR,
        to: 0x8271 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8273 as WCHAR,
        to: 0x8277 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8279 as WCHAR,
        to: 0x8278 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x827a as WCHAR,
        to: 0x85dd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8282 as WCHAR,
        to: 0x7bc0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8288 as WCHAR,
        to: 0x7f8b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8297 as WCHAR,
        to: 0x858c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x829c as WCHAR,
        to: 0x856a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x82a6 as WCHAR,
        to: 0x8606 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x82b8 as WCHAR,
        to: 0x8553 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x82c1 as WCHAR,
        to: 0x84ef as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x82c4 as WCHAR,
        to: 0x8290 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x82c7 as WCHAR,
        to: 0x8466 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x82c8 as WCHAR,
        to: 0x85f6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x82cb as WCHAR,
        to: 0x83a7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x82cc as WCHAR,
        to: 0x8407 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x82cd as WCHAR,
        to: 0x84bc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x82ce as WCHAR,
        to: 0x82e7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x82cf as WCHAR,
        to: 0x8607 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x82df as WCHAR,
        to: 0x830d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x82f9 as WCHAR,
        to: 0x860b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x830e as WCHAR,
        to: 0x8396 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x830f as WCHAR,
        to: 0x8622 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8311 as WCHAR,
        to: 0x8526 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8314 as WCHAR,
        to: 0x584b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8315 as WCHAR,
        to: 0x7162 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8327 as WCHAR,
        to: 0x7e6d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8346 as WCHAR,
        to: 0x834a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8350 as WCHAR,
        to: 0x85a6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x835a as WCHAR,
        to: 0x83a2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x835b as WCHAR,
        to: 0x8558 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x835c as WCHAR,
        to: 0x84fd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x835e as WCHAR,
        to: 0x854e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x835f as WCHAR,
        to: 0x8588 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8360 as WCHAR,
        to: 0x85ba as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8361 as WCHAR,
        to: 0x8569 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8363 as WCHAR,
        to: 0x69ae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8364 as WCHAR,
        to: 0x8477 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8365 as WCHAR,
        to: 0x6ece as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8366 as WCHAR,
        to: 0x7296 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8367 as WCHAR,
        to: 0x7192 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8368 as WCHAR,
        to: 0x8541 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8369 as WCHAR,
        to: 0x85ce as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x836a as WCHAR,
        to: 0x84c0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x836b as WCHAR,
        to: 0x852d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x836d as WCHAR,
        to: 0x8452 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x836f as WCHAR,
        to: 0x85e5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8385 as WCHAR,
        to: 0x849e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x83b1 as WCHAR,
        to: 0x840a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x83b2 as WCHAR,
        to: 0x84ee as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x83b3 as WCHAR,
        to: 0x8494 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x83b4 as WCHAR,
        to: 0x8435 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x83b6 as WCHAR,
        to: 0x859f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x83b7 as WCHAR,
        to: 0x7372 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x83b8 as WCHAR,
        to: 0x8555 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x83b9 as WCHAR,
        to: 0x7469 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x83ba as WCHAR,
        to: 0x9daf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x841d as WCHAR,
        to: 0x863f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8424 as WCHAR,
        to: 0x87a2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8425 as WCHAR,
        to: 0x71df as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8426 as WCHAR,
        to: 0x7e08 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8427 as WCHAR,
        to: 0x856d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8428 as WCHAR,
        to: 0x85a9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8471 as WCHAR,
        to: 0x8525 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8487 as WCHAR,
        to: 0x8546 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8489 as WCHAR,
        to: 0x8562 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x848b as WCHAR,
        to: 0x8523 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x848c as WCHAR,
        to: 0x851e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x84dd as WCHAR,
        to: 0x85cd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x84df as WCHAR,
        to: 0x858a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x84e0 as WCHAR,
        to: 0x863a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x84e3 as WCHAR,
        to: 0x8577 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x84e5 as WCHAR,
        to: 0x93a3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x84e6 as WCHAR,
        to: 0x9a40 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8537 as WCHAR,
        to: 0x8594 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8539 as WCHAR,
        to: 0x861e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x853a as WCHAR,
        to: 0x85fa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x853c as WCHAR,
        to: 0x85f9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8572 as WCHAR,
        to: 0x8604 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8574 as WCHAR,
        to: 0x860a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x85ae as WCHAR,
        to: 0x85ea as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x85d3 as WCHAR,
        to: 0x861a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8616 as WCHAR,
        to: 0x8617 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x864f as WCHAR,
        to: 0x865c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8651 as WCHAR,
        to: 0x616e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x865a as WCHAR,
        to: 0x865b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x866b as WCHAR,
        to: 0x87f2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x866e as WCHAR,
        to: 0x87e3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x867d as WCHAR,
        to: 0x96d6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x867e as WCHAR,
        to: 0x8766 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x867f as WCHAR,
        to: 0x8806 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8680 as WCHAR,
        to: 0x8755 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8681 as WCHAR,
        to: 0x87fb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8682 as WCHAR,
        to: 0x879e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8695 as WCHAR,
        to: 0x8836 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x869d as WCHAR,
        to: 0x8814 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x86ac as WCHAR,
        to: 0x8706 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x86ca as WCHAR,
        to: 0x8831 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x86ce as WCHAR,
        to: 0x8823 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x86cf as WCHAR,
        to: 0x87f6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x86ee as WCHAR,
        to: 0x883b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x86f0 as WCHAR,
        to: 0x87c4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x86f1 as WCHAR,
        to: 0x86fa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x86f2 as WCHAR,
        to: 0x87ef as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x86f3 as WCHAR,
        to: 0x8784 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x86f4 as WCHAR,
        to: 0x8810 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8715 as WCHAR,
        to: 0x86fb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8717 as WCHAR,
        to: 0x8778 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8721 as WCHAR,
        to: 0x881f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8747 as WCHAR,
        to: 0x8805 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8748 as WCHAR,
        to: 0x87c8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8749 as WCHAR,
        to: 0x87ec as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x877c as WCHAR,
        to: 0x87bb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x877e as WCHAR,
        to: 0x8811 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8845 as WCHAR,
        to: 0x91c1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8854 as WCHAR,
        to: 0x929c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8865 as WCHAR,
        to: 0x88dc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x886c as WCHAR,
        to: 0x896f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x886e as WCHAR,
        to: 0x889e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8884 as WCHAR,
        to: 0x8956 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8885 as WCHAR,
        to: 0x88ca as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x889c as WCHAR,
        to: 0x896a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x88ad as WCHAR,
        to: 0x8972 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x88c5 as WCHAR,
        to: 0x88dd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x88c6 as WCHAR,
        to: 0x8960 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x88e2 as WCHAR,
        to: 0x8933 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x88e3 as WCHAR,
        to: 0x895d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x88e4 as WCHAR,
        to: 0x8932 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x891b as WCHAR,
        to: 0x8938 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8934 as WCHAR,
        to: 0x8964 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89c1 as WCHAR,
        to: 0x898b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89c2 as WCHAR,
        to: 0x89c0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89c4 as WCHAR,
        to: 0x898f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89c5 as WCHAR,
        to: 0x8993 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89c6 as WCHAR,
        to: 0x8996 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89c7 as WCHAR,
        to: 0x8998 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89c8 as WCHAR,
        to: 0x89bd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89c9 as WCHAR,
        to: 0x89ba as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89ca as WCHAR,
        to: 0x89ac as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89cb as WCHAR,
        to: 0x89a1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89cc as WCHAR,
        to: 0x89bf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89ce as WCHAR,
        to: 0x89a6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89cf as WCHAR,
        to: 0x89af as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89d0 as WCHAR,
        to: 0x89b2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89d1 as WCHAR,
        to: 0x89b7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89de as WCHAR,
        to: 0x89f4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89e6 as WCHAR,
        to: 0x89f8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89ef as WCHAR,
        to: 0x89f6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a89 as WCHAR,
        to: 0x8b7d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a8a as WCHAR,
        to: 0x8b04 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ba1 as WCHAR,
        to: 0x8a08 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ba2 as WCHAR,
        to: 0x8a02 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ba3 as WCHAR,
        to: 0x8a03 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ba4 as WCHAR,
        to: 0x8a8d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ba5 as WCHAR,
        to: 0x8b4f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ba6 as WCHAR,
        to: 0x8a10 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ba7 as WCHAR,
        to: 0x8a0c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ba8 as WCHAR,
        to: 0x8a0e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ba9 as WCHAR,
        to: 0x8b93 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8baa as WCHAR,
        to: 0x8a15 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bab as WCHAR,
        to: 0x8a16 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bad as WCHAR,
        to: 0x8a13 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bae as WCHAR,
        to: 0x8b70 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8baf as WCHAR,
        to: 0x8a0a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bb0 as WCHAR,
        to: 0x8a18 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bb2 as WCHAR,
        to: 0x8b1b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bb3 as WCHAR,
        to: 0x8af1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bb4 as WCHAR,
        to: 0x8b33 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bb5 as WCHAR,
        to: 0x8a4e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bb6 as WCHAR,
        to: 0x8a1d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bb7 as WCHAR,
        to: 0x8a25 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bb8 as WCHAR,
        to: 0x8a31 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bb9 as WCHAR,
        to: 0x8a1b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bba as WCHAR,
        to: 0x8ad6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bbc as WCHAR,
        to: 0x8a1f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bbd as WCHAR,
        to: 0x8af7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bbe as WCHAR,
        to: 0x8a2d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bbf as WCHAR,
        to: 0x8a2a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bc0 as WCHAR,
        to: 0x8a23 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bc1 as WCHAR,
        to: 0x8b49 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bc2 as WCHAR,
        to: 0x8a41 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bc3 as WCHAR,
        to: 0x8a36 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bc4 as WCHAR,
        to: 0x8a55 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bc5 as WCHAR,
        to: 0x8a5b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bc6 as WCHAR,
        to: 0x8b58 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bc8 as WCHAR,
        to: 0x8a50 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bc9 as WCHAR,
        to: 0x8a34 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bca as WCHAR,
        to: 0x8a3a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bcb as WCHAR,
        to: 0x8a46 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bcc as WCHAR,
        to: 0x8b05 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bcd as WCHAR,
        to: 0x8a5e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bce as WCHAR,
        to: 0x8a58 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bcf as WCHAR,
        to: 0x8a54 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bd1 as WCHAR,
        to: 0x8b6f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bd2 as WCHAR,
        to: 0x8a52 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bd3 as WCHAR,
        to: 0x8a86 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bd4 as WCHAR,
        to: 0x8a84 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bd5 as WCHAR,
        to: 0x8a66 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bd6 as WCHAR,
        to: 0x8a7f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bd7 as WCHAR,
        to: 0x8a69 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bd8 as WCHAR,
        to: 0x8a70 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bd9 as WCHAR,
        to: 0x8a7c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bda as WCHAR,
        to: 0x8aa0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bdb as WCHAR,
        to: 0x8a85 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bdc as WCHAR,
        to: 0x8a75 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bdd as WCHAR,
        to: 0x8a71 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bde as WCHAR,
        to: 0x8a95 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bdf as WCHAR,
        to: 0x8a6c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8be0 as WCHAR,
        to: 0x8a6e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8be1 as WCHAR,
        to: 0x8a6d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8be2 as WCHAR,
        to: 0x8a62 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8be3 as WCHAR,
        to: 0x8a63 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8be4 as WCHAR,
        to: 0x8acd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8be5 as WCHAR,
        to: 0x8a72 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8be6 as WCHAR,
        to: 0x8a73 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8be7 as WCHAR,
        to: 0x8a6b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8be8 as WCHAR,
        to: 0x8ae2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8be9 as WCHAR,
        to: 0x8a61 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8beb as WCHAR,
        to: 0x8aa1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bec as WCHAR,
        to: 0x8aa3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bed as WCHAR,
        to: 0x8a9e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bee as WCHAR,
        to: 0x8a9a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bef as WCHAR,
        to: 0x8aa4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bf0 as WCHAR,
        to: 0x8aa5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bf1 as WCHAR,
        to: 0x8a98 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bf2 as WCHAR,
        to: 0x8aa8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bf3 as WCHAR,
        to: 0x8a91 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bf4 as WCHAR,
        to: 0x8aaa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bf5 as WCHAR,
        to: 0x8aa6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bf6 as WCHAR,
        to: 0x8a92 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bf7 as WCHAR,
        to: 0x8acb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bf8 as WCHAR,
        to: 0x8af8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bf9 as WCHAR,
        to: 0x8acf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bfa as WCHAR,
        to: 0x8afe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bfb as WCHAR,
        to: 0x8b80 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bfc as WCHAR,
        to: 0x8ad1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bfd as WCHAR,
        to: 0x8ab9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bfe as WCHAR,
        to: 0x8ab2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8bff as WCHAR,
        to: 0x8ac9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c00 as WCHAR,
        to: 0x8adb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c01 as WCHAR,
        to: 0x8ab0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c02 as WCHAR,
        to: 0x8ad7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c03 as WCHAR,
        to: 0x8abf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c04 as WCHAR,
        to: 0x8ac2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c05 as WCHAR,
        to: 0x8ad2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c06 as WCHAR,
        to: 0x8ac4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c07 as WCHAR,
        to: 0x8ab6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c08 as WCHAR,
        to: 0x8ac7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c0a as WCHAR,
        to: 0x8abc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c0b as WCHAR,
        to: 0x8b00 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c0c as WCHAR,
        to: 0x8af6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c0d as WCHAR,
        to: 0x8adc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c0e as WCHAR,
        to: 0x8b0a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c0f as WCHAR,
        to: 0x8aeb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c10 as WCHAR,
        to: 0x8ae7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c11 as WCHAR,
        to: 0x8b14 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c12 as WCHAR,
        to: 0x8b01 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c13 as WCHAR,
        to: 0x8b02 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c14 as WCHAR,
        to: 0x8ae4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c15 as WCHAR,
        to: 0x8aed as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c16 as WCHAR,
        to: 0x8afc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c17 as WCHAR,
        to: 0x8b92 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c18 as WCHAR,
        to: 0x8aee as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c19 as WCHAR,
        to: 0x8af3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c1a as WCHAR,
        to: 0x8afa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c1b as WCHAR,
        to: 0x8ae6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c1c as WCHAR,
        to: 0x8b0e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c1d as WCHAR,
        to: 0x8ade as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c1f as WCHAR,
        to: 0x8b28 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c20 as WCHAR,
        to: 0x8b9c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c21 as WCHAR,
        to: 0x8b16 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c22 as WCHAR,
        to: 0x8b1d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c23 as WCHAR,
        to: 0x8b20 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c24 as WCHAR,
        to: 0x8b17 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c25 as WCHAR,
        to: 0x8b1a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c26 as WCHAR,
        to: 0x8b19 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c27 as WCHAR,
        to: 0x8b10 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c28 as WCHAR,
        to: 0x8b39 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c29 as WCHAR,
        to: 0x8b3e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c2a as WCHAR,
        to: 0x8b2b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c2c as WCHAR,
        to: 0x8b2c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c2d as WCHAR,
        to: 0x8b5a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c2e as WCHAR,
        to: 0x8b56 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c2f as WCHAR,
        to: 0x8b59 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c30 as WCHAR,
        to: 0x8b95 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c31 as WCHAR,
        to: 0x8b5c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c32 as WCHAR,
        to: 0x8b4e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c33 as WCHAR,
        to: 0x8b9e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c34 as WCHAR,
        to: 0x8b74 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c35 as WCHAR,
        to: 0x8b6b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c36 as WCHAR,
        to: 0x8b96 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d1d as WCHAR,
        to: 0x8c9d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d1e as WCHAR,
        to: 0x8c9e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d1f as WCHAR,
        to: 0x8ca0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d21 as WCHAR,
        to: 0x8ca2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d22 as WCHAR,
        to: 0x8ca1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d23 as WCHAR,
        to: 0x8cac as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d24 as WCHAR,
        to: 0x8ce2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d25 as WCHAR,
        to: 0x6557 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d26 as WCHAR,
        to: 0x8cec as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d27 as WCHAR,
        to: 0x8ca8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d28 as WCHAR,
        to: 0x8cea as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d29 as WCHAR,
        to: 0x8ca9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d2a as WCHAR,
        to: 0x8caa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d2b as WCHAR,
        to: 0x8ca7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d2c as WCHAR,
        to: 0x8cb6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d2d as WCHAR,
        to: 0x8cfc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d2e as WCHAR,
        to: 0x8caf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d2f as WCHAR,
        to: 0x8cab as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d30 as WCHAR,
        to: 0x8cb3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d31 as WCHAR,
        to: 0x8ce4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d32 as WCHAR,
        to: 0x8cc1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d33 as WCHAR,
        to: 0x8cb0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d34 as WCHAR,
        to: 0x8cbc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d35 as WCHAR,
        to: 0x8cb4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d36 as WCHAR,
        to: 0x8cba as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d37 as WCHAR,
        to: 0x8cb8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d38 as WCHAR,
        to: 0x8cbf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d39 as WCHAR,
        to: 0x8cbb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d3a as WCHAR,
        to: 0x8cc0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d3b as WCHAR,
        to: 0x8cbd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d3c as WCHAR,
        to: 0x8cca as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d3d as WCHAR,
        to: 0x8d04 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d3e as WCHAR,
        to: 0x8cc8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d3f as WCHAR,
        to: 0x8cc4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d40 as WCHAR,
        to: 0x8cb2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d41 as WCHAR,
        to: 0x8cc3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d42 as WCHAR,
        to: 0x8cc2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d43 as WCHAR,
        to: 0x8d13 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d44 as WCHAR,
        to: 0x8cc7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d45 as WCHAR,
        to: 0x8cc5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d46 as WCHAR,
        to: 0x8d10 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d47 as WCHAR,
        to: 0x8cd5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d48 as WCHAR,
        to: 0x8cd1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d49 as WCHAR,
        to: 0x8cda as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d4a as WCHAR,
        to: 0x8cd2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d4b as WCHAR,
        to: 0x8ce6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d4c as WCHAR,
        to: 0x8ced as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d4e as WCHAR,
        to: 0x8d16 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d4f as WCHAR,
        to: 0x8cde as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d50 as WCHAR,
        to: 0x8cdc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d53 as WCHAR,
        to: 0x8ce1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d54 as WCHAR,
        to: 0x8ce0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d55 as WCHAR,
        to: 0x8ce7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d56 as WCHAR,
        to: 0x8cf4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d58 as WCHAR,
        to: 0x8d05 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d59 as WCHAR,
        to: 0x8cfb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d5a as WCHAR,
        to: 0x8cfa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d5b as WCHAR,
        to: 0x8cfd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d5c as WCHAR,
        to: 0x8cfe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d5d as WCHAR,
        to: 0x8d17 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d5e as WCHAR,
        to: 0x8d0a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d60 as WCHAR,
        to: 0x8d08 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d61 as WCHAR,
        to: 0x8d0d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d62 as WCHAR,
        to: 0x8d0f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d63 as WCHAR,
        to: 0x8d1b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d75 as WCHAR,
        to: 0x8d99 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d76 as WCHAR,
        to: 0x8d95 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d8b as WCHAR,
        to: 0x8da8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8db1 as WCHAR,
        to: 0x8db2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8db8 as WCHAR,
        to: 0x8e89 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8dc3 as WCHAR,
        to: 0x8e8d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8dc4 as WCHAR,
        to: 0x8e4c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8dde as WCHAR,
        to: 0x8e92 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8df5 as WCHAR,
        to: 0x8e10 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8df7 as WCHAR,
        to: 0x8e7a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8df8 as WCHAR,
        to: 0x8e55 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8df9 as WCHAR,
        to: 0x8e9a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8dfb as WCHAR,
        to: 0x8e8b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e0a as WCHAR,
        to: 0x8e34 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e0c as WCHAR,
        to: 0x8e8a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e2a as WCHAR,
        to: 0x8e64 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e2c as WCHAR,
        to: 0x8e93 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e2f as WCHAR,
        to: 0x8e91 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e51 as WCHAR,
        to: 0x8ea1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e52 as WCHAR,
        to: 0x8e63 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e70 as WCHAR,
        to: 0x8e95 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e7f as WCHAR,
        to: 0x8ea5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e8f as WCHAR,
        to: 0x8eaa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e9c as WCHAR,
        to: 0x8ea6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8eaf as WCHAR,
        to: 0x8ec0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f66 as WCHAR,
        to: 0x8eca as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f67 as WCHAR,
        to: 0x8ecb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f68 as WCHAR,
        to: 0x8ecc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f69 as WCHAR,
        to: 0x8ed2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f6b as WCHAR,
        to: 0x8ed4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f6c as WCHAR,
        to: 0x8f49 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f6d as WCHAR,
        to: 0x8edb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f6e as WCHAR,
        to: 0x8f2a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f6f as WCHAR,
        to: 0x8edf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f70 as WCHAR,
        to: 0x8f5f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f72 as WCHAR,
        to: 0x8efb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f73 as WCHAR,
        to: 0x8f64 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f74 as WCHAR,
        to: 0x8ef8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f75 as WCHAR,
        to: 0x8ef9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f76 as WCHAR,
        to: 0x8efc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f78 as WCHAR,
        to: 0x8eeb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f79 as WCHAR,
        to: 0x8f62 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f7a as WCHAR,
        to: 0x8efa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f7b as WCHAR,
        to: 0x8f15 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f7c as WCHAR,
        to: 0x8efe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f7d as WCHAR,
        to: 0x8f09 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f7e as WCHAR,
        to: 0x8f0a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f7f as WCHAR,
        to: 0x8f4e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f81 as WCHAR,
        to: 0x8f07 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f82 as WCHAR,
        to: 0x8f05 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f83 as WCHAR,
        to: 0x8f03 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f84 as WCHAR,
        to: 0x8f12 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f85 as WCHAR,
        to: 0x8f14 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f86 as WCHAR,
        to: 0x8f1b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f87 as WCHAR,
        to: 0x8f26 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f88 as WCHAR,
        to: 0x8f29 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f89 as WCHAR,
        to: 0x8f1d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f8a as WCHAR,
        to: 0x8f25 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f8b as WCHAR,
        to: 0x8f1e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f8d as WCHAR,
        to: 0x8f1f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f8e as WCHAR,
        to: 0x8f1c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f8f as WCHAR,
        to: 0x8f33 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f90 as WCHAR,
        to: 0x8f3b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f91 as WCHAR,
        to: 0x8f2f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f93 as WCHAR,
        to: 0x8f38 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f94 as WCHAR,
        to: 0x8f61 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f95 as WCHAR,
        to: 0x8f45 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f96 as WCHAR,
        to: 0x8f44 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f97 as WCHAR,
        to: 0x8f3e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f98 as WCHAR,
        to: 0x8f46 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f99 as WCHAR,
        to: 0x8f4d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f9a as WCHAR,
        to: 0x8f54 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f9e as WCHAR,
        to: 0x8fad as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8fa9 as WCHAR,
        to: 0x8faf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8fab as WCHAR,
        to: 0x8fae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8fb9 as WCHAR,
        to: 0x908a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8fbd as WCHAR,
        to: 0x907c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8fbe as WCHAR,
        to: 0x9054 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8fc1 as WCHAR,
        to: 0x9077 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8fc7 as WCHAR,
        to: 0x904e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8fc8 as WCHAR,
        to: 0x9081 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8fd0 as WCHAR,
        to: 0x904b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8fd8 as WCHAR,
        to: 0x9084 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8fd9 as WCHAR,
        to: 0x9019 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8fdb as WCHAR,
        to: 0x9032 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8fdc as WCHAR,
        to: 0x9060 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8fdd as WCHAR,
        to: 0x9055 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8fde as WCHAR,
        to: 0x9023 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8fdf as WCHAR,
        to: 0x9072 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8fe9 as WCHAR,
        to: 0x9087 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ff3 as WCHAR,
        to: 0x9015 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ff9 as WCHAR,
        to: 0x8de1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9002 as WCHAR,
        to: 0x9069 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9009 as WCHAR,
        to: 0x9078 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x900a as WCHAR,
        to: 0x905c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9012 as WCHAR,
        to: 0x905e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9026 as WCHAR,
        to: 0x9090 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x903b as WCHAR,
        to: 0x908f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9057 as WCHAR,
        to: 0x907a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9065 as WCHAR,
        to: 0x9059 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9093 as WCHAR,
        to: 0x9127 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x909d as WCHAR,
        to: 0x913a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x90ac as WCHAR,
        to: 0x9114 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x90ae as WCHAR,
        to: 0x90f5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x90b9 as WCHAR,
        to: 0x9112 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x90ba as WCHAR,
        to: 0x9134 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x90bb as WCHAR,
        to: 0x9130 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x90cf as WCHAR,
        to: 0x90df as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x90d0 as WCHAR,
        to: 0x9136 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x90d1 as WCHAR,
        to: 0x912d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x90d3 as WCHAR,
        to: 0x9106 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x90e6 as WCHAR,
        to: 0x9148 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x90e7 as WCHAR,
        to: 0x9116 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x90f8 as WCHAR,
        to: 0x9132 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x915d as WCHAR,
        to: 0x919e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9171 as WCHAR,
        to: 0x91ac as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x917d as WCHAR,
        to: 0x91c5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x917e as WCHAR,
        to: 0x91c3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x917f as WCHAR,
        to: 0x91c0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x91ca as WCHAR,
        to: 0x91cb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9274 as WCHAR,
        to: 0x9452 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92ae as WCHAR,
        to: 0x947e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x933e as WCHAR,
        to: 0x93e8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9486 as WCHAR,
        to: 0x91d3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9487 as WCHAR,
        to: 0x91d4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9488 as WCHAR,
        to: 0x91dd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9489 as WCHAR,
        to: 0x91d8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x948a as WCHAR,
        to: 0x91d7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x948b as WCHAR,
        to: 0x91d9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x948c as WCHAR,
        to: 0x91d5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x948d as WCHAR,
        to: 0x91f7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x948e as WCHAR,
        to: 0x91ec as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x948f as WCHAR,
        to: 0x91e7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9490 as WCHAR,
        to: 0x91e4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9492 as WCHAR,
        to: 0x91e9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9493 as WCHAR,
        to: 0x91e3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9494 as WCHAR,
        to: 0x9346 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9495 as WCHAR,
        to: 0x91f9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9497 as WCHAR,
        to: 0x91f5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9499 as WCHAR,
        to: 0x9223 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x949b as WCHAR,
        to: 0x9226 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x949c as WCHAR,
        to: 0x9245 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x949d as WCHAR,
        to: 0x920d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x949e as WCHAR,
        to: 0x9214 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x949f as WCHAR,
        to: 0x9418 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94a0 as WCHAR,
        to: 0x9209 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94a1 as WCHAR,
        to: 0x92c7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94a2 as WCHAR,
        to: 0x92fc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94a3 as WCHAR,
        to: 0x9211 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94a4 as WCHAR,
        to: 0x9210 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94a5 as WCHAR,
        to: 0x9470 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94a6 as WCHAR,
        to: 0x6b3d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94a7 as WCHAR,
        to: 0x921e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94a8 as WCHAR,
        to: 0x93a2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94a9 as WCHAR,
        to: 0x9264 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94aa as WCHAR,
        to: 0x9227 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94ab as WCHAR,
        to: 0x9201 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94ac as WCHAR,
        to: 0x9225 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94ad as WCHAR,
        to: 0x9204 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94ae as WCHAR,
        to: 0x9215 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94af as WCHAR,
        to: 0x9200 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94b0 as WCHAR,
        to: 0x923a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94b1 as WCHAR,
        to: 0x9322 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94b2 as WCHAR,
        to: 0x9266 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94b3 as WCHAR,
        to: 0x9257 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94b4 as WCHAR,
        to: 0x9237 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94b5 as WCHAR,
        to: 0x7f3d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94b6 as WCHAR,
        to: 0x9233 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94b8 as WCHAR,
        to: 0x923d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94b9 as WCHAR,
        to: 0x9238 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94ba as WCHAR,
        to: 0x925e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94bb as WCHAR,
        to: 0x9246 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94bc as WCHAR,
        to: 0x926c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94bd as WCHAR,
        to: 0x926d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94be as WCHAR,
        to: 0x9240 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94bf as WCHAR,
        to: 0x923f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94c0 as WCHAR,
        to: 0x923e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94c1 as WCHAR,
        to: 0x9435 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94c2 as WCHAR,
        to: 0x9251 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94c3 as WCHAR,
        to: 0x9234 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94c4 as WCHAR,
        to: 0x9460 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94c5 as WCHAR,
        to: 0x925b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94c6 as WCHAR,
        to: 0x925a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94c8 as WCHAR,
        to: 0x9230 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94c9 as WCHAR,
        to: 0x9249 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94ca as WCHAR,
        to: 0x9248 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94cb as WCHAR,
        to: 0x924d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94cc as WCHAR,
        to: 0x922e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94cd as WCHAR,
        to: 0x9239 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94ce as WCHAR,
        to: 0x9438 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94d0 as WCHAR,
        to: 0x92ac as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94d1 as WCHAR,
        to: 0x92a0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94d2 as WCHAR,
        to: 0x927a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94d5 as WCHAR,
        to: 0x92aa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94d6 as WCHAR,
        to: 0x92ee as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94d7 as WCHAR,
        to: 0x92cf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94d9 as WCHAR,
        to: 0x9403 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94db as WCHAR,
        to: 0x943a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94dc as WCHAR,
        to: 0x9285 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94dd as WCHAR,
        to: 0x92c1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94df as WCHAR,
        to: 0x92a6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94e0 as WCHAR,
        to: 0x93a7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94e1 as WCHAR,
        to: 0x9358 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94e2 as WCHAR,
        to: 0x9296 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94e3 as WCHAR,
        to: 0x9291 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94e4 as WCHAR,
        to: 0x92cc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94e5 as WCHAR,
        to: 0x92a9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94e7 as WCHAR,
        to: 0x93f5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94e8 as WCHAR,
        to: 0x9293 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94e9 as WCHAR,
        to: 0x93a9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94ea as WCHAR,
        to: 0x927f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94eb as WCHAR,
        to: 0x929a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94ec as WCHAR,
        to: 0x927b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94ed as WCHAR,
        to: 0x9298 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94ee as WCHAR,
        to: 0x931a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94ef as WCHAR,
        to: 0x92ab as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94f0 as WCHAR,
        to: 0x9278 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94f1 as WCHAR,
        to: 0x92a5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94f2 as WCHAR,
        to: 0x93df as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94f3 as WCHAR,
        to: 0x9283 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94f4 as WCHAR,
        to: 0x940b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94f5 as WCHAR,
        to: 0x92a8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94f6 as WCHAR,
        to: 0x9280 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94f7 as WCHAR,
        to: 0x92a3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94f8 as WCHAR,
        to: 0x9444 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94f9 as WCHAR,
        to: 0x9412 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94fa as WCHAR,
        to: 0x92ea as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94fc as WCHAR,
        to: 0x9338 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94fd as WCHAR,
        to: 0x92f1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94fe as WCHAR,
        to: 0x93c8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x94ff as WCHAR,
        to: 0x93d7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9500 as WCHAR,
        to: 0x92b7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9501 as WCHAR,
        to: 0x9396 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9502 as WCHAR,
        to: 0x92f0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9504 as WCHAR,
        to: 0x92e4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9505 as WCHAR,
        to: 0x934b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9506 as WCHAR,
        to: 0x92ef as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9507 as WCHAR,
        to: 0x92e8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9508 as WCHAR,
        to: 0x92b9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9509 as WCHAR,
        to: 0x92bc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x950a as WCHAR,
        to: 0x92dd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x950b as WCHAR,
        to: 0x92d2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x950c as WCHAR,
        to: 0x92c5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9510 as WCHAR,
        to: 0x92b3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9511 as WCHAR,
        to: 0x92bb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9512 as WCHAR,
        to: 0x92c3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9513 as WCHAR,
        to: 0x92df as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9514 as WCHAR,
        to: 0x92e6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9515 as WCHAR,
        to: 0x9312 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9516 as WCHAR,
        to: 0x9306 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9517 as WCHAR,
        to: 0x937a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9519 as WCHAR,
        to: 0x932f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x951a as WCHAR,
        to: 0x9328 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x951b as WCHAR,
        to: 0x931b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x951e as WCHAR,
        to: 0x9301 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x951f as WCHAR,
        to: 0x9315 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9521 as WCHAR,
        to: 0x932b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9522 as WCHAR,
        to: 0x932e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9523 as WCHAR,
        to: 0x947c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9524 as WCHAR,
        to: 0x9318 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9525 as WCHAR,
        to: 0x9310 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9526 as WCHAR,
        to: 0x9326 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9529 as WCHAR,
        to: 0x9308 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x952c as WCHAR,
        to: 0x931f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x952d as WCHAR,
        to: 0x9320 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x952e as WCHAR,
        to: 0x9375 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x952f as WCHAR,
        to: 0x92f8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9530 as WCHAR,
        to: 0x9333 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9531 as WCHAR,
        to: 0x9319 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9532 as WCHAR,
        to: 0x9365 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9534 as WCHAR,
        to: 0x9347 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9535 as WCHAR,
        to: 0x93d8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9536 as WCHAR,
        to: 0x9376 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9537 as WCHAR,
        to: 0x9354 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9538 as WCHAR,
        to: 0x9364 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9539 as WCHAR,
        to: 0x936c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x953a as WCHAR,
        to: 0x937e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x953b as WCHAR,
        to: 0x935b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x953c as WCHAR,
        to: 0x93aa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x953e as WCHAR,
        to: 0x9370 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9540 as WCHAR,
        to: 0x934d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9541 as WCHAR,
        to: 0x9382 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9542 as WCHAR,
        to: 0x93e4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9544 as WCHAR,
        to: 0x9428 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9546 as WCHAR,
        to: 0x93cc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9547 as WCHAR,
        to: 0x93ae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9549 as WCHAR,
        to: 0x9398 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x954a as WCHAR,
        to: 0x9477 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x954c as WCHAR,
        to: 0x942b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x954d as WCHAR,
        to: 0x93b3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x954f as WCHAR,
        to: 0x93a6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9550 as WCHAR,
        to: 0x93ac as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9551 as WCHAR,
        to: 0x938a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9552 as WCHAR,
        to: 0x93b0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9553 as WCHAR,
        to: 0x93b5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9554 as WCHAR,
        to: 0x944c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9556 as WCHAR,
        to: 0x93e2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9557 as WCHAR,
        to: 0x93dc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9558 as WCHAR,
        to: 0x93dd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9559 as WCHAR,
        to: 0x93cd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x955b as WCHAR,
        to: 0x93de as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x955c as WCHAR,
        to: 0x93e1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x955d as WCHAR,
        to: 0x93d1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x955e as WCHAR,
        to: 0x93c3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x955f as WCHAR,
        to: 0x93c7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9561 as WCHAR,
        to: 0x9414 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9563 as WCHAR,
        to: 0x9410 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9564 as WCHAR,
        to: 0x93f7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9566 as WCHAR,
        to: 0x9413 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9567 as WCHAR,
        to: 0x946d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9568 as WCHAR,
        to: 0x9420 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x956a as WCHAR,
        to: 0x93f9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x956b as WCHAR,
        to: 0x9419 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x956c as WCHAR,
        to: 0x944a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x956d as WCHAR,
        to: 0x9433 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x956f as WCHAR,
        to: 0x9432 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9570 as WCHAR,
        to: 0x942e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9571 as WCHAR,
        to: 0x943f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9573 as WCHAR,
        to: 0x9463 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9576 as WCHAR,
        to: 0x9472 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x957f as WCHAR,
        to: 0x9577 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95e8 as WCHAR,
        to: 0x9580 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95e9 as WCHAR,
        to: 0x9582 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95ea as WCHAR,
        to: 0x9583 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95eb as WCHAR,
        to: 0x9586 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95ed as WCHAR,
        to: 0x9589 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95ee as WCHAR,
        to: 0x554f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95ef as WCHAR,
        to: 0x95d6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95f0 as WCHAR,
        to: 0x958f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95f1 as WCHAR,
        to: 0x95c8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95f2 as WCHAR,
        to: 0x9591 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95f3 as WCHAR,
        to: 0x958e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95f4 as WCHAR,
        to: 0x9593 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95f5 as WCHAR,
        to: 0x9594 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95f6 as WCHAR,
        to: 0x958c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95f7 as WCHAR,
        to: 0x60b6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95f8 as WCHAR,
        to: 0x9598 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95f9 as WCHAR,
        to: 0x9b27 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95fa as WCHAR,
        to: 0x95a8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95fb as WCHAR,
        to: 0x805e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95fc as WCHAR,
        to: 0x95e5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95fd as WCHAR,
        to: 0x95a9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95fe as WCHAR,
        to: 0x95ad as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9600 as WCHAR,
        to: 0x95a5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9601 as WCHAR,
        to: 0x95a3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9602 as WCHAR,
        to: 0x95a1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9603 as WCHAR,
        to: 0x95ab as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9604 as WCHAR,
        to: 0x9b2e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9605 as WCHAR,
        to: 0x95b1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9606 as WCHAR,
        to: 0x95ac as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9608 as WCHAR,
        to: 0x95be as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9609 as WCHAR,
        to: 0x95b9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x960a as WCHAR,
        to: 0x95b6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x960b as WCHAR,
        to: 0x9b29 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x960c as WCHAR,
        to: 0x95bf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x960d as WCHAR,
        to: 0x95bd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x960e as WCHAR,
        to: 0x95bb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x960f as WCHAR,
        to: 0x95bc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9610 as WCHAR,
        to: 0x95e1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9611 as WCHAR,
        to: 0x95cc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9612 as WCHAR,
        to: 0x95c3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9614 as WCHAR,
        to: 0x95ca as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9615 as WCHAR,
        to: 0x95cb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9616 as WCHAR,
        to: 0x95d4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9617 as WCHAR,
        to: 0x95d0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9619 as WCHAR,
        to: 0x95d5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x961a as WCHAR,
        to: 0x95de as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x961f as WCHAR,
        to: 0x968a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9633 as WCHAR,
        to: 0x967d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9634 as WCHAR,
        to: 0x9670 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9635 as WCHAR,
        to: 0x9663 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9636 as WCHAR,
        to: 0x968e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9645 as WCHAR,
        to: 0x969b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9646 as WCHAR,
        to: 0x9678 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9647 as WCHAR,
        to: 0x96b4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9648 as WCHAR,
        to: 0x9673 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9649 as WCHAR,
        to: 0x9658 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9655 as WCHAR,
        to: 0x965c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9667 as WCHAR,
        to: 0x9689 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9668 as WCHAR,
        to: 0x9695 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9669 as WCHAR,
        to: 0x96aa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x968f as WCHAR,
        to: 0x96a8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9690 as WCHAR,
        to: 0x96b1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x96b6 as WCHAR,
        to: 0x96b8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x96bd as WCHAR,
        to: 0x96cb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x96be as WCHAR,
        to: 0x96e3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x96cf as WCHAR,
        to: 0x96db as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x96e0 as WCHAR,
        to: 0x8b8e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x96f3 as WCHAR,
        to: 0x9742 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x96fe as WCHAR,
        to: 0x9727 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9701 as WCHAR,
        to: 0x973d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x972d as WCHAR,
        to: 0x9744 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9753 as WCHAR,
        to: 0x975a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9759 as WCHAR,
        to: 0x975c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9765 as WCHAR,
        to: 0x9768 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9791 as WCHAR,
        to: 0x97c3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x97af as WCHAR,
        to: 0x97c9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x97e6 as WCHAR,
        to: 0x97cb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x97e7 as WCHAR,
        to: 0x97cc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x97e9 as WCHAR,
        to: 0x97d3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x97ea as WCHAR,
        to: 0x97d9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x97eb as WCHAR,
        to: 0x97de as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x97ec as WCHAR,
        to: 0x97dc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x97f5 as WCHAR,
        to: 0x97fb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9875 as WCHAR,
        to: 0x9801 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9876 as WCHAR,
        to: 0x9802 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9877 as WCHAR,
        to: 0x9803 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9878 as WCHAR,
        to: 0x9807 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9879 as WCHAR,
        to: 0x9805 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x987a as WCHAR,
        to: 0x9806 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x987b as WCHAR,
        to: 0x9808 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x987c as WCHAR,
        to: 0x980a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x987d as WCHAR,
        to: 0x9811 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x987e as WCHAR,
        to: 0x9867 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x987f as WCHAR,
        to: 0x9813 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9880 as WCHAR,
        to: 0x980e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9881 as WCHAR,
        to: 0x9812 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9882 as WCHAR,
        to: 0x980c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9883 as WCHAR,
        to: 0x980f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9884 as WCHAR,
        to: 0x9810 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9885 as WCHAR,
        to: 0x9871 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9886 as WCHAR,
        to: 0x9818 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9887 as WCHAR,
        to: 0x9817 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9888 as WCHAR,
        to: 0x9838 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9889 as WCHAR,
        to: 0x9821 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x988a as WCHAR,
        to: 0x9830 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x988c as WCHAR,
        to: 0x981c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x988d as WCHAR,
        to: 0x6f41 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x988f as WCHAR,
        to: 0x9826 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9890 as WCHAR,
        to: 0x9824 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9891 as WCHAR,
        to: 0x983b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9893 as WCHAR,
        to: 0x9839 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9894 as WCHAR,
        to: 0x9837 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9896 as WCHAR,
        to: 0x7a4e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9897 as WCHAR,
        to: 0x9846 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9898 as WCHAR,
        to: 0x984c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x989a as WCHAR,
        to: 0x984e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x989b as WCHAR,
        to: 0x9853 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x989c as WCHAR,
        to: 0x984f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x989d as WCHAR,
        to: 0x984d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x989e as WCHAR,
        to: 0x9873 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x989f as WCHAR,
        to: 0x9862 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98a0 as WCHAR,
        to: 0x985b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98a1 as WCHAR,
        to: 0x9859 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98a2 as WCHAR,
        to: 0x9865 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98a4 as WCHAR,
        to: 0x986b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98a6 as WCHAR,
        to: 0x9870 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98a7 as WCHAR,
        to: 0x9874 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98ce as WCHAR,
        to: 0x98a8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98d1 as WCHAR,
        to: 0x98ae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98d2 as WCHAR,
        to: 0x98af as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98d3 as WCHAR,
        to: 0x98b6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98d5 as WCHAR,
        to: 0x98bc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98d8 as WCHAR,
        to: 0x98c4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98d9 as WCHAR,
        to: 0x98c6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98de as WCHAR,
        to: 0x98db as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98e8 as WCHAR,
        to: 0x9957 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x990d as WCHAR,
        to: 0x995c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9965 as WCHAR,
        to: 0x9951 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9967 as WCHAR,
        to: 0x9933 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9968 as WCHAR,
        to: 0x98e9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9969 as WCHAR,
        to: 0x993c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x996a as WCHAR,
        to: 0x98ea as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x996b as WCHAR,
        to: 0x98eb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x996c as WCHAR,
        to: 0x98ed as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x996d as WCHAR,
        to: 0x98ef as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x996e as WCHAR,
        to: 0x98f2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x996f as WCHAR,
        to: 0x991e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9970 as WCHAR,
        to: 0x98fe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9971 as WCHAR,
        to: 0x98fd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9972 as WCHAR,
        to: 0x98fc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9974 as WCHAR,
        to: 0x98f4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9975 as WCHAR,
        to: 0x990c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9976 as WCHAR,
        to: 0x9952 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9977 as WCHAR,
        to: 0x9909 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x997a as WCHAR,
        to: 0x9903 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x997c as WCHAR,
        to: 0x9905 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x997d as WCHAR,
        to: 0x9911 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x997f as WCHAR,
        to: 0x9913 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9981 as WCHAR,
        to: 0x9912 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9984 as WCHAR,
        to: 0x991b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9985 as WCHAR,
        to: 0x9921 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9986 as WCHAR,
        to: 0x9928 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9988 as WCHAR,
        to: 0x994b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x998a as WCHAR,
        to: 0x993f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x998b as WCHAR,
        to: 0x995e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x998d as WCHAR,
        to: 0x9943 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x998f as WCHAR,
        to: 0x993e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9990 as WCHAR,
        to: 0x9948 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9991 as WCHAR,
        to: 0x9949 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9992 as WCHAR,
        to: 0x9945 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9994 as WCHAR,
        to: 0x994c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a6c as WCHAR,
        to: 0x99ac as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a6d as WCHAR,
        to: 0x99ad as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a6e as WCHAR,
        to: 0x99b1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a6f as WCHAR,
        to: 0x99b4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a70 as WCHAR,
        to: 0x99b3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a71 as WCHAR,
        to: 0x9a45 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a73 as WCHAR,
        to: 0x99c1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a74 as WCHAR,
        to: 0x9a62 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a75 as WCHAR,
        to: 0x99d4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a76 as WCHAR,
        to: 0x99db as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a77 as WCHAR,
        to: 0x99df as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a78 as WCHAR,
        to: 0x99d9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a79 as WCHAR,
        to: 0x99d2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a7a as WCHAR,
        to: 0x9a36 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a7b as WCHAR,
        to: 0x99d0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a7c as WCHAR,
        to: 0x99dd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a7d as WCHAR,
        to: 0x99d1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a7e as WCHAR,
        to: 0x99d5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a7f as WCHAR,
        to: 0x9a5b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a80 as WCHAR,
        to: 0x99d8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a81 as WCHAR,
        to: 0x9a4d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a82 as WCHAR,
        to: 0x7f75 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a84 as WCHAR,
        to: 0x9a55 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a85 as WCHAR,
        to: 0x9a4a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a86 as WCHAR,
        to: 0x99f1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a87 as WCHAR,
        to: 0x99ed as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a88 as WCHAR,
        to: 0x99e2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a8a as WCHAR,
        to: 0x9a6a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a8b as WCHAR,
        to: 0x9a01 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a8c as WCHAR,
        to: 0x9a57 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a8f as WCHAR,
        to: 0x99ff as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a90 as WCHAR,
        to: 0x9a0f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a91 as WCHAR,
        to: 0x9a0e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a92 as WCHAR,
        to: 0x9a0d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a93 as WCHAR,
        to: 0x9a05 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a96 as WCHAR,
        to: 0x9a42 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a97 as WCHAR,
        to: 0x9a19 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a98 as WCHAR,
        to: 0x9a2d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a9a as WCHAR,
        to: 0x9a37 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a9b as WCHAR,
        to: 0x9a16 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a9c as WCHAR,
        to: 0x9a41 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a9d as WCHAR,
        to: 0x9a2e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a9e as WCHAR,
        to: 0x9a2b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a9f as WCHAR,
        to: 0x9a38 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9aa0 as WCHAR,
        to: 0x9a43 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9aa1 as WCHAR,
        to: 0x9a3e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9aa2 as WCHAR,
        to: 0x9a44 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9aa3 as WCHAR,
        to: 0x9a4f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9aa4 as WCHAR,
        to: 0x9a5f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9aa5 as WCHAR,
        to: 0x9a65 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9aa7 as WCHAR,
        to: 0x9a64 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ac5 as WCHAR,
        to: 0x9acf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9acb as WCHAR,
        to: 0x9ad6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9acc as WCHAR,
        to: 0x9ad5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9b13 as WCHAR,
        to: 0x9b22 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9b47 as WCHAR,
        to: 0x9b58 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9b49 as WCHAR,
        to: 0x9b4e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c7c as WCHAR,
        to: 0x9b5a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c7f as WCHAR,
        to: 0x9b77 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c81 as WCHAR,
        to: 0x9b6f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c82 as WCHAR,
        to: 0x9b74 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c88 as WCHAR,
        to: 0x9c78 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c8b as WCHAR,
        to: 0x9b92 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c8d as WCHAR,
        to: 0x9b91 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c8e as WCHAR,
        to: 0x9c5f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c90 as WCHAR,
        to: 0x9b90 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c91 as WCHAR,
        to: 0x9bad as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c92 as WCHAR,
        to: 0x9b9a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c94 as WCHAR,
        to: 0x9baa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c95 as WCHAR,
        to: 0x9b9e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c9a as WCHAR,
        to: 0x9c6d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c9b as WCHAR,
        to: 0x9bab as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c9c as WCHAR,
        to: 0x9bae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c9f as WCHAR,
        to: 0x9c58 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ca0 as WCHAR,
        to: 0x9bc1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ca1 as WCHAR,
        to: 0x9c7a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ca2 as WCHAR,
        to: 0x9c31 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ca3 as WCHAR,
        to: 0x9c39 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ca4 as WCHAR,
        to: 0x9bc9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ca5 as WCHAR,
        to: 0x9c23 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ca6 as WCHAR,
        to: 0x9c37 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ca7 as WCHAR,
        to: 0x9bc0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ca8 as WCHAR,
        to: 0x9bca as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ca9 as WCHAR,
        to: 0x9bc7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cab as WCHAR,
        to: 0x9bfd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cad as WCHAR,
        to: 0x9bd6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cae as WCHAR,
        to: 0x9bea as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cb0 as WCHAR,
        to: 0x9beb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cb1 as WCHAR,
        to: 0x9be1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cb2 as WCHAR,
        to: 0x9be4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cb3 as WCHAR,
        to: 0x9be7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cb5 as WCHAR,
        to: 0x9be2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cb6 as WCHAR,
        to: 0x9bf0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cb7 as WCHAR,
        to: 0x9bdb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cb8 as WCHAR,
        to: 0x9be8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cbb as WCHAR,
        to: 0x9bd4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cbd as WCHAR,
        to: 0x9c08 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cc3 as WCHAR,
        to: 0x9c13 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cc4 as WCHAR,
        to: 0x9c77 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cc5 as WCHAR,
        to: 0x9c0d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cc6 as WCHAR,
        to: 0x9c12 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cc7 as WCHAR,
        to: 0x9c09 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ccc as WCHAR,
        to: 0x9c32 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ccd as WCHAR,
        to: 0x9c2d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cce as WCHAR,
        to: 0x9c28 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ccf as WCHAR,
        to: 0x9c25 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cd0 as WCHAR,
        to: 0x9c29 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cd3 as WCHAR,
        to: 0x9c33 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cd4 as WCHAR,
        to: 0x9c3e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cd5 as WCHAR,
        to: 0x9c48 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cd6 as WCHAR,
        to: 0x9c49 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cd7 as WCHAR,
        to: 0x9c3b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cdc as WCHAR,
        to: 0x9c56 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cdd as WCHAR,
        to: 0x9c54 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cde as WCHAR,
        to: 0x9c57 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cdf as WCHAR,
        to: 0x9c52 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ce2 as WCHAR,
        to: 0x9c67 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e1f as WCHAR,
        to: 0x9ce5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e20 as WCHAR,
        to: 0x9ce9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e21 as WCHAR,
        to: 0x96de as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e22 as WCHAR,
        to: 0x9cf6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e23 as WCHAR,
        to: 0x9cf4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e25 as WCHAR,
        to: 0x9dd7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e26 as WCHAR,
        to: 0x9d09 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e28 as WCHAR,
        to: 0x9d07 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e29 as WCHAR,
        to: 0x9d06 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e2a as WCHAR,
        to: 0x9d23 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e2b as WCHAR,
        to: 0x9d87 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e2c as WCHAR,
        to: 0x9e15 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e2d as WCHAR,
        to: 0x9d28 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e2f as WCHAR,
        to: 0x9d26 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e31 as WCHAR,
        to: 0x9d1f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e32 as WCHAR,
        to: 0x9d1d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e33 as WCHAR,
        to: 0x9d1b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e35 as WCHAR,
        to: 0x9d15 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e36 as WCHAR,
        to: 0x9de5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e37 as WCHAR,
        to: 0x9dd9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e38 as WCHAR,
        to: 0x9d2f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e39 as WCHAR,
        to: 0x9d30 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e3a as WCHAR,
        to: 0x9d42 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e3d as WCHAR,
        to: 0x9d3f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e3e as WCHAR,
        to: 0x9e1e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e3f as WCHAR,
        to: 0x9d3b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e41 as WCHAR,
        to: 0x9d53 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e42 as WCHAR,
        to: 0x9e1d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e43 as WCHAR,
        to: 0x9d51 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e44 as WCHAR,
        to: 0x9d60 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e45 as WCHAR,
        to: 0x9d5d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e46 as WCHAR,
        to: 0x9d52 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e47 as WCHAR,
        to: 0x9df4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e48 as WCHAR,
        to: 0x9d5c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e49 as WCHAR,
        to: 0x9d61 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e4a as WCHAR,
        to: 0x9d72 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e4c as WCHAR,
        to: 0x9d6a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e4e as WCHAR,
        to: 0x9d6f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e4f as WCHAR,
        to: 0x9d6c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e51 as WCHAR,
        to: 0x9d89 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e55 as WCHAR,
        to: 0x9d98 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e57 as WCHAR,
        to: 0x9d9a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e58 as WCHAR,
        to: 0x9dbb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e5a as WCHAR,
        to: 0x9dbf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e5c as WCHAR,
        to: 0x9da9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e5e as WCHAR,
        to: 0x9dc2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e63 as WCHAR,
        to: 0x9dbc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e64 as WCHAR,
        to: 0x9db4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e66 as WCHAR,
        to: 0x9e1a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e67 as WCHAR,
        to: 0x9dd3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e68 as WCHAR,
        to: 0x9dda as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e69 as WCHAR,
        to: 0x9def as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e6a as WCHAR,
        to: 0x9de6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e6b as WCHAR,
        to: 0x9df2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e6c as WCHAR,
        to: 0x9df8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e6d as WCHAR,
        to: 0x9dfa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e70 as WCHAR,
        to: 0x9df9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e73 as WCHAR,
        to: 0x9e1b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e7e as WCHAR,
        to: 0x9e7a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ea6 as WCHAR,
        to: 0x9ea5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9eb8 as WCHAR,
        to: 0x9ea9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ebd as WCHAR,
        to: 0x9ebc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ec4 as WCHAR,
        to: 0x9ec3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ec9 as WCHAR,
        to: 0x9ecc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ee9 as WCHAR,
        to: 0x9ef7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9eea as WCHAR,
        to: 0x9ef2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9efe as WCHAR,
        to: 0x9efd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f0b as WCHAR,
        to: 0x9eff as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f0d as WCHAR,
        to: 0x9f09 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f39 as WCHAR,
        to: 0x9f34 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f50 as WCHAR,
        to: 0x9f4a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f51 as WCHAR,
        to: 0x9f4f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f7f as WCHAR,
        to: 0x9f52 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f80 as WCHAR,
        to: 0x9f54 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f83 as WCHAR,
        to: 0x9f5f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f84 as WCHAR,
        to: 0x9f61 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f85 as WCHAR,
        to: 0x9f59 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f86 as WCHAR,
        to: 0x9f60 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f87 as WCHAR,
        to: 0x9f5c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f88 as WCHAR,
        to: 0x9f66 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f89 as WCHAR,
        to: 0x9f6c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f8a as WCHAR,
        to: 0x9f6a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f8b as WCHAR,
        to: 0x9f72 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f8c as WCHAR,
        to: 0x9f77 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f99 as WCHAR,
        to: 0x9f8d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f9a as WCHAR,
        to: 0x9f94 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f9b as WCHAR,
        to: 0x9f95 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f9f as WCHAR,
        to: 0x9f9c as WCHAR,
    },
];

static mut s_tra2sim: [WCHAR_PAIR; 2619] = [
    tagWCHAR_PAIR {
        from: 0x4e1f as WCHAR,
        to: 0x4e22 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e26 as WCHAR,
        to: 0x5e76 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e82 as WCHAR,
        to: 0x4e71 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e99 as WCHAR,
        to: 0x4e98 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4e9e as WCHAR,
        to: 0x4e9a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4f0b as WCHAR,
        to: 0x6c72 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4f15 as WCHAR,
        to: 0x592b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4f47 as WCHAR,
        to: 0x4f2b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4f48 as WCHAR,
        to: 0x5e03 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4f54 as WCHAR,
        to: 0x5360 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4f6a as WCHAR,
        to: 0x5f8a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4f75 as WCHAR,
        to: 0x5e76 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4f86 as WCHAR,
        to: 0x6765 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4f96 as WCHAR,
        to: 0x4ed1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4fb6 as WCHAR,
        to: 0x4fa3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4fb7 as WCHAR,
        to: 0x5c40 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4fc1 as WCHAR,
        to: 0x4fe3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4fc2 as WCHAR,
        to: 0x7cfb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x4fe0 as WCHAR,
        to: 0x4fa0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5000 as WCHAR,
        to: 0x4f25 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5006 as WCHAR,
        to: 0x4fe9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5009 as WCHAR,
        to: 0x4ed3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x500b as WCHAR,
        to: 0x4e2a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5011 as WCHAR,
        to: 0x4eec as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5016 as WCHAR,
        to: 0x5e78 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5023 as WCHAR,
        to: 0x4eff as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x502b as WCHAR,
        to: 0x4f26 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5049 as WCHAR,
        to: 0x4f1f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5074 as WCHAR,
        to: 0x4fa7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5075 as WCHAR,
        to: 0x4fa6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x507a as WCHAR,
        to: 0x54b1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x507d as WCHAR,
        to: 0x4f2a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5091 as WCHAR,
        to: 0x6770 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5096 as WCHAR,
        to: 0x4f27 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5098 as WCHAR,
        to: 0x4f1e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5099 as WCHAR,
        to: 0x5907 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x50a2 as WCHAR,
        to: 0x5bb6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x50ad as WCHAR,
        to: 0x4f63 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x50af as WCHAR,
        to: 0x506c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x50b3 as WCHAR,
        to: 0x4f20 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x50b4 as WCHAR,
        to: 0x4f1b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x50b5 as WCHAR,
        to: 0x503a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x50b7 as WCHAR,
        to: 0x4f24 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x50be as WCHAR,
        to: 0x503e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x50c2 as WCHAR,
        to: 0x507b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x50c5 as WCHAR,
        to: 0x4ec5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x50c7 as WCHAR,
        to: 0x622e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x50c9 as WCHAR,
        to: 0x4f65 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x50d1 as WCHAR,
        to: 0x4fa8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x50d5 as WCHAR,
        to: 0x4ec6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x50e5 as WCHAR,
        to: 0x4fa5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x50e8 as WCHAR,
        to: 0x507e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x50f1 as WCHAR,
        to: 0x96c7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x50f9 as WCHAR,
        to: 0x4ef7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5100 as WCHAR,
        to: 0x4eea as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5102 as WCHAR,
        to: 0x4fac as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5104 as WCHAR,
        to: 0x4ebf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5105 as WCHAR,
        to: 0x5f53 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5108 as WCHAR,
        to: 0x4fa9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5109 as WCHAR,
        to: 0x4fed as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5110 as WCHAR,
        to: 0x50a7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5114 as WCHAR,
        to: 0x4fe6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5115 as WCHAR,
        to: 0x4faa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5118 as WCHAR,
        to: 0x5c3d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x511f as WCHAR,
        to: 0x507f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x512a as WCHAR,
        to: 0x4f18 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5132 as WCHAR,
        to: 0x50a8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5137 as WCHAR,
        to: 0x4fea as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5138 as WCHAR,
        to: 0x7f57 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x513a as WCHAR,
        to: 0x50a9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x513b as WCHAR,
        to: 0x50a5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x513c as WCHAR,
        to: 0x4fe8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5147 as WCHAR,
        to: 0x51f6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x514c as WCHAR,
        to: 0x5151 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5152 as WCHAR,
        to: 0x513f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5157 as WCHAR,
        to: 0x5156 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5167 as WCHAR,
        to: 0x5185 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5169 as WCHAR,
        to: 0x4e24 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x518a as WCHAR,
        to: 0x518c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5191 as WCHAR,
        to: 0x80c4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x51aa as WCHAR,
        to: 0x5e42 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x51c8 as WCHAR,
        to: 0x51c0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x51cd as WCHAR,
        to: 0x51bb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x51dc as WCHAR,
        to: 0x51db as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x51f1 as WCHAR,
        to: 0x51ef as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5225 as WCHAR,
        to: 0x522b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x522a as WCHAR,
        to: 0x5220 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5244 as WCHAR,
        to: 0x522d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5247 as WCHAR,
        to: 0x5219 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x524b as WCHAR,
        to: 0x514b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x524e as WCHAR,
        to: 0x5239 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x525b as WCHAR,
        to: 0x521a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x525d as WCHAR,
        to: 0x5265 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x526e as WCHAR,
        to: 0x5250 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5274 as WCHAR,
        to: 0x5240 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5275 as WCHAR,
        to: 0x521b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5277 as WCHAR,
        to: 0x94f2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5283 as WCHAR,
        to: 0x5212 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5287 as WCHAR,
        to: 0x5267 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5289 as WCHAR,
        to: 0x5218 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x528a as WCHAR,
        to: 0x523d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x528c as WCHAR,
        to: 0x523f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x528d as WCHAR,
        to: 0x5251 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5291 as WCHAR,
        to: 0x5242 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x52bb as WCHAR,
        to: 0x5321 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x52c1 as WCHAR,
        to: 0x52b2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x52d5 as WCHAR,
        to: 0x52a8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x52d7 as WCHAR,
        to: 0x52d6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x52d9 as WCHAR,
        to: 0x52a1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x52db as WCHAR,
        to: 0x52cb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x52dd as WCHAR,
        to: 0x80dc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x52de as WCHAR,
        to: 0x52b3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x52e2 as WCHAR,
        to: 0x52bf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x52e3 as WCHAR,
        to: 0x79ef as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x52e6 as WCHAR,
        to: 0x527f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x52f1 as WCHAR,
        to: 0x52a2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x52f5 as WCHAR,
        to: 0x52b1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x52f8 as WCHAR,
        to: 0x529d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x52fb as WCHAR,
        to: 0x5300 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x532d as WCHAR,
        to: 0x5326 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x532f as WCHAR,
        to: 0x6c47 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5331 as WCHAR,
        to: 0x532e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5340 as WCHAR,
        to: 0x533a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5354 as WCHAR,
        to: 0x534f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5379 as WCHAR,
        to: 0x6064 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x537b as WCHAR,
        to: 0x5374 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5399 as WCHAR,
        to: 0x538d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x53ad as WCHAR,
        to: 0x538c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x53b2 as WCHAR,
        to: 0x5389 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x53b4 as WCHAR,
        to: 0x53a3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x53c3 as WCHAR,
        to: 0x53c2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x53e2 as WCHAR,
        to: 0x4e1b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x540b as WCHAR,
        to: 0x5bf8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5433 as WCHAR,
        to: 0x5434 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5436 as WCHAR,
        to: 0x5450 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5442 as WCHAR,
        to: 0x5415 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x544e as WCHAR,
        to: 0x5c3a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x54bc as WCHAR,
        to: 0x5459 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x54e1 as WCHAR,
        to: 0x5458 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5504 as WCHAR,
        to: 0x5457 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5538 as WCHAR,
        to: 0x5ff5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x554f as WCHAR,
        to: 0x95ee as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x555e as WCHAR,
        to: 0x54d1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x555f as WCHAR,
        to: 0x542f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5563 as WCHAR,
        to: 0x8854 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x559a as WCHAR,
        to: 0x5524 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x55aa as WCHAR,
        to: 0x4e27 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x55ab as WCHAR,
        to: 0x5403 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x55ac as WCHAR,
        to: 0x4e54 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x55ae as WCHAR,
        to: 0x5355 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x55b2 as WCHAR,
        to: 0x54df as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x55c6 as WCHAR,
        to: 0x545b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x55c7 as WCHAR,
        to: 0x556c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x55ce as WCHAR,
        to: 0x5417 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x55da as WCHAR,
        to: 0x545c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x55e9 as WCHAR,
        to: 0x5522 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x55f6 as WCHAR,
        to: 0x54d4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5606 as WCHAR,
        to: 0x53f9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x560d as WCHAR,
        to: 0x55bd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5614 as WCHAR,
        to: 0x5455 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5616 as WCHAR,
        to: 0x5567 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5617 as WCHAR,
        to: 0x5c1d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x561c as WCHAR,
        to: 0x551b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5629 as WCHAR,
        to: 0x54d7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x562e as WCHAR,
        to: 0x5520 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x562f as WCHAR,
        to: 0x5578 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5630 as WCHAR,
        to: 0x53fd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5635 as WCHAR,
        to: 0x54d3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5638 as WCHAR,
        to: 0x5452 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5641 as WCHAR,
        to: 0x6076 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5653 as WCHAR,
        to: 0x5618 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5660 as WCHAR,
        to: 0x54d2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5665 as WCHAR,
        to: 0x54dd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5666 as WCHAR,
        to: 0x54d5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x566f as WCHAR,
        to: 0x55f3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5672 as WCHAR,
        to: 0x54d9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5674 as WCHAR,
        to: 0x55b7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5678 as WCHAR,
        to: 0x5428 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5679 as WCHAR,
        to: 0x5f53 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5680 as WCHAR,
        to: 0x549b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5687 as WCHAR,
        to: 0x5413 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x568c as WCHAR,
        to: 0x54dc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5695 as WCHAR,
        to: 0x565c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5699 as WCHAR,
        to: 0x556e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x56a5 as WCHAR,
        to: 0x54bd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x56a6 as WCHAR,
        to: 0x5456 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x56a8 as WCHAR,
        to: 0x5499 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x56ae as WCHAR,
        to: 0x5411 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x56b3 as WCHAR,
        to: 0x55be as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x56b4 as WCHAR,
        to: 0x4e25 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x56b6 as WCHAR,
        to: 0x5624 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x56c0 as WCHAR,
        to: 0x556d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x56c1 as WCHAR,
        to: 0x55eb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x56c2 as WCHAR,
        to: 0x56a3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x56c5 as WCHAR,
        to: 0x5181 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x56c8 as WCHAR,
        to: 0x5453 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x56c9 as WCHAR,
        to: 0x7f57 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x56cc as WCHAR,
        to: 0x82cf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x56d1 as WCHAR,
        to: 0x5631 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x56ea as WCHAR,
        to: 0x56f1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5707 as WCHAR,
        to: 0x56f5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x570b as WCHAR,
        to: 0x56fd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x570d as WCHAR,
        to: 0x56f4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5712 as WCHAR,
        to: 0x56ed as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5713 as WCHAR,
        to: 0x5706 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5716 as WCHAR,
        to: 0x56fe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5718 as WCHAR,
        to: 0x56e2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5770 as WCHAR,
        to: 0x57a7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x57e1 as WCHAR,
        to: 0x57ad as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x57f7 as WCHAR,
        to: 0x6267 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5805 as WCHAR,
        to: 0x575a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x580a as WCHAR,
        to: 0x57a9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x581d as WCHAR,
        to: 0x57da as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x582f as WCHAR,
        to: 0x5c27 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5831 as WCHAR,
        to: 0x62a5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5834 as WCHAR,
        to: 0x573a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x583f as WCHAR,
        to: 0x78b1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x584a as WCHAR,
        to: 0x5757 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x584b as WCHAR,
        to: 0x8314 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x584f as WCHAR,
        to: 0x57b2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5852 as WCHAR,
        to: 0x57d8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5857 as WCHAR,
        to: 0x6d82 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5862 as WCHAR,
        to: 0x575e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5864 as WCHAR,
        to: 0x57d9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5875 as WCHAR,
        to: 0x5c18 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5879 as WCHAR,
        to: 0x5811 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x588a as WCHAR,
        to: 0x57ab as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5891 as WCHAR,
        to: 0x5892 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x589c as WCHAR,
        to: 0x5760 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x58ae as WCHAR,
        to: 0x5815 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x58b3 as WCHAR,
        to: 0x575f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x58bb as WCHAR,
        to: 0x5899 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x58be as WCHAR,
        to: 0x57a6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x58c7 as WCHAR,
        to: 0x575b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x58ce as WCHAR,
        to: 0x57d9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x58d3 as WCHAR,
        to: 0x538b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x58d8 as WCHAR,
        to: 0x5792 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x58d9 as WCHAR,
        to: 0x5739 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x58da as WCHAR,
        to: 0x5786 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x58de as WCHAR,
        to: 0x574f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x58df as WCHAR,
        to: 0x5784 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x58e2 as WCHAR,
        to: 0x575c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x58e9 as WCHAR,
        to: 0x575d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x58ef as WCHAR,
        to: 0x58ee as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x58fa as WCHAR,
        to: 0x58f6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x58fd as WCHAR,
        to: 0x5bff as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5920 as WCHAR,
        to: 0x591f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5922 as WCHAR,
        to: 0x68a6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x593e as WCHAR,
        to: 0x5939 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5950 as WCHAR,
        to: 0x5942 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5967 as WCHAR,
        to: 0x5965 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5969 as WCHAR,
        to: 0x5941 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x596a as WCHAR,
        to: 0x593a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x596e as WCHAR,
        to: 0x594b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x599d as WCHAR,
        to: 0x5986 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x59b3 as WCHAR,
        to: 0x4f60 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x59cd as WCHAR,
        to: 0x59d7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x59e6 as WCHAR,
        to: 0x5978 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x59ea as WCHAR,
        to: 0x4f84 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5a1b as WCHAR,
        to: 0x5a31 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5a41 as WCHAR,
        to: 0x5a04 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5a66 as WCHAR,
        to: 0x5987 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5a6d as WCHAR,
        to: 0x5a05 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5aa7 as WCHAR,
        to: 0x5a32 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5aaf as WCHAR,
        to: 0x59ab as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5abc as WCHAR,
        to: 0x5aaa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5abd as WCHAR,
        to: 0x5988 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5ad7 as WCHAR,
        to: 0x59aa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5af5 as WCHAR,
        to: 0x59a9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5afb as WCHAR,
        to: 0x5a34 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5b08 as WCHAR,
        to: 0x5a06 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5b0b as WCHAR,
        to: 0x5a75 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5b0c as WCHAR,
        to: 0x5a07 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5b19 as WCHAR,
        to: 0x5af1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5b1d as WCHAR,
        to: 0x8885 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5b21 as WCHAR,
        to: 0x5ad2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5b24 as WCHAR,
        to: 0x5b37 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5b2a as WCHAR,
        to: 0x5ad4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5b30 as WCHAR,
        to: 0x5a74 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5b38 as WCHAR,
        to: 0x5a76 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5b43 as WCHAR,
        to: 0x5a18 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5b4c as WCHAR,
        to: 0x5a08 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5b6b as WCHAR,
        to: 0x5b59 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5b78 as WCHAR,
        to: 0x5b66 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5b7f as WCHAR,
        to: 0x5b6a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5bae as WCHAR,
        to: 0x5bab as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5be2 as WCHAR,
        to: 0x5bdd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5be6 as WCHAR,
        to: 0x5b9e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5be7 as WCHAR,
        to: 0x5b81 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5be9 as WCHAR,
        to: 0x5ba1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5beb as WCHAR,
        to: 0x5199 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5bec as WCHAR,
        to: 0x5bbd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5bf5 as WCHAR,
        to: 0x5ba0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5bf6 as WCHAR,
        to: 0x5b9d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c07 as WCHAR,
        to: 0x5c06 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c08 as WCHAR,
        to: 0x4e13 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c0b as WCHAR,
        to: 0x5bfb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c0d as WCHAR,
        to: 0x5bf9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c0e as WCHAR,
        to: 0x5bfc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c37 as WCHAR,
        to: 0x5c34 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c46 as WCHAR,
        to: 0x5c4a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c5c as WCHAR,
        to: 0x5c49 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c5d as WCHAR,
        to: 0x6249 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c62 as WCHAR,
        to: 0x5c61 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c64 as WCHAR,
        to: 0x5c42 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c68 as WCHAR,
        to: 0x5c66 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5c6c as WCHAR,
        to: 0x5c5e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5ca1 as WCHAR,
        to: 0x5188 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5ca7 as WCHAR,
        to: 0x8fe2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5cf4 as WCHAR,
        to: 0x5c98 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5cf6 as WCHAR,
        to: 0x5c9b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5cfd as WCHAR,
        to: 0x5ce1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5d0d as WCHAR,
        to: 0x5d03 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5d17 as WCHAR,
        to: 0x5c97 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5d19 as WCHAR,
        to: 0x4ed1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5d20 as WCHAR,
        to: 0x5cbd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5d22 as WCHAR,
        to: 0x5ce5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5d33 as WCHAR,
        to: 0x5d5b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5d50 as WCHAR,
        to: 0x5c9a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5d52 as WCHAR,
        to: 0x5ca9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5d81 as WCHAR,
        to: 0x5d5d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5d84 as WCHAR,
        to: 0x5d2d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5d87 as WCHAR,
        to: 0x5c96 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5d97 as WCHAR,
        to: 0x5d02 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5da0 as WCHAR,
        to: 0x5ce4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5da7 as WCHAR,
        to: 0x5cc4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5db8 as WCHAR,
        to: 0x5d58 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5dba as WCHAR,
        to: 0x5cad as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5dbc as WCHAR,
        to: 0x5c7f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5dbd as WCHAR,
        to: 0x5cb3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5dcb as WCHAR,
        to: 0x5cbf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5dd1 as WCHAR,
        to: 0x6f13 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5dd2 as WCHAR,
        to: 0x5ce6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5dd4 as WCHAR,
        to: 0x5dc5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5dd6 as WCHAR,
        to: 0x5ca9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5df0 as WCHAR,
        to: 0x5def as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5df9 as WCHAR,
        to: 0x537a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e25 as WCHAR,
        to: 0x5e05 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e2b as WCHAR,
        to: 0x5e08 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e33 as WCHAR,
        to: 0x5e10 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e36 as WCHAR,
        to: 0x5e26 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e40 as WCHAR,
        to: 0x5e27 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e43 as WCHAR,
        to: 0x5e0f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e57 as WCHAR,
        to: 0x5e3c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e58 as WCHAR,
        to: 0x5e3b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e5f as WCHAR,
        to: 0x5e1c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e63 as WCHAR,
        to: 0x5e01 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e6b as WCHAR,
        to: 0x5e2e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e6c as WCHAR,
        to: 0x5e31 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e79 as WCHAR,
        to: 0x5e72 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e7e as WCHAR,
        to: 0x51e0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5e82 as WCHAR,
        to: 0x4ec4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5eab as WCHAR,
        to: 0x5e93 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5ec1 as WCHAR,
        to: 0x5395 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5ec2 as WCHAR,
        to: 0x53a2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5ec4 as WCHAR,
        to: 0x53a9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5ec8 as WCHAR,
        to: 0x53a6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5eda as WCHAR,
        to: 0x53a8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5edd as WCHAR,
        to: 0x53ae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5edf as WCHAR,
        to: 0x5e99 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5ee0 as WCHAR,
        to: 0x5382 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5ee1 as WCHAR,
        to: 0x5e91 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5ee2 as WCHAR,
        to: 0x5e9f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5ee3 as WCHAR,
        to: 0x5e7f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5ee9 as WCHAR,
        to: 0x5eea as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5eec as WCHAR,
        to: 0x5e90 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5ef1 as WCHAR,
        to: 0x75c8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5ef3 as WCHAR,
        to: 0x5385 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f12 as WCHAR,
        to: 0x5f11 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f14 as WCHAR,
        to: 0x540a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f33 as WCHAR,
        to: 0x5f2a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f35 as WCHAR,
        to: 0x5f20 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f37 as WCHAR,
        to: 0x5f3a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f46 as WCHAR,
        to: 0x522b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f48 as WCHAR,
        to: 0x5f39 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f4a as WCHAR,
        to: 0x5f3a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f4c as WCHAR,
        to: 0x5f25 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f4e as WCHAR,
        to: 0x5f2f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f59 as WCHAR,
        to: 0x6c47 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f65 as WCHAR,
        to: 0x5f66 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f6b as WCHAR,
        to: 0x96d5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f7f as WCHAR,
        to: 0x4f5b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f91 as WCHAR,
        to: 0x5f84 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5f9e as WCHAR,
        to: 0x4ece as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5fa0 as WCHAR,
        to: 0x5f95 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5fa9 as WCHAR,
        to: 0x590d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5fac as WCHAR,
        to: 0x65c1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x5fb9 as WCHAR,
        to: 0x5f7b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6046 as WCHAR,
        to: 0x6052 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6065 as WCHAR,
        to: 0x803b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6085 as WCHAR,
        to: 0x60a6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x60b5 as WCHAR,
        to: 0x6005 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x60b6 as WCHAR,
        to: 0x95f7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x60bd as WCHAR,
        to: 0x51c4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x60e1 as WCHAR,
        to: 0x6076 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x60f1 as WCHAR,
        to: 0x607c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x60f2 as WCHAR,
        to: 0x607d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x60fb as WCHAR,
        to: 0x607b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x611b as WCHAR,
        to: 0x7231 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x611c as WCHAR,
        to: 0x60ec as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6128 as WCHAR,
        to: 0x60ab as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6134 as WCHAR,
        to: 0x6006 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6137 as WCHAR,
        to: 0x607a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x613e as WCHAR,
        to: 0x5ffe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6144 as WCHAR,
        to: 0x6817 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6147 as WCHAR,
        to: 0x6bb7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x614b as WCHAR,
        to: 0x6001 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x614d as WCHAR,
        to: 0x6120 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6158 as WCHAR,
        to: 0x60e8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x615a as WCHAR,
        to: 0x60ed as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x615f as WCHAR,
        to: 0x6078 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6163 as WCHAR,
        to: 0x60ef as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x616a as WCHAR,
        to: 0x6004 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x616b as WCHAR,
        to: 0x6002 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x616e as WCHAR,
        to: 0x8651 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6173 as WCHAR,
        to: 0x60ad as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6176 as WCHAR,
        to: 0x5e86 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x617c as WCHAR,
        to: 0x621a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6182 as WCHAR,
        to: 0x5fe7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x618a as WCHAR,
        to: 0x60eb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6190 as WCHAR,
        to: 0x601c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6191 as WCHAR,
        to: 0x51ed as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6192 as WCHAR,
        to: 0x6126 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x619a as WCHAR,
        to: 0x60ee as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x61a4 as WCHAR,
        to: 0x6124 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x61ab as WCHAR,
        to: 0x60af as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x61ae as WCHAR,
        to: 0x6003 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x61b2 as WCHAR,
        to: 0x5baa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x61b6 as WCHAR,
        to: 0x5fc6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x61c7 as WCHAR,
        to: 0x6073 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x61c9 as WCHAR,
        to: 0x5e94 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x61cc as WCHAR,
        to: 0x603f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x61cd as WCHAR,
        to: 0x6aa9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x61de as WCHAR,
        to: 0x8499 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x61df as WCHAR,
        to: 0x603c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x61e3 as WCHAR,
        to: 0x61d1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x61e8 as WCHAR,
        to: 0x6079 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x61f2 as WCHAR,
        to: 0x60e9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x61f6 as WCHAR,
        to: 0x61d2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x61f7 as WCHAR,
        to: 0x6000 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x61f8 as WCHAR,
        to: 0x60ac as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x61fa as WCHAR,
        to: 0x5fcf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x61fc as WCHAR,
        to: 0x60e7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x61fe as WCHAR,
        to: 0x6151 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6200 as WCHAR,
        to: 0x604b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6207 as WCHAR,
        to: 0x6206 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6209 as WCHAR,
        to: 0x94ba as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6214 as WCHAR,
        to: 0x620b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6227 as WCHAR,
        to: 0x6217 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6229 as WCHAR,
        to: 0x622c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6230 as WCHAR,
        to: 0x6218 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6232 as WCHAR,
        to: 0x620f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6236 as WCHAR,
        to: 0x6237 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6250 as WCHAR,
        to: 0x4ec2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6260 as WCHAR,
        to: 0x53c9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6262 as WCHAR,
        to: 0x6266 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x627a as WCHAR,
        to: 0x62b5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6283 as WCHAR,
        to: 0x62da as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6286 as WCHAR,
        to: 0x64e6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x628e as WCHAR,
        to: 0x6b92 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x62b4 as WCHAR,
        to: 0x66f3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x62cb as WCHAR,
        to: 0x629b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6336 as WCHAR,
        to: 0x5c40 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x633e as WCHAR,
        to: 0x631f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x634d as WCHAR,
        to: 0x625e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6368 as WCHAR,
        to: 0x820d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x636b as WCHAR,
        to: 0x626a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6372 as WCHAR,
        to: 0x5377 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6383 as WCHAR,
        to: 0x626b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6384 as WCHAR,
        to: 0x62a1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6399 as WCHAR,
        to: 0x6323 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x639b as WCHAR,
        to: 0x6302 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x63a1 as WCHAR,
        to: 0x91c7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x63c0 as WCHAR,
        to: 0x62e3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x63da as WCHAR,
        to: 0x626c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x63db as WCHAR,
        to: 0x6362 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x63ee as WCHAR,
        to: 0x6325 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x63f9 as WCHAR,
        to: 0x80cc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6406 as WCHAR,
        to: 0x6784 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x640d as WCHAR,
        to: 0x635f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6416 as WCHAR,
        to: 0x6447 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6417 as WCHAR,
        to: 0x6363 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x641f as WCHAR,
        to: 0x64c0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6436 as WCHAR,
        to: 0x62a2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6451 as WCHAR,
        to: 0x63b4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x645c as WCHAR,
        to: 0x63bc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x645f as WCHAR,
        to: 0x6402 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x646f as WCHAR,
        to: 0x631a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6473 as WCHAR,
        to: 0x62a0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6476 as WCHAR,
        to: 0x629f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x647b as WCHAR,
        to: 0x63ba as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6488 as WCHAR,
        to: 0x635e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6490 as WCHAR,
        to: 0x6491 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6493 as WCHAR,
        to: 0x6320 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x649a as WCHAR,
        to: 0x637b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x649f as WCHAR,
        to: 0x6322 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64a2 as WCHAR,
        to: 0x63b8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64a3 as WCHAR,
        to: 0x63b8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64a5 as WCHAR,
        to: 0x62e8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64ab as WCHAR,
        to: 0x629a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64b2 as WCHAR,
        to: 0x6251 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64b3 as WCHAR,
        to: 0x63ff as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64bb as WCHAR,
        to: 0x631e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64be as WCHAR,
        to: 0x631d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64bf as WCHAR,
        to: 0x6361 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64c1 as WCHAR,
        to: 0x62e5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64c4 as WCHAR,
        to: 0x63b3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64c7 as WCHAR,
        to: 0x62e9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64ca as WCHAR,
        to: 0x51fb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64cb as WCHAR,
        to: 0x6321 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64d4 as WCHAR,
        to: 0x62c5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64da as WCHAR,
        to: 0x636e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64e0 as WCHAR,
        to: 0x6324 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64e3 as WCHAR,
        to: 0x6363 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64ec as WCHAR,
        to: 0x62df as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64ef as WCHAR,
        to: 0x6448 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64f0 as WCHAR,
        to: 0x62e7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64f1 as WCHAR,
        to: 0x6401 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64f2 as WCHAR,
        to: 0x63b7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64f4 as WCHAR,
        to: 0x6269 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64f7 as WCHAR,
        to: 0x64b7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64fa as WCHAR,
        to: 0x6446 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64fb as WCHAR,
        to: 0x64de as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64fc as WCHAR,
        to: 0x64b8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x64fe as WCHAR,
        to: 0x6270 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6504 as WCHAR,
        to: 0x6445 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6506 as WCHAR,
        to: 0x64b5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x650f as WCHAR,
        to: 0x62e2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6514 as WCHAR,
        to: 0x62e6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6516 as WCHAR,
        to: 0x6484 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6519 as WCHAR,
        to: 0x6400 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x651b as WCHAR,
        to: 0x64ba as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x651c as WCHAR,
        to: 0x643a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x651d as WCHAR,
        to: 0x6444 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6522 as WCHAR,
        to: 0x6512 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6523 as WCHAR,
        to: 0x631b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6524 as WCHAR,
        to: 0x644a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x652a as WCHAR,
        to: 0x6405 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x652c as WCHAR,
        to: 0x63fd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6537 as WCHAR,
        to: 0x8003 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6557 as WCHAR,
        to: 0x8d25 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6558 as WCHAR,
        to: 0x53d9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6575 as WCHAR,
        to: 0x654c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6578 as WCHAR,
        to: 0x6570 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6582 as WCHAR,
        to: 0x655b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6583 as WCHAR,
        to: 0x6bd9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6595 as WCHAR,
        to: 0x6593 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x65ac as WCHAR,
        to: 0x65a9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x65b7 as WCHAR,
        to: 0x65ad as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x65c2 as WCHAR,
        to: 0x65d7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6642 as WCHAR,
        to: 0x65f6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6649 as WCHAR,
        to: 0x664b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x665d as WCHAR,
        to: 0x663c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x665e as WCHAR,
        to: 0x66e6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6688 as WCHAR,
        to: 0x6655 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6689 as WCHAR,
        to: 0x6656 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6698 as WCHAR,
        to: 0x9633 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x66a2 as WCHAR,
        to: 0x7545 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x66ab as WCHAR,
        to: 0x6682 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x66b1 as WCHAR,
        to: 0x6635 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x66b8 as WCHAR,
        to: 0x4e86 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x66c4 as WCHAR,
        to: 0x6654 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x66c6 as WCHAR,
        to: 0x5386 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x66c7 as WCHAR,
        to: 0x6619 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x66c9 as WCHAR,
        to: 0x6653 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x66d6 as WCHAR,
        to: 0x66a7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x66e0 as WCHAR,
        to: 0x65f7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x66ec as WCHAR,
        to: 0x6652 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x66f8 as WCHAR,
        to: 0x4e66 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6703 as WCHAR,
        to: 0x4f1a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6727 as WCHAR,
        to: 0x80e7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x672e as WCHAR,
        to: 0x672f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6747 as WCHAR,
        to: 0x572c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6771 as WCHAR,
        to: 0x4e1c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6799 as WCHAR,
        to: 0x6800 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x67b4 as WCHAR,
        to: 0x62d0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x67f5 as WCHAR,
        to: 0x6805 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x687f as WCHAR,
        to: 0x6746 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6894 as WCHAR,
        to: 0x6800 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x689d as WCHAR,
        to: 0x6761 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x689f as WCHAR,
        to: 0x67ad as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x68c4 as WCHAR,
        to: 0x5f03 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x68d6 as WCHAR,
        to: 0x67a8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x68d7 as WCHAR,
        to: 0x67a3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x68df as WCHAR,
        to: 0x680b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x68e7 as WCHAR,
        to: 0x6808 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x68f2 as WCHAR,
        to: 0x6816 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x690f as WCHAR,
        to: 0x6860 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x694a as WCHAR,
        to: 0x6768 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6953 as WCHAR,
        to: 0x67ab as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6968 as WCHAR,
        to: 0x6862 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x696d as WCHAR,
        to: 0x4e1a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6975 as WCHAR,
        to: 0x6781 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x69a6 as WCHAR,
        to: 0x5e72 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x69aa as WCHAR,
        to: 0x6769 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x69ae as WCHAR,
        to: 0x8363 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x69bf as WCHAR,
        to: 0x6864 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x69c3 as WCHAR,
        to: 0x76d8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x69cb as WCHAR,
        to: 0x6784 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x69cd as WCHAR,
        to: 0x67aa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x69d3 as WCHAR,
        to: 0x6760 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x69e7 as WCHAR,
        to: 0x6920 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x69e8 as WCHAR,
        to: 0x6901 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x69f3 as WCHAR,
        to: 0x6868 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a01 as WCHAR,
        to: 0x6869 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a02 as WCHAR,
        to: 0x4e50 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a05 as WCHAR,
        to: 0x679e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a11 as WCHAR,
        to: 0x6881 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a13 as WCHAR,
        to: 0x697c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a19 as WCHAR,
        to: 0x6807 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a1e as WCHAR,
        to: 0x67a2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a23 as WCHAR,
        to: 0x6837 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a38 as WCHAR,
        to: 0x6734 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a39 as WCHAR,
        to: 0x6811 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a3a as WCHAR,
        to: 0x6866 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a48 as WCHAR,
        to: 0x6861 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a4b as WCHAR,
        to: 0x6865 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a5f as WCHAR,
        to: 0x673a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a62 as WCHAR,
        to: 0x692d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a6b as WCHAR,
        to: 0x6a2a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a81 as WCHAR,
        to: 0x6aa9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a89 as WCHAR,
        to: 0x67fd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a94 as WCHAR,
        to: 0x6863 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6a9c as WCHAR,
        to: 0x6867 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6aa2 as WCHAR,
        to: 0x68c0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6aa3 as WCHAR,
        to: 0x6a2f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6aaf as WCHAR,
        to: 0x53f0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ab3 as WCHAR,
        to: 0x69df as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ab8 as WCHAR,
        to: 0x67e0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6abb as WCHAR,
        to: 0x69db as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ac3 as WCHAR,
        to: 0x67dc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ad3 as WCHAR,
        to: 0x6a79 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ada as WCHAR,
        to: 0x6988 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6adb as WCHAR,
        to: 0x6809 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6add as WCHAR,
        to: 0x691f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ade as WCHAR,
        to: 0x6a7c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6adf as WCHAR,
        to: 0x680e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ae5 as WCHAR,
        to: 0x6a71 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ae7 as WCHAR,
        to: 0x69e0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ae8 as WCHAR,
        to: 0x680c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6aea as WCHAR,
        to: 0x67a5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6aec as WCHAR,
        to: 0x6987 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6af3 as WCHAR,
        to: 0x680a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6af8 as WCHAR,
        to: 0x6989 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6afa as WCHAR,
        to: 0x68c2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6afb as WCHAR,
        to: 0x6a31 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b04 as WCHAR,
        to: 0x680f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b0a as WCHAR,
        to: 0x6743 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b0f as WCHAR,
        to: 0x6924 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b12 as WCHAR,
        to: 0x683e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b16 as WCHAR,
        to: 0x6984 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b1e as WCHAR,
        to: 0x68c2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b3d as WCHAR,
        to: 0x94a6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b4e as WCHAR,
        to: 0x53f9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b50 as WCHAR,
        to: 0x6b27 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b5f as WCHAR,
        to: 0x6b24 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b61 as WCHAR,
        to: 0x6b22 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b72 as WCHAR,
        to: 0x5c81 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b77 as WCHAR,
        to: 0x5386 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b78 as WCHAR,
        to: 0x5f52 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b7f as WCHAR,
        to: 0x6b81 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b98 as WCHAR,
        to: 0x6b8b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6b9e as WCHAR,
        to: 0x6b92 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ba4 as WCHAR,
        to: 0x6b87 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6bab as WCHAR,
        to: 0x6b9a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6bae as WCHAR,
        to: 0x6b93 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6baf as WCHAR,
        to: 0x6ba1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6bb2 as WCHAR,
        to: 0x6b7c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6bba as WCHAR,
        to: 0x6740 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6bbc as WCHAR,
        to: 0x58f3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6bc0 as WCHAR,
        to: 0x6bc1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6bc6 as WCHAR,
        to: 0x6bb4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6bcc as WCHAR,
        to: 0x6bcb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6bec as WCHAR,
        to: 0x7403 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6bff as WCHAR,
        to: 0x6bf5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6c08 as WCHAR,
        to: 0x6be1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6c0c as WCHAR,
        to: 0x6c07 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6c23 as WCHAR,
        to: 0x6c14 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6c2b as WCHAR,
        to: 0x6c22 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6c2c as WCHAR,
        to: 0x6c29 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6c33 as WCHAR,
        to: 0x6c32 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6c3e as WCHAR,
        to: 0x6cdb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6c4d as WCHAR,
        to: 0x4e38 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6c4e as WCHAR,
        to: 0x6cdb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6c59 as WCHAR,
        to: 0x6c61 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6c7a as WCHAR,
        to: 0x51b3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6c92 as WCHAR,
        to: 0x6ca1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6c96 as WCHAR,
        to: 0x51b2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6cc1 as WCHAR,
        to: 0x51b5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6d29 as WCHAR,
        to: 0x6cc4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6d36 as WCHAR,
        to: 0x6c79 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6d6c as WCHAR,
        to: 0x91cc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6d79 as WCHAR,
        to: 0x6d43 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6d87 as WCHAR,
        to: 0x6cfe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6dbc as WCHAR,
        to: 0x51c9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6dd2 as WCHAR,
        to: 0x51c4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6dda as WCHAR,
        to: 0x6cea as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6de5 as WCHAR,
        to: 0x6e0c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6de8 as WCHAR,
        to: 0x51c0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6dea as WCHAR,
        to: 0x6ca6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6df5 as WCHAR,
        to: 0x6e0a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6df6 as WCHAR,
        to: 0x6d9e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6dfa as WCHAR,
        to: 0x6d45 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e19 as WCHAR,
        to: 0x6da3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e1b as WCHAR,
        to: 0x51cf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e26 as WCHAR,
        to: 0x6da1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e2c as WCHAR,
        to: 0x6d4b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e3e as WCHAR,
        to: 0x6d51 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e4a as WCHAR,
        to: 0x51d1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e5e as WCHAR,
        to: 0x6d48 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e67 as WCHAR,
        to: 0x6d8c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e6f as WCHAR,
        to: 0x6c64 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e88 as WCHAR,
        to: 0x6ca9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e96 as WCHAR,
        to: 0x51c6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6e9d as WCHAR,
        to: 0x6c9f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6eab as WCHAR,
        to: 0x6e29 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ebc as WCHAR,
        to: 0x6e7f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ec4 as WCHAR,
        to: 0x6ca7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ec5 as WCHAR,
        to: 0x706d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ecc as WCHAR,
        to: 0x6da4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ece as WCHAR,
        to: 0x8365 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6eec as WCHAR,
        to: 0x6caa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6eef as WCHAR,
        to: 0x6ede as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ef2 as WCHAR,
        to: 0x6e17 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ef7 as WCHAR,
        to: 0x5364 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ef8 as WCHAR,
        to: 0x6d52 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6efe as WCHAR,
        to: 0x6eda as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6eff as WCHAR,
        to: 0x6ee1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f01 as WCHAR,
        to: 0x6e14 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f1a as WCHAR,
        to: 0x6ca4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f22 as WCHAR,
        to: 0x6c49 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f23 as WCHAR,
        to: 0x6d9f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f2c as WCHAR,
        to: 0x6e0d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f32 as WCHAR,
        to: 0x6da8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f38 as WCHAR,
        to: 0x6e10 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f3f as WCHAR,
        to: 0x6d46 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f41 as WCHAR,
        to: 0x988d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f51 as WCHAR,
        to: 0x6cfc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f54 as WCHAR,
        to: 0x6d01 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f5b as WCHAR,
        to: 0x6f5c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f5f as WCHAR,
        to: 0x6cfb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f64 as WCHAR,
        to: 0x6da6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f6f as WCHAR,
        to: 0x6d54 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f70 as WCHAR,
        to: 0x6e83 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f77 as WCHAR,
        to: 0x6ed7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f7f as WCHAR,
        to: 0x6da0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f80 as WCHAR,
        to: 0x6da9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f86 as WCHAR,
        to: 0x6d47 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f87 as WCHAR,
        to: 0x6d9d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6f97 as WCHAR,
        to: 0x6da7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6fa0 as WCHAR,
        to: 0x6e11 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6fa4 as WCHAR,
        to: 0x6cfd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6fa9 as WCHAR,
        to: 0x6cf6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6fae as WCHAR,
        to: 0x6d4d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6fb1 as WCHAR,
        to: 0x6dc0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6fc1 as WCHAR,
        to: 0x6d4a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6fc3 as WCHAR,
        to: 0x6d53 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6fd5 as WCHAR,
        to: 0x6e7f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6fd8 as WCHAR,
        to: 0x6cde as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6fdb as WCHAR,
        to: 0x8499 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6fdf as WCHAR,
        to: 0x6d4e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6fe4 as WCHAR,
        to: 0x6d9b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6feb as WCHAR,
        to: 0x6ee5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ff0 as WCHAR,
        to: 0x6f4d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ff1 as WCHAR,
        to: 0x6ee8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ffa as WCHAR,
        to: 0x6e85 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ffc as WCHAR,
        to: 0x6cfa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x6ffe as WCHAR,
        to: 0x6ee4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7005 as WCHAR,
        to: 0x6ee2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7006 as WCHAR,
        to: 0x6e0e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7009 as WCHAR,
        to: 0x6cfb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x700b as WCHAR,
        to: 0x6e16 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x700f as WCHAR,
        to: 0x6d4f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7015 as WCHAR,
        to: 0x6fd2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7018 as WCHAR,
        to: 0x6cf8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x701d as WCHAR,
        to: 0x6ca5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x701f as WCHAR,
        to: 0x6f47 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7020 as WCHAR,
        to: 0x6f46 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7027 as WCHAR,
        to: 0x6cf7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7028 as WCHAR,
        to: 0x6fd1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7030 as WCHAR,
        to: 0x5f25 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7032 as WCHAR,
        to: 0x6f4b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x703e as WCHAR,
        to: 0x6f9c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7043 as WCHAR,
        to: 0x6ca3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7044 as WCHAR,
        to: 0x6ee0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7051 as WCHAR,
        to: 0x6d12 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7055 as WCHAR,
        to: 0x6f13 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7058 as WCHAR,
        to: 0x6ee9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x705d as WCHAR,
        to: 0x704f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7063 as WCHAR,
        to: 0x6e7e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7064 as WCHAR,
        to: 0x6ee6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x707d as WCHAR,
        to: 0x707e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x70ba as WCHAR,
        to: 0x4e3a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x70cf as WCHAR,
        to: 0x4e4c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x70f4 as WCHAR,
        to: 0x70c3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7121 as WCHAR,
        to: 0x65e0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7149 as WCHAR,
        to: 0x70bc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7152 as WCHAR,
        to: 0x709c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7159 as WCHAR,
        to: 0x70df as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7162 as WCHAR,
        to: 0x8315 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7165 as WCHAR,
        to: 0x7115 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7169 as WCHAR,
        to: 0x70e6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x716c as WCHAR,
        to: 0x7080 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7192 as WCHAR,
        to: 0x8367 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7197 as WCHAR,
        to: 0x709d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x71b1 as WCHAR,
        to: 0x70ed as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x71be as WCHAR,
        to: 0x70bd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x71c1 as WCHAR,
        to: 0x70e8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x71c4 as WCHAR,
        to: 0x7130 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x71c8 as WCHAR,
        to: 0x706f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x71c9 as WCHAR,
        to: 0x7096 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x71d0 as WCHAR,
        to: 0x78f7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x71d2 as WCHAR,
        to: 0x70e7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x71d9 as WCHAR,
        to: 0x70eb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x71dc as WCHAR,
        to: 0x7116 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x71df as WCHAR,
        to: 0x8425 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x71e6 as WCHAR,
        to: 0x707f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x71ec as WCHAR,
        to: 0x6bc1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x71ed as WCHAR,
        to: 0x70db as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x71f4 as WCHAR,
        to: 0x70e9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x71fc as WCHAR,
        to: 0x70ec as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x71fe as WCHAR,
        to: 0x7118 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x720d as WCHAR,
        to: 0x70c1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7210 as WCHAR,
        to: 0x7089 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x721b as WCHAR,
        to: 0x70c2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x722d as WCHAR,
        to: 0x4e89 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x723a as WCHAR,
        to: 0x7237 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x723e as WCHAR,
        to: 0x5c14 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7246 as WCHAR,
        to: 0x5899 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7258 as WCHAR,
        to: 0x724d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x726a as WCHAR,
        to: 0x762a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7274 as WCHAR,
        to: 0x62b5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x727d as WCHAR,
        to: 0x7275 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7296 as WCHAR,
        to: 0x8366 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x729b as WCHAR,
        to: 0x7281 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x72a2 as WCHAR,
        to: 0x728a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x72a7 as WCHAR,
        to: 0x727a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x72c0 as WCHAR,
        to: 0x72b6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x72f9 as WCHAR,
        to: 0x72ed as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x72fd as WCHAR,
        to: 0x72c8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7319 as WCHAR,
        to: 0x72f0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7336 as WCHAR,
        to: 0x72b9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x733b as WCHAR,
        to: 0x72f2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7344 as WCHAR,
        to: 0x72f1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7345 as WCHAR,
        to: 0x72ee as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x734e as WCHAR,
        to: 0x5956 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7368 as WCHAR,
        to: 0x72ec as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x736a as WCHAR,
        to: 0x72ef as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x736b as WCHAR,
        to: 0x7303 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x736e as WCHAR,
        to: 0x7315 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7370 as WCHAR,
        to: 0x72de as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7372 as WCHAR,
        to: 0x83b7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7375 as WCHAR,
        to: 0x730e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7377 as WCHAR,
        to: 0x72b7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7378 as WCHAR,
        to: 0x517d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x737a as WCHAR,
        to: 0x736d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x737b as WCHAR,
        to: 0x732e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x737c as WCHAR,
        to: 0x7315 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7380 as WCHAR,
        to: 0x7321 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7386 as WCHAR,
        to: 0x5179 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x73a8 as WCHAR,
        to: 0x73cf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x73ee as WCHAR,
        to: 0x4f69 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x73fe as WCHAR,
        to: 0x73b0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x743a as WCHAR,
        to: 0x73d0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x743f as WCHAR,
        to: 0x73f2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x744b as WCHAR,
        to: 0x73ae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7463 as WCHAR,
        to: 0x7410 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7464 as WCHAR,
        to: 0x7476 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7469 as WCHAR,
        to: 0x83b9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x746a as WCHAR,
        to: 0x739b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x746f as WCHAR,
        to: 0x7405 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7489 as WCHAR,
        to: 0x740f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x74a3 as WCHAR,
        to: 0x7391 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x74a6 as WCHAR,
        to: 0x7477 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x74b0 as WCHAR,
        to: 0x73af as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x74bd as WCHAR,
        to: 0x73ba as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x74ca as WCHAR,
        to: 0x743c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x74cf as WCHAR,
        to: 0x73d1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x74d4 as WCHAR,
        to: 0x748e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x74d6 as WCHAR,
        to: 0x9576 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x74da as WCHAR,
        to: 0x74d2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x750c as WCHAR,
        to: 0x74ef as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7515 as WCHAR,
        to: 0x74ee as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7522 as WCHAR,
        to: 0x4ea7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7526 as WCHAR,
        to: 0x82cf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x755d as WCHAR,
        to: 0x4ea9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7562 as WCHAR,
        to: 0x6bd5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x756b as WCHAR,
        to: 0x753b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x756c as WCHAR,
        to: 0x7572 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7570 as WCHAR,
        to: 0x5f02 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7576 as WCHAR,
        to: 0x5f53 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7587 as WCHAR,
        to: 0x7574 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x758a as WCHAR,
        to: 0x53e0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x75bf as WCHAR,
        to: 0x75f1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x75d9 as WCHAR,
        to: 0x75c9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x75e0 as WCHAR,
        to: 0x9178 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x75f2 as WCHAR,
        to: 0x9ebb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x75f3 as WCHAR,
        to: 0x9ebb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x75fa as WCHAR,
        to: 0x75f9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x760b as WCHAR,
        to: 0x75af as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x760d as WCHAR,
        to: 0x75a1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7613 as WCHAR,
        to: 0x75ea as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x761e as WCHAR,
        to: 0x7617 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7621 as WCHAR,
        to: 0x75ae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7627 as WCHAR,
        to: 0x759f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x763a as WCHAR,
        to: 0x7618 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7642 as WCHAR,
        to: 0x7597 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7646 as WCHAR,
        to: 0x75e8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7647 as WCHAR,
        to: 0x75eb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7649 as WCHAR,
        to: 0x7605 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7658 as WCHAR,
        to: 0x75a0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x765f as WCHAR,
        to: 0x762a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7661 as WCHAR,
        to: 0x75f4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7662 as WCHAR,
        to: 0x75d2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7664 as WCHAR,
        to: 0x7596 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7665 as WCHAR,
        to: 0x75c7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7669 as WCHAR,
        to: 0x765e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x766c as WCHAR,
        to: 0x7663 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x766d as WCHAR,
        to: 0x763f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x766e as WCHAR,
        to: 0x763e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7670 as WCHAR,
        to: 0x75c8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7671 as WCHAR,
        to: 0x762b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7672 as WCHAR,
        to: 0x766b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x767c as WCHAR,
        to: 0x53d1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x769a as WCHAR,
        to: 0x7691 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x76b0 as WCHAR,
        to: 0x75b1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x76b8 as WCHAR,
        to: 0x76b2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x76ba as WCHAR,
        to: 0x76b1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x76bb as WCHAR,
        to: 0x96b3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x76dc as WCHAR,
        to: 0x76d7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x76de as WCHAR,
        to: 0x76cf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x76e1 as WCHAR,
        to: 0x5c3d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x76e3 as WCHAR,
        to: 0x76d1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x76e4 as WCHAR,
        to: 0x76d8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x76e7 as WCHAR,
        to: 0x5362 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x76ea as WCHAR,
        to: 0x8361 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7725 as WCHAR,
        to: 0x7726 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x773e as WCHAR,
        to: 0x4f17 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x774f as WCHAR,
        to: 0x56f0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x775c as WCHAR,
        to: 0x7741 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x775e as WCHAR,
        to: 0x7750 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x776a as WCHAR,
        to: 0x777e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7787 as WCHAR,
        to: 0x772f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x779e as WCHAR,
        to: 0x7792 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x77ad as WCHAR,
        to: 0x4e86 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x77bc as WCHAR,
        to: 0x7751 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x77c7 as WCHAR,
        to: 0x8499 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x77d3 as WCHAR,
        to: 0x80e7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x77da as WCHAR,
        to: 0x77a9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x77ef as WCHAR,
        to: 0x77eb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7832 as WCHAR,
        to: 0x70ae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7843 as WCHAR,
        to: 0x6731 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7864 as WCHAR,
        to: 0x7856 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7868 as WCHAR,
        to: 0x7817 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x786f as WCHAR,
        to: 0x781a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x78a9 as WCHAR,
        to: 0x7855 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x78ad as WCHAR,
        to: 0x7800 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x78ba as WCHAR,
        to: 0x786e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x78bc as WCHAR,
        to: 0x7801 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x78da as WCHAR,
        to: 0x7816 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x78e3 as WCHAR,
        to: 0x789c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x78e7 as WCHAR,
        to: 0x789b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x78ef as WCHAR,
        to: 0x77f6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x78fd as WCHAR,
        to: 0x7857 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x790e as WCHAR,
        to: 0x7840 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7919 as WCHAR,
        to: 0x788d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7926 as WCHAR,
        to: 0x77ff as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x792a as WCHAR,
        to: 0x783a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x792b as WCHAR,
        to: 0x783e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x792c as WCHAR,
        to: 0x77fe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7931 as WCHAR,
        to: 0x783b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7945 as WCHAR,
        to: 0x7946 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7947 as WCHAR,
        to: 0x53ea as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7950 as WCHAR,
        to: 0x4f51 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7955 as WCHAR,
        to: 0x79d8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x797f as WCHAR,
        to: 0x7984 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x798d as WCHAR,
        to: 0x7978 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x798e as WCHAR,
        to: 0x796f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x79a6 as WCHAR,
        to: 0x5fa1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x79aa as WCHAR,
        to: 0x7985 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x79ae as WCHAR,
        to: 0x793c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x79b0 as WCHAR,
        to: 0x7962 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x79b1 as WCHAR,
        to: 0x7977 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x79bf as WCHAR,
        to: 0x79c3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x79c8 as WCHAR,
        to: 0x7c7c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7a05 as WCHAR,
        to: 0x7a0e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7a08 as WCHAR,
        to: 0x79c6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7a1c as WCHAR,
        to: 0x68f1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7a1f as WCHAR,
        to: 0x7980 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7a2e as WCHAR,
        to: 0x79cd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7a31 as WCHAR,
        to: 0x79f0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7a40 as WCHAR,
        to: 0x8c37 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7a4c as WCHAR,
        to: 0x7a23 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7a4d as WCHAR,
        to: 0x79ef as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7a4e as WCHAR,
        to: 0x9896 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7a61 as WCHAR,
        to: 0x7a51 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7a62 as WCHAR,
        to: 0x79fd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7a69 as WCHAR,
        to: 0x7a33 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7a6b as WCHAR,
        to: 0x83b7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7aa9 as WCHAR,
        to: 0x7a9d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7aaa as WCHAR,
        to: 0x6d3c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7aae as WCHAR,
        to: 0x7a77 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7aaf as WCHAR,
        to: 0x7a91 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ab6 as WCHAR,
        to: 0x7aad as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7aba as WCHAR,
        to: 0x7aa5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ac4 as WCHAR,
        to: 0x7a9c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ac5 as WCHAR,
        to: 0x7a8d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ac7 as WCHAR,
        to: 0x7aa6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7aca as WCHAR,
        to: 0x7a83 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7af6 as WCHAR,
        to: 0x7ade as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7b3b as WCHAR,
        to: 0x7b47 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7b46 as WCHAR,
        to: 0x7b14 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7b4d as WCHAR,
        to: 0x7b0b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7b67 as WCHAR,
        to: 0x7b15 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7b84 as WCHAR,
        to: 0x7b85 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7b87 as WCHAR,
        to: 0x4e2a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7b8b as WCHAR,
        to: 0x7b3a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7b8f as WCHAR,
        to: 0x7b5d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7bc0 as WCHAR,
        to: 0x8282 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7bc4 as WCHAR,
        to: 0x8303 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7bc9 as WCHAR,
        to: 0x7b51 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7bcb as WCHAR,
        to: 0x7ba7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7be0 as WCHAR,
        to: 0x7b71 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7be4 as WCHAR,
        to: 0x7b03 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7be9 as WCHAR,
        to: 0x7b5b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7bf3 as WCHAR,
        to: 0x7b5a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c00 as WCHAR,
        to: 0x7ba6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c0d as WCHAR,
        to: 0x7bd3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c11 as WCHAR,
        to: 0x84d1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c1e as WCHAR,
        to: 0x7baa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c21 as WCHAR,
        to: 0x7b80 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c23 as WCHAR,
        to: 0x7bd1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c2b as WCHAR,
        to: 0x7bab as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c37 as WCHAR,
        to: 0x6a90 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c3d as WCHAR,
        to: 0x7b7e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c3e as WCHAR,
        to: 0x5e18 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c43 as WCHAR,
        to: 0x7bee as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c4c as WCHAR,
        to: 0x7b79 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c50 as WCHAR,
        to: 0x85e4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c5c as WCHAR,
        to: 0x7ba8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c5f as WCHAR,
        to: 0x7c41 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c60 as WCHAR,
        to: 0x7b3c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c64 as WCHAR,
        to: 0x7b7e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c65 as WCHAR,
        to: 0x9fa0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c69 as WCHAR,
        to: 0x7b3e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c6a as WCHAR,
        to: 0x7c16 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c6c as WCHAR,
        to: 0x7bf1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c6e as WCHAR,
        to: 0x7ba9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7c72 as WCHAR,
        to: 0x5401 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7cb5 as WCHAR,
        to: 0x7ca4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7cdd as WCHAR,
        to: 0x7cc1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7cde as WCHAR,
        to: 0x7caa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ce2 as WCHAR,
        to: 0x998d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ce7 as WCHAR,
        to: 0x7cae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7cf0 as WCHAR,
        to: 0x56e2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7cf2 as WCHAR,
        to: 0x7c9d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7cf4 as WCHAR,
        to: 0x7c74 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7cf6 as WCHAR,
        to: 0x7c9c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7cfe as WCHAR,
        to: 0x7ea0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d00 as WCHAR,
        to: 0x7eaa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d02 as WCHAR,
        to: 0x7ea3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d04 as WCHAR,
        to: 0x7ea6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d05 as WCHAR,
        to: 0x7ea2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d06 as WCHAR,
        to: 0x7ea1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d07 as WCHAR,
        to: 0x7ea5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d08 as WCHAR,
        to: 0x7ea8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d09 as WCHAR,
        to: 0x7eab as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d0b as WCHAR,
        to: 0x7eb9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d0d as WCHAR,
        to: 0x7eb3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d10 as WCHAR,
        to: 0x7ebd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d13 as WCHAR,
        to: 0x7ebe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d14 as WCHAR,
        to: 0x7eaf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d15 as WCHAR,
        to: 0x7eb0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d17 as WCHAR,
        to: 0x7eb1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d19 as WCHAR,
        to: 0x7eb8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d1a as WCHAR,
        to: 0x7ea7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d1b as WCHAR,
        to: 0x7eb7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d1c as WCHAR,
        to: 0x7ead as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d21 as WCHAR,
        to: 0x7eba as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d30 as WCHAR,
        to: 0x7ec6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d31 as WCHAR,
        to: 0x7ec2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d32 as WCHAR,
        to: 0x7ec1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d33 as WCHAR,
        to: 0x7ec5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d39 as WCHAR,
        to: 0x7ecd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d3a as WCHAR,
        to: 0x7ec0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d3c as WCHAR,
        to: 0x7ecb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d3f as WCHAR,
        to: 0x7ed0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d40 as WCHAR,
        to: 0x7ecc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d42 as WCHAR,
        to: 0x7ec8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d43 as WCHAR,
        to: 0x5f26 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d44 as WCHAR,
        to: 0x7ec4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d46 as WCHAR,
        to: 0x7eca as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d4e as WCHAR,
        to: 0x7ed7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d50 as WCHAR,
        to: 0x7ed3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d55 as WCHAR,
        to: 0x7edd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d5b as WCHAR,
        to: 0x7ee6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d5e as WCHAR,
        to: 0x7ede as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d61 as WCHAR,
        to: 0x7edc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d62 as WCHAR,
        to: 0x7eda as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d66 as WCHAR,
        to: 0x7ed9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d68 as WCHAR,
        to: 0x7ed2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d71 as WCHAR,
        to: 0x7edf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d72 as WCHAR,
        to: 0x4e1d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d73 as WCHAR,
        to: 0x7ee6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d79 as WCHAR,
        to: 0x7ee2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d81 as WCHAR,
        to: 0x7ed1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d83 as WCHAR,
        to: 0x7ee1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d86 as WCHAR,
        to: 0x7ee0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d88 as WCHAR,
        to: 0x7ee8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d8f as WCHAR,
        to: 0x7ee5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d93 as WCHAR,
        to: 0x7ecf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d9c as WCHAR,
        to: 0x7efc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7d9e as WCHAR,
        to: 0x7f0d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7da0 as WCHAR,
        to: 0x7eff as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7da2 as WCHAR,
        to: 0x7ef8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7da3 as WCHAR,
        to: 0x7efb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7dac as WCHAR,
        to: 0x7ef6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7dad as WCHAR,
        to: 0x7ef4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7db0 as WCHAR,
        to: 0x7efe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7db1 as WCHAR,
        to: 0x7eb2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7db2 as WCHAR,
        to: 0x7f51 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7db4 as WCHAR,
        to: 0x7f00 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7db5 as WCHAR,
        to: 0x5f69 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7db8 as WCHAR,
        to: 0x7eb6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7db9 as WCHAR,
        to: 0x7efa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7dba as WCHAR,
        to: 0x7eee as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7dbb as WCHAR,
        to: 0x7efd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7dbd as WCHAR,
        to: 0x7ef0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7dbe as WCHAR,
        to: 0x7eeb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7dbf as WCHAR,
        to: 0x7ef5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7dc4 as WCHAR,
        to: 0x7ef2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7dc7 as WCHAR,
        to: 0x7f01 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7dca as WCHAR,
        to: 0x7d27 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7dcb as WCHAR,
        to: 0x7eef as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7dd2 as WCHAR,
        to: 0x7eea as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7dd7 as WCHAR,
        to: 0x7f03 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7dd8 as WCHAR,
        to: 0x7f04 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7dd9 as WCHAR,
        to: 0x7f02 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7dda as WCHAR,
        to: 0x7ebf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ddd as WCHAR,
        to: 0x7f09 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7dde as WCHAR,
        to: 0x7f0e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7de0 as WCHAR,
        to: 0x7f14 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7de1 as WCHAR,
        to: 0x7f17 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7de3 as WCHAR,
        to: 0x7f18 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7de6 as WCHAR,
        to: 0x7f0c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7de8 as WCHAR,
        to: 0x7f16 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7de9 as WCHAR,
        to: 0x7f13 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7dec as WCHAR,
        to: 0x7f05 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7def as WCHAR,
        to: 0x7eac as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7df1 as WCHAR,
        to: 0x7f11 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7df2 as WCHAR,
        to: 0x7f08 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7df4 as WCHAR,
        to: 0x7ec3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7df6 as WCHAR,
        to: 0x7f0f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7df9 as WCHAR,
        to: 0x7f07 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7dfb as WCHAR,
        to: 0x81f4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e08 as WCHAR,
        to: 0x8426 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e09 as WCHAR,
        to: 0x7f19 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e0a as WCHAR,
        to: 0x7f22 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e0b as WCHAR,
        to: 0x7f12 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e10 as WCHAR,
        to: 0x7ec9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e11 as WCHAR,
        to: 0x7f23 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e1a as WCHAR,
        to: 0x7ee6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e1b as WCHAR,
        to: 0x7f1a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e1d as WCHAR,
        to: 0x7f1c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e1e as WCHAR,
        to: 0x7f1f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e1f as WCHAR,
        to: 0x7f1b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e23 as WCHAR,
        to: 0x53bf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e2b as WCHAR,
        to: 0x7f1d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e2d as WCHAR,
        to: 0x7f21 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e2e as WCHAR,
        to: 0x7f29 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e2f as WCHAR,
        to: 0x6f14 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e31 as WCHAR,
        to: 0x7eb5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e32 as WCHAR,
        to: 0x7f27 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e33 as WCHAR,
        to: 0x7f1a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e34 as WCHAR,
        to: 0x7ea4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e35 as WCHAR,
        to: 0x7f26 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e36 as WCHAR,
        to: 0x7d77 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e37 as WCHAR,
        to: 0x7f15 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e39 as WCHAR,
        to: 0x7f25 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e3d as WCHAR,
        to: 0x603b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e3e as WCHAR,
        to: 0x7ee9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e43 as WCHAR,
        to: 0x7ef7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e45 as WCHAR,
        to: 0x7f2b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e46 as WCHAR,
        to: 0x7f2a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e48 as WCHAR,
        to: 0x8941 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e52 as WCHAR,
        to: 0x7f2f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e54 as WCHAR,
        to: 0x7ec7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e55 as WCHAR,
        to: 0x7f2e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e5a as WCHAR,
        to: 0x7f2d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e5e as WCHAR,
        to: 0x7ed5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e61 as WCHAR,
        to: 0x7ee3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e62 as WCHAR,
        to: 0x7f0b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e69 as WCHAR,
        to: 0x7ef3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e6a as WCHAR,
        to: 0x7ed8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e6b as WCHAR,
        to: 0x7cfb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e6d as WCHAR,
        to: 0x8327 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e6f as WCHAR,
        to: 0x7f33 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e70 as WCHAR,
        to: 0x7f32 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e73 as WCHAR,
        to: 0x7f34 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e79 as WCHAR,
        to: 0x7ece as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e7c as WCHAR,
        to: 0x7ee7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e7d as WCHAR,
        to: 0x7f24 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e7e as WCHAR,
        to: 0x7f31 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e88 as WCHAR,
        to: 0x7f2c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e8a as WCHAR,
        to: 0x7ea9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e8c as WCHAR,
        to: 0x7eed as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e8d as WCHAR,
        to: 0x7d2f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e8f as WCHAR,
        to: 0x7f20 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e93 as WCHAR,
        to: 0x7f28 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e94 as WCHAR,
        to: 0x624d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e96 as WCHAR,
        to: 0x7ea4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e98 as WCHAR,
        to: 0x7f35 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7e9c as WCHAR,
        to: 0x7f06 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f3d as WCHAR,
        to: 0x94b5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f48 as WCHAR,
        to: 0x575b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f4c as WCHAR,
        to: 0x7f42 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f70 as WCHAR,
        to: 0x7f5a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f75 as WCHAR,
        to: 0x9a82 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f77 as WCHAR,
        to: 0x7f62 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f85 as WCHAR,
        to: 0x7f57 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f86 as WCHAR,
        to: 0x7f74 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f88 as WCHAR,
        to: 0x7f81 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7f8b as WCHAR,
        to: 0x8288 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7fa5 as WCHAR,
        to: 0x7f9f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7fa8 as WCHAR,
        to: 0x7fa1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7fa9 as WCHAR,
        to: 0x4e49 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7fd2 as WCHAR,
        to: 0x4e60 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x7ff9 as WCHAR,
        to: 0x7fd8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8011 as WCHAR,
        to: 0x7aef as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x802c as WCHAR,
        to: 0x8027 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8056 as WCHAR,
        to: 0x5723 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x805e as WCHAR,
        to: 0x95fb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x806f as WCHAR,
        to: 0x8054 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8070 as WCHAR,
        to: 0x806a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8072 as WCHAR,
        to: 0x58f0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8073 as WCHAR,
        to: 0x8038 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8075 as WCHAR,
        to: 0x8069 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8076 as WCHAR,
        to: 0x8042 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8077 as WCHAR,
        to: 0x804c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8079 as WCHAR,
        to: 0x804d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x807d as WCHAR,
        to: 0x542c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x807e as WCHAR,
        to: 0x804b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8085 as WCHAR,
        to: 0x8083 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8105 as WCHAR,
        to: 0x80c1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8108 as WCHAR,
        to: 0x8109 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x811b as WCHAR,
        to: 0x80eb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8123 as WCHAR,
        to: 0x5507 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8127 as WCHAR,
        to: 0x7743 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8129 as WCHAR,
        to: 0x4fee as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x812b as WCHAR,
        to: 0x8131 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8139 as WCHAR,
        to: 0x80c0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x814e as WCHAR,
        to: 0x80be as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8161 as WCHAR,
        to: 0x8136 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8166 as WCHAR,
        to: 0x8111 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x816b as WCHAR,
        to: 0x80bf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8173 as WCHAR,
        to: 0x811a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8178 as WCHAR,
        to: 0x80a0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8183 as WCHAR,
        to: 0x817d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x819a as WCHAR,
        to: 0x80a4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x81a0 as WCHAR,
        to: 0x80f6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x81a9 as WCHAR,
        to: 0x817b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x81bd as WCHAR,
        to: 0x80c6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x81be as WCHAR,
        to: 0x810d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x81bf as WCHAR,
        to: 0x8113 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x81c9 as WCHAR,
        to: 0x8138 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x81cd as WCHAR,
        to: 0x8110 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x81cf as WCHAR,
        to: 0x8191 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x81d8 as WCHAR,
        to: 0x814a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x81da as WCHAR,
        to: 0x80ea as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x81df as WCHAR,
        to: 0x810f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x81e0 as WCHAR,
        to: 0x8114 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x81e5 as WCHAR,
        to: 0x5367 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x81e8 as WCHAR,
        to: 0x4e34 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x81fa as WCHAR,
        to: 0x53f0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8207 as WCHAR,
        to: 0x4e0e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8208 as WCHAR,
        to: 0x5174 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8209 as WCHAR,
        to: 0x4e3e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x820a as WCHAR,
        to: 0x65e7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8259 as WCHAR,
        to: 0x8231 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8263 as WCHAR,
        to: 0x6a79 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8264 as WCHAR,
        to: 0x8223 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8266 as WCHAR,
        to: 0x8230 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x826b as WCHAR,
        to: 0x823b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8271 as WCHAR,
        to: 0x8270 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8277 as WCHAR,
        to: 0x8273 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8278 as WCHAR,
        to: 0x8279 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8290 as WCHAR,
        to: 0x82c4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x82bb as WCHAR,
        to: 0x520d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x82e7 as WCHAR,
        to: 0x82ce as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x830d as WCHAR,
        to: 0x82df as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8332 as WCHAR,
        to: 0x5179 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x834a as WCHAR,
        to: 0x8346 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x838a as WCHAR,
        to: 0x5e84 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8396 as WCHAR,
        to: 0x830e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x83a2 as WCHAR,
        to: 0x835a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x83a7 as WCHAR,
        to: 0x82cb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x83ef as WCHAR,
        to: 0x534e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x83f4 as WCHAR,
        to: 0x5eb5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8407 as WCHAR,
        to: 0x82cc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x840a as WCHAR,
        to: 0x83b1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x842c as WCHAR,
        to: 0x4e07 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8435 as WCHAR,
        to: 0x83b4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8449 as WCHAR,
        to: 0x53f6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8452 as WCHAR,
        to: 0x836d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8457 as WCHAR,
        to: 0x7740 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8466 as WCHAR,
        to: 0x82c7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x846f as WCHAR,
        to: 0x836f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8477 as WCHAR,
        to: 0x8364 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8494 as WCHAR,
        to: 0x83b3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x849e as WCHAR,
        to: 0x8385 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x84bc as WCHAR,
        to: 0x82cd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x84c0 as WCHAR,
        to: 0x836a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x84cb as WCHAR,
        to: 0x76d6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x84ee as WCHAR,
        to: 0x83b2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x84ef as WCHAR,
        to: 0x82c1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x84fd as WCHAR,
        to: 0x835c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8506 as WCHAR,
        to: 0x83f1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8514 as WCHAR,
        to: 0x535c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x851e as WCHAR,
        to: 0x848c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8523 as WCHAR,
        to: 0x848b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8525 as WCHAR,
        to: 0x8471 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8526 as WCHAR,
        to: 0x8311 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x852d as WCHAR,
        to: 0x836b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8541 as WCHAR,
        to: 0x8368 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8546 as WCHAR,
        to: 0x8487 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x854e as WCHAR,
        to: 0x835e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8553 as WCHAR,
        to: 0x82b8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8555 as WCHAR,
        to: 0x83b8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8558 as WCHAR,
        to: 0x835b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8562 as WCHAR,
        to: 0x8489 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8569 as WCHAR,
        to: 0x8361 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x856a as WCHAR,
        to: 0x829c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x856d as WCHAR,
        to: 0x8427 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8577 as WCHAR,
        to: 0x84e3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8588 as WCHAR,
        to: 0x835f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x858a as WCHAR,
        to: 0x84df as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x858c as WCHAR,
        to: 0x8297 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8591 as WCHAR,
        to: 0x59dc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8594 as WCHAR,
        to: 0x8537 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x859f as WCHAR,
        to: 0x83b6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x85a6 as WCHAR,
        to: 0x8350 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x85a7 as WCHAR,
        to: 0x69c1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x85a9 as WCHAR,
        to: 0x8428 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x85ba as WCHAR,
        to: 0x8360 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x85cd as WCHAR,
        to: 0x84dd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x85ce as WCHAR,
        to: 0x8369 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x85dd as WCHAR,
        to: 0x827a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x85e5 as WCHAR,
        to: 0x836f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x85ea as WCHAR,
        to: 0x85ae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x85f6 as WCHAR,
        to: 0x82c8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x85f7 as WCHAR,
        to: 0x85af as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x85f9 as WCHAR,
        to: 0x853c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x85fa as WCHAR,
        to: 0x853a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8604 as WCHAR,
        to: 0x8572 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8606 as WCHAR,
        to: 0x82a6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8607 as WCHAR,
        to: 0x82cf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x860a as WCHAR,
        to: 0x8574 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x860b as WCHAR,
        to: 0x82f9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8617 as WCHAR,
        to: 0x8616 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x861a as WCHAR,
        to: 0x85d3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x861e as WCHAR,
        to: 0x8539 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8622 as WCHAR,
        to: 0x830f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x862d as WCHAR,
        to: 0x5170 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x863a as WCHAR,
        to: 0x84e0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x863f as WCHAR,
        to: 0x841d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8655 as WCHAR,
        to: 0x5904 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x865b as WCHAR,
        to: 0x865a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x865c as WCHAR,
        to: 0x864f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x865f as WCHAR,
        to: 0x53f7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8667 as WCHAR,
        to: 0x4e8f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x86fa as WCHAR,
        to: 0x86f1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x86fb as WCHAR,
        to: 0x8715 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8706 as WCHAR,
        to: 0x86ac as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8755 as WCHAR,
        to: 0x8680 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8766 as WCHAR,
        to: 0x867e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8778 as WCHAR,
        to: 0x8717 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8784 as WCHAR,
        to: 0x86f3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x879e as WCHAR,
        to: 0x8682 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x87a2 as WCHAR,
        to: 0x8424 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x87bb as WCHAR,
        to: 0x877c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x87c4 as WCHAR,
        to: 0x86f0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x87c8 as WCHAR,
        to: 0x8748 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x87e3 as WCHAR,
        to: 0x866e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x87ec as WCHAR,
        to: 0x8749 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x87ef as WCHAR,
        to: 0x86f2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x87f2 as WCHAR,
        to: 0x866b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x87f6 as WCHAR,
        to: 0x86cf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x87fb as WCHAR,
        to: 0x8681 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8805 as WCHAR,
        to: 0x8747 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8806 as WCHAR,
        to: 0x867f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8810 as WCHAR,
        to: 0x86f4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8811 as WCHAR,
        to: 0x877e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8814 as WCHAR,
        to: 0x869d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x881f as WCHAR,
        to: 0x8721 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8823 as WCHAR,
        to: 0x86ce as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8831 as WCHAR,
        to: 0x86ca as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8836 as WCHAR,
        to: 0x8695 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x883b as WCHAR,
        to: 0x86ee as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x884a as WCHAR,
        to: 0x8511 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8853 as WCHAR,
        to: 0x672f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x885b as WCHAR,
        to: 0x536b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x885d as WCHAR,
        to: 0x51b2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8879 as WCHAR,
        to: 0x53ea as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x889e as WCHAR,
        to: 0x886e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x88ca as WCHAR,
        to: 0x8885 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x88cf as WCHAR,
        to: 0x91cc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x88dc as WCHAR,
        to: 0x8865 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x88dd as WCHAR,
        to: 0x88c5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x88e1 as WCHAR,
        to: 0x91cc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x88fd as WCHAR,
        to: 0x5236 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8907 as WCHAR,
        to: 0x590d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8932 as WCHAR,
        to: 0x88e4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8933 as WCHAR,
        to: 0x88e2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8938 as WCHAR,
        to: 0x891b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x893b as WCHAR,
        to: 0x4eb5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8956 as WCHAR,
        to: 0x8884 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x895d as WCHAR,
        to: 0x88e3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8960 as WCHAR,
        to: 0x88c6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8964 as WCHAR,
        to: 0x8934 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x896a as WCHAR,
        to: 0x889c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x896c as WCHAR,
        to: 0x6446 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x896f as WCHAR,
        to: 0x886c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8972 as WCHAR,
        to: 0x88ad as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8988 as WCHAR,
        to: 0x6838 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x898b as WCHAR,
        to: 0x89c1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x898f as WCHAR,
        to: 0x89c4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8993 as WCHAR,
        to: 0x89c5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8996 as WCHAR,
        to: 0x89c6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8998 as WCHAR,
        to: 0x89c7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89a1 as WCHAR,
        to: 0x89cb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89a6 as WCHAR,
        to: 0x89ce as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89aa as WCHAR,
        to: 0x4eb2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89ac as WCHAR,
        to: 0x89ca as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89af as WCHAR,
        to: 0x89cf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89b2 as WCHAR,
        to: 0x89d0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89b7 as WCHAR,
        to: 0x89d1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89ba as WCHAR,
        to: 0x89c9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89bd as WCHAR,
        to: 0x89c8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89bf as WCHAR,
        to: 0x89cc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89c0 as WCHAR,
        to: 0x89c2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89d4 as WCHAR,
        to: 0x7b4b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89f4 as WCHAR,
        to: 0x89de as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89f6 as WCHAR,
        to: 0x89ef as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x89f8 as WCHAR,
        to: 0x89e6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a02 as WCHAR,
        to: 0x8ba2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a03 as WCHAR,
        to: 0x8ba3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a08 as WCHAR,
        to: 0x8ba1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a0a as WCHAR,
        to: 0x8baf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a0c as WCHAR,
        to: 0x8ba7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a0e as WCHAR,
        to: 0x8ba8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a10 as WCHAR,
        to: 0x8ba6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a13 as WCHAR,
        to: 0x8bad as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a15 as WCHAR,
        to: 0x8baa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a16 as WCHAR,
        to: 0x8bab as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a17 as WCHAR,
        to: 0x6258 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a18 as WCHAR,
        to: 0x8bb0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a1b as WCHAR,
        to: 0x8bb9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a1d as WCHAR,
        to: 0x8bb6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a1f as WCHAR,
        to: 0x8bbc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a22 as WCHAR,
        to: 0x6b23 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a23 as WCHAR,
        to: 0x8bc0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a25 as WCHAR,
        to: 0x8bb7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a2a as WCHAR,
        to: 0x8bbf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a2d as WCHAR,
        to: 0x8bbe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a31 as WCHAR,
        to: 0x8bb8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a34 as WCHAR,
        to: 0x8bc9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a36 as WCHAR,
        to: 0x8bc3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a3a as WCHAR,
        to: 0x8bca as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a3b as WCHAR,
        to: 0x6ce8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a3c as WCHAR,
        to: 0x8bc1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a41 as WCHAR,
        to: 0x8bc2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a46 as WCHAR,
        to: 0x8bcb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a4e as WCHAR,
        to: 0x8bb5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a50 as WCHAR,
        to: 0x8bc8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a52 as WCHAR,
        to: 0x8bd2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a54 as WCHAR,
        to: 0x8bcf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a55 as WCHAR,
        to: 0x8bc4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a58 as WCHAR,
        to: 0x8bce as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a5b as WCHAR,
        to: 0x8bc5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a5e as WCHAR,
        to: 0x8bcd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a60 as WCHAR,
        to: 0x548f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a61 as WCHAR,
        to: 0x8be9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a62 as WCHAR,
        to: 0x8be2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a63 as WCHAR,
        to: 0x8be3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a66 as WCHAR,
        to: 0x8bd5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a69 as WCHAR,
        to: 0x8bd7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a6b as WCHAR,
        to: 0x8be7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a6c as WCHAR,
        to: 0x8bdf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a6d as WCHAR,
        to: 0x8be1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a6e as WCHAR,
        to: 0x8be0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a70 as WCHAR,
        to: 0x8bd8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a71 as WCHAR,
        to: 0x8bdd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a72 as WCHAR,
        to: 0x8be5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a73 as WCHAR,
        to: 0x8be6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a75 as WCHAR,
        to: 0x8bdc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a7c as WCHAR,
        to: 0x8bd9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a7f as WCHAR,
        to: 0x8bd6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a84 as WCHAR,
        to: 0x8bd4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a85 as WCHAR,
        to: 0x8bdb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a86 as WCHAR,
        to: 0x8bd3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a87 as WCHAR,
        to: 0x5938 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a8c as WCHAR,
        to: 0x5fd7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a8d as WCHAR,
        to: 0x8ba4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a91 as WCHAR,
        to: 0x8bf3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a92 as WCHAR,
        to: 0x8bf6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a95 as WCHAR,
        to: 0x8bde as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a98 as WCHAR,
        to: 0x8bf1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a9a as WCHAR,
        to: 0x8bee as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8a9e as WCHAR,
        to: 0x8bed as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8aa0 as WCHAR,
        to: 0x8bda as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8aa1 as WCHAR,
        to: 0x8beb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8aa3 as WCHAR,
        to: 0x8bec as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8aa4 as WCHAR,
        to: 0x8bef as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8aa5 as WCHAR,
        to: 0x8bf0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8aa6 as WCHAR,
        to: 0x8bf5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8aa8 as WCHAR,
        to: 0x8bf2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8aaa as WCHAR,
        to: 0x8bf4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ab0 as WCHAR,
        to: 0x8c01 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ab2 as WCHAR,
        to: 0x8bfe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ab6 as WCHAR,
        to: 0x8c07 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ab9 as WCHAR,
        to: 0x8bfd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8abc as WCHAR,
        to: 0x8c0a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8abf as WCHAR,
        to: 0x8c03 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ac2 as WCHAR,
        to: 0x8c04 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ac4 as WCHAR,
        to: 0x8c06 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ac7 as WCHAR,
        to: 0x8c08 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ac9 as WCHAR,
        to: 0x8bff as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8acb as WCHAR,
        to: 0x8bf7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8acd as WCHAR,
        to: 0x8be4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8acf as WCHAR,
        to: 0x8bf9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ad1 as WCHAR,
        to: 0x8bfc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ad2 as WCHAR,
        to: 0x8c05 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ad6 as WCHAR,
        to: 0x8bba as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ad7 as WCHAR,
        to: 0x8c02 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8adb as WCHAR,
        to: 0x8c00 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8adc as WCHAR,
        to: 0x8c0d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ade as WCHAR,
        to: 0x8c1d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ae2 as WCHAR,
        to: 0x8be8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ae4 as WCHAR,
        to: 0x8c14 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ae6 as WCHAR,
        to: 0x8c1b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ae7 as WCHAR,
        to: 0x8c10 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8aeb as WCHAR,
        to: 0x8c0f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8aed as WCHAR,
        to: 0x8c15 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8aee as WCHAR,
        to: 0x8c18 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8af1 as WCHAR,
        to: 0x8bb3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8af3 as WCHAR,
        to: 0x8c19 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8af6 as WCHAR,
        to: 0x8c0c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8af7 as WCHAR,
        to: 0x8bbd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8af8 as WCHAR,
        to: 0x8bf8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8afa as WCHAR,
        to: 0x8c1a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8afc as WCHAR,
        to: 0x8c16 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8afe as WCHAR,
        to: 0x8bfa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b00 as WCHAR,
        to: 0x8c0b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b01 as WCHAR,
        to: 0x8c12 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b02 as WCHAR,
        to: 0x8c13 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b04 as WCHAR,
        to: 0x8a8a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b05 as WCHAR,
        to: 0x8bcc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b0a as WCHAR,
        to: 0x8c0e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b0e as WCHAR,
        to: 0x8c1c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b10 as WCHAR,
        to: 0x8c27 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b14 as WCHAR,
        to: 0x8c11 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b16 as WCHAR,
        to: 0x8c21 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b17 as WCHAR,
        to: 0x8c24 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b19 as WCHAR,
        to: 0x8c26 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b1a as WCHAR,
        to: 0x8c25 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b1b as WCHAR,
        to: 0x8bb2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b1d as WCHAR,
        to: 0x8c22 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b20 as WCHAR,
        to: 0x8c23 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b28 as WCHAR,
        to: 0x8c1f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b2b as WCHAR,
        to: 0x8c2a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b2c as WCHAR,
        to: 0x8c2c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b33 as WCHAR,
        to: 0x8bb4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b39 as WCHAR,
        to: 0x8c28 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b3e as WCHAR,
        to: 0x8c29 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b49 as WCHAR,
        to: 0x8bc1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b4e as WCHAR,
        to: 0x8c32 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b4f as WCHAR,
        to: 0x8ba5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b56 as WCHAR,
        to: 0x8c2e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b58 as WCHAR,
        to: 0x8bc6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b59 as WCHAR,
        to: 0x8c2f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b5a as WCHAR,
        to: 0x8c2d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b5c as WCHAR,
        to: 0x8c31 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b5f as WCHAR,
        to: 0x566a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b6b as WCHAR,
        to: 0x8c35 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b6f as WCHAR,
        to: 0x8bd1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b70 as WCHAR,
        to: 0x8bae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b74 as WCHAR,
        to: 0x8c34 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b77 as WCHAR,
        to: 0x62a4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b7d as WCHAR,
        to: 0x8a89 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b80 as WCHAR,
        to: 0x8bfb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b8a as WCHAR,
        to: 0x53d8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b8e as WCHAR,
        to: 0x96e0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b92 as WCHAR,
        to: 0x8c17 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b93 as WCHAR,
        to: 0x8ba9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b95 as WCHAR,
        to: 0x8c30 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b96 as WCHAR,
        to: 0x8c36 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b9a as WCHAR,
        to: 0x8d5e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b9c as WCHAR,
        to: 0x8c20 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8b9e as WCHAR,
        to: 0x8c33 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c48 as WCHAR,
        to: 0x5c82 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c4e as WCHAR,
        to: 0x7ad6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c50 as WCHAR,
        to: 0x4e30 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c54 as WCHAR,
        to: 0x8273 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c6c as WCHAR,
        to: 0x732a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c8d as WCHAR,
        to: 0x72f8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c93 as WCHAR,
        to: 0x732b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c9d as WCHAR,
        to: 0x8d1d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8c9e as WCHAR,
        to: 0x8d1e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ca0 as WCHAR,
        to: 0x8d1f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ca1 as WCHAR,
        to: 0x8d22 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ca2 as WCHAR,
        to: 0x8d21 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ca7 as WCHAR,
        to: 0x8d2b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ca8 as WCHAR,
        to: 0x8d27 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ca9 as WCHAR,
        to: 0x8d29 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8caa as WCHAR,
        to: 0x8d2a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cab as WCHAR,
        to: 0x8d2f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cac as WCHAR,
        to: 0x8d23 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8caf as WCHAR,
        to: 0x8d2e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cb0 as WCHAR,
        to: 0x8d33 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cb2 as WCHAR,
        to: 0x8d40 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cb3 as WCHAR,
        to: 0x8d30 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cb4 as WCHAR,
        to: 0x8d35 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cb6 as WCHAR,
        to: 0x8d2c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cb7 as WCHAR,
        to: 0x4e70 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cb8 as WCHAR,
        to: 0x8d37 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cba as WCHAR,
        to: 0x8d36 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cbb as WCHAR,
        to: 0x8d39 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cbc as WCHAR,
        to: 0x8d34 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cbd as WCHAR,
        to: 0x8d3b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cbf as WCHAR,
        to: 0x8d38 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cc0 as WCHAR,
        to: 0x8d3a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cc1 as WCHAR,
        to: 0x8d32 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cc2 as WCHAR,
        to: 0x8d42 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cc3 as WCHAR,
        to: 0x8d41 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cc4 as WCHAR,
        to: 0x8d3f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cc5 as WCHAR,
        to: 0x8d45 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cc7 as WCHAR,
        to: 0x8d44 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cc8 as WCHAR,
        to: 0x8d3e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cca as WCHAR,
        to: 0x8d3c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cd1 as WCHAR,
        to: 0x8d48 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cd2 as WCHAR,
        to: 0x8d4a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cd3 as WCHAR,
        to: 0x5bbe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cd5 as WCHAR,
        to: 0x8d47 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cda as WCHAR,
        to: 0x8d49 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cdc as WCHAR,
        to: 0x8d50 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cde as WCHAR,
        to: 0x8d4f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ce0 as WCHAR,
        to: 0x8d54 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ce1 as WCHAR,
        to: 0x8d53 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ce2 as WCHAR,
        to: 0x8d24 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ce3 as WCHAR,
        to: 0x5356 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ce4 as WCHAR,
        to: 0x8d31 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ce6 as WCHAR,
        to: 0x8d4b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ce7 as WCHAR,
        to: 0x8d55 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cea as WCHAR,
        to: 0x8d28 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cec as WCHAR,
        to: 0x8d26 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ced as WCHAR,
        to: 0x8d4c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cf4 as WCHAR,
        to: 0x8d56 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cfa as WCHAR,
        to: 0x8d5a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cfb as WCHAR,
        to: 0x8d59 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cfc as WCHAR,
        to: 0x8d2d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cfd as WCHAR,
        to: 0x8d5b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8cfe as WCHAR,
        to: 0x8d5c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d04 as WCHAR,
        to: 0x8d3d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d05 as WCHAR,
        to: 0x8d58 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d08 as WCHAR,
        to: 0x8d60 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d0a as WCHAR,
        to: 0x8d5e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d0d as WCHAR,
        to: 0x8d61 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d0f as WCHAR,
        to: 0x8d62 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d10 as WCHAR,
        to: 0x8d46 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d13 as WCHAR,
        to: 0x8d43 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d16 as WCHAR,
        to: 0x8d4e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d17 as WCHAR,
        to: 0x8d5d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d1b as WCHAR,
        to: 0x8d63 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d95 as WCHAR,
        to: 0x8d76 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8d99 as WCHAR,
        to: 0x8d75 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8da8 as WCHAR,
        to: 0x8d8b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8db2 as WCHAR,
        to: 0x8db1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8de1 as WCHAR,
        to: 0x8ff9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8dfc as WCHAR,
        to: 0x5c40 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e10 as WCHAR,
        to: 0x8df5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e21 as WCHAR,
        to: 0x8737 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e2b as WCHAR,
        to: 0x78b0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e34 as WCHAR,
        to: 0x8e0a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e4c as WCHAR,
        to: 0x8dc4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e55 as WCHAR,
        to: 0x8df8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e60 as WCHAR,
        to: 0x8dd6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e63 as WCHAR,
        to: 0x8e52 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e64 as WCHAR,
        to: 0x8e2a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e7a as WCHAR,
        to: 0x8df7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e89 as WCHAR,
        to: 0x8db8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e8a as WCHAR,
        to: 0x8e0c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e8b as WCHAR,
        to: 0x8dfb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e8d as WCHAR,
        to: 0x8dc3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e91 as WCHAR,
        to: 0x8e2f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e92 as WCHAR,
        to: 0x8dde as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e93 as WCHAR,
        to: 0x8e2c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e95 as WCHAR,
        to: 0x8e70 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8e9a as WCHAR,
        to: 0x8df9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ea1 as WCHAR,
        to: 0x8e51 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ea5 as WCHAR,
        to: 0x8e7f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ea6 as WCHAR,
        to: 0x8e9c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8eaa as WCHAR,
        to: 0x8e8f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ec0 as WCHAR,
        to: 0x8eaf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8eca as WCHAR,
        to: 0x8f66 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ecb as WCHAR,
        to: 0x8f67 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ecc as WCHAR,
        to: 0x8f68 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ecd as WCHAR,
        to: 0x519b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ed2 as WCHAR,
        to: 0x8f69 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ed4 as WCHAR,
        to: 0x8f6b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8edb as WCHAR,
        to: 0x8f6d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8edf as WCHAR,
        to: 0x8f6f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8eeb as WCHAR,
        to: 0x8f78 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ef8 as WCHAR,
        to: 0x8f74 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ef9 as WCHAR,
        to: 0x8f75 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8efa as WCHAR,
        to: 0x8f7a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8efb as WCHAR,
        to: 0x8f72 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8efc as WCHAR,
        to: 0x8f76 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8efe as WCHAR,
        to: 0x8f7c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f03 as WCHAR,
        to: 0x8f83 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f05 as WCHAR,
        to: 0x8f82 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f07 as WCHAR,
        to: 0x8f81 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f09 as WCHAR,
        to: 0x8f7d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f0a as WCHAR,
        to: 0x8f7e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f12 as WCHAR,
        to: 0x8f84 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f13 as WCHAR,
        to: 0x633d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f14 as WCHAR,
        to: 0x8f85 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f15 as WCHAR,
        to: 0x8f7b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f1b as WCHAR,
        to: 0x8f86 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f1c as WCHAR,
        to: 0x8f8e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f1d as WCHAR,
        to: 0x8f89 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f1e as WCHAR,
        to: 0x8f8b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f1f as WCHAR,
        to: 0x8f8d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f25 as WCHAR,
        to: 0x8f8a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f26 as WCHAR,
        to: 0x8f87 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f29 as WCHAR,
        to: 0x8f88 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f2a as WCHAR,
        to: 0x8f6e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f2f as WCHAR,
        to: 0x8f91 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f33 as WCHAR,
        to: 0x8f8f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f38 as WCHAR,
        to: 0x8f93 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f3b as WCHAR,
        to: 0x8f90 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f3e as WCHAR,
        to: 0x8f97 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f3f as WCHAR,
        to: 0x8206 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f42 as WCHAR,
        to: 0x6bc2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f44 as WCHAR,
        to: 0x8f96 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f45 as WCHAR,
        to: 0x8f95 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f46 as WCHAR,
        to: 0x8f98 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f49 as WCHAR,
        to: 0x8f6c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f4d as WCHAR,
        to: 0x8f99 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f4e as WCHAR,
        to: 0x8f7f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f54 as WCHAR,
        to: 0x8f9a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f5f as WCHAR,
        to: 0x8f70 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f61 as WCHAR,
        to: 0x8f94 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f62 as WCHAR,
        to: 0x8f79 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8f64 as WCHAR,
        to: 0x8f73 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8fa6 as WCHAR,
        to: 0x529e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8fad as WCHAR,
        to: 0x8f9e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8fae as WCHAR,
        to: 0x8fab as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8faf as WCHAR,
        to: 0x8fa9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8fb2 as WCHAR,
        to: 0x519c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8fc6 as WCHAR,
        to: 0x8fe4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ff4 as WCHAR,
        to: 0x56de as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x8ffa as WCHAR,
        to: 0x4e43 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9015 as WCHAR,
        to: 0x8ff3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9019 as WCHAR,
        to: 0x8fd9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9023 as WCHAR,
        to: 0x8fde as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9031 as WCHAR,
        to: 0x5468 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9032 as WCHAR,
        to: 0x8fdb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x904a as WCHAR,
        to: 0x6e38 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x904b as WCHAR,
        to: 0x8fd0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x904e as WCHAR,
        to: 0x8fc7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9054 as WCHAR,
        to: 0x8fbe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9055 as WCHAR,
        to: 0x8fdd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9059 as WCHAR,
        to: 0x9065 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x905c as WCHAR,
        to: 0x900a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x905e as WCHAR,
        to: 0x9012 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9060 as WCHAR,
        to: 0x8fdc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9069 as WCHAR,
        to: 0x9002 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9072 as WCHAR,
        to: 0x8fdf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9077 as WCHAR,
        to: 0x8fc1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9078 as WCHAR,
        to: 0x9009 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x907a as WCHAR,
        to: 0x9057 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x907c as WCHAR,
        to: 0x8fbd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9081 as WCHAR,
        to: 0x8fc8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9084 as WCHAR,
        to: 0x8fd8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9087 as WCHAR,
        to: 0x8fe9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x908a as WCHAR,
        to: 0x8fb9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x908f as WCHAR,
        to: 0x903b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9090 as WCHAR,
        to: 0x9026 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x90c3 as WCHAR,
        to: 0x5408 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x90df as WCHAR,
        to: 0x90cf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x90f5 as WCHAR,
        to: 0x90ae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9106 as WCHAR,
        to: 0x90d3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9109 as WCHAR,
        to: 0x4e61 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9112 as WCHAR,
        to: 0x90b9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9114 as WCHAR,
        to: 0x90ac as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9116 as WCHAR,
        to: 0x90e7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9127 as WCHAR,
        to: 0x9093 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x912d as WCHAR,
        to: 0x90d1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9130 as WCHAR,
        to: 0x90bb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9132 as WCHAR,
        to: 0x90f8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9134 as WCHAR,
        to: 0x90ba as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9136 as WCHAR,
        to: 0x90d0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x913a as WCHAR,
        to: 0x909d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9148 as WCHAR,
        to: 0x90e6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9183 as WCHAR,
        to: 0x814c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x919c as WCHAR,
        to: 0x4e11 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x919e as WCHAR,
        to: 0x915d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x91ab as WCHAR,
        to: 0x533b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x91ac as WCHAR,
        to: 0x9171 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x91c0 as WCHAR,
        to: 0x917f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x91c1 as WCHAR,
        to: 0x8845 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x91c3 as WCHAR,
        to: 0x917e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x91c5 as WCHAR,
        to: 0x917d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x91cb as WCHAR,
        to: 0x91ca as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x91d0 as WCHAR,
        to: 0x5398 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x91d3 as WCHAR,
        to: 0x9486 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x91d4 as WCHAR,
        to: 0x9487 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x91d5 as WCHAR,
        to: 0x948c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x91d7 as WCHAR,
        to: 0x948a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x91d8 as WCHAR,
        to: 0x9489 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x91d9 as WCHAR,
        to: 0x948b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x91dd as WCHAR,
        to: 0x9488 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x91e3 as WCHAR,
        to: 0x9493 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x91e4 as WCHAR,
        to: 0x9490 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x91e6 as WCHAR,
        to: 0x6263 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x91e7 as WCHAR,
        to: 0x948f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x91e9 as WCHAR,
        to: 0x9492 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x91ec as WCHAR,
        to: 0x948e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x91f5 as WCHAR,
        to: 0x9497 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x91f7 as WCHAR,
        to: 0x948d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x91f9 as WCHAR,
        to: 0x9495 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9200 as WCHAR,
        to: 0x94af as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9201 as WCHAR,
        to: 0x94ab as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9204 as WCHAR,
        to: 0x94ad as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9209 as WCHAR,
        to: 0x94a0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x920d as WCHAR,
        to: 0x949d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9210 as WCHAR,
        to: 0x94a4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9211 as WCHAR,
        to: 0x94a3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9214 as WCHAR,
        to: 0x949e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9215 as WCHAR,
        to: 0x94ae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x921e as WCHAR,
        to: 0x94a7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9223 as WCHAR,
        to: 0x9499 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9225 as WCHAR,
        to: 0x94ac as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9226 as WCHAR,
        to: 0x949b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9227 as WCHAR,
        to: 0x94aa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x922e as WCHAR,
        to: 0x94cc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9230 as WCHAR,
        to: 0x94c8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9233 as WCHAR,
        to: 0x94b6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9234 as WCHAR,
        to: 0x94c3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9237 as WCHAR,
        to: 0x94b4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9238 as WCHAR,
        to: 0x94b9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9239 as WCHAR,
        to: 0x94cd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x923a as WCHAR,
        to: 0x94b0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x923d as WCHAR,
        to: 0x94b8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x923e as WCHAR,
        to: 0x94c0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x923f as WCHAR,
        to: 0x94bf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9240 as WCHAR,
        to: 0x94be as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9245 as WCHAR,
        to: 0x949c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9246 as WCHAR,
        to: 0x94bb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9248 as WCHAR,
        to: 0x94ca as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9249 as WCHAR,
        to: 0x94c9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x924b as WCHAR,
        to: 0x5228 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x924d as WCHAR,
        to: 0x94cb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9251 as WCHAR,
        to: 0x94c2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9257 as WCHAR,
        to: 0x94b3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x925a as WCHAR,
        to: 0x94c6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x925b as WCHAR,
        to: 0x94c5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x925e as WCHAR,
        to: 0x94ba as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9264 as WCHAR,
        to: 0x94a9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9266 as WCHAR,
        to: 0x94b2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x926c as WCHAR,
        to: 0x94bc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x926d as WCHAR,
        to: 0x94bd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9278 as WCHAR,
        to: 0x94f0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x927a as WCHAR,
        to: 0x94d2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x927b as WCHAR,
        to: 0x94ec as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x927f as WCHAR,
        to: 0x94ea as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9280 as WCHAR,
        to: 0x94f6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9283 as WCHAR,
        to: 0x94f3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9285 as WCHAR,
        to: 0x94dc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9291 as WCHAR,
        to: 0x94e3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9293 as WCHAR,
        to: 0x94e8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9296 as WCHAR,
        to: 0x94e2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9298 as WCHAR,
        to: 0x94ed as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x929a as WCHAR,
        to: 0x94eb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x929c as WCHAR,
        to: 0x8854 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92a0 as WCHAR,
        to: 0x94d1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92a3 as WCHAR,
        to: 0x94f7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92a5 as WCHAR,
        to: 0x94f1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92a6 as WCHAR,
        to: 0x94df as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92a8 as WCHAR,
        to: 0x94f5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92a9 as WCHAR,
        to: 0x94e5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92aa as WCHAR,
        to: 0x94d5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92ab as WCHAR,
        to: 0x94ef as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92ac as WCHAR,
        to: 0x94d0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92b2 as WCHAR,
        to: 0x710a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92b3 as WCHAR,
        to: 0x9510 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92b7 as WCHAR,
        to: 0x9500 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92b9 as WCHAR,
        to: 0x9508 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92bb as WCHAR,
        to: 0x9511 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92bc as WCHAR,
        to: 0x9509 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92c1 as WCHAR,
        to: 0x94dd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92c3 as WCHAR,
        to: 0x9512 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92c5 as WCHAR,
        to: 0x950c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92c7 as WCHAR,
        to: 0x94a1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92cc as WCHAR,
        to: 0x94e4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92cf as WCHAR,
        to: 0x94d7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92d2 as WCHAR,
        to: 0x950b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92dd as WCHAR,
        to: 0x950a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92df as WCHAR,
        to: 0x9513 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92e4 as WCHAR,
        to: 0x9504 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92e6 as WCHAR,
        to: 0x9514 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92e8 as WCHAR,
        to: 0x9507 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92ea as WCHAR,
        to: 0x94fa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92ee as WCHAR,
        to: 0x94d6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92ef as WCHAR,
        to: 0x9506 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92f0 as WCHAR,
        to: 0x9502 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92f1 as WCHAR,
        to: 0x94fd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92f8 as WCHAR,
        to: 0x952f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x92fc as WCHAR,
        to: 0x94a2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9301 as WCHAR,
        to: 0x951e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9304 as WCHAR,
        to: 0x5f55 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9306 as WCHAR,
        to: 0x9516 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9308 as WCHAR,
        to: 0x9529 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9310 as WCHAR,
        to: 0x9525 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9312 as WCHAR,
        to: 0x9515 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9315 as WCHAR,
        to: 0x951f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9318 as WCHAR,
        to: 0x9524 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9319 as WCHAR,
        to: 0x9531 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x931a as WCHAR,
        to: 0x94ee as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x931b as WCHAR,
        to: 0x951b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x931f as WCHAR,
        to: 0x952c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9320 as WCHAR,
        to: 0x952d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9322 as WCHAR,
        to: 0x94b1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9326 as WCHAR,
        to: 0x9526 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9328 as WCHAR,
        to: 0x951a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x932b as WCHAR,
        to: 0x9521 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x932e as WCHAR,
        to: 0x9522 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x932f as WCHAR,
        to: 0x9519 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9333 as WCHAR,
        to: 0x9530 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9336 as WCHAR,
        to: 0x8868 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9338 as WCHAR,
        to: 0x94fc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9346 as WCHAR,
        to: 0x9494 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9347 as WCHAR,
        to: 0x9534 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x934a as WCHAR,
        to: 0x94fe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x934b as WCHAR,
        to: 0x9505 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x934d as WCHAR,
        to: 0x9540 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9354 as WCHAR,
        to: 0x9537 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9358 as WCHAR,
        to: 0x94e1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x935b as WCHAR,
        to: 0x953b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9364 as WCHAR,
        to: 0x9538 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9365 as WCHAR,
        to: 0x9532 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x936c as WCHAR,
        to: 0x9539 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9370 as WCHAR,
        to: 0x953e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9375 as WCHAR,
        to: 0x952e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9376 as WCHAR,
        to: 0x9536 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x937a as WCHAR,
        to: 0x9517 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x937e as WCHAR,
        to: 0x953a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9382 as WCHAR,
        to: 0x9541 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x938a as WCHAR,
        to: 0x9551 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9394 as WCHAR,
        to: 0x7194 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9396 as WCHAR,
        to: 0x9501 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9397 as WCHAR,
        to: 0x67aa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9398 as WCHAR,
        to: 0x9549 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93a2 as WCHAR,
        to: 0x94a8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93a3 as WCHAR,
        to: 0x84e5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93a6 as WCHAR,
        to: 0x954f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93a7 as WCHAR,
        to: 0x94e0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93a9 as WCHAR,
        to: 0x94e9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93aa as WCHAR,
        to: 0x953c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93ac as WCHAR,
        to: 0x9550 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93ae as WCHAR,
        to: 0x9547 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93b0 as WCHAR,
        to: 0x9552 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93b3 as WCHAR,
        to: 0x954d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93b5 as WCHAR,
        to: 0x9553 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93c3 as WCHAR,
        to: 0x955e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93c7 as WCHAR,
        to: 0x955f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93c8 as WCHAR,
        to: 0x94fe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93cc as WCHAR,
        to: 0x9546 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93cd as WCHAR,
        to: 0x9559 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93d1 as WCHAR,
        to: 0x955d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93d7 as WCHAR,
        to: 0x94ff as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93d8 as WCHAR,
        to: 0x9535 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93dc as WCHAR,
        to: 0x9557 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93dd as WCHAR,
        to: 0x9558 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93de as WCHAR,
        to: 0x955b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93df as WCHAR,
        to: 0x94f2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93e1 as WCHAR,
        to: 0x955c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93e2 as WCHAR,
        to: 0x9556 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93e4 as WCHAR,
        to: 0x9542 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93e8 as WCHAR,
        to: 0x933e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93f5 as WCHAR,
        to: 0x94e7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93f7 as WCHAR,
        to: 0x9564 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93f9 as WCHAR,
        to: 0x956a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x93fd as WCHAR,
        to: 0x9508 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9403 as WCHAR,
        to: 0x94d9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9409 as WCHAR,
        to: 0x94e3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x940b as WCHAR,
        to: 0x94f4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9410 as WCHAR,
        to: 0x9563 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9412 as WCHAR,
        to: 0x94f9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9413 as WCHAR,
        to: 0x9566 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9414 as WCHAR,
        to: 0x9561 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9418 as WCHAR,
        to: 0x949f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9419 as WCHAR,
        to: 0x956b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9420 as WCHAR,
        to: 0x9568 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9428 as WCHAR,
        to: 0x9544 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x942b as WCHAR,
        to: 0x954c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x942e as WCHAR,
        to: 0x9570 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9432 as WCHAR,
        to: 0x956f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9433 as WCHAR,
        to: 0x956d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9435 as WCHAR,
        to: 0x94c1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9438 as WCHAR,
        to: 0x94ce as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x943a as WCHAR,
        to: 0x94db as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x943d as WCHAR,
        to: 0x94bd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x943f as WCHAR,
        to: 0x9571 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9444 as WCHAR,
        to: 0x94f8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x944a as WCHAR,
        to: 0x956c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x944c as WCHAR,
        to: 0x9554 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9451 as WCHAR,
        to: 0x76d1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9452 as WCHAR,
        to: 0x9274 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9460 as WCHAR,
        to: 0x94c4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9463 as WCHAR,
        to: 0x9573 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9464 as WCHAR,
        to: 0x5228 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x946d as WCHAR,
        to: 0x9567 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9470 as WCHAR,
        to: 0x94a5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9472 as WCHAR,
        to: 0x9576 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9477 as WCHAR,
        to: 0x954a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x947c as WCHAR,
        to: 0x9523 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x947d as WCHAR,
        to: 0x94bb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x947e as WCHAR,
        to: 0x92ae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x947f as WCHAR,
        to: 0x51ff as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9577 as WCHAR,
        to: 0x957f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9580 as WCHAR,
        to: 0x95e8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9582 as WCHAR,
        to: 0x95e9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9583 as WCHAR,
        to: 0x95ea as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9586 as WCHAR,
        to: 0x95eb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9589 as WCHAR,
        to: 0x95ed as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x958b as WCHAR,
        to: 0x5f00 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x958c as WCHAR,
        to: 0x95f6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x958e as WCHAR,
        to: 0x95f3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x958f as WCHAR,
        to: 0x95f0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9591 as WCHAR,
        to: 0x95f2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9592 as WCHAR,
        to: 0x95f2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9593 as WCHAR,
        to: 0x95f4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9594 as WCHAR,
        to: 0x95f5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9598 as WCHAR,
        to: 0x95f8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95a1 as WCHAR,
        to: 0x9602 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95a3 as WCHAR,
        to: 0x9601 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95a4 as WCHAR,
        to: 0x5408 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95a5 as WCHAR,
        to: 0x9600 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95a8 as WCHAR,
        to: 0x95fa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95a9 as WCHAR,
        to: 0x95fd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95ab as WCHAR,
        to: 0x9603 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95ac as WCHAR,
        to: 0x9606 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95ad as WCHAR,
        to: 0x95fe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95b1 as WCHAR,
        to: 0x9605 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95b6 as WCHAR,
        to: 0x960a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95b9 as WCHAR,
        to: 0x9609 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95bb as WCHAR,
        to: 0x960e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95bc as WCHAR,
        to: 0x960f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95bd as WCHAR,
        to: 0x960d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95be as WCHAR,
        to: 0x9608 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95bf as WCHAR,
        to: 0x960c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95c3 as WCHAR,
        to: 0x9612 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95c6 as WCHAR,
        to: 0x677f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95c8 as WCHAR,
        to: 0x95f1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95ca as WCHAR,
        to: 0x9614 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95cb as WCHAR,
        to: 0x9615 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95cc as WCHAR,
        to: 0x9611 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95d0 as WCHAR,
        to: 0x9617 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95d4 as WCHAR,
        to: 0x9616 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95d5 as WCHAR,
        to: 0x9619 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95d6 as WCHAR,
        to: 0x95ef as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95dc as WCHAR,
        to: 0x5173 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95de as WCHAR,
        to: 0x961a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95e1 as WCHAR,
        to: 0x9610 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95e2 as WCHAR,
        to: 0x8f9f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x95e5 as WCHAR,
        to: 0x95fc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9628 as WCHAR,
        to: 0x5384 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x962f as WCHAR,
        to: 0x5740 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9658 as WCHAR,
        to: 0x9649 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x965c as WCHAR,
        to: 0x9655 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x965d as WCHAR,
        to: 0x9655 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9663 as WCHAR,
        to: 0x9635 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9670 as WCHAR,
        to: 0x9634 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9673 as WCHAR,
        to: 0x9648 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9678 as WCHAR,
        to: 0x9646 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x967d as WCHAR,
        to: 0x9633 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9684 as WCHAR,
        to: 0x5824 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9689 as WCHAR,
        to: 0x9667 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x968a as WCHAR,
        to: 0x961f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x968e as WCHAR,
        to: 0x9636 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9695 as WCHAR,
        to: 0x9668 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x969b as WCHAR,
        to: 0x9645 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x96a8 as WCHAR,
        to: 0x968f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x96aa as WCHAR,
        to: 0x9669 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x96b1 as WCHAR,
        to: 0x9690 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x96b4 as WCHAR,
        to: 0x9647 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x96b8 as WCHAR,
        to: 0x96b6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x96bb as WCHAR,
        to: 0x53ea as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x96cb as WCHAR,
        to: 0x96bd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x96d6 as WCHAR,
        to: 0x867d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x96d9 as WCHAR,
        to: 0x53cc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x96db as WCHAR,
        to: 0x96cf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x96dc as WCHAR,
        to: 0x6742 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x96de as WCHAR,
        to: 0x9e21 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x96e2 as WCHAR,
        to: 0x79bb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x96e3 as WCHAR,
        to: 0x96be as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x96f2 as WCHAR,
        to: 0x4e91 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x96fb as WCHAR,
        to: 0x7535 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9711 as WCHAR,
        to: 0x6cbe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9724 as WCHAR,
        to: 0x6e9c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9727 as WCHAR,
        to: 0x96fe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x973d as WCHAR,
        to: 0x9701 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9742 as WCHAR,
        to: 0x96f3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9744 as WCHAR,
        to: 0x972d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9748 as WCHAR,
        to: 0x7075 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x975a as WCHAR,
        to: 0x9753 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x975c as WCHAR,
        to: 0x9759 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9766 as WCHAR,
        to: 0x817c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9768 as WCHAR,
        to: 0x9765 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x978f as WCHAR,
        to: 0x5de9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x97a6 as WCHAR,
        to: 0x79cb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x97c1 as WCHAR,
        to: 0x7f30 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x97c3 as WCHAR,
        to: 0x9791 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x97c6 as WCHAR,
        to: 0x5343 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x97c9 as WCHAR,
        to: 0x97af as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x97cb as WCHAR,
        to: 0x97e6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x97cc as WCHAR,
        to: 0x97e7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x97d3 as WCHAR,
        to: 0x97e9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x97d9 as WCHAR,
        to: 0x97ea as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x97dc as WCHAR,
        to: 0x97ec as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x97de as WCHAR,
        to: 0x97eb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x97fb as WCHAR,
        to: 0x97f5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x97ff as WCHAR,
        to: 0x54cd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9801 as WCHAR,
        to: 0x9875 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9802 as WCHAR,
        to: 0x9876 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9803 as WCHAR,
        to: 0x9877 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9805 as WCHAR,
        to: 0x9879 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9806 as WCHAR,
        to: 0x987a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9807 as WCHAR,
        to: 0x9878 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9808 as WCHAR,
        to: 0x987b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x980a as WCHAR,
        to: 0x987c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x980c as WCHAR,
        to: 0x9882 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x980e as WCHAR,
        to: 0x9880 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x980f as WCHAR,
        to: 0x9883 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9810 as WCHAR,
        to: 0x9884 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9811 as WCHAR,
        to: 0x987d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9812 as WCHAR,
        to: 0x9881 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9813 as WCHAR,
        to: 0x987f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9817 as WCHAR,
        to: 0x9887 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9818 as WCHAR,
        to: 0x9886 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x981c as WCHAR,
        to: 0x988c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9821 as WCHAR,
        to: 0x9889 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9824 as WCHAR,
        to: 0x9890 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9826 as WCHAR,
        to: 0x988f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x982d as WCHAR,
        to: 0x5934 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9830 as WCHAR,
        to: 0x988a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9837 as WCHAR,
        to: 0x9894 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9838 as WCHAR,
        to: 0x9888 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9839 as WCHAR,
        to: 0x9893 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x983b as WCHAR,
        to: 0x9891 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9846 as WCHAR,
        to: 0x9897 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x984c as WCHAR,
        to: 0x9898 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x984d as WCHAR,
        to: 0x989d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x984e as WCHAR,
        to: 0x989a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x984f as WCHAR,
        to: 0x989c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9853 as WCHAR,
        to: 0x989b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9858 as WCHAR,
        to: 0x613f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9859 as WCHAR,
        to: 0x98a1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x985b as WCHAR,
        to: 0x98a0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x985e as WCHAR,
        to: 0x7c7b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9862 as WCHAR,
        to: 0x989f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9865 as WCHAR,
        to: 0x98a2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9867 as WCHAR,
        to: 0x987e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x986b as WCHAR,
        to: 0x98a4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x986f as WCHAR,
        to: 0x663e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9870 as WCHAR,
        to: 0x98a6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9871 as WCHAR,
        to: 0x9885 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9873 as WCHAR,
        to: 0x989e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9874 as WCHAR,
        to: 0x98a7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98a8 as WCHAR,
        to: 0x98ce as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98ae as WCHAR,
        to: 0x98d1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98af as WCHAR,
        to: 0x98d2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98b1 as WCHAR,
        to: 0x53f0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98b3 as WCHAR,
        to: 0x522e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98b6 as WCHAR,
        to: 0x98d3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98ba as WCHAR,
        to: 0x626c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98bc as WCHAR,
        to: 0x98d5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98c4 as WCHAR,
        to: 0x98d8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98c6 as WCHAR,
        to: 0x98d9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98db as WCHAR,
        to: 0x98de as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98e2 as WCHAR,
        to: 0x9965 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98e9 as WCHAR,
        to: 0x9968 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98ea as WCHAR,
        to: 0x996a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98eb as WCHAR,
        to: 0x996b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98ed as WCHAR,
        to: 0x996c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98ef as WCHAR,
        to: 0x996d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98f2 as WCHAR,
        to: 0x996e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98f4 as WCHAR,
        to: 0x9974 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98fc as WCHAR,
        to: 0x9972 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98fd as WCHAR,
        to: 0x9971 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x98fe as WCHAR,
        to: 0x9970 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9903 as WCHAR,
        to: 0x997a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9905 as WCHAR,
        to: 0x997c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9909 as WCHAR,
        to: 0x9977 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x990a as WCHAR,
        to: 0x517b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x990c as WCHAR,
        to: 0x9975 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9911 as WCHAR,
        to: 0x997d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9912 as WCHAR,
        to: 0x9981 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9913 as WCHAR,
        to: 0x997f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9918 as WCHAR,
        to: 0x4f59 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x991a as WCHAR,
        to: 0x80b4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x991b as WCHAR,
        to: 0x9984 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x991e as WCHAR,
        to: 0x996f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9921 as WCHAR,
        to: 0x9985 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9928 as WCHAR,
        to: 0x9986 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9933 as WCHAR,
        to: 0x9967 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9935 as WCHAR,
        to: 0x5582 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x993c as WCHAR,
        to: 0x9969 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x993d as WCHAR,
        to: 0x9988 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x993e as WCHAR,
        to: 0x998f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x993f as WCHAR,
        to: 0x998a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9943 as WCHAR,
        to: 0x998d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9945 as WCHAR,
        to: 0x9992 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9948 as WCHAR,
        to: 0x9990 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9949 as WCHAR,
        to: 0x9991 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x994b as WCHAR,
        to: 0x9988 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x994c as WCHAR,
        to: 0x9994 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9951 as WCHAR,
        to: 0x9965 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9952 as WCHAR,
        to: 0x9976 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9957 as WCHAR,
        to: 0x98e8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x995c as WCHAR,
        to: 0x990d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x995e as WCHAR,
        to: 0x998b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x99ac as WCHAR,
        to: 0x9a6c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x99ad as WCHAR,
        to: 0x9a6d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x99ae as WCHAR,
        to: 0x51af as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x99b1 as WCHAR,
        to: 0x9a6e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x99b3 as WCHAR,
        to: 0x9a70 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x99b4 as WCHAR,
        to: 0x9a6f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x99c1 as WCHAR,
        to: 0x9a73 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x99d0 as WCHAR,
        to: 0x9a7b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x99d1 as WCHAR,
        to: 0x9a7d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x99d2 as WCHAR,
        to: 0x9a79 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x99d4 as WCHAR,
        to: 0x9a75 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x99d5 as WCHAR,
        to: 0x9a7e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x99d8 as WCHAR,
        to: 0x9a80 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x99d9 as WCHAR,
        to: 0x9a78 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x99db as WCHAR,
        to: 0x9a76 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x99dd as WCHAR,
        to: 0x9a7c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x99df as WCHAR,
        to: 0x9a77 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x99e2 as WCHAR,
        to: 0x9a88 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x99ed as WCHAR,
        to: 0x9a87 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x99f1 as WCHAR,
        to: 0x9a86 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x99ff as WCHAR,
        to: 0x9a8f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a01 as WCHAR,
        to: 0x9a8b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a05 as WCHAR,
        to: 0x9a93 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a0d as WCHAR,
        to: 0x9a92 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a0e as WCHAR,
        to: 0x9a91 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a0f as WCHAR,
        to: 0x9a90 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a16 as WCHAR,
        to: 0x9a9b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a19 as WCHAR,
        to: 0x9a97 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a2b as WCHAR,
        to: 0x9a9e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a2d as WCHAR,
        to: 0x9a98 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a2e as WCHAR,
        to: 0x9a9d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a30 as WCHAR,
        to: 0x817e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a36 as WCHAR,
        to: 0x9a7a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a37 as WCHAR,
        to: 0x9a9a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a38 as WCHAR,
        to: 0x9a9f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a3e as WCHAR,
        to: 0x9aa1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a40 as WCHAR,
        to: 0x84e6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a41 as WCHAR,
        to: 0x9a9c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a42 as WCHAR,
        to: 0x9a96 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a43 as WCHAR,
        to: 0x9aa0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a44 as WCHAR,
        to: 0x9aa2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a45 as WCHAR,
        to: 0x9a71 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a4a as WCHAR,
        to: 0x9a85 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a4d as WCHAR,
        to: 0x9a81 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a4f as WCHAR,
        to: 0x9aa3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a55 as WCHAR,
        to: 0x9a84 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a57 as WCHAR,
        to: 0x9a8c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a5a as WCHAR,
        to: 0x60ca as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a5b as WCHAR,
        to: 0x9a7f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a5f as WCHAR,
        to: 0x9aa4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a62 as WCHAR,
        to: 0x9a74 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a64 as WCHAR,
        to: 0x9aa7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a65 as WCHAR,
        to: 0x9aa5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9a6a as WCHAR,
        to: 0x9a8a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9aaf as WCHAR,
        to: 0x80ae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9acf as WCHAR,
        to: 0x9ac5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ad2 as WCHAR,
        to: 0x810f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ad4 as WCHAR,
        to: 0x4f53 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ad5 as WCHAR,
        to: 0x9acc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ad6 as WCHAR,
        to: 0x9acb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9aee as WCHAR,
        to: 0x53d1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9b06 as WCHAR,
        to: 0x677e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9b0d as WCHAR,
        to: 0x80e1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9b1a as WCHAR,
        to: 0x987b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9b22 as WCHAR,
        to: 0x9b13 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9b25 as WCHAR,
        to: 0x6597 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9b27 as WCHAR,
        to: 0x95f9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9b28 as WCHAR,
        to: 0x54c4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9b29 as WCHAR,
        to: 0x960b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9b2e as WCHAR,
        to: 0x9604 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9b31 as WCHAR,
        to: 0x90c1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9b4e as WCHAR,
        to: 0x9b49 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9b58 as WCHAR,
        to: 0x9b47 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9b5a as WCHAR,
        to: 0x9c7c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9b6f as WCHAR,
        to: 0x9c81 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9b74 as WCHAR,
        to: 0x9c82 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9b77 as WCHAR,
        to: 0x9c7f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9b90 as WCHAR,
        to: 0x9c90 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9b91 as WCHAR,
        to: 0x9c8d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9b92 as WCHAR,
        to: 0x9c8b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9b9a as WCHAR,
        to: 0x9c92 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9b9e as WCHAR,
        to: 0x9c95 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9baa as WCHAR,
        to: 0x9c94 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9bab as WCHAR,
        to: 0x9c9b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9bad as WCHAR,
        to: 0x9c91 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9bae as WCHAR,
        to: 0x9c9c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9bc0 as WCHAR,
        to: 0x9ca7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9bc1 as WCHAR,
        to: 0x9ca0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9bc7 as WCHAR,
        to: 0x9ca9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9bc9 as WCHAR,
        to: 0x9ca4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9bca as WCHAR,
        to: 0x9ca8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9bd4 as WCHAR,
        to: 0x9cbb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9bd6 as WCHAR,
        to: 0x9cad as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9bdb as WCHAR,
        to: 0x9cb7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9be1 as WCHAR,
        to: 0x9cb1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9be2 as WCHAR,
        to: 0x9cb5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9be4 as WCHAR,
        to: 0x9cb2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9be7 as WCHAR,
        to: 0x9cb3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9be8 as WCHAR,
        to: 0x9cb8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9bea as WCHAR,
        to: 0x9cae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9beb as WCHAR,
        to: 0x9cb0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9bf0 as WCHAR,
        to: 0x9cb6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9bfd as WCHAR,
        to: 0x9cab as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c08 as WCHAR,
        to: 0x9cbd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c09 as WCHAR,
        to: 0x9cc7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c0d as WCHAR,
        to: 0x9cc5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c12 as WCHAR,
        to: 0x9cc6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c13 as WCHAR,
        to: 0x9cc3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c23 as WCHAR,
        to: 0x9ca5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c25 as WCHAR,
        to: 0x9ccf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c28 as WCHAR,
        to: 0x9cce as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c29 as WCHAR,
        to: 0x9cd0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c2d as WCHAR,
        to: 0x9ccd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c31 as WCHAR,
        to: 0x9ca2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c32 as WCHAR,
        to: 0x9ccc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c33 as WCHAR,
        to: 0x9cd3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c37 as WCHAR,
        to: 0x9ca6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c39 as WCHAR,
        to: 0x9ca3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c3b as WCHAR,
        to: 0x9cd7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c3e as WCHAR,
        to: 0x9cd4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c48 as WCHAR,
        to: 0x9cd5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c49 as WCHAR,
        to: 0x9cd6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c52 as WCHAR,
        to: 0x9cdf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c54 as WCHAR,
        to: 0x9cdd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c56 as WCHAR,
        to: 0x9cdc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c57 as WCHAR,
        to: 0x9cde as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c58 as WCHAR,
        to: 0x9c9f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c5f as WCHAR,
        to: 0x9c8e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c67 as WCHAR,
        to: 0x9ce2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c6d as WCHAR,
        to: 0x9c9a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c77 as WCHAR,
        to: 0x9cc4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c78 as WCHAR,
        to: 0x9c88 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9c7a as WCHAR,
        to: 0x9ca1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ce5 as WCHAR,
        to: 0x9e1f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ce7 as WCHAR,
        to: 0x51eb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ce9 as WCHAR,
        to: 0x9e20 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cf3 as WCHAR,
        to: 0x51e4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cf4 as WCHAR,
        to: 0x9e23 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9cf6 as WCHAR,
        to: 0x9e22 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d06 as WCHAR,
        to: 0x9e29 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d07 as WCHAR,
        to: 0x9e28 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d09 as WCHAR,
        to: 0x9e26 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d15 as WCHAR,
        to: 0x9e35 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d1b as WCHAR,
        to: 0x9e33 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d1d as WCHAR,
        to: 0x9e32 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d1f as WCHAR,
        to: 0x9e31 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d23 as WCHAR,
        to: 0x9e2a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d26 as WCHAR,
        to: 0x9e2f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d28 as WCHAR,
        to: 0x9e2d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d2f as WCHAR,
        to: 0x9e38 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d30 as WCHAR,
        to: 0x9e39 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d3b as WCHAR,
        to: 0x9e3f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d3f as WCHAR,
        to: 0x9e3d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d42 as WCHAR,
        to: 0x9e3a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d51 as WCHAR,
        to: 0x9e43 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d52 as WCHAR,
        to: 0x9e46 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d53 as WCHAR,
        to: 0x9e41 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d5c as WCHAR,
        to: 0x9e48 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d5d as WCHAR,
        to: 0x9e45 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d60 as WCHAR,
        to: 0x9e44 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d61 as WCHAR,
        to: 0x9e49 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d6a as WCHAR,
        to: 0x9e4c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d6c as WCHAR,
        to: 0x9e4f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d6f as WCHAR,
        to: 0x9e4e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d72 as WCHAR,
        to: 0x9e4a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d87 as WCHAR,
        to: 0x9e2b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d89 as WCHAR,
        to: 0x9e51 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d98 as WCHAR,
        to: 0x9e55 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9d9a as WCHAR,
        to: 0x9e57 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9da9 as WCHAR,
        to: 0x9e5c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9daf as WCHAR,
        to: 0x83ba as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9db1 as WCHAR,
        to: 0x9a9e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9db4 as WCHAR,
        to: 0x9e64 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9dbb as WCHAR,
        to: 0x9e58 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9dbc as WCHAR,
        to: 0x9e63 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9dbf as WCHAR,
        to: 0x9e5a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9dc2 as WCHAR,
        to: 0x9e5e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9dd3 as WCHAR,
        to: 0x9e67 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9dd7 as WCHAR,
        to: 0x9e25 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9dd9 as WCHAR,
        to: 0x9e37 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9dda as WCHAR,
        to: 0x9e68 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9de5 as WCHAR,
        to: 0x9e36 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9de6 as WCHAR,
        to: 0x9e6a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9def as WCHAR,
        to: 0x9e69 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9df2 as WCHAR,
        to: 0x9e6b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9df3 as WCHAR,
        to: 0x9e47 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9df4 as WCHAR,
        to: 0x9e47 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9df8 as WCHAR,
        to: 0x9e6c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9df9 as WCHAR,
        to: 0x9e70 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9dfa as WCHAR,
        to: 0x9e6d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e15 as WCHAR,
        to: 0x9e2c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e1a as WCHAR,
        to: 0x9e66 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e1b as WCHAR,
        to: 0x9e73 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e1d as WCHAR,
        to: 0x9e42 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e1e as WCHAR,
        to: 0x9e3e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e75 as WCHAR,
        to: 0x5364 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e79 as WCHAR,
        to: 0x54b8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e7a as WCHAR,
        to: 0x9e7e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e7c as WCHAR,
        to: 0x7877 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e7d as WCHAR,
        to: 0x76d0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9e97 as WCHAR,
        to: 0x4e3d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ea5 as WCHAR,
        to: 0x9ea6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ea9 as WCHAR,
        to: 0x9eb8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9eb5 as WCHAR,
        to: 0x9762 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ebc as WCHAR,
        to: 0x9ebd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ec3 as WCHAR,
        to: 0x9ec4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ecc as WCHAR,
        to: 0x9ec9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ede as WCHAR,
        to: 0x70b9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ee8 as WCHAR,
        to: 0x515a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ef2 as WCHAR,
        to: 0x9eea as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ef4 as WCHAR,
        to: 0x9709 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9ef7 as WCHAR,
        to: 0x9ee9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9efd as WCHAR,
        to: 0x9efe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9eff as WCHAR,
        to: 0x9f0b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f07 as WCHAR,
        to: 0x9ccc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f09 as WCHAR,
        to: 0x9f0d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f15 as WCHAR,
        to: 0x51ac as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f34 as WCHAR,
        to: 0x9f39 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f4a as WCHAR,
        to: 0x9f50 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f4b as WCHAR,
        to: 0x658b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f4f as WCHAR,
        to: 0x9f51 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f52 as WCHAR,
        to: 0x9f7f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f54 as WCHAR,
        to: 0x9f80 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f59 as WCHAR,
        to: 0x9f85 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f5c as WCHAR,
        to: 0x9f87 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f5f as WCHAR,
        to: 0x9f83 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f60 as WCHAR,
        to: 0x9f86 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f61 as WCHAR,
        to: 0x9f84 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f63 as WCHAR,
        to: 0x51fa as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f66 as WCHAR,
        to: 0x9f88 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f67 as WCHAR,
        to: 0x556e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f6a as WCHAR,
        to: 0x9f8a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f6c as WCHAR,
        to: 0x9f89 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f72 as WCHAR,
        to: 0x9f8b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f77 as WCHAR,
        to: 0x9f8c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f8d as WCHAR,
        to: 0x9f99 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f90 as WCHAR,
        to: 0x5e9e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f94 as WCHAR,
        to: 0x9f9a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f95 as WCHAR,
        to: 0x9f9b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0x9f9c as WCHAR,
        to: 0x9f9f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf901 as WCHAR,
        to: 0x66f4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf904 as WCHAR,
        to: 0x6ed1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf905 as WCHAR,
        to: 0x4e32 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf906 as WCHAR,
        to: 0x53e5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf909 as WCHAR,
        to: 0x5951 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf90a as WCHAR,
        to: 0x91d1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf90b as WCHAR,
        to: 0x5587 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf90c as WCHAR,
        to: 0x5948 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf911 as WCHAR,
        to: 0x87ba as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf912 as WCHAR,
        to: 0x88f8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf915 as WCHAR,
        to: 0x6d1b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf916 as WCHAR,
        to: 0x70d9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf917 as WCHAR,
        to: 0x73de as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf918 as WCHAR,
        to: 0x843d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf919 as WCHAR,
        to: 0x916a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf91c as WCHAR,
        to: 0x5375 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf925 as WCHAR,
        to: 0x62c9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf928 as WCHAR,
        to: 0x5eca as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf929 as WCHAR,
        to: 0x6717 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf92a as WCHAR,
        to: 0x6d6a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf92b as WCHAR,
        to: 0x72fc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf92e as WCHAR,
        to: 0x51b7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf934 as WCHAR,
        to: 0x8001 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf937 as WCHAR,
        to: 0x8def as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf938 as WCHAR,
        to: 0x9732 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf93b as WCHAR,
        to: 0x788c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf940 as WCHAR,
        to: 0x9e7f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf943 as WCHAR,
        to: 0x5f04 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf946 as WCHAR,
        to: 0x7262 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf947 as WCHAR,
        to: 0x78ca as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf949 as WCHAR,
        to: 0x96f7 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf94e as WCHAR,
        to: 0x6f0f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf94f as WCHAR,
        to: 0x7d2f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf951 as WCHAR,
        to: 0x964b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf952 as WCHAR,
        to: 0x52d2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf953 as WCHAR,
        to: 0x808b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf955 as WCHAR,
        to: 0x51cc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf958 as WCHAR,
        to: 0x83f1 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf959 as WCHAR,
        to: 0x9675 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf95e as WCHAR,
        to: 0x4e39 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf960 as WCHAR,
        to: 0x6012 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf961 as WCHAR,
        to: 0x7387 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf963 as WCHAR,
        to: 0x5317 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf965 as WCHAR,
        to: 0x4fbf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf967 as WCHAR,
        to: 0x4e0d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf968 as WCHAR,
        to: 0x6ccc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf96a as WCHAR,
        to: 0x7d22 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf96c as WCHAR,
        to: 0x585e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf96d as WCHAR,
        to: 0x7701 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf971 as WCHAR,
        to: 0x8fb0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf972 as WCHAR,
        to: 0x6c88 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf973 as WCHAR,
        to: 0x62fe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf974 as WCHAR,
        to: 0x82e5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf975 as WCHAR,
        to: 0x63a0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf976 as WCHAR,
        to: 0x7565 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf977 as WCHAR,
        to: 0x4eae as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf97a as WCHAR,
        to: 0x6881 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf97c as WCHAR,
        to: 0x826f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf97e as WCHAR,
        to: 0x91cf as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf981 as WCHAR,
        to: 0x5973 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf983 as WCHAR,
        to: 0x65c5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf989 as WCHAR,
        to: 0x9ece as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf98a as WCHAR,
        to: 0x529b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf98e as WCHAR,
        to: 0x5e74 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf99c as WCHAR,
        to: 0x5217 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf99d as WCHAR,
        to: 0x52a3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf99e as WCHAR,
        to: 0x54bd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf99f as WCHAR,
        to: 0x70c8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9a0 as WCHAR,
        to: 0x88c2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9a2 as WCHAR,
        to: 0x5ec9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9a3 as WCHAR,
        to: 0x5ff5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9a4 as WCHAR,
        to: 0x637b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9a8 as WCHAR,
        to: 0x4ee4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9a9 as WCHAR,
        to: 0x56f9 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9ac as WCHAR,
        to: 0x601c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9ad as WCHAR,
        to: 0x73b2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9af as WCHAR,
        to: 0x7f9a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9b0 as WCHAR,
        to: 0x8046 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9b2 as WCHAR,
        to: 0x96f6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9b5 as WCHAR,
        to: 0x4f8b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9b7 as WCHAR,
        to: 0x91b4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9ba as WCHAR,
        to: 0x4e86 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9bb as WCHAR,
        to: 0x50da as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9bc as WCHAR,
        to: 0x5bee as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9bd as WCHAR,
        to: 0x5c3f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9be as WCHAR,
        to: 0x6599 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9c0 as WCHAR,
        to: 0x71ce as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9c2 as WCHAR,
        to: 0x84fc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9c6 as WCHAR,
        to: 0x962e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9c9 as WCHAR,
        to: 0x67f3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9ca as WCHAR,
        to: 0x6d41 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9cb as WCHAR,
        to: 0x6e9c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9cc as WCHAR,
        to: 0x7409 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9cd as WCHAR,
        to: 0x7559 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9ce as WCHAR,
        to: 0x786b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9d1 as WCHAR,
        to: 0x516d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9d2 as WCHAR,
        to: 0x622e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9d8 as WCHAR,
        to: 0x5f8b as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9da as WCHAR,
        to: 0x6817 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9db as WCHAR,
        to: 0x7387 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9dc as WCHAR,
        to: 0x9686 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9dd as WCHAR,
        to: 0x5229 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9de as WCHAR,
        to: 0x540f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9df as WCHAR,
        to: 0x5c65 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9e0 as WCHAR,
        to: 0x6613 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9e1 as WCHAR,
        to: 0x674e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9e2 as WCHAR,
        to: 0x68a8 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9e3 as WCHAR,
        to: 0x6ce5 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9e4 as WCHAR,
        to: 0x7406 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9e5 as WCHAR,
        to: 0x75e2 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9e6 as WCHAR,
        to: 0x7f79 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9e9 as WCHAR,
        to: 0x91cc as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9eb as WCHAR,
        to: 0x533f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9ec as WCHAR,
        to: 0x6eba as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9ed as WCHAR,
        to: 0x541d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9f3 as WCHAR,
        to: 0x9e9f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9f4 as WCHAR,
        to: 0x6797 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9f5 as WCHAR,
        to: 0x6dcb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9f7 as WCHAR,
        to: 0x7acb as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9f8 as WCHAR,
        to: 0x7b20 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9f9 as WCHAR,
        to: 0x7c92 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9fb as WCHAR,
        to: 0x7099 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9fd as WCHAR,
        to: 0x4ec0 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9fe as WCHAR,
        to: 0x8336 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xf9ff as WCHAR,
        to: 0x523a as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xfa00 as WCHAR,
        to: 0x5207 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xfa01 as WCHAR,
        to: 0x5ea6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xfa02 as WCHAR,
        to: 0x62d3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xfa03 as WCHAR,
        to: 0x7cd6 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xfa04 as WCHAR,
        to: 0x5b85 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xfa05 as WCHAR,
        to: 0x6d1e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xfa06 as WCHAR,
        to: 0x66b4 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xfa08 as WCHAR,
        to: 0x884c as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xfa09 as WCHAR,
        to: 0x964d as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xfa0b as WCHAR,
        to: 0x5ed3 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xfa0c as WCHAR,
        to: 0x5140 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xfa12 as WCHAR,
        to: 0x6674 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xfa17 as WCHAR,
        to: 0x76ca as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xfa19 as WCHAR,
        to: 0x795e as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xfa1a as WCHAR,
        to: 0x7965 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xfa1b as WCHAR,
        to: 0x798f as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xfa1c as WCHAR,
        to: 0x9756 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xfa1d as WCHAR,
        to: 0x7cbe as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xfa1e as WCHAR,
        to: 0x7fbd as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xfa25 as WCHAR,
        to: 0x9038 as WCHAR,
    },
    tagWCHAR_PAIR {
        from: 0xfa26 as WCHAR,
        to: 0x90fd as WCHAR,
    },
];

static mut geoinfodata: [geoinfo_t; 299] = [
    geoinfo_t {
        id: 2 as GEOID,
        iso2W: [
            'A' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'A' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 28 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 3 as GEOID,
        iso2W: [
            'A' as i32 as WCHAR,
            'F' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'A' as i32 as WCHAR,
            'F' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47614 as GEOID,
        uncode: 4 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 4 as GEOID,
        iso2W: [
            'D' as i32 as WCHAR,
            'Z' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'D' as i32 as WCHAR,
            'Z' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42487 as GEOID,
        uncode: 12 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 5 as GEOID,
        iso2W: [
            'A' as i32 as WCHAR,
            'Z' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'A' as i32 as WCHAR,
            'Z' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47611 as GEOID,
        uncode: 31 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 6 as GEOID,
        iso2W: [
            'A' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'A' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47610 as GEOID,
        uncode: 8 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 7 as GEOID,
        iso2W: [
            'A' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'A' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47611 as GEOID,
        uncode: 51 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 8 as GEOID,
        iso2W: [
            'A' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'A' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47610 as GEOID,
        uncode: 20 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 9 as GEOID,
        iso2W: [
            'A' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'A' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42484 as GEOID,
        uncode: 24 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 10 as GEOID,
        iso2W: [
            'A' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'A' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 26286 as GEOID,
        uncode: 16 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 11 as GEOID,
        iso2W: [
            'A' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'A' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 31396 as GEOID,
        uncode: 32 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 12 as GEOID,
        iso2W: [
            'A' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'A' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10210825 as GEOID,
        uncode: 36 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 14 as GEOID,
        iso2W: [
            'A' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'A' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10210824 as GEOID,
        uncode: 40 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 17 as GEOID,
        iso2W: [
            'B' as i32 as WCHAR,
            'H' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'B' as i32 as WCHAR,
            'H' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47611 as GEOID,
        uncode: 48 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 18 as GEOID,
        iso2W: [
            'B' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'B' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 52 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 19 as GEOID,
        iso2W: [
            'B' as i32 as WCHAR,
            'W' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'B' as i32 as WCHAR,
            'W' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039883 as GEOID,
        uncode: 72 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 20 as GEOID,
        iso2W: [
            'B' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'B' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 23581 as GEOID,
        uncode: 60 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 21 as GEOID,
        iso2W: [
            'B' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'B' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10210824 as GEOID,
        uncode: 56 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 22 as GEOID,
        iso2W: [
            'B' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'B' as i32 as WCHAR,
            'H' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 44 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 23 as GEOID,
        iso2W: [
            'B' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'B' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47614 as GEOID,
        uncode: 50 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 24 as GEOID,
        iso2W: [
            'B' as i32 as WCHAR,
            'Z' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'B' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'Z' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 27082 as GEOID,
        uncode: 84 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 25 as GEOID,
        iso2W: [
            'B' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'B' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            'H' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47610 as GEOID,
        uncode: 70 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 26 as GEOID,
        iso2W: [
            'B' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'B' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 31396 as GEOID,
        uncode: 68 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 27 as GEOID,
        iso2W: [
            'M' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'M' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47599 as GEOID,
        uncode: 104 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 28 as GEOID,
        iso2W: [
            'B' as i32 as WCHAR,
            'J' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'B' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42483 as GEOID,
        uncode: 204 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 29 as GEOID,
        iso2W: [
            'B' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'B' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47609 as GEOID,
        uncode: 112 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 30 as GEOID,
        iso2W: [
            'S' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 20900 as GEOID,
        uncode: 90 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 32 as GEOID,
        iso2W: [
            'B' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'B' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 31396 as GEOID,
        uncode: 76 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 34 as GEOID,
        iso2W: [
            'B' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'B' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47614 as GEOID,
        uncode: 64 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 35 as GEOID,
        iso2W: [
            'B' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'B' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47609 as GEOID,
        uncode: 100 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 37 as GEOID,
        iso2W: [
            'B' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'B' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47599 as GEOID,
        uncode: 96 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 38 as GEOID,
        iso2W: [
            'B' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'B' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47603 as GEOID,
        uncode: 108 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 39 as GEOID,
        iso2W: [
            'C' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'C' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 23581 as GEOID,
        uncode: 124 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 40 as GEOID,
        iso2W: [
            'K' as i32 as WCHAR,
            'H' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'K' as i32 as WCHAR,
            'H' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47599 as GEOID,
        uncode: 116 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 41 as GEOID,
        iso2W: [
            'T' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'T' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42484 as GEOID,
        uncode: 148 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 42 as GEOID,
        iso2W: [
            'L' as i32 as WCHAR,
            'K' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'L' as i32 as WCHAR,
            'K' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47614 as GEOID,
        uncode: 144 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 43 as GEOID,
        iso2W: [
            'C' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'C' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42484 as GEOID,
        uncode: 178 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 44 as GEOID,
        iso2W: [
            'C' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'C' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42484 as GEOID,
        uncode: 180 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 45 as GEOID,
        iso2W: [
            'C' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'C' as i32 as WCHAR,
            'H' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47600 as GEOID,
        uncode: 156 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 46 as GEOID,
        iso2W: [
            'C' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'C' as i32 as WCHAR,
            'H' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 31396 as GEOID,
        uncode: 152 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 49 as GEOID,
        iso2W: [
            'C' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'C' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42484 as GEOID,
        uncode: 120 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 50 as GEOID,
        iso2W: [
            'K' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'C' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47603 as GEOID,
        uncode: 174 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 51 as GEOID,
        iso2W: [
            'C' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'C' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 31396 as GEOID,
        uncode: 170 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 54 as GEOID,
        iso2W: [
            'C' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'C' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 27082 as GEOID,
        uncode: 188 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 55 as GEOID,
        iso2W: [
            'C' as i32 as WCHAR,
            'F' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'C' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            'F' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42484 as GEOID,
        uncode: 140 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 56 as GEOID,
        iso2W: [
            'C' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'C' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 192 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 57 as GEOID,
        iso2W: [
            'C' as i32 as WCHAR,
            'V' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'C' as i32 as WCHAR,
            'P' as i32 as WCHAR,
            'V' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42483 as GEOID,
        uncode: 132 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 59 as GEOID,
        iso2W: [
            'C' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'C' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            'P' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47611 as GEOID,
        uncode: 196 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 61 as GEOID,
        iso2W: [
            'D' as i32 as WCHAR,
            'K' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'D' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            'K' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039882 as GEOID,
        uncode: 208 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 62 as GEOID,
        iso2W: [
            'D' as i32 as WCHAR,
            'J' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'D' as i32 as WCHAR,
            'J' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47603 as GEOID,
        uncode: 262 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 63 as GEOID,
        iso2W: [
            'D' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'D' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 212 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 65 as GEOID,
        iso2W: [
            'D' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'D' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 214 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 66 as GEOID,
        iso2W: [
            'E' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'E' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 31396 as GEOID,
        uncode: 218 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 67 as GEOID,
        iso2W: [
            'E' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'E' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42487 as GEOID,
        uncode: 818 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 68 as GEOID,
        iso2W: [
            'I' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'I' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039882 as GEOID,
        uncode: 372 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 69 as GEOID,
        iso2W: [
            'G' as i32 as WCHAR,
            'Q' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'G' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            'Q' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42484 as GEOID,
        uncode: 226 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 70 as GEOID,
        iso2W: [
            'E' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'E' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039882 as GEOID,
        uncode: 233 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 71 as GEOID,
        iso2W: [
            'E' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'E' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47603 as GEOID,
        uncode: 232 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 72 as GEOID,
        iso2W: [
            'S' as i32 as WCHAR,
            'V' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'V' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 27082 as GEOID,
        uncode: 222 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 73 as GEOID,
        iso2W: [
            'E' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'E' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            'H' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47603 as GEOID,
        uncode: 231 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 75 as GEOID,
        iso2W: [
            'C' as i32 as WCHAR,
            'Z' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'C' as i32 as WCHAR,
            'Z' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47609 as GEOID,
        uncode: 203 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 77 as GEOID,
        iso2W: [
            'F' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'F' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039882 as GEOID,
        uncode: 246 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 78 as GEOID,
        iso2W: [
            'F' as i32 as WCHAR,
            'J' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'F' as i32 as WCHAR,
            'J' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 20900 as GEOID,
        uncode: 242 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 80 as GEOID,
        iso2W: [
            'F' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'F' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 21206 as GEOID,
        uncode: 583 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 81 as GEOID,
        iso2W: [
            'F' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'F' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039882 as GEOID,
        uncode: 234 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 84 as GEOID,
        iso2W: [
            'F' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'F' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10210824 as GEOID,
        uncode: 250 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 86 as GEOID,
        iso2W: [
            'G' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'G' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42483 as GEOID,
        uncode: 270 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 87 as GEOID,
        iso2W: [
            'G' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'G' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42484 as GEOID,
        uncode: 266 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 88 as GEOID,
        iso2W: [
            'G' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'G' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47611 as GEOID,
        uncode: 268 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 89 as GEOID,
        iso2W: [
            'G' as i32 as WCHAR,
            'H' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'G' as i32 as WCHAR,
            'H' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42483 as GEOID,
        uncode: 288 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 90 as GEOID,
        iso2W: [
            'G' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'G' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47610 as GEOID,
        uncode: 292 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 91 as GEOID,
        iso2W: [
            'G' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'G' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 308 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 93 as GEOID,
        iso2W: [
            'G' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'G' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 23581 as GEOID,
        uncode: 304 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 94 as GEOID,
        iso2W: [
            'D' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'D' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10210824 as GEOID,
        uncode: 276 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 98 as GEOID,
        iso2W: [
            'G' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'G' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47610 as GEOID,
        uncode: 300 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 99 as GEOID,
        iso2W: [
            'G' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'G' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 27082 as GEOID,
        uncode: 320 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 100 as GEOID,
        iso2W: [
            'G' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'G' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42483 as GEOID,
        uncode: 324 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 101 as GEOID,
        iso2W: [
            'G' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'G' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 31396 as GEOID,
        uncode: 328 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 103 as GEOID,
        iso2W: [
            'H' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'H' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 332 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 104 as GEOID,
        iso2W: [
            'H' as i32 as WCHAR,
            'K' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'H' as i32 as WCHAR,
            'K' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47600 as GEOID,
        uncode: 344 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 106 as GEOID,
        iso2W: [
            'H' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'H' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 27082 as GEOID,
        uncode: 340 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 108 as GEOID,
        iso2W: [
            'H' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'H' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'V' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47610 as GEOID,
        uncode: 191 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 109 as GEOID,
        iso2W: [
            'H' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'H' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47609 as GEOID,
        uncode: 348 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 110 as GEOID,
        iso2W: [
            'I' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'I' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039882 as GEOID,
        uncode: 352 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 111 as GEOID,
        iso2W: [
            'I' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'I' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47599 as GEOID,
        uncode: 360 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 113 as GEOID,
        iso2W: [
            'I' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'I' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47614 as GEOID,
        uncode: 356 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 114 as GEOID,
        iso2W: [
            'I' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'I' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 39070 as GEOID,
        uncode: 86 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 116 as GEOID,
        iso2W: [
            'I' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'I' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47614 as GEOID,
        uncode: 364 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 117 as GEOID,
        iso2W: [
            'I' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'I' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47611 as GEOID,
        uncode: 376 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 118 as GEOID,
        iso2W: [
            'I' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'I' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47610 as GEOID,
        uncode: 380 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 119 as GEOID,
        iso2W: [
            'C' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'C' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            'V' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42483 as GEOID,
        uncode: 384 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 121 as GEOID,
        iso2W: [
            'I' as i32 as WCHAR,
            'Q' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'I' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'Q' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47611 as GEOID,
        uncode: 368 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 122 as GEOID,
        iso2W: [
            'J' as i32 as WCHAR,
            'P' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'J' as i32 as WCHAR,
            'P' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47600 as GEOID,
        uncode: 392 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 124 as GEOID,
        iso2W: [
            'J' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'J' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 388 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 125 as GEOID,
        iso2W: [
            'S' as i32 as WCHAR,
            'J' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'J' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039882 as GEOID,
        uncode: 744 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 126 as GEOID,
        iso2W: [
            'J' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'J' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47611 as GEOID,
        uncode: 400 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 127 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 161832256 as GEOID,
        uncode: 0,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 129 as GEOID,
        iso2W: [
            'K' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'K' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47603 as GEOID,
        uncode: 404 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 130 as GEOID,
        iso2W: [
            'K' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'K' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            'Z' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47590 as GEOID,
        uncode: 417 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 131 as GEOID,
        iso2W: [
            'K' as i32 as WCHAR,
            'P' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'P' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'K' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47600 as GEOID,
        uncode: 408 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 133 as GEOID,
        iso2W: [
            'K' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'K' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 21206 as GEOID,
        uncode: 296 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 134 as GEOID,
        iso2W: [
            'K' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'K' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47600 as GEOID,
        uncode: 410 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 136 as GEOID,
        iso2W: [
            'K' as i32 as WCHAR,
            'W' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'K' as i32 as WCHAR,
            'W' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47611 as GEOID,
        uncode: 414 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 137 as GEOID,
        iso2W: [
            'K' as i32 as WCHAR,
            'Z' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'K' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            'Z' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47590 as GEOID,
        uncode: 398 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 138 as GEOID,
        iso2W: [
            'L' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'L' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47599 as GEOID,
        uncode: 418 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 139 as GEOID,
        iso2W: [
            'L' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'L' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47611 as GEOID,
        uncode: 422 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 140 as GEOID,
        iso2W: [
            'L' as i32 as WCHAR,
            'V' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'L' as i32 as WCHAR,
            'V' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039882 as GEOID,
        uncode: 428 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 141 as GEOID,
        iso2W: [
            'L' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'L' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039882 as GEOID,
        uncode: 440 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 142 as GEOID,
        iso2W: [
            'L' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'L' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42483 as GEOID,
        uncode: 430 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 143 as GEOID,
        iso2W: [
            'S' as i32 as WCHAR,
            'K' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'V' as i32 as WCHAR,
            'K' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47609 as GEOID,
        uncode: 703 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 145 as GEOID,
        iso2W: [
            'L' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'L' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10210824 as GEOID,
        uncode: 438 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 146 as GEOID,
        iso2W: [
            'L' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'L' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039883 as GEOID,
        uncode: 426 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 147 as GEOID,
        iso2W: [
            'L' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'L' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10210824 as GEOID,
        uncode: 442 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 148 as GEOID,
        iso2W: [
            'L' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'L' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42487 as GEOID,
        uncode: 434 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 149 as GEOID,
        iso2W: [
            'M' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'M' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47603 as GEOID,
        uncode: 450 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 151 as GEOID,
        iso2W: [
            'M' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'M' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47600 as GEOID,
        uncode: 446 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 152 as GEOID,
        iso2W: [
            'M' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'M' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47609 as GEOID,
        uncode: 498 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 154 as GEOID,
        iso2W: [
            'M' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'M' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47600 as GEOID,
        uncode: 496 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 156 as GEOID,
        iso2W: [
            'M' as i32 as WCHAR,
            'W' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'M' as i32 as WCHAR,
            'W' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47603 as GEOID,
        uncode: 454 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 157 as GEOID,
        iso2W: [
            'M' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'M' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42483 as GEOID,
        uncode: 466 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 158 as GEOID,
        iso2W: [
            'M' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'M' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10210824 as GEOID,
        uncode: 492 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 159 as GEOID,
        iso2W: [
            'M' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'M' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42487 as GEOID,
        uncode: 504 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 160 as GEOID,
        iso2W: [
            'M' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'M' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47603 as GEOID,
        uncode: 480 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 162 as GEOID,
        iso2W: [
            'M' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'M' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42483 as GEOID,
        uncode: 478 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 163 as GEOID,
        iso2W: [
            'M' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'M' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47610 as GEOID,
        uncode: 470 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 164 as GEOID,
        iso2W: [
            'O' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'O' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47611 as GEOID,
        uncode: 512 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 165 as GEOID,
        iso2W: [
            'M' as i32 as WCHAR,
            'V' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'M' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            'V' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47614 as GEOID,
        uncode: 462 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 166 as GEOID,
        iso2W: [
            'M' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'M' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 27082 as GEOID,
        uncode: 484 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 167 as GEOID,
        iso2W: [
            'M' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'M' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47599 as GEOID,
        uncode: 458 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 168 as GEOID,
        iso2W: [
            'M' as i32 as WCHAR,
            'Z' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'M' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'Z' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47603 as GEOID,
        uncode: 508 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 173 as GEOID,
        iso2W: [
            'N' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'N' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42483 as GEOID,
        uncode: 562 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 174 as GEOID,
        iso2W: [
            'V' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'V' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 20900 as GEOID,
        uncode: 548 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 175 as GEOID,
        iso2W: [
            'N' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'N' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42483 as GEOID,
        uncode: 566 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 176 as GEOID,
        iso2W: [
            'N' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'N' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10210824 as GEOID,
        uncode: 528 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 177 as GEOID,
        iso2W: [
            'N' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'N' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039882 as GEOID,
        uncode: 578 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 178 as GEOID,
        iso2W: [
            'N' as i32 as WCHAR,
            'P' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'N' as i32 as WCHAR,
            'P' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47614 as GEOID,
        uncode: 524 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 180 as GEOID,
        iso2W: [
            'N' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'N' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 21206 as GEOID,
        uncode: 520 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 181 as GEOID,
        iso2W: [
            'S' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 31396 as GEOID,
        uncode: 740 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 182 as GEOID,
        iso2W: [
            'N' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'N' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 27082 as GEOID,
        uncode: 558 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 183 as GEOID,
        iso2W: [
            'N' as i32 as WCHAR,
            'Z' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'N' as i32 as WCHAR,
            'Z' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10210825 as GEOID,
        uncode: 554 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 184 as GEOID,
        iso2W: [
            'P' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'P' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47611 as GEOID,
        uncode: 275 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 185 as GEOID,
        iso2W: [
            'P' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'P' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 31396 as GEOID,
        uncode: 600 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 187 as GEOID,
        iso2W: [
            'P' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'P' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 31396 as GEOID,
        uncode: 604 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 190 as GEOID,
        iso2W: [
            'P' as i32 as WCHAR,
            'K' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'P' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            'K' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47614 as GEOID,
        uncode: 586 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 191 as GEOID,
        iso2W: [
            'P' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'P' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47609 as GEOID,
        uncode: 616 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 192 as GEOID,
        iso2W: [
            'P' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'P' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 27082 as GEOID,
        uncode: 591 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 193 as GEOID,
        iso2W: [
            'P' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'P' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47610 as GEOID,
        uncode: 620 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 194 as GEOID,
        iso2W: [
            'P' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'P' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 20900 as GEOID,
        uncode: 598 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 195 as GEOID,
        iso2W: [
            'P' as i32 as WCHAR,
            'W' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'P' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'W' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 21206 as GEOID,
        uncode: 585 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 196 as GEOID,
        iso2W: [
            'G' as i32 as WCHAR,
            'W' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'G' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42483 as GEOID,
        uncode: 624 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 197 as GEOID,
        iso2W: [
            'Q' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'Q' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47611 as GEOID,
        uncode: 634 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 198 as GEOID,
        iso2W: [
            'R' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'R' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47603 as GEOID,
        uncode: 638 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 199 as GEOID,
        iso2W: [
            'M' as i32 as WCHAR,
            'H' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'M' as i32 as WCHAR,
            'H' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 21206 as GEOID,
        uncode: 584 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 200 as GEOID,
        iso2W: [
            'R' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'R' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47609 as GEOID,
        uncode: 642 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 201 as GEOID,
        iso2W: [
            'P' as i32 as WCHAR,
            'H' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'P' as i32 as WCHAR,
            'H' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47599 as GEOID,
        uncode: 608 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 202 as GEOID,
        iso2W: [
            'P' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'P' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 630 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 203 as GEOID,
        iso2W: [
            'R' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'R' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47609 as GEOID,
        uncode: 643 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 204 as GEOID,
        iso2W: [
            'R' as i32 as WCHAR,
            'W' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'R' as i32 as WCHAR,
            'W' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47603 as GEOID,
        uncode: 646 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 205 as GEOID,
        iso2W: [
            'S' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47611 as GEOID,
        uncode: 682 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 206 as GEOID,
        iso2W: [
            'P' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'P' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 23581 as GEOID,
        uncode: 666 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 207 as GEOID,
        iso2W: [
            'K' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'K' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 659 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 208 as GEOID,
        iso2W: [
            'S' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47603 as GEOID,
        uncode: 690 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 209 as GEOID,
        iso2W: [
            'Z' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'Z' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            'F' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039883 as GEOID,
        uncode: 710 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 210 as GEOID,
        iso2W: [
            'S' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42483 as GEOID,
        uncode: 686 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 212 as GEOID,
        iso2W: [
            'S' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'V' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47610 as GEOID,
        uncode: 705 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 213 as GEOID,
        iso2W: [
            'S' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42483 as GEOID,
        uncode: 694 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 214 as GEOID,
        iso2W: [
            'S' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47610 as GEOID,
        uncode: 674 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 215 as GEOID,
        iso2W: [
            'S' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            'P' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47599 as GEOID,
        uncode: 702 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 216 as GEOID,
        iso2W: [
            'S' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47603 as GEOID,
        uncode: 706 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 217 as GEOID,
        iso2W: [
            'E' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'E' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'P' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47610 as GEOID,
        uncode: 724 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 218 as GEOID,
        iso2W: [
            'L' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'L' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 662 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 219 as GEOID,
        iso2W: [
            'S' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42487 as GEOID,
        uncode: 736 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 220 as GEOID,
        iso2W: [
            'S' as i32 as WCHAR,
            'J' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'J' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039882 as GEOID,
        uncode: 744 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 221 as GEOID,
        iso2W: [
            'S' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'W' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039882 as GEOID,
        uncode: 752 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 222 as GEOID,
        iso2W: [
            'S' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47611 as GEOID,
        uncode: 760 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 223 as GEOID,
        iso2W: [
            'C' as i32 as WCHAR,
            'H' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'C' as i32 as WCHAR,
            'H' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10210824 as GEOID,
        uncode: 756 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 224 as GEOID,
        iso2W: [
            'A' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'A' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47611 as GEOID,
        uncode: 784 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 225 as GEOID,
        iso2W: [
            'T' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'T' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 780 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 227 as GEOID,
        iso2W: [
            'T' as i32 as WCHAR,
            'H' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'T' as i32 as WCHAR,
            'H' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47599 as GEOID,
        uncode: 764 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 228 as GEOID,
        iso2W: [
            'T' as i32 as WCHAR,
            'J' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'T' as i32 as WCHAR,
            'J' as i32 as WCHAR,
            'K' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47590 as GEOID,
        uncode: 762 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 231 as GEOID,
        iso2W: [
            'T' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'T' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 26286 as GEOID,
        uncode: 776 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 232 as GEOID,
        iso2W: [
            'T' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'T' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42483 as GEOID,
        uncode: 768 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 233 as GEOID,
        iso2W: [
            'S' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            'P' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42484 as GEOID,
        uncode: 678 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 234 as GEOID,
        iso2W: [
            'T' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'T' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42487 as GEOID,
        uncode: 788 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 235 as GEOID,
        iso2W: [
            'T' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'T' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47611 as GEOID,
        uncode: 792 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 236 as GEOID,
        iso2W: [
            'T' as i32 as WCHAR,
            'V' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'T' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            'V' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 26286 as GEOID,
        uncode: 798 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 237 as GEOID,
        iso2W: [
            'T' as i32 as WCHAR,
            'W' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'T' as i32 as WCHAR,
            'W' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47600 as GEOID,
        uncode: 158 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 238 as GEOID,
        iso2W: [
            'T' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'T' as i32 as WCHAR,
            'K' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47590 as GEOID,
        uncode: 795 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 239 as GEOID,
        iso2W: [
            'T' as i32 as WCHAR,
            'Z' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'T' as i32 as WCHAR,
            'Z' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47603 as GEOID,
        uncode: 834 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 240 as GEOID,
        iso2W: [
            'U' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'U' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47603 as GEOID,
        uncode: 800 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 241 as GEOID,
        iso2W: [
            'U' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'U' as i32 as WCHAR,
            'K' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47609 as GEOID,
        uncode: 804 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 242 as GEOID,
        iso2W: [
            'G' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'G' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039882 as GEOID,
        uncode: 826 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 244 as GEOID,
        iso2W: [
            'U' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'U' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 23581 as GEOID,
        uncode: 840 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 245 as GEOID,
        iso2W: [
            'B' as i32 as WCHAR,
            'F' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'B' as i32 as WCHAR,
            'F' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42483 as GEOID,
        uncode: 854 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 246 as GEOID,
        iso2W: [
            'U' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'U' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 31396 as GEOID,
        uncode: 858 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 247 as GEOID,
        iso2W: [
            'U' as i32 as WCHAR,
            'Z' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'U' as i32 as WCHAR,
            'Z' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47590 as GEOID,
        uncode: 860 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 248 as GEOID,
        iso2W: [
            'V' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'V' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 670 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 249 as GEOID,
        iso2W: [
            'V' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'V' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 31396 as GEOID,
        uncode: 862 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 251 as GEOID,
        iso2W: [
            'V' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'V' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47599 as GEOID,
        uncode: 704 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 252 as GEOID,
        iso2W: [
            'V' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'V' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 850 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 253 as GEOID,
        iso2W: [
            'V' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'V' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47610 as GEOID,
        uncode: 336 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 254 as GEOID,
        iso2W: [
            'N' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'N' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039883 as GEOID,
        uncode: 516 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 257 as GEOID,
        iso2W: [
            'E' as i32 as WCHAR,
            'H' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'E' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'H' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42487 as GEOID,
        uncode: 732 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 258 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 161832256 as GEOID,
        uncode: 0,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 259 as GEOID,
        iso2W: [
            'W' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'W' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 26286 as GEOID,
        uncode: 882 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 260 as GEOID,
        iso2W: [
            'S' as i32 as WCHAR,
            'Z' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'W' as i32 as WCHAR,
            'Z' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039883 as GEOID,
        uncode: 748 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 261 as GEOID,
        iso2W: [
            'Y' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'Y' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47611 as GEOID,
        uncode: 887 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 263 as GEOID,
        iso2W: [
            'Z' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'Z' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47603 as GEOID,
        uncode: 894 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 264 as GEOID,
        iso2W: [
            'Z' as i32 as WCHAR,
            'W' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'Z' as i32 as WCHAR,
            'W' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47603 as GEOID,
        uncode: 716 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 269 as GEOID,
        iso2W: [
            'C' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47610 as GEOID,
        uncode: 891 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 270 as GEOID,
        iso2W: [
            'M' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'M' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47610 as GEOID,
        uncode: 499 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 271 as GEOID,
        iso2W: [
            'R' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47610 as GEOID,
        uncode: 688 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 273 as GEOID,
        iso2W: [
            'C' as i32 as WCHAR,
            'W' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'C' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            'W' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 531 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 276 as GEOID,
        iso2W: [
            'S' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42487 as GEOID,
        uncode: 728 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 300 as GEOID,
        iso2W: [
            'A' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'A' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 660 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 301 as GEOID,
        iso2W: [
            'A' as i32 as WCHAR,
            'Q' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'A' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 39070 as GEOID,
        uncode: 10 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 302 as GEOID,
        iso2W: [
            'A' as i32 as WCHAR,
            'W' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'A' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            'W' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 533 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 303 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 39070 as GEOID,
        uncode: 0,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 304 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 10210825 as GEOID,
        uncode: 0,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 305 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 161832256 as GEOID,
        uncode: 0,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 306 as GEOID,
        iso2W: [
            'B' as i32 as WCHAR,
            'V' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'B' as i32 as WCHAR,
            'V' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 39070 as GEOID,
        uncode: 74 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 307 as GEOID,
        iso2W: [
            'K' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'C' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 136 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 308 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 10210824 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_BOTH,
    },
    geoinfo_t {
        id: 309 as GEOID,
        iso2W: [
            'C' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'C' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 12 as GEOID,
        uncode: 162 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 310 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 27114 as GEOID,
        uncode: 0,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 311 as GEOID,
        iso2W: [
            'C' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'C' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            'K' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10210825 as GEOID,
        uncode: 166 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 312 as GEOID,
        iso2W: [
            'C' as i32 as WCHAR,
            'K' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'C' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'K' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 26286 as GEOID,
        uncode: 184 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 313 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 10210825 as GEOID,
        uncode: 0,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 314 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 114 as GEOID,
        uncode: 0,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 315 as GEOID,
        iso2W: [
            'F' as i32 as WCHAR,
            'K' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'F' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'K' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 31396 as GEOID,
        uncode: 238 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 317 as GEOID,
        iso2W: [
            'G' as i32 as WCHAR,
            'F' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'G' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            'F' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 31396 as GEOID,
        uncode: 254 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 318 as GEOID,
        iso2W: [
            'P' as i32 as WCHAR,
            'F' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'P' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            'F' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 26286 as GEOID,
        uncode: 258 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 319 as GEOID,
        iso2W: [
            'T' as i32 as WCHAR,
            'F' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'A' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            'F' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 39070 as GEOID,
        uncode: 260 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 321 as GEOID,
        iso2W: [
            'G' as i32 as WCHAR,
            'P' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'G' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'P' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 312 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 322 as GEOID,
        iso2W: [
            'G' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'G' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 21206 as GEOID,
        uncode: 316 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 323 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 39070 as GEOID,
        uncode: 0,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 324 as GEOID,
        iso2W: [
            'G' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'G' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 308 as GEOID,
        uncode: 831 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 325 as GEOID,
        iso2W: [
            'H' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'H' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 39070 as GEOID,
        uncode: 334 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 326 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 161832256 as GEOID,
        uncode: 0,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 327 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 161832256 as GEOID,
        uncode: 0,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 328 as GEOID,
        iso2W: [
            'J' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'J' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 308 as GEOID,
        uncode: 832 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 329 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 161832256 as GEOID,
        uncode: 0,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 330 as GEOID,
        iso2W: [
            'M' as i32 as WCHAR,
            'Q' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'M' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            'Q' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 474 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 331 as GEOID,
        iso2W: [
            'Y' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'M' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47603 as GEOID,
        uncode: 175 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 332 as GEOID,
        iso2W: [
            'M' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'M' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 500 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 333 as GEOID,
        iso2W: [
            'A' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'A' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 530 as INT,
        kind: LOCATION_BOTH,
    },
    geoinfo_t {
        id: 334 as GEOID,
        iso2W: [
            'N' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'N' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 20900 as GEOID,
        uncode: 540 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 335 as GEOID,
        iso2W: [
            'N' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'N' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 26286 as GEOID,
        uncode: 570 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 336 as GEOID,
        iso2W: [
            'N' as i32 as WCHAR,
            'F' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'N' as i32 as WCHAR,
            'F' as i32 as WCHAR,
            'K' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10210825 as GEOID,
        uncode: 574 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 337 as GEOID,
        iso2W: [
            'M' as i32 as WCHAR,
            'P' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'M' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            'P' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 21206 as GEOID,
        uncode: 580 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 338 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 161832256 as GEOID,
        uncode: 0,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 339 as GEOID,
        iso2W: [
            'P' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'P' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 26286 as GEOID,
        uncode: 612 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 340 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 337 as GEOID,
        uncode: 0,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 341 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 337 as GEOID,
        uncode: 0,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 342 as GEOID,
        iso2W: [
            'G' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 39070 as GEOID,
        uncode: 239 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 343 as GEOID,
        iso2W: [
            'S' as i32 as WCHAR,
            'H' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'H' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 42483 as GEOID,
        uncode: 654 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 346 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 337 as GEOID,
        uncode: 0,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 347 as GEOID,
        iso2W: [
            'T' as i32 as WCHAR,
            'K' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'T' as i32 as WCHAR,
            'K' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 26286 as GEOID,
        uncode: 772 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 348 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 39070 as GEOID,
        uncode: 0,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 349 as GEOID,
        iso2W: [
            'T' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'T' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 796 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 351 as GEOID,
        iso2W: [
            'V' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'V' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 92 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 352 as GEOID,
        iso2W: [
            'W' as i32 as WCHAR,
            'F' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'W' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'F' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 26286 as GEOID,
        uncode: 876 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 742 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 39070 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 2129 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 39070 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 10541 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 39070 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 15126 as GEOID,
        iso2W: [
            'I' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'I' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039882 as GEOID,
        uncode: 833 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 19618 as GEOID,
        iso2W: [
            'M' as i32 as WCHAR,
            'K' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'M' as i32 as WCHAR,
            'K' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47610 as GEOID,
        uncode: 807 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 20900 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 27114 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 21206 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 27114 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 21242 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 161832256 as GEOID,
        uncode: 0,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 23581 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 10026358 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 26286 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 27114 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 27082 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 161832257 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 27114 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 39070 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 30967 as GEOID,
        iso2W: [
            'S' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'S' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 534 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 31396 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 161832257 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 31706 as GEOID,
        iso2W: [
            'M' as i32 as WCHAR,
            'F' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'M' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            'F' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 663 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 39070 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 39070 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 42483 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 742 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 42484 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 742 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 42487 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 742 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 47590 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 2129 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 47599 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 2129 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 47600 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 2129 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 47603 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 742 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 47609 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 10541 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 47610 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 10541 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 47611 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 2129 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 47614 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 2129 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 7299303 as GEOID,
        iso2W: [
            'T' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'T' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 47599 as GEOID,
        uncode: 626 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 10026358 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 39070 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 10028789 as GEOID,
        iso2W: [
            'A' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'A' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039882 as GEOID,
        uncode: 248 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 10039880 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 161832257 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 10039882 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 10541 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 10039883 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 742 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 10210824 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 10541 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 10210825 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 27114 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
    geoinfo_t {
        id: 161832015 as GEOID,
        iso2W: [
            'B' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'B' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 10039880 as GEOID,
        uncode: 652 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 161832256 as GEOID,
        iso2W: [
            'U' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'U' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        parent: 27114 as GEOID,
        uncode: 581 as INT,
        kind: LOCATION_NATION,
    },
    geoinfo_t {
        id: 161832257 as GEOID,
        iso2W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
        ],
        iso3W: [
            'X' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
        ],
        parent: 10026358 as GEOID,
        uncode: 0 as INT,
        kind: LOCATION_REGION,
    },
];

unsafe fn get_lcid_codepage(mut lcid: LCID) -> UINT {
    let mut ret: UINT = 0;
    if GetLocaleInfoW(
        lcid,
        LOCALE_IDEFAULTANSICODEPAGE | LOCALE_RETURN_NUMBER,
        core::ptr::addr_of_mut!(ret) as LPWSTR,
        (core::mem::size_of::<UINT>() / core::mem::size_of::<WCHAR>()) as INT,
    ) == 0
    {
        ret = 0;
    }
    ret
}

unsafe fn get_default_sublang(mut lang: LANGID) -> LANGID {
    match lang as i32 {
        10 => {
            return ((SUBLANG_SPANISH_MODERN as u16 as i32) << 10
                | LANG_SPANISH as u16 as i32) as LANGID;
        }
        4 => {
            return ((SUBLANG_CHINESE_SIMPLIFIED as u16 as i32) << 10
                | LANG_CHINESE as u16 as i32) as LANGID;
        }
        4100 => {
            return ((SUBLANG_CHINESE_SIMPLIFIED as u16 as i32) << 10
                | LANG_CHINESE as u16 as i32) as LANGID;
        }
        1028 | 5124 => {
            return ((SUBLANG_CHINESE_HONGKONG as u16 as i32) << 10
                | LANG_CHINESE as u16 as i32) as LANGID;
        }
        _ => {}
    }
    if (lang as i32) >> 10 == SUBLANG_NEUTRAL as i32 {
        lang = ((SUBLANG_DEFAULT as u16 as i32) << 10
            | ((lang as i32) & 0x3ff) as u16 as i32) as LANGID;
    }
    return lang;
}

unsafe fn convert_default_lcid(mut lcid: LCID, lctype: LCTYPE) -> LCID {
    if lcid == LOCALE_SYSTEM_DEFAULT || lcid == LOCALE_USER_DEFAULT || lcid == LOCALE_NEUTRAL {
        let mut default_id: LCID = 0;
        
        match lctype & 0xffff {
            LOCALE_SSORTNAME => {
                default_id = lcid_LC_COLLATE;
            }
            
            LOCALE_FONTSIGNATURE |
            LOCALE_IDEFAULTANSICODEPAGE |
            LOCALE_IDEFAULTCODEPAGE |
            LOCALE_IDEFAULTEBCDICCODEPAGE |
            LOCALE_IDEFAULTMACCODEPAGE |
            LOCALE_IDEFAULTUNIXCODEPAGE => {
                default_id = lcid_LC_CTYPE;
            }
            
            LOCALE_ICURRDIGITS |
            LOCALE_ICURRENCY |
            LOCALE_IINTLCURRDIGITS |
            LOCALE_INEGCURR |
            LOCALE_INEGSEPBYSPACE |
            LOCALE_INEGSIGNPOSN |
            LOCALE_INEGSYMPRECEDES |
            LOCALE_IPOSSEPBYSPACE |
            LOCALE_IPOSSIGNPOSN |
            LOCALE_IPOSSYMPRECEDES |
            LOCALE_SCURRENCY |
            LOCALE_SINTLSYMBOL |
            LOCALE_SMONDECIMALSEP |
            LOCALE_SMONGROUPING |
            LOCALE_SMONTHOUSANDSEP |
            LOCALE_SNATIVECURRNAME => {
                default_id = lcid_LC_MONETARY;
            }
            
            LOCALE_IDIGITS |
            LOCALE_IDIGITSUBSTITUTION |
            LOCALE_ILZERO |
            LOCALE_INEGNUMBER |
            LOCALE_SDECIMAL |
            LOCALE_SGROUPING |
            LOCALE_SNATIVEDIGITS |
            LOCALE_SNEGATIVESIGN |
            LOCALE_SPOSITIVESIGN |
            LOCALE_STHOUSAND => {
                default_id = lcid_LC_NUMERIC;
            }
            
            LOCALE_ICALENDARTYPE |
            LOCALE_ICENTURY |
            LOCALE_IDATE |
            LOCALE_IDAYLZERO |
            LOCALE_IFIRSTDAYOFWEEK |
            LOCALE_IFIRSTWEEKOFYEAR |
            LOCALE_ILDATE |
            LOCALE_IMONLZERO |
            LOCALE_IOPTIONALCALENDAR |
            LOCALE_ITIME |
            LOCALE_ITIMEMARKPOSN |
            LOCALE_ITLZERO |
            LOCALE_S1159 |
            LOCALE_S2359 |
            LOCALE_SABBREVDAYNAME1 |
            LOCALE_SABBREVDAYNAME2 |
            LOCALE_SABBREVDAYNAME3 |
            LOCALE_SABBREVDAYNAME4 |
            LOCALE_SABBREVDAYNAME5 |
            LOCALE_SABBREVDAYNAME6 |
            LOCALE_SABBREVDAYNAME7 |
            LOCALE_SABBREVMONTHNAME1 |
            LOCALE_SABBREVMONTHNAME2 |
            LOCALE_SABBREVMONTHNAME3 |
            LOCALE_SABBREVMONTHNAME4 |
            LOCALE_SABBREVMONTHNAME5 |
            LOCALE_SABBREVMONTHNAME6 |
            LOCALE_SABBREVMONTHNAME7 |
            LOCALE_SABBREVMONTHNAME8 |
            LOCALE_SABBREVMONTHNAME9 |
            LOCALE_SABBREVMONTHNAME10 |
            LOCALE_SABBREVMONTHNAME11 |
            LOCALE_SABBREVMONTHNAME12 |
            LOCALE_SABBREVMONTHNAME13 |
            LOCALE_SDATE |
            LOCALE_SDAYNAME1 |
            LOCALE_SDAYNAME2 |
            LOCALE_SDAYNAME3 |
            LOCALE_SDAYNAME4 |
            LOCALE_SDAYNAME5 |
            LOCALE_SDAYNAME6 |
            LOCALE_SDAYNAME7 |
            LOCALE_SLONGDATE |
            LOCALE_SMONTHNAME1 |
            LOCALE_SMONTHNAME2 |
            LOCALE_SMONTHNAME3 |
            LOCALE_SMONTHNAME4 |
            LOCALE_SMONTHNAME5 |
            LOCALE_SMONTHNAME6 |
            LOCALE_SMONTHNAME7 |
            LOCALE_SMONTHNAME8 |
            LOCALE_SMONTHNAME9 |
            LOCALE_SMONTHNAME10 |
            LOCALE_SMONTHNAME11 |
            LOCALE_SMONTHNAME12 |
            LOCALE_SMONTHNAME13 |
            LOCALE_SSHORTDATE |
            LOCALE_STIME |
            LOCALE_STIMEFORMAT |
            LOCALE_SYEARMONTH => {
                default_id = lcid_LC_TIME;
            }
            
            LOCALE_IPAPERSIZE => {
                default_id = lcid_LC_PAPER;
            }
            
            LOCALE_IMEASURE => {
                default_id = lcid_LC_MEASUREMENT;
            }
            
            LOCALE_ICOUNTRY => {
                default_id = lcid_LC_TELEPHONE;
            }
            
            _ => {}
        }
        
        if default_id != 0 {
            lcid = default_id;
        }
    }
    
    ConvertDefaultLocale(lcid)
}

unsafe fn is_genitive_name_supported(lctype: LCTYPE) -> BOOL {
    match lctype & 0xffff {
        LOCALE_SMONTHNAME1 |
        LOCALE_SMONTHNAME2 |
        LOCALE_SMONTHNAME3 |
        LOCALE_SMONTHNAME4 |
        LOCALE_SMONTHNAME5 |
        LOCALE_SMONTHNAME6 |
        LOCALE_SMONTHNAME7 |
        LOCALE_SMONTHNAME8 |
        LOCALE_SMONTHNAME9 |
        LOCALE_SMONTHNAME10 |
        LOCALE_SMONTHNAME11 |
        LOCALE_SMONTHNAME12 |
        LOCALE_SMONTHNAME13 => 1,
        _ => 0,
    }
}

unsafe fn create_registry_key() -> HANDLE {
    static cplW: [WCHAR; 14] = [
        'C' as WCHAR,
        'o' as WCHAR,
        'n' as WCHAR,
        't' as WCHAR,
        'r' as WCHAR,
        'o' as WCHAR,
        'l' as WCHAR,
        ' ' as WCHAR,
        'P' as WCHAR,
        'a' as WCHAR,
        'n' as WCHAR,
        'e' as WCHAR,
        'l' as WCHAR,
        0,
    ];
    static intlW: [WCHAR; 14] = [
        'I' as WCHAR,
        'n' as WCHAR,
        't' as WCHAR,
        'e' as WCHAR,
        'r' as WCHAR,
        'n' as WCHAR,
        'a' as WCHAR,
        't' as WCHAR,
        'i' as WCHAR,
        'o' as WCHAR,
        'n' as WCHAR,
        'a' as WCHAR,
        'l' as WCHAR,
        0,
    ];
    let mut attr: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut nameW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut cpl_key: HANDLE = core::ptr::null_mut();
    let mut hkey: HANDLE = core::ptr::null_mut();
    
    if RtlOpenCurrentUser(KEY_ALL_ACCESS, &mut hkey) != STATUS_SUCCESS {
        return core::ptr::null_mut();
    }
    
    attr.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    attr.RootDirectory = hkey;
    attr.ObjectName = &mut nameW;
    attr.Attributes = 0;
    attr.SecurityDescriptor = core::ptr::null_mut();
    attr.SecurityQualityOfService = core::ptr::null_mut();
    RtlInitUnicodeString(&mut nameW, cplW.as_ptr());
    
    if NtCreateKey(
        &mut cpl_key,
        KEY_ALL_ACCESS,
        &mut attr,
        0,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
    ) == 0
    {
        NtClose(attr.RootDirectory);
        attr.RootDirectory = cpl_key;
        RtlInitUnicodeString(&mut nameW, intlW.as_ptr());
        if NtCreateKey(
            &mut hkey,
            KEY_ALL_ACCESS,
            &mut attr,
            0,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
        ) != 0
        {
            hkey = core::ptr::null_mut();
        }
    }
    NtClose(attr.RootDirectory);
    hkey
}

#[no_mangle]
pub unsafe extern "system" fn GetUserDefaultLangID() -> LANGID {
    LANGIDFROMLCID(GetUserDefaultLCID())
}

#[no_mangle]
pub unsafe extern "system" fn GetSystemDefaultLangID() -> LANGID {
    LANGIDFROMLCID(GetSystemDefaultLCID())
}

#[no_mangle]
pub unsafe extern "system" fn GetUserDefaultLCID() -> LCID {
    let mut lcid: LCID = 0;
    NtQueryDefaultLocale(1u8, &raw mut lcid);
    lcid
}

#[no_mangle]
pub unsafe extern "system" fn GetSystemDefaultLCID() -> LCID {
    let mut lcid: LCID = 0;
    NtQueryDefaultLocale(0u8, core::ptr::addr_of_mut!(lcid));
    lcid
}

#[no_mangle]
pub unsafe extern "system" fn GetUserDefaultUILanguage() -> LANGID {
    let mut lang: LANGID = 0;
    NtQueryDefaultUILanguage(core::ptr::addr_of_mut!(lang));
    lang
}

#[no_mangle]
pub unsafe extern "system" fn GetSystemDefaultUILanguage() -> LANGID {
    let mut lang: LANGID = 0;
    NtQueryInstallUILanguage(core::ptr::addr_of_mut!(lang));
    lang
}

unsafe fn get_locale_registry_value(lctype: DWORD) -> *mut registry_value {
    let mut i: i32 = 0;
    while (i as usize) < (core::mem::size_of::<[registry_value; 39]>()).wrapping_div(core::mem::size_of::<registry_value>()) {
        if registry_values[i as usize].lctype == lctype {
            return core::ptr::addr_of_mut!(registry_values).cast::<registry_value>().offset(i as isize);
        }
        i += 1;
    }
    core::ptr::null_mut()
}

unsafe fn get_registry_locale_info(
    registry_value: *mut registry_value,
    buffer: LPWSTR,
    len: INT,
) -> INT {
    let mut size: DWORD = 0;
    let mut ret: INT;
    let mut hkey: HANDLE;
    let mut status: NTSTATUS;
    let mut nameW: _UNICODE_STRING = core::mem::zeroed();
    let mut info: *mut KEY_VALUE_PARTIAL_INFORMATION;
    let info_size: i32 = core::mem::size_of::<DWORD>() as i32 + 
                         core::mem::size_of::<DWORD>() as i32 + 
                         core::mem::size_of::<DWORD>() as i32;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(cache_section));

    if (*registry_value).cached_value.is_null() {
        hkey = create_registry_key();
        if hkey.is_null() {
            RtlLeaveCriticalSection(core::ptr::addr_of_mut!(cache_section));
            return -1;
        }

        RtlInitUnicodeString(core::ptr::addr_of_mut!(nameW), (*registry_value).name);
        size = (info_size as usize + (len as usize) * core::mem::size_of::<WCHAR>()) as DWORD;

        info = HeapAlloc(GetProcessHeap(), 0, size as usize) as *mut KEY_VALUE_PARTIAL_INFORMATION;
        if info.is_null() {
            NtClose(hkey);
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            RtlLeaveCriticalSection(core::ptr::addr_of_mut!(cache_section));
            return 0;
        }

        status = NtQueryValueKey(
            hkey,
            core::ptr::addr_of_mut!(nameW),
            KeyValuePartialInformation,
            info as *mut core::ffi::c_void,
            size,
            core::ptr::addr_of_mut!(size),
        );

        if status == STATUS_BUFFER_OVERFLOW && buffer.is_null() && size > info_size as u32 {
            let new_info = HeapReAlloc(GetProcessHeap(), 0, info as *mut core::ffi::c_void, size as SIZE_T)
                as *mut KEY_VALUE_PARTIAL_INFORMATION;
            if !new_info.is_null() {
                info = new_info;
                status = NtQueryValueKey(
                    hkey,
                    core::ptr::addr_of_mut!(nameW),
                    KeyValuePartialInformation,
                    info as *mut core::ffi::c_void,
                    size,
                    core::ptr::addr_of_mut!(size),
                );
            }
        }

        NtClose(hkey);

        if status == STATUS_SUCCESS {
            let mut length: INT = ((size as u32 - info_size as u32) / core::mem::size_of::<WCHAR>() as u32) as INT;
            let cached_value: LPWSTR;

            if length == 0 || *((*info).Data.as_ptr() as *const WCHAR).offset((length - 1) as isize) != 0 {
                length += 1;
            }

            cached_value = HeapAlloc(
                GetProcessHeap(),
                0,
                (length as usize) * core::mem::size_of::<WCHAR>(),
            ) as LPWSTR;

            if cached_value.is_null() {
                HeapFree(GetProcessHeap(), 0, info as *mut core::ffi::c_void);
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                RtlLeaveCriticalSection(core::ptr::addr_of_mut!(cache_section));
                return 0;
            }

            memcpy(
                cached_value as *mut u8,
                (*info).Data.as_ptr() as *const u8,
                ((length - 1) as usize) * core::mem::size_of::<WCHAR>(),
            );
            *cached_value.offset((length - 1) as isize) = 0;
            HeapFree(GetProcessHeap(), 0, info as *mut core::ffi::c_void);
            (*registry_value).cached_value = cached_value;
        } else {
            if status == STATUS_BUFFER_OVERFLOW && buffer.is_null() {
                ret = ((size as u32 - info_size as u32) / core::mem::size_of::<WCHAR>() as u32) as INT;
            } else if status == STATUS_OBJECT_NAME_NOT_FOUND {
                ret = -1;
            } else {
                SetLastError(RtlNtStatusToDosError(status));
                ret = 0;
            }
            HeapFree(GetProcessHeap(), 0, info as *mut core::ffi::c_void);
            RtlLeaveCriticalSection(core::ptr::addr_of_mut!(cache_section));
            return ret;
        }
    }

    ret = lstrlenW((*registry_value).cached_value) + 1;

    if !buffer.is_null() {
        if ret > len {
            SetLastError(ERROR_INSUFFICIENT_BUFFER);
            ret = 0;
        } else {
            lstrcpyW(buffer, (*registry_value).cached_value);
        }
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(cache_section));

    ret
}

#[no_mangle]
pub unsafe extern "system" fn GetLocaleInfoA(
    mut lcid: LCID,
    mut lctype: LCTYPE,
    mut buffer: LPSTR,
    mut len: INT,
) -> INT {
    let mut bufferW: LPWSTR;
    let mut lenW: INT;
    let mut ret: INT;

    TRACE!("(lcid=0x%x,lctype=0x%x,%p,%d)\n", lcid, lctype, buffer, len);

    if len < 0 || (len != 0 && buffer.is_null()) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    if (lctype & !LOCALE_LOCALEINFOFLAGSMASK) == LOCALE_SSHORTTIME
        || (lctype & LOCALE_RETURN_GENITIVE_NAMES) != 0
    {
        SetLastError(ERROR_INVALID_FLAGS);
        return 0;
    }

    if len == 0 {
        buffer = core::ptr::null_mut();
    }

    lenW = GetLocaleInfoW(lcid, lctype, core::ptr::null_mut(), 0);
    if lenW == 0 {
        return 0;
    }

    bufferW = HeapAlloc(GetProcessHeap(), 0, (lenW as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR;
    if bufferW.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    ret = GetLocaleInfoW(lcid, lctype, bufferW, lenW);
    if ret != 0 {
        if (lctype & LOCALE_RETURN_NUMBER) != 0
            || ((lctype & !LOCALE_LOCALEINFOFLAGSMASK) == LOCALE_FONTSIGNATURE)
        {
            ret *= core::mem::size_of::<WCHAR>() as INT;
            if !buffer.is_null() {
                if ret <= len {
                    memcpy(
                        buffer as *mut u8,
                        bufferW as *const u8,
                        ret as usize,
                    );
                } else {
                    SetLastError(ERROR_INSUFFICIENT_BUFFER);
                    ret = 0;
                }
            }
        } else {
            let mut codepage: UINT = CP_ACP;
            if (lctype & LOCALE_USE_CP_ACP) == 0 {
                codepage = get_lcid_codepage(lcid);
            }
            ret = WideCharToMultiByte(
                codepage,
                0,
                bufferW,
                ret,
                buffer,
                len,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
        }
    }
    HeapFree(GetProcessHeap(), 0, bufferW as *mut core::ffi::c_void);
    return ret;
}

unsafe fn get_value_base_by_lctype(lctype: LCTYPE) -> i32 {
    if lctype == LOCALE_ILANGUAGE || lctype == LOCALE_IDEFAULTLANGUAGE {
        16
    } else {
        10
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetLocaleInfoW(
    mut lcid: LCID,
    mut lctype: LCTYPE,
    mut buffer: LPWSTR,
    mut len: INT,
) -> INT {
    let mut lang_id: LANGID;
    let mut hrsrc: HRSRC;
    let mut hmem: HGLOBAL;
    let mut ret: INT;
    let mut lcflags: UINT;
    let mut p: *const WCHAR;
    let mut i: u32;

    if len < 0 || (len != 0 && buffer.is_null()) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    if (lctype & LOCALE_RETURN_GENITIVE_NAMES) != 0 && is_genitive_name_supported(lctype) == 0 {
        SetLastError(ERROR_INVALID_FLAGS);
        return 0;
    }

    if len == 0 {
        buffer = core::ptr::null_mut();
    }

    lcid = convert_default_lcid(lcid, lctype);

    lcflags = (lctype & LOCALE_LOCALEINFOFLAGSMASK) as UINT;
    lctype &= 0xffff;

    TRACE!("(lcid=0x%x,lctype=0x%x,%p,%d)\n", lcid, lctype, buffer, len);

    if (lcflags & LOCALE_NOUSEROVERRIDE) == 0
        && lcid == convert_default_lcid(LOCALE_USER_DEFAULT, lctype)
    {
        let value = get_locale_registry_value(lctype);

        if !value.is_null() {
            if (lcflags & LOCALE_RETURN_NUMBER) != 0 {
                let mut tmp: [WCHAR; 16] = [0; 16];
                ret = get_registry_locale_info(
                    value,
                    tmp.as_mut_ptr(),
                    (core::mem::size_of::<[WCHAR; 16]>() / core::mem::size_of::<WCHAR>()) as INT,
                );
                if ret > 0 {
                    let mut end: *mut WCHAR = core::ptr::null_mut();
                    let number: UINT = strtolW(
                        tmp.as_mut_ptr(),
                        core::ptr::addr_of_mut!(end),
                        get_value_base_by_lctype(lctype),
                    ) as UINT;
                    if *end != 0 {
                        SetLastError(ERROR_INVALID_FLAGS);
                        return 0;
                    }
                    ret = (core::mem::size_of::<UINT>() / core::mem::size_of::<WCHAR>()) as INT;
                    if buffer.is_null() {
                        return ret;
                    }
                    if ret > len {
                        SetLastError(ERROR_INSUFFICIENT_BUFFER);
                        return 0;
                    }
                    memcpy(
                        buffer as *mut _,
                        core::ptr::addr_of!(number) as *const _,
                        core::mem::size_of::<UINT>(),
                    );
                }
            } else {
                ret = get_registry_locale_info(value, buffer, len);
            }

            if ret != -1 {
                return ret;
            }
        }
    }

    lang_id = LANGIDFROMLCID(lcid) as LANGID;

    if SUBLANGID(lang_id as u16) == SUBLANG_NEUTRAL as u32 {
        lang_id = get_default_sublang(lang_id);
    }

    hrsrc = FindResourceExW(
        kernel32_handle,
        RT_STRING as LPWSTR,
        ULongToPtr((lctype >> 4) + 1) as LPWSTR,
        lang_id,
    );
    if hrsrc.is_null() {
        SetLastError(ERROR_INVALID_FLAGS);
        return 0;
    }
    hmem = LoadResource(kernel32_handle, hrsrc);
    if hmem.is_null() {
        return 0;
    }

    p = LockResource(hmem) as *const WCHAR;
    i = 0;
    while i < (lctype & 0x0f) {
        p = p.offset((*p as isize) + 1);
        i += 1;
    }

    if (lcflags & LOCALE_RETURN_NUMBER) != 0 {
        ret = (core::mem::size_of::<UINT>() / core::mem::size_of::<WCHAR>()) as INT;
    } else if is_genitive_name_supported(lctype) != 0 && *p != 0 {
        i = 1;
        while i <= *p as u32 {
            if *p.offset(i as isize) == 0 {
                break;
            }
            i += 1;
        }

        if i <= *p as u32 && (lcflags & LOCALE_RETURN_GENITIVE_NAMES) != 0 {
            ret = (*p as u32 - i + 1) as INT;
            p = p.offset(i as isize);
        } else {
            ret = i as INT;
        }
    } else {
        ret = if lctype == LOCALE_FONTSIGNATURE {
            *p as INT
        } else {
            (*p as INT) + 1
        };
    }

    if buffer.is_null() {
        return ret;
    }

    if ret > len {
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        return 0;
    }

    if (lcflags & LOCALE_RETURN_NUMBER) != 0 {
        let mut number: UINT;
        let mut end: *mut WCHAR = core::ptr::null_mut();
        let tmp: *mut WCHAR = HeapAlloc(
            GetProcessHeap(),
            0,
            ((*p as usize + 1) * core::mem::size_of::<WCHAR>()) as _,
        ) as *mut WCHAR;
        if tmp.is_null() {
            return 0;
        }
        memcpy(
            tmp as *mut _,
            p.offset(1) as *const _,
            (*p as usize) * core::mem::size_of::<WCHAR>(),
        );
        *tmp.offset(*p as isize) = 0;
        number = strtolW(tmp, core::ptr::addr_of_mut!(end), get_value_base_by_lctype(lctype))
            as UINT;
        if *end == 0 {
            memcpy(
                buffer as *mut _,
                core::ptr::addr_of!(number) as *const _,
                core::mem::size_of::<UINT>(),
            );
        } else {
            SetLastError(ERROR_INVALID_FLAGS);
            ret = 0;
        }
        HeapFree(GetProcessHeap(), 0, tmp as _);

        TRACE!("(lcid=0x%x,lctype=0x%x,%p,%d) returning number %d\n", lcid, lctype, buffer, len, number);
    } else {
        memcpy(
            buffer as *mut _,
            p.offset(1) as *const _,
            (ret as usize) * core::mem::size_of::<WCHAR>(),
        );
        if lctype != LOCALE_FONTSIGNATURE {
            *buffer.offset((ret - 1) as isize) = 0;
        }

        TRACE!("(lcid=0x%x,lctype=0x%x,%p,%d) returning %d %s\n", lcid, lctype, buffer, len, ret, debugstr_w(buffer));
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn SetLocaleInfoA(
    lcid: LCID,
    lctype: LCTYPE,
    data: LPCSTR,
) -> BOOL {
    let mut codepage: UINT = CP_ACP;
    let len: DWORD;
    let ret: BOOL;

    if (lctype & LOCALE_USE_CP_ACP) == 0 {
        codepage = get_lcid_codepage(lcid);
    }

    if data.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    len = MultiByteToWideChar(codepage, 0, data, -1, core::ptr::null_mut(), 0) as DWORD;

    let strW = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<WCHAR>()) as *mut WCHAR;
    if strW.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    MultiByteToWideChar(codepage, 0, data, -1, strW, len as i32);
    ret = SetLocaleInfoW(lcid, lctype, strW);
    HeapFree(GetProcessHeap(), 0, strW as *mut core::ffi::c_void);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn SetLocaleInfoW(
    mut lcid: LCID,
    mut lctype: LCTYPE,
    data: LPCWSTR,
) -> BOOL {
    static intlW: [WCHAR; 5] = [b'i' as u16, b'n' as u16, b't' as u16, b'l' as u16, 0];
    let mut valueW: _UNICODE_STRING = core::mem::zeroed();
    let mut status: NTSTATUS;

    lctype &= 0xffff;
    let value = get_locale_registry_value(lctype);

    if data.is_null() || value.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if lctype == LOCALE_IDATE || lctype == LOCALE_ILDATE {
        SetLastError(ERROR_INVALID_FLAGS);
        return 0;
    }

    TRACE!("setting %x (%s) to %s\n", lctype, debugstr_w((*value).name), debugstr_w(data));

    WriteProfileStringW(intlW.as_ptr(), (*value).name, data);

    let hkey = create_registry_key();
    if hkey.is_null() {
        return 0;
    }

    RtlInitUnicodeString(&raw mut valueW, (*value).name);
    status = NtSetValueKey(
        hkey,
        &raw mut valueW,
        0,
        REG_SZ,
        data as PVOID,
        ((strlenW(data) as usize + 1) * core::mem::size_of::<WCHAR>()) as ULONG,
    );

    RtlEnterCriticalSection(&raw mut cache_section);
    HeapFree(GetProcessHeap(), 0, (*value).cached_value as PVOID);
    (*value).cached_value = core::ptr::null_mut();
    RtlLeaveCriticalSection(&raw mut cache_section);

    if lctype == LOCALE_SSHORTDATE || lctype == LOCALE_SLONGDATE {
        let mut szBuff: [WCHAR; 2] = [0; 2];

        let lpD = strrchrW(data, b'd' as u16);
        let lpM = strrchrW(data, b'M' as u16);
        let lpY = strrchrW(data, b'y' as u16);

        if lpD <= lpM {
            szBuff[0] = b'1' as u16;
        } else {
            if lpY <= lpM {
                szBuff[0] = b'2' as u16;
            } else {
                szBuff[0] = b'0' as u16;
            }
        }

        szBuff[1] = 0;

        if lctype == LOCALE_SSHORTDATE {
            lctype = LOCALE_IDATE;
        } else {
            lctype = LOCALE_ILDATE;
        }

        let value = get_locale_registry_value(lctype);

        WriteProfileStringW(intlW.as_ptr(), (*value).name, szBuff.as_ptr());

        RtlInitUnicodeString(&raw mut valueW, (*value).name);
        status = NtSetValueKey(
            hkey,
            &raw mut valueW,
            0,
            REG_SZ,
            szBuff.as_ptr() as PVOID,
            core::mem::size_of::<[WCHAR; 2]>() as ULONG,
        );

        RtlEnterCriticalSection(&raw mut cache_section);
        HeapFree(GetProcessHeap(), 0, (*value).cached_value as PVOID);
        (*value).cached_value = core::ptr::null_mut();
        RtlLeaveCriticalSection(&raw mut cache_section);
    }

    NtClose(hkey);

    if status != 0 {
        SetLastError(RtlNtStatusToDosError(status));
    }
    (status == 0) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn GetThreadLocale() -> LCID {
    let mut ret: LCID = (*NtCurrentTeb()).CurrentLocale;
    if ret == 0 {
        ret = GetUserDefaultLCID();
        (*NtCurrentTeb()).CurrentLocale = ret;
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn SetThreadLocale(mut lcid: LCID) -> BOOL {
    TRACE!("(0x%04X)\n", lcid);

    lcid = ConvertDefaultLocale(lcid);

    if lcid != GetThreadLocale() {
        if IsValidLocale(lcid, LCID_SUPPORTED) == 0 {
            SetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }

        (*NtCurrentTeb()).CurrentLocale = lcid;
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn ConvertDefaultLocale(mut lcid: LCID) -> LCID {

    let mut langid: LANGID;

    match lcid {
        LOCALE_INVARIANT => {}
        LOCALE_SYSTEM_DEFAULT => {
            lcid = GetSystemDefaultLCID();
        }
        LOCALE_USER_DEFAULT | LOCALE_NEUTRAL => {
            lcid = GetUserDefaultLCID();
        }
        _ => {
            langid = LANGIDFROMLCID(lcid);
            if SUBLANGID(langid) == SUBLANG_NEUTRAL {
                langid = get_default_sublang(langid);
                lcid = MAKELCID(langid, SORTIDFROMLCID(lcid));
            }
        }
    }
    lcid
}

#[no_mangle]
pub unsafe extern "system" fn IsValidLocale(lcid: LCID, flags: DWORD) -> BOOL {
    // The C looks the LANGID up as a resource language, and LANG_NEUTRAL (0) matches the
    // loader's any-language fallback, so it called LOCALE_NEUTRAL, LOCALE_USER_DEFAULT
    // and LOCALE_SYSTEM_DEFAULT valid. Windows says all three are not (winetests
    // kernel32/locale.c, beside RtlLcidToLocaleName). It mattered: ucrtbase's downlevel
    // IsValidLocaleName is IsValidLocale(LocaleNameToLCID(name)), and an unknown name
    // maps to 0, so every name "existed" and "en_US.UTF-8" came out as ANSI 1252.
    if lcid == LOCALE_NEUTRAL || lcid == LOCALE_USER_DEFAULT || lcid == LOCALE_SYSTEM_DEFAULT {
        return 0;
    }
    (FindResourceExW(
        kernel32_handle,
        6usize as LPWSTR,
        LOCALE_ILANGUAGE as LPCWSTR,
        LANGIDFROMLCID(lcid) as WORD,
    ) != core::ptr::null_mut()) as BOOL
}

unsafe extern "system" fn enum_lang_proc_a(
    hModule: HMODULE,
    type_0: LPCSTR,
    name: LPCSTR,
    LangID: WORD,
    lParam: LONG_PTR,
) -> BOOL {
    let lpfnLocaleEnum: LOCALE_ENUMPROCA = 
        core::mem::transmute::<isize, LOCALE_ENUMPROCA>(lParam as isize);
    let mut buf: [i8; 20] = [0; 20];
    sprintf(
        buf.as_mut_ptr(),
        b"%08x\0".as_ptr() as *const i8,
        LangID as UINT,
    );
    lpfnLocaleEnum.expect("non-null function pointer")(buf.as_mut_ptr())
}
unsafe extern "system" fn enum_lang_proc_w(
    _hModule: HMODULE,
    _type: LPCWSTR,
    _name: LPCWSTR,
    LangID: WORD,
    lParam: LONG_PTR,
) -> BOOL {
    static FORMAT_W: [WCHAR; 5] = [
        b'%' as u16,
        b'0' as u16,
        b'8' as u16,
        b'x' as u16,
        0,
    ];
    
    let lpfnLocaleEnum: LOCALE_ENUMPROCW = core::mem::transmute(lParam);
    let mut buf: [WCHAR; 20] = [0; 20];
    
    sprintfW(
        buf.as_mut_ptr(),
        FORMAT_W.as_ptr(),
        LangID as u32,
    );
    
    (lpfnLocaleEnum.expect("EnumSystemLocalesW callback must be non-null"))(buf.as_mut_ptr())
}

#[no_mangle]
pub unsafe extern "system" fn EnumSystemLocalesA(
    lpfnLocaleEnum: LOCALE_ENUMPROCA,
    dwFlags: DWORD,
) -> BOOL {

    TRACE!("(%p,%08x)\n", lpfnLocaleEnum.map_or(core::ptr::null(), |f| f as *const core::ffi::c_void), dwFlags);

    EnumResourceLanguagesA(
        kernel32_handle,
        (6 & 0xffff) as LPSTR as LPCSTR,
        LOCALE_ILANGUAGE as LPCSTR,
        as_enumreslang_a(enum_lang_proc_a),
        core::mem::transmute::<LOCALE_ENUMPROCA, LONG_PTR>(lpfnLocaleEnum),
    );

    1
}

#[no_mangle]
pub unsafe extern "system" fn EnumSystemLocalesW(
    lpfnLocaleEnum: LOCALE_ENUMPROCW,
    dwFlags: DWORD,
) -> BOOL {

    DbgPrintEx(
        gDebugChannel,
        DPFLTR_TRACE_LEVEL,
        b"(%s:%d) (%p,%08x)\n\0".as_ptr() as PCSTR,
        b"locale.c\0".as_ptr() as PCSTR,
        3003,
        lpfnLocaleEnum.map_or(core::ptr::null(), |f| f as *const core::ffi::c_void),
        dwFlags,
    );

    EnumResourceLanguagesW(
        kernel32_handle,
        (6u64 & 0xffff) as LPCWSTR,
        LOCALE_ILANGUAGE as LPCWSTR,
        as_enumreslang_w(enum_lang_proc_w),
        core::mem::transmute::<LOCALE_ENUMPROCW, LONG_PTR>(lpfnLocaleEnum),
    );

    1
}

#[no_mangle]
pub unsafe extern "system" fn VerLanguageNameA(
    wLang: DWORD,
    szLang: LPSTR,
    nSize: DWORD,
) -> DWORD {
    GetLocaleInfoA(
        MAKELCID(wLang as LANGID, SORT_DEFAULT),
        LOCALE_SENGLANGUAGE,
        szLang,
        nSize as INT,
    ) as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn VerLanguageNameW(
    wLang: DWORD,
    szLang: LPWSTR,
    nSize: DWORD,
) -> DWORD {
    GetLocaleInfoW(
        MAKELCID(wLang as LANGID, SORT_DEFAULT),
        LOCALE_SENGLANGUAGE,
        szLang,
        nSize as INT,
    ) as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn GetStringTypeW(
    type_0: DWORD,
    src: LPCWSTR,
    mut count: INT,
    mut chartype: LPWORD,
) -> BOOL {
    static TYPE2_MAP: [u8; 16] = [
        C2_NOTAPPLICABLE as u8,
        C2_LEFTTORIGHT as u8,
        C2_RIGHTTOLEFT as u8,
        C2_EUROPENUMBER as u8,
        C2_EUROPESEPARATOR as u8,
        C2_EUROPETERMINATOR as u8,
        C2_ARABICNUMBER as u8,
        C2_COMMONSEPARATOR as u8,
        C2_BLOCKSEPARATOR as u8,
        C2_SEGMENTSEPARATOR as u8,
        C2_WHITESPACE as u8,
        C2_OTHERNEUTRAL as u8,
        C2_RIGHTTOLEFT as u8,
        C2_NOTAPPLICABLE as u8,
        C2_NOTAPPLICABLE as u8,
        C2_OTHERNEUTRAL as u8,
    ];

    if src.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if count == -1 {
        count = (strlenW(src) + 1) as INT;
    }

    match type_0 {
        CT_CTYPE1 => {
            let mut src_ptr = src;
            while count > 0 {
                *chartype = (get_char_typeW(*src_ptr) & 0xfff) as WORD;
                src_ptr = src_ptr.offset(1);
                chartype = chartype.offset(1);
                count -= 1;
            }
        }
        CT_CTYPE2 => {
            let mut src_ptr = src;
            while count > 0 {
                *chartype = TYPE2_MAP[(get_char_typeW(*src_ptr) >> 12) as usize] as WORD;
                src_ptr = src_ptr.offset(1);
                chartype = chartype.offset(1);
                count -= 1;
            }
        }
        CT_CTYPE3 => {
            WARN!("CT_CTYPE3: semi-stub.\n");
            let mut src_ptr = src;
            while count > 0 {
                let c = *src_ptr as INT;
                let type1 = (get_char_typeW(*src_ptr) & 0xfff) as WORD;
                let mut type3: WORD = 0;

                if (type1 & C1_SPACE) != 0 {
                    type3 |= C3_SYMBOL;
                }
                if (type1 & C1_ALPHA) != 0 {
                    type3 |= C3_ALPHA;
                }
                if c >= 0x30A0 && c <= 0x30FF {
                    type3 |= C3_KATAKANA;
                }
                if c >= 0x3040 && c <= 0x309F {
                    type3 |= C3_HIRAGANA;
                }
                if c >= 0x4E00 && c <= 0x9FAF {
                    type3 |= C3_IDEOGRAPH;
                }
                if c == 0x0640 {
                    type3 |= C3_KASHIDA;
                }
                if c >= 0x3000 && c <= 0x303F {
                    type3 |= C3_SYMBOL;
                }
                if c >= 0xD800 && c <= 0xDBFF {
                    type3 |= C3_HIGHSURROGATE;
                }
                if c >= 0xDC00 && c <= 0xDFFF {
                    type3 |= C3_LOWSURROGATE;
                }
                if c >= 0xFF00 && c <= 0xFF60 {
                    type3 |= C3_FULLWIDTH;
                }
                if c >= 0xFF00 && c <= 0xFF20 {
                    type3 |= C3_SYMBOL;
                }
                if c >= 0xFF3B && c <= 0xFF40 {
                    type3 |= C3_SYMBOL;
                }
                if c >= 0xFF5B && c <= 0xFF60 {
                    type3 |= C3_SYMBOL;
                }
                if c >= 0xFF21 && c <= 0xFF3A {
                    type3 |= C3_ALPHA;
                }
                if c >= 0xFF41 && c <= 0xFF5A {
                    type3 |= C3_ALPHA;
                }
                if c >= 0xFFE0 && c <= 0xFFE6 {
                    type3 |= C3_FULLWIDTH;
                }
                if c >= 0xFFE0 && c <= 0xFFE6 {
                    type3 |= C3_SYMBOL;
                }
                if c >= 0xFF61 && c <= 0xFFDC {
                    type3 |= C3_HALFWIDTH;
                }
                if c >= 0xFF61 && c <= 0xFF64 {
                    type3 |= C3_SYMBOL;
                }
                if c >= 0xFF65 && c <= 0xFF9F {
                    type3 |= C3_KATAKANA;
                }
                if c >= 0xFF65 && c <= 0xFF9F {
                    type3 |= C3_ALPHA;
                }
                if c >= 0xFFE8 && c <= 0xFFEE {
                    type3 |= C3_HALFWIDTH;
                }
                if c >= 0xFFE8 && c <= 0xFFEE {
                    type3 |= C3_SYMBOL;
                }

                *chartype = type3;
                src_ptr = src_ptr.offset(1);
                chartype = chartype.offset(1);
                count -= 1;
            }
        }
        _ => {
            SetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn GetStringTypeExW(
    locale: LCID,
    type_0: DWORD,
    src: LPCWSTR,
    count: INT,
    chartype: LPWORD,
) -> BOOL {
    GetStringTypeW(type_0, src, count, chartype)
}

#[no_mangle]
pub unsafe extern "system" fn GetStringTypeA(
    locale: LCID,
    type_0: DWORD,
    src: LPCSTR,
    mut count: INT,
    chartype: LPWORD,
) -> BOOL {
    let mut cp: UINT;
    let countW: INT;
    let srcW: LPWSTR;
    let mut ret: BOOL = 0;

    if count == -1 {
        count = (strlen(src) + 1) as INT;
    }

    cp = get_lcid_codepage(locale);
    if cp == 0 {
        FIXME(b"For locale %04x using current ANSI code page\n\0".as_ptr() as *const i8, locale);
        cp = GetACP();
    }

    countW = MultiByteToWideChar(cp, 0, src, count, core::ptr::null_mut(), 0);
    srcW = HeapAlloc(GetProcessHeap(), 0, (countW as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR;
    if !srcW.is_null() {
        MultiByteToWideChar(cp, 0, src, count, srcW, countW);
        ret = GetStringTypeW(type_0, srcW, countW, chartype);
        HeapFree(GetProcessHeap(), 0, srcW as LPVOID);
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn GetStringTypeExA(
    locale: LCID,
    type_0: DWORD,
    src: LPCSTR,
    count: INT,
    chartype: LPWORD,
) -> BOOL {
    GetStringTypeA(locale, type_0, src, count, chartype)
}

unsafe fn map_byterev(src: *const WCHAR, mut len: i32, dst: *mut WCHAR) {
    let mut src = src;
    let mut dst = dst;
    while len > 0 {
        *dst = RtlUshortByteSwap(*src);
        src = src.offset(1);
        dst = dst.offset(1);
        len -= 1;
    }
}

unsafe fn map_to_hiragana(
    src: *const WCHAR,
    srclen: i32,
    dst: *mut WCHAR,
    dstlen: i32,
) -> i32 {
    let mut src = src;
    let mut srclen = srclen;
    let mut pos: i32 = 0;

    while srclen != 0 {
        let mut wch: WCHAR = *src;
        if (0x30A1 <= wch && wch <= 0x30F3)
            || wch == 0x30F4
            || wch == 0x30F5
            || wch == 0x30FD
            || wch == 0x30FE
        {
            wch = wch - 0x60;
        }
        if pos < dstlen {
            *dst.offset(pos as isize) = wch;
        }
        src = src.offset(1);
        srclen -= 1;
        pos += 1;
    }
    pos
}

unsafe fn map_to_katakana(
    mut src: *const WCHAR,
    mut srclen: i32,
    mut dst: *mut WCHAR,
    mut dstlen: i32,
) -> i32 {
    let mut pos: i32 = 0;
    pos = 0;
    while srclen != 0 {
        let mut wch: WCHAR = *src;
        if 0x3041 <= wch as i32 && wch as i32 <= 0x3093
            || wch as i32 == 3094
            || wch as i32 == 0x3095
            || wch as i32 == 0x309D
            || wch as i32 == 0x309E
        {
            wch = (wch as i32 + 0x60) as WCHAR;
        }
        if pos < dstlen {
            *dst.offset(pos as isize) = wch;
        }
        src = src.offset(1);
        srclen -= 1;
        pos += 1;
    }
    return pos;
}

unsafe fn map_to_halfwidth(
    flags: DWORD,
    mut src: *const WCHAR,
    mut srclen: i32,
    dst: *mut WCHAR,
    dstlen: i32,
) -> i32 {
    let mut pos: i32 = 0;
    let count1 = (*(&raw const full2half_table)).len() as i32;
    let table1 = (*(&raw const full2half_table)).as_ptr();

    while srclen > 0 {
        let mut ch = *src;

        if (flags & LCMAP_KATAKANA) != 0 {
            map_to_katakana(
                core::ptr::addr_of_mut!(ch),
                1,
                core::ptr::addr_of_mut!(ch),
                1,
            );
        } else if (flags & LCMAP_HIRAGANA) != 0 {
            map_to_hiragana(
                core::ptr::addr_of_mut!(ch),
                1,
                core::ptr::addr_of_mut!(ch),
                1,
            );
        }

        if (ch as i32) < 0x3000 {
            if pos < dstlen {
                *dst.offset(pos as isize) = ch;
            }
        } else if 0xFF01 <= (ch as i32) && (ch as i32) <= 0xFF5E {
            if pos < dstlen {
                *dst.offset(pos as isize) = (ch as i32 - 0xFEE0) as WCHAR;
            }
        } else {
            let mut i = count1 - 1;
            while i >= 0 {
                if (*table1.offset(i as isize))[0] as i32 != ch as i32 {
                    i -= 1;
                    continue;
                }

                if (*table1.offset(i as isize))[2] as i32 == 0 {
                    if pos < dstlen {
                        *dst.offset(pos as isize) = (*table1.offset(i as isize))[1];
                    }
                } else if dstlen == 0 {
                    pos += 1;
                } else if pos + 1 < dstlen {
                    *dst.offset(pos as isize) = (*table1.offset(i as isize))[1];
                    pos += 1;
                    *dst.offset(pos as isize) = (*table1.offset(i as isize))[2];
                } else {
                    *dst.offset(pos as isize) = ch;
                }
                break;
            }

            if i < 0 {
                if pos < dstlen {
                    *dst.offset(pos as isize) = ch;
                }
            }
        }

        src = src.offset(1);
        srclen -= 1;
        pos += 1;
    }

    pos
}

unsafe fn map_to_fullwidth(mut src: *const WCHAR, mut srclen: i32, dst: *mut WCHAR, dstlen: i32) -> i32 {
    let mut pos: i32;
    let mut i: i32;
    let table1: *const FULL2HALF_ENTRY = core::ptr::addr_of!(full2half_table) as *const FULL2HALF_ENTRY;
    let table2: *const DAKUTEN_ENTRY = core::ptr::addr_of!(dakuten_table) as *const DAKUTEN_ENTRY;
    let count1: i32 = ARRAY_SIZE!((*(&raw const full2half_table))) as i32;
    let count2: i32 = ARRAY_SIZE!((*(&raw const dakuten_table))) as i32;

    pos = 0;
    while srclen != 0 {
        let ch: WCHAR = *src;

        if ch == 0x20 {
            if pos < dstlen {
                *dst.offset(pos as isize) = 0x3000;
            }
        } else if 0x21 <= ch && ch <= 0x7E {
            if pos < dstlen {
                *dst.offset(pos as isize) = ch + 0xFEE0;
            }
        } else if ch < 0xFF00 {
            if pos < dstlen {
                *dst.offset(pos as isize) = ch;
            }
        } else {
            i = count1 - 1;
            while i >= 0 {
                if GET_HALF1!(table1, i) != ch {
                    i -= 1;
                    continue;
                }

                if GET_HALF2!(table1, i) == 0 {
                    if pos < dstlen {
                        *dst.offset(pos as isize) = GET_FULL!(table1, i);
                    }
                    break;
                }

                if srclen <= 1 || GET_HALF2!(table1, i) != *src.offset(1) {
                    i -= 1;
                    continue;
                }

                srclen -= 1;
                src = src.offset(1);

                if pos < dstlen {
                    *dst.offset(pos as isize) = GET_FULL!(table1, i);
                }
                break;
            }

            if i >= 0 {
                src = src.offset(1);
                srclen -= 1;
                pos += 1;
                continue;
            }

            i = count2 - 1;
            while i >= 0 {
                if GET_SINGLE1!(table2, i) != ch {
                    i -= 1;
                    continue;
                }

                if srclen <= 1 || GET_SINGLE2!(table2, i) != *src.offset(1) {
                    i -= 1;
                    continue;
                }

                srclen -= 1;
                src = src.offset(1);

                if pos < dstlen {
                    *dst.offset(pos as isize) = GET_VOICED!(table2, i);
                }
                break;
            }

            if i >= 0 {
                src = src.offset(1);
                srclen -= 1;
                pos += 1;
                continue;
            }

            if pos < dstlen {
                *dst.offset(pos as isize) = ch;
            }
        }

        src = src.offset(1);
        srclen -= 1;
        pos += 1;
    }

    pos
}

unsafe fn map_to_lowercase(flags: DWORD, src: *const WCHAR, srclen: i32, dst: *mut WCHAR, dstlen: i32) -> i32 {
    let mut pos: i32 = 0;
    let mut src_ptr = src;
    let mut remaining = srclen;
    
    while remaining != 0 {
        let wch = *src_ptr;
        if !((flags & NORM_IGNORESYMBOLS) != 0 && (get_char_typeW(wch) & (C1_PUNCT | C1_SPACE)) != 0) {
            if pos < dstlen {
                *dst.offset(pos as isize) = tolowerW(wch);
            }
            pos += 1;
        }
        src_ptr = src_ptr.offset(1);
        remaining -= 1;
    }
    pos
}

unsafe fn map_to_uppercase(flags: DWORD, src: *const WCHAR, srclen: i32, dst: *mut WCHAR, dstlen: i32) -> i32 {
    let mut pos: i32 = 0;
    let mut src_ptr = src;
    let mut remaining = srclen;
    
    while remaining != 0 {
        let wch: WCHAR = *src_ptr;
        if !((flags & NORM_IGNORESYMBOLS) != 0 && (get_char_typeW(wch) & (C1_PUNCT | C1_SPACE)) != 0) {
            if pos < dstlen {
                *dst.offset(pos as isize) = toupperW(wch);
            }
            pos += 1;
        }
        src_ptr = src_ptr.offset(1);
        remaining -= 1;
    }
    pos
}
unsafe extern "C" fn compare_wchar_pair(
    x: *const core::ffi::c_void,
    y: *const core::ffi::c_void,
) -> core::ffi::c_int {
    let a: *const WCHAR_PAIR = x as *const WCHAR_PAIR;
    let b: *const WCHAR_PAIR = y as *const WCHAR_PAIR;
    if ((*a).from as core::ffi::c_int) < ((*b).from as core::ffi::c_int) {
        return -1;
    }
    if ((*a).from as core::ffi::c_int) > ((*b).from as core::ffi::c_int) {
        return 1;
    }
    return 0;
}

unsafe fn find_wchar_pair(pairs: *const WCHAR_PAIR, count: usize, ch: WCHAR) -> WCHAR {
    let found: PWCHAR_PAIR = bsearch(
        core::ptr::addr_of!(ch) as *const core::ffi::c_void,
        pairs as *const core::ffi::c_void,
        count,
        core::mem::size_of::<WCHAR_PAIR>(),
        Some(compare_wchar_pair),
    ) as PWCHAR_PAIR;
    if !found.is_null() {
        return (*found).to;
    }
    ch
}

unsafe fn map_to_simplified_chinese(
    _flags: DWORD,
    mut src: *const WCHAR,
    mut srclen: i32,
    dst: *mut WCHAR,
    dstlen: i32,
) -> i32 {
    let mut pos: i32 = 0;
    while srclen != 0 {
        let wch: WCHAR = *src;
        if pos < dstlen {
            *dst.offset(pos as isize) = find_wchar_pair(
                core::ptr::addr_of!(s_tra2sim) as *const WCHAR_PAIR,
                core::mem::size_of_val(&*(&raw const s_tra2sim)) / core::mem::size_of::<WCHAR_PAIR>(),
                wch,
            );
        }
        pos += 1;
        src = src.offset(1);
        srclen -= 1;
    }
    pos
}

unsafe fn map_to_traditional_chinese(
    flags: DWORD,
    src: *const WCHAR,
    mut srclen: i32,
    dst: *mut WCHAR,
    dstlen: i32,
) -> i32 {
    let mut pos: i32 = 0;
    let mut src = src;
    
    while srclen != 0 {
        let wch: WCHAR = *src;
        if pos < dstlen {
            *dst.offset(pos as isize) = find_wchar_pair(
                (*(&raw const s_sim2tra)).as_ptr(),
                (*(&raw const s_sim2tra)).len(),
                wch,
            );
        }
        pos += 1;
        src = src.offset(1);
        srclen -= 1;
    }
    
    pos
}

unsafe fn map_remove_ignored(flags: DWORD, src: *const WCHAR, srclen: i32, dst: *mut WCHAR, dstlen: i32) -> i32 {
    let mut pos = 0i32;
    let mut wC1: WORD = 0;
    let mut wC2: WORD = 0;
    let mut wC3: WORD = 0;
    let mut src = src;
    let mut srclen = srclen;

    while srclen != 0 {
        let wch = *src;
        GetStringTypeW(CT_CTYPE1, core::ptr::addr_of!(wch), 1, core::ptr::addr_of_mut!(wC1));
        GetStringTypeW(CT_CTYPE2, core::ptr::addr_of!(wch), 1, core::ptr::addr_of_mut!(wC2));
        GetStringTypeW(CT_CTYPE3, core::ptr::addr_of!(wch), 1, core::ptr::addr_of_mut!(wC3));

        if (flags & NORM_IGNORESYMBOLS) != 0 {
            if ((wC1 & C1_PUNCT) != 0) || ((wC3 & C3_SYMBOL) != 0) {
                src = src.offset(1);
                srclen -= 1;
                continue;
            }
        }
        if (flags & NORM_IGNORENONSPACE) != 0 {
            if ((wC2 & C2_OTHERNEUTRAL) != 0) && ((wC3 & (C3_NONSPACING | C3_DIACRITIC)) != 0) {
                src = src.offset(1);
                srclen -= 1;
                continue;
            }
        }
        if pos < dstlen {
            *dst.offset(pos as isize) = wch;
        }
        pos += 1;

        src = src.offset(1);
        srclen -= 1;
    }
    pos
}

unsafe fn lcmap_string(
    mut flags: DWORD,
    src: *const WCHAR,
    srclen: i32,
    dst: *mut WCHAR,
    dstlen: i32,
) -> i32 {
    let mut ret: i32 = 0;

    if (flags & (LCMAP_LOWERCASE | LCMAP_UPPERCASE)) == (LCMAP_LOWERCASE | LCMAP_UPPERCASE) {
        SetLastError(ERROR_INVALID_FLAGS);
        return 0;
    }

    let switch_flags = flags & !(LCMAP_BYTEREV | LCMAP_LOWERCASE | LCMAP_UPPERCASE | LCMAP_LINGUISTIC_CASING);
    
    'switch: loop {
        match switch_flags {
            LCMAP_HIRAGANA => {
                ret = map_to_hiragana(src, srclen, dst, dstlen);
                break 'switch;
            }
            LCMAP_KATAKANA => {
                ret = map_to_katakana(src, srclen, dst, dstlen);
                break 'switch;
            }
            LCMAP_HALFWIDTH => {
                ret = map_to_halfwidth(flags, src, srclen, dst, dstlen);
                break 'switch;
            }
            _ if switch_flags == (LCMAP_HIRAGANA | LCMAP_HALFWIDTH) => {
                ret = map_to_halfwidth(flags, src, srclen, dst, dstlen);
                break 'switch;
            }
            _ if switch_flags == (LCMAP_KATAKANA | LCMAP_HALFWIDTH) => {
                ret = map_to_halfwidth(flags, src, srclen, dst, dstlen);
                break 'switch;
            }
            LCMAP_FULLWIDTH => {
                ret = map_to_fullwidth(src, srclen, dst, dstlen);
                break 'switch;
            }
            _ if switch_flags == (LCMAP_HIRAGANA | LCMAP_FULLWIDTH) => {
                ret = map_to_fullwidth(src, srclen, dst, dstlen);
                if dstlen != 0 && ret != 0 {
                    map_to_hiragana(dst, ret, dst, dstlen);
                }
                break 'switch;
            }
            _ if switch_flags == (LCMAP_KATAKANA | LCMAP_FULLWIDTH) => {
                ret = map_to_fullwidth(src, srclen, dst, dstlen);
                if dstlen != 0 && ret != 0 {
                    map_to_katakana(dst, ret, dst, dstlen);
                }
                break 'switch;
            }
            LCMAP_SIMPLIFIED_CHINESE => {
                ret = map_to_simplified_chinese(flags, src, srclen, dst, dstlen);
                break 'switch;
            }
            LCMAP_TRADITIONAL_CHINESE => {
                ret = map_to_traditional_chinese(flags, src, srclen, dst, dstlen);
                break 'switch;
            }
            // C original: `case NORM_IGNORENONSPACE: case NORM_IGNORESYMBOLS:
            // case NORM_IGNORENONSPACE|NORM_IGNORESYMBOLS:`. The translation
            // rendered this as the Rust OR-PATTERN `NORM_IGNORENONSPACE |
            // NORM_IGNORESYMBOLS | 0`, whose literal `0` alternative captured
            // switch_flags == 0 — so every plain LCMAP_LOWERCASE/UPPERCASE call
            // (flags still nonzero) hit this arm's flag check and failed with
            // ERROR_INVALID_FLAGS, making the real `0 =>` arm below dead code.
            // Every LCMapString case-mapping in the OS failed this way (loudest
            // victim: svchost's SvchostCharLowerW at service start).
            _ if switch_flags == NORM_IGNORENONSPACE
                || switch_flags == NORM_IGNORESYMBOLS
                || switch_flags == (NORM_IGNORENONSPACE | NORM_IGNORESYMBOLS) => {
                if flags & !(NORM_IGNORENONSPACE | NORM_IGNORESYMBOLS | LCMAP_BYTEREV) != 0 {
                    SetLastError(ERROR_INVALID_FLAGS);
                    return 0;
                }
                ret = map_remove_ignored(flags, src, srclen, dst, dstlen);
                break 'switch;
            }
            0 => {
                if flags & LCMAP_LOWERCASE != 0 {
                    ret = map_to_lowercase(flags, src, srclen, dst, dstlen);
                    flags &= !LCMAP_LOWERCASE;
                    break 'switch;
                }
                if flags & LCMAP_UPPERCASE != 0 {
                    ret = map_to_uppercase(flags, src, srclen, dst, dstlen);
                    flags &= !LCMAP_UPPERCASE;
                    break 'switch;
                }
                if flags & LCMAP_BYTEREV != 0 {
                    if dstlen == 0 {
                        ret = srclen;
                        break 'switch;
                    }
                    ret = core::cmp::min(srclen, dstlen);
                    core::ptr::copy_nonoverlapping(src, dst, ret as usize);
                    break 'switch;
                }
                SetLastError(ERROR_INVALID_FLAGS);
                return 0;
            }
            _ => {
                SetLastError(ERROR_INVALID_FLAGS);
                return 0;
            }
        }
    }

    if dstlen != 0 {
        if flags & LCMAP_LOWERCASE != 0 {
            map_to_lowercase(flags, dst, ret, dst, dstlen);
        }
        if flags & LCMAP_UPPERCASE != 0 {
            map_to_uppercase(flags, dst, ret, dst, dstlen);
        }
        if flags & LCMAP_BYTEREV != 0 {
            map_byterev(dst, core::cmp::min(ret, dstlen), dst);
        }

        if dstlen < ret {
            SetLastError(ERROR_INSUFFICIENT_BUFFER);
            return 0;
        }
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn LCMapStringEx(
    locale: LPCWSTR,
    mut flags: DWORD,
    src: LPCWSTR,
    mut srclen: i32,
    mut dst: LPWSTR,
    dstlen: i32,
    version: LPNLSVERSIONINFO,
    reserved: LPVOID,
    handle: LPARAM,
) -> i32 {
    if !version.is_null() {
        FIXME(b"unsupported version structure %p\n\0".as_ptr() as PCSTR, version);
    }
    if !reserved.is_null() {
        FIXME(b"unsupported reserved pointer %p\n\0".as_ptr() as PCSTR, reserved);
    }
    if handle != 0 {
        static mut once: i32 = 0;
        let fresh = once;
        once = once + 1;
        if fresh == 0 {
            FIXME(b"unsupported lparam %Ix\n\0".as_ptr() as PCSTR, handle);
        }
    }

    if src.is_null() || srclen == 0 || dstlen < 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if srclen < 0 {
        srclen = lstrlenW(src) + 1;
    }

    TRACE!("(%s,0x%08lx,%s,%d,%p,%d)\n", debugstr_w(locale), flags, debugstr_wn(src, srclen), srclen, dst, dstlen);

    flags &= !LOCALE_USE_CP_ACP;

    if src == dst as LPCWSTR && (flags & !(LCMAP_LOWERCASE | LCMAP_UPPERCASE)) != 0 {
        SetLastError(ERROR_INVALID_FLAGS);
        return 0;
    }

    if dstlen == 0 {
        dst = core::ptr::null_mut();
    }

    if (flags & LCMAP_SORTKEY) != 0 {
        let mut ret: i32;

        if srclen < 0 {
            srclen = strlenW(src) as i32;
        }

        ret = wine_get_sortkey(flags as i32, src, srclen, dst as *mut i8, dstlen);
        if ret == 0 {
            SetLastError(ERROR_INSUFFICIENT_BUFFER);
        } else {
            ret += 1;
        }
        return ret;
    }

    if (flags & SORT_STRINGSORT) != 0 {
        SetLastError(ERROR_INVALID_FLAGS);
        return 0;
    }

    lcmap_string(flags, src, srclen, dst, dstlen)
}

#[no_mangle]
pub unsafe extern "system" fn LCMapStringW(
    lcid: LCID,
    flags: DWORD,
    src: LPCWSTR,
    srclen: INT,
    dst: LPWSTR,
    dstlen: INT,
) -> INT {
    TRACE!("(0x%04x,0x%08x,%s,%d,%p,%d)\n", lcid, flags, debugstr_wn(src, srclen), srclen, dst, dstlen);

    LCMapStringEx(
        core::ptr::null(),
        flags,
        src,
        srclen,
        dst,
        dstlen,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
    )
}

#[no_mangle]
pub unsafe extern "system" fn LCMapStringA(
    lcid: LCID,
    flags: DWORD,
    src: LPCSTR,
    srclen: INT,
    dst: LPSTR,
    dstlen: INT,
) -> INT {
    let bufW: LPWSTR = core::ptr::addr_of_mut!((*NtCurrentTeb()).StaticUnicodeBuffer) as LPWSTR;
    let mut srcW: LPWSTR = core::ptr::null_mut();
    let mut dstW: LPWSTR = core::ptr::null_mut();
    let mut ret: INT = 0;
    let mut srclenW: INT;
    let mut dstlenW: INT;
    let mut locale_cp: UINT = CP_ACP;

    if src.is_null() || srclen == 0 || dstlen < 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if (flags & LOCALE_USE_CP_ACP) == 0 {
        locale_cp = get_lcid_codepage(lcid);
    }

    srclenW = MultiByteToWideChar(locale_cp, 0, src, srclen, bufW, 260);
    if srclenW != 0 {
        srcW = bufW;
    } else {
        srclenW = MultiByteToWideChar(locale_cp, 0, src, srclen, core::ptr::null_mut(), 0);
        srcW = HeapAlloc(GetProcessHeap(), 0, (srclenW as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR;
        if srcW.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
        MultiByteToWideChar(locale_cp, 0, src, srclen, srcW, srclenW);
    }

    if (flags & LCMAP_SORTKEY) != 0 {
        if src == dst as LPCSTR {
            SetLastError(ERROR_INVALID_FLAGS);
        } else {
            ret = wine_get_sortkey(flags as i32, srcW, srclenW, dst, dstlen);
            if ret == 0 {
                SetLastError(ERROR_INSUFFICIENT_BUFFER);
            } else {
                ret += 1;
            }
        }
    } else if (flags & SORT_STRINGSORT) != 0 {
        SetLastError(ERROR_INVALID_FLAGS);
    } else {
        dstlenW = LCMapStringEx(
            core::ptr::null(),
            flags,
            srcW,
            srclenW,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
        );
        if dstlenW != 0 {
            dstW = HeapAlloc(GetProcessHeap(), 0, (dstlenW as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR;
            if dstW.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            } else {
                LCMapStringEx(
                    core::ptr::null(),
                    flags,
                    srcW,
                    srclenW,
                    dstW,
                    dstlenW,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    0,
                );
                ret = WideCharToMultiByte(
                    locale_cp,
                    0,
                    dstW,
                    dstlenW,
                    dst,
                    dstlen,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                );
                HeapFree(GetProcessHeap(), 0, dstW as LPVOID);
            }
        }
    }

    if srcW != bufW {
        HeapFree(GetProcessHeap(), 0, srcW as LPVOID);
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn FoldStringA(
    mut dwFlags: DWORD,
    src: LPCSTR,
    srclen: INT,
    dst: LPSTR,
    dstlen: INT,
) -> INT {
    let mut ret: INT = 0;
    let mut srclenW: INT = 0;
    let mut srcW: LPWSTR = core::ptr::null_mut();
    let mut dstW: LPWSTR = core::ptr::null_mut();

    if src.is_null()
        || srclen == 0
        || dstlen < 0
        || (dstlen != 0 && dst.is_null())
        || src == dst as LPCSTR
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    srclenW = MultiByteToWideChar(
        CP_ACP,
        if (dwFlags & MAP_COMPOSITE) != 0 { MB_COMPOSITE } else { 0 },
        src,
        srclen,
        core::ptr::null_mut(),
        0,
    );
    srcW = HeapAlloc(GetProcessHeap(), 0, (srclenW as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR;

    if srcW.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
    } else {
        MultiByteToWideChar(
            CP_ACP,
            if (dwFlags & MAP_COMPOSITE) != 0 { MB_COMPOSITE } else { 0 },
            src,
            srclen,
            srcW,
            srclenW,
        );

        dwFlags = (dwFlags & !MAP_PRECOMPOSED) | MAP_FOLDCZONE;

        ret = FoldStringW(dwFlags, srcW, srclenW, core::ptr::null_mut(), 0);
        if ret != 0 && dstlen != 0 {
            dstW = HeapAlloc(GetProcessHeap(), 0, (ret as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR;

            if dstW.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            } else {
                ret = FoldStringW(dwFlags, srcW, srclenW, dstW, ret);
                if WideCharToMultiByte(CP_ACP, 0, dstW, ret, dst, dstlen, core::ptr::null(), core::ptr::null_mut()) == 0 {
                    ret = 0;
                    SetLastError(ERROR_INSUFFICIENT_BUFFER);
                }
                HeapFree(GetProcessHeap(), 0, dstW as LPVOID);
            }
        }
    }

    HeapFree(GetProcessHeap(), 0, srcW as LPVOID);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn FoldStringW(
    dwFlags: DWORD,
    src: LPCWSTR,
    srclen: INT,
    dst: LPWSTR,
    dstlen: INT,
) -> INT {
    let mut ret: i32;
    let current_block: u64;

    match dwFlags & (MAP_COMPOSITE | MAP_PRECOMPOSED | MAP_EXPAND_LIGATURES) {
        0 => {
            if dwFlags != 0 {
                current_block = 7502529970979898288;
            } else {
                current_block = 15375706305260991869;
            }
        }
        96 | 8224 | 8256 => {
            current_block = 15375706305260991869;
        }
        _ => {
            current_block = 7502529970979898288;
        }
    }

    match current_block {
        7502529970979898288 => {}
        _ => {
            SetLastError(ERROR_INVALID_FLAGS);
            return 0;
        }
    }

    if src.is_null()
        || srclen == 0
        || dstlen < 0
        || (dstlen != 0 && dst.is_null())
        || src == dst as LPCWSTR
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    ret = wine_fold_string(dwFlags as i32, src, srclen, dst, dstlen);
    if ret == 0 {
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
    }
    ret
}
#[no_mangle]
pub unsafe extern "system" fn CompareStringW(
    lcid: LCID,
    flags: DWORD,
    str1: PCWSTR,
    len1: INT,
    str2: PCWSTR,
    len2: INT,
) -> INT {
    CompareStringEx(
        core::ptr::null(),
        flags,
        str1,
        len1,
        str2,
        len2,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
    )
}
#[no_mangle]
pub unsafe extern "system" fn CompareStringEx(
    locale: PCWSTR,
    flags: DWORD,
    str1: PCWSTR,
    len1: INT,
    str2: PCWSTR,
    len2: INT,
    version: LPNLSVERSIONINFO,
    reserved: LPVOID,
    lParam: LPARAM,
) -> INT {
    let supported_flags: DWORD = (NORM_IGNORECASE
        | NORM_IGNORENONSPACE
        | NORM_IGNORESYMBOLS
        | SORT_STRINGSORT
        | NORM_IGNOREKANATYPE
        | NORM_IGNOREWIDTH
        | LOCALE_USE_CP_ACP) as DWORD;
    let semistub_flags: DWORD = (NORM_LINGUISTIC_CASING
        | LINGUISTIC_IGNORECASE
        | 0x10000000) as DWORD;
    static mut ONCE: i32 = 0;

    if !version.is_null() {
        FIXME(b"unexpected version parameter\n\0".as_ptr() as PCSTR);
    }
    if !reserved.is_null() {
        FIXME(b"unexpected reserved value\n\0".as_ptr() as PCSTR);
    }
    if lParam != 0 {
        FIXME(b"unexpected lParam\n\0".as_ptr() as PCSTR);
    }

    if str1.is_null() || str2.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return 0;
    }

    if (flags & !(supported_flags | semistub_flags)) != 0 {
        SetLastError(ERROR_INVALID_FLAGS as DWORD);
        return 0;
    }

    if (flags & semistub_flags) != 0 {
        let fresh0 = ONCE;
        ONCE = ONCE + 1;
        if fresh0 == 0 {
            FIXME(b"semi-stub behavior for flag(s) 0x%x\n\0".as_ptr() as PCSTR);
        }
    }

    let mut len1 = len1;
    let mut len2 = len2;

    if len1 < 0 {
        len1 = strlenW(str1) as INT;
    }
    if len2 < 0 {
        len2 = strlenW(str2) as INT;
    }

    let ret = wine_compare_string(flags as i32, str1, len1 as i32, str2, len2 as i32) as INT;

    if ret != 0 {
        return if ret < 0 { CSTR_LESS_THAN as INT } else { CSTR_GREATER_THAN as INT };
    }
    CSTR_EQUAL as INT
}
#[no_mangle]
pub unsafe extern "system" fn CompareStringA(
    lcid: LCID,
    flags: DWORD,
    str1: LPCSTR,
    len1: INT,
    str2: LPCSTR,
    len2: INT,
) -> INT {
    let mut buf1W = core::ptr::addr_of_mut!((*NtCurrentTeb()).StaticUnicodeBuffer) as *mut WCHAR;
    let mut buf2W = buf1W.offset(130);
    let mut str1W: *mut WCHAR;
    let mut str2W: *mut WCHAR;
    let mut len1W = 0i32;
    let mut len2W = 0i32;
    let ret: INT;
    let mut locale_cp = CP_ACP;
    let mut len1 = len1;
    let mut len2 = len2;

    if str1.is_null() || str2.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    if len1 < 0 {
        len1 = strlen(str1) as INT;
    }
    if len2 < 0 {
        len2 = strlen(str2) as INT;
    }

    if (flags & LOCALE_USE_CP_ACP) == 0 {
        locale_cp = get_lcid_codepage(lcid);
    }

    if len1 != 0 {
        if len1 <= 130 {
            len1W = MultiByteToWideChar(locale_cp, 0, str1, len1, buf1W, 130);
        }
        if len1W != 0 {
            str1W = buf1W;
        } else {
            len1W = MultiByteToWideChar(locale_cp, 0, str1, len1, core::ptr::null_mut(), 0);
            str1W = HeapAlloc(GetProcessHeap(), 0, (len1W as usize) * core::mem::size_of::<WCHAR>()) as *mut WCHAR;
            if str1W.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                return 0;
            }
            MultiByteToWideChar(locale_cp, 0, str1, len1, str1W, len1W);
        }
    } else {
        len1W = 0;
        str1W = buf1W;
    }

    if len2 != 0 {
        if len2 <= 130 {
            len2W = MultiByteToWideChar(locale_cp, 0, str2, len2, buf2W, 130);
        }
        if len2W != 0 {
            str2W = buf2W;
        } else {
            len2W = MultiByteToWideChar(locale_cp, 0, str2, len2, core::ptr::null_mut(), 0);
            str2W = HeapAlloc(GetProcessHeap(), 0, (len2W as usize) * core::mem::size_of::<WCHAR>()) as *mut WCHAR;
            if str2W.is_null() {
                if str1W != buf1W {
                    HeapFree(GetProcessHeap(), 0, str1W as LPVOID);
                }
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                return 0;
            }
            MultiByteToWideChar(locale_cp, 0, str2, len2, str2W, len2W);
        }
    } else {
        len2W = 0;
        str2W = buf2W;
    }

    ret = CompareStringEx(core::ptr::null(), flags, str1W, len1W, str2W, len2W, core::ptr::null_mut(), core::ptr::null_mut(), 0);

    if str1W != buf1W {
        HeapFree(GetProcessHeap(), 0, str1W as LPVOID);
    }
    if str2W != buf2W {
        HeapFree(GetProcessHeap(), 0, str2W as LPVOID);
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn CompareStringOrdinal(
    str1: *const u16,
    len1: i32,
    str2: *const u16,
    len2: i32,
    ignore_case: u8,
) -> i32 {

    let mut len1 = len1;
    let mut len2 = len2;

    if str1.is_null() || str2.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    if len1 < 0 {
        len1 = strlenW(str1);
    }
    if len2 < 0 {
        len2 = strlenW(str2);
    }

    let ret = RtlCompareUnicodeStrings(str1, len1 as SIZE_T, str2, len2 as SIZE_T, ignore_case);
    if ret < 0 {
        return CSTR_LESS_THAN as i32;
    }
    if ret > 0 {
        return CSTR_GREATER_THAN as i32;
    }
    CSTR_EQUAL as i32
}
#[no_mangle]
pub unsafe extern "C" fn NLS_RegOpenKey(hRootKey: HANDLE, szKeyName: LPCWSTR) -> HANDLE {
    let mut keyName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut attr: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut hkey: HANDLE = core::ptr::null_mut();

    RtlInitUnicodeString(core::ptr::addr_of_mut!(keyName), szKeyName);
    
    attr.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    attr.RootDirectory = hRootKey;
    attr.Attributes = OBJ_CASE_INSENSITIVE;
    attr.ObjectName = core::ptr::addr_of_mut!(keyName);
    attr.SecurityDescriptor = core::ptr::null_mut();
    attr.SecurityQualityOfService = core::ptr::null_mut();

    if NtOpenKey(core::ptr::addr_of_mut!(hkey), KEY_READ, core::ptr::addr_of_mut!(attr)) != STATUS_SUCCESS {
        hkey = core::ptr::null_mut();
    }

    hkey
}
#[no_mangle]
pub unsafe extern "C" fn NLS_RegEnumValue(
    hKey: HANDLE,
    ulIndex: UINT,
    szValueName: LPWSTR,
    valueNameSize: ULONG,
    szValueData: LPWSTR,
    valueDataSize: ULONG,
) -> BOOL {
    let mut buffer: [u8; 80] = [0; 80];
    let info: *mut KEY_VALUE_FULL_INFORMATION = buffer.as_mut_ptr() as *mut KEY_VALUE_FULL_INFORMATION;
    let mut dwLen: DWORD = 0;

    if NtEnumerateValueKey(
        hKey,
        ulIndex as ULONG,
        KeyValueFullInformation,
        buffer.as_mut_ptr() as PVOID,
        core::mem::size_of::<[u8; 80]>() as ULONG,
        core::ptr::addr_of_mut!(dwLen),
    ) != STATUS_SUCCESS
        || (*info).NameLength > valueNameSize
        || (*info).DataLength > valueDataSize
    {
        return 0;
    }

    core::ptr::copy_nonoverlapping(
        core::ptr::addr_of_mut!((*info).Name) as *const u8,
        szValueName as *mut u8,
        (*info).NameLength as usize,
    );
    *szValueName.offset(((*info).NameLength as usize / core::mem::size_of::<WCHAR>()) as isize) = 0;

    core::ptr::copy_nonoverlapping(
        buffer.as_ptr().offset((*info).DataOffset as isize),
        szValueData as *mut u8,
        (*info).DataLength as usize,
    );
    *szValueData.offset(((*info).DataLength as usize / core::mem::size_of::<WCHAR>()) as isize) = 0;

    1
}

unsafe fn NLS_RegGetDword(hKey: HANDLE, szValueName: LPCWSTR, lpVal: *mut DWORD) -> BOOL {
    let mut buffer: [BYTE; 128] = [0; 128];
    let info: *const KEY_VALUE_PARTIAL_INFORMATION = 
        buffer.as_mut_ptr() as *const KEY_VALUE_PARTIAL_INFORMATION;
    let mut dwSize: DWORD = core::mem::size_of::<[BYTE; 128]>() as DWORD;
    let mut valueName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    RtlInitUnicodeString(core::ptr::addr_of_mut!(valueName), szValueName);

    if NtQueryValueKey(
        hKey,
        core::ptr::addr_of_mut!(valueName),
        KeyValuePartialInformation,
        buffer.as_mut_ptr() as PVOID,
        dwSize,
        core::ptr::addr_of_mut!(dwSize),
    ) == STATUS_SUCCESS
        && (*info).DataLength == core::mem::size_of::<DWORD>() as DWORD
    {
        core::ptr::copy_nonoverlapping(
            (*info).Data.as_ptr(),
            lpVal as *mut BYTE,
            core::mem::size_of::<DWORD>(),
        );
        return 1;
    }

    0
}

unsafe fn NLS_GetLanguageGroupName(lgrpid: LGRPID, szName: LPWSTR, nameSize: ULONG) -> BOOL {
    let szResourceName: LPCWSTR = ((((lgrpid as u32).wrapping_add(0x2000) >> 4).wrapping_add(1)) as usize) as LPCWSTR;
    let mut bRet: BOOL = 0;

    let mut langId: LANGID = GetSystemDefaultLangID();

    if (langId >> 10) == SUBLANG_NEUTRAL as u16 {
        langId = get_default_sublang(langId);
    }

    let hResource = FindResourceExW(kernel32_handle, RT_STRING as LPCWSTR, szResourceName, langId);

    if !hResource.is_null() {
        let hResDir = LoadResource(kernel32_handle, hResource);

        if !hResDir.is_null() {
            let iResourceIndex: ULONG = lgrpid & 0xf;
            let mut lpResEntry: *const u16 = LockResource(hResDir) as *const u16;

            for _ in 0..iResourceIndex {
                lpResEntry = lpResEntry.offset((*lpResEntry as i32 + 1) as isize);
            }

            if (*lpResEntry as ULONG) < nameSize {
                core::ptr::copy_nonoverlapping(
                    lpResEntry.offset(1),
                    szName,
                    *lpResEntry as usize,
                );
                *szName.offset(*lpResEntry as isize) = 0;
                bRet = 1;
            }
        }
        FreeResource(hResource as HGLOBAL);
    }
    bRet
}

unsafe fn NLS_EnumSystemLanguageGroups(lpProcs: *mut ENUMLANGUAGEGROUP_CALLBACKS) -> BOOL {
    let mut szNumber: [WCHAR; 10] = [0; 10];
    let mut szValue: [WCHAR; 4] = [0; 4];
    let mut hKey: HANDLE;
    let mut bContinue: BOOL = 1;
    let mut ulIndex: ULONG = 0;

    if lpProcs.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    match (*lpProcs).dwFlags {
        0 => {
            (*lpProcs).dwFlags = LGRPID_INSTALLED;
        }
        LGRPID_INSTALLED | LGRPID_SUPPORTED => {}
        _ => {
            SetLastError(ERROR_INVALID_FLAGS);
            return 0;
        }
    }

    hKey = NLS_RegOpenKey(core::ptr::null_mut(), (*(&raw const szLangGroupsKeyName)).as_ptr());

    if hKey.is_null() {
        FIXME(b"NLS registry key not found. Please apply the default registry file 'wine.inf'\n\0".as_ptr() as PCSTR);
    }

    while bContinue != 0 {
        if NLS_RegEnumValue(
            hKey,
            ulIndex,
            szNumber.as_mut_ptr(),
            core::mem::size_of::<[WCHAR; 10]>() as ULONG,
            szValue.as_mut_ptr(),
            core::mem::size_of::<[WCHAR; 4]>() as ULONG,
        ) != 0
        {
            let bInstalled: BOOL = (szValue[0] == b'1' as WCHAR) as BOOL;
            let lgrpid: LGRPID = strtoulW(szNumber.as_mut_ptr(), core::ptr::null_mut(), 16);

            TRACE!("grpid %s (%sinstalled)\n", debugstr_w(szNumber.as_mut_ptr()), if bInstalled != 0 { b"\0".as_ptr() } else { b"not \0".as_ptr() });

            if (*lpProcs).dwFlags == LGRPID_SUPPORTED || bInstalled != 0 {
                let mut szGrpName: [WCHAR; 48] = [0; 48];

                if NLS_GetLanguageGroupName(
                    lgrpid,
                    szGrpName.as_mut_ptr(),
                    (core::mem::size_of::<[WCHAR; 48]>() / core::mem::size_of::<WCHAR>()) as ULONG,
                ) == 0
                {
                    szGrpName[0] = 0;
                }

                if (*lpProcs).procW.is_some() {
                    bContinue = (*lpProcs).procW.unwrap()(
                        lgrpid,
                        szNumber.as_mut_ptr(),
                        szGrpName.as_mut_ptr(),
                        (*lpProcs).dwFlags,
                        (*lpProcs).lParam,
                    );
                } else {
                    let mut szNumberA: [i8; 10] = [0; 10];
                    let mut szGrpNameA: [i8; 48] = [0; 48];

                    WideCharToMultiByte(
                        CP_ACP,
                        0,
                        szNumber.as_mut_ptr(),
                        -1,
                        szNumberA.as_mut_ptr(),
                        core::mem::size_of::<[i8; 10]>() as i32,
                        core::ptr::null(),
                        core::ptr::null_mut(),
                    );
                    WideCharToMultiByte(
                        CP_ACP,
                        0,
                        szGrpName.as_mut_ptr(),
                        -1,
                        szGrpNameA.as_mut_ptr(),
                        core::mem::size_of::<[i8; 48]>() as i32,
                        core::ptr::null(),
                        core::ptr::null_mut(),
                    );

                    bContinue = (*lpProcs).procA.unwrap()(
                        lgrpid,
                        szNumberA.as_mut_ptr(),
                        szGrpNameA.as_mut_ptr(),
                        (*lpProcs).dwFlags,
                        (*lpProcs).lParam,
                    );
                }
            }

            ulIndex = ulIndex.wrapping_add(1);
        } else {
            bContinue = 0;
        }

        if bContinue == 0 {
            break;
        }
    }

    if !hKey.is_null() {
        NtClose(hKey);
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn EnumSystemLanguageGroupsA(
    pLangGrpEnumProc: LANGUAGEGROUP_ENUMPROCA,
    dwFlags: DWORD,
    lParam: LONG_PTR,
) -> BOOL {
    let mut procs = ENUMLANGUAGEGROUP_CALLBACKS {
        procA: pLangGrpEnumProc,
        procW: None,
        dwFlags: dwFlags,
        lParam: lParam,
    };

    TRACE!("(%p,0x%08X,0x%08lX)\n", pLangGrpEnumProc, dwFlags, lParam);

    NLS_EnumSystemLanguageGroups(if pLangGrpEnumProc.is_some() {
        core::ptr::addr_of_mut!(procs)
    } else {
        core::ptr::null_mut()
    })
}

#[no_mangle]
pub unsafe extern "system" fn EnumSystemLanguageGroupsW(
    pLangGrpEnumProc: LANGUAGEGROUP_ENUMPROCW,
    dwFlags: DWORD,
    lParam: LONG_PTR,
) -> BOOL {
    let mut procs: ENUMLANGUAGEGROUP_CALLBACKS = ENUMLANGUAGEGROUP_CALLBACKS {
        procA: None,
        procW: None,
        dwFlags: 0,
        lParam: 0,
    };

    TRACE!("(%p,0x%08X,0x%08lX)\n", pLangGrpEnumProc, dwFlags, lParam);

    procs.procA = None;
    procs.procW = pLangGrpEnumProc;
    procs.dwFlags = dwFlags;
    procs.lParam = lParam;

    return NLS_EnumSystemLanguageGroups(if pLangGrpEnumProc.is_some() {
        core::ptr::addr_of_mut!(procs)
    } else {
        core::ptr::null_mut()
    });
}

#[no_mangle]
pub unsafe extern "system" fn IsValidLanguageGroup(lgrpid: LGRPID, dwFlags: DWORD) -> BOOL {
    static szFormat: [WCHAR; 3] = ['%' as u16, 'x' as u16, '\0' as u16];
    let mut szValueName: [WCHAR; 16] = [0; 16];
    let mut szValue: [WCHAR; 2] = [0; 2];
    let mut bSupported: BOOL = 0;
    let mut bInstalled: BOOL = 0;
    let mut hKey: HANDLE = core::ptr::null_mut();

    match dwFlags {
        LGRPID_INSTALLED | LGRPID_SUPPORTED => {
            hKey = NLS_RegOpenKey(core::ptr::null_mut(), (*(&raw const szLangGroupsKeyName)).as_ptr());

            sprintfW(szValueName.as_mut_ptr(), szFormat.as_ptr(), lgrpid);

            if NLS_RegGetDword(hKey, szValueName.as_ptr(), szValue.as_mut_ptr() as *mut DWORD) != 0 {
                bSupported = 1;

                if szValue[0] == '1' as u16 {
                    bInstalled = 1;
                }
            }

            if !hKey.is_null() {
                NtClose(hKey);
            }
        }
        _ => {}
    }

    if (dwFlags == LGRPID_SUPPORTED && bSupported != 0) ||
       (dwFlags == LGRPID_INSTALLED && bInstalled != 0) {
        return 1;
    }

    0
}

unsafe fn NLS_EnumLanguageGroupLocales(lpProcs: *mut ENUMLANGUAGEGROUPLOCALE_CALLBACKS) -> BOOL {
    static szAlternateSortsKeyName: [WCHAR; 16] = [
        'A' as u16, 'l' as u16, 't' as u16, 'e' as u16, 'r' as u16, 'n' as u16, 'a' as u16,
        't' as u16, 'e' as u16, ' ' as u16, 'S' as u16, 'o' as u16, 'r' as u16, 't' as u16,
        's' as u16, '\0' as u16,
    ];
    
    let mut szNumber: [WCHAR; 10] = [0; 10];
    let mut szValue: [WCHAR; 4] = [0; 4];
    let mut hKey: HANDLE;
    let mut bContinue: BOOL = 1;
    let mut bAlternate: BOOL = 0;
    let mut lgrpid: LGRPID;
    let mut ulIndex: ULONG = 1;

    if lpProcs.is_null() || (*lpProcs).lgrpid == 0 || (*lpProcs).lgrpid > LGRPID_ARMENIAN {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if (*lpProcs).dwFlags != 0 {
        SetLastError(ERROR_INVALID_FLAGS);
        return 0;
    }

    hKey = NLS_RegOpenKey(core::ptr::null_mut(), (*(&raw const szLocaleKeyName)).as_ptr());

    if hKey.is_null() {
        WARN!("NLS registry key not found. Please apply the default registry file 'wine.inf'\n");
    }

    while bContinue != 0 {
        if NLS_RegEnumValue(
            hKey,
            ulIndex,
            szNumber.as_mut_ptr(),
            core::mem::size_of::<[WCHAR; 10]>() as ULONG,
            szValue.as_mut_ptr(),
            core::mem::size_of::<[WCHAR; 4]>() as ULONG,
        ) != 0
        {
            lgrpid = strtoulW(szValue.as_ptr(), core::ptr::null_mut(), 16);

            TRACE!("lcid %s, grpid %d (%smatched)\n", debugstr_w(szNumber.as_ptr()), lgrpid, if lgrpid == (*lpProcs).lgrpid { b"\0".as_ptr() } else { b"not \0".as_ptr() });

            if lgrpid == (*lpProcs).lgrpid {
                let lcid: LCID = strtoulW(szNumber.as_ptr(), core::ptr::null_mut(), 16);

                if (*lpProcs).procW.is_some() {
                    bContinue = (*lpProcs).procW.unwrap()(
                        lgrpid,
                        lcid,
                        szNumber.as_mut_ptr(),
                        (*lpProcs).lParam,
                    );
                } else {
                    let mut szNumberA: [i8; 10] = [0; 10];

                    WideCharToMultiByte(
                        CP_ACP,
                        0,
                        szNumber.as_ptr(),
                        -1,
                        szNumberA.as_mut_ptr(),
                        core::mem::size_of::<[i8; 10]>() as i32,
                        core::ptr::null(),
                        core::ptr::null_mut(),
                    );

                    bContinue = (*lpProcs).procA.unwrap()(
                        lgrpid,
                        lcid,
                        szNumberA.as_mut_ptr(),
                        (*lpProcs).lParam,
                    );
                }
            }

            ulIndex = ulIndex.wrapping_add(1);
        } else {
            if bAlternate == 0 {
                hKey = NLS_RegOpenKey(hKey, szAlternateSortsKeyName.as_ptr());
                bAlternate = 1;
                ulIndex = 0;
            } else {
                bContinue = 0;
            }
        }

        if bContinue == 0 {
            break;
        }
    }

    if !hKey.is_null() {
        NtClose(hKey);
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn EnumLanguageGroupLocalesA(
    pLangGrpLcEnumProc: LANGGROUPLOCALE_ENUMPROCA,
    lgrpid: LGRPID,
    dwFlags: DWORD,
    lParam: LONG_PTR,
) -> BOOL {
    let mut callbacks = ENUMLANGUAGEGROUPLOCALE_CALLBACKS {
        procA: pLangGrpLcEnumProc,
        procW: None,
        dwFlags,
        lgrpid,
        lParam,
    };

    TRACE!("(%p,0x%08X,0x%08X,0x%08lX)\n", pLangGrpLcEnumProc, lgrpid, dwFlags, lParam);

    NLS_EnumLanguageGroupLocales(
        if pLangGrpLcEnumProc.is_some() {
            core::ptr::addr_of_mut!(callbacks)
        } else {
            core::ptr::null_mut()
        }
    )
}

#[no_mangle]
pub unsafe extern "system" fn EnumLanguageGroupLocalesW(
    pLangGrpLcEnumProc: LANGGROUPLOCALE_ENUMPROCW,
    lgrpid: LGRPID,
    dwFlags: DWORD,
    lParam: LONG_PTR,
) -> BOOL {
    let mut callbacks = ENUMLANGUAGEGROUPLOCALE_CALLBACKS {
        procA: None,
        procW: None,
        dwFlags: 0,
        lgrpid: 0,
        lParam: 0,
    };

    TRACE!("(%p,0x%08X,0x%08X,0x%08lX)\n", pLangGrpLcEnumProc, lgrpid, dwFlags, lParam);

    callbacks.procA = None;
    callbacks.procW = pLangGrpLcEnumProc;
    callbacks.dwFlags = dwFlags;
    callbacks.lgrpid = lgrpid;
    callbacks.lParam = lParam;

    NLS_EnumLanguageGroupLocales(
        if pLangGrpLcEnumProc.is_some() {
            core::ptr::addr_of_mut!(callbacks)
        } else {
            core::ptr::null_mut()
        }
    )
}

#[no_mangle]
pub unsafe extern "system" fn InvalidateNLSCache() -> BOOL {
    
    JapaneseEra_ClearCache();
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetUserGeoID(GeoClass: GEOCLASS) -> GEOID {
    let mut ret: GEOID = GEOID_NOT_AVAILABLE as GEOID;
    static mut geoW: [WCHAR; 4] = [
        b'G' as WCHAR,
        b'e' as WCHAR,
        b'o' as WCHAR,
        0,
    ];
    static mut nationW: [WCHAR; 7] = [
        b'N' as WCHAR,
        b'a' as WCHAR,
        b't' as WCHAR,
        b'i' as WCHAR,
        b'o' as WCHAR,
        b'n' as WCHAR,
        0,
    ];
    let mut bufferW: [WCHAR; 40] = [0; 40];
    let mut end: PWSTR = core::ptr::null_mut();
    let mut count: DWORD;
    let mut hkey: HANDLE;
    let mut hSubkey: HANDLE = core::ptr::null_mut();
    let mut keyW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let info: *const KEY_VALUE_PARTIAL_INFORMATION = bufferW.as_mut_ptr() as *const KEY_VALUE_PARTIAL_INFORMATION;
    
    RtlInitUnicodeString(core::ptr::addr_of_mut!(keyW), (*(&raw const nationW)).as_ptr());
    count = core::mem::size_of::<[WCHAR; 40]>() as DWORD;

    hkey = create_registry_key();
    if hkey.is_null() {
        return ret;
    }

    match GeoClass {
        GEOCLASS_NATION => {
            hSubkey = NLS_RegOpenKey(hkey, (*(&raw const geoW)).as_ptr());
            if !hSubkey.is_null() {
                if NtQueryValueKey(
                    hSubkey,
                    core::ptr::addr_of_mut!(keyW),
                    KeyValuePartialInformation,
                    bufferW.as_mut_ptr() as LPVOID,
                    count,
                    core::ptr::addr_of_mut!(count),
                ) == STATUS_SUCCESS && (*info).DataLength != 0
                {
                    ret = wcstol(
                        (*info).Data.as_ptr() as LPCWSTR,
                        core::ptr::addr_of_mut!(end),
                        10,
                    ) as GEOID;
                }
            }
        }
        GEOCLASS_REGION => {
            DbgPrintEx(
                gDebugChannel,
                DPFLTR_WARNING_LEVEL,
                b"(%s:%d) GEOCLASS_REGION not handled yet\n\0".as_ptr() as PCSTR,
                b"dll/win32/kernel32/winnls/string/locale.c\0".as_ptr() as PCSTR,
                4866,
            );
        }
        _ => {}
    }

    NtClose(hkey);
    if !hSubkey.is_null() {
        NtClose(hSubkey);
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn SetUserGeoID(GeoID: GEOID) -> BOOL {
    static geoW: [WCHAR; 4] = [b'G' as u16, b'e' as u16, b'o' as u16, 0];
    static nationW: [WCHAR; 7] = [
        b'N' as u16,
        b'a' as u16,
        b't' as u16,
        b'i' as u16,
        b'o' as u16,
        b'n' as u16,
        0,
    ];
    static formatW: [WCHAR; 3] = [b'%' as u16, b'i' as u16, 0];

    let mut nameW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut keyW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut bufferW: [WCHAR; 10] = [0; 10];
    let mut attr: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut hkey: HANDLE;

    hkey = create_registry_key();
    if hkey.is_null() {
        return 0;
    }

    attr.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    attr.RootDirectory = hkey;
    attr.ObjectName = core::ptr::addr_of_mut!(nameW);
    attr.Attributes = 0;
    attr.SecurityDescriptor = core::ptr::null_mut();
    attr.SecurityQualityOfService = core::ptr::null_mut();

    RtlInitUnicodeString(core::ptr::addr_of_mut!(nameW), geoW.as_ptr());
    RtlInitUnicodeString(core::ptr::addr_of_mut!(keyW), nationW.as_ptr());

    if NtCreateKey(
        core::ptr::addr_of_mut!(hkey),
        KEY_ALL_ACCESS,
        core::ptr::addr_of_mut!(attr),
        0,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
    ) != STATUS_SUCCESS
    {
        NtClose(attr.RootDirectory);
        return 0;
    }

    _snwprintf(
        bufferW.as_mut_ptr(),
        bufferW.len(),
        formatW.as_ptr(),
        GeoID,
    );

    NtSetValueKey(
        hkey,
        core::ptr::addr_of_mut!(keyW),
        0,
        REG_SZ,
        bufferW.as_mut_ptr() as *mut core::ffi::c_void,
        ((wcslen(bufferW.as_mut_ptr()) + 1) * core::mem::size_of::<WCHAR>()) as u32,
    );

    NtClose(attr.RootDirectory);
    NtClose(hkey);
    1
}
unsafe extern "system" fn enum_uilang_proc_a(
    _hModule: HMODULE,
    _type: LPCSTR,
    _name: LPCSTR,
    LangID: WORD,
    lParam: LONG_PTR,
) -> BOOL {
    let enum_uilang = lParam as *mut ENUM_UILANG_CALLBACK;
    let mut buf: [u8; 20] = [0; 20];

    sprintf(
        buf.as_mut_ptr() as *mut i8,
        b"%08x\0".as_ptr() as *const i8,
        LangID as UINT,
    );

    (*enum_uilang).u.procA.expect("non-null function pointer")(
        buf.as_mut_ptr() as LPSTR,
        (*enum_uilang).param,
    )
}
unsafe extern "system" fn enum_uilang_proc_w(
    _hModule: HMODULE,
    _type: LPCWSTR,
    _name: LPCWSTR,
    LangID: WORD,
    lParam: LONG_PTR,
) -> BOOL {
    static formatW: [WCHAR; 5] = [
        b'%' as u16,
        b'0' as u16,
        b'8' as u16,
        b'x' as u16,
        0,
    ];
    let enum_uilang = lParam as *mut ENUM_UILANG_CALLBACK;
    let mut buf: [WCHAR; 20] = [0; 20];

    _snwprintf(
        buf.as_mut_ptr(),
        MAXLONG as usize,
        formatW.as_ptr(),
        LangID as UINT,
    );

    ((*enum_uilang).u.procW.unwrap())(buf.as_mut_ptr(), (*enum_uilang).param)
}

#[no_mangle]
pub unsafe extern "system" fn EnumUILanguagesA(
    pUILangEnumProc: UILANGUAGE_ENUMPROCA,
    dwFlags: DWORD,
    lParam: LONG_PTR,
) -> BOOL {
    let mut enum_uilang = ENUM_UILANG_CALLBACK {
        u: ENUM_UILANG_CALLBACK_u { procA: None },
        flags: 0,
        param: 0,
    };

    TRACE!("%p, %x, %lx\n", pUILangEnumProc, dwFlags, lParam);

    if pUILangEnumProc.is_none() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    if dwFlags != 0 {
        SetLastError(ERROR_INVALID_FLAGS);
        return 0;
    }

    enum_uilang.u.procA = pUILangEnumProc;
    enum_uilang.flags = dwFlags;
    enum_uilang.param = lParam;

    EnumResourceLanguagesA(
        kernel32_handle,
        ((6 as u64) & 0xffffu64) as WORD as ULONG_PTR as LPSTR as LPCSTR,
        LOCALE_ILANGUAGE as LPCSTR,
        as_enumreslang_a(enum_uilang_proc_a),
        core::ptr::addr_of_mut!(enum_uilang) as LONG_PTR,
    );
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn EnumUILanguagesW(
    pUILangEnumProc: UILANGUAGE_ENUMPROCW,
    dwFlags: DWORD,
    lParam: LONG_PTR,
) -> BOOL {
    let mut enum_uilang = ENUM_UILANG_CALLBACK {
        u: C2RustUnnamed_10 { procA: None },
        flags: 0,
        param: 0,
    };

    TRACE!("%p, %x, %lx\n", pUILangEnumProc, dwFlags, lParam);

    if pUILangEnumProc.is_none() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    if dwFlags != 0 {
        SetLastError(ERROR_INVALID_FLAGS);
        return 0;
    }

    enum_uilang.u.procW = pUILangEnumProc;
    enum_uilang.flags = dwFlags;
    enum_uilang.param = lParam;

    EnumResourceLanguagesW(
        kernel32_handle,
        RT_STRING as LPCWSTR,
        LOCALE_ILANGUAGE as LPCWSTR,
        as_enumreslang_w(enum_uilang_proc_w),
        core::ptr::addr_of_mut!(enum_uilang) as LONG_PTR,
    );
    return 1;
}

unsafe fn NLS_EnumSystemCodePages(lpProcs: *mut ENUMSYSTEMCODEPAGES_CALLBACKS) -> BOOL {
    let mut szNumber: [WCHAR; 6] = [0; 6];
    let mut szValue: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut hKey: HANDLE;
    let mut bContinue: BOOL = 1;
    let mut ulIndex: ULONG = 0;

    if lpProcs.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    match (*lpProcs).dwFlags {
        CP_INSTALLED | CP_SUPPORTED => {}
        _ => {
            SetLastError(ERROR_INVALID_FLAGS);
            return 0;
        }
    }

    hKey = NLS_RegOpenKey(
        core::ptr::null_mut(),
        w!("\\Registry\\Machine\\SYSTEM\\CurrentControlSet\\Control\\NLS\\CodePage\0").as_ptr() as PCWSTR,
    );
    if hKey.is_null() {
        WARN!("NLS_RegOpenKey() failed\n");
        return 0;
    }

    while bContinue != 0 {
        if NLS_RegEnumValue(
            hKey,
            ulIndex,
            szNumber.as_mut_ptr(),
            core::mem::size_of::<[WCHAR; 6]>() as ULONG,
            szValue.as_mut_ptr(),
            core::mem::size_of_val(&szValue) as ULONG,
        ) != 0
        {
            if (*lpProcs).dwFlags == CP_SUPPORTED
                || ((*lpProcs).dwFlags == CP_INSTALLED && wcslen(szValue.as_mut_ptr()) > 2)
            {
                if (*lpProcs).procW.is_some() {
                    bContinue = (*lpProcs).procW.unwrap()(szNumber.as_mut_ptr());
                } else {
                    let mut szNumberA: [i8; 6] = [0; 6];

                    WideCharToMultiByte(
                        CP_ACP,
                        0,
                        szNumber.as_ptr(),
                        -1,
                        szNumberA.as_mut_ptr(),
                        core::mem::size_of::<[i8; 6]>() as i32,
                        core::ptr::null(),
                        core::ptr::null_mut(),
                    );
                    bContinue = (*lpProcs).procA.unwrap()(szNumberA.as_mut_ptr());
                }
            }

            ulIndex = ulIndex.wrapping_add(1);
        } else {
            bContinue = 0;
        }

        if bContinue == 0 {
            break;
        }
    }

    if !hKey.is_null() {
        NtClose(hKey);
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn EnumSystemCodePagesW(
    lpCodePageEnumProc: CODEPAGE_ENUMPROCW,
    dwFlags: DWORD,
) -> BOOL {
    let mut procs: ENUMSYSTEMCODEPAGES_CALLBACKS = ENUMSYSTEMCODEPAGES_CALLBACKS {
        procA: None,
        procW: None,
        dwFlags: 0,
    };

    TRACE!("(%p,0x%08X)\n", lpCodePageEnumProc, dwFlags);

    procs.procA = None;
    procs.procW = lpCodePageEnumProc;
    procs.dwFlags = dwFlags;

    NLS_EnumSystemCodePages(if lpCodePageEnumProc.is_some() {
        core::ptr::addr_of_mut!(procs)
    } else {
        core::ptr::null_mut()
    })
}

#[no_mangle]
pub unsafe extern "system" fn EnumSystemCodePagesA(
    lpCodePageEnumProc: CODEPAGE_ENUMPROCA,
    dwFlags: DWORD,
) -> BOOL {
    let mut procs = ENUMSYSTEMCODEPAGES_CALLBACKS {
        procA: lpCodePageEnumProc,
        procW: None,
        dwFlags: dwFlags,
    };

    TRACE!("(%p,0x%08X)\n", lpCodePageEnumProc, dwFlags);

    NLS_EnumSystemCodePages(if lpCodePageEnumProc.is_none() {
        core::ptr::null_mut()
    } else {
        core::ptr::addr_of_mut!(procs)
    })
}

unsafe fn NLS_GetGeoFriendlyName(
    mut Location: GEOID,
    szFriendlyName: LPWSTR,
    cchData: i32,
    lang: LANGID,
) -> i32 {
    Location = (Location as u32).wrapping_add(NLSRC_OFFSET as u32) as GEOID;
    Location &= 0xFFFF;

    if cchData == 0 {
        return GetLocalisedText(
            Location as UINT,
            core::ptr::null_mut(),
            0,
            lang,
        ) as i32;
    }

    if GetLocalisedText(Location as UINT, szFriendlyName, cchData as UINT, lang) != 0 {
        return (strlenW(szFriendlyName).wrapping_add(1)) as i32;
    }

    0
}

unsafe fn get_geoinfo_dataptr(geoid: GEOID) -> *const geoinfo_t {
    let mut min: i32 = 0;
    let mut max: i32 = (core::mem::size_of::<[geoinfo_t; 299]>() / core::mem::size_of::<geoinfo_t>() - 1) as i32;

    while min <= max {
        let n: i32 = (min + max) / 2;
        let ptr: *const geoinfo_t = (*(&raw const geoinfodata)).as_ptr().offset(n as isize);

        if geoid == (*ptr).id {
            return if (*ptr).iso2W[0] != 0 {
                ptr
            } else {
                core::ptr::null()
            };
        }

        if (*ptr).id > geoid {
            max = n - 1;
        } else {
            min = n + 1;
        }
    }

    core::ptr::null()
}

#[no_mangle]
pub unsafe extern "system" fn GetGeoInfoW(
    geoid: GEOID,
    geotype: GEOTYPE,
    data: LPWSTR,
    data_len: i32,
    lang: LANGID,
) -> i32 {
    let ptr: *const geoinfo_t;
    let mut str: *const WCHAR = core::ptr::null::<WCHAR>();
    let mut buffW: [WCHAR; 12] = [0; 12];
    let mut val: i32 = 0;
    let len: i32;

    TRACE!("(%s:%d) %d %d %p %d %d\n", geoid, geotype, data, data_len, lang as i32);

    ptr = get_geoinfo_dataptr(geoid);
    if ptr.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    match geotype {
        8 => {
            return NLS_GetGeoFriendlyName(geoid, data, data_len, lang);
        }
        1 => {
            val = geoid as i32;
        }
        12 => {
            val = (*ptr).uncode as i32;
        }
        13 => {
            val = (*ptr).parent as i32;
        }
        4 | 5 => {
            str = if geotype == 4 {
                core::ptr::addr_of!((*ptr).iso2W) as *const WCHAR
            } else {
                core::ptr::addr_of!((*ptr).iso3W) as *const WCHAR
            };
        }
        6 | 7 | 9 | 10 | 11 | 2 | 3 => {
            FIXME(b"(%s:%d) type %d is not supported\n\0".as_ptr() as PCSTR, geotype);
            SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
            return 0;
        }
        _ => {
            WARN!("(%s:%d) unrecognized type %d\n", geotype);
            SetLastError(ERROR_INVALID_FLAGS);
            return 0;
        }
    }

    if val != 0 {
        static fmtW: [WCHAR; 3] = [b'%' as u16, b'd' as u16, 0];
        sprintfW(
            buffW.as_mut_ptr(),
            fmtW.as_ptr(),
            val,
        );
        str = buffW.as_mut_ptr();
    }

    len = (strlenW(str) + 1) as i32;
    if data.is_null() || data_len == 0 {
        return len;
    }

    memcpy(
        data as *mut u8,
        str as *const u8,
        (min(len, data_len) as usize) * core::mem::size_of::<WCHAR>(),
    );
    if data_len < len {
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
    }
    return if data_len < len { 0 } else { len };
}

#[no_mangle]
pub unsafe extern "system" fn GetGeoInfoA(
    geoid: GEOID,
    geotype: GEOTYPE,
    data: LPSTR,
    data_len: ::core::ffi::c_int,
    lang: LANGID,
) -> ::core::ffi::c_int {
    let mut buffW: *mut WCHAR;
    let mut len: ::core::ffi::c_int;

    TRACE!("%d %d %p %d %d\n", geoid, geotype, data, data_len, lang as i32);

    len = GetGeoInfoW(geoid, geotype, core::ptr::null_mut(), 0, lang);
    if len == 0 {
        return 0;
    }

    buffW = HeapAlloc(
        GetProcessHeap(),
        0,
        (len as usize) * core::mem::size_of::<WCHAR>(),
    ) as *mut WCHAR;
    if buffW.is_null() {
        return 0;
    }

    GetGeoInfoW(geoid, geotype, buffW, len, lang);
    len = WideCharToMultiByte(
        CP_ACP,
        0,
        buffW,
        -1,
        core::ptr::null_mut(),
        0,
        core::ptr::null(),
        core::ptr::null_mut(),
    );
    if data.is_null() || data_len == 0 {
        HeapFree(GetProcessHeap(), 0, buffW as LPVOID);
        return len;
    }

    len = WideCharToMultiByte(
        CP_ACP,
        0,
        buffW,
        -1,
        data,
        data_len,
        core::ptr::null(),
        core::ptr::null_mut(),
    );
    HeapFree(GetProcessHeap(), 0, buffW as LPVOID);

    if data_len < len {
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
    }
    return if data_len < len { 0 } else { len };
}

#[no_mangle]
pub unsafe extern "system" fn EnumSystemGeoID(
    geoclass: GEOCLASS,
    parent: GEOID,
    enumproc: GEO_ENUMPROC,
) -> BOOL {
    let mut i: INT;

    TRACE!("(%d, %d, %p)\n", geoclass, parent, enumproc);

    if enumproc.is_none() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if geoclass != GEOCLASS_NATION && geoclass != GEOCLASS_REGION {
        SetLastError(ERROR_INVALID_FLAGS);
        return 0;
    }

    i = 0;
    while (i as usize) < core::mem::size_of_val(&*(&raw const geoinfodata)) / core::mem::size_of::<geoinfo_t>() {
        let ptr: *const geoinfo_t = &geoinfodata[i as usize];

        if geoclass == GEOCLASS_NATION && (*ptr).kind == LOCATION_REGION {
            i += 1;
            continue;
        }

        if geoclass == GEOCLASS_REGION && (*ptr).kind == LOCATION_NATION {
            i += 1;
            continue;
        }

        if parent != 0 && (*ptr).parent != parent {
            i += 1;
            continue;
        }

        if enumproc.expect("non-null function pointer")((*ptr).id) == 0 {
            return 1;
        }

        i += 1;
    }

    return 1;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Locale support
//  *
//  * Copyright 1995 Martin von Loewis
//  * Copyright 1998 David Lee Lambert
//  * Copyright 2000 Julio César Gázquez
//  * Copyright 2002 Alexandre Julliard for CodeWeavers
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
// DEBUG_CHANNEL(nls);
//
// #include "lcformat_private.h"
//     #include "japanese.h"
//     #define strcasecmp _stricmp
//
// INT WINAPI CompareStringEx(LPCWSTR locale, DWORD flags, LPCWSTR str1, INT len1,
//                            LPCWSTR str2, INT len2, LPNLSVERSIONINFO version, LPVOID reserved, LPARAM lParam);
//
// #undef WINVER
// #define WINVER DLL_EXPORT_VERSION
//
// /* From winnls.h */
// #define LOCALE_NAME_USER_DEFAULT    NULL
//
// #define REG_SZ 1
// extern int wine_fold_string(int flags, const WCHAR *src, int srclen, WCHAR *dst, int dstlen);
// extern int wine_get_sortkey(int flags, const WCHAR *src, int srclen, char *dst, int dstlen);
// extern int wine_compare_string(int flags, const WCHAR *str1, int len1, const WCHAR *str2, int len2);
// extern UINT GetLocalisedText(IN UINT uID, IN LPWSTR lpszDest, IN UINT cchDest, IN LANGID lang);
// #define NLSRC_OFFSET 5000 /* FIXME */
//
// extern HMODULE kernel32_handle;
//
// #define LOCALE_LOCALEINFOFLAGSMASK (LOCALE_NOUSEROVERRIDE|LOCALE_USE_CP_ACP|\
//                                     LOCALE_RETURN_NUMBER|LOCALE_RETURN_GENITIVE_NAMES)
// #define MB_FLAGSMASK (MB_PRECOMPOSED|MB_COMPOSITE|MB_USEGLYPHCHARS|MB_ERR_INVALID_CHARS)
// #define WC_FLAGSMASK (WC_DISCARDNS|WC_SEPCHARS|WC_DEFAULTCHAR|WC_ERR_INVALID_CHARS|\
//                       WC_COMPOSITECHECK|WC_NO_BEST_FIT_CHARS)
//
// /* current code pages */
// static const union cptable *ansi_cptable;
// static const union cptable *oem_cptable;
// static const union cptable *mac_cptable;
// static const union cptable *unix_cptable;  /* NULL if UTF8 */
//
// static const WCHAR szLocaleKeyName[] = {
//     '\\','R','e','g','i','s','t','r','y','\\','M','a','c','h','i','n','e','\\','S','y','s','t','e','m','\\',
//     'C','u','r','r','e','n','t','C','o','n','t','r','o','l','S','e','t','\\',
//     'C','o','n','t','r','o','l','\\','N','l','s','\\','L','o','c','a','l','e',0
// };
//
// static const WCHAR szLangGroupsKeyName[] = {
//     '\\','R','e','g','i','s','t','r','y','\\','M','a','c','h','i','n','e','\\','S','y','s','t','e','m','\\',
//     'C','u','r','r','e','n','t','C','o','n','t','r','o','l','S','e','t','\\',
//     'C','o','n','t','r','o','l','\\','N','l','s','\\',
//     'L','a','n','g','u','a','g','e',' ','G','r','o','u','p','s',0
// };
//
// #if (WINVER >= 0x0600)
// /* Charset to codepage map, sorted by name. */
// static const struct charset_entry
// {
//     const char *charset_name;
//     UINT        codepage;
// } charset_names[] =
// {
//     { "BIG5", 950 },
//     { "CP1250", 1250 },
//     { "CP1251", 1251 },
//     { "CP1252", 1252 },
//     { "CP1253", 1253 },
//     { "CP1254", 1254 },
//     { "CP1255", 1255 },
//     { "CP1256", 1256 },
//     { "CP1257", 1257 },
//     { "CP1258", 1258 },
//     { "CP932", 932 },
//     { "CP936", 936 },
//     { "CP949", 949 },
//     { "CP950", 950 },
//     { "EUCJP", 20932 },
//     { "GB2312", 936 },
//     { "IBM037", 37 },
//     { "IBM1026", 1026 },
//     { "IBM424", 424 },
//     { "IBM437", 437 },
//     { "IBM500", 500 },
//     { "IBM850", 850 },
//     { "IBM852", 852 },
//     { "IBM855", 855 },
//     { "IBM857", 857 },
//     { "IBM860", 860 },
//     { "IBM861", 861 },
//     { "IBM862", 862 },
//     { "IBM863", 863 },
//     { "IBM864", 864 },
//     { "IBM865", 865 },
//     { "IBM866", 866 },
//     { "IBM869", 869 },
//     { "IBM874", 874 },
//     { "IBM875", 875 },
//     { "ISO88591", 28591 },
//     { "ISO885910", 28600 },
//     { "ISO885913", 28603 },
//     { "ISO885914", 28604 },
//     { "ISO885915", 28605 },
//     { "ISO885916", 28606 },
//     { "ISO88592", 28592 },
//     { "ISO88593", 28593 },
//     { "ISO88594", 28594 },
//     { "ISO88595", 28595 },
//     { "ISO88596", 28596 },
//     { "ISO88597", 28597 },
//     { "ISO88598", 28598 },
//     { "ISO88599", 28599 },
//     { "KOI8R", 20866 },
//     { "KOI8U", 21866 },
//     { "UTF8", CP_UTF8 }
// };
// #endif
//
//
// struct locale_name
// {
//     WCHAR  win_name[128];   /* Windows name ("en-US") */
//     WCHAR  lang[128];       /* language ("en") (note: buffer contains the other strings too) */
//     WCHAR *country;         /* country ("US") */
//     WCHAR *charset;         /* charset ("UTF-8") for Unix format only */
//     WCHAR *script;          /* script ("Latn") for Windows format only */
//     WCHAR *modifier;        /* modifier or sort order */
//     LCID   lcid;            /* corresponding LCID */
//     int    matches;         /* number of elements matching LCID (0..4) */
//     UINT   codepage;        /* codepage corresponding to charset */
// };
//
// /* locale ids corresponding to the various Unix locale parameters */
// static LCID lcid_LC_COLLATE;
// static LCID lcid_LC_CTYPE;
// static LCID lcid_LC_MESSAGES;
// static LCID lcid_LC_MONETARY;
// static LCID lcid_LC_NUMERIC;
// static LCID lcid_LC_TIME;
// static LCID lcid_LC_PAPER;
// static LCID lcid_LC_MEASUREMENT;
// static LCID lcid_LC_TELEPHONE;
//
// static const WCHAR iCalendarTypeW[] = {'i','C','a','l','e','n','d','a','r','T','y','p','e',0};
// static const WCHAR iCountryW[] = {'i','C','o','u','n','t','r','y',0};
// static const WCHAR iCurrDigitsW[] = {'i','C','u','r','r','D','i','g','i','t','s',0};
// static const WCHAR iCurrencyW[] = {'i','C','u','r','r','e','n','c','y',0};
// static const WCHAR iDateW[] = {'i','D','a','t','e',0};
// static const WCHAR iDigitsW[] = {'i','D','i','g','i','t','s',0};
// static const WCHAR iFirstDayOfWeekW[] = {'i','F','i','r','s','t','D','a','y','O','f','W','e','e','k',0};
// static const WCHAR iFirstWeekOfYearW[] = {'i','F','i','r','s','t','W','e','e','k','O','f','Y','e','a','r',0};
// static const WCHAR iLDateW[] = {'i','L','D','a','t','e',0};
// static const WCHAR iLZeroW[] = {'i','L','Z','e','r','o',0};
// static const WCHAR iMeasureW[] = {'i','M','e','a','s','u','r','e',0};
// static const WCHAR iNegCurrW[] = {'i','N','e','g','C','u','r','r',0};
// static const WCHAR iNegNumberW[] = {'i','N','e','g','N','u','m','b','e','r',0};
// static const WCHAR iPaperSizeW[] = {'i','P','a','p','e','r','S','i','z','e',0};
// static const WCHAR iTLZeroW[] = {'i','T','L','Z','e','r','o',0};
// static const WCHAR iTimePrefixW[] = {'i','T','i','m','e','P','r','e','f','i','x',0};
// static const WCHAR iTimeW[] = {'i','T','i','m','e',0};
// static const WCHAR s1159W[] = {'s','1','1','5','9',0};
// static const WCHAR s2359W[] = {'s','2','3','5','9',0};
// static const WCHAR sCountryW[] = {'s','C','o','u','n','t','r','y',0};
// static const WCHAR sCurrencyW[] = {'s','C','u','r','r','e','n','c','y',0};
// static const WCHAR sDateW[] = {'s','D','a','t','e',0};
// static const WCHAR sDecimalW[] = {'s','D','e','c','i','m','a','l',0};
// static const WCHAR sGroupingW[] = {'s','G','r','o','u','p','i','n','g',0};
// static const WCHAR sLanguageW[] = {'s','L','a','n','g','u','a','g','e',0};
// static const WCHAR sListW[] = {'s','L','i','s','t',0};
// static const WCHAR sLongDateW[] = {'s','L','o','n','g','D','a','t','e',0};
// static const WCHAR sMonDecimalSepW[] = {'s','M','o','n','D','e','c','i','m','a','l','S','e','p',0};
// static const WCHAR sMonGroupingW[] = {'s','M','o','n','G','r','o','u','p','i','n','g',0};
// static const WCHAR sMonThousandSepW[] = {'s','M','o','n','T','h','o','u','s','a','n','d','S','e','p',0};
// static const WCHAR sNativeDigitsW[] = {'s','N','a','t','i','v','e','D','i','g','i','t','s',0};
// static const WCHAR sNegativeSignW[] = {'s','N','e','g','a','t','i','v','e','S','i','g','n',0};
// static const WCHAR sPositiveSignW[] = {'s','P','o','s','i','t','i','v','e','S','i','g','n',0};
// static const WCHAR sShortDateW[] = {'s','S','h','o','r','t','D','a','t','e',0};
// static const WCHAR sThousandW[] = {'s','T','h','o','u','s','a','n','d',0};
// static const WCHAR sTimeFormatW[] = {'s','T','i','m','e','F','o','r','m','a','t',0};
// static const WCHAR sTimeW[] = {'s','T','i','m','e',0};
// static const WCHAR sYearMonthW[] = {'s','Y','e','a','r','M','o','n','t','h',0};
// static const WCHAR NumShapeW[] = {'N','u','m','s','h','a','p','e',0};
//
// static struct registry_value
// {
//     DWORD           lctype;
//     const WCHAR    *name;
//     WCHAR          *cached_value;
// } registry_values[] =
// {
//     { LOCALE_ICALENDARTYPE, iCalendarTypeW },
//     { LOCALE_ICURRDIGITS, iCurrDigitsW },
//     { LOCALE_ICURRENCY, iCurrencyW },
//     { LOCALE_IDIGITS, iDigitsW },
//     { LOCALE_IFIRSTDAYOFWEEK, iFirstDayOfWeekW },
//     { LOCALE_IFIRSTWEEKOFYEAR, iFirstWeekOfYearW },
//     { LOCALE_ILZERO, iLZeroW },
//     { LOCALE_IMEASURE, iMeasureW },
//     { LOCALE_INEGCURR, iNegCurrW },
//     { LOCALE_INEGNUMBER, iNegNumberW },
//     { LOCALE_IPAPERSIZE, iPaperSizeW },
//     { LOCALE_ITIME, iTimeW },
//     { LOCALE_S1159, s1159W },
//     { LOCALE_S2359, s2359W },
//     { LOCALE_SCURRENCY, sCurrencyW },
//     { LOCALE_SDATE, sDateW },
//     { LOCALE_SDECIMAL, sDecimalW },
//     { LOCALE_SGROUPING, sGroupingW },
//     { LOCALE_SLIST, sListW },
//     { LOCALE_SLONGDATE, sLongDateW },
//     { LOCALE_SMONDECIMALSEP, sMonDecimalSepW },
//     { LOCALE_SMONGROUPING, sMonGroupingW },
//     { LOCALE_SMONTHOUSANDSEP, sMonThousandSepW },
//     { LOCALE_SNEGATIVESIGN, sNegativeSignW },
//     { LOCALE_SPOSITIVESIGN, sPositiveSignW },
//     { LOCALE_SSHORTDATE, sShortDateW },
//     { LOCALE_STHOUSAND, sThousandW },
//     { LOCALE_STIME, sTimeW },
//     { LOCALE_STIMEFORMAT, sTimeFormatW },
//     { LOCALE_SYEARMONTH, sYearMonthW },
//     /* The following are not listed under MSDN as supported,
//      * but seem to be used and also stored in the registry.
//      */
//     { LOCALE_ICOUNTRY, iCountryW },
//     { LOCALE_IDATE, iDateW },
//     { LOCALE_ILDATE, iLDateW },
//     { LOCALE_ITLZERO, iTLZeroW },
//     { LOCALE_SCOUNTRY, sCountryW },
//     { LOCALE_SABBREVLANGNAME, sLanguageW },
//     /* The following are used in XP and later */
//     { LOCALE_IDIGITSUBSTITUTION, NumShapeW },
//     { LOCALE_SNATIVEDIGITS, sNativeDigitsW },
//     { LOCALE_ITIMEMARKPOSN, iTimePrefixW }
// };
//
// static RTL_CRITICAL_SECTION cache_section = { NULL, -1, 0, 0, 0, 0 };
//
//
// /***********************************************************************
//  *		get_lcid_codepage
//  *
//  * Retrieve the ANSI codepage for a given locale.
//  */
// static inline UINT get_lcid_codepage( LCID lcid )
// {
//     UINT ret;
//     if (!GetLocaleInfoW( lcid, LOCALE_IDEFAULTANSICODEPAGE|LOCALE_RETURN_NUMBER, (WCHAR *)&ret,
//                          sizeof(ret)/sizeof(WCHAR) )) ret = 0;
//     return ret;
// }
//
//
// #if (WINVER >= 0x0600)
// #endif // (WINVER >= 0x0600)
//
// static LANGID get_default_sublang( LANGID lang )
// {
//     switch (lang)
//     {
//     case MAKELANGID( LANG_SPANISH, SUBLANG_NEUTRAL ):
//         return MAKELANGID( LANG_SPANISH, SUBLANG_SPANISH_MODERN );
//     case MAKELANGID( LANG_CHINESE, SUBLANG_NEUTRAL ):
//         return MAKELANGID( LANG_CHINESE, SUBLANG_CHINESE_SIMPLIFIED );
//     case MAKELANGID( LANG_CHINESE, SUBLANG_CHINESE_SINGAPORE ):
//         return MAKELANGID( LANG_CHINESE, SUBLANG_CHINESE_SIMPLIFIED );
//     case MAKELANGID( LANG_CHINESE, SUBLANG_CHINESE_TRADITIONAL ):
//     case MAKELANGID( LANG_CHINESE, SUBLANG_CHINESE_MACAU ):
//         return MAKELANGID( LANG_CHINESE, SUBLANG_CHINESE_HONGKONG );
//     }
//     if (SUBLANGID( lang ) == SUBLANG_NEUTRAL) lang = MAKELANGID( PRIMARYLANGID(lang), SUBLANG_DEFAULT );
//     return lang;
// }
//
// #if (WINVER >= 0x0600)
//
// /***********************************************************************
//  *		parse_locale_name
//  *
//  * Parse a locale name into a struct locale_name, handling both Windows and Unix formats.
//  * Unix format is: lang[_country][.charset][@modifier]
//  * Windows format is: lang[-script][-country][_modifier]
//  */
// #endif // (WINVER >= 0x0600)
//
//
// /***********************************************************************
//  *           convert_default_lcid
//  *
//  * Get the default LCID to use for a given lctype in GetLocaleInfo.
//  */
// static LCID convert_default_lcid( LCID lcid, LCTYPE lctype )
// {
//     if (lcid == LOCALE_SYSTEM_DEFAULT ||
//         lcid == LOCALE_USER_DEFAULT ||
//         lcid == LOCALE_NEUTRAL)
//     {
//         LCID default_id = 0;
//
//         switch(lctype & 0xffff)
//         {
//         case LOCALE_SSORTNAME:
//             default_id = lcid_LC_COLLATE;
//             break;
//
//         case LOCALE_FONTSIGNATURE:
//         case LOCALE_IDEFAULTANSICODEPAGE:
//         case LOCALE_IDEFAULTCODEPAGE:
//         case LOCALE_IDEFAULTEBCDICCODEPAGE:
//         case LOCALE_IDEFAULTMACCODEPAGE:
//         case LOCALE_IDEFAULTUNIXCODEPAGE:
//             default_id = lcid_LC_CTYPE;
//             break;
//
//         case LOCALE_ICURRDIGITS:
//         case LOCALE_ICURRENCY:
//         case LOCALE_IINTLCURRDIGITS:
//         case LOCALE_INEGCURR:
//         case LOCALE_INEGSEPBYSPACE:
//         case LOCALE_INEGSIGNPOSN:
//         case LOCALE_INEGSYMPRECEDES:
//         case LOCALE_IPOSSEPBYSPACE:
//         case LOCALE_IPOSSIGNPOSN:
//         case LOCALE_IPOSSYMPRECEDES:
//         case LOCALE_SCURRENCY:
//         case LOCALE_SINTLSYMBOL:
//         case LOCALE_SMONDECIMALSEP:
//         case LOCALE_SMONGROUPING:
//         case LOCALE_SMONTHOUSANDSEP:
//         case LOCALE_SNATIVECURRNAME:
//             default_id = lcid_LC_MONETARY;
//             break;
//
//         case LOCALE_IDIGITS:
//         case LOCALE_IDIGITSUBSTITUTION:
//         case LOCALE_ILZERO:
//         case LOCALE_INEGNUMBER:
//         case LOCALE_SDECIMAL:
//         case LOCALE_SGROUPING:
//         //case LOCALE_SNAN:
//         case LOCALE_SNATIVEDIGITS:
//         case LOCALE_SNEGATIVESIGN:
//         //case LOCALE_SNEGINFINITY:
//         //case LOCALE_SPOSINFINITY:
//         case LOCALE_SPOSITIVESIGN:
//         case LOCALE_STHOUSAND:
//             default_id = lcid_LC_NUMERIC;
//             break;
//
//         case LOCALE_ICALENDARTYPE:
//         case LOCALE_ICENTURY:
//         case LOCALE_IDATE:
//         case LOCALE_IDAYLZERO:
//         case LOCALE_IFIRSTDAYOFWEEK:
//         case LOCALE_IFIRSTWEEKOFYEAR:
//         case LOCALE_ILDATE:
//         case LOCALE_IMONLZERO:
//         case LOCALE_IOPTIONALCALENDAR:
//         case LOCALE_ITIME:
//         case LOCALE_ITIMEMARKPOSN:
//         case LOCALE_ITLZERO:
//         case LOCALE_S1159:
//         case LOCALE_S2359:
//         case LOCALE_SABBREVDAYNAME1:
//         case LOCALE_SABBREVDAYNAME2:
//         case LOCALE_SABBREVDAYNAME3:
//         case LOCALE_SABBREVDAYNAME4:
//         case LOCALE_SABBREVDAYNAME5:
//         case LOCALE_SABBREVDAYNAME6:
//         case LOCALE_SABBREVDAYNAME7:
//         case LOCALE_SABBREVMONTHNAME1:
//         case LOCALE_SABBREVMONTHNAME2:
//         case LOCALE_SABBREVMONTHNAME3:
//         case LOCALE_SABBREVMONTHNAME4:
//         case LOCALE_SABBREVMONTHNAME5:
//         case LOCALE_SABBREVMONTHNAME6:
//         case LOCALE_SABBREVMONTHNAME7:
//         case LOCALE_SABBREVMONTHNAME8:
//         case LOCALE_SABBREVMONTHNAME9:
//         case LOCALE_SABBREVMONTHNAME10:
//         case LOCALE_SABBREVMONTHNAME11:
//         case LOCALE_SABBREVMONTHNAME12:
//         case LOCALE_SABBREVMONTHNAME13:
//         case LOCALE_SDATE:
//         case LOCALE_SDAYNAME1:
//         case LOCALE_SDAYNAME2:
//         case LOCALE_SDAYNAME3:
//         case LOCALE_SDAYNAME4:
//         case LOCALE_SDAYNAME5:
//         case LOCALE_SDAYNAME6:
//         case LOCALE_SDAYNAME7:
//         //case LOCALE_SDURATION:
//         case LOCALE_SLONGDATE:
//         case LOCALE_SMONTHNAME1:
//         case LOCALE_SMONTHNAME2:
//         case LOCALE_SMONTHNAME3:
//         case LOCALE_SMONTHNAME4:
//         case LOCALE_SMONTHNAME5:
//         case LOCALE_SMONTHNAME6:
//         case LOCALE_SMONTHNAME7:
//         case LOCALE_SMONTHNAME8:
//         case LOCALE_SMONTHNAME9:
//         case LOCALE_SMONTHNAME10:
//         case LOCALE_SMONTHNAME11:
//         case LOCALE_SMONTHNAME12:
//         case LOCALE_SMONTHNAME13:
//         case LOCALE_SSHORTDATE:
//         //case LOCALE_SSHORTESTDAYNAME1:
//         //case LOCALE_SSHORTESTDAYNAME2:
//         //case LOCALE_SSHORTESTDAYNAME3:
//         //case LOCALE_SSHORTESTDAYNAME4:
//         //case LOCALE_SSHORTESTDAYNAME5:
//         //case LOCALE_SSHORTESTDAYNAME6:
//         //case LOCALE_SSHORTESTDAYNAME7:
//         case LOCALE_STIME:
//         case LOCALE_STIMEFORMAT:
//         case LOCALE_SYEARMONTH:
//             default_id = lcid_LC_TIME;
//             break;
//
//         case LOCALE_IPAPERSIZE:
//             default_id = lcid_LC_PAPER;
//             break;
//
//         case LOCALE_IMEASURE:
//             default_id = lcid_LC_MEASUREMENT;
//             break;
//
//         case LOCALE_ICOUNTRY:
//             default_id = lcid_LC_TELEPHONE;
//             break;
//         }
//         if (default_id) lcid = default_id;
//     }
//     return ConvertDefaultLocale( lcid );
// }
//
// /***********************************************************************
//  *           is_genitive_name_supported
//  *
//  * Determine could LCTYPE basically support genitive name form or not.
//  */
// static BOOL is_genitive_name_supported( LCTYPE lctype )
// {
//     switch(lctype & 0xffff)
//     {
//     case LOCALE_SMONTHNAME1:
//     case LOCALE_SMONTHNAME2:
//     case LOCALE_SMONTHNAME3:
//     case LOCALE_SMONTHNAME4:
//     case LOCALE_SMONTHNAME5:
//     case LOCALE_SMONTHNAME6:
//     case LOCALE_SMONTHNAME7:
//     case LOCALE_SMONTHNAME8:
//     case LOCALE_SMONTHNAME9:
//     case LOCALE_SMONTHNAME10:
//     case LOCALE_SMONTHNAME11:
//     case LOCALE_SMONTHNAME12:
//     case LOCALE_SMONTHNAME13:
//          return TRUE;
//     default:
//          return FALSE;
//     }
// }
//
// /***********************************************************************
//  *		create_registry_key
//  *
//  * Create the Control Panel\\International registry key.
//  */
// static inline HANDLE create_registry_key(void)
// {
//     static const WCHAR cplW[] = {'C','o','n','t','r','o','l',' ','P','a','n','e','l',0};
//     static const WCHAR intlW[] = {'I','n','t','e','r','n','a','t','i','o','n','a','l',0};
//     OBJECT_ATTRIBUTES attr;
//     UNICODE_STRING nameW;
//     HANDLE cpl_key, hkey = 0;
//
//     if (RtlOpenCurrentUser( KEY_ALL_ACCESS, &hkey ) != STATUS_SUCCESS) return 0;
//
//     attr.Length = sizeof(attr);
//     attr.RootDirectory = hkey;
//     attr.ObjectName = &nameW;
//     attr.Attributes = 0;
//     attr.SecurityDescriptor = NULL;
//     attr.SecurityQualityOfService = NULL;
//     RtlInitUnicodeString( &nameW, cplW );
//
//     if (!NtCreateKey( &cpl_key, KEY_ALL_ACCESS, &attr, 0, NULL, 0, NULL ))
//     {
//         NtClose( attr.RootDirectory );
//         attr.RootDirectory = cpl_key;
//         RtlInitUnicodeString( &nameW, intlW );
//         if (NtCreateKey( &hkey, KEY_ALL_ACCESS, &attr, 0, NULL, 0, NULL )) hkey = 0;
//     }
//     NtClose( attr.RootDirectory );
//     return hkey;
// }
//
//
//
//
// /***********************************************************************
//  *		GetUserDefaultLangID (KERNEL32.@)
//  *
//  * Get the default language Id for the current user.
//  *
//  * PARAMS
//  *  None.
//  *
//  * RETURNS
//  *  The current LANGID of the default language for the current user.
//  */
// LANGID WINAPI GetUserDefaultLangID(void)
// {
//     return LANGIDFROMLCID(GetUserDefaultLCID());
// }
//
//
// /***********************************************************************
//  *		GetSystemDefaultLangID (KERNEL32.@)
//  *
//  * Get the default language Id for the system.
//  *
//  * PARAMS
//  *  None.
//  *
//  * RETURNS
//  *  The current LANGID of the default language for the system.
//  */
// LANGID WINAPI GetSystemDefaultLangID(void)
// {
//     return LANGIDFROMLCID(GetSystemDefaultLCID());
// }
//
//
// /***********************************************************************
//  *		GetUserDefaultLCID (KERNEL32.@)
//  *
//  * Get the default locale Id for the current user.
//  *
//  * PARAMS
//  *  None.
//  *
//  * RETURNS
//  *  The current LCID of the default locale for the current user.
//  */
// LCID WINAPI GetUserDefaultLCID(void)
// {
//     LCID lcid;
//     NtQueryDefaultLocale( TRUE, &lcid );
//     return lcid;
// }
//
//
// /***********************************************************************
//  *		GetSystemDefaultLCID (KERNEL32.@)
//  *
//  * Get the default locale Id for the system.
//  *
//  * PARAMS
//  *  None.
//  *
//  * RETURNS
//  *  The current LCID of the default locale for the system.
//  */
// LCID WINAPI GetSystemDefaultLCID(void)
// {
//     LCID lcid;
//     NtQueryDefaultLocale( FALSE, &lcid );
//     return lcid;
// }
//
//
// /***********************************************************************
//  *		GetUserDefaultUILanguage (KERNEL32.@)
//  *
//  * Get the default user interface language Id for the current user.
//  *
//  * PARAMS
//  *  None.
//  *
//  * RETURNS
//  *  The current LANGID of the default UI language for the current user.
//  */
// LANGID WINAPI GetUserDefaultUILanguage(void)
// {
//     LANGID lang;
//     NtQueryDefaultUILanguage( &lang );
//     return lang;
// }
//
//
// /***********************************************************************
//  *		GetSystemDefaultUILanguage (KERNEL32.@)
//  *
//  * Get the default user interface language Id for the system.
//  *
//  * PARAMS
//  *  None.
//  *
//  * RETURNS
//  *  The current LANGID of the default UI language for the system. This is
//  *  typically the same language used during the installation process.
//  */
// LANGID WINAPI GetSystemDefaultUILanguage(void)
// {
//     LANGID lang;
//     NtQueryInstallUILanguage( &lang );
//     return lang;
// }
//
// #if (WINVER >= 0x0600)
//
//
// #endif
//
//
// /******************************************************************************
//  *		get_locale_registry_value
//  *
//  * Gets the registry value name and cache for a given lctype.
//  */
// static struct registry_value *get_locale_registry_value( DWORD lctype )
// {
//     int i;
//     for (i=0; i < sizeof(registry_values)/sizeof(registry_values[0]); i++)
//         if (registry_values[i].lctype == lctype)
//             return &registry_values[i];
//     return NULL;
// }
//
//
// /******************************************************************************
//  *		get_registry_locale_info
//  *
//  * Retrieve user-modified locale info from the registry.
//  * Return length, 0 on error, -1 if not found.
//  */
// static INT get_registry_locale_info( struct registry_value *registry_value, LPWSTR buffer, INT len )
// {
//     DWORD size;
//     INT ret;
//     HANDLE hkey;
//     NTSTATUS status;
//     UNICODE_STRING nameW;
//     KEY_VALUE_PARTIAL_INFORMATION *info;
//     static const int info_size = FIELD_OFFSET(KEY_VALUE_PARTIAL_INFORMATION, Data);
//
//     RtlEnterCriticalSection( &cache_section );
//
//     if (!registry_value->cached_value)
//     {
//         if (!(hkey = create_registry_key()))
//         {
//             RtlLeaveCriticalSection( &cache_section );
//             return -1;
//         }
//
//         RtlInitUnicodeString( &nameW, registry_value->name );
//         size = info_size + len * sizeof(WCHAR);
//
//         if (!(info = HeapAlloc( GetProcessHeap(), 0, size )))
//         {
//             NtClose( hkey );
//             SetLastError( ERROR_NOT_ENOUGH_MEMORY );
//             RtlLeaveCriticalSection( &cache_section );
//             return 0;
//         }
//
//         status = NtQueryValueKey( hkey, &nameW, KeyValuePartialInformation, info, size, &size );
//
//         /* try again with a bigger buffer when we have to return the correct size */
//         if (status == STATUS_BUFFER_OVERFLOW && !buffer && size > info_size)
//         {
//             KEY_VALUE_PARTIAL_INFORMATION *new_info;
//             if ((new_info = HeapReAlloc( GetProcessHeap(), 0, info, size )))
//             {
//                 info = new_info;
//                 status = NtQueryValueKey( hkey, &nameW, KeyValuePartialInformation, info, size, &size );
//             }
//         }
//
//         NtClose( hkey );
//
//         if (!status)
//         {
//             INT length = (size - info_size) / sizeof(WCHAR);
//             LPWSTR cached_value;
//
//             if (!length || ((WCHAR *)&info->Data)[length-1])
//                 length++;
//
//             cached_value = HeapAlloc( GetProcessHeap(), 0, length * sizeof(WCHAR) );
//
//             if (!cached_value)
//             {
//                 HeapFree( GetProcessHeap(), 0, info );
//                 SetLastError( ERROR_NOT_ENOUGH_MEMORY );
//                 RtlLeaveCriticalSection( &cache_section );
//                 return 0;
//             }
//
//             memcpy( cached_value, info->Data, (length-1) * sizeof(WCHAR) );
//             cached_value[length-1] = 0;
//             HeapFree( GetProcessHeap(), 0, info );
//             registry_value->cached_value = cached_value;
//         }
//         else
//         {
//             if (status == STATUS_BUFFER_OVERFLOW && !buffer)
//             {
//                 ret = (size - info_size) / sizeof(WCHAR);
//             }
//             else if (status == STATUS_OBJECT_NAME_NOT_FOUND)
//             {
//                 ret = -1;
//             }
//             else
//             {
//                 SetLastError( RtlNtStatusToDosError(status) );
//                 ret = 0;
//             }
//             HeapFree( GetProcessHeap(), 0, info );
//             RtlLeaveCriticalSection( &cache_section );
//             return ret;
//         }
//     }
//
//     ret = lstrlenW( registry_value->cached_value ) + 1;
//
//     if (buffer)
//     {
//         if (ret > len)
//         {
//             SetLastError( ERROR_INSUFFICIENT_BUFFER );
//             ret = 0;
//         }
//         else
//         {
//             lstrcpyW( buffer, registry_value->cached_value );
//         }
//     }
//
//     RtlLeaveCriticalSection( &cache_section );
//
//     return ret;
// }
//
//
// /******************************************************************************
//  *		GetLocaleInfoA (KERNEL32.@)
//  *
//  * Get information about an aspect of a locale.
//  *
//  * PARAMS
//  *  lcid   [I] LCID of the locale
//  *  lctype [I] LCTYPE_ flags from "winnls.h"
//  *  buffer [O] Destination for the information
//  *  len    [I] Length of buffer in characters
//  *
//  * RETURNS
//  *  Success: The size of the data requested. If buffer is non-NULL, it is filled
//  *           with the information.
//  *  Failure: 0. Use GetLastError() to determine the cause.
//  *
//  * NOTES
//  *  - LOCALE_NEUTRAL is equal to LOCALE_SYSTEM_DEFAULT
//  *  - The string returned is NUL terminated, except for LOCALE_FONTSIGNATURE,
//  *    which is a bit string.
//  */
// INT WINAPI GetLocaleInfoA( LCID lcid, LCTYPE lctype, LPSTR buffer, INT len )
// {
//     WCHAR *bufferW;
//     INT lenW, ret;
//
//     TRACE( "(lcid=0x%x,lctype=0x%x,%p,%d)\n", lcid, lctype, buffer, len );
//
//     if (len < 0 || (len && !buffer))
//     {
//         SetLastError( ERROR_INVALID_PARAMETER );
//         return 0;
//     }
//     if (((lctype & ~LOCALE_LOCALEINFOFLAGSMASK) == LOCALE_SSHORTTIME) ||
//          (lctype & LOCALE_RETURN_GENITIVE_NAMES))
//     {
//         SetLastError( ERROR_INVALID_FLAGS );
//         return 0;
//     }
//
//     if (!len) buffer = NULL;
//
//     if (!(lenW = GetLocaleInfoW( lcid, lctype, NULL, 0 ))) return 0;
//
//     if (!(bufferW = HeapAlloc( GetProcessHeap(), 0, lenW * sizeof(WCHAR) )))
//     {
//         SetLastError( ERROR_NOT_ENOUGH_MEMORY );
//         return 0;
//     }
//     if ((ret = GetLocaleInfoW( lcid, lctype, bufferW, lenW )))
//     {
//         if ((lctype & LOCALE_RETURN_NUMBER) ||
//             ((lctype & ~LOCALE_LOCALEINFOFLAGSMASK) == LOCALE_FONTSIGNATURE))
//         {
//             /* it's not an ASCII string, just bytes */
//             ret *= sizeof(WCHAR);
//             if (buffer)
//             {
//                 if (ret <= len) memcpy( buffer, bufferW, ret );
//                 else
//                 {
//                     SetLastError( ERROR_INSUFFICIENT_BUFFER );
//                     ret = 0;
//                 }
//             }
//         }
//         else
//         {
//             UINT codepage = CP_ACP;
//             if (!(lctype & LOCALE_USE_CP_ACP)) codepage = get_lcid_codepage( lcid );
//             ret = WideCharToMultiByte( codepage, 0, bufferW, ret, buffer, len, NULL, NULL );
//         }
//     }
//     HeapFree( GetProcessHeap(), 0, bufferW );
//     return ret;
// }
//
// static int get_value_base_by_lctype( LCTYPE lctype )
// {
//     return lctype == LOCALE_ILANGUAGE || lctype == LOCALE_IDEFAULTLANGUAGE ? 16 : 10;
// }
//
// /******************************************************************************
//  *		GetLocaleInfoW (KERNEL32.@)
//  *
//  * See GetLocaleInfoA.
//  */
// INT WINAPI GetLocaleInfoW( LCID lcid, LCTYPE lctype, LPWSTR buffer, INT len )
// {
//     LANGID lang_id;
//     HRSRC hrsrc;
//     HGLOBAL hmem;
//     INT ret;
//     UINT lcflags;
//     const WCHAR *p;
//     unsigned int i;
//
//     if (len < 0 || (len && !buffer))
//     {
//         SetLastError( ERROR_INVALID_PARAMETER );
//         return 0;
//     }
//     if (lctype & LOCALE_RETURN_GENITIVE_NAMES &&
//        !is_genitive_name_supported( lctype ))
//     {
//         SetLastError( ERROR_INVALID_FLAGS );
//         return 0;
//     }
//
//     if (!len) buffer = NULL;
//
//     lcid = convert_default_lcid( lcid, lctype );
//
//     lcflags = lctype & LOCALE_LOCALEINFOFLAGSMASK;
//     lctype &= 0xffff;
//
//     TRACE( "(lcid=0x%x,lctype=0x%x,%p,%d)\n", lcid, lctype, buffer, len );
//
//     /* first check for overrides in the registry */
//
//     if (!(lcflags & LOCALE_NOUSEROVERRIDE) &&
//         lcid == convert_default_lcid( LOCALE_USER_DEFAULT, lctype ))
//     {
//         struct registry_value *value = get_locale_registry_value(lctype);
//
//         if (value)
//         {
//             if (lcflags & LOCALE_RETURN_NUMBER)
//             {
//                 WCHAR tmp[16];
//                 ret = get_registry_locale_info( value, tmp, sizeof(tmp)/sizeof(WCHAR) );
//                 if (ret > 0)
//                 {
//                     WCHAR *end;
//                     UINT number = strtolW( tmp, &end, get_value_base_by_lctype( lctype ) );
//                     if (*end)  /* invalid number */
//                     {
//                         SetLastError( ERROR_INVALID_FLAGS );
//                         return 0;
//                     }
//                     ret = sizeof(UINT)/sizeof(WCHAR);
//                     if (!buffer) return ret;
//                     if (ret > len)
//                     {
//                         SetLastError( ERROR_INSUFFICIENT_BUFFER );
//                         return 0;
//                     }
//                     memcpy( buffer, &number, sizeof(number) );
//                 }
//             }
//             else ret = get_registry_locale_info( value, buffer, len );
//
//             if (ret != -1) return ret;
//         }
//     }
//
//     /* now load it from kernel resources */
//
//     lang_id = LANGIDFROMLCID( lcid );
//
//     /* replace SUBLANG_NEUTRAL by SUBLANG_DEFAULT */
//     if (SUBLANGID(lang_id) == SUBLANG_NEUTRAL) lang_id = get_default_sublang( lang_id );
//
//     if (!(hrsrc = FindResourceExW( kernel32_handle, (LPWSTR)RT_STRING,
//                                    ULongToPtr((lctype >> 4) + 1), lang_id )))
//     {
//         SetLastError( ERROR_INVALID_FLAGS );  /* no such lctype */
//         return 0;
//     }
//     if (!(hmem = LoadResource( kernel32_handle, hrsrc )))
//         return 0;
//
//     p = LockResource( hmem );
//     for (i = 0; i < (lctype & 0x0f); i++) p += *p + 1;
//
//     if (lcflags & LOCALE_RETURN_NUMBER) ret = sizeof(UINT)/sizeof(WCHAR);
//     else if (is_genitive_name_supported( lctype ) && *p)
//     {
//         /* genitive form's stored after a null separator from a nominative */
//         for (i = 1; i <= *p; i++) if (!p[i]) break;
//
//         if (i <= *p && (lcflags & LOCALE_RETURN_GENITIVE_NAMES))
//         {
//             ret = *p - i + 1;
//             p += i;
//         }
//         else ret = i;
//     }
//     else
//         ret = (lctype == LOCALE_FONTSIGNATURE) ? *p : *p + 1;
//
//     if (!buffer) return ret;
//
//     if (ret > len)
//     {
//         SetLastError( ERROR_INSUFFICIENT_BUFFER );
//         return 0;
//     }
//
//     if (lcflags & LOCALE_RETURN_NUMBER)
//     {
//         UINT number;
//         WCHAR *end, *tmp = HeapAlloc( GetProcessHeap(), 0, (*p + 1) * sizeof(WCHAR) );
//         if (!tmp) return 0;
//         memcpy( tmp, p + 1, *p * sizeof(WCHAR) );
//         tmp[*p] = 0;
//         number = strtolW( tmp, &end, get_value_base_by_lctype( lctype ) );
//         if (!*end)
//             memcpy( buffer, &number, sizeof(number) );
//         else  /* invalid number */
//         {
//             SetLastError( ERROR_INVALID_FLAGS );
//             ret = 0;
//         }
//         HeapFree( GetProcessHeap(), 0, tmp );
//
//         TRACE( "(lcid=0x%x,lctype=0x%x,%p,%d) returning number %d\n",
//                lcid, lctype, buffer, len, number );
//     }
//     else
//     {
//         memcpy( buffer, p + 1, ret * sizeof(WCHAR) );
//         if (lctype != LOCALE_FONTSIGNATURE) buffer[ret-1] = 0;
//
//         TRACE( "(lcid=0x%x,lctype=0x%x,%p,%d) returning %d %s\n",
//                lcid, lctype, buffer, len, ret, debugstr_w(buffer) );
//     }
//     return ret;
// }
//
// #if (WINVER >= 0x0600)
//
// #endif
//
// /******************************************************************************
//  *		SetLocaleInfoA	[KERNEL32.@]
//  *
//  * Set information about an aspect of a locale.
//  *
//  * PARAMS
//  *  lcid   [I] LCID of the locale
//  *  lctype [I] LCTYPE_ flags from "winnls.h"
//  *  data   [I] Information to set
//  *
//  * RETURNS
//  *  Success: TRUE. The information given will be returned by GetLocaleInfoA()
//  *           whenever it is called without LOCALE_NOUSEROVERRIDE.
//  *  Failure: FALSE. Use GetLastError() to determine the cause.
//  *
//  * NOTES
//  *  - Values are only be set for the current user locale; the system locale
//  *  settings cannot be changed.
//  *  - Any settings changed by this call are lost when the locale is changed by
//  *  the control panel (in Wine, this happens every time you change LANG).
//  *  - The native implementation of this function does not check that lcid matches
//  *  the current user locale, and simply sets the new values. Wine warns you in
//  *  this case, but behaves the same.
//  */
// BOOL WINAPI SetLocaleInfoA(LCID lcid, LCTYPE lctype, LPCSTR data)
// {
//     UINT codepage = CP_ACP;
//     WCHAR *strW;
//     DWORD len;
//     BOOL ret;
//
//     if (!(lctype & LOCALE_USE_CP_ACP)) codepage = get_lcid_codepage( lcid );
//
//     if (!data)
//     {
//         SetLastError( ERROR_INVALID_PARAMETER );
//         return FALSE;
//     }
//     len = MultiByteToWideChar( codepage, 0, data, -1, NULL, 0 );
//     if (!(strW = HeapAlloc( GetProcessHeap(), 0, len * sizeof(WCHAR) )))
//     {
//         SetLastError( ERROR_NOT_ENOUGH_MEMORY );
//         return FALSE;
//     }
//     MultiByteToWideChar( codepage, 0, data, -1, strW, len );
//     ret = SetLocaleInfoW( lcid, lctype, strW );
//     HeapFree( GetProcessHeap(), 0, strW );
//     return ret;
// }
//
//
// /******************************************************************************
//  *		SetLocaleInfoW	(KERNEL32.@)
//  *
//  * See SetLocaleInfoA.
//  */
// BOOL WINAPI SetLocaleInfoW( LCID lcid, LCTYPE lctype, LPCWSTR data )
// {
//     struct registry_value *value;
//     static const WCHAR intlW[] = {'i','n','t','l',0 };
//     UNICODE_STRING valueW;
//     NTSTATUS status;
//     HANDLE hkey;
//
//     lctype &= 0xffff;
//     value = get_locale_registry_value( lctype );
//
//     if (!data || !value)
//     {
//         SetLastError( ERROR_INVALID_PARAMETER );
//         return FALSE;
//     }
//
//     if (lctype == LOCALE_IDATE || lctype == LOCALE_ILDATE)
//     {
//         SetLastError( ERROR_INVALID_FLAGS );
//         return FALSE;
//     }
//
//     TRACE("setting %x (%s) to %s\n", lctype, debugstr_w(value->name), debugstr_w(data) );
//
//     /* FIXME: should check that data to set is sane */
//
//     /* FIXME: profile functions should map to registry */
//     WriteProfileStringW( intlW, value->name, data );
//
//     if (!(hkey = create_registry_key())) return FALSE;
//     RtlInitUnicodeString( &valueW, value->name );
//     status = NtSetValueKey( hkey, &valueW, 0, REG_SZ, (PVOID)data, (strlenW(data)+1)*sizeof(WCHAR) );
//
//     RtlEnterCriticalSection( &cache_section );
//     HeapFree( GetProcessHeap(), 0, value->cached_value );
//     value->cached_value = NULL;
//     RtlLeaveCriticalSection( &cache_section );
//
//     if (lctype == LOCALE_SSHORTDATE || lctype == LOCALE_SLONGDATE)
//     {
//       /* Set I-value from S value */
//       WCHAR *lpD, *lpM, *lpY;
//       WCHAR szBuff[2];
//
//       lpD = strrchrW(data, 'd');
//       lpM = strrchrW(data, 'M');
//       lpY = strrchrW(data, 'y');
//
//       if (lpD <= lpM)
//       {
//         szBuff[0] = '1'; /* D-M-Y */
//       }
//       else
//       {
//         if (lpY <= lpM)
//           szBuff[0] = '2'; /* Y-M-D */
//         else
//           szBuff[0] = '0'; /* M-D-Y */
//       }
//
//       szBuff[1] = '\0';
//
//       if (lctype == LOCALE_SSHORTDATE)
//         lctype = LOCALE_IDATE;
//       else
//         lctype = LOCALE_ILDATE;
//
//       value = get_locale_registry_value( lctype );
//
//       WriteProfileStringW( intlW, value->name, szBuff );
//
//       RtlInitUnicodeString( &valueW, value->name );
//       status = NtSetValueKey( hkey, &valueW, 0, REG_SZ, szBuff, sizeof(szBuff) );
//
//       RtlEnterCriticalSection( &cache_section );
//       HeapFree( GetProcessHeap(), 0, value->cached_value );
//       value->cached_value = NULL;
//       RtlLeaveCriticalSection( &cache_section );
//     }
//
//     NtClose( hkey );
//
//     if (status) SetLastError( RtlNtStatusToDosError(status) );
//     return !status;
// }
//
//
//
//
// /***********************************************************************
//  *           GetThreadLocale    (KERNEL32.@)
//  *
//  * Get the current threads locale.
//  *
//  * PARAMS
//  *  None.
//  *
//  * RETURNS
//  *  The LCID currently associated with the calling thread.
//  */
// LCID WINAPI GetThreadLocale(void)
// {
//     LCID ret = NtCurrentTeb()->CurrentLocale;
//     if (!ret) NtCurrentTeb()->CurrentLocale = ret = GetUserDefaultLCID();
//     return ret;
// }
//
// /**********************************************************************
//  *           SetThreadLocale    (KERNEL32.@)
//  *
//  * Set the current threads locale.
//  *
//  * PARAMS
//  *  lcid [I] LCID of the locale to set
//  *
//  * RETURNS
//  *  Success: TRUE. The threads locale is set to lcid.
//  *  Failure: FALSE. Use GetLastError() to determine the cause.
//  */
// BOOL WINAPI SetThreadLocale( LCID lcid )
// {
//     TRACE("(0x%04X)\n", lcid);
//
//     lcid = ConvertDefaultLocale(lcid);
//
//     if (lcid != GetThreadLocale())
//     {
//         if (!IsValidLocale(lcid, LCID_SUPPORTED))
//         {
//             SetLastError(ERROR_INVALID_PARAMETER);
//             return FALSE;
//         }
//
//         NtCurrentTeb()->CurrentLocale = lcid;
//     }
//     return TRUE;
// }
//
//
// /******************************************************************************
//  *		ConvertDefaultLocale (KERNEL32.@)
//  *
//  * Convert a default locale identifier into a real identifier.
//  *
//  * PARAMS
//  *  lcid [I] LCID identifier of the locale to convert
//  *
//  * RETURNS
//  *  lcid unchanged, if not a default locale or its sublanguage is
//  *   not SUBLANG_NEUTRAL.
//  *  GetSystemDefaultLCID(), if lcid == LOCALE_SYSTEM_DEFAULT.
//  *  GetUserDefaultLCID(), if lcid == LOCALE_USER_DEFAULT or LOCALE_NEUTRAL.
//  *  Otherwise, lcid with sublanguage changed to SUBLANG_DEFAULT.
//  */
// LCID WINAPI ConvertDefaultLocale( LCID lcid )
// {
//     LANGID langid;
//
//     switch (lcid)
//     {
//     case LOCALE_INVARIANT:
//         /* keep as-is */
//         break;
//     case LOCALE_SYSTEM_DEFAULT:
//         lcid = GetSystemDefaultLCID();
//         break;
//     case LOCALE_USER_DEFAULT:
//     case LOCALE_NEUTRAL:
//         lcid = GetUserDefaultLCID();
//         break;
//     default:
//         /* Replace SUBLANG_NEUTRAL with SUBLANG_DEFAULT */
//         langid = LANGIDFROMLCID(lcid);
//         if (SUBLANGID(langid) == SUBLANG_NEUTRAL)
//         {
//           langid = get_default_sublang( langid );
//           lcid = MAKELCID(langid, SORTIDFROMLCID(lcid));
//         }
//     }
//     return lcid;
// }
//
//
// /******************************************************************************
//  *           IsValidLocale   (KERNEL32.@)
//  *
//  * Determine if a locale is valid.
//  *
//  * PARAMS
//  *  lcid  [I] LCID of the locale to check
//  *  flags [I] LCID_SUPPORTED = Valid, LCID_INSTALLED = Valid and installed on the system
//  *
//  * RETURNS
//  *  TRUE,  if lcid is valid,
//  *  FALSE, otherwise.
//  *
//  * NOTES
//  *  Wine does not currently make the distinction between supported and installed. All
//  *  languages supported are installed by default.
//  */
// BOOL WINAPI IsValidLocale( LCID lcid, DWORD flags )
// {
//     /* check if language is registered in the kernel32 resources */
//     return FindResourceExW( kernel32_handle, (LPWSTR)RT_STRING,
//                             (LPCWSTR)LOCALE_ILANGUAGE, LANGIDFROMLCID(lcid)) != 0;
// }
//
//
// static BOOL __stdcall enum_lang_proc_a( HMODULE hModule, LPCSTR type,
//                                        LPCSTR name, WORD LangID, LONG_PTR lParam )
// {
//     LOCALE_ENUMPROCA lpfnLocaleEnum = (LOCALE_ENUMPROCA)lParam;
//     char buf[20];
//
//     sprintf(buf, "%08x", (UINT)LangID);
//     return lpfnLocaleEnum( buf );
// }
//
// static BOOL __stdcall enum_lang_proc_w( HMODULE hModule, LPCWSTR type,
//                                        LPCWSTR name, WORD LangID, LONG_PTR lParam )
// {
//     static const WCHAR formatW[] = {'%','0','8','x',0};
//     LOCALE_ENUMPROCW lpfnLocaleEnum = (LOCALE_ENUMPROCW)lParam;
//     WCHAR buf[20];
//     sprintfW( buf, formatW, (UINT)LangID );
//     return lpfnLocaleEnum( buf );
// }
//
// /******************************************************************************
//  *           EnumSystemLocalesA  (KERNEL32.@)
//  *
//  * Call a users function for each locale available on the system.
//  *
//  * PARAMS
//  *  lpfnLocaleEnum [I] Callback function to call for each locale
//  *  dwFlags        [I] LOCALE_SUPPORTED=All supported, LOCALE_INSTALLED=Installed only
//  *
//  * RETURNS
//  *  Success: TRUE.
//  *  Failure: FALSE. Use GetLastError() to determine the cause.
//  */
// BOOL WINAPI EnumSystemLocalesA( LOCALE_ENUMPROCA lpfnLocaleEnum, DWORD dwFlags )
// {
//     TRACE("(%p,%08x)\n", lpfnLocaleEnum, dwFlags);
//     EnumResourceLanguagesA( kernel32_handle, (LPSTR)RT_STRING,
//                             (LPCSTR)LOCALE_ILANGUAGE, enum_lang_proc_a,
//                             (LONG_PTR)lpfnLocaleEnum);
//     return TRUE;
// }
//
//
// /******************************************************************************
//  *           EnumSystemLocalesW  (KERNEL32.@)
//  *
//  * See EnumSystemLocalesA.
//  */
// BOOL WINAPI EnumSystemLocalesW( LOCALE_ENUMPROCW lpfnLocaleEnum, DWORD dwFlags )
// {
//     TRACE("(%p,%08x)\n", lpfnLocaleEnum, dwFlags);
//     EnumResourceLanguagesW( kernel32_handle, (LPWSTR)RT_STRING,
//                             (LPCWSTR)LOCALE_ILANGUAGE, enum_lang_proc_w,
//                             (LONG_PTR)lpfnLocaleEnum);
//     return TRUE;
// }
//
//
// struct enum_locale_ex_data
// {
//     LOCALE_ENUMPROCEX proc;
//     DWORD             flags;
//     LPARAM            lparam;
// };
//
//
// /***********************************************************************
//  *           VerLanguageNameA  (KERNEL32.@)
//  *
//  * Get the name of a language.
//  *
//  * PARAMS
//  *  wLang  [I] LANGID of the language
//  *  szLang [O] Destination for the language name
//  *
//  * RETURNS
//  *  Success: The size of the language name. If szLang is non-NULL, it is filled
//  *           with the name.
//  *  Failure: 0. Use GetLastError() to determine the cause.
//  *
//  */
// DWORD WINAPI VerLanguageNameA( DWORD wLang, LPSTR szLang, DWORD nSize )
// {
//     return GetLocaleInfoA( MAKELCID(wLang, SORT_DEFAULT), LOCALE_SENGLANGUAGE, szLang, nSize );
// }
//
//
// /***********************************************************************
//  *           VerLanguageNameW  (KERNEL32.@)
//  *
//  * See VerLanguageNameA.
//  */
// DWORD WINAPI VerLanguageNameW( DWORD wLang, LPWSTR szLang, DWORD nSize )
// {
//     return GetLocaleInfoW( MAKELCID(wLang, SORT_DEFAULT), LOCALE_SENGLANGUAGE, szLang, nSize );
// }
//
//
// /******************************************************************************
//  *           GetStringTypeW    (KERNEL32.@)
//  *
//  * See GetStringTypeA.
//  */
// BOOL WINAPI GetStringTypeW( DWORD type, LPCWSTR src, INT count, LPWORD chartype )
// {
//     static const unsigned char type2_map[16] =
//     {
//         C2_NOTAPPLICABLE,      /* unassigned */
//         C2_LEFTTORIGHT,        /* L */
//         C2_RIGHTTOLEFT,        /* R */
//         C2_EUROPENUMBER,       /* EN */
//         C2_EUROPESEPARATOR,    /* ES */
//         C2_EUROPETERMINATOR,   /* ET */
//         C2_ARABICNUMBER,       /* AN */
//         C2_COMMONSEPARATOR,    /* CS */
//         C2_BLOCKSEPARATOR,     /* B */
//         C2_SEGMENTSEPARATOR,   /* S */
//         C2_WHITESPACE,         /* WS */
//         C2_OTHERNEUTRAL,       /* ON */
//         C2_RIGHTTOLEFT,        /* AL */
//         C2_NOTAPPLICABLE,      /* NSM */
//         C2_NOTAPPLICABLE,      /* BN */
//         C2_OTHERNEUTRAL        /* LRE, LRO, RLE, RLO, PDF */
//     };
//
//     if (!src)
//     {
//         SetLastError( ERROR_INVALID_PARAMETER );
//         return FALSE;
//     }
//
//     if (count == -1) count = strlenW(src) + 1;
//     switch(type)
//     {
//     case CT_CTYPE1:
//         while (count--) *chartype++ = get_char_typeW( *src++ ) & 0xfff;
//         break;
//     case CT_CTYPE2:
//         while (count--) *chartype++ = type2_map[get_char_typeW( *src++ ) >> 12];
//         break;
//     case CT_CTYPE3:
//     {
//         WARN("CT_CTYPE3: semi-stub.\n");
//         while (count--)
//         {
//             int c = *src;
//             WORD type1, type3 = 0; /* C3_NOTAPPLICABLE */
//
//             type1 = get_char_typeW( *src++ ) & 0xfff;
//             /* try to construct type3 from type1 */
//             if(type1 & C1_SPACE) type3 |= C3_SYMBOL;
//             if(type1 & C1_ALPHA) type3 |= C3_ALPHA;
//             if ((c>=0x30A0)&&(c<=0x30FF)) type3 |= C3_KATAKANA;
//             if ((c>=0x3040)&&(c<=0x309F)) type3 |= C3_HIRAGANA;
//             if ((c>=0x4E00)&&(c<=0x9FAF)) type3 |= C3_IDEOGRAPH;
//             if (c == 0x0640) type3 |= C3_KASHIDA;
//             if ((c>=0x3000)&&(c<=0x303F)) type3 |= C3_SYMBOL;
//
//             if ((c>=0xD800)&&(c<=0xDBFF)) type3 |= C3_HIGHSURROGATE;
//             if ((c>=0xDC00)&&(c<=0xDFFF)) type3 |= C3_LOWSURROGATE;
//
//             if ((c>=0xFF00)&&(c<=0xFF60)) type3 |= C3_FULLWIDTH;
//             if ((c>=0xFF00)&&(c<=0xFF20)) type3 |= C3_SYMBOL;
//             if ((c>=0xFF3B)&&(c<=0xFF40)) type3 |= C3_SYMBOL;
//             if ((c>=0xFF5B)&&(c<=0xFF60)) type3 |= C3_SYMBOL;
//             if ((c>=0xFF21)&&(c<=0xFF3A)) type3 |= C3_ALPHA;
//             if ((c>=0xFF41)&&(c<=0xFF5A)) type3 |= C3_ALPHA;
//             if ((c>=0xFFE0)&&(c<=0xFFE6)) type3 |= C3_FULLWIDTH;
//             if ((c>=0xFFE0)&&(c<=0xFFE6)) type3 |= C3_SYMBOL;
//
//             if ((c>=0xFF61)&&(c<=0xFFDC)) type3 |= C3_HALFWIDTH;
//             if ((c>=0xFF61)&&(c<=0xFF64)) type3 |= C3_SYMBOL;
//             if ((c>=0xFF65)&&(c<=0xFF9F)) type3 |= C3_KATAKANA;
//             if ((c>=0xFF65)&&(c<=0xFF9F)) type3 |= C3_ALPHA;
//             if ((c>=0xFFE8)&&(c<=0xFFEE)) type3 |= C3_HALFWIDTH;
//             if ((c>=0xFFE8)&&(c<=0xFFEE)) type3 |= C3_SYMBOL;
//             *chartype++ = type3;
//         }
//         break;
//     }
//     default:
//         SetLastError( ERROR_INVALID_PARAMETER );
//         return FALSE;
//     }
//     return TRUE;
// }
//
//
// /******************************************************************************
//  *           GetStringTypeExW    (KERNEL32.@)
//  *
//  * See GetStringTypeExA.
//  */
// BOOL WINAPI GetStringTypeExW( LCID locale, DWORD type, LPCWSTR src, INT count, LPWORD chartype )
// {
//     /* locale is ignored for Unicode */
//     return GetStringTypeW( type, src, count, chartype );
// }
//
//
// /******************************************************************************
//  *           GetStringTypeA    (KERNEL32.@)
//  *
//  * Get characteristics of the characters making up a string.
//  *
//  * PARAMS
//  *  locale   [I] Locale Id for the string
//  *  type     [I] CT_CTYPE1 = classification, CT_CTYPE2 = directionality, CT_CTYPE3 = typographic info
//  *  src      [I] String to analyse
//  *  count    [I] Length of src in chars, or -1 if src is NUL terminated
//  *  chartype [O] Destination for the calculated characteristics
//  *
//  * RETURNS
//  *  Success: TRUE. chartype is filled with the requested characteristics of each char
//  *           in src.
//  *  Failure: FALSE. Use GetLastError() to determine the cause.
//  */
// BOOL WINAPI GetStringTypeA( LCID locale, DWORD type, LPCSTR src, INT count, LPWORD chartype )
// {
//     UINT cp;
//     INT countW;
//     LPWSTR srcW;
//     BOOL ret = FALSE;
//
//     if(count == -1) count = strlen(src) + 1;
//
//     if (!(cp = get_lcid_codepage( locale )))
//     {
//         FIXME("For locale %04x using current ANSI code page\n", locale);
//         cp = GetACP();
//     }
//
//     countW = MultiByteToWideChar(cp, 0, src, count, NULL, 0);
//     if((srcW = HeapAlloc(GetProcessHeap(), 0, countW * sizeof(WCHAR))))
//     {
//         MultiByteToWideChar(cp, 0, src, count, srcW, countW);
//     /*
//      * NOTE: the target buffer has 1 word for each CHARACTER in the source
//      * string, with multibyte characters there maybe be more bytes in count
//      * than character space in the buffer!
//      */
//         ret = GetStringTypeW(type, srcW, countW, chartype);
//         HeapFree(GetProcessHeap(), 0, srcW);
//     }
//     return ret;
// }
//
// /******************************************************************************
//  *           GetStringTypeExA    (KERNEL32.@)
//  *
//  * Get characteristics of the characters making up a string.
//  *
//  * PARAMS
//  *  locale   [I] Locale Id for the string
//  *  type     [I] CT_CTYPE1 = classification, CT_CTYPE2 = directionality, CT_CTYPE3 = typographic info
//  *  src      [I] String to analyse
//  *  count    [I] Length of src in chars, or -1 if src is NUL terminated
//  *  chartype [O] Destination for the calculated characteristics
//  *
//  * RETURNS
//  *  Success: TRUE. chartype is filled with the requested characteristics of each char
//  *           in src.
//  *  Failure: FALSE. Use GetLastError() to determine the cause.
//  */
// BOOL WINAPI GetStringTypeExA( LCID locale, DWORD type, LPCSTR src, INT count, LPWORD chartype )
// {
//     return GetStringTypeA(locale, type, src, count, chartype);
// }
//
// static inline void map_byterev(const WCHAR *src, int len, WCHAR *dst)
// {
//     while (len--)
//         *dst++ = RtlUshortByteSwap(*src++);
// }
//
// static int map_to_hiragana(const WCHAR *src, int srclen, WCHAR *dst, int dstlen)
// {
//     int pos;
//     for (pos = 0; srclen; src++, srclen--, pos++)
//     {
//         /*
//          * U+30A1 ... U+30F3: Katakana
//          * U+30F4: Katakana Letter VU
//          * U+30F5: Katakana Letter Small KA
//          * U+30FD: Katakana Iteration Mark
//          * U+30FE: Katakana Voiced Iteration Mark
//          */
//         WCHAR wch = *src;
//         if ((0x30A1 <= wch && wch <= 0x30F3) ||
//             wch == 0x30F4 || wch == 0x30F5 || wch == 0x30FD || wch == 0x30FE)
//         {
//             wch -= 0x60; /* Katakana to Hiragana */
//         }
//         if (pos < dstlen)
//             dst[pos] = wch;
//     }
//     return pos;
// }
//
// static int map_to_katakana(const WCHAR *src, int srclen, WCHAR *dst, int dstlen)
// {
//     int pos;
//     for (pos = 0; srclen; src++, srclen--, pos++)
//     {
//         /*
//          * U+3041 ... U+3093: Hiragana
//          * U+3094: Hiragana Letter VU
//          * U+3095: Hiragana Letter Small KA
//          * U+309D: Hiragana Iteration Mark
//          * U+309E: Hiragana Voiced Iteration Mark
//          */
//         WCHAR wch = *src;
//         if ((0x3041 <= wch && wch <= 0x3093) ||
//             wch == 3094 || wch == 0x3095 || wch == 0x309D || wch == 0x309E)
//         {
//             wch += 0x60; /* Hiragana to Katakana */
//         }
//         if (pos < dstlen)
//             dst[pos] = wch;
//     }
//     return pos;
// }
//
// /* The table that contains fullwidth characters and halfwidth characters */
// typedef WCHAR FULL2HALF_ENTRY[3];
// static const FULL2HALF_ENTRY full2half_table[] =
// {
// #define DEFINE_FULL2HALF(full, half1, half2) { full, half1, half2 },
// #include "full2half.h"
// #undef DEFINE_FULL2HALF
// };
// #define GET_FULL(table, index)  ((table)[index][0])
// #define GET_HALF1(table, index) ((table)[index][1])
// #define GET_HALF2(table, index) ((table)[index][2])
//
// /* The table that contains dakuten entries */
// typedef WCHAR DAKUTEN_ENTRY[3];
// static const DAKUTEN_ENTRY dakuten_table[] =
// {
// #define DEFINE_DAKUTEN(voiced, single1, single2, half1, half2) { voiced, single1, single2 },
// #include "dakuten.h"
// #undef DEFINE_DAKUTEN
// };
// #define GET_VOICED(table, index) ((table)[index][0])
// #define GET_SINGLE1(table, index) ((table)[index][1])
// #define GET_SINGLE2(table, index) ((table)[index][2])
//
// static int map_to_halfwidth(DWORD flags, const WCHAR *src, int srclen, WCHAR *dst, int dstlen)
// {
//     int pos, i;
//     const int count1 = (int)ARRAY_SIZE(full2half_table);
//     const FULL2HALF_ENTRY *table1 = full2half_table;
//
//     for (pos = 0; srclen; src++, srclen--, pos++)
//     {
//         WCHAR ch = *src;
//
//         if (flags & LCMAP_KATAKANA)
//             map_to_katakana(&ch, 1, &ch, 1);
//         else if (flags & LCMAP_HIRAGANA)
//             map_to_hiragana(&ch, 1, &ch, 1);
//
//         if (ch < 0x3000) /* Quick judgment */
//         {
//             if (pos < dstlen)
//                 dst[pos] = ch;
//             continue;
//         }
//
//         if (0xFF01 <= ch && ch <= 0xFF5E) /* U+FF01 ... U+FF5E */
//         {
//             if (pos < dstlen)
//                 dst[pos] = ch - 0xFEE0; /* Fullwidth ASCII to halfwidth ASCII */
//             continue;
//         }
//
//         /* Search in table1 (full/half) */
//         for (i = count1 - 1; i >= 0; --i) /* In reverse order */
//         {
//             if (GET_FULL(table1, i) != ch)
//                 continue;
//
//             if (GET_HALF2(table1, i) == 0)
//             {
//                 if (pos < dstlen)
//                     dst[pos] = GET_HALF1(table1, i);
//             }
//             else if (!dstlen)
//             {
//                 pos++;
//             }
//             else if (pos + 1 < dstlen)
//             {
//                 dst[pos++] = GET_HALF1(table1, i);
//                 dst[pos  ] = GET_HALF2(table1, i);
//             }
//             else
//             {
//                 dst[pos] = ch;
//             }
//             break;
//         }
//
//         if (i >= 0)
//             continue;
//
//         if (pos < dstlen)
//             dst[pos] = ch;
//     }
//
//     return pos;
// }
//
// static int map_to_fullwidth(const WCHAR *src, int srclen, WCHAR *dst, int dstlen)
// {
//     int pos, i;
//     const FULL2HALF_ENTRY *table1 = full2half_table;
//     const DAKUTEN_ENTRY *table2 = dakuten_table;
//     const int count1 = (int)ARRAY_SIZE(full2half_table);
//     const int count2 = (int)ARRAY_SIZE(dakuten_table);
//
//     for (pos = 0; srclen; src++, srclen--, pos++)
//     {
//         WCHAR ch = *src;
//
//         if (ch == 0x20) /* U+0020: Space */
//         {
//             if (pos < dstlen)
//                 dst[pos] = 0x3000; /* U+3000: Ideographic Space */
//             continue;
//         }
//
//         if (0x21 <= ch && ch <= 0x7E) /* Mappable halfwidth ASCII */
//         {
//             if (pos < dstlen)
//                 dst[pos] = ch + 0xFEE0; /* U+FF01 ... U+FF5E */
//             continue;
//         }
//
//         if (ch < 0xFF00) /* Quick judgment */
//         {
//             if (pos < dstlen)
//                 dst[pos] = ch;
//             continue;
//         }
//
//         /* Search in table1 (full/half) */
//         for (i = count1 - 1; i >= 0; --i) /* In reverse order */
//         {
//             if (GET_HALF1(table1, i) != ch)
//                 continue; /* Mismatched */
//
//             if (GET_HALF2(table1, i) == 0)
//             {
//                 if (pos < dstlen)
//                     dst[pos] = GET_FULL(table1, i);
//                 break;
//             }
//
//             if (srclen <= 1 || GET_HALF2(table1, i) != src[1])
//                 continue; /* Mismatched */
//
//             --srclen;
//             ++src;
//
//             if (pos < dstlen)
//                 dst[pos] = GET_FULL(table1, i);
//             break;
//         }
//
//         if (i >= 0)
//             continue;
//
//         /* Search in table2 (dakuten) */
//         for (i = count2 - 1; i >= 0; --i) /* In reverse order */
//         {
//             if (GET_SINGLE1(table2, i) != ch)
//                 continue; /* Mismatched */
//
//             if (srclen <= 1 || GET_SINGLE2(table2, i) != src[1])
//                 continue; /* Mismatched */
//
//             --srclen;
//             ++src;
//
//             if (pos < dstlen)
//                 dst[pos] = GET_VOICED(table2, i);
//             break;
//         }
//
//         if (i >= 0)
//             continue;
//
//         if (pos < dstlen)
//             dst[pos] = ch;
//     }
//
//     return pos;
// }
//
// static int map_to_lowercase(DWORD flags, const WCHAR *src, int srclen, WCHAR *dst, int dstlen)
// {
//     int pos;
//     for (pos = 0; srclen; src++, srclen--)
//     {
//         WCHAR wch = *src;
//         if ((flags & NORM_IGNORESYMBOLS) && (get_char_typeW(wch) & (C1_PUNCT | C1_SPACE)))
//             continue;
//         if (pos < dstlen)
//             dst[pos] = tolowerW(wch);
//         pos++;
//     }
//     return pos;
// }
//
// static int map_to_uppercase(DWORD flags, const WCHAR *src, int srclen, WCHAR *dst, int dstlen)
// {
//     int pos;
//     for (pos = 0; srclen; src++, srclen--)
//     {
//         WCHAR wch = *src;
//         if ((flags & NORM_IGNORESYMBOLS) && (get_char_typeW(wch) & (C1_PUNCT | C1_SPACE)))
//             continue;
//         if (pos < dstlen)
//             dst[pos] = toupperW(wch);
//         pos++;
//     }
//     return pos;
// }
//
// typedef struct tagWCHAR_PAIR
// {
//     WCHAR from, to;
// } WCHAR_PAIR, *PWCHAR_PAIR;
//
// /* The table to convert Simplified Chinese to Traditional Chinese */
// static const WCHAR_PAIR s_sim2tra[] =
// {
// #define DEFINE_SIM2TRA(from, to) { from, to },
// #include "sim2tra.h"
// #undef DEFINE_SIM2TRA
// };
//
// /* The table to convert Traditional Chinese to Simplified Chinese */
// static const WCHAR_PAIR s_tra2sim[] =
// {
// #define DEFINE_TRA2SIM(from, to) { from, to },
// #include "tra2sim.h"
// #undef DEFINE_TRA2SIM
// };
//
// /* The comparison function to do bsearch */
// static int compare_wchar_pair(const void *x, const void *y)
// {
//     const WCHAR_PAIR *a = x;
//     const WCHAR_PAIR *b = y;
//     if (a->from < b->from)
//         return -1;
//     if (a->from > b->from)
//         return +1;
//     return 0;
// }
//
// static WCHAR find_wchar_pair(const WCHAR_PAIR *pairs, size_t count, WCHAR ch)
// {
//     PWCHAR_PAIR found = bsearch(&ch, pairs, count, sizeof(WCHAR_PAIR), compare_wchar_pair);
//     if (found)
//         return found->to;
//     return ch;
// }
//
// static int map_to_simplified_chinese(DWORD flags, const WCHAR *src, int srclen, WCHAR *dst, int dstlen)
// {
//     int pos;
//     for (pos = 0; srclen; src++, srclen--)
//     {
//         WCHAR wch = *src;
//         if (pos < dstlen)
//             dst[pos] = find_wchar_pair(s_tra2sim, ARRAY_SIZE(s_tra2sim), wch);
//         pos++;
//     }
//     return pos;
// }
//
// static int map_to_traditional_chinese(DWORD flags, const WCHAR *src, int srclen, WCHAR *dst, int dstlen)
// {
//     int pos;
//     for (pos = 0; srclen; src++, srclen--)
//     {
//         WCHAR wch = *src;
//         if (pos < dstlen)
//             dst[pos] = find_wchar_pair(s_sim2tra, ARRAY_SIZE(s_sim2tra), wch);
//         pos++;
//     }
//     return pos;
// }
//
// static int map_remove_ignored(DWORD flags, const WCHAR *src, int srclen, WCHAR *dst, int dstlen)
// {
//     int pos;
//     WORD wC1, wC2, wC3;
//     for (pos = 0; srclen; src++, srclen--)
//     {
//         WCHAR wch = *src;
//         GetStringTypeW(CT_CTYPE1, &wch, 1, &wC1);
//         GetStringTypeW(CT_CTYPE2, &wch, 1, &wC2);
//         GetStringTypeW(CT_CTYPE3, &wch, 1, &wC3);
//         if (flags & NORM_IGNORESYMBOLS)
//         {
//             if ((wC1 & C1_PUNCT) || (wC3 & C3_SYMBOL))
//                 continue;
//         }
//         if (flags & NORM_IGNORENONSPACE)
//         {
//             if ((wC2 & C2_OTHERNEUTRAL) && (wC3 & (C3_NONSPACING | C3_DIACRITIC)))
//                 continue;
//         }
//         if (pos < dstlen)
//             dst[pos] = wch;
//         pos++;
//     }
//     return pos;
// }
//
// static int lcmap_string(DWORD flags, const WCHAR *src, int srclen, WCHAR *dst, int dstlen)
// {
//     int ret = 0;
//
//     if ((flags & (LCMAP_LOWERCASE | LCMAP_UPPERCASE)) == (LCMAP_LOWERCASE | LCMAP_UPPERCASE))
//     {
//         SetLastError(ERROR_INVALID_FLAGS);
//         return 0;
//     }
//
//     switch (flags & ~(LCMAP_BYTEREV | LCMAP_LOWERCASE | LCMAP_UPPERCASE | LCMAP_LINGUISTIC_CASING))
//     {
//     case LCMAP_HIRAGANA:
//         ret = map_to_hiragana(src, srclen, dst, dstlen);
//         break;
//     case LCMAP_KATAKANA:
//         ret = map_to_katakana(src, srclen, dst, dstlen);
//         break;
//     case LCMAP_HALFWIDTH:
//         ret = map_to_halfwidth(flags, src, srclen, dst, dstlen);
//         break;
//     case LCMAP_HIRAGANA | LCMAP_HALFWIDTH:
//         ret = map_to_halfwidth(flags, src, srclen, dst, dstlen);
//         break;
//     case LCMAP_KATAKANA | LCMAP_HALFWIDTH:
//         ret = map_to_halfwidth(flags, src, srclen, dst, dstlen);
//         break;
//     case LCMAP_FULLWIDTH:
//         ret = map_to_fullwidth(src, srclen, dst, dstlen);
//         break;
//     case LCMAP_HIRAGANA | LCMAP_FULLWIDTH:
//         ret = map_to_fullwidth(src, srclen, dst, dstlen);
//         if (dstlen && ret)
//             map_to_hiragana(dst, ret, dst, dstlen);
//         break;
//     case LCMAP_KATAKANA | LCMAP_FULLWIDTH:
//         ret = map_to_fullwidth(src, srclen, dst, dstlen);
//         if (dstlen && ret)
//             map_to_katakana(dst, ret, dst, dstlen);
//         break;
//     case LCMAP_SIMPLIFIED_CHINESE:
//         ret = map_to_simplified_chinese(flags, src, srclen, dst, dstlen);
//         break;
//     case LCMAP_TRADITIONAL_CHINESE:
//         ret = map_to_traditional_chinese(flags, src, srclen, dst, dstlen);
//         break;
//     case NORM_IGNORENONSPACE:
//     case NORM_IGNORESYMBOLS:
//     case NORM_IGNORENONSPACE | NORM_IGNORESYMBOLS:
//         if (flags & ~(NORM_IGNORENONSPACE | NORM_IGNORESYMBOLS | LCMAP_BYTEREV))
//         {
//             SetLastError(ERROR_INVALID_FLAGS);
//             return 0;
//         }
//         ret = map_remove_ignored(flags, src, srclen, dst, dstlen);
//         break;
//     case 0:
//         if (flags & LCMAP_LOWERCASE)
//         {
//             ret = map_to_lowercase(flags, src, srclen, dst, dstlen);
//             flags &= ~LCMAP_LOWERCASE;
//             break;
//         }
//         if (flags & LCMAP_UPPERCASE)
//         {
//             ret = map_to_uppercase(flags, src, srclen, dst, dstlen);
//             flags &= ~LCMAP_UPPERCASE;
//             break;
//         }
//         if (flags & LCMAP_BYTEREV)
//         {
//             if (dstlen == 0)
//             {
//                 ret = srclen;
//                 break;
//             }
//             ret = min(srclen, dstlen);
//             RtlCopyMemory(dst, src, ret * sizeof(WCHAR));
//             break;
//         }
//         /* fall through */
//     default:
//         SetLastError(ERROR_INVALID_FLAGS);
//         return 0;
//     }
//
//     if (dstlen)
//     {
//         if (flags & LCMAP_LOWERCASE)
//             map_to_lowercase(flags, dst, ret, dst, dstlen);
//         if (flags & LCMAP_UPPERCASE)
//             map_to_uppercase(flags, dst, ret, dst, dstlen);
//         if (flags & LCMAP_BYTEREV)
//             map_byterev(dst, min(ret, dstlen), dst);
//
//         if (dstlen < ret)
//         {
//             SetLastError(ERROR_INSUFFICIENT_BUFFER);
//             return 0;
//         }
//     }
//
//     return ret;
// }
//
// /*************************************************************************
//  *           LCMapStringEx   (KERNEL32.@)
//  *
//  * Map characters in a locale sensitive string.
//  *
//  * PARAMS
//  *  locale   [I] Locale name for the conversion.
//  *  flags    [I] Flags controlling the mapping (LCMAP_ constants from "winnls.h")
//  *  src      [I] String to map
//  *  srclen   [I] Length of src in chars, or -1 if src is NUL terminated
//  *  dst      [O] Destination for mapped string
//  *  dstlen   [I] Length of dst in characters
//  *  version  [I] reserved, must be NULL
//  *  reserved [I] reserved, must be NULL
//  *  lparam   [I] reserved, must be 0
//  *
//  * RETURNS
//  *  Success: The length of the mapped string in dst, including the NUL terminator.
//  *  Failure: 0. Use GetLastError() to determine the cause.
//  */
// INT WINAPI LCMapStringEx(LPCWSTR locale, DWORD flags, LPCWSTR src, INT srclen, LPWSTR dst, INT dstlen,
//                          LPNLSVERSIONINFO version, LPVOID reserved, LPARAM handle)
// {
//     if (version) FIXME("unsupported version structure %p\n", version);
//     if (reserved) FIXME("unsupported reserved pointer %p\n", reserved);
//     if (handle)
//     {
//         static int once;
//         if (!once++) FIXME("unsupported lparam %Ix\n", handle);
//     }
//
//     if (!src || !srclen || dstlen < 0)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     if (srclen < 0) srclen = lstrlenW(src) + 1;
//
//     TRACE( "(%s,0x%08lx,%s,%d,%p,%d)\n",
//            debugstr_w(locale), flags, debugstr_wn(src, srclen), srclen, dst, dstlen );
//
//     flags &= ~LOCALE_USE_CP_ACP;
//
//     if (src == dst && (flags & ~(LCMAP_LOWERCASE | LCMAP_UPPERCASE)))
//     {
//         SetLastError(ERROR_INVALID_FLAGS);
//         return 0;
//     }
//
//     if (!dstlen) dst = NULL;
//
//     if (flags & LCMAP_SORTKEY)
//     {
//         INT ret;
//
//         if (srclen < 0)
//             srclen = strlenW(src);
//
//         ret = wine_get_sortkey(flags, src, srclen, (char *)dst, dstlen);
//         if (ret == 0)
//             SetLastError(ERROR_INSUFFICIENT_BUFFER);
//         else
//             ret++;
//         return ret;
//     }
//
//     /* SORT_STRINGSORT must be used exclusively with LCMAP_SORTKEY */
//     if (flags & SORT_STRINGSORT)
//     {
//         SetLastError(ERROR_INVALID_FLAGS);
//         return 0;
//     }
//
//     return lcmap_string(flags, src, srclen, dst, dstlen);
// }
//
// /*************************************************************************
//  *           LCMapStringW    (KERNEL32.@)
//  *
//  * See LCMapStringA.
//  */
// INT WINAPI LCMapStringW(LCID lcid, DWORD flags, LPCWSTR src, INT srclen,
//                         LPWSTR dst, INT dstlen)
// {
//     TRACE("(0x%04x,0x%08x,%s,%d,%p,%d)\n",
//           lcid, flags, debugstr_wn(src, srclen), srclen, dst, dstlen);
//
//     return LCMapStringEx(NULL, flags, src, srclen, dst, dstlen, NULL, NULL, 0);
// }
//
// /*************************************************************************
//  *           LCMapStringA    (KERNEL32.@)
//  *
//  * Map characters in a locale sensitive string.
//  *
//  * PARAMS
//  *  lcid   [I] LCID for the conversion.
//  *  flags  [I] Flags controlling the mapping (LCMAP_ constants from "winnls.h").
//  *  src    [I] String to map
//  *  srclen [I] Length of src in chars, or -1 if src is NUL terminated
//  *  dst    [O] Destination for mapped string
//  *  dstlen [I] Length of dst in characters
//  *
//  * RETURNS
//  *  Success: The length of the mapped string in dst, including the NUL terminator.
//  *  Failure: 0. Use GetLastError() to determine the cause.
//  */
// INT WINAPI LCMapStringA(LCID lcid, DWORD flags, LPCSTR src, INT srclen,
//                         LPSTR dst, INT dstlen)
// {
//     WCHAR *bufW = NtCurrentTeb()->StaticUnicodeBuffer;
//     LPWSTR srcW, dstW;
//     INT ret = 0, srclenW, dstlenW;
//     UINT locale_cp = CP_ACP;
//
//     if (!src || !srclen || dstlen < 0)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     if (!(flags & LOCALE_USE_CP_ACP)) locale_cp = get_lcid_codepage( lcid );
//
//     srclenW = MultiByteToWideChar(locale_cp, 0, src, srclen, bufW, 260);
//     if (srclenW)
//         srcW = bufW;
//     else
//     {
//         srclenW = MultiByteToWideChar(locale_cp, 0, src, srclen, NULL, 0);
//         srcW = HeapAlloc(GetProcessHeap(), 0, srclenW * sizeof(WCHAR));
//         if (!srcW)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             return 0;
//         }
//         MultiByteToWideChar(locale_cp, 0, src, srclen, srcW, srclenW);
//     }
//
//     if (flags & LCMAP_SORTKEY)
//     {
//         if (src == dst)
//         {
//             SetLastError(ERROR_INVALID_FLAGS);
//             goto map_string_exit;
//         }
//         ret = wine_get_sortkey(flags, srcW, srclenW, dst, dstlen);
//         if (ret == 0)
//             SetLastError(ERROR_INSUFFICIENT_BUFFER);
//         else
//             ret++;
//         goto map_string_exit;
//     }
//
//     if (flags & SORT_STRINGSORT)
//     {
//         SetLastError(ERROR_INVALID_FLAGS);
//         goto map_string_exit;
//     }
//
//     dstlenW = LCMapStringEx(NULL, flags, srcW, srclenW, NULL, 0, NULL, NULL, 0);
//     if (!dstlenW)
//         goto map_string_exit;
//
//     dstW = HeapAlloc(GetProcessHeap(), 0, dstlenW * sizeof(WCHAR));
//     if (!dstW)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         goto map_string_exit;
//     }
//
//     LCMapStringEx(NULL, flags, srcW, srclenW, dstW, dstlenW, NULL, NULL, 0);
//     ret = WideCharToMultiByte(locale_cp, 0, dstW, dstlenW, dst, dstlen, NULL, NULL);
//     HeapFree(GetProcessHeap(), 0, dstW);
//
// map_string_exit:
//     if (srcW != bufW) HeapFree(GetProcessHeap(), 0, srcW);
//     return ret;
// }
//
// /*************************************************************************
//  *           FoldStringA    (KERNEL32.@)
//  *
//  * Map characters in a string.
//  *
//  * PARAMS
//  *  dwFlags [I] Flags controlling chars to map (MAP_ constants from "winnls.h")
//  *  src     [I] String to map
//  *  srclen  [I] Length of src, or -1 if src is NUL terminated
//  *  dst     [O] Destination for mapped string
//  *  dstlen  [I] Length of dst, or 0 to find the required length for the mapped string
//  *
//  * RETURNS
//  *  Success: The length of the string written to dst, including the terminating NUL. If
//  *           dstlen is 0, the value returned is the same, but nothing is written to dst,
//  *           and dst may be NULL.
//  *  Failure: 0. Use GetLastError() to determine the cause.
//  */
// INT WINAPI FoldStringA(DWORD dwFlags, LPCSTR src, INT srclen,
//                        LPSTR dst, INT dstlen)
// {
//     INT ret = 0, srclenW = 0;
//     WCHAR *srcW = NULL, *dstW = NULL;
//
//     if (!src || !srclen || dstlen < 0 || (dstlen && !dst) || src == dst)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     srclenW = MultiByteToWideChar(CP_ACP, dwFlags & MAP_COMPOSITE ? MB_COMPOSITE : 0,
//                                   src, srclen, NULL, 0);
//     srcW = HeapAlloc(GetProcessHeap(), 0, srclenW * sizeof(WCHAR));
//
//     if (!srcW)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         goto FoldStringA_exit;
//     }
//
//     MultiByteToWideChar(CP_ACP, dwFlags & MAP_COMPOSITE ? MB_COMPOSITE : 0,
//                         src, srclen, srcW, srclenW);
//
//     dwFlags = (dwFlags & ~MAP_PRECOMPOSED) | MAP_FOLDCZONE;
//
//     ret = FoldStringW(dwFlags, srcW, srclenW, NULL, 0);
//     if (ret && dstlen)
//     {
//         dstW = HeapAlloc(GetProcessHeap(), 0, ret * sizeof(WCHAR));
//
//         if (!dstW)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto FoldStringA_exit;
//         }
//
//         ret = FoldStringW(dwFlags, srcW, srclenW, dstW, ret);
//         if (!WideCharToMultiByte(CP_ACP, 0, dstW, ret, dst, dstlen, NULL, NULL))
//         {
//             ret = 0;
//             SetLastError(ERROR_INSUFFICIENT_BUFFER);
//         }
//     }
//
//     HeapFree(GetProcessHeap(), 0, dstW);
//
// FoldStringA_exit:
//     HeapFree(GetProcessHeap(), 0, srcW);
//     return ret;
// }
//
// /*************************************************************************
//  *           FoldStringW    (KERNEL32.@)
//  *
//  * See FoldStringA.
//  */
// INT WINAPI FoldStringW(DWORD dwFlags, LPCWSTR src, INT srclen,
//                        LPWSTR dst, INT dstlen)
// {
//     int ret;
//
//     switch (dwFlags & (MAP_COMPOSITE|MAP_PRECOMPOSED|MAP_EXPAND_LIGATURES))
//     {
//     case 0:
//         if (dwFlags)
//           break;
//         /* Fall through for dwFlags == 0 */
//     case MAP_PRECOMPOSED|MAP_COMPOSITE:
//     case MAP_PRECOMPOSED|MAP_EXPAND_LIGATURES:
//     case MAP_COMPOSITE|MAP_EXPAND_LIGATURES:
//         SetLastError(ERROR_INVALID_FLAGS);
//         return 0;
//     }
//
//     if (!src || !srclen || dstlen < 0 || (dstlen && !dst) || src == dst)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     ret = wine_fold_string(dwFlags, src, srclen, dst, dstlen);
//     if (!ret)
//         SetLastError(ERROR_INSUFFICIENT_BUFFER);
//     return ret;
// }
//
// /******************************************************************************
//  *           CompareStringW    (KERNEL32.@)
//  *
//  * See CompareStringA.
//  */
// INT WINAPI CompareStringW(LCID lcid, DWORD flags,
//                           LPCWSTR str1, INT len1, LPCWSTR str2, INT len2)
// {
//     return CompareStringEx(NULL, flags, str1, len1, str2, len2, NULL, NULL, 0);
// }
//
// /******************************************************************************
//  *           CompareStringEx    (KERNEL32.@)
//  */
// INT WINAPI CompareStringEx(LPCWSTR locale, DWORD flags, LPCWSTR str1, INT len1,
//                            LPCWSTR str2, INT len2, LPNLSVERSIONINFO version, LPVOID reserved, LPARAM lParam)
// {
//     DWORD supported_flags = NORM_IGNORECASE|NORM_IGNORENONSPACE|NORM_IGNORESYMBOLS|SORT_STRINGSORT
//                            |NORM_IGNOREKANATYPE|NORM_IGNOREWIDTH|LOCALE_USE_CP_ACP;
//     DWORD semistub_flags = NORM_LINGUISTIC_CASING|LINGUISTIC_IGNORECASE|0x10000000;
//     /* 0x10000000 is related to diacritics in Arabic, Japanese, and Hebrew */
//     INT ret;
//     static int once;
//
//     if (version) FIXME("unexpected version parameter\n");
//     if (reserved) FIXME("unexpected reserved value\n");
//     if (lParam) FIXME("unexpected lParam\n");
//
//     if (!str1 || !str2)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     if (flags & ~(supported_flags|semistub_flags))
//     {
//         SetLastError(ERROR_INVALID_FLAGS);
//         return 0;
//     }
//
//     if (flags & semistub_flags)
//     {
//         if (!once++)
//             FIXME("semi-stub behavior for flag(s) 0x%x\n", flags & semistub_flags);
//     }
//
//     if (len1 < 0) len1 = strlenW(str1);
//     if (len2 < 0) len2 = strlenW(str2);
//
//     ret = wine_compare_string(flags, str1, len1, str2, len2);
//
//     if (ret) /* need to translate result */
//         return (ret < 0) ? CSTR_LESS_THAN : CSTR_GREATER_THAN;
//     return CSTR_EQUAL;
// }
//
// /******************************************************************************
//  *           CompareStringA    (KERNEL32.@)
//  *
//  * Compare two locale sensitive strings.
//  *
//  * PARAMS
//  *  lcid  [I] LCID for the comparison
//  *  flags [I] Flags for the comparison (NORM_ constants from "winnls.h").
//  *  str1  [I] First string to compare
//  *  len1  [I] Length of str1, or -1 if str1 is NUL terminated
//  *  str2  [I] Second string to compare
//  *  len2  [I] Length of str2, or -1 if str2 is NUL terminated
//  *
//  * RETURNS
//  *  Success: CSTR_LESS_THAN, CSTR_EQUAL or CSTR_GREATER_THAN depending on whether
//  *           str1 is less than, equal to or greater than str2 respectively.
//  *  Failure: FALSE. Use GetLastError() to determine the cause.
//  */
// INT WINAPI CompareStringA(LCID lcid, DWORD flags,
//                           LPCSTR str1, INT len1, LPCSTR str2, INT len2)
// {
//     WCHAR *buf1W = NtCurrentTeb()->StaticUnicodeBuffer;
//     WCHAR *buf2W = buf1W + 130;
//     LPWSTR str1W, str2W;
//     INT len1W = 0, len2W = 0, ret;
//     UINT locale_cp = CP_ACP;
//
//     if (!str1 || !str2)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//     if (len1 < 0) len1 = strlen(str1);
//     if (len2 < 0) len2 = strlen(str2);
//
//     if (!(flags & LOCALE_USE_CP_ACP)) locale_cp = get_lcid_codepage( lcid );
//
//     if (len1)
//     {
//         if (len1 <= 130) len1W = MultiByteToWideChar(locale_cp, 0, str1, len1, buf1W, 130);
//         if (len1W)
//             str1W = buf1W;
//         else
//         {
//             len1W = MultiByteToWideChar(locale_cp, 0, str1, len1, NULL, 0);
//             str1W = HeapAlloc(GetProcessHeap(), 0, len1W * sizeof(WCHAR));
//             if (!str1W)
//             {
//                 SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//                 return 0;
//             }
//             MultiByteToWideChar(locale_cp, 0, str1, len1, str1W, len1W);
//         }
//     }
//     else
//     {
//         len1W = 0;
//         str1W = buf1W;
//     }
//
//     if (len2)
//     {
//         if (len2 <= 130) len2W = MultiByteToWideChar(locale_cp, 0, str2, len2, buf2W, 130);
//         if (len2W)
//             str2W = buf2W;
//         else
//         {
//             len2W = MultiByteToWideChar(locale_cp, 0, str2, len2, NULL, 0);
//             str2W = HeapAlloc(GetProcessHeap(), 0, len2W * sizeof(WCHAR));
//             if (!str2W)
//             {
//                 if (str1W != buf1W) HeapFree(GetProcessHeap(), 0, str1W);
//                 SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//                 return 0;
//             }
//             MultiByteToWideChar(locale_cp, 0, str2, len2, str2W, len2W);
//         }
//     }
//     else
//     {
//         len2W = 0;
//         str2W = buf2W;
//     }
//
//     ret = CompareStringEx(NULL, flags, str1W, len1W, str2W, len2W, NULL, NULL, 0);
//
//     if (str1W != buf1W) HeapFree(GetProcessHeap(), 0, str1W);
//     if (str2W != buf2W) HeapFree(GetProcessHeap(), 0, str2W);
//     return ret;
// }
//
// #if (WINVER >= 0x0600)
// /******************************************************************************
//  *           CompareStringOrdinal    (KERNEL32.@)
//  */
// INT WINAPI CompareStringOrdinal(const WCHAR *str1, INT len1, const WCHAR *str2, INT len2, BOOL ignore_case)
// {
//     int ret;
//
//     if (!str1 || !str2)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//     if (len1 < 0) len1 = strlenW(str1);
//     if (len2 < 0) len2 = strlenW(str2);
//
//     ret = RtlCompareUnicodeStrings( str1, len1, str2, len2, ignore_case );
//     if (ret < 0) return CSTR_LESS_THAN;
//     if (ret > 0) return CSTR_GREATER_THAN;
//     return CSTR_EQUAL;
// }
// #endif // (WINVER >= 0x0600)
//
//
// HANDLE NLS_RegOpenKey(HANDLE hRootKey, LPCWSTR szKeyName)
// {
//     UNICODE_STRING keyName;
//     OBJECT_ATTRIBUTES attr;
//     HANDLE hkey;
//
//     RtlInitUnicodeString( &keyName, szKeyName );
//     InitializeObjectAttributes(&attr, &keyName, OBJ_CASE_INSENSITIVE, hRootKey, NULL);
//
//     if (NtOpenKey( &hkey, KEY_READ, &attr ) != STATUS_SUCCESS)
//         hkey = 0;
//
//     return hkey;
// }
//
// BOOL NLS_RegEnumValue(HANDLE hKey, UINT ulIndex,
//                       LPWSTR szValueName, ULONG valueNameSize,
//                       LPWSTR szValueData, ULONG valueDataSize)
// {
//     BYTE buffer[80];
//     KEY_VALUE_FULL_INFORMATION *info = (KEY_VALUE_FULL_INFORMATION *)buffer;
//     DWORD dwLen;
//
//     if (NtEnumerateValueKey( hKey, ulIndex, KeyValueFullInformation,
//         buffer, sizeof(buffer), &dwLen ) != STATUS_SUCCESS ||
//         info->NameLength > valueNameSize ||
//         info->DataLength > valueDataSize)
//     {
//         return FALSE;
//     }
//
//     TRACE("info->Name %s info->DataLength %d\n", debugstr_w(info->Name), info->DataLength);
//
//     memcpy( szValueName, info->Name, info->NameLength);
//     szValueName[info->NameLength / sizeof(WCHAR)] = '\0';
//     memcpy( szValueData, buffer + info->DataOffset, info->DataLength );
//     szValueData[info->DataLength / sizeof(WCHAR)] = '\0';
//
//     TRACE("returning %s %s\n", debugstr_w(szValueName), debugstr_w(szValueData));
//     return TRUE;
// }
//
// static BOOL NLS_RegGetDword(HANDLE hKey, LPCWSTR szValueName, DWORD *lpVal)
// {
//     BYTE buffer[128];
//     const KEY_VALUE_PARTIAL_INFORMATION *info = (KEY_VALUE_PARTIAL_INFORMATION *)buffer;
//     DWORD dwSize = sizeof(buffer);
//     UNICODE_STRING valueName;
//
//     RtlInitUnicodeString( &valueName, szValueName );
//
//     TRACE("%p, %s\n", hKey, debugstr_w(szValueName));
//     if (NtQueryValueKey( hKey, &valueName, KeyValuePartialInformation,
//                          buffer, dwSize, &dwSize ) == STATUS_SUCCESS &&
//         info->DataLength == sizeof(DWORD))
//     {
//         memcpy(lpVal, info->Data, sizeof(DWORD));
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
// static BOOL NLS_GetLanguageGroupName(LGRPID lgrpid, LPWSTR szName, ULONG nameSize)
// {
//     LANGID  langId;
//     LPCWSTR szResourceName = MAKEINTRESOURCEW(((lgrpid + 0x2000) >> 4) + 1);
//     HRSRC   hResource;
//     BOOL    bRet = FALSE;
//
//     /* FIXME: Is it correct to use the system default langid? */
//     langId = GetSystemDefaultLangID();
//
//     if (SUBLANGID(langId) == SUBLANG_NEUTRAL) langId = get_default_sublang( langId );
//
//     hResource = FindResourceExW( kernel32_handle, (LPWSTR)RT_STRING, szResourceName, langId );
//
//     if (hResource)
//     {
//         HGLOBAL hResDir = LoadResource( kernel32_handle, hResource );
//
//         if (hResDir)
//         {
//             ULONG   iResourceIndex = lgrpid & 0xf;
//             LPCWSTR lpResEntry = LockResource( hResDir );
//             ULONG   i;
//
//             for (i = 0; i < iResourceIndex; i++)
//                 lpResEntry += *lpResEntry + 1;
//
//             if (*lpResEntry < nameSize)
//             {
//                 memcpy( szName, lpResEntry + 1, *lpResEntry * sizeof(WCHAR) );
//                 szName[*lpResEntry] = '\0';
//                 bRet = TRUE;
//             }
//
//         }
//         FreeResource( hResource );
//     }
//     return bRet;
// }
//
// /* Callback function ptrs for EnumSystemLanguageGroupsA/W */
// typedef struct
// {
//   LANGUAGEGROUP_ENUMPROCA procA;
//   LANGUAGEGROUP_ENUMPROCW procW;
//   DWORD    dwFlags;
//   LONG_PTR lParam;
// } ENUMLANGUAGEGROUP_CALLBACKS;
//
// /* Internal implementation of EnumSystemLanguageGroupsA/W */
// static BOOL NLS_EnumSystemLanguageGroups(ENUMLANGUAGEGROUP_CALLBACKS *lpProcs)
// {
//     WCHAR szNumber[10], szValue[4];
//     HANDLE hKey;
//     BOOL bContinue = TRUE;
//     ULONG ulIndex = 0;
//
//     if (!lpProcs)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     switch (lpProcs->dwFlags)
//     {
//     case 0:
//         /* Default to LGRPID_INSTALLED */
//         lpProcs->dwFlags = LGRPID_INSTALLED;
//         /* Fall through... */
//     case LGRPID_INSTALLED:
//     case LGRPID_SUPPORTED:
//         break;
//     default:
//         SetLastError(ERROR_INVALID_FLAGS);
//         return FALSE;
//     }
//
//     hKey = NLS_RegOpenKey( 0, szLangGroupsKeyName );
//
//     if (!hKey)
//         FIXME("NLS registry key not found. Please apply the default registry file 'wine.inf'\n");
//
//     while (bContinue)
//     {
//         if (NLS_RegEnumValue( hKey, ulIndex, szNumber, sizeof(szNumber),
//                               szValue, sizeof(szValue) ))
//         {
//             BOOL bInstalled = szValue[0] == '1';
//             LGRPID lgrpid = strtoulW( szNumber, NULL, 16 );
//
//             TRACE("grpid %s (%sinstalled)\n", debugstr_w(szNumber),
//                    bInstalled ? "" : "not ");
//
//             if (lpProcs->dwFlags == LGRPID_SUPPORTED || bInstalled)
//             {
//                 WCHAR szGrpName[48];
//
//                 if (!NLS_GetLanguageGroupName( lgrpid, szGrpName, sizeof(szGrpName) / sizeof(WCHAR) ))
//                     szGrpName[0] = '\0';
//
//                 if (lpProcs->procW)
//                     bContinue = lpProcs->procW( lgrpid, szNumber, szGrpName, lpProcs->dwFlags,
//                                                 lpProcs->lParam );
//                 else
//                 {
//                     char szNumberA[sizeof(szNumber)/sizeof(WCHAR)];
//                     char szGrpNameA[48];
//
//                     /* FIXME: MSDN doesn't say which code page the W->A translation uses,
//                      *        or whether the language names are ever localised. Assume CP_ACP.
//                      */
//
//                     WideCharToMultiByte(CP_ACP, 0, szNumber, -1, szNumberA, sizeof(szNumberA), 0, 0);
//                     WideCharToMultiByte(CP_ACP, 0, szGrpName, -1, szGrpNameA, sizeof(szGrpNameA), 0, 0);
//
//                     bContinue = lpProcs->procA( lgrpid, szNumberA, szGrpNameA, lpProcs->dwFlags,
//                                                 lpProcs->lParam );
//                 }
//             }
//
//             ulIndex++;
//         }
//         else
//             bContinue = FALSE;
//
//         if (!bContinue)
//             break;
//     }
//
//     if (hKey)
//         NtClose( hKey );
//
//     return TRUE;
// }
//
// /******************************************************************************
//  *           EnumSystemLanguageGroupsA    (KERNEL32.@)
//  *
//  * Call a users function for each language group available on the system.
//  *
//  * PARAMS
//  *  pLangGrpEnumProc [I] Callback function to call for each language group
//  *  dwFlags          [I] LGRPID_SUPPORTED=All Supported, LGRPID_INSTALLED=Installed only
//  *  lParam           [I] User parameter to pass to pLangGrpEnumProc
//  *
//  * RETURNS
//  *  Success: TRUE.
//  *  Failure: FALSE. Use GetLastError() to determine the cause.
//  */
// BOOL WINAPI EnumSystemLanguageGroupsA(LANGUAGEGROUP_ENUMPROCA pLangGrpEnumProc,
//                                       DWORD dwFlags, LONG_PTR lParam)
// {
//     ENUMLANGUAGEGROUP_CALLBACKS procs;
//
//     TRACE("(%p,0x%08X,0x%08lX)\n", pLangGrpEnumProc, dwFlags, lParam);
//
//     procs.procA = pLangGrpEnumProc;
//     procs.procW = NULL;
//     procs.dwFlags = dwFlags;
//     procs.lParam = lParam;
//
//     return NLS_EnumSystemLanguageGroups( pLangGrpEnumProc ? &procs : NULL);
// }
//
// /******************************************************************************
//  *           EnumSystemLanguageGroupsW    (KERNEL32.@)
//  *
//  * See EnumSystemLanguageGroupsA.
//  */
// BOOL WINAPI EnumSystemLanguageGroupsW(LANGUAGEGROUP_ENUMPROCW pLangGrpEnumProc,
//                                       DWORD dwFlags, LONG_PTR lParam)
// {
//     ENUMLANGUAGEGROUP_CALLBACKS procs;
//
//     TRACE("(%p,0x%08X,0x%08lX)\n", pLangGrpEnumProc, dwFlags, lParam);
//
//     procs.procA = NULL;
//     procs.procW = pLangGrpEnumProc;
//     procs.dwFlags = dwFlags;
//     procs.lParam = lParam;
//
//     return NLS_EnumSystemLanguageGroups( pLangGrpEnumProc ? &procs : NULL);
// }
//
// /******************************************************************************
//  *           IsValidLanguageGroup    (KERNEL32.@)
//  *
//  * Determine if a language group is supported and/or installed.
//  *
//  * PARAMS
//  *  lgrpid  [I] Language Group Id (LGRPID_ values from "winnls.h")
//  *  dwFlags [I] LGRPID_SUPPORTED=Supported, LGRPID_INSTALLED=Installed
//  *
//  * RETURNS
//  *  TRUE, if lgrpid is supported and/or installed, according to dwFlags.
//  *  FALSE otherwise.
//  */
// BOOL WINAPI IsValidLanguageGroup(LGRPID lgrpid, DWORD dwFlags)
// {
//     static const WCHAR szFormat[] = { '%','x','\0' };
//     WCHAR szValueName[16], szValue[2];
//     BOOL bSupported = FALSE, bInstalled = FALSE;
//     HANDLE hKey;
//
//
//     switch (dwFlags)
//     {
//     case LGRPID_INSTALLED:
//     case LGRPID_SUPPORTED:
//
//         hKey = NLS_RegOpenKey( 0, szLangGroupsKeyName );
//
//         sprintfW( szValueName, szFormat, lgrpid );
//
//         if (NLS_RegGetDword( hKey, szValueName, (LPDWORD)szValue ))
//         {
//             bSupported = TRUE;
//
//             if (szValue[0] == '1')
//                 bInstalled = TRUE;
//         }
//
//         if (hKey)
//             NtClose( hKey );
//
//         break;
//     }
//
//     if ((dwFlags == LGRPID_SUPPORTED && bSupported) ||
//         (dwFlags == LGRPID_INSTALLED && bInstalled))
//         return TRUE;
//
//     return FALSE;
// }
//
// /* Callback function ptrs for EnumLanguageGrouplocalesA/W */
// typedef struct
// {
//   LANGGROUPLOCALE_ENUMPROCA procA;
//   LANGGROUPLOCALE_ENUMPROCW procW;
//   DWORD    dwFlags;
//   LGRPID   lgrpid;
//   LONG_PTR lParam;
// } ENUMLANGUAGEGROUPLOCALE_CALLBACKS;
//
// /* Internal implementation of EnumLanguageGrouplocalesA/W */
// static BOOL NLS_EnumLanguageGroupLocales(ENUMLANGUAGEGROUPLOCALE_CALLBACKS *lpProcs)
// {
//     static const WCHAR szAlternateSortsKeyName[] = {
//       'A','l','t','e','r','n','a','t','e',' ','S','o','r','t','s','\0'
//     };
//     WCHAR szNumber[10], szValue[4];
//     HANDLE hKey;
//     BOOL bContinue = TRUE, bAlternate = FALSE;
//     LGRPID lgrpid;
//     ULONG ulIndex = 1;  /* Ignore default entry of 1st key */
//
//     if (!lpProcs || !lpProcs->lgrpid || lpProcs->lgrpid > LGRPID_ARMENIAN)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     if (lpProcs->dwFlags)
//     {
//         SetLastError(ERROR_INVALID_FLAGS);
//         return FALSE;
//     }
//
//     hKey = NLS_RegOpenKey( 0, szLocaleKeyName );
//
//     if (!hKey)
//         WARN("NLS registry key not found. Please apply the default registry file 'wine.inf'\n");
//
//     while (bContinue)
//     {
//         if (NLS_RegEnumValue( hKey, ulIndex, szNumber, sizeof(szNumber),
//                               szValue, sizeof(szValue) ))
//         {
//             lgrpid = strtoulW( szValue, NULL, 16 );
//
//             TRACE("lcid %s, grpid %d (%smatched)\n", debugstr_w(szNumber),
//                    lgrpid, lgrpid == lpProcs->lgrpid ? "" : "not ");
//
//             if (lgrpid == lpProcs->lgrpid)
//             {
//                 LCID lcid;
//
//                 lcid = strtoulW( szNumber, NULL, 16 );
//
//                 /* FIXME: native returns extra text for a few (17/150) locales, e.g:
//                  * '00000437          ;Georgian'
//                  * At present we only pass the LCID string.
//                  */
//
//                 if (lpProcs->procW)
//                     bContinue = lpProcs->procW( lgrpid, lcid, szNumber, lpProcs->lParam );
//                 else
//                 {
//                     char szNumberA[sizeof(szNumber)/sizeof(WCHAR)];
//
//                     WideCharToMultiByte(CP_ACP, 0, szNumber, -1, szNumberA, sizeof(szNumberA), 0, 0);
//
//                     bContinue = lpProcs->procA( lgrpid, lcid, szNumberA, lpProcs->lParam );
//                 }
//             }
//
//             ulIndex++;
//         }
//         else
//         {
//             /* Finished enumerating this key */
//             if (!bAlternate)
//             {
//                 /* Enumerate alternate sorts also */
//                 hKey = NLS_RegOpenKey( hKey, szAlternateSortsKeyName );
//                 bAlternate = TRUE;
//                 ulIndex = 0;
//             }
//             else
//                 bContinue = FALSE; /* Finished both keys */
//         }
//
//         if (!bContinue)
//             break;
//     }
//
//     if (hKey)
//         NtClose( hKey );
//
//     return TRUE;
// }
//
// /******************************************************************************
//  *           EnumLanguageGroupLocalesA    (KERNEL32.@)
//  *
//  * Call a users function for every locale in a language group available on the system.
//  *
//  * PARAMS
//  *  pLangGrpLcEnumProc [I] Callback function to call for each locale
//  *  lgrpid             [I] Language group (LGRPID_ values from "winnls.h")
//  *  dwFlags            [I] Reserved, set to 0
//  *  lParam             [I] User parameter to pass to pLangGrpLcEnumProc
//  *
//  * RETURNS
//  *  Success: TRUE.
//  *  Failure: FALSE. Use GetLastError() to determine the cause.
//  */
// BOOL WINAPI EnumLanguageGroupLocalesA(LANGGROUPLOCALE_ENUMPROCA pLangGrpLcEnumProc,
//                                       LGRPID lgrpid, DWORD dwFlags, LONG_PTR lParam)
// {
//     ENUMLANGUAGEGROUPLOCALE_CALLBACKS callbacks;
//
//     TRACE("(%p,0x%08X,0x%08X,0x%08lX)\n", pLangGrpLcEnumProc, lgrpid, dwFlags, lParam);
//
//     callbacks.procA   = pLangGrpLcEnumProc;
//     callbacks.procW   = NULL;
//     callbacks.dwFlags = dwFlags;
//     callbacks.lgrpid  = lgrpid;
//     callbacks.lParam  = lParam;
//
//     return NLS_EnumLanguageGroupLocales( pLangGrpLcEnumProc ? &callbacks : NULL );
// }
//
// /******************************************************************************
//  *           EnumLanguageGroupLocalesW    (KERNEL32.@)
//  *
//  * See EnumLanguageGroupLocalesA.
//  */
// BOOL WINAPI EnumLanguageGroupLocalesW(LANGGROUPLOCALE_ENUMPROCW pLangGrpLcEnumProc,
//                                       LGRPID lgrpid, DWORD dwFlags, LONG_PTR lParam)
// {
//     ENUMLANGUAGEGROUPLOCALE_CALLBACKS callbacks;
//
//     TRACE("(%p,0x%08X,0x%08X,0x%08lX)\n", pLangGrpLcEnumProc, lgrpid, dwFlags, lParam);
//
//     callbacks.procA   = NULL;
//     callbacks.procW   = pLangGrpLcEnumProc;
//     callbacks.dwFlags = dwFlags;
//     callbacks.lgrpid  = lgrpid;
//     callbacks.lParam  = lParam;
//
//     return NLS_EnumLanguageGroupLocales( pLangGrpLcEnumProc ? &callbacks : NULL );
// }
//
// /******************************************************************************
//  *           InvalidateNLSCache           (KERNEL32.@)
//  *
//  * Invalidate the cache of NLS values.
//  *
//  * PARAMS
//  *  None.
//  *
//  * RETURNS
//  *  Success: TRUE.
//  *  Failure: FALSE.
//  */
// BOOL WINAPI InvalidateNLSCache(void)
// {
//     JapaneseEra_ClearCache();
//     return TRUE;
// }
//
// /******************************************************************************
//  *           GetUserGeoID (KERNEL32.@)
//  */
// GEOID WINAPI GetUserGeoID( GEOCLASS GeoClass )
// {
//     GEOID ret = GEOID_NOT_AVAILABLE;
//     static const WCHAR geoW[] = {'G','e','o',0};
//     static const WCHAR nationW[] = {'N','a','t','i','o','n',0};
//     WCHAR bufferW[40], *end;
//     DWORD count;
//     HANDLE hkey, hSubkey = 0;
//     UNICODE_STRING keyW;
//     const KEY_VALUE_PARTIAL_INFORMATION *info = (KEY_VALUE_PARTIAL_INFORMATION *)bufferW;
//     RtlInitUnicodeString( &keyW, nationW );
//     count = sizeof(bufferW);
//
//     if(!(hkey = create_registry_key())) return ret;
//
//     switch( GeoClass ){
//     case GEOCLASS_NATION:
//         if ((hSubkey = NLS_RegOpenKey(hkey, geoW)))
//         {
//             if((NtQueryValueKey(hSubkey, &keyW, KeyValuePartialInformation,
//                                 bufferW, count, &count) == STATUS_SUCCESS ) && info->DataLength)
//                 ret = strtolW((LPCWSTR)info->Data, &end, 10);
//         }
//         break;
//     case GEOCLASS_REGION:
//         FIXME("GEOCLASS_REGION not handled yet\n");
//         break;
//     }
//
//     NtClose(hkey);
//     if (hSubkey) NtClose(hSubkey);
//     return ret;
// }
//
// /******************************************************************************
//  *           SetUserGeoID (KERNEL32.@)
//  */
// BOOL WINAPI SetUserGeoID( GEOID GeoID )
// {
//     static const WCHAR geoW[] = {'G','e','o',0};
//     static const WCHAR nationW[] = {'N','a','t','i','o','n',0};
//     static const WCHAR formatW[] = {'%','i',0};
//     UNICODE_STRING nameW,keyW;
//     WCHAR bufferW[10];
//     OBJECT_ATTRIBUTES attr;
//     HANDLE hkey;
//
//     if(!(hkey = create_registry_key())) return FALSE;
//
//     attr.Length = sizeof(attr);
//     attr.RootDirectory = hkey;
//     attr.ObjectName = &nameW;
//     attr.Attributes = 0;
//     attr.SecurityDescriptor = NULL;
//     attr.SecurityQualityOfService = NULL;
//     RtlInitUnicodeString( &nameW, geoW );
//     RtlInitUnicodeString( &keyW, nationW );
//
//     if (NtCreateKey( &hkey, KEY_ALL_ACCESS, &attr, 0, NULL, 0, NULL ) != STATUS_SUCCESS)
//
//     {
//         NtClose(attr.RootDirectory);
//         return FALSE;
//     }
//
//     sprintfW(bufferW, formatW, GeoID);
//     NtSetValueKey(hkey, &keyW, 0, REG_SZ, bufferW, (strlenW(bufferW) + 1) * sizeof(WCHAR));
//     NtClose(attr.RootDirectory);
//     NtClose(hkey);
//     return TRUE;
// }
//
// typedef struct
// {
//     union
//     {
//         UILANGUAGE_ENUMPROCA procA;
//         UILANGUAGE_ENUMPROCW procW;
//     } u;
//     DWORD flags;
//     LONG_PTR param;
// } ENUM_UILANG_CALLBACK;
//
// static BOOL __stdcall enum_uilang_proc_a( HMODULE hModule, LPCSTR type,
//                                          LPCSTR name, WORD LangID, LONG_PTR lParam )
// {
//     ENUM_UILANG_CALLBACK *enum_uilang = (ENUM_UILANG_CALLBACK *)lParam;
//     char buf[20];
//
//     sprintf(buf, "%08x", (UINT)LangID);
//     return enum_uilang->u.procA( buf, enum_uilang->param );
// }
//
// static BOOL __stdcall enum_uilang_proc_w( HMODULE hModule, LPCWSTR type,
//                                          LPCWSTR name, WORD LangID, LONG_PTR lParam )
// {
//     static const WCHAR formatW[] = {'%','0','8','x',0};
//     ENUM_UILANG_CALLBACK *enum_uilang = (ENUM_UILANG_CALLBACK *)lParam;
//     WCHAR buf[20];
//
//     sprintfW( buf, formatW, (UINT)LangID );
//     return enum_uilang->u.procW( buf, enum_uilang->param );
// }
//
// /******************************************************************************
//  *           EnumUILanguagesA (KERNEL32.@)
//  */
// BOOL WINAPI EnumUILanguagesA(UILANGUAGE_ENUMPROCA pUILangEnumProc, DWORD dwFlags, LONG_PTR lParam)
// {
//     ENUM_UILANG_CALLBACK enum_uilang;
//
//     TRACE("%p, %x, %lx\n", pUILangEnumProc, dwFlags, lParam);
//
//     if(!pUILangEnumProc) {
// 	SetLastError(ERROR_INVALID_PARAMETER);
// 	return FALSE;
//     }
//     if(dwFlags) {
// 	SetLastError(ERROR_INVALID_FLAGS);
// 	return FALSE;
//     }
//
//     enum_uilang.u.procA = pUILangEnumProc;
//     enum_uilang.flags = dwFlags;
//     enum_uilang.param = lParam;
//
//     EnumResourceLanguagesA( kernel32_handle, (LPCSTR)RT_STRING,
//                             (LPCSTR)LOCALE_ILANGUAGE, enum_uilang_proc_a,
//                             (LONG_PTR)&enum_uilang);
//     return TRUE;
// }
//
// /******************************************************************************
//  *           EnumUILanguagesW (KERNEL32.@)
//  */
// BOOL WINAPI EnumUILanguagesW(UILANGUAGE_ENUMPROCW pUILangEnumProc, DWORD dwFlags, LONG_PTR lParam)
// {
//     ENUM_UILANG_CALLBACK enum_uilang;
//
//     TRACE("%p, %x, %lx\n", pUILangEnumProc, dwFlags, lParam);
//
//
//     if(!pUILangEnumProc) {
// 	SetLastError(ERROR_INVALID_PARAMETER);
// 	return FALSE;
//     }
//     if(dwFlags) {
// 	SetLastError(ERROR_INVALID_FLAGS);
// 	return FALSE;
//     }
//
//     enum_uilang.u.procW = pUILangEnumProc;
//     enum_uilang.flags = dwFlags;
//     enum_uilang.param = lParam;
//
//     EnumResourceLanguagesW( kernel32_handle, (LPCWSTR)RT_STRING,
//                             (LPCWSTR)LOCALE_ILANGUAGE, enum_uilang_proc_w,
//                             (LONG_PTR)&enum_uilang);
//     return TRUE;
// }
//
// enum locationkind {
//     LOCATION_NATION = 0,
//     LOCATION_REGION,
//     LOCATION_BOTH
// };
//
// struct geoinfo_t {
//     GEOID id;
//     WCHAR iso2W[3];
//     WCHAR iso3W[4];
//     GEOID parent;
//     INT   uncode;
//     enum locationkind kind;
// };
//
// static const struct geoinfo_t geoinfodata[] = {
//     { 2, {'A','G',0}, {'A','T','G',0}, 10039880,  28 }, /* Antigua and Barbuda */
//     { 3, {'A','F',0}, {'A','F','G',0}, 47614,   4 }, /* Afghanistan */
//     { 4, {'D','Z',0}, {'D','Z','A',0}, 42487,  12 }, /* Algeria */
//     { 5, {'A','Z',0}, {'A','Z','E',0}, 47611,  31 }, /* Azerbaijan */
//     { 6, {'A','L',0}, {'A','L','B',0}, 47610,   8 }, /* Albania */
//     { 7, {'A','M',0}, {'A','R','M',0}, 47611,  51 }, /* Armenia */
//     { 8, {'A','D',0}, {'A','N','D',0}, 47610,  20 }, /* Andorra */
//     { 9, {'A','O',0}, {'A','G','O',0}, 42484,  24 }, /* Angola */
//     { 10, {'A','S',0}, {'A','S','M',0}, 26286,  16 }, /* American Samoa */
//     { 11, {'A','R',0}, {'A','R','G',0}, 31396,  32 }, /* Argentina */
//     { 12, {'A','U',0}, {'A','U','S',0}, 10210825,  36 }, /* Australia */
//     { 14, {'A','T',0}, {'A','U','T',0}, 10210824,  40 }, /* Austria */
//     { 17, {'B','H',0}, {'B','H','R',0}, 47611,  48 }, /* Bahrain */
//     { 18, {'B','B',0}, {'B','R','B',0}, 10039880,  52 }, /* Barbados */
//     { 19, {'B','W',0}, {'B','W','A',0}, 10039883,  72 }, /* Botswana */
//     { 20, {'B','M',0}, {'B','M','U',0}, 23581,  60 }, /* Bermuda */
//     { 21, {'B','E',0}, {'B','E','L',0}, 10210824,  56 }, /* Belgium */
//     { 22, {'B','S',0}, {'B','H','S',0}, 10039880,  44 }, /* Bahamas, The */
//     { 23, {'B','D',0}, {'B','G','D',0}, 47614,  50 }, /* Bangladesh */
//     { 24, {'B','Z',0}, {'B','L','Z',0}, 27082,  84 }, /* Belize */
//     { 25, {'B','A',0}, {'B','I','H',0}, 47610,  70 }, /* Bosnia and Herzegovina */
//     { 26, {'B','O',0}, {'B','O','L',0}, 31396,  68 }, /* Bolivia */
//     { 27, {'M','M',0}, {'M','M','R',0}, 47599, 104 }, /* Myanmar */
//     { 28, {'B','J',0}, {'B','E','N',0}, 42483, 204 }, /* Benin */
//     { 29, {'B','Y',0}, {'B','L','R',0}, 47609, 112 }, /* Belarus */
//     { 30, {'S','B',0}, {'S','L','B',0}, 20900,  90 }, /* Solomon Islands */
//     { 32, {'B','R',0}, {'B','R','A',0}, 31396,  76 }, /* Brazil */
//     { 34, {'B','T',0}, {'B','T','N',0}, 47614,  64 }, /* Bhutan */
//     { 35, {'B','G',0}, {'B','G','R',0}, 47609, 100 }, /* Bulgaria */
//     { 37, {'B','N',0}, {'B','R','N',0}, 47599,  96 }, /* Brunei */
//     { 38, {'B','I',0}, {'B','D','I',0}, 47603, 108 }, /* Burundi */
//     { 39, {'C','A',0}, {'C','A','N',0}, 23581, 124 }, /* Canada */
//     { 40, {'K','H',0}, {'K','H','M',0}, 47599, 116 }, /* Cambodia */
//     { 41, {'T','D',0}, {'T','C','D',0}, 42484, 148 }, /* Chad */
//     { 42, {'L','K',0}, {'L','K','A',0}, 47614, 144 }, /* Sri Lanka */
//     { 43, {'C','G',0}, {'C','O','G',0}, 42484, 178 }, /* Congo */
//     { 44, {'C','D',0}, {'C','O','D',0}, 42484, 180 }, /* Congo (DRC) */
//     { 45, {'C','N',0}, {'C','H','N',0}, 47600, 156 }, /* China */
//     { 46, {'C','L',0}, {'C','H','L',0}, 31396, 152 }, /* Chile */
//     { 49, {'C','M',0}, {'C','M','R',0}, 42484, 120 }, /* Cameroon */
//     { 50, {'K','M',0}, {'C','O','M',0}, 47603, 174 }, /* Comoros */
//     { 51, {'C','O',0}, {'C','O','L',0}, 31396, 170 }, /* Colombia */
//     { 54, {'C','R',0}, {'C','R','I',0}, 27082, 188 }, /* Costa Rica */
//     { 55, {'C','F',0}, {'C','A','F',0}, 42484, 140 }, /* Central African Republic */
//     { 56, {'C','U',0}, {'C','U','B',0}, 10039880, 192 }, /* Cuba */
//     { 57, {'C','V',0}, {'C','P','V',0}, 42483, 132 }, /* Cape Verde */
//     { 59, {'C','Y',0}, {'C','Y','P',0}, 47611, 196 }, /* Cyprus */
//     { 61, {'D','K',0}, {'D','N','K',0}, 10039882, 208 }, /* Denmark */
//     { 62, {'D','J',0}, {'D','J','I',0}, 47603, 262 }, /* Djibouti */
//     { 63, {'D','M',0}, {'D','M','A',0}, 10039880, 212 }, /* Dominica */
//     { 65, {'D','O',0}, {'D','O','M',0}, 10039880, 214 }, /* Dominican Republic */
//     { 66, {'E','C',0}, {'E','C','U',0}, 31396, 218 }, /* Ecuador */
//     { 67, {'E','G',0}, {'E','G','Y',0}, 42487, 818 }, /* Egypt */
//     { 68, {'I','E',0}, {'I','R','L',0}, 10039882, 372 }, /* Ireland */
//     { 69, {'G','Q',0}, {'G','N','Q',0}, 42484, 226 }, /* Equatorial Guinea */
//     { 70, {'E','E',0}, {'E','S','T',0}, 10039882, 233 }, /* Estonia */
//     { 71, {'E','R',0}, {'E','R','I',0}, 47603, 232 }, /* Eritrea */
//     { 72, {'S','V',0}, {'S','L','V',0}, 27082, 222 }, /* El Salvador */
//     { 73, {'E','T',0}, {'E','T','H',0}, 47603, 231 }, /* Ethiopia */
//     { 75, {'C','Z',0}, {'C','Z','E',0}, 47609, 203 }, /* Czech Republic */
//     { 77, {'F','I',0}, {'F','I','N',0}, 10039882, 246 }, /* Finland */
//     { 78, {'F','J',0}, {'F','J','I',0}, 20900, 242 }, /* Fiji Islands */
//     { 80, {'F','M',0}, {'F','S','M',0}, 21206, 583 }, /* Micronesia */
//     { 81, {'F','O',0}, {'F','R','O',0}, 10039882, 234 }, /* Faroe Islands */
//     { 84, {'F','R',0}, {'F','R','A',0}, 10210824, 250 }, /* France */
//     { 86, {'G','M',0}, {'G','M','B',0}, 42483, 270 }, /* Gambia, The */
//     { 87, {'G','A',0}, {'G','A','B',0}, 42484, 266 }, /* Gabon */
//     { 88, {'G','E',0}, {'G','E','O',0}, 47611, 268 }, /* Georgia */
//     { 89, {'G','H',0}, {'G','H','A',0}, 42483, 288 }, /* Ghana */
//     { 90, {'G','I',0}, {'G','I','B',0}, 47610, 292 }, /* Gibraltar */
//     { 91, {'G','D',0}, {'G','R','D',0}, 10039880, 308 }, /* Grenada */
//     { 93, {'G','L',0}, {'G','R','L',0}, 23581, 304 }, /* Greenland */
//     { 94, {'D','E',0}, {'D','E','U',0}, 10210824, 276 }, /* Germany */
//     { 98, {'G','R',0}, {'G','R','C',0}, 47610, 300 }, /* Greece */
//     { 99, {'G','T',0}, {'G','T','M',0}, 27082, 320 }, /* Guatemala */
//     { 100, {'G','N',0}, {'G','I','N',0}, 42483, 324 }, /* Guinea */
//     { 101, {'G','Y',0}, {'G','U','Y',0}, 31396, 328 }, /* Guyana */
//     { 103, {'H','T',0}, {'H','T','I',0}, 10039880, 332 }, /* Haiti */
//     { 104, {'H','K',0}, {'H','K','G',0}, 47600, 344 }, /* Hong Kong S.A.R. */
//     { 106, {'H','N',0}, {'H','N','D',0}, 27082, 340 }, /* Honduras */
//     { 108, {'H','R',0}, {'H','R','V',0}, 47610, 191 }, /* Croatia */
//     { 109, {'H','U',0}, {'H','U','N',0}, 47609, 348 }, /* Hungary */
//     { 110, {'I','S',0}, {'I','S','L',0}, 10039882, 352 }, /* Iceland */
//     { 111, {'I','D',0}, {'I','D','N',0}, 47599, 360 }, /* Indonesia */
//     { 113, {'I','N',0}, {'I','N','D',0}, 47614, 356 }, /* India */
//     { 114, {'I','O',0}, {'I','O','T',0}, 39070,  86 }, /* British Indian Ocean Territory */
//     { 116, {'I','R',0}, {'I','R','N',0}, 47614, 364 }, /* Iran */
//     { 117, {'I','L',0}, {'I','S','R',0}, 47611, 376 }, /* Israel */
//     { 118, {'I','T',0}, {'I','T','A',0}, 47610, 380 }, /* Italy */
//     { 119, {'C','I',0}, {'C','I','V',0}, 42483, 384 }, /* Côte d'Ivoire */
//     { 121, {'I','Q',0}, {'I','R','Q',0}, 47611, 368 }, /* Iraq */
//     { 122, {'J','P',0}, {'J','P','N',0}, 47600, 392 }, /* Japan */
//     { 124, {'J','M',0}, {'J','A','M',0}, 10039880, 388 }, /* Jamaica */
//     { 125, {'S','J',0}, {'S','J','M',0}, 10039882, 744 }, /* Jan Mayen */
//     { 126, {'J','O',0}, {'J','O','R',0}, 47611, 400 }, /* Jordan */
//     { 127, {'X','X',0}, {'X','X',0}, 161832256 }, /* Johnston Atoll */
//     { 129, {'K','E',0}, {'K','E','N',0}, 47603, 404 }, /* Kenya */
//     { 130, {'K','G',0}, {'K','G','Z',0}, 47590, 417 }, /* Kyrgyzstan */
//     { 131, {'K','P',0}, {'P','R','K',0}, 47600, 408 }, /* North Korea */
//     { 133, {'K','I',0}, {'K','I','R',0}, 21206, 296 }, /* Kiribati */
//     { 134, {'K','R',0}, {'K','O','R',0}, 47600, 410 }, /* Korea */
//     { 136, {'K','W',0}, {'K','W','T',0}, 47611, 414 }, /* Kuwait */
//     { 137, {'K','Z',0}, {'K','A','Z',0}, 47590, 398 }, /* Kazakhstan */
//     { 138, {'L','A',0}, {'L','A','O',0}, 47599, 418 }, /* Laos */
//     { 139, {'L','B',0}, {'L','B','N',0}, 47611, 422 }, /* Lebanon */
//     { 140, {'L','V',0}, {'L','V','A',0}, 10039882, 428 }, /* Latvia */
//     { 141, {'L','T',0}, {'L','T','U',0}, 10039882, 440 }, /* Lithuania */
//     { 142, {'L','R',0}, {'L','B','R',0}, 42483, 430 }, /* Liberia */
//     { 143, {'S','K',0}, {'S','V','K',0}, 47609, 703 }, /* Slovakia */
//     { 145, {'L','I',0}, {'L','I','E',0}, 10210824, 438 }, /* Liechtenstein */
//     { 146, {'L','S',0}, {'L','S','O',0}, 10039883, 426 }, /* Lesotho */
//     { 147, {'L','U',0}, {'L','U','X',0}, 10210824, 442 }, /* Luxembourg */
//     { 148, {'L','Y',0}, {'L','B','Y',0}, 42487, 434 }, /* Libya */
//     { 149, {'M','G',0}, {'M','D','G',0}, 47603, 450 }, /* Madagascar */
//     { 151, {'M','O',0}, {'M','A','C',0}, 47600, 446 }, /* Macao S.A.R. */
//     { 152, {'M','D',0}, {'M','D','A',0}, 47609, 498 }, /* Moldova */
//     { 154, {'M','N',0}, {'M','N','G',0}, 47600, 496 }, /* Mongolia */
//     { 156, {'M','W',0}, {'M','W','I',0}, 47603, 454 }, /* Malawi */
//     { 157, {'M','L',0}, {'M','L','I',0}, 42483, 466 }, /* Mali */
//     { 158, {'M','C',0}, {'M','C','O',0}, 10210824, 492 }, /* Monaco */
//     { 159, {'M','A',0}, {'M','A','R',0}, 42487, 504 }, /* Morocco */
//     { 160, {'M','U',0}, {'M','U','S',0}, 47603, 480 }, /* Mauritius */
//     { 162, {'M','R',0}, {'M','R','T',0}, 42483, 478 }, /* Mauritania */
//     { 163, {'M','T',0}, {'M','L','T',0}, 47610, 470 }, /* Malta */
//     { 164, {'O','M',0}, {'O','M','N',0}, 47611, 512 }, /* Oman */
//     { 165, {'M','V',0}, {'M','D','V',0}, 47614, 462 }, /* Maldives */
//     { 166, {'M','X',0}, {'M','E','X',0}, 27082, 484 }, /* Mexico */
//     { 167, {'M','Y',0}, {'M','Y','S',0}, 47599, 458 }, /* Malaysia */
//     { 168, {'M','Z',0}, {'M','O','Z',0}, 47603, 508 }, /* Mozambique */
//     { 173, {'N','E',0}, {'N','E','R',0}, 42483, 562 }, /* Niger */
//     { 174, {'V','U',0}, {'V','U','T',0}, 20900, 548 }, /* Vanuatu */
//     { 175, {'N','G',0}, {'N','G','A',0}, 42483, 566 }, /* Nigeria */
//     { 176, {'N','L',0}, {'N','L','D',0}, 10210824, 528 }, /* Netherlands */
//     { 177, {'N','O',0}, {'N','O','R',0}, 10039882, 578 }, /* Norway */
//     { 178, {'N','P',0}, {'N','P','L',0}, 47614, 524 }, /* Nepal */
//     { 180, {'N','R',0}, {'N','R','U',0}, 21206, 520 }, /* Nauru */
//     { 181, {'S','R',0}, {'S','U','R',0}, 31396, 740 }, /* Suriname */
//     { 182, {'N','I',0}, {'N','I','C',0}, 27082, 558 }, /* Nicaragua */
//     { 183, {'N','Z',0}, {'N','Z','L',0}, 10210825, 554 }, /* New Zealand */
//     { 184, {'P','S',0}, {'P','S','E',0}, 47611, 275 }, /* Palestinian Authority */
//     { 185, {'P','Y',0}, {'P','R','Y',0}, 31396, 600 }, /* Paraguay */
//     { 187, {'P','E',0}, {'P','E','R',0}, 31396, 604 }, /* Peru */
//     { 190, {'P','K',0}, {'P','A','K',0}, 47614, 586 }, /* Pakistan */
//     { 191, {'P','L',0}, {'P','O','L',0}, 47609, 616 }, /* Poland */
//     { 192, {'P','A',0}, {'P','A','N',0}, 27082, 591 }, /* Panama */
//     { 193, {'P','T',0}, {'P','R','T',0}, 47610, 620 }, /* Portugal */
//     { 194, {'P','G',0}, {'P','N','G',0}, 20900, 598 }, /* Papua New Guinea */
//     { 195, {'P','W',0}, {'P','L','W',0}, 21206, 585 }, /* Palau */
//     { 196, {'G','W',0}, {'G','N','B',0}, 42483, 624 }, /* Guinea-Bissau */
//     { 197, {'Q','A',0}, {'Q','A','T',0}, 47611, 634 }, /* Qatar */
//     { 198, {'R','E',0}, {'R','E','U',0}, 47603, 638 }, /* Reunion */
//     { 199, {'M','H',0}, {'M','H','L',0}, 21206, 584 }, /* Marshall Islands */
//     { 200, {'R','O',0}, {'R','O','U',0}, 47609, 642 }, /* Romania */
//     { 201, {'P','H',0}, {'P','H','L',0}, 47599, 608 }, /* Philippines */
//     { 202, {'P','R',0}, {'P','R','I',0}, 10039880, 630 }, /* Puerto Rico */
//     { 203, {'R','U',0}, {'R','U','S',0}, 47609, 643 }, /* Russia */
//     { 204, {'R','W',0}, {'R','W','A',0}, 47603, 646 }, /* Rwanda */
//     { 205, {'S','A',0}, {'S','A','U',0}, 47611, 682 }, /* Saudi Arabia */
//     { 206, {'P','M',0}, {'S','P','M',0}, 23581, 666 }, /* St. Pierre and Miquelon */
//     { 207, {'K','N',0}, {'K','N','A',0}, 10039880, 659 }, /* St. Kitts and Nevis */
//     { 208, {'S','C',0}, {'S','Y','C',0}, 47603, 690 }, /* Seychelles */
//     { 209, {'Z','A',0}, {'Z','A','F',0}, 10039883, 710 }, /* South Africa */
//     { 210, {'S','N',0}, {'S','E','N',0}, 42483, 686 }, /* Senegal */
//     { 212, {'S','I',0}, {'S','V','N',0}, 47610, 705 }, /* Slovenia */
//     { 213, {'S','L',0}, {'S','L','E',0}, 42483, 694 }, /* Sierra Leone */
//     { 214, {'S','M',0}, {'S','M','R',0}, 47610, 674 }, /* San Marino */
//     { 215, {'S','G',0}, {'S','G','P',0}, 47599, 702 }, /* Singapore */
//     { 216, {'S','O',0}, {'S','O','M',0}, 47603, 706 }, /* Somalia */
//     { 217, {'E','S',0}, {'E','S','P',0}, 47610, 724 }, /* Spain */
//     { 218, {'L','C',0}, {'L','C','A',0}, 10039880, 662 }, /* St. Lucia */
//     { 219, {'S','D',0}, {'S','D','N',0}, 42487, 736 }, /* Sudan */
//     { 220, {'S','J',0}, {'S','J','M',0}, 10039882, 744 }, /* Svalbard */
//     { 221, {'S','E',0}, {'S','W','E',0}, 10039882, 752 }, /* Sweden */
//     { 222, {'S','Y',0}, {'S','Y','R',0}, 47611, 760 }, /* Syria */
//     { 223, {'C','H',0}, {'C','H','E',0}, 10210824, 756 }, /* Switzerland */
//     { 224, {'A','E',0}, {'A','R','E',0}, 47611, 784 }, /* United Arab Emirates */
//     { 225, {'T','T',0}, {'T','T','O',0}, 10039880, 780 }, /* Trinidad and Tobago */
//     { 227, {'T','H',0}, {'T','H','A',0}, 47599, 764 }, /* Thailand */
//     { 228, {'T','J',0}, {'T','J','K',0}, 47590, 762 }, /* Tajikistan */
//     { 231, {'T','O',0}, {'T','O','N',0}, 26286, 776 }, /* Tonga */
//     { 232, {'T','G',0}, {'T','G','O',0}, 42483, 768 }, /* Togo */
//     { 233, {'S','T',0}, {'S','T','P',0}, 42484, 678 }, /* São Tomé and Príncipe */
//     { 234, {'T','N',0}, {'T','U','N',0}, 42487, 788 }, /* Tunisia */
//     { 235, {'T','R',0}, {'T','U','R',0}, 47611, 792 }, /* Turkey */
//     { 236, {'T','V',0}, {'T','U','V',0}, 26286, 798 }, /* Tuvalu */
//     { 237, {'T','W',0}, {'T','W','N',0}, 47600, 158 }, /* Taiwan */
//     { 238, {'T','M',0}, {'T','K','M',0}, 47590, 795 }, /* Turkmenistan */
//     { 239, {'T','Z',0}, {'T','Z','A',0}, 47603, 834 }, /* Tanzania */
//     { 240, {'U','G',0}, {'U','G','A',0}, 47603, 800 }, /* Uganda */
//     { 241, {'U','A',0}, {'U','K','R',0}, 47609, 804 }, /* Ukraine */
//     { 242, {'G','B',0}, {'G','B','R',0}, 10039882, 826 }, /* United Kingdom */
//     { 244, {'U','S',0}, {'U','S','A',0}, 23581, 840 }, /* United States */
//     { 245, {'B','F',0}, {'B','F','A',0}, 42483, 854 }, /* Burkina Faso */
//     { 246, {'U','Y',0}, {'U','R','Y',0}, 31396, 858 }, /* Uruguay */
//     { 247, {'U','Z',0}, {'U','Z','B',0}, 47590, 860 }, /* Uzbekistan */
//     { 248, {'V','C',0}, {'V','C','T',0}, 10039880, 670 }, /* St. Vincent and the Grenadines */
//     { 249, {'V','E',0}, {'V','E','N',0}, 31396, 862 }, /* Bolivarian Republic of Venezuela */
//     { 251, {'V','N',0}, {'V','N','M',0}, 47599, 704 }, /* Vietnam */
//     { 252, {'V','I',0}, {'V','I','R',0}, 10039880, 850 }, /* Virgin Islands */
//     { 253, {'V','A',0}, {'V','A','T',0}, 47610, 336 }, /* Vatican City */
//     { 254, {'N','A',0}, {'N','A','M',0}, 10039883, 516 }, /* Namibia */
//     { 257, {'E','H',0}, {'E','S','H',0}, 42487, 732 }, /* Western Sahara (disputed) */
//     { 258, {'X','X',0}, {'X','X',0}, 161832256 }, /* Wake Island */
//     { 259, {'W','S',0}, {'W','S','M',0}, 26286, 882 }, /* Samoa */
//     { 260, {'S','Z',0}, {'S','W','Z',0}, 10039883, 748 }, /* Swaziland */
//     { 261, {'Y','E',0}, {'Y','E','M',0}, 47611, 887 }, /* Yemen */
//     { 263, {'Z','M',0}, {'Z','M','B',0}, 47603, 894 }, /* Zambia */
//     { 264, {'Z','W',0}, {'Z','W','E',0}, 47603, 716 }, /* Zimbabwe */
//     { 269, {'C','S',0}, {'S','C','G',0}, 47610, 891 }, /* Serbia and Montenegro (Former) */
//     { 270, {'M','E',0}, {'M','N','E',0}, 47610, 499 }, /* Montenegro */
//     { 271, {'R','S',0}, {'S','R','B',0}, 47610, 688 }, /* Serbia */
//     { 273, {'C','W',0}, {'C','U','W',0}, 10039880, 531 }, /* Curaçao */
//     { 276, {'S','S',0}, {'S','S','D',0}, 42487, 728 }, /* South Sudan */
//     { 300, {'A','I',0}, {'A','I','A',0}, 10039880, 660 }, /* Anguilla */
//     { 301, {'A','Q',0}, {'A','T','A',0}, 39070,  10 }, /* Antarctica */
//     { 302, {'A','W',0}, {'A','B','W',0}, 10039880, 533 }, /* Aruba */
//     { 303, {'X','X',0}, {'X','X',0}, 39070 }, /* Ascension Island */
//     { 304, {'X','X',0}, {'X','X',0}, 10210825 }, /* Ashmore and Cartier Islands */
//     { 305, {'X','X',0}, {'X','X',0}, 161832256 }, /* Baker Island */
//     { 306, {'B','V',0}, {'B','V','T',0}, 39070,  74 }, /* Bouvet Island */
//     { 307, {'K','Y',0}, {'C','Y','M',0}, 10039880, 136 }, /* Cayman Islands */
//     { 308, {'X','X',0}, {'X','X',0}, 10210824, 0, LOCATION_BOTH }, /* Channel Islands */
//     { 309, {'C','X',0}, {'C','X','R',0}, 12, 162 }, /* Christmas Island */
//     { 310, {'X','X',0}, {'X','X',0}, 27114 }, /* Clipperton Island */
//     { 311, {'C','C',0}, {'C','C','K',0}, 10210825, 166 }, /* Cocos (Keeling) Islands */
//     { 312, {'C','K',0}, {'C','O','K',0}, 26286, 184 }, /* Cook Islands */
//     { 313, {'X','X',0}, {'X','X',0}, 10210825 }, /* Coral Sea Islands */
//     { 314, {'X','X',0}, {'X','X',0}, 114 }, /* Diego Garcia */
//     { 315, {'F','K',0}, {'F','L','K',0}, 31396, 238 }, /* Falkland Islands (Islas Malvinas) */
//     { 317, {'G','F',0}, {'G','U','F',0}, 31396, 254 }, /* French Guiana */
//     { 318, {'P','F',0}, {'P','Y','F',0}, 26286, 258 }, /* French Polynesia */
//     { 319, {'T','F',0}, {'A','T','F',0}, 39070, 260 }, /* French Southern and Antarctic Lands */
//     { 321, {'G','P',0}, {'G','L','P',0}, 10039880, 312 }, /* Guadeloupe */
//     { 322, {'G','U',0}, {'G','U','M',0}, 21206, 316 }, /* Guam */
//     { 323, {'X','X',0}, {'X','X',0}, 39070 }, /* Guantanamo Bay */
//     { 324, {'G','G',0}, {'G','G','Y',0}, 308, 831 }, /* Guernsey */
//     { 325, {'H','M',0}, {'H','M','D',0}, 39070, 334 }, /* Heard Island and McDonald Islands */
//     { 326, {'X','X',0}, {'X','X',0}, 161832256 }, /* Howland Island */
//     { 327, {'X','X',0}, {'X','X',0}, 161832256 }, /* Jarvis Island */
//     { 328, {'J','E',0}, {'J','E','Y',0}, 308, 832 }, /* Jersey */
//     { 329, {'X','X',0}, {'X','X',0}, 161832256 }, /* Kingman Reef */
//     { 330, {'M','Q',0}, {'M','T','Q',0}, 10039880, 474 }, /* Martinique */
//     { 331, {'Y','T',0}, {'M','Y','T',0}, 47603, 175 }, /* Mayotte */
//     { 332, {'M','S',0}, {'M','S','R',0}, 10039880, 500 }, /* Montserrat */
//     { 333, {'A','N',0}, {'A','N','T',0}, 10039880, 530, LOCATION_BOTH }, /* Netherlands Antilles (Former) */
//     { 334, {'N','C',0}, {'N','C','L',0}, 20900, 540 }, /* New Caledonia */
//     { 335, {'N','U',0}, {'N','I','U',0}, 26286, 570 }, /* Niue */
//     { 336, {'N','F',0}, {'N','F','K',0}, 10210825, 574 }, /* Norfolk Island */
//     { 337, {'M','P',0}, {'M','N','P',0}, 21206, 580 }, /* Northern Mariana Islands */
//     { 338, {'X','X',0}, {'X','X',0}, 161832256 }, /* Palmyra Atoll */
//     { 339, {'P','N',0}, {'P','C','N',0}, 26286, 612 }, /* Pitcairn Islands */
//     { 340, {'X','X',0}, {'X','X',0}, 337 }, /* Rota Island */
//     { 341, {'X','X',0}, {'X','X',0}, 337 }, /* Saipan */
//     { 342, {'G','S',0}, {'S','G','S',0}, 39070, 239 }, /* South Georgia and the South Sandwich Islands */
//     { 343, {'S','H',0}, {'S','H','N',0}, 42483, 654 }, /* St. Helena */
//     { 346, {'X','X',0}, {'X','X',0}, 337 }, /* Tinian Island */
//     { 347, {'T','K',0}, {'T','K','L',0}, 26286, 772 }, /* Tokelau */
//     { 348, {'X','X',0}, {'X','X',0}, 39070 }, /* Tristan da Cunha */
//     { 349, {'T','C',0}, {'T','C','A',0}, 10039880, 796 }, /* Turks and Caicos Islands */
//     { 351, {'V','G',0}, {'V','G','B',0}, 10039880,  92 }, /* Virgin Islands, British */
//     { 352, {'W','F',0}, {'W','L','F',0}, 26286, 876 }, /* Wallis and Futuna */
//     { 742, {'X','X',0}, {'X','X',0}, 39070, 0, LOCATION_REGION }, /* Africa */
//     { 2129, {'X','X',0}, {'X','X',0}, 39070, 0, LOCATION_REGION }, /* Asia */
//     { 10541, {'X','X',0}, {'X','X',0}, 39070, 0, LOCATION_REGION }, /* Europe */
//     { 15126, {'I','M',0}, {'I','M','N',0}, 10039882, 833 }, /* Man, Isle of */
//     { 19618, {'M','K',0}, {'M','K','D',0}, 47610, 807 }, /* Macedonia, Former Yugoslav Republic of */
//     { 20900, {'X','X',0}, {'X','X',0}, 27114, 0, LOCATION_REGION }, /* Melanesia */
//     { 21206, {'X','X',0}, {'X','X',0}, 27114, 0, LOCATION_REGION }, /* Micronesia */
//     { 21242, {'X','X',0}, {'X','X',0}, 161832256 }, /* Midway Islands */
//     { 23581, {'X','X',0}, {'X','X',0}, 10026358, 0, LOCATION_REGION }, /* Northern America */
//     { 26286, {'X','X',0}, {'X','X',0}, 27114, 0, LOCATION_REGION }, /* Polynesia */
//     { 27082, {'X','X',0}, {'X','X',0}, 161832257, 0, LOCATION_REGION }, /* Central America */
//     { 27114, {'X','X',0}, {'X','X',0}, 39070, 0, LOCATION_REGION }, /* Oceania */
//     { 30967, {'S','X',0}, {'S','X','M',0}, 10039880, 534 }, /* Sint Maarten (Dutch part) */
//     { 31396, {'X','X',0}, {'X','X',0}, 161832257, 0, LOCATION_REGION }, /* South America */
//     { 31706, {'M','F',0}, {'M','A','F',0}, 10039880, 663 }, /* Saint Martin (French part) */
//     { 39070, {'X','X',0}, {'X','X',0}, 39070, 0, LOCATION_REGION }, /* World */
//     { 42483, {'X','X',0}, {'X','X',0}, 742, 0, LOCATION_REGION }, /* Western Africa */
//     { 42484, {'X','X',0}, {'X','X',0}, 742, 0, LOCATION_REGION }, /* Middle Africa */
//     { 42487, {'X','X',0}, {'X','X',0}, 742, 0, LOCATION_REGION }, /* Northern Africa */
//     { 47590, {'X','X',0}, {'X','X',0}, 2129, 0, LOCATION_REGION }, /* Central Asia */
//     { 47599, {'X','X',0}, {'X','X',0}, 2129, 0, LOCATION_REGION }, /* South-Eastern Asia */
//     { 47600, {'X','X',0}, {'X','X',0}, 2129, 0, LOCATION_REGION }, /* Eastern Asia */
//     { 47603, {'X','X',0}, {'X','X',0}, 742, 0, LOCATION_REGION }, /* Eastern Africa */
//     { 47609, {'X','X',0}, {'X','X',0}, 10541, 0, LOCATION_REGION }, /* Eastern Europe */
//     { 47610, {'X','X',0}, {'X','X',0}, 10541, 0, LOCATION_REGION }, /* Southern Europe */
//     { 47611, {'X','X',0}, {'X','X',0}, 2129, 0, LOCATION_REGION }, /* Middle East */
//     { 47614, {'X','X',0}, {'X','X',0}, 2129, 0, LOCATION_REGION }, /* Southern Asia */
//     { 7299303, {'T','L',0}, {'T','L','S',0}, 47599, 626 }, /* Democratic Republic of Timor-Leste */
//     { 10026358, {'X','X',0}, {'X','X',0}, 39070, 0, LOCATION_REGION }, /* Americas */
//     { 10028789, {'A','X',0}, {'A','L','A',0}, 10039882, 248 }, /* Åland Islands */
//     { 10039880, {'X','X',0}, {'X','X',0}, 161832257, 0, LOCATION_REGION }, /* Caribbean */
//     { 10039882, {'X','X',0}, {'X','X',0}, 10541, 0, LOCATION_REGION }, /* Northern Europe */
//     { 10039883, {'X','X',0}, {'X','X',0}, 742, 0, LOCATION_REGION }, /* Southern Africa */
//     { 10210824, {'X','X',0}, {'X','X',0}, 10541, 0, LOCATION_REGION }, /* Western Europe */
//     { 10210825, {'X','X',0}, {'X','X',0}, 27114, 0, LOCATION_REGION }, /* Australia and New Zealand */
//     { 161832015, {'B','L',0}, {'B','L','M',0}, 10039880, 652 }, /* Saint Barthélemy */
//     { 161832256, {'U','M',0}, {'U','M','I',0}, 27114, 581 }, /* U.S. Minor Outlying Islands */
//     { 161832257, {'X','X',0}, {'X','X',0}, 10026358, 0, LOCATION_REGION }, /* Latin America and the Caribbean */
// };
//
// /* Callback function ptrs for EnumSystemCodePagesA/W */
// typedef struct
// {
//   CODEPAGE_ENUMPROCA procA;
//   CODEPAGE_ENUMPROCW procW;
//   DWORD    dwFlags;
// } ENUMSYSTEMCODEPAGES_CALLBACKS;
//
// /* Internal implementation of EnumSystemCodePagesA/W */
// static BOOL NLS_EnumSystemCodePages(ENUMSYSTEMCODEPAGES_CALLBACKS *lpProcs)
// {
//     WCHAR szNumber[5 + 1], szValue[MAX_PATH];
//     HANDLE hKey;
//     BOOL bContinue = TRUE;
//     ULONG ulIndex = 0;
//
//     if (!lpProcs)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     switch (lpProcs->dwFlags)
//     {
//         case CP_INSTALLED:
//         case CP_SUPPORTED:
//             break;
//         default:
//             SetLastError(ERROR_INVALID_FLAGS);
//             return FALSE;
//     }
//
//     hKey = NLS_RegOpenKey(0, L"\\Registry\\Machine\\SYSTEM\\CurrentControlSet\\Control\\NLS\\CodePage");
//     if (!hKey)
//     {
//         WARN("NLS_RegOpenKey() failed\n");
//         return FALSE;
//     }
//
//     while (bContinue)
//     {
//         if (NLS_RegEnumValue(hKey, ulIndex, szNumber, sizeof(szNumber),
//                              szValue, sizeof(szValue)))
//         {
//             if ((lpProcs->dwFlags == CP_SUPPORTED)||
//                 ((lpProcs->dwFlags == CP_INSTALLED)&&(wcslen(szValue) > 2)))
//             {
//                 if (lpProcs->procW)
//                 {
//                     bContinue = lpProcs->procW(szNumber);
//                 }
//                 else
//                 {
//                     char szNumberA[sizeof(szNumber)/sizeof(WCHAR)];
//
//                     WideCharToMultiByte(CP_ACP, 0, szNumber, -1, szNumberA, sizeof(szNumberA), 0, 0);
//                     bContinue = lpProcs->procA(szNumberA);
//                 }
//             }
//
//             ulIndex++;
//
//         } else bContinue = FALSE;
//
//         if (!bContinue)
//             break;
//     }
//
//     if (hKey)
//         NtClose(hKey);
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// EnumSystemCodePagesW (
//     CODEPAGE_ENUMPROCW  lpCodePageEnumProc,
//     DWORD               dwFlags
//     )
// {
//     ENUMSYSTEMCODEPAGES_CALLBACKS procs;
//
//     TRACE("(%p,0x%08X)\n", lpCodePageEnumProc, dwFlags);
//
//     procs.procA = NULL;
//     procs.procW = lpCodePageEnumProc;
//     procs.dwFlags = dwFlags;
//
//     return NLS_EnumSystemCodePages(lpCodePageEnumProc ? &procs : NULL);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// EnumSystemCodePagesA (
//     CODEPAGE_ENUMPROCA lpCodePageEnumProc,
//     DWORD              dwFlags
//     )
// {
//     ENUMSYSTEMCODEPAGES_CALLBACKS procs;
//
//     TRACE("(%p,0x%08X)\n", lpCodePageEnumProc, dwFlags);
//
//     procs.procA = lpCodePageEnumProc;
//     procs.procW = NULL;
//     procs.dwFlags = dwFlags;
//
//     return NLS_EnumSystemCodePages(lpCodePageEnumProc ? &procs : NULL);
// }
//
//
// static int
// NLS_GetGeoFriendlyName(GEOID Location, LPWSTR szFriendlyName, int cchData, LANGID lang)
// {
//     /* FIXME: move *.nls resources out of kernel32 into locale.nls */
//     Location += NLSRC_OFFSET;
//     Location &= 0xFFFF;
//
//     if (cchData == 0)
//         return GetLocalisedText(Location, NULL, 0, lang);
//
//     if (GetLocalisedText(Location, szFriendlyName, (UINT)cchData, lang))
//         return strlenW(szFriendlyName) + 1;
//
//     return 0;
// }
//
// static const struct geoinfo_t *get_geoinfo_dataptr(GEOID geoid)
// {
//     int min, max;
//
//     min = 0;
//     max = sizeof(geoinfodata)/sizeof(struct geoinfo_t)-1;
//
//     while (min <= max) {
//         const struct geoinfo_t *ptr;
//         int n = (min+max)/2;
//
//         ptr = &geoinfodata[n];
//         if (geoid == ptr->id)
//             /* we don't need empty entries */
//             return *ptr->iso2W ? ptr : NULL;
//
//         if (ptr->id > geoid)
//             max = n-1;
//         else
//             min = n+1;
//     }
//
//     return NULL;
// }
//
// /******************************************************************************
//  *           GetGeoInfoW (KERNEL32.@)
//  */
// INT WINAPI GetGeoInfoW(GEOID geoid, GEOTYPE geotype, LPWSTR data, int data_len, LANGID lang)
// {
//     const struct geoinfo_t *ptr;
//     const WCHAR *str = NULL;
//     WCHAR buffW[12];
//     LONG val = 0;
//     INT len;
//
//     TRACE("%d %d %p %d %d\n", geoid, geotype, data, data_len, lang);
//
//     if (!(ptr = get_geoinfo_dataptr(geoid))) {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     switch (geotype) {
//     case GEO_FRIENDLYNAME:
//     {
//         return NLS_GetGeoFriendlyName(geoid, data, data_len, lang);
//     }
//     case GEO_NATION:
//         val = geoid;
//         break;
//     case GEO_ISO_UN_NUMBER:
//         val = ptr->uncode;
//         break;
//     case GEO_PARENT:
//         val = ptr->parent;
//         break;
//     case GEO_ISO2:
//     case GEO_ISO3:
//     {
//         str = geotype == GEO_ISO2 ? ptr->iso2W : ptr->iso3W;
//         break;
//     }
//     case GEO_RFC1766:
//     case GEO_LCID:
//     case GEO_OFFICIALNAME:
//     case GEO_TIMEZONES:
//     case GEO_OFFICIALLANGUAGES:
//     case GEO_LATITUDE:
//     case GEO_LONGITUDE:
//         FIXME("type %d is not supported\n", geotype);
//         SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//         return 0;
//     default:
//         WARN("unrecognized type %d\n", geotype);
//         SetLastError(ERROR_INVALID_FLAGS);
//         return 0;
//     }
//
//     if (val) {
//         static const WCHAR fmtW[] = {'%','d',0};
//         sprintfW(buffW, fmtW, val);
//         str = buffW;
//     }
//
//     len = strlenW(str) + 1;
//     if (!data || !data_len)
//         return len;
//
//     memcpy(data, str, min(len, data_len)*sizeof(WCHAR));
//     if (data_len < len)
//         SetLastError(ERROR_INSUFFICIENT_BUFFER);
//     return data_len < len ? 0 : len;
// }
//
// /******************************************************************************
//  *           GetGeoInfoA (KERNEL32.@)
//  */
// INT WINAPI GetGeoInfoA(GEOID geoid, GEOTYPE geotype, LPSTR data, int data_len, LANGID lang)
// {
//     WCHAR *buffW;
//     INT len;
//
//     TRACE("%d %d %p %d %d\n", geoid, geotype, data, data_len, lang);
//
//     len = GetGeoInfoW(geoid, geotype, NULL, 0, lang);
//     if (!len)
//         return 0;
//
//     buffW = HeapAlloc(GetProcessHeap(), 0, len*sizeof(WCHAR));
//     if (!buffW)
//         return 0;
//
//     GetGeoInfoW(geoid, geotype, buffW, len, lang);
//     len = WideCharToMultiByte(CP_ACP, 0, buffW, -1, NULL, 0, NULL, NULL);
//     if (!data || !data_len) {
//         HeapFree(GetProcessHeap(), 0, buffW);
//         return len;
//     }
//
//     len = WideCharToMultiByte(CP_ACP, 0, buffW, -1, data, data_len, NULL, NULL);
//     HeapFree(GetProcessHeap(), 0, buffW);
//
//     if (data_len < len)
//         SetLastError(ERROR_INSUFFICIENT_BUFFER);
//     return data_len < len ? 0 : len;
// }
//
// /******************************************************************************
//  *           EnumSystemGeoID    (KERNEL32.@)
//  *
//  * Call a users function for every location available on the system.
//  *
//  * PARAMS
//  *  geoclass   [I] Type of information desired (SYSGEOTYPE enum from "winnls.h")
//  *  parent     [I] GEOID for the parent
//  *  enumproc   [I] Callback function to call for each location
//  *
//  * RETURNS
//  *  Success: TRUE.
//  *  Failure: FALSE. Use GetLastError() to determine the cause.
//  */
// BOOL WINAPI EnumSystemGeoID(GEOCLASS geoclass, GEOID parent, GEO_ENUMPROC enumproc)
// {
//     INT i;
//
//     TRACE("(%d, %d, %p)\n", geoclass, parent, enumproc);
//
//     if (!enumproc) {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     if (geoclass != GEOCLASS_NATION && geoclass != GEOCLASS_REGION) {
//         SetLastError(ERROR_INVALID_FLAGS);
//         return FALSE;
//     }
//
//     for (i = 0; i < sizeof(geoinfodata)/sizeof(struct geoinfo_t); i++) {
//         const struct geoinfo_t *ptr = &geoinfodata[i];
//
//         if (geoclass == GEOCLASS_NATION && (ptr->kind == LOCATION_REGION))
//             continue;
//
//         if (geoclass == GEOCLASS_REGION && (ptr->kind == LOCATION_NATION))
//             continue;
//
//         if (parent && ptr->parent != parent)
//             continue;
//
//         if (!enumproc(ptr->id))
//             return TRUE;
//     }
//
//     return TRUE;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
