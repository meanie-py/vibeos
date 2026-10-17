use win32k_sys::win32k::*;
// The crate's own functions: the definition is the declaration.
use crate::windows::font::DrawTextW;
use crate::windows::class::GetWindowLongPtrW;
#[allow(unused_imports)]
use core::ffi::c_void;

// ---- debug-print macros (DbgPrint-free no-ops; mirror cursoricon.rs) ----

// These user32 entry points are inline ntwrapper wrappers (NtUser* multiplexers),
// not external link symbols — import them instead of declaring them extern.
use crate::ntwrapper::{GetDC, SetWindowPos, ValidateRect, SetActiveWindow};

// ---- user32 / gdi32 own-exports + cross-module imports (WINAPI = stdcall) ----
extern "system" {
    fn GetWindowLongPtrA(hWnd: HWND, nIndex: core::ffi::c_int) -> LONG_PTR;
    fn SetWindowLongPtrW(hWnd: HWND, nIndex: core::ffi::c_int, dwNewLong: LONG_PTR) -> LONG_PTR;
    fn IsWindowEnabled(hWnd: HWND) -> BOOL;
    fn IsWindow(hWnd: HWND) -> BOOL;
    fn GetParent(hWnd: HWND) -> HWND;
    fn GetWindow(hWnd: HWND, uCmd: UINT) -> HWND;
    fn CreateWindowExA(
        dwExStyle: DWORD,
        lpClassName: LPCSTR,
        lpWindowName: LPCSTR,
        dwStyle: DWORD,
        x: core::ffi::c_int,
        y: core::ffi::c_int,
        nWidth: core::ffi::c_int,
        nHeight: core::ffi::c_int,
        hWndParent: HWND,
        hMenu: HMENU,
        hInstance: HINSTANCE,
        lpParam: LPVOID,
    ) -> HWND;
    fn MapWindowPoints(hWndFrom: HWND, hWndTo: HWND, lpPoints: LPPOINT, cPoints: UINT) -> core::ffi::c_int;
    fn GetWindowTextW(hWnd: HWND, lpString: LPWSTR, nMaxCount: core::ffi::c_int) -> core::ffi::c_int;
    fn GetClientRect(hWnd: HWND, lpRect: LPRECT) -> BOOL;
    fn GetActiveWindow() -> HWND;
    fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn SetRect(lprc: LPRECT, xLeft: core::ffi::c_int, yTop: core::ffi::c_int, xRight: core::ffi::c_int, yBottom: core::ffi::c_int) -> BOOL;
    fn GetSystemMetrics(nIndex: core::ffi::c_int) -> core::ffi::c_int;
    fn GetSysColor(nIndex: core::ffi::c_int) -> COLORREF;
    fn GetSysColorBrush(nIndex: core::ffi::c_int) -> HBRUSH;
    fn ReleaseDC(hWnd: HWND, hDC: HDC) -> core::ffi::c_int;
    fn FillRect(hDC: HDC, lprc: *const RECT, hbr: HBRUSH) -> core::ffi::c_int;
    fn GetClassLongPtrW(hWnd: HWND, nIndex: core::ffi::c_int) -> ULONG_PTR;
    fn SystemParametersInfoA(uiAction: UINT, uiParam: UINT, pvParam: PVOID, fWinIni: UINT) -> BOOL;
    // gdi32 exports
    fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
    fn GetStockObject(i: core::ffi::c_int) -> HGDIOBJ;
    fn GetObjectA(h: HANDLE, c: core::ffi::c_int, pv: LPVOID) -> core::ffi::c_int;
    fn CreateFontIndirectA(lplf: *const LOGFONTA) -> HFONT;
    fn SetTextColor(hdc: HDC, color: COLORREF) -> COLORREF;
    fn SetBkMode(hdc: HDC, mode: core::ffi::c_int) -> core::ffi::c_int;
    fn DPtoLP(hdc: HDC, lppt: LPPOINT, c: core::ffi::c_int) -> BOOL;
}

// ---- CRT cdecl string helpers (UTF-16) ----
#[inline]
unsafe fn strlenW(mut s: *const WCHAR) -> INT {
    let mut n: INT = 0;
    while *s != 0 {
        s = s.add(1);
        n += 1;
    }
    n
}
#[inline]
unsafe fn strcpyW(dst: LPWSTR, mut src: LPCWSTR) -> LPWSTR {
    let mut d = dst;
    loop {
        *d = *src;
        if *src == 0 {
            break;
        }
        d = d.add(1);
        src = src.add(1);
    }
    dst
}

// ---- wingdi.h RGB()/GetRValue()/GetGValue()/GetBValue() macros ----
#[inline]
fn RGB(r: u8, g: u8, b: u8) -> COLORREF {
    (r as COLORREF) | ((g as COLORREF) << 8) | ((b as COLORREF) << 16)
}
#[inline]
fn GetRValue(rgb: COLORREF) -> u8 {
    (rgb & 0xFF) as u8
}
#[inline]
fn GetGValue(rgb: COLORREF) -> u8 {
    ((rgb >> 8) & 0xFF) as u8
}
#[inline]
fn GetBValue(rgb: COLORREF) -> u8 {
    ((rgb >> 16) & 0xFF) as u8
}

