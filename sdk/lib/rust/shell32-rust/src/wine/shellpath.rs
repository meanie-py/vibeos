//! Path functions + CSIDL special-folder resolution — translation of
//! dll/win32/shell32/wine/shellpath.c (Wave 4, against `crate::shellcom`).
//!
//! Build config: `-D__REACTOS__ -D_SHELL32_ -D_WINE -DUNICODE -D_UNICODE
//! -D_WIN32_WINNT=0x600` (→ `NTDDI_VERSION = NTDDI_WIN6 = NTDDI_VISTA`). The file then
//! `#undef`s `_WIN32_WINNT` and redefines it to `_WIN32_WINNT_WS03` (0x0502), so every
//! `_WIN32_WINNT >= _WIN32_WINNT_WS03` guard is TRUE (PathIsAbsoluteW / PathMakeAbsoluteW
//! and the PRF_REQUIREABSOLUTE tail of PathResolveW are compiled). `NTDDI_VERSION` is NOT
//! changed by that `#undef`, so `NTDDI_VERSION < NTDDI_VISTA` stays FALSE (the extra
//! `|| pszDir` clause in PathMakeUniqueNameW is omitted). Only `__REACTOS__` branches are
//! translated; the Wine-only (`#ifndef __REACTOS__`) code is omitted. Verified against the
//! pre-flip object's import table.
//!
//! Decoration: WINAPI/APIENTRY exports are `extern "system"` (`_Name@N`); the plain-cdecl
//! exports (`PathIsExeW`, `SHELL_RegisterShellFolders`, `SHGetFolderLocationHelper`,
//! `SHGetSpecialFolderID`, `Shell_ParseSpecialFolder`, `_SHGetUserProfileDirectoryW`) are
//! `extern "C"` (`_Name`, and the leading-underscore source name `_SHGetUserProfileDirectoryW`
//! becomes `__SHGetUserProfileDirectoryW`). Debug machinery (TRACE/WARN/ERR/FIXME and the
//! `assert` in IsRemovableDrive) is dropped — no observable behavior.

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use crate::shellcom::*;
use crate::types::*;
use core::ffi::c_void;
use core::ptr::{addr_of, addr_of_mut, null, null_mut, write_volatile};

// ════════════════════════════════════════════════════════════════════════════════════════
//  constants pinned from SDK headers
// ════════════════════════════════════════════════════════════════════════════════════════
const MAX_PATH: usize = 260; // minwindef.h

// HRESULTs (winerror.h)
const E_FAIL: HRESULT = 0x80004005u32 as HRESULT;
const E_INVALIDARG: HRESULT = 0x80070057u32 as HRESULT;
// E_NOT_SUFFICIENT_BUFFER == HRESULT_FROM_WIN32(ERROR_INSUFFICIENT_BUFFER=122) == 0x8007007A
const E_NOT_SUFFICIENT_BUFFER: HRESULT = 0x8007007Au32 as HRESULT;
// (S_OK and S_FALSE: S_OK from crate::types; S_FALSE below)
const S_FALSE: HRESULT = 1; // winerror.h
// STRSAFE error returned by the StringCch* helpers (strsafe.h)
const STRSAFE_E_INSUFFICIENT_BUFFER: HRESULT = 0x8007007Au32 as HRESULT;
const STRSAFE_E_INVALID_PARAMETER: HRESULT = 0x80070057u32 as HRESULT;

// Win32 error codes (winerror.h)
const ERROR_SUCCESS_L: LONG = 0;
const ERROR_FILE_NOT_FOUND: LONG = 2;
const ERROR_PATH_NOT_FOUND: LONG = 3;
const ERROR_NOT_ENOUGH_MEMORY: LONG = 8;
const ERROR_INSUFFICIENT_BUFFER: DWORD = 122;
const ERROR_ALREADY_EXISTS: INT = 183;
const ERROR_FILENAME_EXCED_RANGE: LONG = 206;

const UNICODE_NULL: WCHAR = 0;
const CP_ACP: UINT = 0; // winnls.h

// registry (winreg.h / winnt.h)
const REG_SZ: DWORD = 1;
const REG_EXPAND_SZ: DWORD = 2;
const KEY_READ: REGSAM = 0x20019;
const RRF_RT_REG_SZ: DWORD = 1 << 1; // winreg.h

// HKEY predefined handles (winreg.h)
const HKEY_CLASSES_ROOT: HKEY = 0x80000000u32 as usize as HKEY;
const HKEY_CURRENT_USER: HKEY = 0x80000001u32 as usize as HKEY;
const HKEY_LOCAL_MACHINE: HKEY = 0x80000002u32 as usize as HKEY;
const HKEY_USERS: HKEY = 0x80000003u32 as usize as HKEY;

// token (winnt.h)
const TOKEN_QUERY: DWORD = 0x0008;
const TokenUser: INT = 1; // TOKEN_INFORMATION_CLASS::TokenUser

// drive type (winbase.h)
const DRIVE_REMOVABLE: UINT = 2;

// file attributes (winnt.h)
const FILE_ATTRIBUTE_READONLY: DWORD = 0x00000001;
const FILE_ATTRIBUTE_HIDDEN: DWORD = 0x00000002;
const FILE_ATTRIBUTE_SYSTEM: DWORD = 0x00000004;

// SHGFP_TYPE (shlobj.h enum)
const SHGFP_TYPE_CURRENT: DWORD = 0;
const SHGFP_TYPE_DEFAULT: DWORD = 1;

// CSIDL flags (shlobj.h)
const CSIDL_FLAG_CREATE: INT = 0x8000;
const CSIDL_FLAG_DONT_VERIFY: INT = 0x4000;
const CSIDL_FLAG_NO_ALIAS: INT = 0x1000;
const CSIDL_FOLDER_MASK: INT = 0x00ff;

// CSIDL indices used by name in the register lists / virtual-folder switch (shlobj.h)
const CSIDL_DESKTOP: INT = 0x0000;
const CSIDL_INTERNET: INT = 0x0001;
const CSIDL_PROGRAMS: UINT = 0x0002;
const CSIDL_CONTROLS: INT = 0x0003;
const CSIDL_PRINTERS: INT = 0x0004;
const CSIDL_PERSONAL: UINT = 0x0005;
const CSIDL_FAVORITES: UINT = 0x0006;
const CSIDL_STARTUP: UINT = 0x0007;
const CSIDL_RECENT: UINT = 0x0008;
const CSIDL_SENDTO: UINT = 0x0009;
const CSIDL_BITBUCKET: INT = 0x000a;
const CSIDL_STARTMENU: UINT = 0x000b;
const CSIDL_MYDOCUMENTS: usize = 0x0005; // == CSIDL_PERSONAL
const CSIDL_MYMUSIC: UINT = 0x000d;
const CSIDL_MYVIDEO: UINT = 0x000e;
const CSIDL_DESKTOPDIRECTORY: UINT = 0x0010;
const CSIDL_DRIVES: INT = 0x0011;
const CSIDL_NETWORK: INT = 0x0012;
const CSIDL_NETHOOD: UINT = 0x0013;
const CSIDL_FONTS: UINT = 0x0014;
const CSIDL_TEMPLATES: UINT = 0x0015;
const CSIDL_COMMON_STARTMENU: UINT = 0x0016;
const CSIDL_COMMON_PROGRAMS: UINT = 0x0017;
const CSIDL_COMMON_STARTUP: UINT = 0x0018;
const CSIDL_COMMON_DESKTOPDIRECTORY: UINT = 0x0019;
const CSIDL_APPDATA: UINT = 0x001a;
const CSIDL_PRINTHOOD: UINT = 0x001b;
const CSIDL_LOCAL_APPDATA: UINT = 0x001c;
const CSIDL_COMMON_FAVORITES: UINT = 0x001f;
const CSIDL_INTERNET_CACHE: UINT = 0x0020;
const CSIDL_COOKIES: UINT = 0x0021;
const CSIDL_HISTORY: UINT = 0x0022;
const CSIDL_COMMON_APPDATA: UINT = 0x0023;
const CSIDL_PROGRAM_FILES: u32 = 0x0026;
const CSIDL_MYPICTURES: UINT = 0x0027;
const CSIDL_SYSTEMX86: u32 = 0x0029;
const CSIDL_PROGRAM_FILESX86: u32 = 0x002a;
const CSIDL_PROGRAM_FILES_COMMON: u32 = 0x002b;
const CSIDL_PROGRAM_FILES_COMMONX86: u32 = 0x002c;
const CSIDL_COMMON_TEMPLATES: UINT = 0x002d;
const CSIDL_COMMON_DOCUMENTS: UINT = 0x002e;
const CSIDL_COMMON_ADMINTOOLS: UINT = 0x002f;
const CSIDL_ADMINTOOLS: UINT = 0x0030;
const CSIDL_CONNECTIONS: INT = 0x0031;
const CSIDL_COMMON_MUSIC: UINT = 0x0035;
const CSIDL_COMMON_PICTURES: UINT = 0x0036;
const CSIDL_COMMON_VIDEO: UINT = 0x0037;

// PathCleanupSpec return flags (shlobj.h)
const PCS_FATAL: DWORD = 0x80000000;
const PCS_REPLACEDCHAR: DWORD = 0x00000001;
const PCS_PATHTOOLONG: DWORD = 0x00000008;

// PathGetCharType results (shlwapi.h)
const GCT_INVALID: UINT = 0x0;
const GCT_WILD: UINT = 0x4;
const GCT_SEPARATOR: UINT = 0x8;

// PathResolve flags (shlobj.h)
const PRF_VERIFYEXISTS: DWORD = 0x01;
const PRF_EXECUTABLE: DWORD = 0x02;
const PRF_FIRSTDIRDEF: DWORD = 0x04;
const PRF_DONTFINDLNK: DWORD = 0x08;
const PRF_REQUIREABSOLUTE: DWORD = 0x10;
const PRF_TRYPROGRAMEXTENSIONS: DWORD = PRF_EXECUTABLE | PRF_VERIFYEXISTS; // 0x03

// WHICH_* extension search masks (shlwapi_undoc.h)
const WHICH_LNK: DWORD = 1 << 4;
const WHICH_DEFAULT: DWORD =
    (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3) | (1 << 4) | (1 << 5); // PIF|COM|EXE|BAT|LNK|CMD

// PathQualifyExW short-path validity classes (shlwapi_undoc.h)
const PATH_CHAR_CLASS_DOT: DWORD = 0x00000004;
const PATH_CHAR_CLASS_BACKSLASH: DWORD = 0x00000008;
const PATH_CHAR_CLASS_COLON: DWORD = 0x00000010;
const PATH_CHAR_CLASS_OTHER_VALID: DWORD = 0x00000100;
const VALID_SHORT_PATH_CHAR_CLASSES: DWORD =
    PATH_CHAR_CLASS_DOT | PATH_CHAR_CLASS_BACKSLASH | PATH_CHAR_CLASS_COLON | PATH_CHAR_CLASS_OTHER_VALID;

const MSDOS_8DOT3_LEN: DWORD = 12; // IsLFNDriveW
const MSDOS_8DOT3_FILENAME_TITLE_LEN: INT = 8;

// NT_PRODUCT_TYPE (winnt.h)
const NtProductWinNt: DWORD = 1;
const NtProductLanManNt: DWORD = 2;
const NtProductServer: DWORD = 3;

// ════════════════════════════════════════════════════════════════════════════════════════
//  CSIDL_Data table types
// ════════════════════════════════════════════════════════════════════════════════════════
type CsidlType = u32;
const CSIDL_Type_User: CsidlType = 0;
const CSIDL_Type_InMyDocuments: CsidlType = 1;
const CSIDL_Type_AllUsers: CsidlType = 2;
const CSIDL_Type_CurrVer: CsidlType = 3;
const CSIDL_Type_Disallowed: CsidlType = 4;
const CSIDL_Type_NonExistent: CsidlType = 5;
const CSIDL_Type_WindowsPath: CsidlType = 6;
const CSIDL_Type_SystemPath: CsidlType = 7;
const CSIDL_Type_SystemX86Path: CsidlType = 8;

/// CSIDL_DATA (shellpath.c):
///   { const KNOWNFOLDERID *id; CSIDL_Type type; LPCWSTR szValueName;
///     LPCWSTR szDefaultPath; INT nShell32IconIndex; }
/// `szDefaultPath` is either a real wide-string pointer or `MAKEINTRESOURCEW(IDS_*)`
/// (a small integer cast to a pointer — `IS_INTRESOURCE` then loads it via LoadStringW).
#[repr(C)]
struct CsidlData {
    id: *const GUID,
    typ: CsidlType,
    szValueName: *const WCHAR,
    szDefaultPath: *const WCHAR,
    nShell32IconIndex: INT,
}
// Read-only table holding raw pointers (to other statics / int-resource sentinels).
unsafe impl Sync for CsidlData {}

#[inline]
fn ARRAY_SIZE_CSIDL() -> DWORD {
    CSIDL_DATA.len() as DWORD
}
#[inline]
unsafe fn csidl(i: DWORD) -> &'static CsidlData {
    &*CSIDL_DATA.as_ptr().add(i as usize)
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  imports (resolved via shell32's existing import libs at link)
// ════════════════════════════════════════════════════════════════════════════════════════
extern "system" {
    // user32
    fn CharPrevW(lpszStart: LPCWSTR, lpszCurrent: LPCWSTR) -> LPWSTR; // @8
    fn LoadStringW(hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: INT) -> INT; // @16
    // kernel32
    fn CloseHandle(hObject: HANDLE) -> BOOL; // @4
    fn GetCurrentDirectoryW(nBufferLength: DWORD, lpBuffer: LPWSTR) -> DWORD; // @8
    fn GetCurrentProcess() -> HANDLE; // @0
    fn GetDriveTypeW(lpRootPathName: LPCWSTR) -> UINT; // @4
    fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD; // @4
    fn GetLastError() -> DWORD; // @0
    fn GetProcessHeap() -> HANDLE; // @0
    fn GetShortPathNameA(lpszLongPath: LPCSTR, lpszShortPath: LPSTR, cchBuffer: DWORD) -> DWORD; // @12
    fn GetShortPathNameW(lpszLongPath: LPCWSTR, lpszShortPath: LPWSTR, cchBuffer: DWORD) -> DWORD; // @12
    fn GetSystemDirectoryW(lpBuffer: LPWSTR, uSize: UINT) -> UINT; // @8
    fn GetSystemWow64DirectoryW(lpBuffer: LPWSTR, uSize: UINT) -> UINT; // @8
    fn GetVersion() -> DWORD; // @0
    fn GetVolumeInformationW(
        lpRootPathName: LPCWSTR,
        lpVolumeNameBuffer: LPWSTR,
        nVolumeNameSize: DWORD,
        lpVolumeSerialNumber: LPDWORD,
        lpMaximumComponentLength: LPDWORD,
        lpFileSystemFlags: LPDWORD,
        lpFileSystemNameBuffer: LPWSTR,
        nFileSystemNameSize: DWORD,
    ) -> BOOL; // @32
    fn GetWindowsDirectoryW(lpBuffer: LPWSTR, uSize: UINT) -> UINT; // @8
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID; // @12
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL; // @12
    fn IsWow64Process(hProcess: HANDLE, Wow64Process: *mut BOOL) -> BOOL; // @8
    fn LocalFree(hMem: HANDLE) -> HANDLE; // @4
    fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: LPCSTR,
        cbMultiByte: INT,
        lpWideCharStr: LPWSTR,
        cchWideChar: INT,
    ) -> INT; // @24
    fn SetFileAttributesW(lpFileName: LPCWSTR, dwFileAttributes: DWORD) -> BOOL; // @8
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
    fn WritePrivateProfileStringW(
        lpAppName: LPCWSTR,
        lpKeyName: LPCWSTR,
        lpString: LPCWSTR,
        lpFileName: LPCWSTR,
    ) -> BOOL; // @16
    fn lstrcmpW(s1: LPCWSTR, s2: LPCWSTR) -> INT; // @8
    fn lstrcmpiA(s1: LPCSTR, s2: LPCSTR) -> INT; // @8
    fn lstrcmpiW(s1: LPCWSTR, s2: LPCWSTR) -> INT; // @8
    fn lstrcpyA(d: LPSTR, s: LPCSTR) -> LPSTR; // @8
    fn lstrcpyW(d: LPWSTR, s: LPCWSTR) -> LPWSTR; // @8
    fn lstrcpynW(d: LPWSTR, s: LPCWSTR, n: INT) -> LPWSTR; // @12
    fn lstrlenW(s: LPCWSTR) -> INT; // @4
    // advapi32
    fn ConvertSidToStringSidW(Sid: PVOID, StringSid: *mut LPWSTR) -> BOOL; // @8
    fn GetTokenInformation(
        TokenHandle: HANDLE,
        TokenInformationClass: INT,
        TokenInformation: LPVOID,
        TokenInformationLength: DWORD,
        ReturnLength: LPDWORD,
    ) -> BOOL; // @20
    fn OpenProcessToken(ProcessHandle: HANDLE, DesiredAccess: DWORD, TokenHandle: *mut HANDLE) -> BOOL; // @12
    fn RegCloseKey(hKey: HKEY) -> LONG; // @4
    fn RegCreateKeyW(hKey: HKEY, lpSubKey: LPCWSTR, phkResult: PHKEY) -> LONG; // @12
    fn RegGetValueW(
        hkey: HKEY,
        lpSubKey: LPCWSTR,
        lpValue: LPCWSTR,
        dwFlags: DWORD,
        pdwType: LPDWORD,
        pvData: PVOID,
        pcbData: LPDWORD,
    ) -> LONG; // @28
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
    fn RegSetValueExW(
        hKey: HKEY,
        lpValueName: LPCWSTR,
        Reserved: DWORD,
        dwType: DWORD,
        lpData: *const BYTE,
        cbData: DWORD,
    ) -> LONG; // @24
    // userenv
    fn GetAllUsersProfileDirectoryW(lpProfileDir: LPWSTR, lpcchSize: LPDWORD) -> BOOL; // @8
    fn GetDefaultUserProfileDirectoryW(lpProfileDir: LPWSTR, lpcchSize: LPDWORD) -> BOOL; // @8
    fn GetUserProfileDirectoryW(hToken: HANDLE, lpProfileDir: LPWSTR, lpcchSize: LPDWORD) -> BOOL; // @12
    // shlwapi
    fn PathAddBackslashW(pszPath: LPWSTR) -> LPWSTR; // @4
    fn PathAppendA(pszPath: LPSTR, pszMore: LPCSTR) -> BOOL; // @8
    fn PathAppendW(pszPath: LPWSTR, pszMore: LPCWSTR) -> BOOL; // @8
    fn PathBuildRootW(pszRoot: LPWSTR, iDrive: INT) -> LPWSTR; // @8
    fn PathCombineW(pszDest: LPWSTR, pszDir: LPCWSTR, pszFile: LPCWSTR) -> LPWSTR; // @12
    fn PathFileExistsA(pszPath: LPCSTR) -> BOOL; // @4
    fn PathFileExistsW(pszPath: LPCWSTR) -> BOOL; // @4
    fn PathFileExistsAndAttributesW(pszPath: LPCWSTR, pdwAttr: LPDWORD) -> BOOL; // @8
    fn PathFileExistsDefExtW(pszPath: LPWSTR, dwWhich: DWORD) -> BOOL; // @8
    fn PathFindExtensionA(pszPath: LPCSTR) -> LPSTR; // @4
    fn PathFindExtensionW(pszPath: LPCWSTR) -> LPWSTR; // @4
    fn PathFindOnPathExW(pszPath: LPWSTR, ppszOtherDirs: *const LPCWSTR, dwWhich: DWORD) -> BOOL; // @12
    fn PathFindOnPathW(pszPath: LPWSTR, ppszOtherDirs: *const LPCWSTR) -> BOOL; // @8
    fn PathGetCharTypeA(ch: u8) -> UINT; // @4
    fn PathGetCharTypeW(ch: WCHAR) -> UINT; // @4
    fn PathGetDriveNumberW(lpsz: LPCWSTR) -> INT; // @4
    fn PathIsFileSpecW(pszPath: LPCWSTR) -> BOOL; // @4
    fn PathIsRootA(pszPath: LPCSTR) -> BOOL; // @4
    fn PathIsRootW(pszPath: LPCWSTR) -> BOOL; // @4
    fn PathIsUNCW(pszPath: LPCWSTR) -> BOOL; // @4
    fn PathIsURLW(pszPath: LPCWSTR) -> BOOL; // @4
    fn PathIsValidCharW(ch: WCHAR, dwClass: DWORD) -> BOOL; // @8
    fn PathRemoveBackslashW(pszPath: LPWSTR) -> LPWSTR; // @4
    fn PathRemoveExtensionW(pszPath: LPWSTR); // @4
    fn PathRemoveFileSpecA(pszPath: LPSTR) -> BOOL; // @4
    fn PathRemoveFileSpecW(pszPath: LPWSTR) -> BOOL; // @4
    fn PathStripToRootW(pszPath: LPWSTR) -> BOOL; // @4
    fn PathUnquoteSpacesW(lpsz: LPWSTR) -> BOOL; // @4
    fn StrChrW(lpStart: LPCWSTR, wMatch: WCHAR) -> LPWSTR; // @8
    fn StrCpyNW(pszDst: LPWSTR, pszSrc: LPCWSTR, cchMax: INT) -> LPWSTR; // @12
    fn StringFromGUID2(rguid: REFGUID, lpsz: LPWSTR, cchMax: INT) -> INT; // @12
    // shlwapi (undoc)
    // shlwapi_undoc.h: `DWORD SHAnsiToUnicode(...)` / `INT SHUnicodeToAnsi(...)`. These
    // were declared returning LPWSTR/LPSTR, which on amd64/arm64 reads a 64-bit register
    // the callee only set 32 bits of. Every call site here discards the result, so the
    // bug was latent, but a future caller would have read garbage in the high half.
    fn SHAnsiToUnicode(pszSrc: LPCSTR, pwszDst: LPWSTR, cwchBuf: INT) -> DWORD; // @12
    fn SHUnicodeToAnsi(pwszSrc: LPCWSTR, pszDst: LPSTR, cchBuf: INT) -> INT; // @12
    fn SHExpandEnvironmentStringsForUserW(
        hToken: HANDLE,
        lpSrc: LPCWSTR,
        lpDest: LPWSTR,
        dwSize: DWORD,
    ) -> DWORD; // @16
    // shell32 itself (siblings — already Rust or still C)
    fn SHAlloc(cb: SIZE_T) -> LPVOID; // @4
    fn SHFree(pv: LPVOID); // @4
    fn SHCreateDirectoryExW(hwnd: HWND, pszPath: LPCWSTR, psa: PVOID) -> INT; // @12
    fn SHILCreateFromPathW(pszPath: LPCWSTR, ppidl: *mut LPITEMIDLIST, prgfInOut: *mut DWORD) -> HRESULT; // @12
    fn ILCombine(pidl1: LPCITEMIDLIST, pidl2: LPCITEMIDLIST) -> LPITEMIDLIST; // @8
    fn ILCreateFromPathW(pszPath: LPCWSTR) -> LPITEMIDLIST; // @4
    fn ILFree(pidl: LPITEMIDLIST); // @4
    // shell util (reactos/shellutils.h) — WINAPI
    fn FixSlashesAndColonW(s: LPWSTR); // @4
}

