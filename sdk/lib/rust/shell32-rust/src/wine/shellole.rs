//! OLE/COM glue + drag-drop — translation of dll/win32/shell32/wine/shellole.c.
//!
//! This is the `__REACTOS__` build configuration (the big `#ifndef __REACTOS__`
//! sections — the internal CLSID InterfaceTable, DllGetClassObject, SHGetDesktopFolder,
//! and the whole IDefClF default class-factory implementation — are NOT compiled here;
//! on ReactOS `DllGetClassObject` lives in the C++ side and is imported). All 19 exports
//! are WINAPI → `extern "system"` → `_Name@N` on i386. Confirmed against the pre-flip
//! object's symbol table.
//!
//! `SHFree@4`/`SHAlloc@4` are also *imported* by the already-Rust wine/clipboard.rs;
//! these definitions keep exactly that decoration so the cross-reference still resolves.

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::types::*;
use core::ffi::c_void;
use core::mem::{size_of, transmute, zeroed};
use core::ptr::{null, null_mut, write_volatile};

// ── constants pinned from SDK headers ───────────────────────────────────────────────────
const S_OK_: HRESULT = 0; // winerror.h (types.rs has S_OK too; kept local for clarity)
const E_POINTER: HRESULT = 0x80004003u32 as HRESULT; // winerror.h
const E_INVALIDARG: HRESULT = 0x80070057u32 as HRESULT; // winerror.h
const E_ACCESSDENIED: HRESULT = 0x80070005u32 as HRESULT; // winerror.h
const E_FAIL: HRESULT = 0x80004005u32 as HRESULT; // winerror.h
const REGDB_E_CLASSNOTREG: HRESULT = 0x80040154u32 as HRESULT; // winerror.h
const CO_E_CLASSSTRING: DWORD = 0x800401F3; // winerror.h

const KEY_READ: REGSAM = 0x20019; // winnt.h
const LOAD_WITH_ALTERED_SEARCH_PATH: DWORD = 0x00000008; // winbase.h
const CLSCTX_INPROC_SERVER: DWORD = 0x00000001; // wtypesbase.h (CLSCTX enum)
const MEMCTX_TASK: DWORD = 1; // objidlbase.h (MEMCTX enum)
const CP_ACP: UINT = 0; // winnls.h

const GWL_EXSTYLE: INT = -20; // winuser.h
const WS_EX_ACCEPTFILES: LONG = 16; // winuser.h (0x00000010)

const MAX_PATH: usize = 260; // minwindef.h

// SHPropStgCreate dispositions (winbase.h)
const CREATE_NEW: DWORD = 1;
const CREATE_ALWAYS: DWORD = 2;
const OPEN_EXISTING: DWORD = 3;
const OPEN_ALWAYS: DWORD = 4;

const PRSPEC_PROPID: ULONG = 1; // propidl.idl
const PID_CODEPAGE: PROPID = 1; // msidefs.h / propidl
const VT_I2: VARTYPE = 2; // wtypes.idl

// ── file-local types not yet in types.rs ───────────────────────────────────────────────
pub type SIZE_T = usize; // basetsd.h
pub type ULONG = u32; // winnt.h
pub type PROPID = ULONG; // wtypes.h
pub type VARTYPE = u16; // wtypes.h
pub type HGLOBAL = HANDLE; // winnt.h
pub type HDROP = HANDLE; // shellapi.h
pub type HMODULE = HANDLE; // minwindef.h
pub type LPUNKNOWN = *mut c_void; // unknwn.h (IUnknown opaque here)

/// GUID / CLSID / IID / FMTID — { Data1: u32; Data2: u16; Data3: u16; Data4: [u8;8] } (guiddef.h)
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
pub type FMTID = GUID;
pub type REFIID = *const IID;
pub type REFCLSID = *const CLSID;
pub type REFFMTID = *const FMTID;

/// POINT (windef.h): { LONG x; LONG y; }
#[repr(C)]
#[derive(Copy, Clone)]
pub struct POINT {
    pub x: LONG,
    pub y: LONG,
}

/// DROPFILES (shlobj.h): { DWORD pFiles; POINT pt; BOOL fNC; BOOL fWide; }
#[repr(C)]
pub struct DROPFILES {
    pub pFiles: DWORD,
    pub pt: POINT,
    pub fNC: BOOL,
    pub fWide: BOOL,
}

/// IMalloc — only ever handed back through SHGetMalloc, never dereferenced here. Opaque.
#[repr(C)]
pub struct IMalloc {
    _private: [u8; 0],
}
pub type LPMALLOC = *mut IMalloc;

/// IMoniker — opaque; only an out-pointer slot for SHCreateQueryCancelAutoPlayMoniker.
#[repr(C)]
pub struct IMoniker {
    _private: [u8; 0],
}

