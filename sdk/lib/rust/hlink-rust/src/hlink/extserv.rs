// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_80E09B2F645F: [u16; 3] = [0x000Du16, 0x000A, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(hlink);
// Debug channel for this file is defined by the crate's debug-channel mechanism.

#[repr(C)]
pub struct ExtensionService {
    pub IUnknown_inner: IUnknown,
    pub IAuthenticate_iface: IAuthenticate,
    pub IHttpNegotiate_iface: IHttpNegotiate,
    pub IExtensionServices_iface: IExtensionServices,

    pub outer_unk: *mut IUnknown,
    pub ref_: LONG,

    pub hwnd: HWND,
    pub username: LPWSTR,
    pub password: LPWSTR,
    pub headers: LPWSTR,
}

pub type PExtensionService = *mut ExtensionService;

unsafe extern "C" fn impl_from_IUnknown(iface: *mut IUnknown) -> *mut ExtensionService {
    return (iface as *mut u8).sub(core::mem::offset_of!(ExtensionService, IUnknown_inner)) as *mut ExtensionService;
}

unsafe extern "system" fn ExtServUnk_QueryInterface(iface: *mut IUnknown, riid: REFIID, ppv: *mut *mut ::core::ffi::c_void) -> HRESULT {
    let mut This: *mut ExtensionService = impl_from_IUnknown(iface);

    *ppv = core::ptr::null_mut();

    if IsEqualGUID!(&raw const IID_IUnknown, riid) != false {
        if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/hlink/extserv.c\0".as_ptr() as *const i8,
                b"ExtServUnk_QueryInterface\0".as_ptr() as *const i8,
                52i32,
                b"(%p)->(IID_IUnknown %p)\n\0".as_ptr() as *const i8,
                This,
                ppv,
            );
        }
        *ppv = &raw mut (*This).IUnknown_inner as *mut ::core::ffi::c_void;
    } else if IsEqualGUID!(&raw const IID_IAuthenticate, riid) != false {
        if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/hlink/extserv.c\0".as_ptr() as *const i8,
                b"ExtServUnk_QueryInterface\0".as_ptr() as *const i8,
                55i32,
                b"(%p)->(IID_IAuthenticate %p)\n\0".as_ptr() as *const i8,
                This,
                ppv,
            );
        }
        *ppv = &raw mut (*This).IAuthenticate_iface as *mut ::core::ffi::c_void;
    } else if IsEqualGUID!(&raw const IID_IHttpNegotiate, riid) != false {
        if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_1,
                __dbch_1,
                b"dll/win32/hlink/extserv.c\0".as_ptr() as *const i8,
                b"ExtServUnk_QueryInterface\0".as_ptr() as *const i8,
                58i32,
                b"(%p)->(IID_IHttpNegotiate %p)\n\0".as_ptr() as *const i8,
                This,
                ppv,
            );
        }
        *ppv = &raw mut (*This).IHttpNegotiate_iface as *mut ::core::ffi::c_void;
    } else if IsEqualGUID!(&raw const IID_IExtensionServices, riid) != false {
        if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
            let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_2,
                __dbch_2,
                b"dll/win32/hlink/extserv.c\0".as_ptr() as *const i8,
                b"ExtServUnk_QueryInterface\0".as_ptr() as *const i8,
                61i32,
                b"(%p)->(IID_IExtensionServices %p)\n\0".as_ptr() as *const i8,
                This,
                ppv,
            );
        }
        *ppv = &raw mut (*This).IExtensionServices_iface as *mut ::core::ffi::c_void;
    }

    if !(*ppv).is_null() {
        IUnknown_AddRef(*ppv as *mut IUnknown);
        return S_OK;
    }

    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_FIXME as i32))) != 0 {
        let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_3: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl_3,
            __dbch_3,
            b"dll/win32/hlink/extserv.c\0".as_ptr() as *const i8,
            b"ExtServUnk_QueryInterface\0".as_ptr() as *const i8,
            70i32,
            b"(%p)->(%s %p)\n\0".as_ptr() as *const i8,
            This,
            debugstr_guid(riid as *const _GUID),
            ppv,
        );
    }

    return E_NOINTERFACE;
}

