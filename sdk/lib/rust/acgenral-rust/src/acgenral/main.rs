use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn GetHookAPIs(
    szCommandLine: LPCSTR,
    wszShimName: LPCWSTR,
    pdwHookCount: PDWORD,
) -> PHOOKAPI {
    ShimLib_GetHookAPIs(szCommandLine, wszShimName, pdwHookCount)
}

#[no_mangle]
pub unsafe extern "system" fn NotifyShims(fdwReason: DWORD, ptr: PVOID) -> BOOL {
    ShimLib_NotifyShims(fdwReason, ptr)
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hInstance: HINSTANCE, dwReason: DWORD, lpReserved: LPVOID) -> BOOL {
    match dwReason {
        DLL_PROCESS_ATTACH => {
            ShimLib_Init(hInstance);
        }
        DLL_PROCESS_DETACH => {
            ShimLib_Deinit();
        }
        _ => {}
    }
    return TRUE as BOOL;
}
