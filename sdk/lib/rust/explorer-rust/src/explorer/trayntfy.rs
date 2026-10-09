//! trayntfy.cpp + traydeskbtn.cpp — the notification-area host window
//! (`CTrayNotifyWnd`) and the "Show Desktop" button (`CTrayShowDesktopButton`)
//! it embeds BY VALUE. Translated as a single cluster so the embedded-member
//! layout is free (both are flat `#[repr(C)]` objects reached only through a
//! COM interface pointer or their own WndProc, never by external layout).
//!
//! Object model mirrors trayclock.rs / syspager.rs:
//!   * `CTrayNotifyWnd` = `CComCoClass + CComObjectRootEx + CWindowImpl +
//!     IOleWindow` → flat lpVtbl-first object; single QI-reachable interface
//!     (IOleWindow, IUnknown shares the vtbl). It creates the clock + pager via
//!     the already-Rust `CTrayClockWnd_CreateInstance`/`CSysPagerWnd_CreateInstance`
//!     factories, and hosts `CTrayShowDesktopButton m_ShowDesktopButton` inline.
//!   * `CTrayShowDesktopButton` = `CWindowImpl` only (NO COM) — its own window
//!     class "TrayShowDesktopButtonWClass", its own arch-split WndProc, `this`
//!     recovered from GWLP_USERDATA. A normal registered-class window (not a
//!     subclassed control), so it follows the trayclock Create pattern.
//!
//! The 7 `TrayShowDesktopButton_*` `#[no_mangle] extern "C"` shims at the bottom
//! REPLACE the old C++ `traydeskbtn_shim.cpp`: the now-Rust traywnd.rs obtains a
//! `CTrayShowDesktopButton*` (from CTrayNotifyWnd via TNWM_GETSHOWDESKTOPBUTTON)
//! and drives the button through these symbols. Deleting traydeskbtn_shim.cpp
//! and commenting traydeskbtn.cpp + trayntfy.cpp out of the CMake list hands the
//! symbols (CTrayNotifyWnd_CreateInstance + the 7 shims) to this Rust file.

use core::ffi::c_void;
use core::ptr::{null, null_mut};

use explorer_sys::com::*; // REFIID, S_OK, E_*, IID_IUnknown/IID_IOleWindow, IOleWindow(+Vtbl), IUnknown
use explorer_sys::externs::IUnknown_GetWindow;
use explorer_sys::globals::*; // hExplorerInstance, hProcessHeap
use explorer_sys::server::{
    get_window_this, interlocked_decrement, interlocked_increment, register_class_once,
    wndproc_nccreate_this, CloseThemeData, DrawThemeParentBackground, IsThemeActive, OpenThemeData,
    SetWindowTheme, CW_USEDEFAULT, HTHEME, TOOLINFOW, TOOLTIPS_CLASSW, TTF_IDISHWND, TTF_SUBCLASS,
    TTM_ADDTOOLW, TTS_ALWAYSTIP, TTS_NOPREFIX, TTTOOLINFOW_V1_SIZE,
};
use explorer_sys::MAKEINTRESOURCEW;

use crate::explorer::settings::{g_TaskbarSettings, GetHideClock};
use crate::explorer::syspager::CSysPagerWnd_CreateInstance;
use crate::explorer::trayclock::CTrayClockWnd_CreateInstance;
use explorer_sys::types::TaskbarSettings;

use winsrv_sys::w;
use winsrv_sys::winsrv::*;

// ===========================================================================
// file-scope constants (trayntfy.cpp / traydeskbtn.{h,cpp}) + precomp.h msgs
// ===========================================================================

const TRAY_NOTIFY_WND_SPACING_X: LONG = 1;
const TRAY_NOTIFY_WND_SPACING_Y: LONG = 1;
const CLOCK_TEXT_HACK: LONG = 4;

// precomp.h tray/notify window messages
const TWM_SETTINGSCHANGED: UINT = WM_USER + 300;
const TNWM_GETMINIMUMSIZE: UINT = WM_USER + 0x100;
const TNWM_GETSHOWDESKTOPBUTTON: UINT = WM_USER + 0x7601;
const NTNWM_REALIGN: UINT = 0x1;

// traydeskbtn.h / traydeskbtn.cpp
const TSDB_CLICK: UINT = WM_USER + 100;
const IDI_SHELL32_DESKTOP: i32 = 35;
const IDI_IMAGERES_DESKTOP: i32 = 110;
const SHOW_DESKTOP_TIMER_ID: UINT_PTR = 999;
const SHOW_DESKTOP_TIMER_INTERVAL: UINT = 200;

// resource.h
const IDS_TRAYDESKBTN_TOOLTIP: u16 = 814;

// traywnd.rs: shellcmds — TRAYCMD_TOGGLE_DESKTOP
const TRAYCMD_TOGGLE_DESKTOP: WPARAM = 407;

// uxtheme theme parts / states (vsstyle.h / vssym32.h)
const TNP_BACKGROUND: i32 = 1; // enum TRAYNOTIFYPARTS
const TP_BUTTON: i32 = 1; // enum TOOLBARPARTS
const TS_NORMAL: i32 = 1; // enum TOOLBARSTYLESTATES
const TS_HOT: i32 = 2;
const TS_PRESSED: i32 = 3;
const BP_PUSHBUTTON: i32 = 1; // enum BUTTONPARTS
const PBS_NORMAL: i32 = 1; // enum PUSHBUTTONSTATES
const PBS_HOT: i32 = 2;
const PBS_PRESSED: i32 = 3;
const TMT_CONTENTMARGINS: i32 = 3602; // vssym32.h

// ===========================================================================
// MARGINS (uxtheme.h) — not in the winsrv bindgen surface.
// ===========================================================================

#[repr(C)]
#[derive(Clone, Copy)]
struct MARGINS {
    cxLeftWidth: i32,
    cxRightWidth: i32,
    cyTopHeight: i32,
    cyBottomHeight: i32,
}

// ===========================================================================
// substrate not yet in explorer-sys/winsrv-sys (uxtheme extras, shell32
// ExtractIconExW, shlwapi SHSetWindowBits). explorer.exe already links
// uxtheme/shell32/shlwapi, so these resolve at link time.
// ===========================================================================

extern "system" {
    fn GetThemeMargins(
        hTheme: HTHEME,
        hdc: HDC,
        iPartId: i32,
        iStateId: i32,
        iPropId: i32,
        prc: *const RECT,
        pMargins: *mut MARGINS,
    ) -> HRESULT;
    fn IsThemePartDefined(hTheme: HTHEME, iPartId: i32, iStateId: i32) -> BOOL;
    fn DrawThemeBackground(
        hTheme: HTHEME,
        hdc: HDC,
        iPartId: i32,
        iStateId: i32,
        prc: *const RECT,
        pClipRect: *const RECT,
    ) -> HRESULT;
    fn IsThemeBackgroundPartiallyTransparent(hTheme: HTHEME, iPartId: i32, iStateId: i32) -> BOOL;
    fn GetWindowTheme(hwnd: HWND) -> HTHEME;
}

extern "system" {
    // shell32 (stdcall)
    fn ExtractIconExW(
        lpszFile: LPCWSTR,
        nIconIndex: i32,
        phiconLarge: *mut HICON,
        phiconSmall: *mut HICON,
        nIcons: UINT,
    ) -> UINT;
    // shlwapi (stdcall) — the SetWindowExStyle helper target (precomp.h).
    fn SHSetWindowBits(hWnd: HWND, iType: i32, dwMask: DWORD, dwFlags: DWORD) -> DWORD;
}

/// `SetWindowExStyle(hWnd, mask, style)` (precomp.h static inline) —
/// `SHSetWindowBits(hWnd, GWL_EXSTYLE, mask, style)`.
#[inline]
unsafe fn SetWindowExStyle(hWnd: HWND, dwStyleMask: LONG, dwStyle: LONG) -> LONG {
    SHSetWindowBits(hWnd, GWL_EXSTYLE, dwStyleMask as DWORD, dwStyle as DWORD) as LONG
}