unsafe extern "system" fn ExtServUnk_AddRef(iface: *mut IUnknown) -> ULONG {
    let This: *mut ExtensionService = impl_from_IUnknown(iface);
    let ref_0: LONG = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut core::ffi::c_long) as LONG;
    if ((*__wine_dbch___default).flags as core::ffi::c_int & ((1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/hlink/extserv.c\0".as_ptr() as *const core::ffi::c_char,
            b"ExtServUnk_AddRef\0".as_ptr() as *const core::ffi::c_char,
            79 as core::ffi::c_int,
            b"(%p) ref=%ld\n\0".as_ptr() as *const core::ffi::c_char,
            This,
            ref_0,
        );
    }
    return ref_0 as ULONG;
}

unsafe extern "system" fn ExtServUnk_Release(iface: *mut IUnknown) -> ULONG {
    let This: *mut ExtensionService = impl_from_IUnknown(iface);
    let ref_0: LONG = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_));

    if ref_0 == 0 {
        free((*This).username as *mut core::ffi::c_void);
        free((*This).password as *mut core::ffi::c_void);
        free((*This).headers as *mut core::ffi::c_void);
        free(This as *mut core::ffi::c_void);
    }

    return ref_0 as ULONG;
}

unsafe extern "C" fn impl_from_IAuthenticate(mut iface: *mut IAuthenticate) -> *mut ExtensionService {
    return (iface as *mut u8).sub(core::mem::offset_of!(ExtensionService, IAuthenticate_iface))
        as *mut ExtensionService;
}

unsafe extern "system" fn Authenticate_QueryInterface(iface: *mut IAuthenticate, riid: *const IID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut This: *mut ExtensionService = impl_from_IAuthenticate(iface);
    return IUnknown_QueryInterface((*This).outer_unk, riid, ppv);
}

unsafe extern "system" fn Authenticate_AddRef(iface: *mut IAuthenticate) -> ULONG {
    let mut This: *mut ExtensionService = impl_from_IAuthenticate(iface);
    return IUnknown_AddRef((*This).outer_unk);
}
unsafe extern "system" fn Authenticate_Release(iface: *mut IAuthenticate) -> ULONG {
    let mut This: *mut ExtensionService = impl_from_IAuthenticate(iface);
    return IUnknown_Release((*This).outer_unk);
}

unsafe extern "system" fn Authenticate_Authenticate(iface: *mut IAuthenticate, phwnd: *mut HWND, pszUsername: *mut LPWSTR, pszPassword: *mut LPWSTR) -> HRESULT {
    let This: *mut ExtensionService = impl_from_IAuthenticate(iface);
    TRACE!("(%p)->(%p %p %p)\n", This, phwnd, pszUsername, pszPassword);
    if phwnd.is_null() || pszUsername.is_null() || pszPassword.is_null() {
        return E_INVALIDARG;
    }
    *phwnd = (*This).hwnd;
    *pszUsername = hlink_co_strdupW((*This).username as LPCWSTR);
    *pszPassword = hlink_co_strdupW((*This).password as LPCWSTR);
    return S_OK;
}

unsafe extern "C" fn impl_from_IHttpNegotiate(iface: *mut IHttpNegotiate) -> *mut ExtensionService {
    return (iface as ULONG_PTR)
        .wrapping_sub(core::mem::offset_of!(ExtensionService, IHttpNegotiate_iface) as ULONG_PTR)
        as *mut ExtensionService;
}
unsafe extern "system" fn HttpNegotiate_QueryInterface(iface: *mut IHttpNegotiate, riid: *const IID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let This: *mut ExtensionService = impl_from_IHttpNegotiate(iface);
    return IUnknown_QueryInterface((*This).outer_unk, riid, ppv);
}

unsafe extern "system" fn HttpNegotiate_AddRef(iface: *mut IHttpNegotiate) -> ULONG {
    let This: *mut ExtensionService = impl_from_IHttpNegotiate(iface);
    return IUnknown_AddRef((*This).outer_unk);
}

unsafe extern "system" fn HttpNegotiate_Release(iface: *mut IHttpNegotiate) -> ULONG {
    let mut This: *mut ExtensionService = impl_from_IHttpNegotiate(iface);
    return IUnknown_Release((*This).outer_unk);
}

unsafe extern "system" fn HttpNegotiate_BeginningTransaction(
    iface: *mut IHttpNegotiate,
    szURL: LPCWSTR,
    szHeaders: LPCWSTR,
    dwReserved: DWORD,
    pszAdditionalHeaders: *mut LPWSTR,
) -> HRESULT {
    let This: *mut ExtensionService = impl_from_IHttpNegotiate(iface);
    TRACE!(
        "(%p)->(%s %s %lx %p)\n",
        This,
        debugstr_w(szURL),
        debugstr_w(szHeaders),
        dwReserved,
        pszAdditionalHeaders
    );
    if pszAdditionalHeaders.is_null() {
        return E_INVALIDARG;
    }
    *pszAdditionalHeaders = hlink_co_strdupW((*This).headers as LPCWSTR);
    return S_OK;
}

