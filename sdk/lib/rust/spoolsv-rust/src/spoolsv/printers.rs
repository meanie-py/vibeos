use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn _RpcAbortPrinter(hPrinter: WINSPOOL_PRINTER_HANDLE) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    if AbortPrinter(hPrinter) == 0 {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}

#[no_mangle]
pub unsafe extern "C" fn _RpcAddPrinter(
    pName: WINSPOOL_HANDLE,
    pPrinterContainer: *mut WINSPOOL_PRINTER_CONTAINER,
    pDevModeContainer: *mut WINSPOOL_DEVMODE_CONTAINER,
    pSecurityContainer: *mut WINSPOOL_SECURITY_CONTAINER,
    pHandle: *mut WINSPOOL_PRINTER_HANDLE,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcAddPrinterEx(
    pName: WINSPOOL_HANDLE,
    pPrinterContainer: *mut WINSPOOL_PRINTER_CONTAINER,
    pDevModeContainer: *mut WINSPOOL_DEVMODE_CONTAINER,
    pSecurityContainer: *mut WINSPOOL_SECURITY_CONTAINER,
    pClientInfo: *mut WINSPOOL_SPLCLIENT_CONTAINER,
    pHandle: *mut WINSPOOL_PRINTER_HANDLE,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}
#[no_mangle]
pub unsafe extern "C" fn _RpcClosePrinter(phPrinter: *mut WINSPOOL_PRINTER_HANDLE) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    if ClosePrinter(*phPrinter) != 0 {
        *phPrinter = core::ptr::null_mut();
    } else {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcDeletePrinter(hPrinter: WINSPOOL_PRINTER_HANDLE) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    if DeletePrinter(hPrinter) == 0 {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcEndDocPrinter(hPrinter: WINSPOOL_PRINTER_HANDLE) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    if EndDocPrinter(hPrinter) == 0 {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcEndPagePrinter(hPrinter: WINSPOOL_PRINTER_HANDLE) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS
    {
        return dwErrorCode;
    }

    if EndPagePrinter(hPrinter) == 0 {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcEnumPrinters(Flags: DWORD, Name: WINSPOOL_HANDLE, Level: DWORD, pPrinterEnum: *mut u8, mut cbBuf: DWORD, pcbNeeded: *mut DWORD, pcReturned: *mut DWORD) -> DWORD {
    let mut dwErrorCode: DWORD;
    let mut pPrinterEnumAligned: PBYTE;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    pPrinterEnumAligned = (AlignRpcPtr((pPrinterEnum).cast(), core::ptr::addr_of_mut!(cbBuf))).cast();

    if EnumPrintersW(Flags, Name, Level, pPrinterEnumAligned, cbBuf, pcbNeeded, pcReturned) != 0 {
        // Replace absolute pointer addresses in the output by relative offsets.

        MarshallDownStructuresArray((pPrinterEnumAligned).cast(), *pcReturned, (*pPrinterInfoMarshalling[Level as usize]).pInfo.as_ptr(), (*pPrinterInfoMarshalling[Level as usize]).cbStructureSize, (TRUE) as BOOL);
    } else {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    UndoAlignRpcPtr((pPrinterEnum).cast(), (pPrinterEnumAligned).cast(), cbBuf, pcbNeeded);

    return dwErrorCode;
}

#[no_mangle]
pub unsafe extern "C" fn _RpcFlushPrinter(
    hPrinter: WINSPOOL_PRINTER_HANDLE,
    pBuf: *mut BYTE,
    cbBuf: DWORD,
    pcWritten: *mut DWORD,
    cSleep: DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}
#[no_mangle]
pub unsafe extern "C" fn _RpcGetPrinter(hPrinter: WINSPOOL_PRINTER_HANDLE, Level: DWORD, pPrinter: *mut BYTE, mut cbBuf: DWORD, pcbNeeded: *mut DWORD) -> DWORD {
    let mut dwErrorCode: DWORD;
    let mut pPrinterAligned: PBYTE;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS
    {
        return dwErrorCode;
    }

    pPrinterAligned = (AlignRpcPtr((pPrinter).cast(), core::ptr::addr_of_mut!(cbBuf))).cast();

    if GetPrinterW(hPrinter, Level, pPrinterAligned, cbBuf, pcbNeeded) != 0
    {
        // Replace absolute pointer addresses in the output by relative offsets.

        MarshallDownStructure((pPrinterAligned).cast(), (*pPrinterInfoMarshalling[Level as usize]).pInfo.as_ptr(), (*pPrinterInfoMarshalling[Level as usize]).cbStructureSize, (TRUE) as BOOL);
    }
    else
    {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    UndoAlignRpcPtr((pPrinter).cast(), (pPrinterAligned).cast(), cbBuf, pcbNeeded);

    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcOpenPrinter(pPrinterName: WINSPOOL_HANDLE, phPrinter: *mut WINSPOOL_PRINTER_HANDLE, pDatatype: *mut WCHAR, pDevModeContainer: *mut WINSPOOL_DEVMODE_CONTAINER, AccessRequired: DWORD) -> DWORD {
    let mut dwErrorCode: DWORD;
    let mut Default: PRINTER_DEFAULTSW = core::mem::zeroed();

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    Default.DesiredAccess = AccessRequired;
    Default.pDatatype = pDatatype;
    Default.pDevMode = (*pDevModeContainer).pDevMode as PDEVMODEW;

    if OpenPrinterW(pPrinterName, phPrinter, core::ptr::addr_of_mut!(Default)) == 0 {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}

#[no_mangle]
pub unsafe extern "C" fn _RpcOpenPrinterEx(
    pPrinterName: WINSPOOL_HANDLE,
    pHandle: *mut WINSPOOL_PRINTER_HANDLE,
    pDatatype: *mut WCHAR,
    pDevModeContainer: *mut WINSPOOL_DEVMODE_CONTAINER,
    AccessRequired: DWORD,
    pClientInfo: *mut WINSPOOL_SPLCLIENT_CONTAINER,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}
#[no_mangle]
pub unsafe extern "C" fn _RpcReadPrinter(hPrinter: WINSPOOL_PRINTER_HANDLE, pBuf: *mut BYTE, cbBuf: DWORD, pcNoBytesRead: *mut DWORD) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = RpcImpersonateClient(core::ptr::null_mut()) as DWORD;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    if ReadPrinter(hPrinter, (pBuf).cast(), cbBuf, pcNoBytesRead) == 0 {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}

#[no_mangle]
pub unsafe extern "C" fn _RpcResetPrinter(
    hPrinter: WINSPOOL_PRINTER_HANDLE,
    pDatatype: *mut WCHAR,
    pDevModeContainer: *mut WINSPOOL_DEVMODE_CONTAINER,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}
#[no_mangle]
pub unsafe extern "C" fn _RpcResetPrinterEx(hPrinter: HANDLE, pDatatype: PWSTR, pDevModeContainer: *mut WINSPOOL_DEVMODE_CONTAINER, dwFlags: DWORD) -> DWORD {
    let mut dwErrorCode: DWORD = 0;
    let mut pdw: PRINTER_DEFAULTSW = core::mem::zeroed();

    if !pDatatype.is_null() {
        pdw.pDatatype = pDatatype;
    } else {
        pdw.pDatatype = if (dwFlags & RESETPRINTERDEFAULTDATATYPE) != 0 {
            (-(1 as isize)) as usize as PWSTR
        } else {
            core::ptr::null_mut()
        };
    }

    if !(*pDevModeContainer).pDevMode.is_null() {
        pdw.pDevMode = (*pDevModeContainer).pDevMode as PDEVMODEW;
        // Fixme : Need to check DevMode before forward call, by copy devmode.c from WinSpool.
        //         Local SV!SplIsValidDevmode((PDW)pDevModeContainer->pDevMode, pDevModeContainer->cbBuf)
    } else {
        pdw.pDevMode = if (dwFlags & RESETPRINTERDEFAULTDEVMODE) != 0 {
            (-(1 as isize)) as usize as PDEVMODEW
        } else {
            core::ptr::null_mut()
        };
    }
    pdw.DesiredAccess = 0;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    if ResetPrinterW(hPrinter, core::ptr::addr_of_mut!(pdw)) == 0 {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcSeekPrinter(hPrinter: WINSPOOL_PRINTER_HANDLE, liDistanceToMove: LARGE_INTEGER, pliNewPointer: PLARGE_INTEGER, dwMoveMethod: DWORD, bWrite: BOOL) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS
    {

        return dwErrorCode;
    }

    if SeekPrinter(hPrinter, liDistanceToMove, pliNewPointer, dwMoveMethod, bWrite) == 0
        { dwErrorCode = GetLastError(); }

    RpcRevertToSelf();
    return dwErrorCode;
}

#[no_mangle]
pub unsafe extern "C" fn _RpcSetPrinter(
    hPrinter: WINSPOOL_PRINTER_HANDLE,
    pPrinterContainer: *mut WINSPOOL_PRINTER_CONTAINER,
    pDevModeContainer: *mut WINSPOOL_DEVMODE_CONTAINER,
    pSecurityContainer: *mut WINSPOOL_SECURITY_CONTAINER,
    Command: DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}
#[no_mangle]
pub unsafe extern "C" fn _RpcStartDocPrinter(hPrinter: WINSPOOL_PRINTER_HANDLE, pDocInfoContainer: *mut WINSPOOL_DOC_INFO_CONTAINER, pJobId: *mut DWORD) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    *pJobId = StartDocPrinterW(hPrinter, (*pDocInfoContainer).Level, ((*pDocInfoContainer).DocInfo.pDocInfo1) as PBYTE);
    dwErrorCode = GetLastError();

    RpcRevertToSelf();
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcStartPagePrinter(hPrinter: WINSPOOL_PRINTER_HANDLE) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    if StartPagePrinter(hPrinter) == 0 {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcWritePrinter(hPrinter: HANDLE, pBuf: PBYTE, cbBuf: DWORD, pcWritten: *mut DWORD) -> DWORD {
    let mut dwErrorCode: DWORD = 0;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    if WritePrinter(hPrinter, (pBuf).cast(), cbBuf, pcWritten) == 0 {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
