//! wine/shellord.c -> shellord.rs  (Wave 4, the ordinal grab-bag, 72 exports + 1 data global)
//!
//! Faithful translation of the active build config:
//!   -D__REACTOS__ -D_SHELL32_ -D_WINE -DUNICODE -D_UNICODE -D_WIN32_WINNT=0x600
//!   (DLL_EXPORT_VERSION is *undefined* -> the preprocessor evaluates `(DLL_EXPORT_VERSION)`
//!    as 0, so `(DLL_EXPORT_VERSION) < _WIN32_WINNT_VISTA` is TRUE -> the SHGetSetSettings
//!    "SuperHidden" alt-write branch IS compiled; confirmed via `strings -el` of the obj.)
//! TRACE/WARN/ERR/FIXME and the wine debug machinery are no-ops and are omitted.
//!
//! ⚠ INTEGRATION NOTE: `ShellMessageBoxA` is a genuine C-variadic export (`...` + va_list
//! forwarded to FormatMessageA). Stable Rust cannot DEFINE a variadic fn (`c_variadic`,
//! rust-lang#44930). It was a 3-line trampoline in `dll/win32/shell32/rust_varargs_shim.c`;
//! since 2026-09-24 `ShellMessageBoxA` below RECEIVES the varargs as fixed slots
//! (va-slots-rust) and calls the fixed-arity `_v` body. The exported ordinal-183 symbol + its
//! cdecl variadic ABI are unchanged.
//!
//! ABI: WINAPI -> extern "system" (i386 `_Name@N`); plain cdecl/EXTERN_C -> extern "C"
//! (i386 `_Name`). Imports declared with the convention matching each obj decoration
//! (nm-verified against the pre-flip shellord.c.obj). The `_IL*` PIDL helpers + the COM
//! constructors resolve within the crate / shell32 .cpp side.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_assignments, unused_variables, unused_mut, unused_parens)]

use crate::island::{SFGAO_LINK, SHGFI_ATTRIBUTES, SHGFI_ATTR_SPECIFIED, SHGFI_DISPLAYNAME, SHGFI_PIDL, SHGNLI_NOLNK, SHGNLI_NOUNIQUE, tagSHGDN_SHGDN_INFOLDER};
// The crate's own functions: the definition is the declaration.
use crate::shell32::cidldataobj_free::IDataObject_Constructor;
use crate::shell32::cidldataobj_free::IEnumFORMATETC_Constructor;
use crate::wine::pidl::ILFree;
use crate::wine::pidl::SHGetPathFromIDListW;
use crate::wine::shellpath::SHGetSpecialFolderPathW;
use crate::shell32::shlexec::ShellExecuteExW;
use crate::shell32::shlexec::ShellExecuteW;
// One declaration per external symbol: island.rs.
use crate::island::CallWindowProcW;
use crate::island::CloseHandle;
use crate::island::CoCreateInstance;
use crate::island::CreateFileW;
use crate::island::CreateMRUListW;
use crate::island::DefWindowProcW;
use crate::island::DeregisterShellHookWindow;
use crate::island::FindFirstFileW;
use crate::island::FindNextFileW;
use crate::island::FormatMessageA;
use crate::island::FreeLibrary;
use crate::island::GetProcAddress;
use crate::island::GetPropW;
use crate::island::GetTopWindow;
use crate::island::GetWindowLongW;
use crate::island::GlobalAlloc;
use crate::island::HeapAlloc;
use crate::island::IsWindowUnicode;
use crate::island::LoadLibraryW;
use crate::island::LoadStringA;
use crate::island::LoadStringW;
use crate::island::LocalAlloc;
use crate::island::MessageBoxA;
use crate::island::OpenClipboard;
use crate::island::PathRemoveExtensionW;
use crate::island::RegCloseKey;
use crate::island::RegCreateKeyExW;
use crate::island::RegEnumKeyExW;
use crate::island::RegOpenKeyExW;
use crate::island::RegOpenKeyW;
use crate::island::RegQueryValueExW;
use crate::island::RegSetValueExW;
use crate::island::RegisterShellHookWindow;
use crate::island::RemovePropW;
use crate::island::SHDeleteKeyW;
use crate::island::SHGetShellKey;
use crate::island::SHGetValueW;
use crate::island::SHQueryValueExW;
use crate::island::SHSendMessageBroadcastW;
use crate::island::SHSetValueW;
use crate::island::SetPropW;
use crate::island::SetWindowLongW;
use crate::island::WinHelpW;
use crate::shellcom::*;
use crate::types::*;
use core::ffi::{c_int, c_void};

// ── scalar aliases / message types not in crate::types/shellcom ──────────────────────────
use ntdll_sys::nt::WPARAM; // winnt.h (UINT_PTR)
type LRESULT = LONG_PTR; // winnt.h
type SHORT = i16; // winnt.h
type LPCOLESTR = LPCWSTR; // wtypes.h
use crate::island::HIMAGELIST; // commoncontrols.h
type HPSXA = HANDLE; // undocshell.h
type HPROPSHEETPAGE = HANDLE; // prsht.h
type LPGUID = *mut GUID; // guiddef.h
type LPDROPTARGET = *mut c_void; // ole2 (opaque pass-through)
type LPDATAOBJECT = *mut IDataObject; // objidl
type LPDROPSOURCE = *mut c_void; // ole2 (opaque pass-through)
type LPDWORD = *mut DWORD;
type PCIDLIST_ABSOLUTE = LPCITEMIDLIST;
type PCUIDLIST_RELATIVE_ARRAY = *const LPCITEMIDLIST;
type PFILETIME = *mut FILETIME;
type REGSAM = DWORD;
type LPUNKNOWN = *mut c_void;
type LPENUMFORMATETC = *mut c_void;
type LPFNADDPROPSHEETPAGE = Option<unsafe extern "system" fn(HPROPSHEETPAGE, LPARAM) -> BOOL>;
type WNDPROC = Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT>;

// ── constants (all pinned from SDK headers; header cited inline) ─────────────────────────
const MAX_PATH: usize = 260; // minwindef.h
const CP_ACP: UINT = 0; // winnls.h
const FALSE_U: UINT = 0;

// HRESULT / errors
const E_FAIL: HRESULT = 0x80004005u32 as HRESULT; // winerror.h
const ERROR_FILE_NOT_FOUND: LONG = 2; // winerror.h
const ERROR_MORE_DATA: LONG = 234; // winerror.h
const ERROR_NO_MORE_ITEMS: LONG = 259; // winerror.h
const ERROR_NOT_ENOUGH_MEMORY: LONG = 8; // winerror.h

// registry
const REG_DWORD: DWORD = 4; // winnt.h
const REG_BINARY: DWORD = 3; // winnt.h
const KEY_ENUMERATE_SUB_KEYS: REGSAM = 0x0008; // winnt.h
const KEY_READ: REGSAM = 0x20019; // winnt.h (STANDARD_RIGHTS_READ|QUERY_VALUE|ENUM|NOTIFY)
const KEY_WRITE: REGSAM = 0x20006; // winnt.h
const KEY_ALL_ACCESS: REGSAM = 0xF003F; // winnt.h

// SHKEY_* (shlwapi_undoc.h)
const SHKEY_Root_HKCU: DWORD = 0x1;
const SHKEY_Key_Explorer: DWORD = 0x00;

// SSF_* (shlobj.h)
const SSF_SHOWALLOBJECTS: DWORD = 0x00000001;
const SSF_SHOWEXTENSIONS: DWORD = 0x00000002;
const SSF_SHOWCOMPCOLOR: DWORD = 0x00000008;
const SSF_SORTCOLUMNS: DWORD = 0x00000010;
const SSF_SHOWSYSFILES: DWORD = 0x00000020;
const SSF_DOUBLECLICKINWEBVIEW: DWORD = 0x00000080;
const SSF_SHOWATTRIBCOL: DWORD = 0x00000100;
const SSF_DESKTOPHTML: DWORD = 0x00000200;
const SSF_WIN95CLASSIC: DWORD = 0x00000400;
const SSF_DONTPRETTYPATH: DWORD = 0x00000800;
const SSF_MAPNETDRVBUTTON: DWORD = 0x00001000;
const SSF_SHOWINFOTIP: DWORD = 0x00002000;
const SSF_HIDEICONS: DWORD = 0x00004000;
const SSF_NOCONFIRMRECYCLE: DWORD = 0x00008000;
const SSF_FILTER: DWORD = 0x00010000;
const SSF_WEBVIEW: DWORD = 0x00020000;
const SSF_SHOWSUPERHIDDEN: DWORD = 0x00040000;
const SSF_SEPPROCESS: DWORD = 0x00080000;
const SSF_NONETCRAWLING: DWORD = 0x00100000;
const SSF_STARTPANELON: DWORD = 0x00200000;
// local enum in shellord.c (line 218): values coincide with the Vista SSF_* defines.
const ssf_autocheckselect: DWORD = 0x00800000;
const ssf_iconsonly: DWORD = 0x01000000;
const ssf_showtypeoverlay: DWORD = 0x02000000;
const ssf_showstatusbar: DWORD = 0x04000000;
// SSF_STRUCTONLY (shellord.c #define)
const SSF_STRUCTONLY: DWORD = SSF_NOCONFIRMRECYCLE
    | SSF_DOUBLECLICKINWEBVIEW
    | SSF_DESKTOPHTML
    | SSF_WIN95CLASSIC
    | SSF_SORTCOLUMNS
    | SSF_STARTPANELON;
// SSF_ALL / SSF_IMPLEMENTED (shellord.c, upstream 4fecc908e7d). SERVERADMINUI is
// written by Explorer and read by IsOS(), never through this struct.
const SSF_SERVERADMINUI: DWORD = 0x00000004;
const SSF_ALL: DWORD = 0x07FF_FFFF & !0x40;
const SSF_IMPLEMENTED: DWORD = SSF_ALL & !SSF_SERVERADMINUI;

// SHELLSTATE w0 bit indices (gcc bitfield packing of the first 17 :1 fields)
const SS_SHOWALLOBJECTS: u32 = 0;
const SS_SHOWEXTENSIONS: u32 = 1;
const SS_NOCONFIRMRECYCLE: u32 = 2;
const SS_SHOWSYSFILES: u32 = 3;
const SS_SHOWCOMPCOLOR: u32 = 4;
const SS_DOUBLECLICKINWEBVIEW: u32 = 5;
const SS_DESKTOPHTML: u32 = 6;
const SS_WIN95CLASSIC: u32 = 7;
const SS_DONTPRETTYPATH: u32 = 8;
const SS_SHOWATTRIBCOL: u32 = 9;
const SS_MAPNETDRVBTN: u32 = 10;
const SS_SHOWINFOTIP: u32 = 11;
const SS_HIDEICONS: u32 = 12;
const SS_WEBVIEW: u32 = 13;
const SS_FILTER: u32 = 14;
const SS_SHOWSUPERHIDDEN: u32 = 15;
const SS_NONETCRAWLING: u32 = 16;
// SHELLSTATE w1 bit indices (second 32-bit storage unit at offset 28)
const SS_SEPPROCESS: u32 = 0;
const SS_STARTPANELON: u32 = 1;
const SS_SHOWSTARTPAGE: u32 = 2;
const SS_AUTOCHECKSELECT: u32 = 3;
const SS_ICONSONLY: u32 = 4;
const SS_SHOWTYPEOVERLAY: u32 = 5;
const SS_SHOWSTATUSBAR: u32 = 6;

// CABINETSTATE flags-word bit indices
const CB_FULLPATHTITLE: u32 = 0;
const CB_SAVELOCALVIEW: u32 = 1;
const CB_NOTSHELL: u32 = 2;
const CB_SIMPLEDEFAULT: u32 = 3;
const CB_DONTSHOWDESCBAR: u32 = 4;
const CB_NEWWINDOWMODE: u32 = 5;
const CB_SHOWCOMPCOLOR: u32 = 6;
const CB_DONTPRETTYNAMES: u32 = 7;
const CB_ADMINSCREATECOMMONGROUPS: u32 = 8;

// CSIDL_* (shlobj.h)
const CSIDL_RECENT: c_int = 0x0008;
const CSIDL_INTERNET_CACHE: u32 = 0x0020;
const CSIDL_CDBURN_AREA: u32 = 0x003b;

// SHARD_* (shlobj.h)
const SHARD_PIDL: UINT = 0x00000001;
const SHARD_PATHA: UINT = 0x00000002;
const SHARD_PATHW: UINT = 0x00000003;

// SHGNLI_* / SHOP_* / SEE_MASK_* / CLSCTX / SW_*
const SHGNLI_PIDL: UINT = 0x01; // shlobj.h
const SHOP_FILEPATH: DWORD = 0x02; // shlobj.h
const SEE_MASK_INVOKEIDLIST: ULONG = 0x0000000C; // shellapi.h
const CLSCTX_INPROC_SERVER: DWORD = 0x00000001; // wtypesbase.h
const SW_SHOWNORMAL: c_int = 1; // winuser.h

// SHIL_* (commoncontrols.h)
const SHIL_LARGE: c_int = 0x0;
const SHIL_SMALL: c_int = 0x1;
const SHIL_SYSSMALL: c_int = 0x3;

// SHCONTF_* (shobjidl)
const SHCONTF_FOLDERS: UINT = 32;
const SHCONTF_NONFOLDERS: UINT = 64;

// REST_* (shlobj.h SHELLSTATERESTRICTIONS enum)
const REST_NOFIND: DWORD = 0x00000080;
const REST_NORECENTDOCSHISTORY: DWORD = 0x40000002; // (REST_NOINTERNETICON=0x40000001)+1

// MRU (comctl32_undoc.h)
const MRU_BINARY: DWORD = 0x1;
const MRU_CACHEWRITE: DWORD = 0x2;

// file / heap / window / clipboard / messagebox / format
const INVALID_FILE_ATTRIBUTES: DWORD = 0xFFFFFFFF; // fileapi.h
const FILE_ATTRIBUTE_TEMPORARY: DWORD = 0x00000100; // winnt.h
const GENERIC_READ: DWORD = 0x80000000; // winnt.h
const GENERIC_WRITE: DWORD = 0x40000000; // winnt.h
const FILE_SHARE_READ: DWORD = 0x00000001; // winnt.h
const OPEN_EXISTING: DWORD = 3; // fileapi.h
const SEM_FAILCRITICALERRORS: UINT = 0x0001; // winbase.h
const HEAP_ZERO_MEMORY: DWORD = 0x00000008; // minwinbase.h
const LMEM_FIXED: UINT = 0x0000; // minwinbase.h
const GMEM_MOVEABLE: UINT = 2; // minwinbase.h
const GMEM_SHARE: UINT = 8192; // minwinbase.h
const CF_UNICODETEXT: UINT = 13; // winuser.h
const MB_SETFOREGROUND: UINT = 0x10000; // winuser.h
const OFN_EXPLORER: DWORD = 0x80000; // commdlg.h
const OFN_HIDEREADONLY: DWORD = 4; // commdlg.h
const OFN_FILEMUSTEXIST: DWORD = 0x1000; // commdlg.h
const FORMAT_MESSAGE_ALLOCATE_BUFFER: DWORD = 0x00000100; // winbase.h
const FORMAT_MESSAGE_FROM_STRING: DWORD = 0x00000400; // winbase.h
const MAXLONG: UINT = 0x7fffffff; // winnt.h
const WM_SETTINGCHANGE: UINT = 26; // winuser.h
const WM_KEYDOWN: UINT = 256; // winuser.h
const WM_CHAR: UINT = 258; // winuser.h
const WM_PASTE: UINT = 770; // winuser.h
const WM_NCDESTROY: UINT = 130; // winuser.h
const WM_UNICHAR: UINT = 0x0109; // winuser.h
const WM_IME_CHAR: UINT = 0x0286; // winuser.h
const UNICODE_NOCHAR: WPARAM = 0xffff; // winuser.h
const VK_SHIFT: c_int = 16; // winuser.h
const VK_CONTROL: c_int = 17; // winuser.h
const VK_INSERT: c_int = 45; // winuser.h
const GWL_STYLE: c_int = -16; // winuser.h
const GWLP_WNDPROC: c_int = -4; // winuser.h
const ES_READONLY: LONG_PTR = 0x0800; // winuser.h

// security (setypes.h)
const SECURITY_SERVICE_RID: DWORD = 0x00000006;
const SECURITY_LOCAL_SYSTEM_RID: DWORD = 0x00000012;
const SECURITY_BUILTIN_DOMAIN_RID: DWORD = 0x00000020;
const DOMAIN_ALIAS_RID_ADMINS: ULONG = 0x00000220;

// ── file-local structs (faithful SDK layout) ────────────────────────────────────────────
#[repr(C)]
#[derive(Copy, Clone)]
struct POINT {
    x: LONG,
    y: LONG,
}
type LPPOINT = *mut POINT;
#[repr(C)]
#[derive(Copy, Clone)]
struct RECT {
    left: LONG,
    top: LONG,
    right: LONG,
    bottom: LONG,
}
#[repr(C)]
struct AUTO_SCROLL_DATA {
    _opaque: [u8; 0],
} // shlobj.h (only passed by ptr; stub uses it)

/// SHELLSTATE (shlobj.h) — modeled as its exact 32-byte gcc bitfield layout: two 32-bit
/// flag words (offset 0 and 28) bracketing the scalar members. Bit indices = SS_* above.
/// `packed` because shlobj.h declares it under `pshpack1`: 32 bytes with alignment 1, so a
/// caller's SHELLSTATE need not be 4-aligned (the island renders it the same way).
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct SHELLSTATE {
    w0: u32,             // 17 :1 flags + :15 pad
    dwWin95Unused: DWORD,
    uWin95Unused: UINT,
    lParamSort: LONG,
    iSortDirection: c_int,
    version: UINT,
    uNotUsed: UINT,
    w1: u32,             // 7 :1 flags + fSpareFlags:9 + :15 pad
}
pub type LPSHELLSTATE = *mut SHELLSTATE;
impl SHELLSTATE {
    #[inline]
    fn g0(&self, b: u32) -> BOOL {
        ((self.w0 >> b) & 1) as BOOL
    }
    #[inline]
    fn s0(&mut self, b: u32, v: BOOL) {
        let m = 1u32 << b;
        if v != 0 {
            self.w0 |= m;
        } else {
            self.w0 &= !m;
        }
    }
    #[inline]
    fn g1(&self, b: u32) -> BOOL {
        ((self.w1 >> b) & 1) as BOOL
    }
    #[inline]
    fn s1(&mut self, b: u32, v: BOOL) {
        let m = 1u32 << b;
        if v != 0 {
            self.w1 |= m;
        } else {
            self.w1 &= !m;
        }
    }
}

