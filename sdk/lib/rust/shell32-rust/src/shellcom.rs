//! shellcom — the shared shell/COM substrate for the shell32-rust cohab. Promoted verbatim
//! from the Wave-4 pidl.rs translation so pidl/shellpath/shellord share ONE definition
//! (killing the cross-file extern-decl clash). Wave 0-3 files keep their file-local copies —
//! retrofitting them is deferred to avoid destabilizing the already-landed waves.
//!
//! THE BOUNDARY, which this file now actually honours: **the bindgen island owns DATA
//! LAYOUTS; shellcom owns VTABLE ABI.** prelude.rs has stated that for a while, but shellcom
//! used to declare its own ITEMIDLIST, GUID, SHELLDETAILS and 30-odd other layouts, so any
//! island-first TU that called a shellcom vtable slot split nominally at every PIDL and every
//! GUID — ABI-identical types that rustc rightly refuses to unify. CDefView, the first module
//! to sit on both sides of the line, found this as ~50 errors that a `.cast()` would have
//! silenced at each site without answering anything. The layouts are now `pub use
//! crate::island::{...}`.
//!
//! What shellcom still OWNS is the vtable shape: the island renders a shobjidl interface as a
//! bindgen C++ class with `Option<fn>` slots, and the 40 landed COM-class modules are written
//! against the C `lpVtbl` idiom with raw fn pointers. One crate cannot carry both — a call
//! through an `Option` slot is E0618, and "just `.unwrap()`" would put a panic on every COM
//! call in a shipping DLL. So the interfaces are blocklisted out of the island and live here.
//!
//! THE PREREQUISITE, recorded because it is not obvious and it rejected two submissions
//! before: this file is `#[path]`-included into BOTH shell32-rust and menuband-rust, so
//! `crate::island` has to resolve in both. menuband-rust therefore path-includes island.rs
//! as well and carries the declaration-only `ntdll-sys` dep the island resolves its field
//! types through. Verify BOTH crates on all three targets after touching this file; only the
//! arm64 gate caught it last time.
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use crate::types::*;
use core::ffi::{c_int, c_void};

// ── scalar aliases not in crate::types ──────────────────────────────────────────────────
pub type ULONG = u32; // winnt.h
pub type USHORT = u16; // winnt.h
pub type SIZE_T = usize; // basetsd.h (ULONG_PTR)
pub type LPARAM = LONG_PTR; // windef.h
pub type SIGDN = u32; // shobjidl (v1_enum)
pub const SIGDN_DESKTOPABSOLUTEPARSING: SIGDN = 0x8002_8000;
// shobjidl.idl:4864-4892 — IShellLibrary's parameter enums, all `[v1_enum]` (32-bit), so `u32`
// like SIGDN. They arrived with the post-fork upstream sync ("[SDK][SHELL32] A big bundle of
// NT6+ shell32 implementations and stubs"); the island was last generated on 2026-08-08,
// BEFORE that sync, so it does not carry them. Declared here rather than by regenerating
// the island mid-flip — see the roadmap entry on the stale shell32 island.
pub type LIBRARYFOLDERFILTER = u32;
pub type LIBRARYOPTIONFLAGS = u32;
pub type DEFAULTSAVEFOLDERTYPE = u32;
pub type LIBRARYSAVEFLAGS = u32;
// shtypes.h:129-140 — `#define REFKNOWNFOLDERID const KNOWNFOLDERID &` under C++ and
// `const KNOWNFOLDERID *` under C: a reference is a pointer on the ABI, and because these are
// `#define`d TYPE MACROS rather than typedefs, bindgen never emits them at all — which is why
// these two are missing independently of the island's staleness.
pub type REFKNOWNFOLDERID = *const crate::island::KNOWNFOLDERID;
pub type REFFOLDERTYPEID = *const crate::island::FOLDERTYPEID;
pub type PWSTR = LPWSTR; // winnt.h
pub type VARTYPE = u16; // wtypes.h


// Active Desktop's current implementation is a pure E_NOTIMPL stub. These SDK
// records cross its COM ABI only by pointer, so opaque repr(C) pointees are exact
// for this class; promote their fields from shlobj.h before any body dereferences
// them.



#[repr(C)]
pub struct IErrorLog {
    pub lpVtbl: *mut core::ffi::c_void,
}

// ── GUID family (guiddef.h) ─────────────────────────────────────────────────────────────
/// GUID/CLSID/IID/FMTID — { Data1: u32; Data2: u16; Data3: u16; Data4: [u8;8] }.
// The GUID family comes from the ISLAND. See the boundary note at the top of this file.
pub use crate::island::{CLSID, GUID, IID};
pub type FMTID = GUID;
pub type REFGUID = *const GUID;
pub type REFIID = *const IID;
pub type REFCLSID = *const CLSID;
pub type REFFMTID = *const FMTID;

// ── PIDL on-disk format (shtypes.h + pidl.h, pshpack1 — ALL packed) ──────────────────────
// From the ISLAND, which is header-derived. This file used to carry its own copies with a
// comment saying not to collapse them, because menuband-rust #[path]-includes this file and
// had no island. That premise is gone: menuband-rust now path-includes island.rs too (and
// gained the declaration-only ntdll-sys dep the island resolves its field types through), so
// both crates that compile shellcom.rs have one. Every definition replaced here was diffed
// against the island's first; the island is identical or STRONGER (PERSIST_FOLDER_TARGET_INFO
// was opaque here and has its real layout there).
pub use crate::island::{
    ITEMIDLIST, LPCITEMIDLIST, LPCSHITEMID, LPITEMIDLIST, PCIDLIST_ABSOLUTE,
    PCUIDLIST_RELATIVE, PCUITEMID_CHILD, PCUITEMID_CHILD_ARRAY, PIDLIST_ABSOLUTE,
    PIDLIST_RELATIVE, PITEMID_CHILD, SHITEMID,
};
pub type LPOLESTR = LPWSTR;

