// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_82DB442E7A60: [u16; 5] = [0x006Fu16, 0x0070, 0x0065, 0x006E, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(hlink): debug-channel registration macro, expanded by the caller's DPRINT/TRACE machinery.

pub const HLINK_SAVE_MAGIC: u32 = 0x00000002;

pub const HLINK_SAVE_MONIKER_PRESENT: u32 = 0x01;

pub const HLINK_SAVE_MONIKER_IS_ABSOLUTE: u32 = 0x02;

pub const HLINK_SAVE_LOCATION_PRESENT: u32 = 0x08;

pub const HLINK_SAVE_FRIENDLY_PRESENT: u32 = 0x10;

pub const HLINK_SAVE_TARGET_FRAME_PRESENT: u32 = 0x80;

pub const HLINK_SAVE_ALL: u32 = HLINK_SAVE_TARGET_FRAME_PRESENT
    | HLINK_SAVE_FRIENDLY_PRESENT
    | HLINK_SAVE_LOCATION_PRESENT
    | 0x04
    | HLINK_SAVE_MONIKER_IS_ABSOLUTE
    | HLINK_SAVE_MONIKER_PRESENT;

#[repr(C)]
pub struct HlinkImpl {
    pub IHlink_iface: IHlink,
    pub ref_: LONG,

    pub IPersistStream_iface: IPersistStream,
    pub IDataObject_iface: IDataObject,

    pub FriendlyName: LPWSTR,
    pub Location: LPWSTR,
    pub TargetFrameName: LPWSTR,
    pub Moniker: *mut IMoniker,
    pub Site: *mut IHlinkSite,
    pub SiteData: DWORD,
    pub absolute: BOOL,

    pub IBindStatusCallback_iface: IBindStatusCallback,
    pub bind_callback: *mut IBindStatusCallback,
    pub async_bind_ctx: *mut IBindCtx,
    pub async_flags: DWORD,
    pub async_browse_ctx: *mut IHlinkBrowseContext,
}

pub type PHlinkImpl = *mut HlinkImpl;

unsafe extern "C" fn impl_from_IHlink(iface: *mut IHlink) -> *mut HlinkImpl {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(HlinkImpl, IHlink_iface)) as *mut HlinkImpl;
}

unsafe extern "C" fn impl_from_IPersistStream(iface: *mut IPersistStream) -> *mut HlinkImpl {
    return CONTAINING_RECORD!(iface, HlinkImpl, IPersistStream_iface);
}

unsafe extern "C" fn impl_from_IDataObject(iface: *mut IDataObject) -> *mut HlinkImpl {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(HlinkImpl, IDataObject_iface)) as *mut HlinkImpl;
}

unsafe extern "C" fn __GetMoniker(mut This: *mut HlinkImpl, moniker: *mut *mut IMoniker, mut ref_type: DWORD) -> HRESULT {
    let mut hres: HRESULT = 0;
    if ref_type == HLINKGETREF_DEFAULT as DWORD {
        ref_type = HLINKGETREF_RELATIVE as DWORD;
    }
    if !(*This).Moniker.is_null() {
        let mut mktype: DWORD = MKSYS_NONE as DWORD;
        hres = IMoniker_IsSystemMoniker((*This).Moniker, core::ptr::addr_of_mut!(mktype));
        if hres == S_OK as HRESULT && mktype != MKSYS_NONE as DWORD {
            *moniker = (*This).Moniker;
            IMoniker_AddRef(*moniker);
            return S_OK as HRESULT;
        }
    }

    if ref_type == HLINKGETREF_ABSOLUTE as DWORD && !(*This).Site.is_null() {
        let mut hls_moniker: *mut IMoniker = core::ptr::null_mut();
        hres = IHlinkSite_GetMoniker(
            (*This).Site,
            (*This).SiteData,
            OLEGETMONIKER_FORCEASSIGN as DWORD,
            OLEWHICHMK_CONTAINER as DWORD,
            core::ptr::addr_of_mut!(hls_moniker),
        );
        if hres < 0 as HRESULT {
            return hres;
        }

        if !(*This).Moniker.is_null() {
            hres = IMoniker_ComposeWith(
                hls_moniker,
                (*This).Moniker,
                FALSE as BOOL,
                moniker,
            );
            IMoniker_Release(hls_moniker);
            return hres;
        }

        *moniker = hls_moniker;
        return S_OK as HRESULT;
    }

    *moniker = (*This).Moniker;
    if !(*moniker).is_null() {
        IMoniker_AddRef(*moniker);
    }

    return S_OK as HRESULT;
}

unsafe extern "system" fn IHlink_fnQueryInterface(iface: *mut IHlink, riid: REFIID, ppvObj: *mut LPVOID) -> HRESULT {
    let mut This: *mut HlinkImpl = impl_from_IHlink(iface);

    *ppvObj = core::ptr::null_mut();

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0 || (IsEqualIID!(riid, core::ptr::addr_of!(IID_IHlink)) != 0) {
        *ppvObj = core::ptr::addr_of_mut!((*This).IHlink_iface) as LPVOID;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IPersistStream)) != 0 {
        *ppvObj = core::ptr::addr_of_mut!((*This).IPersistStream_iface) as LPVOID;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IDataObject)) != 0 {
        *ppvObj = core::ptr::addr_of_mut!((*This).IDataObject_iface) as LPVOID;
    }

    if !(*ppvObj).is_null() {
        IUnknown_AddRef(*ppvObj as *mut IUnknown);
        return S_OK;
    }
    return E_NOINTERFACE;
}
unsafe extern "system" fn IHlink_fnAddRef(iface: *mut IHlink) -> ULONG {
    let This: *mut HlinkImpl = impl_from_IHlink(iface);
    let refCount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut i32) as ULONG;
    // TRACE("(%p)->(count=%lu)\n", This, refCount - 1) omitted: debug channel globals are not available in this crate.

    return refCount;
}

unsafe extern "system" fn IHlink_fnRelease(iface: *mut IHlink) -> ULONG {
    let This: *mut HlinkImpl = impl_from_IHlink(iface);
    let mut refCount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_) as *mut ::core::ffi::c_long) as ULONG;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/hlink/link.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"IHlink_fnRelease\0" as *const u8 as *const ::core::ffi::c_char,
            167 as ::core::ffi::c_int,
            b"(%p)->(count=%lu)\n\0" as *const u8 as *const ::core::ffi::c_char,
            This,
            refCount.wrapping_add(1 as ULONG),
        );
    }
    if refCount != 0 {
        return refCount;
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/hlink/link.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"IHlink_fnRelease\0" as *const u8 as *const ::core::ffi::c_char,
            171 as ::core::ffi::c_int,
            b"-- destroying IHlink (%p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            This,
        );
    }
    free((*This).FriendlyName as *mut ::core::ffi::c_void);
    free((*This).TargetFrameName as *mut ::core::ffi::c_void);
    free((*This).Location as *mut ::core::ffi::c_void);
    if !(*This).Moniker.is_null() {
        IMoniker_Release((*This).Moniker);
    }
    if !(*This).Site.is_null() {
        IHlinkSite_Release((*This).Site);
    }
    free(This as *mut ::core::ffi::c_void);
    return 0;
}

