//! Shared import surface for the rustconv-translated modules (hash/oleaut/
//! safearray/variant/vartype/usrmarshal). Their original `use ntdll_sys::nt::*;`
//! header line is swapped for `use crate::prelude::*;` at integration; com-sys
//! (bindgen over the real SDK COM headers) provides the COM/automation types
//! with exact C layouts. Lane-A modules (olesys/dispatch/typelib) carry their
//! own hand-mirrored surface and do not use this prelude — swapping them onto
//! com-sys is the queued mechanical cleanup.
pub use com_sys::*;

// Base aliases com-sys doesn't re-export under the names the translations use.
pub use core::ffi::{c_char, c_int, c_long, c_uint, c_void};

pub use crate::consts::*;
// Tier-D constants, harvested from the real SDK headers with file:line citations
// (tools/harvest_consts.py). Disjoint from consts.rs/winemacros.rs by construction —
// an overlap would be an ambiguous-glob E0659.
pub use crate::tierd_consts::*;
// IID_*/CLSID_* — DECLARED as extern statics (the `uuid` static lib defines them), never
// transcribed. See iids.rs for why a hand-typed GUID is a silent-corruption bug.
pub use crate::iids::*;
// `stdmethod!` is #[macro_export]ed (crate root), so a `use` — not the module
// path — is what brings it into scope for every file that globs the prelude.
// Mirrors ole32-rust/src/prelude.rs.
pub use crate::stdmethod;
pub use crate::winemacros::*;
pub use crate::winemacros_gen::*;
pub use crate::commacros_gen::*;
// Glob, not a 2-name list: the Tier-D modules were translated with --c2rust, which
// preprocesses the C — so wine's TRACE/FIXME macros arrive pre-expanded into the
// raw `ros_dbg_log`/`__wine_debug_channel` surface, which debug_shims now provides.
pub use crate::debug_shims::*;

// Cross-module unqualified calls (C TUs see each other via headers; glob
// re-exports play that role — local extern decls shadow these harmlessly).
pub use crate::hash::*;
pub use crate::oleaut::*;
pub use crate::safearray::*;
pub use crate::variant::*;
pub use crate::usrmarshal::*;
pub use crate::vartype::*;
// Tier-D (the last six C TUs). They call each other across TU boundaries just as
// the C did — olepicture.c calls connpt.c's CreateConnectionPoint, olefont.c and
// olepropframe.c reach into each other — so they need the same glob treatment.
pub use crate::connpt::*;
pub use crate::olefont::*;
pub use crate::olepicture::*;
pub use crate::olepropframe::*;
pub use crate::recinfo::*;
pub use crate::varformat::*;

// wine debug macro forms (TRACE!(..)-style call sites)
pub use crate::{TRACE, WARN, ERR, FIXME, MESSAGE};
pub use crate::{ALIGN_LENGTH, ALIGN_POINTER};
pub use crate::ARRAY_SIZE;

