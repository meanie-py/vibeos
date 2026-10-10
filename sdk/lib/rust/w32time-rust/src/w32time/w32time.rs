// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_23E7B20AEA52: [u16; 53] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069,
    0x0063, 0x0065, 0x0073, 0x005C, 0x0057, 0x0033, 0x0032, 0x0054, 0x0069, 0x006D,
    0x0065, 0x005C, 0x0050, 0x0061, 0x0072, 0x0061, 0x006D, 0x0065, 0x0074, 0x0065,
    0x0072, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_64B1335C67DE: [u16; 7] = [0x004Eu16, 0x006F, 0x0053, 0x0079, 0x006E, 0x0063, 0x0000];
pub static RUSTCONV_WSTR_709779F01788: [u16; 59] = [
    0x0053u16, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F,
    0x006E, 0x005C, 0x0044, 0x0061, 0x0074, 0x0065, 0x0054, 0x0069, 0x006D, 0x0065,
    0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0065, 0x0072, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_7C4E7033DAB7: [u16; 66] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069,
    0x0063, 0x0065, 0x0073, 0x005C, 0x0057, 0x0033, 0x0032, 0x0054, 0x0069, 0x006D,
    0x0065, 0x005C, 0x0054, 0x0069, 0x006D, 0x0065, 0x0050, 0x0072, 0x006F, 0x0076,
    0x0069, 0x0064, 0x0065, 0x0072, 0x0073, 0x005C, 0x004E, 0x0074, 0x0070, 0x0043,
    0x006C, 0x0069, 0x0065, 0x006E, 0x0074, 0x0000,
];
pub static RUSTCONV_WSTR_CE3BA09659D2: [u16; 20] = [
    0x0053u16, 0x0070, 0x0065, 0x0063, 0x0069, 0x0061, 0x006C, 0x0050, 0x006F, 0x006C,
    0x006C, 0x0049, 0x006E, 0x0074, 0x0065, 0x0072, 0x0076, 0x0061, 0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_D47D43486FD2: [u16; 5] = [0x0054u16, 0x0079, 0x0070, 0x0065, 0x0000];

use core::ffi::c_void;

extern "C" {
    fn DbgPrint(Format: PCSTR, ...) -> ULONG;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut ServiceStatus: SERVICE_STATUS = _SERVICE_STATUS {
    dwServiceType: 0,
    dwCurrentState: 0,
    dwControlsAccepted: 0,
    dwWin32ExitCode: 0,
    dwServiceSpecificExitCode: 0,
    dwCheckPoint: 0,
    dwWaitHint: 0,
};

#[no_mangle]
pub static mut hStatus: SERVICE_STATUS_HANDLE =
    ::core::ptr::null::<SERVICE_STATUS_HANDLE__>() as *mut SERVICE_STATUS_HANDLE__;

#[no_mangle]
pub static mut hStopEvent: HANDLE = NULL;

static mut ServiceName: [WCHAR; 8] =
    unsafe { ::core::mem::transmute::<[u8; 16], [WCHAR; 8]>(*b"W\x003\x002\0T\0i\0m\0e\0\0\0") };
#[no_mangle]
pub unsafe extern "C" fn SystemSetTime(lpSystemTime: LPSYSTEMTIME) -> BOOL {
    let mut hToken: HANDLE = core::ptr::null_mut();
    let mut PrevSize: DWORD = 0;
    let mut priv_: TOKEN_PRIVILEGES = core::mem::zeroed();
    let mut previouspriv: TOKEN_PRIVILEGES = core::mem::zeroed();
    let mut Ret: BOOL = FALSE as BOOL;

    if OpenProcessToken(
        GetCurrentProcess(),
        (TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY),
        core::ptr::addr_of_mut!(hToken),
    ) != 0
    {
        priv_.PrivilegeCount = 1;
        priv_.Privileges[0 as usize].Attributes = SE_PRIVILEGE_ENABLED;

        if LookupPrivilegeValueW(
            core::ptr::null_mut(),
            SE_SYSTEMTIME_NAME,
            core::ptr::addr_of_mut!(priv_.Privileges[0 as usize].Luid),
        ) != 0
        {
            if AdjustTokenPrivileges(
                hToken,
                FALSE as BOOL,
                core::ptr::addr_of_mut!(priv_),
                core::mem::size_of::<TOKEN_PRIVILEGES>() as DWORD,
                core::ptr::addr_of_mut!(previouspriv),
                core::ptr::addr_of_mut!(PrevSize),
            ) != 0
                && GetLastError() == ERROR_SUCCESS
            {
                Ret = SetSystemTime(lpSystemTime);

                if previouspriv.PrivilegeCount > 0 {
                    AdjustTokenPrivileges(
                        hToken,
                        FALSE as BOOL,
                        core::ptr::addr_of_mut!(previouspriv),
                        0,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    );
                }
            }
        }
        CloseHandle(hToken);
    }

    return Ret;
}

unsafe extern "C" fn UpdateSystemTime(ulTime: ULONG) -> DWORD {
    let mut ftNew: FILETIME = core::mem::zeroed();
    let mut li: LARGE_INTEGER = core::mem::zeroed();
    let mut stNew: SYSTEMTIME = core::mem::zeroed();

    /* Time at 1st Jan 1900 */
    stNew.wYear = 1900 as WORD;
    stNew.wMonth = 1 as WORD;
    stNew.wDay = 1 as WORD;
    stNew.wHour = 0 as WORD;
    stNew.wMinute = 0 as WORD;
    stNew.wSecond = 0 as WORD;
    stNew.wMilliseconds = 0 as WORD;

    /* Convert to a file time */
    if SystemTimeToFileTime(core::ptr::addr_of_mut!(stNew), core::ptr::addr_of_mut!(ftNew)) == 0 {
        return GetLastError();
    }

    /* Add on the time passed since 1st Jan 1900 */
    li = *(core::ptr::addr_of_mut!(ftNew) as *mut LARGE_INTEGER);
    li.QuadPart += (10000000 as LONGLONG) * (ulTime as LONGLONG);
    ftNew = *(core::ptr::addr_of_mut!(li) as *mut FILETIME);

    /* Convert back to a system time */
    if FileTimeToSystemTime(core::ptr::addr_of_mut!(ftNew), core::ptr::addr_of_mut!(stNew)) == 0 {
        return GetLastError();
    }

    if SystemSetTime(core::ptr::addr_of_mut!(stNew)) == 0 {
        return GetLastError();
    }

    return ERROR_SUCCESS as DWORD;
}

unsafe extern "C" fn GetIntervalSetting() -> DWORD {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwData: DWORD = 0;
    let mut dwSize: DWORD = core::mem::size_of::<DWORD>() as DWORD;
    let mut lRet: LONG = 0;
    dwData = 0 as DWORD;
    lRet = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        RUSTCONV_WSTR_7C4E7033DAB7.as_ptr() as LPCWSTR,
        0 as DWORD,
        KEY_QUERY_VALUE as REGSAM,
        core::ptr::addr_of_mut!(hKey),
    ) as LONG;
    if lRet == ERROR_SUCCESS as LONG {
        // This key holds the update interval in seconds.
        // It is useful for testing to set it to a value of 10 (Decimal).
        // This will cause the clock to try and update every 10 seconds.
        // So you can change the time and expect it to be set back correctly in 10-20 seconds.
        lRet = RegQueryValueExW(
            hKey,
            RUSTCONV_WSTR_CE3BA09659D2.as_ptr() as LPCWSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwData) as LPBYTE,
            core::ptr::addr_of_mut!(dwSize),
        ) as LONG;
        RegCloseKey(hKey);
    }

    if lRet != ERROR_SUCCESS as LONG || dwData < 120 as DWORD {
        return W32TIME_POLL_INTERVAL as DWORD;
    } else {
        return dwData;
    }
}
#[no_mangle]
pub unsafe extern "C" fn SetTime() -> DWORD {
    let mut ulTime: ULONG = 0;
    let mut lRet: LONG = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut szData: [WCHAR; MAX_VALUE_NAME as usize] = [0 as WCHAR; MAX_VALUE_NAME as usize];
    let mut cbName: DWORD = core::mem::size_of::<[WCHAR; MAX_VALUE_NAME as usize]>() as DWORD;

    DbgPrint(b"Entered SetTime.\n\0".as_ptr() as PCSTR);

    lRet = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE as HKEY,
        RUSTCONV_WSTR_709779F01788.as_ptr() as LPCWSTR,
        0 as DWORD,
        KEY_QUERY_VALUE as REGSAM,
        &raw mut hKey,
    ) as LONG;
    if lRet != ERROR_SUCCESS as LONG {
        return lRet as DWORD;
    }

    lRet = RegQueryValueExW(
        hKey,
        core::ptr::null::<WCHAR>(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<DWORD>(),
        &raw mut szData as *mut WCHAR as LPBYTE,
        &raw mut cbName,
    ) as LONG;
    if lRet == ERROR_SUCCESS as LONG {
        cbName = core::mem::size_of::<[WCHAR; MAX_VALUE_NAME as usize]>() as DWORD;
        lRet = RegQueryValueExW(
            hKey,
            &raw mut szData as *mut WCHAR as LPCWSTR,
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<DWORD>(),
            &raw mut szData as *mut WCHAR as LPBYTE,
            &raw mut cbName,
        ) as LONG;
    }

    RegCloseKey(hKey);

    DbgPrint(
        b"Time Server is '%S'.\n\0".as_ptr() as PCSTR,
        &raw mut szData as *mut WCHAR,
    );

    /* Is the given string empty? */
    if cbName == 0 as DWORD
        || szData[0 as usize] == 0 as WCHAR
    {
        DbgPrint(b"The time NTP server couldn't be found, the string is empty!\n\0".as_ptr() as PCSTR);
        return ERROR_INVALID_DATA as DWORD;
    }

    ulTime = GetServerTime(&raw mut szData as *mut WCHAR as LPWSTR) as ULONG;

    if ulTime != 0 as ULONG {
        return UpdateSystemTime(ulTime);
    } else {
        return ERROR_GEN_FAILURE as DWORD;
    }
}
#[no_mangle]
pub unsafe extern "system" fn ControlHandler(request: DWORD) {
    match request {
        SERVICE_CONTROL_STOP | SERVICE_CONTROL_SHUTDOWN => {
            if DbgPrint(
                b"(%s:%d) Stopping W32Time Service\n\0" as *const u8 as PCSTR,
                b"w32time.c\0" as *const u8 as *const ::core::ffi::c_char,
                220 as ::core::ffi::c_int,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                    b"w32time.c\0" as *const u8 as *const ::core::ffi::c_char,
                    220 as ::core::ffi::c_int,
                );
            }

            ServiceStatus.dwWin32ExitCode = 0;
            ServiceStatus.dwCurrentState = SERVICE_STOP_PENDING;
            SetServiceStatus(hStatus, core::ptr::addr_of_mut!(ServiceStatus));

            if !hStopEvent.is_null() {
                SetEvent(hStopEvent);
            }
            return;
        }
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "system" fn W32TmServiceMain(argc: DWORD, argv: *mut LPWSTR) {
    let mut error: LONG;
    let mut dwInterval: DWORD;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut szData: [WCHAR; 8] = [0; 8];
    let mut cbData: DWORD;
    let mut bNoSync: BOOL;

    UNREFERENCED_PARAMETER!(argc);
    UNREFERENCED_PARAMETER!(argv);

    ServiceStatus.dwServiceType = SERVICE_WIN32 as DWORD;
    ServiceStatus.dwCurrentState = SERVICE_START_PENDING as DWORD;
    ServiceStatus.dwControlsAccepted = (SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN) as DWORD;
    ServiceStatus.dwWin32ExitCode = 0 as DWORD;
    ServiceStatus.dwServiceSpecificExitCode = 0 as DWORD;
    ServiceStatus.dwCheckPoint = 0 as DWORD;
    ServiceStatus.dwWaitHint = 0 as DWORD;

    hStatus = RegisterServiceCtrlHandlerW(
        core::ptr::addr_of_mut!(ServiceName) as *mut WCHAR as LPCWSTR,
        Some(ControlHandler),
    );
    if hStatus.is_null() {
        /* Registering Control Handler failed */
        return;
    }

    /* Create the stop event */
    hStopEvent = CreateEventW(
        core::ptr::null_mut(),
        FALSE as BOOL,
        FALSE as BOOL,
        core::ptr::null_mut(),
    );
    if hStopEvent.is_null() {
        ServiceStatus.dwCurrentState = SERVICE_STOPPED as DWORD;
        ServiceStatus.dwWin32ExitCode = GetLastError();
        ServiceStatus.dwControlsAccepted = 0 as DWORD;
        SetServiceStatus(hStatus, core::ptr::addr_of_mut!(ServiceStatus));
        return;
    }

    /* Get the interval */
    dwInterval = GetIntervalSetting();

    /* We report the running status to SCM. */
    ServiceStatus.dwCurrentState = SERVICE_RUNNING as DWORD;
    SetServiceStatus(hStatus, core::ptr::addr_of_mut!(ServiceStatus));

    /* The service's worker loop */
    loop {
        /* The default is NoSync */
        bNoSync = TRUE as BOOL;

        /* TODO: Use RegNotifyChangeKeyValue() when implemented */
        if RegOpenKeyExW(
            HKEY_LOCAL_MACHINE as HKEY,
            RUSTCONV_WSTR_23E7B20AEA52.as_ptr(),
            0 as DWORD,
            KEY_QUERY_VALUE as REGSAM,
            core::ptr::addr_of_mut!(hKey),
        ) == ERROR_SUCCESS as LSTATUS
        {
            cbData = core::mem::size_of::<[WCHAR; 8]>() as DWORD;
            RegQueryValueExW(
                hKey,
                RUSTCONV_WSTR_D47D43486FD2.as_ptr(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(szData) as *mut WCHAR as LPBYTE,
                core::ptr::addr_of_mut!(cbData),
            );
            szData[(ARRAYSIZE!(szData) - 1) as usize] = UNICODE_NULL; /* Avoid buffer overrun */
            bNoSync = (_wcsicmp(
                core::ptr::addr_of_mut!(szData) as *mut WCHAR,
                RUSTCONV_WSTR_64B1335C67DE.as_ptr(),
            ) == 0) as BOOL;
            RegCloseKey(hKey);
        }

        if bNoSync == 0 {
            error = SetTime() as LONG;
            if error != ERROR_SUCCESS as LONG {
                DPRINT!(
                    b"W32Time Service failed to set clock: 0x%08lX\n\0".as_ptr() as *const i8,
                    error,
                );
            }
        }

        if WaitForSingleObject(hStopEvent, dwInterval * 1000 as DWORD)
            == WAIT_OBJECT_0 as DWORD
        {
            CloseHandle(hStopEvent);
            hStopEvent = core::ptr::null_mut();

            ServiceStatus.dwCurrentState = SERVICE_STOPPED as DWORD;
            ServiceStatus.dwWin32ExitCode = ERROR_SUCCESS as DWORD;
            ServiceStatus.dwControlsAccepted = 0 as DWORD;
            SetServiceStatus(hStatus, core::ptr::addr_of_mut!(ServiceStatus));
            DPRINT!(b"Stopped W32Time Service\n\0".as_ptr() as *const i8);
            return;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    fdwReason: DWORD,
    lpvReserved: LPVOID,
) -> BOOL {
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinstDLL as HMODULE);
        }
        DLL_PROCESS_DETACH => {}
        _ => {}
    }

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn W32TimeSyncNow(
    cmdline: LPCWSTR,
    blocking: UINT,
    flags: UINT,
) -> DWORD {
    let mut result: DWORD;
    result = SetTime();
    if result != 0 {
        DPRINT!("W32TimeSyncNow failed and clock not set.\n");
    } else {
        DPRINT!("W32TimeSyncNow succeeded and clock set.\n");
    }
    return result;
}
