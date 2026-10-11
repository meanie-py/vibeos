use crate::prelude::*;
use crate::msctf::categorymgr_cpp_h::CategoryMgr_Constructor;
use crate::msctf::displayattributemgr_cpp_h::DisplayAttributeMgr_Constructor;
use crate::msctf::mlng_cpp_h::UninitINAT;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_49580201C969: [WCHAR; 9] = [0x0046, 0x0069, 0x006C, 0x0065, 0x0050, 0x0061, 0x0074, 0x0068, 0x0000];

use core::ffi::c_void;

extern "system" {
    fn OpenMutexW(dwDesiredAccess: DWORD, bInheritHandle: BOOL, lpName: LPCWSTR) -> HANDLE;
    fn CloseHandle(hObject: HANDLE) -> BOOL;
}

// ---- file-scope preamble (translated) ----
pub static mut gf_CRT_INIT: BOOL = 0;
pub static mut g_fDllProcessDetached: BOOL = 0;
pub static mut g_cs: CRITICAL_SECTION = unsafe { core::mem::zeroed() };
pub static mut g_csInDllMain: CRITICAL_SECTION = unsafe { core::mem::zeroed() };
pub static mut g_csDelayLoad: CRITICAL_SECTION = unsafe { core::mem::zeroed() };
pub static mut g_hInst: HINSTANCE = core::ptr::null_mut();
pub static mut g_bOnWow64: BOOL = 0;
pub static mut g_uACP: UINT = CP_ACP;
pub static mut g_dwOSInfo: DWORD = 0;
pub static mut g_hklDefault: HKL = core::ptr::null_mut();
#[no_mangle]
pub static mut g_dwTLSIndex: DWORD = unsafe { core::mem::zeroed() };
pub static mut gfSharedMemory: BOOL = 0;
pub static mut g_cRefDll: LONG = -1;
pub static mut g_fCUAS: BOOL = 0;
pub static mut g_szCUASImeFile: [TCHAR; 16] = [0; 16];
pub static mut g_dwAppCompatibility: DWORD = 0;
pub static mut g_fNoITextStoreAnchor: BOOL = 1;
pub static mut g_msgPrivate: UINT = 0;
pub static mut g_msgSetFocus: UINT = 0;
pub static mut g_msgThreadTerminate: UINT = 0;
pub static mut g_msgThreadItemChange: UINT = 0;
pub static mut g_msgLBarModal: UINT = 0;
pub static mut g_msgRpcSendReceive: UINT = 0;
pub static mut g_msgThreadMarshal: UINT = 0;
pub static mut g_msgCheckThreadInputIdel: UINT = 0;
pub static mut g_msgStubCleanUp: UINT = 0;
pub static mut g_msgShowFloating: UINT = 0;
pub static mut g_msgLBUpdate: UINT = 0;
pub static mut g_msgNuiMgrDirtyUpdate: UINT = 0;
pub static mut g_fUserSidString: BOOL = 0;
pub static mut g_szUserSidString: [TCHAR; 260] = [0; 260];
pub static mut g_szUserUnique: [TCHAR; 260] = [0; 260];
pub static mut g_szAsmListCache: [TCHAR; 260] = [0; 260];
pub static mut g_szTimListCache: [TCHAR; 260] = [0; 260];
pub static mut g_szLayoutsCache: [TCHAR; 260] = [0; 260];
pub static mut g_mutexLBES: CicMutex = unsafe { core::mem::zeroed() };
pub static mut g_mutexCompart: CicMutex = unsafe { core::mem::zeroed() };
pub static mut g_mutexAsm: CicMutex = unsafe { core::mem::zeroed() };
pub static mut g_mutexLayouts: CicMutex = unsafe { core::mem::zeroed() };
pub static mut g_mutexTMD: CicMutex = unsafe { core::mem::zeroed() };
pub static mut g_SharedMemory: CicFileMappingStatic = unsafe { core::mem::zeroed() };
pub static mut g_uLangHotKeyModifiers: UINT = 0;
pub static mut g_uLangHotKeyVKey: UINT = 0;
pub static mut g_uLangHotKeyVKey2: UINT = 0;
pub static mut g_uKeyTipHotKeyModifiers: UINT = 0;
pub static mut g_uKeyTipHotKeyVKey: UINT = 0;
pub static mut g_uKeyTipHotKeyVKey2: UINT = 0;

