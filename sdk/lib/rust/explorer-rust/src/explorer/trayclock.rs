//! trayclock.cpp — `CTrayClockWnd`, the taskbar clock. The FIRST tier-C
//! COM/window-server class translated to Rust, and the TEMPLATE for the rest
//! (trayntfy, syspager, taskswnd, traywnd, …). It exercises the whole
//! `explorer_sys::server` ATL/WTL-replacement substrate end to end:
//!
//!   * `CComObjectRootEx + CComCoClass + IOleWindow` → a flat `#[repr(C)]`
//!     `lpVtbl`-first object (ks-rust COM shape): `lpVtbl` / `ref_` / `m_hWnd`
//!     then the C++ members. Only ever touched via its IOleWindow* (from the
//!     factory) or its own WndProc (via GWLP_USERDATA), so the layout is free.
//!   * `BEGIN_COM_MAP` → `QueryInterface` walks {IUnknown, IOleWindow} → the
//!     one vtable pointer (AtlInternalQueryInterface: IUnknown == first entry).
//!   * `CWindowImpl + DECLARE_WND_CLASS_EX` → `Create` = `register_class_once`
//!     + `CreateWindowExW(this)`; `BEGIN_MSG_MAP` → the flat `match uMsg` in the
//!     ARCH-SPLIT WndProc (`#[cfg] extern "C"/"stdcall"`, like desktop.rs).
//!   * `ShellObjectCreatorInit<CTrayClockWnd>` → the `*_CreateInstance` factory:
//!     new(ref=0) → AddRef → Initialize(hWndParent) → QI(riid) → Release.
//!   * `CTooltips m_tooltip` → its HWND + TTM_* SendMessage wrappers.

use core::ffi::c_void;
use explorer_sys::com::*;
use explorer_sys::globals::*;
use explorer_sys::server::{
    get_window_this, interlocked_decrement, interlocked_increment, register_class_once,
    wndproc_nccreate_this, CloseThemeData, DrawThemeParentBackground, GetThemeColor, GetThemeFont,
    IsAppThemed, OpenThemeData, SetWindowTheme, ShellExecuteW, CLP_TIME, CW_USEDEFAULT, HTHEME,
    LOCALE_USER_DEFAULT, TMT_FONT, TMT_TEXTCOLOR, TOOLINFOW, TOOLTIPS_CLASSW, TTF_IDISHWND,
    TTF_SUBCLASS, TTM_ADDTOOLW, TTM_UPDATETIPTEXTW, TTS_ALWAYSTIP, TTS_NOPREFIX, TTTOOLINFOW_V1_SIZE,
};
use crate::explorer::settings::{g_TaskbarSettings, GetHideClock};
use explorer_sys::types::TaskbarSettings;
use winsrv_sys::w;
use winsrv_sys::winsrv::*;

// ---- file-scope constants (trayclock.cpp) ----

const CLOCKWND_FORMAT_COUNT: usize = 3;
const CLOCKWND_FORMAT_TIME: usize = 0;
const CLOCKWND_FORMAT_DAY: usize = 1;
const CLOCKWND_FORMAT_DATE: usize = 2;

const ID_TRAYCLOCK_TIMER: UINT_PTR = 0;
const ID_TRAYCLOCK_TIMER_INIT: UINT_PTR = 1;

const TRAY_CLOCK_WND_SPACING_X: LONG = 5;
const TRAY_CLOCK_WND_SPACING_Y: LONG = 0;

// precomp.h tray/notify window messages
const TWM_SETTINGSCHANGED: UINT = WM_USER + 300;
const TNWM_GETMINIMUMSIZE: UINT = WM_USER + 0x100;
const NTNWM_REALIGN: LPARAM = 0x1;

// dwFlags bitfield (union { DWORD dwFlags; struct{ 4×:1 } }) — bit assignment
// is internal-only (nothing external reads this object's layout).
const F_ISTIMERENABLED: DWORD = 1 << 0;
const F_ISINITTIMERENABLED: DWORD = 1 << 1;
const F_LINESMEASURED: DWORD = 1 << 2;
const F_ISHORIZONTAL: DWORD = 1 << 3;

/// `ClockWndFormats[i]` — {IsTime, dwFormatFlags, lpFormat}. A fn (not a const
/// array) because the LPCWSTR entries derive from `w!` statics.
#[inline]
unsafe fn clock_wnd_format(i: usize) -> (BOOL, DWORD, LPCWSTR) {
    match i {
        0 => (1, 0, core::ptr::null()),
        1 => (0, 0, w!("dddd").as_ptr()),
        2 => (0, DATE_SHORTDATE, core::ptr::null()),
        _ => (0, 0, core::ptr::null()),
    }
}

static szTrayClockWndClass: [u16; 16] = [
    b'T' as u16, b'r' as u16, b'a' as u16, b'y' as u16, b'C' as u16, b'l' as u16, b'o' as u16,
    b'c' as u16, b'k' as u16, b'W' as u16, b'C' as u16, b'l' as u16, b'a' as u16, b's' as u16,
    b's' as u16, 0,
];

