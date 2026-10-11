use crate::prelude::*;
use core::ffi::c_void;

impl CCompartmentEventSink {
    pub unsafe fn __ctor_0(&mut self, fnEventSink: FN_EVENTSINK, pUserData: LPVOID) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        // C source has `this_->m_array = ;` with no right-hand side; zero-initialise the member in place.
        core::ptr::write_bytes(core::ptr::addr_of_mut!((*this_).m_array), 0, 1);
        (*this_).m_cRefs = 1;
        (*this_).m_fnEventSink = fnEventSink;
        (*this_).m_pUserData = pUserData;
    }
    pub unsafe fn _Advise(&mut self, pUnknown: *mut IUnknown, rguid: &GUID, bThread: BOOL) -> HRESULT {
        let this_: *mut Self = self;
        let pCesMap: *mut CESMAP = (*this_).m_array.Append(1);
        if pCesMap.is_null() {
            return E_OUTOFMEMORY;
        }
    
        let mut pSource: *mut ITfSource = core::ptr::null_mut();
    
        let mut hr: HRESULT = GetCompartment(pUnknown, rguid, core::ptr::addr_of_mut!((*pCesMap).m_pComp), bThread);
        if hr < 0 {
            hr = (*(*pCesMap).m_pComp).QueryInterface(&IID_ITfSource, core::ptr::addr_of_mut!(pSource) as *mut LPVOID);
            if hr < 0 {
                hr = (*pSource).AdviseSink(&IID_ITfCompartmentEventSink, (this_).cast(), core::ptr::addr_of_mut!((*pCesMap).m_dwCookie));
                if hr < 0 {
                    if !(*pCesMap).m_pComp.is_null() {
                        (*(*pCesMap).m_pComp).Release();
                        (*pCesMap).m_pComp = core::ptr::null_mut();
                    }
                    (*this_).m_array.Remove((*this_).m_array.size() - 1, 1);
                } else {
                    hr = S_OK;
                }
            }
        }
    
        if !pSource.is_null() {
            (*pSource).Release();
        }
    
        return hr;
    }
    pub unsafe fn _Unadvise(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        let mut pCesMap: *mut CESMAP = (*this_).m_array.data();
        let mut cItems: usize = (*this_).m_array.size();
        if cItems == 0 {
            return S_OK;
        }
    
        loop {
            let mut pSource: *mut ITfSource = core::ptr::null_mut();
            let mut hr: HRESULT = (*(*pCesMap).m_pComp).QueryInterface(
                &IID_ITfSource,
                core::ptr::addr_of_mut!(pSource) as *mut LPVOID,
            );
            if hr >= 0 {
                (*pSource).UnadviseSink((*pCesMap).m_dwCookie);
            }
    
            if !(*pCesMap).m_pComp.is_null() {
                (*(*pCesMap).m_pComp).Release();
                (*pCesMap).m_pComp = core::ptr::null_mut();
            }
    
            if !pSource.is_null() {
                (*pSource).Release();
            }
    
            pCesMap = pCesMap.add(1);
            cItems -= 1;
    
            if !(cItems != 0) {
                break;
            }
        }
    
        return S_OK;
    }
    pub unsafe fn QueryInterface(&mut self, riid: &IID, ppvObj: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        let c_tab: [QITAB; 2] = [
            QITAB {
                piid: &raw const IID_ITfCompartmentEventSink,
                dwOffset: core::mem::offset_of!(CCompartmentEventSink, __base0) as DWORD,
            },
            QITAB {
                piid: core::ptr::null(),
                dwOffset: 0,
            },
        ];
        return QISearch(this_ as LPVOID, c_tab.as_ptr(), riid, ppvObj);
    }
    pub unsafe fn AddRef(&mut self) -> ULONG {
        let this_: *mut Self = self;
        return (InterlockedIncrement(core::ptr::addr_of_mut!((*this_).m_cRefs))) as ULONG;
    }
    pub unsafe fn Release(&mut self) -> ULONG {
        let this_: *mut Self = self;
        if InterlockedDecrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) == 0 {
            CCompartmentEventSink__delete(this_);
            return 0;
        }
        return ((*this_).m_cRefs) as ULONG;
    }
    pub unsafe fn OnChange(&mut self, rguid: &GUID) -> HRESULT {
        let this_: *mut Self = self;
        return (((*this_).m_fnEventSink).unwrap_unchecked())((*this_).m_pUserData, rguid);
    }
}

