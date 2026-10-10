// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_05CF8A9FE74B: [u16; 18] = [
    0x0050u16, 0x0072, 0x0069, 0x006E, 0x0074, 0x0065, 0x0072, 0x0044, 0x0072, 0x0069,
    0x0076, 0x0065, 0x0072, 0x0044, 0x0061, 0x0074, 0x0061, 0x0000,
];

use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "C" fn _RpcDeletePrinterData(
    hPrinter: WINSPOOL_PRINTER_HANDLE,
    pValueName: *mut WCHAR,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcDeletePrinterDataEx(
    hPrinter: WINSPOOL_PRINTER_HANDLE,
    pKeyName: *const WCHAR,
    pValueName: *const WCHAR,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcDeletePrinterKey(
    hPrinter: WINSPOOL_PRINTER_HANDLE,
    pKeyName: *const WCHAR,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcEnumPrinterData(
    hPrinter: WINSPOOL_PRINTER_HANDLE,
    dwIndex: DWORD,
    pValueName: *mut WCHAR,
    cbValueName: DWORD,
    pcbValueName: *mut DWORD,
    pType: *mut DWORD,
    pData: *mut BYTE,
    cbData: DWORD,
    pcbData: *mut DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcEnumPrinterKey(
    hPrinter: WINSPOOL_PRINTER_HANDLE,
    pKeyName: *const WCHAR,
    pSubkey: *mut WCHAR,
    cbSubkey: DWORD,
    pcbSubkey: *mut DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}

#[no_mangle]
pub unsafe extern "C" fn _RpcEnumPrinterDataEx(
    hPrinter: WINSPOOL_PRINTER_HANDLE,
    pKeyName: *const WCHAR,
    pEnumValues: *mut BYTE,
    cbEnumValues: DWORD,
    pcbEnumValues: *mut DWORD,
    pnEnumValues: *mut DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_INVALID_FUNCTION
}
#[no_mangle]
pub unsafe extern "C" fn _RpcGetPrinterData(
    mut hPrinter: WINSPOOL_PRINTER_HANDLE,
    mut pValueName: *mut WCHAR,
    mut pType: *mut DWORD,
    mut pData: *mut BYTE,
    mut nSize: DWORD,
    mut pcbNeeded: *mut DWORD,
) -> DWORD {
    return _RpcGetPrinterDataEx(
        hPrinter,
        RUSTCONV_WSTR_05CF8A9FE74B.as_ptr(),
        pValueName,
        pType,
        pData,
        nSize,
        pcbNeeded,
    );
}
#[no_mangle]
pub unsafe extern "C" fn _RpcGetPrinterDataEx(
    hPrinter: WINSPOOL_PRINTER_HANDLE,
    pKeyName: *const WCHAR,
    pValueName: *const WCHAR,
    pType: *mut DWORD,
    pData: *mut BYTE,
    nSize: DWORD,
    pcbNeeded: *mut DWORD,
) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = RpcImpersonateClient(core::ptr::null_mut()) as DWORD;
    if dwErrorCode != ERROR_SUCCESS {
        // ERR debug-channel trace ("RpcImpersonateClient failed with error %lu!")
        // elided: its debug-channel symbols are not available in this translation context.
        return dwErrorCode;
    }

    dwErrorCode = GetPrinterDataExW(
        hPrinter as HANDLE,
        pKeyName as LPCWSTR,
        pValueName as LPCWSTR,
        pType as LPDWORD,
        pData as LPBYTE,
        nSize,
        pcbNeeded as LPDWORD,
    );

    RpcRevertToSelf();

    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcSetPrinterData(
    hPrinter: WINSPOOL_PRINTER_HANDLE,
    pValueName: *mut WCHAR,
    Type: DWORD,
    pData: *mut BYTE,
    cbData: DWORD,
) -> DWORD {
    return _RpcSetPrinterDataEx(hPrinter, RUSTCONV_WSTR_05CF8A9FE74B.as_ptr() as *const WCHAR, pValueName, Type, pData, cbData);
}
#[no_mangle]
pub unsafe extern "C" fn _RpcSetPrinterDataEx(hPrinter: WINSPOOL_PRINTER_HANDLE, pKeyName: *const WCHAR, pValueName: *const WCHAR, Type: DWORD, pData: *mut BYTE, cbData: DWORD) -> DWORD {
    let mut dwErrorCode: DWORD = 0;
    dwErrorCode = RpcImpersonateClient(core::ptr::null_mut()) as DWORD;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }
    dwErrorCode = SetPrinterDataExW(hPrinter, pKeyName, pValueName, Type, pData, cbData);
    RpcRevertToSelf();
    return dwErrorCode;
}
