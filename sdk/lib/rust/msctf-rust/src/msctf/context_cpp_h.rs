use crate::prelude::*;
use crate::msctf::compartmentmgr_cpp_h::CompartmentMgr_Constructor;
use crate::msctf::range_cpp_h::Range_Constructor;
use crate::msctf::range_cpp_h::TF_SELECTION_to_TS_SELECTION_ACP;
use crate::msctf::msctf_cpp_h::advise_sink;
use crate::msctf::msctf_cpp_h::free_sinks;
use crate::msctf::msctf_cpp_h::generate_Cookie;
use crate::msctf::msctf_cpp_h::get_Cookie_data;
use crate::msctf::msctf_cpp_h::get_Cookie_magic;
use crate::msctf::msctf_cpp_h::list;
use crate::msctf::msctf_cpp_h::remove_Cookie;
use crate::msctf::msctf_cpp_h::unadvise_sink;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct EditCookie {
    pub lockType: DWORD,
    pub pOwningContext: *mut CContext,
}

pub type PEditCookie = *mut EditCookie;






































































#[no_mangle]
pub unsafe extern "C" fn Context_Constructor(tidOwner: TfClientId, punk: *mut IUnknown, mgr: *mut ITfDocumentMgr, ppOut: *mut *mut ITfContext, pecTextStore: *mut TfEditCookie) -> HRESULT {
    return CContext::CreateInstance(tidOwner, punk, mgr, ppOut, pecTextStore);
}
#[no_mangle]
pub unsafe extern "C" fn Context_Initialize(iface: *mut ITfContext, manager: *mut ITfDocumentMgr) -> HRESULT {
    let This: *mut CContext = iface as *mut CContext;
    return (*This).Initialize(manager);
}
#[no_mangle]
pub unsafe extern "C" fn Context_Uninitialize(iface: *mut ITfContext) -> HRESULT {
    let This: *mut CContext = (iface as *mut u8).sub(core::mem::offset_of!(CContext, __base0)) as *mut CContext;
    return (*This).Uninitialize();
}

