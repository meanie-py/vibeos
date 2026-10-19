use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_9B349295E8D5: [WCHAR; 11] = [0x0070, 0x0072, 0x006F, 0x0070, 0x0065, 0x0072, 0x0074, 0x0069, 0x0065, 0x0073, 0x0000];

use core::ffi::c_void;







#[no_mangle]
pub unsafe extern "system" fn CFontBackgroundMenu_Create(pFontExt: *mut CFontExt, hwnd: HWND, psf: *mut IShellFolder, ppcm: *mut *mut IContextMenu) -> HRESULT {
    let mut dcm: DEFCONTEXTMENU = core::mem::zeroed();
    ZeroMemory!(core::ptr::addr_of_mut!(dcm) as *mut core::ffi::c_void, core::mem::size_of::<DEFCONTEXTMENU>());
    (dcm).hwnd = hwnd;
    (dcm).psf = psf;
    return ShellObjectCreatorInit!(CFontBackgroundMenu, [pFontExt, core::ptr::addr_of_mut!(dcm)], &IID_IContextMenu, ppcm as *mut *mut core::ffi::c_void);
}

impl CFontBackgroundMenu {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_hwnd = core::ptr::null_mut();
        (*this_).m_pFontExt = core::ptr::null_mut();
        (*this_).m_pfnmcb = core::mem::zeroed();
        {
        }
    }
    pub unsafe fn Initialize(&mut self, pFontExt: *mut CFontExt, pdcm: *const DEFCONTEXTMENU) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_pFontExt = pFontExt;
        (*this_).m_psf.assign((*pdcm).psf);
        (*this_).m_pmcb.assign((*pdcm).pcmcb);
        (*this_).m_hwnd = (*pdcm).hwnd;
        return S_OK;
    }
    pub unsafe fn QueryContextMenu(&mut self, hMenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
        let this_: *mut Self = self;
        let strProp = CStringW::from_resource(IDS_PROPERTIES);
        let mut idCmd: INT = idCmdFirst as INT;
        let idCmdOld: INT = idCmd;
        idCmd += 1;
        AppendMenuW(hMenu, MF_STRING, idCmdOld as UINT_PTR, strProp.GetString());
        return MAKE_HRESULT!(SEVERITY_SUCCESS, 0, (idCmd as UINT).wrapping_sub(idCmdFirst));
    }
    pub unsafe fn InvokeCommand(&mut self, lpcmi: LPCMINVOKECOMMANDINFO) -> HRESULT {
        let this_: *mut Self = self;
        let mut idCmd: INT = if IS_INTRESOURCE!((*lpcmi).lpVerb) { LOWORD!((*lpcmi).lpVerb) as INT } else { -1 };

        if idCmd == 0 || (idCmd == -1 && lstrcmpiA((*lpcmi).lpVerb, b"properties\0".as_ptr() as *const i8) == 0)
        {
            // Open "Fonts" properties
            let mut pidl: LPITEMIDLIST = core::ptr::null_mut();
            let hr: HRESULT = SHGetSpecialFolderLocation((*this_).m_hwnd, (CSIDL_FONTS as i32), core::ptr::addr_of_mut!(pidl));
            if FAILED_UNEXPECTEDLY!(hr) || pidl.is_null()
            {
                return E_FAIL;
            }

            let mut sei: SHELLEXECUTEINFOW = SHELLEXECUTEINFOW {
                cbSize: core::mem::size_of::<SHELLEXECUTEINFOW>() as DWORD,
                fMask: (SEE_MASK_INVOKEIDLIST | SEE_MASK_ASYNCOK) as _,
                hwnd: core::ptr::null_mut(),
                lpVerb: CPP_COM_WSTR_9B349295E8D5.as_ptr(),
                lpFile: core::ptr::null_mut(),
                lpParameters: core::ptr::null_mut(),
                lpDirectory: core::ptr::null_mut(),
                nShow: SW_SHOWNORMAL as _,
                hInstApp: core::ptr::null_mut(),
                lpIDList: pidl as _,
                // C++ `= { .. }` zero-fills the members it does not list.
                ..core::mem::zeroed()
            };
            let bOK: BOOL = ShellExecuteExW(core::ptr::addr_of_mut!(sei));
            if !pidl.is_null() {
                CoTaskMemFree(pidl as _);
            }
            return if bOK != 0 { S_OK } else { E_FAIL };
        }

        if idCmd == (FCIDM_SHVIEW_INSERT as INT) || (idCmd == -1 && lstrcmpiA((*lpcmi).lpVerb, b"paste\0".as_ptr() as *const i8) == 0)
        {
            let mut pDataObj: CComPtr<IDataObject> = CComPtr::new();
            let hr: HRESULT = OleGetClipboard(pDataObj.GetAddressOf());
            if FAILED_UNEXPECTEDLY!(hr) || CheckDataObject(pDataObj.Get()) == 0
            {
                // Show error message
                let text: CStringW = CStringW::from_ptr(MAKEINTRESOURCEW!(IDS_INSTALL_FAILED));
                let title: CStringW = CStringW::from_ptr(MAKEINTRESOURCEW!(IDS_REACTOS_FONTS_FOLDER));
                MessageBoxW((*this_).m_hwnd, text.GetString(), title.GetString(), MB_ICONERROR);
                return E_FAIL;
            }

            return SHSimulateDrop(core::ptr::addr_of_mut!((*(*this_).m_pFontExt).__base4), pDataObj.Get(), 0, core::ptr::null_mut(), core::ptr::null_mut());
        }

        return S_OK;
    }
    pub unsafe fn GetCommandString(&mut self, idCommand: UINT_PTR, uFlags: UINT, lpReserved: *mut UINT, lpszName: LPSTR, uMaxNameLen: UINT) -> HRESULT {
        let this_: *mut Self = self;
        if idCommand == 0 {
            lstrcpynA(lpszName, b"properties\0".as_ptr() as *const i8, uMaxNameLen as i32);
            return S_OK;
        }
        return E_FAIL;
    }
    pub unsafe fn HandleMenuMsg(&mut self, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
        let this_: *mut Self = self;
        return (*this_).HandleMenuMsg2(uMsg, wParam, lParam, core::ptr::null_mut());
    }
    pub unsafe fn HandleMenuMsg2(&mut self, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, plResult: *mut LRESULT) -> HRESULT {
        let this_: *mut Self = self;
        if uMsg == WM_INITMENUPOPUP {
            let hMenu: HMENU = wParam as HMENU;
            DeleteMenu(hMenu, FCIDM_SHVIEW_INSERTLINK, MF_BYCOMMAND);
            return S_OK;
        }
        return E_NOTIMPL;
    }
}
