use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut hInst: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut dll_ref: LONG = 0 as LONG;

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstDLL: HINSTANCE, fdwReason: DWORD, lpvReserved: LPVOID) -> BOOL {
    TRACE!("(%p, %d, %p)\n", hinstDLL, fdwReason, lpvReserved);
    match fdwReason {
        #[cfg(any())]
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinstDLL as HMODULE);
            hInst = hinstDLL;
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(rclsid: REFCLSID, iid: REFIID, ppv: *mut LPVOID) -> HRESULT {
    if IsEqualGUID!(rclsid, &raw const CLSID_CTaskScheduler) {
        return IClassFactory_QueryInterface(&raw mut MSTASK_ClassFactory as LPCLASSFACTORY, iid, ppv);
    }

    return CLASS_E_CLASSNOTAVAILABLE;
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    return if dll_ref != 0 { S_FALSE } else { S_OK };
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    return __wine_register_resources(hInst as HMODULE);
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    return __wine_unregister_resources(hInst);
}
