use crate::prelude::*;
use crate::msctf::categorymgr_cpp_h::CategoryMgr_Constructor;
use crate::msctf::msctf_cpp_h::add_active_textservice;
use crate::msctf::msctf_cpp_h::advise_sink;
use crate::msctf::msctf_cpp_h::free_sinks;
use crate::msctf::msctf_cpp_h::get_Cookie_magic;
use crate::msctf::msctf_cpp_h::get_active_textservice;
use crate::msctf::msctf_cpp_h::list;
use crate::msctf::msctf_cpp_h::unadvise_sink;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_344365C0CD7B: [WCHAR; 11] = [0x0041, 0x0073, 0x0073, 0x0065, 0x006D, 0x0062, 0x006C, 0x0069, 0x0065, 0x0073, 0x0000];
pub static CPP_COM_WSTR_39CDE5D97C33: [WCHAR; 8] = [0x0044, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0000];
pub static CPP_COM_WSTR_4481E1ABB8A3: [WCHAR; 13] = [0x0025, 0x0073, 0x005C, 0x0030, 0x0078, 0x0025, 0x0030, 0x0038, 0x0078, 0x005C, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_508BF086E991: [WCHAR; 13] = [0x0025, 0x0073, 0x005C, 0x0025, 0x0073, 0x005C, 0x0030, 0x0078, 0x0025, 0x0030, 0x0038, 0x0078, 0x0000];
pub static CPP_COM_WSTR_632C9C2426C9: [WCHAR; 16] = [0x0025, 0x0073, 0x005C, 0x0025, 0x0073, 0x005C, 0x0030, 0x0078, 0x0025, 0x0030, 0x0038, 0x0078, 0x005C, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_6AC123285056: [WCHAR; 12] = [0x0044, 0x0065, 0x0073, 0x0063, 0x0072, 0x0069, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_98598801D8CA: [WCHAR; 16] = [0x004C, 0x0061, 0x006E, 0x0067, 0x0075, 0x0061, 0x0067, 0x0065, 0x0050, 0x0072, 0x006F, 0x0066, 0x0069, 0x006C, 0x0065, 0x0000];
pub static CPP_COM_WSTR_9CC9D0F2200B: [WCHAR; 10] = [0x0049, 0x0063, 0x006F, 0x006E, 0x0049, 0x006E, 0x0064, 0x0065, 0x0078, 0x0000];
pub static CPP_COM_WSTR_BA2342D760A2: [WCHAR; 6] = [0x0025, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_D0065156FE9C: [WCHAR; 7] = [0x0045, 0x006E, 0x0061, 0x0062, 0x006C, 0x0065, 0x0000];
pub static CPP_COM_WSTR_E3101A5DE409: [WCHAR; 19] = [0x0025, 0x0073, 0x005C, 0x0025, 0x0073, 0x005C, 0x0025, 0x0073, 0x005C, 0x0030, 0x0078, 0x0025, 0x0030, 0x0038, 0x0078, 0x005C, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_E93F64359204: [WCHAR; 8] = [0x0050, 0x0072, 0x006F, 0x0066, 0x0069, 0x006C, 0x0065, 0x0000];
pub static CPP_COM_WSTR_E9AA956CFC7B: [WCHAR; 9] = [0x0049, 0x0063, 0x006F, 0x006E, 0x0046, 0x0069, 0x006C, 0x0065, 0x0000];

use core::ffi::c_void;

static mut add_userkey_unused_guard: u8 = 0;




































































unsafe extern "C" fn add_userkey(rclsid: REFCLSID, langid: LANGID, guidProfile: REFGUID) {
    let mut key: HKEY = core::mem::zeroed();
    let mut buf: [WCHAR; 39] = [0; 39];
    let mut buf2: [WCHAR; 39] = [0; 39];
    let mut fullkey: [WCHAR; 168] = [0; 168];
    let mut disposition: DWORD = 0;
    let mut error: LSTATUS;

    StringFromGUID2(rclsid, buf.as_mut_ptr(), (_countof!(buf)) as INT);
    StringFromGUID2(guidProfile, buf2.as_mut_ptr(), (_countof!(buf2)) as INT);
    StringCchPrintfW!(fullkey.as_mut_ptr(), _countof!(fullkey), CPP_COM_WSTR_E3101A5DE409.as_ptr(),
                     szwSystemTIPKey, buf.as_mut_ptr(), CPP_COM_WSTR_98598801D8CA.as_ptr(), langid as core::ffi::c_uint, buf2.as_mut_ptr());

    error = RegCreateKeyExW(HKEY_CURRENT_USER, fullkey.as_mut_ptr(), 0, core::ptr::null_mut(), 0,
                            KEY_READ | KEY_WRITE, core::ptr::null_mut(), core::ptr::addr_of_mut!(key), core::ptr::addr_of_mut!(disposition));

    if (error) as u32 == ERROR_SUCCESS && disposition == REG_CREATED_NEW_KEY {
        let mut zero: DWORD = 0;
        RegSetValueExW(key, CPP_COM_WSTR_D0065156FE9C.as_ptr(), 0, REG_DWORD, core::ptr::addr_of_mut!(zero) as PBYTE, core::mem::size_of::<DWORD>() as DWORD);
    }

    if (error) as u32 == ERROR_SUCCESS {
        RegCloseKey(key);
    }
}
#[no_mangle]
pub unsafe extern "C" fn InputProcessorProfiles_Constructor(pUnkOuter: *mut IUnknown, ppOut: *mut *mut IUnknown) -> HRESULT {
    return CInputProcessorProfiles::CreateInstance(pUnkOuter, ppOut as *mut *mut CInputProcessorProfiles);
}

impl CInputProcessorProfiles {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_cRefs = 1;
        (*this_).m_currentLanguage = GetUserDefaultLCID() as LANGID;
        {
            list_init(core::ptr::addr_of_mut!((*this_).m_LanguageProfileNotifySink));
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        free_sinks((core::ptr::addr_of_mut!((*this_).m_LanguageProfileNotifySink)).cast());
    }
    pub unsafe fn CreateInstance(pUnkOuter: *mut IUnknown, ppOut: *mut *mut CInputProcessorProfiles) -> HRESULT {
        if !pUnkOuter.is_null() {
            return CLASS_E_NOAGGREGATION;
        }
    
        let This: *mut CInputProcessorProfiles = CInputProcessorProfiles__new_in_0(cpp_alloc_CicNoThrow::<CInputProcessorProfiles>());
        if This.is_null() {
            return E_OUTOFMEMORY;
        }
    
        *ppOut = This;
    
        return S_OK;
    }
    pub unsafe fn QueryInterface(&mut self, iid: &IID, ppvObj: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        *ppvObj = core::ptr::null_mut();
    
        if IsEqualGUID!(&(*iid), &(IID_IUnknown)) || IsEqualGUID!(&(*iid), &(IID_ITfInputProcessorProfiles)) {
            *ppvObj = core::ptr::addr_of_mut!((*this_).__base0) as LPVOID;
        } else if IsEqualGUID!(&(*iid), &(IID_ITfInputProcessorProfileMgr)) {
            *ppvObj = core::ptr::addr_of_mut!((*this_).__base2) as LPVOID;
        } else if IsEqualGUID!(&(*iid), &(IID_ITfSource)) {
            *ppvObj = core::ptr::addr_of_mut!((*this_).__base1) as LPVOID;
        }
    
        if (*ppvObj).is_null() {
            return E_NOINTERFACE;
        }
    
        (*this_).AddRef();
        return S_OK;
    }
    pub unsafe fn AddRef(&mut self) -> ULONG {
        let this_: *mut Self = self;
        return InterlockedIncrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) as ULONG;
    }
    pub unsafe fn Release(&mut self) -> ULONG {
        let this_: *mut Self = self;
        let ret: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) as ULONG;
        if ret == 0 {
            CInputProcessorProfiles__delete(this_);
        }
        return ret;
    }
    pub unsafe fn Register(&mut self, rclsid: &CLSID) -> HRESULT {
        let this_: *mut Self = self;
        let mut tipkey: HKEY = core::mem::zeroed();
        let mut buf: [WCHAR; 39] = core::mem::zeroed();
        let mut fullkey: [WCHAR; 68] = core::mem::zeroed();
    
        StringFromGUID2(rclsid, buf.as_mut_ptr(), (_countof!(buf)) as INT);
        StringCchPrintfW!(fullkey.as_mut_ptr(), _countof!(fullkey), CPP_COM_WSTR_BA2342D760A2.as_ptr(), szwSystemTIPKey, buf.as_mut_ptr());
    
        if (RegCreateKeyExW(HKEY_LOCAL_MACHINE, fullkey.as_mut_ptr(), 0, core::ptr::null_mut(), 0, KEY_READ | KEY_WRITE, core::ptr::null_mut(),
                            &mut tipkey, core::ptr::null_mut())) as u32 != ERROR_SUCCESS
        {
            return E_FAIL;
        }
    
        RegCloseKey(tipkey);
        return S_OK;
    }
    pub unsafe fn Unregister(&mut self, rclsid: &CLSID) -> HRESULT {
        let this_: *mut Self = self;
        let mut buf = [0u16; 39];
        let mut fullkey = [0u16; 68];
    
        StringFromGUID2(rclsid, buf.as_mut_ptr(), (_countof!(buf)) as INT);
        StringCchPrintfW!(fullkey.as_mut_ptr(), _countof!(fullkey), CPP_COM_WSTR_BA2342D760A2.as_ptr(), szwSystemTIPKey, buf.as_mut_ptr());
    
        RegDeleteTreeW(HKEY_LOCAL_MACHINE, fullkey.as_mut_ptr());
        RegDeleteTreeW(HKEY_CURRENT_USER, fullkey.as_mut_ptr());
        return S_OK;
    }
    pub unsafe fn AddLanguageProfile(&mut self, rclsid: &CLSID, langid: LANGID, guidProfile: &GUID, pchDesc: *const WCHAR, cchDesc: ULONG, pchIconFile: *const WCHAR, cchFile: ULONG, uIconIndex: ULONG) -> HRESULT {
        let this_: *mut Self = self;
        let mut tipkey: HKEY = core::ptr::null_mut();
        let mut fmtkey: HKEY = core::ptr::null_mut();
        let mut buf: [WCHAR; 39] = core::mem::zeroed();
        let mut fullkey: [WCHAR; 100] = core::mem::zeroed();
        let mut disposition: DWORD = 0;
        let mut error: LSTATUS;
    
        StringFromGUID2(rclsid as *const CLSID, buf.as_mut_ptr(), (_countof!(buf)) as INT);
        StringCchPrintfW!(fullkey.as_mut_ptr(), _countof!(fullkey), CPP_COM_WSTR_BA2342D760A2.as_ptr(), szwSystemTIPKey, buf.as_mut_ptr());
    
        error = RegOpenKeyExW(HKEY_LOCAL_MACHINE, fullkey.as_ptr(), 0, KEY_READ | KEY_WRITE, core::ptr::addr_of_mut!(tipkey));
        if (error) as u32 != ERROR_SUCCESS {
            return E_FAIL;
        }
    
        StringFromGUID2(guidProfile as *const GUID, buf.as_mut_ptr(), (_countof!(buf)) as INT);
        StringCchPrintfW!(fullkey.as_mut_ptr(), _countof!(fullkey), CPP_COM_WSTR_4481E1ABB8A3.as_ptr(), CPP_COM_WSTR_98598801D8CA.as_ptr(), langid as core::ffi::c_uint, buf.as_mut_ptr());
    
        error = RegCreateKeyExW(tipkey, fullkey.as_ptr(), 0, core::ptr::null_mut(), 0, KEY_READ | KEY_WRITE,
                                core::ptr::null_mut(), core::ptr::addr_of_mut!(fmtkey), core::ptr::addr_of_mut!(disposition));
        if (error) as u32 == ERROR_SUCCESS {
            let zero: DWORD = 0x0;
            RegSetValueExW(fmtkey, CPP_COM_WSTR_6AC123285056.as_ptr(), 0, REG_SZ, pchDesc as PBYTE, (cchDesc as usize * core::mem::size_of::<WCHAR>()) as DWORD);
            RegSetValueExW(fmtkey, CPP_COM_WSTR_E9AA956CFC7B.as_ptr(), 0, REG_SZ, pchIconFile as PBYTE, (cchFile as usize * core::mem::size_of::<WCHAR>()) as DWORD);
            RegSetValueExW(fmtkey, CPP_COM_WSTR_9CC9D0F2200B.as_ptr(), 0, REG_DWORD, core::ptr::addr_of!(uIconIndex) as PBYTE, core::mem::size_of::<ULONG>() as DWORD);
            if disposition == REG_CREATED_NEW_KEY {
                RegSetValueExW(fmtkey, CPP_COM_WSTR_D0065156FE9C.as_ptr(), 0, REG_DWORD, core::ptr::addr_of!(zero) as PBYTE, core::mem::size_of::<DWORD>() as DWORD);
            }
            RegCloseKey(fmtkey);
            add_userkey(rclsid, langid, guidProfile);
        }
        RegCloseKey(tipkey);
    
        if (error) as u32 == ERROR_SUCCESS { S_OK } else { E_FAIL }
    }
    pub unsafe fn RemoveLanguageProfile(&mut self, rclsid: &CLSID, langid: LANGID, guidProfile: &GUID) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn EnumInputProcessorInfo(&mut self, ppEnum: *mut *mut IEnumGUID) -> HRESULT {
        let this_: *mut Self = self;
        return CProfilesEnumGuid::CreateInstance(ppEnum as *mut *mut CProfilesEnumGuid);
    }
    pub unsafe fn GetDefaultLanguageProfile(&mut self, langid: LANGID, catid: &GUID, pclsid: *mut CLSID, pguidProfile: *mut GUID) -> HRESULT {
        let this_: *mut Self = self;
        let mut fullkey: [WCHAR; 168] = [0; 168];
        let mut buf: [WCHAR; 39] = [0; 39];
        let mut hKey: HKEY = core::mem::zeroed();
        let mut count: DWORD = 0;
        let mut error: LSTATUS = 0;
    
        if cicIsNullPtr(catid as *const GUID) || pclsid.is_null() || pguidProfile.is_null() {
            return E_INVALIDARG;
        }
    
        StringFromGUID2(catid as *const GUID, buf.as_mut_ptr(), _countof!(buf) as i32);
        StringCchPrintfW!(fullkey.as_mut_ptr(), _countof!(fullkey) as usize, CPP_COM_WSTR_632C9C2426C9.as_ptr(), szwSystemCTFKey,
                         CPP_COM_WSTR_344365C0CD7B.as_ptr(), langid as core::ffi::c_uint, buf.as_ptr());
    
        error = RegOpenKeyExW(HKEY_CURRENT_USER, fullkey.as_ptr(), 0, KEY_READ | KEY_WRITE, core::ptr::addr_of_mut!(hKey));
        if (error) as u32 != ERROR_SUCCESS {
            return S_FALSE;
        }
    
        count = core::mem::size_of::<[WCHAR; 39]>() as DWORD;
        error = RegQueryValueExW(hKey, CPP_COM_WSTR_39CDE5D97C33.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), buf.as_mut_ptr() as PBYTE, core::ptr::addr_of_mut!(count));
        if (error) as u32 != ERROR_SUCCESS {
            RegCloseKey(hKey);
            return S_FALSE;
        }
        CLSIDFromString(buf.as_ptr(), pclsid);
    
        error = RegQueryValueExW(hKey, CPP_COM_WSTR_E93F64359204.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), buf.as_mut_ptr() as PBYTE, core::ptr::addr_of_mut!(count));
        if (error) as u32 == ERROR_SUCCESS {
            CLSIDFromString(buf.as_ptr(), pguidProfile);
        }
    
        RegCloseKey(hKey);
    
        return S_OK;
    }
    pub unsafe fn SetDefaultLanguageProfile(&mut self, langid: LANGID, rclsid: &CLSID, guidProfiles: &GUID) -> HRESULT {
        let this_: *mut Self = self;
        let mut fullkey = [0u16; 168];
        let mut buf = [0u16; 39];
        let mut hKey: HKEY = core::mem::zeroed();
        let mut catid: GUID = core::mem::zeroed();
        let mut hr: HRESULT = core::mem::zeroed();
        let mut catmgr: *mut ITfCategoryMgr = core::ptr::null_mut();
        let mut error: LSTATUS = core::mem::zeroed();
        let tipcats: [*const GUID; 3] = [&GUID_TFCAT_TIP_KEYBOARD, &GUID_TFCAT_TIP_SPEECH, &GUID_TFCAT_TIP_HANDWRITING];
    
        if cicIsNullPtr(rclsid) || cicIsNullPtr(guidProfiles) {
            return E_INVALIDARG;
        }
    
        hr = CategoryMgr_Constructor(core::ptr::null_mut(), core::ptr::addr_of_mut!(catmgr) as *mut *mut IUnknown);
        if (hr) < 0 {
            return hr;
        }
    
        if (*catmgr).FindClosestCategory(rclsid, core::ptr::addr_of_mut!(catid), tipcats.as_ptr() as *mut *const GUID, _countof!(tipcats) as ULONG) != S_OK {
            hr = (*catmgr).FindClosestCategory(rclsid, core::ptr::addr_of_mut!(catid), core::ptr::null_mut(), 0);
        }
        (*catmgr).Release();
        if (hr) < 0 {
            return E_FAIL;
        }
    
        StringFromGUID2(core::ptr::addr_of!(catid), buf.as_mut_ptr(), (_countof!(buf)) as INT);
        StringCchPrintfW!(fullkey.as_mut_ptr(), _countof!(fullkey), CPP_COM_WSTR_632C9C2426C9.as_ptr(), szwSystemCTFKey, CPP_COM_WSTR_344365C0CD7B.as_ptr(), langid as core::ffi::c_uint, buf.as_mut_ptr());
    
        error = RegCreateKeyExW(HKEY_CURRENT_USER, fullkey.as_ptr(), 0, core::ptr::null_mut(), 0, KEY_READ | KEY_WRITE,
                                core::ptr::null_mut(), core::ptr::addr_of_mut!(hKey), core::ptr::null_mut());
        if (error) as u32 != ERROR_SUCCESS {
            return E_FAIL;
        }
    
        StringFromGUID2(rclsid, buf.as_mut_ptr(), (_countof!(buf)) as INT);
        RegSetValueExW(hKey, CPP_COM_WSTR_39CDE5D97C33.as_ptr(), 0, REG_SZ, buf.as_mut_ptr() as PBYTE, core::mem::size_of_val(&buf) as u32);
        StringFromGUID2(guidProfiles, buf.as_mut_ptr(), (_countof!(buf)) as INT);
        RegSetValueExW(hKey, CPP_COM_WSTR_E93F64359204.as_ptr(), 0, REG_SZ, buf.as_mut_ptr() as PBYTE, core::mem::size_of_val(&buf) as u32);
        RegCloseKey(hKey);
    
        return S_OK;
    }
    pub unsafe fn ActivateLanguageProfile(&mut self, rclsid: &CLSID, langid: LANGID, guidProfiles: &GUID) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT;
        let mut enabled: BOOL = 0;
        let mut LanguageProfile: TF_LANGUAGEPROFILE = core::mem::zeroed();
    
        if langid != (*this_).m_currentLanguage {
            return E_INVALIDARG;
        }
    
        if ((get_active_textservice(rclsid, core::ptr::null_mut())) != 0) {
            return E_FAIL;
        }
    
        hr = (*this_).IsEnabledLanguageProfile(rclsid, langid, guidProfiles, core::ptr::addr_of_mut!(enabled));
        if hr < 0 || enabled == 0 {
            return E_FAIL;
        }
    
        LanguageProfile.clsid = *rclsid;
        LanguageProfile.langid = langid;
        LanguageProfile.guidProfile = *guidProfiles;
        LanguageProfile.fActive = (TRUE) as i32;
        return add_active_textservice(core::ptr::addr_of_mut!(LanguageProfile));
    }
    pub unsafe fn GetActiveLanguageProfile(&mut self, rclsid: &CLSID, plangid: *mut LANGID, pguidProfile: *mut GUID) -> HRESULT {
        let this_: *mut Self = self;
        let mut profile: TF_LANGUAGEPROFILE = core::mem::zeroed();
    
        if cicIsNullPtr(rclsid as *const CLSID) || plangid.is_null() || pguidProfile.is_null() {
            return E_INVALIDARG;
        }
    
        if get_active_textservice(rclsid, core::ptr::addr_of_mut!(profile)) != 0 {
            *plangid = profile.langid;
            *pguidProfile = profile.guidProfile;
            return S_OK;
        } else {
            *pguidProfile = GUID_NULL;
            return S_FALSE;
        }
    }
    pub unsafe fn GetLanguageProfileDescription(&mut self, rclsid: &CLSID, langid: LANGID, guidProfile: &GUID, pbstrProfile: *mut BSTR) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn GetCurrentLanguage(&mut self, plangid: *mut LANGID) -> HRESULT {
        let this_: *mut Self = self;
        if plangid.is_null() {
            return E_INVALIDARG;
        }
    
        *plangid = (*this_).m_currentLanguage;
        return S_OK;
    }
    pub unsafe fn ChangeCurrentLanguage(&mut self, langid: LANGID) -> HRESULT {
        let this_: *mut Self = self;
        let mut sink: *mut ITfLanguageProfileNotifySink;
        let mut cursor: *mut list;
        let mut accept: BOOL;
    
        SINK_FOR_EACH!(cursor, core::ptr::addr_of_mut!((*this_).m_LanguageProfileNotifySink), ITfLanguageProfileNotifySink, sink, {
            accept = (TRUE) as i32;
            (*sink).OnLanguageChange(langid, core::ptr::addr_of_mut!(accept));
            if !(accept != 0) {
                return E_FAIL;
            }
        });
    
        /* TODO:  On successful language change call OnLanguageChanged sink */
        return E_NOTIMPL;
    }
    pub unsafe fn GetLanguageList(&mut self, ppLangId: *mut *mut LANGID, pulCount: *mut ULONG) -> HRESULT {
        let this_: *mut Self = self;
        *ppLangId = CoTaskMemAlloc((core::mem::size_of::<LANGID>()) as SIZE_T) as *mut LANGID;
        **ppLangId = (*this_).m_currentLanguage;
        *pulCount = 1 as ULONG;
        return S_OK;
    }
    pub unsafe fn EnumLanguageProfiles(&mut self, langid: LANGID, ppEnum: *mut *mut IEnumTfLanguageProfiles) -> HRESULT {
        let this_: *mut Self = self;
        if ppEnum.is_null() {
            return E_INVALIDARG;
        }
    
        let mut profenum: *mut CEnumTfLanguageProfiles = core::ptr::null_mut();
        let hr: HRESULT = CEnumTfLanguageProfiles::CreateInstance(langid, core::ptr::addr_of_mut!(profenum));
        *ppEnum = core::ptr::addr_of_mut!((*profenum).__base0) as *mut IEnumTfLanguageProfiles;
        return hr;
    }
    pub unsafe fn EnableLanguageProfile(&mut self, rclsid: &CLSID, langid: LANGID, guidProfile: &GUID, mut fEnable: BOOL) -> HRESULT {
        let this_: *mut Self = self;
        let mut key: HKEY = core::ptr::null_mut();
        let mut buf: [WCHAR; 39] = [0; 39];
        let mut buf2: [WCHAR; 39] = [0; 39];
        let mut fullkey: [WCHAR; 168] = [0; 168];
        let mut error: LSTATUS = 0;
    
        StringFromGUID2(rclsid as *const CLSID as *const GUID, buf.as_mut_ptr(), _countof!(buf) as i32);
        StringFromGUID2(guidProfile as *const GUID, buf2.as_mut_ptr(), _countof!(buf2) as i32);
        StringCchPrintfW!(fullkey.as_mut_ptr(), _countof!(fullkey), CPP_COM_WSTR_E3101A5DE409.as_ptr(), szwSystemTIPKey, buf.as_mut_ptr(),
                         CPP_COM_WSTR_98598801D8CA.as_ptr(), langid as core::ffi::c_uint, buf2.as_mut_ptr());
    
        error = RegOpenKeyExW(HKEY_CURRENT_USER, fullkey.as_ptr(), 0, KEY_READ | KEY_WRITE, core::ptr::addr_of_mut!(key));
        if (error) as u32 != ERROR_SUCCESS {
            return E_FAIL;
        }
    
        RegSetValueExW(key, CPP_COM_WSTR_D0065156FE9C.as_ptr(), 0, REG_DWORD, core::ptr::addr_of_mut!(fEnable) as LPBYTE, core::mem::size_of::<BOOL>() as DWORD);
        RegCloseKey(key);
        return S_OK;
    }
    pub unsafe fn IsEnabledLanguageProfile(&mut self, rclsid: &CLSID, langid: LANGID, guidProfile: &GUID, pfEnable: *mut BOOL) -> HRESULT {
        let this_: *mut Self = self;
        let mut key: HKEY = core::ptr::null_mut();
        let mut buf: [WCHAR; 39] = core::mem::zeroed();
        let mut buf2: [WCHAR; 39] = core::mem::zeroed();
        let mut fullkey: [WCHAR; 168] = core::mem::zeroed();
        let mut error: LSTATUS;
    
        if pfEnable.is_null() {
            return E_INVALIDARG;
        }
    
        StringFromGUID2(rclsid as *const CLSID, buf.as_mut_ptr(), _countof!(buf) as core::ffi::c_int);
        StringFromGUID2(guidProfile as *const GUID, buf2.as_mut_ptr(), _countof!(buf2) as core::ffi::c_int);
        StringCchPrintfW!(fullkey.as_mut_ptr(), _countof!(fullkey) as core::ffi::c_int, CPP_COM_WSTR_E3101A5DE409.as_ptr(), szwSystemTIPKey,
                         buf.as_ptr(), CPP_COM_WSTR_98598801D8CA.as_ptr(), langid as core::ffi::c_int, buf2.as_ptr());
    
        error = RegOpenKeyExW(HKEY_CURRENT_USER, fullkey.as_ptr(), 0, KEY_READ | KEY_WRITE, core::ptr::addr_of_mut!(key));
        if (error) as u32 == ERROR_SUCCESS
        {
            let mut count: DWORD = core::mem::size_of::<DWORD>() as DWORD;
            error = RegQueryValueExW(key, CPP_COM_WSTR_D0065156FE9C.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), pfEnable as LPBYTE, core::ptr::addr_of_mut!(count));
            RegCloseKey(key);
        }
    
        if (error) as u32 != ERROR_SUCCESS /* Try Default */
        {
            error = RegOpenKeyExW(HKEY_LOCAL_MACHINE, fullkey.as_ptr(), 0, KEY_READ | KEY_WRITE, core::ptr::addr_of_mut!(key));
            if (error) as u32 == ERROR_SUCCESS
            {
                let mut count: DWORD = core::mem::size_of::<DWORD>() as DWORD;
                error = RegQueryValueExW(key, CPP_COM_WSTR_D0065156FE9C.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), pfEnable as LPBYTE, core::ptr::addr_of_mut!(count));
                RegCloseKey(key);
            }
        }
    
        if (error) as u32 == ERROR_SUCCESS { S_OK } else { E_FAIL }
    }
    pub unsafe fn EnableLanguageProfileByDefault(&mut self, rclsid: &CLSID, langid: LANGID, guidProfile: &GUID, fEnable: BOOL) -> HRESULT {
        let this_: *mut Self = self;
        let mut key: HKEY = core::mem::zeroed();
        let mut buf: [WCHAR; 39] = core::mem::zeroed();
        let mut buf2: [WCHAR; 39] = core::mem::zeroed();
        let mut fullkey: [WCHAR; 168] = core::mem::zeroed();
        let mut error: LSTATUS;
    
        StringFromGUID2(rclsid, buf.as_mut_ptr(), (_countof!(buf)) as INT);
        StringFromGUID2(guidProfile, buf2.as_mut_ptr(), (_countof!(buf2)) as INT);
        StringCchPrintfW!(fullkey.as_mut_ptr(), _countof!(fullkey), CPP_COM_WSTR_E3101A5DE409.as_ptr(), szwSystemTIPKey,
                         buf.as_mut_ptr(), CPP_COM_WSTR_98598801D8CA.as_ptr(), langid as core::ffi::c_uint, buf2.as_mut_ptr());
    
        error = RegOpenKeyExW(HKEY_LOCAL_MACHINE, fullkey.as_ptr(), 0, KEY_READ | KEY_WRITE, core::ptr::addr_of_mut!(key));
        if (error) as u32 != ERROR_SUCCESS {
            return E_FAIL;
        }
    
        RegSetValueExW(key, CPP_COM_WSTR_D0065156FE9C.as_ptr(), 0, REG_DWORD, core::ptr::addr_of!(fEnable) as PBYTE, core::mem::size_of::<BOOL>() as DWORD);
        RegCloseKey(key);
        return S_OK;
    }
    pub unsafe fn SubstituteKeyboardLayout(&mut self, rclsid: &CLSID, langid: LANGID, guidProfile: &GUID, hKL: HKL) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn AdviseSink(&mut self, riid: &IID, punk: *mut IUnknown, pdwCookie: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if cicIsNullPtr(riid as *const GUID) || punk.is_null() || pdwCookie.is_null() {
            return E_INVALIDARG;
        }
    
        if IsEqualGUID!(&(*riid), &(IID_ITfLanguageProfileNotifySink)) {
            return advise_sink((core::ptr::addr_of_mut!((*this_).m_LanguageProfileNotifySink)).cast(), &IID_ITfLanguageProfileNotifySink, COOKIE_MAGIC_IPPSINK, punk, pdwCookie);
        }
    
        return E_NOTIMPL;
    }
    pub unsafe fn UnadviseSink(&mut self, dwCookie: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if get_Cookie_magic(dwCookie) != COOKIE_MAGIC_IPPSINK {
            return E_INVALIDARG;
        }
    
        unadvise_sink(dwCookie)
    }
    pub unsafe fn ActivateProfile(&mut self, dwProfileType: DWORD, langid: LANGID, clsid: &CLSID, guidProfile: &GUID, hkl: HKL, dwFlags: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn DeactivateProfile(&mut self, dwProfileType: DWORD, langid: LANGID, clsid: &CLSID, guidProfile: &GUID, hkl: HKL, dwFlags: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn GetProfile(&mut self, dwProfileType: DWORD, langid: LANGID, clsid: &CLSID, guidProfile: &GUID, hkl: HKL, pProfile: *mut TF_INPUTPROCESSORPROFILE) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn EnumProfiles(&mut self, langid: LANGID, ppEnum: *mut *mut IEnumTfInputProcessorProfiles) -> HRESULT {
        let this_: *mut Self = self;
        let mut enum_profiles: *mut CEnumTfInputProcessorProfiles = CEnumTfInputProcessorProfiles__new_in_0(cpp_alloc_CicNoThrow::<CEnumTfInputProcessorProfiles>());
        if enum_profiles.is_null() {
            return E_OUTOFMEMORY;
        }
    
        *ppEnum = core::ptr::addr_of_mut!((*enum_profiles).__base0) as *mut IEnumTfInputProcessorProfiles;
        return S_OK;
    }
    pub unsafe fn ReleaseInputProcessor(&mut self, rclsid: &CLSID, dwFlags: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn RegisterProfile(&mut self, rclsid: &CLSID, langid: LANGID, guidProfile: &GUID, pchDesc: *const WCHAR, cchDesc: ULONG, pchIconFile: *const WCHAR, cchFile: ULONG, uIconIndex: ULONG, hklsubstitute: HKL, dwPreferredLayout: DWORD, bEnabledByDefault: BOOL, dwFlags: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn UnregisterProfile(&mut self, rclsid: &CLSID, langid: LANGID, guidProfile: &GUID, dwFlags: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn GetActiveProfile(&mut self, catid: &GUID, pProfile: *mut TF_INPUTPROCESSORPROFILE) -> HRESULT {
        let this_: *mut Self = self;
        let _ = (this_, catid, pProfile);
        return E_NOTIMPL;
    }
}

impl CProfilesEnumGuid {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_cRefs = 1;
        (*this_).m_key = core::ptr::null_mut();
        (*this_).m_next_index = 0;
        {
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        RegCloseKey((*this_).m_key);
    }
    pub unsafe fn CreateInstance(ppOut: *mut *mut CProfilesEnumGuid) -> HRESULT {
        let This: *mut CProfilesEnumGuid = CProfilesEnumGuid__new_in_0(cpp_alloc_CicNoThrow::<CProfilesEnumGuid>());
        if This.is_null() {
            return E_OUTOFMEMORY;
        }
    
        if (RegCreateKeyExW(HKEY_LOCAL_MACHINE, szwSystemTIPKey.as_ptr(), 0, core::ptr::null_mut(), 0,
                            KEY_READ | KEY_WRITE, core::ptr::null_mut(), core::ptr::addr_of_mut!((*This).m_key), core::ptr::null_mut())) as u32 != ERROR_SUCCESS
        {
            CProfilesEnumGuid__delete(This);
            return E_FAIL;
        }
    
        *ppOut = This;
    
        return S_OK;
    }
    pub unsafe fn QueryInterface(&mut self, iid: &IID, ppvObj: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        *ppvObj = core::ptr::null_mut();
    
        if IsEqualGUID!(&(*iid), &(IID_IUnknown)) || IsEqualGUID!(&(*iid), &(IID_IEnumGUID)) {
            *ppvObj = core::ptr::addr_of_mut!((*this_).__base0) as LPVOID;
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
            CProfilesEnumGuid__delete(this_);
        }
        return ret;
    }
    pub unsafe fn Next(&mut self, celt: ULONG, mut rgelt: *mut GUID, pceltFetched: *mut ULONG) -> HRESULT {
        let this_: *mut Self = self;
        let mut fetched: ULONG = 0;
    
        if rgelt.is_null() {
            return E_POINTER;
        }
    
        if !(*this_).m_key.is_null() {
            // nolint: dowhile-continue -- as in the C++: a key that is no CLSID is skipped and the
            // next one is read into the same rgelt slot.
            while fetched < celt { // nolint: dowhile-continue
                let mut error: LSTATUS;
                let mut hr: HRESULT;
                let mut catid: [u16; 39] = [0; 39];
                let mut cName: DWORD = _countof!(catid) as DWORD;
    
                error = RegEnumKeyExW((*this_).m_key, (*this_).m_next_index, catid.as_mut_ptr(), core::ptr::addr_of_mut!(cName), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
                if (error) as u32 != ERROR_SUCCESS && (error) as u32 != ERROR_MORE_DATA {
                    break;
                }
    
                (*this_).m_next_index += 1;
    
                hr = CLSIDFromString(catid.as_ptr(), rgelt);
                if hr < 0 {
                    continue;
                }
    
                fetched += 1;
                rgelt = rgelt.add(1);
            }
        }
    
        if !pceltFetched.is_null() {
            *pceltFetched = fetched;
        }
        if fetched == celt { S_OK } else { S_FALSE }
    }
    pub unsafe fn Skip(&mut self, celt: ULONG) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_next_index += celt;
        return S_OK;
    }
    pub unsafe fn Reset(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_next_index = 0;
        return S_OK;
    }
    pub unsafe fn Clone(&mut self, ppenum: *mut *mut IEnumGUID) -> HRESULT {
        let this_: *mut Self = self;
        if ppenum.is_null() {
            return E_POINTER;
        }
    
        *ppenum = core::ptr::null_mut();
    
        let mut pEnum: *mut CProfilesEnumGuid = core::ptr::null_mut();
        let mut hr: HRESULT = CProfilesEnumGuid::CreateInstance(core::ptr::addr_of_mut!(pEnum));
        if hr >= 0 {
            (*pEnum).m_next_index = (*this_).m_next_index;
            *ppenum = core::ptr::addr_of_mut!((*pEnum).__base0) as *mut IEnumGUID;
        }
    
        hr
    }
}

impl CEnumTfLanguageProfiles {
    pub unsafe fn __ctor_0(&mut self, langid: LANGID) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_cRefs = 1;
        (*this_).m_langid = langid;
        {
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        RegCloseKey((*this_).m_tipkey);
        if (*this_).m_langkey as usize != 0 {
            RegCloseKey((*this_).m_langkey);
        }
        (*(*this_).m_catmgr).Release();
    }
    pub unsafe fn CreateInstance(langid: LANGID, out: *mut *mut CEnumTfLanguageProfiles) -> HRESULT {
        let mut hr: HRESULT;
        let This: *mut CEnumTfLanguageProfiles = CEnumTfLanguageProfiles__new_in_0(cpp_alloc_CicNoThrow::<CEnumTfLanguageProfiles>(), langid);
        if This.is_null() {
            return E_OUTOFMEMORY;
        }
    
        hr = CategoryMgr_Constructor(core::ptr::null_mut(), core::ptr::addr_of_mut!((*This).m_catmgr) as *mut *mut IUnknown);
        if hr < 0 {
            CEnumTfLanguageProfiles__delete(This);
            return hr;
        }
    
        if (RegCreateKeyExW(HKEY_LOCAL_MACHINE, szwSystemTIPKey.as_ptr(), 0, core::ptr::null_mut(), 0,
                            KEY_READ | KEY_WRITE, core::ptr::null_mut(), core::ptr::addr_of_mut!((*This).m_tipkey), core::ptr::null_mut())) as u32 != ERROR_SUCCESS {
            CEnumTfLanguageProfiles__delete(This);
            return E_FAIL;
        }
    
        *out = This;
    
        return S_OK;
    }
    pub unsafe fn QueryInterface(&mut self, iid: &IID, ppvObj: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        *ppvObj = core::ptr::null_mut();
    
        if IsEqualGUID!(&(*iid), &(IID_IUnknown)) || IsEqualGUID!(&(*iid), &(IID_IEnumTfLanguageProfiles)) {
            *ppvObj = core::ptr::addr_of_mut!((*this_).__base0) as LPVOID;
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
            CEnumTfLanguageProfiles__delete(this_);
        }
        return ret;
    }
    pub unsafe fn Clone(&mut self, ppEnum: *mut *mut IEnumTfLanguageProfiles) -> HRESULT {
        let this_: *mut Self = self;
        if ppEnum.is_null() {
            return E_POINTER;
        }
    
        *ppEnum = core::ptr::null_mut();
    
        let mut new_This: *mut CEnumTfLanguageProfiles = core::ptr::null_mut();
        let mut hr: HRESULT = CEnumTfLanguageProfiles::CreateInstance((*this_).m_langid, core::ptr::addr_of_mut!(new_This));
        if (hr < 0) {
            return hr;
        }
    
        (*new_This).m_tip_index = (*this_).m_tip_index;
        lstrcpynW((*new_This).m_szwCurrentClsid.as_mut_ptr(), (*this_).m_szwCurrentClsid.as_ptr(), (_countof!((*new_This).m_szwCurrentClsid)) as i32);
    
        if !(*this_).m_langkey.is_null() {
            let mut fullkey: [WCHAR; 168] = core::mem::zeroed();
            StringCchPrintfW!(fullkey.as_mut_ptr(), _countof!(fullkey), CPP_COM_WSTR_508BF086E991.as_ptr(), (*this_).m_szwCurrentClsid.as_ptr(), CPP_COM_WSTR_98598801D8CA.as_ptr(), (*this_).m_langid as core::ffi::c_uint);
            RegOpenKeyExW((*new_This).m_tipkey, fullkey.as_ptr(), 0, KEY_READ | KEY_WRITE, core::ptr::addr_of_mut!((*new_This).m_langkey));
            (*new_This).m_lang_index = (*this_).m_lang_index;
        }
    
        *ppEnum = core::ptr::addr_of_mut!((*new_This).__base0) as *mut IEnumTfLanguageProfiles;
        return hr;
    }
    pub unsafe fn Next(&mut self, ulCount: ULONG, mut pProfile: *mut TF_LANGUAGEPROFILE, pcFetch: *mut ULONG) -> HRESULT {
        let this_: *mut Self = self;
        let mut fetched: ULONG = 0;
    
        if pProfile.is_null() {
            return E_POINTER;
        }
    
        if !(*this_).m_tipkey.is_null() {
            while fetched < ulCount {
                let error: LSTATUS;
                let hr: HRESULT;
                let mut cName: DWORD = _countof!((*this_).m_szwCurrentClsid) as DWORD;
                let mut clsid: CLSID = core::mem::zeroed();
    
                error = RegEnumKeyExW(
                    (*this_).m_tipkey,
                    (*this_).m_tip_index,
                    (*this_).m_szwCurrentClsid.as_mut_ptr(),
                    core::ptr::addr_of_mut!(cName),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                if (error) as u32 != ERROR_SUCCESS && (error) as u32 != ERROR_MORE_DATA {
                    break;
                }
    
                (*this_).m_tip_index += 1;
    
                hr = CLSIDFromString(
                    (*this_).m_szwCurrentClsid.as_ptr(),
                    core::ptr::addr_of_mut!(clsid),
                );
                if hr < 0 {
                    continue;
                }
    
                while fetched < ulCount {
                    let res: INT = (*this_).next_LanguageProfile(clsid, pProfile);
                    if res == 1 {
                        fetched += 1;
                        pProfile = pProfile.add(1);
                    } else if res == -1 {
                        break;
                    }
                }
            }
        }
    
        if !pcFetch.is_null() {
            *pcFetch = fetched;
        }
        if fetched == ulCount {
            S_OK
        } else {
            S_FALSE
        }
    }
    pub unsafe fn Reset(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_tip_index = 0;
        if !(*this_).m_langkey.is_null() {
            RegCloseKey((*this_).m_langkey);
        }
        (*this_).m_langkey = core::ptr::null_mut();
        (*this_).m_lang_index = 0;
        return S_OK;
    }
    pub unsafe fn Skip(&mut self, celt: ULONG,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("STUB (%p)\n", this_);
        E_NOTIMPL
    }
    pub unsafe fn next_LanguageProfile(&mut self, clsid: CLSID, tflp: *mut TF_LANGUAGEPROFILE) -> INT {
        let this_: *mut Self = self;
        let mut fullkey: [WCHAR; 168 as usize] = [0 as WCHAR; 168 as usize];
        let mut profileid: [WCHAR; 39 as usize] = [0 as WCHAR; 39 as usize];
        let mut error: LSTATUS = 0;
        let mut cName: DWORD = _countof!(profileid) as DWORD;
        let mut profile: GUID = core::mem::zeroed();
    
        if (*this_).m_langkey.is_null()
        {
            StringCchPrintfW!(fullkey.as_mut_ptr(), _countof!(fullkey) as usize, CPP_COM_WSTR_508BF086E991.as_ptr(), (*this_).m_szwCurrentClsid.as_ptr(),
                             CPP_COM_WSTR_98598801D8CA.as_ptr(), (*this_).m_langid as i32);
            error = RegOpenKeyExW((*this_).m_tipkey, fullkey.as_ptr(), 0, KEY_READ | KEY_WRITE, core::ptr::addr_of_mut!((*this_).m_langkey));
            if (error) as u32 != ERROR_SUCCESS
            {
                (*this_).m_langkey = core::ptr::null_mut();
                return -1;
            }
            (*this_).m_lang_index = 0;
        }
    
        error = RegEnumKeyExW((*this_).m_langkey, (*this_).m_lang_index, profileid.as_mut_ptr(), core::ptr::addr_of_mut!(cName), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
        if (error) as u32 != ERROR_SUCCESS && (error) as u32 != ERROR_MORE_DATA
        {
            RegCloseKey((*this_).m_langkey);
            (*this_).m_langkey = core::ptr::null_mut();
            return -1;
        }
        (*this_).m_lang_index += 1;
    
        if !tflp.is_null()
        {
            let tipcats: [*const GUID; 3 as usize] = [&GUID_TFCAT_TIP_KEYBOARD as *const GUID,
                                               &GUID_TFCAT_TIP_SPEECH as *const GUID,
                                               &GUID_TFCAT_TIP_HANDWRITING as *const GUID];
            let hr: HRESULT = CLSIDFromString(profileid.as_ptr(), core::ptr::addr_of_mut!(profile));
            if hr < 0
            {
                return 0;
            }
    
            (*tflp).clsid = clsid;
            (*tflp).langid = (*this_).m_langid;
            (*tflp).fActive = get_active_textservice(&clsid, core::ptr::null_mut());
            (*tflp).guidProfile = profile;
            if (*(*this_).m_catmgr).FindClosestCategory(&clsid, core::ptr::addr_of_mut!((*tflp).catid), tipcats.as_ptr() as *mut *const GUID, 3) != S_OK
            {
                (*(*this_).m_catmgr).FindClosestCategory(&clsid, core::ptr::addr_of_mut!((*tflp).catid), core::ptr::null_mut(), 0);
            }
        }
    
        return 1;
    }
}

impl CEnumTfInputProcessorProfiles {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_cRefs = 1;
        {
        }
    }
    pub unsafe fn QueryInterface(&mut self, iid: &IID, ppvObj: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        if ppvObj.is_null() {
            return E_POINTER;
        }
    
        *ppvObj = core::ptr::null_mut();
        if IsEqualGUID!(&(*iid), &(IID_IUnknown)) || IsEqualGUID!(&(*iid), &(IID_IEnumTfInputProcessorProfiles)) {
            *ppvObj = core::ptr::addr_of_mut!((*this_).__base0) as *mut IEnumTfInputProcessorProfiles as LPVOID;
        }
    
        if !(*ppvObj).is_null() {
            (*this_).AddRef();
            return S_OK;
        }
    
        return E_NOINTERFACE;
    }
    pub unsafe fn AddRef(&mut self) -> ULONG {
        let this_: *mut Self = self;
        (InterlockedIncrement(core::ptr::addr_of_mut!((*this_).m_cRefs))) as u32
    }
    pub unsafe fn Release(&mut self) -> ULONG {
        let this_: *mut Self = self;
        let ret: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) as ULONG;
        if ret == 0 {
            CEnumTfInputProcessorProfiles__delete(this_);
        }
        return ret;
    }
    pub unsafe fn Clone(&mut self, ppEnum: *mut *mut IEnumTfInputProcessorProfiles,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("(%p)->(%p)\n", this_, ppEnum);
        E_NOTIMPL
    }
    pub unsafe fn Next(&mut self, ulCount: ULONG, pProfile: *mut TF_INPUTPROCESSORPROFILE, pcFetch: *mut ULONG) -> HRESULT {
        let this_: *mut Self = self;
        if !pcFetch.is_null() {
            *pcFetch = 0;
        }
        return S_FALSE;
    }
    pub unsafe fn Reset(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("(%p)\n", this_);
        E_NOTIMPL
    }
    pub unsafe fn Skip(&mut self, ulCount: ULONG,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("(%p)->(%u)\n", this_, ulCount);
        E_NOTIMPL
    }
}
