// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_F3D8D400CF21: [u16; 7] = [0x0044u16, 0x0072, 0x0069, 0x0076, 0x0065, 0x0072, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut PrintMonitorList: LIST_ENTRY = _LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

static mut dwMonitorInfo1Offsets: [DWORD; 2] = [
    core::mem::offset_of!(MONITOR_INFO_1W, pName) as DWORD,
    MAXDWORD as DWORD,
];

static mut dwMonitorInfo2Offsets: [DWORD; 4] = [
    core::mem::offset_of!(MONITOR_INFO_2W, pName) as DWORD,
    core::mem::offset_of!(MONITOR_INFO_2W, pEnvironment) as DWORD,
    core::mem::offset_of!(MONITOR_INFO_2W, pDLLName) as DWORD,
    MAXDWORD as DWORD,
];

static mut MonReg: MONITORREG = unsafe {
    _MONITORREG {
        cbSize: ::core::mem::size_of::<MONITORREG>() as DWORD,
        fpCreateKey: Some(
            CreateKey,
        ),
        fpOpenKey: Some(
            OpenKey,
        ),
        fpCloseKey: Some(CloseKey),
        fpDeleteKey: Some(DeleteKey),
        fpEnumKey: Some(
            EnumKey,
        ),
        fpQueryInfoKey: Some(
            QueryInfoKey,
        ),
        fpSetValue: Some(
            SetValue,
        ),
        fpDeleteValue: Some(DeleteValue),
        fpEnumValue: Some(
            EnumValue,
        ),
        fpQueryValue: Some(
            QueryValue,
        ),
    }
};
#[no_mangle]
pub unsafe extern "C" fn FindPrintMonitor(pwszName: PCWSTR) -> PLOCAL_PRINT_MONITOR {
    let mut pEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut pPrintMonitor: PLOCAL_PRINT_MONITOR = core::ptr::null_mut();
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/printing/providers/localspl/monitors.c\0"
                as *const u8 as *const i8,
            b"FindPrintMonitor\0" as *const u8 as *const i8,
            33 as i32,
            b"FindPrintMonitor(%S)\n\0" as *const u8 as *const i8,
            pwszName,
        );
    }
    if pwszName.is_null() {
        return core::ptr::null_mut();
    }
    pEntry = (*core::ptr::addr_of_mut!(PrintMonitorList)).Flink as PLIST_ENTRY;
    while pEntry != core::ptr::addr_of_mut!(PrintMonitorList) {
        pPrintMonitor = CONTAINING_RECORD!(pEntry, LOCAL_PRINT_MONITOR, Entry);
        if _wcsicmp(
            (*pPrintMonitor).pwszName as *const wchar_t,
            pwszName as *const wchar_t,
        ) == 0
        {
            return pPrintMonitor;
        }
        pEntry = (*pEntry).Flink as PLIST_ENTRY;
    }
    return core::ptr::null_mut();
}
unsafe extern "system" fn CreateKey(
    hcKey: HANDLE,
    pszSubKey: LPCWSTR,
    dwOptions: DWORD,
    samDesired: REGSAM,
    pSecurityAttributes: PSECURITY_ATTRIBUTES,
    phckResult: PHANDLE,
    pdwDisposition: PDWORD,
    hSpooler: HANDLE,
) -> LONG {
    // FIXME("stub\n");
    ERROR_CALL_NOT_IMPLEMENTED as LONG
}
unsafe extern "system" fn OpenKey(
    hcKey: HANDLE,
    pszSubKey: LPCWSTR,
    samDesired: REGSAM,
    phkResult: PHANDLE,
    hSpooler: HANDLE,
) -> LONG {
    // FIXME("stub\n");
    ERROR_CALL_NOT_IMPLEMENTED as LONG
}
unsafe extern "system" fn CloseKey(hcKey: HANDLE, hSpooler: HANDLE) -> LONG {
    // FIXME("stub\n");
    ERROR_CALL_NOT_IMPLEMENTED as LONG
}
unsafe extern "system" fn DeleteKey(
    hcKey: HANDLE,
    pszSubKey: LPCWSTR,
    hSpooler: HANDLE,
) -> LONG {
    // FIXME("stub\n");
    ERROR_CALL_NOT_IMPLEMENTED as LONG
}
unsafe extern "system" fn EnumKey(
    hcKey: HANDLE,
    dwIndex: DWORD,
    pszName: LPWSTR,
    pcchName: PDWORD,
    pftLastWriteTime: PFILETIME,
    hSpooler: HANDLE,
) -> LONG {
    // FIXME("stub\n");
    ERROR_CALL_NOT_IMPLEMENTED as LONG
}
unsafe extern "system" fn QueryInfoKey(
    hcKey: HANDLE,
    pcSubKeys: PDWORD,
    pcbKey: PDWORD,
    pcValues: PDWORD,
    pcbValue: PDWORD,
    pcbData: PDWORD,
    pcbSecurityDescriptor: PDWORD,
    pftLastWriteTime: PFILETIME,
    hSpooler: HANDLE,
) -> LONG {
    // FIXME("stub\n");
    ERROR_CALL_NOT_IMPLEMENTED as LONG
}
unsafe extern "system" fn SetValue(
    hcKey: HANDLE,
    pszValue: LPCWSTR,
    dwType: DWORD,
    pData: *const BYTE,
    cbData: DWORD,
    hSpooler: HANDLE,
) -> LONG {
    // FIXME("stub\n");
    ERROR_CALL_NOT_IMPLEMENTED as LONG
}
unsafe extern "system" fn DeleteValue(
    hcKey: HANDLE,
    pszValue: LPCWSTR,
    hSpooler: HANDLE,
) -> LONG {
    // FIXME("stub\n");
    ERROR_CALL_NOT_IMPLEMENTED as LONG
}
unsafe extern "system" fn EnumValue(
    hcKey: HANDLE,
    dwIndex: DWORD,
    pszValue: LPWSTR,
    pcbValue: PDWORD,
    pType: PDWORD,
    pData: PBYTE,
    pcbData: PDWORD,
    hSpooler: HANDLE,
) -> LONG {
    // FIXME("stub\n");
    ERROR_CALL_NOT_IMPLEMENTED as LONG
}
unsafe extern "system" fn QueryValue(
    hcKey: HANDLE,
    pszValue: LPCWSTR,
    pType: PDWORD,
    pData: PBYTE,
    pcbData: PDWORD,
    hSpooler: HANDLE,
) -> LONG {
    // FIXME("stub\n");
    ERROR_CALL_NOT_IMPLEMENTED as LONG
}
#[no_mangle]
pub unsafe extern "C" fn InitializePrintMonitorList() -> BOOL {
    let wszMonitorsPath: [u16; 48] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072,
    0x006F, 0x006C, 0x005C, 0x0050, 0x0072, 0x0069, 0x006E, 0x0074, 0x005C, 0x004D,
    0x006F, 0x006E, 0x0069, 0x0074, 0x006F, 0x0072, 0x0073, 0x0000,
];
    let cchMonitorsPath: DWORD = (_countof!(wszMonitorsPath) as DWORD).wrapping_sub(1 as DWORD);

    let mut cchMaxSubKey: DWORD = 0;
    let mut cchPrintMonitorName: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut dwSubKeys: DWORD = 0;
    let mut i: DWORD = 0;
    let mut hinstPrintMonitor: HINSTANCE = core::ptr::null_mut();
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hSubKey: HKEY = core::ptr::null_mut();
    let mut MonitorInit: MONITORINIT = core::mem::zeroed();
    let mut pfnInitializePrintMonitor: PInitializePrintMonitor = None;
    let mut pfnInitializePrintMonitor2: PInitializePrintMonitor2 = None;
    let mut pPrintMonitor: PLOCAL_PRINT_MONITOR = core::ptr::null_mut();
    let mut pwszRegistryPath: PWSTR = core::ptr::null_mut();

    // Initialize an empty list for our Print Monitors.
    InitializeListHead(core::ptr::addr_of_mut!(PrintMonitorList));

    'quit: {
        // Open the key containing Print Monitors.
        dwErrorCode = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            core::ptr::addr_of!(wszMonitorsPath) as LPCWSTR,
            0 as DWORD,
            KEY_READ as REGSAM,
            core::ptr::addr_of_mut!(hKey),
        ) as DWORD;
        if dwErrorCode != ERROR_SUCCESS as DWORD {
            break 'quit;
        }

        // Get the number of Print Providers and maximum sub key length.
        dwErrorCode = RegQueryInfoKeyW(
            hKey,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwSubKeys),
            core::ptr::addr_of_mut!(cchMaxSubKey),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as DWORD;
        if dwErrorCode != ERROR_SUCCESS as DWORD {
            break 'quit;
        }

        // Loop through all available Print Providers.
        i = 0 as DWORD;
        while i < dwSubKeys {
            'iter: {
                // Cleanup tasks from the previous run
                if !hSubKey.is_null() {
                    RegCloseKey(hSubKey);
                    hSubKey = core::ptr::null_mut();
                }

                if !pwszRegistryPath.is_null() {
                    DllFreeSplMem(pwszRegistryPath as PVOID);
                    pwszRegistryPath = core::ptr::null_mut();
                }

                if !pPrintMonitor.is_null() {
                    if !(*pPrintMonitor).pwszFileName.is_null() {
                        DllFreeSplMem((*pPrintMonitor).pwszFileName as PVOID);
                    }

                    if !(*pPrintMonitor).pwszName.is_null() {
                        DllFreeSplMem((*pPrintMonitor).pwszName as PVOID);
                    }

                    DllFreeSplMem(pPrintMonitor as PVOID);
                    pPrintMonitor = core::ptr::null_mut();
                }

                // Create a new LOCAL_PRINT_MONITOR structure for it.
                pPrintMonitor = DllAllocSplMem(
                    core::mem::size_of::<LOCAL_PRINT_MONITOR>() as DWORD,
                ) as PLOCAL_PRINT_MONITOR;
                if pPrintMonitor.is_null() {
                    dwErrorCode = ERROR_NOT_ENOUGH_MEMORY as DWORD;
                    break 'quit;
                }

                memset(
                    pPrintMonitor as *mut core::ffi::c_void,
                    0 as core::ffi::c_int,
                    core::mem::size_of::<LOCAL_PRINT_MONITOR>() as usize,
                );

                // Allocate memory for the Print Monitor Name.
                (*pPrintMonitor).pwszName = DllAllocSplMem(
                    (cchMaxSubKey.wrapping_add(1 as DWORD))
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
                ) as PWSTR;
                if (*pPrintMonitor).pwszName.is_null() {
                    dwErrorCode = ERROR_NOT_ENOUGH_MEMORY as DWORD;
                    break 'quit;
                }

                // Get the name of this Print Monitor.
                cchPrintMonitorName = cchMaxSubKey.wrapping_add(1 as DWORD);
                dwErrorCode = RegEnumKeyExW(
                    hKey,
                    i,
                    (*pPrintMonitor).pwszName as LPWSTR,
                    core::ptr::addr_of_mut!(cchPrintMonitorName),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                ) as DWORD;
                if dwErrorCode != ERROR_SUCCESS as DWORD {
                    break 'iter;
                }

                // Open this Print Monitor's registry key.
                dwErrorCode = RegOpenKeyExW(
                    hKey,
                    (*pPrintMonitor).pwszName as LPCWSTR,
                    0 as DWORD,
                    KEY_READ as REGSAM,
                    core::ptr::addr_of_mut!(hSubKey),
                ) as DWORD;
                if dwErrorCode != ERROR_SUCCESS as DWORD {
                    break 'iter;
                }

                // Get the file name of the Print Monitor.
                (*pPrintMonitor).pwszFileName =
                    AllocAndRegQueryWSZ(hSubKey, RUSTCONV_WSTR_F3D8D400CF21.as_ptr()) as PWSTR;
                if (*pPrintMonitor).pwszFileName.is_null() {
                    break 'iter;
                }

                // Try to load it.
                hinstPrintMonitor =
                    LoadLibraryW((*pPrintMonitor).pwszFileName as LPCWSTR);
                if hinstPrintMonitor.is_null() {
                    break 'iter;
                }

                (*pPrintMonitor).hModule = hinstPrintMonitor as HMODULE;

                // Try to find a Level 2 initialization routine first.
                pfnInitializePrintMonitor2 = core::mem::transmute::<FARPROC, PInitializePrintMonitor2>(
                    GetProcAddress(
                        hinstPrintMonitor,
                        b"InitializePrintMonitor2\0".as_ptr() as LPCSTR,
                    ),
                );
                if pfnInitializePrintMonitor2.is_some() {
                    // Prepare a MONITORINIT structure.
                    MonitorInit.cbSize = core::mem::size_of::<MONITORINIT>() as DWORD;
                    MonitorInit.bLocal = TRUE as BOOL;

                    // TODO: Fill the other fields.
                    MonitorInit.hckRegistryRoot = hKey;
                    MonitorInit.pMonitorReg = core::ptr::addr_of_mut!(MonReg) as PMONITORREG;

                    // Call the Level 2 initialization routine.
                    (*pPrintMonitor).pMonitor = (pfnInitializePrintMonitor2.unwrap_unchecked())(
                        core::ptr::addr_of_mut!(MonitorInit),
                        core::ptr::addr_of_mut!((*pPrintMonitor).hMonitor),
                    ) as PVOID;
                    if (*pPrintMonitor).pMonitor.is_null() {
                        break 'iter;
                    }

                    (*pPrintMonitor).bIsLevel2 = TRUE as BOOL;
                } else {
                    // Try to find a Level 1 initialization routine then.
                    pfnInitializePrintMonitor = core::mem::transmute::<FARPROC, PInitializePrintMonitor>(
                        GetProcAddress(
                            hinstPrintMonitor,
                            b"InitializePrintMonitor\0".as_ptr() as LPCSTR,
                        ),
                    );
                    if pfnInitializePrintMonitor.is_some() {
                        // Construct the registry path.
                        pwszRegistryPath = DllAllocSplMem(
                            (cchMonitorsPath
                                .wrapping_add(1 as DWORD)
                                .wrapping_add(cchPrintMonitorName)
                                .wrapping_add(1 as DWORD))
                            .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
                        ) as PWSTR;
                        if pwszRegistryPath.is_null() {
                            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY as DWORD;
                            break 'quit;
                        }

                        CopyMemory!(
                            pwszRegistryPath as *mut core::ffi::c_void,
                            core::ptr::addr_of!(wszMonitorsPath) as *const core::ffi::c_void,
                            (cchMonitorsPath as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
                        );
                        *pwszRegistryPath.add(cchMonitorsPath as usize) = 0x005Cu16;
                        CopyMemory!(
                            pwszRegistryPath.add(
                                cchMonitorsPath.wrapping_add(1 as DWORD) as usize,
                            ) as *mut core::ffi::c_void,
                            (*pPrintMonitor).pwszName as *const core::ffi::c_void,
                            (cchPrintMonitorName.wrapping_add(1 as DWORD) as usize)
                                .wrapping_mul(core::mem::size_of::<WCHAR>()),
                        );

                        // Call the Level 1 initialization routine.
                        (*pPrintMonitor).pMonitor =
                            (pfnInitializePrintMonitor.unwrap_unchecked())(pwszRegistryPath) as PVOID;
                        if (*pPrintMonitor).pMonitor.is_null() {
                            break 'iter;
                        }
                    } else {
                        break 'iter;
                    }
                }

                // Add this Print Monitor to the list.
                InsertTailList(
                    core::ptr::addr_of_mut!(PrintMonitorList),
                    core::ptr::addr_of_mut!((*pPrintMonitor).Entry),
                );
                (*pPrintMonitor).refcount = (*pPrintMonitor).refcount.wrapping_add(1);

                // Don't let the cleanup routine free this.
                pPrintMonitor = core::ptr::null_mut();
            }
            i = i.wrapping_add(1);
        }

        dwErrorCode = ERROR_SUCCESS as DWORD;
    }

    // Cleanup:
    // Inside the loop
    if !hSubKey.is_null() {
        RegCloseKey(hSubKey);
    }

    if !pwszRegistryPath.is_null() {
        DllFreeSplMem(pwszRegistryPath as PVOID);
    }

    if !pPrintMonitor.is_null() {
        if !(*pPrintMonitor).pwszFileName.is_null() {
            DllFreeSplMem((*pPrintMonitor).pwszFileName as PVOID);
        }

        if !(*pPrintMonitor).pwszName.is_null() {
            DllFreeSplMem((*pPrintMonitor).pwszName as PVOID);
        }

        DllFreeSplMem(pPrintMonitor as PVOID);
    }

    // Outside the loop
    if !hKey.is_null() {
        RegCloseKey(hKey);
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS as DWORD) as BOOL;
}

