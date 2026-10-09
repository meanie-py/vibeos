use crate::comctl32::comctl32undoc::Alloc;
use crate::comctl32::commctrl::COMCTL32_GetFontMetrics;
use crate::comctl32::commctrl::COMCTL32_IsReflectedMessage;
use crate::comctl32::commctrl::COMCTL32_RefreshSysColors;
use crate::comctl32::commctrl::DrawStatusTextW;
use crate::comctl32::comctl32undoc::Free;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_statusbar);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const TOOLTIPS_CLASSW: [::core::ffi::c_ushort; 17] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_ushort; 17]>(
        *b"t\0o\0o\0l\0t\0i\0p\0s\0_\0c\0l\0a\0s\0s\x003\x002\0\0\0",
    )
};

pub const STATUSCLASSNAMEW: [::core::ffi::c_ushort; 19] = unsafe {
    ::core::mem::transmute::<[u8; 38], [::core::ffi::c_ushort; 19]>(
        *b"m\0s\0c\0t\0l\0s\0_\0s\0t\0a\0t\0u\0s\0b\0a\0r\x003\x002\0\0\0",
    )
};

static mut themeClass: [WCHAR; 7] = [
    'S' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

// WINE_DEFAULT_DEBUG_CHANNEL(statusbar): debug-channel macro, not emitted.

#[repr(C)]
pub struct STATUSWINDOWPART {
    pub x: INT,
    pub style: INT,
    pub bound: RECT,
    pub text: LPWSTR,
    pub hIcon: HICON,
}

#[repr(C)]
pub struct STATUS_INFO {
    pub Self_: HWND,
    pub Notify: HWND,
    pub numParts: WORD,
    pub height: UINT,
    pub minHeight: UINT,
    pub simple: BOOL,
    pub hwndToolTip: HWND,
    pub hFont: HFONT,
    pub hDefaultFont: HFONT,
    pub clrBk: COLORREF,
    pub bUnicode: BOOL,
    pub part0: STATUSWINDOWPART,
    pub parts: *mut STATUSWINDOWPART,
    pub horizontalBorder: INT,
    pub verticalBorder: INT,
    pub horizontalGap: INT,
}

pub const HORZ_BORDER: i32 = 0;
pub const VERT_BORDER: i32 = 2;
pub const HORZ_GAP: i32 = 2;

// Forward prototypes STATUSBAR_SetPartBounds and STATUSBAR_NotifyFormat are
// not emitted here; they are supplied by the function translations.

unsafe extern "C" fn debugstr_t(text: LPCWSTR, isW: BOOL) -> LPCSTR {
    return if isW != 0 {
        debugstr_w(text as *const WCHAR) as LPCSTR
    } else {
        debugstr_a(text as LPCSTR) as LPCSTR
    };
}

unsafe extern "C" fn STATUSBAR_ComputeHeight(infoPtr: *mut STATUS_INFO) -> UINT {
    let mut theme: HTHEME = core::ptr::null_mut();
    let mut height: UINT = 0;
    let mut tm: TEXTMETRICW = core::mem::zeroed();
    let mut margin: i32;

    COMCTL32_GetFontMetrics(
        if !(*infoPtr).hFont.is_null() {
            (*infoPtr).hFont
        } else {
            (*infoPtr).hDefaultFont
        },
        core::ptr::addr_of_mut!(tm),
    );
    margin = if tm.tmInternalLeading != 0 {
        tm.tmInternalLeading as i32
    } else {
        2
    };
    height = max(
        (tm.tmHeight as i32 + margin + 2 * GetSystemMetrics((SM_CYBORDER) as i32))
            as UINT,
        (*infoPtr).minHeight,
    )
    .wrapping_add((*infoPtr).verticalBorder as UINT);

    theme = GetWindowTheme((*infoPtr).Self_);
    if !theme.is_null() {
        /* Determine bar height from theme such that the content area is
         * textHeight pixels large */
        let hdc: HDC = GetDC((*infoPtr).Self_);
        let mut r: RECT = core::mem::zeroed();

        SetRect(
            core::ptr::addr_of_mut!(r),
            0,
            0,
            0,
            max((*infoPtr).minHeight, tm.tmHeight as UINT) as i32,
        );
        if (GetThemeBackgroundExtent(
            theme,
            hdc,
            SP_PANE as i32,
            0,
            core::ptr::addr_of_mut!(r),
            core::ptr::addr_of_mut!(r),
        )) >= 0
        {
            height = (r.bottom - r.top) as UINT;
        }
        ReleaseDC((*infoPtr).Self_, hdc);
    }

    return height;
}

unsafe extern "C" fn STATUSBAR_DrawSizeGrip(mut theme: HTHEME, mut hdc: HDC, mut lpRect: LPRECT) {
    let mut rc: RECT = *lpRect;
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/status.c\0"
                as *const u8 as *const i8,
            b"STATUSBAR_DrawSizeGrip\0" as *const u8 as *const i8,
            138 as i32,
            b"draw size grip %s\n\0" as *const u8 as *const i8,
            wine_dbgstr_rect(lpRect as *const RECT),
        );
    }
    if !theme.is_null() {
        let mut gripperSize: SIZE = tagSIZE { cx: 0, cy: 0 };
        if (GetThemePartSize(
            theme,
            hdc,
            SP_GRIPPER as i32,
            0,
            lpRect,
            TS_DRAW,
            core::ptr::addr_of_mut!(gripperSize),
        )) >= 0
        {
            rc.left = rc.right - gripperSize.cx;
            rc.top = rc.bottom - gripperSize.cy;
            if (DrawThemeBackground(
                theme,
                hdc,
                SP_GRIPPER as i32,
                0,
                core::ptr::addr_of_mut!(rc),
                core::ptr::null_mut(),
            )) >= 0
            {
                return;
            }
        }
    }
    rc.left = max(
        rc.left,
        rc.right - GetSystemMetrics((SM_CXVSCROLL) as i32) - 1,
    );
    rc.top = max(
        rc.top,
        rc.bottom - GetSystemMetrics((SM_CYHSCROLL) as i32) - 1,
    );
    DrawFrameControl(
        hdc,
        core::ptr::addr_of_mut!(rc),
        DFC_SCROLL,
        DFCS_SCROLLSIZEGRIP,
    );
}

unsafe extern "C" fn STATUSBAR_DrawPart(
    infoPtr: *const STATUS_INFO,
    hdc: HDC,
    part: *const STATUSWINDOWPART,
    itemID: INT,
) {
    let mut r: RECT = (*part).bound;
    let mut border: UINT = BDR_SUNKENOUTER;
    let mut theme: HTHEME = GetWindowTheme((*infoPtr).Self_);
    let mut themePart: INT = (SP_PANE) as i32;
    let mut x: INT = 0;

    if ((*part).style as INT & (SBT_POPOUT as i32)) != 0 {
        border = BDR_RAISEDOUTER;
    } else if ((*part).style as INT & (SBT_NOBORDERS as i32)) != 0 {
        border = 0;
    }

    if !theme.is_null() {
        if (GetWindowLongW((*infoPtr).Self_, GWL_STYLE) as INT & (SBARS_SIZEGRIP as i32)) != 0
            && ((*infoPtr).simple != 0
                || itemID == (*infoPtr).numParts as INT - 1)
        {
            themePart = (SP_GRIPPERPANE) as i32;
        }
        DrawThemeBackground(
            theme,
            hdc,
            themePart,
            0,
            core::ptr::addr_of_mut!(r),
            core::ptr::null_mut(),
        );
    } else {
        DrawEdge(
            hdc,
            core::ptr::addr_of_mut!(r),
            border,
            (BF_RECT | BF_ADJUST) as UINT,
        );
    }

    if !(*part).hIcon.is_null() {
        let mut cy: INT = r.bottom - r.top;
        DrawIconEx(
            hdc,
            r.left + 2,
            r.top,
            (*part).hIcon,
            cy,
            cy,
            0,
            core::ptr::null_mut(),
            DI_NORMAL,
        );
        x = 2 + cy;
    }

    if (*part).style as INT & (SBT_OWNERDRAW as i32) != 0 {
        let mut dis: DRAWITEMSTRUCT = core::mem::zeroed();

        dis.CtlID = GetWindowLongPtrW((*infoPtr).Self_, GWLP_ID) as UINT;
        dis.itemID = itemID as UINT;
        dis.hwndItem = (*infoPtr).Self_;
        dis.hDC = hdc;
        dis.rcItem = r;
        dis.itemData = (*part).text as ULONG_PTR;
        SendMessageW(
            (*infoPtr).Notify,
            WM_DRAWITEM,
            dis.CtlID as WPARAM,
            core::ptr::addr_of_mut!(dis) as LPARAM,
        );
    } else {
        r.left += x;
        if theme.is_null() {
            r.left -= 2;
            DrawStatusTextW(
                hdc,
                core::ptr::addr_of_mut!(r) as LPCRECT,
                (*part).text as LPCWSTR,
                SBT_NOBORDERS as UINT,
            );
        } else {
            r.left += 2;
            r.right -= 2;
            DrawThemeText(
                theme,
                hdc,
                (SP_PANE) as i32,
                0,
                (*part).text as LPCWSTR,
                -1,
                (DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX) as DWORD,
                0 as DWORD,
                core::ptr::addr_of_mut!(r),
            );
        }
    }
}

