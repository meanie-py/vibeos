use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub type PENUM_CREATE_DESTROY_IC = *mut ENUM_CREATE_DESTROY_IC;

pub static mut __wine_dbch_msctfime: __wine_debug_channel = __wine_debug_channel {
    flags: 0,
    name: [b'm' as i8, b's' as i8, b'c' as i8, b't' as i8, b'f' as i8, b'i' as i8, b'm' as i8, b'e' as i8, 0, 0, 0, 0, 0, 0, 0],
};












































impl CicBridge {
    pub unsafe extern "system" fn EnumCreateInputContextCallback(hIMC: HIMC, lParam: LPARAM) -> BOOL {
        let pData: *mut ENUM_CREATE_DESTROY_IC = lParam as *mut ENUM_CREATE_DESTROY_IC;
        (*(*pData).m_pBridge).CreateInputContext((*pData).m_pTLS, hIMC);
        return (TRUE) as BOOL;
    }
    pub unsafe extern "system" fn EnumDestroyInputContextCallback(hIMC: HIMC, lParam: LPARAM) -> BOOL {
        let pData: PENUM_CREATE_DESTROY_IC = lParam as PENUM_CREATE_DESTROY_IC;
        (*(*pData).m_pBridge).DestroyInputContext((*pData).m_pTLS, hIMC);
        return (TRUE) as BOOL;
    }
    pub unsafe fn EscHanjaMode(&mut self, pTLS: *mut TLS, hIMC: HIMC, lpData: LPVOID) -> LRESULT {
        let this_: *mut Self = self;
        let mut imcLock = CicIMCLock__zeroed();
        imcLock.__ctor_0((hIMC).cast());
        if ((imcLock.m_hr) < 0) {
            return imcLock.m_hr as LRESULT;
        }
    
        let mut imeContext: CicIMCCLock<CTFIMECONTEXT> = CicIMCCLock__zeroed();
        imeContext.__ctor_0((imcLock.get()).hCtfImeContext);
        if ((imeContext.m_hr) < 0) {
            return imeContext.m_hr as LRESULT;
        }
    
        let mut pCicIC: *mut CicInputContext = (imeContext.get()).m_pCicIC;
        if pCicIC.is_null() {
            return (TRUE) as LRESULT;
        }
    
        if (*pCicIC).m_bCandidateOpen != 0 {
            return (TRUE) as LRESULT;
        }
    
        (*pCicIC).m_dwUnknown6_5[(4) as usize] |= 0x1;
    
        //FIXME
    
        (*pCicIC).m_dwUnknown6_5[(4) as usize] &= !(0x1);
    
        return (TRUE) as LRESULT;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_bImmxInited = (FALSE) as i32;
        (*this_).m_bUnknown1 = (FALSE) as i32;
        (*this_).m_bDeactivating = (FALSE) as i32;
        (*this_).m_bUnknown2 = (FALSE) as i32;
        (*this_).m_pKeystrokeMgr = core::ptr::null_mut();
        (*this_).m_pDocMgr = core::ptr::null_mut();
        (*this_).m_pThreadMgrEventSink = core::ptr::null_mut();
        (*this_).m_cliendId = 0;
        (*this_).m_cRefs = 1;
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        let pTLS: *mut TLS = TLS::PeekTLS();
        if pTLS.is_null() || (*pTLS).m_pThreadMgr.is_null() {
            return;
        }
    
        if (*this_).DeactivateIMMX(pTLS, (*pTLS).m_pThreadMgr) >= 0 {
            (*this_).UnInitIMMX(pTLS);
        }
    }
    pub unsafe fn QueryInterface(&mut self, riid: &IID, ppvObj: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        let c_tab: [QITAB; 2] = [
            QITAB {
                piid: core::ptr::addr_of!(IID_ITfSysHookSink),
                dwOffset: core::mem::offset_of!(CicBridge, __base0) as DWORD,
            },
            QITAB {
                piid: core::ptr::null(),
                dwOffset: 0,
            },
        ];
        return QISearch(this_ as *mut core::ffi::c_void, c_tab.as_ptr(), riid, ppvObj);
    }
    pub unsafe fn AddRef(&mut self) -> ULONG {
        let this_: *mut Self = self;
        return InterlockedIncrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) as ULONG;
    }
    pub unsafe fn Release(&mut self) -> ULONG {
        let this_: *mut Self = self;
        if InterlockedDecrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) == 0 {
            CicBridge__delete(this_);
            return 0 as ULONG;
        }
        return (*this_).m_cRefs as ULONG;
    }
    pub unsafe fn OnPreFocusDIM(&mut self, hwnd: HWND,
    ) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn OnSysKeyboardProc(&mut self, arg0: UINT,
        arg1: LONG,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn OnSysShellProc(&mut self, arg0: INT,
        arg1: UINT,
        arg2: LONG,
    ) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn InitIMMX(&mut self, pTLS: *mut TLS) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_bImmxInited != 0 {
            return S_OK;
        }
    
        let mut hr: HRESULT = S_OK;
        if (*pTLS).m_pThreadMgr.is_null() {
            let mut pThreadMgr: *mut ITfThreadMgr = core::ptr::null_mut();
            hr = TF_CreateThreadMgr(core::ptr::addr_of_mut!(pThreadMgr));
            if hr < 0 {
                return E_FAIL;
            }
    
            hr = (*pThreadMgr).QueryInterface(
                &IID_ITfThreadMgr_P,
                core::ptr::addr_of_mut!((*pTLS).m_pThreadMgr) as *mut LPVOID,
            );
            if !pThreadMgr.is_null() {
                (*pThreadMgr).Release();
            }
            if hr < 0 {
                return E_FAIL;
            }
        }
    
        if (*this_).m_pThreadMgrEventSink.is_null() {
            (*this_).m_pThreadMgrEventSink = CThreadMgrEventSink__new_in_0(
                cpp_alloc_CicNoThrow::<CThreadMgrEventSink>(),
                Some(CThreadMgrEventSink::DIMCallback),
                None,
                core::ptr::null_mut(),
            );
            if (*this_).m_pThreadMgrEventSink.is_null() {
                (*this_).UnInitIMMX(pTLS);
                return E_FAIL;
            }
        }
    
        (*(*this_).m_pThreadMgrEventSink).SetCallbackPV((*this_).m_pThreadMgrEventSink as LPVOID);
        (*(*this_).m_pThreadMgrEventSink)._Advise(((*pTLS).m_pThreadMgr).cast());
    
        if (*pTLS).m_pProfile.is_null() {
            (*pTLS).m_pProfile = CicProfile__new_in_0(cpp_alloc_CicNoThrow::<CicProfile>());
            if (*pTLS).m_pProfile.is_null() {
                return E_OUTOFMEMORY;
            }
    
            hr = (*(*pTLS).m_pProfile).InitProfileInstance(pTLS);
            if hr < 0 {
                (*this_).UnInitIMMX(pTLS);
                return E_FAIL;
            }
        }
    
        hr = (*(*pTLS).m_pThreadMgr).QueryInterface(
            &IID_ITfKeystrokeMgr_P,
            core::ptr::addr_of_mut!((*this_).m_pKeystrokeMgr) as *mut LPVOID,
        );
        if hr < 0 {
            (*this_).UnInitIMMX(pTLS);
            return E_FAIL;
        }
    
        hr = InitDisplayAttrbuteLib(core::ptr::addr_of_mut!((*this_).m_LibThread));
        if hr < 0 {
            (*this_).UnInitIMMX(pTLS);
            return E_FAIL;
        }
    
        (*this_).m_bImmxInited = (TRUE) as i32;
        return S_OK;
    }
    pub unsafe fn UnInitIMMX(&mut self, pTLS: *mut TLS) -> BOOL {
        let this_: *mut Self = self;
        UninitDisplayAttrbuteLib(core::ptr::addr_of_mut!((*this_).m_LibThread));
        TFUninitLib_Thread(core::ptr::addr_of_mut!((*this_).m_LibThread));
    
        if !(*this_).m_pKeystrokeMgr.is_null() {
            (*(*this_).m_pKeystrokeMgr).Release();
            (*this_).m_pKeystrokeMgr = core::ptr::null_mut();
        }
    
        if !(*pTLS).m_pProfile.is_null() {
            (*(*pTLS).m_pProfile).Release();
            (*pTLS).m_pProfile = core::ptr::null_mut();
        }
    
        if !(*this_).m_pThreadMgrEventSink.is_null() {
            (*(*this_).m_pThreadMgrEventSink)._Unadvise();
            (*(*this_).m_pThreadMgrEventSink).Release();
            (*this_).m_pThreadMgrEventSink = core::ptr::null_mut();
        }
    
        if !(*pTLS).m_pThreadMgr.is_null() {
            (*(*pTLS).m_pThreadMgr).Release();
            (*pTLS).m_pThreadMgr = core::ptr::null_mut();
        }
    
        (*this_).m_bImmxInited = (FALSE) as i32;
        return (TRUE) as BOOL;
    }
    pub unsafe fn ActivateIMMX(&mut self, pTLS: *mut TLS, pThreadMgr: *mut ITfThreadMgr_P) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT = (*pThreadMgr).ActivateEx(core::ptr::addr_of_mut!((*this_).m_cliendId), 1);
        if hr != S_OK {
            (*this_).m_cliendId = 0;
            return E_FAIL;
        }
    
        let tmp_locks: DWORD = (*this_).m_cActivateLocks;
        (*this_).m_cActivateLocks = tmp_locks.wrapping_add(1);
        if tmp_locks != 0 {
            return S_OK;
        }
    
        let mut pSource: *mut ITfSourceSingle = core::ptr::null_mut();
        hr = (*pThreadMgr).QueryInterface(&IID_ITfSourceSingle, core::ptr::addr_of_mut!(pSource) as *mut LPVOID);
        if hr < 0 {
            (*this_).DeactivateIMMX(pTLS, pThreadMgr);
            return hr;
        }
    
        'finish: {
            let pProvider: *mut CFunctionProvider = CFunctionProvider__new_in_0(cpp_alloc_CicNoThrow::<CFunctionProvider>(), (*this_).m_cliendId);
            if pProvider.is_null() {
                hr = E_FAIL;
                break 'finish;
            }
    
            (*pSource).AdviseSingleSink((*this_).m_cliendId, &IID_ITfFunctionProvider, pProvider as *mut IUnknown);
            (*pProvider).__base0.Release();
    
            if (*this_).m_pDocMgr.is_null() {
                hr = (*pThreadMgr).CreateDocumentMgr(core::ptr::addr_of_mut!((*this_).m_pDocMgr));
                if hr < 0 {
                    hr = E_FAIL;
                    break 'finish;
                }
    
                SetCompartmentDWORD((*this_).m_cliendId, ((*this_).m_pDocMgr).cast(), &GUID_COMPARTMENT_CTFIME_DIMFLAGS, TRUE, (FALSE) as BOOL);
            }
    
            (*pThreadMgr).SetSysHookSink(this_ as *mut ITfSysHookSink);
    
            hr = S_OK;
            if (*pTLS).m_bDestroyed != 0 {
                let mut Data: ENUM_CREATE_DESTROY_IC = ENUM_CREATE_DESTROY_IC { m_pTLS: pTLS, m_pBridge: this_ };
                ImmEnumInputContext(0, Some(CicBridge::EnumCreateInputContextCallback), core::ptr::addr_of_mut!(Data) as LPARAM);
            }
        }
    
        // Finish:
        if hr < 0 {
            (*this_).DeactivateIMMX(pTLS, pThreadMgr);
        }
        if !pSource.is_null() {
            (*pSource).Release();
        }
        return hr;
    }
    pub unsafe fn DeactivateIMMX(&mut self, pTLS: *mut TLS, pThreadMgr: *mut ITfThreadMgr_P) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_bDeactivating != 0 {
            return (TRUE) as HRESULT;
        }
    
        (*this_).m_bDeactivating = (TRUE) as i32;
    
        if (*this_).m_cliendId != 0 {
            let mut Data = ENUM_CREATE_DESTROY_IC { m_pTLS: pTLS, m_pBridge: this_ };
            ImmEnumInputContext(0, Some(CicBridge::EnumDestroyInputContextCallback), core::ptr::addr_of_mut!(Data) as LPARAM);
            (*pTLS).m_bDestroyed = (TRUE) as i32;
    
            let mut pSource: *mut ITfSourceSingle = core::ptr::null_mut();
            if (*pThreadMgr).QueryInterface(&IID_ITfSourceSingle, &mut pSource as *mut *mut ITfSourceSingle as *mut LPVOID) == S_OK {
                (*pSource).UnadviseSingleSink((*this_).m_cliendId, &IID_ITfFunctionProvider);
            }
    
            (*this_).m_cliendId = 0;
    
            while (*this_).m_cActivateLocks > 0 {
                (*this_).m_cActivateLocks -= 1;
                (*pThreadMgr).Deactivate();
            }
    
            if !pSource.is_null() {
                (*pSource).Release();
            }
        }
    
        if !(*this_).m_pDocMgr.is_null() {
            (*(*this_).m_pDocMgr).Release();
            (*this_).m_pDocMgr = core::ptr::null_mut();
        }
    
        (*pThreadMgr).SetSysHookSink(core::ptr::null_mut());
    
        (*this_).m_bDeactivating = (FALSE) as i32;
    
        return S_OK;
    }
    pub unsafe fn CreateInputContext(&mut self, pTLS: *mut TLS, hIMC: HIMC) -> HRESULT {
        let this_: *mut Self = self;
        let mut imcLock: CicIMCLock = CicIMCLock__make_0((hIMC).cast());
        if imcLock.m_hr < 0 {
            return imcLock.m_hr;
        }
    
        if imcLock.get().hCtfImeContext == core::ptr::null_mut() {
            let hCtfImeContext: HIMCC = ImmCreateIMCC(core::mem::size_of::<CTFIMECONTEXT>() as DWORD);
            if hCtfImeContext == core::ptr::null_mut() {
                return E_OUTOFMEMORY;
            }
            imcLock.get().hCtfImeContext = (hCtfImeContext).cast();
        }
    
        let mut imeContext: CicIMCCLock<CTFIMECONTEXT> = CicIMCCLock__make_0::<CTFIMECONTEXT>(imcLock.get().hCtfImeContext);
        let mut pCicIC: *mut CicInputContext = imeContext.get().m_pCicIC;
        if !pCicIC.is_null() {
            return S_OK;
        }
    
        pCicIC = CicInputContext__new_in_0(
            cpp_alloc_CicNoThrow::<CicInputContext>(),
            (*this_).m_cliendId,
            core::ptr::addr_of_mut!((*this_).m_LibThread),
            hIMC,
        );
        if pCicIC.is_null() {
            imeContext.unlock();
            imcLock.unlock();
            (*this_).DestroyInputContext(pTLS, hIMC);
            return E_OUTOFMEMORY;
        }
    
        if (*pTLS).m_pThreadMgr.is_null() {
            (*pCicIC).Release();
            imeContext.unlock();
            imcLock.unlock();
            (*this_).DestroyInputContext(pTLS, hIMC);
            return E_NOINTERFACE;
        }
    
        imeContext.get().m_pCicIC = pCicIC;
    
        let hr: HRESULT = (*pCicIC).CreateInputContext(((*pTLS).m_pThreadMgr).cast(), &mut imcLock);
        if hr < 0 {
            (*pCicIC).Release();
            imeContext.get().m_pCicIC = core::ptr::null_mut();
            return hr;
        }
    
        let hWnd: HWND = imcLock.get().hWnd;
        if !hWnd.is_null() && hWnd == GetFocus() {
            let pDocMgr: *mut ITfDocumentMgr = (*this_).GetDocumentManager(&mut imeContext);
            if !pDocMgr.is_null() {
                (*this_).SetAssociate(pTLS, hWnd, hIMC, (*pTLS).m_pThreadMgr, pDocMgr);
                (*pDocMgr).Release();
            }
        }
    
        return hr;
    }
    pub unsafe fn DestroyInputContext(&mut self, pTLS: *mut TLS, hIMC: HIMC) -> HRESULT {
        let this_: *mut Self = self;
        let mut imcLock: CicIMCLock = CicIMCLock__zeroed();
        imcLock.__ctor_0((hIMC).cast());
        let mut hr: HRESULT = imcLock.m_hr;
        if hr < 0 {
            return hr;
        }
    
        hr = E_FAIL;
        let mut imeContext: CicIMCCLock<CTFIMECONTEXT> = CicIMCCLock__zeroed();
        imeContext.__ctor_0((*imcLock.get()).hCtfImeContext);
        if imeContext.operator_bool() {
            hr = imeContext.m_hr;
        }
    
        if (hr >= 0) && ((*imeContext.get()).m_dwCicFlags & 1) == 0 {
            (*imeContext.get()).m_dwCicFlags |= 1;
    
            let mut pCicIC: *mut CicInputContext = (*imeContext.get()).m_pCicIC;
            if !pCicIC.is_null() {
                (*imeContext.get()).m_pCicIC = core::ptr::null_mut();
                hr = (*pCicIC).DestroyInputContext();
                (*pCicIC).Release();
                (*imeContext.get()).m_pCicIC = core::ptr::null_mut();
            }
        }
    
        if !(*imcLock.get()).hCtfImeContext.is_null() {
            ImmDestroyIMCC(((*imcLock.get()).hCtfImeContext).cast());
            (*imcLock.get()).hCtfImeContext = core::ptr::null_mut();
            hr = S_OK;
        }
    
        return hr;
    }
    pub unsafe fn GetInputContext(&mut self, imeContext: &mut CicIMCCLock<CTFIMECONTEXT>) -> *mut ITfContext {
        let this_: *mut Self = self;
        let mut pCicIC: *mut CicInputContext = (*imeContext.get()).m_pCicIC;
        if pCicIC.is_null() {
            return core::ptr::null_mut();
        }
        return (*pCicIC).m_pContext;
    }
    pub unsafe fn SelectEx(&mut self, pTLS: *mut TLS, pThreadMgr: *mut ITfThreadMgr_P, hIMC: HIMC, fSelect: BOOL, hKL: HKL) -> HRESULT {
        let this_: *mut Self = self;
        let mut imcLock: CicIMCLock = CicIMCLock__zeroed();
        imcLock.__ctor_0((hIMC).cast());
        if (imcLock.m_hr) < 0 {
            return imcLock.m_hr;
        }
    
        let mut imeContext: CicIMCCLock<CTFIMECONTEXT> = CicIMCCLock__zeroed::<CTFIMECONTEXT>();
        imeContext.__ctor_0(imcLock.get().hCtfImeContext);
        if (imeContext.m_hr) < 0 {
            return imeContext.m_hr;
        }
    
        let pCicIC: *mut CicInputContext = imeContext.get().m_pCicIC;
        if !pCicIC.is_null() {
            (*pCicIC).m_bSelecting = (TRUE) as i32;
        }
    
        if fSelect != 0 {
            if !pCicIC.is_null() {
                (*pCicIC).m_bCandidateOpen = (FALSE) as i32;
            }
            if imcLock.get().fOpen != 0 {
                (*this_).OnSetOpenStatus(pTLS, pThreadMgr, &mut imcLock, pCicIC);
            }
        } else {
            let pContext: *mut ITfContext = (*this_).GetInputContext(&mut imeContext);
            (*pThreadMgr).RequestPostponedLock(pContext);
            if !pCicIC.is_null() {
                (*pCicIC).m_bSelecting = (FALSE) as i32;
            }
            if !pContext.is_null() {
                (*pContext).Release();
            }
        }
    
        return imeContext.m_hr;
    }
    pub unsafe fn OnSetOpenStatus(&mut self, pTLS: *mut TLS, pThreadMgr: *mut ITfThreadMgr_P, imcLock: &mut CicIMCLock, pCicIC: *mut CicInputContext) -> HRESULT {
        let this_: *mut Self = self;
        if imcLock.get().fOpen == 0 && imcLock.ValidCompositionString() != 0 {
            (*pCicIC).EscbCompComplete(imcLock);
        }
    
        (*pTLS).m_bNowOpening = (TRUE) as i32;
        let hr: HRESULT = SetCompartmentDWORD((*this_).m_cliendId, (pThreadMgr).cast(), &GUID_COMPARTMENT_KEYBOARD_OPENCLOSE, (imcLock.get().fOpen) as DWORD, (FALSE) as BOOL);
        (*pTLS).m_bNowOpening = (FALSE) as i32;
        return hr;
    }
    pub unsafe fn PostTransMsg(&mut self, hWnd: HWND, cTransMsgs: INT, pTransMsgs: *const TRANSMSG) {
        let this_: *mut Self = self;
        let mut i: INT = 0;
        let mut pTransMsgs = pTransMsgs;
        while i < cTransMsgs {
            PostMessageW(hWnd, (*pTransMsgs).message, (*pTransMsgs).wParam, (*pTransMsgs).lParam);
            i += 1;
            pTransMsgs = pTransMsgs.add(1);
        }
    }
    pub unsafe fn GetDocumentManager(&mut self, imeContext: &mut CicIMCCLock<CTFIMECONTEXT>) -> *mut ITfDocumentMgr {
        let this_: *mut Self = self;
        let mut pCicIC: *mut CicInputContext = (*imeContext).get().m_pCicIC;
        if pCicIC.is_null() {
            return core::ptr::null_mut();
        }
    
        (*(*pCicIC).m_pDocumentMgr).AddRef();
        return (*pCicIC).m_pDocumentMgr;
    }
    pub unsafe fn ConfigureGeneral(&mut self, pTLS: *mut TLS, pThreadMgr: *mut ITfThreadMgr, hKL: HKL, hWnd: HWND) -> HRESULT {
        let this_: *mut Self = self;
        let mut pProfile: *mut CicProfile = (*pTLS).m_pProfile;
        if pProfile.is_null() {
            return E_OUTOFMEMORY;
        }
    
        let mut profile: TF_LANGUAGEPROFILE = core::mem::zeroed::<TF_LANGUAGEPROFILE>();
        let mut hr: HRESULT = (*pProfile).GetActiveLanguageProfile(hKL, &GUID_TFCAT_TIP_KEYBOARD, core::ptr::addr_of_mut!(profile));
        if hr < 0 {
            return hr;
        }
    
        let mut pProvider: *mut ITfFunctionProvider = core::ptr::null_mut();
        hr = (*pThreadMgr).GetFunctionProvider(&profile.clsid, core::ptr::addr_of_mut!(pProvider));
        if hr < 0 {
            return hr;
        }
    
        let mut pFnConfigure: *mut ITfFnConfigure = core::ptr::null_mut();
        hr = (*pProvider).GetFunction(&GUID_NULL, &IID_ITfFnConfigure, core::ptr::addr_of_mut!(pFnConfigure) as *mut *mut IUnknown);
        if hr < 0 {
            (*pProvider).Release();
            return hr;
        }
    
        hr = (*pFnConfigure).Show(hWnd, profile.langid, &profile.guidProfile);
    
        (*pFnConfigure).Release();
        (*pProvider).Release();
        return hr;
    }
    pub unsafe fn ConfigureRegisterWord(&mut self, pTLS: *mut TLS, pThreadMgr: *mut ITfThreadMgr, hKL: HKL, hWnd: HWND, lpData: LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        let mut pProfile: *mut CicProfile = (*pTLS).m_pProfile;
        if pProfile.is_null() {
            return E_OUTOFMEMORY;
        }
    
        let mut profile: TF_LANGUAGEPROFILE = core::mem::zeroed();
        let mut hr: HRESULT = (*pProfile).GetActiveLanguageProfile(hKL, &GUID_TFCAT_TIP_KEYBOARD, core::ptr::addr_of_mut!(profile));
        if hr < 0 {
            return hr;
        }
    
        let mut pProvider: *mut ITfFunctionProvider = core::ptr::null_mut();
        hr = (*pThreadMgr).GetFunctionProvider(&profile.clsid, core::ptr::addr_of_mut!(pProvider));
        if hr < 0 {
            return hr;
        }
    
        let mut pFunction: *mut ITfFnConfigureRegisterWord = core::ptr::null_mut();
        hr = (*pProvider).GetFunction(&GUID_NULL, &IID_ITfFnConfigureRegisterWord, core::ptr::addr_of_mut!(pFunction) as *mut *mut IUnknown);
        if hr < 0 {
            (*pProvider).Release();
            return hr;
        }
    
        let pRegWord = lpData as *mut REGISTERWORDW;
        if !pRegWord.is_null() {
            if !(*pRegWord).lpWord.is_null() {
                hr = E_OUTOFMEMORY;
                let bstrWord = SysAllocString((*pRegWord).lpWord);
                if !bstrWord.is_null() {
                    hr = (*pFunction).Show(hWnd, profile.langid, &profile.guidProfile, bstrWord);
                    SysFreeString(bstrWord);
                }
            } else {
                hr = (*pFunction).Show(hWnd, profile.langid, &profile.guidProfile, core::ptr::null_mut());
            }
        }
    
        (*pProvider).Release();
        (*pFunction).Release();
        hr
    }
    pub unsafe fn SetActiveContextAlways(&mut self, pTLS: *mut TLS, hIMC: HIMC, fActive: BOOL, hWnd: HWND, hKL: HKL) -> HRESULT {
        let this_: *mut Self = self;
        let pThreadMgr = (*pTLS).m_pThreadMgr;
        if pThreadMgr.is_null() {
            return E_OUTOFMEMORY;
        }
    
        if fActive != 0 {
            if hIMC.is_null() {
                (*this_).SetAssociate(pTLS, hWnd, hIMC, pThreadMgr, (*this_).m_pDocMgr);
                return S_OK;
            }
    
            let mut imcLock: CicIMCLock = CicIMCLock__zeroed();
            imcLock.__ctor_0((hIMC).cast());
            if imcLock.m_hr < 0 {
                return imcLock.m_hr;
            }
    
            let mut imeContext: CicIMCCLock<CTFIMECONTEXT> = CicIMCCLock__zeroed();
            imeContext.__ctor_0(imcLock.get().hCtfImeContext);
            if imeContext.m_hr < 0 {
                return imeContext.m_hr;
            }
    
            if hIMC == ImmGetContext(hWnd) {
                let pDocMgr: *mut ITfDocumentMgr = (*this_).GetDocumentManager(&mut imeContext);
                if !pDocMgr.is_null() {
                    (*this_).SetAssociate(pTLS, imcLock.get().hWnd, hIMC, pThreadMgr, pDocMgr);
                    (*pDocMgr).Release();
                }
            }
    
            return S_OK;
        }
    
        if !hIMC.is_null() && (IsEALang(LOWORD!(hKL)) == 0) {
            let mut imcLock: CicIMCLock = CicIMCLock__zeroed();
            imcLock.__ctor_0((hIMC).cast());
            if imcLock.m_hr < 0 {
                return imcLock.m_hr;
            }
    
            let mut imeContext: CicIMCCLock<CTFIMECONTEXT> = CicIMCCLock__zeroed();
            imeContext.__ctor_0(imcLock.get().hCtfImeContext);
            if imeContext.m_hr < 0 {
                return imeContext.m_hr;
            }
    
            let pCicIC: *mut CicInputContext = imeContext.get().m_pCicIC;
            if (*pCicIC).m_dwUnknown6_5[2] == 0
                && (*pCicIC).m_dwUnknown6_5[3] == 0
            {
                ImmNotifyIME(hIMC, NI_COMPOSITIONSTR, CPS_COMPLETE, 0);
            }
        }
    
        if hIMC.is_null() || (GetFocus() != hWnd) || (hIMC != ImmGetContext(hWnd)) {
            (*this_).SetAssociate(pTLS, hWnd, hIMC, pThreadMgr, (*this_).m_pDocMgr);
        }
    
        return S_OK;
    }
    pub unsafe fn SetAssociate(&mut self, pTLS: *mut TLS,
        hWnd: HWND,
        hIMC: HIMC,
        pThreadMgr: *mut ITfThreadMgr_P,
        pDocMgr: *mut ITfDocumentMgr,
    ) {
        let this_: *mut Self = self;}
    pub unsafe fn Notify(&mut self, pTLS: *mut TLS, pThreadMgr: *mut ITfThreadMgr_P, hIMC: HIMC, dwAction: DWORD, dwIndex: DWORD, dwValue: DWORD_PTR) -> HRESULT {
        let this_: *mut Self = self;
        let mut imcLock: CicIMCLock = CicIMCLock__make_0((hIMC).cast());
        if imcLock.m_hr < 0 {
            return imcLock.m_hr;
        }
    
        let mut imeContext: CicIMCCLock<CTFIMECONTEXT> = CicIMCCLock__make_0(imcLock.get().hCtfImeContext);
        if imeContext.m_hr < 0 {
            return imeContext.m_hr;
        }
    
        let pCicIC: *mut CicInputContext = imeContext.get().m_pCicIC;
        if pCicIC.is_null() {
            return E_OUTOFMEMORY;
        }
    
        let pProfile: *mut CicProfile = (*pTLS).m_pProfile;
        if pProfile.is_null() {
            return E_OUTOFMEMORY;
        }
    
        let mut LangID: LANGID = 0;
        (*pProfile).GetLangId(core::ptr::addr_of_mut!(LangID));
    
        if dwAction == NI_OPENCANDIDATE {
            if (PRIMARYLANGID!(LangID)) as u32 == LANG_KOREAN {
                if (*this_).DoOpenCandidateHanja(pThreadMgr, &mut imcLock, pCicIC) != 0 {
                    return S_OK;
                }
                return E_FAIL;
            }
            return E_NOTIMPL;
        } else if dwAction == NI_COMPOSITIONSTR {
            if dwIndex == CPS_COMPLETE {
                (*pCicIC).EscbCompComplete(&mut imcLock);
            } else if dwIndex == CPS_CONVERT || dwIndex == CPS_REVERT {
                return E_NOTIMPL;
            } else if dwIndex == CPS_CANCEL {
                (*pCicIC).EscbCompCancel(&mut imcLock);
            } else {
                return E_FAIL;
            }
            return S_OK;
        } else if dwAction == NI_CONTEXTUPDATED {
            // every inner path returns; the C `break` leaves the outer switch
            if dwValue == (IMC_SETCONVERSIONMODE as DWORD_PTR) || dwValue == (IMC_SETSENTENCEMODE as DWORD_PTR) {
                return (*this_).OnSetConversionSentenceMode(pThreadMgr, &mut imcLock, pCicIC, (dwValue) as DWORD, LangID);
            } else if dwValue == (IMC_SETOPENSTATUS as DWORD_PTR) {
                return (*this_).OnSetOpenStatus(pTLS, pThreadMgr, &mut imcLock, pCicIC);
            } else if dwValue == (IMC_SETCANDIDATEPOS as DWORD_PTR) {
                return (*pCicIC).OnSetCandidatePos(pTLS, &mut imcLock);
            } else if dwValue == (IMC_SETCOMPOSITIONFONT as DWORD_PTR) || dwValue == (IMC_SETCOMPOSITIONWINDOW as DWORD_PTR) {
                return E_NOTIMPL;
            } else {
                return E_FAIL;
            }
        } else {
            return E_NOTIMPL;
        }
    }
    pub unsafe fn ProcessKey(&mut self, pTLS: *mut TLS,
        pThreadMgr: *mut ITfThreadMgr_P,
        hIMC: HIMC,
        wParam: WPARAM,
        lParam: LPARAM,
        lpbKeyState: LPBYTE,
        pnUnknown60: *mut INT,
    ) -> BOOL {
        let this_: *mut Self = self;
        (FALSE) as i32
    }
    pub unsafe fn ToAsciiEx(&mut self, pTLS: *mut TLS,
        pThreadMgr: *mut ITfThreadMgr_P,
        uVirtKey: UINT,
        uScanCode: UINT,
        lpbKeyState: LPBYTE,
        lpTransBuf: LPTRANSMSGLIST,
        fuState: UINT,
        hIMC: HIMC,
        pResult: *mut UINT,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn SetCompositionString(&mut self, pTLS: *mut TLS, pThreadMgr: *mut ITfThreadMgr_P, hIMC: HIMC, dwIndex: DWORD, lpComp: LPCVOID, dwCompLen: DWORD, lpRead: LPCVOID, dwReadLen: DWORD) -> BOOL {
        let this_: *mut Self = self;
        let mut imcLock: CicIMCLock = CicIMCLock__zeroed();
        imcLock.__ctor_0((hIMC).cast());
        if (imcLock.m_hr) < 0 {
            return (FALSE) as BOOL;
        }
    
        let mut imeContext: CicIMCCLock<CTFIMECONTEXT> = CicIMCCLock__zeroed::<CTFIMECONTEXT>();
        imeContext.__ctor_0(imcLock.get().hCtfImeContext);
        if (imeContext.m_hr) < 0 {
            return (FALSE) as BOOL;
        }
    
        let mut pCicIC: *mut CicInputContext = imeContext.get().m_pCicIC;
        let mut pProfile = (*pTLS).m_pProfile;
        if pCicIC.is_null() || pProfile.is_null() {
            return (FALSE) as BOOL;
        }
    
        let mut uCodePage: UINT = 0;
        (*pProfile).GetCodePageA(core::ptr::addr_of_mut!(uCodePage));
    
        let mut LangID: LANGID = 0;
        if dwIndex != SCS_SETSTR
            || lpComp.is_null()
            || *(lpComp as *const WORD) != 0
            || dwCompLen == 0
            || ((*pProfile).GetLangId(core::ptr::addr_of_mut!(LangID)) < 0)
            || (PRIMARYLANGID!(LangID)) as u32 != LANG_KOREAN
        {
            return (*pCicIC).SetCompositionString(&mut imcLock, pThreadMgr, dwIndex,
                                                  lpComp, dwCompLen, lpRead, dwReadLen,
                                                  uCodePage);
        }
    
        if (imcLock.get().fdwConversion & IME_CMODE_NATIVE) != 0 {
            ImmNotifyIME(hIMC, NI_COMPOSITIONSTR, CPS_COMPLETE, 0);
            return (TRUE) as BOOL;
        }
    
        return (FALSE) as BOOL;
    }
    pub unsafe fn EscapeKorean(&mut self, pTLS: *mut TLS, hIMC: HIMC, uSubFunc: UINT, lpData: LPVOID) -> LRESULT {
        let this_: *mut Self = self;
        if uSubFunc == IME_ESC_QUERY_SUPPORT {
            return (*(lpData as *mut DWORD) == IME_ESC_HANJA_MODE) as LRESULT;
        }
        if uSubFunc == IME_ESC_HANJA_MODE {
            return (*this_).EscHanjaMode(pTLS, hIMC, lpData);
        }
        return 0;
    }
    pub unsafe fn IsOwnDim(pDocMgr: *mut ITfDocumentMgr) -> BOOL {
        let mut dwDimFlags: DWORD = 0;
        let hr: HRESULT = GetCompartmentDWORD((pDocMgr).cast(), &GUID_COMPARTMENT_CTFIME_DIMFLAGS,
                                           core::ptr::addr_of_mut!(dwDimFlags), (FALSE) as BOOL);
        if (hr < 0) {
            return (FALSE) as BOOL;
        }
        return ((dwDimFlags & 0x1) != 0) as BOOL;
    }
    pub unsafe fn DoOpenCandidateHanja(&mut self, pThreadMgr: *mut ITfThreadMgr_P, imcLock: &mut CicIMCLock, pCicIC: *mut CicInputContext) -> BOOL {
        let this_: *mut Self = self;
        return (FALSE) as BOOL;
    }
    pub unsafe fn OnSetConversionSentenceMode(&mut self, pThreadMgr: *mut ITfThreadMgr_P, imcLock: &mut CicIMCLock, pCicIC: *mut CicInputContext, dwValue: DWORD, LangID: LANGID) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
}

impl ENUM_CREATE_DESTROY_IC {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
