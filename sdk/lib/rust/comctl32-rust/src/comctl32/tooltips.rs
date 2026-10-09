use crate::comctl32::commctrl::COMCTL32_IsReflectedMessage;
use crate::comctl32::commctrl::COMCTL32_RefreshSysColors;
use crate::comctl32::commctrl::DefSubclassProc;
use crate::comctl32::comctl32undoc::Free;
use crate::comctl32::commctrl::RemoveWindowSubclass;
use crate::comctl32::commctrl::SetWindowSubclass;
use crate::comctl32::comctl32undoc::Str_GetPtrAtoW;
use crate::comctl32::string::Str_GetPtrW;
use crate::comctl32::string::Str_SetPtrW;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_tooltips);
extern "system" {
    fn lstrlenW(lpString: LPCWSTR) -> ::core::ffi::c_int;
    fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR;
    fn MultiByteToWideChar( CodePage: UINT, dwFlags: DWORD, lpMultiByteStr: LPCSTR, cbMultiByte: ::core::ffi::c_int, lpWideCharStr: LPWSTR, cchWideChar: ::core::ffi::c_int, ) -> ::core::ffi::c_int;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const TOOLTIPS_CLASSW: [::core::ffi::c_ushort; 17] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_ushort; 17]>(
        *b"t\0o\0o\0l\0t\0i\0p\0s\0_\0c\0l\0a\0s\0s\x003\x002\0\0\0",
    )
};

static mut hTooltipIcons: [HICON; 4] = [::core::ptr::null::<HICON__>() as *mut HICON__; 4];

// WINE_DEFAULT_DEBUG_CHANNEL(tooltips): debug-channel declaration macro, omitted.

pub const INFOTIPSIZE: usize = 1024;
pub const TTI_ERROR: u32 = 3;

#[repr(C)]
pub struct TTTOOL_INFO {
    pub uFlags: UINT,
    pub uInternalFlags: UINT,
    pub hwnd: HWND,
    pub bNotifyUnicode: BOOL,
    pub uId: UINT_PTR,
    pub rect: RECT,
    pub hinst: HINSTANCE,
    pub lpszText: LPWSTR,
    pub lParam: LPARAM,
}

#[repr(C)]
pub struct TOOLTIPS_INFO {
    pub hwndSelf: HWND,
    pub szTipText: [WCHAR; INFOTIPSIZE],
    pub bActive: BOOL,
    pub bTrackActive: BOOL,
    pub uNumTools: UINT,
    pub clrBk: COLORREF,
    pub clrText: COLORREF,
    pub hFont: HFONT,
    pub hTitleFont: HFONT,
    pub xTrackPos: INT,
    pub yTrackPos: INT,
    pub nMaxTipWidth: INT,
    pub nTool: INT, /* tool that mouse was on on last relayed mouse move */
    pub nCurrentTool: INT,
    pub nTrackTool: INT,
    pub nReshowTime: INT,
    pub nAutoPopTime: INT,
    pub nInitialTime: INT,
    pub rcMargin: RECT,
    pub bToolBelow: BOOL,
    pub pszTitle: LPWSTR,
    pub hTitleIcon: HICON,

    pub tools: *mut TTTOOL_INFO,
}

pub type PTTTOOL_INFO = *mut TTTOOL_INFO;
pub type PTOOLTIPS_INFO = *mut TOOLTIPS_INFO;

pub const ID_TIMERSHOW: i32 = 1; /* show delay timer */

pub const ID_TIMERPOP: i32 = 2; /* auto pop timer */

pub const ID_TIMERLEAVE: i32 = 3; /* tool leave timer */

// TOOLTIPS_GetInfoPtr(hWindow): function-like macro, omitted.

pub const NORMAL_TEXT_MARGIN: i32 = 2;

pub const BALLOON_TEXT_MARGIN: i32 = (NORMAL_TEXT_MARGIN + 8);

pub const BALLOON_ROUNDEDNESS: i32 = 16;
pub const BALLOON_STEMHEIGHT: i32 = 18;
pub const BALLOON_STEMWIDTH: i32 = 18;
pub const BALLOON_STEMINDENT: i32 = 16;

pub const BALLOON_ICON_TITLE_SPACING: i32 = 8; /* horizontal spacing between icon and title */

pub const BALLOON_TITLE_TEXT_SPACING: i32 = 8; /* vertical spacing between icon/title and main text */

pub const ICON_HEIGHT: i32 = 16;

pub const ICON_WIDTH: i32 = 16;

pub const MAX_TEXT_SIZE_A: i32 = 80; /* maximum retrieving text size by ANSI message */

// TOOLTIPS_SubclassProc: forward prototype only (a function declaration), omitted here;
// the function itself is emitted with its definition.

static TOOLTIPS_Paint_ps_placeholder: u8 = 0;

unsafe extern "C" fn TOOLTIPS_GetTitleIconIndex(hIcon: HICON) -> UINT_PTR {
    let mut i: UINT;
    i = 0 as UINT;
    while i <= TTI_ERROR as UINT {
        if hTooltipIcons[i as usize] == hIcon {
            return i as UINT_PTR;
        }
        i = i.wrapping_add(1);
    }
    return hIcon as UINT_PTR;
}

unsafe extern "C" fn TOOLTIPS_InitSystemSettings(infoPtr: *mut TOOLTIPS_INFO) {
    let mut nclm: NONCLIENTMETRICSW = core::mem::zeroed();

    (*infoPtr).clrBk = comctl32_color.clrInfoBk;
    (*infoPtr).clrText = comctl32_color.clrInfoText;

    DeleteObject((*infoPtr).hFont as HGDIOBJ);
    nclm.cbSize = core::mem::size_of::<NONCLIENTMETRICSW>() as UINT;
    SystemParametersInfoW(
        SPI_GETNONCLIENTMETRICS as UINT,
        core::mem::size_of::<NONCLIENTMETRICSW>() as UINT,
        core::ptr::addr_of_mut!(nclm) as PVOID,
        0 as UINT,
    );
    (*infoPtr).hFont = CreateFontIndirectW(core::ptr::addr_of_mut!(nclm.lfStatusFont));

    DeleteObject((*infoPtr).hTitleFont as HGDIOBJ);
    nclm.lfStatusFont.lfWeight = FW_BOLD as LONG;
    (*infoPtr).hTitleFont = CreateFontIndirectW(core::ptr::addr_of_mut!(nclm.lfStatusFont));
}

unsafe extern "C" fn TOOLTIPS_customdraw_fill(infoPtr: *const TOOLTIPS_INFO, lpnmttcd: *mut NMTTCUSTOMDRAW, hdc: HDC, rcBounds: *const RECT, uFlags: UINT) {
    ZeroMemory!(lpnmttcd as *mut core::ffi::c_void, core::mem::size_of::<NMTTCUSTOMDRAW>());
    (*lpnmttcd).uDrawFlags = uFlags;
    (*lpnmttcd).nmcd.hdr.hwndFrom = (*infoPtr).hwndSelf;
    (*lpnmttcd).nmcd.hdr.code = NM_CUSTOMDRAW as UINT;
    if (*infoPtr).nCurrentTool != -1 {
        let mut toolPtr: *mut TTTOOL_INFO = (*infoPtr).tools.offset((*infoPtr).nCurrentTool as isize) as *mut TTTOOL_INFO;
        (*lpnmttcd).nmcd.hdr.idFrom = (*toolPtr).uId;
    }
    (*lpnmttcd).nmcd.hdc = hdc;
    (*lpnmttcd).nmcd.rc = *rcBounds;
}

unsafe extern "C" fn TOOLTIPS_notify_customdraw(
    mut dwDrawStage: DWORD,
    mut lpnmttcd: *mut NMTTCUSTOMDRAW,
) -> DWORD {
    let mut result: LRESULT = 0;
    (*lpnmttcd).nmcd.dwDrawStage = dwDrawStage;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/tooltips.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLTIPS_notify_customdraw\0" as *const u8 as *const ::core::ffi::c_char,
            240 as ::core::ffi::c_int,
            b"Notifying stage %d, flags %x, id %x\n\0" as *const u8
                as *const ::core::ffi::c_char,
            (*lpnmttcd).nmcd.dwDrawStage,
            (*lpnmttcd).uDrawFlags,
            (*lpnmttcd).nmcd.hdr.code,
        );
    }
    result = SendMessageW(
        GetParent((*lpnmttcd).nmcd.hdr.hwndFrom),
        WM_NOTIFY as UINT,
        0 as WPARAM,
        lpnmttcd as LPARAM,
    );
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/tooltips.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLTIPS_notify_customdraw\0" as *const u8 as *const ::core::ffi::c_char,
            245 as ::core::ffi::c_int,
            b"Notify result %x\n\0" as *const u8 as *const ::core::ffi::c_char,
            result as ::core::ffi::c_uint,
        );
    }
    return result as DWORD;
}

unsafe extern "C" fn TOOLTIPS_Refresh(infoPtr: *const TOOLTIPS_INFO, hdc: HDC) {
    let mut rc: RECT = core::mem::zeroed();
    let mut oldBkMode: INT = 0;
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut hBrush: HBRUSH = core::ptr::null_mut();
    let mut uFlags: UINT = DT_EXTERNALLEADING as UINT;
    let mut hRgn: HRGN = core::ptr::null_mut();
    let mut dwStyle: DWORD = GetWindowLongW((*infoPtr).hwndSelf, GWL_STYLE) as DWORD;
    let mut nmttcd: NMTTCUSTOMDRAW = core::mem::zeroed();
    let mut cdmode: DWORD = 0;

    if (*infoPtr).nMaxTipWidth > -(1 as INT) {
        uFlags |= DT_WORDBREAK as UINT;
    }
    if (GetWindowLongW((*infoPtr).hwndSelf, GWL_STYLE) as DWORD & TTS_NOPREFIX as DWORD) != 0 {
        uFlags |= DT_NOPREFIX as UINT;
    }
    GetClientRect((*infoPtr).hwndSelf, &raw mut rc);

    hBrush = CreateSolidBrush((*infoPtr).clrBk);

    oldBkMode = SetBkMode(hdc, TRANSPARENT as ::core::ffi::c_int) as INT;
    SetTextColor(hdc, (*infoPtr).clrText);
    hOldFont = SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ) as HFONT;

    /* Custom draw - Call PrePaint once initial properties set up     */
    /* Note: Contrary to MSDN, CDRF_SKIPDEFAULT still draws a tooltip */
    TOOLTIPS_customdraw_fill(infoPtr, &raw mut nmttcd, hdc, &raw mut rc, uFlags);
    cdmode = TOOLTIPS_notify_customdraw(CDDS_PREPAINT as DWORD, &raw mut nmttcd);
    uFlags = nmttcd.uDrawFlags;

    if dwStyle & TTS_BALLOON as DWORD != 0 {
        /* create a region to store result into */
        hRgn = CreateRectRgn(
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );

        GetWindowRgn((*infoPtr).hwndSelf, hRgn);

        /* fill the background */
        FillRgn(hdc, hRgn, hBrush);
        DeleteObject(hBrush as HGDIOBJ);
        hBrush = core::ptr::null_mut();
    } else {
        /* fill the background */
        FillRect(hdc, &raw mut rc as LPCRECT, hBrush);
        DeleteObject(hBrush as HGDIOBJ);
        hBrush = core::ptr::null_mut();
    }

    if (dwStyle & TTS_BALLOON as DWORD != 0) || !(*infoPtr).pszTitle.is_null() {
        /* calculate text rectangle */
        rc.left += (BALLOON_TEXT_MARGIN as INT + (*infoPtr).rcMargin.left) as ::core::ffi::c_int;
        rc.top += (BALLOON_TEXT_MARGIN as INT + (*infoPtr).rcMargin.top) as ::core::ffi::c_int;
        rc.right -= (BALLOON_TEXT_MARGIN as INT + (*infoPtr).rcMargin.right) as ::core::ffi::c_int;
        rc.bottom -= (BALLOON_TEXT_MARGIN as INT + (*infoPtr).rcMargin.bottom) as ::core::ffi::c_int;
        if (*infoPtr).bToolBelow != 0 {
            rc.top += BALLOON_STEMHEIGHT as ::core::ffi::c_int;
        }

        if !(*infoPtr).pszTitle.is_null() {
            let mut rcTitle: RECT = RECT {
                left: rc.left,
                top: rc.top,
                right: rc.right,
                bottom: rc.bottom,
            };
            let mut height: ::core::ffi::c_int = 0;
            let mut icon_present: BOOL = 0;
            let mut prevFont: HFONT = core::ptr::null_mut();

            /* draw icon */
            icon_present = (!(*infoPtr).hTitleIcon.is_null()
                && DrawIconEx(
                    hdc,
                    rc.left,
                    rc.top,
                    (*infoPtr).hTitleIcon,
                    ICON_WIDTH as ::core::ffi::c_int,
                    ICON_HEIGHT as ::core::ffi::c_int,
                    0 as UINT,
                    core::ptr::null_mut(),
                    DI_NORMAL as UINT,
                ) != 0) as ::core::ffi::c_int as BOOL;
            if icon_present != 0 {
                rcTitle.left += (ICON_WIDTH + BALLOON_ICON_TITLE_SPACING) as LONG;
            }

            rcTitle.bottom = (rc.top + ICON_HEIGHT as LONG) as LONG;

            /* draw title text */
            prevFont = SelectObject(hdc, (*infoPtr).hTitleFont as HGDIOBJ) as HFONT;
            height = DrawTextW(
                hdc,
                (*infoPtr).pszTitle as LPCWSTR,
                -(1 as ::core::ffi::c_int),
                &raw mut rcTitle,
                (DT_BOTTOM | DT_SINGLELINE | DT_NOPREFIX) as UINT,
            );
            SelectObject(hdc, prevFont as HGDIOBJ);
            rc.top += height + BALLOON_TITLE_TEXT_SPACING as ::core::ffi::c_int;
        }
    } else {
        /* calculate text rectangle */
        rc.left += (NORMAL_TEXT_MARGIN as INT + (*infoPtr).rcMargin.left) as ::core::ffi::c_int;
        rc.top += (NORMAL_TEXT_MARGIN as INT + (*infoPtr).rcMargin.top) as ::core::ffi::c_int;
        rc.right -= (NORMAL_TEXT_MARGIN as INT + (*infoPtr).rcMargin.right) as ::core::ffi::c_int;
        rc.bottom -= (NORMAL_TEXT_MARGIN as INT + (*infoPtr).rcMargin.bottom) as ::core::ffi::c_int;
    }

    /* draw text */
    uFlags |= DT_EXPANDTABS as UINT;
    DrawTextW(
        hdc,
        &raw const (*infoPtr).szTipText as LPCWSTR,
        -(1 as ::core::ffi::c_int),
        &raw mut rc,
        uFlags,
    );

    /* Custom draw - Call PostPaint after drawing */
    if cdmode & CDRF_NOTIFYPOSTPAINT as DWORD != 0 {
        TOOLTIPS_notify_customdraw(CDDS_POSTPAINT as DWORD, &raw mut nmttcd);
    }

    /* be polite and reset the things we changed in the dc */
    SelectObject(hdc, hOldFont as HGDIOBJ);
    SetBkMode(hdc, oldBkMode as ::core::ffi::c_int);

    if dwStyle & TTS_BALLOON as DWORD != 0 {
        /* frame region because default window proc doesn't do it */
        let mut width: INT = GetSystemMetrics((SM_CXDLGFRAME) as i32) as INT
            - GetSystemMetrics(SM_CXEDGE as ::core::ffi::c_int) as INT;
        let mut height_0: INT = GetSystemMetrics((SM_CYDLGFRAME) as i32) as INT
            - GetSystemMetrics(SM_CYEDGE as ::core::ffi::c_int) as INT;

        hBrush = GetSysColorBrush(COLOR_WINDOWFRAME as ::core::ffi::c_int);
        FrameRgn(
            hdc,
            hRgn,
            hBrush,
            width as ::core::ffi::c_int,
            height_0 as ::core::ffi::c_int,
        );
    }

    if !hRgn.is_null() {
        DeleteObject(hRgn as HGDIOBJ);
    }
}

