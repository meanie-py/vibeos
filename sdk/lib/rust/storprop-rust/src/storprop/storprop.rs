use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub unsafe extern "system" fn VolumePropPageProvider(
    lpPropSheetPageRequest: PSP_PROPSHEETPAGE_REQUEST,
    lpfnAddPropSheetPageProc: LPFNADDPROPSHEETPAGE,
    lParam: LPARAM,
) -> BOOL {
    // DPRINT("VolumePropPageProvider(%p %p %lx)\n", lpPropSheetPageRequest, lpfnAddPropSheetPageProc, lParam);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDll: HINSTANCE,
    dwReason: DWORD,
    reserved: LPVOID,
) -> BOOL {
    match dwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinstDll);
            hInstance = hinstDll;
        }
        DLL_PROCESS_DETACH => {
            hInstance = core::ptr::null_mut();
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}
