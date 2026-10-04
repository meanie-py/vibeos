use win32k_sys::win32k::*;
use core::ffi::c_void;

use crate::internal::{LOWORD, HIWORD, MAKEWPARAM, ValidateHwnd};
use crate::ntwrapper::GetControlColor;
// These are NtUser* syscall forwarders (ntwrapper.h EXTINLINE wrappers), not
// linkable user32 exports — call them through the crate's wrappers.
use crate::ntwrapper::{BeginPaint, EndPaint, GetDC, InvalidateRect, SetCapture, SetFocus};
// SetWindowLongPtrW is a winuser.h macro for SetWindowLongW on x86; class.rs only
// defines a real SetWindowLongPtrW (isize-typed) for non-x86. Local LONG_PTR-typed
// wrapper keeps this file's callsite contract; cfg-split like windows/dialog.rs.
#[cfg(not(target_arch = "x86"))]
#[inline]
unsafe fn SetWindowLongPtrW(hWnd: HWND, nIndex: i32, dwNewLong: LONG_PTR) -> LONG_PTR {
    crate::windows::class::SetWindowLongPtrW(hWnd, nIndex, dwNewLong as isize) as LONG_PTR
}
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn SetWindowLongPtrW(hWnd: HWND, nIndex: i32, dwNewLong: LONG_PTR) -> LONG_PTR {
    crate::windows::class::SetWindowLongW(hWnd, nIndex, dwNewLong as core::ffi::c_long) as LONG_PTR
}

const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