// The rest of the DATA layouts, also from the island. Each was diffed against shellcom's own
// copy before deletion: 26 were byte-for-byte identical (INT vs c_int, [T; 1] vs [T; 1usize]),
// and four -- COMPONENT, COMPONENTSOPT, PERSIST_FOLDER_TARGET_INFO, WALLPAPEROPT -- were
// OPAQUE here and have their real layout there.
//
// STRRET, SHELLDETAILS and PIDLDATA deliberately do NOT move, and the reason is ergonomic
// rather than structural: bindgen renders their anonymous unions as `__bindgen_anon_1` full of
// `__BindgenUnionField`, so `strret.u.pOleStr` would become
// `strret.__bindgen_anon_1.pOleStr.as_ref()` at 40 sites across 12 modules. The layouts agree;
// the boundary exists to make ONE module own a layout, and it is already owned here. Revisit
// if bindgen's union rendering improves.
pub use crate::island::{
    COMPONENT, COMPONENTSOPT, FileStructW, GETPROPERTYSTOREFLAGS, LPCCOMPONENT,
    LPCCOMPONENTSOPT, LPCOMPONENT, LPCOMPONENTSOPT, LPCWALLPAPEROPT, LPSHDRAGIMAGE,
    LPSHELLFOLDER, LPWALLPAPEROPT, PERSIST_FOLDER_TARGET_INFO, PIDLTYPE, PROPERTYKEY,
    SFGAOF, SHCOLSTATEF, SHCOLUMNID, SHCONTF, SHDRAGIMAGE, SHGDNF, SIATTRIBFLAGS,
    SICHINTF, WALLPAPEROPT,
};

pub const SHGDN_NORMAL: SHGDNF = 0;
pub const SHGDN_FORPARSING: SHGDNF = 0x8000;
pub const SFGAO_STORAGE: SFGAOF = 0x0000_0008;
pub const SFGAO_STORAGEANCESTOR: SFGAOF = 0x0080_0000;
pub const SFGAO_VALIDATE: SFGAOF = 0x0100_0000;
pub const SFGAO_FILESYSANCESTOR: SFGAOF = 0x1000_0000;
pub const SFGAO_FOLDER: SFGAOF = 0x2000_0000;
pub const SFGAO_FILESYSTEM: SFGAOF = 0x4000_0000;
pub const SFGAO_HASSUBFOLDER: SFGAOF = 0x8000_0000;
pub const CSIDL_FONTS: INT = 0x0014;
pub const CSIDL_COMMON_ADMINTOOLS: INT = 0x002f;

