use crate::localspl::monitors::FindPrintMonitor;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut _PortList: LIST_ENTRY = _LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};
#[no_mangle]
pub unsafe extern "C" fn FindPort(pwszName: PCWSTR) -> PLOCAL_PORT {
    let mut pEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut pPort: PLOCAL_PORT = core::ptr::null_mut();
    if ((*__wine_dbch___default).flags as core::ffi::c_int
        & ((1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/printing/providers/localspl/ports.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"FindPort\0" as *const u8 as *const core::ffi::c_char,
            20 as core::ffi::c_int,
            b"FindPort(%S)\n\0" as *const u8 as *const core::ffi::c_char,
            pwszName,
        );
    }
    if pwszName.is_null() {
        return core::ptr::null_mut();
    }
    pEntry = (*core::ptr::addr_of_mut!(_PortList)).Flink;
    while pEntry != core::ptr::addr_of_mut!(_PortList) {
        pPort = CONTAINING_RECORD!(pEntry, LOCAL_PORT, Entry);
        if _wcsicmp((*pPort).pwszName, pwszName) == 0 {
            return pPort;
        }
        pEntry = (*pEntry).Flink;
    }
    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn CreatePortEntry(mut pwszName: PCWSTR, mut pPrintMonitor: PLOCAL_PRINT_MONITOR) -> BOOL {
    let mut pPort: PLOCAL_PORT = core::ptr::null_mut();
    let mut cbPortName: DWORD = ((wcslen(pwszName as *const wchar_t) as DWORD)
        + 1)
        * (core::mem::size_of::<WCHAR>() as DWORD);

    pPort = DllAllocSplMem(
        (core::mem::size_of::<LOCAL_PORT>() as DWORD) + cbPortName,
    ) as PLOCAL_PORT;
    if pPort.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return FALSE as BOOL;
    }

    (*pPort).pPrintMonitor = pPrintMonitor;
    (*pPort).pwszName = wcscpy(
        pPort.offset(1 as isize) as PWSTR,
        pwszName,
    ) as PWSTR;

    InsertTailList(
        core::ptr::addr_of_mut!(_PortList),
        core::ptr::addr_of_mut!((*pPort).Entry),
    );

    return TRUE as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn InitializePortList() -> BOOL {
    let mut bReturnValue: BOOL = 0;
    let mut cbNeeded: DWORD = 0;
    let mut cbPortName: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut dwReturned: DWORD = 0;
    let mut i: DWORD = 0;
    let mut pPort: PLOCAL_PORT = core::ptr::null_mut();
    let mut pPrintMonitor: PLOCAL_PRINT_MONITOR = core::ptr::null_mut();
    let mut pEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut p: PPORT_INFO_1W = core::ptr::null_mut();
    let mut pPortInfo1: PPORT_INFO_1W = core::ptr::null_mut();

    'cleanup: {
        // Initialize an empty list for our Ports.
        InitializeListHead(core::ptr::addr_of_mut!(_PortList));

        // Loop through all Print Monitors.
        pEntry = PrintMonitorList.Flink;
        while pEntry != core::ptr::addr_of_mut!(PrintMonitorList) {
            'next: {
                // Cleanup from the previous run.
                if !pPortInfo1.is_null() {
                    DllFreeSplMem(pPortInfo1 as PVOID);
                    pPortInfo1 = core::ptr::null_mut();
                }

                pPrintMonitor = CONTAINING_RECORD!(pEntry, LOCAL_PRINT_MONITOR, Entry);

                // Determine the required buffer size for EnumPorts.
                if (*pPrintMonitor).bIsLevel2 != 0 {
                    bReturnValue = ((*((*pPrintMonitor).pMonitor as PMONITOR2)).pfnEnumPorts.unwrap_unchecked())(
                        (*pPrintMonitor).hMonitor,
                        core::ptr::null_mut(),
                        1,
                        core::ptr::null_mut(),
                        0,
                        &mut cbNeeded,
                        &mut dwReturned,
                    );
                } else {
                    bReturnValue = ((*((*pPrintMonitor).pMonitor as LPMONITOREX)).Monitor.pfnEnumPorts.unwrap_unchecked())(
                        core::ptr::null_mut(),
                        1,
                        core::ptr::null_mut(),
                        0,
                        &mut cbNeeded,
                        &mut dwReturned,
                    );
                }

                // Check the returned error code.
                if GetLastError() != ERROR_INSUFFICIENT_BUFFER {
                    break 'next;
                }

                // Allocate a buffer large enough.
                pPortInfo1 = DllAllocSplMem(cbNeeded) as PPORT_INFO_1W;
                if pPortInfo1.is_null() {
                    dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                    break 'cleanup;
                }

                // Get the ports handled by this monitor.
                if (*pPrintMonitor).bIsLevel2 != 0 {
                    bReturnValue = ((*((*pPrintMonitor).pMonitor as PMONITOR2)).pfnEnumPorts.unwrap_unchecked())(
                        (*pPrintMonitor).hMonitor,
                        core::ptr::null_mut(),
                        1,
                        pPortInfo1 as PBYTE,
                        cbNeeded,
                        &mut cbNeeded,
                        &mut dwReturned,
                    );
                } else {
                    bReturnValue = ((*((*pPrintMonitor).pMonitor as LPMONITOREX)).Monitor.pfnEnumPorts.unwrap_unchecked())(
                        core::ptr::null_mut(),
                        1,
                        pPortInfo1 as PBYTE,
                        cbNeeded,
                        &mut cbNeeded,
                        &mut dwReturned,
                    );
                }

                // Check the return value.
                if bReturnValue == 0 {
                    break 'next;
                }

                // Loop through all returned ports.
                p = pPortInfo1;

                i = 0;
                while i < dwReturned {
                    cbPortName = ((wcslen((*p).pName as *const WCHAR) as DWORD)
                        + 1)
                        * core::mem::size_of::<WCHAR>() as DWORD;

                    // Create a new LOCAL_PORT structure for it.
                    pPort = DllAllocSplMem(
                        core::mem::size_of::<LOCAL_PORT>() as DWORD + cbPortName,
                    ) as PLOCAL_PORT;
                    if pPort.is_null() {
                        dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }

                    (*pPort).pPrintMonitor = pPrintMonitor;
                    (*pPort).pwszName = (pPort as PBYTE)
                        .add(core::mem::size_of::<LOCAL_PORT>())
                        as PWSTR;
                    CopyMemory!(
                        (*pPort).pwszName as PVOID,
                        (*p).pName as *const core::ffi::c_void,
                        cbPortName as usize,
                    );

                    // Insert it into the list and advance to the next port.
                    InsertTailList(
                        core::ptr::addr_of_mut!(_PortList),
                        core::ptr::addr_of_mut!((*pPort).Entry),
                    );
                    p = p.add(1);
                    i += 1;
                }
            }
            pEntry = (*pEntry).Flink;
        }

        dwErrorCode = ERROR_SUCCESS;
    }

    // Cleanup:
    // Inside the loop
    if !pPortInfo1.is_null() {
        DllFreeSplMem(pPortInfo1 as PVOID);
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn LocalEnumPorts(pName: PWSTR, Level: DWORD, pPorts: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD, pcReturned: PDWORD) -> BOOL {
    let mut bReturnValue: BOOL = TRUE as BOOL;
    let mut cbCallBuffer: DWORD = 0;
    let mut cbNeeded: DWORD = 0;
    let mut dwReturned: DWORD = 0;
    let mut pCallBuffer: PBYTE = core::ptr::null_mut();
    let mut pPrintMonitor: PLOCAL_PRINT_MONITOR = core::ptr::null_mut();
    let mut pEntry: PLIST_ENTRY = core::ptr::null_mut();

    // Begin counting.
    *pcbNeeded = 0;
    *pcReturned = 0;

    // At the beginning, we have the full buffer available.
    cbCallBuffer = cbBuf;
    pCallBuffer = pPorts;

    // Loop through all Print Monitors.
    pEntry = PrintMonitorList.Flink as PLIST_ENTRY;
    while pEntry != core::ptr::addr_of_mut!(PrintMonitorList) as PLIST_ENTRY {
        pPrintMonitor = (pEntry as ULONG_PTR).wrapping_sub(
            core::ptr::addr_of_mut!(
                (*core::ptr::null_mut::<LOCAL_PRINT_MONITOR>()).Entry
            ) as ULONG_PTR,
        ) as *mut LOCAL_PRINT_MONITOR as PLOCAL_PRINT_MONITOR;

        // Call the EnumPorts function of this Print Monitor.
        cbNeeded = 0;
        dwReturned = 0;

        if (*pPrintMonitor).bIsLevel2 != 0 {
            bReturnValue = (*((*pPrintMonitor).pMonitor as PMONITOR2))
                .pfnEnumPorts
                .unwrap_unchecked()(
                (*pPrintMonitor).hMonitor,
                pName as LPWSTR,
                Level,
                pCallBuffer as LPBYTE,
                cbCallBuffer,
                core::ptr::addr_of_mut!(cbNeeded),
                core::ptr::addr_of_mut!(dwReturned),
            );
        } else {
            bReturnValue = (*((*pPrintMonitor).pMonitor as LPMONITOREX))
                .Monitor
                .pfnEnumPorts
                .unwrap_unchecked()(
                pName as LPWSTR,
                Level,
                pCallBuffer as LPBYTE,
                cbCallBuffer,
                core::ptr::addr_of_mut!(cbNeeded),
                core::ptr::addr_of_mut!(dwReturned),
            );
        }

        // Add the returned counts to the total values.
        *pcbNeeded = (*pcbNeeded).wrapping_add(cbNeeded);
        *pcReturned = (*pcReturned).wrapping_add(dwReturned);

        // Reduce the available buffer size for the next call without risking an underflow.
        if cbNeeded < cbCallBuffer {
            cbCallBuffer = cbCallBuffer.wrapping_sub(cbNeeded);
        } else {
            cbCallBuffer = 0;
        }

        // Advance the buffer if the caller provided it.
        if !pCallBuffer.is_null() {
            pCallBuffer = pCallBuffer.offset(cbNeeded as isize);
        }

        pEntry = (*pEntry).Flink as PLIST_ENTRY;
    }

    return bReturnValue;
}
#[no_mangle]
pub unsafe extern "system" fn LocalAddPortEx(pName: PWSTR, Level: DWORD, lpBuffer: PBYTE, lpMonitorName: PWSTR) -> BOOL {
    let mut lres: DWORD;
    let mut res: BOOL = FALSE as BOOL;
    let mut pPort: PLOCAL_PORT;
    let mut pPrintMonitor: PLOCAL_PRINT_MONITOR;
    let mut pi: *mut PORT_INFO_1W = lpBuffer as *mut PORT_INFO_1W;

    lres = copy_servername_from_name(pName as LPCWSTR, core::ptr::null_mut::<WCHAR>()) as DWORD;
    if lres != 0 {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }

    if Level != (1 as DWORD) {
        SetLastError(ERROR_INVALID_LEVEL as DWORD);
        return FALSE as BOOL;
    }

    if pi.is_null()
        || lpMonitorName.is_null()
        || *lpMonitorName.offset((0) as isize) == 0
    {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }

    pPrintMonitor = FindPrintMonitor(lpMonitorName as PCWSTR);
    if pPrintMonitor.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }

    pPort = FindPort((*pi).pName as PCWSTR);
    if !pPort.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }

    if (*pPrintMonitor).bIsLevel2 != 0
        && (*((*pPrintMonitor).pMonitor as PMONITOR2))
            .pfnAddPortEx
            .is_some()
    {
        res = (*((*pPrintMonitor).pMonitor as PMONITOR2))
            .pfnAddPortEx
            .unwrap_unchecked()(
            (*pPrintMonitor).hMonitor,
            pName as LPWSTR,
            Level,
            lpBuffer as LPBYTE,
            lpMonitorName as LPWSTR,
        );
    } else if (*pPrintMonitor).bIsLevel2 == 0
        && (*((*pPrintMonitor).pMonitor as LPMONITOREX))
            .Monitor
            .pfnAddPortEx
            .is_some()
    {
        res = (*((*pPrintMonitor).pMonitor as LPMONITOREX))
            .Monitor
            .pfnAddPortEx
            .unwrap_unchecked()(
            pName as LPWSTR,
            Level,
            lpBuffer as LPBYTE,
            lpMonitorName as LPWSTR,
        );
    } else {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    }

    if res != 0 {
        res = CreatePortEntry((*pi).pName as PCWSTR, pPrintMonitor);
    }

    return res;
}
#[no_mangle]
pub unsafe extern "system" fn LocalAddPort(mut pName: LPWSTR, mut hWnd: HWND, mut pMonitorName: LPWSTR) -> BOOL {
    let mut lres: DWORD = 0;
    let mut res: BOOL = FALSE as BOOL;
    let mut pPrintMonitor: PLOCAL_PRINT_MONITOR = ::core::ptr::null_mut::<_LOCAL_PRINT_MONITOR>();

    lres = copy_servername_from_name(pName as LPCWSTR, ::core::ptr::null_mut::<WCHAR>()) as DWORD;
    if lres != 0 {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }

    /* an empty Monitorname is Invalid */
    if *pMonitorName.offset(0 as isize) == 0 {
        SetLastError(ERROR_NOT_SUPPORTED as DWORD);
        return FALSE as BOOL;
    }

    pPrintMonitor = FindPrintMonitor(pMonitorName as PCWSTR);
    if pPrintMonitor.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }

    if (*pPrintMonitor).bIsLevel2 != 0
        && (*((*pPrintMonitor).pMonitor as PMONITOR2)).pfnAddPort.is_some()
    {
        res = (*((*pPrintMonitor).pMonitor as PMONITOR2))
            .pfnAddPort
            .unwrap_unchecked()(
            (*pPrintMonitor).hMonitor,
            pName,
            hWnd,
            pMonitorName,
        );
    } else if (*pPrintMonitor).bIsLevel2 == 0
        && (*((*pPrintMonitor).pMonitor as LPMONITOREX))
            .Monitor
            .pfnAddPort
            .is_some()
    {
        res = (*((*pPrintMonitor).pMonitor as LPMONITOREX))
            .Monitor
            .pfnAddPort
            .unwrap_unchecked()(pName, hWnd, pMonitorName);
    } else {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    }

    if res != 0 {
        let mut cbNeeded: DWORD = 0;
        let mut cReturned: DWORD = 0;
        let mut i: DWORD = 0;
        let mut pPorts: PPORT_INFO_1 = ::core::ptr::null_mut::<_PORT_INFO_1W>();

        //
        // Play it safe,,, we know its Monitor2.... This is ReactOS.
        //
        if LocalEnumPorts(
            pName as PWSTR,
            1 as DWORD,
            ::core::ptr::null_mut::<BYTE>(),
            0 as DWORD,
            &raw mut cbNeeded,
            &raw mut cReturned,
        ) != 0
        {
            pPorts = DllAllocSplMem(cbNeeded) as PPORT_INFO_1;
            if !pPorts.is_null() {
                if LocalEnumPorts(
                    pName as PWSTR,
                    1 as DWORD,
                    pPorts as PBYTE,
                    cbNeeded,
                    &raw mut cbNeeded,
                    &raw mut cReturned,
                ) != 0
                {
                    i = 0 as DWORD;
                    while i < cReturned {
                        if FindPort((*pPorts.offset(i as isize)).pName as PCWSTR)
                            .is_null()
                        {
                            CreatePortEntry(
                                (*pPorts.offset(i as isize)).pName as PCWSTR,
                                pPrintMonitor,
                            );
                        }
                        i += 1;
                    }
                }
                DllFreeSplMem(pPorts as PVOID);
            }
        }
    }

    return res;
}

