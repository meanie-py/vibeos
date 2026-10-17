use win32k_sys::win32k::*;
// The crate's own functions: the definition is the declaration.
use crate::windows::class::GetWindowLongPtrW;
use crate::windows::rect::IsRectEmpty;
use core::ffi::c_void;

// FASTCALL ValidateHwnd + UserApiHook family + the guah/gfServerProcess globals and
// the LoadUserApiHook static-inline live in the shared cohab substrate (cfg-split).
use crate::internal::{
    ValidateHwnd, BeginIfHookedUserApiHook, EndUserApiHook, LoadUserApiHook, guah,
};
// Win32 word/coordinate macros (winuser.h) — provided by the shared substrate.
use crate::internal::{HIWORD, LOWORD, MAKEWPARAM};
// TEB self-pointer inline (used by GetWin32ClientInfo below).
use crate::internal::NtCurrentTeb;
// NtUser* syscall forwarders (ntwrapper.h EXTINLINE wrappers), not linkable exports.
use crate::ntwrapper::{
    BeginPaint, CreateCaret, EndPaint, GetDCEx, HideCaret, InvalidateRect, MoveWindow, SetCapture,
    SetFocus, SetSystemTimer, ShowCaret,
};

const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// Wine debug-channel stubs (WINE_DEFAULT_DEBUG_CHANNEL(scrollbar)) — no-ops.

// C `max()` macro equivalent (windef.h) — used by the scrollbar geometry math.
#[inline]
fn max(a: INT, b: INT) -> INT {
    if a > b { a } else { b }
}

/// user_x.h inline `GetWin32ClientInfo()` — `&NtCurrentTeb()->Win32ClientInfo`.
#[inline]
unsafe fn GetWin32ClientInfo() -> PCLIENTINFO {
    core::ptr::addr_of_mut!((*NtCurrentTeb()).Win32ClientInfo) as *mut _ as PCLIENTINFO
}

/// `DesktopPtrToUser` — user32p.h inline: offsets by `pci->ulClientDelta`.
#[inline]
unsafe fn DesktopPtrToUser(p: PVOID) -> PVOID {
    // WoW64: DESKTOPINFO is 64-bit-laid-out for a 32-bit guest, so the
    // base/limit range check must be done through the normalizing view.
    // Single definition lives in crate::wow64.
    crate::wow64::desktop_ptr_to_user(p)
}

/// `GetControlBrush` — user32p.h inline over the NtUserGetControlBrush syscall.
#[inline]
unsafe fn GetControlBrush(hwnd: HWND, hdc: HDC, ctlType: UINT) -> HBRUSH {
    NtUserGetControlBrush(hwnd, hdc, ctlType)
}

// DefWndControlColor + mirror_rect are non-static cdecl symbols exported by the
// (Rust) defwnd/winpos translation units — link against them.
extern "C" {
    fn DefWndControlColor(hDC: HDC, ctlType: UINT) -> HBRUSH;
    fn mirror_rect(window_rect: *const RECT, rect: *mut RECT);
}

// user32's FASTCALL IntNotifyWinEvent (windows/hook.c, decl in user32p.h) takes an
// HWND — shadow the bindgen win32k::IntNotifyWinEvent(PWND) glob import. i386 = @Name@N.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn IntNotifyWinEvent(event: DWORD, hwnd: HWND, idObject: LONG, idChild: LONG, flags: DWORD);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn IntNotifyWinEvent(event: DWORD, hwnd: HWND, idObject: LONG, idChild: LONG, flags: DWORD);
}

// gdi32 / user32 imports + own user32 exports (extern "system").
extern "system" {
    fn DrawFrameControl(hdc: HDC, rc: LPRECT, uType: u32, uState: u32) -> BOOL;
    fn GetClientRect(hWnd: HWND, lpRect: LPRECT) -> i32;
    fn FillRect(hDC: HDC, lprc: *const RECT, hbr: HBRUSH) -> i32;
    fn GetSysColorBrush(nIndex: i32) -> HBRUSH;
    fn GetSystemMetrics(nIndex: i32) -> i32;
    fn GetParent(hWnd: HWND) -> HWND;
    fn GetWindowLongW(hWnd: HWND, nIndex: core::ffi::c_int) -> i32;
    fn SendMessageW(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn PtInRect(lprc: *const RECT, pt: POINT) -> BOOL;
    fn ReleaseDC(hwnd: HWND, hdc: HDC) -> INT;
    fn CallMsgFilterW(lpMsg: *mut MSG, nCode: INT) -> BOOL;
    fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;
    fn GetCapture() -> HWND;
    fn GetMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT) -> BOOL;
    fn IsWindow(hWnd: HWND) -> BOOL;
    fn ReleaseCapture() -> BOOL;
    fn ScreenToClient(hWnd: HWND, lpPoint: *mut POINT) -> BOOL;
    fn TranslateMessage(lpMsg: *const MSG) -> BOOL;
    fn GetFocus() -> HWND;
    fn SetCaretPos(X: INT, Y: INT) -> BOOL;
    fn DestroyCaret() -> BOOL;
    fn DefWindowProcA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn SetLastError(dwErrCode: DWORD);
    fn OffsetRect(lprc: *mut RECT, dx: INT, dy: INT) -> BOOL;
    fn SelectObject(hdc: HDC, h: *mut c_void) -> *mut c_void;
    fn PatBlt(hdc: HDC, x: INT, y: INT, w: INT, h: INT, rop: DWORD) -> BOOL;
    fn DrawEdge(hdc: HDC, qrc: LPRECT, edge: UINT, grfFlags: UINT) -> BOOL;
    fn MulDiv(nNumber: INT, nNumerator: INT, nDenominator: INT) -> INT;
    fn KillSystemTimer(hWnd: HWND, uIDEvent: UINT_PTR) -> BOOL;
}

// ---- file-scope preamble (translated) ----
pub const SCROLL_NOWHERE: u32 = 0x00;
pub const SCROLL_TOP_ARROW: u32 = 0x01;
pub const SCROLL_TOP_RECT: u32 = 0x02;
pub const SCROLL_THUMB: u32 = 0x03;
pub const SCROLL_BOTTOM_RECT: u32 = 0x04;
pub const SCROLL_BOTTOM_ARROW: u32 = 0x05;
pub const SCROLL_FIRST_DELAY: u32 = 200;
pub const SCROLL_REPEAT_DELAY: u32 = 50;
pub const SCROLL_TIMER: u32 = 0;
pub const SCROLL_MIN_RECT: INT = 4;
pub const SCROLL_MIN_THUMB: INT = 6;
pub const SCROLL_ARROW_THUMB_OVERLAP: INT = 0;

static mut ScrollTrackingWin: HWND = 0 as HWND;
static mut ScrollTrackingBar: INT = 0;
static mut ScrollTrackingPos: INT = 0;
static mut ScrollTrackingVal: INT = 0;
static mut ScrollTrackHitTest: DWORD = SCROLL_NOWHERE;
static mut ScrollTrackVertical: BOOL = 0;
static mut ScrollMovingThumb: BOOL = 0;

/// A scrollbar CONTROL's `SBCALC`, which lives in the window's extra bytes (the
/// tail of `SBWND` past its `WND`). Never `pwnd as PSBWND`: under WoW64 `pwnd`
/// is a 32-bit SNAPSHOT of the 64-bit WND, so indexing past its end reads
/// whatever follows the snapshot, not the window. That made every guest
/// `GetScrollPos(SB_CTL)` return junk while `SetScrollPos` -- a syscall, which
/// the kernel serves from the real window -- worked (user32:scroll 161/170/179).
/// `wnd_extra_base` finds the real extra bytes; the SBWND tail is all 4-byte
/// fields, so its offset from the end of WND is the same on both sides.
#[inline]
unsafe fn SBCalcOf(pwnd: PWND) -> *mut SBCALC {
    crate::wow64::wnd_extra_base(pwnd)
        .add(core::mem::offset_of!(SBWND, SBCalc) - core::mem::size_of::<WND>())
        as *mut SBCALC
}

// `#[inline(never)]` restores the C's codegen shape, and is load-bearing (field manual
// §9.99, middle row). This returns NULL on two arms and every caller derefs the result
// unchecked — exactly as scrollbar.c does. Inlined, LLVM sees NULL reach a deref,
// concludes UB and folds the tail to `ud2`; out of line the caller gets an opaque
// pointer and emits the ordinary load, faulting where the C faults. The NULL arms are
// SB_CTL-with-short-cbwndExtra and a Bar outside SB_{HORZ,VERT,CTL}, and no caller here
// reaches either (GetScrollPos/GetScrollRange send SB_CTL to the window first), so this
// is a codegen fix and not a papered-over bug. The traps only appeared once ERR became
// a real call: silent, LLVM dropped the whole branch and the UB was invisible.
#[inline(never)]
unsafe fn IntGetSBData(pwnd: PWND, Bar: INT) -> PSBDATA {
    crate::wow64::wow64_unsupported("SBINFO");
    let pSBInfo: PSBINFO = DesktopPtrToUser((*pwnd).pSBInfo as *mut _) as PSBINFO;
    match Bar as u32 {
        SB_HORZ => {
            return core::ptr::addr_of_mut!((*pSBInfo).Horz);
        }
        SB_VERT => {
            return core::ptr::addr_of_mut!((*pSBInfo).Vert);
        }
        SB_CTL => {
            if ((*pwnd).cbwndExtra as usize)
                < (core::mem::size_of::<SBWND>() - core::mem::size_of::<WND>())
            {
                ERR!("IntGetSBData Wrong Extra bytes for CTL Scrollbar!\n");
                return core::ptr::null_mut();
            }
            return SBCalcOf(pwnd) as PSBDATA;
        }
        _ => {
            ERR!("IntGetSBData Bad Bar!\n");
        }
    }
    core::ptr::null_mut()
}

