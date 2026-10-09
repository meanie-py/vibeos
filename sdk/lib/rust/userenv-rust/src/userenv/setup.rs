use crate::prelude::*;
use core::ffi::c_void;


extern "system" {
    // ARRAYSIZE removed: a MACRO, not a linkable fn (no live call sites in this module).
    fn CreateDirectoryW(lpPathName: LPCWSTR, lpSecurityAttributes: *mut core::ffi::c_void) -> BOOL;
    fn ExpandEnvironmentStringsW(lpSrc: LPCWSTR, lpDst: *mut WCHAR, cchDst: DWORD) -> DWORD;
    fn GetLastError() -> DWORD;
    fn LoadStringW(hInstance: *mut core::ffi::c_void, uID: UINT, lpBuffer: *mut WCHAR, cchBufferMax: core::ffi::c_int) -> core::ffi::c_int;
    fn RegCloseKey(hKey: HKEY) -> LONG;
    fn RegOpenKeyExW(hKey: HKEY, lpSubKey: LPCWSTR, ulOptions: DWORD, samDesired: DWORD, phkResult: *mut HKEY) -> LONG;
    fn RegSetValueExW(hKey: HKEY, lpValueName: LPCWSTR, Reserved: DWORD, dwType: DWORD, lpData: *const BYTE, cbData: DWORD) -> LONG;
    fn SetLastError(dwErrCode: DWORD);
    // hInstance removed: it is the DLL's HINSTANCE data global (userenv.c), NOT a function —
    // the c2rust output emitted `hInstance()` calls; those now read the static (see the call site).
}

// L"\"
static PATH_SEP_BACKSLASH: [u16; 2] = [b'\\' as u16, 0];
// L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList"
static PROFILE_LIST_KEY: [u16; 57] = [
    b'S' as u16, b'O' as u16, b'F' as u16, b'T' as u16, b'W' as u16, b'A' as u16, b'R' as u16,
    b'E' as u16, b'\\' as u16, b'M' as u16, b'i' as u16, b'c' as u16, b'r' as u16, b'o' as u16,
    b's' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'\\' as u16, b'W' as u16, b'i' as u16,
    b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b's' as u16, b' ' as u16, b'N' as u16,
    b'T' as u16, b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16,
    b'n' as u16, b't' as u16, b'V' as u16, b'e' as u16, b'r' as u16, b's' as u16, b'i' as u16,
    b'o' as u16, b'n' as u16, b'\\' as u16, b'P' as u16, b'r' as u16, b'o' as u16, b'f' as u16,
    b'i' as u16, b'l' as u16, b'e' as u16, b'L' as u16, b'i' as u16, b's' as u16, b't' as u16,
    0,
];
// L"ProfilesDirectory"
static REG_PROFILES_DIRECTORY: [u16; 18] = [
    b'P' as u16, b'r' as u16, b'o' as u16, b'f' as u16, b'i' as u16, b'l' as u16, b'e' as u16,
    b's' as u16, b'D' as u16, b'i' as u16, b'r' as u16, b'e' as u16, b'c' as u16, b't' as u16,
    b'o' as u16, b'r' as u16, b'y' as u16, 0,
];

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut UserShellFolders: [FOLDERDATA; 19] = unsafe {
    [
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"A\0p\0p\0D\0a\0t\0a\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"A\0p\0p\0l\0i\0c\0a\0t\0i\0o\0n\0 \0D\0a\0t\0a\0\0\0") as LPWSTR,
            uId: IDS_APPDATA as UINT,
            bHidden: TRUE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"D\0e\0s\0k\0t\0o\0p\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"D\0e\0s\0k\0t\0o\0p\0\0\0") as LPWSTR,
            uId: IDS_DESKTOP as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"F\0a\0v\0o\0r\0i\0t\0e\0s\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"F\0a\0v\0o\0r\0i\0t\0e\0s\0\0\0") as LPWSTR,
            uId: IDS_FAVORITES as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"P\0e\0r\0s\0o\0n\0a\0l\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"M\0y\0 \0D\0o\0c\0u\0m\0e\0n\0t\0s\0\0\0") as LPWSTR,
            uId: IDS_MYDOCUMENTS as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"N\0e\0t\0H\0o\0o\0d\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"N\0e\0t\0H\0o\0o\0d\0\0\0") as LPWSTR,
            uId: IDS_NETHOOD as UINT,
            bHidden: TRUE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"P\0r\0i\0n\0t\0H\0o\0o\0d\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"P\0r\0i\0n\0t\0H\0o\0o\0d\0\0\0") as LPWSTR,
            uId: IDS_PRINTHOOD as UINT,
            bHidden: TRUE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"R\0e\0c\0e\0n\0t\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"R\0e\0c\0e\0n\0t\0\0\0") as LPWSTR,
            uId: IDS_RECENT as UINT,
            bHidden: TRUE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"S\0e\0n\0d\0T\0o\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"S\0e\0n\0d\0T\0o\0\0\0") as LPWSTR,
            uId: IDS_SENDTO as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"T\0e\0m\0p\0l\0a\0t\0e\0s\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"T\0e\0m\0p\0l\0a\0t\0e\0s\0\0\0") as LPWSTR,
            uId: IDS_TEMPLATES as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"S\0t\0a\0r\0t\0 \0M\0e\0n\0u\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"S\0t\0a\0r\0t\0 \0M\0e\0n\0u\0\0\0") as LPWSTR,
            uId: IDS_STARTMENU as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"P\0r\0o\0g\0r\0a\0m\0s\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"S\0t\0a\0r\0t\0 \0M\0e\0n\0u\0\\\0P\0r\0o\0g\0r\0a\0m\0s\0\0\0") as LPWSTR,
            uId: IDS_PROGRAMS as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"S\0t\0a\0r\0t\0u\0p\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"S\0t\0a\0r\0t\0 \0M\0e\0n\0u\0\\\0P\0r\0o\0g\0r\0a\0m\0s\0\\\0S\0t\0a\0r\0t\0u\0p\0\0\0") as LPWSTR,
            uId: IDS_STARTUP as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"L\0o\0c\0a\0l\0 \0S\0e\0t\0t\0i\0n\0g\0s\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"L\0o\0c\0a\0l\0 \0S\0e\0t\0t\0i\0n\0g\0s\0\0\0") as LPWSTR,
            uId: IDS_LOCALSETTINGS as UINT,
            bHidden: TRUE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"L\0o\0c\0a\0l\0 \0A\0p\0p\0D\0a\0t\0a\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"L\0o\0c\0a\0l\0 \0S\0e\0t\0t\0i\0n\0g\0s\0\\\0A\0p\0p\0l\0i\0c\0a\0t\0i\0o\0n\0 \0D\0a\0t\0a\0\0\0") as LPWSTR,
            uId: IDS_LOCALAPPDATA as UINT,
            bHidden: TRUE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"T\0e\0m\0p\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"L\0o\0c\0a\0l\0 \0S\0e\0t\0t\0i\0n\0g\0s\0\\\0T\0e\0m\0p\0\0\0") as LPWSTR,
            uId: IDS_TEMP as UINT,
            bHidden: FALSE,
            bShellFolder: FALSE,
            bUserShellFolder: FALSE,
        },
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"C\0a\0c\0h\0e\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"L\0o\0c\0a\0l\0 \0S\0e\0t\0t\0i\0n\0g\0s\0\\\0T\0e\0m\0p\0o\0r\0a\0r\0y\0 \0I\0n\0t\0e\0r\0n\0e\0t\0 \0F\0i\0l\0e\0s\0\0\0") as LPWSTR,
            uId: IDS_CACHE as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"H\0i\0s\0t\0o\0r\0y\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"L\0o\0c\0a\0l\0 \0S\0e\0t\0t\0i\0n\0g\0s\0\\\0H\0i\0s\0t\0o\0r\0y\0\0\0") as LPWSTR,
            uId: IDS_HISTORY as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"C\0o\0o\0k\0i\0e\0s\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"C\0o\0o\0k\0i\0e\0s\0\0\0") as LPWSTR,
            uId: IDS_COOKIES as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            lpPath: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            uId: -(1 as ::core::ffi::c_int) as UINT,
            bHidden: FALSE,
            bShellFolder: FALSE,
            bUserShellFolder: FALSE,
        },
    ]
};

