use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_725A66AC62D2: [u16; 7] = [0x0023u16, 0x0033, 0x0032, 0x0037, 0x0036, 0x0038, 0x0000];

pub static WC_LISTBOXW: [::core::ffi::c_ushort; 8] = unsafe {
    ::core::mem::transmute::<[u8; 16], [::core::ffi::c_ushort; 8]>(*b"L\0i\0s\0t\0B\0o\0x\0\0\0")
};

static mut classes: [win_class_data; 3] = unsafe {
    [
        win_class_data {
            name: WC_LISTBOXW.as_ptr() as *const WCHAR,
            idx: 0x10000 as DWORD,
            stub: (TRUE as i32),
            vtbl: ::core::ptr::null::<::core::ffi::c_void>(),
        },
        win_class_data {
            name: RUSTCONV_C2R_WSTR_725A66AC62D2.as_ptr() as *const WCHAR,
            idx: 0x10001 as DWORD,
            stub: (TRUE as i32),
            vtbl: ::core::ptr::null::<::core::ffi::c_void>(),
        },
        win_class_data {
            name: ::core::ptr::null::<WCHAR>(),
            idx: 0,
            stub: 0,
            vtbl: ::core::ptr::null::<::core::ffi::c_void>(),
        },
    ]
};

// WINE_DEFAULT_DEBUG_CHANNEL(oleacc);
#[repr(C)]
pub struct Window {
    pub IAccessible_iface: IAccessible,
    pub IOleWindow_iface: IOleWindow,
    pub IEnumVARIANT_iface: IEnumVARIANT,
    pub IServiceProvider_iface: IServiceProvider,
    pub ref_: LONG,
    pub hwnd: HWND,
}
pub type PWindow = *mut Window;

pub const __WINE_DBCL_FIXME: __wine_debug_class = 0;


unsafe extern "C" fn impl_from_Window(iface: *mut IAccessible) -> *mut Window {
    return (iface as *mut u8).sub(core::mem::offset_of!(Window, IAccessible_iface)) as *mut Window;
}

unsafe extern "system" fn Window_QueryInterface(iface: *mut IAccessible, riid: *const IID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let This: *mut Window = impl_from_Window(iface);

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IAccessible)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IDispatch)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0 {
        *ppv = iface as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IOleWindow)) != 0 {
        *ppv = core::ptr::addr_of_mut!((*This).IOleWindow_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IEnumVARIANT)) != 0 {
        *ppv = core::ptr::addr_of_mut!((*This).IEnumVARIANT_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IServiceProvider)) != 0 {
        *ppv = core::ptr::addr_of_mut!((*This).IServiceProvider_iface) as *mut core::ffi::c_void;
    } else {
        WARN!("no interface: %s\n", debugstr_guid(riid));
        *ppv = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IAccessible_AddRef(iface);
    return S_OK;
}

unsafe extern "system" fn Window_AddRef(iface: *mut IAccessible) -> ULONG {
    let This: *mut Window = impl_from_Window(iface);
    let ref_: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut LONG) as ULONG;

    return ref_;
}

unsafe extern "system" fn Window_Release(iface: *mut IAccessible) -> ULONG {
    let This: *mut Window = impl_from_Window(iface);
    let ref_0: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*This).ref_) as *mut ::core::ffi::c_long,
    ) as ULONG;

    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int))) != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/window.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"Window_Release\0" as *const u8 as *const ::core::ffi::c_char,
            85 as ::core::ffi::c_int,
            b"(%p) ref = %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
            This,
            ref_0,
        );
    }

    if ref_0 == 0 {
        heap_free(This as *mut ::core::ffi::c_void);
    }
    return ref_0;
}