unsafe extern "C" fn STATUSBAR_RefreshPart(infoPtr: *const STATUS_INFO, hdc: HDC, part: *const STATUSWINDOWPART, itemID: i32) {
    let mut hbrBk: HBRUSH = core::ptr::null_mut();
    let mut theme: HTHEME = core::ptr::null_mut();

    if (*part).bound.right < (*part).bound.left {
        return;
    }

    if RectVisible(hdc, core::ptr::addr_of!((*part).bound)) == 0 {
        return;
    }

    theme = GetWindowTheme((*infoPtr).Self_);
    if !theme.is_null() {
        let mut cr: RECT = core::mem::zeroed();
        GetClientRect((*infoPtr).Self_, core::ptr::addr_of_mut!(cr));
        DrawThemeBackground(
            theme,
            hdc,
            0,
            0,
            core::ptr::addr_of_mut!(cr),
            core::ptr::addr_of!((*part).bound),
        );
    } else {
        if (*infoPtr).clrBk != CLR_DEFAULT {
            hbrBk = CreateSolidBrush((*infoPtr).clrBk);
        } else {
            hbrBk = GetSysColorBrush((COLOR_3DFACE) as i32);
        }
        FillRect(hdc, core::ptr::addr_of!((*part).bound), hbrBk);
        if (*infoPtr).clrBk != CLR_DEFAULT {
            DeleteObject((hbrBk).cast());
        }
    }

    STATUSBAR_DrawPart(infoPtr, hdc, part, itemID);
}

