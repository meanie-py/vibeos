use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_theme_scroll);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const WC_SCROLLBARW: [::core::ffi::c_ushort; 10] = unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_ushort; 10]>(
        *b"S\0c\0r\0o\0l\0l\0B\0a\0r\0\0\0",
    )
};

static mut tracking_win: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut tracking_hot_part: SCROLL_HITTEST = (SCROLL_NOWHERE as u32);

pub const SCROLL_MIN_THUMB: i32 = 6;

pub const SCROLL_MIN_RECT: i32 = 4;

pub const SCROLL_NOWHERE: i32 = 0;
pub const SCROLL_TOP_ARROW: i32 = 1;
pub const SCROLL_TOP_RECT: i32 = 2;
pub const SCROLL_THUMB: i32 = 3;
pub const SCROLL_BOTTOM_RECT: i32 = 4;
pub const SCROLL_BOTTOM_ARROW: i32 = 5;

unsafe extern "C" fn calc_thumb_dimensions(
    mut size: ::core::ffi::c_uint,
    mut si: *mut SCROLLINFO,
    mut thumbpos: *mut ::core::ffi::c_uint,
    mut thumbsize: *mut ::core::ffi::c_uint,
) {
    if size <= SCROLL_MIN_RECT as ::core::ffi::c_uint {
        *thumbsize = 0 as ::core::ffi::c_uint;
        *thumbpos = *thumbsize;
    } else if (*si).nPage > ((*si).nMax - (*si).nMin) as UINT {
        *thumbsize = 0 as ::core::ffi::c_uint;
        *thumbpos = *thumbsize;
    } else {
        if (*si).nPage > 0 as UINT {
            *thumbsize = MulDiv(
                size as ::core::ffi::c_int,
                (*si).nPage as ::core::ffi::c_int,
                (*si).nMax - (*si).nMin + 1 as ::core::ffi::c_int,
            ) as ::core::ffi::c_uint;
            if *thumbsize < SCROLL_MIN_THUMB as ::core::ffi::c_uint {
                *thumbsize = SCROLL_MIN_THUMB as ::core::ffi::c_uint;
            }
        } else {
            *thumbsize = GetSystemMetrics((SM_CXVSCROLL) as i32) as ::core::ffi::c_uint;
        }
        if size < *thumbsize {
            *thumbsize = 0 as ::core::ffi::c_uint;
            *thumbpos = *thumbsize;
        } else {
            let mut max: ::core::ffi::c_int = ((*si).nMax as UINT).wrapping_sub(
                max(
                    (*si).nPage.wrapping_sub(1 as UINT),
                    0 as UINT,
                ),
            ) as ::core::ffi::c_int;
            size = size.wrapping_sub(*thumbsize);
            if (*si).nMin >= max {
                *thumbpos = 0 as ::core::ffi::c_uint;
            } else {
                *thumbpos = MulDiv(
                    size as ::core::ffi::c_int,
                    (*si).nTrackPos - (*si).nMin,
                    max - (*si).nMin,
                ) as ::core::ffi::c_uint;
            }
        }
    }
}