// ---------------------------------------------------------------------------
// win_sys — the non-COM Win32 ABI island (bindgen; see generate_win_bindings.sh).
//
// Re-exported BY NAME, never `pub use crate::win_sys::*`: bindgen has to re-emit
// wincodec/prsht's transitive base surface (ULONG, VARIANT, SAFEARRAY, TYPEATTR, …),
// which com-sys already supplies, so a glob would make every overlapping name an
// ambiguous-glob error. Only the names Tier-D actually references cross over.
// ---------------------------------------------------------------------------
// olectl.h — IPicture/IFont/property-frame entry-point structs
pub use crate::win_sys::{FONTDESC, OCPFIPARAMS, PICTDESC};
// prsht.h — olepropframe.c builds a real comctl32 property sheet
pub use crate::win_sys::{HPROPSHEETPAGE, PROPSHEETHEADERW, PROPSHEETPAGEW};
// wingdi.h / winuser.h
pub use crate::win_sys::{BITMAPFILEHEADER, DLGTEMPLATE, ENHMETAHEADER};
pub use crate::win_sys::{HGDIOBJ, HMETAFILE, HMODULE, HRSRC};
/// wingdi.h `typedef struct tagBITMAP { ... } BITMAP;` — bindgen keeps the C tag name.
pub use crate::win_sys::tagBITMAP as BITMAP;
// wincodec.h — olepicture.c's WIC decode path (oleaut32 delay-imports windowscodecs).
// The whole interface family, not just the five olepicture names directly: the generated
// COBJMACRO helpers in commacros_*.rs spell the full vtbl signatures, and e.g.
// `IWICImagingFactory::CreateBitmapScaler` returns an `IWICBitmapScaler**`.
pub use crate::win_sys::{
    IWICBitmap, IWICBitmapClipper, IWICBitmapDecoder, IWICBitmapDecoderInfo, IWICBitmapEncoder,
    IWICBitmapEncoderInfo, IWICBitmapFlipRotator, IWICBitmapFrameDecode, IWICBitmapFrameEncode,
    IWICBitmapLock, IWICBitmapScaler, IWICBitmapSource, IWICColorContext, IWICColorTransform,
    IWICComponentInfo, IWICFastMetadataEncoder, IWICFormatConverter, IWICImagingFactory,
    IWICMetadataQueryReader, IWICMetadataQueryWriter, IWICPalette, IWICStream, WICRect,
};
/// wincodec.h `enum WICDecodeOptions`; bindgen prefixes enum variants with the enum name.
pub use crate::win_sys::WICDecodeOptions_WICDecodeMetadataCacheOnLoad as WICDecodeMetadataCacheOnLoad;
// The WIC scalar/enum aliases the generated vtbl signatures spell.
pub use crate::win_sys::{
    REFWICPixelFormatGUID, WICBitmapAlphaChannelOption, WICBitmapCreateCacheOption,
    WICBitmapDitherType, WICBitmapEncoderCacheOption, WICBitmapInterpolationMode,
    WICBitmapPaletteType, WICBitmapPattern, WICBitmapTransformOptions, WICColor,
    WICColorContextType, WICComponentType, WICDecodeOptions, WICPixelFormatGUID,
};

// Type aliases the translations use that com-sys doesn't export by name.
pub type LPVARIANT = *mut VARIANT;
pub type LPDISPATCH = *mut IDispatch;
pub type LPUNKNOWN = *mut IUnknown;
pub type LPFUNCDESC = *mut FUNCDESC;
pub type LPVARDESC = *mut VARDESC;
pub type LPTYPEATTR = *mut TYPEATTR;
pub type LPTLIBATTR = *mut TLIBATTR;
pub type LPMONIKER = *mut IMoniker;
pub type LPRUNNINGOBJECTTABLE = *mut IRunningObjectTable;
pub type REFIID = *const IID;
pub type REFGUID = *const GUID;
pub type REFCLSID = *const CLSID;
pub type LONG64 = i64;
pub type ULONG64 = u64;
pub type COLORREF = u32;
pub type LCTYPE = u32;
pub type REGSAM = u32;
pub type HKEY = *mut c_void;
pub type HINSTANCE = *mut c_void;
pub type HCURSOR = *mut c_void;

// ---- Tier-D aliases. Each maps to a type that ALREADY EXISTS (com-sys's bindgen spelling
// or the win_sys island); none of these invents a struct. An invented struct with the wrong
// layout is a silent memory-corruption bug, so where a layout was needed it was generated
// (win_sys), not written. ----
/// windef.h — `typedef unsigned char BYTE; typedef BYTE *LPBYTE;`
pub type LPBYTE = *mut u8;
/// winnt.h — `typedef CONST CHAR *PCSTR;`
pub type PCSTR = *const c_char;
/// crtdefs.h — `size_t` is the pointer-sized unsigned type
pub type size_t = usize;

