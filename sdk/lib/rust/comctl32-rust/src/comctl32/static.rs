use crate::comctl32::commctrl::COMCTL32_RefreshSysColors;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_static);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const WC_STATICW: [::core::ffi::c_ushort; 7] = unsafe {
    ::core::mem::transmute::<[u8; 14], [::core::ffi::c_ushort; 7]>(*b"S\0t\0a\0t\0i\0c\0\0\0")
};

static mut staticPaintFunc: [pfPaint; 32] = unsafe {
    [
        Some(STATIC_PaintTextfn as unsafe extern "C" fn(HWND, HDC, DWORD) -> ()),
        Some(STATIC_PaintTextfn as unsafe extern "C" fn(HWND, HDC, DWORD) -> ()),
        Some(STATIC_PaintTextfn as unsafe extern "C" fn(HWND, HDC, DWORD) -> ()),
        Some(STATIC_PaintIconfn as unsafe extern "C" fn(HWND, HDC, DWORD) -> ()),
        Some(STATIC_PaintRectfn as unsafe extern "C" fn(HWND, HDC, DWORD) -> ()),
        Some(STATIC_PaintRectfn as unsafe extern "C" fn(HWND, HDC, DWORD) -> ()),
        Some(STATIC_PaintRectfn as unsafe extern "C" fn(HWND, HDC, DWORD) -> ()),
        Some(STATIC_PaintRectfn as unsafe extern "C" fn(HWND, HDC, DWORD) -> ()),
        Some(STATIC_PaintRectfn as unsafe extern "C" fn(HWND, HDC, DWORD) -> ()),
        Some(STATIC_PaintRectfn as unsafe extern "C" fn(HWND, HDC, DWORD) -> ()),
        None,
        Some(STATIC_PaintTextfn as unsafe extern "C" fn(HWND, HDC, DWORD) -> ()),
        Some(STATIC_PaintTextfn as unsafe extern "C" fn(HWND, HDC, DWORD) -> ()),
        Some(STATIC_PaintOwnerDrawfn as unsafe extern "C" fn(HWND, HDC, DWORD) -> ()),
        Some(STATIC_PaintBitmapfn as unsafe extern "C" fn(HWND, HDC, DWORD) -> ()),
        Some(STATIC_PaintEnhMetafn as unsafe extern "C" fn(HWND, HDC, DWORD) -> ()),
        Some(STATIC_PaintEtchedfn as unsafe extern "C" fn(HWND, HDC, DWORD) -> ()),
        Some(STATIC_PaintEtchedfn as unsafe extern "C" fn(HWND, HDC, DWORD) -> ()),
        Some(STATIC_PaintEtchedfn as unsafe extern "C" fn(HWND, HDC, DWORD) -> ()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ]
};


pub const HFONT_GWL_OFFSET: usize = 0;

pub const HICON_GWL_OFFSET: usize = core::mem::size_of::<HFONT>();

pub const STATIC_EXTRA_BYTES: usize = HICON_GWL_OFFSET + core::mem::size_of::<HICON>();

pub type pfPaint = Option<unsafe extern "C" fn(hwnd: HWND, hdc: HDC, style: DWORD)>;

unsafe extern "C" fn get_icon_size(handle: HICON, size: *mut SIZE) -> BOOL {
    let mut info: ICONINFO = core::mem::zeroed();
    let mut bmp: BITMAP = core::mem::zeroed();
    let mut ret: ::core::ffi::c_int;
    const REACTOS: bool = true;

    if GetIconInfo(handle, &raw mut info) == 0 {
        return FALSE;
    }

    ret = if REACTOS {
        GetObjectW(
            info.hbmMask as HANDLE,
            core::mem::size_of::<BITMAP>() as ::core::ffi::c_int,
            &raw mut bmp as LPVOID,
        )
    } else {
        GetObjectW(
            info.hbmColor as HANDLE,
            core::mem::size_of::<BITMAP>() as ::core::ffi::c_int,
            &raw mut bmp as LPVOID,
        )
    };
    if ret != 0 {
        (*size).cx = bmp.bmWidth;
        (*size).cy = bmp.bmHeight;
        /*
            If this structure defines a black and white icon, this bitmask is formatted
            so that the upper half is the icon AND bitmask and the lower half is
            the icon XOR bitmask.
        */
        if info.hbmColor.is_null() {
            (*size).cy /= 2;
        }
    }

    DeleteObject(info.hbmMask as HGDIOBJ);
    DeleteObject(info.hbmColor as HGDIOBJ);

    return (ret != 0) as BOOL;
}

unsafe extern "C" fn STATIC_SetIcon(hwnd: HWND, hicon: HICON, style: DWORD) -> HICON {
    let mut prevIcon: HICON = core::ptr::null_mut();
    let mut size: SIZE = core::mem::zeroed();

    if (style & SS_TYPEMASK) != SS_ICON {
        return core::ptr::null_mut();
    }
    if !hicon.is_null() && get_icon_size(hicon, core::ptr::addr_of_mut!(size)) == 0 {
        return core::ptr::null_mut();
    }
    prevIcon = SetWindowLongPtrW(hwnd, (HICON_GWL_OFFSET) as INT, hicon as LONG_PTR) as HICON;
    if !hicon.is_null() && (style & SS_CENTERIMAGE) == 0 && (style & SS_REALSIZECONTROL) == 0 {
        /* Windows currently doesn't implement SS_RIGHTJUST */
        {
            SetWindowPos(
                hwnd,
                core::ptr::null_mut(),
                0,
                0,
                size.cx,
                size.cy,
                SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOZORDER,
            );
        }
    }
    return prevIcon;
}

unsafe extern "C" fn STATIC_SetBitmap(mut hwnd: HWND, mut hBitmap: HBITMAP, mut style: DWORD) -> HBITMAP {
    let mut hOldBitmap: HBITMAP = core::ptr::null_mut();
    if (style & SS_TYPEMASK) != SS_BITMAP {
        return core::ptr::null_mut();
    }
    if !hBitmap.is_null() && GetObjectType(hBitmap as HGDIOBJ) != (OBJ_BITMAP) as DWORD {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int))
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/static.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"STATIC_SetBitmap\0" as *const u8 as *const ::core::ffi::c_char,
                166 as ::core::ffi::c_int,
                b"hBitmap != 0, but it's not a bitmap\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        return core::ptr::null_mut();
    }
    hOldBitmap = SetWindowLongPtrW(
        hwnd,
        (HICON_GWL_OFFSET) as ::core::ffi::c_int,
        hBitmap as LONG_PTR,
    ) as HBITMAP;
    if !hBitmap.is_null()
        && (style & SS_CENTERIMAGE) == 0
        && (style & SS_REALSIZECONTROL) == 0
    {
        let mut bm: BITMAP = core::mem::zeroed();
        GetObjectW(
            hBitmap as HANDLE,
            core::mem::size_of::<BITMAP>() as ::core::ffi::c_int,
            core::ptr::addr_of_mut!(bm) as LPVOID,
        );
        /* Windows currently doesn't implement SS_RIGHTJUST */
        /*
        if ((style & SS_RIGHTJUST) != 0)
        {
            RECT wr;
            GetWindowRect(hwnd, &wr);
            SetWindowPos( hwnd, 0, wr.right - bm.bmWidth, wr.bottom - bm.bmHeight,
                          bm.bmWidth, bm.bmHeight, SWP_NOACTIVATE | SWP_NOZORDER );
        }
        else */
        {
            SetWindowPos(
                hwnd,
                core::ptr::null_mut(),
                (0) as ::core::ffi::c_int,
                (0) as ::core::ffi::c_int,
                bm.bmWidth as ::core::ffi::c_int,
                bm.bmHeight as ::core::ffi::c_int,
                (SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOZORDER) as UINT,
            );
        }
    }
    return hOldBitmap;
}

