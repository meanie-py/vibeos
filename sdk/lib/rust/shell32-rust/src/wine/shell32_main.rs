//! Shell basics — translation of dll/win32/shell32/wine/shell32_main.c.
//!
//! Build configuration mirrored: `-D__REACTOS__ -DUNICODE -D_UNICODE -D_WIN32_WINNT=0x600`.
//! All the `#ifdef __REACTOS__` branches are the ACTIVE ones here (the About dialog draws
//! the ROS logo / version / physmem panel and an authors sub-dialog — the Wine-only
//! `#ifndef __REACTOS__` paths do not exist in this object). The Wine `#if 0`/non-ROS
//! blocks are not present. Verified against `strings -el` of the pre-flip object
//! (the only active string literals are the registry path + the two memory format strings).
//!
//! 18 exports: 17 WINAPI (`extern "system"` → `_Name@N` on i386) + `SHELL_IsShortcut`
//! (cdecl `extern "C"`, no `@N`). Import decorations confirmed by nm of the pre-flip obj.
//!
//! The internal PIDL helpers `_ILGetExtension`/`_ILIsDrive` resolve as cdecl C exports from
//! the still-C pidl.c (Wave-3 point 1). The COM `IShellFolder`/`IShellFolder2`/
//! `IShellDetails`/`IExtractIconW` vtables are modeled `#[repr(C)]` exactly as
//! shellole.rs/classes.rs do (only the invoked slots carry real signatures).

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use crate::types::*;
use core::ffi::c_void;
use core::mem::size_of;
use core::ptr::{null, null_mut, write_volatile};

// ── extra type aliases (hoist candidates) ───────────────────────────────────────────────
pub type ULONG = u32; // winnt.h
pub type SIZE_T = usize; // basetsd.h
pub type DWORDLONG = u64; // winnt.h
pub type SFGAOF = DWORD; // shtypes.h
pub type SHGDNF = DWORD; // shobjidl
pub type HICON = HANDLE; // windef.h
pub type HBITMAP = HANDLE; // windef.h
pub type HDC = HANDLE; // windef.h
pub type HGDIOBJ = HANDLE; // windef.h
pub type HRSRC = HANDLE; // minwindef.h
pub type HGLOBAL = HANDLE; // minwindef.h
pub type HMODULE = HANDLE; // minwindef.h
pub type HIMAGELIST = HANDLE; // commctrl.h
pub type HLOCAL = HANDLE; // minwindef.h
pub type WPARAM = UINT_PTR; // minwindef.h
pub type LPARAM = LONG_PTR; // minwindef.h
pub type LRESULT = LONG_PTR; // minwindef.h

/// ITEMIDLIST is opaque here (PIDLs are produced/consumed, never field-accessed in this file).
#[repr(C)]
pub struct ITEMIDLIST {
    _private: [u8; 0],
}
pub type LPITEMIDLIST = *mut ITEMIDLIST;
pub type LPCITEMIDLIST = *const ITEMIDLIST;

/// DLGPROC / DLGTEMPLATE — the template is only ever an opaque resource pointer.
pub type DLGPROC = unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR;

// ── GUID / IID (guiddef.h) ──────────────────────────────────────────────────────────────
#[repr(C)]
#[derive(Copy, Clone)]
pub struct GUID {
    pub Data1: u32,
    pub Data2: u16,
    pub Data3: u16,
    pub Data4: [u8; 8],
}
pub type IID = GUID;
pub type CLSID = GUID;
pub type REFIID = *const IID;
pub type REFCLSID = *const CLSID;

// ── const fn wide/ansi string builders (no L""/_T in Rust) ───────────────────────────────
const fn wsz<const N: usize>(s: &str) -> [WCHAR; N] {
    let b = s.as_bytes();
    let mut out = [0u16; N];
    let mut i = 0;
    while i < b.len() {
        out[i] = b[i] as WCHAR;
        i += 1;
    }
    out
}
const fn asz<const N: usize>(s: &str) -> [CHAR; N] {
    let b = s.as_bytes();
    let mut out = [0i8; N];
    let mut i = 0;
    while i < b.len() {
        out[i] = b[i] as CHAR;
        i += 1;
    }
    out
}

// ── constants pinned from SDK headers ───────────────────────────────────────────────────
const MAX_PATH: usize = 260; // minwindef.h

const ERROR_INVALID_PARAMETER: DWORD = 87; // winerror.h
const LMEM_FIXED: UINT = 0x0000; // minwinbase.h
const CP_ACP: UINT = 0; // winnls.h
const CP_UTF8: UINT = 65001; // winnls.h
const UNICODE_NULL: WCHAR = 0; // winnt.h

const S_OK_: HRESULT = 0; // winerror.h (types.rs has S_OK)
const E_FAIL: HRESULT = 0x80004005u32 as HRESULT; // winerror.h
const DISP_E_MEMBERNOTFOUND: HRESULT = 0x80020003u32 as HRESULT; // winerror.h

// shgfi_get_exe_type / PE parsing
const GENERIC_READ: DWORD = 0x80000000; // winnt.h
const FILE_SHARE_READ: DWORD = 0x00000001; // winnt.h
const OPEN_EXISTING: DWORD = 3; // winbase.h
const SEEK_SET: DWORD = 0; // stdio.h (FILE_BEGIN)
const SCS_DOS_BINARY: DWORD = 1; // winbase.h
const SCS_PIF_BINARY: DWORD = 3; // winbase.h
const IMAGE_NT_SIGNATURE: DWORD = 0x00004550; // winnt.h ("PE\0\0")
const IMAGE_OS2_SIGNATURE: WORD = 0x454E; // winnt.h ("NE")
const IMAGE_FILE_DLL: WORD = 0x2000; // winnt.h
const IMAGE_SUBSYSTEM_WINDOWS_GUI: WORD = 2; // winnt.h

const INVALID_FILE_ATTRIBUTES: DWORD = 0xFFFFFFFF; // fileapi.h ((DWORD)-1)
const FILE_ATTRIBUTE_DIRECTORY: DWORD = 0x00000010; // winnt.h

// SHGFI flags (shellapi.h)
const SHGFI_ICON: UINT = 256;
const SHGFI_DISPLAYNAME: UINT = 512;
const SHGFI_TYPENAME: UINT = 1024;
const SHGFI_ATTRIBUTES: UINT = 2048;
const SHGFI_ICONLOCATION: UINT = 4096;
const SHGFI_EXETYPE: UINT = 8192;
const SHGFI_SYSICONINDEX: UINT = 16384;
const SHGFI_LINKOVERLAY: UINT = 32768;
const SHGFI_SELECTED: UINT = 65536;
const SHGFI_ATTR_SPECIFIED: UINT = 131072;
const SHGFI_SMALLICON: UINT = 1;
const SHGFI_OPENICON: UINT = 2;
const SHGFI_SHELLICONSIZE: UINT = 4;
const SHGFI_PIDL: UINT = 8;
const SHGFI_USEFILEATTRIBUTES: UINT = 16;
const SHGFI_ADDOVERLAYS: UINT = 32;
const SHGFI_OVERLAYINDEX: UINT = 64;
// #define SHGFI_KNOWN_FLAGS (shell32_main.c, in-file)
const SHGFI_KNOWN_FLAGS: UINT = SHGFI_SMALLICON
    | SHGFI_OPENICON
    | SHGFI_SHELLICONSIZE
    | SHGFI_PIDL
    | SHGFI_USEFILEATTRIBUTES
    | SHGFI_ADDOVERLAYS
    | SHGFI_OVERLAYINDEX
    | SHGFI_ICON
    | SHGFI_DISPLAYNAME
    | SHGFI_TYPENAME
    | SHGFI_ATTRIBUTES
    | SHGFI_ICONLOCATION
    | SHGFI_EXETYPE
    | SHGFI_SYSICONINDEX
    | SHGFI_LINKOVERLAY
    | SHGFI_SELECTED
    | SHGFI_ATTR_SPECIFIED;

// GIL flags (shobjidl.idl)
const GIL_OPENICON: UINT = 0x0001;
const GIL_NOTFILENAME: UINT = 0x0008;
const GIL_FORSHORTCUT: UINT = 0x0080;

const SHGDN_INFOLDER: SHGDNF = 1; // shobjidl.idl

// imagelist / icon-load (commctrl.h / winuser.h)
const ILD_NORMAL: UINT = 0x0; // commctrl.h
const SM_CXICON: INT = 11; // winuser.h
const SM_CYICON: INT = 12; // winuser.h
const SM_CXSMICON: INT = 49; // winuser.h
const SM_CYSMICON: INT = 50; // winuser.h
const LR_DEFAULTCOLOR: UINT = 0; // winuser.h
const LR_DEFAULTSIZE: UINT = 64; // winuser.h
const IMAGE_BITMAP: UINT = 0; // winuser.h
const IMAGE_ICON: UINT = 1; // winuser.h
const IDI_WINLOGO: WORD = 32517; // winuser.h MAKEINTRESOURCE(32517)

// SHELL resource IDs (shresdef.h)
const IDI_SHELL_FOLDER: INT = 4;
const IDI_SHELL_PRINTERS_FOLDER: INT = 38;
// The About banner lives in rosbrand.dll now, not in shell32's own resources: upstream's
// "[SHELL32] Use new branding library" (afd1ef71031) deleted reactos.bmp (131) and line.bmp
// (138) from bitmap_res.rc along with their IDB_REACTOS/IDB_LINEBAR defines. This port kept
// loading 131/138 from shell32_hInstance after that merge — resources that no longer exist, so
// both LoadImageW calls failed and the dialog lost its logo, with nothing red anywhere. The
// stale-island audit is what surfaced it (both names vanished from the regenerated island).
// reactos/rosbrand.h:8-9.
const IDB_BRAND_BANNER: WORD = 100;
const IDB_BRAND_BANNERLINE: WORD = 101;
static W_ROSBRAND_DLL: [WCHAR; 13] = wsz("rosbrand.dll");
const LOAD_LIBRARY_AS_DATAFILE: DWORD = 0x0000_0002; // winbase.h
const IDD_ABOUT: WORD = 2;
const IDD_ABOUT_AUTHORS: WORD = 3;
const IDC_ABOUT_ICON: INT = 0x3009;
const IDC_ABOUT_APPNAME: INT = 0x3500;
const IDS_ABOUT_VERSION_STRING: UINT = 0x3501;
const IDC_ABOUT_VERSION: INT = 0x3502;
const IDC_ABOUT_OTHERSTUFF: INT = 0x350D;
const IDC_ABOUT_REG_TO: INT = 0x3506;
const IDC_ABOUT_REG_USERNAME: INT = 0x3507;
const IDC_ABOUT_REG_ORGNAME: INT = 0x3508;
const IDC_ABOUT_PHYSMEM: INT = 0x3503;
const IDC_ABOUT_AUTHORS: INT = 0x4101;
const IDC_ABOUT_AUTHORS_LISTBOX: INT = 0x4102;
const IDS_SHELL_ABOUT_AUTHORS: UINT = 144;
const IDS_SHELL_ABOUT_BACK: UINT = 145;

// window messages / misc (winuser.h, wingdi.h, winreg.h, winnls.h)
const WM_INITDIALOG: UINT = 0x0110;
const WM_PAINT: UINT = 15;
const WM_COMMAND: UINT = 0x0111;
const WM_CLOSE: UINT = 0x0010;
const WM_SETREDRAW: UINT = 0x000B;
const LB_ADDSTRING: UINT = 384;
const STM_SETICON: UINT = 368;
const SW_HIDE: INT = 0;
const SW_SHOW: INT = 5;
const IDOK: WPARAM = 1; // winuser.h
const IDCANCEL: WPARAM = 2; // winuser.h
const SRCCOPY: DWORD = 0x00CC0020; // wingdi.h
const KEY_QUERY_VALUE: REGSAM = 0x0001; // winnt.h
const REG_SZ: DWORD = 1; // winnt.h
const LOCALE_USER_DEFAULT: DWORD = 0x0400; // winnls.h MAKELCID(LANG_USER_DEFAULT,SORT_DEFAULT)
const LOCALE_SDECIMAL: DWORD = 14; // winnls.h
#[cfg(target_arch = "x86")]
const GWLP_HINSTANCE: INT = -6; // winuser.h (GWL_HINSTANCE)
#[cfg(not(target_arch = "x86"))]
const GWLP_HINSTANCE: INT = -6; // winuser.h