unsafe extern "C" fn STATUSBAR_Refresh(mut infoPtr: *mut STATUS_INFO, mut hdc: HDC) -> LRESULT {
    let mut rect: RECT = core::mem::zeroed();
    let mut hbrBk: HBRUSH = core::ptr::null_mut();
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut theme: HTHEME = core::ptr::null_mut();

    if IsWindowVisible((*infoPtr).Self_) == 0 {
        return 0 as LRESULT;
    }

    STATUSBAR_SetPartBounds(infoPtr);

    GetClientRect((*infoPtr).Self_, core::ptr::addr_of_mut!(rect));

    theme = GetWindowTheme((*infoPtr).Self_);
    if !theme.is_null() {
        DrawThemeBackground(
            theme,
            hdc,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            core::ptr::addr_of_mut!(rect),
            core::ptr::null::<RECT>(),
        );
    } else {
        if (*infoPtr).clrBk != CLR_DEFAULT as COLORREF {
            hbrBk = CreateSolidBrush((*infoPtr).clrBk);
        } else {
            hbrBk = GetSysColorBrush(COLOR_3DFACE as ::core::ffi::c_int);
        }
        FillRect(hdc, core::ptr::addr_of_mut!(rect) as LPCRECT, hbrBk);
        if (*infoPtr).clrBk != CLR_DEFAULT as COLORREF {
            DeleteObject(hbrBk as HGDIOBJ);
        }
    }

    hOldFont = SelectObject(
        hdc,
        (if !(*infoPtr).hFont.is_null() {
            (*infoPtr).hFont
        } else {
            (*infoPtr).hDefaultFont
        }) as HGDIOBJ,
    ) as HFONT;

    if (*infoPtr).simple != 0 {
        STATUSBAR_RefreshPart(
            infoPtr,
            hdc,
            core::ptr::addr_of_mut!((*infoPtr).part0),
            0 as ::core::ffi::c_int,
        );
    } else {
        let mut i: ::core::ffi::c_uint = 0;
        i = 0 as ::core::ffi::c_uint;
        while i < (*infoPtr).numParts as ::core::ffi::c_uint {
            STATUSBAR_RefreshPart(
                infoPtr,
                hdc,
                (*infoPtr).parts.offset(i as isize),
                i as ::core::ffi::c_int,
            );
            i = i.wrapping_add(1);
        }
    }

    SelectObject(hdc, hOldFont as HGDIOBJ);

    if (GetWindowLongW((*infoPtr).Self_, GWL_STYLE as ::core::ffi::c_int)
        & SBARS_SIZEGRIP as ::core::ffi::c_int)
        != 0
        && (GetWindowLongW((*infoPtr).Notify, GWL_STYLE as ::core::ffi::c_int)
            & WS_MAXIMIZE as ::core::ffi::c_int)
            == 0
    {
        STATUSBAR_DrawSizeGrip(theme, hdc, core::ptr::addr_of_mut!(rect));
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn STATUSBAR_InternalHitTest(infoPtr: *const STATUS_INFO, pt: *const POINT) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_uint;

    if (*infoPtr).simple != 0 {
        return 255;
    }

    i = 0;
    while i < (*infoPtr).numParts as ::core::ffi::c_uint {
        if (*pt).x >= (*(*infoPtr).parts.offset(i as isize)).bound.left
            && (*pt).x <= (*(*infoPtr).parts.offset(i as isize)).bound.right
        {
            return i as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    return -2;
}

unsafe extern "C" fn STATUSBAR_SetPartBounds(mut infoPtr: *mut STATUS_INFO) {
    let mut part: *mut STATUSWINDOWPART = core::ptr::null_mut::<STATUSWINDOWPART>();
    let mut rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut r: *mut RECT = core::ptr::null_mut::<RECT>();
    let mut i: UINT = 0;

    /* get our window size */
    GetClientRect((*infoPtr).Self_, &raw mut rect);
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/status.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"STATUSBAR_SetPartBounds\0" as *const u8 as *const core::ffi::c_char,
            332 as core::ffi::c_int,
            b"client wnd size is %s\n\0" as *const u8 as *const core::ffi::c_char,
            wine_dbgstr_rect(&raw mut rect),
        );
    }

    rect.left += (*infoPtr).horizontalBorder as core::ffi::c_int;
    rect.top += (*infoPtr).verticalBorder as core::ffi::c_int;

    /* set bounds for simple rectangle */
    (*infoPtr).part0.bound = rect;

    /* set bounds for non-simple rectangles */
    i = 0 as UINT;
    while i < (*infoPtr).numParts as UINT {
        part = (*infoPtr).parts.offset(i as isize) as *mut STATUSWINDOWPART;
        r = &raw mut (*(*infoPtr).parts.offset(i as isize)).bound;
        (*r).top = rect.top;
        (*r).bottom = rect.bottom;
        if i == 0 as UINT {
            (*r).left = 0 as LONG;
        } else {
            (*r).left = ((*(*infoPtr)
                .parts
                .offset(i.wrapping_sub(1 as UINT) as isize))
                .bound
                .right as INT
                + (*infoPtr).horizontalGap) as LONG;
        }
        if (*part).x == -(1) {
            (*r).right = rect.right;
        } else {
            (*r).right = (*part).x as LONG;
        }

        if !(*infoPtr).hwndToolTip.is_null() {
            let mut ti: TTTOOLINFOW = core::mem::zeroed();

            ti.cbSize = core::mem::size_of::<TTTOOLINFOW>() as UINT;
            ti.hwnd = (*infoPtr).Self_;
            ti.uId = i as UINT_PTR;
            ti.rect = *r;
            SendMessageW(
                (*infoPtr).hwndToolTip,
                TTM_NEWTOOLRECTW as UINT,
                0 as WPARAM,
                &raw mut ti as LPARAM,
            );
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn STATUSBAR_Relay2Tip(
    infoPtr: *const STATUS_INFO,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut msg: MSG = core::mem::zeroed();

    msg.hwnd = (*infoPtr).Self_;
    msg.message = uMsg;
    msg.wParam = wParam;
    msg.lParam = lParam;
    msg.time = GetMessageTime() as DWORD;
    msg.pt.x = (LOWORD!(GetMessagePos()) as i16) as LONG;
    msg.pt.y = (HIWORD!(GetMessagePos()) as i16) as LONG;

    return SendMessageW(
        (*infoPtr).hwndToolTip,
        TTM_RELAYEVENT as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(msg) as LPARAM,
    );
}

unsafe extern "C" fn STATUSBAR_GetBorders(infoPtr: *const STATUS_INFO, out: *mut INT) -> BOOL {
    if ((*__wine_dbch___default).flags as i32) & ((1i32) << (__WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/status.c\0".as_ptr() as *const i8,
            b"STATUSBAR_GetBorders\0".as_ptr() as *const i8,
            390i32,
            b"\n\0".as_ptr() as *const i8,
        );
    }
    *out.offset(0 as isize) = (*infoPtr).horizontalBorder;
    *out.offset(1 as isize) = (*infoPtr).verticalBorder;
    *out.offset(2 as isize) = (*infoPtr).horizontalGap;

    return TRUE;
}

unsafe extern "C" fn STATUSBAR_SetBorders(infoPtr: *mut STATUS_INFO, in_: *const INT) -> BOOL {
    (*infoPtr).horizontalBorder = *in_.offset(0 as isize);
    (*infoPtr).verticalBorder = *in_.offset(1 as isize);
    (*infoPtr).horizontalGap = *in_.offset(2 as isize);
    InvalidateRect((*infoPtr).Self_, core::ptr::null_mut(), FALSE);

    return TRUE;
}

unsafe extern "C" fn STATUSBAR_GetIcon(infoPtr: *const STATUS_INFO, nPart: INT) -> HICON {
    /* MSDN says: "simple parts are indexed with -1" */
    if (nPart < -1) || (nPart >= (*infoPtr).numParts as INT) {
        return core::ptr::null_mut();
    }

    if nPart == -1 {
        return (*infoPtr).part0.hIcon;
    } else {
        return (*(*infoPtr).parts.offset(nPart as isize)).hIcon;
    }
}

unsafe extern "C" fn STATUSBAR_GetParts(infoPtr: *const STATUS_INFO, mut num_parts: INT, parts: *mut INT) -> INT {
    let mut i: INT;

    if !parts.is_null() {
        if num_parts > (*infoPtr).numParts as INT {
            num_parts = (*infoPtr).numParts as INT;
        }
        i = 0;
        while i < num_parts {
            *parts.offset(i as isize) = (*(*infoPtr).parts.offset(i as isize)).x;
            i += 1;
        }
    }
    return (*infoPtr).numParts as INT;
}

unsafe extern "C" fn STATUSBAR_GetRect(infoPtr: *const STATUS_INFO, nPart: INT, rect: LPRECT) -> BOOL {
    if nPart >= (*infoPtr).numParts as INT || nPart < 0 {
        return FALSE;
    }
    if (*infoPtr).simple != 0 {
        *rect = (*infoPtr).part0.bound;
    } else {
        *rect = (*(*infoPtr).parts.offset(nPart as isize)).bound;
    }
    return TRUE;
}

unsafe extern "C" fn STATUSBAR_GetTextA(
    mut infoPtr: *mut STATUS_INFO,
    mut nPart: INT,
    mut buf: LPSTR,
) -> LRESULT {
    let mut part: *mut STATUSWINDOWPART = ::core::ptr::null_mut::<STATUSWINDOWPART>();
    let mut result: LRESULT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/status.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"STATUSBAR_GetTextA\0" as *const u8 as *const ::core::ffi::c_char,
            466 as ::core::ffi::c_int,
            b"part %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            nPart,
        );
    }
    if nPart < 0 || nPart >= (*infoPtr).numParts as ::core::ffi::c_int {
        return 0 as LRESULT;
    }
    if (*infoPtr).simple != 0 {
        part = &raw mut (*infoPtr).part0;
    } else {
        part = (*infoPtr).parts.offset(nPart as isize) as *mut STATUSWINDOWPART;
    }
    if (*part).style as ::core::ffi::c_int & (SBT_OWNERDRAW as i32) != 0 {
        result = (*part).text as LRESULT;
    } else {
        let mut len: DWORD = (if !(*part).text.is_null() {
            WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                (*part).text as LPCWSTR,
                -(1 as ::core::ffi::c_int),
                ::core::ptr::null_mut::<CHAR>(),
                0 as ::core::ffi::c_int,
                ::core::ptr::null::<CHAR>(),
                ::core::ptr::null_mut::<BOOL>(),
            ) - 1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as DWORD;
        result = MAKELONG!(len as WORD, (*part).style) as LONG as LRESULT;
        if !(*part).text.is_null() && !buf.is_null() {
            WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                (*part).text as LPCWSTR,
                -(1 as ::core::ffi::c_int),
                buf,
                len.wrapping_add(1 as DWORD) as ::core::ffi::c_int,
                ::core::ptr::null::<CHAR>(),
                ::core::ptr::null_mut::<BOOL>(),
            );
        }
    }
    return result;
}

unsafe extern "C" fn STATUSBAR_GetTextW(
    infoPtr: *mut STATUS_INFO,
    nPart: INT,
    buf: LPWSTR,
) -> LRESULT {
    let mut part: *mut STATUSWINDOWPART = core::ptr::null_mut();
    let mut result: LRESULT;

    if nPart < 0 || nPart >= (*infoPtr).numParts as INT {
        return 0 as LRESULT;
    }

    if (*infoPtr).simple != 0 {
        part = &raw mut (*infoPtr).part0;
    } else {
        part = (*infoPtr).parts.offset(nPart as isize) as *mut STATUSWINDOWPART;
    }

    if ((*part).style & (SBT_OWNERDRAW as i32)) != 0 {
        result = (*part).text as LRESULT;
    } else {
        result = (if !(*part).text.is_null() {
            lstrlenW((*part).text as LPCWSTR)
        } else {
            0 as INT
        }) as LRESULT;
        result |= ((*part).style << 16) as LRESULT;
        if !(*part).text.is_null() && !buf.is_null() {
            lstrcpyW(buf, (*part).text as LPCWSTR);
        }
    }
    return result;
}

unsafe extern "C" fn STATUSBAR_GetTextLength(infoPtr: *mut STATUS_INFO, nPart: INT) -> LRESULT {
    let mut part: *mut STATUSWINDOWPART;
    let mut result: DWORD;

    /* MSDN says: "simple parts use index of 0", so this check is ok. */
    if nPart < 0 || nPart >= (*infoPtr).numParts as INT {
        return 0;
    }

    if (*infoPtr).simple != 0 {
        part = core::ptr::addr_of_mut!((*infoPtr).part0);
    } else {
        part = (*infoPtr).parts.offset(nPart as isize);
    }

    if ((!(*part).style) & (SBT_OWNERDRAW as i32)) != 0 && !(*part).text.is_null() {
        result = lstrlenW((*part).text as LPCWSTR) as DWORD;
    } else {
        result = 0;
    }

    result |= ((*part).style << 16) as DWORD;
    return result as LRESULT;
}

unsafe extern "C" fn STATUSBAR_GetTipTextA(
    infoPtr: *const STATUS_INFO,
    id: INT,
    tip: LPSTR,
    size: INT,
) -> LRESULT {
    if !tip.is_null() {
        let mut buf: [CHAR; (INFOTIPSIZE) as _] = [0; (INFOTIPSIZE) as _];
        buf[0 as usize] = 0 as CHAR;

        if !(*infoPtr).hwndToolTip.is_null() {
            let mut ti: TTTOOLINFOA = core::mem::zeroed();
            ti.cbSize = core::mem::size_of::<TTTOOLINFOA>() as UINT;
            ti.hwnd = (*infoPtr).Self_;
            ti.uId = id as UINT_PTR;
            ti.lpszText = core::ptr::addr_of_mut!(buf) as *mut CHAR as LPSTR;
            SendMessageA(
                (*infoPtr).hwndToolTip,
                TTM_GETTEXTA as UINT,
                0 as WPARAM,
                core::ptr::addr_of_mut!(ti) as LPARAM,
            );
        }
        lstrcpynA(
            tip,
            core::ptr::addr_of_mut!(buf) as *mut CHAR as LPCSTR,
            size as core::ffi::c_int,
        );
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn STATUSBAR_GetTipTextW(infoPtr: *const STATUS_INFO, id: INT, tip: LPWSTR, size: INT) -> LRESULT {
    if !tip.is_null() {
        let mut buf: [WCHAR; (INFOTIPSIZE) as _] = [0; (INFOTIPSIZE) as _];
        buf[0 as usize] = 0 as WCHAR;

        if !(*infoPtr).hwndToolTip.is_null() {
            let mut ti: TTTOOLINFOW = core::mem::zeroed();
            ti.cbSize = core::mem::size_of::<TTTOOLINFOW>() as UINT;
            ti.hwnd = (*infoPtr).Self_;
            ti.uId = id as UINT_PTR;
            ti.lpszText = &raw mut buf as *mut WCHAR as LPWSTR;
            SendMessageW(
                (*infoPtr).hwndToolTip,
                TTM_GETTEXTW as UINT,
                0 as WPARAM,
                &raw mut ti as LPARAM,
            );
        }
        lstrcpynW(
            tip,
            &raw mut buf as *mut WCHAR as LPCWSTR,
            size as ::core::ffi::c_int,
        );
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn STATUSBAR_SetBkColor(infoPtr: *mut STATUS_INFO, color: COLORREF) -> COLORREF {
    let mut oldBkColor: COLORREF;

    oldBkColor = (*infoPtr).clrBk;
    (*infoPtr).clrBk = color;
    InvalidateRect((*infoPtr).Self_, core::ptr::null_mut(), FALSE);

    return oldBkColor;
}

unsafe extern "C" fn STATUSBAR_SetIcon(infoPtr: *mut STATUS_INFO, nPart: INT, hIcon: HICON) -> BOOL {
    if (nPart < -1) || (nPart >= (*infoPtr).numParts as INT) {
        return FALSE;
    }

    /* FIXME: MSDN says "if nPart is -1, the status bar is assumed simple" */
    if nPart == -1 {
        if (*infoPtr).part0.hIcon == hIcon {
            /* same as - no redraw */
            return TRUE;
        }
        (*infoPtr).part0.hIcon = hIcon;
        if (*infoPtr).simple != 0 {
            InvalidateRect(
                (*infoPtr).Self_,
                core::ptr::addr_of_mut!((*infoPtr).part0.bound),
                FALSE,
            );
        }
    } else {
        if (*(*infoPtr).parts.offset(nPart as isize)).hIcon == hIcon {
            /* same as - no redraw */
            return TRUE;
        }

        (*(*infoPtr).parts.offset(nPart as isize)).hIcon = hIcon;
        if !((*infoPtr).simple != 0) {
            InvalidateRect(
                (*infoPtr).Self_,
                core::ptr::addr_of_mut!((*(*infoPtr).parts.offset(nPart as isize)).bound),
                FALSE,
            );
        }
    }
    return TRUE;
}

unsafe extern "C" fn STATUSBAR_SetMinHeight(infoPtr: *mut STATUS_INFO, height: INT) -> BOOL {
    let mut ysize: DWORD = GetSystemMetrics((SM_CYSIZE) as i32) as DWORD;
    if (ysize & (1 as DWORD)) != 0 {
        ysize = ysize.wrapping_sub(1);
    }
    (*infoPtr).minHeight = max(height as DWORD, ysize) as UINT;
    (*infoPtr).height = STATUSBAR_ComputeHeight(infoPtr);
    /* like native, don't resize the control */
    return TRUE;
}

unsafe extern "C" fn STATUSBAR_SetParts(
    infoPtr: *mut STATUS_INFO,
    count: INT,
    parts: LPINT,
) -> BOOL {
    let mut tmp: *mut STATUSWINDOWPART = core::ptr::null_mut();
    let mut i: INT = 0;
    let mut oldNumParts: INT = 0;

    if count == 0 {
        return FALSE;
    }

    oldNumParts = (*infoPtr).numParts as INT;
    (*infoPtr).numParts = count as WORD;
    if oldNumParts > (*infoPtr).numParts as INT {
        i = (*infoPtr).numParts as INT;
        while i < oldNumParts {
            if ((*(*infoPtr).parts.offset(i as isize)).style & (SBT_OWNERDRAW as i32)) == 0 {
                Free((*(*infoPtr).parts.offset(i as isize)).text as LPVOID);
            }
            i += 1;
        }
    } else if oldNumParts < (*infoPtr).numParts as INT {
        tmp = Alloc(
            (core::mem::size_of::<STATUSWINDOWPART>() as usize)
                .wrapping_mul((*infoPtr).numParts as usize) as DWORD,
        ) as *mut STATUSWINDOWPART;
        if tmp.is_null() {
            return FALSE;
        }
        i = 0 as INT;
        while i < oldNumParts {
            *tmp.offset(i as isize) = core::ptr::read((*infoPtr).parts.offset(i as isize));
            i += 1;
        }
        Free((*infoPtr).parts as LPVOID);
        (*infoPtr).parts = tmp;
    }
    if oldNumParts == (*infoPtr).numParts as INT {
        i = 0 as INT;
        while i < oldNumParts {
            if (*(*infoPtr).parts.offset(i as isize)).x != *parts.offset(i as isize) {
                break;
            }
            i += 1;
        }
        if i == oldNumParts {
            return TRUE;
        }
    }

    i = 0 as INT;
    while i < (*infoPtr).numParts as INT {
        (*(*infoPtr).parts.offset(i as isize)).x = *parts.offset(i as isize) as INT;
        i += 1;
    }

    if !(*infoPtr).hwndToolTip.is_null() {
        let mut nTipCount: INT = 0;
        let mut ti: TTTOOLINFOW = core::mem::zeroed();
        let mut wEmpty: WCHAR = 0 as WCHAR;

        ZeroMemory!(
            core::ptr::addr_of_mut!(ti) as *mut core::ffi::c_void,
            core::mem::size_of::<TTTOOLINFOW>() as usize,
        );
        ti.cbSize = core::mem::size_of::<TTTOOLINFOW>() as UINT;
        ti.hwnd = (*infoPtr).Self_;
        ti.lpszText = core::ptr::addr_of_mut!(wEmpty) as LPWSTR;

        nTipCount = SendMessageW(
            (*infoPtr).hwndToolTip,
            TTM_GETTOOLCOUNT as UINT,
            0 as WPARAM,
            0 as LPARAM,
        ) as INT;
        if nTipCount < (*infoPtr).numParts as INT {
            /* add tools */
            i = nTipCount;
            while i < (*infoPtr).numParts as INT {
                ti.uId = i as UINT_PTR;
                SendMessageW(
                    (*infoPtr).hwndToolTip,
                    TTM_ADDTOOLW as UINT,
                    0 as WPARAM,
                    core::ptr::addr_of_mut!(ti) as LPARAM,
                );
                i += 1;
            }
        } else if nTipCount > (*infoPtr).numParts as INT {
            /* delete tools */
            i = nTipCount - 1;
            while i >= (*infoPtr).numParts as INT {
                ti.uId = i as UINT_PTR;
                SendMessageW(
                    (*infoPtr).hwndToolTip,
                    TTM_DELTOOLW as UINT,
                    0 as WPARAM,
                    core::ptr::addr_of_mut!(ti) as LPARAM,
                );
                i -= 1;
            }
        }
    }

    STATUSBAR_SetPartBounds(infoPtr);
    InvalidateRect((*infoPtr).Self_, core::ptr::null::<RECT>(), FALSE as BOOL);
    return TRUE;
}

unsafe extern "C" fn STATUSBAR_SetTextT(
    mut infoPtr: *mut STATUS_INFO,
    mut nPart: INT,
    mut style: WORD,
    mut text: LPWSTR,
    mut isW: BOOL,
) -> BOOL {
    let mut part: *mut STATUSWINDOWPART = core::ptr::null_mut::<STATUSWINDOWPART>();
    let mut changed: BOOL = FALSE;
    let mut oldStyle: INT = 0;

    if (style as INT & (SBT_OWNERDRAW as i32)) != 0 {
    } else if ((*__wine_dbch___default).flags as INT
        & ((1 as INT) << __WINE_DBCL_TRACE as INT))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/status.c\0"
                as *const u8 as *const i8,
            b"STATUSBAR_SetTextT\0" as *const u8 as *const i8,
            722 as INT,
            b"part %d, text %s\n\0" as *const u8 as *const i8,
            nPart,
            debugstr_t(text as LPCWSTR, isW),
        );
    }

    /* MSDN says: "If the parameter is set to SB_SIMPLEID (255), the status
     * window is assumed to be a simple window */

    if nPart == 0x00ff as INT {
        part = core::ptr::addr_of_mut!((*infoPtr).part0);
    } else {
        if !(*infoPtr).parts.is_null()
            && nPart >= 0 as INT
            && nPart < (*infoPtr).numParts as INT
        {
            part = (*infoPtr).parts.offset(nPart as isize) as *mut STATUSWINDOWPART;
        }
    }
    if part.is_null() {
        return FALSE;
    }

    if (*part).style != style as INT {
        changed = TRUE;
    }

    oldStyle = (*part).style;
    (*part).style = style as INT;
    if (style as INT & (SBT_OWNERDRAW as i32)) != 0 {
        if (oldStyle & (SBT_OWNERDRAW as i32)) == 0 {
            Free((*part).text as LPVOID);
        }
        (*part).text = text;
    } else {
        let mut ntext: LPWSTR = 0 as LPWSTR;
        let mut idx: *mut WCHAR = core::ptr::null_mut::<WCHAR>();

        if !text.is_null() && !(isW != 0) {
            let mut atxt: LPCSTR = text as LPCSTR;
            let mut len: DWORD = MultiByteToWideChar(
                CP_ACP as UINT,
                0 as DWORD,
                atxt,
                -(1) as INT,
                core::ptr::null_mut::<WCHAR>(),
                0 as INT,
            ) as DWORD;
            ntext = Alloc(
                (((len + 1 as DWORD) as usize)
                    * core::mem::size_of::<WCHAR>()) as DWORD,
            ) as LPWSTR;
            if ntext.is_null() {
                return FALSE;
            }
            MultiByteToWideChar(
                CP_ACP as UINT,
                0 as DWORD,
                atxt,
                -(1) as INT,
                ntext,
                len as INT,
            );
        } else if !text.is_null() {
            ntext = Alloc(
                (((lstrlenW(text as LPCWSTR) + 1 as INT)
                    as usize)
                    * core::mem::size_of::<WCHAR>()) as DWORD,
            ) as LPWSTR;
            if ntext.is_null() {
                return FALSE;
            }
            lstrcpyW(ntext, text);
        } else {
            ntext = 0 as LPWSTR;
        }

        /* replace nonprintable characters with spaces */
        if !ntext.is_null() {
            idx = ntext as *mut WCHAR;
            while *idx != 0 {
                if iswprint(*idx) == 0 {
                    *idx = ' ' as WCHAR;
                }
                idx = idx.offset(1);
            }
        }

        /* check if text is unchanged -> no need to redraw */
        if !text.is_null() {
            if changed == 0
                && !(*part).text.is_null()
                && lstrcmpW(ntext as LPCWSTR, (*part).text as LPCWSTR) == 0
            {
                Free(ntext as LPVOID);
                return TRUE;
            }
        } else {
            if changed == 0 && (*part).text.is_null() {
                return TRUE;
            }
        }

        if (oldStyle & (SBT_OWNERDRAW as i32)) == 0 {
            Free((*part).text as LPVOID);
        }
        (*part).text = ntext;
    }

    InvalidateRect(
        (*infoPtr).Self_,
        core::ptr::addr_of_mut!((*part).bound) as LPCRECT,
        FALSE,
    );
    UpdateWindow((*infoPtr).Self_);

    return TRUE;
}

unsafe extern "C" fn STATUSBAR_SetTipTextA(infoPtr: *const STATUS_INFO, id: INT, text: LPSTR) -> LRESULT {
    if !(*infoPtr).hwndToolTip.is_null() {
        let mut ti: TTTOOLINFOA = core::mem::zeroed();
        ti.cbSize = core::mem::size_of::<TTTOOLINFOA>() as UINT;
        ti.hwnd = (*infoPtr).Self_;
        ti.uId = id as UINT_PTR;
        ti.hinst = core::ptr::null_mut();
        ti.lpszText = text;
        SendMessageA(
            (*infoPtr).hwndToolTip,
            TTM_UPDATETIPTEXTA,
            0,
            core::ptr::addr_of_mut!(ti) as LPARAM,
        );
    }

    return 0;
}

unsafe extern "C" fn STATUSBAR_SetTipTextW(
    infoPtr: *const STATUS_INFO,
    id: INT,
    text: LPWSTR,
) -> LRESULT {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/status.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"STATUSBAR_SetTipTextW\0" as *const u8 as *const ::core::ffi::c_char,
            814 as ::core::ffi::c_int,
            b"part %d: \"%s\"\n\0" as *const u8 as *const ::core::ffi::c_char,
            id,
            debugstr_w(text as *const WCHAR),
        );
    }
    if !(*infoPtr).hwndToolTip.is_null() {
        let mut ti: TTTOOLINFOW = tagTOOLINFOW {
            cbSize: 0,
            uFlags: 0,
            hwnd: core::ptr::null_mut::<HWND__>(),
            uId: 0,
            rect: tagRECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            hinst: core::ptr::null_mut::<HINSTANCE__>(),
            lpszText: core::ptr::null_mut::<WCHAR>(),
            lParam: 0,
            lpReserved: core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        ti.cbSize = core::mem::size_of::<TTTOOLINFOW>() as UINT;
        ti.hwnd = (*infoPtr).Self_;
        ti.uId = id as UINT_PTR;
        ti.hinst = core::ptr::null_mut();
        ti.lpszText = text;
        SendMessageW(
            (*infoPtr).hwndToolTip,
            TTM_UPDATETIPTEXTW,
            0,
            core::ptr::addr_of_mut!(ti) as LPARAM,
        );
    }
    return 0;
}

unsafe extern "C" fn STATUSBAR_SetUnicodeFormat(infoPtr: *mut STATUS_INFO, bUnicode: BOOL) -> LRESULT {
    let mut bOld: BOOL = (*infoPtr).bUnicode;

    (*infoPtr).bUnicode = bUnicode;

    return bOld as LRESULT;
}

unsafe extern "C" fn STATUSBAR_Simple(infoPtr: *mut STATUS_INFO, simple: BOOL) -> BOOL {
    let mut nmhdr: NMHDR = core::mem::zeroed();
    if (*infoPtr).simple == simple {
        return TRUE;
    }
    (*infoPtr).simple = simple;
    nmhdr.hwndFrom = (*infoPtr).Self_;
    nmhdr.idFrom = GetWindowLongPtrW((*infoPtr).Self_, GWLP_ID) as UINT_PTR;
    nmhdr.code = SBN_SIMPLEMODECHANGE as UINT;
    SendMessageW(
        (*infoPtr).Notify,
        WM_NOTIFY as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(nmhdr) as LPARAM,
    );
    InvalidateRect((*infoPtr).Self_, core::ptr::null::<RECT>(), FALSE);
    return TRUE;
}

unsafe extern "C" fn STATUSBAR_WMDestroy(infoPtr: *mut STATUS_INFO) -> LRESULT {
    let mut i: ::core::ffi::c_uint = 0;

    while i < (*infoPtr).numParts as ::core::ffi::c_uint {
        if ((*(*infoPtr).parts.offset(i as isize)).style as ::core::ffi::c_int & (SBT_OWNERDRAW as i32)) == 0 {
            Free((*(*infoPtr).parts.offset(i as isize)).text as LPVOID);
        }
        i = i.wrapping_add(1);
    }
    if ((*infoPtr).part0.style as ::core::ffi::c_int & (SBT_OWNERDRAW as i32)) == 0 {
        Free((*infoPtr).part0.text as LPVOID);
    }
    Free((*infoPtr).parts as LPVOID);

    /* delete default font */
    if !(*infoPtr).hDefaultFont.is_null() {
        DeleteObject((*infoPtr).hDefaultFont as HGDIOBJ);
    }

    /* delete tool tip control */
    if !(*infoPtr).hwndToolTip.is_null() {
        DestroyWindow((*infoPtr).hwndToolTip);
    }

    CloseThemeData(GetWindowTheme((*infoPtr).Self_));

    SetWindowLongPtrW((*infoPtr).Self_, 0 as ::core::ffi::c_int, 0 as LONG_PTR);
    Free(infoPtr as LPVOID);
    return 0;
}

unsafe extern "C" fn STATUSBAR_WMCreate(
    mut hwnd: HWND,
    mut lpCreate: *const CREATESTRUCTA,
) -> LRESULT {
    let mut infoPtr: *mut STATUS_INFO = ::core::ptr::null_mut::<STATUS_INFO>();
    let mut nclm: NONCLIENTMETRICSW = ::core::mem::zeroed();
    let mut dwStyle: DWORD = 0;
    let mut rect: RECT = ::core::mem::zeroed();
    let mut len: ::core::ffi::c_int = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/status.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"STATUSBAR_WMCreate\0" as *const u8 as *const ::core::ffi::c_char,
            901 as ::core::ffi::c_int,
            b"\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    'create_fail: {
        infoPtr = Alloc(core::mem::size_of::<STATUS_INFO>() as DWORD) as *mut STATUS_INFO;
        if infoPtr.is_null() {
            break 'create_fail;
        }
        SetWindowLongPtrW(
            hwnd,
            0 as ::core::ffi::c_int,
            infoPtr as DWORD_PTR as LONG_PTR,
        );

        (*infoPtr).Self_ = hwnd;
        (*infoPtr).Notify = (*lpCreate).hwndParent;
        (*infoPtr).numParts = 1 as WORD;
        (*infoPtr).parts = 0 as *mut STATUSWINDOWPART;
        (*infoPtr).simple = FALSE as BOOL;
        (*infoPtr).clrBk = CLR_DEFAULT as COLORREF;
        (*infoPtr).hFont = 0 as HFONT;
        (*infoPtr).horizontalBorder = HORZ_BORDER as INT;
        (*infoPtr).verticalBorder = VERT_BORDER as INT;
        (*infoPtr).horizontalGap = HORZ_GAP as INT;
        (*infoPtr).minHeight = GetSystemMetrics(SM_CYSIZE as ::core::ffi::c_int) as UINT;
        if (*infoPtr).minHeight & (1 as UINT) != 0 {
            (*infoPtr).minHeight = (*infoPtr).minHeight.wrapping_sub(1);
        }

        STATUSBAR_NotifyFormat(infoPtr, (*infoPtr).Notify, NF_REQUERY as INT);

        ZeroMemory!(
            &raw mut nclm as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<NONCLIENTMETRICSW>() as size_t,
        );
        nclm.cbSize = ::core::mem::size_of::<NONCLIENTMETRICSW>() as UINT;
        SystemParametersInfoW(
            SPI_GETNONCLIENTMETRICS as UINT,
            nclm.cbSize,
            &raw mut nclm as PVOID,
            0 as UINT,
        );
        (*infoPtr).hDefaultFont = CreateFontIndirectW(&raw mut nclm.lfStatusFont);

        GetClientRect(hwnd, &raw mut rect);

        /* initialize simple case */
        (*infoPtr).part0.bound = rect;
        (*infoPtr).part0.text = 0 as LPWSTR;
        (*infoPtr).part0.x = 0 as INT;
        (*infoPtr).part0.style = 0 as INT;
        (*infoPtr).part0.hIcon = 0 as HICON;

        /* initialize first part */
        (*infoPtr).parts = Alloc(core::mem::size_of::<STATUSWINDOWPART>() as DWORD) as *mut STATUSWINDOWPART;
        if (*infoPtr).parts.is_null() {
            break 'create_fail;
        }
        (*(*infoPtr).parts.offset(0 as isize)).bound = rect;
        (*(*infoPtr).parts.offset(0 as isize)).text = 0 as LPWSTR;
        (*(*infoPtr).parts.offset(0 as isize)).x = -(1 as INT);
        (*(*infoPtr).parts.offset(0 as isize)).style = 0 as INT;
        (*(*infoPtr).parts.offset(0 as isize)).hIcon = 0 as HICON;

        OpenThemeData(hwnd, &raw const themeClass as LPCWSTR);

        let mut lenOk: bool = false;
        if !(*lpCreate).lpszName.is_null() {
            len = lstrlenW((*lpCreate).lpszName as LPCWSTR);
            lenOk = len != 0;
        }
        if lenOk {
            (*(*infoPtr).parts.offset(0 as isize)).text = Alloc(
                (((len + 1 as ::core::ffi::c_int) as usize)
                    .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)) as DWORD,
            ) as LPWSTR;
            if (*(*infoPtr).parts.offset(0 as isize)).text.is_null() {
                break 'create_fail;
            }
            lstrcpyW(
                (*(*infoPtr).parts.offset(0 as isize)).text,
                (*lpCreate).lpszName as LPCWSTR,
            );
        }

        dwStyle = GetWindowLongW(hwnd, GWL_STYLE as ::core::ffi::c_int) as DWORD;
        /* native seems to clear WS_BORDER, too */
        dwStyle &= !(WS_BORDER as DWORD);
        SetWindowLongW(hwnd, GWL_STYLE as ::core::ffi::c_int, dwStyle as LONG);

        (*infoPtr).height = STATUSBAR_ComputeHeight(infoPtr);

        if (dwStyle & (SBT_TOOLTIPS as DWORD)) != 0 {
            (*infoPtr).hwndToolTip = CreateWindowExW(
                0 as DWORD,
                (TOOLTIPS_CLASSW).as_ptr() as LPCWSTR,
                ::core::ptr::null::<WCHAR>(),
                (WS_POPUP as DWORD) | (TTS_ALWAYSTIP as DWORD),
                CW_USEDEFAULT as ::core::ffi::c_int,
                CW_USEDEFAULT as ::core::ffi::c_int,
                CW_USEDEFAULT as ::core::ffi::c_int,
                CW_USEDEFAULT as ::core::ffi::c_int,
                hwnd,
                0 as HMENU,
                GetWindowLongPtrW(hwnd, GWLP_HINSTANCE as ::core::ffi::c_int) as HINSTANCE,
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
            );

            if !(*infoPtr).hwndToolTip.is_null() {
                let mut nmttc: NMTOOLTIPSCREATED = ::core::mem::zeroed();

                nmttc.hdr.hwndFrom = hwnd;
                nmttc.hdr.idFrom = GetWindowLongPtrW(hwnd, GWLP_ID as ::core::ffi::c_int) as UINT_PTR;
                nmttc.hdr.code = NM_TOOLTIPSCREATED as UINT;
                nmttc.hwndToolTips = (*infoPtr).hwndToolTip;

                SendMessageW(
                    (*lpCreate).hwndParent,
                    WM_NOTIFY as UINT,
                    nmttc.hdr.idFrom as WPARAM,
                    &raw mut nmttc as LPARAM,
                );
            }
        }

        return 0 as LRESULT;
    }
    // create_fail:
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/status.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"STATUSBAR_WMCreate\0" as *const u8 as *const ::core::ffi::c_char,
            982 as ::core::ffi::c_int,
            b"    failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if !infoPtr.is_null() {
        STATUSBAR_WMDestroy(infoPtr);
    }
    return -(1 as LRESULT);
}

