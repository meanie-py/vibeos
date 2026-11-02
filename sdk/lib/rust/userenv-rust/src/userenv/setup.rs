use crate::prelude::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::userenv::directory::CreateDirectoryW;
use crate::prelude::RegOpenKeyExW;
use crate::prelude::RegCloseKey;

extern "system" {
    fn ExpandEnvironmentStringsW(lpSrc: LPCWSTR, lpDst: *mut WCHAR, cchDst: DWORD) -> DWORD;
    fn GetLastError() -> DWORD;
    fn RegSetValueExW(hKey: HKEY, lpValueName: LPCWSTR, Reserved: DWORD, dwType: DWORD, lpData: *const BYTE, cbData: DWORD) -> LONG;
    fn SetLastError(dwErrCode: DWORD);
}
extern "C" {
    fn wcslen(s: LPCWSTR) -> usize;
}

// An ASCII literal as a NUL-terminated, 2-byte-aligned UTF-16 static, built at compile time.
macro_rules! w {
    ($s:expr $(,)?) => {{
        const INPUT: &[u8] = $s.as_bytes();
        static OUT: [u16; INPUT.len() + 1] = {
            let mut out = [0u16; INPUT.len() + 1];
            let mut i = 0;
            while i < INPUT.len() {
                assert!(INPUT[i] < 0x80, "w!() takes ASCII literals only");
                out[i] = INPUT[i] as u16;
                i += 1;
            }
            out
        };
        OUT.as_ptr()
    }};
}

// Upstream f0a83320d79 ("Use Vista style users folder") retired the localized folder-name
// string table (resources.h IDS_*): every folder below is now named by its table entry alone,
// under %SystemDrive%\Users and %SystemDrive%\ProgramData.
#[repr(C)]
pub struct _FOLDERDATA {
    pub lpValueName: LPCWSTR,
    pub lpPath: LPCWSTR,
    pub bHidden: BOOL,
    pub bShellFolder: BOOL,
    pub bUserShellFolder: BOOL,
    /// Created under ProgramData rather than the profile (the Public profile's start menu and
    /// templates live there on Vista and later).
    pub bProgramData: BOOL,
}
pub type FOLDERDATA = _FOLDERDATA;
pub type PFOLDERDATA = *const _FOLDERDATA;
// The tables are read-only, and LPCWSTR is a raw pointer, which is not Sync.
unsafe impl Sync for _FOLDERDATA {}

const fn fd(name: LPCWSTR, path: LPCWSTR, hidden: bool, shell: bool, user_shell: bool, program_data: bool) -> FOLDERDATA {
    _FOLDERDATA {
        lpValueName: name,
        lpPath: path,
        bHidden: hidden as BOOL,
        bShellFolder: shell as BOOL,
        bUserShellFolder: user_shell as BOOL,
        bProgramData: program_data as BOOL,
    }
}
const END: FOLDERDATA = fd(core::ptr::null(), core::ptr::null(), false, false, false, false);

