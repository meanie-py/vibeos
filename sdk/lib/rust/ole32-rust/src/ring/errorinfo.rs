use crate::prelude::*;

// COM vtbl thunks use STDMETHODCALLTYPE. com-sys/bindgen encodes that as the
// *concrete* resolution of the Windows "system" ABI — extern "stdcall" on i386 and
// extern "C" on amd64/arm64 — not the "system" alias, which rustc keeps nominally
// distinct (an `extern "system"` fn item will not slot into an `extern "C"` vtbl
// field even where the two are ABI-identical). Emit each thunk with the matching
// concrete ABI. Same macro as oleaut32-rust/src/oleaut.rs.
macro_rules! stdmethod {
    ( fn $name:ident ( $($arg:ident : $ty:ty),* $(,)? ) -> $ret:ty $body:block ) => {
        #[cfg(target_arch = "x86")]
        unsafe extern "stdcall" fn $name ( $($arg : $ty),* ) -> $ret $body
        #[cfg(not(target_arch = "x86"))]
        unsafe extern "C" fn $name ( $($arg : $ty),* ) -> $ret $body
    };
}

// SysAllocString lives in **oleaut32**, not ole32 — com-sys does not carry it, so
// declare it here. It is STDAPI (stdcall on i386), hence `extern "system"`: an
// `extern "C"` decl would ask the linker for an undecorated `SysAllocString` on
// i386 instead of `_SysAllocString@4`.
extern "system" {
    fn SysAllocString(psz: *const u16) -> BSTR;
}

/// errorinfo.c calls `COM_CurrentInfo()` (compobj.c) to reach the per-thread OLE TLS
/// block and its `errorinfo` slot. compobj is ALREADY Rust here, and its
/// `COM_CurrentInfo` is a plain `pub unsafe fn` — a Rust item, NOT a `#[no_mangle]`
/// link symbol. So the extern decl rustconv emitted would be undefined at link; call
/// it by crate path instead.
///
/// compobj carries its own hand-rolled COM types (predating com-sys), so its
/// `oletls.errorinfo` is a `*mut compobj::IErrorInfo` while this module speaks
/// com-sys. Both are `#[repr(C)] { lpVtbl }`, so the raw-pointer cast below is a
/// no-op at runtime — it just crosses the two translation surfaces. Collapsing
/// compobj onto com-sys is the queued cleanup that makes this bridge unnecessary.
#[inline]
unsafe fn COM_CurrentInfo() -> *mut crate::compobj::types::oletls {
    crate::compobj::compobj::COM_CurrentInfo()
}

/// The `errorinfo` TLS slot, retyped into this module's (com-sys) IErrorInfo.
#[inline]
unsafe fn com_current_errorinfo() -> *mut *mut IErrorInfo {
    core::ptr::addr_of_mut!((*COM_CurrentInfo()).errorinfo) as *mut *mut IErrorInfo
}


// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct ErrorInfoImpl {
    pub IErrorInfo_iface: IErrorInfo,
    pub ICreateErrorInfo_iface: ICreateErrorInfo,
    pub ISupportErrorInfo_iface: ISupportErrorInfo,
    pub ref_: LONG,
    pub m_Guid: GUID,
    pub source: *mut WCHAR,
    pub description: *mut WCHAR,
    pub help_file: *mut WCHAR,
    pub m_dwHelpContext: DWORD,
}

pub static ErrorInfoVtbl: IErrorInfoVtbl = IErrorInfoVtbl {
    QueryInterface: Some(IErrorInfoImpl_QueryInterface),
    AddRef: Some(IErrorInfoImpl_AddRef),
    Release: Some(IErrorInfoImpl_Release),
    GetGUID: Some(IErrorInfoImpl_GetGUID),
    GetSource: Some(IErrorInfoImpl_GetSource),
    GetDescription: Some(IErrorInfoImpl_GetDescription),
    GetHelpFile: Some(IErrorInfoImpl_GetHelpFile),
    GetHelpContext: Some(IErrorInfoImpl_GetHelpContext),
};

