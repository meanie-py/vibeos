use ntdll_sys::nt::*;
use core::ffi::{c_void, c_char};

extern "C" {  // CRT (cdecl) — were wrongly declared stdcall (_name@N undefined)
    fn strlen(s: *const i8) -> usize;
    fn wcstol(s: *const u16, endptr: *mut *mut u16, base: i32) -> i32;
    fn memmove(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
}

/// `_wcsicmp` (the C source calls the old `wcsicmp` name, bridged by oldnames.a
/// to msvcrt's `_wcsicmp`). Self-contained Rust impl so mlang needs no cmake
/// oldnames.a / CRT alias: ASCII case-insensitive wide compare. mlang's sole
/// caller passes RFC1766 language tags (ASCII), for which ASCII folding is
/// bit-identical to `_wcsicmp`'s locale folding.
unsafe fn wcsicmp(s1: *const u16, s2: *const u16) -> i32 {
    let (mut a, mut b) = (s1, s2);
    loop {
        let fold = |c: u16| -> u16 { if (b'A' as u16..=b'Z' as u16).contains(&c) { c + 0x20 } else { c } };
        let (c1, c2) = (fold(*a), fold(*b));
        if c1 != c2 {
            return c1 as i32 - c2 as i32;
        }
        if c1 == 0 {
            return 0;
        }
        a = a.add(1);
        b = b.add(1);
    }
}

// ---- MIMECONTF constants (from mlang.h tagMIMECONTF enum) ----
pub type MIMECONTF = u32;
const MIMECONTF_MAILNEWS: MIMECONTF = 0x00000001;
const MIMECONTF_BROWSER: MIMECONTF = 0x00000002;
const MIMECONTF_MINIMAL: MIMECONTF = 0x00000004;
const MIMECONTF_IMPORT: MIMECONTF = 0x00000008;
const MIMECONTF_SAVABLE_MAILNEWS: MIMECONTF = 0x00000100;
const MIMECONTF_SAVABLE_BROWSER: MIMECONTF = 0x00000200;
const MIMECONTF_EXPORT: MIMECONTF = 0x00000400;
const MIMECONTF_PRIVCONVERTER: MIMECONTF = 0x00010000;
const MIMECONTF_VALID: MIMECONTF = 0x00020000;
const MIMECONTF_VALID_NLS: MIMECONTF = 0x00040000;
const MIMECONTF_MIME_IE4: MIMECONTF = 0x10000000;
const MIMECONTF_MIME_LATEST: MIMECONTF = 0x20000000;
const MIMECONTF_MIME_REGISTRY: MIMECONTF = 0x40000000;

// ---- basic Win32 type aliases (needed by COM + mlang) ----
pub type HRESULT = i32;
pub type ULONG = u32;
pub type DWORD = u32;
pub type BOOL = i32;
pub type UINT = u32;
pub type LONG = i32;
pub type WORD = u16;
pub type WCHAR = u16;
pub type LPWSTR = *mut u16;
pub type LPCWSTR = *const u16;
pub type LPSTR = *mut i8;
pub type LPCSTR = *const i8;
pub type LPVOID = *mut c_void;
pub type HANDLE = *mut c_void;
pub type HINSTANCE = *mut c_void;
pub type HKEY = HANDLE;
pub type HFONT = *mut c_void;
pub type HGDIOBJ = *mut c_void;
pub type HDC = *mut c_void;
pub type LSTATUS = LONG;
pub type LANGID = u16;
pub type LCID = DWORD;
pub type SCRIPT_ID = [u8; 4]; // 4-byte script identifier

// ---- GUID / CLSID / IID ----
#[repr(C)]
#[derive(Copy, Clone)]
pub struct GUID {
    pub Data1: u32,
    pub Data2: u16,
    pub Data3: u16,
    pub Data4: [u8; 8],
}
pub type CLSID = GUID;
pub type IID = GUID;

// ---- COM interface forward declarations (we'll need these before vtables) ----
// (These struct types are defined below, but some are referenced by fn pointers)

// ---- COM vtable types (fn pointer structs) ----
// Standard IUnknown-like QueryInterface/AddRef/Release with interface-specific first arg

#[repr(C)]
pub struct IClassFactoryVtbl {
    pub QueryInterface: unsafe extern "C" fn(*mut IClassFactory, *const GUID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "C" fn(*mut IClassFactory) -> u32,
    pub Release: unsafe extern "C" fn(*mut IClassFactory) -> u32,
    pub CreateInstance: unsafe extern "C" fn(*mut IClassFactory, *mut IUnknown, *const GUID, *mut *mut c_void) -> HRESULT,
    pub LockServer: unsafe extern "C" fn(*mut IClassFactory, BOOL) -> HRESULT,
}

#[repr(C)]
pub struct IEnumCodePageVtbl {
    pub QueryInterface: unsafe extern "C" fn(*mut IEnumCodePage, *const GUID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "C" fn(*mut IEnumCodePage) -> u32,
    pub Release: unsafe extern "C" fn(*mut IEnumCodePage) -> u32,
    pub Clone: unsafe extern "C" fn(*mut IEnumCodePage, *mut *mut IEnumCodePage) -> HRESULT,
    pub Next: unsafe extern "C" fn(*mut IEnumCodePage, u32, *mut MIMECPINFO, *mut u32) -> HRESULT,
    pub Reset: unsafe extern "C" fn(*mut IEnumCodePage) -> HRESULT,
    pub Skip: unsafe extern "C" fn(*mut IEnumCodePage, u32) -> HRESULT,
}

#[repr(C)]
pub struct IEnumScriptVtbl {
    pub QueryInterface: unsafe extern "C" fn(*mut IEnumScript, *const GUID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "C" fn(*mut IEnumScript) -> u32,
    pub Release: unsafe extern "C" fn(*mut IEnumScript) -> u32,
    pub Clone: unsafe extern "C" fn(*mut IEnumScript, *mut *mut IEnumScript) -> HRESULT,
    pub Next: unsafe extern "C" fn(*mut IEnumScript, u32, *mut SCRIPTINFO, *mut u32) -> HRESULT,
    pub Reset: unsafe extern "C" fn(*mut IEnumScript) -> HRESULT,
    pub Skip: unsafe extern "C" fn(*mut IEnumScript, u32) -> HRESULT,
}

#[repr(C)]
pub struct IEnumRfc1766Vtbl {
    pub QueryInterface: unsafe extern "C" fn(*mut IEnumRfc1766, *const GUID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "C" fn(*mut IEnumRfc1766) -> u32,
    pub Release: unsafe extern "C" fn(*mut IEnumRfc1766) -> u32,
    pub Clone: unsafe extern "C" fn(*mut IEnumRfc1766, *mut *mut IEnumRfc1766) -> HRESULT,
    pub Next: unsafe extern "C" fn(*mut IEnumRfc1766, u32, *mut RFC1766INFO, *mut u32) -> HRESULT,
    pub Reset: unsafe extern "C" fn(*mut IEnumRfc1766) -> HRESULT,
    pub Skip: unsafe extern "C" fn(*mut IEnumRfc1766, u32) -> HRESULT,
}

#[repr(C)]
pub struct IMLangFontLinkVtbl {
    pub QueryInterface: unsafe extern "C" fn(*mut IMLangFontLink, *const GUID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "C" fn(*mut IMLangFontLink) -> u32,
    pub Release: unsafe extern "C" fn(*mut IMLangFontLink) -> u32,
    pub GetCharCodePages: unsafe extern "C" fn(*mut IMLangFontLink, u16, *mut u32) -> HRESULT,
    pub GetStrCodePages: unsafe extern "C" fn(*mut IMLangFontLink, *const u16, i32, u32, *mut u32, *mut i32) -> HRESULT,
    pub CodePageToCodePages: unsafe extern "C" fn(*mut IMLangFontLink, u32, *mut u32) -> HRESULT,
    pub CodePagesToCodePage: unsafe extern "C" fn(*mut IMLangFontLink, u32, u32, *mut u32) -> HRESULT,
    pub GetFontCodePages: unsafe extern "C" fn(*mut IMLangFontLink, HDC, HFONT, *mut u32) -> HRESULT,
    pub MapFont: unsafe extern "C" fn(*mut IMLangFontLink, HDC, u32, HFONT, *mut HFONT) -> HRESULT,
    pub ReleaseFont: unsafe extern "C" fn(*mut IMLangFontLink, HFONT) -> HRESULT,
    pub ResetFontMapping: unsafe extern "C" fn(*mut IMLangFontLink) -> HRESULT,
}

#[repr(C)]
pub struct IMLangFontLink2Vtbl {
    pub QueryInterface: unsafe extern "C" fn(*mut IMLangFontLink2, *const GUID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "C" fn(*mut IMLangFontLink2) -> u32,
    pub Release: unsafe extern "C" fn(*mut IMLangFontLink2) -> u32,
    pub GetCharCodePages: unsafe extern "C" fn(*mut IMLangFontLink2, u16, *mut u32) -> HRESULT,
    pub GetStrCodePages: unsafe extern "C" fn(*mut IMLangFontLink2, *const u16, i32, u32, *mut u32, *mut i32) -> HRESULT,
    pub CodePageToCodePages: unsafe extern "C" fn(*mut IMLangFontLink2, u32, *mut u32) -> HRESULT,
    pub CodePagesToCodePage: unsafe extern "C" fn(*mut IMLangFontLink2, u32, u32, *mut u32) -> HRESULT,
    pub GetFontCodePages: unsafe extern "C" fn(*mut IMLangFontLink2, HDC, HFONT, *mut u32) -> HRESULT,
    pub ReleaseFont: unsafe extern "C" fn(*mut IMLangFontLink2, HFONT) -> HRESULT,
    pub ResetFontMapping: unsafe extern "C" fn(*mut IMLangFontLink2) -> HRESULT,
    pub MapFont: unsafe extern "C" fn(*mut IMLangFontLink2, HDC, u32, u16, *mut HFONT) -> HRESULT,
    pub GetFontUnicodeRanges: unsafe extern "C" fn(*mut IMLangFontLink2, HDC, *mut u32, *mut UNICODERANGE) -> HRESULT,
    pub GetScriptFontInfo: unsafe extern "C" fn(*mut IMLangFontLink2, SCRIPT_ID, u32, *mut u32, *mut SCRIPTFONTINFO) -> HRESULT,
    pub CodePageToScriptID: unsafe extern "C" fn(*mut IMLangFontLink2, u32, *mut u32) -> HRESULT,
}

#[repr(C)]
pub struct IMultiLanguageVtbl {
    pub QueryInterface: unsafe extern "C" fn(*mut IMultiLanguage, *const GUID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "C" fn(*mut IMultiLanguage) -> u32,
    pub Release: unsafe extern "C" fn(*mut IMultiLanguage) -> u32,
    pub GetNumberOfCodePageInfo: unsafe extern "C" fn(*mut IMultiLanguage, *mut u32) -> HRESULT,
    pub GetCodePageInfo: unsafe extern "C" fn(*mut IMultiLanguage, u32, *mut MIMECPINFO) -> i32,
    pub GetFamilyCodePage: unsafe extern "C" fn(*mut IMultiLanguage, u32, *mut u32) -> HRESULT,
    pub EnumCodePages: unsafe extern "C" fn(*mut IMultiLanguage, DWORD, *mut *mut IEnumCodePage) -> HRESULT,
    pub GetCharsetInfo: unsafe extern "C" fn(*mut IMultiLanguage, *mut u16, *mut MIMECSETINFO) -> i32,
    pub IsConvertible: unsafe extern "C" fn(*mut IMultiLanguage, u32, u32) -> HRESULT,
    pub ConvertString: unsafe extern "C" fn(*mut IMultiLanguage, *mut u32, u32, u32, *mut u8, *mut u32, *mut u8, *mut u32) -> HRESULT,
    pub ConvertStringToUnicode: unsafe extern "C" fn(*mut IMultiLanguage, *mut u32, u32, *mut i8, *mut u32, *mut u16, *mut u32) -> HRESULT,
    pub ConvertStringFromUnicode: unsafe extern "C" fn(*mut IMultiLanguage, *mut u32, u32, *mut u16, *mut u32, *mut i8, *mut u32) -> HRESULT,
    pub ConvertStringReset: unsafe extern "C" fn(*mut IMultiLanguage) -> HRESULT,
    pub GetRfc1766FromLcid: unsafe extern "C" fn(*mut IMultiLanguage, LCID, *mut BSTR) -> HRESULT,
    pub GetLcidFromRfc1766: unsafe extern "C" fn(*mut IMultiLanguage, *mut LCID, BSTR) -> HRESULT,
    pub EnumRfc1766: unsafe extern "C" fn(*mut IMultiLanguage, *mut *mut IEnumRfc1766) -> HRESULT,
    pub GetRfc1766Info: unsafe extern "C" fn(*mut IMultiLanguage, LCID, *mut RFC1766INFO) -> HRESULT,
    pub CreateConvertCharset: unsafe extern "C" fn(*mut IMultiLanguage, u32, u32, u32, *mut *mut IMLangConvertCharset) -> HRESULT,
}

#[repr(C)]
pub struct IMultiLanguage3Vtbl {
    pub QueryInterface: unsafe extern "C" fn(*mut IMultiLanguage3, *const GUID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "C" fn(*mut IMultiLanguage3) -> u32,
    pub Release: unsafe extern "C" fn(*mut IMultiLanguage3) -> u32,
    pub GetNumberOfCodePageInfo: unsafe extern "C" fn(*mut IMultiLanguage3, *mut u32) -> HRESULT,
    pub GetCodePageInfo: unsafe extern "C" fn(*mut IMultiLanguage3, u32, u16, *mut MIMECPINFO) -> HRESULT,
    pub GetFamilyCodePage: unsafe extern "C" fn(*mut IMultiLanguage3, u32, *mut u32) -> HRESULT,
    pub EnumCodePages: unsafe extern "C" fn(*mut IMultiLanguage3, u32, u16, *mut *mut IEnumCodePage) -> HRESULT,
    pub GetCharsetInfo: unsafe extern "C" fn(*mut IMultiLanguage3, *mut u16, *mut MIMECSETINFO) -> HRESULT,
    pub IsConvertible: unsafe extern "C" fn(*mut IMultiLanguage3, u32, u32) -> HRESULT,
    pub ConvertString: unsafe extern "C" fn(*mut IMultiLanguage3, *mut u32, u32, u32, *mut u8, *mut u32, *mut u8, *mut u32) -> HRESULT,
    pub ConvertStringToUnicode: unsafe extern "C" fn(*mut IMultiLanguage3, *mut u32, u32, *mut i8, *mut u32, *mut u16, *mut u32) -> HRESULT,
    pub ConvertStringFromUnicode: unsafe extern "C" fn(*mut IMultiLanguage3, *mut u32, u32, *mut u16, *mut u32, *mut i8, *mut u32) -> HRESULT,
    pub ConvertStringReset: unsafe extern "C" fn(*mut IMultiLanguage3) -> HRESULT,
    pub GetRfc1766FromLcid: unsafe extern "C" fn(*mut IMultiLanguage3, LCID, *mut *mut u16) -> HRESULT,
    pub GetLcidFromRfc1766: unsafe extern "C" fn(*mut IMultiLanguage3, *mut LCID, *const u16) -> HRESULT,
    pub EnumRfc1766: unsafe extern "C" fn(*mut IMultiLanguage3, LANGID, *mut *mut IEnumRfc1766) -> HRESULT,
    pub GetRfc1766Info: unsafe extern "C" fn(*mut IMultiLanguage3, LCID, u16, *mut RFC1766INFO) -> HRESULT,
    pub CreateConvertCharset: unsafe extern "C" fn(*mut IMultiLanguage3, u32, u32, u32, *mut *mut IMLangConvertCharset) -> HRESULT,
    pub ConvertStringInIStream: unsafe extern "C" fn(*mut IMultiLanguage3, *mut u32, u32, *const u16, u32, u32, *mut IStream, *mut IStream) -> HRESULT,
    pub ConvertStringToUnicodeEx: unsafe extern "C" fn(*mut IMultiLanguage3, *mut u32, u32, *mut i8, *mut u32, *mut u16, *mut u32, u32, *mut u16) -> HRESULT,
    pub ConvertStringFromUnicodeEx: unsafe extern "C" fn(*mut IMultiLanguage3, *mut u32, u32, *mut u16, *mut u32, *mut i8, *mut u32, u32, *mut u16) -> HRESULT,
    pub DetectCodepageInIStream: unsafe extern "C" fn(*mut IMultiLanguage3, u32, u32, *mut IStream, *mut DetectEncodingInfo, *mut i32) -> HRESULT,
    pub DetectInputCodepage: unsafe extern "C" fn(*mut IMultiLanguage3, u32, u32, *mut i8, *mut i32, *mut DetectEncodingInfo, *mut i32) -> HRESULT,
    pub ValidateCodePage: unsafe extern "C" fn(*mut IMultiLanguage3, u32, isize) -> HRESULT,
    pub GetCodePageDescription: unsafe extern "C" fn(*mut IMultiLanguage3, u32, u32, *mut u16, i32) -> HRESULT,
    pub IsCodePageInstallable: unsafe extern "C" fn(*mut IMultiLanguage3, u32) -> HRESULT,
    pub SetMimeDBSource: unsafe extern "C" fn(*mut IMultiLanguage3, u32) -> HRESULT,
    pub GetNumberOfScripts: unsafe extern "C" fn(*mut IMultiLanguage3, *mut u32) -> HRESULT,
    pub EnumScripts: unsafe extern "C" fn(*mut IMultiLanguage3, u32, u16, *mut *mut IEnumScript) -> HRESULT,
    pub ValidateCodePageEx: unsafe extern "C" fn(*mut IMultiLanguage3, u32, HWND, u32) -> HRESULT,
    pub DetectOutboundCodePage: unsafe extern "C" fn(*mut IMultiLanguage3, u32, *const u16, u32, *mut u32, u32, *mut u32, *mut u32, *mut u16) -> HRESULT,
    pub DetectOutboundCodePageInIStream: unsafe extern "C" fn(*mut IMultiLanguage3, u32, *mut IStream, *mut u32, u32, *mut u32, *mut u32, *mut u16) -> HRESULT,
}

#[repr(C)]
pub struct IMLangLineBreakConsoleVtbl {
    pub QueryInterface: unsafe extern "C" fn(*mut IMLangLineBreakConsole, *const c_void, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "C" fn(*mut IMLangLineBreakConsole) -> u32,
    pub Release: unsafe extern "C" fn(*mut IMLangLineBreakConsole) -> u32,
    pub BreakLineML: unsafe extern "C" fn(*mut IMLangLineBreakConsole, *mut IMLangString, i32, i32, i32, i32, *mut i32, *mut i32) -> HRESULT,
    pub BreakLineW: unsafe extern "C" fn(*mut IMLangLineBreakConsole, u32, *const u16, i32, i32, *mut i32, *mut i32) -> HRESULT,
    pub BreakLineA: unsafe extern "C" fn(*mut IMLangLineBreakConsole, u32, u32, *const i8, i32, i32, *mut i32, *mut i32) -> HRESULT,
}

#[repr(C)]
pub struct IMLangConvertCharsetVtbl {
    pub QueryInterface: unsafe extern "C" fn(*mut IMLangConvertCharset, *const GUID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "C" fn(*mut IMLangConvertCharset) -> u32,
    pub Release: unsafe extern "C" fn(*mut IMLangConvertCharset) -> u32,
    pub Initialize: unsafe extern "C" fn(*mut IMLangConvertCharset, u32, u32, u32) -> HRESULT,
    pub GetSourceCodePage: unsafe extern "C" fn(*mut IMLangConvertCharset, *mut u32) -> HRESULT,
    pub GetDestinationCodePage: unsafe extern "C" fn(*mut IMLangConvertCharset, *mut u32) -> HRESULT,
    pub GetProperty: unsafe extern "C" fn(*mut IMLangConvertCharset, *mut u32) -> HRESULT,
    pub DoConversion: unsafe extern "C" fn(*mut IMLangConvertCharset, *mut u8, *mut u32, *mut u8, *mut u32) -> HRESULT,
    pub DoConversionToUnicode: unsafe extern "C" fn(*mut IMLangConvertCharset, *mut i8, *mut u32, *mut u16, *mut u32) -> HRESULT,
    pub DoConversionFromUnicode: unsafe extern "C" fn(*mut IMLangConvertCharset, *mut u16, *mut u32, *mut i8, *mut u32) -> HRESULT,
}

// ---- COM interface types (struct with lpVtbl) ----
#[repr(C)]
pub struct IUnknown {
    pub lpVtbl: *const IUnknownVtbl,
}

#[repr(C)]
pub struct IUnknownVtbl {
    pub QueryInterface: unsafe extern "C" fn(*mut IUnknown, *const GUID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "C" fn(*mut IUnknown) -> u32,
    pub Release: unsafe extern "C" fn(*mut IUnknown) -> u32,
}

#[repr(C)]
pub struct IClassFactory {
    pub lpVtbl: *const IClassFactoryVtbl,
}

#[repr(C)]
pub struct IEnumCodePage {
    pub lpVtbl: *const IEnumCodePageVtbl,
}

#[repr(C)]
pub struct IEnumScript {
    pub lpVtbl: *const IEnumScriptVtbl,
}

#[repr(C)]
pub struct IEnumRfc1766 {
    pub lpVtbl: *const IEnumRfc1766Vtbl,
}

#[repr(C)]
pub struct IMLangFontLink {
    pub lpVtbl: *const IMLangFontLinkVtbl,
}

#[repr(C)]
pub struct IMLangFontLink2 {
    pub lpVtbl: *const IMLangFontLink2Vtbl,
}

#[repr(C)]
pub struct IMultiLanguage {
    pub lpVtbl: *const IMultiLanguageVtbl,
}

#[repr(C)]
pub struct IMultiLanguage3 {
    pub lpVtbl: *const IMultiLanguage3Vtbl,
}

#[repr(C)]
pub struct IMLangLineBreakConsole {
    pub lpVtbl: *const IMLangLineBreakConsoleVtbl,
}

#[repr(C)]
pub struct IMLangConvertCharset {
    pub lpVtbl: *const IMLangConvertCharsetVtbl,
}

#[repr(C)]
pub struct IMLangString {
    pub lpVtbl: *const c_void,
}

// ---- data structs ----
#[repr(C)]
pub struct MIMECPINFO {
    pub dwFlags: u32,
    pub uiCodePage: u32,
    pub uiFamilyCodePage: u32,
    pub wszDescription: [u16; 64],
    pub wszWebCharset: [u16; 50],
    pub wszHeaderCharset: [u16; 50],
    pub wszBodyCharset: [u16; 50],
    pub wszFixedWidthFont: [u16; 32],
    pub wszProportionalFont: [u16; 32],
    pub bGDICharset: u8,
    pub __pad: [u8; 3],
}

#[repr(C)]
pub struct SCRIPTINFO {
    pub ScriptId: SCRIPT_ID,
    pub uiCodePage: u32,
    pub wszDescription: [u16; 50],
    pub wszFixedWidthFont: [u16; 32],
    pub wszProportionalFont: [u16; 32],
}

#[repr(C)]
pub struct RFC1766INFO {
    pub lcid: LCID,
    pub wszRfc1766: [u16; 6],
    pub wszLocaleName: [u16; 32],
}

#[repr(C)]
pub struct CHARSETINFO {
    pub ciCharset: u32,
    pub ciACP: u32,
    pub fs: FONTSIGNATURE,
}

#[repr(C)]
pub struct FONTSIGNATURE {
    pub fsUsb: [u32; 4],
    pub fsCsb: [u32; 2],
}

#[repr(C)]
pub struct UNICODERANGE {
    pub wcFrom: u16,
    pub wcTo: u16,
}

#[repr(C)]
pub struct SCRIPTFONTINFO {
    pub scripts: i64,
    pub wszFont: [u16; 32],
}

#[repr(C)]
pub struct DetectEncodingInfo {
    pub nLangID: u32,
    pub nCodePage: u32,
    pub nDocPercent: i32,
    pub nConfidence: i32,
}

#[repr(C)]
pub struct GLYPHSET {
    pub cbThis: u32,
    pub flAccel: u32,
    pub cGlyphsSupported: u32,
    pub cRanges: u32,
    pub ranges: [WCRANGE; 1], // variable-length, 1 for static sizing
}

#[repr(C)]
pub struct WCRANGE {
    pub wcLow: u16,
    pub cGlyphs: u16,
}

#[repr(C)]
pub struct LOGFONTW {
    pub lfHeight: i32,
    pub lfWidth: i32,
    pub lfEscapement: i32,
    pub lfOrientation: i32,
    pub lfWeight: i32,
    pub lfItalic: u8,
    pub lfUnderline: u8,
    pub lfStrikeOut: u8,
    pub lfCharSet: u8,
    pub lfOutPrecision: u8,
    pub lfClipPrecision: u8,
    pub lfQuality: u8,
    pub lfPitchAndFamily: u8,
    pub lfFaceName: [u16; 32],
}

// ---- file-scope type preamble (translated) ----
#[repr(C)]
pub struct MIME_CP_INFO {
    pub description: *const i8,
    pub cp: u32,
    pub flags: u32,
    pub web_charset: *const i8,
    pub header_charset: *const i8,
    pub body_charset: *const i8,
    pub alias: *const u16,
}
unsafe impl Sync for MIME_CP_INFO {}

pub static arabic_cp: [MIME_CP_INFO; 4] = [
    MIME_CP_INFO {
        description: b"Arabic (864)\0".as_ptr() as *const i8,
        cp: 864,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS
             | MIMECONTF_MIME_LATEST,
        web_charset: b"ibm864\0".as_ptr() as *const i8,
        header_charset: b"ibm864\0".as_ptr() as *const i8,
        body_charset: b"ibm864\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Arabic (1006)\0".as_ptr() as *const i8,
        cp: 1006,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS
             | MIMECONTF_MIME_LATEST,
        web_charset: b"ibm1006\0".as_ptr() as *const i8,
        header_charset: b"ibm1006\0".as_ptr() as *const i8,
        body_charset: b"ibm1006\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Arabic (Windows)\0".as_ptr() as *const i8,
        cp: 1256,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_IMPORT
             | MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_SAVABLE_BROWSER
             | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"windows-1256\0".as_ptr() as *const i8,
        header_charset: b"windows-1256\0".as_ptr() as *const i8,
        body_charset: b"windows-1256\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Arabic (ISO)\0".as_ptr() as *const i8,
        cp: 28596,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL
             | MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS
             | MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
        web_charset: b"iso-8859-6\0".as_ptr() as *const i8,
        header_charset: b"iso-8859-6\0".as_ptr() as *const i8,
        body_charset: b"iso-8859-6\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
];

pub static baltic_cp: [MIME_CP_INFO; 4] = [
    MIME_CP_INFO {
        description: b"Baltic (DOS)\0".as_ptr() as *const i8,
        cp: 775,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"ibm775\0".as_ptr() as *const i8,
        header_charset: b"ibm775\0".as_ptr() as *const i8,
        body_charset: b"ibm775\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Baltic (Windows)\0".as_ptr() as *const i8,
        cp: 1257,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL
             | MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS
             | MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"windows-1257\0".as_ptr() as *const i8,
        header_charset: b"windows-1257\0".as_ptr() as *const i8,
        body_charset: b"windows-1257\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Baltic (ISO)\0".as_ptr() as *const i8,
        cp: 28594,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_IMPORT
             | MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_SAVABLE_BROWSER
             | MIMECONTF_EXPORT | MIMECONTF_VALID | MIMECONTF_VALID_NLS
             | MIMECONTF_MIME_LATEST,
        web_charset: b"iso-8859-4\0".as_ptr() as *const i8,
        header_charset: b"iso-8859-4\0".as_ptr() as *const i8,
        body_charset: b"iso-8859-4\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Estonian (ISO)\0".as_ptr() as *const i8,
        cp: 28603,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"iso-8859-13\0".as_ptr() as *const i8,
        header_charset: b"iso-8859-13\0".as_ptr() as *const i8,
        body_charset: b"iso-8859-13\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
];

pub static chinese_simplified_cp: [MIME_CP_INFO; 6] = [
    MIME_CP_INFO {
        description: b"Chinese Simplified (Auto-Select)\0".as_ptr() as *const i8,
        cp: 50936,
        flags: MIMECONTF_IMPORT | MIMECONTF_VALID | MIMECONTF_VALID_NLS
             | MIMECONTF_MIME_LATEST,
        web_charset: b"_autodetect_chs\0".as_ptr() as *const i8,
        header_charset: b"_autodetect_chs\0".as_ptr() as *const i8,
        body_charset: b"_autodetect_chs\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Chinese Simplified (GB2312)\0".as_ptr() as *const i8,
        cp: 936,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL
             | MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_VALID
             | MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS
             | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
        web_charset: b"gb2312\0".as_ptr() as *const i8,
        header_charset: b"gb2312\0".as_ptr() as *const i8,
        body_charset: b"gb2312\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Chinese Simplified (GB2312-80)\0".as_ptr() as *const i8,
        cp: 20936,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"x-cp20936\0".as_ptr() as *const i8,
        header_charset: b"x-cp20936\0".as_ptr() as *const i8,
        body_charset: b"x-cp20936\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Chinese Simplified (HZ)\0".as_ptr() as *const i8,
        cp: 52936,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_IMPORT
             | MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT
             | MIMECONTF_VALID | MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4
             | MIMECONTF_MIME_LATEST,
        web_charset: b"hz-gb-2312\0".as_ptr() as *const i8,
        header_charset: b"hz-gb-2312\0".as_ptr() as *const i8,
        body_charset: b"hz-gb-2312\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Chinese Simplified (GB18030)\0".as_ptr() as *const i8,
        cp: 54936,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL
             | MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_SAVABLE_BROWSER
             | MIMECONTF_EXPORT | MIMECONTF_VALID | MIMECONTF_VALID_NLS
             | MIMECONTF_MIME_LATEST,
        web_charset: b"GB18030\0".as_ptr() as *const i8,
        header_charset: b"GB18030\0".as_ptr() as *const i8,
        body_charset: b"GB18030\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Chinese Simplified (GBK)\0".as_ptr() as *const i8,
        cp: 936,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL
             | MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS
             | MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS
             | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
        web_charset: b"gbk\0".as_ptr() as *const i8,
        header_charset: b"gbk\0".as_ptr() as *const i8,
        body_charset: b"gbk\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
];

pub static chinese_traditional_cp: [MIME_CP_INFO; 3] = [
    MIME_CP_INFO {
        description: b"Chinese Traditional (Auto-Select)\0".as_ptr() as *const i8,
        cp: 50950,
        flags: MIMECONTF_IMPORT | MIMECONTF_VALID | MIMECONTF_VALID_NLS
             | MIMECONTF_MIME_LATEST,
        web_charset: b"_autodetect_cht\0".as_ptr() as *const i8,
        header_charset: b"_autodetect_cht\0".as_ptr() as *const i8,
        body_charset: b"_autodetect_cht\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Chinese Traditional (Big5)\0".as_ptr() as *const i8,
        cp: 950,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL
             | MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS
             | MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
        web_charset: b"big5\0".as_ptr() as *const i8,
        header_charset: b"big5\0".as_ptr() as *const i8,
        body_charset: b"big5\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Chinese Traditional (CNS)\0".as_ptr() as *const i8,
        cp: 20000,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"x-Chinese-CNS\0".as_ptr() as *const i8,
        header_charset: b"x-Chinese-CNS\0".as_ptr() as *const i8,
        body_charset: b"x-Chinese-CNS\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
];

pub static central_european_cp: [MIME_CP_INFO; 4] = [
    MIME_CP_INFO {
        description: b"Central European (DOS)\0".as_ptr() as *const i8,
        cp: 852,
        flags: MIMECONTF_BROWSER | MIMECONTF_IMPORT | MIMECONTF_SAVABLE_BROWSER
             | MIMECONTF_EXPORT | MIMECONTF_VALID | MIMECONTF_VALID_NLS
             | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
        web_charset: b"ibm852\0".as_ptr() as *const i8,
        header_charset: b"ibm852\0".as_ptr() as *const i8,
        body_charset: b"ibm852\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Central European (Windows)\0".as_ptr() as *const i8,
        cp: 1250,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_IMPORT
             | MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_SAVABLE_BROWSER
             | MIMECONTF_EXPORT | MIMECONTF_VALID | MIMECONTF_VALID_NLS
             | MIMECONTF_MIME_LATEST,
        web_charset: b"windows-1250\0".as_ptr() as *const i8,
        header_charset: b"windows-1250\0".as_ptr() as *const i8,
        body_charset: b"windows-1250\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Central European (Mac)\0".as_ptr() as *const i8,
        cp: 10029,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"x-mac-ce\0".as_ptr() as *const i8,
        header_charset: b"x-mac-ce\0".as_ptr() as *const i8,
        body_charset: b"x-mac-ce\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Central European (ISO)\0".as_ptr() as *const i8,
        cp: 28592,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL
             | MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS
             | MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
        web_charset: b"iso-8859-2\0".as_ptr() as *const i8,
        header_charset: b"iso-8859-2\0".as_ptr() as *const i8,
        body_charset: b"iso-8859-2\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
];

pub static cyrillic_cp: [MIME_CP_INFO; 7] = [
    MIME_CP_INFO {
        description: b"OEM Cyrillic\0".as_ptr() as *const i8,
        cp: 855,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"ibm855\0".as_ptr() as *const i8,
        header_charset: b"ibm855\0".as_ptr() as *const i8,
        body_charset: b"ibm855\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Cyrillic (DOS)\0".as_ptr() as *const i8,
        cp: 866,
        flags: MIMECONTF_BROWSER | MIMECONTF_IMPORT | MIMECONTF_SAVABLE_BROWSER
             | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4
             | MIMECONTF_MIME_LATEST,
        web_charset: b"cp866\0".as_ptr() as *const i8,
        header_charset: b"cp866\0".as_ptr() as *const i8,
        body_charset: b"cp866\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Cyrillic (Windows)\0".as_ptr() as *const i8,
        cp: 1251,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_IMPORT
             | MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_SAVABLE_BROWSER
             | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"windows-1251\0".as_ptr() as *const i8,
        header_charset: b"windows-1251\0".as_ptr() as *const i8,
        body_charset: b"windows-1251\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Cyrillic (Mac)\0".as_ptr() as *const i8,
        cp: 10007,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS
             | MIMECONTF_MIME_LATEST,
        web_charset: b"x-mac-cyrillic\0".as_ptr() as *const i8,
        header_charset: b"x-mac-cyrillic\0".as_ptr() as *const i8,
        body_charset: b"x-mac-cyrillic\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Cyrillic (KOI8-R)\0".as_ptr() as *const i8,
        cp: 20866,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL
             | MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS
             | MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
        web_charset: b"koi8-r\0".as_ptr() as *const i8,
        header_charset: b"koi8-r\0".as_ptr() as *const i8,
        body_charset: b"koi8-r\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Cyrillic (KOI8-U)\0".as_ptr() as *const i8,
        cp: 21866,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL
             | MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS
             | MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
        web_charset: b"koi8-u\0".as_ptr() as *const i8,
        header_charset: b"koi8-u\0".as_ptr() as *const i8,
        body_charset: b"koi8-u\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Cyrillic (ISO)\0".as_ptr() as *const i8,
        cp: 28595,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL
             | MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS
             | MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
        web_charset: b"iso-8859-5\0".as_ptr() as *const i8,
        header_charset: b"iso-8859-5\0".as_ptr() as *const i8,
        body_charset: b"iso-8859-5\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
];

pub static greek_cp: [MIME_CP_INFO; 6] = [
    MIME_CP_INFO {
        description: b"Greek (DOS)\0".as_ptr() as *const i8,
        cp: 737,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS
             | MIMECONTF_MIME_LATEST,
        web_charset: b"ibm737\0".as_ptr() as *const i8,
        header_charset: b"ibm737\0".as_ptr() as *const i8,
        body_charset: b"ibm737\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Greek, Modern (DOS)\0".as_ptr() as *const i8,
        cp: 869,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS
             | MIMECONTF_MIME_LATEST,
        web_charset: b"ibm869\0".as_ptr() as *const i8,
        header_charset: b"ibm869\0".as_ptr() as *const i8,
        body_charset: b"ibm869\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"IBM EBCDIC (Greek Modern)\0".as_ptr() as *const i8,
        cp: 875,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS
             | MIMECONTF_MIME_LATEST,
        web_charset: b"cp875\0".as_ptr() as *const i8,
        header_charset: b"cp875\0".as_ptr() as *const i8,
        body_charset: b"cp875\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Greek (Windows)\0".as_ptr() as *const i8,
        cp: 1253,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_IMPORT
             | MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_SAVABLE_BROWSER
             | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"windows-1253\0".as_ptr() as *const i8,
        header_charset: b"windows-1253\0".as_ptr() as *const i8,
        body_charset: b"windows-1253\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Greek (Mac)\0".as_ptr() as *const i8,
        cp: 10006,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS
             | MIMECONTF_MIME_LATEST,
        web_charset: b"x-mac-greek\0".as_ptr() as *const i8,
        header_charset: b"x-mac-greek\0".as_ptr() as *const i8,
        body_charset: b"x-mac-greek\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Greek (ISO)\0".as_ptr() as *const i8,
        cp: 28597,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL
             | MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS
             | MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
        web_charset: b"iso-8859-7\0".as_ptr() as *const i8,
        header_charset: b"iso-8859-7\0".as_ptr() as *const i8,
        body_charset: b"iso-8859-7\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
];

pub static hebrew_cp: [MIME_CP_INFO; 5] = [
    MIME_CP_INFO {
        description: b"Hebrew (424)\0".as_ptr() as *const i8,
        cp: 424,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS
             | MIMECONTF_MIME_LATEST,
        web_charset: b"ibm424\0".as_ptr() as *const i8,
        header_charset: b"ibm424\0".as_ptr() as *const i8,
        body_charset: b"ibm424\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Hebrew (856)\0".as_ptr() as *const i8,
        cp: 856,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS
             | MIMECONTF_MIME_LATEST,
        web_charset: b"cp856\0".as_ptr() as *const i8,
        header_charset: b"cp856\0".as_ptr() as *const i8,
        body_charset: b"cp856\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Hebrew (DOS)\0".as_ptr() as *const i8,
        cp: 862,
        flags: MIMECONTF_BROWSER | MIMECONTF_MINIMAL | MIMECONTF_IMPORT
             | MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS
             | MIMECONTF_MIME_LATEST,
        web_charset: b"dos-862\0".as_ptr() as *const i8,
        header_charset: b"dos-862\0".as_ptr() as *const i8,
        body_charset: b"dos-862\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Hebrew (Windows)\0".as_ptr() as *const i8,
        cp: 1255,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_IMPORT
             | MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_SAVABLE_BROWSER
             | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"windows-1255\0".as_ptr() as *const i8,
        header_charset: b"windows-1255\0".as_ptr() as *const i8,
        body_charset: b"windows-1255\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Hebrew (ISO-Visual)\0".as_ptr() as *const i8,
        cp: 28598,
        flags: MIMECONTF_BROWSER | MIMECONTF_MINIMAL | MIMECONTF_IMPORT
             | MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
        web_charset: b"iso-8859-8\0".as_ptr() as *const i8,
        header_charset: b"iso-8859-8\0".as_ptr() as *const i8,
        body_charset: b"iso-8859-8\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
];

pub static japanese_cp: [MIME_CP_INFO; 8] = [
    MIME_CP_INFO {
        description: b"Japanese (Auto-Select)\0".as_ptr() as *const i8,
        cp: 50932,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL
             | MIMECONTF_IMPORT | MIMECONTF_VALID | MIMECONTF_VALID_NLS
             | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
        web_charset: b"_autodetect\0".as_ptr() as *const i8,
        header_charset: b"_autodetect\0".as_ptr() as *const i8,
        body_charset: b"_autodetect\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Japanese (EUC)\0".as_ptr() as *const i8,
        cp: 51932,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL
             | MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS
             | MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
        web_charset: b"euc-jp\0".as_ptr() as *const i8,
        header_charset: b"euc-jp\0".as_ptr() as *const i8,
        body_charset: b"euc-jp\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Japanese (JIS)\0".as_ptr() as *const i8,
        cp: 50220,
        flags: MIMECONTF_IMPORT | MIMECONTF_MAILNEWS | MIMECONTF_EXPORT
             | MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_VALID_NLS
             | MIMECONTF_PRIVCONVERTER | MIMECONTF_MIME_LATEST
             | MIMECONTF_MIME_IE4,
        web_charset: b"iso-2022-jp\0".as_ptr() as *const i8,
        header_charset: b"iso-2022-jp\0".as_ptr() as *const i8,
        body_charset: b"iso-2022-jp\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Japanese (JIS 0208-1990 and 0212-1990)\0".as_ptr() as *const i8,
        cp: 20932,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS
             | MIMECONTF_VALID | MIMECONTF_PRIVCONVERTER | MIMECONTF_MIME_LATEST,
        web_charset: b"EUC-JP\0".as_ptr() as *const i8,
        header_charset: b"EUC-JP\0".as_ptr() as *const i8,
        body_charset: b"EUC-JP\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Japanese (JIS-Allow 1 byte Kana)\0".as_ptr() as *const i8,
        cp: 50221,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_EXPORT | MIMECONTF_SAVABLE_BROWSER
             | MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_VALID_NLS
             | MIMECONTF_VALID | MIMECONTF_PRIVCONVERTER | MIMECONTF_MIME_LATEST,
        web_charset: b"csISO2022JP\0".as_ptr() as *const i8,
        header_charset: b"iso-2022-jp\0".as_ptr() as *const i8,
        body_charset: b"iso-2022-jp\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Japanese (JIS-Allow 1 byte Kana - SO/SI)\0".as_ptr() as *const i8,
        cp: 50222,
        flags: MIMECONTF_EXPORT | MIMECONTF_VALID_NLS | MIMECONTF_VALID
             | MIMECONTF_PRIVCONVERTER | MIMECONTF_MIME_LATEST,
        web_charset: b"iso-2022-jp\0".as_ptr() as *const i8,
        header_charset: b"iso-2022-jp\0".as_ptr() as *const i8,
        body_charset: b"iso-2022-jp\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Japanese (Mac)\0".as_ptr() as *const i8,
        cp: 10001,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS
             | MIMECONTF_VALID | MIMECONTF_PRIVCONVERTER | MIMECONTF_MIME_LATEST,
        web_charset: b"x-mac-japanese\0".as_ptr() as *const i8,
        header_charset: b"x-mac-japanese\0".as_ptr() as *const i8,
        body_charset: b"x-mac-japanese\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Japanese (Shift-JIS)\0".as_ptr() as *const i8,
        cp: 932,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL
             | MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS
             | MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
        web_charset: b"shift_jis\0".as_ptr() as *const i8,
        header_charset: b"iso-2022-jp\0".as_ptr() as *const i8,
        body_charset: b"iso-2022-jp\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
];

pub static korean_cp: [MIME_CP_INFO; 1] = [
    MIME_CP_INFO {
        description: b"Korean\0".as_ptr() as *const i8,
        cp: 949,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL
             | MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS
             | MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS
             | MIMECONTF_MIME_LATEST,
        web_charset: b"ks_c_5601-1987\0".as_ptr() as *const i8,
        header_charset: b"ks_c_5601-1987\0".as_ptr() as *const i8,
        body_charset: b"ks_c_5601-1987\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
];

pub static thai_cp: [MIME_CP_INFO; 1] = [
    MIME_CP_INFO {
        description: b"Thai (Windows)\0".as_ptr() as *const i8,
        cp: 874,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_MIME_LATEST,
        web_charset: b"ibm-thai\0".as_ptr() as *const i8,
        header_charset: b"ibm-thai\0".as_ptr() as *const i8,
        body_charset: b"ibm-thai\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
];

pub static turkish_cp: [MIME_CP_INFO; 6] = [
    MIME_CP_INFO {
        description: b"Turkish (DOS)\0".as_ptr() as *const i8,
        cp: 857,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"ibm857\0".as_ptr() as *const i8,
        header_charset: b"ibm857\0".as_ptr() as *const i8,
        body_charset: b"ibm857\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"IBM EBCDIC (Turkish Latin-5)\0".as_ptr() as *const i8,
        cp: 1026,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"ibm1026\0".as_ptr() as *const i8,
        header_charset: b"ibm1026\0".as_ptr() as *const i8,
        body_charset: b"ibm1026\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Turkish (Windows)\0".as_ptr() as *const i8,
        cp: 1254,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL
             | MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS
             | MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"windows-1254\0".as_ptr() as *const i8,
        header_charset: b"windows-1254\0".as_ptr() as *const i8,
        body_charset: b"windows-1254\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Turkish (Mac)\0".as_ptr() as *const i8,
        cp: 10081,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"x-mac-turkish\0".as_ptr() as *const i8,
        header_charset: b"x-mac-turkish\0".as_ptr() as *const i8,
        body_charset: b"x-mac-turkish\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Latin 3 (ISO)\0".as_ptr() as *const i8,
        cp: 28593,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_IMPORT
             | MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
        web_charset: b"iso-8859-3\0".as_ptr() as *const i8,
        header_charset: b"iso-8859-3\0".as_ptr() as *const i8,
        body_charset: b"iso-8859-3\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Turkish (ISO)\0".as_ptr() as *const i8,
        cp: 28599,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL
             | MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS
             | MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"iso-8859-9\0".as_ptr() as *const i8,
        header_charset: b"iso-8859-9\0".as_ptr() as *const i8,
        body_charset: b"iso-8859-9\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
];

pub static vietnamese_cp: [MIME_CP_INFO; 1] = [
    MIME_CP_INFO {
        description: b"Vietnamese (Windows)\0".as_ptr() as *const i8,
        cp: 1258,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_IMPORT
             | MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_SAVABLE_BROWSER
             | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4
             | MIMECONTF_MIME_LATEST,
        web_charset: b"windows-1258\0".as_ptr() as *const i8,
        header_charset: b"windows-1258\0".as_ptr() as *const i8,
        body_charset: b"windows-1258\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
];

pub static asciiW: [u16; 6] = [ 'a' as u16, 's' as u16, 'c' as u16, 'i' as u16, 'i' as u16, 0 ];

pub static western_cp: [MIME_CP_INFO; 14] = [
    MIME_CP_INFO {
        description: b"IBM EBCDIC (US-Canada)\0".as_ptr() as *const i8,
        cp: 37,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"ibm037\0".as_ptr() as *const i8,
        header_charset: b"ibm037\0".as_ptr() as *const i8,
        body_charset: b"ibm037\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"OEM United States\0".as_ptr() as *const i8,
        cp: 437,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"ibm437\0".as_ptr() as *const i8,
        header_charset: b"ibm437\0".as_ptr() as *const i8,
        body_charset: b"ibm437\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"IBM EBCDIC (International)\0".as_ptr() as *const i8,
        cp: 500,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"ibm500\0".as_ptr() as *const i8,
        header_charset: b"ibm500\0".as_ptr() as *const i8,
        body_charset: b"ibm500\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Western European (DOS)\0".as_ptr() as *const i8,
        cp: 850,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"ibm850\0".as_ptr() as *const i8,
        header_charset: b"ibm850\0".as_ptr() as *const i8,
        body_charset: b"ibm850\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Portuguese (DOS)\0".as_ptr() as *const i8,
        cp: 860,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"ibm860\0".as_ptr() as *const i8,
        header_charset: b"ibm860\0".as_ptr() as *const i8,
        body_charset: b"ibm860\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Icelandic (DOS)\0".as_ptr() as *const i8,
        cp: 861,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"ibm861\0".as_ptr() as *const i8,
        header_charset: b"ibm861\0".as_ptr() as *const i8,
        body_charset: b"ibm861\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"French Canadian (DOS)\0".as_ptr() as *const i8,
        cp: 863,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"ibm863\0".as_ptr() as *const i8,
        header_charset: b"ibm863\0".as_ptr() as *const i8,
        body_charset: b"ibm863\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Nordic (DOS)\0".as_ptr() as *const i8,
        cp: 865,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"ibm865\0".as_ptr() as *const i8,
        header_charset: b"ibm865\0".as_ptr() as *const i8,
        body_charset: b"ibm865\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Western European (Windows)\0".as_ptr() as *const i8,
        cp: 1252,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL
             | MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS
             | MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"windows-1252\0".as_ptr() as *const i8,
        header_charset: b"windows-1252\0".as_ptr() as *const i8,
        body_charset: b"iso-8859-1\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Western European (Mac)\0".as_ptr() as *const i8,
        cp: 10000,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"macintosh\0".as_ptr() as *const i8,
        header_charset: b"macintosh\0".as_ptr() as *const i8,
        body_charset: b"macintosh\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Icelandic (Mac)\0".as_ptr() as *const i8,
        cp: 10079,
        flags: MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"x-mac-icelandic\0".as_ptr() as *const i8,
        header_charset: b"x-mac-icelandic\0".as_ptr() as *const i8,
        body_charset: b"x-mac-icelandic\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"US-ASCII\0".as_ptr() as *const i8,
        cp: 20127,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_IMPORT | MIMECONTF_EXPORT
             | MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
        web_charset: b"us-ascii\0".as_ptr() as *const i8,
        header_charset: b"us-ascii\0".as_ptr() as *const i8,
        body_charset: b"us-ascii\0".as_ptr() as *const i8,
        alias: &asciiW as *const [u16; 6] as *const u16,
    },
    MIME_CP_INFO {
        description: b"Western European (ISO)\0".as_ptr() as *const i8,
        cp: 28591,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_IMPORT
             | MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_SAVABLE_BROWSER
             | MIMECONTF_EXPORT | MIMECONTF_VALID | MIMECONTF_VALID_NLS
             | MIMECONTF_MIME_LATEST,
        web_charset: b"iso-8859-1\0".as_ptr() as *const i8,
        header_charset: b"iso-8859-1\0".as_ptr() as *const i8,
        body_charset: b"iso-8859-1\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Latin 9 (ISO)\0".as_ptr() as *const i8,
        cp: 28605,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_IMPORT
             | MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_SAVABLE_BROWSER
             | MIMECONTF_EXPORT | MIMECONTF_VALID | MIMECONTF_VALID_NLS
             | MIMECONTF_MIME_LATEST,
        web_charset: b"iso-8859-15\0".as_ptr() as *const i8,
        header_charset: b"iso-8859-15\0".as_ptr() as *const i8,
        body_charset: b"iso-8859-15\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
];

pub static unicode_cp: [MIME_CP_INFO; 3] = [
    MIME_CP_INFO {
        description: b"Unicode\0".as_ptr() as *const i8,
        cp: CP_UNICODE,
        flags: MIMECONTF_MINIMAL | MIMECONTF_IMPORT
             | MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT
             | MIMECONTF_VALID | MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4
             | MIMECONTF_MIME_LATEST,
        web_charset: b"unicode\0".as_ptr() as *const i8,
        header_charset: b"unicode\0".as_ptr() as *const i8,
        body_charset: b"unicode\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Unicode (UTF-7)\0".as_ptr() as *const i8,
        cp: CP_UTF7,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_IMPORT
             | MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_EXPORT | MIMECONTF_VALID
             | MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
        web_charset: b"utf-7\0".as_ptr() as *const i8,
        header_charset: b"utf-7\0".as_ptr() as *const i8,
        body_charset: b"utf-7\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
    MIME_CP_INFO {
        description: b"Unicode (UTF-8)\0".as_ptr() as *const i8,
        cp: CP_UTF8,
        flags: MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_IMPORT
             | MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_SAVABLE_BROWSER
             | MIMECONTF_EXPORT | MIMECONTF_VALID | MIMECONTF_VALID_NLS
             | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
        web_charset: b"utf-8\0".as_ptr() as *const i8,
        header_charset: b"utf-8\0".as_ptr() as *const i8,
        body_charset: b"utf-8\0".as_ptr() as *const i8,
        alias: core::ptr::null(),
    },
];

#[repr(C)]
pub struct mlang_data {
    pub description: *const i8,
    pub family_codepage: u32,
    pub number_of_cp: u32,
    pub mime_cp_info: *const MIME_CP_INFO,
    pub fixed_font: *const i8,
    pub proportional_font: *const i8,
    pub sid: SCRIPT_ID,
}

unsafe impl Sync for mlang_data {}

pub static mlang_data: [mlang_data; 15] = [
    mlang_data {
        description: b"Arabic\0".as_ptr() as *const i8,
        family_codepage: 1256,
        number_of_cp: 4, // ARRAY_SIZE(arabic_cp)
        mime_cp_info: &arabic_cp[0],
        fixed_font: b"Simplified Arabic Fixed\0".as_ptr() as *const i8,
        proportional_font: b"Simplified Arabic\0".as_ptr() as *const i8,
        sid: sidArabic,
    },
    mlang_data {
        description: b"Baltic\0".as_ptr() as *const i8,
        family_codepage: 1257,
        number_of_cp: 4,
        mime_cp_info: &baltic_cp[0],
        fixed_font: b"Courier New\0".as_ptr() as *const i8,
        proportional_font: b"Arial\0".as_ptr() as *const i8,
        sid: sidAsciiLatin,
    },
    mlang_data {
        description: b"Chinese Simplified\0".as_ptr() as *const i8,
        family_codepage: 936,
        number_of_cp: 6,
        mime_cp_info: &chinese_simplified_cp[0],
        fixed_font: b"Simsun\0".as_ptr() as *const i8,
        proportional_font: b"Simsun\0".as_ptr() as *const i8,
        sid: sidHan,
    },
    mlang_data {
        description: b"Chinese Traditional\0".as_ptr() as *const i8,
        family_codepage: 950,
        number_of_cp: 3,
        mime_cp_info: &chinese_traditional_cp[0],
        fixed_font: b"MingLiu\0".as_ptr() as *const i8,
        proportional_font: b"New MingLiu\0".as_ptr() as *const i8,
        sid: sidBopomofo,
    },
    mlang_data {
        description: b"Central European\0".as_ptr() as *const i8,
        family_codepage: 1250,
        number_of_cp: 4,
        mime_cp_info: &central_european_cp[0],
        fixed_font: b"Courier New\0".as_ptr() as *const i8,
        proportional_font: b"Arial\0".as_ptr() as *const i8,
        sid: sidAsciiLatin,
    },
    mlang_data {
        description: b"Cyrillic\0".as_ptr() as *const i8,
        family_codepage: 1251,
        number_of_cp: 7,
        mime_cp_info: &cyrillic_cp[0],
        fixed_font: b"Courier New\0".as_ptr() as *const i8,
        proportional_font: b"Arial\0".as_ptr() as *const i8,
        sid: sidCyrillic,
    },
    mlang_data {
        description: b"Greek\0".as_ptr() as *const i8,
        family_codepage: 1253,
        number_of_cp: 6,
        mime_cp_info: &greek_cp[0],
        fixed_font: b"Courier New\0".as_ptr() as *const i8,
        proportional_font: b"Arial\0".as_ptr() as *const i8,
        sid: sidGreek,
    },
    mlang_data {
        description: b"Hebrew\0".as_ptr() as *const i8,
        family_codepage: 1255,
        number_of_cp: 5,
        mime_cp_info: &hebrew_cp[0],
        fixed_font: b"Miriam Fixed\0".as_ptr() as *const i8,
        proportional_font: b"David\0".as_ptr() as *const i8,
        sid: sidHebrew,
    },
    mlang_data {
        description: b"Japanese\0".as_ptr() as *const i8,
        family_codepage: 932,
        number_of_cp: 8,
        mime_cp_info: &japanese_cp[0],
        fixed_font: b"MS Gothic\0".as_ptr() as *const i8,
        proportional_font: b"MS PGothic\0".as_ptr() as *const i8,
        sid: sidKana,
    },
    mlang_data {
        description: b"Korean\0".as_ptr() as *const i8,
        family_codepage: 949,
        number_of_cp: 1,
        mime_cp_info: &korean_cp[0],
        fixed_font: b"GulimChe\0".as_ptr() as *const i8,
        proportional_font: b"Gulim\0".as_ptr() as *const i8,
        sid: sidHangul,
    },
    mlang_data {
        description: b"Thai\0".as_ptr() as *const i8,
        family_codepage: 874,
        number_of_cp: 1,
        mime_cp_info: &thai_cp[0],
        fixed_font: b"Tahoma\0".as_ptr() as *const i8,
        proportional_font: b"Tahoma\0".as_ptr() as *const i8,
        sid: sidThai,
    },
    mlang_data {
        description: b"Turkish\0".as_ptr() as *const i8,
        family_codepage: 1254,
        number_of_cp: 6,
        mime_cp_info: &turkish_cp[0],
        fixed_font: b"Courier New\0".as_ptr() as *const i8,
        proportional_font: b"Arial\0".as_ptr() as *const i8,
        sid: sidAsciiLatin,
    },
    mlang_data {
        description: b"Vietnamese\0".as_ptr() as *const i8,
        family_codepage: 1258,
        number_of_cp: 1,
        mime_cp_info: &vietnamese_cp[0],
        fixed_font: b"Courier New\0".as_ptr() as *const i8,
        proportional_font: b"Arial\0".as_ptr() as *const i8,
        sid: sidAsciiLatin,
    },
    mlang_data {
        description: b"Western European\0".as_ptr() as *const i8,
        family_codepage: 1252,
        number_of_cp: 14,
        mime_cp_info: &western_cp[0],
        fixed_font: b"Courier New\0".as_ptr() as *const i8,
        proportional_font: b"Arial\0".as_ptr() as *const i8,
        sid: sidAsciiLatin,
    },
    mlang_data {
        description: b"Unicode\0".as_ptr() as *const i8,
        family_codepage: CP_UNICODE,
        number_of_cp: 3,
        mime_cp_info: &unicode_cp[0],
        fixed_font: b"Courier New\0".as_ptr() as *const i8,
        proportional_font: b"Arial\0".as_ptr() as *const i8,
        sid: sidAsciiLatin, // assuming default
    },
];

#[repr(C)]
pub struct font_list {
    pub list_entry: LIST_ENTRY,
    pub base_font: HFONT,
    pub font: HFONT,
    pub charset: u32,
}

pub static mut font_cache: LIST_ENTRY = unsafe {
    LIST_ENTRY {
        Flink: core::ptr::null_mut(),
        Blink: core::ptr::null_mut(),
    }
};

pub static mut font_cache_critical_debug: CRITICAL_SECTION_DEBUG = unsafe {
    CRITICAL_SECTION_DEBUG {
        Type: 0,
        CreatorBackTraceIndex: 0,
        CriticalSection: &raw mut font_cache_critical,
        ProcessLocksList: LIST_ENTRY {
            Flink: &raw const font_cache_critical_debug.ProcessLocksList as *mut LIST_ENTRY,
            Blink: &raw const font_cache_critical_debug.ProcessLocksList as *mut LIST_ENTRY,
        },
        EntryCount: 0,
        ContentionCount: 0,
        __bindgen_anon_1: unsafe { core::mem::zeroed() },
    }
};

pub static mut font_cache_critical: CRITICAL_SECTION = CRITICAL_SECTION {
    DebugInfo: unsafe { &raw mut font_cache_critical_debug },
    LockCount: -1,
    RecursionCount: 0,
    OwningThread: core::ptr::null_mut(),
    LockSemaphore: core::ptr::null_mut(),
    SpinCount: 0,
};

pub static mut dll_count: i32 = 0;

#[repr(C)]
pub struct IClassFactoryImpl {
    pub IClassFactory_iface: IClassFactory,
    pub ref_: i32,
    pub pfnCreateInstance: unsafe extern "C" fn(*mut IUnknown, *mut *mut c_void) -> HRESULT,
}

#[repr(C)]
pub struct object_creation_info {
    pub clsid: *const CLSID,
    pub szClassName: LPCSTR,
    pub pfnCreateInstance: unsafe extern "C" fn(*mut IUnknown, *mut *mut c_void) -> HRESULT,
}

unsafe impl Sync for object_creation_info {}

pub static object_creation: [object_creation_info; 2] = [
    object_creation_info {
        clsid: &CLSID_CMultiLanguage,
        szClassName: b"CLSID_CMultiLanguage\0".as_ptr() as LPCSTR,
        pfnCreateInstance: MultiLanguage_create,
    },
    object_creation_info {
        clsid: &CLSID_CMLangConvertCharset,
        szClassName: b"CLSID_CMLangConvertCharset\0".as_ptr() as LPCSTR,
        pfnCreateInstance: MLangConvertCharset_create,
    },
];

pub static MLANGCF_Vtbl: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: MLANGCF_QueryInterface,
    AddRef: MLANGCF_AddRef,
    Release: MLANGCF_Release,
    CreateInstance: MLANGCF_CreateInstance,
    LockServer: MLANGCF_LockServer,
};

#[repr(C)]
pub struct MLang_impl {
    pub IMLangFontLink_iface: IMLangFontLink,
    pub IMultiLanguage_iface: IMultiLanguage,
    pub IMultiLanguage3_iface: IMultiLanguage3,
    pub IMLangFontLink2_iface: IMLangFontLink2,
    pub IMLangLineBreakConsole_iface: IMLangLineBreakConsole,
    pub ref_: i32,
    pub total_cp: u32,
    pub total_scripts: u32,
}

#[repr(C)]
pub struct EnumCodePage_impl {
    pub IEnumCodePage_iface: IEnumCodePage,
    pub ref_: i32,
    pub cpinfo: *mut MIMECPINFO,
    pub total: u32,
    pub pos: u32,
}

pub static IEnumCodePage_vtbl: IEnumCodePageVtbl = IEnumCodePageVtbl {
    QueryInterface: fnIEnumCodePage_QueryInterface,
    AddRef: fnIEnumCodePage_AddRef,
    Release: fnIEnumCodePage_Release,
    Clone: fnIEnumCodePage_Clone,
    Next: fnIEnumCodePage_Next,
    Reset: fnIEnumCodePage_Reset,
    Skip: fnIEnumCodePage_Skip,
};

#[repr(C)]
pub struct EnumScript_impl {
    pub IEnumScript_iface: IEnumScript,
    pub ref_: i32,
    pub script_info: *mut SCRIPTINFO,
    pub total: u32,
    pub pos: u32,
}

pub static IEnumScript_vtbl: IEnumScriptVtbl = IEnumScriptVtbl {
    QueryInterface: fnIEnumScript_QueryInterface,
    AddRef: fnIEnumScript_AddRef,
    Release: fnIEnumScript_Release,
    Clone: fnIEnumScript_Clone,
    Next: fnIEnumScript_Next,
    Reset: fnIEnumScript_Reset,
    Skip: fnIEnumScript_Skip,
};

pub static IMLangFontLink_vtbl: IMLangFontLinkVtbl = IMLangFontLinkVtbl {
    QueryInterface: fnIMLangFontLink_QueryInterface,
    AddRef: fnIMLangFontLink_AddRef,
    Release: fnIMLangFontLink_Release,
    GetCharCodePages: fnIMLangFontLink_GetCharCodePages,
    GetStrCodePages: fnIMLangFontLink_GetStrCodePages,
    CodePageToCodePages: fnIMLangFontLink_CodePageToCodePages,
    CodePagesToCodePage: fnIMLangFontLink_CodePagesToCodePage,
    GetFontCodePages: fnIMLangFontLink_GetFontCodePages,
    MapFont: fnIMLangFontLink_MapFont,
    ReleaseFont: fnIMLangFontLink_ReleaseFont,
    ResetFontMapping: fnIMLangFontLink_ResetFontMapping,
};

#[repr(C)]
pub struct EnumRfc1766_impl {
    pub IEnumRfc1766_iface: IEnumRfc1766,
    pub ref_: i32,
    pub info: *mut RFC1766INFO,
    pub total: u32,
    pub pos: u32,
}

pub static IEnumRfc1766_vtbl: IEnumRfc1766Vtbl = IEnumRfc1766Vtbl {
    QueryInterface: fnIEnumRfc1766_QueryInterface,
    AddRef: fnIEnumRfc1766_AddRef,
    Release: fnIEnumRfc1766_Release,
    Clone: fnIEnumRfc1766_Clone,
    Next: fnIEnumRfc1766_Next,
    Reset: fnIEnumRfc1766_Reset,
    Skip: fnIEnumRfc1766_Skip,
};

#[repr(C)]
pub struct enum_locales_data {
    pub info: *mut RFC1766INFO,
    pub total: u32,
    pub allocated: u32,
}

pub static IMultiLanguage_vtbl: IMultiLanguageVtbl = IMultiLanguageVtbl {
    QueryInterface: fnIMultiLanguage_QueryInterface,
    AddRef: fnIMultiLanguage_AddRef,
    Release: fnIMultiLanguage_Release,
    GetNumberOfCodePageInfo: fnIMultiLanguage_GetNumberOfCodePageInfo,
    GetCodePageInfo: fnIMultiLanguage_GetCodePageInfo,
    GetFamilyCodePage: fnIMultiLanguage_GetFamilyCodePage,
    EnumCodePages: fnIMultiLanguage_EnumCodePages,
    GetCharsetInfo: fnIMultiLanguage_GetCharsetInfo,
    IsConvertible: fnIMultiLanguage_IsConvertible,
    ConvertString: fnIMultiLanguage_ConvertString,
    ConvertStringToUnicode: fnIMultiLanguage_ConvertStringToUnicode,
    ConvertStringFromUnicode: fnIMultiLanguage_ConvertStringFromUnicode,
    ConvertStringReset: fnIMultiLanguage_ConvertStringReset,
    GetRfc1766FromLcid: fnIMultiLanguage_GetRfc1766FromLcid,
    GetLcidFromRfc1766: fnIMultiLanguage_GetLcidFromRfc1766,
    EnumRfc1766: fnIMultiLanguage_EnumRfc1766,
    GetRfc1766Info: fnIMultiLanguage_GetRfc1766Info,
    CreateConvertCharset: fnIMultiLanguage_CreateConvertCharset,
};

pub static IMultiLanguage3_vtbl: IMultiLanguage3Vtbl = IMultiLanguage3Vtbl {
    QueryInterface: fnIMultiLanguage3_QueryInterface,
    AddRef: fnIMultiLanguage3_AddRef,
    Release: fnIMultiLanguage3_Release,
    GetNumberOfCodePageInfo: fnIMultiLanguage3_GetNumberOfCodePageInfo,
    GetCodePageInfo: fnIMultiLanguage3_GetCodePageInfo,
    GetFamilyCodePage: fnIMultiLanguage3_GetFamilyCodePage,
    EnumCodePages: fnIMultiLanguage3_EnumCodePages,
    GetCharsetInfo: fnIMultiLanguage3_GetCharsetInfo,
    IsConvertible: fnIMultiLanguage3_IsConvertible,
    ConvertString: fnIMultiLanguage3_ConvertString,
    ConvertStringToUnicode: fnIMultiLanguage3_ConvertStringToUnicode,
    ConvertStringFromUnicode: fnIMultiLanguage3_ConvertStringFromUnicode,
    ConvertStringReset: fnIMultiLanguage3_ConvertStringReset,
    GetRfc1766FromLcid: fnIMultiLanguage3_GetRfc1766FromLcid,
    GetLcidFromRfc1766: fnIMultiLanguage3_GetLcidFromRfc1766,
    EnumRfc1766: fnIMultiLanguage3_EnumRfc1766,
    GetRfc1766Info: fnIMultiLanguage3_GetRfc1766Info,
    CreateConvertCharset: fnIMultiLanguage3_CreateConvertCharset,
    ConvertStringInIStream: fnIMultiLanguage3_ConvertStringInIStream,
    ConvertStringToUnicodeEx: fnIMultiLanguage3_ConvertStringToUnicodeEx,
    ConvertStringFromUnicodeEx: fnIMultiLanguage3_ConvertStringFromUnicodeEx,
    DetectCodepageInIStream: fnIMultiLanguage3_DetectCodepageInIStream,
    DetectInputCodepage: fnIMultiLanguage3_DetectInputCodepage,
    ValidateCodePage: fnIMultiLanguage3_ValidateCodePage,
    GetCodePageDescription: fnIMultiLanguage3_GetCodePageDescription,
    IsCodePageInstallable: fnIMultiLanguage3_IsCodePageInstallable,
    SetMimeDBSource: fnIMultiLanguage3_SetMimeDBSource,
    GetNumberOfScripts: fnIMultiLanguage3_GetNumberOfScripts,
    EnumScripts: fnIMultiLanguage3_EnumScripts,
    ValidateCodePageEx: fnIMultiLanguage3_ValidateCodePageEx,
    DetectOutboundCodePage: fnIMultiLanguage3_DetectOutboundCodePage,
    DetectOutboundCodePageInIStream: fnIMultiLanguage3_DetectOutboundCodePageInIStream,
};

pub static IMLangFontLink2_vtbl: IMLangFontLink2Vtbl = IMLangFontLink2Vtbl {
    QueryInterface: fnIMLangFontLink2_QueryInterface,
    AddRef: fnIMLangFontLink2_AddRef,
    Release: fnIMLangFontLink2_Release,
    GetCharCodePages: fnIMLangFontLink2_GetCharCodePages,
    GetStrCodePages: fnIMLangFontLink2_GetStrCodePages,
    CodePageToCodePages: fnIMLangFontLink2_CodePageToCodePages,
    CodePagesToCodePage: fnIMLangFontLink2_CodePagesToCodePage,
    GetFontCodePages: fnIMLangFontLink2_GetFontCodePages,
    ReleaseFont: fnIMLangFontLink2_ReleaseFont,
    ResetFontMapping: fnIMLangFontLink2_ResetFontMapping,
    MapFont: fnIMLangFontLink2_MapFont,
    GetFontUnicodeRanges: fnIMLangFontLink2_GetFontUnicodeRanges,
    GetScriptFontInfo: fnIMLangFontLink2_GetScriptFontInfo,
    CodePageToScriptID: fnIMLangFontLink2_CodePageToScriptID,
};

pub static IMLangLineBreakConsole_vtbl: IMLangLineBreakConsoleVtbl = IMLangLineBreakConsoleVtbl {
    QueryInterface: fnIMLangLineBreakConsole_QueryInterface,
    AddRef: fnIMLangLineBreakConsole_AddRef,
    Release: fnIMLangLineBreakConsole_Release,
    BreakLineML: fnIMLangLineBreakConsole_BreakLineML,
    BreakLineW: fnIMLangLineBreakConsole_BreakLineW,
    BreakLineA: fnIMLangLineBreakConsole_BreakLineA,
};

#[repr(C)]
pub struct convert_charset {
    pub IMLangConvertCharset_iface: IMLangConvertCharset,
    pub ref_: i32,
    pub src_cp: u32,
    pub dst_cp: u32,
}

pub static MLangConvertCharsetVtbl: IMLangConvertCharsetVtbl = IMLangConvertCharsetVtbl {
    QueryInterface: MLangConvertCharset_QueryInterface,
    AddRef: MLangConvertCharset_AddRef,
    Release: MLangConvertCharset_Release,
    Initialize: MLangConvertCharset_Initialize,
    GetSourceCodePage: MLangConvertCharset_GetSourceCodePage,
    GetDestinationCodePage: MLangConvertCharset_GetDestinationCodePage,
    GetProperty: MLangConvertCharset_GetProperty,
    DoConversion: MLangConvertCharset_DoConversion,
    DoConversionToUnicode: MLangConvertCharset_DoConversionToUnicode,
    DoConversionFromUnicode: MLangConvertCharset_DoConversionFromUnicode,
};

// ---- consts / statics / type aliases (deduped) ----
const S_OK: HRESULT = 0;
const S_FALSE: HRESULT = 1;
const E_FAIL: HRESULT = 0x80004005u32 as HRESULT;
const E_INVALIDARG: HRESULT = 0x80070057u32 as HRESULT;
const E_OUTOFMEMORY: HRESULT = 0x8007000Eu32 as HRESULT;
const E_NOINTERFACE: HRESULT = 0x80004002u32 as HRESULT;
const E_NOTIMPL: HRESULT = 0x80004001u32 as HRESULT;
const CLASS_E_CLASSNOTAVAILABLE: HRESULT = 0x80040111u32 as HRESULT;
const CLASS_E_NOAGGREGATION: HRESULT = 0x80040110u32 as HRESULT;
const MAX_RFC1766_NAME: usize = 32;
const MAX_LOCALE_NAME: usize = 85;
const MAX_MIMEFACE_NAME: u32 = 32;
const MAX_MIMECSET_NAME: usize = 64;
const MAX_SCRIPT_NAME: u32 = 50;
const LOCALE_SISO639LANGNAME: u32 = 0x59;
const LOCALE_SISO3166CTRYNAME: u32 = 0x5A;
const LOCALE_USER_DEFAULT: u32 = 0x0400;
const LOCALE_SLANGUAGE: u32 = 0x00000002;
const LOCALE_SENGLANGUAGE: u32 = 0x00001001;
const LCMAP_LOWERCASE: u32 = 0x00000100;
const CP_ACP: UINT = 0;
const CP_USASCII: u32 = 20127;
const CP_UTF7: u32 = 65000;
const CP_UTF8: u32 = 65001;
const CP_UNICODE: u32 = 1200;
const SCRIPTCONTF_SCRIPT_USER: u32 = 1;
const SCRIPTCONTF_SCRIPT_HIDE: u32 = 2;
const SCRIPTCONTF_SCRIPT_SYSTEM: u32 = 4;
const SCRIPTCONTF_FIXED_FONT: DWORD = 1;
const SCRIPTCONTF_PROPORTIONAL_FONT: DWORD = 2;
const TCI_SRCCODEPAGE: u32 = 2; // wingdi.h:962 (was 1 = TCI_SRCCHARSET)
const TCI_SRCFONTSIG: u32 = 3;
const DEFAULT_CHARSET: u8 = 1;
const ERROR_SUCCESS: LSTATUS = 0;
const REG_SZ: u32 = 1;
const REG_DWORD: u32 = 4;
const HKEY_CLASSES_ROOT: HKEY = 0x80000000u32 as HKEY;
const OBJ_FONT: u32 = 6; // wingdi.h:694 (was 7 = OBJ_BITMAP)
const CPIOD_PEEK: u32 = 0x00000400;
const WC_NO_BEST_FIT_CHARS: u32 = 0x00000400;
const STATFLAG_NONAME: u32 = 1;
const MAXLONG: i64 = 0x7FFFFFFF;

// Script IDs (4-byte tags)
const sidAsciiLatin: SCRIPT_ID = [0x00, 0x00, 0x00, 0x00];
const sidArabic: SCRIPT_ID = [0x01, 0x00, 0x00, 0x00];
const sidHan: SCRIPT_ID = [0x12, 0x00, 0x00, 0x00];
const sidBopomofo: SCRIPT_ID = [0x13, 0x00, 0x00, 0x00];
const sidCyrillic: SCRIPT_ID = [0x04, 0x00, 0x00, 0x00];
const sidGreek: SCRIPT_ID = [0x05, 0x00, 0x00, 0x00];
const sidHebrew: SCRIPT_ID = [0x06, 0x00, 0x00, 0x00];
const sidKana: SCRIPT_ID = [0x15, 0x00, 0x00, 0x00];
const sidHangul: SCRIPT_ID = [0x16, 0x00, 0x00, 0x00];
const sidThai: SCRIPT_ID = [0x17, 0x00, 0x00, 0x00];

// CLSIDs
const CLSID_CMultiLanguage: CLSID = GUID { Data1: 0x275c23e2, Data2: 0x3747, Data3: 0x11d0, Data4: [0x9f, 0xea, 0x00, 0xaa, 0x00, 0x3f, 0x86, 0x46] };
const CLSID_CMLangConvertCharset: CLSID = GUID { Data1: 0xd66d6f98, Data2: 0x80cd, Data3: 0x11d0, Data4: [0x9f, 0xea, 0x00, 0xaa, 0x00, 0x3f, 0x86, 0x46] };

// extra missing types
pub type BSTR = *mut u16;
pub type LCTYPE = DWORD;
pub type PMIMECSETINFO = *mut MIMECSETINFO;
#[repr(C)]
pub struct STATSTG {
    pub pwcsName: *mut u16,
    pub kind: u32,
    pub cbSize: u64,
    pub mtime: u64,
    pub ctime: u64,
    pub atime: u64,
    pub grfMode: u32,
    pub grfLocksSupported: u32,
    pub clsid: CLSID,
    pub grfStateBits: u32,
    pub reserved: u32,
}
#[repr(C)]
pub struct MIMECSETINFO {
    pub uiCodePage: u32,
    pub uiInternetEncoding: u32,
    pub wszCharset: [u16; 50],
}
#[repr(C)]
pub struct list {
    pub Flink: *mut list,
    pub Blink: *mut list,
}

// File-scope statics
pub static mut instance: HINSTANCE = 0 as *mut c_void;
pub static mut MLANG_tls_index: DWORD = 0;


// IID statics
pub static IID_IUnknown: GUID = GUID { Data1: 0, Data2: 0, Data3: 0, Data4: [0; 8] };
pub static IID_IEnumCodePage: GUID = GUID { Data1: 0, Data2: 0, Data3: 0, Data4: [0; 8] };
pub static IID_IClassFactory: GUID = GUID { Data1: 0, Data2: 0, Data3: 0, Data4: [0; 8] };
pub static IID_IMultiLanguage: GUID = GUID { Data1: 0, Data2: 0, Data3: 0, Data4: [0; 8] };
pub static IID_IMultiLanguage3: GUID = GUID { Data1: 0, Data2: 0, Data3: 0, Data4: [0; 8] };
pub static IID_IEnumScript: GUID = GUID { Data1: 0, Data2: 0, Data3: 0, Data4: [0; 8] };
pub static IID_IEnumRfc1766: GUID = GUID { Data1: 0, Data2: 0, Data3: 0, Data4: [0; 8] };
pub static IID_IMLangCodePages: GUID = GUID { Data1: 0, Data2: 0, Data3: 0, Data4: [0; 8] };
pub static IID_IMLangFontLink: GUID = GUID { Data1: 0, Data2: 0, Data3: 0, Data4: [0; 8] };
pub static IID_IMLangFontLink2: GUID = GUID { Data1: 0, Data2: 0, Data3: 0, Data4: [0; 8] };
pub static IID_IMultiLanguage2: GUID = GUID { Data1: 0, Data2: 0, Data3: 0, Data4: [0; 8] };
pub static IID_IMLangLineBreakConsole: GUID = GUID { Data1: 0, Data2: 0, Data3: 0, Data4: [0; 8] };
pub static IID_IMLangConvertCharset: GUID = GUID { Data1: 0, Data2: 0, Data3: 0, Data4: [0; 8] };
extern "system" {
    fn DisableThreadLibraryCalls(hLibModule: HINSTANCE) -> BOOL;
    fn TlsAlloc() -> DWORD;
    fn TlsFree(dwTlsIndex: DWORD) -> BOOL;
    fn GetLocaleInfoA(lcid: u32, lctype: u32, lcdata: *mut i8, cchdata: i32) -> i32;
    fn LCMapStringA(locale: u32, dwmapflags: u32, lpsrc: *const i8, cchsrc: i32, lpdest: *mut i8, cchdest: i32) -> i32;
    fn GetLocaleInfoW(locale: u32, lctype: u32, lplcdata: *mut u16, cchdata: i32) -> i32;
    fn LCMapStringW(locale: u32, dwmapflags: u32, lpsrcstr: *const u16, cchsrc: i32, lpdeststr: *mut u16, cchdest: i32) -> i32;
    fn lstrlenW(s: *const u16) -> i32;
    fn EnterCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn LeaveCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn TranslateCharsetInfo(lpCs: *mut u32, lpCsInfo: *mut CHARSETINFO, dwFlags: u32) -> i32;
    fn GetObjectW(h: HFONT, c: i32, pv: *mut core::ffi::c_void) -> i32;
    fn CreateFontIndirectW(lplf: *const LOGFONTW) -> HFONT;
    fn SelectObject(hdc: HDC, h: HFONT) -> HFONT;
    fn GetTextCharset(hdc: HDC) -> i32;
    fn GetProcessHeap() -> HANDLE;
    fn HeapAlloc(hHeap: HANDLE, dwFlags: u32, dwBytes: usize) -> *mut core::ffi::c_void;
    fn list_add_tail(head: *mut LIST_ENTRY, item: *mut LIST_ENTRY);
    fn DeleteObject(hObject: *mut core::ffi::c_void) -> i32;
    fn HeapFree(hHeap: *mut core::ffi::c_void, dwFlags: u32, lpMem: *mut core::ffi::c_void) -> i32;
    fn list_remove(entry: *mut list);
    fn IsEqualGUID(rguid1: *const GUID, rguid2: *const GUID) -> u8;
    fn InterlockedIncrement(Addend: *mut i32) -> i32;
    fn InterlockedDecrement(Addend: *mut i32) -> i32;
    fn debugstr_guid(guid: *const GUID) -> *const core::ffi::c_char;
    fn MultiByteToWideChar( CodePage: u32, dwFlags: u32, lpMultiByteStr: *const i8, cbMultiByte: i32, lpWideCharStr: *mut u16, cchWideChar: i32, ) -> i32;
    fn TlsSetValue(dwTlsIndex: u32, lpTlsValue: *mut core::ffi::c_void) -> i32;
    fn IsValidCodePage(codePage: u32) -> BOOL;
    fn EnumSystemLocalesW(lpLocaleEnumProc: Option<unsafe extern "C" fn(*const u16) -> i32>, dwFlags: u32) -> i32;
    fn GetFontUnicodeRanges(hdc: HDC, lpgs: *mut GLYPHSET) -> u32;
    fn RegCreateKeyW(hKey: HKEY, lpSubKey: *const u16, phkResult: *mut HKEY) -> LSTATUS;
    fn RegSetValueExA(hKey: HKEY, lpValueName: *const i8, Reserved: u32, dwType: u32, lpData: *const u8, cbData: u32) -> LSTATUS;
    fn RegSetValueExW(hKey: HKEY, lpValueName: *const u16, Reserved: u32, dwType: u32, lpData: *const u8, cbData: u32) -> LSTATUS;
    fn RegCloseKey(hKey: HKEY) -> LSTATUS;
    fn TlsGetValue(dwTlsIndex: u32) -> *mut core::ffi::c_void;
    fn wine_dbgstr_w(s: *const u16) -> *const i8;
    fn HeapReAlloc(hHeap: *mut core::ffi::c_void, dwFlags: u32, lpMem: *mut core::ffi::c_void, dwBytes: usize) -> *mut core::ffi::c_void;
    fn SysAllocString(psz: *const u16) -> *mut u16;
    fn GetCurrentObject(hdc: *mut c_void, uObjectType: u32) -> *mut c_void;

    fn lstrcmpiW(lpString1: *const u16, lpString2: *const u16) -> i32;
    fn lstrcpyW(lpString1: *mut u16, lpString2: *const u16) -> *mut u16;
    fn WideCharToMultiByte( CodePage: u32, dwFlags: u32, lpWideCharStr: *const u16, cchWideChar: i32, lpMultiByteStr: *mut i8, cbMultiByte: i32, lpDefaultChar: *const i8, lpUsedDefaultChar: *mut i32, ) -> i32;
    fn GetTextCharsetInfo(hdc: HDC, lpFontSig: *mut FONTSIGNATURE, dwFlags: u32) -> u32;
    fn IsEqualIID(guid1: *const IID, guid2: *const IID) -> i32;
    fn IStream_Stat( this: *mut IStream, pstatstg: *mut STATSTG, grfStatFlag: u32, ) -> i32;
    fn IStream_Read( this: *mut IStream, pv: *mut core::ffi::c_void, cb: u32, pcbRead: *mut u32, ) -> i32;
    fn IStream_Write( this: *mut IStream, pv: *const core::ffi::c_void, cb: u32, pcbWritten: *mut u32, ) -> i32;
    fn lstrlenA(lpString: *const i8) -> i32;
}
unsafe extern "C" {
    fn memcmp(s1: *const core::ffi::c_void, s2: *const core::ffi::c_void, n: usize) -> i32;
    fn __wine_unregister_resources(hInstance: HINSTANCE) -> HRESULT;
    fn debugstr_an(s: *const i8, n: i32) -> *const i8;
    fn debugstr_w(s: *const u16) -> *const i8;
    fn debugstr_wn(s: *const u16, len: i32) -> *const i8;
    fn __wine_register_resources(hinstance: HINSTANCE) -> HRESULT;
    fn debugstr_a(s: *const i8) -> *const i8;
}
// C min() is a macro, not an export — local wrapper (cdrom-rust pattern).
#[inline] fn min(a: LONG, b: LONG) -> LONG { core::cmp::min(a, b) }

extern "C" {
    fn swprintf(buffer: *mut u16, format: *const u16, ...) -> i32;
    fn FIXME(format: *const core::ffi::c_char, ...);
}

unsafe extern "C" fn DetectJapaneseCode(input: *const u8, count: u32) -> u32 {
    let mut code: u32 = 0;
    let mut i: u32 = 0;

    while (code == 0 || code == 51932) && i < count {
        let c1 = *input.add(i as usize);
        if c1 == 0x1b {
            i += 1;
            if i >= count {
                return code;
            }
            let c1 = *input.add(i as usize);
            if c1 == b'$' {
                i += 1;
                if i >= count {
                    return code;
                }
                let c1 = *input.add(i as usize);
                if c1 == b'B' || c1 == b'@' {
                    code = 50220;
                }
            }
            if c1 == b'K' {
                code = 50220;
            }
        } else if c1 >= 129 {
            i += 1;
            if i >= count {
                return code;
            }
            let c2 = *input.add(i as usize);
            if (c1 >= 129 && c1 <= 159) && (c2 >= 64 && c2 <= 160) {
                code = 932;
            } else if (c1 >= 161 && c1 <= 254) && (c2 >= 161 && c2 <= 254) {
                code = 51932;
            } else if (c1 == 142) && (c2 >= 161 && c2 <= 223) {
                code = 51932;
            }
        }
        i += 1;
    }
    return code;
}

unsafe extern "C" fn jis2sjis(p1: *mut u8, p2: *mut u8) {
    let c1 = *p1;
    let c2 = *p2;
    let row = if c1 < 95 { 112i32 } else { 176i32 };
    let cell = if c1 % 2 != 0 { 31 + if c2 > 95 { 1 } else { 0 } } else { 126 };
    *p1 = (((c1 as i32 + 1) >> 1) + row) as u8;
    *p2 = (c2 as i32 + cell) as u8;
}

unsafe extern "C" fn sjis2jis(p1: *mut u8, p2: *mut u8) {
    unsafe {
        let c1: u8 = *p1;
        let c2: u8 = *p2;
        let shift: i32 = if c2 < 159 { 1 } else { 0 };
        let row: i32 = if c1 < 160 { 112 } else { 176 };
        let cell: i32 = if shift != 0 {
            31 + (if c2 > 127 { 1 } else { 0 })
        } else {
            126
        };

        *p1 = (((c1 as i32) - row) << 1 - shift) as u8;
        *p2 = ((c2 as i32) - cell) as u8;
    }
}

unsafe extern "C" fn han2zen(p1: *mut u8, p2: *mut u8) -> i32 {
    let mut maru: i32 = 0;
    let mut nigori: i32 = 0;
    const CHAR1: [u8; 62] = [
        129, 129, 129, 129, 129, 131, 131, 131, 131, 131, 131, 131, 131, 131, 131, 129, 131, 131,
        131, 131, 131, 131, 131, 131, 131, 131, 131, 131, 131, 131, 131, 131, 131, 131, 131, 131,
        131, 131, 131, 131, 131, 131, 131, 131, 131, 131, 131, 131, 131, 131, 131, 131, 131, 131,
        131, 131, 131, 131, 131, 131, 129, 129,
    ];
    const CHAR2: [u8; 63] = [
        66, 117, 118, 65, 69, 146, 64, 66, 68, 70, 72, 131, 133, 135, 98, 91, 65, 67, 69, 71, 73,
        74, 76, 78, 80, 82, 84, 86, 88, 90, 92, 94, 96, 99, 101, 103, 105, 106, 107, 108, 109, 110,
        113, 116, 119, 122, 125, 126, 128, 129, 130, 132, 134, 136, 137, 138, 139, 140, 141, 143,
        147, 74, 75,
    ];

    if *p2 == 222
        && ((*p1 >= 182 && *p1 <= 196) || (*p1 >= 202 && *p1 <= 206) || (*p1 == 179))
    {
        nigori = 1;
    } else if *p2 == 223 && (*p1 >= 202 && *p1 <= 206) {
        maru = 1;
    }

    if *p1 >= 161 && *p1 <= 223 {
        let index: u8 = *p1 - 161;
        *p1 = CHAR1[index as usize];
        *p2 = CHAR2[index as usize];
    }

    if maru != 0 || nigori != 0 {
        if nigori != 0 {
            if (*p2 >= 74 && *p2 <= 103) || (*p2 >= 110 && *p2 <= 122) {
                *p2 += 1;
            } else if *p1 == 131 && *p2 == 69 {
                *p2 = 148;
            }
        } else if maru != 0 && (*p2 >= 110 && *p2 <= 122) {
            *p2 += 2;
        }
        return 1;
    }

    0
}

unsafe extern "C" fn exit_shift(out: *mut i8, c: i32) -> i32 {
    if !out.is_null() {
        *out.add(c as usize) = 0x1b;
        *out.add((c + 1) as usize) = b'(' as i8;
        *out.add((c + 2) as usize) = b'B' as i8;
    }
    3
}

unsafe extern "C" fn enter_shift(out: LPSTR, c: i32) -> i32 {
    if !out.is_null() {
        *out.add(c as usize) = 0x1b as i8;
        *out.add(c as usize + 1) = b'$' as i8;
        *out.add(c as usize + 2) = b'B' as i8;
    }
    3
}

unsafe extern "C" fn LockModule() {
    InterlockedIncrement(core::ptr::addr_of_mut!(dll_count));
}

unsafe extern "C" fn UnlockModule() {
    InterlockedDecrement(core::ptr::addr_of_mut!(dll_count));
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hInstDLL: HINSTANCE, fdwReason: DWORD, lpv: LPVOID) -> BOOL {
    unsafe {
        match fdwReason {
            DLL_PROCESS_ATTACH => {
                instance = hInstDLL;
                MLANG_tls_index = TlsAlloc();
                DisableThreadLibraryCalls(hInstDLL);
            }
            DLL_PROCESS_DETACH => {
                if lpv.is_null() {
                    TlsFree(MLANG_tls_index);
                }
            }
            _ => {}
        }
        return 1;
    }
}

unsafe extern "C" fn GetFamilyCodePage(uiCodePage: u32, puiFamilyCodePage: *mut u32) -> HRESULT {
    TRACE!(b"%u %p\n\0".as_ptr() as *const i8, uiCodePage, puiFamilyCodePage);

    if puiFamilyCodePage.is_null() {
        return S_FALSE;
    }

    for i in 0..mlang_data.len() {
        let data_ref = &*mlang_data.as_ptr().add(i);
        for n in 0..(data_ref.number_of_cp as usize) {
            if (*data_ref.mime_cp_info.add(n)).cp == uiCodePage {
                *puiFamilyCodePage = data_ref.family_codepage;
                return S_OK;
            }
        }
    }

    return S_FALSE;
}

unsafe extern "C" fn lcid_to_rfc1766A(lcid: u32, rfc1766: *mut i8, len: i32) -> i32 {
    let mut buffer = [0i8; MAX_RFC1766_NAME];
    let n = GetLocaleInfoA(lcid, LOCALE_SISO639LANGNAME, buffer.as_mut_ptr(), MAX_RFC1766_NAME as i32);
    let mut i: i32;

    if n != 0 {
        let primary_lang = (lcid & 0x3FF) as u32;         // PRIMARYLANGID
        let sub_lang = ((lcid >> 10) & 0x3F) as u32;      // SUBLANGID
        i = primary_lang as i32;
        if ((primary_lang == LANG_ENGLISH || primary_lang == LANG_CHINESE || primary_lang == LANG_ARABIC) &&
            sub_lang == SUBLANG_DEFAULT) ||
            sub_lang > SUBLANG_DEFAULT
        {
            *buffer.as_mut_ptr().offset((n - 1) as isize) = '-' as i8;
            let i2 = GetLocaleInfoA(lcid, LOCALE_SISO3166CTRYNAME, buffer.as_mut_ptr().offset(n as isize), MAX_RFC1766_NAME as i32 - n);
            if i2 == 0 {
                *buffer.as_mut_ptr().offset((n - 1) as isize) = 0;
            }
            i = i2;
        } else {
            i = 0;
        }
        LCMapStringA(LOCALE_USER_DEFAULT, LCMAP_LOWERCASE, buffer.as_ptr(), n + i, rfc1766, len);
        if (n + i) > len {
            return E_INVALIDARG;
        } else {
            return S_OK;
        }
    }
    return E_FAIL;
}

unsafe extern "C" fn lcid_to_rfc1766W(lcid: u32, rfc1766: *mut u16, len: i32) -> i32 {
    let mut buffer: [u16; MAX_RFC1766_NAME] = [0; MAX_RFC1766_NAME];
    let n = GetLocaleInfoW(lcid, LOCALE_SISO639LANGNAME, buffer.as_mut_ptr(), MAX_RFC1766_NAME as i32);
    let mut i: i32 = 0;
    if n != 0 {
        i = (lcid & 0x3ff) as i32; // PRIMARYLANGID(lcid)
        if (((i == LANG_ENGLISH as i32) || (i == LANG_CHINESE as i32) || (i == LANG_ARABIC as i32)) &&
            ((lcid >> 10) as i32 == SUBLANG_DEFAULT as i32)) ||
            ((lcid >> 10) as i32 > SUBLANG_DEFAULT as i32) {
            buffer[(n - 1) as usize] = '-' as u16;
            i = GetLocaleInfoW(lcid, LOCALE_SISO3166CTRYNAME, buffer.as_mut_ptr().add(n as usize), MAX_RFC1766_NAME as i32 - n);
            if i == 0 {
                buffer[(n - 1) as usize] = '\0' as u16;
            }
        } else {
            i = 0;
        }
        let total_len = n + i;
        LCMapStringW(LOCALE_USER_DEFAULT, LCMAP_LOWERCASE, buffer.as_ptr(), total_len, rfc1766, len);
        if total_len > len {
            return E_INVALIDARG;
        } else {
            return S_OK;
        }
    }
    return E_FAIL;
}

unsafe extern "C" fn lcid_from_rfc1766(iface: *mut IEnumRfc1766, lcid: *mut u32, rfc1766: *const u16) -> i32 {
    let mut info: RFC1766INFO = core::mem::zeroed();
    let mut num: u32 = 0;

    while IEnumRfc1766_Next(iface, 1, &mut info, &mut num) == 0 {
        if wcsicmp(core::ptr::addr_of!(info.wszRfc1766) as *const u16, rfc1766) == 0 {
            *lcid = info.lcid;
            return 0;
        }
        if lstrlenW(rfc1766) == 2 && memcmp(core::ptr::addr_of!(info.wszRfc1766) as *const core::ffi::c_void, rfc1766 as *const core::ffi::c_void, 2 * core::mem::size_of::<u16>()) == 0 {
            *lcid = info.lcid & 0x3ff;
            return 0;
        }
    }

    return E_FAIL;
}

unsafe extern "C" fn map_font(hdc: HDC, codepages: u32, src_font: HFONT, dst_font: *mut HFONT) -> HRESULT {
    if hdc.is_null() || src_font.is_null() {
        return E_FAIL;
    }

    let mut font_list_entry: *mut font_list = core::ptr::null_mut();
    let mut charset_info: CHARSETINFO = core::mem::zeroed();
    let mut new_font: HFONT = core::ptr::null_mut();
    let mut old_font: HFONT = core::ptr::null_mut();
    let mut font_attr: LOGFONTW = core::mem::zeroed();
    let mut found_cached: i32;
    let mut charset: i32;
    let mut ret: i32;

    for i in 0..32u32 {
        let mask: u32 = 1u32 << i;
        if codepages & mask != 0 {
            let mut Csb: [u32; 2] = [mask, 0x0];
            ret = TranslateCharsetInfo(Csb.as_mut_ptr(), &mut charset_info, TCI_SRCFONTSIG);
            if ret == 0 {
                continue;
            }

            /* use cached font if possible */
            found_cached = 0;
            EnterCriticalSection(core::ptr::addr_of_mut!(font_cache_critical));
            // Manually expand LIST_FOR_EACH_ENTRY macro
            {
                let head: *mut LIST_ENTRY = core::ptr::addr_of_mut!(font_cache);
                let mut entry_ptr: *mut LIST_ENTRY = (*head).Flink;
                while entry_ptr != head {
                    font_list_entry = (entry_ptr as *mut u8).offset(
                        -(core::mem::offset_of!(font_list, list_entry) as isize),
                    ) as *mut font_list;

                    if (*font_list_entry).charset == charset_info.ciCharset &&
                       (*font_list_entry).base_font == src_font
                    {
                        if !dst_font.is_null() {
                            *dst_font = (*font_list_entry).font;
                        }
                        found_cached = 1;
                    }
                    entry_ptr = (*entry_ptr).Flink;
                }
            }
            LeaveCriticalSection(core::ptr::addr_of_mut!(font_cache_critical));
            if found_cached != 0 {
                return S_OK;
            }

            GetObjectW(src_font, core::mem::size_of::<LOGFONTW>() as i32,
                       &mut font_attr as *mut _ as *mut core::ffi::c_void);
            font_attr.lfCharSet = charset_info.ciCharset as u8;
            font_attr.lfWidth = 0;
            font_attr.lfFaceName[0] = 0u16;
            new_font = CreateFontIndirectW(&font_attr);
            if new_font.is_null() {
                continue;
            }

            old_font = SelectObject(hdc, new_font);
            charset = GetTextCharset(hdc);
            SelectObject(hdc, old_font);
            if charset as u32 == charset_info.ciCharset {
                font_list_entry = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<font_list>())
                    as *mut font_list;
                if font_list_entry.is_null() {
                    return E_OUTOFMEMORY;
                }

                (*font_list_entry).base_font = src_font;
                (*font_list_entry).font = new_font;
                (*font_list_entry).charset = charset as u32;

                EnterCriticalSection(core::ptr::addr_of_mut!(font_cache_critical));
                list_add_tail(core::ptr::addr_of_mut!(font_cache),
                              core::ptr::addr_of_mut!((*font_list_entry).list_entry));
                LeaveCriticalSection(core::ptr::addr_of_mut!(font_cache_critical));

                if !dst_font.is_null() {
                    *dst_font = new_font;
                }
                return S_OK;
            }
        }
    }

    E_FAIL
}

unsafe extern "C" fn release_font(font: HFONT) -> HRESULT {
    // Forward declarations for unknown functions
    extern "system" {
        fn EnterCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
        fn LeaveCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
        fn GetProcessHeap() -> *mut core::ffi::c_void;
        fn HeapFree(hHeap: *mut core::ffi::c_void, dwFlags: u32, lpMem: *mut core::ffi::c_void) -> i32;
        fn DeleteObject(ho: HGDIOBJ) -> i32;
        fn list_remove(entry: *mut list);
    }

    // Constants
    const E_FAIL: HRESULT = 0x80004005u32 as i32;
    const S_OK: HRESULT = 0;

    let mut hr: HRESULT = E_FAIL;
    let font_cache_ptr = core::ptr::addr_of_mut!(font_cache);
    EnterCriticalSection(core::ptr::addr_of_mut!(font_cache_critical));

    let mut entry: *mut list = (*font_cache_ptr).Flink as *mut list;
    while entry as *mut core::ffi::c_void != font_cache_ptr as *mut core::ffi::c_void {
        let font_list_entry: *mut font_list = (entry as usize - core::mem::offset_of!(font_list, list_entry)) as *mut font_list;
        if (*font_list_entry).font == font {
            list_remove(core::ptr::addr_of_mut!((*font_list_entry).list_entry) as *mut list);
            DeleteObject(font as HGDIOBJ);
            HeapFree(GetProcessHeap(), 0, font_list_entry as *mut core::ffi::c_void);
            hr = S_OK;
            break;
        }
        entry = (*entry).Flink;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(font_cache_critical));

    return hr;
}

unsafe extern "C" fn clear_font_cache() -> HRESULT {
    EnterCriticalSection(&raw const font_cache_critical as *const CRITICAL_SECTION as *mut CRITICAL_SECTION);

    let head: *mut LIST_ENTRY = &raw const font_cache as *const LIST_ENTRY as *mut LIST_ENTRY;
    let mut entry: *mut LIST_ENTRY = (*head).Flink;
    let mut next: *mut LIST_ENTRY = if entry != head {
        (*entry).Flink
    } else {
        core::ptr::null_mut()
    };

    while entry != head {
        let font_list_entry: *mut font_list = entry as *mut font_list;
        let font_list_entry2: *mut font_list = next as *mut font_list;

        list_remove(&raw mut (*font_list_entry).list_entry as *mut list);
        DeleteObject((*font_list_entry).font as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, font_list_entry as *mut core::ffi::c_void);

        entry = next;
        if entry != head {
            next = (*entry).Flink;
        }
    }

    LeaveCriticalSection(&raw const font_cache_critical as *const CRITICAL_SECTION as *mut CRITICAL_SECTION);

    return 0i32; // S_OK
}

#[inline]
unsafe extern "C" fn impl_from_IClassFactory(iface: *mut IClassFactory) -> *mut IClassFactoryImpl {
    (iface as *const u8).sub(core::mem::offset_of!(IClassFactoryImpl, IClassFactory_iface)) as *mut IClassFactoryImpl
}

#[no_mangle]
pub unsafe extern "C" fn MLANGCF_QueryInterface(
    iface: *mut IClassFactory,
    riid: *const GUID,
    ppobj: *mut *mut core::ffi::c_void,
) -> i32 {
    TRACE!(b"%s\n\0".as_ptr() as *const i8, debugstr_guid(riid));

    if IsEqualGUID(riid, &IID_IUnknown) != 0 || IsEqualGUID(riid, &IID_IClassFactory) != 0 {
        IClassFactory_AddRef(iface);
        *ppobj = iface as *mut core::ffi::c_void;
        return 0; // S_OK
    }

    *ppobj = core::ptr::null_mut();
    WARN!(b"(%p)->(%s,%p), not found\n\0".as_ptr() as *const i8, iface as *mut core::ffi::c_void, debugstr_guid(riid), ppobj as *mut core::ffi::c_void);
    return 0x80004002u32 as i32; // E_NOINTERFACE
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    iid: *const GUID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut i: usize = 0;
    let factory: *mut IClassFactoryImpl;

    TRACE!(b"%s %s %p\n\0".as_ptr() as *const core::ffi::c_char, debugstr_guid(rclsid), debugstr_guid(iid), ppv as *const core::ffi::c_void);

    if IsEqualGUID(core::ptr::addr_of!(IID_IClassFactory), iid) == 0
        && IsEqualGUID(core::ptr::addr_of!(IID_IUnknown), iid) == 0
    {
        return E_NOINTERFACE;
    }

    while i < object_creation.len() {
        if IsEqualGUID(object_creation[i].clsid, rclsid) != 0 {
            break;
        }
        i += 1;
    }

    if i == object_creation.len() {
        FIXME(
            b"%s: no class found.\n\0".as_ptr() as *const core::ffi::c_char,
            debugstr_guid(rclsid),
        );
        return CLASS_E_CLASSNOTAVAILABLE;
    }

    TRACE!(b"Creating a class factory for %s\n\0".as_ptr() as *const core::ffi::c_char, object_creation[i].szClassName);

    factory = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<IClassFactoryImpl>(),
    ) as *mut IClassFactoryImpl;

    if factory.is_null() {
        return E_OUTOFMEMORY;
    }

    (*factory).IClassFactory_iface.lpVtbl = &raw const MLANGCF_Vtbl;
    (*factory).ref_ = 1;
    (*factory).pfnCreateInstance = object_creation[i].pfnCreateInstance;

    *ppv = core::ptr::addr_of_mut!((*factory).IClassFactory_iface) as *mut core::ffi::c_void;

    TRACE!(b"(%p) <- %p\n\0".as_ptr() as *const core::ffi::c_char, ppv as *const core::ffi::c_void, core::ptr::addr_of!((*factory).IClassFactory_iface) as *const core::ffi::c_void);

    S_OK
}

unsafe extern "C" fn impl_from_IEnumCodePage(iface: *mut IEnumCodePage) -> *mut EnumCodePage_impl {
    let offset = core::mem::offset_of!(EnumCodePage_impl, IEnumCodePage_iface);
    (iface as *mut u8).sub(offset) as *mut EnumCodePage_impl
}

unsafe extern "C" fn impl_from_IEnumScript(iface: *mut IEnumScript) -> *mut EnumScript_impl {
    (iface as *mut u8)
        .sub(core::mem::offset_of!(EnumScript_impl, IEnumScript_iface))
        as *mut EnumScript_impl
}

unsafe extern "C" fn EnumScript_create(
    mlang: *mut MLang_impl,
    mut dwFlags: u32,
    LangId: LANGID,
    ppEnumScript: *mut *mut IEnumScript,
) -> HRESULT {
    TRACE!("%p, %08x, %04x, %p\n\0".as_ptr() as *const i8, mlang, dwFlags, LangId as u32, ppEnumScript);

    if dwFlags == 0 {
        dwFlags = SCRIPTCONTF_SCRIPT_USER | SCRIPTCONTF_SCRIPT_HIDE | SCRIPTCONTF_SCRIPT_SYSTEM;
    }

    let es_ptr = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<EnumScript_impl>(),
    ) as *mut EnumScript_impl;

    (*es_ptr).IEnumScript_iface.lpVtbl = core::ptr::addr_of!(IEnumScript_vtbl);
    (*es_ptr).ref_ = 1;
    (*es_ptr).pos = 0;
    (*es_ptr).total = (core::mem::size_of_val(&mlang_data) / core::mem::size_of::<mlang_data>()) as u32 - 1;
    (*es_ptr).script_info = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<SCRIPTINFO>() * (*es_ptr).total as usize,
    ) as *mut SCRIPTINFO;

    for i in 0..(*es_ptr).total {
        let elem = (*es_ptr).script_info.add(i as usize);
        (*elem).ScriptId = i.to_ne_bytes();
        (*elem).uiCodePage = mlang_data[i as usize].family_codepage;

        MultiByteToWideChar(
            CP_ACP,
            0,
            mlang_data[i as usize].description,
            -1,
            core::ptr::addr_of_mut!((*elem).wszDescription) as *mut u16,
            MAX_SCRIPT_NAME as i32,
        );
        MultiByteToWideChar(
            CP_ACP,
            0,
            mlang_data[i as usize].fixed_font,
            -1,
            core::ptr::addr_of_mut!((*elem).wszFixedWidthFont) as *mut u16,
            MAX_MIMEFACE_NAME as i32,
        );
        MultiByteToWideChar(
            CP_ACP,
            0,
            mlang_data[i as usize].proportional_font,
            -1,
            core::ptr::addr_of_mut!((*elem).wszProportionalFont) as *mut u16,
            MAX_MIMEFACE_NAME as i32,
        );
    }

    TRACE!("enumerated %d scripts with flags %08x\n\0".as_ptr() as *const i8, (*es_ptr).total, dwFlags);

    *ppEnumScript = core::ptr::addr_of_mut!((*es_ptr).IEnumScript_iface) as *mut IEnumScript;

    S_OK
}

#[inline]
unsafe extern "C" fn impl_from_IMLangFontLink(iface: *mut IMLangFontLink) -> *mut MLang_impl {
    (iface as *mut u8).offset(-(core::mem::offset_of!(MLang_impl, IMLangFontLink_iface) as isize)) as *mut MLang_impl
}

#[inline]
unsafe extern "C" fn impl_from_IMultiLanguage(iface: *mut IMultiLanguage) -> *mut MLang_impl {
    (iface as *mut u8).byte_offset(-(core::mem::offset_of!(MLang_impl, IMultiLanguage_iface) as isize)) as *mut MLang_impl
}

unsafe extern "C" fn impl_from_IEnumRfc1766(iface: *mut IEnumRfc1766) -> *mut EnumRfc1766_impl {
    (iface as *mut u8).sub(core::mem::offset_of!(EnumRfc1766_impl, IEnumRfc1766_iface)) as *mut EnumRfc1766_impl
}

unsafe extern "C" fn EnumRfc1766_create(LangId: u16, ppEnum: *mut *mut IEnumRfc1766) -> i32 {
    let rfc: *mut EnumRfc1766_impl;
    let mut data: enum_locales_data = core::mem::zeroed();

    TRACE!(b"%04x, %p\n\0".as_ptr() as *const i8, LangId as u32, ppEnum);

    rfc = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<EnumRfc1766_impl>()) as *mut EnumRfc1766_impl;
    (*rfc).IEnumRfc1766_iface.lpVtbl = &IEnumRfc1766_vtbl;
    (*rfc).ref_ = 1;
    (*rfc).pos = 0;
    (*rfc).total = 0;

    data.total = 0;
    data.allocated = 160;
    data.info = HeapAlloc(GetProcessHeap(), 0, data.allocated as usize * core::mem::size_of::<RFC1766INFO>()) as *mut RFC1766INFO;
    if data.info.is_null() {
        HeapFree(GetProcessHeap(), 0, rfc as *mut core::ffi::c_void);
        return E_OUTOFMEMORY;
    }

    TlsSetValue(MLANG_tls_index, core::ptr::addr_of_mut!(data) as *mut core::ffi::c_void);
    EnumSystemLocalesW(Some(enum_locales_proc), 0);
    TlsSetValue(MLANG_tls_index, core::ptr::null_mut());

    TRACE!(b"enumerated %d rfc1766 structures\n\0".as_ptr() as *const i8, data.total);

    if data.total == 0 {
        HeapFree(GetProcessHeap(), 0, data.info as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, rfc as *mut core::ffi::c_void);
        return E_FAIL;
    }

    (*rfc).info = data.info;
    (*rfc).total = data.total;

    *ppEnum = &(*rfc).IEnumRfc1766_iface as *const _ as *mut IEnumRfc1766;
    return S_OK;
}

unsafe extern "C" fn impl_from_IMultiLanguage3(iface: *mut IMultiLanguage3) -> *mut MLang_impl {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(MLang_impl, IMultiLanguage3_iface)) as *mut MLang_impl;
}

unsafe extern "C" fn fill_cp_info(ml_data: *const mlang_data, index: u32, mime_cp_info: *mut MIMECPINFO) {
    let mut csi: CHARSETINFO = core::mem::zeroed();

    if TranslateCharsetInfo(
        ((*ml_data).family_codepage as DWORD_PTR) as *mut u32,
        &raw mut csi,
        TCI_SRCCODEPAGE,
    ) != 0
    {
        (*mime_cp_info).bGDICharset = csi.ciCharset as u8;
    } else {
        (*mime_cp_info).bGDICharset = DEFAULT_CHARSET;
    }

    let src = &*(*ml_data).mime_cp_info.add(index as usize);

    (*mime_cp_info).dwFlags = src.flags;
    (*mime_cp_info).uiCodePage = src.cp;
    (*mime_cp_info).uiFamilyCodePage = (*ml_data).family_codepage;

    MultiByteToWideChar(
        CP_ACP,
        0,
        src.description,
        -1,
        (*mime_cp_info).wszDescription.as_mut_ptr(),
        (core::mem::size_of_val(&(*mime_cp_info).wszDescription)
            / core::mem::size_of_val(&(*mime_cp_info).wszDescription[0])) as i32,
    );
    MultiByteToWideChar(
        CP_ACP,
        0,
        src.web_charset,
        -1,
        (*mime_cp_info).wszWebCharset.as_mut_ptr(),
        (core::mem::size_of_val(&(*mime_cp_info).wszWebCharset)
            / core::mem::size_of_val(&(*mime_cp_info).wszWebCharset[0])) as i32,
    );
    MultiByteToWideChar(
        CP_ACP,
        0,
        src.header_charset,
        -1,
        (*mime_cp_info).wszHeaderCharset.as_mut_ptr(),
        (core::mem::size_of_val(&(*mime_cp_info).wszHeaderCharset)
            / core::mem::size_of_val(&(*mime_cp_info).wszHeaderCharset[0])) as i32,
    );
    MultiByteToWideChar(
        CP_ACP,
        0,
        src.body_charset,
        -1,
        (*mime_cp_info).wszBodyCharset.as_mut_ptr(),
        (core::mem::size_of_val(&(*mime_cp_info).wszBodyCharset)
            / core::mem::size_of_val(&(*mime_cp_info).wszBodyCharset[0])) as i32,
    );

    MultiByteToWideChar(
        CP_ACP,
        0,
        (*ml_data).fixed_font,
        -1,
        (*mime_cp_info).wszFixedWidthFont.as_mut_ptr(),
        (core::mem::size_of_val(&(*mime_cp_info).wszFixedWidthFont)
            / core::mem::size_of_val(&(*mime_cp_info).wszFixedWidthFont[0])) as i32,
    );
    MultiByteToWideChar(
        CP_ACP,
        0,
        (*ml_data).proportional_font,
        -1,
        (*mime_cp_info).wszProportionalFont.as_mut_ptr(),
        (core::mem::size_of_val(&(*mime_cp_info).wszProportionalFont)
            / core::mem::size_of_val(&(*mime_cp_info).wszProportionalFont[0])) as i32,
    );

    TRACE!(b"%08x %u %u %s %s %s %s %s %s %d\n\0".as_ptr() as *const i8, (*mime_cp_info).dwFlags, (*mime_cp_info).uiCodePage, (*mime_cp_info).uiFamilyCodePage, wine_dbgstr_w((*mime_cp_info).wszDescription.as_ptr()), wine_dbgstr_w((*mime_cp_info).wszWebCharset.as_ptr()), wine_dbgstr_w((*mime_cp_info).wszHeaderCharset.as_ptr()), wine_dbgstr_w((*mime_cp_info).wszBodyCharset.as_ptr()), wine_dbgstr_w((*mime_cp_info).wszFixedWidthFont.as_ptr()), wine_dbgstr_w((*mime_cp_info).wszProportionalFont.as_ptr()), (*mime_cp_info).bGDICharset as i32);
}

unsafe extern "C" fn fnIMultiLanguage3_ConvertStringReset(iface: *mut IMultiLanguage3) -> HRESULT {
    const E_NOTIMPL: HRESULT = 0x80004001u32 as i32;
    unsafe extern "C" {
        fn FIXME(format: *const i8, ...);
    }
    unsafe {
        FIXME(b"\n\0".as_ptr() as *const i8);
    }
    return E_NOTIMPL;
}

unsafe extern "C" fn fnIMultiLanguage3_DetectCodepageInIStream(
    iface: *mut IMultiLanguage3,
    dwFlag: u32,
    dwPrefWinCodePage: u32,
    pstmIn: *mut IStream,
    lpEncoding: *mut DetectEncodingInfo,
    pnScores: *mut i32,
) -> i32 {
    FIXME(b"\n\0".as_ptr() as *const i8);
    return E_NOTIMPL;
}

unsafe extern "C" fn fnIMultiLanguage3_DetectInputCodepage(
    iface: *mut IMultiLanguage3,
    dwFlag: DWORD,
    dwPrefWinCodePage: DWORD,
    pSrcStr: *mut i8,
    pcSrcSize: *mut i32,
    lpEncoding: *mut DetectEncodingInfo,
    pnScores: *mut i32,
) -> HRESULT {
    FIXME(b"\n\0".as_ptr().cast());
    return E_NOTIMPL;
}

unsafe extern "C" fn fnIMultiLanguage3_ValidateCodePage(
    iface: *mut IMultiLanguage3,
    uiCodePage: u32,
    hwnd: isize,
) -> i32 {
    IMultiLanguage3_ValidateCodePageEx(iface, uiCodePage, hwnd, 0)
}

unsafe extern "C" fn fnIMultiLanguage3_GetCodePageDescription(
    iface: *mut IMultiLanguage3,
    uiCodePage: u32,
    lcid: u32,
    lpWideCharStr: *mut u16,
    cchWideChar: i32,
) -> i32 {
    let mut i: u32;
    let mut n: u32;

    TRACE!(b"%u, %04x, %p, %d\n\0".as_ptr() as *const i8, uiCodePage, lcid, lpWideCharStr as *const core::ffi::c_void, cchWideChar);

    i = 0;
    while (i as usize) < mlang_data.len() {
        n = 0;
        while n < mlang_data[i as usize].number_of_cp {
            if (*mlang_data[i as usize].mime_cp_info.add(n as usize)).cp == uiCodePage {
                MultiByteToWideChar(
                    CP_ACP,
                    0,
                    (*mlang_data[i as usize].mime_cp_info.add(n as usize)).description,
                    -1,
                    lpWideCharStr,
                    cchWideChar,
                );
                return S_OK;
            }
            n = n + 1;
        }
        i = i + 1;
    }

    return S_FALSE;
}

unsafe extern "C" fn fnIMultiLanguage3_IsCodePageInstallable(iface: *mut IMultiLanguage3, uiCodePage: u32) -> HRESULT {
    TRACE!(b"%u\n\0".as_ptr() as *const i8, uiCodePage);
    // FIXME: the installable set is usually larger than the set of valid codepages
    return IMultiLanguage3_ValidateCodePageEx(iface, uiCodePage, 0isize, CPIOD_PEEK);
}

unsafe extern "C" fn fnIMultiLanguage3_SetMimeDBSource(
    iface: *mut IMultiLanguage3,
    dwSource: MIMECONTF,
) -> HRESULT {
    FIXME(
        b"0x%08x\n\0".as_ptr() as *const core::ffi::c_char,
        dwSource,
    );
    0 // S_OK
}

unsafe extern "C" fn impl_from_IMLangFontLink2(iface: *mut IMLangFontLink2) -> *mut MLang_impl {
    let offset = core::mem::offset_of!(MLang_impl, IMLangFontLink2_iface);
    (iface as *mut u8).sub(offset) as *mut MLang_impl
}

unsafe extern "C" fn fnIMLangFontLink2_GetFontUnicodeRanges(
    This: *mut IMLangFontLink2,
    hDC: HDC,
    puiRanges: *mut u32,
    pUranges: *mut UNICODERANGE,
) -> HRESULT {
    let mut size: u32;
    let mut gs: *mut GLYPHSET;

    TRACE!(b"(%p)->%p %p %p\n\0".as_ptr() as *const i8, This as *mut core::ffi::c_void, hDC as *mut core::ffi::c_void, puiRanges as *mut core::ffi::c_void, pUranges as *mut core::ffi::c_void);

    if puiRanges.is_null() {
        return E_INVALIDARG;
    }
    size = GetFontUnicodeRanges(hDC, core::ptr::null_mut());
    if size == 0 {
        return E_FAIL;
    }
    gs = HeapAlloc(GetProcessHeap(), 0, size as usize) as *mut GLYPHSET;
    if gs.is_null() {
        return E_OUTOFMEMORY;
    }

    GetFontUnicodeRanges(hDC, gs);
    *puiRanges = (*gs).cRanges;
    if !pUranges.is_null() {
        let mut i: u32 = 0;
        while i < (*gs).cRanges {
            if i >= *puiRanges {
                break;
            }
            let pUr = &mut *pUranges.offset(i as isize);
            // `ranges` is a C flexible array (`WCRANGE ranges[1]`, psdk/wingdi.h:2906)
            // holding cRanges entries; bindgen's [_; 1] panicked on the second.
            let r = &*(&raw const (*gs).ranges[0]).add(i as usize);
            pUr.wcFrom = r.wcLow;
            pUr.wcTo = r.wcLow + r.cGlyphs;
            i += 1;
        }
        *puiRanges = i;
    }
    HeapFree(GetProcessHeap(), 0, gs as *mut core::ffi::c_void);
    S_OK
}

unsafe extern "C" fn fnIMLangFontLink2_GetScriptFontInfo(
    This: *mut IMLangFontLink2,
    sid: SCRIPT_ID,
    mut dwFlags: DWORD,
    puiFonts: *mut UINT,
    pScriptFont: *mut SCRIPTFONTINFO,
) -> HRESULT {
    let mut i: UINT;
    let mut j: UINT;

    TRACE!(b"(%p)->%u %x %p %p\n\0".as_ptr() as *const i8, This, sid, dwFlags, puiFonts, pScriptFont);

    if dwFlags == 0 {
        dwFlags = SCRIPTCONTF_PROPORTIONAL_FONT;
    }

    i = 0;
    j = 0;
    while i < mlang_data.len() as UINT {
        if sid == mlang_data[i as usize].sid {
            if !pScriptFont.is_null() {
                if j >= *puiFonts {
                    break;
                }

                (*pScriptFont.add(j as usize)).scripts = (1i64 << u32::from_ne_bytes(mlang_data[i as usize].sid)); // SCRIPT_IDS cast

                if dwFlags == SCRIPTCONTF_FIXED_FONT {
                    MultiByteToWideChar(
                        CP_ACP,
                        0,
                        mlang_data[i as usize].fixed_font,
                        -1i32,
                        (*pScriptFont.add(j as usize)).wszFont.as_mut_ptr() as *mut u16,
                        MAX_MIMEFACE_NAME as i32,
                    );
                } else if dwFlags == SCRIPTCONTF_PROPORTIONAL_FONT {
                    MultiByteToWideChar(
                        CP_ACP,
                        0,
                        mlang_data[i as usize].proportional_font,
                        -1i32,
                        (*pScriptFont.add(j as usize)).wszFont.as_mut_ptr() as *mut u16,
                        MAX_MIMEFACE_NAME as i32,
                    );
                }
            }
            j += 1;
        }
        i += 1;
    }

    *puiFonts = j;
    return S_OK;
}

unsafe extern "C" fn fnIMLangFontLink2_CodePageToScriptID(
    This: *mut IMLangFontLink2,
    uiCodePage: u32,
    pSid: *mut u32,
) -> i32 {
    extern "C" {
    }
    unsafe {
        TRACE!(b"(%p)->%i %p\n\0".as_ptr() as *const i8, This, uiCodePage, pSid);
    }
    if uiCodePage == 1200 /* CP_UNICODE */ {
        return -2147467259i32; /* E_FAIL */
    }
    for i in 0..mlang_data.len() as u32 {
        if uiCodePage == mlang_data[i as usize].family_codepage {
            if !pSid.is_null() {
                unsafe {
                    *pSid = u32::from_ne_bytes(mlang_data[i as usize].sid);
                }
            }
            return 0; /* S_OK */
        }
    }
    return -2147467259i32; /* E_FAIL */
}

unsafe extern "C" fn impl_from_IMLangLineBreakConsole(iface: *mut IMLangLineBreakConsole) -> *mut MLang_impl {
    let offset = core::mem::offset_of!(MLang_impl, IMLangLineBreakConsole_iface) as isize;
    let base = (iface as *mut u8).offset(-offset);
    base as *mut MLang_impl
}

unsafe extern "C" fn fnIMLangLineBreakConsole_BreakLineML(
    iface: *mut IMLangLineBreakConsole,
    pSrcMLStr: *mut IMLangString,
    lSrcPos: i32,
    lSrcLen: i32,
    cMinColumns: i32,
    cMaxColumns: i32,
    plLineLen: *mut i32,
    plSkipLen: *mut i32,
) -> i32 {
    let fmt = b"(%p)->%p %i %i %i %i %p %p\n\0".as_ptr() as *const i8;
    FIXME(fmt, iface, pSrcMLStr, lSrcPos, lSrcLen, cMinColumns, cMaxColumns, plLineLen, plSkipLen);
    return E_NOTIMPL;
}

unsafe extern "C" fn fnIMLangLineBreakConsole_BreakLineW(
    iface: *mut IMLangLineBreakConsole,
    locale: LCID,
    pszSrc: *const u16,
    cchSrc: i32,
    cMaxColumns: i32,
    pcchLine: *mut i32,
    pcchSkip: *mut i32,
) -> i32 {
    extern "C" {
        fn debugstr_wn(pszSrc: *const u16, cchSrc: i32) -> *const i8;
    }

    unsafe {
        FIXME(
            b"(%p)->%i %s %i %i %p %p\n\0".as_ptr() as *const i8,
            iface,
            locale,
            debugstr_wn(pszSrc, cchSrc),
            cchSrc,
            cMaxColumns,
            pcchLine,
            pcchSkip,
        );

        *pcchLine = cchSrc;
        *pcchSkip = 0;
        0 // S_OK
    }
}

unsafe extern "C" fn fnIMLangLineBreakConsole_BreakLineA(
    iface: *mut IMLangLineBreakConsole,
    locale: u32,
    uCodePage: u32,
    pszSrc: *const i8,
    cchSrc: i32,
    cMaxColumns: i32,
    pcchLine: *mut i32,
    pcchSkip: *mut i32,
) -> i32 {
    let mut line: i32 = cchSrc;
    let mut skip: i32 = 0;
    FIXME(
        b"(%p)->%i %i %s %i %i %p %p\n\0".as_ptr() as *const i8,
        iface,
        locale,
        uCodePage,
        debugstr_an(pszSrc, cchSrc),
        cchSrc,
        cMaxColumns,
        pcchLine,
        pcchSkip,
    );
    if uCodePage == CP_USASCII && cchSrc > cMaxColumns {
        line = cMaxColumns;
        let mut i: i32 = cMaxColumns - 1;
        while i >= 0 {
            if *pszSrc.add(i as usize) == b' ' as i8 {
                while i >= 0 && *pszSrc.add(i as usize) == b' ' as i8 {
                    i -= 1;
                    line -= 1;
                    skip += 1;
                }
                break;
            }
            i -= 1;
        }
    }
    *pcchLine = line;
    *pcchSkip = skip;
    0 // S_OK
}

unsafe extern "C" fn impl_from_IMLangConvertCharset(iface: *mut IMLangConvertCharset) -> *mut convert_charset {
    let offset = core::mem::offset_of!(convert_charset, IMLangConvertCharset_iface);
    (iface as *mut u8).sub(offset) as *mut convert_charset
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    if dll_count == 0 { 0 /* S_OK */ } else { 1 /* S_FALSE */ }
}

unsafe extern "C" fn register_codepages() -> i32 {
    static db_key_nameW: [u16; 23] = [77,73,77,69, 92,68,97,116,97,98,97,115,101, 92,67,111,100,101,112,97,103,101, 0];
    static familyW: [u16; 7] = [70, 97, 109, 105, 108, 121, 0];
    static formatW: [u16; 3] = [37, 117, 0];

    let mut db_key: HKEY = core::ptr::null_mut();
    let mut key: HKEY = core::ptr::null_mut();
    let mut buf: [u16; 32] = [0u16; 32];

    let status = RegCreateKeyW(HKEY_CLASSES_ROOT, db_key_nameW.as_ptr(), &mut db_key);
    if status != ERROR_SUCCESS {
        return 0;
    }

    let family_start = mlang_data.as_ptr();
    let family_end = family_start.add(mlang_data.len());
    let mut family = family_start;

    while family < family_end {
        let info_start = (*family).mime_cp_info;
        let info_end = info_start.add((*family).number_of_cp as usize);
        let mut info = info_start;

        while info < info_end {
            swprintf(buf.as_mut_ptr(), formatW.as_ptr(), (*info).cp);
            let status = RegCreateKeyW(db_key, buf.as_ptr(), &mut key);
            if status != ERROR_SUCCESS {
                info = info.add(1);
                continue;
            }

            let body_charset_str = (*info).body_charset;
            let body_len = (strlen(body_charset_str) as u32) + 1;
            RegSetValueExA(key, b"BodyCharset\0".as_ptr() as *const i8, 0, REG_SZ, body_charset_str as *const u8, body_len);

            if (*info).cp == (*family).family_codepage {
                let fixed_font_str = (*family).fixed_font;
                let fixed_len = (strlen(fixed_font_str) as u32) + 1;
                RegSetValueExA(key, b"FixedWidthFont\0".as_ptr() as *const i8, 0, REG_SZ, fixed_font_str as *const u8, fixed_len);

                let proportional_font_str = (*family).proportional_font;
                let proportional_len = (strlen(proportional_font_str) as u32) + 1;
                RegSetValueExA(key, b"ProportionalFont\0".as_ptr() as *const i8, 0, REG_SZ, proportional_font_str as *const u8, proportional_len);
            } else {
                let family_cp = core::ptr::addr_of!((*family).family_codepage) as *const u8;
                RegSetValueExW(key, familyW.as_ptr(), 0, REG_DWORD, family_cp, core::mem::size_of::<u32>() as u32);
            }

            RegCloseKey(key);
            info = info.add(1);
        }

        family = family.add(1);
    }

    RegCloseKey(db_key);
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    unsafe {
        __wine_unregister_resources(instance)
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetGlobalFontLinkObject(
    unknown: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if unknown.is_null() {
        return E_INVALIDARG;
    }

    FIXME(
        b"%p: stub\n\0".as_ptr() as *const i8,
        unknown as *const core::ffi::c_void,
    );

    return S_FALSE;
}

unsafe extern "C" fn ConvertJIS2SJIS(input: LPCSTR, count: DWORD, output: LPSTR) -> UINT {
    const TRUE: i32 = 1;
    const FALSE: i32 = 0;

    let mut i: DWORD = 0;
    let mut j: i32 = 0;
    let mut p2: u8;
    let mut p: u8;
    let mut shifted: i32 = FALSE;

    let input = input as *const u8;
    let output = output as *mut u8;

    while i < count {
        p = *input.add(i as usize);
        if p == 0x1b {
            i += 1;
            if i >= count {
                return 0;
            }
            p2 = *input.add(i as usize);
            if p2 == b'$' || p2 == b'(' {
                i += 1;
            }
            if p2 == b'K' || p2 == b'$' {
                shifted = TRUE;
            } else {
                shifted = FALSE;
            }
        } else {
            if shifted != 0 {
                i += 1;
                if i >= count {
                    return 0;
                }
                p2 = *input.add(i as usize);
                jis2sjis(&raw mut p, &raw mut p2);
                *output.add(j as usize) = p;
                j += 1;
                *output.add(j as usize) = p2;
                j += 1;
            } else {
                *output.add(j as usize) = p;
                j += 1;
            }
        }
        i += 1;
    }
    j as UINT
}

unsafe extern "C" fn ConvertSJIS2JIS(input: *const i8, count: u32, output: *mut i8) -> u32 {
    let mut i: u32 = 0;
    let mut j: i32 = 0;
    let mut p2: u8 = 0;
    let mut p: u8 = 0;
    let mut shifted: i32 = 0; /* FALSE */

    while i < count {
        p = (*input.add(i as usize)) as u8 & 0xff;
        if p == 10 || p == 13 { /* NL and CR */
            if shifted != 0 {
                shifted = 0;
                j += exit_shift(output, j);
            }
            if !output.is_null() {
                *output.add(j as usize) = p as i8;
                j += 1;
            } else {
                j += 1;
            }
        } else {
            if (p >= 129 && p <= 159) || (p >= 224 && p <= 239) { /* SJIS1 */
                i += 1;
                if i >= count {
                    return 0;
                }
                p2 = (*input.add(i as usize)) as u8 & 0xff;
                if (p2 >= 64) && (p2 <= 252) { /* SJIS2 */
                    sjis2jis((&mut p) as *mut u8, (&mut p2) as *mut u8);
                    if shifted == 0 {
                        shifted = 1;
                        j += enter_shift(output, j);
                    }
                }

                if !output.is_null() {
                    *output.add(j as usize) = p as i8;
                    j += 1;
                    *output.add(j as usize) = p2 as i8;
                    j += 1;
                } else {
                    j += 2;
                }
            } else {
                if (p >= 161) && (p <= 223) { /* HANKATA */
                    if i + 1 >= count {
                        return 0;
                    }
                    p2 = (*input.add(i as usize + 1)) as u8 & 0xff;
                    i = (i as i32 + han2zen((&mut p) as *mut u8, (&mut p2) as *mut u8)) as u32;
                    sjis2jis((&mut p) as *mut u8, (&mut p2) as *mut u8);
                    if shifted == 0 {
                        shifted = 1;
                        j += enter_shift(output, j);
                    }
                    if !output.is_null() {
                        *output.add(j as usize) = p as i8;
                        j += 1;
                        *output.add(j as usize) = p2 as i8;
                        j += 1;
                    } else {
                        j += 2;
                    }
                } else {
                    if shifted != 0 {
                        shifted = 0;
                        j += exit_shift(output, j);
                    }
                    if !output.is_null() {
                        *output.add(j as usize) = p as i8;
                        j += 1;
                    } else {
                        j += 1;
                    }
                }
            }
        }
        i += 1;
    }
    if shifted != 0 {
        j += exit_shift(output, j);
    }
    j as u32
}

unsafe extern "C" fn MultiLanguage_create(pUnkOuter: *mut IUnknown, ppObj: *mut *mut core::ffi::c_void) -> HRESULT {
    TRACE!(b"Creating MultiLanguage object\n\0".as_ptr().cast());
    if !pUnkOuter.is_null() {
        return CLASS_E_NOAGGREGATION;
    }
    let mlang = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<MLang_impl>()) as *mut MLang_impl;
    (*mlang).IMLangFontLink_iface.lpVtbl = core::ptr::addr_of!(IMLangFontLink_vtbl);
    (*mlang).IMultiLanguage_iface.lpVtbl = core::ptr::addr_of!(IMultiLanguage_vtbl);
    (*mlang).IMultiLanguage3_iface.lpVtbl = core::ptr::addr_of!(IMultiLanguage3_vtbl);
    (*mlang).IMLangFontLink2_iface.lpVtbl = core::ptr::addr_of!(IMLangFontLink2_vtbl);
    (*mlang).IMLangLineBreakConsole_iface.lpVtbl = core::ptr::addr_of!(IMLangLineBreakConsole_vtbl);

    (*mlang).total_cp = 0;
    let mut i: u32 = 0;
    while i < mlang_data.len() as u32 {
        (*mlang).total_cp += mlang_data[i as usize].number_of_cp;
        i += 1;
    }

    (*mlang).total_scripts = (mlang_data.len() - 1) as u32;

    (*mlang).ref_ = 1;
    *ppObj = core::ptr::addr_of!((*mlang).IMultiLanguage_iface) as *mut core::ffi::c_void;
    TRACE!(b"returning %p\n\0".as_ptr().cast(), mlang as *mut core::ffi::c_void);

    LockModule();

    return S_OK;
}

unsafe extern "C" fn MLangConvertCharset_create(outer: *mut IUnknown, obj: *mut *mut core::ffi::c_void) -> i32 {
    let convert: *mut convert_charset;

    if !outer.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    *obj = core::ptr::null_mut();

    convert = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<convert_charset>()) as *mut convert_charset;
    if convert.is_null() {
        return E_OUTOFMEMORY;
    }

    (*convert).IMLangConvertCharset_iface.lpVtbl = core::ptr::addr_of!(MLangConvertCharsetVtbl);
    (*convert).ref_ = 1;

    *obj = core::ptr::addr_of_mut!((*convert).IMLangConvertCharset_iface) as *mut core::ffi::c_void;

    LockModule();

    return S_OK;
}

unsafe extern "C" fn MLANGCF_LockServer(iface: *mut IClassFactory, dolock: i32) -> i32 {
    if dolock != 0 {
        LockModule();
    } else {
        UnlockModule();
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn IsConvertINetStringAvailable(
    dwSrcEncoding: u32,
    dwDstEncoding: u32,
) -> i32 {
    let mut src_family: u32 = 0;
    let mut dst_family: u32 = 0;

    TRACE!(b"%d %d\n\0".as_ptr() as *const i8, dwSrcEncoding as i32, dwDstEncoding as i32);

    if GetFamilyCodePage(dwSrcEncoding, core::ptr::addr_of_mut!(src_family)) != S_OK
        || GetFamilyCodePage(dwDstEncoding, core::ptr::addr_of_mut!(dst_family)) != S_OK
    {
        return S_FALSE;
    }

    if src_family == dst_family {
        return S_OK;
    }

    /* we can convert any codepage to/from unicode */
    if src_family == CP_UNICODE || dst_family == CP_UNICODE {
        return S_OK;
    }

    return S_FALSE;
}

unsafe extern "C" fn fnIMultiLanguage3_GetFamilyCodePage(
    iface: *mut IMultiLanguage3,
    uiCodePage: u32,
    puiFamilyCodePage: *mut u32,
) -> i32 {
    GetFamilyCodePage(uiCodePage, puiFamilyCodePage)
}

#[no_mangle]
pub unsafe extern "system" fn LcidToRfc1766A(
    lcid: u32,
    pszRfc1766: *mut i8,
    nChar: i32,
) -> i32 {
    TRACE!(b"%04x %p %u\n\0".as_ptr() as *const i8, lcid, pszRfc1766 as *const core::ffi::c_void, nChar as u32);

    if pszRfc1766.is_null() {
        return E_INVALIDARG;
    }

    return lcid_to_rfc1766A(lcid, pszRfc1766, nChar);
}

#[no_mangle]
pub unsafe extern "system" fn LcidToRfc1766W(
    lcid: u32,
    pszRfc1766: *mut u16,
    nChar: i32,
) -> i32 {
    {
        let format = b"%04x %p %u\n\0".as_ptr() as *const i8;
        TRACE!(format, lcid, pszRfc1766, nChar);
    }
    if pszRfc1766.is_null() {
        return 0x80070057u32 as i32; // E_INVALIDARG
    }
    lcid_to_rfc1766W(lcid, pszRfc1766, nChar)
}

unsafe extern "C" fn enum_locales_proc(locale: LPCWSTR) -> i32 {
    unsafe {
        let mut end: *mut u16 = core::ptr::null_mut();
        let data = TlsGetValue(MLANG_tls_index) as *mut enum_locales_data;
        let info: *mut RFC1766INFO;

        TRACE!(b"%s\n\0".as_ptr() as *const i8, debugstr_w(locale));

        if (*data).total >= (*data).allocated {
            (*data).allocated += 32;
            (*data).info = HeapReAlloc(
                GetProcessHeap(),
                0u32,
                (*data).info as *mut core::ffi::c_void,
                ((*data).allocated as usize) * core::mem::size_of::<RFC1766INFO>(),
            ) as *mut RFC1766INFO;
            if (*data).info.is_null() {
                return 0i32;
            }
        }

        info = (*data).info.add((*data).total as usize);

        (*info).lcid = wcstol(locale as *const u16, &raw mut end, 16i32) as u32;
        if *end != 0u16 {
            return 0i32;
        }

        (*info).wszRfc1766[0] = 0u16;
        lcid_to_rfc1766W((*info).lcid, (*info).wszRfc1766.as_mut_ptr(), MAX_RFC1766_NAME as i32);

        (*info).wszLocaleName[0] = 0u16;
        GetLocaleInfoW((*info).lcid, LOCALE_SLANGUAGE, (*info).wszLocaleName.as_mut_ptr(), MAX_LOCALE_NAME as i32);

        TRACE!(b"ISO639: %s SLANGUAGE: %s\n\0".as_ptr() as *const i8, wine_dbgstr_w((*info).wszRfc1766.as_ptr()), wine_dbgstr_w((*info).wszLocaleName.as_ptr()));

        (*data).total += 1;

        return 1i32;
    }
}

pub unsafe extern "C" fn fnIMultiLanguage_GetRfc1766Info(
    iface: *mut IMultiLanguage,
    Locale: LCID,
    pRfc1766Info: *mut RFC1766INFO,
) -> HRESULT {
    let mut type_: u32 = LOCALE_SLANGUAGE;

    TRACE!(b"(%p, 0x%04x, %p)\n\0".as_ptr() as *const i8, iface, Locale, pRfc1766Info);

    if pRfc1766Info.is_null() {
        return E_INVALIDARG;
    }

    if ((Locale as u16) & 0x3ff) == LANG_ENGLISH as u16
        || ((Locale as u16) & 0x3ff) == LANG_CHINESE as u16
        || ((Locale as u16) & 0x3ff) == LANG_ARABIC as u16
    {
        if ((Locale >> 10) as u16) == 0 {
            type_ = LOCALE_SENGLANGUAGE;
        }
    } else {
        if ((Locale >> 10) as u16) == 0 {
            TRACE!(b"SUBLANGID missing in 0x%04x\n\0".as_ptr() as *const i8, Locale);
            return E_FAIL;
        }
    }

    (*pRfc1766Info).lcid = Locale;
    (*pRfc1766Info).wszRfc1766[0] = 0;
    (*pRfc1766Info).wszLocaleName[0] = 0;

    let rfc1766w_ptr = core::ptr::addr_of_mut!((*pRfc1766Info).wszRfc1766[0]);
    let localename_ptr = core::ptr::addr_of_mut!((*pRfc1766Info).wszLocaleName[0]);

    if lcid_to_rfc1766W(Locale, rfc1766w_ptr, MAX_RFC1766_NAME as i32) == 0
        && GetLocaleInfoW(Locale, type_, localename_ptr, MAX_LOCALE_NAME as i32) > 0
    {
        return S_OK;
    }

    E_INVALIDARG
}

#[no_mangle]
pub unsafe extern "C" fn fnIMultiLanguage3_GetRfc1766FromLcid(
    iface: *mut IMultiLanguage3,
    lcid: u32,
    pbstrRfc1766: *mut *mut u16,
) -> i32 {
    const MAX_RFC1766_NAME: usize = 65;
    const E_INVALIDARG: i32 = 0x80070057u32 as i32;
    const E_FAIL: i32 = 0x80004005u32 as i32;
    const S_OK: i32 = 0;

    let mut buf: [u16; 65] = [0u16; 65];

    // TRACE("%p %04x %p\n", iface, lcid, pbstrRfc1766);
    let fmt = b"%p %04x %p\n\0".as_ptr() as *const i8;
    unsafe {
        TRACE!(fmt, iface as *mut u8, lcid, pbstrRfc1766 as *mut u8);
    }

    if pbstrRfc1766.is_null() {
        return E_INVALIDARG;
    }

    if lcid_to_rfc1766W(lcid, buf.as_mut_ptr(), MAX_RFC1766_NAME as i32) == 0 {
        let bstr = unsafe { SysAllocString(buf.as_ptr()) };
        unsafe { *pbstrRfc1766 = bstr; }
        return S_OK;
    }

    return E_FAIL;
}

pub unsafe extern "C" fn fnIMultiLanguage3_GetRfc1766Info(
    iface: *mut IMultiLanguage3,
    Locale: LCID,
    LangId: LANGID,
    pRfc1766Info: *mut RFC1766INFO,
) -> HRESULT {
    unsafe {
        const E_INVALIDARG: HRESULT = 0x80070057u32 as i32;
        const E_FAIL: HRESULT = 0x80004005u32 as i32;
        const LOCALE_SLANGUAGE: LCTYPE = 2;
        const LOCALE_SENGLANGUAGE: LCTYPE = 0x1001;
        const MAX_LOCALE_NAME: i32 = 85;
        const MAX_RFC1766_NAME: usize = 65;

        extern "system" {
            fn GetLocaleInfoW(Locale: LCID, LCType: LCTYPE, lpLCData: LPWSTR, cchData: i32) -> i32;
        }

        static mut last_lang: LANGID = 0xFFFF;
        let mut type_: LCTYPE = LOCALE_SLANGUAGE;

        TRACE!(b"(%p, 0x%04x, 0x%04x, %p)\n\0".as_ptr() as *const i8, iface, Locale, LangId as u32, pRfc1766Info);

        if pRfc1766Info.is_null() {
            return E_INVALIDARG;
        }

        let primary_locale = (Locale & 0x3FF) as LANGID;
        let sub_locale = ((Locale >> 10) & 0x3F) as LANGID;

        if primary_locale == LANG_ENGLISH as u16
            || primary_locale == LANG_CHINESE as u16
            || primary_locale == LANG_ARABIC as u16
        {
            if sub_locale == 0 {
                type_ = LOCALE_SENGLANGUAGE;
            }
        } else {
            if sub_locale == 0 {
                TRACE!(b"SUBLANGID missing in 0x%04x\n\0".as_ptr() as *const i8, Locale);
                return E_FAIL;
            }
        }

        (*pRfc1766Info).lcid = Locale;
        (*pRfc1766Info).wszRfc1766[0] = 0;
        (*pRfc1766Info).wszLocaleName[0] = 0;

        let primary_langid = (LangId & 0x3FF) as LANGID;
        if primary_langid != LANG_ENGLISH as u16 && last_lang != LangId {
            FIXME(
                b"Only English names supported (requested: 0x%04x)\n\0".as_ptr() as *const i8,
                LangId as u32,
            );
            last_lang = LangId;
        }

        if lcid_to_rfc1766W(
            Locale,
            (*pRfc1766Info).wszRfc1766.as_mut_ptr(),
            MAX_RFC1766_NAME as i32,
        ) == 0 && GetLocaleInfoW(
            Locale,
            type_,
            (*pRfc1766Info).wszLocaleName.as_mut_ptr(),
            MAX_LOCALE_NAME as i32,
        ) != 0
        {
            return 0;
        }

        return E_INVALIDARG;
    }
}

unsafe extern "C" fn fnIMultiLanguage3_GetLcidFromRfc1766(
    iface: *mut IMultiLanguage3,
    pLocale: *mut u32,
    bstrRfc1766: *const u16,
) -> i32 {
    let mut hr: i32;
    let mut rfc1766: *mut IEnumRfc1766 = core::ptr::null_mut();

    TRACE!(b"%p %p %s\n\0".as_ptr() as *const i8, iface as *mut core::ffi::c_void, pLocale as *mut core::ffi::c_void, debugstr_w(bstrRfc1766));

    if pLocale.is_null() || bstrRfc1766.is_null() {
        return E_INVALIDARG;
    }

    hr = IMultiLanguage3_EnumRfc1766(iface, 0, core::ptr::addr_of_mut!(rfc1766));
    if hr < 0 {
        return hr;
    }

    hr = lcid_from_rfc1766(rfc1766, pLocale, bstrRfc1766);

    IEnumRfc1766_Release(rfc1766);
    return hr;
}

unsafe extern "C" fn fnIMLangFontLink_MapFont(
    iface: *mut IMLangFontLink,
    hDC: HDC,
    dwCodePages: u32,
    hSrcFont: HFONT,
    phDestFont: *mut HFONT,
) -> HRESULT {
    TRACE!(b"(%p)->%p %08x %p %p\n\0".as_ptr() as *const i8, iface as *mut core::ffi::c_void, hDC as *mut core::ffi::c_void, dwCodePages, hSrcFont as *mut core::ffi::c_void, phDestFont as *mut core::ffi::c_void);
    return map_font(hDC, dwCodePages, hSrcFont, phDestFont);
}

#[no_mangle]
pub unsafe extern "C" fn fnIMLangFontLink2_MapFont(
    This: *mut IMLangFontLink2,
    hDC: *mut c_void,
    dwCodePages: u32,
    chSrc: u16,
    pFont: *mut *mut c_void,
) -> i32 {
    let mut old_font: *mut c_void = core::ptr::null_mut();
    let fmt1 = b"(%p)->%p %08x %04x %p\n\0".as_ptr() as *const i8;
    TRACE!(fmt1, This as *const c_void, hDC as *const c_void, dwCodePages, chSrc as u32, pFont as *const c_void);

    if hDC.is_null() {
        return E_FAIL;
    }

    if dwCodePages != 0 {
        old_font = GetCurrentObject(hDC, OBJ_FONT);
        return map_font(hDC, dwCodePages, old_font, pFont);
    } else {
        if pFont.is_null() {
            return E_INVALIDARG;
        }
        let fixme_str = b"the situation where dwCodepages is set to zero is not implemented\n\0".as_ptr() as *const i8;
        FIXME(fixme_str);
        return E_FAIL;
    }
}

unsafe extern "C" fn fnIMLangFontLink_ReleaseFont(
    iface: *mut IMLangFontLink,
    hFont: HFONT,
) -> HRESULT {
    TRACE!(b"(%p)->%p\n\0".as_ptr() as *const i8, iface, hFont);
    return release_font(hFont);
}

unsafe extern "C" fn fnIMLangFontLink2_ReleaseFont(
    This: *mut IMLangFontLink2,
    hFont: HFONT,
) -> HRESULT {
    TRACE!(b"(%p)->%p\n\0".as_ptr() as *const i8, This as *const core::ffi::c_void, hFont as *const core::ffi::c_void);
    release_font(hFont)
}

unsafe extern "C" fn fnIMLangFontLink_ResetFontMapping(iface: *mut IMLangFontLink) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, iface);
    return clear_font_cache();
}

unsafe extern "C" fn fnIMLangFontLink2_ResetFontMapping(
    This: *mut IMLangFontLink2,
) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, This as *const core::ffi::c_void);

    clear_font_cache()
}

unsafe extern "C" fn MLANGCF_AddRef(iface: *mut IClassFactory) -> u32 {
    let this = impl_from_IClassFactory(iface);
    InterlockedIncrement(&raw mut (*this).ref_) as u32
}

unsafe extern "C" fn MLANGCF_Release(iface: *mut IClassFactory) -> u32 {
    let This = impl_from_IClassFactory(iface);
    let ref_val = InterlockedDecrement(&raw mut (*This).ref_) as u32;

    if ref_val == 0 {
        TRACE!(b"Destroying %p\n\0".as_ptr() as *const i8, This);
        HeapFree(GetProcessHeap(), 0, This as *mut core::ffi::c_void);
    }

    ref_val
}

unsafe extern "C" fn MLANGCF_CreateInstance(
    iface: *mut IClassFactory,
    pOuter: *mut IUnknown,
    riid: *const GUID,
    ppobj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This: *mut IClassFactoryImpl = impl_from_IClassFactory(iface);
    let mut hres: HRESULT;
    let mut punk: *mut IUnknown = core::ptr::null_mut();

    TRACE!(b"(%p)->(%p,%s,%p)\n\0".as_ptr() as *const core::ffi::c_char, This as *const core::ffi::c_void, pOuter as *const core::ffi::c_void, debugstr_guid(riid), ppobj as *const core::ffi::c_void);

    *ppobj = core::ptr::null_mut();
    hres = ((*This).pfnCreateInstance)(pOuter, &mut punk as *mut *mut IUnknown as *mut *mut core::ffi::c_void);
    if hres >= 0 {
        hres = IUnknown_QueryInterface(punk, riid, ppobj);
        IUnknown_Release(punk);
    }

    TRACE!(b"returning (%p) -> %x\n\0".as_ptr() as *const core::ffi::c_char, *ppobj, hres);

    hres
}

unsafe extern "C" fn fnIEnumCodePage_QueryInterface(
    iface: *mut IEnumCodePage,
    riid: *const GUID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This: *mut EnumCodePage_impl = impl_from_IEnumCodePage(iface);

    TRACE!(b"%p -> %s\n\0".as_ptr() as *const core::ffi::c_char, This as *mut core::ffi::c_void, debugstr_guid(riid));

    if IsEqualGUID(riid, &IID_IUnknown as *const GUID) != 0
        || IsEqualGUID(riid, &IID_IEnumCodePage as *const GUID) != 0
    {
        IEnumCodePage_AddRef(iface);
        TRACE!(b"Returning IID_IEnumCodePage %p ref = %d\n\0".as_ptr() as *const core::ffi::c_char, This as *mut core::ffi::c_void, (*This).ref_);
        *ppvObject = core::ptr::addr_of_mut!((*This).IEnumCodePage_iface) as *mut core::ffi::c_void;
        return 0; // S_OK
    }

    WARN!(b"(%p)->(%s,%p),not found\n\0".as_ptr() as *const core::ffi::c_char, This as *mut core::ffi::c_void, debugstr_guid(riid), ppvObject as *mut core::ffi::c_void);
    return 0x80004002u32 as HRESULT; // E_NOINTERFACE
}

unsafe extern "C" fn fnIEnumCodePage_AddRef(iface: *mut IEnumCodePage) -> u32 {
    unsafe {
        let This = impl_from_IEnumCodePage(iface);
        return InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as u32;
    }
}

unsafe extern "C" fn fnIEnumCodePage_Release(iface: *mut IEnumCodePage) -> u32 {
    let This = impl_from_IEnumCodePage(iface);
    let ref_val = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_));
    TRACE!(b"%p ref = %d\n\0".as_ptr() as *const i8, This as *const core::ffi::c_void, ref_val as i32);
    if ref_val == 0 {
        TRACE!(b"Destroying %p\n\0".as_ptr() as *const i8, This as *const core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, (*This).cpinfo as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, This as *mut core::ffi::c_void);
    }
    ref_val as u32
}

unsafe extern "C" fn fnIEnumCodePage_Clone(
    iface: *mut IEnumCodePage,
    ppEnum: *mut *mut IEnumCodePage,
) -> HRESULT {
    let This = impl_from_IEnumCodePage(iface);
    FIXME(b"%p %p\n\0".as_ptr() as *const i8, This, ppEnum);
    return E_NOTIMPL;
}

unsafe extern "C" fn fnIEnumCodePage_Next(
    iface: *mut IEnumCodePage,
    celt: u32,
    rgelt: *mut MIMECPINFO,
    pceltFetched: *mut u32,
) -> i32 {
    let mut celt = celt;
    let This: *mut EnumCodePage_impl = impl_from_IEnumCodePage(iface);

    TRACE!(b"%p %u %p %p\n\0".as_ptr() as *const i8, This as *const core::ffi::c_void, celt, rgelt as *const core::ffi::c_void, pceltFetched as *const core::ffi::c_void);

    if pceltFetched.is_null() {
        return 1; // S_FALSE
    }
    *pceltFetched = 0;

    if rgelt.is_null() {
        return 1; // S_FALSE
    }

    if (*This).pos + celt > (*This).total {
        celt = (*This).total - (*This).pos;
    }

    if celt == 0 {
        return 1; // S_FALSE
    }

    core::ptr::copy_nonoverlapping(
        rgelt,
        (*This).cpinfo.add((*This).pos as usize),
        celt as usize,
    );
    *pceltFetched = celt;
    (*This).pos += celt;

    let mut i: u32 = 0;
    while i < celt {
        let r = &*rgelt.add(i as usize);
        TRACE!(b"#%u: %08x %u %u %s %s %s %s %s %s %d\n\0".as_ptr() as *const i8, i, r.dwFlags, r.uiCodePage, r.uiFamilyCodePage, wine_dbgstr_w(r.wszDescription.as_ptr()), wine_dbgstr_w(r.wszWebCharset.as_ptr()), wine_dbgstr_w(r.wszHeaderCharset.as_ptr()), wine_dbgstr_w(r.wszBodyCharset.as_ptr()), wine_dbgstr_w(r.wszFixedWidthFont.as_ptr()), wine_dbgstr_w(r.wszProportionalFont.as_ptr()), r.bGDICharset as i32);
        i += 1;
    }

    return 0; // S_OK
}

#[no_mangle]
pub unsafe extern "C" fn fnIEnumCodePage_Reset(iface: *mut IEnumCodePage) -> HRESULT {
    let This: *mut EnumCodePage_impl = impl_from_IEnumCodePage(iface);
    TRACE!(b"%p\n\0".as_ptr() as *const i8, This);
    (*This).pos = 0;
    0 // S_OK
}

unsafe extern "C" fn fnIEnumCodePage_Skip(iface: *mut IEnumCodePage, celt: u32) -> i32 {
    unsafe {
        let This = impl_from_IEnumCodePage(iface);
        TRACE!(b"%p %u\n\0".as_ptr() as *const i8, This, celt);
        if celt >= (*This).total {
            return 1;
        }
        (*This).pos += celt;
        return 0;
    }
}

unsafe extern "C" fn fnIEnumScript_QueryInterface(
    iface: *mut IEnumScript,
    riid: *const GUID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> i32 {
    let This: *mut EnumScript_impl = impl_from_IEnumScript(iface);

    TRACE!(b"%p -> %s\n\0".as_ptr() as *const i8, This as *mut core::ffi::c_void, debugstr_guid(riid));

    if IsEqualGUID(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
        || IsEqualGUID(riid, core::ptr::addr_of!(IID_IEnumScript)) != 0
    {
        IEnumScript_AddRef(iface);
        TRACE!(b"Returning IID_IEnumScript %p ref = %d\n\0".as_ptr() as *const i8, This as *mut core::ffi::c_void, (*This).ref_);
        *ppvObject = core::ptr::addr_of_mut!((*This).IEnumScript_iface) as *mut core::ffi::c_void;
        return S_OK;
    }

    WARN!(b"(%p)->(%s,%p),not found\n\0".as_ptr() as *const i8, This as *mut core::ffi::c_void, debugstr_guid(riid), ppvObject as *mut core::ffi::c_void);
    E_NOINTERFACE
}

unsafe extern "C" fn fnIEnumScript_AddRef(iface: *mut IEnumScript) -> u32 {
    let This = impl_from_IEnumScript(iface);
    InterlockedIncrement(&raw mut (*This).ref_) as u32
}

unsafe extern "C" fn fnIEnumScript_Release(iface: *mut IEnumScript) -> u32 {
    let This: *mut EnumScript_impl = impl_from_IEnumScript(iface);
    let ref_: u32 = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) as u32;

    TRACE!(b"%p ref = %d\n\0".as_ptr() as *const i8, This as *mut core::ffi::c_void, ref_ as i32);
    if ref_ == 0 {
        TRACE!(b"Destroying %p\n\0".as_ptr() as *const i8, This as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, (*This).script_info as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, This as *mut core::ffi::c_void);
    }

    ref_
}

unsafe extern "C" fn fnIEnumScript_Clone(
    iface: *mut IEnumScript,
    ppEnum: *mut *mut IEnumScript,
) -> HRESULT {
    let This: *mut EnumScript_impl = impl_from_IEnumScript(iface);
    FIXME(b"%p %p: stub!\n\0".as_ptr() as *const i8, This, ppEnum);
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "C" fn fnIEnumScript_Next(
    iface: *mut IEnumScript,
    celt: u32,
    rgelt: *mut SCRIPTINFO,
    pceltFetched: *mut u32,
) -> HRESULT {
    let This = impl_from_IEnumScript(iface);

    TRACE!(b"%p %u %p %p\n\0".as_ptr() as *const i8, This, celt, rgelt, pceltFetched);

    if pceltFetched.is_null() || rgelt.is_null() {
        return 0x80004005u32 as i32; /* E_FAIL */
    }

    *pceltFetched = 0;

    let mut celt = celt;
    if ((*This).pos as usize).wrapping_add(celt as usize) > (*This).total as usize {
        celt = (*This).total - (*This).pos;
    }

    if celt == 0 {
        return 1; /* S_FALSE */
    }

    core::ptr::copy_nonoverlapping(
        (*This).script_info.add((*This).pos as usize),
        rgelt,
        celt as usize,
    );

    *pceltFetched = celt;
    (*This).pos += celt;

    0 /* S_OK */
}

unsafe extern "C" fn fnIEnumScript_Reset(
    iface: *mut IEnumScript,
) -> HRESULT {
    let This = impl_from_IEnumScript(iface);

    extern "C" {
    }

    TRACE!(b"%p\n\0".as_ptr() as *const i8, This);

    (*This).pos = 0;
    return 0; // S_OK
}

unsafe extern "C" fn fnIEnumScript_Skip(
    iface: *mut IEnumScript,
    celt: u32,
) -> i32 {
    let This = impl_from_IEnumScript(iface);
    TRACE!(b"%p %u\n\0".as_ptr() as *const i8, This as *const c_void, celt);
    if celt >= (*This).total {
        return S_FALSE;
    }
    (*This).pos += celt;
    S_OK
}

unsafe extern "C" fn fnIMLangFontLink_QueryInterface(
    iface: *mut IMLangFontLink,
    riid: *const IID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This = impl_from_IMLangFontLink(iface);
    return IMultiLanguage3_QueryInterface(
        core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface),
        riid,
        ppvObject,
    );
}

unsafe extern "C" fn fnIMLangFontLink_AddRef(iface: *mut IMLangFontLink) -> u32 {
    let This: *mut MLang_impl = impl_from_IMLangFontLink(iface);
    IMultiLanguage3_AddRef(core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface))
}

unsafe extern "C" fn fnIMLangFontLink_Release(iface: *mut IMLangFontLink) -> u32 {
    let This: *mut MLang_impl = impl_from_IMLangFontLink(iface);
    return IMultiLanguage3_Release(core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface));
}

unsafe extern "C" fn fnIMLangFontLink_GetCharCodePages(
    iface: *mut IMLangFontLink,
    ch_src: u16,
    codepages: *mut u32,
) -> i32 {
    let this = impl_from_IMLangFontLink(iface);
    IMLangFontLink2_GetCharCodePages(
        core::ptr::addr_of_mut!((*this).IMLangFontLink2_iface),
        ch_src,
        codepages,
    )
}

unsafe extern "C" fn fnIMLangFontLink_GetStrCodePages(
    iface: *mut IMLangFontLink,
    src: *const u16,
    src_len: i32,
    priority_cp: u32,
    codepages: *mut u32,
    ret_len: *mut i32,
) -> HRESULT {
    let This = impl_from_IMLangFontLink(iface);
    return IMLangFontLink2_GetStrCodePages(
        core::ptr::addr_of_mut!((*This).IMLangFontLink2_iface),
        src,
        src_len,
        priority_cp,
        codepages,
        ret_len,
    );
}

unsafe extern "C" fn fnIMLangFontLink_CodePageToCodePages(
    iface: *mut IMLangFontLink,
    codepage: u32,
    codepages: *mut u32,
) -> HRESULT {
    let this = impl_from_IMLangFontLink(iface);
    return IMLangFontLink2_CodePageToCodePages(
        core::ptr::addr_of_mut!((*this).IMLangFontLink2_iface),
        codepage,
        codepages,
    );
}

unsafe extern "C" fn fnIMLangFontLink_CodePagesToCodePage(
    iface: *mut IMLangFontLink,
    codepages: u32,
    def_codepage: u32,
    codepage: *mut u32,
) -> i32 {
    let this: *mut MLang_impl = impl_from_IMLangFontLink(iface);
    return IMLangFontLink2_CodePagesToCodePage(
        core::ptr::addr_of_mut!((*this).IMLangFontLink2_iface),
        codepages,
        def_codepage,
        codepage,
    );
}

unsafe extern "C" fn fnIMLangFontLink_GetFontCodePages(
    iface: *mut IMLangFontLink,
    hdc: HDC,
    hfont: HFONT,
    codepages: *mut u32,
) -> i32 {
    let This: *mut MLang_impl = impl_from_IMLangFontLink(iface);
    return IMLangFontLink2_GetFontCodePages(
        core::ptr::addr_of_mut!((*This).IMLangFontLink2_iface),
        hdc,
        hfont,
        codepages,
    );
}

unsafe extern "C" fn fnIMultiLanguage_QueryInterface(
    iface: *mut IMultiLanguage,
    riid: *const IID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = impl_from_IMultiLanguage(iface);
    IMultiLanguage3_QueryInterface(
        core::ptr::addr_of_mut!((*this).IMultiLanguage3_iface),
        riid,
        obj,
    )
}

unsafe extern "C" fn fnIMultiLanguage_AddRef(iface: *mut IMultiLanguage) -> u32 {
    let This = impl_from_IMultiLanguage(iface);
    return IMultiLanguage3_AddRef(core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface));
}

unsafe extern "C" fn fnIMultiLanguage_Release(iface: *mut IMultiLanguage) -> u32 {
    let This = impl_from_IMultiLanguage(iface);
    return IMultiLanguage3_Release(core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface));
}

unsafe extern "C" fn fnIMultiLanguage_GetNumberOfCodePageInfo(
    iface: *mut IMultiLanguage,
    cp: *mut UINT,
) -> HRESULT {
    unsafe extern "system" {
    }
    unsafe extern "C" {
    }
    let This = impl_from_IMultiLanguage(iface);
    TRACE!(b"(%p, %p)\n\0".as_ptr() as *const i8, This, cp);
    IMultiLanguage3_GetNumberOfCodePageInfo(
        core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface),
        cp,
    )
}

unsafe extern "C" fn fnIMultiLanguage_GetFamilyCodePage(
    iface: *mut IMultiLanguage,
    cp: u32,
    family_cp: *mut u32,
) -> HRESULT {
    let this = impl_from_IMultiLanguage(iface);
    let iface_ptr = &(*this).IMultiLanguage3_iface as *const IMultiLanguage3 as *mut IMultiLanguage3;
    extern "system" {
    }
    IMultiLanguage3_GetFamilyCodePage(iface_ptr, cp, family_cp)
}

unsafe extern "C" fn fnIMultiLanguage_IsConvertible(
    iface: *mut IMultiLanguage,
    src_enc: u32,
    dst_enc: u32,
) -> HRESULT {
    let This = impl_from_IMultiLanguage(iface);
    return IMultiLanguage3_IsConvertible(core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface), src_enc, dst_enc);
}

unsafe extern "C" fn fnIMultiLanguage_ConvertString(
    iface: *mut IMultiLanguage,
    mode: *mut u32,
    src_enc: u32,
    dst_enc: u32,
    src: *mut u8,
    src_size: *mut u32,
    dest: *mut u8,
    dest_size: *mut u32,
) -> HRESULT {
    let This: *mut MLang_impl = impl_from_IMultiLanguage(iface);
    return IMultiLanguage3_ConvertString(
        core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface),
        mode,
        src_enc,
        dst_enc,
        src,
        src_size,
        dest,
        dest_size,
    );
}

unsafe extern "C" fn fnIMultiLanguage_ConvertStringToUnicode(
    iface: *mut IMultiLanguage,
    mode: *mut DWORD,
    src_enc: DWORD,
    src: *mut CHAR,
    src_size: *mut UINT,
    dest: *mut WCHAR,
    dest_size: *mut UINT,
) -> HRESULT {
    let This: *mut MLang_impl = impl_from_IMultiLanguage(iface);
    return IMultiLanguage3_ConvertStringToUnicode(
        core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface),
        mode,
        src_enc,
        src,
        src_size,
        dest,
        dest_size,
    );
}

unsafe extern "C" fn fnIMultiLanguage_ConvertStringFromUnicode(
    iface: *mut IMultiLanguage,
    mode: *mut DWORD,
    encoding: DWORD,
    src: *mut WCHAR,
    src_size: *mut UINT,
    dest: *mut CHAR,
    dest_size: *mut UINT,
) -> HRESULT {
    let This = impl_from_IMultiLanguage(iface);
    return IMultiLanguage3_ConvertStringFromUnicode(
        core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface),
        mode,
        encoding,
        src,
        src_size,
        dest,
        dest_size,
    );
}

unsafe extern "C" fn fnIMultiLanguage_ConvertStringReset(
    iface: *mut IMultiLanguage,
) -> HRESULT {
    let This: *mut MLang_impl = impl_from_IMultiLanguage(iface);
    IMultiLanguage3_ConvertStringReset(core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface))
}

unsafe extern "C" fn fnIMultiLanguage_GetRfc1766FromLcid(
    iface: *mut IMultiLanguage,
    lcid: LCID,
    pbstrRfc1766: *mut BSTR,
) -> HRESULT {
    unsafe {
        let this: *mut MLang_impl = impl_from_IMultiLanguage(iface);
        let iface3: *mut IMultiLanguage3 = core::ptr::addr_of_mut!((*this).IMultiLanguage3_iface);
        IMultiLanguage3_GetRfc1766FromLcid(iface3, lcid, pbstrRfc1766)
    }
}

unsafe extern "C" fn fnIMultiLanguage_GetLcidFromRfc1766(
    iface: *mut IMultiLanguage,
    locale: *mut LCID,
    rfc1766: BSTR,
) -> HRESULT {
    let this = impl_from_IMultiLanguage(iface);
    IMultiLanguage3_GetLcidFromRfc1766(
        core::ptr::addr_of_mut!((*this).IMultiLanguage3_iface),
        locale,
        rfc1766,
    )
}

unsafe extern "C" fn fnIMultiLanguage_CreateConvertCharset(
    iface: *mut IMultiLanguage,
    src_cp: u32,
    dst_cp: u32,
    prop: u32,
    convert_charset: *mut *mut IMLangConvertCharset,
) -> i32 {
    let This = impl_from_IMultiLanguage(iface);
    let iface3 = core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface);
    IMultiLanguage3_CreateConvertCharset(iface3, src_cp, dst_cp, prop, convert_charset)
}

unsafe extern "C" fn fnIEnumRfc1766_QueryInterface(
    iface: *mut IEnumRfc1766,
    riid: *const GUID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> i32 {
    unsafe {
        let this = impl_from_IEnumRfc1766(iface);
        TRACE!(b"%p -> %s\n\0".as_ptr() as *const i8, this as *const core::ffi::c_void, debugstr_guid(riid));
        if IsEqualGUID(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
            || IsEqualGUID(riid, core::ptr::addr_of!(IID_IEnumRfc1766)) != 0
        {
            IEnumRfc1766_AddRef(iface);
            TRACE!(b"Returning IID_IEnumRfc1766 %p ref = %d\n\0".as_ptr() as *const i8, this as *const core::ffi::c_void, (*this).ref_);
            *ppvObject = core::ptr::addr_of_mut!((*this).IEnumRfc1766_iface) as *mut core::ffi::c_void;
            return S_OK;
        }
        WARN!(b"(%p) -> (%s,%p), not found\n\0".as_ptr() as *const i8, this as *const core::ffi::c_void, debugstr_guid(riid), ppvObject);
        return E_NOINTERFACE;
    }
}

unsafe extern "C" fn fnIEnumRfc1766_AddRef(iface: *mut IEnumRfc1766) -> u32 {
    let this = impl_from_IEnumRfc1766(iface);
    InterlockedIncrement(core::ptr::addr_of_mut!((*this).ref_)) as u32
}

unsafe extern "C" fn fnIEnumRfc1766_Release(
    iface: *mut IEnumRfc1766,
) -> u32 {
    let this = impl_from_IEnumRfc1766(iface);
    let ref_count = InterlockedDecrement(core::ptr::addr_of_mut!((*this).ref_)) as u32;

    TRACE!(b"%p ref = %d\n\0".as_ptr() as *const i8, this as *const core::ffi::c_void, ref_count as i32);
    if ref_count == 0 {
        TRACE!(b"Destroying %p\n\0".as_ptr() as *const i8, this as *const core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, (*this).info as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, this as *mut core::ffi::c_void);
    }
    return ref_count;
}

unsafe extern "C" fn fnIEnumRfc1766_Clone(
    iface: *mut IEnumRfc1766,
    ppEnum: *mut *mut IEnumRfc1766,
) -> HRESULT {
    let This = impl_from_IEnumRfc1766(iface);
    FIXME(b"%p %p\n\0".as_ptr() as *const i8, This, ppEnum);
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "C" fn fnIEnumRfc1766_Next(
    iface: *mut IEnumRfc1766,
    mut celt: u32,
    rgelt: *mut RFC1766INFO,
    pceltFetched: *mut u32,
) -> i32 {
    let This = impl_from_IEnumRfc1766(iface);

    TRACE!(b"%p %u %p %p\n\0".as_ptr() as *const i8, This as *const core::ffi::c_void, celt, rgelt as *const core::ffi::c_void, pceltFetched as *const core::ffi::c_void);

    if pceltFetched.is_null() {
        return 1; // S_FALSE
    }
    *pceltFetched = 0;

    if rgelt.is_null() {
        return 1; // S_FALSE
    }

    if (*This).pos + celt > (*This).total {
        celt = (*This).total - (*This).pos;
    }

    if celt == 0 {
        return 1; // S_FALSE
    }

    core::ptr::copy_nonoverlapping(
        (*This).info.add((*This).pos as usize),
        rgelt,
        celt as usize,
    );
    *pceltFetched = celt;
    (*This).pos += celt;

    for i in 0..celt as usize {
        let entry = &*rgelt.add(i);
        let s1 = wine_dbgstr_w(
            &raw const entry.wszRfc1766 as *const u16,
        );
        let s2 = wine_dbgstr_w(
            &raw const entry.wszLocaleName as *const u16,
        );
        TRACE!(b"#%u: %08x %s %s\n\0".as_ptr() as *const i8, i, entry.lcid, s1, s2);
    }

    return 0; // S_OK
}

unsafe extern "C" fn fnIEnumRfc1766_Reset(
    iface: *mut IEnumRfc1766,
) -> i32 {
    let This: *mut EnumRfc1766_impl = impl_from_IEnumRfc1766(iface);
    TRACE!(b"%p\n\0".as_ptr() as *const i8, This);
    (*This).pos = 0;
    return 0i32;
}

unsafe extern "C" fn fnIEnumRfc1766_Skip(iface: *mut IEnumRfc1766, celt: u32) -> HRESULT {
    let This = impl_from_IEnumRfc1766(iface);
    // TRACE("%p %u\n", This, celt);
    if celt >= (*This).total {
        return 1; // S_FALSE
    }
    (*This).pos += celt;
    return 0; // S_OK
}

#[no_mangle]
pub unsafe extern "system" fn Rfc1766ToLcidW(pLocale: *mut u32, pszRfc1766: *const u16) -> i32 {
    let mut enumrfc1766: *mut IEnumRfc1766 = core::ptr::null_mut();
    let mut hr: i32;

    unsafe {
        TRACE!(b"(%p, %s)\n\0".as_ptr() as *const i8, pLocale as *const core::ffi::c_void, debugstr_w(pszRfc1766));

        if pLocale.is_null() || pszRfc1766.is_null() {
            return E_INVALIDARG;
        }

        hr = EnumRfc1766_create(0, core::ptr::addr_of_mut!(enumrfc1766));
        if hr < 0 {
            return hr;
        }

        hr = lcid_from_rfc1766(enumrfc1766, pLocale, pszRfc1766);
        IEnumRfc1766_Release(enumrfc1766);

        hr
    }
}

unsafe extern "C" fn fnIMultiLanguage_EnumRfc1766(
    iface: *mut IMultiLanguage,
    ppEnumRfc1766: *mut *mut IEnumRfc1766,
) -> HRESULT {
    let This = impl_from_IMultiLanguage(iface);
    TRACE!(b"%p %p\n\0".as_ptr() as *const i8, This, ppEnumRfc1766);
    return EnumRfc1766_create(0, ppEnumRfc1766);
}

unsafe extern "C" fn fnIMultiLanguage3_QueryInterface(
    iface: *mut IMultiLanguage3,
    riid: *const GUID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this: *mut MLang_impl = impl_from_IMultiLanguage3(iface);

    TRACE!(b"(%p)->(%s %p)\n\0".as_ptr() as *const i8, this as *const core::ffi::c_void, debugstr_guid(riid), obj as *const core::ffi::c_void);

    if IsEqualGUID(riid, &raw const IID_IUnknown) != 0
        || IsEqualGUID(riid, &raw const IID_IMultiLanguage) != 0
    {
        *obj = core::ptr::addr_of_mut!((*this).IMultiLanguage_iface) as *mut core::ffi::c_void;
    } else if IsEqualGUID(riid, &raw const IID_IMLangCodePages) != 0
        || IsEqualGUID(riid, &raw const IID_IMLangFontLink) != 0
    {
        *obj = core::ptr::addr_of_mut!((*this).IMLangFontLink_iface) as *mut core::ffi::c_void;
    } else if IsEqualGUID(riid, &raw const IID_IMLangFontLink2) != 0 {
        *obj = core::ptr::addr_of_mut!((*this).IMLangFontLink2_iface) as *mut core::ffi::c_void;
    } else if IsEqualGUID(riid, &raw const IID_IMultiLanguage2) != 0
        || IsEqualGUID(riid, &raw const IID_IMultiLanguage3) != 0
    {
        *obj = core::ptr::addr_of_mut!((*this).IMultiLanguage3_iface) as *mut core::ffi::c_void;
    } else if IsEqualGUID(riid, &raw const IID_IMLangLineBreakConsole) != 0 {
        *obj = core::ptr::addr_of_mut!((*this).IMLangLineBreakConsole_iface) as *mut core::ffi::c_void;
    } else {
        WARN!(b"(%p)->(%s,%p),not found\n\0".as_ptr() as *const i8, this as *const core::ffi::c_void, debugstr_guid(riid), obj as *const core::ffi::c_void);
        *obj = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IMultiLanguage3_AddRef(iface);
    S_OK
}

unsafe extern "C" fn fnIMultiLanguage3_AddRef(iface: *mut IMultiLanguage3) -> u32 {
    let This = impl_from_IMultiLanguage3(iface);
    return InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as u32;
}

unsafe extern "C" fn fnIMultiLanguage3_Release(iface: *mut IMultiLanguage3) -> u32 {
    let This = impl_from_IMultiLanguage3(iface);
    let ref_val = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) as u32;
    TRACE!(b"(%p)->(%d)\n\0".as_ptr() as *const i8, This as *const core::ffi::c_void, ref_val as i32);
    if ref_val == 0 {
        HeapFree(GetProcessHeap(), 0, This as *mut core::ffi::c_void);
        UnlockModule();
    }
    return ref_val;
}

unsafe extern "C" fn fnIMultiLanguage3_GetNumberOfCodePageInfo(
    iface: *mut IMultiLanguage3,
    pcCodePage: *mut UINT,
) -> HRESULT {
    let This = impl_from_IMultiLanguage3(iface);
    TRACE!(b"%p, %p\n\0".as_ptr() as *const i8, This as *const core::ffi::c_void, pcCodePage as *const core::ffi::c_void);
    if pcCodePage.is_null() {
        return E_INVALIDARG;
    }
    (*pcCodePage) = (*This).total_cp;
    return S_OK;
}

unsafe extern "C" fn fnIMultiLanguage3_GetCharsetInfo(
    iface: *mut IMultiLanguage3,
    Charset: BSTR,
    pCharsetInfo: PMIMECSETINFO,
) -> HRESULT {
    let This = impl_from_IMultiLanguage3(iface);

    TRACE!(b"%p %s %p\n\0".as_ptr() as *const i8, This as *const _, debugstr_w(Charset) as *const _, pCharsetInfo as *const _);

    if pCharsetInfo.is_null() {
        return E_FAIL;
    }

    let mut i = 0;
    while i < mlang_data.len() {
        let mut n = 0;
        while n < mlang_data[i].number_of_cp as usize {
            let mut csetW = [0u16; MAX_MIMECSET_NAME];
            MultiByteToWideChar(
                CP_ACP,
                0,
                (*mlang_data[i].mime_cp_info.add(n as usize)).web_charset as *const i8,
                -1i32,
                csetW.as_mut_ptr(),
                MAX_MIMECSET_NAME as i32,
            );
            if lstrcmpiW(Charset, csetW.as_ptr()) == 0 {
                (*pCharsetInfo).uiCodePage = mlang_data[i].family_codepage;
                (*pCharsetInfo).uiInternetEncoding = (*mlang_data[i].mime_cp_info.add(n as usize)).cp;
                lstrcpyW((*pCharsetInfo).wszCharset.as_mut_ptr(), csetW.as_ptr());
                return S_OK;
            }
            let alias = (*mlang_data[i].mime_cp_info.add(n as usize)).alias;
            if !alias.is_null() && lstrcmpiW(Charset, alias) == 0 {
                (*pCharsetInfo).uiCodePage = mlang_data[i].family_codepage;
                (*pCharsetInfo).uiInternetEncoding = (*mlang_data[i].mime_cp_info.add(n as usize)).cp;
                lstrcpyW((*pCharsetInfo).wszCharset.as_mut_ptr(), alias);
                return S_OK;
            }
            n += 1;
        }
        i += 1;
    }

    /* FIXME:
     * Since we do not support charsets like iso-2022-jp and do not have
     * them in our database as a primary (web_charset) encoding this loop
     * does an attempt to 'approximate' charset name by header_charset.
     */
    i = 0;
    while i < mlang_data.len() {
        let mut n = 0;
        while n < mlang_data[i].number_of_cp as usize {
            let mut csetW = [0u16; MAX_MIMECSET_NAME];
            MultiByteToWideChar(
                CP_ACP,
                0,
                (*mlang_data[i].mime_cp_info.add(n as usize)).header_charset as *const i8,
                -1i32,
                csetW.as_mut_ptr(),
                MAX_MIMECSET_NAME as i32,
            );
            if lstrcmpiW(Charset, csetW.as_ptr()) == 0 {
                (*pCharsetInfo).uiCodePage = mlang_data[i].family_codepage;
                (*pCharsetInfo).uiInternetEncoding = (*mlang_data[i].mime_cp_info.add(n as usize)).cp;
                lstrcpyW((*pCharsetInfo).wszCharset.as_mut_ptr(), csetW.as_ptr());
                return S_OK;
            }
            n += 1;
        }
        i += 1;
    }

    E_FAIL
}

#[no_mangle]
pub unsafe extern "C" fn fnIMultiLanguage3_EnumRfc1766(
    iface: *mut IMultiLanguage3,
    LangId: LANGID,
    ppEnumRfc1766: *mut *mut IEnumRfc1766,
) -> HRESULT {
    let This: *mut MLang_impl = impl_from_IMultiLanguage3(iface);
    TRACE!(b"%p %p\n\0".as_ptr() as *const i8, This as *mut core::ffi::c_void, ppEnumRfc1766 as *mut core::ffi::c_void);
    return EnumRfc1766_create(LangId, ppEnumRfc1766);
}

unsafe extern "C" fn fnIMultiLanguage3_GetNumberOfScripts(iface: *mut IMultiLanguage3, pnScripts: *mut u32) -> i32 {
    let This: *mut MLang_impl = impl_from_IMultiLanguage3(iface);
    TRACE!(b"%p %p\n\0".as_ptr() as *const i8, This, pnScripts);
    if pnScripts.is_null() {
        return 1;
    }
    (*pnScripts) = (*This).total_scripts;
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn fnIMultiLanguage3_EnumScripts(
    iface: *mut IMultiLanguage3,
    dwFlags: DWORD,
    LangId: LANGID,
    ppEnumScript: *mut *mut IEnumScript,
) -> HRESULT {
    unsafe {
        let This = impl_from_IMultiLanguage3(iface);
        TRACE!(b"%p %08x %04x %p\n\0".as_ptr() as *const i8, This, dwFlags, LangId as u32, ppEnumScript);
        return EnumScript_create(This, dwFlags, LangId, ppEnumScript);
    }
}

unsafe extern "C" fn fnIMultiLanguage3_ValidateCodePageEx(
    iface: *mut IMultiLanguage3,
    uiCodePage: u32,
    hwnd: HWND,
    dwfIODControl: u32,
) -> i32 {
    let This: *mut MLang_impl = impl_from_IMultiLanguage3(iface);
    TRACE!(b"%p %u %p %08x\n\0".as_ptr() as *const i8, This, uiCodePage, hwnd, dwfIODControl);
    if IsValidCodePage(uiCodePage) != 0 {
        return S_OK;
    }
    let mut i = 0u32;
    while i < mlang_data.len() as u32 {
        let mut n = 0u32;
        while n < mlang_data[i as usize].number_of_cp {
            if (*mlang_data[i as usize].mime_cp_info.add(n as usize)).cp == uiCodePage {
                return S_OK;
            }
            n += 1;
        }
        i += 1;
    }
    if dwfIODControl != CPIOD_PEEK {
        FIXME(b"Request to install codepage language pack not handled\n\0".as_ptr() as *const i8);
    }
    S_FALSE
}

unsafe extern "C" fn fnIMultiLanguage3_DetectOutboundCodePage(
    iface: *mut IMultiLanguage3,
    dwFlags: u32,
    lpWideCharStr: *const u16,
    cchWideChar: u32,
    puiPreferredCodePages: *mut u32,
    nPreferredCodePages: u32,
    puiDetectedCodePages: *mut u32,
    pnDetectedCodePages: *mut u32,
    lpSpecialChar: *mut u16,
) -> i32 {
    let This: *mut MLang_impl = impl_from_IMultiLanguage3(iface);

    FIXME(
        b"(%p)->(%08x %s %p %u %p %p(%u) %s)\n\0".as_ptr() as *const i8,
        This,
        dwFlags,
        debugstr_w(lpWideCharStr),
        puiPreferredCodePages,
        nPreferredCodePages,
        puiDetectedCodePages,
        if pnDetectedCodePages.is_null() { 0 } else { *pnDetectedCodePages },
        debugstr_w(lpSpecialChar),
    );

    if puiDetectedCodePages.is_null() || pnDetectedCodePages.is_null() || *pnDetectedCodePages == 0 {
        return E_INVALIDARG;
    }

    *puiDetectedCodePages = CP_UTF8;
    *pnDetectedCodePages = 1;
    return S_OK;
}

unsafe extern "C" fn fnIMultiLanguage3_DetectOutboundCodePageInIStream(
    iface: *mut IMultiLanguage3,
    dwFlags: u32,
    pStrIn: *mut IStream,
    puiPreferredCodePages: *mut u32,
    nPreferredCodePages: u32,
    puiDetectedCodePages: *mut u32,
    pnDetectedCodePages: *mut u32,
    lpSpecialChar: *mut u16,
) -> HRESULT {
    let This = impl_from_IMultiLanguage3(iface);
    // FIXME("(%p)->(%08x %p %p %u %p %p(%u) %s)\n", This, dwFlags, pStrIn,
    //       puiPreferredCodePages, nPreferredCodePages, puiDetectedCodePages,
    //       pnDetectedCodePages, pnDetectedCodePages ? *pnDetectedCodePages : 0,
    //       debugstr_w(lpSpecialChar));

    if puiDetectedCodePages.is_null() || pnDetectedCodePages.is_null() || *pnDetectedCodePages == 0 {
        return E_INVALIDARG;
    }

    *puiDetectedCodePages = CP_UTF8;
    *pnDetectedCodePages = 1;
    return S_OK;
}

#[no_mangle]
pub unsafe extern "C" fn EnumCodePage_create(
    mlang: *mut MLang_impl,
    mut grfFlags: u32,
    LangId: u32,
    ppEnumCodePage: *mut *mut IEnumCodePage,
) -> i32 {
    let mut ecp: *mut EnumCodePage_impl;
    let mut cpinfo: *mut MIMECPINFO;
    let mut i: u32;
    let mut n: u32;

    TRACE!(b"%p, %08x, %04x, %p\n\0".as_ptr() as *const core::ffi::c_char, mlang as *mut core::ffi::c_void, grfFlags, LangId as u32, ppEnumCodePage as *mut core::ffi::c_void);

    if grfFlags == 0 {
        /* enumerate internal data base of encodings */
        grfFlags = 0x00200000; /* MIMECONTF_MIME_LATEST */
    }

    ecp = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<EnumCodePage_impl>(),
    ) as *mut EnumCodePage_impl;
    (*ecp).IEnumCodePage_iface.lpVtbl = &IEnumCodePage_vtbl;
    (*ecp).ref_ = 1;
    (*ecp).pos = 0;
    (*ecp).total = 0;

    i = 0;
    while i < mlang_data.len() as u32 {
        n = 0;
        while n < mlang_data[i as usize].number_of_cp {
            let info = &*mlang_data[i as usize].mime_cp_info.add(n as usize);
            if info.flags & grfFlags != 0 {
                (*ecp).total += 1;
            }
            n += 1;
        }
        i += 1;
    }

    (*ecp).cpinfo = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<MIMECPINFO>() * (*ecp).total as usize,
    ) as *mut MIMECPINFO;
    cpinfo = (*ecp).cpinfo;

    i = 0;
    while i < mlang_data.len() as u32 {
        n = 0;
        while n < mlang_data[i as usize].number_of_cp {
            let info = &*mlang_data[i as usize].mime_cp_info.add(n as usize);
            if info.flags & grfFlags != 0 {
                fill_cp_info(
                    &raw const mlang_data[i as usize],
                    n,
                    cpinfo,
                );
                cpinfo = cpinfo.add(1);
            }
            n += 1;
        }
        i += 1;
    }

    TRACE!(b"enumerated %d codepages with flags %08x\n\0".as_ptr() as *const core::ffi::c_char, (*ecp).total, grfFlags);

    *ppEnumCodePage = &mut (*ecp).IEnumCodePage_iface as *mut IEnumCodePage;

    0 // S_OK
}

unsafe extern "C" fn fnIMultiLanguage_GetCodePageInfo(
    iface: *mut IMultiLanguage,
    uiCodePage: u32,
    pCodePageInfo: *mut MIMECPINFO,
) -> i32 {
    let This: *mut MLang_impl = impl_from_IMultiLanguage(iface);

    TRACE!(b"%p, %u, %p\n\0".as_ptr() as *const i8, This as *const core::ffi::c_void, uiCodePage, pCodePageInfo as *const core::ffi::c_void);

    let mut i: u32 = 0;
    while (i as usize) < mlang_data.len() {
        let mlang: &mlang_data = &mlang_data[i as usize];
        let mut n: u32 = 0;
        while (n as u32) < mlang.number_of_cp {
            if (*mlang.mime_cp_info.add(n as usize)).cp == uiCodePage {
                fill_cp_info(
                    &raw const mlang_data[i as usize],
                    n,
                    pCodePageInfo,
                );
                return S_OK;
            }
            n += 1;
        }
        i += 1;
    }

    return S_FALSE;
}

unsafe extern "C" fn fnIMultiLanguage3_GetCodePageInfo(
    iface: *mut IMultiLanguage3,
    uiCodePage: u32,
    LangId: u16,
    pCodePageInfo: *mut MIMECPINFO,
) -> i32 {
    let this = impl_from_IMultiLanguage3(iface);

    TRACE!(b"%p, %u, %04x, %p\n\0".as_ptr() as *const i8, this, uiCodePage, LangId as u32, pCodePageInfo);

    let array_len = core::mem::size_of_val(&mlang_data) / core::mem::size_of_val(&mlang_data[0]);

    for i in 0..array_len {
        let n_loop = mlang_data[i].number_of_cp;
        for n in 0..n_loop {
            if (*mlang_data[i].mime_cp_info.add(n as usize)).cp == uiCodePage {
                fill_cp_info(
                    core::ptr::addr_of!(mlang_data[i]) as *const mlang_data,
                    n as u32,
                    pCodePageInfo,
                );
                return S_OK;
            }
        }
    }

    return S_FALSE;
}

unsafe extern "C" fn fnIMLangFontLink2_QueryInterface(
    iface: *mut IMLangFontLink2,
    riid: *const GUID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> i32 {
    let This = impl_from_IMLangFontLink2(iface);
    return IMultiLanguage3_QueryInterface(
        core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface),
        riid,
        ppvObject,
    );
}

unsafe extern "C" fn fnIMLangFontLink2_AddRef(iface: *mut IMLangFontLink2) -> u32 {
    let This: *mut MLang_impl = impl_from_IMLangFontLink2(iface);
    IMultiLanguage3_AddRef(core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface))
}

unsafe extern "C" fn fnIMLangFontLink2_Release(iface: *mut IMLangFontLink2) -> u32 {
    let This = impl_from_IMLangFontLink2(iface);
    IMultiLanguage3_Release(core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface))
}

unsafe extern "C" fn fnIMLangFontLink2_GetCharCodePages(
    iface: *mut IMLangFontLink2,
    ch_src: u16,
    ret_codepages: *mut u32,
) -> i32 {
    let This = impl_from_IMLangFontLink2(iface);
    TRACE!(b"(%p)->(%s %p)\n\0".as_ptr() as *const i8, This, debugstr_wn(&ch_src as *const u16, 1), ret_codepages);
    *ret_codepages = 0;
    let array_size = core::mem::size_of_val(&mlang_data) / core::mem::size_of_val(&mlang_data[0]);
    let mut i: u32 = 0;
    while i < array_size as u32 {
        let mut used_dc: i32 = 0;
        let mut buf: i8 = 0;
        let mlang_data_ptr = &mlang_data as *const _ as *const mlang_data;
        let family_codepage = (*mlang_data_ptr.add(i as usize)).family_codepage;
        WideCharToMultiByte(
            family_codepage,
            WC_NO_BEST_FIT_CHARS,
            &ch_src as *const u16,
            1,
            &mut buf as *mut i8,
            1,
            core::ptr::null_mut(),
            &mut used_dc as *mut i32,
        );
        if used_dc == 0 {
            let mut codepages: u32 = 0;
            IMLangFontLink2_CodePageToCodePages(iface, family_codepage, &mut codepages as *mut u32);
            *ret_codepages |= codepages;
        }
        i += 1;
    }
    return S_OK;
}

unsafe extern "C" fn fnIMLangFontLink2_GetStrCodePages(
    iface: *mut IMLangFontLink2,
    src: *const WCHAR,
    src_len: LONG,
    priority_cp: DWORD,
    codepages: *mut DWORD,
    ret_len: *mut LONG,
) -> HRESULT {
    let This: *mut MLang_impl = impl_from_IMLangFontLink2(iface);
    let mut i: LONG;
    let mut cps: DWORD = 0;

    TRACE!(b"(%p)->(%s:%d %x %p %p)\n\0".as_ptr() as *const i8, This, debugstr_wn(src, src_len), src_len, priority_cp, codepages, ret_len);

    if !codepages.is_null() {
        *codepages = 0;
    }
    if !ret_len.is_null() {
        *ret_len = 0;
    }

    if src.is_null() || src_len == 0 || src_len < 0 {
        return E_INVALIDARG;
    }

    i = 0;
    while i < src_len {
        let mut cp: DWORD = 0;
        let ret: HRESULT;

        ret = IMLangFontLink2_GetCharCodePages(iface, *src.offset(i as isize), core::ptr::addr_of_mut!(cp));
        if ret != S_OK {
            return E_FAIL;
        }

        // Wine 10 (upstream 15c9005acb6): a character joins the run only while the code-page
        // set stays non-empty AND it agrees with the run on the priority code page; otherwise
        // the run ends BEFORE it. The old body narrowed unconditionally, so it ran on past
        // the point where the set was empty (winetest mlang.c:1278 expects 3, not 4).
        if cps == 0 {
            cps = cp;
        } else if (cps & cp) != 0 && ((priority_cp & cps) ^ (priority_cp & cp)) == 0 {
            cps &= cp;
        } else {
            i -= 1;
            break;
        }
        i += 1;
    }

    if !codepages.is_null() {
        *codepages = cps;
    }
    if !ret_len.is_null() {
        *ret_len = min(i + 1, src_len);
    }
    return S_OK;
}

unsafe extern "C" fn fnIMLangFontLink2_CodePageToCodePages(iface: *mut IMLangFontLink2, codepage: u32, codepages: *mut u32) -> i32 {
    let This = impl_from_IMLangFontLink2(iface);
    let mut cs: CHARSETINFO = core::mem::zeroed();
    let mut rc: i32 = 0;

    TRACE!(b"(%p)->(%u %p)\n\0".as_ptr() as *const i8, This, codepage, codepages);

    let src: *mut u32 = codepage as usize as *mut u32;
    rc = TranslateCharsetInfo(src, &mut cs as *mut CHARSETINFO, TCI_SRCCODEPAGE);
    if rc != 0 {
        *codepages = cs.fs.fsCsb[0];
        TRACE!(b"resulting codepages 0x%x\n\0".as_ptr() as *const i8, *codepages);
        return S_OK;
    }

    TRACE!(b"codepage not found\n\0".as_ptr() as *const i8);
    *codepages = 0;
    return E_FAIL;
}

unsafe extern "C" fn fnIMLangFontLink2_CodePagesToCodePage(
    iface: *mut IMLangFontLink2,
    codepages: u32,
    def_codepage: u32,
    codepage: *mut u32,
) -> i32 {
    extern "system" {
        fn TranslateCharsetInfo(arg1: *const u32, arg2: *mut CHARSETINFO, arg3: u32) -> i32;
    }
    const TCI_SRCCODEPAGE: u32 = 0x00000002; // wingdi.h:962 (was 1 = TCI_SRCCHARSET)
    const TCI_SRCFONTSIG: u32 = 0x3; // wingdi.h
    const S_OK: i32 = 0;
    const E_FAIL: i32 = 0x80004005u32 as i32;

    unsafe {
        let This: *mut MLang_impl = impl_from_IMLangFontLink2(iface);
        let mut mask: u32 = 0;
        let mut cs: CHARSETINFO = core::mem::zeroed();
        let mut rc: i32;
        let mut i: u32;

        TRACE!(b"(%p)->(0x%x %u %p)\n\0" as *const u8 as *const i8, This, codepages, def_codepage, codepage);

        *codepage = 0;

        rc = TranslateCharsetInfo(
            (def_codepage as usize as *const u32),
            &mut cs,
            TCI_SRCCODEPAGE,
        );
        if (rc != 0) && ((codepages & cs.fs.fsCsb[0]) != 0) {
            TRACE!(b"Found Default Codepage\n\0" as *const u8 as *const i8);
            *codepage = def_codepage;
            return S_OK;
        }

        for i in 0..32 {
            mask = 1u32.wrapping_shl(i);
            if (codepages & mask) != 0 {
                let mut Csb: [u32; 2] = [0u32; 2];
                Csb[0] = mask;
                Csb[1] = 0x0;
                rc = TranslateCharsetInfo(Csb.as_mut_ptr(), &mut cs, TCI_SRCFONTSIG);
                if rc == 0 {
                    continue;
                }
                TRACE!(b"Falling back to least significant found CodePage %u\n\0" as *const u8 as *const i8, cs.ciACP);
                *codepage = cs.ciACP;
                return S_OK;
            }
        }

        TRACE!(b"no codepage found\n\0" as *const u8 as *const i8);
        return E_FAIL;
    }
}

unsafe extern "C" fn fnIMLangFontLink2_GetFontCodePages(
    iface: *mut IMLangFontLink2,
    hdc: HDC,
    hfont: HFONT,
    codepages: *mut DWORD,
) -> HRESULT {
    let This: *mut MLang_impl = impl_from_IMLangFontLink2(iface);
    let mut fontsig: core::mem::MaybeUninit<FONTSIGNATURE> = core::mem::MaybeUninit::uninit();
    let old_font: HFONT;

    TRACE!(b"(%p)->(%p %p %p)\n\0".as_ptr() as *const i8, This as *const core::ffi::c_void, hdc as *const core::ffi::c_void, hfont as *const core::ffi::c_void, codepages as *const core::ffi::c_void);

    // mlang.c zeroes the optional out-param first and fails when the font cannot be
    // selected; both were dropped. Its final store is unguarded (a NULL crashes the C on
    // success) — guarded here too.
    if !codepages.is_null() {
        *codepages = 0;
    }

    old_font = SelectObject(hdc, hfont as *mut core::ffi::c_void) as HFONT;
    if old_font.is_null() {
        return E_FAIL;
    }
    GetTextCharsetInfo(hdc, fontsig.as_mut_ptr(), 0);
    SelectObject(hdc, old_font as *mut core::ffi::c_void);

    let fontsig = fontsig.assume_init();
    if !codepages.is_null() {
        *codepages = fontsig.fsCsb[0];
    }
    TRACE!(b"ret 0x%x\n\0".as_ptr() as *const i8, fontsig.fsCsb[0]);

    return 0; // S_OK
}

unsafe extern "C" fn fnIMLangLineBreakConsole_QueryInterface(
    iface: *mut IMLangLineBreakConsole,
    riid: *const c_void,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IMLangLineBreakConsole(iface);
    return IMultiLanguage3_QueryInterface(
        core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface),
        riid as *const GUID,
        ppvObject,
    );
}

unsafe extern "C" fn fnIMLangLineBreakConsole_AddRef(iface: *mut IMLangLineBreakConsole) -> u32 {
    let this: *mut MLang_impl = impl_from_IMLangLineBreakConsole(iface);
    IMultiLanguage3_AddRef(core::ptr::addr_of_mut!((*this).IMultiLanguage3_iface))
}

unsafe extern "C" fn fnIMLangLineBreakConsole_Release(
    iface: *mut IMLangLineBreakConsole,
) -> u32 {
    let This = impl_from_IMLangLineBreakConsole(iface);
    return IMultiLanguage3_Release(core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface));
}

unsafe extern "C" fn MLangConvertCharset_QueryInterface(
    iface: *mut IMLangConvertCharset,
    riid: *const IID,
    obj: *mut *mut core::ffi::c_void,
) -> i32 {
    let This = impl_from_IMLangConvertCharset(iface);

    TRACE!(b"(%p)->(%s %p)\n\0".as_ptr() as *const i8, This as *const core::ffi::c_void, debugstr_guid(riid), obj as *const core::ffi::c_void);

    if IsEqualIID(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
        || IsEqualIID(riid, core::ptr::addr_of!(IID_IMLangConvertCharset)) != 0
    {
        *obj = core::ptr::addr_of_mut!((*This).IMLangConvertCharset_iface) as *mut core::ffi::c_void;
        IMLangConvertCharset_AddRef(iface);
        return 0; // S_OK
    }

    *obj = core::ptr::null_mut();
    0x80004002u32 as i32 // E_NOINTERFACE
}

unsafe extern "C" fn MLangConvertCharset_AddRef(iface: *mut IMLangConvertCharset) -> u32 {
    let This = impl_from_IMLangConvertCharset(iface);
    let ref_val = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as u32;
    TRACE!(b"(%p)->(%u)\n\0".as_ptr() as *const i8, This as *const core::ffi::c_void, ref_val);
    ref_val
}

unsafe extern "C" fn MLangConvertCharset_Release(iface: *mut IMLangConvertCharset) -> u32 {
    extern "C" {
    }
    extern "system" {
        fn GetProcessHeap() -> *mut core::ffi::c_void;
        fn HeapFree(hHeap: *mut core::ffi::c_void, dwFlags: u32, lpMem: *mut core::ffi::c_void) -> i32;
    }
    let This: *mut convert_charset = impl_from_IMLangConvertCharset(iface);
    let ref_: u32 = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) as u32;
    TRACE!(b"(%p)->(%u)\n\0".as_ptr() as *const core::ffi::c_char, This, ref_);
    if ref_ == 0 {
        HeapFree(GetProcessHeap(), 0, This as *mut core::ffi::c_void);
        UnlockModule();
    }
    ref_
}

#[allow(non_snake_case)]
unsafe extern "C" fn MLangConvertCharset_Initialize(
    iface: *mut IMLangConvertCharset,
    src_cp: u32,
    dst_cp: u32,
    prop: u32,
) -> HRESULT {
    const MLCONVCHARF_USEDEFCHAR: u32 = 0x20000000;
    const S_OK: HRESULT = 0;

    unsafe {
        let This: *mut convert_charset = impl_from_IMLangConvertCharset(iface);
        TRACE!(b"(%p)->(%u %u 0x%08x)\n\0".as_ptr() as *const i8, This as *mut core::ffi::c_void, src_cp, dst_cp, prop);
        let mut prop = prop;
        prop &= !MLCONVCHARF_USEDEFCHAR;
        if prop != 0 {
            FIXME(
                b"property 0x%08x not supported\n\0".as_ptr() as *const i8,
                prop,
            );
        }
        (*This).src_cp = src_cp;
        (*This).dst_cp = dst_cp;
        return S_OK;
    }
}

unsafe extern "C" fn MLangConvertCharset_GetSourceCodePage(
    iface: *mut IMLangConvertCharset,
    src_cp: *mut u32,
) -> i32 {
    let This = impl_from_IMLangConvertCharset(iface);

    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, This, src_cp);

    if src_cp.is_null() {
        return E_INVALIDARG;
    }
    *src_cp = (*This).src_cp;
    return S_OK;
}

unsafe extern "C" fn MLangConvertCharset_GetDestinationCodePage(iface: *mut IMLangConvertCharset, dst_cp: *mut u32) -> i32 {
    let this = impl_from_IMLangConvertCharset(iface);
    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, this as *const core::ffi::c_void, dst_cp as *const core::ffi::c_void);
    if dst_cp.is_null() {
        return E_INVALIDARG;
    }
    *dst_cp = (*this).dst_cp;
    return 0;
}

unsafe extern "C" fn MLangConvertCharset_GetProperty(iface: *mut IMLangConvertCharset, prop: *mut DWORD) -> HRESULT {
    let This = impl_from_IMLangConvertCharset(iface);
    FIXME(b"(%p)->(%p): stub\n\0".as_ptr() as *const i8, This as *mut core::ffi::c_void, prop as *mut core::ffi::c_void);
    return E_NOTIMPL;
}

unsafe extern "C" fn MLangConvertCharset_DoConversion(
    iface: *mut IMLangConvertCharset,
    src: *mut u8,
    src_size: *mut u32,
    dest: *mut u8,
    dest_size: *mut u32,
) -> i32 {
    let This = impl_from_IMLangConvertCharset(iface);
    FIXME(
        b"(%p)->(%p %p %p %p): stub\n\0".as_ptr() as *const i8,
        This,
        src,
        src_size,
        dest,
        dest_size,
    );
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    if register_codepages() == 0 {
        return E_FAIL;
    }
    __wine_register_resources(instance)
}

unsafe extern "C" fn ConvertJISJapaneseToUnicode(
    input: *const i8,
    count: u32,
    output: *mut u16,
    out_count: u32,
) -> u32 {
    let sjis_string = HeapAlloc(GetProcessHeap(), 0, count as usize);
    let mut rc: u32 = ConvertJIS2SJIS(input, count, sjis_string as *mut i8);
    if rc != 0 {
        TRACE!(b"%s\n\0".as_ptr() as *const i8, debugstr_an(sjis_string as *const i8, rc as i32));
        if !output.is_null() {
            rc = MultiByteToWideChar(
                932,
                0,
                sjis_string as *const i8,
                rc as i32,
                output,
                out_count as i32,
            ) as u32;
        } else {
            rc = MultiByteToWideChar(
                932,
                0,
                sjis_string as *const i8,
                rc as i32,
                core::ptr::null_mut(),
                0,
            ) as u32;
        }
    }
    HeapFree(GetProcessHeap(), 0, sjis_string as *mut core::ffi::c_void);
    rc
}

unsafe extern "C" fn ConvertUnknownJapaneseToUnicode(input: *const i8, count: u32, output: *mut u16, out_count: u32) -> u32 {
    let mut sjis_string: *mut i8;
    let mut rc: u32 = 0;
    let code: u32 = DetectJapaneseCode(input as *const u8, count);
    TRACE!(b"Japanese code %i\n\0".as_ptr() as *const i8, code);

    match code {
        0 => {
            if !output.is_null() {
                rc = MultiByteToWideChar(0 /* CP_ACP */, 0, input, count as i32, output, out_count as i32) as u32;
            } else {
                rc = MultiByteToWideChar(0 /* CP_ACP */, 0, input, count as i32, core::ptr::null_mut(), 0) as u32;
            }
        }
        932 => {
            if !output.is_null() {
                rc = MultiByteToWideChar(932, 0, input, count as i32, output, out_count as i32) as u32;
            } else {
                rc = MultiByteToWideChar(932, 0, input, count as i32, core::ptr::null_mut(), 0) as u32;
            }
        }
        51932 => {
            if !output.is_null() {
                rc = MultiByteToWideChar(20932, 0, input, count as i32, output, out_count as i32) as u32;
            } else {
                rc = MultiByteToWideChar(20932, 0, input, count as i32, core::ptr::null_mut(), 0) as u32;
            }
        }
        50220 => {
            sjis_string = HeapAlloc(GetProcessHeap(), 0, count as usize) as *mut i8;
            rc = ConvertJIS2SJIS(input, count, sjis_string);
            if rc != 0 {
                TRACE!(b"%s\n\0".as_ptr() as *const i8, debugstr_an(sjis_string, rc as i32));
                if !output.is_null() {
                    rc = MultiByteToWideChar(932, 0, sjis_string, rc as i32, output, out_count as i32) as u32;
                } else {
                    rc = MultiByteToWideChar(932, 0, sjis_string, rc as i32, core::ptr::null_mut(), 0) as u32;
                }
            }
            HeapFree(GetProcessHeap(), 0, sjis_string as *mut core::ffi::c_void);
        }
        _ => {}
    }

    return rc;
}

unsafe extern "C" fn ConvertJapaneseUnicodeToJIS(
    input: *const u16,
    count: u32,
    output: *mut i8,
    out_count: u32,
) -> u32 {
    unsafe {
        let mut sjis_string: *mut i8;
        let len: i32;
        let mut rc: u32 = 0;

        len = WideCharToMultiByte(932, 0, input, count as i32, core::ptr::null_mut(), 0, core::ptr::null(), core::ptr::null_mut());
        sjis_string = HeapAlloc(GetProcessHeap(), 0, len as usize) as *mut i8;
        WideCharToMultiByte(932, 0, input, count as i32, sjis_string, len, core::ptr::null(), core::ptr::null_mut());
        TRACE!(b"%s\n\0".as_ptr() as *const i8, debugstr_an(sjis_string, len));

        rc = ConvertSJIS2JIS(sjis_string, len as u32, core::ptr::null_mut());
        if out_count >= rc {
            ConvertSJIS2JIS(sjis_string, len as u32, output);
        }
        HeapFree(GetProcessHeap(), 0, sjis_string as *mut core::ffi::c_void);
        return rc;
    }
}

unsafe extern "C" fn fnIMultiLanguage3_CreateConvertCharset(
    iface: *mut IMultiLanguage3,
    src_cp: u32,
    dst_cp: u32,
    prop: u32,
    convert_charset: *mut *mut IMLangConvertCharset,
) -> HRESULT {
    let hr: HRESULT;

    TRACE!(b"(%u %u 0x%08x %p)\n\0".as_ptr() as *const i8, src_cp, dst_cp, prop, convert_charset);

    hr = MLangConvertCharset_create(core::ptr::null_mut() as *mut IUnknown, convert_charset as *mut *mut core::ffi::c_void);
    // FAILED(hr) equivalent: check if HRESULT is negative
    if hr < 0 { return hr; }

    // IMLangConvertCharset_Initialize(*convert_charset, src_cp, dst_cp, prop)
    // This is a COM vtable call; expand the macro faithfully
    let this: *mut IMLangConvertCharset = *convert_charset;
    let vtbl: *mut IMLangConvertCharsetVtbl = (*this).lpVtbl as *mut IMLangConvertCharsetVtbl;
    return ((*vtbl).Initialize)(this, src_cp, dst_cp, prop);
}

unsafe extern "C" fn fnIMultiLanguage3_IsConvertible(
    iface: *mut IMultiLanguage3,
    dwSrcEncoding: u32,
    dwDstEncoding: u32,
) -> i32 {
    unsafe { IsConvertINetStringAvailable(dwSrcEncoding, dwDstEncoding) }
}

#[no_mangle]
pub unsafe extern "system" fn Rfc1766ToLcidA(lcid: *mut LCID, rfc1766A: LPCSTR) -> HRESULT {
    // WCHAR rfc1766W[MAX_RFC1766_NAME + 1];
    let mut rfc1766W: [u16; MAX_RFC1766_NAME + 1] = [0; MAX_RFC1766_NAME + 1];

    if rfc1766A.is_null() {
        return E_INVALIDARG;
    }

    MultiByteToWideChar(
        0, /* CP_ACP */
        0,
        rfc1766A,
        -1,
        rfc1766W.as_mut_ptr(),
        MAX_RFC1766_NAME as i32,
    );
    rfc1766W[MAX_RFC1766_NAME] = 0;

    Rfc1766ToLcidW(lcid, rfc1766W.as_ptr())
}

unsafe extern "C" fn fnIMultiLanguage_EnumCodePages(
    iface: *mut IMultiLanguage,
    grfFlags: DWORD,
    ppEnumCodePage: *mut *mut IEnumCodePage,
) -> HRESULT {
    let This = impl_from_IMultiLanguage(iface);
    TRACE!(b"%p %08x %p\n\0".as_ptr() as *const i8, This, grfFlags, ppEnumCodePage);
    return EnumCodePage_create(This, grfFlags, 0, ppEnumCodePage);
}

unsafe extern "C" fn fnIMultiLanguage3_EnumCodePages(
    iface: *mut IMultiLanguage3,
    grfFlags: u32,
    LangId: u16,
    ppEnumCodePage: *mut *mut IEnumCodePage,
) -> HRESULT {
    extern "C" {
    }

    let This: *mut MLang_impl = impl_from_IMultiLanguage3(iface);
    TRACE!(b"%p %08x %04x %p\n\0".as_ptr() as *const i8, This as *mut core::ffi::c_void, grfFlags as u32, LangId as u32, ppEnumCodePage as *mut *mut IEnumCodePage);
    EnumCodePage_create(This, grfFlags, LangId as u32, ppEnumCodePage)
}

#[no_mangle]
pub unsafe extern "system" fn ConvertINetUnicodeToMultiByte(
    pdwMode: *mut u32,
    dwEncoding: u32,
    pSrcStr: *const u16,
    pcSrcSize: *mut i32,
    pDstStr: *mut u8,
    pcDstSize: *mut i32,
) -> i32 {
    let mut src_len: i32 = -1;
    let mut dwEncoding = dwEncoding;
    let mut destsz: i32;
    let mut size: i32;
    let pcSrcSize_actual: *mut i32;

    let result: i32;
    'outer: loop {
        TRACE!(b"%p %d %s %p %p %p\n\0".as_ptr() as *const i8, pdwMode, dwEncoding, debugstr_w(pSrcStr), pcSrcSize, pDstStr, pcDstSize);

        if pcDstSize.is_null() {
            result = S_OK;
            break 'outer;
        }

        pcSrcSize_actual = if pcSrcSize.is_null() {
            &mut src_len as *mut i32
        } else {
            pcSrcSize
        };

        destsz = if !pDstStr.is_null() { *pcDstSize } else { 0 };
        *pcDstSize = 0;

        if pSrcStr.is_null() || *pcSrcSize_actual == 0 {
            result = S_OK;
            break 'outer;
        }

        if *pcSrcSize_actual == -1 {
            *pcSrcSize_actual = lstrlenW(pSrcStr);
        }

        if dwEncoding == 51932 {
            dwEncoding = 20932;
        }

        if dwEncoding == CP_UNICODE {
            if *pcSrcSize_actual == -1 {
                *pcSrcSize_actual = lstrlenW(pSrcStr);
            }

            let min_val = if *pcSrcSize_actual < destsz {
                *pcSrcSize_actual
            } else {
                destsz
            };
            size = min_val * (core::mem::size_of::<u16>() as i32);

            if !pDstStr.is_null() {
                memmove(
                    pDstStr as *mut core::ffi::c_void,
                    pSrcStr as *const core::ffi::c_void,
                    size as usize,
                );
            }

            if size >= destsz {
                *pcSrcSize_actual = 0;
                *pcDstSize = 0;
                result = E_FAIL;
                break 'outer;
            }
        } else if dwEncoding == 50220 || dwEncoding == 50221 || dwEncoding == 50222 {
            size = ConvertJapaneseUnicodeToJIS(
                pSrcStr,
                *pcSrcSize_actual as u32,
                core::ptr::null_mut(),
                0,
            ) as i32;
            if size == 0 {
                *pcSrcSize_actual = 0;
                *pcDstSize = 0;
                result = E_FAIL;
                break 'outer;
            }

            if !pDstStr.is_null() {
                size = ConvertJapaneseUnicodeToJIS(
                    pSrcStr,
                    *pcSrcSize_actual as u32,
                    pDstStr as *mut i8,
                    destsz as u32,
                ) as i32;
                if size == 0 {
                    *pcSrcSize_actual = 0;
                    *pcDstSize = 0;
                    result = E_FAIL;
                    break 'outer;
                }
            }
        } else {
            size = WideCharToMultiByte(
                dwEncoding,
                0,
                pSrcStr,
                *pcSrcSize_actual,
                core::ptr::null_mut(),
                0,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            if size == 0 {
                *pcSrcSize_actual = 0;
                *pcDstSize = 0;
                result = E_FAIL;
                break 'outer;
            }

            if !pDstStr.is_null() {
                size = WideCharToMultiByte(
                    dwEncoding,
                    0,
                    pSrcStr,
                    *pcSrcSize_actual,
                    pDstStr as *mut i8,
                    destsz,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                );
                if size == 0 {
                    *pcSrcSize_actual = 0;
                    *pcDstSize = 0;
                    result = E_FAIL;
                    break 'outer;
                }
            }
        }

        *pcDstSize = size;
        result = S_OK;
        break 'outer;
    }

    result
}

unsafe extern "C" fn fnIMultiLanguage3_ConvertStringToUnicode(
    iface: *mut IMultiLanguage3,
    pdwMode: *mut u32,
    dwEncoding: u32,
    pSrcStr: *mut i8,
    pcSrcSize: *mut u32,
    pDstStr: *mut u16,
    pcDstSize: *mut u32,
) -> i32 {
    unsafe {
        ConvertINetMultiByteToUnicode(pdwMode, dwEncoding, pSrcStr, pcSrcSize as *mut i32, pDstStr, pcDstSize as *mut i32)
    }
}

unsafe extern "C" fn fnIMultiLanguage3_ConvertStringToUnicodeEx(
    iface: *mut IMultiLanguage3,
    pdwMode: *mut u32,
    dwEncoding: u32,
    pSrcStr: *mut i8,
    pcSrcSize: *mut u32,
    pDstStr: *mut u16,
    pcDstSize: *mut u32,
    dwFlag: u32,
    lpFallBack: *mut u16,
) -> HRESULT {
    if dwFlag != 0 || !lpFallBack.is_null() {
        FIXME(
            b"Ignoring dwFlag (0x%x/%d) and lpFallBack (%p)\n\0".as_ptr() as *const i8,
            dwFlag,
            dwFlag,
            lpFallBack,
        );
    }
    ConvertINetMultiByteToUnicode(
        pdwMode,
        dwEncoding,
        pSrcStr,
        pcSrcSize as *mut i32,
        pDstStr,
        pcDstSize as *mut i32,
    )
}

unsafe extern "C" fn MLangConvertCharset_DoConversionToUnicode(
    iface: *mut IMLangConvertCharset,
    src: *mut i8,
    src_size: *mut u32,
    dest: *mut u16,
    dest_size: *mut u32,
) -> i32 {
    unsafe {
        let This = impl_from_IMLangConvertCharset(iface);
        TRACE!(b"(%p)->(%p %p %p %p)\n\0".as_ptr() as *const i8, This, src, src_size, dest, dest_size);
        ConvertINetMultiByteToUnicode(
            core::ptr::null_mut(),
            (*This).src_cp,
            src,
            src_size as *mut i32,
            dest,
            dest_size as *mut i32,
        )
    }
}

#[no_mangle]
pub unsafe extern "system" fn ConvertINetString(
    pdwMode: *mut u32,
    dwSrcEncoding: u32,
    dwDstEncoding: u32,
    pSrcStr: *const i8,
    pcSrcSize: *mut i32,
    pDstStr: *mut i8,
    pcDstSize: *mut i32,
) -> HRESULT {
    const CP_UNICODE: u32 = 1200;
    const S_OK: HRESULT = 0;

    extern "system" {
        fn ConvertINetMultiByteToUnicode(
            pdwMode: *mut u32,
            dwEncoding: u32,
            pSrcStr: *const i8,
            pcSrcSize: *mut i32,
            pDstStr: *mut u16,
            pcDstSize: *mut i32,
        ) -> HRESULT;
        fn ConvertINetUnicodeToMultiByte(
            pdwMode: *mut u32,
            dwEncoding: u32,
            pSrcStr: *const u16,
            pcSrcSize: *mut i32,
            pDstStr: *mut i8,
            pcDstSize: *mut i32,
        ) -> HRESULT;
        fn GetProcessHeap() -> *mut core::ffi::c_void;
        fn HeapAlloc(
            hHeap: *mut core::ffi::c_void,
            dwFlags: u32,
            dwBytes: usize,
        ) -> *mut core::ffi::c_void;
        fn HeapFree(
            hHeap: *mut core::ffi::c_void,
            dwFlags: u32,
            lpMem: *mut core::ffi::c_void,
        ) -> i32;
    }
    unsafe extern "C" {
        fn debugstr_a(s: *const i8) -> *const i8;
    }

    unsafe {
        let ds = debugstr_a(pSrcStr);
        TRACE!(b"%p %d %d %s %p %p %p\n\0".as_ptr() as *const i8, pdwMode as *const core::ffi::c_void, dwSrcEncoding as i32, dwDstEncoding as i32, ds, pcSrcSize as *const core::ffi::c_void, pDstStr as *const core::ffi::c_void, pcDstSize as *const core::ffi::c_void);

        if dwSrcEncoding == CP_UNICODE {
            let mut cSrcSizeW: i32 = 0;
            let pcSrcSize_used = if !pcSrcSize.is_null() && *pcSrcSize != -1 {
                cSrcSizeW = *pcSrcSize / 2;
                core::ptr::addr_of_mut!(cSrcSizeW)
            } else {
                pcSrcSize
            };
            return ConvertINetUnicodeToMultiByte(
                pdwMode,
                dwDstEncoding,
                pSrcStr as *const u16,
                pcSrcSize_used,
                pDstStr,
                pcDstSize,
            );
        } else if dwDstEncoding == CP_UNICODE {
            let hr = ConvertINetMultiByteToUnicode(
                pdwMode,
                dwSrcEncoding,
                pSrcStr,
                pcSrcSize,
                pDstStr as *mut u16,
                pcDstSize,
            );
            *pcDstSize *= 2;
            return hr;
        } else {
            let mut cDstSizeW: i32 = 0;
            let pDstStrW: *mut u16;
            let mut hr: HRESULT;

            let ds2 = debugstr_a(pSrcStr);
            TRACE!(b"convert %s from %d to %d\n\0".as_ptr() as *const i8, ds2, dwSrcEncoding as i32, dwDstEncoding as i32);

            hr = ConvertINetMultiByteToUnicode(
                pdwMode,
                dwSrcEncoding,
                pSrcStr,
                pcSrcSize,
                core::ptr::null_mut(),
                &mut cDstSizeW,
            );
            if hr != S_OK {
                return hr;
            }

            pDstStrW = HeapAlloc(GetProcessHeap(), 0, cDstSizeW as usize * 2) as *mut u16;
            hr = ConvertINetMultiByteToUnicode(
                pdwMode,
                dwSrcEncoding,
                pSrcStr,
                pcSrcSize,
                pDstStrW,
                &mut cDstSizeW,
            );
            if hr == S_OK {
                hr = ConvertINetUnicodeToMultiByte(
                    pdwMode,
                    dwDstEncoding,
                    pDstStrW as *const u16,
                    &mut cDstSizeW,
                    pDstStr,
                    pcDstSize,
                );
            }

            HeapFree(GetProcessHeap(), 0, pDstStrW as *mut core::ffi::c_void);
            return hr;
        }
    }
}

unsafe extern "C" fn fnIMultiLanguage3_ConvertStringFromUnicode(
    iface: *mut IMultiLanguage3,
    pdwMode: *mut u32,
    dwEncoding: u32,
    pSrcStr: *mut u16,
    pcSrcSize: *mut u32,
    pDstStr: *mut i8,
    pcDstSize: *mut u32,
) -> i32 {
    ConvertINetUnicodeToMultiByte(pdwMode, dwEncoding, pSrcStr, pcSrcSize as *mut i32, pDstStr as *mut u8, pcDstSize as *mut i32)
}

unsafe extern "C" fn fnIMultiLanguage3_ConvertStringFromUnicodeEx(
    This: *mut IMultiLanguage3,
    pdwMode: *mut u32,
    dwEncoding: u32,
    pSrcStr: *mut u16,
    pcSrcSize: *mut u32,
    pDstStr: *mut i8,
    pcDstSize: *mut u32,
    dwFlag: u32,
    lpFallBack: *mut u16,
) -> HRESULT {
    FIXME(b"\n\0".as_ptr() as *const i8);
    ConvertINetUnicodeToMultiByte(pdwMode, dwEncoding, pSrcStr, pcSrcSize as *mut i32, pDstStr as *mut u8, pcDstSize as *mut i32)
}

unsafe extern "C" fn MLangConvertCharset_DoConversionFromUnicode(
    iface: *mut IMLangConvertCharset,
    src: *mut u16,
    src_size: *mut u32,
    dest: *mut i8,
    dest_size: *mut u32,
) -> HRESULT {
    extern "C" {
    }
    extern "system" {
        fn ConvertINetUnicodeToMultiByte(
            lpMode: *const u32,
            dwEncoding: u32,
            lpSrcStr: *const u16,
            lpnSrcCount: *mut i32,
            lpDstStr: *mut i8,
            lpnDstCount: *mut i32,
        ) -> HRESULT;
    }
    let This: *mut convert_charset = impl_from_IMLangConvertCharset(iface);
    TRACE!(b"(%p)->(%p %p %p %p)\n\0".as_ptr() as *const i8, This, src, src_size, dest, dest_size);
    return ConvertINetUnicodeToMultiByte(
        core::ptr::null(),
        (*This).dst_cp,
        src,
        src_size as *mut i32,
        dest,
        dest_size as *mut i32,
    );
}

unsafe extern "C" fn fnIMultiLanguage3_ConvertString(
    iface: *mut IMultiLanguage3,
    pdwMode: *mut u32,
    dwSrcEncoding: u32,
    dwDstEncoding: u32,
    pSrcStr: *mut u8,
    pcSrcSize: *mut u32,
    pDstStr: *mut u8,
    pcDstSize: *mut u32,
) -> HRESULT {
    ConvertINetString(
        pdwMode,
        dwSrcEncoding,
        dwDstEncoding,
        pSrcStr as *const i8,
        pcSrcSize as *mut i32,
        pDstStr as *mut i8,
        pcDstSize as *mut i32,
    )
}

#[no_mangle]
pub unsafe extern "C" fn fnIMultiLanguage3_ConvertStringInIStream(
    iface: *mut IMultiLanguage3,
    pdwMode: *mut u32,
    dwFlag: u32,
    lpFallBack: *const u16,
    dwSrcEncoding: u32,
    dwDstEncoding: u32,
    pstmIn: *mut IStream,
    pstmOut: *mut IStream,
) -> i32 {
    let mut src: *mut i8 = core::ptr::null_mut();
    let mut dst: *mut i8 = core::ptr::null_mut();
    let mut srclen: i32 = 0;
    let mut dstlen: i32 = 0;
    let mut stat: STATSTG = core::mem::zeroed();
    let mut hr: i32;

    TRACE!(b"%p %0x8 %s %u %u %p %p\n\0".as_ptr() as *const i8, pdwMode as *mut core::ffi::c_void, dwFlag, debugstr_w(lpFallBack), dwSrcEncoding, dwDstEncoding, pstmIn as *mut core::ffi::c_void, pstmOut as *mut core::ffi::c_void);

    FIXME(b"dwFlag and lpFallBack not handled\n\0".as_ptr() as *const i8);

    hr = IStream_Stat(pstmIn, &mut stat, STATFLAG_NONAME);
    if hr < 0 {
        return hr;
    }

    if (stat.cbSize as i64) > MAXLONG {
        return E_INVALIDARG;
    }

    src = HeapAlloc(GetProcessHeap(), 0, stat.cbSize as usize) as *mut i8;
    if src.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = 'exit: loop {
        let mut tmp_hr;

        tmp_hr = IStream_Read(
            pstmIn,
            src as *mut core::ffi::c_void,
            stat.cbSize as u32,
            &mut srclen as *mut i32 as *mut u32,
        );
        if tmp_hr < 0 {
            break 'exit tmp_hr;
        }

        tmp_hr = ConvertINetString(
            pdwMode,
            dwSrcEncoding,
            dwDstEncoding,
            src,
            &mut srclen,
            core::ptr::null_mut(),
            &mut dstlen,
        );
        if tmp_hr < 0 {
            break 'exit tmp_hr;
        }

        dst = HeapAlloc(GetProcessHeap(), 0, dstlen as usize) as *mut i8;
        if dst.is_null() {
            tmp_hr = E_OUTOFMEMORY;
            break 'exit tmp_hr;
        }

        tmp_hr = ConvertINetString(
            pdwMode,
            dwSrcEncoding,
            dwDstEncoding,
            src,
            &mut srclen,
            dst,
            &mut dstlen,
        );
        if tmp_hr < 0 {
            break 'exit tmp_hr;
        }

        tmp_hr = IStream_Write(
            pstmOut,
            dst as *const core::ffi::c_void,
            dstlen as u32,
            core::ptr::null_mut(),
        );
        break 'exit tmp_hr;
    };

    HeapFree(GetProcessHeap(), 0, src as *mut core::ffi::c_void);
    HeapFree(GetProcessHeap(), 0, dst as *mut core::ffi::c_void);
    hr
}

#[no_mangle]
pub unsafe extern "system" fn ConvertINetMultiByteToUnicode(
    pdwMode: LPDWORD,
    dwEncoding: DWORD,
    pSrcStr: LPCSTR,
    pcSrcSize: LPINT,
    pDstStr: LPWSTR,
    pcDstSize: LPINT,
) -> HRESULT {
    unsafe {
        let mut src_len: i32 = -1;

        TRACE!(b"%p %d %s %p %p %p\n\0" as *const _ as *const i8, pdwMode as *const core::ffi::c_void, dwEncoding, debugstr_a(pSrcStr), pcSrcSize as *const core::ffi::c_void, pDstStr as *const core::ffi::c_void, pcDstSize as *const core::ffi::c_void);

        if pcDstSize.is_null() {
            return E_FAIL;
        }

        let mut pcSrcSize = if pcSrcSize.is_null() {
            core::ptr::addr_of_mut!(src_len)
        } else {
            pcSrcSize
        };

        if *pcSrcSize == 0 {
            *pcDstSize = 0;
            return S_OK;
        }

        let mut dwEncoding = dwEncoding;
        if dwEncoding == 51932 {
            dwEncoding = 20932;
        }

        match dwEncoding {
            CP_UNICODE => {
                if *pcSrcSize == -1 {
                    *pcSrcSize = lstrlenW(pSrcStr as *const u16);
                }
                *pcDstSize = core::cmp::min(*pcSrcSize, *pcDstSize);
                *pcSrcSize = (*pcSrcSize) * core::mem::size_of::<u16>() as i32;
                if !pDstStr.is_null() {
                    memmove(
                        pDstStr as *mut core::ffi::c_void,
                        pSrcStr as *const core::ffi::c_void,
                        (*pcDstSize as usize) * core::mem::size_of::<u16>(),
                    );
                }
            }
            50220 | 50221 | 50222 => {
                *pcDstSize = ConvertJISJapaneseToUnicode(
                    pSrcStr,
                    *pcSrcSize as u32,
                    pDstStr,
                    *pcDstSize as u32,
                ) as i32;
            }
            50932 => {
                *pcDstSize = ConvertUnknownJapaneseToUnicode(
                    pSrcStr,
                    *pcSrcSize as u32,
                    pDstStr,
                    *pcDstSize as u32,
                ) as i32;
            }
            _ => {
                if *pcSrcSize == -1 {
                    *pcSrcSize = lstrlenA(pSrcStr);
                }
                if !pDstStr.is_null() {
                    *pcDstSize = MultiByteToWideChar(
                        dwEncoding,
                        0,
                        pSrcStr,
                        *pcSrcSize,
                        pDstStr,
                        *pcDstSize,
                    );
                } else {
                    *pcDstSize = MultiByteToWideChar(
                        dwEncoding,
                        0,
                        pSrcStr,
                        *pcSrcSize,
                        core::ptr::null_mut(),
                        0,
                    );
                }
            }
        }

        if *pcDstSize == 0 {
            return E_FAIL;
        }

        S_OK
    }
}

unsafe extern "C" fn fnIMultiLanguage_GetCharsetInfo(
    iface: *mut IMultiLanguage,
    Charset: *mut u16,
    pCharsetInfo: *mut MIMECSETINFO,
) -> i32 {
    let This = impl_from_IMultiLanguage(iface);
    return IMultiLanguage3_GetCharsetInfo(
        core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface),
        Charset,
        pCharsetInfo,
    );
}



