use crate::localmon::tools::DoesPortExist;
use crate::localmon::tools::GetLPTTransmissionRetryTimeout;
use crate::localmon::tools::GetPortNameWithoutColon;
use crate::localmon::tools::GetTypeFromName;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_1FFDAE7B11CC: [u16; 21] = [
    0x0053u16, 0x0065, 0x0074, 0x0044, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074,
    0x0043, 0x006F, 0x006D, 0x006D, 0x0043, 0x006F, 0x006E, 0x0066, 0x0069, 0x0067,
    0x0000,
];
pub static RUSTCONV_WSTR_23CDAB975C35: [u16; 11] = [
    0x0050u16, 0x006F, 0x0072, 0x0074, 0x0045, 0x0078, 0x0069, 0x0073, 0x0074, 0x0073,
    0x0000,
];
pub static RUSTCONV_WSTR_611F4BFBD191: [u16; 8] = [0x0041u16, 0x0064, 0x0064, 0x0050, 0x006F, 0x0072, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_657B9E92C1BA: [u16; 10] = [0x004Du16, 0x006F, 0x006E, 0x0069, 0x0074, 0x006F, 0x0072, 0x0055, 0x0049, 0x0000];
pub static RUSTCONV_WSTR_7AAE39DC0195: [u16; 28] = [
    0x0047u16, 0x0065, 0x0074, 0x0054, 0x0072, 0x0061, 0x006E, 0x0073, 0x006D, 0x0069,
    0x0073, 0x0073, 0x0069, 0x006F, 0x006E, 0x0052, 0x0065, 0x0074, 0x0072, 0x0079,
    0x0054, 0x0069, 0x006D, 0x0065, 0x006F, 0x0075, 0x0074, 0x0000,
];
pub static RUSTCONV_WSTR_7AF58FDC50E8: [u16; 51] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054, 0x005C,
    0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072,
    0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0050, 0x006F, 0x0072, 0x0074, 0x0073,
    0x0000,
];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_9A80C6E935E1: [u16; 11] = [
    0x0044u16, 0x0065, 0x006C, 0x0065, 0x0074, 0x0065, 0x0050, 0x006F, 0x0072, 0x0074,
    0x0000,
];
pub static RUSTCONV_WSTR_B862926D575C: [u16; 53] = [
    0x0053u16, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054, 0x005C,
    0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072,
    0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F,
    0x0077, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_BC56FDFF67BB: [u16; 25] = [
    0x0054u16, 0x0072, 0x0061, 0x006E, 0x0073, 0x006D, 0x0069, 0x0073, 0x0073, 0x0069,
    0x006F, 0x006E, 0x0052, 0x0065, 0x0074, 0x0072, 0x0079, 0x0054, 0x0069, 0x006D,
    0x0065, 0x006F, 0x0075, 0x0074, 0x0000,
];
pub static RUSTCONV_WSTR_DC0356278511: [u16; 26] = [
    0x0043u16, 0x006F, 0x006E, 0x0066, 0x0069, 0x0067, 0x0075, 0x0072, 0x0065, 0x004C,
    0x0050, 0x0054, 0x0050, 0x006F, 0x0072, 0x0074, 0x0043, 0x006F, 0x006D, 0x006D,
    0x0061, 0x006E, 0x0064, 0x004F, 0x004B, 0x0000,
];
pub static RUSTCONV_WSTR_E6050C7B1ADC: [u16; 21] = [
    0x0047u16, 0x0065, 0x0074, 0x0044, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074,
    0x0043, 0x006F, 0x006D, 0x006D, 0x0043, 0x006F, 0x006E, 0x0066, 0x0069, 0x0067,
    0x0000,
];
pub static RUSTCONV_WSTR_EBA07B1C2143: [u16; 12] = [
    0x0050u16, 0x006F, 0x0072, 0x0074, 0x0049, 0x0073, 0x0056, 0x0061, 0x006C, 0x0069,
    0x0064, 0x0000,
];

use core::ffi::c_void;

unsafe extern "C" fn _HandleAddPort(
    pXcv: PLOCALMON_XCV,
    pInputData: PBYTE,
    pcbOutputNeeded: PDWORD,
) -> DWORD {
    let mut res: DWORD = 0;
    let mut cbPortName: DWORD = 0;
    let mut hroot: HKEY = core::ptr::null_mut();
    let mut hToken: HKEY = core::ptr::null_mut();
    let mut pPort: PLOCALMON_PORT = core::ptr::null_mut();
    let pLocalmon: PLOCALMON_HANDLE = (*pXcv).pLocalmon;
    let PortName: PWSTR = pInputData as PWSTR;

    'cleanup: {
        if pLocalmon.is_null() {
            res = ERROR_INVALID_PARAMETER as DWORD;
            break 'cleanup;
        }

        // This action can only happen at SERVER_ACCESS_ADMINISTER access level.
        if ((*pXcv).GrantedAccess & (SERVER_ACCESS_ADMINISTER as ACCESS_MASK)) == 0 {
            res = ERROR_ACCESS_DENIED as DWORD;
            break 'cleanup;
        }

        // Switch to the SYSTEM context for modifying the registry.
        hToken = RevertToPrinterSelf() as HKEY;
        if hToken.is_null() {
            res = GetLastError();
            break 'cleanup;
        }

        res = RegOpenKeyW(
            HKEY_LOCAL_MACHINE,
            RUSTCONV_WSTR_7AF58FDC50E8.as_ptr(),
            &raw mut hroot,
        ) as DWORD;
        if res == ERROR_SUCCESS as DWORD {
            if DoesPortExist(PortName as PCWSTR) != 0 {
                RegCloseKey(hroot);
                res = ERROR_ALREADY_EXISTS as DWORD;
                break 'cleanup;
            }

            cbPortName = ((wcslen(PortName as *const WCHAR) as usize
                + 1 as usize)
                * core::mem::size_of::<WCHAR>()) as DWORD;

            // Create a new LOCALMON_PORT structure for it.
            pPort = DllAllocSplMem(
                (core::mem::size_of::<LOCALMON_PORT>() + cbPortName as usize) as DWORD,
            ) as PLOCALMON_PORT;
            if pPort.is_null() {
                res = ERROR_NOT_ENOUGH_MEMORY as DWORD;
                RegCloseKey(hroot);
                break 'cleanup;
            }
            memset(
                pPort as *mut core::ffi::c_void,
                0 as i32,
                core::mem::size_of::<LOCALMON_PORT>() + cbPortName as usize,
            );

            (*pPort).hFile = INVALID_HANDLE_VALUE;
            (*pPort).pLocalmon = pLocalmon;
            (*pPort).pwszPortName = wcscpy(
                pPort.offset(1 as isize) as *mut WCHAR,
                PortName as *const WCHAR,
            ) as PWSTR;

            // Insert it into the Registry list.
            InsertTailList(
                &raw mut (*pLocalmon).RegistryPorts,
                &raw mut (*pPort).Entry,
            );

            res = RegSetValueExW(
                hroot,
                PortName as LPCWSTR,
                0 as DWORD,
                REG_SZ as DWORD,
                RUSTCONV_WSTR_96A296D224F2.as_ptr() as *const BYTE,
                core::mem::size_of_val(&[0x0000u16]) as DWORD,
            ) as DWORD;
            RegCloseKey(hroot);
        }
    }

    // Cleanup:
    if !hToken.is_null() {
        ImpersonatePrinterClient(hToken as HANDLE);
    }
    SetLastError(res);
    return res;
}

