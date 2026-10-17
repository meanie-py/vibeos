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
// The crate's own functions: the definition is the declaration.
pub use crate::wine::classes::HCR_GetProgIdKeyOfExtension;
pub use crate::wine::classes::HLM_GetIconW;
pub use crate::wine::pidl::ILClone;
pub use crate::wine::pidl::ILCombine;
pub use crate::wine::pidl::ILGetSize;
pub use crate::open_with_menu_tu::SH32_InvokeOpenWith;
pub use crate::wine::shellole::SHAlloc;
pub use crate::shell32::utils::SHBindToObject;
pub use crate::wine::pidl::SHBindToParent;
pub use crate::wine::pidl::SHCloneSpecialIDList;
pub use crate::shell32::utils::SHELL_GetUIObjectOfAbsoluteItem;
pub use crate::desktop_folder_tu::SHGetDesktopFolder;
pub use crate::wine::shellpath::SHGetFolderLocation;
pub use crate::wine::pidl::SHGetPathFromIDListW;
pub use crate::open_with_menu_tu::SHOpenWithDialog;
pub use crate::shell32::shlexec::ShellExecuteExW;
pub use crate::wine::shellstring::StrRetToStrNW;
pub use crate::default_context_menu_tu::_InsertMenuItemW;
// One declaration per external symbol: island.rs.
pub use crate::island::BeginPaint;
pub use crate::island::BitBlt;
pub use crate::island::CLSIDFromString;
pub use crate::island::CloseHandle;
pub use crate::island::CoCreateInstance;
pub use crate::island::CreateCompatibleDC;
pub use crate::island::CreateIconIndirect;
pub use crate::island::CreateMenu;
pub use crate::island::CreatePopupMenu;
pub use crate::island::CreateProcessW;
pub use crate::island::CreatePropertySheetPageW;
pub use crate::island::DeleteDC;
pub use crate::island::DestroyIcon;
pub use crate::island::DestroyMenu;
pub use crate::island::DialogBoxParamW;
pub use crate::island::EnableWindow;
pub use crate::island::EndDialog;
pub use crate::island::EndPaint;
pub use crate::island::EnumWindows;
pub use crate::island::FindFirstFileW;
pub use crate::island::FindNextFileW;
pub use crate::island::FindWindowW;
pub use crate::island::GetClassNameW;
pub use crate::island::GetClientRect;
pub use crate::island::GetDC;
pub use crate::island::GetDlgItem;
pub use crate::island::GetDlgItemTextW;
pub use crate::island::GetIconInfo;
pub use crate::island::GetMenuDefaultItem;
pub use crate::island::GetMenuItemCount;
pub use crate::island::GetMenuItemInfoW;
pub use crate::island::GetModuleFileNameW;
pub use crate::island::GetParent;
pub use crate::island::GetShellWindow;
pub use crate::island::GetSystemMetrics;
pub use crate::island::GetWindowLongW;
pub use crate::island::GetWindowRect;
pub use crate::island::HeapAlloc;
pub use crate::island::HeapReAlloc;
pub use crate::island::ImageList_Create;
pub use crate::island::ImageList_Destroy;
pub use crate::island::ImageList_GetIcon;
pub use crate::island::LoadIconW;
pub use crate::island::LoadImageW;
pub use crate::island::LoadStringW;
pub use crate::island::LocalAlloc;
pub use crate::island::MapWindowPoints;
pub use crate::island::MessageBoxW;
pub use crate::island::PathFindOnPathExW;
pub use crate::island::PathFindOnPathW;
pub use crate::island::PostMessageW;
pub use crate::island::PrivateExtractIconsW;
pub use crate::island::PropertySheetW;
pub use crate::island::RegCloseKey;
pub use crate::island::RegEnumKeyExW;
pub use crate::island::RegEnumKeyW;
pub use crate::island::RegGetValueW;
pub use crate::island::RegOpenKeyExW;
pub use crate::island::RegOpenKeyW;
pub use crate::island::RegQueryInfoKeyW;
pub use crate::island::RegQueryValueW;
pub use crate::island::ReleaseDC;
pub use crate::island::SHAutoComplete;
pub use crate::island::SHGetValueW;
pub use crate::island::SHSendMessageBroadcastW;
pub use crate::island::ScreenToClient;
pub use crate::island::SelectObject;
pub use crate::island::SendDlgItemMessageW;
pub use crate::island::SendMessageW;
pub use crate::island::SetDlgItemTextW;
pub use crate::island::SetTextColor;
pub use crate::island::SetWindowLongW;
pub use crate::island::SetWindowTextW;
pub use crate::island::ShowWindow;
pub use crate::island::StrRetToBufW;
pub use crate::island::TrackPopupMenuEx;
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
pub use crate::island::NMCUSTOMDRAW;

