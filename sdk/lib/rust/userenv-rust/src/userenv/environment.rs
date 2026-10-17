use crate::prelude::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::prelude::RegOpenKeyExW;
use crate::prelude::RegCloseKey;
use crate::prelude::RegQueryValueExW;
use core::ffi::c_void;

extern "C" {
    // ARRAYSIZE removed: it is a MACRO (sizeof(a)/sizeof(a[0])), not a linkable fn — the one live
    // call now uses the Rust array's .len(). RegQueryInfoKey removed: WINAPI (stdcall), resolves
    // as RegQueryInfoKeyW — moved to the extern "system" block below.
    fn _wcsicmp(a: *const u16, b: *const u16) -> i32;
    fn GetUserSidStringFromToken(hToken: HANDLE, SidString: *mut _UNICODE_STRING) -> i32;
}
extern "system" {
    fn GetUserSid(hToken: HANDLE) -> PSID; // stdcall WINAPI export (userenv.spec:142)
}
extern "system" {
    // RegQueryInfoKey is WINAPI (stdcall) → the real advapi32 symbol is RegQueryInfoKeyW.
    fn RegQueryInfoKeyW( hKey: HKEY, lpClass: LPWSTR, lpcchClass: *mut DWORD, lpReserved: *mut DWORD, lpcSubKeys: *mut DWORD, lpcbMaxSubKeyLen: *mut DWORD, lpcbMaxClassLen: *mut DWORD, lpcValues: *mut DWORD, lpcbMaxValueNameLen: *mut DWORD, lpcbMaxValueLen: *mut DWORD, lpcbSecurityDescriptor: *mut DWORD, lpftLastWriteTime: *mut ::core::ffi::c_void, ) -> LONG;
    fn GetEnvironmentVariableW(lpName: LPCWSTR, lpBuffer: LPWSTR, nSize: DWORD) -> DWORD;
    fn GetComputerNameW(lpBuffer: LPWSTR, nSize: *mut DWORD) -> BOOL;
    fn GetAllUsersProfileDirectoryW(lpProfileDir: LPWSTR, lpcchSize: *mut DWORD) -> BOOL;
    fn GetDefaultUserProfileDirectoryW(lpProfileDir: LPWSTR, lpcchSize: *mut DWORD) -> BOOL;
    fn GetUserProfileDirectoryW(hToken: HANDLE, lpProfileDir: LPWSTR, lpcchSize: *mut DWORD) -> BOOL;
    // StringCchCopyNW removed: strsafe.h `static inline`, no linkable symbol → prelude shim.
    fn GlobalAlloc(uFlags: u32, dwBytes: usize) -> *mut core::ffi::c_void;
    fn GlobalFree(hMem: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn MultiByteToWideChar(CodePage: u32, dwFlags: u32, lpMultiByteStr: *const i8, cchMultiByte: i32, lpWideCharStr: *mut u16, cchWideChar: i32) -> i32;
    fn WideCharToMultiByte(CodePage: u32, dwFlags: u32, lpWideCharStr: *const u16, cchWideChar: i32, lpMultiByteStr: *mut i8, cbMultiByte: i32, lpDefaultChar: *const i8, lpUsedDefaultChar: *mut i32) -> i32;
}

// L";"
static SEMICOLON_W: [u16; 2] = [
    b';' as u16, 0,
];
// L"PATH"
static ENV_PATH: [u16; 5] = [
    b'P' as u16, b'A' as u16, b'T' as u16, b'H' as u16, 0,
];
// L"SystemRoot"
static ENV_SYSTEM_ROOT: [u16; 11] = [
    b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16, b'R' as u16,
    b'o' as u16, b'o' as u16, b't' as u16, 0,
];
// L"SystemDrive"
static ENV_SYSTEM_DRIVE: [u16; 12] = [
    b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16, b'D' as u16,
    b'r' as u16, b'i' as u16, b'v' as u16, b'e' as u16, 0,
];
// L"COMPUTERNAME"
static ENV_COMPUTERNAME: [u16; 13] = [
    b'C' as u16, b'O' as u16, b'M' as u16, b'P' as u16, b'U' as u16, b'T' as u16, b'E' as u16,
    b'R' as u16, b'N' as u16, b'A' as u16, b'M' as u16, b'E' as u16, 0,
];
// L"ALLUSERSPROFILE"
static ENV_ALLUSERSPROFILE: [u16; 16] = [
    b'A' as u16, b'L' as u16, b'L' as u16, b'U' as u16, b'S' as u16, b'E' as u16, b'R' as u16,
    b'S' as u16, b'P' as u16, b'R' as u16, b'O' as u16, b'F' as u16, b'I' as u16, b'L' as u16,
    b'E' as u16, 0,
];
// L"USERPROFILE"
static ENV_USERPROFILE: [u16; 12] = [
    b'U' as u16, b'S' as u16, b'E' as u16, b'R' as u16, b'P' as u16, b'R' as u16, b'O' as u16,
    b'F' as u16, b'I' as u16, b'L' as u16, b'E' as u16, 0,
];
// L"SOFTWARE\\Microsoft\\Windows\\CurrentVersion"
static CURRENT_VERSION_KEY: [u16; 42] = [
    b'S' as u16, b'O' as u16, b'F' as u16, b'T' as u16, b'W' as u16, b'A' as u16, b'R' as u16,
    b'E' as u16, b'\\' as u16, b'M' as u16, b'i' as u16, b'c' as u16, b'r' as u16, b'o' as u16,
    b's' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'\\' as u16, b'W' as u16, b'i' as u16,
    b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b's' as u16, b'\\' as u16, b'C' as u16,
    b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16, b'V' as u16,
    b'e' as u16, b'r' as u16, b's' as u16, b'i' as u16, b'o' as u16, b'n' as u16, 0,
];
// L"ProgramFilesDir"
static REG_PROGRAM_FILES_DIR: [u16; 16] = [
    b'P' as u16, b'r' as u16, b'o' as u16, b'g' as u16, b'r' as u16, b'a' as u16, b'm' as u16,
    b'F' as u16, b'i' as u16, b'l' as u16, b'e' as u16, b's' as u16, b'D' as u16, b'i' as u16,
    b'r' as u16, 0,
];
// L"ProgramFiles"
static ENV_PROGRAM_FILES: [u16; 13] = [
    b'P' as u16, b'r' as u16, b'o' as u16, b'g' as u16, b'r' as u16, b'a' as u16, b'm' as u16,
    b'F' as u16, b'i' as u16, b'l' as u16, b'e' as u16, b's' as u16, 0,
];
// L"CommonFilesDir"
static REG_COMMON_FILES_DIR: [u16; 15] = [
    b'C' as u16, b'o' as u16, b'm' as u16, b'm' as u16, b'o' as u16, b'n' as u16, b'F' as u16,
    b'i' as u16, b'l' as u16, b'e' as u16, b's' as u16, b'D' as u16, b'i' as u16, b'r' as u16,
    0,
];
// L"CommonProgramFiles"
static ENV_COMMON_PROGRAM_FILES: [u16; 19] = [
    b'C' as u16, b'o' as u16, b'm' as u16, b'm' as u16, b'o' as u16, b'n' as u16, b'P' as u16,
    b'r' as u16, b'o' as u16, b'g' as u16, b'r' as u16, b'a' as u16, b'm' as u16, b'F' as u16,
    b'i' as u16, b'l' as u16, b'e' as u16, b's' as u16, 0,
];
// L"HOMEDRIVE"
static ENV_HOMEDRIVE: [u16; 10] = [
    b'H' as u16, b'O' as u16, b'M' as u16, b'E' as u16, b'D' as u16, b'R' as u16, b'I' as u16,
    b'V' as u16, b'E' as u16, 0,
];
// L"HOMEPATH"
static ENV_HOMEPATH: [u16; 9] = [
    b'H' as u16, b'O' as u16, b'M' as u16, b'E' as u16, b'P' as u16, b'A' as u16, b'T' as u16,
    b'H' as u16, 0,
];
// L"USERNAME"
static ENV_USERNAME: [u16; 9] = [
    b'U' as u16, b'S' as u16, b'E' as u16, b'R' as u16, b'N' as u16, b'A' as u16, b'M' as u16,
    b'E' as u16, 0,
];
// L"USERDOMAIN"
static ENV_USERDOMAIN: [u16; 11] = [
    b'U' as u16, b'S' as u16, b'E' as u16, b'R' as u16, b'D' as u16, b'O' as u16, b'M' as u16,
    b'A' as u16, b'I' as u16, b'N' as u16, 0,
];
// L"Environment"
static REG_ENVIRONMENT_KEY: [u16; 12] = [
    b'E' as u16, b'n' as u16, b'v' as u16, b'i' as u16, b'r' as u16, b'o' as u16, b'n' as u16,
    b'm' as u16, b'e' as u16, b'n' as u16, b't' as u16, 0,
];
// L"Volatile Environment"
static REG_VOLATILE_ENVIRONMENT_KEY: [u16; 21] = [
    b'V' as u16, b'o' as u16, b'l' as u16, b'a' as u16, b't' as u16, b'i' as u16, b'l' as u16,
    b'e' as u16, b' ' as u16, b'E' as u16, b'n' as u16, b'v' as u16, b'i' as u16, b'r' as u16,
    b'o' as u16, b'n' as u16, b'm' as u16, b'e' as u16, b'n' as u16, b't' as u16, 0,
];

// ---- file-scope preamble (translated) ----
pub const NDEBUG: u32 = 1;
unsafe extern "C" fn SetUserEnvironmentVariable(
    Environment: *mut PWSTR,
    lpName: LPWSTR,
    lpValue: LPWSTR,
    bExpand: i32,
) -> i32 {
    let mut Status: i32;
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let mut SrcValue: _UNICODE_STRING = core::mem::zeroed();
    let mut DstValue: _UNICODE_STRING = core::mem::zeroed();
    let mut Length: u32;
    let mut Buffer: LPVOID = core::ptr::null_mut();
    let mut ShortName: [u16; 260] = [0; 260];

    if bExpand != 0 {
        RtlInitUnicodeString(core::ptr::addr_of_mut!(SrcValue), lpValue);

        Length = (2 * MAX_PATH as u32) * core::mem::size_of::<u16>() as u32;

        DstValue.Length = 0;
        DstValue.MaximumLength = Length as u16;
        Buffer = LocalAlloc(LPTR as u32, Length as usize);
        DstValue.Buffer = Buffer as PWSTR;
        if DstValue.Buffer.is_null() {
            return 0i32;
        }

        Status = RtlExpandEnvironmentStrings_U(
            *Environment,
            core::ptr::addr_of_mut!(SrcValue),
            core::ptr::addr_of_mut!(DstValue),
            core::ptr::addr_of_mut!(Length),
        );
        if !NT_SUCCESS(Status) {
            if !Buffer.is_null() {
                LocalFree(Buffer as HGLOBAL);
            }

            return 0i32;
        }
    } else {
        RtlInitUnicodeString(core::ptr::addr_of_mut!(DstValue), lpValue);
    }

    if _wcsicmp(lpName, win32macros_rust::wide_bytes!(b"T\0E\0M\0P\0\0\0") as *const u16) == 0
        || _wcsicmp(lpName, win32macros_rust::wide_bytes!(b"T\0M\0P\0\0\0") as *const u16) == 0
    {
        if GetShortPathNameW(
            DstValue.Buffer,
            ShortName.as_mut_ptr(),
            ShortName.len() as u32, // C: ARRAYSIZE(ShortName) — element count of WCHAR[MAX_PATH]
        ) != 0
        {
            RtlInitUnicodeString(core::ptr::addr_of_mut!(DstValue), ShortName.as_mut_ptr());
        }
    }

    RtlInitUnicodeString(core::ptr::addr_of_mut!(Name), lpName);

    Status = RtlSetEnvironmentVariable(
        Environment,
        core::ptr::addr_of_mut!(Name),
        core::ptr::addr_of_mut!(DstValue),
    );

    if !Buffer.is_null() {
        LocalFree(Buffer as HGLOBAL);
    }

    if !NT_SUCCESS(Status) {
        return 0i32;
    }

    1i32
}
unsafe extern "C" fn AppendUserEnvironmentVariable(
    Environment: *mut PWSTR,
    lpName: LPWSTR,
    lpValue: LPWSTR,
) -> u8 {
    let mut Status: i32;
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let mut Value: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(Name),
        lpName,
    );

    Value.Length = 0;
    Value.MaximumLength = (1024 * core::mem::size_of::<u16>()) as u16;
    Value.Buffer = LocalAlloc(LPTR as u32, Value.MaximumLength as usize) as LPWSTR;
    if Value.Buffer.is_null() {
        return 0u8;
    }

    *Value.Buffer = 0u16;

    Status = RtlQueryEnvironmentVariable_U(
        *Environment,
        core::ptr::addr_of!(Name),
        core::ptr::addr_of_mut!(Value),
    );
    if NT_SUCCESS(Status) {
        RtlAppendUnicodeToString(
            core::ptr::addr_of_mut!(Value),
            SEMICOLON_W.as_ptr() as PCWSTR,
        );
    }

    RtlAppendUnicodeToString(
        core::ptr::addr_of_mut!(Value),
        lpValue,
    );

    Status = RtlSetEnvironmentVariable(
        Environment,
        core::ptr::addr_of_mut!(Name),
        core::ptr::addr_of_mut!(Value),
    );
    LocalFree(Value.Buffer as LPVOID);
    if !NT_SUCCESS(Status) {
        return 0u8;
    }

    return 1u8;
}
unsafe extern "C" fn GetCurrentUserKey(mut hToken: HANDLE) -> HKEY {
    let mut Error: i32 = 0;
    let mut SidString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<u16>(),
    };
    let mut hKey: HKEY = core::ptr::null_mut::<HKEY__>();
    if GetUserSidStringFromToken(hToken, &raw mut SidString) == 0 {
        return core::ptr::null_mut::<HKEY__>();
    }
    Error = RegOpenKeyExW(
        HKEY_USERS,
        SidString.Buffer as LPCWSTR,
        0 as u32,
        MAXIMUM_ALLOWED as u32,
        &raw mut hKey,
    ) as i32;
    if Error != ERROR_SUCCESS as i32 {
        RtlFreeUnicodeString(&raw mut SidString);
        SetLastError(Error as u32);
        return core::ptr::null_mut::<HKEY__>();
    }
    RtlFreeUnicodeString(&raw mut SidString);
    return hKey;
}
unsafe extern "C" fn GetUserAndDomainName(
    hToken: HANDLE,
    UserName: *mut LPWSTR,
    DomainName: *mut LPWSTR,
) -> i32 {
    let mut bRet: i32 = 1;
    let mut Sid: PSID = core::ptr::null_mut();
    let mut lpUserName: LPWSTR = core::ptr::null_mut();
    let mut lpDomainName: LPWSTR = core::ptr::null_mut();
    let mut cbUserName: u32 = 0;
    let mut cbDomainName: u32 = 0;
    let mut SidNameUse: u32 = 0;

    Sid = GetUserSid(hToken);
    if Sid.is_null() {
        return 0;
    }

    'done: {
    if LookupAccountSidW(
        core::ptr::null(),
        Sid,
        core::ptr::null_mut(),
        &mut cbUserName,
        core::ptr::null_mut(),
        &mut cbDomainName,
        &mut SidNameUse,
    ) == 0
    {
        if GetLastError() != ERROR_INSUFFICIENT_BUFFER {
            bRet = 0;
            break 'done;
        }
    }

    lpUserName = LocalAlloc(LPTR, (cbUserName as usize).wrapping_mul(core::mem::size_of::<u16>())) as LPWSTR;
    if lpUserName.is_null() {
        bRet = 0;
        break 'done;
    }

    lpDomainName = LocalAlloc(LPTR, (cbDomainName as usize).wrapping_mul(core::mem::size_of::<u16>())) as LPWSTR;
    if lpDomainName.is_null() {
        bRet = 0;
        break 'done;
    }

    if LookupAccountSidW(
        core::ptr::null(),
        Sid,
        lpUserName,
        &mut cbUserName,
        lpDomainName,
        &mut cbDomainName,
        &mut SidNameUse,
    ) == 0
    {
        bRet = 0;
    } else {
        *UserName = lpUserName;
        *DomainName = lpDomainName;
    }

    }
    // done:
    if bRet == 0 {
        if !lpUserName.is_null() {
            LocalFree(lpUserName as HLOCAL);
        }

        if !lpDomainName.is_null() {
            LocalFree(lpDomainName as HLOCAL);
        }
    }

    LocalFree(Sid);

    bRet
}
unsafe extern "C" fn SetUserEnvironment(
    mut Environment: *mut PWSTR,
    mut hKey: HKEY,
    mut lpSubKeyName: LPWSTR,
) -> BOOL {
    let mut Error: LONG = 0;
    let mut hEnvKey: HKEY = core::ptr::null_mut();
    let mut dwValues: DWORD = 0;
    let mut dwMaxValueNameLength: DWORD = 0;
    let mut dwMaxValueDataLength: DWORD = 0;
    let mut dwValueNameLength: DWORD = 0;
    let mut dwValueDataLength: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut i: DWORD = 0;
    let mut lpValueName: LPWSTR = core::ptr::null_mut();
    let mut lpValueData: LPWSTR = core::ptr::null_mut();

    Error = RegOpenKeyExW(
        hKey,
        lpSubKeyName,
        0,
        KEY_QUERY_VALUE,
        &mut hEnvKey,
    );
    if Error != ERROR_SUCCESS as LONG {
        SetLastError(Error as DWORD);
        return 0;
    }

    Error = RegQueryInfoKeyW(
        hEnvKey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut dwValues,
        &mut dwMaxValueNameLength,
        &mut dwMaxValueDataLength,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if Error != ERROR_SUCCESS as LONG {
        RegCloseKey(hEnvKey);
        SetLastError(Error as DWORD);
        return 0;
    }

    if dwValues == 0 {
        RegCloseKey(hEnvKey);
        return 1;
    }

    dwMaxValueNameLength = dwMaxValueNameLength.wrapping_add(1);
    lpValueName = LocalAlloc(LPTR, dwMaxValueNameLength as usize * core::mem::size_of::<u16>()) as LPWSTR;
    if lpValueName.is_null() {
        RegCloseKey(hEnvKey);
        return 0;
    }

    lpValueData = LocalAlloc(LPTR, dwMaxValueDataLength as usize) as LPWSTR;
    if lpValueData.is_null() {
        LocalFree(lpValueName as HGLOBAL);
        RegCloseKey(hEnvKey);
        return 0;
    }

    i = 0;
    while i < dwValues {
        dwValueNameLength = dwMaxValueNameLength;
        dwValueDataLength = dwMaxValueDataLength;

        Error = RegEnumValueW(
            hEnvKey,
            i,
            lpValueName,
            &mut dwValueNameLength,
            core::ptr::null_mut(),
            &mut dwType,
            lpValueData as LPBYTE,
            &mut dwValueDataLength,
        );
        if Error == ERROR_SUCCESS as LONG {
            if _wcsicmp(lpValueName, ENV_PATH.as_ptr() as LPCWSTR) == 0 {
                AppendUserEnvironmentVariable(Environment, lpValueName, lpValueData);
            } else {
                SetUserEnvironmentVariable(
                    Environment,
                    lpValueName,
                    lpValueData,
                    (dwType == REG_EXPAND_SZ) as ::core::ffi::c_int,
                );
            }
        } else {
            LocalFree(lpValueData as HGLOBAL);
            LocalFree(lpValueName as HGLOBAL);
            RegCloseKey(hEnvKey);
            return 0;
        }
        i = i.wrapping_add(1);
    }

    LocalFree(lpValueData as HGLOBAL);
    LocalFree(lpValueName as HGLOBAL);
    RegCloseKey(hEnvKey);

    1
}
unsafe extern "C" fn SetSystemEnvironment(Environment: *mut PWSTR) -> BOOL {
    let mut Error: LONG;
    let mut hEnvKey: HKEY = core::ptr::null_mut();
    let mut dwValues: DWORD = 0;
    let mut dwMaxValueNameLength: DWORD = 0;
    let mut dwMaxValueDataLength: DWORD = 0;
    let mut dwValueNameLength: DWORD;
    let mut dwValueDataLength: DWORD;
    let mut dwType: DWORD = 0;
    let mut i: DWORD;
    let mut lpValueName: LPWSTR;
    let mut lpValueData: LPWSTR;

    // L"SYSTEM\CurrentControlSet\Control\Session Manager\Environment"
    static SYSTEM_ENV_KEY_BUF: [u16; 61] = [
        b'S' as u16, b'Y' as u16, b'S' as u16, b'T' as u16, b'E' as u16, b'M' as u16, b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16,
        b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16,
        b'l' as u16, b'S' as u16, b'e' as u16, b't' as u16, b'\\' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16,
        b'o' as u16, b'l' as u16, b'\\' as u16, b'S' as u16, b'e' as u16, b's' as u16, b's' as u16, b'i' as u16, b'o' as u16, b'n' as u16,
        b' ' as u16, b'M' as u16, b'a' as u16, b'n' as u16, b'a' as u16, b'g' as u16, b'e' as u16, b'r' as u16, b'\\' as u16, b'E' as u16,
        b'n' as u16, b'v' as u16, b'i' as u16, b'r' as u16, b'o' as u16, b'n' as u16, b'm' as u16, b'e' as u16, b'n' as u16, b't' as u16,
        0,
    ];
    Error = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        SYSTEM_ENV_KEY_BUF.as_ptr() as *const u16 as LPCWSTR,
        0,
        KEY_QUERY_VALUE,
        &mut hEnvKey,
    );
    if Error != (ERROR_SUCCESS as i32) {
        return 0;
    }

    Error = RegQueryInfoKeyW(
        hEnvKey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut dwValues,
        &mut dwMaxValueNameLength,
        &mut dwMaxValueDataLength,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if Error != (ERROR_SUCCESS as i32) {
        RegCloseKey(hEnvKey);
        return 0;
    }

    if dwValues == 0 {
        RegCloseKey(hEnvKey);
        return 1;
    }

    dwMaxValueNameLength += 1;
    lpValueName = LocalAlloc(LPTR, (dwMaxValueNameLength as usize) * core::mem::size_of::<u16>())
        as LPWSTR;
    if lpValueName.is_null() {
        RegCloseKey(hEnvKey);
        return 0;
    }

    lpValueData = LocalAlloc(LPTR, dwMaxValueDataLength as usize) as LPWSTR;
    if lpValueData.is_null() {
        LocalFree(lpValueName as HLOCAL);
        RegCloseKey(hEnvKey);
        return 0;
    }

    i = 0;
    while i < dwValues {
        dwValueNameLength = dwMaxValueNameLength;
        dwValueDataLength = dwMaxValueDataLength;

        Error = RegEnumValueW(
            hEnvKey,
            i,
            lpValueName,
            &mut dwValueNameLength,
            core::ptr::null_mut(),
            &mut dwType,
            lpValueData as LPBYTE,
            &mut dwValueDataLength,
        );
        if Error == (ERROR_SUCCESS as i32) {
            SetUserEnvironmentVariable(
                Environment,
                lpValueName,
                lpValueData,
                if dwType == REG_EXPAND_SZ { 1 } else { 0 },
            );
        } else {
            LocalFree(lpValueData as HLOCAL);
            LocalFree(lpValueName as HLOCAL);
            RegCloseKey(hEnvKey);
            return 0;
        }
        i += 1;
    }

    LocalFree(lpValueData as HLOCAL);
    LocalFree(lpValueName as HLOCAL);
    RegCloseKey(hEnvKey);

    1
}

#[no_mangle]
pub unsafe extern "system" fn CreateEnvironmentBlock(lpEnvironment: *mut LPVOID, hToken: HANDLE, bInherit: BOOL) -> BOOL {
    let mut Status: NTSTATUS;
    let mut lError: LONG;
    let Environment: *mut PWSTR = lpEnvironment as *mut PWSTR;
    let mut Length: DWORD;
    let mut dwType: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hKeyUser: HKEY;
    let mut lpUserName: LPWSTR = core::ptr::null_mut();
    let mut lpDomainName: LPWSTR = core::ptr::null_mut();
    let mut Buffer: [WCHAR; 260] = [0; 260];
    let mut szValue: [WCHAR; 1024] = [0; 1024];

    if lpEnvironment.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return 0;
    }

    Status = RtlCreateEnvironment(bInherit as u8, Environment);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status) as DWORD);
        return 0;
    }

    Length = 260;
    if GetEnvironmentVariableW(ENV_SYSTEM_ROOT.as_ptr() as LPCWSTR, Buffer.as_mut_ptr(), Length) != 0 {
        SetUserEnvironmentVariable(Environment, ENV_SYSTEM_ROOT.as_ptr() as LPWSTR, Buffer.as_mut_ptr(), 0);
    }

    if GetEnvironmentVariableW(ENV_SYSTEM_DRIVE.as_ptr() as LPCWSTR, Buffer.as_mut_ptr(), Length) != 0 {
        SetUserEnvironmentVariable(Environment, ENV_SYSTEM_DRIVE.as_ptr() as LPWSTR, Buffer.as_mut_ptr(), 0);
    }

    if SetSystemEnvironment(Environment) == 0 {
        RtlDestroyEnvironment(*Environment);
        return 0;
    }

    Length = 260;
    if GetComputerNameW(Buffer.as_mut_ptr(), &mut Length) != 0 {
        SetUserEnvironmentVariable(Environment, ENV_COMPUTERNAME.as_ptr() as LPWSTR, Buffer.as_mut_ptr(), 0);
    }

    Length = 260;
    if GetAllUsersProfileDirectoryW(Buffer.as_mut_ptr(), &mut Length) != 0 {
        SetUserEnvironmentVariable(Environment, ENV_ALLUSERSPROFILE.as_ptr() as LPWSTR, Buffer.as_mut_ptr(), 0);
    }

    Length = 260;
    if GetDefaultUserProfileDirectoryW(Buffer.as_mut_ptr(), &mut Length) != 0 {
        SetUserEnvironmentVariable(Environment, ENV_USERPROFILE.as_ptr() as LPWSTR, Buffer.as_mut_ptr(), 1);
    }

    lError = RegOpenKeyExW(HKEY_LOCAL_MACHINE, CURRENT_VERSION_KEY.as_ptr() as LPCWSTR, 0, KEY_READ as REGSAM, &mut hKey);
    if lError == ERROR_SUCCESS as LONG {
        Length = core::mem::size_of::<[WCHAR; 1024]>() as DWORD;
        lError = RegQueryValueExW(hKey, REG_PROGRAM_FILES_DIR.as_ptr() as LPCWSTR, core::ptr::null_mut(), &mut dwType, szValue.as_mut_ptr() as LPBYTE, &mut Length);
        if lError == ERROR_SUCCESS as LONG {
            SetUserEnvironmentVariable(Environment, ENV_PROGRAM_FILES.as_ptr() as LPWSTR, szValue.as_mut_ptr(), 0);
        }

        Length = core::mem::size_of::<[WCHAR; 1024]>() as DWORD;
        lError = RegQueryValueExW(hKey, REG_COMMON_FILES_DIR.as_ptr() as LPCWSTR, core::ptr::null_mut(), &mut dwType, szValue.as_mut_ptr() as LPBYTE, &mut Length);
        if lError == ERROR_SUCCESS as LONG {
            SetUserEnvironmentVariable(Environment, ENV_COMMON_PROGRAM_FILES.as_ptr() as LPWSTR, szValue.as_mut_ptr(), 0);
        }

        RegCloseKey(hKey);
    }

    if hToken.is_null() {
        return 1;
    }

    hKeyUser = GetCurrentUserKey(hToken);
    if hKeyUser.is_null() {
        RtlDestroyEnvironment(*Environment);
        return 0;
    }

    Length = 260;
    if GetUserProfileDirectoryW(hToken, Buffer.as_mut_ptr(), &mut Length) != 0 {
        let MinLen: DWORD = 2;

        SetUserEnvironmentVariable(Environment, ENV_USERPROFILE.as_ptr() as LPWSTR, Buffer.as_mut_ptr(), 0);

        if Length > MinLen {
            StringCchCopyNW(szValue.as_mut_ptr(), 260, Buffer.as_mut_ptr(), MinLen as usize);
            SetUserEnvironmentVariable(Environment, ENV_HOMEDRIVE.as_ptr() as LPWSTR, szValue.as_mut_ptr(), 0);

            StringCchCopyNW(szValue.as_mut_ptr(), 260, Buffer.as_mut_ptr().offset(MinLen as isize), (Length - MinLen) as usize);
            SetUserEnvironmentVariable(Environment, ENV_HOMEPATH.as_ptr() as LPWSTR, szValue.as_mut_ptr(), 0);
        }
    }

    if GetUserAndDomainName(hToken, &mut lpUserName, &mut lpDomainName) != 0 {
        SetUserEnvironmentVariable(Environment, ENV_USERNAME.as_ptr() as LPWSTR, lpUserName, 0);
        SetUserEnvironmentVariable(Environment, ENV_USERDOMAIN.as_ptr() as LPWSTR, lpDomainName, 0);

        if !lpUserName.is_null() {
            LocalFree(lpUserName as HLOCAL);
        }

        if !lpDomainName.is_null() {
            LocalFree(lpDomainName as HLOCAL);
        }
    }

    SetUserEnvironment(Environment, hKeyUser, REG_ENVIRONMENT_KEY.as_ptr() as LPWSTR);
    SetUserEnvironment(Environment, hKeyUser, REG_VOLATILE_ENVIRONMENT_KEY.as_ptr() as LPWSTR);

    RegCloseKey(hKeyUser);

    1
}

