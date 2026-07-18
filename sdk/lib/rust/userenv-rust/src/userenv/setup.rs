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

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut UserShellFolders: [FOLDERDATA; 19] = unsafe {
    [
        _FOLDERDATA {
            lpValueName: b"A\0p\0p\0D\0a\0t\0a\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"A\0p\0p\0l\0i\0c\0a\0t\0i\0o\0n\0 \0D\0a\0t\0a\0\0\0"
                .as_ptr() as LPWSTR,
            uId: IDS_APPDATA as UINT,
            bHidden: TRUE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: b"D\0e\0s\0k\0t\0o\0p\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"D\0e\0s\0k\0t\0o\0p\0\0\0"
                .as_ptr() as LPWSTR,
            uId: IDS_DESKTOP as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: b"F\0a\0v\0o\0r\0i\0t\0e\0s\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"F\0a\0v\0o\0r\0i\0t\0e\0s\0\0\0"
                .as_ptr() as LPWSTR,
            uId: IDS_FAVORITES as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: b"P\0e\0r\0s\0o\0n\0a\0l\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"M\0y\0 \0D\0o\0c\0u\0m\0e\0n\0t\0s\0\0\0"
                .as_ptr() as LPWSTR,
            uId: IDS_MYDOCUMENTS as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: b"N\0e\0t\0H\0o\0o\0d\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"N\0e\0t\0H\0o\0o\0d\0\0\0"
                .as_ptr() as LPWSTR,
            uId: IDS_NETHOOD as UINT,
            bHidden: TRUE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: b"P\0r\0i\0n\0t\0H\0o\0o\0d\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"P\0r\0i\0n\0t\0H\0o\0o\0d\0\0\0"
                .as_ptr() as LPWSTR,
            uId: IDS_PRINTHOOD as UINT,
            bHidden: TRUE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: b"R\0e\0c\0e\0n\0t\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"R\0e\0c\0e\0n\0t\0\0\0"
                .as_ptr() as LPWSTR,
            uId: IDS_RECENT as UINT,
            bHidden: TRUE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: b"S\0e\0n\0d\0T\0o\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"S\0e\0n\0d\0T\0o\0\0\0"
                .as_ptr() as LPWSTR,
            uId: IDS_SENDTO as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: b"T\0e\0m\0p\0l\0a\0t\0e\0s\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"T\0e\0m\0p\0l\0a\0t\0e\0s\0\0\0"
                .as_ptr() as LPWSTR,
            uId: IDS_TEMPLATES as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: b"S\0t\0a\0r\0t\0 \0M\0e\0n\0u\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"S\0t\0a\0r\0t\0 \0M\0e\0n\0u\0\0\0"
                .as_ptr() as LPWSTR,
            uId: IDS_STARTMENU as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: b"P\0r\0o\0g\0r\0a\0m\0s\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"S\0t\0a\0r\0t\0 \0M\0e\0n\0u\0\\\0P\0r\0o\0g\0r\0a\0m\0s\0\0\0"
                .as_ptr() as LPWSTR,
            uId: IDS_PROGRAMS as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: b"S\0t\0a\0r\0t\0u\0p\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"S\0t\0a\0r\0t\0 \0M\0e\0n\0u\0\\\0P\0r\0o\0g\0r\0a\0m\0s\0\\\0S\0t\0a\0r\0t\0u\0p\0\0\0"
                .as_ptr() as LPWSTR,
            uId: IDS_STARTUP as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: b"L\0o\0c\0a\0l\0 \0S\0e\0t\0t\0i\0n\0g\0s\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"L\0o\0c\0a\0l\0 \0S\0e\0t\0t\0i\0n\0g\0s\0\0\0"
                .as_ptr() as LPWSTR,
            uId: IDS_LOCALSETTINGS as UINT,
            bHidden: TRUE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: b"L\0o\0c\0a\0l\0 \0A\0p\0p\0D\0a\0t\0a\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"L\0o\0c\0a\0l\0 \0S\0e\0t\0t\0i\0n\0g\0s\0\\\0A\0p\0p\0l\0i\0c\0a\0t\0i\0o\0n\0 \0D\0a\0t\0a\0\0\0"
                .as_ptr() as LPWSTR,
            uId: IDS_LOCALAPPDATA as UINT,
            bHidden: TRUE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: b"T\0e\0m\0p\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"L\0o\0c\0a\0l\0 \0S\0e\0t\0t\0i\0n\0g\0s\0\\\0T\0e\0m\0p\0\0\0"
                .as_ptr() as LPWSTR,
            uId: IDS_TEMP as UINT,
            bHidden: FALSE,
            bShellFolder: FALSE,
            bUserShellFolder: FALSE,
        },
        _FOLDERDATA {
            lpValueName: b"C\0a\0c\0h\0e\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"L\0o\0c\0a\0l\0 \0S\0e\0t\0t\0i\0n\0g\0s\0\\\0T\0e\0m\0p\0o\0r\0a\0r\0y\0 \0I\0n\0t\0e\0r\0n\0e\0t\0 \0F\0i\0l\0e\0s\0\0\0"
                .as_ptr() as LPWSTR,
            uId: IDS_CACHE as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: b"H\0i\0s\0t\0o\0r\0y\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"L\0o\0c\0a\0l\0 \0S\0e\0t\0t\0i\0n\0g\0s\0\\\0H\0i\0s\0t\0o\0r\0y\0\0\0"
                .as_ptr() as LPWSTR,
            uId: IDS_HISTORY as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: b"C\0o\0o\0k\0i\0e\0s\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"C\0o\0o\0k\0i\0e\0s\0\0\0"
                .as_ptr() as LPWSTR,
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
            lpValueName: b"C\0o\0m\0m\0o\0n\0 \0A\0p\0p\0D\0a\0t\0a\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"A\0p\0p\0l\0i\0c\0a\0t\0i\0o\0n\0 \0D\0a\0t\0a\0\0\0"
                .as_ptr() as LPWSTR,
            uId: IDS_APPDATA as UINT,
            bHidden: TRUE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: b"C\0o\0m\0m\0o\0n\0 \0D\0e\0s\0k\0t\0o\0p\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"D\0e\0s\0k\0t\0o\0p\0\0\0"
                .as_ptr() as LPWSTR,
            uId: IDS_DESKTOP as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: b"C\0o\0m\0m\0o\0n\0 \0F\0a\0v\0o\0r\0i\0t\0e\0s\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"F\0a\0v\0o\0r\0i\0t\0e\0s\0\0\0"
                .as_ptr() as LPWSTR,
            uId: IDS_FAVORITES as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: b"C\0o\0m\0m\0o\0n\0 \0D\0o\0c\0u\0m\0e\0n\0t\0s\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"M\0y\0 \0D\0o\0c\0u\0m\0e\0n\0t\0s\0\0\0"
                .as_ptr() as LPWSTR,
            uId: IDS_MYDOCUMENTS as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: b"C\0o\0m\0m\0o\0n\0 \0T\0e\0m\0p\0l\0a\0t\0e\0s\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"T\0e\0m\0p\0l\0a\0t\0e\0s\0\0\0"
                .as_ptr() as LPWSTR,
            uId: IDS_TEMPLATES as UINT,
            bHidden: TRUE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: b"C\0o\0m\0m\0o\0n\0 \0S\0t\0a\0r\0t\0 \0M\0e\0n\0u\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"S\0t\0a\0r\0t\0 \0M\0e\0n\0u\0\0\0"
                .as_ptr() as LPWSTR,
            uId: IDS_STARTMENU as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: b"C\0o\0m\0m\0o\0n\0 \0P\0r\0o\0g\0r\0a\0m\0s\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"S\0t\0a\0r\0t\0 \0M\0e\0n\0u\0\\\0P\0r\0o\0g\0r\0a\0m\0s\0\0\0"
                .as_ptr() as LPWSTR,
            uId: IDS_PROGRAMS as UINT,
            bHidden: FALSE,
            bShellFolder: TRUE,
            bUserShellFolder: TRUE,
        },
        _FOLDERDATA {
            lpValueName: b"C\0o\0m\0m\0o\0n\0 \0S\0t\0a\0r\0t\0u\0p\0\0\0"
                .as_ptr() as LPWSTR,
            lpPath: b"S\0t\0a\0r\0t\0 \0M\0e\0n\0u\0\\\0P\0r\0o\0g\0r\0a\0m\0s\0\\\0S\0t\0a\0r\0t\0u\0p\0\0\0"
                .as_ptr() as LPWSTR,
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

static mut StandardProfiles: [PROFILEPARAMS; 2] = unsafe {
    [
        _PROFILEPARAMS {
            pszProfileName: b"D\0e\0f\0a\0u\0l\0t\0 \0U\0s\0e\0r\0\0\0"
                .as_ptr() as LPCWSTR,
            pszProfileRegValue: b"D\0e\0f\0a\0u\0l\0t\0U\0s\0e\0r\0P\0r\0o\0f\0i\0l\0e\0\0\0"
                .as_ptr() as LPCWSTR,
            pszEnvVar: b"U\0S\0E\0R\0P\0R\0O\0F\0I\0L\0E\0\0\0"
                .as_ptr() as LPCWSTR,
            pszEnvVarProfilePath: b"%\0U\0S\0E\0R\0P\0R\0O\0F\0I\0L\0E\0%\0\0\0"
                .as_ptr() as LPCWSTR,
            pFolderList: &raw const UserShellFolders as PFOLDERDATA,
            hRootKey: HKEY_USERS,
            pszShellFoldersKey: b".\0D\0e\0f\0a\0u\0l\0t\0\\\0S\0O\0F\0T\0W\0A\0R\0E\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0W\0i\0n\0d\0o\0w\0s\0\\\0C\0u\0r\0r\0e\0n\0t\0V\0e\0r\0s\0i\0o\0n\0\\\0E\0x\0p\0l\0o\0r\0e\0r\0\\\0S\0h\0e\0l\0l\0 \0F\0o\0l\0d\0e\0r\0s\0\0\0"
                .as_ptr() as LPCWSTR,
            pszUserShellFoldersKey: b".\0D\0e\0f\0a\0u\0l\0t\0\\\0S\0O\0F\0T\0W\0A\0R\0E\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0W\0i\0n\0d\0o\0w\0s\0\\\0C\0u\0r\0r\0e\0n\0t\0V\0e\0r\0s\0i\0o\0n\0\\\0E\0x\0p\0l\0o\0r\0e\0r\0\\\0U\0s\0e\0r\0 \0S\0h\0e\0l\0l\0 \0F\0o\0l\0d\0e\0r\0s\0\0\0"
                .as_ptr() as LPCWSTR,
        },
        _PROFILEPARAMS {
            pszProfileName: b"A\0l\0l\0 \0U\0s\0e\0r\0s\0\0\0"
                .as_ptr() as LPCWSTR,
            pszProfileRegValue: b"A\0l\0l\0U\0s\0e\0r\0s\0P\0r\0o\0f\0i\0l\0e\0\0\0"
                .as_ptr() as LPCWSTR,
            pszEnvVar: b"A\0L\0L\0U\0S\0E\0R\0S\0P\0R\0O\0F\0I\0L\0E\0\0\0"
                .as_ptr() as LPCWSTR,
            pszEnvVarProfilePath: b"%\0A\0L\0L\0U\0S\0E\0R\0S\0P\0R\0O\0F\0I\0L\0E\0%\0\0\0"
                .as_ptr() as LPCWSTR,
            pFolderList: &raw const CommonShellFolders as PFOLDERDATA,
            hRootKey: HKEY_LOCAL_MACHINE,
            pszShellFoldersKey: b"S\0O\0F\0T\0W\0A\0R\0E\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0W\0i\0n\0d\0o\0w\0s\0\\\0C\0u\0r\0r\0e\0n\0t\0V\0e\0r\0s\0i\0o\0n\0\\\0E\0x\0p\0l\0o\0r\0e\0r\0\\\0S\0h\0e\0l\0l\0 \0F\0o\0l\0d\0e\0r\0s\0\0\0"
                .as_ptr() as LPCWSTR,
            pszUserShellFoldersKey: b"S\0O\0F\0T\0W\0A\0R\0E\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0W\0i\0n\0d\0o\0w\0s\0\\\0C\0u\0r\0r\0e\0n\0t\0V\0e\0r\0s\0i\0o\0n\0\\\0E\0x\0p\0l\0o\0r\0e\0r\0\\\0U\0s\0e\0r\0 \0S\0h\0e\0l\0l\0 \0F\0o\0l\0d\0e\0r\0s\0\0\0"
                .as_ptr() as LPCWSTR,
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

pub static mut standardProfiles: [_PROFILEPARAMS; 2] = [
    _PROFILEPARAMS {
        pszProfileName: unsafe { core::mem::transmute(b"Default User\0".as_ptr()) },
        pszProfileRegValue: unsafe { core::mem::transmute(b"DefaultUserProfile\0".as_ptr()) },
        pszEnvVar: unsafe { core::mem::transmute(b"USERPROFILE\0".as_ptr()) },
        pszEnvVarProfilePath: unsafe { core::mem::transmute(b"%USERPROFILE%\0".as_ptr()) },
        pFolderList: unsafe { &UserShellFolders[0] as *const _ as *mut _ },
        hRootKey: unsafe { core::mem::transmute(-2147483646isize) },
        pszShellFoldersKey: unsafe { core::mem::transmute(b".Default\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Shell Folders\0".as_ptr()) },
        pszUserShellFoldersKey: unsafe { core::mem::transmute(b".Default\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders\0".as_ptr()) },
    },
    _PROFILEPARAMS {
        pszProfileName: unsafe { core::mem::transmute(b"All Users\0".as_ptr()) },
        pszProfileRegValue: unsafe { core::mem::transmute(b"AllUsersProfile\0".as_ptr()) },
        pszEnvVar: unsafe { core::mem::transmute(b"ALLUSERSPROFILE\0".as_ptr()) },
        pszEnvVarProfilePath: unsafe { core::mem::transmute(b"%ALLUSERSPROFILE%\0".as_ptr()) },
        pFolderList: unsafe { &CommonShellFolders[0] as *const _ as *mut _ },
        hRootKey: unsafe { core::mem::transmute(-2147483648isize) },
        pszShellFoldersKey: unsafe { core::mem::transmute(b"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Shell Folders\0".as_ptr()) },
        pszUserShellFoldersKey: unsafe { core::mem::transmute(b"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders\0".as_ptr()) },
    },
];

const IDS_PROFILEPATH: UINT = 0;
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

    StringCbCopyW(
        szProfilePath.as_mut_ptr(),
        core::mem::size_of_val(&szProfilePath),
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
        b"\\0".as_ptr() as LPCWSTR,
    );
    StringCbCatW(
        szProfilePath.as_mut_ptr(),
        core::mem::size_of_val(&szProfilePath),
        szBuffer.as_mut_ptr(),
    );

    if CreateDirectoryW(szProfilePath.as_mut_ptr(), core::ptr::null_mut()) == 0 {
        if GetLastError() != ERROR_ALREADY_EXISTS {
            return FALSE as BOOL;
        }

        if AppendSystemPostfix(szBuffer.as_mut_ptr(), (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) as DWORD) == 0 {
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
            b"\\0".as_ptr() as LPCWSTR,
        );
        StringCbCatW(
            szProfilePath.as_mut_ptr(),
            core::mem::size_of_val(&szProfilePath),
            szBuffer.as_mut_ptr(),
        );

        if CreateDirectoryW(szProfilePath.as_mut_ptr(), core::ptr::null_mut()) == 0 {
            if GetLastError() != ERROR_ALREADY_EXISTS {
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
            b"\\0".as_ptr() as LPCWSTR,
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
                b"\\0".as_ptr() as LPCWSTR,
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
                b"\\0".as_ptr() as LPCWSTR,
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

    if LoadStringW(
        hInstance as *mut core::ffi::c_void,
        IDS_PROFILEPATH,
        szProfilesPath.as_mut_ptr(),
        MAX_PATH as core::ffi::c_int,
    ) == 0
    {
        return 0;
    }

    Error = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        b"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList\0".as_ptr() as LPCWSTR,
        0,
        KEY_SET_VALUE,
        &mut hKey,
    );
    if Error != ERROR_SUCCESS as LONG {
        SetLastError(Error as DWORD);
        return 0;
    }

    if ExpandEnvironmentStringsW(
        szBuffer.as_mut_ptr(),
        szProfilesPath.as_mut_ptr(),
        MAX_PATH,
    ) == 0
    {
        RegCloseKey(hKey);
        return 0;
    }

    if CreateDirectoryW(szProfilesPath.as_mut_ptr(), core::ptr::null_mut()) == 0 {
        if GetLastError() != ERROR_ALREADY_EXISTS {
            RegCloseKey(hKey);
            return 0;
        }
    }

    dwLength = ((wcslen(szBuffer.as_mut_ptr()) + 1) * core::mem::size_of::<WCHAR>()) as DWORD;
    Error = RegSetValueExW(
        hKey,
        b"ProfilesDirectory\0".as_ptr() as LPCWSTR,
        0,
        REG_EXPAND_SZ,
        szBuffer.as_mut_ptr() as *const BYTE,
        dwLength,
    );
    if Error != ERROR_SUCCESS as LONG {
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
        RegCloseKey(hKey);
        return 0;
    }

    if CreateStandardProfile(
        szProfilesPath.as_mut_ptr(),
        hKey,
        &mut StandardProfiles[1],
    ) == 0
    {
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

    Error = RegOpenKeyExW(
        hUserKey,
        b"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Shell Folders\0".as_ptr() as *const u16 as LPCWSTR,
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
            StringCbCatW(
                core::ptr::addr_of_mut!(szBuffer) as *mut u16,
                core::mem::size_of_val(&szBuffer),
                b"\\\0".as_ptr() as *const u16,
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