// ── PROPSPEC / PROPVARIANT / STATPROPSETSTG (propidl.idl) ────────────────────────────────
// NONAMELESSUNION is defined → unions are accessed via the `u` field, matching the C.
#[repr(C)]
#[derive(Copy, Clone)]
pub union PROPSPEC_u {
    pub propid: PROPID,
    pub lpwstr: LPWSTR, // forces pointer size/alignment (8 on x64) like the C union
}
/// PROPSPEC: { ULONG ulKind; union { PROPID propid; LPOLESTR lpwstr; } u; }
#[repr(C)]
pub struct PROPSPEC {
    pub ulKind: ULONG,
    pub u: PROPSPEC_u,
}

/// BLOB-shaped value member: { ULONG cbSize; void *pBlobData; } — 8 bytes on i386,
/// 16 bytes on amd64/arm64, which gives PROPVARIANT the correct 16/24-byte size.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct PROP_BLOB {
    pub cbSize: ULONG,
    pub pBlobData: *mut c_void,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union PROPVARIANT_u {
    pub iVal: i16,    // SHORT, value-union offset 0 == struct offset 8
    pub blob: PROP_BLOB,
}
/// PROPVARIANT: leading { VARTYPE vt; WORD wReserved1..3; } header (8 bytes) then value union.
/// Size = 16 on i386, 24 on amd64/arm64 (matches the SDK).
#[repr(C)]
pub struct PROPVARIANT {
    pub vt: VARTYPE,
    pub wReserved1: WORD,
    pub wReserved2: WORD,
    pub wReserved3: WORD,
    pub u: PROPVARIANT_u,
}

/// FILETIME (minwinbase.h): { DWORD dwLowDateTime; DWORD dwHighDateTime; }
#[repr(C)]
#[derive(Copy, Clone)]
pub struct FILETIME {
    pub dwLowDateTime: DWORD,
    pub dwHighDateTime: DWORD,
}
/// STATPROPSETSTG (propidl.idl) — only used as an out buffer for IPropertyStorage::Stat.
#[repr(C)]
pub struct STATPROPSETSTG {
    pub fmtid: FMTID,
    pub clsid: CLSID,
    pub grfFlags: DWORD,
    pub mtime: FILETIME,
    pub ctime: FILETIME,
    pub atime: FILETIME,
    pub dwOSVersion: DWORD,
}

// ── COM vtables (STDMETHODCALLTYPE == __stdcall on i386 → extern "system") ───────────────
// Only the slots actually invoked carry real signatures; the rest are pointer-sized
// placeholders so the slot offsets stay correct.

#[repr(C)]
pub struct IClassFactory {
    pub lpVtbl: *mut IClassFactoryVtbl,
}
#[repr(C)]
pub struct IClassFactoryVtbl {
    pub QueryInterface: usize,
    pub AddRef: usize,
    pub Release: unsafe extern "system" fn(*mut IClassFactory) -> ULONG,
    pub CreateInstance:
        unsafe extern "system" fn(*mut IClassFactory, LPUNKNOWN, REFIID, *mut LPVOID) -> HRESULT,
    pub LockServer: usize,
}

#[repr(C)]
pub struct IPropertySetStorage {
    pub lpVtbl: *mut IPropertySetStorageVtbl,
}
#[repr(C)]
pub struct IPropertySetStorageVtbl {
    pub QueryInterface: usize,
    pub AddRef: usize,
    pub Release: usize,
    pub Create: unsafe extern "system" fn(
        *mut IPropertySetStorage,
        REFFMTID,
        *const CLSID,
        DWORD,
        DWORD,
        *mut *mut IPropertyStorage,
    ) -> HRESULT,
    pub Open: unsafe extern "system" fn(
        *mut IPropertySetStorage,
        REFFMTID,
        DWORD,
        *mut *mut IPropertyStorage,
    ) -> HRESULT,
    pub Delete: unsafe extern "system" fn(*mut IPropertySetStorage, REFFMTID) -> HRESULT,
    pub Enum: usize,
}

#[repr(C)]
pub struct IPropertyStorage {
    pub lpVtbl: *mut IPropertyStorageVtbl,
}
#[repr(C)]
pub struct IPropertyStorageVtbl {
    pub QueryInterface: usize,
    pub AddRef: usize,
    pub Release: unsafe extern "system" fn(*mut IPropertyStorage) -> ULONG,
    pub ReadMultiple: unsafe extern "system" fn(
        *mut IPropertyStorage,
        ULONG,
        *const PROPSPEC,
        *mut PROPVARIANT,
    ) -> HRESULT,
    pub WriteMultiple: unsafe extern "system" fn(
        *mut IPropertyStorage,
        ULONG,
        *const PROPSPEC,
        *const PROPVARIANT,
        PROPID,
    ) -> HRESULT,
    pub DeleteMultiple: usize,
    pub ReadPropertyNames: usize,
    pub WritePropertyNames: usize,
    pub DeletePropertyNames: usize,
    pub Commit: usize,
    pub Revert: usize,
    pub Enum: usize,
    pub SetTimes: usize,
    pub SetClass: usize,
    pub Stat: unsafe extern "system" fn(*mut IPropertyStorage, *mut STATPROPSETSTG) -> HRESULT,
}

