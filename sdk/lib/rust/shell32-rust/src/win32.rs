//! Shared Win32/shell substrate for the translated free-function translation units.
//!
//! Every item here is a DETERMINISTIC fact — an SDK header value, an SDK record layout,
//! or a definition promoted from a module that already proved it. None of it came from a
//! model. Constants carry the header they were read from.
//!
//! Promoted definitions were cross-checked against every other copy in the crate before
//! being moved here: 40 symbols against up to 17 module-local copies each, and 47 extern
//! declarations, with zero value conflicts. Comparison is semantic, not textual — `DWORD`
//! vs `UINT` and `273` vs `0x111` are one fact, while a differing value or width is a hard
//! failure rather than a pick-one (the `CMD = 19`/`CMD = 16` contradiction class).
//!
//! The older modules keep their own local copies; a local definition shadows a glob import,
//! so this module is additive and does not disturb them. Retiring those duplicates is a
//! separate sweep.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use crate::shellcom::*;
use crate::types::*;
use core::ffi::c_void;

// ---------------------------------------------------------------------------
// In-crate implementations. These are DEFINED by this crate, so they must be
// re-exported, never re-declared `extern` — a second declaration would be an
// unresolved duplicate rather than a call to the real body.
// ---------------------------------------------------------------------------

pub use crate::wine::pidl::{_ILGetDataPointer, _ILIsUnicode};
pub use crate::wine::shellord::{CABINETSTATE, ReadCabinetState, WriteCabinetState};

// ---------------------------------------------------------------------------
// SDK record layouts, transcribed from the headers named on each item.
// ---------------------------------------------------------------------------

/// winuser.h `tagCOPYDATASTRUCT`.
#[repr(C)]
pub struct COPYDATASTRUCT {
    pub dwData: ULONG_PTR,
    pub cbData: DWORD,
    pub lpData: *mut c_void,
}

/// commctrl.h `tagNMCUSTOMDRAWINFO`.
#[repr(C)]
pub struct NMCUSTOMDRAW {
    pub hdr: NMHDR,
    pub dwDrawStage: DWORD,
    pub hdc: HDC,
    pub rc: RECT,
    pub dwItemSpec: DWORD_PTR,
    pub uItemState: UINT,
    pub lItemlParam: LPARAM,
}

/// commctrl.h `tagNMTVCUSTOMDRAW`.
#[repr(C)]
pub struct NMTVCUSTOMDRAW {
    pub nmcd: NMCUSTOMDRAW,
    pub clrText: COLORREF,
    pub clrTextBk: COLORREF,
    pub iLevel: INT,
}

/// commctrl.h `tagTVHITTESTINFO`.
#[repr(C)]
pub struct TV_HITTESTINFO {
    pub pt: POINT,
    pub flags: UINT,
    pub hItem: HTREEITEM,
}

/// commctrl.h `tagNMTVKEYDOWN`. `wVKey`/`flags` are packed after the header.
#[repr(C)]
pub struct TV_KEYDOWN {
    pub hdr: NMHDR,
    pub wVKey: WORD,
    pub flags: UINT,
}

pub type HTREEITEM = HANDLE;

/// shellapi.h `_NOTIFYICONDATAA` at NTDDI_VERSION >= NTDDI_WIN2K, through hBalloonIcon.
/// The `uTimeout`/`uVersion` union is one UINT; the W form already lives in
/// user_notification.rs and is promoted below.
#[repr(C)]
pub struct NOTIFYICONDATAA {
    pub cbSize: DWORD,
    pub hWnd: HWND,
    pub uID: UINT,
    pub uFlags: UINT,
    pub uCallbackMessage: UINT,
    pub hIcon: HICON,
    pub szTip: [CHAR; 128],
    pub dwState: DWORD,
    pub dwStateMask: DWORD,
    pub szInfo: [CHAR; 256],
    pub uTimeout: UINT,
    pub szInfoTitle: [CHAR; 64],
    pub dwInfoFlags: DWORD,
    pub guidItem: GUID,
    pub hBalloonIcon: HICON,
}

// shellapi.h derives these with FIELD_OFFSET, so they are offsets, not sizes. `szTip[64]`
// in the V1 macro means "past 64 elements of szTip", not "at szTip".
pub const NOTIFYICONDATAA_V1_SIZE: DWORD =
    (core::mem::offset_of!(NOTIFYICONDATAA, szTip) + 64 * core::mem::size_of::<CHAR>()) as DWORD;
pub const NOTIFYICONDATAA_V2_SIZE: DWORD =
    core::mem::offset_of!(NOTIFYICONDATAA, guidItem) as DWORD;
pub const NOTIFYICONDATAA_V3_SIZE: DWORD =
    core::mem::offset_of!(NOTIFYICONDATAA, hBalloonIcon) as DWORD;
pub const NOTIFYICONDATAW_V1_SIZE: DWORD =
    (core::mem::offset_of!(NOTIFYICONDATAW, szTip) + 64 * core::mem::size_of::<WCHAR>()) as DWORD;
pub const NOTIFYICONDATAW_V2_SIZE: DWORD =
    core::mem::offset_of!(NOTIFYICONDATAW, guidItem) as DWORD;

