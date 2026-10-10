// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_05CF8A9FE74B: [u16; 18] = [
    0x0050u16, 0x0072, 0x0069, 0x006E, 0x0074, 0x0065, 0x0072, 0x0044, 0x0072, 0x0069,
    0x0076, 0x0065, 0x0072, 0x0044, 0x0061, 0x0074, 0x0061, 0x0000,
];
pub static RUSTCONV_WSTR_32CEE5DE951A: [u16; 10] = [0x0050u16, 0x0072, 0x006F, 0x0076, 0x0069, 0x0064, 0x0065, 0x0072, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_BE34F66DC878: [u16; 23] = [
    0x004Eu16, 0x006F, 0x0052, 0x0065, 0x006D, 0x006F, 0x0074, 0x0065, 0x0050, 0x0072,
    0x0069, 0x006E, 0x0074, 0x0065, 0x0072, 0x0044, 0x0072, 0x0069, 0x0076, 0x0065,
    0x0072, 0x0073, 0x0000,
];

use core::ffi::c_void;

extern "C" {
    fn ros_dbg_log( cls: __wine_debug_class, ch: *mut __wine_debug_channel, file: *const core::ffi::c_char, func: *const core::ffi::c_char, line: core::ffi::c_int, fmt: *const core::ffi::c_char, ... );
}
#[no_mangle]
pub unsafe extern "system" fn LocalGetPrinterData(hPrinter: HANDLE, pValueName: PWSTR, pType: PDWORD, pData: PBYTE, nSize: DWORD, pcbNeeded: PDWORD) -> DWORD {
    return LocalGetPrinterDataEx(hPrinter, RUSTCONV_WSTR_05CF8A9FE74B.as_ptr() as PCWSTR, pValueName as PCWSTR, pType, pData, nSize, pcbNeeded);
}

unsafe extern "C" fn _MakePrinterSubKey(pPrinterHandle: PLOCAL_PRINTER_HANDLE, pKeyName: PCWSTR, ppwszSubKey: *mut PWSTR) -> DWORD {
    let wszBackslash: [WCHAR; 2] = [0x005Cu16, 0x0000];

    let mut cbSubKey: size_t = 0;
    let mut p: PWSTR = core::ptr::null_mut();

    // Sanity check
    if pKeyName.is_null() || *pKeyName == 0 {
        return ERROR_INVALID_PARAMETER as DWORD;
    }

    // Allocate a buffer for the subkey "PrinterName\KeyName".
    cbSubKey = (wcslen((*(*pPrinterHandle).pPrinter).pwszPrinterName as *const WCHAR) as size_t
        + 1 as size_t
        + wcslen(pKeyName as *const WCHAR) as size_t
        + 1 as size_t)
        * core::mem::size_of::<WCHAR>();
    *ppwszSubKey = DllAllocSplMem(cbSubKey as DWORD) as PWSTR;
    if (*ppwszSubKey).is_null() {
        return ERROR_NOT_ENOUGH_MEMORY as DWORD;
    }

    // Concatenate the subkey.
    p = *ppwszSubKey;
    StringCbCopyExW(
        p as STRSAFE_LPWSTR,
        cbSubKey,
        (*(*pPrinterHandle).pPrinter).pwszPrinterName as STRSAFE_LPCWSTR,
        core::ptr::addr_of_mut!(p),
        core::ptr::addr_of_mut!(cbSubKey),
        0 as STRSAFE_DWORD,
    );
    StringCbCopyExW(
        p as STRSAFE_LPWSTR,
        cbSubKey,
        core::ptr::addr_of!(wszBackslash) as STRSAFE_LPCWSTR,
        core::ptr::addr_of_mut!(p),
        core::ptr::addr_of_mut!(cbSubKey),
        0 as STRSAFE_DWORD,
    );
    StringCbCopyExW(
        p as STRSAFE_LPWSTR,
        cbSubKey,
        pKeyName as STRSAFE_LPCWSTR,
        core::ptr::addr_of_mut!(p),
        core::ptr::addr_of_mut!(cbSubKey),
        0 as STRSAFE_DWORD,
    );

    return ERROR_SUCCESS as DWORD;
}

unsafe extern "C" fn _LocalGetPrinterHandleData(pPrinterHandle: PLOCAL_PRINTER_HANDLE, pKeyName: PCWSTR, pValueName: PCWSTR, pType: PDWORD, pData: PBYTE, nSize: DWORD, pcbNeeded: PDWORD) -> DWORD {
    let mut dwErrorCode: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut pwszSubKey: PWSTR = core::ptr::null_mut();

    'Cleanup: {
        dwErrorCode = _MakePrinterSubKey(pPrinterHandle, pKeyName, core::ptr::addr_of_mut!(pwszSubKey));
        if dwErrorCode != ERROR_SUCCESS {
            break 'Cleanup;
        }

        // Open the subkey.
        dwErrorCode = RegOpenKeyExW(
            hPrintersKey,
            pwszSubKey,
            0,
            KEY_READ,
            core::ptr::addr_of_mut!(hKey),
        ) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            if (*__wine_dbch___default).flags as i32
                & (1i32) << __WINE_DBCL_ERR as i32
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"win32ss/printing/providers/localspl/printerdata.c\0"
                        as *const u8 as *const i8,
                    b"_LocalGetPrinterHandleData\0" as *const u8 as *const i8,
                    63i32,
                    b"RegOpenKeyExW failed for \"%S\" with error %lu!\n\0" as *const u8 as *const i8,
                    pwszSubKey,
                    dwErrorCode,
                );
            }
            break 'Cleanup;
        }

        // Query the desired value.
        *pcbNeeded = nSize;
        dwErrorCode = RegQueryValueExW(
            hKey,
            pValueName,
            core::ptr::null_mut(),
            pType,
            pData,
            pcbNeeded,
        ) as DWORD;
    }

    if !hKey.is_null() {
        RegCloseKey(hKey);
    }

    if !pwszSubKey.is_null() {
        DllFreeSplMem(pwszSubKey as PVOID);
    }

    return dwErrorCode;
}

