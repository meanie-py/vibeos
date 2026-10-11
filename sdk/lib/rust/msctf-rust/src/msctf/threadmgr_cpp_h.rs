use crate::prelude::*;
use crate::msctf::categorymgr_cpp_h::CategoryMgr_Constructor;
use crate::msctf::compartmentmgr_cpp_h::CompartmentMgr_Constructor;
use crate::msctf::documentmgr_cpp_h::DocumentMgr_Constructor;
use crate::msctf::msctf_cpp_h::activate_textservices;
use crate::msctf::msctf_cpp_h::advise_sink;
use crate::msctf::msctf_cpp_h::deactivate_textservices;
use crate::msctf::msctf_cpp_h::free_sinks;
use crate::msctf::msctf_cpp_h::get_Cookie_magic;
use crate::msctf::msctf_cpp_h::get_textservice_clsid;
use crate::msctf::msctf_cpp_h::get_textservice_sink;
use crate::msctf::msctf_cpp_h::list;
use crate::msctf::msctf_cpp_h::set_textservice_sink;
use crate::msctf::msctf_cpp_h::unadvise_sink;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct PreservedKey {
    pub entry: list,
    pub guid: GUID,
    pub prekey: TF_PRESERVEDKEY,
    pub description: LPWSTR,
    pub tid: TfClientId,
}

#[repr(C)]
pub struct DocumentMgrEntry {
    pub entry: list,
    pub docmgr: *mut ITfDocumentMgr,
}

#[repr(C)]
pub struct AssociatedWindow {
    pub entry: list,
    pub hwnd: HWND,
    pub docmgr: *mut ITfDocumentMgr,
}
















































































#[no_mangle]
pub unsafe extern "C" fn ThreadMgr_Constructor(pUnkOuter: *mut IUnknown, ppOut: *mut *mut IUnknown) -> HRESULT {
    return CThreadMgr::CreateInstance(pUnkOuter, ppOut as *mut *mut CThreadMgr);
}
#[no_mangle]
pub unsafe extern "C" fn ThreadMgr_OnDocumentMgrDestruction(iface: *mut ITfThreadMgr, mgr: *mut ITfDocumentMgr) {
    let This: *mut CThreadMgr = iface as *mut CThreadMgr;
    (*This).OnDocumentMgrDestruction(mgr);
}