/// commctrl.h `tagNMTVCUSTOMDRAW`.
pub use crate::island::NMTVCUSTOMDRAW;

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
        (($a).len()) as u32
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
    pub fn GetMessagePos() -> DWORD;
}

extern "C" {
    pub fn strlen(s: *const i8) -> usize;
    pub fn isprint(c: INT) -> INT;
}

// HPROPSHEETPAGE / SHFILEINFOW: types.rs is the authority; do not redeclare here.
pub type PFILETIME = *mut FILETIME;
pub type LPUNKNOWN = *mut IUnknown;
pub type WNDENUMPROC = unsafe extern "system" fn(HWND, LPARAM) -> BOOL;
/// prsht.h: the page callback. `PSPCB_*` reason in `uMsg`.
pub type LPFNPSPCALLBACKW =
    unsafe extern "system" fn(HWND, UINT, *mut PROPSHEETPAGEW) -> UINT;

/// prsht.h `_PROPSHEETHEADERW`. Each anonymous union is one pointer-width slot; the
/// members are named here as shell32 uses them (`phpage` over `ppsp`, `pStartPage`
/// over `nStartPage`), with the alternates reachable as accessors below.
#[repr(C)]
pub struct PROPSHEETHEADERW {
    pub dwSize: DWORD,
    pub dwFlags: DWORD,
    pub hwndParent: HWND,
    pub hInstance: HINSTANCE,
    pub hIcon: HICON,           // union { hIcon, pszIcon }
    pub pszCaption: LPCWSTR,
    pub nPages: UINT,
    pub pStartPage: LPCWSTR,    // union { nStartPage, pStartPage }
    pub phpage: *mut HPROPSHEETPAGE, // union { ppsp, phpage }
    pub pfnCallback: *mut c_void,
    pub hbmWatermark: HANDLE,
    pub hplWatermark: HANDLE,
    pub hbmHeader: HANDLE,
}
impl PROPSHEETHEADERW {
    /// The `nStartPage` arm of the same union as `pStartPage`.
    #[inline]
    pub unsafe fn nStartPage(&self) -> UINT {
        self.pStartPage as usize as UINT
    }
    #[inline]
    pub unsafe fn set_nStartPage(&mut self, v: UINT) {
        self.pStartPage = v as usize as LPCWSTR;
    }
    /// The `ppsp` arm of the same union as `phpage`.
    #[inline]
    pub unsafe fn ppsp(&self) -> *const PROPSHEETPAGEW {
        self.phpage as *const PROPSHEETPAGEW
    }
}

/// precomp.h defines this `extern inline`, so there is no symbol to import — it is
/// translated here, and its address is taken as the AddPages callback.
pub unsafe extern "system" fn AddPropSheetPageCallback(
    hPage: HPROPSHEETPAGE,
    lParam: LPARAM,
) -> BOOL {
    let pHeader = lParam as *mut PROPSHEETHEADERW;
    if (*pHeader).nPages < MAX_PROPERTY_SHEET_PAGE {
        *(*pHeader).phpage.add((*pHeader).nPages as usize) = hPage;
        (*pHeader).nPages += 1;
        return TRUE;
    }
    FALSE
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

// SH32_InvokeOpenWith — CDECL, not stdcall.
//
// COpenWithMenu.cpp declares it `HRESULT SH32_InvokeOpenWith(...)` with no WINAPI, so it is
// cdecl, and shlexec.cpp called it as cdecl too. This declaration sat in the `extern "system"`
// block below and so said stdcall: on i386 the two disagree about who pops the arguments, and
// shlexec.rs (flipped) has been calling it across that mismatch. It was invisible while the
// definition was still C++ — `scan_calling_conventions` compares a declaration against an
// IN-CRATE definition, and there was none to compare with. Flipping COpenWithMenu.cpp to Rust
// put the definition in the crate and the scanner reported it immediately (CC_MISMATCH).
//
// The other caller, open_with_menu.rs, is fine and stays as it is: it declares the name with
// `#[link_name = "Shell32RustInvokeOpenWith"]`, and that wrapper really is WINAPI/stdcall.
extern "C" {
}

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
pub use crate::island::PROPSHEETPAGEW;

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
}

// CoCreateInstance — promoted from 0 module-local definition(s) (verified-equivalent).
extern "system" {
}

// CoInitialize — promoted from 2 module-local definition(s).
extern "system" {
    pub fn CoInitialize(pvReserved: LPVOID) -> HRESULT;
}

// EnableWindow — promoted from 2 module-local definition(s).
extern "system" {
}

// EnumWindows — promoted from 1 module-local definition(s).
extern "system" {
}

// FindWindowW — promoted from 2 module-local definition(s).
extern "system" {
}

// GetDlgItem — promoted from 4 module-local definition(s).
extern "system" {
}

// GetParent — promoted from 4 module-local definition(s).
extern "system" {
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
}