#[repr(C)]
pub struct IEnumExtraSearch {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct SHELLDETAILS {
    pub fmt: INT,
    pub cxChar: INT,
    pub str_: STRRET,
}


// PT_* type tags (pidl.h:86-121). Pinned exactly; do not guess.
pub const PT_CPLAPPLET: PIDLTYPE = 0x00;
pub const PT_GUID: PIDLTYPE = 0x1F;
pub const PT_DRIVE: PIDLTYPE = 0x23;
pub const PT_DRIVE2: PIDLTYPE = 0x25;
pub const PT_DRIVE3: PIDLTYPE = 0x29;
pub const PT_SHELLEXT: PIDLTYPE = 0x2E;
pub const PT_DRIVE1: PIDLTYPE = 0x2F;
pub const PT_FOLDER1: PIDLTYPE = 0x30;
pub const PT_FOLDER: PIDLTYPE = 0x31;
pub const PT_VALUE: PIDLTYPE = 0x32;
pub const PT_VALUEW: PIDLTYPE = 0x34;
pub const PT_FOLDERW: PIDLTYPE = 0x35;
pub const PT_WORKGRP: PIDLTYPE = 0x41;
pub const PT_COMP: PIDLTYPE = 0x42;
pub const PT_NETPROVIDER: PIDLTYPE = 0x46;
pub const PT_NETWORK: PIDLTYPE = 0x47;
pub const PT_IESPECIAL1: PIDLTYPE = 0x61;
pub const PT_YAGUID: PIDLTYPE = 0x70; // yet another guid
pub const PT_IESPECIAL2: PIDLTYPE = 0xb1;
pub const PT_SHARE: PIDLTYPE = 0xc3;
// __REACTOS__ tags (pidl.h:107-141)
pub const PT_FOLDERTYPEMASK: PIDLTYPE = 0x70;
pub const PT_DESKTOP_REGITEM: PIDLTYPE = 0x1F; // ReactOS pidl.h desktop regitem PIDL type
pub const PT_COMPUTER_REGITEM: PIDLTYPE = 0x2E;
pub const PT_COMPUTER_DRIVE: PIDLTYPE = 0x2F;
pub const PT_FS: PIDLTYPE = 0x30; // Win95 SHSimpleIDListFromPath
pub const PT_FS_FOLDER_FLAG: PIDLTYPE = 0x01;
pub const PT_FS_FILE_FLAG: PIDLTYPE = 0x02;
pub const PT_FS_UNICODE_FLAG: PIDLTYPE = 0x04;
pub const PT_FS_COMMON_FLAG: PIDLTYPE = 0x08;
pub const PT_INTERNET: PIDLTYPE = 0x60;
pub const PT_INTERNET_URL: PIDLTYPE = 0x61;
pub const PT_CONTROLS_OLDREGITEM: PIDLTYPE = 0x70;
pub const PT_CONTROLS_NEWREGITEM: PIDLTYPE = 0x71;

// REGITEMORDER_* sort-order bytes (pidl.h:126-141). Used as the GUID-pidl SortOrder byte.
pub const REGITEMORDER_DEFAULT: BYTE = 0x80;
pub const REGITEMORDER_MYCOMPUTER: BYTE = 0x50;
pub const REGITEMORDER_MYDOCS_DEFAULT: BYTE = 0x48;
pub const REGITEMORDER_NETHOOD: BYTE = 0x58;
pub const REGITEMORDER_RECYCLEBIN: BYTE = 0x60; // REGITEMLOCATION_CONTROLS(=CSIDL_DRIVES) != CSIDL_DESKTOP
pub const REGITEMORDER_INTERNET: BYTE = 0x68;
pub const REGITEMORDER_MYCOMPUTER_CONTROLS: BYTE = 0x1E; // NT5

/// network sub-struct (pidl.h): { WORD dummy; CHAR szNames[1]; } — szNames at PIDLDATA+3.
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct PIDLNetworkStruct {
    pub dummy: WORD,
    pub szNames: [CHAR; 1],
}
/// htmlhelp sub-struct (pidl.h): { WORD dummy; DWORD dummy1; CHAR szName[1]; }
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct PIDLHtmlHelpStruct {
    pub dummy: WORD,
    pub dummy1: DWORD,
    pub szName: [CHAR; 1],
}
// These seven stay LOCAL although the island has them, and it is not an oversight: they are
// the members of `PIDLDATA_u` below, and a Rust union field must be `Copy`. The island's
// bindgen structs do not derive Copy, so re-exporting them turns this union into E0740.
/// GUIDStruct (pidl.h): { BYTE uSortOrder; GUID guid; } — guid at offset 02.
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct GUIDStruct {
    pub uSortOrder: BYTE,
    pub guid: GUID,
}
/// DriveStruct (pidl.h): { CHAR szDriveName[20]; WORD unknown; }
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct DriveStruct {
    pub szDriveName: [CHAR; 20],
    pub unknown: WORD,
}
/// FileStruct (pidl.h): dummy/01 size/02 date/06 time/08 attribs/10 szNames/12 (PIDLDATA-rel).
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct FileStruct {
    pub dummy: BYTE,
    pub dwFileSize: DWORD,
    pub uFileDate: WORD,
    pub uFileTime: WORD,
    pub uFileAttribs: WORD,
    pub szNames: [CHAR; 1],
}
/// PIDLCPanelStruct (pidl.h): dummy/01 iconIdx/02 offsDispName/06 offsComment/08 szName/10.
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct PIDLCPanelStruct {
    pub dummy: BYTE,
    pub iconIdx: DWORD,
    pub offsDispName: WORD,
    pub offsComment: WORD,
    pub szName: [WCHAR; 1],
}
/// ValueWStruct (pidl.h): { WCHAR name[1]; }
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct ValueWStruct {
    pub name: [WCHAR; 1],
}
/// PIDLFontStruct (pidl.h, __REACTOS__).
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct PIDLFontStruct {
    pub dummy: BYTE,
    pub offsFile: WORD,
    pub szName: [WCHAR; 1],
}
/// PIDLPrinterStruct (pidl.h, __REACTOS__).
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct PIDLPrinterStruct {
    pub dummy: BYTE,
    pub Attributes: DWORD,
    pub offsServer: WORD,
    pub szName: [WCHAR; 1],
}

/// PIDLDATA (pidl.h): { PIDLTYPE type; union { … } u; } — type/00, union/01 (packed).
/// Modeled as the type byte + a raw union payload; field access in the bodies is done by
/// PIDLDATA-relative byte offset (read_unaligned) since the whole structure is packed.
#[repr(C, packed)]
pub union PIDLDATA_u {
    pub guid: GUIDStruct,
    pub drive: DriveStruct,
    pub file: FileStruct,
    pub network: PIDLNetworkStruct,
    pub htmlhelp: PIDLHtmlHelpStruct,
    pub cpanel: PIDLCPanelStruct,
    pub valueW: ValueWStruct,
    pub cfont: PIDLFontStruct,
    pub cprinter: PIDLPrinterStruct,
}
#[repr(C, packed)]
pub struct PIDLDATA {
    pub type_: PIDLTYPE,
    pub u: PIDLDATA_u,
}
pub type LPPIDLDATA = *mut PIDLDATA;

// ── STRRET (shtypes.h) ──────────────────────────────────────────────────────────────────
pub const STRRET_WSTR: UINT = 0;
pub const STRRET_OFFSET: UINT = 1;
pub const STRRET_CSTR: UINT = 2;
#[repr(C)]
pub union STRRET_u {
    pub pOleStr: LPWSTR,
    pub uOffset: UINT,
    pub cStr: [CHAR; 260], // MAX_PATH
}
/// STRRET (shtypes.h): { UINT uType; union { LPWSTR pOleStr; UINT uOffset; char cStr[MAX_PATH]; }; }
#[repr(C)]
pub struct STRRET {
    pub uType: UINT,
    pub u: STRRET_u,
}

/// FILETIME (minwinbase.h): { DWORD dwLowDateTime; DWORD dwHighDateTime; }
#[repr(C)]
#[derive(Copy, Clone)]
pub struct FILETIME {
    pub dwLowDateTime: DWORD,
    pub dwHighDateTime: DWORD,
}

// ── VARIANT / SAFEARRAY / SHCOLUMNID (oaidl.idl / shobjidl.idl) ──────────────────────────
// SAFEARRAY: same rule as VARIANT below -- the island owns it. The local copy stopped at
// `pvData` and left the flexible `rgsabound[]` implicit, so it was not even the same size.
pub use crate::island::SAFEARRAY;
// VARIANT: the ISLAND owns this layout. It was a flattened hand-written copy here, which
// made `crate::island::VARIANT` (bindgen's `tagVARIANT`, an anonymous-union nest reached as
// `n1.n2.vt` / `n1.n2.n3.<member>`) a DISTINCT nominal type -- so shlfolder.rs and
// shellutils.rs, which take the island's, could not share one `V_VT!` with CDefView, which
// took this one. The flat copy also carried only four union members, so `VT_INT_PTR`'s
// pointer-width `pintVal` slot did not exist at all.
pub use crate::island::VARIANT;

// ── COM vtables (STDMETHODCALLTYPE == __stdcall on i386 → extern "system") ───────────────
// Only the slots actually invoked carry real signatures; the rest are pointer-sized
// placeholders (`usize`) so the slot offsets stay correct.

#[repr(C)]
pub struct IUnknown {
    pub lpVtbl: *mut IUnknownVtbl,
}
#[repr(C)]
pub struct IUnknownVtbl {
    pub QueryInterface: unsafe extern "system" fn(*mut IUnknown, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IUnknown) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IUnknown) -> ULONG,
}

