//! shellcom — canonical shared shell/COM types for the shell32-rust cohab: the PIDL
//! on-disk format (SHITEMID/ITEMIDLIST/PIDLDATA + PT_* tags), STRRET, the GUID family,
//! and the COM interface vtables. Promoted verbatim from the Wave-4 pidl.rs translation so
//! pidl/shellpath/shellord share ONE definition (killing the cross-file extern-decl clash).
//! Wave 0-3 files keep their file-local copies — retrofitting them is deferred to avoid
//! destabilizing the already-landed waves.
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use crate::types::*;
use core::ffi::c_void;

// ── scalar aliases not in crate::types ──────────────────────────────────────────────────
pub type ULONG = u32; // winnt.h
pub type USHORT = u16; // winnt.h
pub type SIZE_T = usize; // basetsd.h (ULONG_PTR)
pub type LPARAM = LONG_PTR; // windef.h
pub type SFGAOF = DWORD; // shobjidl (ULONG attribute mask)
pub type SHGDNF = DWORD; // shobjidl (SHGDN flags)
pub type SIGDN = u32; // shobjidl (v1_enum)
pub const SIGDN_DESKTOPABSOLUTEPARSING: SIGDN = 0x8002_8000;
pub type SICHINTF = DWORD;
pub type GETPROPERTYSTOREFLAGS = DWORD;
pub type SIATTRIBFLAGS = DWORD;
pub type PWSTR = LPWSTR; // winnt.h
pub type VARTYPE = u16; // wtypes.h
pub type SHCONTF = DWORD;
pub type SHCOLSTATEF = DWORD;

/// SHDRAGIMAGE from shobjidl.h, used by IDragSourceHelper.
#[repr(C)]
pub struct SHDRAGIMAGE {
    pub sizeDragImage: SIZE,
    pub ptOffset: POINT,
    pub hbmpDragImage: HBITMAP,
    pub crColorKey: COLORREF,
}
pub type LPSHDRAGIMAGE = *mut SHDRAGIMAGE;

// Active Desktop's current implementation is a pure E_NOTIMPL stub. These SDK
// records cross its COM ABI only by pointer, so opaque repr(C) pointees are exact
// for this class; promote their fields from shlobj.h before any body dereferences
// them.
#[repr(C)]
pub struct WALLPAPEROPT {
    _opaque: [u8; 0],
}
pub type LPWALLPAPEROPT = *mut WALLPAPEROPT;
pub type LPCWALLPAPEROPT = *const WALLPAPEROPT;

#[repr(C)]
pub struct COMPONENTSOPT {
    _opaque: [u8; 0],
}
pub type LPCOMPONENTSOPT = *mut COMPONENTSOPT;
pub type LPCCOMPONENTSOPT = *const COMPONENTSOPT;

#[repr(C)]
pub struct COMPONENT {
    _opaque: [u8; 0],
}
pub type LPCOMPONENT = *mut COMPONENT;
pub type LPCCOMPONENT = *const COMPONENT;

#[repr(C)]
pub struct IErrorLog {
    pub lpVtbl: *mut core::ffi::c_void,
}

// ── GUID family (guiddef.h) ─────────────────────────────────────────────────────────────
/// GUID/CLSID/IID/FMTID — { Data1: u32; Data2: u16; Data3: u16; Data4: [u8;8] }.
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
pub type REFGUID = *const GUID;
pub type REFIID = *const IID;
pub type REFCLSID = *const CLSID;
pub type REFFMTID = *const FMTID;

// ── PIDL on-disk format (shtypes.h + pidl.h, pshpack1 — ALL packed) ──────────────────────
/// SHITEMID (shtypes.h): { WORD cb; BYTE abID[1]; }
#[repr(C, packed)]
pub struct SHITEMID {
    pub cb: WORD,
    pub abID: [BYTE; 1],
}
/// ITEMIDLIST (shtypes.h): { SHITEMID mkid; }
#[repr(C, packed)]
pub struct ITEMIDLIST {
    pub mkid: SHITEMID,
}
pub type LPITEMIDLIST = *mut ITEMIDLIST;
pub type LPCITEMIDLIST = *const ITEMIDLIST;
pub type PIDLIST_ABSOLUTE = LPITEMIDLIST;
pub type PCIDLIST_ABSOLUTE = LPCITEMIDLIST;
pub type PIDLIST_RELATIVE = LPITEMIDLIST;
pub type PCUIDLIST_RELATIVE = LPCITEMIDLIST;
pub type PCUITEMID_CHILD = LPCITEMIDLIST;
pub type PCUITEMID_CHILD_ARRAY = *const PCUITEMID_CHILD;
pub type PITEMID_CHILD = LPITEMIDLIST;
pub type LPOLESTR = LPWSTR;
pub type LPCSHITEMID = *const SHITEMID;

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

