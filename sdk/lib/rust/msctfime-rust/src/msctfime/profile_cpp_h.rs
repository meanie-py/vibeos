use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub type FN_LanguageProfilesCallback = unsafe extern "system" fn(profile: TF_LANGUAGEPROFILE, lParam: LPARAM) -> HRESULT;













impl CicProfile {
    pub unsafe extern "system" fn ActiveLanguageProfileNotifySinkCallback(rguid1: REFGUID, rguid2: REFGUID, fActivated: BOOL, pUserData: LPVOID) -> INT {
        let pThis: *mut CicProfile = pUserData as *mut CicProfile;
        (*pThis).m_dwFlags &= !(0xE);
        return 0;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {
            (*this_).m_dwFlags = 0;
            (*this_).m_cRefs = 1;
            (*this_).m_pIPProfiles = core::ptr::null_mut();
            (*this_).m_pActiveLanguageProfileNotifySink = core::ptr::null_mut();
            (*this_).m_LangID1 = 0;
            (*this_).m_nCodePage = CP_ACP;
            (*this_).m_LangID2 = 0;
            (*this_).m_dwUnknown1 = 0;
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pIPProfiles.is_null() {
            if (*this_).m_LangID1 != 0 {
                (*(*this_).m_pIPProfiles).ChangeCurrentLanguage((*this_).m_LangID1);
            }
    
            (*(*this_).m_pIPProfiles).Release();
            (*this_).m_pIPProfiles = core::ptr::null_mut();
        }
    
        if !(*this_).m_pActiveLanguageProfileNotifySink.is_null() {
            (*(*this_).m_pActiveLanguageProfileNotifySink)._Unadvise();
            (*(*this_).m_pActiveLanguageProfileNotifySink).Release();
            (*this_).m_pActiveLanguageProfileNotifySink = core::ptr::null_mut();
        }
    }
    pub unsafe fn QueryInterface(&mut self, riid: &IID, ppvObj: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        *ppvObj = core::ptr::null_mut();
        return E_NOINTERFACE;
    }
    pub unsafe fn AddRef(&mut self) -> ULONG {
        let this_: *mut Self = self;
        return (InterlockedIncrement(core::ptr::addr_of_mut!((*this_).m_cRefs))) as ULONG;
    }
    pub unsafe fn Release(&mut self) -> ULONG {
        let this_: *mut Self = self;
        if InterlockedDecrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) == 0 {
            CicProfile__delete(this_);
            return 0;
        }
        return (*this_).m_cRefs as ULONG;
    }
    pub unsafe fn GetActiveLanguageProfile(&mut self, hKL: HKL, catid: &GUID, pProfile: *mut TF_LANGUAGEPROFILE) -> HRESULT {
        let this_: *mut Self = self;
        // `CicInterface_RefCnt<IEnumTfLanguageProfiles> pEnum;`: released when it leaves scope.
        let mut pEnum: *mut IEnumTfLanguageProfiles = core::ptr::null_mut();
        let hr: HRESULT = (*(*this_).m_pIPProfiles).EnumLanguageProfiles(LOWORD!(hKL), core::ptr::addr_of_mut!(pEnum));
        let _pEnum = ComRelease(pEnum as *mut IUnknown);
        if hr < 0 {
            return S_FALSE;
        }
    
        let mut arg: LANG_PROF_ENUM_ARG = core::mem::zeroed();
        arg.catid = *catid;
    
        // `CicEnumValue<IEnumTfLanguageProfiles, TF_LANGUAGEPROFILE>(pEnum, callback, &arg).DoEnum()`
        let ret: DWORD = CicEnumValue_DoEnum(pEnum, CicProfile::LanguageProfilesCallback, &mut arg as *mut LANG_PROF_ENUM_ARG as LPARAM);
        if ret != ERROR_SUCCESS || pProfile.is_null() {
            return S_FALSE;
        }
        *pProfile = arg.profile;
        return S_OK;
    }
    pub unsafe fn GetLangId(&mut self, pLangID: *mut LANGID) -> HRESULT {
        let this_: *mut Self = self;
        *pLangID = 0;
    
        if (*this_).m_pIPProfiles.is_null() {
            return E_FAIL;
        }
    
        if ((*this_).m_dwFlags & 4) != 0 {
            *pLangID = (*this_).m_LangID2;
            return S_OK;
        }
    
        let mut hr: HRESULT = (*(*this_).m_pIPProfiles).GetCurrentLanguage(pLangID);
        if hr >= 0 {
            (*this_).m_dwFlags |= 4;
            (*this_).m_LangID2 = *pLangID;
        }
    
        return hr;
    }
    pub unsafe fn GetCodePageA(&mut self, puCodePage: *mut UINT) -> HRESULT {
        let this_: *mut Self = self;
        if puCodePage.is_null() {
            return E_INVALIDARG;
        }
    
        if ((*this_).m_dwFlags & 2) != 0 {
            *puCodePage = (*this_).m_nCodePage;
            return S_OK;
        }
    
        *puCodePage = 0;
    
        let mut LangID: LANGID = 0;
        let mut hr: HRESULT = (*this_).GetLangId(core::ptr::addr_of_mut!(LangID));
        if hr < 0 {
            return E_FAIL;
        }
    
        let mut szBuff = [0u16; 12];
        let mut cch: INT = GetLocaleInfoW(
            LangID as u32,
            LOCALE_IDEFAULTANSICODEPAGE,
            szBuff.as_mut_ptr(),
            _countof!(szBuff) as INT,
        );
        if cch != 0 {
            szBuff[cch as usize] = 0 as u16;
            let cp: UINT = wcstoul(
                szBuff.as_ptr(),
                core::ptr::null_mut(),
                10,
            ) as UINT;
            (*this_).m_nCodePage = cp;
            *puCodePage = cp;
            (*this_).m_dwFlags |= 2;
        }
    
        return S_OK;
    }
    pub unsafe fn InitProfileInstance(&mut self, pTLS: *mut TLS) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT = TF_CreateInputProcessorProfiles(core::ptr::addr_of_mut!((*this_).m_pIPProfiles));
        if hr < 0 {
            return hr;
        }
    
