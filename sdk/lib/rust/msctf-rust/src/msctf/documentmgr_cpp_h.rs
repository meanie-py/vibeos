use crate::prelude::*;
use crate::msctf::compartmentmgr_cpp_h::CompartmentMgr_Constructor;
use crate::msctf::context_cpp_h::Context_Constructor;
use crate::msctf::context_cpp_h::Context_Initialize;
use crate::msctf::context_cpp_h::Context_Uninitialize;
use crate::msctf::msctf_cpp_h::TF_GetThreadMgr;
use crate::msctf::threadmgr_cpp_h::ThreadMgr_OnDocumentMgrDestruction;
use crate::msctf::msctf_cpp_h::advise_sink;
use crate::msctf::msctf_cpp_h::free_sinks;
use crate::msctf::msctf_cpp_h::get_Cookie_magic;
use crate::msctf::msctf_cpp_h::unadvise_sink;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "C" fn DocumentMgr_Constructor(pThreadMgrSink: *mut ITfThreadMgrEventSink, ppOut: *mut *mut ITfDocumentMgr) -> HRESULT {
    return CDocumentMgr::CreateInstance(pThreadMgrSink, ppOut);
}
#[no_mangle]
pub unsafe extern "C" fn EnumTfContext_Constructor(mgr: *mut CDocumentMgr, ppOut: *mut *mut IEnumTfContexts) -> HRESULT {
    return CEnumTfContext::CreateInstance(mgr, ppOut);
}