unsafe extern "C" fn STATUSBAR_WMGetText(infoPtr: *const STATUS_INFO, size: INT, buf: LPWSTR) -> INT {
    let mut len: INT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/status.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"STATUSBAR_WMGetText\0" as *const u8 as *const ::core::ffi::c_char,
            995 as ::core::ffi::c_int,
            b"\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if (*(*infoPtr).parts.offset(0 as isize)).text.is_null() {
        return 0;
    }
    len = lstrlenW(
        (*(*infoPtr).parts.offset(0 as isize)).text as LPCWSTR,
    ) as INT;
    if size == 0 {
        return len;
    } else if size > len {
        lstrcpyW(
            buf,
            (*(*infoPtr).parts.offset(0 as isize)).text as LPCWSTR,
        );
        return len;
    } else {
        memcpy(
            buf as *mut ::core::ffi::c_void,
            (*(*infoPtr).parts.offset(0 as isize)).text
                as *const ::core::ffi::c_void,
            ((size - 1) as usize)
                .wrapping_mul(::core::mem::size_of::<WCHAR>()),
        );
        *buf.offset((size - 1) as isize) = 0 as WCHAR;
        return size - 1;
    }
}

unsafe extern "C" fn STATUSBAR_WMNCHitTest(infoPtr: *const STATUS_INFO, x: INT, y: INT) -> BOOL {
    if (GetWindowLongW((*infoPtr).Self_, GWL_STYLE) & (SBARS_SIZEGRIP as i32)) != 0
        && !((GetWindowLongW((*infoPtr).Notify, GWL_STYLE) & (WS_MAXIMIZE as i32)) != 0)
    {
        let mut rect: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let mut pt: POINT = POINT { x: 0, y: 0 };

        GetClientRect((*infoPtr).Self_, &raw mut rect);

        pt.x = x as LONG;
        pt.y = y as LONG;
        ScreenToClient((*infoPtr).Self_, &raw mut pt);

        if pt.x >= rect.right as INT - GetSystemMetrics(SM_CXVSCROLL as i32) {
            if (GetWindowLongW((*infoPtr).Self_, GWL_EXSTYLE) & (WS_EX_LAYOUTRTL as i32)) != 0 {
                return (HTBOTTOMLEFT) as BOOL;
            } else {
                return (HTBOTTOMRIGHT) as BOOL;
            }
        }
    }

    return HTERROR;
}