unsafe extern "C" fn TOOLTIPS_GetDispInfoA(
    infoPtr: *const TOOLTIPS_INFO,
    toolPtr: *mut TTTOOL_INFO,
    buffer: *mut WCHAR,
) {
    let mut ttnmdi: NMTTDISPINFOA = core::mem::zeroed();

    /* fill NMHDR struct */
    ZeroMemory!(core::ptr::addr_of_mut!(ttnmdi) as *mut core::ffi::c_void, core::mem::size_of::<NMTTDISPINFOA>() as usize);
    ttnmdi.hdr.hwndFrom = (*infoPtr).hwndSelf;
    ttnmdi.hdr.idFrom = (*toolPtr).uId;
    ttnmdi.hdr.code = TTN_GETDISPINFOA as UINT; /* == TTN_NEEDTEXTA */
    ttnmdi.lpszText = core::ptr::addr_of_mut!(ttnmdi.szText) as LPSTR;
    ttnmdi.uFlags = (*toolPtr).uFlags;
    ttnmdi.lParam = (*toolPtr).lParam;

    SendMessageW((*toolPtr).hwnd, WM_NOTIFY as UINT, (*toolPtr).uId as WPARAM, core::ptr::addr_of_mut!(ttnmdi) as LPARAM);

    if IS_INTRESOURCE!(ttnmdi.lpszText) != false {
        LoadStringW(ttnmdi.hinst, LOWORD!(ttnmdi.lpszText as DWORD_PTR) as UINT,
               buffer as LPWSTR, (INFOTIPSIZE as UINT) as INT);
        if ttnmdi.uFlags & TTF_DI_SETITEM as UINT != 0 {
            (*toolPtr).hinst = ttnmdi.hinst;
            (*toolPtr).lpszText = ttnmdi.lpszText as LPWSTR;
        }
    }
    else if ttnmdi.lpszText == core::ptr::null_mut() {
        *buffer.offset(0 as isize) = '\0' as WCHAR;
    }
    else if ttnmdi.lpszText != LPSTR_TEXTCALLBACKA {
        Str_GetPtrAtoW(ttnmdi.lpszText as LPCSTR, buffer as LPWSTR, (INFOTIPSIZE as UINT) as INT);
        if ttnmdi.uFlags & TTF_DI_SETITEM as UINT != 0 {
            (*toolPtr).hinst = core::ptr::null_mut();
            (*toolPtr).lpszText = core::ptr::null_mut();
            Str_SetPtrW(core::ptr::addr_of_mut!((*toolPtr).lpszText), buffer as LPCWSTR);
        }
    }
    else {
        *buffer.offset(0 as isize) = '\0' as WCHAR;
    }

    /* no text available - try calling parent instead as per native */
    /* FIXME: Unsure if SETITEM should save the value or not        */
    if *buffer.offset(0 as isize) == 0x00 as WCHAR {

        SendMessageW(GetParent((*toolPtr).hwnd), WM_NOTIFY as UINT, (*toolPtr).uId as WPARAM, core::ptr::addr_of_mut!(ttnmdi) as LPARAM);

        if IS_INTRESOURCE!(ttnmdi.lpszText) != false {
            LoadStringW(ttnmdi.hinst, LOWORD!(ttnmdi.lpszText as DWORD_PTR) as UINT,
                   buffer as LPWSTR, (INFOTIPSIZE as UINT) as INT);
        } else if !ttnmdi.lpszText.is_null() &&
                   ttnmdi.lpszText != LPSTR_TEXTCALLBACKA {
            Str_GetPtrAtoW(ttnmdi.lpszText as LPCSTR, buffer as LPWSTR, (INFOTIPSIZE as UINT) as INT);
        }
    }
}

unsafe extern "C" fn TOOLTIPS_GetDispInfoW(infoPtr: *const TOOLTIPS_INFO, toolPtr: *mut TTTOOL_INFO, buffer: *mut WCHAR) {
    let mut ttnmdi: NMTTDISPINFOW = core::mem::zeroed();

    /* fill NMHDR struct */
    ZeroMemory!(core::ptr::addr_of_mut!(ttnmdi) as *mut core::ffi::c_void, core::mem::size_of::<NMTTDISPINFOW>() as usize);
    ttnmdi.hdr.hwndFrom = (*infoPtr).hwndSelf;
    ttnmdi.hdr.idFrom = (*toolPtr).uId;
    ttnmdi.hdr.code = (TTN_GETDISPINFOW) as UINT; /* == TTN_NEEDTEXTW */
    ttnmdi.lpszText = core::ptr::addr_of_mut!(ttnmdi.szText) as *mut WCHAR as LPWSTR;
    ttnmdi.uFlags = (*toolPtr).uFlags;
    ttnmdi.lParam = (*toolPtr).lParam;

    SendMessageW((*toolPtr).hwnd, (WM_NOTIFY) as UINT, (*toolPtr).uId as WPARAM, core::ptr::addr_of_mut!(ttnmdi) as LPARAM);

    if IS_INTRESOURCE!(ttnmdi.lpszText as *const core::ffi::c_void) != false {
        LoadStringW(ttnmdi.hinst, LOWORD!(ttnmdi.lpszText as usize as DWORD) as UINT,
               buffer as LPWSTR, ((INFOTIPSIZE) as UINT) as INT);
        if (ttnmdi.uFlags & ((TTF_DI_SETITEM) as UINT)) != 0 {
            (*toolPtr).hinst = ttnmdi.hinst;
            (*toolPtr).lpszText = ttnmdi.lpszText;
        }
    }
    else if ttnmdi.lpszText == (0) as LPWSTR {
        *buffer.add((0) as usize) = '\0' as WCHAR;
    }
    else if ttnmdi.lpszText != (LPSTR_TEXTCALLBACKW) as LPWSTR {
        Str_GetPtrW(ttnmdi.lpszText as LPCWSTR, buffer as LPWSTR, (INFOTIPSIZE) as INT);
        if (ttnmdi.uFlags & ((TTF_DI_SETITEM) as UINT)) != 0 {
            (*toolPtr).hinst = (0) as HINSTANCE;
            (*toolPtr).lpszText = core::ptr::null_mut();
            Str_SetPtrW(core::ptr::addr_of_mut!((*toolPtr).lpszText), buffer as LPCWSTR);
        }
    }
    else {
        *buffer.add((0) as usize) = '\0' as WCHAR;
    }

    /* no text available - try calling parent instead as per native */
    /* FIXME: Unsure if SETITEM should save the value or not        */
    if *buffer.add((0) as usize) == (0x00) as WCHAR {

        SendMessageW(GetParent((*toolPtr).hwnd), (WM_NOTIFY) as UINT, (*toolPtr).uId as WPARAM, core::ptr::addr_of_mut!(ttnmdi) as LPARAM);

        if IS_INTRESOURCE!(ttnmdi.lpszText as *const core::ffi::c_void) != false {
            LoadStringW(ttnmdi.hinst, LOWORD!(ttnmdi.lpszText as usize as DWORD) as UINT,
                   buffer as LPWSTR, ((INFOTIPSIZE) as UINT) as INT);
        } else if !ttnmdi.lpszText.is_null() &&
                   ttnmdi.lpszText != (LPSTR_TEXTCALLBACKW) as LPWSTR {
            Str_GetPtrW(ttnmdi.lpszText as LPCWSTR, buffer as LPWSTR, (INFOTIPSIZE) as INT);
        }
    }
}

unsafe extern "C" fn TOOLTIPS_GetTipText(infoPtr: *const TOOLTIPS_INFO, nTool: INT, buffer: *mut WCHAR) {
    let toolPtr: *mut TTTOOL_INFO = (*infoPtr).tools.offset(nTool as isize) as *mut TTTOOL_INFO;

    if IS_INTRESOURCE!((*toolPtr).lpszText) {
        /* load a resource */
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/tooltips.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TOOLTIPS_GetTipText\0" as *const u8 as *const ::core::ffi::c_char,
                496 as ::core::ffi::c_int,
                b"load res string %p %x\n\0" as *const u8 as *const ::core::ffi::c_char,
                (*toolPtr).hinst,
                LOWORD!((*toolPtr).lpszText as usize as u32) as ::core::ffi::c_int,
            );
        }
        if LoadStringW(
            (*toolPtr).hinst,
            LOWORD!((*toolPtr).lpszText as usize as u32) as UINT,
            buffer as LPWSTR,
            (INFOTIPSIZE) as INT,
        ) == 0
        {
            *buffer.offset(0 as isize) = '\0' as WCHAR;
        }
    } else if !(*toolPtr).lpszText.is_null() {
        if (*toolPtr).lpszText == LPSTR_TEXTCALLBACKW {
            if (*toolPtr).bNotifyUnicode != 0 {
                TOOLTIPS_GetDispInfoW(infoPtr, toolPtr, buffer);
            } else {
                TOOLTIPS_GetDispInfoA(infoPtr, toolPtr, buffer);
            }
        } else {
            /* the item is a usual (unicode) text */
            lstrcpynW(
                buffer as LPWSTR,
                (*toolPtr).lpszText as LPCWSTR,
                (INFOTIPSIZE as i32),
            );
        }
    } else {
        /* no text available */
        *buffer.offset(0 as isize) = '\0' as WCHAR;
    }

    if (GetWindowLongW((*infoPtr).hwndSelf, GWL_STYLE) & (TTS_NOPREFIX as i32)) == 0 {
        let mut ptrW: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
        ptrW = wcschr(buffer, '\t' as WCHAR) as *mut WCHAR;
        if !ptrW.is_null() {
            *ptrW = 0;
        }
    }

    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/tooltips.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLTIPS_GetTipText\0" as *const u8 as *const ::core::ffi::c_char,
            523 as ::core::ffi::c_int,
            b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_w(buffer),
        );
    }
}

unsafe extern "C" fn TOOLTIPS_CalcTipSize(infoPtr: *const TOOLTIPS_INFO, lpSize: LPSIZE) {
    let mut hdc: HDC = core::ptr::null_mut();
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut style: DWORD = GetWindowLongW((*infoPtr).hwndSelf, GWL_STYLE) as DWORD;
    let mut uFlags: UINT = (DT_EXTERNALLEADING | DT_CALCRECT) as UINT;
    let mut rc: RECT = RECT {
        left: 0 as LONG,
        top: 0 as LONG,
        right: 0 as LONG,
        bottom: 0 as LONG,
    };
    let mut title: SIZE = SIZE {
        cx: 0 as LONG,
        cy: 0 as LONG,
    };

    if (*infoPtr).nMaxTipWidth > -1 {
        rc.right = (*infoPtr).nMaxTipWidth as LONG;
        uFlags |= DT_WORDBREAK as UINT;
    }
    if (style & TTS_NOPREFIX as DWORD) != 0 {
        uFlags |= DT_NOPREFIX as UINT;
    }

    hdc = GetDC((*infoPtr).hwndSelf);
    if !(*infoPtr).pszTitle.is_null() {
        let mut rcTitle: RECT = RECT {
            left: 0 as LONG,
            top: 0 as LONG,
            right: 0 as LONG,
            bottom: 0 as LONG,
        };
        if !(*infoPtr).hTitleIcon.is_null() {
            title.cx = ICON_WIDTH as LONG;
            title.cy = ICON_HEIGHT as LONG;
        }
        if title.cx != 0 as LONG {
            title.cx += BALLOON_ICON_TITLE_SPACING as LONG;
        }
        hOldFont = SelectObject(hdc, (*infoPtr).hTitleFont as HGDIOBJ) as HFONT;
        DrawTextW(
            hdc,
            (*infoPtr).pszTitle as LPCWSTR,
            -1,
            core::ptr::addr_of_mut!(rcTitle),
            (DT_SINGLELINE | DT_NOPREFIX | DT_CALCRECT) as UINT,
        );
        SelectObject(hdc, hOldFont as HGDIOBJ);
        title.cy = max(title.cy, rcTitle.bottom - rcTitle.top) + BALLOON_TITLE_TEXT_SPACING as LONG;
        title.cx += rcTitle.right - rcTitle.left;
    }
    hOldFont = SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ) as HFONT;
    uFlags |= DT_EXPANDTABS as UINT;
    DrawTextW(
        hdc,
        core::ptr::addr_of!((*infoPtr).szTipText) as LPCWSTR,
        -1,
        core::ptr::addr_of_mut!(rc),
        uFlags,
    );
    SelectObject(hdc, hOldFont as HGDIOBJ);
    ReleaseDC((*infoPtr).hwndSelf, hdc);

    if ((style & TTS_BALLOON as DWORD) != 0) || !(*infoPtr).pszTitle.is_null() {
        (*lpSize).cx = max(rc.right - rc.left, title.cx)
            + 2 as LONG * BALLOON_TEXT_MARGIN as LONG
            + (*infoPtr).rcMargin.left
            + (*infoPtr).rcMargin.right;
        (*lpSize).cy = title.cy + rc.bottom - rc.top
            + 2 as LONG * BALLOON_TEXT_MARGIN as LONG
            + (*infoPtr).rcMargin.bottom
            + (*infoPtr).rcMargin.top
            + BALLOON_STEMHEIGHT as LONG;
    } else {
        (*lpSize).cx = rc.right - rc.left
            + 2 as LONG * NORMAL_TEXT_MARGIN as LONG
            + (*infoPtr).rcMargin.left
            + (*infoPtr).rcMargin.right;
        (*lpSize).cy = rc.bottom - rc.top
            + 2 as LONG * NORMAL_TEXT_MARGIN as LONG
            + (*infoPtr).rcMargin.bottom
            + (*infoPtr).rcMargin.top;
    }
}