// ---- the object (ATL bases flattened) ----

#[repr(C)]
pub struct CTrayClockWnd {
    // COM identity — IOleWindow vtbl first (IUnknown shares it).
    pub lpVtbl: *const IOleWindowVtbl,
    // CComObjectRootEx m_dwRef
    pub ref_: LONG,
    // CWindowImpl m_hWnd
    pub m_hWnd: HWND,
    // --- CTrayClockWnd members ---
    pub hFont: HFONT,
    pub textColor: COLORREF,
    pub rcText: RECT,
    pub LocalTime: SYSTEMTIME,
    pub m_tooltip: HWND, // CTooltips → its HWND
    pub dwFlags: DWORD,  // union bitfields (see F_* above)
    pub LineSpacing: DWORD,
    pub CurrentSize: SIZE,
    pub VisibleLines: WORD,
    pub LineSizes: [SIZE; CLOCKWND_FORMAT_COUNT],
    pub szLines: [[u16; 48]; CLOCKWND_FORMAT_COUNT],
}

pub type PCTrayClockWnd = *mut CTrayClockWnd;

impl CTrayClockWnd {
    #[inline]
    fn get_flag(&self, m: DWORD) -> bool {
        self.dwFlags & m != 0
    }
    #[inline]
    fn set_flag(&mut self, m: DWORD, v: bool) {
        if v {
            self.dwFlags |= m;
        } else {
            self.dwFlags &= !m;
        }
    }
}

// ---- CTooltips wrappers (rosctrls.h) — thin TTM_* SendMessage forwards ----

unsafe fn tooltip_create(hwnd_parent: HWND, styles: DWORD) -> HWND {
    // CTooltips::Create: named-class CWindow::Create against "tooltips_class32",
    // default exStyle WS_EX_TOPMOST.
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
        core::ptr::null_mut(),
        hExplorerInstance,
        core::ptr::null_mut(),
    )
}

unsafe fn tooltip_update_tip_text(tooltip: HWND, hwnd: HWND, uid: UINT_PTR, text: LPWSTR) {
    let mut ti: TOOLINFOW = core::mem::zeroed();
    ti.cbSize = TTTOOLINFOW_V1_SIZE();
    ti.hwnd = hwnd;
    ti.uId = uid;
    ti.lpszText = text;
    SendMessageW(
        tooltip,
        TTM_UPDATETIPTEXTW,
        0,
        &ti as *const TOOLINFOW as LPARAM,
    );
}

// ---- construction / destruction (CComObject new/delete of a Rust class) ----

#[no_mangle]
pub unsafe extern "C" fn CTrayClockWnd_ctor(this: *mut CTrayClockWnd) {
    // CTrayClockWnd() : hFont(NULL), textColor(0), dwFlags(0), LineSpacing(0),
    // VisibleLines(0) + ZeroMemory of the rest.
    (*this).lpVtbl = &vt_CTrayClockWnd;
    (*this).ref_ = 0;
    (*this).m_hWnd = core::ptr::null_mut();
    (*this).hFont = core::ptr::null_mut();
    (*this).textColor = 0;
    (*this).rcText = core::mem::zeroed();
    (*this).LocalTime = core::mem::zeroed();
    (*this).m_tooltip = core::ptr::null_mut();
    (*this).dwFlags = 0;
    (*this).LineSpacing = 0;
    (*this).CurrentSize = core::mem::zeroed();
    (*this).VisibleLines = 0;
    (*this).LineSizes = core::mem::zeroed();
    (*this).szLines = core::mem::zeroed();
}

unsafe fn CTrayClockWnd_new() -> *mut CTrayClockWnd {
    let p = HeapAlloc(hProcessHeap, 0, core::mem::size_of::<CTrayClockWnd>() as SIZE_T)
        as *mut CTrayClockWnd;
    if !p.is_null() {
        CTrayClockWnd_ctor(p);
    }
    p
}

unsafe fn CTrayClockWnd_delete(this: *mut CTrayClockWnd) {
    if !this.is_null() {
        // ~CTrayClockWnd() is empty; nothing to tear down (the HWND is owned by
        // the parent and destroyed via WM_(NC)DESTROY).
        HeapFree(hProcessHeap, 0, this as *mut c_void);
    }
}

// ---- COM: IUnknown / IOleWindow (BEGIN_COM_MAP + CComObject) ----

#[no_mangle]
pub unsafe extern "system" fn CTrayClockWnd_QueryInterface(
    iface: *mut IOleWindow,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let this = iface as *mut CTrayClockWnd;
    if ppv.is_null() {
        return E_POINTER;
    }
    // COM_INTERFACE_ENTRY_IID(IID_IOleWindow) + implicit IUnknown (first entry).
    if guid_eq(riid, &IID_IUnknown) || guid_eq(riid, &IID_IOleWindow) {
        *ppv = this as *mut c_void;
        CTrayClockWnd_AddRef(iface);
        return S_OK;
    }
    *ppv = core::ptr::null_mut();
    E_NOINTERFACE
}