// cdecl (EXTERN_C, no WINAPI) — `_Name` imports.
extern "C" {
    fn PathProcessCommandW(lpszPath: LPCWSTR, lpszBuff: LPWSTR, dwBuffSize: DWORD, dwFlags: DWORD) -> LONG;
    fn SHBindToObject(
        psf: *mut IShellFolder,
        pidl: LPCITEMIDLIST,
        riid: REFIID,
        ppvObj: *mut *mut c_void,
    ) -> HRESULT;
    // PIDL helpers (now Rust in pidl.rs; source name `_ILName` → obj `__ILName`).
    fn _ILCreateBitBucket() -> LPITEMIDLIST;
    fn _ILCreateControlPanel() -> LPITEMIDLIST;
    fn _ILCreateDesktop() -> LPITEMIDLIST;
    fn _ILCreateIExplore() -> LPITEMIDLIST;
    fn _ILCreateMyComputer() -> LPITEMIDLIST;
    fn _ILCreateMyDocuments() -> LPITEMIDLIST;
    fn _ILCreateNetwork() -> LPITEMIDLIST;
    fn _ILCreatePrinters() -> LPITEMIDLIST;
}

// data import — shell32's module handle (cdecl `_shell32_hInstance`).
extern "C" {
    static shell32_hInstance: HINSTANCE;
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  small in-file helpers (avoid pulling the __imp_ libc thunks)
// ════════════════════════════════════════════════════════════════════════════════════════
#[inline]
fn SUCCEEDED(hr: HRESULT) -> bool {
    hr >= 0
}
#[inline]
fn FAILED(hr: HRESULT) -> bool {
    hr < 0
}
/// SHELL_OsIsUnicode() — shell32_main.h: `!(GetVersion() & 0x80000000)`.
#[inline]
unsafe fn shell_os_is_unicode() -> bool {
    (GetVersion() & 0x80000000) == 0
}
/// IS_INTRESOURCE(p) — winuser.h: high 16 bits of the pointer are zero.
#[inline]
fn is_intresource(p: *const WCHAR) -> bool {
    ((p as usize) >> 16) == 0
}
/// LOWORD of a pointer-as-resource-id.
#[inline]
fn loword_res(p: *const WCHAR) -> UINT {
    ((p as usize) & 0xffff) as UINT
}
#[inline]
fn min_usize(a: usize, b: usize) -> usize {
    if a < b {
        a
    } else {
        b
    }
}
#[inline]
fn min_i32(a: INT, b: INT) -> INT {
    if a < b {
        a
    } else {
        b
    }
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
unsafe fn r_strcpy(dst: *mut CHAR, mut src: *const CHAR) {
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
#[inline]
unsafe fn r_wcslen(mut s: *const WCHAR) -> usize {
    let mut n = 0usize;
    while *s != 0 {
        s = s.add(1);
        n += 1;
    }
    n
}
/// strcpyW — wide strcpy (wine/unicode.h). Returns dst (unused).
#[inline]
unsafe fn strcpyW(dst: *mut WCHAR, mut src: *const WCHAR) -> *mut WCHAR {
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
/// strcatW — wide strcat.
#[inline]
unsafe fn strcatW(dst: *mut WCHAR, src: *const WCHAR) {
    let end = dst.add(r_wcslen(dst));
    strcpyW(end, src);
}
/// strcmpW — wide strcmp (wine/unicode.h).
#[inline]
unsafe fn strcmpW(mut a: *const WCHAR, mut b: *const WCHAR) -> INT {
    loop {
        let ca = *a;
        let cb = *b;
        if ca != cb {
            return ca as INT - cb as INT;
        }
        if ca == 0 {
            return 0;
        }
        a = a.add(1);
        b = b.add(1);
    }
}
#[inline]
fn towlower_ascii(c: WCHAR) -> WCHAR {
    if (0x41..=0x5A).contains(&c) {
        c + 0x20
    } else {
        c
    }
}
/// strncmpiW — case-insensitive wide compare of up to n chars (wine/unicode.h `strncmpiW`).
#[inline]
unsafe fn strncmpiW(a: *const WCHAR, b: *const WCHAR, n: usize) -> INT {
    let mut i = 0usize;
    while i < n {
        let ca = towlower_ascii(*a.add(i));
        let cb = towlower_ascii(*b.add(i));
        if ca != cb {
            return ca as INT - cb as INT;
        }
        if ca == 0 {
            return 0;
        }
        i += 1;
    }
    0
}
/// wcsicmp — case-insensitive wide compare (msvcrt). Used by PathIsExeW.
#[inline]
unsafe fn wcsicmp(mut a: *const WCHAR, mut b: *const WCHAR) -> INT {
    loop {
        let ca = towlower_ascii(*a);
        let cb = towlower_ascii(*b);
        if ca != cb {
            return ca as INT - cb as INT;
        }
        if ca == 0 {
            return 0;
        }
        a = a.add(1);
        b = b.add(1);
    }
}

// ── StringCch* (strsafe.h) — bounded copies returning HRESULT ─────────────────────────────
/// StringCchCopyW: copy src into dst (capacity cchDest incl. NUL). Truncates+errors on overflow.
unsafe fn StringCchCopyW(dst: LPWSTR, cchDest: usize, src: LPCWSTR) -> HRESULT {
    if cchDest == 0 {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    let mut i = 0usize;
    while i + 1 < cchDest {
        let c = *src.add(i);
        *dst.add(i) = c;
        if c == 0 {
            return S_OK;
        }
        i += 1;
    }
    // ran out of room
    *dst.add(cchDest - 1) = 0;
    if *src.add(i) == 0 {
        S_OK
    } else {
        STRSAFE_E_INSUFFICIENT_BUFFER
    }
}
/// StringCchCopyNW: copy at most cchToCopy chars of src into dst (capacity cchDest incl NUL).
unsafe fn StringCchCopyNW(dst: LPWSTR, cchDest: usize, src: LPCWSTR, cch_to_copy: usize) -> HRESULT {
    if cchDest == 0 {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    let mut i = 0usize;
    while i < cch_to_copy && *src.add(i) != 0 {
        if i >= cchDest - 1 {
            *dst.add(cchDest - 1) = 0;
            return STRSAFE_E_INSUFFICIENT_BUFFER;
        }
        *dst.add(i) = *src.add(i);
        i += 1;
    }
    *dst.add(i) = 0;
    S_OK
}
/// StringCchCatW: append src to dst (capacity cchDest incl. NUL).
unsafe fn StringCchCatW(dst: LPWSTR, cchDest: usize, src: LPCWSTR) -> HRESULT {
    let used = r_wcslen(dst);
    if used >= cchDest {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    StringCchCopyW(dst.add(used), cchDest - used, src)
}

/// Format argument for `string_cch_printf` (covers %s and %d, plus %% literals).
enum FA {
    I(INT),
    S(*const WCHAR),
}
/// Minimal StringCchPrintfW covering exactly the format strings used here:
/// `%d`, ` (%d)`, `%s (%d)%s`, and `%%SystemRoot%%\system32\shell32.dll,%d` (%% → %).
unsafe fn string_cch_printf(dst: LPWSTR, cch_dest: usize, fmt: LPCWSTR, args: &[FA]) -> HRESULT {
    if cch_dest == 0 {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    let mut out = 0usize; // index into dst
    let cap = cch_dest - 1; // reserve NUL
    let mut ai = 0usize; // arg index
    let mut f = fmt;
    macro_rules! emit {
        ($ch:expr) => {{
            if out >= cap {
                *dst.add(cap) = 0;
                return STRSAFE_E_INSUFFICIENT_BUFFER;
            }
            *dst.add(out) = $ch;
            out += 1;
        }};
    }
    loop {
        let c = *f;
        if c == 0 {
            break;
        }
        if c == 0x25 {
            // '%'
            let n = *f.add(1);
            if n == 0x25 {
                // "%%"
                emit!(0x25);
                f = f.add(2);
                continue;
            } else if n == 0x64 {
                // "%d"
                if let Some(FA::I(v)) = args.get(ai) {
                    let v = *v;
                    // signed decimal
                    let mut buf = [0u16; 16];
                    let mut len = 0usize;
                    let neg = v < 0;
                    let mut uv: u32 = if neg { (v as i64).unsigned_abs() as u32 } else { v as u32 };
                    if uv == 0 {
                        buf[len] = 0x30;
                        len += 1;
                    } else {
                        while uv > 0 {
                            buf[len] = 0x30 + (uv % 10) as u16;
                            len += 1;
                            uv /= 10;
                        }
                    }
                    if neg {
                        emit!(0x2D); // '-'
                    }
                    while len > 0 {
                        len -= 1;
                        emit!(buf[len]);
                    }
                }
                ai += 1;
                f = f.add(2);
                continue;
            } else if n == 0x73 {
                // "%s"
                if let Some(FA::S(p)) = args.get(ai) {
                    let mut s = *p;
                    if !s.is_null() {
                        while *s != 0 {
                            let ch = *s;
                            emit!(ch);
                            s = s.add(1);
                        }
                    }
                }
                ai += 1;
                f = f.add(2);
                continue;
            }
        }
        emit!(c);
        f = f.add(1);
    }
    *dst.add(out) = 0;
    S_OK
}

// ════ generated: CSIDL wide-strings + table + FOLDERID externs ════
// ── wide-string statics (UTF-16, NUL-terminated) for the CSIDL table (generated) ──
static WS_Desktop: [WCHAR; 8] = [0x0044, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x0000]; // "Desktop"
static WS_Programs: [WCHAR; 9] = [0x0050, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061, 0x006D, 0x0073, 0x0000]; // "Programs"
static WS_ControlPanelFolder: [WCHAR; 19] = [0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0050, 0x0061, 0x006E, 0x0065, 0x006C, 0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0000]; // "ControlPanelFolder"
static WS_PrintersFolder: [WCHAR; 15] = [0x0050, 0x0072, 0x0069, 0x006E, 0x0074, 0x0065, 0x0072, 0x0073, 0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0000]; // "PrintersFolder"
static WS_Personal: [WCHAR; 9] = [0x0050, 0x0065, 0x0072, 0x0073, 0x006F, 0x006E, 0x0061, 0x006C, 0x0000]; // "Personal"
static WS_Favorites: [WCHAR; 10] = [0x0046, 0x0061, 0x0076, 0x006F, 0x0072, 0x0069, 0x0074, 0x0065, 0x0073, 0x0000]; // "Favorites"
static WS_StartUp: [WCHAR; 8] = [0x0053, 0x0074, 0x0061, 0x0072, 0x0074, 0x0055, 0x0070, 0x0000]; // "StartUp"
static WS_Recent: [WCHAR; 7] = [0x0052, 0x0065, 0x0063, 0x0065, 0x006E, 0x0074, 0x0000]; // "Recent"
static WS_SendTo: [WCHAR; 7] = [0x0053, 0x0065, 0x006E, 0x0064, 0x0054, 0x006F, 0x0000]; // "SendTo"
static WS_RecycleBinFolder: [WCHAR; 17] = [0x0052, 0x0065, 0x0063, 0x0079, 0x0063, 0x006C, 0x0065, 0x0042, 0x0069, 0x006E, 0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0000]; // "RecycleBinFolder"
static WS_Start_Menu: [WCHAR; 11] = [0x0053, 0x0074, 0x0061, 0x0072, 0x0074, 0x0020, 0x004D, 0x0065, 0x006E, 0x0075, 0x0000]; // "Start Menu"
static WS_My_Music: [WCHAR; 9] = [0x004D, 0x0079, 0x0020, 0x004D, 0x0075, 0x0073, 0x0069, 0x0063, 0x0000]; // "My Music"
static WS_My_Video: [WCHAR; 9] = [0x004D, 0x0079, 0x0020, 0x0056, 0x0069, 0x0064, 0x0065, 0x006F, 0x0000]; // "My Video"
static WS_MyComputerFolder: [WCHAR; 17] = [0x004D, 0x0079, 0x0043, 0x006F, 0x006D, 0x0070, 0x0075, 0x0074, 0x0065, 0x0072, 0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0000]; // "MyComputerFolder"
static WS_NetworkPlacesFolder: [WCHAR; 20] = [0x004E, 0x0065, 0x0074, 0x0077, 0x006F, 0x0072, 0x006B, 0x0050, 0x006C, 0x0061, 0x0063, 0x0065, 0x0073, 0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0000]; // "NetworkPlacesFolder"
static WS_NetHood: [WCHAR; 8] = [0x004E, 0x0065, 0x0074, 0x0048, 0x006F, 0x006F, 0x0064, 0x0000]; // "NetHood"
static WS_Fonts: [WCHAR; 6] = [0x0046, 0x006F, 0x006E, 0x0074, 0x0073, 0x0000]; // "Fonts"
static WS_Templates: [WCHAR; 10] = [0x0054, 0x0065, 0x006D, 0x0070, 0x006C, 0x0061, 0x0074, 0x0065, 0x0073, 0x0000]; // "Templates"
static WS_Common_Start_Menu: [WCHAR; 18] = [0x0043, 0x006F, 0x006D, 0x006D, 0x006F, 0x006E, 0x0020, 0x0053, 0x0074, 0x0061, 0x0072, 0x0074, 0x0020, 0x004D, 0x0065, 0x006E, 0x0075, 0x0000]; // "Common Start Menu"
static WS_Common_Programs: [WCHAR; 16] = [0x0043, 0x006F, 0x006D, 0x006D, 0x006F, 0x006E, 0x0020, 0x0050, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061, 0x006D, 0x0073, 0x0000]; // "Common Programs"
static WS_Common_StartUp: [WCHAR; 15] = [0x0043, 0x006F, 0x006D, 0x006D, 0x006F, 0x006E, 0x0020, 0x0053, 0x0074, 0x0061, 0x0072, 0x0074, 0x0055, 0x0070, 0x0000]; // "Common StartUp"
static WS_Common_Desktop: [WCHAR; 15] = [0x0043, 0x006F, 0x006D, 0x006D, 0x006F, 0x006E, 0x0020, 0x0044, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x0000]; // "Common Desktop"
static WS_AppData: [WCHAR; 8] = [0x0041, 0x0070, 0x0070, 0x0044, 0x0061, 0x0074, 0x0061, 0x0000]; // "AppData"
static WS_PrintHood: [WCHAR; 10] = [0x0050, 0x0072, 0x0069, 0x006E, 0x0074, 0x0048, 0x006F, 0x006F, 0x0064, 0x0000]; // "PrintHood"
static WS_Local_AppData: [WCHAR; 14] = [0x004C, 0x006F, 0x0063, 0x0061, 0x006C, 0x0020, 0x0041, 0x0070, 0x0070, 0x0044, 0x0061, 0x0074, 0x0061, 0x0000]; // "Local AppData"
static WS_Common_Favorites: [WCHAR; 17] = [0x0043, 0x006F, 0x006D, 0x006D, 0x006F, 0x006E, 0x0020, 0x0046, 0x0061, 0x0076, 0x006F, 0x0072, 0x0069, 0x0074, 0x0065, 0x0073, 0x0000]; // "Common Favorites"
static WS_Cache: [WCHAR; 6] = [0x0043, 0x0061, 0x0063, 0x0068, 0x0065, 0x0000]; // "Cache"
static WS_Cookies: [WCHAR; 8] = [0x0043, 0x006F, 0x006F, 0x006B, 0x0069, 0x0065, 0x0073, 0x0000]; // "Cookies"
static WS_History: [WCHAR; 8] = [0x0048, 0x0069, 0x0073, 0x0074, 0x006F, 0x0072, 0x0079, 0x0000]; // "History"
static WS_Common_AppData: [WCHAR; 15] = [0x0043, 0x006F, 0x006D, 0x006D, 0x006F, 0x006E, 0x0020, 0x0041, 0x0070, 0x0070, 0x0044, 0x0061, 0x0074, 0x0061, 0x0000]; // "Common AppData"
static WS_Windows: [WCHAR; 8] = [0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0000]; // "Windows"
static WS_System: [WCHAR; 7] = [0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0000]; // "System"
static WS_ProgramFiles: [WCHAR; 13] = [0x0050, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061, 0x006D, 0x0046, 0x0069, 0x006C, 0x0065, 0x0073, 0x0000]; // "ProgramFiles"
static WS_My_Pictures: [WCHAR; 12] = [0x004D, 0x0079, 0x0020, 0x0050, 0x0069, 0x0063, 0x0074, 0x0075, 0x0072, 0x0065, 0x0073, 0x0000]; // "My Pictures"
static WS_ProgramFilesX86: [WCHAR; 16] = [0x0050, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061, 0x006D, 0x0046, 0x0069, 0x006C, 0x0065, 0x0073, 0x0058, 0x0038, 0x0036, 0x0000]; // "ProgramFilesX86"
static WS_Program_Files_x86: [WCHAR; 20] = [0x0050, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061, 0x006D, 0x0020, 0x0046, 0x0069, 0x006C, 0x0065, 0x0073, 0x0020, 0x0028, 0x0078, 0x0038, 0x0036, 0x0029, 0x0000]; // "Program Files (x86)"
static WS_ProgramFilesCommon: [WCHAR; 19] = [0x0050, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061, 0x006D, 0x0046, 0x0069, 0x006C, 0x0065, 0x0073, 0x0043, 0x006F, 0x006D, 0x006D, 0x006F, 0x006E, 0x0000]; // "ProgramFilesCommon"
static WS_ProgramFilesCommonX86: [WCHAR; 22] = [0x0050, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061, 0x006D, 0x0046, 0x0069, 0x006C, 0x0065, 0x0073, 0x0043, 0x006F, 0x006D, 0x006D, 0x006F, 0x006E, 0x0058, 0x0038, 0x0036, 0x0000]; // "ProgramFilesCommonX86"
static WS_Program_Files_x86_Common_Files: [WCHAR; 33] = [0x0050, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061, 0x006D, 0x0020, 0x0046, 0x0069, 0x006C, 0x0065, 0x0073, 0x0020, 0x0028, 0x0078, 0x0038, 0x0036, 0x0029, 0x005C, 0x0043, 0x006F, 0x006D, 0x006D, 0x006F, 0x006E, 0x0020, 0x0046, 0x0069, 0x006C, 0x0065, 0x0073, 0x0000]; // "Program Files (x86)\\Common Files"
static WS_Common_Templates: [WCHAR; 17] = [0x0043, 0x006F, 0x006D, 0x006D, 0x006F, 0x006E, 0x0020, 0x0054, 0x0065, 0x006D, 0x0070, 0x006C, 0x0061, 0x0074, 0x0065, 0x0073, 0x0000]; // "Common Templates"
static WS_Common_Documents: [WCHAR; 17] = [0x0043, 0x006F, 0x006D, 0x006D, 0x006F, 0x006E, 0x0020, 0x0044, 0x006F, 0x0063, 0x0075, 0x006D, 0x0065, 0x006E, 0x0074, 0x0073, 0x0000]; // "Common Documents"
static WS_Common_Administrative_Tools: [WCHAR; 28] = [0x0043, 0x006F, 0x006D, 0x006D, 0x006F, 0x006E, 0x0020, 0x0041, 0x0064, 0x006D, 0x0069, 0x006E, 0x0069, 0x0073, 0x0074, 0x0072, 0x0061, 0x0074, 0x0069, 0x0076, 0x0065, 0x0020, 0x0054, 0x006F, 0x006F, 0x006C, 0x0073, 0x0000]; // "Common Administrative Tools"
static WS_Administrative_Tools: [WCHAR; 21] = [0x0041, 0x0064, 0x006D, 0x0069, 0x006E, 0x0069, 0x0073, 0x0074, 0x0072, 0x0061, 0x0074, 0x0069, 0x0076, 0x0065, 0x0020, 0x0054, 0x006F, 0x006F, 0x006C, 0x0073, 0x0000]; // "Administrative Tools"
static WS_ConnectionsFolder: [WCHAR; 18] = [0x0043, 0x006F, 0x006E, 0x006E, 0x0065, 0x0063, 0x0074, 0x0069, 0x006F, 0x006E, 0x0073, 0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0000]; // "ConnectionsFolder"
static WS_CommonMusic: [WCHAR; 12] = [0x0043, 0x006F, 0x006D, 0x006D, 0x006F, 0x006E, 0x004D, 0x0075, 0x0073, 0x0069, 0x0063, 0x0000]; // "CommonMusic"
static WS_CommonPictures: [WCHAR; 15] = [0x0043, 0x006F, 0x006D, 0x006D, 0x006F, 0x006E, 0x0050, 0x0069, 0x0063, 0x0074, 0x0075, 0x0072, 0x0065, 0x0073, 0x0000]; // "CommonPictures"
static WS_CommonVideo: [WCHAR; 12] = [0x0043, 0x006F, 0x006D, 0x006D, 0x006F, 0x006E, 0x0056, 0x0069, 0x0064, 0x0065, 0x006F, 0x0000]; // "CommonVideo"
static WS_Resources: [WCHAR; 10] = [0x0052, 0x0065, 0x0073, 0x006F, 0x0075, 0x0072, 0x0063, 0x0065, 0x0073, 0x0000]; // "Resources"
static WS_OEM_Links: [WCHAR; 10] = [0x004F, 0x0045, 0x004D, 0x0020, 0x004C, 0x0069, 0x006E, 0x006B, 0x0073, 0x0000]; // "OEM Links"
static WS_CD_Burning: [WCHAR; 11] = [0x0043, 0x0044, 0x0020, 0x0042, 0x0075, 0x0072, 0x006E, 0x0069, 0x006E, 0x0067, 0x0000]; // "CD Burning"
static WS_Local_Settings_Application_Data_Microsoft_CD_Burning: [WCHAR; 53] = [0x004C, 0x006F, 0x0063, 0x0061, 0x006C, 0x0020, 0x0053, 0x0065, 0x0074, 0x0074, 0x0069, 0x006E, 0x0067, 0x0073, 0x005C, 0x0041, 0x0070, 0x0070, 0x006C, 0x0069, 0x0063, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020, 0x0044, 0x0061, 0x0074, 0x0061, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0043, 0x0044, 0x0020, 0x0042, 0x0075, 0x0072, 0x006E, 0x0069, 0x006E, 0x0067, 0x0000]; // "Local Settings\\Application Data\\Microsoft\\CD Burning"

static CSIDL_DATA: [CsidlData; 63] = [
    CsidlData { id: addr_of!(FOLDERID_Desktop), typ: CSIDL_Type_User, szValueName: WS_Desktop.as_ptr(), szDefaultPath: (54 as usize) as *const WCHAR, nShell32IconIndex: 0 }, // 0x00
    CsidlData { id: addr_of!(FOLDERID_InternetFolder), typ: CSIDL_Type_Disallowed, szValueName: null(), szDefaultPath: null(), nShell32IconIndex: 0 }, // 0x01
    CsidlData { id: addr_of!(FOLDERID_Programs), typ: CSIDL_Type_User, szValueName: WS_Programs.as_ptr(), szDefaultPath: (45 as usize) as *const WCHAR, nShell32IconIndex: 0 }, // 0x02
    CsidlData { id: addr_of!(FOLDERID_ControlPanelFolder), typ: CSIDL_Type_SystemPath, szValueName: WS_ControlPanelFolder.as_ptr(), szDefaultPath: null(), nShell32IconIndex: -22 }, // 0x03
    CsidlData { id: addr_of!(FOLDERID_PrintersFolder), typ: CSIDL_Type_SystemPath, szValueName: WS_PrintersFolder.as_ptr(), szDefaultPath: null(), nShell32IconIndex: -38 }, // 0x04
    CsidlData { id: addr_of!(FOLDERID_Documents), typ: CSIDL_Type_User, szValueName: WS_Personal.as_ptr(), szDefaultPath: (9227 as usize) as *const WCHAR, nShell32IconIndex: -235 }, // 0x05
    CsidlData { id: addr_of!(FOLDERID_Favorites), typ: CSIDL_Type_User, szValueName: WS_Favorites.as_ptr(), szDefaultPath: (47 as usize) as *const WCHAR, nShell32IconIndex: -44 }, // 0x06
    CsidlData { id: addr_of!(FOLDERID_Startup), typ: CSIDL_Type_User, szValueName: WS_StartUp.as_ptr(), szDefaultPath: (48 as usize) as *const WCHAR, nShell32IconIndex: 0 }, // 0x07
    CsidlData { id: addr_of!(FOLDERID_Recent), typ: CSIDL_Type_User, szValueName: WS_Recent.as_ptr(), szDefaultPath: (49 as usize) as *const WCHAR, nShell32IconIndex: -21 }, // 0x08
    CsidlData { id: addr_of!(FOLDERID_SendTo), typ: CSIDL_Type_User, szValueName: WS_SendTo.as_ptr(), szDefaultPath: (50 as usize) as *const WCHAR, nShell32IconIndex: 0 }, // 0x09
    CsidlData { id: addr_of!(FOLDERID_RecycleBinFolder), typ: CSIDL_Type_Disallowed, szValueName: WS_RecycleBinFolder.as_ptr(), szDefaultPath: null(), nShell32IconIndex: 0 }, // 0x0a
    CsidlData { id: addr_of!(FOLDERID_StartMenu), typ: CSIDL_Type_User, szValueName: WS_Start_Menu.as_ptr(), szDefaultPath: (51 as usize) as *const WCHAR, nShell32IconIndex: -40 }, // 0x0b
    CsidlData { id: addr_of!(GUID_NULL), typ: CSIDL_Type_Disallowed, szValueName: null(), szDefaultPath: null(), nShell32IconIndex: -235 }, // 0x0c
    CsidlData { id: addr_of!(FOLDERID_Music), typ: CSIDL_Type_InMyDocuments, szValueName: WS_My_Music.as_ptr(), szDefaultPath: (52 as usize) as *const WCHAR, nShell32IconIndex: -237 }, // 0x0d
    CsidlData { id: addr_of!(FOLDERID_Videos), typ: CSIDL_Type_InMyDocuments, szValueName: WS_My_Video.as_ptr(), szDefaultPath: (53 as usize) as *const WCHAR, nShell32IconIndex: -238 }, // 0x0e
    CsidlData { id: addr_of!(GUID_NULL), typ: CSIDL_Type_Disallowed, szValueName: null(), szDefaultPath: null(), nShell32IconIndex: 0 }, // 0x0f
    CsidlData { id: addr_of!(FOLDERID_Desktop), typ: CSIDL_Type_User, szValueName: WS_Desktop.as_ptr(), szDefaultPath: (54 as usize) as *const WCHAR, nShell32IconIndex: 0 }, // 0x10
    CsidlData { id: addr_of!(FOLDERID_ComputerFolder), typ: CSIDL_Type_Disallowed, szValueName: WS_MyComputerFolder.as_ptr(), szDefaultPath: null(), nShell32IconIndex: -43 }, // 0x11
    CsidlData { id: addr_of!(FOLDERID_NetworkFolder), typ: CSIDL_Type_Disallowed, szValueName: WS_NetworkPlacesFolder.as_ptr(), szDefaultPath: null(), nShell32IconIndex: -172 }, // 0x12
    CsidlData { id: addr_of!(FOLDERID_NetHood), typ: CSIDL_Type_User, szValueName: WS_NetHood.as_ptr(), szDefaultPath: (55 as usize) as *const WCHAR, nShell32IconIndex: -15 }, // 0x13
    CsidlData { id: addr_of!(FOLDERID_Fonts), typ: CSIDL_Type_WindowsPath, szValueName: WS_Fonts.as_ptr(), szDefaultPath: WS_Fonts.as_ptr(), nShell32IconIndex: -39 }, // 0x14
    CsidlData { id: addr_of!(FOLDERID_Templates), typ: CSIDL_Type_User, szValueName: WS_Templates.as_ptr(), szDefaultPath: (56 as usize) as *const WCHAR, nShell32IconIndex: 0 }, // 0x15
    CsidlData { id: addr_of!(FOLDERID_CommonStartMenu), typ: CSIDL_Type_AllUsers, szValueName: WS_Common_Start_Menu.as_ptr(), szDefaultPath: (51 as usize) as *const WCHAR, nShell32IconIndex: -40 }, // 0x16
    CsidlData { id: addr_of!(FOLDERID_CommonPrograms), typ: CSIDL_Type_AllUsers, szValueName: WS_Common_Programs.as_ptr(), szDefaultPath: (45 as usize) as *const WCHAR, nShell32IconIndex: 0 }, // 0x17
    CsidlData { id: addr_of!(FOLDERID_CommonStartup), typ: CSIDL_Type_AllUsers, szValueName: WS_Common_StartUp.as_ptr(), szDefaultPath: (48 as usize) as *const WCHAR, nShell32IconIndex: 0 }, // 0x18
    CsidlData { id: addr_of!(FOLDERID_PublicDesktop), typ: CSIDL_Type_AllUsers, szValueName: WS_Common_Desktop.as_ptr(), szDefaultPath: (54 as usize) as *const WCHAR, nShell32IconIndex: 0 }, // 0x19
    CsidlData { id: addr_of!(FOLDERID_RoamingAppData), typ: CSIDL_Type_User, szValueName: WS_AppData.as_ptr(), szDefaultPath: (57 as usize) as *const WCHAR, nShell32IconIndex: 0 }, // 0x1a
    CsidlData { id: addr_of!(FOLDERID_PrintHood), typ: CSIDL_Type_User, szValueName: WS_PrintHood.as_ptr(), szDefaultPath: (58 as usize) as *const WCHAR, nShell32IconIndex: -38 }, // 0x1b
    CsidlData { id: addr_of!(FOLDERID_LocalAppData), typ: CSIDL_Type_User, szValueName: WS_Local_AppData.as_ptr(), szDefaultPath: (59 as usize) as *const WCHAR, nShell32IconIndex: 0 }, // 0x1c
    CsidlData { id: addr_of!(GUID_NULL), typ: CSIDL_Type_NonExistent, szValueName: null(), szDefaultPath: null(), nShell32IconIndex: 0 }, // 0x1d
    CsidlData { id: addr_of!(GUID_NULL), typ: CSIDL_Type_NonExistent, szValueName: null(), szDefaultPath: null(), nShell32IconIndex: 0 }, // 0x1e
    CsidlData { id: addr_of!(FOLDERID_Favorites), typ: CSIDL_Type_AllUsers, szValueName: WS_Common_Favorites.as_ptr(), szDefaultPath: (47 as usize) as *const WCHAR, nShell32IconIndex: -44 }, // 0x1f
    CsidlData { id: addr_of!(FOLDERID_InternetCache), typ: CSIDL_Type_User, szValueName: WS_Cache.as_ptr(), szDefaultPath: (60 as usize) as *const WCHAR, nShell32IconIndex: 0 }, // 0x20
    CsidlData { id: addr_of!(FOLDERID_Cookies), typ: CSIDL_Type_User, szValueName: WS_Cookies.as_ptr(), szDefaultPath: (61 as usize) as *const WCHAR, nShell32IconIndex: 0 }, // 0x21
    CsidlData { id: addr_of!(FOLDERID_History), typ: CSIDL_Type_User, szValueName: WS_History.as_ptr(), szDefaultPath: (62 as usize) as *const WCHAR, nShell32IconIndex: 0 }, // 0x22
    CsidlData { id: addr_of!(FOLDERID_ProgramData), typ: CSIDL_Type_AllUsers, szValueName: WS_Common_AppData.as_ptr(), szDefaultPath: (57 as usize) as *const WCHAR, nShell32IconIndex: 0 }, // 0x23
    CsidlData { id: addr_of!(FOLDERID_Windows), typ: CSIDL_Type_WindowsPath, szValueName: WS_Windows.as_ptr(), szDefaultPath: null(), nShell32IconIndex: 0 }, // 0x24
    CsidlData { id: addr_of!(FOLDERID_System), typ: CSIDL_Type_SystemPath, szValueName: WS_System.as_ptr(), szDefaultPath: null(), nShell32IconIndex: 0 }, // 0x25
    CsidlData { id: addr_of!(FOLDERID_ProgramFiles), typ: CSIDL_Type_CurrVer, szValueName: WS_ProgramFiles.as_ptr(), szDefaultPath: (63 as usize) as *const WCHAR, nShell32IconIndex: 0 }, // 0x26
    CsidlData { id: addr_of!(FOLDERID_Pictures), typ: CSIDL_Type_InMyDocuments, szValueName: WS_My_Pictures.as_ptr(), szDefaultPath: (64 as usize) as *const WCHAR, nShell32IconIndex: -236 }, // 0x27
    CsidlData { id: addr_of!(FOLDERID_Profile), typ: CSIDL_Type_User, szValueName: null(), szDefaultPath: null(), nShell32IconIndex: 0 }, // 0x28
    CsidlData { id: addr_of!(FOLDERID_SystemX86), typ: CSIDL_Type_SystemX86Path, szValueName: null(), szDefaultPath: null(), nShell32IconIndex: -274 }, // 0x29
    CsidlData { id: addr_of!(FOLDERID_ProgramFilesX86), typ: CSIDL_Type_CurrVer, szValueName: WS_ProgramFilesX86.as_ptr(), szDefaultPath: WS_Program_Files_x86.as_ptr(), nShell32IconIndex: 0 }, // 0x2a
    CsidlData { id: addr_of!(FOLDERID_ProgramFilesCommon), typ: CSIDL_Type_CurrVer, szValueName: WS_ProgramFilesCommon.as_ptr(), szDefaultPath: (65 as usize) as *const WCHAR, nShell32IconIndex: 0 }, // 0x2b
    CsidlData { id: addr_of!(FOLDERID_ProgramFilesCommonX86), typ: CSIDL_Type_CurrVer, szValueName: WS_ProgramFilesCommonX86.as_ptr(), szDefaultPath: WS_Program_Files_x86_Common_Files.as_ptr(), nShell32IconIndex: 0 }, // 0x2c
    CsidlData { id: addr_of!(FOLDERID_CommonTemplates), typ: CSIDL_Type_AllUsers, szValueName: WS_Common_Templates.as_ptr(), szDefaultPath: (56 as usize) as *const WCHAR, nShell32IconIndex: 0 }, // 0x2d
    CsidlData { id: addr_of!(FOLDERID_PublicDocuments), typ: CSIDL_Type_AllUsers, szValueName: WS_Common_Documents.as_ptr(), szDefaultPath: (9227 as usize) as *const WCHAR, nShell32IconIndex: -235 }, // 0x2e
    CsidlData { id: addr_of!(FOLDERID_CommonAdminTools), typ: CSIDL_Type_AllUsers, szValueName: WS_Common_Administrative_Tools.as_ptr(), szDefaultPath: (67 as usize) as *const WCHAR, nShell32IconIndex: 0 }, // 0x2f
    CsidlData { id: addr_of!(FOLDERID_AdminTools), typ: CSIDL_Type_User, szValueName: WS_Administrative_Tools.as_ptr(), szDefaultPath: (67 as usize) as *const WCHAR, nShell32IconIndex: 0 }, // 0x30
    CsidlData { id: addr_of!(FOLDERID_ConnectionsFolder), typ: CSIDL_Type_Disallowed, szValueName: WS_ConnectionsFolder.as_ptr(), szDefaultPath: null(), nShell32IconIndex: -175 }, // 0x31
    CsidlData { id: addr_of!(GUID_NULL), typ: CSIDL_Type_Disallowed, szValueName: null(), szDefaultPath: null(), nShell32IconIndex: 0 }, // 0x32
    CsidlData { id: addr_of!(GUID_NULL), typ: CSIDL_Type_Disallowed, szValueName: null(), szDefaultPath: null(), nShell32IconIndex: 0 }, // 0x33
    CsidlData { id: addr_of!(GUID_NULL), typ: CSIDL_Type_Disallowed, szValueName: null(), szDefaultPath: null(), nShell32IconIndex: 0 }, // 0x34
    CsidlData { id: addr_of!(FOLDERID_PublicMusic), typ: CSIDL_Type_AllUsers, szValueName: WS_CommonMusic.as_ptr(), szDefaultPath: (68 as usize) as *const WCHAR, nShell32IconIndex: -237 }, // 0x35
    CsidlData { id: addr_of!(FOLDERID_PublicPictures), typ: CSIDL_Type_AllUsers, szValueName: WS_CommonPictures.as_ptr(), szDefaultPath: (69 as usize) as *const WCHAR, nShell32IconIndex: -236 }, // 0x36
    CsidlData { id: addr_of!(FOLDERID_PublicVideos), typ: CSIDL_Type_AllUsers, szValueName: WS_CommonVideo.as_ptr(), szDefaultPath: (70 as usize) as *const WCHAR, nShell32IconIndex: -238 }, // 0x37
    CsidlData { id: addr_of!(FOLDERID_ResourceDir), typ: CSIDL_Type_WindowsPath, szValueName: null(), szDefaultPath: WS_Resources.as_ptr(), nShell32IconIndex: 0 }, // 0x38
    CsidlData { id: addr_of!(FOLDERID_LocalizedResourcesDir), typ: CSIDL_Type_NonExistent, szValueName: null(), szDefaultPath: null(), nShell32IconIndex: 0 }, // 0x39
    CsidlData { id: addr_of!(FOLDERID_CommonOEMLinks), typ: CSIDL_Type_AllUsers, szValueName: null(), szDefaultPath: WS_OEM_Links.as_ptr(), nShell32IconIndex: 0 }, // 0x3a
    CsidlData { id: addr_of!(FOLDERID_CDBurning), typ: CSIDL_Type_User, szValueName: WS_CD_Burning.as_ptr(), szDefaultPath: WS_Local_Settings_Application_Data_Microsoft_CD_Burning.as_ptr(), nShell32IconIndex: 0 }, // 0x3b
    CsidlData { id: addr_of!(GUID_NULL), typ: CSIDL_Type_Disallowed, szValueName: null(), szDefaultPath: null(), nShell32IconIndex: 0 }, // 0x3c
    CsidlData { id: addr_of!(GUID_NULL), typ: CSIDL_Type_Disallowed, szValueName: null(), szDefaultPath: null(), nShell32IconIndex: 0 }, // 0x3d
    CsidlData { id: addr_of!(GUID_NULL), typ: CSIDL_Type_Disallowed, szValueName: null(), szDefaultPath: null(), nShell32IconIndex: 0 }, // 0x3e
];

// data imports — cdecl `_FOLDERID_*` / `_GUID_NULL` / `_CLSID_*` / `_IID_*` GUID statics.
extern "C" {
    static GUID_NULL: GUID;
    static CLSID_NetworkConnections: CLSID;
    static IID_IShellFolder: IID;
    static FOLDERID_AdminTools: GUID;
    static FOLDERID_CDBurning: GUID;
    static FOLDERID_CommonAdminTools: GUID;
    static FOLDERID_CommonOEMLinks: GUID;
    static FOLDERID_CommonPrograms: GUID;
    static FOLDERID_CommonStartMenu: GUID;
    static FOLDERID_CommonStartup: GUID;
    static FOLDERID_CommonTemplates: GUID;
    static FOLDERID_ComputerFolder: GUID;
    static FOLDERID_ConnectionsFolder: GUID;
    static FOLDERID_ControlPanelFolder: GUID;
    static FOLDERID_Cookies: GUID;
    static FOLDERID_Desktop: GUID;
    static FOLDERID_Documents: GUID;
    static FOLDERID_Favorites: GUID;
    static FOLDERID_Fonts: GUID;
    static FOLDERID_History: GUID;
    static FOLDERID_InternetCache: GUID;
    static FOLDERID_InternetFolder: GUID;
    static FOLDERID_LocalAppData: GUID;
    static FOLDERID_LocalizedResourcesDir: GUID;
    static FOLDERID_Music: GUID;
    static FOLDERID_NetHood: GUID;
    static FOLDERID_NetworkFolder: GUID;
    static FOLDERID_Pictures: GUID;
    static FOLDERID_PrintHood: GUID;
    static FOLDERID_PrintersFolder: GUID;
    static FOLDERID_Profile: GUID;
    static FOLDERID_ProgramData: GUID;
    static FOLDERID_ProgramFiles: GUID;
    static FOLDERID_ProgramFilesCommon: GUID;
    static FOLDERID_ProgramFilesCommonX86: GUID;
    static FOLDERID_ProgramFilesX86: GUID;
    static FOLDERID_Programs: GUID;
    static FOLDERID_PublicDesktop: GUID;
    static FOLDERID_PublicDocuments: GUID;
    static FOLDERID_PublicMusic: GUID;
    static FOLDERID_PublicPictures: GUID;
    static FOLDERID_PublicVideos: GUID;
    static FOLDERID_Recent: GUID;
    static FOLDERID_RecycleBinFolder: GUID;
    static FOLDERID_ResourceDir: GUID;
    static FOLDERID_RoamingAppData: GUID;
    static FOLDERID_SendTo: GUID;
    static FOLDERID_StartMenu: GUID;
    static FOLDERID_Startup: GUID;
    static FOLDERID_System: GUID;
    static FOLDERID_SystemX86: GUID;
    static FOLDERID_Templates: GUID;
    static FOLDERID_Videos: GUID;
    static FOLDERID_Windows: GUID;
}
// FOLDERID count: 51

// ── named literal wide-strings (registry/env/format) ──
static W_USERPROFILE: [WCHAR; 14] = [0x0025, 0x0055, 0x0053, 0x0045, 0x0052, 0x0050, 0x0052, 0x004F, 0x0046, 0x0049, 0x004C, 0x0045, 0x0025, 0x0000]; // "%USERPROFILE%"
static W_ALLUSERSPROFILE: [WCHAR; 18] = [0x0025, 0x0041, 0x004C, 0x004C, 0x0055, 0x0053, 0x0045, 0x0052, 0x0053, 0x0050, 0x0052, 0x004F, 0x0046, 0x0049, 0x004C, 0x0045, 0x0025, 0x0000]; // "%ALLUSERSPROFILE%"
static W_SYSTEMDRIVE: [WCHAR; 14] = [0x0025, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0044, 0x0072, 0x0069, 0x0076, 0x0065, 0x0025, 0x0000]; // "%SystemDrive%"
static W_DOTDEFAULT: [WCHAR; 9] = [0x002E, 0x0044, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0000]; // ".Default"
static W_CURRENTVERSION: [WCHAR; 42] = [0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x0000]; // "Software\\Microsoft\\Windows\\CurrentVersion"
static W_SHFOLDERS: [WCHAR; 65] = [0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0020, 0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0073, 0x0000]; // "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Shell Folders"
static W_SHUSERFOLDERS: [WCHAR; 70] = [0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x0055, 0x0073, 0x0065, 0x0072, 0x0020, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0020, 0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0073, 0x0000]; // "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders"
static W_PRODUCTOPTIONS: [WCHAR; 48] = [0x0053, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x005C, 0x0050, 0x0072, 0x006F, 0x0064, 0x0075, 0x0063, 0x0074, 0x004F, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0073, 0x0000]; // "SYSTEM\\CurrentControlSet\\Control\\ProductOptions"
static W_PRODUCTTYPE: [WCHAR; 12] = [0x0050, 0x0072, 0x006F, 0x0064, 0x0075, 0x0063, 0x0074, 0x0054, 0x0079, 0x0070, 0x0065, 0x0000]; // "ProductType"
static W_WINNT: [WCHAR; 6] = [0x0057, 0x0069, 0x006E, 0x004E, 0x0054, 0x0000]; // "WinNT"
static W_LANMANNT: [WCHAR; 9] = [0x004C, 0x0061, 0x006E, 0x006D, 0x0061, 0x006E, 0x004E, 0x0054, 0x0000]; // "LanmanNT"
static W_DESKTOP_INI: [WCHAR; 12] = [0x0064, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x002E, 0x0069, 0x006E, 0x0069, 0x0000]; // "desktop.ini"
static W_SHELLCLASSINFO: [WCHAR; 16] = [0x002E, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0043, 0x006C, 0x0061, 0x0073, 0x0073, 0x0049, 0x006E, 0x0066, 0x006F, 0x0000]; // ".ShellClassInfo"
static W_ICONRESOURCE: [WCHAR; 13] = [0x0049, 0x0063, 0x006F, 0x006E, 0x0052, 0x0065, 0x0073, 0x006F, 0x0075, 0x0072, 0x0063, 0x0065, 0x0000]; // "IconResource"
static W_ICON_FMT: [WCHAR; 39] = [0x0025, 0x0025, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0052, 0x006F, 0x006F, 0x0074, 0x0025, 0x0025, 0x005C, 0x0073, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0033, 0x0032, 0x005C, 0x0073, 0x0068, 0x0065, 0x006C, 0x006C, 0x0033, 0x0032, 0x002E, 0x0064, 0x006C, 0x006C, 0x002C, 0x0025, 0x0064, 0x0000]; // "%%SystemRoot%%\\system32\\shell32.dll,%d"
static W_BACKSLASH: [WCHAR; 2] = [0x005C, 0x0000]; // "\\"
static W_FMT_D: [WCHAR; 3] = [0x0025, 0x0064, 0x0000]; // "%d"
static W_FMT_PAREN_D: [WCHAR; 6] = [0x0020, 0x0028, 0x0025, 0x0064, 0x0029, 0x0000]; // " (%d)"
static W_FMT_S_PAREN_D_S: [WCHAR; 10] = [0x0025, 0x0073, 0x0020, 0x0028, 0x0025, 0x0064, 0x0029, 0x0025, 0x0073, 0x0000]; // "%s (%d)%s"

// ════════════════════════════════════════════════════════════════════════════════════════
//  reproduced inline helper (shell32_main.h: __SHCloneStrAtoW)
// ════════════════════════════════════════════════════════════════════════════════════════
unsafe fn SHCloneStrAtoW(target: *mut LPWSTR, source: LPCSTR) -> LPWSTR {
    let len = MultiByteToWideChar(CP_ACP, 0, source, -1, null_mut(), 0);
    *target = SHAlloc((len as usize) * 2) as LPWSTR;
    MultiByteToWideChar(CP_ACP, 0, source, -1, *target, len);
    *target
}

// FIXME: We cannot refresh the RtlGetNtProductType value before reboot.
static mut S_DW_PRODUCT_TYPE: DWORD = 0;
unsafe fn DoGetProductType(product_type: *mut DWORD) -> BOOL {
    let mut hkey: HKEY = null_mut();
    let mut sz_value = [0u16; 9];

    if *addr_of!(S_DW_PRODUCT_TYPE) != 0 {
        *product_type = *addr_of!(S_DW_PRODUCT_TYPE);
        return TRUE;
    }

    *product_type = NtProductServer;

    let error = RegOpenKeyExW(HKEY_LOCAL_MACHINE, W_PRODUCTOPTIONS.as_ptr(), 0, KEY_READ, &mut hkey);
    if error != 0 {
        return FALSE;
    }

    let mut cb_value: DWORD = 9 * 2; // sizeof(szValue)
    let error = RegGetValueW(
        hkey,
        null(),
        W_PRODUCTTYPE.as_ptr(),
        RRF_RT_REG_SZ,
        null_mut(),
        sz_value.as_mut_ptr() as PVOID,
        &mut cb_value,
    );
    if error == 0 {
        if lstrcmpW(sz_value.as_ptr(), W_WINNT.as_ptr()) == 0 {
            *product_type = NtProductWinNt;
        } else if lstrcmpW(sz_value.as_ptr(), W_LANMANNT.as_ptr()) == 0 {
            *product_type = NtProductLanManNt;
        }
    }

    *addr_of_mut!(S_DW_PRODUCT_TYPE) = *product_type;

    RegCloseKey(hkey);
    TRUE
}

/// IsRemovableDrive [SHELL32] — APIENTRY/WINAPI.
#[no_mangle]
pub unsafe extern "system" fn IsRemovableDrive(iDrive: DWORD) -> BOOL {
    // L"C:\\" (the drive letter is overwritten below). assert() dropped (debug-only).
    let mut sz_root: [WCHAR; 4] = [0x0043, 0x003A, 0x005C, 0x0000];
    sz_root[0] = (0x41 + iDrive) as WCHAR; // L'A' + iDrive
    (GetDriveTypeW(sz_root.as_ptr()) == DRIVE_REMOVABLE) as BOOL
}

/*  ########## Combining and Constructing paths ##########  */

unsafe fn PathSearchOnExtensionsW(
    pszPath: LPWSTR,
    ppszDirs: *const LPCWSTR,
    bDoSearch: BOOL,
    dwWhich: DWORD,
) -> BOOL {
    if *PathFindExtensionW(pszPath) != 0 {
        return FALSE;
    }
    if bDoSearch != FALSE {
        PathFindOnPathExW(pszPath, ppszDirs, dwWhich)
    } else {
        PathFileExistsDefExtW(pszPath, dwWhich)
    }
}

// (_WIN32_WINNT >= _WIN32_WINNT_WS03) → compiled.
unsafe fn PathIsAbsoluteW(path: LPCWSTR) -> BOOL {
    (PathIsUNCW(path) != FALSE || (PathGetDriveNumberW(path) != -1 && *path.add(2) == 0x5C)) as BOOL
}

unsafe fn PathMakeAbsoluteW(path: LPWSTR) -> BOOL {
    let mut path1 = [0u16; MAX_PATH];
    if path.is_null() {
        return FALSE;
    }
    let cch = GetCurrentDirectoryW(MAX_PATH as DWORD, path1.as_mut_ptr());
    if cch == 0 || cch > MAX_PATH as DWORD {
        return FALSE;
    }
    (!PathCombineW(path, path1.as_ptr(), path).is_null()) as BOOL
}

unsafe fn PathQualifyExW(pszPath: LPWSTR, pszDir: LPCWSTR, dwFlags: DWORD) {
    let mut sz_temp = [0u16; MAX_PATH];
    let mut sz_root = [0u16; MAX_PATH];

    // Save pszPath into szTemp for rebuilding the path later.
    if FAILED(StringCchCopyW(sz_temp.as_mut_ptr(), MAX_PATH, pszPath)) {
        return;
    }

    // Replace every '/' by '\'.
    FixSlashesAndColonW(sz_temp.as_mut_ptr());

    let mut cch_path_left: INT = MAX_PATH as INT;
    let mut pch_temp: *mut WCHAR = sz_temp.as_mut_ptr();
    let mut b_lfn: BOOL = FALSE;

    if PathIsUNCW(sz_temp.as_ptr()) == FALSE {
        // Non-UNC path. Determine and normalize the root drive.
        let mut i_drive = PathGetDriveNumberW(sz_temp.as_ptr());
        if i_drive == -1 {
            if pszDir.is_null() || FAILED(StringCchCopyW(sz_root.as_mut_ptr(), MAX_PATH, pszDir)) {
                sz_root[0] = UNICODE_NULL;
                GetWindowsDirectoryW(sz_root.as_mut_ptr(), MAX_PATH as UINT);
                i_drive = PathGetDriveNumberW(sz_root.as_ptr());
                if i_drive != -1 {
                    PathBuildRootW(sz_root.as_mut_ptr(), i_drive);
                }
            }

            if sz_temp[0] == 0x5C {
                PathStripToRootW(sz_root.as_mut_ptr());
            }
        } else {
            PathBuildRootW(sz_root.as_mut_ptr(), i_drive);
            pch_temp = pch_temp.add(2);
            if *pch_temp == 0x5C {
                pch_temp = pch_temp.add(1);
            }
        }

        b_lfn = IsLFNDriveW(sz_root.as_ptr());
        if b_lfn == FALSE {
            let mut pch = pch_temp;
            while *pch != UNICODE_NULL {
                if PathIsValidCharW(*pch, VALID_SHORT_PATH_CHAR_CLASSES) == FALSE {
                    *pch = 0x5F; // L'_'
                }
                pch = pch.add(1);
            }
        }

        StringCchCopyW(pszPath, MAX_PATH, sz_root.as_ptr());
        cch_path_left -= lstrlenW(pszPath) + 1;
    } else {
        // UNC path: begins with double backslash.
        b_lfn = IsLFNDriveW(pch_temp);
        if b_lfn != FALSE {
            *pszPath.add(2) = UNICODE_NULL; // cut off
            cch_path_left -= 2 + 1;
            pch_temp = pch_temp.add(2);
        } else {
            let mut pch_slash = StrChrW(pszPath.add(2), 0x5C);
            if !pch_slash.is_null() {
                pch_slash = StrChrW(pch_slash.add(1), 0x5C);
            }

            if !pch_slash.is_null() {
                *pch_slash.add(1) = UNICODE_NULL; // cut off
                let diff = pch_slash.offset_from(pszPath);
                pch_temp = pch_temp.offset(diff);
                cch_path_left -= diff as INT + 1;
            } else {
                b_lfn = TRUE;
                *pszPath.add(2) = UNICODE_NULL; // cut off
                cch_path_left -= 2;
                pch_temp = pch_temp.add(2);
            }
        }
    }

    // Start appending the path components of szTemp to pszPath.
    while *pch_temp != 0 && cch_path_left > 0 {
        if *pch_temp == 0x2E {
            // '.'
            let mut b_dots = false;

            if *pch_temp.add(1) == UNICODE_NULL || *pch_temp.add(1) == 0x5C {
                b_dots = true;
            } else if *pch_temp.add(1) == 0x2E
                && (*pch_temp.add(2) == UNICODE_NULL || *pch_temp.add(2) == 0x5C)
            {
                PathRemoveFileSpecW(pszPath);
                b_dots = true;
            }

            if b_dots {
                while *pch_temp != 0 && *pch_temp != 0x5C {
                    pch_temp = pch_temp.add(1);
                }
                while *pch_temp == 0x5C {
                    pch_temp = pch_temp.add(1);
                }
                continue;
            }
        }

        if PathAddBackslashW(pszPath).is_null() {
            break;
        }
        cch_path_left -= 1;

        let mut pch_path: *mut WCHAR = pszPath.add(r_wcslen(pszPath));

        if b_lfn == FALSE {
            // Copy MS-DOS 8.3 filename.
            let mut pch_dot: *mut WCHAR = null_mut();
            let mut sz_title = [0u16; 8 + 1];
            let mut cch_title: INT = 0;
            let mut sz_dot_extension = [0u16; 1 + 3 + 1];
            let mut cch_dot_extension: INT = 0;

            while *pch_temp != 0 && *pch_temp != 0x5C {
                if *pch_temp == 0x2E {
                    pch_dot = pch_temp;
                    cch_dot_extension = 0;
                    sz_dot_extension = [0u16; 1 + 3 + 1];
                }

                if !pch_dot.is_null() {
                    if cch_dot_extension < 1 + 3 {
                        sz_dot_extension[cch_dot_extension as usize] = *pch_temp;
                        cch_dot_extension += 1;
                    }
                } else if cch_title < 8 {
                    sz_title[cch_title as usize] = *pch_temp;
                    cch_title += 1;
                }

                pch_temp = pch_temp.add(1);
            }

            let mut ich = 0usize;
            while sz_title[ich] != 0 && cch_path_left > 0 {
                *pch_path = sz_title[ich];
                pch_path = pch_path.add(1);
                cch_path_left -= 1;
                ich += 1;
            }

            if !pch_dot.is_null() {
                let mut j = 0usize;
                while sz_dot_extension[j] != 0 && cch_path_left > 0 {
                    *pch_path = sz_dot_extension[j];
                    pch_path = pch_path.add(1);
                    cch_path_left -= 1;
                    j += 1;
                }
            }
        } else {
            while *pch_temp != UNICODE_NULL && *pch_temp != 0x5C && cch_path_left > 0 {
                *pch_path = *pch_temp;
                pch_path = pch_path.add(1);
                pch_temp = pch_temp.add(1);
                cch_path_left -= 1;
            }
        }

        while *pch_temp == 0x5C {
            pch_temp = pch_temp.add(1);
        }

        *pch_path = UNICODE_NULL;
    }

    PathRemoveBackslashW(pszPath);

    if (dwFlags & 1) == 0 {
        let pch_path = CharPrevW(pszPath, pszPath.add(r_wcslen(pszPath)));
        if *pch_path == 0x2E {
            *pch_path = UNICODE_NULL;
        }
    }
}

/// PathAppend [SHELL32.36]
#[no_mangle]
pub unsafe extern "system" fn PathAppendAW(lpszPath1: LPVOID, lpszPath2: LPCVOID) -> BOOL {
    if shell_os_is_unicode() {
        PathAppendW(lpszPath1 as LPWSTR, lpszPath2 as LPCWSTR)
    } else {
        PathAppendA(lpszPath1 as LPSTR, lpszPath2 as LPCSTR)
    }
}

// PathGetExtensionA/W [internal] — return value points to the first char after the dot.
unsafe fn PathGetExtensionA(lpszPath: LPCSTR) -> LPSTR {
    let p = PathFindExtensionA(lpszPath);
    if *p != 0 {
        p.add(1)
    } else {
        p
    }
}
unsafe fn PathGetExtensionW(lpszPath: LPCWSTR) -> LPWSTR {
    let p = PathFindExtensionW(lpszPath);
    if *p != 0 {
        p.add(1)
    } else {
        p
    }
}

/// SHPathGetExtension [SHELL32.158]
#[no_mangle]
pub unsafe extern "system" fn SHPathGetExtensionW(lpszPath: LPCWSTR, _void1: DWORD, _void2: DWORD) -> LPVOID {
    PathGetExtensionW(lpszPath) as LPVOID
}

/// PathRemoveFileSpec [SHELL32.35]
#[no_mangle]
pub unsafe extern "system" fn PathRemoveFileSpecAW(lpszPath: LPVOID) -> BOOL {
    if shell_os_is_unicode() {
        PathRemoveFileSpecW(lpszPath as LPWSTR)
    } else {
        PathRemoveFileSpecA(lpszPath as LPSTR)
    }
}

// PathGetShortPathA/W [internal]
unsafe fn PathGetShortPathA(pszPath: LPSTR) {
    let mut path = [0i8; MAX_PATH];
    if GetShortPathNameA(pszPath, path.as_mut_ptr(), MAX_PATH as DWORD) != 0 {
        lstrcpyA(pszPath, path.as_ptr());
    }
}
unsafe fn PathGetShortPathW(pszPath: LPWSTR) {
    let mut path = [0u16; MAX_PATH];
    if GetShortPathNameW(pszPath, path.as_mut_ptr(), MAX_PATH as DWORD) != 0 {
        lstrcpyW(pszPath, path.as_ptr());
    }
}

/// PathGetShortPath [SHELL32.92]
#[no_mangle]
pub unsafe extern "system" fn PathGetShortPathAW(pszPath: LPVOID) {
    if shell_os_is_unicode() {
        PathGetShortPathW(pszPath as LPWSTR);
    }
    PathGetShortPathA(pszPath as LPSTR);
}

/*  ########## Path Testing ##########  */

/// PathIsRoot [SHELL32.29]
#[no_mangle]
pub unsafe extern "system" fn PathIsRootAW(lpszPath: LPCVOID) -> BOOL {
    if shell_os_is_unicode() {
        PathIsRootW(lpszPath as LPCWSTR)
    } else {
        PathIsRootA(lpszPath as LPCSTR)
    }
}

// PathIsExeA [internal]
unsafe fn PathIsExeA(lpszPath: LPCSTR) -> BOOL {
    let lpsz_extension = PathGetExtensionA(lpszPath);
    // {"exe","com","pif","cmd","bat","scf","scr",NULL}
    // [[u8; 3]; 7], not [&[u8]; 7]: a slice ERASES the length, so `e[0..2]` below could
    // not be proven in bounds and LLVM emitted a panic block per index with the length it
    // inferred on that path -- panic(0,0), panic(1,1), panic(2,2). The 1-element one is
    // indistinguishable in the object from a real bindgen [T; 1] flexible array. Keeping
    // the length in the type makes all three checks disappear; every extension here is
    // exactly three characters, which is what the C relies on too.
    const EXTS: [[u8; 3]; 7] = [*b"exe", *b"com", *b"pif", *b"cmd", *b"bat", *b"scf", *b"scr"];
    for e in EXTS.iter() {
        // narrow case-insensitive compare via lstrcmpiA against a stack ANSI string.
        let mut buf = [0i8; 4];
        buf[0] = e[0] as i8;
        buf[1] = e[1] as i8;
        buf[2] = e[2] as i8;
        buf[3] = 0;
        if lstrcmpiA(lpsz_extension, buf.as_ptr()) == 0 {
            return TRUE;
        }
    }
    FALSE
}

/// PathIsExeW [internal] — EXPORTED (cdecl, ordinal-by-name).
#[no_mangle]
pub unsafe extern "C" fn PathIsExeW(lpszPath: LPCWSTR) -> BOOL {
    let lpsz_extension = PathGetExtensionW(lpszPath);
    // {L"exe",L"com",L"pif",L"cmd",L"bat",L"scf",L"scr",L""}
    static EXTS: [[WCHAR; 4]; 7] = [
        [0x65, 0x78, 0x65, 0],
        [0x63, 0x6F, 0x6D, 0],
        [0x70, 0x69, 0x66, 0],
        [0x63, 0x6D, 0x64, 0],
        [0x62, 0x61, 0x74, 0],
        [0x73, 0x63, 0x66, 0],
        [0x73, 0x63, 0x72, 0],
    ];
    // The C walks to its L"" sentinel row; this table has no sentinel row, so iterate it.
    // (It did walk to a sentinel: every non-executable path indexed past the end and
    // panicked -- SHObjectProperties on a folder, found by tools/payloads/shelltail.)
    for ext in EXTS.iter() {
        if wcsicmp(lpsz_extension, ext.as_ptr()) == 0 {
            return TRUE;
        }
    }
    FALSE
}

/// PathIsExe [SHELL32.43]
#[no_mangle]
pub unsafe extern "system" fn PathIsExeAW(path: LPCVOID) -> BOOL {
    if shell_os_is_unicode() {
        PathIsExeW(path as LPCWSTR)
    } else {
        PathIsExeA(path as LPCSTR)
    }
}

/// PathFileExists [SHELL32.45]
#[no_mangle]
pub unsafe extern "system" fn PathFileExistsAW(lpszPath: LPCVOID) -> BOOL {
    if shell_os_is_unicode() {
        PathFileExistsW(lpszPath as LPCWSTR)
    } else {
        PathFileExistsA(lpszPath as LPCSTR)
    }
}

/// IsLFNDriveA [SHELL32.41]
#[no_mangle]
pub unsafe extern "system" fn IsLFNDriveA(lpszPath: LPCSTR) -> BOOL {
    let mut sz_buff_w = [0u16; MAX_PATH];
    let mut psz_w: LPCWSTR = null();
    if !lpszPath.is_null() {
        SHAnsiToUnicode(lpszPath, sz_buff_w.as_mut_ptr(), MAX_PATH as INT);
        psz_w = sz_buff_w.as_ptr();
    }
    IsLFNDriveW(psz_w)
}

/// IsLFNDriveW [SHELL32.42]
#[no_mangle]
pub unsafe extern "system" fn IsLFNDriveW(lpszPath: LPCWSTR) -> BOOL {
    let mut cch_max_file_name: DWORD = 0;
    let mut sz_root = [0u16; MAX_PATH];
    let mut psz_path = lpszPath;

    if psz_path.is_null() || *psz_path == UNICODE_NULL {
        sz_root[0] = 0;
        GetWindowsDirectoryW(sz_root.as_mut_ptr(), MAX_PATH as UINT);
        psz_path = sz_root.as_ptr();
    }

    if PathIsUNCW(psz_path) != FALSE {
        StringCchCopyW(sz_root.as_mut_ptr(), MAX_PATH, psz_path);
        PathStripToRootW(sz_root.as_mut_ptr());

        if StrChrW(sz_root.as_ptr().add(2), 0x5C).is_null() {
            return TRUE; // LFN
        }

        StringCchCatW(sz_root.as_mut_ptr(), MAX_PATH, W_BACKSLASH.as_ptr());
    } else {
        let i_drive = ((*psz_path as i32 - 0x41) & 0x1F) as INT;
        PathBuildRootW(sz_root.as_mut_ptr(), i_drive);

        if IsRemovableDrive(i_drive as DWORD) == FALSE {
            // FIXME: Cache correctly
        }
    }

    // GetVolumeInformation requires a root path.
    if GetVolumeInformationW(
        sz_root.as_ptr(),
        null_mut(),
        0,
        null_mut(),
        &mut cch_max_file_name,
        null_mut(),
        null_mut(),
        0,
    ) == FALSE
    {
        // Don't return FALSE when GetVolumeInformationW fails.
        return TRUE;
    }

    (cch_max_file_name > MSDOS_8DOT3_LEN) as BOOL
}

/// IsLFNDrive [SHELL32.119]
#[no_mangle]
pub unsafe extern "system" fn IsLFNDriveAW(lpszPath: LPCVOID) -> BOOL {
    if shell_os_is_unicode() {
        IsLFNDriveW(lpszPath as LPCWSTR)
    } else {
        IsLFNDriveA(lpszPath as LPCSTR)
    }
}

/*  ########## Creating Something Unique ##########  */

// PathMakeUniqueNameA [internal] — stub (returns TRUE, as in C).
unsafe fn PathMakeUniqueNameA(
    _lpszBuffer: LPSTR,
    _dwBuffSize: DWORD,
    _lpszShortName: LPCSTR,
    _lpszLongName: LPCSTR,
    _lpszPathName: LPCSTR,
) -> BOOL {
    TRUE
}

// PathMakeUniqueNameW [internal] (__REACTOS__)
unsafe fn PathMakeUniqueNameW(
    pszUniqueName: LPWSTR,
    cchMax: UINT,
    pszTemplate: LPCWSTR,
    pszLongPlate: LPCWSTR,
    pszDir: LPCWSTR,
) -> BOOL {
    if cchMax == 0 || pszUniqueName.is_null() {
        return FALSE;
    }

    *pszUniqueName = UNICODE_NULL;

    let mut psz_dest: LPWSTR = pszUniqueName;
    let mut dir_length: INT = 0;
    if !pszDir.is_null() {
        if StringCchCopyW(pszUniqueName, (cchMax - 1) as usize, pszDir) != S_OK {
            return FALSE;
        }
        psz_dest = PathAddBackslashW(pszUniqueName);
        if psz_dest.is_null() {
            return FALSE;
        }
        dir_length = lstrlenW(pszDir);
    }

    let mut psz_title: LPCWSTR = if !pszLongPlate.is_null() {
        pszLongPlate
    } else {
        pszTemplate
    };
    let pch_dot_ext: LPCWSTR;
    let mut format_string: LPCWSTR = W_FMT_D.as_ptr();
    let max_count: INT;
    let mut cch_title: INT;

    // NTDDI_VERSION >= NTDDI_VISTA → the extra `|| pszDir` clause is NOT present.
    if psz_title.is_null() || IsLFNDriveW(pszDir) == FALSE {
        if pszTemplate.is_null() {
            return FALSE;
        }

        pch_dot_ext = PathFindExtensionW(pszTemplate);

        cch_title = pch_dot_ext.offset_from(pszTemplate) as INT;
        if cch_title > 1 {
            let mut pch = pch_dot_ext.sub(1);
            while cch_title > 1 && (0x30..=0x39).contains(&*pch) {
                cch_title -= 1;
                pch = pch.sub(1);
            }
        }

        if cch_title > MSDOS_8DOT3_FILENAME_TITLE_LEN - 1 {
            cch_title = MSDOS_8DOT3_FILENAME_TITLE_LEN - 1;
        }

        let ext_length = lstrlenW(pch_dot_ext);
        while cch_title > 1
            && (dir_length as i64 + cch_title as i64 + ext_length as i64 + 1 > (cchMax as i64 - 1))
        {
            cch_title -= 1;
        }

        max_count = if cch_title <= 0 {
            1
        } else if cch_title == 1 {
            10
        } else {
            100
        };

        psz_title = pszTemplate;
    } else {
        let mut open_paren = StrChrW(psz_title, 0x28); // '('
        if !open_paren.is_null() {
            while !open_paren.is_null() {
                let mut pch = open_paren.add(1);
                while *pch >= 0x30 && *pch <= 0x39 {
                    pch = pch.add(1);
                }
                if *pch == 0x29 {
                    // ')'
                    break;
                }
                open_paren = StrChrW(open_paren.add(1), 0x28);
            }

            if !open_paren.is_null() {
                pch_dot_ext = open_paren.add(1);
                cch_title = pch_dot_ext.offset_from(psz_title) as INT;
            } else {
                pch_dot_ext = PathFindExtensionW(psz_title);
                cch_title = pch_dot_ext.offset_from(psz_title) as INT;
                format_string = W_FMT_PAREN_D.as_ptr();
            }
        } else {
            pch_dot_ext = PathFindExtensionW(psz_title);
            cch_title = pch_dot_ext.offset_from(psz_title) as INT;
            format_string = W_FMT_PAREN_D.as_ptr();
        }

        let remaining_chars: i64 =
            cchMax as i64 - (dir_length as i64 + cch_title as i64 + (lstrlenW(format_string) as i64 - 2));
        max_count = if remaining_chars <= 0 {
            1
        } else if remaining_chars == 1 {
            10
        } else if remaining_chars == 2 {
            100
        } else {
            1000
        };
    }

    if StringCchCopyNW(
        psz_dest,
        (cchMax as i64 - dir_length as i64).max(0) as usize,
        psz_title,
        cch_title.max(0) as usize,
    ) != S_OK
    {
        return FALSE;
    }

    let pch_title: LPWSTR = psz_dest.add(cch_title.max(0) as usize);
    let mut count: INT = 1;
    while count < max_count {
        let mut temp_name = [0u16; MAX_PATH];
        if string_cch_printf(temp_name.as_mut_ptr(), MAX_PATH, format_string, &[FA::I(count)]) != S_OK
            || StringCchCatW(temp_name.as_mut_ptr(), MAX_PATH, pch_dot_ext) != S_OK
        {
            return FALSE;
        }

        let avail = (cchMax as i64 - pch_title.offset_from(pszUniqueName) as i64).max(0) as usize;
        if StringCchCopyW(pch_title, avail, temp_name.as_ptr()) != S_OK {
            return FALSE;
        }

        if PathFileExistsW(pszUniqueName) == FALSE {
            return TRUE;
        }
        count += 1;
    }

    *pszUniqueName = UNICODE_NULL;
    FALSE
}

/// PathMakeUniqueName [SHELL32.47]
#[no_mangle]
pub unsafe extern "system" fn PathMakeUniqueNameAW(
    lpszBuffer: LPVOID,
    dwBuffSize: DWORD,
    lpszShortName: LPCVOID,
    lpszLongName: LPCVOID,
    lpszPathName: LPCVOID,
) -> BOOL {
    if shell_os_is_unicode() {
        PathMakeUniqueNameW(
            lpszBuffer as LPWSTR,
            dwBuffSize,
            lpszShortName as LPCWSTR,
            lpszLongName as LPCWSTR,
            lpszPathName as LPCWSTR,
        )
    } else {
        PathMakeUniqueNameA(
            lpszBuffer as LPSTR,
            dwBuffSize,
            lpszShortName as LPCSTR,
            lpszLongName as LPCSTR,
            lpszPathName as LPCSTR,
        )
    }
}

/// PathYetAnotherMakeUniqueName [SHELL32.75]
#[no_mangle]
pub unsafe extern "system" fn PathYetAnotherMakeUniqueName(
    buffer: LPWSTR,
    path: LPCWSTR,
    shortname: LPCWSTR,
    longname: LPCWSTR,
) -> BOOL {
    let mut path_w = [0u16; MAX_PATH];
    let mut ret_w = [0u16; MAX_PATH];
    let mut i: INT = 2;

    let file: LPCWSTR = if !longname.is_null() { longname } else { shortname };
    PathCombineW(path_w.as_mut_ptr(), path, file);
    strcpyW(ret_w.as_mut_ptr(), path_w.as_ptr());
    PathRemoveExtensionW(path_w.as_mut_ptr());

    let ext = PathFindExtensionW(file);

    // now try to make it unique
    while PathFileExistsW(ret_w.as_ptr()) != FALSE {
        // sprintfW(retW, L"%s (%d)%s", pathW, i, ext)
        string_cch_printf(
            ret_w.as_mut_ptr(),
            MAX_PATH,
            W_FMT_S_PAREN_D_S.as_ptr(),
            &[FA::S(path_w.as_ptr()), FA::I(i), FA::S(ext)],
        );
        i += 1;
    }

    strcpyW(buffer, ret_w.as_ptr());
    TRUE
}

/*  ########## cleaning and resolving paths ##########  */

/// PathCleanupSpec [SHELL32.171] — lpszFile is changed in place.
#[no_mangle]
pub unsafe extern "system" fn PathCleanupSpec(lpszPathW: LPCWSTR, lpszFileW: LPWSTR) -> INT {
    let mut i: INT = 0;
    let mut rc: DWORD = 0;
    let mut length: INT = 0;

    if shell_os_is_unicode() {
        let mut p = lpszFileW;
        if !lpszPathW.is_null() {
            length = r_wcslen(lpszPathW) as INT;
        }
        while *p != 0 {
            let gct = PathGetCharTypeW(*p);
            if gct == GCT_INVALID || gct == GCT_WILD || gct == GCT_SEPARATOR {
                *lpszFileW.add(i as usize) = 0x2D; // '-'
                rc |= PCS_REPLACEDCHAR;
            } else {
                *lpszFileW.add(i as usize) = *p;
            }
            i += 1;
            p = p.add(1);
            if length + i == MAX_PATH as INT {
                rc |= PCS_FATAL | PCS_PATHTOOLONG;
                break;
            }
        }
        *lpszFileW.add(i as usize) = 0;
    } else {
        let lpsz_file_a = lpszFileW as LPSTR;
        let lpsz_path_a = lpszPathW as LPCSTR;
        let mut p = lpsz_file_a;
        if !lpsz_path_a.is_null() {
            length = r_strlen(lpsz_path_a) as INT;
        }
        while *p != 0 {
            let gct = PathGetCharTypeA(*p as u8);
            if gct == GCT_INVALID || gct == GCT_WILD || gct == GCT_SEPARATOR {
                *lpsz_file_a.add(i as usize) = 0x2D; // '-'
                rc |= PCS_REPLACEDCHAR;
            } else {
                *lpsz_file_a.add(i as usize) = *p;
            }
            i += 1;
            p = p.add(1);
            if length + i == MAX_PATH as INT {
                rc |= PCS_FATAL | PCS_PATHTOOLONG;
                break;
            }
        }
        *lpsz_file_a.add(i as usize) = 0;
    }
    rc as INT
}

/// PathQualifyA [SHELL32]
#[no_mangle]
pub unsafe extern "system" fn PathQualifyA(pszPath: LPSTR) {
    let mut sz_path = [0u16; MAX_PATH];
    SHAnsiToUnicode(pszPath, sz_path.as_mut_ptr(), MAX_PATH as INT);
    PathQualifyW(sz_path.as_mut_ptr());
    SHUnicodeToAnsi(sz_path.as_ptr(), pszPath, MAX_PATH as INT);
}

/// PathQualifyW [SHELL32]
#[no_mangle]
pub unsafe extern "system" fn PathQualifyW(pszPath: LPWSTR) {
    PathQualifyExW(pszPath, null(), 0);
}

/// PathQualify [SHELL32.49]
#[no_mangle]
pub unsafe extern "system" fn PathQualifyAW(pszPath: LPVOID) {
    if shell_os_is_unicode() {
        PathQualifyW(pszPath as LPWSTR);
    } else {
        PathQualifyA(pszPath as LPSTR);
    }
}

#[no_mangle]
pub unsafe extern "system" fn PathResolveA(path: LPSTR, dirs: *mut LPCSTR, flags: DWORD) -> BOOL {
    let mut ret: BOOL = FALSE;
    let mut dirs_w: *mut LPWSTR = null_mut();
    let mut c_dirs: DWORD = 0;
    let mut path_w = [0u16; MAX_PATH];

    if !dirs.is_null() {
        c_dirs = 0;
        while !(*dirs.add(c_dirs as usize)).is_null() {
            c_dirs += 1;
        }

        let cb_dirs = (c_dirs as usize + 1) * core::mem::size_of::<LPWSTR>();
        dirs_w = SHAlloc(cb_dirs) as *mut LPWSTR;
        if dirs_w.is_null() {
            // Cleanup (dirs_w is null → nothing to free)
            return ret;
        }

        // ZeroMemory(dirsW, cbDirs)
        let mut z = 0usize;
        while z <= c_dirs as usize {
            *dirs_w.add(z) = null_mut();
            z += 1;
        }
        let mut i_dir = 0u32;
        while i_dir < c_dirs {
            SHCloneStrAtoW(dirs_w.add(i_dir as usize), *dirs.add(i_dir as usize));
            if (*dirs_w.add(i_dir as usize)).is_null() {
                // goto Cleanup
                let mut k = 0u32;
                while k < c_dirs {
                    SHFree(*dirs_w.add(k as usize) as LPVOID);
                    k += 1;
                }
                SHFree(dirs_w as LPVOID);
                return ret;
            }
            i_dir += 1;
        }
    }

    SHAnsiToUnicode(path, path_w.as_mut_ptr(), MAX_PATH as INT);

    ret = PathResolveW(path_w.as_mut_ptr(), dirs_w as *mut LPCWSTR, flags);
    if ret != FALSE {
        SHUnicodeToAnsi(path_w.as_ptr(), path, MAX_PATH as INT);
    }

    // Cleanup
    if !dirs_w.is_null() {
        let mut i_dir = 0u32;
        while i_dir < c_dirs {
            SHFree(*dirs_w.add(i_dir as usize) as LPVOID);
            i_dir += 1;
        }
        SHFree(dirs_w as LPVOID);
    }
    ret
}

// CheckAbsoluteAndFinish tail of PathResolveW (_WIN32_WINNT >= WS03 branch).
unsafe fn path_resolve_check_absolute(path: LPWSTR, flags: DWORD) -> BOOL {
    if (flags & PRF_REQUIREABSOLUTE) == 0 || PathIsAbsoluteW(path) != FALSE {
        return TRUE;
    }
    if PathMakeAbsoluteW(path) == FALSE {
        return FALSE;
    }
    PathFileExistsAndAttributesW(path, null_mut())
}

#[no_mangle]
pub unsafe extern "system" fn PathResolveW(path: LPWSTR, dirs: *mut LPCWSTR, flags: DWORD) -> BOOL {
    let mut dw_which: DWORD = WHICH_DEFAULT;

    if flags & PRF_DONTFINDLNK != 0 {
        dw_which &= !WHICH_LNK;
    }

    if flags & PRF_VERIFYEXISTS != 0 {
        SetLastError(ERROR_FILE_NOT_FOUND as DWORD);
    }

    PathUnquoteSpacesW(path);

    if PathIsRootW(path) != FALSE {
        if *path == 0x5C && *path.add(1) == UNICODE_NULL {
            let dir = if flags & PRF_FIRSTDIRDEF != 0 { *dirs } else { null() };
            PathQualifyExW(path, dir, 0);
        }
        if flags & PRF_VERIFYEXISTS != 0 {
            return PathFileExistsAndAttributesW(path, null_mut());
        }
        return TRUE;
    }

    if PathIsFileSpecW(path) != FALSE {
        if flags & PRF_TRYPROGRAMEXTENSIONS != 0
            && PathSearchOnExtensionsW(path, dirs, TRUE, dw_which) != FALSE
        {
            return TRUE;
        }

        if PathFindOnPathW(path, dirs) != FALSE {
            return path_resolve_check_absolute(path, flags);
        }

        return FALSE;
    }

    if PathIsURLW(path) != FALSE {
        return FALSE;
    }

    let dir = if flags & PRF_FIRSTDIRDEF != 0 { *dirs } else { null() };
    PathQualifyExW(path, dir, 1);

    if flags & PRF_VERIFYEXISTS == 0 {
        return TRUE;
    }

    if flags & PRF_TRYPROGRAMEXTENSIONS == 0
        || PathSearchOnExtensionsW(path, dirs, FALSE, dw_which) == FALSE
    {
        if PathFileExistsAndAttributesW(path, null_mut()) == FALSE {
            return FALSE;
        }
    }

    path_resolve_check_absolute(path, flags)
}

/// PathResolve [SHELL32.51]
#[no_mangle]
pub unsafe extern "system" fn PathResolveAW(path: LPVOID, paths: *mut LPCVOID, flags: DWORD) -> BOOL {
    if shell_os_is_unicode() {
        PathResolveW(path as LPWSTR, paths as *mut LPCWSTR, flags)
    } else {
        PathResolveA(path as LPSTR, paths as *mut LPCSTR, flags)
    }
}

// PathProcessCommandA [internal] — stub mirror of the C.
unsafe fn PathProcessCommandA(lpszPath: LPCSTR, lpszBuff: LPSTR, _dwBuffSize: DWORD, _dwFlags: DWORD) -> LONG {
    if lpszPath.is_null() {
        return -1;
    }
    if !lpszBuff.is_null() {
        r_strcpy(lpszBuff, lpszPath);
    }
    r_strlen(lpszPath) as LONG
}

/// PathProcessCommand [SHELL32.653]
#[no_mangle]
pub unsafe extern "system" fn PathProcessCommandAW(
    lpszPath: LPCVOID,
    lpszBuff: LPVOID,
    dwBuffSize: DWORD,
    dwFlags: DWORD,
) -> LONG {
    if shell_os_is_unicode() {
        PathProcessCommandW(lpszPath as LPCWSTR, lpszBuff as LPWSTR, dwBuffSize, dwFlags)
    } else {
        PathProcessCommandA(lpszPath as LPCSTR, lpszBuff as LPSTR, dwBuffSize, dwFlags)
    }
}

/*  ########## special ##########  */

/// SHGetSpecialFolderID — EXPORTED (cdecl).
#[no_mangle]
pub unsafe extern "C" fn SHGetSpecialFolderID(pszName: LPCWSTR) -> INT {
    let mut idx: DWORD = 0;
    while idx < ARRAY_SIZE_CSIDL() {
        let p_data = csidl(idx);
        if !p_data.szValueName.is_null() && lstrcmpiW(pszName, p_data.szValueName) == 0 {
            return idx as INT;
        }
        idx += 1;
    }
    -1
}

/// Shell_ParseSpecialFolder — EXPORTED (cdecl).
#[no_mangle]
pub unsafe extern "C" fn Shell_ParseSpecialFolder(
    pszStart: LPCWSTR,
    ppch: *mut LPWSTR,
    pcch: *mut INT,
) -> INT {
    let psz_path: LPCWSTR;
    let mut sz_path = [0u16; MAX_PATH];

    let pch_backslash = StrChrW(pszStart, 0x5C);
    if !pch_backslash.is_null() {
        write_volatile(ppch, pch_backslash.add(1));
        let cch = pch_backslash.offset_from(pszStart) as INT + 1;
        write_volatile(pcch, cch);
        StrCpyNW(sz_path.as_mut_ptr(), pszStart, min_i32(cch, MAX_PATH as INT));
        psz_path = sz_path.as_ptr();
    } else {
        write_volatile(ppch, null_mut());
        write_volatile(pcch, lstrlenW(pszStart));
        psz_path = pszStart;
    }

    SHGetSpecialFolderID(psz_path)
}

// _SHExpandEnvironmentStrings (__REACTOS__).
unsafe fn SHExpandEnvironmentStrings_(
    hToken: HANDLE,
    szSrc: LPCWSTR,
    szDest: LPWSTR,
    cchDest: DWORD,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut sz_temp = [0u16; MAX_PATH];

    if szSrc.is_null() || szDest.is_null() {
        return E_INVALIDARG;
    }

    // short-circuit if there's nothing to expand
    if *szSrc != 0x25 {
        strcpyW(szDest, szSrc);
        return S_OK;
    }

    hr = S_OK;

    *szDest = 0;
    strcpyW(sz_temp.as_mut_ptr(), szSrc);
    while SUCCEEDED(hr) && sz_temp[0] == 0x25 {
        let mut do_fallback = false;

        if strncmpiW(sz_temp.as_ptr(), W_ALLUSERSPROFILE.as_ptr(), 17) == 0 {
            // ARRAY_SIZE(L"%ALLUSERSPROFILE%")-1 == 17
            let mut cch_size = cchDest;
            if GetAllUsersProfileDirectoryW(szDest, &mut cch_size) == FALSE {
                do_fallback = true;
            } else {
                PathAppendW(szDest, sz_temp.as_ptr().add(17));
            }
        } else if strncmpiW(sz_temp.as_ptr(), W_USERPROFILE.as_ptr(), 13) == 0 {
            // ARRAY_SIZE(L"%USERPROFILE%")-1 == 13
            let mut cch_size = cchDest;
            if _SHGetUserProfileDirectoryW(hToken, szDest, &mut cch_size) == FALSE {
                do_fallback = true;
            } else {
                PathAppendW(szDest, sz_temp.as_ptr().add(13));
            }
        } else if strncmpiW(sz_temp.as_ptr(), W_SYSTEMDRIVE.as_ptr(), 13) == 0 {
            // ARRAY_SIZE(L"%SystemDrive%")-1 == 13
            if GetSystemDirectoryW(szDest, cchDest) == 0 {
                do_fallback = true;
            } else {
                strcpyW(szDest.add(3), sz_temp.as_ptr().add(13 + 1));
            }
        } else {
            do_fallback = true;
        }

        if do_fallback {
            let ret = SHExpandEnvironmentStringsForUserW(hToken, sz_temp.as_ptr(), szDest, cchDest);
            if ret > cchDest {
                hr = E_NOT_SUFFICIENT_BUFFER;
            } else if ret == 0 {
                hr = HRESULT_FROM_WIN32(GetLastError() as LONG);
            } else if strcmpW(sz_temp.as_ptr(), szDest) == 0 {
                break; // nothing expanded
            }
        }

        if SUCCEEDED(hr) {
            strcpyW(sz_temp.as_mut_ptr(), szDest);
        }
    }

    hr
}

// _SHGetUserShellFolderPath (__REACTOS__).
unsafe fn _SHGetUserShellFolderPath(
    rootKey: HKEY,
    hToken: HANDLE,
    userPrefix: LPCWSTR,
    value: LPCWSTR,
    path: LPWSTR,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut shell_folder_path = [0u16; MAX_PATH];
    let mut user_shell_folder_path = [0u16; MAX_PATH];
    let p_shell_folder_path: LPCWSTR;
    let p_user_shell_folder_path: LPCWSTR;
    let mut user_shell_folder_key: HKEY = null_mut();
    let mut shell_folder_key: HKEY = null_mut();
    let mut dw_type: DWORD = 0;
    let mut dw_path_len: DWORD;

    if !userPrefix.is_null() {
        strcpyW(shell_folder_path.as_mut_ptr(), userPrefix);
        PathAddBackslashW(shell_folder_path.as_mut_ptr());
        strcatW(shell_folder_path.as_mut_ptr(), W_SHFOLDERS.as_ptr());
        p_shell_folder_path = shell_folder_path.as_ptr();
        strcpyW(user_shell_folder_path.as_mut_ptr(), userPrefix);
        PathAddBackslashW(user_shell_folder_path.as_mut_ptr());
        strcatW(user_shell_folder_path.as_mut_ptr(), W_SHUSERFOLDERS.as_ptr());
        p_user_shell_folder_path = user_shell_folder_path.as_ptr();
    } else {
        p_user_shell_folder_path = W_SHUSERFOLDERS.as_ptr();
        p_shell_folder_path = W_SHFOLDERS.as_ptr();
    }

    if RegCreateKeyW(rootKey, p_shell_folder_path, &mut shell_folder_key) != 0 {
        return E_FAIL;
    }
    if RegCreateKeyW(rootKey, p_user_shell_folder_path, &mut user_shell_folder_key) != 0 {
        RegCloseKey(shell_folder_key);
        return E_FAIL;
    }

    dw_path_len = (MAX_PATH * 2) as DWORD;
    if RegQueryValueExW(
        user_shell_folder_key,
        value,
        null_mut(),
        &mut dw_type,
        path as LPBYTE,
        &mut dw_path_len,
    ) == 0
        && (dw_type == REG_EXPAND_SZ || dw_type == REG_SZ)
    {
        *path.add((dw_path_len / 2) as usize) = 0;
        if dw_type == REG_EXPAND_SZ && *path == 0x25 {
            let mut sz_temp = [0u16; MAX_PATH];
            hr = SHExpandEnvironmentStrings_(hToken, path, sz_temp.as_mut_ptr(), MAX_PATH as DWORD);
            if FAILED(hr) {
                RegCloseKey(shell_folder_key);
                RegCloseKey(user_shell_folder_key);
                return hr;
            }
            lstrcpynW(path, sz_temp.as_ptr(), MAX_PATH as INT);
        }
        let ret = RegSetValueExW(
            shell_folder_key,
            value,
            0,
            REG_SZ,
            path as *const BYTE,
            ((r_wcslen(path) + 1) * 2) as DWORD,
        );
        hr = if ret != ERROR_SUCCESS_L {
            HRESULT_FROM_WIN32(ret)
        } else {
            S_OK
        };
    } else {
        hr = E_FAIL;
    }

    RegCloseKey(shell_folder_key);
    RegCloseKey(user_shell_folder_key);
    hr
}

/// _SHGetUserProfileDirectoryW — EXPORTED (cdecl, obj `__SHGetUserProfileDirectoryW`).
#[no_mangle]
pub unsafe extern "C" fn _SHGetUserProfileDirectoryW(
    hToken: HANDLE,
    szPath: LPWSTR,
    lpcchPath: LPDWORD,
) -> BOOL {
    let result: BOOL;
    if hToken.is_null() {
        let mut h: HANDLE = null_mut();
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut h);
        result = GetUserProfileDirectoryW(h, szPath, lpcchPath);
        CloseHandle(h);
    } else if hToken as isize == -1 {
        result = GetDefaultUserProfileDirectoryW(szPath, lpcchPath);
    } else {
        result = GetUserProfileDirectoryW(hToken, szPath, lpcchPath);
    }
    result
}

// _SHGetDefaultValue (__REACTOS__).
unsafe fn _SHGetDefaultValue(hToken: HANDLE, mut folder: BYTE, pszPath: LPWSTR) -> HRESULT {
    let mut hr: HRESULT;
    let mut resource_path = [0u16; MAX_PATH];

    if folder as DWORD >= ARRAY_SIZE_CSIDL() {
        return E_INVALIDARG;
    }
    if pszPath.is_null() {
        return E_INVALIDARG;
    }

    if !hToken.is_null() && hToken as isize != -1 {
        // FIXME: unsupported for user other than current or default
    }

    let is_win64 = core::mem::size_of::<*const c_void>() > core::mem::size_of::<INT>();
    if !is_win64 {
        let mut is_wow64: BOOL = FALSE;
        match folder as u32 {
            x if x == CSIDL_PROGRAM_FILES || x == CSIDL_PROGRAM_FILESX86 => {
                IsWow64Process(GetCurrentProcess(), &mut is_wow64);
                folder = if is_wow64 != FALSE {
                    CSIDL_PROGRAM_FILESX86 as BYTE
                } else {
                    CSIDL_PROGRAM_FILES as BYTE
                };
            }
            x if x == CSIDL_PROGRAM_FILES_COMMON || x == CSIDL_PROGRAM_FILES_COMMONX86 => {
                IsWow64Process(GetCurrentProcess(), &mut is_wow64);
                folder = if is_wow64 != FALSE {
                    CSIDL_PROGRAM_FILES_COMMONX86 as BYTE
                } else {
                    CSIDL_PROGRAM_FILES_COMMON as BYTE
                };
            }
            _ => {}
        }
    }

    let entry = csidl(folder as DWORD);
    match entry.typ {
        CSIDL_Type_User => {
            strcpyW(pszPath, W_USERPROFILE.as_ptr());
        }
        CSIDL_Type_InMyDocuments => {
            strcpyW(pszPath, W_USERPROFILE.as_ptr());
            let mut product_type: DWORD = 0;
            if DoGetProductType(&mut product_type) != FALSE && product_type == NtProductWinNt {
                let mydocs = csidl(CSIDL_MYDOCUMENTS as DWORD);
                if is_intresource(mydocs.szDefaultPath) {
                    let mut sz_item = [0u16; MAX_PATH];
                    LoadStringW(
                        shell32_hInstance,
                        loword_res(mydocs.szDefaultPath),
                        sz_item.as_mut_ptr(),
                        MAX_PATH as INT,
                    );
                    PathAppendW(pszPath, sz_item.as_ptr());
                } else {
                    PathAppendW(pszPath, mydocs.szDefaultPath);
                }
            }
        }
        CSIDL_Type_AllUsers => {
            strcpyW(pszPath, W_ALLUSERSPROFILE.as_ptr());
        }
        CSIDL_Type_CurrVer => {
            strcpyW(pszPath, W_SYSTEMDRIVE.as_ptr());
        }
        _ => { /* no corresponding env. var, do nothing */ }
    }

    hr = S_OK;
    if !entry.szDefaultPath.is_null() {
        if is_intresource(entry.szDefaultPath) {
            if LoadStringW(
                shell32_hInstance,
                loword_res(entry.szDefaultPath),
                resource_path.as_mut_ptr(),
                MAX_PATH as INT,
            ) != 0
            {
                PathAppendW(pszPath, resource_path.as_ptr());
            } else {
                hr = E_FAIL;
            }
        } else {
            PathAppendW(pszPath, entry.szDefaultPath);
        }
    }
    hr
}

// _SHGetCurrentVersionPath.
unsafe fn _SHGetCurrentVersionPath(dwFlags: DWORD, folder: BYTE, pszPath: LPWSTR) -> HRESULT {
    let mut hr: HRESULT;

    if folder as DWORD >= ARRAY_SIZE_CSIDL() {
        return E_INVALIDARG;
    }
    if csidl(folder as DWORD).typ != CSIDL_Type_CurrVer {
        return E_INVALIDARG;
    }
    if pszPath.is_null() {
        return E_INVALIDARG;
    }

    if dwFlags & SHGFP_TYPE_DEFAULT != 0 {
        hr = _SHGetDefaultValue(null_mut(), folder, pszPath);
    } else {
        let mut h_key: HKEY = null_mut();
        if RegCreateKeyW(HKEY_LOCAL_MACHINE, W_CURRENTVERSION.as_ptr(), &mut h_key) != 0 {
            hr = E_FAIL;
        } else {
            let mut dw_type: DWORD = 0;
            let mut dw_path_len: DWORD = (MAX_PATH * 2) as DWORD;

            if RegQueryValueExW(
                h_key,
                csidl(folder as DWORD).szValueName,
                null_mut(),
                &mut dw_type,
                pszPath as LPBYTE,
                &mut dw_path_len,
            ) != 0
                || (dw_type != REG_SZ && dw_type != REG_EXPAND_SZ)
            {
                hr = _SHGetDefaultValue(null_mut(), folder, pszPath);
                dw_type = REG_EXPAND_SZ;

                let mut do_set = true;
                if folder as u32 == CSIDL_PROGRAM_FILESX86
                    || folder as u32 == CSIDL_PROGRAM_FILES_COMMONX86
                {
                    // these two should never be set on 32-bit setups
                    let is_win64 = core::mem::size_of::<*const c_void>() > core::mem::size_of::<INT>();
                    if !is_win64 {
                        let mut is_wow64: BOOL = FALSE;
                        IsWow64Process(GetCurrentProcess(), &mut is_wow64);
                        if is_wow64 == FALSE {
                            do_set = false; // break out of the switch (skip RegSetValueExW)
                        }
                    }
                }
                if do_set {
                    RegSetValueExW(
                        h_key,
                        csidl(folder as DWORD).szValueName,
                        0,
                        dw_type,
                        pszPath as *const BYTE,
                        ((r_wcslen(pszPath) + 1) * 2) as DWORD,
                    );
                }
            } else {
                *pszPath.add((dw_path_len / 2) as usize) = 0;
                hr = S_OK;
            }
            RegCloseKey(h_key);
        }
    }
    hr
}

// _GetUserSidStringFromToken.
unsafe fn _GetUserSidStringFromToken(token: HANDLE) -> LPWSTR {
    let mut info_buffer = [0u8; 64];
    let mut user_info: *mut c_void = info_buffer.as_mut_ptr() as *mut c_void;
    let mut info_size: DWORD = 0;
    let mut sid_str: LPWSTR = null_mut();

    if GetTokenInformation(
        token,
        TokenUser,
        info_buffer.as_mut_ptr() as LPVOID,
        64,
        &mut info_size,
    ) == FALSE
    {
        if GetLastError() != ERROR_INSUFFICIENT_BUFFER {
            return null_mut();
        }
        user_info = HeapAlloc(GetProcessHeap(), 0, info_size as SIZE_T);
        if user_info.is_null() {
            return null_mut();
        }
        if GetTokenInformation(token, TokenUser, user_info as LPVOID, info_size, &mut info_size) == FALSE {
            HeapFree(GetProcessHeap(), 0, user_info);
            return null_mut();
        }
    }

    // TOKEN_USER { SID_AND_ATTRIBUTES User { PSID Sid; ... } } — Sid is at offset 0.
    let sid: PVOID = *(user_info as *const PVOID);
    if ConvertSidToStringSidW(sid, &mut sid_str) == FALSE {
        sid_str = null_mut();
    }

    if user_info != info_buffer.as_mut_ptr() as *mut c_void {
        HeapFree(GetProcessHeap(), 0, user_info);
    }

    sid_str
}

// _SHGetUserProfilePath (__REACTOS__).
unsafe fn _SHGetUserProfilePath(hToken: HANDLE, dwFlags: DWORD, folder: BYTE, pszPath: LPWSTR) -> HRESULT {
    let mut sz_value_name: LPCWSTR;
    let mut buffer = [0u16; 40];
    let mut hr: HRESULT = E_FAIL;

    if folder as DWORD >= ARRAY_SIZE_CSIDL() {
        return E_INVALIDARG;
    }
    let entry = csidl(folder as DWORD);
    if entry.typ != CSIDL_Type_User && entry.typ != CSIDL_Type_InMyDocuments {
        return E_INVALIDARG;
    }
    if pszPath.is_null() {
        return E_INVALIDARG;
    }

    if dwFlags & SHGFP_TYPE_DEFAULT != 0 {
        hr = _SHGetDefaultValue(hToken, folder, pszPath);
    } else {
        let mut user_prefix: LPCWSTR = null();
        let h_root_key: HKEY;

        if hToken as isize == -1 {
            h_root_key = HKEY_USERS;
            user_prefix = W_DOTDEFAULT.as_ptr();
        } else if hToken.is_null() {
            h_root_key = HKEY_CURRENT_USER;
        } else {
            h_root_key = HKEY_USERS;
            user_prefix = _GetUserSidStringFromToken(hToken);
            if user_prefix.is_null() {
                hr = E_FAIL;
                // goto error
                return hr;
            }
        }

        // For CSIDL_Type_User we also use the GUID if no szValueName is provided.
        sz_value_name = entry.szValueName;
        if sz_value_name.is_null() {
            StringFromGUID2(entry.id, buffer.as_mut_ptr(), 39);
            sz_value_name = buffer.as_ptr();
        }

        hr = _SHGetUserShellFolderPath(h_root_key, hToken, user_prefix, sz_value_name, pszPath);
        if FAILED(hr) && h_root_key != HKEY_LOCAL_MACHINE {
            hr = _SHGetUserShellFolderPath(HKEY_LOCAL_MACHINE, hToken, null(), sz_value_name, pszPath);
        }
        if FAILED(hr) {
            hr = _SHGetDefaultValue(hToken, folder, pszPath);
        }
        if !user_prefix.is_null() && user_prefix != W_DOTDEFAULT.as_ptr() {
            LocalFree(user_prefix as HANDLE);
        }
    }
    hr
}

// _SHGetAllUsersProfilePath (__REACTOS__).
unsafe fn _SHGetAllUsersProfilePath(dwFlags: DWORD, folder: BYTE, pszPath: LPWSTR) -> HRESULT {
    let mut hr: HRESULT;

    if folder as DWORD >= ARRAY_SIZE_CSIDL() {
        return E_INVALIDARG;
    }
    if csidl(folder as DWORD).typ != CSIDL_Type_AllUsers {
        return E_INVALIDARG;
    }
    if pszPath.is_null() {
        return E_INVALIDARG;
    }

    if dwFlags & SHGFP_TYPE_DEFAULT != 0 {
        hr = _SHGetDefaultValue(null_mut(), folder, pszPath);
    } else {
        hr = _SHGetUserShellFolderPath(
            HKEY_LOCAL_MACHINE,
            null_mut(),
            null(),
            csidl(folder as DWORD).szValueName,
            pszPath,
        );
        if FAILED(hr) {
            hr = _SHGetDefaultValue(null_mut(), folder, pszPath);
        }
    }
    hr
}

/// SHGetFolderPathW [SHELL32.@]
#[no_mangle]
pub unsafe extern "system" fn SHGetFolderPathW(
    hwndOwner: HWND,
    nFolder: INT,
    hToken: HANDLE,
    dwFlags: DWORD,
    pszPath: LPWSTR,
) -> HRESULT {
    let mut hr = SHGetFolderPathAndSubDirW(hwndOwner, nFolder, hToken, dwFlags, null(), pszPath);
    if HRESULT_FROM_WIN32(ERROR_PATH_NOT_FOUND) == hr {
        hr = HRESULT_FROM_WIN32(ERROR_FILE_NOT_FOUND);
    }
    hr
}

/// SHGetFolderPathAndSubDirA [SHELL32.@]
#[no_mangle]
pub unsafe extern "system" fn SHGetFolderPathAndSubDirA(
    hwndOwner: HWND,
    nFolder: INT,
    hToken: HANDLE,
    dwFlags: DWORD,
    pszSubPath: LPCSTR,
    pszPath: LPSTR,
) -> HRESULT {
    let length: INT;
    let mut hr: HRESULT = S_OK;
    let mut psz_sub_path_w: LPWSTR = null_mut();
    let mut psz_path_w: LPWSTR = null_mut();

    if !pszPath.is_null() {
        psz_path_w = HeapAlloc(GetProcessHeap(), 0, MAX_PATH * 2) as LPWSTR;
        if psz_path_w.is_null() {
            hr = HRESULT_FROM_WIN32(ERROR_NOT_ENOUGH_MEMORY);
            // cleanup
            HeapFree(GetProcessHeap(), 0, psz_path_w as LPVOID);
            HeapFree(GetProcessHeap(), 0, psz_sub_path_w as LPVOID);
            return hr;
        }
    }

    if !pszSubPath.is_null() && *pszSubPath != 0 {
        length = MultiByteToWideChar(CP_ACP, 0, pszSubPath, -1, null_mut(), 0);
        psz_sub_path_w = HeapAlloc(GetProcessHeap(), 0, (length as usize) * 2) as LPWSTR;
        if psz_sub_path_w.is_null() {
            hr = HRESULT_FROM_WIN32(ERROR_NOT_ENOUGH_MEMORY);
            HeapFree(GetProcessHeap(), 0, psz_path_w as LPVOID);
            HeapFree(GetProcessHeap(), 0, psz_sub_path_w as LPVOID);
            return hr;
        }
        MultiByteToWideChar(CP_ACP, 0, pszSubPath, -1, psz_sub_path_w, length);
    }

    hr = SHGetFolderPathAndSubDirW(hwndOwner, nFolder, hToken, dwFlags, psz_sub_path_w, psz_path_w);

    if SUCCEEDED(hr) && !pszPath.is_null() {
        WideCharToMultiByte(CP_ACP, 0, psz_path_w, -1, pszPath, MAX_PATH as INT, null(), null_mut());
    }

    HeapFree(GetProcessHeap(), 0, psz_path_w as LPVOID);
    HeapFree(GetProcessHeap(), 0, psz_sub_path_w as LPVOID);
    hr
}

/// SHGetFolderPathAndSubDirW [SHELL32.@]
#[no_mangle]
pub unsafe extern "system" fn SHGetFolderPathAndSubDirW(
    hwndOwner: HWND,
    nFolder: INT,
    hToken: HANDLE,
    dwFlags: DWORD,
    pszSubPath: LPCWSTR,
    pszPath: LPWSTR,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut sz_build_path = [0u16; MAX_PATH];
    let mut sz_temp = [0u16; MAX_PATH];
    let folder: DWORD = (nFolder & CSIDL_FOLDER_MASK) as DWORD;

    // Windows always NULL-terminates the result regardless of success/failure.
    if !pszPath.is_null() {
        write_volatile(pszPath, 0u16);
    }

    if folder >= ARRAY_SIZE_CSIDL() {
        return E_INVALIDARG;
    }
    if SHGFP_TYPE_CURRENT != dwFlags && SHGFP_TYPE_DEFAULT != dwFlags {
        return E_INVALIDARG;
    }
    sz_temp[0] = 0;
    let typ = csidl(folder).typ;
    match typ {
        CSIDL_Type_Disallowed => {
            hr = E_INVALIDARG;
        }
        CSIDL_Type_NonExistent => {
            hr = S_FALSE;
        }
        CSIDL_Type_WindowsPath => {
            GetWindowsDirectoryW(sz_temp.as_mut_ptr(), MAX_PATH as UINT);
            let dp = csidl(folder).szDefaultPath;
            if !dp.is_null() && !is_intresource(dp) && *dp != 0 {
                PathAddBackslashW(sz_temp.as_mut_ptr());
                strcatW(sz_temp.as_mut_ptr(), dp);
            }
            hr = S_OK;
        }
        CSIDL_Type_SystemPath => {
            GetSystemDirectoryW(sz_temp.as_mut_ptr(), MAX_PATH as UINT);
            let dp = csidl(folder).szDefaultPath;
            if !dp.is_null() && !is_intresource(dp) && *dp != 0 {
                PathAddBackslashW(sz_temp.as_mut_ptr());
                strcatW(sz_temp.as_mut_ptr(), dp);
            }
            hr = S_OK;
        }
        CSIDL_Type_SystemX86Path => {
            if GetSystemWow64DirectoryW(sz_temp.as_mut_ptr(), MAX_PATH as UINT) == 0 {
                GetSystemDirectoryW(sz_temp.as_mut_ptr(), MAX_PATH as UINT);
            }
            let dp = csidl(folder).szDefaultPath;
            if !dp.is_null() && !is_intresource(dp) && *dp != 0 {
                PathAddBackslashW(sz_temp.as_mut_ptr());
                strcatW(sz_temp.as_mut_ptr(), dp);
            }
            hr = S_OK;
        }
        CSIDL_Type_CurrVer => {
            hr = _SHGetCurrentVersionPath(dwFlags, folder as BYTE, sz_temp.as_mut_ptr());
        }
        CSIDL_Type_User | CSIDL_Type_InMyDocuments => {
            hr = _SHGetUserProfilePath(hToken, dwFlags, folder as BYTE, sz_temp.as_mut_ptr());
        }
        CSIDL_Type_AllUsers => {
            hr = _SHGetAllUsersProfilePath(dwFlags, folder as BYTE, sz_temp.as_mut_ptr());
        }
        _ => {
            hr = E_INVALIDARG;
        }
    }

    // Expand environment strings if necessary.
    if sz_temp[0] == 0x25 {
        hr = SHExpandEnvironmentStrings_(hToken, sz_temp.as_ptr(), sz_build_path.as_mut_ptr(), MAX_PATH as DWORD);
    } else {
        strcpyW(sz_build_path.as_mut_ptr(), sz_temp.as_ptr());
    }

    if FAILED(hr) {
        return shgfp_end(nFolder, folder, &sz_build_path, hr);
    }

    if !pszSubPath.is_null() {
        if MAX_PATH < (lstrlenW(sz_build_path.as_ptr()) as usize + lstrlenW(pszSubPath) as usize + 2) {
            hr = HRESULT_FROM_WIN32(ERROR_FILENAME_EXCED_RANGE);
            return shgfp_end(nFolder, folder, &sz_build_path, hr);
        }
        PathAppendW(sz_build_path.as_mut_ptr(), pszSubPath);
        PathRemoveBackslashW(sz_build_path.as_mut_ptr());
    }
    // Copy the path if it's available before we might return.
    if SUCCEEDED(hr) && !pszPath.is_null() {
        strcpyW(pszPath, sz_build_path.as_ptr());
    }

    if nFolder & CSIDL_FLAG_DONT_VERIFY != 0 {
        return shgfp_end(nFolder, folder, &sz_build_path, hr);
    }

    if PathFileExistsW(sz_build_path.as_ptr()) != FALSE {
        return shgfp_end(nFolder, folder, &sz_build_path, hr);
    }

    if nFolder & CSIDL_FLAG_CREATE == 0 {
        hr = HRESULT_FROM_WIN32(ERROR_PATH_NOT_FOUND);
        return shgfp_end(nFolder, folder, &sz_build_path, hr);
    }

    let ret = SHCreateDirectoryExW(hwndOwner, sz_build_path.as_ptr(), null_mut());
    if ret != 0 && ret != ERROR_ALREADY_EXISTS {
        hr = E_FAIL;
        return shgfp_end(nFolder, folder, &sz_build_path, hr);
    }

    shgfp_end(nFolder, folder, &sz_build_path, hr)
}

// `end:` tail of SHGetFolderPathAndSubDirW — the __REACTOS__ desktop.ini creation.
unsafe fn shgfp_end(nFolder: INT, folder: DWORD, sz_build_path: &[u16; MAX_PATH], hr: HRESULT) -> HRESULT {
    if nFolder & CSIDL_FLAG_CREATE != 0 && csidl(folder).nShell32IconIndex != 0 {
        let mut sz_path = [0u16; MAX_PATH];
        // local mutable copy so we can append desktop.ini without disturbing the caller's view
        strcpyW(sz_path.as_mut_ptr(), sz_build_path.as_ptr());

        // make the directory a read-only folder
        let mut dw_attributes = GetFileAttributesW(sz_path.as_ptr());
        dw_attributes |= FILE_ATTRIBUTE_READONLY;
        SetFileAttributesW(sz_path.as_ptr(), dw_attributes);

        // build the desktop.ini file path
        PathAppendW(sz_path.as_mut_ptr(), W_DESKTOP_INI.as_ptr());

        // build the icon location
        let mut sz_icon_location = [0u16; MAX_PATH];
        string_cch_printf(
            sz_icon_location.as_mut_ptr(),
            MAX_PATH,
            W_ICON_FMT.as_ptr(),
            &[FA::I(csidl(folder).nShell32IconIndex)],
        );

        // write desktop.ini
        WritePrivateProfileStringW(
            W_SHELLCLASSINFO.as_ptr(),
            W_ICONRESOURCE.as_ptr(),
            sz_icon_location.as_ptr(),
            sz_path.as_ptr(),
        );
        // flush!
        WritePrivateProfileStringW(null(), null(), null(), sz_path.as_ptr());

        // make the desktop.ini a system and hidden file
        let mut dw_attributes = GetFileAttributesW(sz_path.as_ptr());
        dw_attributes |= FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN;
        SetFileAttributesW(sz_path.as_ptr(), dw_attributes);
    }
    hr
}

/// SHGetFolderPathA [SHELL32.@]
#[no_mangle]
pub unsafe extern "system" fn SHGetFolderPathA(
    hwndOwner: HWND,
    nFolder: INT,
    hToken: HANDLE,
    dwFlags: DWORD,
    pszPath: LPSTR,
) -> HRESULT {
    let mut sz_temp = [0u16; MAX_PATH];

    if !pszPath.is_null() {
        write_volatile(pszPath, 0i8);
    }
    let hr = SHGetFolderPathW(hwndOwner, nFolder, hToken, dwFlags, sz_temp.as_mut_ptr());
    if SUCCEEDED(hr) && !pszPath.is_null() {
        WideCharToMultiByte(CP_ACP, 0, sz_temp.as_ptr(), -1, pszPath, MAX_PATH as INT, null(), null_mut());
    }
    hr
}

unsafe fn _SHRegisterFolders(
    hRootKey: HKEY,
    hToken: HANDLE,
    szUserShellFolderPath: LPCWSTR,
    szShellFolderPath: LPCWSTR,
    folders: &[UINT],
) -> HRESULT {
    let mut sz_value_name: LPCWSTR;
    let mut buffer = [0u16; 40];
    let mut path = [0u16; MAX_PATH];
    let mut hr: HRESULT = S_OK;
    let mut h_user_key: HKEY = null_mut();
    let mut h_key: HKEY = null_mut();
    let mut dw_type: DWORD = 0;
    let mut dw_path_len: DWORD;
    let mut ret: LONG;

    ret = RegCreateKeyW(hRootKey, szUserShellFolderPath, &mut h_user_key);
    if ret != 0 {
        hr = HRESULT_FROM_WIN32(ret);
    } else {
        ret = RegCreateKeyW(hRootKey, szShellFolderPath, &mut h_key);
        if ret != 0 {
            hr = HRESULT_FROM_WIN32(ret);
        }
    }
    let mut i = 0usize;
    while SUCCEEDED(hr) && i < folders.len() {
        dw_path_len = (MAX_PATH * 2) as DWORD;

        let entry = csidl(folders[i]);
        sz_value_name = entry.szValueName;
        if sz_value_name.is_null()
            && (entry.typ == CSIDL_Type_User || entry.typ == CSIDL_Type_InMyDocuments)
        {
            StringFromGUID2(entry.id, buffer.as_mut_ptr(), 39);
            sz_value_name = buffer.as_ptr();
        }

        if RegQueryValueExW(
            h_user_key,
            sz_value_name,
            null_mut(),
            &mut dw_type,
            path.as_mut_ptr() as LPBYTE,
            &mut dw_path_len,
        ) == 0
            && (dw_type == REG_SZ || dw_type == REG_EXPAND_SZ)
        {
            hr = SHGetFolderPathW(
                null_mut(),
                (folders[i] | CSIDL_FLAG_CREATE as UINT) as INT,
                hToken,
                SHGFP_TYPE_CURRENT,
                path.as_mut_ptr(),
            );
        } else {
            path[0] = 0;
            if entry.typ == CSIDL_Type_User || entry.typ == CSIDL_Type_InMyDocuments {
                _SHGetUserProfilePath(hToken, SHGFP_TYPE_CURRENT, folders[i] as BYTE, path.as_mut_ptr());
            } else if entry.typ == CSIDL_Type_AllUsers {
                _SHGetAllUsersProfilePath(SHGFP_TYPE_CURRENT, folders[i] as BYTE, path.as_mut_ptr());
            } else if entry.typ == CSIDL_Type_WindowsPath {
                GetWindowsDirectoryW(path.as_mut_ptr(), MAX_PATH as UINT);
                if !entry.szDefaultPath.is_null() && !is_intresource(entry.szDefaultPath) {
                    PathAddBackslashW(path.as_mut_ptr());
                    strcatW(path.as_mut_ptr(), entry.szDefaultPath);
                }
            } else {
                hr = E_FAIL;
            }
            if path[0] != 0 {
                ret = RegSetValueExW(
                    h_user_key,
                    sz_value_name,
                    0,
                    REG_EXPAND_SZ,
                    path.as_ptr() as *const BYTE,
                    ((r_wcslen(path.as_ptr()) + 1) * 2) as DWORD,
                );
                if ret != 0 {
                    hr = HRESULT_FROM_WIN32(ret);
                } else {
                    hr = SHGetFolderPathW(
                        null_mut(),
                        (folders[i] | CSIDL_FLAG_CREATE as UINT) as INT,
                        hToken,
                        SHGFP_TYPE_CURRENT,
                        path.as_mut_ptr(),
                    );
                    ret = RegSetValueExW(
                        h_key,
                        sz_value_name,
                        0,
                        REG_SZ,
                        path.as_ptr() as *const BYTE,
                        ((r_wcslen(path.as_ptr()) + 1) * 2) as DWORD,
                    );
                    if ret != 0 {
                        hr = HRESULT_FROM_WIN32(ret);
                    }
                }
            }
        }
        i += 1;
    }
    if !h_user_key.is_null() {
        RegCloseKey(h_user_key);
    }
    if !h_key.is_null() {
        RegCloseKey(h_key);
    }
    hr
}

unsafe fn _SHRegisterUserShellFolders(bDefault: BOOL) -> HRESULT {
    static FOLDERS: [UINT; 21] = [
        CSIDL_PROGRAMS,
        CSIDL_PERSONAL,
        CSIDL_FAVORITES,
        CSIDL_APPDATA,
        CSIDL_STARTUP,
        CSIDL_RECENT,
        CSIDL_SENDTO,
        CSIDL_STARTMENU,
        CSIDL_MYMUSIC,
        CSIDL_MYVIDEO,
        CSIDL_DESKTOPDIRECTORY,
        CSIDL_NETHOOD,
        CSIDL_TEMPLATES,
        CSIDL_PRINTHOOD,
        CSIDL_LOCAL_APPDATA,
        CSIDL_INTERNET_CACHE,
        CSIDL_COOKIES,
        CSIDL_HISTORY,
        CSIDL_MYPICTURES,
        CSIDL_FONTS,
        CSIDL_ADMINTOOLS,
    ];
    let mut user_shell_folder_path = [0u16; MAX_PATH];
    let mut shell_folder_path = [0u16; MAX_PATH];
    let p_user_shell_folder_path: LPCWSTR;
    let p_shell_folder_path: LPCWSTR;
    let hr: HRESULT;
    let h_root_key: HKEY;
    let h_token: HANDLE;

    if bDefault != FALSE {
        h_token = -1isize as HANDLE;
        h_root_key = HKEY_USERS;
        strcpyW(user_shell_folder_path.as_mut_ptr(), W_DOTDEFAULT.as_ptr());
        PathAddBackslashW(user_shell_folder_path.as_mut_ptr());
        strcatW(user_shell_folder_path.as_mut_ptr(), W_SHUSERFOLDERS.as_ptr());
        p_user_shell_folder_path = user_shell_folder_path.as_ptr();
        strcpyW(shell_folder_path.as_mut_ptr(), W_DOTDEFAULT.as_ptr());
        PathAddBackslashW(shell_folder_path.as_mut_ptr());
        strcatW(shell_folder_path.as_mut_ptr(), W_SHFOLDERS.as_ptr());
        p_shell_folder_path = shell_folder_path.as_ptr();
    } else {
        h_token = null_mut();
        h_root_key = HKEY_CURRENT_USER;
        p_user_shell_folder_path = W_SHUSERFOLDERS.as_ptr();
        p_shell_folder_path = W_SHFOLDERS.as_ptr();
    }

    hr = _SHRegisterFolders(h_root_key, h_token, p_user_shell_folder_path, p_shell_folder_path, &FOLDERS);
    hr
}

unsafe fn _SHRegisterCommonShellFolders() -> HRESULT {
    static FOLDERS: [UINT; 12] = [
        CSIDL_COMMON_STARTMENU,
        CSIDL_COMMON_PROGRAMS,
        CSIDL_COMMON_STARTUP,
        CSIDL_COMMON_DESKTOPDIRECTORY,
        CSIDL_COMMON_FAVORITES,
        CSIDL_COMMON_APPDATA,
        CSIDL_COMMON_TEMPLATES,
        CSIDL_COMMON_DOCUMENTS,
        CSIDL_COMMON_ADMINTOOLS,
        CSIDL_COMMON_MUSIC,
        CSIDL_COMMON_PICTURES,
        CSIDL_COMMON_VIDEO,
    ];
    _SHRegisterFolders(HKEY_LOCAL_MACHINE, null_mut(), W_SHUSERFOLDERS.as_ptr(), W_SHFOLDERS.as_ptr(), &FOLDERS)
}

/// SHELL_RegisterShellFolders — EXPORTED (cdecl). Register XP-compatible default values.
#[no_mangle]
pub unsafe extern "C" fn SHELL_RegisterShellFolders() -> HRESULT {
    let mut hr = _SHRegisterUserShellFolders(TRUE);
    if SUCCEEDED(hr) {
        hr = _SHRegisterUserShellFolders(FALSE);
    }
    if SUCCEEDED(hr) {
        hr = _SHRegisterCommonShellFolders();
    }
    hr
}

/// SHGetSpecialFolderPathA [SHELL32.@]
#[no_mangle]
pub unsafe extern "system" fn SHGetSpecialFolderPathA(
    hwndOwner: HWND,
    szPath: LPSTR,
    nFolder: INT,
    bCreate: BOOL,
) -> BOOL {
    (SHGetFolderPathA(
        hwndOwner,
        nFolder + if bCreate != FALSE { CSIDL_FLAG_CREATE } else { 0 },
        null_mut(),
        0,
        szPath,
    ) == S_OK) as BOOL
}

/// SHGetSpecialFolderPathW [SHELL32.@]
#[no_mangle]
pub unsafe extern "system" fn SHGetSpecialFolderPathW(
    hwndOwner: HWND,
    szPath: LPWSTR,
    nFolder: INT,
    bCreate: BOOL,
) -> BOOL {
    (SHGetFolderPathW(
        hwndOwner,
        nFolder + if bCreate != FALSE { CSIDL_FLAG_CREATE } else { 0 },
        null_mut(),
        0,
        szPath,
    ) == S_OK) as BOOL
}

/// SHGetFolderLocationHelper — EXPORTED (cdecl, __REACTOS__).
#[no_mangle]
pub unsafe extern "C" fn SHGetFolderLocationHelper(
    hwnd: HWND,
    nFolder: INT,
    clsid: REFCLSID,
    ppidl: *mut LPITEMIDLIST,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut psf: *mut IShellFolder = null_mut();
    let mut parent: LPITEMIDLIST = null_mut();
    let mut child: LPITEMIDLIST = null_mut();

    write_volatile(ppidl, null_mut());
    hr = SHGetFolderLocation(hwnd, nFolder, null_mut(), 0, &mut parent);
    if FAILED(hr) {
        return hr;
    }
    hr = SHBindToObject(
        null_mut(),
        parent,
        addr_of!(IID_IShellFolder),
        &mut psf as *mut *mut IShellFolder as *mut *mut c_void,
    );
    if SUCCEEDED(hr) {
        let mut clsidstr = [0u16; 2 + 38 + 1];
        clsidstr[0] = 0x3A; // ':'
        clsidstr[1] = 0x3A; // ':'
        StringFromGUID2(clsid, clsidstr.as_mut_ptr().add(2), 38 + 1);
        hr = ((*(*psf).lpVtbl).ParseDisplayName)(
            psf,
            hwnd,
            null_mut(),
            clsidstr.as_mut_ptr(),
            null_mut(),
            &mut child,
            null_mut(),
        );
        if SUCCEEDED(hr) {
            write_volatile(ppidl, ILCombine(parent, child));
        }
        ((*(*psf).lpVtbl).Release)(psf);
        ILFree(child);
    }
    ILFree(parent);
    hr
}

/// SHGetFolderLocation [SHELL32.@]
#[no_mangle]
pub unsafe extern "system" fn SHGetFolderLocation(
    hwndOwner: HWND,
    nFolder: INT,
    hToken: HANDLE,
    dwReserved: DWORD,
    ppidl: *mut LPITEMIDLIST,
) -> HRESULT {
    let mut hr: HRESULT = E_INVALIDARG;
    let mut sz_path = [0u16; MAX_PATH];

    if ppidl.is_null() {
        return E_INVALIDARG;
    }
    if dwReserved != 0 {
        return E_INVALIDARG;
    }

    if nFolder & CSIDL_FLAG_NO_ALIAS != 0
        && SHGetSpecialFolderPathW(hwndOwner, sz_path.as_mut_ptr(), nFolder & CSIDL_FOLDER_MASK, FALSE) != FALSE
    {
        let pidl = ILCreateFromPathW(sz_path.as_ptr());
        write_volatile(ppidl, pidl);
        if !pidl.is_null() {
            return S_OK;
        }
    }

    // The virtual folders' locations are not user-dependent.
    write_volatile(ppidl, null_mut());
    let f = nFolder & CSIDL_FOLDER_MASK;
    if f == CSIDL_DESKTOP {
        write_volatile(ppidl, _ILCreateDesktop());
    } else if f == CSIDL_PERSONAL as INT {
        write_volatile(ppidl, _ILCreateMyDocuments());
    } else if f == CSIDL_INTERNET {
        write_volatile(ppidl, _ILCreateIExplore());
    } else if f == CSIDL_CONTROLS {
        write_volatile(ppidl, _ILCreateControlPanel());
    } else if f == CSIDL_PRINTERS {
        write_volatile(ppidl, _ILCreatePrinters());
    } else if f == CSIDL_BITBUCKET {
        write_volatile(ppidl, _ILCreateBitBucket());
    } else if f == CSIDL_DRIVES {
        write_volatile(ppidl, _ILCreateMyComputer());
    } else if f == CSIDL_NETWORK {
        write_volatile(ppidl, _ILCreateNetwork());
    } else if f == CSIDL_CONNECTIONS {
        hr = SHGetFolderLocationHelper(hwndOwner, CSIDL_CONTROLS, addr_of!(CLSID_NetworkConnections), ppidl);
    } else {
        let mut sz_path2 = [0u16; MAX_PATH];
        hr = SHGetFolderPathW(hwndOwner, nFolder, hToken, SHGFP_TYPE_CURRENT, sz_path2.as_mut_ptr());
        if SUCCEEDED(hr) {
            let mut attributes: DWORD = 0;
            hr = SHILCreateFromPathW(sz_path2.as_ptr(), ppidl, &mut attributes);
        } else if hr == HRESULT_FROM_WIN32(ERROR_FILE_NOT_FOUND) {
            hr = E_FAIL;
        }
    }
    if !(*ppidl).is_null() {
        hr = S_OK;
    }

    hr
}

/// SHGetSpecialFolderLocation [SHELL32.@]
#[no_mangle]
pub unsafe extern "system" fn SHGetSpecialFolderLocation(
    hwndOwner: HWND,
    nFolder: INT,
    ppidl: *mut LPITEMIDLIST,
) -> HRESULT {
    if ppidl.is_null() {
        return E_INVALIDARG;
    }
    SHGetFolderLocation(hwndOwner, nFolder, null_mut(), 0, ppidl)
}

// ADDED-TO-TYPES?: none needed — crate::shellcom already provided every shared type this
// file uses (ITEMIDLIST/LPITEMIDLIST, GUID/IID/CLSID/REFCLSID, the IShellFolder vtable with
// ParseDisplayName+Release, STRRET, the scalar aliases). The only file-local additions are
// the CSIDL_Data machinery (CsidlData struct + CsidlType discriminants + the 63-entry table
// and its wide-string statics), which are shellpath-specific and intentionally not shared.
// Data imports declared here (resolve at link): FOLDERID_* / GUID_NULL / CLSID_NetworkConnections
// / IID_IShellFolder / shell32_hInstance.