unsafe extern "C" fn TOOLTIPS_Show(infoPtr: *mut TOOLTIPS_INFO, track_activate: BOOL) {
    let mut toolPtr: *mut TTTOOL_INFO = core::ptr::null_mut();
    let mut monitor: HMONITOR;
    let mut mon_info: MONITORINFO = core::mem::zeroed();
    let mut rect: RECT = core::mem::zeroed();
    let mut size: SIZE = core::mem::zeroed();
    let mut hdr: NMHDR = core::mem::zeroed();
    let mut ptfx: ::core::ffi::c_int = 0;
    let mut style: DWORD = GetWindowLongW((*infoPtr).hwndSelf, GWL_STYLE) as DWORD;
    let mut nTool: INT = 0;
    let mut current: INT = 0;

    if track_activate != 0 {
        if (*infoPtr).nTrackTool == -1 {
            return;
        }
        nTool = (*infoPtr).nTrackTool;
    } else {
        if (*infoPtr).nTool == -1 {
            return;
        }
        nTool = (*infoPtr).nTool;
    }

    current = (*infoPtr).nCurrentTool;
    if track_activate == 0 {
        (*infoPtr).nCurrentTool = (*infoPtr).nTool;
    }

    TOOLTIPS_GetTipText(infoPtr, nTool, core::ptr::addr_of_mut!((*infoPtr).szTipText) as *mut WCHAR);

    if (*infoPtr).szTipText[0] == ('\0' as WCHAR) {
        (*infoPtr).nCurrentTool = current;
        return;
    }

    toolPtr = (*infoPtr).tools.offset(nTool as isize);
    TOOLTIPS_CalcTipSize(infoPtr, core::ptr::addr_of_mut!(size));

    if track_activate != 0 && (((*toolPtr).uFlags & TTF_TRACK) != 0) {
        rect.left = (*infoPtr).xTrackPos;
        rect.top = (*infoPtr).yTrackPos;
        ptfx = rect.left;

        if ((*toolPtr).uFlags & TTF_CENTERTIP) != 0 {
            rect.left -= size.cx / 2;
            if (style & TTS_BALLOON) == 0 {
                rect.top -= size.cy / 2;
            }
        }
        (*infoPtr).bToolBelow = (((*infoPtr).yTrackPos + size.cy)
            <= GetSystemMetrics(SM_CYSCREEN as ::core::ffi::c_int)) as BOOL;
        if (*infoPtr).bToolBelow == 0 {
            rect.top -= size.cy;
        }

        if ((*toolPtr).uFlags & TTF_ABSOLUTE) == 0 {
            if (style & TTS_BALLOON) != 0 {
                rect.left -= BALLOON_STEMINDENT;
            } else {
                let mut rcTool: RECT = core::mem::zeroed();

                if ((*toolPtr).uFlags & TTF_IDISHWND) != 0 {
                    GetWindowRect((*toolPtr).uId as HWND, core::ptr::addr_of_mut!(rcTool));
                } else {
                    rcTool = (*toolPtr).rect;
                    MapWindowPoints(
                        (*toolPtr).hwnd,
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!(rcTool) as LPPOINT,
                        2,
                    );
                }

                /* smart placement */
                if (rect.left + size.cx > rcTool.left)
                    && (rect.left < rcTool.right)
                    && (rect.top + size.cy > rcTool.top)
                    && (rect.top < rcTool.bottom)
                {
                    rect.left = rcTool.right;
                }
            }
        }
    } else {
        if ((*toolPtr).uFlags & TTF_CENTERTIP) != 0 {
            let mut rc: RECT = core::mem::zeroed();

            if ((*toolPtr).uFlags & TTF_IDISHWND) != 0 {
                GetWindowRect((*toolPtr).uId as HWND, core::ptr::addr_of_mut!(rc));
            } else {
                rc = (*toolPtr).rect;
                MapWindowPoints(
                    (*toolPtr).hwnd,
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(rc) as LPPOINT,
                    2,
                );
            }
            rect.left = (rc.left + rc.right - size.cx) / 2;
            if (style & TTS_BALLOON) != 0 {
                ptfx = rc.left + ((rc.right - rc.left) / 2);

                /* CENTERTIP ballon tooltips default to below the field
                 * if they fit on the screen */
                if rc.bottom + size.cy > GetSystemMetrics(SM_CYSCREEN as ::core::ffi::c_int) {
                    rect.top = rc.top - size.cy;
                    (*infoPtr).bToolBelow = FALSE;
                } else {
                    (*infoPtr).bToolBelow = TRUE;
                    rect.top = rc.bottom;
                }
                rect.left = max(0, rect.left - BALLOON_STEMINDENT);
            } else {
                rect.top = rc.bottom + 2;
                (*infoPtr).bToolBelow = TRUE;
            }
        } else {
            GetCursorPos(core::ptr::addr_of_mut!(rect) as LPPOINT);
            if (style & TTS_BALLOON) != 0 {
                ptfx = rect.left;
                if rect.top - size.cy >= 0 {
                    rect.top -= size.cy;
                    (*infoPtr).bToolBelow = FALSE;
                } else {
                    (*infoPtr).bToolBelow = TRUE;
                    rect.top += 20;
                }
                rect.left = max(0, rect.left - BALLOON_STEMINDENT);
            } else {
                rect.top += 20;
                (*infoPtr).bToolBelow = TRUE;
            }
        }
    }

    rect.right = rect.left + size.cx;
    rect.bottom = rect.top + size.cy;

    /* check position */

    monitor = MonitorFromRect(core::ptr::addr_of_mut!(rect) as LPCRECT, MONITOR_DEFAULTTOPRIMARY);
    mon_info.cbSize = core::mem::size_of::<MONITORINFO>() as DWORD;
    GetMonitorInfoW(monitor, core::ptr::addr_of_mut!(mon_info));

    if true {
        if rect.right > mon_info.rcMonitor.right {
            rect.left -= size.cx - (BALLOON_STEMINDENT + BALLOON_STEMWIDTH);
            rect.right -= size.cx - (BALLOON_STEMINDENT + BALLOON_STEMWIDTH);
            if rect.right > mon_info.rcMonitor.right {
                rect.left -= rect.right - mon_info.rcMonitor.right;
                rect.right = mon_info.rcMonitor.right;
            }
        }

        if rect.left < mon_info.rcMonitor.left {
            rect.right += abs(rect.left);
            rect.left = 0;
        }

        if rect.bottom > mon_info.rcMonitor.bottom {
            let mut rc: RECT = core::mem::zeroed();
            if ((*toolPtr).uFlags & TTF_IDISHWND) != 0 {
                GetWindowRect((*toolPtr).uId as HWND, core::ptr::addr_of_mut!(rc));
            } else {
                rc = (*toolPtr).rect;
                MapWindowPoints(
                    (*toolPtr).hwnd,
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(rc) as LPPOINT,
                    2,
                );
            }
            rect.bottom = rc.top - 2;
            rect.top = rect.bottom - size.cy;
        }
    } else {
        if rect.right > mon_info.rcWork.right {
            rect.left -= rect.right - mon_info.rcWork.right + 2;
            rect.right = mon_info.rcWork.right - 2;
        }
        if rect.left < mon_info.rcWork.left {
            rect.left = mon_info.rcWork.left;
        }

        if rect.bottom > mon_info.rcWork.bottom {
            let mut rc: RECT = core::mem::zeroed();

            if ((*toolPtr).uFlags & TTF_IDISHWND) != 0 {
                GetWindowRect((*toolPtr).uId as HWND, core::ptr::addr_of_mut!(rc));
            } else {
                rc = (*toolPtr).rect;
                MapWindowPoints(
                    (*toolPtr).hwnd,
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(rc) as LPPOINT,
                    2,
                );
            }
            rect.bottom = rc.top - 2;
            rect.top = rect.bottom - size.cy;
        }
    }

    AdjustWindowRectEx(
        core::ptr::addr_of_mut!(rect),
        GetWindowLongW((*infoPtr).hwndSelf, GWL_STYLE) as DWORD,
        FALSE,
        GetWindowLongW((*infoPtr).hwndSelf, GWL_EXSTYLE) as DWORD,
    );

    if (style & TTS_BALLOON) != 0 {
        let mut hRgn: HRGN = core::ptr::null_mut();
        let mut hrStem: HRGN = core::ptr::null_mut();
        let mut pts: [POINT; 3] = core::mem::zeroed();

        ptfx -= rect.left;

        if (*infoPtr).bToolBelow != 0 {
            pts[0].x = ptfx;
            pts[0].y = 0;
            if true {
                pts[1].x = max(BALLOON_STEMINDENT, ptfx - BALLOON_STEMWIDTH);
            } else {
                pts[1].x = max(BALLOON_STEMINDENT, ptfx - (BALLOON_STEMWIDTH / 2));
            }
            pts[1].y = BALLOON_STEMHEIGHT;
            pts[2].x = pts[1].x + BALLOON_STEMWIDTH;
            pts[2].y = pts[1].y;
            if pts[2].x > (rect.right - rect.left) - BALLOON_STEMINDENT {
                pts[2].x = (rect.right - rect.left) - BALLOON_STEMINDENT;
                pts[1].x = pts[2].x - BALLOON_STEMWIDTH;
            }
        } else {
            if true {
                pts[0].x = max(BALLOON_STEMINDENT, ptfx - BALLOON_STEMWIDTH);
            } else {
                pts[0].x = max(BALLOON_STEMINDENT, ptfx - (BALLOON_STEMWIDTH / 2));
            }
            pts[0].y = (rect.bottom - rect.top) - BALLOON_STEMHEIGHT;
            pts[1].x = pts[0].x + BALLOON_STEMWIDTH;
            pts[1].y = pts[0].y;
            pts[2].x = ptfx;
            pts[2].y = rect.bottom - rect.top;
            if pts[1].x > (rect.right - rect.left) - BALLOON_STEMINDENT {
                pts[1].x = (rect.right - rect.left) - BALLOON_STEMINDENT;
                pts[0].x = pts[1].x - BALLOON_STEMWIDTH;
            }
        }

        hrStem = CreatePolygonRgn(
            core::ptr::addr_of_mut!(pts) as *mut POINT,
            ARRAY_SIZE!(pts) as ::core::ffi::c_int,
            (ALTERNATE as i32),
        );

        hRgn = CreateRoundRectRgn(
            0,
            if (*infoPtr).bToolBelow != 0 { BALLOON_STEMHEIGHT } else { 0 },
            rect.right - rect.left,
            if true {
                if (*infoPtr).bToolBelow != 0 {
                    rect.bottom - rect.top
                } else {
                    rect.bottom - rect.top - BALLOON_STEMHEIGHT + 1
                }
            } else {
                if (*infoPtr).bToolBelow != 0 {
                    rect.bottom - rect.top
                } else {
                    rect.bottom - rect.top - BALLOON_STEMHEIGHT
                }
            },
            BALLOON_ROUNDEDNESS,
            BALLOON_ROUNDEDNESS,
        );

        CombineRgn(hRgn, hRgn, hrStem, (RGN_OR as i32));
        DeleteObject(hrStem as HGDIOBJ);

        SetWindowRgn((*infoPtr).hwndSelf, hRgn, FALSE);
        /* we don't free the region handle as the system deletes it when
         * it is no longer needed */
    }

    SetWindowPos(
        (*infoPtr).hwndSelf,
        core::ptr::null_mut(),
        rect.left,
        rect.top,
        rect.right - rect.left,
        rect.bottom - rect.top,
        SWP_NOZORDER | SWP_NOACTIVATE,
    );

    hdr.hwndFrom = (*infoPtr).hwndSelf;
    hdr.idFrom = (*toolPtr).uId;
    hdr.code = (TTN_SHOW) as u32;
    SendMessageW(
        (*toolPtr).hwnd,
        WM_NOTIFY,
        (*toolPtr).uId as WPARAM,
        core::ptr::addr_of_mut!(hdr) as LPARAM,
    );

    SetWindowPos(
        (*infoPtr).hwndSelf,
        HWND_TOPMOST,
        0,
        0,
        0,
        0,
        SWP_NOSIZE | SWP_NOMOVE | SWP_SHOWWINDOW | SWP_NOACTIVATE,
    );

    /* repaint the tooltip */
    InvalidateRect((*infoPtr).hwndSelf, core::ptr::null(), TRUE);
    UpdateWindow((*infoPtr).hwndSelf);

    if track_activate == 0 {
        SetTimer(
            (*infoPtr).hwndSelf,
            ID_TIMERPOP as UINT_PTR,
            (*infoPtr).nAutoPopTime as UINT,
            None,
        );

        SetTimer(
            (*infoPtr).hwndSelf,
            ID_TIMERLEAVE as UINT_PTR,
            (*infoPtr).nReshowTime as UINT,
            None,
        );
    }
}