// ── wide-string literals (built as u16 arrays — no _T/L"" in Rust) ──────────────────────
// L"CLSID\\%s\\InprocServer32"
static FMT_INPROC: [WCHAR; 24] = [
    0x43, 0x4C, 0x53, 0x49, 0x44, 0x5C, 0x25, 0x73, 0x5C, 0x49, 0x6E, 0x70, 0x72, 0x6F, 0x63, 0x53,
    0x65, 0x72, 0x76, 0x65, 0x72, 0x33, 0x32, 0x00,
];
// L"LoadWithoutCOM"
static STR_LOADWITHOUTCOM: [WCHAR; 15] = [
    0x4C, 0x6F, 0x61, 0x64, 0x57, 0x69, 0x74, 0x68, 0x6F, 0x75, 0x74, 0x43, 0x4F, 0x4D, 0x00,
];

// ── imports (resolved from shell32's import libs at link) ───────────────────────────────
extern "system" {
    // ole32
    fn CLSIDFromString(lpsz: LPCWSTR, pclsid: *mut CLSID) -> DWORD; // @8
    fn CoCreateInstance(
        rclsid: REFCLSID,
        pUnkOuter: LPUNKNOWN,
        dwClsContext: DWORD,
        riid: REFIID,
        ppv: *mut LPVOID,
    ) -> HRESULT; // @20
    fn CoGetMalloc(dwMemContext: DWORD, ppMalloc: *mut LPMALLOC) -> HRESULT; // @8
    fn CoTaskMemAlloc(cb: SIZE_T) -> LPVOID; // @4
    fn CoTaskMemFree(pv: LPVOID); // @4
    fn CreateClassMoniker(rclsid: REFCLSID, ppmk: *mut *mut c_void) -> HRESULT; // @8
    // shell32 itself (defined on the C++ side under __REACTOS__)
    fn DllGetClassObject(rclsid: REFCLSID, riid: REFIID, ppv: *mut LPVOID) -> HRESULT; // @12
    // kernel32
    fn FreeLibrary(hLibModule: HMODULE) -> BOOL; // @4
    fn GetProcAddress(hModule: HMODULE, lpProcName: LPCSTR) -> *mut c_void; // @8
    fn GetProcessHeap() -> HANDLE; // @0
    fn GetVersion() -> DWORD; // @0
    fn GlobalFree(hMem: HGLOBAL) -> HGLOBAL; // @4
    fn GlobalLock(hMem: HGLOBAL) -> LPVOID; // @4
    fn GlobalUnlock(hMem: HGLOBAL) -> BOOL; // @4
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID; // @12
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL; // @12
    fn LoadLibraryExW(lpLibFileName: LPCWSTR, hFile: HANDLE, dwFlags: DWORD) -> HMODULE; // @12
    fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: LPCSTR,
        cbMultiByte: INT,
        lpWideCharStr: LPWSTR,
        cchWideChar: INT,
    ) -> INT; // @24
    fn WideCharToMultiByte(
        CodePage: UINT,
        dwFlags: DWORD,
        lpWideCharStr: LPCWSTR,
        cchWideChar: INT,
        lpMultiByteStr: LPSTR,
        cbMultiByte: INT,
        lpDefaultChar: LPCSTR,
        lpUsedDefaultChar: *mut BOOL,
    ) -> INT; // @32
    fn lstrcpynA(lpString1: LPSTR, lpString2: LPCSTR, iMaxLength: INT) -> LPSTR; // @12
    fn lstrcpynW(lpString1: LPWSTR, lpString2: LPCWSTR, iMaxLength: INT) -> LPWSTR; // @12
    // advapi32
    fn RegCloseKey(hKey: HKEY) -> LONG; // @4
    fn RegOpenKeyExW(
        hKey: HKEY,
        lpSubKey: LPCWSTR,
        ulOptions: DWORD,
        samDesired: REGSAM,
        phkResult: PHKEY,
    ) -> LONG; // @20
    // shlwapi
    fn SHQueryValueExW(
        hKey: HKEY,
        pszValue: LPCWSTR,
        pdwReserved: LPDWORD,
        pdwType: LPDWORD,
        pvData: LPVOID,
        pcbData: LPDWORD,
    ) -> LONG; // @24 (registry status; matches shellreg.rs + the real shlwapi signature)
    fn SHStringFromGUIDW(guid: REFGUID, lpszDest: LPWSTR, cchMax: INT) -> INT; // @12
}