/// shlobj_undoc.h `tagWALK_TREE_CMD`.
pub const WALK_TREE_SAVE: DWORD = 0;
pub const WALK_TREE_DESTROY: DWORD = 1;
pub const WALK_TREE_DEFAULT: DWORD = 2;
pub const WALK_TREE_REFRESH: DWORD = 3;

/// shlobj_undoc.h: `( (HKEY)INVALID_HANDLE_VALUE )`.
pub const HKEY_REGTREEOPTION_GRAYED: HKEY = usize::MAX as HKEY;

/// winnt.h / cmtypes.h.
pub const KEY_ENUMERATE_SUB_KEYS: DWORD = 0x0008;
/// wtypes.idl `tagCLSCTX`.
pub const CLSCTX_INPROC_SERVER: DWORD = 0x00000001;

// ---------------------------------------------------------------------------
// C macros. A C debug/accessor macro must stay a macro: lowering one to an
// out-of-line no-op fn is an optimization barrier that uncloaks latent ABI
// mismatches (field manual, the `ud2`/BUGCHECK class).
// ---------------------------------------------------------------------------

#[macro_export]
macro_rules! UNREFERENCED_PARAMETER {
    ($($arg:tt)*) => {{}};
}
#[macro_export]
macro_rules! TRACE_ON {
    ($($arg:tt)*) => {
        false
    };
}
#[macro_export]
macro_rules! _countof {
    ($a:expr) => {
        ($a).len()
    };
}
#[macro_export]
macro_rules! FIELD_OFFSET {
    ($t:ty, $f:ident) => {
        core::mem::offset_of!($t, $f) as isize
    };
}

pub use crate::{FIELD_OFFSET, TRACE_ON, UNREFERENCED_PARAMETER, _countof};

#[inline]
pub fn LOWORD(l: DWORD) -> WORD {
    (l & 0xFFFF) as WORD
}
#[inline]
pub fn HIWORD(l: DWORD) -> WORD {
    ((l >> 16) & 0xFFFF) as WORD
}
/// windowsx.h: the coordinates are SIGNED, so a raw mask would break negative
/// (multi-monitor) positions.
#[inline]
pub fn GET_X_LPARAM(lp: LPARAM) -> INT {
    (lp & 0xFFFF) as i16 as INT
}
#[inline]
pub fn GET_Y_LPARAM(lp: LPARAM) -> INT {
    ((lp >> 16) & 0xFFFF) as i16 as INT
}
#[inline]
pub fn MAKEINTRESOURCEW(i: UINT) -> LPWSTR {
    i as usize as LPWSTR
}

/// commctrl.h `TreeView_GetSelection` / `TreeView_HitTest`.
#[inline]
pub unsafe fn TreeView_GetSelection(hwnd: HWND) -> HTREEITEM {
    SendMessageW(hwnd, TVM_GETNEXTITEM, TVGN_CARET as WPARAM, 0) as HTREEITEM
}
#[inline]
pub unsafe fn TreeView_HitTest(hwnd: HWND, lpht: *mut TV_HITTESTINFO) -> HTREEITEM {
    SendMessageW(hwnd, TVM_HITTEST, 0, lpht as LPARAM) as HTREEITEM
}
pub const TVM_GETNEXTITEM: UINT = 0x1100 + 10; // commctrl.h TV_FIRST + 10
pub const TVM_HITTEST: UINT = 0x1100 + 17; // commctrl.h TV_FIRST + 17
pub const TVGN_CARET: UINT = 0x0009; // commctrl.h

/// prsht.h `PropSheet_Changed`.
#[inline]
pub unsafe fn PropSheet_Changed(hDlg: HWND, hwnd: HWND) -> LRESULT {
    SendMessageW(hDlg, PSM_CHANGED, hwnd as WPARAM, 0)
}
pub const PSM_CHANGED: UINT = 0x0400 + 104; // prsht.h WM_USER + 104

/// A byte fill/copy must NOT be hand-written in Rust: LLVM folds a byte loop back into
/// a call to the function itself. These forward to the CRT the DLL already imports.
#[inline]
pub unsafe fn ZeroMemory<T>(dst: *mut T, len: usize) {
    core::ptr::write_bytes(dst as *mut u8, 0u8, len)
}
#[inline]
pub unsafe fn CopyMemory<T, U>(dst: *mut T, src: *const U, len: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, len)
}

// ---------------------------------------------------------------------------
// Imports the DLL already links. Declared, never defined.
// ---------------------------------------------------------------------------

extern "system" {
    pub fn GetClassNameW(hWnd: HWND, lpClassName: LPWSTR, nMaxCount: INT) -> INT;
    pub fn GetMessagePos() -> DWORD;
    pub fn ScreenToClient(hWnd: HWND, lpPoint: *mut POINT) -> BOOL;
    pub fn CreatePropertySheetPageW(konst: *const PROPSHEETPAGEW) -> HPROPSHEETPAGE;
    pub fn PropertySheetW(konst: *const PROPSHEETHEADERW) -> INT_PTR;
}

