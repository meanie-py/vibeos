use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn ros_dbg_log( dbcl: __wine_debug_class, dbch: *mut __wine_debug_channel, file: *const core::ffi::c_char, func: *const core::ffi::c_char, line: core::ffi::c_int, fmt: *const core::ffi::c_char, ... );
}

#[no_mangle]
pub unsafe extern "system" fn AddPrinterDriverExW(mut pName: PWSTR, mut Level: DWORD, mut pDriverInfo: PBYTE, mut dwFileCopyFlags: DWORD) -> WINBOOL {
    let mut bReturnValue: BOOL = 0;
    let mut dwErrorCode: DWORD = ERROR_INVALID_PRINTER_NAME as DWORD;
    let mut pEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut pPrintProvider: PSPOOLSS_PRINT_PROVIDER = core::ptr::null_mut();

    // Loop through all Print Providers.
    pEntry = PrintProviderList.Flink as PLIST_ENTRY;
    'Cleanup: {
        while pEntry != core::ptr::addr_of_mut!(PrintProviderList) {
            pPrintProvider = CONTAINING_RECORD!(pEntry, SPOOLSS_PRINT_PROVIDER, Entry);

            bReturnValue = (*pPrintProvider)
                .PrintProvider
                .fpAddPrinterDriverEx
                .unwrap_unchecked()(
                pName as LPWSTR,
                Level,
                pDriverInfo as LPBYTE,
                dwFileCopyFlags,
            );

            if bReturnValue == ROUTER_SUCCESS as BOOL {
                dwErrorCode = ERROR_SUCCESS as DWORD;
                break 'Cleanup;
            } else if bReturnValue == ROUTER_STOP_ROUTING as BOOL {
                if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"win32ss/printing/base/spoolss/printerdrivers.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"AddPrinterDriverExW\0" as *const u8 as *const ::core::ffi::c_char,
                        32 as ::core::ffi::c_int,
                        b"A Print Provider returned ROUTER_STOP_ROUTING for Printer \"%S\"!\n\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        pName,
                    );
                }
                dwErrorCode = GetLastError() as DWORD;
                break 'Cleanup;
            }

            pEntry = (*pEntry).Flink as PLIST_ENTRY;
        }
    }

    // ERROR_INVALID_NAME by the Print Provider is translated to ERROR_INVALID_PRINTER_NAME here, but not in other APIs as far as I know.
    if dwErrorCode == ERROR_INVALID_NAME as DWORD {
        dwErrorCode = ERROR_INVALID_PRINTER_NAME as DWORD;
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == (ERROR_SUCCESS as DWORD)) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn AddPrinterDriverW(pName: PWSTR, Level: DWORD, pDriverInfo: PBYTE) -> BOOL {
    return AddPrinterDriverExW(pName, Level, pDriverInfo, APD_COPY_NEW_FILES);
}

#[no_mangle]
pub unsafe extern "system" fn DeletePrinterDriverExW(mut pName: PWSTR, mut pEnvironment: PWSTR, mut pDriverName: PWSTR, mut dwDeleteFlag: DWORD, mut dwVersionFlag: DWORD) -> WINBOOL {
    let mut bReturnValue: BOOL = 0;
    let mut dwErrorCode: DWORD = ERROR_INVALID_PRINTER_NAME;
    let mut pEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut pPrintProvider: PSPOOLSS_PRINT_PROVIDER = core::ptr::null_mut();
    pEntry = PrintProviderList.Flink as PLIST_ENTRY;
    while pEntry != &raw mut PrintProviderList {
        pPrintProvider = ((pEntry as usize)
            .wrapping_sub(core::mem::offset_of!(SPOOLSS_PRINT_PROVIDER, Entry)))
            as PSPOOLSS_PRINT_PROVIDER;
        bReturnValue = (*pPrintProvider)
            .PrintProvider
            .fpDeletePrinterDriverEx
            .unwrap_unchecked()(
            pName as LPWSTR,
            pEnvironment as LPWSTR,
            pDriverName as LPWSTR,
            dwDeleteFlag,
            dwVersionFlag,
        );
        if bReturnValue == ROUTER_SUCCESS as BOOL {
            dwErrorCode = ERROR_SUCCESS;
            break;
        } else if bReturnValue == ROUTER_STOP_ROUTING as BOOL {
            if (*__wine_dbch___default).flags as i32
                & (1 as i32) << __WINE_DBCL_ERR as i32
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"win32ss/printing/base/spoolss/printerdrivers.c\0"
                        .as_ptr() as *const i8,
                    b"DeletePrinterDriverExW\0".as_ptr() as *const i8,
                    76 as i32,
                    b"A Print Provider returned ROUTER_STOP_ROUTING for Printer \"%S\"!\n\0"
                        .as_ptr() as *const i8,
                    pName,
                );
            }
            dwErrorCode = GetLastError();
            break;
        } else {
            pEntry = (*pEntry).Flink as PLIST_ENTRY;
        }
    }
    if dwErrorCode == ERROR_INVALID_NAME {
        dwErrorCode = ERROR_INVALID_PRINTER_NAME;
    }
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn DeletePrinterDriverW(pName: PWSTR, pEnvironment: PWSTR, pDriverName: PWSTR) -> i32 {
    return DeletePrinterDriverExW(pName, pEnvironment, pDriverName, 0, 0);
}