unsafe extern "C" fn STATUSBAR_WMPaint(infoPtr: *mut STATUS_INFO, mut hdc: HDC) -> LRESULT {
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/status.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"STATUSBAR_WMPaint\0" as *const u8 as *const ::core::ffi::c_char,
            1045 as ::core::ffi::c_int,
            b"\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if !hdc.is_null() {
        return STATUSBAR_Refresh(infoPtr, hdc);
    }
    hdc = BeginPaint((*infoPtr).Self_, core::ptr::addr_of_mut!(ps));
    STATUSBAR_Refresh(infoPtr, hdc);
    EndPaint((*infoPtr).Self_, core::ptr::addr_of_mut!(ps));

    return 0;
}

unsafe extern "C" fn STATUSBAR_WMSetFont(infoPtr: *mut STATUS_INFO, font: HFONT, redraw: BOOL) -> LRESULT {
    (*infoPtr).hFont = font;

    (*infoPtr).height = STATUSBAR_ComputeHeight(infoPtr);
    SendMessageW((*infoPtr).Self_, WM_SIZE, 0, 0);
    if redraw != 0 {
        InvalidateRect((*infoPtr).Self_, core::ptr::null_mut(), FALSE);
    }

    return 0;
}

unsafe extern "C" fn STATUSBAR_WMSetText(
    mut infoPtr: *const STATUS_INFO,
    mut text: LPCSTR,
) -> BOOL {
    let mut part: *mut STATUSWINDOWPART = ::core::ptr::null_mut::<STATUSWINDOWPART>();
    let mut len: ::core::ffi::c_int = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/status.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"STATUSBAR_WMSetText\0" as *const u8 as *const ::core::ffi::c_char,
            1076 as ::core::ffi::c_int,
            b"\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if (*infoPtr).numParts as ::core::ffi::c_int == 0 {
        return FALSE;
    }
    part = (*infoPtr).parts.offset(0 as isize) as *mut STATUSWINDOWPART;
    Free((*part).text as LPVOID);
    (*part).text = core::ptr::null_mut();
    if !text.is_null() && {
        len = lstrlenW(text as LPCWSTR);
        len != 0
    } {
        (*part).text = Alloc(
            ((len + 1) as usize)
                .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as DWORD,
        ) as LPWSTR;
        if (*part).text.is_null() {
            return FALSE;
        }
        lstrcpyW((*part).text, text as LPCWSTR);
    }
    InvalidateRect((*infoPtr).Self_, &raw mut (*part).bound as LPCRECT, FALSE);
    return TRUE;
}