/// SHELLFLAGSTATE (shlobj.h) — a single 32-bit bitfield word.
#[repr(C)]
#[derive(Copy, Clone)]
struct SHELLFLAGSTATE {
    flags: u32,
}
type LPSHELLFLAGSTATE = *mut SHELLFLAGSTATE;
impl SHELLFLAGSTATE {
    #[inline]
    fn set(&mut self, b: u32, v: BOOL) {
        let m = 1u32 << b;
        if v != 0 {
            self.flags |= m;
        } else {
            self.flags &= !m;
        }
    }
}
// SHELLFLAGSTATE bit indices (differ from SHELLSTATE past bit 12!)
const SFS_HIDEICONS: u32 = 12;
const SFS_AUTOCHECKSELECT: u32 = 13;
const SFS_ICONSONLY: u32 = 14;

/// REGSHELLSTATE (undocshell.h): { DWORD dwSize; SHELLSTATE ss; } (0x24 bytes)
#[repr(C)]
#[derive(Copy, Clone)]
struct REGSHELLSTATE {
    dwSize: DWORD,
    ss: SHELLSTATE,
}
type PREGSHELLSTATE = *mut REGSHELLSTATE;

/// CABINETSTATE (shlobj.h) — 12 bytes: cLength/nVersion, flags word, fMenuEnumFilter.
/// `pub` so win32.rs can re-export it: the SDK spells the flags as bitfields, and this
/// explicit flags word is the proven layout — do not generate a second one.
#[repr(C)]
pub struct CABINETSTATE {
    pub cLength: WORD,
    pub nVersion: WORD,
    pub flags: u32,
    pub fMenuEnumFilter: UINT,
}
impl CABINETSTATE {
    #[inline]
    fn set(&mut self, b: u32, v: BOOL) {
        let m = 1u32 << b;
        if v != 0 {
            self.flags |= m;
        } else {
            self.flags &= !m;
        }
    }
}

/// OPENFILENAMEW (commdlg.h, _WIN32_WINNT>=0x500 layout).
#[repr(C)]
struct OPENFILENAMEW {
    lStructSize: DWORD,
    hwndOwner: HWND,
    hInstance: HINSTANCE,
    lpstrFilter: LPCWSTR,
    lpstrCustomFilter: LPWSTR,
    nMaxCustFilter: DWORD,
    nFilterIndex: DWORD,
    lpstrFile: LPWSTR,
    nMaxFile: DWORD,
    lpstrFileTitle: LPWSTR,
    nMaxFileTitle: DWORD,
    lpstrInitialDir: LPCWSTR,
    lpstrTitle: LPCWSTR,
    Flags: DWORD,
    nFileOffset: WORD,
    nFileExtension: WORD,
    lpstrDefExt: LPCWSTR,
    lCustData: LPARAM,
    lpfnHook: *mut c_void,
    lpTemplateName: LPCWSTR,
    pvReserved: *mut c_void,
    dwReserved: DWORD,
    FlagsEx: DWORD,
}

/// WIN32_FIND_DATAW (minwinbase.h).
#[repr(C)]
#[derive(Copy, Clone)]
struct WIN32_FIND_DATAW {
    dwFileAttributes: DWORD,
    ftCreationTime: FILETIME,
    ftLastAccessTime: FILETIME,
    ftLastWriteTime: FILETIME,
    nFileSizeHigh: DWORD,
    nFileSizeLow: DWORD,
    dwReserved0: DWORD,
    dwReserved1: DWORD,
    cFileName: [WCHAR; MAX_PATH],
    cAlternateFileName: [WCHAR; 14],
}

/// MRUINFOW (comctl32_undoc.h). lpfnCompare carries the MRU_BINARY 3-arg comparator here.
#[repr(C)]
struct MRUINFOW {
    cbSize: DWORD,
    uMax: DWORD,
    fFlags: DWORD,
    hKey: HKEY,
    lpszSubKey: LPCWSTR,
    lpfnCompare: Option<unsafe extern "system" fn(LPCVOID, LPCVOID, DWORD) -> INT>,
}

/// SHELLEXECUTEINFOW (shellapi.h). The hIcon/hMonitor union is one HANDLE-sized field.
#[repr(C)]
struct SHELLEXECUTEINFOW {
    cbSize: DWORD,
    fMask: ULONG,
    hwnd: HWND,
    lpVerb: LPCWSTR,
    lpFile: LPCWSTR,
    lpParameters: LPCWSTR,
    lpDirectory: LPCWSTR,
    nShow: c_int,
    hInstApp: HINSTANCE,
    lpIDList: PVOID,
    lpClass: LPCWSTR,
    hkeyClass: HKEY,
    dwHotKey: DWORD,
    hIconOrMonitor: HANDLE,
    hProcess: HANDLE,
}

/// SID_IDENTIFIER_AUTHORITY (setypes.h): { BYTE Value[6]; }
#[repr(C)]
struct SID_IDENTIFIER_AUTHORITY {
    Value: [BYTE; 6],
}

/// UNICODE_STRING / ANSI_STRING (winternl.h) for GUIDFromString*.
#[repr(C)]
struct UNICODE_STRING {
    Length: USHORT,
    MaximumLength: USHORT,
    Buffer: PWSTR,
}
#[repr(C)]
struct ANSI_STRING {
    Length: USHORT,
    MaximumLength: USHORT,
    Buffer: *mut CHAR,
}

// prop-sheet bookkeeping (shellord.c file-private)
#[repr(C)]
struct PSXA {
    uiCount: UINT,
    uiAllocated: UINT,
    pspsx: [*mut IShellPropSheetExt; 1], // flexible array (allocated max_iface entries)
}
type PPSXA = *mut PSXA;
#[repr(C)]
struct PSXA_CALL {
    lpfnAddReplaceWith: LPFNADDPROPSHEETPAGE,
    lParam: LPARAM,
    bCalled: BOOL,
    bMultiple: BOOL,
    uiCount: UINT,
}
type PPSXA_CALL = *mut PSXA_CALL;

// SHLimitInputEdit subclass bookkeeping (shellord.c file-private)
#[repr(C)]
struct UxSubclassInfo {
    hwnd: HWND,
    fnWndProc: WNDPROC,
    pwszValidChars: LPWSTR,
    pwszInvalidChars: LPWSTR,
}

// ── file-local COM vtables (interfaces shellcom does not carry) ──────────────────────────
// Minimal IUnknown with AddRef/Release typed (shellcom's are usize placeholders).
#[repr(C)]
struct IUnk {
    lpVtbl: *mut IUnkVtbl,
}
#[repr(C)]
struct IUnkVtbl {
    QueryInterface: unsafe extern "system" fn(*mut IUnk, REFIID, *mut *mut c_void) -> HRESULT,
    AddRef: unsafe extern "system" fn(*mut IUnk) -> ULONG,
    Release: unsafe extern "system" fn(*mut IUnk) -> ULONG,
}

#[repr(C)]
struct IShellPropSheetExt {
    lpVtbl: *mut IShellPropSheetExtVtbl,
}
#[repr(C)]
struct IShellPropSheetExtVtbl {
    QueryInterface:
        unsafe extern "system" fn(*mut IShellPropSheetExt, REFIID, *mut *mut c_void) -> HRESULT,
    AddRef: usize,
    Release: unsafe extern "system" fn(*mut IShellPropSheetExt) -> ULONG,
    AddPages:
        unsafe extern "system" fn(*mut IShellPropSheetExt, LPFNADDPROPSHEETPAGE, LPARAM) -> HRESULT,
    ReplacePage: unsafe extern "system" fn(
        *mut IShellPropSheetExt,
        UINT,
        LPFNADDPROPSHEETPAGE,
        LPARAM,
    ) -> HRESULT,
}

#[repr(C)]
struct IShellExtInit {
    lpVtbl: *mut IShellExtInitVtbl,
}
#[repr(C)]
struct IShellExtInitVtbl {
    QueryInterface: usize,
    AddRef: usize,
    Release: unsafe extern "system" fn(*mut IShellExtInit) -> ULONG,
    Initialize: unsafe extern "system" fn(
        *mut IShellExtInit,
        PCIDLIST_ABSOLUTE,
        LPDATAOBJECT,
        HKEY,
    ) -> HRESULT,
}

#[repr(C)]
struct IItemNameLimits {
    lpVtbl: *mut IItemNameLimitsVtbl,
}
#[repr(C)]
struct IItemNameLimitsVtbl {
    QueryInterface: usize,
    AddRef: usize,
    Release: unsafe extern "system" fn(*mut IItemNameLimits) -> ULONG,
    GetValidCharacters:
        unsafe extern "system" fn(*mut IItemNameLimits, *mut LPWSTR, *mut LPWSTR) -> HRESULT,
    GetMaxLength: usize,
}

#[repr(C)]
struct IShellLinkW {
    lpVtbl: *mut IShellLinkWVtbl,
}
#[repr(C)]
struct IShellLinkWVtbl {
    QueryInterface: unsafe extern "system" fn(*mut IShellLinkW, REFIID, *mut *mut c_void) -> HRESULT,
    AddRef: usize,
    Release: unsafe extern "system" fn(*mut IShellLinkW) -> ULONG,
    GetPath: unsafe extern "system" fn(
        *mut IShellLinkW,
        LPWSTR,
        c_int,
        *mut WIN32_FIND_DATAW,
        DWORD,
    ) -> HRESULT,
    GetIDList: usize,
    SetIDList: unsafe extern "system" fn(*mut IShellLinkW, LPCITEMIDLIST) -> HRESULT,
    GetDescription: usize,
    SetDescription: unsafe extern "system" fn(*mut IShellLinkW, LPCWSTR) -> HRESULT,
    GetWorkingDirectory: usize,
    SetWorkingDirectory: usize,
    GetArguments: usize,
    SetArguments: usize,
    GetHotkey: usize,
    SetHotkey: usize,
    GetShowCmd: usize,
    SetShowCmd: usize,
    GetIconLocation: usize,
    SetIconLocation: usize,
    SetRelativePath: usize,
    Resolve: usize,
    SetPath: unsafe extern "system" fn(*mut IShellLinkW, LPCWSTR) -> HRESULT,
}

#[repr(C)]
struct IPersistFile {
    lpVtbl: *mut IPersistFileVtbl,
}
#[repr(C)]
struct IPersistFileVtbl {
    QueryInterface: usize,
    AddRef: usize,
    Release: unsafe extern "system" fn(*mut IPersistFile) -> ULONG,
    GetClassID: usize,
    IsDirty: usize,
    Load: usize,
    Save: unsafe extern "system" fn(*mut IPersistFile, LPCOLESTR, BOOL) -> HRESULT,
    SaveCompleted: unsafe extern "system" fn(*mut IPersistFile, LPCOLESTR) -> HRESULT,
    GetCurFile: usize,
}

// ── helpers ─────────────────────────────────────────────────────────────────────────────
#[inline]
fn SUCCEEDED(hr: HRESULT) -> bool {
    hr >= 0
}
#[inline]
fn FAILED(hr: HRESULT) -> bool {
    hr < 0
}
#[inline]
fn MAKELONG(a: DWORD, b: DWORD) -> DWORD {
    (a & 0xFFFF) | ((b & 0xFFFF) << 16)
}
#[inline]
fn LOWORD(l: usize) -> UINT {
    (l & 0xFFFF) as UINT
}
#[inline]
fn IS_INTRESOURCE(i: usize) -> bool {
    (i >> 16) == 0
}
#[inline]
unsafe fn hkey(v: u32) -> HKEY {
    // Predefined HKEYs are (HKEY)(LONG)0x8000000x -> sign-extend on 64-bit.
    (v as i32 as isize as *mut c_void).cast()
}
#[inline]
unsafe fn invalid_handle() -> HANDLE {
    -1isize as *mut c_void
}
/// Compile-time ASCII -> UTF-16 (NUL-terminated). N must exceed the string length; trailing
/// zeros past the first NUL are harmless for the pointer consumers below.
const fn wide<const N: usize>(s: &str) -> [u16; N] {
    let b = s.as_bytes();
    let mut out = [0u16; N];
    let mut i = 0;
    while i < b.len() && i < N - 1 {
        out[i] = b[i] as u16;
        i += 1;
    }
    out
}

// wide-string literals (NUL-terminated). 96 wide chars covers every literal used here.
static W_COMDLG32: [u16; 32] = wide("comdlg32.dll");
static W_ADVANCED: [u16; 32] = wide("Advanced");
static W_HIDDEN: [u16; 16] = wide("Hidden");
static W_HIDEFILEEXT: [u16; 16] = wide("HideFileExt");
static W_SHOWCOMPCOLOR: [u16; 16] = wide("ShowCompColor");
static W_DONTPRETTYPATH: [u16; 16] = wide("DontPrettyPath");
static W_SHOWATTRIBCOL: [u16; 16] = wide("ShowAttribCol");
static W_MAPNETDRVBTN: [u16; 16] = wide("MapNetDrvBtn");
static W_SHOWINFOTIP: [u16; 16] = wide("ShowInfoTip");
static W_HIDEICONS: [u16; 16] = wide("HideIcons");
static W_WEBVIEW: [u16; 16] = wide("WebView");
static W_FILTER: [u16; 16] = wide("Filter");
static W_SHOWSUPERHIDDEN: [u16; 16] = wide("ShowSuperHidden");
static W_NONETCRAWLING: [u16; 16] = wide("NoNetCrawling");
static W_SEPARATEPROCESS: [u16; 24] = wide("SeparateProcess");
static W_AUTOCHECKSELECT: [u16; 24] = wide("AutoCheckSelect");
static W_ICONSONLY: [u16; 16] = wide("IconsOnly");
static W_SHOWTYPEOVERLAY: [u16; 24] = wide("ShowTypeOverlay");
static W_SHOWSTATUSBAR: [u16; 16] = wide("ShowStatusBar");
static W_SUPERHIDDEN: [u16; 16] = wide("SuperHidden");
static W_SHELLSTATE: [u16; 16] = wide("ShellState");
static W_SETTINGS: [u16; 16] = wide("Settings");
static W_EXPLORER_PATH: [u16; 64] = wide("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer");
static W_CABINETSTATE_PATH: [u16; 80] =
    wide("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\CabinetState");
static W_RECENTDOCS: [u16; 16] = wide("RecentDocs");
static W_PROPSHEETHANDLERS: [u16; 48] = wide("shellex\\PropertySheetHandlers");
static W_STAR_LNK: [u16; 8] = wide("*.lnk");
static W_DOT_LNK: [u16; 8] = wide(".lnk");
static W_DOT_EXE: [u16; 8] = wide(".exe");
static W_SHORTCUT_TO: [u16; 24] = wide("Shortcut to ");
static W_FMT_NEWLINK: [u16; 8] = wide("%s%s%s");
static W_FMT_NUM_NEWLINK: [u16; 16] = wide("%s (%d)%s%s");
static W_DOT: [u16; 2] = wide(".");
static W_EMPTY: [u16; 1] = wide("");
static W_EXT_URL: [u16; 4] = wide("url");
static W_EXT_LNK: [u16; 4] = wide("lnk");
// shellapi.h; the island predates these two (WinVista+ / Win7+).
const SHGNLI_NOLOCNAME: UINT = 0x10;
const SHGNLI_USEURLEXT: UINT = 0x20;
static W_UXSUBCLASSINFO: [u16; 24] = wide("UxSubclassInfo");
static W_OPEN: [u16; 8] = wide("open");
static W_RUNDLL32: [u16; 16] = wide("rundll32.exe");
static W_CONTROL_RUNDLL: [u16; 32] = wide("shell32.dll,Control_RunDLL ");
static W_PROPERTIES: [u16; 16] = wide("properties");
static W_EXPLORER_EXE: [u16; 16] = wide("explorer.exe");
static W_FINDFILES_CLSID: [u16; 48] = wide("::{E17D4FC0-5564-11D1-83F2-00A0C90DC849}");
static A_GETOPENFILENAMEW: [u8; 17] = *b"GetOpenFileNameW\0";