// HKEY_LOCAL_MACHINE = (HKEY)(ULONG_PTR)(LONG)0x80000002
#[inline]
fn hkey_local_machine() -> HKEY {
    0x80000002u32 as i32 as isize as usize as *mut c_void
}

// build-version literals baked at compile time, exactly like the C `#include`s
// (reactos/version.h: COPYRIGHT_YEAR="2026"; buildno.h: KERNEL_VERSION_STR / _BUILD_STR).
static KERNEL_VERSION_STR: [CHAR; 16] = asz("0.4.16-x86-dev");
static KERNEL_VERSION_BUILD_STR: [CHAR; 18] = asz("20260628-b2549d2");

// ── wide string literals ────────────────────────────────────────────────────────────────
static W_IS_SHORTCUT: [WCHAR; 16] = wsz("IsShortcut");
static W_NT_CURRENTVERSION: [WCHAR; 64] = wsz("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion");
static W_REGISTERED_OWNER: [WCHAR; 24] = wsz("RegisteredOwner");
static W_REGISTERED_ORG: [WCHAR; 32] = wsz("RegisteredOrganization");
static W_PERCENT_1: [WCHAR; 4] = wsz("%1"); // L"%1"
static W_PB: [WCHAR; 4] = wsz("PB");
static W_TB: [WCHAR; 4] = wsz("TB");
static W_GB: [WCHAR; 4] = wsz("GB");
static W_FMT_MEM_DEC: [WCHAR; 16] = wsz("%u%s%02u %s"); // L"%u%s%02u %s"
static W_FMT_MEM_MB: [WCHAR; 8] = wsz("%u MB"); // L"%u MB"

// authors list (utf-8), terminated with a NULL pointer — file-private (no external refs).
static AUTHOR0: [CHAR; 30] = asz("Copyright 1993-2026 WINE team");
static AUTHOR1: [CHAR; 33] = asz("Copyright 1998-2026 ReactOS Team");
/// Newtype so the raw-pointer table can be a `static` (raw ptrs aren't `Sync` by default).
struct AuthorsTable([*const CHAR; 3]);
// SAFETY: read-only after init; the authors UI callback is single-threaded.
unsafe impl Sync for AuthorsTable {}
static SHELL_Authors: AuthorsTable =
    AuthorsTable([AUTHOR0.as_ptr(), AUTHOR1.as_ptr(), null::<CHAR>()]);

// ── structs (faithful SDK layouts) ──────────────────────────────────────────────────────
/// SHFILEINFOA (shellapi.h)
#[repr(C)]
pub struct SHFILEINFOA {
    pub hIcon: HICON,
    pub iIcon: INT,
    pub dwAttributes: DWORD,
    pub szDisplayName: [CHAR; MAX_PATH],
    pub szTypeName: [CHAR; 80],
}
/// SHFILEINFOW (shellapi.h)
#[repr(C)]
pub struct SHFILEINFOW {
    pub hIcon: HICON,
    pub iIcon: INT,
    pub dwAttributes: DWORD,
    pub szDisplayName: [WCHAR; MAX_PATH],
    pub szTypeName: [WCHAR; 80],
}

/// STRRET (shtypes.idl, pshpack8 — repr(C) gives identical offsets since 8 is a multiple of 4).
#[repr(C)]
pub union STRRET_u {
    pub pOleStr: LPWSTR,        // STRRET_WSTR
    pub uOffset: UINT,          // STRRET_OFFSET
    pub cStr: [CHAR; MAX_PATH], // STRRET_CSTR
}
#[repr(C)]
pub struct STRRET {
    pub uType: UINT,
    pub u: STRRET_u,
}

/// SHELLDETAILS (shtypes.idl, pshpack1 — `str` lands at offset 8 under plain repr(C) too).
#[repr(C)]
pub struct SHELLDETAILS {
    pub fmt: INT,
    pub cxChar: INT,
    pub str_: STRRET,
}

/// ICONINFO (winuser.h)
#[repr(C)]
pub struct ICONINFO {
    pub fIcon: BOOL,
    pub xHotspot: DWORD,
    pub yHotspot: DWORD,
    pub hbmMask: HBITMAP,
    pub hbmColor: HBITMAP,
}

/// BITMAP (wingdi.h)
#[repr(C)]
pub struct BITMAP {
    pub bmType: LONG,
    pub bmWidth: LONG,
    pub bmHeight: LONG,
    pub bmWidthBytes: LONG,
    pub bmPlanes: WORD,
    pub bmBitsPixel: WORD,
    pub bmBits: LPVOID,
}

/// MEMORYSTATUSEX (winbase.h)
#[repr(C)]
pub struct MEMORYSTATUSEX {
    pub dwLength: DWORD,
    pub dwMemoryLoad: DWORD,
    pub ullTotalPhys: DWORDLONG,
    pub ullAvailPhys: DWORDLONG,
    pub ullTotalPageFile: DWORDLONG,
    pub ullAvailPageFile: DWORDLONG,
    pub ullTotalVirtual: DWORDLONG,
    pub ullAvailVirtual: DWORDLONG,
    pub ullAvailExtendedVirtual: DWORDLONG,
}

/// RECT (windef.h)
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RECT {
    pub left: LONG,
    pub top: LONG,
    pub right: LONG,
    pub bottom: LONG,
}
/// PAINTSTRUCT (winuser.h)
#[repr(C)]
pub struct PAINTSTRUCT {
    pub hdc: HDC,
    pub fErase: BOOL,
    pub rcPaint: RECT,
    pub fRestore: BOOL,
    pub fIncUpdate: BOOL,
    pub rgbReserved: [BYTE; 32],
}

/// ABOUT_INFO (file-private; __REACTOS__ variant with szOSVersion).
#[repr(C)]
pub struct ABOUT_INFO {
    pub szApp: LPCWSTR,
    pub szOSVersion: LPCWSTR,
    pub szOtherStuff: LPCWSTR,
    pub hIcon: HICON,
}

// PE headers (winnt.h)
#[repr(C)]
pub struct IMAGE_DOS_HEADER {
    pub e_magic: WORD,
    pub e_cblp: WORD,
    pub e_cp: WORD,
    pub e_crlc: WORD,
    pub e_cparhdr: WORD,
    pub e_minalloc: WORD,
    pub e_maxalloc: WORD,
    pub e_ss: WORD,
    pub e_sp: WORD,
    pub e_csum: WORD,
    pub e_ip: WORD,
    pub e_cs: WORD,
    pub e_lfarlc: WORD,
    pub e_ovno: WORD,
    pub e_res: [WORD; 4],
    pub e_oemid: WORD,
    pub e_oeminfo: WORD,
    pub e_res2: [WORD; 10],
    pub e_lfanew: LONG,
}
#[repr(C)]
pub struct IMAGE_FILE_HEADER {
    pub Machine: WORD,
    pub NumberOfSections: WORD,
    pub TimeDateStamp: DWORD,
    pub PointerToSymbolTable: DWORD,
    pub NumberOfSymbols: DWORD,
    pub SizeOfOptionalHeader: WORD,
    pub Characteristics: WORD,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct IMAGE_DATA_DIRECTORY {
    pub VirtualAddress: DWORD,
    pub Size: DWORD,
}
#[cfg(target_arch = "x86")]
#[repr(C)]
pub struct IMAGE_OPTIONAL_HEADER {
    pub Magic: WORD,
    pub MajorLinkerVersion: BYTE,
    pub MinorLinkerVersion: BYTE,
    pub SizeOfCode: DWORD,
    pub SizeOfInitializedData: DWORD,
    pub SizeOfUninitializedData: DWORD,
    pub AddressOfEntryPoint: DWORD,
    pub BaseOfCode: DWORD,
    pub BaseOfData: DWORD,
    pub ImageBase: DWORD,
    pub SectionAlignment: DWORD,
    pub FileAlignment: DWORD,
    pub MajorOperatingSystemVersion: WORD,
    pub MinorOperatingSystemVersion: WORD,
    pub MajorImageVersion: WORD,
    pub MinorImageVersion: WORD,
    pub MajorSubsystemVersion: WORD,
    pub MinorSubsystemVersion: WORD,
    pub Win32VersionValue: DWORD,
    pub SizeOfImage: DWORD,
    pub SizeOfHeaders: DWORD,
    pub CheckSum: DWORD,
    pub Subsystem: WORD,
    pub DllCharacteristics: WORD,
    pub SizeOfStackReserve: DWORD,
    pub SizeOfStackCommit: DWORD,
    pub SizeOfHeapReserve: DWORD,
    pub SizeOfHeapCommit: DWORD,
    pub LoaderFlags: DWORD,
    pub NumberOfRvaAndSizes: DWORD,
    pub DataDirectory: [IMAGE_DATA_DIRECTORY; 16],
}
#[cfg(not(target_arch = "x86"))]
#[repr(C)]
pub struct IMAGE_OPTIONAL_HEADER {
    pub Magic: WORD,
    pub MajorLinkerVersion: BYTE,
    pub MinorLinkerVersion: BYTE,
    pub SizeOfCode: DWORD,
    pub SizeOfInitializedData: DWORD,
    pub SizeOfUninitializedData: DWORD,
    pub AddressOfEntryPoint: DWORD,
    pub BaseOfCode: DWORD,
    pub ImageBase: u64,
    pub SectionAlignment: DWORD,
    pub FileAlignment: DWORD,
    pub MajorOperatingSystemVersion: WORD,
    pub MinorOperatingSystemVersion: WORD,
    pub MajorImageVersion: WORD,
    pub MinorImageVersion: WORD,
    pub MajorSubsystemVersion: WORD,
    pub MinorSubsystemVersion: WORD,
    pub Win32VersionValue: DWORD,
    pub SizeOfImage: DWORD,
    pub SizeOfHeaders: DWORD,
    pub CheckSum: DWORD,
    pub Subsystem: WORD,
    pub DllCharacteristics: WORD,
    pub SizeOfStackReserve: u64,
    pub SizeOfStackCommit: u64,
    pub SizeOfHeapReserve: u64,
    pub SizeOfHeapCommit: u64,
    pub LoaderFlags: DWORD,
    pub NumberOfRvaAndSizes: DWORD,
    pub DataDirectory: [IMAGE_DATA_DIRECTORY; 16],
}
#[repr(C)]
pub struct IMAGE_NT_HEADERS {
    pub Signature: DWORD,
    pub FileHeader: IMAGE_FILE_HEADER,
    pub OptionalHeader: IMAGE_OPTIONAL_HEADER,
}
#[repr(C)]
pub struct IMAGE_OS2_HEADER {
    pub ne_magic: WORD,
    pub ne_ver: CHAR,
    pub ne_rev: CHAR,
    pub ne_enttab: WORD,
    pub ne_cbenttab: WORD,
    pub ne_crc: LONG,
    pub ne_flags: WORD,
    pub ne_autodata: WORD,
    pub ne_heap: WORD,
    pub ne_stack: WORD,
    pub ne_csip: LONG,
    pub ne_sssp: LONG,
    pub ne_cseg: WORD,
    pub ne_cmod: WORD,
    pub ne_cbnrestab: WORD,
    pub ne_segtab: WORD,
    pub ne_rsrctab: WORD,
    pub ne_restab: WORD,
    pub ne_modtab: WORD,
    pub ne_imptab: WORD,
    pub ne_nrestab: LONG,
    pub ne_cmovent: WORD,
    pub ne_align: WORD,
    pub ne_cres: WORD,
    pub ne_exetyp: BYTE,
    pub ne_flagsothers: BYTE,
    pub ne_pretthunks: WORD,
    pub ne_psegrefbytes: WORD,
    pub ne_swaparea: WORD,
    pub ne_expver: WORD,
}

// ── COM vtables (STDMETHODCALLTYPE == __stdcall → extern "system"; only invoked slots typed) ─
#[repr(C)]
pub struct IUnknown {
    pub lpVtbl: *mut IUnknownVtbl,
}
#[repr(C)]
pub struct IUnknownVtbl {
    pub QueryInterface: usize,
    pub AddRef: usize,
    pub Release: unsafe extern "system" fn(*mut IUnknown) -> ULONG,
}

#[repr(C)]
pub struct IShellFolder {
    pub lpVtbl: *mut IShellFolderVtbl,
}
#[repr(C)]
pub struct IShellFolderVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IShellFolder, REFIID, *mut LPVOID) -> HRESULT,
    pub AddRef: usize,
    pub Release: unsafe extern "system" fn(*mut IShellFolder) -> ULONG,
    pub ParseDisplayName: usize,
    pub EnumObjects: usize,
    pub BindToObject: usize,
    pub BindToStorage: usize,
    pub CompareIDs: usize,
    pub CreateViewObject: usize,
    pub GetAttributesOf: unsafe extern "system" fn(
        *mut IShellFolder,
        UINT,
        *const LPCITEMIDLIST,
        *mut SFGAOF,
    ) -> HRESULT,
    pub GetUIObjectOf: unsafe extern "system" fn(
        *mut IShellFolder,
        HWND,
        UINT,
        *const LPCITEMIDLIST,
        REFIID,
        *mut UINT,
        *mut LPVOID,
    ) -> HRESULT,
    pub GetDisplayNameOf: unsafe extern "system" fn(
        *mut IShellFolder,
        LPCITEMIDLIST,
        SHGDNF,
        *mut STRRET,
    ) -> HRESULT,
    pub SetNameOf: usize,
}

