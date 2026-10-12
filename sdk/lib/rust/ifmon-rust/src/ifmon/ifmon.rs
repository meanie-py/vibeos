use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hDllInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub unsafe extern "system" fn InitHelperDll(dwNetshVersion: DWORD, pReserved: PVOID) -> DWORD {
    let mut dwError: DWORD;

    dwError = RegisterInterfaceHelper();
    if dwError == ERROR_SUCCESS {
        dwError = RegisterIpHelper();
    }
    if dwError == ERROR_SUCCESS {
        dwError = RegisterWinsockHelper();
    }

    return dwError;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    dwReason: DWORD,
    lpvReserved: LPVOID,
) -> BOOL {
    match dwReason {
        DLL_PROCESS_ATTACH => {
            hDllInstance = hinstDLL;
            DisableThreadLibraryCalls(hinstDLL as HMODULE);
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}