/// `IsEqualGUID` (guiddef.h inline).
#[inline]
unsafe fn guid_eq(a: REFIID, b: *const IID) -> bool {
    let a = &*a;
    let b = &*b;
    a.Data1 == b.Data1 && a.Data2 == b.Data2 && a.Data3 == b.Data3 && a.Data4 == b.Data4
}

/// `CTooltips::Create` — a named-class CWindow::Create against
/// "tooltips_class32", default exStyle WS_EX_TOPMOST (mirrors trayclock).
unsafe fn tooltip_create(hwnd_parent: HWND, styles: DWORD) -> HWND {
    CreateWindowExW(
        WS_EX_TOPMOST,
        TOOLTIPS_CLASSW.as_ptr(),
        w!("").as_ptr(),
        styles,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        hwnd_parent,
        null_mut(),
        hExplorerInstance,
        null_mut(),
    )
}

// ###########################################################################
// CTrayShowDesktopButton (traydeskbtn.{h,cpp}) — CWindowImpl only, no COM.
// ###########################################################################

/// `CTrayShowDesktopButton` — flat `#[repr(C)]` (layout free; reached only via
/// its WndProc / the embedding member / the 7 extern "C" shims). `m_hWnd` first
/// (CWindowImpl), then the traydeskbtn.h members in declaration order.
#[repr(C)]
pub struct CTrayShowDesktopButton {
    m_hWnd: HWND, // CWindowImpl
    m_nClickedTime: LONG,
    m_hTheme: HTHEME,
    m_hFallbackTheme: HTHEME,
    m_ContentMargins: MARGINS,
    m_inset: SIZE,
    m_icon: HICON,
    m_szIcon: SIZE,
    m_highContrastMode: BOOL,
    m_drawWithDedicatedBackground: BOOL,
    m_bHovering: BOOL,
    m_hWndTaskbar: HWND,
    m_tooltip: HWND, // CTooltips → its HWND
    // public members (read/written by CTrayNotifyWnd + the shims):
    m_bPressed: BOOL,
    m_bHorizontal: BOOL,
}

/// `CTrayShowDesktopButton()` — the member-init list + zeroing the rest (the
/// embedding object is HeapAlloc'd; zero the whole button, then apply the
/// ctor's explicit non-zero fields — only `m_inset = {2,2}`).
unsafe fn tsdb_ctor(this: *mut CTrayShowDesktopButton) {
    core::ptr::write_bytes(
        this as *mut u8,
        0,
        core::mem::size_of::<CTrayShowDesktopButton>(),
    );
    (*this).m_inset = SIZE { cx: 2, cy: 2 };
}

/// `INT CTrayShowDesktopButton::WidthOrHeight() const`.
unsafe fn tsdb_width_or_height(this: *const CTrayShowDesktopButton) -> i32 {
    if IsThemeActive() != 0 && (*this).m_highContrastMode == 0 {
        if (*this).m_drawWithDedicatedBackground != 0 {
            if GetSystemMetrics(SM_TABLETPC as i32) != 0 {
                // TODO: DPI scaling.
                24
            } else {
                15
            }
        } else {
            let cur_margin = if (*this).m_bHorizontal != 0 {
                (*this).m_ContentMargins.cxLeftWidth + (*this).m_ContentMargins.cxRightWidth
            } else {
                (*this).m_ContentMargins.cyTopHeight + (*this).m_ContentMargins.cyBottomHeight
            };
            (16 + cur_margin).max(18) + 6
        }
    } else {
        (2 * GetSystemMetrics(SM_CXBORDER as i32) + GetSystemMetrics(SM_CXSMICON as i32))
            .max(2 * GetSystemMetrics(SM_CYBORDER as i32) + GetSystemMetrics(SM_CYSMICON as i32))
    }
}

/// `HRESULT CTrayShowDesktopButton::DoCreate(HWND hwndParent)`.
unsafe fn tsdb_do_create(this: *mut CTrayShowDesktopButton, hwnd_parent: HWND) -> HRESULT {
    let style: DWORD = WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS | BS_DEFPUSHBUTTON;
    tsdb_create(this, hwnd_parent, style);

    if (*this).m_hWnd.is_null() {
        return E_FAIL;
    }

    // Get desktop icon
    let icon_retrieval_failed = ExtractIconExW(
        w!("imageres.dll").as_ptr(),
        -IDI_IMAGERES_DESKTOP,
        null_mut(),
        &mut (*this).m_icon,
        1,
    ) == UINT_MAX;
    if icon_retrieval_failed || (*this).m_icon.is_null() {
        ExtractIconExW(
            w!("shell32.dll").as_ptr(),
            -IDI_SHELL32_DESKTOP,
            null_mut(),
            &mut (*this).m_icon,
            1,
        );
    }

    // Get appropriate size at which to display desktop icon
    (*this).m_szIcon.cx = GetSystemMetrics(SM_CXSMICON as i32);
    (*this).m_szIcon.cy = GetSystemMetrics(SM_CYSMICON as i32);

    // Create tooltip
    (*this).m_tooltip = tooltip_create((*this).m_hWnd, WS_POPUP | TTS_NOPREFIX | TTS_ALWAYSTIP);

    let mut ti: TOOLINFOW = core::mem::zeroed();
    ti.cbSize = TTTOOLINFOW_V1_SIZE();
    ti.uFlags = TTF_IDISHWND | TTF_SUBCLASS;
    ti.hwnd = (*this).m_hWnd;
    ti.uId = (*this).m_hWnd as UINT_PTR;
    ti.hinst = hExplorerInstance;
    ti.lpszText = MAKEINTRESOURCEW(IDS_TRAYDESKBTN_TOOLTIP) as LPWSTR;

    SendMessageW(
        (*this).m_tooltip,
        TTM_ADDTOOLW,
        0,
        &ti as *const TOOLINFOW as LPARAM,
    );

    // Prep visual style
    tsdb_ensure_window_theme(this, TRUE as BOOL);

    // Get HWND of Taskbar
    (*this).m_hWndTaskbar = GetParent(hwnd_parent);
    if IsWindow((*this).m_hWndTaskbar) == 0 {
        return E_FAIL;
    }

    S_OK
}

/// `LRESULT CTrayShowDesktopButton::OnClick(...)` — TSDB_CLICK handler.
unsafe fn tsdb_on_click(
    this: *mut CTrayShowDesktopButton,
    _u_msg: UINT,
    _w_param: WPARAM,
    _l_param: LPARAM,
    _b_handled: *mut BOOL,
) -> LRESULT {
    // The actual action can be delayed as an expected behaviour, but a too-late
    // action is unexpected.
    let n_time0 = (*this).m_nClickedTime;
    let n_time1 = GetMessageTime();
    if n_time1 - n_time0 >= 600 {
        // Ignore after 0.6 sec
        return 0;
    }

    // Show/Hide Desktop
    SendMessageW((*this).m_hWndTaskbar, WM_COMMAND, TRAYCMD_TOGGLE_DESKTOP, 0);

    0
}

/// `VOID CTrayShowDesktopButton::Click()` — called from OnLButtonUp and parent.
unsafe fn tsdb_click(this: *mut CTrayShowDesktopButton) {
    (*this).m_nClickedTime = GetMessageTime();
    PostMessageW((*this).m_hWnd, TSDB_CLICK, 0, 0);
}

unsafe fn tsdb_on_lbutton_up(
    this: *mut CTrayShowDesktopButton,
    _u_msg: UINT,
    _w_param: WPARAM,
    _l_param: LPARAM,
    _b_handled: *mut BOOL,
) -> LRESULT {
    (*this).m_bPressed = FALSE as BOOL;
    ReleaseCapture();
    InvalidateRect((*this).m_hWnd, null(), TRUE as BOOL);

    let mut pt: POINT = core::mem::zeroed();
    GetCursorPos(&mut pt);
    if tsdb_pt_in_button(this, &pt) != 0 {
        tsdb_click(this); // Left-click
    }
    0
}