unsafe extern "system" fn HttpNegotiate_OnResponse(iface: *mut IHttpNegotiate, dwResponseCode: DWORD, szResponseHeaders: LPCWSTR, szRequestHeaders: LPCWSTR, pszAdditionalRequestHeaders: *mut LPWSTR) -> HRESULT {
    let This: *mut ExtensionService = impl_from_IHttpNegotiate(iface);

    *pszAdditionalRequestHeaders = core::ptr::null_mut();
    return S_OK;
}

unsafe extern "C" fn impl_from_IExtensionServices(iface: *mut IExtensionServices) -> *mut ExtensionService {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(ExtensionService, IExtensionServices_iface)) as *mut ExtensionService;
}

unsafe extern "system" fn ExtServ_QueryInterface(iface: *mut IExtensionServices, riid: *const IID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let This: *mut ExtensionService = impl_from_IExtensionServices(iface);
    return IUnknown_QueryInterface((*This).outer_unk, riid, ppv);
}

unsafe extern "system" fn ExtServ_AddRef(iface: *mut IExtensionServices) -> ULONG {
    let mut This: *mut ExtensionService = impl_from_IExtensionServices(iface);
    return IUnknown_AddRef((*This).outer_unk);
}

unsafe extern "system" fn ExtServ_Release(iface: *mut IExtensionServices) -> ULONG {
    let This: *mut ExtensionService = impl_from_IExtensionServices(iface);
    return IUnknown_Release((*This).outer_unk);
}

unsafe extern "C" fn ExtServ_ImplSetAdditionalHeaders(This: *mut ExtensionService, pwzAdditionalHeaders: LPCWSTR) -> HRESULT {
    let mut len: ::core::ffi::c_int = 0;
    free((*This).headers as *mut ::core::ffi::c_void);
    (*This).headers = ::core::ptr::null_mut();

    if pwzAdditionalHeaders.is_null() {
        return S_OK;
    }

    len = lstrlenW(pwzAdditionalHeaders);

    if len != 0
        && (*pwzAdditionalHeaders.offset((len - 1) as isize)) as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
        && (*pwzAdditionalHeaders.offset((len - 1) as isize)) as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
    {
        (*This).headers = malloc(
            (len as usize)
                .wrapping_mul(::core::mem::size_of::<WCHAR>())
                .wrapping_add(::core::mem::size_of_val(&[0x000Du16, 0x000A, 0x0000])),
        ) as LPWSTR;
        memcpy(
            (*This).headers as *mut ::core::ffi::c_void,
            pwzAdditionalHeaders as *const ::core::ffi::c_void,
            (len as usize).wrapping_mul(::core::mem::size_of::<WCHAR>()),
        );
        memcpy(
            (*This).headers.offset(len as isize) as *mut ::core::ffi::c_void,
            RUSTCONV_WSTR_80E09B2F645F.as_ptr() as *const ::core::ffi::c_void,
            ::core::mem::size_of_val(&[0x000Du16, 0x000A, 0x0000]),
        );
    } else {
        (*This).headers = wcsdup(pwzAdditionalHeaders) as LPWSTR;
    }

    return S_OK;
}

unsafe extern "system" fn ExtServ_SetAdditionalHeaders(
    iface: *mut IExtensionServices,
    pwzAdditionalHeaders: LPCWSTR,
) -> HRESULT {
    let This: *mut ExtensionService = impl_from_IExtensionServices(iface);

    return ExtServ_ImplSetAdditionalHeaders(This, pwzAdditionalHeaders);
}

unsafe extern "C" fn ExtServ_ImplSetAuthenticateData(This: *mut ExtensionService, phwnd: HWND, pwzUsername: LPCWSTR, pwzPassword: LPCWSTR) -> HRESULT {
    free((*This).username as *mut core::ffi::c_void);
    free((*This).password as *mut core::ffi::c_void);

    (*This).hwnd = phwnd;
    (*This).username = wcsdup(pwzUsername as *const u16) as LPWSTR;
    (*This).password = wcsdup(pwzPassword as *const u16) as LPWSTR;

    return S_OK;
}

