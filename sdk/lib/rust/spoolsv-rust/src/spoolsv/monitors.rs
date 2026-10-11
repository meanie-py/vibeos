use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn _RpcAddMonitor(pName: WINSPOOL_HANDLE, pMonitorContainer: *mut WINSPOOL_MONITOR_CONTAINER) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS
    {
        return dwErrorCode;
    }

    if ((AddMonitorW(pName, (*pMonitorContainer).Level, (*pMonitorContainer).MonitorInfo.pMonitorInfo2 as PBYTE)) == 0)
    {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcDeleteMonitor(pName: WINSPOOL_HANDLE, pEnvironment: *mut WCHAR, pMonitorName: *mut WCHAR) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    if DeleteMonitorW(pName, pEnvironment, pMonitorName) == 0 {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcEnumMonitors(pName: WINSPOOL_HANDLE, Level: DWORD, pMonitor: *mut BYTE, mut cbBuf: DWORD, pcbNeeded: *mut DWORD, pcReturned: *mut DWORD) -> DWORD {
    let mut dwErrorCode: DWORD;
    let mut pMonitorAligned: PBYTE;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    pMonitorAligned = (AlignRpcPtr((pMonitor).cast(), core::ptr::addr_of_mut!(cbBuf))).cast();

    if EnumMonitorsW(pName, Level, pMonitorAligned, cbBuf, pcbNeeded, pcReturned) != 0 {
        // Replace absolute pointer addresses in the output by relative offsets.

        MarshallDownStructuresArray((pMonitorAligned).cast(), *pcReturned, (*pMonitorInfoMarshalling[Level as usize]).pInfo.as_ptr(), (*pMonitorInfoMarshalling[Level as usize]).cbStructureSize, (TRUE) as BOOL);
    } else {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    UndoAlignRpcPtr((pMonitor).cast(), (pMonitorAligned).cast(), cbBuf, pcbNeeded);

    return dwErrorCode;
}