unsafe extern "C" fn _LocalGetMonitorLevel1(
    mut pPrintMonitor: PLOCAL_PRINT_MONITOR,
    mut ppMonitorInfo: *mut PMONITOR_INFO_1W,
    mut ppMonitorInfoEnd: *mut PBYTE,
    mut pcbNeeded: PDWORD,
) {
    let mut cbMonitorName: DWORD = 0;
    let mut pwszStrings: [PCWSTR; 1] = [core::ptr::null(); 1];

    // Calculate the string lengths.
    if ppMonitorInfo.is_null() {
        cbMonitorName = ((wcslen((*pPrintMonitor).pwszName as *const wchar_t) as usize)
            .wrapping_add(1 as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as usize)) as DWORD;

        *pcbNeeded = (*pcbNeeded).wrapping_add(
            (core::mem::size_of::<MONITOR_INFO_1W>() as DWORD).wrapping_add(cbMonitorName),
        );
        return;
    }

    // Set the pName field.
    pwszStrings[0 as usize] = (*pPrintMonitor).pwszName as PCWSTR;

    // Copy the structure and advance to the next one in the output buffer.
    *ppMonitorInfoEnd = PackStrings(
        core::ptr::addr_of_mut!(pwszStrings) as *mut PCWSTR,
        *ppMonitorInfo as PBYTE,
        core::ptr::addr_of_mut!(dwMonitorInfo1Offsets) as *mut DWORD,
        *ppMonitorInfoEnd,
    );
    *ppMonitorInfo = (*ppMonitorInfo).offset(1);
}

