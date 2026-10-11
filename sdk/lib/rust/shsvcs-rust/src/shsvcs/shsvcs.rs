use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn DllInstall(bInstall: BOOL, cmdline: LPCWSTR) -> HRESULT {
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: REFCLSID,
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    // TRACE("%s %s %p\n", debugstr_guid(rclsid), debugstr_guid(riid), ppv);
    CLASS_E_CLASSNOTAVAILABLE
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    fdwReason: DWORD,
    lpvReserved: LPVOID,
) -> BOOL {
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinstDLL as HMODULE);
        }
        DLL_PROCESS_DETACH => {}
        _ => {}
    }

    return (TRUE) as BOOL;
}