unsafe extern "C" fn _LocalGetPrintServerHandleData(pValueName: PCWSTR, pType: PDWORD, pData: PBYTE, nSize: DWORD, pcbNeeded: PDWORD) -> DWORD {
    let mut dwErrorCode: DWORD = 0;

    if _wcsicmp(pValueName, SPLREG_DEFAULT_SPOOL_DIRECTORY) == 0 ||
        _wcsicmp(pValueName, SPLREG_PORT_THREAD_PRIORITY) == 0 ||
        _wcsicmp(pValueName, SPLREG_SCHEDULER_THREAD_PRIORITY) == 0 ||
        _wcsicmp(pValueName, SPLREG_BEEP_ENABLED) == 0 ||
        _wcsicmp(pValueName, SPLREG_ALLOW_USER_MANAGEFORMS) == 0
    {
        *pcbNeeded = nSize;
        return RegQueryValueExW(hPrintersKey, pValueName, core::ptr::null_mut(), pType, pData, pcbNeeded) as DWORD;
    }
    else if _wcsicmp(pValueName, SPLREG_PORT_THREAD_PRIORITY_DEFAULT) == 0 ||
        _wcsicmp(pValueName, SPLREG_SCHEDULER_THREAD_PRIORITY_DEFAULT) == 0
    {
        // Store a DWORD value as REG_NONE.
        *pType = REG_NONE as DWORD;
        *pcbNeeded = core::mem::size_of::<DWORD>() as DWORD;
        if nSize < *pcbNeeded {
            return ERROR_MORE_DATA as DWORD;
        }

        // Apparently, these values don't serve a purpose anymore.
        *(pData as PDWORD) = 0 as DWORD;
        return ERROR_SUCCESS as DWORD;
    }
    else if _wcsicmp(pValueName, SPLREG_NET_POPUP) == 0 ||
        _wcsicmp(pValueName, SPLREG_RETRY_POPUP) == 0 ||
        _wcsicmp(pValueName, SPLREG_NET_POPUP_TO_COMPUTER) == 0 ||
        _wcsicmp(pValueName, SPLREG_EVENT_LOG) == 0 ||
        _wcsicmp(pValueName, SPLREG_RESTART_JOB_ON_POOL_ERROR) == 0 ||
        _wcsicmp(pValueName, SPLREG_RESTART_JOB_ON_POOL_ENABLED) == 0
    {
        let mut hKey: HKEY = core::ptr::null_mut();

        dwErrorCode = RegOpenKeyExW(hPrintKey, RUSTCONV_WSTR_32CEE5DE951A.as_ptr(), 0, KEY_READ, core::ptr::addr_of_mut!(hKey)) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            return dwErrorCode;
        }

        *pcbNeeded = nSize;
        dwErrorCode = RegQueryValueExW(hKey, pValueName, core::ptr::null_mut(), pType, pData, pcbNeeded) as DWORD;
        RegCloseKey(hKey);
        return dwErrorCode;
    }
    else if _wcsicmp(pValueName, SPLREG_MAJOR_VERSION) == 0
    {
        // Store a DWORD value as REG_NONE.
        *pType = REG_NONE as DWORD;
        *pcbNeeded = core::mem::size_of::<DWORD>() as DWORD;
        if nSize < *pcbNeeded {
            return ERROR_MORE_DATA as DWORD;
        }

        // Apparently, these values don't serve a purpose anymore.
        *(pData as PDWORD) = dwSpoolerMajorVersion;
        return ERROR_SUCCESS as DWORD;
    }
    else if _wcsicmp(pValueName, SPLREG_MINOR_VERSION) == 0
    {
        // Store a DWORD value as REG_NONE.
        *pType = REG_NONE as DWORD;
        *pcbNeeded = core::mem::size_of::<DWORD>() as DWORD;
        if nSize < *pcbNeeded {
            return ERROR_MORE_DATA as DWORD;
        }

        // Apparently, these values don't serve a purpose anymore.
        *(pData as PDWORD) = dwSpoolerMinorVersion;
        return ERROR_SUCCESS as DWORD;
    }
    else if _wcsicmp(pValueName, SPLREG_ARCHITECTURE) == 0
    {
        // Store a string as REG_NONE with the length of the environment name string.
        *pType = REG_NONE as DWORD;
        *pcbNeeded = cbCurrentEnvironment;
        if nSize < *pcbNeeded {
            return ERROR_MORE_DATA as DWORD;
        }

        // Copy the environment name as the output value for SPLREG_ARCHITECTURE.
        CopyMemory!(pData as *mut core::ffi::c_void, core::ptr::addr_of!(wszCurrentEnvironment) as *const core::ffi::c_void, cbCurrentEnvironment as usize);
        return ERROR_SUCCESS as DWORD;
    }
    else if _wcsicmp(pValueName, SPLREG_OS_VERSION) == 0
    {
        let mut pInfo: POSVERSIONINFOW = pData as POSVERSIONINFOW;

        // Store the OSVERSIONINFOW structure as REG_NONE.
        *pType = REG_NONE as DWORD;
        *pcbNeeded = core::mem::size_of::<OSVERSIONINFOW>() as DWORD;
        if nSize < *pcbNeeded {
            return ERROR_MORE_DATA as DWORD;
        }

        // Return OS version information.
        (*pInfo).dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFOW>() as DWORD;
        GetVersionExW(pInfo);
        return ERROR_SUCCESS as DWORD;
    }
    else if _wcsicmp(pValueName, SPLREG_OS_VERSIONEX) == 0
    {
        let mut pInfo: POSVERSIONINFOEXW = pData as POSVERSIONINFOEXW;

        // Store the OSVERSIONINFOEXW structure as REG_NONE.
        *pType = REG_NONE as DWORD;
        *pcbNeeded = core::mem::size_of::<OSVERSIONINFOEXW>() as DWORD;
        if nSize < *pcbNeeded {
            return ERROR_MORE_DATA as DWORD;
        }

        // Return extended OS version information.
        (*pInfo).dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFOEXW>() as DWORD;
        GetVersionExW(pInfo as POSVERSIONINFOW);
        return ERROR_SUCCESS as DWORD;
    }
    else if _wcsicmp(pValueName, SPLREG_DS_PRESENT) == 0
    {
        let mut pInfo: PDSROLE_PRIMARY_DOMAIN_INFO_BASIC = core::ptr::null_mut();

        // We want to store a REG_DWORD value.
        *pType = REG_DWORD as DWORD;
        *pcbNeeded = core::mem::size_of::<DWORD>() as DWORD;
        if nSize < *pcbNeeded {
            return ERROR_MORE_DATA as DWORD;
        }

        // Get information about the domain membership of this computer.
        dwErrorCode = DsRoleGetPrimaryDomainInformation(core::ptr::null_mut(), DsRolePrimaryDomainInfoBasic, core::ptr::addr_of_mut!(pInfo) as *mut PBYTE) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            return dwErrorCode;
        }

        // Return whether this computer is a workstation or server inside a domain.
        *(pData as PDWORD) = ((*pInfo).MachineRole == DsRole_RoleMemberWorkstation || (*pInfo).MachineRole == DsRole_RoleMemberServer) as DWORD;
        DsRoleFreeMemory(pInfo as *mut core::ffi::c_void);
        return ERROR_SUCCESS as DWORD;
    }
    else if _wcsicmp(pValueName, SPLREG_DS_PRESENT_FOR_USER) == 0
    {
        let mut cch: DWORD;
        let mut p: PWSTR;
        let mut wszComputerName: [WCHAR; (MAX_COMPUTERNAME_LENGTH + 1) as usize] = [0; (MAX_COMPUTERNAME_LENGTH + 1) as usize];
        let mut wszUserSam: [WCHAR; (UNLEN + 1) as usize] = [0; (UNLEN + 1) as usize];

        // We want to store a REG_DWORD value.
        *pType = REG_DWORD as DWORD;
        *pcbNeeded = core::mem::size_of::<DWORD>() as DWORD;
        if nSize < *pcbNeeded {
            return ERROR_MORE_DATA as DWORD;
        }

        // Get the local Computer Name.
        cch = (MAX_COMPUTERNAME_LENGTH + 1) as DWORD;
        if GetComputerNameW(core::ptr::addr_of_mut!(wszComputerName) as *mut WCHAR, core::ptr::addr_of_mut!(cch)) == 0
        {
            dwErrorCode = GetLastError();
            return dwErrorCode;
        }

        // Get the User Name in the SAM format.
        // This could either be:
        //     COMPUTERNAME\User
        //     DOMAINNAME\User
        cch = (UNLEN + 1) as DWORD;
        if GetUserNameExW(NameSamCompatible, core::ptr::addr_of_mut!(wszUserSam) as *mut WCHAR, core::ptr::addr_of_mut!(cch)) == 0
        {
            dwErrorCode = GetLastError();
            return dwErrorCode;
        }

        // Terminate the SAM-formatted User Name at the backslash.
        p = wcschr(core::ptr::addr_of_mut!(wszUserSam) as *mut WCHAR, 0x005Cu16) as PWSTR;
        *p = 0 as WCHAR;

        // Compare it with the Computer Name.
        // If they differ, this User is part of a domain.
        *(pData as PDWORD) = (wcscmp(core::ptr::addr_of_mut!(wszUserSam) as *mut WCHAR, core::ptr::addr_of_mut!(wszComputerName) as *mut WCHAR) != 0) as DWORD;
        return ERROR_SUCCESS as DWORD;
    }
    else if _wcsicmp(pValueName, SPLREG_REMOTE_FAX) == 0
    {
        // Store a DWORD value as REG_NONE.
        *pType = REG_NONE as DWORD;
        *pcbNeeded = core::mem::size_of::<DWORD>() as DWORD;
        if nSize < *pcbNeeded {
            return ERROR_MORE_DATA as DWORD;
        }

        // TODO: We don't support any fax service yet, but let's return the same value as Windows Server 2003 here.
        *(pData as PDWORD) = 1 as DWORD;
        return ERROR_SUCCESS as DWORD;
    }
    else if _wcsicmp(pValueName, SPLREG_DNS_MACHINE_NAME) == 0
    {
        let mut cchDnsName: DWORD = 0 as DWORD;

        // Get the length of the fully-qualified computer DNS name.
        GetComputerNameExW(ComputerNameDnsFullyQualified, core::ptr::null_mut::<WCHAR>(), core::ptr::addr_of_mut!(cchDnsName));
        dwErrorCode = GetLastError();
        if dwErrorCode != ERROR_MORE_DATA {
            return dwErrorCode;
        }

        // Check if our supplied buffer is large enough.
        *pType = REG_SZ as DWORD;
        *pcbNeeded = cchDnsName.wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
        if nSize < *pcbNeeded {
            return ERROR_MORE_DATA as DWORD;
        }

        // Get the actual DNS name.
        if GetComputerNameExW(ComputerNameDnsFullyQualified, pData as PWSTR, core::ptr::addr_of_mut!(cchDnsName)) == 0
        {
            dwErrorCode = GetLastError();
            return dwErrorCode;
        }

        // Lowercase the output just like Windows does.
        _wcslwr(pData as PWSTR);
        return ERROR_SUCCESS as DWORD;
    }
    else
    {
        // For all other, unknown settings, we just return ERROR_INVALID_PARAMETER.
        // That also includes SPLREG_WEBSHAREMGMT, which is supported in Windows Server 2003 according to the documentation,
        // but is actually not!
        return ERROR_INVALID_PARAMETER as DWORD;
    }
}