unsafe extern "system" fn IHlink_fnSetHlinkSite(
    iface: *mut IHlink,
    pihlSite: *mut IHlinkSite,
    dwSiteData: DWORD,
) -> HRESULT {
    let This: *mut HlinkImpl = impl_from_IHlink(iface);
    if !(*This).Site.is_null() {
        IHlinkSite_Release((*This).Site);
    }
    (*This).Site = pihlSite;
    if !(*This).Site.is_null() {
        IHlinkSite_AddRef((*This).Site);
    }
    (*This).SiteData = dwSiteData;
    return S_OK;
}

unsafe extern "system" fn IHlink_fnGetHlinkSite(
    iface: *mut IHlink,
    ppihlSite: *mut *mut IHlinkSite,
    pdwSiteData: *mut DWORD,
) -> HRESULT {
    let mut This: *mut HlinkImpl = impl_from_IHlink(iface);

    *ppihlSite = (*This).Site;

    if !(*This).Site.is_null() {
        IHlinkSite_AddRef((*This).Site);
        *pdwSiteData = (*This).SiteData;
    }

    return S_OK;
}

unsafe extern "system" fn IHlink_fnSetMonikerReference(
    iface: *mut IHlink,
    rfHLSETF: DWORD,
    pmkTarget: *mut IMoniker,
    pwzLocation: LPCWSTR,
) -> HRESULT {
    let mut This: *mut HlinkImpl = impl_from_IHlink(iface);

    if rfHLSETF == (0 as DWORD) {
        return E_INVALIDARG;
    }
    if (rfHLSETF & ((HLINKSETF_TARGET | HLINKSETF_LOCATION) as DWORD)) == 0 {
        return rfHLSETF as HRESULT;
    }

    if (rfHLSETF & (HLINKSETF_TARGET as DWORD)) != 0 {
        if !(*This).Moniker.is_null() {
            IMoniker_Release((*This).Moniker);
        }

        (*This).Moniker = pmkTarget;
        if !(*This).Moniker.is_null() {
            let mut pbc: *mut IBindCtx = core::ptr::null_mut::<IBindCtx>();
            let mut display_name: LPOLESTR = core::ptr::null_mut::<OLECHAR>();
            IMoniker_AddRef((*This).Moniker);
            CreateBindCtx(
                0 as DWORD,
                core::ptr::addr_of_mut!(pbc),
            );
            IMoniker_GetDisplayName(
                (*This).Moniker,
                pbc,
                core::ptr::null_mut::<IMoniker>(),
                core::ptr::addr_of_mut!(display_name),
            );
            IBindCtx_Release(pbc);
            (*This).absolute = (!display_name.is_null()
                && !wcschr(display_name as *const WCHAR, ':' as u16).is_null())
                as i32 as BOOL;
            CoTaskMemFree(display_name as LPVOID);
        }
    }

    if (rfHLSETF & (HLINKSETF_LOCATION as DWORD)) != 0 {
        free((*This).Location as *mut core::ffi::c_void);
        (*This).Location = wcsdup(pwzLocation as *const WCHAR) as LPWSTR;
    }

    return S_OK;
}
unsafe extern "system" fn IHlink_fnSetStringReference(iface: *mut IHlink, grfHLSETF: DWORD, pwzTarget: LPCWSTR, pwzLocation: LPCWSTR) -> HRESULT {
    let mut This: *mut HlinkImpl = impl_from_IHlink(iface);

    TRACE!("(%p)->(%li %s %s)\n", This, grfHLSETF, debugstr_w(pwzTarget), debugstr_w(pwzLocation));

    if grfHLSETF > (HLINKSETF_TARGET | HLINKSETF_LOCATION) as DWORD
        && grfHLSETF < -((HLINKSETF_TARGET as i32) | (HLINKSETF_LOCATION as i32)) as DWORD
    {
        return grfHLSETF as HRESULT;
    }

    if grfHLSETF & (HLINKSETF_TARGET as DWORD) != 0 {
        if !(*This).Moniker.is_null() {
            IMoniker_Release((*This).Moniker);
            (*This).Moniker = core::ptr::null_mut();
        }
        if !pwzTarget.is_null() && *pwzTarget as i32 != 0 {
            let mut pMon: *mut IMoniker = core::ptr::null_mut();
            let mut pbc: *mut IBindCtx = core::ptr::null_mut();
            let mut eaten: ULONG = 0;
            let mut r: HRESULT = 0;

            r = CreateBindCtx(0 as DWORD, core::ptr::addr_of_mut!(pbc));
            if r < 0 {
                return E_OUTOFMEMORY;
            }

            r = MkParseDisplayName(pbc, pwzTarget, core::ptr::addr_of_mut!(eaten), core::ptr::addr_of_mut!(pMon));
            IBindCtx_Release(pbc);

            if r < 0 {
                let mut p: LPCWSTR = wcschr(pwzTarget, ':' as u16) as LPCWSTR;
                if !p.is_null() && p.offset_from(pwzTarget) as i64 > 1 as i64 {
                    r = CreateURLMoniker(core::ptr::null_mut(), pwzTarget, core::ptr::addr_of_mut!(pMon));
                } else {
                    r = CreateFileMoniker(pwzTarget, core::ptr::addr_of_mut!(pMon));
                }
                if r < 0 {
                    ERR!("couldn't create moniker for %s, failed with error 0x%08lx\n",
                        debugstr_w(pwzTarget), r);
                    return r;
                }
            }

            IHlink_SetMonikerReference(iface, HLINKSETF_TARGET as DWORD, pMon, core::ptr::null());
            IMoniker_Release(pMon);
        }
    }

    if grfHLSETF & (HLINKSETF_LOCATION as DWORD) != 0 {
        free((*This).Location as *mut core::ffi::c_void);
        (*This).Location = core::ptr::null_mut();
        if !pwzLocation.is_null() && *pwzLocation as i32 != 0 {
            (*This).Location = wcsdup(pwzLocation) as LPWSTR;
        }
    }

    return S_OK;
}