impl CDocumentMgr {
    pub unsafe fn __ctor_0(&mut self, threadMgrSink: *mut ITfThreadMgrEventSink) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_cRefs = 1 as LONG;
        (*this_).m_pCompartmentMgr = core::ptr::null_mut();
        (*this_).m_initialContext = core::ptr::null_mut();
        (*this_).m_pThreadMgrSink = threadMgrSink;
        {
            (*this_).m_contextStack[0 as usize] = core::ptr::null_mut();
            (*this_).m_contextStack[1 as usize] = (*this_).m_contextStack[0 as usize];
    
            list_init(core::ptr::addr_of_mut!((*this_).m_transitoryExtensionSink));
    
            let pDocMgr: *mut ITfDocumentMgr = core::ptr::addr_of_mut!((*this_).__base0) as *mut ITfDocumentMgr;
            let ppCompMgr: *mut *mut ITfCompartmentMgr = core::ptr::addr_of_mut!((*this_).m_pCompartmentMgr) as *mut *mut ITfCompartmentMgr;
            CompartmentMgr_Constructor((pDocMgr).cast(), &IID_IUnknown, ppCompMgr as *mut *mut IUnknown);
    
            let mut cookie: DWORD = core::mem::zeroed();
            Context_Constructor(g_processId, core::ptr::null_mut(), pDocMgr, core::ptr::addr_of_mut!((*this_).m_initialContext), core::ptr::addr_of_mut!(cookie));
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        let mut tm: *mut ITfThreadMgr = core::ptr::null_mut();
        TF_GetThreadMgr(core::ptr::addr_of_mut!(tm));
        if !tm.is_null() {
            ThreadMgr_OnDocumentMgrDestruction(tm, core::ptr::addr_of_mut!((*this_).__base0) as *mut ITfDocumentMgr);
            (*tm).Release();
        }
    
        if !(*this_).m_initialContext.is_null() {
            (*(*this_).m_initialContext).Release();
        }
        if !(*this_).m_contextStack[0 as usize].is_null() {
            (*(*this_).m_contextStack[0 as usize]).Release();
        }
        if !(*this_).m_contextStack[1 as usize].is_null() {
            (*(*this_).m_contextStack[1 as usize]).Release();
        }
    
        free_sinks((core::ptr::addr_of_mut!((*this_).m_transitoryExtensionSink)).cast());
    
        if !(*this_).m_pCompartmentMgr.is_null() {
            (*(*this_).m_pCompartmentMgr).Release();
            (*this_).m_pCompartmentMgr = core::ptr::null_mut();
        }
    }
    pub unsafe fn CreateInstance(pThreadMgrSink: *mut ITfThreadMgrEventSink, ppOut: *mut *mut ITfDocumentMgr) -> HRESULT {
        if ppOut.is_null() {
            return E_POINTER;
        }
    
        if pThreadMgrSink.is_null() {
            return E_INVALIDARG;
        }
    
        let This: *mut CDocumentMgr = CDocumentMgr__new_in_0(cpp_alloc_CicNoThrow::<CDocumentMgr>(), pThreadMgrSink);
        if This.is_null() {
            return E_OUTOFMEMORY;
        }
    
        *ppOut = core::ptr::addr_of_mut!((*This).__base0) as *mut ITfDocumentMgr;
        return S_OK;
    }
    pub unsafe fn QueryInterface(&mut self, iid: &IID, ppvObject: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        *ppvObject = core::ptr::null_mut();
    
        let mut pUnk: *mut IUnknown = core::ptr::null_mut();
        if IsEqualGUID!(&(*iid), &(IID_IUnknown)) || IsEqualGUID!(&(*iid), &(IID_ITfDocumentMgr)) {
            pUnk = core::ptr::addr_of_mut!((*this_).__base0) as *mut ITfDocumentMgr as *mut IUnknown;
        } else if IsEqualGUID!(&(*iid), &(IID_ITfSource)) {
            pUnk = core::ptr::addr_of_mut!((*this_).__base1) as *mut ITfSource as *mut IUnknown;
        } else if IsEqualGUID!(&(*iid), &(IID_ITfCompartmentMgr)) {
            pUnk = (*this_).m_pCompartmentMgr as *mut IUnknown;
        }
    
        if !pUnk.is_null() {
            (*pUnk).AddRef();
            *ppvObject = pUnk as LPVOID;
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
            CDocumentMgr__delete(this_);
        }
        return ret;
    }
    pub unsafe fn CreateContext(&mut self, tidOwner: TfClientId, dwFlags: DWORD, punk: *mut IUnknown, ppic: *mut *mut ITfContext, pecTextStore: *mut TfEditCookie) -> HRESULT {
        let this_: *mut Self = self;
        return Context_Constructor(tidOwner, punk, (this_).cast(), ppic, pecTextStore);
    }
    pub unsafe fn Push(&mut self, pic: *mut ITfContext) -> HRESULT {
        let this_: *mut Self = self;
        if !(*this_).m_contextStack[1].is_null() /* Full */
        {
            return TF_E_STACKFULL;
        }
    
        if pic.is_null()
        {
            return E_INVALIDARG;
        }
    
        let mut check: *mut ITfContext = core::ptr::null_mut();
        let hr: HRESULT = (*pic).QueryInterface(&IID_ITfContext, core::ptr::addr_of_mut!(check) as *mut LPVOID);
        if hr < 0
        {
            return E_INVALIDARG;
        }
    
        if (*this_).m_contextStack[0].is_null()
        {
            (*(*this_).m_pThreadMgrSink).OnInitDocumentMgr((this_).cast());
        }
    
        (*this_).m_contextStack[1] = (*this_).m_contextStack[0];
        (*this_).m_contextStack[0] = check;
    
        Context_Initialize(check, (this_).cast());
        (*(*this_).m_pThreadMgrSink).OnPushContext(check);
    
        return S_OK;
    }
    pub unsafe fn Pop(&mut self, dwFlags: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if dwFlags == TF_POPF_ALL {
            let mut i: usize = 0;
            while i < _countof!((*this_).m_contextStack) as usize {
                if (*this_).m_contextStack[i].is_null() {
                    i += 1;
                    continue;
                }
    
                (*(*this_).m_pThreadMgrSink).OnPopContext((*this_).m_contextStack[i]);
                Context_Uninitialize((*this_).m_contextStack[i]);
                (*(*this_).m_contextStack[i]).Release();
                (*this_).m_contextStack[i] = core::ptr::null_mut();
                i += 1;
            }
    
            (*(*this_).m_pThreadMgrSink).OnUninitDocumentMgr(core::ptr::addr_of_mut!((*this_).__base0));
            return S_OK;
        }
    
        if dwFlags != 0 {
            return E_INVALIDARG;
        }
    
        if (*this_).m_contextStack[1].is_null() {
            // Cannot pop last context
            return E_FAIL;
        }
    
        (*(*this_).m_pThreadMgrSink).OnPopContext((*this_).m_contextStack[0]);
        Context_Uninitialize((*this_).m_contextStack[0]);
    
        if !(*this_).m_contextStack[0].is_null() {
            (*(*this_).m_contextStack[0]).Release();
        }
    
        (*this_).m_contextStack[0] = (*this_).m_contextStack[1];
        (*this_).m_contextStack[1] = core::ptr::null_mut();
    
        if (*this_).m_contextStack[0].is_null() {
            (*(*this_).m_pThreadMgrSink).OnUninitDocumentMgr(core::ptr::addr_of_mut!((*this_).__base0));
        }
    
        return S_OK;
    }
    pub unsafe fn GetTop(&mut self, ppic: *mut *mut ITfContext) -> HRESULT {
        let this_: *mut Self = self;
        if ppic.is_null() {
            return E_INVALIDARG;
        }
    
        let mut target: *mut ITfContext;
        if !(*this_).m_contextStack[0 as usize].is_null() {
            target = (*this_).m_contextStack[0 as usize];
        } else {
            target = (*this_).m_initialContext;
        }
    
        if !target.is_null() {
            (*target).AddRef();
        }
    
        *ppic = target;
        return S_OK;
    }
    pub unsafe fn GetBase(&mut self, ppic: *mut *mut ITfContext) -> HRESULT {
        let this_: *mut Self = self;
        if ppic.is_null() {
            return E_INVALIDARG;
        }
    
        let target: *mut ITfContext;
        if !(*this_).m_contextStack[1 as usize].is_null() {
            target = (*this_).m_contextStack[1 as usize];
        } else if !(*this_).m_contextStack[0 as usize].is_null() {
            target = (*this_).m_contextStack[0 as usize];
        } else {
            target = (*this_).m_initialContext;
        }
    
        if !target.is_null() {
            (*target).AddRef();
        }
    
        *ppic = target;
        return S_OK;
    }
    pub unsafe fn EnumContexts(&mut self, ppEnum: *mut *mut IEnumTfContexts,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // TRACE("%p -> (%p)\n", this_, ppEnum);
        EnumTfContext_Constructor(this_, ppEnum)
    }
    pub unsafe fn AdviseSink(&mut self, riid: &IID, punk: *mut IUnknown, pdwCookie: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if cicIsNullPtr(riid as *const IID) || punk.is_null() || pdwCookie.is_null() {
            return E_INVALIDARG;
        }
    
        if IsEqualGUID!(&(*riid), &(IID_ITfTransitoryExtensionSink)) {
            return advise_sink((core::ptr::addr_of_mut!((*this_).m_transitoryExtensionSink)).cast(), &IID_ITfTransitoryExtensionSink,
                               COOKIE_MAGIC_DMSINK, punk, pdwCookie);
        }
    
        return E_NOTIMPL;
    }
    pub unsafe fn UnadviseSink(&mut self, pdwCookie: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if get_Cookie_magic(pdwCookie) != COOKIE_MAGIC_DMSINK {
            return E_INVALIDARG;
        }
    
        unadvise_sink(pdwCookie)
    }
}