impl CTextEventSink {
    pub unsafe fn __ctor_0(&mut self, fnEndEdit: FN_ENDEDIT, pCallbackPV: LPVOID) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {
        (*this_).m_cRefs = 1;
        (*this_).m_pUnknown = core::ptr::null_mut();
        (*this_).m_dwEditSinkCookie = (0u32).wrapping_sub(1 as DWORD);
        (*this_).m_dwLayoutSinkCookie = (0u32).wrapping_sub(1 as DWORD);
        (*this_).__anon0.m_fnLayoutChange = None;
        (*this_).m_fnEndEdit = fnEndEdit;
        (*this_).m_pCallbackPV = pCallbackPV;
    }
    }
    pub unsafe fn _Advise(&mut self, pUnknown: *mut IUnknown, uFlags: UINT) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_pUnknown = core::ptr::null_mut();
        (*this_).__anon0.m_uFlags = uFlags;
    
        let mut pSource: *mut ITfSource = core::ptr::null_mut();
        let mut hr: HRESULT = (*pUnknown).QueryInterface(&IID_ITfSource, &mut pSource as *mut *mut ITfSource as *mut LPVOID);
        if hr >= 0
        {
            let pSink: *mut ITfTextEditSink = &mut (*this_).__base0 as *mut ITfTextEditSink;
            if (uFlags & 1) != 0 {
                hr = (*pSource).AdviseSink(&IID_ITfTextEditSink, pSink as *mut IUnknown, &mut (*this_).m_dwEditSinkCookie as *mut DWORD);
            }
            if (hr >= 0) && ((uFlags & 2) != 0) {
                hr = (*pSource).AdviseSink(&IID_ITfTextLayoutSink, pSink as *mut IUnknown, &mut (*this_).m_dwLayoutSinkCookie as *mut DWORD);
            }
    
            if hr >= 0
            {
                (*this_).m_pUnknown = pUnknown;
                (*pUnknown).AddRef();
            }
            else
            {
                (*pSource).UnadviseSink((*this_).m_dwEditSinkCookie);
            }
        }
    
        if !pSource.is_null() {
            (*pSource).Release();
        }
    
        hr
    }
    pub unsafe fn _Unadvise(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_pUnknown.is_null() {
            return E_FAIL;
        }
    
        let mut pSource: *mut ITfSource = core::ptr::null_mut();
        let mut hr: HRESULT = (*(*this_).m_pUnknown).QueryInterface(&IID_ITfSource, &mut pSource as *mut *mut ITfSource as *mut LPVOID);
        if ((hr) >= 0) {
            if ((*this_).__anon0.m_uFlags & 1) != 0 {
                hr = (*pSource).UnadviseSink((*this_).m_dwEditSinkCookie);
            }
            if ((*this_).__anon0.m_uFlags & 2) != 0 {
                hr = (*pSource).UnadviseSink((*this_).m_dwLayoutSinkCookie);
            }
    
            (*pSource).Release();
        }
    
        (*(*this_).m_pUnknown).Release();
        (*this_).m_pUnknown = core::ptr::null_mut();
    
        return E_NOTIMPL;
    }
    pub unsafe fn QueryInterface(&mut self, riid: &IID, ppvObj: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        static mut c_tab: [QITAB; 3] = [
            QITAB {
                piid: &raw const IID_ITfTextEditSink,
                dwOffset: core::mem::offset_of!(CTextEventSink, __base0) as DWORD,
            },
            QITAB {
                piid: &raw const IID_ITfTextLayoutSink,
                dwOffset: core::mem::offset_of!(CTextEventSink, __base1) as DWORD,
            },
            QITAB {
                piid: core::ptr::null(),
                dwOffset: 0,
            },
        ];
        return QISearch((this_).cast(), c_tab.as_ptr(), riid, ppvObj);
    }
    pub unsafe fn AddRef(&mut self) -> ULONG {
        let this_: *mut Self = self;
        return (InterlockedIncrement(core::ptr::addr_of_mut!((*this_).m_cRefs))) as ULONG;
    }
    pub unsafe fn Release(&mut self) -> ULONG {
        let this_: *mut Self = self;
        if InterlockedDecrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) == 0 {
            CTextEventSink__delete(this_);
            return 0;
        }
        return (*this_).m_cRefs as ULONG;
    }
    pub unsafe fn OnEndEdit(&mut self, pic: *mut ITfContext, ecReadOnly: TfEditCookie, pEditRecord: *mut ITfEditRecord) -> HRESULT {
        let this_: *mut Self = self;
        let mut Data = TEXT_EVENT_SINK_END_EDIT { m_ecReadOnly: ecReadOnly, m_pEditRecord: pEditRecord, m_pContext: pic };
        return (((*this_).m_fnEndEdit).unwrap_unchecked())(1, (*this_).m_pCallbackPV, core::ptr::addr_of_mut!(Data) as LPVOID);
    }
    pub unsafe fn OnLayoutChange(&mut self, pContext: *mut ITfContext, lcode: TfLayoutCode, pContextView: *mut ITfContextView) -> HRESULT {
        let this_: *mut Self = self;
        match lcode {
            TF_LC_CREATE => {
                return (((*this_).__anon0.m_fnLayoutChange).unwrap_unchecked())(3, (*this_).m_fnEndEdit, pContextView);
            }
            TF_LC_CHANGE => {
                return (((*this_).__anon0.m_fnLayoutChange).unwrap_unchecked())(2, (*this_).m_fnEndEdit, pContextView);
            }
            TF_LC_DESTROY => {
                return (((*this_).__anon0.m_fnLayoutChange).unwrap_unchecked())(4, (*this_).m_fnEndEdit, pContextView);
            }
            _ => {
                return E_INVALIDARG;
            }
        }
    }
}

