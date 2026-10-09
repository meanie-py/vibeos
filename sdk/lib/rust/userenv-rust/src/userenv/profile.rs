use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn GetUserSidStringFromToken(hToken: HANDLE, SidString: *mut _UNICODE_STRING) -> BOOL;
    static mut LocalSystemAuthority: SID_IDENTIFIER_AUTHORITY;
}
extern "system" {
    fn GetUserSid(hToken: *mut core::ffi::c_void) -> PSID; // stdcall WINAPI export (userenv.spec:142)
}
extern "C" {
    fn _wcsupr(s: *mut u16) -> *mut u16; // CRT (cdecl): upper-case a wide string in place
    fn wcscat(strDestination: *mut u16, strSource: *const u16) -> *mut u16;
    fn wcscpy(strDestination: *mut u16, strSource: *const u16) -> *mut u16;
    fn wcslen(str: *const u16) -> usize;
    fn RemoveDirectoryPath(lpPath: LPCWSTR) -> i32;
    // StringCchCopyW/StringCbCatW/StringCbCopyW → prelude inline shims (strsafe `static inline`s,
    // no linkable symbol). ARRAYSIZE → the array's .len() at the call site (a macro, not a fn).
    fn CopyDirectory(lpExistingDirectory: LPCWSTR, lpNewDirectory: LPCWSTR) -> BOOL;
    fn CreateUserHive(lpUserName: LPCWSTR, lpUserProfilePath: LPCWSTR) -> BOOL;
    fn swprintf(buffer: *mut ::core::ffi::c_ushort, format: *const ::core::ffi::c_ushort, ...) -> ::core::ffi::c_int;
    // GetUserSid → extern "system" block above (stdcall WINAPI export).
    fn ReportError(dwFlags: DWORD, lpFormat: LPCWSTR, ...) -> core::ffi::c_void;
}

// L"%SystemRoot%"
static ENV_SYSTEM_ROOT_PCT: [u16; 13] = [
    b'%' as u16, b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16,
    b'R' as u16, b'o' as u16, b'o' as u16, b't' as u16, b'%' as u16, 0,
];
// L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList"
static PROFILE_LIST_KEY3: [u16; 57] = [
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
// L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList\\"
static PROFILE_LIST_KEY_TRAILING_BS: [u16; 58] = [
    b'S' as u16, b'O' as u16, b'F' as u16, b'T' as u16, b'W' as u16, b'A' as u16, b'R' as u16,
    b'E' as u16, b'\\' as u16, b'M' as u16, b'i' as u16, b'c' as u16, b'r' as u16, b'o' as u16,
    b's' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'\\' as u16, b'W' as u16, b'i' as u16,
    b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b's' as u16, b' ' as u16, b'N' as u16,
    b'T' as u16, b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16,
    b'n' as u16, b't' as u16, b'V' as u16, b'e' as u16, b'r' as u16, b's' as u16, b'i' as u16,
    b'o' as u16, b'n' as u16, b'\\' as u16, b'P' as u16, b'r' as u16, b'o' as u16, b'f' as u16,
    b'i' as u16, b'l' as u16, b'e' as u16, b'L' as u16, b'i' as u16, b's' as u16, b't' as u16,
    b'\\' as u16, 0,
];
// L"RefCount"
static REG_REF_COUNT: [u16; 9] = [
    b'R' as u16, b'e' as u16, b'f' as u16, b'C' as u16, b'o' as u16, b'u' as u16, b'n' as u16,
    b't' as u16, 0,
];
// L"ProfileLoadTimeLow"
static REG_PROFILE_LOAD_TIME_LOW: [u16; 19] = [
    b'P' as u16, b'r' as u16, b'o' as u16, b'f' as u16, b'i' as u16, b'l' as u16, b'e' as u16,
    b'L' as u16, b'o' as u16, b'a' as u16, b'd' as u16, b'T' as u16, b'i' as u16, b'm' as u16,
    b'e' as u16, b'L' as u16, b'o' as u16, b'w' as u16, 0,
];
// L"ProfileLoadTimeHigh"
static REG_PROFILE_LOAD_TIME_HIGH: [u16; 20] = [
    b'P' as u16, b'r' as u16, b'o' as u16, b'f' as u16, b'i' as u16, b'l' as u16, b'e' as u16,
    b'L' as u16, b'o' as u16, b'a' as u16, b'd' as u16, b'T' as u16, b'i' as u16, b'm' as u16,
    b'e' as u16, b'H' as u16, b'i' as u16, b'g' as u16, b'h' as u16, 0,
];
// L"Flags"
static REG_FLAGS: [u16; 6] = [
    b'F' as u16, b'l' as u16, b'a' as u16, b'g' as u16, b's' as u16, 0,
];
// L"State"
static REG_STATE: [u16; 6] = [
    b'S' as u16, b't' as u16, b'a' as u16, b't' as u16, b'e' as u16, 0,
];
// L"Sid"
static REG_SID: [u16; 4] = [
    b'S' as u16, b'i' as u16, b'd' as u16, 0,
];
// L"ProfileImagePath"
static REG_PROFILE_IMAGE_PATH: [u16; 17] = [
    b'P' as u16, b'r' as u16, b'o' as u16, b'f' as u16, b'i' as u16, b'l' as u16, b'e' as u16,
    b'I' as u16, b'm' as u16, b'a' as u16, b'g' as u16, b'e' as u16, b'P' as u16, b'a' as u16,
    b't' as u16, b'h' as u16, 0,
];
// L"ProfilesDirectory"
static REG_PROFILES_DIRECTORY2: [u16; 18] = [
    b'P' as u16, b'r' as u16, b'o' as u16, b'f' as u16, b'i' as u16, b'l' as u16, b'e' as u16,
    b's' as u16, b'D' as u16, b'i' as u16, b'r' as u16, b'e' as u16, b'c' as u16, b't' as u16,
    b'o' as u16, b'r' as u16, b'y' as u16, 0,
];
// L"DefaultUserProfile"
static REG_DEFAULT_USER_PROFILE: [u16; 19] = [
    b'D' as u16, b'e' as u16, b'f' as u16, b'a' as u16, b'u' as u16, b'l' as u16, b't' as u16,
    b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'P' as u16, b'r' as u16, b'o' as u16,
    b'f' as u16, b'i' as u16, b'l' as u16, b'e' as u16, 0,
];
// L"\\"
static PATH_SEP_BACKSLASH2: [u16; 2] = [
    b'\\' as u16, 0,
];
// L"\\ntuser.dat"
static NTUSER_DAT_SUFFIX: [u16; 12] = [
    b'\\' as u16, b'n' as u16, b't' as u16, b'u' as u16, b's' as u16, b'e' as u16, b'r' as u16,
    b'.' as u16, b'd' as u16, b'a' as u16, b't' as u16, 0,
];
// L"ReactOS could not load the locally stored user profile. Possible causes of this error include insufficient security rights or a corrupt local profile.\n\nError: %lu\n"
static ERR_PROFILE_LOAD_FAILED: [u16; 164] = [
    b'R' as u16, b'e' as u16, b'a' as u16, b'c' as u16, b't' as u16, b'O' as u16, b'S' as u16,
    b' ' as u16, b'c' as u16, b'o' as u16, b'u' as u16, b'l' as u16, b'd' as u16, b' ' as u16,
    b'n' as u16, b'o' as u16, b't' as u16, b' ' as u16, b'l' as u16, b'o' as u16, b'a' as u16,
    b'd' as u16, b' ' as u16, b't' as u16, b'h' as u16, b'e' as u16, b' ' as u16, b'l' as u16,
    b'o' as u16, b'c' as u16, b'a' as u16, b'l' as u16, b'l' as u16, b'y' as u16, b' ' as u16,
    b's' as u16, b't' as u16, b'o' as u16, b'r' as u16, b'e' as u16, b'd' as u16, b' ' as u16,
    b'u' as u16, b's' as u16, b'e' as u16, b'r' as u16, b' ' as u16, b'p' as u16, b'r' as u16,
    b'o' as u16, b'f' as u16, b'i' as u16, b'l' as u16, b'e' as u16, b'.' as u16, b' ' as u16,
    b'P' as u16, b'o' as u16, b's' as u16, b's' as u16, b'i' as u16, b'b' as u16, b'l' as u16,
    b'e' as u16, b' ' as u16, b'c' as u16, b'a' as u16, b'u' as u16, b's' as u16, b'e' as u16,
    b's' as u16, b' ' as u16, b'o' as u16, b'f' as u16, b' ' as u16, b't' as u16, b'h' as u16,
    b'i' as u16, b's' as u16, b' ' as u16, b'e' as u16, b'r' as u16, b'r' as u16, b'o' as u16,
    b'r' as u16, b' ' as u16, b'i' as u16, b'n' as u16, b'c' as u16, b'l' as u16, b'u' as u16,
    b'd' as u16, b'e' as u16, b' ' as u16, b'i' as u16, b'n' as u16, b's' as u16, b'u' as u16,
    b'f' as u16, b'f' as u16, b'i' as u16, b'c' as u16, b'i' as u16, b'e' as u16, b'n' as u16,
    b't' as u16, b' ' as u16, b's' as u16, b'e' as u16, b'c' as u16, b'u' as u16, b'r' as u16,
    b'i' as u16, b't' as u16, b'y' as u16, b' ' as u16, b'r' as u16, b'i' as u16, b'g' as u16,
    b'h' as u16, b't' as u16, b's' as u16, b' ' as u16, b'o' as u16, b'r' as u16, b' ' as u16,
    b'a' as u16, b' ' as u16, b'c' as u16, b'o' as u16, b'r' as u16, b'r' as u16, b'u' as u16,
    b'p' as u16, b't' as u16, b' ' as u16, b'l' as u16, b'o' as u16, b'c' as u16, b'a' as u16,
    b'l' as u16, b' ' as u16, b'p' as u16, b'r' as u16, b'o' as u16, b'f' as u16, b'i' as u16,
    b'l' as u16, b'e' as u16, b'.' as u16, 10, 10, b'E' as u16, b'r' as u16, b'r' as u16,
    b'o' as u16, b'r' as u16, b':' as u16, b' ' as u16, b'%' as u16, b'l' as u16, b'u' as u16,
    10, 0,
];

// ---- file-scope preamble (translated) ----
pub const NDEBUG: u32 = 1;

const HKEY_LOCAL_MACHINE: HKEY = 0x80000002 as HKEY;
#[no_mangle]
pub unsafe extern "C" fn AppendSystemPostfix(mut lpName: LPWSTR, mut dwMaxLength: DWORD) -> BOOL {
    let mut szSystemRoot: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut lpszPostfix: LPWSTR;
    let mut lpszPtr: LPWSTR;

    if ExpandEnvironmentStringsW(
        ENV_SYSTEM_ROOT_PCT.as_ptr() as LPCWSTR,
        szSystemRoot.as_mut_ptr(),
        (szSystemRoot.len() as DWORD),
    ) == 0
    {
        return 0;
    }

    _wcsupr(szSystemRoot.as_mut_ptr());

    szSystemRoot[2] = '.' as WCHAR;
    lpszPostfix = szSystemRoot.as_mut_ptr().offset(2);
    lpszPtr = lpszPostfix;
    while *lpszPtr != 0 {
        if *lpszPtr == '\\' as WCHAR {
            *lpszPtr = '_' as WCHAR;
        }
        lpszPtr = lpszPtr.offset(1);
    }

    if wcslen(lpName as *const u16)
        .wrapping_add(wcslen(lpszPostfix as *const u16))
        .wrapping_add(1)
        >= dwMaxLength as usize
    {
        SetLastError(ERROR_BUFFER_OVERFLOW as DWORD);
        return 0;
    }

    wcscat(lpName as *mut u16, lpszPostfix as *const u16);

    return 1;
}
unsafe extern "C" fn AcquireRemoveRestorePrivilege(mut bAcquire: BOOL) -> BOOL {
    let mut bRet: BOOL = 0;
    let mut Token: HANDLE = core::ptr::null_mut();
    let mut TokenPriv: TOKEN_PRIVILEGES = TOKEN_PRIVILEGES {
        PrivilegeCount: 0,
        Privileges: [LUID_AND_ATTRIBUTES {
            Luid: LUID {
                LowPart: 0,
                HighPart: 0,
            },
            Attributes: 0,
        }; 1],
    };

    if OpenProcessToken(
        GetCurrentProcess(),
        TOKEN_ADJUST_PRIVILEGES as DWORD,
        &mut Token,
    ) != 0
    {
        TokenPriv.PrivilegeCount = 1u32;
        TokenPriv.Privileges[0].Attributes = if bAcquire != 0 { SE_PRIVILEGE_ENABLED } else { 0u32 };

        if LookupPrivilegeValueA(
            core::ptr::null(),
            b"SeRestorePrivilege\0".as_ptr() as LPCSTR,
            &mut TokenPriv.Privileges[0].Luid,
        ) != 0
        {
            bRet = AdjustTokenPrivileges(
                Token,
                0,
                &mut TokenPriv,
                0u32,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );

            if bRet == 0 {
            } else if GetLastError() == ERROR_NOT_ALL_ASSIGNED {
                bRet = 0;
            }
        }

        CloseHandle(Token);
    }

    bRet
}
unsafe extern "C" fn CheckForLoadedProfile(mut hToken: HANDLE) -> BOOL {
    let mut SidString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<u16>(),
    };
    let mut hKey: HKEY = core::ptr::null_mut();

    if GetUserSidStringFromToken(hToken, &raw mut SidString) == 0 {
        return 0;
    }

    if RegOpenKeyExW(
        HKEY_USERS,
        SidString.Buffer,
        0,
        MAXIMUM_ALLOWED,
        &raw mut hKey,
    ) != 0
    {
        RtlFreeUnicodeString(&raw mut SidString);
        return 0;
    }

    RegCloseKey(hKey);
    RtlFreeUnicodeString(&raw mut SidString);

    return 1;
}
unsafe extern "C" fn CreateProfileMutex(pszSidString: PWSTR) -> HANDLE {
    let mut SecurityDescriptor: _SECURITY_DESCRIPTOR = core::mem::zeroed();
    let mut SecurityAttributes: SECURITY_ATTRIBUTES = core::mem::zeroed();
    let mut pszMutexName: PWSTR = core::ptr::null_mut();
    let mut hMutex: HANDLE = core::ptr::null_mut();

    // L"Global\userenv:  User Profile Mutex for "
    static PROFILE_MUTEX_PREFIX_W: [u16; 41] = [
        b'G' as u16, b'l' as u16, b'o' as u16, b'b' as u16, b'a' as u16, b'l' as u16, b'\\' as u16, b'u' as u16, b's' as u16, b'e' as u16,
        b'r' as u16, b'e' as u16, b'n' as u16, b'v' as u16, b':' as u16, b' ' as u16, b' ' as u16, b'U' as u16, b's' as u16, b'e' as u16,
        b'r' as u16, b' ' as u16, b'P' as u16, b'r' as u16, b'o' as u16, b'f' as u16, b'i' as u16, b'l' as u16, b'e' as u16, b' ' as u16,
        b'M' as u16, b'u' as u16, b't' as u16, b'e' as u16, b'x' as u16, b' ' as u16, b'f' as u16, b'o' as u16, b'r' as u16, b' ' as u16,
        0,
    ];
    let prefix = PROFILE_MUTEX_PREFIX_W.as_ptr() as *const u16;
    let prefix_len = wcslen(prefix);
    let sid_len = wcslen(pszSidString);
    
    pszMutexName = HeapAlloc(
        GetProcessHeap(),
        0,
        (prefix_len + sid_len + 1) * core::mem::size_of::<u16>(),
    ) as PWSTR;
    
    if pszMutexName.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return core::ptr::null_mut();
    }

    wcscpy(pszMutexName, prefix);
    wcscat(pszMutexName, pszSidString);

    InitializeSecurityDescriptor(&mut SecurityDescriptor, SECURITY_DESCRIPTOR_REVISION);

    SetSecurityDescriptorDacl(
        &mut SecurityDescriptor,
        1i32,
        core::ptr::null_mut(),
        0i32,
    );

    SecurityAttributes.nLength = core::mem::size_of::<SECURITY_ATTRIBUTES>() as u32;
    SecurityAttributes.lpSecurityDescriptor = &mut SecurityDescriptor as *mut _ as *mut core::ffi::c_void;
    SecurityAttributes.bInheritHandle = 0i32;

    hMutex = CreateMutexW(&mut SecurityAttributes, 0i32, pszMutexName as LPCWSTR);
    
    if hMutex.is_null() {
        let _ = GetLastError();
    }

    HeapFree(GetProcessHeap(), 0, pszMutexName as *mut core::ffi::c_void);

    hMutex
}
unsafe extern "C" fn IncrementRefCount(
    mut pszSidString: PWSTR,
    mut pdwRefCount: PDWORD) -> DWORD {
    let mut hProfilesKey: HKEY = core::ptr::null_mut();
    let mut hProfileKey: HKEY = core::ptr::null_mut();
    let mut dwRefCount: DWORD = 0;
    let mut dwLength: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut dwError: DWORD = 0;

    'done: {
    dwError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        PROFILE_LIST_KEY3.as_ptr() as LPCWSTR,
        0,
        KEY_QUERY_VALUE,
        &mut hProfilesKey);
    if dwError != ERROR_SUCCESS {
        break 'done;
    }

    dwError = RegOpenKeyExW(
        hProfilesKey,
        pszSidString as LPCWSTR,
        0,
        KEY_QUERY_VALUE | KEY_SET_VALUE,
        &mut hProfileKey);
    if dwError != ERROR_SUCCESS {
        break 'done;
    }

    dwLength = core::mem::size_of::<DWORD>() as DWORD;
    RegQueryValueExW(
        hProfileKey,
        REG_REF_COUNT.as_ptr() as LPCWSTR,
        core::ptr::null_mut(),
        &mut dwType,
        &mut dwRefCount as *mut DWORD as LPBYTE,
        &mut dwLength);

    dwRefCount = dwRefCount.wrapping_add(1);

    dwLength = core::mem::size_of::<DWORD>() as DWORD;
    dwError = RegSetValueExW(
        hProfileKey,
        REG_REF_COUNT.as_ptr() as LPCWSTR,
        0,
        REG_DWORD,
        &dwRefCount as *const DWORD as PBYTE,
        dwLength) as u32;
    if dwError != ERROR_SUCCESS {
        break 'done;
    }

    if !pdwRefCount.is_null() {
        *pdwRefCount = dwRefCount;
    }

    }
    {
        if !hProfileKey.is_null() {
            RegCloseKey(hProfileKey);
        }

        if !hProfilesKey.is_null() {
            RegCloseKey(hProfilesKey);
        }
    }

    dwError
}
unsafe extern "C" fn DecrementRefCount(mut pszSidString: PWSTR, mut pdwRefCount: PDWORD) -> DWORD {
    let mut hProfilesKey: HKEY = core::ptr::null_mut();
    let mut hProfileKey: HKEY = core::ptr::null_mut();
    let mut dwRefCount: DWORD = 0;
    let mut dwLength: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut dwError: DWORD = 0;

    'done: {
    dwError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        PROFILE_LIST_KEY3.as_ptr() as LPCWSTR,
        0,
        KEY_QUERY_VALUE,
        &mut hProfilesKey,
    );
    if dwError != ERROR_SUCCESS {
        break 'done;
    }

    dwError = RegOpenKeyExW(
        hProfilesKey,
        pszSidString as LPCWSTR,
        0,
        KEY_QUERY_VALUE | KEY_SET_VALUE,
        &mut hProfileKey,
    );
    if dwError != ERROR_SUCCESS {
        break 'done;
    }

    dwLength = core::mem::size_of::<DWORD>() as DWORD;
    dwError = RegQueryValueExW(
        hProfileKey,
        REG_REF_COUNT.as_ptr() as LPCWSTR,
        core::ptr::null_mut(),
        &mut dwType,
        &mut dwRefCount as *mut DWORD as LPBYTE,
        &mut dwLength,
    );
    if dwError != ERROR_SUCCESS {
        break 'done;
    }

    dwRefCount = dwRefCount.wrapping_sub(1);

    dwLength = core::mem::size_of::<DWORD>() as DWORD;
    dwError = RegSetValueExW(
        hProfileKey,
        REG_REF_COUNT.as_ptr() as LPCWSTR,
        0,
        REG_DWORD,
        &mut dwRefCount as *mut DWORD as PBYTE,
        dwLength,
    ) as u32;
    if dwError != ERROR_SUCCESS {
        break 'done;
    }

    if !pdwRefCount.is_null() {
        *pdwRefCount = dwRefCount;
    }

    }

    #[allow(unreachable_code)]
    {
        if !hProfileKey.is_null() {
            RegCloseKey(hProfileKey);
        }

        if !hProfilesKey.is_null() {
            RegCloseKey(hProfilesKey);
        }

        return dwError;
    }
}
unsafe extern "C" fn CheckForGuestsAndAdmins(mut hToken: HANDLE, mut pdwState: PDWORD) -> DWORD {
    let mut pGroupInfo: PTOKEN_GROUPS = core::ptr::null_mut();
    let mut pAdministratorsSID: PSID = core::ptr::null_mut();
    let mut pGuestsSID: PSID = core::ptr::null_mut();
    let mut i: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut dwError: DWORD = ERROR_SUCCESS;

    if GetTokenInformation(hToken, TokenGroups, core::ptr::null_mut(), dwSize, &mut dwSize) == 0 {
        dwError = GetLastError();
        if dwError != ERROR_INSUFFICIENT_BUFFER {
            return dwError;
        }
        dwError = ERROR_SUCCESS;
    }

    pGroupInfo = HeapAlloc(GetProcessHeap(), 0, dwSize as usize) as PTOKEN_GROUPS;
    if pGroupInfo.is_null() {
        dwError = ERROR_OUTOFMEMORY;
        return dwError;
    }

    if GetTokenInformation(hToken, TokenGroups, pGroupInfo as PVOID, dwSize, &mut dwSize) == 0 {
        dwError = GetLastError();
        HeapFree(GetProcessHeap(), 0, pGroupInfo as PVOID);
        return dwError;
    }

    if AllocateAndInitializeSid(
        &mut LocalSystemAuthority,
        2,
        SECURITY_BUILTIN_DOMAIN_RID,
        DOMAIN_ALIAS_RID_ADMINS,
        0,
        0,
        0,
        0,
        0,
        0,
        &mut pAdministratorsSID,
    ) == 0
    {
        dwError = GetLastError();
        HeapFree(GetProcessHeap(), 0, pGroupInfo as PVOID);
        return dwError;
    }

    if AllocateAndInitializeSid(
        &mut LocalSystemAuthority,
        2,
        SECURITY_BUILTIN_DOMAIN_RID,
        DOMAIN_ALIAS_RID_GUESTS,
        0,
        0,
        0,
        0,
        0,
        0,
        &mut pGuestsSID,
    ) == 0
    {
        dwError = GetLastError();
        FreeSid(pAdministratorsSID);
        HeapFree(GetProcessHeap(), 0, pGroupInfo as PVOID);
        return dwError;
    }

    i = 0;
    while i < (*pGroupInfo).GroupCount {
        if EqualSid(pAdministratorsSID, (*(&mut (*pGroupInfo).Groups as *mut SID_AND_ATTRIBUTES).offset(i as isize)).Sid) != 0 {
            *pdwState |= 0x0100;
        }

        if EqualSid(pGuestsSID, (*(&mut (*pGroupInfo).Groups as *mut SID_AND_ATTRIBUTES).offset(i as isize)).Sid) != 0 {
            *pdwState |= 0x0080;
        }

        i = i.wrapping_add(1);
    }

    dwError = ERROR_SUCCESS;

    FreeSid(pGuestsSID);
    FreeSid(pAdministratorsSID);
    HeapFree(GetProcessHeap(), 0, pGroupInfo as PVOID);

    dwError
}
unsafe extern "C" fn SetProfileData(
    mut pszSidString: PWSTR,
    mut dwFlags: DWORD,
    mut hToken: HANDLE,
) -> DWORD {
    let mut hProfilesKey: HKEY = core::ptr::null_mut();
    let mut hProfileKey: HKEY = core::ptr::null_mut();
    let mut LoadTime: FILETIME = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let mut dwLength: DWORD = 0;
    let mut dwState: DWORD = 0;
    let mut dwError: DWORD = 0;

    'done: {
    dwError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        PROFILE_LIST_KEY3.as_ptr() as LPCWSTR,
        0,
        KEY_QUERY_VALUE,
        core::ptr::addr_of_mut!(hProfilesKey),
    );
    if dwError != ERROR_SUCCESS {
        break 'done;
    }

    dwError = RegOpenKeyExW(
        hProfilesKey,
        pszSidString as LPCWSTR,
        0,
        KEY_QUERY_VALUE | KEY_SET_VALUE,
        core::ptr::addr_of_mut!(hProfileKey),
    );
    if dwError != ERROR_SUCCESS {
        break 'done;
    }

    GetSystemTimeAsFileTime(core::ptr::addr_of_mut!(LoadTime));

    dwLength = core::mem::size_of::<DWORD>() as DWORD;
    dwError = RegSetValueExW(
        hProfileKey,
        REG_PROFILE_LOAD_TIME_LOW.as_ptr() as LPCWSTR,
        0,
        REG_DWORD,
        core::ptr::addr_of_mut!(LoadTime.dwLowDateTime) as PBYTE,
        dwLength,
    ) as u32;
    if dwError != ERROR_SUCCESS {
        break 'done;
    }

    dwLength = core::mem::size_of::<DWORD>() as DWORD;
    dwError = RegSetValueExW(
        hProfileKey,
        REG_PROFILE_LOAD_TIME_HIGH.as_ptr() as LPCWSTR,
        0,
        REG_DWORD,
        core::ptr::addr_of_mut!(LoadTime.dwHighDateTime) as PBYTE,
        dwLength,
    ) as u32;
    if dwError != ERROR_SUCCESS {
        break 'done;
    }

    dwLength = core::mem::size_of::<DWORD>() as DWORD;
    dwError = RegSetValueExW(
        hProfileKey,
        REG_FLAGS.as_ptr() as LPCWSTR,
        0,
        REG_DWORD,
        core::ptr::addr_of_mut!(dwFlags) as PBYTE,
        dwLength,
    ) as u32;
    if dwError != ERROR_SUCCESS {
        break 'done;
    }

    dwError = CheckForGuestsAndAdmins(hToken, core::ptr::addr_of_mut!(dwState));
    if dwError != ERROR_SUCCESS {
        break 'done;
    }

    dwLength = core::mem::size_of::<DWORD>() as DWORD;
    dwError = RegSetValueExW(
        hProfileKey,
        REG_STATE.as_ptr() as LPCWSTR,
        0,
        REG_DWORD,
        core::ptr::addr_of_mut!(dwState) as PBYTE,
        dwLength,
    ) as u32;
    if dwError != ERROR_SUCCESS {
        break 'done;
    }

    }
    if !hProfileKey.is_null() {
        RegCloseKey(hProfileKey);
    }

    if !hProfilesKey.is_null() {
        RegCloseKey(hProfilesKey);
    }

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn CopySystemProfile(_Unused: u32) -> BOOL {
    let mut szKeyName: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szRawProfilePath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szProfilePath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szDefaultProfilePath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut SidString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut hToken: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut pUserSid: PSID = core::ptr::null_mut();
    let mut hProfileKey: HKEY = core::ptr::null_mut();
    let mut dwDisposition: u32 = 0;
    let mut bResult: BOOL = 0;
    let mut cchSize: u32 = 0;
    let mut dwError: u32 = 0;

    if OpenProcessToken(
        GetCurrentProcess(),
        TOKEN_QUERY | TOKEN_IMPERSONATE,
        &mut hToken,
    ) == 0
    {
        return 0;
    }

    'done: {
    pUserSid = GetUserSid(hToken);
    if pUserSid.is_null() {
        DPRINT1!("CopySystemProfile: GetUserSid", GetLastError());
        break 'done;
    }

    if GetUserSidStringFromToken(hToken, &mut SidString) == 0 {
        DPRINT1!("CopySystemProfile: GetUserSidStringFromToken", GetLastError());
        break 'done;
    }

    StringCbCopyW(
        &mut szKeyName[0],
        core::mem::size_of_val(&szKeyName),
        PROFILE_LIST_KEY_TRAILING_BS.as_ptr() as LPCWSTR,
    );
    StringCbCatW(
        &mut szKeyName[0],
        core::mem::size_of_val(&szKeyName),
        SidString.Buffer,
    );

    RtlFreeUnicodeString(&mut SidString);

    dwError = RegCreateKeyExW(
        0x80000002 as HKEY,
        &szKeyName[0],
        0,
        core::ptr::null_mut(),
        0,
        KEY_WRITE,
        core::ptr::null_mut(),
        &mut hProfileKey,
        &mut dwDisposition,
    ) as u32;
    if dwError != ERROR_SUCCESS {
        DPRINT1!("CopySystemProfile: RegCreateKeyExW(ProfileList\\<sid>)", dwError);
        break 'done;
    }

    dwError = RegSetValueExW(
        hProfileKey,
        REG_SID.as_ptr() as LPCWSTR,
        0,
        REG_BINARY,
        pUserSid as PBYTE,
        RtlLengthSid(pUserSid),
    ) as u32;
    if dwError != ERROR_SUCCESS {
        DPRINT1!("CopySystemProfile: RegSetValueExW(Sid)", dwError);
        break 'done;
    }

    // L"%systemroot%\system32\config\systemprofile"
    static SYSTEMPROFILE_PATH_W: [u16; 43] = [
        b'%' as u16, b's' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16, b'r' as u16, b'o' as u16, b'o' as u16,
        b't' as u16, b'%' as u16, b'\\' as u16, b's' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16, b'3' as u16,
        b'2' as u16, b'\\' as u16, b'c' as u16, b'o' as u16, b'n' as u16, b'f' as u16, b'i' as u16, b'g' as u16, b'\\' as u16, b's' as u16,
        b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16, b'p' as u16, b'r' as u16, b'o' as u16, b'f' as u16, b'i' as u16,
        b'l' as u16, b'e' as u16, 0,
    ];
    wcscpy(
        &mut szRawProfilePath[0],
        SYSTEMPROFILE_PATH_W.as_ptr() as *const u16,
    );

    dwError = RegSetValueExW(
        hProfileKey,
        REG_PROFILE_IMAGE_PATH.as_ptr() as LPCWSTR,
        0,
        REG_EXPAND_SZ,
        &szRawProfilePath[0] as *const u16 as PBYTE,
        ((wcslen(&szRawProfilePath[0]) + 1) * core::mem::size_of::<u16>()) as u32,
    ) as u32;
    if dwError != ERROR_SUCCESS {
        DPRINT1!("CopySystemProfile: RegSetValueExW(ProfileImagePath)", dwError);
        break 'done;
    }

    if ExpandEnvironmentStringsW(
        &szRawProfilePath[0],
        &mut szProfilePath[0],
        szRawProfilePath.len() as u32, // C: ARRAYSIZE(szRawProfilePath)
    ) == 0
    {
        DPRINT1!("CopySystemProfile: ExpandEnvironmentStringsW", GetLastError());
        break 'done;
    }

    if CreateDirectoryW(&szProfilePath[0], core::ptr::null_mut()) == 0 {
        if GetLastError() != ERROR_ALREADY_EXISTS {
            DPRINT1!("CopySystemProfile: CreateDirectoryW(systemprofile)", GetLastError());
        break 'done;
        }
    }

    cchSize = szDefaultProfilePath.len() as u32; // C: ARRAYSIZE(szDefaultProfilePath)
    if GetDefaultUserProfileDirectoryW(&mut szDefaultProfilePath[0], &mut cchSize) == 0 {
        DPRINT1!("CopySystemProfile: GetDefaultUserProfileDirectoryW", GetLastError());
        break 'done;
    }

    if CopyDirectory(&szProfilePath[0], &szDefaultProfilePath[0]) == 0 {
        DPRINT1!("CopySystemProfile: CopyDirectory(Default User -> systemprofile)", GetLastError());
        break 'done;
    }

    bResult = 1;

    }
    if !hProfileKey.is_null() {
        RegCloseKey(hProfileKey);
    }

    RtlFreeUnicodeString(&mut SidString);

    if !pUserSid.is_null() {
        LocalFree(pUserSid as HLOCAL);
    }

    if !hToken.is_null() {
        CloseHandle(hToken);
    }

    return bResult;
}


