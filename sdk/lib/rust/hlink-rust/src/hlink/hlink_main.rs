use crate::hlink::browse_ctx::HLinkBrowseContext_Constructor;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_079544FA595D: [u16; 42] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0049,
    0x006E, 0x0074, 0x0065, 0x0072, 0x006E, 0x0065, 0x0074, 0x0020, 0x0045, 0x0078,
    0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x004D, 0x0061, 0x0069,
    0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_91F80ED522AF: [u16; 12] = [
    0x0053u16, 0x0065, 0x0061, 0x0072, 0x0063, 0x0068, 0x0020, 0x0050, 0x0061, 0x0067,
    0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_C186F67FBF2C: [u16; 5] = [0x002Eu16, 0x0075, 0x0072, 0x006C, 0x0000];
pub static RUSTCONV_WSTR_DE091877802D: [u16; 6] = [0x0066u16, 0x0069, 0x006C, 0x0065, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_F56AA1DB6DE1: [u16; 11] = [
    0x0053u16, 0x0074, 0x0061, 0x0072, 0x0074, 0x0020, 0x0050, 0x0061, 0x0067, 0x0065,
    0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)

static mut HLink_cf: CFImpl = unsafe {
    CFImpl {
        IClassFactory_iface: IClassFactory {
            lpVtbl: &raw const hlcfvt,
        },
        lpfnCI: Some(
            HLink_Constructor
                as unsafe extern "C" fn(
                    *mut IUnknown,
                    *const IID,
                    *mut *mut ::core::ffi::c_void,
                ) -> HRESULT,
        ),
    }
};

static mut HLinkBrowseContext_cf: CFImpl = unsafe {
    CFImpl {
        IClassFactory_iface: IClassFactory {
            lpVtbl: &raw const hlcfvt,
        },
        lpfnCI: Some(
            HLinkBrowseContext_Constructor
                as unsafe extern "C" fn(
                    *mut IUnknown,
                    *const IID,
                    *mut *mut ::core::ffi::c_void,
                ) -> HRESULT,
        ),
    }
};

pub const WINE_DEFAULT_DEBUG_CHANNEL_NAME: &[u8] = b"hlink\0";

// hlink_main.c:31 `typedef HRESULT (*LPFNCREATEINSTANCE)(IUnknown*, REFIID, LPVOID*);` -- no
// calling convention, so cdecl, as both constructors are defined (link.c, browse_ctx.c).
pub type LPFNCREATEINSTANCE =
    Option<unsafe extern "C" fn(*mut IUnknown, REFIID, *mut LPVOID) -> HRESULT>;

#[repr(C)]
pub struct CFImpl {
    pub IClassFactory_iface: IClassFactory,
    pub lpfnCI: LPFNCREATEINSTANCE,
}
pub type PCFImpl = *mut CFImpl;

unsafe extern "C" fn impl_from_IClassFactory(mut iface: *mut IClassFactory) -> *mut CFImpl {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(CFImpl, IClassFactory_iface))
        as *mut CFImpl;
}

#[no_mangle]
pub unsafe extern "system" fn HlinkCreateFromMoniker(
    pimkTrgt: *mut IMoniker,
    pwzLocation: LPCWSTR,
    pwzFriendlyName: LPCWSTR,
    pihlsite: *mut IHlinkSite,
    dwSiteData: DWORD,
    piunkOuter: *mut IUnknown,
    riid: *const IID,
    ppvObj: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut hl: *mut IHlink = ::core::ptr::null_mut::<IHlink>();
    let mut hr: HRESULT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/hlink/hlink_main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"HlinkCreateFromMoniker\0" as *const u8 as *const ::core::ffi::c_char,
            56 as ::core::ffi::c_int,
            b"%p %s %s %p %li %p %s %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            pimkTrgt,
            debugstr_w(pwzLocation as *const WCHAR),
            debugstr_w(pwzFriendlyName as *const WCHAR),
            pihlsite,
            dwSiteData,
            piunkOuter,
            debugstr_guid(riid as *const _GUID),
            ppvObj,
        );
    }
    hr = CoCreateInstance(
        core::ptr::addr_of!(CLSID_StdHlink),
        piunkOuter as LPUNKNOWN,
        CLSCTX_INPROC_SERVER as DWORD,
        core::ptr::addr_of!(IID_IHlink),
        core::ptr::addr_of_mut!(hl) as *mut LPVOID,
    );
    if hr < 0 {
        return hr;
    }
    IHlink_SetMonikerReference(
        hl,
        (HLINKSETF_LOCATION | HLINKSETF_TARGET) as DWORD,
        pimkTrgt,
        pwzLocation,
    );
    if !pwzFriendlyName.is_null() {
        IHlink_SetFriendlyName(hl, pwzFriendlyName);
    }
    if !pihlsite.is_null() {
        IHlink_SetHlinkSite(hl, pihlsite, dwSiteData);
    }
    hr = IHlink_QueryInterface(hl, riid, ppvObj);
    IHlink_Release(hl);
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn HlinkCreateFromString(
    pwzTarget: LPCWSTR,
    pwzLocation: LPCWSTR,
    pwzFriendlyName: LPCWSTR,
    pihlsite: *mut IHlinkSite,
    dwSiteData: DWORD,
    piunkOuter: *mut IUnknown,
    riid: *const IID,
    ppvObj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut hl: *mut IHlink = core::ptr::null_mut::<IHlink>();
    let mut hr: HRESULT = 0;
    let mut hash: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut tgt: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut loc: *const WCHAR = core::ptr::null::<WCHAR>();
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/hlink/hlink_main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"HlinkCreateFromString\0" as *const u8 as *const core::ffi::c_char,
            89 as core::ffi::c_int,
            b"%s %s %s %p %li %p %s %p\n\0" as *const u8 as *const core::ffi::c_char,
            debugstr_w(pwzTarget as *const WCHAR),
            debugstr_w(pwzLocation as *const WCHAR),
            debugstr_w(pwzFriendlyName as *const WCHAR),
            pihlsite,
            dwSiteData,
            piunkOuter,
            debugstr_guid(riid as *const _GUID),
            ppvObj,
        );
    }

    hr = CoCreateInstance(
        &raw const CLSID_StdHlink,
        piunkOuter as LPUNKNOWN,
        CLSCTX_INPROC_SERVER as DWORD,
        &raw const IID_IHlink,
        &raw mut hl as *mut LPVOID,
    );
    if (hr) < 0 {
        return hr;
    }

    if !pwzTarget.is_null() {
        hash = wcschr(pwzTarget as *const WCHAR, '#' as WCHAR) as *mut WCHAR;
        if !hash.is_null() {
            if hash == pwzTarget as *mut WCHAR {
                tgt = core::ptr::null_mut::<WCHAR>();
            } else {
                let mut tgt_len: core::ffi::c_int =
                    hash.offset_from(pwzTarget) as core::ffi::c_longlong as core::ffi::c_int;
                tgt = malloc(
                    ((tgt_len + 1) as size_t)
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
                ) as *mut WCHAR;
                if tgt.is_null() {
                    return E_OUTOFMEMORY;
                }
                memcpy(
                    tgt as *mut core::ffi::c_void,
                    pwzTarget as *const core::ffi::c_void,
                    (tgt_len as size_t).wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
                );
                *tgt.offset(tgt_len as isize) = 0 as WCHAR;
            }
            if pwzLocation.is_null() {
                loc = hash.offset(1 as isize);
            } else {
                loc = pwzLocation as *const WCHAR;
            }
        } else {
            tgt = wcsdup(pwzTarget as *const WCHAR) as *mut WCHAR;
            if tgt.is_null() {
                return E_OUTOFMEMORY;
            }
            loc = pwzLocation as *const WCHAR;
        }
    } else {
        tgt = core::ptr::null_mut::<WCHAR>();
        loc = pwzLocation as *const WCHAR;
    }

    IHlink_SetStringReference(
        hl,
        (HLINKSETF_TARGET | HLINKSETF_LOCATION) as DWORD,
        tgt as LPCWSTR,
        loc as LPCWSTR,
    );

    free(tgt as *mut core::ffi::c_void);

    if !pwzFriendlyName.is_null() {
        IHlink_SetFriendlyName(hl, pwzFriendlyName);
    }

    if !pihlsite.is_null() {
        IHlink_SetHlinkSite(hl, pihlsite, dwSiteData);
    }

    hr = IHlink_QueryInterface(hl, riid, ppvObj);
    IHlink_Release(hl);

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn HlinkCreateBrowseContext(piunkOuter: *mut IUnknown, riid: *const IID, ppvObj: *mut *mut ::core::ffi::c_void) -> HRESULT {
    return CoCreateInstance(core::ptr::addr_of!(CLSID_StdHlinkBrowseContext), piunkOuter, CLSCTX_INPROC_SERVER as DWORD, riid, ppvObj as *mut LPVOID);
}