unsafe extern "C" fn _HandleConfigureLPTPortCommandOK(
    pXcv: PLOCALMON_XCV,
    pInputData: PBYTE,
    pcbOutputNeeded: PDWORD,
) -> DWORD {
    let mut cbBuffer: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hToken: HKEY = core::ptr::null_mut();

    'Cleanup: {
        // Sanity checks
        if pXcv.is_null() || pInputData.is_null() || pcbOutputNeeded.is_null() {
            dwErrorCode = ERROR_INVALID_PARAMETER as DWORD;
            break 'Cleanup;
        }

        *pcbOutputNeeded = 0 as DWORD;

        // This action can only happen at SERVER_ACCESS_ADMINISTER access level.
        if ((*pXcv).GrantedAccess & (SERVER_ACCESS_ADMINISTER as ACCESS_MASK)) == 0 {
            dwErrorCode = ERROR_ACCESS_DENIED as DWORD;
            break 'Cleanup;
        }

        // Switch to the SYSTEM context for modifying the registry.
        hToken = RevertToPrinterSelf() as HKEY;
        if hToken.is_null() {
            dwErrorCode = GetLastError() as DWORD;
            break 'Cleanup;
        }

        // Open the key where our value is stored.
        dwErrorCode = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE as HKEY,
            RUSTCONV_WSTR_B862926D575C.as_ptr(),
            0 as DWORD,
            KEY_SET_VALUE as REGSAM,
            &raw mut hKey,
        ) as DWORD;
        if dwErrorCode != ERROR_SUCCESS as DWORD {
            break 'Cleanup;
        }

        // We don't use cbInputData here, because the buffer pInputData could be bigger than the data it contains.
        cbBuffer = (wcslen(pInputData as PWSTR as *const u16) as DWORD)
            .wrapping_add(1 as DWORD)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);

        // Write the value to the registry.
        dwErrorCode = RegSetValueExW(
            hKey,
            RUSTCONV_WSTR_BC56FDFF67BB.as_ptr(),
            0 as DWORD,
            REG_SZ as DWORD,
            pInputData as *const BYTE,
            cbBuffer,
        ) as DWORD;
        if dwErrorCode != ERROR_SUCCESS as DWORD {
            break 'Cleanup;
        }
    }

    // Cleanup:
    if !hKey.is_null() {
        RegCloseKey(hKey);
    }

    if !hToken.is_null() {
        ImpersonatePrinterClient(hToken as HANDLE);
    }

    return dwErrorCode;
}

