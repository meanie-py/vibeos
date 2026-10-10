use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn AddMonitorW(pName: PWSTR, Level: DWORD, pMonitors: PBYTE) -> BOOL {
    let mut bReturnValue: BOOL = TRUE as BOOL;
    let mut dwErrorCode: DWORD = MAXDWORD;
    let mut pPrintProvider: PSPOOLSS_PRINT_PROVIDER;
    let mut pEntry: PLIST_ENTRY;

    // Loop through all Print Provider.
    pEntry = (*core::ptr::addr_of_mut!(PrintProviderList)).Flink;
    while pEntry != core::ptr::addr_of_mut!(PrintProviderList) {
        pPrintProvider = CONTAINING_RECORD!(pEntry, SPOOLSS_PRINT_PROVIDER, Entry);

        // Check if this Print Provider provides the function.
        if !(*pPrintProvider).PrintProvider.fpAddMonitor.is_none() {
            bReturnValue = (*pPrintProvider)
                .PrintProvider
                .fpAddMonitor
                .unwrap_unchecked()(pName, Level, pMonitors);

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
pub unsafe extern "system" fn DeleteMonitorW(
    pName: PWSTR,
    pEnvironment: PWSTR,
    pMonitorName: PWSTR,
) -> BOOL {
    let mut bReturnValue: BOOL = TRUE as BOOL;
    let mut dwErrorCode: DWORD = MAXDWORD;
    let mut pPrintProvider: PSPOOLSS_PRINT_PROVIDER;
    let mut pEntry: PLIST_ENTRY;

    // Loop through all Print Provider.
    pEntry = (*core::ptr::addr_of_mut!(PrintProviderList)).Flink;
    while pEntry != core::ptr::addr_of_mut!(PrintProviderList) {
        'body: {
            pPrintProvider = CONTAINING_RECORD!(pEntry, SPOOLSS_PRINT_PROVIDER, Entry);

            // Check if this Print Provider provides the function.
            if (*pPrintProvider).PrintProvider.fpDeleteMonitor.is_none() {
                break 'body;
            }

            bReturnValue = (*pPrintProvider)
                .PrintProvider
                .fpDeleteMonitor
                .unwrap_unchecked()(
                pName,
                pEnvironment,
                pMonitorName,
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
pub unsafe extern "system" fn EnumMonitorsW(
    pName: PWSTR,
    Level: DWORD,
    pMonitors: PBYTE,
    cbBuf: DWORD,
    pcbNeeded: PDWORD,
    pcReturned: PDWORD,
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
    if cbBuf != 0 && pMonitors.is_null() {
        SetLastError(ERROR_INVALID_USER_BUFFER);
        return FALSE as BOOL;
    }

    // Begin counting.
    *pcbNeeded = 0;
    *pcReturned = 0;

    // At the beginning, we have the full buffer available.
    cbCallBuffer = cbBuf;
    pCallBuffer = pMonitors;

    // Loop through all Print Provider.
    pEntry = PrintProviderList.Flink as PLIST_ENTRY;
    while pEntry != core::ptr::addr_of_mut!(PrintProviderList) {
        pPrintProvider = CONTAINING_RECORD!(pEntry, SPOOLSS_PRINT_PROVIDER, Entry);

        // Check if this Print Provider provides an EnumMonitors function.
        if (*pPrintProvider).PrintProvider.fpEnumMonitors.is_some() {
            // Call the EnumMonitors function of this Print Provider.
            cbNeeded = 0;
            dwReturned = 0;
            bReturnValue = (*pPrintProvider)
                .PrintProvider
                .fpEnumMonitors
                .unwrap_unchecked()(
                pName as LPWSTR,
                Level,
                pCallBuffer as LPBYTE,
                cbCallBuffer,
                core::ptr::addr_of_mut!(cbNeeded),
                core::ptr::addr_of_mut!(dwReturned),
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
