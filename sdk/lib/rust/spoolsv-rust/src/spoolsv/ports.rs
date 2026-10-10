use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn _RpcAddPort(pName: WINSPOOL_HANDLE, hWnd: ULONG_PTR, pMonitorName: *mut WCHAR) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    if AddPortW(pName, hWnd as HWND, pMonitorName) == 0 {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcAddPortEx(pName: WINSPOOL_HANDLE, pPortContainer: *mut WINSPOOL_PORT_CONTAINER, pPortVarContainer: *mut WINSPOOL_PORT_VAR_CONTAINER, pMonitorName: *mut WCHAR) -> DWORD {
    let mut dwErrorCode: DWORD;
    let Level: DWORD = (*pPortContainer).Level;
    let mut PortInfoFF: WINSPOOL_PORT_INFO_FF = core::mem::zeroed();
    let mut lpBuffer: PBYTE;

    match Level {
        1 => {
            lpBuffer = (*pPortContainer).PortInfo.pPortInfo1 as PBYTE;
        }

        0xFFFFFFFF => {
            PortInfoFF.pPortName = (*(*pPortContainer).PortInfo.pPortInfoFF).pPortName;
            PortInfoFF.cbMonitorData = (*pPortVarContainer).cbMonitorData;
            PortInfoFF.pMonitorData = (*pPortVarContainer).pMonitorData;
            lpBuffer = core::ptr::addr_of_mut!(PortInfoFF) as PBYTE;
        }

        _ => {
            // ERR("Level = %d, unsupported!\n", Level);
            return ERROR_INVALID_LEVEL;
        }
    }

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    if AddPortExW(pName, Level, lpBuffer, pMonitorName) == 0 {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcConfigurePort(pName: WINSPOOL_HANDLE, hWnd: ULONG_PTR, pPortName: *mut WCHAR) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    if ConfigurePortW(pName, hWnd as HWND, pPortName) == 0 {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcDeletePort(pName: WINSPOOL_HANDLE, hWnd: ULONG_PTR, pPortName: *mut WCHAR) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS
    {
        return dwErrorCode;
    }

    if DeletePortW(pName, hWnd as HWND, pPortName) == 0
    {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcEnumPorts(pName: WINSPOOL_HANDLE, Level: DWORD, pPort: *mut BYTE, mut cbBuf: DWORD, pcbNeeded: *mut DWORD, pcReturned: *mut DWORD) -> DWORD {
    let mut dwErrorCode: DWORD;
    let mut pPortAligned: PBYTE;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    pPortAligned = (AlignRpcPtr((pPort).cast(), core::ptr::addr_of_mut!(cbBuf))).cast();

    if EnumPortsW(pName, Level, pPortAligned, cbBuf, pcbNeeded, pcReturned) != 0 {
        // Replace absolute pointer addresses in the output by relative offsets.
        MarshallDownStructuresArray((pPortAligned).cast(), *pcReturned, (*pPortInfoMarshalling[Level as usize]).pInfo.as_ptr(), (*pPortInfoMarshalling[Level as usize]).cbStructureSize, (TRUE) as BOOL);
    } else {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    UndoAlignRpcPtr((pPort).cast(), (pPortAligned).cast(), cbBuf, pcbNeeded);

    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcSetPort(pName: WINSPOOL_HANDLE, pPortName: *mut WCHAR, pPortContainer: *mut WINSPOOL_PORT_CONTAINER) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = (RpcImpersonateClient(core::ptr::null_mut())) as u32;
    if dwErrorCode != ERROR_SUCCESS
    {

        return dwErrorCode;
    }

    if SetPortW(pName, pPortName, (*pPortContainer).Level, (*pPortContainer).PortInfo.pPortInfo3 as PBYTE) == 0
        {
            dwErrorCode = GetLastError();
        }

    RpcRevertToSelf();
    return dwErrorCode;
}