// NOT here: `IShellExtInit`. Five already-landed files (new_menu, dialogs/drvdefext,
// recycle_bin, shell_link, propsheet) carry their own file-local copy — see this module's
// header on Wave 0-3 keeping theirs — so adding a sixth to this glob-imported module makes
// the name E0659-ambiguous in every one of them. Retrofitting all five is a deliberate
// change, not a side effect of translating one more TU; a new TU declares its own copy.
#[repr(C)]
pub struct IShellFolder {
    pub lpVtbl: *mut IShellFolderVtbl,
}
#[repr(C)]
pub struct IShellFolderVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IShellFolder, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IShellFolder) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IShellFolder) -> ULONG,
    pub ParseDisplayName: unsafe extern "system" fn(
        *mut IShellFolder,
        HWND,
        *mut IBindCtx,
        LPWSTR,
        *mut DWORD,
        *mut LPITEMIDLIST,
        *mut DWORD,
    ) -> HRESULT,
    pub EnumObjects: unsafe extern "system" fn(
        *mut IShellFolder,
        HWND,
        DWORD,
        *mut *mut IEnumIDList,
    ) -> HRESULT,
    pub BindToObject: unsafe extern "system" fn(
        *mut IShellFolder,
        LPCITEMIDLIST,
        *mut IBindCtx,
        REFIID,
        *mut *mut c_void,
    ) -> HRESULT,
    pub BindToStorage: unsafe extern "system" fn(
        *mut IShellFolder,
        LPCITEMIDLIST,
        *mut IBindCtx,
        REFIID,
        *mut *mut c_void,
    ) -> HRESULT,
    pub CompareIDs:
        unsafe extern "system" fn(*mut IShellFolder, LPARAM, LPCITEMIDLIST, LPCITEMIDLIST) -> HRESULT,
    pub CreateViewObject:
        unsafe extern "system" fn(*mut IShellFolder, HWND, REFIID, *mut *mut c_void) -> HRESULT,
    pub GetAttributesOf:
        unsafe extern "system" fn(*mut IShellFolder, UINT, *const LPCITEMIDLIST, *mut DWORD) -> HRESULT,
    pub GetUIObjectOf: unsafe extern "system" fn(
        *mut IShellFolder,
        HWND,
        UINT,
        *const LPCITEMIDLIST,
        REFIID,
        *mut UINT,
        *mut *mut c_void,
    ) -> HRESULT,
    pub GetDisplayNameOf:
        unsafe extern "system" fn(*mut IShellFolder, LPCITEMIDLIST, DWORD, *mut STRRET) -> HRESULT,
    pub SetNameOf: unsafe extern "system" fn(
        *mut IShellFolder,
        HWND,
        PCUITEMID_CHILD,
        LPCWSTR,
        SHGDNF,
        *mut PITEMID_CHILD,
    ) -> HRESULT,
}

/// `IContextMenu` in the crate's `lpVtbl` shape. The island renders it as a bindgen C++ class
/// with an opaque `vtable_`, which no translated call site can go through; only the slots with
/// callers here are typed, the rest stay `usize` placeholders at their correct offsets.
#[repr(C)]
pub struct IContextMenu {
    pub lpVtbl: *mut IContextMenuVtbl,
}
#[repr(C)]
pub struct IContextMenuVtbl {
    pub QueryInterface: usize,
    pub AddRef: usize,
    pub Release: unsafe extern "system" fn(*mut IContextMenu) -> ULONG,
    pub QueryContextMenu: unsafe extern "system" fn(
        *mut IContextMenu,
        HMENU,
        UINT,
        UINT,
        UINT,
        UINT,
    ) -> HRESULT,
    pub InvokeCommand: unsafe extern "system" fn(*mut IContextMenu, *mut c_void) -> HRESULT,
    pub GetCommandString: unsafe extern "system" fn(
        *mut IContextMenu,
        UINT_PTR,
        UINT,
        *mut UINT,
        LPSTR,
        UINT,
    ) -> HRESULT,
}