#[no_mangle]
pub unsafe extern "C" fn __cxa_pure_virtual() {
    // ERR("__cxa_pure_virtual\n");
    DebugBreak();
}
#[no_mangle]
pub unsafe extern "C" fn GetUserSIDString() -> LPTSTR {
    let mut hToken: HANDLE = core::ptr::null_mut();
    OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, core::ptr::addr_of_mut!(hToken));
    if hToken.is_null() {
        return core::ptr::null_mut();
    }

    let mut dwLengthNeeded: DWORD = 0;
    GetTokenInformation(hToken, TokenUser, core::ptr::null_mut(), 0, core::ptr::addr_of_mut!(dwLengthNeeded));
    let pTokenUser: PTOKEN_USER = cicMemAllocClear((dwLengthNeeded) as _) as PTOKEN_USER;
    if pTokenUser.is_null() {
        CloseHandle(hToken);
        return core::ptr::null_mut();
    }

    let mut StringSid: LPTSTR = core::ptr::null_mut();
    if GetTokenInformation(hToken, TokenUser, pTokenUser as *mut core::ffi::c_void, dwLengthNeeded, core::ptr::addr_of_mut!(dwLengthNeeded)) == 0
        || ConvertSidToStringSid((*pTokenUser).User.Sid, core::ptr::addr_of_mut!(StringSid)) == 0
    {
        if !StringSid.is_null() {
            LocalFree(StringSid as *mut core::ffi::c_void);
            StringSid = core::ptr::null_mut();
        }
    }

    cicMemFree(pTokenUser as *mut core::ffi::c_void);
    CloseHandle(hToken);
    StringSid
}
#[no_mangle]
pub unsafe extern "C" fn InitUserSidString() -> BOOL {
    if g_fUserSidString != 0 {
        return (TRUE) as BOOL;
    }

    let pszUserSID: LPTSTR = GetUserSIDString();
    if pszUserSID.is_null() {
        return (FALSE) as BOOL;
    }

    StringCchCopy(core::ptr::addr_of_mut!(g_szUserSidString) as LPTSTR, (_countof!(g_szUserSidString)) as _, pszUserSID);
    g_fUserSidString = (TRUE) as i32;
    LocalFree(pszUserSID as _);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn InitUniqueString() -> BOOL {
    g_szUserUnique[0] = ('\0' as u16);

    let dwThreadId: DWORD = GetCurrentThreadId();
    let hDesk: HDESK = GetThreadDesktop(dwThreadId);

    let mut nLengthNeeded: DWORD = 0;
    let mut szName = [0 as TCHAR; (MAX_PATH) as _];
    if (hDesk as usize) != 0 && GetUserObjectInformation((hDesk).cast(), (UOI_NAME as i32), szName.as_mut_ptr() as *mut core::ffi::c_void, _countof!(szName) as DWORD, core::ptr::addr_of_mut!(nLengthNeeded)) != 0 {
        StringCchCat(addr_of_mut!(g_szUserUnique) as *mut TCHAR, (_countof!(g_szUserUnique)) as _, szName.as_ptr());
    }

    if InitUserSidString() == 0 {
        return (FALSE) as BOOL;
    }

    StringCchCat(addr_of_mut!(g_szUserUnique) as *mut TCHAR, (_countof!(g_szUserUnique)) as _, g_szUserSidString.as_ptr());
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn GetDesktopUniqueName(pszName: LPCTSTR, pszBuff: LPTSTR, cchBuff: UINT) {
    StringCchCopy(pszBuff, (cchBuff) as _, pszName);
    StringCchCat(pszBuff, (cchBuff) as _, core::ptr::addr_of!(g_szUserUnique) as LPCTSTR);
}
#[no_mangle]
pub unsafe extern "C" fn StringFromGUID2A(rguid: REFGUID, pszGUID: LPSTR, cchGUID: INT) -> BOOL {
    *pszGUID.add(0 as usize) = ANSI_NULL as i8;

    let mut szWide: [u16; 40 as usize] = [0u16; 40 as usize];
    szWide[0 as usize] = UNICODE_NULL as u16;
    let ret: BOOL = StringFromGUID2(rguid, szWide.as_mut_ptr(), _countof!(szWide) as INT);
    WideCharToMultiByte(CP_ACP as UINT, 0 as DWORD, szWide.as_ptr(), -(1 as INT), pszGUID, cchGUID, core::ptr::null_mut(), core::ptr::null_mut());
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn FullPathExec(pszExeFile: LPCTSTR, pszCmdLine: LPCTSTR, nCmdShow: UINT, bSysWinDir: BOOL) -> BOOL {
    let mut si: STARTUPINFO = core::mem::zeroed();
    let mut pi: PROCESS_INFORMATION = core::mem::zeroed();
    let mut ModPath: CicSystemModulePath = core::mem::zeroed();
    let mut szCommandLine: [TCHAR; (2 * MAX_PATH) as _] = [0; (2 * MAX_PATH) as _];

    ModPath.Init(pszExeFile, bSysWinDir);
    if ModPath.m_cchPath == 0
    {
        return (FALSE) as BOOL;
    }

    StringCchCopy(szCommandLine.as_mut_ptr(), (_countof!(szCommandLine)) as _, pszCmdLine);

    ZeroMemory!(core::ptr::addr_of_mut!(si) as *mut _, core::mem::size_of::<STARTUPINFO>());
    si.cb = core::mem::size_of::<STARTUPINFO>() as DWORD;
    si.wShowWindow = nCmdShow as WORD;
    si.dwFlags = STARTF_USESHOWWINDOW;
    if CreateProcess(ModPath.m_szPath.as_mut_ptr(), szCommandLine.as_mut_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), FALSE as BOOL,
                       NORMAL_PRIORITY_CLASS, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(si), core::ptr::addr_of_mut!(pi)) == 0
    {
        return (FALSE) as BOOL;
    }

    CloseHandle(pi.hProcess);
    CloseHandle(pi.hThread);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn RunCPLSetting(pszCmdLine: LPCTSTR) -> BOOL {
    if pszCmdLine.is_null() {
        return (FALSE) as BOOL;
    }

    let rundll32: [u16; 13] = ['r' as u16, 'u' as u16, 'n' as u16, 'd' as u16, 'l' as u16, 'l' as u16, '3' as u16, '2' as u16, '.' as u16, 'e' as u16, 'x' as u16, 'e' as u16, 0];

    return FullPathExec(rundll32.as_ptr(), pszCmdLine, SW_SHOWMINNOACTIVE, (FALSE) as BOOL);
}

#[no_mangle]
pub unsafe extern "system" fn TF_GetThreadFlags(dwThreadId: DWORD, pdwFlags1: LPDWORD, pdwFlags2: LPDWORD, pdwFlags3: LPDWORD) -> BOOL {
    *pdwFlags3 = 0;
    *pdwFlags2 = *pdwFlags3;
    *pdwFlags1 = *pdwFlags2;
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn TF_CreateCategoryMgr(ppcat: *mut *mut ITfCategoryMgr) -> HRESULT {
    return CategoryMgr_Constructor(core::ptr::null_mut(), ppcat as *mut *mut IUnknown);
}

#[no_mangle]
pub unsafe extern "system" fn TF_CreateCicLoadMutex(pfWinLogon: LPBOOL) -> *mut core::ffi::c_void {
    if pfWinLogon.is_null() {
        return core::ptr::null_mut();
    }
    *pfWinLogon = (FALSE) as i32;
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn TF_CreateDisplayAttributeMgr(ppdam: *mut *mut ITfDisplayAttributeMgr) -> HRESULT {
    return DisplayAttributeMgr_Constructor(core::ptr::null_mut(), ppdam as *mut *mut IUnknown);
}

#[no_mangle]
pub unsafe extern "system" fn TF_DllDetachInOther() -> BOOL {
    // FIXME("()\n");
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn TF_GetGlobalCompartment(ppCompMgr: *mut *mut ITfCompartmentMgr) -> HRESULT {
    *ppCompMgr = core::ptr::null_mut();
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn TF_GetLangIcon(
    LangID: LANGID,
    pszText: PWSTR,
    cchText: INT,
) -> HICON {
    // FIXME("(0x%X, %p, %d)\n", LangID, pszText, cchText);
    (NULL).cast()
}

#[no_mangle]
pub unsafe extern "system" fn TF_IsFullScreenWindowAcitvated() -> BOOL {
    // FIXME("()\n");
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn TF_GetInputScope(hWnd: HWND, ppInputScope: *mut *mut ITfInputScope) -> HRESULT {
    *ppInputScope = core::ptr::null_mut();
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn SetInputScopeXML(hwnd: HWND, pszXML: PCWSTR) -> HRESULT {
    // FIXME("(%p, %p)\n", hwnd, pszXML);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn TF_CUASAppFix(pszName: LPCSTR) -> HRESULT {
    if pszName.is_null() || lstrcmpiA(pszName, b"DelayFirstActivateKeyboardLayout\0".as_ptr() as *const i8) != 0 {
        return E_INVALIDARG;
    }
    g_dwAppCompatibility |= CTF_COMPAT_DELAY_FIRST_ACTIVATE;
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn TF_CheckThreadInputIdle(dwThreadId: DWORD, dwMilliseconds: DWORD) -> LONG {
    let _ = (dwThreadId, dwMilliseconds);
    return -1;
}

#[no_mangle]
pub unsafe extern "system" fn TF_ClearLangBarAddIns(guid: REFGUID) -> HRESULT {
    // FIXME("(%p)\n", guid);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn TF_InvalidAssemblyListCache() -> HRESULT {
    // FIXME("()\n");
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn TF_IsInMarshaling(dwThreadId: DWORD) -> BOOL {
    // FIXME("(%lu)\n", dwThreadId);
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn TF_PostAllThreadMsg(wParam: WPARAM, dwFlags: DWORD) -> HRESULT {
    // FIXME("(%p, 0x%X)\n", wParam, dwFlags);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn TF_InitSystem() -> BOOL {
    // FIXME("()\n");
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn TF_UninitSystem() -> BOOL {
    // FIXME("()\n");
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn TF_RegisterLangBarAddIn(
    rguid: REFGUID,
    pszFilePath: LPCWSTR,
    dwFlags: DWORD,
) -> HRESULT {
    if pszFilePath.is_null() || rguid == &GUID_NULL {
        return E_INVALIDARG;
    }

    let mut szBuff = [0 as TCHAR; (MAX_PATH) as _];
    let mut szGUID = [0 as TCHAR; 40];
    StringCchCopy(szBuff.as_mut_ptr(), (_countof!(szBuff)) as _, (&[0x0053, 0x004f, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005c, 0x004d, 0x0069, 0x0063, 0x0072, 0x006f, 0x0073, 0x006f, 0x0066, 0x0074, 0x005c, 0x0043, 0x0054, 0x0046, 0x005c, 0x004c, 0x0061, 0x006e, 0x0067, 0x0042, 0x0061, 0x0072, 0x0041, 0x0064, 0x0064, 0x0049, 0x006e, 0x005c, 0x0000u16]).as_ptr() /* TEXT "SOFTWARE\\Microsoft\\CTF\\LangBarAddIn\\" */);
    StringFromGUID2T(rguid, szGUID.as_mut_ptr(), (_countof!(szGUID)) as i32);
    StringCchCat(szBuff.as_mut_ptr(), (_countof!(szBuff)) as _, szGUID.as_ptr());

    let mut regKey: CicRegKey = core::mem::zeroed();
    let mut hBaseKey: HKEY = if (dwFlags & 1) != 0 {
        HKEY_LOCAL_MACHINE
    } else {
        HKEY_CURRENT_USER
    };
    let mut error: LSTATUS = regKey.Create(hBaseKey, szBuff.as_ptr());
    if (error) as u32 == ERROR_SUCCESS {
        error = regKey.SetSzW(CPP_COM_WSTR_49580201C969.as_ptr(), pszFilePath);
        if (error) as u32 == ERROR_SUCCESS {
            error = regKey.SetDword((&[0x0045, 0x006e, 0x0061, 0x0062, 0x006c, 0x0065, 0x0000u16]).as_ptr() /* TEXT "Enable" */, ((dwFlags & 4) != 0) as DWORD);
        }
    }

    return if (error) as u32 == ERROR_SUCCESS {
        S_OK
    } else {
        E_FAIL
    };
}

#[no_mangle]
pub unsafe extern "system" fn TF_UnregisterLangBarAddIn(rguid: *const GUID, dwFlags: DWORD) -> HRESULT {
    if rguid == &GUID_NULL {
        return E_INVALIDARG;
    }

    let mut szSubKey: [TCHAR; (MAX_PATH) as _] = core::mem::zeroed();
    StringCchCopy(szSubKey.as_mut_ptr(), (_countof!(szSubKey)) as _, (&[0x0053, 0x004f, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005c, 0x004d, 0x0069, 0x0063, 0x0072, 0x006f, 0x0073, 0x006f, 0x0066, 0x0074, 0x005c, 0x0043, 0x0054, 0x0046, 0x005c, 0x004c, 0x0061, 0x006e, 0x0067, 0x0042, 0x0061, 0x0072, 0x0041, 0x0064, 0x0064, 0x0049, 0x006e, 0x005c, 0x0000u16]).as_ptr() /* TEXT "SOFTWARE\\Microsoft\\CTF\\LangBarAddIn\\" */);

    let mut regKey: CicRegKey = core::mem::zeroed();
    let hBaseKey: HKEY = if (dwFlags & 1) != 0 { HKEY_LOCAL_MACHINE } else { HKEY_CURRENT_USER };
    let error: LSTATUS = regKey.Open(hBaseKey, szSubKey.as_ptr(), KEY_ALL_ACCESS);
    let mut hr: HRESULT = E_FAIL;
    if (error) as u32 == ERROR_SUCCESS {
        let mut szGUID: [TCHAR; 40] = core::mem::zeroed();
        StringFromGUID2T(rguid, szGUID.as_mut_ptr(), (_countof!(szGUID)) as i32);
        regKey.RecurseDeleteKey(szGUID.as_ptr());
        hr = S_OK;
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn TF_RunInputCPL() -> HRESULT {
    let mut ModPath: CicSystemModulePath = core::mem::zeroed();
    let mut szCmdLine: [TCHAR; (2 * MAX_PATH) as _] = core::mem::zeroed();

    // NOTE: We don't support Win95/98/Me
    if (g_dwOSInfo & CIC_OSINFO_XPPLUS) != 0 {
        ModPath.Init((&[0x0069, 0x006e, 0x0070, 0x0075, 0x0074, 0x002e, 0x0064, 0x006c, 0x006c, 0x0000u16]).as_ptr() /* TEXT "input.dll" */, (FALSE) as INT);
    } else {
        ModPath.Init((&[0x0069, 0x006e, 0x0070, 0x0075, 0x0074, 0x002e, 0x0063, 0x0070, 0x006c, 0x0000u16]).as_ptr() /* TEXT "input.cpl" */, (FALSE) as INT);
    }

    if ModPath.m_cchPath == 0 {
        return E_FAIL;
    }

    StringCchPrintf!(
        core::ptr::addr_of_mut!(szCmdLine) as LPTSTR,
        _countof!(szCmdLine),
        (&[0x0072, 0x0075, 0x006e, 0x0064, 0x006c, 0x006c, 0x0033, 0x0032, 0x002e, 0x0065, 0x0078, 0x0065, 0x0020, 0x0073, 0x0068, 0x0065, 0x006c, 0x006c, 0x0033, 0x0032, 0x002e, 0x0064, 0x006c, 0x006c, 0x002c, 0x0043, 0x006f, 0x006e, 0x0074, 0x0072, 0x006f, 0x006c, 0x005f, 0x0052, 0x0075, 0x006e, 0x0044, 0x004c, 0x004c, 0x0020, 0x0025, 0x0073, 0x0000u16]).as_ptr() /* TEXT "rundll32.exe shell32.dll,Control_RunDLL %s" */,
        core::ptr::addr_of!(ModPath.m_szPath) as LPCTSTR,
    );
    if RunCPLSetting(core::ptr::addr_of!(szCmdLine) as LPCTSTR) == 0 {
        return E_FAIL;
    }

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn TF_IsCtfmonRunning() -> BOOL {
    let mut szName: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let CtfmonInstMutex: [u16; 16] = [
        'C' as u16, 't' as u16, 'f' as u16, 'm' as u16, 'o' as u16, 'n' as u16,
        'I' as u16, 'n' as u16, 's' as u16, 't' as u16, 'M' as u16, 'u' as u16,
        't' as u16, 'e' as u16, 'x' as u16, 0,
    ];
    GetDesktopUniqueName(CtfmonInstMutex.as_ptr(), szName.as_mut_ptr(), _countof!(szName) as UINT);

    let hMutex: HANDLE = OpenMutex(MUTEX_ALL_ACCESS, (FALSE) as i32, szName.as_ptr());
    if hMutex.is_null() {
        return (FALSE) as BOOL;
    }

    CloseHandle(hMutex);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn InitLangChangeHotKey() -> BOOL {
    let mut regKey: CicRegKey = core::mem::zeroed();
    let mut szLanguage: [TCHAR; 2] = [0; 2];
    let mut szLayout: [TCHAR; 2] = [0; 2];
    let mut error: LSTATUS;

    szLanguage[0] = ('3' as u16);
    szLayout[0] = ('3' as u16);
    szLanguage[1] = ('\0' as u16);
    szLayout[1] = ('\0' as u16);

    error = regKey.Open(HKEY_CURRENT_USER, (&[0x004b, 0x0065, 0x0079, 0x0062, 0x006f, 0x0061, 0x0072, 0x0064, 0x0020, 0x004c, 0x0061, 0x0079, 0x006f, 0x0075, 0x0074, 0x005c, 0x0054, 0x006f, 0x0067, 0x0067, 0x006c, 0x0065, 0x0000u16]).as_ptr() /* TEXT "Keyboard Layout\\Toggle" */, KEY_READ);
    if (error) as u32 == ERROR_SUCCESS {
        error = regKey.QuerySz((&[0x004c, 0x0061, 0x006e, 0x0067, 0x0075, 0x0061, 0x0067, 0x0065, 0x0020, 0x0048, 0x006f, 0x0074, 0x006b, 0x0065, 0x0079, 0x0000u16]).as_ptr() /* TEXT "Language Hotkey" */, szLanguage.as_mut_ptr(), _countof!(szLanguage));
        if (error) as u32 != ERROR_SUCCESS {
            if g_dwOSInfo & CIC_OSINFO_NT != 0 {
                error = regKey.QuerySz((&[0x0048, 0x006f, 0x0074, 0x006b, 0x0065, 0x0079, 0x0000u16]).as_ptr() /* TEXT "Hotkey" */, szLanguage.as_mut_ptr(), _countof!(szLanguage));
                if (error) as u32 != ERROR_SUCCESS {
                    szLanguage[0] = ('1' as u16);
                }
            } else {
                error = regKey.QuerySz(core::ptr::null_mut(), szLanguage.as_mut_ptr(), _countof!(szLanguage));
                if (error) as u32 != ERROR_SUCCESS {
                    szLanguage[0] = ('1' as u16);
                }
            }

            if (PRIMARYLANGID!(GetSystemDefaultLCID())) as u32 == LANG_CHINESE {
                szLanguage[0] = ('1' as u16);
            }
        }

        error = regKey.QuerySz((&[0x004c, 0x0061, 0x0079, 0x006f, 0x0075, 0x0074, 0x0020, 0x0048, 0x006f, 0x0074, 0x006b, 0x0065, 0x0079, 0x0000u16]).as_ptr() /* TEXT "Layout Hotkey" */, szLayout.as_mut_ptr(), _countof!(szLayout));
        if (error) as u32 != ERROR_SUCCESS {
            szLayout[0] = ('1' as u16);
            if szLanguage[0] != ('2' as u16) {
                szLayout[0] = ('2' as u16);
            }
            if GetSystemMetrics((SM_MIDEASTENABLED as i32)) != 0 {
                szLayout[0] = ('3' as u16);
            }
        }

        szLanguage[1] = ('\0' as u16);
        szLayout[1] = ('\0' as u16);
    }

    if szLanguage[0] == szLayout[0] {
        if szLanguage[0] == ('1' as u16) {
            szLayout[0] = ('2' as u16);
        } else if szLanguage[0] == ('2' as u16) {
            szLayout[0] = ('1' as u16);
        } else {
            szLayout[0] = ('3' as u16);
        }
    }

    EnterCriticalSection(core::ptr::addr_of_mut!(g_csInDllMain));

    let langSel: TCHAR = szLanguage[0];
    if langSel == ('2' as u16) {
        g_uLangHotKeyModifiers = MOD_SHIFT | MOD_CONTROL;
        g_uLangHotKeyVKey2 = VK_CONTROL;
        g_uLangHotKeyVKey = VK_SHIFT;
    } else if langSel == ('3' as u16) {
        g_uLangHotKeyVKey = 0;
        g_uLangHotKeyModifiers = 0;
        g_uLangHotKeyVKey2 = 0;
    } else if langSel == ('4' as u16) {
        g_uLangHotKeyVKey = VK_NUMPAD0;
        g_uLangHotKeyModifiers = 0;
        g_uLangHotKeyVKey2 = 0;
    } else {
        g_uLangHotKeyModifiers = MOD_SHIFT | MOD_ALT;
        g_uLangHotKeyVKey2 = VK_MENU;
        g_uLangHotKeyVKey = VK_SHIFT;
    }

    let layoutSel: TCHAR = szLayout[0];
    if layoutSel == ('2' as u16) {
        g_uKeyTipHotKeyModifiers = MOD_SHIFT | MOD_CONTROL;
        g_uKeyTipHotKeyVKey = VK_SHIFT;
        g_uKeyTipHotKeyVKey2 = VK_CONTROL;
    } else if layoutSel == ('3' as u16) {
        g_uKeyTipHotKeyModifiers = 0;
        g_uKeyTipHotKeyVKey = 0;
        g_uKeyTipHotKeyVKey2 = 0;
    } else if layoutSel == ('4' as u16) {
        g_uKeyTipHotKeyModifiers = 0;
        g_uKeyTipHotKeyVKey = VK_OEM_3;
        g_uKeyTipHotKeyVKey2 = 0;
    } else {
        g_uKeyTipHotKeyModifiers = 0x40 | MOD_SHIFT;
        g_uKeyTipHotKeyVKey = VK_SHIFT;
        g_uKeyTipHotKeyVKey2 = VK_MENU;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(g_csInDllMain));

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn CheckAnchorStores() {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut error: LSTATUS;
    error = RegOpenKeyEx(HKEY_LOCAL_MACHINE, (&[0x0053, 0x004f, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005c, 0x004d, 0x0069, 0x0063, 0x0072, 0x006f, 0x0073, 0x006f, 0x0066, 0x0074, 0x005c, 0x0043, 0x0054, 0x0046, 0x0000u16]).as_ptr() /* TEXT "SOFTWARE\\Microsoft\\CTF" */, 0, KEY_READ, core::ptr::addr_of_mut!(hKey));
    if (error) as u32 != ERROR_SUCCESS {
        return;
    }

    let mut dwData: DWORD = 0;
    let mut cbData: DWORD = core::mem::size_of::<DWORD>() as DWORD;
    error = RegQueryValueEx(hKey, (&[0x0045, 0x006e, 0x0061, 0x0062, 0x006c, 0x0065, 0x0041, 0x006e, 0x0063, 0x0068, 0x006f, 0x0072, 0x0043, 0x006f, 0x006e, 0x0074, 0x0065, 0x0078, 0x0074, 0x0000u16]).as_ptr() /* TEXT "EnableAnchorContext" */, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(dwData) as PBYTE, core::ptr::addr_of_mut!(cbData));
    if (error) as u32 == ERROR_SUCCESS && cbData == core::mem::size_of::<DWORD>() as DWORD && dwData == 1 {
        g_fNoITextStoreAnchor = (FALSE) as i32;
    }

    RegCloseKey(hKey);
}
#[no_mangle]
pub unsafe extern "C" fn InitCUASFlag() {
    let mut regKey1: CicRegKey = core::mem::zeroed();
    let mut error: LSTATUS;

    error = regKey1.Open(HKEY_LOCAL_MACHINE, (&[0x0053, 0x004f, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005c, 0x004d, 0x0069, 0x0063, 0x0072, 0x006f, 0x0073, 0x006f, 0x0066, 0x0074, 0x005c, 0x0043, 0x0054, 0x0046, 0x005c, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006d, 0x0053, 0x0068, 0x0061, 0x0072, 0x0065, 0x0064, 0x005c, 0x0000u16]).as_ptr() /* TEXT "SOFTWARE\\Microsoft\\CTF\\SystemShared\\" */, KEY_READ);
    if (error) as u32 == ERROR_SUCCESS {
        let mut dwValue: DWORD = 0;
        error = regKey1.QueryDword((&[0x0043, 0x0055, 0x0041, 0x0053, 0x0000u16]).as_ptr() /* TEXT "CUAS" */, &mut dwValue);
        if (error) as u32 == ERROR_SUCCESS {
            g_fCUAS = (dwValue != 0) as BOOL;
        }
    }

    (*core::ptr::addr_of_mut!(g_szCUASImeFile))[0] = ('\0' as u16);

    if g_fCUAS == 0 {
        return;
    }

    let mut szImeFile: [TCHAR; 16] = core::mem::zeroed();
    let mut regKey2: CicRegKey = core::mem::zeroed();
    error = regKey2.Open(HKEY_LOCAL_MACHINE,
                         (&[0x0053, 0x006f, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005c, 0x004d, 0x0069, 0x0063, 0x0072, 0x006f, 0x0073, 0x006f, 0x0066, 0x0074, 0x005c, 0x0057, 0x0069, 0x006e, 0x0064, 0x006f, 0x0077, 0x0073, 0x0020, 0x004e, 0x0054, 0x005c, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006e, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006f, 0x006e, 0x005c, 0x0049, 0x004d, 0x004d, 0x0000u16]).as_ptr() /* TEXT "Software\\Microsoft\\Windows NT\\CurrentVersion\\IMM" */, KEY_READ);
    if (error) as u32 == ERROR_SUCCESS {
        error = regKey2.QuerySz((&[0x0049, 0x004d, 0x0045, 0x0020, 0x0046, 0x0069, 0x006c, 0x0065, 0x0000u16]).as_ptr() /* TEXT "IME File" */, szImeFile.as_mut_ptr(), _countof!(szImeFile));
        if (error) as u32 == ERROR_SUCCESS {
            (*core::ptr::addr_of_mut!(g_szCUASImeFile))[(_countof!(g_szCUASImeFile) - 1) as usize] = ('\0' as u16); // Avoid buffer overrun
            StringCchCopy(core::ptr::addr_of_mut!(g_szCUASImeFile) as *mut TCHAR, (_countof!(g_szCUASImeFile)) as _, szImeFile.as_ptr());
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn TFUninitLib() {}
#[no_mangle]
pub unsafe extern "C" fn ProcessAttach(hinstDLL: HINSTANCE) -> BOOL {
    gf_CRT_INIT = (TRUE) as i32;

    InitializeCriticalSectionAndSpinCount(core::ptr::addr_of_mut!(g_cs), 0);
    InitializeCriticalSectionAndSpinCount(core::ptr::addr_of_mut!(g_csInDllMain), 0);
    InitializeCriticalSectionAndSpinCount(core::ptr::addr_of_mut!(g_csDelayLoad), 0);

    g_bOnWow64 = cicIsWow64();
    g_hInst = hinstDLL;
    g_hklDefault = GetKeyboardLayout(0);
    g_dwTLSIndex = TlsAlloc();
    if g_dwTLSIndex == (0u32.wrapping_sub(1 as u32)) {
        return (FALSE) as BOOL;
    }

    g_msgPrivate = RegisterWindowMessageA(b"MSUIM.Msg.Private\0".as_ptr() as *const i8);
    g_msgSetFocus = RegisterWindowMessageA(b"MSUIM.Msg.SetFocus\0".as_ptr() as *const i8);
    g_msgThreadTerminate = RegisterWindowMessageA(b"MSUIM.Msg.ThreadTerminate\0".as_ptr() as *const i8);
    g_msgThreadItemChange = RegisterWindowMessageA(b"MSUIM.Msg.ThreadItemChange\0".as_ptr() as *const i8);
    g_msgLBarModal = RegisterWindowMessageA(b"MSUIM.Msg.LangBarModal\0".as_ptr() as *const i8);
    g_msgRpcSendReceive = RegisterWindowMessageA(b"MSUIM.Msg.RpcSendReceive\0".as_ptr() as *const i8);
    g_msgThreadMarshal = RegisterWindowMessageA(b"MSUIM.Msg.ThreadMarshal\0".as_ptr() as *const i8);
    g_msgCheckThreadInputIdel = RegisterWindowMessageA(b"MSUIM.Msg.CheckThreadInputIdel\0".as_ptr() as *const i8);
    g_msgStubCleanUp = RegisterWindowMessageA(b"MSUIM.Msg.StubCleanUp\0".as_ptr() as *const i8);
    g_msgShowFloating = RegisterWindowMessageA(b"MSUIM.Msg.ShowFloating\0".as_ptr() as *const i8);
    g_msgLBUpdate = RegisterWindowMessageA(b"MSUIM.Msg.LBUpdate\0".as_ptr() as *const i8);
    g_msgNuiMgrDirtyUpdate = RegisterWindowMessageA(b"MSUIM.Msg.MuiMgrDirtyUpdate\0".as_ptr() as *const i8);
    if g_msgPrivate == 0
        || g_msgSetFocus == 0
        || g_msgThreadTerminate == 0
        || g_msgThreadItemChange == 0
        || g_msgLBarModal == 0
        || g_msgRpcSendReceive == 0
        || g_msgThreadMarshal == 0
        || g_msgCheckThreadInputIdel == 0
        || g_msgStubCleanUp == 0
        || g_msgShowFloating == 0
        || g_msgLBUpdate == 0
        || g_msgNuiMgrDirtyUpdate == 0
    {
        return (FALSE) as BOOL;
    }

    cicGetOSInfo(core::ptr::addr_of_mut!(g_uACP), core::ptr::addr_of_mut!(g_dwOSInfo));

    InitUniqueString();

    //FIXME

    gfSharedMemory = (TRUE) as i32;

    //FIXME

    InitCUASFlag();

    //FIXME

    GetDesktopUniqueName((&[0x0043, 0x0054, 0x0046, 0x002e, 0x0041, 0x0073, 0x006d, 0x004c, 0x0069, 0x0073, 0x0074, 0x0043, 0x0061, 0x0063, 0x0068, 0x0065, 0x002e, 0x0046, 0x004d, 0x0050, 0x0000u16]).as_ptr() /* TEXT "CTF.AsmListCache.FMP" */, core::ptr::addr_of_mut!(g_szAsmListCache) as PWSTR, _countof!(g_szAsmListCache) as UINT);
    GetDesktopUniqueName((&[0x0043, 0x0054, 0x0046, 0x002e, 0x0054, 0x0069, 0x006d, 0x004c, 0x0069, 0x0073, 0x0074, 0x0043, 0x0061, 0x0063, 0x0068, 0x0065, 0x002e, 0x0046, 0x004d, 0x0050, 0x0000u16]).as_ptr() /* TEXT "CTF.TimListCache.FMP" */, core::ptr::addr_of_mut!(g_szTimListCache) as PWSTR, _countof!(g_szTimListCache) as UINT);
    GetDesktopUniqueName((&[0x0043, 0x0054, 0x0046, 0x002e, 0x004c, 0x0061, 0x0079, 0x006f, 0x0075, 0x0074, 0x0073, 0x0043, 0x0061, 0x0063, 0x0068, 0x0065, 0x002e, 0x0046, 0x004d, 0x0050, 0x0000u16]).as_ptr() /* TEXT "CTF.LayoutsCache.FMP" */, core::ptr::addr_of_mut!(g_szLayoutsCache) as PWSTR, _countof!(g_szLayoutsCache) as UINT);

    //FIXME

    InitLangChangeHotKey();

    //FIXME

    CheckAnchorStores();

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn ProcessDetach(hinstDLL: HINSTANCE) {
    if gf_CRT_INIT == 0 {
        g_fDllProcessDetached = (TRUE) as i32;
        return;
    }

    if gfSharedMemory != 0 {
        if g_cRefDll != -1 {
            TFUninitLib();
        }
        //FIXME
    }

    UninitINAT();

    //FIXME

    //TF_UninitThreadSystem();

    //FIXME

    if g_dwTLSIndex != (1 as u32).wrapping_neg() as DWORD {
        TlsFree(g_dwTLSIndex);
        g_dwTLSIndex = (1 as u32).wrapping_neg() as DWORD;
    }

    //FIXME

    if gfSharedMemory != 0 {
        (*core::ptr::addr_of_mut!(g_mutexLBES)).Uninit();
        (*core::ptr::addr_of_mut!(g_mutexCompart)).Uninit();
        (*core::ptr::addr_of_mut!(g_mutexAsm)).Uninit();
        //FIXME
        (*core::ptr::addr_of_mut!(g_mutexLayouts)).Uninit();
        (*core::ptr::addr_of_mut!(g_mutexTMD)).Uninit();
        //FIXME
        (*core::ptr::addr_of_mut!(g_SharedMemory)).Close();
    }

    (*core::ptr::addr_of_mut!(g_SharedMemory)).Finalize();

    DeleteCriticalSection(core::ptr::addr_of_mut!(g_cs));
    DeleteCriticalSection(core::ptr::addr_of_mut!(g_csInDllMain));
    DeleteCriticalSection(core::ptr::addr_of_mut!(g_csDelayLoad));

    g_fDllProcessDetached = (TRUE) as i32;
}