#[no_mangle]
pub unsafe extern "system" fn CTrayClockWnd_AddRef(iface: *mut IOleWindow) -> u32 {
    let this = iface as *mut CTrayClockWnd;
    interlocked_increment(&mut (*this).ref_) as u32
}

#[no_mangle]
pub unsafe extern "system" fn CTrayClockWnd_Release(iface: *mut IOleWindow) -> u32 {
    let this = iface as *mut CTrayClockWnd;
    let n = interlocked_decrement(&mut (*this).ref_);
    if n == 0 {
        CTrayClockWnd_delete(this);
        return 0;
    }
    n as u32
}

/// IOleWindow::GetWindow (inline in trayclock.cpp).
#[no_mangle]
pub unsafe extern "system" fn CTrayClockWnd_GetWindow(
    iface: *mut IOleWindow,
    phwnd: *mut HWND,
) -> HRESULT {
    let this = iface as *mut CTrayClockWnd;
    if phwnd.is_null() {
        return E_INVALIDARG;
    }
    *phwnd = (*this).m_hWnd;
    S_OK
}

/// IOleWindow::ContextSensitiveHelp (inline).
#[no_mangle]
pub unsafe extern "system" fn CTrayClockWnd_ContextSensitiveHelp(
    _iface: *mut IOleWindow,
    _f_enter_mode: BOOL,
) -> HRESULT {
    E_NOTIMPL
}

static vt_CTrayClockWnd: IOleWindowVtbl = IOleWindowVtbl {
    QueryInterface: CTrayClockWnd_QueryInterface,
    AddRef: CTrayClockWnd_AddRef,
    Release: CTrayClockWnd_Release,
    GetWindow: CTrayClockWnd_GetWindow,
    ContextSensitiveHelp: CTrayClockWnd_ContextSensitiveHelp,
};

/// `IsEqualGUID` (guiddef.h inline).
#[inline]
unsafe fn guid_eq(a: REFIID, b: *const IID) -> bool {
    let a = &*a;
    let b = &*b;
    a.Data1 == b.Data1 && a.Data2 == b.Data2 && a.Data3 == b.Data3 && a.Data4 == b.Data4
}

// ---- the class methods (private, formerly CTrayClockWnd::*) ----

unsafe fn OnThemeChanged(this: *mut CTrayClockWnd) -> LRESULT {
    let mut clockFont: LOGFONTW = core::mem::zeroed();
    let clockTheme: HTHEME = OpenThemeData((*this).m_hWnd, w!("Clock").as_ptr());

    if !clockTheme.is_null() {
        GetThemeFont(
            clockTheme,
            core::ptr::null_mut(),
            CLP_TIME,
            0,
            TMT_FONT,
            &mut clockFont,
        );

        let hFont = CreateFontIndirectW(&clockFont);

        GetThemeColor(clockTheme, CLP_TIME, 0, TMT_TEXTCOLOR, &mut (*this).textColor);

        if !(*this).hFont.is_null() {
            DeleteObject((*this).hFont as HGDIOBJ);
        }

        SetFont(this, hFont, 0);

        CloseThemeData(clockTheme);
    }

    1
}

unsafe fn MeasureLines(this: *mut CTrayClockWnd) -> BOOL {
    let mut hPrevFont: HFONT = core::ptr::null_mut();
    let mut bRet: BOOL = 1;

    let hDC = GetDC((*this).m_hWnd);
    if !hDC.is_null() {
        if !(*this).hFont.is_null() {
            hPrevFont = SelectObject(hDC, (*this).hFont as HGDIOBJ) as HFONT;
        }

        let mut i = 0usize;
        while i < CLOCKWND_FORMAT_COUNT && bRet != 0 {
            if (*this).szLines[i][0] != 0
                && GetTextExtentPointW(
                    hDC,
                    (*this).szLines[i].as_ptr(),
                    wcslen_local((*this).szLines[i].as_ptr()) as i32,
                    &mut (*this).LineSizes[i],
                ) == 0
            {
                bRet = 0;
                break;
            }
            i += 1;
        }

        if !(*this).hFont.is_null() {
            SelectObject(hDC, hPrevFont as HGDIOBJ);
        }

        ReleaseDC((*this).m_hWnd, hDC);

        if bRet != 0 {
            (*this).LineSpacing = 0;

            let mut c = 0u32;
            let mut i = 0usize;
            while i < CLOCKWND_FORMAT_COUNT {
                if (*this).LineSizes[i].cx > 0 {
                    (*this).LineSpacing += (*this).LineSizes[i].cy as DWORD;
                    c += 1;
                }
                i += 1;
            }

            if c > 0 {
                (*this).LineSpacing = ((*this).LineSpacing / c) / 2;
            }

            return 1;
        }
    }

    0
}