#[repr(C)]
pub struct IShellFolder2 {
    pub lpVtbl: *mut IShellFolder2Vtbl,
}
#[repr(C)]
pub struct IShellFolder2Vtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IShellFolder2, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: usize,
    pub Release: unsafe extern "system" fn(*mut IShellFolder2) -> ULONG,
    pub ParseDisplayName: usize,
    pub EnumObjects: usize,
    pub BindToObject: unsafe extern "system" fn(
        *mut IShellFolder2,
        LPCITEMIDLIST,
        *mut IBindCtx,
        REFIID,
        *mut *mut c_void,
    ) -> HRESULT,
    pub BindToStorage: usize,
    pub CompareIDs: usize,
    pub CreateViewObject: usize,
    pub GetAttributesOf: usize,
    pub GetUIObjectOf: usize,
    pub GetDisplayNameOf: usize,
    pub SetNameOf: usize,
    pub GetDefaultSearchGUID: usize,
    pub EnumSearches: usize,
    pub GetDefaultColumn: unsafe extern "system" fn(
        *mut IShellFolder2,
        DWORD,
        *mut ULONG,
        *mut ULONG,
    ) -> HRESULT,
    pub GetDefaultColumnState:
        unsafe extern "system" fn(*mut IShellFolder2, UINT, *mut SHCOLSTATEF) -> HRESULT,
    pub GetDetailsEx: unsafe extern "system" fn(
        *mut IShellFolder2,
        LPCITEMIDLIST,
        *const SHCOLUMNID,
        *mut VARIANT,
    ) -> HRESULT,
    pub GetDetailsOf: unsafe extern "system" fn(
        *mut IShellFolder2,
        LPCITEMIDLIST,
        UINT,
        *mut SHELLDETAILS,
    ) -> HRESULT,
    pub MapColumnToSCID:
        unsafe extern "system" fn(*mut IShellFolder2, UINT, *mut SHCOLUMNID) -> HRESULT,
}

/// `IShellDetails` (shlobj.h:650, a DECLARE_INTERFACE_ rather than an IDL interface, so
/// it never reaches the island as a `lpVtbl` struct -- the island renders it as a bindgen
/// C++ class with `Option<fn>` slots, which cannot be CALLED). Two methods after IUnknown.
#[repr(C)]
pub struct IShellDetails {
    pub lpVtbl: *mut IShellDetailsVtbl,
}
#[repr(C)]
pub struct IShellDetailsVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IShellDetails, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: usize,
    pub Release: unsafe extern "system" fn(*mut IShellDetails) -> ULONG,
    pub GetDetailsOf: unsafe extern "system" fn(
        *mut IShellDetails,
        PCUITEMID_CHILD,
        UINT,
        *mut SHELLDETAILS,
    ) -> HRESULT,
    pub ColumnClick: unsafe extern "system" fn(*mut IShellDetails, UINT) -> HRESULT,
}

#[repr(C)]
pub struct IEnumIDList {
    pub lpVtbl: *mut IEnumIDListVtbl,
}
#[repr(C)]
pub struct IEnumIDListVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IEnumIDList, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: usize,
    pub Release: unsafe extern "system" fn(*mut IEnumIDList) -> ULONG,
    pub Next:
        unsafe extern "system" fn(*mut IEnumIDList, ULONG, *mut LPITEMIDLIST, *mut ULONG) -> HRESULT,
    pub Skip: unsafe extern "system" fn(*mut IEnumIDList, ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(*mut IEnumIDList) -> HRESULT,
    pub Clone: usize,
}

#[repr(C)]
pub struct IStream {
    pub lpVtbl: *mut IStreamVtbl,
}
#[repr(C)]
pub struct IStreamVtbl {
    pub QueryInterface: usize,
    pub AddRef: unsafe extern "system" fn(*mut IStream) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IStream) -> ULONG,
    pub Read: unsafe extern "system" fn(*mut IStream, *mut c_void, ULONG, *mut ULONG) -> HRESULT,
    pub Write: unsafe extern "system" fn(*mut IStream, *const c_void, ULONG, *mut ULONG) -> HRESULT,
    pub Seek: unsafe extern "system" fn(
        *mut IStream,
        LARGE_INTEGER,
        DWORD,
        *mut ULARGE_INTEGER,
    ) -> HRESULT,
    pub SetSize: usize,
    pub CopyTo: unsafe extern "system" fn(
        *mut IStream,
        *mut IStream,
        ULARGE_INTEGER,
        *mut ULARGE_INTEGER,
        *mut ULARGE_INTEGER,
    ) -> HRESULT,
    pub Commit: usize,
    pub Revert: usize,
    pub LockRegion: usize,
    pub UnlockRegion: usize,
    pub Stat: unsafe extern "system" fn(*mut IStream, *mut STATSTG, DWORD) -> HRESULT,
    pub Clone: usize,
}