extern "C" {
    pub fn strlen(s: *const i8) -> usize;
    pub fn isprint(c: INT) -> INT;
}

pub type HPROPSHEETPAGE = HANDLE;
pub type PFILETIME = *mut FILETIME;
pub type LPUNKNOWN = *mut IUnknown;
pub type WNDENUMPROC = unsafe extern "system" fn(HWND, LPARAM) -> BOOL;
/// prsht.h: the page callback. `PSPCB_*` reason in `uMsg`.
pub type LPFNPSPCALLBACKW =
    unsafe extern "system" fn(HWND, UINT, *mut PROPSHEETPAGEW) -> UINT;

/// prsht.h `_PROPSHEETHEADERW`. Each anonymous union is one pointer-width slot.
#[repr(C)]
pub struct PROPSHEETHEADERW {
    pub dwSize: DWORD,
    pub dwFlags: DWORD,
    pub hwndParent: HWND,
    pub hInstance: HINSTANCE,
    pub hIcon: HICON,
    pub pszCaption: LPCWSTR,
    pub nPages: UINT,
    pub nStartPage: UINT,
    pub ppsp: *const PROPSHEETPAGEW,
    pub pfnCallback: *mut c_void,
    pub hbmWatermark: HANDLE,
    pub hplWatermark: HANDLE,
    pub hbmHeader: HANDLE,
}
pub type LPFNPSPCALLBACK = *mut c_void;

// ---------------------------------------------------------------------------
// Promoted definitions (agreement-verified against every other in-crate copy).
// ---------------------------------------------------------------------------

// CStringW — promoted from 8 module-local definition(s).
pub struct CStringW { buffer: LPWSTR, len: usize, capacity: usize }

// DLGPROC — promoted from 5 module-local definition(s).
pub type DLGPROC = unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR;

// DWLP_MSGRESULT — promoted from 0 module-local definition(s) (verified-equivalent).
pub const DWLP_MSGRESULT: INT = 0;

// DWLP_USER — promoted from 0 module-local definition(s) (verified-equivalent).
pub const DWLP_USER: INT =
    (core::mem::size_of::<LRESULT>() + core::mem::size_of::<DLGPROC>()) as INT;

// ERROR_NOT_SUPPORTED — promoted from 1 module-local definition(s).
pub const ERROR_NOT_SUPPORTED: DWORD = 50;

// GWLP_USERDATA — promoted from 2 module-local definition(s).
pub const GWLP_USERDATA: INT = -21;

// HKEY_LOCAL_MACHINE — promoted from 1 module-local definition(s).
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY;

// IID_IClassFactory — promoted from 1 module-local definition(s).
extern "C" {
    pub static IID_IClassFactory: IID;
}

// IID_IContextMenu — promoted from 17 module-local definition(s).
extern "C" {
    pub static IID_IContextMenu: GUID;
}

// IID_IDataObject — promoted from 17 module-local definition(s).
extern "C" {
    pub static IID_IDataObject: IID;
}

// IID_IExtractIconA — promoted from 9 module-local definition(s).
extern "C" {
    pub static IID_IExtractIconA: GUID;
}

// IID_IExtractIconW — promoted from 10 module-local definition(s).
extern "C" {
    pub static IID_IExtractIconW: GUID;
}

// IID_IPersist — promoted from 11 module-local definition(s).
extern "C" {
    pub static IID_IPersist: IID;
}

// IID_IPersistFolder — promoted from 10 module-local definition(s).
extern "C" {
    pub static IID_IPersistFolder: IID;
}

// IID_IPersistFolder2 — promoted from 13 module-local definition(s).
extern "C" {
    pub static IID_IPersistFolder2: IID;
}

// IID_IPersistFolder3 — promoted from 1 module-local definition(s).
extern "C" {
    pub static IID_IPersistFolder3: IID;
}

// IID_IShellExtInit — promoted from 11 module-local definition(s).
extern "C" {
    pub static IID_IShellExtInit: IID;
}

// IID_IShellFolder — promoted from 18 module-local definition(s).
extern "C" {
    pub static IID_IShellFolder: IID;
}

// IID_IShellFolder2 — promoted from 12 module-local definition(s).
extern "C" {
    pub static IID_IShellFolder2: IID;
}

// IID_IShellLinkA — promoted from 1 module-local definition(s).
extern "C" {
    pub static IID_IShellLinkA: IID;
}

// IID_IShellLinkW — promoted from 1 module-local definition(s).
extern "C" {
    pub static IID_IShellLinkW: IID;
}

// IID_IShellView — promoted from 9 module-local definition(s).
extern "C" {
    pub static IID_IShellView: IID;
}

// IID_IUnknown — promoted from 32 module-local definition(s).
extern "C" {
    pub static IID_IUnknown: IID;
}




// KEY_READ — promoted from 6 module-local definition(s).
pub const KEY_READ: DWORD = 0x20019;

// MB_ICONERROR — promoted from 2 module-local definition(s).
pub const MB_ICONERROR: DWORD = 16;

// NIF_ICON — promoted from 1 module-local definition(s).
pub const NIF_ICON: DWORD = 2;

// NIF_INFO — promoted from 1 module-local definition(s).
pub const NIF_INFO: DWORD = 16;