unsafe extern "C" fn STATUSBAR_WMSize(infoPtr: *mut STATUS_INFO, flags: WORD) -> BOOL {
    let mut width: INT = 0;
    let mut x: INT = 0;
    let mut y: INT = 0;
    let mut parent_rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/status.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"STATUSBAR_WMSize\0" as *const u8 as *const ::core::ffi::c_char,
            1104 as ::core::ffi::c_int,
            b"flags %04x\n\0" as *const u8 as *const ::core::ffi::c_char,
            flags as ::core::ffi::c_int,
        );
    }
    if (flags as INT) as u32 != SIZE_RESTORED && (flags as INT) as u32 != SIZE_MAXIMIZED {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/status.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"STATUSBAR_WMSize\0" as *const u8 as *const ::core::ffi::c_char,
                1107 as ::core::ffi::c_int,
                b"flags MUST be SIZE_RESTORED or SIZE_MAXIMIZED\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        return FALSE;
    }
    if (GetWindowLongW((*infoPtr).Self_, GWL_STYLE) as LONG
        & (CCS_NORESIZE as i32))
        != 0
    {
        return FALSE;
    }
    if GetClientRect((*infoPtr).Notify, core::ptr::addr_of_mut!(parent_rect)) == 0 {
        return FALSE;
    }
    width = (parent_rect.right - parent_rect.left) as INT;
    x = parent_rect.left as INT;
    y = (parent_rect.bottom as UINT).wrapping_sub((*infoPtr).height) as INT;
    MoveWindow(
        (*infoPtr).Self_,
        x,
        y,
        width,
        (*infoPtr).height as INT,
        TRUE,
    );
    STATUSBAR_SetPartBounds(infoPtr);
    parent_rect = (*(*infoPtr)
        .parts
        .offset(((*infoPtr).numParts as INT - 1) as isize))
    .bound;
    InvalidateRect(
        (*infoPtr).Self_,
        core::ptr::addr_of_mut!(parent_rect) as LPCRECT,
        TRUE,
    );
    return TRUE;
}

