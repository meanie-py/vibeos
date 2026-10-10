use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn _RpcAddPrinterDriver(pName: WINSPOOL_HANDLE, pDriverContainer: *mut WINSPOOL_DRIVER_CONTAINER) -> DWORD {
    let dwErrorCode: DWORD;
    let mut pDriverInfo: PBYTE = core::ptr::null_mut();

    let lvl = (*pDriverContainer).Level;
    let mut at: u8 = if lvl == 8 {
        0
    } else if lvl == 6 {
        1
    } else if lvl == 4 {
        2
    } else if lvl == 3 {
        3
    } else if lvl == 2 {
        4
    } else {
        return ERROR_INVALID_LEVEL;
    };

    loop {
        match at {
            0 => {
                let pdi: *mut WINSPOOL_DRIVER_INFO_8 = (*pDriverContainer).DriverInfo.Level8;
                let pdi8w: PDRIVER_INFO_8W = DllAllocSplMem(core::mem::size_of::<DRIVER_INFO_8W>() as u32) as PDRIVER_INFO_8W;
                pDriverInfo = pdi8w as PBYTE;

                (*pdi8w).pszPrintProcessor = (*pdi).pPrintProcessor;
                (*pdi8w).pszVendorSetup = (*pdi).pVendorSetup;
                (*pdi8w).pszzColorProfiles = (*pdi).pszzColorProfiles;
                (*pdi8w).pszInfPath = (*pdi).pInfPath;
                (*pdi8w).pszzCoreDriverDependencies = (*pdi).pszzCoreDriverDependencies;
                (*pdi8w).ftMinInboxDriverVerDate = (*pdi).ftMinInboxDriverVerDate;
                (*pdi8w).dwlMinInboxDriverVerVersion = (*pdi).dwlMinInboxDriverVerVersion;
                at = 1;
                continue;
            }
            1 => {
                let pdi: *mut WINSPOOL_DRIVER_INFO_6 = (*pDriverContainer).DriverInfo.Level6;
                let pdi6w: PDRIVER_INFO_6W;

                if pDriverInfo.is_null() {
                    pdi6w = DllAllocSplMem(core::mem::size_of::<DRIVER_INFO_6W>() as u32) as PDRIVER_INFO_6W;
                    pDriverInfo = pdi6w as PBYTE;
                } else {
                    pdi6w = pDriverInfo as PDRIVER_INFO_6W;
                }

                (*pdi6w).pszMfgName = (*pdi).pMfgName;
                (*pdi6w).pszOEMUrl = (*pdi).pOEMUrl;
                (*pdi6w).pszHardwareID = (*pdi).pHardwareID;
                (*pdi6w).pszProvider = (*pdi).pProvider;
                (*pdi6w).ftDriverDate = (*pdi).ftDriverDate;
                (*pdi6w).dwlDriverVersion = (*pdi).dwlDriverVersion;
                at = 2;
                continue;
            }
            2 => {
                let pdi: *mut WINSPOOL_DRIVER_INFO_4 = (*pDriverContainer).DriverInfo.Level4;
                let pdi4w: PDRIVER_INFO_4W;

                if pDriverInfo.is_null() {
                    pdi4w = DllAllocSplMem(core::mem::size_of::<DRIVER_INFO_4W>() as u32) as PDRIVER_INFO_4W;
                    pDriverInfo = pdi4w as PBYTE;
                } else {
                    pdi4w = pDriverInfo as PDRIVER_INFO_4W;
                }

                (*pdi4w).pszzPreviousNames = (*pdi).pszzPreviousNames;
                at = 3;
                continue;
            }
            3 => {
                let pdi: *mut WINSPOOL_DRIVER_INFO_3 = (*pDriverContainer).DriverInfo.Level3;
                let pdi3w: PDRIVER_INFO_3W;

                if pDriverInfo.is_null() {
                    pdi3w = DllAllocSplMem(core::mem::size_of::<DRIVER_INFO_3W>() as u32) as PDRIVER_INFO_3W;
                    pDriverInfo = pdi3w as PBYTE;
                } else {
                    pdi3w = pDriverInfo as PDRIVER_INFO_3W;
                }

                (*pdi3w).pHelpFile = (*pdi).pHelpFile;
                (*pdi3w).pDependentFiles = (*pdi).pDependentFiles;
                (*pdi3w).pMonitorName = (*pdi).pMonitorName;
                (*pdi3w).pDefaultDataType = (*pdi).pDefaultDataType;
                (*pdi3w).pDependentFiles = (*pdi).pDependentFiles;
                at = 4;
                continue;
            }
            4 => {
                let pdi: *mut WINSPOOL_DRIVER_INFO_2 = (*pDriverContainer).DriverInfo.Level2;
                let pdi2w: PDRIVER_INFO_2W;

                if pDriverInfo.is_null() {
                    pdi2w = DllAllocSplMem(core::mem::size_of::<DRIVER_INFO_2W>() as u32) as PDRIVER_INFO_2W;
                    pDriverInfo = pdi2w as PBYTE;
                } else {
                    pdi2w = pDriverInfo as PDRIVER_INFO_2W;
                }

                (*pdi2w).pName = (*pdi).pName;
                (*pdi2w).pEnvironment = (*pdi).pEnvironment;
                (*pdi2w).pDriverPath = (*pdi).pDriverPath;
                (*pdi2w).pDataFile = (*pdi).pDataFile;
                (*pdi2w).pConfigFile = (*pdi).pConfigFile;
                break;
            }
            _ => break,
        }
    }

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    let mut dwErrorCode = dwErrorCode;
    if AddPrinterDriverW(pName, (*pDriverContainer).Level, pDriverInfo) == 0 {
        dwErrorCode = GetLastError();
    }

    if !pDriverInfo.is_null() {
        DllFreeSplMem(pDriverInfo as LPVOID);
    }

    RpcRevertToSelf();
    dwErrorCode
}
#[no_mangle]
pub unsafe extern "C" fn _RpcAddPrinterDriverEx(pName: WINSPOOL_HANDLE, pDriverContainer: *mut WINSPOOL_DRIVER_CONTAINER, dwFileCopyFlags: DWORD) -> DWORD {
    let mut dwErrorCode: DWORD = 0;
    let mut pDriverInfo: PBYTE = core::ptr::null_mut();

    let lvl: DWORD = (*pDriverContainer).Level;
    let mut at: u8 = if lvl == 8 {
        0
    } else if lvl == 6 {
        1
    } else if lvl == 4 {
        2
    } else if lvl == 3 {
        3
    } else if lvl == 2 {
        4
    } else {
        return ERROR_INVALID_LEVEL;
    };

    // switch with fallthrough: a case at index `at` runs its body and all later bodies
    if at <= 0 {
        let pdi: *mut WINSPOOL_DRIVER_INFO_8 = (*pDriverContainer).DriverInfo.Level8;
        let pdi8w: PDRIVER_INFO_8W = DllAllocSplMem((core::mem::size_of::<DRIVER_INFO_8W>()) as DWORD) as PDRIVER_INFO_8W;
        pDriverInfo = pdi8w as PBYTE;

        (*pdi8w).pszPrintProcessor = (*pdi).pPrintProcessor;
        (*pdi8w).pszVendorSetup = (*pdi).pVendorSetup;
        (*pdi8w).pszzColorProfiles = (*pdi).pszzColorProfiles;
        (*pdi8w).pszInfPath = (*pdi).pInfPath;
        (*pdi8w).pszzCoreDriverDependencies = (*pdi).pszzCoreDriverDependencies;
        (*pdi8w).ftMinInboxDriverVerDate = (*pdi).ftMinInboxDriverVerDate;
        (*pdi8w).dwlMinInboxDriverVerVersion = (*pdi).dwlMinInboxDriverVerVersion;
    }
    if at <= 1 {
        let pdi: *mut WINSPOOL_DRIVER_INFO_6 = (*pDriverContainer).DriverInfo.Level6;
        let mut pdi6w: PDRIVER_INFO_6W = core::ptr::null_mut();

        if pDriverInfo.is_null() {
            pdi6w = DllAllocSplMem((core::mem::size_of::<DRIVER_INFO_6W>()) as DWORD) as PDRIVER_INFO_6W;
            pDriverInfo = pdi6w as PBYTE;
        } else {
            pdi6w = pDriverInfo as PDRIVER_INFO_6W;
        }

        (*pdi6w).pszMfgName = (*pdi).pMfgName;
        (*pdi6w).pszOEMUrl = (*pdi).pOEMUrl;
        (*pdi6w).pszHardwareID = (*pdi).pHardwareID;
        (*pdi6w).pszProvider = (*pdi).pProvider;
        (*pdi6w).ftDriverDate = (*pdi).ftDriverDate;
        (*pdi6w).dwlDriverVersion = (*pdi).dwlDriverVersion;
    }
    if at <= 2 {
        let pdi: *mut WINSPOOL_DRIVER_INFO_4 = (*pDriverContainer).DriverInfo.Level4;
        let mut pdi4w: PDRIVER_INFO_4W = core::ptr::null_mut();

        if pDriverInfo.is_null() {
            pdi4w = DllAllocSplMem((core::mem::size_of::<DRIVER_INFO_4W>()) as DWORD) as PDRIVER_INFO_4W;
            pDriverInfo = pdi4w as PBYTE;
        } else {
            pdi4w = pDriverInfo as PDRIVER_INFO_4W;
        }

        (*pdi4w).pszzPreviousNames = (*pdi).pszzPreviousNames;
    }
    if at <= 3 {
        let pdi: *mut WINSPOOL_DRIVER_INFO_3 = (*pDriverContainer).DriverInfo.Level3;
        let mut pdi3w: PDRIVER_INFO_3W = core::ptr::null_mut();

        if pDriverInfo.is_null() {
            pdi3w = DllAllocSplMem((core::mem::size_of::<DRIVER_INFO_3W>()) as DWORD) as PDRIVER_INFO_3W;
            pDriverInfo = pdi3w as PBYTE;
        } else {
            pdi3w = pDriverInfo as PDRIVER_INFO_3W;
        }

        (*pdi3w).pHelpFile = (*pdi).pHelpFile;
        (*pdi3w).pDependentFiles = (*pdi).pDependentFiles;
        (*pdi3w).pMonitorName = (*pdi).pMonitorName;
        (*pdi3w).pDefaultDataType = (*pdi).pDefaultDataType;
        (*pdi3w).pDependentFiles = (*pdi).pDependentFiles;
    }
    if at <= 4 {
        let pdi: *mut WINSPOOL_DRIVER_INFO_2 = (*pDriverContainer).DriverInfo.Level2;
        let mut pdi2w: PDRIVER_INFO_2W = core::ptr::null_mut();

        if pDriverInfo.is_null() {
            pdi2w = DllAllocSplMem((core::mem::size_of::<DRIVER_INFO_2W>()) as DWORD) as PDRIVER_INFO_2W;
            pDriverInfo = pdi2w as PBYTE;
        } else {
            pdi2w = pDriverInfo as PDRIVER_INFO_2W;
        }

        (*pdi2w).pName = (*pdi).pName;
        (*pdi2w).pEnvironment = (*pdi).pEnvironment;
        (*pdi2w).pDriverPath = (*pdi).pDriverPath;
        (*pdi2w).pDataFile = (*pdi).pDataFile;
        (*pdi2w).pConfigFile = (*pdi).pConfigFile;
    }

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    if AddPrinterDriverExW(pName, (*pDriverContainer).Level, pDriverInfo, dwFileCopyFlags) == 0 {
        dwErrorCode = GetLastError();
    }

    if !pDriverInfo.is_null() {
        DllFreeSplMem((pDriverInfo).cast());
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcDeletePrinterDriver(pName: WINSPOOL_HANDLE, pEnvironment: LPWSTR, pDriverName: LPWSTR) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    if DeletePrinterDriverW(pName, pEnvironment, pDriverName) == 0 {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcDeletePrinterDriverEx(pName: WINSPOOL_HANDLE, pEnvironment: LPWSTR, pDriverName: LPWSTR, dwDeleteFlag: DWORD, dwVersionNum: DWORD) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS
    {
        return dwErrorCode;
    }

    if DeletePrinterDriverExW(pName, pEnvironment, pDriverName, dwDeleteFlag, dwVersionNum) == 0
        {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcEnumPrinterDrivers(pName: WINSPOOL_HANDLE, pEnvironment: *mut WCHAR, Level: DWORD, pDrivers: *mut BYTE, cbBuf: DWORD, pcbNeeded: *mut DWORD, pcReturned: *mut DWORD) -> DWORD {
    let mut dwErrorCode: DWORD;
    let mut pPrinterDriversEnumAligned: PBYTE;
    let mut cbBuf = cbBuf;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS
    {
        return dwErrorCode;
    }

    pPrinterDriversEnumAligned = (AlignRpcPtr((pDrivers).cast(), core::ptr::addr_of_mut!(cbBuf))).cast();

    if EnumPrinterDriversW(pName, pEnvironment, Level, pPrinterDriversEnumAligned, cbBuf, pcbNeeded, pcReturned) != 0
    {
        // Replace absolute pointer addresses in the output by relative offsets.
        MarshallDownStructuresArray((pPrinterDriversEnumAligned).cast(), *pcReturned, (*pPrinterDriverMarshalling[Level as usize]).pInfo.as_ptr(), (*pPrinterDriverMarshalling[Level as usize]).cbStructureSize, (TRUE) as BOOL);
    }
    else
    {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    UndoAlignRpcPtr((pDrivers).cast(), (pPrinterDriversEnumAligned).cast(), cbBuf, pcbNeeded);

    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcGetPrinterDriver(hPrinter: WINSPOOL_PRINTER_HANDLE, pEnvironment: *mut u16, Level: DWORD, pDriver: *mut u8, mut cbBuf: DWORD, pcbNeeded: *mut DWORD) -> DWORD {
    let mut dwErrorCode: DWORD;
    let mut pDriverAligned: *mut u8;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS
    {
        return dwErrorCode;
    }

    pDriverAligned = (AlignRpcPtr((pDriver).cast(), core::ptr::addr_of_mut!(cbBuf))).cast();

    if GetPrinterDriverW(hPrinter, pEnvironment, Level, pDriverAligned, cbBuf, pcbNeeded) != 0
    {
        // Replace relative offset addresses in the output by absolute pointers.
        MarshallDownStructure((pDriverAligned).cast(), (*pPrinterDriverMarshalling[Level as usize]).pInfo.as_ptr(), (*pPrinterDriverMarshalling[Level as usize]).cbStructureSize, (TRUE) as BOOL);
    }
    else
    {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    UndoAlignRpcPtr((pDriver).cast(), (pDriverAligned).cast(), cbBuf, pcbNeeded);

    return dwErrorCode;
}

#[no_mangle]
pub unsafe extern "system" fn YGetPrinterDriver2(
    hPrinter: HANDLE,
    pEnvironment: LPWSTR,
    Level: DWORD,
    pDriver: LPBYTE,
    cbBuf: DWORD,
    pcbNeeded: LPDWORD,
    dwClientMajorVersion: DWORD,
    dwClientMinorVersion: DWORD,
    pdwServerMajorVersion: PDWORD,
    pdwServerMinorVersion: PDWORD,
    bRPC: BOOL,
) -> BOOL {
    let mut cbBuf = cbBuf;
    let mut dwErrorCode: DWORD = 0;
    let mut pDriverAligned: PBYTE;

    if bRPC != 0 {
        dwErrorCode = RpcImpersonateClient(core::ptr::null_mut()) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            return dwErrorCode as BOOL;
        }
    }

    pDriverAligned = (AlignRpcPtr((pDriver).cast(), core::ptr::addr_of_mut!(cbBuf))).cast();

    if GetPrinterDriverExW(
        hPrinter,
        pEnvironment,
        Level,
        pDriverAligned,
        cbBuf,
        pcbNeeded,
        dwClientMajorVersion,
        dwClientMinorVersion,
        pdwServerMajorVersion,
        pdwServerMinorVersion,
    ) != 0
    {
        // Replace relative offset addresses in the output by absolute pointers.
        MarshallDownStructure(
            (pDriverAligned).cast(),
            (*pPrinterDriverMarshalling[Level as usize]).pInfo.as_ptr(),
            (*pPrinterDriverMarshalling[Level as usize]).cbStructureSize,
            (TRUE) as BOOL,
        );
    } else {
        dwErrorCode = GetLastError();
    }

    if bRPC != 0 {
        RpcRevertToSelf();
    }
    UndoAlignRpcPtr((pDriver).cast(), (pDriverAligned).cast(), cbBuf, pcbNeeded);

    return dwErrorCode as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcGetPrinterDriver2(hPrinter: WINSPOOL_PRINTER_HANDLE, pEnvironment: *mut WCHAR, Level: DWORD, pDriver: *mut BYTE, mut cbBuf: DWORD, pcbNeeded: *mut DWORD, dwClientMajorVersion: DWORD, dwClientMinorVersion: DWORD, pdwServerMaxVersion: *mut DWORD, pdwServerMinVersion: *mut DWORD) -> DWORD {
    let mut dwErrorCode: DWORD;
    let mut pDriverAligned: PBYTE;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS
    {
        return dwErrorCode;
    }

    pDriverAligned = (AlignRpcPtr((pDriver).cast(), core::ptr::addr_of_mut!(cbBuf))).cast();

    if GetPrinterDriverExW(hPrinter, pEnvironment, Level, pDriverAligned, cbBuf, pcbNeeded, dwClientMajorVersion, dwClientMinorVersion, pdwServerMaxVersion, pdwServerMinVersion) != 0
    {
        // Replace relative offset addresses in the output by absolute pointers.
        MarshallDownStructure((pDriverAligned).cast(), (*pPrinterDriverMarshalling[Level as usize]).pInfo.as_ptr(), (*pPrinterDriverMarshalling[Level as usize]).cbStructureSize, (TRUE) as BOOL);
    }
    else
    {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    UndoAlignRpcPtr((pDriver).cast(), (pDriverAligned).cast(), cbBuf, pcbNeeded);

    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcGetPrinterDriverDirectory(pName: WINSPOOL_HANDLE, pEnvironment: LPWSTR, Level: DWORD, pDriverDirectory: LPBYTE, cbBuf: DWORD, pcbNeeded: LPDWORD) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS
    {
        return dwErrorCode;
    }

    if GetPrinterDriverDirectoryW(pName, pEnvironment, Level, pDriverDirectory, cbBuf, pcbNeeded) == 0 {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