// MessageBoxW — promoted from 3 module-local definition(s).
extern "system" {
}

// MultiByteToWideChar — promoted from 8 module-local definition(s).
extern "system" {
    pub fn MultiByteToWideChar(CodePage: UINT, dwFlags: DWORD, lpMultiByteStr: LPCSTR, cbMultiByte: INT, lpWideCharStr: LPWSTR, cchWideChar: INT) -> INT;
}

// PostMessage — promoted from 1 module-local definition(s).
// `PostMessage`/`SetWindowLongPtr` are winuser.h MACROS selecting the A/W export —
// there is no symbol of either name to import. Bind the real W entry points.
extern "system" {
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
}

// RegEnumKeyExW — promoted from 0 module-local definition(s) (verified-equivalent).
extern "system" {
}

// RegEnumKeyW — promoted from 4 module-local definition(s).
extern "system" {
}

// RegGetValueW — promoted from 11 module-local definition(s).
extern "system" {
}

// RegOpenKeyExW — promoted from 11 module-local definition(s).
extern "system" {
}

// RegQueryInfoKeyW — promoted from 1 module-local definition(s).
extern "system" {
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
}

// SHSettingsChanged — promoted from 1 module-local definition(s) (in-crate impl).
pub use crate::wine::shpolicy::SHSettingsChanged;

// SHUnicodeToAnsi — promoted from 0 module-local definition(s) (verified-equivalent).
extern "system" {
    pub fn SHUnicodeToAnsi(pwszSrc: LPCWSTR, pszDst: LPSTR, cchBuf: INT) -> INT;
}

// SendMessageW — promoted from 6 module-local definition(s).
extern "system" {
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

// CStubWindow32 is Rust now (crate::atl_stub_windows); re-exported under its old path.
pub use crate::atl_stub_windows::{CStubWindow32_CreateStub_RustBridge, CStubWindow32_DestroyStub_RustBridge};

/// precomp.h `CStubWindow32::TYPE_PROPERTYSHEET` (the enum starts at TYPE_FORMATDRIVE = 1).
pub const CSTUBWINDOW32_TYPE_FORMATDRIVE: UINT = 1;
pub const CSTUBWINDOW32_TYPE_PROPERTYSHEET: UINT = 2;

// ---------------------------------------------------------------------------
// Shell extension interfaces, promoted with their vtables (agreement-checked
// across 10 modules; parameter names, line wrapping and path spellings are not
// type identity, so only a real signature difference would fail).
// ---------------------------------------------------------------------------

// IShellBrowser — promoted from 9 module-local definition(s).
#[repr(C)]
pub struct IShellBrowser {
    pub lpVtbl: *const IShellBrowserVtbl,
}

// IShellBrowserVtbl — promoted from 9 module-local definition(s).
#[repr(C)]
pub struct IShellBrowserVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellBrowser, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellBrowser) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellBrowser) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(This: *mut IShellBrowser, phwnd: *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp: unsafe extern "system" fn(This: *mut IShellBrowser, fEnterMode: BOOL) -> HRESULT,
    pub InsertMenusSB: unsafe extern "system" fn(This: *mut IShellBrowser, hmenuShared: HMENU, lpMenuWidths: LPOLEMENUGROUPWIDTHS) -> HRESULT,
    pub SetMenuSB: unsafe extern "system" fn(This: *mut IShellBrowser, hmenuShared: HMENU, holemenuReserved: HOLEMENU, hwndActiveObject: HWND) -> HRESULT,
    pub RemoveMenusSB: unsafe extern "system" fn(This: *mut IShellBrowser, hmenuShared: HMENU) -> HRESULT,
    pub SetStatusTextSB: unsafe extern "system" fn(This: *mut IShellBrowser, pszStatusText: LPCOLESTR) -> HRESULT,
    pub EnableModelessSB: unsafe extern "system" fn(This: *mut IShellBrowser, fEnable: BOOL) -> HRESULT,
    pub TranslateAcceleratorSB: unsafe extern "system" fn(This: *mut IShellBrowser, pmsg: *mut MSG, wID: WORD) -> HRESULT,
    pub BrowseObject: unsafe extern "system" fn(This: *mut IShellBrowser, pidl: PCUIDLIST_RELATIVE, wFlags: UINT) -> HRESULT,
    pub GetViewStateStream: unsafe extern "system" fn(This: *mut IShellBrowser, grfMode: DWORD, ppStrm: *mut *mut IStream) -> HRESULT,
    pub GetControlWindow: unsafe extern "system" fn(This: *mut IShellBrowser, id: UINT, phwnd: *mut HWND) -> HRESULT,
    pub SendControlMsg: unsafe extern "system" fn(This: *mut IShellBrowser, id: UINT, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, pret: *mut LRESULT) -> HRESULT,
    pub QueryActiveShellView: unsafe extern "system" fn(This: *mut IShellBrowser, ppshv: *mut *mut IShellView) -> HRESULT,
    pub OnViewWindowActive: unsafe extern "system" fn(This: *mut IShellBrowser, pshv: *mut IShellView) -> HRESULT,
    pub SetToolbarItems: unsafe extern "system" fn(This: *mut IShellBrowser, lpButtons: LPTBBUTTONSB, nButtons: UINT, uFlags: UINT) -> HRESULT,
}