// Two corrections to upstream's table: its "My Pictures" entry says L"Music", and its Temp and
// Cache paths spell "Appdata" (harmless on a case-insensitive volume, but the registry shows it).
static UserShellFolders: [FOLDERDATA; 26] = [
    fd(w!("AppData Root"), w!("AppData"), true, false, false, false),
    fd(w!("AppData"), w!("AppData\\Roaming"), false, true, true, false),
    fd(w!("Desktop"), w!("Desktop"), false, true, true, false),
    fd(w!("{374DE290-123F-4565-9164-39C4925E467B}"), w!("Downloads"), false, true, true, false),
    fd(w!("Favorites"), w!("Favorites"), false, true, true, false),
    fd(w!("Personal"), w!("Documents"), false, true, true, false),
    fd(w!("My Music"), w!("Music"), false, true, true, false),
    fd(w!("My Pictures"), w!("Pictures"), false, true, true, false),
    fd(w!("My Video"), w!("Videos"), false, true, true, false),
    fd(w!("NetHood"), w!("AppData\\Roaming\\Microsoft\\Windows\\Network Shortcuts"), true, true, true, false),
    fd(w!("PrintHood"), w!("AppData\\Roaming\\Microsoft\\Windows\\Printer Shortcuts"), true, true, true, false),
    fd(w!("Recent"), w!("AppData\\Roaming\\Microsoft\\Windows\\Recent"), true, true, true, false),
    fd(w!("SendTo"), w!("AppData\\Roaming\\Microsoft\\Windows\\SendTo"), false, true, true, false),
    fd(w!("CD Burning"), w!("AppData\\Roaming\\Microsoft\\Windows\\Burn\\Burn"), false, true, true, false),
    fd(w!("Templates"), w!("Templates"), false, true, true, false),
    fd(w!("Start Menu"), w!("AppData\\Roaming\\Microsoft\\Windows\\Start Menu"), false, true, true, false),
    fd(w!("Programs"), w!("AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs"), false, true, true, false),
    fd(w!("Administrative Tools"), w!("AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Administrative Tools"), false, true, true, false),
    fd(w!("Startup"), w!("AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Startup"), false, true, true, false),
    fd(w!("Local AppData"), w!("AppData\\Local"), false, true, true, false),
    fd(w!("{A520A1A4-1780-4FF6-BD18-167343C5AF16}"), w!("AppData\\LocalLow"), false, false, false, false),
    fd(w!("Temp"), w!("AppData\\Local\\Temp"), false, false, false, false),
    fd(w!("Cache"), w!("AppData\\Local\\Microsoft\\Windows\\INetCache"), false, true, true, false),
    fd(w!("History"), w!("AppData\\Local\\Microsoft\\Windows\\History"), false, true, true, false),
    fd(w!("Cookies"), w!("AppData\\Local\\Microsoft\\Windows\\INetCookies"), false, true, true, false),
    END,
];

static CommonShellFolders: [FOLDERDATA; 11] = [
    fd(w!("{3D644C9B-1FB8-4f30-9B45-F670235F79C0}"), w!("Downloads"), false, true, true, false),
    fd(w!("Common Desktop"), w!("Desktop"), false, true, true, false),
    fd(w!("Common Documents"), w!("Documents"), false, true, true, false),
    fd(w!("Common Templates"), w!("Microsoft\\Windows\\Templates"), true, true, true, true),
    fd(w!("Common Start Menu"), w!("Microsoft\\Windows\\Start Menu"), false, true, true, true),
    fd(w!("Common Programs"), w!("Microsoft\\Windows\\Start Menu\\Programs"), false, true, true, true),
    fd(w!("Common Startup"), w!("Microsoft\\Windows\\Start Menu\\Programs\\Startup"), false, true, true, true),
    fd(w!("CommonMusic"), w!("Music"), false, true, true, false),
    fd(w!("CommonPictures"), w!("Pictures"), false, true, true, false),
    fd(w!("CommonVideo"), w!("Videos"), false, true, true, false),
    END,
];

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
pub type PROFILEPARAMS = _PROFILEPARAMS;
pub type PPROFILEPARAMS = *const _PROFILEPARAMS;
unsafe impl Sync for _PROFILEPARAMS {}

// The C has ONE StandardProfiles table, rooted at HKEY_USERS and HKEY_LOCAL_MACHINE. (This
// crate once shipped a dead second copy with the wrong roots; keep it at one.)
static StandardProfiles: [PROFILEPARAMS; 2] = [
    _PROFILEPARAMS {
        pszProfileName: w!("Default"),
        pszProfileRegValue: w!("Default"),
        pszEnvVar: w!("USERPROFILE"),
        pszEnvVarProfilePath: w!("%USERPROFILE%"),
        pFolderList: UserShellFolders.as_ptr(),
        hRootKey: HKEY_USERS,
        pszShellFoldersKey: w!(".Default\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Shell Folders"),
        pszUserShellFoldersKey: w!(".Default\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders"),
    },
    _PROFILEPARAMS {
        pszProfileName: w!("Public"),
        pszProfileRegValue: w!("Public"),
        pszEnvVar: w!("PUBLIC"),
        pszEnvVarProfilePath: w!("%PUBLIC%"),
        pFolderList: CommonShellFolders.as_ptr(),
        hRootKey: HKEY_LOCAL_MACHINE,
        pszShellFoldersKey: w!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Shell Folders"),
        pszUserShellFoldersKey: w!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders"),
    },
];

