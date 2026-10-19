use crate::prelude::*;
use crate::zipfldr::zipfldr_cpp_h::g_ModuleRefCnt;
use core::ffi::c_void;














impl CSendToZip {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_fCanDragDrop = (FALSE as i32);
        {
            InterlockedIncrement(core::ptr::addr_of_mut!(g_ModuleRefCnt));
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        InterlockedDecrement(core::ptr::addr_of_mut!(g_ModuleRefCnt));
    }
    pub unsafe fn IsDirty(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        S_FALSE
    }
    pub unsafe fn Load(&mut self, pszFileName: LPCOLESTR,
        dwMode: DWORD,
    ) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn Save(&mut self, pszFileName: LPCOLESTR,
        fRemember: BOOL,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn SaveCompleted(&mut self, pszFileName: LPCOLESTR,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn GetCurFile(&mut self, ppszFileName: *mut LPOLESTR,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn GetClassID(&mut self, pclsid: *mut CLSID) -> HRESULT {
        let this_: *mut Self = self;
        *pclsid = CLSID_ZipFolderSendTo;
        return S_OK;
    }
}
