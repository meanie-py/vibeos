use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_1B9334FEECE6: [WCHAR; 2] = [0x0022, 0x0000];
pub static CPP_COM_WSTR_2AA7531E01FA: [WCHAR; 12] = [0x005A, 0x0069, 0x0070, 0x0043, 0x006F, 0x0064, 0x0065, 0x0050, 0x0061, 0x0067, 0x0065, 0x0000];
pub static CPP_COM_WSTR_82DB442E7A60: [WCHAR; 5] = [0x006F, 0x0070, 0x0065, 0x006E, 0x0000];
pub static CPP_COM_WSTR_84DE1F778E09: [WCHAR; 16] = [0x002E, 0x005A, 0x0046, 0x0053, 0x0065, 0x006E, 0x0064, 0x0054, 0x006F, 0x0054, 0x0061, 0x0072, 0x0067, 0x0065, 0x0074, 0x0000];
pub static CPP_COM_WSTR_AB68B18B313D: [WCHAR; 13] = [0x0065, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x002E, 0x0065, 0x0078, 0x0065, 0x0000];
pub static CPP_COM_WSTR_D8643F50C021: [WCHAR; 14] = [0x0055, 0x006E, 0x005A, 0x0069, 0x0070, 0x0043, 0x006F, 0x0064, 0x0065, 0x0050, 0x0061, 0x0067, 0x0065, 0x0000];
pub static CPP_COM_WSTR_E2A6D0759FD4: [WCHAR; 17] = [0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x0052, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut g_hModule: HMODULE = core::ptr::null_mut();
#[no_mangle]
pub static mut g_ModuleRefCnt: i32 = 0;
#[no_mangle]
pub static mut g_FFunc: zlib_filefunc64_def = unsafe { core::mem::zeroed() };

unsafe extern "C" fn init_zlib() {
    fill_win32_filefunc64W(core::ptr::addr_of_mut!(g_FFunc));
}

unsafe extern "C" fn CreateEmptyFile(pszFile: PCWSTR) -> BOOL {
    let mut hFile: HANDLE;
    hFile = CreateFileW(pszFile, GENERIC_WRITE, FILE_SHARE_READ, core::ptr::null_mut(),
                        CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, core::ptr::null_mut());
    if hFile != INVALID_HANDLE_VALUE {
        CloseHandle(hFile);
        return (TRUE) as BOOL;
    }
    return (FALSE) as BOOL;
}

unsafe extern "C" fn CreateSendToZip(pszSendTo: PCWSTR) -> HRESULT {
    let mut szTarget: [u16; (MAX_PATH) as _] = [0u16; (MAX_PATH) as _];
    let mut szSendToFile: [u16; (MAX_PATH) as _] = [0u16; (MAX_PATH) as _];

    LoadStringW(g_hModule, IDS_FRIENDLYNAME, szTarget.as_mut_ptr(), (_countof!(szTarget)) as i32);

    StringCbCopyW(szSendToFile.as_mut_ptr(), (core::mem::size_of_val(&szSendToFile) as i32) as usize, pszSendTo);
    PathAppendW(szSendToFile.as_mut_ptr(), szTarget.as_mut_ptr());
    StringCbCatW(szSendToFile.as_mut_ptr(), (core::mem::size_of_val(&szSendToFile) as i32) as usize, CPP_COM_WSTR_84DE1F778E09.as_ptr());
    if CreateEmptyFile(szSendToFile.as_mut_ptr()) == 0
    {
        DPRINT1!("CreateEmptyFile('%ls')\n", szSendToFile.as_ptr());
        return E_FAIL;
    }
    return S_OK;
}

unsafe extern "C" fn GetDefaultUserSendTo(pszPath: PWSTR) -> HRESULT {
    return SHGetFolderPathW(core::ptr::null_mut(), (CSIDL_SENDTO as i32), INVALID_HANDLE_VALUE,
                            SHGFP_TYPE_DEFAULT, pszPath);
}
#[no_mangle]
pub unsafe extern "C" fn GetZipCodePage(bUnZip: BOOL) -> UINT {
    let mut szValue = [0u16; 16];
    let mut dwType: DWORD = 0;
    let mut cbValue: DWORD = (core::mem::size_of::<[u16; 16]>()) as DWORD;
    let nDefaultCodePage: UINT = if bUnZip != 0 { CP_ACP } else { CP_UTF8 };

    let error: LONG = (SHGetValueW(HKEY_CURRENT_USER, CPP_COM_WSTR_E2A6D0759FD4.as_ptr(),
                             if bUnZip != 0 { CPP_COM_WSTR_D8643F50C021.as_ptr() } else { CPP_COM_WSTR_2AA7531E01FA.as_ptr() },
                             core::ptr::addr_of_mut!(dwType), szValue.as_mut_ptr() as *mut core::ffi::c_void, core::ptr::addr_of_mut!(cbValue))) as i32;
    if error != (ERROR_SUCCESS as LONG) {
        return nDefaultCodePage;
    }

    if cbValue == (core::mem::size_of::<DWORD>()) as DWORD && (dwType == REG_DWORD || dwType == REG_BINARY) {
        return *(szValue.as_ptr() as *const DWORD);
    }

    if dwType != REG_SZ && dwType != REG_EXPAND_SZ {
        return nDefaultCodePage;
    }

    szValue[(_countof!(szValue) - 1) as usize] = UNICODE_NULL;
    return wcstol(szValue.as_mut_ptr() as *const u16, core::ptr::null_mut(), 0) as UINT;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hInstance: HINSTANCE, dwReason: DWORD, lpReserved: LPVOID) -> BOOL {
    match dwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hInstance);
            g_hModule = hInstance;
            gModule.Init((ObjectMap).as_ptr(), hInstance, core::ptr::null::<GUID>());
            init_zlib();
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    if g_ModuleRefCnt != 0 {
        return S_FALSE;
    }
    return gModule.DllCanUnloadNow();
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(rclsid: *const GUID, riid: *const GUID, ppv: *mut LPVOID) -> HRESULT {
    return gModule.DllGetClassObject(rclsid, riid, ppv);
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    let mut hr: HRESULT;

    hr = gModule.DllRegisterServer((FALSE as i32));
    if FAILED_UNEXPECTEDLY!(hr) != false {
        return hr;
    }

    hr = gModule.UpdateRegistryFromResource(IDR_ZIPFLDR, (TRUE as i32), core::ptr::null::<c_void>());
    if (hr) < 0 {
        return hr;
    }

    let mut szSendTo = [0u16; MAX_PATH as usize];
    hr = GetDefaultUserSendTo(szSendTo.as_mut_ptr());
    if (hr) >= 0 {
        CreateSendToZip(szSendTo.as_mut_ptr());
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    let mut hr: HRESULT;

    hr = gModule.DllUnregisterServer((FALSE as i32));
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    hr = gModule.UpdateRegistryFromResource(IDR_ZIPFLDR, (FALSE as i32), core::ptr::null::<c_void>());
    if hr < 0 {
        return hr;
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn RouteTheCallW(hWndOwner: HWND, hInstance: HINSTANCE, lpStringArg: PCWSTR, Show: INT) -> BOOL {
    let mut path = CStringW::from_ptr(lpStringArg);
    PathRemoveBlanksW(path.GetBuffer(-1));
    path.ReleaseBuffer();
    let mut tmp = CStringW::from_ptr(CPP_COM_WSTR_1B9334FEECE6.as_ptr());
    tmp += path.GetString();
    tmp += CPP_COM_WSTR_1B9334FEECE6.as_ptr();
    path = tmp;

    let mut app = [0 as WCHAR; (MAX_PATH) as _];
    GetWindowsDirectoryW(app.as_mut_ptr(), _countof!(app) as UINT);
    PathAppendW(app.as_mut_ptr(), CPP_COM_WSTR_AB68B18B313D.as_ptr());

    ShellExecuteW(core::ptr::null_mut(), CPP_COM_WSTR_82DB442E7A60.as_ptr(), app.as_mut_ptr(), path.GetString(), core::ptr::null_mut(), if Show != 0 { Show } else { (SW_SHOWNORMAL as i32) });
    return (TRUE) as BOOL;
}