extern "system" {
    fn DefWindowProcW(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn DefWindowProcA(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn GetWindowLongPtrW(hWnd: HWND, nIndex: i32) -> isize;
    fn GetWindowLongW(hwnd: HWND, index: i32) -> LONG;
    fn HeapFree(hHeap: *mut core::ffi::c_void, dwFlags: u32, lpMem: *mut core::ffi::c_void) -> i32;
    fn HeapAlloc(hHeap: *mut core::ffi::c_void, dwFlags: u32, dwBytes: usize) -> *mut core::ffi::c_void;
    fn GetProcessHeap() -> *mut core::ffi::c_void;
    fn SelectObject(hdc: HDC, h: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn DrawTextW(hdc: HDC, lpchText: LPCWSTR, cchText: i32, lprc: *mut RECT, format: u32) -> i32;
    fn GetIconInfo(hIcon: HICON, piconinfo: *mut ICONINFO) -> i32;
    fn GetObjectW(h: *mut core::ffi::c_void, c: i32, pv: *mut core::ffi::c_void) -> i32;
    fn DeleteObject(ho: *mut core::ffi::c_void) -> i32;

    // user32 own-exports / gdi32 imports
    fn IsWindow(hWnd: HWND) -> BOOL;
    fn IsWindowVisible(hWnd: HWND) -> BOOL;
    fn IsWindowEnabled(hWnd: HWND) -> BOOL;
    fn ReleaseDC(hWnd: HWND, hDC: HDC) -> INT;
    fn GetParent(hWnd: HWND) -> HWND;
    fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn GetClientRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
    fn ReleaseCapture() -> BOOL;
    fn GetCapture() -> HWND;
    fn PtInRect(lprc: *const RECT, pt: POINT) -> BOOL;
    fn InternalGetWindowText(hWnd: HWND, pString: *mut WCHAR, cchMaxCount: INT) -> INT;
    fn GetNextDlgGroupItem(hDlg: HWND, hCtl: HWND, bPrevious: BOOL) -> HWND;
    fn DrawFocusRect(hDC: HDC, lprc: *const RECT) -> BOOL;
    fn DrawFrameControl(hdc: HDC, lprc: *mut RECT, uType: UINT, uState: UINT) -> BOOL;
    fn DrawEdge(hdc: HDC, qrc: *mut RECT, edge: UINT, grfFlags: UINT) -> BOOL;
    fn DrawStateW(
        hdc: HDC,
        hbrFore: HBRUSH,
        qfnCallBack: DRAWSTATEPROC,
        lData: LPARAM,
        wData: WPARAM,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        uFlags: UINT,
    ) -> BOOL;
    fn FillRect(hDC: HDC, lprc: *const RECT, hbr: HBRUSH) -> i32;
    fn SetRect(lprc: *mut RECT, xLeft: i32, yTop: i32, xRight: i32, yBottom: i32) -> BOOL;
    fn InflateRect(lprc: *mut RECT, dx: i32, dy: i32) -> BOOL;
    fn OffsetRect(lprc: *mut RECT, dx: i32, dy: i32) -> BOOL;
    fn IntersectRect(lprcDst: *mut RECT, lprcSrc1: *const RECT, lprcSrc2: *const RECT) -> BOOL;
    fn SetBkMode(hdc: HDC, mode: i32) -> i32;
    fn SetTextColor(hdc: HDC, color: COLORREF) -> COLORREF;
    fn GetStockObject(i: i32) -> *mut core::ffi::c_void;
    fn GetSysColor(nIndex: i32) -> COLORREF;
    fn GetSysColorBrush(nIndex: i32) -> HBRUSH;
    fn SetDCPenColor(hdc: HDC, color: COLORREF) -> COLORREF;
    fn Rectangle(hdc: HDC, left: i32, top: i32, right: i32, bottom: i32) -> i32;
    fn SelectClipRgn(hdc: HDC, hrgn: HRGN) -> i32;
    fn GetDeviceCaps(hdc: HDC, index: i32) -> i32;
    fn GetCharWidthW(hdc: HDC, iFirst: UINT, iLast: UINT, lpBuffer: *mut i32) -> BOOL;
    fn GetTextMetricsW(hdc: HDC, lptm: *mut TEXTMETRICW) -> BOOL;
}

// `set_control_clipping` — surviving C symbol in user32 (controls/uitools.c), cdecl.
extern "C" {
    fn set_control_clipping(hdc: HDC, rect: *const RECT) -> HRGN;
}

// ---- file-scope preamble (translated) ----
pub const STATE_GWL_OFFSET: u32 = 0;

pub const BUTTON_HFONT_GWL_OFFSET: u32 = core::mem::size_of::<LONG>() as u32;

pub const HIMAGE_GWL_OFFSET: u32 = BUTTON_HFONT_GWL_OFFSET + core::mem::size_of::<HFONT>() as u32;

pub const BUTTON_UISTATE_GWL_OFFSET: u32 = HIMAGE_GWL_OFFSET + core::mem::size_of::<HFONT>() as u32;

pub const NB_EXTRA_BYTES: u32 = BUTTON_UISTATE_GWL_OFFSET + core::mem::size_of::<LONG_PTR>() as u32;

pub const BUTTON_NSTATES: u32 = 0x0F;

pub const BUTTON_BTNPRESSED: u32 = 0x40;

pub const BUTTON_UNKNOWN2: u32 = 0x20;

pub const BUTTON_UNKNOWN3: u32 = 0x10;

pub const BUTTON_BMCLICK: u32 = 0x100;

pub const MAX_BTN_TYPE: usize = 16;

pub static maxCheckState: [WORD; 12] = [
    BST_UNCHECKED as WORD,
    BST_UNCHECKED as WORD,
    BST_CHECKED as WORD,
    BST_CHECKED as WORD,
    BST_CHECKED as WORD,
    BST_INDETERMINATE as WORD,
    BST_INDETERMINATE as WORD,
    BST_UNCHECKED as WORD,
    BST_UNCHECKED as WORD,
    BST_CHECKED as WORD,
    BST_UNCHECKED as WORD,
    BST_UNCHECKED as WORD,
];

pub type pfButtonPaint = Option<unsafe extern "system" fn(hwnd: HWND, hdc: HDC, action: UINT)>;

pub static btnPaintFunc: [pfButtonPaint; 12] = [
    Some(PB_Paint),
    Some(PB_Paint),
    Some(CB_Paint),
    Some(CB_Paint),
    Some(CB_Paint),
    Some(CB_Paint),
    Some(CB_Paint),
    Some(GB_Paint),
    Some(UB_Paint),
    Some(CB_Paint),
    None,
    Some(OB_Paint),
];

pub static buttonW: [WCHAR; 7] = [
    b'B' as WCHAR,
    b'u' as WCHAR,
    b't' as WCHAR,
    b't' as WCHAR,
    b'o' as WCHAR,
    b'n' as WCHAR,
    0,
];


// The state slot is a LONG (BUTTON_HFONT_GWL_OFFSET = sizeof(LONG)), so it is read and
// written with the 32-bit accessors. A LONG_PTR store at offset 0 on 64-bit overwrites
// the low half of the cached HFONT at offset 4: every BM_SETCHECK / BM_SETSTATE zeroed
// the button's font (upstream 057d978fcbf, "Fix extra-byte layout on x64").
unsafe fn get_button_state(hwnd: HWND) -> LONG {
    GetWindowLongW(hwnd, STATE_GWL_OFFSET as i32)
}

unsafe fn set_button_state(hwnd: HWND, state: LONG) {
    crate::windows::class::SetWindowLongW(hwnd, STATE_GWL_OFFSET as i32, state as core::ffi::c_long);
}

unsafe fn set_ui_state(hwnd: HWND, flags: LONG) {
    SetWindowLongPtrW(hwnd, BUTTON_UISTATE_GWL_OFFSET as i32, flags as LONG_PTR);
}

unsafe fn get_ui_state(hwnd: HWND) -> LONG {
    GetWindowLongPtrW(hwnd, BUTTON_UISTATE_GWL_OFFSET as core::ffi::c_int) as LONG
}

unsafe fn get_button_font(hwnd: HWND) -> HFONT {
    GetWindowLongPtrW(hwnd, core::mem::size_of::<LONG>() as core::ffi::c_int) as HFONT
}

unsafe fn set_button_font(hwnd: HWND, font: HFONT) {
    SetWindowLongPtrW(hwnd, core::mem::size_of::<LONG>() as core::ffi::c_int, font as LONG_PTR);
}

unsafe fn get_button_type(window_style: LONG) -> UINT {
    (window_style & BS_TYPEMASK as LONG) as UINT
}

unsafe fn paint_button( hwnd: HWND, style: LONG, action: UINT )
{
    if btnPaintFunc[style as usize].is_some() && IsWindowVisible(hwnd) != 0
    {
        let hdc = GetDC( hwnd );
        (btnPaintFunc[style as usize].unwrap())( hwnd, hdc, action );
        ReleaseDC( hwnd, hdc );
    }
}

unsafe fn get_button_text(hwnd: HWND) -> *mut WCHAR {
    let len: INT = 512;
    let buffer = HeapAlloc(GetProcessHeap(), 0, ((len + 1) as usize * core::mem::size_of::<WCHAR>()) as _) as *mut WCHAR;
    if !buffer.is_null() {
        InternalGetWindowText(hwnd, buffer, len + 1);
    }
    buffer
}

unsafe fn button_update_uistate(hwnd: HWND, unicode: BOOL) -> BOOL {

    let flags: LONG;
    let prevflags: LONG;

    if unicode != 0 {
        flags = DefWindowProcW(hwnd, WM_QUERYUISTATE, 0, 0) as LONG;
    } else {
        flags = DefWindowProcA(hwnd, WM_QUERYUISTATE, 0, 0) as LONG;
    }

    prevflags = get_ui_state(hwnd);

    if prevflags != flags {
        set_ui_state(hwnd, flags);
        return TRUE;
    }

    FALSE
}

/// BUTTON_NOTIFY_PARENT macro — notify the parent that created this button.
unsafe fn BUTTON_NOTIFY_PARENT(hWnd: HWND, code: UINT) {
    SendMessageW(
        GetParent(hWnd),
        WM_COMMAND,
        MAKEWPARAM(
            LOWORD(GetWindowLongPtrW(hWnd, GWLP_ID) as DWORD),
            code as WORD,
        ),
        hWnd as LPARAM,
    );
}

/***********************************************************************
 *           ButtonWndProc_common
 */
#[no_mangle]
pub unsafe extern "system" fn ButtonWndProc_common(
    hWnd: HWND,
    uMsg: UINT,
    mut wParam: WPARAM,
    lParam: LPARAM,
    unicode: BOOL,
) -> LRESULT {
    let mut rect: RECT = core::mem::zeroed();
    let mut pt: POINT = core::mem::zeroed();
    let mut style: LONG = GetWindowLongPtrW(hWnd, GWL_STYLE) as LONG;
    let mut btn_type: UINT = get_button_type(style);
    let mut state: LONG;
    let oldHbitmap: HANDLE;

    let pWnd: PWND = ValidateHwnd(hWnd);
    if !pWnd.is_null() {
        if (*pWnd).fnid == 0 {
            NtUserSetWindowFNID(hWnd, FNID_BUTTON as WORD);
        } else if (*pWnd).fnid != FNID_BUTTON {
            // ERR("Wrong window class for Button! fnId 0x%x\n", pWnd->fnid);
            return 0;
        }
    } else {
        return 0;
    }

    pt.x = (LOWORD(lParam as DWORD) as i16) as LONG;
    pt.y = (HIWORD(lParam as DWORD) as i16) as LONG;

    match uMsg {
        WM_GETDLGCODE => {
            match btn_type {
                BS_USERBUTTON | BS_PUSHBUTTON => {
                    return (DLGC_BUTTON | DLGC_UNDEFPUSHBUTTON) as LRESULT
                }
                BS_DEFPUSHBUTTON => return (DLGC_BUTTON | DLGC_DEFPUSHBUTTON) as LRESULT,
                BS_RADIOBUTTON | BS_AUTORADIOBUTTON => {
                    return (DLGC_BUTTON | DLGC_RADIOBUTTON) as LRESULT
                }
                BS_GROUPBOX => return DLGC_STATIC as LRESULT,
                _ => return DLGC_BUTTON as LRESULT,
            }
        }

        WM_ENABLE => {
            paint_button(hWnd, btn_type as LONG, ODA_DRAWENTIRE);
        }

        WM_CREATE => {
            if btn_type >= MAX_BTN_TYPE as UINT {
                return -1; /* abort */
            }

            /* XP turns a BS_USERBUTTON into BS_PUSHBUTTON */
            if btn_type == BS_USERBUTTON {
                style = (style & !(BS_TYPEMASK as LONG)) | BS_PUSHBUTTON as LONG;
                NtUserAlterWindowStyle(hWnd, GWL_STYLE as DWORD, style);
            }
            set_button_state(hWnd, BST_UNCHECKED as LONG);
            button_update_uistate(hWnd, unicode);
            return 0;
        }

        WM_NCDESTROY => {
            NtUserSetWindowFNID(hWnd, FNID_DESTROY as WORD);
            // fall through to WM_DESTROY
        }
        WM_DESTROY => {}

        WM_ERASEBKGND => {
            if btn_type == BS_OWNERDRAW {
                let hdc = wParam as HDC;
                let mut rc: RECT = core::mem::zeroed();
                let hBrush: HBRUSH;
                let mut parent = GetParent(hWnd);
                if parent.is_null() {
                    parent = hWnd;
                }
                hBrush = GetControlColor(parent, hWnd, hdc, WM_CTLCOLORBTN);
                GetClientRect(hWnd, &mut rc);
                FillRect(hdc, &rc, hBrush);
            }
            return 1;
        }

        WM_PRINTCLIENT | WM_PAINT => {
            let mut ps: PAINTSTRUCT = core::mem::zeroed();
            let hdc = if wParam != 0 {
                wParam as HDC
            } else {
                BeginPaint(hWnd, &mut ps)
            };
            if btnPaintFunc[btn_type as usize].is_some() {
                let nOldMode = SetBkMode(hdc, OPAQUE as i32);
                (btnPaintFunc[btn_type as usize].unwrap())(hWnd, hdc, ODA_DRAWENTIRE);
                SetBkMode(hdc, nOldMode); /* reset painting mode */
            }
            if wParam == 0 {
                EndPaint(hWnd, &ps);
            }
        }

        WM_KEYDOWN => {
            if wParam == VK_SPACE as WPARAM {
                SendMessageW(hWnd, BM_SETSTATE, TRUE as WPARAM, 0);
                set_button_state(hWnd, get_button_state(hWnd) | BUTTON_BTNPRESSED as LONG);
                SetCapture(hWnd);
            }
        }

        WM_LBUTTONDBLCLK => {
            if (style & BS_NOTIFY as LONG) != 0
                || btn_type == BS_RADIOBUTTON
                || btn_type == BS_USERBUTTON
                || btn_type == BS_OWNERDRAW
            {
                BUTTON_NOTIFY_PARENT(hWnd, BN_DOUBLECLICKED);
            } else {
                /* fall through to WM_LBUTTONDOWN */
                SetCapture(hWnd);
                SetFocus(hWnd);
                set_button_state(hWnd, get_button_state(hWnd) | BUTTON_BTNPRESSED as LONG);
                SendMessageW(hWnd, BM_SETSTATE, TRUE as WPARAM, 0);
            }
        }
        WM_LBUTTONDOWN => {
            SetCapture(hWnd);
            SetFocus(hWnd);
            set_button_state(hWnd, get_button_state(hWnd) | BUTTON_BTNPRESSED as LONG);
            SendMessageW(hWnd, BM_SETSTATE, TRUE as WPARAM, 0);
        }

        WM_KEYUP | WM_LBUTTONUP => {
            if uMsg == WM_KEYUP && wParam != VK_SPACE as WPARAM {
                /* WM_KEYUP only continues for VK_SPACE */
            } else {
                let mut TellParent: BOOL = FALSE; // ReactOS see note below.
                state = get_button_state(hWnd);
                if (state & BUTTON_BTNPRESSED as LONG) != 0 {
                    state &= BUTTON_NSTATES as LONG;
                    set_button_state(hWnd, state);
                    if (state & BST_PUSHED as LONG) == 0 {
                        ReleaseCapture();
                    } else {
                        SendMessageW(hWnd, BM_SETSTATE, FALSE as WPARAM, 0);
                        GetClientRect(hWnd, &mut rect);
                        if uMsg == WM_KEYUP || PtInRect(&rect, pt) != 0 {
                            state = get_button_state(hWnd);
                            match btn_type {
                                BS_AUTOCHECKBOX => {
                                    SendMessageW(
                                        hWnd,
                                        BM_SETCHECK,
                                        ((state & BST_CHECKED as LONG) == 0) as WPARAM,
                                        0,
                                    );
                                }
                                BS_AUTORADIOBUTTON => {
                                    BUTTON_CheckAutoRadioButton(hWnd);
                                }
                                BS_AUTO3STATE => {
                                    SendMessageW(
                                        hWnd,
                                        BM_SETCHECK,
                                        if (state & BST_INDETERMINATE as LONG) != 0 {
                                            0
                                        } else {
                                            (state & 3) + 1
                                        } as WPARAM,
                                        0,
                                    );
                                }
                                _ => {}
                            }
                            TellParent = TRUE; // Fix CORE-10194, notify parent after capture released.
                        }
                        ReleaseCapture();
                        if TellParent != 0 {
                            BUTTON_NOTIFY_PARENT(hWnd, BN_CLICKED);
                        }
                    }
                }
            }
        }

        WM_CAPTURECHANGED => {
            if hWnd != lParam as HWND {
                state = get_button_state(hWnd);
                if (state & BUTTON_BTNPRESSED as LONG) != 0 {
                    state &= BUTTON_NSTATES as LONG;
                    set_button_state(hWnd, state);
                    if (state & BST_PUSHED as LONG) != 0 {
                        SendMessageW(hWnd, BM_SETSTATE, FALSE as WPARAM, 0);
                    }
                }
            }
        }

        WM_MOUSEMOVE => {
            if (wParam & MK_LBUTTON as WPARAM) != 0 && GetCapture() == hWnd {
                GetClientRect(hWnd, &mut rect);
                SendMessageW(hWnd, BM_SETSTATE, (PtInRect(&rect, pt) != 0) as WPARAM, 0);
            }
        }

        WM_SETTEXT => {
            /* Clear an old text here as Windows does */
            if (style & WS_VISIBLE as LONG) != 0 {
                let hdc = GetDC(hWnd);
                let hbrush: HBRUSH;
                let mut client: RECT = core::mem::zeroed();
                let mut rc: RECT;
                let mut parent = GetParent(hWnd);
                let message: UINT = if btn_type == BS_PUSHBUTTON
                    || btn_type == BS_DEFPUSHBUTTON
                    || btn_type == BS_PUSHLIKE
                    || btn_type == BS_USERBUTTON
                    || btn_type == BS_OWNERDRAW
                {
                    WM_CTLCOLORBTN
                } else {
                    WM_CTLCOLORSTATIC
                };

                if parent.is_null() {
                    parent = hWnd;
                }
                hbrush = GetControlColor(parent, hWnd, hdc, message);

                GetClientRect(hWnd, &mut client);
                rc = client;
                /* FIXME: check other BS_* handlers */
                if btn_type == BS_GROUPBOX {
                    InflateRect(&mut rc, -7, 1); /* GB_Paint does this */
                }
                BUTTON_CalcLabelRect(hWnd, hdc, &mut rc);
                /* Clip by client rect bounds */
                if rc.right > client.right {
                    rc.right = client.right;
                }
                if rc.bottom > client.bottom {
                    rc.bottom = client.bottom;
                }
                FillRect(hdc, &rc, hbrush);
                ReleaseDC(hWnd, hdc);
            }

            if unicode != 0 {
                DefWindowProcW(hWnd, WM_SETTEXT, wParam, lParam);
            } else {
                DefWindowProcA(hWnd, WM_SETTEXT, wParam, lParam);
            }
            if btn_type == BS_GROUPBOX {
                /* Yes, only for BS_GROUPBOX */
                InvalidateRect(hWnd, core::ptr::null(), TRUE);
            } else {
                paint_button(hWnd, btn_type as LONG, ODA_DRAWENTIRE);
            }
            return 1; /* success. FIXME: check text length */
        }

        WM_SETFONT => {
            set_button_font(hWnd, wParam as HFONT);
            if lParam != 0 {
                InvalidateRect(hWnd, core::ptr::null(), TRUE);
            }
        }

        WM_GETFONT => {
            return get_button_font(hWnd) as LRESULT;
        }

        WM_SETFOCUS => {
            set_button_state(hWnd, get_button_state(hWnd) | BST_FOCUS as LONG);
            paint_button(hWnd, btn_type as LONG, ODA_FOCUS);
            if (style & BS_NOTIFY as LONG) != 0 {
                BUTTON_NOTIFY_PARENT(hWnd, BN_SETFOCUS);
            }
            if ((btn_type == BS_RADIOBUTTON) || (btn_type == BS_AUTORADIOBUTTON))
                && (get_button_state(hWnd) & BST_CHECKED as LONG) == 0
            {
                BUTTON_NOTIFY_PARENT(hWnd, BN_CLICKED);
            }
        }

        WM_KILLFOCUS => {
            state = get_button_state(hWnd);
            set_button_state(hWnd, state & !(BST_FOCUS as LONG));
            paint_button(hWnd, btn_type as LONG, ODA_FOCUS);

            if (state & BUTTON_BTNPRESSED as LONG) != 0 && GetCapture() == hWnd {
                ReleaseCapture();
            }
            if (style & BS_NOTIFY as LONG) != 0 {
                BUTTON_NOTIFY_PARENT(hWnd, BN_KILLFOCUS);
            }

            InvalidateRect(hWnd, core::ptr::null(), FALSE);
        }

        WM_SYSCOLORCHANGE => {
            InvalidateRect(hWnd, core::ptr::null(), FALSE);
        }

        BM_SETSTYLE => {
            btn_type = (wParam & BS_TYPEMASK as WPARAM) as UINT;
            style = (style & !(BS_TYPEMASK as LONG)) | btn_type as LONG;
            NtUserAlterWindowStyle(hWnd, GWL_STYLE as DWORD, style);

            /* Only redraw if lParam flag is set. */
            if lParam != 0 {
                InvalidateRect(hWnd, core::ptr::null(), TRUE);
            }
        }

        BM_CLICK => {
            state = get_button_state(hWnd);
            if (state & BUTTON_BMCLICK as LONG) != 0 {
                // break
            } else {
                set_button_state(hWnd, state | BUTTON_BMCLICK as LONG); // Tracked in STATE_GWL_OFFSET.
                SendMessageW(hWnd, WM_LBUTTONDOWN, 0, 0);
                SendMessageW(hWnd, WM_LBUTTONUP, 0, 0);
                state = get_button_state(hWnd);
                if (state & BUTTON_BMCLICK as LONG) != 0 {
                    state &= !(BUTTON_BMCLICK as LONG);
                    set_button_state(hWnd, state);
                }
            }
        }

        BM_SETIMAGE => {
            /* Check that image format matches button style */
            match style & (BS_BITMAP | BS_ICON) as LONG {
                x if x == BS_BITMAP as LONG => {
                    if wParam != IMAGE_BITMAP as WPARAM {
                        return 0;
                    }
                }
                x if x == BS_ICON as LONG => {
                    if wParam != IMAGE_ICON as WPARAM {
                        return 0;
                    }
                }
                _ => {
                    return 0;
                }
            }
            oldHbitmap = SetWindowLongPtrW(hWnd, HIMAGE_GWL_OFFSET as i32, lParam) as HANDLE;
            InvalidateRect(hWnd, core::ptr::null(), FALSE);
            return oldHbitmap as LRESULT;
        }

        BM_GETIMAGE => {
            return GetWindowLongPtrW(hWnd, HIMAGE_GWL_OFFSET as i32) as LRESULT;
        }

        BM_GETCHECK => {
            return (get_button_state(hWnd) & 3) as LRESULT;
        }

        BM_SETCHECK => {
            if wParam > maxCheckState[btn_type as usize] as WPARAM {
                wParam = maxCheckState[btn_type as usize] as WPARAM;
            }
            state = get_button_state(hWnd);
            if (btn_type == BS_RADIOBUTTON) || (btn_type == BS_AUTORADIOBUTTON) {
                if wParam != 0 {
                    style |= WS_TABSTOP as LONG;
                } else {
                    style &= !(WS_TABSTOP as LONG);
                }
                NtUserAlterWindowStyle(hWnd, GWL_STYLE as DWORD, style);
            }
            if (state & 3) != wParam as LONG {
                set_button_state(hWnd, (state & !3) | wParam as LONG);
                paint_button(hWnd, btn_type as LONG, ODA_SELECT);
            }
        }

        BM_GETSTATE => {
            return get_button_state(hWnd) as LRESULT;
        }

        BM_SETSTATE => {
            state = get_button_state(hWnd);
            if wParam != 0 {
                set_button_state(hWnd, state | BST_PUSHED as LONG);
            } else {
                set_button_state(hWnd, state & !(BST_PUSHED as LONG));
            }

            paint_button(hWnd, btn_type as LONG, ODA_SELECT);
        }

        WM_UPDATEUISTATE => {
            if unicode != 0 {
                DefWindowProcW(hWnd, uMsg, wParam, lParam);
            } else {
                DefWindowProcA(hWnd, uMsg, wParam, lParam);
            }

            if button_update_uistate(hWnd, unicode) != 0 {
                paint_button(hWnd, btn_type as LONG, ODA_DRAWENTIRE);
            }
        }

        WM_NCHITTEST => {
            if btn_type == BS_GROUPBOX {
                return HTTRANSPARENT as LRESULT;
            }
            /* fall through to default */
            return if unicode != 0 {
                DefWindowProcW(hWnd, uMsg, wParam, lParam)
            } else {
                DefWindowProcA(hWnd, uMsg, wParam, lParam)
            };
        }

        _ => {
            return if unicode != 0 {
                DefWindowProcW(hWnd, uMsg, wParam, lParam)
            } else {
                DefWindowProcA(hWnd, uMsg, wParam, lParam)
            };
        }
    }
    0
}

/***********************************************************************
 *           ButtonWndProcW
 */
#[no_mangle]
pub unsafe extern "system" fn ButtonWndProcW(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    if IsWindow(hWnd) == 0 {
        return 0;
    }
    ButtonWndProc_common(hWnd, uMsg, wParam, lParam, TRUE)
}

/***********************************************************************
 *           ButtonWndProcA
 */
#[no_mangle]
pub unsafe extern "system" fn ButtonWndProcA(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    if IsWindow(hWnd) == 0 {
        return 0;
    }
    ButtonWndProc_common(hWnd, uMsg, wParam, lParam, FALSE)
}

unsafe fn BUTTON_BStoDT(mut style: DWORD, ex_style: DWORD) -> UINT {
    let mut dtStyle: UINT = DT_NOCLIP;

    if (style & BS_PUSHLIKE) != 0 {
        style &= !BS_TYPEMASK;
    }

    if (style & BS_MULTILINE) == 0 {
        dtStyle |= DT_SINGLELINE;
    } else {
        dtStyle |= DT_WORDBREAK;
    }

    match style & BS_CENTER {
        x if x == BS_LEFT => {}
        x if x == BS_RIGHT => dtStyle |= DT_RIGHT,
        x if x == BS_CENTER => dtStyle |= DT_CENTER,
        _ => {
            if get_button_type(style as LONG) <= BS_DEFPUSHBUTTON {
                dtStyle |= DT_CENTER;
            }
        }
    }

    if (ex_style & WS_EX_RIGHT) != 0 {
        dtStyle = DT_RIGHT | (dtStyle & !(DT_LEFT | DT_CENTER));
    }

    if get_button_type(style as LONG) != BS_GROUPBOX {
        match style & BS_VCENTER {
            x if x == BS_TOP => {}
            x if x == BS_BOTTOM => dtStyle |= DT_BOTTOM,
            x if x == BS_VCENTER => dtStyle |= DT_VCENTER,
            _ => dtStyle |= DT_VCENTER,
        }
    } else {
        dtStyle |= DT_SINGLELINE;
    }

    dtStyle
}

unsafe fn BUTTON_CalcLabelRect(hwnd: HWND, hdc: HDC, rc: *mut RECT) -> UINT {

    let style = GetWindowLongPtrW(hwnd, GWL_STYLE as i32) as LONG;
    let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE as i32) as LONG;
    let text: *mut WCHAR;
    let mut iconInfo: ICONINFO = core::mem::zeroed();
    let mut bm: BITMAP = core::mem::zeroed();
    let mut dtStyle: UINT = BUTTON_BStoDT(style as DWORD, ex_style as DWORD);
    let mut r: RECT = *rc;
    let n: INT;

    const HIMAGE_GWL_OFFSET: i32 =
        (core::mem::size_of::<LONG>() + core::mem::size_of::<HFONT>()) as i32;

    'done: {
        match (style as u32) & (BS_ICON | BS_BITMAP) {
            BS_TEXT => {
                let hFont: HFONT;
                let mut hPrevFont: *mut core::ffi::c_void = core::ptr::null_mut();

                text = get_button_text(hwnd);
                if text.is_null() {
                    break 'done;
                }
                if *text == 0 {
                    HeapFree(GetProcessHeap(), 0, text as *mut core::ffi::c_void);
                    break 'done;
                }

                hFont = get_button_font(hwnd);
                if !hFont.is_null() {
                    hPrevFont = SelectObject(hdc, hFont as *mut core::ffi::c_void);
                }
                DrawTextW(
                    hdc,
                    text,
                    -1,
                    &mut r,
                    (dtStyle | DT_CALCRECT) & !(DT_VCENTER | DT_BOTTOM),
                );
                if !hPrevFont.is_null() {
                    SelectObject(hdc, hPrevFont);
                }
                HeapFree(GetProcessHeap(), 0, text as *mut core::ffi::c_void);
                if (get_ui_state(hwnd) as u32) & UISF_HIDEACCEL != 0 {
                    dtStyle |= DT_HIDEPREFIX;
                }
            }

            BS_ICON => {
                if GetIconInfo(
                    GetWindowLongPtrW(hwnd, HIMAGE_GWL_OFFSET) as HICON,
                    &mut iconInfo,
                ) == 0
                {
                    break 'done;
                }

                GetObjectW(
                    iconInfo.hbmColor as *mut core::ffi::c_void,
                    core::mem::size_of::<BITMAP>() as i32,
                    &mut bm as *mut BITMAP as *mut core::ffi::c_void,
                );

                r.right = r.left + bm.bmWidth;
                r.bottom = r.top + bm.bmHeight;

                DeleteObject(iconInfo.hbmColor as *mut core::ffi::c_void);
                DeleteObject(iconInfo.hbmMask as *mut core::ffi::c_void);
            }

            BS_BITMAP => {
                if GetObjectW(
                    GetWindowLongPtrW(hwnd, HIMAGE_GWL_OFFSET) as *mut core::ffi::c_void,
                    core::mem::size_of::<BITMAP>() as i32,
                    &mut bm as *mut BITMAP as *mut core::ffi::c_void,
                ) == 0
                {
                    break 'done;
                }

                r.right = r.left + bm.bmWidth;
                r.bottom = r.top + bm.bmHeight;
            }

            _ => {
                break 'done;
            }
        }

        match dtStyle & (DT_CENTER | DT_RIGHT) {
            DT_LEFT => {
                r.left += 1;
                r.right += 1;
            }
            DT_CENTER => {
                let n = r.right - r.left;
                r.left = (*rc).left + (((*rc).right - (*rc).left) - n) / 2;
                r.right = r.left + n;
            }
            DT_RIGHT => {
                let n = r.right - r.left;
                r.right = (*rc).right - 1;
                r.left = r.right - n;
            }
            _ => {}
        }

        match dtStyle & (DT_VCENTER | DT_BOTTOM) {
            DT_TOP => {
                r.top += 1;
                r.bottom += 1;
            }
            DT_VCENTER => {
                let n = r.bottom - r.top;
                r.top = (*rc).top + (((*rc).bottom - 1 - (*rc).top) - n) / 2;
                r.bottom = r.top + n;
            }
            DT_BOTTOM => {
                let n = r.bottom - r.top;
                r.bottom = (*rc).bottom - 1;
                r.top = r.bottom - n;
            }
            _ => {}
        }

        *rc = r;
        return dtStyle;
    }

    (*rc).right = r.left;
    (*rc).bottom = r.top;
    (-1i32) as UINT
}

unsafe extern "system" fn BUTTON_DrawTextCallback(hdc: HDC, lp: LPARAM, wp: WPARAM, cx: i32, cy: i32) -> BOOL {
    let mut rc: RECT = core::mem::zeroed();

    SetRect(&mut rc, 0, 0, cx, cy);
    DrawTextW(hdc, lp as LPCWSTR, -1, &mut rc, wp as UINT);
    TRUE
}

unsafe fn BUTTON_DrawLabel(hwnd: HWND, hdc: HDC, dtFlags: UINT, rc: *const RECT) {
    let mut lpOutputProc: DRAWSTATEPROC = None;
    let lp: LPARAM;
    let mut wp: WPARAM = 0;
    let mut hbr: HBRUSH = core::ptr::null_mut();
    let mut flags: UINT = if IsWindowEnabled(hwnd) != 0 { DSS_NORMAL } else { DSS_DISABLED };
    let state: LONG = get_button_state(hwnd);
    let style: LONG = GetWindowLongPtrW(hwnd, GWL_STYLE) as LONG;
    let mut text: *mut WCHAR = core::ptr::null_mut();

    if (style & BS_PUSHLIKE as LONG) != 0 && (state & BST_INDETERMINATE as LONG) != 0 {
        hbr = GetSysColorBrush(COLOR_GRAYTEXT as i32);
        flags |= DSS_MONO;
    }

    match (style as UINT) & (BS_ICON | BS_BITMAP) {
        BS_TEXT => {
            lpOutputProc = core::mem::transmute::<usize, DRAWSTATEPROC>(
                BUTTON_DrawTextCallback as usize,
            );
            text = get_button_text(hwnd);
            if text.is_null() {
                return;
            }
            lp = text as LPARAM;
            wp = dtFlags as WPARAM;

            if (dtFlags & DT_HIDEPREFIX) != 0 {
                flags |= DSS_HIDEPREFIX;
            }
        }
        BS_ICON => {
            flags |= DST_ICON;
            lp = GetWindowLongPtrW(hwnd, HIMAGE_GWL_OFFSET as i32) as LPARAM;
        }
        BS_BITMAP => {
            flags |= DST_BITMAP;
            lp = GetWindowLongPtrW(hwnd, HIMAGE_GWL_OFFSET as i32) as LPARAM;
        }
        _ => {
            return;
        }
    }

    DrawStateW(
        hdc,
        hbr,
        lpOutputProc,
        lp,
        wp,
        (*rc).left,
        (*rc).top,
        (*rc).right - (*rc).left,
        (*rc).bottom - (*rc).top,
        flags,
    );
    HeapFree(GetProcessHeap(), 0, text as *mut core::ffi::c_void);
}

unsafe extern "system" fn PB_Paint(hwnd: HWND, hDC: HDC, action: UINT) {
    let mut rc: RECT = core::mem::zeroed();
    let mut r: RECT = core::mem::zeroed();
    let mut dtFlags: UINT;
    let mut uState: UINT;
    let hOldPen: HPEN;
    let hOldBrush: HBRUSH;
    let oldBkMode: INT;
    let oldTxtColor: COLORREF;
    let hFont: HFONT;
    let state: LONG = get_button_state(hwnd);
    let style: LONG = GetWindowLongPtrW(hwnd, GWL_STYLE) as LONG;
    let pushedState: BOOL = (state & BST_PUSHED as LONG != 0) as BOOL;
    let mut parent: HWND;
    let hrgn: HRGN;

    GetClientRect(hwnd, &mut rc);

    hFont = get_button_font(hwnd);
    if !hFont.is_null() {
        SelectObject(hDC, hFont as _);
    }
    parent = GetParent(hwnd);
    if parent.is_null() {
        parent = hwnd;
    }
    GetControlColor(parent, hwnd, hDC, WM_CTLCOLORBTN);

    hrgn = set_control_clipping(hDC, &mut rc);

    hOldPen = SelectObject(hDC, GetStockObject(DC_PEN as INT)) as HPEN;
    SetDCPenColor(hDC, GetSysColor(COLOR_WINDOWFRAME as INT));

    hOldBrush = SelectObject(hDC, GetSysColorBrush(COLOR_BTNFACE as INT) as _) as HBRUSH;
    oldBkMode = SetBkMode(hDC, TRANSPARENT as INT);

    if get_button_type(style) == BS_DEFPUSHBUTTON {
        if action != ODA_FOCUS {
            Rectangle(hDC, rc.left, rc.top, rc.right, rc.bottom);
        }
        InflateRect(&mut rc, -1, -1);
    }

    'cleanup: {
        'draw_focus: {
            if action == ODA_FOCUS {
                break 'draw_focus;
            }

            uState = DFCS_BUTTONPUSH;

            if style & BS_FLAT as LONG != 0 {
                uState |= DFCS_MONO;
            } else if pushedState != 0 {
                if get_button_type(style) == BS_DEFPUSHBUTTON {
                    uState |= DFCS_FLAT;
                } else {
                    uState |= DFCS_PUSHED;
                }
            }

            if state & (BST_CHECKED | BST_INDETERMINATE) as LONG != 0 {
                uState |= DFCS_CHECKED;
            }

            DrawFrameControl(hDC, &mut rc, DFC_BUTTON, uState);

            r = rc;
            dtFlags = BUTTON_CalcLabelRect(hwnd, hDC, &mut r);

            if dtFlags == (-1i32) as UINT {
                break 'cleanup;
            }

            if pushedState != 0 {
                OffsetRect(&mut r, 1, 1);
            }

            oldTxtColor = SetTextColor(hDC, GetSysColor(COLOR_BTNTEXT as INT));

            BUTTON_DrawLabel(hwnd, hDC, dtFlags, &r);

            SetTextColor(hDC, oldTxtColor);
        }

        if action == ODA_FOCUS || (state & BST_FOCUS as LONG != 0) {
            if get_ui_state(hwnd) & UISF_HIDEFOCUS as LONG == 0 {
                InflateRect(&mut rc, -2, -2);
                DrawFocusRect(hDC, &mut rc);
            }
        }
    }

    SelectObject(hDC, hOldPen as _);
    SelectObject(hDC, hOldBrush as _);
    SetBkMode(hDC, oldBkMode);
    SelectClipRgn(hDC, hrgn);
    if !hrgn.is_null() {
        DeleteObject(hrgn as _);
    }
}

