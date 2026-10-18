use crate::wmiutils::main::IID_IUnknown;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_1DB8216322DF: [u16; 22] = [
    0x0046u16, 0x0061, 0x0063, 0x0069, 0x006C, 0x0069, 0x0074, 0x0079, 0x0020, 0x0063,
    0x006F, 0x0064, 0x0065, 0x003A, 0x0020, 0x0030, 0x0078, 0x0025, 0x0030, 0x0038,
    0x0078, 0x0000,
];
pub static RUSTCONV_WSTR_A05FB45495E6: [u16; 19] = [
    0x0045u16, 0x0072, 0x0072, 0x006F, 0x0072, 0x0020, 0x0063, 0x006F, 0x0064, 0x0065,
    0x003A, 0x0020, 0x0030, 0x0078, 0x0025, 0x0030, 0x0038, 0x0078, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct status_code {
    pub IWbemStatusCodeText_iface: IWbemStatusCodeText,
    pub refs: LONG,
}

pub type Pstatus_code = *mut status_code;

unsafe extern "C" fn impl_from_IWbemStatusCodeText(
    iface: *mut IWbemStatusCodeText,
) -> *mut status_code {
    return (iface as usize).wrapping_sub(
        core::mem::offset_of!(status_code, IWbemStatusCodeText_iface),
    ) as *mut status_code;
}

unsafe extern "system" fn status_code_AddRef(iface: *mut IWbemStatusCodeText) -> ULONG {
    let status_code: *mut status_code = impl_from_IWbemStatusCodeText(iface);
    return InterlockedIncrement(core::ptr::addr_of_mut!((*status_code).refs)) as ULONG;
}
unsafe extern "system" fn status_code_Release(iface: *mut IWbemStatusCodeText) -> ULONG {
    let status_code: *mut status_code = impl_from_IWbemStatusCodeText(iface);
    let refs: LONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*status_code).refs) as *mut ::core::ffi::c_long,
    ) as LONG;
    if refs == 0 {
        free(status_code as *mut ::core::ffi::c_void);
    }
    return refs as ULONG;
}
unsafe extern "system" fn status_code_QueryInterface(
    iface: *mut IWbemStatusCodeText,
    riid: *const IID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This: *mut status_code = impl_from_IWbemStatusCodeText(iface);

    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IWbemStatusCodeText)) != false
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown)) != false
    {
        *ppvObject = iface as *mut core::ffi::c_void;
    } else {
        return E_NOINTERFACE;
    }
    IWbemStatusCodeText_AddRef(iface);
    return S_OK;
}

unsafe extern "system" fn status_code_GetErrorCodeText(
    iface: *mut IWbemStatusCodeText,
    res: HRESULT,
    lcid: LCID,
    flags: LONG,
    text: *mut BSTR,
) -> HRESULT {
    let mut msg: [WCHAR; 32] = [0; 32];

    swprintf(core::ptr::addr_of_mut!(msg) as *mut WCHAR, (ARRAY_SIZE!(msg)) as _, RUSTCONV_WSTR_A05FB45495E6.as_ptr(), res);
    *text = SysAllocString(core::ptr::addr_of_mut!(msg) as *mut WCHAR);
    return (WBEM_S_NO_ERROR) as HRESULT;
}

unsafe extern "system" fn status_code_GetFacilityCodeText(
    iface: *mut IWbemStatusCodeText,
    res: HRESULT,
    lcid: LCID,
    flags: LONG,
    text: *mut BSTR,
) -> HRESULT {
    let mut msg: [WCHAR; 32] = [0; 32];

    swprintf(msg.as_mut_ptr(), (ARRAY_SIZE!(msg)) as _, RUSTCONV_WSTR_1DB8216322DF.as_ptr(), res);
    *text = SysAllocString(msg.as_mut_ptr());
    return (WBEM_S_NO_ERROR) as HRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn WbemStatusCodeText_create(ppObj: *mut LPVOID) -> HRESULT {
    let mut sc: *mut status_code;

    sc = calloc((1 as usize) as _, (core::mem::size_of::<status_code>()) as _) as *mut status_code;
    if sc.is_null() {
        return E_OUTOFMEMORY;
    }

    (*sc).IWbemStatusCodeText_iface.lpVtbl = core::ptr::addr_of!(status_code_vtbl) as _;
    (*sc).refs = 1 as LONG;

    *ppObj = core::ptr::addr_of_mut!((*sc).IWbemStatusCodeText_iface) as LPVOID;

    return S_OK;
}

// COM vtable tables — generated from statuscode.c by vtbl_gen.py
/// statuscode.c:120 `static const IWbemStatusCodeTextVtbl status_code_vtbl`  (slot order: rust:types_amd64.rs)
pub static status_code_vtbl: IWbemStatusCodeTextVtbl = IWbemStatusCodeTextVtbl {
    QueryInterface: Some(status_code_QueryInterface),
    AddRef: Some(status_code_AddRef),
    Release: Some(status_code_Release),
    GetErrorCodeText: Some(status_code_GetErrorCodeText),
    GetFacilityCodeText: Some(status_code_GetFacilityCodeText),
};