#[no_mangle]
pub unsafe extern "system" fn LocalGetPrinterDataEx(hPrinter: HANDLE, pKeyName: PCWSTR, pValueName: PCWSTR, mut pType: PDWORD, mut pData: PBYTE, nSize: DWORD, pcbNeeded: PDWORD) -> DWORD {
    let mut Temp: BYTE = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut dwTemp: DWORD = 0;
    let mut pHandle: PLOCAL_HANDLE = hPrinter as PLOCAL_HANDLE;
    if pType.is_null() {
        pType = core::ptr::addr_of_mut!(dwTemp) as PDWORD;
    }
    if pData.is_null() {
        pData = core::ptr::addr_of_mut!(Temp) as PBYTE;
    }
    if pHandle.is_null() {
        dwErrorCode = ERROR_INVALID_HANDLE as DWORD;
    } else if pcbNeeded.is_null() {
        dwErrorCode = ERROR_INVALID_PARAMETER as DWORD;
    } else if (*pHandle).HandleType as core::ffi::c_uint == HandleType_Printer as core::ffi::c_int as core::ffi::c_uint {
        dwErrorCode = _LocalGetPrinterHandleData((*pHandle).pSpecificHandle as PLOCAL_PRINTER_HANDLE, pKeyName, pValueName, pType, pData, nSize, pcbNeeded);
    } else if (*pHandle).HandleType as core::ffi::c_uint == HandleType_PrintServer as core::ffi::c_int as core::ffi::c_uint {
        dwErrorCode = _LocalGetPrintServerHandleData(pValueName, pType, pData, nSize, pcbNeeded);
    } else {
        dwErrorCode = ERROR_INVALID_HANDLE as DWORD;
    }
    SetLastError(dwErrorCode);
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "system" fn LocalSetPrinterData(hPrinter: HANDLE, pValueName: PWSTR, Type: DWORD, pData: PBYTE, cbData: DWORD) -> DWORD {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/printing/providers/localspl/printerdata.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LocalSetPrinterData\0" as *const u8 as *const ::core::ffi::c_char,
            363 as ::core::ffi::c_int,
            b"LocalSetPrinterData(%p, %S, %lu, %p, %lu)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hPrinter,
            pValueName,
            Type,
            pData,
            cbData,
        );
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_WARN;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"win32ss/printing/providers/localspl/printerdata.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LocalSetPrinterData\0" as *const u8 as *const ::core::ffi::c_char,
            367 as ::core::ffi::c_int,
            b"This function should never be called!\n\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    }
    return LocalSetPrinterDataEx(
        hPrinter,
        RUSTCONV_WSTR_05CF8A9FE74B.as_ptr() as PCWSTR,
        pValueName as PCWSTR,
        Type,
        pData,
        cbData,
    );
}