#[repr(C)]
pub struct IShellFolder2 {
    pub lpVtbl: *mut IShellFolder2Vtbl,
}
#[repr(C)]
pub struct IShellFolder2Vtbl {
    // IUnknown
    pub QueryInterface: usize,
    pub AddRef: usize,
    pub Release: unsafe extern "system" fn(*mut IShellFolder2) -> ULONG,
    // IShellFolder
    pub ParseDisplayName: usize,
    pub EnumObjects: usize,
    pub BindToObject: usize,
    pub BindToStorage: usize,
    pub CompareIDs: usize,
    pub CreateViewObject: usize,
    pub GetAttributesOf: usize,
    pub GetUIObjectOf: usize,
    pub GetDisplayNameOf: usize,
    pub SetNameOf: usize,
    // IShellFolder2
    pub GetDefaultSearchGUID: usize,
    pub EnumSearches: usize,
    pub GetDefaultColumn: usize,
    pub GetDefaultColumnState: usize,
    pub GetDetailsEx: usize,
    pub GetDetailsOf: unsafe extern "system" fn(
        *mut IShellFolder2,
        LPCITEMIDLIST,
        UINT,
        *mut SHELLDETAILS,
    ) -> HRESULT,
    pub MapColumnToSCID: usize,
}

#[repr(C)]
pub struct IShellDetails {
    pub lpVtbl: *mut IShellDetailsVtbl,
}
#[repr(C)]
pub struct IShellDetailsVtbl {
    pub QueryInterface: usize,
    pub AddRef: usize,
    pub Release: unsafe extern "system" fn(*mut IShellDetails) -> ULONG,
    pub GetDetailsOf: unsafe extern "system" fn(
        *mut IShellDetails,
        LPCITEMIDLIST,
        UINT,
        *mut SHELLDETAILS,
    ) -> HRESULT,
    pub ColumnClick: usize,
}

#[repr(C)]
pub struct IExtractIconW {
    pub lpVtbl: *mut IExtractIconWVtbl,
}
#[repr(C)]
pub struct IExtractIconWVtbl {
    pub QueryInterface: usize,
    pub AddRef: usize,
    pub Release: unsafe extern "system" fn(*mut IExtractIconW) -> ULONG,
    pub GetIconLocation: unsafe extern "system" fn(
        *mut IExtractIconW,
        UINT,
        LPWSTR,
        UINT,
        *mut INT,
        *mut UINT,
    ) -> HRESULT,
    pub Extract: usize,
}

// ── imports (WINAPI → extern "system", @N) ──────────────────────────────────────────────
extern "system" {
    // kernel32
    fn CloseHandle(hObject: HANDLE) -> BOOL; // @4
    fn CreateFileW(
        lpFileName: LPCWSTR,
        dwDesiredAccess: DWORD,
        dwShareMode: DWORD,
        lpSecurityAttributes: LPVOID,
        dwCreationDisposition: DWORD,
        dwFlagsAndAttributes: DWORD,
        hTemplateFile: HANDLE,
    ) -> HANDLE; // @28
    fn GetBinaryTypeW(lpApplicationName: LPCWSTR, lpBinaryType: LPDWORD) -> BOOL; // @8
    fn GetCurrentDirectoryW(nBufferLength: DWORD, lpBuffer: LPWSTR) -> DWORD; // @8
    fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD; // @4
    fn GetLocaleInfoW(Locale: DWORD, LCType: DWORD, lpLCData: LPWSTR, cchData: INT) -> INT; // @16
    fn GetModuleFileNameW(hModule: HMODULE, lpFilename: LPWSTR, nSize: DWORD) -> DWORD; // @12
    fn GetProcessHeap() -> HANDLE; // @0
    fn GlobalMemoryStatusEx(lpBuffer: *mut MEMORYSTATUSEX) -> BOOL; // @4
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID; // @12
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL; // @12
    fn LocalAlloc(uFlags: UINT, uBytes: SIZE_T) -> HLOCAL; // @8
    fn LocalFree(hMem: HLOCAL) -> HLOCAL; // @4
    fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: LPCSTR,
        cbMultiByte: INT,
        lpWideCharStr: LPWSTR,
        cchWideChar: INT,
    ) -> INT; // @24
    fn ReadFile(
        hFile: HANDLE,
        lpBuffer: LPVOID,
        nNumberOfBytesToRead: DWORD,
        lpNumberOfBytesRead: LPDWORD,
        lpOverlapped: LPVOID,
    ) -> BOOL; // @20
    fn SetFilePointer(
        hFile: HANDLE,
        lDistanceToMove: LONG,
        lpDistanceToMoveHigh: *mut LONG,
        dwMoveMethod: DWORD,
    ) -> DWORD; // @16
    fn SetLastError(dwErrCode: DWORD); // @4
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
    fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // @8
    fn lstrcpynW(lpString1: LPWSTR, lpString2: LPCWSTR, iMaxLength: INT) -> LPWSTR; // @12
    fn lstrcmpW(lpString1: LPCWSTR, lpString2: LPCWSTR) -> INT; // @8

    // ole32
    fn CoCreateInstance(
        rclsid: REFCLSID,
        pUnkOuter: *mut c_void,
        dwClsContext: DWORD,
        riid: REFIID,
        ppv: *mut LPVOID,
    ) -> HRESULT; // @20

    // advapi32
    fn RegCloseKey(hKey: HKEY) -> LONG; // @4
    fn RegOpenKeyExW(
        hKey: HKEY,
        lpSubKey: LPCWSTR,
        ulOptions: DWORD,
        samDesired: REGSAM,
        phkResult: PHKEY,
    ) -> LONG; // @20
    fn RegQueryValueExW(
        hKey: HKEY,
        lpValueName: LPCWSTR,
        lpReserved: LPDWORD,
        lpType: LPDWORD,
        lpData: LPBYTE,
        lpcbData: LPDWORD,
    ) -> LONG; // @24

    // shlwapi
    fn PathCombineW(lpszDest: LPWSTR, lpszDir: LPCWSTR, lpszFile: LPCWSTR) -> LPWSTR; // @12
    fn PathFindExtensionW(pszPath: LPCWSTR) -> LPWSTR; // @4
    fn PathIsRelativeW(pszPath: LPCWSTR) -> BOOL; // @4
    fn PathIsRootW(pPath: LPCWSTR) -> BOOL; // @4
    fn StrRetToStrNW(pszBuf: LPWSTR, cchBuf: UINT, pstr: *mut STRRET, pidl: LPCITEMIDLIST)
        -> BOOL; // @16 (shell32 shellstring.rs reimpl returns BOOL)

    // user32
    fn BeginPaint(hWnd: HWND, lpPaint: *mut PAINTSTRUCT) -> HDC; // @8
    fn CreateDialogParamW(
        hInstance: HINSTANCE,
        lpTemplateName: LPCWSTR,
        hWndParent: HWND,
        lpDialogFunc: Option<DLGPROC>,
        dwInitParam: LPARAM,
    ) -> HWND; // @20
    fn CreateIconIndirect(piconinfo: *mut ICONINFO) -> HICON; // @4
    fn DialogBoxIndirectParamW(
        hInstance: HINSTANCE,
        hDialogTemplate: LPVOID,
        hWndParent: HWND,
        lpDialogFunc: Option<DLGPROC>,
        dwInitParam: LPARAM,
    ) -> INT_PTR; // @20
    fn EndDialog(hDlg: HWND, nResult: INT_PTR) -> BOOL; // @8
    fn EndPaint(hWnd: HWND, lpPaint: *const PAINTSTRUCT) -> BOOL; // @8
    fn FindResourceW(hModule: HMODULE, lpName: LPCWSTR, lpType: LPCWSTR) -> HRSRC; // @12
    fn GetDlgItem(hDlg: HWND, nIDDlgItem: INT) -> HWND; // @8
    fn GetIconInfo(hIcon: HICON, piconinfo: *mut ICONINFO) -> BOOL; // @8
    fn GetSystemMetrics(nIndex: INT) -> INT; // @4
    fn GetWindowTextLengthW(hWnd: HWND) -> INT; // @4
    fn GetWindowTextW(hWnd: HWND, lpString: LPWSTR, nMaxCount: INT) -> INT; // @12
    fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // @8
    fn LoadImageW(
        hInst: HINSTANCE,
        name: LPCWSTR,
        type_: UINT,
        cx: INT,
        cy: INT,
        fuLoad: UINT,
    ) -> HANDLE; // @24
    fn LoadStringW(hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: INT) -> INT; // @16
    fn LoadLibraryExW(lpLibFileName: LPCWSTR, hFile: HANDLE, dwFlags: DWORD) -> HINSTANCE; // @12
    fn PrivateExtractIconsW(
        szFileName: LPCWSTR,
        nIconIndex: INT,
        cxIcon: INT,
        cyIcon: INT,
        phicon: *mut HICON,
        piconid: *mut UINT,
        nIcons: UINT,
        flags: UINT,
    ) -> UINT; // @32
    fn SendDlgItemMessageW(
        hDlg: HWND,
        nIDDlgItem: INT,
        Msg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
    ) -> LRESULT; // @20
    fn SetDlgItemTextW(hDlg: HWND, nIDDlgItem: INT, lpString: LPCWSTR) -> BOOL; // @12
    fn SetWindowTextW(hWnd: HWND, lpString: LPCWSTR) -> BOOL; // @8
    fn ShowWindow(hWnd: HWND, nCmdShow: INT) -> BOOL; // @8

    // gdi32
    fn BitBlt(
        hdc: HDC,
        x: INT,
        y: INT,
        cx: INT,
        cy: INT,
        hdcSrc: HDC,
        x1: INT,
        y1: INT,
        rop: DWORD,
    ) -> BOOL; // @36
    fn CreateCompatibleDC(hdc: HDC) -> HDC; // @4
    fn DeleteDC(hdc: HDC) -> BOOL; // @4
    fn DeleteObject(ho: HGDIOBJ) -> BOOL; // @4
    fn GetObjectW(h: HGDIOBJ, c: INT, pv: LPVOID) -> INT; // @12
    fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ; // @8

    // comctl32
    fn ImageList_GetIcon(himl: HIMAGELIST, i: INT, flags: UINT) -> HICON; // @12

    // kernel32 resource
    fn LoadResource(hModule: HMODULE, hResInfo: HRSRC) -> HGLOBAL; // @8
    fn GetWindowLongW(hWnd: HWND, nIndex: INT) -> LONG; // @8 (i386 GWLP_*→GWL_*)
}