impl CThreadMgrEventSink {
    pub unsafe fn __ctor_0(&mut self, fnInit: FN_INITDOCMGR, fnPushPop: FN_PUSHPOP, pvCallbackPV: LPVOID) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {
            (*this_).m_fnInit = fnInit;
            (*this_).m_fnPushPop = fnPushPop;
            (*this_).m_pCallbackPV = pvCallbackPV;
            (*this_).m_cRefs = 1;
        }
    }
    pub unsafe fn SetCallbackPV(&mut self, pv: LPVOID) {
        let this_: *mut Self = self;
        if (*this_).m_pCallbackPV.is_null() {
            (*this_).m_pCallbackPV = pv;
        }
    }
    pub unsafe fn _Advise(&mut self, pThreadMgr: *mut ITfThreadMgr) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_pThreadMgr = core::ptr::null_mut();
    
        let mut hr: HRESULT = E_FAIL;
        let mut pSource: *mut ITfSource = core::ptr::null_mut();
        if (*pThreadMgr).QueryInterface(&IID_ITfSource, core::ptr::addr_of_mut!(pSource) as *mut LPVOID) == S_OK
            && (*pSource).AdviseSink(&IID_ITfThreadMgrEventSink, (this_ as LPVOID).cast(), core::ptr::addr_of_mut!((*this_).m_dwCookie)) == S_OK
        {
            (*this_).m_pThreadMgr = pThreadMgr;
            (*pThreadMgr).AddRef();
            hr = S_OK;
        }
    
        if !pSource.is_null() {
            (*pSource).Release();
        }
    
        hr
    }
    pub unsafe fn _Unadvise(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT = E_FAIL;
        let mut pSource: *mut ITfSource = core::ptr::null_mut();
    
        if !(*this_).m_pThreadMgr.is_null() {
            if (*(*this_).m_pThreadMgr).QueryInterface(&IID_ITfSource, &mut pSource as *mut *mut ITfSource as *mut LPVOID) == S_OK
                && (*pSource).UnadviseSink((*this_).m_dwCookie) == S_OK
            {
                hr = S_OK;
            }
    
            if !pSource.is_null() {
                (*pSource).Release();
            }
        }
    
        if !(*this_).m_pThreadMgr.is_null() {
            (*(*this_).m_pThreadMgr).Release();
            (*this_).m_pThreadMgr = core::ptr::null_mut();
        }
    
        hr
    }
    pub unsafe fn QueryInterface(&mut self, riid: &IID, ppvObj: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        #[allow(non_upper_case_globals)]
        static mut c_tab: [QITAB; 2] = [
            QITAB {
                piid: &raw const IID_ITfThreadMgrEventSink,
                dwOffset: core::mem::offset_of!(CThreadMgrEventSink, __base0) as DWORD,
            },
            QITAB {
                piid: core::ptr::null(),
                dwOffset: 0,
            },
        ];
        return QISearch((this_).cast(), c_tab.as_ptr(), riid, ppvObj);
    }
    pub unsafe fn AddRef(&mut self) -> ULONG {
        let this_: *mut Self = self;
        return InterlockedIncrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) as ULONG;
    }
    pub unsafe fn Release(&mut self) -> ULONG {
        let this_: *mut Self = self;
        if InterlockedDecrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) == 0 {
            CThreadMgrEventSink__delete(this_);
            return 0;
        }
        return (*this_).m_cRefs as ULONG;
    }
    pub unsafe fn OnInitDocumentMgr(&mut self, pdim: *mut ITfDocumentMgr) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_fnInit.is_none() {
            return S_OK;
        }
        return (*this_).m_fnInit.unwrap_unchecked()(0, pdim, core::ptr::null_mut(), (*this_).m_pCallbackPV);
    }
    pub unsafe fn OnUninitDocumentMgr(&mut self, pdim: *mut ITfDocumentMgr) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_fnInit.is_none() {
            return S_OK;
        }
        return (*this_).m_fnInit.unwrap_unchecked()(1, pdim, core::ptr::null_mut(), (*this_).m_pCallbackPV);
    }
    pub unsafe fn OnSetFocus(&mut self, pdimFocus: *mut ITfDocumentMgr, pdimPrevFocus: *mut ITfDocumentMgr) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_fnInit.is_none() {
            return S_OK;
        }
        return (*this_).m_fnInit.unwrap_unchecked()(2, pdimFocus, pdimPrevFocus, (*this_).m_pCallbackPV);
    }
    pub unsafe fn OnPushContext(&mut self, pic: *mut ITfContext) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_fnPushPop.is_none() {
            return S_OK;
        }
        return (((*this_).m_fnPushPop).unwrap_unchecked())(3, pic, (*this_).m_pCallbackPV);
    }
    pub unsafe fn OnPopContext(&mut self, pic: *mut ITfContext) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_fnPushPop.is_none() {
            return S_OK;
        }
        return (((*this_).m_fnPushPop).unwrap_unchecked())(4, pic, (*this_).m_pCallbackPV);
    }
    pub unsafe extern "system" fn DIMCallback(nCode: UINT,
        pDocMgr1: *mut ITfDocumentMgr,
        pDocMgr2: *mut ITfDocumentMgr,
        pUserData: LPVOID,
    ) -> INT {
        E_NOTIMPL
    }
}