// IShellExtInit — promoted from 11 module-local definition(s).
#[repr(C)]
pub struct IShellExtInit {
    pub lpVtbl: *const IShellExtInitVtbl,
}

// IShellExtInitVtbl — promoted from 11 module-local definition(s).
#[repr(C)]
pub struct IShellExtInitVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellExtInit, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellExtInit) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellExtInit) -> ULONG,
    pub Initialize: unsafe extern "system" fn(This: *mut IShellExtInit, pidlFolder: PCIDLIST_ABSOLUTE, pdtobj: *mut IDataObject, hkeyProgID: HKEY) -> HRESULT,
}

// IShellPropSheetExt — promoted from 5 module-local definition(s).
#[repr(C)]
pub struct IShellPropSheetExt {
    pub lpVtbl: *const IShellPropSheetExtVtbl,
}

// IShellPropSheetExtVtbl — promoted from 5 module-local definition(s).
#[repr(C)]
pub struct IShellPropSheetExtVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellPropSheetExt, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellPropSheetExt) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellPropSheetExt) -> ULONG,
    pub AddPages: unsafe extern "system" fn(This: *mut IShellPropSheetExt, pfnAddPage: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT,
    pub ReplacePage: unsafe extern "system" fn(This: *mut IShellPropSheetExt, uPageID: EXPPS, pfnReplaceWith: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT,
}


/// shellapi.h: with UNICODE, `SHFILEINFO` is `SHFILEINFOW`.
pub type SHFILEINFO = crate::types::SHFILEINFOW;

// utils.cpp / CDefView.cpp helpers with no exported symbol of their own.
extern "system" {
    #[link_name = "DataObject_GetOffset_RustBridge"]
    pub fn DataObject_GetOffset(pDO: *mut IDataObject, pt: *mut POINT) -> HRESULT;
}

// Referenced only by IShellBrowser's vtable slots, which this crate does not call.
// Opaque records: the vtable layout is what matters, not these interiors.
#[repr(C)]
pub struct IShellView {
    _opaque: [u8; 0],
}
pub type LPTBBUTTONSB = *mut c_void;
pub type LPFNSVADDPROPSHEETPAGE = *mut c_void;
pub type EXPPS = UINT;

impl IUnknown {
    #[inline]
    pub unsafe fn QueryInterface(&mut self, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT {
        ((*self.lpVtbl).QueryInterface)(self, riid, ppv)
    }
    #[inline]
    pub unsafe fn AddRef(&mut self) -> ULONG {
        ((*self.lpVtbl).AddRef)(self)
    }
    #[inline]
    pub unsafe fn Release(&mut self) -> ULONG {
        ((*self.lpVtbl).Release)(self)
    }
}

impl IShellExtInit {
    #[inline]
    pub unsafe fn Initialize(
        &mut self,
        pidlFolder: PCIDLIST_ABSOLUTE,
        pdtobj: *mut IDataObject,
        hkeyProgID: HKEY,
    ) -> HRESULT {
        ((*self.lpVtbl).Initialize)(self, pidlFolder, pdtobj, hkeyProgID)
    }
}

impl IShellPropSheetExt {
    #[inline]
    pub unsafe fn AddPages(
        &mut self,
        pfnAddPage: LPFNSVADDPROPSHEETPAGE,
        lParam: LPARAM,
    ) -> HRESULT {
        ((*self.lpVtbl).AddPages)(self, pfnAddPage, lParam)
    }
}

extern "C" {
    pub static IID_IShellPropSheetExt: IID;
}

// ---------------------------------------------------------------------------
// OLE data-transfer substrate (objidl.h / ole2.h), and the HIDA clipboard
// format the shell uses to move PIDL arrays through an IDataObject.
// ---------------------------------------------------------------------------

// STGMEDIUM/FORMATETC come from com-sys via types.rs; these are the remaining
// spellings that substrate needs and shell32-rust had only as module-local copies.
pub use crate::shell_item_array::CIDA;
pub use com_sys::{CLIPFORMAT, DVTARGETDEVICE};

/// shtypes.h. shellcom.rs carries only the non-U flavor.
pub type PCUIDLIST_ABSOLUTE = LPCITEMIDLIST;

/// objidl.h `TYMED`.
pub const TYMED_NULL: DWORD = 0;
pub const TYMED_HGLOBAL: DWORD = 1;
/// wtypes.h `DVASPECT`.
pub const DVASPECT_CONTENT: DWORD = 1;
/// winerror.h.
pub const E_UNEXPECTED: HRESULT = 0x8000FFFFu32 as HRESULT;

extern "system" {
    pub fn RegisterClipboardFormatW(lpszFormat: LPCWSTR) -> UINT;
    pub fn GlobalLock(hMem: HGLOBAL) -> *mut c_void;
    pub fn GlobalUnlock(hMem: HGLOBAL) -> BOOL;
    pub fn ReleaseStgMedium(pmedium: *mut STGMEDIUM);
}

/// shlobj.h `CFSTR_SHELLIDLISTW` — "Shell IDList Array", registered lazily.
///
/// C++ caches this in `g_cfHIDA`, a `DECLSPEC_SELECTANY` global in shellutils.h,
/// so every TU in a module shares one registration. This is that cache. A racing
/// double-register is harmless: RegisterClipboardFormatW is idempotent per name
/// and returns the same atom, so the worst case is one redundant call.
static mut G_CF_HIDA: CLIPFORMAT = 0;

pub unsafe fn hida_clipformat() -> CLIPFORMAT {
    if G_CF_HIDA == 0 {
        // L"Shell IDList Array"
        const NAME: [WCHAR; 19] = [
            0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0020, 0x0049, 0x0044, 0x004C, 0x0069, 0x0073,
            0x0074, 0x0020, 0x0041, 0x0072, 0x0072, 0x0061, 0x0079, 0x0000,
        ];
        G_CF_HIDA = RegisterClipboardFormatW(NAME.as_ptr()) as CLIPFORMAT;
    }
    G_CF_HIDA
}

/// shlobj.h `HIDA_GetPIDLFolder` / `HIDA_GetPIDLItem` — static inlines with no
/// exported symbol, so they are translated rather than imported.
#[inline]
pub unsafe fn HIDA_GetPIDLFolder(pida: *const CIDA) -> PCUIDLIST_ABSOLUTE {
    (pida as *const u8).add((*pida).aoffset[0] as usize) as PCUIDLIST_ABSOLUTE
}

#[inline]
pub unsafe fn HIDA_GetPIDLItem(pida: *const CIDA, i: usize) -> PCUIDLIST_RELATIVE {
    // aoffset is a C flexible array member modelled as [UINT; 1], so index it through
    // a raw pointer: `[i + 1]` bounds-check-panics for every i >= 0.
    let aoffset = (*pida).aoffset.as_ptr();
    (pida as *const u8).add(*aoffset.add(i + 1) as usize) as PCUIDLIST_RELATIVE
}

// ---------------------------------------------------------------------------
// Substrate for the Wave-5 free-function TUs (shell32/*.rs).
//
// Same rule as the rest of this module: an item DEFINED by this crate is re-exported,
// never re-declared `extern` — a second declaration is an unresolved duplicate rather
// than a call to the real body.
// ---------------------------------------------------------------------------

pub use crate::shell_link::__SHCloneStrAtoW;
pub use crate::wine::pidl::{
    _ILCopyCidaToaPidl, _ILCreateDesktop, _ILGetDepth, _ILGetFileStructW, _ILIsDesktop,
    _ILIsDrive, _ILIsFolder, _ILIsMyComputer, _ILIsPidlSimple, _ILIsValue, _ILSimpleGetTextW,
};

/// pidl.h:282 `static inline BOOL _ILIsEmpty(pidl) { return _ILIsDesktop(pidl); }`.
#[inline]
pub unsafe fn _ILIsEmpty(pidl: LPCITEMIDLIST) -> BOOL {
    _ILIsDesktop(pidl)
}

// wine/unicode.h aliases the wide-string helpers straight onto the CRT; the names are
// macros there, so they arrive in translated code as calls with no Rust definition.
// Each one is the exact alias the header spells, quoted on the line above it.
extern "C" {
    pub fn wcsncpy(dst: *mut WCHAR, src: *const WCHAR, n: usize) -> *mut WCHAR;
    pub fn wcsrchr(s: *const WCHAR, c: WCHAR) -> *mut WCHAR;
    pub fn wcscat(dst: *mut WCHAR, src: *const WCHAR) -> *mut WCHAR;
    pub fn wcsstr(haystack: *const WCHAR, needle: *const WCHAR) -> *mut WCHAR;
    pub fn _wcsnicmp(s1: *const WCHAR, s2: *const WCHAR, n: usize) -> INT;
    pub fn iswspace(c: INT) -> INT;
    pub fn strcpy(dst: *mut CHAR, src: *const CHAR) -> *mut CHAR;
    pub fn free(block: *mut c_void);
}
// lstrlenA is not a CRT function — it is kernel32's, WINAPI, so `_lstrlenA@4` on i386.
// It sat in the cdecl block above and links only because nothing calls it yet.
extern "system" {
    pub fn lstrlenA(s: *const CHAR) -> INT;
}

/// wine/unicode.h:34 `#define strlenW(s) wcslen((s))`
#[inline]
pub unsafe fn strlenW(s: *const WCHAR) -> usize {
    wcslen(s)
}
/// wine/unicode.h:35 `#define strcpyW(d,s) wcscpy((d),(s))`
#[inline]
pub unsafe fn strcpyW(d: *mut WCHAR, s: *const WCHAR) -> *mut WCHAR {
    wcscpy(d, s)
}
/// wine/unicode.h:36 `#define strcatW(d,s) wcscat((d),(s))`
#[inline]
pub unsafe fn strcatW(d: *mut WCHAR, s: *const WCHAR) -> *mut WCHAR {
    wcscat(d, s)
}
/// wine/unicode.h:38 `#define strstrW(d,s) wcsstr((d),(s))`
#[inline]
pub unsafe fn strstrW(d: *const WCHAR, s: *const WCHAR) -> *mut WCHAR {
    wcsstr(d, s)
}
/// wine/unicode.h:40 `#define strchrW(s,c) wcschr((s),(c))`
#[inline]
pub unsafe fn strchrW(s: *const WCHAR, c: WCHAR) -> *mut WCHAR {
    wcschr(s, c)
}
/// wine/unicode.h:41 `#define strrchrW(s,c) wcsrchr((s),(c))`
#[inline]
pub unsafe fn strrchrW(s: *const WCHAR, c: WCHAR) -> *mut WCHAR {
    wcsrchr(s, c)
}
/// wine/unicode.h:46 `#define strncmpiW(s1,s2,n) _wcsnicmp((s1),(s2),(n))`
#[inline]
pub unsafe fn strncmpiW(s1: *const WCHAR, s2: *const WCHAR, n: usize) -> INT {
    _wcsnicmp(s1, s2, n)
}
/// wine/unicode.h:58 `#define isspaceW(n) iswspace((n))`
#[inline]
pub unsafe fn isspaceW(c: WCHAR) -> INT {
    iswspace(c as INT)
}

// windef.h `min`/`max` are function-like MACROS, so they must stay generic and inline —
// an out-of-line no-op or a single-type fn would silently change which overload the
// translated call resolves to.
#[inline]
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a <= b { a } else { b }
}
#[inline]
pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a >= b { a } else { b }
}

