use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn AddJobW(
    hPrinter: HANDLE,
    Level: DWORD,
    pData: PBYTE,
    cbBuf: DWORD,
    pcbNeeded: PDWORD,
) -> BOOL {
    let pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpAddJob
        .unwrap_unchecked()(
        (*pHandle).hPrinter,
        Level,
        pData,
        cbBuf,
        pcbNeeded,
    );
}

#[no_mangle]
pub unsafe extern "system" fn EnumJobsW(
    hPrinter: HANDLE,
    FirstJob: DWORD,
    NoJobs: DWORD,
    Level: DWORD,
    pJob: PBYTE,
    cbBuf: DWORD,
    pcbNeeded: PDWORD,
    pcReturned: PDWORD,
) -> BOOL {
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpEnumJobs
        .unwrap_unchecked()(
        (*pHandle).hPrinter,
        FirstJob,
        NoJobs,
        Level,
        pJob,
        cbBuf,
        pcbNeeded,
        pcReturned,
    );
}

#[no_mangle]
pub unsafe extern "system" fn GetJobW(
    hPrinter: HANDLE,
    JobId: DWORD,
    Level: DWORD,
    pJob: PBYTE,
    cbBuf: DWORD,
    pcbNeeded: PDWORD,
) -> BOOL {
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpGetJob
        .unwrap_unchecked()(
        (*pHandle).hPrinter,
        JobId,
        Level,
        pJob as LPBYTE,
        cbBuf,
        pcbNeeded as LPDWORD,
    ) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ScheduleJob(hPrinter: HANDLE, dwJobID: DWORD) -> BOOL {
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpScheduleJob
        .unwrap_unchecked()((*pHandle).hPrinter, dwJobID) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetJobW(hPrinter: HANDLE, JobId: DWORD, Level: DWORD, pJobInfo: PBYTE, Command: DWORD) -> BOOL {
    let mut pHandle: PSPOOLSS_PRINTER_HANDLE = hPrinter as PSPOOLSS_PRINTER_HANDLE;

    // Sanity checks.
    if pHandle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }

    return (*(*pHandle).pPrintProvider)
        .PrintProvider
        .fpSetJob
        .unwrap_unchecked()(
        (*pHandle).hPrinter,
        JobId,
        Level,
        pJobInfo,
        Command,
    ) as BOOL;
}
