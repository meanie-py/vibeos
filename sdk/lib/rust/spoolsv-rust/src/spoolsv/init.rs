use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn _RpcSpoolerInit() -> DWORD {
    let mut dwErrorCode: DWORD;
    dwErrorCode = RpcImpersonateClient(core::ptr::null_mut()) as DWORD;
    if dwErrorCode != ERROR_SUCCESS {
        return dwErrorCode;
    }

    if SpoolerInit() == 0 {
        dwErrorCode = GetLastError() as DWORD;
    }

    RpcRevertToSelf();
    return dwErrorCode;
}
