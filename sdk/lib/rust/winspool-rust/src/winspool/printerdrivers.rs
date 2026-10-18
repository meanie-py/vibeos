use crate::winspool::marshalling::MarshallUpStructure;
use ntdll_sys::seh::rpc_try;
use crate::winspool::marshalling::MarshallUpStructuresArray;
use crate::winspool::utils::UnicodeToAnsiInPlace;
use crate::winspool::utils::UnicodeToAnsiZZInPlace;
use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn SetLastError(dwErrCode: DWORD);
    fn GetProcessHeap() -> HANDLE;
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: usize) -> LPVOID;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn MultiByteToWideChar(CodePage: UINT, dwFlags: DWORD, lpMultiByteStr: LPCSTR, cbMultiByte: i32, lpWideCharStr: LPWSTR, cchWideChar: i32) -> i32;
}

// ---- file-scope preamble (translated) ----
unsafe extern "C" fn multi_sz_lenA(str: *const i8) -> i32 {
    let mut ptr: *const i8 = str;
    if str.is_null() { return 0; }
    loop {
        ptr = ptr.offset((lstrlenA(ptr) + 1) as isize);
        if !(*ptr != 0) { break; }
    }
    return (ptr.offset_from(str) as i32) + 1;
}

unsafe extern "C" fn multi_sz_lenW(str: *const u16) -> i32 {
    let mut ptr: *const u16 = str;
    if str.is_null() { return 0; }
    loop {
        ptr = ptr.offset((lstrlenW(ptr) + 1) as isize);
        if !(*ptr != 0) { break; }
    }

    return (ptr.offset_from(str) as i32 + 1);
}

#[no_mangle]
pub unsafe extern "system" fn AddPrinterDriverA(pName: PSTR, Level: u32, pDriverInfo: PBYTE) -> i32 {
    return AddPrinterDriverExA(pName, Level, pDriverInfo, APD_COPY_NEW_FILES);
}