unsafe extern "C" fn _LocalGetMonitorLevel2(
    pPrintMonitor: PLOCAL_PRINT_MONITOR,
    ppMonitorInfo: *mut PMONITOR_INFO_2W,
    ppMonitorInfoEnd: *mut PBYTE,
    pcbNeeded: PDWORD,
) {
    let mut cbFileName: DWORD = 0;
    let mut cbMonitorName: DWORD = 0;
    let mut pwszStrings: [PCWSTR; 3] = [core::ptr::null::<WCHAR>(); 3];

    // Calculate the string lengths.
    if ppMonitorInfo.is_null() {
        cbMonitorName = ((wcslen((*pPrintMonitor).pwszName as *const WCHAR) as DWORD).wrapping_add(1))
            .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
        cbFileName = ((wcslen((*pPrintMonitor).pwszFileName as *const WCHAR) as DWORD).wrapping_add(1))
            .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);

        *pcbNeeded = (*pcbNeeded).wrapping_add(
            (core::mem::size_of::<MONITOR_INFO_2W>() as DWORD)
                .wrapping_add(cbMonitorName)
                .wrapping_add(cbCurrentEnvironment)
                .wrapping_add(cbFileName),
        );
        return;
    }

    // Set the pName field.
    pwszStrings[0] = (*pPrintMonitor).pwszName as PCWSTR;

    // Set the pEnvironment field.
    pwszStrings[1] =
        core::ptr::addr_of!(wszCurrentEnvironment) as *const WCHAR as PCWSTR;

    // Set the pDLLName field.
    pwszStrings[2] = (*pPrintMonitor).pwszFileName as PCWSTR;

    // Copy the structure and advance to the next one in the output buffer.
    *ppMonitorInfoEnd = PackStrings(
        core::ptr::addr_of_mut!(pwszStrings) as *mut PCWSTR,
        *ppMonitorInfo as PBYTE,
        core::ptr::addr_of_mut!(dwMonitorInfo2Offsets) as *mut DWORD,
        *ppMonitorInfoEnd,
    );
    *ppMonitorInfo = (*ppMonitorInfo).add(1);
}

