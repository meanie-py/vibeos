use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_BA5068D6298D: [WCHAR; 15] = [0x007A, 0x0069, 0x0070, 0x0066, 0x006C, 0x0064, 0x0072, 0x002E, 0x0064, 0x006C, 0x006C, 0x002C, 0x002D, 0x0031, 0x0000];

use core::ffi::c_void;






















#[no_mangle]
pub unsafe extern "C" fn _CExplorerCommandProvider_CreateInstance(zipObject: *mut IContextMenu, riid: &IID, ppvOut: *mut LPVOID) -> HRESULT {
    return ShellObjectCreatorInit!(CExplorerCommandProvider, [zipObject], riid, ppvOut);
}

impl CExplorerCommand {
    pub unsafe fn Initialize(&mut self, zipObject: *mut IContextMenu) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_pZipObject.assign(zipObject);
        return S_OK;
    }
    pub unsafe fn GetTitle(&mut self, psiItemArray: *mut IShellItemArray, ppszName: *mut PWSTR) -> HRESULT {
        let this_: *mut Self = self;
            let mut Title = CStringW::from_resource(IDS_MENUITEM);
            return SHStrDup(Title.GetString(), ppszName);
        }
    pub unsafe fn GetIcon(&mut self, psiItemArray: *mut IShellItemArray, ppszIcon: *mut PWSTR) -> HRESULT {
        let this_: *mut Self = self;
            let mut IconName = CStringW::from_ptr(CPP_COM_WSTR_BA5068D6298D.as_ptr());
            return SHStrDup(IconName.GetString(), ppszIcon);
        }
    pub unsafe fn GetToolTip(&mut self, psiItemArray: *mut IShellItemArray, ppszInfotip: *mut PWSTR) -> HRESULT {
        let this_: *mut Self = self;
        let mut HelpText = CStringW::from_resource(IDS_HELPTEXT);
        return SHStrDup(HelpText.GetString(), ppszInfotip);
    }
    pub unsafe fn GetCanonicalName(&mut self, pguidCommandName: *mut GUID) -> HRESULT {
        let this_: *mut Self = self;
        *pguidCommandName = CLSID_ZipFolderExtractAllCommand;
        return S_OK;
    }
    pub unsafe fn GetState(&mut self, psiItemArray: *mut IShellItemArray, fOkToBeSlow: BOOL, pCmdState: *mut EXPCMDSTATE) -> HRESULT {
        let this_: *mut Self = self;
        *pCmdState = ECS_ENABLED;
        return S_OK;
    }
    pub unsafe fn Invoke(&mut self, psiItemArray: *mut IShellItemArray, pbc: *mut IBindCtx) -> HRESULT {
        let this_: *mut Self = self;
        let mut cm: CMINVOKECOMMANDINFO = core::mem::zeroed();
        cm.cbSize = core::mem::size_of::<CMINVOKECOMMANDINFO>() as DWORD;
        cm.fMask = 0;
        cm.lpVerb = EXTRACT_VERBA.as_ptr() as LPCSTR;
        cm.nShow = (SW_SHOW as i32);
        return (*(*this_).m_pZipObject.Get()).InvokeCommand(core::ptr::addr_of_mut!(cm));
    }
    pub unsafe fn GetFlags(&mut self, pFlags: *mut EXPCMDFLAGS) -> HRESULT {
        let this_: *mut Self = self;
        *pFlags = ECF_DEFAULT;
        return S_OK;
    }
    pub unsafe fn EnumSubCommands(&mut self, ppEnum: *mut *mut IEnumExplorerCommand,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // DbgPrint("%s\n", "EnumSubCommands");
        E_NOTIMPL
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
    }
}

impl CEnumExplorerCommand {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_bFirst = true;
        {
        }
    }
    pub unsafe fn Initialize(&mut self, zipObject: *mut IContextMenu) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_pZipObject.assign(zipObject);
        return S_OK;
    }
    pub unsafe fn Next(&mut self, mut celt: ULONG, pUICommand: *mut *mut IExplorerCommand, pceltFetched: *mut ULONG) -> HRESULT {
        let this_: *mut Self = self;
        if pUICommand.is_null() {
            return E_POINTER;
        }

        if !pceltFetched.is_null() {
            *pceltFetched = 0;
        }
        if (*this_).m_bFirst && celt != 0 {
            (*this_).m_bFirst = false;
            celt -= 1;
            let hr: HRESULT = ShellObjectCreatorInit!(CExplorerCommand, [(*this_).m_pZipObject.Get()], &IID_IExplorerCommand, pUICommand as *mut *mut core::ffi::c_void);
            if hr >= 0 {
                if !pceltFetched.is_null() {
                    *pceltFetched = 1;
                }
            }
            return hr;
        }
        return S_FALSE;
    }
    pub unsafe fn Skip(&mut self, celt: ULONG) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_bFirst {
            (*this_).m_bFirst = false;
            return S_OK;
        }
        return S_FALSE;
    }
    pub unsafe fn Reset(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_bFirst = true;
        return S_OK;
    }
    pub unsafe fn Clone(&mut self, ppenum: *mut *mut IEnumExplorerCommand,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
}

impl CExplorerCommandProvider {
    pub unsafe fn Initialize(&mut self, zipObject: *mut IContextMenu) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_pZipObject.assign(zipObject);
        return S_OK;
    }
    pub unsafe fn GetCommands(&mut self, punkSite: *mut IUnknown, riid: &IID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
        let this_: *mut Self = self;
        return ShellObjectCreatorInit!(CEnumExplorerCommand, [(*this_).m_pZipObject.Get()], riid, ppv);
    }
    pub unsafe fn GetCommand(&mut self, rguidCommandId: &GUID, riid: &IID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
        let this_: *mut Self = self;
        UNIMPLEMENTED!();
        *ppv = core::ptr::null_mut();
        return E_NOTIMPL;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
    }
}