const CCH: usize = MAX_PATH as usize;
const CB: usize = CCH * core::mem::size_of::<WCHAR>();

/// `base` + `\` + `tail` into `dst`.
unsafe fn join(dst: &mut [WCHAR; CCH], base: LPCWSTR, tail: LPCWSTR) {
    StringCbCopyW(dst.as_mut_ptr(), CB, base);
    StringCbCatW(dst.as_mut_ptr(), CB, w!("\\"));
    StringCbCatW(dst.as_mut_ptr(), CB, tail);
}

/// Create `Path` and any missing parents. Upstream's version strips the last component with
/// PathCchRemoveFileSpec (pathcch_static, which userenv does not link here); for a path this
/// function builds, that is "cut at the last backslash".
unsafe fn CreateDirectoryRecursive(Path: LPCWSTR, SecurityAttributes: LPSECURITY_ATTRIBUTES) -> BOOL {
    if Path.is_null() || *Path == 0 {
        return FALSE;
    }
    if CreateDirectoryW(Path, core::ptr::null_mut()) != 0 {
        return TRUE;
    }
    let Error = GetLastError();
    if Error == ERROR_ALREADY_EXISTS {
        return TRUE;
    }
    if Error != ERROR_PATH_NOT_FOUND {
        return FALSE;
    }

    let mut Parent: [WCHAR; CCH] = [0; CCH];
    let len = wcslen(Path);
    if len >= CCH {
        return FALSE;
    }
    core::ptr::copy_nonoverlapping(Path, Parent.as_mut_ptr(), len);
    let Some(cut) = Parent[..len].iter().rposition(|&c| c == b'\\' as u16) else {
        return FALSE;
    };
    // A root ("C:\") has no parent to create.
    if cut == 0 || Parent[cut - 1] == b':' as u16 {
        return FALSE;
    }
    Parent[cut] = 0;

    if CreateDirectoryRecursive(Parent.as_ptr(), SecurityAttributes) == 0 {
        return FALSE;
    }
    if CreateDirectoryW(Path, SecurityAttributes) != 0 {
        return TRUE;
    }
    (GetLastError() == ERROR_ALREADY_EXISTS) as BOOL
}