static mut CommonShellFolders: [FOLDERDATA; 9] = unsafe {
    [
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"C\0o\0m\0m\0o\0n\0 \0A\0p\0p\0D\0a\0t\0a\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"A\0p\0p\0l\0i\0c\0a\0t\0i\0o\0n\0 \0D\0a\0t\0a\0\0\0") as LPWSTR,
            uId: IDS_APPDATA as UINT,
            bHidden: TRUE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"C\0o\0m\0m\0o\0n\0 \0D\0e\0s\0k\0t\0o\0p\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"D\0e\0s\0k\0t\0o\0p\0\0\0") as LPWSTR,
            uId: IDS_DESKTOP as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"C\0o\0m\0m\0o\0n\0 \0F\0a\0v\0o\0r\0i\0t\0e\0s\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"F\0a\0v\0o\0r\0i\0t\0e\0s\0\0\0") as LPWSTR,
            uId: IDS_FAVORITES as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"C\0o\0m\0m\0o\0n\0 \0D\0o\0c\0u\0m\0e\0n\0t\0s\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"M\0y\0 \0D\0o\0c\0u\0m\0e\0n\0t\0s\0\0\0") as LPWSTR,
            uId: IDS_MYDOCUMENTS as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"C\0o\0m\0m\0o\0n\0 \0T\0e\0m\0p\0l\0a\0t\0e\0s\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"T\0e\0m\0p\0l\0a\0t\0e\0s\0\0\0") as LPWSTR,
            uId: IDS_TEMPLATES as UINT,
            bHidden: TRUE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"C\0o\0m\0m\0o\0n\0 \0S\0t\0a\0r\0t\0 \0M\0e\0n\0u\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"S\0t\0a\0r\0t\0 \0M\0e\0n\0u\0\0\0") as LPWSTR,
            uId: IDS_STARTMENU as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"C\0o\0m\0m\0o\0n\0 \0P\0r\0o\0g\0r\0a\0m\0s\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"S\0t\0a\0r\0t\0 \0M\0e\0n\0u\0\\\0P\0r\0o\0g\0r\0a\0m\0s\0\0\0") as LPWSTR,
            uId: IDS_PROGRAMS as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: win32macros_rust::wide_bytes!(b"C\0o\0m\0m\0o\0n\0 \0S\0t\0a\0r\0t\0u\0p\0\0\0") as LPWSTR,
            lpPath: win32macros_rust::wide_bytes!(b"S\0t\0a\0r\0t\0 \0M\0e\0n\0u\0\\\0P\0r\0o\0g\0r\0a\0m\0s\0\\\0S\0t\0a\0r\0t\0u\0p\0\0\0") as LPWSTR,
            uId: IDS_STARTUP as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            lpPath: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            uId: -(1 as ::core::ffi::c_int) as UINT,
            bHidden: FALSE,
            bShellFolder: FALSE,
            bUserShellFolder: FALSE,
        },
    ]
};

// The C has ONE StandardProfiles table (setup.c:81). This crate shipped TWO: this one, and a
// `standardProfiles` differing only in case, built from the aligned statics below but carrying
// hRootKey `transmute(-2147483646)` / `transmute(-2147483648)` -- HKEY_LOCAL_MACHINE and
// HKEY_CLASSES_ROOT, where the C says HKEY_USERS and HKEY_LOCAL_MACHINE. It was dead (nothing
// referenced it) so nothing misbehaved, but a wrong copy of the registry-root table is a trap
// for the next reader, and `cargo check` cannot see either problem. Collapsed to one table:
// the correct roots, over the `[u16; N]` statics rather than alignment-1 byte strings.
static mut StandardProfiles: [PROFILEPARAMS; 2] = unsafe {
    [
        _PROFILEPARAMS {
            pszProfileName: DEFAULT_USER_NAME.as_ptr(),
            pszProfileRegValue: DEFAULT_USER_PROFILE_VALUE.as_ptr(),
            pszEnvVar: ENV_USERPROFILE2.as_ptr(),
            pszEnvVarProfilePath: ENV_USERPROFILE_PCT.as_ptr(),
            pFolderList: &raw const UserShellFolders as PFOLDERDATA,
            hRootKey: HKEY_USERS,
            pszShellFoldersKey: DEFAULT_SHELL_FOLDERS_KEY.as_ptr(),
            pszUserShellFoldersKey: DEFAULT_USER_SHELL_FOLDERS_KEY.as_ptr(),
        },
        _PROFILEPARAMS {
            pszProfileName: ALL_USERS_NAME.as_ptr(),
            pszProfileRegValue: ALL_USERS_PROFILE_VALUE.as_ptr(),
            pszEnvVar: ENV_ALLUSERSPROFILE2.as_ptr(),
            pszEnvVarProfilePath: ENV_ALLUSERSPROFILE_PCT.as_ptr(),
            pFolderList: &raw const CommonShellFolders as PFOLDERDATA,
            hRootKey: HKEY_LOCAL_MACHINE,
            pszShellFoldersKey: SHELL_FOLDERS_KEY.as_ptr(),
            pszUserShellFoldersKey: USER_SHELL_FOLDERS_KEY2.as_ptr(),
        },
    ]
};

pub type FOLDERDATA = _FOLDERDATA;
pub type PROFILEPARAMS = _PROFILEPARAMS;

#[repr(C)]
pub struct _FOLDERDATA {
    pub lpValueName: LPWSTR,
    pub lpPath: LPWSTR,
    pub uId: u32,
    pub bHidden: i32,
    pub bShellFolder: i32,
    pub bUserShellFolder: i32,
}
pub type PFOLDERDATA = *mut _FOLDERDATA;

#[repr(C)]
pub struct _PROFILEPARAMS {
    pub pszProfileName: LPCWSTR,
    pub pszProfileRegValue: LPCWSTR,
    pub pszEnvVar: LPCWSTR,
    pub pszEnvVarProfilePath: LPCWSTR,
    pub pFolderList: PFOLDERDATA,
    pub hRootKey: HKEY,
    pub pszShellFoldersKey: LPCWSTR,
    pub pszUserShellFoldersKey: LPCWSTR,
}
pub type PPROFILEPARAMS = *mut _PROFILEPARAMS;