#[no_mangle]
pub unsafe extern "system" fn CreateUserProfileA(pSid: PSID, lpUserName: LPCSTR) -> BOOL {
    let mut pUserNameW: LPWSTR = core::ptr::null_mut();
    let mut nLength: i32;
    let mut bResult: BOOL;

    nLength = MultiByteToWideChar(
        0x00000000,
        0,
        lpUserName,
        -1,
        core::ptr::null_mut(),
        0,
    );
    pUserNameW = HeapAlloc(
        GetProcessHeap(),
        0,
        (nLength as usize) * core::mem::size_of::<u16>(),
    ) as LPWSTR;
    if pUserNameW.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }
    MultiByteToWideChar(
        0x00000000,
        0,
        lpUserName,
        -1,
        pUserNameW,
        nLength,
    );

    bResult = CreateUserProfileExW(
        pSid,
        pUserNameW,
        core::ptr::null(),
        core::ptr::null_mut(),
        0,
        0,
    );

    HeapFree(GetProcessHeap(), 0, pUserNameW as LPVOID);

    bResult
}

#[no_mangle]
pub unsafe extern "system" fn CreateUserProfileW(
    pSid: PSID,
    lpUserName: LPCWSTR,
) -> BOOL {
    CreateUserProfileExW(
        pSid,
        lpUserName,
        core::ptr::null(),
        core::ptr::null_mut(),
        0u32,
        0,
    )
}