unsafe extern "C" fn TOOLTIPS_Hide(infoPtr: *mut TOOLTIPS_INFO) {
    let mut toolPtr: *mut TTTOOL_INFO = core::ptr::null_mut::<TTTOOL_INFO>();
    let mut hdr: NMHDR = core::mem::zeroed();

    // WINE_TRACE debug output elided (no debug channel available in this crate).

    if (*infoPtr).nCurrentTool == -1 {
        return;
    }

    toolPtr = (*infoPtr).tools.offset((*infoPtr).nCurrentTool as isize) as *mut TTTOOL_INFO;
    KillTimer((*infoPtr).hwndSelf, ID_TIMERPOP as UINT_PTR);

    hdr.hwndFrom = (*infoPtr).hwndSelf;
    hdr.idFrom = (*toolPtr).uId;
    hdr.code = TTN_POP as UINT;
    SendMessageW(
        (*toolPtr).hwnd,
        WM_NOTIFY as UINT,
        (*toolPtr).uId as WPARAM,
        core::ptr::addr_of_mut!(hdr) as LPARAM,
    );

    (*infoPtr).nCurrentTool = -1;

    SetWindowPos(
        (*infoPtr).hwndSelf,
        HWND_TOP,
        0,
        0,
        0,
        0,
        (SWP_NOZORDER | SWP_HIDEWINDOW | SWP_NOACTIVATE) as UINT,
    );
}

unsafe extern "C" fn TOOLTIPS_TrackShow(infoPtr: *mut TOOLTIPS_INFO) {
    TOOLTIPS_Show(infoPtr, TRUE);
}

unsafe extern "C" fn TOOLTIPS_TrackHide(infoPtr: *const TOOLTIPS_INFO) {
    let mut toolPtr: *mut TTTOOL_INFO = core::ptr::null_mut();
    let mut hdr: NMHDR = NMHDR {
        hwndFrom: core::ptr::null_mut(),
        idFrom: 0,
        code: 0,
    };

    if (*infoPtr).nTrackTool == -1 {
        return;
    }

    toolPtr = (*infoPtr).tools.offset((*infoPtr).nTrackTool as isize);

    hdr.hwndFrom = (*infoPtr).hwndSelf;
    hdr.idFrom = (*toolPtr).uId;
    hdr.code = (TTN_POP) as u32;
    SendMessageW(
        (*toolPtr).hwnd,
        WM_NOTIFY,
        (*toolPtr).uId as WPARAM,
        core::ptr::addr_of_mut!(hdr) as LPARAM,
    );

    SetWindowPos(
        (*infoPtr).hwndSelf,
        HWND_TOP,
        0,
        0,
        0,
        0,
        SWP_NOZORDER | SWP_HIDEWINDOW | SWP_NOACTIVATE,
    );
}

unsafe extern "C" fn TOOLTIPS_GetToolFromInfoT(
    mut infoPtr: *const TOOLTIPS_INFO,
    mut lpToolInfo: *const TTTOOLINFOW,
) -> INT {
    let mut toolPtr: *mut TTTOOL_INFO = ::core::ptr::null_mut::<TTTOOL_INFO>();
    let mut nTool: UINT = 0;
    nTool = 0 as UINT;
    while nTool < (*infoPtr).uNumTools {
        toolPtr = (*infoPtr).tools.offset(nTool as isize) as *mut TTTOOL_INFO;
        if ((*toolPtr).uFlags & (TTF_IDISHWND as UINT)) == 0
            && (*lpToolInfo).hwnd == (*toolPtr).hwnd
            && (*lpToolInfo).uId == (*toolPtr).uId
        {
            return nTool as INT;
        }
        nTool = nTool.wrapping_add(1);
    }
    nTool = 0 as UINT;
    while nTool < (*infoPtr).uNumTools {
        toolPtr = (*infoPtr).tools.offset(nTool as isize) as *mut TTTOOL_INFO;
        if ((*toolPtr).uFlags & (TTF_IDISHWND as UINT)) != 0
            && (*lpToolInfo).uId == (*toolPtr).uId
        {
            return nTool as INT;
        }
        nTool = nTool.wrapping_add(1);
    }
    return -(1 as INT);
}

unsafe extern "C" fn TOOLTIPS_GetToolFromPoint(infoPtr: *const TOOLTIPS_INFO, hwnd: HWND, lpPt: *const POINT) -> INT {
    let mut toolPtr: *mut TTTOOL_INFO = core::ptr::null_mut();
    let mut nTool: UINT;
    nTool = 0 as UINT;
    while nTool < (*infoPtr).uNumTools {
        toolPtr = (*infoPtr).tools.offset(nTool as isize) as *mut TTTOOL_INFO;
        if ((*toolPtr).uFlags & (TTF_IDISHWND as UINT)) == 0 {
            if !(hwnd != (*toolPtr).hwnd) {
                if !(PtInRect(core::ptr::addr_of!((*toolPtr).rect) as LPCRECT, *lpPt) == 0) {
                    return nTool as INT;
                }
            }
        }
        nTool = nTool.wrapping_add(1);
    }
    nTool = 0 as UINT;
    while nTool < (*infoPtr).uNumTools {
        toolPtr = (*infoPtr).tools.offset(nTool as isize) as *mut TTTOOL_INFO;
        if ((*toolPtr).uFlags & (TTF_IDISHWND as UINT)) != 0 {
            if ((*toolPtr).uId as HWND) == hwnd {
                return nTool as INT;
            }
        }
        nTool = nTool.wrapping_add(1);
    }
    return -(1 as INT);
}

unsafe extern "C" fn TOOLTIPS_CopyInfoT(
    infoPtr: *const TOOLTIPS_INFO,
    index: INT,
    ti: *mut TTTOOLINFOW,
    isW: BOOL,
) {
    let toolPtr: *const TTTOOL_INFO = (*infoPtr).tools.offset(index as isize) as *mut TTTOOL_INFO;
    (*ti).uFlags = (*toolPtr).uFlags;
    (*ti).hwnd = (*toolPtr).hwnd;
    (*ti).uId = (*toolPtr).uId;
    (*ti).rect = (*toolPtr).rect;
    (*ti).hinst = (*toolPtr).hinst;
    if !(*ti).lpszText.is_null() {
        if (*toolPtr).lpszText.is_null()
            || IS_INTRESOURCE!((*toolPtr).lpszText)
            || (*toolPtr).lpszText == LPSTR_TEXTCALLBACKW
        {
            (*ti).lpszText = (*toolPtr).lpszText;
        } else if isW != 0 {
            lstrcpyW((*ti).lpszText, (*toolPtr).lpszText as LPCWSTR);
        } else {
            /* ANSI version, the buffer is maximum 80 bytes without null. */
            WideCharToMultiByte(
                CP_ACP,
                0,
                (*toolPtr).lpszText as LPCWSTR,
                -(1),
                (*ti).lpszText as LPSTR,
                MAX_TEXT_SIZE_A,
                core::ptr::null::<CHAR>(),
                core::ptr::null_mut::<BOOL>(),
            );
        }
    }
    if (*ti).cbSize as usize >= TTTOOLINFOW_V2_SIZE {
        (*ti).lParam = (*toolPtr).lParam;
    }

    /* lpReserved is intentionally not set. */
}

unsafe extern "C" fn TOOLTIPS_IsWindowActive(hwnd: HWND) -> BOOL {
    let hwndActive: HWND = GetActiveWindow();
    if hwndActive.is_null() {
        return FALSE;
    }
    if hwndActive == hwnd {
        return TRUE;
    }
    return IsChild(hwndActive, hwnd);
}

unsafe extern "C" fn TOOLTIPS_CheckTool(infoPtr: *const TOOLTIPS_INFO, bShowTest: BOOL) -> INT {
    let mut pt: POINT = POINT { x: 0, y: 0 };
    let mut hwndTool: HWND;
    let mut nTool: INT;

    GetCursorPos(core::ptr::addr_of_mut!(pt));
    hwndTool = SendMessageW(
        (*infoPtr).hwndSelf,
        TTM_WINDOWFROMPOINT as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(pt) as LPARAM,
    ) as HWND;
    if hwndTool == (0 as HWND) {
        return -(1 as INT);
    }

    ScreenToClient(hwndTool, core::ptr::addr_of_mut!(pt));
    nTool = TOOLTIPS_GetToolFromPoint(infoPtr, hwndTool, core::ptr::addr_of_mut!(pt) as *const POINT);
    if nTool == -(1 as INT) {
        return -(1 as INT);
    }

    if ((GetWindowLongW((*infoPtr).hwndSelf, GWL_STYLE as INT) as INT)
        & (TTS_ALWAYSTIP as INT))
        == 0
        && bShowTest != 0
    {
        let ti: *mut TTTOOL_INFO = (*infoPtr).tools.offset(nTool as isize) as *mut TTTOOL_INFO;
        let hwnd: HWND = if ((*ti).uFlags & (TTF_IDISHWND as UINT)) != 0 {
            (*ti).uId as HWND
        } else {
            (*ti).hwnd
        };

        if TOOLTIPS_IsWindowActive(hwnd) == 0 {
            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/comctl32/tooltips.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"TOOLTIPS_CheckTool\0" as *const u8 as *const ::core::ffi::c_char,
                    1080 as ::core::ffi::c_int,
                    b"not active: hwnd %p, parent %p, active %p\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    hwnd,
                    GetParent(hwnd),
                    GetActiveWindow(),
                );
            }
            return -(1 as INT);
        }
    }

    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/tooltips.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLTIPS_CheckTool\0" as *const u8 as *const ::core::ffi::c_char,
            1085 as ::core::ffi::c_int,
            b"tool %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            nTool,
        );
    }

    return nTool;
}

