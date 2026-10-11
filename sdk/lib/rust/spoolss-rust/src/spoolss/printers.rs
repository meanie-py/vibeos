use crate::spoolss::memory::DllAllocSplMem;
use crate::spoolss::memory::DllFreeSplMem;
use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn AbortPrinter(hPrinter: HANDLE) -> BOOL {
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpAbortPrinter
        .unwrap_unchecked()((*pHandle).hPrinter) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn AddPrinterExW(
    mut pName: PWSTR,
    mut Level: DWORD,
    mut pPrinter: PBYTE,
    mut pClientInfo: PBYTE,
    mut ClientInfoLevel: DWORD,
) -> HANDLE {
    let mut bReturnValue: BOOL = 0;
    let mut dwErrorCode: DWORD = ERROR_INVALID_PRINTER_NAME;
    let mut hPrinter: HANDLE = core::ptr::null_mut();
    let mut pPrinterName: PWSTR = core::ptr::null_mut();
    let mut pEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = core::ptr::null_mut();
    let mut pPrintProvider: PSPOOLSS_PRINT_PROVIDER = core::ptr::null_mut();

    if Level != 2 {
        SetLastError(ERROR_INVALID_LEVEL);
        return hPrinter;
    } else {
        let mut pi2w: PPRINTER_INFO_2W = pPrinter as PPRINTER_INFO_2W;
        pPrinterName = (*pi2w).pPrinterName as PWSTR;
    }

    'Cleanup: {
        pEntry = PrintProviderList.Flink as PLIST_ENTRY;
        while pEntry != &raw mut PrintProviderList {
            pPrintProvider = CONTAINING_RECORD!(pEntry, SPOOLSS_PRINT_PROVIDER, Entry);

            hPrinter = (*pPrintProvider)
                .PrintProvider
                .fpAddPrinterEx
                .unwrap_unchecked()(
                pName as LPWSTR,
                Level,
                pPrinter as LPBYTE,
                pClientInfo as LPBYTE,
                ClientInfoLevel,
            );

            bReturnValue = GetLastError() as BOOL;

            if hPrinter.is_null() && bReturnValue == ERROR_NOT_SUPPORTED as BOOL {
                hPrinter = (*pPrintProvider)
                    .PrintProvider
                    .fpAddPrinter
                    .unwrap_unchecked()(
                    pName as LPWSTR,
                    Level,
                    pPrinter as LPBYTE,
                );
            }

            bReturnValue = GetLastError() as BOOL;

            if bReturnValue == ROUTER_SUCCESS as BOOL && !hPrinter.is_null() {
                pHandle = DllAllocSplMem(
                    core::mem::size_of::<SPOOLSS_PRINTER_HANDLE>() as DWORD,
                ) as PSPOOLSS_PRINTER_HANDLE;
                if pHandle.is_null() {
                    dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                    break 'Cleanup;
                }

                (*pHandle).pPrintProvider = pPrintProvider;
                (*pHandle).hPrinter = hPrinter;

                dwErrorCode = ERROR_SUCCESS;
                break 'Cleanup;
            } else if bReturnValue == ROUTER_STOP_ROUTING as BOOL {
                dwErrorCode = GetLastError() as DWORD;
                break 'Cleanup;
            }

            pEntry = (*pEntry).Flink as PLIST_ENTRY;
        }
    }

    // ERROR_INVALID_NAME by the Print Provider is translated to ERROR_INVALID_PRINTER_NAME here, but not in other APIs as far as I know.
    if dwErrorCode == ERROR_INVALID_NAME {
        dwErrorCode = ERROR_INVALID_PRINTER_NAME;
    }

    SetLastError(dwErrorCode);
    return hPrinter;
}

