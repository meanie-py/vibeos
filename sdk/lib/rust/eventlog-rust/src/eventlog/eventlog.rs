// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0AF4B1084E0C: [u16; 8] = [0x004Du16, 0x0061, 0x0078, 0x0053, 0x0069, 0x007A, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_17BA4ECEC082: [u16; 43] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069,
    0x0063, 0x0065, 0x0073, 0x005C, 0x0045, 0x0076, 0x0065, 0x006E, 0x0074, 0x004C,
    0x006F, 0x0067, 0x0000,
];
pub static RUSTCONV_WSTR_1965DA50B0FA: [u16; 12] = [
    0x0043u16, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0054, 0x0079, 0x0070,
    0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_2F23369E2DF4: [u16; 10] = [0x0052u16, 0x0065, 0x0074, 0x0065, 0x006E, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000];
pub static RUSTCONV_WSTR_5D92AA1DCBC8: [u16; 5] = [0x002Eu16, 0x0065, 0x0076, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_658E528E34B0: [u16; 8] = [0x0025u16, 0x006C, 0x0075, 0x002E, 0x0025, 0x006C, 0x0075, 0x0000];
pub static RUSTCONV_WSTR_8B7548DD541B: [u16; 30] = [
    0x0025u16, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0052, 0x006F, 0x006F,
    0x0074, 0x0025, 0x005C, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0033,
    0x0032, 0x005C, 0x0043, 0x006F, 0x006E, 0x0066, 0x0069, 0x0067, 0x005C, 0x0000,
];
pub static RUSTCONV_WSTR_ABE36AB10481: [u16; 9] = [0x0045u16, 0x0076, 0x0065, 0x006E, 0x0074, 0x004C, 0x006F, 0x0067, 0x0000];
pub static RUSTCONV_WSTR_C4BA81AEA105: [u16; 5] = [0x0046u16, 0x0069, 0x006C, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_F3C6A8F2AC47: [u16; 4] = [0x0025u16, 0x006C, 0x0075, 0x0000];
pub static RUSTCONV_WSTR_F82E64E70394: [u16; 45] = [
    0x0053u16, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054, 0x005C,
    0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072,
    0x0073, 0x0069, 0x006F, 0x006E, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut ServiceName: [WCHAR; 9] =
    unsafe { ::core::mem::transmute::<[u8; 18], [WCHAR; 9]>(*b"E\0v\0e\0n\0t\0L\0o\0g\0\0\0") };

static mut ServiceTable: [SERVICE_TABLE_ENTRYW; 2] = unsafe {
    [
        _SERVICE_TABLE_ENTRYW {
            lpServiceName: &raw const ServiceName as LPWSTR,
            lpServiceProc: Some(ServiceMain),
        },
        _SERVICE_TABLE_ENTRYW {
            lpServiceName: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            lpServiceProc: None,
        },
    ]
};

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
pub static mut ServiceStatusHandle: SERVICE_STATUS_HANDLE =
    ::core::ptr::null::<SERVICE_STATUS_HANDLE__>() as *mut SERVICE_STATUS_HANDLE__;

#[no_mangle]
pub static mut onLiveCD: BOOL = (FALSE) as i32;

#[no_mangle]
pub static mut EventLogSource: PEVENTSOURCE =
    ::core::ptr::null::<_EVENTSOURCE>() as *mut _EVENTSOURCE;

// #define NDEBUG (valueless flag, no constant emitted)

// static VOID __stdcall ServiceMain(DWORD, LPWSTR*); (prototype only; defined elsewhere)

unsafe extern "C" fn UpdateServiceStatus(dwState: DWORD) {
    ServiceStatus.dwServiceType = SERVICE_WIN32_OWN_PROCESS;
    ServiceStatus.dwCurrentState = dwState;
    ServiceStatus.dwControlsAccepted = 0;
    ServiceStatus.dwWin32ExitCode = 0;
    ServiceStatus.dwServiceSpecificExitCode = 0;
    ServiceStatus.dwCheckPoint = 0;

    if dwState == SERVICE_START_PENDING
        || dwState == SERVICE_STOP_PENDING
        || dwState == SERVICE_PAUSE_PENDING
        || dwState == SERVICE_CONTINUE_PENDING
    {
        ServiceStatus.dwWaitHint = 10000;
    } else {
        ServiceStatus.dwWaitHint = 0;
    }

    SetServiceStatus(ServiceStatusHandle, core::ptr::addr_of_mut!(ServiceStatus));
}

unsafe extern "system" fn ServiceControlHandler(
    dwControl: DWORD,
    dwEventType: DWORD,
    lpEventData: LPVOID,
    lpContext: LPVOID,
) -> DWORD {
    DPRINT!(b"ServiceControlHandler() called\n\0".as_ptr() as *const i8);

    match dwControl {
        SERVICE_CONTROL_STOP => {
            DPRINT!(b"  SERVICE_CONTROL_STOP received\n\0".as_ptr() as *const i8);

            LogfReportEvent(
                EVENTLOG_INFORMATION_TYPE as USHORT,
                0,
                EVENT_EventlogStopped,
                0,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
            );

            /* Stop listening to incoming RPC messages */
            RpcMgmtStopServerListening(core::ptr::null_mut());
            UpdateServiceStatus(SERVICE_STOPPED);
            return ERROR_SUCCESS;
        }

        SERVICE_CONTROL_PAUSE => {
            DPRINT!(b"  SERVICE_CONTROL_PAUSE received\n\0".as_ptr() as *const i8);
            UpdateServiceStatus(SERVICE_PAUSED);
            return ERROR_SUCCESS;
        }

        SERVICE_CONTROL_CONTINUE => {
            DPRINT!(b"  SERVICE_CONTROL_CONTINUE received\n\0".as_ptr() as *const i8);
            UpdateServiceStatus(SERVICE_RUNNING);
            return ERROR_SUCCESS;
        }

        SERVICE_CONTROL_INTERROGATE => {
            DPRINT!(b"  SERVICE_CONTROL_INTERROGATE received\n\0".as_ptr() as *const i8);
            SetServiceStatus(
                ServiceStatusHandle,
                core::ptr::addr_of_mut!(ServiceStatus),
            );
            return ERROR_SUCCESS;
        }

        SERVICE_CONTROL_SHUTDOWN => {
            DPRINT!(b"  SERVICE_CONTROL_SHUTDOWN received\n\0".as_ptr() as *const i8);

            LogfReportEvent(
                EVENTLOG_INFORMATION_TYPE as USHORT,
                0,
                EVENT_EventlogStopped,
                0,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
            );

            UpdateServiceStatus(SERVICE_STOPPED);
            return ERROR_SUCCESS;
        }

        _ => {
            DPRINT1!(
                b"  Control %lu received\n\0".as_ptr() as *const i8,
                dwControl,
            );
            return ERROR_CALL_NOT_IMPLEMENTED;
        }
    }
}

unsafe extern "C" fn ServiceInit() -> DWORD {
    let mut hThread: HANDLE = CreateThread(
        core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
        0 as DWORD,
        ::core::mem::transmute::<
            Option<unsafe extern "system" fn(PVOID) -> NTSTATUS>,
            LPTHREAD_START_ROUTINE,
        >(Some(
            PortThreadRoutine as unsafe extern "system" fn(PVOID) -> NTSTATUS,
        )),
        core::ptr::null_mut(),
        0 as DWORD,
        core::ptr::null_mut::<DWORD>(),
    );
    if hThread.is_null() {
        DPRINT!("Cannot create PortThread\n");
        return GetLastError();
    } else {
        CloseHandle(hThread);
    }

    hThread = CreateThread(
        core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
        0 as DWORD,
        Some(RpcThreadRoutine as unsafe extern "system" fn(LPVOID) -> DWORD),
        core::ptr::null_mut(),
        0 as DWORD,
        core::ptr::null_mut::<DWORD>(),
    );

    if hThread.is_null() {
        DPRINT!("Cannot create RpcThread\n");
        return GetLastError();
    } else {
        CloseHandle(hThread);
    }

    return ERROR_SUCCESS as DWORD;
}

unsafe extern "C" fn ReportProductInfoEvent() {
    let mut versionInfo: OSVERSIONINFOW = core::mem::zeroed();
    let mut szBuffer: [WCHAR; 512] = [0; 512];
    let mut str: PWSTR = core::ptr::null_mut();
    let mut cchRemain: size_t = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwValueLength: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut lResult: LONG = ERROR_SUCCESS as LONG;

    ZeroMemory!(
        core::ptr::addr_of_mut!(versionInfo) as *mut core::ffi::c_void,
        core::mem::size_of::<OSVERSIONINFOW>(),
    );
    versionInfo.dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFOW>() as DWORD;

    /* Get version information */
    if GetVersionExW(core::ptr::addr_of_mut!(versionInfo)) == 0 {
        return;
    }

    ZeroMemory!(
        core::ptr::addr_of_mut!(szBuffer) as *mut core::ffi::c_void,
        core::mem::size_of::<[WCHAR; 512]>(),
    );
    str = core::ptr::addr_of_mut!(szBuffer) as PWSTR;
    cchRemain = ARRAYSIZE!(szBuffer) as size_t;

    /* Write the version number into the buffer */
    StringCchPrintfExW!(
        str,
        cchRemain,
        core::ptr::addr_of_mut!(str),
        core::ptr::addr_of_mut!(cchRemain),
        0 as DWORD,
        RUSTCONV_WSTR_658E528E34B0.as_ptr(),
        versionInfo.dwMajorVersion,
        versionInfo.dwMinorVersion,
    );
    str = str.add(1);
    cchRemain += 1;

    /* Write the build number into the buffer */
    StringCchPrintfExW!(
        str,
        cchRemain,
        core::ptr::addr_of_mut!(str),
        core::ptr::addr_of_mut!(cchRemain),
        0 as DWORD,
        RUSTCONV_WSTR_F3C6A8F2AC47.as_ptr(),
        versionInfo.dwBuildNumber,
    );
    str = str.add(1);
    cchRemain += 1;

    /* Write the service pack info into the buffer */
    StringCchCopyExW(
        str,
        cchRemain,
        core::ptr::addr_of_mut!(versionInfo.szCSDVersion) as *const u16,
        core::ptr::addr_of_mut!(str),
        core::ptr::addr_of_mut!(cchRemain),
        0 as DWORD,
    );
    str = str.add(1);
    cchRemain += 1;

    /* Read 'CurrentType' from the registry and write it into the buffer */
    lResult = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        RUSTCONV_WSTR_F82E64E70394.as_ptr(),
        0 as DWORD,
        KEY_QUERY_VALUE as REGSAM,
        core::ptr::addr_of_mut!(hKey),
    ) as LONG;
    if lResult == ERROR_SUCCESS as LONG {
        dwValueLength = cchRemain as DWORD;
        lResult = RegQueryValueExW(
            hKey,
            RUSTCONV_WSTR_1965DA50B0FA.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwType),
            str as LPBYTE,
            core::ptr::addr_of_mut!(dwValueLength),
        ) as LONG;

        RegCloseKey(hKey);
    }

    /* Log the product information */
    LogfReportEvent(
        EVENTLOG_INFORMATION_TYPE as USHORT,
        0 as USHORT,
        EVENT_EventLogProductInfo,
        4 as USHORT,
        core::ptr::addr_of_mut!(szBuffer) as PWSTR,
        0 as ULONG,
        core::ptr::null_mut(),
    );
}

unsafe extern "system" fn ServiceMain(argc: DWORD, argv: *mut LPWSTR) {
    let mut dwError: DWORD;

    UNREFERENCED_PARAMETER!(argc);
    UNREFERENCED_PARAMETER!(argv);

    DPRINT!("ServiceMain() called\n");

    ServiceStatusHandle = RegisterServiceCtrlHandlerExW(
        core::ptr::addr_of_mut!(ServiceName) as *mut WCHAR as LPCWSTR,
        Some(ServiceControlHandler),
        core::ptr::null_mut(),
    );
    if ServiceStatusHandle.is_null() {
        dwError = GetLastError();
        DPRINT1!("RegisterServiceCtrlHandlerW() failed! (Error %lu)\n", dwError);
        return;
    }

    UpdateServiceStatus(SERVICE_START_PENDING);

    dwError = ServiceInit();
    if dwError != ERROR_SUCCESS {
        DPRINT!("Service stopped (dwError: %lu\n", dwError);
        UpdateServiceStatus(SERVICE_START_PENDING);
    } else {
        DPRINT!("Service started\n");
        UpdateServiceStatus(SERVICE_RUNNING);

        ReportProductInfoEvent();

        LogfReportEvent(
            EVENTLOG_INFORMATION_TYPE as USHORT,
            0,
            EVENT_EventlogStarted,
            0,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
        );
    }

    DPRINT!("ServiceMain() done\n");
}

unsafe extern "C" fn LoadLogFile(hKey: HKEY, LogName: PWSTR) -> PLOGFILE {
    let mut MaxValueLen: DWORD = 0;
    let mut ValueLen: DWORD = 0;
    let mut Type: DWORD = 0;
    let mut ExpandedLen: DWORD = 0;
    let mut Buf: PWSTR = core::ptr::null_mut();
    let mut Expanded: PWSTR = core::ptr::null_mut();
    let mut Result: LONG = 0;
    let mut pLogf: PLOGFILE = core::ptr::null_mut();
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let mut ulMaxSize: ULONG = 0;
    let mut ulRetention: ULONG = 0;
    let mut Status: NTSTATUS = 0;

    DbgPrint(b"LoadLogFile: `%S'\n\0" as *const u8 as PCSTR, LogName);

    Result = RegQueryInfoKeyW(hKey, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(),
                              core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(MaxValueLen), core::ptr::null_mut(), core::ptr::null_mut()) as LONG;
    if Result != ERROR_SUCCESS as LONG {
        DbgPrint(b"RegQueryInfoKeyW failed: %lu\n\0" as *const u8 as PCSTR, Result);
        return core::ptr::null_mut();
    }

    MaxValueLen = ROUND_DOWN!(MaxValueLen as usize, core::mem::size_of::<WCHAR>()) as DWORD;
    Buf = HeapAlloc(GetProcessHeap(), 0, MaxValueLen as SIZE_T) as PWSTR;
    if Buf.is_null() {
        DbgPrint(b"Cannot allocate heap!\n\0" as *const u8 as PCSTR);
        return core::ptr::null_mut();
    }

    ValueLen = MaxValueLen;
    Result = RegQueryValueExW(hKey,
                              RUSTCONV_WSTR_C4BA81AEA105.as_ptr(),
                              core::ptr::null_mut(),
                              core::ptr::addr_of_mut!(Type),
                              Buf as LPBYTE,
                              core::ptr::addr_of_mut!(ValueLen)) as LONG;
    /*
     * If we failed, because the registry value was inexistent
     * or the value type was incorrect, create a new "File" value
     * that holds the default event log path.
     */
    if (Result != ERROR_SUCCESS as LONG) || (Type != REG_EXPAND_SZ && Type != REG_SZ) {
        MaxValueLen = ((wcslen(RUSTCONV_WSTR_8B7548DD541B.as_ptr()) as usize
                       + wcslen(LogName as *const u16) as usize
                       + wcslen(RUSTCONV_WSTR_5D92AA1DCBC8.as_ptr()) as usize + 1 as usize)
                       * core::mem::size_of::<WCHAR>()) as DWORD;

        Expanded = HeapReAlloc(GetProcessHeap(), 0, Buf as *mut core::ffi::c_void, MaxValueLen as SIZE_T) as PWSTR;
        if Expanded.is_null() {
            DbgPrint(b"Cannot reallocate heap!\n\0" as *const u8 as PCSTR);
            HeapFree(GetProcessHeap(), 0, Buf as *mut core::ffi::c_void);
            return core::ptr::null_mut();
        }
        Buf = Expanded;

        StringCbCopyW(Buf, MaxValueLen as usize, RUSTCONV_WSTR_8B7548DD541B.as_ptr());
        StringCbCatW(Buf, MaxValueLen as usize, LogName as *const u16);
        StringCbCatW(Buf, MaxValueLen as usize, RUSTCONV_WSTR_5D92AA1DCBC8.as_ptr());

        ValueLen = MaxValueLen;
        Result = RegSetValueExW(hKey,
                                RUSTCONV_WSTR_C4BA81AEA105.as_ptr(),
                                0,
                                REG_EXPAND_SZ,
                                Buf as LPBYTE as *const u8,
                                ValueLen) as LONG;
        if Result != ERROR_SUCCESS as LONG {
            DbgPrint(b"RegSetValueExW failed: %lu\n\0" as *const u8 as PCSTR, Result);
            HeapFree(GetProcessHeap(), 0, Buf as *mut core::ffi::c_void);
            return core::ptr::null_mut();
        }
    }

    ExpandedLen = ExpandEnvironmentStringsW(Buf as *const u16, core::ptr::null_mut(), 0) as DWORD;
    Expanded = HeapAlloc(GetProcessHeap(), 0, (ExpandedLen as SIZE_T) * core::mem::size_of::<WCHAR>() as SIZE_T) as PWSTR;
    if Expanded.is_null() {
        DbgPrint(b"Cannot allocate heap!\n\0" as *const u8 as PCSTR);
        HeapFree(GetProcessHeap(), 0, Buf as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    ExpandEnvironmentStringsW(Buf as *const u16, Expanded, ExpandedLen);

    if RtlDosPathNameToNtPathName_U(Expanded as *const u16, core::ptr::addr_of_mut!(FileName), core::ptr::null_mut(), core::ptr::null_mut()) == 0 {
        DbgPrint(b"Cannot convert path!\n\0" as *const u8 as PCSTR);
        HeapFree(GetProcessHeap(), 0, Expanded as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, Buf as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    DbgPrint(b"%S -> %S\n\0" as *const u8 as PCSTR, Buf, Expanded);

    ValueLen = core::mem::size_of::<ULONG>() as DWORD;
    Result = RegQueryValueExW(hKey,
                              RUSTCONV_WSTR_0AF4B1084E0C.as_ptr(),
                              core::ptr::null_mut(),
                              core::ptr::addr_of_mut!(Type),
                              core::ptr::addr_of_mut!(ulMaxSize) as LPBYTE,
                              core::ptr::addr_of_mut!(ValueLen)) as LONG;
    if (Result != ERROR_SUCCESS as LONG) || (Type != REG_DWORD) {
        ulMaxSize = 512 * 1024; /* 512 kBytes */

        Result = RegSetValueExW(hKey,
                                RUSTCONV_WSTR_0AF4B1084E0C.as_ptr(),
                                0,
                                REG_DWORD,
                                core::ptr::addr_of_mut!(ulMaxSize) as LPBYTE as *const u8,
                                core::mem::size_of::<ULONG>() as DWORD) as LONG;
    }

    ValueLen = core::mem::size_of::<ULONG>() as DWORD;
    Result = RegQueryValueExW(hKey,
                              RUSTCONV_WSTR_2F23369E2DF4.as_ptr(),
                              core::ptr::null_mut(),
                              core::ptr::addr_of_mut!(Type),
                              core::ptr::addr_of_mut!(ulRetention) as LPBYTE,
                              core::ptr::addr_of_mut!(ValueLen)) as LONG;
    if (Result != ERROR_SUCCESS as LONG) || (Type != REG_DWORD) {
        /* On Windows 2003 it is 604800 (secs) == 7 days */
        ulRetention = 0;

        Result = RegSetValueExW(hKey,
                                RUSTCONV_WSTR_2F23369E2DF4.as_ptr(),
                                0,
                                REG_DWORD,
                                core::ptr::addr_of_mut!(ulRetention) as LPBYTE as *const u8,
                                core::mem::size_of::<ULONG>() as DWORD) as LONG;
    }

    // TODO: Add, or use, default values for "AutoBackupLogFiles" (REG_DWORD)
    // and "CustomSD" (REG_SZ).

    Status = LogfCreate(core::ptr::addr_of_mut!(pLogf), LogName as *const u16, core::ptr::addr_of_mut!(FileName), ulMaxSize, ulRetention, TRUE as BOOLEAN, FALSE as BOOLEAN);
    if !NT_SUCCESS!(Status) {
        DbgPrint(b"Failed to create %S! (Status %08lx)\n\0" as *const u8 as PCSTR, Expanded, Status);
    }

    HeapFree(GetProcessHeap(), 0, Expanded as *mut core::ffi::c_void);
    HeapFree(GetProcessHeap(), 0, Buf as *mut core::ffi::c_void);
    RtlFreeHeap(RtlGetProcessHeap!(), 0, FileName.Buffer as *mut core::ffi::c_void);
    return pLogf;
}

unsafe extern "C" fn LoadLogFiles(mut eventlogKey: HKEY) -> BOOL {
    let mut Result: LONG = 0;
    let mut MaxLognameLen: DWORD = 0;
    let mut LognameLen: DWORD = 0;
    let mut dwIndex: DWORD = 0;
    let mut Buf: PWSTR = core::ptr::null_mut::<WCHAR>();
    let mut pLogFile: PLOGFILE = core::ptr::null_mut::<_LOGFILE>();
    Result = RegQueryInfoKeyW(
        eventlogKey,
        core::ptr::null_mut::<WCHAR>(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::addr_of_mut!(MaxLognameLen),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<_FILETIME>(),
    ) as LONG;
    if Result != ERROR_SUCCESS as LONG {
        if DbgPrint(
            b"(%s:%d) RegQueryInfoKeyW failed: %lu\n\0" as *const u8 as PCSTR,
            b"base/services/eventlog/eventlog.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            442 as ::core::ffi::c_int,
            Result,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"base/services/eventlog/eventlog.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                442 as ::core::ffi::c_int,
            );
        }
        return FALSE as BOOL;
    }
    MaxLognameLen = MaxLognameLen.wrapping_add(1);
    Buf = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (MaxLognameLen as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as PWSTR;
    if Buf.is_null() {
        if DbgPrint(
            b"(%s:%d) Error: cannot allocate heap!\n\0" as *const u8 as PCSTR,
            b"base/services/eventlog/eventlog.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            451 as ::core::ffi::c_int,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"base/services/eventlog/eventlog.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                451 as ::core::ffi::c_int,
            );
        }
        return FALSE as BOOL;
    }
    LognameLen = MaxLognameLen;
    dwIndex = 0 as DWORD;
    while RegEnumKeyExW(
        eventlogKey,
        dwIndex,
        Buf,
        core::ptr::addr_of_mut!(LognameLen),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<WCHAR>(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<_FILETIME>(),
    ) == ERROR_SUCCESS as LSTATUS
    {
        let mut SubKey: HKEY = core::ptr::null_mut::<HKEY__>();
        DbgPrint(
            b"(%s:%d) %S\n\0" as *const u8 as PCSTR,
            b"base/services/eventlog/eventlog.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            463 as ::core::ffi::c_int,
            Buf,
        );
        Result = RegOpenKeyExW(
            eventlogKey,
            Buf as LPCWSTR,
            0 as DWORD,
            KEY_ALL_ACCESS as REGSAM,
            core::ptr::addr_of_mut!(SubKey),
        ) as LONG;
        if Result != ERROR_SUCCESS as LONG {
            if DbgPrint(
                b"(%s:%d) Failed to open %S key.\n\0" as *const u8 as PCSTR,
                b"base/services/eventlog/eventlog.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                470 as ::core::ffi::c_int,
                Buf,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                    b"base/services/eventlog/eventlog.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    470 as ::core::ffi::c_int,
                );
            }
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                Buf as PVOID,
            );
            return FALSE as BOOL;
        }
        pLogFile = LoadLogFile(SubKey, Buf);
        if !pLogFile.is_null() {
            DbgPrint(
                b"(%s:%d) Loaded %S\n\0" as *const u8 as PCSTR,
                b"base/services/eventlog/eventlog.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                478 as ::core::ffi::c_int,
                Buf,
            );
            LoadEventSources(SubKey, pLogFile);
        } else {
            if DbgPrint(
                b"(%s:%d) Failed to load %S\n\0" as *const u8 as PCSTR,
                b"base/services/eventlog/eventlog.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                483 as ::core::ffi::c_int,
                Buf,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                    b"base/services/eventlog/eventlog.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    483 as ::core::ffi::c_int,
                );
            }
        }
        RegCloseKey(SubKey);
        LognameLen = MaxLognameLen;
        dwIndex = dwIndex.wrapping_add(1);
    }
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        Buf as PVOID,
    );
    return TRUE as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wmain(mut argc: ::core::ffi::c_int, mut argv: *mut *mut WCHAR) -> ::core::ffi::c_int {
    let mut RetCode: INT = 0;
    let mut Result: LONG = 0;
    let mut elogKey: HKEY = core::ptr::null_mut();
    let mut LogPath: [WCHAR; MAX_PATH as usize] = [0 as WCHAR; MAX_PATH as usize];

    LogfListInitialize();
    InitEventSourceList();

    GetSystemWindowsDirectoryW(core::ptr::addr_of_mut!(LogPath) as LPWSTR, ARRAYSIZE!(LogPath));

    'bye_bye: {
        if GetDriveTypeW(core::ptr::addr_of_mut!(LogPath) as LPCWSTR) == DRIVE_CDROM {
            DPRINT!("LiveCD detected\n");
            onLiveCD = TRUE as BOOL;
        } else {
            Result = RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                RUSTCONV_WSTR_17BA4ECEC082.as_ptr(),
                0,
                KEY_ALL_ACCESS,
                core::ptr::addr_of_mut!(elogKey),
            ) as LONG;
            if Result != ERROR_SUCCESS as LONG {
                DPRINT1!("Fatal error: cannot open eventlog registry key.\n");
                RetCode = 1;
                break 'bye_bye;
            }

            LoadLogFiles(elogKey);
        }

        EventLogSource = GetEventSourceByName(RUSTCONV_WSTR_ABE36AB10481.as_ptr());
        if EventLogSource.is_null() {
            DPRINT1!("The 'EventLog' source is unavailable. The EventLog service will not be able to log its own events.\n");
        }

        StartServiceCtrlDispatcher(core::ptr::addr_of_mut!(ServiceTable) as *mut SERVICE_TABLE_ENTRYW);
    }

    LogfCloseAll();

    return RetCode;
}
#[no_mangle]
pub unsafe extern "C" fn PRINT_RECORD(mut pRec: PEVENTLOGRECORD) {
    let mut i: UINT = 0;
    let mut str: PWSTR = core::ptr::null_mut::<WCHAR>();
    let mut SystemTime: LARGE_INTEGER = core::mem::zeroed();
    let mut Time: TIME_FIELDS = core::mem::zeroed();
    if DbgPrint(
        b"(%s:%d) PRINT_RECORD(0x%p)\n\0" as *const u8 as PCSTR,
        b"base/services/eventlog/eventlog.c\0"
            as *const u8 as *const core::ffi::c_char,
        552 as core::ffi::c_int,
        pRec,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"base/services/eventlog/eventlog.c\0"
                as *const u8 as *const core::ffi::c_char,
            552 as core::ffi::c_int,
        );
    }
    DbgPrint(b"Length = %lu\n\0" as *const u8 as PCSTR, (*pRec).Length);
    DbgPrint(b"Reserved = 0x%x\n\0" as *const u8 as PCSTR, (*pRec).Reserved);
    DbgPrint(b"RecordNumber = %lu\n\0" as *const u8 as PCSTR, (*pRec).RecordNumber);

    RtlSecondsSince1970ToTime((*pRec).TimeGenerated as ULONG, core::ptr::addr_of_mut!(SystemTime));
    RtlTimeToTimeFields(core::ptr::addr_of_mut!(SystemTime), core::ptr::addr_of_mut!(Time));
    DbgPrint(
        b"TimeGenerated = %hu.%hu.%hu %hu:%hu:%hu\n\0" as *const u8 as PCSTR,
        Time.Day as core::ffi::c_int,
        Time.Month as core::ffi::c_int,
        Time.Year as core::ffi::c_int,
        Time.Hour as core::ffi::c_int,
        Time.Minute as core::ffi::c_int,
        Time.Second as core::ffi::c_int,
    );

    RtlSecondsSince1970ToTime((*pRec).TimeWritten as ULONG, core::ptr::addr_of_mut!(SystemTime));
    RtlTimeToTimeFields(core::ptr::addr_of_mut!(SystemTime), core::ptr::addr_of_mut!(Time));
    DbgPrint(
        b"TimeWritten = %hu.%hu.%hu %hu:%hu:%hu\n\0" as *const u8 as PCSTR,
        Time.Day as core::ffi::c_int,
        Time.Month as core::ffi::c_int,
        Time.Year as core::ffi::c_int,
        Time.Hour as core::ffi::c_int,
        Time.Minute as core::ffi::c_int,
        Time.Second as core::ffi::c_int,
    );

    DbgPrint(b"EventID = %lu\n\0" as *const u8 as PCSTR, (*pRec).EventID);

    match (*pRec).EventType as UINT {
        EVENTLOG_ERROR_TYPE => {
            DbgPrint(b"EventType = EVENTLOG_ERROR_TYPE\n\0" as *const u8 as PCSTR);
        }
        EVENTLOG_WARNING_TYPE => {
            DbgPrint(b"EventType = EVENTLOG_WARNING_TYPE\n\0" as *const u8 as PCSTR);
        }
        EVENTLOG_INFORMATION_TYPE => {
            DbgPrint(b"EventType = EVENTLOG_INFORMATION_TYPE\n\0" as *const u8 as PCSTR);
        }
        EVENTLOG_AUDIT_SUCCESS => {
            DbgPrint(b"EventType = EVENTLOG_AUDIT_SUCCESS\n\0" as *const u8 as PCSTR);
        }
        EVENTLOG_AUDIT_FAILURE => {
            DbgPrint(b"EventType = EVENTLOG_AUDIT_FAILURE\n\0" as *const u8 as PCSTR);
        }
        _ => {
            DbgPrint(
                b"EventType = %hu\n\0" as *const u8 as PCSTR,
                (*pRec).EventType as core::ffi::c_int,
            );
        }
    }

    DbgPrint(
        b"NumStrings = %hu\n\0" as *const u8 as PCSTR,
        (*pRec).NumStrings as core::ffi::c_int,
    );
    DbgPrint(
        b"EventCategory = %hu\n\0" as *const u8 as PCSTR,
        (*pRec).EventCategory as core::ffi::c_int,
    );
    DbgPrint(
        b"ReservedFlags = 0x%x\n\0" as *const u8 as PCSTR,
        (*pRec).ReservedFlags as core::ffi::c_int,
    );
    DbgPrint(
        b"ClosingRecordNumber = %lu\n\0" as *const u8 as PCSTR,
        (*pRec).ClosingRecordNumber,
    );
    DbgPrint(b"StringOffset = %lu\n\0" as *const u8 as PCSTR, (*pRec).StringOffset);
    DbgPrint(b"UserSidLength = %lu\n\0" as *const u8 as PCSTR, (*pRec).UserSidLength);
    DbgPrint(b"UserSidOffset = %lu\n\0" as *const u8 as PCSTR, (*pRec).UserSidOffset);
    DbgPrint(b"DataLength = %lu\n\0" as *const u8 as PCSTR, (*pRec).DataLength);
    DbgPrint(b"DataOffset = %lu\n\0" as *const u8 as PCSTR, (*pRec).DataOffset);

    i = ::core::mem::size_of::<EVENTLOGRECORD>() as UINT;
    DbgPrint(
        b"SourceName: %S\n\0" as *const u8 as PCSTR,
        (pRec as ULONG_PTR).wrapping_add(i as ULONG_PTR) as PWSTR,
    );

    i = (i as usize).wrapping_add(
        (wcslen(
            (pRec as ULONG_PTR).wrapping_add(i as ULONG_PTR) as PWSTR as *const wchar_t,
        ) as usize)
            .wrapping_add(1 as usize)
            .wrapping_mul(::core::mem::size_of::<WCHAR>()),
    ) as UINT;
    DbgPrint(
        b"ComputerName: %S\n\0" as *const u8 as PCSTR,
        (pRec as ULONG_PTR).wrapping_add(i as ULONG_PTR) as PWSTR,
    );

    if (*pRec).StringOffset < (*pRec).Length && (*pRec).NumStrings as core::ffi::c_int != 0 {
        DbgPrint(b"Strings:\n\0" as *const u8 as PCSTR);
        str = (pRec as ULONG_PTR).wrapping_add((*pRec).StringOffset as ULONG_PTR) as PWSTR;
        i = 0 as UINT;
        while i < (*pRec).NumStrings as UINT {
            DbgPrint(b"[%u] %S\n\0" as *const u8 as PCSTR, i, str);
            str = str.offset(
                (wcslen(str as *const wchar_t) as isize)
                    .wrapping_add(1 as isize),
            );
            i = i.wrapping_add(1);
        }
    }

    DbgPrint(
        b"Length2 = %lu\n\0" as *const u8 as PCSTR,
        *((pRec as ULONG_PTR)
            .wrapping_add((*pRec).Length as ULONG_PTR)
            .wrapping_sub(4 as ULONG_PTR) as PULONG),
    );
}