unsafe extern "C" fn STATIC_SetEnhMetaFile(
    mut hwnd: HWND,
    mut hEnhMetaFile: HENHMETAFILE,
    mut style: DWORD,
) -> HENHMETAFILE {
    if (style & SS_TYPEMASK) != SS_ENHMETAFILE {
        return core::ptr::null_mut();
    }
    if !hEnhMetaFile.is_null()
        && GetObjectType(hEnhMetaFile as HGDIOBJ) as DWORD != OBJ_ENHMETAFILE as DWORD
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
                b"dll/win32/comctl32/static.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"STATIC_SetEnhMetaFile\0" as *const u8 as *const ::core::ffi::c_char,
                202 as ::core::ffi::c_int,
                b"hEnhMetaFile != 0, but it's not an enhanced metafile\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        return core::ptr::null_mut();
    }
    return SetWindowLongPtrW(
        hwnd,
        HICON_GWL_OFFSET as ::core::ffi::c_int,
        hEnhMetaFile as LONG_PTR,
    ) as HENHMETAFILE;
}

unsafe extern "C" fn STATIC_GetImage(hwnd: HWND, wParam: WPARAM, style: DWORD) -> HANDLE {
    match style & SS_TYPEMASK {
        SS_ICON => {
            if (wParam != IMAGE_ICON as WPARAM) && (wParam != IMAGE_CURSOR as WPARAM) {
                return core::ptr::null_mut();
            }
        }
        SS_BITMAP => {
            if wParam != IMAGE_BITMAP as WPARAM {
                return core::ptr::null_mut();
            }
        }
        SS_ENHMETAFILE => {
            if wParam != IMAGE_ENHMETAFILE as WPARAM {
                return core::ptr::null_mut();
            }
        }
        _ => {
            return core::ptr::null_mut();
        }
    }
    return GetWindowLongPtrW(hwnd, HICON_GWL_OFFSET as ::core::ffi::c_int) as HANDLE;
}