#[no_mangle]
pub unsafe extern "system" fn CreateUserProfileExA(
    pSid: PSID,
    lpUserName: LPCSTR,
    lpUserHive: LPCSTR,
    lpProfileDir: LPSTR,
    dwDirSize: DWORD,
    bWin9xUpg: BOOL,
) -> BOOL {
    let mut pUserNameW: LPWSTR = core::ptr::null_mut();
    let mut pUserHiveW: LPWSTR = core::ptr::null_mut();
    let mut pProfileDirW: LPWSTR = core::ptr::null_mut();
    let mut nLength: INT = 0;
    let mut bResult: BOOL = 0;

    if !lpProfileDir.is_null() && dwDirSize == 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    nLength = MultiByteToWideChar(CP_ACP as UINT, 0, lpUserName, -1, core::ptr::null_mut(), 0);
    pUserNameW = HeapAlloc(GetProcessHeap(), 0, (nLength as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR;
    if pUserNameW.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }
    MultiByteToWideChar(CP_ACP as UINT, 0, lpUserName, -1, pUserNameW, nLength);

    if !lpUserHive.is_null() {
        nLength = MultiByteToWideChar(CP_ACP as UINT, 0, lpUserHive, -1, core::ptr::null_mut(), 0);
        pUserHiveW = HeapAlloc(GetProcessHeap(), 0, (nLength as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR;
        if pUserHiveW.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            HeapFree(GetProcessHeap(), 0, pUserNameW as LPVOID);
            return 0;
        }
        MultiByteToWideChar(CP_ACP as UINT, 0, lpUserHive, -1, pUserHiveW, nLength);
    }

    if !lpProfileDir.is_null() {
        pProfileDirW = HeapAlloc(GetProcessHeap(), 0, (dwDirSize as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR;
        if pProfileDirW.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            if !pUserHiveW.is_null() {
                HeapFree(GetProcessHeap(), 0, pUserHiveW as LPVOID);
            }
            HeapFree(GetProcessHeap(), 0, pUserNameW as LPVOID);
            return 0;
        }
    }

    bResult = CreateUserProfileExW(pSid, pUserNameW as LPCWSTR, pUserHiveW as LPCWSTR, pProfileDirW, dwDirSize, bWin9xUpg);

    if bResult != 0 && !lpProfileDir.is_null() {
        WideCharToMultiByte(CP_ACP as UINT, 0, pProfileDirW as LPCWSTR, -1, lpProfileDir, dwDirSize as INT, core::ptr::null(), core::ptr::null_mut());
    }

    if !pProfileDirW.is_null() {
        HeapFree(GetProcessHeap(), 0, pProfileDirW as LPVOID);
    }

    if !pUserHiveW.is_null() {
        HeapFree(GetProcessHeap(), 0, pUserHiveW as LPVOID);
    }

    if !pUserNameW.is_null() {
        HeapFree(GetProcessHeap(), 0, pUserNameW as LPVOID);
    }

    bResult
}

#[no_mangle]
pub unsafe extern "system" fn CreateUserProfileExW(
    mut pSid: PSID,
    mut lpUserName: LPCWSTR,
    mut lpUserHive: LPCWSTR,
    mut lpProfileDir: LPWSTR,
    mut dwDirSize: DWORD,
    mut bWin9xUpg: BOOL,
) -> BOOL {
    let mut szRawProfilesPath: [u16; 260] = [0; 260];
    let mut szProfilesPath: [u16; 260] = [0; 260];
    let mut szUserProfilePath: [u16; 260] = [0; 260];
    let mut szDefaultUserPath: [u16; 260] = [0; 260];
    let mut szUserProfileName: [u16; 260] = [0; 260];
    let mut szBuffer: [u16; 260] = [0; 260];
    let mut SidString: LPWSTR = core::ptr::null_mut();
    let mut dwType: DWORD = 0;
    let mut dwLength: DWORD = 0;
    let mut dwDisposition: DWORD = 0;
    let mut i: u32 = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut bRet: BOOL = 1;
    let mut Error: i32 = 0;

    if pSid.is_null() || lpUserName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    Error = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        PROFILE_LIST_KEY3.as_ptr() as LPCWSTR,
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    ) as i32;
    if Error != ERROR_SUCCESS as i32 {
        SetLastError(Error as DWORD);
        return 0;
    }

    dwLength = core::mem::size_of::<[u16; 260]>() as DWORD;
    Error = RegQueryValueExW(
        hKey,
        REG_PROFILES_DIRECTORY2.as_ptr() as LPCWSTR,
        core::ptr::null_mut(),
        &mut dwType,
        szRawProfilesPath.as_mut_ptr() as LPBYTE,
        &mut dwLength,
    ) as i32;
    if Error != ERROR_SUCCESS as i32 || (dwType != REG_SZ && dwType != REG_EXPAND_SZ) {
        RegCloseKey(hKey);
        SetLastError(Error as DWORD);
        return 0;
    }

    if ExpandEnvironmentStringsW(
        szRawProfilesPath.as_mut_ptr(),
        szProfilesPath.as_mut_ptr(),
        (core::mem::size_of::<[u16; 260]>() / core::mem::size_of::<u16>()) as DWORD,
    ) == 0
    {
        RegCloseKey(hKey);
        return 0;
    }

    if CreateDirectoryW(szProfilesPath.as_mut_ptr(), core::ptr::null_mut()) == 0 {
        if GetLastError() != ERROR_ALREADY_EXISTS {
            return 0;
        }
    }

    dwLength = core::mem::size_of::<[u16; 260]>() as DWORD;
    Error = RegQueryValueExW(
        hKey,
        REG_DEFAULT_USER_PROFILE.as_ptr() as LPCWSTR,
        core::ptr::null_mut(),
        &mut dwType,
        szBuffer.as_mut_ptr() as LPBYTE,
        &mut dwLength,
    ) as i32;
    if Error != ERROR_SUCCESS as i32 || (dwType != REG_SZ && dwType != REG_EXPAND_SZ) {
        RegCloseKey(hKey);
        SetLastError(Error as DWORD);
        return 0;
    }

    RegCloseKey(hKey);

    StringCbCopyW(
        szUserProfileName.as_mut_ptr() as LPWSTR,
        core::mem::size_of::<[u16; 260]>(),
        lpUserName as LPCWSTR,
    );

    StringCbCopyW(
        szUserProfilePath.as_mut_ptr() as LPWSTR,
        core::mem::size_of::<[u16; 260]>(),
        szProfilesPath.as_mut_ptr() as LPCWSTR,
    );
    StringCbCatW(
        szUserProfilePath.as_mut_ptr() as LPWSTR,
        core::mem::size_of::<[u16; 260]>(),
        PATH_SEP_BACKSLASH2.as_ptr() as LPCWSTR,
    );
    StringCbCatW(
        szUserProfilePath.as_mut_ptr() as LPWSTR,
        core::mem::size_of::<[u16; 260]>(),
        szUserProfileName.as_mut_ptr() as LPCWSTR,
    );

    if CreateDirectoryW(szUserProfilePath.as_mut_ptr(), core::ptr::null_mut()) == 0 {
        if GetLastError() != ERROR_ALREADY_EXISTS {
            return 0;
        }

        i = 0;
        while i < 1000 {
            // L"%s.%03u"
            static FMT_USER_PROFILE_SUFFIX: [u16; 8] = [
                b'%' as u16, b's' as u16, b'.' as u16, b'%' as u16, b'0' as u16, b'3' as u16, b'u' as u16, 0,
            ];
            swprintf(
                szUserProfileName.as_mut_ptr(),
                FMT_USER_PROFILE_SUFFIX.as_ptr() as *const u16,
                lpUserName,
                i,
            );

            StringCbCopyW(
                szUserProfilePath.as_mut_ptr() as LPWSTR,
                core::mem::size_of::<[u16; 260]>(),
                szProfilesPath.as_mut_ptr() as LPCWSTR,
            );
            StringCbCatW(
                szUserProfilePath.as_mut_ptr() as LPWSTR,
                core::mem::size_of::<[u16; 260]>(),
                PATH_SEP_BACKSLASH2.as_ptr() as LPCWSTR,
            );
            StringCbCatW(
                szUserProfilePath.as_mut_ptr() as LPWSTR,
                core::mem::size_of::<[u16; 260]>(),
                szUserProfileName.as_mut_ptr() as LPCWSTR,
            );

            if CreateDirectoryW(szUserProfilePath.as_mut_ptr(), core::ptr::null_mut()) != 0 {
                break;
            }

            if GetLastError() != ERROR_ALREADY_EXISTS {
                return 0;
            }

            i = i.wrapping_add(1);
        }
    }

    StringCbCopyW(
        szDefaultUserPath.as_mut_ptr() as LPWSTR,
        core::mem::size_of::<[u16; 260]>(),
        szProfilesPath.as_mut_ptr() as LPCWSTR,
    );
    StringCbCatW(
        szDefaultUserPath.as_mut_ptr() as LPWSTR,
        core::mem::size_of::<[u16; 260]>(),
        PATH_SEP_BACKSLASH2.as_ptr() as LPCWSTR,
    );
    StringCbCatW(
        szDefaultUserPath.as_mut_ptr() as LPWSTR,
        core::mem::size_of::<[u16; 260]>(),
        szBuffer.as_mut_ptr() as LPCWSTR,
    );

    if CopyDirectory(
        szUserProfilePath.as_mut_ptr(),
        szDefaultUserPath.as_mut_ptr(),
    ) == 0
    {
        return 0;
    }

    if ConvertSidToStringSidW(pSid, &mut SidString) == 0 {
        return 0;
    }

    StringCbCopyW(
        szBuffer.as_mut_ptr() as LPWSTR,
        core::mem::size_of::<[u16; 260]>(),
        PROFILE_LIST_KEY_TRAILING_BS.as_ptr() as LPCWSTR,
    );
    StringCbCatW(
        szBuffer.as_mut_ptr() as LPWSTR,
        core::mem::size_of::<[u16; 260]>(),
        SidString as LPCWSTR,
    );

    'done: {
    Error = RegCreateKeyExW(
        HKEY_LOCAL_MACHINE,
        szBuffer.as_mut_ptr(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_ALL_ACCESS,
        core::ptr::null_mut(),
        &mut hKey,
        &mut dwDisposition,
    );
    if Error != ERROR_SUCCESS as i32 {
        bRet = 0;
        break 'done;
    } else {
        StringCbCopyW(
            szBuffer.as_mut_ptr() as LPWSTR,
            core::mem::size_of::<[u16; 260]>(),
            szRawProfilesPath.as_mut_ptr() as LPCWSTR,
        );
        StringCbCatW(
            szBuffer.as_mut_ptr() as LPWSTR,
            core::mem::size_of::<[u16; 260]>(),
            PATH_SEP_BACKSLASH2.as_ptr() as LPCWSTR,
        );
        StringCbCatW(
            szBuffer.as_mut_ptr() as LPWSTR,
            core::mem::size_of::<[u16; 260]>(),
            szUserProfileName.as_mut_ptr() as LPCWSTR,
        );

        Error = RegSetValueExW(
            hKey,
            REG_PROFILE_IMAGE_PATH.as_ptr() as LPCWSTR,
            0,
            REG_EXPAND_SZ,
            szBuffer.as_mut_ptr() as *const BYTE,
            ((wcslen(szBuffer.as_mut_ptr()) + 1) * core::mem::size_of::<u16>()) as DWORD,
        );
        if Error != ERROR_SUCCESS as i32 {
            RegCloseKey(hKey);
            bRet = 0;
            break 'done;
        } else {
            Error = RegSetValueExW(
                hKey,
                REG_SID.as_ptr() as LPCWSTR,
                0,
                REG_BINARY,
                pSid as *const BYTE,
                GetLengthSid(pSid),
            );
            if Error != ERROR_SUCCESS as i32 {
                RegCloseKey(hKey);
                bRet = 0;
                break 'done;
            } else {
                RegCloseKey(hKey);

                StringCbCopyW(
                    szBuffer.as_mut_ptr() as LPWSTR,
                    core::mem::size_of::<[u16; 260]>(),
                    szUserProfilePath.as_mut_ptr() as LPCWSTR,
                );
                StringCbCatW(
                    szBuffer.as_mut_ptr() as LPWSTR,
                    core::mem::size_of::<[u16; 260]>(),
                    NTUSER_DAT_SUFFIX.as_ptr() as LPCWSTR,
                );

                if AcquireRemoveRestorePrivilege(1) == 0 {
                    Error = GetLastError() as i32;
                    bRet = 0;
                    break 'done;
                }

                Error = RegLoadKeyW(HKEY_USERS, SidString, szBuffer.as_mut_ptr());
                AcquireRemoveRestorePrivilege(0);
                if Error != ERROR_SUCCESS as i32 {
                    bRet = 0;
                    break 'done;
                }

                if CreateUserHive(SidString, szUserProfilePath.as_mut_ptr()) == 0 {
                    Error = GetLastError() as i32;
                    bRet = 0;
                }

                AcquireRemoveRestorePrivilege(1);
                RegUnLoadKeyW(HKEY_USERS, SidString);
                AcquireRemoveRestorePrivilege(0);

                if !lpProfileDir.is_null() && dwDirSize != 0 {
                    StringCchCopyW(lpProfileDir, dwDirSize as usize, szUserProfilePath.as_mut_ptr());
                }
            }
        }
    }

    }


    LocalFree(SidString as HLOCAL);
    SetLastError(Error as DWORD);

    bRet
}

#[no_mangle]
pub unsafe extern "system" fn DeleteProfileA(
    mut lpSidString: LPCSTR,
    mut lpProfilePath: LPCSTR,
    mut lpComputerName: LPCSTR,
) -> BOOL {
    let mut bResult: BOOL = 0;
    let mut SidString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0 as USHORT,
        MaximumLength: 0 as USHORT,
        Buffer: core::ptr::null_mut::<WCHAR>(),
    };
    let mut ProfilePath: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0 as USHORT,
        MaximumLength: 0 as USHORT,
        Buffer: core::ptr::null_mut::<WCHAR>(),
    };
    let mut ComputerName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0 as USHORT,
        MaximumLength: 0 as USHORT,
        Buffer: core::ptr::null_mut::<WCHAR>(),
    };
    if !lpSidString.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&mut SidString, lpSidString as LPSTR as PCSZ);
    }
    if !lpProfilePath.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&mut ProfilePath, lpProfilePath as LPSTR as PCSZ);
    }
    if !lpComputerName.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&mut ComputerName, lpComputerName as LPSTR as PCSZ);
    }
    bResult = DeleteProfileW(
        SidString.Buffer as LPCWSTR,
        ProfilePath.Buffer as LPCWSTR,
        ComputerName.Buffer as LPCWSTR,
    );
    if !lpSidString.is_null() {
        RtlFreeUnicodeString(&mut SidString);
    }
    if !lpProfilePath.is_null() {
        RtlFreeUnicodeString(&mut ProfilePath);
    }
    if !lpComputerName.is_null() {
        RtlFreeUnicodeString(&mut ComputerName);
    }
    bResult
}

macro_rules! goto_done {
    ($dwError:expr, $hProfilesKey:expr, $hProfileKey:expr, $bRet:expr) => {
        SetLastError($dwError);
        if !$hProfileKey.is_null() {
            RegCloseKey($hProfileKey);
        }
        if !$hProfilesKey.is_null() {
            RegCloseKey($hProfilesKey);
        }
        $bRet = 0;
    };
}

#[no_mangle]
pub unsafe extern "system" fn DeleteProfileW(
    lpSidString: LPCWSTR,
    lpProfilePath: LPCWSTR,
    lpComputerName: LPCWSTR,
) -> i32 {
    let mut hProfilesKey: HKEY = core::ptr::null_mut();
    let mut hProfileKey: HKEY = core::ptr::null_mut();
    let mut szRawProfilePath: [u16; 260] = [0; 260];
    let mut szProfilePath: [u16; 260] = [0; 260];
    let mut dwLength: DWORD;
    let mut dwType: DWORD = 0;
    let mut dwError: DWORD = ERROR_SUCCESS;
    let mut bRet: i32 = 0;

    if lpSidString.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if !lpProfilePath.is_null() {
        dwError = RegOpenKeyExW(
            core::ptr::null_mut(),
            PROFILE_LIST_KEY3.as_ptr() as LPCWSTR,
            0,
            KEY_QUERY_VALUE,
            &mut hProfilesKey,
        );
        if dwError != ERROR_SUCCESS {
            goto_done!(dwError, hProfilesKey, hProfileKey, bRet);
            return bRet;
        }

        dwError = RegOpenKeyExW(
            core::ptr::null_mut(),
            lpSidString,
            0,
            KEY_QUERY_VALUE,
            &mut hProfileKey,
        );
        if dwError != ERROR_SUCCESS {
            goto_done!(dwError, hProfilesKey, hProfileKey, bRet);
            return bRet;
        }

        dwLength = core::mem::size_of::<[u16; 260]>() as DWORD;
        dwError = RegQueryValueExW(
            hProfileKey,
            REG_PROFILE_IMAGE_PATH.as_ptr() as LPCWSTR,
            core::ptr::null_mut(),
            &mut dwType,
            szRawProfilePath.as_mut_ptr() as LPBYTE,
            &mut dwLength,
        );
        if dwError != ERROR_SUCCESS {
            goto_done!(dwError, hProfilesKey, hProfileKey, bRet);
            return bRet;
        }

        if ExpandEnvironmentStringsW(
            szRawProfilePath.as_ptr(),
            szProfilePath.as_mut_ptr(),
            260,
        ) == 0
        {
            dwError = GetLastError();
            goto_done!(dwError, hProfilesKey, hProfileKey, bRet);
            return bRet;
        }
    } else {
        StringCchCopyW(
            szProfilePath.as_mut_ptr(),
            260,
            lpProfilePath,
        );
    }

    if RemoveDirectoryPath(szProfilePath.as_ptr()) == 0 {
        dwError = GetLastError();
        goto_done!(dwError, hProfilesKey, hProfileKey, bRet);
        return bRet;
    }

    bRet = 1;

    if !hProfileKey.is_null() {
        RegCloseKey(hProfileKey);
    }

    if !hProfilesKey.is_null() {
        RegCloseKey(hProfilesKey);
    }

    bRet
}

#[no_mangle]
pub unsafe extern "system" fn GetAllUsersProfileDirectoryA(
    mut lpProfileDir: LPSTR,
    mut lpcchSize: LPDWORD,
) -> BOOL {
    let mut lpBuffer: LPWSTR;
    let mut bResult: BOOL;

    if lpcchSize.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    lpBuffer = GlobalAlloc(
        GMEM_FIXED,
        (*lpcchSize as usize).wrapping_mul(core::mem::size_of::<u16>()),
    ) as LPWSTR;
    if lpBuffer.is_null() {
        return 0;
    }

    bResult = GetAllUsersProfileDirectoryW(lpBuffer, lpcchSize);
    if bResult != 0 && !lpProfileDir.is_null() {
        bResult = WideCharToMultiByte(
            0u32,
            0u32,
            lpBuffer,
            -1i32,
            lpProfileDir,
            *lpcchSize as i32,
            core::ptr::null(),
            core::ptr::null_mut(),
        ) as BOOL;
    }

    GlobalFree(lpBuffer as HLOCAL);

    return bResult;
}

