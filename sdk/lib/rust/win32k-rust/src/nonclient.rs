#![allow(non_snake_case, non_upper_case_globals)]
use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::PsGetCurrentThreadWin32Thread;
use core::ffi::c_void;

// --- cdecl (`_Name`) win32k/user exports referenced by nonclient. ---
extern "C" {
    fn MENU_DrawMenuBar(hDC: HDC, Rect: *mut RECT, pWnd: PWND, suppress_draw: BOOL) -> UINT;
}

// --- STDCALL (APIENTRY: `_Name@N` on i386, cdecl elsewhere). ---
extern "system" {
    fn DrawEdge(hdc: HDC, qrc: *mut RECT, edge: UINT, grfFlags: UINT) -> BOOL;
    fn DrawFrameControl(hdc: HDC, rc: *mut RECT, type_: UINT, state: UINT) -> BOOL;
    fn FillRect(hdc: HDC, rc: *const RECT, hbr: HBRUSH) -> INT;
    fn co_IntGetPeekMessage(msg: *mut MSG, hwnd: HWND, min: UINT, max: UINT, flag: UINT, flag2: BOOL) -> BOOL;
}

// --- FASTCALL (`@Name@N`) on i386, cdecl elsewhere. ---
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn RECTL_vInflateRect(rect: *mut RECTL, dx: core::ffi::c_int, dy: core::ffi::c_int);
    fn co_IntSendMessage(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn co_UserSetCapture(hwnd: isize) -> isize;
    fn co_IntSetForegroundWindowMouse(pWnd: PWND) -> BOOL;
    fn co_WinPosSetWindowPos( Wnd: PWND, WndInsertAfter: HWND, x: ::core::ffi::c_int, y: ::core::ffi::c_int, cx: ::core::ffi::c_int, cy: ::core::ffi::c_int, flags: UINT, ) -> BOOL;
    fn co_IntGetScrollBarInfo(pWnd: PWND, hBar: INT, sbi: *mut SCROLLBARINFO) -> BOOL;
    fn MsqSetStateWindow(pti: PTHREADINFO, ty: ULONG, hWnd: HWND) -> HWND;
    fn MsqPostQuitMessage(pti: PTHREADINFO, ExitCode: ULONG);
    fn UpdateThreadWindows(pWnd: PWND, pti: PTHREADINFO, hRgn: HRGN);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn RECTL_vInflateRect(rect: *mut RECTL, dx: core::ffi::c_int, dy: core::ffi::c_int);
    fn co_IntSendMessage(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn co_UserSetCapture(hwnd: isize) -> isize;
    fn co_IntSetForegroundWindowMouse(pWnd: PWND) -> BOOL;
    fn co_WinPosSetWindowPos( Wnd: PWND, WndInsertAfter: HWND, x: ::core::ffi::c_int, y: ::core::ffi::c_int, cx: ::core::ffi::c_int, cy: ::core::ffi::c_int, flags: UINT, ) -> BOOL;
    fn co_IntGetScrollBarInfo(pWnd: PWND, hBar: INT, sbi: *mut SCROLLBARINFO) -> BOOL;
    fn MsqSetStateWindow(pti: PTHREADINFO, ty: ULONG, hWnd: HWND) -> HWND;
    fn MsqPostQuitMessage(pti: PTHREADINFO, ExitCode: ULONG);
    fn UpdateThreadWindows(pWnd: PWND, pti: PTHREADINFO, hRgn: HRGN);
}

// ---- FORCEINLINE / macro helpers (no exported kernel symbol) ----

// UserIsDesktopWindow(pWnd) (desktop.h macro): pWnd && pWnd->fnid == FNID_DESKTOP.
#[inline(always)]
unsafe fn UserIsDesktopWindow(pWnd: PWND) -> BOOL {
    (!pWnd.is_null() && (*pWnd).fnid == FNID_DESKTOP) as BOOL
}

// IntEqualRect(lprc1, lprc2) (rect FORCEINLINE).
#[inline(always)]
unsafe fn IntEqualRect(lprc1: *const RECTL, lprc2: *const RECTL) -> BOOL {
    if lprc1.is_null() || lprc2.is_null() {
        return 0;
    }
    (((*lprc1).left == (*lprc2).left)
        && ((*lprc1).top == (*lprc2).top)
        && ((*lprc1).right == (*lprc2).right)
        && ((*lprc1).bottom == (*lprc2).bottom)) as BOOL
}

// IntIsWindowSnapped(Wnd) (FORCEINLINE BOOLEAN).
#[inline(always)]
unsafe fn IntIsWindowSnapped(Wnd: PWND) -> BOOLEAN {
    (((*Wnd).ExStyle2
        & (WS_EX2_VERTICALLYMAXIMIZEDLEFT | WS_EX2_VERTICALLYMAXIMIZEDRIGHT) as ULONG)
        != 0) as BOOLEAN
}

// IntIsSnapAllowedForWindow(Wnd) (FORCEINLINE BOOLEAN).
#[inline(always)]
unsafe fn IntIsSnapAllowedForWindow(Wnd: PWND) -> BOOLEAN {
    let style: UINT = (*Wnd).style;
    let tbws: UINT = WS_POPUP | WS_VISIBLE | WS_CLIPSIBLINGS | WS_CLIPCHILDREN;
    let tbes: UINT = WS_EX_TOOLWINDOW;
    let istb: bool =
        (style & tbws) == tbws && ((*Wnd).ExStyle & (tbes | WS_EX_APPWINDOW)) == tbes;
    let thickframe: bool = (style & WS_THICKFRAME) != 0
        && (style & (WS_DLGFRAME | WS_BORDER)) != WS_DLGFRAME;
    (thickframe && (style & WS_CHILD) == 0 && !istb) as BOOLEAN
}

// UserGetMenuObject(hMenu) (FORCEINLINE PMENU).
#[inline(always)]
unsafe fn UserGetMenuObject(hMenu: HMENU) -> PMENU {
    let pMenu = UserGetObject(gHandleTable, hMenu as _, TYPE_MENU) as PMENU;
    if pMenu.is_null() {
        EngSetLastError(ERROR_INVALID_MENU_HANDLE);
    }
    pMenu
}

// MAKELONG / LOWORD / HIWORD (minwindef.h macros). Generic over the integer width
// of the args (callers pass WORD/u16 or DWORD/u32).
#[inline(always)]
fn MAKELONG<A: Into<u32>, B: Into<u32>>(low: A, high: B) -> LONG {
    ((low.into() & 0xffff) | ((high.into() & 0xffff) << 16)) as LONG
}
#[inline(always)]
fn LOWORD(l: DWORD) -> DWORD { l & 0xffff }
#[inline(always)]
fn HIWORD(l: DWORD) -> DWORD { (l >> 16) & 0xffff }

// min/max (windef.h macros) and min_l/max_l (LONG variants used in nonclient.c).
#[inline(always)]
fn min<T: PartialOrd>(a: T, b: T) -> T { if a < b { a } else { b } }
#[inline(always)]
fn max<T: PartialOrd>(a: T, b: T) -> T { if a > b { a } else { b } }
#[inline(always)]
fn min_l(a: LONG, b: LONG) -> LONG { if a < b { a } else { b } }
#[inline(always)]
fn max_l(a: LONG, b: LONG) -> LONG { if a > b { a } else { b } }

// HTxxx hit-test constants used as LONG comparisons. (The translator emitted
// _HTxxx() function calls; the C source uses the plain HTxxx constants.)
#[inline(always)] fn _HTTOPLEFT() -> LONG { HTTOPLEFT as LONG }
#[inline(always)] fn _HTTOPRIGHT() -> LONG { HTTOPRIGHT as LONG }
#[inline(always)] fn _HTBOTTOMLEFT() -> LONG { HTBOTTOMLEFT as LONG }
#[inline(always)] fn _HTRIGHT() -> LONG { HTRIGHT as LONG }
#[inline(always)] fn _HTBOTTOM() -> LONG { HTBOTTOM as LONG }

// HWND_TOP = (HWND)0 (winuser.h macro).
const HWND_TOP: HWND = 0usize as HWND;

// UlongToHandle(ul) = (HANDLE)(ULONG_PTR)(ULONG)ul (basetsd.h macro).
#[inline(always)]
fn UlongToHandle(ul: u32) -> HANDLE {
    ul as ULONG_PTR as HANDLE
}

// UserHMGetHandle(obj) = obj->head.h (HANDLE at offset 0 of the object HEAD).
#[inline(always)]
unsafe fn UserHMGetHandle<T>(pobj: *mut T) -> HWND {
    *(pobj as *const HWND)
}

// _THRDESKHEAD/_THROBJHEAD.pti: bindgen flattens HEAD{h, cLockObj} + pti into
// __bindgen_padding_0. pti follows HEAD{HANDLE, DWORD}, ptr-aligned, so it sits at
// 2*usize (8 i386 / 16 amd64/arm64). (Matches focus.rs/simplecall.rs head_pti.)
#[inline(always)]
unsafe fn head_pti(head: *const _THRDESKHEAD) -> PTHREADINFO {
    let off = 2 * core::mem::size_of::<usize>();
    *((head as *const u8).add(off) as *const PTHREADINFO)
}

// gpsi tail (anonymous PERUSERSERVERINFO member): tagSERVERINFO ends with
// __bindgen_padding_0 representing it; cast that tail to access its fields
// (hbrGray/hIconWindows/hIconSmWindows/aiSysMet/ptCursor...). Matches focus.rs.
#[inline(always)]
unsafe fn gpsi_per_user() -> *mut _PERUSERSERVERINFO {
    (gpsi as *mut u8)
        .add(core::mem::offset_of!(tagSERVERINFO, __bindgen_padding_0)) as *mut _PERUSERSERVERINFO
}

// IsSnapEnabled() = g_bWindowSnapEnabled (winpos.h static inline).
#[inline(always)]
unsafe fn IsSnapEnabled() -> BOOL { g_bWindowSnapEnabled }

// GetSnapSetting(member) = IsSnapEnabled() ? gspv.member : 0 (winpos.h macro).
macro_rules! GetSnapSetting {
    ($member:ident) => {{
        if IsSnapEnabled() != 0 { gspv.$member } else { 0 }
    }};
}

// Frame-style predicate macros (nonclient.c / window.h). Translate the C `&&`/`||`
// short-circuit into Rust bool, returning BOOL (i32).
#[inline(always)]
fn UserHasDlgFrameStyle(Style: u32, ExStyle: u32) -> BOOL {
    (((ExStyle & WS_EX_DLGMODALFRAME) != 0)
        || (((Style & WS_DLGFRAME) != 0) && ((Style & WS_THICKFRAME) == 0))) as BOOL
}
#[inline(always)]
fn UserHasThickFrameStyle(Style: u32, ExStyle: u32) -> BOOL {
    let _ = ExStyle;
    (((Style & WS_THICKFRAME) != 0)
        && (!((Style & (WS_DLGFRAME | WS_BORDER)) == WS_DLGFRAME))) as BOOL
}
#[inline(always)]
fn UserHasThinFrameStyle(Style: u32, ExStyle: u32) -> BOOL {
    let _ = ExStyle;
    (((Style & WS_BORDER) != 0) || ((Style & (WS_CHILD | WS_POPUP)) == 0)) as BOOL
}
#[inline(always)]
fn HAS_DLGFRAME(Style: u32, ExStyle: u32) -> BOOL { UserHasDlgFrameStyle(Style, ExStyle) }
#[inline(always)]
fn HAS_THICKFRAME(Style: u32, ExStyle: u32) -> BOOL { UserHasThickFrameStyle(Style, ExStyle) }
#[inline(always)]
fn HAS_THINFRAME(Style: u32, ExStyle: u32) -> BOOL { UserHasThinFrameStyle(Style, ExStyle) }
#[inline(always)]
fn HAS_CLIENTFRAME(Style: u32, ExStyle: u32) -> BOOL {
    (((ExStyle & WS_EX_CLIENTEDGE) != 0) || ((Style & (WS_CHILD | WS_POPUP)) == 0)) as BOOL
}

// RECT/RECTL are layout-identical ({left,top,right,bottom: LONG}) but are distinct
// nominal types in bindgen (tagRECT vs _RECTL). The C source mixes them freely in
// the rect.h FORCEINLINE helpers; model that with a private RectLike trait so the
// inline helpers accept a pointer to either without call-site casts.
trait RectLike {
    unsafe fn rl(p: *const Self) -> LONG;
    unsafe fn rt(p: *const Self) -> LONG;
    unsafe fn rr(p: *const Self) -> LONG;
    unsafe fn rb(p: *const Self) -> LONG;
    unsafe fn set(p: *mut Self, l: LONG, t: LONG, r: LONG, b: LONG);
}
macro_rules! impl_rectlike {
    ($t:ty) => {
        impl RectLike for $t {
            #[inline(always)] unsafe fn rl(p: *const Self) -> LONG { (*p).left }
            #[inline(always)] unsafe fn rt(p: *const Self) -> LONG { (*p).top }
            #[inline(always)] unsafe fn rr(p: *const Self) -> LONG { (*p).right }
            #[inline(always)] unsafe fn rb(p: *const Self) -> LONG { (*p).bottom }
            #[inline(always)] unsafe fn set(p: *mut Self, l: LONG, t: LONG, r: LONG, b: LONG) {
                (*p).left = l; (*p).top = t; (*p).right = r; (*p).bottom = b;
            }
        }
    };
}
impl_rectlike!(RECTL);
impl_rectlike!(RECT);

// RECTL_vSetRect / RECTL_vOffsetRect / RECTL_bIsEmptyRect / RECTL_bPointInRect (rect.h FORCEINLINE).
#[inline(always)]
unsafe fn RECTL_vSetRect<R: RectLike>(prcl: *mut R, left: LONG, top: LONG, right: LONG, bottom: LONG) {
    R::set(prcl, left, top, right, bottom);
}
#[inline(always)]
unsafe fn RECTL_vOffsetRect<R: RectLike>(prcl: *mut R, cx: INT, cy: INT) {
    R::set(prcl, R::rl(prcl) + cx, R::rt(prcl) + cy, R::rr(prcl) + cx, R::rb(prcl) + cy);
}
#[inline(always)]
unsafe fn RECTL_bIsEmptyRect<R: RectLike>(prcl: *const R) -> BOOL {
    (R::rl(prcl) >= R::rr(prcl) || R::rt(prcl) >= R::rb(prcl)) as BOOL
}
#[inline(always)]
unsafe fn RECTL_bPointInRect<R: RectLike>(prcl: *const R, x: INT, y: INT) -> BOOL {
    (x >= R::rl(prcl) && x < R::rr(prcl) && y >= R::rt(prcl) && y < R::rb(prcl)) as BOOL
}
// RECTL_vInflateRect is FASTCALL (real symbol) over RECTL; the C callers pass RECT
// too. Wrap the extern in an inline that casts the pointer (layout-identical).
#[inline(always)]
unsafe fn RECTL_vInflateRect_g<R>(rect: *mut R, dx: core::ffi::c_int, dy: core::ffi::c_int) {
    RECTL_vInflateRect(rect as *mut RECTL, dx, dy);
}

// GreCreateRectRgnIndirect(prc) (region.h macro) ->
// NtGdiCreateRectRgn(prc->left, prc->top, prc->right, prc->bottom).
#[inline(always)]
unsafe fn GreCreateRectRgnIndirect<R: RectLike>(prc: *const R) -> HRGN {
    NtGdiCreateRectRgn(R::rl(prc), R::rt(prc), R::rr(prc), R::rb(prc))
}

