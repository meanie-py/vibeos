// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_3527A8AEEB67: [u16; 10] = [0x0052u16, 0x006F, 0x0073, 0x004C, 0x0061, 0x0075, 0x006E, 0x0063, 0x0068, 0x0000];
pub static RUSTCONV_WSTR_831513BFA930: [u16; 16] = [
    0x004Du16, 0x004D, 0x0043, 0x005F, 0x0043, 0x006F, 0x006E, 0x0073, 0x006F, 0x006C,
    0x0065, 0x0046, 0x0069, 0x006C, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_9EADC9D3A146: [u16; 11] = [
    0x0050u16, 0x0061, 0x0072, 0x0061, 0x006D, 0x0065, 0x0074, 0x0065, 0x0072, 0x0073,
    0x0000,
];
pub static RUSTCONV_WSTR_C4BA81AEA105: [u16; 5] = [0x0046u16, 0x0069, 0x006C, 0x0065, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut CLSID_DOMDocument30: GUID = _GUID {
    Data1: 0xf5078f32 as ::core::ffi::c_uint,
    Data2: 0xc551 as ::core::ffi::c_ushort,
    Data3: 0x11d3 as ::core::ffi::c_ushort,
    Data4: [
        0x89 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x21 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

unsafe extern "C" fn xmldomnode_getattributevalue(mut pnode: *mut IXMLDOMNode, name: LPCWSTR, pout: *mut BSTR) -> HRESULT {
    let mut pmap: *mut IXMLDOMNamedNodeMap = core::ptr::null_mut();
    let mut hr: HRESULT = E_OUTOFMEMORY;
    let mut bsname: BSTR = SysAllocString(name as *const OLECHAR);
    *pout = core::ptr::null_mut();
    if !bsname.is_null() && {
        hr = IXMLDOMNode_get_attributes(pnode, core::ptr::addr_of_mut!(pmap));
        hr >= 0
    } {
        if {
            hr = IXMLDOMNamedNodeMap_getNamedItem(pmap, bsname, core::ptr::addr_of_mut!(pnode));
            hr >= 0
        } {
            hr = HRESULT_FROM_WIN32!(ERROR_NOT_FOUND);
            if !pnode.is_null() {
                hr = IXMLDOMNode_get_text(pnode, pout);
                if hr >= 0 && (*pout).is_null() {
                    hr = HRESULT_FROM_WIN32!(ERROR_NOT_FOUND);
                }
                IXMLDOMNode_Release(pnode);
            }
        }
        IXMLDOMNamedNodeMap_Release(pmap);
    }
    SysFreeString(bsname);
    return hr;
}

unsafe extern "C" fn xmldomelem_getelembytag(pelem: *mut IXMLDOMElement, name: LPCWSTR, index: ::core::ffi::c_long, ppout: *mut *mut IXMLDOMNode) -> HRESULT {
    let mut hr: HRESULT = E_OUTOFMEMORY;
    let mut pnl: *mut IXMLDOMNodeList = ::core::ptr::null_mut::<IXMLDOMNodeList>();
    let mut bsname: BSTR = SysAllocString(name as *const u16);
    *ppout = ::core::ptr::null_mut::<IXMLDOMNode>();
    if !bsname.is_null() && {
        hr = IXMLDOMElement_getElementsByTagName(pelem, bsname, ::core::ptr::addr_of_mut!(pnl));
        hr >= 0
    } {
        hr = IXMLDOMNodeList_get_item(pnl, index, ppout);
        if hr >= 0 && (*ppout).is_null() {
            hr = HRESULT_FROM_WIN32!(ERROR_NO_MORE_ITEMS);
        }
        IUnknown_Release((pnl).cast());
    }
    SysFreeString(bsname);
    return hr;
}

unsafe extern "C" fn xmldomunk_getelembytag(punk: *mut IUnknown, name: LPCWSTR, index: i32, ppout: *mut *mut IXMLDOMNode) -> HRESULT {
    let mut pelem: *mut IXMLDOMElement = core::ptr::null_mut();
    let mut hr: HRESULT = IUnknown_QueryInterface(punk, core::ptr::addr_of!(IID_IXMLDOMElement), (&mut pelem as *mut *mut IXMLDOMElement as *mut core::ffi::c_void).cast());
    if ((hr) < 0) {
        *ppout = core::ptr::null_mut();
        return hr;
    }
    hr = xmldomelem_getelembytag(pelem, name, index, ppout);
    IUnknown_Release((pelem).cast());
    return hr;
}

unsafe extern "C" fn LoadXmlFromVariant(pVarUrl: *mut VARIANT, ppDocElm: *mut *mut IXMLDOMElement) -> HRESULT {
    let mut succ: VARIANT_BOOL = VARIANT_FALSE;
    let mut pDoc: *mut IXMLDOMDocument = core::ptr::null_mut();
    let mut hr: HRESULT = CoCreateInstance(
        core::ptr::addr_of!(CLSID_DOMDocument30),
        core::ptr::null_mut(),
        CLSCTX_INPROC_SERVER,
        core::ptr::addr_of!(IID_IXMLDOMDocument),
        core::ptr::addr_of_mut!(pDoc) as *mut LPVOID,
    );
    if hr < 0 {
        return hr;
    } else {
        hr = IXMLDOMDocument_load(pDoc, *pVarUrl, core::ptr::addr_of_mut!(succ));
        if hr >= 0 && succ != 0 {
            hr = IXMLDOMDocument_get_documentElement(pDoc, ppDocElm);
        } else if hr >= 0 {
            hr = E_FAIL;
        }
    }
    IUnknown_Release(pDoc as *mut IUnknown);
    return hr;
}

unsafe extern "C" fn GetMscRootNode(pszFilePath: PCWSTR, ppRoot: *mut *mut IXMLDOMNode) -> HRESULT {
    let mut hr: HRESULT = 0;
    let len: DWORD = lstrlenW(pszFilePath as LPCWSTR) as DWORD;
    let mut cch: DWORD = (core::mem::size_of::<[core::ffi::c_char; 9]>() as DWORD)
        .wrapping_add(len.wrapping_mul(2));
    let pszUrl: PWSTR = LocalAlloc(
        LPTR as UINT,
        (cch as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as PWSTR;
    if pszUrl.is_null() {
        return E_OUTOFMEMORY;
    }
    hr = UrlCreateFromPathW(
        pszFilePath as LPCWSTR,
        pszUrl as LPWSTR,
        &raw mut cch,
        0 as DWORD,
    );
    if hr >= 0 {
        let mut pDocElm: *mut IXMLDOMElement = core::ptr::null_mut::<IXMLDOMElement>();
        let bsurl: BSTR = SysAllocString(pszUrl as *const OLECHAR);
        let mut v: VARIANT = core::mem::zeroed::<VARIANT>();
        v.n1.n2.vt = VT_BSTR as VARTYPE;
        v.n1.n2.n3.bstrVal = bsurl;
        hr = if !bsurl.is_null() {
            LoadXmlFromVariant(&raw mut v, &raw mut pDocElm)
        } else {
            E_OUTOFMEMORY
        };
        VariantClear(&raw mut v);
        LocalFree(pszUrl as HLOCAL);
        if hr >= 0 {
            let mut bs: BSTR = core::ptr::null_mut();
            hr = IXMLDOMElement_get_nodeName(pDocElm, &raw mut bs);
            if hr >= 0 {
                hr = if lstrcmpiW(bs as LPCWSTR, RUSTCONV_WSTR_831513BFA930.as_ptr()) != 0 {
                    HRESULT_FROM_WIN32!(ERROR_BAD_FORMAT)
                } else {
                    S_OK
                };
                SysFreeString(bs);
                if hr >= 0 {
                    hr = IUnknown_QueryInterface(
                        (pDocElm as *mut core::ffi::c_void).cast(),
                        &raw const IID_IXMLDOMNode,
                        ppRoot as *mut *mut core::ffi::c_void,
                    );
                }
            }
            IUnknown_Release((pDocElm as *mut core::ffi::c_void).cast());
        }
    }
    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn HandleRosMscLaunch(pszCmdLine: PCWSTR) -> BOOL {
    let mut result: BOOL = (FALSE) as i32;
    let mut pRoot: *mut IXMLDOMNode = core::ptr::null_mut();
    let mut hrCom: HRESULT = CoInitialize(core::ptr::null_mut());

    let mut argc: ::core::ffi::c_int = 0;
    let mut argv: *mut LPWSTR = CommandLineToArgvW(pszCmdLine as LPCWSTR, &raw mut argc);
    if !argv.is_null() && argc > 0 {
        if GetMscRootNode(
            *argv.offset(0 as isize) as PCWSTR,
            &raw mut pRoot,
        ) >= 0
        {
            let mut pNode: *mut IXMLDOMNode = core::ptr::null_mut();
            let mut hr: HRESULT = xmldomunk_getelembytag(
                pRoot as *mut IUnknown,
                RUSTCONV_WSTR_3527A8AEEB67.as_ptr(),
                0,
                &raw mut pNode,
            );
            IUnknown_Release(pRoot as *mut IUnknown);
            if hr >= 0 {
                let mut flags: UINT = SEE_MASK_DOENVSUBST | SEE_MASK_FLAG_DDEWAIT | SEE_MASK_FLAG_NO_UI;
                let mut sei: SHELLEXECUTEINFOW = _SHELLEXECUTEINFOW {
                    cbSize: ::core::mem::size_of::<SHELLEXECUTEINFOW>() as DWORD,
                    fMask: flags as ULONG,
                    hwnd: core::ptr::null_mut(),
                    lpVerb: core::ptr::null(),
                    lpFile: core::ptr::null(),
                    lpParameters: core::ptr::null(),
                    lpDirectory: core::ptr::null(),
                    nShow: (SW_SHOW) as i32,
                    hInstApp: core::ptr::null_mut(),
                    lpIDList: core::ptr::null_mut(),
                    lpClass: core::ptr::null(),
                    hkeyClass: core::ptr::null_mut(),
                    dwHotKey: 0,
                    __bindgen_anon_1: _SHELLEXECUTEINFOW__bindgen_ty_1 { hIcon: core::ptr::null_mut(),
                    },
                    hProcess: core::ptr::null_mut(),
                };
                if xmldomnode_getattributevalue(
                    pNode,
                    RUSTCONV_WSTR_C4BA81AEA105.as_ptr(),
                    &raw mut sei.lpFile as *mut BSTR,
                ) >= 0
                {
                    xmldomnode_getattributevalue(
                        pNode,
                        RUSTCONV_WSTR_9EADC9D3A146.as_ptr(),
                        &raw mut sei.lpParameters as *mut BSTR,
                    );
                    result = (!sei.lpFile.is_null()
                        && *sei.lpFile != 0
                        && ShellExecuteExW(&raw mut sei) != 0) as ::core::ffi::c_int as BOOL;
                    SysFreeString(sei.lpFile as BSTR);
                    SysFreeString(sei.lpParameters as BSTR);
                }
                IUnknown_Release(pNode as *mut IUnknown);
            }
        }
        LocalFree(argv as HLOCAL);
    }

    if hrCom >= 0 {
        CoUninitialize();
    }
    return result;
}