unsafe fn CreateStandardProfile(
    pszProfilesPath: LPCWSTR,
    pszProgramDataPath: LPCWSTR,
    pProfileParams: PPROFILEPARAMS,
) -> BOOL {
    let mut Error: LONG;
    let mut lpFolderData: PFOLDERDATA;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut szProfilePath: [WCHAR; CCH] = [0; CCH];
    let mut szBuffer: [WCHAR; CCH] = [0; CCH];

    // The profile NAME goes into szBuffer, which the path is built from (and which
    // AppendSystemPostfix extends if the directory already exists).
    StringCbCopyW(szBuffer.as_mut_ptr(), CB, (*pProfileParams).pszProfileName);
    join(&mut szProfilePath, pszProfilesPath, szBuffer.as_ptr());

    // FIXME: Security!
    if CreateDirectoryW(szProfilePath.as_ptr(), core::ptr::null_mut()) == 0 {
        if GetLastError() != ERROR_ALREADY_EXISTS {
            DPRINT1!("CreateStandardProfile: profile directory", GetLastError());
            return FALSE;
        }
        if AppendSystemPostfix(szBuffer.as_mut_ptr(), CCH as DWORD) == 0 {
            DPRINT1!("CreateStandardProfile: AppendSystemPostfix", GetLastError());
            return FALSE;
        }
        join(&mut szProfilePath, pszProfilesPath, szBuffer.as_ptr());
        if CreateDirectoryW(szProfilePath.as_ptr(), core::ptr::null_mut()) == 0
            && GetLastError() != ERROR_ALREADY_EXISTS
        {
            DPRINT1!("CreateStandardProfile: postfixed profile directory", GetLastError());
            return FALSE;
        }
    }

    // USERPROFILE / PUBLIC for the rest of setup. (The DefaultUserProfile / AllUsersProfile
    // registry values are gone: the ProfileList's Default and Public values hold full paths.)
    SetEnvironmentVariableW((*pProfileParams).pszEnvVar, szProfilePath.as_ptr());

    let folder_base = |f: PFOLDERDATA, profile: &[WCHAR; CCH]| -> LPCWSTR {
        if (*f).bProgramData != 0 { pszProgramDataPath } else { profile.as_ptr() }
    };

    lpFolderData = (*pProfileParams).pFolderList;
    while !(*lpFolderData).lpValueName.is_null() {
        join(&mut szBuffer, folder_base(lpFolderData, &szProfilePath), (*lpFolderData).lpPath);
        // FIXME: Security!
        if CreateDirectoryRecursive(szBuffer.as_ptr(), core::ptr::null_mut()) == 0
            && GetLastError() != ERROR_ALREADY_EXISTS
        {
            DPRINT1!("CreateStandardProfile: folder", GetLastError());
            return FALSE;
        }
        if (*lpFolderData).bHidden != 0 {
            SetFileAttributesW(szBuffer.as_ptr(), FILE_ATTRIBUTE_HIDDEN);
        }
        lpFolderData = lpFolderData.add(1);
    }

    // Shell Folders: expanded paths, REG_SZ.
    Error = RegOpenKeyExW((*pProfileParams).hRootKey, (*pProfileParams).pszShellFoldersKey, 0, KEY_SET_VALUE, &mut hKey);
    if Error != ERROR_SUCCESS as LONG {
        SetLastError(Error as DWORD);
        DPRINT1!("CreateStandardProfile: open Shell Folders", Error);
        return FALSE;
    }
    lpFolderData = (*pProfileParams).pFolderList;
    while !(*lpFolderData).lpValueName.is_null() {
        if (*lpFolderData).bShellFolder != 0 {
            join(&mut szBuffer, folder_base(lpFolderData, &szProfilePath), (*lpFolderData).lpPath);
            let cb = ((wcslen(szBuffer.as_ptr()) + 1) * core::mem::size_of::<WCHAR>()) as DWORD;
            Error = RegSetValueExW(hKey, (*lpFolderData).lpValueName, 0, REG_SZ, szBuffer.as_ptr() as *const BYTE, cb);
            if Error != ERROR_SUCCESS as LONG {
                RegCloseKey(hKey);
                SetLastError(Error as DWORD);
                DPRINT1!("CreateStandardProfile: set Shell Folders value", Error);
                return FALSE;
            }
        }
        lpFolderData = lpFolderData.add(1);
    }
    RegCloseKey(hKey);

    // User Shell Folders: %USERPROFILE% / %PUBLIC% / %ProgramData% relative, REG_EXPAND_SZ.
    Error = RegOpenKeyExW((*pProfileParams).hRootKey, (*pProfileParams).pszUserShellFoldersKey, 0, KEY_SET_VALUE, &mut hKey);
    if Error != ERROR_SUCCESS as LONG {
        SetLastError(Error as DWORD);
        DPRINT1!("CreateStandardProfile: open User Shell Folders", Error);
        return FALSE;
    }
    lpFolderData = (*pProfileParams).pFolderList;
    while !(*lpFolderData).lpValueName.is_null() {
        if (*lpFolderData).bUserShellFolder != 0 {
            let base = if (*lpFolderData).bProgramData != 0 { w!("%ProgramData%") } else { (*pProfileParams).pszEnvVarProfilePath };
            join(&mut szBuffer, base, (*lpFolderData).lpPath);
            let cb = ((wcslen(szBuffer.as_ptr()) + 1) * core::mem::size_of::<WCHAR>()) as DWORD;
            Error = RegSetValueExW(hKey, (*lpFolderData).lpValueName, 0, REG_EXPAND_SZ, szBuffer.as_ptr() as *const BYTE, cb);
            if Error != ERROR_SUCCESS as LONG {
                RegCloseKey(hKey);
                SetLastError(Error as DWORD);
                DPRINT1!("CreateStandardProfile: set User Shell Folders value", Error);
                return FALSE;
            }
        }
        lpFolderData = lpFolderData.add(1);
    }
    RegCloseKey(hKey);

    TRUE
}