/// ocidl.h — com-sys emits this under the C tag name (`tagCONNECTDATA`).
pub type CONNECTDATA = tagCONNECTDATA;
pub type LPCONNECTDATA = *mut CONNECTDATA;
pub type LPENUMCONNECTIONS = *mut IEnumConnections;
pub type LPCLASSFACTORY = *mut IClassFactory;
pub type LPPERSISTSTREAM = *mut IPersistStream;
/// olectl.h — `typedef PICTDESC *LPPICTDESC;` (PICTDESC/FONTDESC come from win_sys)
pub type LPPICTDESC = *mut PICTDESC;
pub type LPFONTDESC = *mut FONTDESC;
/// prsht.h — `typedef PROPSHEETPAGEW *LPPROPSHEETPAGEW;`
pub type LPPROPSHEETPAGEW = *mut PROPSHEETPAGEW;
/// wingdi.h — `typedef TEXTMETRICW *LPTEXTMETRICW;` (TEXTMETRICW is com-sys's)
pub type LPTEXTMETRICW = *mut TEXTMETRICW;
pub type __tagBRECORD = tagVARIANT__bindgen_ty_1___tagVARIANT__bindgen_ty_1___tagBRECORD;

// LIST_ENTRY (winnt layout) — the translations spell the bindgen struct tag
// with a leading underscore (`_LIST_ENTRY`); com-sys doesn't emit it (COM
// headers don't pull winnt's LIST_ENTRY), so mirror it here by value.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct _LIST_ENTRY {
    pub Flink: *mut _LIST_ENTRY,
    pub Blink: *mut _LIST_ENTRY,
}
pub type LIST_ENTRY = _LIST_ENTRY;
unsafe impl Sync for _LIST_ENTRY {}

// RTL_CRITICAL_SECTION mirror. Layout transcribed from ndk/rtltypes.h:1424-1443 — the
// trailing member is a UNION (WineDebugString / Spare[1] / {Flags, CreatorBackTraceIndexHigh,
// SpareWORD}), not the three flat fields this mirror used to carry. The flat form happened to
// have the same size, but it could not express `{ (DWORD_PTR)(__FILE__ ": name") }` — the
// wine-debug-string initializer the C actually uses (see variant.rs:32's comment, which
// noted the dropped slot) — and olefont.c's static critical section initializes exactly that
// arm. Modelled as a union so both spellings are available and the layout matches the header.
#[repr(C)]
#[derive(Clone, Copy)]
pub union RTL_CRITICAL_SECTION_DEBUG_u {
    /// ndk/rtltypes.h:1434 — the arm wine's `DECLARE_CRITICAL_SECTION` initializes
    pub WineDebugString: usize,
    /// ndk/rtltypes.h:1435
    pub Spare: [usize; 1],
    /// ndk/rtltypes.h:1436-1441
    pub s: RTL_CRITICAL_SECTION_DEBUG_flags,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RTL_CRITICAL_SECTION_DEBUG_flags {
    pub Flags: u32,
    pub CreatorBackTraceIndexHigh: u16,
    pub SpareWORD: u16,
}
/// c2rust names the anonymous union positionally; olefont.rs spells it `C2RustUnnamed_3`.
/// Aliased rather than re-declared so there is exactly one type behind both names.
pub type C2RustUnnamed_3 = RTL_CRITICAL_SECTION_DEBUG_u;

#[repr(C)]
pub struct CRITICAL_SECTION_DEBUG {
    pub Type: u16,
    pub CreatorBackTraceIndex: u16,
    pub CriticalSection: *mut CRITICAL_SECTION,
    pub ProcessLocksList: _LIST_ENTRY,
    pub EntryCount: u32,
    pub ContentionCount: u32,
    pub c2rust_unnamed: RTL_CRITICAL_SECTION_DEBUG_u,
}
pub type RTL_CRITICAL_SECTION_DEBUG = CRITICAL_SECTION_DEBUG;
pub type PRTL_CRITICAL_SECTION_DEBUG = *mut CRITICAL_SECTION_DEBUG;
#[repr(C)]
pub struct CRITICAL_SECTION {
    pub DebugInfo: *mut CRITICAL_SECTION_DEBUG,
    pub LockCount: i32,
    pub RecursionCount: i32,
    pub OwningThread: *mut c_void,
    pub LockSemaphore: *mut c_void,
    pub SpinCount: usize,
}
unsafe impl Sync for CRITICAL_SECTION {}
/// c2rust spells the tag name; same type.
pub type _RTL_CRITICAL_SECTION = CRITICAL_SECTION;

// SYSTEMTIME (used via LPSYSTEMTIME in date conversions)
#[repr(C)]
pub struct SYSTEMTIME {
    pub wYear: u16, pub wMonth: u16, pub wDayOfWeek: u16, pub wDay: u16,
    pub wHour: u16, pub wMinute: u16, pub wSecond: u16, pub wMilliseconds: u16,
}
pub type LPSYSTEMTIME = *mut SYSTEMTIME;

// ACTCTX_SECTION_KEYED_DATA (winbase.h; only lpData/ulLength are consumed by
// the typelib actctx path — full layout mirrored for ABI safety).
#[repr(C)]
pub struct ACTCTX_SECTION_KEYED_DATA {
    pub cbSize: u32,
    pub ulDataFormatVersion: u32,
    pub lpData: *mut c_void,
    pub ulLength: u32,
    pub lpSectionGlobalData: *mut c_void,
    pub ulSectionGlobalDataLength: u32,
    pub lpSectionBase: *mut c_void,
    pub ulSectionTotalLength: u32,
    pub hActCtx: *mut c_void,
    pub ulAssemblyRosterIndex: u32,
    pub ulFlags: u32,
    pub AssemblyMetadata: [usize; 8],
}

// com-sys lacks the const-char string alias (headers only produced LPSTR).
pub type LPCSTR = *const c_char;

// Integration aliases: the translations spell these bindgen struct-tags /
// enum-qualified constants with the C tag name; map to the com-sys/consts names.
pub type _CY = CY;
pub type _DECIMAL = DECIMAL;
pub const _VARENUM_VT_I4: u16 = VT_I4;

// C `max(a, b)` (windef.h macro) → generic free fn (usrmarshal wire sizing).
#[inline]
pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a >= b { a } else { b }
}