unsafe extern "C" fn _HandleDeletePort(
    pXcv: PLOCALMON_XCV,
    pInputData: PBYTE,
    pcbOutputNeeded: PDWORD,
) -> DWORD {
    let mut res: DWORD = 0;
    let mut hroot: HKEY = core::ptr::null_mut();
    let mut hToken: HKEY = core::ptr::null_mut();
    let pLocalmon: PLOCALMON_HANDLE = (*pXcv).pLocalmon;
    let mut pPort: PLOCALMON_PORT = core::ptr::null_mut();
    let mut pEntry: PLIST_ENTRY = core::ptr::null_mut();
    let pPortName: PWSTR = pInputData as PWSTR;

    'cleanup: {
        if pLocalmon.is_null() {
            res = ERROR_INVALID_PARAMETER;
            break 'cleanup;
        }

        // This action can only happen at SERVER_ACCESS_ADMINISTER access level.
        if ((*pXcv).GrantedAccess & SERVER_ACCESS_ADMINISTER) == 0 {
            res = ERROR_ACCESS_DENIED;
            break 'cleanup;
        }

        // Switch to the SYSTEM context for modifying the registry.
        hToken = RevertToPrinterSelf() as HKEY;
        if hToken.is_null() {
            res = GetLastError();
            break 'cleanup;
        }

        res = RegOpenKeyW(
            HKEY_LOCAL_MACHINE,
            RUSTCONV_WSTR_7AF58FDC50E8.as_ptr(),
            &raw mut hroot,
        ) as DWORD;
        if res == ERROR_SUCCESS {
            res = RegDeleteValueW(hroot, pPortName as LPCWSTR) as DWORD;

            RegCloseKey(hroot);

            if res == ERROR_SUCCESS {
                EnterCriticalSection(&raw mut (*pLocalmon).Section);

                if !IsListEmpty(&raw mut (*pLocalmon).RegistryPorts) {
                    pEntry = (*pLocalmon).RegistryPorts.Flink;
                    while pEntry != &raw mut (*pLocalmon).RegistryPorts {
                        pPort = CONTAINING_RECORD!(pEntry, LOCALMON_PORT, Entry);

                        if wcscmp(
                            (*pPort).pwszPortName as *const u16,
                            pPortName as *const u16,
                        ) == 0
                        {
                            break;
                        }
                        pEntry = (*pEntry).Flink;
                    }
                }

                LeaveCriticalSection(&raw mut (*pLocalmon).Section);

                if !pPort.is_null() {
                    //
                    EnterCriticalSection(&raw mut (*(*pPort).pLocalmon).Section);
                    RemoveEntryList(&raw mut (*pPort).Entry);
                    LeaveCriticalSection(&raw mut (*(*pPort).pLocalmon).Section);

                    DllFreeSplMem(pPort as PVOID);
                }
            }
            //
        }
    }

    // Cleanup:
    if !hToken.is_null() {
        ImpersonatePrinterClient(hToken as HANDLE);
    }
    SetLastError(res);
    return res;
}