#[no_mangle]
pub unsafe extern "system" fn GetAllUsersProfileDirectoryW(
    lpProfileDir: LPWSTR,
    lpcchSize: LPDWORD,
) -> BOOL {
    let mut szProfilePath: [u16; 260] = [0; 260];
    let mut szBuffer: [u16; 260] = [0; 260];
    let mut dwType: u32 = 0;
    let mut dwLength: u32 = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut Error: i32 = 0;

    if lpcchSize.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    Error = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        win32macros_rust::wide_bytes!(b"S\0O\0F\0T\0W\0A\0R\0E\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0W\0i\0n\0d\0o\0w\0s\0 \0N\0T\0\\\0C\0u\0r\0r\0e\0n\0t\0V\0e\0r\0s\0i\0o\0n\0\\\0P\0r\0o\0f\0i\0l\0e\0L\0i\0s\0t\0\0\0") as LPCWSTR,
        0,
        KEY_QUERY_VALUE,
        core::ptr::addr_of_mut!(hKey),
    ) as i32;
    if Error != ERROR_SUCCESS as i32 {
        SetLastError(Error as u32);
        return 0;
    }

    dwLength = core::mem::size_of::<[u16; 260]>() as u32;
    Error = RegQueryValueExW(
        hKey,
        win32macros_rust::wide_bytes!(b"P\0r\0o\0f\0i\0l\0e\0s\0D\0i\0r\0e\0c\0t\0o\0r\0y\0\0\0") as LPCWSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        szBuffer.as_mut_ptr() as LPBYTE,
        core::ptr::addr_of_mut!(dwLength),
    ) as i32;
    if (Error != ERROR_SUCCESS as i32) || (dwType != REG_SZ && dwType != REG_EXPAND_SZ) {
        RegCloseKey(hKey);
        SetLastError(Error as u32);
        return 0;
    }

    if ExpandEnvironmentStringsW(
        szBuffer.as_ptr(),
        szProfilePath.as_mut_ptr(),
        (core::mem::size_of::<[u16; 260]>() / core::mem::size_of::<u16>()) as u32,
    ) == 0
    {
        RegCloseKey(hKey);
        return 0;
    }

    dwLength = core::mem::size_of::<[u16; 260]>() as u32;
    Error = RegQueryValueExW(
        hKey,
        win32macros_rust::wide_bytes!(b"A\0l\0l\0U\0s\0e\0r\0s\0P\0r\0o\0f\0i\0l\0e\0\0\0") as LPCWSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        szBuffer.as_mut_ptr() as LPBYTE,
        core::ptr::addr_of_mut!(dwLength),
    ) as i32;
    if (Error != ERROR_SUCCESS as i32) || (dwType != REG_SZ && dwType != REG_EXPAND_SZ) {
        RegCloseKey(hKey);
        SetLastError(Error as u32);
        return 0;
    }

    RegCloseKey(hKey);

    StringCbCatW(szProfilePath.as_mut_ptr(), core::mem::size_of::<[u16; 260]>(), win32macros_rust::wide_bytes!(b"\\\0\0\0") as LPCWSTR);
    StringCbCatW(szProfilePath.as_mut_ptr(), core::mem::size_of::<[u16; 260]>(), szBuffer.as_ptr());

    dwLength = (wcslen(szProfilePath.as_ptr()) + 1) as u32;
    if !lpProfileDir.is_null() && *lpcchSize >= dwLength {
        StringCchCopyW(lpProfileDir, *lpcchSize as usize, szProfilePath.as_ptr());
        *lpcchSize = dwLength;
        return 1;
    } else {
        *lpcchSize = dwLength;
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        return 0;
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetDefaultUserProfileDirectoryA(
    mut lpProfileDir: LPSTR,
    mut lpcchSize: LPDWORD,
) -> i32 {
    let mut lpBuffer: LPWSTR;
    let mut bResult: i32;

    if lpcchSize.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    lpBuffer = GlobalAlloc(
        GMEM_FIXED as u32,
        (*lpcchSize as usize).wrapping_mul(core::mem::size_of::<u16>()),
    ) as LPWSTR;
    if lpBuffer.is_null() {
        return 0;
    }

    bResult = GetDefaultUserProfileDirectoryW(lpBuffer, lpcchSize) as BOOL;
    if bResult != 0 && !lpProfileDir.is_null() {
        bResult = WideCharToMultiByte(
            0u32,
            0u32,
            lpBuffer,
            -1i32,
            lpProfileDir,
            *lpcchSize as i32,
            core::ptr::null(),
            core::ptr::null_mut(),
        ) as i32;
    }

    GlobalFree(lpBuffer as *mut core::ffi::c_void);

    bResult
}

#[no_mangle]
pub unsafe extern "system" fn GetDefaultUserProfileDirectoryW(
    lpProfileDir: LPWSTR,
    lpcchSize: LPDWORD,
) -> BOOL {
    let mut szProfilePath: [u16; 260] = [0; 260];
    let mut szBuffer: [u16; 260] = [0; 260];
    let mut dwType: u32 = 0;
    let mut dwLength: u32 = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut Error: i32 = 0;

    if lpcchSize.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    Error = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE as HKEY,
        win32macros_rust::wide_bytes!(b"S\0O\0F\0T\0W\0A\0R\0E\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0W\0i\0n\0d\0o\0w\0s\0 \0N\0T\0\\\0C\0u\0r\0r\0e\0n\0t\0V\0e\0r\0s\0i\0o\0n\0\\\0P\0r\0o\0f\0i\0l\0e\0L\0i\0s\0t\0\0\0") as LPCWSTR,
        0,
        KEY_QUERY_VALUE,
        core::ptr::addr_of_mut!(hKey),
    ) as i32;
    if Error != 0i32 {
        SetLastError(Error as u32);
        return 0;
    }

    dwLength = core::mem::size_of::<[u16; 260]>() as u32;
    Error = RegQueryValueExW(
        hKey,
        win32macros_rust::wide_bytes!(b"P\0r\0o\0f\0i\0l\0e\0s\0D\0i\0r\0e\0c\0t\0o\0r\0y\0\0\0") as LPCWSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(szBuffer) as *mut u16 as LPBYTE,
        core::ptr::addr_of_mut!(dwLength),
    ) as i32;
    if (Error != 0i32) || (dwType != REG_SZ && dwType != REG_EXPAND_SZ) {
        RegCloseKey(hKey);
        SetLastError(Error as u32);
        return 0;
    }

    if ExpandEnvironmentStringsW(
        core::ptr::addr_of_mut!(szBuffer) as *mut u16 as LPCWSTR,
        core::ptr::addr_of_mut!(szProfilePath) as LPWSTR,
        (core::mem::size_of::<[u16; 260]>() / core::mem::size_of::<u16>()) as u32,
    ) == 0 {
        RegCloseKey(hKey);
        return 0;
    }

    dwLength = core::mem::size_of::<[u16; 260]>() as u32;
    Error = RegQueryValueExW(
        hKey,
        win32macros_rust::wide_bytes!(b"D\0e\0f\0a\0u\0l\0t\0U\0s\0e\0r\0P\0r\0o\0f\0i\0l\0e\0\0\0") as LPCWSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(szBuffer) as *mut u16 as LPBYTE,
        core::ptr::addr_of_mut!(dwLength),
    ) as i32;
    if (Error != 0i32) || (dwType != REG_SZ && dwType != REG_EXPAND_SZ) {
        RegCloseKey(hKey);
        SetLastError(Error as u32);
        return 0;
    }

    RegCloseKey(hKey);

    StringCbCatW(
        core::ptr::addr_of_mut!(szProfilePath) as LPWSTR,
        core::mem::size_of::<[u16; 260]>(),
        win32macros_rust::wide_bytes!(b"\\\0\0\0") as LPCWSTR,
    );
    StringCbCatW(
        core::ptr::addr_of_mut!(szProfilePath) as LPWSTR,
        core::mem::size_of::<[u16; 260]>(),
        core::ptr::addr_of_mut!(szBuffer) as *mut u16 as LPCWSTR,
    );

    dwLength = (wcslen(core::ptr::addr_of_mut!(szProfilePath) as *mut u16 as LPCWSTR) + 1) as u32;
    if !lpProfileDir.is_null() && *lpcchSize >= dwLength {
        StringCchCopyW(
            lpProfileDir,
            *lpcchSize as usize,
            core::ptr::addr_of_mut!(szProfilePath) as *mut u16 as LPCWSTR,
        );
        *lpcchSize = dwLength;
        return 1;
    } else {
        *lpcchSize = dwLength;
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        return 0;
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetProfilesDirectoryA(
    mut lpProfileDir: LPSTR,
    mut lpcchSize: LPDWORD,
) -> BOOL {
    let mut lpBuffer: LPWSTR = core::ptr::null_mut();
    let mut bResult: BOOL = 0;
    if lpcchSize.is_null() || lpProfileDir.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    lpBuffer = GlobalAlloc(
        GMEM_FIXED,
        (*lpcchSize as usize).wrapping_mul(core::mem::size_of::<u16>()),
    ) as LPWSTR;
    if lpBuffer.is_null() {
        return FALSE;
    }
    bResult = GetProfilesDirectoryW(lpBuffer, lpcchSize);
    if bResult != 0 && !lpProfileDir.is_null() {
        bResult = WideCharToMultiByte(
            CP_ACP,
            0u32,
            lpBuffer,
            -1i32,
            lpProfileDir,
            *lpcchSize as i32,
            core::ptr::null(),
            core::ptr::null_mut(),
        ) as BOOL;
    }
    GlobalFree(lpBuffer as HGLOBAL);
    return bResult;
}

#[no_mangle]
pub unsafe extern "system" fn GetProfilesDirectoryW(
    lpProfilesDir: LPWSTR,
    lpcchSize: LPDWORD,
) -> BOOL {
    let mut szProfilesPath: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szBuffer: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut dwType: DWORD = 0;
    let mut dwLength: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut Error: LONG = 0;

    if lpcchSize.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    Error = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        PROFILE_LIST_KEY3.as_ptr() as LPCWSTR,
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    ) as i32;
    if Error != (ERROR_SUCCESS as i32) {
        SetLastError(Error as DWORD);
        return 0;
    }

    dwLength = (core::mem::size_of_val(&szBuffer)) as DWORD;
    Error = RegQueryValueExW(
        hKey,
        REG_PROFILES_DIRECTORY2.as_ptr() as LPCWSTR,
        core::ptr::null_mut(),
        &mut dwType,
        szBuffer.as_mut_ptr() as LPBYTE,
        &mut dwLength,
    ) as i32;
    if (Error != (ERROR_SUCCESS as i32)) || (dwType != REG_SZ && dwType != REG_EXPAND_SZ) {
        RegCloseKey(hKey);
        SetLastError(Error as DWORD);
        return 0;
    }

    RegCloseKey(hKey);

    if ExpandEnvironmentStringsW(
        szBuffer.as_ptr(),
        szProfilesPath.as_mut_ptr(),
        (core::mem::size_of_val(&szProfilesPath) / core::mem::size_of::<WCHAR>()) as DWORD,
    ) == 0
    {
        return 0;
    }

    dwLength = (wcslen(szProfilesPath.as_ptr()) + 1) as DWORD;
    if !lpProfilesDir.is_null() && (*lpcchSize >= dwLength) {
        StringCchCopyW(lpProfilesDir, (*lpcchSize as usize), szProfilesPath.as_ptr());
        *lpcchSize = dwLength;
        return 1;
    } else {
        *lpcchSize = dwLength;
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        return 0;
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetProfileType(pdwFlags: PDWORD) -> BOOL {
    let mut SidString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut hToken: HANDLE = core::ptr::null_mut();
    let mut hProfilesKey: HKEY = core::ptr::null_mut();
    let mut hProfileKey: HKEY = core::ptr::null_mut();
    let mut dwType: DWORD = 0;
    let mut dwLength: DWORD = 0;
    let mut dwState: DWORD = 0;
    let mut dwError: DWORD = 0;
    let mut bResult: BOOL = 0;

    if pdwFlags.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, 1, &mut hToken) == 0 {
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut hToken) == 0 {
            return 0;
        }
    }

    'done: {
    if GetUserSidStringFromToken(hToken, &mut SidString) == 0 {
        break 'done;
    }

    // L"SOFTWARE\Microsoft\Windows NT\CurrentVersion\ProfileList"
    static PROFILE_LIST_KEY_BUF: [u16; 57] = [
        b'S' as u16, b'O' as u16, b'F' as u16, b'T' as u16, b'W' as u16, b'A' as u16, b'R' as u16, b'E' as u16, b'\\' as u16, b'M' as u16,
        b'i' as u16, b'c' as u16, b'r' as u16, b'o' as u16, b's' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'\\' as u16, b'W' as u16,
        b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b's' as u16, b' ' as u16, b'N' as u16, b'T' as u16, b'\\' as u16,
        b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16, b'V' as u16, b'e' as u16, b'r' as u16,
        b's' as u16, b'i' as u16, b'o' as u16, b'n' as u16, b'\\' as u16, b'P' as u16, b'r' as u16, b'o' as u16, b'f' as u16, b'i' as u16,
        b'l' as u16, b'e' as u16, b'L' as u16, b'i' as u16, b's' as u16, b't' as u16, 0,
    ];
    dwError = RegOpenKeyExW(
        core::ptr::null_mut(),
        PROFILE_LIST_KEY_BUF.as_ptr() as *const u16 as LPCWSTR,
        0,
        KEY_QUERY_VALUE,
        &mut hProfilesKey,
    );
    if dwError != ERROR_SUCCESS {
        SetLastError(dwError);
        break 'done;
    }

    dwError = RegOpenKeyExW(
        hProfilesKey,
        SidString.Buffer as LPCWSTR,
        0,
        KEY_QUERY_VALUE | KEY_SET_VALUE,
        &mut hProfileKey,
    );
    if dwError != ERROR_SUCCESS {
        SetLastError(dwError);
        break 'done;
    }

    dwLength = core::mem::size_of::<DWORD>() as DWORD;
    // L"State"
    static STATE_VALUE_BUF: [u16; 6] = [b'S' as u16, b't' as u16, b'a' as u16, b't' as u16, b'e' as u16, 0];
    dwError = RegQueryValueExW(
        hProfileKey,
        STATE_VALUE_BUF.as_ptr() as *const u16 as LPCWSTR,
        core::ptr::null_mut(),
        &mut dwType,
        &mut dwState as *mut DWORD as LPBYTE,
        &mut dwLength,
    );
    if dwError != ERROR_SUCCESS {
        SetLastError(dwError);
        break 'done;
    }

    *pdwFlags = 0;

    if dwState & 0x80 != 0 {
        *pdwFlags |= 1;
    }

    bResult = 1;

    }

    'block: {
        if !hProfileKey.is_null() {
            RegCloseKey(hProfileKey);
        }

        if !hProfilesKey.is_null() {
            RegCloseKey(hProfilesKey);
        }

        RtlFreeUnicodeString(&mut SidString);

        CloseHandle(hToken);

        return bResult;
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetUserProfileDirectoryA(
    hToken: HANDLE,
    lpProfileDir: LPSTR,
    lpcchSize: LPDWORD,
) -> i32 {
    let mut lpBuffer: LPWSTR;
    let mut bResult: i32;

    if lpcchSize.is_null() || lpProfileDir.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    lpBuffer = GlobalAlloc(GMEM_FIXED, (*lpcchSize as usize).wrapping_mul(core::mem::size_of::<u16>())) as LPWSTR;
    if lpBuffer.is_null() {
        return 0;
    }

    bResult = GetUserProfileDirectoryW(hToken, lpBuffer, lpcchSize) as BOOL;
    if bResult != 0 && !lpProfileDir.is_null() {
        bResult = WideCharToMultiByte(
            CP_ACP,
            0 as u32,
            lpBuffer as LPCWSTR,
            -1 as i32,
            lpProfileDir,
            *lpcchSize as i32,
            core::ptr::null::<i8>(),
            core::ptr::null_mut::<i32>(),
        ) as i32;
    }

    GlobalFree(lpBuffer as *mut core::ffi::c_void);

    return bResult;
}

#[no_mangle]
pub unsafe extern "system" fn GetUserProfileDirectoryW(
    hToken: HANDLE,
    lpProfileDir: LPWSTR,
    lpcchSize: LPDWORD,
) -> BOOL {
    let mut SidString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut szKeyName: [u16; 260] = [0; 260];
    let mut szRawImagePath: [u16; 260] = [0; 260];
    let mut szImagePath: [u16; 260] = [0; 260];
    let mut dwType: u32 = 0;
    let mut dwLength: u32 = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut Error: i32 = 0;

    if hToken.is_null() {
        SetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    if lpcchSize.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if GetUserSidStringFromToken(hToken, core::ptr::addr_of_mut!(SidString)) == 0 {
        return 0;
    }

    StringCbCopyW(
        core::ptr::addr_of_mut!(szKeyName) as *mut u16,
        core::mem::size_of::<[u16; 260]>(),
        win32macros_rust::wide_bytes!(b"S\0O\0F\0T\0W\0A\0R\0E\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0W\0i\0n\0d\0o\0w\0s\0 \0N\0T\0\\\0C\0u\0r\0r\0e\0n\0t\0V\0e\0r\0s\0i\0o\0n\0\\\0P\0r\0o\0f\0i\0l\0e\0L\0i\0s\0t\0\\\0\0\0") as LPCWSTR,
    );
    StringCbCatW(
        core::ptr::addr_of_mut!(szKeyName) as *mut u16,
        core::mem::size_of::<[u16; 260]>(),
        SidString.Buffer,
    );

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(SidString));

    Error = RegOpenKeyExW(
        0 as HKEY,
        core::ptr::addr_of_mut!(szKeyName) as *mut u16,
        0,
        KEY_QUERY_VALUE,
        core::ptr::addr_of_mut!(hKey),
    ) as i32;
    if Error != ERROR_SUCCESS as i32 {
        SetLastError(Error as u32);
        return 0;
    }

    dwLength = core::mem::size_of::<[u16; 260]>() as u32;
    Error = RegQueryValueExW(
        hKey,
        win32macros_rust::wide_bytes!(b"P\0r\0o\0f\0i\0l\0e\0I\0m\0a\0g\0e\0P\0a\0t\0h\0\0\0") as LPCWSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(szRawImagePath) as *mut u8,
        core::ptr::addr_of_mut!(dwLength),
    ) as i32;
    if (Error != ERROR_SUCCESS as i32) || (dwType != REG_SZ && dwType != REG_EXPAND_SZ) {
        RegCloseKey(hKey);
        SetLastError(Error as u32);
        return 0;
    }

    RegCloseKey(hKey);

    if ExpandEnvironmentStringsW(
        core::ptr::addr_of_mut!(szRawImagePath) as *mut u16,
        szImagePath.as_mut_ptr(),
        (core::mem::size_of::<[u16; 260]>() / core::mem::size_of::<u16>()) as u32,
    ) == 0
    {
        return 0;
    }

    dwLength = (wcslen(szImagePath.as_mut_ptr()) + 1) as u32;
    if !lpProfileDir.is_null() && *lpcchSize >= dwLength {
        StringCchCopyW(
            lpProfileDir,
            *lpcchSize as usize,
            szImagePath.as_mut_ptr(),
        );
        *lpcchSize = dwLength;
        return 1;
    } else {
        *lpcchSize = dwLength;
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        return 0;
    }
}

#[no_mangle]
pub unsafe extern "system" fn LoadUserProfileA(
    hToken: HANDLE,
    lpProfileInfo: LPPROFILEINFOA,
) -> BOOL {
    let mut bResult: BOOL = 0;
    let mut ProfileInfoW: PROFILEINFOW = core::mem::zeroed();
    let mut len: i32;

    if lpProfileInfo.is_null()
        || (*lpProfileInfo).dwSize as usize != core::mem::size_of::<PROFILEINFOA>()
        || (*lpProfileInfo).lpUserName.is_null()
        || *(*lpProfileInfo).lpUserName as i32 == 0
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    'cleanup: {
    ProfileInfoW.dwSize = core::mem::size_of::<PROFILEINFOW>() as DWORD;
    ProfileInfoW.dwFlags = (*lpProfileInfo).dwFlags;

    if !(*lpProfileInfo).lpUserName.is_null() {
        len = MultiByteToWideChar(
            CP_ACP,
            0,
            (*lpProfileInfo).lpUserName,
            -1,
            core::ptr::null_mut(),
            0,
        );
        ProfileInfoW.lpUserName = HeapAlloc(GetProcessHeap(), 0, (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>())) as LPWSTR;
        if ProfileInfoW.lpUserName.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            break 'cleanup;
        } else {
            MultiByteToWideChar(
                CP_ACP,
                0,
                (*lpProfileInfo).lpUserName,
                -1,
                ProfileInfoW.lpUserName,
                len,
            );
        }
    }

    if !(*lpProfileInfo).lpProfilePath.is_null() {
        len = MultiByteToWideChar(
            CP_ACP,
            0,
            (*lpProfileInfo).lpProfilePath,
            -1,
            core::ptr::null_mut(),
            0,
        );
        ProfileInfoW.lpProfilePath = HeapAlloc(GetProcessHeap(), 0, (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>())) as LPWSTR;
        if ProfileInfoW.lpProfilePath.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            break 'cleanup;
        } else {
            MultiByteToWideChar(
                CP_ACP,
                0,
                (*lpProfileInfo).lpProfilePath,
                -1,
                ProfileInfoW.lpProfilePath,
                len,
            );
        }
    }

    if !(*lpProfileInfo).lpDefaultPath.is_null() {
        len = MultiByteToWideChar(
            CP_ACP,
            0,
            (*lpProfileInfo).lpDefaultPath,
            -1,
            core::ptr::null_mut(),
            0,
        );
        ProfileInfoW.lpDefaultPath = HeapAlloc(GetProcessHeap(), 0, (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>())) as LPWSTR;
        if ProfileInfoW.lpDefaultPath.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            break 'cleanup;
        } else {
            MultiByteToWideChar(
                CP_ACP,
                0,
                (*lpProfileInfo).lpDefaultPath,
                -1,
                ProfileInfoW.lpDefaultPath,
                len,
            );
        }
    }

    if !(*lpProfileInfo).lpServerName.is_null() {
        len = MultiByteToWideChar(
            CP_ACP,
            0,
            (*lpProfileInfo).lpServerName,
            -1,
            core::ptr::null_mut(),
            0,
        );
        ProfileInfoW.lpServerName = HeapAlloc(GetProcessHeap(), 0, (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>())) as LPWSTR;
        if ProfileInfoW.lpServerName.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            break 'cleanup;
        } else {
            MultiByteToWideChar(
                CP_ACP,
                0,
                (*lpProfileInfo).lpServerName,
                -1,
                ProfileInfoW.lpServerName,
                len,
            );
        }
    }

    if (ProfileInfoW.dwFlags as u32 & 4u32) != 0 && !(*lpProfileInfo).lpPolicyPath.is_null() {
        len = MultiByteToWideChar(
            CP_ACP,
            0,
            (*lpProfileInfo).lpPolicyPath,
            -1,
            core::ptr::null_mut(),
            0,
        );
        ProfileInfoW.lpPolicyPath = HeapAlloc(GetProcessHeap(), 0, (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>())) as LPWSTR;
        if ProfileInfoW.lpPolicyPath.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            break 'cleanup;
        } else {
            MultiByteToWideChar(
                CP_ACP,
                0,
                (*lpProfileInfo).lpPolicyPath,
                -1,
                ProfileInfoW.lpPolicyPath,
                len,
            );
        }
    }

    bResult = LoadUserProfileW(hToken, core::ptr::addr_of_mut!(ProfileInfoW) as *mut core::ffi::c_void);
    (*lpProfileInfo).hProfile = ProfileInfoW.hProfile;

    }
    {
        if !ProfileInfoW.lpUserName.is_null() {
            HeapFree(GetProcessHeap(), 0, ProfileInfoW.lpUserName as LPVOID);
        }
        if !ProfileInfoW.lpProfilePath.is_null() {
            HeapFree(GetProcessHeap(), 0, ProfileInfoW.lpProfilePath as LPVOID);
        }
        if !ProfileInfoW.lpDefaultPath.is_null() {
            HeapFree(GetProcessHeap(), 0, ProfileInfoW.lpDefaultPath as LPVOID);
        }
        if !ProfileInfoW.lpServerName.is_null() {
            HeapFree(GetProcessHeap(), 0, ProfileInfoW.lpServerName as LPVOID);
        }
        if (ProfileInfoW.dwFlags as u32 & 4u32) != 0 && !ProfileInfoW.lpPolicyPath.is_null() {
            HeapFree(GetProcessHeap(), 0, ProfileInfoW.lpPolicyPath as LPVOID);
        }
    }

    bResult
}