#[no_mangle]
pub unsafe extern "system" fn HlinkNavigate(
    mut phl: *mut IHlink,
    mut phlFrame: *mut IHlinkFrame,
    mut grfHLNF: DWORD,
    mut pbc: LPBC,
    mut pbsc: *mut IBindStatusCallback,
    mut phlbc: *mut IHlinkBrowseContext,
) -> HRESULT {
    let mut r: HRESULT = S_OK;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/hlink/hlink_main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"HlinkNavigate\0" as *const u8 as *const ::core::ffi::c_char,
            165 as ::core::ffi::c_int,
            b"%p %p %li %p %p %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            phl,
            phlFrame,
            grfHLNF,
            pbc,
            pbsc,
            phlbc,
        );
    }
    if !phlFrame.is_null() {
        r = IHlinkFrame_Navigate(phlFrame, grfHLNF, pbc, pbsc, phl);
    } else if !phl.is_null() {
        r = IHlink_Navigate(phl, grfHLNF, pbc, pbsc, phlbc);
    }
    return r;
}

#[no_mangle]
pub unsafe extern "system" fn HlinkOnNavigate(
    mut phlFrame: *mut IHlinkFrame,
    mut phlbc: *mut IHlinkBrowseContext,
    mut grfHLNF: DWORD,
    mut pmkTarget: *mut IMoniker,
    mut pwzLocation: LPCWSTR,
    mut pwzFriendlyName: LPCWSTR,
    mut puHLID: *mut ULONG,
) -> HRESULT {
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
            b"dll/win32/hlink/hlink_main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"HlinkOnNavigate\0" as *const u8 as *const ::core::ffi::c_char,
            185 as ::core::ffi::c_int,
            b"%p %p %li %p %s %s %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            phlFrame,
            phlbc,
            grfHLNF,
            pmkTarget,
            debugstr_w(pwzLocation as *const WCHAR),
            debugstr_w(pwzFriendlyName as *const WCHAR),
            puHLID,
        );
    }
    r = IHlinkBrowseContext_OnNavigateHlink(
        phlbc,
        grfHLNF,
        pmkTarget,
        pwzLocation,
        pwzFriendlyName,
        puHLID,
    );
    if !phlFrame.is_null() {
        r = IHlinkFrame_OnNavigate(
            phlFrame,
            grfHLNF,
            pmkTarget,
            pwzLocation,
            pwzFriendlyName,
            0,
        );
    }
    return r;
}