unsafe fn GetMinimumSize(this: *mut CTrayClockWnd, Horizontal: BOOL, pSize: *mut SIZE) -> WORD {
    let mut iLinesVisible: WORD = 0;
    let mut szMax: SIZE = SIZE { cx: 0, cy: 0 };

    if !(*this).get_flag(F_LINESMEASURED) {
        let m = MeasureLines(this) != 0;
        (*this).set_flag(F_LINESMEASURED, m);
    }

    if !(*this).get_flag(F_LINESMEASURED) {
        return 0;
    }

    // Prevents the date being cut off when the weekday is shorter than the date.
    if (*this).VisibleLines > 1 && g_TaskbarSettings.bPreferDate != 0 {
        szMax.cx = (*this).LineSizes[CLOCKWND_FORMAT_DATE].cx;
    }

    let mut i = 0usize;
    while i < CLOCKWND_FORMAT_COUNT {
        if (*this).LineSizes[i].cx != 0 {
            if iLinesVisible > 0 {
                if Horizontal != 0 {
                    if szMax.cy + (*this).LineSizes[i].cy + (*this).LineSpacing as LONG
                        > (*pSize).cy - (2 * TRAY_CLOCK_WND_SPACING_Y)
                    {
                        break;
                    }
                } else if (*this).LineSizes[i].cx > (*pSize).cx - (2 * TRAY_CLOCK_WND_SPACING_X) {
                    break;
                }

                szMax.cy += (*this).LineSpacing as LONG;
            }

            iLinesVisible += 1;

            szMax.cy += (*this).LineSizes[i].cy;
            if (*this).LineSizes[i].cx > szMax.cx {
                szMax.cx = (*this).LineSizes[i].cx;
            }
        }
        i += 1;
    }

    szMax.cx += 2 * TRAY_CLOCK_WND_SPACING_X;
    szMax.cy += 2 * TRAY_CLOCK_WND_SPACING_Y;

    *pSize = szMax;

    iLinesVisible
}

unsafe fn UpdateWnd(this: *mut CTrayClockWnd) {
    (*this).LineSizes = core::mem::zeroed();

    let szPrevCurrent = (*this).CurrentSize;

    let mut i = 0usize;
    while i < CLOCKWND_FORMAT_COUNT {
        (*this).szLines[i][0] = 0;
        let BufSize = 48i32;

        let (is_time, fmt_flags, lp_format) = clock_wnd_format(i);
        let iRet = if is_time != 0 {
            GetTimeFormatW(
                LOCALE_USER_DEFAULT,
                if g_TaskbarSettings.bShowSeconds != 0 {
                    fmt_flags
                } else {
                    TIME_NOSECONDS
                },
                &(*this).LocalTime,
                lp_format,
                (*this).szLines[i].as_mut_ptr(),
                BufSize,
            )
        } else {
            GetDateFormatW(
                LOCALE_USER_DEFAULT,
                fmt_flags,
                &(*this).LocalTime,
                lp_format,
                (*this).szLines[i].as_mut_ptr(),
                BufSize,
            )
        };

        if iRet != 0 && i == 0 {
            // Set the window text to the time only.
            SetWindowTextW((*this).m_hWnd, (*this).szLines[i].as_ptr());
        }
        i += 1;
    }

    let m = MeasureLines(this) != 0;
    (*this).set_flag(F_LINESMEASURED, m);

    let mut rcClient: RECT = core::mem::zeroed();
    if (*this).get_flag(F_LINESMEASURED) && GetClientRect((*this).m_hWnd, &mut rcClient) != 0 {
        let mut szWnd: SIZE = SIZE {
            cx: rcClient.right,
            cy: rcClient.bottom,
        };

        (*this).VisibleLines = GetMinimumSize(this, (*this).get_flag(F_ISHORIZONTAL) as BOOL, &mut szWnd);
        (*this).CurrentSize = szWnd;
    }

    if IsWindowVisible((*this).m_hWnd) != 0 {
        InvalidateRect((*this).m_hWnd, core::ptr::null(), 1);

        if szPrevCurrent.cx != (*this).CurrentSize.cx || szPrevCurrent.cy != (*this).CurrentSize.cy
        {
            // Ask the parent to resize.
            let hParent = GetParent((*this).m_hWnd);
            let mut nmh: NMHDR = core::mem::zeroed();
            nmh.hwndFrom = hParent;
            nmh.idFrom = 0;
            nmh.code = NTNWM_REALIGN as UINT;
            SendMessageW(hParent, WM_NOTIFY, 0, &nmh as *const NMHDR as LPARAM);
        }
    }

    let iDateLength = GetDateFormatW(
        LOCALE_USER_DEFAULT,
        DATE_LONGDATE,
        &(*this).LocalTime,
        core::ptr::null(),
        core::ptr::null_mut(),
        0,
    );
    if iDateLength <= 0 {
        return;
    }

    // C++ `new WCHAR[iDateLength]` / `delete[]` → process-heap alloc/free.
    let szDate =
        HeapAlloc(hProcessHeap, 0, (iDateLength as SIZE_T) * 2) as *mut u16;
    if szDate.is_null() {
        return;
    }
    if GetDateFormatW(
        LOCALE_USER_DEFAULT,
        DATE_LONGDATE,
        &(*this).LocalTime,
        core::ptr::null(),
        szDate,
        iDateLength,
    ) > 0
    {
        tooltip_update_tip_text((*this).m_tooltip, (*this).m_hWnd, (*this).m_hWnd as UINT_PTR, szDate);
    }
    HeapFree(hProcessHeap, 0, szDate as *mut c_void);
}