#[no_mangle]
pub unsafe extern "system" fn AddPrinterDriverExA(pName: PSTR, Level: DWORD, pDriverInfo: PBYTE, dwFileCopyFlags: DWORD) -> BOOL {
    let mut pdiA: PDRIVER_INFO_8A;
    let mut diW: DRIVER_INFO_8W = core::mem::zeroed();
    let mut nameW: LPWSTR = core::ptr::null_mut();
    let mut lenA: DWORD = 0;
    let mut len: DWORD = 0;
    let mut res: BOOL = FALSE as BOOL;

    pdiA = pDriverInfo as PDRIVER_INFO_8A;
    ZeroMemory!(core::ptr::addr_of_mut!(diW) as LPVOID, core::mem::size_of::<DRIVER_INFO_8W>());

    if Level < 2 || Level == 5 || Level == 7 || Level > 8
    {
        SetLastError(ERROR_INVALID_LEVEL);
        return FALSE as BOOL;
    }

    if pdiA.is_null()
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    /* convert servername to unicode */
    if !pName.is_null()
    {
        len = MultiByteToWideChar(CP_ACP, 0, pName, -1, core::ptr::null_mut(), 0) as DWORD;
        nameW = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, pName, -1, nameW, len as i32);
    }

    /* common fields */
    diW.cVersion = (*pdiA).cVersion;

    if !(*pdiA).pName.is_null()
    {
        len = MultiByteToWideChar(CP_ACP, 0, (*pdiA).pName, -1, core::ptr::null_mut(), 0) as DWORD;
        diW.pName = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as PWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*pdiA).pName, -1, diW.pName, len as i32);
    }

    if !(*pdiA).pEnvironment.is_null()
    {
        len = MultiByteToWideChar(CP_ACP, 0, (*pdiA).pEnvironment, -1, core::ptr::null_mut(), 0) as DWORD;
        diW.pEnvironment = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as PWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*pdiA).pEnvironment, -1, diW.pEnvironment, len as i32);
    }

    if !(*pdiA).pDriverPath.is_null()
    {
        len = MultiByteToWideChar(CP_ACP, 0, (*pdiA).pDriverPath, -1, core::ptr::null_mut(), 0) as DWORD;
        diW.pDriverPath = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as PWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*pdiA).pDriverPath, -1, diW.pDriverPath, len as i32);
    }

    if !(*pdiA).pDataFile.is_null()
    {
        len = MultiByteToWideChar(CP_ACP, 0, (*pdiA).pDataFile, -1, core::ptr::null_mut(), 0) as DWORD;
        diW.pDataFile = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as PWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*pdiA).pDataFile, -1, diW.pDataFile, len as i32);
    }

    if !(*pdiA).pConfigFile.is_null()
    {
        len = MultiByteToWideChar(CP_ACP, 0, (*pdiA).pConfigFile, -1, core::ptr::null_mut(), 0) as DWORD;
        diW.pConfigFile = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as PWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*pdiA).pConfigFile, -1, diW.pConfigFile, len as i32);
    }

    if (Level > 2) && !(*pdiA).pHelpFile.is_null()
    {
        len = MultiByteToWideChar(CP_ACP, 0, (*pdiA).pHelpFile, -1, core::ptr::null_mut(), 0) as DWORD;
        diW.pHelpFile = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as PWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*pdiA).pHelpFile, -1, diW.pHelpFile, len as i32);
    }

    if (Level > 2) && !(*pdiA).pDependentFiles.is_null()
    {
        lenA = multi_sz_lenA((*pdiA).pDependentFiles) as DWORD;
        len = MultiByteToWideChar(CP_ACP, 0, (*pdiA).pDependentFiles, lenA as i32, core::ptr::null_mut(), 0) as DWORD;
        diW.pDependentFiles = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as PWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*pdiA).pDependentFiles, lenA as i32, diW.pDependentFiles, len as i32);
    }

    if (Level > 2) && !(*pdiA).pMonitorName.is_null()
    {
        len = MultiByteToWideChar(CP_ACP, 0, (*pdiA).pMonitorName, -1, core::ptr::null_mut(), 0) as DWORD;
        diW.pMonitorName = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as PWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*pdiA).pMonitorName, -1, diW.pMonitorName, len as i32);
    }

    if (Level > 2) && !(*pdiA).pDefaultDataType.is_null()
    {
        len = MultiByteToWideChar(CP_ACP, 0, (*pdiA).pDefaultDataType, -1, core::ptr::null_mut(), 0) as DWORD;
        diW.pDefaultDataType = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as PWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*pdiA).pDefaultDataType, -1, diW.pDefaultDataType, len as i32);
    }

    if (Level > 3) && !(*pdiA).pszzPreviousNames.is_null()
    {
        lenA = multi_sz_lenA((*pdiA).pszzPreviousNames) as DWORD;
        len = MultiByteToWideChar(CP_ACP, 0, (*pdiA).pszzPreviousNames, lenA as i32, core::ptr::null_mut(), 0) as DWORD;
        diW.pszzPreviousNames = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as PWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*pdiA).pszzPreviousNames, lenA as i32, diW.pszzPreviousNames, len as i32);
    }

    if Level > 5
    {
        diW.ftDriverDate = (*pdiA).ftDriverDate;
        diW.dwlDriverVersion = (*pdiA).dwlDriverVersion;
    }

    if (Level > 5) && !(*pdiA).pszMfgName.is_null()
    {
        len = MultiByteToWideChar(CP_ACP, 0, (*pdiA).pszMfgName, -1, core::ptr::null_mut(), 0) as DWORD;
        diW.pszMfgName = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as PWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*pdiA).pszMfgName, -1, diW.pszMfgName, len as i32);
    }

    if (Level > 5) && !(*pdiA).pszOEMUrl.is_null()
    {
        len = MultiByteToWideChar(CP_ACP, 0, (*pdiA).pszOEMUrl, -1, core::ptr::null_mut(), 0) as DWORD;
        diW.pszOEMUrl = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as PWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*pdiA).pszOEMUrl, -1, diW.pszOEMUrl, len as i32);
    }

    if (Level > 5) && !(*pdiA).pszHardwareID.is_null()
    {
        len = MultiByteToWideChar(CP_ACP, 0, (*pdiA).pszHardwareID, -1, core::ptr::null_mut(), 0) as DWORD;
        diW.pszHardwareID = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as PWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*pdiA).pszHardwareID, -1, diW.pszHardwareID, len as i32);
    }

    if (Level > 5) && !(*pdiA).pszProvider.is_null()
    {
        len = MultiByteToWideChar(CP_ACP, 0, (*pdiA).pszProvider, -1, core::ptr::null_mut(), 0) as DWORD;
        diW.pszProvider = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as PWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*pdiA).pszProvider, -1, diW.pszProvider, len as i32);
    }

    if (Level > 7) && !(*pdiA).pszPrintProcessor.is_null()
    {
        len = MultiByteToWideChar(CP_ACP, 0, (*pdiA).pszPrintProcessor, -1, core::ptr::null_mut(), 0) as DWORD;
        diW.pszPrintProcessor = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as PWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*pdiA).pszPrintProcessor, -1, diW.pszPrintProcessor, len as i32);
    }

    if (Level > 7) && !(*pdiA).pszVendorSetup.is_null()
    {
        len = MultiByteToWideChar(CP_ACP, 0, (*pdiA).pszVendorSetup, -1, core::ptr::null_mut(), 0) as DWORD;
        diW.pszVendorSetup = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as PWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*pdiA).pszVendorSetup, -1, diW.pszVendorSetup, len as i32);
    }

    if (Level > 7) && !(*pdiA).pszzColorProfiles.is_null()
    {
        lenA = multi_sz_lenA((*pdiA).pszzColorProfiles) as DWORD;
        len = MultiByteToWideChar(CP_ACP, 0, (*pdiA).pszzColorProfiles, lenA as i32, core::ptr::null_mut(), 0) as DWORD;
        diW.pszzColorProfiles = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as PWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*pdiA).pszzColorProfiles, lenA as i32, diW.pszzColorProfiles, len as i32);
    }

    if (Level > 7) && !(*pdiA).pszInfPath.is_null()
    {
        len = MultiByteToWideChar(CP_ACP, 0, (*pdiA).pszInfPath, -1, core::ptr::null_mut(), 0) as DWORD;
        diW.pszInfPath = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as PWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*pdiA).pszInfPath, -1, diW.pszInfPath, len as i32);
    }

    if (Level > 7) && !(*pdiA).pszzCoreDriverDependencies.is_null()
    {
        lenA = multi_sz_lenA((*pdiA).pszzCoreDriverDependencies) as DWORD;
        len = MultiByteToWideChar(CP_ACP, 0, (*pdiA).pszzCoreDriverDependencies, lenA as i32, core::ptr::null_mut(), 0) as DWORD;
        diW.pszzCoreDriverDependencies = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<u16>()) as PWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*pdiA).pszzCoreDriverDependencies, lenA as i32, diW.pszzCoreDriverDependencies, len as i32);
    }

    if Level > 7
    {
        diW.dwPrinterDriverAttributes = (*pdiA).dwPrinterDriverAttributes;
        diW.ftMinInboxDriverVerDate = (*pdiA).ftMinInboxDriverVerDate;
        diW.dwlMinInboxDriverVerVersion = (*pdiA).dwlMinInboxDriverVerVersion;
    }

    res = AddPrinterDriverExW(nameW, Level, core::ptr::addr_of_mut!(diW) as LPBYTE, dwFileCopyFlags);

    if !nameW.is_null() { HeapFree(GetProcessHeap(), 0, nameW as LPVOID); }
    if !diW.pName.is_null() { HeapFree(GetProcessHeap(), 0, diW.pName as LPVOID); }
    if !diW.pEnvironment.is_null() { HeapFree(GetProcessHeap(), 0, diW.pEnvironment as LPVOID); }
    if !diW.pDriverPath.is_null() { HeapFree(GetProcessHeap(), 0, diW.pDriverPath as LPVOID); }
    if !diW.pDataFile.is_null() { HeapFree(GetProcessHeap(), 0, diW.pDataFile as LPVOID); }
    if !diW.pConfigFile.is_null() { HeapFree(GetProcessHeap(), 0, diW.pConfigFile as LPVOID); }
    if !diW.pHelpFile.is_null() { HeapFree(GetProcessHeap(), 0, diW.pHelpFile as LPVOID); }
    if !diW.pDependentFiles.is_null() { HeapFree(GetProcessHeap(), 0, diW.pDependentFiles as LPVOID); }
    if !diW.pMonitorName.is_null() { HeapFree(GetProcessHeap(), 0, diW.pMonitorName as LPVOID); }
    if !diW.pDefaultDataType.is_null() { HeapFree(GetProcessHeap(), 0, diW.pDefaultDataType as LPVOID); }
    if !diW.pszzPreviousNames.is_null() { HeapFree(GetProcessHeap(), 0, diW.pszzPreviousNames as LPVOID); }
    if !diW.pszMfgName.is_null() { HeapFree(GetProcessHeap(), 0, diW.pszMfgName as LPVOID); }
    if !diW.pszOEMUrl.is_null() { HeapFree(GetProcessHeap(), 0, diW.pszOEMUrl as LPVOID); }
    if !diW.pszHardwareID.is_null() { HeapFree(GetProcessHeap(), 0, diW.pszHardwareID as LPVOID); }
    if !diW.pszProvider.is_null() { HeapFree(GetProcessHeap(), 0, diW.pszProvider as LPVOID); }
    if !diW.pszPrintProcessor.is_null() { HeapFree(GetProcessHeap(), 0, diW.pszPrintProcessor as LPVOID); }
    if !diW.pszVendorSetup.is_null() { HeapFree(GetProcessHeap(), 0, diW.pszVendorSetup as LPVOID); }
    if !diW.pszzColorProfiles.is_null() { HeapFree(GetProcessHeap(), 0, diW.pszzColorProfiles as LPVOID); }
    if !diW.pszInfPath.is_null() { HeapFree(GetProcessHeap(), 0, diW.pszInfPath as LPVOID); }
    if !diW.pszzCoreDriverDependencies.is_null() { HeapFree(GetProcessHeap(), 0, diW.pszzCoreDriverDependencies as LPVOID); }

    return res;
}