unsafe fn tsdb_on_lbutton_down(
    this: *mut CTrayShowDesktopButton,
    _u_msg: UINT,
    _w_param: WPARAM,
    _l_param: LPARAM,
    _b_handled: *mut BOOL,
) -> LRESULT {
    (*this).m_bPressed = TRUE as BOOL;
    SetCapture((*this).m_hWnd);
    InvalidateRect((*this).m_hWnd, null(), TRUE as BOOL);
    0
}

unsafe fn tsdb_on_setting_changed(
    this: *mut CTrayShowDesktopButton,
    u_msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
    b_handled: *mut BOOL,
) -> LRESULT {
    let ret = tsdb_on_theme_changed(this, u_msg, w_param, l_param, b_handled);
    tsdb_ensure_window_theme(this, TRUE as BOOL);
    ret
}

unsafe fn tsdb_on_theme_changed(
    this: *mut CTrayShowDesktopButton,
    _u_msg: UINT,
    _w_param: WPARAM,
    _l_param: LPARAM,
    _b_handled: *mut BOOL,
) -> LRESULT {
    let mut hc_info: HIGHCONTRASTW = core::mem::zeroed();
    hc_info.cbSize = core::mem::size_of::<HIGHCONTRASTW>() as UINT;
    if SystemParametersInfoW(
        SPI_GETHIGHCONTRAST,
        core::mem::size_of::<HIGHCONTRASTW>() as UINT,
        &mut hc_info as *mut HIGHCONTRASTW as PVOID,
        FALSE as UINT,
    ) != 0
    {
        (*this).m_highContrastMode = (hc_info.dwFlags & HCF_HIGHCONTRASTON) as BOOL;
    }

    if !(*this).m_hTheme.is_null() {
        CloseThemeData((*this).m_hTheme);
        (*this).m_hTheme = null_mut();
    }
    if !(*this).m_hFallbackTheme.is_null() {
        CloseThemeData((*this).m_hFallbackTheme);
        (*this).m_hFallbackTheme = null_mut();
    }

    tsdb_ensure_window_theme(this, FALSE as BOOL);

    InvalidateRect((*this).m_hWnd, null(), TRUE as BOOL);
    0
}

unsafe fn tsdb_on_window_pos_changed(
    this: *mut CTrayShowDesktopButton,
    _u_msg: UINT,
    _w_param: WPARAM,
    _l_param: LPARAM,
    _b_handled: *mut BOOL,
) -> LRESULT {
    tsdb_ensure_window_theme(this, TRUE as BOOL);
    0
}

/// `VOID CTrayShowDesktopButton::EnsureWindowTheme(BOOL setTheme)`.
unsafe fn tsdb_ensure_window_theme(this: *mut CTrayShowDesktopButton, set_theme: BOOL) {
    if set_theme != 0 {
        SetWindowTheme(
            (*this).m_hWnd,
            if (*this).m_bHorizontal != 0 {
                w!("ShowDesktop").as_ptr()
            } else {
                w!("VerticalShowDesktop").as_ptr()
            },
            null(),
        );
    }

    if IsWindow((*this).m_hWndTaskbar) != 0 {
        (*this).m_hFallbackTheme = OpenThemeData((*this).m_hWndTaskbar, w!("Toolbar").as_ptr());
        GetThemeMargins(
            (*this).m_hFallbackTheme,
            null_mut(),
            TP_BUTTON,
            0,
            TMT_CONTENTMARGINS,
            null(),
            &mut (*this).m_ContentMargins,
        );
    } else {
        (*this).m_hFallbackTheme = null_mut();
    }

    let mut content_margins: MARGINS = core::mem::zeroed();
    if GetThemeMargins(
        GetWindowTheme(GetParent((*this).m_hWnd)),
        null_mut(),
        TNP_BACKGROUND,
        0,
        TMT_CONTENTMARGINS,
        null(),
        &mut content_margins,
    ) == S_OK
    {
        (*this).m_inset.cx = (content_margins.cxRightWidth - 5).max(0);
        (*this).m_inset.cy = (content_margins.cyBottomHeight - 5).max(0);
    } else {
        (*this).m_inset.cx = 2;
        (*this).m_inset.cy = 2;
    }

    (*this).m_drawWithDedicatedBackground = FALSE as BOOL;
    if IsThemeActive() != 0 {
        (*this).m_hTheme = OpenThemeData((*this).m_hWnd, w!("Button").as_ptr());
        if !(*this).m_hTheme.is_null() {
            (*this).m_drawWithDedicatedBackground =
                (IsThemePartDefined((*this).m_hTheme, BP_PUSHBUTTON, 0) == 0) as BOOL;
        }
    }
}

unsafe fn tsdb_on_paint(
    this: *mut CTrayShowDesktopButton,
    _u_msg: UINT,
    _w_param: WPARAM,
    _l_param: LPARAM,
    _b_handled: *mut BOOL,
) -> LRESULT {
    let mut rc: RECT = core::mem::zeroed();
    GetClientRect((*this).m_hWnd, &mut rc);

    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    let hdc = BeginPaint((*this).m_hWnd, &mut ps);
    tsdb_on_draw(this, hdc, &mut rc);
    EndPaint((*this).m_hWnd, &ps);

    0
}

unsafe fn tsdb_on_print_client(
    this: *mut CTrayShowDesktopButton,
    _u_msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
    _b_handled: *mut BOOL,
) -> LRESULT {
    if (l_param & PRF_CHECKVISIBLE as LPARAM) != 0 && IsWindowVisible((*this).m_hWnd) == 0 {
        return 0;
    }

    let mut rc: RECT = core::mem::zeroed();
    GetClientRect((*this).m_hWnd, &mut rc);

    let hdc = w_param as HDC;
    tsdb_on_draw(this, hdc, &mut rc);

    0
}

/// `BOOL CTrayShowDesktopButton::PtInButton(LPPOINT ppt) const`.
unsafe fn tsdb_pt_in_button(this: *const CTrayShowDesktopButton, ppt: *const POINT) -> BOOL {
    if ppt.is_null() || IsWindow((*this).m_hWnd) == 0 {
        return FALSE as BOOL;
    }

    let mut rc: RECT = core::mem::zeroed();
    GetWindowRect((*this).m_hWnd, &mut rc);
    let cx_edge = GetSystemMetrics(SM_CXEDGE as i32);
    let cy_edge = GetSystemMetrics(SM_CYEDGE as i32);
    InflateRect(&mut rc, cx_edge.max(1), cy_edge.max(1));

    let inside = if (*this).m_bHorizontal != 0 {
        (*ppt).x > rc.left
    } else {
        (*ppt).y > rc.top
    };
    inside as BOOL
}

/// `VOID CTrayShowDesktopButton::StartHovering()`.
unsafe fn tsdb_start_hovering(this: *mut CTrayShowDesktopButton) {
    if (*this).m_bHovering != 0 {
        return;
    }

    (*this).m_bHovering = TRUE as BOOL;
    InvalidateRect((*this).m_hWnd, null(), TRUE as BOOL);

    SetTimer(
        (*this).m_hWnd,
        SHOW_DESKTOP_TIMER_ID,
        SHOW_DESKTOP_TIMER_INTERVAL,
        None,
    );

    PostMessageW((*this).m_hWndTaskbar, WM_NCPAINT, 0, 0);
}

unsafe fn tsdb_on_mouse_move(
    this: *mut CTrayShowDesktopButton,
    _u_msg: UINT,
    _w_param: WPARAM,
    _l_param: LPARAM,
    _b_handled: *mut BOOL,
) -> LRESULT {
    tsdb_start_hovering(this);
    0
}

unsafe fn tsdb_on_timer(
    this: *mut CTrayShowDesktopButton,
    _u_msg: UINT,
    w_param: WPARAM,
    _l_param: LPARAM,
    _b_handled: *mut BOOL,
) -> LRESULT {
    if w_param as UINT_PTR != SHOW_DESKTOP_TIMER_ID || (*this).m_bHovering == 0 {
        return 0;
    }

    let mut pt: POINT = core::mem::zeroed();
    GetCursorPos(&mut pt);
    if tsdb_pt_in_button(this, &pt) == 0 {
        // The end of hovering?
        (*this).m_bHovering = FALSE as BOOL;
        KillTimer((*this).m_hWnd, SHOW_DESKTOP_TIMER_ID);
        InvalidateRect((*this).m_hWnd, null(), TRUE as BOOL);

        PostMessageW((*this).m_hWndTaskbar, WM_NCPAINT, 0, 0);
    }

    0
}