unsafe extern "system" fn CB_Paint(hwnd: HWND, hDC: HDC, action: UINT) {
    let mut rbox: RECT = core::mem::zeroed();
    let mut rtext: RECT = core::mem::zeroed();
    let mut client: RECT = core::mem::zeroed();
    let hBrush: HBRUSH;
    let delta: i32;
    let mut text_offset: i32 = 0;
    let checkBoxWidth: i32;
    let checkBoxHeight: i32;
    let dtFlags: UINT;
    let hFont: HFONT;
    let state: LONG = get_button_state(hwnd);
    let style: LONG = GetWindowLongPtrW(hwnd, GWL_STYLE as i32) as LONG;
    let ex_style: LONG = GetWindowLongW(hwnd, GWL_EXSTYLE as i32);
    let mut parent: HWND;
    let hrgn: HRGN;

    if (style & BS_PUSHLIKE as LONG) != 0 {
        PB_Paint(hwnd, hDC, action);
        return;
    }

    GetClientRect(hwnd, &mut client);
    rbox = client;
    rtext = client;

    checkBoxWidth = 12 * GetDeviceCaps(hDC, LOGPIXELSX as i32) / 96 + 1;
    checkBoxHeight = 12 * GetDeviceCaps(hDC, LOGPIXELSY as i32) / 96 + 1;

    hFont = get_button_font(hwnd);
    if !hFont.is_null() {
        SelectObject(hDC, hFont as *mut core::ffi::c_void);
    }
    GetCharWidthW(hDC, '0' as UINT, '0' as UINT, &mut text_offset);
    text_offset /= 2;

    parent = GetParent(hwnd);
    if parent.is_null() {
        parent = hwnd;
    }
    hBrush = GetControlColor(parent, hwnd, hDC, WM_CTLCOLORSTATIC);

    hrgn = set_control_clipping(hDC, &client);

    if (style & BS_LEFTTEXT as LONG) != 0 || (ex_style & WS_EX_RIGHT as LONG) != 0 {
        rtext.right -= checkBoxWidth + text_offset;
        rbox.left = rbox.right - checkBoxWidth;
    } else {
        rtext.left += checkBoxWidth + text_offset;
        rbox.right = checkBoxWidth;
    }

    if action == ODA_SELECT {
        FillRect(hDC, &rbox, hBrush);
    }
    if action == ODA_DRAWENTIRE {
        FillRect(hDC, &client, hBrush);
    }

    client = rtext;
    dtFlags = BUTTON_CalcLabelRect(hwnd, hDC, &mut rtext);

    if dtFlags != (-1i32) as UINT {
        rbox.top = rtext.top;
        rbox.bottom = rtext.bottom;
    }

    if action == ODA_DRAWENTIRE || action == ODA_SELECT {
        let mut flags: UINT;

        if (get_button_type(style) == BS_RADIOBUTTON)
            || (get_button_type(style) == BS_AUTORADIOBUTTON)
        {
            flags = DFCS_BUTTONRADIO;
        } else if (state & BST_INDETERMINATE as LONG) != 0 {
            flags = DFCS_BUTTON3STATE;
        } else {
            flags = DFCS_BUTTONCHECK;
        }

        if (state & (BST_CHECKED as LONG | BST_INDETERMINATE as LONG)) != 0 {
            flags |= DFCS_CHECKED;
        }
        if (state & BST_PUSHED as LONG) != 0 {
            flags |= DFCS_PUSHED;
        }

        if (style & WS_DISABLED as LONG) != 0 {
            flags |= DFCS_INACTIVE;
        }

        delta = rbox.bottom - rbox.top - checkBoxHeight;

        if (style & BS_TOP as LONG) != 0 {
            if delta > 0 {
                rbox.bottom = rbox.top + checkBoxHeight;
            } else {
                rbox.top -= -delta / 2 + 1;
                rbox.bottom = rbox.top + checkBoxHeight;
            }
        } else if (style & BS_BOTTOM as LONG) != 0 {
            if delta > 0 {
                rbox.top = rbox.bottom - checkBoxHeight;
            } else {
                rbox.bottom += -delta / 2 + 1;
                rbox.top = rbox.bottom - checkBoxHeight;
            }
        } else {
            if delta > 0 {
                let ofs = delta / 2;
                rbox.bottom -= ofs + 1;
                rbox.top = rbox.bottom - checkBoxHeight;
            } else if delta < 0 {
                let ofs = -delta / 2;
                rbox.top -= ofs + 1;
                rbox.bottom = rbox.top + checkBoxHeight;
            }
        }

        DrawFrameControl(hDC, &mut rbox, DFC_BUTTON, flags);
    }

    if dtFlags == (-1i32) as UINT {
        return;
    }

    if action == ODA_DRAWENTIRE {
        BUTTON_DrawLabel(hwnd, hDC, dtFlags, &rtext);
    }

    if action == ODA_FOCUS || (state & BST_FOCUS as LONG) != 0 {
        if (get_ui_state(hwnd) & UISF_HIDEFOCUS as LONG) == 0 {
            rtext.left -= 1;
            rtext.right += 1;
            IntersectRect(&mut rtext, &rtext, &client);
            DrawFocusRect(hDC, &rtext);
        }
    }
    SelectClipRgn(hDC, hrgn);
    if !hrgn.is_null() {
        DeleteObject(hrgn as *mut core::ffi::c_void);
    }
}