#[no_mangle]
pub unsafe extern "system" fn AddPrinterDriverExW(pName: PWSTR, Level: DWORD, pDriverInfo: PBYTE, dwFileCopyFlags: DWORD) -> BOOL {
    let mut dwErrorCode: DWORD = ERROR_SUCCESS;
    let mut pdi: *mut WINSPOOL_DRIVER_INFO_8 = core::ptr::null_mut();
    let mut pDriverContainer: WINSPOOL_DRIVER_CONTAINER = core::mem::zeroed();

    pDriverContainer.Level = Level;

    let mut at: u8 = if Level == 8 {
        0
    } else if Level == 6 {
        1
    } else if Level == 4 {
        2
    } else if Level == 3 {
        3
    } else if Level == 2 {
        4
    } else {
        5
    };
    loop {
        match at {
            0 => {
                let pdi8w: PDRIVER_INFO_8W = pDriverInfo as PDRIVER_INFO_8W;
                pdi = HeapAlloc(hProcessHeap, 0, core::mem::size_of::<WINSPOOL_DRIVER_INFO_8>()) as *mut WINSPOOL_DRIVER_INFO_8;

                (*pdi).pPrintProcessor = (*pdi8w).pszPrintProcessor;
                (*pdi).pVendorSetup = (*pdi8w).pszVendorSetup;

                (*pdi).pszzColorProfiles = (*pdi8w).pszzColorProfiles;
                (*pdi).cchColorProfiles = 0;
                if !(*pdi8w).pszzColorProfiles.is_null() && *(*pdi8w).pszzColorProfiles != 0 {
                    (*pdi).cchColorProfiles = multi_sz_lenW((*pdi8w).pszzColorProfiles) as DWORD;
                }

                (*pdi).pInfPath = (*pdi8w).pszInfPath;

                (*pdi).pszzCoreDriverDependencies = (*pdi8w).pszzCoreDriverDependencies;
                (*pdi).cchCoreDependencies = 0;
                if !(*pdi8w).pszzCoreDriverDependencies.is_null() && *(*pdi8w).pszzCoreDriverDependencies != 0 {
                    (*pdi).cchCoreDependencies = multi_sz_lenW((*pdi8w).pszzCoreDriverDependencies) as DWORD;
                }

                (*pdi).ftMinInboxDriverVerDate = (*pdi8w).ftMinInboxDriverVerDate;
                (*pdi).dwlMinInboxDriverVerVersion = (*pdi8w).dwlMinInboxDriverVerVersion;
                at = 1;
                continue;
            }
            1 => {
                let pdi6w: PDRIVER_INFO_6W = pDriverInfo as PDRIVER_INFO_6W;
                if pdi.is_null() {
                    pdi = HeapAlloc(hProcessHeap, 0, core::mem::size_of::<WINSPOOL_DRIVER_INFO_6>()) as *mut WINSPOOL_DRIVER_INFO_8;
                }

                (*pdi).pMfgName = (*pdi6w).pszMfgName;
                (*pdi).pOEMUrl = (*pdi6w).pszOEMUrl;
                (*pdi).pHardwareID = (*pdi6w).pszHardwareID;
                (*pdi).pProvider = (*pdi6w).pszProvider;
                (*pdi).ftDriverDate = (*pdi6w).ftDriverDate;
                (*pdi).dwlDriverVersion = (*pdi6w).dwlDriverVersion;
                at = 2;
                continue;
            }
            2 => {
                let pdi4w: PDRIVER_INFO_4W = pDriverInfo as PDRIVER_INFO_4W;
                if pdi.is_null() {
                    pdi = HeapAlloc(hProcessHeap, 0, core::mem::size_of::<WINSPOOL_DRIVER_INFO_4>()) as *mut WINSPOOL_DRIVER_INFO_8;
                }

                (*pdi).pszzPreviousNames = (*pdi4w).pszzPreviousNames;
                (*pdi).cchPreviousNames = 0;
                if !(*pdi4w).pDependentFiles.is_null() && *(*pdi4w).pDependentFiles != 0 {
                    (*pdi).cchPreviousNames = multi_sz_lenW((*pdi4w).pDependentFiles) as DWORD;
                }
                at = 3;
                continue;
            }
            3 => {
                let pdi3w: PDRIVER_INFO_3W = pDriverInfo as PDRIVER_INFO_3W;
                if pdi.is_null() {
                    pdi = HeapAlloc(hProcessHeap, 0, core::mem::size_of::<WINSPOOL_DRIVER_INFO_3>()) as *mut WINSPOOL_DRIVER_INFO_8;
                }

                (*pdi).pHelpFile = (*pdi3w).pHelpFile;
                (*pdi).pDependentFiles = (*pdi3w).pDependentFiles;
                (*pdi).pMonitorName = (*pdi3w).pMonitorName;
                (*pdi).pDefaultDataType = (*pdi3w).pDefaultDataType;

                (*pdi).pDependentFiles = (*pdi3w).pDependentFiles;
                (*pdi).cchDependentFiles = 0;
                if !(*pdi3w).pDependentFiles.is_null() && *(*pdi3w).pDependentFiles != 0 {
                    (*pdi).cchDependentFiles = multi_sz_lenW((*pdi3w).pDependentFiles) as DWORD;
                }
                at = 4;
                continue;
            }
            4 => {
                let pdi2w: PDRIVER_INFO_2W = pDriverInfo as PDRIVER_INFO_2W;
                if pdi.is_null() {
                    pdi = HeapAlloc(hProcessHeap, 0, core::mem::size_of::<WINSPOOL_DRIVER_INFO_2>()) as *mut WINSPOOL_DRIVER_INFO_8;
                }

                (*pdi).pName = (*pdi2w).pName;

                (*pdi).pEnvironment = (*pdi2w).pEnvironment;
                if (*pdi2w).pEnvironment.is_null() || *(*pdi2w).pEnvironment == 0 {
                    (*pdi2w).pEnvironment = wszCurrentEnvironment.as_ptr() as PWSTR;
                }

                (*pdi).pDriverPath = (*pdi2w).pDriverPath;
                (*pdi).pDataFile = (*pdi2w).pDataFile;
                (*pdi).pConfigFile = (*pdi2w).pConfigFile;
                break;
            }
            _ => {
                SetLastError(ERROR_INVALID_LEVEL);
                return FALSE as BOOL;
            }
        }
    }

    pDriverContainer.DriverInfo.Level8 = pdi;

    // RpcExcept(EXCEPTION_EXECUTE_HANDLER) handler: dwErrorCode = RpcExceptionCode();
    // RpcEndExcept
    dwErrorCode = rpc_try(|| _RpcAddPrinterDriverEx(pName, core::ptr::addr_of_mut!(pDriverContainer), dwFileCopyFlags)).unwrap_or_else(|code| code as _);

    if !pdi.is_null() {
        HeapFree(GetProcessHeap(), 0, pdi as LPVOID);
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn AddPrinterDriverW(pName: PWSTR, Level: DWORD, pDriverInfo: PBYTE) -> BOOL {
    return AddPrinterDriverExW(pName, Level, pDriverInfo, APD_COPY_NEW_FILES);
}

#[no_mangle]
pub unsafe extern "system" fn DeletePrinterDriverA(pName: PSTR, pEnvironment: PSTR, pDriverName: PSTR) -> BOOL {
    return DeletePrinterDriverExA(pName, pEnvironment, pDriverName, 0, 0);
}

#[no_mangle]
pub unsafe extern "system" fn DeletePrinterDriverExA(pName: PSTR, pEnvironment: PSTR, pDriverName: PSTR, dwDeleteFlag: DWORD, dwVersionFlag: DWORD) -> BOOL {
    let mut dwErrorCode: DWORD;
    let mut NameW: _UNICODE_STRING = core::mem::zeroed();
    let mut EnvW: _UNICODE_STRING = core::mem::zeroed();
    let mut DriverW: _UNICODE_STRING = core::mem::zeroed();
    let mut ret: BOOL;

    AsciiToUnicode(core::ptr::addr_of_mut!(NameW), pName);
    AsciiToUnicode(core::ptr::addr_of_mut!(EnvW), pEnvironment);
    AsciiToUnicode(core::ptr::addr_of_mut!(DriverW), pDriverName);

    ret = DeletePrinterDriverExW(NameW.Buffer, EnvW.Buffer, DriverW.Buffer, dwDeleteFlag, dwVersionFlag);

    dwErrorCode = GetLastError();

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(DriverW));
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(EnvW));
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(NameW));

    SetLastError(dwErrorCode);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn DeletePrinterDriverExW(pName: PWSTR, pEnvironment: PWSTR, pDriverName: PWSTR, dwDeleteFlag: DWORD, dwVersionFlag: DWORD) -> BOOL {
    let mut pEnvironment = pEnvironment;
    let mut dwErrorCode: DWORD;

    if pDriverName.is_null() || *pDriverName == 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    if pEnvironment.is_null() || *pEnvironment == 0 {
        pEnvironment = core::ptr::addr_of!(wszCurrentEnvironment) as *mut u16;
    }

    // Do the RPC call.
    // RpcTryExcept
    {
        dwErrorCode = rpc_try(|| _RpcDeletePrinterDriverEx(pName, pEnvironment, pDriverName, dwDeleteFlag, dwVersionFlag)).unwrap_or_else(|code| code as _);
    }
    // RpcExcept handler: dead under this tree's USE_DUMMY_PSEH build (the C handler never runs either), and stable Rust has no SEH. Original: dwErrorCode = RpcExceptionCode();
    // RpcEndExcept

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn DeletePrinterDriverW(pName: PWSTR, pEnvironment: PWSTR, pDriverName: PWSTR) -> i32 {
    return DeletePrinterDriverExW(pName, pEnvironment, pDriverName, 0, 0);
}

#[no_mangle]
pub unsafe extern "system" fn EnumPrinterDriversA(pName: PSTR, pEnvironment: PSTR, Level: DWORD, pDriverInfo: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD, pcReturned: PDWORD) -> BOOL {
    let mut ret: BOOL = FALSE as BOOL;
    let mut dwErrorCode: DWORD = 0;
    let mut i: DWORD = 0;
    let mut pNameW: _UNICODE_STRING = core::mem::zeroed();
    let mut pEnvironmentW: _UNICODE_STRING = core::mem::zeroed();
    let mut pwstrNameW: PWSTR = core::ptr::null_mut();
    let mut pwstrEnvironmentW: PWSTR = core::ptr::null_mut();
    let pdi1w: PDRIVER_INFO_1W = pDriverInfo as PDRIVER_INFO_1W;
    let pdi8w: PDRIVER_INFO_8W = pDriverInfo as PDRIVER_INFO_8W;

    pwstrNameW = AsciiToUnicode(core::ptr::addr_of_mut!(pNameW), pName);
    pwstrEnvironmentW = AsciiToUnicode(core::ptr::addr_of_mut!(pEnvironmentW), pEnvironment);

    'cleanup: {
        ret = EnumPrinterDriversW(pwstrNameW, pwstrEnvironmentW, Level, pDriverInfo, cbBuf, pcbNeeded, pcReturned);

        dwErrorCode = GetLastError();

        if ret != 0 {
            i = 0;
            while i < *pcReturned {
                // switch (Level) with C fall-through, lowered as a section dispatch
                let mut sw: u8 = 0;
                if Level == 1 {
                    sw = 1;
                } else if Level == 8 {
                    sw = 2;
                } else if Level == 6 {
                    sw = 3;
                } else if Level == 4 {
                    sw = 4;
                } else if Level == 3 {
                    sw = 5;
                } else if Level == 2 || Level == 5 {
                    sw = 6;
                }

                // case COMMENT_TOKEN_ECHO____ (ends in break)
                if sw == 1 {
                    dwErrorCode = UnicodeToAnsiInPlace((*pdi1w.offset(i as isize)).pName);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                }

                // case COMMENT_TOKEN_ECHO____ (falls through)
                if sw >= 2 && sw <= 2 {
                    dwErrorCode = UnicodeToAnsiInPlace((*pdi8w.offset(i as isize)).pszPrintProcessor);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pdi8w.offset(i as isize)).pszVendorSetup);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiZZInPlace((*pdi8w.offset(i as isize)).pszzColorProfiles);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pdi8w.offset(i as isize)).pszInfPath);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiZZInPlace((*pdi8w.offset(i as isize)).pszzCoreDriverDependencies);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                }

                // case COMMENT_TOKEN_ECHO____ (falls through)
                if sw >= 2 && sw <= 3 {
                    dwErrorCode = UnicodeToAnsiInPlace((*pdi8w.offset(i as isize)).pszMfgName);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }

                    dwErrorCode = UnicodeToAnsiInPlace((*pdi8w.offset(i as isize)).pszOEMUrl);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }

                    dwErrorCode = UnicodeToAnsiInPlace((*pdi8w.offset(i as isize)).pszHardwareID);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }

                    dwErrorCode = UnicodeToAnsiInPlace((*pdi8w.offset(i as isize)).pszProvider);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                }

                // case COMMENT_TOKEN_ECHO____ (falls through)
                if sw >= 2 && sw <= 4 {
                    dwErrorCode = UnicodeToAnsiZZInPlace((*pdi8w.offset(i as isize)).pszzPreviousNames);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                }

                // case COMMENT_TOKEN_ECHO____ (falls through)
                if sw >= 2 && sw <= 5 {
                    dwErrorCode = UnicodeToAnsiInPlace((*pdi8w.offset(i as isize)).pHelpFile);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }

                    dwErrorCode = UnicodeToAnsiZZInPlace((*pdi8w.offset(i as isize)).pDependentFiles);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }

                    dwErrorCode = UnicodeToAnsiInPlace((*pdi8w.offset(i as isize)).pMonitorName);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }

                    dwErrorCode = UnicodeToAnsiInPlace((*pdi8w.offset(i as isize)).pDefaultDataType);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                }

                // case COMMENT_TOKEN_ECHO____ / COMMENT_TOKEN_ECHO____ (last section)
                if sw >= 2 && sw <= 6 {
                    dwErrorCode = UnicodeToAnsiInPlace((*pdi8w.offset(i as isize)).pName);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }

                    dwErrorCode = UnicodeToAnsiInPlace((*pdi8w.offset(i as isize)).pEnvironment);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }

                    dwErrorCode = UnicodeToAnsiInPlace((*pdi8w.offset(i as isize)).pDriverPath);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }

                    dwErrorCode = UnicodeToAnsiInPlace((*pdi8w.offset(i as isize)).pDataFile);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }

                    dwErrorCode = UnicodeToAnsiInPlace((*pdi8w.offset(i as isize)).pConfigFile);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                }

                i += 1;
            }
            dwErrorCode = ERROR_SUCCESS;
        }
    }
    // Cleanup:
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(pNameW));
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(pEnvironmentW));
    SetLastError(dwErrorCode);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn EnumPrinterDriversW(pName: PWSTR, mut pEnvironment: PWSTR, Level: DWORD, pDriverInfo: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD, pcReturned: PDWORD) -> BOOL {
    let mut dwErrorCode: DWORD = 0;

    'Cleanup: {
        // Dismiss invalid levels already at this point.
        if Level < 1 || Level == 7 || Level > 8 {
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'Cleanup;
        }

        if pEnvironment.is_null() || *pEnvironment == 0 {
            pEnvironment = core::ptr::addr_of!(wszCurrentEnvironment) as *const u16 as PWSTR;
        }

        if cbBuf != 0 && !pDriverInfo.is_null() {
            ZeroMemory!(pDriverInfo as *mut core::ffi::c_void, cbBuf as usize);
        }

        // Do the RPC call
        // RpcTryExcept
        {
            dwErrorCode = rpc_try(|| _RpcEnumPrinterDrivers(pName, pEnvironment, Level, pDriverInfo, cbBuf, pcbNeeded, pcReturned)).unwrap_or_else(|code| code as _);
        }
        // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
        // RpcEndExcept;

        if dwErrorCode == ERROR_SUCCESS {
            // Replace relative offset addresses in the output by absolute pointers.
            MarshallUpStructuresArray(cbBuf, pDriverInfo as PVOID, *pcReturned, core::ptr::addr_of!((*pPrinterDriverMarshalling[Level as usize]).pInfo) as *const MARSHALLING_INFO, (*pPrinterDriverMarshalling[Level as usize]).cbStructureSize, TRUE as BOOL);
        }
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetPrinterDriverA(hPrinter: HANDLE, pEnvironment: LPSTR, Level: DWORD, pDriverInfo: LPBYTE, cbBuf: DWORD, pcbNeeded: LPDWORD) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    /*
     * We are mapping multiple different pointers to the same pDriverInfo pointer here so that
     * we can use the same incoming pointer for different Levels
     */
    let pdi1w: PDRIVER_INFO_1W = pDriverInfo as PDRIVER_INFO_1W;
    let pdi8w: PDRIVER_INFO_8W = pDriverInfo as PDRIVER_INFO_8W;

    let mut cch: DWORD = 0;
    let mut pwszEnvironment: PWSTR = core::ptr::null_mut();

    'cleanup: {
        // Check for invalid levels here for early error return. Should be 1-6 & 8.
        if Level < 1 || Level == 7 || Level > 8 {
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'cleanup;
        }

        if !pEnvironment.is_null() {
            // Convert pEnvironment to a Unicode string pwszEnvironment.
            cch = strlen(pEnvironment) as DWORD;

            pwszEnvironment = HeapAlloc(hProcessHeap, 0, ((cch + 1) as usize) * core::mem::size_of::<WCHAR>()) as PWSTR;
            if pwszEnvironment.is_null() {
                dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                break 'cleanup;
            }

            MultiByteToWideChar(CP_ACP, 0, pEnvironment, -1, pwszEnvironment, (cch + 1) as i32);
        }

        if GetPrinterDriverW(hPrinter, pwszEnvironment, Level, pDriverInfo, cbBuf, pcbNeeded) == 0 {
            dwErrorCode = GetLastError();
            break 'cleanup;
        }

        // Do Unicode to ANSI conversions for strings based on Level
        // C switch with fallthrough, lowered as a sequence of gated blocks.
        let mut ft: bool = false;
        'sw: {
            if Level == 1 {
                dwErrorCode = UnicodeToAnsiInPlace((*pdi1w).pName);
                if dwErrorCode != ERROR_SUCCESS {
                    break 'cleanup;
                }
                break 'sw;
            }
            if ft || Level == 8 {
                ft = true;
                dwErrorCode = UnicodeToAnsiInPlace((*pdi8w).pszPrintProcessor);
                if dwErrorCode != ERROR_SUCCESS {
                    break 'cleanup;
                }
                dwErrorCode = UnicodeToAnsiInPlace((*pdi8w).pszVendorSetup);
                if dwErrorCode != ERROR_SUCCESS {
                    break 'cleanup;
                }
                dwErrorCode = UnicodeToAnsiZZInPlace((*pdi8w).pszzColorProfiles);
                if dwErrorCode != ERROR_SUCCESS {
                    break 'cleanup;
                }
                dwErrorCode = UnicodeToAnsiInPlace((*pdi8w).pszInfPath);
                if dwErrorCode != ERROR_SUCCESS {
                    break 'cleanup;
                }
                dwErrorCode = UnicodeToAnsiZZInPlace((*pdi8w).pszzCoreDriverDependencies);
                if dwErrorCode != ERROR_SUCCESS {
                    break 'cleanup;
                }
            }
            if ft || Level == 6 {
                ft = true;
                dwErrorCode = UnicodeToAnsiInPlace((*pdi8w).pszMfgName);
                if dwErrorCode != ERROR_SUCCESS {
                    break 'cleanup;
                }

                dwErrorCode = UnicodeToAnsiInPlace((*pdi8w).pszOEMUrl);
                if dwErrorCode != ERROR_SUCCESS {
                    break 'cleanup;
                }

                dwErrorCode = UnicodeToAnsiInPlace((*pdi8w).pszHardwareID);
                if dwErrorCode != ERROR_SUCCESS {
                    break 'cleanup;
                }

                dwErrorCode = UnicodeToAnsiInPlace((*pdi8w).pszProvider);
                if dwErrorCode != ERROR_SUCCESS {
                    break 'cleanup;
                }
            }
            if ft || Level == 4 {
                ft = true;
                dwErrorCode = UnicodeToAnsiZZInPlace((*pdi8w).pszzPreviousNames);
                if dwErrorCode != ERROR_SUCCESS {
                    break 'cleanup;
                }
            }
            if ft || Level == 3 {
                ft = true;
                dwErrorCode = UnicodeToAnsiInPlace((*pdi8w).pHelpFile);
                if dwErrorCode != ERROR_SUCCESS {
                    break 'cleanup;
                }

                dwErrorCode = UnicodeToAnsiZZInPlace((*pdi8w).pDependentFiles);
                if dwErrorCode != ERROR_SUCCESS {
                    break 'cleanup;
                }

                dwErrorCode = UnicodeToAnsiInPlace((*pdi8w).pMonitorName);
                if dwErrorCode != ERROR_SUCCESS {
                    break 'cleanup;
                }

                dwErrorCode = UnicodeToAnsiInPlace((*pdi8w).pDefaultDataType);
                if dwErrorCode != ERROR_SUCCESS {
                    break 'cleanup;
                }
            }
            if ft || Level == 2 || Level == 5 {
                ft = true;
                dwErrorCode = UnicodeToAnsiInPlace((*pdi8w).pConfigFile);
                if dwErrorCode != ERROR_SUCCESS {
                    break 'cleanup;
                }
                dwErrorCode = UnicodeToAnsiInPlace((*pdi8w).pDataFile);

                if dwErrorCode != ERROR_SUCCESS {
                    break 'cleanup;
                }

                dwErrorCode = UnicodeToAnsiInPlace((*pdi8w).pDriverPath);
                if dwErrorCode != ERROR_SUCCESS {
                    break 'cleanup;
                }

                dwErrorCode = UnicodeToAnsiInPlace((*pdi8w).pEnvironment);
                if dwErrorCode != ERROR_SUCCESS {
                    break 'cleanup;
                }

                dwErrorCode = UnicodeToAnsiInPlace((*pdi8w).pName);
                if dwErrorCode != ERROR_SUCCESS {
                    break 'cleanup;
                }
            }
        }

        dwErrorCode = ERROR_SUCCESS;
    }

    // Cleanup:
    if !pwszEnvironment.is_null() {
        HeapFree(hProcessHeap, 0, pwszEnvironment as LPVOID);
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetPrinterDriverW(hPrinter: HANDLE, mut pEnvironment: LPWSTR, Level: DWORD, pDriverInfo: LPBYTE, cbBuf: DWORD, pcbNeeded: LPDWORD) -> WINBOOL {
    let mut dwErrorCode: DWORD = 0;
    let pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;

    'Cleanup: {
        // Sanity checks.
        if pHandle.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        // Dismiss invalid levels already at this point.
        if Level > 8 || Level == 7 || Level < 1 {
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'Cleanup;
        }

        if pEnvironment.is_null() || *pEnvironment == 0 {
            pEnvironment = core::ptr::addr_of!(wszCurrentEnvironment) as *const WCHAR as PWSTR;
        }

        if cbBuf != 0 && !pDriverInfo.is_null() {
            ZeroMemory!(pDriverInfo as *mut core::ffi::c_void, cbBuf as usize);
        }

        // RpcTryExcept
        dwErrorCode = rpc_try(|| _RpcGetPrinterDriver((*pHandle).hPrinter, pEnvironment, Level, pDriverInfo, cbBuf, pcbNeeded)).unwrap_or_else(|code| code as _);
        // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
        // RpcExcept handler: dead under this tree's USE_DUMMY_PSEH build (the C handler never runs either), and stable Rust has no SEH. Original: dwErrorCode = RpcExceptionCode();
        // RpcEndExcept

        if dwErrorCode == ERROR_SUCCESS {
            // Replace relative offset addresses in the output by absolute pointers.
            MarshallUpStructure(
                cbBuf,
                pDriverInfo as PVOID,
                core::ptr::addr_of!((*pPrinterDriverMarshalling[Level as usize]).pInfo) as *const _MARSHALLING_INFO,
                (*pPrinterDriverMarshalling[Level as usize]).cbStructureSize,
                TRUE as BOOL,
            );
        }
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetPrinterDriverDirectoryA(pName: PSTR, pEnvironment: PSTR, Level: DWORD, pDriverDirectory: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD) -> BOOL {
    let mut nameW: _UNICODE_STRING = core::mem::zeroed();
    let mut environmentW: _UNICODE_STRING = core::mem::zeroed();
    let mut ret: BOOL;
    let mut pcbNeededW: DWORD = 0;
    let len: INT = ((cbBuf as INT) * (core::mem::size_of::<WCHAR>() as INT)) / (core::mem::size_of::<CHAR>() as INT);
    let mut driverDirectoryW: *mut WCHAR = core::ptr::null_mut();

    if len != 0 {
        driverDirectoryW = HeapAlloc(GetProcessHeap(), 0, len as usize) as *mut WCHAR;
    }

    if !pName.is_null() {
        RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(nameW), pName);
    } else {
        nameW.Buffer = core::ptr::null_mut();
    }
    if !pEnvironment.is_null() {
        RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(environmentW), pEnvironment);
    } else {
        environmentW.Buffer = core::ptr::null_mut();
    }

    ret = GetPrinterDriverDirectoryW(nameW.Buffer, environmentW.Buffer, Level, driverDirectoryW as LPBYTE, len as DWORD, core::ptr::addr_of_mut!(pcbNeededW));

    if ret != 0 {
        let needed: DWORD = WideCharToMultiByte(CP_ACP, 0, driverDirectoryW, -1, pDriverDirectory as LPSTR, cbBuf as INT, core::ptr::null_mut(), core::ptr::null_mut()) as DWORD;

        if !pcbNeeded.is_null() {
            *pcbNeeded = needed;
        }

        ret = (needed <= cbBuf) as BOOL;
    } else {
        if !pcbNeeded.is_null() {
            *pcbNeeded = pcbNeededW * (core::mem::size_of::<CHAR>() as DWORD) / (core::mem::size_of::<WCHAR>() as DWORD);
        }
    }

    HeapFree(GetProcessHeap(), 0, driverDirectoryW as *mut core::ffi::c_void);
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(environmentW));
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(nameW));

    ret
}

#[no_mangle]
pub unsafe extern "system" fn GetPrinterDriverDirectoryW(pName: PWSTR, mut pEnvironment: PWSTR, Level: DWORD, pDriverDirectory: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD) -> BOOL {
    let mut dwErrorCode: DWORD = 0;

    'Cleanup: {
        if Level != 1 {
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'Cleanup;
        }

        if pEnvironment.is_null() || *pEnvironment == 0 {
            pEnvironment = core::ptr::addr_of!(wszCurrentEnvironment) as *const u16 as PWSTR;
        }

        // Do the RPC call.
        // RpcTryExcept: the protected body runs as a plain block (SEH is not modeled).
        {
            dwErrorCode = rpc_try(|| _RpcGetPrinterDriverDirectory(pName, pEnvironment, Level, pDriverDirectory, cbBuf, pcbNeeded)).unwrap_or_else(|code| code as _);
        }
        // RpcExcept(EXCEPTION_EXECUTE_HANDLER): handler is kept but cannot be reached
        // without SEH; it is preserved so the call and its assignment remain present.
        // RpcEndExcept
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}