unsafe extern "C" fn _LocalSetPrinterHandleData(mut pPrinterHandle: PLOCAL_PRINTER_HANDLE, mut pKeyName: PCWSTR, mut pValueName: PCWSTR, mut Type: DWORD, mut pData: PBYTE, mut cbData: DWORD) -> DWORD {
    let mut dwErrorCode: DWORD = 0;
    let mut hKey: HKEY = ::core::ptr::null_mut::<HKEY__>();
    let mut pwszSubKey: PWSTR = ::core::ptr::null_mut::<WCHAR>();
    'Cleanup: {
        dwErrorCode = _MakePrinterSubKey(pPrinterHandle, pKeyName, &raw mut pwszSubKey);
        if dwErrorCode != ERROR_SUCCESS as DWORD {
            break 'Cleanup;
        }

        // Open the subkey.
        dwErrorCode = RegOpenKeyExW(
            hPrintersKey,
            pwszSubKey as LPCWSTR,
            0 as DWORD,
            KEY_SET_VALUE as REGSAM,
            &raw mut hKey,
        ) as DWORD;
        if dwErrorCode != ERROR_SUCCESS as DWORD {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"win32ss/printing/providers/localspl/printerdata.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"_LocalSetPrinterHandleData\0" as *const u8
                        as *const ::core::ffi::c_char,
                    386 as ::core::ffi::c_int,
                    b"RegOpenKeyExW failed for \"%S\" with error %lu!\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    pwszSubKey,
                    dwErrorCode,
                );
            }
            break 'Cleanup;
        }

        // Set the value.
        dwErrorCode = RegSetValueExW(
            hKey,
            pValueName as LPCWSTR,
            0 as DWORD,
            Type,
            pData as *const BYTE,
            cbData,
        ) as DWORD;
    }

    if !hKey.is_null() {
        RegCloseKey(hKey);
    }

    if !pwszSubKey.is_null() {
        DllFreeSplMem(pwszSubKey as PVOID);
    }

    return dwErrorCode;
}