// NIF_MESSAGE — promoted from 1 module-local definition(s).
pub const NIF_MESSAGE: DWORD = 1;

// NIF_TIP — promoted from 1 module-local definition(s).
pub const NIF_TIP: DWORD = 4;

// NOTIFYICONDATAW — promoted from 1 module-local definition(s).
#[repr(C)]
pub struct NOTIFYICONDATAW {
    pub cbSize: DWORD, pub hWnd: HWND, pub uID: UINT, pub uFlags: UINT,
    pub uCallbackMessage: UINT, pub hIcon: HICON, pub szTip: [WCHAR; 128],
    pub dwState: DWORD, pub dwStateMask: DWORD, pub szInfo: [WCHAR; 256],
    pub uTimeout: UINT, pub szInfoTitle: [WCHAR; 64], pub dwInfoFlags: DWORD,
    pub guidItem: GUID, pub hBalloonIcon: HICON,
}

// NOTIFYICONDATAW_V3_SIZE — promoted from 1 module-local definition(s).
pub const NOTIFYICONDATAW_V3_SIZE: DWORD =
    core::mem::offset_of!(NOTIFYICONDATAW, hBalloonIcon) as DWORD;

// PROPSHEETPAGE — promoted from 1 module-local definition(s).
pub type PROPSHEETPAGE = PROPSHEETPAGEW;

// PROPSHEETPAGEW — promoted from 4 module-local definition(s) (prefix-merged).
#[repr(C)]
pub struct PROPSHEETPAGEW {
    pub dwSize: DWORD,
    pub dwFlags: DWORD,
    pub hInstance: HINSTANCE,
    pub pszTemplate: LPCWSTR,      // union { pszTemplate, pResource }
    pub hIcon: HICON,              // union { hIcon, pszIcon }
    pub pszTitle: LPCWSTR,
    pub pfnDlgProc: Option<DLGPROC>,
    pub lParam: LPARAM,
    pub pfnCallback: Option<LPFNPSPCALLBACKW>,
    pub pcRefParent: *mut UINT,
    pub pszHeaderTitle: LPCWSTR,
    pub pszHeaderSubTitle: LPCWSTR,
    pub hActCtx: HANDLE,
}

// SHGFI_ICON — promoted from 5 module-local definition(s).
pub const SHGFI_ICON: DWORD = 256;

// SHGFI_PIDL — promoted from 3 module-local definition(s).
pub const SHGFI_PIDL: DWORD = 8;

// WM_COMMAND — promoted from 7 module-local definition(s).
pub const WM_COMMAND: DWORD = 273;

// shell32_hInstance — promoted from 18 module-local definition(s).
extern "C" {
    pub static shell32_hInstance: HINSTANCE;
}

// ---------------------------------------------------------------------------
// Promoted extern declarations (ABI-normalized: parameter names are not ABI).
// ---------------------------------------------------------------------------

// CFolderOptions_ApplyDefFolderSettings_RustBridge — promoted from 1 module-local definition(s) (in-crate impl).
pub use crate::folder_options::CFolderOptions_ApplyDefFolderSettings_RustBridge;

// CFolderOptions_CanSetDefFolderSettings_RustBridge — promoted from 1 module-local definition(s) (in-crate impl).
pub use crate::folder_options::CFolderOptions_CanSetDefFolderSettings_RustBridge;

// CFolderOptions_ResetGlobalAndDefViewFolderSettings_RustBridge — promoted from 1 module-local definition(s) (in-crate impl).
pub use crate::folder_options::CFolderOptions_ResetGlobalAndDefViewFolderSettings_RustBridge;

// CLSIDFromString — promoted from 0 module-local definition(s) (verified-equivalent).
extern "system" {
    pub fn CLSIDFromString(lpsz: LPCOLESTR, pclsid: *mut CLSID) -> HRESULT;
}

// CoCreateInstance — promoted from 0 module-local definition(s) (verified-equivalent).
extern "system" {
    pub fn CoCreateInstance(
        rclsid: REFCLSID,
        pUnkOuter: LPUNKNOWN,
        dwClsContext: DWORD,
        riid: REFIID,
        ppv: *mut LPVOID,
    ) -> HRESULT;
}

// CoInitialize — promoted from 2 module-local definition(s).
extern "system" {
    pub fn CoInitialize(pvReserved: LPVOID) -> HRESULT;
}

// EnableWindow — promoted from 2 module-local definition(s).
extern "system" {
    pub fn EnableWindow(window: HWND, enable: BOOL) -> BOOL;
}

// EnumWindows — promoted from 1 module-local definition(s).
extern "system" {
    pub fn EnumWindows(lpEnumFunc: WNDENUMPROC, lParam: LPARAM) -> BOOL;
}

// FindWindowW — promoted from 2 module-local definition(s).
extern "system" {
    pub fn FindWindowW(lpClassName: LPCWSTR, lpWindowName: LPCWSTR) -> HWND;
}

// GetDlgItem — promoted from 4 module-local definition(s).
extern "system" {
    pub fn GetDlgItem(dlg: HWND, id: INT) -> HWND;
}