#[no_mangle]
pub unsafe extern "system" fn AddPrinterW(
    pName: PWSTR,
    Level: DWORD,
    pPrinter: PBYTE,
) -> HANDLE {
    let mut bReturnValue: BOOL = 0;
    let mut dwErrorCode: DWORD = ERROR_INVALID_PRINTER_NAME as DWORD;
    let mut hPrinter: HANDLE = core::ptr::null_mut();
    let mut pPrinterName: PWSTR = core::ptr::null_mut();
    let mut pEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = core::ptr::null_mut();
    let mut pPrintProvider: PSPOOLSS_PRINT_PROVIDER = core::ptr::null_mut();

    if Level != 2 as DWORD {
        SetLastError(ERROR_INVALID_LEVEL as DWORD);
        return hPrinter;
    } else {
        let pi2w: PPRINTER_INFO_2W = pPrinter as PPRINTER_INFO_2W;
        pPrinterName = (*pi2w).pPrinterName as PWSTR;
    }

    pEntry = PrintProviderList.Flink as PLIST_ENTRY;
    'cleanup: {
        while pEntry != core::ptr::addr_of_mut!(PrintProviderList) {
            pPrintProvider = ((pEntry as ULONG_PTR).wrapping_sub(
                core::ptr::addr_of_mut!(
                    (*core::ptr::null_mut::<SPOOLSS_PRINT_PROVIDER>()).Entry
                ) as ULONG_PTR,
            ) as *mut SPOOLSS_PRINT_PROVIDER) as PSPOOLSS_PRINT_PROVIDER;

            hPrinter = (*pPrintProvider)
                .PrintProvider
                .fpAddPrinter
                .unwrap_unchecked()(
                pName as LPWSTR,
                Level,
                pPrinter as LPBYTE,
            );

            bReturnValue = GetLastError() as BOOL;

            if bReturnValue == ROUTER_SUCCESS as BOOL && !hPrinter.is_null() {
                pHandle = DllAllocSplMem(
                    core::mem::size_of::<SPOOLSS_PRINTER_HANDLE>() as DWORD,
                ) as PSPOOLSS_PRINTER_HANDLE;
                if pHandle.is_null() {
                    dwErrorCode = ERROR_NOT_ENOUGH_MEMORY as DWORD;
                    break 'cleanup;
                }

                (*pHandle).pPrintProvider = pPrintProvider;
                (*pHandle).hPrinter = hPrinter;

                dwErrorCode = ERROR_SUCCESS as DWORD;
                break 'cleanup;
            } else if bReturnValue == ROUTER_STOP_ROUTING as BOOL {
                dwErrorCode = GetLastError() as DWORD;
                break 'cleanup;
            }

            pEntry = (*pEntry).Flink as PLIST_ENTRY;
        }
    }

    if dwErrorCode == ERROR_INVALID_NAME as DWORD {
        dwErrorCode = ERROR_INVALID_PRINTER_NAME as DWORD;
    }

    SetLastError(dwErrorCode);
    return hPrinter;
}

#[no_mangle]
pub unsafe extern "system" fn ClosePrinter(hPrinter: HANDLE) -> WINBOOL {
    let mut bReturnValue: BOOL = 0;
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as WINBOOL;
    }
    bReturnValue = ((*(*pHandle).pPrintProvider).PrintProvider.fpClosePrinter
        .unwrap_unchecked())((*pHandle).hPrinter);
    DllFreeSplMem(pHandle as PVOID);
    return bReturnValue;
}

