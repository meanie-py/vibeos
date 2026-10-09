use crate::comctl32::comctl32undoc::Alloc;
use crate::comctl32::commctrl::COMCTL32_IsReflectedMessage;
use crate::comctl32::comctl32undoc::Free;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_nativefont);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const WC_NATIVEFONTCTLW: [::core::ffi::c_ushort; 14] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_ushort; 14]>(
        *b"N\0a\0t\0i\0v\0e\0F\0o\0n\0t\0C\0t\0l\0\0\0",
    )
};

#[repr(C)]
pub struct NATIVEFONT_INFO {
    pub hwndSelf: HWND,
}

pub type PNATIVEFONT_INFO = *mut NATIVEFONT_INFO;

unsafe extern "C" fn NATIVEFONT_Create(hwnd: HWND) -> LRESULT {
    let mut infoPtr: *mut NATIVEFONT_INFO = core::ptr::null_mut::<NATIVEFONT_INFO>();

    /* allocate memory for info structure */
    infoPtr = Alloc(core::mem::size_of::<NATIVEFONT_INFO>() as DWORD) as *mut NATIVEFONT_INFO;
    SetWindowLongPtrW(hwnd, 0, (infoPtr as DWORD_PTR) as LONG_PTR);

    /* initialize info structure */
    (*infoPtr).hwndSelf = hwnd;

    return 0 as LRESULT;
}

unsafe extern "C" fn NATIVEFONT_Destroy(infoPtr: *mut NATIVEFONT_INFO) -> LRESULT {
    SetWindowLongPtrW((*infoPtr).hwndSelf, 0, 0);
    Free(infoPtr as LPVOID);
    return 0;
}
unsafe extern "system" fn NATIVEFONT_WindowProc(
    mut hwnd: HWND,
    mut uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut infoPtr: *mut NATIVEFONT_INFO = NATIVEFONT_GetInfoPtr!(hwnd);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/nativefont.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"NATIVEFONT_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
            80 as ::core::ffi::c_int,
            b"hwnd=%p msg=%04x wparam=%08lx lparam=%08lx\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hwnd,
            uMsg,
            wParam,
            lParam,
        );
    }
    if infoPtr.is_null() && (uMsg != WM_CREATE) {
        return DefWindowProcW(hwnd, uMsg, wParam, lParam);
    }
    match uMsg {
        WM_CREATE => {
            return NATIVEFONT_Create(hwnd);
        }
        WM_DESTROY => {
            return NATIVEFONT_Destroy(infoPtr);
        }
        WM_MOVE
        | WM_SIZE
        | WM_SHOWWINDOW
        | WM_WINDOWPOSCHANGING
        | WM_WINDOWPOSCHANGED
        | WM_SETFONT
        | WM_GETDLGCODE => {
            /* FIXME("message %04x seen but stubbed\n", uMsg); */
            return DefWindowProcW(hwnd, uMsg, wParam, lParam);
        }
        _ => {
            if (uMsg >= WM_USER)
                && (uMsg < WM_APP)
                && !(COMCTL32_IsReflectedMessage(uMsg) != 0)
            {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl_0,
                        __dbch_0,
                        b"dll/win32/comctl32/nativefont.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"NATIVEFONT_WindowProc\0" as *const u8
                            as *const ::core::ffi::c_char,
                        106 as ::core::ffi::c_int,
                        b"unknown msg %04x wp=%08lx lp=%08lx\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        uMsg,
                        wParam,
                        lParam,
                    );
                }
            }
            return DefWindowProcW(hwnd, uMsg, wParam, lParam);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn NATIVEFONT_Register() {
    let mut wndClass: WNDCLASSW = _WNDCLASSW {
        style: 0,
        lpfnWndProc: None,
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: ::core::ptr::null_mut::<HINSTANCE__>(),
        hIcon: ::core::ptr::null_mut::<HICON__>(),
        hCursor: ::core::ptr::null_mut::<HICON__>(),
        hbrBackground: ::core::ptr::null_mut::<HBRUSH__>(),
        lpszMenuName: ::core::ptr::null::<WCHAR>(),
        lpszClassName: ::core::ptr::null::<WCHAR>(),
    };
    ZeroMemory!(
        &raw mut wndClass as *mut ::core::ffi::c_void,
        core::mem::size_of::<WNDCLASSW>() as usize,
    );
    wndClass.style = CS_GLOBALCLASS as UINT;
    wndClass.lpfnWndProc = Some(NATIVEFONT_WindowProc);
    wndClass.cbClsExtra = 0 as ::core::ffi::c_int;
    wndClass.cbWndExtra =
        core::mem::size_of::<*mut NATIVEFONT_INFO>() as ::core::ffi::c_int;
    wndClass.hCursor = LoadCursorW(
        0 as HINSTANCE,
        IDC_ARROW as LPWSTR as LPCWSTR,
    );
    wndClass.hbrBackground =
        (COLOR_BTNFACE + (1 as ::core::ffi::c_int) as u32) as HBRUSH;
    wndClass.lpszClassName = WC_NATIVEFONTCTLW.as_ptr() as LPCWSTR;
    RegisterClassW(&raw mut wndClass);
}
#[no_mangle]
pub unsafe extern "C" fn NATIVEFONT_Unregister() {
    UnregisterClassW((WC_NATIVEFONTCTLW).as_ptr(), core::ptr::null_mut());
}