        if (*this_).m_pActiveLanguageProfileNotifySink.is_null() {
            let pSink: *mut CActiveLanguageProfileNotifySink =
                CActiveLanguageProfileNotifySink__new_in_0(cpp_alloc_CicNoThrow::<CActiveLanguageProfileNotifySink>(), Some(CicProfile::ActiveLanguageProfileNotifySinkCallback), (this_).cast());
            if pSink.is_null() {
                (*(*this_).m_pIPProfiles).Release();
                (*this_).m_pIPProfiles = core::ptr::null_mut();
                return E_FAIL;
            }
            (*this_).m_pActiveLanguageProfileNotifySink = pSink;
        }
    
        if !(*pTLS).m_pThreadMgr.is_null() {
            (*(*this_).m_pActiveLanguageProfileNotifySink)._Advise(((*pTLS).m_pThreadMgr).cast());
        }
    
        return hr;
    }
    pub unsafe fn IsIME(&mut self, hKL: HKL) -> HRESULT {
        let this_: *mut Self = self;
        let mut pEnum: *mut IEnumTfLanguageProfiles = core::ptr::null_mut();
        let hr: HRESULT = (*(*this_).m_pIPProfiles).EnumLanguageProfiles(LOWORD!(hKL), core::ptr::addr_of_mut!(pEnum));
        let _pEnum = ComRelease(pEnum as *mut IUnknown);
        if hr < 0 {
            return S_FALSE;
        }
    
        let ret: DWORD = CicEnumValue_DoEnum(pEnum, CicProfile::LanguageProfilesCallback, 0);
        return if ret == ERROR_SUCCESS { S_OK } else { S_FALSE };
    }
    pub unsafe extern "system" fn LanguageProfilesCallback(profile: TF_LANGUAGEPROFILE, lParam: LPARAM) -> HRESULT {
        if profile.fActive == 0 || memcmp(core::ptr::addr_of!(profile.clsid) as *const core::ffi::c_void, core::ptr::addr_of!(GUID_NULL) as *const core::ffi::c_void, core::mem::size_of_val(&GUID_NULL)) == 0 {
            return S_FALSE;
        }
        let pArg: PLANG_PROF_ENUM_ARG = lParam as PLANG_PROF_ENUM_ARG;
        if pArg.is_null() {
            return S_OK;
        }
        if memcmp(core::ptr::addr_of!(profile.catid) as *const core::ffi::c_void, core::ptr::addr_of!((*pArg).catid) as *const core::ffi::c_void, core::mem::size_of_val(&profile.catid)) != 0 {
            return S_FALSE;
        }
        (*pArg).profile = profile;
        return S_OK;
    }
}