#[no_mangle]
pub unsafe extern "system" fn HlinkCreateFromData(
    piDataObj: *mut IDataObject,
    pihlsite: *mut IHlinkSite,
    dwSiteData: DWORD,
    piunkOuter: *mut IUnknown,
    riid: *const IID,
    ppvObj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if (*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_FIXME as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/hlink/hlink_main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"HlinkCreateFromData\0" as *const u8 as *const core::ffi::c_char,
            205 as i32,
            b"%p, %p, %ld, %p, %s, %p\n\0" as *const u8 as *const core::ffi::c_char,
            piDataObj,
            pihlsite,
            dwSiteData,
            piunkOuter,
            debugstr_guid(riid as *const _GUID),
            ppvObj,
        );
    }
    *ppvObj = core::ptr::null_mut();
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn HlinkQueryCreateFromData(piDataObj: *mut IDataObject) -> HRESULT {
    // FIXME("%p\n", piDataObj);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn HlinkNavigateToStringReference(
    pwzTarget: LPCWSTR,
    pwzLocation: LPCWSTR,
    pihlsite: *mut IHlinkSite,
    dwSiteData: DWORD,
    pihlframe: *mut IHlinkFrame,
    grfHLNF: DWORD,
    pibc: LPBC,
    pibsc: *mut IBindStatusCallback,
    pihlbc: *mut IHlinkBrowseContext,
) -> HRESULT {
    let mut r: HRESULT;
    let mut hlink: *mut IHlink = core::ptr::null_mut();

    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/hlink/hlink_main.c\0"
                as *const u8 as *const i8,
            b"HlinkNavigateToStringReference\0" as *const u8 as *const i8,
            232 as i32,
            b"%s %s %p %08lx %p %08lx %p %p %p\n\0" as *const u8 as *const i8,
            debugstr_w(pwzTarget as *const WCHAR),
            debugstr_w(pwzLocation as *const WCHAR),
            pihlsite,
            dwSiteData,
            pihlframe,
            grfHLNF,
            pibc,
            pibsc,
            pihlbc,
        );
    }

    r = HlinkCreateFromString(
        pwzTarget,
        pwzLocation,
        core::ptr::null::<WCHAR>(),
        pihlsite,
        dwSiteData,
        core::ptr::null_mut::<IUnknown>(),
        core::ptr::addr_of!(IID_IHlink),
        &mut hlink as *mut *mut IHlink as *mut *mut core::ffi::c_void,
    );
    if r >= 0 {
        r = HlinkNavigate(hlink, pihlframe, grfHLNF, pibc, pibsc, pihlbc);
        IHlink_Release(hlink);
    }

    return r;
}