// GetParent — promoted from 4 module-local definition(s).
extern "system" {
    pub fn GetParent(hwnd: HWND) -> HWND;
}

// GetProcessHeap — promoted from 40 module-local definition(s).
extern "system" {
    pub fn GetProcessHeap() -> HANDLE;
}

// GetSysColor — promoted from 2 module-local definition(s).
extern "system" {
    pub fn GetSysColor(index: INT) -> COLORREF;
}


// HeapAlloc — promoted from 20 module-local definition(s).
extern "system" {
    pub fn HeapAlloc(heap: HANDLE, flags: DWORD, bytes: usize) -> *mut core::ffi::c_void;
}

// ILFree — promoted from 13 module-local definition(s) (in-crate impl).
pub use crate::wine::pidl::ILFree;

// IsEqualIID — a windef.h comparison macro, not an import.
#[inline]
pub unsafe fn IsEqualIID(a: *const GUID, b: *const GUID) -> bool {
    // GUID does not derive PartialEq; compare the record bytes, as IsEqualGUID does.
    core::slice::from_raw_parts(a as *const u8, core::mem::size_of::<GUID>())
        == core::slice::from_raw_parts(b as *const u8, core::mem::size_of::<GUID>())
}

// LoadStringW — promoted from 25 module-local definition(s).
extern "system" {
    pub fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT;
}

// MessageBoxW — promoted from 3 module-local definition(s).
extern "system" {
    pub fn MessageBoxW(owner: HWND, text: LPCWSTR, caption: LPCWSTR, kind: UINT) -> INT;
}

// MultiByteToWideChar — promoted from 8 module-local definition(s).
extern "system" {
    pub fn MultiByteToWideChar(CodePage: UINT, dwFlags: DWORD, lpMultiByteStr: LPCSTR, cbMultiByte: INT, lpWideCharStr: LPWSTR, cchWideChar: INT) -> INT;
}

