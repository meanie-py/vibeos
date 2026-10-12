use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn DllMain(hInstance: HINSTANCE, dwReason: DWORD, lpReserved: LPVOID) -> BOOL {
    match dwReason {
        v if v == DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hInstance);
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}
