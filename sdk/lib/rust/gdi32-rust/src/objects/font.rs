//! objects/font.c → Rust. Font selection/enumeration/metrics client API.
//
// Wave-2b: the largest single gdi32 client file (~50 fns). Same crate/substrate as
// Waves 1a/1b/2 (all landed). Imports: `win32k_sys::win32k::*` + `crate::gdihelp::*`.
//
// NEEDS-TYPE: `CPINFO` is NOT in win32k-sys bindgen — reproduced locally as a `#[repr(C)]`
//   struct matching winnls.h (MaxCharSize:UINT, DefaultChar:[BYTE;2], LeadByte:[BYTE;12]).
// NEEDS-IMPORT: a handful of constants are absent from win32k-sys bindgen and reproduced as
//   local consts with psdk values: CP_ACP(0), CP_THREAD_ACP(3), CP_SYMBOL(42),
//   LOCALE_FONTSIGNATURE(0x58), LPK_GCP(3), UNICODE_NULL(0), OPEN_EXISTING(3),
//   S_OK(0), STRSAFE_E_INSUFFICIENT_BUFFER(0x8007007A), INVALID_HANDLE_VALUE(-1).
//   `GdiGetFontLanguageInfo` (GETDCDWORD enum, value 6) → `_GETDCDWORD_GdiGetFontLanguageInfo`
//   from the win32k_sys glob.
// NEEDS-CHECK: strsafe inline — StringCbCopyW / StringCchCatW / StringCchCopyNW are header
//   inlines (strsafe.h); reproduced locally as faithful bounded wide-string helpers returning
//   HRESULT (S_OK / STRSAFE_E_INSUFFICIENT_BUFFER). The return value is only consulted in
//   IntConvertFontPaths (StringCchCatW != S_OK), matched exactly.
// NEEDS-CHECK: cross-file ABI — FONT_TextMetricWToA / NewTextMetricW2A / NewTextMetricExW2A /
//   IntFontFamilyListUnique / IntConvertFontPaths are non-static FASTCALL → defined here as
//   `#[no_mangle] extern "fastcall"` (text.rs already externs FONT_TextMetricWToA fastcall).
//   GetDCDWord/EnumLogFontExW2A WINAPI (gdiobj.rs precedent); LogFontA2W/LogFontW2A NTAPI
//   (→ extern "system"); HEAP_strdupA2W_buf / HEAP_alloc / HEAP_strdupA2W FASTCALL.
// NEEDS-CHECK: the #if DBG helpers DumpFamilyInfo / DoFontSystemUnittest are compiled out in
//   the release (NDEBUG) build and are omitted here (they reference DPRINT + unittest infra).

use win32k_sys::win32k::*;
use crate::gdihelp::*;

use core::ffi::{c_int, c_void};
use crate::dcattr_f;

// win32k_sys's FALSE/TRUE are bindgen u32 (#define), but BOOL is i32 — shadow with
// BOOL-typed locals so `== FALSE`/`return TRUE` type-check against the i32 BOOL returns.
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// psdk constants not present in win32k_sys bindgen.
const CP_ACP: UINT = 0;
const CP_THREAD_ACP: UINT = 3;
const CP_SYMBOL: UINT = 42;
const LOCALE_FONTSIGNATURE: DWORD = 0x0000_0058;
const LPK_GCP: INT = 3;
const UNICODE_NULL: WCHAR = 0;
const OPEN_EXISTING: DWORD = 3;
const S_OK: HRESULT = 0;
const STRSAFE_E_INSUFFICIENT_BUFFER: HRESULT = 0x8007_007Au32 as HRESULT;

#[inline]
fn invalid_handle_value() -> HANDLE {
    -1isize as HANDLE
}

// ── Local types absent from bindgen (winnls.h) ─────────────────────────────────
const MAX_DEFAULTCHAR: usize = 2;
const MAX_LEADBYTES: usize = 12;

#[repr(C)]
struct CPINFO {
    MaxCharSize: UINT,
    DefaultChar: [BYTE; MAX_DEFAULTCHAR],
    LeadByte: [BYTE; MAX_LEADBYTES],
}

// ── Cross-file gdi32 / kernel32 fns (WINAPI / NTAPI → stdcall on i386) ──────────
extern "system" {
    fn SetLastError(dwErrCode: DWORD);
    fn GdiSetLastError(dwErrCode: DWORD); // misc.rs (WINAPI)
    fn GdiValidateHandle(hobj: HGDIOBJ) -> BOOL; // objects/gdiobj.c
    // kernel32
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn GetProcessHeap() -> HANDLE;
    fn GetCPInfo(CodePage: UINT, lpCPInfo: *mut CPINFO) -> BOOL;
    fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: LPCSTR,
        cbMultiByte: INT,
        lpWideCharStr: LPWSTR,
        cchWideChar: INT,
    ) -> INT;
    fn WideCharToMultiByte(
        CodePage: UINT,
        dwFlags: DWORD,
        lpWideCharStr: LPCWSTR,
        cchWideChar: INT,
        lpMultiByteStr: LPSTR,
        cbMultiByte: INT,
        lpDefaultChar: LPCSTR,
        lpUsedDefaultChar: *mut BOOL,
    ) -> INT;
    fn lstrcpynW(lpString1: LPWSTR, lpString2: LPCWSTR, iMaxLength: INT) -> LPWSTR;
    fn GetWindowsDirectoryW(lpBuffer: LPWSTR, uSize: UINT) -> UINT;
    fn SearchPathW(
        lpPath: LPCWSTR,
        lpFileName: LPCWSTR,
        lpExtension: LPCWSTR,
        nBufferLength: DWORD,
        lpBuffer: LPWSTR,
        lpFilePart: *mut LPWSTR,
    ) -> DWORD;
    fn CreateFileW(
        lpFileName: LPCWSTR,
        dwDesiredAccess: DWORD,
        dwShareMode: DWORD,
        lpSecurityAttributes: PVOID,
        dwCreationDisposition: DWORD,
        dwFlagsAndAttributes: DWORD,
        hTemplateFile: HANDLE,
    ) -> HANDLE;
    fn CloseHandle(hObject: HANDLE) -> BOOL;
    fn GetLocaleInfoW(Locale: LCID, LCType: DWORD, lpLCData: LPWSTR, cchData: INT) -> INT;
    // gdi32 (other objects)
    fn GetDeviceCaps(hdc: HDC, nIndex: INT) -> INT;
    fn GetDCDWord(hdc: HDC, u: UINT, dwError: DWORD) -> DWORD;
    fn EnumLogFontExW2A(fontA: LPENUMLOGFONTEXA, fontW: *const ENUMLOGFONTEXW);
    fn LoadLPK(LpkFunctionID: INT) -> BOOL;
    fn LogFontA2W(pW: LPLOGFONTW, pA: *const LOGFONTA);
    // gdi32 text.c (Wave-2, WINAPI)
    fn GdiGetCodePage(hdc: HDC) -> DWORD;
    fn GetTextCharset(hdc: HDC) -> INT;
    fn GetTextMetricsW(hdc: HDC, lptm: LPTEXTMETRICW) -> BOOL;
    fn GetTextExtentPointW(hdc: HDC, lpString: LPCWSTR, cchString: INT, lpsz: LPSIZE) -> BOOL;
    fn GetTextExtentPoint32W(hdc: HDC, lpString: LPCWSTR, cchString: INT, lpSize: LPSIZE) -> BOOL;
    // ntdll NTAPI
    fn RtlAllocateHeap(HeapHandle: PVOID, Flags: ULONG, Size: SIZE_T) -> PVOID;
    fn RtlFreeHeap(HeapHandle: PVOID, Flags: ULONG, BaseAddress: PVOID) -> BOOLEAN;
    fn RtlInitAnsiString(DestinationString: *mut STRING, SourceString: PCSTR);
    fn RtlAnsiStringToUnicodeString(
        DestinationString: PUNICODE_STRING,
        SourceString: *const STRING,
        AllocateDestinationString: BOOLEAN,
    ) -> NTSTATUS;
    fn RtlFreeUnicodeString(UnicodeString: PUNICODE_STRING);
    fn RtlDosPathNameToNtPathName_U(
        DosFileName: LPCWSTR,
        NtFileName: PUNICODE_STRING,
        FilePart: *mut PWSTR,
        Reserved: PVOID,
    ) -> BOOLEAN;
}

// `(GDI_HANDLE_GET_TYPE(hdc) != GDI_OBJECT_TYPE_DC) || !GdiValidateHandle(hdc)`, negated: the
// DC check upstream's GetCharWidth / GetGlyphIndices rewrites (#8690, #8747) put in front of
// every width and glyph-index query, so a stale or non-DC handle fails with
// ERROR_INVALID_PARAMETER instead of reaching the syscall.
#[inline]
unsafe fn is_live_dc(hdc: HDC) -> bool {
    GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == GDI_OBJECT_TYPE_DC as usize
        && GdiValidateHandle(hdc as HGDIOBJ) != 0
}

// RtlULongMult — __forceinline from ntintsafe.h (NOT an ntdll export), so reimplement
// locally (matches win32k-rust/src/surface.rs). Checked u32 multiply: STATUS_SUCCESS (0)
// or STATUS_INTEGER_OVERFLOW (0xC0000095) with *out = ULONG_MAX on overflow.
#[inline(always)]
unsafe fn RtlULongMult(uMultiplicand: ULONG, uMultiplier: ULONG, puResult: *mut ULONG) -> NTSTATUS {
    let result = (uMultiplicand as u64).wrapping_mul(uMultiplier as u64);
    if result > (ULONG::MAX as u64) {
        *puResult = ULONG::MAX;
        0xC0000095u32 as NTSTATUS
    } else {
        *puResult = result as ULONG;
        0
    }
}

// ── FASTCALL gdi32-private helpers (gdi32p.h) ──────────────────────────────────
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn HEAP_strdupA2W_buf(lpszA: PCSTR, pszStaticBuff: PWSTR, cchStaticBuff: SIZE_T) -> PWSTR;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn HEAP_strdupA2W_buf(lpszA: PCSTR, pszStaticBuff: PWSTR, cchStaticBuff: SIZE_T) -> PWSTR;
}

// ── gdi32-private global (dllmain.rs) ──────────────────────────────────────────
extern "C" {
    static mut gbLpk: BOOL;
}

// ── CRT (cdecl → extern "C") ───────────────────────────────────────────────────
type QsortCmp = unsafe extern "C" fn(*const c_void, *const c_void) -> c_int;
extern "C" {
    fn qsort(base: *mut c_void, nmemb: usize, size: usize, compar: QsortCmp);
    fn _wcsicmp(s1: *const WCHAR, s2: *const WCHAR) -> c_int;
    fn wcslen(s: *const WCHAR) -> usize;
    fn strlen(s: *const core::ffi::c_char) -> usize;
}

// ── FONTENUMPROC callback fn-pointer types (wingdi.h, WINAPI) ───────────────────
type FONTENUMPROCW_T = unsafe extern "system" fn(
    *const ENUMLOGFONTEXW,
    *const NEWTEXTMETRICEXW,
    DWORD,
    LPARAM,
) -> INT;
type FONTENUMPROCA_T = unsafe extern "system" fn(
    *const ENUMLOGFONTEXA,
    *const NEWTEXTMETRICEXA,
    DWORD,
    LPARAM,
) -> INT;
// LPK GCP hook (gdi32p.h `extern LPKGCP LpkGetCharacterPlacement`)
type LPKGCP = unsafe extern "system" fn(
    HDC,
    LPCWSTR,
    INT,
    INT,
    LPGCP_RESULTSW,
    DWORD,
    DWORD,
) -> DWORD;
extern "C" {
    static mut LpkGetCharacterPlacement: LPKGCP;
}

// ── Small inline substrate (mirrors text.rs) ───────────────────────────────────
/// `RtlGetProcessHeap()` — `NtCurrentPeb()->ProcessHeap`.
#[inline]
unsafe fn RtlGetProcessHeap() -> PVOID {
    (*(*NtCurrentTeb()).ProcessEnvironmentBlock).ProcessHeap
}

/// `HEAP_alloc` (gdi32p.h static-inline FASTCALL) — `RtlAllocateHeap(hProcessHeap, 0, len)`.
#[inline]
unsafe fn HEAP_alloc(len: SIZE_T) -> PVOID {
    RtlAllocateHeap(RtlGetProcessHeap(), 0, len)
}

/// `HEAP_free` (gdi32p.h static-inline FASTCALL) — `RtlFreeHeap(hProcessHeap, 0, memory)`.
#[inline]
unsafe fn HEAP_free(memory: PVOID) {
    RtlFreeHeap(RtlGetProcessHeap(), 0, memory);
}

/// `HEAP_strdupA2W_buf_free` (gdi32p.h static-inline FASTCALL).
#[inline]
unsafe fn HEAP_strdupA2W_buf_free(pszDynamicBuff: PWSTR, pszStaticBuff: PWSTR) {
    if !pszDynamicBuff.is_null() && pszDynamicBuff != pszStaticBuff {
        HEAP_free(pszDynamicBuff as PVOID);
    }
}

/// `NT_SUCCESS(Status)` — `Status >= 0`.
#[inline]
fn NT_SUCCESS(Status: NTSTATUS) -> bool {
    Status >= 0
}

/// `MAKELONG(lo, hi)`.
#[inline]
fn MAKELONG(lo: USHORT, hi: USHORT) -> DWORD {
    (lo as DWORD) | ((hi as DWORD) << 16)
}

/// `fabs` over f64 as a macro — i386 `-sse,-sse2` forbids an f64 *value* crossing a fn
/// boundary, so this must inline at the call site (math stays in-body, x87). Mirrors the
/// GDI_ROUND!/coord.rs _fmul! pattern.
macro_rules! dabs {
    ($x:expr) => {{
        let x: f64 = $x;
        if x < 0.0 {
            -x
        } else {
            x
        }
    }};
}

/// `GDI_ROUND(FLOAT val)` (font.c:24) — `(int)floor(val + 0.5)`.
///
/// MUST be a macro, not a fn: i386 builds this crate with `-sse,-sse2`, so an `f32`
/// cannot cross a Rust function boundary. The float math stays inside the macro body
/// (x87). `floor` is implemented inline (no libm): truncate-toward-zero with a -1 fix
/// for the negative case.
macro_rules! GDI_ROUND {
    ($v:expr) => {{
        // C does the whole world-to-viewport transform in `double` then (int)floor(val+0.5);
        // keep f64 here (no narrowing to f32) for faithfulness. i386 -sse,-sse2 forbids an
        // f64 value crossing a fn boundary, so this stays a macro (math in-body, x87).
        let val: f64 = ($v) as f64;
        let t: f64 = val + 0.5;
        let i: f64 = t as i64 as f64; // truncate toward zero
        (if i > t { i - 1.0 } else { i }) as INT
    }};
}