unsafe extern "system" fn IHlink_fnGetMonikerReference(iface: *mut IHlink, dwWhichRef: DWORD, ppimkTarget: *mut *mut IMoniker, ppwzLocation: *mut LPWSTR) -> HRESULT {
    let mut This: *mut HlinkImpl = impl_from_IHlink(iface);
    TRACE!("(%p) -> (%li %p %p)\n", This, dwWhichRef, ppimkTarget, ppwzLocation);
    if !ppimkTarget.is_null() {
        let mut hres: HRESULT = __GetMoniker(This, ppimkTarget, dwWhichRef);
        if hres < 0 {
            if !ppwzLocation.is_null() {
                *ppwzLocation = core::ptr::null_mut();
            }
            return hres;
        }
    }
    if !ppwzLocation.is_null() {
        IHlink_GetStringReference(iface, dwWhichRef, core::ptr::null_mut(), ppwzLocation);
    }
    return S_OK;
}

unsafe extern "system" fn IHlink_fnGetStringReference(
    iface: *mut IHlink,
    dwWhichRef: DWORD,
    ppwzTarget: *mut LPWSTR,
    ppwzLocation: *mut LPWSTR,
) -> HRESULT {
    let mut This: *mut HlinkImpl = impl_from_IHlink(iface);

    if dwWhichRef != (-(1 as ::core::ffi::c_int)) as DWORD
        && (dwWhichRef & !((HLINKGETREF_DEFAULT | HLINKGETREF_ABSOLUTE | HLINKGETREF_RELATIVE) as DWORD)) != 0
    {
        if !ppwzTarget.is_null() {
            *ppwzTarget = ::core::ptr::null_mut::<WCHAR>();
        }
        if !ppwzLocation.is_null() {
            *ppwzLocation = ::core::ptr::null_mut::<WCHAR>();
        }
        return E_INVALIDARG as HRESULT;
    }

    if !ppwzTarget.is_null() {
        let mut mon: *mut IMoniker = ::core::ptr::null_mut::<IMoniker>();
        let mut hres: HRESULT = __GetMoniker(This, core::ptr::addr_of_mut!(mon), dwWhichRef);
        if hres < (0 as HRESULT) {
            if !ppwzLocation.is_null() {
                *ppwzLocation = ::core::ptr::null_mut::<WCHAR>();
            }
            return hres;
        }
        if !mon.is_null() {
            let mut pbc: *mut IBindCtx = ::core::ptr::null_mut::<IBindCtx>();
            CreateBindCtx(0 as DWORD, core::ptr::addr_of_mut!(pbc));
            IMoniker_GetDisplayName(
                mon,
                pbc,
                ::core::ptr::null_mut::<IMoniker>(),
                ppwzTarget as *mut LPOLESTR,
            );
            IBindCtx_Release(pbc);
            IMoniker_Release(mon);
        } else {
            *ppwzTarget = ::core::ptr::null_mut::<WCHAR>();
        }
    }
    if !ppwzLocation.is_null() {
        *ppwzLocation = hlink_co_strdupW((*This).Location as LPCWSTR);
    }

    return S_OK as HRESULT;
}

unsafe extern "system" fn IHlink_fnSetFriendlyName(
    iface: *mut IHlink,
    pwzFriendlyName: LPCWSTR,
) -> HRESULT {
    let mut This: *mut HlinkImpl = impl_from_IHlink(iface);
    TRACE!("(%p) -> (%s)\n", This, debugstr_w(pwzFriendlyName));
    free((*This).FriendlyName as *mut core::ffi::c_void);
    (*This).FriendlyName = wcsdup(pwzFriendlyName as *const u16) as LPWSTR;
    return S_OK;
}
unsafe extern "system" fn IHlink_fnGetFriendlyName(
    mut iface: *mut IHlink,
    mut grfHLFNAMEF: DWORD,
    mut ppwzFriendlyName: *mut LPWSTR,
) -> HRESULT {
    let mut This: *mut HlinkImpl = impl_from_IHlink(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/hlink/link.c\0"
                as *const u8 as *const i8,
            b"IHlink_fnGetFriendlyName\0" as *const u8 as *const i8,
            413 as i32,
            b"(%p) -> (%li %p)\n\0" as *const u8 as *const i8,
            This,
            grfHLFNAMEF,
            ppwzFriendlyName,
        );
    }

    if !(*This).FriendlyName.is_null() {
        *ppwzFriendlyName = hlink_co_strdupW((*This).FriendlyName as LPCWSTR);
    } else {
        let mut moniker: *mut IMoniker = core::ptr::null_mut::<IMoniker>();
        let mut hres: HRESULT = __GetMoniker(
            This,
            core::ptr::addr_of_mut!(moniker),
            HLINKGETREF_DEFAULT as DWORD,
        );
        if hres < 0 {
            *ppwzFriendlyName = core::ptr::null_mut::<WCHAR>();
            return hres;
        }
        if !moniker.is_null() {
            let mut bcxt: *mut IBindCtx = core::ptr::null_mut::<IBindCtx>();
            CreateBindCtx(0 as DWORD, core::ptr::addr_of_mut!(bcxt));

            IMoniker_GetDisplayName(
                moniker,
                bcxt,
                core::ptr::null_mut::<IMoniker>(),
                ppwzFriendlyName as *mut LPOLESTR,
            );
            IBindCtx_Release(bcxt);
            IMoniker_Release(moniker);
        } else {
            *ppwzFriendlyName = core::ptr::null_mut::<WCHAR>();
        }
    }

    return S_OK;
}

unsafe extern "system" fn IHlink_fnSetTargetFrameName(
    mut iface: *mut IHlink,
    mut pwzTargetFramename: LPCWSTR,
) -> HRESULT {
    let mut This: *mut HlinkImpl = impl_from_IHlink(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/hlink/link.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"IHlink_fnSetTargetFrameName\0" as *const u8 as *const ::core::ffi::c_char,
            448 as ::core::ffi::c_int,
            b"(%p)->(%s)\n\0" as *const u8 as *const ::core::ffi::c_char,
            This,
            debugstr_w(pwzTargetFramename as *const WCHAR),
        );
    }
    free((*This).TargetFrameName as *mut ::core::ffi::c_void);
    (*This).TargetFrameName =
        wcsdup(pwzTargetFramename as *const u16) as LPWSTR;
    return S_OK;
}

unsafe extern "system" fn IHlink_fnGetTargetFrameName(
    mut iface: *mut IHlink,
    mut ppwzTargetFrameName: *mut LPWSTR,
) -> HRESULT {
    let mut This: *mut HlinkImpl = impl_from_IHlink(iface);

    TRACE!("(%p)->(%p)\n", This, ppwzTargetFrameName);

    if (*This).TargetFrameName.is_null() {
        *ppwzTargetFrameName = core::ptr::null_mut();
        return S_FALSE;
    }

    *ppwzTargetFrameName = hlink_co_strdupW((*This).TargetFrameName as LPCWSTR);
    if (*ppwzTargetFrameName).is_null() {
        return E_OUTOFMEMORY;
    }

    return S_OK;
}
unsafe extern "system" fn IHlink_fnGetMiscStatus(
    iface: *mut IHlink,
    pdwStatus: *mut DWORD,
) -> HRESULT {
    // FIXME("\n");
    E_NOTIMPL
}