unsafe extern "C" fn hit_test(mut hwnd: HWND, mut theme: HTHEME, mut pt: POINT) -> SCROLL_HITTEST {
    let mut r: RECT = core::mem::zeroed();
    let style: DWORD = GetWindowLongW(hwnd, GWL_STYLE) as DWORD;
    let vertical: BOOL = (style & SBS_VERT as DWORD) as BOOL;
    let mut sz: SIZE = core::mem::zeroed();
    let mut si: SCROLLINFO = core::mem::zeroed();
    let mut offset: ::core::ffi::c_uint = 0;
    let mut size: ::core::ffi::c_uint = 0;
    let mut upsize: ::core::ffi::c_uint = 0;
    let mut downsize: ::core::ffi::c_uint = 0;
    let mut thumbpos: ::core::ffi::c_uint = 0;
    let mut thumbsize: ::core::ffi::c_uint = 0;

    GetWindowRect(hwnd, core::ptr::addr_of_mut!(r));
    OffsetRect(
        core::ptr::addr_of_mut!(r),
        -(r.left as ::core::ffi::c_int),
        -(r.top as ::core::ffi::c_int),
    );

    if vertical != 0 {
        offset = pt.y as ::core::ffi::c_uint;
        size = r.bottom as ::core::ffi::c_uint;

        if GetThemePartSize(
            theme,
            core::ptr::null_mut(),
            SBP_ARROWBTN as ::core::ffi::c_int,
            ABS_UPNORMAL as ::core::ffi::c_int,
            core::ptr::null_mut(),
            TS_DRAW,
            core::ptr::addr_of_mut!(sz),
        ) < 0
        {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/comctl32/theme_scrollbar.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"hit_test\0" as *const u8 as *const ::core::ffi::c_char,
                    99 as ::core::ffi::c_int,
                    b"Could not get up arrow size.\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            upsize = 0 as ::core::ffi::c_uint;
        } else {
            upsize = sz.cy as ::core::ffi::c_uint;
        }

        if GetThemePartSize(
            theme,
            core::ptr::null_mut(),
            SBP_ARROWBTN as ::core::ffi::c_int,
            ABS_DOWNNORMAL as ::core::ffi::c_int,
            core::ptr::null_mut(),
            TS_DRAW,
            core::ptr::addr_of_mut!(sz),
        ) < 0
        {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_WARN;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/theme_scrollbar.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"hit_test\0" as *const u8 as *const ::core::ffi::c_char,
                    105 as ::core::ffi::c_int,
                    b"Could not get down arrow size.\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            downsize = 0 as ::core::ffi::c_uint;
        } else {
            downsize = sz.cy as ::core::ffi::c_uint;
        }
    } else {
        offset = pt.x as ::core::ffi::c_uint;
        size = r.right as ::core::ffi::c_uint;

        if GetThemePartSize(
            theme,
            core::ptr::null_mut(),
            SBP_ARROWBTN as ::core::ffi::c_int,
            ABS_LEFTNORMAL as ::core::ffi::c_int,
            core::ptr::null_mut(),
            TS_DRAW,
            core::ptr::addr_of_mut!(sz),
        ) < 0
        {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_WARN;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/win32/comctl32/theme_scrollbar.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"hit_test\0" as *const u8 as *const ::core::ffi::c_char,
                    114 as ::core::ffi::c_int,
                    b"Could not get left arrow size.\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            upsize = 0 as ::core::ffi::c_uint;
        } else {
            upsize = sz.cx as ::core::ffi::c_uint;
        }

        if GetThemePartSize(
            theme,
            core::ptr::null_mut(),
            SBP_ARROWBTN as ::core::ffi::c_int,
            ABS_RIGHTNORMAL as ::core::ffi::c_int,
            core::ptr::null_mut(),
            TS_DRAW,
            core::ptr::addr_of_mut!(sz),
        ) < 0
        {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
                != 0
            {
                let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_2: __wine_debug_class = __WINE_DBCL_WARN;
                ros_dbg_log(
                    __dbcl_2,
                    __dbch_2,
                    b"dll/win32/comctl32/theme_scrollbar.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"hit_test\0" as *const u8 as *const ::core::ffi::c_char,
                    120 as ::core::ffi::c_int,
                    b"Could not get right arrow size.\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            downsize = 0 as ::core::ffi::c_uint;
        } else {
            downsize = sz.cx as ::core::ffi::c_uint;
        }
    }

    if pt.x < 0
        || pt.x > r.right
        || pt.y < 0
        || pt.y > r.bottom
    {
        return (SCROLL_NOWHERE) as SCROLL_HITTEST;
    }

    if size
        < (SCROLL_MIN_RECT as ::core::ffi::c_uint)
            .wrapping_add(upsize)
            .wrapping_add(downsize)
    {
        downsize = size
            .wrapping_sub(SCROLL_MIN_RECT as ::core::ffi::c_uint)
            .wrapping_div(2 as ::core::ffi::c_uint);
        upsize = downsize;
    }

    if offset < upsize {
        return (SCROLL_TOP_ARROW) as SCROLL_HITTEST;
    }

    if offset > size.wrapping_sub(downsize) {
        return (SCROLL_BOTTOM_ARROW) as SCROLL_HITTEST;
    }

    si.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
    si.fMask = SIF_ALL as UINT;
    if GetScrollInfo(hwnd, SB_CTL as ::core::ffi::c_int, core::ptr::addr_of_mut!(si)) == 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
            != 0
        {
            let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_3: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl_3,
                __dbch_3,
                b"dll/win32/comctl32/theme_scrollbar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"hit_test\0" as *const u8 as *const ::core::ffi::c_char,
                141 as ::core::ffi::c_int,
                b"GetScrollInfo failed.\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return (SCROLL_NOWHERE) as SCROLL_HITTEST;
    }

    calc_thumb_dimensions(
        size.wrapping_sub(upsize).wrapping_sub(downsize),
        core::ptr::addr_of_mut!(si),
        core::ptr::addr_of_mut!(thumbpos),
        core::ptr::addr_of_mut!(thumbsize),
    );

    if offset < upsize.wrapping_add(thumbpos) {
        return (SCROLL_TOP_RECT) as SCROLL_HITTEST;
    } else if offset < upsize.wrapping_add(thumbpos).wrapping_add(thumbsize) {
        return (SCROLL_THUMB) as SCROLL_HITTEST;
    } else {
        return (SCROLL_BOTTOM_RECT) as SCROLL_HITTEST;
    }
}

unsafe extern "C" fn redraw_part(hwnd: HWND, theme: HTHEME, part: SCROLL_HITTEST) {
    let style: DWORD = GetWindowLongW(hwnd, GWL_STYLE) as DWORD;
    let vertical: BOOL = (style & SBS_VERT as DWORD) as BOOL;
    let mut sz: SIZE = SIZE { cx: 0, cy: 0 };
    let mut r: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut partrect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut size: ::core::ffi::c_uint = 0;
    let mut upsize: ::core::ffi::c_uint = 0;
    let mut downsize: ::core::ffi::c_uint = 0;

    if part as ::core::ffi::c_uint == SCROLL_NOWHERE as ::core::ffi::c_uint {
        /* redraw everything */
        InvalidateRect(hwnd, core::ptr::null(), TRUE);
        return;
    }

    GetWindowRect(hwnd, core::ptr::addr_of_mut!(r));
    OffsetRect(
        core::ptr::addr_of_mut!(r),
        -(r.left as ::core::ffi::c_int),
        -(r.top as ::core::ffi::c_int),
    );

    if vertical != 0 {
        size = r.bottom as ::core::ffi::c_uint;

        if GetThemePartSize(
            theme,
            core::ptr::null_mut(),
            SBP_ARROWBTN as i32,
            ABS_UPNORMAL as i32,
            core::ptr::null_mut(),
            TS_DRAW,
            core::ptr::addr_of_mut!(sz),
        ) < 0
        {
            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_WARN as ::core::ffi::c_int)))
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_WARN,
                    __wine_dbch___default,
                    b"dll/win32/comctl32/theme_scrollbar.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    b"redraw_part\0".as_ptr() as *const ::core::ffi::c_char,
                    175 as ::core::ffi::c_int,
                    b"Could not get up arrow size.\n\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            upsize = 0 as ::core::ffi::c_uint;
        } else {
            upsize = sz.cy as ::core::ffi::c_uint;
        }

        if GetThemePartSize(
            theme,
            core::ptr::null_mut(),
            SBP_ARROWBTN as i32,
            ABS_DOWNNORMAL as i32,
            core::ptr::null_mut(),
            TS_DRAW,
            core::ptr::addr_of_mut!(sz),
        ) < 0
        {
            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_WARN as ::core::ffi::c_int)))
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_WARN,
                    __wine_dbch___default,
                    b"dll/win32/comctl32/theme_scrollbar.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    b"redraw_part\0".as_ptr() as *const ::core::ffi::c_char,
                    181 as ::core::ffi::c_int,
                    b"Could not get down arrow size.\n\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            downsize = 0 as ::core::ffi::c_uint;
        } else {
            downsize = sz.cy as ::core::ffi::c_uint;
        }
    } else {
        size = r.right as ::core::ffi::c_uint;

        if GetThemePartSize(
            theme,
            core::ptr::null_mut(),
            SBP_ARROWBTN as i32,
            ABS_LEFTNORMAL as i32,
            core::ptr::null_mut(),
            TS_DRAW,
            core::ptr::addr_of_mut!(sz),
        ) < 0
        {
            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_WARN as ::core::ffi::c_int)))
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_WARN,
                    __wine_dbch___default,
                    b"dll/win32/comctl32/theme_scrollbar.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    b"redraw_part\0".as_ptr() as *const ::core::ffi::c_char,
                    189 as ::core::ffi::c_int,
                    b"Could not get left arrow size.\n\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            upsize = 0 as ::core::ffi::c_uint;
        } else {
            upsize = sz.cx as ::core::ffi::c_uint;
        }

        if GetThemePartSize(
            theme,
            core::ptr::null_mut(),
            SBP_ARROWBTN as i32,
            ABS_RIGHTNORMAL as i32,
            core::ptr::null_mut(),
            TS_DRAW,
            core::ptr::addr_of_mut!(sz),
        ) < 0
        {
            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_WARN as ::core::ffi::c_int)))
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_WARN,
                    __wine_dbch___default,
                    b"dll/win32/comctl32/theme_scrollbar.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    b"redraw_part\0".as_ptr() as *const ::core::ffi::c_char,
                    195 as ::core::ffi::c_int,
                    b"Could not get right arrow size.\n\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            downsize = 0 as ::core::ffi::c_uint;
        } else {
            downsize = sz.cx as ::core::ffi::c_uint;
        }
    }

    if size
        < (SCROLL_MIN_RECT as ::core::ffi::c_uint)
            .wrapping_add(upsize)
            .wrapping_add(downsize)
    {
        downsize = size
            .wrapping_sub(SCROLL_MIN_RECT as ::core::ffi::c_uint)
            .wrapping_div(2 as ::core::ffi::c_uint);
        upsize = downsize;
    }

    partrect = r;

    if part as ::core::ffi::c_uint == SCROLL_TOP_ARROW as ::core::ffi::c_uint {
        if vertical != 0 {
            partrect.bottom = (partrect.top as ::core::ffi::c_uint).wrapping_add(upsize) as LONG;
        } else {
            partrect.right = (partrect.left as ::core::ffi::c_uint).wrapping_add(upsize) as LONG;
        }
    } else if part as ::core::ffi::c_uint == SCROLL_BOTTOM_ARROW as ::core::ffi::c_uint {
        if vertical != 0 {
            partrect.top = (partrect.bottom as ::core::ffi::c_uint).wrapping_sub(downsize) as LONG;
        } else {
            partrect.left = (partrect.right as ::core::ffi::c_uint).wrapping_sub(downsize) as LONG;
        }
    } else {
        let mut thumbpos: ::core::ffi::c_uint = 0;
        let mut thumbsize: ::core::ffi::c_uint = 0;
        let mut si: SCROLLINFO = SCROLLINFO {
            cbSize: 0,
            fMask: 0,
            nMin: 0,
            nMax: 0,
            nPage: 0,
            nPos: 0,
            nTrackPos: 0,
        };

        si.cbSize = ::core::mem::size_of::<SCROLLINFO>() as UINT;
        si.fMask = SIF_ALL as UINT;
        if GetScrollInfo(
            hwnd,
            SB_CTL as i32,
            core::ptr::addr_of_mut!(si),
        ) == 0
        {
            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_WARN as ::core::ffi::c_int)))
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_WARN,
                    __wine_dbch___default,
                    b"dll/win32/comctl32/theme_scrollbar.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    b"redraw_part\0".as_ptr() as *const ::core::ffi::c_char,
                    223 as ::core::ffi::c_int,
                    b"GetScrollInfo failed.\n\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            return;
        }

        calc_thumb_dimensions(
            size.wrapping_sub(upsize).wrapping_sub(downsize),
            core::ptr::addr_of_mut!(si),
            core::ptr::addr_of_mut!(thumbpos),
            core::ptr::addr_of_mut!(thumbsize),
        );

        if part as ::core::ffi::c_uint == SCROLL_TOP_RECT as ::core::ffi::c_uint {
            if vertical != 0 {
                partrect.top = (r.top as ::core::ffi::c_uint).wrapping_add(upsize) as LONG;
                partrect.bottom =
                    (partrect.top as ::core::ffi::c_uint).wrapping_add(thumbpos) as LONG;
            } else {
                partrect.left = (r.left as ::core::ffi::c_uint).wrapping_add(upsize) as LONG;
                partrect.right =
                    (partrect.left as ::core::ffi::c_uint).wrapping_add(thumbpos) as LONG;
            }
        } else if part as ::core::ffi::c_uint == SCROLL_THUMB as ::core::ffi::c_uint {
            if vertical != 0 {
                partrect.top = (r.top as ::core::ffi::c_uint)
                    .wrapping_add(upsize)
                    .wrapping_add(thumbpos) as LONG;
                partrect.bottom =
                    (partrect.top as ::core::ffi::c_uint).wrapping_add(thumbsize) as LONG;
            } else {
                partrect.left = (r.left as ::core::ffi::c_uint)
                    .wrapping_add(upsize)
                    .wrapping_add(thumbpos) as LONG;
                partrect.right =
                    (partrect.left as ::core::ffi::c_uint).wrapping_add(thumbsize) as LONG;
            }
        } else if part as ::core::ffi::c_uint == SCROLL_BOTTOM_RECT as ::core::ffi::c_uint {
            if vertical != 0 {
                partrect.top = (r.top as ::core::ffi::c_uint)
                    .wrapping_add(upsize)
                    .wrapping_add(thumbpos)
                    .wrapping_add(thumbsize) as LONG;
                partrect.bottom = (r.bottom as ::core::ffi::c_uint).wrapping_sub(downsize) as LONG;
            } else {
                partrect.left = (r.left as ::core::ffi::c_uint)
                    .wrapping_add(upsize)
                    .wrapping_add(thumbpos)
                    .wrapping_add(thumbsize) as LONG;
                partrect.right = (r.right as ::core::ffi::c_uint).wrapping_sub(downsize) as LONG;
            }
        }
    }

    InvalidateRect(
        hwnd,
        core::ptr::addr_of_mut!(partrect) as LPCRECT,
        TRUE,
    );
}