// SDK records the Wave-5 TUs need that a per-class module already proved. RE-EXPORTED,
// not re-transcribed: a second copy of a layout is a second identity, and the two drift
// silently. Promoting the definitions here outright is a separate sweep — see the header
// note about local definitions shadowing this module's glob.
pub use com_sys::{HFONT, SECURITY_ATTRIBUTES};
pub use crate::admin_tools_folder::IPersistFolder;
pub use crate::cfs_folder::{
    CComHeapPtr, IMoniker, IPersistFolder3, IShellFolderViewCB, PERSIST_FOLDER_TARGET_INFO,
    QCMINFO, WIN32_FIND_DATAW,
};
pub use crate::control_panel_folder::{IContextMenu, ITEMIDLIST_ABSOLUTE, SHELLEXECUTEINFOW};
pub use crate::copy_to_menu::{BROWSEINFOW, IObjectWithSite, SHFILEOPSTRUCTW};
pub use crate::drv_def_ext::HDROP;
pub use crate::file_def_ext::OPENASINFO;
pub use crate::new_menu::{PROCESS_INFORMATION, STARTUPINFOW};
pub use crate::shell_dispatch::SHELLSTATE;
pub use crate::shell_link::{
    BITMAPINFO, BITMAPINFOHEADER, CMINVOKECOMMANDINFOEX, HIMAGELIST, HLOCAL, IExtractIconW,
    RGBQUAD,
};
pub use crate::wine::shell32_main::{BITMAP, ICONINFO, PAINTSTRUCT};
pub use crate::wine::shellstring::LPSTRRET;

