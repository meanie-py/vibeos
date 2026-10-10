use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn AddFormW(hPrinter: HANDLE, Level: DWORD, pForm: PBYTE) -> BOOL {
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpAddForm
        .unwrap_unchecked()((*pHandle).hPrinter, Level, pForm as LPBYTE)
        as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn DeleteFormW(hPrinter: HANDLE, pFormName: PWSTR) -> BOOL {
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpDeleteForm
        .unwrap_unchecked()((*pHandle).hPrinter, pFormName) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn EnumFormsW(
    hPrinter: HANDLE,
    Level: DWORD,
    pForm: PBYTE,
    cbBuf: DWORD,
    pcbNeeded: PDWORD,
    pcReturned: PDWORD,
) -> BOOL {
    let pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    if cbBuf != 0 && pForm.is_null() {
        SetLastError(ERROR_INVALID_USER_BUFFER);
        return FALSE as BOOL;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpEnumForms
        .unwrap_unchecked()(
        (*pHandle).hPrinter,
        Level,
        pForm as LPBYTE,
        cbBuf,
        pcbNeeded as LPDWORD,
        pcReturned as LPDWORD,
    ) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetFormW(hPrinter: HANDLE, pFormName: PWSTR, Level: DWORD, pForm: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD) -> BOOL {
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    if cbBuf != 0 && !pForm.is_null() {
        SetLastError(ERROR_INVALID_USER_BUFFER);
        return FALSE as BOOL;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpGetForm
        .unwrap_unchecked()(
        (*pHandle).hPrinter,
        pFormName,
        Level,
        pForm,
        cbBuf,
        pcbNeeded,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SetFormW(hPrinter: HANDLE, pFormName: PWSTR, Level: DWORD, pForm: PBYTE) -> BOOL {
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpSetForm
        .unwrap_unchecked()(
        (*pHandle).hPrinter,
        pFormName,
        Level,
        pForm,
    );
}
