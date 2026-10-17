use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_3D611D143C84: [WCHAR; 6] = [0x0049, 0x006E, 0x0064, 0x0065, 0x0078, 0x0000];
pub static CPP_COM_WSTR_58723767DC4D: [WCHAR; 39] = [0x002F, 0x004D, 0x004D, 0x0043, 0x005F, 0x0043, 0x006F, 0x006E, 0x0073, 0x006F, 0x006C, 0x0065, 0x0046, 0x0069, 0x006C, 0x0065, 0x002F, 0x0056, 0x0069, 0x0073, 0x0075, 0x0061, 0x006C, 0x0041, 0x0074, 0x0074, 0x0072, 0x0069, 0x0062, 0x0075, 0x0074, 0x0065, 0x0073, 0x002F, 0x0049, 0x0063, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_7F8358217CC3: [WCHAR; 6] = [0x0058, 0x0050, 0x0061, 0x0074, 0x0068, 0x0000];
pub static CPP_COM_WSTR_9673FFED05EC: [WCHAR; 18] = [0x0053, 0x0065, 0x006C, 0x0065, 0x0063, 0x0074, 0x0069, 0x006F, 0x006E, 0x004C, 0x0061, 0x006E, 0x0067, 0x0075, 0x0061, 0x0067, 0x0065, 0x0000];
pub static CPP_COM_WSTR_C4BA81AEA105: [WCHAR; 5] = [0x0046, 0x0069, 0x006C, 0x0065, 0x0000];
pub static CPP_COM_WSTR_CF1BD038F83F: [WCHAR; 10] = [0x0025, 0x0073, 0x006D, 0x006D, 0x0063, 0x002E, 0x0065, 0x0078, 0x0065, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// The C preamble contains no file-scope type, constant, enum, or table declarations. It only defines four `extern "C"` functions (`DllCanUnloadNow`, `DllGetClassObject`, `DllRegisterServer`, `DllUnregisterServer`), which are excluded from this pass. Nothing is emitted here.



















unsafe extern "C" fn SetXPathSelectionLanguage(pDoc: *mut IXMLDOMDocument2) -> HRESULT {
    let mut v: VARIANT = core::mem::zeroed();
    V_VT!(core::ptr::addr_of_mut!(v)) = VT_BSTR as VARTYPE;
    V_BSTR!(core::ptr::addr_of_mut!(v)) = CPP_COM_WSTR_7F8358217CC3.as_ptr() as PWSTR; // Use XPath, not the old XSLPattern
    return (*pDoc).setProperty(CPP_COM_WSTR_9673FFED05EC.as_ptr() as PWSTR, v);
}

unsafe extern "C" fn GetAttributeValue(mut pNode: *mut IXMLDOMNode, pszName: LPCWSTR, pOut: *mut BSTR) -> HRESULT {
    let mut pMap: CComPtr<IXMLDOMNamedNodeMap> = CComPtr::new();
    let mut hr: HRESULT;
    *pOut = core::ptr::null_mut();
    hr = (*pNode).get_attributes(pMap.GetAddressOf());
    if hr >= 0 {
        hr = (*pMap.Get()).getNamedItem(pszName as PWSTR, core::ptr::addr_of_mut!(pNode));
        if hr >= 0 {
            hr = HRESULT_FROM_WIN32!(ERROR_NOT_FOUND);
            if !pNode.is_null() {
                hr = (*pNode).get_text(pOut);
                if (hr >= 0) && (*pOut).is_null() {
                    hr = HRESULT_FROM_WIN32!(ERROR_NOT_FOUND);
                }
                (*pNode).Release();
            }
        }
    }
    return hr;
}

unsafe extern "C" fn LoadXmlFromVariant(pVar: *mut VARIANT, ppDoc: *mut *mut IXMLDOMDocument2) -> HRESULT {
    let mut succ: VARIANT_BOOL = VARIANT_FALSE;
    let mut hr: HRESULT = CoCreateInstance(&CLSID_DOMDocument30, core::ptr::null_mut(), CLSCTX_INPROC_SERVER as DWORD, &IID_IXMLDOMDocument, ppDoc as *mut *mut core::ffi::c_void);
    if hr < 0 {
        return hr;
    } else {
        hr = (*(*ppDoc)).load(*pVar, core::ptr::addr_of_mut!(succ));
        if hr >= 0 && succ != 0 {
            return hr;
        }
    }
    (*(*ppDoc)).Release();
    return if hr >= 0 { E_FAIL } else { hr };
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    return g_Module.DllCanUnloadNow();
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const CLSID,
    riid: *const CLSID,
    ppv: *mut LPVOID,
) -> HRESULT {
    return (*core::ptr::addr_of_mut!(g_Module)).DllGetClassObject(rclsid, riid, ppv);
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    return g_Module.DllRegisterServer(FALSE as BOOL);
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    return g_Module.DllUnregisterServer(FALSE as BOOL);
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hInstance: HINSTANCE, dwReason: DWORD, lpReserved: LPVOID) -> BOOL {
    match dwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hInstance);
            g_Module.Init(ObjectMap.as_ptr(), hInstance, core::ptr::null::<c_void>());
        }
        _ => {}
    }
    return TRUE as BOOL;
}