unsafe extern "system" fn IHlink_fnNavigate(
    iface: *mut IHlink,
    flags: DWORD,
    user_bind_ctx: *mut IBindCtx,
    bind_callback: *mut IBindStatusCallback,
    browse_ctx: *mut IHlinkBrowseContext,
) -> HRESULT {
    let mut This: *mut HlinkImpl = impl_from_IHlink(iface);
    let mut mon: *mut IMoniker = core::ptr::null_mut::<IMoniker>();
    let mut r: HRESULT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/hlink/link.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"IHlink_fnNavigate\0" as *const u8 as *const ::core::ffi::c_char,
            489 as ::core::ffi::c_int,
            b"hlink %p, flags %#lx, user_bind_ctx %p, bind_callback %p, browse_ctx %p.\n\0"
                as *const u8 as *const ::core::ffi::c_char,
            This,
            flags,
            user_bind_ctx,
            bind_callback,
            browse_ctx,
        );
    }

    if !(*This).async_bind_ctx.is_null() {
        return E_UNEXPECTED;
    }

    r = __GetMoniker(This, core::ptr::addr_of_mut!(mon), (HLINKGETREF_ABSOLUTE) as DWORD);

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/hlink/link.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"IHlink_fnNavigate\0" as *const u8 as *const ::core::ffi::c_char,
            495 as ::core::ffi::c_int,
            b"Moniker %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            mon,
        );
    }

    if r >= 0 {
        let mut bind_ctx: *mut IBindCtx = core::ptr::null_mut::<IBindCtx>();
        let mut unk: *mut IUnknown = core::ptr::null_mut::<IUnknown>();
        let mut target: *mut IHlinkTarget = core::ptr::null_mut::<IHlinkTarget>();

        if !browse_ctx.is_null() {
            r = IHlinkBrowseContext_GetObject(
                browse_ctx,
                mon,
                (TRUE) as BOOL,
                core::ptr::addr_of_mut!(unk),
            );
            if r != S_OK {
                CreateBindCtx(0, core::ptr::addr_of_mut!(bind_ctx));
                RegisterBindStatusCallback(
                    bind_ctx,
                    core::ptr::addr_of_mut!((*This).IBindStatusCallback_iface),
                    core::ptr::null_mut::<*mut IBindStatusCallback>(),
                    0,
                );
                (*This).bind_callback = bind_callback;
                r = IMoniker_BindToObject(
                    mon,
                    bind_ctx,
                    core::ptr::null_mut::<IMoniker>(),
                    core::ptr::addr_of!(IID_IUnknown),
                    core::ptr::addr_of_mut!(unk) as *mut *mut ::core::ffi::c_void,
                );
                if r == MK_S_ASYNCHRONOUS {
                    (*This).async_bind_ctx = bind_ctx;
                    (*This).async_flags = flags;
                    if !bind_callback.is_null() {
                        IBindStatusCallback_AddRef(bind_callback);
                    }
                    (*This).async_browse_ctx = browse_ctx;
                    IHlinkBrowseContext_AddRef((*This).async_browse_ctx);
                    IMoniker_Release(mon);
                    return r;
                }
            }
            if r == S_OK {
                r = IUnknown_QueryInterface(
                    unk,
                    core::ptr::addr_of!(IID_IHlinkTarget),
                    core::ptr::addr_of_mut!(target) as *mut *mut ::core::ffi::c_void,
                );
                IUnknown_Release(unk);
            }
            if r == S_OK {
                if !bind_ctx.is_null() {
                    IHlinkTarget_SetBrowseContext(target, browse_ctx);
                }
                r = IHlinkTarget_Navigate(target, flags, (*This).Location as LPCWSTR);
                IHlinkTarget_Release(target);
            }

            RevokeBindStatusCallback(
                bind_ctx,
                core::ptr::addr_of_mut!((*This).IBindStatusCallback_iface),
            );
            if !bind_ctx.is_null() {
                IBindCtx_Release(bind_ctx);
            }
        } else {
            let mut target: LPWSTR = core::ptr::null_mut::<WCHAR>();

            r = IHlink_GetStringReference(
                iface,
                (HLINKGETREF_DEFAULT) as DWORD,
                core::ptr::addr_of_mut!(target),
                core::ptr::null_mut::<LPWSTR>(),
            );
            if r >= 0 && !target.is_null() {
                ShellExecuteW(
                    core::ptr::null_mut::<HWND__>(),
                    RUSTCONV_WSTR_82DB442E7A60.as_ptr(),
                    target as LPCWSTR,
                    core::ptr::null::<WCHAR>(),
                    core::ptr::null::<WCHAR>(),
                    (SW_SHOW) as INT,
                );
                CoTaskMemFree(target as LPVOID);
                r = DRAGDROP_S_DROP;
            }
        }
        IMoniker_Release(mon);
    }

    if !(*This).Site.is_null() {
        IHlinkSite_OnNavigationComplete(
            (*This).Site,
            (*This).SiteData,
            0,
            r,
            core::ptr::null_mut::<WCHAR>(),
        );
    }

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_1,
            __dbch_1,
            b"dll/win32/hlink/link.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"IHlink_fnNavigate\0" as *const u8 as *const ::core::ffi::c_char,
            556 as ::core::ffi::c_int,
            b"Finished Navigation\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    return r;
}
unsafe extern "system" fn IHlink_fnSetAdditionalParams(
    iface: *mut IHlink,
    pwzAdditionalParams: LPCWSTR,
) -> HRESULT {
    // TRACE("Not implemented in native IHlink\n");
    E_NOTIMPL
}
unsafe extern "system" fn IHlink_fnGetAdditionalParams(
    iface: *mut IHlink,
    ppwzAdditionalParams: *mut LPWSTR,
) -> HRESULT {
    // TRACE("Not implemented in native IHlink\n");
    E_NOTIMPL
}

unsafe extern "system" fn IDataObject_fnQueryInterface(
    iface: *mut IDataObject,
    riid: *const IID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let mut This: *mut HlinkImpl = impl_from_IDataObject(iface);
    TRACE!("%p\n", This);
    return IHlink_QueryInterface(
        core::ptr::addr_of_mut!((*This).IHlink_iface),
        riid,
        ppvObj as *mut *mut core::ffi::c_void,
    );
}

unsafe extern "system" fn IDataObject_fnAddRef(iface: *mut IDataObject) -> ULONG {
    let mut This: *mut HlinkImpl = impl_from_IDataObject(iface);
    return IHlink_AddRef(core::ptr::addr_of_mut!((*This).IHlink_iface));
}