unsafe extern "C" fn _HandleGetDefaultCommConfig(
    mut pInputData: PBYTE,
    mut pOutputData: PBYTE,
    mut cbOutputData: DWORD,
    mut pcbOutputNeeded: PDWORD,
) -> DWORD {
    // Sanity checks
    if pInputData.is_null() || pcbOutputNeeded.is_null() {
        return ERROR_INVALID_PARAMETER as DWORD;
    }

    *pcbOutputNeeded = core::mem::size_of::<COMMCONFIG>() as DWORD;

    // Check if the supplied buffer is large enough.
    if cbOutputData < *pcbOutputNeeded {
        return ERROR_INSUFFICIENT_BUFFER as DWORD;
    }

    // Finally get the port configuration.
    if GetDefaultCommConfigW(
        pInputData as PCWSTR,
        pOutputData as LPCOMMCONFIG,
        pcbOutputNeeded,
    ) == 0
    {
        return GetLastError();
    }

    return ERROR_SUCCESS as DWORD;
}

unsafe extern "C" fn _HandleGetTransmissionRetryTimeout(pOutputData: PBYTE, cbOutputData: DWORD, pcbOutputNeeded: PDWORD) -> DWORD {
    let mut dwTimeout: DWORD = 0;

    // Sanity checks
    if pOutputData.is_null() || pcbOutputNeeded.is_null() {
        return ERROR_INVALID_PARAMETER as DWORD;
    }

    *pcbOutputNeeded = core::mem::size_of::<DWORD>() as DWORD;

    // Check if the supplied buffer is large enough.
    if cbOutputData < *pcbOutputNeeded {
        return ERROR_INSUFFICIENT_BUFFER as DWORD;
    }

    // Retrieve and copy the number.
    dwTimeout = GetLPTTransmissionRetryTimeout();
    CopyMemory!(
        pOutputData as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(dwTimeout) as *const core::ffi::c_void,
        core::mem::size_of::<DWORD>() as usize,
    );
    return ERROR_SUCCESS as DWORD;
}