#[no_mangle]
pub unsafe extern "system" fn LocalEnumMonitors(pName: PWSTR, Level: DWORD, mut pMonitors: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD, pcReturned: PDWORD) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let mut pMonitorInfoEnd: PBYTE = core::ptr::null_mut();
    let mut pEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut pPrintMonitor: PLOCAL_PRINT_MONITOR = core::ptr::null_mut();

    'Cleanup: {
        // Sanity checks.
        if Level > 2 {
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'Cleanup;
        }

        // Begin counting.
        *pcbNeeded = 0;
        *pcReturned = 0;

        // Count the required buffer size and the number of monitors.
        pEntry = (*core::ptr::addr_of_mut!(PrintMonitorList)).Flink;
        while pEntry != core::ptr::addr_of_mut!(PrintMonitorList) {
            pPrintMonitor = CONTAINING_RECORD!(pEntry, LOCAL_PRINT_MONITOR, Entry) as PLOCAL_PRINT_MONITOR;

            if Level == 1 {
                _LocalGetMonitorLevel1(pPrintMonitor, core::ptr::null_mut(), core::ptr::null_mut(), pcbNeeded);
            } else if Level == 2 {
                _LocalGetMonitorLevel2(pPrintMonitor, core::ptr::null_mut(), core::ptr::null_mut(), pcbNeeded);
            }

            pEntry = (*pEntry).Flink;
        }

        // Check if the supplied buffer is large enough.
        if cbBuf < *pcbNeeded {
            dwErrorCode = ERROR_INSUFFICIENT_BUFFER;
            break 'Cleanup;
        }

        // Copy over the Monitor information.
        pMonitorInfoEnd = pMonitors.add(*pcbNeeded as usize);

        pEntry = (*core::ptr::addr_of_mut!(PrintMonitorList)).Flink;
        while pEntry != core::ptr::addr_of_mut!(PrintMonitorList) {
            pPrintMonitor = CONTAINING_RECORD!(pEntry, LOCAL_PRINT_MONITOR, Entry) as PLOCAL_PRINT_MONITOR;

            if Level == 1 {
                _LocalGetMonitorLevel1(
                    pPrintMonitor,
                    core::ptr::addr_of_mut!(pMonitors) as *mut PMONITOR_INFO_1W,
                    core::ptr::addr_of_mut!(pMonitorInfoEnd),
                    core::ptr::null_mut(),
                );
            } else if Level == 2 {
                _LocalGetMonitorLevel2(
                    pPrintMonitor,
                    core::ptr::addr_of_mut!(pMonitors) as *mut PMONITOR_INFO_2W,
                    core::ptr::addr_of_mut!(pMonitorInfoEnd),
                    core::ptr::null_mut(),
                );
            }

            *pcReturned = (*pcReturned).wrapping_add(1);
            pEntry = (*pEntry).Flink;
        }

        dwErrorCode = ERROR_SUCCESS;
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn AddPrintMonitorList(pName: LPCWSTR, mut DllName: LPWSTR) -> BOOL {
    let wszMonitorsPath: [u16; 49] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072,
    0x006F, 0x006C, 0x005C, 0x0050, 0x0072, 0x0069, 0x006E, 0x0074, 0x005C, 0x004D,
    0x006F, 0x006E, 0x0069, 0x0074, 0x006F, 0x0072, 0x0073, 0x005C, 0x0000,
];
    let cchMonitorsPath: DWORD = (_countof!(wszMonitorsPath) - 1) as DWORD;

    let mut wszRegRoot: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    let mut cchPrintMonitorName: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut hinstPrintMonitor: HINSTANCE = core::ptr::null_mut();
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut MonitorInit: MONITORINIT = core::mem::zeroed();
    let mut pfnInitializePrintMonitor: PInitializePrintMonitor = None;
    let mut pfnInitializePrintMonitor2: PInitializePrintMonitor2 = None;
    let mut pPrintMonitor: PLOCAL_PRINT_MONITOR = core::ptr::null_mut();
    let mut pwszRegistryPath: PWSTR = core::ptr::null_mut();

    StringCbCopyW(
        wszRegRoot.as_mut_ptr(),
        core::mem::size_of::<[u16; MAX_PATH as usize]>(),
        wszMonitorsPath.as_ptr(),
    );
    StringCbCatW(
        wszRegRoot.as_mut_ptr(),
        core::mem::size_of::<[u16; MAX_PATH as usize]>(),
        pName,
    );

    // Open the key containing Print Monitors.
    dwErrorCode = RegOpenKeyW(
        HKEY_LOCAL_MACHINE,
        wszRegRoot.as_mut_ptr(),
        core::ptr::addr_of_mut!(hKey),
    ) as DWORD;
    if dwErrorCode != ERROR_SUCCESS {
        // goto Cleanup
    } else {
        'Cleanup: {
            if dwErrorCode != ERROR_SUCCESS {
                break 'Cleanup;
            }

            // Create a new LOCAL_PRINT_MONITOR structure for it.
            pPrintMonitor = DllAllocSplMem(
                core::mem::size_of::<LOCAL_PRINT_MONITOR>() as DWORD,
            ) as PLOCAL_PRINT_MONITOR;
            if pPrintMonitor.is_null() {
                dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                break 'Cleanup;
            }

            memset(
                pPrintMonitor as *mut core::ffi::c_void,
                0,
                core::mem::size_of::<LOCAL_PRINT_MONITOR>(),
            );

            // Allocate memory for the Print Monitor Name.
            (*pPrintMonitor).pwszName = AllocSplStr(pName) as PWSTR;
            if (*pPrintMonitor).pwszName.is_null() {
                dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                break 'Cleanup;
            }

            cchPrintMonitorName =
                wcslen((*pPrintMonitor).pwszName as *const u16) as DWORD;

            if DllName.is_null() {
                let mut namesize: DWORD = 0;

                dwErrorCode = RegQueryValueExW(
                    hKey,
                    RUSTCONV_WSTR_F3D8D400CF21.as_ptr(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(namesize),
                ) as DWORD;

                if dwErrorCode == ERROR_SUCCESS {
                    DllName = DllAllocSplMem(namesize) as LPWSTR;

                    RegQueryValueExW(
                        hKey,
                        RUSTCONV_WSTR_F3D8D400CF21.as_ptr(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        DllName as LPBYTE,
                        core::ptr::addr_of_mut!(namesize),
                    );

                    (*pPrintMonitor).pwszFileName = DllName;
                } else {
                    break 'Cleanup;
                }
            } else {
                (*pPrintMonitor).pwszFileName =
                    AllocSplStr(DllName) as PWSTR;
            }

            // Try to load it.
            hinstPrintMonitor = LoadLibraryW((*pPrintMonitor).pwszFileName);
            if hinstPrintMonitor.is_null() {
                dwErrorCode = GetLastError();
                break 'Cleanup;
            }

            (*pPrintMonitor).hModule = hinstPrintMonitor as HMODULE;

            // Try to find a Level 2 initialization routine first.
            pfnInitializePrintMonitor2 = core::mem::transmute::<FARPROC, PInitializePrintMonitor2>(
                GetProcAddress(
                    hinstPrintMonitor,
                    b"InitializePrintMonitor2\0".as_ptr() as *const i8,
                ),
            );
            if pfnInitializePrintMonitor2.is_some() {
                // Prepare a MONITORINIT structure.
                MonitorInit.cbSize = core::mem::size_of::<MONITORINIT>() as DWORD;
                MonitorInit.bLocal = TRUE as BOOL;

                // TODO: Fill the other fields.
                MonitorInit.hckRegistryRoot = hKey;
                MonitorInit.pMonitorReg = core::ptr::addr_of_mut!(MonReg);

                // Call the Level 2 initialization routine.
                (*pPrintMonitor).pMonitor = (pfnInitializePrintMonitor2.unwrap_unchecked())(
                    core::ptr::addr_of_mut!(MonitorInit),
                    core::ptr::addr_of_mut!((*pPrintMonitor).hMonitor),
                ) as PVOID;
                if (*pPrintMonitor).pMonitor.is_null() {
                    break 'Cleanup;
                }

                (*pPrintMonitor).bIsLevel2 = TRUE as BOOL;
            } else {
                // Try to find a Level 1 initialization routine then.
                pfnInitializePrintMonitor = core::mem::transmute::<FARPROC, PInitializePrintMonitor>(
                    GetProcAddress(
                        hinstPrintMonitor,
                        b"InitializePrintMonitor\0".as_ptr() as *const i8,
                    ),
                );
                if pfnInitializePrintMonitor.is_some() {
                    // Construct the registry path.
                    pwszRegistryPath = DllAllocSplMem(
                        ((cchMonitorsPath
                            + 1
                            + cchPrintMonitorName
                            + 1) as usize
                            * core::mem::size_of::<u16>()) as DWORD,
                    ) as PWSTR;
                    if pwszRegistryPath.is_null() {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'Cleanup;
                    }

                    CopyMemory!(
                        pwszRegistryPath as *mut core::ffi::c_void,
                        wszMonitorsPath.as_ptr() as *const core::ffi::c_void,
                        cchMonitorsPath as usize * core::mem::size_of::<u16>(),
                    );
                    *pwszRegistryPath.offset(cchMonitorsPath as isize) = 0x005Cu16;
                    CopyMemory!(
                        pwszRegistryPath
                            .offset((cchMonitorsPath + 1) as isize)
                            as *mut core::ffi::c_void,
                        (*pPrintMonitor).pwszName as *const core::ffi::c_void,
                        (cchPrintMonitorName + 1) as usize
                            * core::mem::size_of::<u16>(),
                    );

                    // Call the Level 1 initialization routine.
                    (*pPrintMonitor).pMonitor =
                        (pfnInitializePrintMonitor.unwrap_unchecked())(pwszRegistryPath) as PVOID;
                    if (*pPrintMonitor).pMonitor.is_null() {
                        break 'Cleanup;
                    }
                } else {
                    dwErrorCode = ERROR_PROC_NOT_FOUND;
                    break 'Cleanup;
                }
            }

            // Add this Print Monitor to the list.
            InsertTailList(
                core::ptr::addr_of_mut!(PrintMonitorList),
                core::ptr::addr_of_mut!((*pPrintMonitor).Entry),
            );

            (*pPrintMonitor).refcount = (*pPrintMonitor).refcount.wrapping_add(1);

            // Don't let the cleanup routine free this.
            pPrintMonitor = core::ptr::null_mut();

            dwErrorCode = ERROR_SUCCESS;
        }
    }

    // Cleanup:
    if !pwszRegistryPath.is_null() {
        DllFreeSplMem(pwszRegistryPath as PVOID);
    }

    if !pPrintMonitor.is_null() {
        if !(*pPrintMonitor).pwszFileName.is_null() {
            DllFreeSplMem((*pPrintMonitor).pwszFileName as PVOID);
        }

        if !(*pPrintMonitor).pwszName.is_null() {
            DllFreeSplMem((*pPrintMonitor).pwszName as PVOID);
        }

        DllFreeSplMem(pPrintMonitor as PVOID);
    }

    // Outside the loop
    if !hKey.is_null() {
        RegCloseKey(hKey);
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn LocalAddMonitor(pName: PWSTR, Level: DWORD, pMonitors: PBYTE) -> BOOL {
    let mut env: PPRINTENV_T = core::ptr::null_mut();
    let mut mi2w: LPMONITOR_INFO_2W = core::ptr::null_mut();
    let mut hroot: HKEY = core::ptr::null_mut();
    let mut hentry: HKEY = core::ptr::null_mut();
    let mut disposition: DWORD = 0;
    let mut res: BOOL = FALSE as BOOL;
    let wszMonitorsPath: [u16; 49] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072,
    0x006F, 0x006C, 0x005C, 0x0050, 0x0072, 0x0069, 0x006E, 0x0074, 0x005C, 0x004D,
    0x006F, 0x006E, 0x0069, 0x0074, 0x006F, 0x0072, 0x0073, 0x005C, 0x0000,
];

    mi2w = pMonitors as LPMONITOR_INFO_2W;

    if copy_servername_from_name(pName, core::ptr::null_mut()) != 0 {
        SetLastError(ERROR_ACCESS_DENIED);
        return FALSE as BOOL;
    }

    if (*mi2w).pName.is_null() || *(*mi2w).pName.offset(0 as isize) == 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    env = validate_envW((*mi2w).pEnvironment);
    if env.is_null() {
        return FALSE as BOOL;
    }

    if (*mi2w).pDLLName.is_null() || *(*mi2w).pDLLName.offset(0 as isize) == 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    if RegCreateKeyW(
        HKEY_LOCAL_MACHINE,
        core::ptr::addr_of!(wszMonitorsPath) as LPCWSTR,
        core::ptr::addr_of_mut!(hroot),
    ) != ERROR_SUCCESS as LONG
    {
        return FALSE as BOOL;
    }

    if RegCreateKeyExW(
        hroot,
        (*mi2w).pName as LPCWSTR,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_WRITE | KEY_QUERY_VALUE,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hentry),
        core::ptr::addr_of_mut!(disposition),
    ) == ERROR_SUCCESS as LONG
    {
        let mut namesize: DWORD = 0;

        if (disposition == REG_OPENED_EXISTING_KEY)
            && (RegQueryValueExW(
                hentry,
                RUSTCONV_WSTR_F3D8D400CF21.as_ptr(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(namesize),
            ) == ERROR_SUCCESS as LONG)
        {
            SetLastError(ERROR_PRINT_MONITOR_ALREADY_INSTALLED);
        } else {
            let len: INT = ((lstrlenW((*mi2w).pDLLName as LPCWSTR) + 1)
                as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>()) as INT;

            res = (RegSetValueExW(
                hentry,
                RUSTCONV_WSTR_F3D8D400CF21.as_ptr(),
                0,
                REG_SZ,
                (*mi2w).pDLLName as LPBYTE,
                len as DWORD,
            ) == ERROR_SUCCESS as LONG) as BOOL;

            res = AddPrintMonitorList((*mi2w).pName as LPCWSTR, (*mi2w).pDLLName);

            if res == 0 {
                RegDeleteKeyW(hroot, (*mi2w).pName as LPCWSTR);
            } else {
                SetLastError(ERROR_SUCCESS);
            }
        }

        RegCloseKey(hentry);
    }

    RegCloseKey(hroot);
    return res;
}
#[no_mangle]
pub unsafe extern "system" fn LocalDeleteMonitor(pName: PWSTR, pEnvironment: PWSTR, pMonitorName: PWSTR) -> BOOL {
    let mut hroot: HKEY = core::ptr::null_mut();
    let mut lres: LONG;
    let mut pPrintMonitor: PLOCAL_PRINT_MONITOR;
    let wszMonitorsPath: [WCHAR; 49] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072,
    0x006F, 0x006C, 0x005C, 0x0050, 0x0072, 0x0069, 0x006E, 0x0074, 0x005C, 0x004D,
    0x006F, 0x006E, 0x0069, 0x0074, 0x006F, 0x0072, 0x0073, 0x005C, 0x0000,
];

    lres = copy_servername_from_name(pName, core::ptr::null_mut());
    if lres != 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"win32ss/printing/providers/localspl/monitors.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"LocalDeleteMonitor\0" as *const u8 as *const ::core::ffi::c_char,
                737 as ::core::ffi::c_int,
                b"server %s not supported\n\0" as *const u8 as *const ::core::ffi::c_char,
                debugstr_w(pName as *const WCHAR),
            );
        }
        SetLastError(ERROR_INVALID_NAME);
        return FALSE as BOOL;
    }

    /*  pEnvironment is ignored in Windows for the local Computer */
    if pMonitorName.is_null() || *pMonitorName.offset(0 as isize) == 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_1,
                __dbch_1,
                b"win32ss/printing/providers/localspl/monitors.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"LocalDeleteMonitor\0" as *const u8 as *const ::core::ffi::c_char,
                745 as ::core::ffi::c_int,
                b"pMonitorName %s is invalid\n\0" as *const u8 as *const ::core::ffi::c_char,
                debugstr_w(pMonitorName as *const WCHAR),
            );
        }
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    pPrintMonitor = FindPrintMonitor(pMonitorName);
    if !pPrintMonitor.is_null() {
        if (*pPrintMonitor).refcount != 0 {
            (*pPrintMonitor).refcount = (*pPrintMonitor).refcount.wrapping_sub(1);
        }

        if (*pPrintMonitor).refcount == 0 {
            /* Unload the monitor if it's loaded */
            RemoveEntryList(core::ptr::addr_of_mut!((*pPrintMonitor).Entry));

            if (*pPrintMonitor).bIsLevel2 != 0 {
                let mut pm2: PMONITOR2 = (*pPrintMonitor).pMonitor as PMONITOR2;
                if !pm2.is_null() && (*pm2).pfnShutdown.is_some() {
                    (*pm2).pfnShutdown.unwrap_unchecked()(
                        (*pPrintMonitor).hMonitor,
                    );
                }
            }

            if !(*pPrintMonitor).hModule.is_null() {
                FreeLibrary((*pPrintMonitor).hModule);
            }

            if !(*pPrintMonitor).pwszFileName.is_null() {
                DllFreeSplStr((*pPrintMonitor).pwszFileName);
            }

            if !(*pPrintMonitor).pwszName.is_null() {
                DllFreeSplStr((*pPrintMonitor).pwszName);
            }

            DllFreeSplMem(pPrintMonitor as PVOID);
            pPrintMonitor = core::ptr::null_mut();
        }
    } else {
        /* nothing */
    }

    if RegCreateKeyW(
        HKEY_LOCAL_MACHINE,
        &raw const wszMonitorsPath as LPCWSTR,
        &raw mut hroot,
    ) != ERROR_SUCCESS as LONG
    {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_3: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_3,
                __dbch_3,
                b"win32ss/printing/providers/localspl/monitors.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"LocalDeleteMonitor\0" as *const u8 as *const ::core::ffi::c_char,
                788 as ::core::ffi::c_int,
                b"unable to create key %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                debugstr_w(&raw const wszMonitorsPath as *const WCHAR),
            );
        }
        return FALSE as BOOL;
    }

    if RegDeleteTreeW(hroot, pMonitorName) == ERROR_SUCCESS as LONG {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
            != 0
        {
            let __dbch_4: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_4: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                __dbcl_4,
                __dbch_4,
                b"win32ss/printing/providers/localspl/monitors.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"LocalDeleteMonitor\0" as *const u8 as *const ::core::ffi::c_char,
                794 as ::core::ffi::c_int,
                b"%s deleted\n\0" as *const u8 as *const ::core::ffi::c_char,
                debugstr_w(pMonitorName as *const WCHAR),
            );
        }
        RegCloseKey(hroot);
        return TRUE as BOOL;
    }

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch_5: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_5: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl_5,
            __dbch_5,
            b"win32ss/printing/providers/localspl/monitors.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LocalDeleteMonitor\0" as *const u8 as *const ::core::ffi::c_char,
            799 as ::core::ffi::c_int,
            b"%s does not exist\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_w(pMonitorName as *const WCHAR),
        );
    }

    RegCloseKey(hroot);

    /* NT: ERROR_UNKNOWN_PRINT_MONITOR (3000), 9x: ERROR_INVALID_PARAMETER (87) */
    SetLastError(ERROR_UNKNOWN_PRINT_MONITOR);
    return FALSE as BOOL;
}