unsafe fn tsdb_on_destroy(
    this: *mut CTrayShowDesktopButton,
    _u_msg: UINT,
    _w_param: WPARAM,
    _l_param: LPARAM,
    _b_handled: *mut BOOL,
) -> LRESULT {
    if !(*this).m_hTheme.is_null() {
        CloseThemeData((*this).m_hTheme);
        (*this).m_hTheme = null_mut();
    }
    if !(*this).m_hFallbackTheme.is_null() {
        CloseThemeData((*this).m_hFallbackTheme);
        (*this).m_hFallbackTheme = null_mut();
    }
    0
}

/// `VOID CTrayShowDesktopButton::OnDraw(HDC hdc, LPRECT prc)`.
unsafe fn tsdb_on_draw(this: *mut CTrayShowDesktopButton, hdc: HDC, prc: *const RECT) {
    let mut rc: RECT = RECT {
        left: (*prc).left,
        top: (*prc).top,
        right: (*prc).right,
        bottom: (*prc).bottom,
    };
    let mut hbr_background: HBRUSH = null_mut();

    if !(*this).m_hTheme.is_null() {
        let theme: HTHEME;
        let mut part: i32 = 0;
        let mut state: i32 = 0;
        if (*this).m_drawWithDedicatedBackground != 0 {
            theme = (*this).m_hTheme;

            state = if (*this).m_bPressed != 0 {
                PBS_PRESSED
            } else if (*this).m_bHovering != 0 {
                PBS_HOT
            } else {
                PBS_NORMAL
            };
        } else {
            part = TP_BUTTON;
            theme = (*this).m_hFallbackTheme;

            state = if (*this).m_bPressed != 0 {
                TS_PRESSED
            } else if (*this).m_bHovering != 0 {
                TS_HOT
            } else {
                TS_NORMAL
            };

            if (*this).m_bHorizontal != 0 {
                rc.right -= (*this).m_inset.cx;
            } else {
                rc.bottom -= (*this).m_inset.cy;
            }
        }

        if IsThemeBackgroundPartiallyTransparent(theme, part, state) != 0 {
            DrawThemeParentBackground((*this).m_hWnd, hdc, null());
        }

        DrawThemeBackground(theme, hdc, part, state, &rc, &rc);
    } else {
        hbr_background = GetSysColorBrush(COLOR_3DFACE as i32);
        FillRect(hdc, &rc, hbr_background);

        if (*this).m_bPressed != 0 || (*this).m_bHovering != 0 {
            let edge = if (*this).m_bPressed != 0 {
                BDR_SUNKENOUTER
            } else {
                BDR_RAISEDINNER
            };
            DrawEdge(hdc, &mut rc, edge, BF_RECT);
        }
    }

    if (*this).m_highContrastMode != 0 || (*this).m_drawWithDedicatedBackground == 0 {
        // Prepare to draw icon

        // Determine X-position of icon's top-left corner
        let mut icon_x = rc.left;
        icon_x += (rc.right - icon_x) / 2;
        icon_x -= (*this).m_szIcon.cx / 2;

        // Determine Y-position of icon's top-left corner
        let mut icon_y = rc.top;
        icon_y += (rc.bottom - icon_y) / 2;
        icon_y -= (*this).m_szIcon.cy / 2;

        // Ok now actually draw the icon itself
        if !(*this).m_icon.is_null() {
            DrawIconEx(
                hdc,
                icon_x,
                icon_y,
                (*this).m_icon,
                0,
                0,
                0,
                hbr_background,
                DI_NORMAL,
            );
        }
    }
}

// ---- button WndProc (BEGIN_MSG_MAP) — arch-split callback ABI ----

unsafe fn tsdb_process(
    this: *mut CTrayShowDesktopButton,
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut b_handled: BOOL = TRUE as BOOL;
    let mut l_result: LRESULT = 0;

    match uMsg {
        WM_LBUTTONDOWN | WM_NCLBUTTONDOWN => {
            l_result = tsdb_on_lbutton_down(this, uMsg, wParam, lParam, &mut b_handled)
        }
        WM_LBUTTONUP | WM_NCLBUTTONUP => {
            l_result = tsdb_on_lbutton_up(this, uMsg, wParam, lParam, &mut b_handled)
        }
        WM_SETTINGCHANGE => {
            l_result = tsdb_on_setting_changed(this, uMsg, wParam, lParam, &mut b_handled)
        }
        WM_THEMECHANGED => {
            l_result = tsdb_on_theme_changed(this, uMsg, wParam, lParam, &mut b_handled)
        }
        WM_WINDOWPOSCHANGED => {
            l_result = tsdb_on_window_pos_changed(this, uMsg, wParam, lParam, &mut b_handled)
        }
        WM_PAINT => l_result = tsdb_on_paint(this, uMsg, wParam, lParam, &mut b_handled),
        WM_PRINTCLIENT => {
            l_result = tsdb_on_print_client(this, uMsg, wParam, lParam, &mut b_handled)
        }
        WM_TIMER => l_result = tsdb_on_timer(this, uMsg, wParam, lParam, &mut b_handled),
        WM_MOUSEMOVE => l_result = tsdb_on_mouse_move(this, uMsg, wParam, lParam, &mut b_handled),
        WM_DESTROY => l_result = tsdb_on_destroy(this, uMsg, wParam, lParam, &mut b_handled),
        x if x == TSDB_CLICK => {
            l_result = tsdb_on_click(this, uMsg, wParam, lParam, &mut b_handled)
        }
        _ => b_handled = FALSE as BOOL,
    }

    if b_handled != 0 {
        l_result
    } else {
        DefWindowProcW(hWnd, uMsg, wParam, lParam)
    }
}

#[no_mangle]
pub unsafe extern "system" fn CTrayShowDesktopButton_WindowProc(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    CTrayShowDesktopButton_window_proc_impl(hWnd, uMsg, wParam, lParam)
}

#[inline]
unsafe fn CTrayShowDesktopButton_window_proc_impl(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let this = if uMsg == WM_NCCREATE {
        let p = wndproc_nccreate_this(hWnd, lParam) as *mut CTrayShowDesktopButton;
        if !p.is_null() {
            (*p).m_hWnd = hWnd;
        }
        p
    } else {
        get_window_this(hWnd) as *mut CTrayShowDesktopButton
    };

    if this.is_null() {
        return DefWindowProcW(hWnd, uMsg, wParam, lParam);
    }

    let r = tsdb_process(this, hWnd, uMsg, wParam, lParam);

    if uMsg == WM_NCDESTROY {
        (*this).m_hWnd = null_mut();
    }
    r
}

/// `CWindowImpl::Create` for the button —
/// `DECLARE_WND_CLASS_EX(szTrayShowDesktopButton, CS_HREDRAW|CS_VREDRAW, COLOR_3DFACE)`.
unsafe fn tsdb_create(this: *mut CTrayShowDesktopButton, hWndParent: HWND, dwStyle: DWORD) -> HWND {
    register_class_once(
        hExplorerInstance,
        w!("TrayShowDesktopButtonWClass").as_ptr(),
        Some(CTrayShowDesktopButton_WindowProc),
        CS_HREDRAW | CS_VREDRAW,
        COLOR_3DFACE as i32,
    );

    (*this).m_hWnd = CreateWindowExW(
        0,
        w!("TrayShowDesktopButtonWClass").as_ptr(),
        null(),
        dwStyle,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        hWndParent,
        null_mut(),
        hExplorerInstance,
        this as LPVOID,
    );
    (*this).m_hWnd
}

// ---- the 7 extern "C" shims (replace traydeskbtn_shim.cpp) ----

#[no_mangle]
pub unsafe extern "C" fn TrayShowDesktopButton_PtInButton(
    p: *mut CTrayShowDesktopButton,
    ppt: *const POINT,
) -> BOOL {
    tsdb_pt_in_button(p, ppt)
}

