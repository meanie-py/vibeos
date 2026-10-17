use win32k_sys::win32k::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::windows::dialog::FindResourceA;
use crate::windows::cursoricon::FindResourceW;
use crate::windows::window::HeapAlloc;
use crate::windows::message::HeapReAlloc;
use crate::windows::cursoricon::LoadResource;
// The crate's own functions: the definition is the declaration.
use crate::windows::class::GetWindowLongPtrW;
use crate::controls::scrollbar::SetScrollInfo;
use core::ffi::c_void;

// WINE debug macros — no-ops (mirror windows/menu.rs).

// Substrate helpers (correctly split per ABI on i386).
use crate::internal::{HIWORD, LOWORD, NtCurrentTeb, ValidateHwnd};
// CRT + gdi32 glue.
use crate::winbase::{
    memcpy, CP_ACP, DeleteObject, LineTo, MoveToEx, MultiByteToWideChar, SelectObject,
};
// NtUserx* multiplexers + EXTINLINE ntwrapper.h wrappers (ShowWindow, SetWindowPos,
// SetFocus, GetDC, RedrawWindow, DestroyWindow, MoveWindow, DeleteMenu, DestroyMenu,
// RemoveMenu, SetMenuDefaultItem, InvalidateRect, ShowScrollBar, KillTimer, SetTimer).
use crate::ntwrapper::{


    DeleteMenu, DestroyMenu, DestroyWindow, GetDC, InvalidateRect, MoveWindow, RemoveMenu,
    RedrawWindow, SetFocus, SetMenuDefaultItem, SetWindowPos, ShowScrollBar, ShowWindow,
};

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


// ── NT_SUCCESS (ntdef.h) ──────────────────────────────────────────────────────
#[inline]
unsafe fn NT_SUCCESS(s: NTSTATUS) -> bool { s >= 0 }

// ── min/max (C macros) ────────────────────────────────────────────────────────
#[inline]
fn min<T: PartialOrd>(a: T, b: T) -> T { if a < b { a } else { b } }
#[inline]
fn max<T: PartialOrd>(a: T, b: T) -> T { if a > b { a } else { b } }

// ── wine wide-string helpers (wine/unicode.h) ────────────────────────────────
#[inline]
unsafe fn strlenW(s: LPCWSTR) -> UINT {
    let mut n: UINT = 0;
    while *s.offset(n as isize) != 0 { n += 1; }
    n
}
#[inline]
unsafe fn strcpyW(dst: LPWSTR, src: LPCWSTR) -> LPWSTR {
    let mut i: isize = 0;
    loop {
        let c = *src.offset(i);
        *dst.offset(i) = c;
        if c == 0 { break; }
        i += 1;
    }
    dst
}
#[inline]
unsafe fn strcatW(dst: LPWSTR, src: LPCWSTR) -> LPWSTR {
    let mut d = dst;
    while *d != 0 { d = d.offset(1); }
    strcpyW(d, src);
    dst
}

// ── winuser.h MAKE* macros ───────────────────────────────────────────────────
#[inline]
fn MAKELRESULT(lo: WORD, hi: WORD) -> LRESULT { (((hi as DWORD) << 16) | (lo as DWORD)) as LRESULT }
#[inline]
fn MAKEINTRESOURCEW(i: WORD) -> LPWSTR { i as usize as LPWSTR }

// ── winuser.h constants (not in win32k_sys) ──────────────────────────────────
const HWND_TOP: HWND = 0 as HWND;
const HWND_BOTTOM: HWND = 1 as HWND;
const HBMMENU_MBAR_RESTORE: usize = 2;
const HBMMENU_MBAR_MINIMIZE: usize = 3;
const HBMMENU_MBAR_CLOSE: usize = 5;
const HBMMENU_MBAR_CLOSE_D: usize = 6;
const IDI_WINLOGO: usize = 32517;
const RT_DIALOG: usize = 5; // MAKEINTRESOURCE(5)

// ── user32 globals (linkable exports) ────────────────────────────────────────
extern "C" {
    static User32Instance: HINSTANCE; // dllmain.c / user32p.h
}

// GetWindowLongPtr (no suffix) maps to the W form on Unicode builds.
#[inline]
unsafe fn GetWindowLongPtr(hWnd: HWND, nIndex: INT) -> LONG_PTR { GetWindowLongPtrW(hWnd, nIndex) }

// user32-internal cdecl helper (windows/win.c).
extern "C" {
    fn WIN_GetFullHandle(hwnd: HWND) -> HWND;
    fn memset(dest: *mut c_void, c: INT, count: usize) -> *mut c_void;
}

// ── kernel32 imports (extern "system") ───────────────────────────────────────
extern "system" {
    fn GetProcessHeap() -> *mut c_void;
    fn HeapFree(hHeap: *mut c_void, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn SetLastError(dwErrCode: DWORD);
    fn LockResource(hResData: *mut c_void) -> LPVOID;
}

// ── gdi32 imports (extern "system") ──────────────────────────────────────────
extern "system" {
    fn ReleaseDC(hWnd: HWND, hDC: HDC) -> INT;
    fn CreateCompatibleDC(hdc: HDC) -> HDC;
    fn CreateCompatibleBitmap(hdc: HDC, cx: INT, cy: INT) -> HBITMAP;
    fn DeleteDC(hdc: HDC) -> BOOL;
    fn BitBlt(hdc: HDC, x: INT, y: INT, cx: INT, cy: INT, hdcSrc: HDC, x1: INT, y1: INT, rop: DWORD) -> BOOL;
    fn GetStockObject(i: INT) -> HGDIOBJ;
    fn SetMapMode(hdc: HDC, iMode: INT) -> INT;
}

// ── user32 public exports referenced here (extern "system") ──────────────────
extern "system" {
    fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn SendMessageTimeoutW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM, fuFlags: UINT, uTimeout: UINT, lpdwResult: *mut DWORD_PTR) -> LRESULT;
    fn PostMessageA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    fn DefWindowProcA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn GetWindowLongW(hWnd: HWND, nIndex: INT) -> LONG;
    fn GetWindowLongPtrA(hWnd: HWND, nIndex: INT) -> LONG_PTR;
    fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR;
    fn GetClassLongPtrW(hWnd: HWND, nIndex: INT) -> ULONG_PTR;
    fn GetParent(hwnd: HWND) -> HWND;
    fn GetWindow(hWnd: HWND, uCmd: UINT) -> HWND;
    fn GetTopWindow(hWnd: HWND) -> HWND;
    fn GetDesktopWindow() -> HWND;
    fn GetActiveWindow() -> HWND;
    fn SetForegroundWindow(hWnd: HWND) -> BOOL;
    fn FindWindowW(lpClassName: LPCWSTR, lpWindowName: LPCWSTR) -> HWND;
    fn IsZoomed(hwnd: HWND) -> BOOL;
    fn IsIconic(hwnd: HWND) -> BOOL;
    fn IsWindow(hwnd: HWND) -> BOOL;
    fn IsWindowVisible(hWnd: HWND) -> BOOL;
    fn IsWindowEnabled(hWnd: HWND) -> BOOL;
    fn EnableWindow(hWnd: HWND, bEnable: BOOL) -> BOOL;
    fn EnumChildWindows(hWndParent: HWND, lpEnumFunc: WNDENUMPROC, lParam: LPARAM) -> BOOL;
    fn ChildWindowFromPoint(hWndParent: HWND, Point: POINT) -> HWND;
    fn ArrangeIconicWindows(hWnd: HWND) -> UINT;
    fn BeginDeferWindowPos(nNumWindows: INT) -> HDWP;
    fn DeferWindowPos(hWinPosInfo: HDWP, hWnd: HWND, hWndInsertAfter: HWND, x: INT, y: INT, cx: INT, cy: INT, uFlags: UINT) -> HDWP;
    fn GetSystemMetrics(nIndex: INT) -> INT;
    fn GetKeyState(nVirtKey: INT) -> SHORT;
    fn GetWindowRect(hWnd: HWND, lpRect: LPRECT) -> BOOL;
    fn GetWindowInfo(hwnd: HWND, pwi: LPWINDOWINFO) -> BOOL;
    fn GetClientRect(hWnd: HWND, lpRect: LPRECT) -> BOOL;
    fn AdjustWindowRectEx(lpRect: LPRECT, dwStyle: DWORD, bMenu: BOOL, dwExStyle: DWORD) -> BOOL;
    fn SetRect(lprc: LPRECT, xLeft: INT, yTop: INT, xRight: INT, yBottom: INT) -> BOOL;
    fn SetRectEmpty(lprc: LPRECT) -> BOOL;
    fn OffsetRect(lprc: LPRECT, dx: INT, dy: INT) -> BOOL;
    fn UnionRect(lprcDst: LPRECT, lprcSrc1: *const RECT, lprcSrc2: *const RECT) -> BOOL;
    fn GetWindowTextW(hWnd: HWND, lpString: LPWSTR, nMaxCount: INT) -> INT;
    fn InternalGetWindowText(hWnd: HWND, pString: LPWSTR, cchMaxCount: INT) -> INT;
    fn lstrcpynW(lpString1: LPWSTR, lpString2: LPCWSTR, iMaxLength: INT) -> LPWSTR;
    fn CreateWindowExW(dwExStyle: DWORD, lpClassName: LPCWSTR, lpWindowName: LPCWSTR, dwStyle: DWORD, x: INT, y: INT, nWidth: INT, nHeight: INT, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID) -> HWND;
    fn CreateWindowExA(dwExStyle: DWORD, lpClassName: LPCSTR, lpWindowName: LPCSTR, dwStyle: DWORD, x: INT, y: INT, nWidth: INT, nHeight: INT, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID) -> HWND;
    fn GetSystemMenu(hWnd: HWND, bRevert: BOOL) -> HMENU;
    fn SetMenu(hWnd: HWND, hMenu: HMENU) -> BOOL;
    fn GetMenu(hWnd: HWND) -> HMENU;
    fn GetSubMenu(hMenu: HMENU, nPos: INT) -> HMENU;
    fn IsMenu(hMenu: HMENU) -> BOOL;
    fn GetMenuItemCount(hMenu: HMENU) -> INT;
    fn GetMenuItemID(hMenu: HMENU, nPos: INT) -> UINT;
    fn GetMenuState(hMenu: HMENU, uId: UINT, uFlags: UINT) -> UINT;
    fn GetMenuItemInfoW(hmenu: HMENU, item: UINT, fByPosition: BOOL, lpmii: *mut MENUITEMINFOW) -> BOOL;
    fn AppendMenuW(hMenu: HMENU, uFlags: UINT, uIDNewItem: UINT_PTR, lpNewItem: LPCWSTR) -> BOOL;
    fn InsertMenuA(hMenu: HMENU, uPosition: UINT, uFlags: UINT, uIDNewItem: UINT_PTR, lpNewItem: LPCSTR) -> BOOL;
    fn EnableMenuItem(hMenu: HMENU, uIDEnableItem: UINT, uEnable: UINT) -> BOOL;
    fn CheckMenuItem(hMenu: HMENU, uIDCheckItem: UINT, uCheck: UINT) -> DWORD;
    fn DrawMenuBar(hWnd: HWND) -> BOOL;
    fn LoadStringW(hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: INT) -> INT;
    fn LoadBitmapW(hInstance: HINSTANCE, lpBitmapName: LPCWSTR) -> HBITMAP;
    fn LoadImageW(hInst: HINSTANCE, name: LPCWSTR, type_: UINT, cx: INT, cy: INT, fuLoad: UINT) -> HANDLE;
    fn DrawIconEx(hdc: HDC, xLeft: INT, yTop: INT, hIcon: HICON, cxWidth: INT, cyWidth: INT, istepIfAniCur: UINT, hbrFlickerFreeDraw: HBRUSH, diFlags: UINT) -> BOOL;
    fn GetSysColorBrush(nIndex: INT) -> HBRUSH;
    fn GetDlgItem(hDlg: HWND, nIDDlgItem: INT) -> HWND;
    fn EndDialog(hDlg: HWND, nResult: INT_PTR) -> BOOL;
    fn DialogBoxIndirectParamA(hInstance: HINSTANCE, hDialogTemplate: LPCVOID, hWndParent: HWND, lpDialogFunc: *mut c_void, dwInitParam: LPARAM) -> INT_PTR;
    fn MonitorFromPoint(pt: POINT, dwFlags: DWORD) -> HMONITOR;
    fn GetMonitorInfoW(hMonitor: HMONITOR, lpmi: LPMONITORINFO) -> BOOL;
    fn GetScrollPos(hWnd: HWND, nBar: INT) -> INT;
    fn GetScrollRange(hWnd: HWND, nBar: INT, lpMinPos: LPINT, lpMaxPos: LPINT) -> BOOL;
    fn SetScrollPos(hWnd: HWND, nBar: INT, nPos: INT, bRedraw: BOOL) -> INT;
    fn ScrollWindowEx(hWnd: HWND, dx: INT, dy: INT, prcScroll: LPCRECT, prcClip: LPCRECT, hrgnUpdate: HRGN, prcUpdate: LPRECT, flags: UINT) -> INT;
}