#[repr(C)]
pub struct IDataObject {
    pub lpVtbl: *mut IDataObjectVtbl,
}
#[repr(C)]
pub struct IDataObjectVtbl {
    // A `usize` slot and a fn-pointer slot are the same size, so typing one changes the vtable's
    // TYPE and not its LAYOUT. These three were typed for CFSDropTarget, which calls all of them;
    // it is additive because a `usize` slot cannot be called, so no existing caller could have
    // been reading them (checked: nothing in the crate constructs an IDataObjectVtbl literal, and
    // every other consumer passes `*mut IDataObject` around opaquely).
    pub QueryInterface:
        unsafe extern "system" fn(*mut IDataObject, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IDataObject) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IDataObject) -> ULONG,
    pub GetData: unsafe extern "system" fn(*mut IDataObject, *mut c_void, *mut c_void) -> HRESULT,
    pub GetDataHere: usize,
    // `c_void` for the FORMATETC, matching GetData/SetData below for the reason given there.
    pub QueryGetData: unsafe extern "system" fn(*mut IDataObject, *mut c_void) -> HRESULT,
    pub GetCanonicalFormatEtc: usize,
    // Typed for shellutils.h's DataObject_SetData. FORMATETC/STGMEDIUM stay `c_void` to match
    // GetData above, so the one shape of this vtable serves both island and hand-written callers.
    pub SetData:
        unsafe extern "system" fn(*mut IDataObject, *mut c_void, *mut c_void, BOOL) -> HRESULT,
    pub EnumFormatEtc: usize,
    pub DAdvise: usize,
    pub DUnadvise: usize,
    pub EnumDAdvise: usize,
}

/// IDropTarget (oleidl.h). All slots are typed because shell32 passes this interface
/// across the shlwapi boundary and releases it locally.
#[repr(C)]
pub struct IDropTarget {
    pub lpVtbl: *mut IDropTargetVtbl,
}
#[repr(C)]
pub struct IDropTargetVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IDropTarget, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IDropTarget) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IDropTarget) -> ULONG,
    pub DragEnter: unsafe extern "system" fn(
        *mut IDropTarget,
        *mut IDataObject,
        DWORD,
        POINTL,
        *mut DWORD,
    ) -> HRESULT,
    pub DragOver: unsafe extern "system" fn(*mut IDropTarget, DWORD, POINTL, *mut DWORD) -> HRESULT,
    pub DragLeave: unsafe extern "system" fn(*mut IDropTarget) -> HRESULT,
    pub Drop: unsafe extern "system" fn(
        *mut IDropTarget,
        *mut IDataObject,
        DWORD,
        POINTL,
        *mut DWORD,
    ) -> HRESULT,
}
extern "C" {
    pub static IID_IDropTarget: IID;
}

#[repr(C)]
pub struct IPersistFolder2 {
    pub lpVtbl: *mut IPersistFolder2Vtbl,
}
#[repr(C)]
pub struct IPersistFolder2Vtbl {
    pub QueryInterface: usize,
    pub AddRef: usize,
    pub Release: unsafe extern "system" fn(*mut IPersistFolder2) -> ULONG,
    pub GetClassID: usize,
    pub Initialize: usize,
    pub GetCurFolder: unsafe extern "system" fn(*mut IPersistFolder2, *mut LPITEMIDLIST) -> HRESULT,
}


// IPersistFolder / IPersistFolder3 in the `lpVtbl` shape. The island has bindgen's rendering of
// both, but that one puts `Option<fn>` in every slot; keeping the crate on ONE vtable shape is
// what lets a translated `p->Initialize(..)` become a plain dispatch instead of an `.unwrap()`.
// Slot order is the inheritance chain: IUnknown, then IPersist::GetClassID, then each
// derivation's own methods in declaration order.
#[repr(C)]
pub struct IPersistFolder {
    pub lpVtbl: *mut IPersistFolderVtbl,
}
#[repr(C)]
pub struct IPersistFolderVtbl {
    pub QueryInterface: usize,
    pub AddRef: usize,
    pub Release: unsafe extern "system" fn(*mut IPersistFolder) -> ULONG,
    pub GetClassID: usize,
    pub Initialize: unsafe extern "system" fn(*mut IPersistFolder, LPCITEMIDLIST) -> HRESULT,
}

#[repr(C)]
pub struct IPersistFolder3 {
    pub lpVtbl: *mut IPersistFolder3Vtbl,
}
#[repr(C)]
pub struct IPersistFolder3Vtbl {
    pub QueryInterface: usize,
    pub AddRef: usize,
    pub Release: unsafe extern "system" fn(*mut IPersistFolder3) -> ULONG,
    pub GetClassID: usize,
    pub Initialize: usize,
    pub GetCurFolder: usize,
    pub InitializeEx: unsafe extern "system" fn(
        *mut IPersistFolder3,
        *mut IBindCtx,
        LPCITEMIDLIST,
        *const PERSIST_FOLDER_TARGET_INFO,
    ) -> HRESULT,
    pub GetFolderTargetInfo: usize,
}

#[repr(C)]
pub struct IPersistIDList {
    pub lpVtbl: *mut IPersistIDListVtbl,
}
#[repr(C)]
pub struct IPersistIDListVtbl {
    pub QueryInterface: usize,
    pub AddRef: usize,
    pub Release: unsafe extern "system" fn(*mut IPersistIDList) -> ULONG,
    pub GetClassID: usize,
    pub SetIDList: usize,
    pub GetIDList: unsafe extern "system" fn(*mut IPersistIDList, *mut LPITEMIDLIST) -> HRESULT,
}