unsafe extern "system" fn ExtServ_SetAuthenticateData(
    mut iface: *mut IExtensionServices,
    mut phwnd: HWND,
    mut pwzUsername: LPCWSTR,
    mut pwzPassword: LPCWSTR,
) -> HRESULT {
    let mut This: *mut ExtensionService = impl_from_IExtensionServices(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/hlink/extserv.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"ExtServ_SetAuthenticateData\0" as *const u8 as *const ::core::ffi::c_char,
            283 as ::core::ffi::c_int,
            b"(%p)->(%p %s %s)\n\0" as *const u8 as *const ::core::ffi::c_char,
            This,
            phwnd,
            debugstr_w(pwzUsername as *const WCHAR),
            debugstr_w(pwzPassword as *const WCHAR),
        );
    }
    return ExtServ_ImplSetAuthenticateData(This, phwnd, pwzUsername, pwzPassword);
}

#[no_mangle]
pub unsafe extern "system" fn HlinkCreateExtensionServices(
    pwzAdditionalHeaders: LPCWSTR,
    phwnd: HWND,
    pszUsername: LPCWSTR,
    pszPassword: LPCWSTR,
    punkOuter: *mut IUnknown,
    riid: *const IID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut ret: *mut ExtensionService = core::ptr::null_mut::<ExtensionService>();
    let mut hres: HRESULT = S_OK;

    ret = calloc(
        1,
        (core::mem::size_of::<ExtensionService>()) as _,
    ) as *mut ExtensionService;
    if ret.is_null() {
        return E_OUTOFMEMORY;
    }

    (*ret).IUnknown_inner.lpVtbl = &raw const ExtServUnkVtbl;
    (*ret).IAuthenticate_iface.lpVtbl = &raw const AuthenticateVtbl;
    (*ret).IHttpNegotiate_iface.lpVtbl = &raw const HttpNegotiateVtbl;
    (*ret).IExtensionServices_iface.lpVtbl = &raw const ExtServVtbl;
    (*ret).ref_ = 1;

    ExtServ_ImplSetAuthenticateData(ret, phwnd, pszUsername, pszPassword);
    ExtServ_ImplSetAdditionalHeaders(ret, pwzAdditionalHeaders);

    if punkOuter.is_null() {
        (*ret).outer_unk = &raw mut (*ret).IUnknown_inner;
        hres = IUnknown_QueryInterface(&raw mut (*ret).IUnknown_inner, riid, ppv);
        IUnknown_Release(&raw mut (*ret).IUnknown_inner);
    } else if IsEqualGUID!(&raw const IID_IUnknown, riid) != false {
        (*ret).outer_unk = punkOuter;
        *ppv = &raw mut (*ret).IUnknown_inner as *mut core::ffi::c_void;
    } else {
        IUnknown_Release(&raw mut (*ret).IUnknown_inner);
        hres = E_INVALIDARG;
    }

    return hres;
}

// COM vtable tables — generated from extserv.c by vtbl_gen.py
/// extserv.c:101 `static const IUnknownVtbl ExtServUnkVtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static ExtServUnkVtbl: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: Some(ExtServUnk_QueryInterface),
    AddRef: Some(ExtServUnk_AddRef),
    Release: Some(ExtServUnk_Release),
};

/// extserv.c:147 `static const IAuthenticateVtbl AuthenticateVtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static AuthenticateVtbl: IAuthenticateVtbl = IAuthenticateVtbl {
    QueryInterface: Some(Authenticate_QueryInterface),
    AddRef: Some(Authenticate_AddRef),
    Release: Some(Authenticate_Release),
    Authenticate: Some(Authenticate_Authenticate),
};

/// extserv.c:204 `static const IHttpNegotiateVtbl HttpNegotiateVtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static HttpNegotiateVtbl: IHttpNegotiateVtbl = IHttpNegotiateVtbl {
    QueryInterface: Some(HttpNegotiate_QueryInterface),
    AddRef: Some(HttpNegotiate_AddRef),
    Release: Some(HttpNegotiate_Release),
    BeginningTransaction: Some(HttpNegotiate_BeginningTransaction),
    OnResponse: Some(HttpNegotiate_OnResponse),
};

/// extserv.c:288 `static const IExtensionServicesVtbl ExtServVtbl`  (slot order: rust:types_amd64.rs)
pub static ExtServVtbl: IExtensionServicesVtbl = IExtensionServicesVtbl {
    QueryInterface: Some(ExtServ_QueryInterface),
    AddRef: Some(ExtServ_AddRef),
    Release: Some(ExtServ_Release),
    SetAdditionalHeaders: Some(ExtServ_SetAdditionalHeaders),
    SetAuthenticateData: Some(ExtServ_SetAuthenticateData),
};
