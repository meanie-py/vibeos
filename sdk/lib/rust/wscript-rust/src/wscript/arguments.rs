use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut argums: *mut *mut WCHAR = ::core::ptr::null::<*mut WCHAR>() as *mut *mut WCHAR;

#[no_mangle]
pub static mut numOfArgs: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut arguments_obj: IArguments2 = unsafe {
    IArguments2 {
        lpVtbl: &raw const Arguments2Vtbl,
    }
};

unsafe extern "system" fn Arguments2_QueryInterface(iface: *mut IArguments2, riid: *const GUID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    WINE_TRACE!(b"(%s %p)\n\0".as_ptr() as *const i8, wine_dbgstr_guid(riid), ppv);

    if IsEqualGUID!(core::ptr::addr_of!(IID_IUnknown), riid) != false
       || IsEqualGUID!(core::ptr::addr_of!(IID_IDispatch), riid) != false
       || IsEqualGUID!(core::ptr::addr_of!(IID_IArguments2), riid) != false {
        *ppv = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    *ppv = core::ptr::null_mut();
    return E_NOINTERFACE;
}
unsafe extern "system" fn Arguments2_AddRef(iface: *mut IArguments2) -> ULONG {
    2
}
unsafe extern "system" fn Arguments2_Release(iface: *mut IArguments2) -> ULONG {
    1
}
unsafe extern "system" fn Arguments2_GetTypeInfoCount(iface: *mut IArguments2, pctinfo: *mut UINT) -> HRESULT {
    WINE_TRACE!(b"(%p)\n\0".as_ptr() as *const i8, pctinfo);

    *pctinfo = 1;
    return S_OK;
}

unsafe extern "system" fn Arguments2_GetTypeInfo(
    iface: *mut IArguments2,
    iTInfo: UINT,
    lcid: LCID,
    ppTInfo: *mut *mut ITypeInfo,
) -> HRESULT {
    WINE_TRACE!(
        b"(%x %x %p\n\0".as_ptr() as *const i8,
        iTInfo,
        lcid,
        ppTInfo,
    );

    ITypeInfo_AddRef(arguments_ti);
    *ppTInfo = arguments_ti;
    return S_OK;
}

unsafe extern "system" fn Arguments2_GetIDsOfNames(
    iface: *mut IArguments2,
    riid: *const IID,
    rgszNames: *mut LPOLESTR,
    cNames: UINT,
    lcid: LCID,
    rgDispId: *mut DISPID,
) -> HRESULT {
    WINE_TRACE!(
        b"(%s %p %d %x %p)\n\0".as_ptr() as *const i8,
        wine_dbgstr_guid(riid),
        rgszNames,
        cNames,
        lcid,
        rgDispId,
    );

    return ITypeInfo_GetIDsOfNames(arguments_ti, rgszNames, cNames, rgDispId as *mut MEMBERID);
}

unsafe extern "system" fn Arguments2_Invoke(
    iface: *mut IArguments2,
    dispIdMember: DISPID,
    riid: *const IID,
    lcid: LCID,
    wFlags: WORD,
    pDispParams: *mut DISPPARAMS,
    pVarResult: *mut VARIANT,
    pExcepInfo: *mut EXCEPINFO,
    puArgErr: *mut UINT,
) -> HRESULT {
    WINE_TRACE!(
        b"(%d %p %p)\n\0".as_ptr() as *const i8,
        dispIdMember,
        pDispParams,
        pVarResult,
    );

    return ITypeInfo_Invoke(
        arguments_ti,
        iface as PVOID,
        dispIdMember as MEMBERID,
        wFlags,
        pDispParams,
        pVarResult,
        pExcepInfo,
        puArgErr,
    );
}

unsafe extern "system" fn Arguments2_Item(iface: *mut IArguments2, index: LONG, out_Value: *mut BSTR) -> HRESULT {
    WINE_TRACE!(b"(%d %p)\n\0".as_ptr() as *const i8, index, out_Value);

    if index < 0 as LONG || index >= numOfArgs as LONG {
        return E_INVALIDARG;
    }
    *out_Value = SysAllocString(*argums.offset(index as isize));
    if (*out_Value).is_null() {
        return E_OUTOFMEMORY;
    }

    return S_OK;
}

unsafe extern "system" fn Arguments2_Count(iface: *mut IArguments2, out_Count: *mut LONG) -> HRESULT {
    WINE_TRACE!(b"(%p)\n\0".as_ptr() as *const i8, out_Count);

    *out_Count = numOfArgs;
    return S_OK;
}
unsafe extern "system" fn Arguments2_get_length(iface: *mut IArguments2, out_Count: *mut LONG) -> HRESULT {
    WINE_TRACE!("(%p)\n", out_Count);

    *out_Count = numOfArgs;
    return S_OK;
}

// COM vtable tables — generated from arguments.c by vtbl_gen.py
/// arguments.c:127 `static const IArguments2Vtbl Arguments2Vtbl`  (slot order: rust:types_amd64.rs)
pub static Arguments2Vtbl: IArguments2Vtbl = IArguments2Vtbl {
    QueryInterface: Some(Arguments2_QueryInterface),
    AddRef: Some(Arguments2_AddRef),
    Release: Some(Arguments2_Release),
    GetTypeInfoCount: Some(Arguments2_GetTypeInfoCount),
    GetTypeInfo: Some(Arguments2_GetTypeInfo),
    GetIDsOfNames: Some(Arguments2_GetIDsOfNames),
    Invoke: Some(Arguments2_Invoke),
    Item: Some(Arguments2_Item),
    Count: Some(Arguments2_Count),
    get_length: Some(Arguments2_get_length),
};