#[no_mangle]
pub unsafe extern "system" fn HlinkIsShortcut(pwzFileName: LPCWSTR) -> HRESULT {
    let mut len: i32 = 0;

    if pwzFileName.is_null() {
        return E_INVALIDARG;
    }

    len = lstrlenW(pwzFileName) - 4;
    if len < 0 {
        return S_FALSE;
    }

    return if wcsicmp(pwzFileName.offset(len as isize), RUSTCONV_WSTR_C186F67FBF2C.as_ptr()) != 0 {
        S_FALSE
    } else {
        S_OK
    };
}

#[no_mangle]
pub unsafe extern "system" fn HlinkGetSpecialReference(mut uReference: ULONG, mut ppwzReference: *mut LPWSTR) -> HRESULT {
    let mut res: DWORD;
    let mut type_0: DWORD = core::mem::zeroed();
    let mut size: DWORD = 100;
    let mut value_name: LPCWSTR;
    let mut buf: *mut WCHAR;
    let mut hkey: HKEY = core::mem::zeroed();

    TRACE!("(%lu %p)\n", uReference, ppwzReference);

    *ppwzReference = core::ptr::null_mut();

    if uReference == HLSR_HOME {
        value_name = RUSTCONV_WSTR_F56AA1DB6DE1.as_ptr();
    } else if uReference == HLSR_SEARCHPAGE {
        value_name = RUSTCONV_WSTR_91F80ED522AF.as_ptr();
    } else if uReference == HLSR_HISTORYFOLDER {
        return E_NOTIMPL;
    } else {
        return E_INVALIDARG;
    }

    res = RegOpenKeyW(HKEY_CURRENT_USER, RUSTCONV_WSTR_079544FA595D.as_ptr(), core::ptr::addr_of_mut!(hkey)) as DWORD;
    if res != ERROR_SUCCESS {
        WARN!("Could not open key: %lu\n", res);
        return HRESULT_FROM_WIN32!(res);
    }

    buf = CoTaskMemAlloc(size as SIZE_T) as *mut WCHAR;
    res = RegQueryValueExW(
        hkey,
        value_name,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(type_0),
        buf as LPBYTE,
        core::ptr::addr_of_mut!(size),
    ) as DWORD;
    buf = CoTaskMemRealloc(buf as LPVOID, size as SIZE_T) as *mut WCHAR;
    if res == ERROR_MORE_DATA {
        res = RegQueryValueExW(
            hkey,
            value_name,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(type_0),
            buf as LPBYTE,
            core::ptr::addr_of_mut!(size),
        ) as DWORD;
    }
    RegCloseKey(hkey);
    if res != ERROR_SUCCESS {
        WARN!("Could not query value %s: %lu\n", debugstr_w(value_name), res);
        CoTaskMemFree(buf as LPVOID);
        return HRESULT_FROM_WIN32!(res);
    }

    *ppwzReference = buf as LPWSTR;
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn HlinkTranslateURL(
    pwzURL: LPCWSTR,
    grfFlags: DWORD,
    ppwzTranslatedURL: *mut LPWSTR,
) -> HRESULT {
    // FIXME("(%s %08lx %p)\n", debugstr_w(pwzURL), grfFlags, ppwzTranslatedURL);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn HlinkUpdateStackItem(
    mut frame: *mut IHlinkFrame,
    mut bc: *mut IHlinkBrowseContext,
    mut hlid: ULONG,
    mut target: *mut IMoniker,
    mut location: LPCWSTR,
    mut friendly_name: LPCWSTR,
) -> HRESULT {
    let mut hr: HRESULT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/hlink/hlink_main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"HlinkUpdateStackItem\0" as *const u8 as *const ::core::ffi::c_char,
            329 as ::core::ffi::c_int,
            b"(%p %p 0x%lx %p %s %s)\n\0" as *const u8 as *const ::core::ffi::c_char,
            frame,
            bc,
            hlid,
            target,
            debugstr_w(location as *const WCHAR),
            debugstr_w(friendly_name as *const WCHAR),
        );
    }
    if frame.is_null() && bc.is_null() {
        return E_INVALIDARG;
    }
    if !frame.is_null() {
        hr = IHlinkFrame_UpdateHlink(frame, hlid, target, location, friendly_name);
    } else {
        hr = IHlinkBrowseContext_UpdateHlink(bc, hlid, target, location, friendly_name);
    }
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn HlinkParseDisplayName(
    pibc: LPBC,
    mut pwzDisplayName: LPCWSTR,
    fNoForceAbs: BOOL,
    pcchEaten: *mut ULONG,
    ppimk: *mut *mut IMoniker,
) -> HRESULT {
    let mut eaten: ULONG = 0;
    let mut len: ULONG;
    let mut hres: HRESULT;

    TRACE!(
        "(%p %s %x %p %p)\n",
        pibc,
        debugstr_w(pwzDisplayName),
        fNoForceAbs,
        pcchEaten,
        ppimk
    );

    if fNoForceAbs != 0 {
        FIXME!("Unsupported fNoForceAbs\n");
    }

    len = (ARRAY_SIZE!(RUSTCONV_WSTR_DE091877802D) as ULONG) - 1;
    if wcsnicmp(pwzDisplayName, RUSTCONV_WSTR_DE091877802D.as_ptr(), len as usize) == 0 {
        pwzDisplayName = pwzDisplayName.add(len as usize);
        eaten += len;

        while *pwzDisplayName == '/' as u16 {
            pwzDisplayName = pwzDisplayName.add(1);
            eaten += 1;
        }
    } else {
        hres = MkParseDisplayNameEx(pibc, pwzDisplayName, pcchEaten, ppimk);
        if hres >= 0 {
            return hres;
        }

        hres = MkParseDisplayName(pibc, pwzDisplayName, pcchEaten, ppimk);
        if hres >= 0 {
            return hres;
        }
    }

    hres = CreateFileMoniker(pwzDisplayName, ppimk);
    if hres >= 0 {
        *pcchEaten = eaten + lstrlenW(pwzDisplayName) as ULONG;
    }

    return hres;
}