unsafe extern "C" fn TOOLTIPS_Activate(mut infoPtr: *mut TOOLTIPS_INFO, mut activate: BOOL) -> LRESULT {
    (*infoPtr).bActive = activate;
    if (*infoPtr).bActive == 0 && (*infoPtr).nCurrentTool != -1 {
        TOOLTIPS_Hide(infoPtr);
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn TOOLTIPS_AddToolT(
    mut infoPtr: *mut TOOLTIPS_INFO,
    mut ti: *const TTTOOLINFOW,
    mut isW: BOOL,
) -> LRESULT {
    let mut toolPtr: *mut TTTOOL_INFO = core::ptr::null_mut();
    let mut nResult: INT;

    if ti.is_null() {
        return FALSE as LRESULT;
    }

    if (*ti).cbSize as usize > TTTOOLINFOW_V3_SIZE as usize && isW != 0 {
        return FALSE as LRESULT;
    }

    if (*infoPtr).uNumTools == 0 as UINT {
        (*infoPtr).tools = Alloc(core::mem::size_of::<TTTOOL_INFO>() as DWORD) as *mut TTTOOL_INFO;
        toolPtr = (*infoPtr).tools;
    } else {
        let mut oldTools: *mut TTTOOL_INFO = (*infoPtr).tools;
        (*infoPtr).tools = Alloc(
            (::core::mem::size_of::<TTTOOL_INFO>() as usize)
                .wrapping_mul(((*infoPtr).uNumTools.wrapping_add(1 as UINT)) as usize)
                as DWORD,
        ) as *mut TTTOOL_INFO;
        memcpy(
            (*infoPtr).tools as *mut ::core::ffi::c_void,
            oldTools as *const ::core::ffi::c_void,
            ((*infoPtr).uNumTools as size_t).wrapping_mul(core::mem::size_of::<TTTOOL_INFO>() as size_t),
        );
        Free(oldTools as LPVOID);
        toolPtr = (*infoPtr).tools.offset((*infoPtr).uNumTools as isize) as *mut TTTOOL_INFO;
    }

    (*infoPtr).uNumTools = (*infoPtr).uNumTools.wrapping_add(1);

    /* copy tool data */
    (*toolPtr).uFlags = (*ti).uFlags;
    (*toolPtr).uInternalFlags = (*ti).uFlags & ((TTF_SUBCLASS | TTF_IDISHWND) as UINT);
    (*toolPtr).hwnd = (*ti).hwnd;
    (*toolPtr).uId = (*ti).uId;
    (*toolPtr).rect = (*ti).rect;
    (*toolPtr).hinst = (*ti).hinst;

    if (*ti).cbSize as usize >= TTTOOLINFOW_V1_SIZE as usize {
        if IS_INTRESOURCE!((*ti).lpszText) {
            /* integer resource id: store as is */
            (*toolPtr).lpszText = (*ti).lpszText;
        } else if !(*ti).lpszText.is_null() {
            if (*ti).lpszText == LPSTR_TEXTCALLBACKW {
                /* LPSTR_TEXTCALLBACK */
                (*toolPtr).lpszText = LPSTR_TEXTCALLBACKW;
            } else if isW != 0 {
                let mut len: INT = lstrlenW((*ti).lpszText as LPCWSTR) as INT;
                (*toolPtr).lpszText = Alloc(
                    ((len + 1 as INT) as usize)
                        .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)
                        as DWORD,
                ) as LPWSTR;
                lstrcpyW((*toolPtr).lpszText, (*ti).lpszText as LPCWSTR);
            } else {
                let mut len: INT = MultiByteToWideChar(
                    CP_ACP as UINT,
                    0 as DWORD,
                    (*ti).lpszText as LPSTR as LPCSTR,
                    -(1 as INT),
                    core::ptr::null_mut::<WCHAR>(),
                    0 as INT,
                ) as INT;
                (*toolPtr).lpszText = Alloc(
                    (len as usize).wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as DWORD,
                ) as LPWSTR;
                MultiByteToWideChar(
                    CP_ACP as UINT,
                    0 as DWORD,
                    (*ti).lpszText as LPSTR as LPCSTR,
                    -(1 as INT),
                    (*toolPtr).lpszText,
                    len,
                );
            }
        }
    }

    if (*ti).cbSize as usize >= TTTOOLINFOW_V2_SIZE as usize {
        (*toolPtr).lParam = (*ti).lParam;
    }

    /* install subclassing hook */
    if ((*toolPtr).uInternalFlags & (TTF_SUBCLASS as UINT)) != 0 {
        if ((*toolPtr).uInternalFlags & (TTF_IDISHWND as UINT)) != 0 {
            SetWindowSubclass(
                (*toolPtr).uId as HWND,
                Some(
                    TOOLTIPS_SubclassProc,
                ),
                1 as UINT_PTR,
                (*infoPtr).hwndSelf as DWORD_PTR,
            );
        } else {
            SetWindowSubclass(
                (*toolPtr).hwnd,
                Some(
                    TOOLTIPS_SubclassProc,
                ),
                1 as UINT_PTR,
                (*infoPtr).hwndSelf as DWORD_PTR,
            );
        }
    }

    nResult = SendMessageW(
        (*toolPtr).hwnd,
        WM_NOTIFYFORMAT as UINT,
        (*infoPtr).hwndSelf as WPARAM,
        NF_QUERY as LPARAM,
    ) as INT;
    if nResult == NFR_ANSI as INT {
        (*toolPtr).bNotifyUnicode = FALSE as BOOL;
    } else if nResult == NFR_UNICODE as INT {
        (*toolPtr).bNotifyUnicode = TRUE as BOOL;
    } else {
    }

    return TRUE as LRESULT;
}

unsafe extern "C" fn TOOLTIPS_ResetSubclass(toolPtr: *const TTTOOL_INFO) {
    if (*toolPtr).uInternalFlags & TTF_SUBCLASS != 0 {
        SetWindowSubclass(
            if (*toolPtr).uInternalFlags & TTF_IDISHWND != 0 {
                (*toolPtr).uId as HWND
            } else {
                (*toolPtr).hwnd
            },
            Some(
                TOOLTIPS_SubclassProc,
            ),
            1,
            0,
        );
    }
}

unsafe extern "C" fn TOOLTIPS_FreeToolText(toolPtr: *mut TTTOOL_INFO) {
    if !(*toolPtr).lpszText.is_null() {
        if !IS_INTRESOURCE!((*toolPtr).lpszText) && (*toolPtr).lpszText != LPSTR_TEXTCALLBACKW {
            Free((*toolPtr).lpszText as LPVOID);
        }
        (*toolPtr).lpszText = core::ptr::null_mut();
    }
}

unsafe extern "C" fn TOOLTIPS_SetToolText(
    toolPtr: *mut TTTOOL_INFO,
    text: *mut WCHAR,
    is_unicode: BOOL,
) {
    let mut len: ::core::ffi::c_int = 0;

    TOOLTIPS_FreeToolText(toolPtr);

    if ((text as ULONG_PTR) >> 16) == 0 {
        (*toolPtr).lpszText = text as LPWSTR;
    } else if text == LPSTR_TEXTCALLBACKW {
        (*toolPtr).lpszText = LPSTR_TEXTCALLBACKW;
    } else if !text.is_null() {
        if is_unicode != 0 {
            len = lstrlenW(text as LPCWSTR);
            (*toolPtr).lpszText = Alloc(
                ((len + 1) as usize)
                    .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as DWORD,
            ) as LPWSTR;
            if !(*toolPtr).lpszText.is_null() {
                lstrcpyW((*toolPtr).lpszText, text as LPCWSTR);
            }
        } else {
            len = MultiByteToWideChar(
                CP_ACP,
                0,
                text as *mut ::core::ffi::c_char as LPCSTR,
                -1,
                ::core::ptr::null_mut::<WCHAR>(),
                0,
            );
            (*toolPtr).lpszText = Alloc(
                (len as usize).wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as DWORD,
            ) as LPWSTR;
            if !(*toolPtr).lpszText.is_null() {
                MultiByteToWideChar(
                    CP_ACP,
                    0,
                    text as *mut ::core::ffi::c_char as LPCSTR,
                    -1,
                    (*toolPtr).lpszText,
                    len,
                );
            }
        }
    }
}

unsafe extern "C" fn TOOLTIPS_DelToolT(
    infoPtr: *mut TOOLTIPS_INFO,
    ti: *const TTTOOLINFOW,
    isW: BOOL,
) -> LRESULT {
    let mut toolPtr: *mut TTTOOL_INFO;
    let mut nTool: INT;

    if ti.is_null() {
        return 0;
    }
    if isW != 0
        && ((*ti).cbSize as usize) > (TTTOOLINFOW_V2_SIZE as usize)
        && ((*ti).cbSize as usize) != (TTTOOLINFOW_V3_SIZE as usize)
    {
        return 0;
    }
    if (*infoPtr).uNumTools == 0 as UINT {
        return 0;
    }

    nTool = TOOLTIPS_GetToolFromInfoT(infoPtr, ti);

    if nTool == -(1 as INT) {
        return 0;
    }

    /* make sure the tooltip has disappeared before deleting it */
    TOOLTIPS_Hide(infoPtr);

    toolPtr = (*infoPtr).tools.offset(nTool as isize);
    TOOLTIPS_FreeToolText(toolPtr);
    TOOLTIPS_ResetSubclass(toolPtr);

    /* delete tool from tool list */
    if (*infoPtr).uNumTools == 1 as UINT {
        Free((*infoPtr).tools as LPVOID);
        (*infoPtr).tools = core::ptr::null_mut::<TTTOOL_INFO>();
    } else {
        let oldTools: *mut TTTOOL_INFO = (*infoPtr).tools;
        (*infoPtr).tools = Alloc(
            (core::mem::size_of::<TTTOOL_INFO>() as usize)
                .wrapping_mul((*infoPtr).uNumTools.wrapping_sub(1 as UINT) as usize)
                as DWORD,
        ) as *mut TTTOOL_INFO;

        if nTool > 0 as INT {
            memcpy(
                (*infoPtr).tools.offset(0 as isize) as *mut ::core::ffi::c_void,
                oldTools.offset(0 as isize) as *const ::core::ffi::c_void,
                (nTool as usize).wrapping_mul(core::mem::size_of::<TTTOOL_INFO>()),
            );
        }
        if (nTool as UINT) < (*infoPtr).uNumTools.wrapping_sub(1 as UINT) {
            memcpy(
                (*infoPtr).tools.offset(nTool as isize) as *mut ::core::ffi::c_void,
                oldTools.offset((nTool + 1 as INT) as isize)
                    as *const ::core::ffi::c_void,
                (((*infoPtr).uNumTools.wrapping_sub(nTool as UINT))
                    .wrapping_sub(1 as UINT) as usize)
                    .wrapping_mul(core::mem::size_of::<TTTOOL_INFO>()),
            );
        }

        Free(oldTools as LPVOID);
    }

    /* update any indices affected by delete */

    /* destroying tool that mouse was on on last relayed mouse move */
    if (*infoPtr).nTool == nTool {
        /* -1 means no current tool (0 means first tool) */
        (*infoPtr).nTool = -(1 as INT);
    } else if (*infoPtr).nTool > nTool {
        (*infoPtr).nTool -= 1;
    }

    if (*infoPtr).nTrackTool == nTool {
        /* -1 means no current tool (0 means first tool) */
        (*infoPtr).nTrackTool = -(1 as INT);
    } else if (*infoPtr).nTrackTool > nTool {
        (*infoPtr).nTrackTool -= 1;
    }

    if (*infoPtr).nCurrentTool == nTool {
        /* -1 means no current tool (0 means first tool) */
        (*infoPtr).nCurrentTool = -(1 as INT);
    } else if (*infoPtr).nCurrentTool > nTool {
        (*infoPtr).nCurrentTool -= 1;
    }

    (*infoPtr).uNumTools = (*infoPtr).uNumTools.wrapping_sub(1);

    return 0;
}

unsafe extern "C" fn TOOLTIPS_EnumToolsT(
    mut infoPtr: *const TOOLTIPS_INFO,
    mut uIndex: UINT,
    mut ti: *mut TTTOOLINFOW,
    mut isW: BOOL,
) -> LRESULT {
    if ti.is_null() || ((*ti).cbSize as usize) < TTTOOLINFOW_V1_SIZE {
        return (FALSE) as LRESULT;
    }
    if uIndex >= (*infoPtr).uNumTools {
        return (FALSE) as LRESULT;
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/tooltips.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLTIPS_EnumToolsT\0" as *const u8 as *const ::core::ffi::c_char,
            1329 as ::core::ffi::c_int,
            b"index=%u\n\0" as *const u8 as *const ::core::ffi::c_char,
            uIndex,
        );
    }
    TOOLTIPS_CopyInfoT(infoPtr, uIndex as INT, ti, isW);
    return (TRUE) as LRESULT;
}

unsafe extern "C" fn TOOLTIPS_GetBubbleSize(infoPtr: *const TOOLTIPS_INFO, lpToolInfo: *const TTTOOLINFOW) -> LRESULT {
    let mut nTool: INT = 0;
    let mut size: SIZE = SIZE { cx: 0, cy: 0 };

    if lpToolInfo.is_null() {
        return (FALSE) as LRESULT;
    }
    if ((*lpToolInfo).cbSize as usize) < TTTOOLINFOW_V1_SIZE as usize {
        return (FALSE) as LRESULT;
    }

    nTool = TOOLTIPS_GetToolFromInfoT(infoPtr, lpToolInfo);
    if nTool == -1 {
        return 0;
    }

    TOOLTIPS_CalcTipSize(infoPtr, core::ptr::addr_of_mut!(size));

    return MAKELRESULT!(size.cx, size.cy);
}

unsafe extern "C" fn TOOLTIPS_GetCurrentToolT(infoPtr: *const TOOLTIPS_INFO, ti: *mut TTTOOLINFOW, isW: BOOL) -> LRESULT {
    if !ti.is_null() {
        if ((*ti).cbSize as usize) < TTTOOLINFOW_V1_SIZE {
            return FALSE as LRESULT;
        }

        if (*infoPtr).nCurrentTool != -1 {
            TOOLTIPS_CopyInfoT(infoPtr, (*infoPtr).nCurrentTool, ti, isW);
        }
    }

    return ((*infoPtr).nCurrentTool != -1) as i32 as LRESULT;
}

unsafe extern "C" fn TOOLTIPS_GetDelayTime(
    mut infoPtr: *const TOOLTIPS_INFO,
    mut duration: DWORD,
) -> LRESULT {
    match duration {
        TTDT_RESHOW => return (*infoPtr).nReshowTime as LRESULT,
        TTDT_AUTOPOP => return (*infoPtr).nAutoPopTime as LRESULT,
        TTDT_INITIAL | TTDT_AUTOMATIC => {
            return (*infoPtr).nInitialTime as LRESULT
        }
        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/comctl32/tooltips.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"TOOLTIPS_GetDelayTime\0" as *const u8
                        as *const ::core::ffi::c_char,
                    1388 as ::core::ffi::c_int,
                    b"Invalid duration flag %x\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    duration,
                );
            }
        }
    }
    return -(1) as LRESULT;
}

unsafe extern "C" fn TOOLTIPS_GetMargin(infoPtr: *const TOOLTIPS_INFO, rect: *mut RECT) -> LRESULT {
    if !rect.is_null() {
        *rect = (*infoPtr).rcMargin;
    }

    return 0;
}

unsafe extern "C" fn TOOLTIPS_GetMaxTipWidth(infoPtr: *const TOOLTIPS_INFO) -> LRESULT {
    ((*infoPtr).nMaxTipWidth) as _
}

unsafe extern "C" fn TOOLTIPS_GetTextT(infoPtr: *const TOOLTIPS_INFO, ti: *mut TTTOOLINFOW, isW: BOOL) -> LRESULT {
    let mut nTool: INT;

    if ti.is_null() { return 0; }
    if (*ti).cbSize < (TTTOOLINFOW_V1_SIZE) as _ {
        return 0;
    }

    nTool = TOOLTIPS_GetToolFromInfoT(infoPtr, ti);
    if nTool == -1 { return 0; }

    if (*(*infoPtr).tools.offset(nTool as isize)).lpszText.is_null() {
        return 0;
    }

    if isW != 0 {
        *(*ti).lpszText.offset(0 as isize) = '\0' as WCHAR;
        TOOLTIPS_GetTipText(infoPtr, nTool, (*ti).lpszText);
    } else {
        let mut buffer: [WCHAR; INFOTIPSIZE] = [0; INFOTIPSIZE];

        /* NB this API is broken, there is no way for the app to determine
           what size buffer it requires nor a way to specify how long the
           one it supplies is.  According to the test result, it's up to
           80 bytes by the ANSI version. */

        buffer[0 as usize] = '\0' as WCHAR;
        TOOLTIPS_GetTipText(infoPtr, nTool, buffer.as_mut_ptr());
        WideCharToMultiByte(
            CP_ACP,
            0,
            buffer.as_ptr() as LPCWSTR,
            -1,
            (*ti).lpszText as LPSTR,
            MAX_TEXT_SIZE_A,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        );
    }

    return 0;
}

unsafe extern "C" fn TOOLTIPS_GetTipBkColor(infoPtr: *const TOOLTIPS_INFO) -> LRESULT {
    ((*infoPtr).clrBk) as _
}

unsafe extern "C" fn TOOLTIPS_GetTipTextColor(infoPtr: *const TOOLTIPS_INFO) -> LRESULT {
    ((*infoPtr).clrText) as _
}

unsafe extern "C" fn TOOLTIPS_GetToolCount(infoPtr: *const TOOLTIPS_INFO) -> LRESULT {
    ((*infoPtr).uNumTools) as _
}

unsafe extern "C" fn TOOLTIPS_GetToolInfoT(
    infoPtr: *const TOOLTIPS_INFO,
    ti: *mut TTTOOLINFOW,
    isW: BOOL,
) -> LRESULT {
    let mut nTool: INT = 0;
    let mut hwnd: HWND = core::ptr::null_mut();

    if ti.is_null() {
        return FALSE as LRESULT;
    }
    if ((*ti).cbSize as usize) < TTTOOLINFOW_V1_SIZE as usize {
        return FALSE as LRESULT;
    }
    if (*infoPtr).uNumTools == 0 {
        return FALSE as LRESULT;
    }

    nTool = TOOLTIPS_GetToolFromInfoT(infoPtr, ti);
    if nTool == -1 {
        return FALSE as LRESULT;
    }

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/tooltips.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLTIPS_GetToolInfoT\0" as *const u8 as *const ::core::ffi::c_char,
            1487 as ::core::ffi::c_int,
            b"tool %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            nTool,
        );
    }

    hwnd = (*ti).hwnd;
    TOOLTIPS_CopyInfoT(infoPtr, nTool, ti, isW);
    (*ti).hwnd = hwnd;

    return TRUE as LRESULT;
}

