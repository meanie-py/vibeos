use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_80073828D8A0: [WCHAR; 10] = [0x006E, 0x0074, 0x0064, 0x006C, 0x006C, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000];
pub static CPP_COM_WSTR_97C62B2764B1: [WCHAR; 11] = [0x006D, 0x0073, 0x0069, 0x006D, 0x0074, 0x0066, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000];
pub static CPP_COM_WSTR_B33212B38D14: [WCHAR; 28] = [0x004D, 0x0053, 0x0043, 0x0054, 0x0046, 0x0049, 0x004D, 0x0045, 0x003A, 0x003A, 0x0046, 0x0075, 0x006E, 0x0063, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020, 0x0050, 0x0072, 0x006F, 0x0076, 0x0069, 0x0064, 0x0065, 0x0072, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub type FN_DllShutdownInProgress = Option<unsafe extern "system" fn() -> BOOLEAN>;
pub type FN_MsimtfIsGuidMapEnable = Option<unsafe extern "system" fn(hIMC: HIMC, pbValue: LPBOOL) -> BOOL>;
pub static mut g_hMSIMTF: HINSTANCE = core::ptr::null_mut();























#[no_mangle]
pub unsafe extern "C" fn IsEALang(mut LangID: LANGID) -> BOOL {
    if LangID == 0 {
        let pTLS: *mut TLS = TLS::GetTLS();
        if pTLS.is_null() || (*pTLS).m_pProfile.is_null() {
            return (FALSE) as BOOL;
        }

        (*(*pTLS).m_pProfile).GetLangId(core::ptr::addr_of_mut!(LangID));
    }

    let p = PRIMARYLANGID!(LangID);
    if (p) as u32 == LANG_CHINESE || (p) as u32 == LANG_JAPANESE || (p) as u32 == LANG_KOREAN {
        return (TRUE) as BOOL;
    } else {
        return (FALSE) as BOOL;
    }
}
#[no_mangle]
pub unsafe extern "C" fn DllShutdownInProgress() -> BOOLEAN {
    static mut s_fnDllShutdownInProgress: FN_DllShutdownInProgress = None;
    let mut hNTDLL: HMODULE;

    if s_fnDllShutdownInProgress.is_some() {
        return s_fnDllShutdownInProgress.unwrap()();
    }

    hNTDLL = cicGetSystemModuleHandle(CPP_COM_WSTR_80073828D8A0.as_ptr(), (FALSE) as BOOL);
    s_fnDllShutdownInProgress = core::mem::transmute(
        GetProcAddress(hNTDLL, b"RtlDllShutdownInProgress\0".as_ptr() as *const i8),
    );
    if s_fnDllShutdownInProgress.is_none() {
        return (FALSE) as BOOLEAN;
    }

    return s_fnDllShutdownInProgress.unwrap()();
}
#[no_mangle]
pub unsafe extern "C" fn IsInteractiveUserLogon() -> BOOL {
    let mut bOK: BOOL;
    let mut IsMember: BOOL = (FALSE) as i32;
    let mut pSid: PSID = core::ptr::null_mut();
    let mut IdentAuth: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY { Value: [0, 0, 0, 0, 0, 5] } /* SECURITY_NT_AUTHORITY */;

    if !(AllocateAndInitializeSid(core::ptr::addr_of_mut!(IdentAuth), 1, SECURITY_INTERACTIVE_RID,
                                  0, 0, 0, 0, 0, 0, 0, core::ptr::addr_of_mut!(pSid)) != 0)
    {
        return (FALSE) as BOOL;
    }

    bOK = CheckTokenMembership(core::ptr::null_mut(), pSid, core::ptr::addr_of_mut!(IsMember));

    if !pSid.is_null() {
        FreeSid(pSid);
    }

    return (bOK != 0 && IsMember != 0) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn GetCharsetFromLangId(dwValue: DWORD) -> BYTE {
    let mut info: CHARSETINFO = core::mem::zeroed();
    if TranslateCharsetInfo(dwValue as DWORD_PTR as *mut DWORD, &mut info as *mut CHARSETINFO, TCI_SRCLOCALE) == 0 {
        return 0 as BYTE;
    }
    return info.ciCharset as BYTE;
}
#[no_mangle]
pub unsafe extern "C" fn GetActiveContext() -> HIMC {
    let mut hwndFocus: HWND = GetFocus();
    if hwndFocus.is_null() {
        hwndFocus = GetActiveWindow();
    }
    return ImmGetContext(hwndFocus);
}
#[no_mangle]
pub unsafe extern "C" fn MsimtfIsGuidMapEnable(hIMC: HIMC, pbValue: LPBOOL) -> BOOL {
    static mut s_fn: FN_MsimtfIsGuidMapEnable = None;
    if cicGetFN(core::ptr::addr_of_mut!(g_hMSIMTF), core::ptr::addr_of_mut!(s_fn), CPP_COM_WSTR_97C62B2764B1.as_ptr(), b"MsimtfIsGuidMapEnable\0".as_ptr() as *const i8) == 0 {
        return (FALSE) as BOOL;
    }
    return s_fn.unwrap()(hIMC, pbValue);
}
#[no_mangle]
pub unsafe extern "C" fn IsVKDBEKey(uVirtKey: UINT) -> BOOL {
    match uVirtKey {
        VK_KANJI | VK_CONVERT => {
            return (TRUE) as BOOL;
        }
        _ => {
            return (VK_OEM_ATTN <= uVirtKey && uVirtKey <= VK_PA1) as BOOL;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn GetUIMCat(pLibThread: PCIC_LIBTHREAD) -> *mut ITfCategoryMgr {
    if pLibThread.is_null() {
        return core::ptr::null_mut();
    }

    if !(*pLibThread).m_pCategoryMgr.is_null() {
        return (*pLibThread).m_pCategoryMgr;
    }

    let hr: HRESULT = cicCoCreateInstance(
        &CLSID_TF_CategoryMgr,
        core::ptr::null_mut(),
        CLSCTX_INPROC_SERVER,
        &IID_ITfCategoryMgr,
        core::ptr::addr_of_mut!((*pLibThread).m_pCategoryMgr) as *mut *mut ITfCategoryMgr as *mut LPVOID,
    );
    if hr < 0 {
        return core::ptr::null_mut();
    }
    return (*pLibThread).m_pCategoryMgr;
}

unsafe extern "C" fn LibEnumItemsInCategory(pLibThread: PCIC_LIBTHREAD, rguid: REFGUID, ppEnum: *mut *mut IEnumGUID) -> HRESULT {
    let pCat: *mut ITfCategoryMgr = GetUIMCat(pLibThread);
    if pCat.is_null() {
        return E_FAIL;
    }
    return (*pCat).EnumItemsInCategory(rguid, ppEnum);
}
#[no_mangle]
pub unsafe extern "C" fn InitDisplayAttrbuteLib(pLibThread: PCIC_LIBTHREAD) -> HRESULT {
    if pLibThread.is_null() {
        return E_FAIL;
    }

    if !(*pLibThread).m_pDisplayAttrMgr.is_null() {
        (*(*pLibThread).m_pDisplayAttrMgr).Release();
        (*pLibThread).m_pDisplayAttrMgr = core::ptr::null_mut();
    }

    if cicCoCreateInstance(
        core::ptr::addr_of!(CLSID_TF_DisplayAttributeMgr),
        core::ptr::null_mut(),
        CLSCTX_INPROC_SERVER,
        core::ptr::addr_of!(IID_ITfDisplayAttributeMgr),
        core::ptr::addr_of_mut!((*pLibThread).m_pDisplayAttrMgr) as *mut *mut core::ffi::c_void,
    ) < 0
    {
        return E_FAIL;
    }

    let mut pEnumGuid: *mut IEnumGUID = core::ptr::null_mut();
    LibEnumItemsInCategory(pLibThread, &GUID_TFCAT_DISPLAYATTRIBUTEPROPERTY, core::ptr::addr_of_mut!(pEnumGuid));

    let mut hr: HRESULT = E_OUTOFMEMORY;

    EnterCriticalSection(core::ptr::addr_of_mut!(g_csLock));
    if !pEnumGuid.is_null() && g_pPropCache.is_null() {
        // `new(cicNoThrow) CDispAttrPropCache` -- CicArray<GUID>: allocate, then construct if it worked.
        g_pPropCache = cpp_alloc_CicNoThrow::<CDispAttrPropCache>();
        if !g_pPropCache.is_null() {
            (*g_pPropCache).__ctor_0();
        }
        if !g_pPropCache.is_null() {
            (*g_pPropCache).Add(GUID_PROP_ATTRIBUTE);
            let mut guid: GUID = core::mem::zeroed();
            while (*pEnumGuid).Next(1, core::ptr::addr_of_mut!(guid), core::ptr::null_mut())
                == S_OK
            {
                if !IsEqualGUID!(core::ptr::addr_of!(guid), &GUID_PROP_ATTRIBUTE) {
                    (*g_pPropCache).Add(guid);
                }
            }
            hr = S_OK;
        }
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(g_csLock));

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn UninitDisplayAttrbuteLib(pLibThread: PCIC_LIBTHREAD) -> HRESULT {
    if pLibThread.is_null() {
        return E_FAIL;
    }

    if !(*pLibThread).m_pDisplayAttrMgr.is_null() {
        (*(*pLibThread).m_pDisplayAttrMgr).Release();
        (*pLibThread).m_pDisplayAttrMgr = core::ptr::null_mut();
    }

    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn GetCompartment(pUnknown: *mut IUnknown, rguid: REFGUID, ppComp: *mut *mut ITfCompartment, bThread: BOOL) -> HRESULT {
    *ppComp = core::ptr::null_mut();

    let mut pThreadMgr: *mut ITfThreadMgr = core::ptr::null_mut();
    let mut pCompMgr: *mut ITfCompartmentMgr = core::ptr::null_mut();

    let mut hr: HRESULT;
    if bThread != 0 {
        hr = (*pUnknown).QueryInterface(core::ptr::addr_of!(IID_ITfThreadMgr), core::ptr::addr_of_mut!(pThreadMgr) as *mut *mut core::ffi::c_void);
        if (hr < 0) {
            return hr;
        }

        hr = (*pThreadMgr).GetGlobalCompartment(core::ptr::addr_of_mut!(pCompMgr));
    } else {
        hr = (*pUnknown).QueryInterface(core::ptr::addr_of!(IID_ITfCompartmentMgr), core::ptr::addr_of_mut!(pCompMgr) as *mut *mut core::ffi::c_void);
    }

    if (hr >= 0) {
        hr = E_FAIL;
        if !pCompMgr.is_null() {
            hr = (*pCompMgr).GetCompartment(rguid, ppComp);
            (*pCompMgr).Release();
        }
    }

    if !pThreadMgr.is_null() {
        (*pThreadMgr).Release();
    }

    hr
}
#[no_mangle]
pub unsafe extern "C" fn SetCompartmentDWORD(cookie: TfEditCookie, pUnknown: *mut IUnknown, rguid: REFGUID, dwValue: DWORD, bThread: BOOL) -> HRESULT {
    let mut pComp: *mut ITfCompartment = core::ptr::null_mut();
    let mut hr: HRESULT = GetCompartment(pUnknown, rguid, core::ptr::addr_of_mut!(pComp), bThread);
    if hr < 0 {
        return hr;
    }

    let mut vari: VARIANT = core::mem::zeroed();
    V_I4!(core::ptr::addr_of_mut!(vari)) = dwValue as LONG;
    V_VT!(core::ptr::addr_of_mut!(vari)) = VT_I4 as VARTYPE;
    hr = (*pComp).SetValue(cookie, core::ptr::addr_of_mut!(vari));

    (*pComp).Release();
    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn GetCompartmentDWORD(pUnknown: *mut IUnknown, rguid: *const GUID, pdwValue: LPDWORD, bThread: BOOL) -> HRESULT {
    *pdwValue = 0;

    let mut pComp: *mut ITfCompartment = core::ptr::null_mut();
    let mut hr: HRESULT = GetCompartment(pUnknown, rguid, core::ptr::addr_of_mut!(pComp), bThread);
    if hr < 0 {
        return hr;
    }

    let mut vari: VARIANT = core::mem::zeroed();
    hr = (*pComp).GetValue(core::ptr::addr_of_mut!(vari));
    if hr == S_OK {
        *pdwValue = V_I4!(core::ptr::addr_of_mut!(vari)) as DWORD;
    }

    (*pComp).Release();
    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn SetCompartmentUnknown(cookie: TfEditCookie, pUnknown: *mut IUnknown, rguid: *const GUID, punkValue: *mut IUnknown) -> HRESULT {
    let mut pComp: *mut ITfCompartment = core::ptr::null_mut();
    let mut hr: HRESULT = GetCompartment(pUnknown, rguid, core::ptr::addr_of_mut!(pComp), (FALSE) as BOOL);
    if hr < 0 {
        return hr;
    }

    let mut vari: VARIANT = core::mem::zeroed();
    V_UNKNOWN!(core::ptr::addr_of_mut!(vari)) = punkValue;
    V_VT!(core::ptr::addr_of_mut!(vari)) = VT_UNKNOWN as VARTYPE;
    hr = (*pComp).SetValue(cookie, core::ptr::addr_of_mut!(vari));

    (*pComp).Release();
    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn ClearCompartment(tid: TfClientId, pUnknown: *mut IUnknown, rguid: REFGUID, bThread: BOOL) -> HRESULT {
    let mut pCompMgr: *mut ITfCompartmentMgr = core::ptr::null_mut();
    let mut pThreadMgr: *mut ITfThreadMgr = core::ptr::null_mut();

    let mut hr: HRESULT;
    if bThread != 0 {
        hr = (*pUnknown).QueryInterface(core::ptr::addr_of!(IID_ITfThreadMgr), core::ptr::addr_of_mut!(pThreadMgr) as *mut *mut core::ffi::c_void);
        if hr < 0 {
            return hr;
        }

        hr = (*pThreadMgr).GetGlobalCompartment(core::ptr::addr_of_mut!(pCompMgr));
    } else {
        hr = (*pUnknown).QueryInterface(core::ptr::addr_of!(IID_ITfCompartmentMgr), core::ptr::addr_of_mut!(pCompMgr) as *mut *mut core::ffi::c_void);
    }

    if hr >= 0 {
        hr = E_FAIL;
        if !pCompMgr.is_null() {
            hr = (*pCompMgr).ClearCompartment(tid, rguid);
            (*pCompMgr).Release();
        }
    }

    if !pThreadMgr.is_null() {
        (*pThreadMgr).Release();
    }

    return hr;
}

impl CModeBias {
    pub unsafe fn ConvertModeBias__0(&mut self, bias: LONG) -> GUID {
        let this_: *mut Self = self;
        let mut pguid: GUID = GUID_NULL;
        for item in g_ModeBiasMap.iter() {
            if item.m_bias == bias {
                pguid = *item.m_guid;
                break;
            }
        }
    
        return pguid;
    }
    pub unsafe fn ConvertModeBias__1(&mut self, guid: &GUID) -> LONG {
        let this_: *mut Self = self;
        let mut i: usize = 0;
        while i < (*core::ptr::addr_of!(g_ModeBiasMap)).len() {
            let item: *mut _ = core::ptr::addr_of_mut!(g_ModeBiasMap[i]);
            if IsEqualGUID!(guid as *const GUID, core::ptr::addr_of!((*item).m_guid)) {
                return (*item).m_bias;
            }
            i += 1;
        }
        return 0;
    }
    pub unsafe fn SetModeBias(&mut self, rguid: &GUID) {
        let this_: *mut Self = self;
        (*this_).m_guid = *rguid;
    }
}

impl CFunctionProviderBase {
    pub unsafe fn __ctor_0(&mut self, clientId: TfClientId) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {
            (*this_).m_clientId = clientId;
            (*this_).m_guid = GUID_NULL;
            (*this_).m_bstr = core::ptr::null_mut();
            (*this_).m_cRefs = 1;
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if DllShutdownInProgress() == 0 {
            SysFreeString((*this_).m_bstr);
        }
    }
    pub unsafe fn QueryInterface(&mut self, riid: &IID, ppvObj: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        let c_tab: [QITAB; 2] = [
            QITAB { piid: &IID_ITfFunctionProvider as *const IID, dwOffset: (core::mem::offset_of!(CFunctionProviderBase, __base0) as DWORD_PTR) as u32 },
            QITAB { piid: core::ptr::null(), dwOffset: 0 },
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
            CFunctionProviderBase__delete(this_);
            return 0;
        }
        return (*this_).m_cRefs as ULONG;
    }
    pub unsafe fn GetType(&mut self, guid: *mut GUID) -> HRESULT {
        let this_: *mut Self = self;
        *guid = (*this_).m_guid;
        return S_OK;
    }
    pub unsafe fn GetDescription(&mut self, desc: *mut BSTR) -> HRESULT {
        let this_: *mut Self = self;
        *desc = SysAllocString((*this_).m_bstr);
        return if !(*desc).is_null() { S_OK } else { E_OUTOFMEMORY };
    }
    pub unsafe fn Init(&mut self, rguid: &GUID, psz: LPCWSTR) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_bstr = SysAllocString(psz);
        (*this_).m_guid = *rguid;
        return ((*this_).m_bstr != core::ptr::null_mut()) as BOOL;
    }
}

impl CFunctionProvider {
    pub unsafe fn __ctor_0(&mut self, clientId: TfClientId) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(clientId);
        (*this_).__init_vtables();
        {
            (*this_).__base0.Init(&CLSID_CAImmLayer, CPP_COM_WSTR_B33212B38D14.as_ptr());
        }
    }
    pub unsafe fn GetFunction(&mut self, guid: &GUID, riid: &IID, func: *mut *mut IUnknown) -> HRESULT {
        let this_: *mut Self = self;
        *func = core::ptr::null_mut();
    
        if IsEqualGUID!(guid, &GUID_NULL) && IsEqualIID!(riid, &IID_IAImmFnDocFeed)
        {
            let obj: *mut CFnDocFeed = CFnDocFeed__new_in_0(cpp_alloc_CicNoThrow::<CFnDocFeed>());
            *func = core::ptr::addr_of_mut!((*obj).__base0) as *mut IUnknown;
            if !(*func).is_null() {
                return S_OK;
            }
        }
    
        return E_NOINTERFACE;
    }
}

impl CFnDocFeed {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {
            (*this_).m_cRefs = 1;
        }
    }
    pub unsafe fn QueryInterface(&mut self, riid: &IID, ppvObj: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        let c_tab: [QITAB; 2] = [
            QITAB {
                piid: &IID_IAImmFnDocFeed as *const IID,
                dwOffset: core::mem::offset_of!(CFnDocFeed, __base0) as DWORD,
            },
            QITAB {
                piid: core::ptr::null_mut(),
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
            CFnDocFeed__delete(this_);
            return 0;
        }
        return (*this_).m_cRefs as ULONG;
    }
    pub unsafe fn DocFeed(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        let pTLS: *mut TLS = TLS::GetTLS();
        if pTLS.is_null() {
            return E_OUTOFMEMORY;
        }
    
        let hIMC: HIMC = GetActiveContext();
        let mut imcLock: CicIMCLock = CicIMCLock__make_0((hIMC).cast());
        if (imcLock.m_hr) < 0 {
            return imcLock.m_hr;
        }
    
        let mut imeContext: CicIMCCLock<CTFIMECONTEXT> =
            CicIMCCLock__make_0(imcLock.get().hCtfImeContext);
        if (imeContext.m_hr) < 0 {
            return imeContext.m_hr;
        }
        let pCicIC: *mut CicInputContext = imeContext.get().m_pCicIC;
        if pCicIC.is_null() {
            return E_FAIL;
        }
    
        let mut uCodePage: UINT = CP_ACP;
        (*(*pTLS).m_pProfile).GetCodePageA(core::ptr::addr_of_mut!(uCodePage));
        (*pCicIC).SetupDocFeedString(&mut imcLock, uCodePage);
        return S_OK;
    }
    pub unsafe fn ClearDocFeedBuffer(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        let _ = this_;
        if TLS::GetTLS().is_null() {
            return E_OUTOFMEMORY;
        }
    
        // The locks are RAII values, as in the C++: Drop unlocks them on every return path.
        let hIMC: HIMC = GetActiveContext();
        let mut imcLock: CicIMCLock = CicIMCLock__make_0((hIMC).cast());
        if imcLock.m_hr < 0 {
            return imcLock.m_hr;
        }
    
        let imeContext: CicIMCCLock<CTFIMECONTEXT> = CicIMCCLock__make_0(imcLock.get().hCtfImeContext);
        if imeContext.m_hr < 0 {
            return imeContext.m_hr;
        }
    
        let pCicIC: *mut CicInputContext = imeContext.get().m_pCicIC;
        if pCicIC.is_null() {
            return E_FAIL;
        }
    
        (*pCicIC).EscbClearDocFeedBuffer(&mut imcLock, (TRUE) as BOOL);
        return S_OK;
    }
    pub unsafe fn StartReconvert(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        let mut pTLS: *mut TLS = TLS::GetTLS();
        if pTLS.is_null() {
            return E_OUTOFMEMORY;
        }
        let pThreadMgr: *mut ITfThreadMgr_P = (*pTLS).m_pThreadMgr;
        if pThreadMgr.is_null() {
            return E_OUTOFMEMORY;
        }
    
        let hIMC: HIMC = GetActiveContext();
        let mut imcLock: CicIMCLock = CicIMCLock__zeroed();
        imcLock.__ctor_0((hIMC).cast());
        if (imcLock.m_hr) < 0 {
            return imcLock.m_hr;
        }
    
        let mut imeContext: CicIMCCLock<CTFIMECONTEXT> = CicIMCCLock__zeroed();
        imeContext.__ctor_0(imcLock.get().hCtfImeContext);
        if (imeContext.m_hr) < 0 {
            return imeContext.m_hr;
        }
        let pCicIC: *mut CicInputContext = imeContext.get().m_pCicIC;
        if pCicIC.is_null() {
            return E_FAIL;
        }
    
        let mut uCodePage: UINT = CP_ACP;
        (*(*pTLS).m_pProfile).GetCodePageA(core::ptr::addr_of_mut!(uCodePage));
    
        (*pCicIC).m_bReconverting = (TRUE) as i32;
        (*pCicIC).SetupReconvertString(
            &mut imcLock,
            pThreadMgr,
            uCodePage,
            0,
            0,
        );
        (*pCicIC).EndReconvertString(&mut imcLock);
        (*pCicIC).m_bReconverting = (FALSE) as i32;
        return S_OK;
    }
    pub unsafe fn StartUndoCompositionString(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        let pTLS: *mut TLS = TLS::GetTLS();
        if pTLS.is_null() {
            return E_OUTOFMEMORY;
        }
        let pThreadMgr: *mut ITfThreadMgr_P = (*pTLS).m_pThreadMgr;
        if pThreadMgr.is_null() {
            return E_OUTOFMEMORY;
        }
    
        let hIMC: HIMC = GetActiveContext();
        let mut imcLock: CicIMCLock = CicIMCLock__make_0((hIMC).cast());
        if imcLock.m_hr < 0 {
            return imcLock.m_hr;
        }
    
        let mut imeContext: CicIMCCLock<CTFIMECONTEXT> =
            CicIMCCLock__make_0(imcLock.get().hCtfImeContext);
        if imeContext.m_hr < 0 {
            return imeContext.m_hr;
        }
        let pCicIC: *mut CicInputContext = imeContext.get().m_pCicIC;
        if pCicIC.is_null() {
            return E_FAIL;
        }
    
        let mut uCodePage: UINT = CP_ACP;
        (*(*pTLS).m_pProfile).GetCodePageA(core::ptr::addr_of_mut!(uCodePage));
    
        (*pCicIC).SetupReconvertString(
            &mut imcLock,
            pThreadMgr,
            uCodePage,
            0,
            TRUE as BOOL,
        );
        (*pCicIC).EndReconvertString(&mut imcLock);
        return S_OK;
    }
}

impl MODEBIAS {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
