use crate::prelude::*;
use core::ffi::c_void;

























unsafe extern "C" fn IsSingleBitSet(dwValue: DWORD) -> BOOL {
    return ((dwValue != 0) && ((dwValue & (dwValue - 1)) == 0)) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn LangBarMgr_Constructor(pUnkOuter: *mut IUnknown, ppOut: *mut *mut IUnknown) -> HRESULT {
    if !pUnkOuter.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    let pLangBarMgr: *mut CLangBarMgr = CLangBarMgr__new_in_0(cpp_alloc_CicNoThrow::<CLangBarMgr>());
    if pLangBarMgr.is_null() {
        return E_OUTOFMEMORY;
    }

    let hr: HRESULT = (*pLangBarMgr).QueryInterface(&IID_ITfLangBarMgr, ppOut as *mut PVOID);

    (*pLangBarMgr).Release();
    return hr;
}

impl CLangBarMgr {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_cRefs = 1;
    }
    pub unsafe fn QueryInterface(&mut self, riid: &IID, ppvObj: *mut PVOID) -> HRESULT {
        let this_: *mut Self = self;
        if ppvObj.is_null() {
            return E_INVALIDARG;
        }
    
        *ppvObj = core::ptr::null_mut();
        if IsEqualGUID!(&(*riid), &(IID_IUnknown)) || IsEqualGUID!(riid, &IID_ITfLangBarMgr) || IsEqualGUID!(&(*riid), &(IID_ITfLangBarMgr_P)) {
            *ppvObj = core::ptr::addr_of_mut!((*this_).__base0) as PVOID;
        }
    
        if !(*ppvObj).is_null() {
            (*this_).AddRef();
            return S_OK;
        }
    
        return E_NOINTERFACE;
    }
    pub unsafe fn AddRef(&mut self) -> ULONG {
        let this_: *mut Self = self;
        return (InterlockedIncrement(core::ptr::addr_of_mut!((*this_).m_cRefs))) as ULONG;
    }
    pub unsafe fn Release(&mut self) -> ULONG {
        let this_: *mut Self = self;
        let mut ret: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) as ULONG;
        if ret == 0 {
            CLangBarMgr__delete(this_);
        }
        return ret;
    }
    pub unsafe fn AdviseEventSink(&mut self, pSink: *mut ITfLangBarEventSink, hwnd: HWND, dwflags: DWORD, pdwCookie: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if pSink.is_null() {
            return E_INVALIDARG;
        }
        return E_NOTIMPL;
    }
    pub unsafe fn UnAdviseEventSink(&mut self, dwCookie: DWORD,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("(0x%lX)\n", dwCookie);
        E_NOTIMPL
    }
    pub unsafe fn GetThreadMarshalInterface(&mut self, dwThreadId: DWORD, dwType: DWORD, riid: &IID, ppunk: *mut *mut IUnknown) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn GetThreadLangBarItemMgr(&mut self, dwThreadId: DWORD,
        pplbie: *mut *mut ITfLangBarItemMgr,
        pdwThreadid: *mut DWORD,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("(%lu, %p, %p)\n", dwThreadId, pplbie, pdwThreadid);
        E_NOTIMPL
    }
    pub unsafe fn GetInputProcessorProfiles(&mut self, dwThreadId: DWORD,
        ppaip: *mut *mut ITfInputProcessorProfiles,
        pdwThreadid: *mut DWORD,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("(%lu, %p, %p)\n", dwThreadId, ppaip, pdwThreadid);
        E_NOTIMPL
    }
    pub unsafe fn RestoreLastFocus(&mut self, dwThreadId: *mut DWORD,
        fPrev: BOOL,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("(%p, %d)\n", dwThreadId, fPrev);
        E_NOTIMPL
    }
    pub unsafe fn SetModalInput(&mut self, pSink: *mut ITfLangBarEventSink,
        dwThreadId: DWORD,
        dwFlags: DWORD,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("(%p, %lu, 0x%lX)\n", pSink, dwThreadId, dwFlags);
        E_NOTIMPL
    }
    pub unsafe fn ShowFloating(&mut self, dwFlags: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if CLangBarMgr::CheckFloatingBits(dwFlags) == 0 {
            return E_INVALIDARG;
        }
        return CLangBarMgr::s_ShowFloating(dwFlags);
    }
    pub unsafe fn GetShowFloatingStatus(&mut self, pdwFlags: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if pdwFlags.is_null() {
            return E_INVALIDARG;
        }
    
        return CLangBarMgr::s_GetShowFloatingStatus(pdwFlags);
    }
    pub unsafe fn GetPrevShowFloatingStatus(&mut self, pdwStatus: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if pdwStatus.is_null() {
            return E_INVALIDARG;
        }
        return E_NOTIMPL;
    }
    pub unsafe fn CheckFloatingBits(dwBits: DWORD) -> BOOL {
        return ((IsSingleBitSet(dwBits & (TF_SFT_VISIBILITY_GROUP as u32)) != 0)
            && (IsSingleBitSet(dwBits & (TF_SFT_TRANSPARENCY_GROUP as u32)) != 0)
            && (IsSingleBitSet(dwBits & (TF_SFT_LABEL_GROUP as u32)) != 0)
            && (IsSingleBitSet(dwBits & (TF_SFT_EXTRA_ICON_GROUP as u32)) != 0)) as BOOL;
    }
    pub unsafe fn s_GetShowFloatingStatus(pdwFlags: *mut DWORD) -> HRESULT {
        E_NOTIMPL
    }
    pub unsafe fn s_ShowFloating(dwFlags: DWORD) -> HRESULT {
        E_NOTIMPL
    }
}