// undocuser.h: WC_ICONTITLE == MAKEINTATOM(0x8004). Wine names it ICONTITLE_CLASS_ATOM.
const ICONTITLE_CLASS_ATOM: usize = 0x8004;

// Wine's WIN_SetOwner(hWnd, owner): set the window's owner. Faithful operation =
// GWLP_HWNDPARENT for an owned (non-child) window. (ICONTITLE_Create is gated out
// of the ReactOS build via #ifndef __REACTOS__, but the translation kept it.)
#[inline]
unsafe fn WIN_SetOwner(hWnd: HWND, owner: HWND) -> HWND {
    SetWindowLongPtrW(hWnd, GWLP_HWNDPARENT, owner as LONG_PTR) as HWND
}

// ---- file-scope preamble (translated) ----
static mut bMultiLineTitle: BOOL = 0;
static mut hIconTitleFont: HFONT = core::ptr::null_mut();

unsafe fn ICONTITLE_Create(owner: HWND) -> HWND {
    let hWnd: HWND;
    let instance = GetWindowLongPtrA(owner, GWLP_HINSTANCE) as HINSTANCE;
    let mut style: LONG = WS_CLIPSIBLINGS as LONG;

    if IsWindowEnabled(owner) == 0 {
        style |= WS_DISABLED as LONG;
    }
    if GetWindowLongPtrA(owner, GWL_STYLE) as isize & (WS_CHILD as isize) != 0 {
        hWnd = CreateWindowExA(
            0,
            ICONTITLE_CLASS_ATOM as LPCSTR,
            core::ptr::null_mut(),
            (style | WS_CHILD as LONG) as u32,
            0,
            0,
            1,
            1,
            GetParent(owner),
            core::ptr::null_mut(),
            instance,
            core::ptr::null_mut(),
        );
    } else {
        hWnd = CreateWindowExA(
            0,
            ICONTITLE_CLASS_ATOM as LPCSTR,
            core::ptr::null_mut(),
            style as u32,
            0,
            0,
            1,
            1,
            owner,
            core::ptr::null_mut(),
            instance,
            core::ptr::null_mut(),
        );
    }
    WIN_SetOwner(hWnd, owner);
    SetWindowLongPtrW(
        hWnd,
        GWL_STYLE,
        (GetWindowLongPtrW(hWnd, GWL_STYLE) as isize & !((WS_CAPTION | WS_BORDER) as isize)) as LONG_PTR,
    );
    hWnd
}

unsafe fn ICONTITLE_SetTitlePos(hwnd: HWND, owner: HWND) -> BOOL {
    static emptyTitleText: [WCHAR; 6] = [b'<' as WCHAR, b'.' as WCHAR, b'.' as WCHAR, b'.' as WCHAR, b'>' as WCHAR, 0];
    let mut str: [WCHAR; 80] = [0; 80];
    let hDC: HDC;
    let hPrevFont: HFONT;
    let mut rect: RECT = core::mem::zeroed();
    let cx: INT;
    let cy: INT;
    let mut pt: POINT = core::mem::zeroed();

    let mut length = GetWindowTextW(owner, str.as_mut_ptr(), (core::mem::size_of::<[WCHAR; 80]>() / core::mem::size_of::<WCHAR>()) as INT);

    while length != 0 && str[(length - 1) as usize] == b' ' as WCHAR {
        length -= 1;
        str[length as usize] = 0;
    }

    if length == 0 {
        strcpyW(str.as_mut_ptr(), emptyTitleText.as_ptr());
        length = strlenW(str.as_ptr()) as INT;
    }

    hDC = GetDC(hwnd);
    if hDC.is_null() {
        return FALSE as BOOL;
    }

    hPrevFont = SelectObject(hDC, hIconTitleFont as _) as HFONT;

    SetRect(
        core::ptr::addr_of_mut!(rect),
        0,
        0,
        GetSystemMetrics(SM_CXICONSPACING as _) - GetSystemMetrics(SM_CXBORDER as _) * 2,
        GetSystemMetrics(SM_CYBORDER as _) * 2,
    );

    DrawTextW(
        hDC,
        str.as_ptr(),
        length,
        core::ptr::addr_of_mut!(rect),
        (DT_CALCRECT | DT_CENTER | DT_NOPREFIX | DT_WORDBREAK | (if bMultiLineTitle != 0 { 0 } else { DT_SINGLELINE })) as _,
    );

    SelectObject(hDC, hPrevFont as _);
    ReleaseDC(hwnd, hDC);

    cx = rect.right - rect.left + 4 * GetSystemMetrics(SM_CXBORDER as _);
    cy = rect.bottom - rect.top;

    pt.x = (GetSystemMetrics(SM_CXICON as _) - cx) / 2;
    pt.y = GetSystemMetrics(SM_CYICON as _);

    MapWindowPoints(owner, GetParent(hwnd), core::ptr::addr_of_mut!(pt) as LPPOINT, 1);

    SetWindowPos(hwnd, owner, pt.x, pt.y, cx, cy, SWP_NOACTIVATE as _);
    TRUE as BOOL
}