#[no_mangle]
pub unsafe extern "C" fn TrayShowDesktopButton_OnLButtonDown(
    p: *mut CTrayShowDesktopButton,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    pbHandled: *mut BOOL,
) -> LRESULT {
    tsdb_on_lbutton_down(p, uMsg, wParam, lParam, pbHandled)
}

#[no_mangle]
pub unsafe extern "C" fn TrayShowDesktopButton_OnLButtonUp(
    p: *mut CTrayShowDesktopButton,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    pbHandled: *mut BOOL,
) -> LRESULT {
    tsdb_on_lbutton_up(p, uMsg, wParam, lParam, pbHandled)
}

#[no_mangle]
pub unsafe extern "C" fn TrayShowDesktopButton_Click(p: *mut CTrayShowDesktopButton) {
    tsdb_click(p);
}

#[no_mangle]
pub unsafe extern "C" fn TrayShowDesktopButton_GetHWND(p: *mut CTrayShowDesktopButton) -> HWND {
    (*p).m_hWnd
}

#[no_mangle]
pub unsafe extern "C" fn TrayShowDesktopButton_IsWindow(p: *mut CTrayShowDesktopButton) -> BOOL {
    IsWindow((*p).m_hWnd)
}

#[no_mangle]
pub unsafe extern "C" fn TrayShowDesktopButton_GetPressed(p: *mut CTrayShowDesktopButton) -> BOOL {
    (*p).m_bPressed
}

// ###########################################################################
// CTrayNotifyWnd (trayntfy.cpp) — CComCoClass + CComObjectRootEx + CWindowImpl
// + IOleWindow. Flat lpVtbl-first object; hosts the clock, pager, and the
// embedded CTrayShowDesktopButton.
// ###########################################################################

#[repr(C)]
pub struct CTrayNotifyWnd {
    // COM identity — IOleWindow vtbl first (IUnknown shares it).
    lpVtbl: *const IOleWindowVtbl,
    ref_: LONG,
    m_hWnd: HWND,
    // --- CTrayNotifyWnd members (declaration order) ---
    m_clock: *mut IUnknown,                      // CComPtr<IUnknown>
    m_ShowDesktopButton: CTrayShowDesktopButton, // embedded BY VALUE
    m_pager: *mut IUnknown,                      // CComPtr<IUnknown>
    m_hwndClock: HWND,
    m_hwndShowDesktop: HWND,
    m_hwndPager: HWND,
    TrayTheme: HTHEME,
    trayClockMinSize: SIZE,
    trayShowDesktopSize: SIZE,
    trayNotifySize: SIZE,
    ContentMargin: MARGINS,
    IsHorizontal: BOOL,
}

// ---- construction / destruction ----

unsafe fn CTrayNotifyWnd_ctor(this: *mut CTrayNotifyWnd) {
    // Zero the whole object (matches the C++ member ZeroMemory of the SIZE/
    // MARGINS members + NULL/FALSE inits), then set lpVtbl and run the embedded
    // button's ctor.
    core::ptr::write_bytes(this as *mut u8, 0, core::mem::size_of::<CTrayNotifyWnd>());
    (*this).lpVtbl = &vt_CTrayNotifyWnd;
    (*this).ref_ = 0;
    tsdb_ctor(core::ptr::addr_of_mut!((*this).m_ShowDesktopButton));
}

unsafe fn CTrayNotifyWnd_new() -> *mut CTrayNotifyWnd {
    let p = HeapAlloc(
        hProcessHeap,
        0,
        core::mem::size_of::<CTrayNotifyWnd>() as SIZE_T,
    ) as *mut CTrayNotifyWnd;
    if !p.is_null() {
        CTrayNotifyWnd_ctor(p);
    }
    p
}

unsafe fn CTrayNotifyWnd_delete(this: *mut CTrayNotifyWnd) {
    if !this.is_null() {
        // ~CTrayNotifyWnd() is empty; the child windows are owned by USER (the
        // parent destroys them via WM_(NC)DESTROY). The CComPtr children would
        // Release here; process-lifetime object, so we leave them (matches the
        // trayclock/syspager templates).
        HeapFree(hProcessHeap, 0, this as *mut c_void);
    }
}

// ---- COM: IUnknown / IOleWindow (BEGIN_COM_MAP + CComObject) ----

#[no_mangle]
pub unsafe extern "system" fn CTrayNotifyWnd_QueryInterface(
    iface: *mut IOleWindow,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let this = iface as *mut CTrayNotifyWnd;
    if ppv.is_null() {
        return E_POINTER;
    }
    if guid_eq(riid, &IID_IUnknown) || guid_eq(riid, &IID_IOleWindow) {
        *ppv = this as *mut c_void;
        CTrayNotifyWnd_AddRef(iface);
        return S_OK;
    }
    *ppv = null_mut();
    E_NOINTERFACE
}

#[no_mangle]
pub unsafe extern "system" fn CTrayNotifyWnd_AddRef(iface: *mut IOleWindow) -> u32 {
    let this = iface as *mut CTrayNotifyWnd;
    interlocked_increment(&mut (*this).ref_) as u32
}

#[no_mangle]
pub unsafe extern "system" fn CTrayNotifyWnd_Release(iface: *mut IOleWindow) -> u32 {
    let this = iface as *mut CTrayNotifyWnd;
    let n = interlocked_decrement(&mut (*this).ref_);
    if n == 0 {
        CTrayNotifyWnd_delete(this);
        return 0;
    }
    n as u32
}

/// IOleWindow::GetWindow (inline in trayntfy.cpp).
#[no_mangle]
pub unsafe extern "system" fn CTrayNotifyWnd_GetWindow(
    iface: *mut IOleWindow,
    phwnd: *mut HWND,
) -> HRESULT {
    let this = iface as *mut CTrayNotifyWnd;
    if phwnd.is_null() {
        return E_INVALIDARG;
    }
    *phwnd = (*this).m_hWnd;
    S_OK
}

/// IOleWindow::ContextSensitiveHelp (inline).
#[no_mangle]
pub unsafe extern "system" fn CTrayNotifyWnd_ContextSensitiveHelp(
    _iface: *mut IOleWindow,
    _f_enter_mode: BOOL,
) -> HRESULT {
    E_NOTIMPL
}

static vt_CTrayNotifyWnd: IOleWindowVtbl = IOleWindowVtbl {
    QueryInterface: CTrayNotifyWnd_QueryInterface,
    AddRef: CTrayNotifyWnd_AddRef,
    Release: CTrayNotifyWnd_Release,
    GetWindow: CTrayNotifyWnd_GetWindow,
    ContextSensitiveHelp: CTrayNotifyWnd_ContextSensitiveHelp,
};

// ---- the class methods (formerly CTrayNotifyWnd::*) ----

/// `LRESULT OnThemeChanged()` — the parameterless form.
unsafe fn tnw_on_theme_changed(this: *mut CTrayNotifyWnd) -> LRESULT {
    if !(*this).TrayTheme.is_null() {
        CloseThemeData((*this).TrayTheme);
    }

    (*this).TrayTheme = if IsThemeActive() != 0 {
        OpenThemeData((*this).m_hWnd, w!("TrayNotify").as_ptr())
    } else {
        null_mut()
    };

    if !(*this).TrayTheme.is_null() {
        SetWindowExStyle((*this).m_hWnd, WS_EX_STATICEDGE as LONG, 0);

        GetThemeMargins(
            (*this).TrayTheme,
            null_mut(),
            TNP_BACKGROUND,
            0,
            TMT_CONTENTMARGINS,
            null(),
            &mut (*this).ContentMargin,
        );
    } else {
        SetWindowExStyle(
            (*this).m_hWnd,
            WS_EX_STATICEDGE as LONG,
            WS_EX_STATICEDGE as LONG,
        );

        (*this).ContentMargin.cxLeftWidth = 2;
        (*this).ContentMargin.cxRightWidth = 2;
        (*this).ContentMargin.cyTopHeight = 2;
        (*this).ContentMargin.cyBottomHeight = 2;
    }

    TRUE as LRESULT
}