unsafe extern "C" fn STATIC_LoadIconW(hInstance: HINSTANCE, name: LPCWSTR, style: DWORD) -> HICON {
    let mut hicon: HICON = core::ptr::null_mut();

    if !hInstance.is_null() && ((hInstance as ULONG_PTR) >> 16) != 0 {
        if (style & SS_REALSIZEIMAGE) != 0 {
            hicon = LoadImageW(hInstance, name, IMAGE_ICON, 0, 0, LR_SHARED) as HICON;
        } else {
            hicon = LoadIconW(hInstance, name) as HICON;
            if hicon.is_null() {
                hicon = LoadCursorW(hInstance, name) as HICON;
            }
        }
    }
    if hicon.is_null() {
        hicon = LoadIconW(core::ptr::null_mut(), name) as HICON;
    }
    /* Windows doesn't try to load a standard cursor,
       probably because most IDs for standard cursors conflict
       with the IDs for standard icons anyway */
    return hicon;
}

unsafe extern "C" fn STATIC_TryPaintFcn(mut hwnd: HWND, mut full_style: LONG) {
    let mut style: LONG = full_style & (SS_TYPEMASK) as i32;
    let mut rc: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    GetClientRect(hwnd, core::ptr::addr_of_mut!(rc));
    if IsRectEmpty(core::ptr::addr_of_mut!(rc) as LPCRECT) == 0
        && IsWindowVisible(hwnd) != 0
        && staticPaintFunc[style as usize].is_some()
    {
        let mut hdc: HDC = core::ptr::null_mut();
        let mut hrgn: HRGN = core::ptr::null_mut();
        hdc = GetDC(hwnd);
        hrgn = set_control_clipping(hdc, core::ptr::addr_of_mut!(rc));
        (staticPaintFunc[style as usize].unwrap_unchecked())(
            hwnd,
            hdc,
            full_style as DWORD,
        );
        SelectClipRgn(hdc, hrgn);
        if !hrgn.is_null() {
            DeleteObject(hrgn as HGDIOBJ);
        }
        ReleaseDC(hwnd, hdc);
    }
}

unsafe extern "C" fn STATIC_SendWmCtlColorStatic(hwnd: HWND, hdc: HDC) -> HBRUSH {
    let mut hBrush: HBRUSH = core::ptr::null_mut();
    let mut parent: HWND = GetParent(hwnd);

    if parent.is_null() {
        parent = hwnd;
    }
    hBrush = SendMessageW(
        parent,
        WM_CTLCOLORSTATIC as UINT,
        hdc as WPARAM,
        hwnd as LPARAM,
    ) as HBRUSH;
    if hBrush.is_null() {
        hBrush = DefWindowProcW(
            parent,
            WM_CTLCOLORSTATIC as UINT,
            hdc as WPARAM,
            hwnd as LPARAM,
        ) as HBRUSH;
    }
    return hBrush;
}

unsafe extern "C" fn hasTextStyle(style: DWORD) -> BOOL {
    match style & SS_TYPEMASK {
        SS_SIMPLE
        | SS_LEFT
        | SS_LEFTNOWORDWRAP
        | SS_CENTER
        | SS_RIGHT
        | SS_OWNERDRAW => {
            return TRUE;
        }
        _ => {}
    }

    return FALSE;
}