impl CMscExtractIcon {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_File = core::ptr::null_mut();
        {
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        SHFree((*this_).m_File as *mut core::ffi::c_void);
    }
    pub unsafe fn GetIconLocationFromMsc(&mut self, pszIconFile: PWSTR, cchMax: UINT, piIndex: *mut i32) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_File.is_null() {
            return E_UNEXPECTED;
        }

        let mut pStream: CComPtr<IStream> = CComPtr::new();
        let mut hr: HRESULT = SHCreateStreamOnFileW((*this_).m_File, STGM_READ | STGM_SHARE_DENY_WRITE, pStream.GetAddressOf() as *mut *mut IStream);
        if hr < 0 {
            return hr;
        }

        let mut v: VARIANT = core::mem::zeroed();
        V_VT!(core::ptr::addr_of_mut!(v)) = VT_UNKNOWN as VARTYPE;
        V_UNKNOWN!(core::ptr::addr_of_mut!(v)) = pStream.Get() as *mut IUnknown;
        let mut pDoc: CComPtr<IXMLDOMDocument2> = CComPtr::new();
        hr = LoadXmlFromVariant(core::ptr::addr_of_mut!(v), pDoc.GetAddressOf());
        if hr < 0 {
            return hr;
        }
        SetXPathSelectionLanguage(pDoc.Get());

        let mut pDomNode: CComPtr<IXMLDOMNode> = CComPtr::new();
        hr = (*pDoc.Get()).selectSingleNode(CPP_COM_WSTR_58723767DC4D.as_ptr() as PWSTR, pDomNode.GetAddressOf() as *mut *mut IXMLDOMNode);
        if hr < 0 {
            return hr;
        }
        if hr != S_OK { // Some files have an empty <VisualAttributes/> element
            return HRESULT_FROM_WIN32!(ERROR_NOT_FOUND);
        }

        let mut bstr: BSTR = core::ptr::null_mut();
        *piIndex = 0;
        if GetAttributeValue(pDomNode.Get(), CPP_COM_WSTR_3D611D143C84.as_ptr(), core::ptr::addr_of_mut!(bstr)) >= 0 {
            *piIndex = StrToIntW(bstr);
            SysFreeString(bstr);
        }

        hr = GetAttributeValue(pDomNode.Get(), CPP_COM_WSTR_C4BA81AEA105.as_ptr(), core::ptr::addr_of_mut!(bstr));
        if hr >= 0 {
            hr = StringCchCopyW(pszIconFile, cchMax as usize, bstr);
            SysFreeString(bstr);
        }
        return hr;
    }
    pub unsafe fn GetDefaultIconLocation(&mut self, pszIconFile: PWSTR, cchMax: UINT, piIndex: *mut i32) -> HRESULT {
        let this_: *mut Self = self;
        *piIndex = 0;
        let mut szDir: [u16; MAX_PATH as usize + 1 + 8 + 3] = [0; MAX_PATH as usize + 1 + 8 + 3];
        GetSystemDirectory(szDir.as_mut_ptr(), MAX_PATH);
        PathAddBackslashW(szDir.as_mut_ptr());
        return if StringCchPrintfW!(pszIconFile, cchMax, CPP_COM_WSTR_CF1BD038F83F.as_ptr(), szDir.as_ptr()) == S_OK { S_OK } else { S_FALSE };
    }
    pub unsafe fn Extract(&mut self, pszFile: PCWSTR,
        nIconIndex: UINT,
        phiconLarge: *mut HICON,
        phiconSmall: *mut HICON,
        nIconSize: UINT,
    ) -> HRESULT {
        let this_: *mut Self = self;
        S_FALSE
    }
    pub unsafe fn GetIconLocation(&mut self, GilIn: UINT, pszIconFile: PWSTR, cchMax: UINT, piIndex: *mut i32, GilOut: *mut UINT) -> HRESULT {
        let this_: *mut Self = self;
        *GilOut = GIL_PERINSTANCE; // Every .msc file has a unique icon

        if (GilIn & GIL_ASYNC) != 0 {
            return E_PENDING;
        }

        if ((GilIn & GIL_DEFAULTICON) != 0) || (((*this_).GetIconLocationFromMsc(pszIconFile, cchMax, piIndex)) < 0) {
            return (*this_).GetDefaultIconLocation(pszIconFile, cchMax, piIndex);
        }
        return S_OK;
    }
    pub unsafe fn GetClassID(&mut self, pClassID: *mut CLSID,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn IsDirty(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn Load(&mut self, pszFileName: LPCOLESTR, dwMode: DWORD) -> HRESULT {
        let this_: *mut Self = self;
            SHFree((*this_).m_File as LPVOID);
            (*this_).m_File = core::ptr::null_mut();
            return SHStrDupW(pszFileName, core::ptr::addr_of_mut!((*this_).m_File));
        }
    pub unsafe fn Save(&mut self, pszFileName: LPCOLESTR,
        fRemember: BOOL,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn SaveCompleted(&mut self, pszFileName: LPCOLESTR,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn GetCurFile(&mut self, ppszFileName: *mut LPOLESTR,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
}