unsafe fn tnw_on_theme_changed_msg(
    this: *mut CTrayNotifyWnd,
    _u_msg: UINT,
    _w_param: WPARAM,
    _l_param: LPARAM,
    _b_handled: *mut BOOL,
) -> LRESULT {
    tnw_on_theme_changed(this)
}

unsafe fn tnw_on_create(
    this: *mut CTrayNotifyWnd,
    _u_msg: UINT,
    _w_param: WPARAM,
    _l_param: LPARAM,
    _b_handled: *mut BOOL,
) -> LRESULT {
    let mut hr = CTrayClockWnd_CreateInstance(
        (*this).m_hWnd,
        &IID_IUnknown as *const IID,
        &mut (*this).m_clock as *mut *mut IUnknown as *mut *mut c_void,
    );
    if hr < 0 {
        return FALSE as LRESULT;
    }

    hr = IUnknown_GetWindow((*this).m_clock, &mut (*this).m_hwndClock);
    if hr < 0 {
        return FALSE as LRESULT;
    }

    hr = CSysPagerWnd_CreateInstance(
        (*this).m_hWnd,
        &IID_IUnknown as *const IID,
        &mut (*this).m_pager as *mut *mut IUnknown as *mut *mut c_void,
    );
    if hr < 0 {
        return FALSE as LRESULT;
    }

    hr = IUnknown_GetWindow((*this).m_pager, &mut (*this).m_hwndPager);
    if hr < 0 {
        return FALSE as LRESULT;
    }

    // Create the 'Show Desktop' button
    tsdb_do_create(
        core::ptr::addr_of_mut!((*this).m_ShowDesktopButton),
        (*this).m_hWnd,
    );
    (*this).m_hwndShowDesktop = (*this).m_ShowDesktopButton.m_hWnd;

    TRUE as LRESULT
}

/// `BOOL GetMinimumSize(IN OUT PSIZE pSize)`.
unsafe fn tnw_get_minimum_size(this: *mut CTrayNotifyWnd, p_size: *mut SIZE) -> BOOL {
    let mut clock_size = SIZE { cx: 0, cy: 0 };
    let mut tray_size = SIZE { cx: 0, cy: 0 };
    let mut show_desktop_size = SIZE { cx: 0, cy: 0 };
    let b_hide_clock = GetHideClock();

    if b_hide_clock == 0 {
        // The `goto NoClock` short-circuits the SendMessage; in every branch
        // trayClockMinSize ends up = clockSize (as modified so far).
        let mut got_clock = true;
        if (*this).IsHorizontal != 0 {
            clock_size.cy = (*p_size).cy;
            if clock_size.cy <= 0 {
                got_clock = false;
            }
        } else {
            clock_size.cx = (*p_size).cx;
            if clock_size.cx <= 0 {
                got_clock = false;
            }
        }

        if got_clock {
            SendMessageW(
                (*this).m_hwndClock,
                TNWM_GETMINIMUMSIZE,
                (*this).IsHorizontal as WPARAM,
                &mut clock_size as *mut SIZE as LPARAM,
            );
        }
    }
    (*this).trayClockMinSize = clock_size;

    if (*this).IsHorizontal != 0 {
        tray_size.cy = (*p_size).cy - 2 * TRAY_NOTIFY_WND_SPACING_Y;
    } else {
        tray_size.cx = (*p_size).cx - 2 * TRAY_NOTIFY_WND_SPACING_X;
    }

    SendMessageW(
        (*this).m_hwndPager,
        TNWM_GETMINIMUMSIZE,
        (*this).IsHorizontal as WPARAM,
        &mut tray_size as *mut SIZE as LPARAM,
    );

    (*this).trayNotifySize = tray_size;

    let mut show_desktop_button_extent: i32 = 0;
    if g_TaskbarSettings.bShowDesktopButton != 0 {
        show_desktop_button_extent =
            tsdb_width_or_height(core::ptr::addr_of!((*this).m_ShowDesktopButton));
        if (*this).IsHorizontal != 0 {
            show_desktop_size.cx = show_desktop_button_extent;
            show_desktop_size.cy = (*p_size).cy;
        } else {
            show_desktop_size.cx = (*p_size).cx;
            show_desktop_size.cy = show_desktop_button_extent;
        }
    }
    (*this).trayShowDesktopSize = show_desktop_size;

    if (*this).IsHorizontal != 0 {
        (*p_size).cx = 2 * TRAY_NOTIFY_WND_SPACING_X;

        if b_hide_clock == 0 {
            (*p_size).cx += TRAY_NOTIFY_WND_SPACING_X + (*this).trayClockMinSize.cx;
        }

        if g_TaskbarSettings.bShowDesktopButton != 0 {
            (*p_size).cx += show_desktop_button_extent;
        }

        (*p_size).cx += tray_size.cx;
        (*p_size).cx += (*this).ContentMargin.cxLeftWidth + (*this).ContentMargin.cxRightWidth;
    } else {
        (*p_size).cy = 2 * TRAY_NOTIFY_WND_SPACING_Y;

        if b_hide_clock == 0 {
            (*p_size).cy += TRAY_NOTIFY_WND_SPACING_Y + (*this).trayClockMinSize.cy;
        }

        if g_TaskbarSettings.bShowDesktopButton != 0 {
            (*p_size).cy += show_desktop_button_extent;
        }

        (*p_size).cy += tray_size.cy;
        (*p_size).cy += (*this).ContentMargin.cyTopHeight + (*this).ContentMargin.cyBottomHeight;
    }

    TRUE as BOOL
}

/// `VOID Size(IN OUT SIZE *pszClient)`.
unsafe fn tnw_size(this: *mut CTrayNotifyWnd, psz_client: *mut SIZE) {
    let mut rc_client = RECT {
        left: 0,
        top: 0,
        right: (*psz_client).cx,
        bottom: (*psz_client).cy,
    };
    tnw_align_controls(this, &mut rc_client);
    (*psz_client).cx = rc_client.right - rc_client.left;
    (*psz_client).cy = rc_client.bottom - rc_client.top;
}