pub static CreateErrorInfoVtbl: ICreateErrorInfoVtbl = ICreateErrorInfoVtbl {
    QueryInterface: Some(ICreateErrorInfoImpl_QueryInterface),
    AddRef: Some(ICreateErrorInfoImpl_AddRef),
    Release: Some(ICreateErrorInfoImpl_Release),
    SetGUID: Some(ICreateErrorInfoImpl_SetGUID),
    SetSource: Some(ICreateErrorInfoImpl_SetSource),
    SetDescription: Some(ICreateErrorInfoImpl_SetDescription),
    SetHelpFile: Some(ICreateErrorInfoImpl_SetHelpFile),
    SetHelpContext: Some(ICreateErrorInfoImpl_SetHelpContext),
};

pub static SupportErrorInfoVtbl: ISupportErrorInfoVtbl = ISupportErrorInfoVtbl {
    QueryInterface: Some(ISupportErrorInfoImpl_QueryInterface),
    AddRef: Some(ISupportErrorInfoImpl_AddRef),
    Release: Some(ISupportErrorInfoImpl_Release),
    InterfaceSupportsErrorInfo: Some(ISupportErrorInfoImpl_InterfaceSupportsErrorInfo),
};
unsafe fn heap_strdupW(str: *const u16) -> *mut u16 {
    let mut ret: *mut u16 = core::ptr::null_mut();

    if !str.is_null() {
        let size: usize;

        size = ((lstrlenW(str) + 1) as usize) * core::mem::size_of::<u16>();
        ret = heap_alloc(size as SIZE_T) as *mut u16;
        if !ret.is_null() {
            core::ptr::copy_nonoverlapping(str as *const u8, ret as *mut u8, size);
        }
    }

    ret
}

unsafe fn impl_from_IErrorInfo(iface: *mut IErrorInfo) -> *mut ErrorInfoImpl {
    (iface as *mut u8).offset(-(core::mem::offset_of!(ErrorInfoImpl, IErrorInfo_iface) as isize)) as *mut ErrorInfoImpl
}

unsafe fn impl_from_ICreateErrorInfo(iface: *mut ICreateErrorInfo) -> *mut ErrorInfoImpl {
    (iface as *mut u8).offset(-(core::mem::offset_of!(ErrorInfoImpl, ICreateErrorInfo_iface) as isize)) as *mut ErrorInfoImpl
}

unsafe fn impl_from_ISupportErrorInfo(iface: *mut ISupportErrorInfo) -> *mut ErrorInfoImpl {
    (iface as *mut u8).offset(-(core::mem::offset_of!(ErrorInfoImpl, ISupportErrorInfo_iface) as isize)) as *mut ErrorInfoImpl
}
stdmethod! { fn IErrorInfoImpl_QueryInterface(
    iface: *mut IErrorInfo,
    riid: *const GUID,
    ppvoid: *mut LPVOID
) -> HRESULT {

    let This = impl_from_IErrorInfo(iface);
    TRACE!(b"(%p)->(%s,%p)\n\0".as_ptr(), This, debugstr_guid(riid), ppvoid);

    *ppvoid = core::ptr::null_mut();

    if IsEqualIID(riid, &IID_IUnknown) || IsEqualIID(riid, &IID_IErrorInfo) {
        *ppvoid = core::ptr::addr_of_mut!((*This).IErrorInfo_iface) as LPVOID;
    } else if IsEqualIID(riid, &IID_ICreateErrorInfo) {
        *ppvoid = core::ptr::addr_of_mut!((*This).ICreateErrorInfo_iface) as LPVOID;
    } else if IsEqualIID(riid, &IID_ISupportErrorInfo) {
        *ppvoid = core::ptr::addr_of_mut!((*This).ISupportErrorInfo_iface) as LPVOID;
    }

    if !(*ppvoid).is_null() {
        IUnknown_AddRef(*ppvoid as *mut IUnknown);
        TRACE!(b"-- Interface: (%p)->(%p)\n\0".as_ptr(), ppvoid, *ppvoid);
        return S_OK;
    }
    TRACE!(b"-- Interface: E_NOINTERFACE\n\0".as_ptr());
    return E_NOINTERFACE;
} }