#[no_mangle]
pub unsafe extern "system" fn LoadUserProfileW(
    mut hToken: HANDLE,
    mut lpProfileInfo: *mut core::ffi::c_void,
) -> BOOL {
    let lpProfileInfo = lpProfileInfo as *mut core::ffi::c_void;
    
    let mut szUserHivePath: [u16; 260] = [0; 260];
    let mut dwLength: DWORD = 0;
    let mut UserSid: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut SidString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut hProfileMutex: HANDLE = core::ptr::null_mut();
    let mut dwError: DWORD = MAXDWORD;
    let mut ret: BOOL = 0;

    if lpProfileInfo.is_null()
        || (*(lpProfileInfo as *mut core::ffi::c_void as *const u32)).wrapping_add(1) == 0
        || (*(lpProfileInfo as *mut *mut u16)).is_null()
        || **(lpProfileInfo as *mut *mut u16) == 0
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    'cleanup: {
    ret = GetUserSidStringFromToken(hToken, &mut SidString);
    if ret == 0 {
        loop {
            break 'cleanup;
        }
    }
    ret = 0;

    hProfileMutex = CreateProfileMutex(SidString.Buffer);
    if hProfileMutex.is_null() {
        loop {
            break 'cleanup;
        }
    }

    WaitForSingleObject(hProfileMutex, INFINITE);

    if CheckForLoadedProfile(hToken) != 0 {
    } else {
        let lpProfilePath = *(lpProfileInfo as *mut *mut u16).add(4);

        if !lpProfilePath.is_null() {
            StringCbCopyW(
                szUserHivePath.as_mut_ptr(),
                core::mem::size_of::<[u16; 260]>(),
                lpProfilePath,
            );
        } else {
            dwLength = (core::mem::size_of::<[u16; 260]>() / core::mem::size_of::<u16>()) as DWORD;
            if GetProfilesDirectoryW(szUserHivePath.as_mut_ptr(), &mut dwLength) == 0 {
                loop {
                    break 'cleanup;
                }
            }
            StringCbCatW(
                szUserHivePath.as_mut_ptr(),
                core::mem::size_of::<[u16; 260]>(),
                PATH_SEP_BACKSLASH2.as_ptr() as LPCWSTR,
            );
            StringCbCatW(
                szUserHivePath.as_mut_ptr(),
                core::mem::size_of::<[u16; 260]>(),
                *(lpProfileInfo as *mut *mut u16).add(3),
            );
        }

        StringCbCatW(
            szUserHivePath.as_mut_ptr(),
            core::mem::size_of::<[u16; 260]>(),
            NTUSER_DAT_SUFFIX.as_ptr() as LPCWSTR,
        );

        if GetFileAttributesW(szUserHivePath.as_mut_ptr() as LPCWSTR) == 0xFFFFFFFF {
            if GetTokenInformation(hToken, TokenUser, core::ptr::null_mut(), 0, &mut dwLength) != 0
                || GetLastError() != ERROR_INSUFFICIENT_BUFFER
            {
                loop {
                    break 'cleanup;
                }
            }

            UserSid = HeapAlloc(GetProcessHeap(), 0, dwLength as usize);
            if UserSid.is_null() {
                dwError = ERROR_NOT_ENOUGH_MEMORY;
                loop {
                    break 'cleanup;
                }
            }

            if GetTokenInformation(hToken, TokenUser, UserSid, dwLength, &mut dwLength) == 0 {
                loop {
                    break 'cleanup;
                }
            }

            ret = CreateUserProfileW(
                *(UserSid as *mut *mut core::ffi::c_void),
                *(lpProfileInfo as *mut *mut u16).add(3),
            );
            if ret == 0 {
                loop {
                    break 'cleanup;
                }
            }
            ret = 0;
        }

        if AcquireRemoveRestorePrivilege(1) == 0 {
            loop {
                break 'cleanup;
            }
        }

        dwError = RegLoadKeyW(
            0x80000003 as HKEY,
            SidString.Buffer,
            szUserHivePath.as_mut_ptr() as LPCWSTR,
        ) as u32;
        AcquireRemoveRestorePrivilege(0);

        if dwError == ERROR_SHARING_VIOLATION {
            dwError = ERROR_SUCCESS;
        }

        if dwError != ERROR_SUCCESS {
            loop {
                break 'cleanup;
            }
        }

        SetProfileData(SidString.Buffer, *(lpProfileInfo as *mut u32).add(6), hToken);
    }

    dwError = RegOpenKeyExW(
        0x80000003 as HKEY,
        SidString.Buffer,
        0,
        MAXIMUM_ALLOWED,
        &mut *(lpProfileInfo as *mut HKEY).add(5),
    );
    if dwError != ERROR_SUCCESS {
        loop {
            break 'cleanup;
        }
    }

    dwError = IncrementRefCount(SidString.Buffer, core::ptr::null_mut());
    if dwError != ERROR_SUCCESS {
        loop {
            break 'cleanup;
        }
    }

    ret = 1;

    }
    if dwError == MAXDWORD {
        dwError = GetLastError();
    }

    if !UserSid.is_null() {
        HeapFree(GetProcessHeap(), 0, UserSid);
    }

    if !hProfileMutex.is_null() {
        ReleaseMutex(hProfileMutex);
        CloseHandle(hProfileMutex);
    }

    RtlFreeUnicodeString(&mut SidString);

    if ret == 0 {
        ReportError(
            *(lpProfileInfo as *mut u32).add(6),
            ERR_PROFILE_LOAD_FAILED.as_ptr() as LPCWSTR,
            dwError,
        );
    }

    SetLastError(dwError);
    ret
}
#[no_mangle]
pub unsafe extern "system" fn UnloadUserProfile(mut hToken: HANDLE, mut hProfile: HANDLE) -> BOOL {
    let mut SidString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut hProfileMutex: HANDLE = core::ptr::null_mut();
    let mut hProfilesKey: HKEY = core::ptr::null_mut();
    let mut hProfileKey: HKEY = core::ptr::null_mut();
    let mut dwRefCount: DWORD = 0;
    let mut dwLength: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut dwState: DWORD = 0;
    let mut dwError: DWORD = 0;
    let mut bRet: BOOL = 0;

    if hProfile.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if GetUserSidStringFromToken(hToken, &mut SidString) == 0 {
        return 0;
    }

    'cleanup: {
    hProfileMutex = CreateProfileMutex(SidString.Buffer);
    if hProfileMutex.is_null() {
        break 'cleanup;
    } else {
        WaitForSingleObject(hProfileMutex, INFINITE);
        RegFlushKey(hProfile as HKEY);
        RegCloseKey(hProfile as HKEY);

        dwError = DecrementRefCount(SidString.Buffer, &mut dwRefCount);
        if dwError != ERROR_SUCCESS {
            SetLastError(dwError);
            break 'cleanup;
        }

        if dwRefCount == 0 {
            if AcquireRemoveRestorePrivilege(1) == 0 {
                break 'cleanup;
            }

            let mut hUserKey: HKEY = core::ptr::null_mut();
            dwError = RegOpenKeyExW(
                HKEY_USERS,
                SidString.Buffer,
                0,
                KEY_WRITE,
                &mut hUserKey,
            );
            if dwError == ERROR_SUCCESS {
                RegDeleteKeyW(hUserKey, win32macros_rust::wide_bytes!(b"V\0o\0l\0a\0t\0i\0l\0e\0 \0E\0n\0v\0i\0r\0o\0n\0m\0e\0n\0t\0\0\0") as PWSTR);
                RegCloseKey(hUserKey);
            }

            dwError = RegUnLoadKeyW(HKEY_USERS, SidString.Buffer);
            AcquireRemoveRestorePrivilege(0);

            if dwError != ERROR_SUCCESS {
                SetLastError(dwError);
                break 'cleanup;
            }

            dwError = RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                win32macros_rust::wide_bytes!(b"S\0O\0F\0T\0W\0A\0R\0E\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0W\0i\0n\0d\0o\0w\0s\0 \0N\0T\0\\\0C\0u\0r\0r\0e\0n\0t\0V\0e\0r\0s\0i\0o\0n\0\\\0P\0r\0o\0f\0i\0l\0e\0L\0i\0s\0t\0\0\0") as PWSTR,
                0,
                KEY_QUERY_VALUE,
                &mut hProfilesKey,
            );
            if dwError != ERROR_SUCCESS {
                SetLastError(dwError);
                break 'cleanup;
            }

            dwError = RegOpenKeyExW(
                hProfilesKey,
                SidString.Buffer,
                0,
                KEY_QUERY_VALUE,
                &mut hProfileKey,
            );
            if dwError != ERROR_SUCCESS {
                SetLastError(dwError);
                break 'cleanup;
            }

            dwLength = core::mem::size_of::<DWORD>() as DWORD;
            dwError = RegQueryValueExW(
                hProfileKey,
                win32macros_rust::wide_bytes!(b"S\0t\0a\0t\0e\0\0\0") as PWSTR,
                core::ptr::null_mut(),
                &mut dwType,
                &mut dwState as *mut DWORD as PBYTE,
                &mut dwLength,
            );
            if dwError != ERROR_SUCCESS {
                SetLastError(dwError);
                break 'cleanup;
            }

            if dwState & 0x80 != 0 {
                if DeleteProfileW(SidString.Buffer, core::ptr::null(), core::ptr::null()) == 0 {
                    break 'cleanup;
                }
            }
        }

        bRet = 1;
    }

    }
    if !hProfileKey.is_null() {
        RegCloseKey(hProfileKey);
    }

    if !hProfilesKey.is_null() {
        RegCloseKey(hProfilesKey);
    }

    if !hProfileMutex.is_null() {
        ReleaseMutex(hProfileMutex);
        CloseHandle(hProfileMutex);
    }

    RtlFreeUnicodeString(&mut SidString);

    return bRet;
}