unsafe extern "C" fn TOOLTIPS_HitTestT(
    mut infoPtr: *const TOOLTIPS_INFO,
    mut lptthit: LPTTHITTESTINFOW,
    mut isW: BOOL,
) -> LRESULT {
    let mut nTool: INT = 0;
    if lptthit == core::ptr::null_mut() {
        return FALSE as LRESULT;
    }
    nTool = TOOLTIPS_GetToolFromPoint(infoPtr, (*lptthit).hwnd, core::ptr::addr_of_mut!((*lptthit).pt));
    if nTool == -1 {
        return FALSE as LRESULT;
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/tooltips.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLTIPS_HitTestT\0" as *const u8 as *const ::core::ffi::c_char,
            1510 as ::core::ffi::c_int,
            b"tool %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            nTool,
        );
    }
    if (*lptthit).ti.cbSize as usize >= TTTOOLINFOW_V1_SIZE {
        TOOLTIPS_CopyInfoT(infoPtr, nTool, core::ptr::addr_of_mut!((*lptthit).ti), isW);
    }
    return TRUE as LRESULT;
}

unsafe extern "C" fn TOOLTIPS_NewToolRectT(
    mut infoPtr: *mut TOOLTIPS_INFO,
    mut ti: *const TTTOOLINFOW,
) -> LRESULT {
    let mut nTool: INT = 0;
    if ti.is_null() {
        return 0;
    }
    if ((*ti).cbSize as usize) < TTTOOLINFOW_V1_SIZE {
        return (FALSE) as LRESULT;
    }
    nTool = TOOLTIPS_GetToolFromInfoT(infoPtr, ti);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/tooltips.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLTIPS_NewToolRectT\0" as *const u8 as *const ::core::ffi::c_char,
            1531 as ::core::ffi::c_int,
            b"nTool = %d, rect = %s\n\0" as *const u8 as *const ::core::ffi::c_char,
            nTool,
            wine_dbgstr_rect(core::ptr::addr_of!((*ti).rect)),
        );
    }
    if nTool == -1 {
        return 0;
    }
    (*(*infoPtr).tools.offset(nTool as isize)).rect = (*ti).rect;
    return 0;
}

unsafe extern "C" fn TOOLTIPS_Pop(infoPtr: *mut TOOLTIPS_INFO) -> LRESULT {
    TOOLTIPS_Hide(infoPtr);

    return 0;
}

unsafe extern "C" fn TOOLTIPS_RelayEvent(infoPtr: *mut TOOLTIPS_INFO, lpMsg: LPMSG) -> LRESULT {
    let mut pt: POINT = POINT { x: 0, y: 0 };
    let mut nOldTool: INT;

    if lpMsg.is_null() {
        return 0;
    }

    match (*lpMsg).message {
        WM_LBUTTONDOWN
        | WM_LBUTTONUP
        | WM_MBUTTONDOWN
        | WM_MBUTTONUP
        | WM_RBUTTONDOWN
        | WM_RBUTTONUP => {
            TOOLTIPS_Hide(infoPtr);
        }

        WM_MOUSEMOVE => {
            pt.x = LOWORD!((*lpMsg).lParam as DWORD) as i16 as LONG;
            pt.y = HIWORD!((*lpMsg).lParam as DWORD) as i16 as LONG;
            nOldTool = (*infoPtr).nTool;
            (*infoPtr).nTool = TOOLTIPS_GetToolFromPoint(
                infoPtr,
                (*lpMsg).hwnd,
                core::ptr::addr_of_mut!(pt),
            );

            if (*infoPtr).nTool != nOldTool {
                if (*infoPtr).nTool == -1 {
                    /* Moved out of all tools */
                    TOOLTIPS_Hide(infoPtr);
                    KillTimer((*infoPtr).hwndSelf, (ID_TIMERLEAVE) as UINT_PTR);
                } else if nOldTool == -1 {
                    /* Moved from outside */
                    if (*infoPtr).bActive != 0 {
                        SetTimer(
                            (*infoPtr).hwndSelf,
                            ID_TIMERSHOW as UINT_PTR,
                            (*infoPtr).nInitialTime as UINT,
                            None,
                        );
                    }
                } else {
                    /* Moved from one to another */
                    TOOLTIPS_Hide(infoPtr);
                    KillTimer((*infoPtr).hwndSelf, (ID_TIMERLEAVE) as UINT_PTR);
                    if (*infoPtr).bActive != 0 {
                        SetTimer(
                            (*infoPtr).hwndSelf,
                            ID_TIMERSHOW as UINT_PTR,
                            (*infoPtr).nReshowTime as UINT,
                            None,
                        );
                    }
                }
            } else if (*infoPtr).nCurrentTool != -1 {
                /* restart autopop */
                KillTimer((*infoPtr).hwndSelf, (ID_TIMERPOP) as UINT_PTR);
                SetTimer(
                    (*infoPtr).hwndSelf,
                    ID_TIMERPOP as UINT_PTR,
                    (*infoPtr).nAutoPopTime as UINT,
                    None,
                );
            } else if (*infoPtr).nTool != -1 && (*infoPtr).bActive != 0 {
                /* previous show attempt didn't result in tooltip so try again */
                SetTimer(
                    (*infoPtr).hwndSelf,
                    ID_TIMERSHOW as UINT_PTR,
                    (*infoPtr).nInitialTime as UINT,
                    None,
                );
            }
        }

        _ => {}
    }

    return 0;
}

unsafe extern "C" fn TOOLTIPS_SetDelayTime(
    infoPtr: *mut TOOLTIPS_INFO,
    duration: DWORD,
    mut nTime: INT,
) -> LRESULT {
    match duration {
        TTDT_AUTOMATIC => {
            if nTime <= 0 {
                nTime = GetDoubleClickTime() as INT;
            }
            (*infoPtr).nReshowTime = nTime / 5;
            (*infoPtr).nAutoPopTime = nTime * 10;
            (*infoPtr).nInitialTime = nTime;
        }
        TTDT_RESHOW => {
            if nTime < 0 {
                nTime = GetDoubleClickTime().wrapping_div(5 as UINT) as INT;
            }
            (*infoPtr).nReshowTime = nTime;
        }
        TTDT_AUTOPOP => {
            if nTime < 0 {
                nTime = GetDoubleClickTime().wrapping_mul(10 as UINT) as INT;
            }
            (*infoPtr).nAutoPopTime = nTime;
        }
        TTDT_INITIAL => {
            if nTime < 0 {
                nTime = GetDoubleClickTime() as INT;
            }
            (*infoPtr).nInitialTime = nTime;
        }
        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/comctl32/tooltips.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"TOOLTIPS_SetDelayTime\0" as *const u8
                        as *const ::core::ffi::c_char,
                    1645 as ::core::ffi::c_int,
                    b"Invalid duration flag %x\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    duration,
                );
            }
        }
    }
    return 0;
}

unsafe extern "C" fn TOOLTIPS_SetMargin(infoPtr: *mut TOOLTIPS_INFO, rect: *const RECT) -> LRESULT {
    if !rect.is_null() {
        (*infoPtr).rcMargin = *rect;
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn TOOLTIPS_SetMaxTipWidth(infoPtr: *mut TOOLTIPS_INFO, MaxWidth: INT) -> LRESULT {
    let mut nTemp: INT = (*infoPtr).nMaxTipWidth;

    (*infoPtr).nMaxTipWidth = MaxWidth;

    return nTemp as LRESULT;
}

unsafe extern "C" fn TOOLTIPS_SetTipBkColor(infoPtr: *mut TOOLTIPS_INFO, clrBk: COLORREF) -> LRESULT {
    (*infoPtr).clrBk = clrBk;

    return 0 as LRESULT;
}

unsafe extern "C" fn TOOLTIPS_SetTipTextColor(infoPtr: *mut TOOLTIPS_INFO, clrText: COLORREF) -> LRESULT {
    (*infoPtr).clrText = clrText;

    return 0 as LRESULT;
}

unsafe extern "C" fn TOOLTIPS_SetTitleT(
    infoPtr: *mut TOOLTIPS_INFO,
    uTitleIcon: UINT_PTR,
    pszTitle: LPCWSTR,
    isW: BOOL,
) -> LRESULT {
    let mut size: UINT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/tooltips.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLTIPS_SetTitleT\0" as *const u8 as *const ::core::ffi::c_char,
            1699 as ::core::ffi::c_int,
            b"hwnd = %p, title = %s, icon = %p\n\0" as *const u8
                as *const ::core::ffi::c_char,
            (*infoPtr).hwndSelf,
            debugstr_w(pszTitle as *const WCHAR),
            uTitleIcon as *mut ::core::ffi::c_void,
        );
    }

    Free((*infoPtr).pszTitle as LPVOID);

    if !pszTitle.is_null() {
        if isW != 0 {
            size = ((lstrlenW(pszTitle) + 1) as usize)
                .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as UINT;
            (*infoPtr).pszTitle = Alloc(size as DWORD) as LPWSTR;
            if (*infoPtr).pszTitle.is_null() {
                return FALSE as LRESULT;
            }
            memcpy(
                (*infoPtr).pszTitle as *mut ::core::ffi::c_void,
                pszTitle as *const ::core::ffi::c_void,
                size as usize,
            );
        } else {
            size = (::core::mem::size_of::<WCHAR>() as usize).wrapping_mul(
                MultiByteToWideChar(
                    CP_ACP,
                    0,
                    pszTitle as LPCSTR,
                    -(1 as ::core::ffi::c_int),
                    ::core::ptr::null_mut::<WCHAR>(),
                    0 as ::core::ffi::c_int,
                ) as usize,
            ) as UINT;
            (*infoPtr).pszTitle = Alloc(size as DWORD) as LPWSTR;
            if (*infoPtr).pszTitle.is_null() {
                return FALSE as LRESULT;
            }
            MultiByteToWideChar(
                CP_ACP,
                0,
                pszTitle as LPCSTR,
                -(1 as ::core::ffi::c_int),
                (*infoPtr).pszTitle,
                (size as usize).wrapping_div(::core::mem::size_of::<WCHAR>() as usize)
                    as ::core::ffi::c_int,
            );
        }
    } else {
        (*infoPtr).pszTitle = ::core::ptr::null_mut::<WCHAR>();
    }

    if uTitleIcon <= TTI_ERROR as UINT_PTR {
        (*infoPtr).hTitleIcon = hTooltipIcons[uTitleIcon as usize];
    } else {
        (*infoPtr).hTitleIcon = CopyIcon(uTitleIcon as HICON);
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
            b"dll/win32/comctl32/tooltips.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLTIPS_SetTitleT\0" as *const u8 as *const ::core::ffi::c_char,
            1730 as ::core::ffi::c_int,
            b"icon = %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*infoPtr).hTitleIcon,
        );
    }

    return TRUE as LRESULT;
}