//
// For TranslateCharsetInfo
//
const MAXTCIINDEX: usize = 32;

const fn tci(cs: UINT, acp: UINT, csb0: DWORD) -> CHARSETINFO {
    tagCHARSETINFO {
        ciCharset: cs,
        ciACP: acp,
        fs: tagFONTSIGNATURE {
            fsUsb: [0, 0, 0, 0],
            fsCsb: [csb0, 0],
        },
    }
}

static FONT_tci: [CHARSETINFO; MAXTCIINDEX] = [
    /* ANSI */
    tci(ANSI_CHARSET, 1252, FS_LATIN1),
    tci(EASTEUROPE_CHARSET, 1250, FS_LATIN2),
    tci(RUSSIAN_CHARSET, 1251, FS_CYRILLIC),
    tci(GREEK_CHARSET, 1253, FS_GREEK),
    tci(TURKISH_CHARSET, 1254, FS_TURKISH),
    tci(HEBREW_CHARSET, 1255, FS_HEBREW),
    tci(ARABIC_CHARSET, 1256, FS_ARABIC),
    tci(BALTIC_CHARSET, 1257, FS_BALTIC),
    tci(VIETNAMESE_CHARSET, 1258, FS_VIETNAMESE),
    /* reserved by ANSI */
    tci(DEFAULT_CHARSET, 0, FS_LATIN1),
    tci(DEFAULT_CHARSET, 0, FS_LATIN1),
    tci(DEFAULT_CHARSET, 0, FS_LATIN1),
    tci(DEFAULT_CHARSET, 0, FS_LATIN1),
    tci(DEFAULT_CHARSET, 0, FS_LATIN1),
    tci(DEFAULT_CHARSET, 0, FS_LATIN1),
    tci(DEFAULT_CHARSET, 0, FS_LATIN1),
    /* ANSI and OEM */
    tci(THAI_CHARSET, 874, FS_THAI),
    tci(SHIFTJIS_CHARSET, 932, FS_JISJAPAN),
    tci(GB2312_CHARSET, 936, FS_CHINESESIMP),
    tci(HANGEUL_CHARSET, 949, FS_WANSUNG),
    tci(CHINESEBIG5_CHARSET, 950, FS_CHINESETRAD),
    tci(JOHAB_CHARSET, 1361, FS_JOHAB),
    /* reserved for alternate ANSI and OEM */
    tci(DEFAULT_CHARSET, 0, FS_LATIN1),
    tci(DEFAULT_CHARSET, 0, FS_LATIN1),
    tci(DEFAULT_CHARSET, 0, FS_LATIN1),
    tci(DEFAULT_CHARSET, 0, FS_LATIN1),
    tci(DEFAULT_CHARSET, 0, FS_LATIN1),
    tci(DEFAULT_CHARSET, 0, FS_LATIN1),
    tci(DEFAULT_CHARSET, 0, FS_LATIN1),
    tci(DEFAULT_CHARSET, 0, FS_LATIN1),
    /* reserved for system */
    tci(DEFAULT_CHARSET, 0, FS_LATIN1),
    tci(SYMBOL_CHARSET, CP_SYMBOL, FS_SYMBOL),
];

const INITIAL_FAMILY_COUNT: INT = 64;

/***********************************************************************
 *              TEXTMETRIC conversion functions.
 */
fastcall_fn! {
#[no_mangle]
pub unsafe fn FONT_TextMetricWToA(ptmW: *const TEXTMETRICW, ptmA: LPTEXTMETRICA) {
    (*ptmA).tmHeight = (*ptmW).tmHeight;
    (*ptmA).tmAscent = (*ptmW).tmAscent;
    (*ptmA).tmDescent = (*ptmW).tmDescent;
    (*ptmA).tmInternalLeading = (*ptmW).tmInternalLeading;
    (*ptmA).tmExternalLeading = (*ptmW).tmExternalLeading;
    (*ptmA).tmAveCharWidth = (*ptmW).tmAveCharWidth;
    (*ptmA).tmMaxCharWidth = (*ptmW).tmMaxCharWidth;
    (*ptmA).tmWeight = (*ptmW).tmWeight;
    (*ptmA).tmOverhang = (*ptmW).tmOverhang;
    (*ptmA).tmDigitizedAspectX = (*ptmW).tmDigitizedAspectX;
    (*ptmA).tmDigitizedAspectY = (*ptmW).tmDigitizedAspectY;
    (*ptmA).tmFirstChar = (if ((*ptmW).tmFirstChar as INT) < 255 {
        (*ptmW).tmFirstChar as INT
    } else {
        255
    }) as BYTE;
    if (*ptmW).tmCharSet == SYMBOL_CHARSET as BYTE {
        (*ptmA).tmFirstChar = 0x1e;
        (*ptmA).tmLastChar = 0xff; /* win9x behaviour - we need the OS2 table data to calculate correctly */
    } else {
        (*ptmA).tmFirstChar = ((*ptmW).tmDefaultChar as INT - 1) as BYTE;
        (*ptmA).tmLastChar = (if ((*ptmW).tmLastChar as INT) < 0xff {
            (*ptmW).tmLastChar as INT
        } else {
            0xff
        }) as BYTE;
    }
    (*ptmA).tmDefaultChar = (*ptmW).tmDefaultChar as CHAR as BYTE;
    (*ptmA).tmBreakChar = (*ptmW).tmBreakChar as CHAR as BYTE;
    (*ptmA).tmItalic = (*ptmW).tmItalic;
    (*ptmA).tmUnderlined = (*ptmW).tmUnderlined;
    (*ptmA).tmStruckOut = (*ptmW).tmStruckOut;
    (*ptmA).tmPitchAndFamily = (*ptmW).tmPitchAndFamily;
    (*ptmA).tmCharSet = (*ptmW).tmCharSet;
}
}

/***********************************************************************
 *           FONT_mbtowc
 *
 * Returns a Unicode translation of str using the charset of the
 * currently selected font in hdc.  See font.c for the full contract.
 */
unsafe fn FONT_mbtowc(
    hdc: HDC,
    str_: LPCSTR,
    mut count: INT,
    plenW: *mut INT,
    pCP: *mut UINT,
) -> LPWSTR {
    let cp: UINT = GdiGetCodePage(hdc);
    let lenW: INT;
    let strW: LPWSTR;

    if count == -1 {
        count = strlen(str_) as INT;
    }
    lenW = MultiByteToWideChar(cp, 0, str_, count, core::ptr::null_mut(), 0);
    strW = HeapAlloc(
        GetProcessHeap(),
        0,
        (lenW as usize * core::mem::size_of::<WCHAR>()) as SIZE_T,
    ) as LPWSTR;
    if strW.is_null() {
        return core::ptr::null_mut();
    }
    if MultiByteToWideChar(cp, 0, str_, count, strW, lenW) == 0 {
        HeapFree(GetProcessHeap(), 0, strW as LPVOID);
        return core::ptr::null_mut();
    }
    // DPRINT("mapped %s -> %S\n", str, strW);
    if !plenW.is_null() {
        *plenW = lenW;
    }
    if !pCP.is_null() {
        *pCP = cp;
    }
    strW
}

unsafe fn FONT_GetCharsByRangeA(
    hdc: HDC,
    firstChar: UINT,
    lastChar: UINT,
    pByteLen: PINT,
) -> LPSTR {
    let count: INT = (lastChar as INT) - (firstChar as INT) + 1;
    let mut i: INT;
    let mut c: UINT;
    let str_: LPSTR;

    if count <= 0 {
        return core::ptr::null_mut();
    }

    match GdiGetCodePage(hdc) {
        932 | 936 | 949 | 950 | 1361 => {
            if lastChar > 0xffff {
                return core::ptr::null_mut();
            }
            if (firstChar ^ lastChar) > 0xff {
                return core::ptr::null_mut();
            }
        }
        _ => {
            if lastChar > 0xff {
                return core::ptr::null_mut();
            }
        }
    }

    str_ = HeapAlloc(GetProcessHeap(), 0, (count * 2 + 1) as SIZE_T) as LPSTR;
    if str_.is_null() {
        return core::ptr::null_mut();
    }

    i = 0;
    c = firstChar;
    while c <= lastChar {
        if c > 0xff {
            *str_.offset(i as isize) = (c >> 8) as BYTE as CHAR;
            i += 1;
        }
        *str_.offset(i as isize) = c as BYTE as CHAR;
        i += 1;
        c += 1;
    }
    *str_.offset(i as isize) = b'\0' as CHAR;

    *pByteLen = i;

    str_
}

fastcall_fn! {
#[no_mangle]
pub unsafe fn NewTextMetricW2A(tma: *mut NEWTEXTMETRICA, tmw: *mut NEWTEXTMETRICW) {
    FONT_TextMetricWToA(tmw as *const TEXTMETRICW, tma as LPTEXTMETRICA);
    (*tma).ntmFlags = (*tmw).ntmFlags;
    (*tma).ntmSizeEM = (*tmw).ntmSizeEM;
    (*tma).ntmCellHeight = (*tmw).ntmCellHeight;
    (*tma).ntmAvgWidth = (*tmw).ntmAvgWidth;
}
}

fastcall_fn! {
#[no_mangle]
pub unsafe fn NewTextMetricExW2A(
    tma: *mut NEWTEXTMETRICEXA,
    tmw: *mut NEWTEXTMETRICEXW,
) {
    NewTextMetricW2A(&mut (*tma).ntmTm, &mut (*tmw).ntmTm);
    (*tma).ntmFontSig = (*tmw).ntmFontSig;
}
}

// IntFontFamilyCompareEx's flags
const IFFCX_CHARSET: DWORD = 1;
const IFFCX_STYLE: DWORD = 2;

unsafe fn IntFontFamilyCompareEx(
    ffi1: *const FONTFAMILYINFO,
    ffi2: *const FONTFAMILYINFO,
    dwCompareFlags: DWORD,
) -> c_int {
    let plf1: *const LOGFONTW = &(*ffi1).EnumLogFontEx.elfLogFont;
    let plf2: *const LOGFONTW = &(*ffi2).EnumLogFontEx.elfLogFont;
    let WeightDiff1: ULONG;
    let WeightDiff2: ULONG;
    let cmp: c_int = _wcsicmp((*plf1).lfFaceName.as_ptr(), (*plf2).lfFaceName.as_ptr());
    if cmp != 0 {
        return cmp;
    }
    if dwCompareFlags & IFFCX_CHARSET != 0 {
        if (*plf1).lfCharSet < (*plf2).lfCharSet {
            return -1;
        }
        if (*plf1).lfCharSet > (*plf2).lfCharSet {
            return 1;
        }
    }
    if dwCompareFlags & IFFCX_STYLE != 0 {
        WeightDiff1 = ((*plf1).lfWeight - FW_NORMAL as LONG).wrapping_abs() as ULONG;
        WeightDiff2 = ((*plf2).lfWeight - FW_NORMAL as LONG).wrapping_abs() as ULONG;
        if WeightDiff1 < WeightDiff2 {
            return -1;
        }
        if WeightDiff1 > WeightDiff2 {
            return 1;
        }
        if (*plf1).lfItalic < (*plf2).lfItalic {
            return -1;
        }
        if (*plf1).lfItalic > (*plf2).lfItalic {
            return 1;
        }
    }
    0
}

unsafe extern "C" fn IntFontFamilyCompare(ffi1: *const c_void, ffi2: *const c_void) -> c_int {
    IntFontFamilyCompareEx(
        ffi1 as *const FONTFAMILYINFO,
        ffi2 as *const FONTFAMILYINFO,
        IFFCX_STYLE | IFFCX_CHARSET,
    )
}

// IntEnumFontFamilies' flags:
const IEFF_UNICODE: DWORD = 1;
const IEFF_EXTENDED: DWORD = 2;

fastcall_fn! {
#[no_mangle]
pub unsafe fn IntFontFamilyListUnique(
    InfoList: *mut FONTFAMILYINFO,
    mut nCount: INT,
    plf: *const LOGFONTW,
    dwFlags: DWORD,
) -> c_int {
    let mut first: *mut FONTFAMILYINFO;
    let last: *mut FONTFAMILYINFO;
    let mut result: *mut FONTFAMILYINFO;
    let mut dwCompareFlags: DWORD = 0;

    if (*plf).lfFaceName[0] != 0 {
        dwCompareFlags |= IFFCX_STYLE;
    }

    if (dwFlags & IEFF_EXTENDED) != 0 && (*plf).lfCharSet == DEFAULT_CHARSET as BYTE {
        dwCompareFlags |= IFFCX_CHARSET;
    }

    first = InfoList;
    last = InfoList.offset(nCount as isize);

    /* std::unique(first, last, IntFontFamilyCompareEx); */
    if first == last {
        return 0;
    }

    result = first;
    loop {
        first = first.offset(1);
        if first == last {
            break;
        }
        if IntFontFamilyCompareEx(result, first, dwCompareFlags) != 0 {
            result = result.offset(1);
            *result = *first;
        }
    }
    result = result.offset(1);
    nCount = result.offset_from(InfoList) as INT;

    nCount
}
}