unsafe fn IntDrawScrollInterior(
    hWnd: HWND,
    hDC: HDC,
    nBar: INT,
    Vertical: BOOL,
    ScrollBarInfo: PSCROLLBARINFO,
) {
    let ThumbSize: INT = (*ScrollBarInfo).xyThumbBottom - (*ScrollBarInfo).xyThumbTop;
    let mut ThumbTop: INT = (*ScrollBarInfo).xyThumbTop;
    let mut Rect: RECT = core::mem::zeroed();
    let hSaveBrush: HBRUSH;
    let hBrush: HBRUSH;
    let mut TopSelected: BOOL = 0;
    let mut BottomSelected: BOOL = 0;

    if (*ScrollBarInfo).rgstate[SCROLL_TOP_RECT as usize] & STATE_SYSTEM_PRESSED != 0 {
        TopSelected = 1;
    }
    if (*ScrollBarInfo).rgstate[SCROLL_BOTTOM_RECT as usize] & STATE_SYSTEM_PRESSED != 0 {
        BottomSelected = 1;
    }

    if nBar == SB_CTL as INT {
        let b = GetControlBrush(hWnd, hDC, WM_CTLCOLORSCROLLBAR);
        if b.is_null() {
            hBrush = GetSysColorBrush(COLOR_SCROLLBAR as INT);
        } else {
            hBrush = b;
        }
    } else {
        hBrush = DefWndControlColor(hDC, CTLCOLOR_SCROLLBAR);
    }

    hSaveBrush = SelectObject(hDC, hBrush as _) as HBRUSH;

    if Vertical != 0 {
        Rect.top = (*ScrollBarInfo).rcScrollBar.top + (*ScrollBarInfo).dxyLineButton;
        Rect.bottom = (*ScrollBarInfo).rcScrollBar.bottom - (*ScrollBarInfo).dxyLineButton;
        Rect.left = (*ScrollBarInfo).rcScrollBar.left;
        Rect.right = (*ScrollBarInfo).rcScrollBar.right;
    } else {
        Rect.top = (*ScrollBarInfo).rcScrollBar.top;
        Rect.bottom = (*ScrollBarInfo).rcScrollBar.bottom;
        Rect.left = (*ScrollBarInfo).rcScrollBar.left + (*ScrollBarInfo).dxyLineButton;
        Rect.right = (*ScrollBarInfo).rcScrollBar.right - (*ScrollBarInfo).dxyLineButton;
    }

    if (*ScrollBarInfo).xyThumbBottom == 0 {
        PatBlt(
            hDC,
            Rect.left,
            Rect.top,
            Rect.right - Rect.left,
            Rect.bottom - Rect.top,
            PATCOPY,
        );

        SelectObject(hDC, hSaveBrush as _);
        return;
    }

    ThumbTop -= (*ScrollBarInfo).dxyLineButton;

    if (*ScrollBarInfo).dxyLineButton != 0 {
        if Vertical != 0 {
            if ThumbSize != 0 {
                PatBlt(
                    hDC,
                    Rect.left,
                    Rect.top,
                    Rect.right - Rect.left,
                    ThumbTop,
                    if TopSelected != 0 { BLACKNESS } else { PATCOPY },
                );
                Rect.top += ThumbTop;
                PatBlt(
                    hDC,
                    Rect.left,
                    Rect.top + ThumbSize,
                    Rect.right - Rect.left,
                    Rect.bottom - Rect.top - ThumbSize,
                    if BottomSelected != 0 { BLACKNESS } else { PATCOPY },
                );
                Rect.bottom = Rect.top + ThumbSize;
            } else {
                if ThumbTop != 0 {
                    PatBlt(
                        hDC,
                        Rect.left,
                        (*ScrollBarInfo).dxyLineButton,
                        Rect.right - Rect.left,
                        Rect.bottom - Rect.top,
                        PATCOPY,
                    );
                }
            }
        } else {
            if ThumbSize != 0 {
                PatBlt(
                    hDC,
                    Rect.left,
                    Rect.top,
                    ThumbTop,
                    Rect.bottom - Rect.top,
                    if TopSelected != 0 { BLACKNESS } else { PATCOPY },
                );
                Rect.left += ThumbTop;
                PatBlt(
                    hDC,
                    Rect.left + ThumbSize,
                    Rect.top,
                    Rect.right - Rect.left - ThumbSize,
                    Rect.bottom - Rect.top,
                    if BottomSelected != 0 { BLACKNESS } else { PATCOPY },
                );
                Rect.right = Rect.left + ThumbSize;
            } else {
                if ThumbTop != 0 {
                    PatBlt(
                        hDC,
                        (*ScrollBarInfo).dxyLineButton,
                        Rect.top,
                        Rect.right - Rect.left,
                        Rect.bottom - Rect.top,
                        PATCOPY,
                    );
                }
            }
        }
    }

    if ThumbSize != 0 {
        DrawEdge(
            hDC,
            core::ptr::addr_of_mut!(Rect),
            EDGE_RAISED,
            (BF_RECT | BF_MIDDLE) as UINT,
        );
    }

    SelectObject(hDC, hSaveBrush as _);
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntDrawScrollArrows(hDC: HDC, ScrollBarInfo: PSCROLLBARINFO, Vertical: BOOL) {
    IntDrawScrollArrows_impl(hDC, ScrollBarInfo, Vertical)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntDrawScrollArrows(hDC: HDC, ScrollBarInfo: PSCROLLBARINFO, Vertical: BOOL) {
    IntDrawScrollArrows_impl(hDC, ScrollBarInfo, Vertical)
}

unsafe fn IntDrawScrollArrows_impl(hDC: HDC, ScrollBarInfo: PSCROLLBARINFO, Vertical: BOOL) {

    let mut RectLT: RECT = core::mem::transmute_copy(&(*ScrollBarInfo).rcScrollBar);
    let mut RectRB: RECT = core::mem::transmute_copy(&(*ScrollBarInfo).rcScrollBar);
    let mut ScrollDirFlagLT: INT;
    let mut ScrollDirFlagRB: INT;

    if Vertical != 0 {
        ScrollDirFlagLT = DFCS_SCROLLUP as INT;
        ScrollDirFlagRB = DFCS_SCROLLDOWN as INT;
        RectLT.bottom = RectLT.top + (*ScrollBarInfo).dxyLineButton;
        RectRB.top = RectRB.bottom - (*ScrollBarInfo).dxyLineButton;
    } else {
        ScrollDirFlagLT = DFCS_SCROLLLEFT as INT;
        ScrollDirFlagRB = DFCS_SCROLLRIGHT as INT;
        RectLT.right = RectLT.left + (*ScrollBarInfo).dxyLineButton;
        RectRB.left = RectRB.right - (*ScrollBarInfo).dxyLineButton;
    }

    if (*ScrollBarInfo).rgstate[SCROLL_TOP_ARROW as usize] & STATE_SYSTEM_PRESSED != 0 {
        ScrollDirFlagLT |= (DFCS_PUSHED | DFCS_FLAT) as INT;
    }
    if (*ScrollBarInfo).rgstate[SCROLL_TOP_ARROW as usize] & STATE_SYSTEM_UNAVAILABLE != 0 {
        ScrollDirFlagLT |= DFCS_INACTIVE as INT;
    }
    if (*ScrollBarInfo).rgstate[SCROLL_BOTTOM_ARROW as usize] & STATE_SYSTEM_PRESSED != 0 {
        ScrollDirFlagRB |= (DFCS_PUSHED | DFCS_FLAT) as INT;
    }
    if (*ScrollBarInfo).rgstate[SCROLL_BOTTOM_ARROW as usize] & STATE_SYSTEM_UNAVAILABLE != 0 {
        ScrollDirFlagRB |= DFCS_INACTIVE as INT;
    }

    DrawFrameControl(hDC, core::ptr::addr_of_mut!(RectLT), DFC_SCROLL as u32, ScrollDirFlagLT as u32);
    DrawFrameControl(hDC, core::ptr::addr_of_mut!(RectRB), DFC_SCROLL as u32, ScrollDirFlagRB as u32);
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntScrollDrawMovingThumb(
    Dc: HDC,
    ScrollBarInfo: PSCROLLBARINFO,
    Vertical: BOOL,
) {
    IntScrollDrawMovingThumbImpl(Dc, ScrollBarInfo, Vertical);
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntScrollDrawMovingThumb(
    Dc: HDC,
    ScrollBarInfo: PSCROLLBARINFO,
    Vertical: BOOL,
) {
    IntScrollDrawMovingThumbImpl(Dc, ScrollBarInfo, Vertical);
}

unsafe fn IntScrollDrawMovingThumbImpl(
    Dc: HDC,
    ScrollBarInfo: PSCROLLBARINFO,
    Vertical: BOOL,
) {
    let mut pos_: INT = ScrollTrackingPos;
    let mut MaxSize: INT;
    let OldTop: INT;

    if Vertical != 0 {
        MaxSize = (*ScrollBarInfo).rcScrollBar.bottom - (*ScrollBarInfo).rcScrollBar.top;
    } else {
        MaxSize = (*ScrollBarInfo).rcScrollBar.right - (*ScrollBarInfo).rcScrollBar.left;
    }

    MaxSize -= (*ScrollBarInfo).dxyLineButton + (*ScrollBarInfo).xyThumbBottom - (*ScrollBarInfo).xyThumbTop;

    if pos_ < (*ScrollBarInfo).dxyLineButton {
        pos_ = (*ScrollBarInfo).dxyLineButton;
    } else if MaxSize < pos_ {
        pos_ = MaxSize;
    }

    OldTop = (*ScrollBarInfo).xyThumbTop;
    (*ScrollBarInfo).xyThumbBottom = pos_ + (*ScrollBarInfo).xyThumbBottom - (*ScrollBarInfo).xyThumbTop;
    (*ScrollBarInfo).xyThumbTop = pos_;
    IntDrawScrollInterior(ScrollTrackingWin, Dc, ScrollTrackingBar, Vertical, ScrollBarInfo);
    (*ScrollBarInfo).xyThumbBottom = OldTop + (*ScrollBarInfo).xyThumbBottom - (*ScrollBarInfo).xyThumbTop;
    (*ScrollBarInfo).xyThumbTop = OldTop;

    ScrollMovingThumb = if ScrollMovingThumb != 0 { 0 } else { 1 };
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntScrollGetObjectId(SBType: INT) -> LONG {
    if SBType == SB_VERT as INT {
        return OBJID_VSCROLL as LONG;
    }
    if SBType == SB_HORZ as INT {
        return OBJID_HSCROLL as LONG;
    }
    OBJID_CLIENT as LONG
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntScrollGetObjectId(SBType: INT) -> LONG {
    if SBType == SB_VERT as INT {
        return OBJID_VSCROLL as LONG;
    }
    if SBType == SB_HORZ as INT {
        return OBJID_HSCROLL as LONG;
    }
    OBJID_CLIENT as LONG
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGetScrollBarInfo(Wnd: HWND, Bar: INT, ScrollBarInfo: PSCROLLBARINFO) -> BOOL {
    (*ScrollBarInfo).cbSize = core::mem::size_of::<SCROLLBARINFO>() as UINT;

    NtUserGetScrollBarInfo(Wnd, IntScrollGetObjectId(Bar), ScrollBarInfo)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGetScrollBarInfo(Wnd: HWND, Bar: INT, ScrollBarInfo: PSCROLLBARINFO) -> BOOL {
    (*ScrollBarInfo).cbSize = core::mem::size_of::<SCROLLBARINFO>() as UINT;

    NtUserGetScrollBarInfo(Wnd, IntScrollGetObjectId(Bar), ScrollBarInfo)
}

fastcall_fn! {
unsafe fn IntUpdateScrollArrows(
    Wnd: HWND,
    hDC: HDC,
    ScrollBarInfo: PSCROLLBARINFO,
    info: *mut SETSCROLLBARINFO,
    SBType: INT,
    Arrow: INT,
    Vertical: BOOL,
    Pressed: BOOL,
) {
    if Pressed != 0 {
        (*ScrollBarInfo).rgstate[Arrow as usize] |= STATE_SYSTEM_PRESSED;
    } else {
        (*ScrollBarInfo).rgstate[Arrow as usize] &= !STATE_SYSTEM_PRESSED;
    }
    (*info).rgstate[Arrow as usize] = (*ScrollBarInfo).rgstate[Arrow as usize];
    NtUserSetScrollBarInfo(Wnd, IntScrollGetObjectId(SBType), info);

    IntDrawScrollArrows(hDC, ScrollBarInfo, Vertical);
}
}

unsafe fn IntDrawScrollBar(Wnd: HWND, DC: HDC, Bar: INT) {
    let mut Info: SCROLLBARINFO = core::mem::zeroed();
    let Vertical: BOOL;

    match Bar as u32 {
        SB_HORZ => {
            Vertical = 0;
        }
        SB_VERT => {
            Vertical = 1;
        }
        SB_CTL => {
            Vertical = ((GetWindowLongPtrW(Wnd, GWL_STYLE) & (SBS_VERT as LONG_PTR)) != 0) as BOOL;
        }
        _ => {
            return;
        }
    }

    if IntGetScrollBarInfo(Wnd, Bar, &mut Info) == 0 {
        return;
    }

    if IsRectEmpty(core::ptr::addr_of!(Info.rcScrollBar) as *const RECT) != 0 {
        return;
    }

    if Info.dxyLineButton != 0 {
        IntDrawScrollArrows(DC, &mut Info, Vertical);
    }

    IntDrawScrollInterior(Wnd, DC, Bar, Vertical, &mut Info);

    if Wnd == GetFocus() && SB_CTL as INT == Bar {
        if Vertical != 0 {
            SetCaretPos(Info.rcScrollBar.top + 1, Info.dxyLineButton + 1);
        } else {
            SetCaretPos(Info.dxyLineButton + 1, Info.rcScrollBar.top + 1);
        }
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntScrollPtInRectEx(Rect: LPRECT, Pt: POINT, Vertical: BOOL) -> BOOL {
    IntScrollPtInRectEx_impl(Rect, Pt, Vertical)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntScrollPtInRectEx(Rect: LPRECT, Pt: POINT, Vertical: BOOL) -> BOOL {
    IntScrollPtInRectEx_impl(Rect, Pt, Vertical)
}

unsafe fn IntScrollPtInRectEx_impl(Rect: LPRECT, Pt: POINT, Vertical: BOOL) -> BOOL {
    let mut TempRect: RECT = *Rect;
    let scrollbarWidth: i32;

    if Vertical != 0 {
        scrollbarWidth = (*Rect).right - (*Rect).left;
        TempRect.left -= scrollbarWidth * 8;
        TempRect.right += scrollbarWidth * 8;
        TempRect.top -= scrollbarWidth * 2;
        TempRect.bottom += scrollbarWidth * 2;
    } else {
        scrollbarWidth = (*Rect).bottom - (*Rect).top;
        TempRect.left -= scrollbarWidth * 2;
        TempRect.right += scrollbarWidth * 2;
        TempRect.top -= scrollbarWidth * 8;
        TempRect.bottom += scrollbarWidth * 8;
    }

    PtInRect(&TempRect, Pt)
}

fastcall_fn! {
unsafe fn IntScrollHitTest(ScrollBarInfo: PSCROLLBARINFO, Vertical: BOOL, mut Pt: POINT, Dragging: BOOL) -> DWORD {

    let ArrowSize: INT;
    let ThumbSize: INT;
    let ThumbPos: INT;

    if (Dragging != 0 && IntScrollPtInRectEx(core::ptr::addr_of_mut!((*ScrollBarInfo).rcScrollBar) as *mut RECT, Pt, Vertical) == 0)
        || PtInRect(core::ptr::addr_of_mut!((*ScrollBarInfo).rcScrollBar) as LPCRECT, Pt) == 0
    {
        return SCROLL_NOWHERE;
    }

    ThumbPos = (*ScrollBarInfo).xyThumbTop;
    ThumbSize = (*ScrollBarInfo).xyThumbBottom - ThumbPos;
    ArrowSize = (*ScrollBarInfo).dxyLineButton;

    if Vertical != 0 {
        if Pt.y < (*ScrollBarInfo).rcScrollBar.top + ArrowSize {
            return SCROLL_TOP_ARROW;
        }
        if Pt.y >= (*ScrollBarInfo).rcScrollBar.bottom - ArrowSize {
            return SCROLL_BOTTOM_ARROW;
        }
        if ThumbPos == 0 {
            return SCROLL_TOP_RECT;
        }
        Pt.y -= (*ScrollBarInfo).rcScrollBar.top;
        if Pt.y < ThumbPos {
            return SCROLL_TOP_RECT;
        }
        if Pt.y >= ThumbPos + ThumbSize {
            return SCROLL_BOTTOM_RECT;
        }
    } else {
        if Pt.x < (*ScrollBarInfo).rcScrollBar.left + ArrowSize {
            return SCROLL_TOP_ARROW;
        }
        if Pt.x >= (*ScrollBarInfo).rcScrollBar.right - ArrowSize {
            return SCROLL_BOTTOM_ARROW;
        }
        if ThumbPos == 0 {
            return SCROLL_TOP_RECT;
        }
        Pt.x -= (*ScrollBarInfo).rcScrollBar.left;
        if Pt.x < ThumbPos {
            return SCROLL_TOP_RECT;
        }
        if Pt.x >= ThumbPos + ThumbSize {
            return SCROLL_BOTTOM_RECT;
        }
    }

    SCROLL_THUMB
}
}

fastcall_fn! {
unsafe fn IntScrollGetScrollBarRect(
    Wnd: HWND,
    Bar: INT,
    Rect: *mut RECT,
    ArrowSize: *mut INT,
    ThumbSize: *mut INT,
    ThumbPos: *mut INT,
) -> BOOL {
    let Pixels: INT;
    let Vertical: BOOL;
    let pWnd: PWND;
    let pSBInfo: PSBINFO;
    let pSBData: PSBDATA;

    pWnd = ValidateHwnd(Wnd);
    if pWnd.is_null() {
        return 0;
    }
    crate::wow64::wow64_unsupported("SBINFO");
    pSBInfo = DesktopPtrToUser((*pWnd).pSBInfo as PVOID) as PSBINFO;

    *Rect = core::mem::transmute_copy(&(*pWnd).rcClient);
    OffsetRect(Rect, -(*pWnd).rcWindow.left, -(*pWnd).rcWindow.top);
    if (*pWnd).ExStyle & WS_EX_LAYOUTRTL != 0 {
        mirror_rect(core::ptr::addr_of_mut!((*pWnd).rcWindow) as *const RECT, Rect);
    }

    match Bar as u32 {
        SB_HORZ => {
            (*Rect).top = (*Rect).bottom;
            (*Rect).bottom += GetSystemMetrics(SM_CYHSCROLL as i32);
            if (*pWnd).style & WS_BORDER != 0 {
                (*Rect).left -= 1;
                (*Rect).right += 1;
            } else if (*pWnd).style & WS_VSCROLL != 0 {
                (*Rect).right += 1;
            }
            Vertical = 0;
            pSBData = core::ptr::addr_of_mut!((*pSBInfo).Horz);
        }
        SB_VERT => {
            if (*pWnd).ExStyle & WS_EX_LEFTSCROLLBAR != 0 {
                (*Rect).right = (*Rect).left;
                (*Rect).left -= GetSystemMetrics(SM_CXVSCROLL as i32);
            } else {
                (*Rect).left = (*Rect).right;
                (*Rect).right += GetSystemMetrics(SM_CXVSCROLL as i32);
            }
            if (*pWnd).style & WS_BORDER != 0 {
                (*Rect).top -= 1;
                (*Rect).bottom += 1;
            } else if (*pWnd).style & WS_HSCROLL != 0 {
                (*Rect).bottom += 1;
            }
            Vertical = 1;
            pSBData = core::ptr::addr_of_mut!((*pSBInfo).Vert);
        }
        SB_CTL => {
            GetClientRect(Wnd, Rect);
            Vertical = ((*pWnd).style & SBS_VERT) as BOOL;
            pSBData = SBCalcOf(pWnd) as PSBDATA;
        }
        _ => {
            return 0;
        }
    }

    if Vertical != 0 {
        Pixels = (*Rect).bottom - (*Rect).top;
    } else {
        Pixels = (*Rect).right - (*Rect).left;
    }

    let mut Pixels = Pixels;

    if Pixels <= 2 * GetSystemMetrics(SM_CXVSCROLL as i32) + SCROLL_MIN_RECT {
        if SCROLL_MIN_RECT < Pixels {
            *ArrowSize = (Pixels - SCROLL_MIN_RECT) / 2;
        } else {
            *ArrowSize = 0;
        }
        *ThumbPos = 0;
        *ThumbSize = 0;
    } else {
        *ArrowSize = GetSystemMetrics(SM_CXVSCROLL as i32);
        Pixels -= 2 * (GetSystemMetrics(SM_CXVSCROLL as i32) - SCROLL_ARROW_THUMB_OVERLAP);
        if (*pSBData).page != 0 {
            *ThumbSize = MulDiv(
                Pixels,
                (*pSBData).page,
                (*pSBData).posMax - (*pSBData).posMin + 1,
            );
            if *ThumbSize < SCROLL_MIN_THUMB {
                *ThumbSize = SCROLL_MIN_THUMB;
            }
        } else {
            *ThumbSize = GetSystemMetrics(SM_CXVSCROLL as i32);
        }

        Pixels -= *ThumbSize;
        if Pixels < 0
            || ((*pSBInfo).WSBflags & ESB_DISABLE_BOTH as i32) == ESB_DISABLE_BOTH as i32
        {
            *ThumbPos = 0;
            *ThumbSize = 0;
        } else {
            let Max = (*pSBData).posMax - max((*pSBData).page - 1, 0);
            if (*pSBData).posMin >= Max {
                *ThumbPos = *ArrowSize - SCROLL_ARROW_THUMB_OVERLAP;
            } else {
                *ThumbPos = *ArrowSize - SCROLL_ARROW_THUMB_OVERLAP
                    + MulDiv(
                        Pixels,
                        (*pSBData).pos - (*pSBData).posMin,
                        Max - (*pSBData).posMin,
                    );
            }
        }
    }
    Vertical
}
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntScrollGetThumbVal(
    Wnd: HWND,
    SBType: INT,
    ScrollBarInfo: PSCROLLBARINFO,
    Vertical: BOOL,
    mut pos_: INT,
) -> UINT {
    IntScrollGetThumbValImpl(Wnd, SBType, ScrollBarInfo, Vertical, pos_)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntScrollGetThumbVal(
    Wnd: HWND,
    SBType: INT,
    ScrollBarInfo: PSCROLLBARINFO,
    Vertical: BOOL,
    pos_: INT,
) -> UINT {
    IntScrollGetThumbValImpl(Wnd, SBType, ScrollBarInfo, Vertical, pos_)
}

unsafe fn IntScrollGetThumbValImpl(
    Wnd: HWND,
    SBType: INT,
    ScrollBarInfo: PSCROLLBARINFO,
    Vertical: BOOL,
    mut pos_: INT,
) -> UINT {
    let pWnd: PWND;
    let pSBData: PSBDATA;
    let mut Pixels: INT = if Vertical != 0 {
        (*ScrollBarInfo).rcScrollBar.bottom - (*ScrollBarInfo).rcScrollBar.top
    } else {
        (*ScrollBarInfo).rcScrollBar.right - (*ScrollBarInfo).rcScrollBar.left
    };

    pWnd = ValidateHwnd(Wnd);
    if pWnd.is_null() {
        return 0;
    }

    pSBData = IntGetSBData(pWnd, SBType);

    Pixels -= 2 * (*ScrollBarInfo).dxyLineButton;
    if Pixels <= 0 {
        return (*pSBData).posMin as UINT;
    }

    Pixels -= (*ScrollBarInfo).xyThumbBottom - (*ScrollBarInfo).xyThumbTop;
    if Pixels <= 0 {
        return (*pSBData).posMin as UINT;
    }

    pos_ = pos_ - (*ScrollBarInfo).dxyLineButton;
    if pos_ < 0 {
        pos_ = 0;
    }
    if pos_ > Pixels {
        pos_ = Pixels;
    }

    if (*pSBData).page == 0 {
        pos_ *= (*pSBData).posMax - (*pSBData).posMin;
    } else {
        pos_ *= (*pSBData).posMax - (*pSBData).posMin - (*pSBData).page as INT + 1;
    }

    ((*pSBData).posMin + ((pos_ + Pixels / 2) / Pixels)) as UINT
}

unsafe fn IntScrollClipPos(lpRect: PRECT, mut pt: POINT) -> POINT {
    if pt.x < (*lpRect).left {
        pt.x = (*lpRect).left;
    } else if pt.x > (*lpRect).right {
        pt.x = (*lpRect).right;
    }

    if pt.y < (*lpRect).top {
        pt.y = (*lpRect).top;
    } else if pt.y > (*lpRect).bottom {
        pt.y = (*lpRect).bottom;
    }

    pt
}

#[cfg(target_pointer_width = "64")]
unsafe extern "C" fn IntScrollDrawSizeGrip(Wnd: HWND, Dc: HDC) {
    IntScrollDrawSizeGrip_impl(Wnd, Dc)
}

#[cfg(target_pointer_width = "32")]
unsafe extern "fastcall" fn IntScrollDrawSizeGrip(Wnd: HWND, Dc: HDC) {
    IntScrollDrawSizeGrip_impl(Wnd, Dc)
}

#[inline]
unsafe fn IntScrollDrawSizeGrip_impl(Wnd: HWND, Dc: HDC) {

    let mut Rect: RECT = core::mem::zeroed();

    GetClientRect(Wnd, &mut Rect);
    FillRect(Dc, &Rect, GetSysColorBrush(COLOR_SCROLLBAR as i32));
    Rect.left = core::cmp::max(Rect.left, Rect.right - GetSystemMetrics(SM_CXVSCROLL as i32) - 1);
    Rect.top = core::cmp::max(Rect.top, Rect.bottom - GetSystemMetrics(SM_CYHSCROLL as i32) - 1);
    DrawFrameControl(Dc, &mut Rect, DFC_SCROLL, DFCS_SCROLLSIZEGRIP);
}

unsafe fn SCROLL_RefreshScrollBar(hwnd: HWND, nBar: INT, arrows: BOOL, interior: BOOL) {
    let hdc: HDC = GetDCEx(hwnd, core::ptr::null_mut(), DCX_CACHE | (if nBar == SB_CTL as INT { 0 } else { DCX_WINDOW }));
    if hdc.is_null() {
        return;
    }

    IntDrawScrollBar(hwnd, hdc, nBar);
    ReleaseDC(hwnd, hdc);
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntScrollHandleKbdEvent(
    Wnd: HWND,
    mut wParam: WPARAM,
    lParam: LPARAM,
) {
    IntScrollHandleKbdEventImpl(Wnd, wParam, lParam)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntScrollHandleKbdEvent(
    Wnd: HWND,
    mut wParam: WPARAM,
    lParam: LPARAM,
) {
    IntScrollHandleKbdEventImpl(Wnd, wParam, lParam)
}

unsafe fn IntScrollHandleKbdEventImpl(Wnd: HWND, mut wParam: WPARAM, lParam: LPARAM) {

    if 0 == (lParam & PFD_DOUBLEBUFFER_DONTCARE as LPARAM) {
        HideCaret(Wnd);
    }

    match wParam as u32 {
        VK_PRIOR => wParam = SB_PAGEUP as WPARAM,
        VK_NEXT => wParam = SB_PAGEDOWN as WPARAM,
        VK_HOME => wParam = SB_TOP as WPARAM,
        VK_END => wParam = SB_BOTTOM as WPARAM,
        VK_UP => wParam = SB_LINEUP as WPARAM,
        VK_DOWN => wParam = SB_LINEDOWN as WPARAM,
        _ => return,
    }

    SendMessageW(
        GetParent(Wnd),
        if 0 != (GetWindowLongPtrW(Wnd, GWL_STYLE as core::ffi::c_int) & (SBS_VERT as LONG_PTR)) {
            WM_VSCROLL
        } else {
            WM_HSCROLL
        },
        wParam,
        Wnd as LPARAM,
    );
}

fastcall_fn! {
unsafe fn IntScrollHandleScrollEvent(Wnd: HWND, SBType: INT, Msg: UINT, mut Pt: POINT) {
    static mut PrevPt: POINT = POINT { x: 0, y: 0 };
    static mut TrackThumbPos: UINT = 0;
    static mut LastClickPos: INT = 0;
    static mut LastMousePos: INT = 0;

    let mut HitTest: DWORD;
    let WndOwner: HWND;
    let WndCtl: HWND;
    let Vertical: BOOL;
    let Dc: HDC;
    let mut ScrollBarInfo: SCROLLBARINFO = core::mem::zeroed();
    let mut NewInfo: SETSCROLLBARINFO = core::mem::zeroed();

    if IntGetScrollBarInfo(Wnd, SBType, &mut ScrollBarInfo) == 0 {
        return;
    }
    if (ScrollTrackHitTest == SCROLL_NOWHERE) && (Msg != WM_LBUTTONDOWN) {
        if Wnd == GetCapture() {
            ReleaseCapture();
        }
        return;
    }

    NewInfo.nTrackPos = ScrollTrackingVal;
    NewInfo.reserved = ScrollBarInfo.reserved;
    core::ptr::copy_nonoverlapping(
        ScrollBarInfo.rgstate.as_ptr(),
        NewInfo.rgstate.as_mut_ptr(),
        (CCHILDREN_SCROLLBAR + 1) as usize,
    );

    if SBType == SB_CTL as INT
        && (GetWindowLongPtrW(Wnd, GWL_STYLE) & (((SBS_SIZEGRIP | SBS_SIZEBOX) as LONG_PTR))) != 0
    {
        match Msg {
            WM_LBUTTONDOWN => {
                HideCaret(Wnd);
                SetCapture(Wnd);
                PrevPt = Pt;
                ScrollTrackHitTest = SCROLL_THUMB;
            }
            WM_MOUSEMOVE => {
                GetClientRect(GetParent(GetParent(Wnd)), core::ptr::addr_of_mut!(ScrollBarInfo.rcScrollBar) as *mut RECT);
                PrevPt = Pt;
            }
            WM_LBUTTONUP => {
                ReleaseCapture();
                ScrollTrackHitTest = SCROLL_NOWHERE;
                if Wnd == GetFocus() {
                    ShowCaret(Wnd);
                }
            }
            WM_SYSTIMER => {
                Pt = PrevPt;
            }
            _ => {}
        }
        return;
    }

    Dc = GetDCEx(
        Wnd,
        core::ptr::null_mut(),
        DCX_CACHE | (if SB_CTL as INT == SBType { 0 } else { DCX_WINDOW }),
    );
    if SB_VERT as INT == SBType {
        Vertical = 1;
    } else if SB_HORZ as INT == SBType {
        Vertical = 0;
    } else {
        Vertical = (0 != (GetWindowLongPtrW(Wnd, GWL_STYLE) & ((SBS_VERT as LONG_PTR)))) as BOOL;
    }
    WndOwner = if SB_CTL as INT == SBType { GetParent(Wnd) } else { Wnd };
    WndCtl = if SB_CTL as INT == SBType { Wnd } else { core::ptr::null_mut() };

    match Msg {
        WM_LBUTTONDOWN => {
            HideCaret(Wnd);
            ScrollTrackVertical = Vertical;
            ScrollTrackHitTest = IntScrollHitTest(&mut ScrollBarInfo, Vertical, Pt, 0);
            HitTest = ScrollTrackHitTest;
            LastClickPos = if Vertical != 0 {
                Pt.y - ScrollBarInfo.rcScrollBar.top
            } else {
                Pt.x - ScrollBarInfo.rcScrollBar.left
            };
            LastMousePos = LastClickPos;
            TrackThumbPos = ScrollBarInfo.xyThumbTop as UINT;
            PrevPt = Pt;
            if SBType == SB_CTL as INT
                && (GetWindowLongPtrW(Wnd, GWL_STYLE) & ((WS_TABSTOP as LONG_PTR))) != 0
            {
                SetFocus(Wnd);
            }
            SetCapture(Wnd);
            if ScrollBarInfo.rgstate[ScrollTrackHitTest as usize] != STATE_SYSTEM_UNAVAILABLE {
                IntUpdateScrollArrows(
                    Wnd, Dc, &mut ScrollBarInfo, &mut NewInfo, SBType,
                    ScrollTrackHitTest as INT, Vertical, 1,
                );
            }
        }
        WM_MOUSEMOVE => {
            HitTest = IntScrollHitTest(&mut ScrollBarInfo, Vertical, Pt, 1);
            PrevPt = Pt;
        }
        WM_LBUTTONUP => {
            HitTest = SCROLL_NOWHERE;
            ReleaseCapture();
            if Wnd == GetFocus() {
                ShowCaret(Wnd);
            }
            if ScrollBarInfo.rgstate[ScrollTrackHitTest as usize] != STATE_SYSTEM_UNAVAILABLE {
                IntUpdateScrollArrows(
                    Wnd, Dc, &mut ScrollBarInfo, &mut NewInfo, SBType,
                    ScrollTrackHitTest as INT, Vertical, 0,
                );
                IntDrawScrollInterior(Wnd, Dc, SBType, Vertical, &mut ScrollBarInfo);
            }
        }
        WM_SYSTIMER => {
            Pt = PrevPt;
            HitTest = IntScrollHitTest(&mut ScrollBarInfo, Vertical, Pt, 0);
        }
        _ => {
            return;
        }
    }

    match ScrollTrackHitTest {
        SCROLL_NOWHERE => {}
        SCROLL_TOP_ARROW => {
            if HitTest == ScrollTrackHitTest {
                if (WM_LBUTTONDOWN == Msg) || (WM_SYSTIMER == Msg) {
                    SendMessageW(
                        WndOwner,
                        if Vertical != 0 { WM_VSCROLL } else { WM_HSCROLL },
                        SB_LINEUP as WPARAM,
                        WndCtl as LPARAM,
                    );
                }
                SetSystemTimer(
                    Wnd,
                    SCROLL_TIMER as UINT_PTR,
                    if WM_LBUTTONDOWN == Msg { SCROLL_FIRST_DELAY } else { SCROLL_REPEAT_DELAY },
                    None,
                );
                if ScrollBarInfo.rgstate[ScrollTrackHitTest as usize] != STATE_SYSTEM_UNAVAILABLE {
                    ScrollBarInfo.rgstate[ScrollTrackHitTest as usize] &= STATE_SYSTEM_PRESSED;
                    if ScrollBarInfo.rgstate[ScrollTrackHitTest as usize] == 0 {
                        IntUpdateScrollArrows(
                            Wnd, Dc, &mut ScrollBarInfo, &mut NewInfo, SBType,
                            ScrollTrackHitTest as INT, Vertical, 1,
                        );
                    }
                }
            } else {
                IntUpdateScrollArrows(
                    Wnd, Dc, &mut ScrollBarInfo, &mut NewInfo, SBType,
                    ScrollTrackHitTest as INT, Vertical, 0,
                );
                KillSystemTimer(Wnd, SCROLL_TIMER as UINT_PTR);
            }
        }
        SCROLL_TOP_RECT => {
            if HitTest == ScrollTrackHitTest {
                if (WM_LBUTTONDOWN == Msg) || (WM_SYSTIMER == Msg) {
                    SendMessageW(
                        WndOwner,
                        if Vertical != 0 { WM_VSCROLL } else { WM_HSCROLL },
                        SB_PAGEUP as WPARAM,
                        WndCtl as LPARAM,
                    );
                }
                SetSystemTimer(
                    Wnd,
                    SCROLL_TIMER as UINT_PTR,
                    if WM_LBUTTONDOWN == Msg { SCROLL_FIRST_DELAY } else { SCROLL_REPEAT_DELAY },
                    None,
                );
            } else {
                KillSystemTimer(Wnd, SCROLL_TIMER as UINT_PTR);
            }
        }
        SCROLL_THUMB => {
            if Msg == WM_LBUTTONDOWN {
                ScrollTrackingWin = Wnd;
                ScrollTrackingBar = SBType;
                ScrollTrackingPos = TrackThumbPos as INT + LastMousePos - LastClickPos;
                ScrollTrackingVal = IntScrollGetThumbVal(
                    Wnd, SBType, &mut ScrollBarInfo, Vertical, ScrollTrackingPos,
                ) as INT;
                NewInfo.nTrackPos = ScrollTrackingVal;
                NtUserSetScrollBarInfo(Wnd, IntScrollGetObjectId(SBType), &mut NewInfo);
                IntScrollDrawMovingThumb(Dc, &mut ScrollBarInfo, Vertical);
            } else if Msg == WM_LBUTTONUP {
                ScrollTrackingWin = 0 as HWND;
                ScrollTrackingVal = 0;
                IntDrawScrollInterior(Wnd, Dc, SBType, Vertical, &mut ScrollBarInfo);
            } else {
                let pos_: UINT;
                if IntScrollPtInRectEx(core::ptr::addr_of_mut!(ScrollBarInfo.rcScrollBar) as *mut RECT, Pt, Vertical) == 0 {
                    pos_ = LastClickPos as UINT;
                } else {
                    Pt = IntScrollClipPos(core::ptr::addr_of_mut!(ScrollBarInfo.rcScrollBar) as *mut RECT, Pt);
                    pos_ = if Vertical != 0 {
                        (Pt.y - ScrollBarInfo.rcScrollBar.top) as UINT
                    } else {
                        (Pt.x - ScrollBarInfo.rcScrollBar.left) as UINT
                    };
                }
                if pos_ != LastMousePos as UINT || ScrollMovingThumb == 0 {
                    LastMousePos = pos_ as INT;
                    ScrollTrackingPos = TrackThumbPos as INT + pos_ as INT - LastClickPos;
                    ScrollTrackingVal = IntScrollGetThumbVal(
                        Wnd, SBType, &mut ScrollBarInfo, Vertical, ScrollTrackingPos,
                    ) as INT;
                    NewInfo.nTrackPos = ScrollTrackingVal;
                    NtUserSetScrollBarInfo(Wnd, IntScrollGetObjectId(SBType), &mut NewInfo);
                    IntScrollDrawMovingThumb(Dc, &mut ScrollBarInfo, Vertical);
                    SendMessageW(
                        WndOwner,
                        if Vertical != 0 { WM_VSCROLL } else { WM_HSCROLL },
                        MAKEWPARAM(SB_THUMBTRACK as u16, ScrollTrackingVal as u16),
                        WndCtl as LPARAM,
                    );
                }
            }
        }
        SCROLL_BOTTOM_RECT => {
            if HitTest == ScrollTrackHitTest {
                if (Msg == WM_LBUTTONDOWN) || (Msg == WM_SYSTIMER) {
                    SendMessageW(
                        WndOwner,
                        if Vertical != 0 { WM_VSCROLL } else { WM_HSCROLL },
                        SB_PAGEDOWN as WPARAM,
                        WndCtl as LPARAM,
                    );
                }
                SetSystemTimer(
                    Wnd,
                    SCROLL_TIMER as UINT_PTR,
                    if WM_LBUTTONDOWN == Msg { SCROLL_FIRST_DELAY } else { SCROLL_REPEAT_DELAY },
                    None,
                );
            } else {
                KillSystemTimer(Wnd, SCROLL_TIMER as UINT_PTR);
            }
        }
        SCROLL_BOTTOM_ARROW => {
            if HitTest == ScrollTrackHitTest {
                if (Msg == WM_LBUTTONDOWN) || (Msg == WM_SYSTIMER) {
                    SendMessageW(
                        WndOwner,
                        if Vertical != 0 { WM_VSCROLL } else { WM_HSCROLL },
                        SB_LINEDOWN as WPARAM,
                        WndCtl as LPARAM,
                    );
                }
                SetSystemTimer(
                    Wnd,
                    SCROLL_TIMER as UINT_PTR,
                    if WM_LBUTTONDOWN == Msg { SCROLL_FIRST_DELAY } else { SCROLL_REPEAT_DELAY },
                    None,
                );
                if ScrollBarInfo.rgstate[ScrollTrackHitTest as usize] != STATE_SYSTEM_UNAVAILABLE {
                    ScrollBarInfo.rgstate[ScrollTrackHitTest as usize] &= STATE_SYSTEM_PRESSED;
                    if ScrollBarInfo.rgstate[ScrollTrackHitTest as usize] == 0 {
                        IntUpdateScrollArrows(
                            Wnd, Dc, &mut ScrollBarInfo, &mut NewInfo, SBType,
                            ScrollTrackHitTest as INT, Vertical, 1,
                        );
                    }
                }
            } else {
                IntUpdateScrollArrows(
                    Wnd, Dc, &mut ScrollBarInfo, &mut NewInfo, SBType,
                    ScrollTrackHitTest as INT, Vertical, 0,
                );
                KillSystemTimer(Wnd, SCROLL_TIMER as UINT_PTR);
            }
        }
        _ => {}
    }

    if Msg == WM_LBUTTONDOWN {
        if SCROLL_THUMB == HitTest {
            let Val = IntScrollGetThumbVal(
                Wnd, SBType, &mut ScrollBarInfo, Vertical,
                TrackThumbPos as INT + LastMousePos - LastClickPos,
            );
            SendMessageW(
                WndOwner,
                if Vertical != 0 { WM_VSCROLL } else { WM_HSCROLL },
                MAKEWPARAM(SB_THUMBTRACK as u16, Val as u16),
                WndCtl as LPARAM,
            );
        }
    }

    if Msg == WM_LBUTTONUP {
        HitTest = ScrollTrackHitTest;
        ScrollTrackHitTest = SCROLL_NOWHERE;

        if SCROLL_THUMB == HitTest {
            let Val = IntScrollGetThumbVal(
                Wnd, SBType, &mut ScrollBarInfo, Vertical,
                TrackThumbPos as INT + LastMousePos - LastClickPos,
            );
            SendMessageW(
                WndOwner,
                if Vertical != 0 { WM_VSCROLL } else { WM_HSCROLL },
                MAKEWPARAM(SB_THUMBPOSITION as u16, Val as u16),
                WndCtl as LPARAM,
            );
        }
        SendMessageW(
            WndOwner,
            if Vertical != 0 { WM_VSCROLL } else { WM_HSCROLL },
            SB_ENDSCROLL as WPARAM,
            WndCtl as LPARAM,
        );

        ScrollTrackingWin = 0 as HWND;
    }

    ReleaseDC(Wnd, Dc);
}
}

unsafe fn IntScrollCreateScrollBar(Wnd: HWND, lpCreate: LPCREATESTRUCTW) {

    let mut Info: SCROLLINFO = core::mem::zeroed();

    Info.cbSize = core::mem::size_of::<SCROLLINFO>() as u32;
    Info.fMask = (SIF_RANGE | SIF_PAGE | SIF_POS) as u32;
    Info.nMin = 0;
    Info.nMax = 0;
    Info.nPage = 0;
    Info.nPos = 0;
    Info.nTrackPos = 0;
    NtUserSetScrollInfo(Wnd, SB_CTL as core::ffi::c_int, &mut Info, 0i32);

    if 0 != ((*lpCreate).style & (SBS_SIZEGRIP | SBS_SIZEBOX) as i32) {
        if 0 != ((*lpCreate).style & SBS_SIZEBOXTOPLEFTALIGN as i32) {
            MoveWindow(
                Wnd,
                (*lpCreate).x,
                (*lpCreate).y,
                GetSystemMetrics(SM_CXVSCROLL as core::ffi::c_int) + 1,
                GetSystemMetrics(SM_CYHSCROLL as core::ffi::c_int) + 1,
                0,
            );
        } else if 0 != ((*lpCreate).style & SBS_SIZEBOXBOTTOMRIGHTALIGN as i32) {
            MoveWindow(
                Wnd,
                (*lpCreate).x + (*lpCreate).cx - GetSystemMetrics(SM_CXVSCROLL as core::ffi::c_int) - 1,
                (*lpCreate).y + (*lpCreate).cy - GetSystemMetrics(SM_CYHSCROLL as core::ffi::c_int) - 1,
                GetSystemMetrics(SM_CXVSCROLL as core::ffi::c_int) + 1,
                GetSystemMetrics(SM_CYHSCROLL as core::ffi::c_int) + 1,
                0,
            );
        }
    } else if 0 != ((*lpCreate).style & SBS_VERT as i32) {
        if 0 != ((*lpCreate).style & SBS_LEFTALIGN as i32) {
            MoveWindow(
                Wnd,
                (*lpCreate).x,
                (*lpCreate).y,
                GetSystemMetrics(SM_CXVSCROLL as core::ffi::c_int) + 1,
                (*lpCreate).cy,
                0,
            );
        } else if 0 != ((*lpCreate).style & SBS_RIGHTALIGN as i32) {
            MoveWindow(
                Wnd,
                (*lpCreate).x + (*lpCreate).cx - GetSystemMetrics(SM_CXVSCROLL as core::ffi::c_int) - 1,
                (*lpCreate).y,
                GetSystemMetrics(SM_CXVSCROLL as core::ffi::c_int) + 1,
                (*lpCreate).cy,
                0,
            );
        }
    } else {
        if 0 != ((*lpCreate).style & SBS_TOPALIGN as i32) {
            MoveWindow(
                Wnd,
                (*lpCreate).x,
                (*lpCreate).y,
                (*lpCreate).cx,
                GetSystemMetrics(SM_CYHSCROLL as core::ffi::c_int) + 1,
                0,
            );
        } else if 0 != ((*lpCreate).style & SBS_BOTTOMALIGN as i32) {
            MoveWindow(
                Wnd,
                (*lpCreate).x,
                (*lpCreate).y + (*lpCreate).cy - GetSystemMetrics(SM_CYHSCROLL as core::ffi::c_int) - 1,
                (*lpCreate).cx,
                GetSystemMetrics(SM_CYHSCROLL as core::ffi::c_int) + 1,
                0,
            );
        }
    }
}

// Cross-TU FASTCALL symbol: the (Rust) defwnd.rs declares `ScrollTrackScrollBar`
// (extern "fastcall" on i386, "C" elsewhere) and calls it. Export with matching
// decoration (i386 → `@ScrollTrackScrollBar@12`).
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ScrollTrackScrollBar(Wnd: HWND, SBType: INT, Pt: POINT) {
    ScrollTrackScrollBar_impl(Wnd, SBType, Pt)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ScrollTrackScrollBar(Wnd: HWND, SBType: INT, Pt: POINT) {
    ScrollTrackScrollBar_impl(Wnd, SBType, Pt)
}

unsafe fn ScrollTrackScrollBar_impl(Wnd: HWND, SBType: INT, mut Pt: POINT) {

    let mut Msg: MSG = core::mem::zeroed();
    let mut XOffset: UINT = 0;
    let mut YOffset: UINT = 0;

    if SBType != SB_CTL as INT {
        let pwnd = ValidateHwnd(Wnd);
        if pwnd.is_null() {
            return;
        }
        XOffset = ((*pwnd).rcClient.left - (*pwnd).rcWindow.left) as UINT;
        YOffset = ((*pwnd).rcClient.top - (*pwnd).rcWindow.top) as UINT;
        ScreenToClient(Wnd, &mut Pt);
        Pt.x += XOffset as LONG;
        Pt.y += YOffset as LONG;
    }

    IntScrollHandleScrollEvent(Wnd, SBType, WM_LBUTTONDOWN, Pt);

    loop {
        if GetMessageW(&mut Msg, core::ptr::null_mut(), 0, 0) == 0 {
            break;
        }
        if CallMsgFilterW(&mut Msg, MSGF_SCROLLBAR as INT) != 0 {
            // C `continue` in `do { } while (Msg.message != WM_LBUTTONUP && GetCapture() ==
            // Wnd)` still tests that condition: a filtered button-up or lost capture ends
            // tracking. A bare `continue` skipped it and kept tracking.
            if Msg.message == WM_LBUTTONUP || GetCapture() != Wnd {
                break;
            }
            continue;
        }
        if Msg.message == WM_LBUTTONUP
            || Msg.message == WM_MOUSEMOVE
            || (Msg.message == WM_SYSTIMER && Msg.wParam == SCROLL_TIMER as WPARAM)
        {
            Pt.x = (LOWORD(Msg.lParam as DWORD) as LONG) + XOffset as LONG;
            Pt.y = (HIWORD(Msg.lParam as DWORD) as LONG) + YOffset as LONG;
            IntScrollHandleScrollEvent(Wnd, SBType, Msg.message, Pt);
        } else {
            TranslateMessage(&Msg);
            DispatchMessageW(&Msg);
        }
        if IsWindow(Wnd) == 0 {
            ReleaseCapture();
            break;
        }
        if Msg.message == WM_LBUTTONUP || GetCapture() != Wnd {
            break;
        }
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntSetScrollInfo(Wnd: HWND, Info: LPCSCROLLINFO, bRedraw: BOOL) -> DWORD {
    let Ret = NtUserSetScrollInfo(Wnd, SB_CTL as i32, Info, bRedraw);
    if Ret != 0 {
        IntNotifyWinEvent(EVENT_OBJECT_VALUECHANGE as u32, Wnd, OBJID_CLIENT as i32, CHILDID_SELF as i32, WEF_SETBYWNDPTI as u32);
    }
    Ret
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntSetScrollInfo(Wnd: HWND, Info: LPCSCROLLINFO, bRedraw: BOOL) -> DWORD {
    let Ret = NtUserSetScrollInfo(Wnd, SB_CTL as i32, Info, bRedraw);
    if Ret != 0 {
        IntNotifyWinEvent(EVENT_OBJECT_VALUECHANGE as u32, Wnd, OBJID_CLIENT as i32, CHILDID_SELF as i32, WEF_SETBYWNDPTI as u32);
    }
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn ScrollBarWndProc_common(
    DefWindowProc: WNDPROC,
    Wnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    unicode: BOOL,
) -> LRESULT {
    let pWnd: PWND;
    let pSBCalc: *mut SBCALC;
    let mut ScrollInfo: SCROLLINFO = core::mem::zeroed();

    pWnd = ValidateHwnd(Wnd);
    if !pWnd.is_null() {
        if (*pWnd).fnid == 0 {
            TRACE!(b"ScrollBar CTL size %d\n\0".as_ptr(),
                  (core::mem::size_of::<SBWND>() - core::mem::size_of::<WND>()) as INT);
            if ((*pWnd).cbwndExtra as usize) < (core::mem::size_of::<SBWND>() - core::mem::size_of::<WND>()) {
                ERR!("Wrong Extra bytes for Scrollbar!\n");
                return 0;
            }

            if Msg != WM_CREATE {
                return DefWindowProc.unwrap()(Wnd, Msg, wParam, lParam);
            }
            NtUserSetWindowFNID(Wnd, FNID_SCROLLBAR as u16);
        } else {
            if (*pWnd).fnid != FNID_SCROLLBAR {
                ERR!("Wrong window class for Scrollbar!\n");
                return 0;
            }
        }
    }

    if IsWindow(Wnd) == 0 {
        return 0;
    }

    pSBCalc = SBCalcOf(pWnd);

    match Msg {
        WM_CREATE => {
            IntScrollCreateScrollBar(Wnd, lParam as LPCREATESTRUCTW);
        }

        WM_ENABLE => {
            return SendMessageW(Wnd, SBM_ENABLE_ARROWS,
                if wParam != 0 { ESB_ENABLE_BOTH as WPARAM } else { ESB_DISABLE_BOTH as WPARAM }, 0);
        }

        WM_LBUTTONDBLCLK | WM_LBUTTONDOWN => {
            if GetWindowLongW(Wnd, GWL_STYLE) & SBS_SIZEGRIP as i32 != 0 {
                SendMessageW(GetParent(Wnd), WM_SYSCOMMAND,
                    (SC_SIZE + (if GetWindowLongW(Wnd, GWL_EXSTYLE) & WS_EX_LAYOUTRTL as i32 != 0 {
                        WMSZ_BOTTOMLEFT
                    } else {
                        WMSZ_BOTTOMRIGHT
                    })) as WPARAM, lParam);
            } else {
                let mut Pt: POINT = core::mem::zeroed();
                Pt.x = (LOWORD(lParam as u32) as i16) as LONG;
                Pt.y = (HIWORD(lParam as u32) as i16) as LONG;
                ScrollTrackScrollBar(Wnd, SB_CTL as INT, Pt);
            }
        }

        WM_LBUTTONUP | WM_MOUSEMOVE | WM_SYSTIMER => {
            let mut Pt: POINT = core::mem::zeroed();
            Pt.x = (LOWORD(lParam as u32) as i16) as LONG;
            Pt.y = (HIWORD(lParam as u32) as i16) as LONG;
            IntScrollHandleScrollEvent(Wnd, SB_CTL as INT, Msg, Pt);
        }

        WM_KEYDOWN => {
            IntScrollHandleKbdEvent(Wnd, wParam, lParam);
        }

        WM_KEYUP => {
            ShowCaret(Wnd);
        }

        WM_SETFOCUS => {
            let mut Rect: RECT = core::mem::zeroed();
            let mut ArrowSize: INT = 0;
            let mut ThumbSize: INT = 0;
            let mut ThumbPos: INT = 0;
            let Vertical: INT;

            Vertical = IntScrollGetScrollBarRect(Wnd, SB_CTL as INT, &mut Rect,
                &mut ArrowSize, &mut ThumbSize, &mut ThumbPos);
            if Vertical == 0 {
                CreateCaret(Wnd, 1 as HBITMAP, ThumbSize - 2, Rect.bottom - Rect.top - 2);
                SetCaretPos(ThumbPos + 1, Rect.top + 1);
            } else {
                CreateCaret(Wnd, 1 as HBITMAP, Rect.right - Rect.left - 2, ThumbSize - 2);
                SetCaretPos(Rect.top + 1, ThumbPos + 1);
            }
            ShowCaret(Wnd);
        }

        WM_KILLFOCUS => {
            let mut Rect: RECT = core::mem::zeroed();
            let mut ArrowSize: INT = 0;
            let mut ThumbSize: INT = 0;
            let mut ThumbPos: INT = 0;
            let Vertical: INT;

            Vertical = IntScrollGetScrollBarRect(Wnd, SB_CTL as INT, &mut Rect,
                &mut ArrowSize, &mut ThumbSize, &mut ThumbPos);
            if Vertical == 0 {
                Rect.left = ThumbPos + 1;
                Rect.right = Rect.left + ThumbSize;
            } else {
                Rect.top = ThumbPos + 1;
                Rect.bottom = Rect.top + ThumbSize;
            }
            HideCaret(Wnd);
            InvalidateRect(Wnd, &mut Rect, 0);
            DestroyCaret();
        }

        WM_ERASEBKGND => {
            return 1;
        }

        WM_GETDLGCODE => {
            return DLGC_WANTARROWS as LRESULT;
        }

        WM_PAINT => {
            let mut Ps: PAINTSTRUCT = core::mem::zeroed();
            let Dc: HDC;

            Dc = if wParam != 0 { wParam as HDC } else { BeginPaint(Wnd, &mut Ps) };

            if GetWindowLongPtrW(Wnd, GWL_STYLE) & (SBS_SIZEGRIP as LONG_PTR) != 0 {
                IntScrollDrawSizeGrip(Wnd, Dc);
            } else if GetWindowLongPtrW(Wnd, GWL_STYLE) & (SBS_SIZEBOX as LONG_PTR) != 0 {
                let mut Rect: RECT = core::mem::zeroed();
                GetClientRect(Wnd, &mut Rect);
                FillRect(Dc, &mut Rect, GetSysColorBrush(COLOR_SCROLLBAR as i32));
            } else {
                IntDrawScrollBar(Wnd, Dc, SB_CTL as INT);
            }

            if wParam == 0 {
                EndPaint(Wnd, &mut Ps);
            }
        }

        SBM_GETPOS => {
            return (*pSBCalc).pos as LRESULT;
        }

        SBM_GETRANGE => {
            *(wParam as LPINT) = (*pSBCalc).posMin;
            *(lParam as LPINT) = (*pSBCalc).posMax;
            return 0;
        }

        SBM_ENABLE_ARROWS => {
            return EnableScrollBar(Wnd, SB_CTL as UINT, wParam as UINT) as LRESULT;
        }

        SBM_SETPOS => {
            ScrollInfo.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
            ScrollInfo.fMask = (SIF_POS | SIF_PREVIOUSPOS) as UINT;
            ScrollInfo.nPos = wParam as INT;
            return IntSetScrollInfo(Wnd, &mut ScrollInfo, lParam as INT) as LRESULT;
        }

        SBM_SETRANGEREDRAW | SBM_SETRANGE => {
            ScrollInfo.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
            ScrollInfo.fMask = (SIF_RANGE | SIF_PREVIOUSPOS) as UINT;
            ScrollInfo.nMin = wParam as INT;
            ScrollInfo.nMax = lParam as INT;
            return IntSetScrollInfo(Wnd, &mut ScrollInfo,
                if Msg == SBM_SETRANGEREDRAW { 1 } else { 0 }) as LRESULT;
        }

        SBM_SETSCROLLINFO => {
            return IntSetScrollInfo(Wnd, lParam as LPCSCROLLINFO, wParam as INT) as LRESULT;
        }

        SBM_GETSCROLLINFO => {
            let pSBData: PSBDATA = pSBCalc as PSBDATA;
            let ret: DWORD = NtUserSBGetParms(Wnd, SB_CTL as INT, pSBData, lParam as *mut SCROLLINFO) as DWORD;
            if ret == 0 {
                ERR!("SBM_GETSCROLLINFO No ScrollInfo\n");
            }
            return ret as LRESULT;
        }

        SBM_GETSCROLLBARINFO => {
            (*(lParam as PSCROLLBARINFO)).cbSize = core::mem::size_of::<SCROLLBARINFO>() as DWORD;
            return NtUserGetScrollBarInfo(Wnd, OBJID_CLIENT as LONG, lParam as PSCROLLBARINFO) as LRESULT;
        }

        0x00e5 | 0x00e7 | 0x00e8 | 0x00ec | 0x00ed | 0x00ee | 0x00ef => {
            WARN!(b"unknown Win32 msg %04x wp=%08lx lp=%08lx\n\0".as_ptr(), Msg, wParam, lParam);
        }

        _ => {
            if WM_USER <= Msg {
                WARN!(b"unknown msg %04x wp=%04lx lp=%08lx\n\0".as_ptr(), Msg, wParam, lParam);
            }
            if unicode != 0 {
                return DefWindowProcW(Wnd, Msg, wParam, lParam);
            } else {
                return DefWindowProcA(Wnd, Msg, wParam, lParam);
            }
        }
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn ScrollBarWndProcW(
    Wnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    // DefWindowProcW is extern "system"; the WNDPROC param is bindgen extern "stdcall"
    // (identical convention per arch) — reinterpret the fn pointer into WNDPROC.
    ScrollBarWndProc_common(
        core::mem::transmute::<usize, WNDPROC>(DefWindowProcW as usize),
        Wnd, Msg, wParam, lParam, 1,
    )
}

#[no_mangle]
pub unsafe extern "system" fn ScrollBarWndProcA(
    Wnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    // DefWindowProcA is extern "system"; the WNDPROC param is bindgen extern "stdcall"
    // (identical convention per arch) — reinterpret the fn pointer into WNDPROC.
    ScrollBarWndProc_common(
        core::mem::transmute::<usize, WNDPROC>(DefWindowProcA as usize),
        Wnd, Msg, wParam, lParam, 0,
    )
}

#[no_mangle]
pub unsafe extern "system" fn EnableScrollBar(hwnd: HWND, nBar: UINT, flags: UINT) -> BOOL {
    let mut Ret: BOOL = 0;

    LoadUserApiHook();

    let Hook = BeginIfHookedUserApiHook();

    if Hook == 0 {
        Ret = NtUserEnableScrollBar(hwnd, nBar, flags);
        if Ret == 0 {
            return Ret;
        }
        SCROLL_RefreshScrollBar(hwnd, nBar as INT, 1, 1);
        return Ret;
    }

    Ret = guah.EnableScrollBar.unwrap()(hwnd, nBar, flags);

    EndUserApiHook();

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn RealGetScrollInfo(Wnd: HWND, SBType: INT, Info: LPSCROLLINFO) -> BOOL {
    let pWnd: PWND;
    let pSBData: PSBDATA;

    if SB_CTL as INT == SBType {
        return SendMessageW(Wnd, SBM_GETSCROLLINFO, 0, Info as LPARAM) as BOOL;
    }

    pWnd = ValidateHwnd(Wnd);
    if pWnd.is_null() {
        return 0;
    }

    if SBType < SB_HORZ as INT || SBType > SB_VERT as INT {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    if (*pWnd).pSBInfo.is_null() {
        SetLastError(ERROR_NO_SCROLLBARS);
        return 0;
    }
    pSBData = IntGetSBData(pWnd, SBType);
    NtUserSBGetParms(Wnd, SBType, pSBData, Info)
}

#[no_mangle]
pub unsafe extern "system" fn GetScrollBarInfo(hwnd: HWND, idObject: LONG, info: LPSCROLLBARINFO) -> BOOL {
    let Ret: BOOL;
    let pWnd: PWND = ValidateHwnd(hwnd);
    TRACE!(b"hwnd=%p idObject=%d info=%p\n\0".as_ptr() as *const i8, hwnd, idObject, info);
    if pWnd.is_null() {
        return 0;
    }
    Ret = NtUserGetScrollBarInfo(hwnd, idObject, info);
    OffsetRect(
        core::ptr::addr_of_mut!((*info).rcScrollBar) as *mut RECT,
        (*pWnd).rcWindow.left,
        (*pWnd).rcWindow.top,
    );
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn GetScrollInfo(Wnd: HWND, SBType: INT, Info: LPSCROLLINFO) -> BOOL {
    let Hook: BOOL;
    let mut Ret: BOOL = 0;

    LoadUserApiHook();

    Hook = BeginIfHookedUserApiHook();

    /* Bypass SEH and go direct. */
    if Hook == 0 {
        return RealGetScrollInfo(Wnd, SBType, Info);
    }

    Ret = guah.GetScrollInfo.unwrap()(Wnd, SBType, Info);

    EndUserApiHook();

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn GetScrollPos(Wnd: HWND, Bar: INT) -> INT {
    let pwnd: PWND;
    let pSBData: PSBDATA;

    TRACE!(b"Wnd=%p Bar=%d\n\0".as_ptr() as *const i8, Wnd, Bar);

    if SB_CTL as INT == Bar {
        return SendMessageW(Wnd, SBM_GETPOS, 0 as WPARAM, 0 as LPARAM) as INT;
    } else if Bar == SB_HORZ as INT || Bar == SB_VERT as INT {
        pwnd = ValidateHwnd(Wnd);
        if pwnd.is_null() {
            return 0;
        }

        if !(*pwnd).pSBInfo.is_null() {
            pSBData = IntGetSBData(pwnd, Bar);
            return (*pSBData).pos;
        }

        SetLastError(ERROR_NO_SCROLLBARS);
        TRACE!(b"GetScrollPos No Scroll Info\n\0".as_ptr() as *const i8);
        return 0;
    }
    SetLastError(ERROR_INVALID_PARAMETER);
    0
}

#[no_mangle]
pub unsafe extern "system" fn GetScrollRange(
    Wnd: HWND,
    Bar: INT,
    MinPos: LPINT,
    MaxPos: LPINT,
) -> BOOL {
    let pwnd: PWND;
    let pSBData: PSBDATA;

    TRACE!(
        b"Wnd=%x Bar=%d Min=%p Max=%p\n\0".as_ptr() as *const i8,
        Wnd,
        Bar,
        MinPos,
        MaxPos,
    );

    if SB_CTL as INT == Bar {
        return SendMessageW(
            Wnd,
            SBM_GETRANGE,
            MinPos as WPARAM,
            MaxPos as LPARAM,
        ) as BOOL;
    } else if Bar == SB_HORZ as INT || Bar == SB_VERT as INT {
        pwnd = ValidateHwnd(Wnd);
        if pwnd.is_null() {
            return 0;
        }

        if !(*pwnd).pSBInfo.is_null() {
            pSBData = IntGetSBData(pwnd, Bar);
            *MinPos = (*pSBData).posMin;
            *MaxPos = (*pSBData).posMax;
        } else {
            SetLastError(ERROR_NO_SCROLLBARS);
            *MinPos = 0;
            *MaxPos = 0;
        }
        return 1;
    }
    SetLastError(ERROR_INVALID_PARAMETER);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RealSetScrollInfo(
    Wnd: HWND,
    SBType: core::ffi::c_int,
    Info: LPCSCROLLINFO,
    bRedraw: i32,
) -> core::ffi::c_int {

    if SB_CTL as core::ffi::c_int == SBType {
        SendMessageW(
            Wnd,
            SBM_SETSCROLLINFO,
            bRedraw as WPARAM,
            Info as LPARAM,
        ) as core::ffi::c_int
    } else {
        NtUserSetScrollInfo(Wnd, SBType, Info as LPSCROLLINFO, bRedraw) as core::ffi::c_int
    }
}

#[no_mangle]
pub unsafe extern "system" fn SetScrollInfo(
    Wnd: HWND,
    SBType: core::ffi::c_int,
    Info: LPCSCROLLINFO,
    bRedraw: i32,
) -> core::ffi::c_int {
    let Hook: i32;
    let mut Ret: core::ffi::c_int = 0;

    LoadUserApiHook();

    Hook = BeginIfHookedUserApiHook();

    if Hook == 0 {
        return RealSetScrollInfo(Wnd, SBType, Info, bRedraw);
    }

    Ret = guah.SetScrollInfo.unwrap()(Wnd, SBType, Info, bRedraw);

    EndUserApiHook();

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn SetScrollPos(
    hWnd: HWND,
    nBar: core::ffi::c_int,
    nPos: core::ffi::c_int,
    bRedraw: i32,
) -> core::ffi::c_int {
    let mut ScrollInfo: SCROLLINFO = core::mem::zeroed();

    ScrollInfo.cbSize = core::mem::size_of::<SCROLLINFO>() as u32;
    ScrollInfo.fMask = (SIF_POS | SIF_PREVIOUSPOS) as u32;
    ScrollInfo.nPos = nPos;

    SetScrollInfo(hWnd, nBar, &ScrollInfo, bRedraw)
}

#[no_mangle]
pub unsafe extern "system" fn SetScrollRange(
    hWnd: HWND,
    nBar: core::ffi::c_int,
    nMinPos: core::ffi::c_int,
    nMaxPos: core::ffi::c_int,
    bRedraw: i32,
) -> i32 {
    let pWnd: PWND;
    let mut ScrollInfo: SCROLLINFO = core::mem::zeroed();

    pWnd = ValidateHwnd(hWnd);
    if pWnd.is_null() {
        return 0;
    }

    if (nMaxPos as i64 - nMinPos as i64) > MAXLONG as i64 {
        SetLastError(ERROR_INVALID_SCROLLBAR_RANGE);
        return 0;
    }

    ScrollInfo.cbSize = core::mem::size_of::<SCROLLINFO>() as u32;
    ScrollInfo.fMask = SIF_RANGE as u32;
    ScrollInfo.nMin = nMinPos;
    ScrollInfo.nMax = nMaxPos;
    SetScrollInfo(hWnd, nBar, &ScrollInfo, bRedraw);
    1
}