// ---- file-scope preamble (translated) ----
pub const MDI_MAXTITLELENGTH: UINT = 0xa1;

pub const WM_MDICALCCHILDSCROLL: UINT = 0x003F;

pub const MDI_MOREWINDOWSLIMIT: UINT = 9;

pub const MDI_IDC_LISTBOX: UINT = 100;

pub const IDS_MDI_MOREWINDOWS: UINT = 13;

pub const MDIF_NEEDUPDATE: UINT = 0x0001;

#[repr(C)]
pub struct MDICLIENTINFO {
    pub reserved: LONG,
    pub nActiveChildren: UINT,
    pub hwndChildMaximized: HWND,
    pub hwndActiveChild: HWND,
    pub child: *mut HWND,
    pub hFrameMenu: HMENU,
    pub hWindowMenu: HMENU,
    pub idFirstChild: UINT,
    pub frameTitle: LPWSTR,
    pub nTotalCreated: UINT,
    pub mdiFlags: UINT,
    pub sbRecalc: UINT,
    pub hBmpClose: HBITMAP,
}

#[repr(C)]
pub struct CASCADE_INFO {
    pub hwndTop: HWND,
    pub wFlags: UINT,
    pub hwndParent: HWND,
    pub hwndDesktop: HWND,
    pub hTrayWnd: HWND,
    pub hwndProgman: HWND,
    pub ahwnd: *mut HWND,
    pub chwnd: DWORD,
}

#[no_mangle]
pub unsafe extern "C" fn WIN_ListChildren(hWndparent: HWND) -> *mut HWND {
    let mut dwCount: DWORD = 0;
    let mut pHwnd: *mut HWND;
    let hHeap: HANDLE;
    let mut Status: NTSTATUS;

    Status = NtUserBuildHwndList(
        core::ptr::null_mut(),
        hWndparent,
        0u8,
        0,
        dwCount,
        core::ptr::null_mut(),
        &mut dwCount,
    );

    if !NT_SUCCESS(Status) {
        return core::ptr::null_mut();
    }

    hHeap = GetProcessHeap();

    pHwnd = HeapAlloc(
        hHeap,
        0,
        (core::mem::size_of::<HWND>() as DWORD * (dwCount + 1)) as _,
    ) as *mut HWND;
    if pHwnd.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return core::ptr::null_mut();
    }

    Status = NtUserBuildHwndList(
        core::ptr::null_mut(),
        hWndparent,
        0u8,
        0,
        dwCount,
        pHwnd,
        &mut dwCount,
    );

    if !NT_SUCCESS(Status) {
        if !pHwnd.is_null() {
            HeapFree(hHeap, 0, pHwnd as LPVOID);
        }
        return core::ptr::null_mut();
    }

    *pHwnd.offset(dwCount as isize) = 0 as HWND;

    pHwnd
}

unsafe fn MDI_GetChildByID(hwnd: HWND, id: UINT, ci: *mut MDICLIENTINFO) -> HWND {
    let mut i: i32 = 0;
    while (*ci).nActiveChildren != 0 {
        if GetWindowLongPtrW(*(*ci).child.offset(i as isize), GWLP_ID) == id as LONG_PTR {
            return *(*ci).child.offset(i as isize);
        }
        i += 1;
    }
    core::ptr::null_mut()
}

unsafe fn MDI_PostUpdate(hwnd: HWND, ci: *mut MDICLIENTINFO, recalc: WORD) {
    if ((*ci).mdiFlags & MDIF_NEEDUPDATE) == 0 {
        (*ci).mdiFlags |= MDIF_NEEDUPDATE;
        PostMessageA(hwnd, WM_MDICALCCHILDSCROLL, 0, 0);
    }
    (*ci).sbRecalc = recalc as UINT;
}

unsafe fn get_client_info(client: HWND) -> *mut MDICLIENTINFO {
    GetWindowLongPtr(client, GWLP_MDIWND as INT) as *mut MDICLIENTINFO
}

unsafe fn is_close_enabled(hwnd: HWND, mut hSysMenu: HMENU) -> BOOL {
    if GetClassLongPtrW(hwnd, GCL_STYLE as _) & CS_NOCLOSE as DWORD_PTR != 0 {
        return 0;
    }

    if hSysMenu.is_null() {
        hSysMenu = GetSystemMenu(hwnd, 0);
    }
    if !hSysMenu.is_null() {
        let state: UINT = GetMenuState(hSysMenu, SC_CLOSE as UINT, MF_BYCOMMAND as UINT);
        if state == 0xFFFFFFFF || (state & (MF_DISABLED as UINT | MF_GRAYED as UINT)) != 0 {
            return 0;
        }
    }
    1
}

unsafe fn MDI_GetWindow(clientInfo: *mut MDICLIENTINFO, mut hWnd: HWND, bNext: BOOL, mut dwStyleMask: DWORD) -> HWND {
    let mut i: i32;
    let list: *mut HWND;
    let mut last: HWND = core::ptr::null_mut();

    dwStyleMask |= (WS_DISABLED | WS_VISIBLE) as DWORD;
    if hWnd.is_null() {
        hWnd = (*clientInfo).hwndActiveChild;
    }

    list = WIN_ListChildren(GetParent(hWnd));
    if list.is_null() {
        return core::ptr::null_mut();
    }

    i = 0;
    /* start from next after hWnd */
    while !(*list.offset(i as isize)).is_null() && *list.offset(i as isize) != hWnd {
        i += 1;
    }
    if !(*list.offset(i as isize)).is_null() {
        i += 1;
    }

    'found: loop {
        while !(*list.offset(i as isize)).is_null() {
            let cur = *list.offset(i as isize);
            if !GetWindow(cur, GW_OWNER as UINT).is_null() {
                i += 1;
                continue;
            }
            if (GetWindowLongPtrW(cur, GWL_STYLE) as DWORD & dwStyleMask) != WS_VISIBLE as DWORD {
                i += 1;
                continue;
            }
            last = cur;
            if bNext != 0 {
                break 'found;
            }
            i += 1;
        }
        /* now restart from the beginning */
        i = 0;
        while !(*list.offset(i as isize)).is_null() && *list.offset(i as isize) != hWnd {
            let cur = *list.offset(i as isize);
            if !GetWindow(cur, GW_OWNER as UINT).is_null() {
                i += 1;
                continue;
            }
            if (GetWindowLongPtrW(cur, GWL_STYLE) as DWORD & dwStyleMask) != WS_VISIBLE as DWORD {
                i += 1;
                continue;
            }
            last = cur;
            if bNext != 0 {
                break 'found;
            }
            i += 1;
        }
        break;
    }

    HeapFree(GetProcessHeap(), 0, list as LPVOID);
    last
}

#[no_mangle]
pub unsafe extern "C" fn MDI_CalcDefaultChildPos(hwndClient: HWND, mut total: INT, lpPos: LPPOINT, delta: INT, id: *mut UINT) {
    let nstagger: INT;
    let mut rect: RECT = core::mem::zeroed();
    let spacing: INT = GetSystemMetrics(SM_CYCAPTION as i32) + GetSystemMetrics(SM_CYFRAME as i32) - 1;

    if total < 0 {
        let ci: *mut MDICLIENTINFO = get_client_info(hwndClient);
        total = if !ci.is_null() { (*ci).nTotalCreated as INT } else { 0 };
        *id = if !ci.is_null() { (*ci).idFirstChild + (*ci).nActiveChildren } else { 0 };
        TRACE!(b"MDI child id %04x\n\0".as_ptr() as LPCSTR, *id);
    }

    GetClientRect(hwndClient, &mut rect);
    if rect.bottom - rect.top - delta >= spacing {
        rect.bottom -= delta;
    }

    nstagger = (rect.bottom - rect.top) / (3 * spacing);
    (*lpPos.offset(1)).x = rect.right - rect.left - nstagger * spacing;
    (*lpPos.offset(1)).y = rect.bottom - rect.top - nstagger * spacing;
    (*lpPos.offset(0)).x = spacing * (total % (nstagger + 1));
    (*lpPos.offset(0)).y = (*lpPos.offset(0)).x;
}

unsafe fn MDISetMenu(hwnd: HWND, hmenuFrame: HMENU, hmenuWindow: HMENU) -> LRESULT {
    let ci: *mut MDICLIENTINFO;
    let hwndFrame = GetParent(hwnd);

    if !hmenuFrame.is_null() && IsMenu(hmenuFrame) == 0 {
        return 0;
    }

    if !hmenuWindow.is_null() && IsMenu(hmenuWindow) == 0 {
        return 0;
    }

    ci = get_client_info(hwnd);
    if ci.is_null() {
        return 0;
    }

    if !hmenuFrame.is_null() {
        if hmenuFrame == (*ci).hFrameMenu {
            return hmenuFrame as LRESULT;
        }

        if !(*ci).hwndChildMaximized.is_null() {
            MDI_RestoreFrameMenu(hwndFrame, (*ci).hwndChildMaximized, (*ci).hBmpClose);
        }
    }

    if !hmenuWindow.is_null() && hmenuWindow != (*ci).hWindowMenu {
        if !(*ci).hWindowMenu.is_null() && (*ci).nActiveChildren != 0 {
            let nActiveChildren_old = (*ci).nActiveChildren;

            (*ci).nActiveChildren = 0;
            MDI_RefreshMenu(ci);

            (*ci).hWindowMenu = hmenuWindow;

            (*ci).nActiveChildren = nActiveChildren_old;
            MDI_RefreshMenu(ci);
        } else {
            (*ci).hWindowMenu = hmenuWindow;
        }
    }

    if !hmenuFrame.is_null() {
        SetMenu(hwndFrame, hmenuFrame);
        if hmenuFrame != (*ci).hFrameMenu {
            let oldFrameMenu = (*ci).hFrameMenu;

            (*ci).hFrameMenu = hmenuFrame;
            if !(*ci).hwndChildMaximized.is_null() {
                MDI_AugmentFrameMenu(hwndFrame, (*ci).hwndChildMaximized);
            }

            return oldFrameMenu as LRESULT;
        }
    }

    0
}

unsafe fn MDI_RefreshMenu(ci: *mut MDICLIENTINFO) -> LRESULT {
    let mut buf: [u16; MDI_MAXTITLELENGTH as usize] = [0; MDI_MAXTITLELENGTH as usize];

    if (*ci).hWindowMenu.is_null() {
        return 0;
    }

    if IsMenu((*ci).hWindowMenu) == 0 {
        return 0;
    }

    let count = GetMenuItemCount((*ci).hWindowMenu) as UINT;
    let mut i: UINT = 0;
    while i < count {
        let mut mii: MENUITEMINFOW = core::mem::zeroed();
        mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as u32;
        mii.fMask = MIIM_TYPE;
        if GetMenuItemInfoW((*ci).hWindowMenu, i, 1u8 as i32, &mut mii) != 0 {
            if mii.fType & MF_SEPARATOR != 0 {
                let mut mii2: MENUITEMINFOW = core::mem::zeroed();
                mii2.cbSize = core::mem::size_of::<MENUITEMINFOW>() as u32;
                mii2.fMask = MIIM_ID;
                if GetMenuItemInfoW((*ci).hWindowMenu, i + 1, 1u8 as i32, &mut mii2) != 0 {
                    if mii2.wID == (*ci).idFirstChild {
                        while RemoveMenu((*ci).hWindowMenu, i, MF_BYPOSITION) != 0 {}
                        break;
                    }
                }
            }
        }
        i += 1;
    }

    let mut visible: UINT = 0;
    let mut i: UINT = 0;
    while i < (*ci).nActiveChildren {
        if GetWindowLongPtrW(*(*ci).child.offset(i as isize), GWL_STYLE as INT) as DWORD_PTR & (WS_VISIBLE as DWORD_PTR as i64 as isize as DWORD_PTR) != 0 {
            let id = (*ci).idFirstChild + visible;

            if visible == MDI_MOREWINDOWSLIMIT {
                LoadStringW(User32Instance, IDS_MDI_MOREWINDOWS as UINT, buf.as_mut_ptr(), (core::mem::size_of_val(&buf) / core::mem::size_of::<u16>()) as i32);
                AppendMenuW((*ci).hWindowMenu, MF_STRING, id as UINT_PTR, buf.as_ptr() as LPCWSTR);
                break;
            }

            if visible == 0 {
                AppendMenuW((*ci).hWindowMenu, MF_SEPARATOR, 0, core::ptr::null_mut());
            }

            visible += 1;

            SetWindowLongPtrW(*(*ci).child.offset(i as isize), GWLP_ID as INT, id as LONG_PTR);

            buf[0] = b'&' as u16;
            buf[1] = b'0' as u16 + visible as u16;
            buf[2] = b' ' as u16;
            InternalGetWindowText(*(*ci).child.offset(i as isize), buf.as_mut_ptr().offset(3), (core::mem::size_of_val(&buf) / core::mem::size_of::<u16>()) as i32 - 3);
            AppendMenuW((*ci).hWindowMenu, MF_STRING, id as UINT_PTR, buf.as_ptr() as LPCWSTR);

            if *(*ci).child.offset(i as isize) == (*ci).hwndActiveChild {
                CheckMenuItem((*ci).hWindowMenu, id, MF_CHECKED);
            }
        }
        i += 1;
    }

    (*ci).hFrameMenu as LRESULT
}