// msvcrt math the C called directly.
extern "C" {
    pub fn pow(x: f64, y: f64) -> f64;
    pub fn fmod(x: f64, y: f64) -> f64;
    pub fn sqrt(x: f64) -> f64;
    pub fn atan2(y: f64, x: f64) -> f64;
    pub fn exp(x: f64) -> f64;
    pub fn log(x: f64) -> f64;
    pub fn ceil(x: f64) -> f64;
    pub fn floor(x: f64) -> f64;
}

// msvcrt mem/string/ctype (cdecl, exact CRT prototypes).
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
    pub fn wcslen(s: *const u16) -> usize;

    // wctype.h:103-104 — `wint_t __cdecl towupper(wint_t)`. wint_t is `unsigned short`
    // (vcruntime/crtdefs.h:216), NOT int. cdecl, so `extern "C"` is correct here — unlike
    // the Win32 imports below, which are all WINAPI/__stdcall.
    pub fn towupper(c: wint_t) -> wint_t;
    pub fn towlower(c: wint_t) -> wint_t;

    // wchar.h:1078-1084 — the MS non-conforming form: `int __cdecl swprintf(wchar_t*,
    // const wchar_t*, ...)` (no count parameter), which is what varformat.c calls.
    pub fn swprintf(buffer: *mut u16, format: *const u16, ...) -> c_int;
}

/// vcruntime/crtdefs.h:216 — `typedef unsigned short wint_t;`
pub type wint_t = u16;

// rpcrt4 (usrmarshal raises NDR exceptions)
extern "system" {
    pub fn RpcRaiseException(exception: u32);
}