// Base types the promoted declarations reference. Aliases of types this crate already
// has, so they are declared here rather than re-exported from whichever module happened
// to spell them first; `LPTPMPARAMS` is a real pointer type owned by wine::shlmenu.
pub use crate::island::HGDIOBJ; // windef.h
pub use crate::island::HMODULE; // minwindef.h
pub type SHORT = i16; // winnt.h
pub use core::ffi::c_int;
pub use crate::wine::shlmenu::LPTPMPARAMS;

// ---------------------------------------------------------------------------
// Win32/shell imports promoted here by `rustconv.extern_promote`. Each was already
// declared in one or more sibling modules; promotion required every copy to agree
// SEMANTICALLY (ABI + parameter TYPES + return type, after resolving type aliases —
// parameter names are not ABI, and `LPWSTR`/`*mut u16` are one type).
//
// Four names did NOT agree, and were adjudicated against their headers rather than
// picked: SHAnsiToUnicode (one module had it returning a POINTER; shlwapi_undoc.h:264
// says DWORD, and its one call site discards the value, so nothing was miscompiled),
// PathUnquoteSpacesW (one had it returning nothing; shlwapi.h:395 says BOOL), plus
// GetSystemMetrics and HLM_GetIconW (UINT vs INT first parameter; winuser.h:4904 and
// shell32_main.h:79 both say int). Those six files were corrected in place.
//
// Module-local copies stay for now — a local declaration shadows a glob import, so this
// is additive. Retiring them is the separate sweep the header note mentions.
// ---------------------------------------------------------------------------