unsafe fn MDI_ChildGetMinMaxInfo( client: HWND, hwnd: HWND, lpMinMax: *mut MINMAXINFO )
{
    let mut rect: RECT = core::mem::zeroed();

    GetClientRect( client, &mut rect );
    AdjustWindowRectEx( &mut rect, GetWindowLongPtrW( hwnd, GWL_STYLE as i32 ) as DWORD,
                        0, GetWindowLongPtrW( hwnd, GWL_EXSTYLE as i32 ) as DWORD);

    rect.right -= rect.left;
    (*lpMinMax).ptMaxSize.x = rect.right;
    rect.bottom -= rect.top;
    (*lpMinMax).ptMaxSize.y = rect.bottom;

    (*lpMinMax).ptMaxPosition.x = rect.left;
    (*lpMinMax).ptMaxPosition.y = rect.top;
}

unsafe fn MDI_SwitchActiveChild(ci: *mut MDICLIENTINFO, hwndTo: HWND, activate: BOOL) {
    let hwndPrev: HWND = (*ci).hwndActiveChild;

    TRACE!(b"from %p, to %p\n\0".as_ptr() as *const i8, hwndPrev, hwndTo);

    if hwndTo != hwndPrev {
        let was_zoomed = IsZoomed(hwndPrev);

        if was_zoomed != 0 {
            SendMessageW(hwndPrev, WM_SETREDRAW, 0 as WPARAM, 0 as LPARAM);
            ShowWindow(hwndPrev, SW_RESTORE as i32);
            SendMessageW(hwndPrev, WM_SETREDRAW, 1 as WPARAM, 0 as LPARAM);

            SetWindowPos(hwndTo, HWND_TOP, 0, 0, 0, 0, (SWP_NOMOVE | SWP_NOSIZE) as u32);
            ShowWindow(hwndTo, SW_MAXIMIZE as i32);
        }
        SetWindowPos(hwndTo, HWND_TOP, 0, 0, 0, 0,
            (SWP_NOMOVE | SWP_NOSIZE | (if activate != 0 { 0 } else { SWP_NOACTIVATE })) as u32);
    }
}

unsafe fn MDIDestroyChild(client: HWND, ci: *mut MDICLIENTINFO, child: HWND, flagDestroy: BOOL) -> LRESULT {
    let mut i: UINT;

    if child == (*ci).hwndActiveChild {
        let next = MDI_GetWindow(ci, child, 1, 0);
        if !next.is_null() {
            MDI_SwitchActiveChild(ci, next, 1);
        } else {
            ShowWindow(child, SW_HIDE as i32);
            if child == (*ci).hwndChildMaximized {
                let frame = GetParent(client);
                MDI_RestoreFrameMenu(frame, child, (*ci).hBmpClose);
                (*ci).hwndChildMaximized = core::ptr::null_mut();
                MDI_UpdateFrameText(frame, client, 1, core::ptr::null_mut());
            }
            if flagDestroy != 0 {
                MDI_ChildActivate(client, core::ptr::null_mut());
            }
        }
    }

    i = 0;
    while i < (*ci).nActiveChildren {
        if *(*ci).child.offset(i as isize) == child {
            let new_child = HeapAlloc(GetProcessHeap(), 0, (((*ci).nActiveChildren - 1) as usize * core::mem::size_of::<HWND>()) as SIZE_T) as *mut HWND;
            if !new_child.is_null() {
                memcpy(new_child as *mut _, (*ci).child as *const _, i as usize * core::mem::size_of::<HWND>());
                if i + 1 < (*ci).nActiveChildren {
                    memcpy(new_child.offset(i as isize) as *mut _, (*ci).child.offset((i + 1) as isize) as *const _, ((*ci).nActiveChildren - i - 1) as usize * core::mem::size_of::<HWND>());
                }
                HeapFree(GetProcessHeap(), 0, (*ci).child as *mut _);
                (*ci).child = new_child;
            } else {
                let mut c: UINT = i;
                while c < (*ci).nActiveChildren - 1 {
                    *(*ci).child.offset(c as isize) = *(*ci).child.offset((c + 1) as isize);
                    c += 1;
                }
            }

            (*ci).nActiveChildren -= 1;
            break;
        }
        i += 1;
    }

    if flagDestroy != 0 {
        SendMessageW(client, WM_MDIREFRESHMENU, 0, 0);
        MDI_PostUpdate(GetParent(child), ci, (SB_BOTH + 1) as WORD);
        DestroyWindow(child);
    }

    0
}

unsafe fn MDI_ChildActivate(client: HWND, child: HWND) -> LONG {
    let clientInfo: *mut MDICLIENTINFO;
    let prevActiveWnd: HWND;
    let frame: HWND;
    let isActiveFrameWnd: BOOL;

    clientInfo = get_client_info(client);

    if (*clientInfo).hwndActiveChild == child {
        return 0;
    }

    TRACE!(b"%p\n\0".as_ptr() as *const i8, child);

    frame = GetParent(client);
    isActiveFrameWnd = (GetActiveWindow() == frame) as BOOL;
    prevActiveWnd = (*clientInfo).hwndActiveChild;

    if !prevActiveWnd.is_null() {
        SendMessageW(prevActiveWnd, WM_NCACTIVATE, 0 as WPARAM, 0 as LPARAM);
        SendMessageW(prevActiveWnd, WM_MDIACTIVATE, prevActiveWnd as WPARAM, child as LPARAM);
    }

    MDI_SwitchActiveChild(clientInfo, child, 0 as BOOL);
    (*clientInfo).hwndActiveChild = child;

    MDI_RefreshMenu(clientInfo);

    if isActiveFrameWnd != 0 {
        SendMessageW(child, WM_NCACTIVATE, 1 as WPARAM, 0 as LPARAM);
        if SetFocus(client) == client {
            SendMessageW(client, WM_SETFOCUS, client as WPARAM, 0 as LPARAM);
        }
    }

    SendMessageW(child, WM_MDIACTIVATE, prevActiveWnd as WPARAM, child as LPARAM);
    1
}

unsafe fn CreateMDIMenuBitmap() -> HBITMAP {
    let hDCSrc: HDC = CreateCompatibleDC(0 as HDC);
    let hDCDest: HDC = CreateCompatibleDC(hDCSrc);
    let hbClose: HBITMAP = LoadBitmapW(0 as HINSTANCE, MAKEINTRESOURCEW(OBM_OLD_CLOSE as u16));
    let hbCopy: HBITMAP;
    let mut hobjSrc;
    let hobjDest;

    hobjSrc = SelectObject(hDCSrc, hbClose as LPVOID);
    hbCopy = CreateCompatibleBitmap(hDCSrc, GetSystemMetrics(SM_CXSIZE as i32), GetSystemMetrics(SM_CYSIZE as i32));
    hobjDest = SelectObject(hDCDest, hbCopy as LPVOID);

    BitBlt(hDCDest, 0, 0, GetSystemMetrics(SM_CXSIZE as i32), GetSystemMetrics(SM_CYSIZE as i32),
           hDCSrc, GetSystemMetrics(SM_CXSIZE as i32), 0, SRCCOPY);

    SelectObject(hDCSrc, hobjSrc);
    DeleteObject(hbClose as LPVOID);
    DeleteDC(hDCSrc);

    hobjSrc = SelectObject(hDCDest, GetStockObject(BLACK_PEN as INT) as HGDIOBJ);

    MoveToEx(hDCDest, GetSystemMetrics(SM_CXSIZE as i32) - 1, 0, core::ptr::null_mut());
    LineTo(hDCDest, GetSystemMetrics(SM_CXSIZE as i32) - 1, GetSystemMetrics(SM_CYSIZE as i32) - 1);

    SelectObject(hDCDest, hobjSrc);
    SelectObject(hDCDest, hobjDest);
    DeleteDC(hDCDest);

    hbCopy
}

unsafe extern "system" fn MDICascade(client: HWND, ci: *mut MDICLIENTINFO) -> LONG {
    let mut win_array: *mut HWND;
    let mut has_icons: BOOL = 0u8 as BOOL;
    let mut i: INT;
    let mut total: INT;

    if !(*ci).hwndChildMaximized.is_null() {
        SendMessageW(client, WM_MDIRESTORE, (*ci).hwndChildMaximized as WPARAM, 0);
    }

    if (*ci).nActiveChildren == 0 {
        return 0;
    }

    win_array = WIN_ListChildren(client);
    if win_array.is_null() {
        return 0;
    }

    /* remove all the windows we don't want */
    i = 0;
    total = 0;
    while !(*win_array.offset(i as isize)).is_null() {
        if IsWindowVisible(*win_array.offset(i as isize)) == 0 {
            i += 1;
            continue;
        }
        if !GetWindow(*win_array.offset(i as isize), GW_OWNER as UINT).is_null() {
            i += 1;
            continue;
        }
        if IsIconic(*win_array.offset(i as isize)) != 0 {
            has_icons = 1u8 as BOOL;
            i += 1;
            continue;
        }
        *win_array.offset(total as isize) = *win_array.offset(i as isize);
        total += 1;
        i += 1;
    }
    *win_array.offset(total as isize) = core::ptr::null_mut();

    if total != 0 {
        let mut delta: INT = 0;
        let mut n: INT = 0;
        let mut i: INT;
        let mut pos: [POINT; 2] = [core::mem::zeroed(); 2];
        if has_icons != 0 {
            delta = GetSystemMetrics(SM_CYICONSPACING as INT) + GetSystemMetrics(SM_CYICON as INT);
        }

        /* walk the list (backwards) and move windows */
        i = total - 1;
        while i >= 0 {
            let style: LONG;
            let mut posOptions: LONG = (SWP_DRAWFRAME | SWP_NOACTIVATE | SWP_NOZORDER) as LONG;

            MDI_CalcDefaultChildPos(client, n, pos.as_mut_ptr(), delta, core::ptr::null_mut());
            n += 1;
            TRACE!(
                b"move %p to (%ld,%ld) size [%ld,%ld]\n\0".as_ptr() as LPCSTR,
                *win_array.offset(i as isize),
                pos[0].x,
                pos[0].y,
                pos[1].x,
                pos[1].y,
            );
            style = GetWindowLongW(*win_array.offset(i as isize), GWL_STYLE as INT);

            if (style & WS_SIZEBOX as LONG) == 0 {
                posOptions |= SWP_NOSIZE as LONG;
            }
            SetWindowPos(
                *win_array.offset(i as isize),
                core::ptr::null_mut(),
                pos[0].x,
                pos[0].y,
                pos[1].x,
                pos[1].y,
                posOptions as UINT,
            );
            i -= 1;
        }
    }
    HeapFree(GetProcessHeap(), 0, win_array as LPVOID);

    if has_icons != 0 {
        ArrangeIconicWindows(client);
    }
    0
}

