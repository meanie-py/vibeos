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
pub use crate::winemacros::*;
pub use crate::winemacros_gen::*;
pub use crate::commacros_gen::*;
pub use crate::debug_shims::{FAILED, SUCCEEDED};

// Cross-module unqualified calls (C TUs see each other via headers; glob
// re-exports play that role — local extern decls shadow these harmlessly).
pub use crate::hash::*;
pub use crate::oleaut::*;
pub use crate::safearray::*;
pub use crate::variant::*;
pub use crate::usrmarshal::*;
pub use crate::vartype::*;

// wine debug macro forms (TRACE!(..)-style call sites)
pub use crate::{TRACE, WARN, ERR, FIXME};
pub use crate::{ALIGN_LENGTH, ALIGN_POINTER};

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
pub type __tagBRECORD = tagVARIANT__bindgen_ty_1___tagVARIANT__bindgen_ty_1___tagBRECORD;

// RTL_CRITICAL_SECTION mirror (winnt layout; used by value in module statics).
#[repr(C)]
pub struct CRITICAL_SECTION_DEBUG {
    pub Type: u16,
    pub CreatorBackTraceIndex: u16,
    pub CriticalSection: *mut CRITICAL_SECTION,
    pub ProcessLocksList: [*mut c_void; 2],
    pub EntryCount: u32,
    pub ContentionCount: u32,
    pub Flags: u32,
    pub CreatorBackTraceIndexHigh: u16,
    pub SpareWORD: u16,
}
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

// msvcrt math the C called directly.
extern "C" {
    pub fn pow(x: f64, y: f64) -> f64;
    pub fn fmod(x: f64, y: f64) -> f64;
    pub fn sqrt(x: f64) -> f64;
    pub fn atan2(y: f64, x: f64) -> f64;
    pub fn exp(x: f64) -> f64;
    pub fn log(x: f64) -> f64;
}

// msvcrt mem/string/ctype (cdecl, exact CRT prototypes).
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
    pub fn wcslen(s: *const u16) -> usize;
}

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

// user32 / gdi32
extern "system" {
    pub fn GetSysColor(nIndex: c_int) -> DWORD;
    pub fn CopyIcon(hIcon: HICON) -> HICON;
    pub fn GetPaletteEntries(hpal: HPALETTE, iStart: UINT, cEntries: UINT,
                             pPalEntries: *mut PALETTEENTRY) -> UINT;
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