unsafe fn IntEnumFontFamilies(
    Dc: HDC,
    LogFont: *const LOGFONTW,
    EnumProc: PVOID,
    lParam: LPARAM,
    dwFlags: DWORD,
) -> c_int {
    let mut FontFamilyCount: INT;
    let mut Info: PFONTFAMILYINFO;
    let mut Ret: INT = 1;
    let mut i: INT;
    let mut EnumLogFontExA: ENUMLOGFONTEXA = core::mem::zeroed();
    let mut NewTextMetricExA: NEWTEXTMETRICEXA = core::mem::zeroed();
    let mut lfW: LOGFONTW = core::mem::zeroed();
    let mut InfoCount: LONG;
    let mut DataSize: ULONG;
    let Status: NTSTATUS;

    DataSize = INITIAL_FAMILY_COUNT as ULONG * core::mem::size_of::<FONTFAMILYINFO>() as ULONG;
    Info = RtlAllocateHeap(GetProcessHeap() as PVOID, 0, DataSize as SIZE_T) as PFONTFAMILYINFO;
    if Info.is_null() {
        return 1;
    }

    /* Initialize the LOGFONT structure */
    // ZeroMemory(&lfW, sizeof(lfW)) — done by zeroed() above.
    if LogFont.is_null() {
        lfW.lfCharSet = DEFAULT_CHARSET as BYTE;
    } else {
        lfW.lfCharSet = (*LogFont).lfCharSet;
        lfW.lfPitchAndFamily = (*LogFont).lfPitchAndFamily;
        StringCbCopyW(
            lfW.lfFaceName.as_mut_ptr(),
            core::mem::size_of_val(&lfW.lfFaceName) as SIZE_T,
            (*LogFont).lfFaceName.as_ptr(),
        );
    }

    /* Retrieve the font information */
    InfoCount = INITIAL_FAMILY_COUNT;
    FontFamilyCount = NtGdiGetFontFamilyInfo(Dc, &lfW, Info, &mut InfoCount);
    if FontFamilyCount < 0 {
        RtlFreeHeap(GetProcessHeap() as PVOID, 0, Info as PVOID);
        return 1;
    }

    /* Resize the buffer if the buffer is too small */
    if INITIAL_FAMILY_COUNT < InfoCount {
        RtlFreeHeap(GetProcessHeap() as PVOID, 0, Info as PVOID);

        Status = RtlULongMult(
            InfoCount as ULONG,
            core::mem::size_of::<FONTFAMILYINFO>() as ULONG,
            &mut DataSize,
        );
        if !NT_SUCCESS(Status) || DataSize > LONG_MAX as ULONG {
            // DPRINT1("Overflowed.\n");
            return 1;
        }
        Info = RtlAllocateHeap(GetProcessHeap() as PVOID, 0, DataSize as SIZE_T) as PFONTFAMILYINFO;
        if Info.is_null() {
            return 1;
        }
        FontFamilyCount = NtGdiGetFontFamilyInfo(Dc, &lfW, Info, &mut InfoCount);
        if FontFamilyCount < 0 || FontFamilyCount < InfoCount {
            RtlFreeHeap(GetProcessHeap() as PVOID, 0, Info as PVOID);
            return 1;
        }
    }

    /* Sort and remove redundant information */
    qsort(
        Info as *mut c_void,
        FontFamilyCount as usize,
        core::mem::size_of::<FONTFAMILYINFO>(),
        IntFontFamilyCompare,
    );
    FontFamilyCount = IntFontFamilyListUnique(Info, FontFamilyCount, &lfW, dwFlags);

    /* call the callback */
    i = 0;
    while i < FontFamilyCount {
        let info_i: *mut FONTFAMILYINFO = Info.offset(i as isize);
        if dwFlags & IEFF_UNICODE != 0 {
            let proc_: FONTENUMPROCW_T = core::mem::transmute(EnumProc);
            Ret = proc_(
                &(*info_i).EnumLogFontEx,
                &(*info_i).NewTextMetricEx,
                (*info_i).FontType,
                lParam,
            );
        } else {
            // Could use EnumLogFontExW2A here?
            LogFontW2A(
                &mut EnumLogFontExA.elfLogFont,
                &(*info_i).EnumLogFontEx.elfLogFont,
            );
            WideCharToMultiByte(
                CP_THREAD_ACP,
                0,
                (*info_i).EnumLogFontEx.elfFullName.as_ptr(),
                -1,
                EnumLogFontExA.elfFullName.as_mut_ptr() as LPSTR,
                LF_FULLFACESIZE as INT,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            WideCharToMultiByte(
                CP_THREAD_ACP,
                0,
                (*info_i).EnumLogFontEx.elfStyle.as_ptr(),
                -1,
                EnumLogFontExA.elfStyle.as_mut_ptr() as LPSTR,
                LF_FACESIZE as INT,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            WideCharToMultiByte(
                CP_THREAD_ACP,
                0,
                (*info_i).EnumLogFontEx.elfScript.as_ptr(),
                -1,
                EnumLogFontExA.elfScript.as_mut_ptr() as LPSTR,
                LF_FACESIZE as INT,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            NewTextMetricExW2A(&mut NewTextMetricExA, &mut (*info_i).NewTextMetricEx);
            let proc_: FONTENUMPROCA_T = core::mem::transmute(EnumProc);
            Ret = proc_(
                &EnumLogFontExA,
                &NewTextMetricExA,
                (*info_i).FontType,
                lParam,
            );
        }

        if Ret == 0 {
            break;
        }
        i += 1;
    }

    RtlFreeHeap(GetProcessHeap() as PVOID, 0, Info as PVOID);

    Ret
}

// `LogFontW2A` (gdi32p.h NTAPI). Declared here (NTAPI → extern "system").
extern "system" {
    fn LogFontW2A(pA: LPLOGFONTA, pW: *const LOGFONTW);
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn EnumFontFamiliesExW(
    hdc: HDC,
    lpLogfont: LPLOGFONTW,
    lpEnumFontFamExProc: PVOID, // FONTENUMPROCW (fn-ptr passed as PVOID, like the C cast)
    lParam: LPARAM,
    _dwFlags: DWORD,
) -> INT {
    // DPRINT(...) dropped
    IntEnumFontFamilies(
        hdc,
        lpLogfont,
        lpEnumFontFamExProc,
        lParam,
        IEFF_UNICODE | IEFF_EXTENDED,
    )
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn EnumFontFamiliesW(
    hdc: HDC,
    lpszFamily: LPCWSTR,
    lpEnumFontFamProc: PVOID, // FONTENUMPROCW
    lParam: LPARAM,
) -> INT {
    let mut LogFont: LOGFONTW = core::mem::zeroed();

    // DPRINT(...) dropped

    LogFont.lfCharSet = DEFAULT_CHARSET as BYTE;
    if !lpszFamily.is_null() {
        if *lpszFamily == 0 {
            return 1;
        }
        lstrcpynW(LogFont.lfFaceName.as_mut_ptr(), lpszFamily, LF_FACESIZE as INT);
    }

    IntEnumFontFamilies(hdc, &LogFont, lpEnumFontFamProc, lParam, IEFF_UNICODE)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn EnumFontFamiliesExA(
    hdc: HDC,
    lpLogfont: LPLOGFONTA,
    lpEnumFontFamExProc: PVOID, // FONTENUMPROCA
    lParam: LPARAM,
    _dwFlags: DWORD,
) -> INT {
    let mut LogFontW: LOGFONTW = core::mem::zeroed();
    let pLogFontW: *const LOGFONTW;

    // DPRINT(...) dropped

    if !lpLogfont.is_null() {
        LogFontA2W(&mut LogFontW, lpLogfont);
        pLogFontW = &LogFontW;
    } else {
        pLogFontW = core::ptr::null();
    }

    /* no need to convert LogFontW back to lpLogFont b/c it's an [in] parameter only */
    IntEnumFontFamilies(hdc, pLogFontW, lpEnumFontFamExProc, lParam, IEFF_EXTENDED)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn EnumFontFamiliesA(
    hdc: HDC,
    lpszFamily: LPCSTR,
    lpEnumFontFamProc: PVOID, // FONTENUMPROCA
    lParam: LPARAM,
) -> INT {
    let mut LogFont: LOGFONTW = core::mem::zeroed();

    // DPRINT(...) dropped

    LogFont.lfCharSet = DEFAULT_CHARSET as BYTE;
    if !lpszFamily.is_null() {
        if *lpszFamily == 0 {
            return 1;
        }
        MultiByteToWideChar(
            CP_THREAD_ACP,
            0,
            lpszFamily,
            -1,
            LogFont.lfFaceName.as_mut_ptr(),
            LF_FACESIZE as INT,
        );
    }

    IntEnumFontFamilies(hdc, &LogFont, lpEnumFontFamProc, lParam, 0)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetCharacterPlacementA(
    hdc: HDC,
    lpString: LPCSTR,
    uCount: INT,
    nMaxExtent: INT,
    lpResults: *mut GCP_RESULTSA,
    dwFlags: DWORD,
) -> DWORD {
    let lpStringW: *mut WCHAR;
    let mut uCountW: INT = 0;
    let mut resultsW: GCP_RESULTSW = core::mem::zeroed();
    let ret: DWORD;
    let mut font_cp: UINT = 0;

    if lpString.is_null()
        || uCount <= 0
        || lpResults.is_null()
        || (nMaxExtent < 0 && nMaxExtent != -1)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    /* both structs are equal in size */
    core::ptr::copy_nonoverlapping(
        lpResults as *const u8,
        &mut resultsW as *mut GCP_RESULTSW as *mut u8,
        core::mem::size_of::<GCP_RESULTSW>(),
    );

    lpStringW = FONT_mbtowc(hdc, lpString, uCount, &mut uCountW, &mut font_cp);
    if lpStringW.is_null() {
        return 0;
    }
    if !(*lpResults).lpOutString.is_null() {
        resultsW.lpOutString = HeapAlloc(
            GetProcessHeap(),
            0,
            (core::mem::size_of::<WCHAR>() * uCountW as usize) as SIZE_T,
        ) as LPWSTR;
        if resultsW.lpOutString.is_null() {
            HeapFree(GetProcessHeap(), 0, lpStringW as LPVOID);
            return 0;
        }
    }

    ret = GetCharacterPlacementW(hdc, lpStringW, uCountW, nMaxExtent, &mut resultsW, dwFlags);

    (*lpResults).nGlyphs = resultsW.nGlyphs;
    (*lpResults).nMaxFit = resultsW.nMaxFit;

    if !(*lpResults).lpOutString.is_null() {
        WideCharToMultiByte(
            font_cp,
            0,
            resultsW.lpOutString,
            uCountW,
            (*lpResults).lpOutString,
            uCount,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
    }

    HeapFree(GetProcessHeap(), 0, lpStringW as LPVOID);
    HeapFree(GetProcessHeap(), 0, resultsW.lpOutString as LPVOID);

    ret
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetCharacterPlacementW(
    hdc: HDC,
    lpString: LPCWSTR,
    uCount: INT,
    nMaxExtent: INT,
    lpResults: *mut GCP_RESULTSW,
    dwFlags: DWORD,
) -> DWORD {
    let mut ret: DWORD = 0;
    let mut size: SIZE = core::mem::zeroed();
    let mut i: UINT;
    let mut nSet: UINT;
    // DPRINT("GetCharacterPlacementW\n");

    // if (dwFlags&(~GCP_REORDER)) DPRINT(...)
    // if (lpResults->lpClass) DPRINT(...)

    nSet = uCount as UINT;
    if nSet > (*lpResults).nGlyphs {
        nSet = (*lpResults).nGlyphs;
    }

    /* return number of initialized fields */
    (*lpResults).nGlyphs = nSet;

    if dwFlags & GCP_REORDER != 0 {
        if LoadLPK(LPK_GCP) != FALSE {
            return LpkGetCharacterPlacement(
                hdc, lpString, uCount, nMaxExtent, lpResults, dwFlags, 0,
            );
        }
    }

    /* Treat the case where no special handling was requested in a fastpath way */
    /* copy will do if the GCP_REORDER flag is not set */
    if !(*lpResults).lpOutString.is_null() {
        lstrcpynW((*lpResults).lpOutString, lpString, nSet as INT);
    }

    if !(*lpResults).lpOrder.is_null() {
        i = 0;
        while i < nSet {
            *(*lpResults).lpOrder.offset(i as isize) = i;
            i += 1;
        }
    }

    /* FIXME: Will use the placement chars */
    if !(*lpResults).lpDx.is_null() {
        let mut c: INT = 0;
        i = 0;
        while i < nSet {
            if GetCharWidth32W(
                hdc,
                *lpString.offset(i as isize) as UINT,
                *lpString.offset(i as isize) as UINT,
                &mut c,
            ) != FALSE
            {
                *(*lpResults).lpDx.offset(i as isize) = c;
            }
            i += 1;
        }
    }

    if !(*lpResults).lpCaretPos.is_null() && (dwFlags & GCP_REORDER) == 0 {
        let mut pos: INT = 0;

        *(*lpResults).lpCaretPos.offset(0) = 0;
        i = 1;
        while i < nSet {
            if GetTextExtentPoint32W(hdc, lpString.offset((i - 1) as isize), 1, &mut size) != FALSE
            {
                pos += size.cx;
                *(*lpResults).lpCaretPos.offset(i as isize) = pos;
            }
            i += 1;
        }
    }

    if !(*lpResults).lpGlyphs.is_null() {
        NtGdiGetGlyphIndicesW(hdc, lpString, nSet as INT, (*lpResults).lpGlyphs as PWORD, 0);
    }

    if GetTextExtentPoint32W(hdc, lpString, uCount, &mut size) != FALSE {
        ret = MAKELONG(size.cx as USHORT, size.cy as USHORT);
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn NewGetCharacterPlacementW(
    hdc: HDC,
    lpString: LPCWSTR,
    uCount: INT,
    nMaxExtent: INT,
    lpResults: *mut GCP_RESULTSW,
    dwFlags: DWORD,
) -> DWORD {
    let mut nSet: ULONG;
    let mut Size: SIZE = core::mem::zeroed(); // {0,0}

    if lpString.is_null() || uCount <= 0 || (nMaxExtent < 0 && nMaxExtent != -1) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if lpResults.is_null() {
        if GetTextExtentPointW(hdc, lpString, uCount, &mut Size) != FALSE {
            return MAKELONG(Size.cx as USHORT, Size.cy as USHORT);
        }
        return 0;
    }

    nSet = uCount as ULONG;
    if nSet > (*lpResults).nGlyphs as ULONG {
        nSet = (*lpResults).nGlyphs as ULONG;
    }

    NtGdiGetCharacterPlacementW(
        hdc,
        lpString,
        nSet as INT,
        nMaxExtent,
        lpResults as LPGCP_RESULTSW,
        dwFlags,
    )
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetCharABCWidthsFloatW(
    hdc: HDC,
    FirstChar: UINT,
    LastChar: UINT,
    abcF: LPABCFLOAT,
) -> BOOL {
    // DPRINT("GetCharABCWidthsFloatW\n");
    if abcF.is_null() || (FirstChar > LastChar) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    NtGdiGetCharABCWidthsW(
        hdc,
        FirstChar,
        (LastChar - FirstChar + 1) as ULONG,
        core::ptr::null(),
        0,
        abcF as PVOID,
    )
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetCharWidthFloatW(
    hdc: HDC,
    iFirstChar: UINT,
    iLastChar: UINT,
    pxBuffer: PFLOAT,
) -> BOOL {
    // DPRINT("GetCharWidthsFloatW\n");
    if pxBuffer.is_null() || (iFirstChar > iLastChar) || (iLastChar >> 16) != 0 /* HIWORD */
        || !is_live_dc(hdc)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    NtGdiGetCharWidthW(
        hdc,
        iFirstChar,
        iLastChar - iFirstChar + 1,
        core::ptr::null(),
        0,
        pxBuffer as PVOID,
    )
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetCharWidthW(
    hdc: HDC,
    iFirstChar: UINT,
    iLastChar: UINT,
    lpBuffer: LPINT,
) -> BOOL {
    // DPRINT("GetCharWidthsW\n");
    if lpBuffer.is_null() || (iFirstChar > iLastChar) || (iLastChar >> 16) != 0 /* HIWORD */
        || !is_live_dc(hdc)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    NtGdiGetCharWidthW(
        hdc,
        iFirstChar,
        iLastChar - iFirstChar + 1,
        core::ptr::null(),
        GCW_NOFLOAT as FLONG,
        lpBuffer as PVOID,
    )
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetCharWidth32W(
    hdc: HDC,
    iFirstChar: UINT,
    iLastChar: UINT,
    lpBuffer: LPINT,
) -> BOOL {
    // DPRINT("GetCharWidths32W\n");
    if lpBuffer.is_null() || (iFirstChar > iLastChar) || (iLastChar >> 16) != 0 /* HIWORD */
        || !is_live_dc(hdc)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    NtGdiGetCharWidthW(
        hdc,
        iFirstChar,
        iLastChar - iFirstChar + 1,
        core::ptr::null(),
        (GCW_NOFLOAT | GCW_WIN32) as FLONG,
        lpBuffer as PVOID,
    )
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetCharABCWidthsW(
    hdc: HDC,
    FirstChar: UINT,
    LastChar: UINT,
    lpabc: LPABC,
) -> BOOL {
    // DPRINT("GetCharABCWidthsW\n");
    if lpabc.is_null() || (FirstChar > LastChar) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    NtGdiGetCharABCWidthsW(
        hdc,
        FirstChar,
        (LastChar - FirstChar + 1) as ULONG,
        core::ptr::null(),
        GCABCW_NOFLOAT as FLONG,
        lpabc as PVOID,
    )
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetCharWidthA(
    hdc: HDC,
    iFirstChar: UINT,
    iLastChar: UINT,
    lpBuffer: LPINT,
) -> BOOL {
    let mut wlen: INT = 0;
    let mut count: INT = 0;
    let str_: LPSTR;
    let wstr: LPWSTR;
    let ret: BOOL;

    // DPRINT("GetCharWidthsA\n");

    if lpBuffer.is_null() || !is_live_dc(hdc) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    str_ = FONT_GetCharsByRangeA(hdc, iFirstChar, iLastChar, &mut count);
    if str_.is_null() {
        return FALSE;
    }

    wstr = FONT_mbtowc(hdc, str_, count + 1, &mut wlen, core::ptr::null_mut());
    if wstr.is_null() {
        HeapFree(GetProcessHeap(), 0, str_ as LPVOID);
        return FALSE;
    }
    let _ = wlen;

    ret = NtGdiGetCharWidthW(
        hdc,
        *wstr as UINT,
        count as UINT,
        wstr,
        GCW_NOFLOAT as FLONG,
        lpBuffer as PVOID,
    );

    HeapFree(GetProcessHeap(), 0, str_ as LPVOID);
    HeapFree(GetProcessHeap(), 0, wstr as LPVOID);

    ret
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetCharWidth32A(
    hdc: HDC,
    iFirstChar: UINT,
    iLastChar: UINT,
    lpBuffer: LPINT,
) -> BOOL {
    let mut count: INT = 0;
    let str_: LPSTR;
    let wstr: LPWSTR;
    let ret: BOOL;

    // DPRINT("GetCharWidths32A\n");

    if lpBuffer.is_null() || !is_live_dc(hdc) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    str_ = FONT_GetCharsByRangeA(hdc, iFirstChar, iLastChar, &mut count);
    if str_.is_null() {
        return FALSE;
    }

    let mut wlen: INT = 0;
    wstr = FONT_mbtowc(hdc, str_, count + 1, &mut wlen, core::ptr::null_mut());
    if wstr.is_null() {
        HeapFree(GetProcessHeap(), 0, str_ as LPVOID);
        return FALSE;
    }

    ret = NtGdiGetCharWidthW(
        hdc,
        *wstr as UINT,
        count as UINT,
        wstr,
        (GCW_NOFLOAT | GCW_WIN32) as FLONG,
        lpBuffer as PVOID,
    );

    HeapFree(GetProcessHeap(), 0, str_ as LPVOID);
    HeapFree(GetProcessHeap(), 0, wstr as LPVOID);

    ret
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetCharWidthFloatA(
    hdc: HDC,
    iFirstChar: UINT,
    iLastChar: UINT,
    pxBuffer: PFLOAT,
) -> BOOL {
    let mut count: INT = 0;
    let str_: LPSTR;
    let wstr: LPWSTR;
    let ret: BOOL;

    // DPRINT("GetCharWidthsFloatA\n");

    if pxBuffer.is_null() || !is_live_dc(hdc) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    str_ = FONT_GetCharsByRangeA(hdc, iFirstChar, iLastChar, &mut count);
    if str_.is_null() {
        return FALSE;
    }

    let mut wlen: INT = 0;
    wstr = FONT_mbtowc(hdc, str_, count + 1, &mut wlen, core::ptr::null_mut());
    if wstr.is_null() {
        HeapFree(GetProcessHeap(), 0, str_ as LPVOID);
        return FALSE;
    }
    ret = NtGdiGetCharWidthW(hdc, *wstr as UINT, count as UINT, wstr, 0, pxBuffer as PVOID);

    HeapFree(GetProcessHeap(), 0, str_ as LPVOID);
    HeapFree(GetProcessHeap(), 0, wstr as LPVOID);

    ret
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetCharABCWidthsA(
    hdc: HDC,
    iFirstChar: UINT,
    iLastChar: UINT,
    lpabc: LPABC,
) -> BOOL {
    let mut count: INT = 0;
    let str_: LPSTR;
    let wstr: LPWSTR;
    let ret: BOOL;

    // DPRINT("GetCharABCWidthsA\n");

    str_ = FONT_GetCharsByRangeA(hdc, iFirstChar, iLastChar, &mut count);
    if str_.is_null() {
        return FALSE;
    }

    let mut wlen: INT = 0;
    wstr = FONT_mbtowc(hdc, str_, count + 1, &mut wlen, core::ptr::null_mut());
    if wstr.is_null() {
        HeapFree(GetProcessHeap(), 0, str_ as LPVOID);
        return FALSE;
    }

    ret = NtGdiGetCharABCWidthsW(
        hdc,
        *wstr as UINT,
        (wlen - 1) as ULONG,
        wstr as PCWCH,
        GCABCW_NOFLOAT as FLONG,
        lpabc as PVOID,
    );

    HeapFree(GetProcessHeap(), 0, str_ as LPVOID);
    HeapFree(GetProcessHeap(), 0, wstr as LPVOID);

    ret
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetCharABCWidthsFloatA(
    hdc: HDC,
    iFirstChar: UINT,
    iLastChar: UINT,
    lpABCF: LPABCFLOAT,
) -> BOOL {
    let mut count: INT = 0;
    let str_: LPSTR;
    let wstr: LPWSTR;
    let ret: BOOL;

    // DPRINT("GetCharABCWidthsFloatA\n");

    str_ = FONT_GetCharsByRangeA(hdc, iFirstChar, iLastChar, &mut count);
    if str_.is_null() {
        return FALSE;
    }

    let mut wlen: INT = 0;
    wstr = FONT_mbtowc(hdc, str_, count + 1, &mut wlen, core::ptr::null_mut());
    if wstr.is_null() {
        HeapFree(GetProcessHeap(), 0, str_ as LPVOID);
        return FALSE;
    }
    ret = NtGdiGetCharABCWidthsW(hdc, *wstr as UINT, count as ULONG, wstr, 0, lpABCF as PVOID);

    HeapFree(GetProcessHeap(), 0, str_ as LPVOID);
    HeapFree(GetProcessHeap(), 0, wstr as LPVOID);

    ret
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetCharABCWidthsI(
    hdc: HDC,
    giFirst: UINT,
    cgi: UINT,
    pgi: LPWORD,
    lpabc: LPABC,
) -> BOOL {
    // DPRINT("GetCharABCWidthsI\n");
    NtGdiGetCharABCWidthsW(
        hdc,
        giFirst,
        cgi as ULONG,
        pgi as PCWCH,
        (GCABCW_NOFLOAT | GCABCW_INDICES) as FLONG,
        lpabc as PVOID,
    )
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetCharWidthI(
    hdc: HDC,
    giFirst: UINT,
    cgi: UINT,
    pgi: LPWORD,
    lpBuffer: LPINT,
) -> BOOL {
    // DPRINT("GetCharWidthsI\n");
    if lpBuffer.is_null() || (pgi.is_null() && (giFirst >> 16) != 0 /* HIWORD */) {
        // Cannot be at max.
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    if cgi == 0 {
        return TRUE;
    }
    NtGdiGetCharWidthW(
        hdc,
        giFirst,
        cgi,
        pgi as PCWCH,
        (GCW_INDICES | GCW_NOFLOAT | GCW_WIN32) as FLONG,
        lpBuffer as PVOID,
    )
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetFontLanguageInfo(hDc: HDC) -> DWORD {
    let Gcp: DWORD;
    let mut Ret: DWORD = 0;
    if gbLpk != FALSE {
        Ret = NtGdiGetTextCharsetInfo(hDc, core::ptr::null_mut(), 0) as DWORD;
        if (Ret == ARABIC_CHARSET as DWORD) || (Ret == HEBREW_CHARSET as DWORD) {
            Ret = GCP_KASHIDA | GCP_DIACRITIC | GCP_LIGATE | GCP_GLYPHSHAPE | GCP_REORDER;
        }
    }
    Gcp = GetDCDWord(
        hDc,
        _GETDCDWORD_GdiGetFontLanguageInfo as UINT,
        GCP_ERROR,
    );
    if Gcp == GCP_ERROR {
        Gcp
    } else {
        Ret = Gcp | Ret;
        Ret
    }
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetGlyphIndicesA(
    hdc: HDC,
    lpstr: LPCSTR,
    count: INT,
    pgi: LPWORD,
    flags: DWORD,
) -> DWORD {
    let Ret: DWORD;
    let lpstrW: *mut WCHAR;
    let mut countW: INT = 0;

    if count > 0 && !is_live_dc(hdc) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return GDI_ERROR;
    }

    lpstrW = FONT_mbtowc(hdc, lpstr, count, &mut countW, core::ptr::null_mut());

    if lpstrW.is_null() {
        return GDI_ERROR;
    }

    Ret = NtGdiGetGlyphIndicesW(hdc, lpstrW, countW, pgi as PWORD, flags);
    HeapFree(GetProcessHeap(), 0, lpstrW as LPVOID);
    Ret
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetGlyphOutlineA(
    hdc: HDC,
    uChar: UINT,
    uFormat: UINT,
    lpgm: LPGLYPHMETRICS,
    cbBuffer: DWORD,
    lpvBuffer: LPVOID,
    lpmat2: *const MAT2,
) -> DWORD {
    let mut p: LPWSTR = core::ptr::null_mut();
    let ret: DWORD;
    let c: UINT;
    // DPRINT("GetGlyphOutlineA uChar %x\n", uChar);
    if lpgm.is_null() || lpmat2.is_null() {
        return GDI_ERROR;
    }
    if (uFormat & GGO_GLYPH_INDEX) == 0 {
        let len: INT;
        let mut mbchs: [CHAR; 2] = [0; 2];
        if uChar > 0xff {
            /* but, 2 bytes character only */
            len = 2;
            mbchs[0] = ((uChar & 0xff00) >> 8) as CHAR;
            mbchs[1] = (uChar & 0xff) as CHAR;
        } else {
            len = 1;
            mbchs[0] = (uChar & 0xff) as CHAR;
        }
        p = FONT_mbtowc(hdc, mbchs.as_ptr(), len, core::ptr::null_mut(), core::ptr::null_mut());
        if p.is_null() {
            return GDI_ERROR;
        }
        c = *p as UINT;
    } else {
        c = uChar;
    }
    ret = NtGdiGetGlyphOutline(
        hdc,
        c as WCHAR,
        uFormat,
        lpgm,
        cbBuffer,
        lpvBuffer,
        lpmat2 as *const MAT2,
        TRUE,
    );
    HeapFree(GetProcessHeap(), 0, p as LPVOID);
    ret
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetGlyphOutlineW(
    hdc: HDC,
    uChar: UINT,
    uFormat: UINT,
    lpgm: LPGLYPHMETRICS,
    mut cbBuffer: DWORD,
    lpvBuffer: LPVOID,
    lpmat2: *const MAT2,
) -> DWORD {
    // DPRINT("GetGlyphOutlineW uChar %x\n", uChar);
    if lpgm.is_null() || lpmat2.is_null() {
        return GDI_ERROR;
    }
    if lpvBuffer.is_null() {
        cbBuffer = 0;
    }
    NtGdiGetGlyphOutline(
        hdc,
        uChar as WCHAR,
        uFormat,
        lpgm,
        cbBuffer,
        lpvBuffer,
        lpmat2 as *const MAT2,
        TRUE,
    )
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetGlyphOutlineWow(
    _a0: DWORD,
    _a1: DWORD,
    _a2: DWORD,
    _a3: DWORD,
    _a4: DWORD,
    _a5: DWORD,
    _a6: DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetOutlineTextMetricsA(
    hdc: HDC,
    cbData: UINT,
    lpOTM: LPOUTLINETEXTMETRICA,
) -> UINT {
    let mut buf: [u8; 512] = [0; 512];
    let mut ret: UINT;
    let mut needed: UINT;
    let mut lpOTMW: *mut OUTLINETEXTMETRICW = buf.as_mut_ptr() as *mut OUTLINETEXTMETRICW;
    let mut output: LPOUTLINETEXTMETRICA = lpOTM;
    let mut left: INT;
    let mut len: INT;

    ret = GetOutlineTextMetricsW(hdc, 0, core::ptr::null_mut());
    if ret == 0 {
        return 0;
    }
    if ret as usize > core::mem::size_of_val(&buf) {
        lpOTMW = HeapAlloc(GetProcessHeap(), 0, ret as SIZE_T) as *mut OUTLINETEXTMETRICW;
        if lpOTMW.is_null() {
            return 0;
        }
    }
    GetOutlineTextMetricsW(hdc, ret, lpOTMW);

    needed = core::mem::size_of::<OUTLINETEXTMETRICA>() as UINT;
    if !((*lpOTMW).otmpFamilyName.is_null()) {
        needed += WideCharToMultiByte(
            CP_ACP,
            0,
            (lpOTMW as *mut u8).offset((*lpOTMW).otmpFamilyName as isize) as *const u16 as LPCWSTR,
            -1,
            core::ptr::null_mut(),
            0,
            core::ptr::null(),
            core::ptr::null_mut(),
        ) as UINT;
    }
    if !((*lpOTMW).otmpFaceName.is_null()) {
        needed += WideCharToMultiByte(
            CP_ACP,
            0,
            (lpOTMW as *mut u8).offset((*lpOTMW).otmpFaceName as isize) as *const u16 as LPCWSTR,
            -1,
            core::ptr::null_mut(),
            0,
            core::ptr::null(),
            core::ptr::null_mut(),
        ) as UINT;
    }
    if !((*lpOTMW).otmpStyleName.is_null()) {
        needed += WideCharToMultiByte(
            CP_ACP,
            0,
            (lpOTMW as *mut u8).offset((*lpOTMW).otmpStyleName as isize) as *const u16 as LPCWSTR,
            -1,
            core::ptr::null_mut(),
            0,
            core::ptr::null(),
            core::ptr::null_mut(),
        ) as UINT;
    }
    if !((*lpOTMW).otmpFullName.is_null()) {
        needed += WideCharToMultiByte(
            CP_ACP,
            0,
            (lpOTMW as *mut u8).offset((*lpOTMW).otmpFullName as isize) as *const u16 as LPCWSTR,
            -1,
            core::ptr::null_mut(),
            0,
            core::ptr::null(),
            core::ptr::null_mut(),
        ) as UINT;
    }

    'end: {
        if lpOTM.is_null() {
            ret = needed;
            break 'end;
        }

        // DPRINT("needed = %u\n", needed);
        if needed > cbData {
            /* Since the supplied buffer isn't big enough, we'll alloc one
            that is and memcpy the first cbData bytes into the lpOTM at
            the end. */
            output = HeapAlloc(GetProcessHeap(), 0, needed as SIZE_T) as LPOUTLINETEXTMETRICA;
            if output.is_null() {
                break 'end;
            }
        }

        (*output).otmSize = if needed < cbData { needed } else { cbData };
        ret = (*output).otmSize;
        FONT_TextMetricWToA(
            &(*lpOTMW).otmTextMetrics,
            &mut (*output).otmTextMetrics,
        );
        (*output).otmFiller = 0;
        (*output).otmPanoseNumber = (*lpOTMW).otmPanoseNumber;
        (*output).otmfsSelection = (*lpOTMW).otmfsSelection;
        (*output).otmfsType = (*lpOTMW).otmfsType;
        (*output).otmsCharSlopeRise = (*lpOTMW).otmsCharSlopeRise;
        (*output).otmsCharSlopeRun = (*lpOTMW).otmsCharSlopeRun;
        (*output).otmItalicAngle = (*lpOTMW).otmItalicAngle;
        (*output).otmEMSquare = (*lpOTMW).otmEMSquare;
        (*output).otmAscent = (*lpOTMW).otmAscent;
        (*output).otmDescent = (*lpOTMW).otmDescent;
        (*output).otmLineGap = (*lpOTMW).otmLineGap;
        (*output).otmsCapEmHeight = (*lpOTMW).otmsCapEmHeight;
        (*output).otmsXHeight = (*lpOTMW).otmsXHeight;
        (*output).otmrcFontBox = (*lpOTMW).otmrcFontBox;
        (*output).otmMacAscent = (*lpOTMW).otmMacAscent;
        (*output).otmMacDescent = (*lpOTMW).otmMacDescent;
        (*output).otmMacLineGap = (*lpOTMW).otmMacLineGap;
        (*output).otmusMinimumPPEM = (*lpOTMW).otmusMinimumPPEM;
        (*output).otmptSubscriptSize = (*lpOTMW).otmptSubscriptSize;
        (*output).otmptSubscriptOffset = (*lpOTMW).otmptSubscriptOffset;
        (*output).otmptSuperscriptSize = (*lpOTMW).otmptSuperscriptSize;
        (*output).otmptSuperscriptOffset = (*lpOTMW).otmptSuperscriptOffset;
        (*output).otmsStrikeoutSize = (*lpOTMW).otmsStrikeoutSize;
        (*output).otmsStrikeoutPosition = (*lpOTMW).otmsStrikeoutPosition;
        (*output).otmsUnderscoreSize = (*lpOTMW).otmsUnderscoreSize;
        (*output).otmsUnderscorePosition = (*lpOTMW).otmsUnderscorePosition;

        let mut ptr_local: *mut u8 =
            (output as *mut u8).add(core::mem::size_of::<OUTLINETEXTMETRICA>());
        left = needed as INT - core::mem::size_of::<OUTLINETEXTMETRICA>() as INT;

        if !((*lpOTMW).otmpFamilyName.is_null()) {
            (*output).otmpFamilyName = (ptr_local as usize - output as usize) as PSTR;
            len = WideCharToMultiByte(
                CP_ACP,
                0,
                (lpOTMW as *mut u8).offset((*lpOTMW).otmpFamilyName as isize) as *const u16
                    as LPCWSTR,
                -1,
                ptr_local as LPSTR,
                left,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            left -= len;
            ptr_local = ptr_local.offset(len as isize);
        } else {
            (*output).otmpFamilyName = core::ptr::null_mut();
        }

        if !((*lpOTMW).otmpFaceName.is_null()) {
            (*output).otmpFaceName = (ptr_local as usize - output as usize) as PSTR;
            len = WideCharToMultiByte(
                CP_ACP,
                0,
                (lpOTMW as *mut u8).offset((*lpOTMW).otmpFaceName as isize) as *const u16 as LPCWSTR,
                -1,
                ptr_local as LPSTR,
                left,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            left -= len;
            ptr_local = ptr_local.offset(len as isize);
        } else {
            (*output).otmpFaceName = core::ptr::null_mut();
        }

        if !((*lpOTMW).otmpStyleName.is_null()) {
            (*output).otmpStyleName = (ptr_local as usize - output as usize) as PSTR;
            len = WideCharToMultiByte(
                CP_ACP,
                0,
                (lpOTMW as *mut u8).offset((*lpOTMW).otmpStyleName as isize) as *const u16
                    as LPCWSTR,
                -1,
                ptr_local as LPSTR,
                left,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            left -= len;
            ptr_local = ptr_local.offset(len as isize);
        } else {
            (*output).otmpStyleName = core::ptr::null_mut();
        }

        if !((*lpOTMW).otmpFullName.is_null()) {
            (*output).otmpFullName = (ptr_local as usize - output as usize) as PSTR;
            len = WideCharToMultiByte(
                CP_ACP,
                0,
                (lpOTMW as *mut u8).offset((*lpOTMW).otmpFullName as isize) as *const u16 as LPCWSTR,
                -1,
                ptr_local as LPSTR,
                left,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            left -= len;
        } else {
            (*output).otmpFullName = core::ptr::null_mut();
        }

        // ASSERT(left == 0);
        let _ = left;

        if output != lpOTM {
            core::ptr::copy_nonoverlapping(output as *const u8, lpOTM as *mut u8, cbData as usize);
            HeapFree(GetProcessHeap(), 0, output as LPVOID);

            /* check if the string offsets really fit into the provided size */
            /* FIXME: should we check string length as well? */
            if (*lpOTM).otmpFamilyName as usize >= (*lpOTM).otmSize as usize {
                (*lpOTM).otmpFamilyName = core::ptr::null_mut(); /* doesn't fit */
            }
            if (*lpOTM).otmpFaceName as usize >= (*lpOTM).otmSize as usize {
                (*lpOTM).otmpFaceName = core::ptr::null_mut(); /* doesn't fit */
            }
            if (*lpOTM).otmpStyleName as usize >= (*lpOTM).otmSize as usize {
                (*lpOTM).otmpStyleName = core::ptr::null_mut(); /* doesn't fit */
            }
            if (*lpOTM).otmpFullName as usize >= (*lpOTM).otmSize as usize {
                (*lpOTM).otmpFullName = core::ptr::null_mut(); /* doesn't fit */
            }
        }
    }
    // end:
    if lpOTMW != (buf.as_mut_ptr() as *mut OUTLINETEXTMETRICW) {
        HeapFree(GetProcessHeap(), 0, lpOTMW as LPVOID);
    }

    ret
}

/* Performs a device to world transformation on the specified size (which
 * is in integer format).
 */
unsafe fn INTERNAL_YDSTOWS(xForm: *mut XFORM, height: INT) -> INT {
    let floatHeight: f64;

    /* Perform operation with floating point */
    floatHeight = height as f64 * (*xForm).eM22 as f64;
    /* Round to integers */
    GDI_ROUND!(floatHeight)
}

/* scale width and height but don't mirror them */
unsafe fn width_to_LP(xForm: *mut XFORM, width: INT) -> INT {
    GDI_ROUND!(width as f64 * dabs!((*xForm).eM11 as f64))
}

unsafe fn height_to_LP(xForm: *mut XFORM, height: INT) -> INT {
    GDI_ROUND!(height as f64 * dabs!((*xForm).eM22 as f64))
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetOutlineTextMetricsW(
    hdc: HDC,
    cbData: UINT,
    lpOTM: LPOUTLINETEXTMETRICW,
) -> UINT {
    let mut Tmd: TMDIFF = core::mem::zeroed(); // Should not be zero.
    let mut Size: UINT;
    let mut AvailableSize: UINT;
    let mut StringSize: UINT;
    let mut DevToWorld: XFORM = core::mem::zeroed();
    let LocalOTM: *mut OUTLINETEXTMETRICW;
    let mut Str: *mut WCHAR;
    let mut Ptr: *mut BYTE;

    /* Get the structure */
    Size = NtGdiGetOutlineTextMetricsInternalW(hdc, 0, core::ptr::null_mut(), &mut Tmd);
    if Size == 0 {
        return 0;
    }
    if lpOTM.is_null() || (cbData < core::mem::size_of::<OUTLINETEXTMETRICW>() as UINT) {
        return Size;
    }

    LocalOTM = HeapAlloc(GetProcessHeap(), 0, Size as SIZE_T) as *mut OUTLINETEXTMETRICW;
    (*LocalOTM).otmSize = Size;
    Size = NtGdiGetOutlineTextMetricsInternalW(hdc, Size, LocalOTM, &mut Tmd);
    if Size == 0 {
        HeapFree(GetProcessHeap(), 0, LocalOTM as LPVOID);
        return 0;
    }

    if NtGdiGetTransform(hdc, GdiDeviceSpaceToWorldSpace, &mut DevToWorld) == FALSE {
        // DPRINT1("NtGdiGetTransform failed!\n");
        HeapFree(GetProcessHeap(), 0, LocalOTM as LPVOID);
        SetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    /* Fill in DC specific data */
    (*LocalOTM).otmTextMetrics.tmDigitizedAspectX = GetDeviceCaps(hdc, LOGPIXELSX as INT) as LONG;
    (*LocalOTM).otmTextMetrics.tmDigitizedAspectY = GetDeviceCaps(hdc, LOGPIXELSY as INT) as LONG;
    (*LocalOTM).otmTextMetrics.tmHeight =
        height_to_LP(&mut DevToWorld, (*LocalOTM).otmTextMetrics.tmHeight) as LONG;
    (*LocalOTM).otmTextMetrics.tmAscent =
        height_to_LP(&mut DevToWorld, (*LocalOTM).otmTextMetrics.tmAscent) as LONG;
    (*LocalOTM).otmTextMetrics.tmDescent =
        height_to_LP(&mut DevToWorld, (*LocalOTM).otmTextMetrics.tmDescent) as LONG;
    (*LocalOTM).otmTextMetrics.tmInternalLeading =
        height_to_LP(&mut DevToWorld, (*LocalOTM).otmTextMetrics.tmInternalLeading) as LONG;
    (*LocalOTM).otmTextMetrics.tmExternalLeading =
        height_to_LP(&mut DevToWorld, (*LocalOTM).otmTextMetrics.tmExternalLeading) as LONG;
    (*LocalOTM).otmTextMetrics.tmAveCharWidth =
        width_to_LP(&mut DevToWorld, (*LocalOTM).otmTextMetrics.tmAveCharWidth) as LONG;
    (*LocalOTM).otmTextMetrics.tmMaxCharWidth =
        width_to_LP(&mut DevToWorld, (*LocalOTM).otmTextMetrics.tmMaxCharWidth) as LONG;
    (*LocalOTM).otmTextMetrics.tmOverhang =
        width_to_LP(&mut DevToWorld, (*LocalOTM).otmTextMetrics.tmOverhang) as LONG;
    (*LocalOTM).otmAscent = height_to_LP(&mut DevToWorld, (*LocalOTM).otmAscent);
    (*LocalOTM).otmDescent = height_to_LP(&mut DevToWorld, (*LocalOTM).otmDescent);
    (*LocalOTM).otmLineGap =
        (INTERNAL_YDSTOWS(&mut DevToWorld, (*LocalOTM).otmLineGap as INT)).abs() as UINT;
    (*LocalOTM).otmsCapEmHeight =
        (INTERNAL_YDSTOWS(&mut DevToWorld, (*LocalOTM).otmsCapEmHeight as INT)).abs() as UINT;
    (*LocalOTM).otmsXHeight =
        (INTERNAL_YDSTOWS(&mut DevToWorld, (*LocalOTM).otmsXHeight as INT)).abs() as UINT;
    (*LocalOTM).otmrcFontBox.top = height_to_LP(&mut DevToWorld, (*LocalOTM).otmrcFontBox.top);
    (*LocalOTM).otmrcFontBox.bottom =
        height_to_LP(&mut DevToWorld, (*LocalOTM).otmrcFontBox.bottom);
    (*LocalOTM).otmrcFontBox.left = width_to_LP(&mut DevToWorld, (*LocalOTM).otmrcFontBox.left);
    (*LocalOTM).otmrcFontBox.right = width_to_LP(&mut DevToWorld, (*LocalOTM).otmrcFontBox.right);
    (*LocalOTM).otmMacAscent = height_to_LP(&mut DevToWorld, (*LocalOTM).otmMacAscent);
    (*LocalOTM).otmMacDescent = height_to_LP(&mut DevToWorld, (*LocalOTM).otmMacDescent);
    (*LocalOTM).otmMacLineGap =
        (INTERNAL_YDSTOWS(&mut DevToWorld, (*LocalOTM).otmMacLineGap as INT)).abs() as UINT;
    (*LocalOTM).otmptSubscriptSize.x =
        width_to_LP(&mut DevToWorld, (*LocalOTM).otmptSubscriptSize.x);
    (*LocalOTM).otmptSubscriptSize.y =
        height_to_LP(&mut DevToWorld, (*LocalOTM).otmptSubscriptSize.y);
    (*LocalOTM).otmptSubscriptOffset.x =
        width_to_LP(&mut DevToWorld, (*LocalOTM).otmptSubscriptOffset.x);
    (*LocalOTM).otmptSubscriptOffset.y =
        height_to_LP(&mut DevToWorld, (*LocalOTM).otmptSubscriptOffset.y);
    (*LocalOTM).otmptSuperscriptSize.x =
        width_to_LP(&mut DevToWorld, (*LocalOTM).otmptSuperscriptSize.x);
    (*LocalOTM).otmptSuperscriptSize.y =
        height_to_LP(&mut DevToWorld, (*LocalOTM).otmptSuperscriptSize.y);
    (*LocalOTM).otmptSuperscriptOffset.x =
        width_to_LP(&mut DevToWorld, (*LocalOTM).otmptSuperscriptOffset.x);
    (*LocalOTM).otmptSuperscriptOffset.y =
        height_to_LP(&mut DevToWorld, (*LocalOTM).otmptSuperscriptOffset.y);
    (*LocalOTM).otmsStrikeoutSize =
        (INTERNAL_YDSTOWS(&mut DevToWorld, (*LocalOTM).otmsStrikeoutSize as INT)).abs() as UINT;
    (*LocalOTM).otmsStrikeoutPosition =
        height_to_LP(&mut DevToWorld, (*LocalOTM).otmsStrikeoutPosition);
    (*LocalOTM).otmsUnderscoreSize =
        height_to_LP(&mut DevToWorld, (*LocalOTM).otmsUnderscoreSize);
    (*LocalOTM).otmsUnderscorePosition =
        height_to_LP(&mut DevToWorld, (*LocalOTM).otmsUnderscorePosition);

    /* Copy what we can */
    core::ptr::copy_nonoverlapping(
        LocalOTM as *const u8,
        lpOTM as *mut u8,
        (if Size < cbData { Size } else { cbData }) as usize,
    );

    (*lpOTM).otmpFamilyName = core::ptr::null_mut();
    (*lpOTM).otmpFaceName = core::ptr::null_mut();
    (*lpOTM).otmpStyleName = core::ptr::null_mut();
    (*lpOTM).otmpFullName = core::ptr::null_mut();

    Size = core::mem::size_of::<OUTLINETEXTMETRICW>() as UINT;
    AvailableSize = cbData - Size;
    Ptr = (lpOTM as *mut BYTE).add(core::mem::size_of::<OUTLINETEXTMETRICW>());

    /* Fix string values up */
    if !((*LocalOTM).otmpFamilyName.is_null()) {
        Str = (LocalOTM as *mut u8).offset((*LocalOTM).otmpFamilyName as isize) as *mut WCHAR;
        StringSize = ((wcslen(Str) + 1) * core::mem::size_of::<WCHAR>()) as UINT;
        if AvailableSize >= StringSize {
            core::ptr::copy_nonoverlapping(Str as *const u8, Ptr, StringSize as usize);
            (*lpOTM).otmpFamilyName = (Ptr as usize - lpOTM as usize) as PSTR;
            Ptr = Ptr.offset(StringSize as isize);
            AvailableSize -= StringSize;
            Size += StringSize;
        }
    }

    if !((*LocalOTM).otmpFaceName.is_null()) {
        Str = (LocalOTM as *mut u8).offset((*LocalOTM).otmpFaceName as isize) as *mut WCHAR;
        StringSize = ((wcslen(Str) + 1) * core::mem::size_of::<WCHAR>()) as UINT;
        if AvailableSize >= StringSize {
            core::ptr::copy_nonoverlapping(Str as *const u8, Ptr, StringSize as usize);
            (*lpOTM).otmpFaceName = (Ptr as usize - lpOTM as usize) as PSTR;
            Ptr = Ptr.offset(StringSize as isize);
            AvailableSize -= StringSize;
            Size += StringSize;
        }
    }

    if !((*LocalOTM).otmpStyleName.is_null()) {
        Str = (LocalOTM as *mut u8).offset((*LocalOTM).otmpStyleName as isize) as *mut WCHAR;
        StringSize = ((wcslen(Str) + 1) * core::mem::size_of::<WCHAR>()) as UINT;
        if AvailableSize >= StringSize {
            core::ptr::copy_nonoverlapping(Str as *const u8, Ptr, StringSize as usize);
            (*lpOTM).otmpStyleName = (Ptr as usize - lpOTM as usize) as PSTR;
            Ptr = Ptr.offset(StringSize as isize);
            AvailableSize -= StringSize;
            Size += StringSize;
        }
    }

    if !((*LocalOTM).otmpFullName.is_null()) {
        Str = (LocalOTM as *mut u8).offset((*LocalOTM).otmpFullName as isize) as *mut WCHAR;
        StringSize = ((wcslen(Str) + 1) * core::mem::size_of::<WCHAR>()) as UINT;
        if AvailableSize >= StringSize {
            core::ptr::copy_nonoverlapping(Str as *const u8, Ptr, StringSize as usize);
            (*lpOTM).otmpFullName = (Ptr as usize - lpOTM as usize) as PSTR;
            Ptr = Ptr.offset(StringSize as isize);
            AvailableSize -= StringSize;
            Size += StringSize;
        }
    }
    let _ = (AvailableSize, Ptr);

    (*lpOTM).otmSize = Size;

    HeapFree(GetProcessHeap(), 0, LocalOTM as LPVOID);

    Size
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetKerningPairsW(
    hdc: HDC,
    cPairs: ULONG,
    pkpDst: LPKERNINGPAIR,
) -> DWORD {
    if (cPairs != 0) || (pkpDst.is_null()) {
        NtGdiGetKerningPairs(hdc, cPairs, pkpDst as *mut KERNINGPAIR)
    } else {
        SetLastError(ERROR_INVALID_PARAMETER);
        0
    }
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetKerningPairsA(
    hDC: HDC,
    cPairs: DWORD,
    mut kern_pairA: LPKERNINGPAIR,
) -> DWORD {
    let charset: INT;
    let mut csi: CHARSETINFO = core::mem::zeroed();
    let mut cpi: CPINFO = core::mem::zeroed();
    let mut i: DWORD;
    let total_kern_pairs: DWORD;
    let mut kern_pairs_copied: DWORD = 0;
    let kern_pairW: *mut KERNINGPAIR;

    if cPairs == 0 && !kern_pairA.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    charset = GetTextCharset(hDC);
    if TranslateCharsetInfo(
        (charset as ULONG) as usize as LPDWORD,
        &mut csi,
        TCI_SRCCHARSET,
    ) == FALSE
    {
        // DPRINT1("Can't find codepage for charset %d\n", charset);
        return 0;
    }
    /* GetCPInfo() will fail on CP_SYMBOL, and WideCharToMultiByte is supposed
     * to fail on an invalid character for CP_SYMBOL.
     */
    cpi.DefaultChar[0] = 0;
    if csi.ciACP != CP_SYMBOL && GetCPInfo(csi.ciACP, &mut cpi) == FALSE {
        // DPRINT1("Can't find codepage %u info\n", csi.ciACP);
        return 0;
    }
    // DPRINT("charset %d => codepage %u\n", charset, csi.ciACP);

    total_kern_pairs = NtGdiGetKerningPairs(hDC, 0, core::ptr::null_mut());
    if total_kern_pairs == 0 {
        return 0;
    }

    if cPairs == 0 && kern_pairA.is_null() {
        return total_kern_pairs;
    }

    kern_pairW = HeapAlloc(
        GetProcessHeap(),
        0,
        (total_kern_pairs as usize * core::mem::size_of::<KERNINGPAIR>()) as SIZE_T,
    ) as *mut KERNINGPAIR;
    if kern_pairW.is_null() {
        return 0;
    }
    GetKerningPairsW(hDC, total_kern_pairs, kern_pairW);

    i = 0;
    while i < total_kern_pairs {
        let mut first: CHAR = 0;
        let mut second: CHAR = 0;
        let kp: *mut KERNINGPAIR = kern_pairW.offset(i as isize);

        if WideCharToMultiByte(
            csi.ciACP,
            0,
            &(*kp).wFirst as *const WORD as LPCWSTR,
            1,
            &mut first as *mut CHAR as LPSTR,
            1,
            core::ptr::null(),
            core::ptr::null_mut(),
        ) == 0
        {
            i += 1;
            continue;
        }

        if WideCharToMultiByte(
            csi.ciACP,
            0,
            &(*kp).wSecond as *const WORD as LPCWSTR,
            1,
            &mut second as *mut CHAR as LPSTR,
            1,
            core::ptr::null(),
            core::ptr::null_mut(),
        ) == 0
        {
            i += 1;
            continue;
        }

        if first == cpi.DefaultChar[0] as CHAR || second == cpi.DefaultChar[0] as CHAR {
            i += 1;
            continue;
        }

        if !kern_pairA.is_null() {
            if kern_pairs_copied >= cPairs {
                break;
            }

            (*kern_pairA).wFirst = first as BYTE as WORD;
            (*kern_pairA).wSecond = second as BYTE as WORD;
            (*kern_pairA).iKernAmount = (*kp).iKernAmount;
            kern_pairA = kern_pairA.offset(1);
        }
        kern_pairs_copied += 1;
        i += 1;
    }

    HeapFree(GetProcessHeap(), 0, kern_pairW as LPVOID);

    kern_pairs_copied
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateFontIndirectExA(elfexd: *const ENUMLOGFONTEXDVA) -> HFONT {
    if !elfexd.is_null() {
        let mut Logfont: ENUMLOGFONTEXDVW = core::mem::zeroed();

        EnumLogFontExW2A(
            elfexd as LPENUMLOGFONTEXA,
            &Logfont.elfEnumLogfontEx,
        );

        core::ptr::copy_nonoverlapping(
            &(*elfexd).elfDesignVector as *const DESIGNVECTOR as *const u8,
            &mut Logfont.elfDesignVector as *mut DESIGNVECTOR as *mut u8,
            core::mem::size_of::<DESIGNVECTOR>(),
        );

        NtGdiHfontCreate(&Logfont, 0, 0, 0, core::ptr::null_mut())
    } else {
        core::ptr::null_mut()
    }
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateFontIndirectExW(elfexd: *const ENUMLOGFONTEXDVW) -> HFONT {
    /* Msdn: Note, this function ignores the elfDesignVector member in
            ENUMLOGFONTEXDV.
     */
    if !elfexd.is_null() {
        NtGdiHfontCreate(elfexd, 0, 0, 0, core::ptr::null_mut())
    } else {
        core::ptr::null_mut()
    }
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateFontIndirectA(lplf: *const LOGFONTA) -> HFONT {
    if !lplf.is_null() {
        let mut tlf: LOGFONTW = core::mem::zeroed();

        LogFontA2W(&mut tlf, lplf);
        CreateFontIndirectW(&tlf)
    } else {
        core::ptr::null_mut()
    }
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateFontIndirectW(lplf: *const LOGFONTW) -> HFONT {
    if !lplf.is_null() {
        let mut Logfont: ENUMLOGFONTEXDVW = core::mem::zeroed();

        core::ptr::copy_nonoverlapping(
            lplf as *const u8,
            &mut Logfont.elfEnumLogfontEx.elfLogFont as *mut LOGFONTW as *mut u8,
            core::mem::size_of::<LOGFONTW>(),
        );
        // Need something other than just cleaning memory here.
        // Guess? Use caller data to determine the rest.
        core::ptr::write_bytes(
            Logfont.elfEnumLogfontEx.elfFullName.as_mut_ptr(),
            0,
            Logfont.elfEnumLogfontEx.elfFullName.len(),
        );
        core::ptr::write_bytes(
            Logfont.elfEnumLogfontEx.elfStyle.as_mut_ptr(),
            0,
            Logfont.elfEnumLogfontEx.elfStyle.len(),
        );
        core::ptr::write_bytes(
            Logfont.elfEnumLogfontEx.elfScript.as_mut_ptr(),
            0,
            Logfont.elfEnumLogfontEx.elfScript.len(),
        );

        Logfont.elfDesignVector.dvNumAxes = 0; // No more than MM_MAX_NUMAXES

        core::ptr::write_bytes(
            &mut Logfont.elfDesignVector as *mut DESIGNVECTOR as *mut u8,
            0,
            core::mem::size_of::<DESIGNVECTOR>(),
        );

        CreateFontIndirectExW(&Logfont)
    } else {
        core::ptr::null_mut()
    }
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateFontA(
    nHeight: c_int,
    nWidth: c_int,
    nEscapement: c_int,
    nOrientation: c_int,
    fnWeight: c_int,
    fdwItalic: DWORD,
    fdwUnderline: DWORD,
    fdwStrikeOut: DWORD,
    fdwCharSet: DWORD,
    fdwOutputPrecision: DWORD,
    fdwClipPrecision: DWORD,
    fdwQuality: DWORD,
    fdwPitchAndFamily: DWORD,
    lpszFace: LPCSTR,
) -> HFONT {
    let mut StringA: STRING = core::mem::zeroed();
    let mut StringU: UNICODE_STRING = core::mem::zeroed();
    let ret: HFONT;

    RtlInitAnsiString(&mut StringA, lpszFace);
    RtlAnsiStringToUnicodeString(&mut StringU, &StringA, TRUE as BOOLEAN);

    ret = CreateFontW(
        nHeight,
        nWidth,
        nEscapement,
        nOrientation,
        fnWeight,
        fdwItalic,
        fdwUnderline,
        fdwStrikeOut,
        fdwCharSet,
        fdwOutputPrecision,
        fdwClipPrecision,
        fdwQuality,
        fdwPitchAndFamily,
        StringU.Buffer,
    );

    RtlFreeUnicodeString(&mut StringU);

    ret
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateFontW(
    nHeight: c_int,
    nWidth: c_int,
    nEscapement: c_int,
    nOrientation: c_int,
    nWeight: c_int,
    fnItalic: DWORD,
    fdwUnderline: DWORD,
    fdwStrikeOut: DWORD,
    fdwCharSet: DWORD,
    fdwOutputPrecision: DWORD,
    fdwClipPrecision: DWORD,
    fdwQuality: DWORD,
    fdwPitchAndFamily: DWORD,
    lpszFace: LPCWSTR,
) -> HFONT {
    let mut logfont: LOGFONTW = core::mem::zeroed();

    logfont.lfHeight = nHeight;
    logfont.lfWidth = nWidth;
    logfont.lfEscapement = nEscapement;
    logfont.lfOrientation = nOrientation;
    logfont.lfWeight = nWeight;
    logfont.lfItalic = fnItalic as BYTE;
    logfont.lfUnderline = fdwUnderline as BYTE;
    logfont.lfStrikeOut = fdwStrikeOut as BYTE;
    logfont.lfCharSet = fdwCharSet as BYTE;
    logfont.lfOutPrecision = fdwOutputPrecision as BYTE;
    logfont.lfClipPrecision = fdwClipPrecision as BYTE;
    logfont.lfQuality = fdwQuality as BYTE;
    logfont.lfPitchAndFamily = fdwPitchAndFamily as BYTE;

    if !lpszFace.is_null() {
        let Size: INT =
            (core::mem::size_of_val(&logfont.lfFaceName) / core::mem::size_of::<WCHAR>()) as INT;
        wcsncpy(
            logfont.lfFaceName.as_mut_ptr(),
            lpszFace,
            (Size - 1) as usize,
        );
        /* Be 101% sure to have '\0' at end of string */
        logfont.lfFaceName[(Size - 1) as usize] = b'\0' as WCHAR;
    } else {
        logfont.lfFaceName[0] = 0; // L'\0'
    }

    CreateFontIndirectW(&logfont)
}

extern "C" {
    fn wcsncpy(dest: *mut WCHAR, src: *const WCHAR, n: usize) -> *mut WCHAR;
}

// Convert single or multiple font path(s)
fastcall_fn! {
#[no_mangle]
pub unsafe fn IntConvertFontPaths(
    pszFiles: PCWSTR,
    pcFiles: PDWORD,
    pcwc: PDWORD,
    pfl: PDWORD,
    _bFlag: BOOL,
) -> PWSTR {
    // FIXME: pfl
    // FIXME: bFlag
    let _ = pfl;

    *pcwc = 0;
    *pcFiles = 0;

    if *pszFiles == 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    // Build "Fonts" path
    let mut szFontsDir: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    GetWindowsDirectoryW(szFontsDir.as_mut_ptr(), szFontsDir.len() as UINT);
    StringCchCatW(
        szFontsDir.as_mut_ptr(),
        szFontsDir.len() as SIZE_T,
        wsz(&FONTS_SUFFIX),
    );

    // Count the number of paths separated by '|'.
    let mut pathCount: ULONG = 1;
    {
        let mut pch1: PCWSTR = pszFiles;
        while *pch1 != 0 {
            if *pch1 == b'|' as WCHAR {
                pathCount += 1;
            }
            pch1 = pch1.offset(1);
        }
    }

    // Allocate memory for the paths.
    let cchBuff: SIZE_T = (pathCount as SIZE_T) * (MAX_PATH as SIZE_T);
    let pszBuff: PWSTR =
        HEAP_alloc((cchBuff as usize * core::mem::size_of::<WCHAR>()) as SIZE_T) as PWSTR;
    if pszBuff.is_null() {
        return core::ptr::null_mut();
    }

    *pszBuff.offset(0) = UNICODE_NULL;
    *pcFiles = pathCount;

    // Convert paths
    let mut dwError: DWORD = ERROR_SUCCESS;
    let mut pch1: PCWSTR;
    let mut pch1Prev: PCWSTR;
    let mut bFirst: BOOL = TRUE;
    pch1 = pszFiles;
    pch1Prev = pszFiles;
    loop {
        if *pch1 != 0 && *pch1 != b'|' as WCHAR {
            pch1 = pch1.offset(1);
            continue;
        }

        let spanLen: usize = pch1.offset_from(pch1Prev) as usize;
        if spanLen < DOTEXT_COUNT || spanLen >= MAX_PATH as usize {
            dwError = ERROR_INVALID_FUNCTION;
            break;
        }

        let mut szFileName: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let mut szFullPath: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
        StringCchCopyNW(
            szFileName.as_mut_ptr(),
            szFileName.len() as SIZE_T,
            pch1Prev,
            spanLen as SIZE_T,
        );

        // Search file
        if SearchPathW(
            wsz(&DOT_DIR),
            szFileName.as_ptr(),
            core::ptr::null(),
            szFullPath.len() as DWORD,
            szFullPath.as_mut_ptr(),
            core::ptr::null_mut(),
        ) == 0
            && SearchPathW(
                szFontsDir.as_ptr(),
                szFileName.as_ptr(),
                core::ptr::null(),
                szFullPath.len() as DWORD,
                szFullPath.as_mut_ptr(),
                core::ptr::null_mut(),
            ) == 0
        {
            dwError = ERROR_FILE_NOT_FOUND;
            break;
        }

        if bFirst != FALSE {
            bFirst = FALSE;
        } else {
            let cch: usize = wcslen(szFullPath.as_ptr());
            if cch < DOTPFB_COUNT {
                dwError = ERROR_INVALID_FUNCTION;
                break;
            }

            // Check filename extension
            let pchDotExt: PCWSTR = szFullPath.as_ptr().add(cch - 4);
            if _wcsicmp(pchDotExt, wsz(&DOT_PFB)) != 0 && _wcsicmp(pchDotExt, wsz(&DOT_MMM)) != 0 {
                dwError = ERROR_INVALID_FUNCTION;
                break;
            }
        }

        // Convert to an NT path
        let mut NtAbsPath: UNICODE_STRING = core::mem::zeroed();
        if RtlDosPathNameToNtPathName_U(
            szFullPath.as_ptr(),
            &mut NtAbsPath,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            dwError = ERROR_OUTOFMEMORY;
            break;
        }

        // Append a path and '|' to pszBuff
        if StringCchCatW(pszBuff, cchBuff, NtAbsPath.Buffer) != S_OK
            || StringCchCatW(pszBuff, cchBuff, wsz(&BAR)) != S_OK
        {
            RtlFreeUnicodeString(&mut NtAbsPath);
            dwError = ERROR_INVALID_FUNCTION;
            break;
        }

        RtlFreeUnicodeString(&mut NtAbsPath);

        if *pch1 == 0 {
            break;
        }

        pch1Prev = pch1.offset(1);
        pch1 = pch1.offset(1);
    }

    if dwError != ERROR_SUCCESS {
        HEAP_free(pszBuff as PVOID);
        *pcwc = 0;
        *pcFiles = 0;
        if dwError != ERROR_INVALID_FUNCTION {
            SetLastError(dwError);
        }
        return core::ptr::null_mut();
    }

    *pcwc = wcslen(pszBuff) as DWORD;

    // Convert '|' to '\0'
    {
        let mut pch2: PWSTR = pszBuff;
        while *pch2 != 0 {
            if *pch2 == b'|' as WCHAR {
                *pch2 = UNICODE_NULL;
            }
            pch2 = pch2.offset(1);
        }
    }

    pszBuff
}
}

// Wide string literals used by IntConvertFontPaths (UTF-16, NUL-terminated).
static FONTS_SUFFIX: [WCHAR; 7] = [b'\\' as WCHAR, b'F' as WCHAR, b'o' as WCHAR, b'n' as WCHAR, b't' as WCHAR, b's' as WCHAR, 0];
static DOT_DIR: [WCHAR; 2] = [b'.' as WCHAR, 0];
static DOT_PFB: [WCHAR; 5] = [b'.' as WCHAR, b'p' as WCHAR, b'f' as WCHAR, b'b' as WCHAR, 0];
static DOT_MMM: [WCHAR; 5] = [b'.' as WCHAR, b'm' as WCHAR, b'm' as WCHAR, b'm' as WCHAR, 0];
static BAR: [WCHAR; 2] = [b'|' as WCHAR, 0];
// _countof(L".ttf") = 5, _countof(L".pfb") = 5
const DOTEXT_COUNT: usize = 5;
const DOTPFB_COUNT: usize = 5;

#[inline]
fn wsz(s: &'static [WCHAR]) -> *const WCHAR {
    s.as_ptr()
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateScalableFontResourceA(
    _fdwHidden: DWORD,
    _lpszFontRes: LPCSTR,
    _lpszFontFile: LPCSTR,
    _lpszCurrentPath: LPCSTR,
) -> BOOL {
    FALSE
}

/* @implemented */
#[no_mangle]
pub unsafe extern "system" fn AddFontResourceExW(
    lpszFilename: LPCWSTR,
    fl: DWORD,
    _pvReserved: PVOID,
) -> INT {
    if fl & !(FR_PRIVATE | FR_NOT_ENUM) != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    GdiAddFontResourceW(lpszFilename, fl, core::ptr::null_mut())
}

/* @implemented */
#[no_mangle]
pub unsafe extern "system" fn AddFontResourceExA(
    lpszFilename: LPCSTR,
    fl: DWORD,
    _pvReserved: PVOID,
) -> INT {
    if fl & !(FR_PRIVATE | FR_NOT_ENUM) != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if lpszFilename.is_null() {
        return 0;
    }

    let FilenameW: PWSTR;
    let mut szBuff: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];

    // _SEH2_TRY: under USE_DUMMY_PSEH the except body is dead code.
    FilenameW = HEAP_strdupA2W_buf(lpszFilename, szBuff.as_mut_ptr(), szBuff.len() as SIZE_T);

    if FilenameW.is_null() {
        SetLastError(ERROR_OUTOFMEMORY);
        return 0;
    }

    let ret: INT = GdiAddFontResourceW(FilenameW, fl, core::ptr::null_mut());
    HEAP_strdupA2W_buf_free(FilenameW, szBuff.as_mut_ptr());
    ret
}

/* @implemented */
#[no_mangle]
pub unsafe extern "system" fn AddFontResourceA(lpszFilename: LPCSTR) -> INT {
    AddFontResourceExA(lpszFilename, 0, core::ptr::null_mut())
}

/* @implemented */
#[no_mangle]
pub unsafe extern "system" fn AddFontResourceW(lpszFilename: LPCWSTR) -> INT {
    GdiAddFontResourceW(lpszFilename, 0, core::ptr::null_mut())
}

/* @implemented */
#[no_mangle]
pub unsafe extern "system" fn RemoveFontResourceW(lpFileName: LPCWSTR) -> BOOL {
    RemoveFontResourceExW(lpFileName, 0, core::ptr::null_mut())
}

/* @implemented */
#[no_mangle]
pub unsafe extern "system" fn RemoveFontResourceA(lpFileName: LPCSTR) -> BOOL {
    RemoveFontResourceExA(lpFileName, 0, core::ptr::null_mut())
}

/* @implemented */
#[no_mangle]
pub unsafe extern "system" fn RemoveFontResourceExA(
    lpFileName: LPCSTR,
    fl: DWORD,
    pdv: PVOID,
) -> BOOL {
    if fl & !(FR_PRIVATE | FR_NOT_ENUM) != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    if lpFileName.is_null() {
        return FALSE;
    }

    let mut szBuff: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let FilenameW: PWSTR;

    // _SEH2_TRY: under USE_DUMMY_PSEH the except body is dead code.
    FilenameW = HEAP_strdupA2W_buf(lpFileName, szBuff.as_mut_ptr(), szBuff.len() as SIZE_T);

    if FilenameW.is_null() {
        SetLastError(ERROR_OUTOFMEMORY);
        return FALSE;
    }

    let result: BOOL = RemoveFontResourceExW(FilenameW, fl, pdv);
    HEAP_strdupA2W_buf_free(FilenameW, szBuff.as_mut_ptr());
    result
}

/* @implemented */
#[no_mangle]
pub unsafe extern "system" fn RemoveFontResourceExW(
    lpFileName: LPCWSTR,
    mut fl: DWORD,
    _pdv: PVOID,
) -> BOOL {
    // DPRINT("RemoveFontResourceExW\n");

    if fl & !(FR_PRIVATE | FR_NOT_ENUM) != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    if lpFileName.is_null() {
        return FALSE;
    }

    let mut cFiles: ULONG = 0;
    let mut cwc: ULONG = 0;
    let pszConverted: PWSTR = IntConvertFontPaths(lpFileName, &mut cFiles, &mut cwc, &mut fl, TRUE);
    if pszConverted.is_null() {
        return FALSE;
    }

    let ret: BOOL = NtGdiRemoveFontResourceW(
        pszConverted,
        cwc,
        cFiles,
        fl,
        0,
        core::ptr::null(),
    );
    HEAP_free(pszConverted as PVOID);
    ret
}

/***********************************************************************
 *           GdiGetCharDimensions
 *
 * Gets the average width of the characters in the English alphabet.
 * See font.c for the full documentation block.
 */
#[no_mangle]
pub unsafe extern "system" fn GdiGetCharDimensions(
    hdc: HDC,
    lptm: LPTEXTMETRICW,
    height: *mut LONG,
) -> LONG {
    let mut sz: SIZE = core::mem::zeroed();
    let mut tm: TEXTMETRICW = core::mem::zeroed();
    static ALPHABET: [WCHAR; 53] = [
        b'a' as WCHAR, b'b' as WCHAR, b'c' as WCHAR, b'd' as WCHAR, b'e' as WCHAR, b'f' as WCHAR,
        b'g' as WCHAR, b'h' as WCHAR, b'i' as WCHAR, b'j' as WCHAR, b'k' as WCHAR, b'l' as WCHAR,
        b'm' as WCHAR, b'n' as WCHAR, b'o' as WCHAR, b'p' as WCHAR, b'q' as WCHAR, b'r' as WCHAR,
        b's' as WCHAR, b't' as WCHAR, b'u' as WCHAR, b'v' as WCHAR, b'w' as WCHAR, b'x' as WCHAR,
        b'y' as WCHAR, b'z' as WCHAR, b'A' as WCHAR, b'B' as WCHAR, b'C' as WCHAR, b'D' as WCHAR,
        b'E' as WCHAR, b'F' as WCHAR, b'G' as WCHAR, b'H' as WCHAR, b'I' as WCHAR, b'J' as WCHAR,
        b'K' as WCHAR, b'L' as WCHAR, b'M' as WCHAR, b'N' as WCHAR, b'O' as WCHAR, b'P' as WCHAR,
        b'Q' as WCHAR, b'R' as WCHAR, b'S' as WCHAR, b'T' as WCHAR, b'U' as WCHAR, b'V' as WCHAR,
        b'W' as WCHAR, b'X' as WCHAR, b'Y' as WCHAR, b'Z' as WCHAR, 0,
    ];

    if GetTextMetricsW(hdc, &mut tm) == FALSE {
        return 0;
    }

    if GetTextExtentPointW(hdc, ALPHABET.as_ptr(), 52, &mut sz) == FALSE {
        return 0;
    }

    if !lptm.is_null() {
        *lptm = tm;
    }
    if !height.is_null() {
        *height = tm.tmHeight;
    }

    (sz.cx / 26 + 1) / 2
}

/*************************************************************************
 * TranslateCharsetInfo [GDI32.@]
 *
 * Fills a CHARSETINFO structure for a character set, code page, or font.
 * See font.c for the full documentation block.
 */
#[no_mangle]
pub unsafe extern "system" fn TranslateCharsetInfo(
    lpSrc: LPDWORD,
    lpCs: LPCHARSETINFO,
    flags: DWORD,
) -> BOOL {
    let mut index: c_int = 0;
    match flags {
        TCI_SRCFONTSIG => {
            while index < MAXTCIINDEX as c_int && (*lpSrc >> index & 0x0001) == 0 {
                index += 1;
            }
        }
        TCI_SRCCODEPAGE => {
            while index < MAXTCIINDEX as c_int
                && (lpSrc as usize as ULONG) != FONT_tci[index as usize].ciACP
            {
                index += 1;
            }
        }
        TCI_SRCCHARSET => {
            while index < MAXTCIINDEX as c_int
                && (lpSrc as usize as ULONG) != FONT_tci[index as usize].ciCharset
            {
                index += 1;
            }
        }
        TCI_SRCLOCALE => {
            let lCid: LCID = lpSrc as usize as LCID;
            let mut LocSig: LOCALESIGNATURE = core::mem::zeroed();
            let Ret: INT = GetLocaleInfoW(
                lCid,
                LOCALE_FONTSIGNATURE,
                &mut LocSig as *mut LOCALESIGNATURE as LPWSTR,
                0,
            );
            if GetLocaleInfoW(
                lCid,
                LOCALE_FONTSIGNATURE,
                &mut LocSig as *mut LOCALESIGNATURE as LPWSTR,
                Ret,
            ) != 0
            {
                while index < MAXTCIINDEX as c_int
                    && (LocSig.lsCsbDefault[0] >> index & 0x0001) == 0
                {
                    index += 1;
                }
                // break out of match (the C `break` ends the switch case)
            } else {
                // fall through to default
                GdiSetLastError(ERROR_INVALID_PARAMETER);
                return FALSE;
            }
        }
        _ => {
            GdiSetLastError(ERROR_INVALID_PARAMETER);
            return FALSE;
        }
    }
    if index >= MAXTCIINDEX as c_int
        || FONT_tci[index as usize].ciCharset == DEFAULT_CHARSET as UINT
    {
        return FALSE;
    }
    // DPRINT(...)
    core::ptr::copy_nonoverlapping(
        &FONT_tci[index as usize] as *const CHARSETINFO as *const u8,
        lpCs as *mut u8,
        core::mem::size_of::<CHARSETINFO>(),
    );
    TRUE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetMapperFlags(hDC: HDC, flags: DWORD) -> DWORD {
    let mut Ret: DWORD = GDI_ERROR;
    let Dc_Attr: PDC_ATTR;

    /* Get the DC attribute */
    Dc_Attr = GdiGetDcAttr(hDC);
    if Dc_Attr.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return GDI_ERROR;
    }

    if (*NtCurrentTeb()).GdiTebBatch.HDC == hDC as HANDLE {
        if (*dcattr_f!(Dc_Attr, ulDirty_)) & DC_FONTTEXT_DIRTY != 0 {
            NtGdiFlush();
            (*dcattr_f!(Dc_Attr, ulDirty_)) &= !(DC_MODE_DIRTY | DC_FONTTEXT_DIRTY);
        }
    }

    if flags & !1 != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
    } else {
        Ret = (*dcattr_f!(Dc_Attr, flFontMapper));
        (*dcattr_f!(Dc_Attr, flFontMapper)) = flags;
    }
    Ret
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn EnumFontsW(
    hDC: HDC,
    lpFaceName: LPCWSTR,
    FontFunc: PVOID, // FONTENUMPROCW
    lParam: LPARAM,
) -> INT {
    // #if 0 path uses NtGdiEnumFonts; the live path forwards to EnumFontFamiliesW.
    EnumFontFamiliesW(hDC, lpFaceName, FontFunc, lParam)
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn EnumFontsA(
    hDC: HDC,
    lpFaceName: LPCSTR,
    FontFunc: PVOID, // FONTENUMPROCA
    lParam: LPARAM,
) -> INT {
    // #if 0 path uses NtGdiEnumFonts; the live path forwards to EnumFontFamiliesA.
    EnumFontFamiliesA(hDC, lpFaceName, FontFunc, lParam)
}

const EfdFontFamilies: ULONG = 3;

#[no_mangle]
pub unsafe extern "system" fn NewEnumFontFamiliesExW(
    hDC: HDC,
    lpLogfont: LPLOGFONTW,
    lpEnumFontFamExProcW: PVOID, // FONTENUMPROCW
    lParam: LPARAM,
    _dwFlags: DWORD,
) -> INT {
    let idEnum: ULONG_PTR;
    let mut cbDataSize: ULONG = 0;
    let mut cbRetSize: ULONG = 0;
    let mut pEfdw: PENUMFONTDATAW;
    let pBuffer: PBYTE;
    let pMax: PBYTE;
    let mut ret: INT = 1;

    /* Open enumeration handle and find out how much memory we need */
    idEnum = NtGdiEnumFontOpen(
        hDC,
        EfdFontFamilies,
        0,
        LF_FACESIZE,
        if !lpLogfont.is_null() && (*lpLogfont).lfFaceName[0] != 0 {
            (*lpLogfont).lfFaceName.as_mut_ptr()
        } else {
            core::ptr::null_mut()
        },
        if !lpLogfont.is_null() {
            (*lpLogfont).lfCharSet as ULONG
        } else {
            DEFAULT_CHARSET as ULONG
        },
        &mut cbDataSize,
    );
    if idEnum == 0 {
        return 0;
    }
    if cbDataSize == 0 {
        NtGdiEnumFontClose(idEnum);
        return 0;
    }

    /* Allocate memory */
    pBuffer = HeapAlloc(GetProcessHeap(), 0, cbDataSize as SIZE_T) as PBYTE;
    if pBuffer.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        NtGdiEnumFontClose(idEnum);
        return 0;
    }

    /* Do the enumeration */
    if NtGdiEnumFontChunk(hDC, idEnum, cbDataSize, &mut cbRetSize, pBuffer as PENUMFONTDATAW)
        == FALSE
    {
        HeapFree(GetProcessHeap(), 0, pBuffer as LPVOID);
        NtGdiEnumFontClose(idEnum);
        return 0;
    }
    let _ = cbRetSize;

    /* Get start and end address */
    pEfdw = pBuffer as PENUMFONTDATAW;
    pMax = pBuffer.offset(cbDataSize as isize);

    /* Iterate through the structures */
    while (pEfdw as PBYTE) < pMax && ret != 0 {
        let pNtmwi: PNTMW_INTERNAL =
            ((pEfdw as ULONG_PTR) + (*pEfdw).ulNtmwiOffset as ULONG_PTR) as PNTMW_INTERNAL;

        let proc_: FONTENUMPROCW_T = core::mem::transmute(lpEnumFontFamExProcW);
        ret = proc_(
            &(*pEfdw).elfexdv.elfEnumLogfontEx,
            &(*pNtmwi).ntmw,
            (*pEfdw).dwFontType,
            lParam,
        );

        pEfdw = ((pEfdw as ULONG_PTR) + (*pEfdw).cbSize as ULONG_PTR) as PENUMFONTDATAW;
    }

    /* Release the memory and close handle */
    HeapFree(GetProcessHeap(), 0, pBuffer as LPVOID);
    NtGdiEnumFontClose(idEnum);

    ret
}

/* @implemented */
#[no_mangle]
pub unsafe extern "system" fn GdiAddFontResourceW(
    lpszFilename: LPCWSTR,
    mut fl: FLONG,
    pdv: *mut DESIGNVECTOR,
) -> INT {
    let mut cFiles: ULONG = 0;
    let mut cwc: ULONG = 0;
    let mut pszConverted: PWSTR =
        IntConvertFontPaths(lpszFilename, &mut cFiles, &mut cwc, &mut fl, FALSE);
    if pszConverted.is_null() {
        return 0;
    }

    let mut ret: INT = NtGdiAddFontResourceW(pszConverted, cwc, cFiles, fl, 0, pdv);
    HEAP_free(pszConverted as PVOID);
    if ret != 0 {
        return ret;
    }

    pszConverted = IntConvertFontPaths(lpszFilename, &mut cFiles, &mut cwc, &mut fl, TRUE);
    if pszConverted.is_null() {
        return 0;
    }

    ret = NtGdiAddFontResourceW(pszConverted, cwc, cFiles, fl, 0, pdv);
    HEAP_free(pszConverted as PVOID);
    if ret == 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
    }
    ret
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn AddFontMemResourceEx(
    pbFont: PVOID,
    cbFont: DWORD,
    _pdv: PVOID,
    pcFonts: *mut DWORD,
) -> HANDLE {
    if !pbFont.is_null() && cbFont != 0 && !pcFonts.is_null() {
        return NtGdiAddFontMemResourceEx(pbFont, cbFont, core::ptr::null(), 0, pcFonts as PDWORD);
    }
    SetLastError(ERROR_INVALID_PARAMETER);
    core::ptr::null_mut()
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn RemoveFontMemResourceEx(fh: HANDLE) -> BOOL {
    if !fh.is_null() {
        return NtGdiRemoveFontMemResourceEx(fh);
    }
    SetLastError(ERROR_INVALID_PARAMETER);
    FALSE
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn AddFontResourceTracking(_lpString: LPCSTR, _unknown: INT) -> INT {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn RemoveFontResourceTracking(_lpString: LPCSTR, _unknown: INT) -> INT {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

#[no_mangle]
pub unsafe extern "system" fn CreateScalableFontResourceW(
    _fdwHidden: DWORD,
    lpszFontRes: LPCWSTR,
    _lpszFontFile: LPCWSTR,
    _lpszCurrentPath: LPCWSTR,
) -> BOOL {
    let f: HANDLE;

    // UNIMPLEMENTED;

    /* fHidden=1 - only visible for the calling app, read-only, not
     * enumerated with EnumFonts/EnumFontFamilies
     * lpszCurrentPath can be NULL
     */

    /* If the output file already exists, return the ERROR_FILE_EXISTS error as specified in MSDN */
    f = CreateFileW(
        lpszFontRes,
        0,
        0,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut(),
    );
    if f != invalid_handle_value() {
        CloseHandle(f);
        SetLastError(ERROR_FILE_EXISTS);
        return FALSE;
    }
    FALSE /* create failed */
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn bInitSystemAndFontsDirectoriesW(
    _SystemDir: *mut LPWSTR,
    _FontsDir: *mut LPWSTR,
) -> BOOL {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn EudcLoadLinkW(
    _pBaseFaceName: LPCWSTR,
    _pEudcFontPath: LPCWSTR,
    _iPriority: INT,
    _iFontLinkType: INT,
) -> BOOL {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn EudcUnloadLinkW(
    _pBaseFaceName: LPCWSTR,
    _pEudcFontPath: LPCWSTR,
) -> BOOL {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetEUDCTimeStamp() -> ULONG {
    NtGdiGetEudcTimeStampEx(core::ptr::null_mut(), 0, TRUE)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetEUDCTimeStampExW(lpBaseFaceName: LPWSTR) -> DWORD {
    let retValue: DWORD;

    if lpBaseFaceName.is_null() {
        retValue = NtGdiGetEudcTimeStampEx(core::ptr::null_mut(), 0, FALSE);
    } else {
        retValue =
            NtGdiGetEudcTimeStampEx(lpBaseFaceName, wcslen(lpBaseFaceName) as ULONG, FALSE);
    }

    retValue
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetFontAssocStatus(hdc: HDC) -> ULONG {
    let mut retValue: ULONG = 0;

    if !hdc.is_null() {
        retValue = NtGdiQueryFontAssocInfo(hdc);
    }

    retValue
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn QueryFontAssocStatus() -> DWORD {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn UnloadNetworkFonts(_unknown: DWORD) {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetFontData(
    hdc: HDC,
    dwTable: DWORD,
    dwOffset: DWORD,
    lpvBuffer: LPVOID,
    mut cbData: DWORD,
) -> DWORD {
    if lpvBuffer.is_null() {
        cbData = 0;
    }
    NtGdiGetFontData(hdc, dwTable, dwOffset, lpvBuffer, cbData)
}

#[no_mangle]
pub unsafe extern "system" fn cGetTTFFromFOT(
    _x1: DWORD,
    _x2: DWORD,
    _x3: DWORD,
    _x4: DWORD,
    _x5: DWORD,
    _x6: DWORD,
    _x7: DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

// ── strsafe.h inline reimplementations (bounded wide-string helpers) ────────────
// NEEDS-CHECK: faithful enough for the call sites used here. StringCb* counts BYTES;
// StringCch* counts WCHARs. All NUL-terminate the destination and clamp to the buffer.

/// `StringCchCopyNW(dest, cchDest, src, cchToCopy)` — copy up to `cchToCopy` source
/// chars into `dest` (capacity `cchDest` incl. NUL), then NUL-terminate.
unsafe fn StringCchCopyNW(
    pszDest: *mut WCHAR,
    cchDest: SIZE_T,
    pszSrc: *const WCHAR,
    cchToCopy: SIZE_T,
) -> HRESULT {
    if cchDest == 0 {
        return STRSAFE_E_INSUFFICIENT_BUFFER;
    }
    let mut hr: HRESULT = S_OK;
    let mut i: SIZE_T = 0;
    while i < cchToCopy && i < cchDest - 1 {
        let c = *pszSrc.offset(i as isize);
        if c == 0 {
            break;
        }
        *pszDest.offset(i as isize) = c;
        i += 1;
    }
    if i < cchToCopy && i == cchDest - 1 && *pszSrc.offset(i as isize) != 0 {
        hr = STRSAFE_E_INSUFFICIENT_BUFFER;
    }
    *pszDest.offset(i as isize) = 0;
    hr
}

/// `StringCchCatW(dest, cchDest, src)` — append `src` to `dest` (capacity `cchDest`
/// incl. NUL). Returns S_OK or STRSAFE_E_INSUFFICIENT_BUFFER.
unsafe fn StringCchCatW(pszDest: *mut WCHAR, cchDest: SIZE_T, pszSrc: *const WCHAR) -> HRESULT {
    let mut dlen: SIZE_T = 0;
    while dlen < cchDest && *pszDest.offset(dlen as isize) != 0 {
        dlen += 1;
    }
    if dlen >= cchDest {
        return STRSAFE_E_INSUFFICIENT_BUFFER;
    }
    let mut i: SIZE_T = 0;
    loop {
        let c = *pszSrc.offset(i as isize);
        if c == 0 {
            break;
        }
        if dlen + i >= cchDest - 1 {
            *pszDest.offset((dlen + i) as isize) = 0;
            return STRSAFE_E_INSUFFICIENT_BUFFER;
        }
        *pszDest.offset((dlen + i) as isize) = c;
        i += 1;
    }
    *pszDest.offset((dlen + i) as isize) = 0;
    S_OK
}

/// `StringCbCopyW(dest, cbDest, src)` — `cb` is BYTES; copy `src` into `dest`,
/// clamped to `cbDest/sizeof(WCHAR)` and NUL-terminated.
unsafe fn StringCbCopyW(pszDest: *mut WCHAR, cbDest: SIZE_T, pszSrc: *const WCHAR) -> HRESULT {
    let cchDest: SIZE_T = cbDest / core::mem::size_of::<WCHAR>() as SIZE_T;
    if cchDest == 0 {
        return STRSAFE_E_INSUFFICIENT_BUFFER;
    }
    let mut hr: HRESULT = S_OK;
    let mut i: SIZE_T = 0;
    loop {
        let c = *pszSrc.offset(i as isize);
        if c == 0 {
            break;
        }
        if i >= cchDest - 1 {
            hr = STRSAFE_E_INSUFFICIENT_BUFFER;
            break;
        }
        *pszDest.offset(i as isize) = c;
        i += 1;
    }
    *pszDest.offset(i as isize) = 0;
    hr
}
