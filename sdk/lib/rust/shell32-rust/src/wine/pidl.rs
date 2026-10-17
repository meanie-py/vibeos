//! PIDL core — translation of dll/win32/shell32/wine/pidl.c (the foundation file).
//!
//! This is the `__REACTOS__` build configuration
//! (`-D__REACTOS__ -D_SHELL32_ -D_WINE -DUNICODE -D_WIN32_WINNT=0x600`,
//! `NTDDI_VERSION = NTDDI_WIN6 = NTDDI_LONGHORN`). Only the `__REACTOS__` branch of every
//! `#if`/`#ifdef` is translated; the Wine-only branches (`_ILHACKCompareSimpleIds`,
//! `SHGetRealIDL`, `_ILCreateGuidFromStr*`, the non-reactos `ILIsEqual`/`ILFindChild`/
//! `ILIsParent` paths) are NOT compiled and are omitted. Verified against the pre-flip
//! object's symbol table.
//!
//! Decoration: the `IL*`/`SH*` WINAPI exports are `extern "system"` → `_Name@N`; the
//! internal `_IL*` helpers are cdecl `extern "C"` and the leading `_` in the source name
//! plus the i686-gnu cdecl `_` prefix give the `__IL…` symbols seen in nm. `ILGetDisplayNameExW`
//! and `SHELL32_CreateSimpleIDListFromPath` are plain cdecl (`_Name`).
//!
//! Debug machinery (TRACE/WARN/ERR/FIXME, `pdump`, `pcheck`) is dropped: under `__REACTOS__`
//! the only `pcheck` call site (ILLoadFromStream) feeds an empty `if`-body, and `pdump` is
//! pure trace output — neither has observable behavior, so both are omitted (this removes the
//! `_pcheck`/`_pdump` imports the C obj had).

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use crate::types::*;
// The crate's own functions: the definition is the declaration.
use crate::wine::shellole::SHAlloc;
use crate::desktop_folder_tu::SHGetDesktopFolder;
use crate::wine::shellpath::SHGetSpecialFolderLocation;
use crate::wine::shellstring::StrRetToStrNW;
// One declaration per external symbol: island.rs.
use crate::island::CoTaskMemAlloc;
use crate::island::FindFirstFileW;
use crate::island::HeapAlloc;
use crate::island::StrRetToBufW;
use crate::island::StrRetToStrW;
use core::ffi::c_void;
use core::mem::{size_of, zeroed};
use core::ptr::{addr_of, addr_of_mut, null, null_mut, read_unaligned, write_unaligned, write_volatile};

use crate::shellcom::*;

// ════════════════════════════════════════════════════════════════════════════════════════
//  pidl-specific constants, imports, helpers, and the 81 exports
// ════════════════════════════════════════════════════════════════════════════════════════

// ── constants pinned from SDK headers ───────────────────────────────────────────────────
const MAX_PATH: usize = 260; // minwindef.h

const S_FALSE: HRESULT = 1; // winerror.h
const E_FAIL: HRESULT = 0x80004005u32 as HRESULT; // winerror.h
const E_INVALIDARG: HRESULT = 0x80070057u32 as HRESULT; // winerror.h
const E_OUTOFMEMORY: HRESULT = 0x8007000Eu32 as HRESULT; // winerror.h
const E_NOTIMPL: HRESULT = 0x80004001u32 as HRESULT; // winerror.h
const E_NOINTERFACE: HRESULT = 0x80004002u32 as HRESULT; // winerror.h
const NO_ERROR: HRESULT = 0; // winerror.h

const CP_ACP: UINT = 0; // winnls.h
const ERROR_FILE_NOT_FOUND: LONG = 2; // winerror.h
const UNICODE_NULL: WCHAR = 0;

// ILGDN_* (undocshell.h:84-86) — ILGetDisplayNameEx `type` parameter.
const ILGDN_FORPARSING: DWORD = 0;
const ILGDN_NORMAL: DWORD = 1;
const ILGDN_INFOLDER: DWORD = 2;

// SHGDN_* (shobjidl.idl:161-166).
const SHGDN_NORMAL: DWORD = 0;
const SHGDN_INFOLDER: DWORD = 1;
const SHGDN_FORADDRESSBAR: DWORD = 0x4000;
const SHGDN_FORPARSING: DWORD = 0x8000;

// SIGDN values (shobjidl.idl:371-381).
const SIGDN_NORMALDISPLAY: SIGDN = 0;
const SIGDN_PARENTRELATIVEPARSING: SIGDN = 0x80018001;
const SIGDN_DESKTOPABSOLUTEPARSING: SIGDN = 0x80028000;
const SIGDN_PARENTRELATIVEEDITING: SIGDN = 0x80031001;
const SIGDN_DESKTOPABSOLUTEEDITING: SIGDN = 0x8004c000;
const SIGDN_FILESYSPATH: SIGDN = 0x80058000;
const SIGDN_URL: SIGDN = 0x80068000;
const SIGDN_PARENTRELATIVEFORADDRESSBAR: SIGDN = 0x8007c001;
const SIGDN_PARENTRELATIVE: SIGDN = 0x80080001;

const SFGAO_FILESYSTEM: DWORD = 0x40000000; // shobjidl.idl:209
const SHCIDS_CANONICALONLY: DWORD = 0x10000000; // shobjidl.idl:226
const DOGIF_ONLY_IF_ONE: DWORD = 0x8; // shobjidl.idl:602

// SHGetDataFromIDList nFormat (shlobj.h:1457-1459).
const SHGDFIL_FINDDATA: INT = 1;
const SHGDFIL_NETRESOURCE: INT = 2;
const SHGDFIL_DESCRIPTIONID: INT = 3;

const PID_DESCRIPTIONID: DWORD = 2; // shlguid.h:171

const CSIDL_CONTROLS: INT = 0x0003; // shlobj.h:2182
const CSIDL_FLAG_CREATE: INT = 0x8000; // shlobj.h:2241

const ALIAS_ANY: DWORD = 0xFFFF; // shell32_main.h:121

// FILE_ATTRIBUTE_* (winnt.h).
const FILE_ATTRIBUTE_READONLY: DWORD = 0x00000001;
const FILE_ATTRIBUTE_HIDDEN: DWORD = 0x00000002;
const FILE_ATTRIBUTE_SYSTEM: DWORD = 0x00000004;
const FILE_ATTRIBUTE_DIRECTORY: DWORD = 0x00000010;
const FILE_ATTRIBUTE_ARCHIVE: DWORD = 0x00000020;
const FILE_ATTRIBUTE_COMPRESSED: DWORD = 0x00000800;

// VARIANT vt tags (wtypes.idl:729-775).
const VT_EMPTY: VARTYPE = 0;
const VT_UI1: VARTYPE = 17;
const VT_ARRAY: VARTYPE = 0x2000;

// L"Parse Translate Aliases" (shobjidl.idl:232 STR_PARSE_TRANSLATE_ALIASES).
static STR_PARSE_TRANSLATE_ALIASES: [WCHAR; 24] = [
    0x50, 0x61, 0x72, 0x73, 0x65, 0x20, 0x54, 0x72, 0x61, 0x6e, 0x73, 0x6c, 0x61, 0x74, 0x65, 0x20,
    0x41, 0x6c, 0x69, 0x61, 0x73, 0x65, 0x73, 0x00,
];
// "x:\\\0" — _ILCreateDrive template.
static DRIVE_TEMPLATE: [CHAR; 4] = [b'x' as CHAR, b':' as CHAR, b'\\' as CHAR, 0];
// "Entire Network\0" — _ILCreateEntireNetwork.
static ENTIRE_NETWORK: [CHAR; 15] = [
    b'E' as CHAR, b'n' as CHAR, b't' as CHAR, b'i' as CHAR, b'r' as CHAR, b'e' as CHAR,
    b' ' as CHAR, b'N' as CHAR, b'e' as CHAR, b't' as CHAR, b'w' as CHAR, b'o' as CHAR,
    b'r' as CHAR, b'k' as CHAR, 0,
];

// WIN32_FIND_DATA{W,A} (minwinbase.h) — file-local (general win32, not PIDL/COM canonical).
#[repr(C)]
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
#[repr(C)]
struct WIN32_FIND_DATAA {
    dwFileAttributes: DWORD,
    ftCreationTime: FILETIME,
    ftLastAccessTime: FILETIME,
    ftLastWriteTime: FILETIME,
    nFileSizeHigh: DWORD,
    nFileSizeLow: DWORD,
    dwReserved0: DWORD,
    dwReserved1: DWORD,
    cFileName: [CHAR; MAX_PATH],
    cAlternateFileName: [CHAR; 14],
}

/// CIDA / *LPIDA (shlobj.h): { UINT cidl; UINT aoffset[1]; }
#[repr(C)]
pub struct CIDA {
    pub cidl: UINT,
    pub aoffset: [UINT; 1],
}

// ── imports (resolved from shell32's import libs + its own exports at link) ──────────────
extern "system" {
    // ole32
// @4
    fn CoTaskMemFree(pv: LPVOID); // @4
    // kernel32
    fn DosDateTimeToFileTime(fatdate: WORD, fattime: WORD, lpft: *mut FILETIME) -> BOOL; // @12
    fn FileTimeToDosDateTime(lpft: *const FILETIME, lpfat: *mut WORD, lpft2: *mut WORD) -> BOOL; // @12
    fn FindClose(hFindFile: HANDLE) -> BOOL; // @4
// @8
    fn GetProcessHeap() -> HANDLE; // @0
    fn GetVersion() -> DWORD; // @0
// @12
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL; // @12
    fn LocalFree(hMem: HANDLE) -> HANDLE; // @4
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
    fn lstrlenW(lpString: LPCWSTR) -> INT; // @4
    // shlwapi
    fn PathFindExtensionW(pszPath: LPCWSTR) -> LPWSTR; // @4
    fn StrDupW(lpsz: LPCWSTR) -> LPWSTR; // @4
    fn StrFormatKBSizeW(qdw: i64, pszBuf: LPWSTR, cchBuf: UINT) -> LPWSTR; // @16
// @16
// @16
// @12
    // oleaut32
    fn SafeArrayGetLBound(psa: *mut SAFEARRAY, nDim: UINT, plLbound: *mut LONG) -> HRESULT; // @12
    fn SafeArrayGetUBound(psa: *mut SAFEARRAY, nDim: UINT, plUbound: *mut LONG) -> HRESULT; // @12
    fn VariantClear(pvarg: *mut VARIANT) -> HRESULT; // @4
    // shell32 itself (defined in the still-Rust/C siblings)
// @4
    fn SHFree(pv: LPVOID); // @4
// @4
// @12
    fn IFileSystemBindData_Constructor(pfd: *const WIN32_FIND_DATAW, ppV: *mut LPBC) -> HRESULT; // @8
    // comctl32
    fn Alloc(cb: INT) -> LPVOID; // @4
    fn Free(lpv: LPVOID) -> BOOL; // @4
}

// cdecl (EXTERN_C, no WINAPI) — undecorated `_Name` imports.
extern "C" {
    fn BindCtx_GetUIWindow(pBindCtx: *mut IBindCtx) -> HWND;
    fn BindCtx_RegisterObjectParam(
        pBindCtx: *mut IBindCtx,
        pszKey: LPWSTR,
        punk: *mut IUnknown,
        ppbc: *mut LPBC,
    ) -> HRESULT;
    fn HCR_GetClassNameW(riid: REFIID, szDest: LPWSTR, len: DWORD) -> BOOL;
    fn SHELL32_AliasTranslatePidl(pidl: LPCITEMIDLIST, ppidlNew: *mut LPITEMIDLIST, dwFlags: DWORD) -> HRESULT;
    fn SHGetFolderLocationHelper(hwnd: HWND, nFolder: INT, clsid: REFCLSID, ppidl: *mut LPITEMIDLIST) -> HRESULT;
}

// data imports — undecorated `_CLSID_*` / `_IID_*` / `_FMTID_*` statics.
extern "C" {
    static CLSID_AdminFolderShortcut: CLSID;
    static CLSID_ControlPanel: CLSID;
    static CLSID_Internet: CLSID;
    static CLSID_MyComputer: CLSID;
    static CLSID_MyDocuments: CLSID;
    static CLSID_NetworkPlaces: CLSID;
    static CLSID_Printers: CLSID;
    static CLSID_RecycleBin: CLSID;
    static CLSID_ShellDesktop: CLSID;
    static FMTID_ShellDetails: FMTID;
    static IID_IDataObject: IID;
    static IID_IFolderView: IID;
    static IID_IPersistFolder2: IID;
    static IID_IPersistIDList: IID;
    static IID_IShellFolder: IID;
    static IID_IShellFolder2: IID;
    static IID_IShellItem: IID;
}