#[repr(C)]
pub struct IFolderView {
    pub lpVtbl: *mut IFolderViewVtbl,
}
#[repr(C)]
pub struct IFolderViewVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IFolderView, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IFolderView) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IFolderView) -> ULONG,
    pub GetCurrentViewMode:
        unsafe extern "system" fn(*mut IFolderView, *mut UINT) -> HRESULT,
    pub SetCurrentViewMode: unsafe extern "system" fn(*mut IFolderView, UINT) -> HRESULT,
    pub GetFolder:
        unsafe extern "system" fn(*mut IFolderView, REFIID, *mut *mut c_void) -> HRESULT,
    // This vtable used to STOP at GetFolder, and every slot after it was `usize` because the
    // vtable was only ever READ through a `*mut` handed over by C++. That is no longer true:
    // CDefView implements IFolderView and CONSTRUCTS this literal, and a `usize` slot rejects
    // the fn item rather than accepting it silently. So every slot now carries its real
    // signature, transcribed from shobjidl.idl's `interface IFolderView` — the declaration
    // order there IS the ABI, and it is the only authority for it.
    pub Item: unsafe extern "system" fn(*mut IFolderView, c_int, *mut PITEMID_CHILD) -> HRESULT,
    pub ItemCount: unsafe extern "system" fn(*mut IFolderView, UINT, *mut c_int) -> HRESULT,
    pub Items:
        unsafe extern "system" fn(*mut IFolderView, UINT, REFIID, *mut *mut c_void) -> HRESULT,
    pub GetSelectionMarkedItem:
        unsafe extern "system" fn(*mut IFolderView, *mut c_int) -> HRESULT,
    pub GetFocusedItem: unsafe extern "system" fn(*mut IFolderView, *mut c_int) -> HRESULT,
    pub GetItemPosition:
        unsafe extern "system" fn(*mut IFolderView, PCUITEMID_CHILD, *mut POINT) -> HRESULT,
    pub GetSpacing: unsafe extern "system" fn(*mut IFolderView, *mut POINT) -> HRESULT,
    pub GetDefaultSpacing: unsafe extern "system" fn(*mut IFolderView, *mut POINT) -> HRESULT,
    pub GetAutoArrange: unsafe extern "system" fn(*mut IFolderView) -> HRESULT,
    pub SelectItem: unsafe extern "system" fn(*mut IFolderView, c_int, DWORD) -> HRESULT,
    pub SelectAndPositionItems: unsafe extern "system" fn(
        *mut IFolderView,
        UINT,
        PCUITEMID_CHILD_ARRAY,
        *mut POINT,
        DWORD,
    ) -> HRESULT,
}

#[repr(C)]
pub struct IShellItem {
    pub lpVtbl: *mut IShellItemVtbl,
}
#[repr(C)]
pub struct IShellItemVtbl {
    pub QueryInterface: unsafe extern "system" fn(*mut IShellItem, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IShellItem) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IShellItem) -> ULONG,
    pub BindToHandler: unsafe extern "system" fn(*mut IShellItem, *mut IBindCtx, REFGUID, REFIID, *mut *mut c_void) -> HRESULT,
    pub GetParent: unsafe extern "system" fn(*mut IShellItem, *mut *mut IShellItem) -> HRESULT,
    pub GetDisplayName: unsafe extern "system" fn(*mut IShellItem, SIGDN, *mut LPWSTR) -> HRESULT,
    pub GetAttributes: unsafe extern "system" fn(*mut IShellItem, SFGAOF, *mut SFGAOF) -> HRESULT,
    pub Compare: unsafe extern "system" fn(*mut IShellItem, *mut IShellItem, SICHINTF, *mut INT) -> HRESULT,
}

pub type REFPROPERTYKEY = *const PROPERTYKEY;

#[repr(C)]
pub struct IEnumShellItems {
    pub lpVtbl: *const c_void,
}

#[repr(C)]
pub struct IShellItemArray {
    pub lpVtbl: *const IShellItemArrayVtbl,
}
#[repr(C)]
pub struct IShellItemArrayVtbl {
    pub QueryInterface: unsafe extern "system" fn(*mut IShellItemArray, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IShellItemArray) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IShellItemArray) -> ULONG,
    pub BindToHandler: unsafe extern "system" fn(*mut IShellItemArray, *mut IBindCtx, REFGUID, REFIID, *mut *mut c_void) -> HRESULT,
    pub GetPropertyStore: unsafe extern "system" fn(*mut IShellItemArray, GETPROPERTYSTOREFLAGS, REFIID, *mut *mut c_void) -> HRESULT,
    pub GetPropertyDescriptionList: unsafe extern "system" fn(*mut IShellItemArray, REFPROPERTYKEY, REFIID, *mut *mut c_void) -> HRESULT,
    pub GetAttributes: unsafe extern "system" fn(*mut IShellItemArray, SIATTRIBFLAGS, SFGAOF, *mut SFGAOF) -> HRESULT,
    pub GetCount: unsafe extern "system" fn(*mut IShellItemArray, *mut DWORD) -> HRESULT,
    pub GetItemAt: unsafe extern "system" fn(*mut IShellItemArray, DWORD, *mut *mut IShellItem) -> HRESULT,
    pub EnumItems: unsafe extern "system" fn(*mut IShellItemArray, *mut *mut IEnumShellItems) -> HRESULT,
}

extern "C" {
    pub static IID_IShellItem: IID;
    pub static IID_IShellItemArray: IID;
}

