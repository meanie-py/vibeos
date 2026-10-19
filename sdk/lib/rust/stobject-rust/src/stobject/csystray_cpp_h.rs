use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_258F57B256D3: [WCHAR; 10] = [0x0053, 0x0048, 0x0045, 0x004C, 0x004C, 0x0048, 0x004F, 0x004F, 0x004B, 0x0000];
pub static CPP_COM_WSTR_51AA45493C0B: [WCHAR; 13] = [0x0072, 0x0075, 0x006E, 0x0064, 0x006C, 0x006C, 0x0033, 0x0032, 0x002E, 0x0065, 0x0078, 0x0065, 0x0000];
pub static CPP_COM_WSTR_51AE552436FD: [WCHAR; 9] = [0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0073, 0x0000];
pub static CPP_COM_WSTR_7B2B8EE39C4F: [WCHAR; 11] = [0x0025, 0x0068, 0x0073, 0x0020, 0x0025, 0x0068, 0x0073, 0x0025, 0x0068, 0x0073, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[no_mangle]
// `_countof(g_IconHandlers)`, counted off the array TYPE through a raw pointer: a static
// initializer may not read the `static mut` table, and the count needs no read.
pub static g_NumIcons: i32 = {
    const fn len_of<T, const N: usize>(_: *const [T; N]) -> usize { N }
    len_of(unsafe { core::ptr::addr_of!(g_IconHandlers) }) as i32
};


























impl CSysTray {
    pub unsafe extern "system" fn s_SysTrayThreadProc(param: PVOID) -> DWORD {
        let st: *mut CSysTray = param as *mut CSysTray;
        return (*st).SysTrayThreadProc() as DWORD;
    }
    pub unsafe fn SysTrayMessageLoop(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        let mut ret: i32;
        let mut msg: MSG = core::mem::zeroed();
        loop {
            ret = GetMessage(core::ptr::addr_of_mut!(msg), core::ptr::null_mut(), 0, 0);
            if !(ret != 0) {
                break;
            }
            if ret < 0 {
                break;
            }
            TranslateMessage(core::ptr::addr_of_mut!(msg));
            DispatchMessage(core::ptr::addr_of_mut!(msg));
        }
        return S_OK;
    }
    pub unsafe fn SysTrayThreadProc(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        let mut strFileName: [u16; (MAX_PATH) as _] = [0u16; (MAX_PATH) as _];
        GetModuleFileNameW(g_hInstance, strFileName.as_mut_ptr(), MAX_PATH);
        let hLib: HMODULE = LoadLibraryW(strFileName.as_ptr());

        CoInitializeEx(core::ptr::null_mut(), COINIT_DISABLE_OLE1DDE | COINIT_APARTMENTTHREADED);

        (*this_).Create(core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), 0, 0, 0);

        let ret: HRESULT = (*this_).SysTrayMessageLoop();

        CoUninitialize();

        (*this_).Release();
        FreeLibraryAndExitThread(hLib, (ret) as DWORD);
    }
    pub unsafe fn CreateSysTrayThread(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).AddRef();

        let hThread: HANDLE = CreateThread(core::ptr::null_mut(), 0, Some(CSysTray::s_SysTrayThreadProc), this_ as *mut core::ffi::c_void, 0, core::ptr::null_mut());

        CloseHandle(hThread);

        return S_OK;
    }
    pub unsafe fn DestroySysTrayWindow(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).__base2.DestroyWindow() == 0 {
            // Window is from another thread, ask it politely to destroy itself:
            (*this_).__base2.SendMessageW__0(WM_CLOSE, 0, 0);
        }
        return S_OK;
    }
    pub unsafe fn InitIcons(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        let mut i: UINT = 0;
        while i < (g_NumIcons as UINT) {
            if ((*this_).dwServicesEnabled & g_IconHandlers[i as usize].dwServiceFlag) != 0 {
                let hr: HRESULT = ((g_IconHandlers[i as usize].pfnInit).unwrap_unchecked())(this_);
                if hr < 0 {
                    return hr;
                }
            }
            i += 1;
        }
        let mut i: UINT = 0;
        while i < (_countof!(g_StandaloneHandlers) as UINT) {
            ((g_StandaloneHandlers[i as usize].pfnInit).unwrap_unchecked())(this_);
            i += 1;
        }

        (*this_).InitNetShell()
    }
    pub unsafe fn ShutdownIcons(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        let mut i: UINT = 0;
        while i < g_NumIcons as UINT {
            if ((*this_).dwServicesEnabled & g_IconHandlers[i as usize].dwServiceFlag) != 0 {
                (*this_).dwServicesEnabled &= !g_IconHandlers[i as usize].dwServiceFlag;
                let hr: HRESULT = ((g_IconHandlers[i as usize].pfnShutdown).unwrap_unchecked())(this_);
                FAILED_UNEXPECTEDLY!(hr);
            }
            i += 1;
        }
        let mut i: UINT = 0;
        while i < _countof!(g_StandaloneHandlers) as UINT {
            ((g_StandaloneHandlers[i as usize].pfnShutdown).unwrap_unchecked())(this_);
            i += 1;
        }

        (*this_).ShutdownNetShell()
    }
    pub unsafe fn UpdateIcons(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        let mut i: i32 = 0;
        while i < g_NumIcons {
            if ((*this_).dwServicesEnabled & g_IconHandlers[i as usize].dwServiceFlag) != 0 {
                let hr: HRESULT = ((g_IconHandlers[i as usize].pfnUpdate).unwrap_unchecked())(this_);
                if (hr < 0) {
                    return hr;
                }
            }
            i += 1;
        }

        return S_OK;
    }
    pub unsafe fn ProcessIconMessage(&mut self, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, lResult: &mut LRESULT) -> HRESULT {
        let this_: *mut Self = self;
        let mut i: UINT = 0;
        while i < g_NumIcons as UINT {
            let hr: HRESULT = ((g_IconHandlers[i as usize].pfnMessage).unwrap_unchecked())(this_, uMsg, wParam, lParam, lResult);
            if hr < 0 {
                return hr;
            }
            if hr == S_OK {
                return hr;
            }
            i += 1;
        }
        let mut i: UINT = 0;
        while i < _countof!(g_StandaloneHandlers) as UINT {
            let hr: HRESULT = ((g_StandaloneHandlers[i as usize].pfnMessage).unwrap_unchecked())(this_, uMsg, wParam, lParam, lResult);
            if hr < 0 {
                return hr;
            }
            if hr == S_OK {
                return hr;
            }
            i += 1;
        }

        // Not handled by anyone, so return accordingly.
        return S_FALSE;
    }
    pub unsafe fn InitNetShell(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        let hr: HRESULT = CoCreateInstance(&CLSID_ConnectionTray, core::ptr::null_mut(), 1, core::ptr::addr_of!(IID_IOleCommandTarget), core::ptr::addr_of_mut!((*this_).pctNetShell) as *mut *mut core::ffi::c_void);
        if hr < 0 {
            return hr;
        }

        return (*(*this_).pctNetShell.p).Exec(&CGID_ShellServiceObject,
                                 OLECMDID_NEW,
                                 OLECMDEXECOPT_DODEFAULT, core::ptr::null_mut(), core::ptr::null_mut());
    }
    pub unsafe fn ShutdownNetShell(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).pctNetShell.p.is_null() {
            return S_FALSE;
        }
        let hr: HRESULT = (*(*this_).pctNetShell.p).Exec(
            &CGID_ShellServiceObject as *const GUID,
            OLECMDID_SAVE,
            OLECMDEXECOPT_DODEFAULT,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        (*this_).pctNetShell.Release();
        return hr;
    }
    pub unsafe fn GetServicesEnabled(&mut self) {
        let this_: *mut Self = self;
        let mut hKey: HKEY = core::ptr::null_mut();
        let mut dwSize: DWORD;

        /* Enable power, volume and hotplug by default */
        (*this_).dwServicesEnabled = POWER_SERVICE_FLAG | VOLUME_SERVICE_FLAG | HOTPLUG_SERVICE_FLAG;

        if RegCreateKeyExW(HKEY_CURRENT_USER, REGSTR_PATH_SYSTRAY,
                            0,
                            core::ptr::null_mut(),
                            REG_OPTION_NON_VOLATILE,
                            KEY_READ,
                            core::ptr::null_mut(),
                            core::ptr::addr_of_mut!(hKey),
                            core::ptr::null_mut()) == ERROR_SUCCESS as _
        {
            dwSize = core::mem::size_of::<DWORD>() as DWORD;
            RegQueryValueExW(hKey,
                             CPP_COM_WSTR_51AE552436FD.as_ptr(),
                             core::ptr::null_mut(),
                             core::ptr::null_mut(),
                             core::ptr::addr_of_mut!((*this_).dwServicesEnabled) as LPBYTE,
                             core::ptr::addr_of_mut!(dwSize));

            RegCloseKey(hKey);
        }
    }
    pub unsafe fn NotifyIcon(&mut self, code: INT, uId: UINT, hIcon: HICON, szTip: LPCWSTR, dwstate: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        let mut nim: NOTIFYICONDATA = core::mem::zeroed();

        nim.cbSize = core::mem::size_of::<NOTIFYICONDATA>() as DWORD;
        nim.uFlags = NIF_MESSAGE | NIF_ICON | NIF_STATE | NIF_TIP;
        nim.hIcon = hIcon;
        nim.uID = uId;
        nim.uCallbackMessage = uId;
        nim.dwState = dwstate;
        nim.dwStateMask = NIS_HIDDEN;
        nim.hWnd = (*this_).__base2.m_hWnd;
        nim.__bindgen_anon_1.uVersion = NOTIFYICON_VERSION;
        if !szTip.is_null() {
            StringCchCopy(core::ptr::addr_of_mut!(nim.szTip).cast(), (_countof!({ nim.szTip })) as _, szTip);
        } else {
            nim.szTip[0 as usize] = 0;
        }
        let ret: BOOL = Shell_NotifyIcon(code as DWORD, core::ptr::addr_of_mut!(nim));
        return if ret != 0 { S_OK } else { E_FAIL };
    }
    pub unsafe fn EnableService(&mut self, dwServiceFlag: DWORD, bEnable: BOOL) {
        let this_: *mut Self = self;
        let mut hKey: HKEY = core::mem::zeroed();

        if bEnable != 0 {
            (*this_).dwServicesEnabled |= dwServiceFlag;
        } else {
            (*this_).dwServicesEnabled &= !dwServiceFlag;
        }

        if RegCreateKeyExW(HKEY_CURRENT_USER,
                            REGSTR_PATH_SYSTRAY,
                            0,
                            core::ptr::null_mut(),
                            REG_OPTION_NON_VOLATILE,
                            KEY_WRITE,
                            core::ptr::null_mut(),
                            core::ptr::addr_of_mut!(hKey),
                            core::ptr::null_mut()) == (ERROR_SUCCESS as i32)
        {
            let mut dwConfig: DWORD = (*this_).dwServicesEnabled & !STANDALONESERVICEMASK;
            RegSetValueExW(hKey,
                           CPP_COM_WSTR_51AE552436FD.as_ptr(),
                           0,
                           REG_DWORD,
                           core::ptr::addr_of_mut!(dwConfig) as LPBYTE,
                           core::mem::size_of::<DWORD>() as DWORD);

            RegCloseKey(hKey);
        }

        (*this_).ConfigurePollTimer();
    }
    pub unsafe fn IsServiceEnabled(&mut self, dwServiceFlag: DWORD) -> BOOL {
        let this_: *mut Self = self;
        return ((*this_).dwServicesEnabled & dwServiceFlag) as BOOL;
    }
    pub unsafe fn ConfigurePollTimer(&mut self) {
        let this_: *mut Self = self;
        // FIXME: VOLUME_SERVICE_FLAG uses mixerOpen(CALLBACK_WINDOW) but the timer
        //        still has to be here because CORE-20778
        // FIXME: POWER_SERVICE_FLAG should use WM_DEVICECHANGE, WM_POWERBROADCAST

        let fNeedsTimer: DWORD = POWER_SERVICE_FLAG;
        if ((*this_).dwServicesEnabled & fNeedsTimer) != 0 {
            (*this_).__base2.SetTimer((POLL_TIMER_ID as _), 2000, None);
        } else {
            (*this_).__base2.KillTimer((POLL_TIMER_ID as _));
        }
    }
    pub unsafe fn RunDll(Dll: PCSTR, Parameters: PCSTR) {
        let mut buf = [0u16; 400];
        let mut rundll = [0u16; (MAX_PATH) as _];
        GetSystemDirectory(rundll.as_mut_ptr(), _countof!(rundll) as u32);
        PathAppendW(rundll.as_mut_ptr(), CPP_COM_WSTR_51AA45493C0B.as_ptr());

        wsprintfW(buf.as_mut_ptr(), CPP_COM_WSTR_7B2B8EE39C4F.as_ptr(), b"shell32.dll,Control_RunDLL\0".as_ptr(), Dll, Parameters);
        ShellExecuteW(core::ptr::null_mut(), core::ptr::null(), rundll.as_ptr(), buf.as_ptr(), core::ptr::null(), (SW_SHOW) as INT);
    }
    pub unsafe fn RunAccessCpl(Parameters: PCSTR) {
        CSysTray::RunDll(b"access.cpl\0".as_ptr() as PCSTR, Parameters);
    }
    pub unsafe fn ProcessWindowMessage(&mut self, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, lResult: &mut LRESULT, dwMsgMapID: DWORD) -> BOOL {
        let this_: *mut Self = self;
        let hr: HRESULT;

        if hWnd != (*this_).__base2.m_hWnd {
            return (FALSE) as BOOL;
        }

        if uMsg == (*this_).wm_SHELLHOOK && (*this_).wm_SHELLHOOK != 0 {
            if wParam == (HSHELL_ACCESSIBILITYSTATE as _) && lParam == (ACCESS_STICKYKEYS as LPARAM) {
                StickyKeys_Update!(this_);
            } else if wParam == (HSHELL_ACCESSIBILITYSTATE as _) && lParam == (ACCESS_MOUSEKEYS as LPARAM) {
                MouseKeys_Update(this_);
            }
            *lResult = 0;
            return (TRUE) as BOOL;
        }

        match uMsg {
            WM_NCCREATE | WM_NCDESTROY | MM_MIXM_CONTROL_CHANGE => {
                return (FALSE) as BOOL;
            }

            WM_CLOSE => {
                return (*this_).__base2.DestroyWindow();
            }

            WM_CREATE => {
                (*this_).GetServicesEnabled();
                (*this_).InitIcons();
                RegisterShellHookWindow(hWnd);
                (*this_).ConfigurePollTimer();
                return (TRUE) as BOOL;
            }

            WM_TIMER => {
                if wParam == (POLL_TIMER_ID as _) {
                    (*this_).UpdateIcons();
                } else {
                    (*this_).ProcessIconMessage(uMsg, wParam, lParam, lResult);
                }
                return (TRUE) as BOOL;
            }

            WM_SETTINGCHANGE => {
                if wParam == (SPI_SETSTICKYKEYS as _) {
                    StickyKeys_Update!(this_);
                }
                if wParam == (SPI_SETMOUSEKEYS as _) {
                    MouseKeys_Update(this_);
                }
            }

            MM_MIXM_LINE_CHANGE => {
                (*this_).ProcessIconMessage(uMsg, wParam, lParam, lResult);
                return (TRUE) as BOOL;
            }

            WM_DESTROY => {
                (*this_).__base2.KillTimer((POLL_TIMER_ID as _));
                DeregisterShellHookWindow(hWnd);
                (*this_).ShutdownIcons();
                PostQuitMessage(0);
                return (TRUE) as BOOL;
            }

            _ => {}
        }

        hr = (*this_).ProcessIconMessage(uMsg, wParam, lParam, lResult);
        if hr < 0 {
            return (FALSE) as BOOL;
        }

        return (hr == S_OK) as BOOL;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base2.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        (*this_).dwServicesEnabled = 0;
        {
            (*this_).wm_SHELLHOOK = RegisterWindowMessageW(CPP_COM_WSTR_258F57B256D3.as_ptr());
        }
    }
    pub unsafe fn QueryStatus(&mut self, pguidCmdGroup: *const GUID,
        cCmds: ULONG,
        prgCmds: *mut OLECMD,
        pCmdText: *mut OLECMDTEXT,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // UNIMPLEMENTED;
        S_OK
    }
    pub unsafe fn Exec(&mut self, pguidCmdGroup: *const GUID, nCmdID: DWORD, nCmdexecopt: DWORD, pvaIn: *mut VARIANT, pvaOut: *mut VARIANT) -> HRESULT {
        let this_: *mut Self = self;
        if !IsEqualGUID!(pguidCmdGroup, &CGID_ShellServiceObject) {
            return E_FAIL;
        }

        if nCmdID == OLECMDID_NEW { // init
            return (*this_).CreateSysTrayThread();
        } else if nCmdID == OLECMDID_SAVE { // shutdown
            return (*this_).DestroySysTrayWindow();
        }
        return S_OK;
    }
}