impl CActiveLanguageProfileNotifySink {
    pub unsafe fn __ctor_0(&mut self, fnCompare: FN_COMPARE, pUserData: *mut core::ffi::c_void) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {
            (*this_).m_dwConnection = (1 as DWORD).wrapping_neg();
            (*this_).m_fnCompare = fnCompare;
            (*this_).m_cRefs = 1;
            (*this_).m_pUserData = pUserData;
        }
    }
    pub unsafe fn _Advise(&mut self, pThreadMgr: *mut ITfThreadMgr) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_pThreadMgr = core::ptr::null_mut();
    
        let mut pSource: *mut ITfSource = core::ptr::null_mut();
        let mut hr: HRESULT = (*pThreadMgr).QueryInterface(&IID_ITfSource, &mut pSource as *mut *mut ITfSource as *mut LPVOID);
        if ((hr) < 0) {
            return E_FAIL;
        }
    
        hr = (*pSource).AdviseSink(&IID_ITfActiveLanguageProfileNotifySink, (this_ as LPVOID).cast(), &mut (*this_).m_dwConnection);
        if ((hr) >= 0) {
            (*this_).m_pThreadMgr = pThreadMgr;
            (*pThreadMgr).AddRef();
            hr = S_OK;
        } else {
            hr = E_FAIL;
        }
    
        if !pSource.is_null() {
            (*pSource).Release();
        }
    
        return hr;
    }
    pub unsafe fn _Unadvise(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_pThreadMgr.is_null() {
            return E_FAIL;
        }
    
        let mut pSource: *mut ITfSource = core::ptr::null_mut();
        let mut hr: HRESULT = (*(*this_).m_pThreadMgr).QueryInterface(&IID_ITfSource, &mut pSource as *mut *mut ITfSource as *mut LPVOID);
        if hr >= 0 {
            hr = (*pSource).UnadviseSink((*this_).m_dwConnection);
            if hr >= 0 {
                hr = S_OK;
            }
        }
    
        if !pSource.is_null() {
            (*pSource).Release();
        }
    
        if !(*this_).m_pThreadMgr.is_null() {
            (*(*this_).m_pThreadMgr).Release();
            (*this_).m_pThreadMgr = core::ptr::null_mut();
        }
    
        hr
    }
    pub unsafe fn QueryInterface(&mut self, riid: &IID, ppvObj: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        let c_tab: [QITAB; 2] = [
            QITAB {
                piid: &raw const IID_ITfActiveLanguageProfileNotifySink,
                dwOffset: core::mem::offset_of!(CActiveLanguageProfileNotifySink, __base0) as DWORD,
            },
            QITAB {
                piid: core::ptr::null(),
                dwOffset: 0,
            },
        ];
        return QISearch((this_).cast(), c_tab.as_ptr(), riid, ppvObj);
    }
    pub unsafe fn AddRef(&mut self) -> ULONG {
        let this_: *mut Self = self;
        return (InterlockedIncrement(core::ptr::addr_of_mut!((*this_).m_cRefs))) as ULONG;
    }
    pub unsafe fn Release(&mut self) -> ULONG {
        let this_: *mut Self = self;
        if InterlockedDecrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) == 0 {
            CActiveLanguageProfileNotifySink__delete(this_);
            return 0;
        }
        return (*this_).m_cRefs as ULONG;
    }
    pub unsafe fn OnActivated(&mut self, clsid: &CLSID, guidProfile: &GUID, fActivated: BOOL) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_fnCompare.is_none() {
            return 0;
        }
    
        return (*this_).m_fnCompare.unwrap_unchecked()(clsid, guidProfile, fActivated, (*this_).m_pUserData);
    }
}

impl TEXT_EVENT_SINK_END_EDIT {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