// kernel32 (WINAPI = extern "system"; prototypes per sdk/include/psdk).
extern "system" {
    pub fn MultiByteToWideChar(CodePage: UINT, dwFlags: DWORD, lpMultiByteStr: LPCSTR,
                               cbMultiByte: c_int, lpWideCharStr: LPWSTR, cchWideChar: c_int) -> c_int;
    pub fn WideCharToMultiByte(CodePage: UINT, dwFlags: DWORD, lpWideCharStr: LPCWSTR,
                               cchWideChar: c_int, lpMultiByteStr: LPSTR, cbMultiByte: c_int,
                               lpDefaultChar: LPCSTR, lpUsedDefaultChar: *mut BOOL) -> c_int;
    pub fn GetLocaleInfoW(Locale: LCID, LCType: LCTYPE, lpLCData: LPWSTR, cchData: c_int) -> c_int;
    pub fn ConvertDefaultLocale(Locale: LCID) -> LCID;
    pub fn GetVersion() -> DWORD;
    pub fn GetCurrentProcess() -> HANDLE;
    pub fn IsWow64Process(hProcess: HANDLE, Wow64Process: *mut BOOL) -> BOOL;
    pub fn GetEnvironmentVariableW(lpName: LPCWSTR, lpBuffer: LPWSTR, nSize: DWORD) -> DWORD;
    pub fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> *mut c_void;
    pub fn HeapReAlloc(hHeap: HANDLE, dwFlags: DWORD, lpMem: *mut c_void, dwBytes: SIZE_T) -> *mut c_void;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: *mut c_void) -> BOOL;
    pub fn InitializeCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn DeleteCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn EnterCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn LeaveCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn FindActCtxSectionGuid(dwFlags: DWORD, lpExtensionGuid: *const GUID, ulSectionId: ULONG,
                                 lpGuidToFind: *const GUID,
                                 ReturnedData: *mut ACTCTX_SECTION_KEYED_DATA) -> BOOL;
    pub fn lstrlenW(lpString: LPCWSTR) -> c_int;
}