unsafe extern "C" fn _HandleMonitorUI(
    mut pOutputData: PBYTE,
    mut cbOutputData: DWORD,
    mut pcbOutputNeeded: PDWORD,
) -> DWORD {
    let wszMonitorUI: [WCHAR; 12] = [
    0x004Cu16, 0x006F, 0x0063, 0x0061, 0x006C, 0x0055, 0x0049, 0x002E, 0x0064, 0x006C,
    0x006C, 0x0000,
];
    if pcbOutputNeeded.is_null() {
        return ERROR_INVALID_PARAMETER as DWORD;
    }
    *pcbOutputNeeded = ::core::mem::size_of::<[WCHAR; 12]>() as DWORD;
    if cbOutputData < *pcbOutputNeeded {
        return ERROR_INSUFFICIENT_BUFFER as DWORD;
    }
    if pOutputData.is_null() {
        return ERROR_INVALID_PARAMETER as DWORD;
    }
    CopyMemory!(
        pOutputData as *mut ::core::ffi::c_void,
        &raw const wszMonitorUI as *const WCHAR as *const ::core::ffi::c_void,
        ::core::mem::size_of::<[WCHAR; 12]>() as usize,
    );
    return ERROR_SUCCESS as DWORD;
}

unsafe extern "C" fn _HandlePortExists(
    mut pInputData: PBYTE,
    mut pOutputData: PBYTE,
    mut cbOutputData: DWORD,
    mut pcbOutputNeeded: PDWORD,
) -> DWORD {
    if pInputData.is_null() || pOutputData.is_null() || pcbOutputNeeded.is_null() {
        return ERROR_INVALID_PARAMETER as DWORD;
    }
    *pcbOutputNeeded = ::core::mem::size_of::<BOOL>() as DWORD;
    if cbOutputData < *pcbOutputNeeded {
        return ERROR_INSUFFICIENT_BUFFER as DWORD;
    }
    *(pOutputData as PBOOL) = DoesPortExist(pInputData as PCWSTR);
    return GetLastError();
}

unsafe extern "C" fn _HandlePortIsValid(
    pInputData: PBYTE,
    pOutputData: PBYTE,
    cbOutputData: DWORD,
    pcbOutputNeeded: PDWORD,
) -> DWORD {
    let mut res: DWORD;

    res = GetTypeFromName(pInputData as LPCWSTR);

    /* names, that we have recognized, are valid */
    if res != 0 {
        return ERROR_SUCCESS;
    }

    /* ERROR_ACCESS_DENIED, ERROR_PATH_NOT_FOUND or something else */
    return GetLastError();
}

unsafe extern "C" fn _HandleSetDefaultCommConfig(mut pXcv: PLOCALMON_XCV, mut pInputData: PBYTE, mut pcbOutputNeeded: PDWORD) -> DWORD {
    let mut dwErrorCode: DWORD = 0;
    let mut hToken: HANDLE = core::ptr::null_mut();
    let mut pCommConfig: LPCOMMCONFIG = core::ptr::null_mut();
    let mut pwszPortNameWithoutColon: PWSTR = core::ptr::null_mut();
    if pXcv.is_null()
        || (*pXcv).pwszObject.is_null()
        || *(*pXcv).pwszObject.offset(0 as isize) == 0
        || *(*pXcv).pwszObject.offset(1 as isize) == 0
        || pInputData.is_null()
        || pcbOutputNeeded.is_null()
    {
        dwErrorCode = ERROR_INVALID_PARAMETER as DWORD;
    } else {
        *pcbOutputNeeded = 0 as DWORD;
        if (*pXcv).GrantedAccess & (SERVER_ACCESS_ADMINISTER as ACCESS_MASK) == 0 {
            dwErrorCode = ERROR_ACCESS_DENIED as DWORD;
        } else {
            dwErrorCode = GetPortNameWithoutColon(
                (*pXcv).pwszObject as PCWSTR,
                &raw mut pwszPortNameWithoutColon,
            );
            if !(dwErrorCode != ERROR_SUCCESS as DWORD) {
                hToken = RevertToPrinterSelf();
                if hToken.is_null() {
                    dwErrorCode = GetLastError();
                    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int))
                        != 0
                    {
                        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                        ros_dbg_log(
                            __dbcl,
                            __dbch,
                            b"win32ss/printing/monitors/localmon/xcv.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"_HandleSetDefaultCommConfig\0" as *const u8
                                as *const ::core::ffi::c_char,
                            476 as ::core::ffi::c_int,
                            b"RevertToPrinterSelf failed with error %lu!\n\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            dwErrorCode,
                        );
                    }
                } else {
                    pCommConfig = pInputData as LPCOMMCONFIG;
                    if SetDefaultCommConfigW(
                        pwszPortNameWithoutColon as LPCWSTR,
                        pCommConfig,
                        (*pCommConfig).dwSize,
                    ) == 0
                    {
                        dwErrorCode = GetLastError();
                        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int))
                            != 0
                        {
                            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                            ros_dbg_log(
                                __dbcl_0,
                                __dbch_0,
                                b"win32ss/printing/monitors/localmon/xcv.c\0"
                                    as *const u8 as *const ::core::ffi::c_char,
                                b"_HandleSetDefaultCommConfig\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                485 as ::core::ffi::c_int,
                                b"SetDefaultCommConfigW failed with error %lu!\n\0"
                                    as *const u8 as *const ::core::ffi::c_char,
                                dwErrorCode,
                            );
                        }
                    } else {
                        dwErrorCode = ERROR_SUCCESS as DWORD;
                    }
                }
            }
        }
    }
    if !hToken.is_null() {
        ImpersonatePrinterClient(hToken);
    }
    if !pwszPortNameWithoutColon.is_null() {
        DllFreeSplMem(pwszPortNameWithoutColon as PVOID);
    }
    return dwErrorCode;
}