unsafe extern "system" fn STATIC_WindowProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut lResult: LRESULT = 0 as LRESULT;
    let mut full_style: LONG = GetWindowLongW(hwnd, GWL_STYLE);
    let mut style: LONG = full_style & (SS_TYPEMASK) as i32;

    if IsWindow(hwnd) == 0 {
        return 0 as LRESULT;
    }

    match uMsg {
        WM_CREATE => {
            if style < 0 || (style) as u32 > SS_TYPEMASK {
                if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/win32/comctl32/static.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"STATIC_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
                        335 as ::core::ffi::c_int,
                        b"Unknown style 0x%02x\n\0" as *const u8 as *const ::core::ffi::c_char,
                        style,
                    );
                }
                return -1 as LRESULT;
            }
        }

        WM_NCDESTROY => {
            if (style) as u32 == SS_ICON {
                /* FIXME: DestroyIcon32( STATIC_SetIcon( wndPtr, 0 ) ); is intentionally not done */
            } else {
                return DefWindowProcW(hwnd, uMsg, wParam, lParam);
            }
        }

        WM_ERASEBKGND => {
            /* do all painting in WM_PAINT like Windows does */
            return 1 as LRESULT;
        }

        WM_PRINTCLIENT | WM_PAINT => {
            let mut ps: PAINTSTRUCT = core::mem::zeroed();
            let mut rect: RECT = core::mem::zeroed();
            let mut hdc: HDC = if wParam != 0 {
                wParam as HDC
            } else {
                BeginPaint(hwnd, core::ptr::addr_of_mut!(ps))
            };
            GetClientRect(hwnd, core::ptr::addr_of_mut!(rect));
            if staticPaintFunc[style as usize].is_some() {
                let mut hrgn: HRGN = set_control_clipping(hdc, core::ptr::addr_of_mut!(rect));
                staticPaintFunc[style as usize].unwrap_unchecked()(
                    hwnd,
                    hdc,
                    full_style as DWORD,
                );
                SelectClipRgn(hdc, hrgn);
                if !hrgn.is_null() {
                    DeleteObject(hrgn as HGDIOBJ);
                }
            }
            if wParam == 0 {
                EndPaint(hwnd, core::ptr::addr_of_mut!(ps));
            }
        }

        WM_ENABLE => {
            STATIC_TryPaintFcn(hwnd, full_style);
            if (full_style & (SS_NOTIFY as LONG)) != 0 {
                if wParam != 0 {
                    SendMessageW(
                        GetParent(hwnd),
                        WM_COMMAND,
                        MAKEWPARAM!(
                            GetWindowLongPtrW(hwnd, GWLP_ID) as WORD,
                            STN_ENABLE as WORD,
                        ),
                        hwnd as LPARAM,
                    );
                } else {
                    SendMessageW(
                        GetParent(hwnd),
                        WM_COMMAND,
                        MAKEWPARAM!(
                            GetWindowLongPtrW(hwnd, GWLP_ID) as WORD,
                            STN_DISABLE as WORD,
                        ),
                        hwnd as LPARAM,
                    );
                }
            }
        }

        WM_SYSCOLORCHANGE => {
            COMCTL32_RefreshSysColors();
            STATIC_TryPaintFcn(hwnd, full_style);
        }

        WM_NCCREATE => {
            let mut cs: *mut CREATESTRUCTW = lParam as *mut CREATESTRUCTW;

            if (full_style & (SS_SUNKEN as LONG)) != 0 {
                SetWindowLongW(
                    hwnd,
                    GWL_EXSTYLE,
                    GetWindowLongW(hwnd, GWL_EXSTYLE) | (WS_EX_STATICEDGE) as i32,
                );
            }

            match (style) as u32 {
                SS_ICON => {
                    let mut hIcon: HICON = core::ptr::null_mut();

                    hIcon = STATIC_LoadIconW(
                        (*cs).hInstance,
                        (*cs).lpszName,
                        full_style as DWORD,
                    );
                    STATIC_SetIcon(hwnd, hIcon, full_style as DWORD);
                }
                SS_BITMAP => {
                    if (((*cs).hInstance as ULONG_PTR) >> 16) != 0 {
                        let mut hBitmap: HBITMAP = core::ptr::null_mut();
                        hBitmap = LoadBitmapW((*cs).hInstance, (*cs).lpszName);
                        STATIC_SetBitmap(hwnd, hBitmap, full_style as DWORD);
                    }
                }
                _ => {}
            }
            /* SS_ENHMETAFILE: Despite what MSDN says, Windows does not load
               the enhanced metafile that was specified as the window text. */
            return DefWindowProcW(hwnd, uMsg, wParam, lParam);
        }

        WM_SETTEXT => {
            if hasTextStyle(full_style as DWORD) != 0 {
                lResult = DefWindowProcW(hwnd, uMsg, wParam, lParam);
                STATIC_TryPaintFcn(hwnd, full_style);
            }
        }

        WM_SETFONT => {
            if hasTextStyle(full_style as DWORD) != 0 {
                SetWindowLongPtrW(hwnd, (HFONT_GWL_OFFSET) as INT, wParam as LONG_PTR);
                if LOWORD!(lParam) != 0 {
                    RedrawWindow(
                        hwnd,
                        core::ptr::null_mut(),
                        0 as HRGN,
                        (RDW_INVALIDATE
                            | RDW_ERASE
                            | RDW_UPDATENOW
                            | RDW_ALLCHILDREN) as UINT,
                    );
                }
            }
        }

        WM_GETFONT => {
            return GetWindowLongPtrW(hwnd, (HFONT_GWL_OFFSET) as INT) as LRESULT;
        }

        WM_NCHITTEST => {
            if (full_style & (SS_NOTIFY as LONG)) != 0 {
                return HTCLIENT as LRESULT;
            } else {
                return HTTRANSPARENT as LRESULT;
            }
        }

        WM_GETDLGCODE => {
            return DLGC_STATIC as LRESULT;
        }

        WM_LBUTTONDOWN | WM_NCLBUTTONDOWN => {
            if (full_style & (SS_NOTIFY as LONG)) != 0 {
                SendMessageW(
                    GetParent(hwnd),
                    WM_COMMAND,
                    MAKEWPARAM!(
                        GetWindowLongPtrW(hwnd, GWLP_ID) as WORD,
                        STN_CLICKED as WORD,
                    ),
                    hwnd as LPARAM,
                );
            }
            return 0 as LRESULT;
        }

        WM_LBUTTONDBLCLK | WM_NCLBUTTONDBLCLK => {
            if (full_style & (SS_NOTIFY as LONG)) != 0 {
                SendMessageW(
                    GetParent(hwnd),
                    WM_COMMAND,
                    MAKEWPARAM!(
                        GetWindowLongPtrW(hwnd, GWLP_ID) as WORD,
                        STN_DBLCLK as WORD,
                    ),
                    hwnd as LPARAM,
                );
            }
            return 0 as LRESULT;
        }

        STM_GETIMAGE => {
            return STATIC_GetImage(hwnd, wParam, full_style as DWORD) as LRESULT;
        }

        STM_GETICON => {
            return STATIC_GetImage(hwnd, (IMAGE_ICON) as WPARAM, full_style as DWORD)
                as LRESULT;
        }

        STM_SETIMAGE => {
            match wParam as UINT {
                IMAGE_BITMAP => {
                    lResult = STATIC_SetBitmap(hwnd, lParam as HBITMAP, full_style as DWORD)
                        as LRESULT;
                }
                IMAGE_ENHMETAFILE => {
                    lResult = STATIC_SetEnhMetaFile(
                        hwnd,
                        lParam as HENHMETAFILE,
                        full_style as DWORD,
                    ) as LRESULT;
                }
                IMAGE_ICON | IMAGE_CURSOR => {
                    lResult = STATIC_SetIcon(hwnd, lParam as HICON, full_style as DWORD)
                        as LRESULT;
                }
                _ => {
                    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int)
                            << (__WINE_DBCL_FIXME as ::core::ffi::c_int)))
                        != 0
                    {
                        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
                        ros_dbg_log(
                            __dbcl_0,
                            __dbch_0,
                            b"dll/win32/comctl32/static.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"STATIC_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
                            491 as ::core::ffi::c_int,
                            b"STM_SETIMAGE: Unhandled type %lx\n\0" as *const u8
                                as *const ::core::ffi::c_char,
                            wParam,
                        );
                    }
                }
            }
            STATIC_TryPaintFcn(hwnd, full_style);
        }

        STM_SETICON => {
            lResult = STATIC_SetIcon(hwnd, wParam as HICON, full_style as DWORD) as LRESULT;
            STATIC_TryPaintFcn(hwnd, full_style);
        }

        _ => {
            return DefWindowProcW(hwnd, uMsg, wParam, lParam);
        }
    }
    return lResult;
}