// ---------------------------------------------------------------------------
// user32 / gdi32 / kernel32 / ole32 — THE CARDINAL RULE LIVES HERE.
//
// Every one of these is WINAPI, i.e. __stdcall on i386. They MUST be `extern "system"`.
// An `extern "C"` declaration of a stdcall import links on amd64/arm64 (both are cdecl
// there) and corrupts the stack on i386 — the caller cleans args the callee already
// popped. It is the project's #1 historical crash class and it is SILENT on two of three
// arches. The CRT block above (`memcpy`, `towupper`, `swprintf`, …) is `extern "C"`
// because those genuinely ARE cdecl; the split is not stylistic.
//
// Prototypes taken from sdk/include/psdk/{wingdi,winuser,winbase,winnls,objbase}.h.
// ---------------------------------------------------------------------------
extern "system" {
    pub fn GetSysColor(nIndex: c_int) -> DWORD;
    pub fn CopyIcon(hIcon: HICON) -> HICON;
    pub fn GetPaletteEntries(hpal: HPALETTE, iStart: UINT, cEntries: UINT,
                             pPalEntries: *mut PALETTEENTRY) -> UINT;

    // --- gdi32: DC + object management ---
    pub fn GetDC(hWnd: HWND) -> HDC;
    pub fn ReleaseDC(hWnd: HWND, hDC: HDC) -> c_int;
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC;
    pub fn DeleteDC(hdc: HDC) -> BOOL;
    pub fn GetDeviceCaps(hdc: HDC, nIndex: c_int) -> c_int;
    pub fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
    pub fn DeleteObject(ho: HGDIOBJ) -> BOOL;
    pub fn GetCurrentObject(hdc: HDC, r#type: UINT) -> HGDIOBJ;
    pub fn GetObjectW(h: HGDIOBJ, c: c_int, pv: LPVOID) -> c_int;
    pub fn BitBlt(hdc: HDC, x: c_int, y: c_int, cx: c_int, cy: c_int,
                  hdcSrc: HDC, x1: c_int, y1: c_int, rop: DWORD) -> BOOL;

    // --- gdi32: bitmaps ---
    pub fn CreateBitmap(nWidth: c_int, nHeight: c_int, nPlanes: UINT,
                        nBitCount: UINT, lpBits: *const c_void) -> HBITMAP;
    pub fn CreateDIBitmap(hdc: HDC, pbmih: *const c_void, flInit: DWORD,
                          pjBits: *const c_void, pbmi: *const c_void, iUsage: UINT) -> HBITMAP;
    pub fn CreateDIBSection(hdc: HDC, pbmi: *const c_void, usage: UINT,
                            ppvBits: *mut *mut c_void, hSection: HANDLE,
                            offset: DWORD) -> HBITMAP;

    // --- gdi32: metafiles ---
    pub fn DeleteEnhMetaFile(hmf: HENHMETAFILE) -> BOOL;
    pub fn DeleteMetaFile(hmf: HMETAFILE) -> BOOL;
    pub fn PlayEnhMetaFile(hdc: HDC, hmf: HENHMETAFILE, lprect: *const RECT) -> BOOL;
    pub fn PlayMetaFile(hdc: HDC, hmf: HMETAFILE) -> BOOL;

    // --- user32 ---
    pub fn GetClientRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
    pub fn DestroyIcon(hIcon: HICON) -> BOOL;
    pub fn CreateIconFromResourceEx(presbits: *mut u8, dwResSize: DWORD, fIcon: BOOL,
                                    dwVer: DWORD, cxDesired: c_int, cyDesired: c_int,
                                    Flags: UINT) -> HICON;
    // NOTE: GetWindowLongPtrW/SetWindowLongPtrW are declared per-arch BELOW, not here —
    // on 32-bit they are not exports at all (winuser.h:4930 `#define`s them onto the
    // non-Ptr forms). See the shim after this block.

    // --- kernel32: file + memory + locale ---
    pub fn GetLastError() -> DWORD;
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn CreateFileW(lpFileName: LPCWSTR, dwDesiredAccess: DWORD, dwShareMode: DWORD,
                       lpSecurityAttributes: *mut c_void, dwCreationDisposition: DWORD,
                       dwFlagsAndAttributes: DWORD, hTemplateFile: HANDLE) -> HANDLE;
    pub fn ReadFile(hFile: HANDLE, lpBuffer: LPVOID, nNumberOfBytesToRead: DWORD,
                    lpNumberOfBytesRead: *mut DWORD, lpOverlapped: *mut c_void) -> BOOL;
    pub fn GetFileSize(hFile: HANDLE, lpFileSizeHigh: *mut DWORD) -> DWORD;
    pub fn GlobalAlloc(uFlags: UINT, dwBytes: SIZE_T) -> HGLOBAL;
    pub fn GlobalFree(hMem: HGLOBAL) -> HGLOBAL;
    pub fn CompareStringW(Locale: LCID, dwCmpFlags: DWORD, lpString1: LPCWSTR, cchCount1: c_int,
                          lpString2: LPCWSTR, cchCount2: c_int) -> c_int;
    pub fn GetNumberFormatW(Locale: LCID, dwFlags: DWORD, lpValue: LPCWSTR,
                            lpFormat: *const c_void, lpNumberStr: LPWSTR,
                            cchNumber: c_int) -> c_int;

    // --- ole32 (STDAPI == __stdcall) ---
    pub fn CoInitialize(pvReserved: LPVOID) -> HRESULT;
    pub fn CoUninitialize();
    pub fn CoCreateInstance(rclsid: REFCLSID, pUnkOuter: *mut IUnknown, dwClsContext: DWORD,
                            riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    pub fn CreateStreamOnHGlobal(hGlobal: HGLOBAL, fDeleteOnRelease: BOOL,
                                 ppstm: *mut *mut IStream) -> HRESULT;

    // --- windowscodecs (delay-imported; CMakeLists.txt:118) ---
    pub fn WICConvertBitmapSource(dstFormat: REFGUID, pISrc: *mut IWICBitmapSource,
                                  ppIDst: *mut *mut IWICBitmapSource) -> HRESULT;
}

// ---------------------------------------------------------------------------
// GetWindowLongPtrW / SetWindowLongPtrW — a MACRO on 32-bit, an export on 64-bit.
//
// winuser.h:4926-4932:
//     #ifdef _WIN64
//       LONG_PTR WINAPI GetWindowLongPtrW(HWND, int);      // a real user32 export
//     #else
//       #define GetWindowLongPtrW GetWindowLongW           // NOT an export at all
//     #endif
//
// Declaring the Ptr spelling unconditionally links on amd64/arm64 and fails i386 with
// `undefined symbol _GetWindowLongPtrW@8` — which is how the linker found this. Mirror the
// header instead: on 32-bit, forward to the non-Ptr export (LONG and LONG_PTR are both
// 32-bit there, so the conversion is exact).
// ---------------------------------------------------------------------------
#[cfg(target_arch = "x86")]
extern "system" {
    fn GetWindowLongW(hWnd: HWND, nIndex: c_int) -> LONG;
    fn SetWindowLongW(hWnd: HWND, nIndex: c_int, dwNewLong: LONG) -> LONG;
}

#[cfg(target_arch = "x86")]
#[inline]
pub unsafe fn GetWindowLongPtrW(hWnd: HWND, nIndex: c_int) -> LONG_PTR {
    GetWindowLongW(hWnd, nIndex) as LONG_PTR
}

#[cfg(target_arch = "x86")]
#[inline]
pub unsafe fn SetWindowLongPtrW(hWnd: HWND, nIndex: c_int, dwNewLong: LONG_PTR) -> LONG_PTR {
    SetWindowLongW(hWnd, nIndex, dwNewLong as LONG) as LONG_PTR
}

#[cfg(not(target_arch = "x86"))]
extern "system" {
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: c_int) -> LONG_PTR;
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: c_int, dwNewLong: LONG_PTR) -> LONG_PTR;
}