stdmethod! { fn IErrorInfoImpl_AddRef(iface: *mut IErrorInfo) -> u32 {
    let This = impl_from_IErrorInfo(iface);
    TRACE!(b"(%p)->(count=%u)\n\0".as_ptr(), This, (*This).ref_);
    InterlockedIncrement(&mut (*This).ref_) as u32
} }

stdmethod! { fn IErrorInfoImpl_Release(iface: *mut IErrorInfo) -> u32 {
    let This = impl_from_IErrorInfo(iface);
    let ref_ = InterlockedDecrement(&mut (*This).ref_) as u32;

    TRACE!(b"(%p)->(count=%u)\n\0".as_ptr() as *const i8, This, ref_ + 1);

    if ref_ == 0 {
        TRACE!(b"-- destroying IErrorInfo(%p)\n\0".as_ptr() as *const i8, This);

        heap_free((*This).source as LPVOID);
        heap_free((*This).description as LPVOID);
        heap_free((*This).help_file as LPVOID);
        heap_free(This as LPVOID);
    }
    ref_
} }

stdmethod! { fn IErrorInfoImpl_GetGUID(
    iface: *mut IErrorInfo,
    pGUID: *mut GUID
) -> HRESULT {
    let This = impl_from_IErrorInfo(iface);
    TRACE!(b"(%p)->(count=%u)\n\0".as_ptr(), This, (*This).ref_);
    if pGUID.is_null() {
        return E_INVALIDARG;
    }
    *pGUID = (*This).m_Guid;
    S_OK
} }

stdmethod! { fn IErrorInfoImpl_GetSource(
    iface: *mut IErrorInfo,
    pBstrSource: *mut BSTR
) -> HRESULT {
    let This = impl_from_IErrorInfo(iface);
    TRACE!(b"(%p)->(pBstrSource=%p)\n\0".as_ptr(), This, pBstrSource);
    if pBstrSource.is_null() {
        return E_INVALIDARG;
    }
    *pBstrSource = SysAllocString((*This).source);
    return S_OK;
} }

stdmethod! { fn IErrorInfoImpl_GetDescription(
    iface: *mut IErrorInfo,
    pBstrDescription: *mut BSTR,
) -> HRESULT {
    let This = impl_from_IErrorInfo(iface);

    TRACE!(b"(%p)->(pBstrDescription=%p)\n\0".as_ptr() as _, This, pBstrDescription);
    if pBstrDescription.is_null() {
        return E_INVALIDARG;
    }
    *pBstrDescription = SysAllocString((*This).description);

    S_OK
} }

stdmethod! { fn IErrorInfoImpl_GetHelpFile(
    iface: *mut IErrorInfo,
    pBstrHelpFile: *mut BSTR
) -> HRESULT {
    let This = impl_from_IErrorInfo(iface);

    TRACE!(b"(%p)->(pBstrHelpFile=%p)\n\0".as_ptr() as *const i8, This, pBstrHelpFile);
    if pBstrHelpFile.is_null() {
        return E_INVALIDARG;
    }
    *pBstrHelpFile = SysAllocString((*This).help_file);

    S_OK
} }
stdmethod! { fn IErrorInfoImpl_GetHelpContext(
    iface: *mut IErrorInfo,
    pdwHelpContext: *mut DWORD,
) -> HRESULT {
    let This = impl_from_IErrorInfo(iface);
    TRACE!(b"(%p)->(pdwHelpContext=%p)\n\0".as_ptr() as *const i8, This, pdwHelpContext);
    if pdwHelpContext.is_null() {
        return E_INVALIDARG;
    }
    *pdwHelpContext = (*This).m_dwHelpContext;

    S_OK
} }