#[no_mangle]
pub unsafe extern "system" fn LocalmonXcvClosePort(hXcv: HANDLE) -> BOOL {
    let pXcv: PLOCALMON_XCV = hXcv as PLOCALMON_XCV;

    // Sanity checks
    if pXcv.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    // Remove it from the list and free the memory.
    EnterCriticalSection(&raw mut (*(*pXcv).pLocalmon).Section);
    RemoveEntryList(&raw mut (*pXcv).Entry);
    LeaveCriticalSection(&raw mut (*(*pXcv).pLocalmon).Section);
    DllFreeSplMem(pXcv as PVOID);

    SetLastError(ERROR_SUCCESS);
    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn LocalmonXcvDataPort(hXcv: HANDLE, pszDataName: PCWSTR, pInputData: PBYTE, cbInputData: DWORD, pOutputData: PBYTE, cbOutputData: DWORD, pcbOutputNeeded: PDWORD) -> DWORD {
    // Sanity checks
    if pszDataName.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    // Call the appropriate handler for the requested data name.
    if wcscmp(pszDataName, RUSTCONV_WSTR_611F4BFBD191.as_ptr()) == 0 {
        return _HandleAddPort(hXcv as PLOCALMON_XCV, pInputData, pcbOutputNeeded);
    }

    if wcscmp(pszDataName, RUSTCONV_WSTR_DC0356278511.as_ptr()) == 0 {
        return _HandleConfigureLPTPortCommandOK(hXcv as PLOCALMON_XCV, pInputData, pcbOutputNeeded);
    }

    if wcscmp(pszDataName, RUSTCONV_WSTR_9A80C6E935E1.as_ptr()) == 0 {
        return _HandleDeletePort(hXcv as PLOCALMON_XCV, pInputData, pcbOutputNeeded);
    }

    if wcscmp(pszDataName, RUSTCONV_WSTR_E6050C7B1ADC.as_ptr()) == 0 {
        return _HandleGetDefaultCommConfig(pInputData, pOutputData, cbOutputData, pcbOutputNeeded);
    }

    if wcscmp(pszDataName, RUSTCONV_WSTR_7AAE39DC0195.as_ptr()) == 0 {
        return _HandleGetTransmissionRetryTimeout(pOutputData, cbOutputData, pcbOutputNeeded);
    }

    if wcscmp(pszDataName, RUSTCONV_WSTR_657B9E92C1BA.as_ptr()) == 0 {
        return _HandleMonitorUI(pOutputData, cbOutputData, pcbOutputNeeded);
    }

    if wcscmp(pszDataName, RUSTCONV_WSTR_23CDAB975C35.as_ptr()) == 0 {
        return _HandlePortExists(pInputData, pOutputData, cbOutputData, pcbOutputNeeded);
    }

    if wcscmp(pszDataName, RUSTCONV_WSTR_EBA07B1C2143.as_ptr()) == 0 {
        return _HandlePortIsValid(pInputData, pOutputData, cbOutputData, pcbOutputNeeded);
    }

    if wcscmp(pszDataName, RUSTCONV_WSTR_1FFDAE7B11CC.as_ptr()) == 0 {
        return _HandleSetDefaultCommConfig(hXcv as PLOCALMON_XCV, pInputData, pcbOutputNeeded);
    }

    return ERROR_INVALID_PARAMETER;
}

#[no_mangle]
pub unsafe extern "system" fn LocalmonXcvOpenPort(hMonitor: HANDLE, pwszObject: PCWSTR, GrantedAccess: ACCESS_MASK, phXcv: PHANDLE) -> BOOL {
    let mut cbObject: DWORD = 0 as DWORD;
    let mut dwErrorCode: DWORD = 0;
    let mut pLocalmon: PLOCALMON_HANDLE = hMonitor as PLOCALMON_HANDLE;
    let mut pXcv: PLOCALMON_XCV = core::ptr::null_mut::<LOCALMON_XCV>();
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_FIXME as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/printing/monitors/localmon/xcv.c\0"
                as *const u8 as *const i8,
            b"LocalmonXcvOpenPort\0" as *const u8 as *const i8,
            573 as i32,
            b"LocalmonXcvOpenPort(%p, %S, %lu, %p)\n\0" as *const u8 as *const i8,
            hMonitor,
            pwszObject,
            GrantedAccess,
            phXcv,
        );
    }
    'Cleanup: {
        // Sanity checks
        if pLocalmon.is_null() || phXcv.is_null() {
            dwErrorCode = ERROR_INVALID_PARAMETER as DWORD;
            break 'Cleanup;
        }

        if !pwszObject.is_null() {
            cbObject = ((wcslen(pwszObject as *const WCHAR) as usize)
                .wrapping_add(1 as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>()))
                as DWORD;
        }

        // Create a new LOCALMON_XCV structure and fill the relevant fields.
        pXcv = DllAllocSplMem(
            (core::mem::size_of::<LOCALMON_XCV>() as usize).wrapping_add(cbObject as usize)
                as DWORD,
        ) as PLOCALMON_XCV;
        if pXcv.is_null() {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY as DWORD;
            if (*__wine_dbch___default).flags as i32
                & (1 as i32) << __WINE_DBCL_ERR as i32
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"win32ss/printing/monitors/localmon/xcv.c\0"
                        as *const u8 as *const i8,
                    b"LocalmonXcvOpenPort\0" as *const u8 as *const i8,
                    590 as i32,
                    b"DllAllocSplMem failed with error %lu!\n\0" as *const u8 as *const i8,
                    GetLastError(),
                );
            }
            break 'Cleanup;
        }

        (*pXcv).pLocalmon = pLocalmon;
        (*pXcv).GrantedAccess = GrantedAccess;

        if cbObject != 0 {
            (*pXcv).pwszObject = ((pXcv as PBYTE)
                .add(core::mem::size_of::<LOCALMON_XCV>()))
                as PWSTR;
            CopyMemory!(
                (*pXcv).pwszObject as *mut core::ffi::c_void,
                pwszObject as *const core::ffi::c_void,
                cbObject as usize,
            );
        }

        InsertTailList(
            core::ptr::addr_of_mut!((*pLocalmon).XcvHandles),
            core::ptr::addr_of_mut!((*pXcv).Entry),
        );

        // Return it as the Xcv handle.
        *phXcv = pXcv as HANDLE;
        dwErrorCode = ERROR_SUCCESS as DWORD;
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS as DWORD) as BOOL;
}