#[no_mangle]
pub unsafe extern "system" fn HlinkResolveMonikerForData(
    pimkReference: LPMONIKER,
    reserved: DWORD,
    pibc: LPBC,
    cFmtetc: ULONG,
    rgFmtetc: *mut FORMATETC,
    pibsc: *mut IBindStatusCallback,
    pimkBase: LPMONIKER,
) -> HRESULT {
    let mut name: LPOLESTR = core::ptr::null_mut();
    let mut bctx: *mut IBindCtx = core::ptr::null_mut();
    let mut mksys: DWORD = 0;
    let mut obj: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut hres: HRESULT;

    TRACE!(
        "(%p %lx %p %ld %p %p %p)\n",
        pimkReference,
        reserved,
        pibc,
        cFmtetc,
        rgFmtetc,
        pibsc,
        pimkBase
    );

    if cFmtetc != 0 || !rgFmtetc.is_null() || !pimkBase.is_null() {
        FIXME!("Unsupported args\n");
    }

    hres = RegisterBindStatusCallback(
        pibc as *mut IBindCtx,
        pibsc,
        core::ptr::null_mut(),
        0,
    );
    if hres < 0 {
        return hres;
    }

    hres = IMoniker_IsSystemMoniker(pimkReference as *mut IMoniker, &raw mut mksys);
    if hres >= 0 && mksys != MKSYS_URLMONIKER {
        WARN!("sysmk = %lx\n", mksys);
    }

    CreateBindCtx(0, &raw mut bctx);
    hres = IMoniker_GetDisplayName(
        pimkReference as *mut IMoniker,
        bctx,
        core::ptr::null_mut(),
        &raw mut name,
    );
    IBindCtx_Release(bctx);
    if hres >= 0 {
        TRACE!("got display name %s\n", debugstr_w(name as *const WCHAR));
        CoTaskMemFree(name as LPVOID);
    }

    return IMoniker_BindToStorage(
        pimkReference as *mut IMoniker,
        pibc as *mut IBindCtx,
        core::ptr::null_mut(),
        &raw const IID_IUnknown,
        &raw mut obj,
    );
}