// PostMessage — promoted from 1 module-local definition(s).
// `PostMessage`/`SetWindowLongPtr` are winuser.h MACROS selecting the A/W export —
// there is no symbol of either name to import. Bind the real W entry points.
extern "system" {
    pub fn PostMessageW(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> BOOL;
}
#[inline]
pub unsafe fn PostMessage(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> BOOL {
    PostMessageW(hwnd, msg, wparam, lparam)
}

// The `...Ptr...` forms exist only on 64-bit: user32.spec marks both
// `-arch=x86_64,arm64`, and on i386 winuser.h defines SetWindowLongPtrW to
// SetWindowLongW. Import whichever the target actually exports — asking for
// `_SetWindowLongPtrW@12` on i386 is a link error, not a portability wrapper.
#[cfg(not(target_pointer_width = "32"))]
extern "system" {
    pub fn SetWindowLongPtrW(hwnd: HWND, index: INT, value: LONG_PTR) -> LONG_PTR;
    pub fn GetWindowLongPtrW(hwnd: HWND, index: INT) -> LONG_PTR;
}
#[cfg(target_pointer_width = "32")]
extern "system" {
    pub fn SetWindowLongW(hwnd: HWND, index: INT, value: LONG) -> LONG;
    pub fn GetWindowLongW(hwnd: HWND, index: INT) -> LONG;
}

#[cfg(not(target_pointer_width = "32"))]
#[inline]
pub unsafe fn SetWindowLongPtr(hwnd: HWND, index: INT, value: LONG_PTR) -> LONG_PTR {
    SetWindowLongPtrW(hwnd, index, value)
}
#[cfg(not(target_pointer_width = "32"))]
#[inline]
pub unsafe fn GetWindowLongPtr(hwnd: HWND, index: INT) -> LONG_PTR {
    GetWindowLongPtrW(hwnd, index)
}
#[cfg(target_pointer_width = "32")]
#[inline]
pub unsafe fn SetWindowLongPtr(hwnd: HWND, index: INT, value: LONG_PTR) -> LONG_PTR {
    SetWindowLongW(hwnd, index, value as LONG) as LONG_PTR
}
#[cfg(target_pointer_width = "32")]
#[inline]
pub unsafe fn GetWindowLongPtr(hwnd: HWND, index: INT) -> LONG_PTR {
    GetWindowLongW(hwnd, index) as LONG_PTR
}


// RegCloseKey — promoted from 13 module-local definition(s).
extern "system" {
    pub fn RegCloseKey(key: HKEY) -> LONG;
}

// RegEnumKeyExW — promoted from 0 module-local definition(s) (verified-equivalent).
extern "system" {
    pub fn RegEnumKeyExW(
        hKey: HKEY,
        dwIndex: DWORD,
        lpName: LPWSTR,
        lpcchName: LPDWORD,
        lpReserved: LPDWORD,
        lpClass: LPWSTR,
        lpcchClass: LPDWORD,
        lpftLastWriteTime: PFILETIME,
    ) -> LONG;
}

// RegEnumKeyW — promoted from 4 module-local definition(s).
extern "system" {
    pub fn RegEnumKeyW(key: HKEY, index: DWORD, name: LPWSTR, capacity: DWORD) -> LONG;
}

// RegGetValueW — promoted from 11 module-local definition(s).
extern "system" {
    pub fn RegGetValueW(key: HKEY, subkey: LPCWSTR, value: LPCWSTR, flags: DWORD, kind: LPDWORD, data: LPVOID, bytes: LPDWORD) -> LONG;
}

// RegOpenKeyExW — promoted from 11 module-local definition(s).
extern "system" {
    pub fn RegOpenKeyExW(key: HKEY, subkey: LPCWSTR, options: DWORD, access: REGSAM, result: PHKEY) -> LONG;
}

// RegQueryInfoKeyW — promoted from 1 module-local definition(s).
extern "system" {
    pub fn RegQueryInfoKeyW(hKey: HKEY, lpClass: LPWSTR, lpcchClass: LPDWORD, lpReserved: LPDWORD, lpcSubKeys: LPDWORD, lpcbMaxSubKeyLen: LPDWORD, lpcbMaxClassLen: LPDWORD, lpcValues: LPDWORD, lpcbMaxValueNameLen: LPDWORD, lpcbMaxValueLen: LPDWORD, lpcbSecurityDescriptor: LPDWORD, lpftLastWriteTime: *mut FILETIME) -> LONG;
}

// SHCLSIDFromStringW — promoted from 2 module-local definition(s) (in-crate impl).
pub use crate::wine::shellole::SHCLSIDFromStringW;

// SHCoCreateInstance — promoted from 2 module-local definition(s) (in-crate impl).
pub use crate::wine::shellole::SHCoCreateInstance;

// SHELL_DataObject_ILCloneFullItem — promoted from 3 module-local definition(s).
extern "system" {
    #[link_name = "SHELL_DataObject_ILCloneFullItem_RustBridge"]
    pub fn SHELL_DataObject_ILCloneFullItem(pDO: *mut IDataObject, Index: UINT) -> PIDLIST_ABSOLUTE;
}

// SHELL_ErrorBox — promoted from 1 module-local definition(s).
extern "system" {
    #[link_name = "SHELL_ErrorBox_RustBridge"]
    pub fn SHELL_ErrorBox(hwndOwner: HWND, Error: UINT) -> UINT;
}

// SHExtCoCreateInstance — promoted from 1 module-local definition(s) (in-crate impl).
pub use crate::wine::shellole::SHExtCoCreateInstance;

// SHFree — promoted from 10 module-local definition(s) (in-crate impl).
pub use crate::wine::shellole::SHFree;

// SHGetFileInfoW — promoted from 8 module-local definition(s) (in-crate impl).
pub use crate::wine::shell32_main::SHGetFileInfoW;

// SHGetNameFromIDList — promoted from 4 module-local definition(s) (in-crate impl).
pub use crate::wine::pidl::SHGetNameFromIDList;

// SHGetSetSettings — promoted from 2 module-local definition(s) (in-crate impl).
pub use crate::wine::shellord::SHGetSetSettings;

// SHSendMessageBroadcastW — promoted from 1 module-local definition(s).
extern "system" {
    pub fn SHSendMessageBroadcastW(msg: UINT, w: WPARAM, l: LPARAM) -> DWORD;
}

// SHSettingsChanged — promoted from 1 module-local definition(s) (in-crate impl).
pub use crate::wine::shpolicy::SHSettingsChanged;

// SHUnicodeToAnsi — promoted from 0 module-local definition(s) (verified-equivalent).
extern "system" {
    pub fn SHUnicodeToAnsi(pwszSrc: LPCWSTR, pszDst: LPSTR, cchBuf: INT) -> INT;
}

// SendMessageW — promoted from 6 module-local definition(s).
extern "system" {
    pub fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
}


// WideCharToMultiByte — promoted from 8 module-local definition(s).
extern "system" {
    pub fn WideCharToMultiByte(page: UINT, flags: DWORD, wide: LPCWSTR, wide_len: INT, multi: LPSTR, multi_len: INT, default_char: LPCSTR, used_default: *mut BOOL) -> INT;
}



// lstrcpynA — promoted from 4 module-local definition(s).
extern "system" {
    pub fn lstrcpynA(lpString1: LPSTR, lpString2: LPCSTR, iMaxLength: INT) -> LPSTR;
}

// wsprintfW — promoted from 3 module-local definition(s).
extern "C" {
    pub fn wsprintfW(buffer: LPWSTR, format: LPCWSTR, ...) -> INT;
}

// ---------------------------------------------------------------------------
// SDK header constants. Provenance on every line.
// ---------------------------------------------------------------------------
pub const ERROR_ALREADY_EXISTS: DWORD = 0xb7;  // winerror.h
pub const ERROR_MORE_DATA: DWORD = 0xea;  // winerror.h
pub const ERROR_NO_MORE_ITEMS: DWORD = 0x103;  // winerror.h
pub const ICON_BIG: DWORD = 0x1;  // winuser.h
pub const SHGFI_ADDOVERLAYS: DWORD = 0x20;  // shellapi.h
pub const WM_COPYDATA: DWORD = 0x4a;  // winuser.h
pub const WM_SETICON: DWORD = 0x80;  // winuser.h
pub const WM_SETTINGCHANGE: DWORD = 0x1a;  // winuser.h
pub const NIF_STATE: DWORD = 0x8;  // shellapi.h
pub const NIF_GUID: DWORD = 0x20;  // shellapi.h
pub const VK_F5: DWORD = 0x74;  // winuser.h
pub const VK_SPACE: DWORD = 0x20;  // winuser.h
pub const COLOR_WINDOW: DWORD = 0x5;  // winuser.h
pub const COLOR_GRAYTEXT: DWORD = 0x11;  // winuser.h
pub const ERROR_CAN_NOT_COMPLETE: DWORD = 0x3eb;  // winerror.h
pub const CDDS_PREPAINT: DWORD = 0x1;  // commctrl.h
pub const CDDS_ITEMPREPAINT: DWORD = 0x10001;  // commctrl.h
pub const CDRF_DODEFAULT: DWORD = 0x0;  // commctrl.h
pub const CDRF_NOTIFYITEMDRAW: DWORD = 0x20;  // commctrl.h
pub const CDRF_NEWFONT: DWORD = 0x2;  // commctrl.h
pub const CDIS_SELECTED: DWORD = 0x1;  // commctrl.h
pub const PSH_PROPTITLE: DWORD = 0x1;  // prsht.h
pub const PSH_USEPSTARTPAGE: DWORD = 0x40;  // prsht.h
pub const PSP_DEFAULT: DWORD = 0x0;  // prsht.h
pub const PSP_USETITLE: DWORD = 0x8;  // prsht.h
pub const PSP_USECALLBACK: DWORD = 0x80;  // prsht.h
pub const TABDMC_NOTIFY: DWORD = 0x1;  // undocshell.h
pub const NI_NOTIFY_SIG: DWORD = 0x34753423;  // undocshell.h
pub const IDC_VIEW_TREEVIEW: DWORD = 0x36b3;  // shresdef.h
pub const IDC_VIEW_APPLY_TO_ALL: DWORD = 0x36b1;  // shresdef.h
pub const IDC_VIEW_RESET_ALL: DWORD = 0x36b2;  // shresdef.h
pub const IDS_CANTSHOWPROPERTIES: DWORD = 0x9f;  // shresdef.h
pub const FCIDM_DESKBROWSER_REFRESH: DWORD = 0xa220;  // shresdef.h
pub const MAX_PROPERTY_SHEET_PAGE: DWORD = 0x20;  // precomp.h


// ---------------------------------------------------------------------------
// Interface IIDs and class CLSIDs. Published by the ole32/shell32 import libs;
// never spelled out as byte literals here.
// ---------------------------------------------------------------------------

extern "C" {
    pub static IID_IAutoComplete: IID;
    pub static IID_IAutoComplete2: IID;
    pub static IID_IDropSource: IID;
    pub static IID_IOleCommandTarget: IID;
    pub static IID_IViewObject: IID;
    pub static IID_IShellIconOverlayIdentifier: IID;
    pub static IID_IRegTreeOptions: IID;
    pub static CLSID_CRegTreeOptions: CLSID;
}

// ---------------------------------------------------------------------------
// undocshell.h records.
// ---------------------------------------------------------------------------

/// undocshell.h `_TRAYNOTIFYDATAW`. Always carries the latest NOTIFYICONDATAW.
#[repr(C)]
pub struct TRAYNOTIFYDATAW {
    pub dwSignature: DWORD,
    pub dwMessage: DWORD,
    pub nid: NOTIFYICONDATAW,
}

// systray.cpp spells these as function-local #defines:
//   NIDV1_TIP_SIZE_x = (NOTIFYICONDATA<x>_V1_SIZE - FIELD_OFFSET(NOTIFYICONDATA<x>, szTip))
//                      / sizeof(<char type>)
// which is just the 64-element V1 szTip bound. Derived, not assumed.
pub const NIDV1_TIP_SIZE_A: INT = 64;
pub const NIDV1_TIP_SIZE_W: INT = 64;

// ---------------------------------------------------------------------------
// shlobj_undoc.h `IRegTreeOptions`, and shlobj.h `IShellIconOverlayIdentifier`.
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct IRegTreeOptions {
    pub lpVtbl: *const IRegTreeOptionsVtbl,
}
#[repr(C)]
pub struct IRegTreeOptionsVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        *mut IRegTreeOptions,
        REFIID,
        *mut *mut c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IRegTreeOptions) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IRegTreeOptions) -> ULONG,
    pub InitTree: unsafe extern "system" fn(
        *mut IRegTreeOptions,
        HWND,
        HKEY,
        *const i8,
        *const i8,
    ) -> HRESULT,
    pub WalkTree: unsafe extern "system" fn(*mut IRegTreeOptions, DWORD) -> HRESULT,
    pub ToggleItem: unsafe extern "system" fn(*mut IRegTreeOptions, HTREEITEM) -> HRESULT,
    pub ShowHelp: unsafe extern "system" fn(*mut IRegTreeOptions, HTREEITEM, DWORD) -> HRESULT,
}
impl IRegTreeOptions {
    #[inline]
    pub unsafe fn InitTree(
        &mut self,
        hwnd: HWND,
        hkey: HKEY,
        pszRoot: *const i8,
        pszSub: *const i8,
    ) -> HRESULT {
        ((*self.lpVtbl).InitTree)(self, hwnd, hkey, pszRoot, pszSub)
    }
    #[inline]
    pub unsafe fn WalkTree(&mut self, cmd: DWORD) -> HRESULT {
        ((*self.lpVtbl).WalkTree)(self, cmd)
    }
    #[inline]
    pub unsafe fn ToggleItem(&mut self, item: HTREEITEM) -> HRESULT {
        ((*self.lpVtbl).ToggleItem)(self, item)
    }
    #[inline]
    pub unsafe fn ShowHelp(&mut self, item: HTREEITEM, flags: DWORD) -> HRESULT {
        ((*self.lpVtbl).ShowHelp)(self, item, flags)
    }
    #[inline]
    pub unsafe fn Release(&mut self) -> ULONG {
        ((*self.lpVtbl).Release)(self)
    }
}

