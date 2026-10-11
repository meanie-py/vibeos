use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn SplGetSpoolFileInfo(
    hPrinter: HANDLE,
    hProcessHandle: HANDLE,
    Level: DWORD,
    pFileInfo: PFILE_INFO_1,
    dwSize: DWORD,
    dwNeeded: *mut DWORD,
) -> BOOL {
    let mut Ret: BOOL = 0;
    let mut hHandle: HANDLE = core::ptr::null_mut();
    let mut hSourceProcessHandle: HANDLE = core::ptr::null_mut();
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }

    hSourceProcessHandle = GetCurrentProcess();

    // No Local? Ok, what ever...

    Ret = (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpGetSpoolFileInfo
        .unwrap_unchecked()(
        (*pHandle).hPrinter,
        core::ptr::null_mut::<LPWSTR>(),
        core::ptr::addr_of_mut!(hHandle),
        hProcessHandle,
        hSourceProcessHandle,
    );
    if Ret != 0 {
        (*pFileInfo).hSpoolFileHandle = hHandle;
        (*pFileInfo).bInheritHandle = TRUE as BOOL;
        (*pFileInfo).dwOptions = DUPLICATE_CLOSE_SOURCE as DWORD;
    }

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn SplCommitSpoolData(
    hPrinter: HANDLE,
    hProcessHandle: HANDLE,
    cbCommit: DWORD,
    Level: DWORD,
    pFileInfo: PFILE_INFO_1,
    dwSize: DWORD,
    dwNeeded: *mut DWORD,
) -> BOOL {
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }

    (*pFileInfo).hSpoolFileHandle = INVALID_HANDLE_VALUE;
    (*pFileInfo).bInheritHandle = TRUE as BOOL;
    (*pFileInfo).dwOptions = DUPLICATE_CLOSE_SOURCE as DWORD;

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpCommitSpoolData
        .unwrap_unchecked()(hPrinter, cbCommit);
}

#[no_mangle]
pub unsafe extern "system" fn SplCloseSpoolFileHandle(hPrinter: HANDLE) -> BOOL {
    let pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }
    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpCloseSpoolFileHandle
        .unwrap_unchecked()((*pHandle).hPrinter);
}
