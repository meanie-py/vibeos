use crate::prelude::*;
use core::ffi::c_void;




















#[no_mangle]
pub unsafe extern "C" fn DisplayAttributeMgr_Constructor(pUnkOuter: *mut IUnknown, ppOut: *mut *mut IUnknown) -> HRESULT {
    if !pUnkOuter.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    let This: *mut CDisplayAttributeMgr = CDisplayAttributeMgr__new_in_0(cpp_alloc_CicNoThrow::<CDisplayAttributeMgr>());
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    let hr: HRESULT = (*This).QueryInterface(&IID_ITfDisplayAttributeMgr, ppOut as *mut *mut core::ffi::c_void);
    (*This).Release();
    return hr;
}

impl CDisplayAttributeMgr {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_cRefs = 1;
        {
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        // TRACE("destroying %p\n", this_);
    }
    pub unsafe fn QueryInterface(&mut self, riid: &IID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT {
        let this_: *mut Self = self;
        if ppvObj.is_null() {
            return E_INVALIDARG;
        }
    
        *ppvObj = core::ptr::null_mut();
    
        if IsEqualGUID!(&(*riid), &(IID_IUnknown)) ||
            IsEqualGUID!(&(*riid), &(IID_ITfDisplayAttributeMgr)) ||
            IsEqualGUID!(&(*riid), &(IID_CDisplayAttributeMgr))
        {
            *ppvObj = this_ as *mut core::ffi::c_void;
        }
        else if IsEqualGUID!(&(*riid), &(IID_ITfDisplayAttributeCollectionMgr))
        {
            *ppvObj = core::ptr::addr_of_mut!((*this_).__base1) as *mut core::ffi::c_void;
        }
    
        if (*ppvObj).is_null() {
            return E_NOINTERFACE;
        }
    
        (*this_).AddRef();
        return S_OK;
    }
    pub unsafe fn AddRef(&mut self) -> ULONG {
        let this_: *mut Self = self;
        return (InterlockedIncrement(core::ptr::addr_of_mut!((*this_).m_cRefs))) as ULONG;
    }
    pub unsafe fn Release(&mut self) -> ULONG {
        let this_: *mut Self = self;
        let ret: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) as ULONG;
        if ret == 0 {
            CDisplayAttributeMgr__delete(this_);
        }
        return ret;
    }
    pub unsafe fn OnUpdateInfo(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("()\n");
        E_NOTIMPL
    }
    pub unsafe fn EnumDisplayAttributeInfo(&mut self, ppEnum: *mut *mut IEnumTfDisplayAttributeInfo,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("(%p)\n", ppEnum);
        E_NOTIMPL
    }
    pub unsafe fn GetDisplayAttributeInfo(&mut self, guid: &GUID, ppInfo: *mut *mut ITfDisplayAttributeInfo, pclsidOwner: *mut CLSID) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn UnknownMethod(&mut self, unused: DWORD,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("(0x%lX)\n", unused);
        E_NOTIMPL
    }
    pub unsafe fn _IsInCollection(&mut self, rguid: &GUID) -> BOOL {
        let this_: *mut Self = self;
        return (FALSE) as BOOL;
    }
    pub unsafe fn _AdviseMarkupCollection(&mut self, pProcessor: *mut ITfTextInputProcessor,
        dwCookie: DWORD,
    ) {
        let this_: *mut Self = self;
        // FIXME("(%p, %u)\n", pProcessor, dwCookie);
    }
    pub unsafe fn _UnadviseMarkupCollection(&mut self, dwCookie: DWORD,
    ) {
        let this_: *mut Self = self;
        // FIXME("(%u)\n", dwCookie);
    }
    pub unsafe fn _SetThis(&mut self) {
        let this_: *mut Self = self;
        // FIXME("()\n");
    }
}