extern "C" {
    pub fn PathIsExeW(path: LPCWSTR) -> BOOL;
    pub fn _wcsicmp(left: LPCWSTR, right: LPCWSTR) -> INT;
    pub fn _wtoi(str: *const WCHAR) -> INT;
    pub fn memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
    pub fn memset(dst: *mut core::ffi::c_void, value: INT, bytes: usize) -> *mut core::ffi::c_void;
    pub fn swprintf(buffer: *mut WCHAR, format: *const WCHAR, ...) -> INT;
}

extern "system" {
    pub fn CoTaskMemFree(memory: LPVOID);
    pub fn CoUninitialize();
    pub fn DeleteFileW(path: LPCWSTR) -> BOOL;
    pub fn DeleteObject(object: HGDIOBJ) -> BOOL;
    pub fn ExpandEnvironmentStringsW(lpSrc: LPCWSTR, lpDst: LPWSTR, nSize: DWORD) -> DWORD;
    pub fn FindClose(hFindFile: HANDLE) -> BOOL;
    pub fn FindExecutableW(file: LPCWSTR, directory: LPCWSTR, result: LPWSTR) -> HINSTANCE;
    pub fn GetBinaryTypeW(lpApplicationName: LPCWSTR, lpBinaryType: LPDWORD) -> BOOL;
    pub fn GetCurrentDirectoryW(nBufferLength: DWORD, lpBuffer: LPWSTR) -> DWORD;
    pub fn GetDriveTypeW(lpRootPathName: LPCWSTR) -> UINT;
    pub fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD;
    pub fn GetFullPathNameW(name: LPCWSTR, len: DWORD, buf: LPWSTR, part: *mut LPWSTR) -> DWORD;
    pub fn GetKeyState(vk: c_int) -> SHORT;
    pub fn GetLastError() -> DWORD;
    pub fn GetObjectW(h: HGDIOBJ, c: INT, pv: LPVOID) -> INT;
    pub fn GetShortPathNameW(lpszLongPath: LPCWSTR, lpszShortPath: LPWSTR, cchBuffer: DWORD) -> DWORD;
    pub fn GetTickCount() -> DWORD;
    pub fn GetVersion() -> DWORD;
    pub fn GetWindowsDirectoryW(lpBuffer: LPWSTR, uSize: UINT) -> UINT;
    pub fn HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    pub fn ILCreateFromPathW(pszPath: LPCWSTR) -> LPITEMIDLIST;
    pub fn ILFindLastID(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;
    pub fn ILGetNext(pidl: PCUIDLIST_RELATIVE) -> LPITEMIDLIST;
    pub fn ILIsEqual(pidl1: LPCITEMIDLIST, pidl2: LPCITEMIDLIST) -> BOOL;
    pub fn ILRemoveLastID(pidl: LPITEMIDLIST) -> BOOL;
    pub fn LocalFree(hMem: HANDLE) -> HANDLE;
    pub fn MessageBeep(t: UINT) -> BOOL;
    pub fn MoveFileW(old: LPCWSTR, new: LPCWSTR) -> BOOL;
    pub fn PathAddBackslashW(pszPath: LPWSTR) -> LPWSTR;
    pub fn PathAddExtensionW(pszPath: LPWSTR, pszExt: LPCWSTR) -> BOOL;
    pub fn PathAppendW(pszPath: LPWSTR, pszMore: LPCWSTR) -> BOOL;
    pub fn PathCombineW(lpszDest: LPWSTR, lpszDir: LPCWSTR, lpszFile: LPCWSTR) -> LPWSTR;
    pub fn PathCreateFromUrlW(pszUrl: LPCWSTR, pszPath: LPWSTR, pcchPath: *mut DWORD, dwFlags: DWORD) -> HRESULT;
    pub fn PathFileExistsW(pszPath: LPCWSTR) -> BOOL;
    pub fn PathFindExtensionW(pszPath: LPCWSTR) -> LPWSTR;
    pub fn PathFindFileNameW(pszPath: LPCWSTR) -> LPWSTR;
    pub fn PathGetArgsW(path: LPCWSTR) -> LPWSTR;
    pub fn PathGetDriveNumberW(path: LPCWSTR) -> INT;
    pub fn PathIsDirectoryW(pszPath: LPCWSTR) -> BOOL;
    pub fn PathIsRelativeW(pszPath: LPCWSTR) -> BOOL;
    pub fn PathIsURLW(pszPath: LPCWSTR) -> BOOL;
    pub fn PathRemoveArgsW(path: LPWSTR);
    pub fn PathRemoveFileSpecW(pszPath: LPWSTR) -> BOOL;
    pub fn PathResolveW(path: LPWSTR, dirs: *mut LPCWSTR, flags: DWORD) -> BOOL;
    pub fn PathStripToRootW(pszPath: LPWSTR) -> BOOL;
    pub fn PathUnquoteSpacesW(path: LPWSTR) -> BOOL;
    pub fn PathYetAnotherMakeUniqueName(buffer: LPWSTR, path: LPCWSTR, short: LPCWSTR, long: LPCWSTR) -> BOOL;
    pub fn SHAllocShared(pvData: LPCVOID, dwSize: DWORD, dwProcessId: DWORD) -> HANDLE;
    pub fn SHAnsiToUnicode(pszSrc: LPCSTR, pwszDst: LPWSTR, cwchBuf: INT) -> DWORD;
    pub fn SHChangeNotify(event: LONG, flags: UINT, item1: LPCVOID, item2: LPCVOID);
    pub fn SHELL_CreateSHDESCRIPTIONID(pVar: *mut VARIANT, Id: DWORD, pCLSID: *const CLSID) -> HRESULT;
    pub fn SHELL_CreateVariantBuffer(pVar: *mut VARIANT, cb: UINT) -> HRESULT;
    pub fn SHLockShared(hData: HANDLE, dwProcessId: DWORD) -> PVOID;
    pub fn SHUnlockShared(pvData: PVOID) -> BOOL;
    pub fn SearchPathW(lpPath: LPCWSTR, lpFileName: LPCWSTR, lpExtension: LPCWSTR, nBufferLength: DWORD, lpBuffer: LPWSTR, lpFilePart: *mut LPWSTR) -> DWORD;
    pub fn SetFileAttributesW(lpFileName: LPCWSTR, dwFileAttributes: DWORD) -> BOOL;
    pub fn SetLastError(dwErrCode: DWORD);
    pub fn Shell_GetImageLists(lpBigList: *mut HIMAGELIST, lpSmallList: *mut HIMAGELIST) -> BOOL;
    pub fn StrChrW(start: LPCWSTR, character: WCHAR) -> LPWSTR;
    pub fn StrCmpIW(psz1: LPCWSTR, psz2: LPCWSTR) -> INT;
    pub fn StrCpyNW(pszDst: LPWSTR, pszSrc: LPCWSTR, cchMax: INT) -> LPWSTR;
    pub fn StrDupW(lpsz: LPCWSTR) -> LPWSTR;
    pub fn StrPBrkW(psz: LPCWSTR, pszSet: LPCWSTR) -> LPWSTR;
    pub fn StrTrimW(psz: LPWSTR, pszTrimChars: LPCWSTR) -> BOOL;
    pub fn StringFromGUID2(rguid: REFGUID, lpsz: LPWSTR, cchMax: INT) -> INT;
    pub fn UrlIsW(pszUrl: LPCWSTR, UrlIs: DWORD) -> BOOL;
    pub fn VariantChangeType(dst: *mut VARIANT, src: *const VARIANT, flags: USHORT, vt: VARTYPE) -> HRESULT;
    pub fn lstrcatW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR;
    pub fn lstrcmpiW(left: LPCWSTR, right: LPCWSTR) -> INT;
    pub fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR;
    pub fn lstrlenW(lpString: LPCWSTR) -> INT;
}

// NOT `extern "system"`. utils.cpp declares both of these as a plain `HRESULT Foo(...)` with
// no WINAPI (precomp.h:296 for SHGetNameAndFlagsW), so they are CDECL — which is also what the
// bindgen island says, and what shell32/utils.rs now DEFINES them as. Declaring them stdcall
// here was invisible while utils.cpp owned the symbols, because the C++ ones are mangled and
// nothing resolved; once the Rust definition landed unmangled these decls started binding to
// it, and on i386 a cdecl callee entered through a stdcall declaration corrupts the stack.
// The other seven declarations of these two names in this crate all carry
// `#[link_name = "..._RustBridge"]`, which IS `EXTERN_C ... WINAPI`, and are correct as-is.
// (scan_cc_mismatch)
extern "C" {
    pub fn SHGetNameAndFlagsW(pidl: LPCITEMIDLIST, flags: DWORD, text: LPWSTR, cch: UINT, attributes: *mut DWORD) -> HRESULT;
    pub fn SHILAppend(pidl: LPITEMIDLIST, ppidl: *mut LPITEMIDLIST) -> HRESULT;
}