/// Expand `unexpanded` into `expanded`, create it, and store the unexpanded form as `value`
/// under the ProfileList key (REG_EXPAND_SZ).
unsafe fn create_root(hKey: HKEY, value: LPCWSTR, unexpanded: LPCWSTR, expanded: &mut [WCHAR; CCH], hidden: bool) -> BOOL {
    if ExpandEnvironmentStringsW(unexpanded, expanded.as_mut_ptr(), CCH as DWORD) == 0 {
        DPRINT1!("InitializeProfiles: ExpandEnvironmentStringsW", GetLastError());
        return FALSE;
    }
    // FIXME: Security!
    if CreateDirectoryW(expanded.as_ptr(), core::ptr::null_mut()) == 0 && GetLastError() != ERROR_ALREADY_EXISTS {
        DPRINT1!("InitializeProfiles: CreateDirectoryW", GetLastError());
        return FALSE;
    }
    if hidden {
        SetFileAttributesW(expanded.as_ptr(), FILE_ATTRIBUTE_HIDDEN);
    }
    let cb = ((wcslen(unexpanded) + 1) * core::mem::size_of::<WCHAR>()) as DWORD;
    let Error = RegSetValueExW(hKey, value, 0, REG_EXPAND_SZ, unexpanded as *const BYTE, cb);
    if Error != ERROR_SUCCESS as LONG {
        DPRINT1!("InitializeProfiles: RegSetValueExW", Error);
        SetLastError(Error as DWORD);
        return FALSE;
    }
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn InitializeProfiles() -> BOOL {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut szProfilesPath: [WCHAR; CCH] = [0; CCH];
    let mut szProgramDataPath: [WCHAR; CCH] = [0; CCH];

    let Error = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        w!("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList"),
        0,
        KEY_SET_VALUE,
        &mut hKey,
    );
    if Error != ERROR_SUCCESS as LONG {
        DPRINT1!("InitializeProfiles: RegOpenKeyExW(ProfileList)", Error);
        SetLastError(Error as DWORD);
        return FALSE;
    }

    // The fixed Vista roots. Before f0a83320d79 the profiles root was the localized
    // IDS_PROFILEPATH string; that resource is gone, and loading it is what failed.
    let ok = 'init: {
        if create_root(hKey, w!("ProgramData"), w!("%SystemDrive%\\ProgramData"), &mut szProgramDataPath, true) == 0
            || create_root(hKey, w!("ProfilesDirectory"), w!("%SystemDrive%\\Users"), &mut szProfilesPath, false) == 0
        {
            break 'init false;
        }
        if CreateStandardProfile(szProfilesPath.as_ptr(), szProgramDataPath.as_ptr(), &StandardProfiles[0]) == 0 {
            DPRINT1!("CreateStandardProfile(Default)", GetLastError());
            break 'init false;
        }
        if CreateStandardProfile(szProfilesPath.as_ptr(), szProgramDataPath.as_ptr(), &StandardProfiles[1]) == 0 {
            DPRINT1!("CreateStandardProfile(Public)", GetLastError());
            break 'init false;
        }
        true
    };

    RegCloseKey(hKey);
    ok as BOOL
}