unsafe extern "system" fn IDataObject_fnRelease(iface: *mut IDataObject) -> ULONG {
    let mut This: *mut HlinkImpl = impl_from_IDataObject(iface);
    return IHlink_Release(core::ptr::addr_of_mut!((*This).IHlink_iface));
}
unsafe extern "system" fn IDataObject_fnGetData(
    iface: *mut IDataObject,
    pformatetcIn: *mut FORMATETC,
    pmedium: *mut STGMEDIUM,
) -> HRESULT {
    // FIXME("\n");
    E_NOTIMPL
}
unsafe extern "system" fn IDataObject_fnGetDataHere(
    iface: *mut IDataObject,
    pformatetc: *mut FORMATETC,
    pmedium: *mut STGMEDIUM,
) -> HRESULT {
    // FIXME("\n");
    E_NOTIMPL
}
unsafe extern "system" fn IDataObject_fnQueryGetData(
    iface: *mut IDataObject,
    pformatetc: *mut FORMATETC,
) -> HRESULT {
    // FIXME("\n");
    E_NOTIMPL
}
unsafe extern "system" fn IDataObject_fnGetConicalFormatEtc(
    iface: *mut IDataObject,
    pformatetcIn: *mut FORMATETC,
    pformatetcOut: *mut FORMATETC,
) -> HRESULT {
    // FIXME("\n");
    E_NOTIMPL
}
unsafe extern "system" fn IDataObject_fnSetData(
    iface: *mut IDataObject,
    pformatetc: *mut FORMATETC,
    pmedium: *mut STGMEDIUM,
    fRelease: BOOL,
) -> HRESULT {
    // FIXME("\n");
    E_NOTIMPL
}
unsafe extern "system" fn IDataObject_fnEnumFormatEtc(
    iface: *mut IDataObject,
    dwDirection: DWORD,
    ppenumFormatEtc: *mut *mut IEnumFORMATETC,
) -> HRESULT {
    // FIXME("\n");
    E_NOTIMPL
}
unsafe extern "system" fn IDataObject_fnDAdvise(
    iface: *mut IDataObject,
    pformatetc: *mut FORMATETC,
    advf: DWORD,
    pAdvSink: *mut IAdviseSink,
    pdwConnection: *mut DWORD,
) -> HRESULT {
    // FIXME("\n");
    E_NOTIMPL
}
unsafe extern "system" fn IDataObject_fnDUnadvise(
    iface: *mut IDataObject,
    dwConnection: DWORD,
) -> HRESULT {
    // FIXME("\n");
    E_NOTIMPL
}
unsafe extern "system" fn IDataObject_fnEnumDAdvise(
    iface: *mut IDataObject,
    ppenumAdvise: *mut *mut IEnumSTATDATA,
) -> HRESULT {
    // FIXME("\n");
    E_NOTIMPL
}

unsafe extern "system" fn IPersistStream_fnQueryInterface(iface: *mut IPersistStream, riid: *const IID, ppvObj: *mut LPVOID) -> HRESULT {
    let mut This: *mut HlinkImpl = impl_from_IPersistStream(iface);
    TRACE!("(%p)\n", This);
    return IHlink_QueryInterface(core::ptr::addr_of_mut!((*This).IHlink_iface), riid, ppvObj as *mut *mut core::ffi::c_void);
}

unsafe extern "system" fn IPersistStream_fnAddRef(iface: *mut IPersistStream) -> ULONG {
    let mut This: *mut HlinkImpl = impl_from_IPersistStream(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/hlink/link.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"IPersistStream_fnAddRef\0" as *const u8 as *const ::core::ffi::c_char,
            708 as ::core::ffi::c_int,
            b"(%p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            This,
        );
    }
    return IHlink_AddRef(&raw mut (*This).IHlink_iface);
}

unsafe extern "system" fn IPersistStream_fnRelease(iface: *mut IPersistStream) -> ULONG {
    let This: *mut HlinkImpl = impl_from_IPersistStream(iface);
    return IHlink_Release(core::ptr::addr_of_mut!((*This).IHlink_iface));
}

unsafe extern "system" fn IPersistStream_fnGetClassID(
    iface: *mut IPersistStream,
    pClassID: *mut CLSID,
) -> HRESULT {
    let This: *mut HlinkImpl = impl_from_IPersistStream(iface);
    *pClassID = CLSID_StdHlink as CLSID;
    return S_OK;
}
unsafe extern "system" fn IPersistStream_fnIsDirty(iface: *mut IPersistStream) -> HRESULT {
    // FIXME("\n");
    E_NOTIMPL
}

unsafe extern "C" fn write_hlink_string(pStm: *mut IStream, str: LPCWSTR) -> HRESULT {
    let mut len: DWORD = 0;
    let mut hr: HRESULT = 0;

    len = (lstrlenW(str) + 1) as DWORD;

    hr = IStream_Write(
        pStm,
        core::ptr::addr_of_mut!(len) as *const core::ffi::c_void,
        core::mem::size_of::<DWORD>() as ULONG,
        core::ptr::null_mut::<ULONG>(),
    );
    if hr < 0 {
        return hr;
    }

    hr = IStream_Write(
        pStm,
        str as *const core::ffi::c_void,
        (len * (core::mem::size_of::<WCHAR>() as DWORD)) as ULONG,
        core::ptr::null_mut::<ULONG>(),
    );
    if hr < 0 {
        return hr;
    }

    return S_OK;
}

unsafe extern "C" fn size_hlink_string(str: LPCWSTR) -> ULONG {
    return (core::mem::size_of::<DWORD>() as usize).wrapping_add(
        ((lstrlenW(str) + 1) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>()),
    ) as ULONG;
}