// L"Default User"
static DEFAULT_USER_NAME: [u16; 13] = [
    b'D' as u16, b'e' as u16, b'f' as u16, b'a' as u16, b'u' as u16, b'l' as u16, b't' as u16,
    b' ' as u16, b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, 0,
];
// L"DefaultUserProfile"
static DEFAULT_USER_PROFILE_VALUE: [u16; 19] = [
    b'D' as u16, b'e' as u16, b'f' as u16, b'a' as u16, b'u' as u16, b'l' as u16, b't' as u16,
    b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'P' as u16, b'r' as u16, b'o' as u16,
    b'f' as u16, b'i' as u16, b'l' as u16, b'e' as u16, 0,
];
// L"USERPROFILE"
static ENV_USERPROFILE2: [u16; 12] = [
    b'U' as u16, b'S' as u16, b'E' as u16, b'R' as u16, b'P' as u16, b'R' as u16, b'O' as u16,
    b'F' as u16, b'I' as u16, b'L' as u16, b'E' as u16, 0,
];
// L"%USERPROFILE%"
static ENV_USERPROFILE_PCT: [u16; 14] = [
    b'%' as u16, b'U' as u16, b'S' as u16, b'E' as u16, b'R' as u16, b'P' as u16, b'R' as u16,
    b'O' as u16, b'F' as u16, b'I' as u16, b'L' as u16, b'E' as u16, b'%' as u16, 0,
];
// L".Default\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Shell Folders"
static DEFAULT_SHELL_FOLDERS_KEY: [u16; 74] = [
    b'.' as u16, b'D' as u16, b'e' as u16, b'f' as u16, b'a' as u16, b'u' as u16, b'l' as u16,
    b't' as u16, b'\\' as u16, b'S' as u16, b'O' as u16, b'F' as u16, b'T' as u16, b'W' as u16,
    b'A' as u16, b'R' as u16, b'E' as u16, b'\\' as u16, b'M' as u16, b'i' as u16, b'c' as u16,
    b'r' as u16, b'o' as u16, b's' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'\\' as u16,
    b'W' as u16, b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b's' as u16,
    b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16, b'n' as u16,
    b't' as u16, b'V' as u16, b'e' as u16, b'r' as u16, b's' as u16, b'i' as u16, b'o' as u16,
    b'n' as u16, b'\\' as u16, b'E' as u16, b'x' as u16, b'p' as u16, b'l' as u16, b'o' as u16,
    b'r' as u16, b'e' as u16, b'r' as u16, b'\\' as u16, b'S' as u16, b'h' as u16, b'e' as u16,
    b'l' as u16, b'l' as u16, b' ' as u16, b'F' as u16, b'o' as u16, b'l' as u16, b'd' as u16,
    b'e' as u16, b'r' as u16, b's' as u16, 0,
];
// L".Default\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders"
static DEFAULT_USER_SHELL_FOLDERS_KEY: [u16; 79] = [
    b'.' as u16, b'D' as u16, b'e' as u16, b'f' as u16, b'a' as u16, b'u' as u16, b'l' as u16,
    b't' as u16, b'\\' as u16, b'S' as u16, b'O' as u16, b'F' as u16, b'T' as u16, b'W' as u16,
    b'A' as u16, b'R' as u16, b'E' as u16, b'\\' as u16, b'M' as u16, b'i' as u16, b'c' as u16,
    b'r' as u16, b'o' as u16, b's' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'\\' as u16,
    b'W' as u16, b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b's' as u16,
    b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16, b'n' as u16,
    b't' as u16, b'V' as u16, b'e' as u16, b'r' as u16, b's' as u16, b'i' as u16, b'o' as u16,
    b'n' as u16, b'\\' as u16, b'E' as u16, b'x' as u16, b'p' as u16, b'l' as u16, b'o' as u16,
    b'r' as u16, b'e' as u16, b'r' as u16, b'\\' as u16, b'U' as u16, b's' as u16, b'e' as u16,
    b'r' as u16, b' ' as u16, b'S' as u16, b'h' as u16, b'e' as u16, b'l' as u16, b'l' as u16,
    b' ' as u16, b'F' as u16, b'o' as u16, b'l' as u16, b'd' as u16, b'e' as u16, b'r' as u16,
    b's' as u16, 0,
];
// L"All Users"
static ALL_USERS_NAME: [u16; 10] = [
    b'A' as u16, b'l' as u16, b'l' as u16, b' ' as u16, b'U' as u16, b's' as u16, b'e' as u16,
    b'r' as u16, b's' as u16, 0,
];
// L"AllUsersProfile"
static ALL_USERS_PROFILE_VALUE: [u16; 16] = [
    b'A' as u16, b'l' as u16, b'l' as u16, b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16,
    b's' as u16, b'P' as u16, b'r' as u16, b'o' as u16, b'f' as u16, b'i' as u16, b'l' as u16,
    b'e' as u16, 0,
];
// L"ALLUSERSPROFILE"
static ENV_ALLUSERSPROFILE2: [u16; 16] = [
    b'A' as u16, b'L' as u16, b'L' as u16, b'U' as u16, b'S' as u16, b'E' as u16, b'R' as u16,
    b'S' as u16, b'P' as u16, b'R' as u16, b'O' as u16, b'F' as u16, b'I' as u16, b'L' as u16,
    b'E' as u16, 0,
];
// L"%ALLUSERSPROFILE%"
static ENV_ALLUSERSPROFILE_PCT: [u16; 18] = [
    b'%' as u16, b'A' as u16, b'L' as u16, b'L' as u16, b'U' as u16, b'S' as u16, b'E' as u16,
    b'R' as u16, b'S' as u16, b'P' as u16, b'R' as u16, b'O' as u16, b'F' as u16, b'I' as u16,
    b'L' as u16, b'E' as u16, b'%' as u16, 0,
];
// L"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Shell Folders"
static SHELL_FOLDERS_KEY: [u16; 65] = [
    b'S' as u16, b'O' as u16, b'F' as u16, b'T' as u16, b'W' as u16, b'A' as u16, b'R' as u16,
    b'E' as u16, b'\\' as u16, b'M' as u16, b'i' as u16, b'c' as u16, b'r' as u16, b'o' as u16,
    b's' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'\\' as u16, b'W' as u16, b'i' as u16,
    b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b's' as u16, b'\\' as u16, b'C' as u16,
    b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16, b'V' as u16,
    b'e' as u16, b'r' as u16, b's' as u16, b'i' as u16, b'o' as u16, b'n' as u16, b'\\' as u16,
    b'E' as u16, b'x' as u16, b'p' as u16, b'l' as u16, b'o' as u16, b'r' as u16, b'e' as u16,
    b'r' as u16, b'\\' as u16, b'S' as u16, b'h' as u16, b'e' as u16, b'l' as u16, b'l' as u16,
    b' ' as u16, b'F' as u16, b'o' as u16, b'l' as u16, b'd' as u16, b'e' as u16, b'r' as u16,
    b's' as u16, 0,
];
// L"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders"
static USER_SHELL_FOLDERS_KEY2: [u16; 70] = [
    b'S' as u16, b'O' as u16, b'F' as u16, b'T' as u16, b'W' as u16, b'A' as u16, b'R' as u16,
    b'E' as u16, b'\\' as u16, b'M' as u16, b'i' as u16, b'c' as u16, b'r' as u16, b'o' as u16,
    b's' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'\\' as u16, b'W' as u16, b'i' as u16,
    b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b's' as u16, b'\\' as u16, b'C' as u16,
    b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16, b'V' as u16,
    b'e' as u16, b'r' as u16, b's' as u16, b'i' as u16, b'o' as u16, b'n' as u16, b'\\' as u16,
    b'E' as u16, b'x' as u16, b'p' as u16, b'l' as u16, b'o' as u16, b'r' as u16, b'e' as u16,
    b'r' as u16, b'\\' as u16, b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b' ' as u16,
    b'S' as u16, b'h' as u16, b'e' as u16, b'l' as u16, b'l' as u16, b' ' as u16, b'F' as u16,
    b'o' as u16, b'l' as u16, b'd' as u16, b'e' as u16, b'r' as u16, b's' as u16, 0,
];