unsafe fn MDITile(client: HWND, ci: *mut MDICLIENTINFO, wParam: WPARAM) {
    let win_array: *mut HWND;
    let mut i: i32;
    let mut total: i32;
    let mut rows: i32;
    let mut columns: i32;
    let mut has_icons: BOOL = 0u8 as BOOL;

    if !(*ci).hwndChildMaximized.is_null() {
        SendMessageW(client, WM_MDIRESTORE, (*ci).hwndChildMaximized as WPARAM, 0);
    }

    if (*ci).nActiveChildren == 0 {
        return;
    }

    win_array = WIN_ListChildren(client);
    if win_array.is_null() {
        return;
    }

    /* remove all the windows we don't want */
    i = 0;
    total = 0;
    rows = 0;
    while !(*win_array.offset(i as isize)).is_null() {
        let w = *win_array.offset(i as isize);
        if IsWindowVisible(w) == 0 {
            i += 1;
            continue;
        }
        if !GetWindow(w, GW_OWNER as u32).is_null() {
            i += 1;
            continue;
        }
        if IsIconic(w) != 0 {
            has_icons = 1u8 as BOOL;
            i += 1;
            continue;
        }
        if (wParam & MDITILE_SKIPDISABLED as WPARAM) != 0 && IsWindowEnabled(w) == 0 {
            i += 1;
            continue;
        }
        if total == (rows * (rows + 2)) {
            rows += 1;
        }
        *win_array.offset(total as isize) = w;
        total += 1;
        i += 1;
    }
    *win_array.offset(total as isize) = core::ptr::null_mut();

    TRACE!(b"%u windows to tile\n\0".as_ptr() as LPCSTR, total);

    if total != 0 {
        let mut pWnd: *mut HWND = win_array;
        let mut rect: RECT = core::mem::zeroed();
        let mut x: i32;
        let mut y: i32;
        let xsize: i32;
        let ysize: i32;
        let mut r: i32;
        let mut c: i32;
        let mut ii: i32;
        let mut ysize_var: i32;

        GetClientRect(client, &mut rect);
        columns = total / rows;

        if (wParam & MDITILE_HORIZONTAL as WPARAM) != 0 {
            let tmp = rows;
            rows = columns;
            columns = tmp;
        }

        if has_icons != 0 {
            y = rect.bottom - 2 * GetSystemMetrics(SM_CYICONSPACING as i32) - GetSystemMetrics(SM_CYICON as i32);
            rect.bottom = if y - GetSystemMetrics(SM_CYICON as i32) < rect.top { rect.bottom } else { y };
        }

        ysize = rect.bottom / rows;
        xsize = rect.right / columns;
        ysize_var = ysize;
        let _ = ysize_var;

        x = 0;
        ii = 0;
        c = 1;
        let mut ysize_cur = ysize;
        while c <= columns && !(*pWnd).is_null() {
            if c == columns {
                rows = total - ii;
                ysize_cur = rect.bottom / rows;
            }

            y = 0;
            r = 1;
            while r <= rows && !(*pWnd).is_null() {
                let mut posOptions: i32 = (SWP_DRAWFRAME | SWP_NOACTIVATE | SWP_NOZORDER) as i32;
                let style: i32 = GetWindowLongW(*win_array.offset(ii as isize), GWL_STYLE as i32);
                if (style & WS_SIZEBOX as i32) == 0 {
                    posOptions |= SWP_NOSIZE as i32;
                }

                SetWindowPos(*pWnd, core::ptr::null_mut(), x, y, xsize, ysize_cur, posOptions as u32);
                y += ysize_cur;
                pWnd = pWnd.offset(1);
                r += 1;
                ii += 1;
            }
            x += xsize;
            c += 1;
        }
    }

    HeapFree(GetProcessHeap(), 0, win_array as LPVOID);
    if has_icons != 0 {
        ArrangeIconicWindows(client);
    }
}

unsafe fn MDI_AugmentFrameMenu(frame: HWND, hChild: HWND) -> BOOL {
    let menu: HMENU = GetMenu(frame);
    let hSysPopup: HMENU;
    let mut hSysMenuBitmap: HBITMAP = 0 as HBITMAP;
    let mut hIcon: HICON;
    let nItems: INT;
    let iId: UINT;

    if menu.is_null() {
        return 0u8 as BOOL;
    }

    nItems = GetMenuItemCount(menu) - 1;
    iId = GetMenuItemID(menu, nItems);
    if iId == SC_RESTORE || iId == SC_CLOSE {
        return 0u8 as BOOL;
    }

    hSysPopup = GetSystemMenu(hChild, 0u8 as BOOL);
    if hSysPopup.is_null() {
        return 0u8 as BOOL;
    }

    AppendMenuW(menu, MF_HELP | MF_BITMAP, SC_CLOSE as UINT_PTR,
        if is_close_enabled(hChild, hSysPopup) != 0 {
            HBMMENU_MBAR_CLOSE as LPCWSTR
        } else {
            HBMMENU_MBAR_CLOSE_D as LPCWSTR
        });
    AppendMenuW(menu, MF_HELP | MF_BITMAP, SC_RESTORE as UINT_PTR, HBMMENU_MBAR_RESTORE as LPCWSTR);
    AppendMenuW(menu, MF_HELP | MF_BITMAP, SC_MINIMIZE as UINT_PTR, HBMMENU_MBAR_MINIMIZE as LPCWSTR);

    hIcon = SendMessageW(hChild, WM_GETICON, ICON_SMALL as WPARAM, 0 as LPARAM) as HICON;
    if hIcon.is_null() {
        hIcon = GetClassLongPtrW(hChild, GCLP_HICONSM) as HICON;
    }
    if hIcon.is_null() {
        hIcon = SendMessageW(hChild, WM_GETICON, ICON_BIG as WPARAM, 0 as LPARAM) as HICON;
    }
    if hIcon.is_null() {
        hIcon = GetClassLongPtrW(hChild, GCLP_HICON) as HICON;
    }
    if hIcon.is_null() {
        hIcon = LoadImageW(0 as HINSTANCE, IDI_WINLOGO as LPWSTR, IMAGE_ICON,
            GetSystemMetrics(SM_CXSMICON as i32), GetSystemMetrics(SM_CYSMICON as i32), LR_DEFAULTCOLOR) as HICON;
    }
    if !hIcon.is_null() {
        let hMemDC: HDC;
        let hBitmap: HBITMAP;
        let hOldBitmap: HBITMAP;
        let hdc: HDC = GetDC(hChild);

        if !hdc.is_null() {
            let cx: INT;
            let cy: INT;
            cx = GetSystemMetrics(SM_CXSMICON as i32);
            cy = GetSystemMetrics(SM_CYSMICON as i32);
            hMemDC = CreateCompatibleDC(hdc);
            hBitmap = CreateCompatibleBitmap(hdc, cx, cy);
            hOldBitmap = SelectObject(hMemDC, hBitmap as HGDIOBJ) as HBITMAP;
            SetMapMode(hMemDC, MM_TEXT as INT);
            DrawIconEx(hMemDC, 0, 0, hIcon, cx, cy, 0, GetSysColorBrush(COLOR_MENU as INT), DI_NORMAL);
            SelectObject(hMemDC, hOldBitmap as HGDIOBJ);
            DeleteDC(hMemDC);
            ReleaseDC(hChild, hdc);
            hSysMenuBitmap = hBitmap;
        }
    }

    if InsertMenuA(menu, 0, MF_BYPOSITION | MF_BITMAP | MF_POPUP,
        hSysPopup as UINT_PTR, hSysMenuBitmap as LPSTR) == 0 {
        DestroyMenu(hSysPopup);
        return 0u8 as BOOL;
    }

    EnableMenuItem(hSysPopup, SC_SIZE, MF_BYCOMMAND | MF_GRAYED);
    EnableMenuItem(hSysPopup, SC_MOVE, MF_BYCOMMAND | MF_GRAYED);
    EnableMenuItem(hSysPopup, SC_MAXIMIZE, MF_BYCOMMAND | MF_GRAYED);
    SetMenuDefaultItem(hSysPopup, SC_CLOSE, 0u32);

    DrawMenuBar(frame);

    1u8 as BOOL
}

unsafe fn MDI_RestoreFrameMenu(frame: HWND, hChild: HWND, hBmpClose: HBITMAP) -> BOOL {
    let mut menuInfo: MENUITEMINFOW = core::mem::zeroed();
    let menu: HMENU = GetMenu(frame);
    let nItems: INT;
    let iId: UINT;

    TRACE!(b"frame %p,child %p\n\0".as_ptr() as LPCSTR, frame, hChild);

    if menu.is_null() {
        return 0u8 as BOOL;
    }

    nItems = GetMenuItemCount(menu) - 1;
    iId = GetMenuItemID(menu, nItems);
    if !(iId == SC_RESTORE || iId == SC_CLOSE) {
        ERR!("no system buttons then nothing to do\n");
        return 0u8 as BOOL;
    }

    memset(
        core::ptr::addr_of_mut!(menuInfo) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<MENUITEMINFOW>(),
    );
    menuInfo.cbSize = core::mem::size_of::<MENUITEMINFOW>() as UINT;
    menuInfo.fMask = MIIM_DATA | MIIM_TYPE | MIIM_BITMAP;

    GetMenuItemInfoW(menu, 0, 1u8 as BOOL, core::ptr::addr_of_mut!(menuInfo));

    RemoveMenu(menu, 0, MF_BYPOSITION);

    if (menuInfo.fType & MFT_BITMAP) != 0
        && menuInfo.dwTypeData != core::ptr::null_mut()
        && menuInfo.dwTypeData != hBmpClose as LPWSTR
    {
        DeleteObject(menuInfo.dwTypeData as LPVOID);
    }

    if !menuInfo.hbmpItem.is_null() {
        DeleteObject(menuInfo.hbmpItem as LPVOID);
    }

    DeleteMenu(menu, SC_CLOSE, MF_BYCOMMAND);
    DeleteMenu(menu, SC_RESTORE, MF_BYCOMMAND);
    DeleteMenu(menu, SC_MINIMIZE, MF_BYCOMMAND);

    DrawMenuBar(frame);

    1u8 as BOOL
}

