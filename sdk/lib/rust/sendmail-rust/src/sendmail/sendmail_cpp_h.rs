use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_DC3AE41EAAA2: [WCHAR; 10] = [0x002E, 0x0044, 0x0065, 0x0073, 0x006B, 0x004C, 0x0069, 0x006E, 0x006B, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut g_ModuleRefCnt: LONG = 0;
pub static mut g_hModule: HINSTANCE = core::ptr::null_mut();

unsafe extern "C" fn CreateEmptyFile(pszFile: LPCWSTR) -> i32 {
    let mut hFile: HANDLE;
    hFile = CreateFileW(pszFile, GENERIC_WRITE, FILE_SHARE_READ, core::ptr::null_mut(),
                        CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, core::ptr::null_mut());
    if hFile != INVALID_HANDLE_VALUE {
        CloseHandle(hFile);
        return (TRUE as i32);
    }
    return (FALSE as i32);
}

unsafe extern "C" fn CreateSendToDeskLink(pszSendTo: LPCWSTR) -> HRESULT {
    let mut szTarget: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szSendToFile: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    LoadStringW(g_hModule, IDS_DESKLINK, szTarget.as_mut_ptr(), (_countof!(szTarget)) as i32);
    StringCbCatW(szTarget.as_mut_ptr(), (core::mem::size_of::<[u16; (MAX_PATH) as _]>() as i32) as usize, CPP_COM_WSTR_DC3AE41EAAA2.as_ptr());

    StringCbCopyW(szSendToFile.as_mut_ptr(), (core::mem::size_of::<[u16; (MAX_PATH) as _]>() as i32) as usize, pszSendTo);
    PathAppendW(szSendToFile.as_mut_ptr(), szTarget.as_mut_ptr());

    if CreateEmptyFile(szSendToFile.as_mut_ptr()) == 0 {
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
    return gModule.DllCanUnloadNow();
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const core::ffi::c_void,
    riid: *const core::ffi::c_void,
    ppv: *mut LPVOID,
) -> HRESULT {
    let mut hr: HRESULT;

    hr = (*core::ptr::addr_of_mut!(gModule)).DllGetClassObject(rclsid, riid, ppv);

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    let mut hr: HRESULT = (*core::ptr::addr_of_mut!(gModule)).DllRegisterServer((FALSE as i32));
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    let mut szSendTo: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    hr = GetDefaultUserSendTo(szSendTo.as_mut_ptr());
    if (hr >= 0) {
        CreateSendToDeskLink(szSendTo.as_ptr());
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    let hr: HRESULT = gModule.DllUnregisterServer((FALSE as i32));
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn CreateShellLink(pszLinkPath: LPCWSTR, pszTargetPath: LPCWSTR, pidlTarget: LPCITEMIDLIST, pszArg: LPCWSTR, pszDir: LPCWSTR, pszIconPath: LPCWSTR, iIconNr: INT, pszComment: LPCWSTR) -> HRESULT {
    let mut psl: CComPtr<IShellLinkW> = CComPtr::new();
    let mut hr: HRESULT = CoCreateInstance(&CLSID_ShellLink, core::ptr::null_mut(),
                                  CLSCTX_INPROC_SERVER,
                                  core::ptr::addr_of!(IID_IShellLinkW), psl.GetAddressOf() as *mut *mut core::ffi::c_void);
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    if !pszTargetPath.is_null() {
        hr = (*psl.Get()).SetPath(pszTargetPath);
        if FAILED_UNEXPECTEDLY!(hr) {
            return hr;
        }
    }
    else if !pidlTarget.is_null() {
        hr = (*psl.Get()).SetIDList(pidlTarget);
        if FAILED_UNEXPECTEDLY!(hr) {
            return hr;
        }
    }
    else {
        return E_INVALIDARG;
    }

    if !pszArg.is_null() {
        hr = (*psl.Get()).SetArguments(pszArg);
    }

    if !pszDir.is_null() {
        hr = (*psl.Get()).SetWorkingDirectory(pszDir);
    }

    if !pszIconPath.is_null() {
        hr = (*psl.Get()).SetIconLocation(pszIconPath, iIconNr);
    }

    if !pszComment.is_null() {
        hr = (*psl.Get()).SetDescription(pszComment);
    }

    let mut ppf: CComPtr<IPersistFile> = CComPtr::new();
    hr = (*psl.Get()).QueryInterface(core::ptr::addr_of!(IID_IPersistFile), ppf.GetAddressOf() as *mut *mut core::ffi::c_void);
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    hr = (*ppf.Get()).Save(pszLinkPath, (TRUE as i32));
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    return hr;
}
#[no_mangle]
pub unsafe extern "system" fn DllMain(hInstance: HINSTANCE, dwReason: DWORD, fImpLoad: LPVOID) -> BOOL {
    if dwReason == DLL_PROCESS_ATTACH {
        g_hModule = hInstance;
        (*core::ptr::addr_of_mut!(gModule)).Init(ObjectMap.as_ptr(), hInstance, core::ptr::null::<core::ffi::c_void>());
        DisableThreadLibraryCalls(hInstance);
    } else if dwReason == DLL_PROCESS_DETACH {
        (*core::ptr::addr_of_mut!(gModule)).Term();
    }
    return (TRUE) as BOOL;
}
