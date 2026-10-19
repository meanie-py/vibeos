use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut g_hInstance: HINSTANCE = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstDLL: HINSTANCE, fdwReason: u32, fImpLoad: LPVOID) -> i32 {
    if fdwReason == DLL_PROCESS_ATTACH {
        g_hInstance = hinstDLL;
        DisableThreadLibraryCalls(g_hInstance);

        g_Module.Init(ObjectMap.as_ptr(), g_hInstance, core::ptr::null_mut::<core::ffi::c_void>());
    } else if fdwReason == DLL_PROCESS_DETACH {
        g_hInstance = core::ptr::null_mut();
        g_Module.Term();
    }
    return (TRUE as i32);
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    return (*core::ptr::addr_of_mut!(g_Module)).DllRegisterServer((FALSE as i32));
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    return (*core::ptr::addr_of_mut!(g_Module)).DllUnregisterServer((FALSE as i32));
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(rclsid: *const core::ffi::c_void, riid: *const core::ffi::c_void, ppv: *mut LPVOID) -> HRESULT {
    return g_Module.DllGetClassObject(rclsid, riid, ppv);
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    return g_Module.DllCanUnloadNow();
}