// shell32 internal helpers (cdecl, from C pidl.c / shell32 .cpp side — no @N)
extern "C" {
    fn _ILGetExtension(pidl: LPCITEMIDLIST, pOut: LPWSTR, uOutSize: UINT) -> BOOL; // _ILGetExtension
    fn _ILIsDrive(pidl: LPCITEMIDLIST) -> BOOL; // _ILIsDrive
    fn HCR_GetProgIdKeyOfExtension(szExtension: LPCWSTR, phKey: PHKEY, AllowFallback: BOOL)
        -> HRESULT;
    fn HCR_MapTypeToValueW(
        szExtension: LPCWSTR,
        szFileType: LPWSTR,
        len: LONG,
        bPrependDot: BOOL,
    ) -> BOOL;
    fn HCR_GetIconW(
        szClass: LPCWSTR,
        szDest: LPWSTR,
        szName: LPCWSTR,
        len: DWORD,
        picon_idx: *mut INT,
    ) -> BOOL;
    fn SHELL32_AssocGetFSDirectoryDescription(Buf: LPWSTR, cchBuf: UINT) -> HRESULT;
    fn SHELL32_AssocGetFileDescription(Name: LPCWSTR, Buf: LPWSTR, cchBuf: UINT) -> HRESULT;
    fn SHELL32_CreateSimpleIDListFromPath(pszPath: LPCWSTR, dwAttributes: DWORD) -> LPITEMIDLIST;
    fn PidlToSicIndex(
        sh: *mut IShellFolder,
        pidl: LPCITEMIDLIST,
        bBigIcon: BOOL,
        uFlags: UINT,
        pIndex: *mut INT,
    ) -> BOOL;
    fn SIC_GetIconIndex(sSourceFile: LPCWSTR, dwSourceIndex: INT, dwFlags: DWORD) -> INT;
}

// shell32 exported WINAPI helpers used here (@N)
extern "system" {
    fn SHGetPathFromIDListW(pidl: LPCITEMIDLIST, pszPath: LPWSTR) -> BOOL; // @8
    fn SHILCreateFromPathW(
        path: LPCWSTR,
        ppidl: *mut LPITEMIDLIST,
        prgfInOut: LPDWORD,
    ) -> HRESULT; // @12
    fn SHBindToParent(
        pidl: LPCITEMIDLIST,
        riid: REFIID,
        ppv: *mut LPVOID,
        ppidlLast: *mut LPCITEMIDLIST,
    ) -> HRESULT; // @16
    fn SHFree(pv: LPVOID); // @4
    fn Shell_GetImageLists(lpBigList: *mut HIMAGELIST, lpSmallList: *mut HIMAGELIST) -> BOOL; // @8
}

// shell32 exported data (cdecl-decorated globals, no @N)
extern "C" {
    static IID_IExtractIconW: IID;
    static IID_IShellDetails: IID;
    static IID_IShellFolder: IID;
    static IID_IShellFolder2: IID;
    static IID_IUnknown: IID;
    static shell32_hInstance: HINSTANCE;
    static swShell32Name: WCHAR; // WCHAR swShell32Name[MAX_PATH] — address of first element
    static ShellLargeIconSize: INT;
    static ShellSmallIconSize: INT;
}

// msvcrt — swprintf is the legacy variadic form ReactOS shell32 uses (matches shellole.rs).
extern "C" {
    fn swprintf(buffer: *mut WCHAR, format: *const WCHAR, ...) -> INT;
}