impl CContext {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_cRefs = 1;
        (*this_).m_connected = (FALSE) as i32;
        (*this_).m_CompartmentMgr = core::ptr::null_mut();
        (*this_).m_manager = core::ptr::null_mut();
        (*this_).m_pITextStoreACP = core::ptr::null_mut();
        (*this_).m_pITfContextOwnerCompositionSink = core::ptr::null_mut();
        (*this_).m_currentEditSession = core::ptr::null_mut();
        {
            list_init(core::ptr::addr_of_mut!((*this_).m_pContextKeyEventSink));
            list_init(core::ptr::addr_of_mut!((*this_).m_pEditTransactionSink));
            list_init(core::ptr::addr_of_mut!((*this_).m_pStatusSink));
            list_init(core::ptr::addr_of_mut!((*this_).m_pTextEditSink));
            list_init(core::ptr::addr_of_mut!((*this_).m_pTextLayoutSink));
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        let mut cookie: *mut EditCookie;
    
        if !(*this_).m_pITextStoreACP.is_null() {
            (*(*this_).m_pITextStoreACP).Release();
        }
    
        if !(*this_).m_pITfContextOwnerCompositionSink.is_null() {
            (*(*this_).m_pITfContextOwnerCompositionSink).Release();
        }
    
        if (*this_).m_defaultCookie != 0 {
            cookie = remove_Cookie((*this_).m_defaultCookie) as *mut EditCookie;
            cicMemFree((cookie).cast());
            (*this_).m_defaultCookie = 0;
        }
    
        free_sinks((core::ptr::addr_of_mut!((*this_).m_pContextKeyEventSink)).cast());
        free_sinks((core::ptr::addr_of_mut!((*this_).m_pEditTransactionSink)).cast());
        free_sinks((core::ptr::addr_of_mut!((*this_).m_pStatusSink)).cast());
        free_sinks((core::ptr::addr_of_mut!((*this_).m_pTextEditSink)).cast());
        free_sinks((core::ptr::addr_of_mut!((*this_).m_pTextLayoutSink)).cast());
    
        if !(*this_).m_CompartmentMgr.is_null() {
            (*(*this_).m_CompartmentMgr).Release();
        }
    }
    pub unsafe fn CreateInstance(tidOwner: TfClientId, punk: *mut IUnknown, mgr: *mut ITfDocumentMgr, ppOut: *mut *mut ITfContext, pecTextStore: *mut TfEditCookie) -> HRESULT {
        let This: *mut CContext = CContext__new_in_0(cpp_alloc_CicNoThrow::<CContext>());
        if This.is_null() {
            return E_OUTOFMEMORY;
        }
    
        let cookie: *mut EditCookie = cicMemAlloc(core::mem::size_of::<EditCookie>()) as *mut EditCookie;
        if cookie.is_null() {
            CContext__delete(This);
            return E_OUTOFMEMORY;
        }
    
        (*This).m_tidOwner = tidOwner;
        (*This).m_manager = mgr;
    
        CompartmentMgr_Constructor((core::ptr::addr_of_mut!((*This).__base0)).cast(), &IID_IUnknown, core::ptr::addr_of_mut!((*This).m_CompartmentMgr) as *mut *mut IUnknown);
    
        (*cookie).lockType = TF_ES_READ;
        (*cookie).pOwningContext = This;
    
        if !punk.is_null() {
            (*punk).QueryInterface(&IID_ITextStoreACP, core::ptr::addr_of_mut!((*This).m_pITextStoreACP) as *mut *mut core::ffi::c_void);
            (*punk).QueryInterface(&IID_ITfContextOwnerCompositionSink, core::ptr::addr_of_mut!((*This).m_pITfContextOwnerCompositionSink) as *mut *mut core::ffi::c_void);
    
            if (*This).m_pITextStoreACP.is_null() && (*This).m_pITfContextOwnerCompositionSink.is_null() {
                FIXME!("Unhandled pUnk\n");
            }
        }
    
        (*This).m_defaultCookie = generate_Cookie(COOKIE_MAGIC_EDITCOOKIE, (cookie).cast());
        *pecTextStore = (*This).m_defaultCookie;
    
        list_init(core::ptr::addr_of_mut!((*This).m_pContextKeyEventSink));
        list_init(core::ptr::addr_of_mut!((*This).m_pEditTransactionSink));
        list_init(core::ptr::addr_of_mut!((*This).m_pStatusSink));
        list_init(core::ptr::addr_of_mut!((*This).m_pTextEditSink));
        list_init(core::ptr::addr_of_mut!((*This).m_pTextLayoutSink));
    
        *ppOut = core::ptr::addr_of_mut!((*This).__base0);
    
        return S_OK;
    }
    pub unsafe fn Initialize(&mut self, manager: *mut ITfDocumentMgr) -> HRESULT {
        let this_: *mut Self = self;
        if !(*this_).m_pITextStoreACP.is_null() {
            (*(*this_).m_pITextStoreACP).AdviseSink(
                &IID_ITextStoreACPSink,
                (core::ptr::addr_of_mut!((*this_).__base5) as *mut ITextStoreACPSink).cast(),
                TS_AS_ALL_SINKS,
            );
        }
        (*this_).m_connected = (TRUE) as i32;
        (*this_).m_manager = manager;
        return S_OK;
    }
    pub unsafe fn Uninitialize(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        if !(*this_).m_pITextStoreACP.is_null() {
            (*(*this_).m_pITextStoreACP).UnadviseSink(
                (core::ptr::addr_of_mut!((*this_).__base5) as *mut ITextStoreACPSink).cast(),
            );
        }
        (*this_).m_connected = (FALSE) as i32;
        (*this_).m_manager = core::ptr::null_mut();
        return S_OK;
    }
    pub unsafe fn QueryInterface(&mut self, riid: &IID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT {
        let this_: *mut Self = self;
        *ppvObj = core::ptr::null_mut();
    
        let mut pUnk: *mut IUnknown = core::ptr::null_mut();
        if IsEqualGUID!(&(*riid), &(IID_IUnknown)) || IsEqualGUID!(&(*riid), &(IID_ITfContext)) {
            pUnk = core::ptr::addr_of_mut!((*this_).__base0) as *mut IUnknown;
        } else if IsEqualGUID!(&(*riid), &(IID_ITfSource)) {
            pUnk = core::ptr::addr_of_mut!((*this_).__base1) as *mut IUnknown;
        } else if IsEqualGUID!(&(*riid), &(IID_ITfContextOwnerCompositionServices)) {
            pUnk = core::ptr::addr_of_mut!((*this_).__base2) as *mut IUnknown;
        } else if IsEqualGUID!(&(*riid), &(IID_ITfInsertAtSelection)) {
            pUnk = core::ptr::addr_of_mut!((*this_).__base3) as *mut IUnknown;
        } else if IsEqualGUID!(&(*riid), &(IID_ITfCompartmentMgr)) {
            pUnk = (*this_).m_CompartmentMgr as *mut IUnknown;
        } else if IsEqualGUID!(&(*riid), &(IID_ITfSourceSingle)) {
            pUnk = core::ptr::addr_of_mut!((*this_).__base4) as *mut IUnknown;
        } else if IsEqualGUID!(&(*riid), &(IID_ITextStoreACPSink)) {
            pUnk = core::ptr::addr_of_mut!((*this_).__base5) as *mut IUnknown;
        } else if IsEqualGUID!(&(*riid), &(IID_ITextStoreACPServices)) {
            pUnk = core::ptr::addr_of_mut!((*this_).__base6) as *mut IUnknown;
        }
    
        if !pUnk.is_null() {
            (*pUnk).AddRef();
            *ppvObj = pUnk as *mut core::ffi::c_void;
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
        let ret: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) as ULONG;
        if ret == 0 {
            CContext__delete(this_);
        }
        return ret;
    }
    pub unsafe fn RequestEditSession(&mut self, tid: TfClientId, pes: *mut ITfEditSession, dwFlags: DWORD, phrSession: *mut HRESULT) -> HRESULT {
        let this_: *mut Self = self;
        let hr: HRESULT;
        let mut dwLockFlags: DWORD = 0x0;
    
        if (dwFlags & TF_ES_READ) == 0 && (dwFlags & TF_ES_READWRITE) == 0
        {
            *phrSession = E_FAIL;
            return E_INVALIDARG;
        }
    
        if (*this_).m_pITextStoreACP.is_null()
        {
            *phrSession = E_FAIL;
            return E_FAIL;
        }
    
        if (dwFlags & TF_ES_ASYNC) == 0
            { dwLockFlags |= TS_LF_SYNC; }
    
        if (dwFlags & TF_ES_READWRITE) == TF_ES_READWRITE
            { dwLockFlags |= TS_LF_READWRITE; }
        else if (dwFlags & TF_ES_READ) != 0
            { dwLockFlags |= TS_LF_READ; }
    
        if (*this_).m_documentStatus.dwDynamicFlags == 0
            { (*(*this_).m_pITextStoreACP).GetStatus(core::ptr::addr_of_mut!((*this_).m_documentStatus)); }
    
        if ((dwFlags & TF_ES_READWRITE) == TF_ES_READWRITE) &&
            ((*this_).m_documentStatus.dwDynamicFlags & TS_SD_READONLY) != 0
        {
            *phrSession = TS_E_READONLY;
            return S_OK;
        }
    
        hr = (*pes).QueryInterface(&IID_ITfEditSession, core::ptr::addr_of_mut!((*this_).m_currentEditSession) as *mut *mut core::ffi::c_void);
        if hr < 0
        {
            *phrSession = E_FAIL;
            return E_INVALIDARG;
        }
    
        return (*(*this_).m_pITextStoreACP).RequestLock(dwLockFlags, phrSession);
    }
    pub unsafe fn InWriteSession(&mut self, tid: TfClientId,
        pfWriteSession: *mut BOOL,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB:(%p)\n", this_);
        E_NOTIMPL
    }
    pub unsafe fn GetSelection(&mut self, ec: TfEditCookie, ulIndex: ULONG, ulCount: ULONG, pSelection: *mut TF_SELECTION, pcFetched: *mut ULONG) -> HRESULT {
        let this_: *mut Self = self;
        let mut count: ULONG;
        let mut i: ULONG;
        let mut totalFetched: ULONG = 0;
        let mut hr: HRESULT = S_OK;
    
        if pSelection.is_null() || pcFetched.is_null() {
            return E_INVALIDARG;
        }
    
        *pcFetched = 0;
    
        if (*this_).m_connected == 0 {
            return TF_E_DISCONNECTED;
        }
    
        if get_Cookie_magic(ec) != COOKIE_MAGIC_EDITCOOKIE {
            return TF_E_NOLOCK;
        }
    
        if (*this_).m_pITextStoreACP.is_null() {
    
            return E_NOTIMPL;
        }
    
        count = if ulIndex == (TF_DEFAULT_SELECTION as ULONG) { 1 } else { ulCount };
    
        i = 0;
        while i < count {
            let mut fetched: DWORD = 0;
            let mut acps: TS_SELECTION_ACP = core::mem::zeroed();
    
            hr = (*(*this_).m_pITextStoreACP).GetSelection(ulIndex + i, 1, core::ptr::addr_of_mut!(acps), core::ptr::addr_of_mut!(fetched));
            if hr == TS_E_NOLOCK {
                return TF_E_NOLOCK;
            } else if (hr < 0) {
                break;
            }
    
            (*pSelection.add(totalFetched as usize)).style.ase = acps.style.ase as TfActiveSelEnd;
            (*pSelection.add(totalFetched as usize)).style.fInterimChar = acps.style.fInterimChar;
            Range_Constructor((this_).cast(), (acps.acpStart) as DWORD, (acps.acpEnd) as DWORD, core::ptr::addr_of_mut!((*pSelection.add(totalFetched as usize)).range));
            totalFetched += 1;
            i += 1;
        }
    
        *pcFetched = totalFetched;
        return hr;
    }
    pub unsafe fn SetSelection(&mut self, ec: TfEditCookie, ulCount: ULONG, pSelection: *const TF_SELECTION) -> HRESULT {
        let this_: *mut Self = self;
        let mut acp: *mut TS_SELECTION_ACP;
        let mut i: ULONG;
        let mut hr: HRESULT;
    
        if (*this_).m_pITextStoreACP.is_null() {
            return E_NOTIMPL;
        }
    
        if get_Cookie_magic(ec) != COOKIE_MAGIC_EDITCOOKIE {
            return TF_E_NOLOCK;
        }
    
        acp = cicMemAlloc(((core::mem::size_of::<TS_SELECTION_ACP>() as ULONG) * ulCount) as _) as *mut TS_SELECTION_ACP;
        if acp.is_null() {
            return E_OUTOFMEMORY;
        }
    
        i = 0;
        while i < ulCount {
            if TF_SELECTION_to_TS_SELECTION_ACP(pSelection.add(i as usize), acp.add(i as usize)) < 0 {
                cicMemFree((acp).cast());
                return E_FAIL;
            }
            i += 1;
        }
    
        hr = (*(*this_).m_pITextStoreACP).SetSelection(ulCount, acp);
    
        cicMemFree((acp).cast());
    
        return hr;
    }
    pub unsafe fn GetStart(&mut self, ec: TfEditCookie, ppStart: *mut *mut ITfRange) -> HRESULT {
        let this_: *mut Self = self;
        if ppStart.is_null() {
            return E_INVALIDARG;
        }
    
        *ppStart = core::ptr::null_mut();
    
        if (*this_).m_connected == 0 {
            return TF_E_DISCONNECTED;
        }
    
        if get_Cookie_magic(ec) != COOKIE_MAGIC_EDITCOOKIE {
            return TF_E_NOLOCK;
        }
    
        return Range_Constructor((this_).cast(), 0, 0, ppStart);
    }
    pub unsafe fn GetEnd(&mut self, ec: TfEditCookie, ppEnd: *mut *mut ITfRange) -> HRESULT {
        let this_: *mut Self = self;
        let mut end: LONG = 0;
    
        if ppEnd.is_null() {
            return E_INVALIDARG;
        }
    
        *ppEnd = core::ptr::null_mut();
    
        if (*this_).m_connected == 0 {
            return TF_E_DISCONNECTED;
        }
    
        if get_Cookie_magic(ec) != COOKIE_MAGIC_EDITCOOKIE {
            return TF_E_NOLOCK;
        }
    
        if (*this_).m_pITextStoreACP.is_null() {
            return E_NOTIMPL;
        }
    
        (*(*this_).m_pITextStoreACP).GetEndACP(core::ptr::addr_of_mut!(end));
    
        return Range_Constructor((this_).cast(), (end) as DWORD, (end) as DWORD, ppEnd);
    }
    pub unsafe fn GetActiveView(&mut self, ppView: *mut *mut ITfContextView,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB:(%p)\n", this_);
        E_NOTIMPL
    }
    pub unsafe fn EnumViews(&mut self, ppEnum: *mut *mut IEnumTfContextViews,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB:(%p)\n", this_);
        E_NOTIMPL
    }
    pub unsafe fn GetStatus(&mut self, pdcs: *mut TF_STATUS) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_connected == 0 {
            return TF_E_DISCONNECTED;
        }
    
        if pdcs.is_null() {
            return E_INVALIDARG;
        }
    
        if (*this_).m_pITextStoreACP.is_null() {
            return E_NOTIMPL;
        }
    
        (*(*this_).m_pITextStoreACP).GetStatus(core::ptr::addr_of_mut!((*this_).m_documentStatus));
    
        *pdcs = (*this_).m_documentStatus;
    
        return S_OK;
    }
    pub unsafe fn GetProperty(&mut self, guidProp: &GUID, ppProp: *mut *mut ITfProperty) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn GetAppProperty(&mut self, guidProp: &GUID, ppProp: *mut *mut ITfReadOnlyProperty) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn TrackProperties(&mut self, prgProp: *mut *const GUID,
        cProp: ULONG,
        prgAppProp: *mut *const GUID,
        cAppProp: ULONG,
        ppProperty: *mut *mut ITfReadOnlyProperty,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB:(%p)\n", this_);
        E_NOTIMPL
    }
    pub unsafe fn EnumProperties(&mut self, ppEnum: *mut *mut IEnumTfProperties,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB:(%p)\n", this_);
        E_NOTIMPL
    }
    pub unsafe fn GetDocumentMgr(&mut self, ppDm: *mut *mut ITfDocumentMgr) -> HRESULT {
        let this_: *mut Self = self;
        if ppDm.is_null() {
            return E_INVALIDARG;
        }
    
        *ppDm = (*this_).m_manager;
        if (*this_).m_manager.is_null() {
            return S_FALSE;
        }
    
        (*(*this_).m_manager).AddRef();
        return S_OK;
    }
    pub unsafe fn CreateRangeBackup(&mut self, ec: TfEditCookie,
        pRange: *mut ITfRange,
        ppBackup: *mut *mut ITfRangeBackup,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB:(%p)\n", this_);
        E_NOTIMPL
    }
    pub unsafe fn AdviseSink(&mut self, riid: &IID, punk: *mut IUnknown, pdwCookie: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if cicIsNullPtr(riid as *const IID) || punk.is_null() || pdwCookie.is_null() {
            return E_INVALIDARG;
        }
    
        if IsEqualGUID!(&(*riid), &(IID_ITfTextEditSink)) {
            return advise_sink((core::ptr::addr_of_mut!((*this_).m_pTextEditSink)).cast(), &IID_ITfTextEditSink, COOKIE_MAGIC_CONTEXTSINK, punk, pdwCookie);
        }
    
        return E_NOTIMPL;
    }
    pub unsafe fn UnadviseSink(&mut self, dwCookie: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if get_Cookie_magic(dwCookie) != COOKIE_MAGIC_CONTEXTSINK {
            return E_INVALIDARG;
        }
    
        return unadvise_sink(dwCookie);
    }
    pub unsafe fn StartComposition(&mut self, ecWrite: TfEditCookie,
        pCompositionRange: *mut ITfRange,
        pSink: *mut ITfCompositionSink,
        ppComposition: *mut *mut ITfComposition,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB:(%p) %#x %p %p %p\n", this_, ecWrite, pCompositionRange, pSink, ppComposition);
        E_NOTIMPL
    }
    pub unsafe fn EnumCompositions(&mut self, ppEnum: *mut *mut IEnumITfCompositionView,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB:(%p) %p\n", this_, ppEnum);
        E_NOTIMPL
    }
    pub unsafe fn FindComposition(&mut self, ecRead: TfEditCookie,
        pTestRange: *mut ITfRange,
        ppEnum: *mut *mut IEnumITfCompositionView,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB:(%p) %#x %p %p\n", this_, ecRead, pTestRange, ppEnum);
        E_NOTIMPL
    }
    pub unsafe fn TakeOwnership(&mut self, ecWrite: TfEditCookie,
        pComposition: *mut ITfCompositionView,
        pSink: *mut ITfCompositionSink,
        ppComposition: *mut *mut ITfComposition,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB:(%p) %#x %p %p %p\n", this_, ecWrite, pComposition, pSink, ppComposition);
        E_NOTIMPL
    }
    pub unsafe fn TerminateComposition(&mut self, pComposition: *mut ITfCompositionView,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB:(%p) %p\n", this_, pComposition);
        E_NOTIMPL
    }
    pub unsafe fn InsertTextAtSelection(&mut self, ec: TfEditCookie, dwFlags: DWORD, pchText: *const WCHAR, cch: LONG, ppRange: *mut *mut ITfRange) -> HRESULT {
        let this_: *mut Self = self;
        let mut cookie: *mut EditCookie;
        let mut acpStart: LONG = 0;
        let mut acpEnd: LONG = 0;
        let mut change: TS_TEXTCHANGE = core::mem::zeroed();
        let mut hr: HRESULT;
    
        if (*this_).m_connected == 0 {
            return TF_E_DISCONNECTED;
        }
    
        if get_Cookie_magic(ec) != COOKIE_MAGIC_EDITCOOKIE {
            return TF_E_NOLOCK;
        }
    
        cookie = get_Cookie_data(ec) as *mut EditCookie;
    
        if ((*cookie).lockType & TS_LF_READWRITE) != TS_LF_READWRITE {
            return TS_E_READONLY;
        }
    
        if (*this_).m_pITextStoreACP.is_null() {
            return E_NOTIMPL;
        }
    
        hr = (*(*this_).m_pITextStoreACP).InsertTextAtSelection(
            dwFlags,
            pchText,
            (cch) as u32,
            core::ptr::addr_of_mut!(acpStart),
            core::ptr::addr_of_mut!(acpEnd),
            core::ptr::addr_of_mut!(change),
        );
        if hr >= 0 {
            Range_Constructor((this_).cast(), (change.acpStart) as DWORD, (change.acpNewEnd) as DWORD, ppRange);
        }
    
        return hr;
    }
    pub unsafe fn InsertEmbeddedAtSelection(&mut self, ec: TfEditCookie,
        dwFlags: DWORD,
        pDataObject: *mut IDataObject,
        ppRange: *mut *mut ITfRange,
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
    pub unsafe fn OnTextChange(&mut self, dwFlags: DWORD,
        pChange: *const TS_TEXTCHANGE,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB:(%p)\n", this_);
        S_OK
    }
    pub unsafe fn OnSelectionChange(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB:(%p)\n", this_);
        S_OK
    }
    pub unsafe fn OnLayoutChange(&mut self, lcode: TsLayoutCode,
        vcView: TsViewCookie,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB:(%p)\n", this_);
        S_OK
    }
    pub unsafe fn OnStatusChange(&mut self, dwFlags: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT = 0;
        let mut hrSession: HRESULT = 0;
    
        if (*this_).m_pITextStoreACP.is_null() {
            return E_NOTIMPL;
        }
    
        hr = (*(*this_).m_pITextStoreACP).RequestLock(TS_LF_READ, core::ptr::addr_of_mut!(hrSession));
    
        if ((hr) >= 0) && ((hrSession) >= 0) {
            (*this_).m_documentStatus.dwDynamicFlags = dwFlags;
        }
    
        return S_OK;
    }
    pub unsafe fn OnAttrsChange(&mut self, acpStart: LONG,
        acpEnd: LONG,
        cAttrs: ULONG,
        paAttrs: *const TS_ATTRID,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB:(%p)\n", this_);
        E_NOTIMPL
    }
    pub unsafe fn OnLockGranted(&mut self, dwLockFlags: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT;
        let mut cookie: *mut EditCookie;
        let mut sinkcookie: *mut EditCookie;
        let mut ec: TfEditCookie;
        let mut cursor: *mut list = core::ptr::null_mut();
    
        if (*this_).m_currentEditSession.is_null() {
            return S_OK;
        }
    
        cookie = cicMemAlloc(core::mem::size_of::<EditCookie>()) as *mut EditCookie;
        if cookie.is_null() {
            return E_OUTOFMEMORY;
        }
    
        sinkcookie = cicMemAlloc(core::mem::size_of::<EditCookie>()) as *mut EditCookie;
        if sinkcookie.is_null() {
            cicMemFree(cookie as *mut core::ffi::c_void);
            return E_OUTOFMEMORY;
        }
    
        (*cookie).lockType = dwLockFlags;
        (*cookie).pOwningContext = this_;
        ec = generate_Cookie(COOKIE_MAGIC_EDITCOOKIE, (cookie).cast());
    
        hr = (*(*this_).m_currentEditSession).DoEditSession(ec);
    
        if (dwLockFlags & TS_LF_READWRITE) == TS_LF_READWRITE {
            let mut sink: *mut ITfTextEditSink = core::ptr::null_mut();
            let mut sc: TfEditCookie;
    
            (*sinkcookie).lockType = TS_LF_READ;
            (*sinkcookie).pOwningContext = this_;
            sc = generate_Cookie(COOKIE_MAGIC_EDITCOOKIE, (sinkcookie).cast());
    
            /*TODO: implement ITfEditRecord */
            SINK_FOR_EACH!(cursor, core::ptr::addr_of_mut!((*this_).m_pTextEditSink), ITfTextEditSink, sink, {
                (*sink).OnEndEdit(core::ptr::addr_of_mut!((*this_).__base0) as *mut ITfContext, sc, core::ptr::null_mut());
            });
            sinkcookie = remove_Cookie(sc) as *mut EditCookie;
        }
        cicMemFree(sinkcookie as *mut core::ffi::c_void);
    
        (*(*this_).m_currentEditSession).Release();
        (*this_).m_currentEditSession = core::ptr::null_mut();
    
        /* Edit Cookie is only valid during the edit session */
        cookie = remove_Cookie(ec) as *mut EditCookie;
        cicMemFree(cookie as *mut core::ffi::c_void);
    
        hr
    }
    pub unsafe fn OnStartEditTransaction(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB:(%p)\n", this_);
        E_NOTIMPL
    }
    pub unsafe fn OnEndEditTransaction(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB:(%p)\n", this_);
        E_NOTIMPL
    }
    pub unsafe fn Serialize(&mut self, prop: *mut ITfProperty,
        range: *mut ITfRange,
        header: *mut TF_PERSISTENT_PROPERTY_HEADER_ACP,
        stream: *mut IStream,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("stub: %p %p %p %p %p\n", this_, prop, range, header, stream);
        E_NOTIMPL
    }
    pub unsafe fn Unserialize(&mut self, prop: *mut ITfProperty,
        header: *const TF_PERSISTENT_PROPERTY_HEADER_ACP,
        stream: *mut IStream,
        loader: *mut ITfPersistentPropertyLoaderACP,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("stub: %p %p %p %p %p\n", this_, prop, header, stream, loader);
        E_NOTIMPL
    }
    pub unsafe fn ForceLoadProperty(&mut self, prop: *mut ITfProperty,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("stub: %p %p\n", this_, prop);
        E_NOTIMPL
    }
    pub unsafe fn CreateRange(&mut self, start: LONG,
        end: LONG,
        range: *mut *mut ITfRangeACP,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("stub: %p %d %d %p\n", this_, start, end, range);
        S_OK
    }
}

impl tagEditCookie {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