// IDS_PROFILEPATH comes from the prelude (resources.h: 1). A local copy here said 0, a
// resource that does not exist: LoadStringW failed and GUI setup stopped at
// "InitializeProfiles() failed" on every install.
unsafe extern "C" fn CreateStandardProfile(
    pszProfilesPath: LPCWSTR,
    hProfileListKey: HKEY,
    pProfileParams: PPROFILEPARAMS,
) -> BOOL {
    let mut Error: LONG;
    let mut lpFolderData: PFOLDERDATA;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwLength: DWORD;
    let mut szProfilePath: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szBuffer: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];

    // C: the profile NAME goes into szBuffer, which the path below is built from. Copied into
    // szProfilePath (overwritten on the next line), szBuffer stayed empty and "Default User" /
    // "All Users" were created as the profiles directory itself.
    StringCbCopyW(
        szBuffer.as_mut_ptr(),
        core::mem::size_of_val(&szBuffer),
        (*pProfileParams).pszProfileName,
    );

    StringCbCopyW(
        szProfilePath.as_mut_ptr(),
        core::mem::size_of_val(&szProfilePath),
        pszProfilesPath,
    );
    StringCbCatW(
        szProfilePath.as_mut_ptr(),
        core::mem::size_of_val(&szProfilePath),
        PATH_SEP_BACKSLASH.as_ptr() as LPCWSTR,
    );
    StringCbCatW(
        szProfilePath.as_mut_ptr(),
        core::mem::size_of_val(&szProfilePath),
        szBuffer.as_mut_ptr(),
    );

    if CreateDirectoryW(szProfilePath.as_mut_ptr(), core::ptr::null_mut()) == 0 {
        if GetLastError() != ERROR_ALREADY_EXISTS {
            profile_fail!("CreateStandardProfile step 1", GetLastError());

            return FALSE as BOOL;
        }

        if AppendSystemPostfix(szBuffer.as_mut_ptr(), (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) as DWORD) == 0 {
            profile_fail!("CreateStandardProfile step 2", GetLastError());

            return FALSE as BOOL;
        }

        StringCbCopyW(
            szProfilePath.as_mut_ptr(),
            core::mem::size_of_val(&szProfilePath),
            pszProfilesPath,
        );
        StringCbCatW(
            szProfilePath.as_mut_ptr(),
            core::mem::size_of_val(&szProfilePath),
            PATH_SEP_BACKSLASH.as_ptr() as LPCWSTR,
        );
        StringCbCatW(
            szProfilePath.as_mut_ptr(),
            core::mem::size_of_val(&szProfilePath),
            szBuffer.as_mut_ptr(),
        );

        if CreateDirectoryW(szProfilePath.as_mut_ptr(), core::ptr::null_mut()) == 0 {
            if GetLastError() != ERROR_ALREADY_EXISTS {
                profile_fail!("CreateStandardProfile step 3", GetLastError());

                return FALSE as BOOL;
            }
        }
    }

    dwLength = ((wcslen(szBuffer.as_mut_ptr()) + 1) * core::mem::size_of::<WCHAR>()) as DWORD;
    Error = RegSetValueExW(
        hProfileListKey,
        (*pProfileParams).pszProfileRegValue,
        0,
        REG_SZ,
        szBuffer.as_mut_ptr() as LPBYTE,
        dwLength,
    );
    if Error != ERROR_SUCCESS as LONG {
        SetLastError(Error as DWORD);
        profile_fail!("CreateStandardProfile step 4", GetLastError());

        return FALSE as BOOL;
    }

    SetEnvironmentVariableW((*pProfileParams).pszEnvVar, szProfilePath.as_mut_ptr());

    lpFolderData = (*pProfileParams).pFolderList;
    while !(*lpFolderData).lpValueName.is_null() {
        StringCbCopyW(
            szBuffer.as_mut_ptr(),
            core::mem::size_of_val(&szBuffer),
            szProfilePath.as_mut_ptr(),
        );
        StringCbCatW(
            szBuffer.as_mut_ptr(),
            core::mem::size_of_val(&szBuffer),
            PATH_SEP_BACKSLASH.as_ptr() as LPCWSTR,
        );

        dwLength = wcslen(szBuffer.as_mut_ptr()) as DWORD;
        if LoadStringW(
            hInstance as *mut core::ffi::c_void,
            (*lpFolderData).uId,
            szBuffer.as_mut_ptr().add(dwLength as usize),
            ((core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) - dwLength as usize) as i32,
        ) == 0
        {
            StringCbCatW(
                szBuffer.as_mut_ptr(),
                core::mem::size_of_val(&szBuffer),
                (*lpFolderData).lpPath,
            );
        }

        if CreateDirectoryW(szBuffer.as_mut_ptr(), core::ptr::null_mut()) == 0 {
            if GetLastError() != ERROR_ALREADY_EXISTS {
                profile_fail!("CreateStandardProfile step 5", GetLastError());

                return FALSE as BOOL;
            }
        }

        if (*lpFolderData).bHidden != 0 {
            SetFileAttributesW(szBuffer.as_mut_ptr(), FILE_ATTRIBUTE_HIDDEN);
        }

        lpFolderData = lpFolderData.offset(1);
    }

    Error = RegOpenKeyExW(
        (*pProfileParams).hRootKey,
        (*pProfileParams).pszShellFoldersKey,
        0,
        KEY_SET_VALUE,
        &mut hKey,
    );
    if Error != ERROR_SUCCESS as LONG {
        SetLastError(Error as DWORD);
        profile_fail!("CreateStandardProfile step 6", GetLastError());

        return FALSE as BOOL;
    }

    lpFolderData = (*pProfileParams).pFolderList;
    while !(*lpFolderData).lpValueName.is_null() {
        if (*lpFolderData).bShellFolder != 0 {
            StringCbCopyW(
                szBuffer.as_mut_ptr(),
                core::mem::size_of_val(&szBuffer),
                szProfilePath.as_mut_ptr(),
            );
            StringCbCatW(
                szBuffer.as_mut_ptr(),
                core::mem::size_of_val(&szBuffer),
                PATH_SEP_BACKSLASH.as_ptr() as LPCWSTR,
            );

            dwLength = wcslen(szBuffer.as_mut_ptr()) as DWORD;
            if LoadStringW(
                hInstance as *mut core::ffi::c_void,
                (*lpFolderData).uId,
                szBuffer.as_mut_ptr().add(dwLength as usize),
                ((core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) - dwLength as usize) as i32,
            ) == 0
            {
                StringCbCatW(
                    szBuffer.as_mut_ptr(),
                    core::mem::size_of_val(&szBuffer),
                    (*lpFolderData).lpPath,
                );
            }

            dwLength = ((wcslen(szBuffer.as_mut_ptr()) + 1) * core::mem::size_of::<WCHAR>()) as DWORD;
            Error = RegSetValueExW(
                hKey,
                (*lpFolderData).lpValueName,
                0,
                REG_SZ,
                szBuffer.as_mut_ptr() as LPBYTE,
                dwLength,
            );
            if Error != ERROR_SUCCESS as LONG {
                RegCloseKey(hKey);
                SetLastError(Error as DWORD);
                profile_fail!("CreateStandardProfile step 7", GetLastError());

                return FALSE as BOOL;
            }
        }

        lpFolderData = lpFolderData.offset(1);
    }

    RegCloseKey(hKey);

    Error = RegOpenKeyExW(
        (*pProfileParams).hRootKey,
        (*pProfileParams).pszUserShellFoldersKey,
        0,
        KEY_SET_VALUE,
        &mut hKey,
    );
    if Error != ERROR_SUCCESS as LONG {
        SetLastError(Error as DWORD);
        profile_fail!("CreateStandardProfile step 8", GetLastError());

        return FALSE as BOOL;
    }

    lpFolderData = (*pProfileParams).pFolderList;
    while !(*lpFolderData).lpValueName.is_null() {
        if (*lpFolderData).bUserShellFolder != 0 {
            StringCbCopyW(
                szBuffer.as_mut_ptr(),
                core::mem::size_of_val(&szBuffer),
                (*pProfileParams).pszEnvVarProfilePath,
            );
            StringCbCatW(
                szBuffer.as_mut_ptr(),
                core::mem::size_of_val(&szBuffer),
                PATH_SEP_BACKSLASH.as_ptr() as LPCWSTR,
            );

            dwLength = wcslen(szBuffer.as_mut_ptr()) as DWORD;
            if LoadStringW(
                hInstance as *mut core::ffi::c_void,
                (*lpFolderData).uId,
                szBuffer.as_mut_ptr().add(dwLength as usize),
                ((core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) - dwLength as usize) as i32,
            ) == 0
            {
                StringCbCatW(
                    szBuffer.as_mut_ptr(),
                    core::mem::size_of_val(&szBuffer),
                    (*lpFolderData).lpPath,
                );
            }

            dwLength = ((wcslen(szBuffer.as_mut_ptr()) + 1) * core::mem::size_of::<WCHAR>()) as DWORD;
            Error = RegSetValueExW(
                hKey,
                (*lpFolderData).lpValueName,
                0,
                REG_EXPAND_SZ,
                szBuffer.as_mut_ptr() as LPBYTE,
                dwLength,
            );
            if Error != ERROR_SUCCESS as LONG {
                RegCloseKey(hKey);
                SetLastError(Error as DWORD);
                profile_fail!("CreateStandardProfile step 9", GetLastError());

                return FALSE as BOOL;
            }
        }

        lpFolderData = lpFolderData.offset(1);
    }

    RegCloseKey(hKey);

    TRUE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn InitializeProfiles() -> BOOL {
    let mut Error: LONG = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwLength: DWORD = 0;
    let mut szProfilesPath: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szBuffer: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];

    // The unexpanded path goes into szBuffer (stored in the registry as REG_EXPAND_SZ) and is
    // expanded INTO szProfilesPath below. Loaded into szProfilesPath instead, the expansion of
    // the still-empty szBuffer overwrote it with "", CreateDirectoryW("") failed, and GUI setup
    // stopped at "InitializeProfiles() failed / Rebooting now!" on every install.
    if LoadStringW(
        hInstance as *mut core::ffi::c_void,
        IDS_PROFILEPATH,
        szBuffer.as_mut_ptr(),
        MAX_PATH as core::ffi::c_int,
    ) == 0
    {
        profile_fail!("LoadStringW(IDS_PROFILEPATH)", GetLastError());
        return 0;
    }

    Error = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        PROFILE_LIST_KEY.as_ptr() as LPCWSTR,
        0,
        KEY_SET_VALUE,
        &mut hKey,
    );
    if Error != ERROR_SUCCESS as LONG {
        profile_fail!("RegOpenKeyExW(ProfileList)", Error);
        SetLastError(Error as DWORD);
        return 0;
    }

    if ExpandEnvironmentStringsW(
        szBuffer.as_mut_ptr(),
        szProfilesPath.as_mut_ptr(),
        MAX_PATH,
    ) == 0
    {
        profile_fail!("ExpandEnvironmentStringsW", GetLastError());
        RegCloseKey(hKey);
        return 0;
    }

    if CreateDirectoryW(szProfilesPath.as_mut_ptr(), core::ptr::null_mut()) == 0 {
        if GetLastError() != ERROR_ALREADY_EXISTS {
            profile_fail!("CreateDirectoryW(profiles directory)", GetLastError());
            RegCloseKey(hKey);
            return 0;
        }
    }

    dwLength = ((wcslen(szBuffer.as_mut_ptr()) + 1) * core::mem::size_of::<WCHAR>()) as DWORD;
    Error = RegSetValueExW(
        hKey,
        REG_PROFILES_DIRECTORY.as_ptr() as LPCWSTR,
        0,
        REG_EXPAND_SZ,
        szBuffer.as_mut_ptr() as *const BYTE,
        dwLength,
    );
    if Error != ERROR_SUCCESS as LONG {
        profile_fail!("RegSetValueExW(ProfilesDirectory)", Error);
        RegCloseKey(hKey);
        SetLastError(Error as DWORD);
        return 0;
    }

    if CreateStandardProfile(
        szProfilesPath.as_mut_ptr(),
        hKey,
        &mut StandardProfiles[0],
    ) == 0
    {
        profile_fail!("CreateStandardProfile(Default User)", GetLastError());
        RegCloseKey(hKey);
        return 0;
    }

    if CreateStandardProfile(
        szProfilesPath.as_mut_ptr(),
        hKey,
        &mut StandardProfiles[1],
    ) == 0
    {
        profile_fail!("CreateStandardProfile(All Users)", GetLastError());
        RegCloseKey(hKey);
        return 0;
    }

    RegCloseKey(hKey);

    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn UpdateUsersShellFolderSettings(
    lpUserProfilePath: LPCWSTR,
    hUserKey: HKEY,
) -> BOOL {
    let mut szBuffer: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut dwLength: u32;
    let mut lpFolderData: *mut _FOLDERDATA;
    let mut hFoldersKey: HKEY = core::ptr::null_mut();
    let mut Error: i32;

    // L"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders"
    static SHELL_FOLDERS_KEY_BUF: [u16; 65] = [
        b'S' as u16, b'O' as u16, b'F' as u16, b'T' as u16, b'W' as u16, b'A' as u16, b'R' as u16, b'E' as u16, b'\\' as u16, b'M' as u16,
        b'i' as u16, b'c' as u16, b'r' as u16, b'o' as u16, b's' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'\\' as u16, b'W' as u16,
        b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b's' as u16, b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16,
        b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16, b'V' as u16, b'e' as u16, b'r' as u16, b's' as u16, b'i' as u16, b'o' as u16,
        b'n' as u16, b'\\' as u16, b'E' as u16, b'x' as u16, b'p' as u16, b'l' as u16, b'o' as u16, b'r' as u16, b'e' as u16, b'r' as u16,
        b'\\' as u16, b'S' as u16, b'h' as u16, b'e' as u16, b'l' as u16, b'l' as u16, b' ' as u16, b'F' as u16, b'o' as u16, b'l' as u16,
        b'd' as u16, b'e' as u16, b'r' as u16, b's' as u16, 0,
    ];
    Error = RegOpenKeyExW(
        hUserKey,
        SHELL_FOLDERS_KEY_BUF.as_ptr() as *const u16 as LPCWSTR,
        0,
        KEY_SET_VALUE,
        &mut hFoldersKey,
    ) as i32;
    if Error != ERROR_SUCCESS as i32 {
        SetLastError(Error as u32);
        return 0;
    }

    lpFolderData = core::ptr::addr_of_mut!(UserShellFolders[0]);
    while !(*lpFolderData).lpValueName.is_null() {
        if (*lpFolderData).bShellFolder != 0 {
            StringCbCopyW(
                core::ptr::addr_of_mut!(szBuffer) as *mut u16,
                core::mem::size_of_val(&szBuffer),
                lpUserProfilePath,
            );
            // L"\"
            static BACKSLASH_W: [u16; 2] = [b'\\' as u16, 0];
            StringCbCatW(
                core::ptr::addr_of_mut!(szBuffer) as *mut u16,
                core::mem::size_of_val(&szBuffer),
                BACKSLASH_W.as_ptr() as *const u16,
            );

            dwLength = wcslen(core::ptr::addr_of_mut!(szBuffer) as *mut u16) as u32;
            if LoadStringW(
                hInstance as *mut core::ffi::c_void,
                (*lpFolderData).uId,
                (core::ptr::addr_of_mut!(szBuffer) as *mut u16).offset(dwLength as isize),
                (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>() - dwLength as usize) as i32,
            ) == 0
            {
                StringCbCatW(
                    core::ptr::addr_of_mut!(szBuffer) as *mut u16,
                    core::mem::size_of_val(&szBuffer),
                    (*lpFolderData).lpPath,
                );
            }

            dwLength = ((wcslen(core::ptr::addr_of_mut!(szBuffer) as *mut u16) + 1) * core::mem::size_of::<u16>()) as u32;
            Error = RegSetValueExW(
                hFoldersKey,
                (*lpFolderData).lpValueName,
                0,
                REG_SZ,
                core::ptr::addr_of_mut!(szBuffer) as *mut u16 as LPBYTE,
                dwLength,
            ) as i32;
            if Error != ERROR_SUCCESS as i32 {
                RegCloseKey(hFoldersKey);
                SetLastError(Error as u32);
                return 0;
            }
        }

        lpFolderData = lpFolderData.offset(1);
    }

    RegCloseKey(hFoldersKey);

    return 1;
}
// enforce_import_abi: moved 1 import decl(s) into the extern block their
// exporting DLL's .spec requires: wcslen->C.
// The CRT is cdecl, Win32 is stdcall, and the C source does not say which is which —
// a wrong guess is a silent i386 stack walk on every call. The .spec is the truth.
extern "C" {
    fn wcslen(s: LPCWSTR) -> usize;
}



// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/userenv/setup.c
//  * PURPOSE:         Profile setup functions
//  * PROGRAMMERS:     Eric Kohl
//  *                  Hermes Belusca-Maito
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// #include "resources.h"
//
// typedef struct _FOLDERDATA
// {
//     LPWSTR lpValueName;
//     LPWSTR lpPath;
//     UINT uId;
//     BOOL bHidden;
//     BOOL bShellFolder;
//     BOOL bUserShellFolder;
// } FOLDERDATA, *PFOLDERDATA;
//
//
// static FOLDERDATA
// UserShellFolders[] =
// {
//     {L"AppData", L"Application Data", IDS_APPDATA, TRUE, TRUE, TRUE},
//     {L"Desktop", L"Desktop", IDS_DESKTOP, FALSE, TRUE, TRUE},
//     {L"Favorites", L"Favorites", IDS_FAVORITES, FALSE, TRUE, TRUE},
//     {L"Personal", L"My Documents", IDS_MYDOCUMENTS, FALSE, TRUE, TRUE},
//     {L"NetHood", L"NetHood", IDS_NETHOOD, TRUE, TRUE, TRUE},
//     {L"PrintHood", L"PrintHood", IDS_PRINTHOOD, TRUE, TRUE, TRUE},
//     {L"Recent", L"Recent", IDS_RECENT, TRUE, TRUE, TRUE},
//     {L"SendTo", L"SendTo", IDS_SENDTO, FALSE, TRUE, TRUE},
//     {L"Templates", L"Templates", IDS_TEMPLATES, FALSE, TRUE, TRUE},
//     {L"Start Menu", L"Start Menu", IDS_STARTMENU, FALSE, TRUE, TRUE},
//     {L"Programs", L"Start Menu\\Programs", IDS_PROGRAMS, FALSE, TRUE, TRUE},
//     {L"Startup", L"Start Menu\\Programs\\Startup", IDS_STARTUP, FALSE, TRUE, TRUE},
//     {L"Local Settings", L"Local Settings", IDS_LOCALSETTINGS, TRUE, TRUE, TRUE},
//     {L"Local AppData", L"Local Settings\\Application Data", IDS_LOCALAPPDATA, TRUE, TRUE, TRUE},
//     {L"Temp", L"Local Settings\\Temp", IDS_TEMP, FALSE, FALSE, FALSE},
//     {L"Cache", L"Local Settings\\Temporary Internet Files", IDS_CACHE, FALSE, TRUE, TRUE},
//     {L"History", L"Local Settings\\History", IDS_HISTORY, FALSE, TRUE, TRUE},
//     {L"Cookies", L"Cookies", IDS_COOKIES, FALSE, TRUE, TRUE},
//     {NULL, NULL, -1, FALSE, FALSE, FALSE}
// };
//
//
// static FOLDERDATA
// CommonShellFolders[] =
// {
//     {L"Common AppData", L"Application Data", IDS_APPDATA, TRUE, TRUE, TRUE},
//     {L"Common Desktop", L"Desktop", IDS_DESKTOP, FALSE, TRUE, TRUE},
//     {L"Common Favorites", L"Favorites", IDS_FAVORITES, FALSE, TRUE, TRUE},
//     {L"Common Documents", L"My Documents", IDS_MYDOCUMENTS, FALSE, TRUE, TRUE},
//     {L"Common Templates", L"Templates", IDS_TEMPLATES, TRUE, TRUE, TRUE},
//     {L"Common Start Menu", L"Start Menu", IDS_STARTMENU, FALSE, TRUE, TRUE},
//     {L"Common Programs", L"Start Menu\\Programs", IDS_PROGRAMS, FALSE, TRUE, TRUE},
//     {L"Common Startup", L"Start Menu\\Programs\\Startup", IDS_STARTUP, FALSE, TRUE, TRUE},
//     {NULL, NULL, -1, FALSE, FALSE, FALSE}
// };
//
//
// typedef struct _PROFILEPARAMS
// {
//     LPCWSTR pszProfileName;
//     LPCWSTR pszProfileRegValue;
//     LPCWSTR pszEnvVar;
//     LPCWSTR pszEnvVarProfilePath;
//     PFOLDERDATA pFolderList;
//     HKEY hRootKey;
//     LPCWSTR pszShellFoldersKey;
//     LPCWSTR pszUserShellFoldersKey;
// } PROFILEPARAMS, *PPROFILEPARAMS;
//
//
// static PROFILEPARAMS
// StandardProfiles[] =
// {
//     {
//         L"Default User", L"DefaultUserProfile",
//         L"USERPROFILE", L"%USERPROFILE%",
//         UserShellFolders,
//         HKEY_USERS,
//         L".Default\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Shell Folders",
//         L".Default\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders"
//     },
//     {
//         L"All Users", L"AllUsersProfile",
//         L"ALLUSERSPROFILE", L"%ALLUSERSPROFILE%",
//         CommonShellFolders,
//         HKEY_LOCAL_MACHINE,
//         L"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Shell Folders",
//         L"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders"
//     },
// };
//
//
// static
// BOOL
// CreateStandardProfile(IN LPCWSTR pszProfilesPath,
//                       IN HKEY hProfileListKey,
//                       IN PPROFILEPARAMS pProfileParams)
// {
//     LONG Error;
//     PFOLDERDATA lpFolderData;
//     HKEY hKey;
//     DWORD dwLength;
//     WCHAR szProfilePath[MAX_PATH];
//     WCHAR szBuffer[MAX_PATH];
//
//     /*
//      * Create the standard profile main directory
//      */
//
//     StringCbCopyW(szBuffer, sizeof(szBuffer), pProfileParams->pszProfileName);
//
//     /* Build the profile directory path */
//     StringCbCopyW(szProfilePath, sizeof(szProfilePath), pszProfilesPath);
//     StringCbCatW(szProfilePath, sizeof(szProfilePath), L"\\");
//     StringCbCatW(szProfilePath, sizeof(szProfilePath), szBuffer);
//
//     /* Attempt profile directory creation */
//     // FIXME: Security!
//     if (!CreateDirectoryW(szProfilePath, NULL))
//     {
//         if (GetLastError() != ERROR_ALREADY_EXISTS)
//         {
//             DPRINT1("Error: %lu\n", GetLastError());
//             return FALSE;
//         }
//
//         /* Directory existed, let's try to append the postfix */
//         if (!AppendSystemPostfix(szBuffer, ARRAYSIZE(szBuffer)))
//         {
//             DPRINT1("AppendSystemPostfix() failed\n", GetLastError());
//             return FALSE;
//         }
//
//         /* Attempt again creation with appended postfix */
//         StringCbCopyW(szProfilePath, sizeof(szProfilePath), pszProfilesPath);
//         StringCbCatW(szProfilePath, sizeof(szProfilePath), L"\\");
//         StringCbCatW(szProfilePath, sizeof(szProfilePath), szBuffer);
//
//         // FIXME: Security!
//         if (!CreateDirectoryW(szProfilePath, NULL))
//         {
//             if (GetLastError() != ERROR_ALREADY_EXISTS)
//             {
//                 DPRINT1("Error: %lu\n", GetLastError());
//                 return FALSE;
//             }
//         }
//     }
//
//     /* Set 'DefaultUserProfile' / 'AllUsersProfile' value */
//     /* Store the default user / all users profile path in the registry */
//     dwLength = (wcslen(szBuffer) + 1) * sizeof(WCHAR);
//     Error = RegSetValueExW(hProfileListKey,
//                            pProfileParams->pszProfileRegValue,
//                            0,
//                            REG_SZ,
//                            (LPBYTE)szBuffer,
//                            dwLength);
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", Error);
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     /* Set 'Default User' / 'All Users' profile */
//     SetEnvironmentVariableW(pProfileParams->pszEnvVar, szProfilePath);
//
//
//     /*
//      * Create the standard profile sub-directories and associated registry keys
//      */
//
//     /* Create 'Default User' / 'All Users' subdirectories */
//     /* FIXME: Take these paths from the registry */
//
//     lpFolderData = pProfileParams->pFolderList;
//     while (lpFolderData->lpValueName != NULL)
//     {
//         StringCbCopyW(szBuffer, sizeof(szBuffer), szProfilePath);
//         StringCbCatW(szBuffer, sizeof(szBuffer), L"\\");
//
//         /* Append the folder name */
//         dwLength = wcslen(szBuffer);
//         if (!LoadStringW(hInstance,
//                          lpFolderData->uId,
//                          &szBuffer[dwLength],
//                          ARRAYSIZE(szBuffer) - dwLength))
//         {
//             /* Use the default name instead */
//             StringCbCatW(szBuffer, sizeof(szBuffer), lpFolderData->lpPath);
//         }
//
//         // FIXME: Security!
//         if (!CreateDirectoryW(szBuffer, NULL))
//         {
//             if (GetLastError() != ERROR_ALREADY_EXISTS)
//             {
//                 DPRINT1("Error: %lu\n", GetLastError());
//                 return FALSE;
//             }
//         }
//
//         if (lpFolderData->bHidden)
//             SetFileAttributesW(szBuffer, FILE_ATTRIBUTE_HIDDEN);
//
//         lpFolderData++;
//     }
//
//     /* Set 'Shell Folders' values */
//     Error = RegOpenKeyExW(pProfileParams->hRootKey,
//                           pProfileParams->pszShellFoldersKey,
//                           0,
//                           KEY_SET_VALUE,
//                           &hKey);
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", Error);
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     /*
//      * NOTE: This is identical to UpdateUsersShellFolderSettings().
//      */
//     lpFolderData = pProfileParams->pFolderList;
//     while (lpFolderData->lpValueName != NULL)
//     {
//         if (lpFolderData->bShellFolder)
//         {
//             StringCbCopyW(szBuffer, sizeof(szBuffer), szProfilePath);
//             StringCbCatW(szBuffer, sizeof(szBuffer), L"\\");
//
//             /* Append the folder name */
//             dwLength = wcslen(szBuffer);
//             if (!LoadStringW(hInstance,
//                              lpFolderData->uId,
//                              &szBuffer[dwLength],
//                              ARRAYSIZE(szBuffer) - dwLength))
//             {
//                 /* Use the default name instead */
//                 StringCbCatW(szBuffer, sizeof(szBuffer), lpFolderData->lpPath);
//             }
//
//             dwLength = (wcslen(szBuffer) + 1) * sizeof(WCHAR);
//             Error = RegSetValueExW(hKey,
//                                    lpFolderData->lpValueName,
//                                    0,
//                                    REG_SZ,
//                                    (LPBYTE)szBuffer,
//                                    dwLength);
//             if (Error != ERROR_SUCCESS)
//             {
//                 DPRINT1("Error: %lu\n", Error);
//                 RegCloseKey(hKey);
//                 SetLastError((DWORD)Error);
//                 return FALSE;
//             }
//         }
//
//         lpFolderData++;
//     }
//
//     RegCloseKey(hKey);
//
//     /* Set 'User Shell Folders' values */
//     Error = RegOpenKeyExW(pProfileParams->hRootKey,
//                           pProfileParams->pszUserShellFoldersKey,
//                           0,
//                           KEY_SET_VALUE,
//                           &hKey);
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", Error);
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     lpFolderData = pProfileParams->pFolderList;
//     while (lpFolderData->lpValueName != NULL)
//     {
//         if (lpFolderData->bUserShellFolder)
//         {
//             StringCbCopyW(szBuffer, sizeof(szBuffer), pProfileParams->pszEnvVarProfilePath);
//             StringCbCatW(szBuffer, sizeof(szBuffer), L"\\");
//
//             /* Append the folder name */
//             dwLength = wcslen(szBuffer);
//             if (!LoadStringW(hInstance,
//                              lpFolderData->uId,
//                              &szBuffer[dwLength],
//                              ARRAYSIZE(szBuffer) - dwLength))
//             {
//                 /* Use the default name instead */
//                 StringCbCatW(szBuffer, sizeof(szBuffer), lpFolderData->lpPath);
//             }
//
//             dwLength = (wcslen(szBuffer) + 1) * sizeof(WCHAR);
//             Error = RegSetValueExW(hKey,
//                                    lpFolderData->lpValueName,
//                                    0,
//                                    REG_EXPAND_SZ,
//                                    (LPBYTE)szBuffer,
//                                    dwLength);
//             if (Error != ERROR_SUCCESS)
//             {
//                 DPRINT1("Error: %lu\n", Error);
//                 RegCloseKey(hKey);
//                 SetLastError((DWORD)Error);
//                 return FALSE;
//             }
//         }
//
//         lpFolderData++;
//     }
//
//     RegCloseKey(hKey);
//
//     return TRUE;
// }
//
//
// BOOL
// WINAPI
// InitializeProfiles(VOID)
// {
//     LONG Error;
//     HKEY hKey;
//     DWORD dwLength;
//     WCHAR szProfilesPath[MAX_PATH];
//     WCHAR szBuffer[MAX_PATH];
//
//     DPRINT("InitializeProfiles()\n");
//
//     /* Load profiles directory path */
//     if (!LoadStringW(hInstance,
//                      IDS_PROFILEPATH,
//                      szBuffer,
//                      ARRAYSIZE(szBuffer)))
//     {
//         DPRINT1("Error: %lu\n", GetLastError());
//         return FALSE;
//     }
//
//     Error = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                           L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList",
//                           0,
//                           KEY_SET_VALUE,
//                           &hKey);
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", Error);
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     /* Expand it */
//     if (!ExpandEnvironmentStringsW(szBuffer,
//                                    szProfilesPath,
//                                    ARRAYSIZE(szProfilesPath)))
//     {
//         DPRINT1("Error: %lu\n", GetLastError());
//         RegCloseKey(hKey);
//         return FALSE;
//     }
//
//     /* Create profiles directory */
//     // FIXME: Security!
//     if (!CreateDirectoryW(szProfilesPath, NULL))
//     {
//         if (GetLastError() != ERROR_ALREADY_EXISTS)
//         {
//             DPRINT1("Error: %lu\n", GetLastError());
//             RegCloseKey(hKey);
//             return FALSE;
//         }
//     }
//
//     /* Store the profiles directory path (unexpanded) in the registry */
//     dwLength = (wcslen(szBuffer) + 1) * sizeof(WCHAR);
//     Error = RegSetValueExW(hKey,
//                            L"ProfilesDirectory",
//                            0,
//                            REG_EXPAND_SZ,
//                            (LPBYTE)szBuffer,
//                            dwLength);
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", Error);
//         RegCloseKey(hKey);
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     /* Create 'Default User' profile directory path */
//     if (!CreateStandardProfile(szProfilesPath, hKey, &StandardProfiles[0]))
//     {
//         DPRINT1("CreateStandardProfile(L\"%S\") failed.\n", StandardProfiles[0].pszProfileName);
//         RegCloseKey(hKey);
//         return FALSE;
//     }
//
//     /* Create 'All Users' profile directory path */
//     if (!CreateStandardProfile(szProfilesPath, hKey, &StandardProfiles[1]))
//     {
//         DPRINT1("CreateStandardProfile(L\"%S\") failed.\n", StandardProfiles[1].pszProfileName);
//         RegCloseKey(hKey);
//         return FALSE;
//     }
//
//     RegCloseKey(hKey);
//
//     DPRINT("Success\n");
//
//     return TRUE;
// }
//
//
// /*
//  * NOTE: See CreateStandardProfile() too.
//  * Used by registry.c!CreateUserHive()
//  */
// BOOL
// UpdateUsersShellFolderSettings(LPCWSTR lpUserProfilePath,
//                                HKEY hUserKey)
// {
//     WCHAR szBuffer[MAX_PATH];
//     DWORD dwLength;
//     PFOLDERDATA lpFolderData;
//     HKEY hFoldersKey;
//     LONG Error;
//
//     DPRINT("UpdateUsersShellFolderSettings() called\n");
//
//     DPRINT("User profile path: %S\n", lpUserProfilePath);
//
//     Error = RegOpenKeyExW(hUserKey,
//                           L"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Shell Folders",
//                           0,
//                           KEY_SET_VALUE,
//                           &hFoldersKey);
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", Error);
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     lpFolderData = &UserShellFolders[0];
//     while (lpFolderData->lpValueName != NULL)
//     {
//         if (lpFolderData->bShellFolder)
//         {
//             StringCbCopyW(szBuffer, sizeof(szBuffer), lpUserProfilePath);
//             StringCbCatW(szBuffer, sizeof(szBuffer), L"\\");
//
//             /* Append the folder name */
//             dwLength = wcslen(szBuffer);
//             if (!LoadStringW(hInstance,
//                              lpFolderData->uId,
//                              &szBuffer[dwLength],
//                              ARRAYSIZE(szBuffer) - dwLength))
//             {
//                 /* Use the default name instead */
//                 StringCbCatW(szBuffer, sizeof(szBuffer), lpFolderData->lpPath);
//             }
//
//             DPRINT("%S: %S\n", lpFolderData->lpValueName, szBuffer);
//
//             dwLength = (wcslen(szBuffer) + 1) * sizeof(WCHAR);
//             Error = RegSetValueExW(hFoldersKey,
//                                    lpFolderData->lpValueName,
//                                    0,
//                                    REG_SZ,
//                                    (LPBYTE)szBuffer,
//                                    dwLength);
//             if (Error != ERROR_SUCCESS)
//             {
//                 DPRINT1("Error: %lu\n", Error);
//                 RegCloseKey(hFoldersKey);
//                 SetLastError((DWORD)Error);
//                 return FALSE;
//             }
//         }
//
//         lpFolderData++;
//     }
//
//     RegCloseKey(hFoldersKey);
//
//     DPRINT("UpdateUsersShellFolderSettings() done\n");
//
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