// ── tiny in-file helpers (avoid the __imp_ libc thunks) ─────────────────────────────────
#[inline]
fn SUCCEEDED(hr: HRESULT) -> bool {
    hr >= 0
}
#[inline]
fn swshell32name_ptr() -> *const WCHAR {
    // SAFETY: `swShell32Name` is a WCHAR[MAX_PATH] export; take the address of its first elem.
    unsafe { &swShell32Name as *const WCHAR }
}
/// msvcrt `isspace` semantics for the ASCII control set (command lines are ASCII).
#[inline]
fn r_isspace(c: WCHAR) -> bool {
    matches!(c, 0x20 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D)
}
/// msvcrt `isblank` — space and tab.
#[inline]
fn r_isblank(c: WCHAR) -> bool {
    c == 0x20 || c == 0x09
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
#[inline]
unsafe fn r_wcscpy(dst: *mut WCHAR, mut src: *const WCHAR) {
    let mut d = dst;
    loop {
        let c = *src;
        *d = c;
        if c == 0 {
            break;
        }
        d = d.add(1);
        src = src.add(1);
    }
}
/// MAKEINTRESOURCEW(i) = (LPWSTR)(ULONG_PTR)(WORD)i
#[inline]
fn makeintresourcew(i: WORD) -> LPCWSTR {
    i as usize as LPCWSTR
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  CommandLineToArgvW
// ════════════════════════════════════════════════════════════════════════════════════════
#[no_mangle]
pub unsafe extern "system" fn CommandLineToArgvW(lpCmdline: LPCWSTR, numargs: *mut INT) -> *mut LPWSTR {
    let mut argc: DWORD;
    let argv: *mut LPWSTR;
    let mut s: *const WCHAR;
    let mut d: *mut WCHAR;
    let cmdline: *mut WCHAR;
    let mut qcount: INT;
    let mut bcount: INT;

    if numargs.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return null_mut();
    }

    if *lpCmdline == 0 {
        // Return the path to the executable
        let mut deslen: DWORD = MAX_PATH as DWORD;
        let mut size: SIZE_T = size_of::<LPWSTR>() * 2 + deslen as usize * size_of::<WCHAR>();
        loop {
            let p = LocalAlloc(LMEM_FIXED, size) as *mut LPWSTR;
            if p.is_null() {
                return null_mut();
            }
            let len = GetModuleFileNameW(null_mut(), p.add(2) as LPWSTR, deslen);
            if len == 0 {
                LocalFree(p as HLOCAL);
                return null_mut();
            }
            if len < deslen {
                write_volatile(p, p.add(2) as LPWSTR);
                write_volatile(p.add(1), null_mut());
                write_volatile(numargs, 1);
                return p;
            }
            deslen *= 2;
            size = size_of::<LPWSTR>() * 2 + deslen as usize * size_of::<WCHAR>();
            LocalFree(p as HLOCAL);
        }
    }

    // --- First count the arguments
    argc = 1;
    s = lpCmdline;
    // The first argument, the executable path, follows special rules
    if *s == b'"' as WCHAR {
        s = s.add(1);
        while *s != 0 {
            let c = *s;
            s = s.add(1);
            if c == b'"' as WCHAR {
                break;
            }
        }
    } else {
        while *s != 0 && !r_isspace(*s) {
            s = s.add(1);
        }
    }
    // skip to the first argument, if any
    while r_isblank(*s) {
        s = s.add(1);
    }
    if *s != 0 {
        argc += 1;
    }

    // Analyze the remaining arguments
    qcount = 0;
    bcount = 0;
    while *s != 0 {
        if r_isblank(*s) && qcount == 0 {
            while r_isblank(*s) {
                s = s.add(1);
            }
            if *s != 0 {
                argc += 1;
            }
            bcount = 0;
        } else if *s == b'\\' as WCHAR {
            bcount += 1;
            s = s.add(1);
        } else if *s == b'"' as WCHAR {
            if (bcount & 1) == 0 {
                qcount += 1;
            }
            s = s.add(1);
            bcount = 0;
            while *s == b'"' as WCHAR {
                qcount += 1;
                s = s.add(1);
            }
            qcount %= 3;
            if qcount == 2 {
                qcount = 0;
            }
        } else {
            bcount = 0;
            s = s.add(1);
        }
    }

    // Allocate in a single lump.
    argv = LocalAlloc(
        LMEM_FIXED,
        (argc as usize + 1) * size_of::<LPWSTR>() + (r_wcslen(lpCmdline) + 1) * size_of::<WCHAR>(),
    ) as *mut LPWSTR;
    if argv.is_null() {
        return null_mut();
    }
    cmdline = argv.add(argc as usize + 1) as *mut WCHAR;
    r_wcscpy(cmdline, lpCmdline);

    // --- Then split and copy the arguments
    d = cmdline;
    write_volatile(argv, d);
    argc = 1;
    // The first argument, the executable path, follows special rules
    if *d == b'"' as WCHAR {
        s = d.add(1);
        while *s != 0 {
            if *s == b'"' as WCHAR {
                s = s.add(1);
                break;
            }
            *d = *s;
            d = d.add(1);
            s = s.add(1);
        }
    } else {
        while *d != 0 && !r_isspace(*d) {
            d = d.add(1);
        }
        s = d;
        if *s != 0 {
            s = s.add(1);
        }
    }
    // close the executable path
    *d = 0;
    d = d.add(1);
    // skip to the first argument and initialize it if any
    while r_isblank(*s) {
        s = s.add(1);
    }

    if *s == 0 {
        write_volatile(argv.add(argc as usize), null_mut());
        write_volatile(numargs, argc as INT);
        return argv;
    }

    // Split and copy the remaining arguments
    write_volatile(argv.add(argc as usize), d);
    argc += 1;
    qcount = 0;
    bcount = 0;
    while *s != 0 {
        if r_isblank(*s) && qcount == 0 {
            // close the argument
            *d = 0;
            d = d.add(1);
            bcount = 0;
            // skip to the next one and initialize it if any
            loop {
                s = s.add(1);
                if !r_isblank(*s) {
                    break;
                }
            }
            if *s != 0 {
                write_volatile(argv.add(argc as usize), d);
                argc += 1;
            }
        } else if *s == b'\\' as WCHAR {
            *d = *s;
            d = d.add(1);
            s = s.add(1);
            bcount += 1;
        } else if *s == b'"' as WCHAR {
            if (bcount & 1) == 0 {
                // even number of '\': half of them, plus a quote we erase
                d = d.offset(-((bcount / 2) as isize));
                qcount += 1;
            } else {
                // odd number of '\': half of them followed by a literal '"'
                d = d.offset(-((bcount / 2) as isize) - 1);
                *d = b'"' as WCHAR;
                d = d.add(1);
            }
            s = s.add(1);
            bcount = 0;
            // count consecutive quotes (qcount already counts the opening quote)
            while *s == b'"' as WCHAR {
                qcount += 1;
                if qcount == 3 {
                    *d = b'"' as WCHAR;
                    d = d.add(1);
                    qcount = 0;
                }
                s = s.add(1);
            }
            if qcount == 2 {
                qcount = 0;
            }
        } else {
            *d = *s;
            d = d.add(1);
            s = s.add(1);
            bcount = 0;
        }
    }
    *d = 0; // '\0'
    write_volatile(argv.add(argc as usize), null_mut());
    write_volatile(numargs, argc as INT);

    argv
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  SHELL_GetDetailsOfToBuffer (internal)
// ════════════════════════════════════════════════════════════════════════════════════════
unsafe fn SHELL_GetDetailsOfToBuffer(
    psf: *mut IShellFolder,
    pidl: LPCITEMIDLIST,
    col: UINT,
    Buf: LPWSTR,
    cchBuf: UINT,
) -> HRESULT {
    let mut psf2: *mut IShellFolder2 = null_mut();
    let mut psd: *mut IShellDetails = null_mut();
    let mut details: SHELLDETAILS = core::mem::zeroed();

    let mut hr = ((*(*psf).lpVtbl).QueryInterface)(
        psf,
        &IID_IShellFolder2,
        &mut psf2 as *mut *mut IShellFolder2 as *mut LPVOID,
    );
    if SUCCEEDED(hr) {
        hr = ((*(*psf2).lpVtbl).GetDetailsOf)(psf2, pidl, col, &mut details);
        ((*(*psf2).lpVtbl).Release)(psf2);
    } else {
        hr = ((*(*psf).lpVtbl).QueryInterface)(
            psf,
            &IID_IShellDetails,
            &mut psd as *mut *mut IShellDetails as *mut LPVOID,
        );
        if SUCCEEDED(hr) {
            hr = ((*(*psd).lpVtbl).GetDetailsOf)(psd, pidl, col, &mut details);
            ((*(*psd).lpVtbl).Release)(psd);
        }
    }
    if SUCCEEDED(hr) {
        hr = if StrRetToStrNW(Buf, cchBuf, &mut details.str_, pidl) != 0 {
            S_OK_
        } else {
            E_FAIL
        };
    }
    hr
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  shgfi_get_exe_type (internal)
// ════════════════════════════════════════════════════════════════════════════════════════
unsafe fn shgfi_get_exe_type(szFullPath: LPCWSTR) -> DWORD {
    let mut BinaryType: DWORD = 0;
    let mut mz_header: IMAGE_DOS_HEADER = core::mem::zeroed();
    let mut nt: IMAGE_NT_HEADERS = core::mem::zeroed();
    let mut len: DWORD = 0;
    let mut magic: [CHAR; 4] = [0; 4];

    let status = GetBinaryTypeW(szFullPath, &mut BinaryType);
    if status == FALSE {
        return 0;
    }
    if BinaryType == SCS_DOS_BINARY || BinaryType == SCS_PIF_BINARY {
        return 0x4d5a;
    }

    let hfile = CreateFileW(
        szFullPath,
        GENERIC_READ,
        FILE_SHARE_READ,
        null_mut(),
        OPEN_EXISTING,
        0,
        null_mut(),
    );
    if hfile as usize == usize::MAX {
        // INVALID_HANDLE_VALUE
        return 0;
    }

    SetFilePointer(hfile, 0, null_mut(), SEEK_SET);
    ReadFile(
        hfile,
        &mut mz_header as *mut IMAGE_DOS_HEADER as LPVOID,
        size_of::<IMAGE_DOS_HEADER>() as DWORD,
        &mut len,
        null_mut(),
    );

    SetFilePointer(hfile, mz_header.e_lfanew, null_mut(), SEEK_SET);
    ReadFile(
        hfile,
        magic.as_mut_ptr() as LPVOID,
        size_of::<[CHAR; 4]>() as DWORD,
        &mut len,
        null_mut(),
    );
    let magic_dword = (magic.as_ptr() as *const DWORD).read_unaligned();
    let magic_word = (magic.as_ptr() as *const WORD).read_unaligned();
    if magic_dword == IMAGE_NT_SIGNATURE {
        SetFilePointer(hfile, mz_header.e_lfanew, null_mut(), SEEK_SET);
        ReadFile(
            hfile,
            &mut nt as *mut IMAGE_NT_HEADERS as LPVOID,
            size_of::<IMAGE_NT_HEADERS>() as DWORD,
            &mut len,
            null_mut(),
        );
        CloseHandle(hfile);
        // DLL files are not executable and should return 0
        if (nt.FileHeader.Characteristics & IMAGE_FILE_DLL) != 0 {
            return 0;
        }
        if nt.OptionalHeader.Subsystem == IMAGE_SUBSYSTEM_WINDOWS_GUI {
            return IMAGE_NT_SIGNATURE
                | ((nt.OptionalHeader.MajorSubsystemVersion as DWORD) << 24)
                | ((nt.OptionalHeader.MinorSubsystemVersion as DWORD) << 16);
        }
        return IMAGE_NT_SIGNATURE;
    } else if magic_word == IMAGE_OS2_SIGNATURE {
        let mut ne: IMAGE_OS2_HEADER = core::mem::zeroed();
        SetFilePointer(hfile, mz_header.e_lfanew, null_mut(), SEEK_SET);
        ReadFile(
            hfile,
            &mut ne as *mut IMAGE_OS2_HEADER as LPVOID,
            size_of::<IMAGE_OS2_HEADER>() as DWORD,
            &mut len,
            null_mut(),
        );
        CloseHandle(hfile);
        if ne.ne_exetyp == 2 {
            return (IMAGE_OS2_SIGNATURE as DWORD) | ((ne.ne_expver as DWORD) << 16);
        }
        return 0;
    }
    CloseHandle(hfile);
    0
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  SHELL_IsShortcut (internal, cdecl extern "C")
// ════════════════════════════════════════════════════════════════════════════════════════
#[no_mangle]
pub unsafe extern "C" fn SHELL_IsShortcut(pidlLast: LPCITEMIDLIST) -> BOOL {
    let mut szTemp: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut keyCls: HKEY = null_mut();
    let mut ret: BOOL = FALSE;

    if _ILGetExtension(pidlLast, szTemp.as_mut_ptr(), szTemp.len() as UINT) != FALSE
        && SUCCEEDED(HCR_GetProgIdKeyOfExtension(szTemp.as_ptr(), &mut keyCls, FALSE))
    {
        ret = (RegQueryValueExW(
            keyCls,
            W_IS_SHORTCUT.as_ptr(),
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
        ) == ERROR_SUCCESS) as BOOL;
        RegCloseKey(keyCls);
    }
    ret
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  SHGetFileInfoW
// ════════════════════════════════════════════════════════════════════════════════════════
#[no_mangle]
pub unsafe extern "system" fn SHGetFileInfoW(
    path: LPCWSTR,
    mut dwFileAttributes: DWORD,
    psfi: *mut SHFILEINFOW,
    _sizeofpsfi: UINT,
    mut flags: UINT,
) -> DWORD_PTR {
    let mut szLocation: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut szFullPath: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut iIndex: INT = 0;
    let mut ret: DWORD_PTR = TRUE as DWORD_PTR;
    let mut dwAttributes: DWORD = 0;
    let mut psfParent: *mut IShellFolder = null_mut();
    let mut pei: *mut IExtractIconW = null_mut();
    let mut pidlLast: LPCITEMIDLIST = null();
    let mut pidl: LPITEMIDLIST = null_mut();
    let mut pidlFree: LPITEMIDLIST = null_mut();
    let mut hr: HRESULT = S_OK_;
    let mut IconNotYetLoaded: BOOL = TRUE;
    let mut uGilFlags: UINT = 0;
    let mut big_icons: HIMAGELIST = null_mut();
    let mut small_icons: HIMAGELIST = null_mut();

    if path.is_null() {
        return FALSE as DWORD_PTR;
    }

    // windows initializes these values regardless of the flags
    if !psfi.is_null() {
        write_volatile((*psfi).szDisplayName.as_mut_ptr(), 0);
        write_volatile((*psfi).szTypeName.as_mut_ptr(), 0);
        write_volatile(&mut (*psfi).hIcon, null_mut());
    }

    if (flags & SHGFI_PIDL) == 0 {
        // works with absolute and relative paths
        if PathIsRelativeW(path) != FALSE {
            GetCurrentDirectoryW(MAX_PATH as DWORD, szLocation.as_mut_ptr());
            PathCombineW(szFullPath.as_mut_ptr(), szLocation.as_ptr(), path);
        } else {
            lstrcpynW(szFullPath.as_mut_ptr(), path, MAX_PATH as INT);
        }

        if (flags & SHGFI_TYPENAME) != 0 && PathIsRootW(szFullPath.as_ptr()) == FALSE {
            if (flags & SHGFI_USEFILEATTRIBUTES) == 0 {
                dwFileAttributes = GetFileAttributesW(szFullPath.as_ptr());
                if dwFileAttributes == INVALID_FILE_ATTRIBUTES {
                    dwFileAttributes = 0;
                }
            }
            let hr2: HRESULT = if (dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                SHELL32_AssocGetFSDirectoryDescription(
                    (*psfi).szTypeName.as_mut_ptr(),
                    (*psfi).szTypeName.len() as UINT,
                )
            } else {
                SHELL32_AssocGetFileDescription(
                    path,
                    (*psfi).szTypeName.as_mut_ptr(),
                    (*psfi).szTypeName.len() as UINT,
                )
            };
            if SUCCEEDED(hr2) {
                flags &= !SHGFI_TYPENAME;
                if (flags & !SHGFI_USEFILEATTRIBUTES) == 0 {
                    return ret; // bail out early
                }
            }
        }
    } else {
        SHGetPathFromIDListW(path as LPCITEMIDLIST, szFullPath.as_mut_ptr());
    }

    if (flags & SHGFI_EXETYPE) != 0 && (flags & SHGFI_SYSICONINDEX) == 0 {
        if (flags & SHGFI_USEFILEATTRIBUTES) != 0 {
            return TRUE as DWORD_PTR;
        } else if GetFileAttributesW(szFullPath.as_ptr()) != INVALID_FILE_ATTRIBUTES {
            return shgfi_get_exe_type(szFullPath.as_ptr()) as DWORD_PTR;
        }
    }

    // psfi is NULL normally to query EXE type
    if psfi.is_null() {
        return FALSE as DWORD_PTR;
    }

    if (flags & SHGFI_PIDL) != 0 {
        pidl = path as LPITEMIDLIST;
        hr = if !pidl.is_null() { S_OK_ } else { E_FAIL };
    } else {
        if (flags & SHGFI_USEFILEATTRIBUTES) != 0 {
            pidl = SHELL32_CreateSimpleIDListFromPath(szFullPath.as_ptr(), dwFileAttributes);
            hr = if !pidl.is_null() { S_OK_ } else { E_FAIL };
        } else {
            hr = SHILCreateFromPathW(szFullPath.as_ptr(), &mut pidl, &mut dwAttributes);
        }
        pidlFree = pidl;
    }

    if SUCCEEDED(hr) {
        hr = SHBindToParent(
            pidl,
            &IID_IShellFolder,
            &mut psfParent as *mut *mut IShellFolder as *mut LPVOID,
            &mut pidlLast,
        );
    }

    // attributes of the child
    if SUCCEEDED(hr) && (flags & SHGFI_ATTRIBUTES) != 0 {
        if (flags & SHGFI_ATTR_SPECIFIED) == 0 {
            write_volatile(&mut (*psfi).dwAttributes, 0xffffffff);
        }
        hr = ((*(*psfParent).lpVtbl).GetAttributesOf)(
            psfParent,
            1,
            &pidlLast as *const LPCITEMIDLIST,
            &mut (*psfi).dwAttributes,
        );
    }

    if (flags & SHGFI_USEFILEATTRIBUTES) != 0 && (flags & SHGFI_ICON) != 0 {
        write_volatile(&mut (*psfi).dwAttributes, 0);
    }

    // displayname
    if SUCCEEDED(hr) && (flags & SHGFI_DISPLAYNAME) != 0 {
        let mut str_: STRRET = core::mem::zeroed();
        write_volatile((*psfi).szDisplayName.as_mut_ptr(), UNICODE_NULL);
        hr = ((*(*psfParent).lpVtbl).GetDisplayNameOf)(
            psfParent,
            pidlLast,
            SHGDN_INFOLDER,
            &mut str_,
        );
        if SUCCEEDED(hr) {
            StrRetToStrNW(
                (*psfi).szDisplayName.as_mut_ptr(),
                (*psfi).szDisplayName.len() as UINT,
                &mut str_,
                pidlLast,
            );
        }
    }

    // type name
    if SUCCEEDED(hr) && (flags & SHGFI_TYPENAME) != 0 {
        let col: UINT = if _ILIsDrive(pidlLast) != FALSE { 1 } else { 2 }; // SHFSF_COL_TYPE
        write_volatile((*psfi).szTypeName.as_mut_ptr(), UNICODE_NULL);
        hr = SHELL_GetDetailsOfToBuffer(
            psfParent,
            pidlLast,
            col,
            (*psfi).szTypeName.as_mut_ptr(),
            (*psfi).szTypeName.len() as UINT,
        );
    }

    // ### icons ###
    Shell_GetImageLists(&mut big_icons, &mut small_icons);

    if (flags & SHGFI_OPENICON) != 0 {
        uGilFlags |= GIL_OPENICON;
    }

    if (flags & SHGFI_LINKOVERLAY) != 0 {
        uGilFlags |= GIL_FORSHORTCUT;
    } else if (flags & SHGFI_ADDOVERLAYS) != 0
        || (flags & (SHGFI_ICON | SHGFI_SMALLICON)) == SHGFI_ICON
    {
        if SHELL_IsShortcut(pidlLast) != FALSE {
            uGilFlags |= GIL_FORSHORTCUT;
        }
    }

    // SHGFI_OVERLAYINDEX / SHGFI_SELECTED — FIXME no-ops in the C

    // iconlocation
    if SUCCEEDED(hr) && (flags & SHGFI_ICONLOCATION) != 0 {
        let mut uDummy: UINT = 0;
        let mut uFlags: UINT = 0;

        if (flags & SHGFI_USEFILEATTRIBUTES) != 0 && (flags & SHGFI_PIDL) == 0 {
            if (dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                lstrcpyW((*psfi).szDisplayName.as_mut_ptr(), swshell32name_ptr());
                write_volatile(&mut (*psfi).iIcon, -IDI_SHELL_FOLDER);
            } else {
                let mut sTemp: [WCHAR; MAX_PATH] = [0; MAX_PATH];
                let szExt = PathFindExtensionW(szFullPath.as_ptr());
                if !szExt.is_null()
                    && HCR_MapTypeToValueW(szExt, sTemp.as_mut_ptr(), MAX_PATH as LONG, TRUE)
                        != FALSE
                    && HCR_GetIconW(
                        sTemp.as_ptr(),
                        sTemp.as_mut_ptr(),
                        null(),
                        MAX_PATH as DWORD,
                        &mut (*psfi).iIcon,
                    ) != FALSE
                {
                    if lstrcmpW(W_PERCENT_1.as_ptr(), sTemp.as_ptr()) != 0 {
                        r_wcscpy((*psfi).szDisplayName.as_mut_ptr(), sTemp.as_ptr());
                    } else {
                        // the icon is in the file
                        r_wcscpy((*psfi).szDisplayName.as_mut_ptr(), szFullPath.as_ptr());
                    }
                } else {
                    ret = FALSE as DWORD_PTR;
                }
            }
        } else if !psfParent.is_null() {
            hr = ((*(*psfParent).lpVtbl).GetUIObjectOf)(
                psfParent,
                null_mut(),
                1,
                &pidlLast as *const LPCITEMIDLIST,
                &IID_IExtractIconW,
                &mut uDummy,
                &mut pei as *mut *mut IExtractIconW as *mut LPVOID,
            );
            if SUCCEEDED(hr) {
                hr = ((*(*pei).lpVtbl).GetIconLocation)(
                    pei,
                    uGilFlags,
                    szLocation.as_mut_ptr(),
                    MAX_PATH as UINT,
                    &mut iIndex,
                    &mut uFlags,
                );
                if (uFlags & GIL_NOTFILENAME) != 0 {
                    ret = FALSE as DWORD_PTR;
                } else {
                    lstrcpyW((*psfi).szDisplayName.as_mut_ptr(), szLocation.as_ptr());
                    write_volatile(&mut (*psfi).iIcon, iIndex);
                }
                ((*(*pei).lpVtbl).Release)(pei);
            }
        }
    }

    // icon index (or load icon)
    if SUCCEEDED(hr) && (flags & (SHGFI_ICON | SHGFI_SYSICONINDEX)) != 0 {
        if (flags & SHGFI_USEFILEATTRIBUTES) != 0 && (flags & SHGFI_PIDL) == 0 {
            let mut sTemp: [WCHAR; MAX_PATH] = [0; MAX_PATH];
            let mut icon_idx: INT = 0;

            lstrcpynW(sTemp.as_mut_ptr(), szFullPath.as_ptr(), MAX_PATH as INT);

            if (dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                write_volatile(
                    &mut (*psfi).iIcon,
                    SIC_GetIconIndex(swshell32name_ptr(), -IDI_SHELL_FOLDER, 0),
                );
            } else {
                write_volatile(&mut (*psfi).iIcon, 0);
                let szExt = PathFindExtensionW(sTemp.as_ptr());
                if !szExt.is_null()
                    && HCR_MapTypeToValueW(szExt, sTemp.as_mut_ptr(), MAX_PATH as LONG, TRUE)
                        != FALSE
                    && HCR_GetIconW(
                        sTemp.as_ptr(),
                        sTemp.as_mut_ptr(),
                        null(),
                        MAX_PATH as DWORD,
                        &mut icon_idx,
                    ) != FALSE
                {
                    if lstrcmpW(W_PERCENT_1.as_ptr(), sTemp.as_ptr()) == 0 {
                        // icon is in the file
                        r_wcscpy(sTemp.as_mut_ptr(), szFullPath.as_ptr());
                    }

                    if (flags & SHGFI_SYSICONINDEX) != 0 {
                        let v = SIC_GetIconIndex(sTemp.as_ptr(), icon_idx, 0);
                        write_volatile(&mut (*psfi).iIcon, v);
                        if (*psfi).iIcon == -1 {
                            write_volatile(&mut (*psfi).iIcon, 0);
                        }
                    } else {
                        let cxIcon: INT;
                        let cyIcon: INT;
                        if (flags & SHGFI_SHELLICONSIZE) != 0 {
                            if (flags & SHGFI_SMALLICON) != 0 {
                                cxIcon = ShellSmallIconSize;
                                cyIcon = ShellSmallIconSize;
                            } else {
                                cxIcon = ShellLargeIconSize;
                                cyIcon = ShellLargeIconSize;
                            }
                        } else if (flags & SHGFI_SMALLICON) != 0 {
                            cxIcon = GetSystemMetrics(SM_CXSMICON);
                            cyIcon = GetSystemMetrics(SM_CYSMICON);
                        } else {
                            cxIcon = GetSystemMetrics(SM_CXICON);
                            cyIcon = GetSystemMetrics(SM_CYICON);
                        }

                        let r = PrivateExtractIconsW(
                            sTemp.as_ptr(),
                            icon_idx,
                            cxIcon,
                            cyIcon,
                            &mut (*psfi).hIcon,
                            null_mut(),
                            1,
                            0,
                        );
                        if r != 0 && r != u32::MAX {
                            IconNotYetLoaded = FALSE;
                            write_volatile(&mut (*psfi).iIcon, icon_idx);
                        }
                    }
                }
            }
        } else if !psfParent.is_null() {
            if PidlToSicIndex(
                psfParent,
                pidlLast,
                ((flags & SHGFI_SMALLICON) == 0) as BOOL,
                uGilFlags,
                &mut (*psfi).iIcon,
            ) == FALSE
            {
                ret = FALSE as DWORD_PTR;
            }
        }
        if ret != 0 && (flags & SHGFI_SYSICONINDEX) != 0 {
            ret = if (flags & SHGFI_SMALLICON) != 0 {
                small_icons as DWORD_PTR
            } else {
                big_icons as DWORD_PTR
            };
        }
    }

    // icon handle
    if SUCCEEDED(hr) && (flags & SHGFI_ICON) != 0 && IconNotYetLoaded != FALSE {
        let h = if (flags & SHGFI_SMALLICON) != 0 {
            ImageList_GetIcon(small_icons, (*psfi).iIcon, ILD_NORMAL)
        } else {
            ImageList_GetIcon(big_icons, (*psfi).iIcon, ILD_NORMAL)
        };
        write_volatile(&mut (*psfi).hIcon, h);
    }

    // (flags & ~SHGFI_KNOWN_FLAGS) — FIXME no-op in the C
    let _ = SHGFI_KNOWN_FLAGS;

    if !psfParent.is_null() {
        ((*(*psfParent).lpVtbl).Release)(psfParent);
    }
    SHFree(pidlFree as LPVOID);

    if hr != S_OK_ {
        ret = FALSE as DWORD_PTR;
    }

    ret
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  SHGetFileInfoA
// ════════════════════════════════════════════════════════════════════════════════════════
#[no_mangle]
pub unsafe extern "system" fn SHGetFileInfoA(
    path: LPCSTR,
    dwFileAttributes: DWORD,
    psfi: *mut SHFILEINFOA,
    _sizeofpsfi: UINT,
    flags: UINT,
) -> DWORD_PTR {
    let mut temppath: LPWSTR = null_mut();
    let pathW: LPCWSTR;
    let ret: DWORD_PTR;
    let mut temppsfi: SHFILEINFOW = core::mem::zeroed();

    if (flags & SHGFI_PIDL) != 0 {
        // path contains a pidl
        pathW = path as LPCWSTR;
    } else {
        let len = MultiByteToWideChar(CP_ACP, 0, path, -1, null_mut(), 0);
        temppath = HeapAlloc(GetProcessHeap(), 0, len as usize * size_of::<WCHAR>()) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, path, -1, temppath, len);
        pathW = temppath;
    }

    if !psfi.is_null() {
        temppsfi.hIcon = (*psfi).hIcon;
        temppsfi.iIcon = (*psfi).iIcon;
        temppsfi.dwAttributes = (*psfi).dwAttributes;

        ret = SHGetFileInfoW(
            pathW,
            dwFileAttributes,
            &mut temppsfi,
            size_of::<SHFILEINFOW>() as UINT,
            flags,
        );
        write_volatile(&mut (*psfi).hIcon, temppsfi.hIcon);
        write_volatile(&mut (*psfi).iIcon, temppsfi.iIcon);
        write_volatile(&mut (*psfi).dwAttributes, temppsfi.dwAttributes);

        WideCharToMultiByte(
            CP_ACP,
            0,
            temppsfi.szDisplayName.as_ptr(),
            -1,
            (*psfi).szDisplayName.as_mut_ptr(),
            size_of::<[CHAR; MAX_PATH]>() as INT,
            null(),
            null_mut(),
        );
        WideCharToMultiByte(
            CP_ACP,
            0,
            temppsfi.szTypeName.as_ptr(),
            -1,
            (*psfi).szTypeName.as_mut_ptr(),
            size_of::<[CHAR; 80]>() as INT,
            null(),
            null_mut(),
        );
    } else {
        ret = SHGetFileInfoW(pathW, dwFileAttributes, null_mut(), 0, flags);
    }

    HeapFree(GetProcessHeap(), 0, temppath as LPVOID);
    ret
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  DuplicateIcon
// ════════════════════════════════════════════════════════════════════════════════════════
#[no_mangle]
pub unsafe extern "system" fn DuplicateIcon(_hInstance: HINSTANCE, hIcon: HICON) -> HICON {
    let mut IconInfo: ICONINFO = core::mem::zeroed();
    let mut hDupIcon: HICON = null_mut();

    if GetIconInfo(hIcon, &mut IconInfo) != FALSE {
        hDupIcon = CreateIconIndirect(&mut IconInfo);
        DeleteObject(IconInfo.hbmMask);
        DeleteObject(IconInfo.hbmColor);
    }
    hDupIcon
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  ExtractIconA / ExtractIconW
// ════════════════════════════════════════════════════════════════════════════════════════
#[no_mangle]
pub unsafe extern "system" fn ExtractIconA(
    hInstance: HINSTANCE,
    lpszFile: LPCSTR,
    nIconIndex: UINT,
) -> HICON {
    let len = MultiByteToWideChar(CP_ACP, 0, lpszFile, -1, null_mut(), 0);
    let lpwstrFile = HeapAlloc(GetProcessHeap(), 0, len as usize * size_of::<WCHAR>()) as LPWSTR;

    MultiByteToWideChar(CP_ACP, 0, lpszFile, -1, lpwstrFile, len);
    let ret = ExtractIconW(hInstance, lpwstrFile, nIconIndex);
    HeapFree(GetProcessHeap(), 0, lpwstrFile as LPVOID);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn ExtractIconW(
    _hInstance: HINSTANCE,
    lpszFile: LPCWSTR,
    nIconIndex: UINT,
) -> HICON {
    let mut hIcon: HICON = null_mut();
    let cx = GetSystemMetrics(SM_CXICON);
    let cy = GetSystemMetrics(SM_CYICON);

    if nIconIndex == u32::MAX {
        let ret = PrivateExtractIconsW(lpszFile, 0, cx, cy, null_mut(), null_mut(), 0, LR_DEFAULTCOLOR);
        if ret != u32::MAX && ret != 0 {
            return ret as UINT_PTR as HICON;
        }
        return null_mut();
    } else {
        let ret = PrivateExtractIconsW(
            lpszFile,
            nIconIndex as INT,
            cx,
            cy,
            &mut hIcon,
            null_mut(),
            1,
            LR_DEFAULTCOLOR,
        );
        if ret == u32::MAX {
            return 1usize as HICON;
        } else if ret > 0 && !hIcon.is_null() {
            return hIcon;
        }
    }
    null_mut()
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  Printer_LoadIconsW
// ════════════════════════════════════════════════════════════════════════════════════════
#[no_mangle]
pub unsafe extern "system" fn Printer_LoadIconsW(
    wsPrinterName: LPCWSTR,
    pLargeIcon: *mut HICON,
    pSmallIcon: *mut HICON,
) {
    let iconindex: INT = IDI_SHELL_PRINTERS_FOLDER;

    // FIXME: select icon by printer name (unimplemented; the C just FIXMEs).
    let _ = wsPrinterName;

    if !pLargeIcon.is_null() {
        let h = LoadImageW(
            shell32_hInstance,
            makeintresourcew(iconindex as WORD),
            IMAGE_ICON,
            0,
            0,
            LR_DEFAULTCOLOR | LR_DEFAULTSIZE,
        );
        write_volatile(pLargeIcon, h as HICON);
    }

    if !pSmallIcon.is_null() {
        let h = LoadImageW(
            shell32_hInstance,
            makeintresourcew(iconindex as WORD),
            IMAGE_ICON,
            16,
            16,
            LR_DEFAULTCOLOR,
        );
        write_volatile(pSmallIcon, h as HICON);
    }
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  Printers_RegisterWindowW / Printers_UnregisterWindow (stubs)
// ════════════════════════════════════════════════════════════════════════════════════════
#[no_mangle]
pub unsafe extern "system" fn Printers_RegisterWindowW(
    _wsPrinter: LPCWSTR,
    _dwType: DWORD,
    _phClassPidl: *mut HANDLE,
    _phwnd: *mut HWND,
) -> BOOL {
    FALSE
}

#[no_mangle]
pub unsafe extern "system" fn Printers_UnregisterWindow(_hClassPidl: HANDLE, _hwnd: HWND) {}

// ════════════════════════════════════════════════════════════════════════════════════════
//  SHHelpShortcuts_RunDLLA / W (stubs)
// ════════════════════════════════════════════════════════════════════════════════════════
#[no_mangle]
pub unsafe extern "system" fn SHHelpShortcuts_RunDLLA(
    _dwArg1: DWORD,
    _dwArg2: DWORD,
    _dwArg3: DWORD,
    _dwArg4: DWORD,
) -> DWORD {
    0
}

#[no_mangle]
pub unsafe extern "system" fn SHHelpShortcuts_RunDLLW(
    _dwArg1: DWORD,
    _dwArg2: DWORD,
    _dwArg3: DWORD,
    _dwArg4: DWORD,
) -> DWORD {
    0
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  SHLoadInProc
// ════════════════════════════════════════════════════════════════════════════════════════
const CLSCTX_INPROC_SERVER: DWORD = 0x00000001; // wtypesbase.h

#[no_mangle]
pub unsafe extern "system" fn SHLoadInProc(rclsid: REFCLSID) -> HRESULT {
    let mut ptr: LPVOID = null_mut();

    CoCreateInstance(
        rclsid,
        null_mut(),
        CLSCTX_INPROC_SERVER,
        &IID_IUnknown,
        &mut ptr,
    );
    if !ptr.is_null() {
        let pUnk = ptr as *mut IUnknown;
        ((*(*pUnk).lpVtbl).Release)(pUnk);
        return S_OK_;
    }
    DISP_E_MEMBERNOTFOUND
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  SetRegTextData (internal)
// ════════════════════════════════════════════════════════════════════════════════════════
unsafe fn SetRegTextData(hWnd: HWND, hKey: HKEY, Value: LPCWSTR, uID: INT) {
    let mut dwBufferSize: DWORD = 0;
    let mut dwType: DWORD = 0;

    if RegQueryValueExW(hKey, Value, null_mut(), &mut dwType, null_mut(), &mut dwBufferSize)
        == ERROR_SUCCESS
        && dwType == REG_SZ
    {
        let lpBuffer = HeapAlloc(GetProcessHeap(), 0, dwBufferSize as usize) as LPWSTR;
        if !lpBuffer.is_null() {
            if RegQueryValueExW(
                hKey,
                Value,
                null_mut(),
                &mut dwType,
                lpBuffer as LPBYTE,
                &mut dwBufferSize,
            ) == ERROR_SUCCESS
            {
                SetDlgItemTextW(hWnd, uID, lpBuffer);
            }
            HeapFree(GetProcessHeap(), 0, lpBuffer as LPVOID);
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  AboutAuthorsDlgProc
// ════════════════════════════════════════════════════════════════════════════════════════
#[no_mangle]
pub unsafe extern "system" fn AboutAuthorsDlgProc(
    hWnd: HWND,
    msg: UINT,
    _wParam: WPARAM,
    _lParam: LPARAM,
) -> INT_PTR {
    if msg == WM_INITDIALOG {
        let mut pstr: *const *const CHAR = SHELL_Authors.0.as_ptr();

        SendDlgItemMessageW(hWnd, IDC_ABOUT_AUTHORS_LISTBOX, WM_SETREDRAW, FALSE as WPARAM, 0);

        while !(*pstr).is_null() {
            let mut name: [WCHAR; 64] = [0; 64];
            // authors list is in utf-8 format
            MultiByteToWideChar(CP_UTF8, 0, *pstr, -1, name.as_mut_ptr(), name.len() as INT);
            SendDlgItemMessageW(
                hWnd,
                IDC_ABOUT_AUTHORS_LISTBOX,
                LB_ADDSTRING,
                usize::MAX, // (WPARAM)-1
                name.as_ptr() as LPARAM,
            );
            pstr = pstr.add(1);
        }

        SendDlgItemMessageW(hWnd, IDC_ABOUT_AUTHORS_LISTBOX, WM_SETREDRAW, TRUE as WPARAM, 0);
        return TRUE as INT_PTR;
    }
    FALSE as INT_PTR
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  AboutDlgProc (internal, __REACTOS__)
// ════════════════════════════════════════════════════════════════════════════════════════
// function-static state (nm: _AboutDlgProc@16.{hWndAuthors,bShowingAuthors})
static mut ABOUT_cxLogoBmp: DWORD = 0;
static mut ABOUT_cyLogoBmp: DWORD = 0;
static mut ABOUT_cyLineBmp: DWORD = 0;
static mut ABOUT_hLogoBmp: HBITMAP = null_mut();
static mut ABOUT_hLineBmp: HBITMAP = null_mut();
static mut ABOUT_hWndAuthors: HWND = null_mut();
static mut ABOUT_bShowingAuthors: BOOL = FALSE;

unsafe extern "system" fn AboutDlgProc(
    hWnd: HWND,
    msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match msg {
        WM_INITDIALOG => {
            let info = lParam as *mut ABOUT_INFO;

            if !info.is_null() {
                let mut hRegKey: HKEY = null_mut();
                let mut MemStat: MEMORYSTATUSEX = core::mem::zeroed();
                let mut szAppTitle: [WCHAR; 512] = [0; 512];
                let mut szAppTitleTemplate: [WCHAR; 512] = [0; 512];
                let mut szAuthorsText: [WCHAR; 20] = [0; 20];

                // Preload the ROS bitmaps — from rosbrand.dll, mapped as a data file, exactly as
                // shell32_main.c:1093 does. Like the C, the module is never freed: the HBITMAPs
                // LoadImage returns are independent GDI objects, and the C leaves the mapping
                // for the life of the process. If rosbrand.dll is missing the bitmaps stay NULL,
                // which the `!is_null()` test below already treats as "no banner".
                let hBrand = LoadLibraryExW(W_ROSBRAND_DLL.as_ptr(), null_mut(), LOAD_LIBRARY_AS_DATAFILE);
                if !hBrand.is_null() {
                    ABOUT_hLogoBmp = LoadImageW(
                        hBrand,
                        makeintresourcew(IDB_BRAND_BANNER),
                        IMAGE_BITMAP,
                        0,
                        0,
                        LR_DEFAULTCOLOR,
                    ) as HBITMAP;
                    ABOUT_hLineBmp = LoadImageW(
                        hBrand,
                        makeintresourcew(IDB_BRAND_BANNERLINE),
                        IMAGE_BITMAP,
                        0,
                        0,
                        LR_DEFAULTCOLOR,
                    ) as HBITMAP;
                }

                if !ABOUT_hLogoBmp.is_null() && !ABOUT_hLineBmp.is_null() {
                    let mut bmpLogo: BITMAP = core::mem::zeroed();

                    GetObjectW(
                        ABOUT_hLogoBmp,
                        size_of::<BITMAP>() as INT,
                        &mut bmpLogo as *mut BITMAP as LPVOID,
                    );
                    ABOUT_cxLogoBmp = bmpLogo.bmWidth as DWORD;
                    ABOUT_cyLogoBmp = bmpLogo.bmHeight as DWORD;

                    GetObjectW(
                        ABOUT_hLineBmp,
                        size_of::<BITMAP>() as INT,
                        &mut bmpLogo as *mut BITMAP as LPVOID,
                    );
                    ABOUT_cyLineBmp = bmpLogo.bmHeight as DWORD;
                }

                // App-specific stuff
                SendDlgItemMessageW(
                    hWnd,
                    IDC_ABOUT_ICON,
                    STM_SETICON,
                    (*info).hIcon as WPARAM,
                    0,
                );

                GetWindowTextW(hWnd, szAppTitleTemplate.as_mut_ptr(), szAppTitleTemplate.len() as INT);
                swprintf(szAppTitle.as_mut_ptr(), szAppTitleTemplate.as_ptr(), (*info).szApp);
                SetWindowTextW(hWnd, szAppTitle.as_ptr());

                SetDlgItemTextW(hWnd, IDC_ABOUT_APPNAME, (*info).szApp);
                SetDlgItemTextW(hWnd, IDC_ABOUT_VERSION, (*info).szOSVersion);
                SetDlgItemTextW(hWnd, IDC_ABOUT_OTHERSTUFF, (*info).szOtherStuff);

                // registered owner/organization
                if RegOpenKeyExW(
                    hkey_local_machine(),
                    W_NT_CURRENTVERSION.as_ptr(),
                    0,
                    KEY_QUERY_VALUE,
                    &mut hRegKey,
                ) == ERROR_SUCCESS
                {
                    SetRegTextData(hWnd, hRegKey, W_REGISTERED_OWNER.as_ptr(), IDC_ABOUT_REG_USERNAME);
                    SetRegTextData(hWnd, hRegKey, W_REGISTERED_ORG.as_ptr(), IDC_ABOUT_REG_ORGNAME);

                    if GetWindowTextLengthW(GetDlgItem(hWnd, IDC_ABOUT_REG_USERNAME)) == 0
                        && GetWindowTextLengthW(GetDlgItem(hWnd, IDC_ABOUT_REG_ORGNAME)) == 0
                    {
                        ShowWindow(GetDlgItem(hWnd, IDC_ABOUT_REG_TO), SW_HIDE);
                    }
                    RegCloseKey(hRegKey);
                }

                // installed physical memory
                MemStat.dwLength = size_of::<MEMORYSTATUSEX>() as DWORD;
                if GlobalMemoryStatusEx(&mut MemStat) != FALSE {
                    let mut szBuf: [WCHAR; 12] = [0; 12];

                    if MemStat.ullTotalPhys > 1024 * 1024 * 1024 {
                        let dTotalPhys: f64;
                        let mut szDecimalSeparator: [WCHAR; 4] = [0; 4];
                        let mut szUnits: [WCHAR; 3] = [0; 3];

                        // GBs or more
                        MemStat.ullTotalPhys /= 1024 * 1024;

                        if MemStat.ullTotalPhys > 1024 * 1024 {
                            // TBs or more
                            MemStat.ullTotalPhys /= 1024;

                            if MemStat.ullTotalPhys > 1024 * 1024 {
                                // PBs or more
                                MemStat.ullTotalPhys /= 1024;
                                dTotalPhys = MemStat.ullTotalPhys as f64 / 1024.0;
                                r_wcscpy(szUnits.as_mut_ptr(), W_PB.as_ptr());
                            } else {
                                dTotalPhys = MemStat.ullTotalPhys as f64 / 1024.0;
                                r_wcscpy(szUnits.as_mut_ptr(), W_TB.as_ptr());
                            }
                        } else {
                            dTotalPhys = MemStat.ullTotalPhys as f64 / 1024.0;
                            r_wcscpy(szUnits.as_mut_ptr(), W_GB.as_ptr());
                        }

                        if GetLocaleInfoW(
                            LOCALE_USER_DEFAULT,
                            LOCALE_SDECIMAL,
                            szDecimalSeparator.as_mut_ptr(),
                            szDecimalSeparator.len() as INT,
                        ) > 0
                        {
                            let uIntegral: UINT = dTotalPhys as UINT;
                            let uDecimals: u8 =
                                ((dTotalPhys * 100.0) as UINT).wrapping_sub(uIntegral * 100) as u8;

                            swprintf(
                                szBuf.as_mut_ptr(),
                                W_FMT_MEM_DEC.as_ptr(),
                                uIntegral,
                                szDecimalSeparator.as_ptr(),
                                uDecimals as UINT,
                                szUnits.as_ptr(),
                            );
                        }
                    } else {
                        // MBs — no decimals
                        swprintf(
                            szBuf.as_mut_ptr(),
                            W_FMT_MEM_MB.as_ptr(),
                            (MemStat.ullTotalPhys / 1024 / 1024) as UINT,
                        );
                    }

                    SetDlgItemTextW(hWnd, IDC_ABOUT_PHYSMEM, szBuf.as_ptr());
                }

                // Authors sub-dialog
                ABOUT_hWndAuthors = CreateDialogParamW(
                    shell32_hInstance,
                    makeintresourcew(IDD_ABOUT_AUTHORS),
                    hWnd,
                    Some(AboutAuthorsDlgProc as DLGPROC),
                    0,
                );
                LoadStringW(
                    shell32_hInstance,
                    IDS_SHELL_ABOUT_AUTHORS,
                    szAuthorsText.as_mut_ptr(),
                    szAuthorsText.len() as INT,
                );
                SetDlgItemTextW(hWnd, IDC_ABOUT_AUTHORS, szAuthorsText.as_ptr());
            }
            TRUE as INT_PTR
        }

        WM_PAINT => {
            if !ABOUT_hLogoBmp.is_null() && !ABOUT_hLineBmp.is_null() {
                let mut ps: PAINTSTRUCT = core::mem::zeroed();
                let hdc = BeginPaint(hWnd, &mut ps);
                let hdcMem = CreateCompatibleDC(hdc);

                if !hdcMem.is_null() {
                    let hOldObj = SelectObject(hdcMem, ABOUT_hLogoBmp);
                    BitBlt(
                        hdc,
                        0,
                        0,
                        ABOUT_cxLogoBmp as INT,
                        ABOUT_cyLogoBmp as INT,
                        hdcMem,
                        0,
                        0,
                        SRCCOPY,
                    );

                    SelectObject(hdcMem, ABOUT_hLineBmp);
                    BitBlt(
                        hdc,
                        0,
                        ABOUT_cyLogoBmp as INT,
                        ABOUT_cxLogoBmp as INT,
                        ABOUT_cyLineBmp as INT,
                        hdcMem,
                        0,
                        0,
                        SRCCOPY,
                    );

                    SelectObject(hdcMem, hOldObj);
                    DeleteDC(hdcMem);
                }
                EndPaint(hWnd, &ps);
            }
            0
        }

        WM_COMMAND => {
            if wParam == IDOK || wParam == IDCANCEL {
                EndDialog(hWnd, TRUE as INT_PTR);
                return TRUE as INT_PTR;
            }
            if wParam == IDC_ABOUT_AUTHORS as WPARAM {
                let mut szAuthorsText: [WCHAR; 20] = [0; 20];
                if ABOUT_bShowingAuthors != FALSE {
                    LoadStringW(
                        shell32_hInstance,
                        IDS_SHELL_ABOUT_AUTHORS,
                        szAuthorsText.as_mut_ptr(),
                        szAuthorsText.len() as INT,
                    );
                    ShowWindow(ABOUT_hWndAuthors, SW_HIDE);
                } else {
                    LoadStringW(
                        shell32_hInstance,
                        IDS_SHELL_ABOUT_BACK,
                        szAuthorsText.as_mut_ptr(),
                        szAuthorsText.len() as INT,
                    );
                    ShowWindow(ABOUT_hWndAuthors, SW_SHOW);
                }
                SetDlgItemTextW(hWnd, IDC_ABOUT_AUTHORS, szAuthorsText.as_ptr());
                ABOUT_bShowingAuthors = (ABOUT_bShowingAuthors == FALSE) as BOOL;
                return TRUE as INT_PTR;
            }
            0
        }

        WM_CLOSE => {
            EndDialog(hWnd, TRUE as INT_PTR);
            0
        }

        _ => 0,
    }
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  ShellAboutA / ShellAboutW
// ════════════════════════════════════════════════════════════════════════════════════════
#[no_mangle]
pub unsafe extern "system" fn ShellAboutA(
    hWnd: HWND,
    szApp: LPCSTR,
    szOtherStuff: LPCSTR,
    hIcon: HICON,
) -> BOOL {
    let mut appW: LPWSTR = null_mut();
    let mut otherW: LPWSTR = null_mut();

    if !szApp.is_null() {
        let len = MultiByteToWideChar(CP_ACP, 0, szApp, -1, null_mut(), 0);
        appW = HeapAlloc(GetProcessHeap(), 0, len as usize * size_of::<WCHAR>()) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, szApp, -1, appW, len);
    }
    if !szOtherStuff.is_null() {
        let len = MultiByteToWideChar(CP_ACP, 0, szOtherStuff, -1, null_mut(), 0);
        otherW = HeapAlloc(GetProcessHeap(), 0, len as usize * size_of::<WCHAR>()) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, szOtherStuff, -1, otherW, len);
    }

    let ret = ShellAboutW(hWnd, appW, otherW, hIcon);

    HeapFree(GetProcessHeap(), 0, otherW as LPVOID);
    HeapFree(GetProcessHeap(), 0, appW as LPVOID);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn ShellAboutW(
    hWnd: HWND,
    szApp: LPCWSTR,
    szOtherStuff: LPCWSTR,
    hIcon: HICON,
) -> BOOL {
    let mut info: ABOUT_INFO = core::mem::zeroed();
    let mut szVersionString: [WCHAR; 256] = [0; 256];
    let mut szFormat: [WCHAR; 256] = [0; 256];

    // Preload the dialog template (DialogBoxIndirectParamW uses the caller's hInstance).
    let hRes = FindResourceW(
        shell32_hInstance,
        makeintresourcew(IDD_ABOUT),
        makeintresourcew(5), // RT_DIALOG
    );
    if hRes.is_null() {
        return FALSE;
    }

    let DlgTemplate = LoadResource(shell32_hInstance, hRes);
    if DlgTemplate.is_null() {
        return FALSE;
    }

    // Output the version OS kernel strings
    LoadStringW(
        shell32_hInstance,
        IDS_ABOUT_VERSION_STRING,
        szFormat.as_mut_ptr(),
        szFormat.len() as INT,
    );
    // shell32_main.c:1348 bounds this with StringCchPrintfW. The translation used the
    // unbounded swprintf and NUL-terminated slot 255 afterwards, which is too late: a
    // translated IDS_ABOUT_VERSION_STRING long enough to overflow had already written past
    // the 256-WCHAR stack buffer.
    strsafe_rust::StringCchPrintfW!(
        szVersionString.as_mut_ptr(),
        szVersionString.len(),
        szFormat.as_ptr(),
        KERNEL_VERSION_STR.as_ptr(),
        KERNEL_VERSION_BUILD_STR.as_ptr(),
    );

    info.szApp = szApp;
    info.szOSVersion = szVersionString.as_ptr();
    info.szOtherStuff = szOtherStuff;
    info.hIcon = if !hIcon.is_null() {
        hIcon
    } else {
        LoadIconW(null_mut(), IDI_WINLOGO as usize as LPCWSTR)
    };

    #[cfg(target_arch = "x86")]
    let hinst = GetWindowLongW(hWnd, GWLP_HINSTANCE) as usize as HINSTANCE;
    #[cfg(not(target_arch = "x86"))]
    let hinst = {
        extern "system" {
            fn GetWindowLongPtrW(hWnd: HWND, nIndex: INT) -> LONG_PTR;
        }
        GetWindowLongPtrW(hWnd, GWLP_HINSTANCE) as usize as HINSTANCE
    };

    DialogBoxIndirectParamW(
        hinst,
        DlgTemplate,
        hWnd,
        Some(AboutDlgProc as DLGPROC),
        &mut info as *mut ABOUT_INFO as LPARAM,
    ) as BOOL
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  FreeIconList / SHLoadNonloadedIconOverlayIdentifiers (stubs)
// ════════════════════════════════════════════════════════════════════════════════════════
#[no_mangle]
pub unsafe extern "system" fn FreeIconList(_dw: DWORD) {}

#[no_mangle]
pub unsafe extern "system" fn SHLoadNonloadedIconOverlayIdentifiers() -> HRESULT {
    S_OK_
}

// ADDED-TO-TYPES?: GUID/IID/CLSID + REFIID/REFCLSID, ULONG/SIZE_T/DWORDLONG, HICON/HBITMAP/HDC/
// HGDIOBJ/HRSRC/HGLOBAL/HMODULE/HIMAGELIST/HLOCAL, WPARAM/LPARAM/LRESULT, ITEMIDLIST/
// LP(C)ITEMIDLIST, SFGAOF/SHGDNF, STRRET (also in shellstring.rs — hoist once), SHFILEINFOA/W,
// RECT, ICONINFO, BITMAP, MEMORYSTATUSEX, PAINTSTRUCT, the PE-header structs (IMAGE_DOS_HEADER/
// IMAGE_FILE_HEADER/IMAGE_OPTIONAL_HEADER{32,64}/IMAGE_NT_HEADERS/IMAGE_OS2_HEADER/
// IMAGE_DATA_DIRECTORY) — all common Win32 layouts. SHELLDETAILS, ABOUT_INFO and the COM
// vtables (IShellFolder/IShellFolder2/IShellDetails/IExtractIconW/IUnknown) are shell/COM-
// specific (keep file-local or move to a shared com module).