unsafe fn Update(this: *mut CTrayClockWnd) {
    GetLocalTime(&mut (*this).LocalTime);
    UpdateWnd(this);
}

unsafe fn CalculateDueTime(this: *mut CTrayClockWnd) -> UINT {
    GetLocalTime(&mut (*this).LocalTime);
    let mut uiDueTime = 1000u32 - (*this).LocalTime.wMilliseconds as u32;
    if g_TaskbarSettings.bShowSeconds == 0 {
        uiDueTime += (59 - (*this).LocalTime.wSecond as u32) * 1000;
    }
    uiDueTime
}

unsafe fn ResetTime(this: *mut CTrayClockWnd) -> BOOL {
    // Disable all timers.
    if (*this).get_flag(F_ISTIMERENABLED) {
        KillTimer((*this).m_hWnd, ID_TRAYCLOCK_TIMER);
        (*this).set_flag(F_ISTIMERENABLED, false);
    } else if (*this).get_flag(F_ISINITTIMERENABLED) {
        KillTimer((*this).m_hWnd, ID_TRAYCLOCK_TIMER_INIT);
    }

    let uiDueTime = CalculateDueTime(this);

    let Ret =
        (SetTimer((*this).m_hWnd, ID_TRAYCLOCK_TIMER_INIT, uiDueTime, None) != 0) as BOOL;
    (*this).set_flag(F_ISINITTIMERENABLED, Ret != 0);

    Ret
}

unsafe fn CalibrateTimer(this: *mut CTrayClockWnd) {
    KillTimer((*this).m_hWnd, ID_TRAYCLOCK_TIMER_INIT);
    (*this).set_flag(F_ISINITTIMERENABLED, false);

    let uiDueTime = CalculateDueTime(this);

    let (uiWait1, uiWait2) = if g_TaskbarSettings.bShowSeconds != 0 {
        (1000u32 - 200, 1000u32)
    } else {
        (60 * 1000u32 - 200, 60 * 1000u32)
    };

    if uiDueTime > uiWait1 {
        let Ret = (SetTimer((*this).m_hWnd, ID_TRAYCLOCK_TIMER, uiWait2, None) != 0) as BOOL;
        (*this).set_flag(F_ISTIMERENABLED, Ret != 0);
    } else {
        ResetTime(this);
    }

    Update(this);
}

unsafe fn PaintLine(
    this: *mut CTrayClockWnd,
    hDC: HDC,
    rcClient: *mut RECT,
    LineNumber: UINT,
    szLinesIndex: UINT,
) {
    if (*this).LineSizes[LineNumber as usize].cx == 0 {
        return;
    }

    let HShift: LONG = if (*this).get_flag(F_ISHORIZONTAL)
        && ((*this).VisibleLines <= 1 || (*&raw const g_TaskbarSettings).UseCompactTrayIcons() != 0)
    {
        0
    } else {
        TRAY_CLOCK_WND_SPACING_X
    };

    TextOutW(
        hDC,
        (((*rcClient).right - (*this).LineSizes[szLinesIndex as usize].cx) / 2) + HShift,
        (*rcClient).top + TRAY_CLOCK_WND_SPACING_Y,
        (*this).szLines[szLinesIndex as usize].as_ptr(),
        wcslen_local((*this).szLines[szLinesIndex as usize].as_ptr()) as i32,
    );

    (*rcClient).top += (*this).LineSizes[LineNumber as usize].cy + (*this).LineSpacing as LONG;
}

unsafe fn SetFont(this: *mut CTrayClockWnd, hNewFont: HFONT, bRedraw: BOOL) {
    (*this).hFont = hNewFont;
    let m = MeasureLines(this) != 0;
    (*this).set_flag(F_LINESMEASURED, m);
    if bRedraw != 0 {
        InvalidateRect((*this).m_hWnd, core::ptr::null(), 1);
    }
}

unsafe fn DrawBackground(this: *mut CTrayClockWnd, hdc: HDC) -> LRESULT {
    let mut rect: RECT = core::mem::zeroed();
    GetClientRect((*this).m_hWnd, &mut rect);
    DrawThemeParentBackground((*this).m_hWnd, hdc, &rect);
    1
}

// ---- message handlers (MESSAGE_HANDLER bodies) ----

unsafe fn OnDestroy(
    this: *mut CTrayClockWnd,
    _uMsg: UINT,
    _wParam: WPARAM,
    _lParam: LPARAM,
    _bHandled: *mut BOOL,
) -> LRESULT {
    if (*this).get_flag(F_ISTIMERENABLED) {
        KillTimer((*this).m_hWnd, ID_TRAYCLOCK_TIMER);
    } else if (*this).get_flag(F_ISINITTIMERENABLED) {
        KillTimer((*this).m_hWnd, ID_TRAYCLOCK_TIMER_INIT);
    }
    1
}

