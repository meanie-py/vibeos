use win32k_sys::win32k::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::winbase::ExtTextOutW;
use crate::windows::window::HeapAlloc;
use crate::windows::message::HeapReAlloc;
// The crate's own functions: the definition is the declaration.
use crate::windows::draw::DrawStateW;
use crate::windows::font::DrawTextW;
use crate::windows::class::GetWindowLongPtrW;
use crate::windows::rect::IsRectEmpty;
use crate::windows::cursoricon::LoadCursorW;
use core::ffi::c_void;

// WARN/ERR/FIXME/TRACE are wine debug-channel macros (no-op here, matching the
// other integrated user32-rust bodies — window.rs/message.rs).

// LOWORD / MAKEWPARAM are macros — use crate::internal::*. ValidateHwnd lives in
// crate::internal (correctly fastcall-split on i386).
use crate::internal::{ValidateHwnd, LOWORD, MAKEWPARAM};
// SetWindowLongPtrW is a winuser.h macro for SetWindowLongW on x86; class.rs only
// defines a real (isize-typed) SetWindowLongPtrW for non-x86. cfg-split.
#[cfg(not(target_arch = "x86"))]
#[inline]
unsafe fn SetWindowLongPtrW(hwnd: HWND, index: i32, newLong: LONG_PTR) -> LONG_PTR {
    crate::windows::class::SetWindowLongPtrW(hwnd, index, newLong as isize) as LONG_PTR
}
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn SetWindowLongPtrW(hwnd: HWND, index: i32, newLong: LONG_PTR) -> LONG_PTR {
    crate::windows::class::SetWindowLongW(hwnd, index, newLong as core::ffi::c_long) as LONG_PTR
}

// IS_INTRESOURCE — wine/winuser macro: a pointer with the high bits clear
// (value < 0x10000) is an int-resource. Returns BOOL to match the C `!= 0` sites.
#[inline]
unsafe fn IS_INTRESOURCE(p: *const c_void) -> BOOL {
    ((p as usize) >> 16 == 0) as BOOL
}