/// `VOID AlignControls(IN CONST PRECT prcClient OPTIONAL)`. `prc_client` may be
/// null (→ use GetClientRect); otherwise it is in/out.
unsafe fn tnw_align_controls(this: *mut CTrayNotifyWnd, prc_client: *mut RECT) {
    let mut rc_client: RECT = core::mem::zeroed();
    if !prc_client.is_null() {
        rc_client = *prc_client;
    } else {
        GetClientRect((*this).m_hWnd, &mut rc_client);
    }

    rc_client.left += (*this).ContentMargin.cxLeftWidth;
    rc_client.top += (*this).ContentMargin.cyTopHeight;
    rc_client.right -= (*this).ContentMargin.cxRightWidth;
    rc_client.bottom -= (*this).ContentMargin.cyBottomHeight;

    let swp_flags: UINT = SWP_DRAWFRAME | SWP_NOCOPYBITS | SWP_NOZORDER;

    if g_TaskbarSettings.bShowDesktopButton != 0 {
        let mut pt_show_desktop = POINT {
            x: rc_client.left,
            y: rc_client.top,
        };
        let mut show_desktop_size = SIZE {
            cx: rc_client.right - rc_client.left,
            cy: rc_client.bottom - rc_client.top,
        };

        let cxy_show_desktop =
            tsdb_width_or_height(core::ptr::addr_of!((*this).m_ShowDesktopButton));
        if (*this).IsHorizontal != 0 {
            if (*this).TrayTheme.is_null() {
                pt_show_desktop.y -= (*this).ContentMargin.cyTopHeight;
                show_desktop_size.cy +=
                    (*this).ContentMargin.cyTopHeight + (*this).ContentMargin.cyBottomHeight;
            }

            rc_client.right -= cxy_show_desktop - (*this).ContentMargin.cxRightWidth;

            pt_show_desktop.x = rc_client.right;
            show_desktop_size.cx = cxy_show_desktop;

            // HACK: Clock has layout problems - remove this once addressed.
            rc_client.right -= CLOCK_TEXT_HACK;
        } else {
            if (*this).TrayTheme.is_null() {
                pt_show_desktop.x -= (*this).ContentMargin.cxLeftWidth;
                show_desktop_size.cx +=
                    (*this).ContentMargin.cxLeftWidth + (*this).ContentMargin.cxRightWidth;
            }

            rc_client.bottom -= cxy_show_desktop - (*this).ContentMargin.cyBottomHeight;

            pt_show_desktop.y = rc_client.bottom;
            show_desktop_size.cy = cxy_show_desktop;

            // HACK: Clock has layout problems - remove this once addressed.
            rc_client.bottom -= CLOCK_TEXT_HACK;
        }

        // Resize and reposition the button
        SetWindowPos(
            (*this).m_hwndShowDesktop,
            null_mut(),
            pt_show_desktop.x,
            pt_show_desktop.y,
            show_desktop_size.cx,
            show_desktop_size.cy,
            swp_flags,
        );
    }

    if GetHideClock() == 0 {
        let mut pt_clock = POINT {
            x: rc_client.left,
            y: rc_client.top,
        };
        let mut clock_size = SIZE {
            cx: rc_client.right - rc_client.left,
            cy: rc_client.bottom - rc_client.top,
        };

        if (*this).IsHorizontal != 0 {
            rc_client.right -= (*this).trayClockMinSize.cx;

            pt_clock.x = rc_client.right;
            clock_size.cx = (*this).trayClockMinSize.cx;
        } else {
            rc_client.bottom -= (*this).trayClockMinSize.cy;

            pt_clock.y = rc_client.bottom;
            clock_size.cy = (*this).trayClockMinSize.cy;
        }

        SetWindowPos(
            (*this).m_hwndClock,
            null_mut(),
            pt_clock.x,
            pt_clock.y,
            clock_size.cx,
            clock_size.cy,
            swp_flags,
        );
    }

    let pt_pager: POINT;
    if (*this).IsHorizontal != 0 {
        let mut y = ((rc_client.bottom - rc_client.top) - (*this).trayNotifySize.cy) / 2;
        if (*&raw const g_TaskbarSettings).UseCompactTrayIcons() != 0 {
            y += (*this).ContentMargin.cyTopHeight;
        }
        pt_pager = POINT {
            x: (*this).ContentMargin.cxLeftWidth,
            y,
        };
    } else {
        let mut x = ((rc_client.right - rc_client.left) - (*this).trayNotifySize.cx) / 2;
        if (*&raw const g_TaskbarSettings).UseCompactTrayIcons() != 0 {
            x += (*this).ContentMargin.cxLeftWidth;
        }
        pt_pager = POINT {
            x,
            y: (*this).ContentMargin.cyTopHeight,
        };
    }

    SetWindowPos(
        (*this).m_hwndPager,
        null_mut(),
        pt_pager.x,
        pt_pager.y,
        (*this).trayNotifySize.cx,
        (*this).trayNotifySize.cy,
        swp_flags,
    );

    if !prc_client.is_null() {
        (*prc_client).left = rc_client.left - (*this).ContentMargin.cxLeftWidth;
        (*prc_client).top = rc_client.top - (*this).ContentMargin.cyTopHeight;
        (*prc_client).right = rc_client.right + (*this).ContentMargin.cxRightWidth;
        (*prc_client).bottom = rc_client.bottom + (*this).ContentMargin.cyBottomHeight;
    }
}

unsafe fn tnw_on_erase_background(
    this: *mut CTrayNotifyWnd,
    _u_msg: UINT,
    w_param: WPARAM,
    _l_param: LPARAM,
    b_handled: *mut BOOL,
) -> LRESULT {
    let hdc = w_param as HDC;

    if (*this).TrayTheme.is_null() {
        *b_handled = FALSE as BOOL;
        return 0;
    }

    let mut rect: RECT = core::mem::zeroed();
    GetClientRect((*this).m_hWnd, &mut rect);
    if IsThemeBackgroundPartiallyTransparent((*this).TrayTheme, TNP_BACKGROUND, 0) != 0 {
        DrawThemeParentBackground((*this).m_hWnd, hdc, &rect);
    }

    DrawThemeBackground((*this).TrayTheme, hdc, TNP_BACKGROUND, 0, &rect, null());

    TRUE as LRESULT
}

unsafe fn tnw_on_get_minimum_size(
    this: *mut CTrayNotifyWnd,
    _u_msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
    _b_handled: *mut BOOL,
) -> LRESULT {
    let horizontal = w_param as BOOL;

    if horizontal != (*this).IsHorizontal {
        (*this).IsHorizontal = horizontal;
    }

    SetWindowTheme(
        (*this).m_hWnd,
        if (*this).IsHorizontal != 0 {
            w!("TrayNotifyHoriz").as_ptr()
        } else {
            w!("TrayNotifyVert").as_ptr()
        },
        null(),
    );
    (*this).m_ShowDesktopButton.m_bHorizontal = horizontal;

    tnw_get_minimum_size(this, l_param as *mut SIZE) as LRESULT
}

unsafe fn tnw_on_get_show_desktop_button(
    this: *mut CTrayNotifyWnd,
    _u_msg: UINT,
    w_param: WPARAM,
    _l_param: LPARAM,
    b_handled: *mut BOOL,
) -> LRESULT {
    if w_param == 0 {
        return 0;
    }

    let ptr = w_param as *mut *mut CTrayShowDesktopButton;
    if (*this).m_ShowDesktopButton.m_hWnd.is_null() {
        *ptr = null_mut();
        return 0;
    }

    *ptr = core::ptr::addr_of_mut!((*this).m_ShowDesktopButton);
    *b_handled = TRUE as BOOL;
    0
}

unsafe fn tnw_on_size(
    this: *mut CTrayNotifyWnd,
    _u_msg: UINT,
    _w_param: WPARAM,
    l_param: LPARAM,
    _b_handled: *mut BOOL,
) -> LRESULT {
    let mut client_size = SIZE {
        cx: (l_param & 0xFFFF) as u16 as i32,
        cy: ((l_param >> 16) & 0xFFFF) as u16 as i32,
    };

    tnw_size(this, &mut client_size);

    TRUE as LRESULT
}

unsafe fn tnw_on_nc_hit_test(
    this: *mut CTrayNotifyWnd,
    _u_msg: UINT,
    _w_param: WPARAM,
    l_param: LPARAM,
    _b_handled: *mut BOOL,
) -> LRESULT {
    let pt = POINT {
        x: (l_param & 0xFFFF) as i16 as i32,
        y: ((l_param >> 16) & 0xFFFF) as i16 as i32,
    };

    let btn = core::ptr::addr_of!((*this).m_ShowDesktopButton);
    if !(*btn).m_hWnd.is_null() && tsdb_pt_in_button(btn, &pt) != 0 {
        return HTCLIENT as LRESULT;
    }

    HTTRANSPARENT as LRESULT
}

unsafe fn tnw_on_mouse_move(
    this: *mut CTrayNotifyWnd,
    _u_msg: UINT,
    _w_param: WPARAM,
    _l_param: LPARAM,
    _b_handled: *mut BOOL,
) -> LRESULT {
    let mut pt: POINT = core::mem::zeroed();
    GetCursorPos(&mut pt);

    let btn = core::ptr::addr_of_mut!((*this).m_ShowDesktopButton);
    if !(*btn).m_hWnd.is_null() && tsdb_pt_in_button(btn, &pt) != 0 {
        tsdb_start_hovering(btn);
    }

    TRUE as LRESULT
}

unsafe fn tnw_on_ctx_menu(
    this: *mut CTrayNotifyWnd,
    u_msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
    b_handled: *mut BOOL,
) -> LRESULT {
    *b_handled = TRUE as BOOL;

    if w_param as HWND == (*this).m_hwndClock {
        SendMessageW(GetParent((*this).m_hWnd), u_msg, w_param, l_param)
    } else {
        0
    }
}

unsafe fn tnw_on_clock_message(
    this: *mut CTrayNotifyWnd,
    u_msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
    _b_handled: *mut BOOL,
) -> LRESULT {
    SendMessageW((*this).m_hwndClock, u_msg, w_param, l_param)
}