pub type REFGUID = *const GUID;

// user32 GetWindowLongPtr/SetWindowLongPtr — on i386 the SDK maps these macros to the
// *Long* (32-bit) entry points (confirmed: _GetWindowLongA@8 / _SetWindowLongA@12).
#[cfg(target_arch = "x86")]
extern "system" {
    fn IsWindow(hWnd: HWND) -> BOOL; // @4
    fn GetWindowLongA(hWnd: HWND, nIndex: INT) -> LONG; // @8
    fn SetWindowLongA(hWnd: HWND, nIndex: INT, dwNewLong: LONG) -> LONG; // @12
}
#[cfg(not(target_arch = "x86"))]
extern "system" {
    fn IsWindow(hWnd: HWND) -> BOOL;
    fn GetWindowLongPtrA(hWnd: HWND, nIndex: INT) -> LONG_PTR;
    fn SetWindowLongPtrA(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR;
}

// msvcrt — swprintf is the legacy variadic form ReactOS shell32 uses.
extern "C" {
    fn swprintf(buffer: *mut WCHAR, format: *const WCHAR, ...) -> INT;
}

// data imports (GUIDs) — undecorated names (`_IID_IClassFactory` / `_CLSID_QueryCancelAutoPlay`)
extern "C" {
    static IID_IClassFactory: IID;
    static CLSID_QueryCancelAutoPlay: CLSID;
}

// ── small in-file libc helpers (avoid pulling in the __imp_ thunks) ──────────────────────
#[inline]
fn SUCCEEDED(hr: HRESULT) -> bool {
    hr >= 0
}
#[inline]
fn FAILED(hr: HRESULT) -> bool {
    hr < 0
}

#[inline]
unsafe fn r_strlen(mut s: *const CHAR) -> usize {
    let mut n = 0usize;
    while *s != 0 {
        s = s.add(1);
        n += 1;
    }
    n
}
#[inline]
unsafe fn r_wcslen(mut s: *const WCHAR) -> usize {
    let mut n = 0usize;
    while *s != 0 {
        s = s.add(1);
        n += 1;
    }
    n
}
/// memset(p, 0, n)
#[inline]
unsafe fn r_zero(p: *mut u8, n: usize) {
    let mut i = 0usize;
    while i < n {
        *p.add(i) = 0;
        i += 1;
    }
}

/// SHELL_OsIsUnicode() — shell32_main.h: `!(GetVersion() & 0x80000000)`.
#[inline]
unsafe fn shell_os_is_unicode() -> bool {
    (GetVersion() & 0x80000000) == 0
}

/// HKEY_CLASSES_ROOT = ((HKEY)(LONG_PTR)(LONG)0x80000000) — sign-extended on 64-bit.
#[inline]
fn hkey_classes_root() -> HKEY {
    0x80000000u32 as i32 as isize as usize as *mut c_void
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  SH32_CoCreateInstance
// ════════════════════════════════════════════════════════════════════════════════════════
#[no_mangle]
pub unsafe extern "system" fn SH32_CoCreateInstance(
    aclsid: LPCWSTR,
    clsid: *const CLSID,
    pUnkOuter: LPUNKNOWN,
    dwClsCtx: DWORD,
    refiid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    let mut iid: CLSID = zeroed();
    let mut myclsid: *const CLSID = clsid;
    let mut sKeyName: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut sClassID: [WCHAR; 60] = [0; 60];
    let mut sDllPath: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut hKey: HKEY = null_mut();
    let mut dwSize: DWORD;
    let mut pcf: *mut IClassFactory = null_mut();

    if ppv.is_null() {
        return E_POINTER;
    }
    write_volatile(ppv, null_mut());

    // if the clsid is a string, convert it
    if clsid.is_null() {
        if aclsid.is_null() {
            return REGDB_E_CLASSNOTREG;
        }
        SHCLSIDFromStringW(aclsid, &mut iid);
        myclsid = &iid;
    }

    if SUCCEEDED(DllGetClassObject(
        myclsid,
        &IID_IClassFactory,
        &mut pcf as *mut *mut IClassFactory as *mut LPVOID,
    )) {
        let hres = ((*(*pcf).lpVtbl).CreateInstance)(pcf, pUnkOuter, refiid, ppv);
        ((*(*pcf).lpVtbl).Release)(pcf);
        // goto end; hKey is still 0 here.
        return hres;
    }

    // we look up the dll path in the registry
    SHStringFromGUIDW(myclsid, sClassID.as_mut_ptr(), sClassID.len() as INT);
    swprintf(sKeyName.as_mut_ptr(), FMT_INPROC.as_ptr(), sClassID.as_ptr());

    if RegOpenKeyExW(
        hkey_classes_root(),
        sKeyName.as_ptr(),
        0,
        KEY_READ,
        &mut hKey,
    ) != 0
    {
        return E_ACCESSDENIED;
    }

    let hres: HRESULT = loop {
        // if a special registry key is set, load a shell extension without ole32
        if (dwClsCtx & CLSCTX_INPROC_SERVER) != 0
            && SHQueryValueExW(
                hKey,
                STR_LOADWITHOUTCOM.as_ptr(),
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
            ) == 0
        {
            type DllGetClassObjectFunc =
                unsafe extern "system" fn(REFCLSID, REFIID, *mut LPVOID) -> HRESULT;

            dwSize = (size_of::<WCHAR>() * MAX_PATH) as DWORD; // sizeof(sDllPath)
            SHQueryValueExW(
                hKey,
                null(),
                null_mut(),
                null_mut(),
                sDllPath.as_mut_ptr() as LPVOID,
                &mut dwSize,
            );

            let hLibrary = LoadLibraryExW(sDllPath.as_ptr(), null_mut(), LOAD_WITH_ALTERED_SEARCH_PATH);
            if hLibrary.is_null() {
                break E_ACCESSDENIED;
            }
            let proc = GetProcAddress(hLibrary, b"DllGetClassObject\0".as_ptr() as LPCSTR);
            if proc.is_null() {
                FreeLibrary(hLibrary);
                break E_ACCESSDENIED;
            }
            let dll_get_class_object: DllGetClassObjectFunc = transmute(proc);
            let h = dll_get_class_object(
                myclsid,
                &IID_IClassFactory,
                &mut pcf as *mut *mut IClassFactory as *mut LPVOID,
            );
            if FAILED(h) {
                break h;
            }

            let h2 = ((*(*pcf).lpVtbl).CreateInstance)(pcf, pUnkOuter, refiid, ppv);
            ((*(*pcf).lpVtbl).Release)(pcf);
            break h2;
        } else {
            // load an external dll in the usual way
            break CoCreateInstance(myclsid, pUnkOuter, dwClsCtx, refiid, ppv);
        }
    };

    // end:
    if !hKey.is_null() {
        RegCloseKey(hKey);
    }
    hres
}

#[no_mangle]
pub unsafe extern "system" fn SHCoCreateInstance(
    aclsid: LPCWSTR,
    clsid: *const CLSID,
    pUnkOuter: LPUNKNOWN,
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    SH32_CoCreateInstance(aclsid, clsid, pUnkOuter, CLSCTX_INPROC_SERVER, riid, ppv)
}

#[no_mangle]
pub unsafe extern "system" fn SH32_ExtCoCreateInstance(
    aclsid: LPCWSTR,
    clsid: *const CLSID,
    pUnkOuter: LPUNKNOWN,
    dwClsCtx: DWORD,
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    // TODO: Verify CLSID is allowed (Shell Extensions\Approved) if REST_ENFORCESHELLEXTSECURITY
    SH32_CoCreateInstance(aclsid, clsid, pUnkOuter, dwClsCtx, riid, ppv)
}

#[no_mangle]
pub unsafe extern "system" fn SHExtCoCreateInstance(
    aclsid: LPCWSTR,
    clsid: *const CLSID,
    pUnkOuter: LPUNKNOWN,
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    SH32_ExtCoCreateInstance(aclsid, clsid, pUnkOuter, CLSCTX_INPROC_SERVER, riid, ppv)
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  SHCLSIDFromString{A,W,AW}
// ════════════════════════════════════════════════════════════════════════════════════════
#[no_mangle]
pub unsafe extern "system" fn SHCLSIDFromStringA(clsid: LPCSTR, id: *mut CLSID) -> DWORD {
    let mut buffer: [WCHAR; 40] = [0; 40];
    if MultiByteToWideChar(CP_ACP, 0, clsid, -1, buffer.as_mut_ptr(), 40) == 0 {
        return CO_E_CLASSSTRING;
    }
    CLSIDFromString(buffer.as_ptr(), id)
}

#[no_mangle]
pub unsafe extern "system" fn SHCLSIDFromStringW(clsid: LPCWSTR, id: *mut CLSID) -> DWORD {
    CLSIDFromString(clsid, id)
}

#[no_mangle]
pub unsafe extern "system" fn SHCLSIDFromStringAW(clsid: LPCVOID, id: *mut CLSID) -> DWORD {
    if shell_os_is_unicode() {
        SHCLSIDFromStringW(clsid as LPCWSTR, id)
    } else {
        SHCLSIDFromStringA(clsid as LPCSTR, id)
    }
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  SHGetMalloc / SHAlloc / SHFree
// ════════════════════════════════════════════════════════════════════════════════════════
#[no_mangle]
pub unsafe extern "system" fn SHGetMalloc(lpmal: *mut LPMALLOC) -> HRESULT {
    CoGetMalloc(MEMCTX_TASK, lpmal)
}

#[no_mangle]
pub unsafe extern "system" fn SHAlloc(len: SIZE_T) -> LPVOID {
    CoTaskMemAlloc(len)
}

#[no_mangle]
pub unsafe extern "system" fn SHFree(pv: LPVOID) {
    CoTaskMemFree(pv);
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  Drag & drop
// ════════════════════════════════════════════════════════════════════════════════════════
#[no_mangle]
pub unsafe extern "system" fn DragAcceptFiles(hWnd: HWND, b: BOOL) {
    if IsWindow(hWnd) == FALSE {
        return;
    }

    #[cfg(target_arch = "x86")]
    let mut exstyle: LONG = GetWindowLongA(hWnd, GWL_EXSTYLE);
    #[cfg(not(target_arch = "x86"))]
    let mut exstyle: LONG = GetWindowLongPtrA(hWnd, GWL_EXSTYLE) as LONG;

    if b != FALSE {
        exstyle |= WS_EX_ACCEPTFILES;
    } else {
        exstyle &= !WS_EX_ACCEPTFILES;
    }

    #[cfg(target_arch = "x86")]
    SetWindowLongA(hWnd, GWL_EXSTYLE, exstyle);
    #[cfg(not(target_arch = "x86"))]
    SetWindowLongPtrA(hWnd, GWL_EXSTYLE, exstyle as LONG_PTR);
}

#[no_mangle]
pub unsafe extern "system" fn DragFinish(h: HDROP) {
    GlobalFree(h);
}

#[no_mangle]
pub unsafe extern "system" fn DragQueryPoint(hDrop: HDROP, p: *mut POINT) -> BOOL {
    let lpDropFileStruct = GlobalLock(hDrop) as *mut DROPFILES;

    write_volatile(p, (*lpDropFileStruct).pt);
    let bRet = (*lpDropFileStruct).fNC;

    GlobalUnlock(hDrop);
    bRet
}

#[no_mangle]
pub unsafe extern "system" fn DragQueryFileA(
    hDrop: HDROP,
    lFile: UINT,
    lpszFile: LPSTR,
    lLength: UINT,
) -> UINT {
    let mut i: UINT = 0;
    let lpDropFileStruct = GlobalLock(hDrop) as *mut DROPFILES;

    if lpDropFileStruct.is_null() {
        GlobalUnlock(hDrop);
        return i;
    }

    let mut lpDrop = (lpDropFileStruct as *mut CHAR).offset((*lpDropFileStruct).pFiles as isize);

    if (*lpDropFileStruct).fWide != FALSE {
        let mut lpszFileW: LPWSTR = null_mut();

        if !lpszFile.is_null() && lFile != 0xFFFFFFFF {
            lpszFileW =
                HeapAlloc(GetProcessHeap(), 0, lLength as usize * size_of::<WCHAR>()) as LPWSTR;
            if lpszFileW.is_null() {
                GlobalUnlock(hDrop);
                return i;
            }
        }
        i = DragQueryFileW(hDrop, lFile, lpszFileW, lLength);

        if !lpszFileW.is_null() {
            WideCharToMultiByte(
                CP_ACP,
                0,
                lpszFileW,
                -1,
                lpszFile,
                lLength as INT,
                null(),
                null_mut(),
            );
            HeapFree(GetProcessHeap(), 0, lpszFileW as LPVOID);
        }
        GlobalUnlock(hDrop);
        return i;
    }

    // while (i++ < lFile)
    loop {
        let cond = i < lFile;
        i = i.wrapping_add(1);
        if !cond {
            break;
        }
        // while (*lpDrop++);  — skip filename
        loop {
            let c = *lpDrop;
            lpDrop = lpDrop.add(1);
            if c == 0 {
                break;
            }
        }
        if *lpDrop == 0 {
            i = if lFile == 0xFFFFFFFF { i } else { 0 };
            GlobalUnlock(hDrop);
            return i;
        }
    }

    i = r_strlen(lpDrop) as UINT;
    if lpszFile.is_null() {
        GlobalUnlock(hDrop);
        return i; // needed buffer size
    }
    lstrcpynA(lpszFile, lpDrop, lLength as INT);
    GlobalUnlock(hDrop);
    i
}

#[no_mangle]
pub unsafe extern "system" fn DragQueryFileW(
    hDrop: HDROP,
    lFile: UINT,
    lpszwFile: LPWSTR,
    lLength: UINT,
) -> UINT {
    let mut i: UINT = 0;
    let lpDropFileStruct = GlobalLock(hDrop) as *mut DROPFILES;

    if lpDropFileStruct.is_null() {
        GlobalUnlock(hDrop);
        return i;
    }

    let mut lpwDrop =
        (lpDropFileStruct as *mut CHAR).offset((*lpDropFileStruct).pFiles as isize) as *mut WCHAR;

    if (*lpDropFileStruct).fWide == FALSE {
        let mut lpszFileA: LPSTR = null_mut();

        if !lpszwFile.is_null() && lFile != 0xFFFFFFFF {
            lpszFileA = HeapAlloc(GetProcessHeap(), 0, lLength as usize) as LPSTR;
            if lpszFileA.is_null() {
                GlobalUnlock(hDrop);
                return i;
            }
        }
        i = DragQueryFileA(hDrop, lFile, lpszFileA, lLength);

        if !lpszFileA.is_null() {
            MultiByteToWideChar(CP_ACP, 0, lpszFileA, -1, lpszwFile, lLength as INT);
            HeapFree(GetProcessHeap(), 0, lpszFileA as LPVOID);
        }
        GlobalUnlock(hDrop);
        return i;
    }

    i = 0;
    // while (i++ < lFile)
    loop {
        let cond = i < lFile;
        i = i.wrapping_add(1);
        if !cond {
            break;
        }
        // while (*lpwDrop++);  — skip filename
        loop {
            let c = *lpwDrop;
            lpwDrop = lpwDrop.add(1);
            if c == 0 {
                break;
            }
        }
        if *lpwDrop == 0 {
            i = if lFile == 0xFFFFFFFF { i } else { 0 };
            GlobalUnlock(hDrop);
            return i;
        }
    }

    i = r_wcslen(lpwDrop) as UINT;
    if lpszwFile.is_null() {
        GlobalUnlock(hDrop);
        return i; // needed buffer size
    }
    lstrcpynW(lpszwFile, lpwDrop, lLength as INT);
    GlobalUnlock(hDrop);
    i
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  SHPropStg{Create,ReadMultiple,WriteMultiple}
// ════════════════════════════════════════════════════════════════════════════════════════

/// The shared "OPEN_EXISTING tail" of SHPropStgCreate (the last switch case + return S_OK).
unsafe fn shpropstg_create_tail(
    hres: HRESULT,
    ppstg: *mut *mut IPropertyStorage,
    puCodePage: *mut UINT,
) -> HRESULT {
    if FAILED(hres) {
        return hres;
    }

    if !puCodePage.is_null() {
        let mut prop: PROPSPEC = zeroed();
        let mut ret: PROPVARIANT = zeroed();

        prop.ulKind = PRSPEC_PROPID;
        prop.u.propid = PID_CODEPAGE;
        let h = ((*(**ppstg).lpVtbl).ReadMultiple)(*ppstg, 1, &prop, &mut ret);
        if FAILED(h) || ret.vt != VT_I2 {
            write_volatile(puCodePage, 0);
        } else {
            write_volatile(puCodePage, ret.u.iVal as UINT);
        }
    }

    S_OK_
}

#[no_mangle]
pub unsafe extern "system" fn SHPropStgCreate(
    psstg: *mut IPropertySetStorage,
    fmtid: REFFMTID,
    pclsid: *const CLSID,
    grfFlags: DWORD,
    grfMode: DWORD,
    dwDisposition: DWORD,
    ppstg: *mut *mut IPropertyStorage,
    puCodePage: *mut UINT,
) -> HRESULT {
    let mut hres = ((*(*psstg).lpVtbl).Open)(psstg, fmtid, grfMode, ppstg);

    // The C `switch` falls through case-to-case; mirror that cascade explicitly.
    match dwDisposition {
        CREATE_ALWAYS => {
            if SUCCEEDED(hres) {
                ((*(**ppstg).lpVtbl).Release)(*ppstg);
                hres = ((*(*psstg).lpVtbl).Delete)(psstg, fmtid);
                if FAILED(hres) {
                    return hres;
                }
                hres = E_FAIL;
            }
            // fall through → OPEN_ALWAYS/CREATE_NEW
            if FAILED(hres) {
                hres =
                    ((*(*psstg).lpVtbl).Create)(psstg, fmtid, pclsid, grfFlags, grfMode, ppstg);
            }
            // fall through → OPEN_EXISTING
            shpropstg_create_tail(hres, ppstg, puCodePage)
        }
        OPEN_ALWAYS | CREATE_NEW => {
            if FAILED(hres) {
                hres =
                    ((*(*psstg).lpVtbl).Create)(psstg, fmtid, pclsid, grfFlags, grfMode, ppstg);
            }
            shpropstg_create_tail(hres, ppstg, puCodePage)
        }
        OPEN_EXISTING => shpropstg_create_tail(hres, ppstg, puCodePage),
        _ => S_OK_,
    }
}

#[no_mangle]
pub unsafe extern "system" fn SHPropStgReadMultiple(
    pps: *mut IPropertyStorage,
    uCodePage: UINT,
    cpspec: ULONG,
    rgpspec: *const PROPSPEC,
    rgvar: *mut PROPVARIANT,
) -> HRESULT {
    let mut stat: STATPROPSETSTG = zeroed();

    r_zero(rgvar as *mut u8, cpspec as usize * size_of::<PROPVARIANT>());
    let hres = ((*(*pps).lpVtbl).ReadMultiple)(pps, cpspec, rgpspec, rgvar);
    if FAILED(hres) {
        return hres;
    }

    if uCodePage == 0 {
        let mut prop: PROPSPEC = zeroed();
        let mut ret: PROPVARIANT = zeroed();

        prop.ulKind = PRSPEC_PROPID;
        prop.u.propid = PID_CODEPAGE;
        let hres = ((*(*pps).lpVtbl).ReadMultiple)(pps, 1, &prop, &mut ret);
        if FAILED(hres) || ret.vt != VT_I2 {
            return S_OK_;
        }

        // uCodePage = ret.u.iVal;  (local, unused beyond this point)
        let _uCodePage = ret.u.iVal as UINT; // nolint: let-shadow (C never reads uCodePage after this)
    }

    let hres = ((*(*pps).lpVtbl).Stat)(pps, &mut stat);
    if FAILED(hres) {
        return S_OK_;
    }

    // TODO: do something with codepage and stat
    S_OK_
}

#[no_mangle]
pub unsafe extern "system" fn SHPropStgWriteMultiple(
    pps: *mut IPropertyStorage,
    uCodePage: *mut UINT,
    cpspec: ULONG,
    rgpspec: *const PROPSPEC,
    rgvar: *mut PROPVARIANT,
    propidNameFirst: PROPID,
) -> HRESULT {
    let mut stat: STATPROPSETSTG = zeroed();
    let codepage: UINT;

    let hres = ((*(*pps).lpVtbl).Stat)(pps, &mut stat);
    if FAILED(hres) {
        return hres;
    }

    if !uCodePage.is_null() && *uCodePage != 0 {
        codepage = *uCodePage;
    } else {
        let mut prop: PROPSPEC = zeroed();
        let mut ret: PROPVARIANT = zeroed();

        prop.ulKind = PRSPEC_PROPID;
        prop.u.propid = PID_CODEPAGE;
        let hres = ((*(*pps).lpVtbl).ReadMultiple)(pps, 1, &prop, &mut ret);
        if FAILED(hres) {
            return hres;
        }
        if ret.vt != VT_I2 || ret.u.iVal == 0 {
            return E_FAIL;
        }

        codepage = ret.u.iVal as UINT;
        if !uCodePage.is_null() {
            write_volatile(uCodePage, codepage);
        }
    }
    let _ = codepage; // TODO: do something with codepage and stat

    ((*(*pps).lpVtbl).WriteMultiple)(pps, cpspec, rgpspec, rgvar, propidNameFirst)
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  SHCreateQueryCancelAutoPlayMoniker
// ════════════════════════════════════════════════════════════════════════════════════════
#[no_mangle]
pub unsafe extern "system" fn SHCreateQueryCancelAutoPlayMoniker(
    moniker: *mut *mut IMoniker,
) -> HRESULT {
    if moniker.is_null() {
        return E_INVALIDARG;
    }
    CreateClassMoniker(&CLSID_QueryCancelAutoPlay, moniker as *mut *mut c_void)
}

// ADDED-TO-TYPES?: GUID/CLSID/IID/FMTID + REFIID/REFCLSID/REFFMTID/REFGUID, POINT, DROPFILES,
// SIZE_T, ULONG, PROPID, VARTYPE, HGLOBAL, HDROP, HMODULE, LPUNKNOWN, FILETIME — all common
// shell/COM layouts worth hoisting to types.rs. PROPSPEC/PROPVARIANT/STATPROPSETSTG and the
// IClassFactory/IPropertySetStorage/IPropertyStorage vtables are COM-specific (keep file-local
// or move to a shared com module). NOTE: POINT/DROPFILES/SIZE_T/HGLOBAL also defined in
// wine/clipboard.rs — hoist once to avoid the duplicate when both are in the crate.