// ── imports (WINAPI -> extern "system", `@N`-decorated) ──────────────────────────────────
extern "system" {
    fn AddMRUData(hList: HANDLE, lpData: LPCVOID, cbData: DWORD) -> INT;
    fn AllocateAndInitializeSid(
        pIdentifierAuthority: *mut SID_IDENTIFIER_AUTHORITY,
        nSubAuthorityCount: BYTE,
        nSubAuthority0: DWORD,
        nSubAuthority1: DWORD,
        nSubAuthority2: DWORD,
        nSubAuthority3: DWORD,
        nSubAuthority4: DWORD,
        nSubAuthority5: DWORD,
        nSubAuthority6: DWORD,
        nSubAuthority7: DWORD,
        pSid: *mut PVOID,
    ) -> BOOL;
    fn CheckTokenMembership(TokenHandle: HANDLE, SidToCheck: PVOID, IsMember: *mut BOOL) -> BOOL;
    fn CloseClipboard() -> BOOL;
    fn CoFreeUnusedLibraries();
    fn CoInitialize(pvReserved: LPVOID) -> HRESULT;
    fn CoTaskMemFree(pv: LPVOID);
    fn CoUninitialize();
    fn DeleteFileW(name: LPCWSTR) -> BOOL;
    pub fn DoDragDrop(
        pDataObj: *mut c_void,
        pDropSource: *mut c_void,
        dwOKEffects: DWORD,
        pdwEffect: LPDWORD,
    ) -> HRESULT;
    fn ExpandEnvironmentStringsA(src: LPCSTR, dst: LPSTR, count: DWORD) -> DWORD;
    fn ExpandEnvironmentStringsW(src: LPCWSTR, dst: LPWSTR, count: DWORD) -> DWORD;
    fn FindClose(h: HANDLE) -> BOOL;
    fn FindMRUData(hList: HANDLE, lpData: LPCVOID, cbData: DWORD, lpRegNum: *mut INT) -> INT;
    fn FreeMRUList(hList: HANDLE);
    fn FreeSid(pSid: PVOID) -> PVOID;
    fn GetClipboardData(fmt: UINT) -> HANDLE;
    fn GetDriveTypeW(root: LPCWSTR) -> UINT;
    fn GetFileAttributesW(name: LPCWSTR) -> DWORD;
    fn GetFullPathNameW(name: LPCWSTR, len: DWORD, buf: LPWSTR, part: *mut LPWSTR) -> DWORD;
    fn GetKeyState(vk: c_int) -> SHORT;
    fn GetLongPathNameW(short_path: LPCWSTR, long_path: LPWSTR, len: DWORD) -> DWORD;
    fn GetProcessHeap() -> HANDLE;
    fn GetTempPathW(len: DWORD, buf: LPWSTR) -> DWORD;
    fn GetVersion() -> DWORD;
    fn GlobalLock(h: HANDLE) -> LPVOID;
    fn GlobalUnlock(h: HANDLE) -> BOOL;
    fn HeapFree(h: HANDLE, flags: DWORD, mem: LPVOID) -> BOOL;
    fn ILCreateFromPathW(path: LPCWSTR) -> LPITEMIDLIST;
    fn IShellLink_ConstructFromPath(path: LPCWSTR, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn LocalFree(h: HANDLE) -> HANDLE;
    fn MessageBeep(t: UINT) -> BOOL;
    fn MultiByteToWideChar(
        cp: UINT,
        flags: DWORD,
        src: LPCSTR,
        srclen: c_int,
        dst: LPWSTR,
        dstlen: c_int,
    ) -> c_int;
    fn OleFlushClipboard() -> HRESULT;
    fn OleInitialize(pvReserved: LPVOID) -> HRESULT;
    fn PathAddBackslashW(path: LPWSTR) -> LPWSTR;
    fn PathAppendW(path: LPWSTR, more: LPCWSTR) -> BOOL;
    fn PathFileExistsW(path: LPCWSTR) -> BOOL;
    fn PathFindExtensionW(path: LPCWSTR) -> LPWSTR;
    fn PathFindFileNameW(path: LPCWSTR) -> LPWSTR;
    fn PathIsDirectoryW(path: LPCWSTR) -> BOOL;
    fn PathIsEqualOrSubFolder(a: LPCWSTR, b: LPCWSTR) -> BOOL;
    pub fn RegisterDragDrop(h: HWND, pDropTarget: *mut c_void) -> HRESULT;
    pub fn RevokeDragDrop(h: HWND) -> HRESULT;
    fn SHAnsiToUnicode(src: LPCSTR, dst: LPWSTR, cch: c_int) -> DWORD;
    fn SHCLSIDFromStringW(s: LPCWSTR, clsid: *mut CLSID) -> HRESULT;
    fn SHGlobalCounterGetValue(h: HANDLE) -> LONG;
    fn SHGlobalCounterIncrement(h: HANDLE) -> LONG;
    fn SHRestricted(policy: DWORD) -> DWORD;
    fn SHStartNetConnectionDialogA(h: HWND, name: LPCSTR, ty: DWORD) -> HRESULT;
    fn SHStartNetConnectionDialogW(h: HWND, name: LPCWSTR, ty: DWORD) -> HRESULT;
    fn SHStrDupW(src: LPCWSTR, dst: *mut LPWSTR) -> HRESULT;
    fn SetClipboardData(fmt: UINT, h: HANDLE) -> HANDLE;
    fn SetTaskmanWindow(h: HWND) -> BOOL;
    fn Shell_GetImageLists(large: *mut HIMAGELIST, small: *mut HIMAGELIST) -> BOOL;
    fn StrCatBuffW(dst: LPWSTR, src: LPCWSTR, cch: c_int) -> LPWSTR;
    fn WideCharToMultiByte(
        cp: UINT,
        flags: DWORD,
        src: LPCWSTR,
        srclen: c_int,
        dst: LPSTR,
        dstlen: c_int,
        defchar: LPCSTR,
        used: *mut BOOL,
    ) -> c_int;
    fn lstrcmpiW(a: LPCWSTR, b: LPCWSTR) -> c_int;
    fn lstrcpyW(dst: LPWSTR, src: LPCWSTR) -> LPWSTR;
    fn lstrcpynW(dst: LPWSTR, src: LPCWSTR, n: c_int) -> LPWSTR;
    fn lstrlenW(s: LPCWSTR) -> c_int;
    // ntdll (Rtl*), stdcall:
    fn RtlAnsiStringToUnicodeString(
        dst: *mut UNICODE_STRING,
        src: *const ANSI_STRING,
        alloc: BOOLEAN,
    ) -> LONG;
    fn RtlGUIDFromString(s: *const UNICODE_STRING, guid: *mut GUID) -> LONG;
    fn RtlInitAnsiString(dst: *mut ANSI_STRING, src: LPCSTR);
    fn RtlInitUnicodeString(dst: *mut UNICODE_STRING, src: LPCWSTR);
}

// GetWindowLongPtr/SetWindowLongPtr: real exports differ per pointer width.
#[cfg(target_pointer_width = "32")]
extern "system" {
}
#[cfg(target_pointer_width = "64")]
extern "system" {
    fn GetWindowLongPtrW(h: HWND, idx: c_int) -> LONG_PTR;
    fn SetWindowLongPtrW(h: HWND, idx: c_int, val: LONG_PTR) -> LONG_PTR;
}
#[cfg(target_pointer_width = "32")]
#[inline]
unsafe fn get_window_long_ptr(h: HWND, idx: c_int) -> LONG_PTR {
    GetWindowLongW(h, idx) as LONG_PTR
}
#[cfg(target_pointer_width = "32")]
#[inline]
unsafe fn set_window_long_ptr(h: HWND, idx: c_int, v: LONG_PTR) -> LONG_PTR {
    SetWindowLongW(h, idx, v as LONG) as LONG_PTR
}
#[cfg(target_pointer_width = "64")]
#[inline]
unsafe fn get_window_long_ptr(h: HWND, idx: c_int) -> LONG_PTR {
    GetWindowLongPtrW(h, idx)
}
#[cfg(target_pointer_width = "64")]
#[inline]
unsafe fn set_window_long_ptr(h: HWND, idx: c_int, v: LONG_PTR) -> LONG_PTR {
    SetWindowLongPtrW(h, idx, v)
}

// ── imports (cdecl -> extern "C"): internal shell32 helpers + COM ctors + msvcrt + data ──
extern "C" {
    fn SHELL32_GetDefaultShellState(pss: LPSHELLSTATE);
    fn SHELL32_ReadRegShellState(prss: PREGSHELLSTATE) -> BOOL;
    fn SHELL32_WriteRegShellState(prss: PREGSHELLSTATE) -> LONG;
    fn SHELL_GetCachedGlobalCounter(phGlobalCounter: *mut HANDLE, rguid: REFGUID) -> HANDLE;
    fn SIC_Initialize() -> BOOL;
    // msvcrt
    fn wcscat(dst: *mut WCHAR, src: *const WCHAR) -> *mut WCHAR;
    fn wcschr(s: *const WCHAR, c: WCHAR) -> *mut WCHAR;
    fn wcsrchr(s: *const WCHAR, c: WCHAR) -> *mut WCHAR;
    fn wcslen(s: *const WCHAR) -> usize;
    fn _snwprintf(buf: *mut WCHAR, count: usize, fmt: *const WCHAR, ...) -> c_int;
    // data globals (const GUID/IID/CLSID exported by shell32)
    static CLSID_ShellLink: CLSID;
    static IID_IEnumFORMATETC: IID;
    static IID_IItemNameLimits: IID;
    static IID_IPersistFile: IID;
    static IID_IShellExtInit: IID;
    static IID_IShellLinkW: IID;
    static IID_IShellPropSheetExt: IID;
}

#[inline]
unsafe fn SHELL_OsIsUnicode() -> BOOL {
    ((GetVersion() & 0x80000000) == 0) as BOOL
}

// ── data global (external linkage; obj symbol `_g_hShellState`) ──────────────────────────
// SHELL_GCOUNTER_DEFINE_HANDLE(g_hShellState) -> `HANDLE g_hShellState = NULL;`
#[no_mangle]
pub static mut g_hShellState: HANDLE = core::ptr::null_mut();

// ── file-local statics (internal linkage in C) ──────────────────────────────────────────
// SHELL_GCOUNTER_DEFINE_GUID(SHGCGUID_ShellState, 0x7cb834f0,0x527b,0x11d2,0x9d,0x1f,0x00,0x00,0xf8,0x05,0xca,0x57)
static SHGCGUID_ShellState: GUID = GUID {
    Data1: 0x7cb834f0,
    Data2: 0x527b,
    Data3: 0x11d2,
    Data4: [0x9d, 0x1f, 0x00, 0x00, 0xf8, 0x05, 0xca, 0x57],
};
static mut g_ShellStateCounter: LONG = 0;
static mut g_CachedSSF: UINT = 0;
static mut g_ShellState: REGSHELLSTATE = REGSHELLSTATE {
    dwSize: 0,
    ss: SHELLSTATE {
        w0: 0,
        dwWin95Unused: 0,
        uWin95Unused: 0,
        lParamSort: 0,
        iSortDirection: 0,
        version: 0,
        uNotUsed: 0,
        w1: 0,
    },
};
static mut SHELL32_IExplorerInterface: LPUNKNOWN = core::ptr::null_mut();
static mut SHRDD_ole_initialized: BOOL = FALSE; // SHRegisterDragDrop function-local static

// =========================================================================================
//  Exports
// =========================================================================================

/// ParseFieldA [internal]
#[no_mangle]
pub unsafe extern "system" fn ParseFieldA(
    src: LPCSTR,
    mut nField: DWORD,
    mut dst: LPSTR,
    mut len: DWORD,
) -> DWORD {
    if src.is_null() || *src == 0 || dst.is_null() || len == 0 {
        return 0;
    }
    let mut s = src;
    // skip n fields delimited by ','
    while nField > 1 {
        if *s == 0 {
            return FALSE as DWORD;
        }
        let c = *s;
        s = s.add(1);
        if c == b',' as CHAR {
            nField -= 1;
        }
    }
    // copy part till the next ',' to dst
    while *s != 0 && *s != b',' as CHAR && {
        let old = len;
        len = len.wrapping_sub(1);
        old > 0
    } {
        *dst = *s;
        dst = dst.add(1);
        s = s.add(1);
    }
    *dst = 0;
    TRUE as DWORD
}

/// ParseFieldW [internal]
#[no_mangle]
pub unsafe extern "system" fn ParseFieldW(
    src: LPCWSTR,
    mut nField: DWORD,
    mut dst: LPWSTR,
    mut len: DWORD,
) -> DWORD {
    if src.is_null() || *src == 0 || dst.is_null() || len == 0 {
        return 0;
    }
    let mut s = src;
    while nField > 1 {
        if *s == 0 {
            return FALSE as DWORD;
        }
        let c = *s;
        s = s.add(1);
        if c == b',' as WCHAR {
            nField -= 1;
        }
    }
    while *s != 0 && *s != b',' as WCHAR && {
        let old = len;
        len = len.wrapping_sub(1);
        old > 0
    } {
        *dst = *s;
        dst = dst.add(1);
        s = s.add(1);
    }
    *dst = 0;
    TRUE as DWORD
}

/// ParseField [SHELL32.58]
#[no_mangle]
pub unsafe extern "system" fn ParseFieldAW(
    src: LPCVOID,
    nField: DWORD,
    dst: LPVOID,
    len: DWORD,
) -> DWORD {
    if SHELL_OsIsUnicode() != 0 {
        ParseFieldW(src as LPCWSTR, nField, dst as LPWSTR, len)
    } else {
        ParseFieldA(src as LPCSTR, nField, dst as LPSTR, len)
    }
}

/// GetFileNameFromBrowse [SHELL32.63]
#[no_mangle]
pub unsafe extern "system" fn GetFileNameFromBrowse(
    hwndOwner: HWND,
    lpstrFile: LPWSTR,
    nMaxFile: UINT,
    lpstrInitialDir: LPCWSTR,
    lpstrDefExt: LPCWSTR,
    lpstrFilter: LPCWSTR,
    lpstrTitle: LPCWSTR,
) -> BOOL {
    type GetOpenFileNameProc = unsafe extern "system" fn(*mut OPENFILENAMEW) -> BOOL;

    let hmodule = LoadLibraryW(W_COMDLG32.as_ptr());
    if hmodule.is_null() {
        return FALSE;
    }
    let proc = GetProcAddress(hmodule, A_GETOPENFILENAMEW.as_ptr() as LPCSTR);
    if proc.is_none() {
        FreeLibrary(hmodule);
        return FALSE;
    }
    let pGetOpenFileNameW: GetOpenFileNameProc = core::mem::transmute(proc);

    let mut ofn: OPENFILENAMEW = core::mem::zeroed();
    ofn.lStructSize = core::mem::size_of::<OPENFILENAMEW>() as DWORD;
    ofn.hwndOwner = hwndOwner;
    ofn.lpstrFilter = lpstrFilter;
    ofn.lpstrFile = lpstrFile;
    ofn.nMaxFile = nMaxFile;
    ofn.lpstrInitialDir = lpstrInitialDir;
    ofn.lpstrTitle = lpstrTitle;
    ofn.lpstrDefExt = lpstrDefExt;
    ofn.Flags = OFN_EXPLORER | OFN_HIDEREADONLY | OFN_FILEMUSTEXIST;
    let ret = pGetOpenFileNameW(&mut ofn);

    FreeLibrary(hmodule);
    ret
}

/// SHELL_GlobalCounterChanged [internal, cdecl] (active shlwapi_undoc.h #else branch:
/// DECLAREPARAMETERS -> `HANDLE handle`; PARAMETERS -> `handle`; GlobalCounterGet -> SHGlobalCounterGetValue)
#[no_mangle]
pub unsafe extern "C" fn SHELL_GlobalCounterChanged(pCounter: *mut LONG, handle: HANDLE) -> BOOL {
    let count = SHGlobalCounterGetValue(handle);
    if *pCounter == count {
        return FALSE;
    }
    *pCounter = count;
    TRUE
}

// The export takes the header's SHELLSTATE (the island's bindgen bitfields), which is what every
// caller holds; the body works on this file's bit-exact model of the same 32 bytes.
const _: () = assert!(
    core::mem::size_of::<SHELLSTATE>() == core::mem::size_of::<crate::island::SHELLSTATE>()
        && core::mem::align_of::<SHELLSTATE>() == core::mem::align_of::<crate::island::SHELLSTATE>()
);

/// SHGetSetSettings [SHELL32.68]
#[no_mangle]
pub unsafe extern "system" fn SHGetSetSettings(
    lpss: *mut crate::island::SHELLSTATE,
    dwMask: DWORD,
    bSet: BOOL,
) {
    let lpss: LPSHELLSTATE = lpss.cast();
    let inverted: DWORD = SSF_SHOWEXTENSIONS;
    let gpss: LPSHELLSTATE = &mut g_ShellState.ss;

    if g_hShellState.is_null() {
        // SHELL_GlobalCounterCreate(&SHGCGUID_ShellState, g_hShellState)
        SHELL_GetCachedGlobalCounter(&mut g_hShellState, &SHGCGUID_ShellState);
    }

    if lpss.is_null() {
        SHGlobalCounterIncrement(g_hShellState);
        return;
    }

    let hKeyAdv = SHGetShellKey(SHKEY_Root_HKCU | SHKEY_Key_Explorer, W_ADVANCED.as_ptr(), bSet);
    if hKeyAdv.is_null() && bSet != 0 {
        return;
    }

    // advanced-fields table: (regname, word(0/1), bit, SSF)
    let adv: [(*const u16, u8, u32, DWORD); 16] = [
        (W_HIDEFILEEXT.as_ptr(), 0, SS_SHOWEXTENSIONS, SSF_SHOWEXTENSIONS),
        (W_SHOWCOMPCOLOR.as_ptr(), 0, SS_SHOWCOMPCOLOR, SSF_SHOWCOMPCOLOR),
        (W_DONTPRETTYPATH.as_ptr(), 0, SS_DONTPRETTYPATH, SSF_DONTPRETTYPATH),
        (W_SHOWATTRIBCOL.as_ptr(), 0, SS_SHOWATTRIBCOL, SSF_SHOWATTRIBCOL),
        (W_MAPNETDRVBTN.as_ptr(), 0, SS_MAPNETDRVBTN, SSF_MAPNETDRVBUTTON),
        (W_SHOWINFOTIP.as_ptr(), 0, SS_SHOWINFOTIP, SSF_SHOWINFOTIP),
        (W_HIDEICONS.as_ptr(), 0, SS_HIDEICONS, SSF_HIDEICONS),
        (W_WEBVIEW.as_ptr(), 0, SS_WEBVIEW, SSF_WEBVIEW),
        (W_FILTER.as_ptr(), 0, SS_FILTER, SSF_FILTER),
        (W_SHOWSUPERHIDDEN.as_ptr(), 0, SS_SHOWSUPERHIDDEN, SSF_SHOWSUPERHIDDEN),
        (W_NONETCRAWLING.as_ptr(), 0, SS_NONETCRAWLING, SSF_NONETCRAWLING),
        (W_SEPARATEPROCESS.as_ptr(), 1, SS_SEPPROCESS, SSF_SEPPROCESS),
        (W_AUTOCHECKSELECT.as_ptr(), 1, SS_AUTOCHECKSELECT, ssf_autocheckselect),
        (W_ICONSONLY.as_ptr(), 1, SS_ICONSONLY, ssf_iconsonly),
        (W_SHOWTYPEOVERLAY.as_ptr(), 1, SS_SHOWTYPEOVERLAY, ssf_showtypeoverlay),
        (W_SHOWSTATUSBAR.as_ptr(), 1, SS_SHOWSTATUSBAR, ssf_showstatusbar),
    ];

    // gpss accessors for a (word,bit)
    #[inline]
    unsafe fn gget(p: LPSHELLSTATE, word: u8, bit: u32) -> BOOL {
        if word == 0 {
            (*p).g0(bit)
        } else {
            (*p).g1(bit)
        }
    }
    #[inline]
    unsafe fn gset(p: LPSHELLSTATE, word: u8, bit: u32, v: BOOL) {
        if word == 0 {
            (*p).s0(bit, v)
        } else {
            (*p).s1(bit, v)
        }
    }

    if bSet != 0 {
        let mut changed: DWORD = 0;
        // The whole struct is about to be written to the registry, so every member
        // not yet cached must be read back first, except the ones being set. The
        // pre-fix test was `dwMask & ~g_CachedSSF`, which skipped the backfill
        // whenever the flag being set was already cached and wrote whatever gpss
        // held for everything else (upstream 4fecc908e7d, CORE-20585).
        let notcached: DWORD = !g_CachedSSF & SSF_IMPLEMENTED;
        if (notcached & !dwMask) != 0 {
            let mut tempstate: SHELLSTATE = core::mem::zeroed();
            SHGetSetSettings((&raw mut tempstate).cast(), notcached & !dwMask, FALSE); // read entries not in gpss/g_CachedSSF
        }

        // SHGSS_WriteAdv helper
        unsafe fn write_adv(hKeyAdv: HKEY, name: *const u16, value: DWORD, changed: &mut DWORD) {
            let mut val = value;
            let cb = core::mem::size_of::<DWORD>() as DWORD;
            if SHSetValueW(
                hKeyAdv,
                core::ptr::null(),
                name,
                REG_DWORD,
                &mut val as *mut DWORD as LPVOID,
                cb,
            ) == 0
            {
                *changed += 1;
            }
        }

        if (dwMask & SSF_SHOWALLOBJECTS) != 0
            && (*gpss).g0(SS_SHOWALLOBJECTS) != (*lpss).g0(SS_SHOWALLOBJECTS)
        {
            (*gpss).s0(SS_SHOWALLOBJECTS, (*lpss).g0(SS_SHOWALLOBJECTS));
            write_adv(
                hKeyAdv,
                W_HIDDEN.as_ptr(),
                if (*lpss).g0(SS_SHOWALLOBJECTS) != 0 { 1 } else { 2 },
                &mut changed,
            );
        }
        // SHGSS_SetStruct(fShowSysFiles, SSF_SHOWSYSFILES)
        if (dwMask & SSF_SHOWSYSFILES) != 0
            && (*gpss).g0(SS_SHOWSYSFILES) != (*lpss).g0(SS_SHOWSYSFILES)
        {
            (*gpss).s0(SS_SHOWSYSFILES, (*lpss).g0(SS_SHOWSYSFILES));
            changed += 1;
        }
        // SHGSS_GetSetAdv(SHGSS_SetAdv)
        for &(name, word, bit, ssf) in adv.iter() {
            if (dwMask & ssf) != 0 && gget(gpss, word, bit) != gget(lpss, word, bit) {
                gset(gpss, word, bit, gget(lpss, word, bit));
                let bitval: DWORD = gget(gpss, word, bit) as DWORD;
                let value: DWORD = if (ssf & inverted) != 0 {
                    (bitval == 0) as DWORD
                } else {
                    (bitval != 0) as DWORD
                };
                write_adv(hKeyAdv, name, value, &mut changed);
            }
        }
        // SHGSS_GetSetStruct(SHGSS_SetStruct) — bitfields + the two scalar sort fields
        macro_rules! set_struct_bit {
            ($word:expr, $bit:expr, $ssf:expr) => {
                if (dwMask & $ssf) != 0 && gget(gpss, $word, $bit) != gget(lpss, $word, $bit) {
                    gset(gpss, $word, $bit, gget(lpss, $word, $bit));
                    changed += 1;
                }
            };
        }
        set_struct_bit!(0, SS_NOCONFIRMRECYCLE, SSF_NOCONFIRMRECYCLE);
        set_struct_bit!(0, SS_DOUBLECLICKINWEBVIEW, SSF_DOUBLECLICKINWEBVIEW);
        set_struct_bit!(0, SS_DESKTOPHTML, SSF_DESKTOPHTML);
        set_struct_bit!(0, SS_WIN95CLASSIC, SSF_WIN95CLASSIC);
        if (dwMask & SSF_SORTCOLUMNS) != 0 && (*gpss).lParamSort != (*lpss).lParamSort {
            (*gpss).lParamSort = (*lpss).lParamSort;
            changed += 1;
        }
        if (dwMask & SSF_SORTCOLUMNS) != 0 && (*gpss).iSortDirection != (*lpss).iSortDirection {
            (*gpss).iSortDirection = (*lpss).iSortDirection;
            changed += 1;
        }
        set_struct_bit!(1, SS_STARTPANELON, SSF_STARTPANELON);

        if changed != 0 {
            // (DLL_EXPORT_VERSION) < _WIN32_WINNT_VISTA is TRUE (DLL_EXPORT_VERSION undefined)
            if (dwMask & SSF_SHOWSUPERHIDDEN) != 0 {
                let mut val: DWORD = ((*gpss).g0(SS_SHOWSUPERHIDDEN) != FALSE) as DWORD;
                SHSetValueW(
                    hKeyAdv,
                    core::ptr::null(),
                    W_SUPERHIDDEN.as_ptr(),
                    REG_DWORD,
                    &mut val as *mut DWORD as LPVOID,
                    core::mem::size_of::<DWORD>() as DWORD,
                );
            }
            SHELL32_WriteRegShellState(&mut g_ShellState);
            SHGetSetSettings(core::ptr::null_mut(), 0, TRUE); // invalidate counter
            SHSendMessageBroadcastW(WM_SETTINGCHANGE, 0, W_SHELLSTATE.as_ptr() as LPARAM);
        }
    } else {
        let mut read: DWORD = 0;
        let mut data: DWORD = 0;
        let mut cb: DWORD = 0;
        let mut dummy: DWORD = 0;
        if SHELL_GlobalCounterChanged(&mut g_ShellStateCounter, g_hShellState) != 0 {
            g_CachedSSF = 0;
        }

        // SHGSS_ReadAdv(L"Hidden", SSF_SHOWALLOBJECTS | SSF_SHOWSYSFILES)
        let hidden_mask = SSF_SHOWALLOBJECTS | SSF_SHOWSYSFILES;
        if (g_CachedSSF & hidden_mask) != hidden_mask {
            cb = core::mem::size_of::<DWORD>() as DWORD;
            if SHQueryValueExW(
                hKeyAdv,
                W_HIDDEN.as_ptr(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &mut data as *mut DWORD as LPVOID,
                &mut cb,
            ) == 0
                && cb == core::mem::size_of::<DWORD>() as DWORD
            {
                read |= hidden_mask;
                (*gpss).s0(SS_SHOWALLOBJECTS, (data == 1) as BOOL);
                (*gpss).s0(SS_SHOWSYSFILES, (data > 1) as BOOL);
            }
        }
        // SHGSS_GetField(fShowAllObjects, SSF_SHOWALLOBJECTS)
        if (dwMask & SSF_SHOWALLOBJECTS) != 0 {
            (*lpss).s0(SS_SHOWALLOBJECTS, (*gpss).g0(SS_SHOWALLOBJECTS));
            dummy |= SSF_SHOWALLOBJECTS;
        }
        if (dwMask & SSF_SHOWSYSFILES) != 0 {
            (*lpss).s0(SS_SHOWSYSFILES, (*gpss).g0(SS_SHOWSYSFILES));
            dummy |= SSF_SHOWSYSFILES;
        }
        // SHGSS_GetSetAdv(SHGSS_GetAdv)
        for &(name, word, bit, ssf) in adv.iter() {
            let mut readadv = false;
            if (g_CachedSSF & ssf) != ssf {
                cb = core::mem::size_of::<DWORD>() as DWORD;
                if SHQueryValueExW(
                    hKeyAdv,
                    name,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    &mut data as *mut DWORD as LPVOID,
                    &mut cb,
                ) == 0
                    && cb == core::mem::size_of::<DWORD>() as DWORD
                {
                    read |= ssf;
                    readadv = true;
                }
            }
            if readadv {
                let v: BOOL = if (ssf & inverted) != 0 {
                    (data == FALSE_U) as BOOL
                } else {
                    (data != FALSE_U) as BOOL
                };
                gset(gpss, word, bit, v);
            }
            if (dwMask & ssf) != 0 {
                gset(lpss, word, bit, gget(gpss, word, bit));
                read |= ssf;
            }
        }

        if (dwMask & !(read | g_CachedSSF)) != 0 {
            let mut rss: REGSHELLSTATE = core::mem::zeroed();
            if SHELL32_ReadRegShellState(&mut rss) != 0 {
                // SHGSS_GetSetStruct(SHGSS_CacheField): rss.ss -> gpss
                macro_rules! cache_bit {
                    ($word:expr, $bit:expr, $ssf:expr) => {
                        if (dwMask & $ssf) != 0 {
                            gset(gpss, $word, $bit, gget(&mut rss.ss, $word, $bit));
                            read |= $ssf;
                        }
                    };
                }
                cache_bit!(0, SS_NOCONFIRMRECYCLE, SSF_NOCONFIRMRECYCLE);
                cache_bit!(0, SS_DOUBLECLICKINWEBVIEW, SSF_DOUBLECLICKINWEBVIEW);
                cache_bit!(0, SS_DESKTOPHTML, SSF_DESKTOPHTML);
                cache_bit!(0, SS_WIN95CLASSIC, SSF_WIN95CLASSIC);
                if (dwMask & SSF_SORTCOLUMNS) != 0 {
                    (*gpss).lParamSort = rss.ss.lParamSort;
                    read |= SSF_SORTCOLUMNS;
                }
                if (dwMask & SSF_SORTCOLUMNS) != 0 {
                    (*gpss).iSortDirection = rss.ss.iSortDirection;
                    read |= SSF_SORTCOLUMNS;
                }
                cache_bit!(1, SS_STARTPANELON, SSF_STARTPANELON);
            } else {
                SHELL32_GetDefaultShellState(gpss);
                read = 0; // the advanced items we read are no longer valid in gpss
                g_CachedSSF = SSF_STRUCTONLY;
                // HACKFIX (upstream 39ad1b6824f, CORE-20585): persist the defaults
                // when no ShellState value exists yet.
                rss.ss = *gpss;
                SHELL32_WriteRegShellState(&mut rss);
            }
        }
        // SHGSS_GetSetStruct(SHGSS_GetField): gpss -> lpss
        macro_rules! get_bit {
            ($word:expr, $bit:expr, $ssf:expr) => {
                if (dwMask & $ssf) != 0 {
                    gset(lpss, $word, $bit, gget(gpss, $word, $bit));
                    dummy |= $ssf;
                }
            };
        }
        get_bit!(0, SS_NOCONFIRMRECYCLE, SSF_NOCONFIRMRECYCLE);
        get_bit!(0, SS_DOUBLECLICKINWEBVIEW, SSF_DOUBLECLICKINWEBVIEW);
        get_bit!(0, SS_DESKTOPHTML, SSF_DESKTOPHTML);
        get_bit!(0, SS_WIN95CLASSIC, SSF_WIN95CLASSIC);
        if (dwMask & SSF_SORTCOLUMNS) != 0 {
            (*lpss).lParamSort = (*gpss).lParamSort;
            dummy |= SSF_SORTCOLUMNS;
        }
        if (dwMask & SSF_SORTCOLUMNS) != 0 {
            (*lpss).iSortDirection = (*gpss).iSortDirection;
            dummy |= SSF_SORTCOLUMNS;
        }
        get_bit!(1, SS_STARTPANELON, SSF_STARTPANELON);

        g_CachedSSF |= read;
    }
    if !hKeyAdv.is_null() {
        RegCloseKey(hKeyAdv);
    }
}

/// SHGetSettings [SHELL32.@]
#[no_mangle]
pub unsafe extern "system" fn SHGetSettings(lpsfs: LPSHELLFLAGSTATE, dwMask: DWORD) {
    let mut ss: SHELLSTATE = core::mem::zeroed();
    SHGetSetSettings((&raw mut ss).cast(), dwMask & !(SSF_SORTCOLUMNS | SSF_FILTER), FALSE);
    // *lpsfs = *(LPSHELLFLAGSTATE)&ss;  (raw copy of the first 32-bit flag word)
    (*lpsfs).flags = ss.w0;
    if (dwMask & SSF_HIDEICONS) != 0 {
        (*lpsfs).set(SFS_HIDEICONS, ss.g0(SS_HIDEICONS));
    }
    if (dwMask & ssf_autocheckselect) != 0 {
        (*lpsfs).set(SFS_AUTOCHECKSELECT, ss.g1(SS_AUTOCHECKSELECT));
    }
    if (dwMask & ssf_iconsonly) != 0 {
        (*lpsfs).set(SFS_ICONSONLY, ss.g1(SS_ICONSONLY));
    }
}

/// SHShellFolderView_Message [SHELL32.73]
#[no_mangle]
pub unsafe extern "system" fn SHShellFolderView_Message(
    hwndCabinet: HWND,
    uMessage: UINT,
    lParam: LPARAM,
) -> LRESULT {
    0
}

/// RegisterShellHook [SHELL32.181]
#[no_mangle]
pub unsafe extern "system" fn RegisterShellHook(hWnd: HWND, dwType: DWORD) -> BOOL {
    if dwType == 3 {
        SetTaskmanWindow(hWnd);
        return RegisterShellHookWindow(hWnd);
    } else if dwType == 0 {
        return DeregisterShellHookWindow(hWnd);
    }
    FALSE
}

/// ShellMessageBoxA [SHELL32.183], the EXPORTED variadic (cdecl, WINAPIV) entry point.
///
/// Stable Rust cannot DEFINE a C-variadic (`c_variadic`, rust-lang#44930), so this used to be
/// a `va_start` trampoline in rust_varargs_shim.c. It RECEIVES the varargs instead, the
/// va-slots way (`va_slots_rust`, which freeldr's and the kernel's printf family already use):
/// on all three of our targets a variadic call puts its arguments in the same slots as a
/// fixed-arity call with that many parameters (i386 cdecl stack; win64 RCX-R9 then stack;
/// arm64 x0-x7 then stack, where Windows passes varargs in integer registers), and `va_list`
/// is a plain pointer. So the trailing slots are declared as ordinary parameters and staged
/// into an array that a `va_list` points at. FormatMessageA takes a `va_list *` for
/// `Arguments`, which is what `&ap` is.
#[no_mangle]
pub unsafe extern "C" fn ShellMessageBoxA(
    hInstance: HINSTANCE,
    hWnd: HWND,
    lpText: LPCSTR,
    lpCaption: LPCSTR,
    uType: UINT,
    a0: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize,
    a6: usize, a7: usize, a8: usize, a9: usize, a10: usize, a11: usize,
) -> c_int {
    let mut stage = va_slots_rust::stage(&[a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11]);
    let mut ap: *mut c_void = stage.as_mut_ptr() as *mut c_void;
    ShellMessageBoxA_v(
        hInstance,
        hWnd,
        lpText,
        lpCaption,
        uType,
        &mut ap as *mut *mut c_void as *mut c_void,
    )
}

/// The body of `ShellMessageBoxA`; `ap` points at the `va_list` FormatMessageA consumes.
pub unsafe extern "C" fn ShellMessageBoxA_v(
    hInstance: HINSTANCE,
    hWnd: HWND,
    lpText: LPCSTR,
    lpCaption: LPCSTR,
    uType: UINT,
    ap: *mut c_void,
) -> c_int {
    let mut szText: *mut CHAR = core::ptr::null_mut();
    let mut szTitle: [CHAR; 100] = [0; 100];
    let pszText: LPCSTR;
    let mut pszTitle: LPCSTR = szTitle.as_ptr();
    let mut pszTemp: LPSTR = core::ptr::null_mut();
    let ret: c_int;

    if IS_INTRESOURCE(lpCaption as usize) {
        LoadStringA(hInstance, LOWORD(lpCaption as usize), szTitle.as_mut_ptr(), 100);
    } else {
        pszTitle = lpCaption;
    }

    if IS_INTRESOURCE(lpText as usize) {
        // Determine the ANSI length from the Unicode resource.
        let mut ptr: *const WCHAR = core::ptr::null();
        let mut len: UINT = LoadStringW(
            hInstance,
            LOWORD(lpText as usize),
            &mut ptr as *mut *const WCHAR as LPWSTR,
            0,
        ) as UINT;
        len = WideCharToMultiByte(
            CP_ACP,
            0,
            ptr,
            len as c_int,
            core::ptr::null_mut(),
            0,
            core::ptr::null(),
            core::ptr::null_mut(),
        ) as UINT;
        if len != 0 {
            szText = HeapAlloc(
                GetProcessHeap(),
                0,
                (((len + 1) as SIZE_T) * core::mem::size_of::<CHAR>() as SIZE_T) as _,
            ) as *mut CHAR;
            if !szText.is_null() {
                LoadStringA(hInstance, LOWORD(lpText as usize), szText, (len + 1) as c_int);
            }
        }
        pszText = szText;
        if pszText.is_null() {
            return 0;
        }
    } else {
        pszText = lpText;
    }

    // `ap` IS the `va_list *` FormatMessageA wants for `Arguments` (the C shim passed
    // `&ap` of its va_start'ed list) — forward it unchanged.
    FormatMessageA(
        FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_STRING,
        pszText as LPCVOID,
        0,
        0,
        &mut pszTemp as *mut LPSTR as LPSTR,
        0,
        (ap).cast(),
    );

    ret = MessageBoxA(hWnd, pszTemp, pszTitle, uType | MB_SETFOREGROUND);

    HeapFree(GetProcessHeap(), 0, szText as LPVOID);
    LocalFree(pszTemp as HANDLE);
    ret
}

/// SHRegisterDragDrop [SHELL32.86]
#[no_mangle]
pub unsafe extern "system" fn SHRegisterDragDrop(hWnd: HWND, pDropTarget: LPDROPTARGET) -> HRESULT {
    if SHRDD_ole_initialized == FALSE {
        let hr = OleInitialize(core::ptr::null_mut());
        if FAILED(hr) {
            return hr;
        }
        SHRDD_ole_initialized = TRUE;
    }
    RegisterDragDrop(hWnd, pDropTarget)
}

/// SHRevokeDragDrop [SHELL32.87]
#[no_mangle]
pub unsafe extern "system" fn SHRevokeDragDrop(hWnd: HWND) -> HRESULT {
    RevokeDragDrop(hWnd)
}

/// SHDoDragDrop [SHELL32.88] (FIXME-stub forwarding to ole32 DoDragDrop)
#[no_mangle]
pub unsafe extern "system" fn SHDoDragDrop(
    hWnd: HWND,
    lpDataObject: LPDATAOBJECT,
    lpDropSource: LPDROPSOURCE,
    dwOKEffect: DWORD,
    pdwEffect: LPDWORD,
) -> HRESULT {
    DoDragDrop(
        lpDataObject as *mut c_void,
        lpDropSource as *mut c_void,
        dwOKEffect,
        pdwEffect,
    )
}

/// ArrangeWindows [SHELL32.184] (unimplemented in WinXP SP3; stub)
#[no_mangle]
pub unsafe extern "system" fn ArrangeWindows(
    hwndParent: HWND,
    dwReserved: DWORD,
    lpRect: *const RECT,
    cKids: WORD,
    lpKids: *const HWND,
) -> WORD {
    0
}

/// SignalFileOpen [SHELL32.103] (stub)
#[no_mangle]
pub unsafe extern "system" fn SignalFileOpen(pidl: PCIDLIST_ABSOLUTE) -> BOOL {
    FALSE
}

/// SHADD_compare_mru [internal, __REACTOS__ binary comparator]
unsafe extern "system" fn SHADD_compare_mru(data1: LPCVOID, data2: LPCVOID, cbData: DWORD) -> INT {
    let iCmp = lstrcmpiW(data1 as LPCWSTR, data2 as LPCWSTR);
    if iCmp != 0 {
        return iCmp;
    }
    let mut psz1 = data1 as *const WCHAR;
    let mut psz2 = data2 as *const WCHAR;
    psz1 = psz1.add((lstrlenW(psz1) + 1) as usize);
    psz2 = psz2.add((lstrlenW(psz2) + 1) as usize);
    lstrcmpiW(psz1, psz2)
}

/// DoStoreMRUData [internal, __REACTOS__]
unsafe fn DoStoreMRUData(
    pbBuffer: LPBYTE,
    pcbBuffer: LPDWORD,
    pszTargetTitle: LPCWSTR,
    pszTargetPath: LPCWSTR,
    pszLinkTitle: LPCWSTR,
) -> BOOL {
    let mut ib: DWORD = 0;
    let mut cb: DWORD;
    let cchTargetTitle = lstrlenW(pszTargetTitle);
    let cchTargetPath = lstrlenW(pszTargetPath);
    let cchLinkTitle = lstrlenW(pszLinkTitle);

    cb = ((cchTargetTitle + 1 + cchTargetPath + 1 + cchLinkTitle + 2) as DWORD)
        * core::mem::size_of::<WCHAR>() as DWORD;
    if cb > *pcbBuffer {
        return FALSE;
    }

    core::ptr::write_bytes(pbBuffer, 0, *pcbBuffer as usize);

    cb = ((cchTargetTitle + 1) as DWORD) * core::mem::size_of::<WCHAR>() as DWORD;
    if ib + cb > *pcbBuffer {
        return FALSE;
    }
    core::ptr::copy_nonoverlapping(
        pszTargetTitle as *const u8,
        pbBuffer.add(ib as usize),
        cb as usize,
    );
    ib += cb;

    cb = ((cchTargetPath + 1) as DWORD) * core::mem::size_of::<WCHAR>() as DWORD;
    if ib + cb > *pcbBuffer {
        return FALSE;
    }
    core::ptr::copy_nonoverlapping(
        pszTargetPath as *const u8,
        pbBuffer.add(ib as usize),
        cb as usize,
    );
    ib += cb;

    cb = ((cchLinkTitle + 1) as DWORD) * core::mem::size_of::<WCHAR>() as DWORD;
    if ib + cb > *pcbBuffer {
        return FALSE;
    }
    core::ptr::copy_nonoverlapping(
        pszLinkTitle as *const u8,
        pbBuffer.add(ib as usize),
        cb as usize,
    );
    ib += cb;

    *pcbBuffer = ib;
    TRUE
}

/// SHAddToRecentDocs [SHELL32.@] (__REACTOS__ implementation)
#[no_mangle]
pub unsafe extern "system" fn SHAddToRecentDocs(uFlags: UINT, pv: LPCVOID) {
    let mut szTargetPath: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut szLinkDir: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut szLinkFile: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut szDescription: [WCHAR; 80] = [0; 80];
    let mut szPath: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut cbBuffer: DWORD;
    let mut find: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut hExplorerKey: HKEY = core::ptr::null_mut();
    let error: LONG;
    let mut hMRUList: HANDLE = core::ptr::null_mut();
    let mut psl: *mut IShellLinkW = core::ptr::null_mut();
    let mut pPf: *mut IPersistFile = core::ptr::null_mut();
    let mut hr: HRESULT;
    let mut Buffer: [u8; (MAX_PATH + 64) * 2] = [0; (MAX_PATH + 64) * 2];
    let mut ret: INT;

    // check policy
    ret = SHRestricted(REST_NORECENTDOCSHISTORY) as INT;
    if ret != 0 {
        return;
    }

    szTargetPath[0] = 0;
    if !pv.is_null() {
        match uFlags {
            x if x == SHARD_PATHA => {
                MultiByteToWideChar(
                    CP_ACP,
                    0,
                    pv as LPCSTR,
                    -1,
                    szLinkDir.as_mut_ptr(),
                    MAX_PATH as c_int,
                );
                GetFullPathNameW(
                    szLinkDir.as_ptr(),
                    MAX_PATH as DWORD,
                    szTargetPath.as_mut_ptr(),
                    core::ptr::null_mut(),
                );
            }
            x if x == SHARD_PATHW => {
                GetFullPathNameW(
                    pv as LPCWSTR,
                    MAX_PATH as DWORD,
                    szTargetPath.as_mut_ptr(),
                    core::ptr::null_mut(),
                );
            }
            x if x == SHARD_PIDL => {
                SHGetPathFromIDListW(pv as LPCITEMIDLIST, szLinkDir.as_mut_ptr());
                GetFullPathNameW(
                    szLinkDir.as_ptr(),
                    MAX_PATH as DWORD,
                    szTargetPath.as_mut_ptr(),
                    core::ptr::null_mut(),
                );
            }
            _ => {
                return;
            }
        }
    }

    // get recent folder
    if SHGetSpecialFolderPathW(core::ptr::null_mut(), szLinkDir.as_mut_ptr(), CSIDL_RECENT, FALSE)
        == 0
    {
        return;
    }

    // open Explorer key
    error = RegCreateKeyExW(
        hkey(0x80000001), // HKEY_CURRENT_USER
        W_EXPLORER_PATH.as_ptr(),
        0,
        core::ptr::null_mut(),
        0,
        KEY_READ | KEY_WRITE,
        core::ptr::null_mut(),
        &mut hExplorerKey,
        core::ptr::null_mut(),
    );
    if error != 0 {
        return;
    }

    if pv.is_null() {
        lstrcpynW(szLinkFile.as_mut_ptr(), szLinkDir.as_ptr(), MAX_PATH as c_int);
        PathAppendW(szLinkFile.as_mut_ptr(), W_STAR_LNK.as_ptr());

        let hFind = FindFirstFileW(szLinkFile.as_ptr(), (&raw mut find).cast());
        if hFind != invalid_handle() {
            loop {
                lstrcpynW(szLinkFile.as_mut_ptr(), szLinkDir.as_ptr(), MAX_PATH as c_int);
                PathAppendW(szLinkFile.as_mut_ptr(), find.cFileName.as_ptr());
                DeleteFileW(szLinkFile.as_ptr());
                if FindNextFileW(hFind, (&raw mut find).cast()) == 0 {
                    break;
                }
            }
            FindClose(hFind);
        }

        SHDeleteKeyW(hExplorerKey, W_RECENTDOCS.as_ptr());
        RegCloseKey(hExplorerKey);
        return;
    }

    if szTargetPath[0] == 0
        || PathFileExistsW(szTargetPath.as_ptr()) == 0
        || PathIsDirectoryW(szTargetPath.as_ptr()) != 0
    {
        RegCloseKey(hExplorerKey);
        return;
    }

    hr = CoInitialize(core::ptr::null_mut());
    if FAILED(hr) {
        RegCloseKey(hExplorerKey);
        return;
    }

    // check if file is a shortcut
    ret = 0;
    let mut pchDotExt = PathFindExtensionW(szTargetPath.as_ptr());
    while lstrcmpiW(pchDotExt, W_DOT_LNK.as_ptr()) == 0 {
        hr = IShellLink_ConstructFromPath(
            szTargetPath.as_ptr(),
            &IID_IShellLinkW,
            &mut psl as *mut *mut IShellLinkW as *mut LPVOID,
        );
        if FAILED(hr) {
            // goto Quit
            SHAddToRecentDocs_quit(hMRUList, pPf, psl, hExplorerKey);
            return;
        }

        ((*(*psl).lpVtbl).GetPath)(
            psl,
            szPath.as_mut_ptr(),
            MAX_PATH as c_int,
            core::ptr::null_mut(),
            0,
        );
        ((*(*psl).lpVtbl).Release)(psl);
        psl = core::ptr::null_mut();

        lstrcpynW(szTargetPath.as_mut_ptr(), szPath.as_ptr(), MAX_PATH as c_int);
        pchDotExt = PathFindExtensionW(szTargetPath.as_ptr());

        ret += 1;
        if ret >= 8 {
            SHAddToRecentDocs_quit(hMRUList, pPf, psl, hExplorerKey);
            return;
        }
    }
    if lstrcmpiW(pchDotExt, W_DOT_EXE.as_ptr()) == 0 {
        // executables are not added
        SHAddToRecentDocs_quit(hMRUList, pPf, psl, hExplorerKey);
        return;
    }

    // *** JOB 0: Build strings ***
    let pchTargetTitle = PathFindFileNameW(szTargetPath.as_ptr());

    lstrcpyW(szDescription.as_mut_ptr(), W_SHORTCUT_TO.as_ptr());
    StrCatBuffW(szDescription.as_mut_ptr(), pchTargetTitle, 80);

    lstrcpynW(szLinkFile.as_mut_ptr(), szLinkDir.as_ptr(), MAX_PATH as c_int);
    PathAppendW(szLinkFile.as_mut_ptr(), pchTargetTitle);
    StrCatBuffW(szLinkFile.as_mut_ptr(), W_DOT_LNK.as_ptr(), MAX_PATH as c_int);
    let pchLinkTitle = PathFindFileNameW(szLinkFile.as_ptr());

    // *** JOB 1: registry RecentDocs MRU ***
    cbBuffer = core::mem::size_of_val(&Buffer) as DWORD;
    ret = DoStoreMRUData(
        Buffer.as_mut_ptr(),
        &mut cbBuffer,
        pchTargetTitle,
        szTargetPath.as_ptr(),
        pchLinkTitle,
    );
    if ret == 0 {
        SHAddToRecentDocs_quit(hMRUList, pPf, psl, hExplorerKey);
        return;
    }

    let mut mru: MRUINFOW = core::mem::zeroed();
    mru.cbSize = core::mem::size_of::<MRUINFOW>() as DWORD;
    mru.uMax = 16;
    mru.fFlags = MRU_BINARY | MRU_CACHEWRITE;
    mru.hKey = hExplorerKey;
    mru.lpszSubKey = W_RECENTDOCS.as_ptr();
    mru.lpfnCompare = Some(SHADD_compare_mru);
    hMRUList = CreateMRUListW((&raw mut mru).cast());
    if hMRUList.is_null() {
        SHAddToRecentDocs_quit(hMRUList, pPf, psl, hExplorerKey);
        return;
    }

    // already exists?
    ret = FindMRUData(hMRUList, Buffer.as_ptr() as LPCVOID, cbBuffer, core::ptr::null_mut());
    if ret >= 0 {
        let hFile = CreateFileW(
            szLinkFile.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            core::ptr::null_mut(),
        );
        if hFile != invalid_handle() {
            CloseHandle(hFile);
            SHAddToRecentDocs_quit(hMRUList, pPf, psl, hExplorerKey);
            return;
        }
    }

    // add MRU data
    ret = AddMRUData(hMRUList, Buffer.as_ptr() as LPCVOID, cbBuffer);
    if ret < 0 {
        SHAddToRecentDocs_quit(hMRUList, pPf, psl, hExplorerKey);
        return;
    }

    // *** JOB 2: create shortcut ***
    hr = CoCreateInstance(
        &CLSID_ShellLink,
        core::ptr::null_mut(),
        CLSCTX_INPROC_SERVER,
        &IID_IShellLinkW,
        &mut psl as *mut *mut IShellLinkW as *mut LPVOID,
    );
    if FAILED(hr) {
        SHAddToRecentDocs_quit(hMRUList, pPf, psl, hExplorerKey);
        return;
    }

    hr = ((*(*psl).lpVtbl).QueryInterface)(
        psl,
        &IID_IPersistFile,
        &mut pPf as *mut *mut IPersistFile as *mut *mut c_void,
    );
    if FAILED(hr) {
        SHAddToRecentDocs_quit(hMRUList, pPf, psl, hExplorerKey);
        return;
    }

    if uFlags == SHARD_PIDL {
        hr = ((*(*psl).lpVtbl).SetIDList)(psl, pv as LPCITEMIDLIST);
    } else {
        hr = ((*(*psl).lpVtbl).SetPath)(psl, pv as LPCWSTR);
    }

    ((*(*psl).lpVtbl).SetDescription)(psl, szDescription.as_ptr());

    hr = ((*(*pPf).lpVtbl).Save)(pPf, szLinkFile.as_ptr(), TRUE);
    // (failure only logged in C)
    hr = ((*(*pPf).lpVtbl).SaveCompleted)(pPf, szLinkFile.as_ptr());

    SHAddToRecentDocs_quit(hMRUList, pPf, psl, hExplorerKey);
}

/// Quit: cleanup block of SHAddToRecentDocs.
unsafe fn SHAddToRecentDocs_quit(
    hMRUList: HANDLE,
    pPf: *mut IPersistFile,
    psl: *mut IShellLinkW,
    hExplorerKey: HKEY,
) {
    if !hMRUList.is_null() {
        FreeMRUList(hMRUList);
    }
    if !pPf.is_null() {
        ((*(*pPf).lpVtbl).Release)(pPf);
    }
    if !psl.is_null() {
        ((*(*psl).lpVtbl).Release)(psl);
    }
    CoUninitialize();
    RegCloseKey(hExplorerKey);
}

/// SHWinHelp [SHELL32.127]
#[no_mangle]
pub unsafe extern "system" fn SHWinHelp(
    hwnd: HWND,
    pszHelp: LPCWSTR,
    uCommand: UINT,
    dwData: ULONG_PTR,
) -> HRESULT {
    if WinHelpW(hwnd, pszHelp, uCommand, dwData) == 0 {
        return FALSE;
    }
    TRUE
}

/// SHRunControlPanel [SHELL32.161] (__REACTOS__)
#[no_mangle]
pub unsafe extern "system" fn SHRunControlPanel(commandLine: LPCWSTR, parent: HWND) -> BOOL {
    let mut parameters: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    // initialize with L"shell32.dll,Control_RunDLL "
    let init_len = wcslen(W_CONTROL_RUNDLL.as_ptr());
    core::ptr::copy_nonoverlapping(W_CONTROL_RUNDLL.as_ptr(), parameters.as_mut_ptr(), init_len + 1);
    // IShellDispatch::ControlPanelItem passes a script-supplied BSTR straight through,
    // and a null one would crash wcscat (upstream a83411cfefe).
    if commandLine.is_null() {
        return FALSE;
    }
    wcscat(parameters.as_mut_ptr(), commandLine);

    (ShellExecuteW(
        parent,
        W_OPEN.as_ptr(),
        W_RUNDLL32.as_ptr(),
        parameters.as_ptr(),
        core::ptr::null(),
        SW_SHOWNORMAL,
    ) as INT_PTR
        > 32) as BOOL
}

/// SHSetInstanceExplorer [SHELL32.176]
#[no_mangle]
pub unsafe extern "system" fn SHSetInstanceExplorer(lpUnknown: LPUNKNOWN) {
    SHELL32_IExplorerInterface = lpUnknown;
}

/// SHGetInstanceExplorer [SHELL32.@]
#[no_mangle]
pub unsafe extern "system" fn SHGetInstanceExplorer(lpUnknown: *mut LPUNKNOWN) -> HRESULT {
    *lpUnknown = SHELL32_IExplorerInterface;
    if SHELL32_IExplorerInterface.is_null() {
        return E_FAIL;
    }
    // IUnknown_AddRef
    let p = SHELL32_IExplorerInterface as *mut IUnk;
    ((*(*p).lpVtbl).AddRef)(p);
    S_OK
}

/// SHFreeUnusedLibraries [SHELL32.123]
#[no_mangle]
pub unsafe extern "system" fn SHFreeUnusedLibraries() {
    CoFreeUnusedLibraries();
}

/// DAD_AutoScroll [SHELL32.129] (stub)
#[no_mangle]
pub unsafe extern "system" fn DAD_AutoScroll(
    hwnd: HWND,
    samples: *mut AUTO_SCROLL_DATA,
    pt: *const POINT,
) -> BOOL {
    FALSE
}

/// DAD_DragEnter [SHELL32.130] (stub)
#[no_mangle]
pub unsafe extern "system" fn DAD_DragEnter(hwnd: HWND) -> BOOL {
    FALSE
}

/// DAD_DragEnterEx [SHELL32.131] (stub)
#[no_mangle]
pub unsafe extern "system" fn DAD_DragEnterEx(hwnd: HWND, p: POINT) -> BOOL {
    FALSE
}

/// DAD_DragMove [SHELL32.134] (stub)
#[no_mangle]
pub unsafe extern "system" fn DAD_DragMove(p: POINT) -> BOOL {
    FALSE
}

/// DAD_DragLeave [SHELL32.132] (stub)
#[no_mangle]
pub unsafe extern "system" fn DAD_DragLeave() -> BOOL {
    FALSE
}

/// DAD_SetDragImage [SHELL32.136] (stub)
#[no_mangle]
pub unsafe extern "system" fn DAD_SetDragImage(himlTrack: HIMAGELIST, lppt: LPPOINT) -> BOOL {
    FALSE
}

/// DAD_ShowDragImage [SHELL32.137] (stub)
#[no_mangle]
pub unsafe extern "system" fn DAD_ShowDragImage(bShow: BOOL) -> BOOL {
    FALSE
}

/// ReadCabinetState [SHELL32.651]
#[no_mangle]
pub unsafe extern "system" fn ReadCabinetState(cs: *mut CABINETSTATE, mut length: c_int) -> BOOL {
    let mut hk: HKEY = core::ptr::null_mut();
    let mut ty: DWORD;
    let mut r: DWORD;

    if cs.is_null() || length < core::mem::size_of::<CABINETSTATE>() as c_int {
        return FALSE;
    }

    r = RegOpenKeyW(hkey(0x80000001), W_CABINETSTATE_PATH.as_ptr(), &mut hk) as DWORD;
    if r == 0 {
        ty = REG_BINARY;
        r = RegQueryValueExW(
            hk,
            W_SETTINGS.as_ptr(),
            core::ptr::null_mut(),
            &mut ty,
            cs as LPBYTE,
            &mut length as *mut c_int as LPDWORD,
        ) as DWORD;
        RegCloseKey(hk);
    }

    if r != 0
        || ((*cs).cLength as usize) < core::mem::size_of::<CABINETSTATE>()
        || (*cs).cLength as c_int != length
    {
        let mut shellstate: SHELLSTATE = core::mem::zeroed();
        shellstate.s0(SS_WIN95CLASSIC, FALSE);
        SHGetSetSettings((&raw mut shellstate).cast(), SSF_WIN95CLASSIC, FALSE);

        core::ptr::write_bytes(cs as *mut u8, 0, core::mem::size_of::<CABINETSTATE>());
        (*cs).cLength = core::mem::size_of::<CABINETSTATE>() as WORD;
        (*cs).nVersion = 2;
        (*cs).set(CB_FULLPATHTITLE, FALSE);
        (*cs).set(CB_SAVELOCALVIEW, TRUE);
        (*cs).set(CB_NOTSHELL, FALSE);
        (*cs).set(CB_SIMPLEDEFAULT, TRUE);
        (*cs).set(CB_DONTSHOWDESCBAR, FALSE);
        (*cs).set(CB_NEWWINDOWMODE, shellstate.g0(SS_WIN95CLASSIC));
        (*cs).set(CB_SHOWCOMPCOLOR, FALSE);
        (*cs).set(CB_DONTPRETTYNAMES, FALSE);
        (*cs).set(CB_ADMINSCREATECOMMONGROUPS, TRUE);
        (*cs).fMenuEnumFilter = SHCONTF_FOLDERS | SHCONTF_NONFOLDERS;
    }

    TRUE
}

/// WriteCabinetState [SHELL32.652]
#[no_mangle]
pub unsafe extern "system" fn WriteCabinetState(cs: *mut CABINETSTATE) -> BOOL {
    let mut r: DWORD;
    let mut hk: HKEY = core::ptr::null_mut();

    if cs.is_null() {
        return FALSE;
    }

    r = RegCreateKeyExW(
        hkey(0x80000001),
        W_CABINETSTATE_PATH.as_ptr(),
        0,
        core::ptr::null_mut(),
        0,
        KEY_ALL_ACCESS,
        core::ptr::null_mut(),
        &mut hk,
        core::ptr::null_mut(),
    ) as DWORD;
    if r == 0 {
        r = RegSetValueExW(
            hk,
            W_SETTINGS.as_ptr(),
            0,
            REG_BINARY,
            cs as *const BYTE,
            (*cs).cLength as DWORD,
        ) as DWORD;
        RegCloseKey(hk);
    }
    (r == 0) as BOOL
}

/// FileIconInit [SHELL32.660]
#[no_mangle]
pub unsafe extern "system" fn FileIconInit(bFullInit: BOOL) -> BOOL {
    SIC_Initialize()
}

/// SetAppStartingCursor [SHELL32.99] (stub)
#[no_mangle]
pub unsafe extern "system" fn SetAppStartingCursor(u: HWND, v: DWORD) -> HRESULT {
    0
}

/// SHLoadOLE [SHELL32.151] (stub)
#[no_mangle]
pub unsafe extern "system" fn SHLoadOLE(lParam: LPARAM) -> HRESULT {
    S_OK
}

/// DriveType [SHELL32.64]
#[no_mangle]
pub unsafe extern "system" fn DriveType(DriveType: c_int) -> c_int {
    let mut root: [WCHAR; 4] = wide("A:\\");
    root[0] = (b'A' as c_int + DriveType) as WCHAR;
    GetDriveTypeW(root.as_ptr()) as c_int
}

/// InvalidateDriveType [SHELL32.65] (stub)
#[no_mangle]
pub unsafe extern "system" fn InvalidateDriveType(u: c_int) -> c_int {
    0
}

/// SHAbortInvokeCommand [SHELL32.198] (stub)
#[no_mangle]
pub unsafe extern "system" fn SHAbortInvokeCommand() -> HRESULT {
    1
}

/// SHOutOfMemoryMessageBox [SHELL32.126] (stub)
#[no_mangle]
pub unsafe extern "system" fn SHOutOfMemoryMessageBox(
    hwndOwner: HWND,
    lpCaption: LPCSTR,
    uType: UINT,
) -> c_int {
    0
}

/// SHFlushClipboard [SHELL32.121]
#[no_mangle]
pub unsafe extern "system" fn SHFlushClipboard() -> HRESULT {
    OleFlushClipboard()
}

/// SHWaitForFileToOpen [SHELL32.97] (stub)
#[no_mangle]
pub unsafe extern "system" fn SHWaitForFileToOpen(
    pidl: LPCITEMIDLIST,
    dwFlags: DWORD,
    dwTimeout: DWORD,
) -> BOOL {
    FALSE
}

/// RLBuildListOfPaths [SHELL32.146] (stub)
#[no_mangle]
pub unsafe extern "system" fn RLBuildListOfPaths() -> DWORD {
    0
}

/// SHValidateUNC [SHELL32.173] (stub)
#[no_mangle]
pub unsafe extern "system" fn SHValidateUNC(hwndOwner: HWND, pszFile: PWSTR, fConnect: UINT) -> BOOL {
    FALSE
}

/// DoEnvironmentSubstA [SHELL32.@]
#[no_mangle]
pub unsafe extern "system" fn DoEnvironmentSubstA(pszString: LPSTR, cchString: UINT) -> DWORD {
    let mut res: BOOL = FALSE;
    let mut len: DWORD = cchString;

    if pszString.is_null() {
        return 0;
    }
    let dst = HeapAlloc(
        GetProcessHeap(),
        0,
        ((cchString as SIZE_T) * core::mem::size_of::<CHAR>() as SIZE_T) as _,
    ) as LPSTR;
    if !dst.is_null() {
        len = ExpandEnvironmentStringsA(pszString, dst, cchString);
        if len != 0 && len < cchString {
            res = TRUE;
            core::ptr::copy_nonoverlapping(dst as *const u8, pszString as *mut u8, len as usize);
        } else {
            len = cchString;
        }
        HeapFree(GetProcessHeap(), 0, dst as LPVOID);
    }
    MAKELONG(len, res as DWORD)
}

/// DoEnvironmentSubstW [SHELL32.@]
#[no_mangle]
pub unsafe extern "system" fn DoEnvironmentSubstW(pszString: LPWSTR, cchString: UINT) -> DWORD {
    let mut res: BOOL = FALSE;
    let mut len: DWORD = cchString;

    if cchString < MAXLONG {
        let dst = HeapAlloc(
            GetProcessHeap(),
            0,
            ((cchString as SIZE_T) * core::mem::size_of::<WCHAR>() as SIZE_T) as _,
        ) as LPWSTR;
        if !dst.is_null() {
            len = ExpandEnvironmentStringsW(pszString, dst, cchString);
            if len != 0 && len <= cchString {
                res = TRUE;
                core::ptr::copy_nonoverlapping(
                    dst as *const u8,
                    pszString as *mut u8,
                    len as usize * core::mem::size_of::<WCHAR>(),
                );
            } else {
                len = cchString;
            }
            HeapFree(GetProcessHeap(), 0, dst as LPVOID);
        }
    }
    MAKELONG(len, res as DWORD)
}

/// DoEnvironmentSubst [SHELL32.53]
#[no_mangle]
pub unsafe extern "system" fn DoEnvironmentSubstAW(x: LPVOID, y: UINT) -> DWORD {
    if SHELL_OsIsUnicode() != 0 {
        DoEnvironmentSubstW(x as LPWSTR, y)
    } else {
        DoEnvironmentSubstA(x as LPSTR, y)
    }
}

/// GUIDFromStringA [SHELL32.703]
#[no_mangle]
pub unsafe extern "system" fn GUIDFromStringA(str: LPCSTR, guid: LPGUID) -> BOOL {
    let mut ansi_str: ANSI_STRING = core::mem::zeroed();
    let mut szWide: [WCHAR; 40] = [0; 40];
    let mut guid_str = UNICODE_STRING {
        Length: 0,
        MaximumLength: core::mem::size_of_val(&szWide) as USHORT,
        Buffer: szWide.as_mut_ptr(),
    };
    if *str != b'{' as CHAR {
        return FALSE;
    }
    RtlInitAnsiString(&mut ansi_str, str);
    (RtlAnsiStringToUnicodeString(&mut guid_str, &ansi_str, 0) == 0
        && RtlGUIDFromString(&guid_str, guid) == 0) as BOOL
}

/// GUIDFromStringW [SHELL32.704]
#[no_mangle]
pub unsafe extern "system" fn GUIDFromStringW(str: LPCWSTR, guid: LPGUID) -> BOOL {
    let mut guid_str: UNICODE_STRING = core::mem::zeroed();
    if str.is_null() || *str != b'{' as WCHAR {
        return FALSE;
    }
    RtlInitUnicodeString(&mut guid_str, str);
    (RtlGUIDFromString(&guid_str, guid) == 0) as BOOL
}

/// PathIsTemporaryA [SHELL32.713] (__REACTOS__)
#[no_mangle]
pub unsafe extern "system" fn PathIsTemporaryA(Str: LPCSTR) -> BOOL {
    let mut szWide: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    SHAnsiToUnicode(Str, szWide.as_mut_ptr(), MAX_PATH as c_int);
    PathIsTemporaryW(szWide.as_ptr())
}

/// PathIsTemporaryW [SHELL32.714] (__REACTOS__)
#[no_mangle]
pub unsafe extern "system" fn PathIsTemporaryW(Str: LPCWSTR) -> BOOL {
    let mut szLongPath: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut szTempPath: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut pszTarget: LPCWSTR = Str;

    let attrs = GetFileAttributesW(Str);
    if attrs != INVALID_FILE_ATTRIBUTES && (attrs & FILE_ATTRIBUTE_TEMPORARY) != 0 {
        return TRUE;
    }

    if GetTempPathW(MAX_PATH as DWORD, szTempPath.as_mut_ptr()) == 0
        || GetLongPathNameW(szTempPath.as_ptr(), szTempPath.as_mut_ptr(), MAX_PATH as DWORD) == 0
    {
        return FALSE;
    }

    if GetLongPathNameW(Str, szLongPath.as_mut_ptr(), MAX_PATH as DWORD) != 0 {
        pszTarget = szLongPath.as_ptr();
    }

    (PathIsEqualOrSubFolder(szTempPath.as_ptr(), pszTarget) != 0
        || PathIsEqualOrSubFolder(CSIDL_INTERNET_CACHE as usize as LPCWSTR, pszTarget) != 0
        || PathIsEqualOrSubFolder(CSIDL_CDBURN_AREA as usize as LPCWSTR, pszTarget) != 0)
        as BOOL
}

/// PsxaCall [internal callback]
unsafe extern "system" fn PsxaCall(hpage: HPROPSHEETPAGE, lParam: LPARAM) -> BOOL {
    let Call = lParam as PPSXA_CALL;
    if !Call.is_null() {
        if ((*Call).bMultiple != 0 || (*Call).bCalled == 0) {
            if let Some(cb) = (*Call).lpfnAddReplaceWith {
                if cb(hpage, (*Call).lParam) != 0 {
                    (*Call).bCalled = TRUE;
                    (*Call).uiCount += 1;
                    return TRUE;
                }
            }
        }
    }
    FALSE
}

/// SHAddFromPropSheetExtArray [SHELL32.167]
#[no_mangle]
pub unsafe extern "system" fn SHAddFromPropSheetExtArray(
    hpsxa: HPSXA,
    lpfnAddPage: LPFNADDPROPSHEETPAGE,
    lParam: LPARAM,
) -> UINT {
    let psxa = hpsxa as PPSXA;
    if !psxa.is_null() {
        let mut Call: PSXA_CALL = core::mem::zeroed();
        Call.lpfnAddReplaceWith = lpfnAddPage;
        Call.lParam = lParam;
        Call.bMultiple = TRUE;

        let base = (*psxa).pspsx.as_ptr();
        let mut i: UINT = 0;
        while i != (*psxa).uiCount {
            let pspsx = *base.add(i as usize);
            ((*(*pspsx).lpVtbl).AddPages)(pspsx, Some(PsxaCall), &mut Call as *mut PSXA_CALL as LPARAM);
            i += 1;
        }
        return Call.uiCount;
    }
    0
}

/// SHCreatePropSheetExtArray [SHELL32.168]
#[no_mangle]
pub unsafe extern "system" fn SHCreatePropSheetExtArray(
    hKey: HKEY,
    pszSubKey: LPCWSTR,
    max_iface: UINT,
) -> HPSXA {
    SHCreatePropSheetExtArrayEx(hKey, pszSubKey, max_iface, core::ptr::null_mut())
}

/// SHCreatePropSheetExtArrayEx [SHELL32.194]
#[no_mangle]
pub unsafe extern "system" fn SHCreatePropSheetExtArrayEx(
    hKey: HKEY,
    pszSubKey: LPCWSTR,
    max_iface: UINT,
    pDataObj: LPDATAOBJECT,
) -> HPSXA {
    let mut szHandler: [WCHAR; 64] = [0; 64];
    let mut dwHandlerLen: DWORD;
    let mut szClsidHandler: [WCHAR; 39] = [0; 39];
    let mut dwClsidSize: DWORD;
    let mut clsid: CLSID = core::mem::zeroed();
    let mut lRet: LONG;
    let mut dwIndex: DWORD;
    let mut psxi: *mut IShellExtInit = core::ptr::null_mut();
    let mut pspsx: *mut IShellPropSheetExt = core::ptr::null_mut();
    let mut hkBase: HKEY = core::ptr::null_mut();
    let mut hkPropSheetHandlers: HKEY = core::ptr::null_mut();
    let mut psxa: PPSXA = core::ptr::null_mut();

    if max_iface == 0 {
        return core::ptr::null_mut();
    }

    lRet = RegOpenKeyW(hKey, pszSubKey, &mut hkBase);
    if lRet != 0 {
        return core::ptr::null_mut();
    }

    lRet = RegOpenKeyExW(
        hkBase,
        W_PROPSHEETHANDLERS.as_ptr(),
        0,
        KEY_ENUMERATE_SUB_KEYS,
        &mut hkPropSheetHandlers,
    );
    RegCloseKey(hkBase);
    if lRet == 0 {
        // FIELD_OFFSET(PSXA, pspsx[max_iface])
        let alloc_size = core::mem::size_of::<UINT>() * 2
            + max_iface as usize * core::mem::size_of::<*mut IShellPropSheetExt>();
        psxa = LocalAlloc(LMEM_FIXED, (alloc_size) as SIZE_T) as PPSXA;
        if !psxa.is_null() {
            core::ptr::write_bytes(psxa as *mut u8, 0, alloc_size);
            (*psxa).uiAllocated = max_iface;

            dwIndex = 0;
            loop {
                dwHandlerLen = (szHandler.len()) as DWORD;
                lRet = RegEnumKeyExW(
                    hkPropSheetHandlers,
                    {
                        let i = dwIndex;
                        dwIndex += 1;
                        i
                    },
                    szHandler.as_mut_ptr(),
                    &mut dwHandlerLen,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                if lRet != 0 {
                    if lRet == ERROR_MORE_DATA {
                        continue;
                    }
                    if lRet == ERROR_NO_MORE_ITEMS {
                        lRet = 0;
                    }
                    break;
                }

                // CLSID stored in the key name itself, or in its default value.
                lRet = SHCLSIDFromStringW(szHandler.as_ptr(), &mut clsid);
                if FAILED(lRet) {
                    dwClsidSize = core::mem::size_of_val(&szClsidHandler) as DWORD;
                    if SHGetValueW(
                        hkPropSheetHandlers,
                        szHandler.as_ptr(),
                        core::ptr::null(),
                        core::ptr::null_mut(),
                        szClsidHandler.as_mut_ptr() as LPVOID,
                        &mut dwClsidSize,
                    ) == 0
                    {
                        szClsidHandler[szClsidHandler.len() - 1] = 0;
                        lRet = SHCLSIDFromStringW(szClsidHandler.as_ptr(), &mut clsid);
                    }
                }

                if SUCCEEDED(lRet) {
                    if SUCCEEDED(CoCreateInstance(
                        &clsid,
                        core::ptr::null_mut(),
                        CLSCTX_INPROC_SERVER,
                        &IID_IShellPropSheetExt,
                        &mut pspsx as *mut *mut IShellPropSheetExt as *mut LPVOID,
                    )) {
                        if SUCCEEDED(((*(*pspsx).lpVtbl).QueryInterface)(
                            pspsx,
                            &IID_IShellExtInit,
                            &mut psxi as *mut *mut IShellExtInit as *mut *mut c_void,
                        )) {
                            if SUCCEEDED(((*(*psxi).lpVtbl).Initialize)(
                                psxi,
                                core::ptr::null(),
                                pDataObj,
                                hKey,
                            )) {
                                let base = (*psxa).pspsx.as_mut_ptr();
                                let idx = (*psxa).uiCount;
                                (*psxa).uiCount += 1;
                                *base.add(idx as usize) = pspsx;
                            } else {
                                ((*(*psxi).lpVtbl).Release)(psxi);
                                ((*(*pspsx).lpVtbl).Release)(pspsx);
                            }
                        } else {
                            ((*(*pspsx).lpVtbl).Release)(pspsx);
                        }
                    }
                }

                if (*psxa).uiCount == (*psxa).uiAllocated {
                    break;
                }
            }
        } else {
            lRet = ERROR_NOT_ENOUGH_MEMORY;
        }

        RegCloseKey(hkPropSheetHandlers);
    }

    if lRet != 0 && !psxa.is_null() {
        SHDestroyPropSheetExtArray(psxa as HPSXA);
        psxa = core::ptr::null_mut();
    }

    psxa as HPSXA
}

/// SHReplaceFromPropSheetExtArray [SHELL32.170]
#[no_mangle]
pub unsafe extern "system" fn SHReplaceFromPropSheetExtArray(
    hpsxa: HPSXA,
    uPageID: UINT,
    lpfnReplaceWith: LPFNADDPROPSHEETPAGE,
    lParam: LPARAM,
) -> UINT {
    let psxa = hpsxa as PPSXA;
    if !psxa.is_null() {
        let mut Call: PSXA_CALL = core::mem::zeroed();
        Call.lpfnAddReplaceWith = lpfnReplaceWith;
        Call.lParam = lParam;

        let base = (*psxa).pspsx.as_ptr();
        let mut i: UINT = 0;
        while i != (*psxa).uiCount {
            Call.bCalled = FALSE;
            let pspsx = *base.add(i as usize);
            ((*(*pspsx).lpVtbl).ReplacePage)(
                pspsx,
                uPageID,
                Some(PsxaCall),
                &mut Call as *mut PSXA_CALL as LPARAM,
            );
            i += 1;
        }
        return Call.uiCount;
    }
    0
}

/// SHDestroyPropSheetExtArray [SHELL32.169]
#[no_mangle]
pub unsafe extern "system" fn SHDestroyPropSheetExtArray(hpsxa: HPSXA) {
    let psxa = hpsxa as PPSXA;
    if !psxa.is_null() {
        let base = (*psxa).pspsx.as_ptr();
        let mut i: UINT = 0;
        while i != (*psxa).uiCount {
            let pspsx = *base.add(i as usize);
            ((*(*pspsx).lpVtbl).Release)(pspsx);
            i += 1;
        }
        LocalFree(psxa as HANDLE);
    }
}

/// CIDLData_CreateFromIDArray [SHELL32.83]
#[no_mangle]
pub unsafe extern "system" fn CIDLData_CreateFromIDArray(
    pidlFolder: PCIDLIST_ABSOLUTE,
    cpidlFiles: UINT,
    lppidlFiles: PCUIDLIST_RELATIVE_ARRAY,
    ppdataObject: *mut LPDATAOBJECT,
) -> HRESULT {
    let hwnd: HWND = core::ptr::null_mut();
    IDataObject_Constructor(hwnd, pidlFolder, lppidlFiles, cpidlFiles, FALSE, ppdataObject)
}

/// SHCreateStdEnumFmtEtc [SHELL32.74]
#[no_mangle]
pub unsafe extern "system" fn SHCreateStdEnumFmtEtc(
    cFormats: UINT,
    lpFormats: *const c_void,
    ppenumFormatetc: *mut LPENUMFORMATETC,
) -> HRESULT {
    let mut pef: *mut IUnk = core::ptr::null_mut();
    let mut hRes = IEnumFORMATETC_Constructor(cFormats, (lpFormats).cast(), (&raw mut pef).cast());
    if FAILED(hRes) {
        return hRes;
    }
    ((*(*pef).lpVtbl).AddRef)(pef);
    hRes = ((*(*pef).lpVtbl).QueryInterface)(
        pef,
        &IID_IEnumFORMATETC,
        ppenumFormatetc as *mut *mut c_void,
    );
    ((*(*pef).lpVtbl).Release)(pef);
    hRes
}

/// SHFindFiles [SHELL32.90]
#[no_mangle]
pub unsafe extern "system" fn SHFindFiles(
    pidlFolder: PCIDLIST_ABSOLUTE,
    pidlSaveFile: PCIDLIST_ABSOLUTE,
) -> BOOL {
    if SHRestricted(REST_NOFIND) != 0 {
        return FALSE;
    }
    (ShellExecuteW(
        core::ptr::null_mut(),
        core::ptr::null(),
        W_EXPLORER_EXE.as_ptr(),
        W_FINDFILES_CLSID.as_ptr(),
        core::ptr::null(),
        SW_SHOWNORMAL,
    ) as INT_PTR
        > 32) as BOOL
}

use crate::prelude::{
    GetCurrentProcessId, SHChangeNotify, StrCpyNW, StrCpyNXW, SHCNE_UPDATEIMAGE, SHCNF_IDLIST,
};
// Upstream's is a FIXME stub too (stubs.cpp:413), so the cross-process path matches the C.
use crate::shell32::stubs::SHLookupIconIndexW;

// undocshell.h:179-199, inside `#include <pshpack1.h>`: the two SHCNE_UPDATEIMAGE payloads
// SHUpdateImageW sends and SHHandleUpdateImage decodes. Not in any bindgen island.
#[repr(C, packed)]
struct SHCNF_UPDATEIMAGE_DATA_1 {
    cbSize: u16,
    iIndex: INT,
    iEffective: INT,
    uFlags: UINT,
    iEffective2: INT,
    terminator: u16,
}

#[repr(C, packed)]
struct SHCNF_UPDATEIMAGE_DATA_2 {
    cbOffset: u16,
    iIndex: INT,
    iEffectiveImageIndex: INT,
    uFlags: UINT,
    dwProcessId: DWORD,
    szHashItem: [u16; MAX_PATH as usize],
    terminator: u16,
}

// All three were empty/-1 bodies: the translation took the Wine `#else` FIXME arm instead of
// the `#ifdef __REACTOS__` one (found by `rustconv body-audit`). So an icon change was never
// announced, and a receiver could never resolve one.

/// SHUpdateImageW [SHELL32.192]
#[no_mangle]
pub unsafe extern "system" fn SHUpdateImageW(
    pszHashItem: LPCWSTR,
    iIndex: c_int,
    uFlags: UINT,
    iImageIndex: c_int,
) {
    // If iImageIndex == -1 (undetermined), it falls back to the default value of 1.
    let effective = if iImageIndex == -1 { 1 } else { iImageIndex };

    let item1 = SHCNF_UPDATEIMAGE_DATA_1 {
        cbSize: core::mem::size_of::<SHCNF_UPDATEIMAGE_DATA_1>() as u16,
        iIndex,
        iEffective: effective,
        uFlags,
        iEffective2: effective,
        terminator: 0,
    };

    let mut item2: SHCNF_UPDATEIMAGE_DATA_2 = core::mem::zeroed();
    let hash = (&raw mut item2.szHashItem).cast::<u16>();
    let end = StrCpyNXW(hash, pszHashItem, MAX_PATH as c_int);
    *end = 0;
    item2.cbOffset = (end as usize - (&raw const item2) as usize) as u16;
    item2.iIndex = iIndex;
    item2.iEffectiveImageIndex = effective;
    item2.uFlags = uFlags;
    item2.dwProcessId = GetCurrentProcessId();
    item2.terminator = 0;

    SHChangeNotify(
        SHCNE_UPDATEIMAGE as LONG,
        SHCNF_IDLIST as UINT,
        (&raw const item1).cast(),
        (&raw const item2).cast(),
    );
}

/// SHUpdateImageA [SHELL32.191]
#[no_mangle]
pub unsafe extern "system" fn SHUpdateImageA(
    pszHashItem: LPCSTR,
    iIndex: INT,
    uFlags: UINT,
    iImageIndex: INT,
) {
    let mut hash = [0u16; MAX_PATH as usize];
    SHAnsiToUnicode(pszHashItem, hash.as_mut_ptr(), hash.len() as INT);
    SHUpdateImageW(hash.as_ptr(), iIndex, uFlags, iImageIndex);
}

/// SHHandleUpdateImage
#[no_mangle]
pub unsafe extern "system" fn SHHandleUpdateImage(pidlExtra: PCIDLIST_ABSOLUTE) -> INT {
    if pidlExtra.is_null() {
        return -1;
    }
    // UNALIGNED in the C: the payload rides in a pidl, so read it unaligned.
    let data = core::ptr::read_unaligned(pidlExtra as *const SHCNF_UPDATEIMAGE_DATA_2);
    if data.dwProcessId == GetCurrentProcessId() {
        return data.iEffectiveImageIndex;
    }
    let mut hash = [0u16; MAX_PATH as usize];
    let src = data.szHashItem;
    StrCpyNW(hash.as_mut_ptr(), src.as_ptr(), hash.len() as c_int);
    SHLookupIconIndexW(hash.as_ptr(), data.iIndex, data.uFlags)
}

/// SHObjectProperties
#[no_mangle]
pub unsafe extern "system" fn SHObjectProperties(
    hwnd: HWND,
    dwType: DWORD,
    szObject: LPCWSTR,
    szPage: LPCWSTR,
) -> BOOL {
    let mut pidl: LPITEMIDLIST = core::ptr::null_mut();
    if dwType == SHOP_FILEPATH {
        pidl = ILCreateFromPathW(szObject);
    }
    if !pidl.is_null() {
        let mut sei: SHELLEXECUTEINFOW = core::mem::zeroed();
        sei.cbSize = core::mem::size_of::<SHELLEXECUTEINFOW>() as DWORD;
        sei.fMask = SEE_MASK_INVOKEIDLIST;
        sei.hwnd = hwnd;
        sei.lpVerb = W_PROPERTIES.as_ptr();
        sei.lpFile = core::ptr::null();
        sei.lpParameters = szPage;
        sei.lpDirectory = core::ptr::null();
        sei.nShow = SW_SHOWNORMAL;
        sei.hInstApp = core::ptr::null_mut();
        sei.lpIDList = pidl as PVOID;
        let result = ShellExecuteExW((&raw mut sei).cast());
        ILFree(pidl);
        return result;
    }
    TRUE
}

/// SHGetNewLinkInfoA
#[no_mangle]
pub unsafe extern "system" fn SHGetNewLinkInfoA(
    pszLinkTo: LPCSTR,
    pszDir: LPCSTR,
    pszName: LPSTR,
    pfMustCopy: *mut BOOL,
    uFlags: UINT,
) -> BOOL {
    let mut wszLinkTo: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut wszDir: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut wszName: [WCHAR; MAX_PATH] = [0; MAX_PATH];

    MultiByteToWideChar(CP_ACP, 0, pszLinkTo, -1, wszLinkTo.as_mut_ptr(), MAX_PATH as c_int);
    MultiByteToWideChar(CP_ACP, 0, pszDir, -1, wszDir.as_mut_ptr(), MAX_PATH as c_int);

    let res = SHGetNewLinkInfoW(
        wszLinkTo.as_ptr(),
        wszDir.as_ptr(),
        wszName.as_mut_ptr(),
        pfMustCopy,
        uFlags,
    );

    if res != 0 {
        WideCharToMultiByte(
            CP_ACP,
            0,
            wszName.as_ptr(),
            -1,
            pszName,
            MAX_PATH as c_int,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
    }
    res
}

/// SHGetNewLinkInfoW
#[no_mangle]
pub unsafe extern "system" fn SHGetNewLinkInfoW(
    mut pszLinkTo: LPCWSTR,
    pszDir: LPCWSTR,
    pszName: LPWSTR,
    pfMustCopy: *mut BOOL,
    mut uFlags: UINT,
) -> BOOL {
    let HANDLEDFLAGS: UINT = SHGNLI_PIDL | SHGNLI_NOUNIQUE | SHGNLI_NOLNK | SHGNLI_NOLOCNAME | SHGNLI_USEURLEXT;
    let mut pszDotForExt: LPCWSTR = W_DOT.as_ptr();
    let mut pszExt: LPCWSTR = if (uFlags & SHGNLI_USEURLEXT) != 0 { W_EXT_URL.as_ptr() } else { W_EXT_LNK.as_ptr() };
    let mut szTarget: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut i: c_int = 2;
    let pidl: LPCITEMIDLIST = if (uFlags & SHGNLI_PIDL) != 0 { pszLinkTo as LPCITEMIDLIST } else { core::ptr::null() };
    let mut fi: SHFILEINFOW = core::mem::zeroed();
    let mut UseDisplayName = false;
    let SHGFIPidlFlag: UINT = if pidl.is_null() { 0 } else { SHGFI_PIDL };

    // The C FIXME-traces flags outside HANDLEDFLAGS (SHGNLI_PREFIXNAME) and ignores them.
    let _ = uFlags & !HANDLEDFLAGS;

    *pfMustCopy = FALSE;
    fi.dwAttributes = (SFGAO_FILESYSTEM | SFGAO_LINK) as DWORD;
    if crate::cfs_folder::SHGetFileInfoW(
        pszLinkTo,
        0,
        &mut fi,
        core::mem::size_of::<SHFILEINFOW>() as UINT,
        SHGFI_DISPLAYNAME | SHGFI_ATTRIBUTES | SHGFI_ATTR_SPECIFIED | SHGFIPidlFlag,
    ) != 0
    {
        UseDisplayName = true;
    }

    if !pidl.is_null() {
        if !UseDisplayName {
            return FALSE;
        }

        pszLinkTo = core::ptr::addr_of!(fi.szDisplayName).cast();
        let shgdn: u32 = if (uFlags & SHGNLI_NOLOCNAME) != 0 { SHGDN_FORPARSING as u32 } else { SHGDN_NORMAL as u32 };
        if (fi.dwAttributes & SFGAO_LINK as DWORD) != 0
            && crate::shell32::utils::SHGetNameAndFlagsW(
                pidl,
                shgdn | tagSHGDN_SHGDN_INFOLDER as u32,
                szTarget.as_mut_ptr(),
                szTarget.len() as u32,
                core::ptr::null_mut(),
            ) >= 0
        {
            // Including possible .lnk extension
            strsafe_rust::StringCchCopyW(core::ptr::addr_of_mut!(fi.szDisplayName).cast(), (core::ptr::addr_of!(fi.szDisplayName) as *const [WCHAR]).len(), szTarget.as_ptr());
        }

        if crate::shell32::utils::SHGetNameAndFlagsW(
            pidl,
            SHGDN_FORPARSING as u32,
            szTarget.as_mut_ptr(),
            szTarget.len() as u32,
            core::ptr::null_mut(),
        ) >= 0
        {
            pszLinkTo = szTarget.as_ptr();
        } else {
            fi.dwAttributes &= !(SFGAO_FILESYSTEM as DWORD); // We can't validate this PIDL as a path
        }
    } else if (uFlags & SHGNLI_NOLOCNAME) != 0 {
        UseDisplayName = false;
    }

    /* FIXME: should test if the file is a shortcut or DOS program */
    if (fi.dwAttributes & SFGAO_FILESYSTEM as DWORD) != 0 && GetFileAttributesW(pszLinkTo) == INVALID_FILE_ATTRIBUTES {
        return FALSE;
    }

    if (fi.dwAttributes & SFGAO_LINK as DWORD) != 0 {
        *pfMustCopy = TRUE; // The caller is not supposed to create a .lnk pointing to a .lnk

        // Remove the "inputname.lnk" extension so that the "(%d)" handling can add it back correctly
        uFlags &= !SHGNLI_NOLNK;
        pszExt = PathFindExtensionW(pszLinkTo);
        pszDotForExt = W_EMPTY.as_ptr();
        UseDisplayName = true;
        if pszLinkTo != core::ptr::addr_of!(fi.szDisplayName).cast() {
            strsafe_rust::StringCchCopyW(core::ptr::addr_of_mut!(fi.szDisplayName).cast(), (core::ptr::addr_of!(fi.szDisplayName) as *const [WCHAR]).len(), pszLinkTo);
        }
        if *pszExt != 0 {
            // pszExt points into pszLinkTo. When that is fi.szDisplayName itself (a PIDL whose
            // parsing name could not be read), this NUL lands on the dot and pszExt reads as
            // empty, so the name gets no extension -- the C aliases the same way.
            PathRemoveExtensionW(core::ptr::addr_of_mut!(fi.szDisplayName).cast());
        }
    }

    if (uFlags & SHGNLI_NOLNK) != 0 {
        // The caller does not want us to append an extension
        pszExt = W_EMPTY.as_ptr();
        pszDotForExt = W_EMPTY.as_ptr();
    }

    let basename: LPCWSTR = PathFindFileNameW(if UseDisplayName { core::ptr::addr_of!(fi.szDisplayName).cast() } else { pszLinkTo });
    *pszName = 0;
    if (uFlags & SHGNLI_NOUNIQUE) == 0 {
        // SHGNLI_NOUNIQUE does not prefix the directory
        strsafe_rust::StringCchCopyW(pszName, MAX_PATH as usize, pszDir);
        if PathAddBackslashW(pszName).is_null() {
            return FALSE;
        }
    }
    let dst_basename = pszName.add(wcslen(pszName));
    let cch = pszName.add(MAX_PATH as usize).offset_from(dst_basename) as usize;

    strsafe_rust::StringCchPrintfW!(dst_basename, cch, W_FMT_NEWLINK.as_ptr(), basename, pszDotForExt, pszExt);
    while (uFlags & SHGNLI_NOUNIQUE) == 0 && GetFileAttributesW(pszName) != INVALID_FILE_ATTRIBUTES {
        strsafe_rust::StringCchPrintfW!(dst_basename, cch, W_FMT_NUM_NEWLINK.as_ptr(), basename, i, pszDotForExt, pszExt);
        i += 1;
    }
    TRUE
}

/// SHStartNetConnectionDialog [SHELL32.@] (__REACTOS__)
#[no_mangle]
pub unsafe extern "system" fn SHStartNetConnectionDialog(
    hwnd: HWND,
    pszRemoteName: LPCSTR,
    dwType: DWORD,
) -> HRESULT {
    if SHELL_OsIsUnicode() != 0 {
        SHStartNetConnectionDialogW(hwnd, pszRemoteName as LPCWSTR, dwType)
    } else {
        SHStartNetConnectionDialogA(hwnd, pszRemoteName, dwType)
    }
}

/// SHSetLocalizedName [SHELL32.@] (stub)
#[no_mangle]
pub unsafe extern "system" fn SHSetLocalizedName(
    pszPath: LPCWSTR,
    pszResModule: LPCWSTR,
    idsRes: c_int,
) -> HRESULT {
    S_OK
}

/// LinkWindow_RegisterClass [SHELL32.258] (stub)
#[no_mangle]
pub unsafe extern "system" fn LinkWindow_RegisterClass() -> BOOL {
    // Superclass comctl32's SysLink as "Link Window". The pre-fix stub returned TRUE
    // having registered nothing, so a caller's CreateWindow on the class then failed
    // (upstream d6cc284435e, which moved it to utils.cpp).
    use crate::island as i;
    static W_SYSLINK: [u16; 8] = [
        b'S' as u16, b'y' as u16, b's' as u16, b'L' as u16, b'i' as u16, b'n' as u16,
        b'k' as u16, 0,
    ];
    static W_LINK_WINDOW: [u16; 12] = [
        b'L' as u16, b'i' as u16, b'n' as u16, b'k' as u16, b' ' as u16, b'W' as u16,
        b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, 0,
    ];
    let mut iccx: i::INITCOMMONCONTROLSEX = core::mem::zeroed();
    iccx.dwSize = core::mem::size_of::<i::INITCOMMONCONTROLSEX>() as i::DWORD;
    iccx.dwICC = i::ICC_LINK_CLASS as i::DWORD;
    i::InitCommonControlsEx(&mut iccx);

    let mut wcx: i::WNDCLASSEXW = core::mem::zeroed();
    wcx.cbSize = core::mem::size_of::<i::WNDCLASSEXW>() as i::UINT;
    if i::GetClassInfoExW(core::ptr::null_mut(), W_SYSLINK.as_ptr(), &mut wcx) == 0 {
        return FALSE;
    }
    wcx.lpszClassName = W_LINK_WINDOW.as_ptr();
    (i::RegisterClassExW(&wcx) != 0
        || i::GetLastError() == i::ERROR_CLASS_ALREADY_EXISTS as i::DWORD) as BOOL
}

/// LinkWindow_UnregisterClass [SHELL32.259] (stub)
#[no_mangle]
pub unsafe extern "system" fn LinkWindow_UnregisterClass(dwUnused: DWORD) -> BOOL {
    TRUE
}

/// SHFlushSFCache [SHELL32.526] (no-op in Wine/ReactOS)
#[no_mangle]
pub unsafe extern "system" fn SHFlushSFCache() {}

/// SHGetImageList [SHELL32.727] (__REACTOS__)
#[no_mangle]
pub unsafe extern "system" fn SHGetImageList(
    iImageList: c_int,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let mut hLarge: HIMAGELIST = core::ptr::null_mut();
    let mut hSmall: HIMAGELIST = core::ptr::null_mut();

    if iImageList != SHIL_LARGE && iImageList != SHIL_SMALL && iImageList != SHIL_SYSSMALL {
        return E_FAIL;
    }

    Shell_GetImageLists(&mut hLarge, &mut hSmall);
    // ReactOS: return the raw pointer, no Duplicate.
    let hNew: HIMAGELIST = if iImageList == SHIL_LARGE { hLarge } else { hSmall };
    let p = hNew as *mut IUnk;
    ((*(*p).lpVtbl).QueryInterface)(p, riid, ppv)
}

/// SHTestTokenMembership [SHELL32.245]
#[no_mangle]
pub unsafe extern "system" fn SHTestTokenMembership(TokenHandle: HANDLE, ulRID: ULONG) -> BOOL {
    let mut ntAuth = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 5], // SECURITY_NT_AUTHORITY
    };
    let nSubAuthority0: DWORD;
    let nSubAuthority1: DWORD;
    let nSubAuthorityCount: DWORD;
    let mut SidToCheck: PVOID = core::ptr::null_mut();
    let mut IsMember: BOOL = FALSE;

    if ulRID == SECURITY_SERVICE_RID || ulRID == SECURITY_LOCAL_SYSTEM_RID {
        nSubAuthority0 = ulRID;
        nSubAuthority1 = 0;
        nSubAuthorityCount = 1;
    } else {
        nSubAuthority0 = SECURITY_BUILTIN_DOMAIN_RID;
        nSubAuthority1 = ulRID;
        nSubAuthorityCount = 2;
    }

    if AllocateAndInitializeSid(
        &mut ntAuth,
        nSubAuthorityCount as BYTE,
        nSubAuthority0,
        nSubAuthority1,
        0,
        0,
        0,
        0,
        0,
        0,
        &mut SidToCheck,
    ) == 0
    {
        return FALSE;
    }

    if CheckTokenMembership(TokenHandle, SidToCheck, &mut IsMember) == 0 {
        IsMember = FALSE;
    }

    FreeSid(SidToCheck);
    IsMember
}

/// IsUserAnAdmin [SHELL32.680]
#[no_mangle]
pub unsafe extern "system" fn IsUserAnAdmin() -> BOOL {
    SHTestTokenMembership(core::ptr::null_mut(), DOMAIN_ALIAS_RID_ADMINS)
}

// ── SHLimitInputEdit subclassing ─────────────────────────────────────────────────────────
unsafe fn UxSubclassInfo_Destroy(pInfo: *mut UxSubclassInfo) {
    if pInfo.is_null() {
        return;
    }
    RemovePropW((*pInfo).hwnd, W_UXSUBCLASSINFO.as_ptr());
    CoTaskMemFree((*pInfo).pwszValidChars as LPVOID);
    CoTaskMemFree((*pInfo).pwszInvalidChars as LPVOID);
    set_window_long_ptr(
        (*pInfo).hwnd,
        GWLP_WNDPROC,
        core::mem::transmute::<WNDPROC, LONG_PTR>((*pInfo).fnWndProc),
    );
    HeapFree(GetProcessHeap(), 0, pInfo as LPVOID);
}

unsafe fn DoSanitizeText(
    pszSanitized: LPWSTR,
    pszInvalidChars: LPCWSTR,
    pszValidChars: LPCWSTR,
) -> BOOL {
    let mut pch1 = pszSanitized;
    let mut pch2 = pszSanitized;
    let mut bFound: BOOL = FALSE;

    while *pch1 != 0 {
        if !pszInvalidChars.is_null() {
            if !wcschr(pszInvalidChars, *pch1).is_null() {
                bFound = TRUE;
                pch1 = pch1.add(1);
                continue;
            }
        } else if !pszValidChars.is_null() {
            if wcschr(pszValidChars, *pch1).is_null() {
                bFound = TRUE;
                pch1 = pch1.add(1);
                continue;
            }
        }
        *pch2 = *pch1;
        pch2 = pch2.add(1);
        pch1 = pch1.add(1);
    }
    *pch2 = 0;
    bFound
}

unsafe fn DoSanitizeClipboard(hwnd: HWND, pInfo: *mut UxSubclassInfo) {
    let mut hData: HANDLE;
    let mut pszText: LPWSTR;
    let mut pszSanitized: LPWSTR = core::ptr::null_mut();
    let mut cbData: DWORD;

    if (get_window_long_ptr(hwnd, GWL_STYLE) & ES_READONLY) != 0 {
        return;
    }
    if OpenClipboard(hwnd) == 0 {
        return;
    }

    hData = GetClipboardData(CF_UNICODETEXT);
    pszText = GlobalLock(hData) as LPWSTR;
    if pszText.is_null() {
        CloseClipboard();
        return;
    }
    SHStrDupW(pszText, &mut pszSanitized);
    GlobalUnlock(hData);

    if !pszSanitized.is_null()
        && DoSanitizeText(pszSanitized, (*pInfo).pwszInvalidChars, (*pInfo).pwszValidChars) != 0
    {
        MessageBeep(0xFFFFFFFF);

        cbData = ((lstrlenW(pszSanitized) + 1) as DWORD) * core::mem::size_of::<WCHAR>() as DWORD;
        hData = GlobalAlloc(GMEM_MOVEABLE | GMEM_SHARE, cbData as SIZE_T);
        pszText = GlobalLock(hData) as LPWSTR;
        if !pszText.is_null() {
            core::ptr::copy_nonoverlapping(
                pszSanitized as *const u8,
                pszText as *mut u8,
                cbData as usize,
            );
            GlobalUnlock(hData);
            SetClipboardData(CF_UNICODETEXT, hData);
        }
    }

    CoTaskMemFree(pszSanitized as LPVOID);
    CloseClipboard();
}

unsafe extern "system" fn LimitEditWindowProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let pInfo = GetPropW(hwnd, W_UXSUBCLASSINFO.as_ptr()) as *mut UxSubclassInfo;
    if pInfo.is_null() {
        return DefWindowProcW(hwnd, uMsg, (wParam) as WPARAM, lParam);
    }
    let fnWndProc = (*pInfo).fnWndProc;

    if uMsg == WM_KEYDOWN {
        if GetKeyState(VK_SHIFT) < 0 && wParam == VK_INSERT as WPARAM {
            DoSanitizeClipboard(hwnd, pInfo);
        } else if GetKeyState(VK_CONTROL) < 0 && wParam == b'V' as WPARAM {
            DoSanitizeClipboard(hwnd, pInfo);
        }
        return CallWindowProcW((fnWndProc) as WNDPROC, hwnd, uMsg, (wParam) as WPARAM, lParam);
    } else if uMsg == WM_PASTE {
        DoSanitizeClipboard(hwnd, pInfo);
        return CallWindowProcW((fnWndProc) as WNDPROC, hwnd, uMsg, (wParam) as WPARAM, lParam);
    } else if uMsg == WM_CHAR {
        if GetKeyState(VK_CONTROL) < 0 && wParam == b'V' as WPARAM {
            return 0; // break -> return 0
        }
        if !(*pInfo).pwszInvalidChars.is_null() {
            if !wcschr((*pInfo).pwszInvalidChars, wParam as WCHAR).is_null() {
                MessageBeep(0xFFFFFFFF);
                return 0;
            }
        } else if !(*pInfo).pwszValidChars.is_null() {
            if wcschr((*pInfo).pwszValidChars, wParam as WCHAR).is_null() {
                MessageBeep(0xFFFFFFFF);
                return 0;
            }
        }
        return CallWindowProcW((fnWndProc) as WNDPROC, hwnd, uMsg, (wParam) as WPARAM, lParam);
    } else if uMsg == WM_UNICHAR || uMsg == WM_IME_CHAR {
        if uMsg == WM_UNICHAR && wParam == UNICODE_NOCHAR {
            return TRUE as LRESULT;
        }
        // WM_IME_CHAR (and WM_UNICHAR fall-through):
        let mut wch: WCHAR = wParam as WCHAR;
        if GetKeyState(VK_CONTROL) < 0 && wch == b'V' as WCHAR {
            return 0; // break
        }
        if IsWindowUnicode(hwnd) == 0 && ((wch >> 8) & 0xff) != 0 {
            let data: [CHAR; 2] = [((wch >> 8) & 0xff) as CHAR, (wch & 0xff) as CHAR];
            MultiByteToWideChar(CP_ACP, 0, data.as_ptr(), 2, &mut wch, 1);
        }
        if !(*pInfo).pwszInvalidChars.is_null() {
            if !wcschr((*pInfo).pwszInvalidChars, wch).is_null() {
                MessageBeep(0xFFFFFFFF);
                return 0;
            }
        } else if !(*pInfo).pwszValidChars.is_null() {
            if wcschr((*pInfo).pwszValidChars, wch).is_null() {
                MessageBeep(0xFFFFFFFF);
                return 0;
            }
        }
        return CallWindowProcW((fnWndProc) as WNDPROC, hwnd, uMsg, (wParam) as WPARAM, lParam);
    } else if uMsg == WM_NCDESTROY {
        UxSubclassInfo_Destroy(pInfo);
        return CallWindowProcW((fnWndProc) as WNDPROC, hwnd, uMsg, (wParam) as WPARAM, lParam);
    } else {
        return CallWindowProcW((fnWndProc) as WNDPROC, hwnd, uMsg, (wParam) as WPARAM, lParam);
    }
}

unsafe fn UxSubclassInfo_Create(
    hwnd: HWND,
    valid: LPWSTR,
    invalid: LPWSTR,
) -> *mut UxSubclassInfo {
    let pInfo = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<UxSubclassInfo>()) as _,
    ) as *mut UxSubclassInfo;
    if pInfo.is_null() {
        CoTaskMemFree(valid as LPVOID);
        CoTaskMemFree(invalid as LPVOID);
        return core::ptr::null_mut();
    }

    (*pInfo).fnWndProc = core::mem::transmute::<LONG_PTR, WNDPROC>(set_window_long_ptr(
        hwnd,
        GWLP_WNDPROC,
        LimitEditWindowProc as usize as LONG_PTR,
    ));
    if (*pInfo).fnWndProc.is_none() {
        CoTaskMemFree(valid as LPVOID);
        CoTaskMemFree(invalid as LPVOID);
        HeapFree(GetProcessHeap(), 0, pInfo as LPVOID);
        return core::ptr::null_mut();
    }

    (*pInfo).hwnd = hwnd;
    (*pInfo).pwszValidChars = valid;
    (*pInfo).pwszInvalidChars = invalid;
    let mut p = pInfo;
    if SetPropW(hwnd, W_UXSUBCLASSINFO.as_ptr(), pInfo as HANDLE) == 0 {
        UxSubclassInfo_Destroy(pInfo);
        p = core::ptr::null_mut();
    }
    p
}

