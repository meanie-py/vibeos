use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn AddPortExW(
    pName: PWSTR,
    Level: DWORD,
    lpBuffer: PBYTE,
    lpMonitorName: PWSTR,
) -> BOOL {
    let mut bReturnValue: BOOL = TRUE as BOOL;
    let mut dwErrorCode: DWORD = MAXDWORD;
    let mut pPrintProvider: PSPOOLSS_PRINT_PROVIDER = core::ptr::null_mut();
    let mut pEntry: PLIST_ENTRY = core::ptr::null_mut();
    pEntry = (*core::ptr::addr_of_mut!(PrintProviderList)).Flink;
    while pEntry != core::ptr::addr_of_mut!(PrintProviderList) {
        'body: {
            pPrintProvider = ((pEntry as ULONG_PTR).wrapping_sub(
                core::mem::offset_of!(SPOOLSS_PRINT_PROVIDER, Entry) as ULONG_PTR,
            )) as *mut SPOOLSS_PRINT_PROVIDER as PSPOOLSS_PRINT_PROVIDER;

            if (*pPrintProvider).PrintProvider.fpAddPortEx.is_none() {
                break 'body;
            }

            bReturnValue = (*pPrintProvider)
                .PrintProvider
                .fpAddPortEx
                .unwrap_unchecked()(
                pName as LPWSTR,
                Level,
                lpBuffer as LPBYTE,
                lpMonitorName as LPWSTR,
            );

            if bReturnValue == 0 {
                dwErrorCode = GetLastError();
            }

            if dwErrorCode != ERROR_SUCCESS {
                dwErrorCode = GetLastError();
            }
        }
        pEntry = (*pEntry).Flink;
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn AddPortW(pName: PWSTR, hWnd: HWND, pMonitorName: PWSTR) -> BOOL {
    let mut bReturnValue: BOOL = TRUE as BOOL;
    let mut dwErrorCode: DWORD = MAXDWORD;
    let mut pPrintProvider: PSPOOLSS_PRINT_PROVIDER = core::ptr::null_mut();
    let mut pEntry: PLIST_ENTRY = core::ptr::null_mut();

    // Loop through all Print Provider.
    pEntry = (*core::ptr::addr_of_mut!(PrintProviderList)).Flink as PLIST_ENTRY;
    while pEntry != core::ptr::addr_of_mut!(PrintProviderList) {
        pPrintProvider = CONTAINING_RECORD!(pEntry, SPOOLSS_PRINT_PROVIDER, Entry) as PSPOOLSS_PRINT_PROVIDER;

        // Check if this Print Provider provides the function.
        if !(*pPrintProvider).PrintProvider.fpAddPort.is_none() {
            bReturnValue = (*pPrintProvider)
                .PrintProvider
                .fpAddPort
                .unwrap_unchecked()(
                pName as LPWSTR, hWnd, pMonitorName as LPWSTR
            );

            if bReturnValue == 0 {
                dwErrorCode = GetLastError();
            }

            // dwErrorCode shall not be overwritten if a previous call already succeeded.
            if dwErrorCode != ERROR_SUCCESS as DWORD {
                dwErrorCode = GetLastError();
            }
        }

        pEntry = (*pEntry).Flink as PLIST_ENTRY;
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS as DWORD) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ConfigurePortW(pName: PWSTR, hWnd: HWND, pPortName: PWSTR) -> BOOL {
    let mut bReturnValue: BOOL = TRUE as BOOL;
    let mut dwErrorCode: DWORD = MAXDWORD;
    let mut pPrintProvider: PSPOOLSS_PRINT_PROVIDER;
    let mut pEntry: PLIST_ENTRY;

    // Loop through all Print Provider.
    pEntry = (*core::ptr::addr_of_mut!(PrintProviderList)).Flink;
    while pEntry != core::ptr::addr_of_mut!(PrintProviderList) {
        pPrintProvider = CONTAINING_RECORD!(pEntry, SPOOLSS_PRINT_PROVIDER, Entry);

        // Check if this Print Provider provides the function.
        if !(*pPrintProvider).PrintProvider.fpConfigurePort.is_none() {
            bReturnValue = (*pPrintProvider)
                .PrintProvider
                .fpConfigurePort
                .unwrap_unchecked()(pName, hWnd, pPortName);

            if bReturnValue == 0 {
                dwErrorCode = GetLastError();
            }

            // dwErrorCode shall not be overwritten if a previous call already succeeded.
            if dwErrorCode != ERROR_SUCCESS {
                dwErrorCode = GetLastError();
            }
        }
        pEntry = (*pEntry).Flink;
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn DeletePortW(pName: PWSTR, hWnd: HWND, pPortName: PWSTR) -> BOOL {
    let mut bReturnValue: BOOL = TRUE as BOOL;
    let mut dwErrorCode: DWORD = MAXDWORD;
    let mut pPrintProvider: PSPOOLSS_PRINT_PROVIDER = core::ptr::null_mut();
    let mut pEntry: PLIST_ENTRY = core::ptr::null_mut();

    // Loop through all Print Provider.
    pEntry = PrintProviderList.Flink;
    while pEntry != core::ptr::addr_of_mut!(PrintProviderList) {
        pPrintProvider = CONTAINING_RECORD!(pEntry, SPOOLSS_PRINT_PROVIDER, Entry);

        // Check if this Print Provider provides the function.
        if (*pPrintProvider).PrintProvider.fpDeletePort.is_some() {
            bReturnValue = (*pPrintProvider)
                .PrintProvider
                .fpDeletePort
                .unwrap_unchecked()(
                pName as LPWSTR,
                hWnd,
                pPortName as LPWSTR,
            );

            if bReturnValue == 0 {
                dwErrorCode = GetLastError();
            }

            // dwErrorCode shall not be overwritten if a previous call already succeeded.
            if dwErrorCode != ERROR_SUCCESS {
                dwErrorCode = GetLastError();
            }
        }

        pEntry = (*pEntry).Flink;
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn EnumPortsW(
    mut pName: PWSTR,
    mut Level: DWORD,
    mut pPorts: PBYTE,
    mut cbBuf: DWORD,
    mut pcbNeeded: PDWORD,
    mut pcReturned: PDWORD,
) -> BOOL {
    let mut bReturnValue: BOOL = TRUE as BOOL;
    let mut cbCallBuffer: DWORD;
    let mut cbNeeded: DWORD;
    let mut dwReturned: DWORD;
    let mut dwErrorCode: DWORD = MAXDWORD;
    let mut pCallBuffer: PBYTE;
    let mut pPrintProvider: PSPOOLSS_PRINT_PROVIDER;
    let mut pEntry: PLIST_ENTRY;

    // Sanity checks.
    if cbBuf != 0 && pPorts.is_null() {
        SetLastError(ERROR_INVALID_USER_BUFFER);
        return FALSE as BOOL;
    }

    // Begin counting.
    *pcbNeeded = 0;
    *pcReturned = 0;

    // At the beginning, we have the full buffer available.
    cbCallBuffer = cbBuf;
    pCallBuffer = pPorts;

    // Loop through all Print Provider.
    pEntry = PrintProviderList.Flink as PLIST_ENTRY;
    while pEntry != &raw mut PrintProviderList {
        pPrintProvider = CONTAINING_RECORD!(pEntry, SPOOLSS_PRINT_PROVIDER, Entry);

        // Check if this Print Provider provides an EnumPorts function.
        if !(*pPrintProvider).PrintProvider.fpEnumPorts.is_none() {
            // Call the EnumPorts function of this Print Provider.
            cbNeeded = 0;
            dwReturned = 0;
            bReturnValue = (*pPrintProvider)
                .PrintProvider
                .fpEnumPorts
                .unwrap_unchecked()(
                pName as LPWSTR,
                Level,
                pCallBuffer as LPBYTE,
                cbCallBuffer,
                &raw mut cbNeeded,
                &raw mut dwReturned,
            );

            if bReturnValue == 0 {
                dwErrorCode = GetLastError();
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

            // dwErrorCode shall not be overwritten if a previous EnumPrinters call already succeeded.
            if dwErrorCode != ERROR_SUCCESS {
                dwErrorCode = GetLastError();
            }
        }
        pEntry = (*pEntry).Flink as PLIST_ENTRY;
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetPortW(
    pName: PWSTR,
    pPortName: PWSTR,
    dwLevel: DWORD,
    pPortInfo: PBYTE,
) -> BOOL {
    let mut bReturnValue: BOOL = TRUE as BOOL;
    let mut dwErrorCode: DWORD = MAXDWORD;
    let mut pPrintProvider: PSPOOLSS_PRINT_PROVIDER = core::ptr::null_mut();
    let mut pEntry: PLIST_ENTRY = core::ptr::null_mut();

    // Loop through all Print Provider.
    pEntry = (*core::ptr::addr_of_mut!(PrintProviderList)).Flink as PLIST_ENTRY;
    while pEntry != core::ptr::addr_of_mut!(PrintProviderList) {
        pPrintProvider = CONTAINING_RECORD!(pEntry, SPOOLSS_PRINT_PROVIDER, Entry) as PSPOOLSS_PRINT_PROVIDER;

        // Check if this Print Provider provides the function.
        if !(*pPrintProvider).PrintProvider.fpSetPort.is_none() {
            bReturnValue = (*pPrintProvider)
                .PrintProvider
                .fpSetPort
                .unwrap_unchecked()(
                pName as LPWSTR,
                pPortName as LPWSTR,
                dwLevel,
                pPortInfo as LPBYTE,
            );

            if bReturnValue == 0 {
                dwErrorCode = GetLastError();
            }

            // dwErrorCode shall not be overwritten if a previous call already succeeded.
            if dwErrorCode != ERROR_SUCCESS {
                dwErrorCode = GetLastError();
            }
        }
        pEntry = (*pEntry).Flink as PLIST_ENTRY;
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}
