use crate::comctl32::commctrl::COMCTL32_IsReflectedMessage;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_commctrl);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const FLATSB_CLASSW: [WCHAR; 15] = [
    'f' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    't' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'b' as i32 as WCHAR,
    '_' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    's' as i32 as WCHAR,
    's' as i32 as WCHAR,
    '3' as i32 as WCHAR,
    '2' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];


#[repr(C)]
pub struct FLATSB_INFO {
    pub dwDummy: DWORD, /* just to keep the compiler happy ;-) */
}

pub type LPFLATSB_INFO = *mut FLATSB_INFO;

#[no_mangle]
pub unsafe extern "system" fn InitializeFlatSB(hwnd: HWND) -> BOOL {
    // TRACE("[%p]\n", hwnd);
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn UninitializeFlatSB(hwnd: HWND) -> HRESULT {
    // TRACE("[%p]\n", hwnd);
    S_FALSE
}

#[no_mangle]
pub unsafe extern "system" fn FlatSB_GetScrollProp(
    hwnd: HWND,
    propIndex: INT,
    prop: LPINT,
) -> BOOL {
    // TRACE("[%p] propIndex=%d\n", hwnd, propIndex);
    FALSE
}

#[no_mangle]
pub unsafe extern "system" fn FlatSB_SetScrollProp(
    hwnd: HWND,
    index: UINT,
    newValue: INT,
    flag: BOOL,
) -> BOOL {
    // TRACE("[%p] index=%u newValue=%d flag=%d\n", hwnd, index, newValue, flag);
    FALSE
}

#[no_mangle]
pub unsafe extern "system" fn FlatSB_EnableScrollBar(
    hwnd: HWND,
    nBar: i32,
    flags: UINT,
) -> BOOL {
    EnableScrollBar(hwnd, (nBar) as UINT, flags)
}

#[no_mangle]
pub unsafe extern "system" fn FlatSB_ShowScrollBar(hwnd: HWND, nBar: i32, fShow: BOOL) -> BOOL {
    ShowScrollBar(hwnd, nBar, fShow)
}

#[no_mangle]
pub unsafe extern "system" fn FlatSB_GetScrollRange(
    hwnd: HWND,
    nBar: i32,
    min: LPINT,
    max: LPINT,
) -> BOOL {
    GetScrollRange(hwnd, nBar, min, max)
}

#[no_mangle]
pub unsafe extern "system" fn FlatSB_GetScrollInfo(
    hwnd: HWND,
    nBar: i32,
    info: LPSCROLLINFO,
) -> BOOL {
    GetScrollInfo(hwnd, nBar, info)
}

#[no_mangle]
pub unsafe extern "system" fn FlatSB_GetScrollPos(hwnd: HWND, nBar: i32) -> INT {
    GetScrollPos(hwnd, nBar)
}

#[no_mangle]
pub unsafe extern "system" fn FlatSB_SetScrollPos(
    hwnd: HWND,
    nBar: i32,
    pos: INT,
    bRedraw: BOOL,
) -> INT {
    SetScrollPos(hwnd, nBar, pos, bRedraw)
}

#[no_mangle]
pub unsafe extern "system" fn FlatSB_SetScrollInfo(
    hwnd: HWND,
    nBar: i32,
    info: LPSCROLLINFO,
    bRedraw: BOOL,
) -> INT {
    SetScrollInfo(hwnd, nBar, info, bRedraw)
}

#[no_mangle]
pub unsafe extern "system" fn FlatSB_SetScrollRange(
    hwnd: HWND,
    nBar: i32,
    min: INT,
    max: INT,
    bRedraw: BOOL,
) -> INT {
    SetScrollRange(hwnd, nBar, min, max, bRedraw)
}

unsafe extern "C" fn FlatSB_Create(hwnd: HWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    // TRACE("[%p] wParam=%04lx lParam=%08lx\n", hwnd, wParam, lParam);
    0
}

unsafe extern "C" fn FlatSB_Destroy(hwnd: HWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    // TRACE("[%p] wParam=%04lx lParam=%08lx\n", hwnd, wParam, lParam);
    0
}

unsafe extern "system" fn FlatSB_WindowProc(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    if GetWindowLongPtrW(hwnd, 0) == 0 && uMsg != WM_CREATE {
        return DefWindowProcW(hwnd, uMsg, wParam, lParam);
    }
    match uMsg {
        m if m == WM_CREATE => return FlatSB_Create(hwnd, wParam, lParam),
        m if m == WM_DESTROY => return FlatSB_Destroy(hwnd, wParam, lParam),
        _ => {
            if uMsg >= WM_USER && uMsg < WM_APP && COMCTL32_IsReflectedMessage(uMsg) == 0 {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/win32/comctl32/flatsb.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"FlatSB_WindowProc\0" as *const u8
                            as *const ::core::ffi::c_char,
                        260 as ::core::ffi::c_int,
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
pub unsafe extern "C" fn FLATSB_Register() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    ZeroMemory!(
        core::ptr::addr_of_mut!(wndClass) as *mut core::ffi::c_void,
        core::mem::size_of::<WNDCLASSW>(),
    );
    wndClass.style = CS_GLOBALCLASS as UINT;
    wndClass.lpfnWndProc = Some(FlatSB_WindowProc);
    wndClass.cbClsExtra = 0 as i32;
    wndClass.cbWndExtra = core::mem::size_of::<*mut FLATSB_INFO>() as i32;
    wndClass.hCursor = LoadCursorW(
        core::ptr::null_mut(),
        IDC_ARROW as usize as LPWSTR,
    );
    wndClass.hbrBackground = (COLOR_WINDOW + 1) as HBRUSH;
    wndClass.lpszClassName = FLATSB_CLASSW.as_ptr() as LPCWSTR;

    RegisterClassW(core::ptr::addr_of_mut!(wndClass));
}
#[no_mangle]
pub unsafe extern "C" fn FLATSB_Unregister() {
    UnregisterClassW((FLATSB_CLASSW).as_ptr(), core::ptr::null_mut());
}