/// SHLimitInputEdit [SHELL32.@]
#[no_mangle]
pub unsafe extern "system" fn SHLimitInputEdit(hWnd: HWND, psf: *mut IShellFolder) -> HRESULT {
    let mut pLimits: *mut IItemNameLimits = core::ptr::null_mut();
    let mut hr: HRESULT;

    let mut pInfo = GetPropW(hWnd, W_UXSUBCLASSINFO.as_ptr()) as *mut UxSubclassInfo;
    if !pInfo.is_null() {
        UxSubclassInfo_Destroy(pInfo);
        pInfo = core::ptr::null_mut();
    }

    hr = ((*(*psf).lpVtbl).QueryInterface)(
        psf,
        &IID_IItemNameLimits,
        &mut pLimits as *mut *mut IItemNameLimits as *mut *mut c_void,
    );
    if FAILED(hr) {
        return hr;
    }

    let mut pwszValidChars: LPWSTR = core::ptr::null_mut();
    let mut pwszInvalidChars: LPWSTR = core::ptr::null_mut();
    hr = ((*(*pLimits).lpVtbl).GetValidCharacters)(pLimits, &mut pwszValidChars, &mut pwszInvalidChars);
    if FAILED(hr) {
        ((*(*pLimits).lpVtbl).Release)(pLimits);
        return hr;
    }

    pInfo = UxSubclassInfo_Create(hWnd, pwszValidChars, pwszInvalidChars);
    if pInfo.is_null() {
        hr = E_FAIL;
    }

    ((*(*pLimits).lpVtbl).Release)(pLimits);
    hr
}

/// SHLimitInputCombo [SHELL32.748] (__REACTOS__)
#[no_mangle]
pub unsafe extern "system" fn SHLimitInputCombo(hWnd: HWND, psf: *mut IShellFolder) -> HRESULT {
    let hwndEdit = GetTopWindow(hWnd);
    if hwndEdit.is_null() {
        return E_FAIL;
    }
    SHLimitInputEdit(hwndEdit, psf)
}

// ADDED-TO-TYPES?: SHELLSTATE/SHELLFLAGSTATE/REGSHELLSTATE/CABINETSTATE (bitfield structs),
//   OPENFILENAMEW, WIN32_FIND_DATAW, MRUINFOW, SHELLEXECUTEINFOW, SID_IDENTIFIER_AUTHORITY,
//   UNICODE_STRING/ANSI_STRING, POINT/RECT, and the COM vtables IShellLinkW/IPersistFile/
//   IShellPropSheetExt/IShellExtInit/IItemNameLimits — all common enough to hoist into
//   crate::shellcom / crate::types later.