#[repr(C)]
pub struct IBindCtx {
    pub lpVtbl: *mut IBindCtxVtbl,
}
#[repr(C)]
pub struct IBindCtxVtbl {
    // THIRTEEN slots, in objidl.idl order. This struct previously stopped after Release, so a
    // caller needing GetObjectParam got "no field" -- which looks nothing like a vtable
    // problem and is why it hid.
    //
    // objidl.idl also declares RemoteSetBindOptions and RemoteGetBindOptions, each marked
    // `[call_as(...)]`. Those are RPC proxy SUBSTITUTES for the local methods, NOT additional
    // slots. Counting them gives 15 and shifts everything after SetBindOptions by two, which
    // compiles perfectly and calls the wrong method forever.
    pub QueryInterface: usize,
    pub AddRef: unsafe extern "system" fn(*mut IBindCtx) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IBindCtx) -> ULONG,
    pub RegisterObjectBound: usize,
    pub RevokeObjectBound: usize,
    pub ReleaseBoundObjects: usize,
    // Typed rather than left a placeholder: IFileSystemBindData_Constructor calls it.
    // Same BIND_OPTS as GetBindOptions below, so the two stay one layout.
    pub SetBindOptions:
        unsafe extern "system" fn(*mut IBindCtx, *mut BIND_OPTS) -> HRESULT,
    pub GetBindOptions:
        unsafe extern "system" fn(*mut IBindCtx, *mut BIND_OPTS) -> HRESULT,
    pub GetRunningObjectTable: usize,
    pub RegisterObjectParam:
        unsafe extern "system" fn(*mut IBindCtx, LPOLESTR, *mut IUnknown) -> HRESULT,
    pub GetObjectParam:
        unsafe extern "system" fn(*mut IBindCtx, LPOLESTR, *mut *mut IUnknown) -> HRESULT,
    pub EnumObjectParam: usize,
    pub RevokeObjectParam: usize,
}
pub type LPBC = *mut IBindCtx;

// ---------------------------------------------------------------------------------------------
// Interfaces the bindgen island renders OPAQUE (no lpVtbl), needed by utils.cpp's translation
// ---------------------------------------------------------------------------------------------
//
// Slot ORDER is the entire ABI contract here and nothing checks it: call sites resolve by NAME,
// so a slot in the wrong position calls the wrong method with the wrong arguments and compiles
// perfectly. Each layout below is the IDL's own order from sdk/include/psdk/shobjidl.idl,
// transcribed method by method, with the three inherited IUnknown slots first.
//
// Following this module's existing convention, a slot nothing calls is a `usize` placeholder:
// it must occupy its position, but giving it a signature no caller uses would be inventing an
// ABI detail rather than recording one.

use crate::types::LARGE_INTEGER;

// These four DUPLICATE definitions that also live in recycle_bin.rs / open_with_menu.rs /
// new_menu.rs, for exactly the reason the PIDL block above gives: this file is `#[path]`-included
// into menuband-rust, which has none of those modules. `use crate::recycle_bin::...` here
// compiles in shell32-rust and breaks menuband-rust with E0432 -- and only the arm64 gate
// surfaces it, so a green local `cargo check -p shell32-rust` on all three targets says nothing.
// I did exactly that and had two submissions rejected before reading the note above.
//
// Layout-identical copies, transcribed from those modules. Do not "collapse" them to a
// `use`; the whole point is that this file depends on `crate::types` and nothing else.
//
// Deliberately NOT `pub`: they exist only to type the vtable slots below. Publishing them
// would collide by glob with the copies in those modules and make the NAME ambiguous in every
// consumer that globs both -- six E0659s the first time I tried it.

// BIND_OPTS / STATSTG come from com-sys through `crate::types`, and WIN32_FIND_DATAW from
// the island. They used to be private copies here, which typed the vtable slots below but
// made every one of those slots UNCALLABLE from another module ("type ... is private"). A
// plain (non-`pub`) import fixes both halves: the type in the signature is public, and the
// name is not re-exported, so no consumer that globs both this module and its own copy gets
// the E0659 the comment above warns about.
use crate::island::WIN32_FIND_DATAW;

// ULARGE_INTEGER: the island owns it, same rule as VARIANT/SAFEARRAY above. The local copy
// here kept only the QuadPart arm, so a body that reached for `.u.HighPart` -- which is what
// utils.cpp's stream copy does -- had nowhere to put it.
use crate::island::ULARGE_INTEGER;



/// `interface IFileSystemBindData : IUnknown` — SetFindData, GetFindData (in that order).
#[repr(C)]
pub struct IFileSystemBindData {
    pub lpVtbl: *const IFileSystemBindDataVtbl,
}

#[repr(C)]
pub struct IFileSystemBindDataVtbl {
    pub QueryInterface: usize,
    pub AddRef: usize,
    pub Release: unsafe extern "system" fn(*mut IFileSystemBindData) -> ULONG,
    pub SetFindData: usize,
    pub GetFindData:
        unsafe extern "system" fn(*mut IFileSystemBindData, *mut WIN32_FIND_DATAW) -> HRESULT,
}

/// `interface IInitializeCommand : IUnknown` — one method, Initialize.
#[repr(C)]
pub struct IInitializeCommand {
    pub lpVtbl: *const IInitializeCommandVtbl,
}

#[repr(C)]
pub struct IInitializeCommandVtbl {
    pub QueryInterface: usize,
    pub AddRef: usize,
    pub Release: unsafe extern "system" fn(*mut IInitializeCommand) -> ULONG,
    /// `HRESULT Initialize([in, string] LPCWSTR pszCommandName, [in, optional] IPropertyBag *ppb)`
    pub Initialize: unsafe extern "system" fn(
        *mut IInitializeCommand,
        LPCWSTR,
        *mut c_void,
    ) -> HRESULT,
}

/// `interface IObjectWithSelection : IUnknown` — SetSelection, GetSelection (in that order).
#[repr(C)]
pub struct IObjectWithSelection {
    pub lpVtbl: *const IObjectWithSelectionVtbl,
}

#[repr(C)]
pub struct IObjectWithSelectionVtbl {
    pub QueryInterface: usize,
    pub AddRef: usize,
    pub Release: unsafe extern "system" fn(*mut IObjectWithSelection) -> ULONG,
    pub SetSelection:
        unsafe extern "system" fn(*mut IObjectWithSelection, *mut IShellItemArray) -> HRESULT,
    pub GetSelection: usize,
}