unsafe extern "C" fn STATIC_PaintOwnerDrawfn(hwnd: HWND, hdc: HDC, style: DWORD) {
    let mut dis: DRAWITEMSTRUCT = core::mem::zeroed();
    let mut font: HFONT = core::ptr::null_mut();
    let mut oldFont: HFONT = core::ptr::null_mut();
    let id: UINT = GetWindowLongPtrW(hwnd, GWLP_ID) as UINT;

    dis.CtlType = ODT_STATIC as UINT;
    dis.CtlID = id;
    dis.itemID = 0 as UINT;
    dis.itemAction = ODA_DRAWENTIRE as UINT;
    dis.itemState = (if IsWindowEnabled(hwnd) != 0 {
        0 as ::core::ffi::c_int
    } else {
        ODS_DISABLED as ::core::ffi::c_int
    }) as UINT;
    dis.hwndItem = hwnd;
    dis.hDC = hdc;
    dis.itemData = 0 as ULONG_PTR;
    GetClientRect(hwnd, core::ptr::addr_of_mut!(dis.rcItem));

    font = GetWindowLongPtrW(hwnd, (HFONT_GWL_OFFSET) as INT) as HFONT;
    if !font.is_null() {
        oldFont = SelectObject(hdc, font as HGDIOBJ) as HFONT;
    }
    SendMessageW(
        GetParent(hwnd),
        WM_CTLCOLORSTATIC as UINT,
        hdc as WPARAM,
        hwnd as LPARAM,
    );
    SendMessageW(
        GetParent(hwnd),
        WM_DRAWITEM as UINT,
        id as WPARAM,
        core::ptr::addr_of_mut!(dis) as LPARAM,
    );
    if !font.is_null() {
        SelectObject(hdc, oldFont as HGDIOBJ);
    }
}

unsafe extern "system" fn STATIC_DrawTextCallback(hdc: HDC, lp: LPARAM, wp: WPARAM, cx: i32, cy: i32) -> BOOL {
    let mut rc: RECT = core::mem::zeroed();
    SetRect(core::ptr::addr_of_mut!(rc), 0, 0, cx, cy);
    DrawTextW(hdc, lp as LPCWSTR, -1, core::ptr::addr_of_mut!(rc), wp as UINT);
    return TRUE;
}

