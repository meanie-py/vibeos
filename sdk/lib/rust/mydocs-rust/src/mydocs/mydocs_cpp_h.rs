use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_5436A89AD1F3: [WCHAR; 8] = [0x002E, 0x006D, 0x0079, 0x0064, 0x006F, 0x0063, 0x0073, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut g_ModuleRefCnt: LONG = 0;

unsafe extern "C" fn CreateEmptyFile(pszFile: LPCWSTR) -> BOOL {
    let mut hFile: HANDLE;
    hFile = CreateFileW(pszFile, GENERIC_WRITE, FILE_SHARE_READ, core::ptr::null_mut(),
                        CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, core::ptr::null_mut());
    if hFile != INVALID_HANDLE_VALUE {
        CloseHandle(hFile);
        return (TRUE) as BOOL;
    }
    return (FALSE) as BOOL;
}

unsafe extern "C" fn CreateSendToMyDocuments(pszSendTo: LPCWSTR) -> HRESULT {
    let mut szTarget = [0u16; (MAX_PATH) as _];
    let mut szSendToFile = [0u16; (MAX_PATH) as _];

    SHGetSpecialFolderPathW(core::ptr::null_mut(), szTarget.as_mut_ptr(), (CSIDL_MYDOCUMENTS as i32), (FALSE) as BOOL);

    StringCbCopyW(szSendToFile.as_mut_ptr(), core::mem::size_of::<[u16; (MAX_PATH) as _]>(), pszSendTo);
    PathAppendW(szSendToFile.as_mut_ptr(), PathFindFileNameW(szTarget.as_ptr()));
    StringCbCatW(szSendToFile.as_mut_ptr(), core::mem::size_of::<[u16; (MAX_PATH) as _]>(), CPP_COM_WSTR_5436A89AD1F3.as_ptr());

    if CreateEmptyFile(szSendToFile.as_ptr()) == 0
    {
        return E_FAIL;
    }

    return S_OK;
}

unsafe extern "C" fn GetDefaultUserSendTo(pszPath: LPWSTR) -> HRESULT {
    return SHGetFolderPathW(core::ptr::null_mut(), (CSIDL_SENDTO as i32), INVALID_HANDLE_VALUE,
                            SHGFP_TYPE_DEFAULT, pszPath);
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    if g_ModuleRefCnt != 0 {
        return S_FALSE;
    }
    return (*core::ptr::addr_of_mut!(gModule)).DllCanUnloadNow();
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(rclsid: *const GUID, riid: *const GUID, ppv: *mut LPVOID) -> HRESULT {
    let mut hr: HRESULT;

    hr = (*core::ptr::addr_of_mut!(gModule)).DllGetClassObject(rclsid, riid, ppv);

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    let mut hr: HRESULT = gModule.DllRegisterServer((FALSE as i32));
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    let mut szSendTo: [u16; (MAX_PATH) as _] = [0u16; (MAX_PATH) as _];
    hr = GetDefaultUserSendTo(szSendTo.as_mut_ptr());
    if hr >= 0 {
        CreateSendToMyDocuments(szSendTo.as_ptr());
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    let mut hr: HRESULT = gModule.DllUnregisterServer((FALSE as i32));
    if FAILED_UNEXPECTEDLY!(hr) != false {
        return hr;
    }

    return S_OK;
}
#[no_mangle]
pub unsafe extern "system" fn DllMain(hInstance: HINSTANCE, dwReason: DWORD, fImpLoad: LPVOID) -> BOOL {
    if dwReason == DLL_PROCESS_ATTACH {
        gModule.Init(ObjectMap.as_ptr(), hInstance, core::ptr::null::<core::ffi::c_void>());
        DisableThreadLibraryCalls(hInstance);
    } else if dwReason == DLL_PROCESS_DETACH {
        gModule.Term();
    }
    return (TRUE) as BOOL;
}