unsafe extern "C" fn theme_changed(infoPtr: *const STATUS_INFO) -> LRESULT {
    let theme: HTHEME = GetWindowTheme((*infoPtr).Self_);
    CloseThemeData(theme);
    OpenThemeData((*infoPtr).Self_, core::ptr::addr_of!(themeClass) as LPCWSTR);
    return 0;
}

unsafe extern "C" fn STATUSBAR_NotifyFormat(infoPtr: *mut STATUS_INFO, from: HWND, cmd: INT) -> LRESULT {
    if (cmd) as u32 == NF_REQUERY {
        let mut i: INT = SendMessageW(from, WM_NOTIFYFORMAT as UINT, (*infoPtr).Self_ as WPARAM, NF_QUERY as LPARAM) as INT;
        (*infoPtr).bUnicode = ((i) as u32 == NFR_UNICODE) as ::core::ffi::c_int as BOOL;
    }
    return (if (*infoPtr).bUnicode != 0 {
        NFR_UNICODE
    } else {
        NFR_ANSI
    }) as LRESULT;
}

unsafe extern "C" fn STATUSBAR_SendMouseNotify(
    infoPtr: *const STATUS_INFO,
    code: UINT,
    msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut nm: NMMOUSE = core::mem::zeroed();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/status.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"STATUSBAR_SendMouseNotify\0" as *const u8 as *const ::core::ffi::c_char,
            1156 as ::core::ffi::c_int,
            b"code %04x, lParam=%lx\n\0" as *const u8 as *const ::core::ffi::c_char,
            code,
            lParam,
        );
    }
    nm.hdr.hwndFrom = (*infoPtr).Self_;
    nm.hdr.idFrom = GetWindowLongPtrW((*infoPtr).Self_, GWLP_ID) as UINT_PTR;
    nm.hdr.code = code;
    nm.pt.x = (LOWORD!(lParam) as i16) as LONG;
    nm.pt.y = (HIWORD!(lParam) as i16) as LONG;
    nm.dwItemSpec = STATUSBAR_InternalHitTest(infoPtr, &raw mut nm.pt) as DWORD_PTR;
    nm.dwItemData = 0 as DWORD_PTR;
    nm.dwHitInfo = 0x30000 as LPARAM;

    /* Do default processing if WM_NOTIFY returns zero */
    if SendMessageW(
        (*infoPtr).Notify,
        WM_NOTIFY as UINT,
        nm.hdr.idFrom as WPARAM,
        &raw mut nm as LPARAM,
    ) == 0
    {
        return DefWindowProcW((*infoPtr).Self_, msg, wParam, lParam);
    }
    return 0 as LRESULT;
}