#[no_mangle]
pub unsafe extern "C" fn UpdateUsersShellFolderSettings(lpUserProfilePath: LPCWSTR, hUserKey: HKEY) -> BOOL {
    let mut szBuffer: [WCHAR; CCH] = [0; CCH];
    let mut hFoldersKey: HKEY = core::ptr::null_mut();

    let mut Error = RegOpenKeyExW(
        hUserKey,
        w!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Shell Folders"),
        0,
        KEY_SET_VALUE,
        &mut hFoldersKey,
    );
    if Error != ERROR_SUCCESS as LONG {
        SetLastError(Error as DWORD);
        return FALSE;
    }

    let mut lpFolderData: PFOLDERDATA = UserShellFolders.as_ptr();
    while !(*lpFolderData).lpValueName.is_null() {
        if (*lpFolderData).bShellFolder != 0 {
            join(&mut szBuffer, lpUserProfilePath, (*lpFolderData).lpPath);
            let cb = ((wcslen(szBuffer.as_ptr()) + 1) * core::mem::size_of::<WCHAR>()) as DWORD;
            Error = RegSetValueExW(hFoldersKey, (*lpFolderData).lpValueName, 0, REG_SZ, szBuffer.as_ptr() as *const BYTE, cb);
            if Error != ERROR_SUCCESS as LONG {
                RegCloseKey(hFoldersKey);
                SetLastError(Error as DWORD);
                return FALSE;
            }
        }
        lpFolderData = lpFolderData.add(1);
    }

    RegCloseKey(hFoldersKey);
    TRUE
}

