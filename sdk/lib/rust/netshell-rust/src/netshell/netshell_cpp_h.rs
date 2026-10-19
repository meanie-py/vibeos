use crate::prelude::*;
use core::ffi::c_void;


// ---- file-scope preamble (translated) ----
pub static mut g_hModule: HMODULE = core::ptr::null_mut();
#[no_mangle]
pub static mut netshell_hInstance: HINSTANCE = core::ptr::null_mut();
#[no_mangle]
pub unsafe extern "C" fn InitializePropertySheetPage(resname: LPWSTR, dlgproc: DLGPROC, lParam: LPARAM, szTitle: LPWSTR) -> HPROPSHEETPAGE {
    let mut ppage: PROPSHEETPAGEW = core::mem::zeroed();

    memset(core::ptr::addr_of_mut!(ppage) as *mut core::ffi::c_void, 0x0 as i32, core::mem::size_of::<PROPSHEETPAGEW>());
    ppage.dwSize = core::mem::size_of::<PROPSHEETPAGEW>() as u32;
    ppage.dwFlags = PSP_DEFAULT as u32;
    ppage.__bindgen_anon_1.pszTemplate = resname;
    ppage.pfnDlgProc = dlgproc;
    ppage.lParam = lParam;
    ppage.hInstance = netshell_hInstance;
    if !szTitle.is_null()
    {
        ppage.dwFlags |= PSP_USETITLE as u32;
        ppage.pszTitle = szTitle;
    }
    return CreatePropertySheetPageW(core::ptr::addr_of!(ppage));
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstDLL: HINSTANCE, fdwReason: DWORD, fImpLoad: LPVOID) -> BOOL {
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            netshell_hInstance = hinstDLL;
            DisableThreadLibraryCalls(netshell_hInstance);
            gModule.Init((ObjectMap).as_ptr(), netshell_hInstance, core::ptr::null::<GUID>());
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    S_FALSE
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    let mut hr: HRESULT;

    hr = gModule.DllRegisterServer((FALSE as i32));
    if FAILED_UNEXPECTEDLY!(hr) != false {
        return hr;
    }

    hr = gModule.UpdateRegistryFromResource(IDR_NETSHELL, (TRUE as i32), core::ptr::null::<c_void>());
    if FAILED_UNEXPECTEDLY!(hr) != false {
        return hr;
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    let mut hr: HRESULT;

    hr = gModule.DllUnregisterServer((FALSE as i32));
    if FAILED_UNEXPECTEDLY!(hr) != false {
        return hr;
    }

    hr = gModule.UpdateRegistryFromResource(IDR_NETSHELL, FALSE as i32, core::ptr::null::<c_void>());
    if FAILED_UNEXPECTEDLY!(hr) != false {
        return hr;
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(rclsid: *const GUID, riid: *const GUID, ppv: *mut LPVOID) -> HRESULT {
    return (*core::ptr::addr_of_mut!(gModule)).DllGetClassObject(rclsid, riid, ppv);
}

#[no_mangle]
pub unsafe extern "system" fn NcFreeNetconProperties(pProps: *mut NETCON_PROPERTIES) {
    CoTaskMemFree((*pProps).pszwName as LPVOID);
    CoTaskMemFree((*pProps).pszwDeviceName as LPVOID);
    CoTaskMemFree(pProps as LPVOID);
}

#[no_mangle]
pub unsafe extern "system" fn NcIsValidConnectionName(mut pszwName: PCWSTR) -> BOOL {
    if pszwName.is_null() {
        return (FALSE) as BOOL;
    }

    let mut nonSpace: BOOL = (FALSE as i32);
    while *pszwName != 0 {
        let c: u16 = *pszwName;
        pszwName = pszwName.add(1);
        match c {
            0x005Cu16 | 0x002Fu16 | 0x003Au16 | 0x002Au16 | 0x0009u16 | 0x003Fu16
            | 0x003Cu16 | 0x003Eu16 | 0x007Cu16 | 0x0022u16 => {
                return (FALSE) as BOOL;
            }
            0x0020u16 => {
            }
            _ => {
                nonSpace = (TRUE as i32);
            }
        }
    }
    nonSpace
}