// oleaut32's OWN exports, defined elsewhere in this crate. Re-exported (not re-declared as
// externs) so the signatures are type-checked against the definitions rather than merely
// agreeing at link time.
pub use crate::dispatch::DispGetIDsOfNames;
pub use crate::typelib::{LoadRegTypeLib, LoadTypeLib};

// ---- basetsd.h:73-76 handle/int conversion macros ----
/// basetsd.h:73 — `HandleToUlong(h) ((ULONG)(ULONG_PTR)(h))`
#[inline]
pub fn HandleToUlong(h: HANDLE) -> ULONG {
    h as usize as ULONG
}
/// basetsd.h:75 — `ULongToHandle(h) ((HANDLE)(ULONG_PTR)(h))`
#[inline]
pub fn ULongToHandle(h: ULONG) -> HANDLE {
    h as usize as HANDLE
}
/// basetsd.h:76 — `LongToHandle(h) ((HANDLE)(LONG_PTR)(h))` (sign-extends, unlike the above)
#[inline]
pub fn LongToHandle(h: LONG) -> HANDLE {
    h as isize as HANDLE
}

/// winerror.h:197 — `__HRESULT_FROM_WIN32(x)`:
/// `((HRESULT)(x) > 0 ? ((HRESULT)(((x) & 0xFFFF) | (FACILITY_WIN32 << 16) | 0x80000000)) : (HRESULT)(x))`
/// FACILITY_WIN32 = 7 (winerror.h:29). Note the guard is `> 0`, not `>= 0` — S_OK (0) passes
/// through unchanged.
#[inline]
pub fn HRESULT_FROM_WIN32(x: u32) -> HRESULT {
    if (x as HRESULT) > 0 {
        ((x & 0x0000FFFF) | (7 << 16) | 0x80000000) as HRESULT
    } else {
        x as HRESULT
    }
}