// ==== COM vtable-dispatch wrappers (wine ICOM_CALL macros the translation left undefined) ====
#[no_mangle]
pub unsafe extern "system" fn IEnumRfc1766_Next(iface: *mut IEnumRfc1766, celt: u32, rgelt: *mut RFC1766INFO, pceltFetched: *mut u32) -> i32 {
    ((*(*iface).lpVtbl).Next)(iface, celt, rgelt, pceltFetched)
}
#[no_mangle]
pub unsafe extern "system" fn IClassFactory_AddRef(iface: *mut IClassFactory) -> u32 {
    ((*(*iface).lpVtbl).AddRef)(iface)
}
#[no_mangle]
pub unsafe extern "system" fn IMultiLanguage3_IsConvertible(iface: *mut IMultiLanguage3, src_enc: u32, dst_enc: u32) -> HRESULT {
    ((*(*iface).lpVtbl).IsConvertible)(iface, src_enc, dst_enc)
}
#[no_mangle]
pub unsafe extern "system" fn IMultiLanguage3_ValidateCodePageEx( iface: *mut IMultiLanguage3, uiCodePage: u32, hwnd: isize, dwFlags: u32, ) -> i32 {
    ((*(*iface).lpVtbl).ValidateCodePageEx)(iface, uiCodePage, hwnd as HWND, dwFlags)
}
#[no_mangle]
pub unsafe extern "system" fn IMultiLanguage3_EnumRfc1766(iface: *mut IMultiLanguage3, langid: u16, ppEnum: *mut *mut IEnumRfc1766) -> i32 {
    ((*(*iface).lpVtbl).EnumRfc1766)(iface, langid, ppEnum)
}
#[no_mangle]
pub unsafe extern "system" fn IEnumRfc1766_Release(This: *mut IEnumRfc1766) -> u32 {
    ((*(*This).lpVtbl).Release)(This)
}
#[no_mangle]
pub unsafe extern "system" fn IUnknown_QueryInterface(punk: *mut IUnknown, riid: *const GUID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    ((*(*punk).lpVtbl).QueryInterface)(punk, riid, ppv)
}
#[no_mangle]
pub unsafe extern "system" fn IUnknown_Release(punk: *mut IUnknown) -> u32 {
    ((*(*punk).lpVtbl).Release)(punk)
}
#[no_mangle]
pub unsafe extern "system" fn IEnumScript_AddRef(iface: *mut IEnumScript) -> u32 {
    ((*(*iface).lpVtbl).AddRef)(iface)
}
#[no_mangle]
pub unsafe extern "system" fn IMultiLanguage3_QueryInterface( iface: *mut IMultiLanguage3, riid: *const IID, ppvObject: *mut *mut core::ffi::c_void, ) -> HRESULT {
    ((*(*iface).lpVtbl).QueryInterface)(iface, riid, ppvObject)
}
#[no_mangle]
pub unsafe extern "system" fn IMultiLanguage3_AddRef(iface: *mut IMultiLanguage3) -> u32 {
    ((*(*iface).lpVtbl).AddRef)(iface)
}
#[no_mangle]
pub unsafe extern "system" fn IMultiLanguage3_Release(iface: *mut IMultiLanguage3) -> u32 {
    ((*(*iface).lpVtbl).Release)(iface)
}
#[no_mangle]
pub unsafe extern "system" fn IMLangFontLink2_GetCharCodePages(iface: *mut IMLangFontLink2, ch_src: u16, codepages: *mut u32) -> i32 {
    ((*(*iface).lpVtbl).GetCharCodePages)(iface, ch_src, codepages)
}
#[no_mangle]
pub unsafe extern "system" fn IMLangFontLink2_GetStrCodePages( iface: *mut IMLangFontLink2, src: *const u16, src_len: i32, priority_cp: u32, codepages: *mut u32, ret_len: *mut i32, ) -> HRESULT {
    ((*(*iface).lpVtbl).GetStrCodePages)(iface, src, src_len, priority_cp, codepages, ret_len)
}
#[no_mangle]
pub unsafe extern "system" fn IMLangFontLink2_CodePageToCodePages( iface: *mut IMLangFontLink2, codepage: u32, codepages: *mut u32, ) -> HRESULT {
    ((*(*iface).lpVtbl).CodePageToCodePages)(iface, codepage, codepages)
}
#[no_mangle]
pub unsafe extern "system" fn IMLangFontLink2_CodePagesToCodePage( iface: *mut IMLangFontLink2, codepages: u32, def_codepage: u32, codepage: *mut u32, ) -> i32 {
    ((*(*iface).lpVtbl).CodePagesToCodePage)(iface, codepages, def_codepage, codepage)
}
#[no_mangle]
pub unsafe extern "system" fn IMLangFontLink2_GetFontCodePages( iface: *mut IMLangFontLink2, hdc: HDC, hfont: HFONT, codepages: *mut u32, ) -> i32 {
    ((*(*iface).lpVtbl).GetFontCodePages)(iface, hdc, hfont, codepages)
}
#[no_mangle]
pub unsafe extern "system" fn IMultiLanguage3_ConvertString( iface: *mut IMultiLanguage3, mode: *mut u32, src_enc: u32, dst_enc: u32, src: *mut u8, src_size: *mut u32, dest: *mut u8, dest_size: *mut u32, ) -> HRESULT {
    ((*(*iface).lpVtbl).ConvertString)(iface, mode, src_enc, dst_enc, src, src_size, dest, dest_size)
}
#[no_mangle]
pub unsafe extern "system" fn IMultiLanguage3_ConvertStringToUnicode(iface: *mut IMultiLanguage3, mode: *mut DWORD, src_enc: DWORD, src: *mut CHAR, src_size: *mut UINT, dest: *mut WCHAR, dest_size: *mut UINT) -> HRESULT {
    ((*(*iface).lpVtbl).ConvertStringToUnicode)(iface, mode, src_enc, src, src_size, dest, dest_size)
}
#[no_mangle]
pub unsafe extern "system" fn IMultiLanguage3_ConvertStringFromUnicode( iface: *mut IMultiLanguage3, mode: *mut DWORD, encoding: DWORD, src: *mut WCHAR, src_size: *mut UINT, dest: *mut CHAR, dest_size: *mut UINT, ) -> HRESULT {
    ((*(*iface).lpVtbl).ConvertStringFromUnicode)(iface, mode, encoding, src, src_size, dest, dest_size)
}
#[no_mangle]
pub unsafe extern "system" fn IMultiLanguage3_ConvertStringReset(iface: *mut IMultiLanguage3) -> HRESULT {
    ((*(*iface).lpVtbl).ConvertStringReset)(iface)
}
#[no_mangle]
pub unsafe extern "system" fn IMultiLanguage3_GetRfc1766FromLcid( iface: *mut IMultiLanguage3, lcid: LCID, pbstrRfc1766: *mut BSTR, ) -> HRESULT {
    ((*(*iface).lpVtbl).GetRfc1766FromLcid)(iface, lcid, pbstrRfc1766)
}
#[no_mangle]
pub unsafe extern "system" fn IMultiLanguage3_GetLcidFromRfc1766( pIMultiLanguage3: *mut IMultiLanguage3, locale: *mut LCID, rfc1766: BSTR, ) -> HRESULT {
    ((*(*pIMultiLanguage3).lpVtbl).GetLcidFromRfc1766)(pIMultiLanguage3, locale, rfc1766)
}
#[no_mangle]
pub unsafe extern "system" fn IMultiLanguage3_CreateConvertCharset(iface: *mut IMultiLanguage3, src_cp: u32, dst_cp: u32, prop: u32, convert_charset: *mut *mut IMLangConvertCharset) -> i32 {
    ((*(*iface).lpVtbl).CreateConvertCharset)(iface, src_cp, dst_cp, prop, convert_charset)
}
#[no_mangle]
pub unsafe extern "system" fn IEnumRfc1766_AddRef(iface: *mut IEnumRfc1766) -> u32 {
    ((*(*iface).lpVtbl).AddRef)(iface)
}
#[no_mangle]
pub unsafe extern "system" fn IMLangConvertCharset_AddRef(iface: *mut IMLangConvertCharset) -> u32 {
    ((*(*iface).lpVtbl).AddRef)(iface)
}
#[no_mangle]
pub unsafe extern "system" fn IMultiLanguage3_GetCharsetInfo( iface: *mut IMultiLanguage3, Charset: *mut u16, pCharsetInfo: *mut MIMECSETINFO, ) -> i32 {
    ((*(*iface).lpVtbl).GetCharsetInfo)(iface, Charset, pCharsetInfo)
}
#[no_mangle]
pub unsafe extern "system" fn IMultiLanguage3_GetNumberOfCodePageInfo(iface: *mut IMultiLanguage3, pc: *mut UINT) -> HRESULT {
    ((*(*iface).lpVtbl).GetNumberOfCodePageInfo)(iface, pc)
}
#[no_mangle]
pub unsafe extern "system" fn IEnumCodePage_AddRef(iface: *mut IEnumCodePage) -> u32 {
    ((*(*iface).lpVtbl).AddRef)(iface)
}

#[no_mangle]
pub unsafe extern "system" fn IMultiLanguage3_GetFamilyCodePage(iface: *mut IMultiLanguage3, cp: u32, family_cp: *mut u32) -> HRESULT {
    ((*(*iface).lpVtbl).GetFamilyCodePage)(iface, cp, family_cp)
}