#[no_mangle]
pub unsafe extern "system" fn HlinkClone(
    hlink: *mut IHlink,
    riid: *const IID,
    hls: *mut IHlinkSite,
    site_data: DWORD,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut mk: *mut IMoniker = core::ptr::null_mut();
    let mut clone_mk: *mut IMoniker = core::ptr::null_mut();
    let mut loc: *mut WCHAR = core::ptr::null_mut();
    let mut name: *mut WCHAR = core::ptr::null_mut();
    let mut hres: HRESULT = 0;

    if hlink.is_null() || riid.is_null() || obj.is_null() {
        return E_INVALIDARG;
    }

    *obj = core::ptr::null_mut();

    hres = IHlink_GetMonikerReference(
        hlink,
        (HLINKGETREF_DEFAULT) as DWORD,
        core::ptr::addr_of_mut!(mk),
        core::ptr::addr_of_mut!(loc),
    );
    if hres < 0 {
        return hres;
    }

    'cleanup: {
        if !mk.is_null() {
            let mut strm: *mut IStream = core::ptr::null_mut();
            let mut lgint: LARGE_INTEGER = core::mem::zeroed();

            hres = CreateStreamOnHGlobal(
                core::ptr::null_mut(),
                (TRUE) as BOOL,
                core::ptr::addr_of_mut!(strm),
            );
            if hres < 0 {
                IMoniker_Release(mk);
                break 'cleanup;
            }

            hres = OleSaveToStream(mk as *mut IPersistStream, strm);
            if hres < 0 {
                IStream_Release(strm);
                IMoniker_Release(mk);
                break 'cleanup;
            }
            IMoniker_Release(mk);

            lgint.QuadPart = 0;
            hres = IStream_Seek(strm, lgint, STREAM_SEEK_SET, core::ptr::null_mut());
            if hres < 0 {
                IStream_Release(strm);
                break 'cleanup;
            }

            hres = OleLoadFromStream(
                strm,
                core::ptr::addr_of!(IID_IMoniker),
                core::ptr::addr_of_mut!(clone_mk) as *mut *mut core::ffi::c_void,
            );
            IStream_Release(strm);
            if hres < 0 {
                break 'cleanup;
            }
        }

        hres = IHlink_GetFriendlyName(
            hlink,
            (HLFNAMEF_DEFAULT) as DWORD,
            core::ptr::addr_of_mut!(name),
        );
        if hres < 0 {
            break 'cleanup;
        }

        hres = HlinkCreateFromMoniker(
            clone_mk,
            loc as LPCWSTR,
            name as LPCWSTR,
            hls,
            site_data,
            core::ptr::null_mut(),
            core::ptr::addr_of!(IID_IHlink),
            obj,
        );
    }

    if !clone_mk.is_null() {
        IMoniker_Release(clone_mk);
    }
    CoTaskMemFree(loc as LPVOID);
    CoTaskMemFree(name as LPVOID);
    return hres;
}