// ── small in-file libc/helpers (avoid pulling the __imp_ thunks) ─────────────────────────
#[inline]
fn SUCCEEDED(hr: HRESULT) -> bool {
    hr >= 0
}
#[inline]
fn FAILED(hr: HRESULT) -> bool {
    hr < 0
}
#[inline]
unsafe fn rd_u16(p: *const u8) -> u16 {
    read_unaligned(p as *const u16)
}
#[inline]
unsafe fn rd_u32(p: *const u8) -> u32 {
    read_unaligned(p as *const u32)
}
#[inline]
unsafe fn wr_u16(p: *mut u8, v: u16) {
    write_unaligned(p as *mut u16, v)
}
#[inline]
unsafe fn wr_u32(p: *mut u8, v: u32) {
    write_unaligned(p as *mut u32, v)
}
/// pidl->mkid.cb (WORD at offset 0).
#[inline]
unsafe fn il_cb(p: LPCITEMIDLIST) -> WORD {
    rd_u16(p as *const u8)
}
#[inline]
unsafe fn il_set_cb(p: LPITEMIDLIST, v: WORD) {
    wr_u16(p as *mut u8, v)
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
#[inline]
unsafe fn r_strcpy(dst: *mut CHAR, mut src: *const CHAR) -> *mut CHAR {
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
    dst
}
#[inline]
unsafe fn r_memcpy(dst: *mut u8, src: *const u8, n: usize) {
    let mut i = 0usize;
    while i < n {
        *dst.add(i) = *src.add(i);
        i += 1;
    }
}
#[inline]
unsafe fn r_memset(p: *mut u8, val: u8, n: usize) {
    let mut i = 0usize;
    while i < n {
        *p.add(i) = val;
        i += 1;
    }
}
#[inline]
unsafe fn r_memcmp(a: *const u8, b: *const u8, n: usize) -> i32 {
    let mut i = 0usize;
    while i < n {
        let x = *a.add(i);
        let y = *b.add(i);
        if x != y {
            return x as i32 - y as i32;
        }
        i += 1;
    }
    0
}
/// toupper for an ASCII byte value (matches msvcrt for the drive-letter use).
#[inline]
fn r_toupper(c: i32) -> i32 {
    if c >= 'a' as i32 && c <= 'z' as i32 {
        c - 32
    } else {
        c
    }
}
/// SHELL_OsIsUnicode() — shell32_main.h: `!(GetVersion() & 0x80000000)`.
#[inline]
unsafe fn shell_os_is_unicode() -> bool {
    (GetVersion() & 0x80000000) == 0
}
/// IsEqualIID/IsEqualGUID — 16-byte compare.
#[inline]
unsafe fn is_equal_iid(a: *const GUID, b: *const GUID) -> bool {
    r_memcmp(a as *const u8, b as *const u8, size_of::<GUID>()) == 0
}

// ── private inline PIDL helpers (static in the C, not exported) ──────────────────────────
/// _ILUnsafeNext (pidl.c:492) — advance by mkid.cb bytes (no terminator check).
#[inline]
unsafe fn _il_unsafe_next(pidl: LPCITEMIDLIST) -> LPCITEMIDLIST {
    (pidl as *const u8).add(il_cb(pidl) as usize) as LPCITEMIDLIST
}
/// _ILGetType (pidl.h:144) — pidl && cb>=3 ? abID[0] : 0.
#[inline]
unsafe fn _il_get_type(pidl: LPCITEMIDLIST) -> BYTE {
    if !pidl.is_null() && il_cb(pidl) >= 3 {
        *(pidl as *const u8).add(2)
    } else {
        0
    }
}
/// _ILGetFSType / _ILIsFolderOrFile (pidl.h:149) — (type & PT_FOLDERTYPEMASK)==PT_FS ? type : 0.
#[inline]
unsafe fn _il_get_fs_type(pidl: LPCITEMIDLIST) -> BYTE {
    let t = _il_get_type(pidl);
    if (t & PT_FOLDERTYPEMASK) == PT_FS {
        t
    } else {
        0
    }
}
/// _ILIsEmpty (pidl.h:282) == _ILIsDesktop.
#[inline]
unsafe fn _il_is_empty(pidl: LPCITEMIDLIST) -> bool {
    pidl.is_null() || il_cb(pidl) == 0
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  ILGetDisplayName family
// ════════════════════════════════════════════════════════════════════════════════════════
/// ILGetDisplayNameExA (pidl.c:84) — static.
unsafe fn ILGetDisplayNameExA(psf: LPSHELLFOLDER, pidl: LPCITEMIDLIST, path: LPSTR, typ: DWORD) -> BOOL {
    if pidl.is_null() || path.is_null() {
        return FALSE;
    }
    let mut wPath: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    let ret = ILGetDisplayNameExW(psf, pidl, wPath.as_mut_ptr(), typ);
    WideCharToMultiByte(CP_ACP, 0, wPath.as_ptr(), -1, path, MAX_PATH as INT, null(), null_mut());
    ret
}

/// ILGetDisplayNameExW (pidl.c:101) — cdecl export `_ILGetDisplayNameExW`.
#[no_mangle]
pub unsafe extern "C" fn ILGetDisplayNameExW(
    psf: LPSHELLFOLDER,
    pidl: LPCITEMIDLIST,
    path: LPWSTR,
    typ: DWORD,
) -> BOOL {
    let mut lsf: LPSHELLFOLDER = psf;
    let mut ret: HRESULT = NO_ERROR;
    let mut strret: STRRET = zeroed();

    if pidl.is_null() || path.is_null() {
        return FALSE;
    }

    if lsf.is_null() {
        ret = SHGetDesktopFolder(&mut lsf);
        if FAILED(ret) {
            return FALSE;
        }
    }

    let flag: DWORD = match typ {
        ILGDN_FORPARSING => SHGDN_FORPARSING | SHGDN_FORADDRESSBAR,
        ILGDN_NORMAL => SHGDN_NORMAL,
        ILGDN_INFOLDER => SHGDN_INFOLDER,
        _ => SHGDN_FORPARSING | SHGDN_FORADDRESSBAR,
    };

    if il_cb(pidl) == 0 || typ == ILGDN_FORPARSING {
        ret = ((*(*lsf).lpVtbl).GetDisplayNameOf)(lsf, pidl, flag, &mut strret);
        if SUCCEEDED(ret) && StrRetToStrNW(path, MAX_PATH as UINT, &mut strret, (pidl).cast()) == FALSE {
            ret = E_FAIL;
        }
    } else {
        let mut psfParent: LPSHELLFOLDER = null_mut();
        let mut pidllast: LPCITEMIDLIST = null();
        ret = SHBindToParent(
            pidl,
            &IID_IShellFolder,
            &mut psfParent as *mut LPSHELLFOLDER as *mut LPVOID,
            &mut pidllast,
        );
        if SUCCEEDED(ret) {
            ret = ((*(*psfParent).lpVtbl).GetDisplayNameOf)(psfParent, pidllast, flag, &mut strret);
            if SUCCEEDED(ret) && StrRetToStrNW(path, MAX_PATH as UINT, &mut strret, (pidllast).cast()) == FALSE {
                ret = E_FAIL;
            }
            ((*(*psfParent).lpVtbl).Release)(psfParent);
        }
    }

    if psf.is_null() {
        ((*(*lsf).lpVtbl).Release)(lsf);
    }
    if SUCCEEDED(ret) {
        TRUE
    } else {
        FALSE
    }
}

/// ILGetDisplayNameEx [SHELL32.186] (pidl.c:172).
#[no_mangle]
pub unsafe extern "system" fn ILGetDisplayNameEx(
    psf: LPSHELLFOLDER,
    pidl: LPCITEMIDLIST,
    path: LPVOID,
    typ: DWORD,
) -> BOOL {
    if shell_os_is_unicode() {
        ILGetDisplayNameExW(psf, pidl, path as LPWSTR, typ)
    } else {
        ILGetDisplayNameExA(psf, pidl, path as LPSTR, typ)
    }
}

/// ILGetDisplayName [SHELL32.15] (pidl.c:184).
#[no_mangle]
pub unsafe extern "system" fn ILGetDisplayName(pidl: LPCITEMIDLIST, path: LPVOID) -> BOOL {
    if shell_os_is_unicode() {
        ILGetDisplayNameExW(null_mut(), pidl, path as LPWSTR, ILGDN_FORPARSING)
    } else {
        ILGetDisplayNameExA(null_mut(), pidl, path as LPSTR, ILGDN_FORPARSING)
    }
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  ILFindLastID / ILRemoveLastID / ILClone / ILCloneFirst
// ════════════════════════════════════════════════════════════════════════════════════════
/// ILFindLastID [SHELL32.16] (pidl.c:199).
#[no_mangle]
pub unsafe extern "system" fn ILFindLastID(pidl: LPCITEMIDLIST) -> LPITEMIDLIST {
    if pidl.is_null() {
        return null_mut();
    }
    let mut pidl = pidl;
    let mut pidl_last = pidl;
    while il_cb(pidl) != 0 {
        pidl_last = pidl;
        pidl = ILGetNext(pidl);
    }
    pidl_last as LPITEMIDLIST
}

/// ILRemoveLastID [SHELL32.17] (pidl.c:222).
#[no_mangle]
pub unsafe extern "system" fn ILRemoveLastID(pidl: LPITEMIDLIST) -> BOOL {
    if _il_is_empty(pidl) {
        return FALSE;
    }
    il_set_cb(ILFindLastID(pidl), 0);
    TRUE
}

/// ILClone [SHELL32.18] (pidl.c:238).
#[no_mangle]
pub unsafe extern "system" fn ILClone(pidl: LPCITEMIDLIST) -> LPITEMIDLIST {
    if pidl.is_null() {
        return null_mut();
    }
    let len = ILGetSize(pidl);
    let newpidl = SHAlloc(len as SIZE_T) as LPITEMIDLIST;
    if !newpidl.is_null() {
        r_memcpy(newpidl as *mut u8, pidl as *const u8, len as usize);
    }
    newpidl
}

/// ILCloneFirst [SHELL32.19] (pidl.c:263).
#[no_mangle]
pub unsafe extern "system" fn ILCloneFirst(pidl: LPCITEMIDLIST) -> LPITEMIDLIST {
    let mut pidl_new: LPITEMIDLIST = null_mut();
    if !pidl.is_null() {
        let len = il_cb(pidl);
        pidl_new = SHAlloc((len as SIZE_T) + 2) as LPITEMIDLIST;
        if !pidl_new.is_null() {
            r_memcpy(pidl_new as *mut u8, pidl as *const u8, len as usize + 2); // 2 -> mind a desktop pidl
            if len != 0 {
                il_set_cb(ILGetNext(pidl_new), 0);
            }
        }
    }
    pidl_new
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  Stream load/save
// ════════════════════════════════════════════════════════════════════════════════════════
/// ILLoadFromStream (SHELL32.26) (pidl.c:294).
#[no_mangle]
pub unsafe extern "system" fn ILLoadFromStream(pStream: *mut IStream, ppPidl: *mut LPITEMIDLIST) -> HRESULT {
    let mut wLen: WORD = 0;
    let mut dwBytesRead: DWORD = 0;
    let mut ret: HRESULT = E_FAIL;

    SHFree(*ppPidl as LPVOID);
    write_volatile(ppPidl, null_mut());

    ((*(*pStream).lpVtbl).AddRef)(pStream);

    if SUCCEEDED(((*(*pStream).lpVtbl).Read)(
        pStream,
        &mut wLen as *mut WORD as *mut c_void,
        2,
        &mut dwBytesRead,
    )) {
        if wLen != 0 {
            write_volatile(ppPidl, SHAlloc(wLen as SIZE_T) as LPITEMIDLIST);
            if SUCCEEDED(((*(*pStream).lpVtbl).Read)(
                pStream,
                *ppPidl as *mut c_void,
                wLen as ULONG,
                &mut dwBytesRead,
            )) {
                ret = S_OK;
            } else {
                SHFree(*ppPidl as LPVOID);
                write_volatile(ppPidl, null_mut());
            }
        } else {
            write_volatile(ppPidl, null_mut());
            ret = S_OK;
        }
    }

    // __REACTOS__: the `if (*ppPidl && !pcheck(*ppPidl)) { }` body is empty (WARN dropped) →
    // we don't know all pidl formats and must allow loading unknown ones. Omitted.

    ((*(*pStream).lpVtbl).Release)(pStream);
    ret
}

/// ILSaveToStream (SHELL32.27) (pidl.c:354).
#[no_mangle]
pub unsafe extern "system" fn ILSaveToStream(pStream: *mut IStream, pPidl: LPCITEMIDLIST) -> HRESULT {
    let mut ret: HRESULT = E_FAIL;
    ((*(*pStream).lpVtbl).AddRef)(pStream);

    let wLen: WORD = ILGetSize(pPidl) as WORD;

    if SUCCEEDED(((*(*pStream).lpVtbl).Write)(
        pStream,
        &wLen as *const WORD as *const c_void,
        2,
        null_mut(),
    )) && SUCCEEDED(((*(*pStream).lpVtbl).Write)(
        pStream,
        pPidl as *const c_void,
        wLen as ULONG,
        null_mut(),
    )) {
        ret = S_OK;
    }
    ((*(*pStream).lpVtbl).Release)(pStream);
    ret
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  SHILCreateFromPath{A,W,AW}
// ════════════════════════════════════════════════════════════════════════════════════════
/// SHILCreateFromPathA [SHELL32.28] (pidl.c:392).
#[no_mangle]
pub unsafe extern "system" fn SHILCreateFromPathA(
    path: LPCSTR,
    ppidl: *mut LPITEMIDLIST,
    attributes: *mut DWORD,
) -> HRESULT {
    let mut lpszDisplayName: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    if MultiByteToWideChar(CP_ACP, 0, path, -1, lpszDisplayName.as_mut_ptr(), MAX_PATH as INT) == 0 {
        lpszDisplayName[MAX_PATH - 1] = 0;
    }
    SHILCreateFromPathW(lpszDisplayName.as_ptr(), ppidl, attributes)
}

/// SHILCreateFromPathW (pidl.c:404).
#[no_mangle]
pub unsafe extern "system" fn SHILCreateFromPathW(
    path: LPCWSTR,
    ppidl: *mut LPITEMIDLIST,
    attributes: *mut DWORD,
) -> HRESULT {
    let mut sf: LPSHELLFOLDER = null_mut();
    let mut pchEaten: DWORD = 0;
    let mut ret: HRESULT = E_FAIL;

    if SUCCEEDED(SHGetDesktopFolder(&mut sf)) {
        ret = ((*(*sf).lpVtbl).ParseDisplayName)(
            sf,
            null_mut(),
            null_mut(),
            path as LPWSTR,
            &mut pchEaten,
            ppidl,
            attributes,
        );
        ((*(*sf).lpVtbl).Release)(sf);
    }
    ret
}

/// SHILCreateFromPathAW (pidl.c:420).
#[no_mangle]
pub unsafe extern "system" fn SHILCreateFromPathAW(
    path: LPCVOID,
    ppidl: *mut LPITEMIDLIST,
    attributes: *mut DWORD,
) -> HRESULT {
    if shell_os_is_unicode() {
        SHILCreateFromPathW(path as LPCWSTR, ppidl, attributes)
    } else {
        SHILCreateFromPathA(path as LPCSTR, ppidl, attributes)
    }
}

/// SHCloneSpecialIDList [SHELL32.89] (pidl.c:446).
#[no_mangle]
pub unsafe extern "system" fn SHCloneSpecialIDList(
    hwndOwner: HWND,
    nFolder: INT,
    fCreate: BOOL,
) -> LPITEMIDLIST {
    let mut ppidl: LPITEMIDLIST = null_mut();
    let mut nFolder = nFolder;
    if fCreate != FALSE {
        nFolder |= CSIDL_FLAG_CREATE;
    }
    SHGetSpecialFolderLocation(hwndOwner, nFolder, &mut ppidl);
    ppidl
}

/// ILGlobalClone [SHELL32.20] (pidl.c:472) — clone via comctl32 Alloc.
#[no_mangle]
pub unsafe extern "system" fn ILGlobalClone(pidl: LPCITEMIDLIST) -> LPITEMIDLIST {
    if pidl.is_null() {
        return null_mut();
    }
    let len = ILGetSize(pidl);
    let newpidl = Alloc(len as INT) as LPITEMIDLIST;
    if !newpidl.is_null() {
        r_memcpy(newpidl as *mut u8, pidl as *const u8, len as usize);
    }
    newpidl
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  Depth / equality / parent / child  (__REACTOS__ paths)
// ════════════════════════════════════════════════════════════════════════════════════════
/// _ILGetDepth (pidl.c:497) — number of SHITEMIDs.
#[no_mangle]
pub unsafe extern "C" fn _ILGetDepth(pidl: LPCITEMIDLIST) -> UINT {
    let mut pidl = pidl;
    let mut i: UINT = 0;
    loop {
        if pidl.is_null() || il_cb(pidl) == 0 {
            return i;
        }
        pidl = _il_unsafe_next(pidl);
        i += 1;
    }
}

/// _ILMemCmpEqualIDList (pidl.c:507) — static.
unsafe fn _il_memcmp_equal_idlist(mut p1: LPCITEMIDLIST, mut p2: LPCITEMIDLIST) -> BOOL {
    loop {
        let cb1: DWORD = if !p1.is_null() { il_cb(p1) as DWORD } else { 0x80000000 }; // Empty != NULL
        let cb2: DWORD = if !p2.is_null() { il_cb(p2) as DWORD } else { 0x80000000 };
        if cb1 != cb2 {
            return FALSE;
        }
        if (cb1 & 0xffff) == 0 {
            return if cb1 == cb2 { TRUE } else { FALSE };
        }
        if r_memcmp(p1 as *const u8, p2 as *const u8, cb1 as usize) != 0 {
            return FALSE;
        }
        p1 = _il_unsafe_next(p1);
        p2 = _il_unsafe_next(p2);
    }
}

/// ILIsEqual [SHELL32.21] (pidl.c:583) — __REACTOS__ path.
#[no_mangle]
pub unsafe extern "system" fn ILIsEqual(pidl1: LPCITEMIDLIST, pidl2: LPCITEMIDLIST) -> BOOL {
    if pidl1 == pidl2 || _il_memcmp_equal_idlist(pidl1, pidl2) != FALSE {
        return TRUE;
    }

    let depth1 = _ILGetDepth(pidl1);
    if depth1 != 0 && depth1 == _ILGetDepth(pidl2) {
        let mut psf_desktop: LPSHELLFOLDER = null_mut();
        if SUCCEEDED(SHGetDesktopFolder(&mut psf_desktop)) {
            let hr = ((*(*psf_desktop).lpVtbl).CompareIDs)(
                psf_desktop,
                SHCIDS_CANONICALONLY as LPARAM,
                pidl1,
                pidl2,
            );
            ((*(*psf_desktop).lpVtbl).Release)(psf_desktop);
            return if hr == 0 { TRUE } else { FALSE };
        }
    }
    FALSE
}

/// _ILIsParentEx (pidl.c:639).
#[no_mangle]
pub unsafe extern "C" fn _ILIsParentEx(
    pidlParent: LPCITEMIDLIST,
    pidlChild: LPCITEMIDLIST,
    bImmediate: BOOL,
) -> LPCITEMIDLIST {
    let p_parent_root = pidlParent;
    let p_child_root = pidlChild;
    let mut p_result: LPCITEMIDLIST = null();
    let mut cb: SIZE_T = 0;

    if pidlParent.is_null() || pidlChild.is_null() {
        return p_result;
    }

    let mut pidl_parent = pidlParent;
    let mut pidl_child = pidlChild;

    while il_cb(pidl_parent) != 0 {
        cb += il_cb(pidl_child) as SIZE_T;
        if il_cb(pidl_child) == 0 {
            if il_cb(pidl_parent) != 0 {
                return p_result; // The child is shorter than the parent
            } else {
                break;
            }
        }
        pidl_child = _il_unsafe_next(pidl_child);
        pidl_parent = _il_unsafe_next(pidl_parent);
    }

    if bImmediate != FALSE && (il_cb(pidl_child) == 0 || il_cb(_il_unsafe_next(pidl_child)) != 0) {
        return p_result; // Same as parent or a deeper grandchild
    }

    let pidl = SHAlloc(cb + (size_of::<WORD>()) as SIZE_T) as LPITEMIDLIST;
    if !pidl.is_null() {
        r_memcpy(pidl as *mut u8, p_child_root as *const u8, (cb) as _);
        r_memset((pidl as *mut u8).add((cb) as _), 0, size_of::<WORD>());
        if ILIsEqual(p_parent_root, pidl) != FALSE {
            p_result = pidl_child;
        }
        ILFree(pidl);
    }
    p_result
}

/// ILIsParent [SHELL32.23] (pidl.c:698) — __REACTOS__ path.
#[no_mangle]
pub unsafe extern "system" fn ILIsParent(
    pidlParent: LPCITEMIDLIST,
    pidlChild: LPCITEMIDLIST,
    bImmediate: BOOL,
) -> BOOL {
    if !_ILIsParentEx(pidlParent, pidlChild, bImmediate).is_null() {
        TRUE
    } else {
        FALSE
    }
}

/// ILFindChild [SHELL32.24] (pidl.c:751) — __REACTOS__ path.
#[no_mangle]
pub unsafe extern "system" fn ILFindChild(pidl1: LPITEMIDLIST, pidl2: LPCITEMIDLIST) -> LPITEMIDLIST {
    if _ILIsDesktop(pidl1) != FALSE {
        return pidl2 as LPITEMIDLIST;
    }
    _ILIsParentEx(pidl1, pidl2, FALSE) as LPITEMIDLIST
}

/// ILCombine [SHELL32.25] (pidl.c:817).
#[no_mangle]
pub unsafe extern "system" fn ILCombine(pidl1: LPCITEMIDLIST, pidl2: LPCITEMIDLIST) -> LPITEMIDLIST {
    if pidl1.is_null() && pidl2.is_null() {
        return null_mut();
    }
    if pidl1.is_null() {
        return ILClone(pidl2);
    }
    if pidl2.is_null() {
        return ILClone(pidl1);
    }

    let len1 = ILGetSize(pidl1) - 2;
    let len2 = ILGetSize(pidl2);
    let pidl_new = SHAlloc((len1 + len2) as SIZE_T) as LPITEMIDLIST;
    if !pidl_new.is_null() {
        r_memcpy(pidl_new as *mut u8, pidl1 as *const u8, len1 as usize);
        r_memcpy((pidl_new as *mut u8).add(len1 as usize), pidl2 as *const u8, len2 as usize);
    }
    pidl_new
}

/// SHLogILFromFSIL [SHELL32.95] (pidl.c:912) — __REACTOS__ path.
#[no_mangle]
pub unsafe extern "system" fn SHLogILFromFSIL(pidl: LPITEMIDLIST) -> LPITEMIDLIST {
    let mut pidl_new: LPITEMIDLIST = null_mut();
    SHELL32_AliasTranslatePidl(pidl as LPCITEMIDLIST, &mut pidl_new, ALIAS_ANY);
    pidl_new
}

/// ILGetSize [SHELL32.152] (pidl.c:941).
#[no_mangle]
pub unsafe extern "system" fn ILGetSize(pidl: LPCITEMIDLIST) -> UINT {
    let mut len: UINT = 0;
    if !pidl.is_null() {
        let mut si = pidl as *const u8; // &pidl->mkid
        loop {
            let cb = rd_u16(si);
            if cb == 0 {
                break;
            }
            len += cb as UINT;
            si = si.add(cb as usize);
        }
        len += 2;
    }
    len
}

/// ILGetNext [SHELL32.153] (pidl.c:977).
#[no_mangle]
pub unsafe extern "system" fn ILGetNext(pidl: LPCITEMIDLIST) -> LPITEMIDLIST {
    if !pidl.is_null() {
        let len = il_cb(pidl);
        if len != 0 {
            return (pidl as *const u8).add(len as usize) as LPITEMIDLIST;
        }
    }
    null_mut()
}

/// ILAppendID [SHELL32.154] (pidl.c:1011).
#[no_mangle]
pub unsafe extern "system" fn ILAppendID(pidl: LPITEMIDLIST, item: LPCSHITEMID, bEnd: BOOL) -> LPITEMIDLIST {
    let itemid = item as LPCITEMIDLIST;

    if _ILIsDesktop(pidl) != FALSE {
        let idl_ret = ILClone(itemid);
        SHFree(pidl as LPVOID);
        return idl_ret;
    }

    let idl_ret = if bEnd != FALSE {
        ILCombine(pidl, itemid)
    } else {
        ILCombine(itemid, pidl)
    };
    SHFree(pidl as LPVOID);
    idl_ret
}

/// ILFree [SHELL32.155] (pidl.c:1051).
#[no_mangle]
pub unsafe extern "system" fn ILFree(pidl: LPITEMIDLIST) {
    SHFree(pidl as LPVOID);
}

/// ILGlobalFree [SHELL32.156] (pidl.c:1071).
#[no_mangle]
pub unsafe extern "system" fn ILGlobalFree(pidl: LPITEMIDLIST) {
    Free(pidl as LPVOID);
}

/// ILCreateFromPathA [SHELL32.189] (pidl.c:1092).
#[no_mangle]
pub unsafe extern "system" fn ILCreateFromPathA(path: LPCSTR) -> LPITEMIDLIST {
    let mut pidlnew: LPITEMIDLIST = null_mut();
    if SUCCEEDED(SHILCreateFromPathA(path, &mut pidlnew, null_mut())) {
        return pidlnew;
    }
    null_mut()
}

/// ILCreateFromPathW [SHELL32.190] (pidl.c:1108).
#[no_mangle]
pub unsafe extern "system" fn ILCreateFromPathW(path: LPCWSTR) -> LPITEMIDLIST {
    let mut pidlnew: LPITEMIDLIST = null_mut();
    if SUCCEEDED(SHILCreateFromPathW(path, &mut pidlnew, null_mut())) {
        return pidlnew;
    }
    null_mut()
}

/// ILCreateFromPath [SHELL32.157] (pidl.c:1122).
#[no_mangle]
pub unsafe extern "system" fn ILCreateFromPathAW(path: LPCVOID) -> LPITEMIDLIST {
    if shell_os_is_unicode() {
        ILCreateFromPathW(path as LPCWSTR)
    } else {
        ILCreateFromPathA(path as LPCSTR)
    }
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  Path parsing / simple ID lists
// ════════════════════════════════════════════════════════════════════════════════════════
/// _ILParsePathW (pidl.c:1156) — static.
unsafe fn _il_parse_path_w(
    path: LPCWSTR,
    lpFindFile: *const WIN32_FIND_DATAW,
    bBindCtx: BOOL,
    ppidl: *mut LPITEMIDLIST,
    prgfInOut: *mut DWORD,
) -> HRESULT {
    let mut p_sf: LPSHELLFOLDER = null_mut();
    let mut p_bc: LPBC = null_mut();

    let mut ret = SHGetDesktopFolder(&mut p_sf);
    if FAILED(ret) {
        return ret;
    }

    if !lpFindFile.is_null() || bBindCtx != FALSE {
        ret = IFileSystemBindData_Constructor(lpFindFile, &mut p_bc);
    }

    if SUCCEEDED(ret) {
        ret = ((*(*p_sf).lpVtbl).ParseDisplayName)(
            p_sf,
            null_mut(),
            p_bc,
            path as LPWSTR,
            null_mut(),
            ppidl,
            prgfInOut,
        );
    }

    if !p_bc.is_null() {
        ((*(*p_bc).lpVtbl).Release)(p_bc);
    }
    ((*(*p_sf).lpVtbl).Release)(p_sf);

    if FAILED(ret) && !ppidl.is_null() {
        write_volatile(ppidl, null_mut());
    }
    ret
}

/// SHELL32_CreateSimpleIDListFromPath (pidl.c:1195) — cdecl export `_SHELL32_CreateSimpleIDListFromPath`.
#[no_mangle]
pub unsafe extern "C" fn SHELL32_CreateSimpleIDListFromPath(
    pszPath: LPCWSTR,
    dwAttributes: DWORD,
) -> LPITEMIDLIST {
    let mut data: WIN32_FIND_DATAW = zeroed();
    data.dwFileAttributes = dwAttributes;
    let mut pidl: LPITEMIDLIST = null_mut();
    _il_parse_path_w(pszPath, &data, TRUE, &mut pidl, null_mut());
    pidl
}

/// SHSimpleIDListFromPathA [SHELL32.162] (pidl.c:1223).
#[no_mangle]
pub unsafe extern "system" fn SHSimpleIDListFromPathA(lpszPath: LPCSTR) -> LPITEMIDLIST {
    let mut pidl: LPITEMIDLIST = null_mut();
    let mut w_path: LPWSTR = null_mut();

    if !lpszPath.is_null() {
        let len = MultiByteToWideChar(CP_ACP, 0, lpszPath, -1, null_mut(), 0);
        w_path = HeapAlloc(GetProcessHeap(), 0, (len as SIZE_T * size_of::<WCHAR>() as SIZE_T) as _) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, lpszPath, -1, w_path, len);
    }

    _il_parse_path_w(w_path, null(), TRUE, &mut pidl, null_mut());

    HeapFree(GetProcessHeap(), 0, w_path as LPVOID);
    pidl
}

/// SHSimpleIDListFromPathW (pidl.c:1245).
#[no_mangle]
pub unsafe extern "system" fn SHSimpleIDListFromPathW(lpszPath: LPCWSTR) -> LPITEMIDLIST {
    let mut pidl: LPITEMIDLIST = null_mut();
    _il_parse_path_w(lpszPath, null(), TRUE, &mut pidl, null_mut());
    pidl
}

/// SHSimpleIDListFromPathAW (pidl.c:1257).
#[no_mangle]
pub unsafe extern "system" fn SHSimpleIDListFromPathAW(lpszPath: LPCVOID) -> LPITEMIDLIST {
    if shell_os_is_unicode() {
        SHSimpleIDListFromPathW(lpszPath as LPCWSTR)
    } else {
        SHSimpleIDListFromPathA(lpszPath as LPCSTR)
    }
}

// ── IShellFolder2 detail helpers + SHELL_VariantToBuffer (pidl.c:1264 / shell32_main.h) ──
/// SHELL_GetSafeArrayDataPtr (shell32_main.h:287).
unsafe fn shell_get_safearray_dataptr(p_sa: *mut SAFEARRAY, pcb: *mut SIZE_T) -> *mut c_void {
    if (*p_sa).cDims != 1 {
        return null_mut();
    }
    let mut lob: LONG = 0;
    let mut upb: LONG = 0;
    SafeArrayGetLBound(p_sa, 1, &mut lob);
    SafeArrayGetUBound(p_sa, 1, &mut upb);
    *pcb = (upb - lob) as SIZE_T + 1;
    (*p_sa).pvData
}

/// SHELL_VariantToBuffer (shell32_main.h:299).
unsafe fn shell_variant_to_buffer(p_var: *mut VARIANT, p_data: *mut c_void, cb: SIZE_T) -> HRESULT {
    if (*p_var).n1.n2.vt != (VT_ARRAY | VT_UI1) {
        return E_INVALIDARG;
    }
    let mut cb_arr: SIZE_T = 0;
    let p_arr_data = shell_get_safearray_dataptr((*p_var).n1.n2.n3.parray, &mut cb_arr);
    if p_arr_data.is_null() || cb_arr < cb {
        return E_FAIL;
    }
    r_memcpy(p_data as *mut u8, p_arr_data as *const u8, (cb) as _);
    if cb_arr > cb {
        S_FALSE
    } else {
        S_OK
    }
}

/// GetShellFolder2ItemDetailsExToBuffer (pidl.c:1264) — static.
unsafe fn get_shellfolder2_item_details_ex_to_buffer(
    psf2: *mut IShellFolder2,
    pidl: LPCITEMIDLIST,
    pscid: *const SHCOLUMNID,
    buf: *mut c_void,
    cb: UINT,
) -> HRESULT {
    let mut var: VARIANT = zeroed();
    var.n1.n2.vt = VT_EMPTY;
    let mut hr = ((*(*psf2).lpVtbl).GetDetailsEx)(psf2, pidl, pscid, &mut var);
    if SUCCEEDED(hr) {
        hr = shell_variant_to_buffer(&mut var, buf, cb as SIZE_T);
        VariantClear(&mut var);
    }
    hr
}

/// GetShellFolder1ItemDetailsExToBuffer (pidl.c:1278) — static.
unsafe fn get_shellfolder1_item_details_ex_to_buffer(
    psf: LPSHELLFOLDER,
    pidl: LPCITEMIDLIST,
    pscid: *const SHCOLUMNID,
    buf: *mut c_void,
    cb: UINT,
) -> HRESULT {
    let mut psf2: *mut IShellFolder2 = null_mut();
    let mut hr = ((*(*psf).lpVtbl).QueryInterface)(
        psf,
        &IID_IShellFolder2,
        &mut psf2 as *mut *mut IShellFolder2 as *mut *mut c_void,
    );
    if SUCCEEDED(hr) {
        hr = get_shellfolder2_item_details_ex_to_buffer(psf2, pidl, pscid, buf, cb);
        ((*(*psf2).lpVtbl).Release)(psf2);
    }
    hr
}

/// SHGetDataFromIDListA [SHELL32.247] (pidl.c:1298).
#[no_mangle]
pub unsafe extern "system" fn SHGetDataFromIDListA(
    psf: LPSHELLFOLDER,
    pidl: LPCITEMIDLIST,
    nFormat: INT,
    dest: LPVOID,
    len: INT,
) -> HRESULT {
    if psf.is_null() || dest.is_null() {
        return E_INVALIDARG;
    }

    match nFormat {
        SHGDFIL_FINDDATA => {
            let pfd = dest as *mut WIN32_FIND_DATAA;

            if _ILIsDrive(pidl) != FALSE || _ILIsSpecialFolder(pidl) != FALSE {
                return E_INVALIDARG;
            }
            if (len as usize) < size_of::<WIN32_FIND_DATAA>() {
                return E_INVALIDARG;
            }

            r_memset(pfd as *mut u8, 0, size_of::<WIN32_FIND_DATAA>());
            _ILGetFileDateTime(pidl, addr_of_mut!((*pfd).ftLastWriteTime));
            (*pfd).dwFileAttributes = _ILGetFileAttributes(pidl, null_mut(), 0);
            (*pfd).nFileSizeLow = _ILGetFileSize(pidl, null_mut(), 0);

            let filename = _ILGetTextPointer(pidl);
            let shortname = _ILGetSTextPointer(pidl);

            if !filename.is_null() {
                lstrcpynA(addr_of_mut!((*pfd).cFileName) as LPSTR, filename, MAX_PATH as INT);
            } else {
                (*pfd).cFileName[0] = 0;
            }

            if !shortname.is_null() {
                lstrcpynA(addr_of_mut!((*pfd).cAlternateFileName) as LPSTR, shortname, 14);
            } else {
                (*pfd).cAlternateFileName[0] = 0;
            }
            S_OK
        }
        SHGDFIL_DESCRIPTIONID => SHGetDataFromIDListW(psf, pidl, nFormat, dest, len),
        SHGDFIL_NETRESOURCE => E_INVALIDARG, // FIXME: stub (break → return E_INVALIDARG)
        _ => E_INVALIDARG,                   // ERR unknown SHGDFIL
    }
}

/// SHGetDataFromIDListW [SHELL32.248] (pidl.c:1358).
#[no_mangle]
pub unsafe extern "system" fn SHGetDataFromIDListW(
    psf: LPSHELLFOLDER,
    pidl: LPCITEMIDLIST,
    nFormat: INT,
    dest: LPVOID,
    len: INT,
) -> HRESULT {
    if psf.is_null() || dest.is_null() {
        return E_INVALIDARG;
    }

    match nFormat {
        SHGDFIL_FINDDATA => {
            let pfd = dest as *mut WIN32_FIND_DATAW;

            if _ILIsDrive(pidl) != FALSE {
                return E_INVALIDARG;
            }
            if (len as usize) < size_of::<WIN32_FIND_DATAW>() {
                return E_INVALIDARG;
            }

            r_memset(pfd as *mut u8, 0, size_of::<WIN32_FIND_DATAW>());
            _ILGetFileDateTime(pidl, addr_of_mut!((*pfd).ftLastWriteTime));
            (*pfd).dwFileAttributes = _ILGetFileAttributes(pidl, null_mut(), 0);
            (*pfd).nFileSizeLow = _ILGetFileSize(pidl, null_mut(), 0);

            let filename = _ILGetTextPointer(pidl);
            let shortname = _ILGetSTextPointer(pidl);

            if filename.is_null() {
                (*pfd).cFileName[0] = 0;
            } else if MultiByteToWideChar(
                CP_ACP,
                0,
                filename,
                -1,
                addr_of_mut!((*pfd).cFileName) as LPWSTR,
                MAX_PATH as INT,
            ) == 0
            {
                (*pfd).cFileName[MAX_PATH - 1] = 0;
            }

            if shortname.is_null() {
                (*pfd).cAlternateFileName[0] = 0;
            } else if MultiByteToWideChar(
                CP_ACP,
                0,
                shortname,
                -1,
                addr_of_mut!((*pfd).cAlternateFileName) as LPWSTR,
                14,
            ) == 0
            {
                (*pfd).cAlternateFileName[13] = 0;
            }
            S_OK
        }
        SHGDFIL_DESCRIPTIONID => {
            // TODO: Use PKEY_DescriptionID when the propsys headers are ready
            let scid = SHCOLUMNID {
                fmtid: FMTID_ShellDetails,
                pid: PID_DESCRIPTIONID,
            };
            get_shellfolder1_item_details_ex_to_buffer(psf, pidl, &scid, dest, len as UINT)
        }
        SHGDFIL_NETRESOURCE => E_INVALIDARG, // FIXME: stub
        _ => E_INVALIDARG,                   // ERR unknown SHGDFIL
    }
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  Path-from-IDList / bind / parse / object
// ════════════════════════════════════════════════════════════════════════════════════════
/// SHGetPathFromIDListA [SHELL32.@] (pidl.c:1434).
#[no_mangle]
pub unsafe extern "system" fn SHGetPathFromIDListA(pidl: LPCITEMIDLIST, pszPath: LPSTR) -> BOOL {
    let mut wsz_path: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    let b_success = SHGetPathFromIDListW(pidl, wsz_path.as_mut_ptr());
    WideCharToMultiByte(CP_ACP, 0, wsz_path.as_ptr(), -1, pszPath, MAX_PATH as INT, null(), null_mut());
    b_success
}

/// SHGetPathCchFromIDListW [SHELL32.@] (pidl.c:1450).
#[no_mangle]
pub unsafe extern "system" fn SHGetPathCchFromIDListW(
    pidl: LPCITEMIDLIST,
    pszPath: LPWSTR,
    cchPathMax: SIZE_T,
) -> HRESULT {
    let mut pidl_last: LPCITEMIDLIST = null();
    let mut psf_folder: LPSHELLFOLDER = null_mut();
    let mut strret: STRRET = zeroed();

    write_volatile(pszPath, UNICODE_NULL);
    if pidl.is_null() {
        return E_FAIL;
    }

    let mut hr = SHBindToParent(
        pidl,
        &IID_IShellFolder,
        &mut psf_folder as *mut LPSHELLFOLDER as *mut LPVOID,
        &mut pidl_last,
    );
    if FAILED(hr) {
        return hr;
    }

    let mut dw_attributes: DWORD = SFGAO_FILESYSTEM;
    hr = ((*(*psf_folder).lpVtbl).GetAttributesOf)(psf_folder, 1, &mut pidl_last, &mut dw_attributes);
    if FAILED(hr) || (dw_attributes & SFGAO_FILESYSTEM) == 0 {
        ((*(*psf_folder).lpVtbl).Release)(psf_folder);
        return E_FAIL;
    }

    hr = ((*(*psf_folder).lpVtbl).GetDisplayNameOf)(psf_folder, pidl_last, SHGDN_FORPARSING, &mut strret);
    ((*(*psf_folder).lpVtbl).Release)(psf_folder);
    if FAILED(hr) {
        return hr;
    }

    StrRetToBufW((&raw mut strret).cast(), pidl_last, pszPath, cchPathMax as UINT)
}

/// SHGetPathFromIDListW [SHELL32.@] (pidl.c:1496).
#[no_mangle]
pub unsafe extern "system" fn SHGetPathFromIDListW(pidl: LPCITEMIDLIST, pszPath: LPWSTR) -> BOOL {
    if SUCCEEDED(SHGetPathCchFromIDListW(pidl, pszPath, (MAX_PATH) as SIZE_T)) {
        TRUE
    } else {
        FALSE
    }
}

/// SHBindToParent [shell version 5.0] (pidl.c:1504).
#[no_mangle]
pub unsafe extern "system" fn SHBindToParent(
    pidl: LPCITEMIDLIST,
    riid: REFIID,
    ppv: *mut LPVOID,
    ppidlLast: *mut LPCITEMIDLIST,
) -> HRESULT {
    if pidl.is_null() || ppv.is_null() {
        return E_INVALIDARG;
    }

    write_volatile(ppv, null_mut());
    if !ppidlLast.is_null() {
        write_volatile(ppidlLast, null());
    }

    let mut psf_desktop: LPSHELLFOLDER = null_mut();
    let mut hr = SHGetDesktopFolder(&mut psf_desktop);
    if FAILED(hr) {
        return hr;
    }

    if _ILIsPidlSimple(pidl) != FALSE {
        // we are on desktop level
        hr = ((*(*psf_desktop).lpVtbl).QueryInterface)(psf_desktop, riid, ppv);
    } else {
        let pidl_parent = ILClone(pidl);
        ILRemoveLastID(pidl_parent);
        hr = ((*(*psf_desktop).lpVtbl).BindToObject)(psf_desktop, pidl_parent, null_mut(), riid, ppv);
        SHFree(pidl_parent as LPVOID);
    }

    ((*(*psf_desktop).lpVtbl).Release)(psf_desktop);

    if SUCCEEDED(hr) && !ppidlLast.is_null() {
        write_volatile(ppidlLast, ILFindLastID(pidl));
    }
    hr
}

/// SHParseDisplayName [shell version 6.0] (pidl.c:1548).
#[no_mangle]
pub unsafe extern "system" fn SHParseDisplayName(
    pszName: LPCWSTR,
    pbc: *mut IBindCtx,
    ppidl: *mut LPITEMIDLIST,
    sfgaoIn: SFGAOF,
    psfgaoOut: *mut SFGAOF,
) -> HRESULT {
    write_volatile(ppidl, null_mut());
    if !psfgaoOut.is_null() {
        write_volatile(psfgaoOut, 0);
    }

    let psz_name_dup = StrDupW(pszName);
    if psz_name_dup.is_null() {
        return E_OUTOFMEMORY;
    }

    let mut psf_desktop: LPSHELLFOLDER = null_mut();
    let mut p_bind_ctx: LPBC = null_mut();
    let mut hr = SHGetDesktopFolder(&mut psf_desktop);
    if FAILED(hr) {
        LocalFree(psz_name_dup as HANDLE);
        return hr;
    }

    let mut pbc = pbc;
    if pbc.is_null() {
        hr = BindCtx_RegisterObjectParam(
            null_mut(),
            STR_PARSE_TRANSLATE_ALIASES.as_ptr() as LPWSTR,
            null_mut(),
            &mut p_bind_ctx,
        );
        pbc = p_bind_ctx;
    }

    if SUCCEEDED(hr) {
        let mut sfgao: ULONG = sfgaoIn;
        let mut cch_eaten: ULONG = 0;
        let hwnd_ui = BindCtx_GetUIWindow(pbc);
        hr = ((*(*psf_desktop).lpVtbl).ParseDisplayName)(
            psf_desktop,
            hwnd_ui,
            pbc,
            psz_name_dup,
            &mut cch_eaten,
            ppidl,
            if !psfgaoOut.is_null() { &mut sfgao } else { null_mut() },
        );
        if SUCCEEDED(hr) && !psfgaoOut.is_null() {
            write_volatile(psfgaoOut, sfgao & sfgaoIn);
        }
    }

    LocalFree(psz_name_dup as HANDLE);

    if !psf_desktop.is_null() {
        ((*(*psf_desktop).lpVtbl).Release)(psf_desktop);
    }
    if !p_bind_ctx.is_null() {
        ((*(*p_bind_ctx).lpVtbl).Release)(p_bind_ctx);
    }
    hr
}

/// SHGetNameFromIDList [SHELL32.@] (pidl.c:1610).
#[no_mangle]
pub unsafe extern "system" fn SHGetNameFromIDList(
    pidl: LPCITEMIDLIST,
    sigdnName: SIGDN,
    ppszName: *mut PWSTR,
) -> HRESULT {
    let mut psfparent: LPSHELLFOLDER = null_mut();
    let mut child_pidl: LPCITEMIDLIST = null();
    let mut disp_name: STRRET = zeroed();

    write_volatile(ppszName, null_mut());
    let mut ret = SHBindToParent(
        pidl,
        &IID_IShellFolder,
        &mut psfparent as *mut LPSHELLFOLDER as *mut LPVOID,
        &mut child_pidl,
    );
    if SUCCEEDED(ret) {
        match sigdnName {
            SIGDN_NORMALDISPLAY
            | SIGDN_PARENTRELATIVEPARSING
            | SIGDN_PARENTRELATIVEEDITING
            | SIGDN_DESKTOPABSOLUTEPARSING
            | SIGDN_DESKTOPABSOLUTEEDITING
            | SIGDN_PARENTRELATIVEFORADDRESSBAR
            | SIGDN_PARENTRELATIVE => {
                disp_name.uType = STRRET_WSTR;
                ret = ((*(*psfparent).lpVtbl).GetDisplayNameOf)(
                    psfparent,
                    child_pidl,
                    sigdnName & 0xffff,
                    &mut disp_name,
                );
                if SUCCEEDED(ret) {
                    ret = StrRetToStrW((&raw mut disp_name).cast(), pidl, ppszName);
                }
            }
            SIGDN_FILESYSPATH => {
                let buf = CoTaskMemAlloc((size_of::<WCHAR>() * MAX_PATH) as SIZE_T) as LPWSTR;
                write_volatile(ppszName, buf);
                if SHGetPathFromIDListW(pidl, buf) != FALSE {
                    ret = S_OK;
                } else {
                    CoTaskMemFree(buf as LPVOID);
                    ret = E_INVALIDARG;
                }
            }
            // SIGDN_URL and default
            _ => {
                ret = E_FAIL;
            }
        }
        ((*(*psfparent).lpVtbl).Release)(psfparent);
    }
    ret
}

// Upstream-sync 2026-09: the previous body here (`return E_NOTIMPL; // FIXME`) was a
// faithful translation of the WRONG C source. wine/pidl.c wraps its own copy of this
// function in `#ifndef __REACTOS__` (dead in our build even before the cohab flip: this
// tree always defines __REACTOS__), while CShellItem.cpp -- also flipped, in the SAME
// crate -- carries the REAL, always-compiled ReactOS implementation. The lost-exports
// scanner only checks "does ANY Rust fn provide this symbol", so the dead-C-derived stub
// silently satisfied it and the export was never flagged, even after the tail-comment
// fix. Ported CShellItem.cpp's real body 1:1 (HIDA -> ILCombine -> SHCreateItemFromIDList,
// with a CF_HDROP fallback); CF_URL fallback stays unimplemented, matching upstream's own
// `FIXME("... should try CF_URL.")` (a no-op FIXME, not a dropped code path).
// DOGIF_ONLY_IF_ONE, CP_ACP, lstrlenW and MultiByteToWideChar are already declared
// file-scope above (lines ~47-186) for other exports in this TU; reused, not redeclared.
const DOGIF_NO_HDROP: DWORD = 2;
const DOGIF_NO_URL: DWORD = 4;
const CF_HDROP: DWORD = 15;
const TYMED_HGLOBAL: DWORD = 1;
const DVASPECT_CONTENT: DWORD = 1;

extern "system" {
    fn GlobalLock(memory: HGLOBAL) -> LPVOID;
    fn GlobalUnlock(memory: HGLOBAL) -> BOOL;
    fn RegisterClipboardFormatW(fmt: LPCWSTR) -> UINT;
    fn ReleaseStgMedium(pmedium: *mut STGMEDIUM);
    fn lstrlenA(s: LPCSTR) -> INT;
}

#[repr(C)]
struct DROPFILES {
    pFiles: DWORD,
    pt: crate::island::POINT,
    fNC: BOOL,
    fWide: BOOL,
}

/// SHGetItemFromDataObject [SHELL32.@] — real body, from CShellItem.cpp (not pidl.c's
/// dead `#ifndef __REACTOS__` twin; see the note above).
#[no_mangle]
pub unsafe extern "system" fn SHGetItemFromDataObject(
    pdtobj: *mut IDataObject,
    dwFlags: DWORD,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    use crate::shellutils::hida_clipformat;

    if pdtobj.is_null() {
        return E_INVALIDARG;
    }

    let mut fmt: FORMATETC = zeroed();
    fmt.cfFormat = hida_clipformat();
    fmt.ptd = null_mut();
    fmt.dwAspect = DVASPECT_CONTENT;
    fmt.lindex = -1;
    fmt.tymed = TYMED_HGLOBAL;

    let mut medium: STGMEDIUM = zeroed();
    let mut hr = ((*(*pdtobj).lpVtbl).GetData)(pdtobj, addr_of_mut!(fmt).cast(), addr_of_mut!(medium).cast());
    if hr >= 0 {
        let pida = GlobalLock(medium.__bindgen_anon_1.hGlobal) as *const crate::island::CIDA;
        if !pida.is_null()
            && (*pida).cidl >= 1
            && (((*pida).cidl > 1 && (dwFlags & DOGIF_ONLY_IF_ONE) == 0) || (*pida).cidl == 1)
        {
            let pidl = ILCombine(
                crate::shellutils::HIDA_GetPIDLFolder(pida).cast(),
                crate::shellutils::HIDA_GetPIDLItem(pida, 0).cast(),
            );
            if !pidl.is_null() {
                hr = crate::shell_item::SHCreateItemFromIDList(pidl, riid, ppv);
                ILFree(pidl);
            } else {
                hr = E_OUTOFMEMORY;
            }
        } else {
            hr = E_FAIL;
        }

        if !medium.__bindgen_anon_1.hGlobal.is_null() {
            GlobalUnlock(medium.__bindgen_anon_1.hGlobal);
            ReleaseStgMedium(&mut medium);
        }
    }

    if hr < 0 && (dwFlags & DOGIF_NO_HDROP) == 0 {
        let mut fmt2: FORMATETC = zeroed();
        fmt2.cfFormat = CF_HDROP as u16;
        fmt2.ptd = null_mut();
        fmt2.dwAspect = DVASPECT_CONTENT;
        fmt2.lindex = -1;
        fmt2.tymed = TYMED_HGLOBAL;
        let mut medium2: STGMEDIUM = zeroed();

        hr = ((*(*pdtobj).lpVtbl).GetData)(pdtobj, addr_of_mut!(fmt2).cast(), addr_of_mut!(medium2).cast());
        if hr >= 0 {
            let df = GlobalLock(medium2.__bindgen_anon_1.hGlobal) as *const DROPFILES;
            hr = E_FAIL;
            if !df.is_null() {
                let files = (df as *const u8).add((*df).pFiles as usize);
                if (*df).fWide == 0 {
                    let first = files as LPCSTR;
                    let multi = *files.add(lstrlenA(first) as usize + 1) != 0;
                    if !(multi && (dwFlags & DOGIF_ONLY_IF_ONE) != 0) {
                        let mut filename: [WCHAR; MAX_PATH] = [0; MAX_PATH];
                        MultiByteToWideChar(
                            CP_ACP,
                            0,
                            first,
                            -1,
                            filename.as_mut_ptr(),
                            filename.len() as INT,
                        );
                        hr = crate::shell_item::SHCreateItemFromParsingName(
                            filename.as_ptr(),
                            null_mut(),
                            riid,
                            ppv,
                        );
                    }
                } else {
                    // WCHAR-unit pointer arithmetic, matching the C's
                    // `*(first + lstrlenW(first) + 1) != 0` -- a byte-stride read here
                    // would wrongly treat a WCHAR whose low byte is 0 as the terminator.
                    let first = files as *const WCHAR;
                    let multi = *first.add(lstrlenW(first) as usize + 1) != 0;
                    if !(multi && (dwFlags & DOGIF_ONLY_IF_ONE) != 0) {
                        hr = crate::shell_item::SHCreateItemFromParsingName(first, null_mut(), riid, ppv);
                    }
                }
                GlobalUnlock(medium2.__bindgen_anon_1.hGlobal);
            }
            ReleaseStgMedium(&mut medium2);
        }
    }

    if hr < 0 && (dwFlags & DOGIF_NO_URL) == 0 {
        // FIXME("Failed to create item, should try CF_URL.\n"); -- matches upstream:
        // a logged FIXME, not an implemented fallback.
    }

    hr
}

/// SHGetIDListFromObject [SHELL32.@] (pidl.c:1680).
#[no_mangle]
pub unsafe extern "system" fn SHGetIDListFromObject(
    punk: *mut IUnknown,
    ppidl: *mut LPITEMIDLIST,
) -> HRESULT {
    if punk.is_null() {
        return E_NOINTERFACE;
    }

    write_volatile(ppidl, null_mut());

    // Try IPersistIDList
    let mut ppersidl: *mut IPersistIDList = null_mut();
    let mut ret = ((*(*punk).lpVtbl).QueryInterface)(
        punk,
        &IID_IPersistIDList,
        &mut ppersidl as *mut *mut IPersistIDList as *mut *mut c_void,
    );
    if SUCCEEDED(ret) {
        ret = ((*(*ppersidl).lpVtbl).GetIDList)(ppersidl, ppidl);
        ((*(*ppersidl).lpVtbl).Release)(ppersidl);
        if SUCCEEDED(ret) {
            return ret;
        }
    }

    // Try IPersistFolder2
    let mut ppf2: *mut IPersistFolder2 = null_mut();
    ret = ((*(*punk).lpVtbl).QueryInterface)(
        punk,
        &IID_IPersistFolder2,
        &mut ppf2 as *mut *mut IPersistFolder2 as *mut *mut c_void,
    );
    if SUCCEEDED(ret) {
        ret = ((*(*ppf2).lpVtbl).GetCurFolder)(ppf2, ppidl);
        ((*(*ppf2).lpVtbl).Release)(ppf2);
        if SUCCEEDED(ret) {
            return ret;
        }
    }

    // Try IDataObject
    let mut pdo: *mut IDataObject = null_mut();
    ret = ((*(*punk).lpVtbl).QueryInterface)(
        punk,
        &IID_IDataObject,
        &mut pdo as *mut *mut IDataObject as *mut *mut c_void,
    );
    if SUCCEEDED(ret) {
        let mut psi: *mut IShellItem = null_mut();
        ret = SHGetItemFromDataObject(
            pdo,
            DOGIF_ONLY_IF_ONE,
            &IID_IShellItem,
            &mut psi as *mut *mut IShellItem as *mut *mut c_void,
        );
        if SUCCEEDED(ret) {
            ret = SHGetIDListFromObject(psi as *mut IUnknown, ppidl);
            ((*(*psi).lpVtbl).Release)(psi);
        }
        ((*(*pdo).lpVtbl).Release)(pdo);
        if SUCCEEDED(ret) {
            return ret;
        }
    }

    // Try IFolderView
    let mut pfv: *mut IFolderView = null_mut();
    ret = ((*(*punk).lpVtbl).QueryInterface)(
        punk,
        &IID_IFolderView,
        &mut pfv as *mut *mut IFolderView as *mut *mut c_void,
    );
    if SUCCEEDED(ret) {
        let mut psf: LPSHELLFOLDER = null_mut();
        ret = ((*(*pfv).lpVtbl).GetFolder)(
            pfv,
            &IID_IShellFolder,
            &mut psf as *mut LPSHELLFOLDER as *mut *mut c_void,
        );
        if SUCCEEDED(ret) {
            ret = SHGetIDListFromObject(psf as *mut IUnknown, ppidl);
        }
        ((*(*pfv).lpVtbl).Release)(pfv);
        return ret;
    }

    ret
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  internal functions — 1. creating pidls
// ════════════════════════════════════════════════════════════════════════════════════════
/// _ILAlloc (pidl.c:1768) — static. Allocates size+5; cb = size+3; sets type; terminates.
unsafe fn _il_alloc(typ: PIDLTYPE, size: u32) -> LPITEMIDLIST {
    let pidl_out = SHAlloc((size as SIZE_T) + 5) as LPITEMIDLIST;
    if !pidl_out.is_null() {
        r_memset(pidl_out as *mut u8, 0, (size as usize) + 5);
        il_set_cb(pidl_out, (size + 3) as WORD);

        let p_data = _ILGetDataPointer(pidl_out);
        if !p_data.is_null() {
            *(p_data as *mut u8) = typ; // pData->type
        }
        let pidl_next = ILGetNext(pidl_out);
        if !pidl_next.is_null() {
            il_set_cb(pidl_next, 0);
        }
    }
    pidl_out
}

/// _ILCreateDesktop (pidl.c:1794).
#[no_mangle]
pub unsafe extern "C" fn _ILCreateDesktop() -> LPITEMIDLIST {
    let ret = SHAlloc(2) as LPITEMIDLIST;
    if !ret.is_null() {
        il_set_cb(ret, 0);
    }
    ret
}

/// _ILCreateMyComputer (pidl.c:1805).
#[no_mangle]
pub unsafe extern "C" fn _ILCreateMyComputer() -> LPITEMIDLIST {
    _ILCreateGuid(PT_DESKTOP_REGITEM, &CLSID_MyComputer, REGITEMORDER_MYCOMPUTER)
}

/// _ILCreateMyDocuments (pidl.c:1811).
#[no_mangle]
pub unsafe extern "C" fn _ILCreateMyDocuments() -> LPITEMIDLIST {
    _ILCreateGuid(PT_DESKTOP_REGITEM, &CLSID_MyDocuments, REGITEMORDER_MYDOCS_DEFAULT)
}

/// _ILCreateIExplore (pidl.c:1817).
#[no_mangle]
pub unsafe extern "C" fn _ILCreateIExplore() -> LPITEMIDLIST {
    _ILCreateGuid(PT_DESKTOP_REGITEM, &CLSID_Internet, REGITEMORDER_INTERNET)
}

/// _ILCreateControlPanel (pidl.c:1823).
#[no_mangle]
pub unsafe extern "C" fn _ILCreateControlPanel() -> LPITEMIDLIST {
    let parent = _ILCreateMyComputer();
    let mut ret: LPITEMIDLIST = null_mut();
    if !parent.is_null() {
        let cpl = _ILCreateGuid(PT_COMPUTER_REGITEM, &CLSID_ControlPanel, REGITEMORDER_MYCOMPUTER_CONTROLS);
        if !cpl.is_null() {
            ret = ILCombine(parent, cpl);
            SHFree(cpl as LPVOID);
        }
        SHFree(parent as LPVOID);
    }
    ret
}

/// _ILCreatePrinters (pidl.c:1841) — __REACTOS__ path.
#[no_mangle]
pub unsafe extern "C" fn _ILCreatePrinters() -> LPITEMIDLIST {
    // Note: Wine returns the Win95 PIDL; NT5 moved it into CSIDL_CONTROLS.
    let mut pidl: LPITEMIDLIST = null_mut();
    SHGetFolderLocationHelper(null_mut(), CSIDL_CONTROLS, &CLSID_Printers, &mut pidl);
    pidl
}

/// _ILCreateNetwork (pidl.c:1868).
#[no_mangle]
pub unsafe extern "C" fn _ILCreateNetwork() -> LPITEMIDLIST {
    _ILCreateGuid(PT_DESKTOP_REGITEM, &CLSID_NetworkPlaces, REGITEMORDER_NETHOOD)
}

/// _ILCreateBitBucket (pidl.c:1874).
#[no_mangle]
pub unsafe extern "C" fn _ILCreateBitBucket() -> LPITEMIDLIST {
    _ILCreateGuid(PT_DESKTOP_REGITEM, &CLSID_RecycleBin, REGITEMORDER_RECYCLEBIN)
}

/// _ILCreateAdminTools (pidl.c:1880).
#[no_mangle]
pub unsafe extern "C" fn _ILCreateAdminTools() -> LPITEMIDLIST {
    _ILCreateGuid(PT_GUID, &CLSID_AdminFolderShortcut, REGITEMORDER_DEFAULT) // FIXME
}

/// _ILCreateGuid (pidl.c:1886).
#[no_mangle]
pub unsafe extern "C" fn _ILCreateGuid(typ: PIDLTYPE, guid: REFIID, SortOrder: BYTE) -> LPITEMIDLIST {
    if typ == PT_SHELLEXT || typ == PT_GUID || typ == PT_YAGUID {
        let pidl_out = _il_alloc(typ, size_of::<GUIDStruct>() as u32);
        if !pidl_out.is_null() {
            let p_data = _ILGetDataPointer(pidl_out) as *mut u8;
            // pData->u.guid.uSortOrder (PIDLDATA+1) ; pData->u.guid.guid (PIDLDATA+2)
            *p_data.add(1) = SortOrder;
            r_memcpy(p_data.add(2), guid as *const u8, size_of::<GUID>());
        }
        pidl_out
    } else {
        null_mut()
    }
}

/// _ILCreateFromFindDataW (pidl.c:1940).
#[no_mangle]
pub unsafe extern "C" fn _ILCreateFromFindDataW(wfd: *const WIN32_FIND_DATAW) -> LPITEMIDLIST {
    if wfd.is_null() {
        return null_mut();
    }

    let mut buff: [CHAR; MAX_PATH + 14 + 1] = [0; MAX_PATH + 14 + 1];

    // prepare buffer with both names
    let len = WideCharToMultiByte(
        CP_ACP,
        0,
        addr_of!((*wfd).cFileName) as *const WCHAR,
        -1,
        buff.as_mut_ptr(),
        MAX_PATH as INT,
        null(),
        null_mut(),
    );
    let len1 = WideCharToMultiByte(
        CP_ACP,
        0,
        addr_of!((*wfd).cAlternateFileName) as *const WCHAR,
        -1,
        buff.as_mut_ptr().add(len as usize),
        buff.len() as INT - len,
        null(),
        null_mut(),
    );
    let alen: u32 = (len + len1) as u32;

    let typ: PIDLTYPE = if ((*wfd).dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
        PT_FOLDER
    } else {
        PT_VALUE
    };

    let wlen: u32 = lstrlenW(addr_of!((*wfd).cFileName) as *const WCHAR) as u32 + 1;
    // size = FIELD_OFFSET(FileStruct, szNames[alen + (alen&1)])  (= 11 + ..)
    //      + FIELD_OFFSET(FileStructW, wszName[wlen])            (= 20 + 2*wlen)
    //      + sizeof(WORD)
    let size: u32 = (11 + alen + (alen & 1)) + (20 + 2 * wlen) + size_of::<WORD>() as u32;

    let pidl = _il_alloc(typ, size);
    if !pidl.is_null() {
        // pData = _ILGetDataPointer(pidl); fs = &pData->u.file  (PIDLDATA-relative offsets used)
        let p_data = _ILGetDataPointer(pidl) as *mut u8;

        // FileTimeToDosDateTime(&wfd->ftLastWriteTime, &fs->uFileDate(+6), &fs->uFileTime(+8))
        FileTimeToDosDateTime(
            addr_of!((*wfd).ftLastWriteTime),
            p_data.add(6) as *mut WORD,
            p_data.add(8) as *mut WORD,
        );
        // fs->dwFileSize (+2) = nFileSizeLow ; fs->uFileAttribs (+10) = dwFileAttributes
        wr_u32(p_data.add(2), (*wfd).nFileSizeLow);
        wr_u16(p_data.add(10), (*wfd).dwFileAttributes as WORD);
        // memcpy(fs->szNames(+12), buff, alen)
        r_memcpy(p_data.add(12), buff.as_ptr() as *const u8, alen as usize);

        // fsw = (FileStructW*)(pData->u.file.szNames + alen + (alen & 1))
        let fsw = p_data.add(12 + (alen + (alen & 1)) as usize);
        // fsw->cbLen = FIELD_OFFSET(FileStructW, wszName[wlen]) + sizeof(WORD)
        wr_u16(fsw, ((20 + 2 * wlen) + size_of::<WORD>() as u32) as WORD);
        FileTimeToDosDateTime(
            addr_of!((*wfd).ftCreationTime),
            fsw.add(8) as *mut WORD,
            fsw.add(10) as *mut WORD,
        );
        FileTimeToDosDateTime(
            addr_of!((*wfd).ftLastAccessTime),
            fsw.add(12) as *mut WORD,
            fsw.add(14) as *mut WORD,
        );
        // memcpy(fsw->wszName(+20), wfd->cFileName, wlen*sizeof(WCHAR))
        r_memcpy(
            fsw.add(20), // nolint: magic-offset (Wine FileStructW layout, matches Wine C)
            addr_of!((*wfd).cFileName) as *const u8,
            (wlen * 2) as usize,
        );

        // pOffsetW = (WORD*)((LPBYTE)pidl + pidl->mkid.cb - sizeof(WORD)); *pOffsetW = fsw - pidl
        let cb = il_cb(pidl) as usize;
        let p_offset_w = (pidl as *mut u8).add(cb - size_of::<WORD>());
        wr_u16(p_offset_w, ((fsw as usize) - (pidl as usize)) as WORD);
    }
    pidl
}

/// _ILCreateFromPathW (pidl.c:1988).
#[no_mangle]
pub unsafe extern "C" fn _ILCreateFromPathW(szPath: LPCWSTR, ppidl: *mut LPITEMIDLIST) -> HRESULT {
    if ppidl.is_null() {
        return E_INVALIDARG;
    }

    let mut stffile: WIN32_FIND_DATAW = zeroed();
    let h_file = FindFirstFileW(szPath, (&raw mut stffile).cast());
    let invalid = usize::MAX as HANDLE; // INVALID_HANDLE_VALUE
    if h_file == invalid {
        return HRESULT_FROM_WIN32(ERROR_FILE_NOT_FOUND);
    }
    FindClose(h_file);

    let p = _ILCreateFromFindDataW(&stffile);
    write_volatile(ppidl, p);

    if !p.is_null() {
        S_OK
    } else {
        E_OUTOFMEMORY
    }
}

/// _ILCreateDrive (pidl.c:2007).
#[no_mangle]
pub unsafe extern "C" fn _ILCreateDrive(lpszNew: LPCWSTR) -> LPITEMIDLIST {
    let pidl_out = _il_alloc(PT_DRIVE, size_of::<DriveStruct>() as u32);
    if !pidl_out.is_null() {
        let psz_dest = _ILGetTextPointer(pidl_out);
        if !psz_dest.is_null() {
            r_strcpy(psz_dest, DRIVE_TEMPLATE.as_ptr());
            *psz_dest = r_toupper(rd_u16(lpszNew as *const u8) as i32) as CHAR; // toupper(lpszNew[0])
        }
    }
    pidl_out
}

/// _ILCreateEntireNetwork (pidl.c:2027).
#[no_mangle]
pub unsafe extern "C" fn _ILCreateEntireNetwork() -> LPITEMIDLIST {
    // FIELD_OFFSET(PIDLDATA, u.network.szNames[sizeof("Entire Network")]) = 3 + 15
    let pidl_out = _il_alloc(PT_NETWORK, 3 + 15);
    if !pidl_out.is_null() {
        let p_data = _ILGetDataPointer(pidl_out) as *mut u8;
        wr_u16(p_data.add(1), 0); // u.network.dummy
        r_strcpy(p_data.add(3) as *mut CHAR, ENTIRE_NETWORK.as_ptr()); // u.network.szNames
    }
    pidl_out
}

/// _ILGetDrive (pidl.c:2052).
#[no_mangle]
pub unsafe extern "C" fn _ILGetDrive(pidl: LPCITEMIDLIST, pOut: LPWSTR, uSize: UINT) -> DWORD {
    let mut pidl = pidl;
    if _ILIsMyComputer(pidl) != FALSE {
        pidl = ILGetNext(pidl);
    }
    if !pidl.is_null() && _ILIsDrive(pidl) != FALSE {
        return _ILSimpleGetTextW(pidl, pOut, uSize);
    }
    0
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  internal functions — 2. testing pidls
// ════════════════════════════════════════════════════════════════════════════════════════
/// _ILIsUnicode (pidl.c:2079).
#[no_mangle]
pub unsafe extern "C" fn _ILIsUnicode(pidl: LPCITEMIDLIST) -> BOOL {
    if (_il_get_fs_type(pidl) & PT_FS_UNICODE_FLAG) != 0 {
        TRUE
    } else {
        FALSE
    }
}

/// _ILIsDesktop (pidl.c:2086).
#[no_mangle]
pub unsafe extern "C" fn _ILIsDesktop(pidl: LPCITEMIDLIST) -> BOOL {
    if pidl.is_null() || il_cb(pidl) == 0 {
        TRUE
    } else {
        FALSE
    }
}

/// _ILIsMyDocuments (pidl.c:2093).
#[no_mangle]
pub unsafe extern "C" fn _ILIsMyDocuments(pidl: LPCITEMIDLIST) -> BOOL {
    let iid = _ILGetGUIDPointer(pidl);
    if !iid.is_null() && is_equal_iid(iid, &CLSID_MyDocuments) {
        TRUE
    } else {
        FALSE
    }
}

/// _ILIsMyComputer (pidl.c:2104).
#[no_mangle]
pub unsafe extern "C" fn _ILIsMyComputer(pidl: LPCITEMIDLIST) -> BOOL {
    let iid = _ILGetGUIDPointer(pidl);
    if !iid.is_null() && is_equal_iid(iid, &CLSID_MyComputer) {
        TRUE
    } else {
        FALSE
    }
}

/// _ILIsBitBucket (pidl.c:2115).
#[no_mangle]
pub unsafe extern "C" fn _ILIsBitBucket(pidl: LPCITEMIDLIST) -> BOOL {
    let iid = _ILGetGUIDPointer(pidl);
    if !iid.is_null() && is_equal_iid(iid, &CLSID_RecycleBin) {
        TRUE
    } else {
        FALSE
    }
}

/// _ILIsSpecialFolder (pidl.c:2126).
#[no_mangle]
pub unsafe extern "C" fn _ILIsSpecialFolder(pidl: LPCITEMIDLIST) -> BOOL {
    let lp_pdata = _ILGetDataPointer(pidl) as *const u8;
    let cond = !pidl.is_null()
        && ((!lp_pdata.is_null() && {
            let t = *lp_pdata;
            t == PT_GUID || t == PT_SHELLEXT || t == PT_YAGUID
        }) || (!pidl.is_null() && il_cb(pidl) == 0));
    if cond {
        TRUE
    } else {
        FALSE
    }
}

/// _ILIsDrive (pidl.c:2137).
#[no_mangle]
pub unsafe extern "C" fn _ILIsDrive(pidl: LPCITEMIDLIST) -> BOOL {
    let typ = _il_get_type(pidl);
    let fldrtype = PT_DRIVE & PT_FOLDERTYPEMASK; // 0x20
    if (typ & PT_FOLDERTYPEMASK) == fldrtype && typ != PT_COMPUTER_REGITEM {
        TRUE
    } else {
        FALSE
    }
}

/// _ILIsFolder (pidl.c:2144).
#[no_mangle]
pub unsafe extern "C" fn _ILIsFolder(pidl: LPCITEMIDLIST) -> BOOL {
    let typ = _il_get_fs_type(pidl);
    if (typ & PT_FS_FOLDER_FLAG) != 0 || (typ == PT_FS && !ILGetNext(pidl).is_null()) {
        TRUE
    } else {
        FALSE
    }
}

/// _ILIsValue (pidl.c:2151).
#[no_mangle]
pub unsafe extern "C" fn _ILIsValue(pidl: LPCITEMIDLIST) -> BOOL {
    let typ = _il_get_fs_type(pidl);
    if typ != 0 && (typ & PT_FS_FOLDER_FLAG) == 0 {
        TRUE
    } else {
        FALSE
    }
}

/// _ILIsCPanelStruct (pidl.c:2157).
#[no_mangle]
pub unsafe extern "C" fn _ILIsCPanelStruct(pidl: LPCITEMIDLIST) -> BOOL {
    let lp_pdata = _ILGetDataPointer(pidl) as *const u8;
    if !pidl.is_null() && !lp_pdata.is_null() && *lp_pdata == 0 {
        TRUE
    } else {
        FALSE
    }
}

/// _ILIsPidlSimple (pidl.c:2169).
#[no_mangle]
pub unsafe extern "C" fn _ILIsPidlSimple(pidl: LPCITEMIDLIST) -> BOOL {
    let mut ret = TRUE;
    if _ILIsDesktop(pidl) == FALSE {
        let len = il_cb(pidl);
        let pidlnext = (pidl as *const u8).add(len as usize) as LPCITEMIDLIST;
        if il_cb(pidlnext) != 0 {
            ret = FALSE;
        }
    }
    ret
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  internal functions — 3. getting values from pidls
// ════════════════════════════════════════════════════════════════════════════════════════
/// _ILSimpleGetTextW (pidl.c:2198).
#[no_mangle]
pub unsafe extern "C" fn _ILSimpleGetTextW(pidl: LPCITEMIDLIST, szOut: LPWSTR, uOutSize: UINT) -> DWORD {
    let mut dw_return: DWORD;
    let p_file_struct_w = _ILGetFileStructW(pidl);

    if !p_file_struct_w.is_null() {
        let wsz_name = (p_file_struct_w as *const u8).add(20) as *const WCHAR; // FileStructW.wszName // nolint: magic-offset (Wine FileStructW layout, matches Wine C)
        lstrcpynW(szOut, wsz_name, uOutSize as INT);
        dw_return = lstrlenW(wsz_name) as DWORD;
    } else {
        dw_return = 0;
        if pidl.is_null() {
            return 0;
        }
        if !szOut.is_null() {
            *szOut = 0;
        }

        let mut sz_temp: [WCHAR; MAX_PATH] = [0; MAX_PATH];
        if _ILIsDesktop(pidl) != FALSE {
            // desktop
            if HCR_GetClassNameW(&CLSID_ShellDesktop, sz_temp.as_mut_ptr(), MAX_PATH as DWORD) != FALSE {
                if !szOut.is_null() {
                    lstrcpynW(szOut, sz_temp.as_ptr(), uOutSize as INT);
                }
                dw_return = lstrlenW(sz_temp.as_ptr()) as DWORD;
            }
        } else {
            let sz_src_w = _il_get_text_pointer_w(pidl);
            if !sz_src_w.is_null() {
                // unicode filesystem
                if !szOut.is_null() {
                    lstrcpynW(szOut, sz_src_w, uOutSize as INT);
                }
                dw_return = lstrlenW(sz_src_w) as DWORD;
            } else {
                let sz_src = _ILGetTextPointer(pidl);
                if !sz_src.is_null() {
                    // filesystem
                    MultiByteToWideChar(CP_ACP, 0, sz_src, -1, sz_temp.as_mut_ptr(), MAX_PATH as INT);
                    if !szOut.is_null() {
                        lstrcpynW(szOut, sz_temp.as_ptr(), uOutSize as INT);
                    }
                    dw_return = lstrlenW(sz_temp.as_ptr()) as DWORD;
                } else {
                    let riid = _ILGetGUIDPointer(pidl);
                    if !riid.is_null() {
                        // special folder
                        if HCR_GetClassNameW(riid, sz_temp.as_mut_ptr(), MAX_PATH as DWORD) != FALSE {
                            if !szOut.is_null() {
                                lstrcpynW(szOut, sz_temp.as_ptr(), uOutSize as INT);
                            }
                            dw_return = lstrlenW(sz_temp.as_ptr()) as DWORD;
                        }
                    }
                    // else: ERR no text (dropped)
                }
            }
        }
    }
    dw_return
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  internal functions — 4. getting pointers to parts of pidls
// ════════════════════════════════════════════════════════════════════════════════════════
/// _ILGetDataPointer (pidl.c:2278).
#[no_mangle]
pub unsafe extern "C" fn _ILGetDataPointer(pidl: LPCITEMIDLIST) -> LPPIDLDATA {
    if !_il_is_empty(pidl) {
        return (pidl as *const u8).add(2) as LPPIDLDATA; // &pidl->mkid.abID
    }
    null_mut()
}

/// _ILGetTextPointerW (pidl.c:2289) — static.
unsafe fn _il_get_text_pointer_w(pidl: LPCITEMIDLIST) -> LPWSTR {
    let pdata = _ILGetDataPointer(pidl) as *mut u8;
    if pdata.is_null() {
        return null_mut();
    }

    if (_il_get_fs_type(pidl) & PT_FS_UNICODE_FLAG) != 0 {
        return pdata.add(12) as LPWSTR; // (LPWSTR)pdata->u.file.szNames
    }

    match *pdata {
        // PT_CPLAPPLET (__REACTOS__ r54423): pdata->u.cpanel.szName
        PT_CPLAPPLET => pdata.add(10) as LPWSTR,
        // PT_GUID/PT_SHELLEXT/PT_YAGUID, drives, IESPECIAL, network → NULL (and default)
        _ => null_mut(),
    }
}

/// _ILGetTextPointer (pidl.c:2342).
#[no_mangle]
pub unsafe extern "C" fn _ILGetTextPointer(pidl: LPCITEMIDLIST) -> LPSTR {
    let pdata = _ILGetDataPointer(pidl) as *mut u8;
    if pdata.is_null() {
        return null_mut();
    }

    let typ = _il_get_fs_type(pidl);
    if typ != 0 && (typ & PT_FS_UNICODE_FLAG) == 0 {
        return pdata.add(12) as LPSTR; // pdata->u.file.szNames
    }

    if _ILIsDrive(pidl) != FALSE {
        return pdata.add(1) as LPSTR; // pdata->u.drive.szDriveName
    }

    match *pdata {
        PT_GUID | PT_SHELLEXT | PT_YAGUID => null_mut(),
        PT_IESPECIAL1 | PT_IESPECIAL2 => pdata.add(12) as LPSTR, // pdata->u.file.szNames
        PT_WORKGRP | PT_COMP | PT_NETWORK | PT_NETPROVIDER | PT_SHARE => pdata.add(3) as LPSTR, // u.network.szNames
        _ => null_mut(),
    }
}

/// _ILGetSTextPointer (pidl.c:2383) — static.
unsafe fn _il_get_stext_pointer(pidl: LPCITEMIDLIST) -> LPSTR {
    let pdata = _ILGetDataPointer(pidl) as *mut u8;
    if pdata.is_null() {
        return null_mut();
    }

    let mut typ = *pdata; // pdata->type
    if _il_get_fs_type(pidl) != 0 && (typ & PT_FS_UNICODE_FLAG) == 0 {
        typ = PT_FS;
    }

    match typ {
        PT_FS | PT_IESPECIAL1 | PT_IESPECIAL2 => {
            let base = pdata.add(12) as *mut CHAR; // u.file.szNames
            base.add(r_strlen(base) + 1)
        }
        PT_WORKGRP => {
            let base = pdata.add(3) as *mut CHAR; // u.network.szNames
            base.add(r_strlen(base) + 1)
        }
        _ => null_mut(),
    }
}

/// _ILGetSTextPointer thin alias for callers within this file.
#[inline]
unsafe fn _ILGetSTextPointer(pidl: LPCITEMIDLIST) -> LPSTR {
    _il_get_stext_pointer(pidl)
}

/// _ILGetGUIDPointer (pidl.c:2414).
#[no_mangle]
pub unsafe extern "C" fn _ILGetGUIDPointer(pidl: LPCITEMIDLIST) -> *mut IID {
    let pdata = _ILGetDataPointer(pidl) as *mut u8;
    if pdata.is_null() {
        return null_mut();
    }
    match *pdata {
        PT_SHELLEXT | PT_GUID | PT_YAGUID => pdata.add(2) as *mut IID, // &pdata->u.guid.guid
        _ => null_mut(),
    }
}

/// _ILGetFileStructW (pidl.c:2450).
#[no_mangle]
pub unsafe extern "C" fn _ILGetFileStructW(pidl: LPCITEMIDLIST) -> *mut FileStructW {
    if _il_get_fs_type(pidl) == 0 {
        return null_mut(); // !_ILIsFolderOrFile
    }

    let pidl_b = pidl as *const u8;
    let cb = il_cb(pidl);
    let cb_offset = rd_u16(pidl_b.add(cb as usize - size_of::<WORD>()));
    let p_file_struct_w = pidl_b.add(cb_offset as usize) as *mut u8;

    // Sanity checks (matches the C, short-circuited so cbLen is read only when in range):
    if (cb_offset & 0x1) != 0
        || (cb_offset as usize) < size_of::<WORD>() + size_of::<PIDLTYPE>() + size_of::<FileStruct>()
        || (cb_offset as usize)
            > (cb as usize)
                .wrapping_sub(size_of::<WORD>())
                .wrapping_sub(size_of::<FileStructW>())
        || (cb as usize) != (cb_offset as usize) + rd_u16(p_file_struct_w) as usize
    {
        return null_mut();
    }

    p_file_struct_w as *mut FileStructW
}

/// _ILGetFileDateTime (pidl.c:2492).
#[no_mangle]
pub unsafe extern "C" fn _ILGetFileDateTime(pidl: LPCITEMIDLIST, pFt: *mut FILETIME) -> BOOL {
    if _il_get_fs_type(pidl) > PT_FS {
        let pdata = _ILGetDataPointer(pidl) as *const u8;
        return DosDateTimeToFileTime(rd_u16(pdata.add(6)), rd_u16(pdata.add(8)), pFt); // uFileDate, uFileTime
    }
    FALSE
}

/// _ILGetFileSize (pidl.c:2519).
#[no_mangle]
pub unsafe extern "C" fn _ILGetFileSize(pidl: LPCITEMIDLIST, pOut: LPWSTR, uOutSize: UINT) -> DWORD {
    let pdata = _ILGetDataPointer(pidl) as *const u8;
    if pdata.is_null() {
        return 0;
    }

    if (_il_get_fs_type(pidl) & PT_FS_FILE_FLAG) != 0 {
        let dw_size = rd_u32(pdata.add(2)); // u.file.dwFileSize
        if !pOut.is_null() {
            StrFormatKBSizeW(dw_size as i64, pOut, uOutSize);
        }
        return dw_size;
    }
    if !pOut.is_null() {
        *pOut = UNICODE_NULL;
    }
    0
}

/// _ILGetExtension (pidl.c:2538).
#[no_mangle]
pub unsafe extern "C" fn _ILGetExtension(pidl: LPCITEMIDLIST, pOut: LPWSTR, uOutSize: UINT) -> BOOL {
    if pidl.is_null() {
        return FALSE;
    }

    let pidl_temp = ILFindLastID(pidl);

    if _ILIsValue(pidl_temp) == FALSE {
        return FALSE;
    }

    let mut sz_temp: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    if _ILSimpleGetTextW(pidl_temp, sz_temp.as_mut_ptr(), MAX_PATH as UINT) == 0 {
        return FALSE;
    }

    let mut p_point = PathFindExtensionW(sz_temp.as_ptr());
    if *p_point == 0 {
        return FALSE;
    }

    p_point = p_point.add(1);
    lstrcpynW(pOut, p_point, uOutSize as INT);
    TRUE
}

/// _ILGetFileAttributes (pidl.c:2585).
#[no_mangle]
pub unsafe extern "C" fn _ILGetFileAttributes(pidl: LPCITEMIDLIST, pOut: LPWSTR, uOutSize: UINT) -> DWORD {
    let mut w_attrib: DWORD = 0;
    if _il_get_fs_type(pidl) != 0 {
        let pdata = _ILGetDataPointer(pidl) as *const u8;
        w_attrib = rd_u16(pdata.add(10)) as DWORD; // u.file.uFileAttribs
    }

    if uOutSize >= 6 {
        let mut i: usize = 0;
        if w_attrib & FILE_ATTRIBUTE_READONLY != 0 {
            *pOut.add(i) = 'R' as WCHAR;
            i += 1;
        }
        if w_attrib & FILE_ATTRIBUTE_HIDDEN != 0 {
            *pOut.add(i) = 'H' as WCHAR;
            i += 1;
        }
        if w_attrib & FILE_ATTRIBUTE_SYSTEM != 0 {
            *pOut.add(i) = 'S' as WCHAR;
            i += 1;
        }
        if w_attrib & FILE_ATTRIBUTE_ARCHIVE != 0 {
            *pOut.add(i) = 'A' as WCHAR;
            i += 1;
        }
        if w_attrib & FILE_ATTRIBUTE_COMPRESSED != 0 {
            *pOut.add(i) = 'C' as WCHAR;
            i += 1;
        }
        *pOut.add(i) = UNICODE_NULL;
    }
    w_attrib
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  aPidl helpers
// ════════════════════════════════════════════════════════════════════════════════════════
/// _ILFreeaPidl (pidl.c:2614).
#[no_mangle]
pub unsafe extern "C" fn _ILFreeaPidl(apidl: *mut LPITEMIDLIST, cidl: UINT) {
    if !apidl.is_null() {
        let mut i: UINT = 0;
        while i < cidl {
            SHFree(*apidl.add(i as usize) as LPVOID);
            i += 1;
        }
        SHFree(apidl as LPVOID);
    }
}

/// _ILCopyaPidl (pidl.c:2629).
#[no_mangle]
pub unsafe extern "C" fn _ILCopyaPidl(apidlsrc: *const LPCITEMIDLIST, cidl: UINT) -> *mut LPITEMIDLIST {
    if apidlsrc.is_null() {
        return null_mut();
    }

    let apidldest = SHAlloc(cidl as SIZE_T * size_of::<LPITEMIDLIST>() as SIZE_T) as *mut LPITEMIDLIST;
    if apidldest.is_null() {
        return null_mut(); // C deref's unconditionally; guard to avoid UB on OOM
    }

    let mut i: UINT = 0;
    while i < cidl {
        let clone = ILClone(*apidlsrc.add(i as usize));
        *apidldest.add(i as usize) = clone;
        if clone.is_null() {
            _ILFreeaPidl(apidldest, i);
            return null_mut();
        }
        i += 1;
    }
    apidldest
}

/// _ILCopyCidaToaPidl (pidl.c:2654).
#[no_mangle]
pub unsafe extern "C" fn _ILCopyCidaToaPidl(pidl: *mut LPITEMIDLIST, cida: *const CIDA) -> *mut LPITEMIDLIST {
    let cidl = rd_u32(cida as *const u8); // cida->cidl
    let dst = SHAlloc(cidl as SIZE_T * size_of::<LPITEMIDLIST>() as SIZE_T) as *mut LPITEMIDLIST;
    if dst.is_null() {
        return null_mut();
    }

    let mut i: UINT = 0;
    while i < cidl {
        let clone = ILClone(hida_get_pidl_item(cida, i as usize));
        *dst.add(i as usize) = clone;
        if clone.is_null() {
            _ILFreeaPidl(dst, i);
            return null_mut();
        }
        i += 1;
    }

    if !pidl.is_null() {
        let c = ILClone(hida_get_pidl_folder(cida));
        write_volatile(pidl, c);
        if c.is_null() {
            _ILFreeaPidl(dst, cidl);
            return null_mut();
        }
    }
    dst
}

// HIDA helpers (shellutils.h:708/713).
#[inline]
unsafe fn cida_aoffset(cida: *const CIDA, k: usize) -> u32 {
    rd_u32((cida as *const u8).add(4 + 4 * k)) // cida->aoffset[k]
}
#[inline]
unsafe fn hida_get_pidl_folder(cida: *const CIDA) -> LPCITEMIDLIST {
    (cida as *const u8).add(cida_aoffset(cida, 0) as usize) as LPCITEMIDLIST
}
#[inline]
unsafe fn hida_get_pidl_item(cida: *const CIDA, i: usize) -> LPCITEMIDLIST {
    (cida as *const u8).add(cida_aoffset(cida, i + 1) as usize) as LPCITEMIDLIST
}

// ADDED-TO-TYPES?: the entire shellcom fence (PIDL/STRRET/GUID/VARIANT/SAFEARRAY/SHCOLUMNID +
// the COM vtables) is promoted to crate::shellcom. WIN32_FIND_DATA{A,W}/CIDA/FILETIME are
// general-win32 candidates for types.rs (FILETIME currently also in wine/shellole.rs).