#[no_mangle]
pub unsafe extern "system" fn DestroyEnvironmentBlock(lpEnvironment: LPVOID) -> i32 {
    if lpEnvironment.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    RtlDestroyEnvironment(lpEnvironment as PWSTR);

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn ExpandEnvironmentStringsForUserW(
    hToken: HANDLE,
    lpSrc: LPCWSTR,
    lpDest: LPWSTR,
    dwSize: u32,
) -> i32 {
    let mut Ret: i32 = 0;
    let mut lpEnvironment: LPVOID = core::ptr::null_mut();

    if lpSrc.is_null() || lpDest.is_null() || dwSize == 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if CreateEnvironmentBlock(core::ptr::addr_of_mut!(lpEnvironment), hToken, 0) != 0 {
        let mut SrcU: _UNICODE_STRING = _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: core::ptr::null_mut(),
        };
        let mut DestU: _UNICODE_STRING = _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: core::ptr::null_mut(),
        };
        let mut Status: i32;

        RtlInitUnicodeString(core::ptr::addr_of_mut!(SrcU), lpSrc);
        DestU.Length = 0;
        DestU.MaximumLength = (dwSize as usize * core::mem::size_of::<u16>()) as u16;
        DestU.Buffer = lpDest as PWSTR;

        Status = RtlExpandEnvironmentStrings_U(
            lpEnvironment as PWSTR,
            core::ptr::addr_of_mut!(SrcU),
            core::ptr::addr_of_mut!(DestU),
            core::ptr::null_mut(),
        );

        DestroyEnvironmentBlock(lpEnvironment);

        if NT_SUCCESS(Status) {
            Ret = 1;
        } else {
            SetLastError(RtlNtStatusToDosError(Status));
        }
    }

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn ExpandEnvironmentStringsForUserA(
    hToken: *mut core::ffi::c_void,
    lpSrc: *const i8,
    lpDest: *mut i8,
    dwSize: u32,
) -> i32 {
    let mut Ret: i32 = 0;
    let mut dwSrcLen: u32;
    let mut lpSrcW: *mut u16 = core::ptr::null_mut();
    let mut lpDestW: *mut u16 = core::ptr::null_mut();

    if lpSrc.is_null() || lpDest.is_null() || dwSize == 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    dwSrcLen = strlen(lpSrc) as u32;
    lpSrcW = GlobalAlloc(GMEM_FIXED as u32, ((dwSrcLen as usize) + 1) * core::mem::size_of::<u16>()) as *mut u16;
    
    if lpSrcW.is_null() || MultiByteToWideChar(0x0, 0, lpSrc, -1, lpSrcW, (dwSrcLen + 1) as i32) == 0 {
        goto_cleanup(&mut lpSrcW, &mut lpDestW);
        return Ret;
    }

    lpDestW = GlobalAlloc(GMEM_FIXED as u32, (dwSize as usize) * core::mem::size_of::<u16>()) as *mut u16;
    if lpDestW.is_null() {
        goto_cleanup(&mut lpSrcW, &mut lpDestW);
        return Ret;
    }

    Ret = ExpandEnvironmentStringsForUserW(hToken, lpSrcW, lpDestW, dwSize);
    if Ret != 0 {
        if WideCharToMultiByte(0x0, 0, lpDestW, -1, lpDest, dwSize as i32, core::ptr::null(), core::ptr::null_mut()) == 0 {
            Ret = 0;
        }
    }

    goto_cleanup(&mut lpSrcW, &mut lpDestW);
    Ret
}