unsafe fn OnPaint(
    this: *mut CTrayClockWnd,
    _uMsg: UINT,
    wParam: WPARAM,
    _lParam: LPARAM,
    _bHandled: *mut BOOL,
) -> LRESULT {
    let mut rcClient: RECT = core::mem::zeroed();
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    let mut hDC = wParam as HDC;

    if wParam == 0 {
        hDC = BeginPaint((*this).m_hWnd, &mut ps);
    }

    if hDC.is_null() {
        return 0;
    }

    if (*this).get_flag(F_LINESMEASURED) && GetClientRect((*this).m_hWnd, &mut rcClient) != 0 {
        let iPrevBkMode = SetBkMode(hDC, TRANSPARENT as i32);

        if IsAppThemed() == 0 {
            (*this).textColor = GetSysColor(COLOR_BTNTEXT as i32);
        }

        SetTextColor(hDC, (*this).textColor);

        let hPrevFont = SelectObject(hDC, (*this).hFont as HGDIOBJ) as HFONT;

        rcClient.top = (rcClient.bottom - (*this).CurrentSize.cy) / 2;
        rcClient.bottom = rcClient.top + (*this).CurrentSize.cy;

        if (*this).VisibleLines == 2 {
            PaintLine(this, hDC, &mut rcClient, 0, CLOCKWND_FORMAT_TIME as UINT);
            PaintLine(
                this,
                hDC,
                &mut rcClient,
                1,
                if g_TaskbarSettings.bPreferDate != 0 {
                    CLOCKWND_FORMAT_DATE as UINT
                } else {
                    CLOCKWND_FORMAT_DAY as UINT
                },
            );
        } else {
            let mut i = 0u32;
            let mut line = 0u16;
            while (i as usize) < CLOCKWND_FORMAT_COUNT && line < (*this).VisibleLines {
                PaintLine(this, hDC, &mut rcClient, i, i);
                line += 1;
                i += 1;
            }
        }

        SelectObject(hDC, hPrevFont as HGDIOBJ);
        SetBkMode(hDC, iPrevBkMode);
    }

    if wParam == 0 {
        EndPaint((*this).m_hWnd, &ps);
    }

    1
}

unsafe fn OnEraseBackground(
    this: *mut CTrayClockWnd,
    _uMsg: UINT,
    wParam: WPARAM,
    _lParam: LPARAM,
    bHandled: *mut BOOL,
) -> LRESULT {
    let hdc = wParam as HDC;
    if IsAppThemed() == 0 {
        *bHandled = 0;
        return 0;
    }
    DrawBackground(this, hdc)
}

unsafe fn OnThemeChangedMsg(
    this: *mut CTrayClockWnd,
    _uMsg: UINT,
    _wParam: WPARAM,
    _lParam: LPARAM,
    _bHandled: *mut BOOL,
) -> LRESULT {
    OnThemeChanged(this)
}

unsafe fn OnTimer(
    this: *mut CTrayClockWnd,
    _uMsg: UINT,
    wParam: WPARAM,
    _lParam: LPARAM,
    _bHandled: *mut BOOL,
) -> LRESULT {
    match wParam as UINT_PTR {
        ID_TRAYCLOCK_TIMER => Update(this),
        ID_TRAYCLOCK_TIMER_INIT => CalibrateTimer(this),
        _ => {}
    }
    1
}

unsafe fn OnGetMinimumSize(
    this: *mut CTrayClockWnd,
    _uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    _bHandled: *mut BOOL,
) -> LRESULT {
    (*this).set_flag(F_ISHORIZONTAL, wParam != 0);
    (GetMinimumSize(this, wParam as BOOL, lParam as *mut SIZE) != 0) as LRESULT
}

unsafe fn OnContextMenu(
    this: *mut CTrayClockWnd,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    _bHandled: *mut BOOL,
) -> LRESULT {
    SendMessageW(GetParent((*this).m_hWnd), uMsg, wParam, lParam)
}

unsafe fn OnSetFont(
    this: *mut CTrayClockWnd,
    _uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    _bHandled: *mut BOOL,
) -> LRESULT {
    SetFont(this, wParam as HFONT, (lParam & 0xFFFF) as BOOL);
    1
}

unsafe fn OnCreate(
    this: *mut CTrayClockWnd,
    _uMsg: UINT,
    _wParam: WPARAM,
    _lParam: LPARAM,
    _bHandled: *mut BOOL,
) -> LRESULT {
    (*this).m_tooltip = tooltip_create((*this).m_hWnd, WS_POPUP | TTS_NOPREFIX | TTS_ALWAYSTIP);

    let mut ti: TOOLINFOW = core::mem::zeroed();
    ti.cbSize = TTTOOLINFOW_V1_SIZE();
    ti.uFlags = TTF_IDISHWND | TTF_SUBCLASS;
    ti.hwnd = (*this).m_hWnd;
    ti.uId = (*this).m_hWnd as UINT_PTR;
    ti.lpszText = core::ptr::null_mut();
    ti.lParam = 0;

    SendMessageW(
        (*this).m_tooltip,
        TTM_ADDTOOLW,
        0,
        &ti as *const TOOLINFOW as LPARAM,
    );

    if GetHideClock() == 0 {
        ResetTime(this);
    }

    Update(this);

    1
}