unsafe extern "C" fn TOOLTIPS_SetToolInfoT(
    mut infoPtr: *mut TOOLTIPS_INFO,
    mut ti: *const TTTOOLINFOW,
    mut isW: BOOL,
) -> LRESULT {
    let mut toolPtr: *mut TTTOOL_INFO = core::ptr::null_mut();
    let mut nTool: INT = 0;

    if ti.is_null() {
        return 0 as LRESULT;
    }
    if ((*ti).cbSize as usize) < (TTTOOLINFOW_V1_SIZE) as usize {
        return 0 as LRESULT;
    }

    nTool = TOOLTIPS_GetToolFromInfoT(infoPtr, ti);
    if nTool == -(1 as INT) {
        return 0 as LRESULT;
    }

    toolPtr = (*infoPtr).tools.offset(nTool as isize);

    /* copy tool data */
    (*toolPtr).uFlags = (*ti).uFlags;
    (*toolPtr).hwnd = (*ti).hwnd;
    (*toolPtr).uId = (*ti).uId;
    (*toolPtr).rect = (*ti).rect;
    (*toolPtr).hinst = (*ti).hinst;

    TOOLTIPS_SetToolText(toolPtr, (*ti).lpszText as *mut WCHAR, isW);

    if (*ti).cbSize as usize >= (TTTOOLINFOW_V2_SIZE) as usize {
        (*toolPtr).lParam = (*ti).lParam;
    }

    if (*infoPtr).nCurrentTool == nTool {
        TOOLTIPS_GetTipText(
            infoPtr,
            (*infoPtr).nCurrentTool,
            core::ptr::addr_of_mut!((*infoPtr).szTipText) as *mut WCHAR,
        );

        if (*infoPtr).szTipText[(0) as usize] == (0) as WCHAR {
            TOOLTIPS_Hide(infoPtr);
        } else {
            TOOLTIPS_Show(infoPtr, FALSE as BOOL);
        }
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn TOOLTIPS_TrackActivate(
    mut infoPtr: *mut TOOLTIPS_INFO,
    mut track_activate: BOOL,
    mut ti: *const TTTOOLINFOA,
) -> LRESULT {
    if track_activate != 0 {
        if ti.is_null() {
            return 0 as LRESULT;
        }
        if ((*ti).cbSize as usize) < (TTTOOLINFOA_V1_SIZE as usize) {
            return FALSE as LRESULT;
        }
        (*infoPtr).nTrackTool = TOOLTIPS_GetToolFromInfoT(infoPtr, ti as *const TTTOOLINFOW);
        if (*infoPtr).nTrackTool != -(1 as INT) {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/comctl32/tooltips.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"TOOLTIPS_TrackActivate\0" as *const u8
                        as *const ::core::ffi::c_char,
                    1790 as ::core::ffi::c_int,
                    b"activated\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            (*infoPtr).bTrackActive = TRUE as BOOL;
            TOOLTIPS_TrackShow(infoPtr);
        }
    } else {
        TOOLTIPS_TrackHide(infoPtr);

        (*infoPtr).bTrackActive = FALSE as BOOL;
        (*infoPtr).nTrackTool = -(1 as INT);

        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/tooltips.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TOOLTIPS_TrackActivate\0" as *const u8 as *const ::core::ffi::c_char,
                1802 as ::core::ffi::c_int,
                b"deactivated\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn TOOLTIPS_TrackPosition(infoPtr: *mut TOOLTIPS_INFO, coord: LPARAM) -> LRESULT {
    (*infoPtr).xTrackPos = LOWORD!(coord as DWORD) as INT;
    (*infoPtr).yTrackPos = HIWORD!(coord as DWORD) as INT;

    if (*infoPtr).bTrackActive != 0 {
        TOOLTIPS_TrackShow(infoPtr);
    }

    return 0;
}

unsafe extern "C" fn TOOLTIPS_Update(infoPtr: *mut TOOLTIPS_INFO) -> LRESULT {
    if (*infoPtr).nCurrentTool != -1 {
        UpdateWindow((*infoPtr).hwndSelf);
    }

    return 0;
}

unsafe extern "C" fn TOOLTIPS_UpdateTipTextT(
    infoPtr: *mut TOOLTIPS_INFO,
    ti: *const TTTOOLINFOW,
    isW: BOOL,
) -> LRESULT {
    let mut toolPtr: *mut TTTOOL_INFO = ::core::ptr::null_mut::<TTTOOL_INFO>();
    let mut nTool: INT = 0;

    if ti.is_null() {
        return 0 as LRESULT;
    }
    if ((*ti).cbSize as usize) < TTTOOLINFOW_V1_SIZE {
        return FALSE as LRESULT;
    }

    nTool = TOOLTIPS_GetToolFromInfoT(infoPtr, ti);
    if nTool == -1 {
        return 0 as LRESULT;
    }

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/tooltips.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLTIPS_UpdateTipTextT\0" as *const u8 as *const ::core::ffi::c_char,
            1850 as ::core::ffi::c_int,
            b"tool %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            nTool,
        );
    }

    toolPtr = (*infoPtr).tools.offset(nTool as isize) as *mut TTTOOL_INFO;

    (*toolPtr).hinst = (*ti).hinst;

    TOOLTIPS_SetToolText(toolPtr, (*ti).lpszText as *mut WCHAR, isW);

    if (*infoPtr).nCurrentTool == -1 {
        return 0 as LRESULT;
    }
    /* force repaint */
    if (*infoPtr).bActive != 0 {
        TOOLTIPS_Show(infoPtr, FALSE);
    } else if (*infoPtr).bTrackActive != 0 {
        TOOLTIPS_Show(infoPtr, TRUE);
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn TOOLTIPS_Create(hwnd: HWND) -> LRESULT {
    let mut infoPtr: *mut TOOLTIPS_INFO = core::ptr::null_mut::<TOOLTIPS_INFO>();

    /* allocate memory for info structure */
    infoPtr = Alloc(core::mem::size_of::<TOOLTIPS_INFO>() as DWORD) as *mut TOOLTIPS_INFO;
    SetWindowLongPtrW(hwnd, 0, infoPtr as DWORD_PTR as LONG_PTR);

    /* initialize info structure */
    (*infoPtr).bActive = TRUE;
    (*infoPtr).bTrackActive = FALSE;

    (*infoPtr).nMaxTipWidth = -1;
    (*infoPtr).nTool = -1;
    (*infoPtr).nCurrentTool = -1;
    (*infoPtr).nTrackTool = -1;
    (*infoPtr).hwndSelf = hwnd;

    /* initialize colours and fonts */
    TOOLTIPS_InitSystemSettings(infoPtr);

    TOOLTIPS_SetDelayTime(infoPtr, TTDT_AUTOMATIC, 0);

    SetWindowPos(hwnd, HWND_TOP, 0, 0, 0, 0, SWP_NOZORDER | SWP_HIDEWINDOW | SWP_NOACTIVATE);

    return 0;
}

unsafe extern "C" fn TOOLTIPS_Destroy(mut infoPtr: *mut TOOLTIPS_INFO) -> LRESULT {
    let mut toolPtr: *mut TTTOOL_INFO = core::ptr::null_mut::<TTTOOL_INFO>();
    let mut i: UINT = 0;
    i = 0 as UINT;
    while i < (*infoPtr).uNumTools {
        toolPtr = (*infoPtr).tools.offset(i as isize) as *mut TTTOOL_INFO;
        TOOLTIPS_FreeToolText(toolPtr);
        TOOLTIPS_ResetSubclass(toolPtr);
        i = i.wrapping_add(1);
    }
    Free((*infoPtr).tools as LPVOID);
    /* free title string */
    Free((*infoPtr).pszTitle as LPVOID);
    /* free title icon if not a standard one */
    if TOOLTIPS_GetTitleIconIndex((*infoPtr).hTitleIcon) > TTI_ERROR as UINT_PTR {
        DeleteObject((*infoPtr).hTitleIcon as HGDIOBJ);
    }
    /* delete fonts */
    DeleteObject((*infoPtr).hFont as HGDIOBJ);
    DeleteObject((*infoPtr).hTitleFont as HGDIOBJ);
    /* free tool tips info data */
    SetWindowLongPtrW((*infoPtr).hwndSelf, 0 as ::core::ffi::c_int, 0 as LONG_PTR);
    Free(infoPtr as LPVOID);
    return 0 as LRESULT;
}

unsafe extern "C" fn TOOLTIPS_GetFont(infoPtr: *const TOOLTIPS_INFO) -> LRESULT {
    return (*infoPtr).hFont as LRESULT;
}

unsafe extern "C" fn TOOLTIPS_MouseMessage(infoPtr: *mut TOOLTIPS_INFO) -> LRESULT {
    TOOLTIPS_Hide(infoPtr);

    return 0;
}

unsafe extern "C" fn TOOLTIPS_NCCreate(hwnd: HWND) -> LRESULT {
    let mut dwStyle: DWORD = GetWindowLongW(hwnd, GWL_STYLE) as DWORD;
    let mut dwExStyle: DWORD = GetWindowLongW(hwnd, GWL_EXSTYLE) as DWORD;

    dwStyle &= !(WS_CHILD | /*WS_MAXIMIZE |*/ WS_BORDER | WS_DLGFRAME) as DWORD;
    dwStyle |= (WS_POPUP | WS_BORDER | WS_CLIPSIBLINGS) as DWORD;

    /* WS_BORDER only draws a border round the window rect, not the
     * window region, therefore it is useless to us in balloon mode */
    if dwStyle & TTS_BALLOON as DWORD != 0 {
        dwStyle &= !(WS_BORDER as DWORD);
    }

    SetWindowLongW(hwnd, GWL_STYLE, dwStyle as LONG);

    dwExStyle |= WS_EX_TOOLWINDOW as DWORD;
    SetWindowLongW(hwnd, GWL_EXSTYLE, dwExStyle as LONG);

    return TRUE as LRESULT;
}

unsafe extern "C" fn TOOLTIPS_NCHitTest(
    infoPtr: *const TOOLTIPS_INFO,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let nTool: INT = if (*infoPtr).bTrackActive != 0 {
        (*infoPtr).nTrackTool
    } else {
        (*infoPtr).nTool
    };

    if nTool > -1 && (nTool as UINT) < (*infoPtr).uNumTools {
        if (*(*infoPtr).tools.offset(nTool as isize)).uFlags & TTF_TRANSPARENT as UINT != 0 {
            return HTTRANSPARENT as LRESULT;
        }
    }

    return DefWindowProcW((*infoPtr).hwndSelf, WM_NCHITTEST as UINT, wParam, lParam);
}

unsafe extern "C" fn TOOLTIPS_NotifyFormat(
    mut infoPtr: *mut TOOLTIPS_INFO,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut toolPtr: *mut TTTOOL_INFO = (*infoPtr).tools;
    let mut nResult: LRESULT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/tooltips.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLTIPS_NotifyFormat\0" as *const u8 as *const ::core::ffi::c_char,
            1996 as ::core::ffi::c_int,
            b"infoPtr=%p wParam=%lx lParam=%p\n\0" as *const u8
                as *const ::core::ffi::c_char,
            infoPtr,
            wParam,
            lParam as PVOID,
        );
    }
    if lParam == (NF_QUERY) as LPARAM {
        if (*toolPtr).bNotifyUnicode != 0 {
            return (NFR_UNICODE) as LRESULT;
        } else {
            return (NFR_ANSI) as LRESULT;
        }
    } else if lParam == (NF_REQUERY) as LPARAM {
        nResult = SendMessageW(
            (*toolPtr).hwnd,
            (WM_NOTIFYFORMAT) as UINT,
            (*infoPtr).hwndSelf as WPARAM,
            (NF_QUERY) as LPARAM,
        );
        if nResult == (NFR_ANSI) as LRESULT {
            (*toolPtr).bNotifyUnicode = (FALSE) as BOOL;
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/tooltips.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"TOOLTIPS_NotifyFormat\0" as *const u8
                        as *const ::core::ffi::c_char,
                    2010 as ::core::ffi::c_int,
                    b" -- WM_NOTIFYFORMAT returns: NFR_ANSI\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
        } else if nResult == (NFR_UNICODE) as LRESULT {
            (*toolPtr).bNotifyUnicode = (TRUE) as BOOL;
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/win32/comctl32/tooltips.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"TOOLTIPS_NotifyFormat\0" as *const u8
                        as *const ::core::ffi::c_char,
                    2013 as ::core::ffi::c_int,
                    b" -- WM_NOTIFYFORMAT returns: NFR_UNICODE\n\0" as *const u8
                        as *const ::core::ffi::c_char,
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
                b"dll/win32/comctl32/tooltips.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TOOLTIPS_NotifyFormat\0" as *const u8 as *const ::core::ffi::c_char,
                2015 as ::core::ffi::c_int,
                b" -- WM_NOTIFYFORMAT returns: error!\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        return nResult;
    }
    return (0) as LRESULT;
}

unsafe extern "C" fn TOOLTIPS_Paint(infoPtr: *const TOOLTIPS_INFO, hDC: HDC) -> LRESULT {
    let mut hdc: HDC;
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    hdc = if hDC.is_null() {
        BeginPaint((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(ps))
    } else {
        hDC
    };
    TOOLTIPS_Refresh(infoPtr, hdc);
    if hDC.is_null() {
        EndPaint((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(ps));
    }
    return 0;
}

unsafe extern "C" fn TOOLTIPS_SetFont(
    mut infoPtr: *mut TOOLTIPS_INFO,
    mut hFont: HFONT,
    mut redraw: BOOL,
) -> LRESULT {
    let mut lf: LOGFONTW = core::mem::zeroed::<LOGFONTW>();
    if GetObjectW(
        hFont as HANDLE,
        ::core::mem::size_of::<LOGFONTW>() as ::core::ffi::c_int,
        &raw mut lf as LPVOID,
    ) == 0
    {
        return 0 as LRESULT;
    }
    DeleteObject((*infoPtr).hFont as HGDIOBJ);
    (*infoPtr).hFont = CreateFontIndirectW(&raw mut lf);
    DeleteObject((*infoPtr).hTitleFont as HGDIOBJ);
    lf.lfWeight = FW_BOLD as LONG;
    (*infoPtr).hTitleFont = CreateFontIndirectW(&raw mut lf);
    if redraw != 0 && (*infoPtr).nCurrentTool != -(1 as ::core::ffi::c_int) {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/tooltips.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TOOLTIPS_SetFont\0" as *const u8 as *const ::core::ffi::c_char,
                2057 as ::core::ffi::c_int,
                b"full redraw needed\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn TOOLTIPS_GetTextLength(infoPtr: *const TOOLTIPS_INFO) -> LRESULT {
    return lstrlenW(core::ptr::addr_of!((*infoPtr).szTipText) as LPCWSTR) as LRESULT;
}

unsafe extern "C" fn TOOLTIPS_OnWMGetText(infoPtr: *const TOOLTIPS_INFO, size: WPARAM, pszText: LPWSTR) -> LRESULT {
    let mut res: LRESULT = 0;

    if size == 0 {
        return 0;
    }

    res = min(
        (lstrlenW(&raw const (*infoPtr).szTipText as LPCWSTR) + 1) as WPARAM,
        size,
    ) as LRESULT;
    memcpy(
        pszText as *mut ::core::ffi::c_void,
        &raw const (*infoPtr).szTipText as *const WCHAR as *const ::core::ffi::c_void,
        (res as usize).wrapping_mul(::core::mem::size_of::<WCHAR>()),
    );
    *pszText.offset((res - 1) as isize) = '\0' as WCHAR;
    return res - 1;
}

unsafe extern "C" fn TOOLTIPS_Timer(mut infoPtr: *mut TOOLTIPS_INFO, mut iTimer: INT) -> LRESULT {
    let mut nOldTool: INT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/tooltips.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLTIPS_Timer\0" as *const u8 as *const ::core::ffi::c_char,
            2106 as ::core::ffi::c_int,
            b"timer %d (%p) expired\n\0" as *const u8 as *const ::core::ffi::c_char,
            iTimer,
            (*infoPtr).hwndSelf,
        );
    }
    match iTimer {
        ID_TIMERSHOW => {
            KillTimer((*infoPtr).hwndSelf, ID_TIMERSHOW as UINT_PTR);
            nOldTool = (*infoPtr).nTool;
            (*infoPtr).nTool = TOOLTIPS_CheckTool(infoPtr, TRUE);
            if (*infoPtr).nTool == nOldTool {
                TOOLTIPS_Show(infoPtr, FALSE);
            }
        }
        ID_TIMERPOP => {
            TOOLTIPS_Hide(infoPtr);
        }
        ID_TIMERLEAVE => {
            nOldTool = (*infoPtr).nTool;
            (*infoPtr).nTool = TOOLTIPS_CheckTool(infoPtr, FALSE);
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/tooltips.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"TOOLTIPS_Timer\0" as *const u8 as *const ::core::ffi::c_char,
                    2124 as ::core::ffi::c_int,
                    b"tool (%p) %d %d %d\n\0" as *const u8 as *const ::core::ffi::c_char,
                    (*infoPtr).hwndSelf,
                    nOldTool,
                    (*infoPtr).nTool,
                    (*infoPtr).nCurrentTool,
                );
            }
            if (*infoPtr).nTool != nOldTool {
                if (*infoPtr).nTool == -(1) {
                    TOOLTIPS_Hide(infoPtr);
                    KillTimer((*infoPtr).hwndSelf, ID_TIMERLEAVE as UINT_PTR);
                } else if nOldTool == -(1) {
                    if (*__wine_dbch___default).flags as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                        != 0
                    {
                        let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_1: __wine_debug_class = __WINE_DBCL_ERR;
                        ros_dbg_log(
                            __dbcl_1,
                            __dbch_1,
                            b"dll/win32/comctl32/tooltips.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"TOOLTIPS_Timer\0" as *const u8
                                as *const ::core::ffi::c_char,
                            2130 as ::core::ffi::c_int,
                            b"How did this happen?\n\0" as *const u8
                                as *const ::core::ffi::c_char,
                        );
                    }
                } else {
                    TOOLTIPS_Hide(infoPtr);
                    KillTimer((*infoPtr).hwndSelf, ID_TIMERLEAVE as UINT_PTR);
                    if (*infoPtr).bActive != 0 {
                        SetTimer(
                            (*infoPtr).hwndSelf,
                            ID_TIMERSHOW as UINT_PTR,
                            (*infoPtr).nReshowTime as UINT,
                            None,
                        );
                        if (*__wine_dbch___default).flags as ::core::ffi::c_int
                            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                            != 0
                        {
                            let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                            let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
                            ros_dbg_log(
                                __dbcl_2,
                                __dbch_2,
                                b"dll/win32/comctl32/tooltips.c\0"
                                    as *const u8 as *const ::core::ffi::c_char,
                                b"TOOLTIPS_Timer\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                2136 as ::core::ffi::c_int,
                                b"timer 1 started!\n\0" as *const u8
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    }
                }
            }
        }
        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_3: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_3,
                    __dbch_3,
                    b"dll/win32/comctl32/tooltips.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"TOOLTIPS_Timer\0" as *const u8 as *const ::core::ffi::c_char,
                    2143 as ::core::ffi::c_int,
                    b"Unknown timer id %d\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    iTimer,
                );
            }
        }
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn TOOLTIPS_WinIniChange(infoPtr: *mut TOOLTIPS_INFO) -> LRESULT {
    TOOLTIPS_InitSystemSettings(infoPtr);

    return 0;
}

unsafe extern "system" fn TOOLTIPS_SubclassProc(
    mut hwnd: HWND,
    mut message: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
    mut uID: UINT_PTR,
    mut dwRef: DWORD_PTR,
) -> LRESULT {
    let mut infoPtr: *mut TOOLTIPS_INFO = TOOLTIPS_GetInfoPtr!(dwRef as HWND);
    let mut msg: MSG = core::mem::zeroed();
    match message {
        WM_MOUSEMOVE | WM_LBUTTONDOWN | WM_LBUTTONUP | WM_MBUTTONDOWN | WM_MBUTTONUP | WM_RBUTTONDOWN | WM_RBUTTONUP => {
            if !infoPtr.is_null() {
                msg.hwnd = hwnd;
                msg.message = message;
                msg.wParam = wParam;
                msg.lParam = lParam;
                TOOLTIPS_RelayEvent(infoPtr, core::ptr::addr_of_mut!(msg));
            }
        }
        WM_NCDESTROY => {
            RemoveWindowSubclass(
                hwnd,
                Some(
                    TOOLTIPS_SubclassProc
                        as unsafe extern "system" fn(
                            HWND,
                            UINT,
                            WPARAM,
                            LPARAM,
                            UINT_PTR,
                            DWORD_PTR,
                        ) -> LRESULT,
                ),
                1 as UINT_PTR,
            );
        }
        _ => {}
    }
    return DefSubclassProc(hwnd, message, wParam, lParam);
}

unsafe extern "system" fn TOOLTIPS_WindowProc(
    mut hwnd: HWND,
    mut uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut infoPtr: *mut TOOLTIPS_INFO = GetWindowLongPtrW(hwnd, 0) as *mut TOOLTIPS_INFO;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/tooltips.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLTIPS_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
            2199 as ::core::ffi::c_int,
            b"hwnd=%p msg=%x wparam=%lx lParam=%lx\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hwnd,
            uMsg,
            wParam,
            lParam,
        );
    }
    if !infoPtr.is_null() == false
        && (uMsg != WM_CREATE)
        && (uMsg != WM_NCCREATE)
    {
        return DefWindowProcW(hwnd, uMsg, wParam, lParam);
    }
    match uMsg {
        TTM_ACTIVATE => {
            return TOOLTIPS_Activate(infoPtr, wParam as BOOL);
        }
        TTM_ADDTOOLA | TTM_ADDTOOLW => {
            return TOOLTIPS_AddToolT(
                infoPtr,
                lParam as LPTTTOOLINFOW as *const TTTOOLINFOW,
                (uMsg == TTM_ADDTOOLW) as ::core::ffi::c_int,
            );
        }
        TTM_DELTOOLA | TTM_DELTOOLW => {
            return TOOLTIPS_DelToolT(
                infoPtr,
                lParam as LPTOOLINFOW as *const TTTOOLINFOW,
                (uMsg == TTM_DELTOOLW) as ::core::ffi::c_int,
            );
        }
        TTM_ENUMTOOLSA | TTM_ENUMTOOLSW => {
            return TOOLTIPS_EnumToolsT(
                infoPtr,
                wParam as UINT,
                lParam as LPTTTOOLINFOW as *mut TTTOOLINFOW,
                (uMsg == TTM_ENUMTOOLSW) as ::core::ffi::c_int,
            );
        }
        TTM_GETBUBBLESIZE => {
            return TOOLTIPS_GetBubbleSize(infoPtr, lParam as LPTTTOOLINFOW as *const TTTOOLINFOW);
        }
        TTM_GETCURRENTTOOLA | TTM_GETCURRENTTOOLW => {
            return TOOLTIPS_GetCurrentToolT(
                infoPtr,
                lParam as LPTTTOOLINFOW as *mut TTTOOLINFOW,
                (uMsg == TTM_GETCURRENTTOOLW) as ::core::ffi::c_int,
            );
        }
        TTM_GETDELAYTIME => {
            return TOOLTIPS_GetDelayTime(infoPtr, wParam as DWORD);
        }
        TTM_GETMARGIN => {
            return TOOLTIPS_GetMargin(infoPtr, lParam as *mut RECT);
        }
        TTM_GETMAXTIPWIDTH => {
            return TOOLTIPS_GetMaxTipWidth(infoPtr);
        }
        TTM_GETTEXTA | TTM_GETTEXTW => {
            return TOOLTIPS_GetTextT(
                infoPtr,
                lParam as LPTTTOOLINFOW as *mut TTTOOLINFOW,
                (uMsg == TTM_GETTEXTW) as ::core::ffi::c_int,
            );
        }
        TTM_GETTIPBKCOLOR => {
            return TOOLTIPS_GetTipBkColor(infoPtr);
        }
        TTM_GETTIPTEXTCOLOR => {
            return TOOLTIPS_GetTipTextColor(infoPtr);
        }
        TTM_GETTOOLCOUNT => {
            return TOOLTIPS_GetToolCount(infoPtr);
        }
        TTM_GETTOOLINFOA | TTM_GETTOOLINFOW => {
            return TOOLTIPS_GetToolInfoT(
                infoPtr,
                lParam as LPTTTOOLINFOW as *mut TTTOOLINFOW,
                (uMsg == TTM_GETTOOLINFOW) as ::core::ffi::c_int,
            );
        }
        TTM_HITTESTA | TTM_HITTESTW => {
            return TOOLTIPS_HitTestT(
                infoPtr,
                lParam as LPTTHITTESTINFOW,
                (uMsg == TTM_HITTESTW) as ::core::ffi::c_int,
            );
        }
        TTM_NEWTOOLRECTA | TTM_NEWTOOLRECTW => {
            return TOOLTIPS_NewToolRectT(infoPtr, lParam as LPTTTOOLINFOW as *const TTTOOLINFOW);
        }
        TTM_POP => {
            return TOOLTIPS_Pop(infoPtr);
        }
        TTM_RELAYEVENT => {
            return TOOLTIPS_RelayEvent(infoPtr, lParam as LPMSG);
        }
        TTM_SETDELAYTIME => {
            return TOOLTIPS_SetDelayTime(
                infoPtr,
                wParam as DWORD,
                LOWORD!(lParam) as INT,
            );
        }
        TTM_SETMARGIN => {
            return TOOLTIPS_SetMargin(infoPtr, lParam as LPRECT as *const RECT);
        }
        TTM_SETMAXTIPWIDTH => {
            return TOOLTIPS_SetMaxTipWidth(infoPtr, lParam as INT);
        }
        TTM_SETTIPBKCOLOR => {
            return TOOLTIPS_SetTipBkColor(infoPtr, wParam as COLORREF);
        }
        TTM_SETTIPTEXTCOLOR => {
            return TOOLTIPS_SetTipTextColor(infoPtr, wParam as COLORREF);
        }
        TTM_SETTITLEA | TTM_SETTITLEW => {
            return TOOLTIPS_SetTitleT(
                infoPtr,
                wParam as UINT_PTR,
                lParam as LPCWSTR,
                (uMsg == TTM_SETTITLEW) as ::core::ffi::c_int,
            );
        }
        TTM_SETTOOLINFOA | TTM_SETTOOLINFOW => {
            return TOOLTIPS_SetToolInfoT(
                infoPtr,
                lParam as LPTTTOOLINFOW as *const TTTOOLINFOW,
                (uMsg == TTM_SETTOOLINFOW) as ::core::ffi::c_int,
            );
        }
        TTM_TRACKACTIVATE => {
            return TOOLTIPS_TrackActivate(
                infoPtr,
                wParam as BOOL,
                lParam as LPTTTOOLINFOA as *const TTTOOLINFOA,
            );
        }
        TTM_TRACKPOSITION => {
            return TOOLTIPS_TrackPosition(infoPtr, lParam);
        }
        TTM_UPDATE => {
            return TOOLTIPS_Update(infoPtr);
        }
        TTM_UPDATETIPTEXTA | TTM_UPDATETIPTEXTW => {
            return TOOLTIPS_UpdateTipTextT(
                infoPtr,
                lParam as LPTTTOOLINFOW as *const TTTOOLINFOW,
                (uMsg == TTM_UPDATETIPTEXTW) as ::core::ffi::c_int,
            );
        }
        TTM_WINDOWFROMPOINT => {
            return WindowFromPoint(*(lParam as LPPOINT)) as LRESULT;
        }
        WM_CREATE => {
            return TOOLTIPS_Create(hwnd);
        }
        WM_DESTROY => {
            return TOOLTIPS_Destroy(infoPtr);
        }
        WM_ERASEBKGND => {
            /* we draw the background in WM_PAINT */
            return 0 as LRESULT;
        }
        WM_GETFONT => {
            return TOOLTIPS_GetFont(infoPtr);
        }
        WM_GETTEXT => {
            return TOOLTIPS_OnWMGetText(infoPtr, wParam, lParam as LPWSTR);
        }
        WM_GETTEXTLENGTH => {
            return TOOLTIPS_GetTextLength(infoPtr);
        }
        WM_LBUTTONDOWN
        | WM_LBUTTONUP
        | WM_MBUTTONDOWN
        | WM_MBUTTONUP
        | WM_RBUTTONDOWN
        | WM_RBUTTONUP
        | WM_MOUSEMOVE => {
            return TOOLTIPS_MouseMessage(infoPtr);
        }
        WM_NCCREATE => {
            return TOOLTIPS_NCCreate(hwnd);
        }
        WM_NCHITTEST => {
            return TOOLTIPS_NCHitTest(infoPtr, wParam, lParam);
        }
        WM_NOTIFYFORMAT => {
            return TOOLTIPS_NotifyFormat(infoPtr, wParam, lParam);
        }
        WM_PRINTCLIENT | WM_PAINT => {
            return TOOLTIPS_Paint(infoPtr, wParam as HDC);
        }
        WM_SETFONT => {
            return TOOLTIPS_SetFont(infoPtr, wParam as HFONT, LOWORD!(lParam) as BOOL);
        }
        WM_SYSCOLORCHANGE => {
            COMCTL32_RefreshSysColors();
            return 0 as LRESULT;
        }
        WM_TIMER => {
            return TOOLTIPS_Timer(infoPtr, wParam as INT);
        }
        WM_WININICHANGE => {
            return TOOLTIPS_WinIniChange(infoPtr);
        }
        _ => {
            if uMsg >= WM_USER
                && uMsg < WM_APP
                && COMCTL32_IsReflectedMessage(uMsg) == 0
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
                        b"dll/win32/comctl32/tooltips.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"TOOLTIPS_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
                        2368 as ::core::ffi::c_int,
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
    };
}
#[no_mangle]
pub unsafe extern "C" fn TOOLTIPS_Register() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    ZeroMemory!(
        core::ptr::addr_of_mut!(wndClass) as *mut core::ffi::c_void,
        core::mem::size_of::<WNDCLASSW>() as usize,
    );
    wndClass.style = (CS_GLOBALCLASS | CS_DBLCLKS | CS_SAVEBITS) as UINT;
    wndClass.lpfnWndProc =
        Some(TOOLTIPS_WindowProc);
    wndClass.cbClsExtra = 0 as core::ffi::c_int;
    wndClass.cbWndExtra =
        core::mem::size_of::<*mut TOOLTIPS_INFO>() as core::ffi::c_int;
    wndClass.hCursor = LoadCursorW(
        core::ptr::null_mut(),
        IDC_ARROW as LPWSTR as LPCWSTR,
    );
    wndClass.hbrBackground = core::ptr::null_mut();
    wndClass.lpszClassName = (TOOLTIPS_CLASSW).as_ptr();

    RegisterClassW(core::ptr::addr_of_mut!(wndClass));

    hTooltipIcons[TTI_NONE as usize] = core::ptr::null_mut();
    hTooltipIcons[TTI_INFO as usize] = LoadImageW(
        COMCTL32_hModule,
        MAKEINTRESOURCE!(IDI_TT_INFO_SM) as LPCWSTR,
        IMAGE_ICON,
        0,
        0,
        0,
    ) as HICON;
    hTooltipIcons[TTI_WARNING as usize] = LoadImageW(
        COMCTL32_hModule,
        MAKEINTRESOURCE!(IDI_TT_WARN_SM) as LPCWSTR,
        IMAGE_ICON,
        0,
        0,
        0,
    ) as HICON;
    hTooltipIcons[TTI_ERROR as usize] = LoadImageW(
        COMCTL32_hModule,
        MAKEINTRESOURCE!(IDI_TT_ERROR_SM) as LPCWSTR,
        IMAGE_ICON,
        0,
        0,
        0,
    ) as HICON;
}
#[no_mangle]
pub unsafe extern "C" fn TOOLTIPS_Unregister() {
    let mut i: ::core::ffi::c_int = 0;
    i = (TTI_INFO as i32);
    while i <= (TTI_ERROR as i32) {
        DestroyIcon(hTooltipIcons[i as usize]);
        i += 1;
    }
    UnregisterClassW((TOOLTIPS_CLASSW).as_ptr(), core::ptr::null_mut());
}