unsafe extern "system" fn HLinkCF_fnQueryInterface(
    iface: LPCLASSFACTORY,
    riid: REFIID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let mut This: *mut CFImpl = impl_from_IClassFactory(iface as *mut IClassFactory);

    *ppvObj = core::ptr::null_mut();

    if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown))) != 0)
        || ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IClassFactory))) != 0)
    {
        *ppvObj = core::ptr::addr_of_mut!((*This).IClassFactory_iface) as LPVOID;
        return S_OK;
    }

    return E_NOINTERFACE;
}
unsafe extern "system" fn HLinkCF_fnAddRef(iface: LPCLASSFACTORY) -> ULONG {
    2
}
unsafe extern "system" fn HLinkCF_fnRelease(iface: LPCLASSFACTORY) -> ULONG {
    1
}

unsafe extern "system" fn HLinkCF_fnCreateInstance(
    iface: LPCLASSFACTORY,
    pUnkOuter: LPUNKNOWN,
    riid: REFIID,
    ppvObject: *mut LPVOID,
) -> HRESULT {
    let mut This: *mut CFImpl = impl_from_IClassFactory(iface as *mut IClassFactory);
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/hlink/hlink_main.c\0"
                as *const u8 as *const i8,
            b"HLinkCF_fnCreateInstance\0" as *const u8 as *const i8,
            518 as i32,
            b"%p->(%p,%s,%p)\n\0" as *const u8 as *const i8,
            This,
            pUnkOuter,
            debugstr_guid(riid as *const _GUID),
            ppvObject,
        );
    }
    *ppvObject = core::ptr::null_mut();
    return (*This).lpfnCI.unwrap_unchecked()(
        pUnkOuter as *mut IUnknown,
        riid,
        ppvObject,
    );
}
unsafe extern "system" fn HLinkCF_fnLockServer(iface: LPCLASSFACTORY, fLock: BOOL) -> HRESULT {
    // FIXME("%p %d\n", iface, fLock);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(rclsid: *const CLSID, iid: *const IID, ppv: *mut LPVOID) -> HRESULT {
    let mut pcf: *mut IClassFactory = core::ptr::null_mut();
    if ppv.is_null() {
        return E_INVALIDARG;
    }
    *ppv = core::ptr::null_mut();
    if IsEqualIID!(rclsid, &raw const CLSID_StdHlink) != 0 {
        pcf = core::ptr::addr_of_mut!(HLink_cf.IClassFactory_iface);
    } else if IsEqualIID!(rclsid, &raw const CLSID_StdHlinkBrowseContext) != 0 {
        pcf = core::ptr::addr_of_mut!(HLinkBrowseContext_cf.IClassFactory_iface);
    } else {
        return CLASS_E_CLASSNOTAVAILABLE;
    }

    return IClassFactory_QueryInterface(pcf, iid, ppv);
}

// COM vtable tables — generated from hlink_main.c by vtbl_gen.py
/// hlink_main.c:531 `static const IClassFactoryVtbl hlcfvt`  (slot order: com-sys/bindgen_i386.rs)
pub static hlcfvt: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: Some(HLinkCF_fnQueryInterface),
    AddRef: Some(HLinkCF_fnAddRef),
    Release: Some(HLinkCF_fnRelease),
    CreateInstance: Some(HLinkCF_fnCreateInstance),
    LockServer: Some(HLinkCF_fnLockServer),
};