#[no_mangle]
pub unsafe extern "system" fn LocalConfigurePort(pName: PWSTR, hWnd: HWND, pPortName: PWSTR) -> BOOL {
    let mut lres: LONG = 0;
    let mut res: DWORD = 0;
    let mut pPrintPort: PLOCAL_PORT = core::ptr::null_mut();
    let mut pPrintMonitor: PLOCAL_PRINT_MONITOR = core::ptr::null_mut();

    lres = copy_servername_from_name(pName as LPCWSTR, core::ptr::null_mut::<WCHAR>());
    if lres != 0 {
        SetLastError(ERROR_INVALID_NAME);
        return FALSE as BOOL;
    }

    /* an empty Portname is Invalid, but can popup a Dialog */
    if *pPortName.offset(0 as isize) == 0 {
        SetLastError(ERROR_NOT_SUPPORTED);
        return FALSE as BOOL;
    }

    pPrintPort = FindPort(pPortName as PCWSTR);
    if pPrintPort.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    pPrintMonitor = (*pPrintPort).pPrintMonitor;

    if (*pPrintMonitor).bIsLevel2 != 0
        && (*((*pPrintMonitor).pMonitor as PMONITOR2))
            .pfnConfigurePort
            .is_some()
    {
        res = (*((*pPrintMonitor).pMonitor as PMONITOR2))
            .pfnConfigurePort
            .unwrap_unchecked()(
            (*pPrintMonitor).hMonitor,
            pName as LPWSTR,
            hWnd,
            pPortName as LPWSTR,
        ) as DWORD;
    } else if (*pPrintMonitor).bIsLevel2 == 0
        && (*((*pPrintMonitor).pMonitor as LPMONITOREX))
            .Monitor
            .pfnConfigurePort
            .is_some()
    {
        res = (*((*pPrintMonitor).pMonitor as LPMONITOREX))
            .Monitor
            .pfnConfigurePort
            .unwrap_unchecked()(
            pName as LPWSTR, hWnd, pPortName as LPWSTR
        ) as DWORD;
    } else {
        SetLastError(ERROR_INVALID_PARAMETER);
    }

    return res as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn LocalDeletePort(mut pName: PWSTR, mut hWnd: HWND, mut pPortName: PWSTR) -> BOOL {
    let mut lres: LONG;
    let mut res: DWORD = FALSE;
    let mut pPrintPort: PLOCAL_PORT;
    let mut pPrintMonitor: PLOCAL_PRINT_MONITOR;

    lres = copy_servername_from_name(pName as LPCWSTR, core::ptr::null_mut());
    if lres != 0 {
        SetLastError(ERROR_INVALID_NAME);
        return FALSE as BOOL;
    }

    if *pPortName.offset(0 as isize) == 0 {
        SetLastError(ERROR_NOT_SUPPORTED);
        return FALSE as BOOL;
    }

    pPrintPort = FindPort(pPortName as PCWSTR);
    if pPrintPort.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    pPrintMonitor = (*pPrintPort).pPrintMonitor;

    if (*pPrintMonitor).bIsLevel2 != 0
        && (*((*pPrintMonitor).pMonitor as PMONITOR2)).pfnDeletePort.is_some()
    {
        res = (*((*pPrintMonitor).pMonitor as PMONITOR2))
            .pfnDeletePort
            .unwrap_unchecked()(
            (*pPrintMonitor).hMonitor,
            pName as LPWSTR,
            hWnd,
            pPortName as LPWSTR,
        ) as DWORD;
    } else if (*pPrintMonitor).bIsLevel2 == 0
        && (*((*pPrintMonitor).pMonitor as LPMONITOREX)).Monitor.pfnDeletePort.is_some()
    {
        res = (*((*pPrintMonitor).pMonitor as LPMONITOREX))
            .Monitor
            .pfnDeletePort
            .unwrap_unchecked()(
            pName as LPWSTR,
            hWnd,
            pPortName as LPWSTR,
        ) as DWORD;
    }

    RemoveEntryList(core::ptr::addr_of_mut!((*pPrintPort).Entry));

    DllFreeSplMem(pPrintPort as PVOID);

    return res as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn LocalSetPort(pName: PWSTR, pPortName: PWSTR, dwLevel: DWORD, pPortInfo: PBYTE) -> BOOL {
    let mut lres: LONG;
    let mut res: DWORD = 0;
    let mut ppi3w: PPORT_INFO_3W = pPortInfo as PPORT_INFO_3W;
    let mut pPrintPort: PLOCAL_PORT;

    lres = copy_servername_from_name(pName as *const u16, core::ptr::null_mut());
    if lres != 0 {
        SetLastError(ERROR_INVALID_NAME);
        return FALSE as BOOL;
    }

    if (dwLevel < 1) || (dwLevel > 2) {
        SetLastError(ERROR_INVALID_LEVEL);
        return FALSE as BOOL;
    }

    if ppi3w.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    pPrintPort = FindPort(pPortName as PCWSTR);
    if pPrintPort.is_null() {
        SetLastError(ERROR_UNKNOWN_PORT);
        return FALSE as BOOL;
    }

    return res as BOOL;
}
