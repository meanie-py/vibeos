use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "C" fn _RpcCreatePrinterIC(
    hPrinter: WINSPOOL_PRINTER_HANDLE,
    pHandle: *mut WINSPOOL_GDI_HANDLE,
    pDevModeContainer: *mut WINSPOOL_DEVMODE_CONTAINER,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcPlayGdiScriptOnPrinterIC(
    hPrinterIC: WINSPOOL_GDI_HANDLE,
    pIn: *mut BYTE,
    cIn: DWORD,
    pOut: *mut BYTE,
    cOut: DWORD,
    ul: DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcDeletePrinterIC(phPrinterIC: *mut WINSPOOL_GDI_HANDLE) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcAddPrinterConnection(pName: WINSPOOL_HANDLE) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcDeletePrinterConnection(pName: WINSPOOL_HANDLE) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}
#[no_mangle]
pub unsafe extern "C" fn _RpcPrinterMessageBox(
    mut hPrinter: WINSPOOL_PRINTER_HANDLE,
    mut Error: DWORD,
    mut hWnd: ULONG_PTR,
    mut pText: *mut WCHAR,
    mut pCaption: *mut WCHAR,
    mut dwType: DWORD,
) -> DWORD {
    let mut dwErrorCode: DWORD = 0;
    dwErrorCode = RpcImpersonateClient(core::ptr::null_mut()) as DWORD;
    if dwErrorCode != ERROR_SUCCESS as DWORD {
        return dwErrorCode;
    }
    PrinterMessageBoxW(
        hPrinter as HANDLE,
        Error,
        hWnd as HWND,
        pText as LPWSTR,
        pCaption as LPWSTR,
        dwType,
    );
    dwErrorCode = GetLastError();
    RpcRevertToSelf();
    return dwErrorCode;
}

#[no_mangle]
pub unsafe extern "C" fn _RpcSetAllocFailCount() -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcClusterSplOpen() -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcClusterSplClose() -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcClusterSplIsAlive() -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcAddPerMachineConnection(
    pServer: WINSPOOL_HANDLE,
    pPrinterName: *const WCHAR,
    pPrintServer: *const WCHAR,
    pProvider: *const WCHAR,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcDeletePerMachineConnection(
    pServer: WINSPOOL_HANDLE,
    pPrinterName: *const WCHAR,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcEnumPerMachineConnections(
    pServer: WINSPOOL_HANDLE,
    pPrinterEnum: *mut BYTE,
    cbBuf: DWORD,
    pcbNeeded: *mut DWORD,
    pcReturned: *mut DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcSplOpenPrinter() -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcSendRecvBidiData(
    hPrinter: WINSPOOL_PRINTER_HANDLE,
    pAction: *const WCHAR,
    pReqData: *mut WINSPOOL_BIDI_REQUEST_CONTAINER,
    ppRespData: *mut *mut WINSPOOL_BIDI_RESPONSE_CONTAINER,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcAddDriverCatalog() -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}
