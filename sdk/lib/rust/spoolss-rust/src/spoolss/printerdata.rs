// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_05CF8A9FE74B: [u16; 18] = [
    0x0050u16, 0x0072, 0x0069, 0x006E, 0x0074, 0x0065, 0x0072, 0x0044, 0x0072, 0x0069,
    0x0076, 0x0065, 0x0072, 0x0044, 0x0061, 0x0074, 0x0061, 0x0000,
];

use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn DeletePrinterDataExW(hPrinter: HANDLE, pKeyName: PCWSTR, pValueName: PCWSTR) -> DWORD {
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpDeletePrinterDataEx
        .unwrap_unchecked()(
        (*pHandle).hPrinter,
        pKeyName as LPCWSTR,
        pValueName as LPCWSTR,
    );
}

#[no_mangle]
pub unsafe extern "system" fn DeletePrinterDataW(hPrinter: HANDLE, pValueName: PWSTR) -> DWORD {
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as DWORD;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpDeletePrinterData
        .unwrap_unchecked()((*pHandle).hPrinter, pValueName as LPWSTR);
}

#[no_mangle]
pub unsafe extern "system" fn DeletePrinterKeyW(hPrinter: HANDLE, pKeyName: PCWSTR) -> DWORD {
    let pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpDeletePrinterKey
        .unwrap_unchecked()((*pHandle).hPrinter, pKeyName as LPCWSTR);
}

#[no_mangle]
pub unsafe extern "system" fn EnumPrinterDataExW(
    hPrinter: HANDLE,
    pKeyName: PCWSTR,
    pEnumValues: PBYTE,
    cbEnumValues: DWORD,
    pcbEnumValues: PDWORD,
    pnEnumValues: PDWORD,
) -> DWORD {
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpEnumPrinterDataEx
        .unwrap_unchecked()(
        (*pHandle).hPrinter,
        pKeyName as LPCWSTR,
        pEnumValues as LPBYTE,
        cbEnumValues,
        pcbEnumValues as LPDWORD,
        pnEnumValues as LPDWORD,
    );
}

#[no_mangle]
pub unsafe extern "system" fn EnumPrinterDataW(hPrinter: HANDLE, dwIndex: DWORD, pValueName: PWSTR, cbValueName: DWORD, pcbValueName: PDWORD, pType: PDWORD, pData: PBYTE, cbData: DWORD, pcbData: PDWORD) -> DWORD {
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpEnumPrinterData
        .unwrap_unchecked()(
        (*pHandle).hPrinter,
        dwIndex,
        pValueName as LPWSTR,
        cbValueName,
        pcbValueName as LPDWORD,
        pType as LPDWORD,
        pData as LPBYTE,
        cbData,
        pcbData as LPDWORD,
    );
}

#[no_mangle]
pub unsafe extern "system" fn EnumPrinterKeyW(
    hPrinter: HANDLE,
    pKeyName: PCWSTR,
    pSubkey: PWSTR,
    cbSubkey: DWORD,
    pcbSubkey: PDWORD,
) -> DWORD {
    let pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpEnumPrinterKey
        .unwrap_unchecked()(
        (*pHandle).hPrinter,
        pKeyName as LPCWSTR,
        pSubkey as LPWSTR,
        cbSubkey,
        pcbSubkey as LPDWORD,
    );
}

#[no_mangle]
pub unsafe extern "system" fn GetPrinterDataExW(
    hPrinter: HANDLE,
    pKeyName: LPCWSTR,
    pValueName: LPCWSTR,
    pType: LPDWORD,
    pData: LPBYTE,
    nSize: DWORD,
    pcbNeeded: LPDWORD,
) -> DWORD {
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity check.
    if pHandle.is_null() {
        // Yes, Windows checks for the handle here and sets the last error to ERROR_INVALID_HANDLE,
        // but returns FALSE and not the error code.
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return FALSE as DWORD;
    }

    // Call GetPrinterDataEx of the Print Provider.
    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpGetPrinterDataEx
        .unwrap_unchecked()(
        (*pHandle).hPrinter,
        pKeyName,
        pValueName,
        pType,
        pData,
        nSize,
        pcbNeeded,
    );
}

#[no_mangle]
pub unsafe extern "system" fn GetPrinterDataW(hPrinter: HANDLE, pValueName: LPWSTR, pType: LPDWORD, pData: LPBYTE, nSize: DWORD, pcbNeeded: LPDWORD) -> DWORD {
    return GetPrinterDataExW(hPrinter, RUSTCONV_WSTR_05CF8A9FE74B.as_ptr(), pValueName, pType, pData, nSize, pcbNeeded);
}

#[no_mangle]
pub unsafe extern "system" fn SetPrinterDataExW(
    hPrinter: HANDLE,
    pKeyName: LPCWSTR,
    pValueName: LPCWSTR,
    Type: DWORD,
    pData: LPBYTE,
    cbData: DWORD,
) -> DWORD {
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity check.
    if pHandle.is_null() {
        // Yes, Windows checks for the handle here and sets the last error to ERROR_INVALID_HANDLE,
        // but returns FALSE and not the error code.
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return FALSE as DWORD;
    }

    // Call SetPrinterDataEx of the Print Provider.
    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpSetPrinterDataEx
        .unwrap_unchecked()(
        (*pHandle).hPrinter,
        pKeyName,
        pValueName,
        Type,
        pData,
        cbData,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SetPrinterDataW(hPrinter: HANDLE, pValueName: PWSTR, Type: DWORD, pData: PBYTE, cbData: DWORD) -> DWORD {
    return SetPrinterDataExW(hPrinter, RUSTCONV_WSTR_05CF8A9FE74B.as_ptr(), pValueName as LPCWSTR, Type, pData as LPBYTE, cbData);
}