unsafe fn ICONTITLE_Paint(hwnd: HWND, owner: HWND, hDC: HDC, bActive: BOOL) -> BOOL {
    let mut rect: RECT = core::mem::zeroed();
    let hPrevFont: HFONT;
    let mut hBrush: HBRUSH;
    let mut textColor: COLORREF = 0;

    if bActive != 0 {
        hBrush = GetSysColorBrush(COLOR_ACTIVECAPTION as _);
        textColor = GetSysColor(COLOR_CAPTIONTEXT as _);
    } else {
        if GetWindowLongPtrA(hwnd, GWL_STYLE as _) as isize & WS_CHILD as isize != 0 {
            hBrush = GetClassLongPtrW(hwnd, GCLP_HBRBACKGROUND as _) as HBRUSH;
            if !hBrush.is_null() {
                let level: i32;
                let mut logBrush: LOGBRUSH = core::mem::zeroed();
                GetObjectA(
                    hBrush as _,
                    core::mem::size_of::<LOGBRUSH>() as _,
                    core::ptr::addr_of_mut!(logBrush) as _,
                );
                level = GetRValue(logBrush.lbColor) as i32
                    + GetGValue(logBrush.lbColor) as i32
                    + GetBValue(logBrush.lbColor) as i32;
                if level < (0x7F * 3) {
                    textColor = RGB(0xFF, 0xFF, 0xFF);
                }
            } else {
                hBrush = GetStockObject(WHITE_BRUSH as _) as HBRUSH;
            }
        } else {
            hBrush = GetStockObject(BLACK_BRUSH as _) as HBRUSH;
            textColor = RGB(0xFF, 0xFF, 0xFF);
        }
    }

    GetClientRect(hwnd, core::ptr::addr_of_mut!(rect));
    DPtoLP(hDC, core::ptr::addr_of_mut!(rect) as LPPOINT, 2);
    FillRect(hDC, core::ptr::addr_of_mut!(rect), hBrush);

    hPrevFont = SelectObject(hDC, hIconTitleFont as _) as HFONT;
    if !hPrevFont.is_null() {
        let mut buffer: [u16; 80] = [0; 80];

        let length = GetWindowTextW(owner, buffer.as_mut_ptr(), 80);
        SetTextColor(hDC, textColor);
        SetBkMode(hDC, TRANSPARENT as _);

        DrawTextW(
            hDC,
            buffer.as_ptr(),
            length,
            core::ptr::addr_of_mut!(rect),
            DT_CENTER
                | DT_NOPREFIX
                | DT_WORDBREAK
                | (if bMultiLineTitle != 0 { 0 } else { DT_SINGLELINE }),
        );

        SelectObject(hDC, hPrevFont as _);
    }
    (!hPrevFont.is_null()) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn IconTitleWndProc(hWnd: HWND, msg: UINT, wParam: WPARAM, mut lParam: LPARAM) -> LRESULT {
    let owner = GetWindow(hWnd, GW_OWNER as _);

    if IsWindow(hWnd) == 0 {
        return 0;
    }

    match msg {
        WM_CREATE => {
            if hIconTitleFont.is_null() {
                let mut logFont: LOGFONTA = core::mem::zeroed();
                SystemParametersInfoA(SPI_GETICONTITLELOGFONT, 0, core::ptr::addr_of_mut!(logFont) as _, 0);
                SystemParametersInfoA(SPI_GETICONTITLEWRAP, 0, core::ptr::addr_of_mut!(bMultiLineTitle) as _, 0);
                hIconTitleFont = CreateFontIndirectA(core::ptr::addr_of_mut!(logFont));
            }
            if !hIconTitleFont.is_null() { 0 } else { -1 }
        }
        WM_NCHITTEST => HTCAPTION as LRESULT,
        WM_NCMOUSEMOVE | WM_NCLBUTTONDBLCLK => SendMessageW(owner, msg, wParam, lParam),
        WM_ACTIVATE => {
            if wParam != 0 {
                SetActiveWindow(owner);
            }
            0
        }
        WM_CLOSE => 0,
        WM_SHOWWINDOW => {
            if wParam != 0 {
                ICONTITLE_SetTitlePos(hWnd, owner);
            }
            0
        }
        WM_ERASEBKGND => {
            if GetWindowLongPtrW(owner, GWL_STYLE) as isize & (WS_CHILD as isize) != 0 {
                lParam = SendMessageW(owner, WM_ISACTIVEICON, 0, 0);
            } else {
                lParam = (owner == GetActiveWindow()) as LPARAM;
            }
            if ICONTITLE_Paint(hWnd, owner, wParam as HDC, lParam as BOOL) != 0 {
                ValidateRect(hWnd, core::ptr::null_mut());
            }
            1
        }
        _ => DefWindowProcW(hWnd, msg, wParam, lParam),
    }
}