#[repr(C)]
pub struct IShellIconOverlayIdentifier {
    pub lpVtbl: *const IShellIconOverlayIdentifierVtbl,
}
#[repr(C)]
pub struct IShellIconOverlayIdentifierVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        *mut IShellIconOverlayIdentifier,
        REFIID,
        *mut *mut c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IShellIconOverlayIdentifier) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IShellIconOverlayIdentifier) -> ULONG,
    pub IsMemberOf:
        unsafe extern "system" fn(*mut IShellIconOverlayIdentifier, LPCWSTR, DWORD) -> HRESULT,
    pub GetOverlayInfo: unsafe extern "system" fn(
        *mut IShellIconOverlayIdentifier,
        LPWSTR,
        INT,
        *mut INT,
        *mut DWORD,
    ) -> HRESULT,
    pub GetPriority:
        unsafe extern "system" fn(*mut IShellIconOverlayIdentifier, *mut INT) -> HRESULT,
}
impl IShellIconOverlayIdentifier {
    #[inline]
    pub unsafe fn IsMemberOf(&mut self, path: LPCWSTR, attrs: DWORD) -> HRESULT {
        ((*self.lpVtbl).IsMemberOf)(self, path, attrs)
    }
    #[inline]
    pub unsafe fn GetPriority(&mut self, prio: *mut INT) -> HRESULT {
        ((*self.lpVtbl).GetPriority)(self, prio)
    }
    #[inline]
    pub unsafe fn GetOverlayInfo(
        &mut self,
        file: LPWSTR,
        cch: INT,
        index: *mut INT,
        flags: *mut DWORD,
    ) -> HRESULT {
        ((*self.lpVtbl).GetOverlayInfo)(self, file, cch, index, flags)
    }
}

