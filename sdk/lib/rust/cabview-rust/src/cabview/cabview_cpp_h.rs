use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations in this preamble: it contains only four function definitions (DllCanUnloadNow, DllGetClassObject, DllRegisterServer, DllUnregisterServer), which are translated separately. No typedefs, #defines, enums, or static tables.

#[no_mangle]
pub unsafe extern "system" fn DllMain(hInstance: HINSTANCE, dwReason: DWORD, lpReserved: LPVOID) -> BOOL {
    match dwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hInstance);
            g_Module.Init(ObjectMap.as_ptr(), hInstance, core::ptr::null::<core::ffi::c_void>());
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    return (*core::ptr::addr_of!(g_Module)).DllCanUnloadNow();
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const core::ffi::c_void,
    riid: *const core::ffi::c_void,
    ppv: *mut LPVOID,
) -> HRESULT {
    return (*core::ptr::addr_of_mut!(g_Module)).DllGetClassObject(rclsid, riid, ppv);
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    let mut hr: HRESULT;

    hr = (*core::ptr::addr_of_mut!(g_Module)).DllRegisterServer((FALSE as i32));
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    hr = (*core::ptr::addr_of_mut!(g_Module)).UpdateRegistryFromResource(
        IDR_FOLDER,
        (TRUE as i32),
        core::ptr::null_mut::<core::ffi::c_void>(),
    );
    if ((hr) < 0) {
        return hr;
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    let hr1: HRESULT = (*core::ptr::addr_of_mut!(g_Module)).DllUnregisterServer((FALSE as i32));
    let hr2: HRESULT = (*core::ptr::addr_of_mut!(g_Module)).UpdateRegistryFromResource(IDR_FOLDER, (FALSE as i32), core::ptr::null::<c_void>());
    if FAILED_UNEXPECTEDLY!(hr1)
    {
        return hr1;
    }
    if FAILED_UNEXPECTEDLY!(hr2)
    {
        return hr2;
    }
    return S_OK;
}