unsafe fn BUTTON_CheckAutoRadioButton(hwnd: HWND) {
    let parent: HWND;
    let mut sibling: HWND;
    let start: HWND;

    parent = GetParent(hwnd);
    start = hwnd;
    sibling = hwnd;
    loop {
        if sibling.is_null() {
            break;
        }
        if (SendMessageW(sibling, WM_GETDLGCODE, 0, 0) & (DLGC_BUTTON as LPARAM | DLGC_RADIOBUTTON as LPARAM))
            == (DLGC_BUTTON as LPARAM | DLGC_RADIOBUTTON as LPARAM)
        {
            SendMessageW(
                sibling,
                BM_SETCHECK,
                if sibling == hwnd { BST_CHECKED } else { BST_UNCHECKED } as WPARAM,
                0,
            );
        }
        sibling = GetNextDlgGroupItem(parent, sibling, FALSE);
        if sibling == start {
            break;
        }
    }
}

unsafe extern "system" fn GB_Paint(hwnd: HWND, hDC: HDC, action: UINT) {
    let mut rc: RECT = core::mem::zeroed();
    let mut rcFrame: RECT;
    let hbr: HBRUSH;
    let hFont: HFONT;
    let dtFlags: UINT;
    let mut tm: TEXTMETRICW = core::mem::zeroed();
    let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as LONG;
    let mut parent: HWND;
    let hrgn: HRGN;

    hFont = get_button_font(hwnd);
    if !hFont.is_null() {
        SelectObject(hDC, hFont as _);
    }
    parent = GetParent(hwnd);
    if parent.is_null() {
        parent = hwnd;
    }
    hbr = GetControlColor(parent, hwnd, hDC, WM_CTLCOLORSTATIC);

    GetClientRect(hwnd, &mut rc);
    rcFrame = rc;
    hrgn = set_control_clipping(hDC, &mut rc);

    GetTextMetricsW(hDC, &mut tm);
    rcFrame.top += (tm.tmHeight / 2) - 1;
    DrawEdge(hDC, &mut rcFrame, EDGE_ETCHED, BF_RECT | (if (style & BS_FLAT as LONG) != 0 { BF_FLAT } else { 0 }));

    InflateRect(&mut rc, -7, 1);
    dtFlags = BUTTON_CalcLabelRect(hwnd, hDC, &mut rc);

    if dtFlags != (-1i32 as UINT) {
        rc.left -= 1;
        rc.right += 1;
        rc.bottom += 1;
        FillRect(hDC, &rc, hbr);
        rc.left += 1;
        rc.right -= 1;
        rc.bottom -= 1;

        BUTTON_DrawLabel(hwnd, hDC, dtFlags, &rc);
    }
    SelectClipRgn(hDC, hrgn);
    if !hrgn.is_null() {
        DeleteObject(hrgn as _);
    }
}