impl tagPreservedKey {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl tagDocumentMgrs {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl tagAssociatedWindow {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl CThreadMgr {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_cRefs = 1;
        (*this_).m_CompartmentMgr = core::ptr::null_mut();
        (*this_).m_focus = core::ptr::null_mut();
        (*this_).m_activationCount = 0;
        (*this_).m_foregroundKeyEventSink = core::ptr::null_mut();
        {
            (*this_).m_foregroundTextService = GUID_NULL;
    
            list_init(core::ptr::addr_of_mut!((*this_).m_CurrentPreservedKeys));
            list_init(core::ptr::addr_of_mut!((*this_).m_CreatedDocumentMgrs));
    
            list_init(core::ptr::addr_of_mut!((*this_).m_AssociatedFocusWindows));
            (*this_).m_focusHook = core::ptr::null_mut();
    
            /* kept as separate lists to reduce unnecessary iterations */
            list_init(core::ptr::addr_of_mut!((*this_).m_ActiveLanguageProfileNotifySink));
            list_init(core::ptr::addr_of_mut!((*this_).m_DisplayAttributeNotifySink));
            list_init(core::ptr::addr_of_mut!((*this_).m_KeyTraceEventSink));
            list_init(core::ptr::addr_of_mut!((*this_).m_PreservedKeyNotifySink));
            list_init(core::ptr::addr_of_mut!((*this_).m_ThreadFocusSink));
            list_init(core::ptr::addr_of_mut!((*this_).m_ThreadMgrEventSink));
            list_init(core::ptr::addr_of_mut!((*this_).m_UIElementSink));
            list_init(core::ptr::addr_of_mut!((*this_).m_InputProcessorProfileActivationSink));
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        let mut cursor: *mut list;
        let mut cursor2: *mut list;
    
        /* unhook right away */
        if !(*this_).m_focusHook.is_null() {
            UnhookWindowsHookEx((*this_).m_focusHook);
        }
    
        TlsSetValue(g_dwTLSIndex, core::ptr::null_mut());
    
        if !(*this_).m_focus.is_null() {
            (*(*this_).m_focus).Release();
        }
    
        free_sinks((core::ptr::addr_of_mut!((*this_).m_ActiveLanguageProfileNotifySink)).cast());
        free_sinks((core::ptr::addr_of_mut!((*this_).m_DisplayAttributeNotifySink)).cast());
        free_sinks((core::ptr::addr_of_mut!((*this_).m_KeyTraceEventSink)).cast());
        free_sinks((core::ptr::addr_of_mut!((*this_).m_PreservedKeyNotifySink)).cast());
        free_sinks((core::ptr::addr_of_mut!((*this_).m_ThreadFocusSink)).cast());
        free_sinks((core::ptr::addr_of_mut!((*this_).m_ThreadMgrEventSink)).cast());
        free_sinks((core::ptr::addr_of_mut!((*this_).m_UIElementSink)).cast());
        free_sinks((core::ptr::addr_of_mut!((*this_).m_InputProcessorProfileActivationSink)).cast());
    
        // LIST_FOR_EACH_SAFE(cursor, cursor2, &this_->this_->m_CurrentPreservedKeys)
        // (source names `this_->this_`; the only valid member path is `(*this_)`)
        cursor = ((*this_).m_CurrentPreservedKeys.next).cast();
        while cursor != core::ptr::addr_of_mut!((*this_).m_CurrentPreservedKeys) {
            cursor2 = (*cursor).next;
            {
                let key: *mut PreservedKey = LIST_ENTRY!(cursor, PreservedKey, entry);
                list_remove((cursor).cast());
                cicMemFree(((*key).description).cast());
                cicMemFree((key).cast());
            }
            cursor = cursor2;
        }
    
        // LIST_FOR_EACH_SAFE(cursor, cursor2, &this_->this_->m_CreatedDocumentMgrs)
        cursor = ((*this_).m_CreatedDocumentMgrs.next).cast();
        while cursor != core::ptr::addr_of_mut!((*this_).m_CreatedDocumentMgrs) {
            cursor2 = (*cursor).next;
            {
                let mgr: *mut DocumentMgrEntry = LIST_ENTRY!(cursor, DocumentMgrEntry, entry);
                list_remove((cursor).cast());
    
                cicMemFree((mgr).cast());
            }
            cursor = cursor2;
        }
    
        // LIST_FOR_EACH_SAFE(cursor, cursor2, &this_->this_->m_AssociatedFocusWindows)
        cursor = ((*this_).m_AssociatedFocusWindows.next).cast();
        while cursor != core::ptr::addr_of_mut!((*this_).m_AssociatedFocusWindows) {
            cursor2 = (*cursor).next;
            {
                let wnd: *mut AssociatedWindow = LIST_ENTRY!(cursor, AssociatedWindow, entry);
                list_remove((cursor).cast());
                cicMemFree((wnd).cast());
            }
            cursor = cursor2;
        }
    
        (*(*this_).m_CompartmentMgr).Release();
    }
    pub unsafe fn CreateInstance(pUnkOuter: *mut IUnknown, ppOut: *mut *mut CThreadMgr) -> HRESULT {
        if !pUnkOuter.is_null() {
            return CLASS_E_NOAGGREGATION;
        }
    
        /* Only 1 ThreadMgr is created per thread */
        let mut This: *mut CThreadMgr = TlsGetValue(g_dwTLSIndex) as *mut CThreadMgr;
        if !This.is_null() {
            (*This).AddRef();
            *ppOut = This;
            return S_OK;
        }
    
        This = CThreadMgr__new_in_0(cpp_alloc_CicNoThrow::<CThreadMgr>());
        if This.is_null() {
            return E_OUTOFMEMORY;
        }
    
        TlsSetValue(g_dwTLSIndex, This as LPVOID);
    
        let mut pCompMgr: *mut ITfCompartmentMgr = core::ptr::null_mut();
        CompartmentMgr_Constructor(
            (core::ptr::addr_of_mut!((*This).__base0) as *mut ITfThreadMgrEx).cast(),
            &IID_IUnknown,
            core::ptr::addr_of_mut!(pCompMgr) as *mut *mut IUnknown,
        );
        (*This).m_CompartmentMgr = pCompMgr;
    
        *ppOut = This;
        return S_OK;
    }
    pub unsafe fn OnDocumentMgrDestruction(&mut self, mgr: *mut ITfDocumentMgr) {
        let this_: *mut Self = self;
        // C source reads `this_->this_->m_CreatedDocumentMgrs`; CThreadMgr has no `this_` field, so the second hop is dropped.
        let head: *mut list = core::ptr::addr_of_mut!((*this_).m_CreatedDocumentMgrs);
        let mut cursor: *mut list = (*head).next;
        while cursor != head {
            let mgrentry: *mut DocumentMgrEntry = LIST_ENTRY!(cursor, DocumentMgrEntry, entry);
            if (*mgrentry).docmgr == mgr {
                list_remove((cursor).cast());
                cicMemFree((mgrentry).cast());
                return;
            }
            cursor = (*cursor).next;
        }
    }
    pub unsafe fn QueryInterface(&mut self, iid: &IID, ppvObject: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        *ppvObject = core::ptr::null_mut();
    
        let mut pUnk: *mut IUnknown = core::ptr::null_mut();
        if IsEqualGUID!(&(*iid), &(IID_IUnknown)) || IsEqualGUID!(&(*iid), &(IID_ITfThreadMgr)) || IsEqualGUID!(&(*iid), &(IID_ITfThreadMgrEx)) {
            pUnk = core::ptr::addr_of_mut!((*this_).__base0) as *mut IUnknown;
        } else if IsEqualGUID!(&(*iid), &(IID_ITfSource)) {
            pUnk = core::ptr::addr_of_mut!((*this_).__base1) as *mut IUnknown;
        } else if IsEqualGUID!(&(*iid), &(IID_ITfKeystrokeMgr)) {
            pUnk = core::ptr::addr_of_mut!((*this_).__base2) as *mut IUnknown;
        } else if IsEqualGUID!(&(*iid), &(IID_ITfMessagePump)) {
            pUnk = core::ptr::addr_of_mut!((*this_).__base3) as *mut IUnknown;
        } else if IsEqualGUID!(&(*iid), &(IID_ITfClientId)) {
            pUnk = core::ptr::addr_of_mut!((*this_).__base4) as *mut IUnknown;
        } else if IsEqualGUID!(&(*iid), &(IID_ITfCompartmentMgr)) {
            pUnk = (*this_).m_CompartmentMgr as *mut IUnknown;
        } else if IsEqualGUID!(&(*iid), &(IID_ITfUIElementMgr)) {
            pUnk = core::ptr::addr_of_mut!((*this_).__base5) as *mut IUnknown;
        } else if IsEqualGUID!(&(*iid), &(IID_ITfSourceSingle)) {
            pUnk = core::ptr::addr_of_mut!((*this_).__base6) as *mut IUnknown;
        }
    
        if !pUnk.is_null() {
            *ppvObject = pUnk as LPVOID;
            (*pUnk).AddRef();
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
        if !(ret != 0) {
            CThreadMgr__delete(this_);
        }
        return ret;
    }
    pub unsafe fn Activate(&mut self, ptid: *mut TfClientId) -> HRESULT {
        let this_: *mut Self = self;
        return (*this_).ActivateEx(ptid, 0);
    }
    pub unsafe fn Deactivate(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_activationCount == 0 {
            return E_UNEXPECTED;
        }
    
        (*this_).m_activationCount -= 1;
    
        if (*this_).m_activationCount == 0 {
            if !(*this_).m_focus.is_null() {
                (*this_).OnSetFocus(core::ptr::null_mut(), (*this_).m_focus);
                (*(*this_).m_focus).Release();
                (*this_).m_focus = core::ptr::null_mut();
            }
        }
    
        deactivate_textservices();
        return S_OK;
    }
    pub unsafe fn CreateDocumentMgr(&mut self, ppdim: *mut *mut ITfDocumentMgr) -> HRESULT {
        let this_: *mut Self = self;
        if ppdim.is_null() {
            return E_INVALIDARG;
        }
    
        let mgrentry: *mut DocumentMgrEntry = cicMemAlloc(core::mem::size_of::<DocumentMgrEntry>()) as *mut DocumentMgrEntry;
        if mgrentry.is_null() {
            return E_OUTOFMEMORY;
        }
    
        let hr: HRESULT = DocumentMgr_Constructor((this_).cast(), ppdim);
        if hr >= 0 {
            (*mgrentry).docmgr = *ppdim;
            list_add_head(
                core::ptr::addr_of_mut!((*this_).m_CreatedDocumentMgrs),
                (core::ptr::addr_of_mut!((*mgrentry).entry)).cast(),
            );
        } else {
            cicMemFree(mgrentry as LPVOID);
        }
    
        hr
    }
    pub unsafe fn EnumDocumentMgrs(&mut self, ppEnum: *mut *mut IEnumTfDocumentMgrs) -> HRESULT {
        let this_: *mut Self = self;
        if ppEnum.is_null() {
            return E_INVALIDARG;
        }
    
        return CEnumTfDocumentMgr::CreateInstance((core::ptr::addr_of_mut!((*this_).m_CreatedDocumentMgrs)).cast(), ppEnum as *mut *mut CEnumTfDocumentMgr);
    }
    pub unsafe fn GetFocus(&mut self, ppdimFocus: *mut *mut ITfDocumentMgr) -> HRESULT {
        let this_: *mut Self = self;
        if ppdimFocus.is_null() {
            return E_INVALIDARG;
        }
    
        *ppdimFocus = (*this_).m_focus;
    
        if (*this_).m_focus.is_null() {
            return S_FALSE;
        }
    
        (*(*this_).m_focus).AddRef();
        return S_OK;
    }
    pub unsafe fn SetFocus(&mut self, pdimFocus: *mut ITfDocumentMgr) -> HRESULT {
        let this_: *mut Self = self;
        let mut check: *mut ITfDocumentMgr = core::ptr::null_mut();
    
        if pdimFocus.is_null() {
            check = core::ptr::null_mut();
        } else if (((*pdimFocus).QueryInterface(&IID_ITfDocumentMgr, core::ptr::addr_of_mut!(check) as *mut LPVOID)) < 0) {
            return E_INVALIDARG;
        }
    
        (*this_).OnSetFocus(check, (*this_).m_focus);
    
        if !(*this_).m_focus.is_null() {
            (*(*this_).m_focus).Release();
        }
    
        (*this_).m_focus = check;
        return S_OK;
    }
    pub unsafe fn AssociateFocus(&mut self, hwnd: HWND, pdimNew: *mut ITfDocumentMgr, ppdimPrev: *mut *mut ITfDocumentMgr) -> HRESULT {
        let this_: *mut Self = self;
        let mut cursor: *mut list = core::ptr::null_mut();
        let mut cursor2: *mut list = core::ptr::null_mut();
        let mut wnd: *mut AssociatedWindow;
    
        if ppdimPrev.is_null() {
            return E_INVALIDARG;
        }
    
        *ppdimPrev = core::ptr::null_mut();
    
        // LIST_FOR_EACH_SAFE(cursor, cursor2, &(*this_).m_AssociatedFocusWindows) expanded inline
        let head: *mut list = core::ptr::addr_of_mut!((*this_).m_AssociatedFocusWindows);
        cursor = (*head).next;
        while cursor != head {
            cursor2 = (*cursor).next;
            // LIST_ENTRY!(cursor, AssociatedWindow, entry)
            wnd = (cursor as *mut u8).sub(core::mem::offset_of!(AssociatedWindow, entry)) as *mut AssociatedWindow;
            if (*wnd).hwnd == hwnd {
                if !(*wnd).docmgr.is_null() {
                    (*(*wnd).docmgr).AddRef();
                }
                *ppdimPrev = (*wnd).docmgr;
                (*wnd).docmgr = pdimNew;
                if GetFocus() == hwnd {
                    (*this_).SetFocus(pdimNew);
                }
                return S_OK;
            }
            cursor = cursor2;
        }
    
        wnd = cicMemAlloc(core::mem::size_of::<AssociatedWindow>()) as *mut AssociatedWindow;
        (*wnd).hwnd = hwnd;
        (*wnd).docmgr = pdimNew;
        list_add_head(core::ptr::addr_of_mut!((*this_).m_AssociatedFocusWindows), (core::ptr::addr_of_mut!((*wnd).entry)).cast());
    
        if GetFocus() == hwnd {
            (*this_).SetFocus(pdimNew);
        }
    
        (*this_).SetupWindowsHook();
        return S_OK;
    }
    pub unsafe fn IsThreadFocus(&mut self, pfThreadFocus: *mut BOOL) -> HRESULT {
        let this_: *mut Self = self;
        if pfThreadFocus.is_null() {
            return E_INVALIDARG;
        }
    
        let focus: HWND = GetFocus();
        *pfThreadFocus = (focus as usize == 0) as BOOL;
        return S_OK;
    }
    pub unsafe fn GetFunctionProvider(&mut self, clsid: &CLSID, ppFuncProv: *mut *mut ITfFunctionProvider) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn EnumFunctionProviders(&mut self, ppEnum: *mut *mut IEnumTfFunctionProviders,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB:(%p)\n", this_);
        E_NOTIMPL
    }
    pub unsafe fn GetGlobalCompartment(&mut self, ppCompMgr: *mut *mut ITfCompartmentMgr) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT;
    
        if ppCompMgr.is_null() {
            return E_INVALIDARG;
        }
    
        if g_globalCompartmentMgr.is_null() {
            hr = CompartmentMgr_Constructor(core::ptr::null_mut(), &IID_ITfCompartmentMgr, core::ptr::addr_of_mut!(g_globalCompartmentMgr) as *mut *mut IUnknown);
            if hr < 0 {
                return hr;
            }
        }
    
        (*g_globalCompartmentMgr).AddRef();
        *ppCompMgr = g_globalCompartmentMgr;
        return S_OK;
    }
    pub unsafe fn ActivateEx(&mut self, id: *mut TfClientId, flags: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if id.is_null() {
            return E_INVALIDARG;
        }
    
        if flags != 0 {
            FIXME!("Unimplemented flags %#x\n", flags);
        }
    
        if g_processId == 0 {
            let mut guid: GUID = core::mem::zeroed();
            CoCreateGuid(core::ptr::addr_of_mut!(guid));
            (*this_).GetClientId(&guid, core::ptr::addr_of_mut!(g_processId));
        }
    
        activate_textservices((this_).cast());
        (*this_).m_activationCount += 1;
        *id = g_processId;
        return S_OK;
    }
    pub unsafe fn GetActiveFlags(&mut self, flags: *mut DWORD,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB:(%p)\n", this_);
        E_NOTIMPL
    }
    pub unsafe fn AdviseSink(&mut self, riid: &IID, punk: *mut IUnknown, pdwCookie: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if cicIsNullPtr(riid as *const GUID) || punk.is_null() || pdwCookie.is_null() {
            return E_INVALIDARG;
        }
    
        if IsEqualGUID!(&(*riid), &(IID_ITfThreadMgrEventSink)) {
            return advise_sink((core::ptr::addr_of_mut!((*this_).m_ThreadMgrEventSink)).cast(), &IID_ITfThreadMgrEventSink, COOKIE_MAGIC_TMSINK, punk, pdwCookie);
        }
    
        if IsEqualGUID!(&(*riid), &(IID_ITfThreadFocusSink)) {
            return advise_sink((core::ptr::addr_of_mut!((*this_).m_ThreadFocusSink)).cast(), &IID_ITfThreadFocusSink, COOKIE_MAGIC_THREADFOCUSSINK, punk, pdwCookie);
        }
    
        if IsEqualGUID!(&(*riid), &(IID_ITfActiveLanguageProfileNotifySink)) {
            return advise_sink((core::ptr::addr_of_mut!((*this_).m_ActiveLanguageProfileNotifySink)).cast(), &IID_ITfActiveLanguageProfileNotifySink, COOKIE_MAGIC_ACTIVELANGSINK, punk, pdwCookie);
        }
    
        if IsEqualGUID!(&(*riid), &(IID_ITfKeyTraceEventSink)) {
            return advise_sink((core::ptr::addr_of_mut!((*this_).m_KeyTraceEventSink)).cast(), &IID_ITfKeyTraceEventSink, COOKIE_MAGIC_KEYTRACESINK, punk, pdwCookie);
        }
    
        if IsEqualGUID!(&(*riid), &(IID_ITfUIElementSink)) {
            return advise_sink((core::ptr::addr_of_mut!((*this_).m_UIElementSink)).cast(), &IID_ITfUIElementSink, COOKIE_MAGIC_UIELEMENTSINK, punk, pdwCookie);
        }
    
        if IsEqualGUID!(&(*riid), &(IID_ITfInputProcessorProfileActivationSink)) {
            return advise_sink((core::ptr::addr_of_mut!((*this_).m_InputProcessorProfileActivationSink)).cast(), &IID_ITfInputProcessorProfileActivationSink, COOKIE_MAGIC_INPUTPROCESSORPROFILEACTIVATIONSINK, punk, pdwCookie);
        }
    
        return E_NOTIMPL;
    }
    pub unsafe fn UnadviseSink(&mut self, dwCookie: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        let mut magic: DWORD;
    
        magic = get_Cookie_magic(dwCookie);
        if magic != COOKIE_MAGIC_TMSINK
            && magic != COOKIE_MAGIC_THREADFOCUSSINK
            && magic != COOKIE_MAGIC_KEYTRACESINK
            && magic != COOKIE_MAGIC_UIELEMENTSINK
            && magic != COOKIE_MAGIC_INPUTPROCESSORPROFILEACTIVATIONSINK
        {
            return E_INVALIDARG;
        }
    
        return unadvise_sink(dwCookie);
    }
    pub unsafe fn AdviseKeyEventSink(&mut self, tid: TfClientId, pSink: *mut ITfKeyEventSink, fForeground: BOOL) -> HRESULT {
        let this_: *mut Self = self;
        let mut textservice: CLSID = core::mem::zeroed();
        let mut check: *mut ITfKeyEventSink = core::ptr::null_mut();
    
        if tid == 0 || pSink.is_null() {
            return E_INVALIDARG;
        }
    
        textservice = get_textservice_clsid(tid);
        if IsEqualGUID!(&(GUID_NULL), &(textservice)) {
            return E_INVALIDARG;
        }
    
        get_textservice_sink(tid, &IID_ITfKeyEventSink, core::ptr::addr_of_mut!(check) as *mut *mut IUnknown);
        if !check.is_null() {
            return CONNECT_E_ADVISELIMIT;
        }
    
        if (*pSink).QueryInterface(&IID_ITfKeyEventSink, core::ptr::addr_of_mut!(check) as *mut LPVOID) < 0 {
            return E_INVALIDARG;
        }
    
        set_textservice_sink(tid, &IID_ITfKeyEventSink, (check).cast());
    
        if fForeground != 0 {
            if !(*this_).m_foregroundKeyEventSink.is_null() {
                (*(*this_).m_foregroundKeyEventSink).OnSetFocus((FALSE) as i32);
                (*(*this_).m_foregroundKeyEventSink).Release();
            }
            (*check).AddRef();
            (*check).OnSetFocus((TRUE) as i32);
            (*this_).m_foregroundKeyEventSink = check;
            (*this_).m_foregroundTextService = textservice;
        }
        return S_OK;
    }
    pub unsafe fn UnadviseKeyEventSink(&mut self, tid: TfClientId) -> HRESULT {
        let this_: *mut Self = self;
        let mut textservice: CLSID = core::mem::zeroed();
        let mut check: *mut ITfKeyEventSink = core::ptr::null_mut();
    
        if !(tid != 0) {
            return E_INVALIDARG;
        }
    
        textservice = get_textservice_clsid(tid);
        if IsEqualGUID!(&(GUID_NULL), &(textservice)) {
            return E_INVALIDARG;
        }
    
        get_textservice_sink(tid, &IID_ITfKeyEventSink, core::ptr::addr_of_mut!(check) as *mut *mut IUnknown);
    
        if check.is_null() {
            return CONNECT_E_NOCONNECTION;
        }
    
        set_textservice_sink(tid, &IID_ITfKeyEventSink, core::ptr::null_mut());
        (*check).Release();
    
        if (*this_).m_foregroundKeyEventSink == check {
            (*(*this_).m_foregroundKeyEventSink).Release();
            (*this_).m_foregroundKeyEventSink = core::ptr::null_mut();
            (*this_).m_foregroundTextService = GUID_NULL;
        }
        return S_OK;
    }
    pub unsafe fn GetForeground(&mut self, pclsid: *mut CLSID) -> HRESULT {
        let this_: *mut Self = self;
        if pclsid.is_null() {
            return E_INVALIDARG;
        }
    
        if IsEqualGUID!(&((*this_).m_foregroundTextService), &(GUID_NULL)) {
            return S_FALSE;
        }
    
        *pclsid = (*this_).m_foregroundTextService;
        return S_OK;
    }
    pub unsafe fn TestKeyDown(&mut self, wParam: WPARAM, lParam: LPARAM, pfEaten: *mut BOOL) -> HRESULT {
        let this_: *mut Self = self;
        if pfEaten.is_null() {
            return E_INVALIDARG;
        }
        *pfEaten = (FALSE) as i32;
        return S_OK;
    }
    pub unsafe fn TestKeyUp(&mut self, wParam: WPARAM, lParam: LPARAM, pfEaten: *mut BOOL) -> HRESULT {
        let this_: *mut Self = self;
        if pfEaten.is_null() {
            return E_INVALIDARG;
        }
        *pfEaten = (FALSE) as i32;
        return S_OK;
    }
    pub unsafe fn KeyDown(&mut self, wParam: WPARAM, lParam: LPARAM, pfEaten: *mut BOOL) -> HRESULT {
        let this_: *mut Self = self;
        if pfEaten.is_null() {
            return E_INVALIDARG;
        }
        *pfEaten = (FALSE) as i32;
        return E_NOTIMPL;
    }
    pub unsafe fn KeyUp(&mut self, wParam: WPARAM, lParam: LPARAM, pfEaten: *mut BOOL) -> HRESULT {
        let this_: *mut Self = self;
        if pfEaten.is_null() {
            return E_INVALIDARG;
        }
        *pfEaten = (FALSE) as i32;
        return E_NOTIMPL;
    }
    pub unsafe fn GetPreservedKey(&mut self, pic: *mut ITfContext,
        pprekey: *const TF_PRESERVEDKEY,
        pguid: *mut GUID,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB:(%p)\n", this_);
        E_NOTIMPL
    }
    pub unsafe fn IsPreservedKey(&mut self, rguid: &GUID, pprekey: *const TF_PRESERVEDKEY, pfRegistered: *mut BOOL) -> HRESULT {
        let this_: *mut Self = self;
        let mut cursor: *mut list;
    
        if cicIsNullPtr(rguid as *const GUID) || pprekey.is_null() || pfRegistered.is_null() {
            return E_INVALIDARG;
        }
    
        // LIST_FOR_EACH(cursor, &(*this_).m_CurrentPreservedKeys)
        let list_head: *mut list = core::ptr::addr_of_mut!((*this_).m_CurrentPreservedKeys);
        cursor = (*list_head).next;
        while cursor != list_head {
            // PreservedKey* key = LIST_ENTRY!(cursor, PreservedKey, entry);
            let key: *mut PreservedKey = ((cursor as *mut u8)
                .sub(core::mem::offset_of!(PreservedKey, entry)))
                as *mut PreservedKey;
            if IsEqualGUID!(&(*rguid), &((*key).guid))
                && (*pprekey).uVKey == (*key).prekey.uVKey
                && (*pprekey).uModifiers == (*key).prekey.uModifiers
            {
                *pfRegistered = (TRUE) as i32;
                return S_OK;
            }
            cursor = (*cursor).next;
        }
    
        *pfRegistered = (FALSE) as i32;
        return S_FALSE;
    }
    pub unsafe fn PreserveKey(&mut self, tid: TfClientId, rguid: &GUID, prekey: *const TF_PRESERVEDKEY, pchDesc: *const WCHAR, cchDesc: ULONG) -> HRESULT {
        let this_: *mut Self = self;
        let mut cursor: *mut list;
    
        if tid == 0 || cicIsNullPtr(rguid as *const GUID) || prekey.is_null() || (cchDesc != 0 && pchDesc.is_null()) {
            return E_INVALIDARG;
        }
    
        let lst: *mut list = core::ptr::addr_of_mut!((*this_).m_CurrentPreservedKeys) as *mut list;
        cursor = (list_head(lst)).cast();
        while !cursor.is_null() {
            let key: *mut PreservedKey = (cursor as *mut u8).sub(core::mem::offset_of!(PreservedKey, entry)) as *mut PreservedKey;
            if IsEqualGUID!(&(*rguid), &((*key).guid)) && (*prekey).uVKey == (*key).prekey.uVKey && (*prekey).uModifiers == (*key).prekey.uModifiers {
                return TF_E_ALREADY_EXISTS;
            }
            cursor = (list_next(lst, cursor)).cast();
        }
    
        let newkey: *mut PreservedKey = cicMemAlloc(core::mem::size_of::<PreservedKey>()) as *mut PreservedKey;
        if newkey.is_null() {
            return E_OUTOFMEMORY;
        }
    
        (*newkey).guid = *rguid;
        (*newkey).prekey = *prekey;
        (*newkey).tid = tid;
        (*newkey).description = core::ptr::null_mut();
        if cchDesc != 0 {
            (*newkey).description = cicMemAlloc((cchDesc as usize + 1 as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR;
            if (*newkey).description.is_null() {
                cicMemFree((newkey).cast());
                return E_OUTOFMEMORY;
            }
            CopyMemory!((*newkey).description as *mut core::ffi::c_void, pchDesc as *const core::ffi::c_void, cchDesc as usize * core::mem::size_of::<WCHAR>());
            *(*newkey).description.add(cchDesc as usize) = UNICODE_NULL;
        }
    
        list_add_head(core::ptr::addr_of_mut!((*this_).m_CurrentPreservedKeys), (core::ptr::addr_of_mut!((*newkey).entry)).cast());
        return S_OK;
    }
    pub unsafe fn UnpreserveKey(&mut self, rguid: &GUID, pprekey: *const TF_PRESERVEDKEY) -> HRESULT {
        let this_: *mut Self = self;
        let mut key: *mut PreservedKey = core::ptr::null_mut();
        let mut cursor: *mut list = core::ptr::null_mut();
    
        if pprekey.is_null() || cicIsNullPtr(rguid as *const GUID) {
            return E_INVALIDARG;
        }
    
        let list_head_p: *mut list = core::ptr::addr_of_mut!((*this_).m_CurrentPreservedKeys);
        cursor = (*list_head_p).next;
        while cursor != list_head_p {
            key = LIST_ENTRY!(cursor, PreservedKey, entry);
            if IsEqualGUID!(&(*rguid), &((*key).guid))
                && (*pprekey).uVKey == (*key).prekey.uVKey
                && (*pprekey).uModifiers == (*key).prekey.uModifiers
            {
                break;
            }
            key = core::ptr::null_mut();
            cursor = (*cursor).next;
        }
    
        if key.is_null() {
            return CONNECT_E_NOCONNECTION;
        }
    
        list_remove((core::ptr::addr_of_mut!((*key).entry)).cast());
        cicMemFree(((*key).description).cast());
        cicMemFree((key).cast());
    
        return S_OK;
    }
    pub unsafe fn SetPreservedKeyDescription(&mut self, rguid: &GUID, pchDesc: *const WCHAR, cchDesc: ULONG) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn GetPreservedKeyDescription(&mut self, rguid: &GUID, pbstrDesc: *mut BSTR) -> HRESULT {
        let this_: *mut Self = self;
        let _ = (this_, rguid, pbstrDesc);
        return E_NOTIMPL;
    }
    pub unsafe fn SimulatePreservedKey(&mut self, pic: *mut ITfContext, rguid: &GUID, pfEaten: *mut BOOL) -> HRESULT {
        let this_: *mut Self = self;
        if pfEaten.is_null() {
            return E_INVALIDARG;
        }
        *pfEaten = (FALSE) as i32;
        return E_NOTIMPL;
    }
    pub unsafe fn PeekMessageA(&mut self, pMsg: LPMSG, hwnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT, wRemoveMsg: UINT, pfResult: *mut BOOL) -> HRESULT {
        let this_: *mut Self = self;
        if pfResult.is_null() {
            return E_INVALIDARG;
        }
        *pfResult = PeekMessageA(pMsg, hwnd, wMsgFilterMin, wMsgFilterMax, wRemoveMsg);
        return S_OK;
    }
    pub unsafe fn GetMessageA(&mut self, pMsg: LPMSG, hwnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT, pfResult: *mut BOOL) -> HRESULT {
        let this_: *mut Self = self;
        if pfResult.is_null() {
            return E_INVALIDARG;
        }
        *pfResult = GetMessageA(pMsg, hwnd, wMsgFilterMin, wMsgFilterMax);
        return S_OK;
    }
    pub unsafe fn PeekMessageW(&mut self, pMsg: LPMSG, hwnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT, wRemoveMsg: UINT, pfResult: *mut BOOL) -> HRESULT {
        let this_: *mut Self = self;
        if pfResult.is_null() {
            return E_INVALIDARG;
        }
        *pfResult = PeekMessageW(pMsg, hwnd, wMsgFilterMin, wMsgFilterMax, wRemoveMsg);
        return S_OK;
    }
    pub unsafe fn GetMessageW(&mut self, pMsg: LPMSG, hwnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT, pfResult: *mut BOOL) -> HRESULT {
        let this_: *mut Self = self;
        if pfResult.is_null() {
            return E_INVALIDARG;
        }
        *pfResult = GetMessageW(pMsg, hwnd, wMsgFilterMin, wMsgFilterMax);
        return S_OK;
    }
    pub unsafe fn GetClientId(&mut self, rclsid: &CLSID, ptid: *mut TfClientId) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT;
        let mut catmgr: *mut ITfCategoryMgr = core::ptr::null_mut();
    
        CategoryMgr_Constructor(core::ptr::null_mut(), core::ptr::addr_of_mut!(catmgr) as *mut *mut IUnknown);
        hr = (*catmgr).RegisterGUID(rclsid, ptid);
        (*catmgr).Release();
    
        return hr;
    }
    pub unsafe fn BeginUIElement(&mut self, element: *mut ITfUIElement,
        show: *mut BOOL,
        id: *mut DWORD,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB:(%p)\n", this_);
        E_NOTIMPL
    }
    pub unsafe fn UpdateUIElement(&mut self, id: DWORD,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB:(%p)\n", this_);
        E_NOTIMPL
    }
    pub unsafe fn EndUIElement(&mut self, id: DWORD,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB:(%p)\n", this_);
        E_NOTIMPL
    }
    pub unsafe fn GetUIElement(&mut self, id: DWORD,
        element: *mut *mut ITfUIElement,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB:(%p)\n", this_);
        E_NOTIMPL
    }
    pub unsafe fn EnumUIElements(&mut self, enum_elements: *mut *mut IEnumTfUIElements,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB:(%p)\n", this_);
        E_NOTIMPL
    }
    pub unsafe fn AdviseSingleSink(&mut self, tid: TfClientId, riid: &IID, punk: *mut IUnknown) -> HRESULT {
        let this_: *mut Self = self;
        let _ = (this_, tid, riid, punk);
        return E_NOTIMPL;
    }
    pub unsafe fn UnadviseSingleSink(&mut self, tid: TfClientId, riid: &IID) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn OnInitDocumentMgr(&mut self, pdim: *mut ITfDocumentMgr) -> HRESULT {
        let this_: *mut Self = self;
        let mut sink: *mut ITfThreadMgrEventSink;
        let mut cursor: *mut list;
    
        SINK_FOR_EACH!(cursor, core::ptr::addr_of_mut!((*this_).m_ThreadMgrEventSink), ITfThreadMgrEventSink, sink, {
            (*sink).OnInitDocumentMgr(pdim);
        });
    
        return S_OK;
    }
    pub unsafe fn OnUninitDocumentMgr(&mut self, pdim: *mut ITfDocumentMgr) -> HRESULT {
        let this_: *mut Self = self;
        let mut sink: *mut ITfThreadMgrEventSink;
        let mut cursor: *mut list;
    
        SINK_FOR_EACH!(cursor, core::ptr::addr_of_mut!((*this_).m_ThreadMgrEventSink), ITfThreadMgrEventSink, sink, {
            (*sink).OnUninitDocumentMgr(pdim);
        });
    
        return S_OK;
    }
    pub unsafe fn OnSetFocus(&mut self, pdimFocus: *mut ITfDocumentMgr, pdimPrevFocus: *mut ITfDocumentMgr) -> HRESULT {
        let this_: *mut Self = self;
        let mut sink: *mut ITfThreadMgrEventSink;
        let mut cursor: *mut list;
    
        SINK_FOR_EACH!(cursor, core::ptr::addr_of_mut!((*this_).m_ThreadMgrEventSink), ITfThreadMgrEventSink, sink, {
            (*sink).OnSetFocus(pdimFocus, pdimPrevFocus);
        });
    
        return S_OK;
    }
    pub unsafe fn OnPushContext(&mut self, pic: *mut ITfContext) -> HRESULT {
        let this_: *mut Self = self;
        let mut sink: *mut ITfThreadMgrEventSink = core::ptr::null_mut();
        let mut cursor: *mut list = core::ptr::null_mut();
    
        SINK_FOR_EACH!(cursor, core::ptr::addr_of_mut!((*this_).m_ThreadMgrEventSink), ITfThreadMgrEventSink, sink, {
            (*sink).OnPushContext(pic);
        });
    
        return S_OK;
    }
    pub unsafe fn OnPopContext(&mut self, pic: *mut ITfContext) -> HRESULT {
        let this_: *mut Self = self;
        let mut sink: *mut ITfThreadMgrEventSink = core::ptr::null_mut();
        let mut cursor: *mut list = core::ptr::null_mut();
    
        SINK_FOR_EACH!(cursor, core::ptr::addr_of_mut!((*this_).m_ThreadMgrEventSink), ITfThreadMgrEventSink, sink, {
            (*sink).OnPopContext(pic);
        });
    
        return S_OK;
    }
    pub unsafe extern "system" fn ThreadFocusHookProc(nCode: i32, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let This: *mut CThreadMgr = TlsGetValue(g_dwTLSIndex) as *mut CThreadMgr;
        if This.is_null() {
            return 0;
        }
        return (*This)._ThreadFocusHookProc(nCode, wParam, lParam);
    }
    pub unsafe fn _ThreadFocusHookProc(&mut self, nCode: INT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let this_: *mut Self = self;
        if (*this_).m_focusHook.is_null() {
            return 0;
        }
    
        if nCode == (HCBT_SETFOCUS as i32) { /* focus change within our thread */
            let mut cursor: *mut list;
    
            let list_head: *mut list = core::ptr::addr_of_mut!((*this_).m_AssociatedFocusWindows);
            cursor = (*list_head).next;
            while cursor != list_head {
                let wnd: *mut AssociatedWindow = LIST_ENTRY!(cursor, AssociatedWindow, entry);
                if (*wnd).hwnd == (wParam as HWND) {
                    if (*this_).m_focus != (*wnd).docmgr {
                        (*this_).SetFocus((*wnd).docmgr);
                    }
                    break;
                }
                cursor = (*cursor).next;
            }
        }
    
        return CallNextHookEx((*this_).m_focusHook, nCode, wParam, lParam);
    }
    pub unsafe fn SetupWindowsHook(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_focusHook.is_null() {
            (*this_).m_focusHook = SetWindowsHookExW((WH_CBT as i32), Some(CThreadMgr::ThreadFocusHookProc), core::ptr::null_mut(), GetCurrentThreadId());
            if (*this_).m_focusHook.is_null() {
                return E_FAIL;
            }
            return S_OK;
        }
        return S_FALSE;
    }
}

impl CEnumTfDocumentMgr {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_cRefs = 1;
        (*this_).m_index = core::ptr::null_mut();
        (*this_).m_head = core::ptr::null_mut();
        {
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        // TRACE("destroying %p\n", this_);
    }
    pub unsafe fn CreateInstance(head: *mut list, ppOut: *mut *mut CEnumTfDocumentMgr) -> HRESULT {
        let mut This: *mut CEnumTfDocumentMgr = CEnumTfDocumentMgr__new_in_0(cpp_alloc_CicNoThrow::<CEnumTfDocumentMgr>());
        if This.is_null() {
            return E_OUTOFMEMORY;
        }
    
        (*This).m_head = (head).cast();
        (*This).m_index = list_head((*This).m_head);
    
        *ppOut = This;
    
        return S_OK;
    }
    pub unsafe fn QueryInterface(&mut self, iid: &IID, ppvObject: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        *ppvObject = core::ptr::null_mut();
    
        if (IsEqualGUID!(&(*iid), &(IID_IUnknown))) || (IsEqualGUID!(&(*iid), &(IID_IEnumTfDocumentMgrs))) {
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
            CEnumTfDocumentMgr__delete(this_);
        }
        return ret;
    }
    pub unsafe fn Clone(&mut self, ppEnum: *mut *mut IEnumTfDocumentMgrs) -> HRESULT {
        let this_: *mut Self = self;
        if ppEnum.is_null() {
            return E_POINTER;
        }
    
        *ppEnum = core::ptr::null_mut();
    
        let mut cloned: *mut CEnumTfDocumentMgr = core::ptr::null_mut();
        let hr: HRESULT = CEnumTfDocumentMgr::CreateInstance(((*this_).m_head).cast(), core::ptr::addr_of_mut!(cloned));
        if hr >= 0 {
            *ppEnum = core::ptr::addr_of_mut!((*cloned).__base0) as *mut IEnumTfDocumentMgrs;
            (*cloned).m_index = (*this_).m_index;
        }
    
        return hr;
    }
    pub unsafe fn Next(&mut self, ulCount: ULONG, mut rgDocumentMgr: *mut *mut ITfDocumentMgr, pcFetched: *mut ULONG) -> HRESULT {
        let this_: *mut Self = self;
        let mut fetched: ULONG = 0;
    
        if rgDocumentMgr.is_null() {
            return E_POINTER;
        }
    
        while fetched < ulCount {
            let mut mgrentry: *mut DocumentMgrEntry;
            if (*this_).m_index.is_null() {
                break;
            }
    
            mgrentry = LIST_ENTRY!((*this_).m_index, DocumentMgrEntry, entry);
            if mgrentry.is_null() {
                break;
            }
    
            *rgDocumentMgr = (*mgrentry).docmgr;
            (**rgDocumentMgr).AddRef();
    
            (*this_).m_index = list_next((*this_).m_head, (*this_).m_index);
            fetched += 1;
            rgDocumentMgr = rgDocumentMgr.add(1);
        }
    
        if !pcFetched.is_null() {
            *pcFetched = fetched;
        }
        return if fetched == ulCount { S_OK } else { S_FALSE };
    }
    pub unsafe fn Reset(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_index = list_head((*this_).m_head);
        return S_OK;
    }
    pub unsafe fn Skip(&mut self, ulCount: ULONG) -> HRESULT {
        let this_: *mut Self = self;
        let mut i: ULONG = 0;
        while i < ulCount && !(*this_).m_index.is_null() {
            (*this_).m_index = list_next((*this_).m_head, (*this_).m_index);
            i += 1;
        }
        return S_OK;
    }
}
