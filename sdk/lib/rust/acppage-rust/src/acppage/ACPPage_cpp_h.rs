use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut g_hModule: HMODULE = core::ptr::null_mut();
pub static mut g_ModuleRefCnt: LONG = 0;

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    if g_ModuleRefCnt != 0 {
        return S_FALSE;
    }
    return (*core::ptr::addr_of_mut!(gModule)).DllCanUnloadNow();
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(rclsid: REFCLSID, riid: REFIID, ppv: *mut LPVOID) -> HRESULT {
    return (*core::ptr::addr_of_mut!(gModule)).DllGetClassObject(rclsid, riid, ppv);
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    let mut hr: HRESULT;

    hr = (*core::ptr::addr_of_mut!(gModule)).DllRegisterServer(FALSE as BOOL);
    if (hr < 0) {
        return hr;
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    let mut hr: HRESULT;

    hr = (*core::ptr::addr_of_mut!(gModule)).DllUnregisterServer(FALSE as BOOL);
    if (hr < 0) {
        return hr;
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn GetExeFromLnk(pszLnk: PCWSTR, pszExe: PWSTR, cchSize: usize) -> BOOL {
    let mut init: CCoInit = CCoInit::new();
    if FAILED_UNEXPECTEDLY!(init.hr) {
        return (FALSE) as BOOL;
    }

    let mut spShellLink: CComPtr<IShellLinkW> = CComPtr::new();
    if FAILED_UNEXPECTEDLY!(CoCreateInstance(
        &CLSID_ShellLink,
        core::ptr::null_mut(),
        CLSCTX_INPROC_SERVER,
        core::ptr::addr_of!(IID_IShellLinkW),
        spShellLink.GetAddressOf() as *mut LPVOID,
    )) {
        return (FALSE) as BOOL;
    }

    let mut spPersistFile: CComPtr<IPersistFile> = CComPtr::new();
    if FAILED_UNEXPECTEDLY!((*spShellLink.Get()).QueryInterface(
        core::ptr::addr_of!(IID_IPersistFile),
        spPersistFile.GetAddressOf() as *mut LPVOID,
    )) {
        return (FALSE) as BOOL;
    }

    if FAILED_UNEXPECTEDLY!((*spPersistFile.Get()).Load(pszLnk, STGM_READ))
        || FAILED_UNEXPECTEDLY!((*spShellLink.Get()).Resolve(
            core::ptr::null_mut(),
            SLR_NO_UI | SLR_NOUPDATE | SLR_NOSEARCH,
        ))
    {
        return (FALSE) as BOOL;
    }

    (!FAILED_UNEXPECTEDLY!((*spShellLink.Get()).GetPath(
        pszExe,
        cchSize as i32,
        core::ptr::null_mut(),
        SLGP_RAWPATH,
    ))) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hInstance: HINSTANCE, dwReason: DWORD, lpReserved: LPVOID) -> BOOL {
    match dwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hInstance);
            g_hModule = hInstance;
            (*core::ptr::addr_of_mut!(gModule)).Init(ObjectMap.as_ptr(), hInstance, core::ptr::null::<c_void>());
        }
        DLL_PROCESS_DETACH => {
            (*core::ptr::addr_of_mut!(gModule)).Term();
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}