// enforce_import_abi: moved 37 import decl(s) into the extern block their
// exporting DLL's .spec requires: AdjustTokenPrivileges->system, AllocateAndInitializeSid->system, CloseHandle->system, ConvertSidToStringSidW->system, CreateDirectoryW->system, CreateMutexW->system, EqualSid->system, ExpandEnvironmentStringsW->system, FreeSid->system, GetCurrentProcess->system, GetCurrentThread->system, GetLastError->system, GetLengthSid->system, GetProcessHeap->system, GetTokenInformation->system, GlobalAlloc->system, GlobalFree->system, HeapAlloc->system, HeapFree->system, InitializeSecurityDescriptor->system, LocalFree->system, LookupPrivilegeValueA->system, MultiByteToWideChar->system, OpenProcessToken->system, OpenThreadToken->system, RegCloseKey->system, RegCreateKeyExW->system, RegLoadKeyW->system, RegOpenKeyExW->system, RegQueryValueExW->system, RegSetValueExW->system, RegUnLoadKeyW->system, ReleaseMutex->system, SetLastError->system, SetSecurityDescriptorDacl->system, WaitForSingleObject->system, WideCharToMultiByte->system.
// The CRT is cdecl, Win32 is stdcall, and the C source does not say which is which —
// a wrong guess is a silent i386 stack walk on every call. The .spec is the truth.
extern "system" {
    fn RegOpenKeyExW(hKey: HKEY, lpSubKey: LPCWSTR, ulOptions: u32, samDesired: u32, phkResult: *mut HKEY) -> u32;
    fn RegCloseKey(hKey: HKEY) -> u32;
    fn AllocateAndInitializeSid(pIdentifierAuthority: *mut SID_IDENTIFIER_AUTHORITY, nSubAuthorityCount: u8, dwSubAuthority0: u32, dwSubAuthority1: u32, dwSubAuthority2: u32, dwSubAuthority3: u32, dwSubAuthority4: u32, dwSubAuthority5: u32, dwSubAuthority6: u32, dwSubAuthority7: u32, pSid: *mut PSID) -> i32;
    fn EqualSid(pSid1: PSID, pSid2: PSID) -> i32;
    fn FreeSid(pSid: PSID) -> PVOID;
    fn GetLastError() -> DWORD;
    fn GetProcessHeap() -> HANDLE;
    fn GetTokenInformation(TokenHandle: HANDLE, TokenInformationClass: u32, TokenInformation: PVOID, TokenInformationLength: DWORD, ReturnLength: PDWORD) -> i32;
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: usize) -> PVOID;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: PVOID) -> i32;
    fn OpenProcessToken(hProcess: HANDLE, dwDesiredAccess: DWORD, phToken: *mut HANDLE) -> BOOL;
    fn GetCurrentProcess() -> HANDLE;
    fn LookupPrivilegeValueA(lpSystemName: LPCSTR, lpName: LPCSTR, lpLuid: *mut LUID) -> BOOL;
    fn AdjustTokenPrivileges(TokenHandle: HANDLE, DisableAllPrivileges: BOOL, NewState: *mut TOKEN_PRIVILEGES, BufferLength: DWORD, PreviousState: *mut TOKEN_PRIVILEGES, ReturnLength: *mut DWORD) -> BOOL;
    fn CloseHandle(hObject: HANDLE) -> BOOL;
    fn CreateMutexW(lpMutexAttributes: *mut SECURITY_ATTRIBUTES, bInitialOwner: i32, lpName: LPCWSTR) -> HANDLE;
    fn InitializeSecurityDescriptor(pSecurityDescriptor: *mut _SECURITY_DESCRIPTOR, dwRevision: u32) -> i32;
    fn SetLastError(dwErrCode: u32);
    fn SetSecurityDescriptorDacl(pSecurityDescriptor: *mut _SECURITY_DESCRIPTOR, bDaclPresent: i32, pDacl: *mut core::ffi::c_void, bDaclDefaulted: i32) -> i32;
    fn ExpandEnvironmentStringsW(lpSrc: LPCWSTR, lpDst: LPWSTR, cchDst: DWORD) -> DWORD;
    fn RegQueryValueExW(hKey: HKEY, lpValueName: LPCWSTR, lpReserved: *mut DWORD, lpType: *mut DWORD, lpData: LPBYTE, lpcbData: *mut DWORD) -> DWORD;
    fn GetCurrentThread() -> HANDLE;
    fn OpenThreadToken(ThreadHandle: HANDLE, DesiredAccess: DWORD, OpenAsSelf: BOOL, TokenHandle: *mut HANDLE) -> BOOL;
    fn ConvertSidToStringSidW(pSid: PSID, ppStringSid: *mut LPWSTR) -> BOOL;
    fn CreateDirectoryW(lpPathName: LPCWSTR, lpSecurityAttributes: *mut SECURITY_ATTRIBUTES) -> BOOL;
    fn GetLengthSid(pSid: PSID) -> DWORD;
    fn LocalFree(hMem: HLOCAL) -> HLOCAL;
    fn RegCreateKeyExW(hKey: HKEY, lpSubKey: LPCWSTR, Reserved: DWORD, lpClass: LPWSTR, dwOptions: DWORD, samDesired: ::core::ffi::c_ulong, lpSecurityAttributes: *mut SECURITY_ATTRIBUTES, phkResult: *mut HKEY, lpdwDisposition: *mut DWORD) -> LONG;
    fn RegLoadKeyW(hKey: HKEY, lpSubKey: LPCWSTR, lpFile: LPCWSTR) -> LONG;
    fn RegSetValueExW(hKey: HKEY, lpValueName: LPCWSTR, Reserved: DWORD, dwType: DWORD, lpData: *const BYTE, cbData: DWORD) -> LONG;
    fn RegUnLoadKeyW(hKey: HKEY, lpSubKey: LPCWSTR) -> u32;
    fn GlobalAlloc(uFlags: u32, dwBytes: usize) -> *mut core::ffi::c_void;
    fn GlobalFree(hMem: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn WideCharToMultiByte( CodePage: u32, dwFlags: u32, lpWideCharStr: LPCWSTR, cchWideChar: i32, lpMultiByteStr: LPSTR, cbMultiByte: i32, lpDefaultChar: *const i8, lpUsedDefaultChar: *mut i32, ) -> i32;
    fn MultiByteToWideChar(CodePage: UINT, dwFlags: DWORD, lpMultiByteStr: LPCSTR, cchMultiByte: i32, lpWideCharStr: LPWSTR, cchWideChar: i32) -> i32;
    fn ReleaseMutex(hMutex: HANDLE) -> BOOL;
    fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/userenv/profile.c
//  * PURPOSE:         User profile code
//  * PROGRAMMERS:     Eric Kohl
//  *                  Herv� Poussineau
//  */
//
// #include "precomp.h"
//
// #include <sddl.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS ***************************************************************/
//
// BOOL
// AppendSystemPostfix(LPWSTR lpName,
//                     DWORD dwMaxLength)
// {
//     WCHAR szSystemRoot[MAX_PATH];
//     LPWSTR lpszPostfix;
//     LPWSTR lpszPtr;
//
//     /* Build profile name postfix */
//     if (!ExpandEnvironmentStringsW(L"%SystemRoot%",
//                                    szSystemRoot,
//                                    ARRAYSIZE(szSystemRoot)))
//     {
//         DPRINT1("Error: %lu\n", GetLastError());
//         return FALSE;
//     }
//
//     _wcsupr(szSystemRoot);
//
//     /* Get name postfix */
//     szSystemRoot[2] = L'.';
//     lpszPostfix = &szSystemRoot[2];
//     lpszPtr = lpszPostfix;
//     while (*lpszPtr != (WCHAR)0)
//     {
//         if (*lpszPtr == L'\\')
//             *lpszPtr = L'_';
//         lpszPtr++;
//     }
//
//     if (wcslen(lpName) + wcslen(lpszPostfix) + 1 >= dwMaxLength)
//     {
//         DPRINT1("Error: buffer overflow\n");
//         SetLastError(ERROR_BUFFER_OVERFLOW);
//         return FALSE;
//     }
//
//     wcscat(lpName, lpszPostfix);
//
//     return TRUE;
// }
//
//
// static
// BOOL
// AcquireRemoveRestorePrivilege(IN BOOL bAcquire)
// {
//     BOOL bRet = FALSE;
//     HANDLE Token;
//     TOKEN_PRIVILEGES TokenPriv;
//
//     DPRINT("AcquireRemoveRestorePrivilege(%d)\n", bAcquire);
//
//     if (OpenProcessToken(GetCurrentProcess(),
//                          TOKEN_ADJUST_PRIVILEGES,
//                          &Token))
//     {
//         TokenPriv.PrivilegeCount = 1;
//         TokenPriv.Privileges[0].Attributes = (bAcquire ? SE_PRIVILEGE_ENABLED : 0);
//
//         if (LookupPrivilegeValue(NULL, SE_RESTORE_NAME, &TokenPriv.Privileges[0].Luid))
//         {
//             bRet = AdjustTokenPrivileges(Token, FALSE, &TokenPriv, 0, NULL, NULL);
//
//             if (!bRet)
//             {
//                 DPRINT1("AdjustTokenPrivileges() failed with error %lu\n", GetLastError());
//             }
//             else if (GetLastError() == ERROR_NOT_ALL_ASSIGNED)
//             {
//                 DPRINT1("AdjustTokenPrivileges() succeeded, but with not all privileges assigned\n");
//                 bRet = FALSE;
//             }
//         }
//         else
//         {
//             DPRINT1("LookupPrivilegeValue() failed with error %lu\n", GetLastError());
//         }
//
//         CloseHandle(Token);
//     }
//     else
//     {
//         DPRINT1("OpenProcessToken() failed with error %lu\n", GetLastError());
//     }
//
//     return bRet;
// }
//
//
// static
// BOOL
// CheckForLoadedProfile(HANDLE hToken)
// {
//     UNICODE_STRING SidString;
//     HKEY hKey;
//
//     DPRINT("CheckForLoadedProfile() called\n");
//
//     /* Get the user SID string */
//     if (!GetUserSidStringFromToken(hToken, &SidString))
//     {
//         DPRINT1("GetUserSidStringFromToken() failed\n");
//         return FALSE;
//     }
//
//     if (RegOpenKeyExW(HKEY_USERS,
//                       SidString.Buffer,
//                       0,
//                       MAXIMUM_ALLOWED,
//                       &hKey))
//     {
//         DPRINT("Profile not loaded\n");
//         RtlFreeUnicodeString(&SidString);
//         return FALSE;
//     }
//
//     RegCloseKey(hKey);
//
//     RtlFreeUnicodeString(&SidString);
//
//     DPRINT("Profile already loaded\n");
//
//     return TRUE;
// }
//
//
// static
// HANDLE
// CreateProfileMutex(
//      PWSTR pszSidString)
// {
//     SECURITY_DESCRIPTOR SecurityDescriptor;
//     SECURITY_ATTRIBUTES SecurityAttributes;
//     PWSTR pszMutexName = NULL;
//     HANDLE hMutex = NULL;
//
//     pszMutexName = HeapAlloc(GetProcessHeap(),
//                              0,
//                              (wcslen(L"Global\\userenv:  User Profile Mutex for ") + wcslen(pszSidString) + 1) * sizeof(WCHAR));
//     if (pszMutexName == NULL)
//     {
//         DPRINT("Failed to allocate the mutex name buffer!\n");
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return NULL;
//     }
//
//     /* Build the profile mutex name */
//     wcscpy(pszMutexName, L"Global\\userenv:  User Profile Mutex for ");
//     wcscat(pszMutexName, pszSidString);
//
//     /* Initialize the security descriptor */
//     InitializeSecurityDescriptor(&SecurityDescriptor,
//                                  SECURITY_DESCRIPTOR_REVISION);
//
//     /* Set a NULL-DACL (everyone has access) */
//     SetSecurityDescriptorDacl(&SecurityDescriptor,
//                               TRUE,
//                               NULL,
//                               FALSE);
//
//     /* Initialize the security attributes */
//     SecurityAttributes.nLength = sizeof(SecurityAttributes);
//     SecurityAttributes.lpSecurityDescriptor = &SecurityDescriptor;
//     SecurityAttributes.bInheritHandle = FALSE;
//
//     /* Create the profile mutex */
//     hMutex = CreateMutexW(&SecurityAttributes,
//                           FALSE,
//                           pszMutexName);
//     if (hMutex == NULL)
//     {
//         DPRINT1("Failed to create the profile mutex (Error %lu)\n", GetLastError());
//     }
//
//     HeapFree(GetProcessHeap(), 0, pszMutexName);
//
//     return hMutex;
// }
//
//
// static
// DWORD
// IncrementRefCount(
//     PWSTR pszSidString,
//     PDWORD pdwRefCount)
// {
//     HKEY hProfilesKey = NULL, hProfileKey = NULL;
//     DWORD dwRefCount = 0, dwLength, dwType;
//     DWORD dwError;
//
//     DPRINT1("IncrementRefCount(%S %p)\n",
//             pszSidString, pdwRefCount);
//
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList",
//                             0,
//                             KEY_QUERY_VALUE,
//                             &hProfilesKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", dwError);
//         goto done;
//     }
//
//     dwError = RegOpenKeyExW(hProfilesKey,
//                             pszSidString,
//                             0,
//                             KEY_QUERY_VALUE | KEY_SET_VALUE,
//                             &hProfileKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", dwError);
//         goto done;
//     }
//
//     /* Get the reference counter */
//     dwLength = sizeof(dwRefCount);
//     RegQueryValueExW(hProfileKey,
//                      L"RefCount",
//                      NULL,
//                      &dwType,
//                      (PBYTE)&dwRefCount,
//                      &dwLength);
//
//     dwRefCount++;
//
//     dwLength = sizeof(dwRefCount);
//     dwError = RegSetValueExW(hProfileKey,
//                              L"RefCount",
//                              0,
//                              REG_DWORD,
//                              (PBYTE)&dwRefCount,
//                              dwLength);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", dwError);
//         goto done;
//     }
//
//     if (pdwRefCount != NULL)
//         *pdwRefCount = dwRefCount;
//
// done:
//     if (hProfileKey != NULL)
//         RegCloseKey(hProfileKey);
//
//     if (hProfilesKey != NULL)
//         RegCloseKey(hProfilesKey);
//
//     return dwError;
// }
//
//
// static
// DWORD
// DecrementRefCount(
//     PWSTR pszSidString,
//     PDWORD pdwRefCount)
// {
//     HKEY hProfilesKey = NULL, hProfileKey = NULL;
//     DWORD dwRefCount = 0, dwLength, dwType;
//     DWORD dwError;
//
//     DPRINT1("DecrementRefCount(%S %p)\n",
//             pszSidString, pdwRefCount);
//
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList",
//                             0,
//                             KEY_QUERY_VALUE,
//                             &hProfilesKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", dwError);
//         goto done;
//     }
//
//     dwError = RegOpenKeyExW(hProfilesKey,
//                             pszSidString,
//                             0,
//                             KEY_QUERY_VALUE | KEY_SET_VALUE,
//                             &hProfileKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", dwError);
//         goto done;
//     }
//
//     /* Get the reference counter */
//     dwLength = sizeof(dwRefCount);
//     dwError = RegQueryValueExW(hProfileKey,
//                                L"RefCount",
//                                NULL,
//                                &dwType,
//                                (PBYTE)&dwRefCount,
//                                &dwLength);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", dwError);
//         goto done;
//     }
//
//     dwRefCount--;
//
//     dwLength = sizeof(dwRefCount);
//     dwError = RegSetValueExW(hProfileKey,
//                              L"RefCount",
//                              0,
//                              REG_DWORD,
//                              (PBYTE)&dwRefCount,
//                              dwLength);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", dwError);
//         goto done;
//     }
//
//     if (pdwRefCount != NULL)
//         *pdwRefCount = dwRefCount;
//
// done:
//     if (hProfileKey != NULL)
//         RegCloseKey(hProfileKey);
//
//     if (hProfilesKey != NULL)
//         RegCloseKey(hProfilesKey);
//
//     return dwError;
// }
//
//
// static
// DWORD
// CheckForGuestsAndAdmins(
//      HANDLE hToken,
//      PDWORD pdwState)
// {
//     PTOKEN_GROUPS pGroupInfo = NULL;
//     PSID pAdministratorsSID = NULL;
//     PSID pGuestsSID = NULL;
//     DWORD i, dwSize = 0;
//     DWORD dwError = ERROR_SUCCESS;
//
//     DPRINT("CheckForGuestsAndAdmins(%p %p)\n", hToken, pdwState);
//
//     /* Get the buffer size */
//     if (!GetTokenInformation(hToken, TokenGroups, NULL, dwSize, &dwSize))
//     {
//         dwError = GetLastError();
//         if (dwError != ERROR_INSUFFICIENT_BUFFER)
//         {
//             DPRINT1("GetTokenInformation() failed (Error %lu)\n", dwError);
//             return dwError;
//         }
//
//         dwError = ERROR_SUCCESS;
//     }
//
//     /* Allocate the buffer */
//     pGroupInfo = (PTOKEN_GROUPS)HeapAlloc(GetProcessHeap(), 0, dwSize);
//     if (pGroupInfo == NULL)
//     {
//         dwError = ERROR_OUTOFMEMORY;
//         DPRINT1("HeapAlloc() failed (Error %lu)\n", dwError);
//         goto done;
//     }
//
//     /* Get the token groups */
//     if (!GetTokenInformation(hToken, TokenGroups, pGroupInfo, dwSize, &dwSize))
//     {
//         dwError = GetLastError();
//         DPRINT1("GetTokenInformation() failed (Error %lu)\n", dwError);
//         goto done;
//     }
//
//     /* Build the Administrators Group SID */
//     if(!AllocateAndInitializeSid(&LocalSystemAuthority,
//                                  2,
//                                  SECURITY_BUILTIN_DOMAIN_RID,
//                                  DOMAIN_ALIAS_RID_ADMINS,
//                                  0, 0, 0, 0, 0, 0,
//                                  &pAdministratorsSID))
//     {
//         dwError = GetLastError();
//         DPRINT1("AllocateAndInitializeSid() failed (Error %lu)\n", dwError);
//         goto done;
//     }
//
//     /* Build the Guests Group SID */
//     if(!AllocateAndInitializeSid(&LocalSystemAuthority,
//                                  2,
//                                  SECURITY_BUILTIN_DOMAIN_RID,
//                                  DOMAIN_ALIAS_RID_GUESTS,
//                                  0, 0, 0, 0, 0, 0,
//                                  &pGuestsSID))
//     {
//         dwError = GetLastError();
//         DPRINT1("AllocateAndInitializeSid() failed (Error %lu)\n", dwError);
//         goto done;
//     }
//
//     /* Check for Administratos or Guests group memberships */
//     for (i = 0; i < pGroupInfo->GroupCount; i++)
//     {
//         if (EqualSid(pAdministratorsSID, pGroupInfo->Groups[i].Sid))
//         {
//             *pdwState |= 0x0100; // PROFILE_ADMIN_USER
//         }
//
//         if (EqualSid(pGuestsSID, pGroupInfo->Groups[i].Sid))
//         {
//             *pdwState |= 0x0080; // PROFILE_GUESTS_USER
//         }
//     }
//
//     dwError = ERROR_SUCCESS;
//
// done:
//     if (pGuestsSID != NULL)
//         FreeSid(pGuestsSID);
//
//     if (pAdministratorsSID != NULL)
//         FreeSid(pAdministratorsSID);
//
//     if (pGroupInfo != NULL)
//         HeapFree(GetProcessHeap(), 0, pGroupInfo);
//
//     return dwError;
// }
//
//
// static
// DWORD
// SetProfileData(
//      PWSTR pszSidString,
//      DWORD dwFlags,
//      HANDLE hToken)
// {
//     HKEY hProfilesKey = NULL, hProfileKey = NULL;
//     FILETIME LoadTime;
//     DWORD dwLength, dwState = 0;
//     DWORD dwError;
//
//     DPRINT("SetProfileData(%S %p)\n", pszSidString, hToken);
//
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList",
//                             0,
//                             KEY_QUERY_VALUE,
//                             &hProfilesKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", dwError);
//         goto done;
//     }
//
//     dwError = RegOpenKeyExW(hProfilesKey,
//                             pszSidString,
//                             0,
//                             KEY_QUERY_VALUE | KEY_SET_VALUE,
//                             &hProfileKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", dwError);
//         goto done;
//     }
//
//     /* Set the profile load time */
//     GetSystemTimeAsFileTime(&LoadTime);
//
//     dwLength = sizeof(LoadTime.dwLowDateTime);
//     dwError = RegSetValueExW(hProfileKey,
//                              L"ProfileLoadTimeLow",
//                              0,
//                              REG_DWORD,
//                              (PBYTE)&LoadTime.dwLowDateTime,
//                              dwLength);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", dwError);
//         goto done;
//     }
//
//     dwLength = sizeof(LoadTime.dwHighDateTime);
//     dwError = RegSetValueExW(hProfileKey,
//                              L"ProfileLoadTimeHigh",
//                              0,
//                              REG_DWORD,
//                              (PBYTE)&LoadTime.dwHighDateTime,
//                              dwLength);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", dwError);
//         goto done;
//     }
//
//     dwLength = sizeof(dwFlags);
//     dwError = RegSetValueExW(hProfileKey,
//                              L"Flags",
//                              0,
//                              REG_DWORD,
//                              (PBYTE)&dwFlags,
//                              dwLength);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", dwError);
//         goto done;
//     }
//
//     dwError = CheckForGuestsAndAdmins(hToken,
//                                       &dwState);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", dwError);
//         goto done;
//     }
//
//     dwLength = sizeof(dwState);
//     dwError = RegSetValueExW(hProfileKey,
//                              L"State",
//                              0,
//                              REG_DWORD,
//                              (PBYTE)&dwState,
//                              dwLength);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", dwError);
//         goto done;
//     }
//
// done:
//     if (hProfileKey != NULL)
//         RegCloseKey(hProfileKey);
//
//     if (hProfilesKey != NULL)
//         RegCloseKey(hProfilesKey);
//
//     return dwError;
// }
//
//
// /* PUBLIC FUNCTIONS ********************************************************/
//
// BOOL
// WINAPI
// CopySystemProfile(
//      ULONG Unused)
// {
//     WCHAR szKeyName[MAX_PATH];
//     WCHAR szRawProfilePath[MAX_PATH];
//     WCHAR szProfilePath[MAX_PATH];
//     WCHAR szDefaultProfilePath[MAX_PATH];
//     UNICODE_STRING SidString = {0, 0, NULL};
//     HANDLE hToken = NULL;
//     PSID pUserSid = NULL;
//     HKEY hProfileKey = NULL;
//     DWORD dwDisposition;
//     BOOL bResult = FALSE;
//     DWORD cchSize;
//     DWORD dwError;
//
//     DPRINT1("CopySystemProfile()\n");
//
//     if (!OpenProcessToken(GetCurrentProcess(),
//                           TOKEN_QUERY | TOKEN_IMPERSONATE,
//                           &hToken))
//     {
//         DPRINT1("Failed to open the process token (Error %lu)\n", GetLastError());
//         return FALSE;
//     }
//
//     pUserSid = GetUserSid(hToken);
//     if (pUserSid == NULL)
//     {
//         DPRINT1("Failed to get the users SID (Error %lu)\n", GetLastError());
//         goto done;
//     }
//
//     /* Get the user SID string */
//     if (!GetUserSidStringFromToken(hToken, &SidString))
//     {
//         DPRINT1("GetUserSidStringFromToken() failed\n");
//         goto done;
//     }
//
//     StringCbCopyW(szKeyName, sizeof(szKeyName),
//                   L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList\\");
//     StringCbCatW(szKeyName, sizeof(szKeyName), SidString.Buffer);
//
//     RtlFreeUnicodeString(&SidString);
//
//     dwError = RegCreateKeyExW(HKEY_LOCAL_MACHINE,
//                               szKeyName,
//                               0, NULL, 0,
//                               KEY_WRITE,
//                               NULL,
//                               &hProfileKey,
//                               &dwDisposition);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Failed to create the profile key for the %s profile (Error %lu)\n",
//                 SidString.Buffer, dwError);
//         goto done;
//     }
//
//     dwError = RegSetValueExW(hProfileKey,
//                              L"Sid",
//                              0,
//                              REG_BINARY,
//                              (PBYTE)pUserSid,
//                              RtlLengthSid(pUserSid));
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Failed to set the SID value (Error %lu)\n", dwError);
//         goto done;
//     }
//
//     wcscpy(szRawProfilePath,
//            L"%systemroot%\\system32\\config\\systemprofile");
//
//     dwError = RegSetValueExW(hProfileKey,
//                              L"ProfileImagePath",
//                              0,
//                              REG_EXPAND_SZ,
//                              (PBYTE)szRawProfilePath,
//                              (wcslen(szRawProfilePath) + 1) * sizeof(WCHAR));
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Failed to set the ProfileImagePath value (Error %lu)\n", dwError);
//         goto done;
//     }
//
//     /* Expand the raw profile path */
//     if (!ExpandEnvironmentStringsW(szRawProfilePath,
//                                    szProfilePath,
//                                    ARRAYSIZE(szProfilePath)))
//     {
//         DPRINT1("Failled to expand the raw profile path (Error %lu)\n", GetLastError());
//         goto done;
//     }
//
//     /* Create the profile directory if it does not exist yet */
//     // FIXME: Security!
//     if (!CreateDirectoryW(szProfilePath, NULL))
//     {
//         if (GetLastError() != ERROR_ALREADY_EXISTS)
//         {
//             DPRINT1("Failed to create the profile directory (Error %lu)\n", GetLastError());
//             goto done;
//         }
//     }
//
//     /* Get the path of the default profile */
//     cchSize = ARRAYSIZE(szDefaultProfilePath);
//     if (!GetDefaultUserProfileDirectoryW(szDefaultProfilePath, &cchSize))
//     {
//         DPRINT1("Failed to create the default profile path (Error %lu)\n", GetLastError());
//         goto done;
//     }
//
//     /* Copy the default profile into the new profile directory */
//     // FIXME: Security!
//     if (!CopyDirectory(szProfilePath, szDefaultProfilePath))
//     {
//         DPRINT1("Failed to copy the default profile directory (Error %lu)\n", GetLastError());
//         goto done;
//     }
//
//     bResult = TRUE;
//
// done:
//     if (hProfileKey != NULL)
//         RegCloseKey(hProfileKey);
//
//     RtlFreeUnicodeString(&SidString);
//
//     if (pUserSid != NULL)
//         LocalFree(pUserSid);
//
//     if (hToken != NULL)
//         CloseHandle(hToken);
//
//     return bResult;
// }
//
//
// BOOL
// WINAPI
// CreateUserProfileA(
//      PSID pSid,
//      LPCSTR lpUserName)
// {
//     LPWSTR pUserNameW = NULL;
//     INT nLength;
//     BOOL bResult;
//
//     DPRINT("CreateUserProfileA(%p %s)\n", pSid, lpUserName);
//
//     /* Convert lpUserName to Unicode */
//     nLength = MultiByteToWideChar(CP_ACP, 0, lpUserName, -1, NULL, 0);
//     pUserNameW = HeapAlloc(GetProcessHeap(), 0, nLength * sizeof(WCHAR));
//     if (pUserNameW == NULL)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//     MultiByteToWideChar(CP_ACP, 0, lpUserName, -1, pUserNameW, nLength);
//
//     /* Call the Ex function */
//     bResult = CreateUserProfileExW(pSid,
//                                    pUserNameW,
//                                    NULL,
//                                    NULL,
//                                    0,
//                                    FALSE);
//
//     HeapFree(GetProcessHeap(), 0, pUserNameW);
//
//     return bResult;
// }
//
//
// BOOL
// WINAPI
// CreateUserProfileW(
//      PSID pSid,
//      LPCWSTR lpUserName)
// {
//     DPRINT("CreateUserProfileW(%p %S)\n", pSid, lpUserName);
//
//     /* Call the Ex function */
//     return CreateUserProfileExW(pSid,
//                                 lpUserName,
//                                 NULL,
//                                 NULL,
//                                 0,
//                                 FALSE);
// }
//
//
// BOOL
// WINAPI
// CreateUserProfileExA(
//      PSID pSid,
//      LPCSTR lpUserName,
//      LPCSTR lpUserHive,
//      LPSTR lpProfileDir,
//      DWORD dwDirSize,
//      BOOL bWin9xUpg)
// {
//     LPWSTR pUserNameW = NULL;
//     LPWSTR pUserHiveW = NULL;
//     LPWSTR pProfileDirW = NULL;
//     INT nLength;
//     BOOL bResult = FALSE;
//
//     DPRINT("CreateUserProfileExA(%p %s %s %p %lu %d)\n",
//            pSid, lpUserName, lpUserHive, lpProfileDir, dwDirSize, bWin9xUpg);
//
//     /* Check the parameters */
//     if (lpProfileDir != NULL && dwDirSize == 0)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* Convert lpUserName to Unicode */
//     nLength = MultiByteToWideChar(CP_ACP, 0, lpUserName, -1, NULL, 0);
//     pUserNameW = HeapAlloc(GetProcessHeap(), 0, nLength * sizeof(WCHAR));
//     if (pUserNameW == NULL)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         goto done;
//     }
//     MultiByteToWideChar(CP_ACP, 0, lpUserName, -1, pUserNameW, nLength);
//
//     /* Convert lpUserHive to Unicode */
//     if (lpUserHive != NULL)
//     {
//         nLength = MultiByteToWideChar(CP_ACP, 0, lpUserHive, -1, NULL, 0);
//         pUserHiveW = HeapAlloc(GetProcessHeap(), 0, nLength * sizeof(WCHAR));
//         if (pUserHiveW == NULL)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto done;
//         }
//         MultiByteToWideChar(CP_ACP, 0, lpUserHive, -1, pUserHiveW, nLength);
//     }
//
//     /* Allocate a Unicode buffer for lpProfileDir */
//     if (lpProfileDir != NULL)
//     {
//         pProfileDirW = HeapAlloc(GetProcessHeap(), 0, dwDirSize * sizeof(WCHAR));
//         if (pProfileDirW == NULL)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto done;
//         }
//     }
//
//     /* Call the Unicode function */
//     bResult = CreateUserProfileExW(pSid,
//                                    (LPCWSTR)pUserNameW,
//                                    (LPCWSTR)pUserHiveW,
//                                    pProfileDirW,
//                                    dwDirSize,
//                                    bWin9xUpg);
//
//     /* Convert the profile path to ANSI */
//     if (bResult && lpProfileDir != NULL)
//     {
//         WideCharToMultiByte(CP_ACP, 0, pProfileDirW, -1, lpProfileDir, dwDirSize, NULL, NULL);
//     }
//
// done:
//     /* Free the buffers */
//     if (pProfileDirW != NULL)
//         HeapFree(GetProcessHeap(), 0, pProfileDirW);
//
//     if (pUserHiveW != NULL)
//         HeapFree(GetProcessHeap(), 0, pUserHiveW);
//
//     if (pUserNameW != NULL)
//         HeapFree(GetProcessHeap(), 0, pUserNameW);
//
//     return bResult;
// }
//
//
// BOOL
// WINAPI
// CreateUserProfileExW(
//      PSID pSid,
//      LPCWSTR lpUserName,
//      LPCWSTR lpUserHive,
//      LPWSTR lpProfileDir,
//      DWORD dwDirSize,
//      BOOL bWin9xUpg)
// {
//     WCHAR szRawProfilesPath[MAX_PATH];
//     WCHAR szProfilesPath[MAX_PATH];
//     WCHAR szUserProfilePath[MAX_PATH];
//     WCHAR szDefaultUserPath[MAX_PATH];
//     WCHAR szUserProfileName[MAX_PATH];
//     WCHAR szBuffer[MAX_PATH];
//     LPWSTR SidString;
//     DWORD dwType, dwLength;
//     DWORD dwDisposition;
//     UINT i;
//     HKEY hKey;
//     BOOL bRet = TRUE;
//     LONG Error;
//
//     DPRINT("CreateUserProfileExW(%p %S %S %p %lu %d)\n",
//            pSid, lpUserName, lpUserHive, lpProfileDir, dwDirSize, bWin9xUpg);
//
//     /* Parameters validation */
//     if (!pSid || !lpUserName)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /*
//      * TODO:
//      *  - Add support for lpUserHive.
//      *  - bWin9xUpg is obsolete. Don't waste your time implementing this.
//      */
//
//     Error = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                           L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList",
//                           0,
//                           KEY_QUERY_VALUE,
//                           &hKey);
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", Error);
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     /* Get profiles path */
//     dwLength = sizeof(szRawProfilesPath);
//     Error = RegQueryValueExW(hKey,
//                              L"ProfilesDirectory",
//                              NULL,
//                              &dwType,
//                              (LPBYTE)szRawProfilesPath,
//                              &dwLength);
//     if ((Error != ERROR_SUCCESS) || (dwType != REG_SZ && dwType != REG_EXPAND_SZ))
//     {
//         DPRINT1("Error: %lu\n", Error);
//         RegCloseKey(hKey);
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     /* Expand it */
//     if (!ExpandEnvironmentStringsW(szRawProfilesPath,
//                                    szProfilesPath,
//                                    ARRAYSIZE(szProfilesPath)))
//     {
//         DPRINT1("Error: %lu\n", GetLastError());
//         RegCloseKey(hKey);
//         return FALSE;
//     }
//
//     /* Create the profiles directory if it does not exist yet */
//     // FIXME: Security!
//     if (!CreateDirectoryW(szProfilesPath, NULL))
//     {
//         if (GetLastError() != ERROR_ALREADY_EXISTS)
//         {
//             DPRINT1("Error: %lu\n", GetLastError());
//             return FALSE;
//         }
//     }
//
//     /* Get default user path */
//     dwLength = sizeof(szBuffer);
//     Error = RegQueryValueExW(hKey,
//                              L"DefaultUserProfile",
//                              NULL,
//                              &dwType,
//                              (LPBYTE)szBuffer,
//                              &dwLength);
//     if ((Error != ERROR_SUCCESS) || (dwType != REG_SZ && dwType != REG_EXPAND_SZ))
//     {
//         DPRINT1("Error: %lu\n", Error);
//         RegCloseKey(hKey);
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     RegCloseKey(hKey);
//
//     StringCbCopyW(szUserProfileName, sizeof(szUserProfileName), lpUserName);
//
//     /* Create user profile directory */
//
//     StringCbCopyW(szUserProfilePath, sizeof(szUserProfilePath), szProfilesPath);
//     StringCbCatW(szUserProfilePath, sizeof(szUserProfilePath), L"\\");
//     StringCbCatW(szUserProfilePath, sizeof(szUserProfilePath), szUserProfileName);
//
//     // FIXME: Security!
//     if (!CreateDirectoryW(szUserProfilePath, NULL))
//     {
//         if (GetLastError() != ERROR_ALREADY_EXISTS)
//         {
//             DPRINT1("Error: %lu\n", GetLastError());
//             return FALSE;
//         }
//
//         for (i = 0; i < 1000; i++)
//         {
//             swprintf(szUserProfileName, L"%s.%03u", lpUserName, i);
//
//             StringCbCopyW(szUserProfilePath, sizeof(szUserProfilePath), szProfilesPath);
//             StringCbCatW(szUserProfilePath, sizeof(szUserProfilePath), L"\\");
//             StringCbCatW(szUserProfilePath, sizeof(szUserProfilePath), szUserProfileName);
//
//             // FIXME: Security!
//             if (CreateDirectoryW(szUserProfilePath, NULL))
//                 break;
//
//             if (GetLastError() != ERROR_ALREADY_EXISTS)
//             {
//                 DPRINT1("Error: %lu\n", GetLastError());
//                 return FALSE;
//             }
//         }
//     }
//
//     /* Copy default user directory */
//
//     StringCbCopyW(szDefaultUserPath, sizeof(szDefaultUserPath), szProfilesPath);
//     StringCbCatW(szDefaultUserPath, sizeof(szDefaultUserPath), L"\\");
//     StringCbCatW(szDefaultUserPath, sizeof(szDefaultUserPath), szBuffer);
//
//     // FIXME: Security!
//     if (!CopyDirectory(szUserProfilePath, szDefaultUserPath))
//     {
//         DPRINT1("Error: %lu\n", GetLastError());
//         return FALSE;
//     }
//
//     /* Add profile to profile list */
//     if (!ConvertSidToStringSidW(pSid,
//                                 &SidString))
//     {
//         DPRINT1("Error: %lu\n", GetLastError());
//         return FALSE;
//     }
//
//     StringCbCopyW(szBuffer, sizeof(szBuffer),
//                   L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList\\");
//     StringCbCatW(szBuffer, sizeof(szBuffer), SidString);
//
//     /* Create user profile key */
//     Error = RegCreateKeyExW(HKEY_LOCAL_MACHINE,
//                             szBuffer,
//                             0,
//                             NULL,
//                             REG_OPTION_NON_VOLATILE,
//                             KEY_ALL_ACCESS,
//                             NULL,
//                             &hKey,
//                             &dwDisposition);
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", Error);
//         bRet = FALSE;
//         goto done;
//     }
//
//     /* Create non-expanded user profile path */
//     StringCbCopyW(szBuffer, sizeof(szBuffer), szRawProfilesPath);
//     StringCbCatW(szBuffer, sizeof(szBuffer), L"\\");
//     StringCbCatW(szBuffer, sizeof(szBuffer), szUserProfileName);
//
//     /* Set 'ProfileImagePath' value (non-expanded) */
//     Error = RegSetValueExW(hKey,
//                            L"ProfileImagePath",
//                            0,
//                            REG_EXPAND_SZ,
//                            (LPBYTE)szBuffer,
//                            (wcslen(szBuffer) + 1) * sizeof(WCHAR));
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", Error);
//         RegCloseKey(hKey);
//         bRet = FALSE;
//         goto done;
//     }
//
//     /* Set 'Sid' value */
//     Error = RegSetValueExW(hKey,
//                            L"Sid",
//                            0,
//                            REG_BINARY,
//                            pSid,
//                            GetLengthSid(pSid));
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", Error);
//         RegCloseKey(hKey);
//         bRet = FALSE;
//         goto done;
//     }
//
//     RegCloseKey(hKey);
//
//     /* Create user hive file */
//
//     /* Use the default hive file name */
//     StringCbCopyW(szBuffer, sizeof(szBuffer), szUserProfilePath);
//     StringCbCatW(szBuffer, sizeof(szBuffer), L"\\ntuser.dat");
//
//     /* Acquire restore privilege */
//     if (!AcquireRemoveRestorePrivilege(TRUE))
//     {
//         Error = GetLastError();
//         DPRINT1("Error: %lu\n", Error);
//         bRet = FALSE;
//         goto done;
//     }
//
//     /* Load the user hive */
//     Error = RegLoadKeyW(HKEY_USERS,
//                         SidString,
//                         szBuffer);
//     AcquireRemoveRestorePrivilege(FALSE);
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", Error);
//         bRet = FALSE;
//         goto done;
//     }
//
//     /* Initialize user hive */
//     if (!CreateUserHive(SidString, szUserProfilePath))
//     {
//         Error = GetLastError();
//         DPRINT1("Error: %lu\n", Error);
//         bRet = FALSE;
//     }
//
//     /* Unload the hive */
//     AcquireRemoveRestorePrivilege(TRUE);
//     RegUnLoadKeyW(HKEY_USERS, SidString);
//     AcquireRemoveRestorePrivilege(FALSE);
//
//     /*
//      * If the caller wants to retrieve the user profile path,
//      * give it now. 'dwDirSize' is the number of characters.
//      */
//     if (lpProfileDir && dwDirSize)
//         StringCchCopyW(lpProfileDir, dwDirSize, szUserProfilePath);
//
// done:
//     LocalFree((HLOCAL)SidString);
//     SetLastError((DWORD)Error);
//
//     DPRINT("CreateUserProfileExW() done\n");
//
//     return bRet;
// }
//
//
// BOOL
// WINAPI
// DeleteProfileA(
//      LPCSTR lpSidString,
//      LPCSTR lpProfilePath,
//      LPCSTR lpComputerName)
// {
//     BOOL bResult;
//     UNICODE_STRING SidString = {0, 0, NULL}, ProfilePath = {0, 0, NULL}, ComputerName = {0, 0, NULL};
//
//     DPRINT("DeleteProfileA() called\n");
//
//     /* Conversion to UNICODE */
//     if (lpSidString)
//         RtlCreateUnicodeStringFromAsciiz(&SidString,
//                                          (LPSTR)lpSidString);
//
//     if (lpProfilePath)
//         RtlCreateUnicodeStringFromAsciiz(&ProfilePath,
//                                          (LPSTR)lpProfilePath);
//
//     if (lpComputerName)
//         RtlCreateUnicodeStringFromAsciiz(&ComputerName,
//                                          (LPSTR)lpComputerName);
//
//     /* Call the UNICODE function */
//     bResult = DeleteProfileW(SidString.Buffer,
//                              ProfilePath.Buffer,
//                              ComputerName.Buffer);
//
//     /* Memory cleanup */
//     if (lpSidString)
//         RtlFreeUnicodeString(&SidString);
//
//     if (lpProfilePath)
//         RtlFreeUnicodeString(&ProfilePath);
//
//     if (lpComputerName)
//         RtlFreeUnicodeString(&ComputerName);
//
//     return bResult;
// }
//
//
// BOOL
// WINAPI
// DeleteProfileW(
//      LPCWSTR lpSidString,
//      LPCWSTR lpProfilePath,
//      LPCWSTR lpComputerName)
// {
//     HKEY hProfilesKey = NULL, hProfileKey = NULL;
//     WCHAR szRawProfilePath[MAX_PATH], szProfilePath[MAX_PATH];
//     DWORD dwLength, dwType;
//     DWORD dwError = ERROR_SUCCESS;
//     BOOL bRet = FALSE;
//
//     DPRINT("DeleteProfileW(%S %S %S)\n", lpSidString, lpProfilePath, lpComputerName);
//
//     if (lpSidString == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     if (lpProfilePath != NULL)
//     {
//         dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                                 L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList",
//                                 0,
//                                 KEY_QUERY_VALUE,
//                                 &hProfilesKey);
//         if (dwError != ERROR_SUCCESS)
//         {
//             DPRINT1("Error: %lu\n", dwError);
//             goto done;
//         }
//
//         dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                                 lpSidString,
//                                 0,
//                                 KEY_QUERY_VALUE,
//                                 &hProfileKey);
//         if (dwError != ERROR_SUCCESS)
//         {
//             DPRINT1("Error: %lu\n", dwError);
//             goto done;
//         }
//
//         /* Get the profile image path */
//         dwLength = sizeof(szRawProfilePath);
//         dwError = RegQueryValueExW(hProfileKey,
//                                    L"ProfileImagePath",
//                                    NULL,
//                                    &dwType,
//                                    (PBYTE)szRawProfilePath,
//                                    &dwLength);
//         if (dwError != ERROR_SUCCESS)
//         {
//              DPRINT1("Error: %lu\n", dwError);
//              goto done;
//         }
//
//         if (!ExpandEnvironmentStringsW(szRawProfilePath,
//                                        szProfilePath,
//                                        ARRAYSIZE(szProfilePath)))
//         {
//             dwError = GetLastError();
//             DPRINT1("Failled to expand the raw profile path (Error %lu)\n", dwError);
//             goto done;
//         }
//     }
//     else
//     {
//         StringCchCopyW(szProfilePath, ARRAYSIZE(szProfilePath), lpProfilePath);
//     }
//
//     DPRINT("ProfilePath: %S\n", szProfilePath);
//
//     if (!RemoveDirectoryPath(szProfilePath))
//     {
//         dwError = GetLastError();
//         DPRINT1("Error: %lu\n", dwError);
//         goto done;
//     }
//
//     bRet = TRUE;
//
// done:
//     if (hProfileKey != NULL)
//         RegCloseKey(hProfileKey);
//
//     if (hProfilesKey != NULL)
//         RegCloseKey(hProfilesKey);
//
//     return bRet;
// }
//
//
// BOOL
// WINAPI
// GetAllUsersProfileDirectoryA(
//      LPSTR lpProfileDir,
//      LPDWORD lpcchSize)
// {
//     LPWSTR lpBuffer;
//     BOOL bResult;
//
//     if (!lpcchSize)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     lpBuffer = GlobalAlloc(GMEM_FIXED,
//                            *lpcchSize * sizeof(WCHAR));
//     if (lpBuffer == NULL)
//         return FALSE;
//
//     bResult = GetAllUsersProfileDirectoryW(lpBuffer,
//                                            lpcchSize);
//     if (bResult && lpProfileDir)
//     {
//         bResult = WideCharToMultiByte(CP_ACP,
//                                       0,
//                                       lpBuffer,
//                                       -1,
//                                       lpProfileDir,
//                                       *lpcchSize,
//                                       NULL,
//                                       NULL);
//     }
//
//     GlobalFree(lpBuffer);
//
//     return bResult;
// }
//
//
// BOOL
// WINAPI
// GetAllUsersProfileDirectoryW(
//      LPWSTR lpProfileDir,
//      LPDWORD lpcchSize)
// {
//     WCHAR szProfilePath[MAX_PATH];
//     WCHAR szBuffer[MAX_PATH];
//     DWORD dwType, dwLength;
//     HKEY hKey;
//     LONG Error;
//
//     if (!lpcchSize)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     Error = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                           L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList",
//                           0,
//                           KEY_QUERY_VALUE,
//                           &hKey);
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", Error);
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     /* Get profiles path */
//     dwLength = sizeof(szBuffer);
//     Error = RegQueryValueExW(hKey,
//                              L"ProfilesDirectory",
//                              NULL,
//                              &dwType,
//                              (LPBYTE)szBuffer,
//                              &dwLength);
//     if ((Error != ERROR_SUCCESS) || (dwType != REG_SZ && dwType != REG_EXPAND_SZ))
//     {
//         DPRINT1("Error: %lu\n", Error);
//         RegCloseKey(hKey);
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     /* Expand it */
//     if (!ExpandEnvironmentStringsW(szBuffer,
//                                    szProfilePath,
//                                    ARRAYSIZE(szProfilePath)))
//     {
//         DPRINT1("Error: %lu\n", GetLastError());
//         RegCloseKey(hKey);
//         return FALSE;
//     }
//
//     /* Get 'AllUsersProfile' name */
//     dwLength = sizeof(szBuffer);
//     Error = RegQueryValueExW(hKey,
//                              L"AllUsersProfile",
//                              NULL,
//                              &dwType,
//                              (LPBYTE)szBuffer,
//                              &dwLength);
//     if ((Error != ERROR_SUCCESS) || (dwType != REG_SZ && dwType != REG_EXPAND_SZ))
//     {
//         DPRINT1("Error: %lu\n", Error);
//         RegCloseKey(hKey);
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     RegCloseKey(hKey);
//
//     StringCbCatW(szProfilePath, sizeof(szProfilePath), L"\\");
//     StringCbCatW(szProfilePath, sizeof(szProfilePath), szBuffer);
//
//     dwLength = wcslen(szProfilePath) + 1;
//     if (lpProfileDir && (*lpcchSize >= dwLength))
//     {
//         StringCchCopyW(lpProfileDir, *lpcchSize, szProfilePath);
//         *lpcchSize = dwLength;
//         return TRUE;
//     }
//     else // if (!lpProfileDir || (*lpcchSize < dwLength))
//     {
//         *lpcchSize = dwLength;
//         SetLastError(ERROR_INSUFFICIENT_BUFFER);
//         return FALSE;
//     }
// }
//
//
// BOOL
// WINAPI
// GetDefaultUserProfileDirectoryA(
//      LPSTR lpProfileDir,
//      LPDWORD lpcchSize)
// {
//     LPWSTR lpBuffer;
//     BOOL bResult;
//
//     if (!lpcchSize)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     lpBuffer = GlobalAlloc(GMEM_FIXED,
//                            *lpcchSize * sizeof(WCHAR));
//     if (lpBuffer == NULL)
//         return FALSE;
//
//     bResult = GetDefaultUserProfileDirectoryW(lpBuffer,
//                                               lpcchSize);
//     if (bResult && lpProfileDir)
//     {
//         bResult = WideCharToMultiByte(CP_ACP,
//                                       0,
//                                       lpBuffer,
//                                       -1,
//                                       lpProfileDir,
//                                       *lpcchSize,
//                                       NULL,
//                                       NULL);
//     }
//
//     GlobalFree(lpBuffer);
//
//     return bResult;
// }
//
//
// BOOL
// WINAPI
// GetDefaultUserProfileDirectoryW(
//      LPWSTR lpProfileDir,
//      LPDWORD lpcchSize)
// {
//     WCHAR szProfilePath[MAX_PATH];
//     WCHAR szBuffer[MAX_PATH];
//     DWORD dwType, dwLength;
//     HKEY hKey;
//     LONG Error;
//
//     if (!lpcchSize)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     Error = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                           L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList",
//                           0,
//                           KEY_QUERY_VALUE,
//                           &hKey);
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", Error);
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     /* Get profiles path */
//     dwLength = sizeof(szBuffer);
//     Error = RegQueryValueExW(hKey,
//                              L"ProfilesDirectory",
//                              NULL,
//                              &dwType,
//                              (LPBYTE)szBuffer,
//                              &dwLength);
//     if ((Error != ERROR_SUCCESS) || (dwType != REG_SZ && dwType != REG_EXPAND_SZ))
//     {
//         DPRINT1("Error: %lu\n", Error);
//         RegCloseKey(hKey);
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     /* Expand it */
//     if (!ExpandEnvironmentStringsW(szBuffer,
//                                    szProfilePath,
//                                    ARRAYSIZE(szProfilePath)))
//     {
//         DPRINT1("Error: %lu\n", GetLastError());
//         RegCloseKey(hKey);
//         return FALSE;
//     }
//
//     /* Get 'DefaultUserProfile' name */
//     dwLength = sizeof(szBuffer);
//     Error = RegQueryValueExW(hKey,
//                              L"DefaultUserProfile",
//                              NULL,
//                              &dwType,
//                              (LPBYTE)szBuffer,
//                              &dwLength);
//     if ((Error != ERROR_SUCCESS) || (dwType != REG_SZ && dwType != REG_EXPAND_SZ))
//     {
//         DPRINT1("Error: %lu\n", Error);
//         RegCloseKey(hKey);
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     RegCloseKey(hKey);
//
//     StringCbCatW(szProfilePath, sizeof(szProfilePath), L"\\");
//     StringCbCatW(szProfilePath, sizeof(szProfilePath), szBuffer);
//
//     dwLength = wcslen(szProfilePath) + 1;
//     if (lpProfileDir && (*lpcchSize >= dwLength))
//     {
//         StringCchCopyW(lpProfileDir, *lpcchSize, szProfilePath);
//         *lpcchSize = dwLength;
//         return TRUE;
//     }
//     else // if (!lpProfileDir || (*lpcchSize < dwLength))
//     {
//         *lpcchSize = dwLength;
//         SetLastError(ERROR_INSUFFICIENT_BUFFER);
//         return FALSE;
//     }
// }
//
//
// BOOL
// WINAPI
// GetProfilesDirectoryA(
//      LPSTR lpProfileDir, // 
//      LPDWORD lpcchSize)
// {
//     LPWSTR lpBuffer;
//     BOOL bResult;
//
//     if (!lpcchSize || !lpProfileDir)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     lpBuffer = GlobalAlloc(GMEM_FIXED,
//                            *lpcchSize * sizeof(WCHAR));
//     if (lpBuffer == NULL)
//         return FALSE;
//
//     bResult = GetProfilesDirectoryW(lpBuffer,
//                                     lpcchSize);
//     if (bResult && lpProfileDir)
//     {
//         bResult = WideCharToMultiByte(CP_ACP,
//                                       0,
//                                       lpBuffer,
//                                       -1,
//                                       lpProfileDir,
//                                       *lpcchSize,
//                                       NULL,
//                                       NULL);
//     }
//
//     GlobalFree(lpBuffer);
//
//     return bResult;
// }
//
//
// BOOL
// WINAPI
// GetProfilesDirectoryW(
//      LPWSTR lpProfilesDir, // 
//      LPDWORD lpcchSize)
// {
//     WCHAR szProfilesPath[MAX_PATH];
//     WCHAR szBuffer[MAX_PATH];
//     DWORD dwType, dwLength;
//     HKEY hKey;
//     LONG Error;
//
//     if (!lpcchSize)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     Error = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                           L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList",
//                           0,
//                           KEY_QUERY_VALUE,
//                           &hKey);
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", Error);
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     /* Get profiles path */
//     dwLength = sizeof(szBuffer);
//     Error = RegQueryValueExW(hKey,
//                              L"ProfilesDirectory",
//                              NULL,
//                              &dwType,
//                              (LPBYTE)szBuffer,
//                              &dwLength);
//     if ((Error != ERROR_SUCCESS) || (dwType != REG_SZ && dwType != REG_EXPAND_SZ))
//     {
//         DPRINT1("Error: %lu\n", Error);
//         RegCloseKey(hKey);
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     RegCloseKey(hKey);
//
//     /* Expand it */
//     if (!ExpandEnvironmentStringsW(szBuffer,
//                                    szProfilesPath,
//                                    ARRAYSIZE(szProfilesPath)))
//     {
//         DPRINT1("Error: %lu\n", GetLastError());
//         return FALSE;
//     }
//
//     dwLength = wcslen(szProfilesPath) + 1;
//     if (lpProfilesDir && (*lpcchSize >= dwLength))
//     {
//         StringCchCopyW(lpProfilesDir, *lpcchSize, szProfilesPath);
//         *lpcchSize = dwLength;
//         return TRUE;
//     }
//     else // if (!lpProfilesDir || (*lpcchSize < dwLength))
//     {
//         *lpcchSize = dwLength;
//         SetLastError(ERROR_INSUFFICIENT_BUFFER);
//         return FALSE;
//     }
// }
//
//
// BOOL
// WINAPI
// GetProfileType(
//      PDWORD pdwFlags)
// {
//     UNICODE_STRING SidString = {0, 0, NULL};
//     HANDLE hToken;
//     HKEY hProfilesKey = NULL, hProfileKey = NULL;
//     DWORD dwType, dwLength, dwState = 0;
//     DWORD dwError;
//     BOOL bResult = FALSE;
//
//     DPRINT("GetProfileType(%p)\n", pdwFlags);
//
//     if (pdwFlags == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     if (!OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, TRUE, &hToken))
//     {
//         if (!OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &hToken))
//         {
//             DPRINT1("Failed to open a token (Error %lu)\n", GetLastError());
//             return FALSE;
//         }
//     }
//
//     /* Get the user SID string */
//     if (!GetUserSidStringFromToken(hToken, &SidString))
//     {
//         DPRINT1("GetUserSidStringFromToken() failed\n");
//         goto done;
//     }
//
//     DPRINT("SID: %wZ\n", &SidString);
//
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList",
//                             0,
//                             KEY_QUERY_VALUE,
//                             &hProfilesKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", dwError);
//         SetLastError(dwError);
//         goto done;
//     }
//
//     dwError = RegOpenKeyExW(hProfilesKey,
//                             SidString.Buffer,
//                             0,
//                             KEY_QUERY_VALUE | KEY_SET_VALUE,
//                             &hProfileKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", dwError);
//         SetLastError(dwError);
//         goto done;
//     }
//
//     /* Get the State value */
//     dwLength = sizeof(dwState);
//     dwError = RegQueryValueExW(hProfileKey,
//                                L"State",
//                                NULL,
//                                &dwType,
//                                (PBYTE)&dwState,
//                                &dwLength);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", dwError);
//         SetLastError(dwError);
//         goto done;
//     }
//
//     *pdwFlags = 0;
//
//     if (dwState & 0x80) /* PROFILE_GUEST_USER */
//         *pdwFlags |= PT_TEMPORARY;
//
//     /* FIXME: Add checks for PT_MANDATORY and PT_ROAMING */
//
//     bResult = TRUE;
//
// done:
//     if (hProfileKey != NULL)
//         RegCloseKey(hProfileKey);
//
//     if (hProfilesKey != NULL)
//         RegCloseKey(hProfilesKey);
//
//     RtlFreeUnicodeString(&SidString);
//
//     CloseHandle(hToken);
//
//     return bResult;
// }
//
//
// BOOL
// WINAPI
// GetUserProfileDirectoryA(
//      HANDLE hToken,
//      LPSTR lpProfileDir,
//      LPDWORD lpcchSize)
// {
//     LPWSTR lpBuffer;
//     BOOL bResult;
//
//     if (!lpcchSize || !lpProfileDir)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     lpBuffer = GlobalAlloc(GMEM_FIXED,
//                            *lpcchSize * sizeof(WCHAR));
//     if (lpBuffer == NULL)
//         return FALSE;
//
//     bResult = GetUserProfileDirectoryW(hToken,
//                                        lpBuffer,
//                                        lpcchSize);
//     if (bResult && lpProfileDir)
//     {
//         bResult = WideCharToMultiByte(CP_ACP,
//                                       0,
//                                       lpBuffer,
//                                       -1,
//                                       lpProfileDir,
//                                       *lpcchSize,
//                                       NULL,
//                                       NULL);
//     }
//
//     GlobalFree(lpBuffer);
//
//     return bResult;
// }
//
//
// BOOL
// WINAPI
// GetUserProfileDirectoryW(
//      HANDLE hToken,
//      LPWSTR lpProfileDir,
//      LPDWORD lpcchSize)
// {
//     UNICODE_STRING SidString;
//     WCHAR szKeyName[MAX_PATH];
//     WCHAR szRawImagePath[MAX_PATH];
//     WCHAR szImagePath[MAX_PATH];
//     DWORD dwType, dwLength;
//     HKEY hKey;
//     LONG Error;
//
//     if (!hToken)
//     {
//         SetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     if (!lpcchSize)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* Get the user SID string */
//     if (!GetUserSidStringFromToken(hToken, &SidString))
//     {
//         DPRINT1("GetUserSidStringFromToken() failed\n");
//         return FALSE;
//     }
//
//     DPRINT("SidString: '%wZ'\n", &SidString);
//
//     StringCbCopyW(szKeyName, sizeof(szKeyName),
//                   L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList\\");
//     StringCbCatW(szKeyName, sizeof(szKeyName), SidString.Buffer);
//
//     RtlFreeUnicodeString(&SidString);
//
//     DPRINT("KeyName: '%S'\n", szKeyName);
//
//     Error = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                           szKeyName,
//                           0,
//                           KEY_QUERY_VALUE,
//                           &hKey);
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", Error);
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     dwLength = sizeof(szRawImagePath);
//     Error = RegQueryValueExW(hKey,
//                              L"ProfileImagePath",
//                              NULL,
//                              &dwType,
//                              (LPBYTE)szRawImagePath,
//                              &dwLength);
//     if ((Error != ERROR_SUCCESS) || (dwType != REG_SZ && dwType != REG_EXPAND_SZ))
//     {
//         DPRINT1("Error: %lu\n", Error);
//         RegCloseKey(hKey);
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     RegCloseKey(hKey);
//
//     DPRINT("RawImagePath: '%S'\n", szRawImagePath);
//
//     /* Expand it */
//     if (!ExpandEnvironmentStringsW(szRawImagePath,
//                                    szImagePath,
//                                    ARRAYSIZE(szImagePath)))
//     {
//         DPRINT1("Error: %lu\n", GetLastError());
//         return FALSE;
//     }
//
//     DPRINT("ImagePath: '%S'\n", szImagePath);
//
//     dwLength = wcslen(szImagePath) + 1;
//     if (lpProfileDir && (*lpcchSize >= dwLength))
//     {
//         StringCchCopyW(lpProfileDir, *lpcchSize, szImagePath);
//         *lpcchSize = dwLength;
//         return TRUE;
//     }
//     else // if (!lpProfileDir || (*lpcchSize < dwLength))
//     {
//         *lpcchSize = dwLength;
//         SetLastError(ERROR_INSUFFICIENT_BUFFER);
//         return FALSE;
//     }
// }
//
//
// BOOL
// WINAPI
// LoadUserProfileA(
//      HANDLE hToken,
//      LPPROFILEINFOA lpProfileInfo)
// {
//     BOOL bResult = FALSE;
//     PROFILEINFOW ProfileInfoW = {0};
//     int len;
//
//     DPRINT("LoadUserProfileA() called\n");
//
//     /* Check profile info */
//     if (!lpProfileInfo || (lpProfileInfo->dwSize != sizeof(PROFILEINFOA)) ||
//         (lpProfileInfo->lpUserName == NULL) || (lpProfileInfo->lpUserName[0] == 0))
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* Convert the structure to UNICODE... */
//     ProfileInfoW.dwSize = sizeof(ProfileInfoW);
//     ProfileInfoW.dwFlags = lpProfileInfo->dwFlags;
//
//     if (lpProfileInfo->lpUserName)
//     {
//         len = MultiByteToWideChar(CP_ACP, 0, lpProfileInfo->lpUserName, -1, NULL, 0);
//         ProfileInfoW.lpUserName = HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR));
//         if (!ProfileInfoW.lpUserName)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto cleanup;
//         }
//         MultiByteToWideChar(CP_ACP, 0, lpProfileInfo->lpUserName, -1, ProfileInfoW.lpUserName, len);
//     }
//
//     if (lpProfileInfo->lpProfilePath)
//     {
//         len = MultiByteToWideChar(CP_ACP, 0, lpProfileInfo->lpProfilePath, -1, NULL, 0);
//         ProfileInfoW.lpProfilePath = HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR));
//         if (!ProfileInfoW.lpProfilePath)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto cleanup;
//         }
//         MultiByteToWideChar(CP_ACP, 0, lpProfileInfo->lpProfilePath, -1, ProfileInfoW.lpProfilePath, len);
//     }
//
//     if (lpProfileInfo->lpDefaultPath)
//     {
//         len = MultiByteToWideChar(CP_ACP, 0, lpProfileInfo->lpDefaultPath, -1, NULL, 0);
//         ProfileInfoW.lpDefaultPath = HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR));
//         if (!ProfileInfoW.lpDefaultPath)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto cleanup;
//         }
//         MultiByteToWideChar(CP_ACP, 0, lpProfileInfo->lpDefaultPath, -1, ProfileInfoW.lpDefaultPath, len);
//     }
//
//     if (lpProfileInfo->lpServerName)
//     {
//         len = MultiByteToWideChar(CP_ACP, 0, lpProfileInfo->lpServerName, -1, NULL, 0);
//         ProfileInfoW.lpServerName = HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR));
//         if (!ProfileInfoW.lpServerName)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto cleanup;
//         }
//         MultiByteToWideChar(CP_ACP, 0, lpProfileInfo->lpServerName, -1, ProfileInfoW.lpServerName, len);
//     }
//
//     if ((ProfileInfoW.dwFlags & PI_APPLYPOLICY) != 0 && lpProfileInfo->lpPolicyPath)
//     {
//         len = MultiByteToWideChar(CP_ACP, 0, lpProfileInfo->lpPolicyPath, -1, NULL, 0);
//         ProfileInfoW.lpPolicyPath = HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR));
//         if (!ProfileInfoW.lpPolicyPath)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto cleanup;
//         }
//         MultiByteToWideChar(CP_ACP, 0, lpProfileInfo->lpPolicyPath, -1, ProfileInfoW.lpPolicyPath, len);
//     }
//
//     /* ... and call the UNICODE function */
//     bResult = LoadUserProfileW(hToken, &ProfileInfoW);
//
//     /* Save the returned value */
//     lpProfileInfo->hProfile = ProfileInfoW.hProfile;
//
// cleanup:
//     /* Memory cleanup */
//     if (ProfileInfoW.lpUserName)
//         HeapFree(GetProcessHeap(), 0, ProfileInfoW.lpUserName);
//
//     if (ProfileInfoW.lpProfilePath)
//         HeapFree(GetProcessHeap(), 0, ProfileInfoW.lpProfilePath);
//
//     if (ProfileInfoW.lpDefaultPath)
//         HeapFree(GetProcessHeap(), 0, ProfileInfoW.lpDefaultPath);
//
//     if (ProfileInfoW.lpServerName)
//         HeapFree(GetProcessHeap(), 0, ProfileInfoW.lpServerName);
//
//     if ((ProfileInfoW.dwFlags & PI_APPLYPOLICY) != 0 && ProfileInfoW.lpPolicyPath)
//         HeapFree(GetProcessHeap(), 0, ProfileInfoW.lpPolicyPath);
//
//     return bResult;
// }
//
//
// BOOL
// WINAPI
// LoadUserProfileW(
//      HANDLE hToken,
//      LPPROFILEINFOW lpProfileInfo)
// {
//     WCHAR szUserHivePath[MAX_PATH];
//     DWORD dwLength;
//     PTOKEN_USER UserSid = NULL;
//     UNICODE_STRING SidString = { 0, 0, NULL };
//     HANDLE hProfileMutex = NULL;
//     DWORD dwError = MAXDWORD;
//     BOOL ret = FALSE;
//
//     DPRINT("LoadUserProfileW(%p %p)\n", hToken, lpProfileInfo);
//
//     /* Check profile info */
//     if (!lpProfileInfo || (lpProfileInfo->dwSize != sizeof(PROFILEINFOW)) ||
//         (lpProfileInfo->lpUserName == NULL) || (lpProfileInfo->lpUserName[0] == 0))
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     DPRINT("UserName: %S\n", lpProfileInfo->lpUserName);
//
//     /* Get the user SID string */
//     ret = GetUserSidStringFromToken(hToken, &SidString);
//     if (!ret)
//     {
//         DPRINT1("GetUserSidStringFromToken() failed\n");
//         goto cleanup;
//     }
//     ret = FALSE;
//
//     /* Create the profile mutex */
//     hProfileMutex = CreateProfileMutex(SidString.Buffer);
//     if (hProfileMutex == NULL)
//     {
//         DPRINT1("Failed to create the profile mutex\n");
//         goto cleanup;
//     }
//
//     /* Wait for the profile mutex */
//     WaitForSingleObject(hProfileMutex, INFINITE);
//
//     /* Don't load a profile twice */
//     if (CheckForLoadedProfile(hToken))
//     {
//         DPRINT1("Profile %S already loaded\n", SidString.Buffer);
//     }
//     else
//     {
//         DPRINT1("Loading profile %S\n", SidString.Buffer);
//
//         /*
//          * NOTE: lpProfilePath specifies the path to a *roaming* user profile,
//          * for example on a domain server, if any. It is then used to create
//          * a local image (copy) of the profile on the local computer.
//          *
//          * FIXME: We currently don't implement this in ReactOS; instead, we use
//          * it directly as *the* user's profile path, without doing any copy...
//          */
//         if (lpProfileInfo->lpProfilePath)
//         {
//             /* Use the caller's specified roaming user profile path */
//             StringCbCopyW(szUserHivePath, sizeof(szUserHivePath), lpProfileInfo->lpProfilePath);
//         }
//         else
//         {
//             /* Build a default user profile path */
//             dwLength = _countof(szUserHivePath);
//             if (!GetProfilesDirectoryW(szUserHivePath, &dwLength))
//             {
//                 DPRINT1("GetProfilesDirectoryW() failed (Error %ld)\n", GetLastError());
//                 goto cleanup;
//             }
//             StringCbCatW(szUserHivePath, sizeof(szUserHivePath), L"\\");
//             StringCbCatW(szUserHivePath, sizeof(szUserHivePath), lpProfileInfo->lpUserName);
//         }
//
//         /* Create user hive name */
//         StringCbCatW(szUserHivePath, sizeof(szUserHivePath), L"\\ntuser.dat");
//         DPRINT("szUserHivePath: %S\n", szUserHivePath);
//
//         /* Create user profile directory if needed */
//         if (GetFileAttributesW(szUserHivePath) == INVALID_FILE_ATTRIBUTES)
//         {
//             /* Get user sid */
//             if (GetTokenInformation(hToken, TokenUser, NULL, 0, &dwLength) ||
//                 GetLastError() != ERROR_INSUFFICIENT_BUFFER)
//             {
//                 DPRINT1("GetTokenInformation() failed\n");
//                 goto cleanup;
//             }
//
//             UserSid = (PTOKEN_USER)HeapAlloc(GetProcessHeap(), 0, dwLength);
//             if (!UserSid)
//             {
//                 dwError = ERROR_NOT_ENOUGH_MEMORY;
//                 DPRINT1("HeapAlloc() failed\n");
//                 goto cleanup;
//             }
//
//             if (!GetTokenInformation(hToken, TokenUser, UserSid, dwLength, &dwLength))
//             {
//                 DPRINT1("GetTokenInformation() failed\n");
//                 goto cleanup;
//             }
//
//             /* Create profile */
//             ret = CreateUserProfileW(UserSid->User.Sid, lpProfileInfo->lpUserName);
//             if (!ret)
//             {
//                 DPRINT1("CreateUserProfileW() failed\n");
//                 goto cleanup;
//             }
//             ret = FALSE;
//         }
//
//         /* Acquire restore privilege */
//         if (!AcquireRemoveRestorePrivilege(TRUE))
//         {
//             DPRINT1("AcquireRemoveRestorePrivilege() failed (Error %ld)\n", GetLastError());
//             goto cleanup;
//         }
//
//         /* Load the user registry hive */
//         dwError = RegLoadKeyW(HKEY_USERS,
//                               SidString.Buffer,
//                               szUserHivePath);
//         AcquireRemoveRestorePrivilege(FALSE);
//
//         /* HACK: Do not fail if the profile has already been loaded! */
//         if (dwError == ERROR_SHARING_VIOLATION)
//             dwError = ERROR_SUCCESS;
//
//         if (dwError != ERROR_SUCCESS)
//         {
//             DPRINT1("RegLoadKeyW() failed (Error %ld)\n", dwError);
//             goto cleanup;
//         }
//
//         SetProfileData(SidString.Buffer,
//                        lpProfileInfo->dwFlags,
//                        hToken);
//     }
//
//     /* Open future HKEY_CURRENT_USER */
//     dwError = RegOpenKeyExW(HKEY_USERS,
//                             SidString.Buffer,
//                             0,
//                             MAXIMUM_ALLOWED,
//                             (PHKEY)&lpProfileInfo->hProfile);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("RegOpenKeyExW() failed (Error %ld)\n", dwError);
//         goto cleanup;
//     }
//
//     dwError = IncrementRefCount(SidString.Buffer, NULL);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("IncrementRefCount() failed (Error %ld)\n", dwError);
//         goto cleanup;
//     }
//
//     ret = TRUE;
//
// cleanup:
//     if (dwError == MAXDWORD)
//         dwError = GetLastError();
//
//     if (UserSid != NULL)
//         HeapFree(GetProcessHeap(), 0, UserSid);
//
//     if (hProfileMutex != NULL)
//     {
//         ReleaseMutex(hProfileMutex);
//         CloseHandle(hProfileMutex);
//     }
//
//     RtlFreeUnicodeString(&SidString);
//
//     /* Report any error occurred */
//     if (!ret)
//     {
//         // TODO: Use a localized *.mc resource message* error.
//         ReportError(lpProfileInfo->dwFlags,
//                     L"ReactOS could not load the locally stored user profile. "
//                     L"Possible causes of this error include insufficient security rights or a corrupt local profile.\n"
//                     L"\nError: %lu\n",
//                     dwError);
//     }
//
//     DPRINT("LoadUserProfileW() done\n");
//     SetLastError(dwError);
//     return ret;
// }
//
//
// BOOL
// WINAPI
// UnloadUserProfile(
//      HANDLE hToken,
//      HANDLE hProfile)
// {
//     UNICODE_STRING SidString = {0, 0, NULL};
//     HANDLE hProfileMutex = NULL;
//     HKEY hProfilesKey = NULL, hProfileKey = NULL;
//     DWORD dwRefCount = 0, dwLength, dwType, dwState = 0;
//     DWORD dwError;
//     BOOL bRet = FALSE;
//
//     DPRINT("UnloadUserProfile() called\n");
//
//     if (hProfile == NULL)
//     {
//         DPRINT1("Invalid profile handle\n");
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* Get the user SID string */
//     if (!GetUserSidStringFromToken(hToken, &SidString))
//     {
//         DPRINT1("GetUserSidStringFromToken() failed\n");
//         return FALSE;
//     }
//
//     DPRINT("SidString: '%wZ'\n", &SidString);
//
//     /* Create the profile mutex */
//     hProfileMutex = CreateProfileMutex(SidString.Buffer);
//     if (hProfileMutex == NULL)
//     {
//         DPRINT1("Failed to create the profile mutex\n");
//         goto cleanup;
//     }
//
//     /* Wait for the profile mutex */
//     WaitForSingleObject(hProfileMutex, INFINITE);
//
//     /* Close the profile handle */
//     RegFlushKey(hProfile);
//     RegCloseKey(hProfile);
//
//     dwError = DecrementRefCount(SidString.Buffer, &dwRefCount);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("DecrementRefCount() failed (Error %lu)\n", dwError);
//         SetLastError(dwError);
//         goto cleanup;
//     }
//
//     if (dwRefCount == 0)
//     {
//         DPRINT("RefCount is 0: Unload the Hive!\n");
//
//         /* Acquire restore privilege */
//         if (!AcquireRemoveRestorePrivilege(TRUE))
//         {
//             DPRINT1("AcquireRemoveRestorePrivilege() failed (Error %lu)\n", GetLastError());
//             goto cleanup;
//         }
//
//         /* HACK */
//         {
//             HKEY hUserKey;
//
//             dwError = RegOpenKeyExW(HKEY_USERS,
//                                     SidString.Buffer,
//                                     0,
//                                     KEY_WRITE,
//                                     &hUserKey);
//             if (dwError == ERROR_SUCCESS)
//             {
//                 RegDeleteKeyW(hUserKey,
//                               L"Volatile Environment");
//
//                 RegCloseKey(hUserKey);
//             }
//         }
//         /* End of HACK */
//
//         /* Unload the hive */
//         dwError = RegUnLoadKeyW(HKEY_USERS,
//                                 SidString.Buffer);
//
//         /* Remove restore privilege */
//         AcquireRemoveRestorePrivilege(FALSE);
//
//         if (dwError != ERROR_SUCCESS)
//         {
//             DPRINT1("RegUnLoadKeyW() failed (Error %lu)\n", dwError);
//             SetLastError(dwError);
//             goto cleanup;
//         }
//
//         dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                                 L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList",
//                                 0,
//                                 KEY_QUERY_VALUE,
//                                 &hProfilesKey);
//         if (dwError != ERROR_SUCCESS)
//         {
//             DPRINT1("RegOpenKeyExW() failed (Error %lu)\n", dwError);
//             SetLastError(dwError);
//             goto cleanup;
//         }
//
//         dwError = RegOpenKeyExW(hProfilesKey,
//                                 SidString.Buffer,
//                                 0,
//                                 KEY_QUERY_VALUE,
//                                 &hProfileKey);
//         if (dwError != ERROR_SUCCESS)
//         {
//             DPRINT1("RegOpenKeyExW() failed (Error %lu)\n", dwError);
//             SetLastError(dwError);
//             goto cleanup;
//         }
//
//         /* Get the State value */
//         dwLength = sizeof(dwState);
//         dwError = RegQueryValueExW(hProfileKey,
//                                    L"State",
//                                    NULL,
//                                    &dwType,
//                                    (PBYTE)&dwState,
//                                    &dwLength);
//         if (dwError != ERROR_SUCCESS)
//         {
//             DPRINT1("RegQueryValueExW() failed (Error %lu)\n", dwError);
//             SetLastError(dwError);
//             goto cleanup;
//         }
//
//         /* Delete a guest profile */
//         if (dwState & 0x80) // PROFILE_GUEST_USER
//         {
//             if (!DeleteProfileW(SidString.Buffer, NULL, NULL))
//             {
//                 DPRINT1("DeleteProfile(%S, NULL, NULL) failed (Error %lu)\n",
//                         SidString.Buffer, GetLastError());
//                 goto cleanup;
//             }
//         }
//     }
//
//     bRet = TRUE;
//
// cleanup:
//     if (hProfileKey != NULL)
//         RegCloseKey(hProfileKey);
//
//     if (hProfilesKey != NULL)
//         RegCloseKey(hProfilesKey);
//
//     if (hProfileMutex != NULL)
//     {
//         ReleaseMutex(hProfileMutex);
//         CloseHandle(hProfileMutex);
//     }
//
//     RtlFreeUnicodeString(&SidString);
//
//     DPRINT("UnloadUserProfile() done\n");
//
//     return bRet;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