#[no_mangle]
pub unsafe extern "system" fn DeletePrinter(hPrinter: HANDLE) -> BOOL {
    let pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpDeletePrinter
        .unwrap_unchecked()((*pHandle).hPrinter) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn EndDocPrinter(hPrinter: HANDLE) -> BOOL {
    let pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpEndDocPrinter
        .unwrap_unchecked()((*pHandle).hPrinter);
}

#[no_mangle]
pub unsafe extern "system" fn EndPagePrinter(hPrinter: HANDLE) -> BOOL {
    let pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpEndPagePrinter
        .unwrap_unchecked()((*pHandle).hPrinter);
}

#[no_mangle]
pub unsafe extern "system" fn EnumPrintersW(
    Flags: DWORD,
    Name: PWSTR,
    Level: DWORD,
    pPrinterEnum: PBYTE,
    cbBuf: DWORD,
    pcbNeeded: PDWORD,
    pcReturned: PDWORD,
) -> BOOL {
    let mut cbCallBuffer: DWORD = 0;
    let mut cbNeeded: DWORD = 0;
    let mut dwErrorCode: DWORD = MAXDWORD as DWORD;
    let mut dwReturned: DWORD = 0;
    let mut pCallBuffer: PBYTE = core::ptr::null_mut();
    let mut Ret: BOOL = FALSE as BOOL;
    let mut pPrintProvider: PSPOOLSS_PRINT_PROVIDER = core::ptr::null_mut();
    let mut pEntry: PLIST_ENTRY = core::ptr::null_mut();

    // Begin counting.
    *pcbNeeded = 0 as DWORD;
    *pcReturned = 0 as DWORD;

    'Cleanup: {
        if cbBuf != 0 && pPrinterEnum.is_null() {
            dwErrorCode = ERROR_INVALID_USER_BUFFER as DWORD;
            break 'Cleanup;
        }

        // At the beginning, we have the full buffer available.
        cbCallBuffer = cbBuf;
        pCallBuffer = pPrinterEnum;

        // Loop through all Print Providers.
        pEntry = (*core::ptr::addr_of_mut!(PrintProviderList)).Flink as PLIST_ENTRY;
        while pEntry != core::ptr::addr_of_mut!(PrintProviderList) {
            pPrintProvider = CONTAINING_RECORD!(pEntry, SPOOLSS_PRINT_PROVIDER, Entry)
                as PSPOOLSS_PRINT_PROVIDER;

            // Call the EnumPrinters function of this Print Provider.
            cbNeeded = 0 as DWORD;
            dwReturned = 0 as DWORD;
            Ret = (*pPrintProvider)
                .PrintProvider
                .fpEnumPrinters
                .unwrap_unchecked()(
                Flags,
                Name as LPWSTR,
                Level,
                pCallBuffer as LPBYTE,
                cbCallBuffer,
                core::ptr::addr_of_mut!(cbNeeded),
                core::ptr::addr_of_mut!(dwReturned),
            );

            if Ret == 0 {
                dwErrorCode = GetLastError();
            }

            // Add the returned counts to the total values.
            *pcbNeeded = (*pcbNeeded).wrapping_add(cbNeeded);
            *pcReturned = (*pcReturned).wrapping_add(dwReturned);

            // Reduce the available buffer size for the next call without risking an underflow.
            if cbNeeded < cbCallBuffer {
                cbCallBuffer = cbCallBuffer.wrapping_sub(cbNeeded);
            } else {
                cbCallBuffer = 0 as DWORD;
            }

            // Advance the buffer if the caller provided it.
            if !pCallBuffer.is_null() {
                pCallBuffer = pCallBuffer.offset(cbNeeded as isize);
            }

            // dwErrorCode shall not be overwritten if a previous EnumPrinters call already succeeded.
            if dwErrorCode != ERROR_SUCCESS as DWORD {
                dwErrorCode = GetLastError();
            }

            pEntry = (*pEntry).Flink as PLIST_ENTRY;
        }
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS as DWORD) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetPrinterW(hPrinter: HANDLE, Level: DWORD, pPrinter: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD) -> BOOL {
    let pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    return (*(*pHandle).pPrintProvider).PrintProvider.fpGetPrinter.unwrap_unchecked()((*pHandle).hPrinter, Level, pPrinter, cbBuf, pcbNeeded);
}

#[no_mangle]
pub unsafe extern "system" fn PrinterMessageBoxW(hPrinter: HANDLE, Error: DWORD, hWnd: HWND, pText: LPWSTR, pCaption: LPWSTR, dwType: DWORD) -> DWORD {
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpPrinterMessageBox
        .unwrap_unchecked()(
        (*pHandle).hPrinter,
        Error,
        hWnd,
        pText,
        pCaption,
        dwType,
    );
}

#[no_mangle]
pub unsafe extern "system" fn OpenPrinterW(pPrinterName: PWSTR, phPrinter: PHANDLE, pDefault: PPRINTER_DEFAULTSW) -> WINBOOL {
    let mut bReturnValue: BOOL = 0;
    let mut dwErrorCode: DWORD = ERROR_INVALID_PRINTER_NAME as DWORD;
    let mut hPrinter: HANDLE = core::ptr::null_mut();
    let mut pEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = core::ptr::null_mut();
    let mut pPrintProvider: PSPOOLSS_PRINT_PROVIDER = core::ptr::null_mut();

    'Cleanup: {
        pEntry = (*core::ptr::addr_of_mut!(PrintProviderList)).Flink;
        while pEntry != core::ptr::addr_of_mut!(PrintProviderList) as PLIST_ENTRY {
            pPrintProvider =
                CONTAINING_RECORD!(pEntry, SPOOLSS_PRINT_PROVIDER, Entry) as PSPOOLSS_PRINT_PROVIDER;

            bReturnValue = (*pPrintProvider)
                .PrintProvider
                .fpOpenPrinter
                .unwrap_unchecked()(
                pPrinterName,
                core::ptr::addr_of_mut!(hPrinter),
                pDefault,
            );
            if bReturnValue == ROUTER_SUCCESS as BOOL {
                pHandle = DllAllocSplMem(
                    core::mem::size_of::<SPOOLSS_PRINTER_HANDLE>() as DWORD,
                ) as PSPOOLSS_PRINTER_HANDLE;
                if pHandle.is_null() {
                    dwErrorCode = ERROR_NOT_ENOUGH_MEMORY as DWORD;
                    break 'Cleanup;
                }

                (*pHandle).pPrintProvider = pPrintProvider;
                (*pHandle).hPrinter = hPrinter;
                *phPrinter = pHandle as HANDLE;

                dwErrorCode = ERROR_SUCCESS as DWORD;
                break 'Cleanup;
            } else if bReturnValue == ROUTER_STOP_ROUTING as BOOL {
                dwErrorCode = GetLastError();
                break 'Cleanup;
            }

            pEntry = (*pEntry).Flink;
        }
    }

    // ERROR_INVALID_NAME by the Print Provider is translated to ERROR_INVALID_PRINTER_NAME here, but not in other APIs as far as I know.
    if dwErrorCode == ERROR_INVALID_NAME as DWORD {
        dwErrorCode = ERROR_INVALID_PRINTER_NAME as DWORD;
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS as DWORD) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ReadPrinter(
    hPrinter: HANDLE,
    pBuf: PVOID,
    cbBuf: DWORD,
    pNoBytesRead: PDWORD,
) -> BOOL {
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpReadPrinter
        .unwrap_unchecked()(
        (*pHandle).hPrinter,
        pBuf as LPVOID,
        cbBuf,
        pNoBytesRead as LPDWORD,
    ) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SeekPrinter(hPrinter: HANDLE, liDistanceToMove: LARGE_INTEGER, pliNewPointer: PLARGE_INTEGER, dwMoveMethod: DWORD, bWrite: BOOL) -> BOOL {
    let pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }
    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpSeekPrinter
        .unwrap_unchecked()(
        (*pHandle).hPrinter,
        liDistanceToMove,
        pliNewPointer,
        dwMoveMethod,
        bWrite,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SetPrinterW(
    hPrinter: HANDLE,
    Level: DWORD,
    pPrinter: PBYTE,
    Command: DWORD,
) -> BOOL {
    let pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpSetPrinter
        .unwrap_unchecked()(
        (*pHandle).hPrinter,
        Level,
        pPrinter as LPBYTE,
        Command,
    );
}

#[no_mangle]
pub unsafe extern "system" fn StartDocPrinterW(hPrinter: HANDLE, Level: DWORD, pDocInfo: PBYTE) -> DWORD {
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpStartDocPrinter
        .unwrap_unchecked()(
        (*pHandle).hPrinter, Level, pDocInfo
    );
}

#[no_mangle]
pub unsafe extern "system" fn StartPagePrinter(hPrinter: HANDLE) -> BOOL {
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpStartPagePrinter
        .unwrap_unchecked()((*pHandle).hPrinter) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn WritePrinter(hPrinter: HANDLE, pBuf: PVOID, cbBuf: DWORD, pcWritten: PDWORD) -> BOOL {
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }
    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpWritePrinter
        .unwrap_unchecked()(
        (*pHandle).hPrinter,
        pBuf,
        cbBuf,
        pcWritten,
    );
}

#[no_mangle]
pub unsafe extern "system" fn XcvDataW(hXcv: HANDLE, pszDataName: PCWSTR, pInputData: PBYTE, cbInputData: DWORD, pOutputData: PBYTE, cbOutputData: DWORD, pcbOutputNeeded: PDWORD, pdwStatus: PDWORD) -> BOOL {
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hXcv as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpXcvData
        .unwrap_unchecked()(
        (*pHandle).hPrinter,
        pszDataName as LPCWSTR,
        pInputData,
        cbInputData,
        pOutputData,
        cbOutputData,
        pcbOutputNeeded,
        pdwStatus,
    ) as BOOL;
}