// ---------------------------------------------------------------------------
// CRT and wine debug helpers the DLL already imports.
// ---------------------------------------------------------------------------

pub type c_char = i8;

extern "C" {
    /// Variadic; declared with the C ABI so callers pass through unchanged.
    pub fn sprintf(buf: *mut c_char, fmt: *const c_char, ...) -> INT;
    pub fn wine_dbg_sprintf(fmt: *const c_char, ...) -> *const c_char;
    pub fn HCR_GetClassNameA(riid: REFIID, name: LPSTR, len: DWORD) -> BOOL;
}

/// The SDK tag name for the same record.
pub type _GUID = GUID;
pub type _MSG = MSG;

// ---------------------------------------------------------------------------
// Window messages and notification codes used as `match` PATTERNS.
//
// These must be in scope: a bare uppercase name in a match arm that resolves to
// nothing is not an error in Rust — it is an irrefutable BINDING that swallows
// every value and makes the arms after it unreachable. verify_match_consts
// caught exactly that here (§9.112, the class that once shipped a BUGCHECK 0x1E).
// ---------------------------------------------------------------------------

pub const WM_DESTROY: UINT = 0x0002; // winuser.h
pub const WM_INITDIALOG: UINT = 0x0110; // winuser.h
pub const WM_NOTIFY: UINT = 0x004E; // winuser.h

// commctrl.h/prsht.h derive these by UNSIGNED wrap from a *_FIRST base
// (`#define NM_FIRST (0U-0U)`, `#define NM_CLICK (NM_FIRST-2)`). Spelling the
// subtraction keeps the derivation checkable instead of baking a hex literal.
pub const NM_FIRST: UINT = 0u32.wrapping_sub(0);
pub const NM_CLICK: UINT = NM_FIRST.wrapping_sub(2);
pub const NM_CUSTOMDRAW: UINT = NM_FIRST.wrapping_sub(12);
pub const TVN_FIRST: UINT = 0u32.wrapping_sub(400);
pub const TVN_KEYDOWN: UINT = TVN_FIRST.wrapping_sub(12);
pub const PSN_FIRST: UINT = 0u32.wrapping_sub(200);
pub const PSN_APPLY: UINT = PSN_FIRST.wrapping_sub(2);

pub const IDC_VIEW_RESTORE_DEFAULTS: DWORD = 0x36b4; // shresdef.h