impl CEnumTfContext {
    pub unsafe fn __ctor_0(&mut self, mgr: *mut CDocumentMgr) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_cRefs = 1;
        (*this_).m_index = 0;
        (*this_).m_pDocMgr = mgr;
        {
            if !mgr.is_null() {
                (*mgr).AddRef();
            }
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pDocMgr.is_null() {
            (*(*this_).m_pDocMgr).Release();
            (*this_).m_pDocMgr = core::ptr::null_mut();
        }
    }
    pub unsafe fn CreateInstance(mgr: *mut CDocumentMgr, ppOut: *mut *mut IEnumTfContexts) -> HRESULT {
        if ppOut.is_null() {
            return E_POINTER;
        }
    
        let This: *mut CEnumTfContext = CEnumTfContext__new_in_0(cpp_alloc_CicNoThrow::<CEnumTfContext>(), mgr);
        if This.is_null() {
            return E_OUTOFMEMORY;
        }
    
        *ppOut = core::ptr::addr_of_mut!((*This).__base0) as *mut IEnumTfContexts;
        return S_OK;
    }
    pub unsafe fn QueryInterface(&mut self, iid: &IID, ppvObject: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        *ppvObject = core::ptr::null_mut();
    
        if IsEqualIID!(iid as *const IID, core::ptr::addr_of!(IID_IUnknown)) != false
            || IsEqualIID!(iid as *const IID, core::ptr::addr_of!(IID_IEnumTfContexts)) != false
        {
            *ppvObject = core::ptr::addr_of_mut!((*this_).__base0) as LPVOID;
        }
    
        if !(*ppvObject).is_null() {
            (*this_).AddRef();
            return S_OK;
        }
    
        return E_NOINTERFACE;
    }
    pub unsafe fn AddRef(&mut self) -> ULONG {
        let this_: *mut Self = self;
        return InterlockedIncrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) as ULONG;
    }
    pub unsafe fn Release(&mut self) -> ULONG {
        let this_: *mut Self = self;
        let ret: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) as ULONG;
        if ret == 0 {
            CEnumTfContext__delete(this_);
        }
        return ret;
    }
    pub unsafe fn Next(&mut self, ulCount: ULONG, mut rgContext: *mut *mut ITfContext, pcFetched: *mut ULONG) -> HRESULT {
        let this_: *mut Self = self;
        if rgContext.is_null() {
            return E_POINTER;
        }
    
        let mut fetched: ULONG = 0;
        while fetched < ulCount {
            if ((*this_).m_index as usize) >= (_countof!((*(*this_).m_pDocMgr).m_contextStack) as usize) {
                break;
            }
    
            if (*(*this_).m_pDocMgr).m_contextStack[(*this_).m_index as usize].is_null() {
                break;
            }
    
            *rgContext = (*(*this_).m_pDocMgr).m_contextStack[(*this_).m_index as usize];
            (**rgContext).AddRef();
    
            fetched += 1;
            (*this_).m_index += 1;
            rgContext = rgContext.add(1);
        }
    
        if !pcFetched.is_null() {
            *pcFetched = fetched;
        }
    
        return if fetched == ulCount { S_OK } else { S_FALSE };
    }
    pub unsafe fn Skip(&mut self, celt: ULONG) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_index = (*this_).m_index.wrapping_add(celt);
        return S_OK;
    }
    pub unsafe fn Reset(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_index = 0;
        return S_OK;
    }
    pub unsafe fn Clone(&mut self, ppenum: *mut *mut IEnumTfContexts) -> HRESULT {
        let this_: *mut Self = self;
        if ppenum.is_null() {
            return E_POINTER;
        }
    
        let This: *mut CEnumTfContext = CEnumTfContext__new_in_0(cpp_alloc_CicNoThrow::<CEnumTfContext>(), (*this_).m_pDocMgr);
        if This.is_null() {
            return E_OUTOFMEMORY;
        }
    
        (*This).m_index = (*this_).m_index;
        *ppenum = core::ptr::addr_of_mut!((*This).__base0) as *mut IEnumTfContexts;
        return S_OK;
    }
}