stdmethod! { fn ICreateErrorInfoImpl_QueryInterface(
    iface: *mut ICreateErrorInfo,
    riid: *const GUID,
    ppvoid: *mut LPVOID
) -> HRESULT {
    let This = impl_from_ICreateErrorInfo(iface);
    IErrorInfo_QueryInterface(core::ptr::addr_of_mut!((*This).IErrorInfo_iface), riid, ppvoid)
} }
stdmethod! { fn ICreateErrorInfoImpl_AddRef(
    iface: *mut ICreateErrorInfo
) -> ULONG {
    let This = impl_from_ICreateErrorInfo(iface);
    IErrorInfo_AddRef(core::ptr::addr_of_mut!((*This).IErrorInfo_iface))
} }

stdmethod! { fn ICreateErrorInfoImpl_Release(iface: *mut ICreateErrorInfo) -> u32 {
    let This = impl_from_ICreateErrorInfo(iface);
    IErrorInfo_Release(core::ptr::addr_of_mut!((*This).IErrorInfo_iface))
} }

stdmethod! { fn ICreateErrorInfoImpl_SetGUID(
    iface: *mut ICreateErrorInfo,
    rguid: *const GUID,
) -> HRESULT {
    let This = impl_from_ICreateErrorInfo(iface);
    TRACE!(b"(%p)->(%s)\n\0".as_ptr() as *const i8, This, debugstr_guid(rguid));
    (*This).m_Guid = *rguid;
    S_OK
} }

stdmethod! { fn ICreateErrorInfoImpl_SetSource(
    iface: *mut ICreateErrorInfo,
    szSource: LPOLESTR,
) -> HRESULT {

    let This = impl_from_ICreateErrorInfo(iface);
    TRACE!(b"(%p): %s\n\0".as_ptr(), This, debugstr_w(szSource));

    heap_free((*This).source as LPVOID);
    (*This).source = heap_strdupW(szSource);

    S_OK
} }

stdmethod! { fn ICreateErrorInfoImpl_SetDescription(
    iface: *mut ICreateErrorInfo,
    szDescription: LPOLESTR,
) -> HRESULT {

    let This = impl_from_ICreateErrorInfo(iface);
    TRACE!(b"(%p): %s\n\0".as_ptr(), This, debugstr_w(szDescription));

    heap_free((*This).description as LPVOID);
    (*This).description = heap_strdupW(szDescription);
    S_OK
} }

stdmethod! { fn ICreateErrorInfoImpl_SetHelpFile(
    iface: *mut ICreateErrorInfo,
    szHelpFile: LPOLESTR,
) -> HRESULT {

    let This = impl_from_ICreateErrorInfo(iface);
    TRACE!(b"(%p,%s)\n\0".as_ptr(), This, debugstr_w(szHelpFile));
    heap_free((*This).help_file as LPVOID);
    (*This).help_file = heap_strdupW(szHelpFile);
    S_OK
} }
stdmethod! { fn ICreateErrorInfoImpl_SetHelpContext(
    iface: *mut ICreateErrorInfo,
    dwHelpContext: DWORD,
) -> HRESULT {
    let This = impl_from_ICreateErrorInfo(iface);
    TRACE!(b"(%p,%d)\n\0".as_ptr(), This, dwHelpContext);
    (*This).m_dwHelpContext = dwHelpContext;
    S_OK
} }

stdmethod! { fn ISupportErrorInfoImpl_QueryInterface(
    iface: *mut ISupportErrorInfo,
    riid: *const GUID,
    ppvoid: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_ISupportErrorInfo(iface);
    IErrorInfo_QueryInterface(core::ptr::addr_of_mut!((*This).IErrorInfo_iface), riid, ppvoid)
} }

stdmethod! { fn ISupportErrorInfoImpl_AddRef(iface: *mut ISupportErrorInfo) -> u32 {
    let This = impl_from_ISupportErrorInfo(iface);
    IErrorInfo_AddRef(core::ptr::addr_of_mut!((*This).IErrorInfo_iface))
} }

stdmethod! { fn ISupportErrorInfoImpl_Release(iface: *mut ISupportErrorInfo) -> u32 {
    let This = impl_from_ISupportErrorInfo(iface);
    IErrorInfo_Release(core::ptr::addr_of_mut!((*This).IErrorInfo_iface))
} }