unsafe extern "C" fn scroll_event(mut hwnd: HWND, mut theme: HTHEME, mut msg: UINT, mut pt: POINT) {
    let mut hittest: SCROLL_HITTEST;
    let mut tme: TRACKMOUSEEVENT = core::mem::zeroed();
    if (GetWindowLongW(hwnd, GWL_STYLE) as u32 & (SBS_SIZEGRIP as u32 | SBS_SIZEBOX as u32)) != 0 {
        return;
    }
    hittest = hit_test(hwnd, theme, pt);
    match msg {
        WM_MOUSEMOVE => {
            hittest = hit_test(hwnd, theme, pt);
            tracking_win = hwnd;
        }
        WM_MOUSELEAVE => {
            if tracking_win == hwnd {
                hittest = (SCROLL_NOWHERE as u32);
            }
        }
        _ => {}
    }
    tme.cbSize = core::mem::size_of::<TRACKMOUSEEVENT>() as DWORD;
    tme.dwFlags = TME_QUERY as DWORD;
    TrackMouseEvent(core::ptr::addr_of_mut!(tme));
    if (tme.dwFlags & (TME_LEAVE as DWORD)) == 0 || tme.hwndTrack != hwnd {
        tme.dwFlags = TME_LEAVE as DWORD;
        tme.hwndTrack = hwnd;
        TrackMouseEvent(core::ptr::addr_of_mut!(tme));
    }
    if tracking_win != hwnd && msg == WM_MOUSELEAVE as UINT {
        redraw_part(hwnd, theme, (SCROLL_NOWHERE) as SCROLL_HITTEST);
        return;
    }
    if tracking_win == hwnd && (hittest as u32) != (tracking_hot_part as u32) {
        let mut oldhotpart: SCROLL_HITTEST = tracking_hot_part;
        tracking_hot_part = hittest;
        if (hittest as u32) != (SCROLL_NOWHERE as u32) {
            redraw_part(hwnd, theme, hittest);
        } else {
            tracking_win = core::ptr::null_mut();
        }
        if (oldhotpart as u32) != (SCROLL_NOWHERE as u32) {
            redraw_part(hwnd, theme, oldhotpart);
        }
    }
}