// advapi32
extern "system" {
    pub fn RegOpenKeyExA(hKey: HKEY, lpSubKey: LPCSTR, ulOptions: DWORD, samDesired: REGSAM,
                         phkResult: *mut HKEY) -> LONG;
    pub fn RegCloseKey(hKey: HKEY) -> LONG;
    pub fn RegQueryValueA(hKey: HKEY, lpSubKey: LPCSTR, lpData: LPSTR, lpcbData: *mut LONG) -> LONG;
    pub fn RegQueryValueExA(hKey: HKEY, lpValueName: LPCSTR, lpReserved: *mut DWORD,
                            lpType: *mut DWORD, lpData: *mut u8, lpcbData: *mut DWORD) -> LONG;
}

// ole32
extern "system" {
    pub fn CoTaskMemAlloc(cb: SIZE_T) -> *mut c_void;
    pub fn CoTaskMemRealloc(pv: *mut c_void, cb: SIZE_T) -> *mut c_void;
    pub fn CoTaskMemFree(pv: *mut c_void);
    pub fn CoGetMalloc(dwMemContext: DWORD, ppMalloc: *mut *mut IMalloc) -> HRESULT;
    pub fn CreateItemMoniker(lpszDelim: LPCWSTR, lpszItem: LPCWSTR, ppmk: *mut *mut IMoniker) -> HRESULT;
    pub fn GetRunningObjectTable(reserved: DWORD, pprot: *mut *mut IRunningObjectTable) -> HRESULT;
    pub fn StringFromGUID2(rguid: REFGUID, lpsz: LPOLESTR, cchMax: c_int) -> c_int;
    // objidl usrmarshal helpers (__RPC_USER = stdcall), exported by ole32.
    pub fn WdtpInterfacePointer_UserSize(pFlags: *mut ULONG, RealFlags: ULONG, StartingSize: ULONG,
                                         punk: *mut IUnknown, riid: REFIID) -> ULONG;
    pub fn WdtpInterfacePointer_UserMarshal(pFlags: *mut ULONG, RealFlags: ULONG, pBuffer: *mut u8,
                                            punk: *mut IUnknown, riid: REFIID) -> *mut u8;
    pub fn WdtpInterfacePointer_UserUnmarshal(pFlags: *mut ULONG, pBuffer: *mut u8,
                                              ppunk: *mut *mut IUnknown, riid: REFIID) -> *mut u8;
    pub fn WdtpInterfacePointer_UserFree(punk: *mut IUnknown);
}

// In-DLL C-side symbols the translations call: the widl-generated proxy/stub
// unit (add_rpcproxy_files: oaidl_p.c/ocidl_p.c/dlldata.c) and tmarshal.c /
// olefont.c / olepicture.c (not translated). WINAPI/STDMETHODCALLTYPE unless
// the C declared them plain (the _get_STD*_CF pair is cdecl in oleaut.c:1056).
extern "system" {
    pub fn OLEAUTPS_DllGetClassObject(rclsid: REFCLSID, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT;
    pub fn OLEAUTPS_DllMain(hInstDll: HINSTANCE, fdwReason: DWORD, lpvReserved: *mut c_void) -> BOOL;
    pub fn OLEAUTPS_DllRegisterServer() -> HRESULT;
    pub fn OLEAUTPS_DllUnregisterServer() -> HRESULT;
    pub fn CreateProxyFromTypeInfo(typeinfo: *mut ITypeInfo, outer: *mut IUnknown, iid: REFIID,
                                   proxy: *mut *mut IRpcProxyBuffer, obj: *mut *mut c_void) -> HRESULT;
    pub fn CreateStubFromTypeInfo(typeinfo: *mut ITypeInfo, iid: REFIID, server: *mut IUnknown,
                                  stub: *mut *mut IRpcStubBuffer) -> HRESULT;
}
extern "C" {
    pub fn _get_STDFONT_CF(ppv: *mut *mut c_void);
    pub fn _get_STDPIC_CF(ppv: *mut *mut c_void);
}