unsafe extern "C" fn read_hlink_string(pStm: *mut IStream, out_str: *mut LPWSTR) -> HRESULT {
    let mut str: LPWSTR = core::ptr::null_mut();
    let mut len: DWORD = 0;
    let mut read: ULONG = 0;
    let mut hr: HRESULT = 0;

    hr = IStream_Read(
        pStm,
        core::ptr::addr_of_mut!(len) as *mut core::ffi::c_void,
        core::mem::size_of::<DWORD>() as ULONG,
        core::ptr::addr_of_mut!(read),
    );
    if hr < 0 {
        return hr;
    }
    if read as usize != core::mem::size_of::<DWORD>() as usize {
        return STG_E_READFAULT;
    }

    str = malloc(
        (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize),
    ) as LPWSTR;
    if str.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = IStream_Read(
        pStm,
        str as *mut core::ffi::c_void,
        (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as ULONG,
        core::ptr::addr_of_mut!(read),
    );
    if hr < 0 {
        free(str as *mut core::ffi::c_void);
        return hr;
    }
    if read as usize != (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize) {
        free(str as *mut core::ffi::c_void);
        return STG_E_READFAULT;
    }

    *out_str = str;
    return S_OK;
}

unsafe extern "system" fn IPersistStream_fnLoad(
    iface: *mut IPersistStream,
    pStm: *mut IStream,
) -> HRESULT {
    let mut r: HRESULT;
    let mut hdr: [DWORD; 2] = [0; 2];
    let mut read: DWORD = core::mem::zeroed();
    let mut This: *mut HlinkImpl = impl_from_IPersistStream(iface);

    r = IStream_Read(
        pStm,
        core::ptr::addr_of_mut!(hdr) as *mut core::ffi::c_void,
        core::mem::size_of::<[DWORD; 2]>() as ULONG,
        core::ptr::addr_of_mut!(read),
    );
    'end: {
        if (read as usize) != core::mem::size_of::<[DWORD; 2]>()
            || hdr[0 as usize] != HLINK_SAVE_MAGIC
        {
            r = E_FAIL;
            break 'end;
        }
        if (hdr[1 as usize] & !HLINK_SAVE_ALL) != 0 {
            FIXME!(
                "unknown flag(s) 0x%lx\n",
                hdr[1 as usize] & !HLINK_SAVE_ALL
            );
        }

        if (hdr[1 as usize] & HLINK_SAVE_TARGET_FRAME_PRESENT) != 0 {
            r = read_hlink_string(
                pStm,
                core::ptr::addr_of_mut!((*This).TargetFrameName),
            );
            if r < 0 {
                break 'end;
            }
        }

        if (hdr[1 as usize] & HLINK_SAVE_FRIENDLY_PRESENT) != 0 {
            if (hdr[1 as usize] & 0x4) == 0 {
                FIXME!("0x4 flag not present with friendly name flag - not sure what this means\n");
            }
            r = read_hlink_string(
                pStm,
                core::ptr::addr_of_mut!((*This).FriendlyName),
            );
            if r < 0 {
                break 'end;
            }
        }

        if (hdr[1 as usize] & HLINK_SAVE_MONIKER_PRESENT) != 0 {
            r = OleLoadFromStream(
                pStm,
                core::ptr::addr_of!(IID_IMoniker),
                core::ptr::addr_of_mut!((*This).Moniker) as *mut LPVOID,
            );
            if r < 0 {
                break 'end;
            }
            (*This).absolute = ((hdr[1 as usize] & HLINK_SAVE_MONIKER_IS_ABSOLUTE)
                != 0) as core::ffi::c_int as BOOL;
        }

        if (hdr[1 as usize] & HLINK_SAVE_LOCATION_PRESENT) != 0 {
            r = read_hlink_string(
                pStm,
                core::ptr::addr_of_mut!((*This).Location),
            );
            if r < 0 {
                break 'end;
            }
        }
    }

    TRACE!("Load Result 0x%lx (%p)\n", r, (*This).Moniker);

    return r;
}

unsafe extern "system" fn IPersistStream_fnSave(iface: *mut IPersistStream, pStm: *mut IStream, fClearDirty: BOOL) -> HRESULT {
    let mut r: HRESULT;
    let This: *mut HlinkImpl = impl_from_IPersistStream(iface);
    let mut hdr: [DWORD; 2] = [0; 2];
    let mut moniker: *mut IMoniker = core::ptr::null_mut();

    r = __GetMoniker(This, core::ptr::addr_of_mut!(moniker), (HLINKGETREF_DEFAULT) as DWORD);
    if r < 0 {
        return r;
    }
    r = E_FAIL;

    hdr[0] = HLINK_SAVE_MAGIC;
    hdr[1] = 0;

    if !moniker.is_null() {
        hdr[1] |= HLINK_SAVE_MONIKER_PRESENT;
    }
    if (*This).absolute != 0 {
        hdr[1] |= HLINK_SAVE_MONIKER_IS_ABSOLUTE;
    }
    if !(*This).Location.is_null() {
        hdr[1] |= HLINK_SAVE_LOCATION_PRESENT;
    }
    if !(*This).FriendlyName.is_null() {
        hdr[1] |= HLINK_SAVE_FRIENDLY_PRESENT | 4 /* FIXME */;
    }
    if !(*This).TargetFrameName.is_null() {
        hdr[1] |= HLINK_SAVE_TARGET_FRAME_PRESENT;
    }

    IStream_Write(
        pStm,
        core::ptr::addr_of_mut!(hdr) as *const core::ffi::c_void,
        core::mem::size_of::<[DWORD; 2]>() as ULONG,
        core::ptr::null_mut(),
    );

    'end: {
        if !(*This).TargetFrameName.is_null() {
            r = write_hlink_string(pStm, (*This).TargetFrameName);
            if r < 0 {
                break 'end;
            }
        }

        if !(*This).FriendlyName.is_null() {
            r = write_hlink_string(pStm, (*This).FriendlyName);
            if r < 0 {
                break 'end;
            }
        }

        if !moniker.is_null() {
            let mut monstream: *mut IPersistStream = core::ptr::null_mut();

            monstream = core::ptr::null_mut();
            IMoniker_QueryInterface(
                moniker,
                core::ptr::addr_of!(IID_IPersistStream),
                core::ptr::addr_of_mut!(monstream) as *mut LPVOID,
            );
            if !monstream.is_null() {
                r = OleSaveToStream(monstream, pStm);
                IPersistStream_Release(monstream);
            }
            if r < 0 {
                break 'end;
            }
        }

        if !(*This).Location.is_null() {
            r = write_hlink_string(pStm, (*This).Location);
            if r < 0 {
                break 'end;
            }
        }
    }

    if !moniker.is_null() {
        IMoniker_Release(moniker);
    }

    return r;
}

