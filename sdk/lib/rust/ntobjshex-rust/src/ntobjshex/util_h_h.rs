use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn MakeStrRetFromString__0(string: LPCWSTR, cbLength: DWORD, str: *mut STRRET) -> HRESULT {
    (*str).uType = STRRET_WSTR;

    let blen: DWORD = cbLength + core::mem::size_of::<WCHAR>() as DWORD;
    (*str).__bindgen_anon_1.pOleStr = CoTaskMemAlloc((blen) as SIZE_T) as LPWSTR;
    return StringCbCopyNW((*str).__bindgen_anon_1.pOleStr, blen as usize, string, cbLength as usize);
}
#[no_mangle]
pub unsafe extern "C" fn MakeStrRetFromString__1(string: LPCWSTR, str: *mut STRRET) -> HRESULT {
    let stringLength: DWORD = (wcslen(string) as DWORD)
        .wrapping_mul(core::mem::size_of::<u16>() as DWORD);
    return MakeStrRetFromString__0(string, stringLength, str);
}
#[no_mangle]
pub unsafe extern "C" fn MakeVariantString(pv: *mut VARIANT, string: PCWSTR) -> HRESULT {
    V_VT!(pv) = (VT_BSTR as u16);
    V_BSTR!(pv) = SysAllocString(string);
    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn GetFullName(pidl: PCIDLIST_ABSOLUTE, uFlags: DWORD, strName: PWSTR, cchName: DWORD) -> HRESULT {
    let mut psfDesktop: CComPtr<IShellFolder> = CComPtr::new();
    let mut str: STRRET = core::mem::zeroed();
    let mut hr: HRESULT;

    hr = SHGetDesktopFolder(psfDesktop.GetAddressOf());
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    hr = (*psfDesktop.Get()).GetDisplayNameOf(pidl, uFlags, core::ptr::addr_of_mut!(str));
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    return StrRetToBufW(core::ptr::addr_of_mut!(str), pidl, strName, cchName);
}