extern "system" {
    // --- gdi32 / user32 / cohab imports (resolve at link) ---
    fn SetRect(lprc: *mut RECT, xLeft: core::ffi::c_int, yTop: core::ffi::c_int, xRight: core::ffi::c_int, yBottom: core::ffi::c_int) -> i32;
    fn GetClientRect(hwnd: HWND, rc: *mut RECT) -> i32;
    fn GetWindowLongW(hwnd: HWND, index: i32) -> i32;
    fn SetWindowLongW(hwnd: HWND, index: i32, newLong: LONG) -> LONG;
    fn SelectObject(hdc: HDC, h: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn FillRect(hdc: HDC, rc: *const RECT, hbr: HBRUSH) -> i32;
    fn FrameRect(hdc: HDC, rc: *const RECT, hbr: HBRUSH) -> i32;
    fn DrawEdge(hdc: HDC, qrc: *mut RECT, edge: UINT, grfFlags: UINT) -> BOOL;
    fn IsWindowEnabled(hwnd: HWND) -> i32;
    fn IsWindow(hwnd: HWND) -> BOOL;
    fn IsWindowVisible(hwnd: HWND) -> BOOL;
    fn GetParent(hwnd: HWND) -> HWND;
    fn ReleaseDC(hwnd: HWND, hdc: HDC) -> INT;
    fn SelectClipRgn(hdc: HDC, hrgn: HRGN) -> i32;
    fn DefWindowProcW(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn DefWindowProcA(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn SendMessageW(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn LoadIconA(hInstance: HINSTANCE, name: LPCSTR) -> HICON;
    fn LoadIconW(hInstance: HINSTANCE, name: LPCWSTR) -> HICON;
    fn LoadCursorA(hInstance: HINSTANCE, name: LPCSTR) -> HCURSOR;
    fn LoadImageA(hInstance: HINSTANCE, name: LPCSTR, type_: UINT, cx: i32, cy: i32, flags: UINT) -> HANDLE;
    fn LoadImageW(hInstance: HINSTANCE, name: LPCWSTR, type_: UINT, cx: i32, cy: i32, flags: UINT) -> HANDLE;
    fn LoadBitmapA(hInstance: HINSTANCE, name: LPCSTR) -> HBITMAP;
    fn LoadBitmapW(hInstance: HINSTANCE, name: LPCWSTR) -> HBITMAP;
    fn SetTextColor(hdc: HDC, color: COLORREF) -> COLORREF;
    fn GetSysColor(index: i32) -> COLORREF;
    fn GetProcessHeap() -> *mut core::ffi::c_void;
    fn HeapFree(heap: *mut core::ffi::c_void, flags: u32, mem: *mut core::ffi::c_void) -> i32;
    fn InternalGetWindowText(hwnd: HWND, text: *mut u16, max: i32) -> i32;
    fn DrawIconEx(hdc: HDC, x: i32, y: i32, hIcon: HICON, cx: i32, cy: i32, istep: u32, flickerFreeDraw: HBRUSH, flags: u32) -> i32;
    fn CreateCompatibleDC(hdc: HDC) -> HDC;
    fn CreateSolidBrush(color: COLORREF) -> HBRUSH;
    fn DeleteDC(hdc: HDC) -> i32;
    fn DeleteObject(ho: *mut core::ffi::c_void) -> i32;
    fn GetObjectType(h: *mut core::ffi::c_void) -> DWORD;
    fn GetObjectW(h: *mut core::ffi::c_void, c: i32, pv: *mut core::ffi::c_void) -> i32;
    fn GetPixel(hdc: HDC, x: i32, y: i32) -> COLORREF;
    fn StretchBlt( hdcDest: HDC, xDest: i32, yDest: i32, wDest: i32, hDest: i32, hdcSrc: HDC, xSrc: i32, ySrc: i32, wSrc: i32, hSrc: i32, rop: DWORD, ) -> i32;
    fn PlayEnhMetaFile(hdc: HDC, hmf: HENHMETAFILE, lprect: *const RECT) -> i32;
}

// cdecl C helpers in user32 (controls/uitools.c) — plain `_name`, no @N decoration.
extern "C" {
    fn set_control_clipping(hdc: HDC, rect: *const RECT) -> HRGN;
    fn get_icon_size(hIcon: HICON, size: *mut SIZE) -> i32;
}

/// `STATIC_update_uistate()` — user_x.h `static __inline` (no exported symbol),
/// reproduced faithfully. STATIC_UISTATE_GWL_OFFSET == sizeof(HFONT)+sizeof(HICON).
#[inline]
unsafe fn STATIC_update_uistate(hwnd: HWND, unicode: BOOL) -> BOOL {
    let flags: LONG;
    let prevflags: LONG;

    if unicode != 0 {
        flags = DefWindowProcW(hwnd, WM_QUERYUISTATE, 0, 0) as LONG;
    } else {
        flags = DefWindowProcA(hwnd, WM_QUERYUISTATE, 0, 0) as LONG;
    }

    prevflags = GetWindowLongW(hwnd, UISTATE_GWL_OFFSET as i32);

    if prevflags != flags {
        SetWindowLongW(hwnd, UISTATE_GWL_OFFSET as i32, flags);
        return 1;
    }

    0
}

// ---- file-scope preamble (translated) ----
pub static mut color_3dshadow: COLORREF = 0;
pub static mut color_3ddkshadow: COLORREF = 0;
pub static mut color_3dhighlight: COLORREF = 0;

pub const HFONT_GWL_OFFSET: usize = 0;
pub const HICON_GWL_OFFSET: usize = core::mem::size_of::<HFONT>();
pub const UISTATE_GWL_OFFSET: usize = HICON_GWL_OFFSET + core::mem::size_of::<HICON>();
pub const STATIC_EXTRA_BYTES: usize = UISTATE_GWL_OFFSET + core::mem::size_of::<LONG>();

pub type pfPaint = Option<unsafe extern "system" fn(hwnd: HWND, hdc: HDC, style: DWORD)>;

pub static staticPaintFunc: [pfPaint; (SS_TYPEMASK + 1) as usize] =
[
    Some(STATIC_PaintTextfn),      /* SS_LEFT */
    Some(STATIC_PaintTextfn),      /* SS_CENTER */
    Some(STATIC_PaintTextfn),      /* SS_RIGHT */
    Some(STATIC_PaintIconfn),      /* SS_ICON */
    Some(STATIC_PaintRectfn),      /* SS_BLACKRECT */
    Some(STATIC_PaintRectfn),      /* SS_GRAYRECT */
    Some(STATIC_PaintRectfn),      /* SS_WHITERECT */
    Some(STATIC_PaintRectfn),      /* SS_BLACKFRAME */
    Some(STATIC_PaintRectfn),      /* SS_GRAYFRAME */
    Some(STATIC_PaintRectfn),      /* SS_WHITEFRAME */
    None,                          /* SS_USERITEM */
    Some(STATIC_PaintTextfn),      /* SS_SIMPLE */
    Some(STATIC_PaintTextfn),      /* SS_LEFTNOWORDWRAP */
    Some(STATIC_PaintOwnerDrawfn), /* SS_OWNERDRAW */
    Some(STATIC_PaintBitmapfn),    /* SS_BITMAP */
    Some(STATIC_PaintEnhMetafn),   /* SS_ENHMETAFILE */
    Some(STATIC_PaintEtchedfn),    /* SS_ETCHEDHORZ */
    Some(STATIC_PaintEtchedfn),    /* SS_ETCHEDVERT */
    Some(STATIC_PaintEtchedfn),    /* SS_ETCHEDFRAME */
    // C declares the table [SS_TYPEMASK+1] (== 32); indices 19..=31 are
    // implicit NULL in the C aggregate initializer. Pad to match the
    // declared length so `staticPaintFunc[style]` (style up to SS_TYPEMASK)
    // stays in-bounds.
    None, None, None, None, None, None, None, None, None, None, None, None, None,
];

pub static staticW: [u16; 7] = [b'S' as u16, b't' as u16, b'a' as u16, b't' as u16, b'i' as u16, b'c' as u16, 0];

#[repr(C)]
pub struct builtin_class_descr {
    pub name: LPCWSTR,
    pub style: UINT,
    pub procA: WNDPROC,
    pub procW: WNDPROC,
    pub extra: INT,
    pub cursor: LPCWSTR,
    pub brush: HBRUSH,
}

// winuser.h `IDC_ARROW` == MAKEINTRESOURCEW(32512). Not in win32k-sys; reproduced
// locally (matching defwnd.rs's local MAKEINTRESOURCEW pattern).
const IDC_ARROW: LPCWSTR = 32512usize as LPCWSTR;

// `STATIC_builtin_class` — referenced cross-TU by the surviving C
// controls/regcontrol.c (`&STATIC_builtin_class`), so it must exist with that
// exact (unmangled) symbol name. Faithful to the C definition.
unsafe impl Sync for builtin_class_descr {}

#[no_mangle]
pub static STATIC_builtin_class: builtin_class_descr = builtin_class_descr {
    name: staticW.as_ptr(),
    style: CS_DBLCLKS | CS_PARENTDC,
    procA: unsafe {
        core::mem::transmute::<
            unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT,
            WNDPROC,
        >(StaticWndProcA)
    },
    procW: unsafe {
        core::mem::transmute::<
            unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT,
            WNDPROC,
        >(StaticWndProcW)
    },
    extra: STATIC_EXTRA_BYTES as INT,
    cursor: IDC_ARROW,
    brush: 0 as HBRUSH,
};

unsafe fn STATIC_SetIcon(hwnd: HWND, hicon: HICON, style: DWORD) -> HICON {
    let mut size: SIZE = core::mem::zeroed();

    if (style & SS_TYPEMASK) != SS_ICON {
        return core::ptr::null_mut();
    }
    if !hicon.is_null() && get_icon_size(hicon, &mut size) == 0 {
        WARN!(b"hicon != 0, but invalid\n\0".as_ptr() as *const i8);
        return core::ptr::null_mut();
    }
    let prevIcon = SetWindowLongPtrW(hwnd, HICON_GWL_OFFSET as i32, hicon as LONG_PTR) as HICON;
    if !hicon.is_null() && (style & SS_CENTERIMAGE) == 0 && (style & SS_REALSIZECONTROL) == 0 {
        crate::ntwrapper::SetWindowPos(
            hwnd,
            core::ptr::null_mut(),
            0,
            0,
            size.cx,
            size.cy,
            SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOZORDER,
        );
    }
    prevIcon
}

unsafe fn STATIC_SetBitmap(hwnd: HWND, hBitmap: HBITMAP, style: DWORD) -> HBITMAP {
    let hOldBitmap: HBITMAP;

    if (style & SS_TYPEMASK) != SS_BITMAP {
        return core::ptr::null_mut();
    }
    if !hBitmap.is_null() && GetObjectType(hBitmap as *mut core::ffi::c_void) != OBJ_BITMAP as DWORD {
        return core::ptr::null_mut();
    }
    hOldBitmap = SetWindowLongPtrW(
        hwnd,
        core::mem::size_of::<HFONT>() as core::ffi::c_int,
        hBitmap as LONG_PTR,
    ) as HBITMAP;
    if !hBitmap.is_null()
        && (style & SS_CENTERIMAGE) == 0
        && (style & SS_REALSIZECONTROL) == 0
    {
        let mut bm: BITMAP = core::mem::zeroed();
        GetObjectW(
            hBitmap as *mut core::ffi::c_void,
            core::mem::size_of::<BITMAP>() as core::ffi::c_int,
            core::ptr::addr_of_mut!(bm) as *mut core::ffi::c_void,
        );
        crate::ntwrapper::SetWindowPos(
            hwnd,
            core::ptr::null_mut(),
            0,
            0,
            bm.bmWidth,
            bm.bmHeight,
            SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOZORDER,
        );
    }
    hOldBitmap
}

unsafe fn STATIC_SetEnhMetaFile(hwnd: HWND, hEnhMetaFile: HENHMETAFILE, style: DWORD) -> HENHMETAFILE {
    if (style & SS_TYPEMASK) != SS_ENHMETAFILE {
        return core::ptr::null_mut();
    }
    if !hEnhMetaFile.is_null() && GetObjectType(hEnhMetaFile as *mut _) != OBJ_ENHMETAFILE {
        WARN!(b"hEnhMetaFile != 0, but it's not an enhanced metafile\n\0".as_ptr() as LPCSTR);
        return core::ptr::null_mut();
    }
    SetWindowLongPtrW(hwnd, core::mem::size_of::<HFONT>() as i32, hEnhMetaFile as LONG_PTR) as HENHMETAFILE
}

unsafe fn STATIC_GetImage(hwnd: HWND, wParam: WPARAM, style: DWORD) -> HANDLE {
    match style & SS_TYPEMASK {
        x if x == SS_ICON => {
            if (wParam != IMAGE_ICON as WPARAM) && (wParam != IMAGE_CURSOR as WPARAM) {
                return core::ptr::null_mut();
            }
        }
        x if x == SS_BITMAP => {
            if wParam != IMAGE_BITMAP as WPARAM {
                return core::ptr::null_mut();
            }
        }
        x if x == SS_ENHMETAFILE => {
            if wParam != IMAGE_ENHMETAFILE as WPARAM {
                return core::ptr::null_mut();
            }
        }
        _ => return core::ptr::null_mut(),
    }
    GetWindowLongPtrW(hwnd, core::mem::size_of::<HFONT>() as i32) as HANDLE
}

unsafe fn STATIC_LoadIconA(hInstance: HINSTANCE, name: LPCSTR, style: DWORD) -> HICON {
    let mut hicon: HICON = core::ptr::null_mut();

    if !hInstance.is_null() && ((hInstance as ULONG_PTR) >> 16) != 0 {
        if (style & SS_REALSIZEIMAGE) != 0 {
            hicon = LoadImageA(hInstance, name, IMAGE_ICON, 0, 0, LR_SHARED) as HICON;
        } else {
            hicon = LoadIconA(hInstance, name);
            if hicon.is_null() {
                hicon = LoadCursorA(hInstance, name) as HICON;
            }
        }
    }
    if hicon.is_null() {
        hicon = LoadIconA(core::ptr::null_mut(), name);
    }
    hicon
}

unsafe fn STATIC_LoadIconW(hInstance: HINSTANCE, name: LPCWSTR, style: DWORD) -> HICON {
    let mut hicon: HICON = core::ptr::null_mut();

    if !hInstance.is_null() && ((hInstance as ULONG_PTR) >> 16) != 0 {
        if (style & SS_REALSIZEIMAGE) != 0 {
            hicon = LoadImageW(hInstance, name, IMAGE_ICON, 0, 0, LR_SHARED) as HICON;
        } else {
            hicon = LoadIconW(hInstance, name);
            if hicon.is_null() {
                hicon = LoadCursorW(hInstance, name) as HICON;
            }
        }
    }
    if hicon.is_null() {
        hicon = LoadIconW(core::ptr::null_mut(), name);
    }
    hicon
}

unsafe fn STATIC_TryPaintFcn(hwnd: HWND, full_style: LONG) {
    let style: LONG = full_style & SS_TYPEMASK as LONG;
    let mut rc: RECT = core::mem::zeroed();

    GetClientRect(hwnd, &mut rc);
    if IsRectEmpty(&mut rc) == 0
        && IsWindowVisible(hwnd) != 0
        && staticPaintFunc[style as usize].is_some()
    {
        let hdc: HDC = crate::ntwrapper::GetDC(hwnd);
        let hrgn: HRGN = set_control_clipping(hdc, &mut rc);
        (staticPaintFunc[style as usize].unwrap())(hwnd, hdc, full_style as DWORD);
        SelectClipRgn(hdc, hrgn);
        if !hrgn.is_null() {
            DeleteObject(hrgn as _);
        }
        ReleaseDC(hwnd, hdc);
    }
}

unsafe fn STATIC_SendWmCtlColorStatic(hwnd: HWND, hdc: HDC) -> HBRUSH {
    crate::ntwrapper::GetControlBrush(hwnd, hdc, WM_CTLCOLORSTATIC)
}

unsafe fn STATIC_InitColours() {
    color_3ddkshadow = GetSysColor(COLOR_3DDKSHADOW as i32);
    color_3dshadow = GetSysColor(COLOR_3DSHADOW as i32);
    color_3dhighlight = GetSysColor(COLOR_3DHIGHLIGHT as i32);
}

unsafe fn hasTextStyle(style: DWORD) -> BOOL {
    match style & SS_TYPEMASK {
        x if x == SS_SIMPLE
            || x == SS_LEFT
            || x == SS_LEFTNOWORDWRAP
            || x == SS_CENTER
            || x == SS_RIGHT
            || x == SS_OWNERDRAW =>
        {
            return 1;
        }
        _ => {}
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn StaticWndProc_common(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, unicode: BOOL) -> LRESULT {
    let mut lResult: LRESULT = 0;
    let full_style: LONG = GetWindowLongW(hwnd, GWL_STYLE);
    let style: LONG = full_style & SS_TYPEMASK as LONG;

    let pWnd: PWND = ValidateHwnd(hwnd);
    if !pWnd.is_null() {
        if (*pWnd).fnid == 0 {
            NtUserSetWindowFNID(hwnd, FNID_STATIC as u16);
        } else {
            if (*pWnd).fnid != FNID_STATIC {
                ERR!("Wrong window class for Static! fnId 0x%x\n", (*pWnd).fnid as u32);
                return 0;
            }
        }
    }

    if IsWindow(hwnd) == 0 {
        return 0;
    }

    match uMsg {
        WM_CREATE => {
            if style < 0 || style > SS_TYPEMASK as LONG {
                ERR!("Unknown style 0x%02lx\n", style);
                return -1;
            }
            STATIC_update_uistate(hwnd, unicode);
            STATIC_InitColours();
        }

        WM_NCDESTROY => {
            NtUserSetWindowFNID(hwnd, FNID_DESTROY as u16);
            if style == SS_ICON as LONG {
                // see FIXME comment
            } else {
                return if unicode != 0 {
                    DefWindowProcW(hwnd, uMsg, wParam, lParam)
                } else {
                    DefWindowProcA(hwnd, uMsg, wParam, lParam)
                };
            }
        }

        WM_ERASEBKGND => {
            return 1;
        }

        WM_PRINTCLIENT | WM_PAINT => {
            let mut ps: PAINTSTRUCT = core::mem::zeroed();
            let mut rect: RECT = core::mem::zeroed();
            let hdc: HDC = if wParam != 0 {
                wParam as HDC
            } else {
                crate::ntwrapper::BeginPaint(hwnd, &mut ps)
            };
            GetClientRect(hwnd, &mut rect);
            if staticPaintFunc[style as usize].is_some() {
                let hrgn: HRGN = set_control_clipping(hdc, &mut rect);
                (staticPaintFunc[style as usize].unwrap())(hwnd, hdc, full_style as DWORD);
                SelectClipRgn(hdc, hrgn);
                if !hrgn.is_null() {
                    DeleteObject(hrgn as HGDIOBJ);
                }
            }
            if wParam == 0 {
                crate::ntwrapper::EndPaint(hwnd, &ps);
            }
        }

        WM_ENABLE => {
            STATIC_TryPaintFcn(hwnd, full_style);
            if full_style & SS_NOTIFY as LONG != 0 {
                if wParam != 0 {
                    SendMessageW(GetParent(hwnd), WM_COMMAND,
                        MAKEWPARAM(GetWindowLongPtrW(hwnd, GWLP_ID) as u16, STN_ENABLE as u16) as WPARAM,
                        hwnd as LPARAM);
                } else {
                    SendMessageW(GetParent(hwnd), WM_COMMAND,
                        MAKEWPARAM(GetWindowLongPtrW(hwnd, GWLP_ID) as u16, STN_DISABLE as u16) as WPARAM,
                        hwnd as LPARAM);
                }
            }
        }

        WM_SYSCOLORCHANGE => {
            STATIC_InitColours();
            STATIC_TryPaintFcn(hwnd, full_style);
        }

        WM_NCCREATE => {
            let cs: *mut CREATESTRUCTW = lParam as *mut CREATESTRUCTW;

            if full_style & SS_SUNKEN as LONG != 0 {
                SetWindowLongW(hwnd, GWL_EXSTYLE,
                    GetWindowLongW(hwnd, GWL_EXSTYLE) | WS_EX_STATICEDGE as LONG);
            }

            match style as u32 {
                SS_ICON => {
                    let hIcon: HICON;
                    if unicode != 0 || IS_INTRESOURCE((*cs).lpszName as *const core::ffi::c_void) != 0 {
                        hIcon = STATIC_LoadIconW((*cs).hInstance, (*cs).lpszName, full_style as DWORD);
                    } else {
                        hIcon = STATIC_LoadIconA((*cs).hInstance, (*cs).lpszName as LPCSTR, full_style as DWORD);
                    }
                    STATIC_SetIcon(hwnd, hIcon, full_style as DWORD);
                }
                SS_BITMAP => {
                    if ((*cs).hInstance as ULONG_PTR) >> 16 != 0 {
                        let hBitmap: HBITMAP;
                        if unicode != 0 || IS_INTRESOURCE((*cs).lpszName as *const core::ffi::c_void) != 0 {
                            hBitmap = LoadBitmapW((*cs).hInstance, (*cs).lpszName);
                        } else {
                            hBitmap = LoadBitmapA((*cs).hInstance, (*cs).lpszName as LPCSTR);
                        }
                        STATIC_SetBitmap(hwnd, hBitmap, full_style as DWORD);
                    }
                }
                _ => {}
            }

            return if unicode != 0 {
                DefWindowProcW(hwnd, uMsg, wParam, lParam)
            } else {
                DefWindowProcA(hwnd, uMsg, wParam, lParam)
            };
        }

        WM_SETTEXT => {
            if hasTextStyle(full_style as DWORD) != 0 {
                if unicode != 0 {
                    lResult = DefWindowProcW(hwnd, uMsg, wParam, lParam);
                } else {
                    lResult = DefWindowProcA(hwnd, uMsg, wParam, lParam);
                }
                STATIC_TryPaintFcn(hwnd, full_style);
            }
        }

        WM_SETFONT => {
            if hasTextStyle(full_style as DWORD) != 0 {
                SetWindowLongPtrW(hwnd, HFONT_GWL_OFFSET as i32, wParam as LONG_PTR);
                if LOWORD(lParam as DWORD) != 0 {
                    crate::ntwrapper::RedrawWindow(hwnd, core::ptr::null_mut(), core::ptr::null_mut(),
                        (RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW | RDW_ALLCHILDREN) as UINT);
                }
            }
        }

        WM_GETFONT => {
            return GetWindowLongPtrW(hwnd, HFONT_GWL_OFFSET as i32) as LRESULT;
        }

        WM_NCHITTEST => {
            if full_style & SS_NOTIFY as LONG != 0 {
                return HTCLIENT as LRESULT;
            } else {
                return HTTRANSPARENT as LRESULT;
            }
        }

        WM_GETDLGCODE => {
            return DLGC_STATIC as LRESULT;
        }

        WM_LBUTTONDOWN | WM_NCLBUTTONDOWN => {
            if full_style & SS_NOTIFY as LONG != 0 {
                SendMessageW(GetParent(hwnd), WM_COMMAND,
                    MAKEWPARAM(GetWindowLongPtrW(hwnd, GWLP_ID) as u16, STN_CLICKED as u16) as WPARAM,
                    hwnd as LPARAM);
            }
            return 0;
        }

        WM_LBUTTONDBLCLK | WM_NCLBUTTONDBLCLK => {
            if full_style & SS_NOTIFY as LONG != 0 {
                SendMessageW(GetParent(hwnd), WM_COMMAND,
                    MAKEWPARAM(GetWindowLongPtrW(hwnd, GWLP_ID) as u16, STN_DBLCLK as u16) as WPARAM,
                    hwnd as LPARAM);
            }
            return 0;
        }

        STM_GETIMAGE => {
            return STATIC_GetImage(hwnd, wParam, full_style as DWORD) as LRESULT;
        }

        STM_GETICON => {
            return STATIC_GetImage(hwnd, IMAGE_ICON as WPARAM, full_style as DWORD) as LRESULT;
        }

        STM_SETIMAGE => {
            match wParam as u32 {
                IMAGE_BITMAP => {
                    if style != SS_BITMAP as LONG {
                        return 0;
                    }
                    lResult = STATIC_SetBitmap(hwnd, lParam as HBITMAP, full_style as DWORD) as LRESULT;
                }
                IMAGE_ENHMETAFILE => {
                    if style != SS_ENHMETAFILE as LONG {
                        return 0;
                    }
                    lResult = STATIC_SetEnhMetaFile(hwnd, lParam as HENHMETAFILE, full_style as DWORD) as LRESULT;
                }
                IMAGE_ICON | IMAGE_CURSOR => {
                    if style != SS_ICON as LONG {
                        return 0;
                    }
                    lResult = STATIC_SetIcon(hwnd, lParam as HICON, full_style as DWORD) as LRESULT;
                }
                _ => {
                    FIXME!("STM_SETIMAGE: Unhandled type %lx\n", wParam);
                }
            }
            STATIC_TryPaintFcn(hwnd, full_style);
        }

        STM_SETICON => {
            lResult = STATIC_SetIcon(hwnd, wParam as HICON, full_style as DWORD) as LRESULT;
            STATIC_TryPaintFcn(hwnd, full_style);
        }

        WM_UPDATEUISTATE => {
            if unicode != 0 {
                DefWindowProcW(hwnd, uMsg, wParam, lParam);
            } else {
                DefWindowProcA(hwnd, uMsg, wParam, lParam);
            }

            if STATIC_update_uistate(hwnd, unicode) != 0 && hasTextStyle(full_style as DWORD) != 0 {
                crate::ntwrapper::RedrawWindow(hwnd, core::ptr::null_mut(), core::ptr::null_mut(),
                    (RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW | RDW_ALLCHILDREN) as UINT);
            }
        }

        _ => {
            return if unicode != 0 {
                DefWindowProcW(hwnd, uMsg, wParam, lParam)
            } else {
                DefWindowProcA(hwnd, uMsg, wParam, lParam)
            };
        }
    }
    lResult
}

#[no_mangle]
pub unsafe extern "system" fn StaticWndProcA(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    if IsWindow(hWnd) == 0 {
        return 0;
    }
    StaticWndProc_common(hWnd, uMsg, wParam, lParam, 0)
}

#[no_mangle]
pub unsafe extern "system" fn StaticWndProcW(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    if IsWindow(hWnd) == 0 {
        return 0;
    }
    StaticWndProc_common(hWnd, uMsg, wParam, lParam, 1)
}

unsafe extern "system" fn STATIC_PaintOwnerDrawfn(hwnd: HWND, hdc: HDC, _style: DWORD) {
    let mut dis: DRAWITEMSTRUCT = core::mem::zeroed();
    let mut oldFont: HFONT = core::ptr::null_mut();
    let id = GetWindowLongPtrW(hwnd, GWLP_ID) as UINT;

    dis.CtlType = ODT_STATIC as UINT;
    dis.CtlID = id;
    dis.itemID = 0;
    dis.itemAction = ODA_DRAWENTIRE;
    dis.itemState = if IsWindowEnabled(hwnd) != 0 { 0 } else { ODS_DISABLED };
    dis.hwndItem = hwnd;
    dis.hDC = hdc;
    dis.itemData = 0;
    GetClientRect(hwnd, core::ptr::addr_of_mut!(dis.rcItem) as *mut RECT);

    let font = GetWindowLongPtrW(hwnd, HFONT_GWL_OFFSET as i32) as HFONT;
    if !font.is_null() {
        oldFont = SelectObject(hdc, font as *mut core::ffi::c_void) as HFONT;
    }
    STATIC_SendWmCtlColorStatic(hwnd, hdc);
    SendMessageW(GetParent(hwnd), WM_DRAWITEM, id as WPARAM, core::ptr::addr_of_mut!(dis) as LPARAM);
    if !font.is_null() {
        SelectObject(hdc, oldFont as *mut core::ffi::c_void);
    }
}

unsafe extern "system" fn STATIC_DrawTextCallback(hdc: HDC, lp: LPARAM, wp: WPARAM, cx: core::ffi::c_int, cy: core::ffi::c_int) -> i32 {
    let mut rc: RECT = core::mem::zeroed();

    SetRect(&mut rc, 0, 0, cx, cy);
    DrawTextW(hdc, lp as LPCWSTR, -1, &mut rc, wp as u32);
    1
}

unsafe extern "system" fn STATIC_PaintTextfn(hwnd: HWND, hdc: HDC, style: DWORD) {

    let mut rc: RECT = core::mem::zeroed();
    let hBrush: HBRUSH;
    let mut hFont: HFONT = core::ptr::null_mut();
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut format: UINT;
    let mut len: INT;
    let mut buf_size: INT;
    let mut text: *mut WCHAR = core::ptr::null_mut();

    GetClientRect(hwnd, &mut rc);

    match style & SS_TYPEMASK {
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
        _ => {
            return;
        }
    }

    if GetWindowLongW(hwnd, GWL_EXSTYLE) & WS_EX_RIGHT as i32 != 0 {
        format = DT_RIGHT as UINT | (format & !((DT_LEFT | DT_CENTER) as UINT));
    }

    if style & SS_NOPREFIX != 0 {
        format |= DT_NOPREFIX as UINT;
    } else if GetWindowLongW(hwnd, (core::mem::size_of::<HFONT>() + core::mem::size_of::<HICON>()) as i32) & UISF_HIDEACCEL as i32 != 0 {
        format |= DT_HIDEPREFIX as UINT;
    }

    if style & SS_TYPEMASK != SS_SIMPLE {
        if style & SS_CENTERIMAGE != 0 {
            format |= (DT_SINGLELINE | DT_VCENTER) as UINT;
        }
        if style & SS_EDITCONTROL != 0 {
            format |= DT_EDITCONTROL as UINT;
        }
        if style & SS_ENDELLIPSIS != 0 {
            format |= (DT_SINGLELINE | DT_END_ELLIPSIS) as UINT;
        }
        if style & SS_PATHELLIPSIS != 0 {
            format |= (DT_SINGLELINE | DT_PATH_ELLIPSIS) as UINT;
        }
        if style & SS_WORDELLIPSIS != 0 {
            format |= (DT_SINGLELINE | DT_WORD_ELLIPSIS) as UINT;
        }
    }

    hFont = GetWindowLongPtrW(hwnd, 0) as HFONT;
    if !hFont.is_null() {
        hOldFont = SelectObject(hdc, hFont as *mut core::ffi::c_void) as HFONT;
    }

    hBrush = STATIC_SendWmCtlColorStatic(hwnd, hdc);

    if style & SS_TYPEMASK != SS_SIMPLE {
        FillRect(hdc, &rc, hBrush);
        if IsWindowEnabled(hwnd) == 0 {
            SetTextColor(hdc, GetSysColor(COLOR_GRAYTEXT as i32));
        }
    }

    'no_TextOut: {
        buf_size = 256;
        text = HeapAlloc(GetProcessHeap(), 0, (buf_size as usize * core::mem::size_of::<WCHAR>()) as SIZE_T) as *mut WCHAR;
        if text.is_null() {
            break 'no_TextOut;
        }

        loop {
            len = InternalGetWindowText(hwnd, text, buf_size);
            if len != buf_size - 1 {
                break;
            }
            buf_size *= 2;
            text = HeapReAlloc(GetProcessHeap(), 0, text as *mut core::ffi::c_void, (buf_size as usize * core::mem::size_of::<WCHAR>()) as SIZE_T) as *mut WCHAR;
            if text.is_null() {
                break 'no_TextOut;
            }
        }

        if len == 0 {
            break 'no_TextOut;
        }

        if (style & SS_TYPEMASK) == SS_SIMPLE && style & SS_NOPREFIX != 0 {
            ExtTextOutW(hdc, rc.left, rc.top, ETO_CLIPPED | ETO_OPAQUE, &rc, text, (len) as UINT, core::ptr::null());
        } else {
            let mut flags: UINT = DST_COMPLEX as UINT;
            if style & WS_DISABLED != 0 {
                flags |= DSS_DISABLED as UINT;
            }
            DrawStateW(hdc, hBrush, Some(STATIC_DrawTextCallback),
                       text as LPARAM, format as WPARAM,
                       rc.left, rc.top,
                       rc.right - rc.left, rc.bottom - rc.top,
                       flags);
        }
    }

    HeapFree(GetProcessHeap(), 0, text as *mut core::ffi::c_void);

    if !hFont.is_null() {
        SelectObject(hdc, hOldFont as *mut core::ffi::c_void);
    }
}

unsafe extern "system" fn STATIC_PaintRectfn(hwnd: HWND, hdc: HDC, style: DWORD) {
    let mut rc: RECT = core::mem::zeroed();
    let hBrush: HBRUSH;

    GetClientRect(hwnd, &mut rc);

    STATIC_SendWmCtlColorStatic(hwnd, hdc);

    match style & SS_TYPEMASK {
        SS_BLACKRECT => {
            hBrush = CreateSolidBrush(color_3ddkshadow);
            FillRect(hdc, &mut rc, hBrush);
        }
        SS_GRAYRECT => {
            hBrush = CreateSolidBrush(color_3dshadow);
            FillRect(hdc, &mut rc, hBrush);
        }
        SS_WHITERECT => {
            hBrush = CreateSolidBrush(color_3dhighlight);
            FillRect(hdc, &mut rc, hBrush);
        }
        SS_BLACKFRAME => {
            hBrush = CreateSolidBrush(color_3ddkshadow);
            FrameRect(hdc, &mut rc, hBrush);
        }
        SS_GRAYFRAME => {
            hBrush = CreateSolidBrush(color_3dshadow);
            FrameRect(hdc, &mut rc, hBrush);
        }
        SS_WHITEFRAME => {
            hBrush = CreateSolidBrush(color_3dhighlight);
            FrameRect(hdc, &mut rc, hBrush);
        }
        _ => {
            return;
        }
    }
    DeleteObject(hBrush as _);
}

unsafe extern "system" fn STATIC_PaintIconfn(hwnd: HWND, hdc: HDC, style: DWORD) {

    let mut rc: RECT = core::mem::zeroed();
    let mut iconRect: RECT = core::mem::zeroed();
    let hbrush: HBRUSH;
    let hIcon: HICON;
    let mut size: SIZE = core::mem::zeroed();

    GetClientRect(hwnd, &mut rc);
    hbrush = STATIC_SendWmCtlColorStatic(hwnd, hdc);
    hIcon = GetWindowLongPtrW(hwnd, HICON_GWL_OFFSET as i32) as HICON;
    if hIcon.is_null() || get_icon_size(hIcon, &mut size) == 0 {
        FillRect(hdc, &rc, hbrush);
    } else {
        if (style & SS_CENTERIMAGE) != 0 {
            iconRect.left = (rc.right - rc.left) / 2 - size.cx / 2;
            iconRect.top = (rc.bottom - rc.top) / 2 - size.cy / 2;
            iconRect.right = iconRect.left + size.cx;
            iconRect.bottom = iconRect.top + size.cy;
        } else {
            iconRect = rc;
        }
        FillRect(hdc, &rc, hbrush);
        DrawIconEx(hdc, iconRect.left, iconRect.top, hIcon, iconRect.right - iconRect.left,
                   iconRect.bottom - iconRect.top, 0, core::ptr::null_mut(), DI_NORMAL);
    }
}

unsafe extern "system" fn STATIC_PaintBitmapfn(hwnd: HWND, hdc: HDC, style: DWORD) {

    let hMemDC: HDC;
    let hBitmap: HBITMAP;
    let oldbitmap: *mut core::ffi::c_void;

    STATIC_SendWmCtlColorStatic(hwnd, hdc);

    hBitmap = GetWindowLongPtrW(hwnd, core::mem::size_of::<HFONT>() as i32) as HBITMAP;
    if !hBitmap.is_null()
        && GetObjectType(hBitmap as *mut core::ffi::c_void) == OBJ_BITMAP as DWORD
        && {
            hMemDC = CreateCompatibleDC(hdc);
            !hMemDC.is_null()
        }
    {
        let mut bm: BITMAP = core::mem::zeroed();
        let mut rcClient: RECT = core::mem::zeroed();

        GetObjectW(
            hBitmap as *mut core::ffi::c_void,
            core::mem::size_of::<BITMAP>() as i32,
            core::ptr::addr_of_mut!(bm) as *mut core::ffi::c_void,
        );
        oldbitmap = SelectObject(hMemDC, hBitmap as *mut core::ffi::c_void);

        GetClientRect(hwnd, &mut rcClient);
        if style & SS_CENTERIMAGE != 0 {
            let hbrush = CreateSolidBrush(GetPixel(hMemDC, 0, 0));

            FillRect(hdc, &rcClient, hbrush);

            rcClient.left = (rcClient.right - rcClient.left) / 2 - bm.bmWidth / 2;
            rcClient.top = (rcClient.bottom - rcClient.top) / 2 - bm.bmHeight / 2;
            rcClient.right = rcClient.left + bm.bmWidth;
            rcClient.bottom = rcClient.top + bm.bmHeight;

            DeleteObject(hbrush as *mut core::ffi::c_void);
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
        SelectObject(hMemDC, oldbitmap);
        DeleteDC(hMemDC);
    }
}

unsafe extern "system" fn STATIC_PaintEnhMetafn(hwnd: HWND, hdc: HDC, _style: DWORD) {
    let mut rc: RECT = core::mem::zeroed();
    let hbrush: HBRUSH;

    GetClientRect(hwnd, &mut rc);
    hbrush = STATIC_SendWmCtlColorStatic(hwnd, hdc);
    FillRect(hdc, &mut rc, hbrush);
    let hEnhMetaFile = GetWindowLongPtrW(hwnd, HICON_GWL_OFFSET as i32) as HENHMETAFILE;
    if !hEnhMetaFile.is_null() {
        if GetObjectType(hEnhMetaFile as _) == OBJ_ENHMETAFILE {
            PlayEnhMetaFile(hdc, hEnhMetaFile, &mut rc);
        }
    }
}

unsafe extern "system" fn STATIC_PaintEtchedfn(hwnd: HWND, hdc: HDC, style: DWORD) {
    let mut rc: RECT = core::mem::zeroed();

    GetClientRect(hwnd, &mut rc);
    match style & SS_TYPEMASK {
        SS_ETCHEDHORZ => {
            DrawEdge(hdc, &mut rc, EDGE_ETCHED, BF_TOP | BF_BOTTOM);
        }
        SS_ETCHEDVERT => {
            DrawEdge(hdc, &mut rc, EDGE_ETCHED, BF_LEFT | BF_RIGHT);
        }
        SS_ETCHEDFRAME => {
            DrawEdge(hdc, &mut rc, EDGE_ETCHED, BF_RECT);
        }
        _ => {}
    }
}
