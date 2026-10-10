use crate::winspool::marshalling::MarshallUpStructure;
use crate::winspool::marshalling::MarshallUpStructuresArray;
use crate::winspool::devmode::RosConvertUnicodeDevModeToAnsiDevmode;
use crate::winspool::utils::UnicodeToAnsiInPlace;
use crate::winspool::utils::UpdateTrayIcon;
use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: usize) -> *mut core::ffi::c_void;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: *mut core::ffi::c_void) -> BOOL;
    fn GetProcessHeap() -> HANDLE;
    fn GdiConvertToDevmodeW(pDevmodeA: *const core::ffi::c_void) -> *mut core::ffi::c_void;
    fn SetLastError(dwErrCode: DWORD);
}

#[no_mangle]
pub unsafe extern "system" fn AddJobA(hPrinter: HANDLE, Level: DWORD, pData: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD) -> BOOL {
    let mut ret: BOOL;

    if Level != 1 {
        SetLastError(ERROR_INVALID_LEVEL);
        return FALSE as BOOL;
    }

    ret = AddJobW(hPrinter, Level, pData, cbBuf, pcbNeeded);

    if ret != 0 {
        let mut dwErrorCode: DWORD;
        let addjobW: *mut ADDJOB_INFO_1W = pData as *mut ADDJOB_INFO_1W;
        dwErrorCode = UnicodeToAnsiInPlace((*addjobW).Path);
        if dwErrorCode != ERROR_SUCCESS {
            ret = FALSE as BOOL;
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn AddJobW(hPrinter: HANDLE, Level: DWORD, pData: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD) -> WINBOOL {
    let mut dwErrorCode: DWORD = 0;
    let pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;

    'quit: {
        if pHandle.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'quit;
        }

        // Do the RPC call
        // RpcTryExcept
        {
            dwErrorCode = _RpcAddJob((*pHandle).hPrinter, Level, pData, cbBuf, pcbNeeded);
        }
        // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
        // Reached only when the RPC call raises an exception (SEH path).
        if false {
            dwErrorCode = RpcExceptionCode!() as DWORD;
        }
        // RpcEndExcept;

        if dwErrorCode == ERROR_SUCCESS {
            let pji1w: *mut JOB_INFO_1W = pData as *mut JOB_INFO_1W;

            // Replace relative offset addresses in the output by absolute pointers.
            MarshallUpStructure(cbBuf, pData as PVOID, AddJobInfo1Marshalling.pInfo.as_ptr(), AddJobInfo1Marshalling.cbStructureSize, TRUE as BOOL);
            (*pHandle).set_bJob(TRUE as BOOL);
            UpdateTrayIcon(hPrinter, (*pji1w).JobId);
        }
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn EnumJobsA(hPrinter: HANDLE, FirstJob: DWORD, NoJobs: DWORD, Level: DWORD, pJob: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD, pcReturned: PDWORD) -> BOOL {
    let mut dwErrorCode: DWORD;
    let mut i: DWORD;
    let mut pji1w: *mut JOB_INFO_1W = pJob as *mut JOB_INFO_1W;
    let mut pji2a: *mut JOB_INFO_2A = pJob as *mut JOB_INFO_2A;
    let mut pji2w: *mut JOB_INFO_2W = pJob as *mut JOB_INFO_2W;

    'cleanup: {
        if Level == 3 {
            return EnumJobsW(hPrinter, FirstJob, NoJobs, Level, pJob, cbBuf, pcbNeeded, pcReturned);
        }

        if EnumJobsW(hPrinter, FirstJob, NoJobs, Level, pJob, cbBuf, pcbNeeded, pcReturned) != 0 {
            if Level == 1 {
                i = 0;
                while i < *pcReturned {
                    dwErrorCode = UnicodeToAnsiInPlace((*pji1w.add(i as usize)).pPrinterName);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji1w.add(i as usize)).pMachineName);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji1w.add(i as usize)).pUserName);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji1w.add(i as usize)).pDocument);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji1w.add(i as usize)).pDatatype);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji1w.add(i as usize)).pStatus);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    i += 1;
                }
            } else if Level == 2 {
                i = 0;
                while i < *pcReturned {
                    dwErrorCode = UnicodeToAnsiInPlace((*pji2w.add(i as usize)).pPrinterName);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji2w.add(i as usize)).pMachineName);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji2w.add(i as usize)).pUserName);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji2w.add(i as usize)).pDocument);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji2w.add(i as usize)).pNotifyName);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji2w.add(i as usize)).pDatatype);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji2w.add(i as usize)).pPrintProcessor);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji2w.add(i as usize)).pParameters);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji2w.add(i as usize)).pStatus);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    if !(*pji2w.add(i as usize)).pDevMode.is_null() {
                        RosConvertUnicodeDevModeToAnsiDevmode((*pji2w.add(i as usize)).pDevMode, (*pji2a.add(i as usize)).pDevMode);
                    }
                    i += 1;
                }
            }
            return TRUE as BOOL;
        }
    }
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn EnumJobsW(hPrinter: HANDLE, FirstJob: DWORD, NoJobs: DWORD, Level: DWORD, pJob: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD, pcReturned: PDWORD) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;

    'Cleanup: {
        if pHandle.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        // Do the RPC call
        // RpcTryExcept
        dwErrorCode = _RpcEnumJobs((*pHandle).hPrinter, FirstJob, NoJobs, Level, pJob, cbBuf, pcbNeeded, pcReturned);
        // RpcExcept(EXCEPTION_EXECUTE_HANDLER): handler is reached only via SEH unwind,
        // which the integration layer owns; kept here verbatim as a dead branch.
        if false {
            dwErrorCode = RpcExceptionCode!() as DWORD;
        }
        // RpcEndExcept;

        if dwErrorCode == ERROR_SUCCESS {
            // Replace relative offset addresses in the output by absolute pointers for JOB_INFO_1W and JOB_INFO_2W.
            if Level <= 2 {
                MarshallUpStructuresArray(cbBuf, pJob as PVOID, *pcReturned, (*pJobInfoMarshalling[Level as usize]).pInfo.as_ptr(), (*pJobInfoMarshalling[Level as usize]).cbStructureSize, TRUE as BOOL);
            }
        }
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetJobA(hPrinter: HANDLE, JobId: DWORD, Level: DWORD, pJob: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let pji1w: *mut JOB_INFO_1W = pJob as *mut JOB_INFO_1W;
    let pji2a: *mut JOB_INFO_2A = pJob as *mut JOB_INFO_2A;
    let pji2w: *mut JOB_INFO_2W = pJob as *mut JOB_INFO_2W;

    'cleanup: {
        if Level == 3 {
            return GetJobW(hPrinter, JobId, Level, pJob, cbBuf, pcbNeeded);
        }

        if GetJobW(hPrinter, JobId, Level, pJob, cbBuf, pcbNeeded) != 0 {
            match Level {
                1 => {
                    dwErrorCode = UnicodeToAnsiInPlace((*pji1w).pPrinterName);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji1w).pMachineName);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji1w).pUserName);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji1w).pDocument);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji1w).pDatatype);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji1w).pStatus);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                }

                2 => {
                    dwErrorCode = UnicodeToAnsiInPlace((*pji2w).pPrinterName);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji2w).pMachineName);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji2w).pUserName);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji2w).pDocument);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji2w).pNotifyName);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji2w).pDatatype);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji2w).pPrintProcessor);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji2w).pParameters);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    dwErrorCode = UnicodeToAnsiInPlace((*pji2w).pStatus);
                    if dwErrorCode != ERROR_SUCCESS {
                        break 'cleanup;
                    }
                    if !(*pji2w).pDevMode.is_null() {
                        RosConvertUnicodeDevModeToAnsiDevmode((*pji2w).pDevMode, (*pji2a).pDevMode);
                    }
                }

                _ => {}
            }
            return TRUE as BOOL;
        }
    }
    // Cleanup:
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn GetJobW(hPrinter: HANDLE, JobId: DWORD, Level: DWORD, pJob: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD) -> WINBOOL {
    let mut dwErrorCode: DWORD;
    let pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;

    'Cleanup: {
        if pHandle.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        // Do the RPC call
        // RpcTryExcept
        {
            dwErrorCode = _RpcGetJob((*pHandle).hPrinter, JobId, Level, pJob, cbBuf, pcbNeeded);
        }
        // RpcExcept(EXCEPTION_EXECUTE_HANDLER): structured exception handling has no Rust
        // equivalent; the handler is kept, guarded so it is never entered.
        let rpc_faulted: bool = false;
        if rpc_faulted {
            dwErrorCode = RpcExceptionCode!() as DWORD;
        }
        // RpcEndExcept;

        if dwErrorCode == ERROR_SUCCESS {
            // Replace relative offset addresses in the output by absolute pointers.
            MarshallUpStructure(cbBuf, pJob as PVOID, (*pJobInfoMarshalling[Level as usize]).pInfo.as_ptr(), (*pJobInfoMarshalling[Level as usize]).cbStructureSize, TRUE as BOOL);
        }
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ScheduleJob(hPrinter: HANDLE, dwJobID: DWORD) -> WINBOOL {
    let mut dwErrorCode: DWORD;
    let pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;

    'Cleanup: {
        if pHandle.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        // Do the RPC call
        // RpcTryExcept
        {
            dwErrorCode = _RpcScheduleJob((*pHandle).hPrinter, dwJobID);
        }
        // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
        let rpc_exception_raised: bool = false;
        if rpc_exception_raised {
            dwErrorCode = RpcExceptionCode!() as DWORD;
        }
        // RpcEndExcept;

        if dwErrorCode == ERROR_SUCCESS {
            (*pHandle).set_bJob(FALSE);
        }
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    (dwErrorCode == ERROR_SUCCESS) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn SetJobA(hPrinter: HANDLE, JobId: DWORD, Level: DWORD, pJobInfo: PBYTE, Command: DWORD) -> BOOL {
    let ret: BOOL;
    let mut JobW: LPBYTE = core::ptr::null_mut();
    let mut usBuffer: _UNICODE_STRING = core::mem::zeroed();

    /* JobId, pPrinterName, pMachineName, pDriverName, Size, Submitted, Time and TotalPages
       are all ignored by SetJob, so we don't bother copying them */
    match Level {
        0 => {
            JobW = core::ptr::null_mut();
        }
        1 => {
            let info1A: *mut JOB_INFO_1A = pJobInfo as *mut JOB_INFO_1A;
            let info1W: *mut JOB_INFO_1W = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<JOB_INFO_1W>()) as *mut JOB_INFO_1W;
            ZeroMemory!(info1W as *mut core::ffi::c_void, core::mem::size_of::<JOB_INFO_1W>());

            JobW = info1W as LPBYTE;
            (*info1W).pUserName = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*info1A).pUserName);
            (*info1W).pDocument = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*info1A).pDocument);
            (*info1W).pDatatype = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*info1A).pDatatype);
            (*info1W).pStatus = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*info1A).pStatus);
            (*info1W).Status = (*info1A).Status;
            (*info1W).Priority = (*info1A).Priority;
            (*info1W).Position = (*info1A).Position;
            (*info1W).PagesPrinted = (*info1A).PagesPrinted;
        }
        2 => {
            let info2A: *mut JOB_INFO_2A = pJobInfo as *mut JOB_INFO_2A;
            let info2W: *mut JOB_INFO_2W = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<JOB_INFO_2W>()) as *mut JOB_INFO_2W;
            ZeroMemory!(info2W as *mut core::ffi::c_void, core::mem::size_of::<JOB_INFO_2W>());

            JobW = info2W as LPBYTE;
            (*info2W).pUserName = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*info2A).pUserName);
            (*info2W).pDocument = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*info2A).pDocument);
            (*info2W).pNotifyName = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*info2A).pNotifyName);
            (*info2W).pDatatype = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*info2A).pDatatype);
            (*info2W).pPrintProcessor = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*info2A).pPrintProcessor);
            (*info2W).pParameters = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*info2A).pParameters);
            (*info2W).pDevMode = if !(*info2A).pDevMode.is_null() {
                GdiConvertToDevmodeW((*info2A).pDevMode as *const core::ffi::c_void) as _
            } else {
                core::ptr::null_mut()
            };
            (*info2W).pStatus = AsciiToUnicode(core::ptr::addr_of_mut!(usBuffer), (*info2A).pStatus);
            (*info2W).pSecurityDescriptor = (*info2A).pSecurityDescriptor;
            (*info2W).Status = (*info2A).Status;
            (*info2W).Priority = (*info2A).Priority;
            (*info2W).Position = (*info2A).Position;
            (*info2W).StartTime = (*info2A).StartTime;
            (*info2W).UntilTime = (*info2A).UntilTime;
            (*info2W).PagesPrinted = (*info2A).PagesPrinted;
        }
        3 => {
            JobW = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<JOB_INFO_3>()) as LPBYTE;
            memcpy(JobW as *mut core::ffi::c_void, pJobInfo as *const core::ffi::c_void, core::mem::size_of::<JOB_INFO_3>());
        }
        _ => {
            ERR!(b"Level = %d, unsupported!\n\0".as_ptr(), Level);
            SetLastError(ERROR_INVALID_LEVEL);
            return FALSE as BOOL;
        }
    }

    ret = SetJobW(hPrinter, JobId, Level, JobW, Command);

    match Level {
        1 => {
            let info1W: *mut JOB_INFO_1W = JobW as *mut JOB_INFO_1W;
            if !(*info1W).pUserName.is_null() {
                HeapFree(GetProcessHeap(), 0, (*info1W).pUserName as *mut core::ffi::c_void);
            }
            if !(*info1W).pDocument.is_null() {
                HeapFree(GetProcessHeap(), 0, (*info1W).pDocument as *mut core::ffi::c_void);
            }
            if !(*info1W).pDatatype.is_null() {
                HeapFree(GetProcessHeap(), 0, (*info1W).pDatatype as *mut core::ffi::c_void);
            }
            if !(*info1W).pStatus.is_null() {
                HeapFree(GetProcessHeap(), 0, (*info1W).pStatus as *mut core::ffi::c_void);
            }
        }
        2 => {
            let info2W: *mut JOB_INFO_2W = JobW as *mut JOB_INFO_2W;
            if !(*info2W).pUserName.is_null() {
                HeapFree(GetProcessHeap(), 0, (*info2W).pUserName as *mut core::ffi::c_void);
            }
            if !(*info2W).pDocument.is_null() {
                HeapFree(GetProcessHeap(), 0, (*info2W).pDocument as *mut core::ffi::c_void);
            }
            if !(*info2W).pNotifyName.is_null() {
                HeapFree(GetProcessHeap(), 0, (*info2W).pNotifyName as *mut core::ffi::c_void);
            }
            if !(*info2W).pDatatype.is_null() {
                HeapFree(GetProcessHeap(), 0, (*info2W).pDatatype as *mut core::ffi::c_void);
            }
            if !(*info2W).pPrintProcessor.is_null() {
                HeapFree(GetProcessHeap(), 0, (*info2W).pPrintProcessor as *mut core::ffi::c_void);
            }
            if !(*info2W).pParameters.is_null() {
                HeapFree(GetProcessHeap(), 0, (*info2W).pParameters as *mut core::ffi::c_void);
            }
            if !(*info2W).pDevMode.is_null() {
                HeapFree(GetProcessHeap(), 0, (*info2W).pDevMode as *mut core::ffi::c_void);
            }
            if !(*info2W).pStatus.is_null() {
                HeapFree(GetProcessHeap(), 0, (*info2W).pStatus as *mut core::ffi::c_void);
            }
        }
        _ => {}
    }
    HeapFree(GetProcessHeap(), 0, JobW as *mut core::ffi::c_void);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetJobW(hPrinter: HANDLE, JobId: DWORD, Level: DWORD, pJobInfo: PBYTE, Command: DWORD) -> WINBOOL {
    let mut dwErrorCode: DWORD = 0;
    let pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;
    let mut JobContainer: WINSPOOL_JOB_CONTAINER = core::mem::zeroed();

    'Cleanup: {
        if pHandle.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        // pJobContainer->JobInfo is a union of pointers, so we can just set any element to our BYTE pointer.
        JobContainer.Level = Level;
        JobContainer.JobInfo.Level1 = pJobInfo as *mut WINSPOOL_JOB_INFO_1;

        // Do the RPC call (RpcTryExcept block; the exception flag is set by the RPC
        // exception mechanism, which is owned by the integration layer)
        let rpc_faulted: bool = false;
        if !rpc_faulted {
            dwErrorCode = _RpcSetJob((*pHandle).hPrinter, JobId, core::ptr::addr_of_mut!(JobContainer), Command);
        } else {
            // RpcExcept(EXCEPTION_EXECUTE_HANDLER)
            dwErrorCode = RpcExceptionCode!() as DWORD;
        }
        // RpcEndExcept
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}
// enforce_import_abi: moved 1 import decl(s) into the extern block their
// exporting DLL's .spec requires: memcpy->C.
// The CRT is cdecl, Win32 is stdcall, and the C source does not say which is which —
// a wrong guess is a silent i386 stack walk on every call. The .spec is the truth.