unsafe fn tnw_on_taskbar_settings_changed(
    this: *mut CTrayNotifyWnd,
    u_msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
    b_handled: *mut BOOL,
) -> LRESULT {
    let new_settings = l_param as *const TaskbarSettings;

    // Toggle show desktop button
    if (*new_settings).bShowDesktopButton != g_TaskbarSettings.bShowDesktopButton {
        g_TaskbarSettings.bShowDesktopButton = (*new_settings).bShowDesktopButton;
        ShowWindow(
            (*this).m_hwndShowDesktop,
            if g_TaskbarSettings.bShowDesktopButton != 0 {
                SW_SHOW as i32
            } else {
                SW_HIDE as i32
            },
        );

        // Ask the parent to resize
        let mut nmh: NMHDR = core::mem::zeroed();
        nmh.hwndFrom = (*this).m_hWnd;
        nmh.idFrom = 0;
        nmh.code = NTNWM_REALIGN;
        SendMessageW((*this).m_hWnd, WM_NOTIFY, 0, &nmh as *const NMHDR as LPARAM);
    }

    g_TaskbarSettings.bHideInactiveIcons = (*new_settings).bHideInactiveIcons;

    tnw_on_clock_message(this, u_msg, w_param, l_param, b_handled)
}

unsafe fn tnw_on_pager_message(
    this: *mut CTrayNotifyWnd,
    u_msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
    _b_handled: *mut BOOL,
) -> LRESULT {
    SendMessageW((*this).m_hwndPager, u_msg, w_param, l_param)
}

/// `LRESULT OnRealign(INT uCode, LPNMHDR hdr, BOOL& bHandled)` — the
/// NOTIFY_CODE_HANDLER(NTNWM_REALIGN) body.
unsafe fn tnw_on_realign(this: *mut CTrayNotifyWnd, hdr: *mut NMHDR) -> LRESULT {
    (*hdr).hwndFrom = (*this).m_hWnd;
    SendMessageW(GetParent((*this).m_hWnd), WM_NOTIFY, 0, hdr as LPARAM)
}

// ---- the WndProc (BEGIN_MSG_MAP) — arch-split callback ABI ----

unsafe fn CTrayNotifyWnd_process(
    this: *mut CTrayNotifyWnd,
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut b_handled: BOOL = TRUE as BOOL;
    let mut l_result: LRESULT = 0;

    match uMsg {
        WM_CREATE => l_result = tnw_on_create(this, uMsg, wParam, lParam, &mut b_handled),
        WM_THEMECHANGED => {
            l_result = tnw_on_theme_changed_msg(this, uMsg, wParam, lParam, &mut b_handled)
        }
        WM_ERASEBKGND => {
            l_result = tnw_on_erase_background(this, uMsg, wParam, lParam, &mut b_handled)
        }
        WM_SIZE => l_result = tnw_on_size(this, uMsg, wParam, lParam, &mut b_handled),
        WM_NCHITTEST => l_result = tnw_on_nc_hit_test(this, uMsg, wParam, lParam, &mut b_handled),
        WM_MOUSEMOVE | WM_NCMOUSEMOVE => {
            l_result = tnw_on_mouse_move(this, uMsg, wParam, lParam, &mut b_handled)
        }
        WM_CONTEXTMENU => l_result = tnw_on_ctx_menu(this, uMsg, wParam, lParam, &mut b_handled),
        WM_NCLBUTTONDBLCLK | WM_SETFONT => {
            l_result = tnw_on_clock_message(this, uMsg, wParam, lParam, &mut b_handled)
        }
        WM_SETTINGCHANGE | WM_COPYDATA => {
            l_result = tnw_on_pager_message(this, uMsg, wParam, lParam, &mut b_handled)
        }
        x if x == TWM_SETTINGSCHANGED => {
            l_result = tnw_on_taskbar_settings_changed(this, uMsg, wParam, lParam, &mut b_handled)
        }
        x if x == TNWM_GETMINIMUMSIZE => {
            l_result = tnw_on_get_minimum_size(this, uMsg, wParam, lParam, &mut b_handled)
        }
        x if x == TNWM_GETSHOWDESKTOPBUTTON => {
            l_result = tnw_on_get_show_desktop_button(this, uMsg, wParam, lParam, &mut b_handled)
        }
        WM_NOTIFY => {
            let hdr = lParam as *mut NMHDR;
            if !hdr.is_null() && (*hdr).code == NTNWM_REALIGN {
                l_result = tnw_on_realign(this, hdr);
            } else {
                b_handled = FALSE as BOOL;
            }
        }
        _ => b_handled = FALSE as BOOL,
    }

    if b_handled != 0 {
        l_result
    } else {
        DefWindowProcW(hWnd, uMsg, wParam, lParam)
    }
}

#[no_mangle]
pub unsafe extern "system" fn CTrayNotifyWnd_WindowProc(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    CTrayNotifyWnd_window_proc_impl(hWnd, uMsg, wParam, lParam)
}

#[inline]
unsafe fn CTrayNotifyWnd_window_proc_impl(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let this = if uMsg == WM_NCCREATE {
        let p = wndproc_nccreate_this(hWnd, lParam) as *mut CTrayNotifyWnd;
        if !p.is_null() {
            (*p).m_hWnd = hWnd;
        }
        p
    } else {
        get_window_this(hWnd) as *mut CTrayNotifyWnd
    };

    if this.is_null() {
        return DefWindowProcW(hWnd, uMsg, wParam, lParam);
    }

    let r = CTrayNotifyWnd_process(this, hWnd, uMsg, wParam, lParam);

    if uMsg == WM_NCDESTROY {
        (*this).m_hWnd = null_mut();
    }
    r
}

// ---- CWindowImpl::Create + Initialize + factory ----

/// `DECLARE_WND_CLASS_EX(szTrayNotifyWndClass, CS_DBLCLKS, COLOR_3DFACE)` +
/// `Create(hwndParent, 0, NULL, dwStyle, dwExStyle)`.
unsafe fn CTrayNotifyWnd_Create(
    this: *mut CTrayNotifyWnd,
    hWndParent: HWND,
    dwStyle: DWORD,
    dwExStyle: DWORD,
) -> HWND {
    register_class_once(
        hExplorerInstance,
        w!("TrayNotifyWnd").as_ptr(),
        Some(CTrayNotifyWnd_WindowProc),
        CS_DBLCLKS,
        COLOR_3DFACE as i32,
    );

    (*this).m_hWnd = CreateWindowExW(
        dwExStyle,
        w!("TrayNotifyWnd").as_ptr(),
        null(),
        dwStyle,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        hWndParent,
        null_mut(),
        hExplorerInstance,
        this as LPVOID,
    );
    (*this).m_hWnd
}

/// `HRESULT Initialize(IN HWND hwndParent)`.
unsafe fn CTrayNotifyWnd_Initialize(this: *mut CTrayNotifyWnd, hwnd_parent: HWND) -> HRESULT {
    let dw_style: DWORD = WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS | WS_CLIPCHILDREN;
    CTrayNotifyWnd_Create(this, hwnd_parent, dw_style, WS_EX_STATICEDGE);
    if (*this).m_hWnd.is_null() {
        E_FAIL
    } else {
        S_OK
    }
}

#[no_mangle]
pub unsafe extern "C" fn CTrayNotifyWnd_CreateInstance(
    hwndParent: HWND,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    // ShellObjectCreatorInit<CTrayNotifyWnd>(hwndParent, riid, ppv).
    let pobj = CTrayNotifyWnd_new();
    if pobj.is_null() {
        return E_OUTOFMEMORY;
    }
    let iface = pobj as *mut IOleWindow;

    CTrayNotifyWnd_AddRef(iface);

    let mut hr = CTrayNotifyWnd_Initialize(pobj, hwndParent);
    if hr >= 0 {
        hr = CTrayNotifyWnd_QueryInterface(iface, riid, ppv);
    }

    CTrayNotifyWnd_Release(iface);
    hr
}