unsafe extern "system" fn IPersistStream_fnGetSizeMax(
    mut iface: *mut IPersistStream,
    mut pcbSize: *mut ULARGE_INTEGER,
) -> HRESULT {
    let mut r: HRESULT = 0;
    let mut This: *mut HlinkImpl = impl_from_IPersistStream(iface);
    let mut moniker: *mut IMoniker = ::core::ptr::null_mut::<IMoniker>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/hlink/link.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"IPersistStream_fnGetSizeMax\0" as *const u8 as *const ::core::ffi::c_char,
            923 as ::core::ffi::c_int,
            b"(%p) Moniker(%p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            This,
            (*This).Moniker,
        );
    }
    (*pcbSize).QuadPart =
        (::core::mem::size_of::<DWORD>() as usize).wrapping_mul(2 as usize)
            as ULONGLONG;
    if !(*This).TargetFrameName.is_null() {
        (*pcbSize).QuadPart = (*pcbSize)
            .QuadPart
            .wrapping_add(size_hlink_string((*This).TargetFrameName as LPCWSTR) as ULONGLONG);
    }
    if !(*This).FriendlyName.is_null() {
        (*pcbSize).QuadPart = (*pcbSize)
            .QuadPart
            .wrapping_add(size_hlink_string((*This).FriendlyName as LPCWSTR) as ULONGLONG);
    }
    r = __GetMoniker(
        This,
        &raw mut moniker,
        HLINKGETREF_DEFAULT as DWORD,
    );
    if r < 0 {
        return r;
    }
    r = E_FAIL;
    if !moniker.is_null() {
        let mut monstream: *mut IPersistStream = ::core::ptr::null_mut::<IPersistStream>();
        IMoniker_QueryInterface(
            moniker,
            &raw const IID_IPersistStream,
            &raw mut monstream as *mut *mut ::core::ffi::c_void,
        );
        if !monstream.is_null() {
            let mut mon_size: ULARGE_INTEGER = _ULARGE_INTEGER {
                __bindgen_anon_1: _ULARGE_INTEGER__bindgen_ty_1 { LowPart: 0,
                    HighPart: 0,
                },
            };
            r = IPersistStream_GetSizeMax(monstream, &raw mut mon_size);
            (*pcbSize).QuadPart = (*pcbSize).QuadPart.wrapping_add(mon_size.QuadPart);
            IPersistStream_Release(monstream);
        }
        IMoniker_Release(moniker);
    }
    if !(*This).Location.is_null() {
        (*pcbSize).QuadPart = (*pcbSize)
            .QuadPart
            .wrapping_add(size_hlink_string((*This).Location as LPCWSTR) as ULONGLONG);
    }
    return r;
}

unsafe extern "C" fn impl_from_IBindStatusCallback(iface: *mut IBindStatusCallback) -> *mut HlinkImpl {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(HlinkImpl, IBindStatusCallback_iface)) as *mut HlinkImpl;
}

unsafe extern "system" fn bind_callback_QueryInterface(iface: *mut IBindStatusCallback, iid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualGUID!(iid, core::ptr::addr_of!(IID_IUnknown)) || IsEqualGUID!(iid, core::ptr::addr_of!(IID_IBindStatusCallback)) {
        *out = iface as *mut core::ffi::c_void;
        IBindStatusCallback_AddRef(*out as *mut IBindStatusCallback);
        return S_OK;
    }

    WARN!("No interface for %s.\n", debugstr_guid(iid));
    return E_NOINTERFACE;
}

unsafe extern "system" fn bind_callback_AddRef(iface: *mut IBindStatusCallback) -> ULONG {
    let hlink: *mut HlinkImpl = impl_from_IBindStatusCallback(iface);
    return IHlink_AddRef(core::ptr::addr_of_mut!((*hlink).IHlink_iface));
}

unsafe extern "system" fn bind_callback_Release(iface: *mut IBindStatusCallback) -> ULONG {
    let hlink: *mut HlinkImpl = impl_from_IBindStatusCallback(iface);
    return IHlink_Release(core::ptr::addr_of_mut!((*hlink).IHlink_iface));
}

unsafe extern "system" fn bind_callback_OnStartBinding(iface: *mut IBindStatusCallback, reserved: DWORD, binding: *mut IBinding) -> HRESULT {
    let mut hlink: *mut HlinkImpl = impl_from_IBindStatusCallback(iface);
    TRACE!("hlink %p, reserved %#lx, binding %p.\n", hlink, reserved, binding);
    if !(*hlink).bind_callback.is_null() {
        return IBindStatusCallback_OnStartBinding((*hlink).bind_callback, reserved, binding);
    }
    return S_OK;
}
unsafe extern "system" fn bind_callback_GetPriority(
    iface: *mut IBindStatusCallback,
    priority: *mut LONG,
) -> HRESULT {
    // FIXME("iface %p, priority %p, stub!\n", iface, priority);
    E_NOTIMPL
}

unsafe extern "system" fn bind_callback_OnLowResource(iface: *mut IBindStatusCallback, reserved: DWORD) -> HRESULT {
    let hlink: *mut HlinkImpl = impl_from_IBindStatusCallback(iface);
    if !(*hlink).bind_callback.is_null() {
        return IBindStatusCallback_OnLowResource((*hlink).bind_callback, reserved);
    }
    return S_OK;
}

unsafe extern "system" fn bind_callback_OnProgress(iface: *mut IBindStatusCallback, progress: ULONG, max: ULONG, status: ULONG, text: *const WCHAR) -> HRESULT {
    let mut hlink: *mut HlinkImpl = impl_from_IBindStatusCallback(iface);

    if !(*hlink).bind_callback.is_null() {
        return IBindStatusCallback_OnProgress((*hlink).bind_callback, progress, max, status, text as LPCWSTR);
    }
    return S_OK;
}

unsafe extern "system" fn bind_callback_OnStopBinding(
    iface: *mut IBindStatusCallback,
    hr: HRESULT,
    error: *const WCHAR,
) -> HRESULT {
    let hlink: *mut HlinkImpl = impl_from_IBindStatusCallback(iface);

    if !(*hlink).bind_callback.is_null() {
        IBindStatusCallback_OnStopBinding((*hlink).bind_callback, hr, error as LPCWSTR);
    }

    if !(*hlink).async_bind_ctx.is_null() {
        if !(*hlink).bind_callback.is_null() {
            IBindStatusCallback_Release((*hlink).bind_callback);
        }
        RevokeBindStatusCallback((*hlink).async_bind_ctx as LPBC, iface);
        IBindCtx_Release((*hlink).async_bind_ctx);
        IHlinkBrowseContext_Release((*hlink).async_browse_ctx);
        (*hlink).async_bind_ctx = ::core::ptr::null_mut::<IBindCtx>();
    }
    return S_OK;
}

unsafe extern "system" fn bind_callback_GetBindInfo(
    iface: *mut IBindStatusCallback,
    bind_flags: *mut DWORD,
    bind_info: *mut BINDINFO,
) -> HRESULT {
    let mut hlink: *mut HlinkImpl = impl_from_IBindStatusCallback(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/hlink/link.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"bind_callback_GetBindInfo\0" as *const u8 as *const ::core::ffi::c_char,
            1069 as ::core::ffi::c_int,
            b"hlink %p, bind_flags %p, bind_info %p.\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hlink,
            bind_flags,
            bind_info,
        );
    }
    if !(*hlink).bind_callback.is_null() {
        return IBindStatusCallback_GetBindInfo((*hlink).bind_callback, bind_flags, bind_info);
    }
    return S_OK;
}
unsafe extern "system" fn bind_callback_OnDataAvailable(
    iface: *mut IBindStatusCallback,
    flags: DWORD,
    size: DWORD,
    formatetc: *mut FORMATETC,
    stgmed: *mut STGMEDIUM,
) -> HRESULT {
    // FIXME("iface %p, flags %#lx, size %ld, formatetc %p, stgmed %p, stub!\n", iface, flags, size, formatetc, stgmed);
    E_NOTIMPL
}