// HAS_MENU(pWnd,style) (window.h macro).
#[inline(always)]
unsafe fn HAS_MENU(pWnd: PWND, style: u32) -> BOOL {
    (((style & (WS_CHILD | WS_POPUP)) != WS_CHILD)
        && (*pWnd).IDMenu != 0
        && IntIsMenu(UlongToHandle((*pWnd).IDMenu as u32) as _) != 0) as BOOL
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn UserDrawWindowFrame(
    hdc: HDC,
    rect: *mut RECTL,
    width: ULONG,
    height: ULONG,
) {
    UserDrawWindowFrameImpl(hdc, rect, width, height)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn UserDrawWindowFrame(
    hdc: HDC,
    rect: *mut RECTL,
    width: ULONG,
    height: ULONG,
) {
    UserDrawWindowFrameImpl(hdc, rect, width, height)
}

unsafe fn UserDrawWindowFrameImpl(
    hdc: HDC,
    rect: *mut RECTL,
    width: ULONG,
    height: ULONG,
) {
    let hbrush: HBRUSH = NtGdiSelectBrush(hdc, (*gpsi_per_user()).hbrGray);
    NtGdiPatBlt(
        hdc,
        (*rect).left,
        (*rect).top,
        (*rect).right - (*rect).left - width as LONG,
        height as LONG,
        PATINVERT,
    );
    NtGdiPatBlt(
        hdc,
        (*rect).left,
        (*rect).top + height as LONG,
        width as LONG,
        (*rect).bottom - (*rect).top - height as LONG,
        PATINVERT,
    );
    NtGdiPatBlt(
        hdc,
        (*rect).left + width as LONG,
        (*rect).bottom,
        (*rect).right - (*rect).left - width as LONG,
        -(height as LONG),
        PATINVERT,
    );
    NtGdiPatBlt(
        hdc,
        (*rect).right,
        (*rect).top,
        -(width as LONG),
        (*rect).bottom - (*rect).top - height as LONG,
        PATINVERT,
    );
    NtGdiSelectBrush(hdc, hbrush);
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn UserDrawMovingFrame(hdc: HDC, rect: *mut RECTL, thickframe: BOOL) {
    UserDrawMovingFrameImpl(hdc, rect, thickframe)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn UserDrawMovingFrame(hdc: HDC, rect: *mut RECTL, thickframe: BOOL) {
    UserDrawMovingFrameImpl(hdc, rect, thickframe)
}

unsafe fn UserDrawMovingFrameImpl(hdc: HDC, rect: *mut RECTL, thickframe: BOOL) {
    if thickframe != 0 {
        UserDrawWindowFrame(
            hdc,
            rect,
            UserGetSystemMetrics(SM_CXFRAME as ULONG) as ULONG,
            UserGetSystemMetrics(SM_CYFRAME as ULONG) as ULONG,
        );
    } else {
        UserDrawWindowFrame(hdc, rect, 1, 1);
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn NC_GetInsideRect(Wnd: PWND, rect: *mut RECT) {
    NC_GetInsideRect_impl(Wnd, rect)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn NC_GetInsideRect(Wnd: PWND, rect: *mut RECT) {
    NC_GetInsideRect_impl(Wnd, rect)
}

unsafe fn NC_GetInsideRect_impl(Wnd: PWND, rect: *mut RECT) {

    let Style: ULONG = (*Wnd).style;
    let ExStyle: ULONG = (*Wnd).ExStyle;

    (*rect).top = 0;
    (*rect).left = 0;
    (*rect).right = (*Wnd).rcWindow.right - (*Wnd).rcWindow.left;
    (*rect).bottom = (*Wnd).rcWindow.bottom - (*Wnd).rcWindow.top;

    if (Style & WS_ICONIC) != 0 {
        return;
    }

    let has_thick = (Style & WS_THICKFRAME) != 0
        && !((Style & (WS_DLGFRAME | WS_BORDER)) == WS_DLGFRAME);
    let has_dlg = (ExStyle & WS_EX_DLGMODALFRAME) != 0
        || ((Style & WS_DLGFRAME) != 0 && (Style & WS_THICKFRAME) == 0);
    let has_thin = (Style & WS_BORDER) != 0 || ((Style & (WS_CHILD | WS_POPUP)) == 0);

    if has_thick {
        RECTL_vInflateRect_g(
            rect,
            -UserGetSystemMetrics(SM_CXFRAME as _),
            -UserGetSystemMetrics(SM_CYFRAME as _),
        );
    } else if has_dlg {
        RECTL_vInflateRect_g(
            rect,
            -UserGetSystemMetrics(SM_CXDLGFRAME as _),
            -UserGetSystemMetrics(SM_CYDLGFRAME as _),
        );
        if (ExStyle & WS_EX_DLGMODALFRAME) != 0 {
            RECTL_vInflateRect_g(rect, -1, 0);
        }
    } else if has_thin {
        RECTL_vInflateRect_g(
            rect,
            -UserGetSystemMetrics(SM_CXBORDER as _),
            -UserGetSystemMetrics(SM_CYBORDER as _),
        );
    }

    if (Style & WS_CHILD) != 0 && (ExStyle & WS_EX_MDICHILD) == 0 {
        if (ExStyle & WS_EX_CLIENTEDGE) != 0 {
            RECTL_vInflateRect_g(
                rect,
                -UserGetSystemMetrics(SM_CXEDGE as _),
                -UserGetSystemMetrics(SM_CYEDGE as _),
            );
        }
        if (ExStyle & WS_EX_STATICEDGE) != 0 {
            RECTL_vInflateRect_g(
                rect,
                -UserGetSystemMetrics(SM_CXBORDER as _),
                -UserGetSystemMetrics(SM_CYBORDER as _),
            );
        }
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn NC_GetSysPopupPos(Wnd: PWND, Rect: *mut RECT) {
    NC_GetSysPopupPos_impl(Wnd, Rect)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn NC_GetSysPopupPos(Wnd: PWND, Rect: *mut RECT) {
    NC_GetSysPopupPos_impl(Wnd, Rect)
}

unsafe fn NC_GetSysPopupPos_impl(Wnd: PWND, Rect: *mut RECT) {

    let mut WindowRect: RECT = core::mem::zeroed();

    if ((*Wnd).style & WS_MINIMIZE) != 0 {
        IntGetWindowRect(Wnd, Rect as _);
    } else {
        NC_GetInsideRect(Wnd, Rect);
        IntGetWindowRect(Wnd, &mut WindowRect as *mut _ as _);
        RECTL_vOffsetRect(Rect, WindowRect.left, WindowRect.top);
        if ((*Wnd).style & WS_CHILD) != 0 {
            IntClientToScreen(IntGetParent(Wnd), Rect as *mut POINTL);
        }
        (*Rect).right = (*Rect).left + UserGetSystemMetrics(SM_CYCAPTION as _) - 1;
        (*Rect).bottom = (*Rect).top + UserGetSystemMetrics(SM_CYCAPTION as _) - 1;
    }
}

unsafe fn GetSnapActivationPoint(Wnd: PWND, pt: POINT) -> UINT {
    let mut wa: RECT = core::mem::zeroed();
    if GetSnapSetting!(bDockMoving) == 0 {
        return HTNOWHERE as UINT;
    }
    UserSystemParametersInfo(SPI_GETWORKAREA, 0, core::ptr::addr_of_mut!(wa) as *mut _, 0);

    if pt.x <= wa.left {
        return HTLEFT as UINT;
    }
    if pt.x >= wa.right - 1 {
        return HTRIGHT as UINT;
    }
    if pt.y <= wa.top {
        return HTTOP as UINT;
    }
    HTNOWHERE as UINT
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn DefWndStartSizeMove(Wnd: PWND, wParam: WPARAM, capturePoint: *mut POINT) -> LONG {
    DefWndStartSizeMove_impl(Wnd, wParam, capturePoint)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn DefWndStartSizeMove(Wnd: PWND, wParam: WPARAM, capturePoint: *mut POINT) -> LONG {
    DefWndStartSizeMove_impl(Wnd, wParam, capturePoint)
}

unsafe fn DefWndStartSizeMove_impl(Wnd: PWND, wParam: WPARAM, capturePoint: *mut POINT) -> LONG {
    let mut hittest: LONG = 0;
    let mut pt: POINT = core::mem::zeroed();
    let mut msg: MSG = core::mem::zeroed();
    let rectWindow: RECT;
    let Style: ULONG = (*Wnd).style;
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as _;

    rectWindow = core::mem::transmute((*Wnd).rcWindow);

    if (wParam & 0xfff0) == SC_MOVE as WPARAM {
        let mut rect: RECT = rectWindow;
        if (Style & WS_SYSMENU) != 0 {
            rect.left += UserGetSystemMetrics(SM_CXSIZE) + 1;
        }
        if (Style & WS_MINIMIZEBOX) != 0 {
            rect.right -= UserGetSystemMetrics(SM_CXSIZE) + 1;
        }
        if (Style & WS_MAXIMIZEBOX) != 0 {
            rect.right -= UserGetSystemMetrics(SM_CXSIZE) + 1;
        }
        pt.x = (rect.right + rect.left) / 2;
        pt.y = rect.top + UserGetSystemMetrics(SM_CYSIZE) / 2;
        hittest = HTCAPTION as LONG;
        *capturePoint = pt;
    } else {
        pt.x = 0;
        pt.y = 0;
        while hittest == 0 {
            if co_IntGetPeekMessage(&mut msg, core::ptr::null_mut(), 0, 0, PM_REMOVE, 1) == 0 {
                return 0;
            }
            if IntCallMsgFilter(&mut msg, MSGF_SIZE as INT) != 0 {
                continue;
            }

            // A WM_QUIT removed here never reaches the thread's outer GetMessage()
            // loop, so a thread that posted its own quit mid-drag waits forever.
            // Upstream's repro: drag the first-boot setup window while it finishes,
            // setup stalls and the shell never starts. Re-post and leave the loop.
            // Upstream-Commit: cf470941031 (#9087)
            if msg.message == WM_QUIT {
                MsqPostQuitMessage(pti, msg.wParam as ULONG);
                return 0;
            }

            match msg.message {
                x if x == WM_MOUSEMOVE => {
                    pt.x = min(max(msg.pt.x, rectWindow.left), rectWindow.right - 1);
                    pt.y = min(max(msg.pt.y, rectWindow.top), rectWindow.bottom - 1);
                    hittest = GetNCHitEx(Wnd, pt) as LONG;
                    if (hittest < HTLEFT as LONG) || (hittest > HTBOTTOMRIGHT as LONG) {
                        hittest = 0;
                    }
                }
                x if x == WM_LBUTTONUP => {
                    return 0;
                }
                x if x == WM_KEYDOWN => {
                    match msg.wParam as u32 {
                        VK_UP => {
                            hittest = HTTOP as LONG;
                            pt.x = (rectWindow.left + rectWindow.right) / 2;
                            pt.y = rectWindow.top + UserGetSystemMetrics(SM_CYFRAME) / 2;
                        }
                        VK_DOWN => {
                            hittest = HTBOTTOM as LONG;
                            pt.x = (rectWindow.left + rectWindow.right) / 2;
                            pt.y = rectWindow.bottom - UserGetSystemMetrics(SM_CYFRAME) / 2;
                        }
                        VK_LEFT => {
                            hittest = HTLEFT as LONG;
                            pt.x = rectWindow.left + UserGetSystemMetrics(SM_CXFRAME) / 2;
                            pt.y = (rectWindow.top + rectWindow.bottom) / 2;
                        }
                        VK_RIGHT => {
                            hittest = HTRIGHT as LONG;
                            pt.x = rectWindow.right - UserGetSystemMetrics(SM_CXFRAME) / 2;
                            pt.y = (rectWindow.top + rectWindow.bottom) / 2;
                        }
                        VK_RETURN | VK_ESCAPE => {
                            return 0;
                        }
                        _ => {}
                    }
                }
                _ => {
                    IntTranslateKbdMessage(&mut msg, 0);
                    (*pti).TIF_flags |= TIF_MOVESIZETRACKING;
                    IntDispatchMessage(&mut msg);
                    (*pti).TIF_flags |= TIF_MOVESIZETRACKING;
                }
            }
        }
        *capturePoint = pt;
    }

    UserSetCursorPos(pt.x, pt.y, 0, 0, 0);
    co_IntSendMessage(
        UserHMGetHandle(Wnd),
        WM_SETCURSOR,
        UserHMGetHandle(Wnd) as WPARAM,
        MAKELONG(hittest as WORD, WM_MOUSEMOVE as WORD) as LPARAM,
    );
    hittest
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn DefWndDoSizeMove(pwnd: PWND, wParam: WORD) {
    DefWndDoSizeMove_impl(pwnd, wParam)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn DefWndDoSizeMove(pwnd: PWND, wParam: WORD) {
    DefWndDoSizeMove_impl(pwnd, wParam)
}

unsafe fn DefWndDoSizeMove_impl(pwnd: PWND, wParam: WORD) {
    let mut msg: MSG = core::mem::zeroed();
    let mut sizingRect: RECT = core::mem::zeroed();
    let mut mouseRect: RECT = core::mem::zeroed();
    let mut origRect: RECT = core::mem::zeroed();
    let mut unmodRect: RECT = core::mem::zeroed();
    let mut snapPreviewRect: RECT = core::mem::zeroed();
    let mut pFrameRect: PRECT = core::ptr::addr_of_mut!(sizingRect);
    let hdc: HDC;
    let mut hittest: LONG = (wParam & 0x0f) as LONG;
    let mut DragCursor: PCURICON_OBJECT = core::ptr::null_mut();
    let mut OldCursor: PCURICON_OBJECT = core::ptr::null_mut();
    let mut minTrack: POINT = core::mem::zeroed();
    let mut maxTrack: POINT = core::mem::zeroed();
    let mut capturePoint: POINT;
    let mut pt: POINT = core::mem::zeroed();
    let Style: ULONG;
    let ExStyle: ULONG;
    let mut orgSnap: UINT = IntGetWindowSnapEdge(pwnd) as UINT;
    let mut snap: UINT = orgSnap;
    let thickframe: BOOL;
    let mut iconic: BOOL;
    let mut moved: BOOL = 0;
    let mut DragFullWindows: BOOL = 0;
    let mut pWndParent: PWND = core::ptr::null_mut();
    let syscommand: WPARAM = (wParam & 0xfff0) as WPARAM;
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    Style = (*pwnd).style;
    ExStyle = (*pwnd).ExStyle;
    iconic = if (Style & WS_MINIMIZE as ULONG) != 0 { 1 } else { 0 };

    if ((Style & WS_MAXIMIZE as ULONG) != 0 && syscommand != SC_MOVE as WPARAM) || IntIsWindowVisible(pwnd) == 0 {
        return;
    }
    if (Style & (WS_MAXIMIZE as ULONG | WS_CHILD as ULONG)) == WS_MAXIMIZE as ULONG {
        orgSnap = HTTOP as UINT;
        snap = HTTOP as UINT;
    }

    thickframe = if UserHasThickFrameStyle(Style, ExStyle) != 0 && iconic == 0 { 1 } else { 0 };

    UserSystemParametersInfo(SPI_GETDRAGFULLWINDOWS, 0, core::ptr::addr_of_mut!(DragFullWindows) as *mut _, 0);

    pt.x = (*pti).ptLast.x;
    pt.y = (*pti).ptLast.y;
    capturePoint = pt;
    UserClipCursor(core::ptr::null_mut());

    if syscommand == SC_MOVE as WPARAM {
        if hittest == 0 {
            hittest = DefWndStartSizeMove(pwnd, wParam as WPARAM, core::ptr::addr_of_mut!(capturePoint));
        }
        if hittest == 0 {
            return;
        }
    } else {
        if thickframe == 0 {
            return;
        }
        if hittest != 0 && (syscommand != SC_MOUSEMENU as WPARAM) {
            hittest += HTLEFT as LONG - WMSZ_LEFT as LONG;
        } else {
            co_UserSetCapture(UserHMGetHandle(pwnd) as _);
            hittest = DefWndStartSizeMove(pwnd, wParam as WPARAM, core::ptr::addr_of_mut!(capturePoint));
            if hittest == 0 {
                IntReleaseCapture();
                return;
            }
        }
    }

    co_WinPosGetMinMaxInfo(pwnd, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(minTrack) as _, core::ptr::addr_of_mut!(maxTrack) as _);
    sizingRect = core::mem::transmute((*pwnd).rcWindow);
    origRect = sizingRect;
    if (Style & WS_CHILD as ULONG) != 0 {
        pWndParent = IntGetParent(pwnd);
        IntGetClientRect(pWndParent, core::ptr::addr_of_mut!(mouseRect) as _);
        IntMapWindowPoints(pWndParent, core::ptr::null_mut(), core::ptr::addr_of_mut!(mouseRect) as LPPOINT as _, 2);
        IntMapWindowPoints(core::ptr::null_mut(), pWndParent, core::ptr::addr_of_mut!(sizingRect) as LPPOINT as _, 2);
        unmodRect = sizingRect;
    } else {
        if (ExStyle & WS_EX_TOPMOST as ULONG) == 0 {
            UserSystemParametersInfo(SPI_GETWORKAREA, 0, core::ptr::addr_of_mut!(mouseRect) as *mut _, 0);
        } else {
            RECTL_vSetRect(core::ptr::addr_of_mut!(mouseRect) as *mut _, 0, 0, UserGetSystemMetrics(SM_CXSCREEN), UserGetSystemMetrics(SM_CYSCREEN));
        }
        unmodRect = sizingRect;
    }

    let on_left = |h: LONG| h == HTLEFT as LONG || h == _HTTOPLEFT() || h == _HTBOTTOMLEFT();
    let on_right = |h: LONG| h == _HTRIGHT() || h == _HTTOPRIGHT() || h == HTBOTTOMRIGHT as LONG;
    let on_top = |h: LONG| h == HTTOP as LONG || h == _HTTOPLEFT() || h == _HTTOPRIGHT();
    let on_bottom = |h: LONG| h == _HTBOTTOM() || h == _HTBOTTOMLEFT() || h == HTBOTTOMRIGHT as LONG;

    if on_left(hittest) {
        mouseRect.left = max_l(mouseRect.left, sizingRect.right - maxTrack.x + capturePoint.x - sizingRect.left);
        mouseRect.right = min_l(mouseRect.right, sizingRect.right - minTrack.x + capturePoint.x - sizingRect.left);
    } else if on_right(hittest) {
        mouseRect.left = max_l(mouseRect.left, sizingRect.left + minTrack.x + capturePoint.x - sizingRect.right);
        mouseRect.right = min_l(mouseRect.right, sizingRect.left + maxTrack.x + capturePoint.x - sizingRect.right);
    }
    if on_top(hittest) {
        mouseRect.top = max_l(mouseRect.top, sizingRect.bottom - maxTrack.y + capturePoint.y - sizingRect.top);
        mouseRect.bottom = min_l(mouseRect.bottom, sizingRect.bottom - minTrack.y + capturePoint.y - sizingRect.top);
    } else if on_bottom(hittest) {
        mouseRect.top = max_l(mouseRect.top, sizingRect.top + minTrack.y + capturePoint.y - sizingRect.bottom);
        mouseRect.bottom = min_l(mouseRect.bottom, sizingRect.top + maxTrack.y + capturePoint.y - sizingRect.bottom);
    }

    hdc = UserGetDCEx(pWndParent, core::ptr::null_mut(), DCX_CACHE);
    if iconic != 0 {
        DragCursor = (*(*pwnd).pcls).spicn;
        if !DragCursor.is_null() {
            UserReferenceObject(DragCursor as *mut _);
        } else {
            let CursorHandle = co_IntSendMessage(UserHMGetHandle(pwnd), WM_QUERYDRAGICON, 0, 0) as HCURSOR;
            if !CursorHandle.is_null() {
                DragCursor = UserGetCurIconObject(CursorHandle);
            } else {
                iconic = 0;
            }
        }
    }

    co_UserRedrawWindow(pwnd, core::ptr::null_mut(), core::ptr::null_mut(), (RDW_UPDATENOW | RDW_ALLCHILDREN) as ULONG);

    IntNotifyWinEvent(EVENT_SYSTEM_MOVESIZESTART, pwnd, OBJID_WINDOW as LONG, CHILDID_SELF as LONG, 0);

    co_IntSendMessage(UserHMGetHandle(pwnd), WM_ENTERSIZEMOVE, 0, 0);

    MsqSetStateWindow(pti, MSQ_STATE_MOVESIZE, UserHMGetHandle(pwnd));

    if IntGetCapture() != UserHMGetHandle(pwnd) {
        co_UserSetCapture(UserHMGetHandle(pwnd) as _);
    }

    (*head_pti(core::ptr::addr_of!((*pwnd).head))).TIF_flags |= TIF_MOVESIZETRACKING;

    loop {
        let mut dx: i32 = 0;
        let mut dy: i32 = 0;

        if co_IntGetPeekMessage(core::ptr::addr_of_mut!(msg), core::ptr::null_mut::<c_void>() as _, 0, 0, PM_REMOVE, 1) == 0 {
            break;
        }
        if IntCallMsgFilter(core::ptr::addr_of_mut!(msg), MSGF_SIZE as i32) != 0 {
            continue;
        }

        // A WM_QUIT removed here never reaches the thread's outer GetMessage()
        // loop, so a thread that posted its own quit mid-drag waits forever.
        // Upstream's repro: drag the first-boot setup window while it finishes,
        // setup stalls and the shell never starts. Re-post and leave the loop.
        // Upstream-Commit: cf470941031 (#9087)
        if msg.message == WM_QUIT {
            MsqPostQuitMessage(pti, msg.wParam as ULONG);
            break;
        }

        if msg.message == WM_KEYDOWN && (msg.wParam == VK_RETURN as WPARAM || msg.wParam == VK_ESCAPE as WPARAM) {
            break;
        }

        if g_bWindowSnapEnabled == 0 && (msg.message == WM_LBUTTONUP
            || (msg.message == WM_MOUSEMOVE && (msg.wParam & MK_LBUTTON as WPARAM) == 0)) {
            break;
        } else if g_bWindowSnapEnabled != 0 && (msg.message == WM_LBUTTONUP
            || (msg.message == WM_MOUSEMOVE && (msg.wParam & MK_LBUTTON as WPARAM) == 0)) {
            if hittest == HTCAPTION as LONG && thickframe != 0
                && IntIsSnapAllowedForWindow(pwnd) != 0 && (ExStyle & WS_EX_MDICHILD as ULONG) == 0 {
                let wasSnap: BOOLEAN = IntIsWindowSnapped(pwnd);
                let snapTo: UINT = if iconic != 0 { HTNOWHERE as UINT } else { GetSnapActivationPoint(pwnd, pt) };
                if snapTo != 0 {
                    if DragFullWindows != 0 {
                        co_IntSnapWindow(pwnd, snapTo);
                        if wasSnap == 0 {
                            (*pwnd).InternalPos.NormalRect = core::mem::transmute(origRect);
                        }
                    }
                    snap = snapTo;
                }
            }
            break;
        }

        if (msg.message != WM_KEYDOWN) && (msg.message != WM_MOUSEMOVE) {
            IntTranslateKbdMessage(core::ptr::addr_of_mut!(msg), 0);
            IntDispatchMessage(core::ptr::addr_of_mut!(msg));
            continue;
        }

        pt = core::mem::transmute(msg.pt);

        if msg.message == WM_KEYDOWN {
            match msg.wParam as u32 {
                x if x == VK_UP as u32 => pt.y -= 8,
                x if x == VK_DOWN as u32 => pt.y += 8,
                x if x == VK_LEFT as u32 => pt.x -= 8,
                x if x == VK_RIGHT as u32 => pt.x += 8,
                _ => {}
            }
        }

        pt.x = max_l(pt.x, mouseRect.left);
        pt.x = min_l(pt.x, mouseRect.right - 1);
        pt.y = max_l(pt.y, mouseRect.top);
        pt.y = min_l(pt.y, mouseRect.bottom - 1);

        dx = pt.x - capturePoint.x;
        dy = pt.y - capturePoint.y;

        if dx != 0 || dy != 0 {
            if moved == 0 {
                moved = 1;
                if iconic != 0 {
                    OldCursor = UserSetCursor(DragCursor, 0);
                    UserShowCursor(1);
                } else if DragFullWindows == 0 {
                    UserDrawMovingFrame(hdc, core::ptr::addr_of_mut!(sizingRect) as *mut RECTL, thickframe);
                }
            }

            if msg.message == WM_KEYDOWN {
                UserSetCursorPos(pt.x, pt.y, 0, 0, 0);
            } else {
                let mut newRect: RECT = unmodRect;

                if iconic == 0 && DragFullWindows == 0 {
                    UserDrawMovingFrame(hdc, pFrameRect as *mut RECTL, thickframe);
                    pFrameRect = core::ptr::addr_of_mut!(sizingRect);
                }
                if hittest == HTCAPTION as LONG {
                    let pr: PRECT = core::ptr::addr_of_mut!(newRect);
                    let width: LONG;
                    let height: LONG;
                    let capcy: LONG;
                    let snapTo: LONG;
                    if snap != 0 && syscommand == SC_MOVE as WPARAM && iconic == 0
                        && RECTL_bIsEmptyRect(core::ptr::addr_of_mut!((*pwnd).InternalPos.NormalRect) as *mut _) == 0 {
                        *pr = core::mem::transmute((*pwnd).InternalPos.NormalRect);
                        origRect = *pr;

                        capcy = UserGetSystemMetrics(if (ExStyle & WS_EX_TOPMOST as ULONG) != 0 { SM_CYSMCAPTION } else { SM_CYCAPTION });
                        width = (*pr).right - (*pr).left;
                        height = (*pr).bottom - (*pr).top;
                        (*pr).left = pt.x - width / 2;
                        (*pr).right = (*pr).left + width;
                        (*pr).top = mouseRect.top;
                        (*pr).bottom = (*pr).top + height;
                        if (*pr).left < mouseRect.left {
                            (*pr).left = mouseRect.left;
                            (*pr).right = (*pr).left + width;
                        }
                        if ((*pwnd).ExStyle & WS_EX_LAYOUTRTL as ULONG) != 0 && (*pr).right > mouseRect.right {
                            (*pr).left = mouseRect.right - width;
                            (*pr).right = (*pr).left + width;
                        }
                        UserSetCursorPos(pt.x, (*pr).top + capcy / 2, 0, 0, 0);
                        snap = 0;
                        dx = 0;
                        dy = 0;
                        if DragFullWindows != 0 {
                            IntSetStyle(pwnd, 0, WS_MAXIMIZE as ULONG);
                            IntSetSnapEdge(pwnd, HTNOWHERE);
                            co_WinPosSetWindowPos(pwnd, HWND_TOP, (*pr).left, (*pr).top, width, height, SWP_NOACTIVATE as UINT);
                        }
                    } else if snap == 0 && syscommand == SC_MOVE as WPARAM && iconic == 0 {
                        snapTo = GetSnapActivationPoint(pwnd, pt) as LONG;
                        if snapTo != 0 {
                            co_IntCalculateSnapPosition(pwnd, snapTo as UINT, core::ptr::addr_of_mut!(snapPreviewRect) as _);
                            if DragFullWindows != 0 {
                            } else {
                                pFrameRect = core::ptr::addr_of_mut!(snapPreviewRect);
                                UserDrawMovingFrame(hdc, pFrameRect as *mut RECTL, thickframe);
                                continue;
                            }
                        }
                    }

                    RECTL_vOffsetRect(core::ptr::addr_of_mut!(newRect) as *mut _, dx, dy);
                }
                if on_left(hittest) {
                    newRect.left += dx;
                } else if on_right(hittest) {
                    newRect.right += dx;
                }
                if on_top(hittest) {
                    newRect.top += dy;
                } else if on_bottom(hittest) {
                    newRect.bottom += dy;
                }

                capturePoint = pt;

                unmodRect = newRect;

                if syscommand == SC_SIZE as WPARAM {
                    let mut wpSizingHit: WPARAM = 0;
                    if hittest >= HTLEFT as LONG && hittest <= HTBOTTOMRIGHT as LONG {
                        wpSizingHit = (WMSZ_LEFT as LONG + (hittest - HTLEFT as LONG)) as WPARAM;
                    }
                    co_IntSendMessage(UserHMGetHandle(pwnd), WM_SIZING, wpSizingHit, core::ptr::addr_of_mut!(newRect) as LPARAM);
                } else {
                    co_IntSendMessage(UserHMGetHandle(pwnd), WM_MOVING, 0, core::ptr::addr_of_mut!(newRect) as LPARAM);
                }

                if iconic == 0 {
                    if DragFullWindows == 0 {
                        UserDrawMovingFrame(hdc, core::ptr::addr_of_mut!(newRect) as *mut RECTL, thickframe);
                    } else {
                        let hrgnNew: HRGN;
                        let mut hrgnOrig: HRGN = GreCreateRectRgnIndirect(core::ptr::addr_of_mut!((*pwnd).rcWindow));

                        if !(*pwnd).hrgnClip.is_null() {
                            NtGdiCombineRgn(hrgnOrig, hrgnOrig, (*pwnd).hrgnClip, RGN_AND as i32);
                        }

                        co_WinPosSetWindowPos(pwnd,
                            core::ptr::null_mut(),
                            newRect.left,
                            newRect.top,
                            newRect.right - newRect.left,
                            newRect.bottom - newRect.top,
                            (SWP_NOACTIVATE | if hittest == HTCAPTION as LONG { SWP_NOSIZE } else { 0 }) as UINT);

                        hrgnNew = GreCreateRectRgnIndirect(core::ptr::addr_of_mut!((*pwnd).rcWindow));
                        if !(*pwnd).hrgnClip.is_null() {
                            NtGdiCombineRgn(hrgnNew, hrgnNew, (*pwnd).hrgnClip, RGN_AND as i32);
                        }

                        if !hrgnNew.is_null() {
                            if !hrgnOrig.is_null() {
                                NtGdiCombineRgn(hrgnOrig, hrgnOrig, hrgnNew, RGN_DIFF as i32);
                            }
                        } else if !hrgnOrig.is_null() {
                            GreDeleteObject(hrgnOrig as *mut _);
                            hrgnOrig = core::ptr::null_mut();
                        }

                        UpdateThreadWindows((*UserGetDesktopWindow()).spwndChild, pti, hrgnOrig);

                        if !hrgnOrig.is_null() {
                            GreDeleteObject(hrgnOrig as *mut _);
                        }
                        if !hrgnNew.is_null() {
                            GreDeleteObject(hrgnNew as *mut _);
                        }
                    }
                }
                sizingRect = newRect;
            }
        }
    }

    (*head_pti(core::ptr::addr_of!((*pwnd).head))).TIF_flags &= !TIF_MOVESIZETRACKING;

    IntReleaseCapture();

    if iconic != 0 {
        if moved != 0 {
            UserShowCursor(0);
            OldCursor = UserSetCursor(OldCursor, 0);
        }
        if !OldCursor.is_null() {
            UserDereferenceObject(OldCursor as *mut _);
        }
    } else {
        let eraseFinalFrame: UINT = if moved != 0 && DragFullWindows == 0 { 1 } else { 0 };
        if eraseFinalFrame != 0 {
            UserDrawMovingFrame(hdc, pFrameRect as *mut RECTL, thickframe);
        }
    }

    UserReleaseDC(core::ptr::null_mut(), hdc, 0);

    if co_HOOK_CallHooks(WH_CBT as i32, HCBT_MOVESIZE as i32, UserHMGetHandle(pwnd) as WPARAM, core::ptr::addr_of_mut!(sizingRect) as LPARAM) != 0 {
        moved = 0;
    }

    IntNotifyWinEvent(EVENT_SYSTEM_MOVESIZEEND, pwnd, OBJID_WINDOW as LONG, CHILDID_SELF as LONG, 0);

    MsqSetStateWindow(pti, MSQ_STATE_MOVESIZE, core::ptr::null_mut());

    co_IntSendMessage(UserHMGetHandle(pwnd), WM_EXITSIZEMOVE, 0, 0);
    co_IntSendMessage(UserHMGetHandle(pwnd), WM_SETVISIBLE, (((*pwnd).style & WS_MINIMIZE as ULONG) != 0) as WPARAM, 0);

    if moved != 0 {
        let forceSizing: BOOL = if iconic == 0 && hittest == HTCAPTION as LONG && ((orgSnap != 0) != (snap != 0)) { 1 } else { 0 };
        let swp: UINT = if forceSizing == 0 && hittest == HTCAPTION as LONG { SWP_NOSIZE as UINT } else { 0 };

        if !((msg.message == WM_KEYDOWN) && (msg.wParam == VK_ESCAPE as WPARAM)) {
            if DragFullWindows == 0 || iconic != 0 {
                if snap != 0 {
                    co_IntSnapWindow(pwnd, snap);
                } else {
                    if orgSnap != 0 && snap == 0 {
                        IntSetStyle(pwnd, 0, WS_MAXIMIZE as ULONG);
                        IntSetSnapInfo(pwnd, HTNOWHERE, core::ptr::null_mut());
                    }
                    co_WinPosSetWindowPos(pwnd, HWND_TOP, sizingRect.left, sizingRect.top,
                        sizingRect.right - sizingRect.left,
                        sizingRect.bottom - sizingRect.top, swp);
                }
            }
        } else {
            if orgSnap != 0 {
                co_IntSnapWindow(pwnd, orgSnap);
            } else if DragFullWindows != 0 {
                co_WinPosSetWindowPos(pwnd, HWND_TOP, origRect.left, origRect.top,
                    origRect.right - origRect.left,
                    origRect.bottom - origRect.top, swp);
            }
        }
    }

    if IntIsWindow(UserHMGetHandle(pwnd)) != 0 {
        if iconic != 0 && moved == 0 && (Style & WS_SYSMENU as ULONG) != 0 {
            co_IntSendMessage(UserHMGetHandle(pwnd), WM_SYSCOMMAND, (SC_MOUSEMENU + HTSYSMENU) as WPARAM, MAKELONG(pt.x as WORD, pt.y as WORD) as LPARAM);
        }
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn NC_IconForWindow(pWnd: PWND) -> PCURICON_OBJECT {
    NC_IconForWindow_impl(pWnd)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn NC_IconForWindow(pWnd: PWND) -> PCURICON_OBJECT {
    NC_IconForWindow_impl(pWnd)
}

unsafe fn NC_IconForWindow_impl(pWnd: PWND) -> PCURICON_OBJECT {
    let mut pIcon: PCURICON_OBJECT = core::ptr::null_mut();
    let mut hIcon: HICON;

    hIcon = UserGetProp(pWnd, (*gpsi).atomIconSmProp, 1u8) as HICON;
    if hIcon.is_null() {
        hIcon = UserGetProp(pWnd, (*gpsi).atomIconProp, 1u8) as HICON;
    }

    if hIcon.is_null() && !(*(*pWnd).pcls).spicnSm.is_null() {
        return (*(*pWnd).pcls).spicnSm;
    }
    if hIcon.is_null() && !(*(*pWnd).pcls).spicn.is_null() {
        return (*(*pWnd).pcls).spicn;
    }

    if hIcon.is_null() && ((*pWnd).ExStyle & WS_EX_DLGMODALFRAME) == 0 {
        hIcon = (*gpsi_per_user()).hIconSmWindows;
        if hIcon.is_null() {
            hIcon = (*gpsi_per_user()).hIconWindows;
        }
    }

    if !hIcon.is_null() {
        pIcon = UserGetObjectNoErr(gHandleTable, hIcon as _, TYPE_CURSOR as _) as PCURICON_OBJECT;
    }

    pIcon
}

#[no_mangle]
pub unsafe extern "C" fn UserDrawSysMenuButton(
    pWnd: PWND,
    hDC: HDC,
    Rect: LPRECT,
    Down: BOOL,
) -> BOOL {
    let WindowIcon: PCURICON_OBJECT;
    let mut Ret: BOOL = 0;

    WindowIcon = NC_IconForWindow(pWnd);
    if !WindowIcon.is_null() {
        UserReferenceObject(WindowIcon as _);

        Ret = UserDrawIconEx(
            hDC,
            (*Rect).left + 2,
            (*Rect).top + 2,
            WindowIcon,
            UserGetSystemMetrics(SM_CXSMICON as _),
            UserGetSystemMetrics(SM_CYSMICON as _),
            0,
            core::ptr::null_mut(),
            DI_NORMAL,
        );

        UserDereferenceObject(WindowIcon as _);
    }
    Ret
}

unsafe fn IntIsScrollBarVisible(pWnd: PWND, hBar: INT) -> BOOL {
    let mut sbi: SCROLLBARINFO = core::mem::zeroed();
    sbi.cbSize = core::mem::size_of::<SCROLLBARINFO>() as u32;

    if co_IntGetScrollBarInfo(pWnd, hBar, core::ptr::addr_of_mut!(sbi)) == 0 {
        return 0;
    }

    if (sbi.rgstate[0] & STATE_SYSTEM_OFFSCREEN) != 0 {
        0
    } else {
        1
    }
}

unsafe fn UserDrawCaptionButton(pWnd: PWND, Rect: LPRECT, Style: DWORD, ExStyle: DWORD, hDC: HDC, bDown: BOOL, Type: ULONG) {
    let mut TempRect: RECT;

    if (Style & WS_SYSMENU) == 0 {
        return;
    }

    TempRect = *Rect;

    if Type == DFCS_CAPTIONMIN {
        if (ExStyle & WS_EX_TOOLWINDOW) != 0 {
            return;
        }

        if (Style & WS_SYSMENU) != 0 {
            TempRect.right -= UserGetSystemMetrics(SM_CXSIZE as ULONG) + 1;
        }

        if (Style & (WS_MAXIMIZEBOX | WS_MINIMIZEBOX)) != 0 {
            TempRect.right -= UserGetSystemMetrics(SM_CXSIZE as ULONG) - 2;
        }

        TempRect.left = TempRect.right - UserGetSystemMetrics(SM_CXSIZE as ULONG) + 1;
        TempRect.bottom = TempRect.top + UserGetSystemMetrics(SM_CYSIZE as ULONG) - 2;
        TempRect.top += 2;
        TempRect.right -= 1;

        DrawFrameControl(
            hDC,
            core::ptr::addr_of_mut!(TempRect),
            DFC_CAPTION,
            (if (Style & WS_MINIMIZE) != 0 { DFCS_CAPTIONRESTORE } else { DFCS_CAPTIONMIN })
                | (if bDown != 0 { DFCS_PUSHED } else { 0 })
                | (if (Style & WS_MINIMIZEBOX) != 0 { 0 } else { DFCS_INACTIVE }),
        );
    } else if Type == DFCS_CAPTIONMAX {
        if (ExStyle & WS_EX_TOOLWINDOW) != 0 {
            return;
        }

        if (Style & WS_SYSMENU) != 0 {
            TempRect.right -= UserGetSystemMetrics(SM_CXSIZE as ULONG) + 1;
        }

        TempRect.left = TempRect.right - UserGetSystemMetrics(SM_CXSIZE as ULONG) + 1;
        TempRect.bottom = TempRect.top + UserGetSystemMetrics(SM_CYSIZE as ULONG) - 2;
        TempRect.top += 2;
        TempRect.right -= 1;

        DrawFrameControl(
            hDC,
            core::ptr::addr_of_mut!(TempRect),
            DFC_CAPTION,
            (if (Style & WS_MAXIMIZE) != 0 { DFCS_CAPTIONRESTORE } else { DFCS_CAPTIONMAX })
                | (if bDown != 0 { DFCS_PUSHED } else { 0 })
                | (if (Style & WS_MAXIMIZEBOX) != 0 { 0 } else { DFCS_INACTIVE }),
        );
    } else if Type == DFCS_CAPTIONCLOSE {
        let pSysMenu: PMENU = IntGetSystemMenu(pWnd, 0);
        let MenuState: UINT = IntGetMenuState(
            if !pSysMenu.is_null() { UserHMGetHandle(pSysMenu) as _ } else { core::ptr::null_mut() },
            SC_CLOSE,
            MF_BYCOMMAND,
        );

        if (ExStyle & WS_EX_TOOLWINDOW) != 0 {
            TempRect.left = TempRect.right - UserGetSystemMetrics(SM_CXSMSIZE as ULONG);
            TempRect.bottom = TempRect.top + UserGetSystemMetrics(SM_CYSMSIZE as ULONG) - 2;
        } else {
            TempRect.left = TempRect.right - UserGetSystemMetrics(SM_CXSIZE as ULONG);
            TempRect.bottom = TempRect.top + UserGetSystemMetrics(SM_CYSIZE as ULONG) - 2;
        }
        TempRect.top += 2;
        TempRect.right -= 2;

        DrawFrameControl(
            hDC,
            core::ptr::addr_of_mut!(TempRect),
            DFC_CAPTION,
            DFCS_CAPTIONCLOSE
                | (if bDown != 0 { DFCS_PUSHED } else { 0 })
                | (if (MenuState & (MF_GRAYED | MF_DISABLED)) == 0
                    && ((*(*pWnd).pcls).style & CS_NOCLOSE) == 0
                {
                    0
                } else {
                    DFCS_INACTIVE
                }),
        );
    }
}

unsafe fn UserDrawCaptionButtonWnd(pWnd: PWND, hDC: HDC, bDown: BOOL, Type: ULONG) {
    let mut WindowRect: RECT = core::mem::zeroed();
    let mut WindowBorder: SIZE = core::mem::zeroed();

    IntGetWindowRect(pWnd, core::ptr::addr_of_mut!(WindowRect) as _);

    WindowRect.right -= WindowRect.left;
    WindowRect.bottom -= WindowRect.top;
    WindowRect.left = 0;
    WindowRect.top = 0;

    UserGetWindowBorders((*pWnd).style, (*pWnd).ExStyle, &mut WindowBorder, 0);

    RECTL_vInflateRect_g(&mut WindowRect, -WindowBorder.cx, -WindowBorder.cy);

    UserDrawCaptionButton(pWnd, &mut WindowRect, (*pWnd).style, (*pWnd).ExStyle, hDC, bDown, Type);
}

unsafe fn NC_DrawFrame(hDC: HDC, CurrentRect: *mut RECT, Active: BOOL, Style: DWORD, ExStyle: DWORD) {

    /* Firstly the "thick" frame */
    if (Style & WS_THICKFRAME) != 0 && (Style & WS_MINIMIZE) == 0 {
        let Width: LONG =
            (UserGetSystemMetrics(SM_CXFRAME as ULONG) - UserGetSystemMetrics(SM_CXDLGFRAME as ULONG)) *
            UserGetSystemMetrics(SM_CXBORDER as ULONG);

        let Height: LONG =
            (UserGetSystemMetrics(SM_CYFRAME as ULONG) - UserGetSystemMetrics(SM_CYDLGFRAME as ULONG)) *
            UserGetSystemMetrics(SM_CYBORDER as ULONG);

        NtGdiSelectBrush(hDC, IntGetSysColorBrush(if Active != 0 { COLOR_ACTIVEBORDER as i32 } else { COLOR_INACTIVEBORDER as i32 }) as _);

        NtGdiPatBlt(hDC, (*CurrentRect).left, (*CurrentRect).top, (*CurrentRect).right - (*CurrentRect).left, Height, PATCOPY as DWORD);
        NtGdiPatBlt(hDC, (*CurrentRect).left, (*CurrentRect).top, Width, (*CurrentRect).bottom - (*CurrentRect).top, PATCOPY as DWORD);
        NtGdiPatBlt(hDC, (*CurrentRect).left, (*CurrentRect).bottom, (*CurrentRect).right - (*CurrentRect).left, -Height, PATCOPY as DWORD);
        NtGdiPatBlt(hDC, (*CurrentRect).right, (*CurrentRect).top, -Width, (*CurrentRect).bottom - (*CurrentRect).top, PATCOPY as DWORD);

        RECTL_vInflateRect_g(CurrentRect, -Width, -Height);
    }

    /* Now the other bit of the frame */
    if (Style & (WS_DLGFRAME | WS_BORDER)) != 0 || (ExStyle & WS_EX_DLGMODALFRAME) != 0 {
        let Width: LONG = UserGetSystemMetrics(SM_CXBORDER as ULONG);
        let Height: LONG = UserGetSystemMetrics(SM_CYBORDER as ULONG);

        NtGdiSelectBrush(hDC, IntGetSysColorBrush(
            if (ExStyle & (WS_EX_DLGMODALFRAME | WS_EX_CLIENTEDGE)) != 0 { COLOR_3DFACE as i32 }
            else if (ExStyle & WS_EX_STATICEDGE) != 0 { COLOR_WINDOWFRAME as i32 }
            else if (Style & (WS_DLGFRAME | WS_THICKFRAME)) != 0 { COLOR_3DFACE as i32 }
            else { COLOR_WINDOWFRAME as i32 }) as _);

        NtGdiPatBlt(hDC, (*CurrentRect).left, (*CurrentRect).top, (*CurrentRect).right - (*CurrentRect).left, Height, PATCOPY as DWORD);
        NtGdiPatBlt(hDC, (*CurrentRect).left, (*CurrentRect).top, Width, (*CurrentRect).bottom - (*CurrentRect).top, PATCOPY as DWORD);
        NtGdiPatBlt(hDC, (*CurrentRect).left, (*CurrentRect).bottom, (*CurrentRect).right - (*CurrentRect).left, -Height, PATCOPY as DWORD);
        NtGdiPatBlt(hDC, (*CurrentRect).right, (*CurrentRect).top, -Width, (*CurrentRect).bottom - (*CurrentRect).top, PATCOPY as DWORD);

        RECTL_vInflateRect_g(CurrentRect, -Width, -Height);
    }
}

#[no_mangle]
unsafe extern "C" fn UserDrawCaptionBar(pWnd: PWND, hDC: HDC, mut Flags: INT) {

    let Style: DWORD;
    let ExStyle: DWORD;
    let mut WindowRect: RECT = core::mem::zeroed();
    let mut CurrentRect: RECT = core::mem::zeroed();
    let mut TempRect: RECT;
    let PreviousPen: HPEN;
    let mut Gradient: BOOL = 0;
    let mut pIcon: PCURICON_OBJECT = core::ptr::null_mut();

    if (Flags & DC_NOVISIBLE as i32) == 0 && IntIsWindowVisible(pWnd) == 0 {
        return;
    }

    Style = (*pWnd).style;
    ExStyle = (*pWnd).ExStyle;

    IntGetWindowRect(pWnd, core::ptr::addr_of_mut!(WindowRect) as _);

    CurrentRect.top = 0;
    CurrentRect.left = 0;
    CurrentRect.right = WindowRect.right - WindowRect.left;
    CurrentRect.bottom = WindowRect.bottom - WindowRect.top;

    /* Draw outer edge */
    if UserHasWindowEdge(Style, ExStyle) != 0 {
        DrawEdge(hDC, &mut CurrentRect, EDGE_RAISED, BF_RECT | BF_ADJUST);
    } else if (ExStyle & WS_EX_STATICEDGE) != 0 {
        NtGdiSelectBrush(hDC, IntGetSysColorBrush(COLOR_BTNSHADOW as i32) as _);
        NtGdiPatBlt(hDC, CurrentRect.left, CurrentRect.top, CurrentRect.right - CurrentRect.left, 1, PATCOPY);
        NtGdiPatBlt(hDC, CurrentRect.left, CurrentRect.top, 1, CurrentRect.bottom - CurrentRect.top, PATCOPY);

        NtGdiSelectBrush(hDC, IntGetSysColorBrush(COLOR_BTNHIGHLIGHT as i32) as _);
        NtGdiPatBlt(hDC, CurrentRect.left, CurrentRect.bottom - 1, CurrentRect.right - CurrentRect.left, 1, PATCOPY);
        NtGdiPatBlt(hDC, CurrentRect.right - 1, CurrentRect.top, 1, CurrentRect.bottom - CurrentRect.top, PATCOPY);

        RECTL_vInflateRect_g(&mut CurrentRect, -1, -1);
    }

    if (Flags & DC_FRAME as i32) != 0 {
        NC_DrawFrame(hDC, &mut CurrentRect, Flags & DC_ACTIVE as i32, Style, ExStyle);
    }

    /* Draw caption */
    if (Style & WS_CAPTION) == WS_CAPTION {
        TempRect = CurrentRect;

        Flags |= (DC_TEXT | DC_BUTTONS) as i32;

        if UserSystemParametersInfo(SPI_GETGRADIENTCAPTIONS, 0, &mut Gradient as *mut _ as *mut _, 0) != 0 && Gradient != 0 {
            Flags |= DC_GRADIENT as i32;
        }

        if (ExStyle & WS_EX_TOOLWINDOW) != 0 {
            Flags |= DC_SMALLCAP as i32;
            TempRect.bottom = TempRect.top + UserGetSystemMetrics(SM_CYSMCAPTION as ULONG) - 1;
            CurrentRect.top += UserGetSystemMetrics(SM_CYSMCAPTION as ULONG);
        } else {
            TempRect.bottom = TempRect.top + UserGetSystemMetrics(SM_CYCAPTION as ULONG) - 1;
            CurrentRect.top += UserGetSystemMetrics(SM_CYCAPTION as ULONG);
        }

        if (Flags & DC_ICON as i32) == 0
            && (Flags & DC_SMALLCAP as i32) == 0
            && (Style & WS_SYSMENU) != 0
            && (ExStyle & WS_EX_TOOLWINDOW) == 0
        {
            pIcon = NC_IconForWindow(pWnd);
        }

        let hIcon = if !pIcon.is_null() {
            UserHMGetHandle(pIcon)
        } else {
            core::ptr::null_mut()
        };
        UserDrawCaption(pWnd, hDC, &mut TempRect as *mut _ as _, core::ptr::null_mut(), hIcon as _, core::ptr::null_mut(), Flags as _);

        /* Draw buttons */
        if (Style & WS_SYSMENU) != 0 {
            UserDrawCaptionButton(pWnd, &mut TempRect, Style, ExStyle, hDC, 0, DFCS_CAPTIONCLOSE);
            if (Style & (WS_MAXIMIZEBOX | WS_MINIMIZEBOX)) != 0 && (ExStyle & WS_EX_TOOLWINDOW) == 0 {
                UserDrawCaptionButton(pWnd, &mut TempRect, Style, ExStyle, hDC, 0, DFCS_CAPTIONMIN);
                UserDrawCaptionButton(pWnd, &mut TempRect, Style, ExStyle, hDC, 0, DFCS_CAPTIONMAX);
            }
        }

        if (Style & WS_MINIMIZE) == 0 {
            /* Line under caption */
            PreviousPen = NtGdiSelectPen(hDC, NtGdiGetStockObject(DC_PEN as i32) as _);

            let color = if (ExStyle & (WS_EX_STATICEDGE | WS_EX_CLIENTEDGE | WS_EX_DLGMODALFRAME)) == WS_EX_STATICEDGE {
                COLOR_WINDOWFRAME
            } else {
                COLOR_3DFACE
            };
            IntSetDCPenColor(hDC, IntGetSysColor(color as i32));

            GreMoveTo(hDC, TempRect.left, TempRect.bottom, core::ptr::null_mut());

            NtGdiLineTo(hDC, TempRect.right, TempRect.bottom);

            NtGdiSelectPen(hDC, PreviousPen);
        }
    }

    if (Style & WS_MINIMIZE) == 0 {
        /* Draw menu bar */
        if ((*pWnd).state & WNDS_HASMENU) != 0 && (*pWnd).IDMenu != 0 {
            let menu: PMENU = UserGetMenuObject(UlongToHandle((*pWnd).IDMenu as u32) as _);
            if !menu.is_null() {
                TempRect = CurrentRect;
                TempRect.bottom = TempRect.top + (*menu).cyMenu as LONG;
                CurrentRect.top += MENU_DrawMenuBar(hDC, &mut TempRect, pWnd, 0) as LONG;
            }
        }

        if (ExStyle & WS_EX_CLIENTEDGE) != 0 {
            DrawEdge(hDC, &mut CurrentRect, EDGE_SUNKEN, BF_RECT | BF_ADJUST);
        }
    }
}

#[no_mangle]
unsafe extern "C" fn NC_DoNCPaint(pWnd: PWND, hDC: HDC, mut Flags: INT) -> LRESULT {

    // winuser.h: DC_NC = DC_CAPTION | DC_FRAME (0x900C). The translation had
    // invented DC_FRAME|DC_ACTIVE here, dropping caption icon/text/buttons.
    const DC_NC: DWORD = DC_CAPTION | DC_FRAME;

    let Style: DWORD;
    let ExStyle: DWORD;
    let Parent: PWND;
    let mut WindowRect: RECT = core::mem::zeroed();
    let mut CurrentRect: RECT = core::mem::zeroed();
    let mut TempRect: RECT;
    let mut Active: BOOL = 0;

    if IntIsWindowVisible(pWnd) == 0
        || (((*pWnd).state & WNDS_NONCPAINT) != 0 && ((*pWnd).state & WNDS_FORCEMENUDRAW) == 0)
        || IntEqualRect(
            core::ptr::addr_of_mut!((*pWnd).rcWindow),
            core::ptr::addr_of_mut!((*pWnd).rcClient),
        ) != 0
    {
        return 0;
    }

    Style = (*pWnd).style;

    Parent = IntGetParent(pWnd);
    ExStyle = (*pWnd).ExStyle;

    if Flags == -1 {
        if (ExStyle & WS_EX_MDICHILD) != 0 {
            Active = IntIsChildWindow((*gpqForeground).spwndActive, pWnd);

            if Active != 0 {
                Active = (UserHMGetHandle(pWnd)
                    == co_IntSendMessage(UserHMGetHandle(Parent), WM_MDIGETACTIVE, 0, 0) as HWND)
                    as BOOL;
            }
        } else {
            Active = (gpqForeground == (*head_pti(core::ptr::addr_of!((*pWnd).head))).MessageQueue) as BOOL;
        }
        Flags = DC_NC as INT;
    } else {
        Flags |= DC_NC as INT;
    }

    IntGetWindowRect(pWnd, core::ptr::addr_of_mut!(WindowRect) as _);

    CurrentRect.top = 0;
    CurrentRect.left = 0;
    CurrentRect.right = WindowRect.right - WindowRect.left;
    CurrentRect.bottom = WindowRect.bottom - WindowRect.top;

    if UserHasWindowEdge((*pWnd).style, (*pWnd).ExStyle) != 0 {
        DrawEdge(
            hDC,
            core::ptr::addr_of_mut!(CurrentRect),
            EDGE_RAISED,
            BF_RECT | BF_ADJUST,
        );
    } else if ((*pWnd).ExStyle & WS_EX_STATICEDGE) != 0 {
        NtGdiSelectBrush(hDC, IntGetSysColorBrush(COLOR_BTNSHADOW as i32) as _);
        NtGdiPatBlt(
            hDC,
            CurrentRect.left,
            CurrentRect.top,
            CurrentRect.right - CurrentRect.left,
            1,
            PATCOPY,
        );
        NtGdiPatBlt(
            hDC,
            CurrentRect.left,
            CurrentRect.top,
            1,
            CurrentRect.bottom - CurrentRect.top,
            PATCOPY,
        );

        NtGdiSelectBrush(hDC, IntGetSysColorBrush(COLOR_BTNHIGHLIGHT as i32) as _);
        NtGdiPatBlt(
            hDC,
            CurrentRect.left,
            CurrentRect.bottom - 1,
            CurrentRect.right - CurrentRect.left,
            1,
            PATCOPY,
        );
        NtGdiPatBlt(
            hDC,
            CurrentRect.right - 1,
            CurrentRect.top,
            1,
            CurrentRect.bottom - CurrentRect.top,
            PATCOPY,
        );

        RECTL_vInflateRect_g(core::ptr::addr_of_mut!(CurrentRect), -1, -1);
    }

    if (Flags as DWORD & DC_FRAME) != 0 {
        NC_DrawFrame(
            hDC,
            core::ptr::addr_of_mut!(CurrentRect),
            if Active != 0 {
                Active
            } else {
                (Flags as DWORD & DC_ACTIVE) as BOOL
            },
            Style,
            ExStyle,
        );
    }

    if (Style & WS_CAPTION) == WS_CAPTION {
        let PreviousPen: HPEN;
        let mut Gradient: BOOL = 0;

        if (Flags as DWORD & DC_REDRAWHUNGWND) != 0 {
            Flags &= !(DC_REDRAWHUNGWND as INT);
            Flags |= DC_NOSENDMSG as INT;
        }

        if UserSystemParametersInfo(
            SPI_GETGRADIENTCAPTIONS,
            0,
            core::ptr::addr_of_mut!(Gradient) as *mut core::ffi::c_void,
            0,
        ) != 0
            && Gradient != 0
        {
            Flags |= DC_GRADIENT as INT;
        }

        if Active != 0 {
            if ((*pWnd).state & WNDS_ACTIVEFRAME) != 0 {
                Flags |= DC_ACTIVE as INT;
            }
        }

        TempRect = CurrentRect;

        if (ExStyle & WS_EX_TOOLWINDOW) != 0 {
            Flags |= DC_SMALLCAP as INT;
            TempRect.bottom = TempRect.top + UserGetSystemMetrics(SM_CYSMCAPTION as ULONG) - 1;
            CurrentRect.top += UserGetSystemMetrics(SM_CYSMCAPTION as ULONG);
        } else {
            TempRect.bottom = TempRect.top + UserGetSystemMetrics(SM_CYCAPTION as ULONG) - 1;
            CurrentRect.top += UserGetSystemMetrics(SM_CYCAPTION as ULONG);
        }

        UserDrawCaption(
            pWnd,
            hDC,
            core::ptr::addr_of_mut!(TempRect) as _,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            Flags as DWORD,
        );

        if (Style & WS_SYSMENU) != 0 {
            UserDrawCaptionButton(
                pWnd,
                core::ptr::addr_of_mut!(TempRect),
                Style,
                ExStyle,
                hDC,
                0,
                DFCS_CAPTIONCLOSE,
            );
            if (Style & (WS_MAXIMIZEBOX | WS_MINIMIZEBOX)) != 0
                && (ExStyle & WS_EX_TOOLWINDOW) == 0
            {
                UserDrawCaptionButton(
                    pWnd,
                    core::ptr::addr_of_mut!(TempRect),
                    Style,
                    ExStyle,
                    hDC,
                    0,
                    DFCS_CAPTIONMIN,
                );
                UserDrawCaptionButton(
                    pWnd,
                    core::ptr::addr_of_mut!(TempRect),
                    Style,
                    ExStyle,
                    hDC,
                    0,
                    DFCS_CAPTIONMAX,
                );
            }
        }
        if (Style & WS_MINIMIZE) == 0 {
            PreviousPen = NtGdiSelectPen(hDC, NtGdiGetStockObject(DC_PEN as i32) as _) as HPEN;

            IntSetDCPenColor(
                hDC,
                IntGetSysColor(
                    if (ExStyle
                        & (WS_EX_STATICEDGE | WS_EX_CLIENTEDGE | WS_EX_DLGMODALFRAME))
                        == WS_EX_STATICEDGE
                    {
                        COLOR_WINDOWFRAME as i32
                    } else {
                        COLOR_3DFACE as i32
                    },
                ),
            );

            GreMoveTo(hDC, TempRect.left, TempRect.bottom, core::ptr::null_mut());

            NtGdiLineTo(hDC, TempRect.right, TempRect.bottom);

            NtGdiSelectPen(hDC, PreviousPen);
        }
    }

    if (Style & WS_MINIMIZE) == 0 {
        if ((*pWnd).state & WNDS_HASMENU) != 0 && (*pWnd).IDMenu != 0 {
            if (Flags as DWORD & DC_NOSENDMSG) == 0 {
                let menu: PMENU =
                    UserGetMenuObject(UlongToHandle((*pWnd).IDMenu as u32) as HMENU);
                if !menu.is_null() {
                    TempRect = CurrentRect;
                    TempRect.bottom = TempRect.top + (*menu).cyMenu as LONG;
                    CurrentRect.top += MENU_DrawMenuBar(
                        hDC,
                        core::ptr::addr_of_mut!(TempRect),
                        pWnd,
                        0,
                    ) as LONG;
                }
            }
        }

        if (ExStyle & WS_EX_CLIENTEDGE) != 0 {
            DrawEdge(
                hDC,
                core::ptr::addr_of_mut!(CurrentRect),
                EDGE_SUNKEN,
                BF_RECT | BF_ADJUST,
            );
        }

        if (Style & WS_VSCROLL) != 0
            && (Style & WS_HSCROLL) != 0
            && IntIsScrollBarVisible(pWnd, OBJID_VSCROLL as INT) != 0
            && IntIsScrollBarVisible(pWnd, OBJID_HSCROLL as INT) != 0
        {
            let mut ParentClientRect: RECT = core::mem::zeroed();

            TempRect = CurrentRect;

            if (ExStyle & WS_EX_LEFTSCROLLBAR) != 0 {
                TempRect.right = TempRect.left + UserGetSystemMetrics(SM_CXVSCROLL as ULONG);
            } else {
                TempRect.left = TempRect.right - UserGetSystemMetrics(SM_CXVSCROLL as ULONG);
            }

            TempRect.top = TempRect.bottom - UserGetSystemMetrics(SM_CYHSCROLL as ULONG);

            FillRect(
                hDC,
                core::ptr::addr_of_mut!(TempRect),
                IntGetSysColorBrush(COLOR_BTNFACE as i32) as _,
            );

            if !Parent.is_null() {
                IntGetClientRect(Parent, core::ptr::addr_of_mut!(ParentClientRect) as _);

                let has_size_grip = (Style & WS_CHILD) == 0
                    && (Style & WS_THICKFRAME) != 0
                    && (Style & WS_MAXIMIZE) == 0
                    || ((Style & WS_CHILD) != 0
                        && ((*Parent).style & WS_THICKFRAME) != 0
                        && ((*Parent).style & WS_MAXIMIZE) == 0
                        && (WindowRect.right - WindowRect.left == ParentClientRect.right)
                        && (WindowRect.bottom - WindowRect.top == ParentClientRect.bottom));

                if has_size_grip {
                    DrawFrameControl(
                        hDC,
                        core::ptr::addr_of_mut!(TempRect),
                        DFC_SCROLL,
                        DFCS_SCROLLSIZEGRIP,
                    );
                }
            }

            IntDrawScrollBar(pWnd, hDC, SB_VERT as INT);
            IntDrawScrollBar(pWnd, hDC, SB_HORZ as INT);
        } else {
            if (Style & WS_VSCROLL) != 0 && IntIsScrollBarVisible(pWnd, OBJID_VSCROLL as INT) != 0 {
                IntDrawScrollBar(pWnd, hDC, SB_VERT as INT);
            } else if (Style & WS_HSCROLL) != 0
                && IntIsScrollBarVisible(pWnd, OBJID_HSCROLL as INT) != 0
            {
                IntDrawScrollBar(pWnd, hDC, SB_HORZ as INT);
            }
        }
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn NC_HandleNCCalcSize(
    Wnd: PWND,
    wparam: WPARAM,
    Rect: *mut RECTL,
    bSuspended: i32,
) -> LRESULT {
    let mut Result: LRESULT = 0;
    let mut WindowBorders: SIZE = core::mem::zeroed();
    let OrigRect: RECT;
    let Style: i32 = (*Wnd).style as i32;
    let mut exStyle: i32 = (*Wnd).ExStyle as i32;

    if Rect.is_null() {
        return Result;
    }
    OrigRect = *(Rect as *mut RECT);

    (*Wnd).state &= !WNDS_HASCAPTION;

    if wparam != 0 {
        if (*(*Wnd).pcls).style & CS_VREDRAW != 0 {
            Result |= WVR_VREDRAW as LRESULT;
        }
        if (*(*Wnd).pcls).style & CS_HREDRAW != 0 {
            Result |= WVR_HREDRAW as LRESULT;
        }
        Result |= WVR_VALIDRECTS as LRESULT;
    }

    if (*Wnd).style & WS_MINIMIZE == 0 {
        if UserHasWindowEdge((*Wnd).style, (*Wnd).ExStyle) != 0 {
            UserGetWindowBorders((*Wnd).style, (*Wnd).ExStyle, &mut WindowBorders, 0);
            RECTL_vInflateRect_g(Rect, -WindowBorders.cx, -WindowBorders.cy);
        } else if ((*Wnd).ExStyle & WS_EX_STATICEDGE != 0) || ((*Wnd).style & WS_BORDER != 0) {
            RECTL_vInflateRect_g(Rect, -1, -1);
        }

        if (*Wnd).style & WS_CAPTION == WS_CAPTION {
            (*Wnd).state |= WNDS_HASCAPTION;

            if (*Wnd).ExStyle & WS_EX_TOOLWINDOW != 0 {
                (*Rect).top += UserGetSystemMetrics(SM_CYSMCAPTION as ULONG);
            } else {
                (*Rect).top += UserGetSystemMetrics(SM_CYCAPTION as ULONG);
            }
        }

        if HAS_MENU(Wnd, Style as u32) != 0 {
            let hDC = UserGetDCEx(Wnd, 0 as _, (DCX_USESTYLE | DCX_WINDOW) as u32);

            (*Wnd).state |= WNDS_HASMENU;

            if !hDC.is_null() {
                let mut CliRect: RECT = *(Rect as *mut RECT);
                CliRect.bottom -= OrigRect.top;
                CliRect.right -= OrigRect.left;
                CliRect.left -= OrigRect.left;
                CliRect.top -= OrigRect.top;
                if bSuspended == 0 {
                    (*Rect).top += MENU_DrawMenuBar(hDC, &mut CliRect, Wnd, 1) as i32;
                }
                UserReleaseDC(Wnd, hDC, 0);
            }
        }

        if (*Wnd).ExStyle & WS_EX_CLIENTEDGE != 0 {
            RECTL_vInflateRect_g(
                Rect,
                -2 * UserGetSystemMetrics(SM_CXBORDER as ULONG),
                -2 * UserGetSystemMetrics(SM_CYBORDER as ULONG),
            );
        }

        if Style & WS_VSCROLL as i32 != 0 {
            if (*Rect).right - (*Rect).left >= UserGetSystemMetrics(SM_CXVSCROLL as ULONG) {
                (*Wnd).state |= WNDS_HASVERTICALSCROOLLBAR;

                if wparam == 0 && (exStyle & WS_EX_LAYOUTRTL as i32 != 0) {
                    exStyle ^= WS_EX_LEFTSCROLLBAR as i32;
                }

                if exStyle & WS_EX_LEFTSCROLLBAR as i32 != 0 {
                    (*Rect).left += UserGetSystemMetrics(SM_CXVSCROLL as ULONG);
                } else {
                    (*Rect).right -= UserGetSystemMetrics(SM_CXVSCROLL as ULONG);
                }
            }
        }

        if Style & WS_HSCROLL as i32 != 0 {
            if (*Rect).bottom - (*Rect).top > UserGetSystemMetrics(SM_CYHSCROLL as ULONG) {
                (*Wnd).state |= WNDS_HASHORIZONTALSCROLLBAR;

                (*Rect).bottom -= UserGetSystemMetrics(SM_CYHSCROLL as ULONG);
            }
        }

        if (*Rect).top > (*Rect).bottom {
            (*Rect).bottom = (*Rect).top;
        }

        if (*Rect).left > (*Rect).right {
            (*Rect).right = (*Rect).left;
        }
    } else {
        (*Rect).right = (*Rect).left;
        (*Rect).bottom = (*Rect).top;
    }

    Result
}

unsafe fn NC_DoNCActive(Wnd: PWND) -> INT {
    let mut Ret: INT = 0;

    if IntGetSysColor(COLOR_CAPTIONTEXT as i32) != IntGetSysColor(COLOR_INACTIVECAPTIONTEXT as i32)
        || IntGetSysColor(COLOR_ACTIVECAPTION as i32) != IntGetSysColor(COLOR_INACTIVECAPTION as i32)
    {
        Ret = DC_CAPTION as INT;
    }

    if ((*Wnd).style & WS_MINIMIZED as u32) == 0
        && UserHasThickFrameStyle((*Wnd).style, (*Wnd).ExStyle) != 0
    {
        Ret = DC_FRAME as INT;
    }

    Ret
}

#[no_mangle]
unsafe extern "C" fn NC_HandleNCActivate(Wnd: PWND, mut wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let Flags: INT;

    if (wParam & DC_ACTIVE as WPARAM) != 0 {
        (*Wnd).state |= (WNDS_ACTIVEFRAME | WNDS_HASCAPTION) as u32;
        wParam = (DC_CAPTION | DC_ACTIVE) as WPARAM;
    } else {
        (*Wnd).state &= !(WNDS_ACTIVEFRAME as u32);
        wParam = DC_CAPTION as WPARAM;
    }

    if ((*Wnd).state & WNDS_NONCPAINT as u32) != 0 || ((*Wnd).style & WS_VISIBLE as u32) == 0 {
        return 1 as LRESULT;
    }

    if lParam != -1 as LPARAM {
        Flags = NC_DoNCActive(Wnd);
        if Flags != 0 {
            let hDC: HDC;
            let mut hRgnTemp: HRGN = core::ptr::null_mut();
            let hRgn: HRGN = lParam as HRGN;

            if GreIsHandleValid(hRgn as _) != 0 {
                hRgnTemp = NtGdiCreateRectRgn(0, 0, 0, 0);
                if NtGdiCombineRgn(hRgnTemp, hRgn, core::ptr::null_mut(), RGN_COPY as INT) == ERROR as INT {
                    GreDeleteObject(hRgnTemp as _);
                    hRgnTemp = core::ptr::null_mut();
                }
            }

            hDC = UserGetDCEx(Wnd, hRgnTemp as _, (DCX_WINDOW | DCX_USESTYLE) as u32);
            if !hDC.is_null() {
                NC_DoNCPaint(Wnd, hDC, (wParam as INT) | Flags);
                UserReleaseDC(Wnd, hDC, 0u8 as BOOL);
            } else {
                GreDeleteObject(hRgnTemp as _);
            }
        }
    }

    1 as LRESULT
}

unsafe fn NC_DoButton(pWnd: PWND, wParam: WPARAM, lParam: LPARAM) {
    let mut Msg: MSG = core::mem::zeroed();
    let WindowDC: HDC;
    let mut Pressed: BOOL = 1;
    let mut OldState: BOOL;
    let SCMsg: WPARAM;
    let SysMenu: PMENU;
    let ButtonType: ULONG;
    let Style: DWORD;
    let MenuState: UINT;

    Style = (*pWnd).style;
    match wParam as u32 {
        HTCLOSE => {
            SysMenu = IntGetSystemMenu(pWnd, 0);
            MenuState = IntGetMenuState(
                if !SysMenu.is_null() {
                    UserHMGetHandle(SysMenu as *mut _) as _
                } else {
                    core::ptr::null_mut()
                },
                SC_CLOSE,
                MF_BYCOMMAND,
            );
            if (Style & WS_SYSMENU) == 0
                || (MenuState & (MF_GRAYED | MF_DISABLED)) != 0
                || ((*(*pWnd).pcls).style & CS_NOCLOSE) != 0
            {
                return;
            }
            ButtonType = DFCS_CAPTIONCLOSE;
            SCMsg = SC_CLOSE as WPARAM;
        }
        HTMINBUTTON => {
            if (Style & WS_MINIMIZEBOX) == 0 {
                return;
            }
            ButtonType = DFCS_CAPTIONMIN;
            SCMsg = (if (Style & WS_MINIMIZE) != 0 {
                SC_RESTORE
            } else {
                SC_MINIMIZE
            }) as WPARAM;
        }
        HTMAXBUTTON => {
            if (Style & WS_MAXIMIZEBOX) == 0 {
                return;
            }
            ButtonType = DFCS_CAPTIONMAX;
            SCMsg = (if (Style & WS_MAXIMIZE) != 0 {
                SC_RESTORE
            } else {
                SC_MAXIMIZE
            }) as WPARAM;
        }
        _ => {
            return;
        }
    }

    co_IntUpdateWindows(pWnd, RDW_ALLCHILDREN, 0);

    WindowDC = UserGetWindowDC(pWnd);
    UserDrawCaptionButtonWnd(pWnd, WindowDC, 1, ButtonType);

    co_UserSetCapture(UserHMGetHandle(pWnd as *mut _) as _);

    loop {
        if co_IntGetPeekMessage(
            &mut Msg,
            core::ptr::null_mut(),
            WM_MOUSEFIRST,
            WM_MOUSELAST,
            PM_REMOVE,
            1,
        ) == 0
        {
            break;
        }
        if IntCallMsgFilter(&mut Msg, MSGF_MAX as INT) != 0 {
            continue;
        }

        if Msg.message == WM_LBUTTONUP {
            break;
        }

        if Msg.message != WM_MOUSEMOVE {
            continue;
        }

        OldState = Pressed;
        Pressed = (GetNCHitEx(pWnd, core::mem::transmute(Msg.pt)) == wParam as DWORD) as BOOL;
        if Pressed != OldState {
            UserDrawCaptionButtonWnd(pWnd, WindowDC, Pressed, ButtonType);
        }
    }

    if Pressed != 0 {
        UserDrawCaptionButtonWnd(pWnd, WindowDC, 0, ButtonType);
    }
    IntReleaseCapture();
    UserReleaseDC(pWnd, WindowDC, 0);
    if Pressed != 0 {
        co_IntSendMessage(
            UserHMGetHandle(pWnd as *mut _),
            WM_SYSCOMMAND,
            SCMsg,
            if SCMsg == SC_CLOSE as WPARAM {
                lParam
            } else {
                MAKELONG(Msg.pt.x as WORD, Msg.pt.y as WORD) as LPARAM
            },
        );
    }
}

#[no_mangle]
unsafe extern "C" fn NC_HandleNCLButtonDown(pWnd: PWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {

    match wParam as u32 {
        HTCAPTION => {
            let mut TopWnd: PWND = pWnd;
            let mut parent: PWND;
            loop {
                if ((*TopWnd).style & (WS_POPUP | WS_CHILD)) != WS_CHILD {
                    break;
                }
                parent = UserGetAncestor(TopWnd, GA_PARENT as _);
                if parent.is_null() || UserIsDesktopWindow(parent) != 0 {
                    break;
                }
                TopWnd = parent;
            }

            if (!pWnd.is_null() && ((*pWnd).ExStyle & WS_EX_NOACTIVATE) != 0)
                || co_IntSetForegroundWindowMouse(TopWnd) != 0
                || UserGetActiveWindow() == UserHMGetHandle(TopWnd)
            {
                co_IntSendMessage(
                    UserHMGetHandle(pWnd),
                    WM_SYSCOMMAND,
                    (SC_MOVE + HTCAPTION) as WPARAM,
                    lParam,
                );
            }
        }
        HTSYSMENU => {
            let style: LONG = (*pWnd).style as LONG;
            if (style & WS_SYSMENU as LONG) != 0 {
                if (style & WS_MINIMIZE as LONG) == 0 {
                    let mut rect: RECT = core::mem::zeroed();
                    let hDC: HDC = UserGetWindowDC(pWnd);
                    NC_GetInsideRect(pWnd, &mut rect);
                    UserDrawSysMenuButton(pWnd, hDC, &mut rect, 1);
                    UserReleaseDC(pWnd, hDC, 0);
                }
                co_IntSendMessage(
                    UserHMGetHandle(pWnd),
                    WM_SYSCOMMAND,
                    (SC_MOUSEMENU + HTSYSMENU) as WPARAM,
                    lParam,
                );
            }
        }
        HTMENU => {
            co_IntSendMessage(
                UserHMGetHandle(pWnd),
                WM_SYSCOMMAND,
                (SC_MOUSEMENU + HTMENU) as WPARAM,
                lParam,
            );
        }
        HTHSCROLL => {
            co_IntSendMessage(
                UserHMGetHandle(pWnd),
                WM_SYSCOMMAND,
                (SC_HSCROLL + HTHSCROLL) as WPARAM,
                lParam,
            );
        }
        HTVSCROLL => {
            co_IntSendMessage(
                UserHMGetHandle(pWnd),
                WM_SYSCOMMAND,
                (SC_VSCROLL + HTVSCROLL) as WPARAM,
                lParam,
            );
        }
        HTMINBUTTON | HTMAXBUTTON | HTCLOSE => {
            NC_DoButton(pWnd, wParam, lParam);
        }
        HTLEFT | HTRIGHT | HTTOP | HTBOTTOM | HTTOPLEFT | HTTOPRIGHT | HTBOTTOMLEFT
        | HTBOTTOMRIGHT => {
            co_IntSendMessage(
                UserHMGetHandle(pWnd),
                WM_SYSCOMMAND,
                (SC_SIZE as WPARAM) + wParam - ((HTLEFT - WMSZ_LEFT) as WPARAM),
                lParam,
            );
        }
        HTBORDER => {}
        _ => {}
    }
    0
}

#[no_mangle]
unsafe extern "C" fn NC_HandleNCLButtonDblClk(pWnd: PWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {

    let Style: ULONG = (*pWnd).style;
    match wParam as u32 {
        HTCAPTION => {
            if (Style & WS_CAPTION) == WS_CAPTION && (Style & WS_MAXIMIZEBOX) != 0 {
                co_IntSendMessage(
                    UserHMGetHandle(pWnd),
                    WM_SYSCOMMAND,
                    (if (Style & (WS_MINIMIZE | WS_MAXIMIZE)) != 0 {
                        SC_RESTORE
                    } else {
                        SC_MAXIMIZE
                    }) as WPARAM,
                    0,
                );
            }
        }
        HTSYSMENU => {
            let SysMenu: PMENU = IntGetSystemMenu(pWnd, 0);
            let state: UINT = IntGetMenuState(
                if !SysMenu.is_null() {
                    UserHMGetHandle(SysMenu as PWND) as _
                } else {
                    core::ptr::null_mut()
                },
                SC_CLOSE,
                MF_BYCOMMAND,
            );

            if (state & (MF_DISABLED | MF_GRAYED)) != 0 || state == 0xFFFFFFFF {
                return 0;
            }

            co_IntSendMessage(UserHMGetHandle(pWnd), WM_SYSCOMMAND, SC_CLOSE as WPARAM, lParam);
        }
        HTTOP | HTBOTTOM => {
            let sizingRect: RECT = core::mem::transmute((*pWnd).rcWindow);
            let mut mouseRect: RECT = core::mem::zeroed();

            if ((*pWnd).ExStyle & WS_EX_MDICHILD) != 0 {
                return 0;
            }

            UserSystemParametersInfo(
                SPI_GETWORKAREA,
                0,
                core::ptr::addr_of_mut!(mouseRect) as *mut _,
                0,
            );

            co_WinPosSetWindowPos(
                pWnd,
                core::ptr::null_mut(),
                sizingRect.left,
                mouseRect.top,
                sizingRect.right - sizingRect.left,
                mouseRect.bottom - mouseRect.top,
                0,
            );
        }
        _ => {
            return NC_HandleNCLButtonDown(pWnd, wParam, lParam);
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn NC_HandleNCRButtonDown(pwnd: PWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut msg: MSG = core::mem::zeroed();
    let mut hittest: INT = wParam as INT;

    match hittest as u32 {
        HTCAPTION | HTSYSMENU => {
            'block: {
                if IntGetSystemMenu(pwnd, 0).is_null() {
                    break 'block;
                }

                co_UserSetCapture(UserHMGetHandle(pwnd) as _);
                loop {
                    if co_IntGetPeekMessage(&mut msg, core::ptr::null_mut(), WM_MOUSEFIRST, WM_MOUSELAST, PM_REMOVE, 1) == 0 {
                        break;
                    }
                    if IntCallMsgFilter(&mut msg, MSGF_MAX as INT) != 0 {
                        continue;
                    }
                    if msg.message == WM_RBUTTONUP {
                        hittest = GetNCHitEx(pwnd, core::mem::transmute(msg.pt)) as INT;
                        break;
                    }
                    if UserHMGetHandle(pwnd) != IntGetCapture() {
                        return 0;
                    }
                }
                IntReleaseCapture();

                if hittest == HTCAPTION as INT || hittest == HTSYSMENU as INT || hittest == HTHSCROLL as INT || hittest == HTVSCROLL as INT {
                    TRACE!(b"Msg pt %x and Msg.lParam %x and lParam %x\n\0".as_ptr() as *const i8, MAKELONG(msg.pt.x as _, msg.pt.y as _), msg.lParam, lParam);
                    co_IntSendMessage(UserHMGetHandle(pwnd), WM_CONTEXTMENU, UserHMGetHandle(pwnd) as WPARAM, MAKELONG(msg.pt.x as u16, msg.pt.y as u16) as LPARAM);
                }
            }
        }
        _ => {}
    }
    0
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn GetNCHitEx(pWnd: PWND, pt: POINT) -> DWORD {
    GetNCHitEx_impl(pWnd, pt)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn GetNCHitEx(pWnd: PWND, pt: POINT) -> DWORD {
    GetNCHitEx_impl(pWnd, pt)
}

unsafe fn GetNCHitEx_impl(pWnd: PWND, pt: POINT) -> DWORD {
    let mut rcWindow: RECT = core::mem::zeroed();
    let mut rcClient: RECT = core::mem::zeroed();
    let mut Style: DWORD;
    let mut ExStyle: DWORD;

    if pWnd.is_null() {
        return HTNOWHERE as DWORD;
    }

    if UserIsDesktopWindow(pWnd) != 0 {
        rcClient.left = 0;
        rcClient.top = 0;
        rcWindow.left = 0;
        rcWindow.top = 0;
        rcWindow.right = UserGetSystemMetrics(SM_CXSCREEN);
        rcWindow.bottom = UserGetSystemMetrics(SM_CYSCREEN);
        rcClient.right = UserGetSystemMetrics(SM_CXSCREEN);
        rcClient.bottom = UserGetSystemMetrics(SM_CYSCREEN);
    } else {
        rcClient = core::mem::transmute((*pWnd).rcClient);
        rcWindow = core::mem::transmute((*pWnd).rcWindow);
    }

    if RECTL_bPointInRect(core::ptr::addr_of_mut!(rcWindow), pt.x, pt.y) == 0 {
        return HTNOWHERE as DWORD;
    }

    Style = (*pWnd).style;
    ExStyle = (*pWnd).ExStyle;

    if Style & WS_MINIMIZE != 0 {
        return HTCAPTION as DWORD;
    }

    if RECTL_bPointInRect(core::ptr::addr_of_mut!(rcClient), pt.x, pt.y) != 0 {
        return HTCLIENT as DWORD;
    }

    /* Check borders */
    if HAS_THICKFRAME(Style, ExStyle) != 0 {
        RECTL_vInflateRect_g(
            core::ptr::addr_of_mut!(rcWindow),
            -UserGetSystemMetrics(SM_CXFRAME),
            -UserGetSystemMetrics(SM_CYFRAME),
        );
        if RECTL_bPointInRect(core::ptr::addr_of_mut!(rcWindow), pt.x, pt.y) == 0 {
            /* Check top sizing border */
            if pt.y < rcWindow.top {
                if pt.x < rcWindow.left + UserGetSystemMetrics(SM_CXSIZE) {
                    return HTTOPLEFT as DWORD;
                }
                if pt.x >= rcWindow.right - UserGetSystemMetrics(SM_CXSIZE) {
                    return HTTOPRIGHT as DWORD;
                }
                return HTTOP as DWORD;
            }
            /* Check bottom sizing border */
            if pt.y >= rcWindow.bottom {
                if pt.x < rcWindow.left + UserGetSystemMetrics(SM_CXSIZE) {
                    return HTBOTTOMLEFT as DWORD;
                }
                if pt.x >= rcWindow.right - UserGetSystemMetrics(SM_CXSIZE) {
                    return HTBOTTOMRIGHT as DWORD;
                }
                return HTBOTTOM as DWORD;
            }
            /* Check left sizing border */
            if pt.x < rcWindow.left {
                if pt.y < rcWindow.top + UserGetSystemMetrics(SM_CYSIZE) {
                    return HTTOPLEFT as DWORD;
                }
                if pt.y >= rcWindow.bottom - UserGetSystemMetrics(SM_CYSIZE) {
                    return HTBOTTOMLEFT as DWORD;
                }
                return HTLEFT as DWORD;
            }
            /* Check right sizing border */
            if pt.x >= rcWindow.right {
                if pt.y < rcWindow.top + UserGetSystemMetrics(SM_CYSIZE) {
                    return HTTOPRIGHT as DWORD;
                }
                if pt.y >= rcWindow.bottom - UserGetSystemMetrics(SM_CYSIZE) {
                    return HTBOTTOMRIGHT as DWORD;
                }
                return HTRIGHT as DWORD;
            }
        }
    } else {
        /* No thick frame */
        if HAS_DLGFRAME(Style, ExStyle) != 0 {
            RECTL_vInflateRect_g(
                core::ptr::addr_of_mut!(rcWindow),
                -UserGetSystemMetrics(SM_CXDLGFRAME),
                -UserGetSystemMetrics(SM_CYDLGFRAME),
            );
        } else if HAS_THINFRAME(Style, ExStyle) != 0 {
            RECTL_vInflateRect_g(
                core::ptr::addr_of_mut!(rcWindow),
                -UserGetSystemMetrics(SM_CXBORDER),
                -UserGetSystemMetrics(SM_CYBORDER),
            );
        } else if HAS_CLIENTFRAME(Style, ExStyle) != 0 {
            RECTL_vInflateRect_g(
                core::ptr::addr_of_mut!(rcWindow),
                -UserGetSystemMetrics(SM_CXEDGE),
                -UserGetSystemMetrics(SM_CYEDGE),
            );
        }
        if RECTL_bPointInRect(core::ptr::addr_of_mut!(rcWindow), pt.x, pt.y) == 0 {
            return HTBORDER as DWORD;
        }
    }

    /* Check caption */
    if (Style & WS_CAPTION) == WS_CAPTION {
        if ExStyle & WS_EX_TOOLWINDOW != 0 {
            rcWindow.top += UserGetSystemMetrics(SM_CYSMCAPTION) - 1;
        } else {
            rcWindow.top += UserGetSystemMetrics(SM_CYCAPTION) - 1;
        }
        if RECTL_bPointInRect(core::ptr::addr_of_mut!(rcWindow), pt.x, pt.y) == 0 {
            let min_or_max_box: BOOL =
                ((Style & WS_SYSMENU != 0) && (Style & (WS_MINIMIZEBOX | WS_MAXIMIZEBOX) != 0)) as BOOL;
            if ExStyle & WS_EX_LAYOUTRTL != 0 {
                /* Check system menu */
                if (Style & WS_SYSMENU != 0)
                    && (ExStyle & WS_EX_TOOLWINDOW == 0)
                    && !NC_IconForWindow(pWnd).is_null()
                {
                    rcWindow.right -= UserGetSystemMetrics(SM_CYCAPTION) - 1;
                    if pt.x > rcWindow.right {
                        return HTSYSMENU as DWORD;
                    }
                }

                /* Check close button */
                if Style & WS_SYSMENU != 0 {
                    rcWindow.left += UserGetSystemMetrics(SM_CYCAPTION);
                    if pt.x < rcWindow.left {
                        return HTCLOSE as DWORD;
                    }
                }

                /* Check maximize box */
                if min_or_max_box != 0 && (ExStyle & WS_EX_TOOLWINDOW == 0) {
                    rcWindow.left += UserGetSystemMetrics(SM_CXSIZE);
                    if pt.x < rcWindow.left {
                        return HTMAXBUTTON as DWORD;
                    }
                }

                /* Check minimize box */
                if min_or_max_box != 0 && (ExStyle & WS_EX_TOOLWINDOW == 0) {
                    rcWindow.left += UserGetSystemMetrics(SM_CXSIZE);
                    if pt.x < rcWindow.left {
                        return HTMINBUTTON as DWORD;
                    }
                }
            } else {
                /* Check system menu */
                if (Style & WS_SYSMENU != 0)
                    && (ExStyle & WS_EX_TOOLWINDOW == 0)
                    && !NC_IconForWindow(pWnd).is_null()
                {
                    rcWindow.left += UserGetSystemMetrics(SM_CYCAPTION) - 1;
                    if pt.x < rcWindow.left {
                        return HTSYSMENU as DWORD;
                    }
                }

                /* Check close button */
                if Style & WS_SYSMENU != 0 {
                    rcWindow.right -= UserGetSystemMetrics(SM_CYCAPTION);
                    if pt.x > rcWindow.right {
                        return HTCLOSE as DWORD;
                    }
                }

                /* Check maximize box */
                if min_or_max_box != 0 && (ExStyle & WS_EX_TOOLWINDOW == 0) {
                    rcWindow.right -= UserGetSystemMetrics(SM_CXSIZE);
                    if pt.x > rcWindow.right {
                        return HTMAXBUTTON as DWORD;
                    }
                }

                /* Check minimize box */
                if min_or_max_box != 0 && (ExStyle & WS_EX_TOOLWINDOW == 0) {
                    rcWindow.right -= UserGetSystemMetrics(SM_CXSIZE);
                    if pt.x > rcWindow.right {
                        return HTMINBUTTON as DWORD;
                    }
                }
            }
            return HTCAPTION as DWORD;
        }
    }

    /* Check menu bar */
    if HAS_MENU(pWnd, Style) != 0
        && (pt.y < rcClient.top)
        && (pt.x >= rcClient.left)
        && (pt.x < rcClient.right)
    {
        return HTMENU as DWORD;
    }

    /* Check vertical scroll bar */
    if ExStyle & WS_EX_LAYOUTRTL != 0 {
        ExStyle ^= WS_EX_LEFTSCROLLBAR;
    }
    if Style & WS_VSCROLL != 0 {
        if (ExStyle & WS_EX_LEFTSCROLLBAR) != 0 {
            rcClient.left -= UserGetSystemMetrics(SM_CXVSCROLL);
        } else {
            rcClient.right += UserGetSystemMetrics(SM_CXVSCROLL);
        }
        if RECTL_bPointInRect(core::ptr::addr_of_mut!(rcClient), pt.x, pt.y) != 0 {
            return HTVSCROLL as DWORD;
        }
    }

    /* Check horizontal scroll bar */
    if Style & WS_HSCROLL != 0 {
        rcClient.bottom += UserGetSystemMetrics(SM_CYHSCROLL);
        if RECTL_bPointInRect(core::ptr::addr_of_mut!(rcClient), pt.x, pt.y) != 0 {
            /* Check size box */
            if (Style & WS_VSCROLL != 0)
                && ((((ExStyle & WS_EX_LEFTSCROLLBAR) != 0)
                    && (pt.x <= rcClient.left + UserGetSystemMetrics(SM_CXVSCROLL)))
                    || (((ExStyle & WS_EX_LEFTSCROLLBAR) == 0)
                        && (pt.x >= rcClient.right - UserGetSystemMetrics(SM_CXVSCROLL))))
            {
                return HTSIZE as DWORD;
            }
            return HTHSCROLL as DWORD;
        }
    }

    HTNOWHERE as DWORD
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS Win32k subsystem
//  * PURPOSE:          Miscellaneous User functions
//  * FILE:             win32ss/user/ntuser/nonclient.c
//  * PROGRAMER:
//  */
//
// #include <win32k.h>
// #include <windowsx.h>
//
// DBG_DEFAULT_CHANNEL(UserDefwnd);
//
// #define UserHasDlgFrameStyle(Style, ExStyle)                                   \
//  (((ExStyle) & WS_EX_DLGMODALFRAME) ||                                         \
//   (((Style) & WS_DLGFRAME) && (!((Style) & WS_THICKFRAME))))
//
// #define UserHasThickFrameStyle(Style, ExStyle)                                 \
//   (((Style) & WS_THICKFRAME) &&                                                \
//    (!(((Style) & (WS_DLGFRAME | WS_BORDER)) == WS_DLGFRAME)))
//
// #define UserHasThinFrameStyle(Style, ExStyle)                                  \
//   (((Style) & WS_BORDER) || (!((Style) & (WS_CHILD | WS_POPUP))))
//
// #define ON_LEFT_BORDER(hit) \
//  (((hit) == HTLEFT) || ((hit) == HTTOPLEFT) || ((hit) == HTBOTTOMLEFT))
// #define ON_RIGHT_BORDER(hit) \
//  (((hit) == HTRIGHT) || ((hit) == HTTOPRIGHT) || ((hit) == HTBOTTOMRIGHT))
// #define ON_TOP_BORDER(hit) \
//  (((hit) == HTTOP) || ((hit) == HTTOPLEFT) || ((hit) == HTTOPRIGHT))
// #define ON_BOTTOM_BORDER(hit) \
//  (((hit) == HTBOTTOM) || ((hit) == HTBOTTOMLEFT) || ((hit) == HTBOTTOMRIGHT))
//
// #define HASSIZEGRIP(Style, ExStyle, ParentStyle, WindowRect, ParentClientRect) \
//             ((!(Style & WS_CHILD) && (Style & WS_THICKFRAME) && !(Style & WS_MAXIMIZE))  || \
//              ((Style & WS_CHILD) && (ParentStyle & WS_THICKFRAME) && !(ParentStyle & WS_MAXIMIZE) && \
//              (WindowRect.right - WindowRect.left == ParentClientRect.right) && \
//              (WindowRect.bottom - WindowRect.top == ParentClientRect.bottom)))
//
//
// VOID FASTCALL
// UserDrawWindowFrame(HDC hdc,
//                     RECTL *rect,
// 		    ULONG width,
// 		    ULONG height)
// {
//    HBRUSH hbrush = NtGdiSelectBrush( hdc, gpsi->hbrGray );
//    NtGdiPatBlt( hdc, rect->left, rect->top, rect->right - rect->left - width, height, PATINVERT );
//    NtGdiPatBlt( hdc, rect->left, rect->top + height, width, rect->bottom - rect->top - height, PATINVERT );
//    NtGdiPatBlt( hdc, rect->left + width, rect->bottom, rect->right - rect->left - width, -(LONG)height, PATINVERT );
//    NtGdiPatBlt( hdc, rect->right, rect->top, -(LONG)width, rect->bottom - rect->top - height, PATINVERT );
//    NtGdiSelectBrush( hdc, hbrush );
// }
//
// VOID FASTCALL
// UserDrawMovingFrame(HDC hdc,
//                     RECTL *rect,
//                     BOOL thickframe)
// {
//    if (thickframe) UserDrawWindowFrame(hdc, rect, UserGetSystemMetrics(SM_CXFRAME), UserGetSystemMetrics(SM_CYFRAME));
//    else UserDrawWindowFrame(hdc, rect, 1, 1);
// }
//
// /***********************************************************************
//  *           NC_GetInsideRect
//  *
//  * Get the 'inside' rectangle of a window, i.e. the whole window rectangle
//  * but without the borders (if any).
//  */
// void FASTCALL // Previously known as "UserGetInsideRectNC"
// NC_GetInsideRect(PWND Wnd, RECT *rect)
// {
//     ULONG Style;
//     ULONG ExStyle;
//
//     Style = Wnd->style;
//     ExStyle = Wnd->ExStyle;
//
//     rect->top    = rect->left = 0;
//     rect->right  = Wnd->rcWindow.right - Wnd->rcWindow.left;
//     rect->bottom = Wnd->rcWindow.bottom - Wnd->rcWindow.top;
//
//     if (Style & WS_ICONIC) return;
//
//     /* Remove frame from rectangle */
//     if (UserHasThickFrameStyle(Style, ExStyle))
//     {
//         RECTL_vInflateRect_g(rect, -UserGetSystemMetrics(SM_CXFRAME), -UserGetSystemMetrics(SM_CYFRAME));
//     }
//     else if (UserHasDlgFrameStyle(Style, ExStyle))
//     {
//         RECTL_vInflateRect_g(rect, -UserGetSystemMetrics(SM_CXDLGFRAME), -UserGetSystemMetrics(SM_CYDLGFRAME));
//         /* FIXME: this isn't in NC_AdjustRect? why not? */
//         if (ExStyle & WS_EX_DLGMODALFRAME)
//             RECTL_vInflateRect_g( rect, -1, 0 );
//     }
//     else if (UserHasThinFrameStyle(Style, ExStyle))
//     {
//         RECTL_vInflateRect_g(rect, -UserGetSystemMetrics(SM_CXBORDER), -UserGetSystemMetrics(SM_CYBORDER));
//     }
//
//     /* We have additional border information if the window
//      * is a child (but not an MDI child) */
//     if ((Style & WS_CHILD) && !(ExStyle & WS_EX_MDICHILD))
//     {
//        if (ExStyle & WS_EX_CLIENTEDGE)
//           RECTL_vInflateRect (rect, -UserGetSystemMetrics(SM_CXEDGE), -UserGetSystemMetrics(SM_CYEDGE));
//        if (ExStyle & WS_EX_STATICEDGE)
//           RECTL_vInflateRect (rect, -UserGetSystemMetrics(SM_CXBORDER), -UserGetSystemMetrics(SM_CYBORDER));
//     }
// }
//
// /***********************************************************************
//  *           NC_GetSysPopupPos
//  */
// void FASTCALL
// NC_GetSysPopupPos(PWND Wnd, RECT *Rect)
// {
//   RECT WindowRect;
//
//   if ((Wnd->style & WS_MINIMIZE) != 0)
//   {
//       IntGetWindowRect(Wnd, Rect);
//   }
//   else
//     {
//       NC_GetInsideRect(Wnd, Rect);
//       IntGetWindowRect(Wnd, &WindowRect);
//       RECTL_vOffsetRect(Rect, WindowRect.left, WindowRect.top);
//       if (Wnd->style & WS_CHILD)
//       {
//           IntClientToScreen(IntGetParent(Wnd), (POINT *) Rect);
//       }
//       Rect->right = Rect->left + UserGetSystemMetrics(SM_CYCAPTION) - 1;
//       Rect->bottom = Rect->top + UserGetSystemMetrics(SM_CYCAPTION) - 1;
//     }
// }
//
// static UINT
// GetSnapActivationPoint(PWND Wnd, POINT pt)
// {
//     // TODO: SPI_GETMOUSEDOCKTHRESHOLD
//     RECT wa;
//     if (!GetSnapSetting(bDockMoving))
//         return HTNOWHERE;
//     UserSystemParametersInfo(SPI_GETWORKAREA, 0, &wa, 0); /* FIXME: MultiMon of PWND */
//
//     if (pt.x <= wa.left) return HTLEFT;
//     if (pt.x >= wa.right-1) return HTRIGHT;
//     if (pt.y <= wa.top) return HTTOP; /* Maximize */
//     return HTNOWHERE;
// }
//
// LONG FASTCALL
// DefWndStartSizeMove(PWND Wnd, WPARAM wParam, POINT *capturePoint)
// {
//    LONG hittest = 0;
//    POINT pt;
//    MSG msg;
//    RECT rectWindow;
//    ULONG Style = Wnd->style;
//    PTHREADINFO pti = PsGetCurrentThreadWin32Thread();
//
//    rectWindow = Wnd->rcWindow;
//
//    if ((wParam & 0xfff0) == SC_MOVE)
//    {
//        /* Move pointer at the center of the caption */
//        RECT rect = rectWindow;
//        /* Note: to be exactly centered we should take the different types
//         * of border into account, but it shouldn't make more than a few pixels
//         * of difference so let's not bother with that */
//        if (Style & WS_SYSMENU)
//           rect.left += UserGetSystemMetrics(SM_CXSIZE) + 1;
//        if (Style & WS_MINIMIZEBOX)
// 	  rect.right -= UserGetSystemMetrics(SM_CXSIZE) + 1;
//        if (Style & WS_MAXIMIZEBOX)
//           rect.right -= UserGetSystemMetrics(SM_CXSIZE) + 1;
//        pt.x = (rect.right + rect.left) / 2;
//        pt.y = rect.top + UserGetSystemMetrics(SM_CYSIZE)/2;
//        hittest = HTCAPTION;
//        *capturePoint = pt;
//    }
//    else  /* SC_SIZE */
//    {
//        pt.x = pt.y = 0;
//        while (!hittest)
//        {
//           if (!co_IntGetPeekMessage(&msg, 0, 0, 0, PM_REMOVE, TRUE)) return 0;
//           if (IntCallMsgFilter( &msg, MSGF_SIZE )) continue;
//
// 	  switch(msg.message)
// 	    {
// 	    case WM_MOUSEMOVE:
// 	      //// Clamp the mouse position to the window rectangle when starting a window resize.
//               pt.x = min( max( msg.pt.x, rectWindow.left ), rectWindow.right - 1 );
//               pt.y = min( max( msg.pt.y, rectWindow.top ), rectWindow.bottom - 1 );
// 	      hittest = GetNCHitEx(Wnd, pt);
// 	      if ((hittest < HTLEFT) || (hittest > HTBOTTOMRIGHT)) hittest = 0;
// 	      break;
//
// 	    case WM_LBUTTONUP:
// 	      return 0;
//
// 	    case WM_KEYDOWN:
// 	      switch (msg.wParam)
// 		{
// 		case VK_UP:
// 		  hittest = HTTOP;
// 		  pt.x = (rectWindow.left+rectWindow.right)/2;
// 		  pt.y =  rectWindow.top + UserGetSystemMetrics(SM_CYFRAME) / 2;
// 		  break;
// 		case VK_DOWN:
// 		  hittest = HTBOTTOM;
// 		  pt.x = (rectWindow.left+rectWindow.right)/2;
// 		  pt.y =  rectWindow.bottom - UserGetSystemMetrics(SM_CYFRAME) / 2;
// 		  break;
// 		case VK_LEFT:
// 		  hittest = HTLEFT;
// 		  pt.x =  rectWindow.left + UserGetSystemMetrics(SM_CXFRAME) / 2;
// 		  pt.y = (rectWindow.top+rectWindow.bottom)/2;
// 		  break;
// 		case VK_RIGHT:
// 		  hittest = HTRIGHT;
// 		  pt.x =  rectWindow.right - UserGetSystemMetrics(SM_CXFRAME) / 2;
// 		  pt.y = (rectWindow.top+rectWindow.bottom)/2;
// 		  break;
// 		case VK_RETURN:
// 		case VK_ESCAPE:
// 		  return 0;
// 		}
//               break;
//             default:
//               IntTranslateKbdMessage( &msg, 0 );
//               pti->TIF_flags |= TIF_MOVESIZETRACKING;
//               IntDispatchMessage( &msg );
//               pti->TIF_flags |= TIF_MOVESIZETRACKING;
//               break;
// 	    }
//        }
//        *capturePoint = pt;
//     }
//     UserSetCursorPos(pt.x, pt.y, 0, 0, FALSE);
//     co_IntSendMessage(UserHMGetHandle(Wnd), WM_SETCURSOR, (WPARAM)UserHMGetHandle(Wnd), MAKELONG(hittest, WM_MOUSEMOVE));
//     return hittest;
// }
//
// //
// //  System Command Size and Move
// //
// //  Perform SC_MOVE and SC_SIZE commands.
// //
// VOID FASTCALL
// DefWndDoSizeMove(PWND pwnd, WORD wParam)
// {
//    MSG msg;
//    RECT sizingRect, mouseRect, origRect, unmodRect, snapPreviewRect;
//    PRECT pFrameRect = &sizingRect;
//    HDC hdc;
//    LONG hittest = (LONG)(wParam & 0x0f);
//    PCURICON_OBJECT DragCursor = NULL, OldCursor = NULL;
//    POINT minTrack, maxTrack;
//    POINT capturePoint, pt;
//    ULONG Style, ExStyle;
//    UINT orgSnap = IntGetWindowSnapEdge(pwnd), snap = orgSnap;
//    BOOL thickframe;
//    BOOL iconic;
//    BOOL moved = FALSE;
//    BOOL DragFullWindows = FALSE;
//    PWND pWndParent = NULL;
//    WPARAM syscommand = (wParam & 0xfff0);
//    PTHREADINFO pti = PsGetCurrentThreadWin32Thread();
//    //PMONITOR mon = 0; Don't port sync from wine!!! This breaks explorer task bar sizing!!
//    //                  The task bar can grow in size and can not reduce due to the change
//    //                  in the work area.
//
//    Style = pwnd->style;
//    ExStyle = pwnd->ExStyle;
//    iconic = (Style & WS_MINIMIZE) != 0;
//
//    if (((Style & WS_MAXIMIZE) && syscommand != SC_MOVE) || !IntIsWindowVisible(pwnd)) return;
//    if ((Style & (WS_MAXIMIZE | WS_CHILD)) == WS_MAXIMIZE)
//        orgSnap = snap = HTTOP;
//
//    thickframe = UserHasThickFrameStyle(Style, ExStyle) && !iconic;
//
//    //
//    // Show window contents while dragging the window, get flag from registry data.
//    //
//    UserSystemParametersInfo(SPI_GETDRAGFULLWINDOWS, 0, &DragFullWindows, 0);
//
//    pt.x = pti->ptLast.x;
//    pt.y = pti->ptLast.y;
//    capturePoint = pt;
//    UserClipCursor( NULL );
//
//    TRACE("pwnd %p command %04lx, hittest %d, pos %d,%d\n",
//           pwnd, syscommand, hittest, pt.x, pt.y);
//
//    if (syscommand == SC_MOVE)
//    {
//       if (!hittest) hittest = DefWndStartSizeMove(pwnd, wParam, &capturePoint);
//       if (!hittest) return;
//    }
//    else  /* SC_SIZE */
//    {
//       if (!thickframe) return;
//       if (hittest && (syscommand != SC_MOUSEMENU))
//       {
//           hittest += (HTLEFT - WMSZ_LEFT);
//       }
//       else
//       {
//           co_UserSetCapture(UserHMGetHandle(pwnd));
//           hittest = DefWndStartSizeMove(pwnd, wParam, &capturePoint);
//           if (!hittest)
//           {
//               IntReleaseCapture();
//               return;
//           }
//       }
//    }
//
//    /* Get min/max info */
//
//    co_WinPosGetMinMaxInfo(pwnd, NULL, NULL, &minTrack, &maxTrack);
//    sizingRect = pwnd->rcWindow;
//    origRect = sizingRect;
//    if (Style & WS_CHILD)
//    {
//       pWndParent = IntGetParent(pwnd);
//       IntGetClientRect( pWndParent, &mouseRect );
//       IntMapWindowPoints( pWndParent, 0, (LPPOINT)&mouseRect, 2 );
//       IntMapWindowPoints( 0, pWndParent, (LPPOINT)&sizingRect, 2 );
//       unmodRect = sizingRect;
//    }
//    else
//    {
//       if (!(ExStyle & WS_EX_TOPMOST))
//       {
//         UserSystemParametersInfo(SPI_GETWORKAREA, 0, &mouseRect, 0);
//       }
//       else
//       {
//         RECTL_vSetRect(&mouseRect, 0, 0, UserGetSystemMetrics(SM_CXSCREEN), UserGetSystemMetrics(SM_CYSCREEN));
//       }
//       unmodRect = sizingRect;
//    }
//
//    if (ON_LEFT_BORDER(hittest))
//    {
//       mouseRect.left  = max( mouseRect.left, sizingRect.right-maxTrack.x+capturePoint.x-sizingRect.left );
//       mouseRect.right = min( mouseRect.right, sizingRect.right-minTrack.x+capturePoint.x-sizingRect.left );
//    }
//    else if (ON_RIGHT_BORDER(hittest))
//    {
//       mouseRect.left  = max( mouseRect.left, sizingRect.left+minTrack.x+capturePoint.x-sizingRect.right );
//       mouseRect.right = min( mouseRect.right, sizingRect.left+maxTrack.x+capturePoint.x-sizingRect.right );
//    }
//    if (ON_TOP_BORDER(hittest))
//    {
//       mouseRect.top    = max( mouseRect.top, sizingRect.bottom-maxTrack.y+capturePoint.y-sizingRect.top );
//       mouseRect.bottom = min( mouseRect.bottom,sizingRect.bottom-minTrack.y+capturePoint.y-sizingRect.top);
//    }
//    else if (ON_BOTTOM_BORDER(hittest))
//    {
//       mouseRect.top    = max( mouseRect.top, sizingRect.top+minTrack.y+capturePoint.y-sizingRect.bottom );
//       mouseRect.bottom = min( mouseRect.bottom, sizingRect.top+maxTrack.y+capturePoint.y-sizingRect.bottom );
//    }
//
//    hdc = UserGetDCEx( pWndParent, 0, DCX_CACHE );
//    if (iconic)
//    {
//        DragCursor = pwnd->pcls->spicn;
//        if (DragCursor)
//        {
//            UserReferenceObject(DragCursor);
//        }
//        else
//        {
//            HCURSOR CursorHandle = (HCURSOR)co_IntSendMessage( UserHMGetHandle(pwnd), WM_QUERYDRAGICON, 0, 0 );
//            if (CursorHandle)
//            {
//                DragCursor = UserGetCurIconObject(CursorHandle);
//            }
//            else
//            {
//                iconic = FALSE;
//            }
//        }
//    }
//
//    /* repaint the window before moving it around */
//    co_UserRedrawWindow( pwnd, NULL, 0, RDW_UPDATENOW | RDW_ALLCHILDREN);
//
//    IntNotifyWinEvent( EVENT_SYSTEM_MOVESIZESTART, pwnd, OBJID_WINDOW, CHILDID_SELF, 0);
//
//    co_IntSendMessage( UserHMGetHandle(pwnd), WM_ENTERSIZEMOVE, 0, 0 );
//
//    MsqSetStateWindow(pti, MSQ_STATE_MOVESIZE, UserHMGetHandle(pwnd));
//
//    if (IntGetCapture() != UserHMGetHandle(pwnd)) co_UserSetCapture( UserHMGetHandle(pwnd) );
//
//    pwnd->head.pti->TIF_flags |= TIF_MOVESIZETRACKING;
//
//    for(;;)
//    {
//       int dx = 0, dy = 0;
//
//       if (!co_IntGetPeekMessage(&msg, 0, 0, 0, PM_REMOVE, TRUE)) break;
//       if (IntCallMsgFilter( &msg, MSGF_SIZE )) continue;
//
//       if (msg.message == WM_KEYDOWN && (msg.wParam == VK_RETURN || msg.wParam == VK_ESCAPE))
//          break; // Exit on Return or Esc
//
//       if (!g_bWindowSnapEnabled && (msg.message == WM_LBUTTONUP ||
//          (msg.message == WM_MOUSEMOVE && (msg.wParam & MK_LBUTTON) == 0)))
//       { // If no WindowSnapEnabled: Exit on button-up immediately
//          break;
//       }
//       else if (g_bWindowSnapEnabled && (msg.message == WM_LBUTTONUP ||
//               (msg.message == WM_MOUSEMOVE && (msg.wParam & MK_LBUTTON) == 0)))
//       { // If WindowSnapEnabled: Decide whether to snap before exiting
//          if (hittest == HTCAPTION && thickframe && /* Check for snapping if was moved by caption */
//              IntIsSnapAllowedForWindow(pwnd) && (ExStyle & WS_EX_MDICHILD) == 0)
//          {
//             BOOLEAN wasSnap = IntIsWindowSnapped(pwnd); /* Need the live snap state, not orgSnap nor maximized state */
//             UINT snapTo = iconic ? HTNOWHERE : GetSnapActivationPoint(pwnd, pt);
//             if (snapTo)
//             {
//                 if (DragFullWindows)
//                 {
//                     co_IntSnapWindow(pwnd, snapTo);
//                     if (!wasSnap)
//                         pwnd->InternalPos.NormalRect = origRect;
//                 }
//                 snap = snapTo;
//             }
//          }
//          break;
//       }
//
//       if ((msg.message != WM_KEYDOWN) && (msg.message != WM_MOUSEMOVE))
//       {
//          IntTranslateKbdMessage( &msg , 0 );
//          IntDispatchMessage( &msg );
//          continue;  /* We are not interested in other messages */
//       }
//
//       pt = msg.pt;
//
//       if (msg.message == WM_KEYDOWN) switch(msg.wParam)
//       {
//       case VK_UP:    pt.y -= 8; break;
//       case VK_DOWN:  pt.y += 8; break;
//       case VK_LEFT:  pt.x -= 8; break;
//       case VK_RIGHT: pt.x += 8; break;
//       }
//
//       pt.x = max( pt.x, mouseRect.left );
//       pt.x = min( pt.x, mouseRect.right - 1 );
//       pt.y = max( pt.y, mouseRect.top );
//       pt.y = min( pt.y, mouseRect.bottom - 1 );
//
//       dx = pt.x - capturePoint.x;
//       dy = pt.y - capturePoint.y;
//
//       if (dx || dy)
//       {
//       if (!moved)
//       {
//           moved = TRUE;
//           if (iconic) /* ok, no system popup tracking */
//           {
//               OldCursor = UserSetCursor(DragCursor, FALSE);
//               UserShowCursor(TRUE);
//           }
//           else if (!DragFullWindows)
//              UserDrawMovingFrame(hdc, &sizingRect, thickframe);
//       }
//
//       if (msg.message == WM_KEYDOWN)
//       {
//           UserSetCursorPos(pt.x, pt.y, 0, 0, FALSE);
//       }
//       else
//       {
//           RECT newRect = unmodRect;
//
//           if (!iconic && !DragFullWindows)
//           {
//               UserDrawMovingFrame(hdc, pFrameRect, thickframe);
//               pFrameRect = &sizingRect;
//           }
//           if (hittest == HTCAPTION)
//           {
//               /* Restore window size if it is snapped */
//               PRECT pr = &newRect;
//               LONG width, height, capcy, snapTo;
//               if (snap && syscommand == SC_MOVE && !iconic &&
//                   !RECTL_bIsEmptyRect(&pwnd->InternalPos.NormalRect))
//               {
//                   *pr = pwnd->InternalPos.NormalRect;
//                   origRect = *pr; /* Save normal size - is required when window unsnapped from one side and snapped to another holding mouse down */
//
//                   /* Try to position the center of the caption where the mouse is horizontally */
//                   capcy = UserGetSystemMetrics((ExStyle & WS_EX_TOPMOST) ? SM_CYSMCAPTION : SM_CYCAPTION); /* No border, close enough */
//                   width = pr->right - pr->left;
//                   height = pr->bottom - pr->top;
//                   pr->left = pt.x - width / 2;
//                   pr->right = pr->left + width;
//                   pr->top = mouseRect.top;
//                   pr->bottom = pr->top + height;
//                   if (pr->left < mouseRect.left)
//                   {
//                       pr->left = mouseRect.left;
//                       pr->right = pr->left + width;
//                   }
//                   if ((pwnd->ExStyle & WS_EX_LAYOUTRTL) && pr->right > mouseRect.right)
//                   {
//                       pr->left = mouseRect.right - width;
//                       pr->right = pr->left + width;
//                   }
//                   UserSetCursorPos(pt.x, pr->top + capcy / 2, 0, 0, FALSE);
//                   snap = FALSE;
//                   dx = dy = 0; /* Don't offset this move */
//                   if (DragFullWindows)
//                   {
//                       IntSetStyle(pwnd, 0, WS_MAXIMIZE);
//                       IntSetSnapEdge(pwnd, HTNOWHERE);
//
//                       /* Have to move and size it now because we don't want SWP_NOSIZE */
//                       co_WinPosSetWindowPos(pwnd, HWND_TOP, pr->left, pr->top, width, height, SWP_NOACTIVATE);
//                   }
//               }
//               else if (!snap && syscommand == SC_MOVE && !iconic)
//               {
//                   if ((snapTo = GetSnapActivationPoint(pwnd, pt)) != 0)
//                   {
//                       co_IntCalculateSnapPosition(pwnd, snapTo, &snapPreviewRect);
//                       if (DragFullWindows)
//                       {
//                           /* TODO: Show preview of snap */
//                       }
//                       else
//                       {
//                           pFrameRect = &snapPreviewRect;
//                           UserDrawMovingFrame(hdc, pFrameRect, thickframe);
//                           continue;
//                       }
//                   }
//               }
//
//               /* regular window moving */
//               RECTL_vOffsetRect(&newRect, dx, dy);
//           }
//           if (ON_LEFT_BORDER(hittest)) newRect.left += dx;
//           else if (ON_RIGHT_BORDER(hittest)) newRect.right += dx;
//           if (ON_TOP_BORDER(hittest)) newRect.top += dy;
//           else if (ON_BOTTOM_BORDER(hittest)) newRect.bottom += dy;
//
//           capturePoint = pt;
//
//               //
//               //  Save the new position to the unmodified rectangle. This allows explorer task bar
//               //  sizing. Explorer will forces back the position unless a certain amount of sizing
//               //  has occurred.
//               //
//               unmodRect = newRect;
//
//               /* Determine the hit location */
//               if (syscommand == SC_SIZE)
//               {
//                   WPARAM wpSizingHit = 0;
//
//                   if (hittest >= HTLEFT && hittest <= HTBOTTOMRIGHT)
//                       wpSizingHit = WMSZ_LEFT + (hittest - HTLEFT);
//                   co_IntSendMessage( UserHMGetHandle(pwnd), WM_SIZING, wpSizingHit, (LPARAM)&newRect );
//               }
//               else
//                   co_IntSendMessage( UserHMGetHandle(pwnd), WM_MOVING, 0, (LPARAM)&newRect );
//
//               if (!iconic)
//               {
//                  if (!DragFullWindows)
//                      UserDrawMovingFrame( hdc, &newRect, thickframe );
//                  else
//                  {  // Moving the whole window now!
//                     HRGN hrgnNew;
//                     HRGN hrgnOrig = GreCreateRectRgnIndirect(&pwnd->rcWindow);
//
//                     if (pwnd->hrgnClip != NULL)
//                        NtGdiCombineRgn(hrgnOrig, hrgnOrig, pwnd->hrgnClip, RGN_AND);
//
//                     //// This causes the mdi child window to jump up when it is moved.
//                     //IntMapWindowPoints( 0, pWndParent, (POINT *)&rect, 2 );
//                     co_WinPosSetWindowPos(pwnd,
//                                           NULL,
//                                           newRect.left,
//                                           newRect.top,
//                                           newRect.right - newRect.left,
//                                           newRect.bottom - newRect.top,
//                                           SWP_NOACTIVATE | ((hittest == HTCAPTION) ? SWP_NOSIZE : 0));
//
//                     hrgnNew = GreCreateRectRgnIndirect(&pwnd->rcWindow);
//                     if (pwnd->hrgnClip != NULL)
//                        NtGdiCombineRgn(hrgnNew, hrgnNew, pwnd->hrgnClip, RGN_AND);
//
//                     if (hrgnNew)
//                     {
//                        if (hrgnOrig)
//                           NtGdiCombineRgn(hrgnOrig, hrgnOrig, hrgnNew, RGN_DIFF);
//                     }
//                     else
//                     {
//                        if (hrgnOrig)
//                        {
//                           GreDeleteObject(hrgnOrig);
//                           hrgnOrig = 0;
//                        }
//                     }
//
//                     // Update all the windows after the move or size, including this window.
//                     UpdateThreadWindows(UserGetDesktopWindow()->spwndChild, pti, hrgnOrig);
//
//                     if (hrgnOrig) GreDeleteObject(hrgnOrig);
//                     if (hrgnNew) GreDeleteObject(hrgnNew);
//                  }
//               }
//               sizingRect = newRect;
//         }
//       }
//    }
//
//    pwnd->head.pti->TIF_flags &= ~TIF_MOVESIZETRACKING;
//
//    IntReleaseCapture();
//
//    if ( iconic )
//    {
//       if ( moved ) /* restore cursors, show icon title later on */
//       {
//           UserShowCursor( FALSE );
//           OldCursor = UserSetCursor(OldCursor, FALSE);
//       }
//
//       /* It could be that the cursor was already changed while we were proceeding,
//        * so we must unreference whatever cursor was current at the time we restored the old one.
//        * Maybe it is DragCursor, but maybe it is another one and DragCursor got already freed.
//        */
//       if (OldCursor) UserDereferenceObject(OldCursor);
//    }
//    else
//    {
//       UINT eraseFinalFrame = moved && !DragFullWindows;
//       if (eraseFinalFrame)
//          UserDrawMovingFrame(hdc, pFrameRect, thickframe); // Undo the XOR drawing
//    }
//
//    UserReleaseDC(NULL, hdc, FALSE);
//
//    //// This causes the mdi child window to jump up when it is moved.
//    //if (pWndParent) IntMapWindowPoints( 0, pWndParent, (POINT *)&sizingRect, 2 );
//
//    if (co_HOOK_CallHooks(WH_CBT, HCBT_MOVESIZE, (WPARAM)UserHMGetHandle(pwnd), (LPARAM)&sizingRect))
//    {
//       ERR("DoSizeMove : WH_CBT Call Hook return!\n");
//       moved = FALSE;
//    }
//
//    IntNotifyWinEvent( EVENT_SYSTEM_MOVESIZEEND, pwnd, OBJID_WINDOW, CHILDID_SELF, 0);
//
//    MsqSetStateWindow(pti, MSQ_STATE_MOVESIZE, NULL);
//
//    co_IntSendMessage( UserHMGetHandle(pwnd), WM_EXITSIZEMOVE, 0, 0 );
//    //// wine mdi hack
//    co_IntSendMessage( UserHMGetHandle(pwnd), WM_SETVISIBLE, !!(pwnd->style & WS_MINIMIZE), 0L);
//    ////
//    /* window moved or resized */
//    if (moved)
//    {
//       BOOL forceSizing = !iconic && hittest == HTCAPTION && (!!orgSnap != !!snap);
//       UINT swp = (!forceSizing && hittest == HTCAPTION) ? SWP_NOSIZE : 0;
//
//       /* if the moving/resizing isn't canceled call SetWindowPos
//        * with the new position or the new size of the window
//        */
//       if (!((msg.message == WM_KEYDOWN) && (msg.wParam == VK_ESCAPE)) )
//       {
//          /* NOTE: SWP_NOACTIVATE prevents document window activation in Word 6 */
//          if (!DragFullWindows || iconic)
//          {
//             if (snap)
//             {
//                co_IntSnapWindow(pwnd, snap);
//             }
//             else
//             {
//                if (orgSnap && !snap)
//                {
//                   IntSetStyle(pwnd, 0, WS_MAXIMIZE);
//                   IntSetSnapInfo(pwnd, HTNOWHERE, NULL);
//                }
//                co_WinPosSetWindowPos(pwnd, HWND_TOP, sizingRect.left, sizingRect.top,
//                                      sizingRect.right - sizingRect.left,
//                                      sizingRect.bottom - sizingRect.top, swp);
//             }
//          }
//       }
//       else
//       {
//          /* restore previous size/position */
//          if (orgSnap)
//          {
//             co_IntSnapWindow(pwnd, orgSnap);
//          }
//          else if (DragFullWindows)
//          {
//             co_WinPosSetWindowPos(pwnd, HWND_TOP, origRect.left, origRect.top,
//                                   origRect.right - origRect.left,
//                                   origRect.bottom - origRect.top, swp);
//          }
//       }
//    }
//
//    if (IntIsWindow(UserHMGetHandle(pwnd)))
//    {
//       /* Single click brings up the system menu when iconized */
//       if (iconic && !moved && (Style & WS_SYSMENU))
//       {
//          co_IntSendMessage(UserHMGetHandle(pwnd), WM_SYSCOMMAND, SC_MOUSEMENU + HTSYSMENU, MAKELONG(pt.x, pt.y));
//       }
//    }
// }
//
// PCURICON_OBJECT FASTCALL NC_IconForWindow( PWND pWnd )
// {
//     PCURICON_OBJECT pIcon = NULL;
//     HICON hIcon;
//
//    hIcon = UserGetProp(pWnd, gpsi->atomIconSmProp, TRUE);
//    if (!hIcon) hIcon = UserGetProp(pWnd, gpsi->atomIconProp, TRUE);
//
//    if (!hIcon && pWnd->pcls->spicnSm)
//        return pWnd->pcls->spicnSm;
//    if (!hIcon && pWnd->pcls->spicn)
//        return pWnd->pcls->spicn;
//
//    // WARNING: Wine code has this test completely wrong. The following is how
//    // Windows behaves for windows having the WS_EX_DLGMODALFRAME style set:
//    // it does not use the default icon! And it does not check for DS_MODALFRAME.
//    if (!hIcon && !(pWnd->ExStyle & WS_EX_DLGMODALFRAME))
//    {
//       hIcon = gpsi->hIconSmWindows; // Both are IDI_WINLOGO Small
//       if (!hIcon) hIcon = gpsi->hIconWindows;   // Reg size.
//    }
//    if (hIcon)
//    {
//        pIcon = (PCURICON_OBJECT)UserGetObjectNoErr(gHandleTable,
//                                                    hIcon,
//                                                    TYPE_CURSOR);
//    }
//    return pIcon;
// }
//
// BOOL
// UserDrawSysMenuButton(PWND pWnd, HDC hDC, LPRECT Rect, BOOL Down)
// {
//    PCURICON_OBJECT WindowIcon;
//    BOOL Ret = FALSE;
//
//    if ((WindowIcon = NC_IconForWindow(pWnd)))
//    {
//       UserReferenceObject(WindowIcon);
//
//       Ret = UserDrawIconEx(hDC,
//                            Rect->left + 2,
//                            Rect->top + 2,
//                            WindowIcon,
//                            UserGetSystemMetrics(SM_CXSMICON),
//                            UserGetSystemMetrics(SM_CYSMICON),
//                            0, NULL, DI_NORMAL);
//
//       UserDereferenceObject(WindowIcon);
//    }
//    return Ret;
// }
//
// BOOL
// IntIsScrollBarVisible(PWND pWnd, INT hBar)
// {
//   SCROLLBARINFO sbi;
//   sbi.cbSize = sizeof(SCROLLBARINFO);
//
//   if(!co_IntGetScrollBarInfo(pWnd, hBar, &sbi))
//     return FALSE;
//
//   return !(sbi.rgstate[0] & STATE_SYSTEM_OFFSCREEN);
// }
//
// /*
//  * FIXME:
//  * - Cache bitmaps, then just bitblt instead of calling DFC() (and
//  *   wasting precious CPU cycles) every time
//  * - Center the buttons vertically in the rect
//  */
// VOID
// UserDrawCaptionButton(PWND pWnd, LPRECT Rect, DWORD Style, DWORD ExStyle, HDC hDC, BOOL bDown, ULONG Type)
// {
//    RECT TempRect;
//
//    if (!(Style & WS_SYSMENU))
//    {
//       return;
//    }
//
//    TempRect = *Rect;
//
//    switch (Type)
//    {
//       case DFCS_CAPTIONMIN:
//       {
//          if (ExStyle & WS_EX_TOOLWINDOW)
//             return; /* ToolWindows don't have min/max buttons */
//
//          if (Style & WS_SYSMENU)
//              TempRect.right -= UserGetSystemMetrics(SM_CXSIZE) + 1;
//
//          if (Style & (WS_MAXIMIZEBOX | WS_MINIMIZEBOX))
//              TempRect.right -= UserGetSystemMetrics(SM_CXSIZE) - 2;
//
//          TempRect.left = TempRect.right - UserGetSystemMetrics(SM_CXSIZE) + 1;
//          TempRect.bottom = TempRect.top + UserGetSystemMetrics(SM_CYSIZE) - 2;
//          TempRect.top += 2;
//          TempRect.right -= 1;
//
//          DrawFrameControl(hDC, &TempRect, DFC_CAPTION,
//                           ((Style & WS_MINIMIZE) ? DFCS_CAPTIONRESTORE : DFCS_CAPTIONMIN) |
//                           (bDown ? DFCS_PUSHED : 0) |
//                           ((Style & WS_MINIMIZEBOX) ? 0 : DFCS_INACTIVE));
//          break;
//       }
//       case DFCS_CAPTIONMAX:
//       {
//          if (ExStyle & WS_EX_TOOLWINDOW)
//              return; /* ToolWindows don't have min/max buttons */
//
//          if (Style & WS_SYSMENU)
//              TempRect.right -= UserGetSystemMetrics(SM_CXSIZE) + 1;
//
//          TempRect.left = TempRect.right - UserGetSystemMetrics(SM_CXSIZE) + 1;
//          TempRect.bottom = TempRect.top + UserGetSystemMetrics(SM_CYSIZE) - 2;
//          TempRect.top += 2;
//          TempRect.right -= 1;
//
//          DrawFrameControl(hDC, &TempRect, DFC_CAPTION,
//                           ((Style & WS_MAXIMIZE) ? DFCS_CAPTIONRESTORE : DFCS_CAPTIONMAX) |
//                           (bDown ? DFCS_PUSHED : 0) |
//                           ((Style & WS_MAXIMIZEBOX) ? 0 : DFCS_INACTIVE));
//          break;
//       }
//       case DFCS_CAPTIONCLOSE:
//       {
//           PMENU pSysMenu = IntGetSystemMenu(pWnd, FALSE);
//           UINT MenuState = IntGetMenuState(pSysMenu ? UserHMGetHandle(pSysMenu) : NULL, SC_CLOSE, MF_BYCOMMAND); /* in case of error MenuState==0xFFFFFFFF */
//
//          /* A tool window has a smaller Close button */
//          if (ExStyle & WS_EX_TOOLWINDOW)
//          {
//             TempRect.left = TempRect.right - UserGetSystemMetrics(SM_CXSMSIZE);
//             TempRect.bottom = TempRect.top + UserGetSystemMetrics(SM_CYSMSIZE) - 2;
//          }
//          else
//          {
//             TempRect.left = TempRect.right - UserGetSystemMetrics(SM_CXSIZE);
//             TempRect.bottom = TempRect.top + UserGetSystemMetrics(SM_CYSIZE) - 2;
//          }
//          TempRect.top += 2;
//          TempRect.right -= 2;
//
//          DrawFrameControl(hDC, &TempRect, DFC_CAPTION,
//                           (DFCS_CAPTIONCLOSE | (bDown ? DFCS_PUSHED : 0) |
//                           ((!(MenuState & (MF_GRAYED|MF_DISABLED)) && !(pWnd->pcls->style & CS_NOCLOSE)) ? 0 : DFCS_INACTIVE)));
//          break;
//       }
//    }
// }
//
// VOID
// UserDrawCaptionButtonWnd(PWND pWnd, HDC hDC, BOOL bDown, ULONG Type)
// {
//    RECT WindowRect;
//    SIZE WindowBorder;
//
//    IntGetWindowRect(pWnd, &WindowRect);
//
//    WindowRect.right -= WindowRect.left;
//    WindowRect.bottom -= WindowRect.top;
//    WindowRect.left = WindowRect.top = 0;
//
//    UserGetWindowBorders(pWnd->style, pWnd->ExStyle, &WindowBorder, FALSE);
//
//    RECTL_vInflateRect_g(&WindowRect, -WindowBorder.cx, -WindowBorder.cy);
//
//    UserDrawCaptionButton(pWnd, &WindowRect, pWnd->style, pWnd->ExStyle, hDC, bDown, Type);
// }
//
// VOID
// NC_DrawFrame( HDC hDC, RECT *CurrentRect, BOOL Active, DWORD Style, DWORD ExStyle)
// {
//    /* Firstly the "thick" frame */
//    if ((Style & WS_THICKFRAME) && !(Style & WS_MINIMIZE))
//    {
//       LONG Width =
//          (UserGetSystemMetrics(SM_CXFRAME) - UserGetSystemMetrics(SM_CXDLGFRAME)) *
//          UserGetSystemMetrics(SM_CXBORDER);
//
//       LONG Height =
//          (UserGetSystemMetrics(SM_CYFRAME) - UserGetSystemMetrics(SM_CYDLGFRAME)) *
//          UserGetSystemMetrics(SM_CYBORDER);
//
//       NtGdiSelectBrush(hDC, IntGetSysColorBrush(Active ? COLOR_ACTIVEBORDER : COLOR_INACTIVEBORDER));
//
//       /* Draw frame */
//       NtGdiPatBlt(hDC, CurrentRect->left, CurrentRect->top, CurrentRect->right - CurrentRect->left, Height, PATCOPY);
//       NtGdiPatBlt(hDC, CurrentRect->left, CurrentRect->top, Width, CurrentRect->bottom - CurrentRect->top, PATCOPY);
//       NtGdiPatBlt(hDC, CurrentRect->left, CurrentRect->bottom, CurrentRect->right - CurrentRect->left, -Height, PATCOPY);
//       NtGdiPatBlt(hDC, CurrentRect->right, CurrentRect->top, -Width, CurrentRect->bottom - CurrentRect->top, PATCOPY);
//
//       RECTL_vInflateRect_g(CurrentRect, -Width, -Height);
//    }
//
//    /* Now the other bit of the frame */
//    if (Style & (WS_DLGFRAME | WS_BORDER) || ExStyle & WS_EX_DLGMODALFRAME)
//    {
//       LONG Width = UserGetSystemMetrics(SM_CXBORDER);
//       LONG Height = UserGetSystemMetrics(SM_CYBORDER);
//
//       NtGdiSelectBrush(hDC, IntGetSysColorBrush(
//          (ExStyle & (WS_EX_DLGMODALFRAME | WS_EX_CLIENTEDGE)) ? COLOR_3DFACE :
//          (ExStyle & WS_EX_STATICEDGE) ? COLOR_WINDOWFRAME :
//          (Style & (WS_DLGFRAME | WS_THICKFRAME)) ? COLOR_3DFACE :
//          COLOR_WINDOWFRAME));
//
//       /* Draw frame */
//       NtGdiPatBlt(hDC, CurrentRect->left, CurrentRect->top, CurrentRect->right - CurrentRect->left, Height, PATCOPY);
//       NtGdiPatBlt(hDC, CurrentRect->left, CurrentRect->top, Width, CurrentRect->bottom - CurrentRect->top, PATCOPY);
//       NtGdiPatBlt(hDC, CurrentRect->left, CurrentRect->bottom, CurrentRect->right - CurrentRect->left, -Height, PATCOPY);
//       NtGdiPatBlt(hDC, CurrentRect->right, CurrentRect->top, -Width, CurrentRect->bottom - CurrentRect->top, PATCOPY);
//
//       RECTL_vInflateRect_g(CurrentRect, -Width, -Height);
//    }
// }
// /* --- pretranslation split: the remaining function groups live in nonclient.split2..2.c and are #included below so this stays ONE translation unit (zero CMake/linkage change). Reassembled verbatim by cpp. --- */
// /* nonclient.split2.c — #include fragment of nonclient.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original nonclient.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */
//
//
// VOID UserDrawCaptionBar(
//    PWND pWnd,
//    HDC hDC,
//    INT Flags)
// {
//    DWORD Style, ExStyle;
//    RECT WindowRect, CurrentRect, TempRect;
//    HPEN PreviousPen;
//    BOOL Gradient = FALSE;
//    PCURICON_OBJECT pIcon = NULL;
//
//    if (!(Flags & DC_NOVISIBLE) && !IntIsWindowVisible(pWnd)) return;
//
//    TRACE("UserDrawCaptionBar: pWnd %p, hDc %p, Flags 0x%x.\n", pWnd, hDC, Flags);
//
//    Style = pWnd->style;
//    ExStyle = pWnd->ExStyle;
//
//    IntGetWindowRect(pWnd, &WindowRect);
//
//    CurrentRect.top = CurrentRect.left = 0;
//    CurrentRect.right = WindowRect.right - WindowRect.left;
//    CurrentRect.bottom = WindowRect.bottom - WindowRect.top;
//
//    /* Draw outer edge */
//    if (UserHasWindowEdge(Style, ExStyle))
//    {
//       DrawEdge(hDC, &CurrentRect, EDGE_RAISED, BF_RECT | BF_ADJUST);
//    }
//    else if (ExStyle & WS_EX_STATICEDGE)
//    {
//       NtGdiSelectBrush(hDC, IntGetSysColorBrush(COLOR_BTNSHADOW));
//       NtGdiPatBlt(hDC, CurrentRect.left, CurrentRect.top, CurrentRect.right - CurrentRect.left, 1, PATCOPY);
//       NtGdiPatBlt(hDC, CurrentRect.left, CurrentRect.top, 1, CurrentRect.bottom - CurrentRect.top, PATCOPY);
//
//       NtGdiSelectBrush(hDC, IntGetSysColorBrush(COLOR_BTNHIGHLIGHT));
//       NtGdiPatBlt(hDC, CurrentRect.left, CurrentRect.bottom - 1, CurrentRect.right - CurrentRect.left, 1, PATCOPY);
//       NtGdiPatBlt(hDC, CurrentRect.right - 1, CurrentRect.top, 1, CurrentRect.bottom - CurrentRect.top, PATCOPY);
//
//       RECTL_vInflateRect_g(&CurrentRect, -1, -1);
//    }
//
//    if (Flags & DC_FRAME) NC_DrawFrame(hDC, &CurrentRect, (Flags & DC_ACTIVE), Style, ExStyle);
//
//    /* Draw caption */
//    if ((Style & WS_CAPTION) == WS_CAPTION)
//    {
//       TempRect = CurrentRect;
//
//       Flags |= DC_TEXT|DC_BUTTONS; // Icon will be checked if not already set.
//
//       if (UserSystemParametersInfo(SPI_GETGRADIENTCAPTIONS, 0, &Gradient, 0) && Gradient)
//       {
//          Flags |= DC_GRADIENT;
//       }
//
//       if (ExStyle & WS_EX_TOOLWINDOW)
//       {
//          Flags |= DC_SMALLCAP;
//          TempRect.bottom = TempRect.top + UserGetSystemMetrics(SM_CYSMCAPTION) - 1;
//          CurrentRect.top += UserGetSystemMetrics(SM_CYSMCAPTION);
//       }
//       else
//       {
//          TempRect.bottom = TempRect.top + UserGetSystemMetrics(SM_CYCAPTION) - 1;
//          CurrentRect.top += UserGetSystemMetrics(SM_CYCAPTION);
//       }
//
//       if (!(Flags & DC_ICON)               &&
//           !(Flags & DC_SMALLCAP)           &&
//            (Style & WS_SYSMENU)            &&
//           !(ExStyle & WS_EX_TOOLWINDOW) )
//       {
//          pIcon = NC_IconForWindow(pWnd); // Force redraw of caption with icon if DC_ICON not flaged....
//       }
//       UserDrawCaption(pWnd, hDC, &TempRect, NULL, pIcon ? UserHMGetHandle(pIcon) : NULL, NULL, Flags);
//
//       /* Draw buttons */
//       if (Style & WS_SYSMENU)
//       {
//          UserDrawCaptionButton(pWnd, &TempRect, Style, ExStyle, hDC, FALSE, DFCS_CAPTIONCLOSE);
//          if ((Style & (WS_MAXIMIZEBOX | WS_MINIMIZEBOX)) && !(ExStyle & WS_EX_TOOLWINDOW))
//          {
//             UserDrawCaptionButton(pWnd, &TempRect, Style, ExStyle, hDC, FALSE, DFCS_CAPTIONMIN);
//             UserDrawCaptionButton(pWnd, &TempRect, Style, ExStyle, hDC, FALSE, DFCS_CAPTIONMAX);
//          }
//       }
//
//       if (!(Style & WS_MINIMIZE))
//       {
//          /* Line under caption */
//          PreviousPen = NtGdiSelectPen(hDC, NtGdiGetStockObject(DC_PEN));
//
//          IntSetDCPenColor( hDC, IntGetSysColor(((ExStyle & (WS_EX_STATICEDGE|WS_EX_CLIENTEDGE|WS_EX_DLGMODALFRAME)) == WS_EX_STATICEDGE) ?
//                                              COLOR_WINDOWFRAME : COLOR_3DFACE));
//
//          GreMoveTo(hDC, TempRect.left, TempRect.bottom, NULL);
//
//          NtGdiLineTo(hDC, TempRect.right, TempRect.bottom);
//
//          NtGdiSelectPen(hDC, PreviousPen);
//       }
//    }
//
//    if (!(Style & WS_MINIMIZE))
//    {
//       /* Draw menu bar */
//       if (pWnd->state & WNDS_HASMENU && pWnd->IDMenu) // Should be pWnd->spmenu
//       {
//           PMENU menu;
//           if ((menu = UserGetMenuObject(UlongToHandle(pWnd->IDMenu)))) // FIXME! Use pWnd->spmenu,
//           {
//              TempRect = CurrentRect;
//              TempRect.bottom = TempRect.top + menu->cyMenu; // Should be pWnd->spmenu->cyMenu;
//              CurrentRect.top += MENU_DrawMenuBar(hDC, &TempRect, pWnd, FALSE);
//           }
//       }
//
//       if (ExStyle & WS_EX_CLIENTEDGE)
//       {
//           DrawEdge(hDC, &CurrentRect, EDGE_SUNKEN, BF_RECT | BF_ADJUST);
//       }
//    }
// }
//
// // Note from Wine:
// /* MSDN docs are pretty idiotic here, they say app CAN use clipRgn in
//    the call to GetDCEx implying that it is allowed not to use it either.
//    However, the suggested GetDCEx(    , DCX_WINDOW | DCX_INTERSECTRGN)
//    will cause clipRgn to be deleted after ReleaseDC().
//    Now, how is the "system" supposed to tell what happened?
//  */
// /*
//  * FIXME:
//  * - Drawing of WS_BORDER after scrollbars
//  * - Correct drawing of size-box
//  */
// LRESULT
// NC_DoNCPaint(PWND pWnd, HDC hDC, INT Flags)
// {
//    DWORD Style, ExStyle;
//    PWND Parent;
//    RECT WindowRect, CurrentRect, TempRect;
//    BOOL Active = FALSE;
//
//    if (!IntIsWindowVisible(pWnd) ||
//        (pWnd->state & WNDS_NONCPAINT && !(pWnd->state & WNDS_FORCEMENUDRAW)) ||
//         IntEqualRect(&pWnd->rcWindow, &pWnd->rcClient) )
//       return 0;
//
//    Style = pWnd->style;
//
//    TRACE("DefWndNCPaint: pWnd %p, hDc %p, Active %s.\n", pWnd, hDC, Flags & DC_ACTIVE ? "TRUE" : "FALSE");
//
//    Parent = IntGetParent(pWnd);
//    ExStyle = pWnd->ExStyle;
//
//    if (Flags == -1) // NC paint mode.
//    {
//       if (ExStyle & WS_EX_MDICHILD)
//       {
//          Active = IntIsChildWindow(gpqForeground->spwndActive, pWnd);
//
//          if (Active)
//             Active = (UserHMGetHandle(pWnd) == (HWND)co_IntSendMessage(UserHMGetHandle(Parent), WM_MDIGETACTIVE, 0, 0));
//       }
//       else
//       {
//          Active = (gpqForeground == pWnd->head.pti->MessageQueue);
//       }
//       Flags = DC_NC; // Redraw everything!
//    }
//    else
//       Flags |= DC_NC;
//
//
//    IntGetWindowRect(pWnd, &WindowRect);
//
//    CurrentRect.top = CurrentRect.left = 0;
//    CurrentRect.right = WindowRect.right - WindowRect.left;
//    CurrentRect.bottom = WindowRect.bottom - WindowRect.top;
//
//    /* Draw outer edge */
//    if (UserHasWindowEdge(pWnd->style, pWnd->ExStyle))
//    {
//       DrawEdge(hDC, &CurrentRect, EDGE_RAISED, BF_RECT | BF_ADJUST);
//    }
//    else if (pWnd->ExStyle & WS_EX_STATICEDGE)
//    {
//       NtGdiSelectBrush(hDC, IntGetSysColorBrush(COLOR_BTNSHADOW));
//       NtGdiPatBlt(hDC, CurrentRect.left, CurrentRect.top, CurrentRect.right - CurrentRect.left, 1, PATCOPY);
//       NtGdiPatBlt(hDC, CurrentRect.left, CurrentRect.top, 1, CurrentRect.bottom - CurrentRect.top, PATCOPY);
//
//       NtGdiSelectBrush(hDC, IntGetSysColorBrush(COLOR_BTNHIGHLIGHT));
//       NtGdiPatBlt(hDC, CurrentRect.left, CurrentRect.bottom - 1, CurrentRect.right - CurrentRect.left, 1, PATCOPY);
//       NtGdiPatBlt(hDC, CurrentRect.right - 1, CurrentRect.top, 1, CurrentRect.bottom - CurrentRect.top, PATCOPY);
//
//       RECTL_vInflateRect_g(&CurrentRect, -1, -1);
//    }
//
//    if (Flags & DC_FRAME) NC_DrawFrame(hDC, &CurrentRect, Active ? Active : (Flags & DC_ACTIVE), Style, ExStyle);
//
//    /* Draw caption */
//    if ((Style & WS_CAPTION) == WS_CAPTION)
//    {
//       HPEN PreviousPen;
//       BOOL Gradient = FALSE;
//
//       if (Flags & DC_REDRAWHUNGWND)
//       {
//          Flags &= ~DC_REDRAWHUNGWND;
//          Flags |= DC_NOSENDMSG;
//       }
//
//       if (UserSystemParametersInfo(SPI_GETGRADIENTCAPTIONS, 0, &Gradient, 0) && Gradient)
//       {
//          Flags |= DC_GRADIENT;
//       }
//
//       if (Active)
//       {
//          if (pWnd->state & WNDS_ACTIVEFRAME)
//             Flags |= DC_ACTIVE;
//          else
//          {
//             ERR("Wnd is active and not set active!\n");
//          }
//       }
//
//       TempRect = CurrentRect;
//
//       if (ExStyle & WS_EX_TOOLWINDOW)
//       {
//          Flags |= DC_SMALLCAP;
//          TempRect.bottom = TempRect.top + UserGetSystemMetrics(SM_CYSMCAPTION) - 1;
//          CurrentRect.top += UserGetSystemMetrics(SM_CYSMCAPTION);
//       }
//       else
//       {
//          TempRect.bottom = TempRect.top + UserGetSystemMetrics(SM_CYCAPTION) - 1;
//          CurrentRect.top += UserGetSystemMetrics(SM_CYCAPTION);
//       }
//
//       UserDrawCaption(pWnd, hDC, &TempRect, NULL, NULL, NULL, Flags);
//
//       /* Draw buttons */
//       if (Style & WS_SYSMENU)
//       {
//          UserDrawCaptionButton(pWnd, &TempRect, Style, ExStyle, hDC, FALSE, DFCS_CAPTIONCLOSE);
//          if ((Style & (WS_MAXIMIZEBOX | WS_MINIMIZEBOX)) && !(ExStyle & WS_EX_TOOLWINDOW))
//          {
//             UserDrawCaptionButton(pWnd, &TempRect, Style, ExStyle, hDC, FALSE, DFCS_CAPTIONMIN);
//             UserDrawCaptionButton(pWnd, &TempRect, Style, ExStyle, hDC, FALSE, DFCS_CAPTIONMAX);
//          }
//       }
//       if (!(Style & WS_MINIMIZE))
//       {
//         /* Line under caption */
//          PreviousPen = NtGdiSelectPen(hDC, NtGdiGetStockObject(DC_PEN));
//
//          IntSetDCPenColor( hDC, IntGetSysColor(
//                          ((ExStyle & (WS_EX_STATICEDGE | WS_EX_CLIENTEDGE | WS_EX_DLGMODALFRAME)) == WS_EX_STATICEDGE) ?
//                           COLOR_WINDOWFRAME : COLOR_3DFACE));
//
//          GreMoveTo(hDC, TempRect.left, TempRect.bottom, NULL);
//
//          NtGdiLineTo(hDC, TempRect.right, TempRect.bottom);
//
//          NtGdiSelectPen(hDC, PreviousPen);
//       }
//    }
//
//    if (!(Style & WS_MINIMIZE))
//    {
//      /* Draw menu bar */
//      if (pWnd->state & WNDS_HASMENU && pWnd->IDMenu) // Should be pWnd->spmenu
//      {
//          if (!(Flags & DC_NOSENDMSG))
//          {
//              PMENU menu;
//              // Fix crash in test_menu_locked_by_window, should use pWnd->spmenu....
//              if ((menu = UserGetMenuObject(UlongToHandle(pWnd->IDMenu)))) // FIXME! Use pWnd->spmenu,
//              {
//                 TempRect = CurrentRect;
//                 TempRect.bottom = TempRect.top + menu->cyMenu; // Should be pWnd->spmenu->cyMenu;
//                 CurrentRect.top += MENU_DrawMenuBar(hDC, &TempRect, pWnd, FALSE);
//              }
//          }
//      }
//
//      if (ExStyle & WS_EX_CLIENTEDGE)
//      {
//          DrawEdge(hDC, &CurrentRect, EDGE_SUNKEN, BF_RECT | BF_ADJUST);
//      }
//
//      /* Draw the scrollbars */
//      if ((Style & WS_VSCROLL) && (Style & WS_HSCROLL) &&
//          IntIsScrollBarVisible(pWnd, OBJID_VSCROLL) && IntIsScrollBarVisible(pWnd, OBJID_HSCROLL))
//      {
//         RECT ParentClientRect;
//
//         TempRect = CurrentRect;
//
//         if (ExStyle & WS_EX_LEFTSCROLLBAR)
//            TempRect.right = TempRect.left + UserGetSystemMetrics(SM_CXVSCROLL);
//         else
//            TempRect.left = TempRect.right - UserGetSystemMetrics(SM_CXVSCROLL);
//
//         TempRect.top = TempRect.bottom - UserGetSystemMetrics(SM_CYHSCROLL);
//
//         FillRect(hDC, &TempRect, IntGetSysColorBrush(COLOR_BTNFACE));
//
//         if (Parent)
//         {
//            IntGetClientRect(Parent, &ParentClientRect);
//
//            if (HASSIZEGRIP(Style, ExStyle, Parent->style, WindowRect, ParentClientRect))
//            {
//               DrawFrameControl(hDC, &TempRect, DFC_SCROLL, DFCS_SCROLLSIZEGRIP);
//            }
//         }
//
//         IntDrawScrollBar(pWnd, hDC, SB_VERT);
//         IntDrawScrollBar(pWnd, hDC, SB_HORZ);
//      }
//      else
//      {
//         if (Style & WS_VSCROLL && IntIsScrollBarVisible(pWnd, OBJID_VSCROLL))
//         {
//            IntDrawScrollBar(pWnd, hDC, SB_VERT);
//         }
//         else if (Style & WS_HSCROLL && IntIsScrollBarVisible(pWnd, OBJID_HSCROLL))
//         {
//            IntDrawScrollBar(pWnd, hDC, SB_HORZ);
//         }
//      }
//    }
//    return 0; // For WM_NCPAINT message, return 0.
// }
//
// /* Win: xxxCalcClientRect */
// LRESULT NC_HandleNCCalcSize( PWND Wnd, WPARAM wparam, RECTL *Rect, BOOL Suspended )
// {
//    LRESULT Result = 0;
//    SIZE WindowBorders;
//    RECT OrigRect;
//    LONG Style = Wnd->style;
//    LONG exStyle = Wnd->ExStyle;
//
//    if (Rect == NULL)
//    {
//       return Result;
//    }
//    OrigRect = *Rect;
//
//    Wnd->state &= ~WNDS_HASCAPTION;
//
//    if (wparam)
//    {
//       if (Wnd->pcls->style & CS_VREDRAW)
//       {
//          Result |= WVR_VREDRAW;
//       }
//       if (Wnd->pcls->style & CS_HREDRAW)
//       {
//          Result |= WVR_HREDRAW;
//       }
//       Result |= WVR_VALIDRECTS;
//    }
//
//    if (!(Wnd->style & WS_MINIMIZE))
//    {
//       if (UserHasWindowEdge(Wnd->style, Wnd->ExStyle))
//       {
//          UserGetWindowBorders(Wnd->style, Wnd->ExStyle, &WindowBorders, FALSE);
//          RECTL_vInflateRect_g(Rect, -WindowBorders.cx, -WindowBorders.cy);
//       }
//       else if ((Wnd->ExStyle & WS_EX_STATICEDGE) || (Wnd->style & WS_BORDER))
//       {
//          RECTL_vInflateRect_g(Rect, -1, -1);
//       }
//
//       if ((Wnd->style & WS_CAPTION) == WS_CAPTION)
//       {
//          Wnd->state |= WNDS_HASCAPTION;
//
//          if (Wnd->ExStyle & WS_EX_TOOLWINDOW)
//             Rect->top += UserGetSystemMetrics(SM_CYSMCAPTION);
//          else
//             Rect->top += UserGetSystemMetrics(SM_CYCAPTION);
//       }
//
//       if (HAS_MENU(Wnd, Style))
//       {
//          HDC hDC = UserGetDCEx(Wnd, 0, DCX_USESTYLE | DCX_WINDOW);
//
//          Wnd->state |= WNDS_HASMENU;
//
//          if (hDC)
//          {
//            RECT CliRect = *Rect;
//            CliRect.bottom -= OrigRect.top;
//            CliRect.right -= OrigRect.left;
//            CliRect.left -= OrigRect.left;
//            CliRect.top -= OrigRect.top;
//            if (!Suspended) Rect->top += MENU_DrawMenuBar(hDC, &CliRect, Wnd, TRUE);
//            UserReleaseDC(Wnd, hDC, FALSE);
//          }
//       }
//
//       if (Wnd->ExStyle & WS_EX_CLIENTEDGE)
//       {
//          RECTL_vInflateRect_g(Rect, -2 * UserGetSystemMetrics(SM_CXBORDER), -2 * UserGetSystemMetrics(SM_CYBORDER));
//       }
//
//       if (Style & WS_VSCROLL)
//       {
//          if (Rect->right - Rect->left >= UserGetSystemMetrics(SM_CXVSCROLL))
//          {
//              Wnd->state |= WNDS_HASVERTICALSCROOLLBAR;
//
//              /* rectangle is in screen coords when wparam is false */
//              if (!wparam && (exStyle & WS_EX_LAYOUTRTL)) exStyle ^= WS_EX_LEFTSCROLLBAR;
//
//              if((exStyle & WS_EX_LEFTSCROLLBAR) != 0)
//                  Rect->left  += UserGetSystemMetrics(SM_CXVSCROLL);
//              else
//                  Rect->right -= UserGetSystemMetrics(SM_CXVSCROLL);
//           }
//       }
//
//       if (Style & WS_HSCROLL)
//       {
//          if( Rect->bottom - Rect->top > UserGetSystemMetrics(SM_CYHSCROLL))
//          {
//               Wnd->state |= WNDS_HASHORIZONTALSCROLLBAR;
//
//               Rect->bottom -= UserGetSystemMetrics(SM_CYHSCROLL);
//          }
//       }
//
//       if (Rect->top > Rect->bottom)
//          Rect->bottom = Rect->top;
//
//       if (Rect->left > Rect->right)
//          Rect->right = Rect->left;
//    }
//    else
//    {
//       Rect->right = Rect->left;
//       Rect->bottom = Rect->top;
//    }
//
//    return Result;
// }
//
// static
// INT NC_DoNCActive(PWND Wnd)
// {
//    INT Ret = 0;
//
//    if ( IntGetSysColor(COLOR_CAPTIONTEXT) != IntGetSysColor(COLOR_INACTIVECAPTIONTEXT) ||
//         IntGetSysColor(COLOR_ACTIVECAPTION) != IntGetSysColor(COLOR_INACTIVECAPTION) )
//        Ret = DC_CAPTION;
//
//    if (!(Wnd->style & WS_MINIMIZED) && UserHasThickFrameStyle(Wnd->style, Wnd->ExStyle))
//    {
//       //if (IntGetSysColor(COLOR_ACTIVEBORDER) != IntGetSysColor(COLOR_INACTIVEBORDER)) // Why are these the same?
//       {
//           Ret = DC_FRAME;
//       }
//    }
//    return Ret;
// }
//
// LRESULT NC_HandleNCActivate( PWND Wnd, WPARAM wParam, LPARAM lParam )
// {
//    INT Flags;
//   /* Lotus Notes draws menu descriptions in the caption of its main
//    * window. When it wants to restore original "system" view, it just
//    * sends WM_NCACTIVATE message to itself. Any optimizations here in
//    * attempt to minimize redrawings lead to a not restored caption.
//    */
//    if (wParam & DC_ACTIVE)
//    {
//       Wnd->state |= WNDS_ACTIVEFRAME|WNDS_HASCAPTION;
//       wParam = DC_CAPTION|DC_ACTIVE;
//    }
//    else
//    {
//       Wnd->state &= ~WNDS_ACTIVEFRAME;
//       wParam = DC_CAPTION;
//    }
//
//    if ((Wnd->state & WNDS_NONCPAINT) || !(Wnd->style & WS_VISIBLE))
//       return TRUE;
//
//    /* This isn't documented but is reproducible in at least XP SP2 and
//     * Outlook 2007 depends on it
//     */
//    // MSDN:
//    // If this parameter is set to -1, DefWindowProc does not repaint the
//    // nonclient area to reflect the state change.
//    if ( lParam != -1 &&
//         ( Flags = NC_DoNCActive(Wnd)) != 0 )
//    {
//       HDC hDC;
//       HRGN hRgnTemp = NULL, hRgn = (HRGN)lParam;
//
//       if (GreIsHandleValid(hRgn))
//       {
//          hRgnTemp = NtGdiCreateRectRgn(0, 0, 0, 0);
//          if (NtGdiCombineRgn(hRgnTemp, hRgn, 0, RGN_COPY) == ERROR)
//          {
//             GreDeleteObject(hRgnTemp);
//             hRgnTemp = NULL;
//          }
//       }
//
//       if ((hDC = UserGetDCEx(Wnd, hRgnTemp, DCX_WINDOW|DCX_USESTYLE)))
//       {
//          NC_DoNCPaint(Wnd, hDC, wParam | Flags); // Redraw MENUs.
//          UserReleaseDC(Wnd, hDC, FALSE);
//       }
//       else
//          GreDeleteObject(hRgnTemp);
//    }
//
//    return TRUE;
// }
//
// VOID
// NC_DoButton(PWND pWnd, WPARAM wParam, LPARAM lParam)
// {
//    MSG Msg;
//    HDC WindowDC;
//    BOOL Pressed = TRUE, OldState;
//    WPARAM SCMsg;
//    PMENU SysMenu;
//    ULONG ButtonType;
//    DWORD Style;
//    UINT MenuState;
//
//    Style = pWnd->style;
//    switch (wParam)
//    {
//       case HTCLOSE:
//          SysMenu = IntGetSystemMenu(pWnd, FALSE);
//          MenuState = IntGetMenuState(SysMenu ? UserHMGetHandle(SysMenu) : NULL, SC_CLOSE, MF_BYCOMMAND); /* in case of error MenuState==0xFFFFFFFF */
//          if (!(Style & WS_SYSMENU) || (MenuState & (MF_GRAYED|MF_DISABLED)) || (pWnd->pcls->style & CS_NOCLOSE))
//             return;
//          ButtonType = DFCS_CAPTIONCLOSE;
//          SCMsg = SC_CLOSE;
//          break;
//       case HTMINBUTTON:
//          if (!(Style & WS_MINIMIZEBOX))
//             return;
//          ButtonType = DFCS_CAPTIONMIN;
//          SCMsg = ((Style & WS_MINIMIZE) ? SC_RESTORE : SC_MINIMIZE);
//          break;
//       case HTMAXBUTTON:
//          if (!(Style & WS_MAXIMIZEBOX))
//             return;
//          ButtonType = DFCS_CAPTIONMAX;
//          SCMsg = ((Style & WS_MAXIMIZE) ? SC_RESTORE : SC_MAXIMIZE);
//          break;
//
//       default:
//          ASSERT(FALSE);
//          return;
//    }
//
//    /*
//     * FIXME: Not sure where to do this, but we must flush the pending
//     * window updates when someone clicks on the close button and at
//     * the same time the window is overlapped with another one. This
//     * looks like a good place for now...
//     */
//    co_IntUpdateWindows(pWnd, RDW_ALLCHILDREN, FALSE);
//
//    WindowDC = UserGetWindowDC(pWnd);
//    UserDrawCaptionButtonWnd(pWnd, WindowDC, TRUE, ButtonType);
//
//    co_UserSetCapture(UserHMGetHandle(pWnd));
//
//    for (;;)
//    {
//       if (!co_IntGetPeekMessage(&Msg, 0, WM_MOUSEFIRST, WM_MOUSELAST, PM_REMOVE, TRUE)) break;
//       if (IntCallMsgFilter( &Msg, MSGF_MAX )) continue;
//
//       if (Msg.message == WM_LBUTTONUP)
//          break;
//
//       if (Msg.message != WM_MOUSEMOVE)
//          continue;
//
//       OldState = Pressed;
//       Pressed = (GetNCHitEx(pWnd, Msg.pt) == wParam);
//       if (Pressed != OldState)
//          UserDrawCaptionButtonWnd(pWnd, WindowDC, Pressed, ButtonType);
//    }
//
//    if (Pressed)
//       UserDrawCaptionButtonWnd(pWnd, WindowDC, FALSE, ButtonType);
//    IntReleaseCapture();
//    UserReleaseDC(pWnd, WindowDC, FALSE);
//    if (Pressed)
//       co_IntSendMessage(UserHMGetHandle(pWnd), WM_SYSCOMMAND, SCMsg, SCMsg == SC_CLOSE ? lParam : MAKELONG(Msg.pt.x,Msg.pt.y));
// }
//
//
// LRESULT
// NC_HandleNCLButtonDown(PWND pWnd, WPARAM wParam, LPARAM lParam)
// {
//     switch (wParam)
//     {
//         case HTCAPTION:
//         {
//             PWND TopWnd = pWnd, parent;
//             while(1)
//             {
//                 if ((TopWnd->style & (WS_POPUP|WS_CHILD)) != WS_CHILD)
//                     break;
//                 parent = UserGetAncestor( TopWnd, GA_PARENT );
//                 if (!parent || UserIsDesktopWindow(parent)) break;
//                 TopWnd = parent;
//             }
//
//             if ( (pWnd && (pWnd->ExStyle & WS_EX_NOACTIVATE)) ||
//                  co_IntSetForegroundWindowMouse(TopWnd) ||
//                  //NtUserCallHwndLock(hTopWnd, HWNDLOCK_ROUTINE_SETFOREGROUNDWINDOWMOUSE) ||
//                  UserGetActiveWindow() == UserHMGetHandle(TopWnd))
//             {
//                co_IntSendMessage(UserHMGetHandle(pWnd), WM_SYSCOMMAND, SC_MOVE + HTCAPTION, lParam);
//             }
//             break;
//         }
//         case HTSYSMENU:
//         {
//           LONG style = pWnd->style;
//           if (style & WS_SYSMENU)
//           {
//               if(!(style & WS_MINIMIZE) )
//               {
//                 RECT rect;
//                 HDC hDC = UserGetWindowDC(pWnd);
//                 NC_GetInsideRect(pWnd, &rect);
//                 UserDrawSysMenuButton(pWnd, hDC, &rect, TRUE);
//                 UserReleaseDC( pWnd, hDC, FALSE );
//               }
// 	      co_IntSendMessage(UserHMGetHandle(pWnd), WM_SYSCOMMAND, SC_MOUSEMENU + HTSYSMENU, lParam);
// 	  }
// 	  break;
//         }
//         case HTMENU:
//         {
//             co_IntSendMessage(UserHMGetHandle(pWnd), WM_SYSCOMMAND, SC_MOUSEMENU + HTMENU, lParam);
//             break;
//         }
//         case HTHSCROLL:
//         {
//             co_IntSendMessage(UserHMGetHandle(pWnd), WM_SYSCOMMAND, SC_HSCROLL + HTHSCROLL, lParam);
//             break;
//         }
//         case HTVSCROLL:
//         {
//             co_IntSendMessage(UserHMGetHandle(pWnd), WM_SYSCOMMAND, SC_VSCROLL + HTVSCROLL, lParam);
//             break;
//         }
//         case HTMINBUTTON:
//         case HTMAXBUTTON:
//         case HTCLOSE:
//         {
//           NC_DoButton(pWnd, wParam, lParam);
//           break;
//         }
//         case HTLEFT:
//         case HTRIGHT:
//         case HTTOP:
//         case HTBOTTOM:
//         case HTTOPLEFT:
//         case HTTOPRIGHT:
//         case HTBOTTOMLEFT:
//         case HTBOTTOMRIGHT:
//         {
//            /* Old comment:
//             * "make sure hittest fits into 0xf and doesn't overlap with HTSYSMENU"
//             * This was previously done by setting wParam=SC_SIZE + wParam - 2
//             */
//            /* But that is not what WinNT does. Instead it sends this. This
//             * is easy to differentiate from HTSYSMENU, because HTSYSMENU adds
//             * SC_MOUSEMENU into wParam.
//             */
//             co_IntSendMessage(UserHMGetHandle(pWnd), WM_SYSCOMMAND, SC_SIZE + wParam - (HTLEFT - WMSZ_LEFT), lParam);
//             break;
//         }
//         case HTBORDER:
//             break;
//     }
//     return(0);
// }
//
//
// LRESULT
// NC_HandleNCLButtonDblClk(PWND pWnd, WPARAM wParam, LPARAM lParam)
// {
//   ULONG Style;
//
//   Style = pWnd->style;
//   switch(wParam)
//   {
//     case HTCAPTION:
//     {
//       /* Maximize/Restore the window */
//       if((Style & WS_CAPTION) == WS_CAPTION && (Style & WS_MAXIMIZEBOX))
//       {
//         co_IntSendMessage(UserHMGetHandle(pWnd), WM_SYSCOMMAND, ((Style & (WS_MINIMIZE | WS_MAXIMIZE)) ? SC_RESTORE : SC_MAXIMIZE), 0);
//       }
//       break;
//     }
//     case HTSYSMENU:
//     {
//       PMENU SysMenu = IntGetSystemMenu(pWnd, FALSE);
//       UINT state = IntGetMenuState(SysMenu ? UserHMGetHandle(SysMenu) : NULL, SC_CLOSE, MF_BYCOMMAND);
//
//       /* If the close item of the sysmenu is disabled or not present do nothing */
//       if ((state & (MF_DISABLED | MF_GRAYED)) || (state == 0xFFFFFFFF))
//           break;
//
//       co_IntSendMessage(UserHMGetHandle(pWnd), WM_SYSCOMMAND, SC_CLOSE, lParam);
//       break;
//     }
//     case HTTOP:
//     case HTBOTTOM:
//     {
//       RECT sizingRect = pWnd->rcWindow, mouseRect;
//
//       if (pWnd->ExStyle & WS_EX_MDICHILD)
//           break;
//
//       UserSystemParametersInfo(SPI_GETWORKAREA, 0, &mouseRect, 0);
//
//       co_WinPosSetWindowPos(pWnd,
//                             NULL,
//                             sizingRect.left,
//                             mouseRect.top,
//                             sizingRect.right - sizingRect.left,
//                             mouseRect.bottom - mouseRect.top,
//                             0);
//       break;
//     }
//     default:
//       return NC_HandleNCLButtonDown(pWnd, wParam, lParam);
//   }
//   return(0);
// }
//
// /***********************************************************************
//  *           NC_HandleNCRButtonDown
//  *
//  * Handle a WM_NCRBUTTONDOWN message. Called from DefWindowProc().
//  */
// LRESULT NC_HandleNCRButtonDown(PWND pwnd, WPARAM wParam, LPARAM lParam)
// {
//   MSG msg;
//   INT hittest = wParam;
//
//   switch (hittest)
//   {
//   case HTCAPTION:
//   case HTSYSMENU:
//       if (!IntGetSystemMenu( pwnd, FALSE )) break;
//
//       co_UserSetCapture( UserHMGetHandle(pwnd) );
//       for (;;)
//       {
//           if (!co_IntGetPeekMessage(&msg, 0, WM_MOUSEFIRST, WM_MOUSELAST, PM_REMOVE, TRUE)) break;
//           if (IntCallMsgFilter( &msg, MSGF_MAX )) continue;
//           if (msg.message == WM_RBUTTONUP)
//           {
//              hittest = GetNCHitEx( pwnd, msg.pt );
//              break;
//           }
//           if (UserHMGetHandle(pwnd) != IntGetCapture()) return 0;
//       }
//       IntReleaseCapture();
//
//       if (hittest == HTCAPTION || hittest == HTSYSMENU || hittest == HTHSCROLL || hittest == HTVSCROLL)
//       {
//          TRACE("Msg pt %x and Msg.lParam %x and lParam %x\n",MAKELONG(msg.pt.x,msg.pt.y),msg.lParam,lParam);
//          co_IntSendMessage( UserHMGetHandle(pwnd), WM_CONTEXTMENU, (WPARAM)UserHMGetHandle(pwnd), MAKELONG(msg.pt.x,msg.pt.y));
//       }
//       break;
//   }
//   return 0;
// }
//
//
//
// DWORD FASTCALL
// GetNCHitEx(PWND pWnd, POINT pt)
// {
//    RECT rcWindow, rcClient;
//    DWORD Style, ExStyle;
//
//    if (!pWnd) return HTNOWHERE;
//
//    if (UserIsDesktopWindow(pWnd))
//    {
//       rcClient.left = rcClient.top = rcWindow.left = rcWindow.top = 0;
//       rcWindow.right  = UserGetSystemMetrics(SM_CXSCREEN);
//       rcWindow.bottom = UserGetSystemMetrics(SM_CYSCREEN);
//       rcClient.right  = UserGetSystemMetrics(SM_CXSCREEN);
//       rcClient.bottom = UserGetSystemMetrics(SM_CYSCREEN);
//    }
//    else
//    {
//       rcClient = pWnd->rcClient;
//       rcWindow = pWnd->rcWindow;
//    }
//
//    if (!RECTL_bPointInRect(&rcWindow, pt.x, pt.y)) return HTNOWHERE;
//
//    Style = pWnd->style;
//    ExStyle = pWnd->ExStyle;
//
//    if (Style & WS_MINIMIZE) return HTCAPTION;
//
//    if (RECTL_bPointInRect( &rcClient,  pt.x, pt.y )) return HTCLIENT;
//
//    /* Check borders */
//    if (HAS_THICKFRAME( Style, ExStyle ))
//    {
//       RECTL_vInflateRect_g(&rcWindow, -UserGetSystemMetrics(SM_CXFRAME), -UserGetSystemMetrics(SM_CYFRAME) );
//       if (!RECTL_bPointInRect(&rcWindow, pt.x, pt.y ))
//       {
//             /* Check top sizing border */
//             if (pt.y < rcWindow.top)
//             {
//                 if (pt.x < rcWindow.left+UserGetSystemMetrics(SM_CXSIZE)) return HTTOPLEFT;
//                 if (pt.x >= rcWindow.right-UserGetSystemMetrics(SM_CXSIZE)) return HTTOPRIGHT;
//                 return HTTOP;
//             }
//             /* Check bottom sizing border */
//             if (pt.y >= rcWindow.bottom)
//             {
//                 if (pt.x < rcWindow.left+UserGetSystemMetrics(SM_CXSIZE)) return HTBOTTOMLEFT;
//                 if (pt.x >= rcWindow.right-UserGetSystemMetrics(SM_CXSIZE)) return HTBOTTOMRIGHT;
//                 return HTBOTTOM;
//             }
//             /* Check left sizing border */
//             if (pt.x < rcWindow.left)
//             {
//                 if (pt.y < rcWindow.top+UserGetSystemMetrics(SM_CYSIZE)) return HTTOPLEFT;
//                 if (pt.y >= rcWindow.bottom-UserGetSystemMetrics(SM_CYSIZE)) return HTBOTTOMLEFT;
//                 return HTLEFT;
//             }
//             /* Check right sizing border */
//             if (pt.x >= rcWindow.right)
//             {
//                 if (pt.y < rcWindow.top+UserGetSystemMetrics(SM_CYSIZE)) return HTTOPRIGHT;
//                 if (pt.y >= rcWindow.bottom-UserGetSystemMetrics(SM_CYSIZE)) return HTBOTTOMRIGHT;
//                 return HTRIGHT;
//             }
//         }
//     }
//     else  /* No thick frame */
//     {
//         if (HAS_DLGFRAME( Style, ExStyle ))
//             RECTL_vInflateRect_g(&rcWindow, -UserGetSystemMetrics(SM_CXDLGFRAME), -UserGetSystemMetrics(SM_CYDLGFRAME));
//         else if (HAS_THINFRAME( Style, ExStyle ))
//             RECTL_vInflateRect_g(&rcWindow, -UserGetSystemMetrics(SM_CXBORDER), -UserGetSystemMetrics(SM_CYBORDER));
//         else if (HAS_CLIENTFRAME( Style, ExStyle ))
//             RECTL_vInflateRect_g(&rcWindow, -UserGetSystemMetrics(SM_CXEDGE), -UserGetSystemMetrics(SM_CYEDGE));
//         if (!RECTL_bPointInRect( &rcWindow, pt.x, pt.y  )) return HTBORDER;
//     }
//
//     /* Check caption */
//
//     if ((Style & WS_CAPTION) == WS_CAPTION)
//     {
//         if (ExStyle & WS_EX_TOOLWINDOW)
//             rcWindow.top += UserGetSystemMetrics(SM_CYSMCAPTION) - 1;
//         else
//             rcWindow.top += UserGetSystemMetrics(SM_CYCAPTION) - 1;
//         if (!RECTL_bPointInRect( &rcWindow, pt.x, pt.y ))
//         {
//             BOOL min_or_max_box = (Style & WS_SYSMENU) && (Style & (WS_MINIMIZEBOX|WS_MAXIMIZEBOX));
//             if (ExStyle & WS_EX_LAYOUTRTL)
//             {
//                 /* Check system menu */
//                 if ((Style & WS_SYSMENU) && !(ExStyle & WS_EX_TOOLWINDOW) && NC_IconForWindow(pWnd))
//                 {
//                     rcWindow.right -= UserGetSystemMetrics(SM_CYCAPTION) - 1;
//                     if (pt.x > rcWindow.right) return HTSYSMENU;
//                 }
//
//                 /* Check close button */
//                 if (Style & WS_SYSMENU)
//                 {
//                     rcWindow.left += UserGetSystemMetrics(SM_CYCAPTION);
//                     if (pt.x < rcWindow.left) return HTCLOSE;
//                 }
//
//                 /* Check maximize box */
//                 /* In Win95 there is automatically a Maximize button when there is a minimize one */
//                 if (min_or_max_box && !(ExStyle & WS_EX_TOOLWINDOW))
//                 {
//                     rcWindow.left += UserGetSystemMetrics(SM_CXSIZE);
//                     if (pt.x < rcWindow.left) return HTMAXBUTTON;
//                 }
//
//                 /* Check minimize box */
//                 if (min_or_max_box && !(ExStyle & WS_EX_TOOLWINDOW))
//                 {
//                     rcWindow.left += UserGetSystemMetrics(SM_CXSIZE);
//                     if (pt.x < rcWindow.left) return HTMINBUTTON;
//                 }
//             }
//             else
//             {
//                 /* Check system menu */
//                 if ((Style & WS_SYSMENU) && !(ExStyle & WS_EX_TOOLWINDOW) && NC_IconForWindow(pWnd))
//                 {
//                     rcWindow.left += UserGetSystemMetrics(SM_CYCAPTION) - 1;
//                     if (pt.x < rcWindow.left) return HTSYSMENU;
//                 }
//
//                 /* Check close button */
//                 if (Style & WS_SYSMENU)
//                 {
//                     rcWindow.right -= UserGetSystemMetrics(SM_CYCAPTION);
//                     if (pt.x > rcWindow.right) return HTCLOSE;
//                 }
//
//                 /* Check maximize box */
//                 /* In Win95 there is automatically a Maximize button when there is a minimize one */
//                 if (min_or_max_box && !(ExStyle & WS_EX_TOOLWINDOW))
//                 {
//                     rcWindow.right -= UserGetSystemMetrics(SM_CXSIZE);
//                     if (pt.x > rcWindow.right) return HTMAXBUTTON;
//                 }
//
//                 /* Check minimize box */
//                 if (min_or_max_box && !(ExStyle & WS_EX_TOOLWINDOW))
//                 {
//                     rcWindow.right -= UserGetSystemMetrics(SM_CXSIZE);
//                     if (pt.x > rcWindow.right) return HTMINBUTTON;
//                 }
//             }
//             return HTCAPTION;
//         }
//     }
//
//       /* Check menu bar */
//
//     if (HAS_MENU( pWnd, Style ) && (pt.y < rcClient.top) &&
//         (pt.x >= rcClient.left) && (pt.x < rcClient.right))
//         return HTMENU;
//
//       /* Check vertical scroll bar */
//
//     if (ExStyle & WS_EX_LAYOUTRTL) ExStyle ^= WS_EX_LEFTSCROLLBAR;
//     if (Style & WS_VSCROLL)
//     {
//         if((ExStyle & WS_EX_LEFTSCROLLBAR) != 0)
//             rcClient.left -= UserGetSystemMetrics(SM_CXVSCROLL);
//         else
//             rcClient.right += UserGetSystemMetrics(SM_CXVSCROLL);
//         if (RECTL_bPointInRect( &rcClient, pt.x, pt.y )) return HTVSCROLL;
//     }
//
//       /* Check horizontal scroll bar */
//
//     if (Style & WS_HSCROLL)
//     {
//         rcClient.bottom += UserGetSystemMetrics(SM_CYHSCROLL);
//         if (RECTL_bPointInRect( &rcClient, pt.x, pt.y ))
//         {
//             /* Check size box */
//             if ((Style & WS_VSCROLL) &&
//                 ((((ExStyle & WS_EX_LEFTSCROLLBAR) != 0) && (pt.x <= rcClient.left + UserGetSystemMetrics(SM_CXVSCROLL))) ||
//                 (((ExStyle & WS_EX_LEFTSCROLLBAR) == 0) && (pt.x >= rcClient.right - UserGetSystemMetrics(SM_CXVSCROLL)))))
//                 return HTSIZE;
//             return HTHSCROLL;
//         }
//     }
//
//     /* Has to return HTNOWHERE if nothing was found
//        Could happen when a window has a customized non client area */
//     return HTNOWHERE;
// }
//
// /* EOF */
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
