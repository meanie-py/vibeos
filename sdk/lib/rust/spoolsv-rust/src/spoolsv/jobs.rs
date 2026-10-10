use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn _RpcAddJob(hPrinter: WINSPOOL_PRINTER_HANDLE, Level: DWORD, pAddJob: *mut BYTE, cbBuf: DWORD, pcbNeeded: *mut DWORD) -> DWORD {
    let mut dwErrorCode: DWORD;
    let mut pAddJobAligned: PBYTE;
    let mut cbBuf = cbBuf;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS
    {
        return dwErrorCode;
    }

    pAddJobAligned = (AlignRpcPtr((pAddJob).cast(), core::ptr::addr_of_mut!(cbBuf))).cast();

    if AddJobW(hPrinter, Level, pAddJobAligned, cbBuf, pcbNeeded) != 0
    {
        // Replace absolute pointer addresses in the output by relative offsets.
        MarshallDownStructure((pAddJobAligned).cast(), (AddJobInfo1Marshalling.pInfo).as_ptr(), AddJobInfo1Marshalling.cbStructureSize, (TRUE) as BOOL);
    }
    else
    {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    UndoAlignRpcPtr((pAddJob).cast(), (pAddJobAligned).cast(), cbBuf, pcbNeeded);

    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcEnumJobs(hPrinter: WINSPOOL_PRINTER_HANDLE, FirstJob: DWORD, NoJobs: DWORD, Level: DWORD, pJob: *mut BYTE, cbBuf: DWORD, pcbNeeded: *mut DWORD, pcReturned: *mut DWORD) -> DWORD {
    let mut dwErrorCode: DWORD;
    let mut pJobAligned: PBYTE;
    let mut cbBuf = cbBuf;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    pJobAligned = (AlignRpcPtr((pJob).cast(), core::ptr::addr_of_mut!(cbBuf))).cast();

    if EnumJobsW(hPrinter, FirstJob, NoJobs, Level, pJobAligned, cbBuf, pcbNeeded, pcReturned) != 0 {
        // Replace absolute pointer addresses in the output by relative offsets for JOB_INFO_1W and JOB_INFO_2W.
        if Level <= 2 {
            MarshallDownStructuresArray((pJobAligned).cast(), *pcReturned, (*pJobInfoMarshalling[Level as usize]).pInfo.as_ptr(), (*pJobInfoMarshalling[Level as usize]).cbStructureSize, (TRUE) as BOOL);
        }
    } else {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    UndoAlignRpcPtr((pJob).cast(), (pJobAligned).cast(), cbBuf, pcbNeeded);

    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcGetJob(hPrinter: WINSPOOL_PRINTER_HANDLE, JobId: DWORD, Level: DWORD, pJob: *mut BYTE, mut cbBuf: DWORD, pcbNeeded: *mut DWORD) -> DWORD {
    let mut dwErrorCode: DWORD;
    let mut pJobAligned: PBYTE;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        //
        return dwErrorCode;
    }

    pJobAligned = (AlignRpcPtr((pJob).cast(), core::ptr::addr_of_mut!(cbBuf))).cast();

    if GetJobW(hPrinter, JobId, Level, pJobAligned, cbBuf, pcbNeeded) != 0 {
        // Replace absolute pointer addresses in the output by relative offsets.
        MarshallDownStructure((pJobAligned).cast(), (*pJobInfoMarshalling[Level as usize]).pInfo.as_ptr(), (*pJobInfoMarshalling[Level as usize]).cbStructureSize, (TRUE) as BOOL);
    } else {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    UndoAlignRpcPtr((pJob).cast(), (pJobAligned).cast(), cbBuf, pcbNeeded);

    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcScheduleJob(hPrinter: WINSPOOL_PRINTER_HANDLE, JobId: DWORD) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    if ScheduleJob(hPrinter, JobId) == 0 {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcSetJob(hPrinter: WINSPOOL_PRINTER_HANDLE, JobId: DWORD, pJobContainer: *mut WINSPOOL_JOB_CONTAINER, Command: DWORD) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS
    {
        return dwErrorCode;
    }

    // pJobContainer->JobInfo is a union of pointers, so we can just convert any element to our BYTE pointer.
    if SetJobW(hPrinter, JobId, (*pJobContainer).Level, (*pJobContainer).JobInfo.Level1 as PBYTE, Command) == 0
    {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