/// PIDLTYPE (pidl.h) — first byte of PIDLDATA, one of the PT_* tags below.
pub type PIDLTYPE = BYTE;

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
/// FileStructW (pidl.h) — appended after FileStruct with 2-byte alignment; a trailing WORD
/// holds the offset (from the PIDL start) to this member. wszName at offset 20.
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct FileStructW {
    pub cbLen: WORD,
    pub dummy1: [BYTE; 6],
    pub uCreationDate: WORD,
    pub uCreationTime: WORD,
    pub uLastAccessDate: WORD,
    pub uLastAccessTime: WORD,
    pub dummy2: [BYTE; 4],
    pub wszName: [WCHAR; 1],
}
/// ValueWStruct (pidl.h): { WCHAR name[1]; }
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct ValueWStruct {
    pub name: [WCHAR; 1],
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
/// SAFEARRAY (oaidl.idl). Only cDims/pvData are read here.
#[repr(C)]
pub struct SAFEARRAY {
    pub cDims: USHORT,
    pub fFeatures: USHORT,
    pub cbElements: ULONG,
    pub cLocks: ULONG,
    pub pvData: PVOID,
    // SAFEARRAYBOUND rgsabound[] follows (flexible)
}
/// VARIANT value union — only vt (offset 0) and parray (offset 8) are touched; the
/// `_brecord`/`_dec` members force the union to 8 bytes on i386 / 16 on amd64-arm64 so
/// sizeof(VARIANT) is 16/24 (matches the SDK, for VariantClear).
#[repr(C)]
pub union VARIANT_value {
    pub iVal: i16,
    pub lVal: i32,
    pub parray: *mut SAFEARRAY,
    pub _brecord: [usize; 2],
}
#[repr(C)]
pub struct VARIANT {
    pub vt: VARTYPE,
    pub wReserved1: WORD,
    pub wReserved2: WORD,
    pub wReserved3: WORD,
    pub val: VARIANT_value,
}
/// SHCOLUMNID (shobjidl.idl): { GUID fmtid; DWORD pid; }
#[repr(C)]
pub struct SHCOLUMNID {
    pub fmtid: GUID,
    pub pid: DWORD,
}

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
    pub SetNameOf: usize,
}
pub type LPSHELLFOLDER = *mut IShellFolder;

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
    pub BindToObject: usize,
    pub BindToStorage: usize,
    pub CompareIDs: usize,
    pub CreateViewObject: usize,
    pub GetAttributesOf: usize,
    pub GetUIObjectOf: usize,
    pub GetDisplayNameOf: usize,
    pub SetNameOf: usize,
    pub GetDefaultSearchGUID: usize,
    pub EnumSearches: usize,
    pub GetDefaultColumn: usize,
    pub GetDefaultColumnState: usize,
    pub GetDetailsEx: unsafe extern "system" fn(
        *mut IShellFolder2,
        LPCITEMIDLIST,
        *const SHCOLUMNID,
        *mut VARIANT,
    ) -> HRESULT,
    pub GetDetailsOf: usize,
    pub MapColumnToSCID: usize,
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
    pub Skip: usize,
    pub Reset: usize,
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
    pub Seek: usize,
    pub SetSize: usize,
    pub CopyTo: usize,
    pub Commit: usize,
    pub Revert: usize,
    pub LockRegion: usize,
    pub UnlockRegion: usize,
    pub Stat: usize,
    pub Clone: usize,
}

#[repr(C)]
pub struct IDataObject {
    pub lpVtbl: *mut IDataObjectVtbl,
}
#[repr(C)]
pub struct IDataObjectVtbl {
    pub QueryInterface: usize,
    pub AddRef: usize,
    pub Release: unsafe extern "system" fn(*mut IDataObject) -> ULONG,
    pub GetData: unsafe extern "system" fn(*mut IDataObject, *mut c_void, *mut c_void) -> HRESULT,
    pub GetDataHere: usize,
    pub QueryGetData: usize,
    pub GetCanonicalFormatEtc: usize,
    pub SetData: usize,
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
    pub QueryInterface: usize,
    pub AddRef: usize,
    pub Release: unsafe extern "system" fn(*mut IFolderView) -> ULONG,
    pub GetCurrentViewMode: usize,
    pub SetCurrentViewMode: usize,
    pub GetFolder:
        unsafe extern "system" fn(*mut IFolderView, REFIID, *mut *mut c_void) -> HRESULT,
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

#[repr(C)]
pub struct PROPERTYKEY {
    pub fmtid: GUID,
    pub pid: DWORD,
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
    pub QueryInterface: usize,
    pub AddRef: usize,
    pub Release: unsafe extern "system" fn(*mut IBindCtx) -> ULONG,
}
pub type LPBC = *mut IBindCtx;