unsafe extern "C" fn _LocalSetPrintServerHandleData(pValueName: PCWSTR, Type: DWORD, pData: PBYTE, cbData: DWORD) -> DWORD {
    let mut dwErrorCode: DWORD = 0;

    if _wcsicmp(pValueName, SPLREG_DEFAULT_SPOOL_DIRECTORY) == 0 ||
        _wcsicmp(pValueName, SPLREG_PORT_THREAD_PRIORITY) == 0 ||
        _wcsicmp(pValueName, SPLREG_SCHEDULER_THREAD_PRIORITY) == 0 ||
        _wcsicmp(pValueName, SPLREG_BEEP_ENABLED) == 0 ||
        _wcsicmp(pValueName, SPLREG_ALLOW_USER_MANAGEFORMS) == 0
    {
        return RegSetValueExW(hPrintersKey, pValueName as LPCWSTR, 0, Type, pData as *const BYTE, cbData) as DWORD;
    } else if _wcsicmp(pValueName, SPLREG_NET_POPUP) == 0 ||
        _wcsicmp(pValueName, SPLREG_RETRY_POPUP) == 0 ||
        _wcsicmp(pValueName, SPLREG_NET_POPUP_TO_COMPUTER) == 0 ||
        _wcsicmp(pValueName, SPLREG_EVENT_LOG) == 0 ||
        _wcsicmp(pValueName, SPLREG_RESTART_JOB_ON_POOL_ERROR) == 0 ||
        _wcsicmp(pValueName, SPLREG_RESTART_JOB_ON_POOL_ENABLED) == 0 ||
        _wcsicmp(pValueName, RUSTCONV_WSTR_BE34F66DC878.as_ptr()) == 0
    {
        let mut hKey: HKEY = core::ptr::null_mut();

        dwErrorCode = RegOpenKeyExW(hPrintKey, RUSTCONV_WSTR_32CEE5DE951A.as_ptr(), 0, KEY_SET_VALUE, core::ptr::addr_of_mut!(hKey)) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            if (*__wine_dbch___default).flags as core::ffi::c_int
                & (1 as core::ffi::c_int) << __WINE_DBCL_ERR as core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"win32ss/printing/providers/localspl/printerdata.c\0"
                        .as_ptr() as *const core::ffi::c_char,
                    b"_LocalSetPrintServerHandleData\0".as_ptr() as *const core::ffi::c_char,
                    429 as core::ffi::c_int,
                    b"RegOpenKeyExW failed for \"Providers\" with error %lu!\n\0"
                        .as_ptr() as *const core::ffi::c_char,
                    dwErrorCode,
                );
            }
            return dwErrorCode;
        }

        dwErrorCode = RegSetValueExW(hKey, pValueName as LPCWSTR, 0, Type, pData as *const BYTE, cbData) as DWORD;
        RegCloseKey(hKey);
        return dwErrorCode;
    } else if _wcsicmp(pValueName, SPLREG_WEBSHAREMGMT) == 0 {
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_WARN as core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"win32ss/printing/providers/localspl/printerdata.c\0"
                    .as_ptr() as *const core::ffi::c_char,
                b"_LocalSetPrintServerHandleData\0".as_ptr() as *const core::ffi::c_char,
                439 as core::ffi::c_int,
                b"Attempting to set WebShareMgmt, which is based on IIS and therefore not supported. Returning fake success!\n\0"
                    .as_ptr() as *const core::ffi::c_char,
            );
        }
        return ERROR_SUCCESS;
    } else {
        // For all other, unknown settings, we just return ERROR_INVALID_PARAMETER.
        return ERROR_INVALID_PARAMETER;
    }
}

#[no_mangle]
pub unsafe extern "system" fn LocalSetPrinterDataEx(hPrinter: HANDLE, pKeyName: LPCWSTR, pValueName: LPCWSTR, Type: DWORD, pData: LPBYTE, cbData: DWORD) -> DWORD {
    let mut dwErrorCode: DWORD;
    let pHandle: PLOCAL_HANDLE = hPrinter as PLOCAL_HANDLE;

    if pHandle.is_null() {
        dwErrorCode = ERROR_INVALID_HANDLE as DWORD;
    } else if (*pHandle).HandleType as ::core::ffi::c_uint
        == HandleType_Printer as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        dwErrorCode = _LocalSetPrinterHandleData(
            (*pHandle).pSpecificHandle as PLOCAL_PRINTER_HANDLE,
            pKeyName,
            pValueName,
            Type,
            pData,
            cbData,
        );
    } else if (*pHandle).HandleType as ::core::ffi::c_uint
        == HandleType_PrintServer as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        dwErrorCode = _LocalSetPrintServerHandleData(pValueName, Type, pData, cbData);
    } else {
        dwErrorCode = ERROR_INVALID_HANDLE as DWORD;
    }

    SetLastError(dwErrorCode);
    return dwErrorCode;
}