// ============================================================================
// ORIGINAL C SOURCE (dll/win32/userenv/setup.c, upstream f0a83320d79)
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
//     BOOL bHidden;
//     BOOL bShellFolder;
//     BOOL bUserShellFolder;
//     BOOL bProgramData;
// } FOLDERDATA, *PFOLDERDATA;
//
//
// static FOLDERDATA
// UserShellFolders[] =
// {
//     {L"AppData Root", L"AppData", TRUE, FALSE, FALSE, FALSE},
//     {L"AppData", L"AppData\\Roaming", FALSE, TRUE, TRUE, FALSE},
//     {L"Desktop", L"Desktop", FALSE, TRUE, TRUE, FALSE},
//     {L"{374DE290-123F-4565-9164-39C4925E467B}", L"Downloads", FALSE, TRUE, TRUE, FALSE},
//     {L"Favorites", L"Favorites", FALSE, TRUE, TRUE, FALSE},
//     {L"Personal", L"Documents", FALSE, TRUE, TRUE, FALSE},
//     {L"My Music", L"Music", FALSE, TRUE, TRUE, FALSE},
//     {L"My Pictures", L"Music", FALSE, TRUE, TRUE, FALSE},
//     {L"My Video", L"Videos", FALSE, TRUE, TRUE, FALSE},
//     {L"NetHood", L"AppData\\Roaming\\Microsoft\\Windows\\Network Shortcuts", TRUE, TRUE, TRUE, FALSE},
//     {L"PrintHood", L"AppData\\Roaming\\Microsoft\\Windows\\Printer Shortcuts", TRUE, TRUE, TRUE, FALSE},
//     {L"Recent", L"AppData\\Roaming\\Microsoft\\Windows\\Recent", TRUE, TRUE, TRUE, FALSE},
//     {L"SendTo", L"AppData\\Roaming\\Microsoft\\Windows\\SendTo", FALSE, TRUE, TRUE, FALSE},
//     {L"CD Burning", L"AppData\\Roaming\\Microsoft\\Windows\\Burn\\Burn", FALSE, TRUE, TRUE, FALSE},
//     {L"Templates", L"Templates", FALSE, TRUE, TRUE, FALSE},
//     {L"Start Menu", L"AppData\\Roaming\\Microsoft\\Windows\\Start Menu", FALSE, TRUE, TRUE, FALSE},
//     {L"Programs", L"AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs", FALSE, TRUE, TRUE, FALSE},
//     {L"Administrative Tools", L"AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Administrative Tools", FALSE, TRUE, TRUE, FALSE},
//     {L"Startup", L"AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Startup", FALSE, TRUE, TRUE, FALSE},
//     {L"Local AppData", L"AppData\\Local", FALSE, TRUE, TRUE, FALSE},
//     {L"{A520A1A4-1780-4FF6-BD18-167343C5AF16}", L"AppData\\LocalLow", FALSE, FALSE, FALSE, FALSE},
//     {L"Temp", L"Appdata\\Local\\Temp", FALSE, FALSE, FALSE, FALSE},
//     {L"Cache", L"Appdata\\Local\\Microsoft\\Windows\\INetCache", FALSE, TRUE, TRUE, FALSE},
//     {L"History", L"AppData\\Local\\Microsoft\\Windows\\History", FALSE, TRUE, TRUE, FALSE},
//     {L"Cookies", L"AppData\\Local\\Microsoft\\Windows\\INetCookies", FALSE, TRUE, TRUE, FALSE},
//     {NULL, NULL, FALSE, FALSE, FALSE, FALSE}
// };
//
//
// static FOLDERDATA
// CommonShellFolders[] =
// {
//     {L"{3D644C9B-1FB8-4f30-9B45-F670235F79C0}", L"Downloads", FALSE, TRUE, TRUE, FALSE},
//     {L"Common Desktop", L"Desktop", FALSE, TRUE, TRUE, FALSE},
//     {L"Common Documents", L"Documents", FALSE, TRUE, TRUE, FALSE},
//     {L"Common Templates", L"Microsoft\\Windows\\Templates", TRUE, TRUE, TRUE, TRUE},
//     {L"Common Start Menu", L"Microsoft\\Windows\\Start Menu", FALSE, TRUE, TRUE, TRUE},
//     {L"Common Programs", L"Microsoft\\Windows\\Start Menu\\Programs", FALSE, TRUE, TRUE, TRUE},
//     {L"Common Startup", L"Microsoft\\Windows\\Start Menu\\Programs\\Startup", FALSE, TRUE, TRUE, TRUE},
//     {L"CommonMusic", L"Music", FALSE, TRUE, TRUE, FALSE},
//     {L"CommonPictures", L"Pictures", FALSE, TRUE, TRUE, FALSE},
//     {L"CommonVideo", L"Videos", FALSE, TRUE, TRUE, FALSE},
//     {NULL, NULL, FALSE, FALSE, FALSE, FALSE}
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
//         L"Default", L"Default",
//         L"USERPROFILE", L"%USERPROFILE%",
//         UserShellFolders,
//         HKEY_USERS,
//         L".Default\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Shell Folders",
//         L".Default\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders"
//     },
//     {
//         L"Public", L"Public",
//         L"PUBLIC", L"%PUBLIC%",
//         CommonShellFolders,
//         HKEY_LOCAL_MACHINE,
//         L"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Shell Folders",
//         L"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders"
//     },
// };
//
// static BOOL
// CreateDirectoryRecursive(
//     _In_ PCWSTR Path,
//     _In_ LPSECURITY_ATTRIBUTES SecurityAttributes)
// {
//     WCHAR Parent[MAX_PATH];
//     DWORD Error;
//
//     if (!Path || !*Path)
//         return FALSE;
//
//     if (CreateDirectoryW(Path, NULL))
//         return TRUE;
//
//     Error = GetLastError();
//
//     if (Error == ERROR_ALREADY_EXISTS)
//         return TRUE;
//
//     if (Error != ERROR_PATH_NOT_FOUND)
//         return FALSE;
//
//     if (FAILED(StringCchCopyW(Parent, ARRAYSIZE(Parent), Path)))
//         return FALSE;
//
//     if (FAILED(PathCchRemoveFileSpec(Parent, ARRAYSIZE(Parent))))
//         return FALSE;
//
//     if (!Parent[0] || !lstrcmpW(Parent, Path))
//         return FALSE;
//
//     if (!CreateDirectoryRecursive(Parent, SecurityAttributes))
//         return FALSE;
//
//     if (CreateDirectoryW(Path, SecurityAttributes))
//         return TRUE;
//
//     return GetLastError() == ERROR_ALREADY_EXISTS;
// }
//
// static
// BOOL
// CreateStandardProfile(
//     _In_ LPCWSTR pszProfilesPath,
//     _In_ LPCWSTR pszProgramDataPath,
//     _In_ HKEY hProfileListKey,
//     _In_ PPROFILEPARAMS pProfileParams)
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
//         if (lpFolderData->bProgramData)
//         {
//             StringCbCopyW(szBuffer, sizeof(szBuffer), pszProgramDataPath);
//         }
//         else
//         {
//             StringCbCopyW(szBuffer, sizeof(szBuffer), szProfilePath);
//         }
//
//         StringCbCatW(szBuffer, sizeof(szBuffer), L"\\");
//         StringCbCatW(szBuffer, sizeof(szBuffer), lpFolderData->lpPath);
//
//         // FIXME: Security!
//         if (!CreateDirectoryRecursive(szBuffer, NULL))
//         {
//             if (GetLastError() != ERROR_ALREADY_EXISTS)
//             {
//                 DPRINT1("Failed to create %ws, error: %lu\n", szBuffer, GetLastError());
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
//             if (lpFolderData->bProgramData)
//             {
//                 StringCbCopyW(szBuffer, sizeof(szBuffer), pszProgramDataPath);
//             }
//             else
//             {
//                 StringCbCopyW(szBuffer, sizeof(szBuffer), szProfilePath);
//             }
//             
//             StringCbCatW(szBuffer, sizeof(szBuffer), L"\\");
//             StringCbCatW(szBuffer, sizeof(szBuffer), lpFolderData->lpPath);
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
//             if (lpFolderData->bProgramData)
//                 StringCbCopyW(szBuffer, sizeof(szBuffer), L"%ProgramData%");
//             else
//                 StringCbCopyW(szBuffer, sizeof(szBuffer), pProfileParams->pszEnvVarProfilePath);
//             StringCbCatW(szBuffer, sizeof(szBuffer), L"\\");
//             StringCbCatW(szBuffer, sizeof(szBuffer), lpFolderData->lpPath);
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
//     WCHAR szProgramDataPath[MAX_PATH];
//     WCHAR szBuffer[MAX_PATH];
//
//     DPRINT("InitializeProfiles()\n");
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
//     /* Create ProgramData directory */
//     StringCchCopyW(szBuffer, _countof(szBuffer), L"%SystemDrive%\\ProgramData");
//     if (!ExpandEnvironmentStringsW(szBuffer,
//                                    szProgramDataPath,
//                                    ARRAYSIZE(szProgramDataPath)))
//     {
//         DPRINT1("Error: %lu\n", GetLastError());
//         RegCloseKey(hKey);
//         return FALSE;
//     }
//     if (!CreateDirectoryW(szProgramDataPath, NULL))
//     {
//         if (GetLastError() != ERROR_ALREADY_EXISTS)
//         {
//             DPRINT1("Error: %lu\n", GetLastError());
//             RegCloseKey(hKey);
//             return FALSE;
//         }
//     }
//     SetFileAttributesW(szProgramDataPath, FILE_ATTRIBUTE_HIDDEN);
//
//     /* Store the profiles directory path (unexpanded) in the registry */
//     dwLength = (wcslen(szBuffer) + 1) * sizeof(WCHAR);
//     Error = RegSetValueExW(hKey,
//                            L"ProgramData",
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
//     /* Expand users path */
//     wcscpy(szBuffer, L"%SystemDrive%\\Users");
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
//     if (!CreateStandardProfile(szProfilesPath, szProgramDataPath, hKey, &StandardProfiles[0]))
//     {
//         DPRINT1("CreateStandardProfile(L\"%S\") failed.\n", StandardProfiles[0].pszProfileName);
//         RegCloseKey(hKey);
//         return FALSE;
//     }
//
//     /* Create 'All Users' profile directory path */
//     if (!CreateStandardProfile(szProfilesPath, szProgramDataPath, hKey, &StandardProfiles[1]))
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
//             StringCbCatW(szBuffer, sizeof(szBuffer), lpFolderData->lpPath);
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
// END ORIGINAL C SOURCE