unsafe extern "C" fn paint_scrollbar(hwnd: HWND, theme: HTHEME) {
    let mut dc: HDC = core::ptr::null_mut();
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    let mut r: RECT = core::mem::zeroed();
    let mut style: DWORD = GetWindowLongW(hwnd, GWL_STYLE) as DWORD;
    let mut vertical: BOOL = (style & SBS_VERT) as BOOL;
    let mut disabled: BOOL = (IsWindowEnabled(hwnd) == 0) as BOOL;

    GetWindowRect(hwnd, core::ptr::addr_of_mut!(r));
    OffsetRect(
        core::ptr::addr_of_mut!(r),
        -r.left,
        -r.top,
    );

    dc = BeginPaint(hwnd, core::ptr::addr_of_mut!(ps));

    if (style & SBS_SIZEBOX) != 0 || (style & SBS_SIZEGRIP) != 0 {
        let mut state: i32 = 0;

        if (style & SBS_SIZEBOXTOPLEFTALIGN) != 0 {
            state = (SZB_TOPLEFTALIGN) as i32;
        } else {
            state = (SZB_RIGHTALIGN) as i32;
        }

        DrawThemeBackground(
            theme,
            dc,
            (SBP_SIZEBOX) as i32,
            state,
            core::ptr::addr_of_mut!(r),
            core::ptr::null_mut(),
        );
    } else {
        let mut sbi: SCROLLBARINFO = core::mem::zeroed();
        let mut si: SCROLLINFO = core::mem::zeroed();
        let mut thumbpos: u32 = 0;
        let mut thumbsize: u32 = 0;
        let mut uppertrackstate: i32 = 0;
        let mut lowertrackstate: i32 = 0;
        let mut thumbstate: i32 = 0;
        let mut partrect: RECT = core::mem::zeroed();
        let mut trackrect: RECT = core::mem::zeroed();
        let mut grippersize: SIZE = core::mem::zeroed();

        sbi.cbSize = core::mem::size_of::<SCROLLBARINFO>() as DWORD;
        GetScrollBarInfo(hwnd, (OBJID_CLIENT) as LONG, core::ptr::addr_of_mut!(sbi));

        si.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
        si.fMask = SIF_ALL as UINT;
        GetScrollInfo(hwnd, (SB_CTL) as i32, core::ptr::addr_of_mut!(si));

        trackrect = r;

        if disabled != 0 {
            uppertrackstate = (SCRBS_DISABLED) as i32;
            lowertrackstate = (SCRBS_DISABLED) as i32;
            thumbstate = (SCRBS_DISABLED) as i32;
        } else {
            uppertrackstate = (SCRBS_NORMAL) as i32;
            lowertrackstate = (SCRBS_NORMAL) as i32;
            thumbstate = (SCRBS_NORMAL) as i32;

            if tracking_win == hwnd {
                if tracking_hot_part == (SCROLL_TOP_RECT as u32) {
                    uppertrackstate = (SCRBS_HOT) as i32;
                } else if tracking_hot_part == (SCROLL_BOTTOM_RECT as u32) {
                    lowertrackstate = (SCRBS_HOT) as i32;
                } else if tracking_hot_part == (SCROLL_THUMB as u32) {
                    thumbstate = (SCRBS_HOT) as i32;
                }
            }
        }

        if vertical != 0 {
            let mut upsize: SIZE = core::mem::zeroed();
            let mut downsize: SIZE = core::mem::zeroed();
            let mut uparrowstate: i32 = 0;
            let mut downarrowstate: i32 = 0;

            if disabled != 0 {
                uparrowstate = (ABS_UPDISABLED) as i32;
                downarrowstate = (ABS_DOWNDISABLED) as i32;
            } else {
                uparrowstate = (ABS_UPNORMAL) as i32;
                downarrowstate = (ABS_DOWNNORMAL) as i32;

                if tracking_win == hwnd {
                    if tracking_hot_part == (SCROLL_TOP_ARROW as u32) {
                        uparrowstate = (ABS_UPHOT) as i32;
                    } else if tracking_hot_part == (SCROLL_BOTTOM_ARROW as u32) {
                        downarrowstate = (ABS_DOWNHOT) as i32;
                    }
                }
            }

            if (GetThemePartSize(
                theme,
                dc,
                (SBP_ARROWBTN) as i32,
                uparrowstate as i32,
                core::ptr::null_mut(),
                TS_DRAW,
                core::ptr::addr_of_mut!(upsize),
            )) < 0 {
                return;
            }

            if (GetThemePartSize(
                theme,
                dc,
                (SBP_ARROWBTN) as i32,
                downarrowstate as i32,
                core::ptr::null_mut(),
                TS_DRAW,
                core::ptr::addr_of_mut!(downsize),
            )) < 0 {
                return;
            }

            if r.bottom - r.top - upsize.cy - downsize.cy < SCROLL_MIN_RECT {
                downsize.cy = (r.bottom - r.top - SCROLL_MIN_RECT) / 2;
                upsize.cy = downsize.cy;
            }

            partrect = r;
            partrect.bottom = partrect.top + upsize.cy;
            DrawThemeBackground(
                theme,
                dc,
                (SBP_ARROWBTN) as i32,
                uparrowstate,
                core::ptr::addr_of_mut!(partrect),
                core::ptr::null_mut(),
            );

            trackrect.top = partrect.bottom;

            partrect.bottom = r.bottom;
            partrect.top = partrect.bottom - downsize.cy;
            DrawThemeBackground(
                theme,
                dc,
                (SBP_ARROWBTN) as i32,
                downarrowstate,
                core::ptr::addr_of_mut!(partrect),
                core::ptr::null_mut(),
            );

            trackrect.bottom = partrect.top;

            calc_thumb_dimensions(
                (trackrect.bottom - trackrect.top) as u32,
                core::ptr::addr_of_mut!(si),
                core::ptr::addr_of_mut!(thumbpos),
                core::ptr::addr_of_mut!(thumbsize),
            );

            if thumbpos > 0 {
                partrect.top = trackrect.top;
                partrect.bottom = (partrect.top as u32).wrapping_add(thumbpos) as LONG;

                DrawThemeBackground(
                    theme,
                    dc,
                    (SBP_UPPERTRACKVERT) as i32,
                    uppertrackstate,
                    core::ptr::addr_of_mut!(partrect),
                    core::ptr::null_mut(),
                );
            }

            if thumbsize > 0 {
                partrect.top = (trackrect.top as u32).wrapping_add(thumbpos) as LONG;
                partrect.bottom = (partrect.top as u32).wrapping_add(thumbsize) as LONG;

                DrawThemeBackground(
                    theme,
                    dc,
                    (SBP_THUMBBTNVERT) as i32,
                    thumbstate,
                    core::ptr::addr_of_mut!(partrect),
                    core::ptr::null_mut(),
                );

                if (GetThemePartSize(
                    theme,
                    dc,
                    (SBP_GRIPPERVERT) as i32,
                    thumbstate as i32,
                    core::ptr::null_mut(),
                    TS_DRAW,
                    core::ptr::addr_of_mut!(grippersize),
                )) >= 0 {
                    let mut margins: MARGINS = core::mem::zeroed();

                    if (GetThemeMargins(
                        theme,
                        dc,
                        (SBP_THUMBBTNVERT) as i32,
                        thumbstate,
                        (TMT_CONTENTMARGINS as i32),
                        core::ptr::addr_of_mut!(partrect),
                        core::ptr::addr_of_mut!(margins),
                    )) >= 0 {
                        if (grippersize.cy as u32)
                            <= thumbsize
                                .wrapping_sub(margins.cyTopHeight as u32)
                                .wrapping_sub(margins.cyBottomHeight as u32)
                        {
                            DrawThemeBackground(
                                theme,
                                dc,
                                (SBP_GRIPPERVERT) as i32,
                                thumbstate,
                                core::ptr::addr_of_mut!(partrect),
                                core::ptr::null_mut(),
                            );
                        }
                    }
                }
            }

            if thumbpos.wrapping_add(thumbsize) < (trackrect.bottom - trackrect.top) as u32 {
                partrect.bottom = trackrect.bottom;
                partrect.top = (trackrect.top as u32)
                    .wrapping_add(thumbsize)
                    .wrapping_add(thumbpos) as LONG;

                DrawThemeBackground(
                    theme,
                    dc,
                    (SBP_LOWERTRACKVERT) as i32,
                    lowertrackstate,
                    core::ptr::addr_of_mut!(partrect),
                    core::ptr::null_mut(),
                );
            }
        } else {
            let mut leftsize: SIZE = core::mem::zeroed();
            let mut rightsize: SIZE = core::mem::zeroed();
            let mut leftarrowstate: i32 = 0;
            let mut rightarrowstate: i32 = 0;

            if disabled != 0 {
                leftarrowstate = (ABS_LEFTDISABLED) as i32;
                rightarrowstate = (ABS_RIGHTDISABLED) as i32;
            } else {
                leftarrowstate = (ABS_LEFTNORMAL) as i32;
                rightarrowstate = (ABS_RIGHTNORMAL) as i32;

                if tracking_win == hwnd {
                    if tracking_hot_part == (SCROLL_TOP_ARROW as u32) {
                        leftarrowstate = (ABS_LEFTHOT) as i32;
                    } else if tracking_hot_part == (SCROLL_BOTTOM_ARROW as u32) {
                        rightarrowstate = (ABS_RIGHTHOT) as i32;
                    }
                }
            }

            if (GetThemePartSize(
                theme,
                dc,
                (SBP_ARROWBTN) as i32,
                leftarrowstate as i32,
                core::ptr::null_mut(),
                TS_DRAW,
                core::ptr::addr_of_mut!(leftsize),
            )) < 0 {
                return;
            }

            if (GetThemePartSize(
                theme,
                dc,
                (SBP_ARROWBTN) as i32,
                rightarrowstate as i32,
                core::ptr::null_mut(),
                TS_DRAW,
                core::ptr::addr_of_mut!(rightsize),
            )) < 0 {
                return;
            }

            if r.right - r.left - leftsize.cx - rightsize.cx < SCROLL_MIN_RECT {
                rightsize.cx = (r.right - r.left - SCROLL_MIN_RECT) / 2;
                leftsize.cx = rightsize.cx;
            }

            partrect = r;
            partrect.right = partrect.left + leftsize.cx;
            DrawThemeBackground(
                theme,
                dc,
                (SBP_ARROWBTN) as i32,
                leftarrowstate,
                core::ptr::addr_of_mut!(partrect),
                core::ptr::null_mut(),
            );

            trackrect.left = partrect.right;

            partrect.right = r.right;
            partrect.left = partrect.right - rightsize.cx;
            DrawThemeBackground(
                theme,
                dc,
                (SBP_ARROWBTN) as i32,
                rightarrowstate,
                core::ptr::addr_of_mut!(partrect),
                core::ptr::null_mut(),
            );

            trackrect.right = partrect.left;

            calc_thumb_dimensions(
                (trackrect.right - trackrect.left) as u32,
                core::ptr::addr_of_mut!(si),
                core::ptr::addr_of_mut!(thumbpos),
                core::ptr::addr_of_mut!(thumbsize),
            );

            if thumbpos > 0 {
                partrect.left = trackrect.left;
                partrect.right = (partrect.left as u32).wrapping_add(thumbpos) as LONG;

                DrawThemeBackground(
                    theme,
                    dc,
                    (SBP_UPPERTRACKHORZ) as i32,
                    uppertrackstate,
                    core::ptr::addr_of_mut!(partrect),
                    core::ptr::null_mut(),
                );
            }

            if thumbsize > 0 {
                partrect.left = (trackrect.left as u32).wrapping_add(thumbpos) as LONG;
                partrect.right = (partrect.left as u32).wrapping_add(thumbsize) as LONG;

                DrawThemeBackground(
                    theme,
                    dc,
                    (SBP_THUMBBTNHORZ) as i32,
                    thumbstate,
                    core::ptr::addr_of_mut!(partrect),
                    core::ptr::null_mut(),
                );

                if (GetThemePartSize(
                    theme,
                    dc,
                    (SBP_GRIPPERHORZ) as i32,
                    thumbstate as i32,
                    core::ptr::null_mut(),
                    TS_DRAW,
                    core::ptr::addr_of_mut!(grippersize),
                )) >= 0 {
                    let mut margins: MARGINS = core::mem::zeroed();

                    if (GetThemeMargins(
                        theme,
                        dc,
                        (SBP_THUMBBTNHORZ) as i32,
                        thumbstate,
                        (TMT_CONTENTMARGINS as i32),
                        core::ptr::addr_of_mut!(partrect),
                        core::ptr::addr_of_mut!(margins),
                    )) >= 0 {
                        if (grippersize.cx as u32)
                            <= thumbsize
                                .wrapping_sub(margins.cxLeftWidth as u32)
                                .wrapping_sub(margins.cxRightWidth as u32)
                        {
                            DrawThemeBackground(
                                theme,
                                dc,
                                (SBP_GRIPPERHORZ) as i32,
                                thumbstate,
                                core::ptr::addr_of_mut!(partrect),
                                core::ptr::null_mut(),
                            );
                        }
                    }
                }
            }

            if thumbpos.wrapping_add(thumbsize) < (trackrect.right - trackrect.left) as u32 {
                partrect.right = trackrect.right;
                partrect.left = (trackrect.left as u32)
                    .wrapping_add(thumbsize)
                    .wrapping_add(thumbpos) as LONG;

                DrawThemeBackground(
                    theme,
                    dc,
                    (SBP_LOWERTRACKHORZ) as i32,
                    lowertrackstate,
                    core::ptr::addr_of_mut!(partrect),
                    core::ptr::null_mut(),
                );
            }
        }
    }

    EndPaint(hwnd, core::ptr::addr_of_mut!(ps));
}
#[no_mangle]
pub unsafe extern "system" fn THEMING_ScrollbarSubclassProc(
    hwnd: HWND,
    msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    dwRefData: ULONG_PTR,
) -> LRESULT {
    let themeClass: *const WCHAR = (WC_SCROLLBARW).as_ptr();
    let mut theme: HTHEME = core::ptr::null_mut();
    let mut result: LRESULT = 0;
    let mut pt: POINT = POINT { x: 0, y: 0 };

    match msg {
        WM_CREATE => {
            result = THEMING_CallOriginalClass(hwnd, msg, wParam, lParam);
            OpenThemeData(hwnd, themeClass as LPCWSTR);
            return result;
        }
        WM_DESTROY => {
            theme = GetWindowTheme(hwnd);
            CloseThemeData(theme);
            return THEMING_CallOriginalClass(hwnd, msg, wParam, lParam);
        }
        WM_THEMECHANGED => {
            theme = GetWindowTheme(hwnd);
            CloseThemeData(theme);
            OpenThemeData(hwnd, themeClass as LPCWSTR);
        }
        WM_SYSCOLORCHANGE => {
            theme = GetWindowTheme(hwnd);
            if theme.is_null() {
                return THEMING_CallOriginalClass(hwnd, msg, wParam, lParam);
            }
            /* Do nothing. When themed, a WM_THEMECHANGED will be received, too,
             * which will do the repaint. */
        }
        WM_PAINT => {
            theme = GetWindowTheme(hwnd);
            if theme.is_null() {
                return THEMING_CallOriginalClass(hwnd, msg, wParam, lParam);
            }

            paint_scrollbar(hwnd, theme);
        }
        WM_MOUSEMOVE | WM_MOUSELEAVE => {
            theme = GetWindowTheme(hwnd);
            if theme.is_null() {
                return THEMING_CallOriginalClass(hwnd, msg, wParam, lParam);
            }

            pt.x = LOWORD!(lParam as DWORD) as i16 as LONG;
            pt.y = HIWORD!(lParam as DWORD) as i16 as LONG;
            scroll_event(hwnd, theme, msg, pt);
        }
        _ => {
            return THEMING_CallOriginalClass(hwnd, msg, wParam, lParam);
        }
    }

    return 0;
}
