use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub static mut g_hInstance: HINSTANCE = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstDLL: HINSTANCE, fdwReason: DWORD, fImpLoad: LPVOID) -> BOOL {
    if fdwReason == DLL_PROCESS_ATTACH {
        g_hInstance = hinstDLL;
        DisableThreadLibraryCalls(g_hInstance);

        g_Module.Init(ObjectMap.as_ptr(), g_hInstance, core::ptr::null::<core::ffi::c_void>());
    } else if fdwReason == DLL_PROCESS_DETACH {
        g_hInstance = core::ptr::null_mut();
        g_Module.Term();
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    return g_Module.DllRegisterServer((FALSE as i32));
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    return (*core::ptr::addr_of_mut!(g_Module)).DllUnregisterServer((FALSE as i32));
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(rclsid: *const c_void, riid: *const c_void, ppv: *mut LPVOID) -> HRESULT {
    return (*core::ptr::addr_of_mut!(g_Module)).DllGetClassObject(rclsid, riid, ppv);
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    g_Module.DllCanUnloadNow()
}