unsafe fn OnSize(
    this: *mut CTrayClockWnd,
    _uMsg: UINT,
    _wParam: WPARAM,
    lParam: LPARAM,
    _bHandled: *mut BOOL,
) -> LRESULT {
    // LOWORD/HIWORD are unsigned (client width/height).
    let mut szClient: SIZE = SIZE {
        cx: (lParam & 0xFFFF) as u16 as i32,
        cy: ((lParam >> 16) & 0xFFFF) as u16 as i32,
    };

    (*this).VisibleLines = GetMinimumSize(this, (*this).get_flag(F_ISHORIZONTAL) as BOOL, &mut szClient);
    (*this).CurrentSize = szClient;

    UpdateWnd(this);
    1
}

unsafe fn OnTaskbarSettingsChanged(
    this: *mut CTrayClockWnd,
    _uMsg: UINT,
    _wParam: WPARAM,
    lParam: LPARAM,
    _bHandled: *mut BOOL,
) -> LRESULT {
    let mut bRealign: BOOL = 0;
    let mut bHideClock = GetHideClock();
    let newSettings = lParam as *const TaskbarSettings;

    if (*newSettings).sr.HideClock() != (IsWindowVisible((*this).m_hWnd) == 0) as DWORD {
        (*&raw mut g_TaskbarSettings).sr.set_HideClock((*newSettings).sr.HideClock());
        bHideClock = GetHideClock();
        ShowWindow((*this).m_hWnd, if bHideClock != 0 { SW_HIDE } else { SW_SHOW } as i32);
        bRealign = 1;

        if bHideClock != 0 {
            if (*this).get_flag(F_ISTIMERENABLED) {
                KillTimer((*this).m_hWnd, ID_TRAYCLOCK_TIMER);
                (*this).set_flag(F_ISTIMERENABLED, false);
            } else if (*this).get_flag(F_ISINITTIMERENABLED) {
                KillTimer((*this).m_hWnd, ID_TRAYCLOCK_TIMER_INIT);
                (*this).set_flag(F_ISINITTIMERENABLED, false);
            }
        } else {
            ResetTime(this);
        }
    }

    if (*newSettings).bShowSeconds != g_TaskbarSettings.bShowSeconds {
        g_TaskbarSettings.bShowSeconds = (*newSettings).bShowSeconds;
        if bHideClock == 0 {
            bRealign = 1;
            ResetTime(this);
        }
    }

    if (*newSettings).bPreferDate != g_TaskbarSettings.bPreferDate {
        g_TaskbarSettings.bPreferDate = (*newSettings).bPreferDate;
        bRealign = 1;
    }

    if bRealign != 0 {
        let hParent = GetParent((*this).m_hWnd);
        let mut nmh: NMHDR = core::mem::zeroed();
        nmh.hwndFrom = hParent;
        nmh.idFrom = 0;
        nmh.code = NTNWM_REALIGN as UINT;
        SendMessageW(hParent, WM_NOTIFY, 0, &nmh as *const NMHDR as LPARAM);
        Update(this);
    }
    0
}

unsafe fn OnLButtonDblClick(
    this: *mut CTrayClockWnd,
    _uMsg: UINT,
    _wParam: WPARAM,
    _lParam: LPARAM,
    _bHandled: *mut BOOL,
) -> LRESULT {
    if IsWindowVisible((*this).m_hWnd) != 0 {
        ShellExecuteW(
            (*this).m_hWnd,
            core::ptr::null(),
            w!("timedate.cpl").as_ptr(),
            core::ptr::null(),
            core::ptr::null(),
            SW_NORMAL as i32,
        );
    }
    1
}

// ---- the WndProc (BEGIN_MSG_MAP) — arch-split callback ABI ----

unsafe fn CTrayClockWnd_process(
    this: *mut CTrayClockWnd,
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut bHandled: BOOL = 1;
    let mut lResult: LRESULT = 0;

    match uMsg {
        WM_CREATE => lResult = OnCreate(this, uMsg, wParam, lParam, &mut bHandled),
        WM_DESTROY => lResult = OnDestroy(this, uMsg, wParam, lParam, &mut bHandled),
        WM_ERASEBKGND => lResult = OnEraseBackground(this, uMsg, wParam, lParam, &mut bHandled),
        WM_SIZE => lResult = OnSize(this, uMsg, wParam, lParam, &mut bHandled),
        WM_PAINT | WM_PRINTCLIENT => lResult = OnPaint(this, uMsg, wParam, lParam, &mut bHandled),
        WM_THEMECHANGED => lResult = OnThemeChangedMsg(this, uMsg, wParam, lParam, &mut bHandled),
        WM_TIMER => lResult = OnTimer(this, uMsg, wParam, lParam, &mut bHandled),
        WM_CONTEXTMENU => lResult = OnContextMenu(this, uMsg, wParam, lParam, &mut bHandled),
        WM_SETFONT => lResult = OnSetFont(this, uMsg, wParam, lParam, &mut bHandled),
        x if x == TNWM_GETMINIMUMSIZE => {
            lResult = OnGetMinimumSize(this, uMsg, wParam, lParam, &mut bHandled)
        }
        x if x == TWM_SETTINGSCHANGED => {
            lResult = OnTaskbarSettingsChanged(this, uMsg, wParam, lParam, &mut bHandled)
        }
        WM_LBUTTONDBLCLK => lResult = OnLButtonDblClick(this, uMsg, wParam, lParam, &mut bHandled),
        _ => bHandled = 0,
    }

    if bHandled != 0 {
        lResult
    } else {
        DefWindowProcW(hWnd, uMsg, wParam, lParam)
    }
}

