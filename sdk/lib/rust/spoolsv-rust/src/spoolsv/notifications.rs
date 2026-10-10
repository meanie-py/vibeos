use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "C" fn _RpcClientFindFirstPrinterChangeNotification() -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcFindClosePrinterChangeNotification() -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcFindNextPrinterChangeNotification() -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcRemoteFindFirstPrinterChangeNotification(
    hPrinter: WINSPOOL_PRINTER_HANDLE,
    fdwFlags: DWORD,
    fdwOptions: DWORD,
    pszLocalMachine: *mut WCHAR,
    dwPrinterLocal: DWORD,
    cbBuffer: DWORD,
    pBuffer: *mut BYTE,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcRemoteFindFirstPrinterChangeNotificationEx(
    hPrinter: WINSPOOL_PRINTER_HANDLE,
    fdwFlags: DWORD,
    fdwOptions: DWORD,
    pszLocalMachine: *mut WCHAR,
    dwPrinterLocal: DWORD,
    pOptions: *mut WINSPOOL_V2_NOTIFY_OPTIONS,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcReplyClosePrinter(
    phNotify: *mut WINSPOOL_PRINTER_HANDLE,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcReplyOpenPrinter(
    pMachine: WINSPOOL_HANDLE,
    phPrinterNotify: *mut WINSPOOL_PRINTER_HANDLE,
    dwPrinterRemote: DWORD,
    dwType: DWORD,
    cbBuffer: DWORD,
    pBuffer: *mut BYTE,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcRouterFindFirstPrinterChangeNotificationOld() -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcRouterRefreshPrinterChangeNotification(
    hPrinter: WINSPOOL_PRINTER_HANDLE,
    dwColor: DWORD,
    pOptions: *mut WINSPOOL_V2_NOTIFY_OPTIONS,
    ppInfo: *mut *mut WINSPOOL_V2_NOTIFY_INFO,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcRouterReplyPrinter(
    hNotify: WINSPOOL_PRINTER_HANDLE,
    fdwFlags: DWORD,
    cbBuffer: DWORD,
    pBuffer: *mut BYTE,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcRouterReplyPrinterEx(
    hNotify: WINSPOOL_PRINTER_HANDLE,
    dwColor: DWORD,
    fdwFlags: DWORD,
    pdwResult: *mut DWORD,
    dwReplyType: DWORD,
    Reply: WINSPOOL_V2_UREPLY_PRINTER,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcWaitForPrinterChange(
    hPrinter: WINSPOOL_PRINTER_HANDLE,
    Flags: DWORD,
    pFlags: *mut DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}