unsafe extern "system" fn bind_callback_OnObjectAvailable(iface: *mut IBindStatusCallback, iid: REFIID, unk: *mut IUnknown) -> HRESULT {
    let hlink: *mut HlinkImpl = impl_from_IBindStatusCallback(iface);
    let mut target: *mut IHlinkTarget = core::mem::zeroed();
    let mut hr: HRESULT;

    if !(*hlink).bind_callback.is_null() {
        IBindStatusCallback_OnObjectAvailable((*hlink).bind_callback, iid, unk);
    }

    if !(*hlink).async_bind_ctx.is_null() {
        hr = IUnknown_QueryInterface(unk, core::ptr::addr_of!(IID_IHlinkTarget), core::ptr::addr_of_mut!(target) as *mut *mut core::ffi::c_void);
        if hr < 0 {
            return hr;
        }

        IHlinkTarget_SetBrowseContext(target, (*hlink).async_browse_ctx);
        hr = IHlinkTarget_Navigate(target, (*hlink).async_flags, (*hlink).Location as LPCWSTR);
        IHlinkTarget_Release(target);

        if !(*hlink).Site.is_null() {
            IHlinkSite_OnNavigationComplete((*hlink).Site, (*hlink).SiteData, 0, hr, core::ptr::null_mut());
        }

        return hr;
    }

    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn HLink_Constructor(pUnkOuter: *mut IUnknown, riid: *const IID, ppv: *mut *mut ::core::ffi::c_void) -> HRESULT {
    let mut hl: *mut HlinkImpl = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    *ppv = core::ptr::null_mut();

    if !pUnkOuter.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    hl = calloc((1 as usize) as _, (core::mem::size_of::<HlinkImpl>() as usize) as _) as *mut HlinkImpl;
    if hl.is_null() {
        return E_OUTOFMEMORY;
    }

    (*hl).ref_ = 1;
    (*hl).IHlink_iface.lpVtbl = &raw const hlvt;
    (*hl).IPersistStream_iface.lpVtbl = &raw const psvt;
    (*hl).IDataObject_iface.lpVtbl = &raw const dovt;
    (*hl).IBindStatusCallback_iface.lpVtbl = &raw const bind_callback_vtbl;

    hr = IHlink_QueryInterface(core::ptr::addr_of_mut!((*hl).IHlink_iface), riid, ppv);
    IHlink_Release(core::ptr::addr_of_mut!((*hl).IHlink_iface));

    return hr;
}

// COM vtable tables — generated from link.c by vtbl_gen.py
/// link.c:574 `static const IHlinkVtbl hlvt`  (slot order: rust:types_amd64.rs)
pub static hlvt: IHlinkVtbl = IHlinkVtbl {
    QueryInterface: Some(IHlink_fnQueryInterface),
    AddRef: Some(IHlink_fnAddRef),
    Release: Some(IHlink_fnRelease),
    SetHlinkSite: Some(IHlink_fnSetHlinkSite),
    GetHlinkSite: Some(IHlink_fnGetHlinkSite),
    SetMonikerReference: Some(IHlink_fnSetMonikerReference),
    GetMonikerReference: Some(IHlink_fnGetMonikerReference),
    SetStringReference: Some(IHlink_fnSetStringReference),
    GetStringReference: Some(IHlink_fnGetStringReference),
    SetFriendlyName: Some(IHlink_fnSetFriendlyName),
    GetFriendlyName: Some(IHlink_fnGetFriendlyName),
    SetTargetFrameName: Some(IHlink_fnSetTargetFrameName),
    GetTargetFrameName: Some(IHlink_fnGetTargetFrameName),
    GetMiscStatus: Some(IHlink_fnGetMiscStatus),
    Navigate: Some(IHlink_fnNavigate),
    SetAdditionalParams: Some(IHlink_fnSetAdditionalParams),
    GetAdditionalParams: Some(IHlink_fnGetAdditionalParams),
};

/// link.c:681 `static const IDataObjectVtbl dovt`  (slot order: com-sys/bindgen_i386.rs)
pub static dovt: IDataObjectVtbl = IDataObjectVtbl {
    QueryInterface: Some(IDataObject_fnQueryInterface),
    AddRef: Some(IDataObject_fnAddRef),
    Release: Some(IDataObject_fnRelease),
    GetData: Some(IDataObject_fnGetData),
    GetDataHere: Some(IDataObject_fnGetDataHere),
    QueryGetData: Some(IDataObject_fnQueryGetData),
    GetCanonicalFormatEtc: Some(IDataObject_fnGetConicalFormatEtc),
    SetData: Some(IDataObject_fnSetData),
    EnumFormatEtc: Some(IDataObject_fnEnumFormatEtc),
    DAdvise: Some(IDataObject_fnDAdvise),
    DUnadvise: Some(IDataObject_fnDUnadvise),
    EnumDAdvise: Some(IDataObject_fnEnumDAdvise),
};

/// link.c:959 `static const IPersistStreamVtbl psvt`  (slot order: com-sys/bindgen_i386.rs)
pub static psvt: IPersistStreamVtbl = IPersistStreamVtbl {
    QueryInterface: Some(IPersistStream_fnQueryInterface),
    AddRef: Some(IPersistStream_fnAddRef),
    Release: Some(IPersistStream_fnRelease),
    GetClassID: Some(IPersistStream_fnGetClassID),
    IsDirty: Some(IPersistStream_fnIsDirty),
    Load: Some(IPersistStream_fnLoad),
    Save: Some(IPersistStream_fnSave),
    GetSizeMax: Some(IPersistStream_fnGetSizeMax),
};

/// link.c:1115 `static const IBindStatusCallbackVtbl bind_callback_vtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static bind_callback_vtbl: IBindStatusCallbackVtbl = IBindStatusCallbackVtbl {
    QueryInterface: Some(bind_callback_QueryInterface),
    AddRef: Some(bind_callback_AddRef),
    Release: Some(bind_callback_Release),
    OnStartBinding: Some(bind_callback_OnStartBinding),
    GetPriority: Some(bind_callback_GetPriority),
    OnLowResource: Some(bind_callback_OnLowResource),
    OnProgress: Some(bind_callback_OnProgress),
    OnStopBinding: Some(bind_callback_OnStopBinding),
    GetBindInfo: Some(bind_callback_GetBindInfo),
    OnDataAvailable: Some(bind_callback_OnDataAvailable),
    OnObjectAvailable: Some(bind_callback_OnObjectAvailable),
};