/// The registered class WndProc. Recovers `this` from GWLP_USERDATA (stashed on
/// WM_NCCREATE) and dispatches; the ATL CWindowImpl thunk equivalent. ABI is
/// arch-split (bindgen WNDPROC is `extern "stdcall"` on i386, `extern "C"` on
/// LLP64) exactly like desktop.rs's thread proc.
#[no_mangle]
pub unsafe extern "system" fn CTrayClockWnd_WindowProc(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    CTrayClockWnd_window_proc_impl(hWnd, uMsg, wParam, lParam)
}

#[inline]
unsafe fn CTrayClockWnd_window_proc_impl(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let this = if uMsg == WM_NCCREATE {
        let p = wndproc_nccreate_this(hWnd, lParam) as *mut CTrayClockWnd;
        if !p.is_null() {
            (*p).m_hWnd = hWnd;
        }
        p
    } else {
        get_window_this(hWnd) as *mut CTrayClockWnd
    };

    if this.is_null() {
        return DefWindowProcW(hWnd, uMsg, wParam, lParam);
    }

    let r = CTrayClockWnd_process(this, hWnd, uMsg, wParam, lParam);

    if uMsg == WM_NCDESTROY {
        (*this).m_hWnd = core::ptr::null_mut();
    }
    r
}

// ---- CWindowImpl::Create + Initialize ----

unsafe fn CTrayClockWnd_Create(this: *mut CTrayClockWnd, hWndParent: HWND, dwStyle: DWORD) -> HWND {
    // DECLARE_WND_CLASS_EX(szTrayClockWndClass, CS_DBLCLKS, COLOR_3DFACE)
    register_class_once(
        hExplorerInstance,
        szTrayClockWndClass.as_ptr(),
        Some(CTrayClockWnd_WindowProc),
        CS_DBLCLKS,
        COLOR_3DFACE as i32,
    );

    (*this).m_hWnd = CreateWindowExW(
        0,
        szTrayClockWndClass.as_ptr(),
        core::ptr::null(),
        dwStyle,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        hWndParent,
        core::ptr::null_mut(),
        hExplorerInstance,
        this as LPVOID,
    );
    (*this).m_hWnd
}

unsafe fn CTrayClockWnd_Initialize(this: *mut CTrayClockWnd, hWndParent: HWND) -> HRESULT {
    (*this).set_flag(F_ISHORIZONTAL, true);

    let mut dwStyle: DWORD = WS_CHILD | WS_CLIPSIBLINGS;
    if GetHideClock() == 0 {
        dwStyle |= WS_VISIBLE;
    }

    CTrayClockWnd_Create(this, hWndParent, dwStyle);
    if (*this).m_hWnd.is_null() {
        return E_FAIL;
    }

    SetWindowTheme((*this).m_hWnd, w!("TrayNotify").as_ptr(), core::ptr::null());

    S_OK
}

// ---- the factory (ShellObjectCreatorInit<CTrayClockWnd>) ----

#[no_mangle]
pub unsafe extern "C" fn CTrayClockWnd_CreateInstance(
    hwndParent: HWND,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    // ShellObjectCreatorInit<T>(hwndParent, riid, ppv): new (ref=0) → AddRef →
    // Initialize(arg) → QI(riid) → Release. (CComObject::CreateInstance runs
    // FinalConstruct, which CTrayClockWnd doesn't override, so ctor suffices.)
    let pobj = CTrayClockWnd_new();
    if pobj.is_null() {
        return E_OUTOFMEMORY;
    }
    let iface = pobj as *mut IOleWindow;

    CTrayClockWnd_AddRef(iface);

    let mut hr = CTrayClockWnd_Initialize(pobj, hwndParent);
    if hr >= 0 {
        hr = CTrayClockWnd_QueryInterface(iface, riid, ppv);
    }

    CTrayClockWnd_Release(iface);
    hr
}

// wcslen for a raw *const u16 (no_std; msvcrt wcslen is also available but a
// tiny local avoids an FFI hop in the hot paint path).
#[inline]
unsafe fn wcslen_local(mut s: *const u16) -> usize {
    let mut n = 0usize;
    while *s != 0 {
        n += 1;
        s = s.add(1);
    }
    n
}
