use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn _RpcXcvData(hXcv: WINSPOOL_PRINTER_HANDLE, pszDataName: *const WCHAR, pInputData: *mut BYTE, cbInputData: DWORD, pOutputData: *mut BYTE, cbOutputData: DWORD, pcbOutputNeeded: *mut DWORD, pdwStatus: *mut DWORD) -> DWORD {
    let mut dwErrorCode: DWORD;

    dwErrorCode = RpcImpersonateClient(core::ptr::null_mut()) as DWORD;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    if XcvDataW(hXcv, pszDataName, pInputData, cbInputData, pOutputData, cbOutputData, pcbOutputNeeded, pdwStatus) == 0 {
        dwErrorCode = GetLastError();
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