unsafe extern "C" fn STATIC_PaintTextfn(hwnd: HWND, hdc: HDC, style: DWORD) {
    let mut rc: RECT = core::mem::zeroed();
    let mut hBrush: HBRUSH = core::ptr::null_mut();
    let mut hFont: HFONT = core::ptr::null_mut();
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut format: UINT = 0;
    let mut len: INT = 0;
    let mut buf_size: INT = 0;
    let mut text: *mut WCHAR = core::ptr::null_mut();

    GetClientRect(hwnd, core::ptr::addr_of_mut!(rc));

    match style as core::ffi::c_ulong & SS_TYPEMASK as core::ffi::c_ulong {
        SS_LEFT => {
            format = (DT_LEFT | DT_EXPANDTABS | DT_WORDBREAK) as UINT;
        }
        SS_CENTER => {
            format = (DT_CENTER | DT_EXPANDTABS | DT_WORDBREAK) as UINT;
        }
        SS_RIGHT => {
            format = (DT_RIGHT | DT_EXPANDTABS | DT_WORDBREAK) as UINT;
        }
        SS_SIMPLE => {
            format = (DT_LEFT | DT_SINGLELINE) as UINT;
        }
        SS_LEFTNOWORDWRAP => {
            format = (DT_LEFT | DT_EXPANDTABS) as UINT;
        }
        _ => return,
    }

    if GetWindowLongW(hwnd, GWL_EXSTYLE) as core::ffi::c_int
        & WS_EX_RIGHT as core::ffi::c_int
        != 0
    {
        format = DT_RIGHT as UINT
            | (format & !(DT_LEFT | DT_CENTER) as UINT);
    }

    if style & SS_NOPREFIX as DWORD != 0 {
        format |= DT_NOPREFIX as UINT;
    }

    if style as core::ffi::c_ulong & SS_TYPEMASK as core::ffi::c_ulong
        != SS_SIMPLE as core::ffi::c_ulong
    {
        if style & SS_CENTERIMAGE as DWORD != 0 {
            format |= (DT_SINGLELINE | DT_VCENTER) as UINT;
        }
        if style & SS_EDITCONTROL as DWORD != 0 {
            format |= DT_EDITCONTROL as UINT;
        }
        if style & SS_ENDELLIPSIS as DWORD != 0 {
            format |= (DT_SINGLELINE | DT_END_ELLIPSIS) as UINT;
        }
        if style & SS_PATHELLIPSIS as DWORD != 0 {
            format |= (DT_SINGLELINE | DT_PATH_ELLIPSIS) as UINT;
        }
        if style & SS_WORDELLIPSIS as DWORD != 0 {
            format |= (DT_SINGLELINE | DT_WORD_ELLIPSIS) as UINT;
        }
    }

    hFont = GetWindowLongPtrW(hwnd, (HFONT_GWL_OFFSET) as INT) as HFONT;
    if !hFont.is_null() {
        hOldFont = SelectObject(hdc, hFont as HGDIOBJ) as HFONT;
    }

    /* SS_SIMPLE controls: WM_CTLCOLORSTATIC is sent, but the returned
                           brush is not used */
    hBrush = STATIC_SendWmCtlColorStatic(hwnd, hdc);

    if style as core::ffi::c_ulong & SS_TYPEMASK as core::ffi::c_ulong
        != SS_SIMPLE as core::ffi::c_ulong
    {
        FillRect(hdc, core::ptr::addr_of_mut!(rc), hBrush);
        if IsWindowEnabled(hwnd) == 0 {
            SetTextColor(
                hdc,
                GetSysColor((COLOR_GRAYTEXT) as i32) as COLORREF,
            );
        }
    }

    buf_size = 256 as INT;
    'no_TextOut: {
        text = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (buf_size as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as *mut WCHAR;
        if text.is_null() {
            break 'no_TextOut;
        }

        loop {
            len = InternalGetWindowText(hwnd, text as LPWSTR, buf_size);
            if !(len == buf_size - 1 as INT) {
                break;
            }
            buf_size *= 2 as INT;
            text = HeapReAlloc(
                GetProcessHeap(),
                0 as DWORD,
                text as PVOID,
                (buf_size as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as *mut WCHAR;
            if text.is_null() {
                break 'no_TextOut;
            }
        }

        if len == 0 {
            break 'no_TextOut;
        }

        if (style as core::ffi::c_ulong & SS_TYPEMASK as core::ffi::c_ulong
            == SS_SIMPLE as core::ffi::c_ulong)
            && (style & SS_NOPREFIX as DWORD != 0)
        {
            /* Windows uses the faster ExtTextOut() to draw the text and
               to paint the whole client rectangle with the text background
               color. Reference: "Static Controls" by Kyle Marsh, 1992 */
            ExtTextOutW(
                hdc,
                rc.left as core::ffi::c_int,
                rc.top as core::ffi::c_int,
                (ETO_CLIPPED | ETO_OPAQUE) as UINT,
                core::ptr::addr_of_mut!(rc),
                text as LPCWSTR,
                len as UINT,
                core::ptr::null::<INT>(),
            );
        } else {
            let mut flags: UINT = DST_COMPLEX as UINT;
            if style & WS_DISABLED as DWORD != 0 {
                flags |= DSS_DISABLED as UINT;
            }
            DrawStateW(
                hdc,
                hBrush,
                Some(
                    STATIC_DrawTextCallback,
                ),
                text as LPARAM,
                format as WPARAM,
                rc.left as core::ffi::c_int,
                rc.top as core::ffi::c_int,
                rc.right as core::ffi::c_int - rc.left as core::ffi::c_int,
                rc.bottom as core::ffi::c_int - rc.top as core::ffi::c_int,
                flags,
            );
        }
    }

    HeapFree(GetProcessHeap(), 0 as DWORD, text as PVOID);

    if !hFont.is_null() {
        SelectObject(hdc, hOldFont as HGDIOBJ);
    }
}

unsafe extern "C" fn STATIC_PaintRectfn(hwnd: HWND, hdc: HDC, style: DWORD) {
    let mut rc: RECT = core::mem::zeroed();
    let mut hBrush: HBRUSH = core::ptr::null_mut();

    GetClientRect( hwnd, core::ptr::addr_of_mut!(rc) as LPRECT);

    /* FIXME: send WM_CTLCOLORSTATIC */
    match (style & SS_TYPEMASK) {
        SS_BLACKRECT => {
            hBrush = CreateSolidBrush(comctl32_color.clr3dDkShadow);
            FillRect( hdc, core::ptr::addr_of_mut!(rc) as LPCRECT, hBrush );
        }
        SS_GRAYRECT => {
            hBrush = CreateSolidBrush(comctl32_color.clr3dShadow);
            FillRect( hdc, core::ptr::addr_of_mut!(rc) as LPCRECT, hBrush );
        }
        SS_WHITERECT => {
            hBrush = CreateSolidBrush(comctl32_color.clr3dHilight);
            FillRect( hdc, core::ptr::addr_of_mut!(rc) as LPCRECT, hBrush );
        }
        SS_BLACKFRAME => {
            hBrush = CreateSolidBrush(comctl32_color.clr3dDkShadow);
            FrameRect( hdc, core::ptr::addr_of_mut!(rc) as LPCRECT, hBrush );
        }
        SS_GRAYFRAME => {
            hBrush = CreateSolidBrush(comctl32_color.clr3dShadow);
            FrameRect( hdc, core::ptr::addr_of_mut!(rc) as LPCRECT, hBrush );
        }
        SS_WHITEFRAME => {
            hBrush = CreateSolidBrush(comctl32_color.clr3dHilight);
            FrameRect( hdc, core::ptr::addr_of_mut!(rc) as LPCRECT, hBrush );
        }
        _ => return,
    }
    DeleteObject( hBrush as HGDIOBJ );
}

unsafe extern "C" fn STATIC_PaintIconfn(hwnd: HWND, hdc: HDC, style: DWORD) {
    let mut rc: RECT = core::mem::zeroed();
    let mut iconRect: RECT = core::mem::zeroed();
    let mut hbrush: HBRUSH = core::ptr::null_mut();
    let mut hIcon: HICON = core::ptr::null_mut();
    let mut size: SIZE = core::mem::zeroed();

    GetClientRect(hwnd, core::ptr::addr_of_mut!(rc));
    hbrush = STATIC_SendWmCtlColorStatic(hwnd, hdc);
    hIcon = GetWindowLongPtrW(hwnd, HICON_GWL_OFFSET as ::core::ffi::c_int) as HICON;
    if hIcon.is_null() || get_icon_size(hIcon, core::ptr::addr_of_mut!(size)) == 0 {
        FillRect(hdc, core::ptr::addr_of_mut!(rc), hbrush);
    } else {
        if (style & SS_CENTERIMAGE as DWORD) != 0 {
            iconRect.left = (rc.right - rc.left) / 2 - size.cx / 2;
            iconRect.top = (rc.bottom - rc.top) / 2 - size.cy / 2;
            iconRect.right = iconRect.left + size.cx;
            iconRect.bottom = iconRect.top + size.cy;
        } else {
            iconRect = rc;
        }
        FillRect(hdc, core::ptr::addr_of_mut!(rc), hbrush);
        DrawIconEx(
            hdc,
            iconRect.left as ::core::ffi::c_int,
            iconRect.top as ::core::ffi::c_int,
            hIcon,
            (iconRect.right - iconRect.left) as ::core::ffi::c_int,
            (iconRect.bottom - iconRect.top) as ::core::ffi::c_int,
            0 as UINT,
            core::ptr::null_mut(),
            DI_NORMAL as UINT,
        );
    }
}

unsafe extern "C" fn STATIC_PaintBitmapfn(hwnd: HWND, hdc: HDC, style: DWORD) {
    let mut hMemDC: HDC = core::ptr::null_mut();
    let mut hBitmap: HBITMAP = core::ptr::null_mut();
    let mut oldbitmap: HBITMAP = core::ptr::null_mut();
    let mut hbrush: HBRUSH = core::ptr::null_mut();

    hbrush = STATIC_SendWmCtlColorStatic(hwnd, hdc);

    hBitmap = GetWindowLongPtrW(hwnd, (HICON_GWL_OFFSET) as INT) as HBITMAP;
    if !hBitmap.is_null()
        && GetObjectType(hBitmap as HGDIOBJ) == OBJ_BITMAP as DWORD
        && {
            hMemDC = CreateCompatibleDC(hdc);
            !hMemDC.is_null()
        }
    {
        let mut bm: BITMAP = core::mem::zeroed();
        let mut rcClient: RECT = core::mem::zeroed();
        let mut brush: LOGBRUSH = core::mem::zeroed();

        GetObjectW(
            hBitmap as HANDLE,
            core::mem::size_of::<BITMAP>() as core::ffi::c_int,
            core::ptr::addr_of_mut!(bm) as LPVOID,
        );
        oldbitmap = SelectObject(hMemDC, hBitmap as HGDIOBJ) as HBITMAP;

        /* Set the background color for monochrome bitmaps
           to the color of the background brush */
        if GetObjectW(
            hbrush as HANDLE,
            core::mem::size_of::<LOGBRUSH>() as core::ffi::c_int,
            core::ptr::addr_of_mut!(brush) as LPVOID,
        ) != 0
        {
            if brush.lbStyle == BS_SOLID as UINT {
                SetBkColor(hdc, brush.lbColor);
            }
        }
        GetClientRect(hwnd, core::ptr::addr_of_mut!(rcClient));
        if (style & SS_CENTERIMAGE as DWORD) != 0 {
            FillRect(hdc, core::ptr::addr_of_mut!(rcClient) as LPCRECT, hbrush);
            rcClient.left = (rcClient.right - rcClient.left) / 2
                - bm.bmWidth / 2;
            rcClient.top = (rcClient.bottom - rcClient.top) / 2
                - bm.bmHeight / 2;
            rcClient.right = rcClient.left + bm.bmWidth;
            rcClient.bottom = rcClient.top + bm.bmHeight;
        }
        StretchBlt(
            hdc,
            rcClient.left,
            rcClient.top,
            rcClient.right - rcClient.left,
            rcClient.bottom - rcClient.top,
            hMemDC,
            0,
            0,
            bm.bmWidth,
            bm.bmHeight,
            SRCCOPY as DWORD,
        );
        SelectObject(hMemDC, oldbitmap as HGDIOBJ);
        DeleteDC(hMemDC);
    }
}

unsafe extern "C" fn STATIC_PaintEnhMetafn(hwnd: HWND, hdc: HDC, style: DWORD) {
    let mut hEnhMetaFile: HENHMETAFILE = core::ptr::null_mut();
    let mut rc: RECT = core::mem::zeroed();
    let mut hbrush: HBRUSH = core::ptr::null_mut();

    GetClientRect(hwnd, core::ptr::addr_of_mut!(rc));
    hbrush = STATIC_SendWmCtlColorStatic(hwnd, hdc);
    FillRect(hdc, core::ptr::addr_of_mut!(rc) as LPCRECT, hbrush);
    hEnhMetaFile = GetWindowLongPtrW(hwnd, (HICON_GWL_OFFSET) as core::ffi::c_int) as HENHMETAFILE;
    if !hEnhMetaFile.is_null() {
        /* The control's current font is not selected into the
           device context! */
        if GetObjectType(hEnhMetaFile as HGDIOBJ) == (OBJ_ENHMETAFILE) as DWORD {
            PlayEnhMetaFile(hdc, hEnhMetaFile, core::ptr::addr_of_mut!(rc) as LPCRECT);
        }
    }
}

unsafe extern "C" fn STATIC_PaintEtchedfn(hwnd: HWND, hdc: HDC, style: DWORD) {
    let mut rc: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    GetClientRect(hwnd, core::ptr::addr_of_mut!(rc));
    let Masked: DWORD = style & SS_TYPEMASK;
    if Masked == SS_ETCHEDHORZ {
        DrawEdge(
            hdc,
            core::ptr::addr_of_mut!(rc),
            EDGE_ETCHED,
            BF_TOP | BF_BOTTOM,
        );
    } else if Masked == SS_ETCHEDVERT {
        DrawEdge(
            hdc,
            core::ptr::addr_of_mut!(rc),
            EDGE_ETCHED,
            BF_LEFT | BF_RIGHT,
        );
    } else if Masked == SS_ETCHEDFRAME {
        DrawEdge(
            hdc,
            core::ptr::addr_of_mut!(rc),
            EDGE_ETCHED,
            BF_RECT,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn STATIC_Register() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    memset(
        core::ptr::addr_of_mut!(wndClass) as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        core::mem::size_of::<WNDCLASSW>() as size_t,
    );
    wndClass.style = (CS_DBLCLKS | CS_PARENTDC | CS_GLOBALCLASS) as UINT;
    wndClass.lpfnWndProc = Some(STATIC_WindowProc);
    wndClass.cbClsExtra = 0 as ::core::ffi::c_int;
    wndClass.cbWndExtra = STATIC_EXTRA_BYTES as ::core::ffi::c_int;
    wndClass.hCursor = LoadCursorW(
        core::ptr::null_mut(),
        (IDC_ARROW as usize) as LPWSTR,
    );
    wndClass.hbrBackground = core::ptr::null_mut();
    wndClass.lpszClassName = (WC_STATICW).as_ptr();
    RegisterClassW(core::ptr::addr_of_mut!(wndClass));
}
#[no_mangle]
pub unsafe extern "C" fn STATIC_Unregister() {
    UnregisterClassW((WC_STATICW).as_ptr(), core::ptr::null_mut());
}