unsafe extern "system" fn StatusWindowProc(
    mut hwnd: HWND,
    mut msg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut infoPtr: *mut STATUS_INFO =
        GetWindowLongPtrW(hwnd, 0) as *mut STATUS_INFO;
    let mut nPart: INT = (wParam as INT) & 0x00ff;
    let mut res: LRESULT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/status.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"StatusWindowProc\0" as *const u8 as *const ::core::ffi::c_char,
            1183 as ::core::ffi::c_int,
            b"hwnd=%p msg=%x wparam=%lx lparam=%lx\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hwnd,
            msg,
            wParam,
            lParam,
        );
    }
    if infoPtr.is_null() && msg != WM_CREATE {
        return DefWindowProcW(hwnd, msg, wParam, lParam);
    }
    match msg {
        SB_GETBORDERS => {
            return STATUSBAR_GetBorders(infoPtr, lParam as *mut INT) as LRESULT;
        }
        SB_GETICON => {
            return STATUSBAR_GetIcon(infoPtr, nPart) as LRESULT;
        }
        SB_GETPARTS => {
            return STATUSBAR_GetParts(infoPtr, wParam as INT, lParam as *mut INT) as LRESULT;
        }
        SB_GETRECT => {
            return STATUSBAR_GetRect(infoPtr, nPart, lParam as LPRECT) as LRESULT;
        }
        SB_GETTEXTA => {
            return STATUSBAR_GetTextA(infoPtr, nPart, lParam as LPSTR);
        }
        SB_GETTEXTW => {
            return STATUSBAR_GetTextW(infoPtr, nPart, lParam as LPWSTR);
        }
        SB_GETTEXTLENGTHA | SB_GETTEXTLENGTHW => {
            return STATUSBAR_GetTextLength(infoPtr, nPart);
        }
        SB_GETTIPTEXTA => {
            return STATUSBAR_GetTipTextA(
                infoPtr,
                LOWORD!(wParam) as INT,
                lParam as LPSTR,
                HIWORD!(wParam) as INT,
            );
        }
        SB_GETTIPTEXTW => {
            return STATUSBAR_GetTipTextW(
                infoPtr,
                LOWORD!(wParam) as INT,
                lParam as LPWSTR,
                HIWORD!(wParam) as INT,
            );
        }
        SB_GETUNICODEFORMAT => return (*infoPtr).bUnicode as LRESULT,
        SB_ISSIMPLE => return (*infoPtr).simple as LRESULT,
        SB_SETBORDERS => {
            return STATUSBAR_SetBorders(infoPtr, lParam as *mut INT) as LRESULT;
        }
        SB_SETBKCOLOR => {
            return STATUSBAR_SetBkColor(infoPtr, lParam as COLORREF) as LRESULT;
        }
        SB_SETICON => {
            return STATUSBAR_SetIcon(infoPtr, nPart, lParam as HICON) as LRESULT;
        }
        SB_SETMINHEIGHT => {
            return STATUSBAR_SetMinHeight(infoPtr, wParam as INT) as LRESULT;
        }
        SB_SETPARTS => {
            return STATUSBAR_SetParts(infoPtr, wParam as INT, lParam as LPINT) as LRESULT;
        }
        SB_SETTEXTA => {
            return STATUSBAR_SetTextT(
                infoPtr,
                nPart,
                (wParam & 0xff00) as WORD,
                lParam as LPWSTR,
                FALSE,
            ) as LRESULT;
        }
        SB_SETTEXTW => {
            return STATUSBAR_SetTextT(
                infoPtr,
                nPart,
                (wParam & 0xff00) as WORD,
                lParam as LPWSTR,
                TRUE,
            ) as LRESULT;
        }
        SB_SETTIPTEXTA => {
            return STATUSBAR_SetTipTextA(infoPtr, wParam as INT, lParam as LPSTR);
        }
        SB_SETTIPTEXTW => {
            return STATUSBAR_SetTipTextW(infoPtr, wParam as INT, lParam as LPWSTR);
        }
        SB_SETUNICODEFORMAT => {
            return STATUSBAR_SetUnicodeFormat(infoPtr, wParam as BOOL);
        }
        SB_SIMPLE => {
            return STATUSBAR_Simple(infoPtr, wParam as BOOL) as LRESULT;
        }
        WM_CREATE => {
            return STATUSBAR_WMCreate(hwnd, lParam as LPCREATESTRUCTA);
        }
        WM_DESTROY => {
            return STATUSBAR_WMDestroy(infoPtr);
        }
        WM_GETFONT => {
            return (if !(*infoPtr).hFont.is_null() {
                (*infoPtr).hFont
            } else {
                (*infoPtr).hDefaultFont
            }) as LRESULT;
        }
        WM_GETTEXT => {
            return (STATUSBAR_WMGetText(infoPtr, wParam as INT, lParam as LPWSTR)) as LRESULT;
        }
        WM_GETTEXTLENGTH => {
            return LOWORD!(STATUSBAR_GetTextLength(infoPtr, 0)) as LRESULT;
        }
        WM_LBUTTONDBLCLK => {
            return STATUSBAR_SendMouseNotify(infoPtr, NM_DBLCLK, msg, wParam, lParam);
        }
        WM_LBUTTONUP => {
            return STATUSBAR_SendMouseNotify(infoPtr, NM_CLICK, msg, wParam, lParam);
        }
        WM_MOUSEMOVE => {
            return STATUSBAR_Relay2Tip(infoPtr, msg, wParam, lParam);
        }
        WM_NCHITTEST => {
            res = (STATUSBAR_WMNCHitTest(
                infoPtr,
                (LOWORD!(lParam) as i16) as INT,
                (HIWORD!(lParam) as i16) as INT,
            )) as _;
            if res != (HTERROR) as _ {
                return res;
            }
            return DefWindowProcW(hwnd, msg, wParam, lParam);
        }
        WM_NCLBUTTONUP | WM_NCLBUTTONDOWN => {
            PostMessageW((*infoPtr).Notify, msg, wParam, lParam);
            return 0 as LRESULT;
        }
        WM_NOTIFYFORMAT => {
            return STATUSBAR_NotifyFormat(infoPtr, wParam as HWND, lParam as INT);
        }
        WM_PRINTCLIENT | WM_PAINT => {
            return STATUSBAR_WMPaint(infoPtr, wParam as HDC);
        }
        WM_RBUTTONDBLCLK => {
            return STATUSBAR_SendMouseNotify(infoPtr, NM_RDBLCLK, msg, wParam, lParam);
        }
        WM_RBUTTONUP => {
            return STATUSBAR_SendMouseNotify(infoPtr, NM_RCLICK, msg, wParam, lParam);
        }
        WM_SETFONT => {
            return STATUSBAR_WMSetFont(
                infoPtr,
                wParam as HFONT,
                LOWORD!(lParam) as BOOL,
            );
        }
        WM_SETTEXT => {
            return STATUSBAR_WMSetText(infoPtr, lParam as LPCSTR) as LRESULT;
        }
        WM_SIZE => {
            if STATUSBAR_WMSize(infoPtr, wParam as WORD) != 0 {
                return 0 as LRESULT;
            }
            return DefWindowProcW(hwnd, msg, wParam, lParam);
        }
        WM_SYSCOLORCHANGE => {
            COMCTL32_RefreshSysColors();
            return 0 as LRESULT;
        }
        WM_THEMECHANGED => {
            return theme_changed(infoPtr);
        }
        _ => {
            if msg >= WM_USER
                && msg < WM_APP
                && COMCTL32_IsReflectedMessage(msg) == 0
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
                        b"dll/win32/comctl32/status.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"StatusWindowProc\0" as *const u8 as *const ::core::ffi::c_char,
                        1323 as ::core::ffi::c_int,
                        b"unknown msg %04x wp=%04lx lp=%08lx\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        msg,
                        wParam,
                        lParam,
                    );
                }
            }
            return DefWindowProcW(hwnd, msg, wParam, lParam);
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn STATUS_Register() {

    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    ZeroMemory!(
        core::ptr::addr_of_mut!(wndClass) as *mut core::ffi::c_void,
        core::mem::size_of::<WNDCLASSW>(),
    );
    wndClass.style = (CS_GLOBALCLASS | CS_DBLCLKS | CS_VREDRAW) as UINT;
    wndClass.lpfnWndProc =
        Some(StatusWindowProc);
    wndClass.cbClsExtra = 0;
    wndClass.cbWndExtra = core::mem::size_of::<*mut STATUS_INFO>() as core::ffi::c_int;
    wndClass.hCursor = LoadCursorW(
        core::ptr::null_mut(),
        IDC_ARROW as LPWSTR as LPCWSTR,
    );
    wndClass.hbrBackground = (COLOR_BTNFACE + 1) as HBRUSH;
    wndClass.lpszClassName = (STATUSCLASSNAMEW).as_ptr();

    RegisterClassW(core::ptr::addr_of_mut!(wndClass));
}
#[no_mangle]
pub unsafe extern "C" fn STATUS_Unregister() {
    UnregisterClassW((STATUSCLASSNAMEW).as_ptr(), core::ptr::null_mut());
}