unsafe fn goto_cleanup(lpSrcW: &mut *mut u16, lpDestW: &mut *mut u16) {
    if !lpSrcW.is_null() {
        GlobalFree(*lpSrcW as *mut core::ffi::c_void);
    }
    if !lpDestW.is_null() {
        GlobalFree(*lpDestW as *mut core::ffi::c_void);
    }
}
// enforce_import_abi: moved 10 import decl(s) into the extern block their
// exporting DLL's .spec requires: GetLastError->system, GetShortPathNameW->system, LocalAlloc->system, LocalFree->system, LookupAccountSidW->system, RegCloseKey->system, RegEnumValueW->system, RegOpenKeyExW->system, SetLastError->system, strlen->C.
// The CRT is cdecl, Win32 is stdcall, and the C source does not say which is which —
// a wrong guess is a silent i386 stack walk on every call. The .spec is the truth.
extern "C" {
    fn strlen(s: *const i8) -> usize;
}
extern "system" {
    fn GetLastError() -> u32;
    fn GetShortPathNameW(lpszLongPath: LPCWSTR, lpszShortPath: LPWSTR, cchBuffer: u32) -> u32;
    fn LocalAlloc(uFlags: u32, uBytes: usize) -> LPVOID;
    fn LocalFree(hMem: HGLOBAL) -> HGLOBAL;
    fn SetLastError(dwErrCode: u32);
    fn LookupAccountSidW( lpSystemName: LPCWSTR, lpSid: PSID, lpName: LPWSTR, cchName: *mut u32, lpReferencedDomainName: LPWSTR, cchReferencedDomainName: *mut u32, peUse: *mut u32, ) -> i32;
    fn RegEnumValueW( hKey: HKEY, dwIndex: DWORD, lpValueName: LPWSTR, lpcchValueName: *mut DWORD, lpReserved: *mut DWORD, lpType: *mut DWORD, lpData: LPBYTE, lpcbData: *mut DWORD, ) -> LONG;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  ReactOS kernel
//  *  Copyright (C) 2004 ReactOS Team
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/userenv/environment.c
//  * PURPOSE:         User environment functions
//  * PROGRAMMER:      Eric Kohl
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// static
// BOOL
// SetUserEnvironmentVariable(PWSTR* Environment,
//                            LPWSTR lpName,
//                            LPWSTR lpValue,
//                            BOOL bExpand)
// {
//     NTSTATUS Status;
//     UNICODE_STRING Name;
//     UNICODE_STRING SrcValue, DstValue;
//     ULONG Length;
//     PVOID Buffer = NULL;
//     WCHAR ShortName[MAX_PATH];
//
//     if (bExpand)
//     {
//         RtlInitUnicodeString(&SrcValue, lpValue);
//
//         Length = 2 * MAX_PATH * sizeof(WCHAR);
//
//         DstValue.Length = 0;
//         DstValue.MaximumLength = Length;
//         DstValue.Buffer = Buffer = LocalAlloc(LPTR, Length);
//         if (DstValue.Buffer == NULL)
//         {
//             DPRINT1("LocalAlloc() failed\n");
//             return FALSE;
//         }
//
//         Status = RtlExpandEnvironmentStrings_U(*Environment,
//                                                &SrcValue,
//                                                &DstValue,
//                                                &Length);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("RtlExpandEnvironmentStrings_U() failed (Status %lx)\n", Status);
//             DPRINT1("Length %lu\n", Length);
//
//             if (Buffer)
//                 LocalFree(Buffer);
//
//             return FALSE;
//         }
//     }
//     else
//     {
//         RtlInitUnicodeString(&DstValue, lpValue);
//     }
//
//     if (!_wcsicmp(lpName, L"TEMP") || !_wcsicmp(lpName, L"TMP"))
//     {
//         if (GetShortPathNameW(DstValue.Buffer, ShortName, ARRAYSIZE(ShortName)))
//         {
//             RtlInitUnicodeString(&DstValue, ShortName);
//         }
//         else
//         {
//             DPRINT("GetShortPathNameW() failed for %S (Error %lu)\n", DstValue.Buffer, GetLastError());
//         }
//     }
//
//     RtlInitUnicodeString(&Name, lpName);
//
//     DPRINT("Value: %wZ\n", &DstValue);
//
//     Status = RtlSetEnvironmentVariable(Environment,
//                                        &Name,
//                                        &DstValue);
//
//     if (Buffer)
//         LocalFree(Buffer);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("RtlSetEnvironmentVariable() failed (Status %lx)\n", Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// static
// BOOL
// AppendUserEnvironmentVariable(PWSTR* Environment,
//                               LPWSTR lpName,
//                               LPWSTR lpValue)
// {
//     NTSTATUS Status;
//     UNICODE_STRING Name, Value;
//
//     RtlInitUnicodeString(&Name, lpName);
//
//     Value.Length = 0;
//     Value.MaximumLength = 1024 * sizeof(WCHAR);
//     Value.Buffer = LocalAlloc(LPTR, Value.MaximumLength);
//     if (Value.Buffer == NULL)
//         return FALSE;
//
//     Value.Buffer[0] = UNICODE_NULL;
//
//     Status = RtlQueryEnvironmentVariable_U(*Environment,
//                                            &Name,
//                                            &Value);
//     if (NT_SUCCESS(Status))
//         RtlAppendUnicodeToString(&Value, L";");
//
//     RtlAppendUnicodeToString(&Value, lpValue);
//
//     Status = RtlSetEnvironmentVariable(Environment,
//                                        &Name,
//                                        &Value);
//     LocalFree(Value.Buffer);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("RtlSetEnvironmentVariable() failed (Status %lx)\n", Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// static
// HKEY
// GetCurrentUserKey(HANDLE hToken)
// {
//     LONG Error;
//     UNICODE_STRING SidString;
//     HKEY hKey;
//
//     if (!GetUserSidStringFromToken(hToken, &SidString))
//     {
//         DPRINT1("GetUserSidFromToken() failed\n");
//         return NULL;
//     }
//
//     Error = RegOpenKeyExW(HKEY_USERS,
//                           SidString.Buffer,
//                           0,
//                           MAXIMUM_ALLOWED,
//                           &hKey);
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("RegOpenKeyExW() failed (Error %ld)\n", Error);
//         RtlFreeUnicodeString(&SidString);
//         SetLastError((DWORD)Error);
//         return NULL;
//     }
//
//     RtlFreeUnicodeString(&SidString);
//
//     return hKey;
// }
//
//
// static
// BOOL
// GetUserAndDomainName(IN HANDLE hToken,
//                      OUT LPWSTR *UserName,
//                      OUT LPWSTR *DomainName)
// {
//     BOOL bRet = TRUE;
//     PSID Sid = NULL;
//     LPWSTR lpUserName = NULL;
//     LPWSTR lpDomainName = NULL;
//     DWORD cbUserName = 0;
//     DWORD cbDomainName = 0;
//     SID_NAME_USE SidNameUse;
//
//     Sid = GetUserSid(hToken);
//     if (Sid == NULL)
//     {
//         DPRINT1("GetUserSid() failed\n");
//         return FALSE;
//     }
//
//     if (!LookupAccountSidW(NULL,
//                            Sid,
//                            NULL,
//                            &cbUserName,
//                            NULL,
//                            &cbDomainName,
//                            &SidNameUse))
//     {
//         if (GetLastError() != ERROR_INSUFFICIENT_BUFFER)
//         {
//             bRet = FALSE;
//             goto done;
//         }
//     }
//
//     lpUserName = LocalAlloc(LPTR, cbUserName * sizeof(WCHAR));
//     if (lpUserName == NULL)
//     {
//         bRet = FALSE;
//         goto done;
//     }
//
//     lpDomainName = LocalAlloc(LPTR, cbDomainName * sizeof(WCHAR));
//     if (lpDomainName == NULL)
//     {
//         bRet = FALSE;
//         goto done;
//     }
//
//     if (!LookupAccountSidW(NULL,
//                            Sid,
//                            lpUserName,
//                            &cbUserName,
//                            lpDomainName,
//                            &cbDomainName,
//                            &SidNameUse))
//     {
//         bRet = FALSE;
//         goto done;
//     }
//
//     *UserName = lpUserName;
//     *DomainName = lpDomainName;
//
// done:
//     if (bRet == FALSE)
//     {
//         if (lpUserName != NULL)
//             LocalFree(lpUserName);
//
//         if (lpDomainName != NULL)
//             LocalFree(lpDomainName);
//     }
//
//     LocalFree(Sid);
//
//     return bRet;
// }
//
//
// static
// BOOL
// SetUserEnvironment(PWSTR* Environment,
//                    HKEY hKey,
//                    LPWSTR lpSubKeyName)
// {
//     LONG Error;
//     HKEY hEnvKey;
//     DWORD dwValues;
//     DWORD dwMaxValueNameLength;
//     DWORD dwMaxValueDataLength;
//     DWORD dwValueNameLength;
//     DWORD dwValueDataLength;
//     DWORD dwType;
//     DWORD i;
//     LPWSTR lpValueName;
//     LPWSTR lpValueData;
//
//     Error = RegOpenKeyExW(hKey,
//                           lpSubKeyName,
//                           0,
//                           KEY_QUERY_VALUE,
//                           &hEnvKey);
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("RegOpenKeyExW() failed (Error %ld)\n", Error);
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     Error = RegQueryInfoKey(hEnvKey,
//                             NULL,
//                             NULL,
//                             NULL,
//                             NULL,
//                             NULL,
//                             NULL,
//                             &dwValues,
//                             &dwMaxValueNameLength,
//                             &dwMaxValueDataLength,
//                             NULL,
//                             NULL);
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("RegQueryInforKey() failed (Error %ld)\n", Error);
//         RegCloseKey(hEnvKey);
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     if (dwValues == 0)
//     {
//         RegCloseKey(hEnvKey);
//         return TRUE;
//     }
//
//     /* Allocate buffers */
//     dwMaxValueNameLength++;
//     lpValueName = LocalAlloc(LPTR, dwMaxValueNameLength * sizeof(WCHAR));
//     if (lpValueName == NULL)
//     {
//         RegCloseKey(hEnvKey);
//         return FALSE;
//     }
//
//     lpValueData = LocalAlloc(LPTR, dwMaxValueDataLength);
//     if (lpValueData == NULL)
//     {
//         LocalFree(lpValueName);
//         RegCloseKey(hEnvKey);
//         return FALSE;
//     }
//
//     /* Enumerate values */
//     for (i = 0; i < dwValues; i++)
//     {
//         dwValueNameLength = dwMaxValueNameLength;
//         dwValueDataLength = dwMaxValueDataLength;
//
//         Error = RegEnumValueW(hEnvKey,
//                               i,
//                               lpValueName,
//                               &dwValueNameLength,
//                               NULL,
//                               &dwType,
//                               (LPBYTE)lpValueData,
//                               &dwValueDataLength);
//         if (Error == ERROR_SUCCESS)
//         {
//             if (!_wcsicmp(lpValueName, L"PATH"))
//             {
//                 /* Append 'Path' environment variable */
//                 AppendUserEnvironmentVariable(Environment,
//                                               lpValueName,
//                                               lpValueData);
//             }
//             else
//             {
//                 /* Set environment variable */
//                 SetUserEnvironmentVariable(Environment,
//                                            lpValueName,
//                                            lpValueData,
//                                            (dwType == REG_EXPAND_SZ));
//             }
//         }
//         else
//         {
//             LocalFree(lpValueData);
//             LocalFree(lpValueName);
//             RegCloseKey(hEnvKey);
//
//             return FALSE;
//         }
//     }
//
//     LocalFree(lpValueData);
//     LocalFree(lpValueName);
//     RegCloseKey(hEnvKey);
//
//     return TRUE;
// }
//
//
// static
// BOOL
// SetSystemEnvironment(PWSTR* Environment)
// {
//     LONG Error;
//     HKEY hEnvKey;
//     DWORD dwValues;
//     DWORD dwMaxValueNameLength;
//     DWORD dwMaxValueDataLength;
//     DWORD dwValueNameLength;
//     DWORD dwValueDataLength;
//     DWORD dwType;
//     DWORD i;
//     LPWSTR lpValueName;
//     LPWSTR lpValueData;
//
//     Error = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                           L"SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment",
//                           0,
//                           KEY_QUERY_VALUE,
//                           &hEnvKey);
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("RegOpenKeyExW() failed (Error %ld)\n", Error);
//         return FALSE;
//     }
//
//     Error = RegQueryInfoKey(hEnvKey,
//                             NULL,
//                             NULL,
//                             NULL,
//                             NULL,
//                             NULL,
//                             NULL,
//                             &dwValues,
//                             &dwMaxValueNameLength,
//                             &dwMaxValueDataLength,
//                             NULL,
//                             NULL);
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("RegQueryInforKey() failed (Error %ld)\n", Error);
//         RegCloseKey(hEnvKey);
//         return FALSE;
//     }
//
//     if (dwValues == 0)
//     {
//         RegCloseKey(hEnvKey);
//         return TRUE;
//     }
//
//     /* Allocate buffers */
//     dwMaxValueNameLength++;
//     lpValueName = LocalAlloc(LPTR, dwMaxValueNameLength * sizeof(WCHAR));
//     if (lpValueName == NULL)
//     {
//         RegCloseKey(hEnvKey);
//         return FALSE;
//     }
//
//     lpValueData = LocalAlloc(LPTR, dwMaxValueDataLength);
//     if (lpValueData == NULL)
//     {
//         LocalFree(lpValueName);
//         RegCloseKey(hEnvKey);
//         return FALSE;
//     }
//
//     /* Enumerate values */
//     for (i = 0; i < dwValues; i++)
//     {
//         dwValueNameLength = dwMaxValueNameLength;
//         dwValueDataLength = dwMaxValueDataLength;
//
//         Error = RegEnumValueW(hEnvKey,
//                               i,
//                               lpValueName,
//                               &dwValueNameLength,
//                               NULL,
//                               &dwType,
//                               (LPBYTE)lpValueData,
//                               &dwValueDataLength);
//         if (Error == ERROR_SUCCESS)
//         {
//             /* Set environment variable */
//             SetUserEnvironmentVariable(Environment,
//                                        lpValueName,
//                                        lpValueData,
//                                        (dwType == REG_EXPAND_SZ));
//         }
//         else
//         {
//             LocalFree(lpValueData);
//             LocalFree(lpValueName);
//             RegCloseKey(hEnvKey);
//
//             return FALSE;
//         }
//     }
//
//     LocalFree(lpValueData);
//     LocalFree(lpValueName);
//     RegCloseKey(hEnvKey);
//
//     return TRUE;
// }
//
//
// BOOL
// WINAPI
// CreateEnvironmentBlock(OUT LPVOID *lpEnvironment,
//                        IN HANDLE hToken,
//                        IN BOOL bInherit)
// {
//     NTSTATUS Status;
//     LONG lError;
//     PWSTR* Environment = (PWSTR*)lpEnvironment;
//     DWORD Length;
//     DWORD dwType;
//     HKEY hKey;
//     HKEY hKeyUser;
//     LPWSTR lpUserName = NULL;
//     LPWSTR lpDomainName = NULL;
//     WCHAR Buffer[MAX_PATH];
//     WCHAR szValue[1024];
//
//     DPRINT("CreateEnvironmentBlock() called\n");
//
//     if (lpEnvironment == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     Status = RtlCreateEnvironment((BOOLEAN)bInherit, Environment);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("RtlCreateEnvironment() failed (Status %lx)\n", Status);
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     /* Set 'SystemRoot' variable */
//     Length = ARRAYSIZE(Buffer);
//     if (GetEnvironmentVariableW(L"SystemRoot", Buffer, Length))
//     {
//         SetUserEnvironmentVariable(Environment,
//                                    L"SystemRoot",
//                                    Buffer,
//                                    FALSE);
//     }
//
//     /* Set 'SystemDrive' variable */
//     if (GetEnvironmentVariableW(L"SystemDrive", Buffer, Length))
//     {
//         SetUserEnvironmentVariable(Environment,
//                                    L"SystemDrive",
//                                    Buffer,
//                                    FALSE);
//     }
//
//     /* Set variables from Session Manager */
//     if (!SetSystemEnvironment(Environment))
//     {
//         RtlDestroyEnvironment(*Environment);
//         return FALSE;
//     }
//
//     /* Set 'COMPUTERNAME' variable */
//     Length = ARRAYSIZE(Buffer);
//     if (GetComputerNameW(Buffer, &Length))
//     {
//         SetUserEnvironmentVariable(Environment,
//                                    L"COMPUTERNAME",
//                                    Buffer,
//                                    FALSE);
//     }
//
//     /* Set 'ALLUSERSPROFILE' variable */
//     Length = ARRAYSIZE(Buffer);
//     if (GetAllUsersProfileDirectoryW(Buffer, &Length))
//     {
//         SetUserEnvironmentVariable(Environment,
//                                    L"ALLUSERSPROFILE",
//                                    Buffer,
//                                    FALSE);
//     }
//
//     /* Set 'USERPROFILE' variable to the default users profile */
//     Length = ARRAYSIZE(Buffer);
//     if (GetDefaultUserProfileDirectoryW(Buffer, &Length))
//     {
//         SetUserEnvironmentVariable(Environment,
//                                    L"USERPROFILE",
//                                    Buffer,
//                                    TRUE);
//     }
//
//     lError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                            L"SOFTWARE\\Microsoft\\Windows\\CurrentVersion",
//                            0,
//                            KEY_READ,
//                            &hKey);
//     if (lError == ERROR_SUCCESS)
//     {
//         Length = sizeof(szValue);
//         lError = RegQueryValueExW(hKey,
//                                   L"ProgramFilesDir",
//                                   NULL,
//                                   &dwType,
//                                   (LPBYTE)szValue,
//                                   &Length);
//         if (lError == ERROR_SUCCESS)
//         {
//             SetUserEnvironmentVariable(Environment,
//                                        L"ProgramFiles",
//                                        szValue,
//                                        FALSE);
//         }
//
//         Length = sizeof(szValue);
//         lError = RegQueryValueExW(hKey,
//                                   L"CommonFilesDir",
//                                   NULL,
//                                   &dwType,
//                                   (LPBYTE)szValue,
//                                   &Length);
//         if (lError == ERROR_SUCCESS)
//         {
//             SetUserEnvironmentVariable(Environment,
//                                        L"CommonProgramFiles",
//                                        szValue,
//                                        FALSE);
//         }
//
//         RegCloseKey(hKey);
//     }
//
//     /*
//      * If no user token is specified, the system environment variables are set
//      * and we stop here, otherwise continue setting the user-specific variables.
//      */
//     if (hToken == NULL)
//         return TRUE;
//
//     hKeyUser = GetCurrentUserKey(hToken);
//     if (hKeyUser == NULL)
//     {
//         DPRINT1("GetCurrentUserKey() failed\n");
//         RtlDestroyEnvironment(*Environment);
//         return FALSE;
//     }
//
//     /* Set 'USERPROFILE' variable */
//     Length = ARRAYSIZE(Buffer);
//     if (GetUserProfileDirectoryW(hToken, Buffer, &Length))
//     {
//         DWORD MinLen = 2;
//
//         SetUserEnvironmentVariable(Environment,
//                                    L"USERPROFILE",
//                                    Buffer,
//                                    FALSE);
//
//         // FIXME: Strangely enough the following two environment variables
//         // are not set by userenv.dll in Windows... See r68284 / CORE-9875
//         // FIXME2: This is done by msgina.dll !!
//
//         /* At least <drive letter>:<path> */
//         if (Length > MinLen)
//         {
//             /* Set 'HOMEDRIVE' variable */
//             StringCchCopyNW(szValue, ARRAYSIZE(Buffer), Buffer, MinLen);
//             SetUserEnvironmentVariable(Environment,
//                                        L"HOMEDRIVE",
//                                        szValue,
//                                        FALSE);
//
//             /* Set 'HOMEPATH' variable */
//             StringCchCopyNW(szValue, ARRAYSIZE(Buffer), Buffer + MinLen, Length - MinLen);
//             SetUserEnvironmentVariable(Environment,
//                                        L"HOMEPATH",
//                                        szValue,
//                                        FALSE);
//         }
//     }
//     else
//     {
//         DPRINT1("GetUserProfileDirectoryW failed with error %lu\n", GetLastError());
//     }
//
//     if (GetUserAndDomainName(hToken,
//                              &lpUserName,
//                              &lpDomainName))
//     {
//         /* Set 'USERNAME' variable */
//         SetUserEnvironmentVariable(Environment,
//                                    L"USERNAME",
//                                    lpUserName,
//                                    FALSE);
//
//         /* Set 'USERDOMAIN' variable */
//         SetUserEnvironmentVariable(Environment,
//                                    L"USERDOMAIN",
//                                    lpDomainName,
//                                    FALSE);
//
//         if (lpUserName != NULL)
//             LocalFree(lpUserName);
//
//         if (lpDomainName != NULL)
//             LocalFree(lpDomainName);
//     }
//
//     /* Set user environment variables */
//     SetUserEnvironment(Environment,
//                        hKeyUser,
//                        L"Environment");
//
//     /* Set user volatile environment variables */
//     SetUserEnvironment(Environment,
//                        hKeyUser,
//                        L"Volatile Environment");
//
//     RegCloseKey(hKeyUser);
//
//     return TRUE;
// }
//
//
// BOOL
// WINAPI
// DestroyEnvironmentBlock(IN LPVOID lpEnvironment)
// {
//     DPRINT("DestroyEnvironmentBlock() called\n");
//
//     if (lpEnvironment == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     RtlDestroyEnvironment(lpEnvironment);
//
//     return TRUE;
// }
//
//
// BOOL
// WINAPI
// ExpandEnvironmentStringsForUserW(IN HANDLE hToken,
//                                  IN LPCWSTR lpSrc,
//                                  OUT LPWSTR lpDest,
//                                  IN DWORD dwSize)
// {
//     BOOL Ret = FALSE;
//     PVOID lpEnvironment;
//
//     if (lpSrc == NULL || lpDest == NULL || dwSize == 0)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     if (CreateEnvironmentBlock(&lpEnvironment,
//                                hToken,
//                                FALSE))
//     {
//         UNICODE_STRING SrcU, DestU;
//         NTSTATUS Status;
//
//         /* Initialize the strings */
//         RtlInitUnicodeString(&SrcU, lpSrc);
//         DestU.Length = 0;
//         DestU.MaximumLength = dwSize * sizeof(WCHAR);
//         DestU.Buffer = lpDest;
//
//         /* Expand the strings */
//         Status = RtlExpandEnvironmentStrings_U((PWSTR)lpEnvironment,
//                                                &SrcU,
//                                                &DestU,
//                                                NULL);
//
//         DestroyEnvironmentBlock(lpEnvironment);
//
//         if (NT_SUCCESS(Status))
//         {
//             Ret = TRUE;
//         }
//         else
//         {
//             SetLastError(RtlNtStatusToDosError(Status));
//         }
//     }
//
//     return Ret;
// }
//
//
// BOOL
// WINAPI
// ExpandEnvironmentStringsForUserA(IN HANDLE hToken,
//                                  IN LPCSTR lpSrc,
//                                  OUT LPSTR lpDest,
//                                  IN DWORD dwSize)
// {
//     BOOL Ret = FALSE;
//     DWORD dwSrcLen;
//     LPWSTR lpSrcW = NULL, lpDestW = NULL;
//
//     if (lpSrc == NULL || lpDest == NULL || dwSize == 0)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     dwSrcLen = strlen(lpSrc);
//     lpSrcW = (LPWSTR)GlobalAlloc(GMEM_FIXED,
//                                  (dwSrcLen + 1) * sizeof(WCHAR));
//     if (lpSrcW == NULL ||
//         MultiByteToWideChar(CP_ACP,
//                             0,
//                             lpSrc,
//                             -1,
//                             lpSrcW,
//                             dwSrcLen + 1) == 0)
//     {
//         goto Cleanup;
//     }
//
//     lpDestW = (LPWSTR)GlobalAlloc(GMEM_FIXED,
//                                   dwSize * sizeof(WCHAR));
//     if (lpDestW == NULL)
//         goto Cleanup;
//
//     Ret = ExpandEnvironmentStringsForUserW(hToken,
//                                            lpSrcW,
//                                            lpDestW,
//                                            dwSize);
//     if (Ret)
//     {
//         if (WideCharToMultiByte(CP_ACP,
//                                 0,
//                                 lpDestW,
//                                 -1,
//                                 lpDest,
//                                 dwSize,
//                                 NULL,
//                                 NULL) == 0)
//         {
//             Ret = FALSE;
//         }
//     }
//
// Cleanup:
//     if (lpSrcW != NULL)
//         GlobalFree((HGLOBAL)lpSrcW);
//
//     if (lpDestW != NULL)
//         GlobalFree((HGLOBAL)lpDestW);
//
//     return Ret;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