unsafe extern "system" fn UB_Paint(hwnd: HWND, hDC: HDC, action: UINT) {
    let mut rc: RECT = core::mem::zeroed();
    let hBrush: HBRUSH;
    let hFont: HFONT;
    let state: LONG = get_button_state(hwnd);
    let mut parent: HWND;

    GetClientRect(hwnd, &mut rc);

    hFont = get_button_font(hwnd);
    if !hFont.is_null() {
        SelectObject(hDC, hFont as _);
    }

    parent = GetParent(hwnd);
    if parent.is_null() {
        parent = hwnd;
    }
    hBrush = GetControlColor(parent, hwnd, hDC, WM_CTLCOLORBTN);

    FillRect(hDC, &rc, hBrush);
    if action == ODA_FOCUS || (state & BST_FOCUS as LONG) != 0 {
        if (get_ui_state(hwnd) & UISF_HIDEFOCUS as LONG) == 0 {
            DrawFocusRect(hDC, &rc);
        }
    }

    match action {
        ODA_FOCUS => {
            BUTTON_NOTIFY_PARENT(
                hwnd,
                if (state & BST_FOCUS as LONG) != 0 { BN_SETFOCUS } else { BN_KILLFOCUS },
            );
        }
        ODA_SELECT => {
            BUTTON_NOTIFY_PARENT(
                hwnd,
                if (state & BST_PUSHED as LONG) != 0 { BN_HILITE } else { BN_UNHILITE },
            );
        }
        _ => {
            BUTTON_NOTIFY_PARENT(hwnd, BN_PAINT);
        }
    }
}