stdmethod! { fn ISupportErrorInfoImpl_InterfaceSupportsErrorInfo(
    iface: *mut ISupportErrorInfo,
    riid: *const GUID,
) -> HRESULT {

    let This = impl_from_ISupportErrorInfo(iface);
    TRACE!(b"(%p)->(%s)\n\0".as_ptr(), This, debugstr_guid(riid));
    if IsEqualIID(riid, core::ptr::addr_of!((*This).m_Guid)) {
        S_OK
    } else {
        S_FALSE
    }
} }

unsafe fn IErrorInfoImpl_Constructor() -> *mut IErrorInfo {

    let This = heap_alloc(core::mem::size_of::<ErrorInfoImpl>() as SIZE_T) as *mut ErrorInfoImpl;

    if This.is_null() {
        return core::ptr::null_mut();
    }

    (*This).IErrorInfo_iface.lpVtbl = &ErrorInfoVtbl as *const IErrorInfoVtbl as *mut IErrorInfoVtbl;
    (*This).ICreateErrorInfo_iface.lpVtbl =
        &CreateErrorInfoVtbl as *const ICreateErrorInfoVtbl as *mut ICreateErrorInfoVtbl;
    (*This).ISupportErrorInfo_iface.lpVtbl =
        &SupportErrorInfoVtbl as *const ISupportErrorInfoVtbl as *mut ISupportErrorInfoVtbl;
    (*This).ref_ = 1;
    (*This).source = core::ptr::null_mut();
    (*This).description = core::ptr::null_mut();
    (*This).help_file = core::ptr::null_mut();
    (*This).m_dwHelpContext = 0;

    core::ptr::addr_of_mut!((*This).IErrorInfo_iface)
}

#[no_mangle]
pub unsafe extern "system" fn CreateErrorInfo(pperrinfo: *mut *mut ICreateErrorInfo) -> HRESULT {
    let pei: *mut IErrorInfo;
    let res: HRESULT;

    TRACE!(b"(%p)\n\0".as_ptr(), pperrinfo);

    if pperrinfo.is_null() {
        return E_INVALIDARG;
    }

    pei = IErrorInfoImpl_Constructor();
    if pei.is_null() {
        return E_OUTOFMEMORY;
    }

    res = IErrorInfo_QueryInterface(pei, &IID_ICreateErrorInfo, pperrinfo as *mut LPVOID);
    IErrorInfo_Release(pei);

    res
}

#[no_mangle]
pub unsafe extern "system" fn GetErrorInfo(
    dwReserved: ULONG,
    pperrinfo: *mut *mut IErrorInfo,
) -> HRESULT {
    let slot = com_current_errorinfo();

    TRACE!(
        b"(%d, %p, %p)\n\0".as_ptr(),
        dwReserved,
        pperrinfo,
        *slot,
    );

    if dwReserved != 0 {
        ERR!(b"dwReserved (0x%x) != 0\n\0".as_ptr(), dwReserved);
        return E_INVALIDARG;
    }

    if pperrinfo.is_null() {
        return E_INVALIDARG;
    }

    if (*slot).is_null() {
        *pperrinfo = core::ptr::null_mut();
        return S_FALSE;
    }

    *pperrinfo = *slot;

    /* clear thread error state */
    *slot = core::ptr::null_mut();

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn SetErrorInfo(dwReserved: ULONG, perrinfo: *mut IErrorInfo) -> HRESULT {

    TRACE!(b"(%d, %p)\n\0".as_ptr(), dwReserved, perrinfo);

    if dwReserved != 0 {
        ERR!(b"dwReserved (0x%x) != 0\n\0".as_ptr(), dwReserved);
        return E_INVALIDARG;
    }

    let slot = com_current_errorinfo();

    /* release old errorinfo */
    let pei = *slot;
    if !pei.is_null() {
        IErrorInfo_Release(pei);
    }

    /* set to new value */
    *slot = perrinfo;
    if !perrinfo.is_null() {
        IErrorInfo_AddRef(perrinfo);
    }

    S_OK
}
