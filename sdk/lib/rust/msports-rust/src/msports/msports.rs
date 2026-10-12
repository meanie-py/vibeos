use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstDll: HINSTANCE, dwReason: DWORD, reserved: LPVOID) -> BOOL {
    match dwReason {
        DLL_PROCESS_ATTACH => {
            TRACE!("DLL_PROCESS_ATTACH\n");
            hInstance = hinstDll;
            DisableThreadLibraryCalls(hinstDll);
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}