unsafe extern "system" fn OB_Paint(hwnd: HWND, hDC: HDC, action: UINT) {
    let state = get_button_state(hwnd);
    let mut dis: DRAWITEMSTRUCT = core::mem::zeroed();
    let id = GetWindowLongPtrW(hwnd, GWLP_ID);
    let mut parent;
    let hFont;
    let mut hPrevFont: HFONT = core::ptr::null_mut();
    let hrgn;

    dis.CtlType = ODT_BUTTON;
    dis.CtlID = id as UINT;
    dis.itemID = 0;
    dis.itemAction = action;
    dis.itemState = (if (state & BST_FOCUS as LONG) != 0 { ODS_FOCUS } else { 0 })
        | (if (state & BST_PUSHED as LONG) != 0 { ODS_SELECTED } else { 0 })
        | (if IsWindowEnabled(hwnd) != 0 { 0 } else { ODS_DISABLED });
    dis.hwndItem = hwnd;
    dis.hDC = hDC;
    dis.itemData = 0;
    GetClientRect(hwnd, core::ptr::addr_of_mut!(dis.rcItem) as *mut RECT);

    hFont = get_button_font(hwnd);
    if !hFont.is_null() {
        hPrevFont = SelectObject(hDC, hFont as *mut core::ffi::c_void) as HFONT;
    }
    parent = GetParent(hwnd);
    if parent.is_null() {
        parent = hwnd;
    }
    GetControlColor(parent, hwnd, hDC, WM_CTLCOLORBTN);

    hrgn = set_control_clipping(hDC, core::ptr::addr_of_mut!(dis.rcItem) as *const RECT);

    SendMessageW(
        GetParent(hwnd),
        WM_DRAWITEM,
        id as WPARAM,
        core::ptr::addr_of_mut!(dis) as LPARAM,
    );
    if !hPrevFont.is_null() {
        SelectObject(hDC, hPrevFont as *mut core::ffi::c_void);
    }
    SelectClipRgn(hDC, hrgn);
    if !hrgn.is_null() {
        DeleteObject(hrgn as *mut core::ffi::c_void);
    }
}

// ─── BUTTON_builtin_class — cross-TU data symbol read by controls/regcontrol.c ───
// Faithful to controls/button.c:142.
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
unsafe impl Sync for builtin_class_descr {}

#[no_mangle]
pub static BUTTON_builtin_class: builtin_class_descr = builtin_class_descr {
    name: buttonW.as_ptr(),
    style: CS_DBLCLKS | CS_VREDRAW | CS_HREDRAW | CS_PARENTDC,
    procA: unsafe {
        core::mem::transmute::<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT, WNDPROC>(ButtonWndProcA)
    },
    procW: unsafe {
        core::mem::transmute::<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT, WNDPROC>(ButtonWndProcW)
    },
    extra: NB_EXTRA_BYTES as INT,
    cursor: 32512 as LPCWSTR, // IDC_ARROW
    brush: 0 as HBRUSH,
};