unsafe extern "system" fn Window_GetTypeInfoCount(iface: *mut IAccessible, pctinfo: *mut UINT) -> HRESULT {
    let mut This: *mut Window = impl_from_Window(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_FIXME as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/window.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"Window_GetTypeInfoCount\0" as *const u8 as *const core::ffi::c_char,
            95 as i32,
            b"(%p)->(%p)\n\0" as *const u8 as *const core::ffi::c_char,
            This,
            pctinfo,
        );
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn Window_GetTypeInfo(
    iface: *mut IAccessible,
    iTInfo: UINT,
    lcid: LCID,
    ppTInfo: *mut *mut ITypeInfo,
) -> HRESULT {
    let mut This: *mut Window = impl_from_Window(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/window.c\0"
                .as_ptr() as *const ::core::ffi::c_char,
            b"Window_GetTypeInfo\0".as_ptr() as *const ::core::ffi::c_char,
            103 as ::core::ffi::c_int,
            b"(%p)->(%u %lx %p)\n\0".as_ptr() as *const ::core::ffi::c_char,
            This,
            iTInfo,
            lcid,
            ppTInfo,
        );
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn Window_GetIDsOfNames(
    iface: *mut IAccessible,
    riid: *const IID,
    rgszNames: *mut LPOLESTR,
    cNames: UINT,
    lcid: LCID,
    rgDispId: *mut DISPID,
) -> HRESULT {
    let This: *mut Window = impl_from_Window(iface);
    TRACE!(
        "(%p)->(%s %p %u %lx %p)\n",
        This,
        debugstr_guid(riid as *const _),
        rgszNames,
        cNames,
        lcid,
        rgDispId
    );
    return E_NOTIMPL;
}

unsafe extern "system" fn Window_Invoke(iface: *mut IAccessible, dispIdMember: DISPID, riid: *const IID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT {
    let mut This: *mut Window = impl_from_Window(iface);
    TRACE!("(%p)->(%lx %s %lx %x %p %p %p %p)\n", This, dispIdMember, debugstr_guid(riid), lcid, wFlags as i32, pDispParams, pVarResult, pExcepInfo, puArgErr);
    return E_NOTIMPL;
}

unsafe extern "system" fn Window_get_accParent(
    iface: *mut IAccessible,
    ppdispParent: *mut *mut IDispatch,
) -> HRESULT {
    let mut This: *mut Window = impl_from_Window(iface);
    if ((*__wine_dbch___default).flags as i32) & ((1 as i32) << (__WINE_DBCL_FIXME as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/window.c\0"
                as *const u8 as *const i8,
            b"Window_get_accParent\0" as *const u8 as *const i8,
            129 as i32,
            b"(%p)->(%p)\n\0" as *const u8 as *const i8,
            This,
            ppdispParent,
        );
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn Window_get_accChildCount(
    iface: *mut IAccessible,
    pcountChildren: *mut LONG,
) -> HRESULT {
    let mut This: *mut Window = impl_from_Window(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/window.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"Window_get_accChildCount\0" as *const u8 as *const ::core::ffi::c_char,
            136 as ::core::ffi::c_int,
            b"(%p)->(%p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            This,
            pcountChildren,
        );
    }
    return E_NOTIMPL;
}
unsafe extern "system" fn Window_get_accChild(
    mut iface: *mut IAccessible,
    mut varChildID: VARIANT,
    mut ppdispChild: *mut *mut IDispatch,
) -> HRESULT {
    let mut This: *mut Window = impl_from_Window(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_FIXME as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/window.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"Window_get_accChild\0" as *const u8 as *const ::core::ffi::c_char,
            144 as ::core::ffi::c_int,
            b"(%p)->(%s %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            This,
            debugstr_variant(core::ptr::addr_of_mut!(varChildID)),
            ppdispChild,
        );
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn Window_get_accName(iface: *mut IAccessible, mut varID: VARIANT, pszName: *mut BSTR) -> HRESULT {
    let mut This: *mut Window = impl_from_Window(iface);
    if ((*__wine_dbch___default).flags as core::ffi::c_int & ((1 as core::ffi::c_int) << __WINE_DBCL_FIXME as core::ffi::c_int)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/window.c\0".as_ptr() as *const core::ffi::c_char,
            b"Window_get_accName\0".as_ptr() as *const core::ffi::c_char,
            151 as core::ffi::c_int,
            b"(%p)->(%s %p)\n\0".as_ptr() as *const core::ffi::c_char,
            This,
            debugstr_variant(core::ptr::addr_of_mut!(varID)),
            pszName,
        );
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn Window_get_accValue(
    iface: *mut IAccessible,
    mut varID: VARIANT,
    pszValue: *mut BSTR,
) -> HRESULT {
    let mut This: *mut Window = impl_from_Window(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/window.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"Window_get_accValue\0" as *const u8 as *const ::core::ffi::c_char,
            158 as ::core::ffi::c_int,
            b"(%p)->(%s %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            This,
            debugstr_variant(&raw mut varID),
            pszValue,
        );
    }
    return E_NOTIMPL;
}
unsafe extern "system" fn Window_get_accDescription(
    mut iface: *mut IAccessible,
    mut varID: VARIANT,
    mut pszDescription: *mut BSTR,
) -> HRESULT {
    let mut This: *mut Window = impl_from_Window(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/window.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"Window_get_accDescription\0" as *const u8 as *const ::core::ffi::c_char,
            166 as ::core::ffi::c_int,
            b"(%p)->(%s %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            This,
            debugstr_variant(&raw mut varID),
            pszDescription,
        );
    }
    return E_NOTIMPL;
}
unsafe extern "system" fn Window_get_accRole(iface: *mut IAccessible, varID: VARIANT, pvarRole: *mut VARIANT) -> HRESULT {
    let mut This: *mut Window = impl_from_Window(iface);
    return E_NOTIMPL;
}

unsafe extern "system" fn Window_get_accState(iface: *mut IAccessible, mut varID: VARIANT, pvarState: *mut VARIANT) -> HRESULT {
    let mut This: *mut Window = impl_from_Window(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_FIXME as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/window.c\0" as *const u8 as *const i8,
            b"Window_get_accState\0" as *const u8 as *const i8,
            180i32,
            b"(%p)->(%s %p)\n\0" as *const u8 as *const i8,
            This,
            debugstr_variant(core::ptr::addr_of_mut!(varID)),
            pvarState,
        );
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn Window_get_accHelp(iface: *mut IAccessible, mut varID: VARIANT, pszHelp: *mut BSTR) -> HRESULT {
    let This: *mut Window = impl_from_Window(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/window.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"Window_get_accHelp\0" as *const u8 as *const ::core::ffi::c_char,
            187 as ::core::ffi::c_int,
            b"(%p)->(%s %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            This,
            debugstr_variant(&raw mut varID),
            pszHelp,
        );
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn Window_get_accHelpTopic(
    iface: *mut IAccessible,
    pszHelpFile: *mut BSTR,
    mut varID: VARIANT,
    pidTopic: *mut LONG,
) -> HRESULT {
    let mut This: *mut Window = impl_from_Window(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_FIXME as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/window.c\0"
                as *const u8 as *const i8,
            b"Window_get_accHelpTopic\0" as *const u8 as *const i8,
            195i32,
            b"(%p)->(%p %s %p)\n\0" as *const u8 as *const i8,
            This,
            pszHelpFile,
            debugstr_variant(&raw mut varID),
            pidTopic,
        );
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn Window_get_accKeyboardShortcut(
    mut iface: *mut IAccessible,
    mut varID: VARIANT,
    mut pszKeyboardShortcut: *mut BSTR,
) -> HRESULT {
    let mut This: *mut Window = impl_from_Window(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/window.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"Window_get_accKeyboardShortcut\0" as *const u8 as *const ::core::ffi::c_char,
            203 as ::core::ffi::c_int,
            b"(%p)->(%s %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            This,
            debugstr_variant(&raw mut varID),
            pszKeyboardShortcut,
        );
    }
    return E_NOTIMPL;
}
unsafe extern "system" fn Window_get_accFocus(
    iface: *mut IAccessible,
    pvarID: *mut VARIANT,
) -> HRESULT {
    let This: *mut Window = impl_from_Window(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_FIXME as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/window.c\0"
                as *const u8 as *const i8,
            b"Window_get_accFocus\0" as *const u8 as *const i8,
            210i32,
            b"(%p)->(%p)\n\0" as *const u8 as *const i8,
            This,
            pvarID,
        );
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn Window_get_accSelection(iface: *mut IAccessible, pvarID: *mut VARIANT) -> HRESULT {
    let mut This: *mut Window = impl_from_Window(iface);
    return E_NOTIMPL;
}

unsafe extern "system" fn Window_get_accDefaultAction(
    iface: *mut IAccessible,
    mut varID: VARIANT,
    pszDefaultAction: *mut BSTR,
) -> HRESULT {
    let mut This: *mut Window = impl_from_Window(iface);
    if (*__wine_dbch___default).flags as i32
        & (1i32) << __WINE_DBCL_FIXME as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/window.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"Window_get_accDefaultAction\0" as *const u8 as *const core::ffi::c_char,
            225i32,
            b"(%p)->(%s %p)\n\0" as *const u8 as *const core::ffi::c_char,
            This,
            debugstr_variant(core::ptr::addr_of_mut!(varID)),
            pszDefaultAction,
        );
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn Window_accSelect(iface: *mut IAccessible, flagsSelect: LONG, mut varID: VARIANT) -> HRESULT {
    let mut This: *mut Window = impl_from_Window(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_FIXME as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/window.c\0".as_ptr() as *const i8,
            b"Window_accSelect\0".as_ptr() as *const i8,
            232 as i32,
            b"(%p)->(%lx %s)\n\0".as_ptr() as *const i8,
            This,
            flagsSelect,
            debugstr_variant(core::ptr::addr_of_mut!(varID)),
        );
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn Window_accLocation(iface: *mut IAccessible, pxLeft: *mut LONG, pyTop: *mut LONG, pcxWidth: *mut LONG, pcyHeight: *mut LONG, varID: VARIANT) -> HRESULT {
    let mut This: *mut Window = impl_from_Window(iface);
    return E_NOTIMPL;
}

unsafe extern "system" fn Window_accNavigate(
    iface: *mut IAccessible,
    navDir: LONG,
    mut varStart: VARIANT,
    pvarEnd: *mut VARIANT,
) -> HRESULT {
    let mut This: *mut Window = impl_from_Window(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_FIXME as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/window.c\0"
                as *const u8 as *const i8,
            b"Window_accNavigate\0" as *const u8 as *const i8,
            249 as i32,
            b"(%p)->(%ld %s %p)\n\0" as *const u8 as *const i8,
            This,
            navDir,
            debugstr_variant(&raw mut varStart),
            pvarEnd,
        );
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn Window_accHitTest(iface: *mut IAccessible, x: LONG, y: LONG, v: *mut VARIANT) -> HRESULT {
    let mut This: *mut Window = impl_from_Window(iface);
    let mut disp: *mut IDispatch = core::ptr::null_mut();
    let mut pt: POINT = POINT { x: 0, y: 0 };
    let mut hr: HRESULT;
    let mut rect: RECT = core::mem::zeroed();

    DPRINT1!("(%p)->(%ld %ld %p)\n", This, x, y, v);

    (*v).n1.n2.vt = VT_EMPTY as VARTYPE;
    if GetClientRect((*This).hwnd, &raw mut rect) == 0 {
        return E_FAIL;
    }
    if ClientToScreen((*This).hwnd, &raw mut rect as *mut POINT) == 0
        || ClientToScreen(
            (*This).hwnd,
            (&raw mut rect as *mut POINT).offset(1 as isize),
        ) == 0
    {
        return E_FAIL;
    }
    pt.x = x;
    pt.y = y;
    if PtInRect(&raw mut rect as *const RECT, pt) != 0 {
        hr = AccessibleObjectFromWindow(
            (*This).hwnd,
            OBJID_CLIENT as DWORD,
            core::ptr::addr_of!(IID_IDispatch),
            &raw mut disp as *mut *mut core::ffi::c_void,
        );
        if hr < 0 {
            return hr;
        }
        if disp.is_null() {
            return E_FAIL;
        }

        (*v).n1.n2.vt = VT_DISPATCH as VARTYPE;
        (*v).n1.n2.n3.pdispVal = disp;
        return S_OK;
    }

    WARN!("non-client area not handled yet\n");
    return E_NOTIMPL;
}
unsafe extern "system" fn Window_accDoDefaultAction(
    mut iface: *mut IAccessible,
    mut varID: VARIANT,
) -> HRESULT {
    let mut This: *mut Window = impl_from_Window(iface);
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_FIXME as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/window.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"Window_accDoDefaultAction\0" as *const u8 as *const core::ffi::c_char,
            291 as core::ffi::c_int,
            b"(%p)->(%s)\n\0" as *const u8 as *const core::ffi::c_char,
            This,
            debugstr_variant(&raw mut varID),
        );
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn Window_put_accName(iface: *mut IAccessible, varID: VARIANT, pszName: BSTR) -> HRESULT {
    let mut This: *mut Window = impl_from_Window(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        let mut varID = varID;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/window.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"Window_put_accName\0" as *const u8 as *const ::core::ffi::c_char,
            298 as ::core::ffi::c_int,
            b"(%p)->(%s %s)\n\0" as *const u8 as *const ::core::ffi::c_char,
            This,
            debugstr_variant(&raw mut varID),
            debugstr_w(pszName as *const WCHAR),
        );
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn Window_put_accValue(
    mut iface: *mut IAccessible,
    mut varID: VARIANT,
    mut pszValue: BSTR,
) -> HRESULT {
    let mut This: *mut Window = impl_from_Window(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/window.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"Window_put_accValue\0" as *const u8 as *const ::core::ffi::c_char,
            305 as ::core::ffi::c_int,
            b"(%p)->(%s %s)\n\0" as *const u8 as *const ::core::ffi::c_char,
            This,
            debugstr_variant(core::ptr::addr_of_mut!(varID)),
            debugstr_w(pszValue as *const WCHAR),
        );
    }
    return E_NOTIMPL;
}

unsafe extern "C" fn impl_from_Window_OleWindow(iface: *mut IOleWindow) -> *mut Window {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(Window, IOleWindow_iface)) as *mut Window;
}

unsafe extern "system" fn Window_OleWindow_QueryInterface(iface: *mut IOleWindow, riid: *const IID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut This: *mut Window = impl_from_Window_OleWindow(iface);
    return IAccessible_QueryInterface(core::ptr::addr_of_mut!((*This).IAccessible_iface), riid, ppv);
}

unsafe extern "system" fn Window_OleWindow_AddRef(iface: *mut IOleWindow) -> ULONG {
    let This: *mut Window = impl_from_Window_OleWindow(iface);
    return IAccessible_AddRef(core::ptr::addr_of_mut!((*This).IAccessible_iface));
}

unsafe extern "system" fn Window_OleWindow_Release(iface: *mut IOleWindow) -> ULONG {
    let mut This: *mut Window = impl_from_Window_OleWindow(iface);
    return IAccessible_Release(core::ptr::addr_of_mut!((*This).IAccessible_iface));
}

unsafe extern "system" fn Window_OleWindow_GetWindow(iface: *mut IOleWindow, hwnd: *mut HWND) -> HRESULT {
    let mut This: *mut Window = impl_from_Window_OleWindow(iface);
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/window.c\0" as *const u8 as *const i8,
            b"Window_OleWindow_GetWindow\0" as *const u8 as *const i8,
            367 as i32,
            b"(%p)->(%p)\n\0" as *const u8 as *const i8,
            This,
            hwnd,
        );
    }
    *hwnd = (*This).hwnd;
    return S_OK;
}

unsafe extern "system" fn Window_OleWindow_ContextSensitiveHelp(iface: *mut IOleWindow, fEnterMode: BOOL) -> HRESULT {
    let mut This: *mut Window = impl_from_Window_OleWindow(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_FIXME as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/window.c\0".as_ptr() as *const i8,
            b"Window_OleWindow_ContextSensitiveHelp\0".as_ptr() as *const i8,
            376i32,
            b"(%p)->(%x)\n\0".as_ptr() as *const i8,
            This,
            fEnterMode,
        );
    }
    return E_NOTIMPL;
}

unsafe extern "C" fn impl_from_Window_EnumVARIANT(iface: *mut IEnumVARIANT) -> *mut Window {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(Window, IEnumVARIANT_iface)) as *mut Window;
}

unsafe extern "system" fn Window_EnumVARIANT_QueryInterface(iface: *mut IEnumVARIANT, riid: *const IID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut This: *mut Window = impl_from_Window_EnumVARIANT(iface);
    return IAccessible_QueryInterface(core::ptr::addr_of_mut!((*This).IAccessible_iface), riid, ppv);
}

unsafe extern "system" fn Window_EnumVARIANT_AddRef(iface: *mut IEnumVARIANT) -> ULONG {
    let mut This: *mut Window = impl_from_Window_EnumVARIANT(iface);
    return IAccessible_AddRef(core::ptr::addr_of_mut!((*This).IAccessible_iface));
}

unsafe extern "system" fn Window_EnumVARIANT_Release(iface: *mut IEnumVARIANT) -> ULONG {
    let mut This: *mut Window = impl_from_Window_EnumVARIANT(iface);
    return IAccessible_Release(core::ptr::addr_of_mut!((*This).IAccessible_iface));
}

unsafe extern "system" fn Window_EnumVARIANT_Next(
    iface: *mut IEnumVARIANT,
    celt: ULONG,
    rgVar: *mut VARIANT,
    pCeltFetched: *mut ULONG,
) -> HRESULT {
    let This: *mut Window = impl_from_Window_EnumVARIANT(iface);
    return E_NOTIMPL;
}

unsafe extern "system" fn Window_EnumVARIANT_Skip(iface: *mut IEnumVARIANT, celt: ULONG) -> HRESULT {
    let This: *mut Window = impl_from_Window_EnumVARIANT(iface);
    if ((*__wine_dbch___default).flags as i32) & ((1i32) << (__WINE_DBCL_FIXME as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/window.c\0"
                .as_ptr() as *const i8,
            b"Window_EnumVARIANT_Skip\0".as_ptr() as *const i8,
            422,
            b"(%p)->(%lu)\n\0".as_ptr() as *const i8,
            This,
            celt,
        );
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn Window_EnumVARIANT_Reset(iface: *mut IEnumVARIANT) -> HRESULT {
    let mut This: *mut Window = impl_from_Window_EnumVARIANT(iface);
    FIXME!("(%p)\n", This);
    return E_NOTIMPL;
}

unsafe extern "system" fn Window_EnumVARIANT_Clone(iface: *mut IEnumVARIANT, ppEnum: *mut *mut IEnumVARIANT) -> HRESULT {
    let mut This: *mut Window = impl_from_Window_EnumVARIANT(iface);
    return E_NOTIMPL;
}

unsafe extern "C" fn impl_from_Window_ServiceProvider(iface: *mut IServiceProvider) -> *mut Window {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(Window, IServiceProvider_iface)) as *mut Window;
}

unsafe extern "system" fn Window_ServiceProvider_QueryInterface(iface: *mut IServiceProvider, riid: *const IID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut This: *mut Window = impl_from_Window_ServiceProvider(iface);
    return IAccessible_QueryInterface(core::ptr::addr_of_mut!((*This).IAccessible_iface), riid, ppv);
}

unsafe extern "system" fn Window_ServiceProvider_AddRef(iface: *mut IServiceProvider) -> ULONG {
    let mut This: *mut Window = impl_from_Window_ServiceProvider(iface);
    return IAccessible_AddRef(core::ptr::addr_of_mut!((*This).IAccessible_iface));
}

unsafe extern "system" fn Window_ServiceProvider_Release(mut iface: *mut IServiceProvider) -> ULONG {
    let mut This: *mut Window = impl_from_Window_ServiceProvider(iface);
    return IAccessible_Release(core::ptr::addr_of_mut!((*This).IAccessible_iface));
}

unsafe extern "system" fn Window_ServiceProvider_QueryService(
    iface: *mut IServiceProvider,
    guid_service: *const GUID,
    riid: *const IID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut This: *mut Window = impl_from_Window_ServiceProvider(iface);

    *ppv = core::ptr::null_mut();
    if IsEqualIID!(guid_service, &raw const IIS_IsOleaccProxy) != 0 {
        return IAccessible_QueryInterface(&raw mut (*This).IAccessible_iface, riid, ppv);
    }

    return E_INVALIDARG;
}
#[no_mangle]
pub unsafe extern "C" fn create_window_object(hwnd: HWND, iid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut window: *mut Window = core::ptr::null_mut();
    let mut hres: HRESULT;

    if ((IsWindow(hwnd)) == 0) {
        return E_FAIL;
    }

    window = heap_alloc_zero((core::mem::size_of::<Window>() as SIZE_T) as _) as *mut Window;
    if window.is_null() {
        return E_OUTOFMEMORY;
    }

    find_class_data(hwnd, core::ptr::addr_of!(classes) as *const win_class_data);

    (*window).IAccessible_iface.lpVtbl = core::ptr::addr_of!(WindowVtbl);
    (*window).IOleWindow_iface.lpVtbl = core::ptr::addr_of!(WindowOleWindowVtbl);
    (*window).IEnumVARIANT_iface.lpVtbl = core::ptr::addr_of!(WindowEnumVARIANTVtbl);
    (*window).IServiceProvider_iface.lpVtbl = core::ptr::addr_of!(WindowServiceProviderVtbl);
    (*window).ref_ = 1;
    (*window).hwnd = hwnd;

    hres = IAccessible_QueryInterface(core::ptr::addr_of_mut!((*window).IAccessible_iface), iid, obj);
    IAccessible_Release(core::ptr::addr_of_mut!((*window).IAccessible_iface));
    return hres;
}

// COM vtable tables — generated from window.c by vtbl_gen.py
/// window.c:309 `static const IAccessibleVtbl WindowVtbl`  (slot order: rust:types_amd64.rs)
pub static WindowVtbl: IAccessibleVtbl = IAccessibleVtbl {
    QueryInterface: Some(Window_QueryInterface),
    AddRef: Some(Window_AddRef),
    Release: Some(Window_Release),
    GetTypeInfoCount: Some(Window_GetTypeInfoCount),
    GetTypeInfo: Some(Window_GetTypeInfo),
    GetIDsOfNames: Some(Window_GetIDsOfNames),
    Invoke: Some(Window_Invoke),
    get_accParent: Some(Window_get_accParent),
    get_accChildCount: Some(Window_get_accChildCount),
    get_accChild: Some(Window_get_accChild),
    get_accName: Some(Window_get_accName),
    get_accValue: Some(Window_get_accValue),
    get_accDescription: Some(Window_get_accDescription),
    get_accRole: Some(Window_get_accRole),
    get_accState: Some(Window_get_accState),
    get_accHelp: Some(Window_get_accHelp),
    get_accHelpTopic: Some(Window_get_accHelpTopic),
    get_accKeyboardShortcut: Some(Window_get_accKeyboardShortcut),
    get_accFocus: Some(Window_get_accFocus),
    get_accSelection: Some(Window_get_accSelection),
    get_accDefaultAction: Some(Window_get_accDefaultAction),
    accSelect: Some(Window_accSelect),
    accLocation: Some(Window_accLocation),
    accNavigate: Some(Window_accNavigate),
    accHitTest: Some(Window_accHitTest),
    accDoDefaultAction: Some(Window_accDoDefaultAction),
    put_accName: Some(Window_put_accName),
    put_accValue: Some(Window_put_accValue),
};

/// window.c:380 `static const IOleWindowVtbl WindowOleWindowVtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static WindowOleWindowVtbl: IOleWindowVtbl = IOleWindowVtbl {
    QueryInterface: Some(Window_OleWindow_QueryInterface),
    AddRef: Some(Window_OleWindow_AddRef),
    Release: Some(Window_OleWindow_Release),
    GetWindow: Some(Window_OleWindow_GetWindow),
    ContextSensitiveHelp: Some(Window_OleWindow_ContextSensitiveHelp),
};

/// window.c:440 `static const IEnumVARIANTVtbl WindowEnumVARIANTVtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static WindowEnumVARIANTVtbl: IEnumVARIANTVtbl = IEnumVARIANTVtbl {
    QueryInterface: Some(Window_EnumVARIANT_QueryInterface),
    AddRef: Some(Window_EnumVARIANT_AddRef),
    Release: Some(Window_EnumVARIANT_Release),
    Next: Some(Window_EnumVARIANT_Next),
    Skip: Some(Window_EnumVARIANT_Skip),
    Reset: Some(Window_EnumVARIANT_Reset),
    Clone: Some(Window_EnumVARIANT_Clone),
};

/// window.c:487 `static const IServiceProviderVtbl WindowServiceProviderVtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static WindowServiceProviderVtbl: IServiceProviderVtbl = IServiceProviderVtbl {
    QueryInterface: Some(Window_ServiceProvider_QueryInterface),
    AddRef: Some(Window_ServiceProvider_AddRef),
    Release: Some(Window_ServiceProvider_Release),
    QueryService: Some(Window_ServiceProvider_QueryService),
};
