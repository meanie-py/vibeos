use crate::prelude::*;
use core::ffi::c_void;

impl EXTRACTCALLBACKDATA {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl CEnumIDList {
    pub unsafe extern "system" fn DPADestroyCallback(pidl: *mut core::ffi::c_void, pData: *mut core::ffi::c_void) -> i32 {
            SHFree(pidl);
            return (TRUE as i32);
        }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_Pos = 0;
        {
            (*this_).m_Items = DPA_Create(0);
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        DPA_DestroyCallback((*this_).m_Items, Some(CEnumIDList::DPADestroyCallback), core::ptr::null_mut());
    }
    pub unsafe fn Append(&mut self, pidl: LPCITEMIDLIST) -> HRESULT {
        let this_: *mut Self = self;
        return if DPA_AppendPtr!((*this_).m_Items, pidl as *mut core::ffi::c_void) != DPA_ERR { S_OK } else { E_OUTOFMEMORY };
    }
    pub unsafe fn GetCount(&mut self) -> UINT {
        let this_: *mut Self = self;
        return if !(*this_).m_Items.is_null() { DPA_GetPtrCount!((*this_).m_Items) as UINT } else { 0 };
    }
    pub unsafe fn Next(&mut self, celt: ULONG, rgelt: *mut PITEMID_CHILD, pceltFetched: *mut ULONG) -> HRESULT {
        let this_: *mut Self = self;
        if rgelt.is_null() {
            return E_INVALIDARG;
        }
        let mut hr: HRESULT = S_FALSE;
        let mut count: UINT = (*this_).GetCount();
        let mut fetched: UINT = 0;
        if (*this_).m_Pos < count && fetched < celt {
            hr = SHILClone(DPA_FastGetPtr!((*this_).m_Items, (*this_).m_Pos) as LPITEMIDLIST, rgelt.add(fetched as usize));
            if hr >= 0 {
                fetched += 1;
            }
        }
        if !pceltFetched.is_null() {
            *pceltFetched = fetched;
        }
        (*this_).m_Pos += fetched;
        return if (hr) < 0 {
            hr
        } else if celt == fetched && fetched != 0 {
            S_OK
        } else {
            S_FALSE
        };
    }
    pub unsafe fn Reset(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_Pos = 0;
        return S_OK;
    }
    pub unsafe fn Skip(&mut self, celt: ULONG) -> HRESULT {
        let this_: *mut Self = self;
        let count: UINT = (*this_).GetCount();
        let newpos: UINT = (*this_).m_Pos + celt;
        if celt > count || newpos >= count {
            return E_INVALIDARG;
        }
        (*this_).m_Pos = newpos;
        return S_OK;
    }
    pub unsafe fn Clone(&mut self, ppenum: *mut *mut IEnumIDList) -> HRESULT {
        let this_: *mut Self = self;
        UNIMPLEMENTED!();
        *ppenum = core::ptr::null_mut();
        return E_NOTIMPL;
    }
    pub unsafe fn CreateInstance() -> *mut CEnumIDList {
        let mut obj: CComPtr<CEnumIDList> = CComPtr::new();
        return if (ShellObjectCreator(&mut obj) >= 0) {
            obj.Detach()
        } else {
            core::ptr::null_mut()
        };
    }
}

impl CCabFolder {
    pub unsafe fn CreateEnum(&mut self, List: *mut *mut CEnumIDList) -> HRESULT {
        let this_: *mut Self = self;
        let pEIDL: *mut CEnumIDList = CEnumIDList::CreateInstance();
        *List = pEIDL;
        return if !pEIDL.is_null() { (*pEIDL).Fill((*this_).m_CurDir.m_pData, core::ptr::null_mut(), 0) } else { E_OUTOFMEMORY };
    }
    pub unsafe fn GetDefaultSearchGUID(&mut self, pguid: *mut GUID,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn EnumSearches(&mut self, ppenum: *mut *mut IEnumExtraSearch,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn BindToStorage(&mut self, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: &IID, ppvOut: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        UNIMPLEMENTED!();
        return E_NOTIMPL;
    }
    pub unsafe fn SetNameOf(&mut self, hwndOwner: HWND,
        pidl: PCUITEMID_CHILD,
        lpName: LPCOLESTR,
        dwFlags: DWORD,
        pPidlOut: *mut PITEMID_CHILD,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn GetCurFolder(&mut self, pidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
        let this_: *mut Self = self;
        let curdir: LPITEMIDLIST = (*this_).m_CurDir.m_pData;
        return if !curdir.is_null() {
            SHILClone(curdir, pidl)
        } else {
            E_UNEXPECTED
        };
    }
    pub unsafe fn Initialize(&mut self, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
        let this_: *mut Self = self;
        let mut path: [u16; (MAX_PATH) as _] = [0u16; (MAX_PATH) as _];
        if SHGetPathFromIDListW(pidl, path.as_mut_ptr()) != 0 {
            let curdir: PIDLIST_ABSOLUTE = ILClone(pidl);
            if !curdir.is_null() {
                (*this_).m_CurDir.Attach(curdir);
                return S_OK;
            }
            return E_OUTOFMEMORY;
        }
        return E_INVALIDARG;
    }
    pub unsafe fn GetClassID(&mut self, lpClassId: *mut CLSID) -> HRESULT {
        let this_: *mut Self = self;
        *lpClassId = CLSID_CabFolder;
        return S_OK;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_ShellViewWindow = core::ptr::null_mut();
    }
}

extern "system" {
    /// commctrl.h's `DPA_DestroyCallback` (comctl32): outside the island, which allowlists
    /// only the SDK headers the module's own code names.
    fn DPA_DestroyCallback(
        hdpa: HDPA,
        pfnCB: Option<unsafe extern "system" fn(p: *mut c_void, pData: *mut c_void) -> i32>,
        pData: *mut c_void,
    );
}
