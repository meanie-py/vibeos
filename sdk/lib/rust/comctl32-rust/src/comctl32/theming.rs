use crate::comctl32::theme_scrollbar::THEMING_ScrollbarSubclassProc;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_theming);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const WC_SCROLLBARW: [::core::ffi::c_ushort; 10] = unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_ushort; 10]>(
        *b"S\0c\0r\0o\0l\0l\0B\0a\0r\0\0\0",
    )
};

static mut subclasses: [ThemingSubclass; 1] = unsafe {
    [ThemingSubclass {
        className: WC_SCROLLBARW.as_ptr() as *const WCHAR,
        subclassProc: Some(
            THEMING_ScrollbarSubclassProc,
        ),
    }]
};

pub const NUM_SUBCLASSES: usize = (::core::mem::size_of::<[ThemingSubclass; 1]>() as usize)
    .wrapping_div(::core::mem::size_of::<ThemingSubclass>() as usize);

static mut originalProcs: [WNDPROC; 1] = [None; 1];

static mut atRefDataProp: ATOM = 0;

static mut atSubclassProp: ATOM = 0;

static mut subclassProcs: [WNDPROC; 1] = [Some(subclass_proc0)];

/// theming.c:72 `MAKE_SUBCLASS_PROC(0)` -- a macro-generated function, so the scanner never
/// planned it. The subclass number goes in a prop for THEMING_CallOriginalClass, then the
/// subclass proc gets the window's ref data. (Only proc 0 exists under __REACTOS__.)
unsafe extern "system" fn subclass_proc0(wnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    SetPropW(wnd, MAKEINTATOM!(atSubclassProp) as LPCWSTR, 0usize as HANDLE);
    let refData: ULONG_PTR = GetPropW(wnd, MAKEINTATOM!(atRefDataProp) as LPCWSTR) as ULONG_PTR;
    TRACE!("%d; (%p, %x, %lx, %lx, %lx)\n", 0, wnd, msg, wParam, lParam, refData);
    let result: LRESULT =
        (subclasses[0].subclassProc.unwrap_unchecked())(wnd, msg, wParam, lParam, refData);
    TRACE!("result = %lx\n", result);
    result
}

pub type THEMING_SUBCLASSPROC =
    Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM, ULONG_PTR) -> LRESULT>;

#[repr(C)]
pub struct ThemingSubclass {
    pub className: *const WCHAR,
    pub subclassProc: THEMING_SUBCLASSPROC,
}

pub type PThemingSubclass = *mut ThemingSubclass;

unsafe impl Sync for ThemingSubclass {}

pub static WC_SCROLLBARW_DATA: [WCHAR; 10] = [0x53, 0x63, 0x72, 0x6F, 0x6C, 0x6C, 0x42, 0x61, 0x72, 0x00];

#[no_mangle]
pub unsafe extern "C" fn THEMING_Initialize(mut hActCtx5: HANDLE, mut hActCtx6: HANDLE) {
    let mut i: ::core::ffi::c_uint = 0;
    static subclassPropName: [WCHAR; 17] = [
    0x0043u16, 0x0043, 0x0033, 0x0032, 0x0054, 0x0068, 0x0065, 0x006D, 0x0069, 0x006E,
    0x0067, 0x0053, 0x0075, 0x0062, 0x0043, 0x006C, 0x0000,
];
    static refDataPropName: [WCHAR; 16] = [
    0x0043u16, 0x0043, 0x0033, 0x0032, 0x0054, 0x0068, 0x0065, 0x006D, 0x0069, 0x006E,
    0x0067, 0x0044, 0x0061, 0x0074, 0x0061, 0x0000,
];
    let mut ulCookie: ULONG_PTR = 0;
    let mut ret: BOOL = 0;
    let mut bActivated: BOOL = 0;
    atSubclassProp = GlobalAddAtomW(&raw const subclassPropName as LPCWSTR);
    atRefDataProp = GlobalAddAtomW(&raw const refDataPropName as LPCWSTR);
    i = 0;
    while (i as usize) < NUM_SUBCLASSES {
        'body: {
            let mut class: WNDCLASSEXW = core::mem::zeroed();

            class.cbSize = ::core::mem::size_of::<WNDCLASSEXW>() as UINT;

            bActivated = ActivateActCtx(hActCtx5, &raw mut ulCookie);
            ret = GetClassInfoExW(
                ::core::ptr::null_mut::<HINSTANCE__>(),
                subclasses[i as usize].className as LPCWSTR,
                &raw mut class,
            );
            if bActivated != 0 {
                DeactivateActCtx(0, ulCookie);
            }

            if ret == 0 {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/win32/comctl32/theming.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"THEMING_Initialize\0" as *const u8 as *const ::core::ffi::c_char,
                        144 as ::core::ffi::c_int,
                        b"Could not retrieve information for class %s\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        debugstr_w(subclasses[i as usize].className),
                    );
                }
                break 'body;
            }

            originalProcs[i as usize] = class.lpfnWndProc;
            class.lpfnWndProc = subclassProcs[i as usize];
            class.style |= CS_GLOBALCLASS as UINT;
            class.hInstance = COMCTL32_hModule as HINSTANCE;

            if class.lpfnWndProc.is_none() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl_0,
                        __dbch_0,
                        b"dll/win32/comctl32/theming.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"THEMING_Initialize\0" as *const u8 as *const ::core::ffi::c_char,
                        157 as ::core::ffi::c_int,
                        b"Missing proc for class %s\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        debugstr_w(subclasses[i as usize].className),
                    );
                }
                break 'body;
            }

            bActivated = ActivateActCtx(hActCtx6, &raw mut ulCookie);
            if RegisterClassExW(&raw mut class) == 0 {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_1: __wine_debug_class = __WINE_DBCL_WARN;
                    ros_dbg_log(
                        __dbcl_1,
                        __dbch_1,
                        b"dll/win32/comctl32/theming.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"THEMING_Initialize\0" as *const u8 as *const ::core::ffi::c_char,
                        168 as ::core::ffi::c_int,
                        b"Could not re-register class %s: %x\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        debugstr_w(subclasses[i as usize].className),
                        GetLastError(),
                    );
                }
            } else if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_2,
                    __dbch_2,
                    b"dll/win32/comctl32/theming.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"THEMING_Initialize\0" as *const u8 as *const ::core::ffi::c_char,
                    177 as ::core::ffi::c_int,
                    b"Re-registered class %s\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    debugstr_w(subclasses[i as usize].className),
                );
            }

            if bActivated != 0 {
                DeactivateActCtx(0, ulCookie);
            }
        }
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn THEMING_Uninitialize() {
    let mut i: ::core::ffi::c_uint = 0;

    if atSubclassProp == 0 {
        return;
    }

    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < NUM_SUBCLASSES as usize {
        UnregisterClassW(
            subclasses[i as usize].className as LPCWSTR,
            core::ptr::null_mut(),
        );
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn THEMING_CallOriginalClass(wnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let subclass: INT_PTR = GetPropW(wnd, MAKEINTATOM!(atSubclassProp)) as INT_PTR;
    let oldProc: WNDPROC = originalProcs[subclass as usize];
    return CallWindowProcW(oldProc, wnd, msg, wParam, lParam);
}
