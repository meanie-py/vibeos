use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_09A7C6FF8E65: [WCHAR; 8] = [0x0044, 0x0069, 0x0073, 0x0070, 0x006C, 0x0061, 0x0079, 0x0000];
pub static CPP_COM_WSTR_211520423D8F: [WCHAR; 11] = [0x0053, 0x0074, 0x0061, 0x0074, 0x0065, 0x0046, 0x006C, 0x0061, 0x0067, 0x0073, 0x0000];
pub static CPP_COM_WSTR_24D82B636CA0: [WCHAR; 34] = [0x0025, 0x0073, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0072, 0x006F, 0x006F, 0x0074, 0x0025, 0x005C, 0x0073, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0033, 0x0032, 0x005C, 0x0073, 0x0068, 0x0065, 0x006C, 0x006C, 0x0033, 0x0032, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000];
pub static CPP_COM_WSTR_629ADA403786: [WCHAR; 9] = [0x0050, 0x0072, 0x0069, 0x006F, 0x0072, 0x0069, 0x0074, 0x0079, 0x0000];
pub static CPP_COM_WSTR_6AC123285056: [WCHAR; 12] = [0x0044, 0x0065, 0x0073, 0x0063, 0x0072, 0x0069, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_78242CF2BDD3: [WCHAR; 12] = [0x0050, 0x0072, 0x006F, 0x0070, 0x0065, 0x0072, 0x0074, 0x0079, 0x0042, 0x0061, 0x0067, 0x0000];
pub static CPP_COM_WSTR_BB388302883B: [WCHAR; 19] = [0x0041, 0x0064, 0x0076, 0x0061, 0x006E, 0x0063, 0x0065, 0x0064, 0x0042, 0x0075, 0x0074, 0x0074, 0x006F, 0x006E, 0x0054, 0x0065, 0x0078, 0x0074, 0x0000];
pub static CPP_COM_WSTR_BB7A045AEFB4: [WCHAR; 9] = [0x0049, 0x0063, 0x006F, 0x006E, 0x0050, 0x0061, 0x0074, 0x0068, 0x0000];
pub static CPP_COM_WSTR_F817F5B94934: [WCHAR; 21] = [0x0043, 0x004C, 0x0053, 0x0049, 0x0044, 0x005C, 0x0025, 0x0073, 0x005C, 0x0044, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0049, 0x0063, 0x006F, 0x006E, 0x0000];

use core::ffi::c_void;








impl CCleanupHandler {
    pub unsafe fn __ctor_0(&mut self, subKey: &mut CRegKey, keyName: &CStringW, guid: &GUID) {
        let this_: *mut Self = self;
        (*this_).hSubKey = core::ptr::read(subKey);
        (*this_).KeyName = CStringW::from_ptr((*keyName).GetString());
        (*this_).Guid = *guid;
        (*this_).dwFlags = 0;
        (*this_).Priority = 0;
        (*this_).StateFlags = 0;
        (*this_).SpaceUsed = 0;
        (*this_).ShowHandler = true;
        (*this_).hIcon = core::ptr::null_mut();
        {
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).Deactivate();
        DestroyIcon((*this_).hIcon);
    }
    pub unsafe fn Deactivate(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).Handler.p.is_null() {
            let mut dwFlags: DWORD = 0;
            (*(*this_).Handler.p).Deactivate(core::ptr::addr_of_mut!(dwFlags));
            if (dwFlags & EVCF_REMOVEFROMLIST) != 0 {
                UNIMPLEMENTED_DBGBREAK!();
            }
        }
    }
    pub unsafe fn Initialize(&mut self, pcwszVolume: LPCWSTR) -> bool {
        let this_: *mut Self = self;
        if FAILED_UNEXPECTEDLY!(CoCreateInstance(
            core::ptr::addr_of!((*this_).Guid),
            core::ptr::null_mut(),
            CLSCTX_INPROC_SERVER,
            core::ptr::addr_of!(IID_IEmptyVolumeCache),
            core::ptr::addr_of_mut!((*this_).Handler) as *mut *mut core::ffi::c_void,
        )) {
            return false;
        }
    
        let mut dwSize: DWORD = core::mem::size_of::<DWORD>() as DWORD;
        if (*this_).hSubKey.QueryBinaryValue(
            CPP_COM_WSTR_629ADA403786.as_ptr(),
            core::ptr::addr_of_mut!((*this_).Priority) as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(dwSize),
        ) != (ERROR_SUCCESS as i32)
        {
            if (*this_).hSubKey.QueryDWORDValue(CPP_COM_WSTR_629ADA403786.as_ptr(), &mut (*this_).Priority) != (ERROR_SUCCESS as i32) {
                (*this_).Priority = 200;
            }
        }
    
        dwSize = core::mem::size_of::<DWORD>() as DWORD;
        if (*this_).hSubKey.QueryDWORDValue(CPP_COM_WSTR_211520423D8F.as_ptr(), &mut (*this_).StateFlags) != (ERROR_SUCCESS as i32) {
            (*this_).StateFlags = 0;
        }
    
        let mut PathBuffer = [0 as WCHAR; (MAX_PATH) as _];
        let mut nChars: ULONG = _countof!(PathBuffer);
        if (*this_).hSubKey.QueryStringValue(
            CPP_COM_WSTR_BB7A045AEFB4.as_ptr(),
            PathBuffer.as_mut_ptr(),
            core::ptr::addr_of_mut!(nChars),
        ) != (ERROR_SUCCESS as i32)
        {
            let mut Tmp = CStringW::new();
            let mut GuidStr = [0 as WCHAR; 50];
            if StringFromGUID2(
                core::ptr::addr_of!((*this_).Guid),
                GuidStr.as_mut_ptr(),
                (_countof!(GuidStr)) as INT,
            ) != 0
            {
                atl_format!(Tmp, CPP_COM_WSTR_F817F5B94934.as_ptr(), GuidStr.as_ptr());
                let mut clsid = CRegKey__make_0();
                nChars = _countof!(PathBuffer);
                if clsid.Open(HKEY_CLASSES_ROOT, Tmp.GetString(), KEY_READ) != (ERROR_SUCCESS as i32)
                    || clsid.QueryStringValue(
                        core::ptr::null(),
                        PathBuffer.as_mut_ptr(),
                        core::ptr::addr_of_mut!(nChars),
                    ) != (ERROR_SUCCESS as i32)
                {
                    PathBuffer[0] = UNICODE_NULL;
                }
            }
        }
        if PathBuffer[0] == 0 {
            StringCchCopyW(
                PathBuffer.as_mut_ptr(),
                (_countof!(PathBuffer)) as _,
                CPP_COM_WSTR_24D82B636CA0.as_ptr(),
            );
        }
    
        let mut Index: i32 = 0;
        let mut ptr: *mut WCHAR = wcschr(PathBuffer.as_mut_ptr(), 0x002Cu16) as *mut WCHAR;
        if !ptr.is_null() {
            *ptr = UNICODE_NULL;
            ptr = ptr.add(1);
            Index = wcstol(ptr, core::ptr::null_mut(), 10) as i32;
        }
    
        let mut Large: HICON = core::mem::zeroed();
        let mut Small: HICON = core::mem::zeroed();
        let mut Result: UINT = ExtractIconExW(
            PathBuffer.as_mut_ptr(),
            Index,
            core::ptr::addr_of_mut!(Large),
            core::ptr::addr_of_mut!(Small),
            1,
        );
        if Result < 1 {
            Result = ExtractIconExW(
                CPP_COM_WSTR_24D82B636CA0.as_ptr(),
                0,
                core::ptr::addr_of_mut!(Large),
                core::ptr::addr_of_mut!(Small),
                1,
            );
        }
        if Result >= 1 {
            (*this_).hIcon = Small;
            if (*this_).hIcon.is_null() {
                (*this_).hIcon = Large;
            } else {
                DestroyIcon(Large);
            }
        }
    
        // These options should come from the command line
        // dwFlags |= EVCF_SETTINGSMODE;
        // dwFlags |= EVCF_OUTOFDISKSPACE;
    
        let mut spHandler2: CComPtr<IEmptyVolumeCache2> = CComPtr::new();
        let mut hr: HRESULT = (*(*this_).Handler.Get()).QueryInterface(
            core::ptr::addr_of!(IID_IEmptyVolumeCache2),
            spHandler2.GetAddressOf() as *mut *mut core::ffi::c_void,
        );
        if hr >= 0 {
            hr = (*spHandler2.Get()).InitializeEx(
                (*this_).hSubKey.op_to_HKEY(),
                pcwszVolume,
                (*this_).KeyName.GetString(),
                core::ptr::addr_of_mut!((*this_).wszDisplayName) as *mut *mut WCHAR,
                core::ptr::addr_of_mut!((*this_).wszDescription) as *mut *mut WCHAR,
                core::ptr::addr_of_mut!((*this_).wszBtnText) as *mut *mut WCHAR,
                core::ptr::addr_of_mut!((*this_).dwFlags),
            );
            if FAILED_UNEXPECTEDLY!(hr) {
                return false;
            }
    
            // No files to clean will return S_FALSE;
            if hr != S_OK {
                return false;
            }
        } else {
            // Observed behavior:
            // When Initialize is called, wszDescription is actually pointing to data
            // wszDescription.AllocateBytes(0x400u);
            hr = (*(*this_).Handler.Get()).Initialize(
                (*this_).hSubKey.op_to_HKEY(),
                pcwszVolume,
                core::ptr::addr_of_mut!((*this_).wszDisplayName) as *mut *mut WCHAR,
                core::ptr::addr_of_mut!((*this_).wszDescription) as *mut *mut WCHAR,
                core::ptr::addr_of_mut!((*this_).dwFlags),
            );
            if FAILED_UNEXPECTEDLY!(hr) {
                return false;
            }
    
            // No files to clean will return S_FALSE;
            if hr != S_OK {
                return false;
            }
    
            let mut spBag: CComPtr<IPropertyBag> = CComPtr::new();
            let mut GuidStr = [0 as WCHAR; 50];
            nChars = _countof!(GuidStr);
            if (*this_).hSubKey.QueryStringValue(
                CPP_COM_WSTR_78242CF2BDD3.as_ptr(),
                GuidStr.as_mut_ptr(),
                core::ptr::addr_of_mut!(nChars),
            ) == (ERROR_SUCCESS as i32)
            {
                let mut guid: GUID = core::mem::zeroed();
                if !FAILED_UNEXPECTEDLY!(CLSIDFromString(
                    GuidStr.as_ptr(),
                    core::ptr::addr_of_mut!(guid),
                )) {
                    FAILED_UNEXPECTEDLY!(CoCreateInstance(
                        core::ptr::addr_of!(guid),
                        core::ptr::null_mut(),
                        CLSCTX_INPROC_SERVER,
                        core::ptr::addr_of!(IID_IPropertyBag),
                        spBag.GetAddressOf() as *mut *mut core::ffi::c_void,
                    ));
                }
            }
            (*this_).ReadProperty(
                CPP_COM_WSTR_09A7C6FF8E65.as_ptr(),
                spBag.Get(),
                &mut *core::ptr::addr_of_mut!((*this_).wszDisplayName),
            );
            (*this_).ReadProperty(
                CPP_COM_WSTR_6AC123285056.as_ptr(),
                spBag.Get(),
                &mut *core::ptr::addr_of_mut!((*this_).wszDescription),
            );
    
            if ((*this_).dwFlags & EVCF_HASSETTINGS) != 0 {
                (*this_).ReadProperty(
                    CPP_COM_WSTR_BB388302883B.as_ptr(),
                    spBag.Get(),
                    &mut *core::ptr::addr_of_mut!((*this_).wszBtnText),
                );
            }
        }
    
        if ((*this_).dwFlags & EVCF_ENABLEBYDEFAULT) != 0 && ((*this_).StateFlags & (HANDLER_STATE_SELECTED as u32)) == 0 {
            (*this_).StateFlags |= (HANDLER_STATE_SELECTED as u32);
        }
    
        // For convenience
        if (*this_).wszDisplayName.is_null() {
            SHStrDupW(
                (*this_).KeyName.GetString(),
                core::ptr::addr_of_mut!((*this_).wszDisplayName) as *mut *mut WCHAR,
            );
        }
    
        true
    }
    pub unsafe fn ReadProperty(&mut self, Name: LPCWSTR, pBag: *mut IPropertyBag, storage: &mut CComHeapPtr<WCHAR>) {
        let this_: *mut Self = self;
        if !storage.is_null() {
            return;
        }
    
        if !pBag.is_null() {
            // `CComVariant tmp;` -- a VARIANT that ~CComVariant VariantClear()s on scope exit,
            // written out: atl_rust has no CComVariant.
            let mut tmp: VARIANT = core::mem::zeroed();
            tmp.__bindgen_anon_1.__bindgen_anon_1.vt = VT_BSTR as VARTYPE;
            let hr: HRESULT = (*pBag).Read(Name, core::ptr::addr_of_mut!(tmp), core::ptr::null_mut());
            if !FAILED_UNEXPECTEDLY!(hr) && tmp.__bindgen_anon_1.__bindgen_anon_1.vt == VT_BSTR as VARTYPE {
                SHStrDupW(tmp.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.bstrVal, storage.put());
            }
            VariantClear(core::ptr::addr_of_mut!(tmp));
        }
    
        if storage.is_null() {
            let mut TmpStr: [WCHAR; 0x200] = [0; 0x200];
            let mut dwSize: DWORD = _countof!(TmpStr) as DWORD;
    
            if (*this_).hSubKey.QueryStringValue(Name, TmpStr.as_mut_ptr(), core::ptr::addr_of_mut!(dwSize)) == (ERROR_SUCCESS as i32) {
                let mut ResolvedStr: [WCHAR; 0x200] = [0; 0x200];
                SHLoadIndirectString(TmpStr.as_ptr(), ResolvedStr.as_mut_ptr(), _countof!(ResolvedStr) as u32, core::ptr::null_mut());
                SHStrDupW(ResolvedStr.as_ptr(), storage.put());
            }
        }
    }
    pub unsafe fn HasSettings(&mut self) -> BOOL {
        let this_: *mut Self = self;
        return (((*this_).dwFlags & EVCF_HASSETTINGS) != 0) as BOOL;
    }
    pub unsafe fn DontShowIfZero(&mut self) -> BOOL {
        let this_: *mut Self = self;
        return if ((*this_).dwFlags & EVCF_DONTSHOWIFZERO) != 0 { 1 } else { 0 };
    }
}
