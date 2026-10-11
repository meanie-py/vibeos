// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_356589F8D0E8: [u16; 34] = [
    0x0025u16, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0052, 0x006F, 0x006F,
    0x0074, 0x0025, 0x005C, 0x0073, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0033,
    0x0032, 0x005C, 0x0063, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x002E,
    0x0065, 0x0078, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_4ACC97D39AA8: [u16; 14] = [
    0x0075u16, 0x0073, 0x0065, 0x0072, 0x0070, 0x0061, 0x0073, 0x0073, 0x0077, 0x006F,
    0x0072, 0x0064, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations in this preamble. Its only content is the conditional `UsersRunDll` function definition (`#if DLL_EXPORT_VERSION < _WIN32_WINNT_VISTA`), which is handled by the per-function pass.

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(rclsid: *const CLSID, riid: *const IID, ppv: *mut LPVOID) -> HRESULT {
    *ppv = core::ptr::null_mut();
    return E_NOINTERFACE;
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
pub unsafe extern "system" fn DllMain(hinstDLL: HINSTANCE, dwReason: DWORD, lpvReserved: LPVOID) -> BOOL {
    let mut iccx: INITCOMMONCONTROLSEX = tagINITCOMMONCONTROLSEX {
        dwSize: 0,
        dwICC: 0,
    };
    hInstance = hinstDLL;
    match dwReason {
        DLL_PROCESS_ATTACH => {
            iccx.dwSize = ::core::mem::size_of::<INITCOMMONCONTROLSEX>() as DWORD;
            iccx.dwICC = (ICC_STANDARD_CLASSES | ICC_LISTVIEW_CLASSES) as DWORD;
            InitCommonControlsEx(core::ptr::addr_of_mut!(iccx));
            DisableThreadLibraryCalls(hInstance as HMODULE);
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ClearAutoLogon() -> i32 {
    0
}

#[no_mangle]
pub unsafe extern "system" fn UsersRunDllW(hWnd: HWND, hInst: HINSTANCE, pszCmdLine: LPWSTR, nCmdShow: i32) {
    let mut sei: SHELLEXECUTEINFOW = core::mem::zeroed();
    sei.cbSize = core::mem::size_of::<SHELLEXECUTEINFOW>() as DWORD;
    sei.fMask = (SEE_MASK_FLAG_DDEWAIT | SEE_MASK_DOENVSUBST) as ULONG;
    sei.lpFile = RUSTCONV_WSTR_356589F8D0E8.as_ptr();
    sei.lpParameters = RUSTCONV_WSTR_4ACC97D39AA8.as_ptr();
    sei.nShow = (SW_SHOWDEFAULT as i32);
    ShellExecuteExW(&raw mut sei);
}

#[no_mangle]
pub unsafe extern "system" fn UsersRunDll(hWnd: HWND, hInst: HINSTANCE, pszCmdLine: LPSTR, nCmdShow: ::core::ffi::c_int) {
    UsersRunDllW(hWnd, hInst, RUSTCONV_WSTR_96A296D224F2.as_ptr() as LPWSTR, nCmdShow);
}