#[no_mangle]
pub unsafe extern "system" fn EnumPrinterDriversW(pName: PWSTR, pEnvironment: PWSTR, Level: DWORD, pDriverInfo: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD, pcReturned: PDWORD) -> BOOL {
    let mut cbCallBuffer: DWORD;
    let mut cbNeeded: DWORD;
    let mut dwErrorCode: DWORD = MAXDWORD;
    let mut dwReturned: DWORD;
    let mut pCallBuffer: PBYTE;
    let mut Ret: BOOL = FALSE as BOOL;
    let mut pPrintProvider: PSPOOLSS_PRINT_PROVIDER;
    let mut pEntry: PLIST_ENTRY;

    // Begin counting.
    *pcbNeeded = 0;
    *pcReturned = 0;

    'Cleanup: {
        if cbBuf != 0 && pDriverInfo.is_null() {
            dwErrorCode = ERROR_INVALID_USER_BUFFER;
            break 'Cleanup;
        }

        // At the beginning, we have the full buffer available.
        cbCallBuffer = cbBuf;
        pCallBuffer = pDriverInfo;

        // Loop through all Print Providers.
        pEntry = (*core::ptr::addr_of_mut!(PrintProviderList)).Flink;
        while pEntry != core::ptr::addr_of_mut!(PrintProviderList) {
            pPrintProvider = CONTAINING_RECORD!(pEntry, SPOOLSS_PRINT_PROVIDER, Entry);

            // Call the EnumPrinters function of this Print Provider.
            cbNeeded = 0;
            dwReturned = 0;

            Ret = (*pPrintProvider)
                .PrintProvider
                .fpEnumPrinterDrivers
                .unwrap_unchecked()(
                pName as LPWSTR,
                pEnvironment as LPWSTR,
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

            pEntry = (*pEntry).Flink;
        }
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetPrinterDriverW(hPrinter: HANDLE, pEnvironment: PWSTR, Level: DWORD, pDriverInfo: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD) -> BOOL {
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpGetPrinterDriver
        .unwrap_unchecked()(
        (*pHandle).hPrinter,
        pEnvironment as LPWSTR,
        Level,
        pDriverInfo as LPBYTE,
        cbBuf,
        pcbNeeded as LPDWORD,
    ) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetPrinterDriverExW(
    hPrinter: HANDLE,
    pEnvironment: LPWSTR,
    Level: DWORD,
    pDriverInfo: LPBYTE,
    cbBuf: DWORD,
    pcbNeeded: LPDWORD,
    dwClientMajorVersion: DWORD,
    dwClientMinorVersion: DWORD,
    pdwServerMajorVersion: PDWORD,
    pdwServerMinorVersion: PDWORD,
) -> BOOL {
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    if cbBuf != 0 && pDriverInfo.is_null() {
        SetLastError(ERROR_INVALID_USER_BUFFER);
        return FALSE as BOOL;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpGetPrinterDriverEx
        .unwrap_unchecked()(
        (*pHandle).hPrinter,
        pEnvironment,
        Level,
        pDriverInfo,
        cbBuf,
        pcbNeeded,
        dwClientMajorVersion,
        dwClientMinorVersion,
        pdwServerMajorVersion,
        pdwServerMinorVersion,
    );
}

#[no_mangle]
pub unsafe extern "system" fn GetPrinterDriverDirectoryW(pName: PWSTR, pEnvironment: PWSTR, Level: DWORD, pDriverDirectory: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD) -> WINBOOL {

    let mut bReturnValue: BOOL;
    let mut dwErrorCode: DWORD = ERROR_INVALID_PRINTER_NAME;
    let mut pEntry: PLIST_ENTRY;
    let mut pPrintProvider: PSPOOLSS_PRINT_PROVIDER;

    if cbBuf != 0 && pDriverDirectory.is_null() {
        SetLastError(ERROR_INVALID_USER_BUFFER);
        return FALSE as WINBOOL;
    }

    'Cleanup: {
        // Loop through all Print Providers.
        pEntry = (*core::ptr::addr_of_mut!(PrintProviderList)).Flink;
        while pEntry != core::ptr::addr_of_mut!(PrintProviderList) {
            pPrintProvider = CONTAINING_RECORD!(pEntry, SPOOLSS_PRINT_PROVIDER, Entry) as PSPOOLSS_PRINT_PROVIDER;

            bReturnValue = (*pPrintProvider)
                .PrintProvider
                .fpGetPrinterDriverDirectory
                .unwrap_unchecked()(
                pName as LPWSTR,
                pEnvironment as LPWSTR,
                Level,
                pDriverDirectory as LPBYTE,
                cbBuf,
                pcbNeeded as LPDWORD,
            );

            if bReturnValue == ROUTER_SUCCESS as BOOL {
                dwErrorCode = ERROR_SUCCESS;
                break 'Cleanup;
            } else if bReturnValue == ROUTER_STOP_ROUTING as BOOL {
                if (*__wine_dbch___default).flags as core::ffi::c_int
                    & (1 as core::ffi::c_int) << __WINE_DBCL_ERR as core::ffi::c_int
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"win32ss/printing/base/spoolss/printerdrivers.c\0"
                            as *const u8 as *const core::ffi::c_char,
                        b"GetPrinterDriverDirectoryW\0" as *const u8
                            as *const core::ffi::c_char,
                        241 as core::ffi::c_int,
                        b"A Print Provider returned ROUTER_STOP_ROUTING for Printer \"%S\"!\n\0"
                            as *const u8 as *const core::ffi::c_char,
                        pName,
                    );
                }
                dwErrorCode = GetLastError();
                break 'Cleanup;
            }

            pEntry = (*pEntry).Flink;
        }
    }

    // ERROR_INVALID_NAME by the Print Provider is translated to ERROR_INVALID_PRINTER_NAME here, but not in other APIs as far as I know.
    if dwErrorCode == ERROR_INVALID_NAME {
        dwErrorCode = ERROR_INVALID_PRINTER_NAME;
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}
