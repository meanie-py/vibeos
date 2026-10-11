use crate::prelude::*;
use core::ffi::c_void;












































impl CInputContextOwner {
    pub unsafe fn __ctor_0(&mut self, fnCallback: FN_IC_OWNER_CALLBACK, pCallbackPV: LPVOID) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {
            (*this_).m_dwCookie = (-(1 as i32)) as DWORD;
            (*this_).m_fnCallback = fnCallback;
            (*this_).m_cRefs = 1 as LONG;
            (*this_).m_pCallbackPV = pCallbackPV;
        }
    }
    pub unsafe fn _Advise(&mut self, pContext: *mut IUnknown) -> HRESULT {
        let this_: *mut Self = self;
        let mut pSource: *mut ITfSource = core::ptr::null_mut();
    
        (*this_).m_pContext = core::ptr::null_mut();
    
        let mut hr: HRESULT = E_FAIL;
        if ((*(*this_).m_pContext).QueryInterface(&IID_ITfSource, core::ptr::addr_of_mut!(pSource) as *mut LPVOID) >= 0) &&
            ((*pSource).AdviseSink(&IID_ITfContextOwner,
                                          (core::ptr::addr_of_mut!((*this_).__base0) as *mut ITfContextOwner).cast(), core::ptr::addr_of_mut!((*this_).m_dwCookie)) >= 0)
        {
            (*this_).m_pContext = pContext;
            (*(*this_).m_pContext).AddRef();
            hr = S_OK;
        }
    
        if !pSource.is_null() {
            (*pSource).Release();
        }
    
        return hr;
    }
    pub unsafe fn _Unadvise(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        let mut pSource: *mut ITfSource = core::ptr::null_mut();
    
        let mut hr: HRESULT = E_FAIL;
        if !(*this_).m_pContext.is_null() {
            if ((*(*this_).m_pContext).QueryInterface(&IID_ITfSource, &mut pSource as *mut *mut ITfSource as *mut LPVOID)) >= 0
                && ((*pSource).UnadviseSink((*this_).m_dwCookie)) >= 0
            {
                hr = S_OK;
            }
        }
    
        if !(*this_).m_pContext.is_null() {
            (*(*this_).m_pContext).Release();
            (*this_).m_pContext = core::ptr::null_mut();
        }
    
        if !pSource.is_null() {
            (*pSource).Release();
        }
    
        hr
    }
    pub unsafe fn QueryInterface(&mut self, riid: &IID, ppvObj: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        let c_tab: [QITAB; 3] = [
            QITAB { piid: &IID_ITfContextOwner as *const IID, dwOffset: core::mem::offset_of!(CInputContextOwner, __base0) as DWORD },
            QITAB { piid: &IID_ITfMouseTrackerACP as *const IID, dwOffset: core::mem::offset_of!(CInputContextOwner, __base1) as DWORD },
            QITAB { piid: core::ptr::null(), dwOffset: 0 },
        ];
        return QISearch((this_).cast(), c_tab.as_ptr(), riid, ppvObj);
    }
    pub unsafe fn AddRef(&mut self) -> ULONG {
        let this_: *mut Self = self;
        (*this_).m_cRefs += 1;
        (*this_).m_cRefs as ULONG
    }
    pub unsafe fn Release(&mut self) -> ULONG {
        let this_: *mut Self = self;
        (*this_).m_cRefs -= 1;
        if (*this_).m_cRefs == 0 {
            CInputContextOwner__delete(this_);
            return 0;
        }
        return (*this_).m_cRefs as ULONG;
    }
    pub unsafe fn GetACPFromPoint(&mut self, ptScreen: *const POINT,
        dwFlags: DWORD,
        pacp: *mut LONG,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn GetTextExt(&mut self, acpStart: LONG,
        acpEnd: LONG,
        prc: *mut RECT,
        pfClipped: *mut BOOL,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn GetScreenExt(&mut self, mut prc: *mut RECT) -> HRESULT {
        let this_: *mut Self = self;
        return (((*this_).m_fnCallback).unwrap_unchecked())(2, core::ptr::addr_of_mut!(prc) as LPVOID, (*this_).m_pCallbackPV);
    }
    pub unsafe fn GetStatus(&mut self, mut pdcs: *mut TF_STATUS) -> HRESULT {
        let this_: *mut Self = self;
        return (((*this_).m_fnCallback).unwrap_unchecked())(6, core::ptr::addr_of_mut!(pdcs) as _, (*this_).m_pCallbackPV);
    }
    pub unsafe fn GetWnd(&mut self, mut phwnd: *mut HWND) -> HRESULT {
        let this_: *mut Self = self;
        return (((*this_).m_fnCallback).unwrap_unchecked())(7, core::ptr::addr_of_mut!(phwnd) as LPVOID, (*this_).m_pCallbackPV);
    }
    pub unsafe fn GetAttribute(&mut self, rguidAttribute: &GUID, pvarValue: *mut VARIANT) -> HRESULT {
        let this_: *mut Self = self;
        let _ = (this_, rguidAttribute, pvarValue);
        return E_NOTIMPL;
    }
    pub unsafe fn AdviseMouseSink(&mut self, range: *mut ITfRangeACP, pSink: *mut ITfMouseSink, pdwCookie: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        let mut args: MOUSE_SINK_ARGS = MOUSE_SINK_ARGS {
            range: range,
            pSink: pSink,
            pdwCookie: pdwCookie,
        };
        return (((*this_).m_fnCallback).unwrap_unchecked())(9, core::ptr::addr_of_mut!(args) as LPVOID, (*this_).m_pCallbackPV);
    }
    pub unsafe fn UnadviseMouseSink(&mut self, dwCookie: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        return (((*this_).m_fnCallback).unwrap_unchecked())(10, core::ptr::addr_of!(dwCookie) as LPVOID, (*this_).m_pCallbackPV);
    }
}

impl CicInputContext {
    pub unsafe fn __ctor_0(&mut self, cliendId: TfClientId, pLibThread: PCIC_LIBTHREAD, hIMC: HIMC) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {
            (*this_).m_hIMC = hIMC;
            (*this_).m_dwQueryPos = 0;
            (*this_).m_cRefs = 1;
        }
    }
    pub unsafe fn QueryInterface(&mut self, riid: &IID, ppvObj: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        let c_tab: [QITAB; 3] = [
            QITAB {
                piid: &IID_ITfCleanupContextSink as *const IID,
                dwOffset: core::mem::offset_of!(CicInputContext, __base0) as DWORD,
            },
            QITAB {
                piid: &IID_ITfContextOwnerCompositionSink as *const IID,
                dwOffset: core::mem::offset_of!(CicInputContext, __base1) as DWORD,
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
            CicInputContext__delete(this_);
            return 0;
        }
        return (*this_).m_cRefs as ULONG;
    }
    pub unsafe fn OnCleanupContext(&mut self, ecWrite: TfEditCookie, pic: *mut ITfContext) -> HRESULT {
        let this_: *mut Self = self;
        let pTLS: *mut TLS = TLS::PeekTLS();
        if pTLS.is_null() || (*pTLS).m_pProfile.is_null() {
            return E_OUTOFMEMORY;
        }
    
        let mut LangID: LANGID = 0;
        (*(*pTLS).m_pProfile).GetLangId(core::ptr::addr_of_mut!(LangID));
    
        let mut IMEInfo: IMEINFO = core::mem::zeroed();
        let mut szPath: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
        if Inquire(
            core::ptr::addr_of_mut!(IMEInfo),
            szPath.as_mut_ptr(),
            0,
            UlongToHandle!(LangID as u32) as HKL,
        ) != S_OK
        {
            return E_FAIL;
        }
    
        let mut pProp: *mut ITfProperty = core::ptr::null_mut();
        if !((IMEInfo.fdwProperty & IME_PROP_COMPLETE_ON_UNSELECT) != 0) {
            return S_OK;
        }
    
        let mut hr: HRESULT = (*pic).GetProperty(&GUID_PROP_COMPOSING, core::ptr::addr_of_mut!(pProp));
        if (hr) < 0 {
            return S_OK;
        }
    
        let mut pRanges: *mut IEnumTfRanges = core::ptr::null_mut();
        hr = (*pProp).EnumRanges(ecWrite, core::ptr::addr_of_mut!(pRanges), core::ptr::null_mut());
        if (hr) >= 0 {
            let mut pRange: *mut ITfRange = core::ptr::null_mut();
            while (*pRanges).Next(1, core::ptr::addr_of_mut!(pRange), core::ptr::null_mut()) == S_OK {
                let mut vari: VARIANT = core::mem::zeroed();
                V_VT!(core::ptr::addr_of_mut!(vari)) = VT_EMPTY as VARTYPE;
                (*pProp).GetValue(ecWrite, pRange, core::ptr::addr_of_mut!(vari));
                if V_VT!(core::ptr::addr_of_mut!(vari)) == VT_I4 as VARTYPE {
                    if V_I4!(core::ptr::addr_of_mut!(vari)) != 0 {
                        (*pProp).Clear(ecWrite, pRange);
                    }
                }
                (*pRange).Release();
                pRange = core::ptr::null_mut();
            }
            (*pRanges).Release();
        }
        (*pProp).Release();
    
        return S_OK;
    }
    pub unsafe fn OnStartComposition(&mut self, pComposition: *mut ITfCompositionView, pfOk: *mut BOOL) -> HRESULT {
        let this_: *mut Self = self;
        if ((*this_).m_cCompLocks <= 0) || (*this_).m_bReconverting != 0
        {
            *pfOk = (TRUE) as i32;
            (*this_).m_cCompLocks += 1;
        }
        else
        {
            *pfOk = (FALSE) as i32;
        }
        return S_OK;
    }
    pub unsafe fn OnUpdateComposition(&mut self, pComposition: *mut ITfCompositionView,
        pRangeNew: *mut ITfRange,
    ) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn OnEndComposition(&mut self, pComposition: *mut ITfCompositionView) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_cCompLocks -= 1;
        return S_OK;
    }
    pub unsafe fn OnCompositionTerminated(&mut self, ecWrite: TfEditCookie,
        pComposition: *mut ITfComposition,
    ) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn GetGuidAtom(&mut self, imcLock: &mut CicIMCLock, iAtom: BYTE, pdwGuidAtom: LPDWORD) -> HRESULT {
        let this_: *mut Self = self;
        let mut imeContext = CicIMCCLock__make_0::<CTFIMECONTEXT>(imcLock.get().hCompStr);
        if imeContext.m_hr < 0 {
            return imeContext.m_hr;
        }
    
        let mut hr: HRESULT = E_FAIL;
        if (iAtom as WORD) < (*this_).m_cGuidAtoms {
            *pdwGuidAtom = (*this_).m_adwGuidAtoms[iAtom as usize];
            hr = S_OK;
        }
    
        return hr;
    }
    pub unsafe fn CreateInputContext(&mut self, pThreadMgr: *mut ITfThreadMgr, imcLock: &mut CicIMCLock) -> HRESULT {
        let this_: *mut Self = self;
        let _ = (this_, pThreadMgr, imcLock);
        //FIXME
        return E_NOTIMPL;
    }
    pub unsafe fn DestroyInputContext(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        let mut pSource: *mut ITfSourceSingle = core::ptr::null_mut();
    
        if !(*this_).m_pContext.is_null() && (*(*this_).m_pContext).QueryInterface(&IID_ITfSourceSingle, &mut pSource as *mut *mut ITfSourceSingle as *mut LPVOID) == S_OK {
            (*pSource).UnadviseSingleSink((*this_).m_clientId, &IID_ITfCleanupContextSink);
        }
    
        //FIXME: m_dwUnknown5
    
        if !(*this_).m_pTextEventSink.is_null() {
            (*(*this_).m_pTextEventSink)._Unadvise();
            (*(*this_).m_pTextEventSink).Release();
            (*this_).m_pTextEventSink = core::ptr::null_mut();
        }
    
        if !(*this_).m_pCompEventSink2.is_null() {
            (*(*this_).m_pCompEventSink2)._Unadvise();
            (*(*this_).m_pCompEventSink2).Release();
            (*this_).m_pCompEventSink2 = core::ptr::null_mut();
        }
    
        if !(*this_).m_pCompEventSink1.is_null() {
            (*(*this_).m_pCompEventSink1)._Unadvise();
            (*(*this_).m_pCompEventSink1).Release();
            (*this_).m_pCompEventSink1 = core::ptr::null_mut();
        }
    
        if !(*this_).m_pInputContextOwner.is_null() {
            (*(*this_).m_pInputContextOwner)._Unadvise();
            (*(*this_).m_pInputContextOwner).Release();
            (*this_).m_pInputContextOwner = core::ptr::null_mut();
        }
    
        if !(*this_).m_pDocumentMgr.is_null() {
            (*(*this_).m_pDocumentMgr).Pop(1);
        }
    
        if !(*this_).m_pContext.is_null() {
            ClearCompartment((*this_).m_clientId, ((*this_).m_pContext).cast(), &GUID_COMPARTMENT_CTFIME_CICINPUTCONTEXT, 0);
            (*(*this_).m_pContext).Release();
            (*this_).m_pContext = core::ptr::null_mut();
        }
    
        if !(*this_).m_pContextOwnerServices.is_null() {
            (*(*this_).m_pContextOwnerServices).Release();
            (*this_).m_pContextOwnerServices = core::ptr::null_mut();
        }
    
        // FIXME: m_pICOwnerCallback
    
        if !(*this_).m_pDocumentMgr.is_null() {
            (*(*this_).m_pDocumentMgr).Release();
            (*this_).m_pDocumentMgr = core::ptr::null_mut();
        }
    
        if !pSource.is_null() {
            (*pSource).Release();
        }
    
        return S_OK;
    }
    pub unsafe fn SetCompositionString(&mut self, imcLock: &mut CicIMCLock, pThreadMgr: *mut ITfThreadMgr_P, dwIndex: DWORD, lpComp: LPCVOID, dwCompLen: DWORD, lpRead: LPCVOID, dwReadLen: DWORD, uCodePage: UINT) -> BOOL {
        let this_: *mut Self = self;
        return (FALSE) as BOOL;
    }
    pub unsafe fn SetupDocFeedString(&mut self, imcLock: &mut CicIMCLock, uCodePage: UINT) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn EscbClearDocFeedBuffer(&mut self, imcLock: &mut CicIMCLock, bFlag: BOOL) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn EscbCompComplete(&mut self, imcLock: &mut CicIMCLock) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn EscbCompCancel(&mut self, imcLock: &mut CicIMCLock) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn SetupReconvertString(&mut self, imcLock: &mut CicIMCLock, pThreadMgr: *mut ITfThreadMgr_P, uCodePage: UINT, uMsg: UINT, bUndo: BOOL) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn MsImeMouseHandler(&mut self, dwUnknown58: DWORD, dwUnknown59: DWORD, keys: UINT, imcLock: &mut CicIMCLock) -> HRESULT {
        let this_: *mut Self = self;
        let _ = (this_, dwUnknown58, dwUnknown59, keys, imcLock);
        return E_NOTIMPL;
    }
    pub unsafe fn EndReconvertString(&mut self, imcLock: &mut CicIMCLock) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn DelayedReconvertFuncCall(&mut self, imcLock: &mut CicIMCLock) -> HRESULT {
        let this_: *mut Self = self;
        let _ = (this_, imcLock);
        return E_NOTIMPL;
    }
    pub unsafe fn ClearPrevCandidatePos(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_dwUnknown8 = 0;
        ZeroMemory!(core::ptr::addr_of_mut!((*this_).m_rcCandidate1) as *mut core::ffi::c_void, core::mem::size_of::<RECT>());
        ZeroMemory!(core::ptr::addr_of_mut!((*this_).m_CandForm) as *mut core::ffi::c_void, core::mem::size_of::<CANDIDATEFORM>());
        ZeroMemory!(core::ptr::addr_of_mut!((*this_).m_rcCandidate2) as *mut core::ffi::c_void, core::mem::size_of::<RECT>());
        (*this_).m_dwQueryPos = 0;
    }
    pub unsafe fn OnSetCandidatePos(&mut self, pTLS: *mut TLS, imcLock: &mut CicIMCLock) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
}

impl MOUSE_SINK_ARGS {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