unsafe fn MDI_UpdateFrameText(frame: HWND, hClient: HWND, repaint: BOOL, mut lpTitle: LPCWSTR) {
    let mut lpBuffer: [u16; (MDI_MAXTITLELENGTH + 1) as usize] = [0; (MDI_MAXTITLELENGTH + 1) as usize];
    let ci = get_client_info(hClient);

    TRACE!(b"frameText %s\n\0".as_ptr() as *const i8, debugstr_w(lpTitle));

    if ci.is_null() {
        return;
    }

    if lpTitle.is_null() && (*ci).frameTitle.is_null() {
        GetWindowTextW(frame, lpBuffer.as_mut_ptr(), (lpBuffer.len()) as i32);
        lpTitle = lpBuffer.as_ptr();
    }

    if !lpTitle.is_null() {
        HeapFree(GetProcessHeap(), 0, (*ci).frameTitle as LPVOID);
        (*ci).frameTitle = HeapAlloc(
            GetProcessHeap(),
            0,
            ((strlenW(lpTitle) + 1) as usize * core::mem::size_of::<u16>()) as SIZE_T,
        ) as LPWSTR;
        if !(*ci).frameTitle.is_null() {
            strcpyW((*ci).frameTitle, lpTitle);
        }
    }

    if !(*ci).frameTitle.is_null() {
        if !(*ci).hwndChildMaximized.is_null() {
            let lpBracket: [u16; 5] = [' ' as u16, '-' as u16, ' ' as u16, '[' as u16, 0];
            let lpBracket2: [u16; 2] = [']' as u16, 0];
            let i_frame_text_length = strlenW((*ci).frameTitle);

            lstrcpynW(lpBuffer.as_mut_ptr(), (*ci).frameTitle, MDI_MAXTITLELENGTH as INT);

            if i_frame_text_length + 6 < MDI_MAXTITLELENGTH {
                strcatW(lpBuffer.as_mut_ptr(), lpBracket.as_ptr());
                if GetWindowTextW(
                    (*ci).hwndActiveChild,
                    lpBuffer.as_mut_ptr().add((i_frame_text_length + 4) as usize),
                    (MDI_MAXTITLELENGTH - i_frame_text_length - 5) as INT,
                ) != 0
                {
                    strcatW(lpBuffer.as_mut_ptr(), lpBracket2.as_ptr());
                } else {
                    lpBuffer[i_frame_text_length as usize] = 0;
                }
            }
        } else {
            lstrcpynW(lpBuffer.as_mut_ptr(), (*ci).frameTitle, (MDI_MAXTITLELENGTH + 1) as INT);
        }
    } else {
        lpBuffer[0] = 0;
    }

    DefWindowProcW(frame, WM_SETTEXT, 0, lpBuffer.as_ptr() as LPARAM);

    if repaint != 0 {
        if NtUserCallTwoParam(
            frame as DWORD_PTR,
            DC_ACTIVE as DWORD_PTR,
            TWOPARAM_ROUTINE_REDRAWTITLE,
        ) == 0
        {
            SetWindowPos(
                frame,
                0 as HWND,
                0,
                0,
                0,
                0,
                SWP_FRAMECHANGED | SWP_NOSIZE | SWP_NOMOVE | SWP_NOACTIVATE | SWP_NOZORDER,
            );
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn MDIClientWndProc_common(
    hwnd: HWND,
    message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    unicode: BOOL,
) -> LRESULT {
    let mut ci: *mut MDICLIENTINFO = get_client_info(hwnd);

    if ci.is_null() {
        if message == WM_NCCREATE {
            ci = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (core::mem::size_of::<MDICLIENTINFO>()) as SIZE_T,
            ) as *mut MDICLIENTINFO;
            if ci.is_null() {
                return 0;
            }
            SetWindowLongPtrW(hwnd, GWLP_MDIWND as INT, ci as LONG_PTR);
            (*ci).hBmpClose = 0 as HBITMAP;
            NtUserSetWindowFNID(hwnd, FNID_MDICLIENT as WORD);
        }
        return if unicode != 0 {
            DefWindowProcW(hwnd, message, wParam, lParam)
        } else {
            DefWindowProcA(hwnd, message, wParam, lParam)
        };
    }

    match message {
        WM_CREATE => {
            let cs = lParam as LPCREATESTRUCTA;
            let ccs = (*cs).lpCreateParams as LPCLIENTCREATESTRUCT;

            (*ci).hWindowMenu = (*ccs).hWindowMenu as HMENU;
            (*ci).idFirstChild = (*ccs).idFirstChild as UINT;
            (*ci).hwndChildMaximized = 0 as HWND;
            (*ci).child = core::ptr::null_mut();
            (*ci).nActiveChildren = 0;
            (*ci).nTotalCreated = 0;
            (*ci).frameTitle = core::ptr::null_mut();
            (*ci).mdiFlags = 0;
            (*ci).hFrameMenu = GetMenu((*cs).hwndParent);

            if (*ci).hBmpClose == 0 as HBITMAP {
                (*ci).hBmpClose = CreateMDIMenuBitmap();
            }

            return 0;
        }

        WM_DESTROY => {
            if !(*ci).hwndChildMaximized.is_null() {
                MDI_RestoreFrameMenu(
                    GetParent(hwnd),
                    (*ci).hwndChildMaximized,
                    (*ci).hBmpClose,
                );
            }

            (*ci).nActiveChildren = 0;
            MDI_RefreshMenu(ci);

            HeapFree(GetProcessHeap(), 0, (*ci).child as LPVOID);
            HeapFree(GetProcessHeap(), 0, (*ci).frameTitle as LPVOID);
            HeapFree(GetProcessHeap(), 0, ci as LPVOID);
            SetWindowLongPtrW(hwnd, GWLP_MDIWND as INT, 0);
            return 0;
        }

        WM_NCDESTROY => {
            NtUserSetWindowFNID(hwnd, FNID_DESTROY as WORD);
            return 0;
        }

        WM_MDIACTIVATE => {
            if (*ci).hwndActiveChild != wParam as HWND {
                SetWindowPos(
                    wParam as HWND,
                    0 as HWND,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOMOVE,
                );
            }
            return 0;
        }

        WM_MDICASCADE => {
            return MDICascade(hwnd, ci) as LRESULT;
        }

        WM_MDICREATE => {
            if lParam != 0 {
                let child: HWND;
                if unicode != 0 {
                    let csW = lParam as *mut MDICREATESTRUCTW;
                    child = CreateWindowExW(
                        WS_EX_MDICHILD,
                        (*csW).szClass,
                        (*csW).szTitle,
                        (*csW).style,
                        (*csW).x,
                        (*csW).y,
                        (*csW).cx,
                        (*csW).cy,
                        hwnd,
                        0 as HMENU,
                        (*csW).hOwner as HINSTANCE,
                        (*csW).lParam as LPVOID,
                    );
                } else {
                    let csA = lParam as *mut MDICREATESTRUCTA;
                    child = CreateWindowExA(
                        WS_EX_MDICHILD,
                        (*csA).szClass,
                        (*csA).szTitle,
                        (*csA).style,
                        (*csA).x,
                        (*csA).y,
                        (*csA).cx,
                        (*csA).cy,
                        hwnd,
                        0 as HMENU,
                        (*csA).hOwner as HINSTANCE,
                        (*csA).lParam as LPVOID,
                    );
                }
                return child as LRESULT;
            }
            return 0;
        }

        WM_MDIDESTROY => {
            return MDIDestroyChild(hwnd, ci, wParam as HWND, 1);
        }

        WM_MDIGETACTIVE => {
            if lParam != 0 {
                *(lParam as *mut BOOL) = IsZoomed((*ci).hwndActiveChild);
            }
            return (*ci).hwndActiveChild as LRESULT;
        }

        WM_MDIICONARRANGE => {
            (*ci).mdiFlags |= MDIF_NEEDUPDATE;
            ArrangeIconicWindows(hwnd);
            (*ci).sbRecalc = SB_BOTH + 1;
            PostMessageA(hwnd, WM_MDICALCCHILDSCROLL, 0, 0);
            return 0;
        }

        WM_MDIMAXIMIZE => {
            ShowWindow(wParam as HWND, SW_MAXIMIZE as INT);
            return 0;
        }

        WM_MDINEXT => {
            let hwnd2: HWND = if wParam != 0 {
                WIN_GetFullHandle(wParam as HWND)
            } else {
                (*ci).hwndActiveChild
            };
            let next = MDI_GetWindow(ci, hwnd2, (lParam == 0) as BOOL, 0);
            MDI_SwitchActiveChild(ci, next, 1);
            if lParam == 0 {
                SetWindowPos(
                    hwnd2,
                    HWND_BOTTOM,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOMOVE | SWP_NOACTIVATE,
                );
            }
        }

        WM_MDIRESTORE => {
            ShowWindow(wParam as HWND, SW_SHOWNORMAL as INT);
            return 0;
        }

        WM_MDISETMENU => {
            return MDISetMenu(hwnd, wParam as HMENU, lParam as HMENU);
        }

        WM_MDIREFRESHMENU => {
            return MDI_RefreshMenu(ci);
        }

        WM_MDITILE => {
            (*ci).mdiFlags |= MDIF_NEEDUPDATE;
            ShowScrollBar(hwnd, SB_BOTH as INT, 0);
            MDITile(hwnd, ci, wParam);
            (*ci).mdiFlags &= !MDIF_NEEDUPDATE;
            return 0;
        }

        WM_VSCROLL | WM_HSCROLL => {
            (*ci).mdiFlags |= MDIF_NEEDUPDATE;
            ScrollChildren(hwnd, message, wParam, lParam);
            (*ci).mdiFlags &= !MDIF_NEEDUPDATE;
            return 0;
        }

        WM_SETFOCUS => {
            if !(*ci).hwndActiveChild.is_null() && IsIconic((*ci).hwndActiveChild) == 0 {
                SetFocus((*ci).hwndActiveChild);
            }
            return 0;
        }

        WM_NCACTIVATE => {
            if !(*ci).hwndActiveChild.is_null() {
                SendMessageW((*ci).hwndActiveChild, message, wParam, lParam);
            }
        }

        WM_PARENTNOTIFY => {
            match LOWORD(wParam as DWORD) as UINT {
                WM_CREATE => {
                    if GetWindowLongPtrW(lParam as HWND, GWL_EXSTYLE) & WS_EX_MDICHILD as LONG_PTR
                        != 0
                    {
                        if (*ci).child.is_null() {
                            (*ci).child = HeapAlloc(
                                GetProcessHeap(),
                                0,
                                (core::mem::size_of::<HWND>()) as SIZE_T,
                            ) as *mut HWND;
                        } else {
                            (*ci).child = HeapReAlloc(
                                GetProcessHeap(),
                                0,
                                (*ci).child as LPVOID,
                                (core::mem::size_of::<HWND>()
                                    * ((*ci).nActiveChildren as usize + 1)) as SIZE_T,
                            ) as *mut HWND;
                        }

                        if !(*ci).child.is_null() {
                            *(*ci).child.offset((*ci).nActiveChildren as isize) = lParam as HWND;
                            (*ci).nTotalCreated += 1;
                            (*ci).nActiveChildren += 1;
                        }
                    }
                }

                WM_LBUTTONDOWN => {
                    let mut pt: POINT = core::mem::zeroed();
                    pt.x = (LOWORD(lParam as DWORD) as i16) as LONG;
                    pt.y = (HIWORD(lParam as DWORD) as i16) as LONG;
                    let child = ChildWindowFromPoint(hwnd, pt);

                    if !child.is_null() && child != hwnd && child != (*ci).hwndActiveChild {
                        SetWindowPos(
                            child,
                            0 as HWND,
                            0,
                            0,
                            0,
                            0,
                            SWP_NOSIZE | SWP_NOMOVE,
                        );
                    }
                }

                WM_DESTROY => {
                    return MDIDestroyChild(hwnd, ci, WIN_GetFullHandle(lParam as HWND), 0);
                }

                _ => {}
            }
            return 0;
        }

        WM_SIZE => {
            if !(*ci).hwndActiveChild.is_null() && IsZoomed((*ci).hwndActiveChild) != 0 {
                let mut rect: RECT = core::mem::zeroed();
                SetRect(
                    &mut rect,
                    0,
                    0,
                    LOWORD(lParam as DWORD) as INT,
                    HIWORD(lParam as DWORD) as INT,
                );
                AdjustWindowRectEx(
                    &mut rect,
                    GetWindowLongPtrA((*ci).hwndActiveChild, GWL_STYLE) as DWORD,
                    0,
                    GetWindowLongPtrA((*ci).hwndActiveChild, GWL_EXSTYLE) as DWORD,
                );
                MoveWindow(
                    (*ci).hwndActiveChild,
                    rect.left,
                    rect.top,
                    rect.right - rect.left,
                    rect.bottom - rect.top,
                    1,
                );
            } else {
                MDI_PostUpdate(hwnd, ci, (SB_BOTH + 1) as WORD);
            }
        }

        WM_MDICALCCHILDSCROLL => {
            if ((*ci).mdiFlags & MDIF_NEEDUPDATE) != 0 && (*ci).sbRecalc != 0 {
                CalcChildScroll(hwnd, ((*ci).sbRecalc - 1) as INT);
                (*ci).sbRecalc = 0;
                (*ci).mdiFlags &= !MDIF_NEEDUPDATE;
            }
            return 0;
        }

        _ => {}
    }

    if unicode != 0 {
        DefWindowProcW(hwnd, message, wParam, lParam)
    } else {
        DefWindowProcA(hwnd, message, wParam, lParam)
    }
}

#[no_mangle]
pub unsafe extern "system" fn MDIClientWndProcA(
    hwnd: HWND,
    message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    if IsWindow(hwnd) == 0 {
        return 0;
    }
    MDIClientWndProc_common(hwnd, message, wParam, lParam, 0)
}

#[no_mangle]
pub unsafe extern "system" fn MDIClientWndProcW(
    hwnd: HWND,
    message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    if IsWindow(hwnd) == 0 {
        return 0;
    }
    MDIClientWndProc_common(hwnd, message, wParam, lParam, 1)
}

#[no_mangle]
pub unsafe extern "system" fn DefFrameProcA(
    hwnd: HWND,
    hwndMDIClient: HWND,
    message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    if !hwndMDIClient.is_null() {
        match message {
            WM_SETTEXT => {
                let len = MultiByteToWideChar(CP_ACP, 0, lParam as LPSTR, -1, core::ptr::null_mut(), 0);
                let text = HeapAlloc(GetProcessHeap(), 0, ((len as usize) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
                if text.is_null() {
                    return 0;
                }
                MultiByteToWideChar(CP_ACP, 0, lParam as LPSTR, -1, text, len);
                MDI_UpdateFrameText(hwnd, hwndMDIClient, 0u8 as BOOL, text);
                HeapFree(GetProcessHeap(), 0, text as LPVOID);
                return 1;
            }
            WM_COMMAND | WM_NCACTIVATE | WM_NEXTMENU | WM_SETFOCUS | WM_SIZE => {
                return DefFrameProcW(hwnd, hwndMDIClient, message, wParam, lParam);
            }
            _ => {}
        }
    }
    DefWindowProcA(hwnd, message, wParam, lParam)
}

#[no_mangle]
pub unsafe extern "system" fn DefFrameProcW(
    hwnd: HWND,
    hwndMDIClient: HWND,
    message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let ci = get_client_info(hwndMDIClient);

    if !ci.is_null() {
        match message {
            WM_COMMAND => 'block: {
                let id = LOWORD(wParam as u32) as WORD;
                if (id as UINT) < (*ci).idFirstChild
                    || (id as UINT) >= (*ci).idFirstChild + (*ci).nActiveChildren
                {
                    if ((id as i32 - 0xf000) & 0xf00f) != 0 {
                        break 'block;
                    }
                    if (*ci).hwndChildMaximized.is_null() {
                        break 'block;
                    }
                    match id as UINT {
                        SC_CLOSE => {
                            if is_close_enabled((*ci).hwndActiveChild, core::ptr::null_mut()) == 0 {
                                break 'block;
                            }
                            return SendMessageW(
                                (*ci).hwndChildMaximized,
                                WM_SYSCOMMAND,
                                wParam,
                                lParam,
                            );
                        }
                        SC_SIZE | SC_MOVE | SC_MINIMIZE | SC_MAXIMIZE | SC_NEXTWINDOW
                        | SC_PREVWINDOW | SC_RESTORE => {
                            return SendMessageW(
                                (*ci).hwndChildMaximized,
                                WM_SYSCOMMAND,
                                wParam,
                                lParam,
                            );
                        }
                        _ => {}
                    }
                } else {
                    let childHwnd: HWND;
                    if (id as UINT - (*ci).idFirstChild) == MDI_MOREWINDOWSLIMIT as UINT {
                        childHwnd = MDI_MoreWindowsDialog(hwndMDIClient);
                    } else {
                        childHwnd = MDI_GetChildByID(hwndMDIClient, id as UINT, ci);
                    }

                    if !childHwnd.is_null() {
                        SendMessageW(hwndMDIClient, WM_MDIACTIVATE, childHwnd as WPARAM, 0);
                    }
                }
            }

            WM_NCACTIVATE => {
                SendMessageW(hwndMDIClient, message, wParam, lParam);
            }

            WM_SETTEXT => {
                MDI_UpdateFrameText(hwnd, hwndMDIClient, 0, lParam as LPWSTR);
                return 1;
            }

            WM_SETFOCUS => {
                SetFocus(hwndMDIClient);
            }

            WM_SIZE => {
                MoveWindow(
                    hwndMDIClient,
                    0,
                    0,
                    LOWORD(lParam as u32) as i32,
                    HIWORD(lParam as u32) as i32,
                    1,
                );
            }

            WM_NEXTMENU => {
                let next_menu = lParam as *mut MDINEXTMENU;

                if IsIconic(hwnd) == 0
                    && !(*ci).hwndActiveChild.is_null()
                    && IsZoomed((*ci).hwndActiveChild) == 0
                {
                    if (wParam == VK_LEFT as WPARAM
                        && GetMenu(hwnd) == (*next_menu).hmenuIn)
                        || (wParam == VK_RIGHT as WPARAM
                            && GetSubMenu(GetMenu(hwnd), 0) == (*next_menu).hmenuIn)
                    {
                        (*next_menu).hmenuNext = GetSubMenu(GetMenu((*ci).hwndActiveChild), 0);
                        (*next_menu).hwndNext = (*ci).hwndActiveChild;
                    }
                }
                return 0;
            }

            _ => {}
        }
    }

    DefWindowProcW(hwnd, message, wParam, lParam)
}

#[no_mangle]
pub unsafe extern "system" fn DefMDIChildProcA( mut hwnd: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM ) -> LRESULT {
    let client = GetParent(hwnd);
    let ci = get_client_info(client);

    hwnd = WIN_GetFullHandle(hwnd);
    if ci.is_null() {
        return DefWindowProcA(hwnd, message, wParam, lParam);
    }

    match message {
        WM_SETTEXT => {
            DefWindowProcA(hwnd, message, wParam, lParam);
            if (*ci).hwndChildMaximized == hwnd {
                MDI_UpdateFrameText(GetParent(client), client, 1u8 as BOOL, core::ptr::null_mut());
            }
            MDI_RefreshMenu(ci);
            1
        }
        WM_GETMINMAXINFO
        | WM_MENUCHAR
        | WM_CLOSE
        | WM_SETFOCUS
        | WM_CHILDACTIVATE
        | WM_SYSCOMMAND
        | WM_SHOWWINDOW
        | WM_SETVISIBLE
        | WM_SIZE
        | WM_NEXTMENU
        | WM_SYSCHAR
        | WM_DESTROY => DefMDIChildProcW(hwnd, message, wParam, lParam),
        _ => DefWindowProcA(hwnd, message, wParam, lParam),
    }
}

#[no_mangle]
pub unsafe extern "system" fn DefMDIChildProcW( mut hwnd: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM ) -> LRESULT
{
    let client = GetParent(hwnd);
    let ci = get_client_info( client );

    hwnd = WIN_GetFullHandle( hwnd );
    if ci.is_null() { return DefWindowProcW( hwnd, message, wParam, lParam ); }

    match message
    {
        WM_SETTEXT => {
            DefWindowProcW(hwnd, message, wParam, lParam);
            if (*ci).hwndChildMaximized == hwnd {
                MDI_UpdateFrameText( GetParent(client), client, 1, core::ptr::null_mut() );
            }
            MDI_RefreshMenu( ci );
            return 1;
        }

        WM_GETMINMAXINFO => {
            MDI_ChildGetMinMaxInfo( client, hwnd, lParam as *mut MINMAXINFO );
            return 0;
        }

        WM_MENUCHAR => {
            return MAKELRESULT( 0, MNC_CLOSE as WORD );
        }

        WM_CLOSE => {
            SendMessageW( client, WM_MDIDESTROY, hwnd as WPARAM, 0 );
            return 0;
        }

        WM_SETFOCUS => {
            if (*ci).hwndActiveChild != hwnd {
                MDI_ChildActivate( client, hwnd );
            }
        }

        WM_CHILDACTIVATE => {
            if IsWindowEnabled( hwnd ) != 0 {
                MDI_ChildActivate( client, hwnd );
            }
            return 0;
        }

        WM_SYSCOMMAND => {
            match (wParam & 0xfff0) as UINT {
                SC_MOVE => {
                    if (*ci).hwndChildMaximized == hwnd {
                        return 0;
                    }
                }
                SC_RESTORE | SC_MINIMIZE => {}
                SC_MAXIMIZE => {
                    if (*ci).hwndChildMaximized == hwnd {
                        return SendMessageW( GetParent(client), message, wParam, lParam);
                    }
                }
                SC_NEXTWINDOW => {
                    SendMessageW( client, WM_MDINEXT, (*ci).hwndActiveChild as WPARAM, 0);
                    return 0;
                }
                SC_PREVWINDOW => {
                    SendMessageW( client, WM_MDINEXT, (*ci).hwndActiveChild as WPARAM, 1);
                    return 0;
                }
                _ => {}
            }
        }

        WM_SHOWWINDOW | WM_SETVISIBLE => {
            MDI_PostUpdate(client, ci, (SB_BOTH+1) as WORD);
        }

        WM_SIZE => {
            if (*ci).hwndChildMaximized == hwnd && wParam != SIZE_MAXIMIZED as WPARAM
            {
                (*ci).hwndChildMaximized = core::ptr::null_mut();

                let frame = GetParent(client);
                MDI_RestoreFrameMenu( frame, hwnd, (*ci).hBmpClose );
                MDI_UpdateFrameText( frame, client, 1, core::ptr::null_mut() );
            }

            if wParam == SIZE_MAXIMIZED as WPARAM
            {
                let hMaxChild = (*ci).hwndChildMaximized;

                if hMaxChild == hwnd { return DefWindowProcW(hwnd, message, wParam, lParam); }

                if !hMaxChild.is_null()
                {
                    SendMessageW( hMaxChild, WM_SETREDRAW, 0, 0 );

                    MDI_RestoreFrameMenu( GetParent(client), hMaxChild, (*ci).hBmpClose );
                    ShowWindow( hMaxChild, SW_SHOWNOACTIVATE as INT );

                    SendMessageW( hMaxChild, WM_SETREDRAW, 1, 0 );
                }

                (*ci).hwndChildMaximized = hwnd;

                let frame = GetParent(client);
                MDI_AugmentFrameMenu( frame, hwnd );
                MDI_UpdateFrameText( frame, client, 1, core::ptr::null_mut() );
            }

            if wParam == SIZE_MINIMIZED as WPARAM
            {
                let mut switchTo = MDI_GetWindow( ci, hwnd, 1, WS_MINIMIZE as DWORD );

                if switchTo.is_null() { switchTo = hwnd; }
                SendMessageW( switchTo, WM_CHILDACTIVATE, 0, 0 );
            }

            MDI_PostUpdate(client, ci, (SB_BOTH+1) as WORD);
        }

        WM_NEXTMENU => {
            let next_menu = lParam as *mut MDINEXTMENU;
            let parent = GetParent(client);

            if wParam == VK_LEFT as WPARAM
            {
                (*next_menu).hmenuNext = GetSubMenu( GetMenu(parent), 0 );
            }
            if wParam == VK_RIGHT as WPARAM
            {
                (*next_menu).hmenuNext = GetMenu(parent);
            }
            (*next_menu).hwndNext = parent;
            return 0;
        }

        WM_SYSCHAR => {
            if wParam == b'-' as WPARAM
            {
                SendMessageW( hwnd, WM_SYSCOMMAND, SC_KEYMENU as WPARAM, VK_SPACE as LPARAM);
                return 0;
            }
        }

        WM_DESTROY => {
            MDI_RefreshMenu(ci);
        }

        _ => {}
    }
    DefWindowProcW(hwnd, message, wParam, lParam)
}

#[no_mangle]
pub unsafe extern "system" fn CreateMDIWindowA(
    lpClassName: LPCSTR,
    lpWindowName: LPCSTR,
    dwStyle: DWORD,
    X: INT,
    Y: INT,
    nWidth: INT,
    nHeight: INT,
    hWndParent: HWND,
    hInstance: HINSTANCE,
    lParam: LPARAM,
) -> HWND {
    CreateWindowExA(
        WS_EX_MDICHILD,
        lpClassName,
        lpWindowName,
        dwStyle,
        X,
        Y,
        nWidth,
        nHeight,
        hWndParent,
        0 as HMENU,
        hInstance,
        lParam as LPVOID,
    )
}

#[no_mangle]
pub unsafe extern "system" fn CreateMDIWindowW(
    lpClassName: LPCWSTR,
    lpWindowName: LPCWSTR,
    dwStyle: DWORD,
    X: INT,
    Y: INT,
    nWidth: INT,
    nHeight: INT,
    hWndParent: HWND,
    hInstance: HINSTANCE,
    lParam: LPARAM,
) -> HWND {
    CreateWindowExW(
        WS_EX_MDICHILD,
        lpClassName,
        lpWindowName,
        dwStyle,
        X,
        Y,
        nWidth,
        nHeight,
        hWndParent,
        0 as HMENU,
        hInstance,
        lParam as LPVOID,
    )
}

#[no_mangle]
pub unsafe extern "system" fn TranslateMDISysAccel(hwndClient: HWND, msg: LPMSG) -> BOOL {
    if (*msg).message == WM_KEYDOWN || (*msg).message == WM_SYSKEYDOWN {
        let ci = get_client_info(hwndClient);
        let mut wParam: WPARAM = 0;

        if ci.is_null() || IsWindowEnabled((*ci).hwndActiveChild) == 0 {
            return 0;
        }

        if (GetKeyState(VK_CONTROL as i32) as i32 & 0x8000u16 as i16 as i32) != 0
            && (GetKeyState(VK_MENU as i32) as i32 & 0x8000u16 as i16 as i32) == 0
        {
            match (*msg).wParam as i32 {
                x if x == VK_F6 as i32 || x == VK_TAB as i32 => {
                    wParam = if (GetKeyState(VK_SHIFT as i32) as i32 & 0x8000u16 as i16 as i32) != 0 {
                        SC_NEXTWINDOW as WPARAM
                    } else {
                        SC_PREVWINDOW as WPARAM
                    };
                }
                x if x == VK_F4 as i32 || x == VK_RBUTTON as i32 => {
                    if is_close_enabled((*ci).hwndActiveChild, core::ptr::null_mut()) != 0 {
                        wParam = SC_CLOSE as WPARAM;
                    } else {
                        return 0;
                    }
                }
                _ => return 0,
            }
            TRACE!(b"wParam = %04lx\n\0".as_ptr() as LPCSTR, wParam);
            SendMessageW((*ci).hwndActiveChild, WM_SYSCOMMAND, wParam, (*msg).wParam as LPARAM);
            return 1;
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn CalcChildScroll(hwnd: HWND, scroll: INT) {
    let mut info: SCROLLINFO = core::mem::zeroed();
    let mut childRect: RECT = core::mem::zeroed();
    let mut clientRect: RECT = core::mem::zeroed();
    let list: *mut HWND;
    let mut style: DWORD;
    let mut WindowInfo: WINDOWINFO = core::mem::zeroed();

    GetClientRect(hwnd, &mut clientRect);
    SetRectEmpty(&mut childRect);

    WindowInfo.cbSize = core::mem::size_of::<WINDOWINFO>() as u32;
    if GetWindowInfo(hwnd, &mut WindowInfo) == 0 {
        return;
    }

    list = WIN_ListChildren(hwnd);
    if !list.is_null() {
        let mut i = 0isize;
        while !(*list.offset(i)).is_null() {
            style = GetWindowLongPtrW(*list.offset(i), GWL_STYLE as i32) as DWORD;
            if (style & WS_MAXIMIZE) != 0 {
                HeapFree(GetProcessHeap(), 0, list as LPVOID);
                ShowScrollBar(hwnd, SB_BOTH as i32, FALSE as BOOL);
                return;
            }
            if (style & WS_VISIBLE) != 0 {
                let mut rect: RECT = core::mem::zeroed();
                GetWindowRect(*list.offset(i), &mut rect);
                OffsetRect(&mut rect, -WindowInfo.rcClient.left, -WindowInfo.rcClient.top);
                UnionRect(&mut childRect, &rect, &childRect);
            }
            i += 1;
        }
        HeapFree(GetProcessHeap(), 0, list as LPVOID);
    }
    UnionRect(&mut childRect, &clientRect, &childRect);

    info.cbSize = core::mem::size_of::<SCROLLINFO>() as u32;
    info.fMask = (SIF_POS | SIF_RANGE | SIF_PAGE) as u32;
    info.nPos = 0;

    style = GetWindowLongW(hwnd, GWL_STYLE as i32) as DWORD;
    match scroll as u32 {
        x if x == SB_BOTH || x == SB_HORZ => {
            if (style & (WS_HSCROLL | WS_VSCROLL)) != 0 {
                info.nMin = childRect.left;
                info.nMax = childRect.right;
                info.nPage = (1 + clientRect.right - clientRect.left) as u32;
                SetScrollInfo(hwnd, SB_HORZ as i32, &info, TRUE as BOOL);
            }
            if scroll as u32 == SB_HORZ {
            } else {
                if (style & (WS_HSCROLL | WS_VSCROLL)) != 0 {
                    info.nMin = childRect.top;
                    info.nMax = childRect.bottom;
                    info.nPage = (1 + clientRect.bottom - clientRect.top) as u32;
                    SetScrollInfo(hwnd, SB_VERT as i32, &info, TRUE as BOOL);
                }
            }
        }
        x if x == SB_VERT => {
            if (style & (WS_HSCROLL | WS_VSCROLL)) != 0 {
                info.nMin = childRect.top;
                info.nMax = childRect.bottom;
                info.nPage = (1 + clientRect.bottom - clientRect.top) as u32;
                SetScrollInfo(hwnd, SB_VERT as i32, &info, TRUE as BOOL);
            }
        }
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "system" fn ScrollChildren(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) {

    let mut newPos: INT = -1;
    let mut curPos: INT;
    let length: INT;
    let mut minPos: INT = 0;
    let mut maxPos: INT = 0;
    let shift: INT;
    let mut rect: RECT = core::mem::zeroed();

    GetClientRect(hWnd, &mut rect);

    match uMsg {
        WM_HSCROLL => {
            GetScrollRange(hWnd, SB_HORZ as INT, &mut minPos, &mut maxPos);
            curPos = GetScrollPos(hWnd, SB_HORZ as INT);
            length = (rect.right - rect.left) / 2;
            shift = GetSystemMetrics(SM_CYHSCROLL as INT);
        }
        WM_VSCROLL => {
            GetScrollRange(hWnd, SB_VERT as INT, &mut minPos, &mut maxPos);
            curPos = GetScrollPos(hWnd, SB_VERT as INT);
            length = (rect.bottom - rect.top) / 2;
            shift = GetSystemMetrics(SM_CXVSCROLL as INT);
        }
        _ => return,
    }

    match wParam as u32 {
        SB_LINEUP => {
            newPos = curPos - shift;
        }
        SB_LINEDOWN => {
            newPos = curPos + shift;
        }
        SB_PAGEUP => {
            newPos = curPos - length;
        }
        SB_PAGEDOWN => {
            newPos = curPos + length;
        }
        SB_THUMBPOSITION => {
            newPos = (lParam & 0xFFFF) as INT;
        }
        SB_THUMBTRACK => {
            return;
        }
        SB_TOP => {
            newPos = minPos;
        }
        SB_BOTTOM => {
            newPos = maxPos;
        }
        SB_ENDSCROLL => {
            CalcChildScroll(
                hWnd,
                if uMsg == WM_VSCROLL { SB_VERT as INT } else { SB_HORZ as INT },
            );
            return;
        }
        _ => {}
    }

    if newPos > maxPos {
        newPos = maxPos;
    } else if newPos < minPos {
        newPos = minPos;
    }

    SetScrollPos(
        hWnd,
        if uMsg == WM_VSCROLL { SB_VERT as INT } else { SB_HORZ as INT },
        newPos,
        1,
    );

    if uMsg == WM_VSCROLL {
        ScrollWindowEx(
            hWnd,
            0,
            curPos - newPos,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            (SW_INVALIDATE | SW_ERASE | SW_SCROLLCHILDREN) as UINT,
        );
    } else {
        ScrollWindowEx(
            hWnd,
            curPos - newPos,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            (SW_INVALIDATE | SW_ERASE | SW_SCROLLCHILDREN) as UINT,
        );
    }
}

// WNDENUMPROC is `extern "stdcall"` on i386, `extern "C"` on amd64/arm64 in win32k_sys.
unsafe extern "system" fn GetCascadeChildProc(hwnd: HWND, lParam: LPARAM) -> BOOL {
    GetCascadeChildProcImpl(hwnd, lParam)
}

unsafe fn GetCascadeChildProcImpl(hwnd: HWND, lParam: LPARAM) -> BOOL {
    let mut count: DWORD;
    let ahwnd: *mut HWND;
    let pInfo = lParam as *mut CASCADE_INFO;

    if hwnd == (*pInfo).hwndDesktop || hwnd == (*pInfo).hTrayWnd ||
        hwnd == (*pInfo).hwndProgman || hwnd == (*pInfo).hwndTop
    {
        return TRUE as BOOL;
    }

    if !(*pInfo).hwndParent.is_null() && GetParent(hwnd) != (*pInfo).hwndParent {
        return TRUE as BOOL;
    }

    if ((*pInfo).wFlags & MDITILE_SKIPDISABLED) != 0 && IsWindowEnabled(hwnd) == 0 {
        return TRUE as BOOL;
    }

    if IsWindowVisible(hwnd) == 0 || IsIconic(hwnd) != 0 {
        return TRUE as BOOL;
    }

    count = (*pInfo).chwnd;
    let size = (count + 1) * core::mem::size_of::<HWND>() as DWORD;

    if count == 0 || (*pInfo).ahwnd.is_null() {
        count = 0;
        (*pInfo).ahwnd = HeapAlloc(GetProcessHeap(), 0, (size) as SIZE_T) as *mut HWND;
    } else {
        ahwnd = HeapReAlloc(GetProcessHeap(), 0, (*pInfo).ahwnd as LPVOID, (size) as SIZE_T) as *mut HWND;
        if ahwnd.is_null() {
            HeapFree(GetProcessHeap(), 0, (*pInfo).ahwnd as LPVOID);
        }
        (*pInfo).ahwnd = ahwnd;
    }

    if (*pInfo).ahwnd.is_null() {
        (*pInfo).chwnd = 0;
        return FALSE as BOOL;
    }

    *(*pInfo).ahwnd.offset(count as isize) = hwnd;
    (*pInfo).chwnd = count + 1;
    TRUE as BOOL
}

unsafe fn QuerySizeFix(hwnd: HWND, pcx: LPINT, pcy: LPINT) -> BOOL {
    let mut mmi: MINMAXINFO = core::mem::zeroed();
    let mut dwResult: DWORD_PTR = 0;

    mmi.ptMinTrackSize.x = 0;
    mmi.ptMinTrackSize.y = 0;
    mmi.ptMaxTrackSize.x = MAXLONG as LONG;
    mmi.ptMaxTrackSize.y = MAXLONG as LONG;
    if SendMessageTimeoutW(
        hwnd,
        WM_GETMINMAXINFO,
        0,
        core::ptr::addr_of_mut!(mmi) as LPARAM,
        (SMTO_ABORTIFHUNG | SMTO_NORMAL) as UINT,
        1000,
        &mut dwResult,
    ) != 0
    {
        *pcx = min(max(*pcx, mmi.ptMinTrackSize.x), mmi.ptMaxTrackSize.x);
        *pcy = min(max(*pcy, mmi.ptMinTrackSize.y), mmi.ptMaxTrackSize.y);
        return 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn CascadeWindows(
    hwndParent: HWND,
    wFlags: UINT,
    lpRect: LPCRECT,
    cKids: UINT,
    lpKids: *const HWND,
) -> WORD {
    let mut info: CASCADE_INFO = core::mem::zeroed();
    let mut hwnd: HWND;
    let hwndTop: HWND;
    let mut hwndPrev: HWND;
    let hMon: HMONITOR;
    let mut mi: MONITORINFO = core::mem::zeroed();
    let mut rcWork: RECT = core::mem::zeroed();
    let mut rcWnd: RECT = core::mem::zeroed();
    let mut i: DWORD;
    let mut ret: DWORD = 0;
    let mut x: INT;
    let mut y: INT;
    let mut cx: INT;
    let mut cy: INT;
    let mut cxNew: INT;
    let mut cyNew: INT;
    let cxWork: INT;
    let cyWork: INT;
    let dx: INT;
    let dy: INT;
    let mut hDWP: HDWP;
    let mut pt: POINT = core::mem::zeroed();

    hwndTop = GetTopWindow(hwndParent);

    core::ptr::write_bytes(&mut info as *mut CASCADE_INFO, 0, 1);
    info.hwndDesktop = GetDesktopWindow();
    info.hTrayWnd = FindWindowW(
        w!("Shell_TrayWnd").as_ptr() as LPCWSTR,
        core::ptr::null_mut(),
    );
    info.hwndProgman = FindWindowW(
        w!("Progman").as_ptr() as LPCWSTR,
        core::ptr::null_mut(),
    );
    info.hwndParent = hwndParent;
    info.wFlags = wFlags;

    if cKids == 0 || lpKids.is_null() {
        info.hwndTop = hwndTop;
        EnumChildWindows(
            hwndParent,
            Some(GetCascadeChildProc),
            &mut info as *mut CASCADE_INFO as LPARAM,
        );

        info.hwndTop = core::ptr::null_mut();
        GetCascadeChildProc(hwndTop, &mut info as *mut CASCADE_INFO as LPARAM);
    } else {
        info.chwnd = cKids;
        info.ahwnd = lpKids as *mut HWND;
    }

    if info.chwnd == 0 || info.ahwnd.is_null() {
        return ret as WORD;
    }

    'cleanup: {
        if !lpRect.is_null() {
            rcWork = *lpRect;
        } else if !hwndParent.is_null() {
            GetClientRect(hwndParent, &mut rcWork);
        } else {
            pt.x = 0;
            pt.y = 0;
            hMon = MonitorFromPoint(pt, MONITOR_DEFAULTTOPRIMARY as DWORD);
            mi.cbSize = core::mem::size_of::<MONITORINFO>() as DWORD;
            GetMonitorInfoW(hMon, &mut mi);
            rcWork = core::mem::transmute::<_RECTL, RECT>(mi.rcWork);
        }

        hDWP = BeginDeferWindowPos(info.chwnd as INT);
        if hDWP.is_null() {
            break 'cleanup;
        }

        x = rcWork.left;
        y = rcWork.top;
        dx = GetSystemMetrics(SM_CXSIZEFRAME as INT) + GetSystemMetrics(SM_CXSIZE as INT);
        dy = GetSystemMetrics(SM_CYSIZEFRAME as INT) + GetSystemMetrics(SM_CYSIZE as INT);
        cxWork = rcWork.right - rcWork.left;
        cyWork = rcWork.bottom - rcWork.top;
        hwndPrev = core::ptr::null_mut();

        i = info.chwnd;
        while i > 0 {
            i -= 1;
            hwnd = *info.ahwnd.offset(i as isize);

            if IsWindowVisible(hwnd) == 0 || IsIconic(hwnd) != 0 {
                continue;
            }

            if (info.wFlags & MDITILE_SKIPDISABLED as UINT) != 0 && IsWindowEnabled(hwnd) == 0 {
                continue;
            }

            if IsZoomed(hwnd) != 0 {
                ShowWindow(hwnd, (SW_RESTORE | SW_SHOWNA) as INT);
            }

            GetWindowRect(hwnd, &mut rcWnd);
            cx = rcWnd.right - rcWnd.left;
            cxNew = cx;
            cy = rcWnd.bottom - rcWnd.top;
            cyNew = cy;

            if info.chwnd != 1
                && (GetWindowLongPtrW(hwnd, GWL_STYLE) & WS_THICKFRAME as LONG_PTR) != 0
            {
                let max_cx = (cxWork * 5) / 7;
                let min_cx = (cxWork * 4) / 7;
                let max_cy = (cyWork * 5) / 7;
                let min_cy = (cyWork * 4) / 7;
                cxNew = core::cmp::max(core::cmp::min(cxNew, max_cx), min_cx);
                cyNew = core::cmp::max(core::cmp::min(cyNew, max_cy), min_cy);
                if cx != cxNew || cy != cyNew {
                    if QuerySizeFix(hwnd, &mut cxNew, &mut cyNew) != 0 {
                        cx = cxNew;
                        cy = cyNew;
                    }
                }
            }

            if x + cx > rcWork.right {
                x = rcWork.left;
            }
            if y + cy > rcWork.bottom {
                y = rcWork.top;
            }

            hDWP = DeferWindowPos(
                hDWP,
                hwnd,
                HWND_TOP,
                x,
                y,
                cx,
                cy,
                SWP_NOACTIVATE as UINT,
            );
            if hDWP.is_null() {
                ret = 0;
                break 'cleanup;
            }

            x += dx;
            y += dy;
            hwndPrev = hwnd;
            ret += 1;
        }

        NtUserEndDeferWindowPosEx(hDWP, 1u8 as BOOL);

        if !hwndPrev.is_null() {
            SetForegroundWindow(hwndPrev);
        }
    }

    if cKids == 0 || lpKids.is_null() {
        HeapFree(GetProcessHeap(), 0, info.ahwnd as LPVOID);
    }

    ret as WORD
}

#[no_mangle]
pub unsafe extern "system" fn CascadeChildWindows(parent: HWND, flags: UINT) -> WORD {
    CascadeWindows(parent, flags, core::ptr::null_mut(), 0, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "system" fn TileWindows(
    hwndParent: HWND,
    wFlags: UINT,
    lpRect: LPCRECT,
    cKids: UINT,
    lpKids: *const HWND,
) -> WORD {
    let mut hwnd: HWND;
    let hwndTop: HWND;
    let mut hwndPrev: HWND;
    let mut info: CASCADE_INFO = core::mem::zeroed();
    let mut rcWork: RECT = core::mem::zeroed();
    let mut rcWnd: RECT = core::mem::zeroed();
    let mut i: DWORD;
    let mut iRow: DWORD;
    let mut iColumn: DWORD;
    let mut cRows: DWORD;
    let mut cColumns: DWORD;
    let mut ret: DWORD = 0;
    let mut x: INT;
    let mut y: INT;
    let mut cx: INT;
    let mut cy: INT;
    let mut cxNew: INT;
    let mut cyNew: INT;
    let cxWork: INT;
    let cyWork: INT;
    let mut cxCell: INT;
    let mut cyCell: INT;
    let cxMin2: INT;
    let cyMin3: INT;
    let mut hDWP: HDWP;
    let mut mi: MONITORINFO = core::mem::zeroed();
    let hMon: HMONITOR;
    let mut pt: POINT = core::mem::zeroed();

    hwndTop = GetTopWindow(hwndParent);

    core::ptr::write_bytes(&mut info as *mut CASCADE_INFO, 0, 1);
    info.hwndDesktop = GetDesktopWindow();
    info.hTrayWnd = FindWindowW(
        w!("Shell_TrayWnd").as_ptr() as LPCWSTR,
        core::ptr::null_mut(),
    );
    info.hwndProgman = FindWindowW(
        w!("Progman").as_ptr() as LPCWSTR,
        core::ptr::null_mut(),
    );
    info.hwndParent = hwndParent;
    info.wFlags = wFlags;

    if cKids == 0 || lpKids.is_null() {
        info.hwndTop = hwndTop;
        EnumChildWindows(hwndParent, Some(GetCascadeChildProc), &mut info as *mut _ as LPARAM);

        info.hwndTop = core::ptr::null_mut();
        GetCascadeChildProc(hwndTop, &mut info as *mut _ as LPARAM);
    } else {
        info.chwnd = cKids;
        info.ahwnd = lpKids as *mut HWND;
    }

    if info.chwnd == 0 || info.ahwnd.is_null() {
        return ret as WORD;
    }

    if !lpRect.is_null() {
        rcWork = *lpRect;
    } else if !hwndParent.is_null() {
        GetClientRect(hwndParent, &mut rcWork);
    } else {
        pt.x = 0;
        pt.y = 0;
        hMon = MonitorFromPoint(pt, MONITOR_DEFAULTTOPRIMARY as DWORD);
        mi.cbSize = core::mem::size_of::<MONITORINFO>() as DWORD;
        GetMonitorInfoW(hMon, &mut mi);
        rcWork = core::mem::transmute::<_RECTL, RECT>(mi.rcWork);
    }

    cxWork = rcWork.right - rcWork.left;
    cyWork = rcWork.bottom - rcWork.top;

    cxMin2 = GetSystemMetrics(SM_CXMIN as INT) * 2;
    cyMin3 = GetSystemMetrics(SM_CYMIN as INT) * 3;

    if info.wFlags & MDITILE_HORIZONTAL != 0 {
        cColumns = info.chwnd;
        cRows = 1;
        loop {
            cxCell = (cxWork as DWORD / cColumns) as INT;
            cyCell = (cyWork as DWORD / cRows) as INT;
            if cyCell <= cyMin3 || cxCell >= cxMin2 {
                break;
            }

            cRows += 1;
            cColumns = (info.chwnd + cRows - 1) / cRows;
        }
    } else {
        cRows = info.chwnd;
        cColumns = 1;
        loop {
            cxCell = (cxWork as DWORD / cColumns) as INT;
            cyCell = (cyWork as DWORD / cRows) as INT;
            if cxCell <= cxMin2 || cyCell >= cyMin3 {
                break;
            }

            cColumns += 1;
            cRows = (info.chwnd + cColumns - 1) / cColumns;
        }
    }

    'cleanup: {
        hDWP = BeginDeferWindowPos(info.chwnd as INT);
        if hDWP.is_null() {
            break 'cleanup;
        }

        x = rcWork.left;
        y = rcWork.top;
        hwndPrev = core::ptr::null_mut();
        iRow = 0;
        iColumn = 0;
        i = info.chwnd;
        while i > 0 {
            i -= 1;
            hwnd = *info.ahwnd.offset(i as isize);

            if IsZoomed(hwnd) != 0 {
                ShowWindow(hwnd, (SW_RESTORE | SW_SHOWNA) as INT);
            }

            GetWindowRect(hwnd, &mut rcWnd);
            cx = rcWnd.right - rcWnd.left;
            cy = rcWnd.bottom - rcWnd.top;

            if GetWindowLongPtrW(hwnd, GWL_STYLE as INT) as DWORD & WS_THICKFRAME != 0 {
                cxNew = cxCell;
                cyNew = cyCell;
                if QuerySizeFix(hwnd, &mut cxNew, &mut cyNew) != 0 {
                    cx = cxNew;
                    cy = cyNew;
                }
            }

            hDWP = DeferWindowPos(hDWP, hwnd, HWND_TOP, x, y, cx, cy, SWP_NOACTIVATE);
            if hDWP.is_null() {
                ret = 0;
                break 'cleanup;
            }

            if info.wFlags & MDITILE_HORIZONTAL != 0 {
                x += cxCell;
                iColumn += 1;
                if iColumn >= cColumns {
                    iColumn = 0;
                    iRow += 1;
                    x = rcWork.left;
                    y += cyCell;
                }
            } else {
                y += cyCell;
                iRow += 1;
                if iRow >= cRows {
                    iRow = 0;
                    iColumn += 1;
                    x += cxCell;
                    y = rcWork.top;
                }
            }
            hwndPrev = hwnd;
            ret += 1;
        }

        NtUserEndDeferWindowPosEx(hDWP, 1i32);

        if !hwndPrev.is_null() {
            SetForegroundWindow(hwndPrev);
        }
    }

    if cKids == 0 || lpKids.is_null() {
        HeapFree(GetProcessHeap(), 0, info.ahwnd as LPVOID);
    }

    ret as WORD
}

#[no_mangle]
pub unsafe extern "system" fn TileChildWindows(parent: HWND, flags: UINT) -> WORD {
    TileWindows(parent, flags, core::ptr::null_mut(), 0, core::ptr::null_mut())
}

unsafe extern "system" fn MDI_MoreWindowsDlgProc(hDlg: HWND, iMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match iMsg {
        WM_INITDIALOG => {
            let mut widest: UINT = 0;
            let mut length: UINT;
            let mut i: UINT;
            let ci: *mut MDICLIENTINFO = get_client_info(lParam as HWND);
            let hListBox: HWND = GetDlgItem(hDlg, MDI_IDC_LISTBOX as core::ffi::c_int);

            i = 0;
            while i < (*ci).nActiveChildren {
                let mut buffer: [WCHAR; MDI_MAXTITLELENGTH as usize] = [0; MDI_MAXTITLELENGTH as usize];

                if InternalGetWindowText(
                    *(*ci).child.offset(i as isize),
                    buffer.as_mut_ptr(),
                    (core::mem::size_of_val(&buffer) / core::mem::size_of::<WCHAR>()) as core::ffi::c_int,
                ) == 0
                {
                    i += 1;
                    continue;
                }
                SendMessageW(hListBox, LB_ADDSTRING, 0, buffer.as_ptr() as LPARAM);
                SendMessageW(hListBox, LB_SETITEMDATA, i as WPARAM, *(*ci).child.offset(i as isize) as LPARAM);
                length = strlenW(buffer.as_ptr()) as UINT;
                if length > widest {
                    widest = length;
                }
                i += 1;
            }
            SendMessageW(hListBox, LB_SETHORIZONTALEXTENT, (widest * 6) as WPARAM, 0);

            SendMessageW(hListBox, LB_SETCURSEL, MDI_MOREWINDOWSLIMIT as WPARAM, 0);
            return 1;
        }

        WM_COMMAND => {
            match LOWORD(wParam as DWORD) as UINT {
                v if v == IDOK as UINT => {
                    let hListBox: HWND = GetDlgItem(hDlg, MDI_IDC_LISTBOX as core::ffi::c_int);
                    let index: UINT = SendMessageW(hListBox, LB_GETCURSEL, 0, 0) as UINT;
                    let res: LRESULT = SendMessageW(hListBox, LB_GETITEMDATA, index as WPARAM, 0);
                    EndDialog(hDlg, res as INT_PTR);
                    return 1;
                }
                v if v == IDCANCEL as UINT => {
                    EndDialog(hDlg, 0);
                    return 1;
                }
                _ => {
                    if HIWORD(wParam as DWORD) as UINT != LBN_DBLCLK as UINT {
                    } else {
                        let hListBox: HWND = GetDlgItem(hDlg, MDI_IDC_LISTBOX as core::ffi::c_int);
                        let index: UINT = SendMessageW(hListBox, LB_GETCURSEL, 0, 0) as UINT;
                        let res: LRESULT = SendMessageW(hListBox, LB_GETITEMDATA, index as WPARAM, 0);
                        EndDialog(hDlg, res as INT_PTR);
                        return 1;
                    }
                }
            }
        }

        _ => {}
    }
    0
}

unsafe fn MDI_MoreWindowsDialog(hwnd: HWND) -> HWND {

    let hRes: HRSRC = FindResourceA(
        User32Instance,
        b"MDI_MOREWINDOWS\0".as_ptr() as LPCSTR,
        RT_DIALOG as LPCSTR,
    );

    if hRes.is_null() {
        return core::ptr::null_mut();
    }

    let hDlgTmpl = LoadResource(User32Instance, hRes);

    if hDlgTmpl.is_null() {
        return core::ptr::null_mut();
    }

    let template = LockResource(hDlgTmpl);

    if template.is_null() {
        return core::ptr::null_mut();
    }

    DialogBoxIndirectParamA(
        User32Instance,
        template,
        hwnd,
        MDI_MoreWindowsDlgProc as *mut core::ffi::c_void,
        hwnd as LPARAM,
    ) as HWND
}
